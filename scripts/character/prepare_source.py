#!/usr/bin/env python3
"""Assemble pinned CC0 anatomy data for the native Ignis character compiler.

Only data is reused from MakeHuman. No MakeHuman program code is imported.
The engine evaluates sparse shapes and fits the skeleton itself.
"""
import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
import math
from pathlib import Path
import urllib.request

ROOT = Path(__file__).resolve().parents[2]
DATA = ROOT / 'assets/characters/female-base'
OUTPUT = ROOT / 'artifacts/character-base'
REVISION = 'a8bc2d54ff0ac92e78ff71431b1023eda42bf482'
REMOTE = f'https://raw.githubusercontent.com/makehumancommunity/makehuman/{REVISION}/'


def specifications():
    """Each factor is an explicit interpolation basis over a named control."""
    def basis(parameter, level):
        return {'parameter': parameter, 'curve': [[0, int(level == 0)], [.5, int(level == 1)], [1, int(level == 2)]]}
    def signed(parameter, high):
        return {'parameter': parameter, 'curve': [[-1, int(not high)], [0, 0], [1, int(high)]]}
    specs = []
    for population in ['african', 'asian', 'caucasian']:
        specs.append((f'macrodetails/{population}-female-young.target', [], 1/3))
    for mi, muscle in enumerate(['minmuscle', 'averagemuscle', 'maxmuscle']):
        for wi, weight in enumerate(['minweight', 'averageweight', 'maxweight']):
            factors = [basis('musculature', mi), basis('adiposity', wi)]
            specs.append((f'macrodetails/universal-female-young-{muscle}-{weight}.target', factors, 1))
            for hi, height in [(0, 'minheight'), (2, 'maxheight')]:
                specs.append((f'macrodetails/height/female-young-{muscle}-{weight}-{height}.target', factors + [basis('stature', hi)], 1))
            for ci, cup in enumerate(['mincup', 'averagecup', 'maxcup']):
                for fi, firmness in enumerate(['minfirmness', 'averagefirmness', 'maxfirmness']):
                    if ci == fi == 1:
                        continue
                    specs.append((f'breast/female-young-{muscle}-{weight}-{cup}-{firmness}.target', factors + [basis('breast_volume', ci), basis('breast_shape_firmness', fi)], 1))
    pairs = {
        'hip_width': 'hip/hip-scale-horiz',
        'torso_width': 'torso/torso-scale-horiz', 'torso_depth': 'torso/torso-scale-depth',
        'shoulder_width': 'torso/torso-vshape', 'buttocks_volume': 'buttocks/buttocks-volume',
        'breast_separation': 'breast/breast-dist', 'breast_projection': 'breast/breast-point',
        'nose_width': 'nose/nose-scale-horiz', 'nose_length': 'nose/nose-scale-vert',
        'jaw_width': 'chin/chin-width', 'chin_projection': 'chin/chin-prominent',
        'mouth_width': 'mouth/mouth-scale-horiz',
    }
    for parameter, path in pairs.items():
        for suffix, high in [('decr', False), ('incr', True)]:
            specs.append((f'{path}-{suffix}.target', [signed(parameter, high)], 1))
    for side in ['l', 'r']:
        for segment in ['upperarm', 'lowerarm', 'upperleg', 'lowerleg']:
            for suffix, high in [('decr', False), ('incr', True)]:
                specs.append((f'armslegs/{side}-{segment}-scale-vert-{suffix}.target', [signed(segment + '_length', high)], 1))
    return specs


def retrieve(paths):
    manifest_path = DATA / 'provenance.json'
    previous = json.loads(manifest_path.read_text()) if manifest_path.exists() else {}
    if previous and previous['revision'] != REVISION:
        raise ValueError('Cannot merge graphical data from another revision')
    known = {row['path']: row for row in previous.get('files', [])}
    def one(path):
        destination = DATA / path
        if not destination.exists():
            destination.parent.mkdir(parents=True, exist_ok=True)
            data = urllib.request.urlopen(REMOTE + path, timeout=45).read()
            destination.write_bytes(data)
        row = {'path': path, 'sha256': hashlib.sha256(destination.read_bytes()).hexdigest()}
        if path in known and known[path] != row:
            raise ValueError('Pinned graphical data checksum changed: ' + path)
        return row
    with ThreadPoolExecutor(max_workers=6) as pool:
        rows = list(pool.map(one, paths))
    known.update((row['path'], row) for row in rows)
    manifest = {'repository': 'https://github.com/makehumancommunity/makehuman', 'revision': REVISION,
                'license': 'CC0-1.0 (graphical data only)', 'files': list(known.values())}
    (DATA / 'provenance.json').write_text(json.dumps(manifest, indent=2) + '\n')


