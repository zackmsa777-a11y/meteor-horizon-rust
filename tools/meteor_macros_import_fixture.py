"""Original-layout macros.nbt fixtures, for the isolated Meteor runtime harness.

The caller owns launching and its scratch directory. This only writes generated
NBT there; imported messages only change a local Meteor setting.
"""
import copy
import gzip
import json
import struct
from pathlib import Path


def _string(value):
    # Fixtures are ASCII; UTF-8 and Java modified UTF-8 are identical here.
    raw = value.encode('ascii')
    return struct.pack('>H', len(raw)) + raw


def _payload(kind, value):
    if kind == 8:
        return _string(value)
    if kind == 9:
        child, entries = value
        return bytes([child]) + struct.pack('>i', len(entries)) + b''.join(
            _payload(child, entry) for entry in entries)
    if kind == 10:
        return b''.join(bytes([tag]) + _string(name) + _payload(tag, item)
                        for name, (tag, item) in value.items()) + b'\0'
    raise ValueError(kind)


def _macro(name, messages, key='key.keyboard.unknown', modifiers=()):
    def setting(label, kind, value):
        return {'name': (8, label), 'value': (kind, value)}
    settings = [setting('name', 8, name), setting('messages', 9, (8, messages)),
                setting('keybind', 10, {'key': (8, key),
                                       'modifiers': (9, (8, list(modifiers)))})]
    return {'settings': (10, {'groups': (9, (10, [
        {'name': (8, 'General'), 'settings': (9, (10, settings))}]))})}


def _bytes(entries):
    return b'\x0a\0\0' + _payload(10, {'macros': (9, (10, entries))})


def run_macros_import(ctl, check, state, scratch):
    scratch = Path(scratch)
    folder = scratch / 'generated-macro-import'
    folder.mkdir(exist_ok=True)
    ctl('release')
    if ctl('state')['meteor']['gui']['open']:
        ctl('key', 'rshift', 'tap')
    ctl('command', 'meteor macro clear')
    original = copy.deepcopy(ctl('state')['meteor']['config'])

    def current():
        return ctl('state')['meteor']

    def import_file(name, contents):
        path = folder / name
        path.write_bytes(contents)
        ctl('command', 'meteor import macros ' + json.dumps(str(path)))
        ctl('wait', 2)

    def execute(name):
        ctl('command', 'meteor macro ' + json.dumps(name))
        ctl('wait', 2)

    def zoom():
        return current()['config']['zoom']['zoom']

    try:
        for module in original['modules']['modules']:
            if module['active']:
                ctl('command', 'meteor toggle ' + module['id'])
        ctl('command', 'meteor set zoom smooth false')
        ctl('command', 'meteor set zoom zoom 2')
        ctl('command', 'meteor toggle zoom')
        ctl('wait', 3)
        before = current()
        imported = [
            _macro('Generated Import Local', ['/meteor set zoom zoom 7'],
                   'key.keyboard.k', ['CONTROL', 'SHIFT']),
            _macro('Generated Duplicate', ['/meteor set zoom zoom 11']),
            _macro('Generated Duplicate', ['/meteor set zoom zoom 13']),
            _macro('Generated Unsupported Bind', ['/meteor set zoom zoom 17'],
                   'key.keyboard.g', ['CAPS_LOCK']),
        ]
        import_file('macros.nbt', _bytes(imported))
        after = state('macros_nbt_imported')['meteor']
        definitions = after['config']['macros']['macros']
        check('NBT macro import preserves definitions and duplicate order',
              [m['name'] for m in definitions] == ['Generated Import Local',
               'Generated Duplicate', 'Generated Duplicate', 'Generated Unsupported Bind'])
        check('NBT macro import maps modern modifier chord',
              definitions[0]['keybind'] == {'key': 'key.keyboard.k', 'modifiers': 3})
        check('NBT unsupported macro bind becomes unset', definitions[3]['keybind']['key'] is None)
        warning = json.dumps(ctl('messages', 30)['styled']).lower()
        check('NBT unsupported bind is reported', 'caps_lock' in warning and 'skipped' in warning, warning)
        expected = copy.deepcopy(before['config'])
        expected['macros'] = after['config']['macros']
        check('NBT macro import preserves all other settings', after['config'] == expected)
        check('NBT import executes no messages',
              after['macros']['dispatched'] == before['macros']['dispatched'] and zoom() == 2)
        check('NBT macro import preserves active Zoom lifecycle',
              after['zoom']['factor'] == before['zoom']['factor'])
        execute('Generated Import Local')
        check('Explicit imported macro executes local command', zoom() == 7)
        execute('Generated Duplicate')
        check('Duplicate macro lookup retains first source entry', zoom() == 11)
        snapshot = current()
        import_file('malformed.nbt', _bytes(imported)[:-3])
        rejected = current()
        check('Malformed NBT macro import is atomic',
              rejected['config'] == snapshot['config'] and
              rejected['macros']['dispatched'] == snapshot['macros']['dispatched'])
        execute('Generated Import Local')
        check('Rejected import preserves live handles', zoom() == 7)
        ctl('command', 'meteor macro "Generated Import Local" 3s')
        replacement = [_macro('Generated Import Local', ['/meteor set zoom zoom 19'])]
        snapshot = current()
        import_file('replacement.nbt', gzip.compress(_bytes(replacement), mtime=0))
        check('Gzip replacement is loaded without executing',
              current()['config']['macros']['macros'][0]['messages'] == ['/meteor set zoom zoom 19']
              and current()['macros']['dispatched'] == snapshot['macros']['dispatched'])
        ctl('command', 'meteor set zoom zoom 5')
        ctl('wait', 64)
        check('NBT replacement preserves queued old macro handle', zoom() == 7)
        execute('Generated Import Local')
        check('NBT replacement uses new live macro handle', zoom() == 19)
        import_file('empty.nbt', _bytes([]))
        snapshot = current()
        execute('Generated Import Local')
        check('Empty NBT clears live definitions without dispatch',
              current()['config']['macros']['macros'] == [] and
              current()['macros']['dispatched'] == snapshot['macros']['dispatched'])
        state('macros_nbt_complete')
    finally:
        ctl('command', 'meteor macro clear')
        # Restore this fresh test client's captured definitions through native JSON;
        # this also handles Unicode names that the ASCII fixture writer excludes.
        for index in reversed(range(len(current()['config']['macros']['macros']))):
            ctl('command', 'meteor macros remove ' + str(index))
        for macro in original['macros']['macros']:
            ctl('command', 'meteor macros add ' + json.dumps(macro))
        if any(m['id'] == 'zoom' and m['active']
               for m in current()['config']['modules']['modules']):
            ctl('command', 'meteor toggle zoom')
        for field in ('zoom', 'smooth'):
            ctl('command', 'meteor set zoom ' + field + ' ' + json.dumps(original['zoom'][field]))
        active = {m['id']: m['active'] for m in current()['config']['modules']['modules']}
        for module in original['modules']['modules']:
            if active[module['id']] != module['active']:
                ctl('command', 'meteor toggle ' + module['id'])
