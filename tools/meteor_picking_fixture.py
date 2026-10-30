"""Real picking/render/input checks in the caller's fresh isolated world only."""
import json


def run_picking(ctl, check, state, scratch, report, x, y, z):
    ctl('release')
    for module in ctl('state')['meteor']['config']['modules']['modules']:
        if module['active']:
            ctl('command', 'meteor toggle ' + module['id'])
    ctl('command', 'clear @s')
    ctl('command', 'effect clear @s')
    ctl('command', 'gamemode creative')
    ctl('command', 'time set noon')
    ctl('command', 'weather clear')
    ctl('command', 'attribute @s minecraft:block_interaction_range base set 4.5')
    ctl('command', 'attribute @s minecraft:entity_interaction_range base set 3')
    ctl('command', f'fill {x-8} {y-1} {z-5} {x+8} {y-1} {z+12} minecraft:stone')
    ctl('command', f'fill {x-8} {y} {z-5} {x+8} {y+8} {z+12} minecraft:air')
    ctl('command', 'kill @e[type=minecraft:cow]')
    ctl('command', 'kill @e[type=minecraft:armor_stand]')
    ctl('tp', x+0.5, y, z+0.5)
    ctl('look', 0, 0)
    ctl('command', 'item replace entity @s weapon.mainhand with minecraft:stone 16')
    ctl('command', 'item replace entity @s weapon.offhand with minecraft:air')
    ctl('wait', 12)

    def block_at(pos):
        value = ctl('blocks', *pos, *pos)
        return value['blocks'][0][3] if value['blocks'] else 'minecraft:air'

    def preview(label):
        ctl('wait', 5)
        return state(label)['meteor']['air_place']

    def counts(value):
        return value['preview_line_vertices'], value['preview_triangle_vertices']

    def image(label):
        path = scratch / (label + '.png')
        ctl('screenshot', path)
        return path

    expected = [x, y+1, z+3]
    ctl('mouse', 'right', 'click')
    ctl('wait', 4)
    check('Ordinary block use cannot place into the empty ray endpoint', block_at(expected) == 'minecraft:air')
    ctl('command', 'meteor set air-place custom-range true')
    ctl('command', 'meteor set air-place range 3')
    ctl('command', 'meteor toggle air-place')
    both = preview('air_place_both')
    check('AirPlace caches the real custom-range miss position', both['cached_hit'] == expected, both)
    check('AirPlace Both submits edges and translucent faces', counts(both) == (24, 36), both)
    image('air-place-both-default-alpha')
    color = json.dumps(dict(r=204, g=0, b=0, a=128, rainbow=False), separators=(',', ':'))
    ctl('command', 'meteor set air-place side-color ' + color)
    ctl('command', 'meteor set air-place shape-mode Sides')
    sides = preview('air_place_sides')
    check('AirPlace Sides submits only actual face geometry', counts(sides) == (0, 36), sides)
    sides_path = image('air-place-sides-alpha128')
    ctl('command', 'meteor set air-place shape-mode Lines')
    lines = preview('air_place_lines')
    check('AirPlace Lines submits only actual edge geometry', counts(lines) == (24, 0), lines)
    lines_path = image('air-place-lines')
    ctl('command', 'meteor set air-place render false')
    hidden = preview('air_place_render_disabled')
    check('AirPlace render toggle removes submitted overlay geometry', counts(hidden) == (0, 0), hidden)
    hidden_path = image('air-place-render-off')
    # Geometry counters alone cannot prove GPU output. The red fixture color is
    # absent from this central sky/stone area with the overlay disabled.
    from PIL import Image
    def red_pixels(path):
        pixels = Image.open(path).convert('RGB').crop((240, 100, 720, 430)).getdata()
        return sum(r > 100 and r > g + 45 and r > b + 45 for r, g, b in pixels)
    red = {name: red_pixels(path) for name, path in [('sides', sides_path), ('lines', lines_path), ('off', hidden_path)]}
    report['air_place_overlay_red_pixels'] = red
    check('AirPlace translucent sides produce real colored GPU pixels', red['sides'] > red['off'] + 100, red)
    check('AirPlace lines produce real colored GPU pixels', red['lines'] > red['off'] + 20, red)
    ctl('command', 'meteor set air-place render true')
    ctl('command', 'meteor set air-place shape-mode Both')
    ctl('mouse', 'right', 'click')
    ctl('wait', 8)
    check('AirPlace places a real unsupported block through the server', block_at(expected).startswith('minecraft:stone'), block_at(expected))
    check('Occupied AirPlace target stops rendering the preview', counts(preview('air_place_occupied')) == (0, 0))
    ctl('command', f'setblock {expected[0]} {expected[1]} {expected[2]} minecraft:air')
    ctl('command', 'meteor toggle air-place')
    ctl('wait', 3)
    ctl('mouse', 'right', 'click')
    ctl('wait', 5)
    check('Disabling AirPlace restores empty-space block use', block_at(expected) == 'minecraft:air')
    ctl('command', 'meteor toggle air-place')
    ctl('command', 'meteor set air-place range 2')
    check('AirPlace changed range updates the actual cached target', preview('air_place_range2')['cached_hit'] == [x, y+1, z+2])
    ctl('command', 'meteor set air-place custom-range false')
    native_range = ctl('state')['meteor']['interaction_ranges']['block']
    native_target = [x, y+1, int((z+0.5+native_range)//1)]
    check('AirPlace default range reads actual block interaction attribute', preview('air_place_native_range')['cached_hit'] == native_target)
    ctl('command', 'meteor set air-place custom-range true')
    ctl('command', 'meteor set air-place range 0')
    overlapping = preview('air_place_obstructed')['cached_hit']
    ctl('mouse', 'right', 'click')
    ctl('wait', 5)
    check('AirPlace actual entity collision prevents block inside player', block_at(overlapping) == 'minecraft:air')
    ctl('command', 'meteor set air-place range 3')
    ctl('command', 'item replace entity @s weapon.mainhand with minecraft:stick')
    check('AirPlace excludes nonplaceable hand items from preview', counts(preview('air_place_stick')) == (0, 0))
    ctl('command', 'item replace entity @s weapon.offhand with minecraft:stone 8')
    offhand = preview('air_place_offhand')
    check('AirPlace recognizes a placeable offhand', counts(offhand) == (24, 36))
    ctl('mouse', 'right', 'click')
    ctl('wait', 8)
    check('AirPlace actually places with offhand after main-hand pass', block_at(offhand['cached_hit']).startswith('minecraft:stone'))
    ctl('command', f'setblock {expected[0]} {expected[1]} {expected[2]} minecraft:air')
    ctl('command', 'item replace entity @s weapon.offhand with minecraft:air')
    ctl('command', 'item replace entity @s weapon.mainhand with minecraft:cow_spawn_egg 4')
    check('AirPlace spawn egg enters actual preview path', counts(preview('air_place_spawn_egg')) == (24, 36))
    ctl('mouse', 'right', 'click')
    ctl('wait', 8)
    cows = [e for e in ctl('entities')['entities'] if e.get('type') == 'minecraft:cow' and e.get('health', 0) > 0]
    check('AirPlace spawn egg produces a real server entity', bool(cows), cows)
    ctl('command', 'kill @e[type=minecraft:cow]')
    ctl('wait', 24)
    marker = [x+6, y, z]
    ctl('command', f'setblock {marker[0]} {marker[1]} {marker[2]} minecraft:air')
    ctl('command', 'item replace entity @s weapon.mainhand with minecraft:armor_stand 4')
    preview('air_place_armor_stand')
    ctl('mouse', 'right', 'click')
    ctl('wait', 8)
    # Generic control entities currently omits armor stands. Ask the real server
    # selector to place a generated marker only when the spawned stand exists.
    ctl('command', f'execute if entity @e[type=minecraft:armor_stand,limit=1] run setblock {marker[0]} {marker[1]} {marker[2]} minecraft:gold_block')
    ctl('wait', 5)
    check('AirPlace armor stand produces a real server entity', block_at(marker).startswith('minecraft:gold_block'), block_at(marker))
    ctl('command', 'kill @e[type=minecraft:armor_stand]')
    ctl('command', 'gamemode survival')
    ctl('command', 'item replace entity @s weapon.mainhand with minecraft:firework_rocket 5')
    check('Grounded firework enters AirPlace preview path', counts(preview('air_place_firework')) == (24, 36))
    ctl('mouse', 'right', 'click')
    ctl('wait', 8)
    check('AirPlace grounded firework use consumes a real server item', ctl('inventory')['counts'].get('minecraft:firework_rocket', 0) == 4)
    ctl('command', 'meteor toggle air-place')

    # LiquidInteract retries SOURCE_ONLY only when the ordinary max-distance
    # block pick missed. Water is deliberately floating, with no wall behind it.
    ctl('release')
    ctl('command', 'gamemode creative')
    ctl('command', 'item replace entity @s weapon.mainhand with minecraft:stone 16')
    ctl('command', f'fill {x-4} {y} {z-2} {x+4} {y+5} {z+8} minecraft:air')
    ctl('tp', x+0.5, y, z+0.5)
    ctl('look', 0, 0)
    water = [x, y+1, z+2]
    wall = [x, y+1, z+4]
    # Source water flows down and can otherwise push the player until the solid
    # positive control leaves reach. Feet-height supports stay below the eye ray.
    for support in [(x,y,z-1),(x-1,y,z),(x+1,y,z)]:
        ctl('command', f'setblock {support[0]} {support[1]} {support[2]} minecraft:stone')
    ctl('command', f'setblock {water[0]} {water[1]} {water[2]} minecraft:water[level=0]')
    ctl('wait', 6)
    check('Ordinary crosshair ignores floating source water', state('liquid_off')['looking_at']['kind'] == 'none')
    ctl('mouse', 'right', 'click')
    ctl('wait', 4)
    check('Ordinary block use does not replace unpicked source water', block_at(water).startswith('minecraft:water'))
    ctl('command', 'meteor toggle liquid-interact')
    ctl('wait', 4)
    liquid = state('liquid_source')['looking_at']
    check('LiquidInteract picks actual source-fluid shape', liquid['kind'] == 'block' and liquid['block']['pos'] == water and not liquid['block']['inside'], liquid)
    ctl('mouse', 'right', 'click')
    ctl('wait', 8)
    check('LiquidInteract enables actual block placement into source water', block_at(water).startswith('minecraft:stone'))
    ctl('command', f'setblock {water[0]} {water[1]} {water[2]} minecraft:water[level=0]')
    ctl('command', f'setblock {wall[0]} {wall[1]} {wall[2]} minecraft:stone')
    ctl('tp', x+0.5, y, z+0.5)
    ctl('wait', 5)
    solid = state('liquid_solid_priority')['looking_at']
    check('Existing ordinary solid hit wins over nearer source fluid', solid['kind'] == 'block' and solid['block']['pos'] == wall, solid)
    ctl('command', f'fill {x-4} {y} {z-2} {x+4} {y+5} {z+8} minecraft:air')
    # A lone manually set flowing cell drains on its next scheduled update. Feed
    # this cell from a real source OFF the ray, over a floor below eye height.
    ctl('command', f'fill {x-8} {y+1} {z-8} {x+8} {y+1} {z+12} minecraft:stone')
    ctl('command', f'setblock {x+1} {y+2} {z+2} minecraft:water[level=0]')
    ctl('command', f'setblock {x} {y+1} {z} minecraft:air')
    ctl('command', f'setblock {x} {y} {z} minecraft:stone')
    ctl('wait', 18)
    ctl('tp', x+0.5, y+1, z+0.5)
    ctl('look', 0, 0)
    ctl('wait', 2)
    flow = state('liquid_flowing_stable')['looking_at']
    actual = block_at([x,y+2,z+2])
    check('Flowing-water fixture remains a real non-source state', 'minecraft:water[level=' in actual and 'level=0]' not in actual, actual)
    check('LiquidInteract excludes flowing fluid from source-only pick', flow['kind'] == 'none', flow)
    ctl('command', f'fill {x-4} {y} {z-2} {x+4} {y+5} {z+8} minecraft:air')
    ctl('tp', x+0.5, y, z+0.5)
    inside = [x, y+1, z]
    ctl('command', f'setblock {inside[0]} {inside[1]} {inside[2]} minecraft:water[level=0]')
    ctl('wait', 3)
    submerged = state('liquid_inside')['looking_at']
    check('Fluid ray preserves actual inside-hit flag and direction', submerged['kind'] == 'block' and submerged['block']['pos'] == inside and submerged['block']['inside'] and bool(submerged['block']['face']), submerged)
    ctl('command', 'meteor toggle liquid-interact')
    ctl('wait', 3)
    check('Disabling LiquidInteract restores fluid miss', ctl('state')['looking_at']['kind'] == 'none')
    ctl('command', f'fill {x-4} {y} {z-2} {x+4} {y+5} {z+8} minecraft:air')

    # Invulnerable generated cow stays in the ray instead of being knocked away
    # by the baseline attack; all successful results are real server mining.
    ctl('command', 'gamemode survival')
    ctl('command', 'attribute @s minecraft:block_interaction_range base set 4.5')
    ctl('command', 'attribute @s minecraft:entity_interaction_range base set 3')
    target = [x, y+1, z+3]
    def trace_fixture(label, main='iron_pickaxe', off='air', block='stone', expected_kind='entity'):
        ctl('release')
        ctl('command', 'kill @e[type=minecraft:cow]')
        ctl('command', f'setblock {target[0]} {target[1]} {target[2]} minecraft:{block}')
        ctl('command', f'summon minecraft:cow {x+0.5} {y} {z+2.5} {{NoAI:1b,Silent:1b,Invulnerable:1b}}')
        ctl('command', 'item replace entity @s weapon.mainhand with minecraft:' + main)
        ctl('command', 'item replace entity @s weapon.offhand with minecraft:' + off)
        ctl('tp', x+0.5, y, z+0.5)
        ctl('look', 0, 10)
        ctl('wait', 8)
        before = state(label + '_before')['looking_at']
        check(label + ' chooses the actual expected hit', before['kind'] == expected_kind, before)
        ctl('mouse', 'left', 'down')
        ctl('wait', 30)
        ctl('mouse', 'left', 'up')
        ctl('wait', 5)
        value = block_at(target)
        report[label + '_block'] = value
        return value == 'minecraft:air'

    check('Ordinary entity trace prevents mining behind generated cow', not trace_fixture('trace_baseline'))
    ctl('command', 'meteor toggle no-mining-trace')
    check('NoMiningTrace actually mines behind selected entity', trace_fixture('trace_main_pickaxe', expected_kind='block'))
    check('NoMiningTrace honors an offhand pickaxe tag', trace_fixture('trace_offhand_pickaxe', main='iron_shovel', off='iron_pickaxe', block='dirt', expected_kind='block'))
    check('Default NoMiningTrace requires a pickaxe in either hand', not trace_fixture('trace_requires_pickaxe', main='iron_shovel', block='dirt'))
    ctl('command', 'meteor set no-mining-trace only-when-holding-a-pickaxe false')
    check('NoMiningTrace optional pickaxe restriction supports actual other tool', trace_fixture('trace_any_tool', main='iron_shovel', block='dirt', expected_kind='block'))
    ctl('command', 'meteor set no-mining-trace only-when-holding-a-pickaxe true')
    ctl('command', 'meteor set no-mining-trace blacklisted-entities ["minecraft:cow"]')
    check('NoMiningTrace blacklist preserves actual entity interaction', not trace_fixture('trace_blacklisted_cow'))
    ctl('command', 'meteor set no-mining-trace blacklisted-entities []')
    ctl('command', 'meteor set reach extra-block-reach -3.5')
    ctl('command', 'meteor set reach extra-entity-reach 0')
    ctl('command', 'meteor toggle reach')
    check('NoMiningTrace preserves source raw-block return past shorter block reach', trace_fixture('trace_raw_max_range', expected_kind='block'))
    ctl('command', 'meteor toggle reach')
    ctl('command', f'setblock {target[0]} {target[1]} {target[2]} minecraft:air')
    ctl('wait', 5)
    check('NoMiningTrace keeps entity hit when there is no real block to substitute', state('trace_no_block')['looking_at']['kind'] == 'entity')
    ctl('command', 'meteor toggle no-mining-trace')
    check('Disabling NoMiningTrace restores actual entity obstruction', not trace_fixture('trace_disabled'))
    ctl('release')
    ctl('command', 'meteor set reach extra-block-reach 0')
    ctl('command', 'kill @e[type=minecraft:cow]')
    state('picking_complete')
