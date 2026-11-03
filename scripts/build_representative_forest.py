#!/usr/bin/env python3
"""Build-time vegetation on published native terrain; renderer/physics unchanged."""
import argparse
import bisect
import copy
import hashlib
import json
import math
import random
from collections import defaultdict
from pathlib import Path

from build_landscape_forest import tree_assets
from verify_lab import Client, ROOT, SCENE_VERSION


def world(instance, p):
    x, y, z = (v*instance.get('scale', 1) for v in p)
    qx, qy, qz, qw = instance.get('rotation', [0, 0, 0, 1])
    tx, ty, tz = 2*(qy*z-qz*y), 2*(qz*x-qx*z), 2*(qx*y-qy*x)
    return (x+qw*tx+qy*tz-qz*ty+instance['position'][0],
            y+qw*ty+qz*tx-qx*tz+instance['position'][1],
            z+qw*tz+qx*ty-qy*tx+instance['position'][2])


class Grid:
    """Exact source diagonal/height interpolation, including rigid Y rotation."""
    def __init__(self, instance, asset):
        self.instance, self.points = instance, asset['geometry']['positions']
        p = self.points
        self.width = next((i for i in range(1, len(p)) if p[i][2] != p[0][2]), len(p))
        assert self.width > 1 and len(p) % self.width == 0
        self.depth = len(p)//self.width
        assert self.depth > 1
        self.x0, self.z0 = p[0][0], p[0][2]
        self.xs = [v[0] for v in p[:self.width]]
        self.zs = [p[row*self.width][2] for row in range(self.depth)]
        assert all(b>a for a,b in zip(self.xs,self.xs[1:]))
        assert all(b>a for a,b in zip(self.zs,self.zs[1:]))
        for row in range(self.depth):
            for col in range(self.width):
                v = p[row*self.width+col]
                assert abs(v[0]-self.xs[col]) < .001
                assert abs(v[2]-self.zs[row]) < .001
        triangles = asset['geometry']['triangles']
        assert len(triangles) == 2*(self.width-1)*(self.depth-1)
        for row in range(self.depth-1):
            for col in range(self.width-1):
                a = row*self.width+col
                i = 2*(row*(self.width-1)+col)
                assert triangles[i] == [a, a+self.width, a+1]
                assert triangles[i+1] == [a+1, a+self.width, a+self.width+1]
        qx, qy, qz, qw = instance.get('rotation', [0, 0, 0, 1])
        assert abs(qx)+abs(qz) < 1e-7, 'Placement grid must remain a height surface'
        self.cos, self.sin = 1-2*qy*qy, 2*qy*qw
        corners = [world(instance, p[i]) for i in [0, self.width-1, len(p)-self.width, len(p)-1]]
        self.bounds = [min(v[0] for v in corners), min(v[2] for v in corners),
                       max(v[0] for v in corners), max(v[2] for v in corners)]

    def sample(self, x, z):
        i = self.instance
        x, z = (x-i['position'][0])/i.get('scale', 1), (z-i['position'][2])/i.get('scale', 1)
        norm = self.cos*self.cos+self.sin*self.sin
        x, z = (self.cos*x-self.sin*z)/norm, (self.sin*x+self.cos*z)/norm
        if not (self.xs[0]-1e-6 <= x <= self.xs[-1]+1e-6 and self.zs[0]-1e-6 <= z <= self.zs[-1]+1e-6):
            return None
        x, z = min(self.xs[-1],max(self.xs[0],x)), min(self.zs[-1],max(self.zs[0],z))
        col = min(self.width-2,max(0,bisect.bisect_right(self.xs,x)-1))
        row = min(self.depth-2,max(0,bisect.bisect_right(self.zs,z)-1))
        dx, dz = self.xs[col+1]-self.xs[col], self.zs[row+1]-self.zs[row]
        u, v = (x-self.xs[col])/dx, (z-self.zs[row])/dz
        a = row*self.width+col
        ya, yb, yc, yd = [self.points[k][1] for k in [a, a+1, a+self.width, a+self.width+1]]
        if u+v <= 1:
            y = ya+(yb-ya)*u+(yc-ya)*v
            gx, gz = (yb-ya)/dx, (yc-ya)/dz
        else:
            y = yd+(yc-yd)*(1-u)+(yb-yd)*(1-v)
            gx, gz = (yd-yc)/dx, (yd-yb)/dz
        return i['position'][1]+y*i.get('scale', 1), 1/math.sqrt(1+gx*gx+gz*gz)


