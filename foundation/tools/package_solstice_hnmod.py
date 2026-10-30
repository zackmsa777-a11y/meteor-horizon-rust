#!/usr/bin/env python3
"""Bounded normal schema-1 Meteor Horizon/Solstice source packager.

Independent from the held Meteor migration.4 contract. Packaging requires an
explicit provenance freeze; it never compiles, installs or executes mod source.
"""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path
import re
import tomllib
import zipfile

ID = "meteor_client"
VERSION = "0.1.0-solstice.2"
HORIZON = "b76ae35f5b9b2d82c0e6818fc0ef93bb5277ec73"
MIXINS = "cb1ad23022392d88f006cf94cbf6552a73f5df12"
PREFIX = "mixins/meteor_client/"
PROVENANCE = PREFIX + "data/solstice-provenance.json"
MAX_BYTES = 64 * 1024 * 1024
EXPECTED_MANIFEST = {"schema": 1, "id": ID, "version": VERSION, "game": "26.3", "horizon": HORIZON, "mixins": MIXINS}
NOTICES = {
    "licenses/Solstice-MIT.txt": "d83a80f6c77f9b699da7d8cffab3da3c80015f622c97514380b1a609b2e3890e",
    "project-notices/Horizon-MIT.txt": "b4b374ae9d0f2d8f7be35c6783e41327431da3b5662605da9ba31493630621df",
    "project-notices/Meteor-GPL-3.0-only.txt": "b1e60bceb8b63b265dfe040691bd5ec7a21da3c31be3db4b2925518e9c0836df",
    "project-notices/Starscript-MIT.txt": "0efbee07393749f2e2e96a16466b6b05c8244adba0cf97e6daf453e4fe5e9c3f",
}
DATA_PATHS = {PREFIX + "data/" + p for p in NOTICES} | {PROVENANCE, PREFIX + "data/component-licenses.json"}
HOST_SUPPORT_FILES = {
    "crates/horizon-client/src/game.rs", "crates/horizon-client/src/game/fishing.rs",
    "crates/horizon-client-render/src/mesher.rs", "crates/horizon-client/src/clock.rs",
    "crates/horizon-client-input/src/input.rs",
}


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


