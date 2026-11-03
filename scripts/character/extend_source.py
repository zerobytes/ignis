#!/usr/bin/env python3
"""Build source2: bounded facial fields, regional mass/definition and rest posture.

Only pinned graphical data is acquired by --fetch. All deformation assembly,
landmark fields, waist shaping and posture masks here are authored for Ignis.
The source1 file remains intact for existing scenes and reversible migration.
"""
import argparse
import copy
import hashlib
import json
import math
from pathlib import Path
import urllib.request

ROOT = Path(__file__).resolve().parents[2]
DATA = ROOT / 'assets/characters/female-base'
OUT = ROOT / 'artifacts/character-base'
REVISION = 'a8bc2d54ff0ac92e78ff71431b1023eda42bf482'
EXTRA = ([f'armslegs/{side}-{segment}-muscle-incr.target'
          for side in ['l', 'r'] for segment in ['upperarm', 'lowerarm', 'upperleg', 'lowerleg', 'upperarm-shoulder']]
         + ['torso/torso-muscle-dorsi-incr.target', 'torso/torso-muscle-pectoral-incr.target', 'stomach/stomach-tone-incr.target'])


def acquire():
    manifest = json.loads((DATA / 'provenance.json').read_text())
    rows = {r['path']: r for r in manifest['files']}
    for relative in EXTRA:
        path = 'makehuman/data/targets/' + relative
        file = DATA / path
        if not file.exists():
            file.parent.mkdir(parents=True, exist_ok=True)
            file.write_bytes(urllib.request.urlopen(f'https://raw.githubusercontent.com/makehumancommunity/makehuman/{REVISION}/{path}', timeout=45).read())
        row = {'path': path, 'sha256': hashlib.sha256(file.read_bytes()).hexdigest()}
        if path in rows and rows[path] != row:
            raise ValueError('Pinned graphical data checksum changed: ' + path)
        rows[path] = row
    manifest['files'] = list(rows.values())
    (DATA / 'provenance.json').write_text(json.dumps(manifest, indent=2) + '\n')


def response(curve, x):
    if x <= curve[0][0]:
        return curve[0][1]
    for a, b in zip(curve, curve[1:]):
        if x <= b[0]:
            return a[1] + (b[1]-a[1]) * (x-a[0])/(b[0]-a[0])
    return curve[-1][1]


def neutral(source):
    points = copy.deepcopy(source['positions'])
    values = {p['name']: p['default'] for p in source['parameters']}
    for t in source['targets']:
        gain = t['gain'] * math.prod(response(f['curve'], values[f['parameter']]) for f in t['factors'])
        for d in t['deltas'] if gain else []:
            for a in range(3):
                points[d['vertex']][a] += gain*d['offset'][a]
    return points


