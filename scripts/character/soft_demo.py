#!/usr/bin/env python3
"""Validate field continuity and native facial edits, without touching the editor."""
import json
import math
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT/'scripts'))
from verify_lab import Client
from soft_source import CHANNEL, topology
from spherical_demo import sphere, winding

OUT = ROOT/'artifacts/character-soft'
SOURCE = 'artifacts/character-base/human-soft.ignis-character'


def fields():
    source = json.loads((ROOT/SOURCE).read_text())
    parent = json.loads((ROOT/'artifacts/character-base/human-spherical.ignis-character').read_text())
    for key in ['positions', 'parts', 'joints', 'handles', 'warps', 'eyes', 'refinements']:
        assert source[key] == parent[key], key
    assert {p['name']: p for p in source['parameters'] if p['name'] != CHANNEL} == {p['name']: p for p in parent['parameters']}
    old = {t['name']: t for t in parent['targets']}
    for t in source['targets'][:len(parent['targets'])]:
        assert {**t, 'factors': t['factors'][:-1]} == old[t['name']] if t['name'].startswith('handle_') and t['factors'][-1]['parameter'] == CHANNEL else t == old[t['name']]
    ids, neighbors, rows = topology(source)
    local = {v: i for i, v in enumerate(ids)}
    targets = {t['name']: t for t in source['targets']}
    report = []
    for h in source['handles']:
        if h['part'] != 'body':
            continue
        values = []
        for axis, limit in enumerate(h['limits']):
            t = targets['soft_'+h['name']+'_'+str(axis)]
            assert all(all(v == 0 for i, v in enumerate(d['offset']) if i != axis) for d in t['deltas'])
            values.append({local[d['vertex']]: d['offset'][axis]/limit for d in t['deltas'] if d['vertex'] in local})
        assert values[0].keys() == values[1].keys() == values[2].keys()
        assert max(abs(values[0][v]-values[a][v]) for v in values[0] for a in [1, 2]) < 1e-6
        w = values[0]
        seeds = {local[v] for v in h['anchor']['vertices']}
        # Every authored support must be one connected skin patch.
        reached = set(seeds)
        pending = list(seeds)
        while pending:
            for j in neighbors[pending.pop()]:
                if j in w and j not in reached:
                    reached.add(j)
                    pending.append(j)
        assert reached == w.keys(), h['name']
        response = sum(sum(factor*w.get(v, 0.) for v, factor in rows[a].items()) for a in seeds)/len(seeds)
        assert abs(response-1) < 2e-6, (h['name'], response)
        adjacent = set().union(*(neighbors[a] for a in seeds))
        assert min(w.get(v, 0.) for v in adjacent) >= .849999
        original = {local[d['vertex']]: d['offset'][0]/h['limits'][0] for d in old['handle_'+h['name']+'_0']['deltas'] if d['vertex'] in local}
        def jump(weights):
            return max(abs(weights.get(v, 0.)-weights.get(j, 0.)) for v in weights for j in neighbors[v])
        assert jump(w) < jump(original)*.901, h['name']
        assert max(w.values()) < 1.12
        report.append(dict(name=h['name'], before=len(original), after=len(w),
                           old_edge_jump=jump(original), soft_edge_jump=jump(w), anchor_response=response))
    assert len(report) == 93
    return source, report