def collect(root, *, require_frozen=True):
    root = Path(root)
    if linked(root) or not root.is_dir():
        raise ValueError('Expected real bundle directory')
    entries, aliases = {}, set()
    for path in sorted(root.rglob('*')):
        name = path.relative_to(root).as_posix()
        if linked(path) or not safe_path(name) or name.upper() in aliases:
            raise ValueError('Linked, unsafe or aliased payload: ' + name)
        aliases.add(name.upper())
        if path.is_dir():
            continue
        if not path.is_file():
            raise ValueError('Non-file payload: ' + name)
        if path.stat().st_size > MAX_BYTES:
            raise ValueError('Payload exceeds stock limit')
        entries[name] = path.read_bytes()
        if len(entries) > 512 or sum(map(len, entries.values())) > MAX_BYTES:
            raise ValueError('Package exceeds stock 512-entry/64MiB limits')
    if json.loads(entries.get('horizon-native.json', b'{}')) != EXPECTED_MANIFEST:
        raise ValueError('Expected exact normal schema1 foundation manifest')
    mixin_path = PREFIX + 'horizon-mixin.toml'
    mixin = tomllib.loads(entries[mixin_path].decode('utf-8'))
    if mixin['mod']['id'] != ID or mixin['mod']['version'] != VERSION or mixin['compatibility']['horizon_revision'] != HORIZON:
        raise ValueError('Mixin/native identity/version/revision differs')
    if any(k in mixin for k in ('crate', 'dependency', 'post_build')):
        raise ValueError('Foundation contract adds no crate/dependency/post_build')
    sources = set()
    for record in mixin.get('module', []):
        if record['file'] not in HOST_SUPPORT_FILES:
            raise ValueError('Unreviewed support destination: ' + record['file'])
        source = record['source']
        if not safe_path(source) or '/' in source or not source.endswith('.rs'):
            raise ValueError('Support source must be a direct bounded Rust file')
        sources.add(PREFIX + source)
    for record in mixin.get('injection', []):
        source = record['helper']
        if not safe_path(source) or '/' in source or not source.endswith('.rs'):
            raise ValueError('Injection helper must be a direct bounded Rust file')
        sources.add(PREFIX + source)
    allowed = {'horizon-native.json', mixin_path} | DATA_PATHS | sources
    if set(entries) != allowed:
        raise ValueError('Unlisted or missing payload: ' + repr(sorted(set(entries) ^ allowed)))
    for name, data in entries.items():
        data.decode('utf-8')
        if b'\0' in data:
            raise ValueError('Expected text payload without NUL: ' + name)
    for name, digest in NOTICES.items():
        if sha(entries[PREFIX + 'data/' + name]) != digest:
            raise ValueError('Changed preserved component notice: ' + name)
    components = json.loads(entries[PREFIX + 'data/component-licenses.json'])
    if components['active_foundation']['license'] != 'MIT' or components['active_foundation']['copyright'] != 'Copyright (c) 2026 PointOfPressure':
        raise ValueError('Active foundation component attribution changed')
    provenance = json.loads(entries[PROVENANCE])
    if provenance['upstream']['commit'] != '2b11af4dfae7b08f9faa65d6fd3f4ab7b32030d1' or len(provenance['original_files']) != 40:
        raise ValueError('Expected complete pinned upstream source provenance')
    originals = provenance['original_files']
    if len({r['path'] for r in originals}) != 40 or any(not safe_path(r['path']) for r in originals):
        raise ValueError('Original provenance paths differ')
    checkout = ''.join(r['sha256'] + '  ' + r['path'] + '\n' for r in sorted(originals, key=lambda r: r['path'])).encode()
    blobs = ''.join(r['git_blob_bytes_sha256'] + '  ' + r['path'] + '\n' for r in sorted(originals, key=lambda r: r['path'])).encode()
    if sha(checkout) != 'a3e4c2dfb56349afafe1d8b011e289ed8924e16846736886c906364ba0241425' or sha(blobs) != 'dfa9db840119a7b85b86dfe4b8b9f0669608605178a9754ed49df965aa254820':
        raise ValueError('Pinned original source hashes differ')
    if provenance['candidate_native_manifest'] != EXPECTED_MANIFEST:
        raise ValueError('Provenance candidate identity/pins differ')
    if require_frozen:
        if provenance.get('status') != 'frozen-source-awaiting-validation':
            raise ValueError('Explicit provenance freeze is required before packaging')
        frozen = provenance.get('frozen_payloads', [])
        records = {r['path']: r for r in frozen}
        if len(records) != len(frozen) or set(records) != set(entries) - {PROVENANCE}:
            raise ValueError('Frozen provenance file set differs')
        for name, record in records.items():
            if record['sha256'] != sha(entries[name]) or record['bytes'] != len(entries[name]):
                raise ValueError('Frozen payload changed: ' + name)
    return entries


def build(bundle, output):
    entries = collect(bundle)
    output = Path(output)
    if output.exists():
        raise ValueError('Preserve existing archive; choose a new output path')
    output.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(output, 'x', compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
        for name, data in sorted(entries.items()):
            item = zipfile.ZipInfo(name, date_time=(2026, 1, 1, 0, 0, 0))
            item.compress_type = zipfile.ZIP_DEFLATED
            item.external_attr = 0o100644 << 16
            archive.writestr(item, data)
    return {'format': 'standard horizon-native.json schema1 .hnmod', 'manifest': EXPECTED_MANIFEST,
            'package': output.name, 'package_sha256': sha(output.read_bytes()), 'bytes': output.stat().st_size,
            'entries': len(entries), 'expanded_bytes': sum(map(len, entries.values())),
            'payloads': [{'path': p, 'bytes': len(b), 'sha256': sha(b)} for p, b in sorted(entries.items())],
            'scope': 'Source-package checks only; no compile/install/gameplay result.'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', type=Path, default=Path(__file__).resolve().parents[1] / 'bundle')
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    if args.report.exists():
        parser.error('Preserve existing report; choose a new output path')
    result = build(args.bundle, args.output)
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({k: v for k, v in result.items() if k != 'payloads'}, indent=2))


if __name__ == '__main__':
    main()