def obj(path):
    positions, groups, group = [], {}, 'default'
    for line in path.read_text().splitlines():
        values = line.split()
        if not values:
            continue
        if values[0] == 'v':
            x, y, z = map(float, values[1:4])
            positions.append([-x * .1, y * .1, -z * .1])
        elif values[0] == 'g':
            group = values[1]
        elif values[0] == 'f':
            face = [int(v.split('/')[0]) - 1 for v in values[1:]]
            groups.setdefault(group, []).append(face)
    return positions, groups


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--fetch', action='store_true', help='fetch pinned graphical data and record checksums')
    args = parser.parse_args()
    specs = specifications()
    if args.fetch:
        tree = json.load(urllib.request.urlopen(f'https://api.github.com/repos/makehumancommunity/makehuman/git/trees/{REVISION}?recursive=1'))
        available = {x['path'] for x in tree['tree']}
        missing = [p for p, _, _ in specs if 'makehuman/data/targets/' + p not in available]
        if missing:
            raise ValueError('Unknown target paths: ' + ', '.join(missing))
        retrieve(['LICENSE.md', 'LICENSE.ASSETS.md', 'makehuman/data/3dobjs/base.obj',
                  'makehuman/data/rigs/default.mhskel', 'makehuman/data/rigs/default_weights.mhw'] +
                 ['makehuman/data/targets/' + p for p, _, _ in specs])
    manifest = json.loads((DATA / 'provenance.json').read_text())
    for row in manifest['files']:
        if hashlib.sha256((DATA / row['path']).read_bytes()).hexdigest() != row['sha256']:
            raise ValueError('Source checksum differs: ' + row['path'])
    positions, groups = obj(DATA / 'makehuman/data/3dobjs/base.obj')
    skeleton = json.loads((DATA / 'makehuman/data/rigs/default.mhskel').read_text())
    weights = json.loads((DATA / 'makehuman/data/rigs/default_weights.mhw').read_text())['weights']
    # Topological order is explicit, stable and independent from JSON object order.
    names = []
    def visit(name):
        if name in names:
            return
        parent = skeleton['bones'][name]['parent']
        if parent:
            visit(parent)
        names.append(name)
    for name in sorted(skeleton['bones']):
        visit(name)
    indices = {name: i for i, name in enumerate(names)}
    joints = [{'name': n.replace('.', '_'), 'parent': indices.get(skeleton['bones'][n]['parent']),
               'head': {'vertices': skeleton['joints'][skeleton['bones'][n]['head']]},
               'tail': {'vertices': skeleton['joints'][skeleton['bones'][n]['tail']]}} for n in names]
    vertex_weights = [[] for _ in positions]
    for name, entries in weights.items():
        for vertex, value in entries:
            if value > 0:
                vertex_weights[vertex].append((indices[name], value))
    def native_weight(rows):
        if not rows or len(rows) > 16:
            raise ValueError(f'Invalid influence count: {len(rows)}')
        total = sum(w for _, w in rows)
        rows = sorted((j, w/total) for j, w in rows)
        first = rows[:4] + [(0, 0)] * max(0, 4-len(rows))
        result = {'joints': [j for j, _ in first], 'weights': [w for _, w in first]}
        if len(rows) > 4:
            result['extra'] = [{'joint': j, 'weight': w} for j, w in rows[4:]]
        return result
    skin_faces = groups['body']
    vertices = sorted({v for f in skin_faces for v in f})
    remap = {v: i for i, v in enumerate(vertices)}
    triangles = [[remap[f[0]], remap[f[i]], remap[f[i+1]]] for f in skin_faces for i in range(1, len(f)-1)]
    part = {'name': 'body', 'vertices': vertices, 'triangles': triangles,
            'weights': [native_weight(vertex_weights[i]) for i in vertices], 'color': [.68, .47, .36],
            'material': {'reflectance': {'roughness': .66, 'f0': .025}}}
    parts = [part]
    # Independently authored ocular geometry, fitted to the anatomical eye joints.
    # Separate material caps keep an inspectable iris/pupil without texture support.
    for side in ['L', 'R']:
        joint = indices['eye.' + side]
        anchor = joints[joint]['head']['vertices']
        center = [sum(positions[i][a] for i in anchor)/len(anchor) for a in range(3)]
        for name, limit, radius, color in [('sclera', math.pi, .012, [.82, .81, .76]),
                                          ('iris', .48, .01208, [.19, .105, .045]),
                                          ('pupil', .18, .01216, [.008, .009, .012])]:
            first = len(positions)
            rings, segments = (20 if name == 'sclera' else 5), 40
            points = [[center[0], center[1], center[2]-radius]]
            for row in range(1, rings+1):
                theta = limit*row/(rings+1) if name == 'sclera' else limit*row/rings
                for col in range(segments):
                    phi = math.tau*col/segments
                    points.append([center[0]+radius*math.sin(theta)*math.cos(phi),
                                   center[1]+radius*math.sin(theta)*math.sin(phi),
                                   center[2]-radius*math.cos(theta)])
            faces = [[0, 1+(i+1)%segments, 1+i] for i in range(segments)]
            for row in range(rings-1):
                for col in range(segments):
                    a = 1+row*segments+col
                    b = 1+row*segments+(col+1)%segments
                    faces.extend([[a,b+segments,a+segments],[a,b,b+segments]])
            if name == 'sclera':
                end = len(points)
                points.append([center[0],center[1],center[2]+radius])
                faces.extend([[1+(rings-1)*segments+i,1+(rings-1)*segments+(i+1)%segments,end] for i in range(segments)])
            positions.extend(points)
            parts.append({'name':f'eye_{side}_{name}', 'vertices':list(range(first,len(positions))),
                          'triangles':faces,'weights':[native_weight([(joint,1)]) for _ in points],
                          'socket':joint,'color':color,'material':{'reflectance':{'roughness':.24,'f0':.04}}})
    targets, parameter_names = [], set()
    for path, factors, gain in specs:
        deltas = []
        for line in (DATA / 'makehuman/data/targets' / path).read_text().splitlines():
            v = line.split()
            if not v or v[0].startswith('#'):
                continue
            index, x, y, z = int(v[0]), *map(float, v[1:4])
            deltas.append({'vertex': index, 'offset': [-x*.1, y*.1, -z*.1]})
        parameter_names.update(f['parameter'] for f in factors)
        targets.append({'name': 't_' + str(len(targets)), 'gain': gain, 'factors': factors,
                        'deltas': sorted(deltas, key=lambda d: d['vertex'])})
    unsigned = {'musculature', 'adiposity', 'stature', 'breast_volume', 'breast_shape_firmness'}
    parameters = [{'name': n, 'minimum': 0 if n in unsigned else -1, 'maximum': 1,
                   'default': .5 if n in unsigned else 0} for n in sorted(parameter_names)]
    source = {'version': 1, 'id': 'female_adult_v1', 'provenance': f'MakeHuman CC0 anatomy, targets and rig; {REVISION}. Ignis compiler/configuration authored independently.',
              'positions': positions, 'parameters': parameters, 'targets': targets, 'joints': joints, 'parts': parts}
    OUTPUT.mkdir(parents=True, exist_ok=True)
    destination = OUTPUT / 'female.ignis-character'
    destination.write_text(json.dumps(source, separators=(',', ':')))
    print(json.dumps({'source': str(destination), 'bytes': destination.stat().st_size,
                      'parameters': len(parameters), 'targets': len(targets), 'bones': len(joints),
                      'skin_vertices': len(vertices), 'triangles': len(triangles),
                      'max_skin_influences': max(len(vertex_weights[i]) for i in vertices)}))


if __name__ == '__main__':
    main()
