#!/usr/bin/env python3
"""Audit/freeze only the explicitly copied foundation bundle; never build/install."""
import argparse
import json
from pathlib import Path

import package_native_hnmod as package


def freeze(bundle, notes):
    entries = package.collect(bundle, require_frozen=False)
    provenance = json.loads(entries[package.PROVENANCE])
    if provenance.get('status') == 'frozen-source-awaiting-validation':
        raise ValueError('Provenance already frozen; preserve that snapshot')
    initial = {r['destination']: r for r in provenance['initial_copies']}
    if set(notes) - set(entries) or any(not isinstance(v, str) or not v.strip() for v in notes.values()):
        raise ValueError('Change notes must name existing payloads and contain descriptions')
    changes = []
    for name, data in sorted(entries.items()):
        if name == package.PROVENANCE:
            continue
        if name.endswith('.rs') and (name not in initial or initial[name]['initial_candidate_sha256'] != package.sha(data)):
            if name not in notes:
                raise ValueError('Explicit production adaptation note required: ' + name)
        if name in notes:
            changes.append({'path': name, 'description': notes[name], 'sha256': package.sha(data)})
    provenance['adaptation_notes'] = changes
    provenance['frozen_payloads'] = [{'path': name, 'bytes': len(data), 'sha256': package.sha(data)}
                                     for name, data in sorted(entries.items()) if name != package.PROVENANCE]
    provenance['status'] = 'frozen-source-awaiting-validation'
    provenance['freeze_scope'] = 'Payload bytes frozen; not a parser/plan/apply/compile/runtime result. Provenance self-hash is covered by the external package report.'
    path = Path(bundle) / package.PROVENANCE
    path.write_text(json.dumps(provenance, indent=2) + '\n', encoding='utf-8')
    return package.collect(bundle)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', type=Path, default=Path(__file__).resolve().parents[1] / 'bundle')
    parser.add_argument('--structural-only', action='store_true', help='Do not require final provenance; no package-ready claim')
    parser.add_argument('--freeze', action='store_true', help='Explicitly record held source payload hashes in provenance')
    parser.add_argument('--change-notes', type=Path, help='JSON path->description map, required with --freeze')
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    if args.report.exists():
        parser.error('Preserve existing report; choose a new output path')
    if args.freeze:
        if args.structural_only or not args.change_notes:
            parser.error('--freeze requires --change-notes and forbids --structural-only')
        entries = freeze(args.bundle, json.loads(args.change_notes.read_text(encoding='utf-8')))
    else:
        entries = package.collect(args.bundle, require_frozen=not args.structural_only)
    result = {'status': 'PASS: structure only; source not frozen' if args.structural_only else 'PASS: explicit frozen payload contract',
              'entries': len(entries), 'expanded_bytes': sum(map(len, entries.values())),
              'payloads': [{'path': p, 'bytes': len(b), 'sha256': package.sha(b)} for p, b in sorted(entries.items())],
              'scope': 'No upstream stock parser, compile, installer or game was executed.'}
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({k: v for k, v in result.items() if k != 'payloads'}, indent=2))


if __name__ == '__main__':
    main()
