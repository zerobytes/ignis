#!/usr/bin/env python3
"""Render the native junction probe against overlapping tubes, at fixed view/light."""
import json
import os
from pathlib import Path

PROJECT = Path(__file__).resolve().parents[1]
OUTPUT = PROJECT/'artifacts/organic-junction'
(OUTPUT/'artifacts').mkdir(parents=True, exist_ok=True)
(OUTPUT/'examples').mkdir(exist_ok=True)
os.environ['IGNIS_VERIFY_ROOT'] = str(OUTPUT)
from verify_lab import Client


def components(g):
    neighbors = [set() for _ in g['positions']]
    for a, b, c in g['triangles']:
        for i, j in [(a, b), (b, c), (c, a)]:
            neighbors[i].add(j); neighbors[j].add(i)
    unseen = set(range(len(neighbors))); count = 0
    while unseen:
        count += 1; pending = [unseen.pop()]
        while pending:
            for j in neighbors[pending.pop()]:
                if j in unseen:
                    unseen.remove(j); pending.append(j)
    return count


def run():
    probe = json.loads((PROJECT/'artifacts/organic-probe.json').read_text())
    assert components(probe['overlap']) == 3 and components(probe['connected']) == 1 and components(probe['fused']) == 1
    c = Client(PROJECT/'target/release/ignis', False)
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {}, 'clientInfo': {'name': 'ignis-organic-probe', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        c.call('generate', recipe={'seed': 85, 'width': 16, 'depth': 16, 'obstacles': 0})
        c.call('edit_scene', edits=[{'op': 'upsert', 'entity': {**e, 'visible': False}} for e in c.call('inspect')['scene']['entities'] if e['id'] != 'floor'])
        c.call('set_camera', camera={'position': [4.6, 3.7, 8.5], 'target': [0, 2.35, 0], 'fov_degrees': 36})
        c.call('set_light', light={'direction': [-0.45, -0.9, -0.65], 'ambient': 0.28})
        for id in ['overlap', 'connected', 'fused']:
            c.call('edit_mesh', edit={'kind': 'put_asset', 'asset': {'id': id, 'geometry': probe[id], 'shading': 'smooth'}})
        instance = {'id': 'fork', 'asset': 'overlap', 'position': [0, 2.3, 0], 'rotation': [0, 0, 0, 1], 'scale': 1, 'color': [0.2, 0.115, 0.055], 'playback': None}
        c.call('edit_mesh', edit={'kind': 'put_instance', 'instance': instance})
        c.call('capture', path='artifacts/before.png', width=1000, height=1000)
        instance['asset'] = 'connected'
        c.call('edit_mesh', edit={'kind': 'put_instance', 'instance': instance})
        c.call('capture', path='artifacts/topology-only.png', width=1000, height=1000)
        instance['asset'] = 'fused'
        c.call('edit_mesh', edit={'kind': 'put_instance', 'instance': instance})
        topology = c.call('inspect_mesh_topology', asset='fused')['topology']
        assert topology['boundary_edges'] == 0 and topology['non_manifold_edges'] == 0 and not topology['isolated_vertices']
        c.call('capture', path='artifacts/after.png', width=1000, height=1000)
        c.call('set_camera', camera={'position': [-4.6, 3.7, -8.5], 'target': [0, 2.35, 0], 'fov_degrees': 36})
        c.call('capture', path='artifacts/back.png', width=1000, height=1000)
        c.call('set_camera', camera={'position': [4.6, 3.7, 8.5], 'target': [0, 2.35, 0], 'fov_degrees': 36})
        c.call('save_scene', path='examples/organic-junction-probe.scene.json')
        # Camera edits are separate authoring transactions; explicitly switch
        # the mesh so undo/redo checks the same completed construction.
        instance['asset'] = 'connected'
        c.call('edit_mesh', edit={'kind': 'put_instance', 'instance': instance})
        instance['asset'] = 'fused'
        c.call('edit_mesh', edit={'kind': 'put_instance', 'instance': instance})
        c.call('undo'); assert c.call('inspect_meshes')['instances'][0]['asset'] == 'connected'
        c.call('redo'); assert c.call('inspect_meshes')['instances'][0]['asset'] == 'fused'
        c.call('save_replay', path='artifacts/organic.replay.json')
        assert c.call('run_replay', path='artifacts/organic.replay.json')['meshes_matched']
        c.call('capture_replay', replay_path='artifacts/organic.replay.json', path='artifacts/replay.png', width=1000, height=1000)
        assert (OUTPUT/'artifacts/after.png').read_bytes() == (OUTPUT/'artifacts/replay.png').read_bytes()
        assert (OUTPUT/'artifacts/before.png').read_bytes() != (OUTPUT/'artifacts/after.png').read_bytes()
        report = {'passed': True, 'checks': 3, 'build_id': c.call('inspect')['build_id'], 'junction': probe['junction'],
                  'connected_components': 1, 'control_components': 3, 'fused_radial_degree': probe['fused_radial_degree'], 'native_window': False}
        (OUTPUT/'artifacts/report.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps(report))
    finally:
        c.close()


if __name__ == '__main__':
    run()
