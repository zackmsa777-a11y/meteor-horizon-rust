"""Isolated26 runtime fragment; import run_host_gui(ctl,check,state,scratch).
No launch/files outside caller's generated scratch; does not edit shared harness.
"""
import json


def run_host_gui(ctl, check, state, scratch):
    ctl('release')
    if ctl('state')['meteor']['gui']['open']:
        ctl('key', 'rshift', 'tap')
    for module in ctl('state')['meteor']['config']['modules']['modules']:
        if module['active']:
            ctl('command', 'meteor toggle ' + module['id'])
    ctl('command', 'gamemode creative')
    ctl('command', 'clear @s')
    ctl('command', 'effect clear @s')
    ctl('hotbar', 1)
    ctl('command', 'item replace entity @s weapon.mainhand with minecraft:stone 7')
    ctl('command', 'item replace entity @s weapon.offhand with minecraft:diamond_pickaxe')
    ctl('command', 'effect give @s minecraft:speed 60 1 true')
    ctl('wait', 6)
    fixture = state('starscript_live_fixture')
    check('Starscript fixture has actual held stack', fixture['held']['count'] == 7)
    old_zoom = fixture['meteor']['config']['zoom']['zoom']
    name = 'Generated Host Bindings 26'
    index = len(fixture['meteor']['config']['macros']['macros'])
    macro = {'name': name, 'messages': [], 'keybind': {'key': None, 'modifiers': 0}}
    # Use a generated local client setting as an observable expression result.
    # A parse/evaluation failure leaves sentinel19, rather than accidentally passing.
    def expression(label, condition):
        ctl('command', 'meteor set zoom zoom 19')
        macro['messages'] = ['/meteor set zoom zoom {' + condition + ' ? 13 : 17}']
        verb = 'add ' if expression.first else 'edit ' + str(index) + ' '
        ctl('command', 'meteor macros ' + verb + json.dumps(macro))
        expression.first = False
        ctl('command', 'meteor macro ' + json.dumps(name))
        ctl('wait', 2)
        actual = ctl('state')['meteor']
        check(label, actual['config']['zoom']['zoom'] == 13, actual.get('macros'))
    expression.first = True
    expression('Starscript hand registry id is live', 'player.hand.id == "minecraft:stone"')
    expression('Starscript hand count is live', 'player.hand.count == 7')
    expression('Starscript inventory slot maps actual hotbar', 'player.get_item(0).count == 7')
    expression('Starscript item counter resolves unqualified registry id', 'player.count_items("stone") == 7')
    expression('Starscript offhand uses actual item durability', 'player.offhand.durability == 1561')
    expression('Starscript inventory slot40 is offhand', 'player.get_item(40).id == "minecraft:diamond_pickaxe"')
    expression('Starscript potion presence uses actual effects', 'player.has_potion_effect("speed")')
    expression('Starscript potion amplifier maps source level', 'player.get_potion_effect("speed").level == 2')
    expression('Starscript effect duration is measured ticks', 'player.get_potion_effect("speed").duration > 0')
    expression('Starscript unknown effects return null', 'player.get_potion_effect("minecraft:unregistered_effect") == null')
    # Change real engine state after earlier reads to catch stale snapshots.
    ctl('command', 'item replace entity @s weapon.mainhand with minecraft:stone 3')
    ctl('command', 'effect clear @s')
    ctl('command', 'time set 6000')
    ctl('command', 'weather clear')
    ctl('wait', 6)
    expression('Starscript refreshes changed item count', 'player.count_items("minecraft:stone") == 3')
    expression('Starscript refreshes cleared effects', 'player.has_potion_effect("speed") == false')
    ctl('command', 'time set 6000')
    expression('Starscript world clock formats actual noon', '(server.time == "12:00") or (server.time == "12:01") or (server.time == "12:02")')
    expression('Starscript actual clear weather', 'server.weather == "Clear"')
    expression('Starscript camera reads actual position', 'abs(camera.pos.x - player.pos.x) < 10')
    expression('Starscript opposite dimension converts actual coordinate', 'player.opposite_dim_pos.x == player.pos.x / 8')
    ctl('command', 'meteor macros remove ' + str(index))
    ctl('command', 'meteor set zoom zoom ' + str(old_zoom))
    # Real module controls; all positions come from last rendered settings rectangles.
    ctl('key', 'rshift', 'tap')
    ctl('wait', 3)
    ctl('type', 'air-place')
    ctl('meteor-cursor', 30, 88)  # Existing module grid's first filtered row.
    ctl('mouse', 'right', 'click')
    ctl('wait', 3)
    check('AirPlace actual settings page opens', ctl('state')['meteor']['gui']['selected'] == 'air-place')
    def target(key):
        # Scroll through real WindowEvent path, not a synthetic settings mutation.
        ctl('scroll-at', 200, 160, 20)
        ctl('wait', 2)
        for _ in range(12):
            rect = ctl('state')['meteor']['gui']['settings'].get(key)
            if rect is not None:
                ctl('meteor-cursor', rect[0] + rect[2]/2, rect[1] + rect[3]/2)
                ctl('mouse', 'left', 'click')
                ctl('wait', 1)
                return
            ctl('scroll-at', 200, 160, -1)
            ctl('wait', 2)
        raise AssertionError('No rendered settings target: ' + key)
    def edit(key, value):
        target(key)
        ctl('key', 'ctrl', 'down')
        ctl('key', 'a', 'tap')
        ctl('key', 'ctrl', 'up')
        ctl('type', str(value))
        ctl('key', 'enter', 'tap')
        ctl('wait', 2)
    before = dict(ctl('state')['meteor']['config']['air_place']['side_color'])
    edit('air-place/side_color.r', 23)
    color = ctl('state')['meteor']['config']['air_place']['side_color']
    check('Color GUI atomically changes actual red channel', color == dict(before, r=23), color)
    edit('air-place/side_color.a', 64)
    check('Color GUI changes actual opacity', ctl('state')['meteor']['config']['air_place']['side_color']['a'] == 64)
    edit('air-place/side_color.r', 999)
    invalid = ctl('state')['meteor']
    check('Color GUI rejects out-of-range channel without mutation', invalid['config']['air_place']['side_color']['r'] == 23 and bool(invalid['gui']['error']))
    ctl('key', 'escape', 'tap')
    target('air-place/side_color.rainbow')
    check('Color GUI toggles actual rainbow flag', ctl('state')['meteor']['config']['air_place']['side_color']['rainbow'] != before['rainbow'])
    edit('config/rainbow_speed', 0.75)
    check('Color GUI global rainbow speed persists', ctl('state')['meteor']['config']['rainbow_speed'] == 0.75)
    ctl('screenshot', scratch / 'air-place-color-controls.png')
    ctl('key', 'rshift', 'tap')
    state('starscript_colors_complete')
