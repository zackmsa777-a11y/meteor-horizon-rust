"""Isolated native Meteor smoke tests. User approved future isolated port tests.
No multiplayer or existing saves; optional flags extend the tested modules.
"""
from pathlib import Path
import json
import os
import subprocess
import time
import uuid
import argparse
import hashlib
import shutil

parser = argparse.ArgumentParser()
parser.add_argument('--auto-clicker', action='store_true', help='Real isolated click cadence, screen filtering and held attack')
parser.add_argument('--macros-import', action='store_true', help='Original-layout generated NBT macro imports and queued-handle preservation')
parser.add_argument('--air-jump', action='store_true', help='Include isolated AirJump checks')
parser.add_argument('--sprint', action='store_true', help='Include isolated Sprint checks')
parser.add_argument('--gui-move', action='store_true', help='Include isolated GUIMove checks')
parser.add_argument('--high-jump', action='store_true', help='Include isolated HighJump checks')
parser.add_argument('--native-gui', action='store_true', help='Include isolated native GUI/HUD checks')
parser.add_argument('--walk', action='store_true', help='Include isolated AutoWalk/SafeWalk checks')
parser.add_argument('--step', action='store_true', help='Include isolated Step/ReverseStep checks')
parser.add_argument('--climb', action='store_true', help='Include isolated FastClimb/Spider/Parkour/Timer checks')
parser.add_argument('--friends', action='store_true', help='Include isolated native Friends GUI checks')
parser.add_argument('--nbt-import', action='store_true', help='Include generated-fixture NBT import checks')
parser.add_argument('--player-actions', action='store_true', help='Include isolated AutoRespawn/BreakDelay/FastUse checks')
parser.add_argument('--fullbright', action='store_true', help='Include isolated Gamma/Potion/Luminance checks')
parser.add_argument('--slow-movement', action='store_true', help='Isolated Sneak/NoSlow/Slippy movement comparisons')
parser.add_argument('--profiles', action='store_true', help='Generated native profiles and live module/HUD restoration')
parser.add_argument('--macros', action='store_true', help='Generated Starscript macros dispatched only in the isolated world')
parser.add_argument('--system-guis', action='store_true', help='Native Macros/Profiles editor interaction checks')
parser.add_argument('--reach-multitask', action='store_true', help='Real chest reach and simultaneous-use/mining/attack checks')
parser.add_argument('--flight', action='store_true', help='Actual Flight modes, movement, abilities and outgoing packets')
parser.add_argument('--picking', action='store_true', help='Real AirPlace, LiquidInteract, NoMiningTrace placement/render/mining checks')
parser.add_argument('--host-gui', action='store_true', help='Real Starscript host bindings and RGBA/rainbow GUI controls')
parser.add_argument('--source-root', type=Path, help='Horizon source tree containing target/fast binaries')
parser.add_argument('--resources', type=Path, help='Locally prepared Minecraft resources (not distributed)')
parser.add_argument('--client-binary', type=Path, help='Test an exact installed native client instead of target/fast')
parser.add_argument('--control-binary', type=Path, help='Control CLI used with an explicitly selected client')
parser.add_argument('--hud-instances', action='store_true', help='Generated repeated TextHud and real settings/physical placement checks')
args = parser.parse_args()

workspace = Path(__file__).resolve().parent.parent
source_root = args.source_root or (workspace if (workspace / 'Cargo.toml').exists() else workspace / 'work/horizon-source/horizon-main')
binary_dir = source_root / 'target/fast'
client_binary = (args.client_binary or binary_dir / 'horizon-client.exe').resolve()
control_binary = (args.control_binary or binary_dir / 'horizon-ctl.exe').resolve()
resources = args.resources or (source_root / '.local/horizon-client/26.3')
if not resources.is_dir():
    raise SystemExit('Pass --resources with the locally prepared Minecraft resource directory.')
scratch = workspace / 'work' / ('meteor-runtime-' + uuid.uuid4().hex)
scratch.mkdir(parents=True)
home = scratch / 'game'
home.mkdir()
socket = scratch / 'control.sock'
env = os.environ.copy()
isolation_folders = {
    'HORIZON_HOME': 'game', 'HORIZON_LAUNCHER_HOME': 'launcher', 'XDG_DATA_HOME': 'data',
    'XDG_CONFIG_HOME': 'config', 'HORIZON_LAUNCHER_TRASH': 'trash',
    'HORIZON_LAUNCHER_CLIPBOARD': 'clipboard', 'HORIZON_LAUNCHER_DESKTOP': 'desktop',
}
for key, folder in isolation_folders.items(): env[key] = str(scratch / folder)
env.update(HORIZON_LAUNCHER_NO_SELF_UPDATE='1', HORIZON_NO_AUDIO='1', HORIZON_NO_DISCORD='1', HORIZON_TELEMETRY_URL='off')
(home / 'options.txt').write_text('version:5023\npauseOnLostFocus:false\nmaxFps:60\n' + ('key_key.jump:key.keyboard.j\n' if args.air_jump else ''), encoding='utf-8')
report = {'scratch': str(scratch), 'checks': [], 'states': {}, 'runtime_approved': True,
          'binary_sha256': hashlib.sha256(client_binary.read_bytes()).hexdigest()}
report['modules'] = ['Zoom', 'AutoJump'] + (['AirJump'] if args.air_jump else []) + (['Sprint'] if args.sprint else []) + (['GUIMove'] if args.gui_move else [])
if args.high_jump: report['modules'].append('HighJump')
if args.native_gui: report['modules'] += ['ClickGUI', 'HUD']
if args.walk: report['modules'] += ['AutoWalk', 'SafeWalk']
if args.step: report['modules'] += ['Step', 'ReverseStep']
if args.climb: report['modules'] += ['FastClimb', 'Spider', 'Parkour', 'Timer']
if args.friends: report['modules'] += ['Friends']
if args.nbt_import: report['modules'] += ['NBTImport']
if args.player_actions: report['modules'] += ['AutoRespawn', 'BreakDelay', 'FastUse']
if args.fullbright: report['modules'] += ['Fullbright']
if args.slow_movement: report['modules'] += ['Sneak', 'NoSlow', 'Slippy']
if args.profiles: report['modules'] += ['Profiles']
if args.macros: report['modules'] += ['Macros', 'Starscript']
if args.system_guis: report['modules'] += ['MacrosGUI', 'ProfilesGUI']
if args.reach_multitask: report['modules'] += ['Reach', 'Multitask']
if args.auto_clicker: report['modules'] += ['AutoClicker']
if args.macros_import: report['modules'] += ['MacrosNBTImport']
if args.flight: report['modules'] += ['Flight']
if args.picking: report['modules'] += ['AirPlace', 'LiquidInteract', 'NoMiningTrace']
if args.host_gui: report['modules'] += ['StarscriptHost', 'ColorGUI']
if args.hud_instances: report['modules'] += ['TextHUD', 'HUDInstances', 'HudBox']
creationflags = subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0

def ctl(*args, timeout=45):
    # The generic control `tp` edits only the client position. Once an integrated server
    # is running it can correct a reset across the test obstacle back to the old position.
    # Teleport the authoritative player so fixture resets cannot masquerade as physics.
    if args and args[0] == 'tp':
        args = ('command', 'tp @s ' + ' '.join(map(str, args[1:])))
    result = subprocess.run([str(control_binary), '--socket', str(socket), *map(str, args)],
        cwd=home, env=env, capture_output=True, text=True, encoding='utf-8', timeout=timeout,
        creationflags=creationflags)
    if result.returncode: raise RuntimeError(f'{args}: {result.stdout} {result.stderr}')
    data = json.loads(result.stdout)
    if data.get('error'): raise RuntimeError(f'{args}: {data}')
    return data

def check(name, condition, detail=None):
    report['checks'].append({'name': name, 'passed': bool(condition), 'detail': detail})
    print(f'{name}: {"PASS" if condition else "FAIL"}', flush=True)
    if not condition: raise AssertionError(name)

def state(name):
    value = ctl('state')
    report['states'][name] = value
    return value

