#!/usr/bin/env python3
"""Add a configurable population of full geometric trees to the existing native mountain landscape.

Only the population is new: source terrain/light/entities are preserved. IGFT1 is
build input; runtime uses native mesh assets, GPU instances and ATM certificates.
"""
import argparse
import copy
import hashlib
import json
import math
import random
import struct
from collections import Counter
from pathlib import Path

from verify_lab import Client, ROOT, SCENE_VERSION

OUT = ROOT / 'artifacts/landscape-forest'
PREFIX = 'artifacts/landscape-forest/'
TREE = ROOT / 'examples/assets/forest-tree.bin'


def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


def tree_assets():
    data = TREE.read_bytes()
    assert data[:8] == b'IGFT1\0\0\0'
    nv, nt = struct.unpack_from('<II', data, 8)
    assert len(data) == 16 + nv * 12 + nt * 16
    vertices = list(struct.iter_unpack('<3f', data[16:16+nv*12]))
    faces = list(struct.iter_unpack('<4I', data[16+nv*12:]))
    low = min(p[1] for p in vertices)
    height = max(p[1] for p in vertices) - low
    assets = []
    for part, name, smooth in [(5, 'forest-bark', 'smooth'), (6, 'forest-leaves', 'flat')]:
        positions, triangles, remap = [], [], {}
        for *triangle, material in faces:
            if material != part:
                continue
            mapped = []
            for index in triangle:
                assert index < nv
                if index not in remap:
                    x, y, z = vertices[index]
                    remap[index] = len(positions)
                    positions.append([x, y-low, z])
                mapped.append(remap[index])
            triangles.append(mapped)
        assets.append(dict(id=name, geometry=dict(positions=positions, triangles=triangles), shading=smooth))
    assert sum(len(a['geometry']['triangles']) for a in assets) == nt == 20292
    return assets, height


class Mountain:
    """Exact barycentric sampling of the authored triangulated height field."""
    def __init__(self, instance, asset):
        self.instance = instance
        self.points = asset['geometry']['positions']
        self.n = math.isqrt(len(self.points))-1
        assert (self.n+1)**2 == len(self.points)
        self.scale = instance['scale']
        self.origin = instance['position']
        _, qy, _, qw = instance['rotation']
        self.cos, self.sin = 1-2*qy*qy, 2*qy*qw
        self.x0, self.z0 = self.points[0][0], self.points[0][2]
        self.dx = self.points[self.n][0]-self.x0
        self.dz = self.points[-1][2]-self.z0
        # Explicitly verify the diagonal/topology assumed by interpolation.
        expected = []
        for z in range(self.n):
            for x in range(self.n):
                a = z*(self.n+1)+x
                expected.extend([[a,a+self.n+1,a+1],[a+1,a+self.n+1,a+self.n+2]])
        assert expected == asset['geometry']['triangles']

    def world_xz(self, u, v):
        x, z = self.x0+u*self.dx, self.z0+v*self.dz
        return (self.origin[0]+self.scale*(self.cos*x+self.sin*z),
                self.origin[2]+self.scale*(-self.sin*x+self.cos*z))

    def height(self, x, z):
        x, z = (x-self.origin[0])/self.scale, (z-self.origin[2])/self.scale
        # Invert the actual quaternion-derived XZ matrix, including f32 roundoff.
        length = self.cos*self.cos+self.sin*self.sin
        x, z = (self.cos*x-self.sin*z)/length, (self.sin*x+self.cos*z)/length
        u, v = (x-self.x0)/self.dx*self.n, (z-self.z0)/self.dz*self.n
        if not (0 <= u <= self.n and 0 <= v <= self.n):
            return None
        ix, iz = min(int(u),self.n-1), min(int(v),self.n-1)
        u, v = u-ix, v-iz
        a = iz*(self.n+1)+ix
        ya, yb, yc, yd = [self.points[i][1] for i in [a,a+1,a+self.n+1,a+self.n+2]]
        y = ya+(yb-ya)*u+(yc-ya)*v if u+v <= 1 else yd+(yc-yd)*(1-u)+(yb-yd)*(1-v)
        return self.origin[1]+y*self.scale


