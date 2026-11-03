#!/usr/bin/env python3
"""Explicit reauthoring proposal for the shared tree's first fork, no scene edits."""
import json
import argparse
import math
import os
import random
import subprocess
from pathlib import Path

PROJECT = Path(__file__).resolve().parents[1]
OUTPUT = PROJECT/'artifacts/tree-junction-proposal'
(OUTPUT/'artifacts').mkdir(parents=True, exist_ok=True)
os.environ['IGNIS_VERIFY_ROOT'] = str(OUTPUT)
from verify_lab import Client


def add(a, b): return [x+y for x, y in zip(a, b)]
def sub(a, b): return [x-y for x, y in zip(a, b)]
def mul(a, k): return [x*k for x in a]
def dot(a, b): return sum(x*y for x, y in zip(a, b))
def unit(a): return mul(a, 1/math.sqrt(dot(a, a)))
def trunk(t): return [0.18*math.sin(t*2.2)+0.10*t, 4.8*t, 0.15*math.sin(t*3)]
def tangent(t): return unit([0.396*math.cos(t*2.2)+0.1, 4.8, 0.45*math.cos(t*3)])
def radius(t): return 0.46*(1-t*21/24)**1.25+0.015


def split(points, t):
    levels = [points]
    while len(levels[-1]) > 1:
        levels.append([add(mul(a, 1-t), mul(b, t)) for a, b in zip(levels[-1], levels[-1][1:])])
    return [row[0] for row in levels], [row[-1] for row in levels[::-1]]


def bezier(points, t): return split(points, t)[0][-1]


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--samples',type=int,default=32)
    parser.add_argument('--refinement',type=int,default=1)
    parser.add_argument('--blend-radius',type=float,default=0.04)
    parser.add_argument('--center-radius',type=float,default=0.29)
    parser.add_argument('--output',type=Path,default=OUTPUT/'proposal.json')
    args=parser.parse_args()
    OUTPUT.mkdir(parents=True, exist_ok=True)
    rng = random.Random(20261026)
    for _ in range(8):
        rng.uniform(-0.12, 0.12); rng.uniform(0.8, 1.1)
    anchor = trunk(0.34); end = [-1.55, 5.25, 0.70]
    c1 = add(anchor, mul(sub(end, anchor), 0.28)); c1[1] += 0.45
    c2 = add(end, [rng.uniform(-0.45, 0.45), -1.15, rng.uniform(-0.4, 0.4)])
    source = [anchor, c1, c2, end]
    _, retained = split(source, 0.3)
    splice = retained[0]; outgoing = unit(sub(retained[1], retained[0]))
    side_center = add(anchor, [-0.52, 0.18, 0.02]); side_normal = unit([-0.9, 0.43, 0.04])
    # A new G1 neck, joining the unchanged original curve at old parameter0.30.
    neck = [side_center, add(side_center, mul(side_normal, 0.3)), sub(splice, mul(outgoing, 0.3)), splice]

    def collar(id, center, normal, r):
        radial = unit(sub([0, 0, 1], mul(normal, normal[2])))
        return {'id': id, 'center': center, 'normal': normal, 'radial': radial, 'radius': r, 'samples': args.samples}

    request = {'collars': [collar('trunk_below', trunk(0.24), mul(tangent(0.24), -1), radius(0.24)),
                            collar('limb_zero', side_center, side_normal, 0.19),
                            collar('trunk_above', trunk(0.42), tangent(0.42), radius(0.42))],
               'center': anchor, 'center_radius': args.center_radius, 'blend_radius': args.blend_radius,
               'guard_length': 0.025, 'max_displacement': 0.3}
    protected = []
    for t in [0.30, 0.44, 0.58, 0.72, 0.86, 1.0]:
        old = bezier(source, t); new = bezier(retained, (t-0.3)/0.7)
        error = math.dist(old, new); assert error < 1e-12
        protected.append({'source_t': t, 'retained_t': (t-0.3)/0.7, 'position': old, 'error': error})
    # Supplied source curves/anchors remain data; this bridge runs the Rust kernel.
    completed = subprocess.run([str(PROJECT/'target/release/examples/organic_fused_request')], input=json.dumps({**request,'refinement_levels':args.refinement}), capture_output=True, text=True, check=True)
    library = json.loads(completed.stdout)
    c = Client(PROJECT/'target/release/ignis', False)
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {}, 'clientInfo': {'name': 'ignis-tree-junction-proposal', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        before = c.call('inspect')
        result = c.call('preview_mesh_junction', collars=request['collars'], options={'refinement_levels': args.refinement},
                        fusion={k: v for k, v in request.items() if k != 'collars'})
        assert c.call('inspect') == before
        assert library['passed'] and result == library['junction']
        build_id = before['build_id']
        evaluated = {'passed': True, 'junction': result}
    finally:
        c.close()
    proposal = {'source': 'tree-authoring-study17fb9ed first primary fork', 'request': request, 'refinement_levels': args.refinement,
                'source_curve': source, 'new_neck': neck, 'retained_curve': retained,
                'protected_crown_anchors': protected,
                'decisions': ['Replace trunk only between parameters0.24 and0.42, before next fork0.44',
                              'Move only the first limb neck outward and lower; keep every existing crown anchor at parameters0.30..1',
                              'Join the retained curve with matching tangent direction, not an automatic source-vertex transfer',
                              'Reuse returned collar indices when generating adjoining sweeps; interior/wood IDs are rebuilt explicitly'],
                'evaluation': evaluated, 'scene_mutated': False, 'mcp': True, 'library_matched': True, 'build_id': build_id}
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_text(json.dumps(proposal, indent=2)+'\n')
    print(json.dumps({'passed': evaluated['passed'], 'scene_mutated': False, 'mcp': True, 'library_matched': True, 'build_id': build_id, 'anchor_errors': [p['error'] for p in protected],
                      'result': evaluated.get('junction', {}).get('report', evaluated.get('error'))}))
    assert evaluated['passed'], evaluated


if __name__ == '__main__':
    main()