class Water:
    def __init__(self, scene, ids, bounds, margin=6):
        self.triangles, self.bins, self.margin = [], defaultdict(list), margin
        self.cell_size = max(32, max(bounds[2]-bounds[0],bounds[3]-bounds[1])/128)
        assets = {a['id']: a for a in scene['meshes']['assets']}
        for instance in scene['meshes']['instances']:
            if instance['id'] not in ids:
                continue
            geometry = assets[instance['asset']]['geometry']
            points = [world(instance, p) for p in geometry['positions']]
            for face in geometry['triangles']:
                t = [points[i] for i in face]
                x0 = max(bounds[0], min(p[0] for p in t)-margin)
                z0 = max(bounds[1], min(p[2] for p in t)-margin)
                x1 = min(bounds[2], max(p[0] for p in t)+margin)
                z1 = min(bounds[3], max(p[2] for p in t)+margin)
                if x1 < x0 or z1 < z0:
                    continue
                n = len(self.triangles)
                self.triangles.append(t)
                # Clip the ocean's huge footprint to the placement envelope.
                for x in range(math.floor(x0/self.cell_size), math.floor(x1/self.cell_size)+1):
                    for z in range(math.floor(z0/self.cell_size), math.floor(z1/self.cell_size)+1):
                        self.bins[x,z].append(n)

    def excludes(self, x, y, z):
        for n in self.bins.get((math.floor(x/self.cell_size), math.floor(z/self.cell_size)), ()):
            a, b, c = self.triangles[n]
            determinant = (b[2]-c[2])*(a[0]-c[0])+(c[0]-b[0])*(a[2]-c[2])
            if abs(determinant) < 1e-12:
                continue
            u = ((b[2]-c[2])*(x-c[0])+(c[0]-b[0])*(z-c[2]))/determinant
            v = ((c[2]-a[2])*(x-c[0])+(a[0]-c[0])*(z-c[2]))/determinant
            if min(u, v, 1-u-v) >= -1e-9:
                if y <= u*a[1]+v*b[1]+(1-u-v)*c[1]+2:
                    return True
            else:
                for p, q in [(a,b), (b,c), (c,a)]:
                    dx, dz = q[0]-p[0], q[2]-p[2]
                    t = max(0, min(1, ((x-p[0])*dx+(z-p[2])*dz)/max(dx*dx+dz*dz, 1e-20)))
                    if ((x-p[0]-t*dx)**2+(z-p[2]-t*dz)**2 <= self.margin**2
                            and y <= p[1]+t*(q[1]-p[1])+2):
                        return True
        return False


def grove(x, z, seed):
    x, z = x/140, z/140
    ix, iz = math.floor(x), math.floor(z)
    u, v = x-ix, z-iz
    u, v = u*u*(3-2*u), v*v*(3-2*v)
    def at(a, b):
        h = (a*0x9e3779b1 ^ b*0x85ebca77 ^ seed) & 0xffffffff
        h = ((h ^ (h >> 16))*0x45d9f3b) & 0xffffffff
        return (h ^ (h >> 16))/0xffffffff
    return ((1-u)*at(ix,iz)+u*at(ix+1,iz))*(1-v)+((1-u)*at(ix,iz+1)+u*at(ix+1,iz+1))*v


