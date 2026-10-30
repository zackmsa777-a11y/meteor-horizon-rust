"""Generated singleplayer native HUD instance/editor checks; no user files."""
import json


def run_hud_instances(ctl, check, state, scratch):
    ctl('release')
    if ctl('state')['meteor']['gui']['open']:
        ctl('key', 'rshift', 'tap')
    for module in ctl('state')['meteor']['config']['modules']['modules']:
        if module['active']:
            ctl('command', 'meteor toggle ' + module['id'])
    ctl('command', 'gamemode creative')
    ctl('hotbar', 1)
    ctl('command', 'item replace entity @s weapon.mainhand with minecraft:stone 7')
    ctl('wait', 6)
    legacy = ctl('state')['meteor']['config']['hud']['elements'][:9]
    ctl('key', 'rshift', 'tap')
    ctl('wait', 3)
    rect = ctl('state')['meteor']['gui']['tabs']['hud']
    ctl('meteor-cursor', rect[0]+rect[2]/2, rect[1]+rect[3]/2)
    ctl('mouse', 'left', 'click')
    ctl('wait', 3)
    if not ctl('state')['meteor']['config']['hud']['active']:
        # Existing legacy HUD header's real enabled toggle region, x80..200/y0..29.
        ctl('meteor-cursor', 100, 12)
        ctl('mouse', 'left', 'click')
        ctl('wait', 2)

    def click(key, scroll=False):
        if scroll:
            ctl('scroll-at', 200, 160, 30)
            ctl('wait', 2)
        for _ in range(30 if scroll else 1):
            rect = ctl('state')['meteor']['gui']['hud']['targets'].get(key)
            if rect is not None:
                ctl('meteor-cursor', rect[0]+rect[2]/2, rect[1]+rect[3]/2)
                ctl('mouse', 'left', 'click')
                ctl('wait', 2)
                return
            ctl('scroll-at', 200, 160, -1)
            ctl('wait', 2)
        raise AssertionError('HUD target missing: '+key)

    def edit(path, text):
        click(path, True)
        ctl('key', 'ctrl', 'down')
        ctl('key', 'a', 'tap')
        ctl('key', 'ctrl', 'up')
        ctl('type', str(text))
        ctl('key', 'enter', 'tap')
        ctl('wait', 6)

    click('add-text')
    first = ctl('state')['meteor']['gui']['hud']['selected']
    config = ctl('state')['meteor']['config']['hud']
    first_id = config['elements'][first]['id']
    check('HUD GUI creates real source Text instance', config['elements'][first]['kind'] == 'text')
    edit(f'/elements/{first}/text/text', 'Live count {player.hand.count}')
    rendered = ctl('state')['meteor']['gui']['hud']['outputs'][first_id]
    check('TextHud evaluates live Starscript through renderer', rendered['text'] == 'Live count 7' and rendered['visible'], rendered)
    ctl('screenshot', scratch / 'hud-text-settings.png')
    edit(f'/elements/{first}/text/scale', 0.1)
    failed = ctl('state')['meteor']
    check('TextHud invalid scale is rejected atomically', failed['config']['hud']['elements'][first]['text']['scale'] == 1 and bool(failed['gui']['hud']['error']))
    ctl('key', 'escape', 'tap')  # Cancel the invalid field; retain panel.
    edit(f'/elements/{first}/text/condition', '{false}')
    click(f'/elements/{first}/text/shown', True)  # Always -> WhenTrue
    ctl('wait', 6)
    check('TextHud condition hides false text', not ctl('state')['meteor']['gui']['hud']['outputs'][first_id]['visible'])
    edit(f'/elements/{first}/text/condition', '{true}')
    check('TextHud changed condition reevaluates', ctl('state')['meteor']['gui']['hud']['outputs'][first_id]['visible'])
    click(f'/elements/{first}/source_box/x_anchor', True)  # Left -> Center; retains pixels initially.
    edit(f'/elements/{first}/source_box/x', 0)
    ctl('key', 'escape', 'tap')  # Close settings panel.
    ctl('wait', 3)
    geometry = state('hud_text_center_geometry')['meteor']['gui']['hud']
    box = next(e['rect'] for e in geometry['elements'] if e['index'] == first)
    scale = geometry['scale']
    expected = geometry['viewport'][0]//2 - round(box[2]*scale)//2
    check('TextHud centered source box uses physical integer geometry', abs(box[0]*scale-expected) < 0.01, {'rect':box,'scale':scale,'expected_x':expected})
    click('duplicate')
    duplicate = ctl('state')['meteor']['gui']['hud']['selected']
    config = ctl('state')['meteor']['config']['hud']
    duplicate_id = config['elements'][duplicate]['id']
    check('HUD permits repeated Text kinds with independent identities', duplicate_id != first_id and config['elements'][duplicate]['kind'] == 'text')
    click('edit')
    edit(f'/elements/{duplicate}/text/text', 'Second {player.hand.count + 1}')
    outputs = ctl('state')['meteor']['gui']['hud']['outputs']
    check('Repeated Text settings remain independent', outputs[first_id]['text'] == 'Live count 7' and outputs[duplicate_id]['text'] == 'Second 8', outputs)
    edit(f'/elements/{duplicate}/source_box/y', 140)
    ctl('key', 'escape', 'tap')
    ctl('key', 'rshift', 'tap')
    ctl('command', 'item replace entity @s weapon.mainhand with minecraft:stone 3')
    ctl('wait', 8)
    live = state('hud_text_live_update')['meteor']['gui']['hud']['outputs']
    check('TextHud refreshes actual engine state after update delay', live[first_id]['text'] == 'Live count 3' and live[duplicate_id]['text'] == 'Second 4', live)
    ctl('screenshot', scratch / 'hud-text-live.png')
    # Save/load generated profile exercises actual persistence through existing adapter.
    ctl('command', 'meteor profiles create "Generated Text HUD" hud')
    ctl('key', 'rshift', 'tap')
    ctl('wait', 3)
    rect=ctl('state')['meteor']['gui']['tabs']['hud']
    ctl('meteor-cursor',rect[0]+rect[2]/2,rect[1]+rect[3]/2)
    ctl('mouse','left','click')
    ctl('wait',3)
    # Select duplicate via its actual rendered rectangle, then delete selected instance.
    geometry=ctl('state')['meteor']['gui']['hud']
    rect=next(e['rect'] for e in geometry['elements'] if e['index']==duplicate)
    ctl('meteor-cursor',rect[0]+rect[2]/2,rect[1]+rect[3]/2)
    ctl('mouse','left','click')
    ctl('wait',2)
    click('delete')
    check('HUD GUI deletes only the selected duplicate', not any(e['id']==duplicate_id for e in ctl('state')['meteor']['config']['hud']['elements']))
    ctl('key','rshift','tap')
    ctl('command','meteor profiles load "Generated Text HUD"')
    ctl('wait',8)
    saved=state('hud_text_profile_restored')['meteor']['config']['hud']
    check('HUD profile restores repeated Text instances and their settings', any(e['id']==duplicate_id and e['text']['text']=='Second {player.hand.count + 1}' for e in saved['elements']))
    check('New HUD instance work preserves legacy preset layout', saved['elements'][:9] == legacy)
    candidates=[]
    for path in scratch.rglob('config.json'):
        try:
            value=json.loads(path.read_text(encoding='utf-8-sig'))
            if isinstance(value,dict) and isinstance(value.get('hud'),dict):
                candidates.append(value['hud'])
        except (ValueError,OSError):
            pass
    check('HUD instance schema persists to generated config on disk', any(h.get('version')==2 and any(e.get('id')==duplicate_id for e in h.get('elements',[])) for h in candidates))