def main():
    source, field_report = fields()
    report = dict(fields=field_report, checks=['93 connected fields: adjacent ring follows at least85%, bounded peak, smaller edge jumps and calibrated1mm anchor response on all XYZ axes'], captures=[])
    if '--fields-only' in sys.argv:
        print(json.dumps(dict(passed=True, fields=len(field_report))))
        return
    c = Client(ROOT/'target/release/ignis', log_path=OUT/'runtime.log')
    call = c.call
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {}, 'clientInfo': {'name': 'soft-fields', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        call('load_scene', path='artifacts/character-soft/user-before.scene.json')
        call('open_character_panel', open=True, prefix='female')
        old = call('inspect_character_editor')
        assert old['ready']
        old_binding = old['binding']
        materials = {p: call('inspect_mesh', asset='female_'+p)['asset']['material'] for p in ['body', 'eye_R_iris']}
        def change(values):
            call('set_character_parameters', prefix='female', configuration={'parameters': values}, gesture=None)
        def state():
            return call('inspect_character_editor')
        def capture(label, oblique=False):
            target = [0., 1.57, -.10]
            call('set_camera', camera=dict(position=[.44 if oblique else 0., 1.59, -.95], target=target, fov_degrees=30.))
            call('capture', path='artifacts/character-soft/'+label+'.png', width=1100, height=1100)
            report['captures'].append(label)
        capture('before-front')
        capture('before-oblique', True)
        call('upgrade_character_source', prefix='female', path=SOURCE, reference=state()['reference'])
        upgraded = state()
        binding = upgraded['binding']
        params = binding['recipe']['configuration']['parameters']
        assert len(params) == 393 and len(upgraded['handles']) == 95 and params[CHANNEL] == 1
        assert all(params[k] == v for k, v in old_binding['recipe']['configuration']['parameters'].items())
        call('undo')
        assert state()['binding'] == old_binding
        call('redo')
        assert state()['binding'] == binding
        change({CHANNEL: 0})
        assert state()['binding']['bake_sha256'] == old_binding['bake_sha256']
        change({CHANNEL: 1})
        assert state()['binding'] == binding
        for p, m in materials.items():
            assert call('inspect_mesh', asset='female_'+p)['asset']['material'] == m
        report['checks'].append('all392 authored coefficients/materials retained, source upgrade undo/redo exact, softness0 exactly recovers parent bake')
        capture('after-front')
        capture('after-oblique', True)
        call('save_scene', path='artifacts/character-soft/accepted.scene.json')
        call('save_character_configuration', prefix='female', path='artifacts/character-soft/current.ignis-character-config')
        call('save_portable_save', path='artifacts/character-soft/body.ignis-portable')
        call('load_portable_save', path='artifacts/character-soft/body.ignis-portable', mode='recorded')
        call('open_character_panel', open=True, prefix='female')
        assert state()['binding'] == binding
        call('capture', path='artifacts/character-soft/reopened.png', width=1100, height=1100)
        assert (OUT/'reopened.png').read_bytes() == (OUT/'after-oblique.png').read_bytes()
        (OUT/'reopened.png').unlink()
        report['checks'].append('393 values/source retained in Scene/configuration/portable, reopened native pixels exact')
        for eye in source['eyes']:
            g = call('inspect_mesh', asset='female_'+eye['sclera'])['asset']['geometry']
            center, radius, error = sphere(g['positions'])
            assert error < 3e-7 and winding(g, center) > 0
        # Isolate displacement quality from the user's many combined edits.
        zero = {ch: 0. for h in source['handles'] for ch in h['parameters']}
        change(zero)
        zero_body = call('inspect_mesh', asset='female_body')['asset']['geometry']
        change({CHANNEL: 0})
        assert call('inspect_mesh', asset='female_body')['asset']['geometry'] == zero_body
        triangle_edges = {tuple(sorted((a, b))) for t in zero_body['triangles'] for a, b in zip(t, t[1:]+t[:1])}
        comparisons = []
        for name in ['jaw_back_R', 'chin_side_R', 'nose_tip_side_R', 'nostril_roof_R', 'malar_upper_R', 'cupid_valley', 'lower_lip_lobe_R', 'brow_arch_R']:
            item = dict(name=name)
            for amount in [0., 1.]:
                change({CHANNEL: amount})
                before = state()
                call('preview_character_handle', prefix='female', handle=name, offset=[0, 0, .003], symmetry=False, reference=before['reference'])
                assert state()['binding'] == before['binding']
                call('set_character_handle', prefix='female', handle=name, offset=[0, 0, .003], symmetry=False, reference=before['reference'])
                geometry = call('inspect_mesh', asset='female_body')['asset']['geometry']
                displacement = [[p[k]-q[k] for k in range(3)] for p, q in zip(geometry['positions'], zero_body['positions'])]
                jump = max(math.dist(displacement[a], displacement[b]) for a, b in triangle_edges)
                item['soft' if amount else 'local'] = jump
                call('set_character_handle', prefix='female', handle=name, offset=[0, 0, 0], symmetry=False, reference=state()['reference'])
            assert item['soft'] < item['local'], item
            comparisons.append(item)
        report['native_edge_comparisons'] = comparisons
        before = state()['binding']
        change_request = dict(prefix='female', configuration={'parameters': {CHANNEL: 2}}, gesture=None)
        call('set_character_parameters', expected_error=True, **change_request)
        assert state()['binding'] == before
        call('load_scene', path='artifacts/character-soft/accepted.scene.json')
        call('open_character_panel', open=True, prefix='female')
        assert state()['binding'] == binding
        report['checks'].append('8 native3mm edits reduce neighboring displacement jumps; zero offsets preserve parent shape; preview/bounds atomic; eyes remain spherical')
        report.update(passed=True, build=call('inspect')['build_id'], source_sha256=binding['recipe']['source_sha256'])
    except BaseException as e:
        report.update(passed=False, error=str(e))
        raise
    finally:
        (OUT/'verification.json').write_text(json.dumps(report, indent=2)+'\n')
        c.close()
    print(json.dumps({k: v for k, v in report.items() if k != 'fields'}, indent=2))


if __name__ == '__main__':
    main()