def shared_edges(grids, assets):
    """Check only actual matching grid boundaries, not inferred height fields."""
    edges, reports = {}, []
    for grid in grids:
        width, depth = grid.width, grid.depth
        geometry = assets[grid.instance['asset']]['geometry']
        for indices in [list(range(width)),list(range((depth-1)*width,depth*width)),
                        [r*width for r in range(depth)],[r*width+width-1 for r in range(depth)]]:
            points = [world(grid.instance,grid.points[i]) for i in indices]
            coords = [geometry['material_coordinates'][i] for i in indices] if geometry.get('material_coordinates') else []
            ends = [tuple(round(p[k],5) for k in [0,2]) for p in [points[0],points[-1]]]
            if ends[0] > ends[1]:
                ends.reverse();points.reverse();coords.reverse()
            key = tuple(ends)
            if key not in edges:
                edges[key] = (grid.instance['id'],points,coords)
                continue
            other, prior, material = edges[key]
            assert len(prior)==len(points), 'Nonconforming shared edge'
            error = max(math.dist(a,b) for a,b in zip(prior,points))
            assert error<.002, 'Crack in published shared edge'
            material_error = max((math.dist(a,b) for a,b in zip(material,coords)),default=0)
            assert len(material)==len(coords) and material_error<.002, 'Discontinuous material coordinates'
            reports.append(dict(instances=[other,grid.instance['id']],vertices=len(points),
                                max_position_error_meters=error,max_material_error=material_error))
    return reports


