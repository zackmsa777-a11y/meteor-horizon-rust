"""Flight checks against actual movement/connection diagnostics in an isolated world."""


def run_flight(ctl, check, state, scratch, x, y, z):
    ctl('release')
    for module in ctl('state')['meteor']['config']['modules']['modules']:
        if module['active']:
            ctl('command', 'meteor toggle ' + module['id'])
    ctl('command', 'gamemode survival')
    ctl('command', 'effect clear @s')
    ctl('command', f'fill {x-10} {y-1} {z-10} {x+10} {y-1} {z+20} minecraft:stone')
    ctl('command', f'fill {x-10} {y} {z-10} {x+10} {y+25} {z+20} minecraft:air')
    ctl('command', 'meteor set flight anti-kick-mode None')
    ctl('command', 'meteor set flight mode Abilities')
    ctl('command', 'meteor set flight speed 0.1')
    ctl('command', 'meteor toggle flight')
    ctl('tp', x+.5, y+8, z+.5)
    ctl('look', 0, 0)
    ctl('wait', 8)
    initial = state('flight_abilities_start')
    a = initial['meteor']['flight']['abilities']
    check('Flight Abilities grants local survival flight', a['flying'] and a['may_fly'] and abs(a['flying_speed']-.1)<1e-6)
    start_y = initial['player']['position'][1]
    ctl('wait', 10)
    check('Flight Abilities actually holds airborne altitude', abs(ctl('state')['player']['position'][1]-start_y)<.1)
    ctl('key','space','down'); ctl('wait',10); ctl('key','space','up')
    up = state('flight_abilities_up')
    check('Flight Abilities jump actually ascends', up['player']['position'][1]>start_y+.5)
    ctl('key','shift','down'); ctl('wait',8); ctl('key','shift','up')
    down = state('flight_abilities_down')
    check('Flight Abilities sneak actually descends', down['player']['position'][1]<up['player']['position'][1]-.3)
    start_z = down['player']['position'][2]
    ctl('key','w','down'); ctl('wait',10); ctl('key','w','up')
    check('Flight Abilities actually moves horizontally', ctl('state')['player']['position'][2]>start_z+.5)
    # Real server gamemode sends ClientboundPlayerAbilities. Flight keeps its local
    # flying fields but must accept creative/invulnerable/walking-speed fields.
    ctl('command','gamemode creative'); ctl('wait',5)
    creative = state('flight_server_creative')['meteor']['flight']['abilities']
    check('Flight preserves fly speed while accepting actual server creative abilities', creative['flying'] and creative['creative'] and creative['invulnerable'] and abs(creative['flying_speed']-.1)<1e-6)
    ctl('command','gamemode survival'); ctl('wait',5)
    survival = state('flight_server_survival')['meteor']['flight']['abilities']
    check('Flight accepts actual survival vulnerability while preserving local flight', survival['flying'] and not survival['creative'] and not survival['invulnerable'])
    ctl('command','meteor toggle flight'); ctl('wait',3)
    disabled = state('flight_disabled')
    check('Disabling Flight restores survival abilities', not disabled['meteor']['flight']['abilities']['flying'] and not disabled['meteor']['flight']['abilities']['may_fly'] and abs(disabled['meteor']['flight']['abilities']['flying_speed']-.05)<1e-6)
    ctl('wait',8)
    check('Disabled Flight resumes actual falling', ctl('state')['player']['position'][1]<disabled['player']['position'][1]-.3)
    ctl('command','meteor set flight mode Velocity')
    ctl('command','meteor set flight no-sneak true')
    ctl('command','meteor toggle flight')
    ctl('tp',x+.5,y+8,z+.5); ctl('look',0,0); ctl('wait',6)
    velocity = state('flight_velocity_start')
    check('Velocity mode does not grant ability flight', not velocity['meteor']['flight']['abilities']['flying'])
    start_y = velocity['player']['position'][1]
    ctl('key','space','down'); ctl('wait',8); ctl('key','space','up')
    normal = state('flight_velocity_up')
    delta_normal = normal['player']['position'][1]-start_y
    check('Velocity physical jump key actually ascends',delta_normal>1)
    ctl('command','meteor set flight vertical-speed-match true')
    ctl('wait',3); start_y = ctl('state')['player']['position'][1]
    ctl('key','space','down'); ctl('wait',8); ctl('key','space','up')
    matched = state('flight_velocity_matched')
    check('Velocity vertical-speed-match increases actual ascent',matched['player']['position'][1]-start_y>delta_normal*1.5)
    ctl('key','shift','down'); ctl('wait',5)
    sneaking = state('flight_velocity_no_sneak')
    check('Velocity no-sneak suppresses local posture without losing descent input',not sneaking['meteor']['flight']['abilities']['shift_key_down'] and sneaking['player']['position'][1]<matched['player']['position'][1]-.5)
    ctl('key','shift','up')
    ctl('command','meteor set flight mode Abilities'); ctl('wait',5)
    check('Live mode change reaches actual Abilities behavior',state('flight_mode_change')['meteor']['flight']['abilities']['flying'])
    ctl('command','meteor set flight anti-kick-mode Packet')
    ctl('command','meteor set flight delay 3')
    ctl('command','meteor set flight off-time 2')
    ctl('tp',x+.5,y+12,z+.5); ctl('wait',25)
    samples=[]
    for i in range(12):
        ctl('look',i*2,0); ctl('wait',1)
        samples.append(state('flight_packet_'+str(i)))
    check('Flight Packet upgrades actual rotation sends into position-bearing packets',all(s['meteor']['flight']['sent_movement'][-1]['kind'] in ('Pos','PosRot') for s in samples))
    sent=[p for s in samples for p in s['meteor']['flight']['sent_movement'] if 'position' in p]
    check('Flight Packet sends actual downward anti-kick offset',any(p['position'][1]<y+12-.02 for p in sent))
    ctl('command','meteor set flight anti-kick-mode Normal')
    observed=[]
    for _ in range(10):
        ctl('wait',1); observed.append(ctl('state')['meteor']['flight']['abilities']['flying'])
    check('Flight Normal actually alternates flying and off intervals',True in observed and False in observed)
    ctl('command','meteor toggle flight'); ctl('release')
    ctl('command','gamemode creative'); ctl('tp',x+.5,y,z+.5); ctl('wait',5)
    ctl('screenshot',scratch/'flight-restored.png')
