"""Owned, isolated smoke session for a frozen native foundation.

Reads JSON requests from stdin, records actual control replies, and gracefully
quits on EOF. Physical keyboard/visual observations are performed separately
through the computer-use API; ctl key/type do not establish that coverage.
Only generated singleplayer data is used. No launch on import.
"""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import uuid
import zipfile


def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    for name in ['client', 'control', 'resources', 'package', 'bundle', 'scratch']:
        p.add_argument('--'+name, type=Path, required=True)
    for name in ['client', 'control', 'package', 'script']:
        p.add_argument('--'+name+'-sha256', required=True)
    p.add_argument('--width', type=int, default=960)
    p.add_argument('--height', type=int, default=540)
    a = p.parse_args()
    for name in ['client','control','resources','package','bundle']:
        setattr(a, name, getattr(a,name).resolve(strict=True))
    expected = {name:getattr(a,name+'_sha256') for name in ['client','control','package','script']}
    paths = {'client':a.client,'control':a.control,'package':a.package,'script':Path(__file__).resolve()}
    def frozen():
        assert all(sha(paths[k])==v for k,v in expected.items()), 'Frozen input changed'
        files = {f.relative_to(a.bundle).as_posix():sha(f) for f in a.bundle.rglob('*') if f.is_file()}
        with zipfile.ZipFile(a.package) as z:
            assert len(z.namelist())==len(set(z.namelist()))
            assert set(z.namelist())==set(files)
            assert all(hashlib.sha256(z.read(k)).hexdigest()==v for k,v in files.items())
        return files
    before = frozen()
    home = a.scratch.resolve()/('foundation-smoke-'+uuid.uuid4().hex)
    home.mkdir(parents=True)
    env = os.environ.copy()
    names = ['HORIZON_HOME','HORIZON_LAUNCHER_HOME','XDG_DATA_HOME','XDG_CONFIG_HOME',
             'HORIZON_LAUNCHER_TRASH','HORIZON_LAUNCHER_CLIPBOARD','HORIZON_LAUNCHER_DESKTOP','TEMP','TMP']
    for k in names:
        d=home/k.lower(); d.mkdir(); env[k]=str(d)
    env.update(HORIZON_NO_AUDIO='1',HORIZON_NO_DISCORD='1',HORIZON_TELEMETRY_URL='off',HORIZON_LAUNCHER_NO_SELF_UPDATE='1')
    game_home=Path(env['HORIZON_HOME'])
    (game_home/'options.txt').write_text('version:5023\npauseOnLostFocus:false\nmaxFps:60\nfov:70\n')
    socket=home/'control.sock'
    report={'kind':'native-foundation-smoke','frozen':expected,'source_files':before,
            'started_utc':datetime.now(timezone.utc).isoformat(),'events':[],
            'isolation':{k:env[k] for k in names},
            'limits':['Scoped smoke observations, not validation of all registered modules.',
                      'ctl key/type bypass raw Winit keyboard input; physical coverage is separately observed.']}
    def save():
        (home/'report.json').write_text(json.dumps(report,indent=2)+'\n')
    def ctl(words):
        r=subprocess.run([str(a.control),'--socket',str(socket),*map(str,words)],cwd=game_home,env=env,
                         capture_output=True,text=True,encoding='utf8',timeout=60,creationflags=subprocess.CREATE_NO_WINDOW)
        assert r.returncode==0,(r.returncode,r.stdout,r.stderr)
        v=json.loads(r.stdout)
        assert 'error' not in v,v
        return v
    launch=[str(a.client),'--resources',str(a.resources),'--world','MeteorHorizonFoundationSmoke',
            '--seed','1','--test-terrain','--creative','--render-distance','2',
            '--width',str(a.width),'--height',str(a.height),'--control-socket',str(socket)]
    allowed={'state','ui','click','scroll-at','screenshot','wait','release','key','type','messages',
             'command','disconnect','open','look','mouse-move','quit'}
    with (home/'process.log').open('w',encoding='utf8') as log:
        game=subprocess.Popen(launch,cwd=game_home,env=env,stdout=log,stderr=log,creationflags=subprocess.CREATE_NO_WINDOW)
        report.update(pid=game.pid,launch=launch)
        report['processes']=[{'pid':game.pid,'viewport':[a.width,a.height]}]
        save()
        def ready():
            end=time.monotonic()+180
            while True:
                assert game.poll() is None,'Client exited during startup'
                try:
                    s=ctl(['state'])
                    if s.get('player') and s.get('screen')=='playing': return s
                except (AssertionError,subprocess.TimeoutExpired,json.JSONDecodeError): pass
                if time.monotonic()>end: raise TimeoutError('Generated world startup')
                time.sleep(.5)
        try:
            s=ready()
            print(json.dumps({'ready':True,'pid':game.pid,'home':str(home),'report':str(home/'report.json'),'state':s}),flush=True)
            for line in sys.stdin:
                request=json.loads(line)
                if request.get('finish'): break
                assert frozen()==before
                if 'ctl' in request:
                    words=request['ctl']; assert words and words[0] in allowed
                    if words[0]=='screenshot':
                        label=words[1]; assert label.replace('-','').replace('_','').isalnum()
                        words=['screenshot',str(home/(label+'.png'))]
                    result=ctl(words)
                elif 'restart' in request:
                    width,height=request['restart']
                    assert isinstance(width,int) and isinstance(height,int) and 320<=width<=3840 and 180<=height<=2160
                    f=game_home/'horizon-mods/.horizon-mod-data/meteor_client/config'
                    config=f.read_bytes() if f.exists() else None
                    ctl(['release']); ctl(['quit']); code=game.wait(timeout=30)
                    report['processes'][-1].update(exit_code=code,graceful_quit_requested=True)
                    assert code==0
                    command=list(launch)
                    command[command.index('--width')+1]=str(width)
                    command[command.index('--height')+1]=str(height)
                    game=subprocess.Popen(command,cwd=game_home,env=env,stdout=log,stderr=log,creationflags=subprocess.CREATE_NO_WINDOW)
                    report['processes'].append({'pid':game.pid,'viewport':[width,height]})
                    report['pid']=game.pid
                    s=ready()
                    assert (f.read_bytes() if f.exists() else None)==config,'Generated config changed on restart'
                    result={'pid':game.pid,'state':s,'config_byte_identical_after_restart':True,
                            'scope':'Fresh process/viewport in this generated home; not live resize.'}
                elif 'config' in request:
                    f=game_home/'horizon-mods/.horizon-mod-data/meteor_client/config'
                    result={'config':f.read_text() if f.exists() else None}
                elif 'observation' in request: result={'manual_observation':request['observation']}
                else: raise ValueError('Unsupported request')
                report['events'].append({'request':request,'result':result,'utc':datetime.now(timezone.utc).isoformat()})
                save(); print(json.dumps(result),flush=True)
                if request.get('ctl',[None])[0]=='quit': break
        except Exception as e:
            report['error']=repr(e); raise
        finally:
            if game.poll() is None:
                try: ctl(['release']); ctl(['quit']); report['graceful_quit_requested']=True
                except Exception as e: report['quit_error']=repr(e)
            try: report['exit_code']=game.wait(timeout=30)
            except subprocess.TimeoutExpired:
                report['exit_code']=None; report['shutdown_gap']='Client did not quit within30s; no forced termination.'
            report['frozen_unchanged']=frozen()==before
            report['processes'][-1].update(exit_code=report['exit_code'],graceful_quit_requested=report.get('graceful_quit_requested',False))
            save(); print(json.dumps({'report':str(home/'report.json'),'exit_code':report['exit_code']}),flush=True)


if __name__=='__main__':
    main()
