"""Audit a reconstruction review ledger against current files, without scoring art.

Inspired by img2threejs pass gating. This independent client tool checks evidence
identity and coverage; a valid ledger cannot prove semantic likeness.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as f:
        for block in iter(lambda: f.read(1024*1024), b''):
            h.update(block)
    return h.hexdigest()


def audit(manifest, base):
    if manifest.get('version') != 1:
        raise ValueError('unsupported review ledger version')
    evidence = manifest.get('evidence', {})
    items = manifest.get('items', [])
    if not isinstance(evidence, dict) or not isinstance(items, list) or not evidence or not items:
        raise ValueError('nonempty evidence map and item list required')
    problems = []
    for name, row in evidence.items():
        path = base/row['path']
        if not path.is_file():
            problems.append(f'{name}: evidence file missing')
        elif digest(path) != row['sha256']:
            problems.append(f'{name}: evidence changed; review is stale')
    ids = [row['id'] for row in items]
    if any(not isinstance(v, str) or not v for v in ids) or len(ids) != len(set(ids)):
        raise ValueError('review item ids must be nonempty and unique')
    pending = []
    for row in items:
        if row.get('verdict') not in {'supported', 'divergent', 'unobserved', 'pending'}:
            raise ValueError(f"{row['id']}: unknown verdict")
        if not row.get('criterion') or not row.get('diagnosis'):
            problems.append(f"{row['id']}: missing criterion/diagnosis")
        if row['verdict'] != 'unobserved' and not row.get('evidence'):
            problems.append(f"{row['id']}: missing evidence references")
        for name in row.get('evidence', []):
            if name not in evidence:
                problems.append(f"{row['id']}: unknown evidence {name}")
        if row['verdict'] in {'divergent', 'pending'}:
            pending.append(row['id'])
    for dependency in manifest.get('required_items', []):
        if dependency not in ids:
            problems.append(f'missing planned item {dependency}')
    return {'evidence_current': not problems, 'problems': problems, 'remaining_items': pending,
            'recorded_items': len(items), 'all_observable_items_supported': not problems and not pending,
            'meaning': 'Evidence identity/declared coverage only. Verdicts are human/agent observations, not measurements.'}


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('ledger', type=Path)
    args = p.parse_args()
    result = audit(json.loads(args.ledger.read_text()), args.ledger.resolve().parent)
    print(json.dumps(result, indent=2))
    return 0 if result['evidence_current'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
