#!/usr/bin/env python3
"""Package the migration bundle using Horizon's existing schema-1 .hnmod layout."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import tomllib
import zipfile

NATIVE_MANIFEST = "horizon-native.json"
MOD_ID = "meteor_client"
VERSION = "0.1.0-migration.1"
HORIZON = "b76ae35f5b9b2d82c0e6818fc0ef93bb5277ec73"
MIXINS = "cb1ad23022392d88f006cf94cbf6552a73f5df12"
MAX_BYTES = 64 * 1024 * 1024


def sha(data):
    return hashlib.sha256(data).hexdigest()


def safe_path(name):
    if not name or any(c in name for c in '\\:<>"|?*') or any(ord(c) < 32 for c in name):
        return False
    for part in name.split('/'):
        if not part or part in ('.', '..') or part.endswith(('.', ' ')):
            return False
        stem = part.split('.')[0].upper()
        if stem in {'CON', 'PRN', 'AUX', 'NUL'} or re.fullmatch(r'(COM|LPT)[1-9¹²³]', stem):
            return False
    return True


def linked(path):
    return path.is_symlink() or getattr(path, 'is_junction', lambda: False)()


def collect(root):
    root = Path(root)
    if linked(root) or not root.is_dir():
        raise ValueError('Bundle root must be a real directory')
    entries = {}
    aliases = set()
    for path in sorted(root.rglob('*')):
        name = path.relative_to(root).as_posix()
        if linked(path):
            raise ValueError('Linked bundle entry: ' + name)
        if not safe_path(name) or name.upper() in aliases:
            raise ValueError('Unsafe or aliased bundle path: ' + name)
        aliases.add(name.upper())
        if path.is_dir():
            continue
        if not path.is_file():
            raise ValueError('Non-file bundle entry: ' + name)
        entries[name] = path.read_bytes()
    expected = {'schema': 1, 'id': MOD_ID, 'version': VERSION, 'game': '26.3',
                'horizon': HORIZON, 'mixins': MIXINS}
    if json.loads(entries.get(NATIVE_MANIFEST, b'{}')) != expected:
        raise ValueError('Expected the normal pinned horizon-native.json schema-1 contract')
    prefix = f'mixins/{MOD_ID}/'
    for name, data in entries.items():
        if name == NATIVE_MANIFEST:
            continue
        if not name.startswith(prefix) or not (name.endswith(('.rs', '.toml')) or name.startswith(prefix + 'data/')):
            raise ValueError('Outside standard .hnmod Rust/TOML/data layout: ' + name)
        if name.endswith(('.jar', '.class', '.jfr', '.hprof', '.exe', '.dll')):
            raise ValueError('Executable or private game artifact: ' + name)
        if '/core/' in name and name.endswith('Cargo.lock'):
            raise ValueError('Bundle crate must not include Cargo.lock')
        if not name.startswith(prefix + 'data/'):
            data.decode('utf-8')
            if b'\0' in data:
                raise ValueError('Source must be UTF-8 text without NUL: ' + name)
    manifest = tomllib.loads(entries[prefix + 'horizon-mixin.toml'].decode())
    if manifest['mod']['id'] != MOD_ID or manifest['mod']['version'] != VERSION:
        raise ValueError('Mixin identity differs from native manifest')
    if manifest['compatibility']['horizon_revision'] != HORIZON or 'post_build' in manifest:
        raise ValueError('Proof must pin pristine Horizon and omit post_build')
    if not any(c == {'id': 'core', 'path': 'core', 'dependent': 'horizon-client'} for c in manifest.get('crate', [])):
        raise ValueError('Missing standard core bundle-crate declaration')
    cargo = tomllib.loads(entries[prefix + 'core/Cargo.toml'].decode())
    if cargo['package']['edition'] != '2024' or 'workspace' in cargo:
        raise ValueError('Core crate must declare edition 2024 and no workspace')
    for notice in ('Meteor-GPL-3.0-only.txt', 'Starscript-MIT.txt', 'Horizon-MIT.txt'):
        if not entries.get(prefix + 'data/licenses/' + notice):
            raise ValueError('Missing preserved notice: ' + notice)
    provenance = json.loads(entries[prefix + 'data/core-provenance.json'])
    for record in provenance['files']:
        name = prefix + record['destination']
        if sha(entries[name]) != record['package_sha256']:
            raise ValueError('Copied source differs from recorded provenance: ' + name)
    if len(entries) > 512 or sum(map(len, entries.values())) > MAX_BYTES:
        raise ValueError('Package exceeds standard .hnmod limits')
    return entries


def build(bundle, output):
    entries = collect(bundle)
    output = Path(output)
    output.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(output, 'w', compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
        for name, data in sorted(entries.items()):
            info = zipfile.ZipInfo(name, date_time=(2026, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            archive.writestr(info, data)
    return {'path': str(output.resolve()), 'sha256': sha(output.read_bytes()), 'entries': len(entries),
            'bytes': output.stat().st_size, 'format': 'standard horizon-native.json schema 1 .hnmod',
            'id': MOD_ID, 'version': VERSION, 'horizon': HORIZON, 'mixins': MIXINS}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', type=Path, default=Path(__file__).resolve().parents[1] / 'bundle')
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--report', type=Path, help='Write complete per-payload hashes for review')
    args = parser.parse_args()
    result = build(args.bundle, args.output)
    if args.report:
        entries = collect(args.bundle)
        report = dict(result, payloads=[{'path': name, 'bytes': len(data), 'sha256': sha(data)}
                                       for name, data in sorted(entries.items())])
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(result, indent=2))