def build(args):
    source_bytes = args.base.read_bytes()
    source = json.loads(source_bytes)
    scene = copy.deepcopy(source)
    placement = json.loads(args.placement.read_text())
    source_camera = copy.deepcopy(source['camera'])
    views = placement.get('views', {})
    assert isinstance(views, dict), 'Named camera views required'
    views = dict(views)
    views.setdefault('scenic',views['panorama'] if placement.get('regions') else
                     dict(position=[100,220,800],target=[0,130,-150],fov_degrees=60))
    if args.initial_view:
        scene['camera'] = copy.deepcopy(views[args.initial_view])
    expected_camera = copy.deepcopy(scene['camera'])
    area = placement['bounds_xz']
    bounds = [*area['min'], *area['max']]
    assert bounds[0] < bounds[2] and bounds[1] < bounds[3]
    assets = {a['id']: a for a in source['meshes']['assets']}
    instances = {i['id']: i for i in source['meshes']['instances']}
    ids = placement['terrain_instances']
    assert ids and all(i in source['collision_meshes'] for i in ids)
    grids = [Grid(instances[i], assets[instances[i]['asset']]) for i in ids]
    seams = shared_edges(grids,assets)
    bins = defaultdict(list)
    for i, grid in enumerate(grids):
        x0, z0, x1, z1 = grid.bounds
        for x in range(math.floor(x0/256), math.floor(x1/256)+1):
            for z in range(math.floor(z0/256), math.floor(z1/256)+1):
                bins[x,z].append(i)
    water = Water(scene, set(placement['water_instances']), bounds)
    protected = [(e['transform']['position'][0], e['transform']['position'][2], 12)
                 for e in scene['entities'] if e.get('body') == 'player']
    protected.extend((p['center'][0], p['center'][1], p['radius_meters'])
                     for p in placement.get('clearings', []))
    rng = random.Random(args.seed)
    occupied, records, groups = defaultdict(list), [], [[], []]
    tree_geometry, source_height = tree_assets()
    assert not any(a['id'] in assets for a in tree_geometry)
    minimum_cos = math.cos(math.radians(args.max_slope))
    attempts = 0
    zones = placement.get('forest_zones', [])
    assert not zones or sum(zone['trees'] for zone in zones) == args.trees
    if zones:
        zones = copy.deepcopy(zones)
        total = 0
        for zone in zones:
            total += zone['trees']
            zone['end'] = total
    while len(records) < args.trees:
        attempts += 1
        if attempts > max(100000, args.trees*300):
            raise ValueError('Requested forest does not fit the placement constraints')
        zone = next((zone for zone in zones if len(records)<zone['end']),None)
        area = zone['bounds_xz'] if zone else area
        envelope = [*area['min'],*area['max']]
        x, z = round(rng.uniform(envelope[0], envelope[2]), 4), round(rng.uniform(envelope[1], envelope[3]), 4)
        if zone and 'exclude_bounds_xz' in zone:
            exclusion = zone['exclude_bounds_xz']
            if exclusion['min'][0]<=x<=exclusion['max'][0] and exclusion['min'][1]<=z<=exclusion['max'][1]:
                continue
        if rng.random() > .12+.88*grove(x,z,args.seed)**1.4:
            continue
        candidates = [grids[i].sample(x,z) for i in bins.get((math.floor(x/256), math.floor(z/256)), ())]
        candidates = [p for p in candidates if p is not None]
        if not candidates:
            continue
        y, cos_slope = max(candidates)
        if cos_slope < minimum_cos or water.excludes(x,y,z):
            continue
        if any((x-px)**2+(z-pz)**2 < radius**2 for px,pz,radius in protected):
            continue
        cell = math.floor(x/args.spacing), math.floor(z/args.spacing)
        if any((x-px)**2+(z-pz)**2 < args.spacing**2 for dx in [-1,0,1] for dz in [-1,0,1]
               for px,pz in occupied.get((cell[0]+dx,cell[1]+dz), ())):
            continue
        occupied[cell].append((x,z))
        height, yaw = rng.uniform(12,26), rng.random()*math.tau
        position = [x, round(y-.18,4), z]
        rotation = [0, math.sin(yaw/2), 0, math.cos(yaw/2)]
        scale = height/source_height
        color = [rng.uniform(.055,.12),rng.uniform(.14,.30),rng.uniform(.018,.065)]
        n = len(records)
        for part, asset, tint in [(0,'forest-bark',[.12,.055,.022]),(1,'forest-leaves',color)]:
            groups[part].append(dict(id=f'grove-{n:05d}-{part}',asset=asset,position=position,
                                     rotation=rotation,scale=scale,color=tint,playback=None))
        records.append(dict(position=position,surface_height=y,height_meters=height))
    scene['meshes'].setdefault('objects', []).extend(
        dict(id=f'grove-{n:05d}', members=[f'grove-{n:05d}-0', f'grove-{n:05d}-1'])
        for n in range(len(records)))
    scene['meshes']['assets'].extend(tree_geometry)
    for group in groups:
        scene['meshes']['instances'].extend(group)
    scene['version'] = SCENE_VERSION
    for key in ['entities','light','collision_meshes']:
        assert scene[key] == source[key], key
    assert scene['meshes']['assets'][:len(source['meshes']['assets'])] == source['meshes']['assets']
    assert scene['meshes']['instances'][:len(source['meshes']['instances'])] == source['meshes']['instances']
    args.output.mkdir(parents=True,exist_ok=True)
    path = args.output/'world.scene.json'
    path.write_text(json.dumps(scene,separators=(',',':')))
    assert args.base.read_bytes() == source_bytes
    report = dict(trees=len(records),components=len(records)*2,seed=args.seed,attempts=attempts,
                  minimum_spacing_meters=args.spacing,max_slope_degrees=args.max_slope,
                  source_sha256=hashlib.sha256(source_bytes).hexdigest(),surface_instances=ids,
                  source_preserved=True,tree_collision=False,records=records,
                  initial_view=args.initial_view,initial_camera=expected_camera,shared_edges=seams)
    (args.output/'forest-population.json').write_text(json.dumps(report,separators=(',',':')))
    if not args.prepare_only:
        c = Client(ROOT/'target/release/ignis', log_path=args.output/'forest-runtime.log')
        try:
            c.request('initialize',dict(protocolVersion='2025-11-25',capabilities={},clientInfo=dict(name='representative-forest',version='1')))
            c.request('notifications/initialized',notification=True)
            build_id = c.call('inspect')['build_id']
            relative = str(path.relative_to(ROOT))
            c.call('load_scene',path=relative)
            c.call('edit_scene',edits=[dict(op='compile_distant_meshes')])
            c.call('save_scene',path=relative)
            c.call('load_scene',path=relative)
            native = json.loads(path.read_text())
            for key in ['entities','light','collision_meshes']:
                assert native[key] == source[key], f'Native save changed {key}'
            assert all(abs(a-b)<.0001 for key in ['position','target']
                       for a,b in zip(native['camera'][key],expected_camera[key]))
            assert native['meshes']['assets'][:len(source['meshes']['assets'])] == source['meshes']['assets']
            assert native['meshes']['instances'][:len(source['meshes']['instances'])] == source['meshes']['instances']
            by_id = {i['id']:i for i in native['meshes']['instances']}
            root_error = 0
            for n in range(len(records)):
                x, y, z = by_id[f'grove-{n:05d}-0']['position']
                heights = [grids[i].sample(x,z) for i in bins.get((math.floor(x/256),math.floor(z/256)),())]
                ground = max(p[0] for p in heights if p is not None)
                root_error = max(root_error, abs(y+.18-ground))
            assert root_error < .002, f'Native roots diverged from source: {root_error}m'
            captures = [dict(name='walk',camera=source_camera)]+[
                dict(name=name,camera=camera) for name,camera in views.items() if name != 'play']
            images = []
            for view in captures:
                c.call('set_camera',camera=view['camera'])
                if args.player_view and view['name']=='walk':
                    player = next(e for e in source['entities'] if e.get('body')=='player')
                    camera = view['camera']
                    yaw = math.degrees(math.atan2(camera['target'][0]-camera['position'][0],
                                                   camera['position'][2]-camera['target'][2]))
                    config = placement.get('player_view') or dict(actor=player['id'],eye_height=min(1.65,player['transform']['size'][1]*.9),
                                                    yaw_degrees=yaw,pitch_degrees=0,fov_degrees=camera['fov_degrees'])
                    c.call('set_player_view',view=config)
                    c.call('step',ticks=40)
                meta = c.call('capture',path=str((args.output/(view['name']+'.png')).relative_to(ROOT)),width=1728,height=1117)
                images.append(dict(name=view['name'],metadata=meta))
            c.call('set_camera',camera=expected_camera)
            (args.output/'forest-verification.json').write_text(json.dumps(dict(build_id=build_id,captures=images,
                collision=c.call('inspect_mesh_collisions'),source_sha256=report['source_sha256'],
                native_source_preserved=True,max_root_surface_error_meters=root_error),indent=2))
        finally:
            c.close()
    print(json.dumps({k:v for k,v in report.items() if k != 'records'}))


if __name__ == '__main__':
    p = argparse.ArgumentParser()
    p.add_argument('--base',type=Path,default=ROOT/'artifacts/representative-world/terrain.scene.json')
    p.add_argument('--placement',type=Path,default=ROOT/'artifacts/representative-world/placement.json')
    p.add_argument('--output',type=Path,default=ROOT/'artifacts/representative-world')
    p.add_argument('--trees',type=int,default=12000)
    p.add_argument('--seed',type=int,default=184917)
    p.add_argument('--spacing',type=float,default=5.5)
    p.add_argument('--max-slope',type=float,default=42)
    p.add_argument('--prepare-only',action='store_true')
    p.add_argument('--initial-view',default='scenic',help='Named initial camera; empty preserves base camera')
    p.add_argument('--player-view',action='store_true',help='Capture walking view from physical player eyes')
    args = p.parse_args()
    args.base = args.base.resolve()
    args.placement = args.placement.resolve()
    args.output = args.output.resolve()
    if not args.output.is_relative_to(ROOT):
        p.error('Output must remain inside the project root')
    if args.trees < 1 or not math.isfinite(args.spacing) or args.spacing <= 0 or not 0 <= args.max_slope < 85:
        p.error('Population/spacing/slope invalid')
    build(args)