def build(base, trees=10000):
    if trees < 1:
        raise ValueError("Tree count must be positive")
    scene = json.loads(base.read_text())
    source = copy.deepcopy(scene)
    assert not any(a['id'].startswith('forest-') for a in scene['meshes']['assets'])
    scene['version'] = SCENE_VERSION
    assert not scene['light'].get('atmosphere',{}).get('edge_softness')
    assets, source_height = tree_assets()
    by_id = {a['id']: a for a in scene['meshes']['assets']}
    mountains = [Mountain(i, by_id[i['asset']]) for i in scene['meshes']['instances'] if i['id'].startswith('range-')]
    assert len(mountains) == 5
    scene['meshes']['assets'].extend(assets)
    rng = random.Random(1737231)
    records, groups, occupied = [], [[], []], {}
    quotas = [5000, 2500, 1400, 700, 400]
    chain_counts = Counter()
    # Repeat the original seeded distribution so increasing the count preserves
    # every previously generated tree, instead of moving the existing population.
    for _batch in range((trees + 9999) // 10000):
        for chain, base_quota in enumerate(quotas):
            quota = min(base_quota, trees - len(records))
            attempts = 0
            accepted = 0
            while accepted < quota:
                attempts += 1
                assert attempts < 200000, 'Unable to place the requested population'
                if chain == 0 and accepted < 1000:
                    # A dense, human-scale patch near the previously approved box.
                    x, z = rng.gauss(424.,145.), rng.gauss(8145.,110.)
                else:
                    # Groves on both sides, with more coverage of camera-facing slopes.
                    u = rng.uniform(.08,.92)
                    v = rng.uniform(.44,.91) if rng.random() < .75 else rng.uniform(.10,.56)
                    x, z = mountains[chain].world_xz(u,v)
                x, z = round(x,3), round(z,3)
                if max(abs(x),abs(z)) > 9900 or math.hypot(x-424.0188,z-8144.873) < 8:
                    continue
                surfaces = [(h,j) for j,m in enumerate(mountains) if (h:=m.height(x,z)) is not None]
                if not surfaces:
                    continue
                y, actual_chain = max(surfaces)
                if y < 35:
                    continue
                # Keep distinct stems and some room around the scale reference.
                cell = (math.floor(x/7), math.floor(z/7))
                if any((x-px)**2+(z-pz)**2 < 7**2 for dx in [-1,0,1] for dz in [-1,0,1]
                       for px,pz in occupied.get((cell[0]+dx,cell[1]+dz),[])):
                    continue
                occupied.setdefault(cell,[]).append((x,z))
                height = round(rng.uniform(14,27),3)
                yaw = rng.random()*math.tau
                position = [x,round(y-.55,3),z]
                rotation = [0,round(math.sin(yaw/2),7),0,round(math.cos(yaw/2),7)]
                scale = round(height/source_height,5)
                color = [round(rng.uniform(.055,.115),3),round(rng.uniform(.14,.29),3),round(rng.uniform(.018,.055),3)]
                number = len(records)
                for part, asset, tint in [(0,'forest-bark',[.12,.055,.022]),(1,'forest-leaves',color)]:
                    groups[part].append(dict(id=f'tree-{number:05d}-{part}',asset=asset,position=position,
                        rotation=rotation,scale=scale,color=tint,playback=None))
                records.append(dict(id=number,chain=actual_chain,position=position,height_m=round(source_height*scale,3)))
                chain_counts[actual_chain] += 1
                accepted += 1
    assert len(records) == trees
    scene['meshes'].setdefault('objects', []).extend(
        dict(id=f'tree-{n:05d}', members=[f'tree-{n:05d}-0', f'tree-{n:05d}-1'])
        for n in range(trees))
    for group in groups:
        scene['meshes']['instances'].extend(group)
    assert scene['meshes']['instances'][:len(source['meshes']['instances'])] == source['meshes']['instances']
    assert scene['meshes']['assets'][:len(source['meshes']['assets'])] == source['meshes']['assets']
    assert scene['entities'] == source['entities'] and scene['light'] == source['light']
    # Start above the dense patch: foreground canopy and the original mountain horizon.
    scene['camera'] = dict(position=[445,920,8280],target=[390,880,7780],fov_degrees=60)
    OUT.mkdir(parents=True,exist_ok=True)
    encoded = json.dumps(scene,separators=(',',':')).encode()
    (OUT/'forest.scene.json').write_bytes(encoded)
    report = dict(seed=1737231,base=str(base),base_sha256=hashlib.sha256(base.read_bytes()).hexdigest(),
        source_tree_sha256=hashlib.sha256(TREE.read_bytes()).hexdigest(),trees=len(records),
        tree_mesh_instances=sum(map(len,groups)),total_mesh_instances=len(scene['meshes']['instances']),
        tree_triangles_each=20292,tree_triangles_total=20292*trees,
        original_landscape_preserved=True,original_entities_sha256=digest(source['entities']),
        original_light_sha256=digest(source['light']),by_chain=dict(chain_counts),
        bounds=[[min(r['position'][axis] for r in records),max(r['position'][axis] for r in records)] for axis in range(3)],
        height_range_m=[min(r['height_m'] for r in records),max(r['height_m'] for r in records)],
        root_depth_m=.55,minimum_stem_spacing_m=7,scene_bytes=len(encoded),camera=scene['camera'],
        limitations=['visual meshes without tree colliders or wind','same source species with scale/yaw/color variation',
                     'finite existing landscape, no new streaming/culling','trees distributed in groves, not uniform density across every square kilometre'])
    (OUT/'population.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report),flush=True)
    return report


def compile_and_capture(binary, report):
    client = Client(binary)
    try:
        client.request('initialize',dict(protocolVersion='2025-11-25',capabilities={},clientInfo=dict(name='landscape-forest',version='1')))
        client.request('notifications/initialized',notification=True)
        client.call('load_scene',path=PREFIX+'forest.scene.json')
        client.call('edit_scene',edits=[dict(op='compile_distant_meshes')])
        client.call('save_scene',path=PREFIX+'forest.scene.json')
        client.call('load_scene',path=PREFIX+'forest.scene.json')
        meta = client.call('capture',path=PREFIX+'forest.png',width=1728,height=1117)
        report.update(build_id=client.call('inspect')['build_id'],capture=meta,
                      persisted_scene_bytes=(OUT/'forest.scene.json').stat().st_size,
                      native_reload_verified=True)
        (OUT/'verification.json').write_text(json.dumps(report,indent=2)+'\n')
        print(json.dumps(dict(captured=True,adapter=meta['adapter'])),flush=True)
    finally:
        client.close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--base',type=Path,default=ROOT/'artifacts/landscape-live/landscape.scene.json')
    parser.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis')
    parser.add_argument('--build-only',action='store_true')
    parser.add_argument('--trees',type=int,default=10000)
    parser.add_argument('--output',type=Path,default=OUT)
    args = parser.parse_args()
    OUT = args.output.resolve()
    PREFIX = OUT.relative_to(ROOT).as_posix() + "/"
    report = build(args.base,args.trees)
    if not args.build_only:
        compile_and_capture(args.binary,report)
