#!/usr/bin/env python3
"""Build a small validator against a caller-supplied pristine Horizon source export."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile


def build(horizon_source, target_dir, cargo='cargo', offline=False):
    horizon_source = Path(horizon_source).resolve(strict=True)
    target_dir = Path(target_dir).resolve()
    parser = horizon_source / 'crates/horizon-mods/src/index/native.rs'
    dependency = horizon_source / 'crates/horizon-mods'
    if not parser.is_file() or not (dependency / 'Cargo.toml').is_file():
        raise ValueError('Horizon source must contain the stock horizon-mods native parser')
    if target_dir.is_relative_to(horizon_source) or horizon_source.is_relative_to(target_dir):
        raise ValueError('Validator target must be separate from the pristine Horizon source')
    # JSON basic-string quoting is valid TOML; forward slashes avoid Windows escape ambiguity.
    dependency_literal = json.dumps(dependency.as_posix(), ensure_ascii=False)
    manifest = (
        '[package]\nname = "meteor-standard-hnmod-validator"\nversion = "0.1.0"\n'
        'edition = "2024"\npublish = false\n\n[workspace]\n\n[dependencies]\n'
        f'horizon-mods = {{ path = {dependency_literal}, features = ["index"] }}\n'
        'serde_json = "=1.0.151"\n'
    )
    template = Path(__file__).resolve().parent / 'standard-validator/src/main.rs'
    with tempfile.TemporaryDirectory(prefix='meteor-standard-validator-') as temporary:
        root = Path(temporary)
        (root / 'src').mkdir()
        (root / 'Cargo.toml').write_text(manifest, encoding='utf-8', newline='\n')
        shutil.copyfile(template, root / 'src/main.rs')
        command = [str(cargo), 'build', '--manifest-path', str(root / 'Cargo.toml'),
                   '--target-dir', str(target_dir)]
        if offline:
            command.append('--offline')
        subprocess.run(command, check=True)
    executable = target_dir / 'debug' / ('meteor-standard-hnmod-validator.exe' if os.name == 'nt' else 'meteor-standard-hnmod-validator')
    if not executable.is_file():
        raise RuntimeError('Cargo succeeded without creating the expected validator')
    return executable


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--horizon-source', type=Path, required=True)
    parser.add_argument('--target-dir', type=Path, required=True)
    parser.add_argument('--cargo', default='cargo', help='Cargo executable; use your configured pinned toolchain')
    parser.add_argument('--offline', action='store_true', help='Require dependencies already present in Cargo cache')
    args = parser.parse_args()
    print(json.dumps({'validator': str(build(args.horizon_source, args.target_dir, args.cargo, args.offline))}, indent=2))