log = (scratch / 'process.log').open('w', encoding='utf-8')
game = subprocess.Popen([
    str(client_binary), '--resources',
    str(resources),
    '--world', 'MeteorRuntimeTest', '--seed', '1', '--creative', '--test-terrain',
    '--render-distance', '2', '--offscreen', '--width', '960', '--height', '540',
    '--control-socket', str(socket),
], cwd=home, env=env, stdout=log, stderr=log, creationflags=creationflags)
report['pid'] = game.pid
print(f'Runtime PID {game.pid}; scratch {scratch}', flush=True)
(workspace / 'work/meteor-runtime-current.json').write_text(json.dumps(report), encoding='utf-8')
try:
    deadline = time.monotonic() + 180
    last_progress = 0
    while True:
        if game.poll() is not None: raise RuntimeError(f'Game exited {game.returncode}; see {scratch / "process.log"}')
        try:
            current = ctl('state', timeout=5)
            if current.get('player') and current.get('screen') == 'playing': break
        except (RuntimeError, subprocess.TimeoutExpired, json.JSONDecodeError): pass
        if time.monotonic() > deadline: raise TimeoutError('World startup did not finish in 180 seconds')
        if time.monotonic() - last_progress > 20:
            print('Waiting for the isolated world to load...', flush=True)
            last_progress = time.monotonic()
        time.sleep(1)
    ctl('wait', 40)
    baseline = state('baseline')
    check('Meteor adapter is present', isinstance(baseline.get('meteor'), dict))
    check('Baseline zoom factor is one', baseline['meteor']['zoom']['factor'] == 1)
    ctl('screenshot', scratch / 'baseline.png')
    ctl('command', 'meteor toggle zoom')
    ctl('wait', 10)
    zoom = state('zoom')
    check('Zoom reaches factor six', abs(zoom['meteor']['zoom']['factor'] - 6) < 1e-5)
    check('Sensitivity reduced before mouse curve', abs(zoom['meteor']['zoom']['sensitivity'] - baseline['meteor']['zoom']['sensitivity'] / 3) < 1e-6)
    check('Default Zoom hides hands', zoom['meteor']['zoom']['hide_hands'])
    ctl('screenshot', scratch / 'zoom.png')
    ctl('command', 'meteor set zoom hide-HUD true')
    ctl('wait', 2)
    check('Zoom HUD option reaches adapter', state('hidden_hud')['meteor']['zoom']['hide_hud'])
    ctl('command', 'meteor set zoom cinematic true')
    ctl('wait', 2)
    check('Cinematic camera reaches mouse options', state('cinematic')['meteor']['zoom']['cinematic'])
    ctl('command', 'meteor toggle zoom')
    ctl('wait', 10)
    restored = state('restored')
    check('Zoom fade-out restores factor', restored['meteor']['zoom']['factor'] == 1)
    check('Sensitivity restored', restored['meteor']['zoom']['sensitivity'] == baseline['meteor']['zoom']['sensitivity'])
    check('HUD and hands restored', not restored['meteor']['zoom']['hide_hud'] and not restored['meteor']['zoom']['hide_hands'])
    # Build a flat platform in the isolated test world, then observe real motion without input.
    position = restored['player']['position']
    x, y, z = int(position[0] // 1), int(position[1] // 1), int(position[2] // 1)
    ctl('command', f'fill {x-2} {y-1} {z-2} {x+2} {y-1} {z+2} minecraft:stone')
    ctl('tp', x+0.5, y, z+0.5)
    ctl('wait', 20)
    before_jump = state('before_auto_jump')
    check('Player settled on ground', before_jump['player']['on_ground'])
    ctl('command', 'meteor toggle auto-jump')
    samples = []
    for i in range(20):
        ctl('wait', 2)
        samples.append(ctl('state')['player'])
    report['auto_jump_samples'] = samples
    # Control round-trips can repeatedly land after the jump apex. Increasing measured height
    # proves upward travel even if none of the instantaneous velocity samples caught the ascent.
    heights = [before_jump['player']['position'][1]] + [p['position'][1] for p in samples]
    check('AutoJump produces upward motion without jump input', any(b > a + 0.01 for a, b in zip(heights, heights[1:])))
    check('AutoJump raises player above the platform', max(p['position'][1] for p in samples) > before_jump['player']['position'][1] + 1.0)
    check('AutoJump leaves ground', any(not p['on_ground'] for p in samples))
    ctl('command', 'meteor toggle auto-jump')
    ctl('wait', 40)
    check('Disabling AutoJump returns to rest', state('auto_jump_off')['player']['on_ground'])
    if args.air_jump:
        ctl('command', 'gamemode survival')
        ctl('command', 'effect give @s minecraft:slow_falling 60 0 true')
        ctl('command', 'meteor toggle air-jump')
        # A rebound physical J key must jump while airborne.
        ctl('tp', x+0.5, y+8, z+0.5)
        ctl('wait', 2)
        airborne = state('airborne_before_press')
        check('AirJump fixture is airborne', not airborne['player']['on_ground'])
        ctl('key', 'j', 'down')
        after_press = state('air_jump_press')
        ctl('key', 'j', 'up')
        check('Rebound jump key creates upward motion in air', after_press['player']['velocity'][1] > 0)
        check('AirJump captures press height', after_press['meteor']['air_jump_level'] >= y+6)
        ctl('command', 'meteor set air-jump maintain-level true')
        ctl('key', 'j', 'down')
        maintained = []
        for i in range(16):
            ctl('wait', 3)
            maintained.append(ctl('state'))
        report['air_jump_samples'] = maintained
        level = maintained[0]['meteor']['air_jump_level']
        check('Held rebound jump maintains captured level', all(s['player']['position'][1] >= level for s in maintained))
        ctl('key', 'j', 'up')
        ctl('command', 'meteor toggle air-jump')
        check('Disabling AirJump clears activation height', state('air_jump_off')['meteor']['air_jump_level'] is None)
        ctl('command', 'effect clear @s')
        ctl('tp', x+0.5, y, z+0.5)
        ctl('wait', 20)
        check('AirJump fixture returns to ground', state('air_jump_grounded')['player']['on_ground'])
    if args.sprint:
        import math
        ctl('command', 'gamemode survival')
        ctl('command', f'fill {x-16} {y-1} {z-16} {x+16} {y-1} {z+16} minecraft:stone')
        ctl('command', f'fill {x-16} {y} {z-16} {x+16} {y+4} {z+16} minecraft:air')
        def movement_sample(name, key):
            ctl('release')
            ctl('tp', x+0.5, y, z+0.5)
            ctl('wait', 10)
            ctl('key', key, 'down')
            ctl('wait', 10)
            result = state(name)
            ctl('key', key, 'up')
            return result
        def horizontal_speed(value):
            v = value['player']['velocity']
            return math.hypot(v[0], v[2])
        normal = movement_sample('sprint_disabled_forward', 'w')
        check('Baseline forward movement does not sprint', not normal['player']['sprinting'])
        ctl('command', 'meteor toggle sprint')
        strict = movement_sample('sprint_strict_forward', 'w')
        check('Strict Sprint starts without sprint-key input', strict['player']['sprinting'])
        check('Strict Sprint increases actual movement speed', horizontal_speed(strict) > horizontal_speed(normal) * 1.15)
        sideways = movement_sample('sprint_strict_sideways', 'a')
        check('Strict Sprint rejects sideways-only movement', not sideways['player']['sprinting'])
        ctl('command', 'meteor set sprint sprint-mode Rage')
        rage = movement_sample('sprint_rage_sideways', 'a')
        check('Rage Sprint supports sideways movement', rage['player']['sprinting'])
        check('Rage Sprint increases sideways speed', horizontal_speed(rage) > horizontal_speed(sideways) * 1.15)
        ctl('command', 'meteor set sprint sprint-while-stationary true')
        ctl('wait', 5)
        check('Rage stationary option keeps sprinting without movement', state('sprint_stationary')['player']['sprinting'])
        ctl('command', 'meteor toggle sprint')
        ctl('wait', 5)
        check('Disabling Sprint restores vanilla stationary behavior', not state('sprint_off')['player']['sprinting'])
    if args.gui_move:
        import math
        ctl('command', 'gamemode survival')
        ctl('command', f'fill {x-16} {y-1} {z-16} {x+16} {y-1} {z+16} minecraft:stone')
        ctl('command', f'fill {x-16} {y} {z-16} {x+16} {y+4} {z+16} minecraft:air')
        ctl('tp', x+0.5, y, z+0.5)
        ctl('wait', 10)
        ctl('key', 'e', 'tap')
        check('GUIMove fixture opens inventory', state('gui_inventory')['screen'] == 'inventory')
        ctl('key', 'w', 'down')
        ctl('wait', 10)
        disabled = state('gui_disabled')
        ctl('key', 'w', 'up')
        def moving(s):
            v = s['player']['velocity']
            return math.hypot(v[0], v[2]) > 0.05
        check('Inventory normally blocks movement', not moving(disabled))
        ctl('command', 'meteor toggle gui-move')
        ctl('key', 'w', 'down')
        ctl('wait', 10)
        check('GUIMove allows inventory movement', moving(state('gui_enabled')))
        ctl('screenshot', scratch / 'gui-move.png')
        ctl('release')
        ctl('wait', 10)
        check('Global input release clears GUIMove movement', not moving(state('gui_global_release')))
        ctl('key', 'w', 'down')
        ctl('wait', 10)
        check('GUIMove accepts a fresh press after global release', moving(state('gui_repressed')))
        ctl('key', 'w', 'up')
        ctl('wait', 10)
        check('GUIMove releases movement', not moving(state('gui_released')))
        before = state('gui_before_rotation')['player']['yaw']
        ctl('key', 'right', 'down')
        ctl('wait', 5)
        ctl('key', 'right', 'up')
        check('Inventory arrow key rotates view', abs(state('gui_rotated')['player']['yaw'] - before) > 1)
        ctl('command', 'meteor set gui-move guis GUI')
        ctl('key', 'w', 'down')
        ctl('wait', 10)
        check('GUI-only setting excludes vanilla inventory', not moving(state('gui_excluded')))
        ctl('key', 'w', 'up')
        ctl('command', 'meteor set gui-move guis Inventory')
        ctl('command', 'meteor toggle gui-move')
        ctl('key', 'e', 'tap')
        check('GUIMove fixture closes inventory', state('gui_closed')['screen'] == 'playing')
    if args.high_jump:
        ctl('command', 'gamemode survival')
        ctl('command', 'effect clear @s')
        ctl('command', f'fill {x-2} {y-1} {z-2} {x+2} {y-1} {z+2} minecraft:stone')
        ctl('command', f'fill {x-2} {y} {z-2} {x+2} {y+12} {z+2} minecraft:air')
        jump_key = 'j' if args.air_jump else 'space'
        def jump_peak(name):
            ctl('release')
            ctl('tp', x+0.5, y, z+0.5)
            ctl('wait', 10)
            initial = state(name + '_before')
            check(name + ' starts grounded', initial['player']['on_ground'])
            ctl('key', jump_key, 'down')
            ctl('wait', 2)
            samples = [ctl('state')['player']]
            ctl('key', jump_key, 'up')
            for i in range(18):
                ctl('wait', 1)
                samples.append(ctl('state')['player'])
            report[name + '_samples'] = samples
            return max(p['position'][1] for p in samples) - initial['player']['position'][1]
        ordinary = jump_peak('high_jump_disabled')
        check('Baseline jump rises normally', ordinary > 0.4)
        ctl('command', 'meteor set high-jump jump-multiplier 2')
        ctl('command', 'meteor toggle high-jump')
        boosted = jump_peak('high_jump_double')
        check('HighJump multiplier increases measured jump height', boosted > ordinary * 2 and boosted > 3)
        ctl('command', 'meteor set high-jump jump-multiplier 0')
        zero = jump_peak('high_jump_zero')
        check('Zero multiplier removes normal vertical jump', zero < 0.1)
        ctl('command', 'meteor toggle high-jump')
        restored = jump_peak('high_jump_restored')
        check('Disabling HighJump restores normal jump', 0.4 < restored < 1.4)
        report['high_jump_peaks'] = {'disabled': ordinary, 'double': boosted, 'zero': zero, 'restored': restored}
    if args.native_gui:
        ctl('release')
        ctl('key', 'rshift', 'tap')
        ctl('wait', 3)
        check('Right Shift opens native GUI', state('native_gui_open')['meteor']['gui']['open'])
        ctl('screenshot', scratch / 'native-gui.png')
        ctl('type', 'zoom')
        check('GUI search accepts text', state('native_gui_search')['meteor']['gui']['query'] == 'zoom')
        ctl('screenshot', scratch / 'native-gui-search.png')
        def gui_module(module):
            return next(m for m in ctl('state')['meteor']['config']['modules']['modules'] if m['id'] == module)
        ctl('meteor-cursor', 30, 88)
        ctl('mouse', 'left', 'click')
        check('GUI left click toggles selected module', gui_module('zoom')['active'])
        ctl('mouse', 'right', 'click')
        ctl('wait', 2)
        check('GUI right click opens module settings', state('native_gui_settings')['meteor']['gui']['selected'] == 'zoom')
        ctl('screenshot', scratch / 'native-gui-settings.png')
        ctl('meteor-cursor', 200, 60)
        ctl('mouse', 'left', 'click')
        check('Settings enabled row toggles module off', not gui_module('zoom')['active'])
        ctl('meteor-cursor', 200, 80)
        ctl('mouse', 'left', 'click')
        ctl('key', 'c', 'tap')
        check('GUI captures keyboard binding', gui_module('zoom')['keybind']['key'] == 'key.keyboard.c')
        ctl('meteor-cursor', 200, 80)
        ctl('mouse', 'left', 'click')
        ctl('key', 'ctrl', 'down')
        ctl('key', 'c', 'tap')
        ctl('key', 'ctrl', 'up')
        check('GUI captures keyboard modifier chord', gui_module('zoom')['keybind'] == {'key':'key.keyboard.c', 'modifiers':2})
        ctl('command', 'meteor bind zoom none')
        ctl('key', 'esc', 'tap')
        ctl('wait', 2)
        ctl('meteor-cursor', 130, 12)
        ctl('mouse', 'left', 'click')
        ctl('wait', 3)
        editor = state('hud_editor')
        check('HUD button opens editor', editor['meteor']['gui']['hud_editor'])
        check('HUD defaults disabled', not editor['meteor']['config']['hud']['active'])
        ctl('meteor-cursor', 120, 12)
        ctl('mouse', 'left', 'click')
        ctl('wait', 2)
        check('HUD editor enables overlay', state('hud_enabled')['meteor']['config']['hud']['active'])
        ctl('screenshot', scratch / 'hud-editor.png')
        # Editor viewport keeps the two top-left elements individually selectable.
        initial_offset = state('hud_before_drag')['meteor']['config']['hud']['elements'][0]['offset']
        ctl('meteor-cursor', 10, 36)
        ctl('mouse', 'left', 'down')
        ctl('meteor-cursor', 50, 66)
        ctl('mouse', 'left', 'up')
        moved_offset = state('hud_after_drag')['meteor']['config']['hud']['elements'][0]['offset']
        check('HUD watermark drag persists its offset', moved_offset == [initial_offset[0] + 40, initial_offset[1] + 30])
        ctl('wait', 2)
        ctl('screenshot', scratch / 'hud-editor-positioned.png')
        ctl('key', 'esc', 'tap')
        check('Escape returns from HUD to modules', not state('hud_back')['meteor']['gui']['hud_editor'])
        ctl('key', 'rshift', 'tap')
        ctl('wait', 3)
        check('Right Shift closes native GUI', not state('native_gui_closed')['meteor']['gui']['open'])
        ctl('screenshot', scratch / 'native-hud.png')
    if args.walk:
        import math
        def active(module):
            return next(m['active'] for m in ctl('state')['meteor']['config']['modules']['modules'] if m['id'] == module)
        ctl('release')
        ctl('command', 'gamemode survival')
        ctl('command', f'fill {x-8} {y-1} {z-8} {x+8} {y-1} {z+8} minecraft:stone')
        ctl('command', f'fill {x-8} {y} {z-8} {x+8} {y+10} {z+8} minecraft:air')
        ctl('tp', x+0.5, y, z+0.5)
        ctl('look', 0, 0)
        ctl('wait', 8)
        start = state('auto_walk_before')['player']['position']
        ctl('command', 'meteor set auto-walk mode Simple')
        ctl('command', 'meteor toggle auto-walk')
        ctl('wait', 12)
        walked = state('auto_walk_forward')['player']['position']
        check('AutoWalk Simple moves without held keys', walked[2] > start[2] + 0.5)
        ctl('command', 'meteor set auto-walk disable-on-input true')
        ctl('key', 'a', 'down')
        check('Manual movement disables AutoWalk', not active('auto-walk'))
        ctl('release')
        ctl('wait', 10)
        velocity = state('auto_walk_off')['player']['velocity']
        check('AutoWalk releases forced movement', math.hypot(velocity[0], velocity[2]) < 0.05)
        ctl('command', 'meteor set auto-walk mode Smart')
        ctl('command', 'meteor toggle auto-walk')
        ctl('wait', 4)
        check('Missing path manager disables Smart mode explicitly', not active('auto-walk'))
        # Raised platform over clear air: same movement input with/without ledge clipping.
        top = y + 6
        ctl('command', f'fill {x-4} {y} {z-4} {x+4} {top+3} {z+8} minecraft:air')
        ctl('command', f'fill {x-2} {top-1} {z-2} {x+2} {top-1} {z+2} minecraft:stone')
        ctl('tp', x+0.5, top, z+0.5)
        ctl('wait', 8)
        ctl('command', 'meteor toggle safe-walk')
        ctl('command', 'meteor set safe-walk render true')
        ctl('command', 'meteor set safe-walk render-player-box true')
        ctl('key', 'w', 'down')
        ctl('wait', 35)
        edge = state('safe_walk_edge')['player']
        check('SafeWalk keeps player on platform', edge['position'][1] >= top - 0.1 and edge['position'][2] < z+3.4)
        ctl('release')
        # Upstream renders only sneak-mode bounds; default edge-distance=.3 nearly
        # collapses the red horizontal rectangle, so choose a visible .1 inset.
        ctl('command', 'meteor set safe-walk sneak true')
        ctl('command', 'meteor set safe-walk edge-distance 0.1')
        ctl('key', 'f5', 'tap')
        ctl('look', 0, 30)
        ctl('wait', 2)
        ctl('screenshot', scratch / 'safe-walk-outline.png')
        ctl('key', 'f5', 'tap')
        ctl('key', 'f5', 'tap')
        ctl('look', 0, 0)
        ctl('command', 'meteor set safe-walk sneak false')
        ctl('command', 'meteor toggle safe-walk')
        ctl('key', 'w', 'down')
        ctl('wait', 18)
        fallen = state('safe_walk_disabled')['player']
        check('Disabling SafeWalk permits leaving edge', fallen['position'][1] < top - 0.5)
        ctl('release')
    if args.step:
        ctl('release')
        ctl('command', 'gamemode survival')
        ctl('command', 'effect clear @s')
        # Keep the stepped height occupied across delayed control samples under build load.
        # The disabled control must still stop at the same full-block leading face.
        ctl('command', f'fill {x-5} {y-1} {z-5} {x+5} {y-1} {z+60} minecraft:stone')
        ctl('command', f'fill {x-5} {y} {z-5} {x+5} {y+8} {z+60} minecraft:air')
        ctl('command', f'fill {x-3} {y} {z+2} {x+3} {y} {z+60} minecraft:stone')
        def walk_over_step(label):
            ctl('release')
            ctl('tp', x+0.5, y, z+0.5)
            ctl('look', 0, 0)
            ctl('wait', 8)
            reset = state(label + '_reset')['player']['position']
            check(label + ' fixture reset reaches start', abs(reset[0]-(x+0.5)) < 0.1 and abs(reset[2]-(z+0.5)) < 0.1)
            ctl('key', 'w', 'down')
            samples = []
            for _ in range(9):
                ctl('wait', 3)
                samples.append(ctl('state')['player'])
            ctl('release')
            report[label + '_samples'] = samples
            return samples
        ordinary = walk_over_step('step_disabled')
        check('Ordinary walking cannot climb full block', max(p['position'][2] for p in ordinary) < z+2)
        ctl('command', 'meteor toggle step')
        stepped = walk_over_step('step_enabled')
        check('Step climbs full block without jumping', max(p['position'][1] for p in stepped) > y+0.8 and stepped[-1]['position'][2] > z+3)
        ctl('command', 'meteor set step safe-step true')
        safe = walk_over_step('step_safe_no_crystals')
        check('SafeStep permits safe ascent with no crystals', safe[-1]['position'][2] > z+3)
        ctl('command', 'meteor toggle step')
        restored = walk_over_step('step_restored')
        check('Disabling Step restores ordinary height', max(p['position'][2] for p in restored) < z+2)
        ctl('command', f'fill {x-3} {y} {z+2} {x+3} {y} {z+60} minecraft:air')
        # A three-block drop: probe intersects the real lower floor and writes the source's
        # direct downward velocity during Tick.Post, before the next movement tick.
        top = y+3
        ctl('command', f'fill {x-2} {y} {z-2} {x+2} {top-1} {z+1} minecraft:stone')
        ctl('tp', x+0.5, top, z+0.5)
        ctl('wait', 8)
        ctl('command', 'meteor toggle reverse-step')
        ctl('key', 'w', 'down')
        reverse = []
        for _ in range(8):
            ctl('wait', 1)
            reverse.append(ctl('state')['player'])
        ctl('release')
        report['reverse_step_samples'] = reverse
        check('ReverseStep writes configured downward speed', any(p['velocity'][1] <= -2.9 for p in reverse))
        ctl('command', 'meteor toggle reverse-step')
        ctl('tp', x+0.5, top, z+0.5)
        ctl('wait', 8)
        ctl('key', 'w', 'down')
        ctl('wait', 1)
        check('Disabling ReverseStep restores normal vertical velocity', state('reverse_step_off')['player']['velocity'][1] > -1)
        ctl('release')
    if args.climb:
        ctl('release')
        ctl('command', 'gamemode survival')
        ctl('command', f'fill {x-4} {y-1} {z-4} {x+4} {y-1} {z+6} minecraft:stone')
        ctl('command', f'fill {x-4} {y} {z-4} {x+4} {y+30} {z+6} minecraft:air')
        ctl('command', f'fill {x-2} {y} {z+2} {x+2} {y+25} {z+2} minecraft:stone')
        ctl('tp', x+0.5, y, z+0.5)
        ctl('look', 0, 0)
        ctl('wait', 8)
        ctl('key', 'w', 'down')
        ctl('wait', 15)
        check('Ordinary wall collision does not climb', state('spider_baseline')['player']['position'][1] < y+0.1)
        ctl('command', 'meteor toggle spider')
        ctl('wait', 15)
        check('Spider climbs solid wall', state('spider_enabled')['player']['position'][1] > y+0.8)
        ctl('command', 'meteor toggle spider')
        ctl('release')
        ctl('tp', x+0.5, y, z+0.5)
        ctl('wait', 8)
        ctl('key', 'w', 'down')
        ctl('wait', 15)
        check('Disabling Spider restores wall collision', state('spider_disabled')['player']['position'][1] < y+0.1)
        ctl('release')
        ctl('command', f'fill {x} {y} {z+1} {x} {y+25} {z+1} minecraft:ladder[facing=north]')
        ctl('tp', x+0.5, y, z+1.5)
        ctl('wait', 8)
        ctl('command', 'meteor set fast-climb climb-speed 0.5')
        ctl('command', 'meteor toggle fast-climb')
        ctl('key', 'w', 'down')
        ctl('wait', 8)
        climb = state('fast_climb_speed')['player']
        check('FastClimb applies configured ladder speed', abs(climb['velocity'][1]-0.5) < 0.01 and climb['position'][1] > y+0.5)
        def tick_rate(label):
            first = ctl('state')
            before = time.monotonic()
            time.sleep(1.5)
            last = state(label)
            return (last['ticks'] - first['ticks']) / (time.monotonic() - before)
        ctl('command', 'meteor set fast-climb timer-mode true')
        ctl('command', 'meteor set fast-climb timer 2')
        timer_climb = tick_rate('fast_climb_timer')
        check('FastClimb timer changes client tick pacing', timer_climb > 30)
        ctl('release')
        ctl('tp', x+0.5, y, z-1.5)
        ctl('wait', 6)
        ordinary_rate = tick_rate('fast_climb_timer_released')
        check('Leaving ladder clears FastClimb timer override', 12 < ordinary_rate < 28)
        ctl('command', 'meteor toggle fast-climb')
        ctl('command', 'meteor set timer multiplier 2')
        ctl('command', 'meteor toggle timer')
        boosted_rate = tick_rate('timer_enabled')
        check('Timer doubles observed client pacing', boosted_rate > ordinary_rate * 1.5)
        ctl('command', 'meteor toggle timer')
        restored_rate = tick_rate('timer_disabled')
        check('Disabling Timer restores client pacing', 12 < restored_rate < 28)
        report['timer_rates'] = {'fast_climb':timer_climb, 'normal':ordinary_rate, 'enabled':boosted_rate, 'restored':restored_rate}
        ctl('command', f'fill {x-4} {y} {z-4} {x+4} {y+30} {z+6} minecraft:air')
        top = y+4
        ctl('command', f'fill {x-2} {top-1} {z-2} {x+2} {top-1} {z+2} minecraft:stone')
        def ledge_walk(label):
            ctl('release')
            ctl('tp', x+0.5, top, z+0.5)
            ctl('wait', 8)
            ctl('key', 'w', 'down')
            samples = []
            for _ in range(16):
                ctl('wait', 1)
                samples.append(ctl('state')['player'])
            ctl('release')
            report[label + '_samples'] = samples
            return samples
        normal_ledge = ledge_walk('parkour_disabled')
        check('Ordinary ledge walk does not jump', max(p['position'][1] for p in normal_ledge) <= top+0.1)
        ctl('command', 'meteor set parkour edge-distance 0.1')
        ctl('command', 'meteor toggle parkour')
        parkour = ledge_walk('parkour_enabled')
        check('Parkour jumps automatically at ledge', max(p['position'][1] for p in parkour) > top+0.5)
        ctl('command', 'meteor toggle parkour')
    if args.friends:
        ctl('release')
        ctl('key', 'rshift', 'tap')
        ctl('wait', 2)
        ctl('meteor-cursor', 185, 12)
        ctl('mouse', 'left', 'click')
        ctl('wait', 2)
        check('Friends button opens native page', state('friends_open')['meteor']['gui']['friends']['open'])
        ctl('type', 'NativeTestFriend')
        ctl('key', 'enter', 'tap')
        friends = state('friends_added')['meteor']['config']['friends']['friends']
        check('Friends GUI adds offline name', len(friends) == 1 and friends[0]['name'] == 'NativeTestFriend' and friends[0]['id'] is None)
        ctl('type', 'nativetestfriend')
        ctl('key', 'enter', 'tap')
        duplicate = state('friends_duplicate')['meteor']
        check('Friends rejects case-insensitive duplicate', len(duplicate['config']['friends']['friends']) == 1 and bool(duplicate['gui']['friends']['error']))
        ctl('screenshot', scratch / 'friends.png')
        ctl('meteor-cursor', 470, 43)
        ctl('mouse', 'left', 'click')
        check('Friends Remove button updates real list', state('friends_removed')['meteor']['config']['friends']['friends'] == [])
        ctl('key', 'esc', 'tap')
        check('Friends Escape returns to modules', not state('friends_back')['meteor']['gui']['friends']['open'])
        ctl('key', 'rshift', 'tap')
    if args.nbt_import:
        import struct
        # Small original-format fixtures generated inside this isolated test home.
        # Never read the user's real Meteor settings or profiles.
        def nbt_string(value):
            raw = value.encode('utf-8')
            return struct.pack('>H', len(raw)) + raw
        def nbt_payload(kind, value):
            if kind == 1: return struct.pack('>b', value)
            if kind == 6: return struct.pack('>d', value)
            if kind == 8: return nbt_string(value)
            if kind == 9:
                element, values = value
                return bytes([element]) + struct.pack('>i', len(values)) + b''.join(nbt_payload(element, v) for v in values)
            if kind == 10:
                return b''.join(bytes([t]) + nbt_string(k) + nbt_payload(t, v) for k, (t, v) in value.items()) + b'\0'
            raise ValueError(kind)
        def nbt_file(name, root):
            path = scratch / name
            path.write_bytes(b'\x0a\0\0' + nbt_payload(10, root))
            return path
        settings = [
            {'name':(8,'zoom'), 'value':(6,4.5)},
            {'name':(8,'smooth'), 'value':(1,0)},
        ]
        zoom_module = {'name':(8,'zoom'), 'active':(1,1),
            'keybind':(10,{'key':(8,'key.keyboard.k'), 'modifiers':(9,(8,['CONTROL']))}),
            'settings':(10,{'groups':(9,(10,[{'name':(8,'General'), 'settings':(9,(10,settings))}]))})}
        path = nbt_file('generated modules.nbt', {'modules':(9,(10,[zoom_module,{'name':(8,'unported-fixture-module')}]))})
        ctl('command', f'meteor import modules "{path}"')
        ctl('wait', 4)
        imported = state('nbt_modules_imported')['meteor']
        check('NBT module import reaches live Zoom behavior', abs(imported['zoom']['factor']-4.5) < 0.01)
        zoom_module_state = next(m for m in imported['config']['modules']['modules'] if m['id'] == 'zoom')
        check('NBT keybind imports modifier and key', zoom_module_state['keybind'] == {'key':'key.keyboard.k','modifiers':2})
        check('Unported NBT module is not registered', not any(m['id'] == 'unported-fixture-module' for m in imported['config']['modules']['modules']))
        friends_file = nbt_file('generated friends.nbt', {'friends':(9,(10,[{'name':(8,'ImportedOffline')},{'name':(8,'importedoffline')}]))})
        ctl('command', f'meteor import friends "{friends_file}"')
        friends_import = state('nbt_friends_imported')['meteor']
        check('NBT friends import deduplicates offline names', len(friends_import['config']['friends']['friends']) == 1 and friends_import['config']['friends']['friends'][0]['name'] == 'ImportedOffline')
        check('Friends NBT import preserves active module lifecycle', abs(friends_import['zoom']['factor']-4.5) < 0.01)
        before = json.dumps(friends_import['config'], sort_keys=True)
        broken = scratch / 'generated malformed.nbt'
        broken.write_bytes(path.read_bytes()[:-1])
        ctl('command', f'meteor import modules "{broken}"')
        check('Malformed NBT leaves live config untouched', json.dumps(state('nbt_rejected')['meteor']['config'], sort_keys=True) == before)
        ctl('command', 'meteor toggle zoom')
    if args.player_actions:
        ctl('release')
        ctl('command', 'gamemode creative')
        ctl('command', f'fill {x-3} {y-1} {z-3} {x+3} {y-1} {z+16} minecraft:stone')
        ctl('command', f'fill {x-3} {y} {z-3} {x+3} {y+5} {z+16} minecraft:air')
        ctl('command', 'attribute @s minecraft:block_interaction_range base set 16')
        def break_line(label):
            ctl('release')
            ctl('command', f'fill {x} {y+1} {z+2} {x} {y+1} {z+13} minecraft:stone')
            ctl('tp', x+0.5, y, z+0.5)
            ctl('look', 0, 0)
            ctl('wait', 8)
            ctl('mouse', 'left', 'down')
            ctl('wait', 12)
            ctl('mouse', 'left', 'up')
            ctl('wait', 3)
            blocks = ctl('blocks', x, y+1, z+2, x, y+1, z+13)
            report[label] = blocks
            return 12-len(blocks['blocks'])
        ordinary_breaks = break_line('break_delay_normal')
        ctl('command', 'meteor toggle break-delay')
        quick_breaks = break_line('break_delay_enabled')
        check('BreakDelay accelerates real held-button block breaking', quick_breaks > ordinary_breaks and ordinary_breaks > 0)
        ctl('command', 'meteor toggle break-delay')
        restored_breaks = break_line('break_delay_disabled')
        check('Disabling BreakDelay restores ordinary breaking cadence', restored_breaks < quick_breaks)
        report['break_counts'] = {'normal':ordinary_breaks,'enabled':quick_breaks,'restored':restored_breaks}
        ctl('command', 'attribute @s minecraft:block_interaction_range base set 4.5')
        ctl('command', 'gamemode survival')
        ctl('hotbar', 1)
        ctl('look', 0, -70)
        def use_bottles(label):
            ctl('release')
            ctl('command', 'item replace entity @s weapon.mainhand with minecraft:experience_bottle 64')
            ctl('wait', 5)
            before = ctl('state')['held']['count']
            ctl('mouse', 'right', 'down')
            ctl('wait', 10)
            ctl('mouse', 'right', 'up')
            ctl('wait', 3)
            held = state(label)['held']
            return before - (held['count'] if held else 0)
        normal_uses = use_bottles('fast_use_normal')
        ctl('command', 'meteor toggle fast-use')
        fast_uses = use_bottles('fast_use_enabled')
        check('FastUse increases actual bottle uses', fast_uses > normal_uses and normal_uses > 0)
        ctl('command', 'meteor set fast-use mode Some')
        ctl('command', 'meteor set fast-use items minecraft:stone')
        filtered_uses = use_bottles('fast_use_filtered')
        check('FastUse item filter retains normal unlisted-item delay', filtered_uses < fast_uses)
        old_items = ctl('state')['meteor']['config']['fast_use']['items']
        ctl('command', 'meteor set fast-use items minecraft:not_a_real_item')
        check('Unknown FastUse item edit leaves settings unchanged', ctl('state')['meteor']['config']['fast_use']['items'] == old_items)
        ctl('command', 'meteor toggle fast-use')
        report['use_counts'] = {'normal':normal_uses,'enabled':fast_uses,'filtered':filtered_uses}
        ctl('command', f'spawnpoint @s {x} {y} {z}')
        ctl('command', 'kill @s')
        ctl('wait', 8)
        check('Ordinary death opens death screen', state('respawn_baseline')['screen'] == 'dead')
        ctl('respawn')
        ctl('wait', 15)
        ctl('command', 'meteor toggle auto-respawn')
        ctl('command', 'kill @s')
        ctl('wait', 20)
        respawned = state('auto_respawn_enabled')
        check('AutoRespawn returns to live gameplay', respawned['screen'] == 'playing' and respawned['player']['health'] > 0)
        ctl('command', 'meteor toggle auto-respawn')
        ctl('command', 'kill @s')
        ctl('wait', 8)
        check('Disabling AutoRespawn restores death screen', state('auto_respawn_disabled')['screen'] == 'dead')
        ctl('respawn')
        ctl('wait', 15)
    if args.fullbright:
        from PIL import Image, ImageStat
        ctl('release')
        ctl('command', 'effect clear @s')
        ctl('command', 'time set midnight')
        ctl('command', 'weather clear')
        ctl('command', f'fill {x-4} {y-1} {z-4} {x+4} {y+4} {z+4} minecraft:stone')
        ctl('command', f'fill {x-3} {y} {z-3} {x+3} {y+3} {z+3} minecraft:air')
        ctl('tp', x+0.5, y, z+0.5)
        ctl('look', 0, 0)
        ctl('wait', 60)
        def chamber_brightness(label):
            path = scratch / (label + '.png')
            ctl('screenshot', path)
            with Image.open(path) as img:
                w, h = img.size
                # World surface only: exclude HUD, chat and hand from the measurement.
                crop = img.convert('RGB').crop((int(w*.30),int(h*.25),int(w*.65),int(h*.60)))
                value = sum(ImageStat.Stat(crop).mean)/3
            report.setdefault('fullbright_brightness', {})[label] = value
            return value
        dark = chamber_brightness('fullbright_baseline')
        raw_light = ctl('light', x, y+1, z, x, y+1, z)
        ctl('command', 'meteor toggle fullbright')
        ctl('wait', 6)
        gamma = chamber_brightness('fullbright_gamma')
        check('Fullbright Gamma visibly lights sealed chamber', gamma > dark+20)
        ctl('command', 'meteor toggle fullbright')
        ctl('wait', 6)
        check('Disabling Gamma restores darkness', chamber_brightness('fullbright_gamma_off') < gamma-15)
        ctl('command', 'meteor set fullbright mode Luminance')
        ctl('command', 'meteor set fullbright minimum-light-level 15')
        ctl('command', 'meteor toggle fullbright')
        ctl('wait', 80)
        luminance = state('fullbright_luminance')['meteor']['fullbright']
        check('Luminance selects block-only floor', luminance['block_floor'] == 15 and luminance['sky_floor'] == 0)
        check('Luminance visibly lights remeshed chamber', chamber_brightness('fullbright_luminance') > dark+15)
        check('Luminance preserves actual world light data', ctl('light', x, y+1, z, x, y+1, z) == raw_light)
        ctl('command', 'meteor set fullbright light-type SKY')
        ctl('wait', 80)
        sky = state('fullbright_sky')['meteor']['fullbright']
        check('Luminance switches independent sky channel', sky['block_floor'] == 0 and sky['sky_floor'] == 15)
        check('Sky luminance changes visible lighting', chamber_brightness('fullbright_sky') > dark+3)
        ctl('command', 'meteor toggle fullbright')
        ctl('wait', 60)
        check('Disabling Luminance clears render floors', state('fullbright_luminance_off')['meteor']['fullbright']['sky_floor'] == 0)
        ctl('command', 'meteor set fullbright mode Potion')
        ctl('command', 'meteor toggle fullbright')
        ctl('wait', 8)
        check('Potion mode refreshes actual Night Vision duration', state('fullbright_potion')['meteor']['fullbright']['night_vision_ticks'] >= 400)
        check('Potion mode visibly lights chamber', chamber_brightness('fullbright_potion') > dark+15)
        ctl('command', 'meteor toggle fullbright')
        ctl('wait', 3)
        check('Disabling Potion removes client Night Vision', state('fullbright_potion_off')['meteor']['fullbright']['night_vision_ticks'] is None)
    if args.slow_movement:
        import math
        ctl('release')
        ctl('command', 'gamemode survival')
        ctl('command', 'effect clear @s')
        ctl('command', f'fill {x-8} {y} {z-8} {x+8} {y+5} {z+30} minecraft:air')
        def floor(block):
            ctl('command', f'fill {x-8} {y-1} {z-8} {x+8} {y-1} {z+30} minecraft:{block}')
        def speed_of(value):
            v = value['player']['velocity']
            return math.hypot(v[0], v[2])
        def eye_height(value):
            return value['player']['eye'][1] - value['player']['position'][1]
        def move_sample(label, shield=False):
            ctl('release')
            ctl('tp', x+0.5, y+0.1, z+0.5)
            ctl('look', 0, 0)
            ctl('wait', 12)
            if shield: ctl('mouse', 'right', 'down')
            ctl('key', 'w', 'down')
            ctl('wait', 12)
            sample = state(label)
            ctl('release')
            return sample
        floor('stone')
        normal = move_sample('sneak_normal')
        ctl('command', 'meteor toggle sneak')
        sneak = move_sample('sneak_vanilla')
        # Generic state.sneaking reports physical Shift, not the simulated player pose.
        check('Vanilla Sneak crouches without a held key', eye_height(sneak) < eye_height(normal)-0.2 and sneak['meteor']['slow_movement']['moving_slowly'])
        check('Vanilla Sneak slows actual movement', speed_of(sneak) < speed_of(normal)*0.6)
        ctl('command', 'meteor set sneak mode Packet')
        packet = move_sample('sneak_packet')
        check('Packet Sneak preserves local walking speed', speed_of(packet) > speed_of(sneak)*1.5 and abs(eye_height(packet)-eye_height(normal)) < 0.01)
        check('Packet Sneak sends actual sneak input to integrated server', bool(packet['meteor']['last_sent_input'] & 32))
        ctl('command', 'meteor toggle sneak')
        restored = move_sample('sneak_disabled')
        check('Disabling Sneak restores ordinary posture', abs(eye_height(restored)-eye_height(normal)) < 0.01)
        check('Disabling Sneak clears outgoing sneak input', not restored['meteor']['last_sent_input'] & 32)
        ctl('command', 'item replace entity @s weapon.mainhand with minecraft:shield')
        shield_normal = move_sample('no_slow_shield_normal', shield=True)
        ctl('command', 'meteor toggle no-slow')
        shield_fast = move_sample('no_slow_shield_enabled', shield=True)
        check('NoSlow removes actual shield-use movement slowdown', speed_of(shield_fast) > speed_of(shield_normal)*2)
        ctl('command', 'meteor set no-slow items false')
        shield_filtered = move_sample('no_slow_items_disabled', shield=True)
        check('NoSlow items setting restores shield slowdown', speed_of(shield_filtered) < speed_of(shield_fast)*0.6)
        ctl('command', 'meteor toggle no-slow')
        ctl('command', 'item replace entity @s weapon.mainhand with minecraft:air')
        for block in ('soul_sand', 'honey_block'):
            floor(block)
            slow = move_sample('no_slow_' + block + '_baseline')
            ctl('command', 'meteor toggle no-slow')
            fast = move_sample('no_slow_' + block + '_enabled')
            check('NoSlow removes ' + block + ' walking slowdown', speed_of(fast) > speed_of(slow)*1.5)
            ctl('command', 'meteor toggle no-slow')
        floor('stone')
        def coast(label):
            move_sample(label + '_moving')
            start = ctl('state')['player']['position'][2]
            ctl('wait', 10)
            return state(label)['player']['position'][2] - start
        normal_coast = coast('slippy_baseline')
        ctl('command', 'meteor toggle slippy')
        slippery_coast = coast('slippy_enabled')
        check('Slippy increases real distance after releasing input', slippery_coast > normal_coast+0.3)
        ctl('command', 'meteor set slippy list-mode Whitelist')
        filtered_coast = coast('slippy_empty_whitelist')
        check('Empty Slippy whitelist restores ordinary friction', filtered_coast < slippery_coast*0.6)
        ctl('command', 'meteor toggle slippy')
        report['coast_distances'] = {'normal':normal_coast, 'slippy':slippery_coast, 'filtered':filtered_coast}
        ctl('command', f'fill {x-2} {y} {z-2} {x+2} {y+2} {z+2} minecraft:water')
        ctl('tp', x+0.5, y+0.3, z+0.5)
        ctl('wait', 5)
        fluid_normal = state('no_slow_water_baseline')['meteor']['slow_movement']
        check('Water fixture reaches actual fluid sensing', fluid_normal['raw_water'] and fluid_normal['in_water'])
        ctl('command', 'meteor set no-slow fluid-drag true')
        ctl('command', 'meteor toggle no-slow')
        ctl('wait', 2)
        fluid_filtered = state('no_slow_water_enabled')['meteor']['slow_movement']
        check('NoSlow changes local water getter while retaining raw fluid state', fluid_filtered['raw_water'] and not fluid_filtered['in_water'])
        ctl('command', 'meteor toggle no-slow')
        ctl('wait', 2)
        fluid_restored = state('no_slow_water_restored')['meteor']['slow_movement']
        check('Disabling NoSlow restores water getter', fluid_restored['raw_water'] and fluid_restored['in_water'])
    if args.profiles:
        ctl('release')
        ctl('command', 'gamemode creative')
        ctl('command', 'meteor set zoom zoom 3')
        ctl('command', 'meteor set zoom smooth false')
        ctl('command', 'meteor toggle zoom')
        ctl('wait', 3)
        ctl('command', 'meteor profiles create "Generated Modules" modules')
        stores = list(home.rglob('profiles.json'))
        check('Profile command creates isolated persistent store', len(stores) == 1)
        profile_path = stores[0]
        ctl('command', 'meteor friends add ProfileFriend')
        ctl('command', 'meteor set zoom zoom 7')
        ctl('command', 'meteor toggle zoom')
        ctl('command', 'meteor profiles load "Generated Modules"')
        ctl('wait', 3)
        restored = state('profiles_modules_loaded')['meteor']
        check('Module profile restores active Zoom behavior and settings', abs(restored['zoom']['factor']-3) < 0.01)
        check('Module profile preserves Friends outside its scope', any(f['name'] == 'ProfileFriend' for f in restored['config']['friends']['friends']))
        ctl('command', 'meteor profiles create "Generated HUD" hud')
        hud_saved = ctl('state')['meteor']['config']['hud']['active']
        ctl('key', 'rshift', 'tap')
        ctl('meteor-cursor', 130, 12)
        ctl('mouse', 'left', 'click')
        ctl('meteor-cursor', 120, 12)
        ctl('mouse', 'left', 'click')
        ctl('key', 'esc', 'tap')
        ctl('key', 'rshift', 'tap')
        check('Profile fixture changes actual HUD enabled state', ctl('state')['meteor']['config']['hud']['active'] != hud_saved)
        ctl('command', 'meteor set zoom zoom 5')
        zoom_before_hud_load = ctl('state')['meteor']['zoom']['factor']
        ctl('command', 'meteor profiles load "Generated HUD"')
        ctl('wait', 3)
        hud_restored = state('profiles_hud_loaded')['meteor']
        check('HUD profile restores saved overlay visibility', hud_restored['config']['hud']['active'] == hud_saved)
        # Zoom captures its factor on activation in the source; editing the setting
        # must not restart it. A HUD-only profile preserves both values independently.
        check('HUD profile preserves active module settings and behavior', hud_restored['config']['zoom']['zoom'] == 5 and abs(hud_restored['zoom']['factor']-zoom_before_hud_load) < 0.01)
        ctl('command', 'meteor profiles create "Generated Empty"')
        empty = next(p for p in json.loads(profile_path.read_text())['entries'] if p['name'] == 'Generated Empty')
        check('New profile defaults to no saved sections', not any(empty['save'].values()))
        before = profile_path.read_bytes()
        ctl('command', 'meteor profiles sections "Generated Empty" waypoints')
        check('Unsupported profile section leaves store untouched', profile_path.read_bytes() == before)
        config_before = json.dumps(ctl('state')['meteor']['config'], sort_keys=True)
        profile_path.write_text('{malformed generated fixture', encoding='utf-8')
        ctl('command', 'meteor profiles load "Generated Modules"')
        ctl('command', 'meteor profiles create "Must Not Overwrite" modules')
        check('Malformed profiles preserve live config and source file', json.dumps(ctl('state')['meteor']['config'], sort_keys=True) == config_before and profile_path.read_text() == '{malformed generated fixture')
        profile_path.write_bytes(before)
        ctl('command', 'meteor profiles delete "Generated Empty"')
        check('Profile delete persists removal', not any(p['name'] == 'Generated Empty' for p in json.loads(profile_path.read_text())['entries']))
        ctl('command', 'meteor set zoom zoom 4')
        ctl('command', 'meteor profiles create "Generated Later" modules')
        ctl('command', 'meteor profiles joins "Generated Modules" MeteorRuntimeTest')
        ctl('command', 'meteor profiles joins "Generated Later" MeteorRuntimeTest')
        ctl('command', 'meteor toggle zoom')
        # Persist only this test's generated fixture, with fresh homes for every process.
        launch_args = list(game.args)
        ctl('quit')
        game.wait(timeout=30)
        check('First profile test client exits cleanly', game.returncode == 0)
        if not home.is_relative_to(scratch) or any(p.is_symlink() for p in home.rglob('*')):
            raise RuntimeError('Generated fixture must remain within its owned scratch directory without links')
        restart_root = scratch / ('restart-' + uuid.uuid4().hex)
        restart_root.mkdir()
        restart_home = restart_root / 'game'
        shutil.copytree(home, restart_home)
        home = restart_home
        for key, folder in isolation_folders.items(): env[key] = str(restart_root / folder)
        socket = restart_root / 'control.sock'
        launch_args[launch_args.index('--control-socket')+1] = str(socket)
        report['profile_restart_scratch'] = str(restart_root)
        game = subprocess.Popen(launch_args, cwd=home, env=env, stdout=log, stderr=log, creationflags=creationflags)
        report['profile_restart_pid'] = game.pid
        deadline = time.monotonic() + 180
        while True:
            if game.poll() is not None: raise RuntimeError('Profile restart exited before loading')
            try:
                joined = ctl('state', timeout=5)
                if joined.get('player') and joined.get('screen') == 'playing': break
            except (RuntimeError, subprocess.TimeoutExpired, json.JSONDecodeError): pass
            if time.monotonic() > deadline: raise TimeoutError('Profile restart did not load')
            time.sleep(1)
        ctl('wait', 12)
        autoload = state('profiles_join_autoload')['meteor']
        check('Matching profiles autoload in insertion order on generated-world reopen', abs(autoload['zoom']['factor']-4) < 0.01)
    if args.macros:
        ctl('release')
        ctl('command', 'meteor set zoom zoom 7')
        generated_macro = {'name':'Generated Local', 'messages':['/meteor set zoom zoom {round(2.6)}'],
                           'keybind':{'key':'key.keyboard.m','modifiers':2}}
        ctl('command', 'meteor macros add ' + json.dumps(generated_macro))
        ctl('key', 'm', 'tap')
        ctl('wait', 3)
        check('Macro chord requires its modifier', ctl('state')['meteor']['config']['zoom']['zoom'] == 7)
        ctl('key', 'ctrl', 'down')
        ctl('key', 'm', 'tap')
        ctl('key', 'ctrl', 'up')
        ctl('wait', 3)
        check('Bound macro evaluates Starscript and dispatches local command', ctl('state')['meteor']['config']['zoom']['zoom'] == 3)
        ctl('command', 'meteor macro "Generated Local" 1s')
        generated_macro['messages'] = ['/meteor set zoom zoom 5']
        ctl('command', 'meteor macros edit 0 ' + json.dumps(generated_macro))
        ctl('wait', 24)
        check('Delayed macro observes edits to the scheduled object', ctl('state')['meteor']['config']['zoom']['zoom'] == 5)
        ctl('command', 'meteor set zoom zoom 2')
        ctl('command', 'meteor macro "Generated Local" 1s')
        ctl('command', 'meteor macros clear "Generated Local"')
        ctl('wait', 24)
        check('Clearing scheduled macro prevents later dispatch', ctl('state')['meteor']['config']['zoom']['zoom'] == 2)
        ctl('command', 'meteor macro "Generated Local" 1s')
        ctl('command', 'meteor macros remove 0')
        ctl('wait', 24)
        check('Removing macro preserves already scheduled object like source', ctl('state')['meteor']['config']['zoom']['zoom'] == 5)
        generated_macro = {'name':'Generated Mixed', 'messages':['{1 + }', '/meteor set zoom zoom {round(player.pos.y)}', '/item replace entity @s weapon.mainhand with minecraft:stone {2+3}'],
                           'keybind':{'key':None,'modifiers':0}}
        ctl('command', 'clear @s')
        ctl('hotbar', 1)
        ctl('command', 'meteor macros add ' + json.dumps(generated_macro))
        expected_y = int(ctl('state')['player']['position'][1]+0.5)
        ctl('command', 'meteor macro "Generated Mixed"')
        ctl('wait', 5)
        mixed = state('macro_mixed_dispatch')
        check('Macro skips invalid expression and reads actual player context', mixed['meteor']['config']['zoom']['zoom'] == expected_y)
        check('Macro dispatches evaluated command to actual isolated server', mixed['held'] is not None and mixed['held']['count'] == 5)
        ctl('command', 'meteor profiles create "Generated Macro Profile" macros')
        ctl('command', 'meteor macro "Generated Mixed" 1s')
        edited_mixed = dict(generated_macro, messages=['/meteor set zoom zoom 6'])
        ctl('command', 'meteor macros edit 0 ' + json.dumps(edited_mixed))
        factor_before = ctl('state')['meteor']['zoom']['factor']
        ctl('command', 'meteor profiles load "Generated Macro Profile"')
        macro_profile = ctl('state')['meteor']
        check('Macro profile restores saved definitions and preserves module lifecycle', macro_profile['config']['macros']['macros'][0]['messages'] == generated_macro['messages'] and abs(macro_profile['zoom']['factor']-factor_before) < 0.01)
        ctl('wait', 24)
        check('Macro profile replacement preserves previously queued object identity', ctl('state')['meteor']['config']['zoom']['zoom'] == 6)
    if args.system_guis:
        ctl('release')
        ctl('key', 'rshift', 'tap')
        ctl('wait', 3)
        def click_system_target(page, name):
            gui = ctl('state')['meteor']['gui']
            rect = gui[page]['targets'][name] if page else gui['tabs'][name]
            ctl('meteor-cursor', rect[0]+rect[2]/2, rect[1]+rect[3]/2)
            ctl('mouse', 'left', 'click')
            ctl('wait', 2)
        click_system_target(None, 'macros')
        check('Macros tab opens its actual editor page', ctl('state')['meteor']['gui']['macros']['open'])
        click_system_target('macros', 'create')
        click_system_target('macros', 'name')
        ctl('type', 'Generated GUI Macro')
        click_system_target('macros', 'line/0')
        ctl('type', '/meteor set zoom zoom {3*3}')
        click_system_target('macros', 'bind')
        ctl('key', 'ctrl', 'down')
        ctl('key', 'n', 'tap')
        ctl('key', 'ctrl', 'up')
        ctl('screenshot', scratch / 'macro-editor.png')
        click_system_target('macros', 'save')
        gui_macro = next(m for m in ctl('state')['meteor']['config']['macros']['macros'] if m['name'] == 'Generated GUI Macro')
        check('Macro GUI persists message and modifier binding', gui_macro['messages'] == ['/meteor set zoom zoom {3*3}'] and gui_macro['keybind'] == {'key':'key.keyboard.n','modifiers':2})
        click_system_target('macros', 'back')
        ctl('key', 'rshift', 'tap')
        ctl('key', 'ctrl', 'down')
        ctl('key', 'n', 'tap')
        ctl('key', 'ctrl', 'up')
        ctl('wait', 3)
        check('GUI-created macro executes through its real keybind', ctl('state')['meteor']['config']['zoom']['zoom'] == 9)
        ctl('key', 'rshift', 'tap')
        ctl('wait', 3)
        click_system_target(None, 'profiles')
        check('Profiles tab opens its actual page', ctl('state')['meteor']['gui']['profiles']['open'])
        click_system_target('profiles', 'create')
        click_system_target('profiles', 'name')
        ctl('type', 'Generated GUI Profile')
        click_system_target('profiles', 'modules')
        click_system_target('profiles', 'hud')
        ctl('screenshot', scratch / 'profile-editor.png')
        click_system_target('profiles', 'apply')
        check('Profiles GUI creates and lists its profile', 'Generated GUI Profile' in ctl('state')['meteor']['gui']['profiles']['names'])
        click_system_target('profiles', 'save/Generated GUI Profile')
        check('Profiles GUI requests confirmation before replacing snapshot', ctl('state')['meteor']['gui']['profiles']['confirmation'] is not None)
        click_system_target('profiles', 'save/Generated GUI Profile')
        check('Confirmed profile save completes', not ctl('state')['meteor']['gui']['profiles']['failed'])
        ctl('screenshot', scratch / 'profiles-list.png')
        click_system_target('profiles', 'delete/Generated GUI Profile')
        click_system_target('profiles', 'delete/Generated GUI Profile')
        check('Confirmed profile deletion updates visible list', 'Generated GUI Profile' not in ctl('state')['meteor']['gui']['profiles']['names'])
        click_system_target('profiles', 'back')
        ctl('key', 'rshift', 'tap')
    if args.reach_multitask:
        ctl('release')
        ctl('command', 'gamemode survival')
        ctl('command', 'clear @s')
        ctl('command', f'fill {x-5} {y-1} {z-5} {x+5} {y-1} {z+12} minecraft:stone')
        ctl('command', f'fill {x-5} {y} {z-5} {x+5} {y+5} {z+12} minecraft:air')
        ctl('tp', x+0.5, y, z+1.0)
        ctl('look', 0, 0)
        ctl('command', f'setblock {x} {y+1} {z+6} minecraft:chest')
        ctl('wait', 8)
        ctl('mouse', 'right', 'click')
        ctl('wait', 4)
        check('Distant chest is outside ordinary client reach', ctl('state')['screen'] == 'playing')
        ctl('command', 'meteor set reach extra-block-reach 2')
        ctl('command', 'meteor toggle reach')
        ctl('wait', 2)
        state('reach_enabled_before_use')
        ctl('mouse', 'right', 'click')
        ctl('wait', 5)
        check('Reach opens actual distant chest through integrated server', state('reach_enabled_after_use')['screen'] == 'container')
        ctl('key', 'esc', 'tap')
        ctl('command', 'meteor toggle reach')
        ctl('mouse', 'right', 'click')
        ctl('wait', 4)
        check('Disabling Reach restores ordinary targeting', ctl('state')['screen'] == 'playing')
        ctl('command', f'setblock {x} {y+1} {z+6} minecraft:air')
        ctl('command', 'meteor set reach extra-block-reach 0')
        ctl('command', 'meteor set reach extra-entity-reach 8')
        ctl('command', 'meteor toggle reach')
        ctl('command', f'summon minecraft:cow {x+0.5} {y} {z+8.5} {{NoAI:1b,Silent:1b}}')
        ctl('look', 0, 3)
        ctl('wait', 8)
        distant_entity = ctl('state')['looking_at']['entity']
        check('Reach extends actual entity targeting independently', distant_entity is not None and distant_entity['type'] == 'minecraft:cow')
        # Check real attacks, not the old control API's unoccluded raw mob candidate.
        # Horizon's existing server attack picker rechecks against a fixed 3-block
        # range. Lower the real synced client attribute to 1, then add 2 via Reach;
        # the positive control stays within server authority and tests both paths.
        report['reach_server_attribute'] = ctl('command', 'attribute @s minecraft:entity_interaction_range base set 1')
        print('Reach fixture attribute reply: ' + json.dumps(report['reach_server_attribute']), flush=True)
        ctl('command', 'kill @e[type=minecraft:cow]')
        ctl('command', 'meteor set reach extra-block-reach -3')
        ctl('command', 'meteor set reach extra-entity-reach 2')
        ctl('command', f'summon minecraft:cow {x+0.5} {y} {z+4.0} {{NoAI:1b,Silent:1b}}')
        ctl('look', 0, 6)
        ctl('wait', 24)
        def living_cow():
            return next(e for e in ctl('entities')['entities'] if e.get('type') == 'minecraft:cow' and e.get('health', 0) > 0)
        def cow_health(entity_id):
            return next((e['health'] for e in ctl('entities')['entities'] if e.get('id') == entity_id), 0)
        visible_cow = living_cow()['id']
        ctl('command', 'meteor toggle reach')
        ctl('wait', 2)
        check('Disabled Reach respects actual reduced entity attribute', ctl('state')['looking_at']['entity'] is None)
        ctl('command', 'meteor toggle reach')
        ctl('wait', 2)
        ctl('mouse', 'left', 'click')
        ctl('wait', 5)
        report['reach_visible_attack_entities'] = ctl('entities')
        check('Extended entity targeting dispatches an actual visible attack', cow_health(visible_cow) < 10)
        ctl('command', 'kill @e[type=minecraft:cow]')
        ctl('command', f'summon minecraft:cow {x+0.5} {y} {z+4.0} {{NoAI:1b,Silent:1b}}')
        ctl('wait', 8)
        hidden_cow = living_cow()['id']
        ctl('command', f'fill {x-2} {y} {z+3} {x+2} {y+3} {z+3} minecraft:stone')
        ctl('wait', 24)
        ctl('mouse', 'left', 'click')
        ctl('wait', 5)
        check('Wall beyond shorter block reach prevents actual entity attack', cow_health(hidden_cow) == 10)
        ctl('command', 'meteor toggle reach')
        ctl('command', 'attribute @s minecraft:entity_interaction_range base set 3')
        ctl('command', f'fill {x-2} {y} {z+3} {x+2} {y+3} {z+3} minecraft:air')
        ctl('command', 'kill @e[type=minecraft:cow]')
        ctl('command', 'item replace entity @s weapon.mainhand with minecraft:iron_pickaxe')
        ctl('command', 'item replace entity @s weapon.offhand with minecraft:shield')
        def mine_with_shield(label):
            ctl('release')
            ctl('command', f'setblock {x} {y+1} {z+2} minecraft:stone')
            ctl('look', 0, 0)
            ctl('wait', 5)
            ctl('mouse', 'right', 'down')
            ctl('wait', 3)
            ctl('mouse', 'left', 'down')
            ctl('wait', 20)
            blocks = ctl('blocks', x, y+1, z+2, x, y+1, z+2)
            report[label] = blocks
            using = state(label + '_state')['using']
            check(label + ' retains ongoing item use', using is not None and using['ticks'] >= 20)
            ctl('release')
            ctl('wait', 3)
            return bool(blocks['blocks'])
        check('Ordinary item use blocks continued mining', mine_with_shield('multitask_mining_baseline'))
        ctl('command', 'meteor toggle multitask')
        check('Multitask permits actual mining while using shield', not mine_with_shield('multitask_mining_enabled'))
        ctl('command', 'meteor set multitask attacking-entities false')
        check('Entity attack setting does not disable simultaneous mining', not mine_with_shield('multitask_mining_entities_off'))
        ctl('command', f'setblock {x} {y+1} {z+2} minecraft:air')
        ctl('command', 'item replace entity @s weapon.mainhand with minecraft:iron_sword')
        def attack_with_shield(label):
            ctl('release')
            ctl('command', 'kill @e[type=minecraft:cow]')
            ctl('command', f'summon minecraft:cow {x+0.5} {y} {z+2.5} {{NoAI:1b,Silent:1b}}')
            ctl('look', 0, 10)
            ctl('wait', 24)
            ctl('mouse', 'right', 'down')
            ctl('wait', 3)
            ctl('mouse', 'left', 'click')
            ctl('wait', 5)
            entities = ctl('entities')['entities']
            report[label] = entities
            ctl('release')
            cows = [e for e in entities if e.get('type') == 'minecraft:cow' and e.get('health', 0) > 0]
            return min((e['health'] for e in cows), default=0)
        protected = attack_with_shield('multitask_entity_setting_off')
        check('Disabled entity-attack setting preserves use-item attack gate', protected == 10)
        ctl('command', 'meteor set multitask attacking-entities true')
        attacked = attack_with_shield('multitask_entity_setting_on')
        check('Multitask attacks actual entity while using item', attacked < protected)
        ctl('command', 'meteor toggle multitask')
        check('Disabling Multitask restores use-item attack gate', attack_with_shield('multitask_entity_disabled') == 10)
    if args.macros_import:
        from meteor_macros_import_fixture import run_macros_import
        run_macros_import(ctl, check, state, scratch)
    if args.auto_clicker:
        from meteor_auto_clicker_fixture import run_auto_clicker
        run_auto_clicker(ctl, check, state, scratch, report, x, y, z)
    if args.flight:
        from meteor_flight_fixture import run_flight
        run_flight(ctl, check, state, scratch, x, y, z)
    if args.picking:
        from meteor_picking_fixture import run_picking
        run_picking(ctl, check, state, scratch, report, x, y, z)
    if args.host_gui:
        from meteor_host_gui_fixture import run_host_gui
        run_host_gui(ctl, check, state, scratch)
    if args.hud_instances:
        from meteor_hud_instances_fixture import run_hud_instances
        run_hud_instances(ctl, check, state, scratch)
    report['passed'] = True
except Exception as error:
    report['passed'] = False
    report['error'] = repr(error)
    print(f'Runtime test failed: {error}', flush=True)
    if game.poll() is None:
        try:
            report['failure_state'] = ctl('state')
            ctl('screenshot', scratch / 'failure.png')
        except Exception as diagnostic_error:
            report['failure_diagnostic_error'] = repr(diagnostic_error)
finally:
    if game.poll() is None:
        try: ctl('quit', timeout=15)
        except Exception as error: report['quit_error'] = repr(error)
        try: game.wait(timeout=20)
        except subprocess.TimeoutExpired:
            game.terminate()
            game.wait(timeout=10)
            report['forced_shutdown'] = True
    report['exit_code'] = game.returncode
    log.close()
    (scratch / 'report.json').write_text(json.dumps(report, indent=2), encoding='utf-8')
    (workspace / 'work/meteor-runtime-current.json').write_text(json.dumps(report, indent=2), encoding='utf-8')
    print(f'Report: {scratch / "report.json"}', flush=True)
if not report.get('passed'): raise SystemExit(1)