def read_target(path):
    result = {}
    for line in (DATA / 'makehuman/data/targets' / path).read_text().splitlines():
        fields = line.split()
        if fields and not fields[0].startswith('#'):
            i, x, y, z = int(fields[0]), *map(float, fields[1:4])
            result[i] = [-x*.1, y*.1, -z*.1]
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--fetch', action='store_true')
    args = parser.parse_args()
    if args.fetch:
        acquire()
    manifest = json.loads((DATA / 'provenance.json').read_text())
    for row in manifest['files']:
        if hashlib.sha256((DATA / row['path']).read_bytes()).hexdigest() != row['sha256']:
            raise ValueError('Source checksum differs: ' + row['path'])
    base = json.loads((OUT / 'female.ignis-character').read_text())
    if base['version'] != 1:
        raise ValueError('Expected the original source1 fixture')
    source = copy.deepcopy(base)
    source.update(version=2, id='female_adult_v2', handles=[], warps=[])
    source['provenance'] += ' Source2: Ignis-authored waist, bounded face landmarks, extreme mass extensions and posture masks; additional pinned CC0 muscle targets.'
    for p in source['parameters']:
        if p['name'] in ['adiposity', 'musculature']:
            p['maximum'] = 2
    for t in source['targets']:
        for f in t['factors']:
            if f['parameter'] in ['adiposity', 'musculature']:
                f['curve'].append([2, f['curve'][-1][1]])
    positions = neutral(base)
    skin = next(p for p in source['parts'] if p['name'] == 'body')
    skin_ids = set(skin['vertices'])
    original_count = min(p['vertices'][0] for p in source['parts'] if p.get('socket') is not None)
    skin_weights = dict(zip(skin['vertices'], skin['weights']))
    joints = {j['name']: i for i, j in enumerate(source['joints'])}

    def parameter(name, minimum=-1, maximum=1, default=0):
        source['parameters'].append(dict(name=name, minimum=minimum, maximum=maximum, default=default))

    def factor(name, curve=None):
        return dict(parameter=name, curve=curve or [[-1, -1], [0, 0], [1, 1]])

    def target(name, vectors, factors, gain=1):
        deltas = [dict(vertex=i, offset=[round(v, 9) for v in v]) for i, v in sorted(vectors.items())
                  if max(map(abs, v)) > 1e-9]
        source['targets'].append(dict(name=name, gain=gain, factors=factors, deltas=deltas))

    # Extend beyond the existing macro endpoints, with muscle-conditioned fat shapes.
    for level, muscle in enumerate(['minmuscle', 'averagemuscle', 'maxmuscle']):
        maximum = read_target(f'macrodetails/universal-female-young-{muscle}-maxweight.target')
        average = read_target(f'macrodetails/universal-female-young-{muscle}-averageweight.target')
        vectors = {i: [2.75*(maximum.get(i, [0, 0, 0])[a]-average.get(i, [0, 0, 0])[a]) for a in range(3)]
                   for i in maximum.keys() | average.keys()}
        basis = [[0, int(level == 0)], [.5, int(level == 1)], [1, int(level == 2)], [2, int(level == 2)]]
        target(f'extreme_adiposity_{level}', vectors, [factor('adiposity', [[0, 0], [1, 0], [2, 1]]), factor('musculature', basis)])
    # The upstream female maximum concentrates mass on hips/thighs. Additional
    # abdominal/flank/back tissue gives the new endpoint an obese torso as well.
    fat_tissue = {}
    for i in skin_ids:
        x, y, z = positions[i]
        upper = max(0, min(1, (.60-y)/.15))
        lower = max(0, min(1, (y+.20)/.12))
        lateral = max(0, min(1, (.28-abs(x))/.10))
        w = math.exp(-((y-.12)/.29)**2)*upper*lower*lateral
        front = max(0, min(1, (-z-.015)/.06))
        dx = x*.90*w
        dz = ((z+.015)*1.35-.15*front)*w
        neck = math.exp(-((y-.64)/.075)**2) * max(0, 1-(abs(x)/.10)**2)
        if .51<y<.73:
            dx += x*.38*neck
            dz -= .025*neck*max(0,min(1,(-z-.015)/.05))
        if abs(dx)+abs(dz)>1e-8:
            fat_tissue[i]=[dx,0,dz]
    target('abdominal_flank_adiposity',fat_tissue,[factor('adiposity',[[0,0],[1,0],[2,1]])])
    for level, weight in enumerate(['minweight', 'averageweight', 'maxweight']):
        maximum = read_target(f'macrodetails/universal-female-young-maxmuscle-{weight}.target')
        average = read_target(f'macrodetails/universal-female-young-averagemuscle-{weight}.target')
        vectors = {i: [1.5*(maximum.get(i, [0, 0, 0])[a]-average.get(i, [0, 0, 0])[a]) for a in range(3)]
                   for i in maximum.keys() | average.keys()}
        basis = [[0, int(level == 0)], [.5, int(level == 1)], [1, int(level == 2)], [2, int(level == 2)]]
        target(f'extreme_musculature_{level}', vectors, [factor('musculature', [[0, 0], [1, 0], [2, 1]]), factor('adiposity', basis)])
    parameter('muscle_definition', 0, 1)
    for i, path in enumerate(EXTRA):
        vectors = read_target(path)
        target(f'regional_muscle_mass_{i}', vectors, [factor('musculature', [[0, 0], [1, 0], [2, 1]])], .65)
        target(f'regional_muscle_definition_{i}', vectors, [factor('muscle_definition', [[0, 0], [1, 1]]),
               factor('adiposity', [[0, 1], [.5, 1], [1, .25], [2, .05]])], .35)
    # Smooth radial hypertrophy follows limb axes, tapering at tendons/joints.
    # It adds bulk without multiplying sharp regional sculpt endpoints.
    segments=[]
    for side in ['L','R']:
        for region,amount in [('upperarm',.65),('lowerarm',.45),('upperleg',.42),('lowerleg',.45)]:
            first=source['joints'][joints[region+'01_'+side]]
            last=source['joints'][joints[region+'02_'+side]]
            a=[sum(positions[v][axis] for v in first['head']['vertices'])/len(first['head']['vertices']) for axis in range(3)]
            b=[sum(positions[v][axis] for v in last['tail']['vertices'])/len(last['tail']['vertices']) for axis in range(3)]
            axis=[b[i]-a[i] for i in range(3)];length=math.sqrt(sum(v*v for v in axis));axis=[v/length for v in axis]
            segments.append((region,side,a,axis,length,amount))
    hypertrophy={}
    for vertex,row in skin_weights.items():
        influences=list(zip(row['joints'],row['weights']))+[(v['joint'],v['weight']) for v in row.get('extra',[])]
        delta=[0.,0.,0.]
        for region,side,a,axis,length,amount in segments:
            weight=sum(value for bone,value in influences if source['joints'][bone]['name'].startswith(region) and source['joints'][bone]['name'].endswith('_'+side))
            if weight<=.0001:
                continue
            relative=[positions[vertex][i]-a[i] for i in range(3)]
            distance=sum(relative[i]*axis[i] for i in range(3));t=min(1,max(0,distance/length))
            envelope=math.sin(math.pi*t)**.8
            for i in range(3):
                delta[i]+=(relative[i]-axis[i]*distance)*amount*envelope*weight
        if max(map(abs,delta))>1e-8:
            hypertrophy[vertex]=delta
    target('smooth_limb_hypertrophy',hypertrophy,[factor('musculature',[[0,0],[1,0],[2,1]])])
    # Modelled abdominal relief, independent of shader/lighting and gated by definition.
    abs_vectors = {}
    for i in skin_ids:
        x, y, z = positions[i]
        if -.10 < y < .29 and -.20 < z < -.045 and abs(x) < .13:
            front = min(1, max(0, (-z-.045)/.045))
            lobes = sum(math.exp(-((x-side*.044)/.030)**2-((y-row)/.029)**2)
                        for side in [-1, 1] for row in [.0, .075, .15, .225])
            abs_vectors[i] = [0, 0, -.010*lobes*front]
    target('abdominal_relief', abs_vectors, [factor('muscle_definition', [[0, 0], [1, 1]]),
           factor('adiposity', [[0, 1], [.5, .8], [1, .1], [2, 0]])])

    parameter('waist_width')
    waist = {}
    for i, (x, y, z) in enumerate(positions[:original_count]):
        if -.015 < y < .315 and abs(x) < .24:
            w = max(0, 1-((y-.155)/.16)**2)**2
            w *= min(1, max(0, (.24-abs(x))/.055))
            waist[i] = [x*.52*w, 0, (z+.015)*.40*w]
    target('independent_waist', waist, [factor('waist_width')])

    def smooth(a, b, value):
        t = min(1, max(0, (value-a)/(b-a)))
        return t*t*(3-2*t)

    # Each hinge affects its whole descendant region (arms included), while feet
    # remain outside all masks. Helper anchors receive the same regional motion.
    for name, joint, lo, hi, angle in [
        ('posture_pelvis', 'spine05', -.06, .13, 13),
        ('posture_lumbar', 'spine03', .08, .26, 14),
        ('posture_thoracic', 'spine02', .21, .47, 18),
        ('posture_head_pitch', 'neck03', .59, .73, 16),
    ]:
        parameter(name)
        weights = []
        for i, (x, y, z) in enumerate(positions[:original_count]):
            w = smooth(lo, hi, y)
            if name in ['posture_pelvis', 'posture_lumbar', 'posture_thoracic'] and i in skin_ids:
                # Native skin weights identify arm descendants below the hinge height.
                row = skin_weights[i]
                influences = list(zip(row['joints'], row['weights'])) + [(v['joint'], v['weight']) for v in row.get('extra', [])]
                arm = sum(value for bone, value in influences if any(k in source['joints'][bone]['name'] for k in ['arm', 'hand', 'finger', 'thumb', 'shoulder', 'clavicle']))
                w = max(w, arm)
            # Lateral helper points belong to the arms, too.
            if name in ['posture_pelvis', 'posture_lumbar', 'posture_thoracic'] and abs(x) > .23 and y > -.1:
                w = 1
            if w > 1e-7:
                weights.append(dict(vertex=i, weight=round(w, 7)))
        source['warps'].append(dict(name=name, parameter=name, origin=source['joints'][joints[joint]]['head'],
                                    axis=[1, 0, 0], angle_degrees=angle, weights=weights))
    parameter('posture_head_retraction')
    target('head_retraction', {i: [0, 0, .065*smooth(.52, .70, p[1])] for i, p in enumerate(positions[:original_count]) if p[1] > .52}, [factor('posture_head_retraction')])
    parameter('posture_shoulders')
    target('shoulder_retraction', {i: [0, 0, .065*smooth(.035, .14, abs(x))*smooth(-.10, .35, y)*(1-smooth(.56, .65, y))]
                                    for i, (x, y, z) in enumerate(positions[:original_count]) if abs(x) > .035 and -.10 < y < .65}, [factor('posture_shoulders')])

    eye_joint = source['joints'][joints['eye_R']]
    eye_y = sum(positions[i][1] for i in eye_joint['head']['vertices'])/len(eye_joint['head']['vertices'])
    head_joint = joints['head']
    def landmark(name, label, center, radii, limits, mirror=None, eye=None):
        # Each visible anchor is an actual vertex, so the shared depth picker can
        # hide far-side points without an x-ray overlay or guessed screen bounds.
        part = skin
        anchor = min(skin['vertices'], key=lambda i: sum(((positions[i][a]-center[a])/radii[a])**2 for a in range(3)))
        if eye:
            part = next(p for p in source['parts'] if p['name'] == f'eye_{eye}_sclera')
            anchor = part['vertices'][0]
        else:
            center = positions[anchor]
        parameters = ['face_'+name+'_'+a for a in ['x', 'y', 'z']]
        for p, limit in zip(parameters, limits):
            parameter(p, -limit, limit)
        weights = {}
        for i, p in enumerate(positions[:original_count]):
            if p[1] < eye_y-.23:
                continue
            d = sum(((p[a]-center[a])/radii[a])**2 for a in range(3))
            if d < 1:
                weights[i] = (1-d)**2
        if eye:
            j = source['joints'][joints['eye_'+eye]]
            for i in j['head']['vertices'] + j['tail']['vertices']:
                weights[i] = 1
        for axis in range(3):
            target('handle_'+name+'_'+str(axis), {i: [limits[axis]*w if a == axis else 0 for a in range(3)] for i, w in weights.items()},
                   [factor(parameters[axis], [[-limits[axis], -1], [0, 0], [limits[axis], 1]])])
        source['handles'].append(dict(name=name, label=label, part=part['name'], anchor=dict(vertices=[anchor]),
                                     parameters=parameters, limits=limits, mirror=mirror, midline=mirror is None, frame_joint=head_joint))

    for name, label, y, z, radius, limits in [
        ('forehead_upper', 'Testa superior', .105, -.095, [.07, .055, .065], [.018, .025, .025]),
        ('forehead_middle', 'Centro da testa', .063, -.11, [.05, .04, .045], [.015, .02, .022]),
        ('glabella', 'Entre as sobrancelhas', .012, -.136, [.027, .023, .033], [.010, .014, .015]),
        ('nose_bridge', 'Ponte do nariz', -.006, -.133, [.019, .035, .027], [.012, .018, .018]),
        ('nose_tip', 'Ponta do nariz', -.029, -.1513, [.023, .026, .033], [.012, .015, .025]),
        ('upper_lip', 'Centro do lábio superior', -.0562, -.1449, [.027, .014, .032], [.008, .010, .018]),
        ('lower_lip', 'Centro do lábio inferior', -.0674, -.1419, [.028, .014, .030], [.008, .012, .018]),
        ('chin', 'Ponta do queixo', -.0874, -.1337, [.044, .030, .045], [.018, .025, .030]),
    ]:
        landmark(name, label, [0, eye_y+y, z], radius, limits)
    paired = [
        ('temple', 'Têmpora', .067, .015, -.055, [.035, .040, .045], [.018, .018, .018]),
        ('forehead_side', 'Lateral da testa', .040, .070, -.097, [.035, .040, .040], [.018, .020, .020]),
        ('brow_inner', 'Sobrancelha interna', .018, .012, -.136, [.019, .018, .025], [.009, .014, .013]),
        ('brow_arch', 'Arco da sobrancelha', .037, .014, -.132, [.020, .018, .025], [.010, .015, .014]),
        ('brow_outer', 'Cauda da sobrancelha', .055, .008, -.116, [.023, .023, .030], [.013, .015, .015]),
        ('eye', 'Órbita e olho', .031, 0, -.124, [.030, .028, .042], [.012, .012, .012]),
        ('upper_lid', 'Pálpebra superior', .033, .008, -.131, [.021, .012, .026], [.004, .005, .005]),
        ('lower_lid', 'Pálpebra inferior', .033, -.010, -.130, [.021, .012, .026], [.004, .005, .005]),
        ('eye_inner', 'Canto interno do olho', .018, -.002, -.131, [.013, .016, .020], [.005, .006, .006]),
        ('eye_outer', 'Canto externo do olho', .048, -.002, -.114, [.016, .017, .022], [.007, .007, .007]),
        ('cheekbone', 'Maçã do rosto', .045, -.026, -.122, [.035, .030, .040], [.020, .020, .025]),
        ('cheek', 'Bochecha', .044, -.047, -.115, [.040, .030, .045], [.023, .022, .025]),
        ('jaw_angle', 'Ângulo da mandíbula', .058, -.080, -.030, [.034, .035, .047], [.023, .022, .022]),
        ('mouth_corner', 'Canto da boca', .024, -.0604, -.133, [.020, .017, .030], [.012, .012, .015]),
        ('nose_wing', 'Asa do nariz', .014, -.037, -.138, [.017, .017, .025], [.009, .010, .011]),
        ('ear_upper', 'Orelha superior', .079, .015, -.002, [.025, .035, .040], [.020, .022, .020]),
        ('ear_lobe', 'Lóbulo da orelha', .076, -.052, -.003, [.025, .027, .035], [.016, .018, .018]),
    ]
    for name, label, x, y, z, radius, limits in paired:
        for side, sign, text in [('L', -1, 'E'), ('R', 1, 'D')]:
            landmark(name+'_'+side, label+' · '+text, [sign*x, eye_y+y, z], radius, limits,
                     mirror=name+('_R' if side == 'L' else '_L'), eye=side if name == 'eye' else None)
    source['parameters'].sort(key=lambda p: p['name'])
    destination = OUT / 'female-extended.ignis-character'
    encoded = json.dumps(source, separators=(',', ':')).encode()
    if len(encoded) > 64*1024*1024:
        raise ValueError('Extended source exceeds64MiB')
    destination.write_bytes(encoded)
    (OUT / 'extended-default.json').write_text(json.dumps({'parameters': {p['name']: p['default'] for p in source['parameters']}}, indent=2)+'\n')
    result = dict(path=str(destination), bytes=len(encoded), parameters=len(source['parameters']), targets=len(source['targets']),
                  handles=len(source['handles']), warps=len(source['warps']), sha256=hashlib.sha256(encoded).hexdigest())
    (ROOT / 'artifacts/character-controls/source.json').write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps(result))


if __name__ == '__main__':
    main()
