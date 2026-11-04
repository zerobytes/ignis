#!/usr/bin/env python3
"""Exercise the native living-plant dispatcher, persistence, and actual render.

No Python growth grammar: every branch/leaf/flower/fruit comes from Rust commands.
Own MCP process only; stable outputs replace the previous local review.
"""
import argparse
import copy
import hashlib
import json
import time
from pathlib import Path

from demo_groundcover import fit, relative, setup
from groundcover_assets import ROOT, atomic_json
from verify_lab import Client

OUT = ROOT / 'artifacts/living-plants'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, default=ROOT / 'target/release/ignis')
    args = parser.parse_args()
    OUT.mkdir(parents=True, exist_ok=True)
    definition = json.loads((ROOT / 'examples/living-plant.recipe.json').read_text())
    c = Client(args.binary, log_path=OUT / 'runtime.log')
    report = dict(stages={}, persistence={}, experimental=True)

    def botany(operation):
        return c.call('botany', operation=operation)

    def action(kind, **fields):
        return botany(dict(kind='living', prefix='living', action=dict(kind=kind, **fields)))

    def capture(name):
        c.call('capture', path=relative(OUT / f'{name}.png'), width=1400, height=1100)

    def equal_capture(name, reference):
        capture(name)
        assert (OUT / f'{name}.png').read_bytes() == (OUT / f'{reference}.png').read_bytes()

    try:
        c.request('initialize', dict(protocolVersion='2025-11-25', capabilities={}, clientInfo=dict(name='living-plant-review', version='1')))
        c.request('notifications/initialized', notification=True)
        schema = c.request('tools/list')['result']['tools'][0]['inputSchema']
        assert all(token in json.dumps(schema) for token in ['create_living', 'living', 'harvest'])
        atomic_json(OUT / 'schema.json', schema)
        report['build_id'] = setup(c)
        report['binary_sha256'] = hashlib.sha256(args.binary.read_bytes()).hexdigest()
        ground = dict(id='living_ground', geometry=dict(positions=[[-20, -.025, -20], [20, -.025, -20], [-20, -.025, 20], [20, -.025, 20]], triangles=[[0, 2, 1], [1, 2, 3]]))
        instance = dict(id='living_ground', asset='living_ground', position=[0, 0, 0], rotation=[0, 0, 0, 1], scale=1, color=[.22, .27, .14])
        c.call('edit_scene', edits=[dict(op='mesh', edit=dict(kind='put_asset', asset=ground)), dict(op='mesh', edit=dict(kind='put_instance', instance=instance))])
        c.call('set_light', light=dict(direction=[.5, -.82, -.35], ambient=.3, direct_intensity=1.1,
            environment=dict(sky=[.46, .61, .78], horizon=[.59, .65, .62], ground=[.16, .19, .08], axis=[0, 1, 0], intensity=.35),
            shadow=dict(center=[0, 2, 0], extent=7, resolution=2048, bias=.0005)))
        c.call('set_camera', camera=dict(position=[1.6, 2.5, 6.9], target=[0, 1.95, 0], fov_degrees=39))
        c.call('save_scene', path=relative(OUT / 'empty.scene.json'))
        botany(dict(kind='create_living', definition=definition, born=0))
        botany(dict(kind='save', prefix='living', path='assets/botany/living-specimen.ignis-botany'))
        for day, name in [(0, 'seedling'), (10, 'first-branch'), (40, 'growing'), (80, 'flowers'), (86, 'green-fruit'), (90, 'ripe-fruit')]:
            started = time.monotonic()
            result = action('advance', time=day)
            report['stages'][name] = dict(day=day, command_seconds=time.monotonic() - started, compiled=result['compiled'])
            capture(name)
        fruit = dict(site='node0', cohort=0, slot=0)
        action('harvest', fruit=fruit)
        action('prune', site='node1')
        capture('intervened')
        before = action('inspect')['plant']
        revision = c.call('inspect')['revision']
        for bad in [dict(kind='advance', time=10), dict(kind='harvest', fruit=fruit)]:
            c.call('botany', expected_error=True, operation=dict(kind='living', prefix='living', action=bad))
            assert c.call('inspect')['revision'] == revision
            assert action('inspect')['plant'] == before
        c.call('save_scene', path=relative(OUT / 'saved.scene.json'))
        action('advance', time=100)
        c.call('load_scene', path=relative(OUT / 'saved.scene.json'))
        assert action('inspect')['plant'] == before
        equal_capture('scene-roundtrip', 'intervened')
        report['persistence']['scene_pixels_equal'] = True
        c.call('save_portable_save', path=relative(OUT / 'saved.ignis-portable'))
        action('advance', time=180)
        c.call('load_portable_save', path=relative(OUT / 'saved.ignis-portable'))
        assert action('inspect')['plant'] == before
        equal_capture('portable-roundtrip', 'intervened')
        report['persistence']['portable_pixels_equal'] = True
        botany(dict(kind='save', prefix='living', path=relative(OUT / 'saved.ignis-botany')))
        action('advance', time=180)
        capture('second-flowers')
        action('advance', time=190)
        capture('second-fruit')
        production = action('inspect')['plant']['production']
        assert fruit in production['harvested'] and 'node1' in production['removed_sites']
        attached = action('inspect')['attached']
        assert len(attached) == 7 and all(identity['cohort'] == 1 for identity, _ in attached)
        report['second_cohort'] = dict(attached=attached, harvested=production['harvested'], removed_sites=production['removed_sites'])
        c.call('save_replay', path=relative(OUT / 'growth.replay.json'))
        assert c.call('run_replay', path=relative(OUT / 'growth.replay.json'))['matched']
        report['persistence']['replay_matched'] = True
        botany(dict(kind='open', open=True, prefix='living'))
        c.call('capture_editor', path=relative(OUT / 'editor.png'), width=1600, height=1100)
        c.call('save_scene', path=relative(OUT / 'living.scene.json'))
        botany(dict(kind='load', path=relative(OUT / 'saved.ignis-botany'), publish=True))
        assert action('inspect')['plant'] == before
        equal_capture('source-roundtrip', 'intervened')
        report['persistence']['source_pixels_equal'] = True

        # A native three-specimen storyboard; same definition/seed, different times.
        c.call('load_scene', path=relative(OUT / 'empty.scene.json'))
        for index, (prefix, day) in enumerate([('young', 40), ('flowering', 80), ('fruiting', 90)]):
            d = copy.deepcopy(definition)
            d['prefix'] = prefix
            botany(dict(kind='create_living', definition=d, born=0))
            botany(dict(kind='living', prefix=prefix, action=dict(kind='advance', time=day)))
            r = next(r for r in botany(dict(kind='inspect'))['recipes'] if r['prefix'] == prefix)
            r['settings']['position'] = [(index - 1) * 2.2, 0, 0]
            botany(dict(kind='publish', recipe=r))
        fit(c, [-3.1, 0, -1], [3.1, 4.25, 1], OUT / 'gallery.png', direction=(.04, .12, 1), width=1920, height=1200)
        c.call('save_scene', path=relative(OUT / 'gallery.scene.json'))
        c.call('load_scene', path=relative(OUT / 'gallery.scene.json'))
        c.call('capture', path=relative(OUT / 'gallery-roundtrip.png'), width=1920, height=1200)
        assert (OUT / 'gallery.png').read_bytes() == (OUT / 'gallery-roundtrip.png').read_bytes()
        report['persistence']['gallery_pixels_equal'] = True
        report['versions'] = dict(scene=c.call('inspect')['scene']['version'], source=8, replay=131)
        atomic_json(OUT / 'verification.json', report)
        names = ['gallery', 'seedling', 'first-branch', 'growing', 'flowers', 'green-fruit', 'ripe-fruit', 'intervened', 'second-flowers', 'second-fruit', 'editor']
        (OUT / 'review.html').write_text('<!doctype html><meta charset="utf-8"><title>Living plant review</title><style>body{background:#20242a;color:#eee;font:16px system-ui;max-width:1400px;margin:auto}img{max-width:100%}</style><h1>Native living-plant prototype</h1><p>Explicit editor time advance. Growth, reproduction and persisted interventions; no automatic gameplay clock or physical fruit drops.</p>' + ''.join(f'<h2>{name}</h2><img src="{name}.png">' for name in names))
        print(json.dumps(report, indent=2))
    finally:
        c.close()


if __name__ == '__main__':
    main()
