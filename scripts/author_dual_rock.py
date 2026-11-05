#!/usr/bin/env python3
"""Author a local Dual Contouring rock study via MCP. No mesh math in Python."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT
from open_earth_walk import init

PREFIX = 'artifacts/dual-rock'

def author(recipe_path):
    recipe = json.loads(recipe_path.read_text())
    out = ROOT / PREFIX
    out.mkdir(parents=True, exist_ok=True)
    client = Client(ROOT / 'target/release/ignis', log_path=out / 'author.log')
    try:
        init(client)
        client.call('generate', recipe=dict(width=60, depth=48, seed=221, obstacles=0))
        scene = client.call('inspect')['scene']
        player = copy.deepcopy(next(e for e in scene['entities'] if e['id'] == 'player'))
        player['transform'] = dict(position=[0, .85, -8], size=[.7, 1.6, .7])
        player['footsteps'] = None
        floor = copy.deepcopy(next(e for e in scene['entities'] if e['id'] == 'floor'))
        floor['color'] = [.24, .29, .22]
        edits = [dict(op='remove', id=e['id']) for e in scene['entities'] if e['id'] not in ('floor', 'player')]
        edits += [dict(op='upsert', entity=e) for e in (player, floor)]
        client.call('edit_scene', edits=edits)
        material = json.loads((ROOT / 'assets/materials/rock/fractured.ignis-material').read_text())['material']
        material['procedural']['recipe']['tile_size'] = [3, 3]
        result = client.call('publish_fractured_rock', publication=dict(settings=recipe['settings'], asset='dual_rock', instance='dual_rock', position=[0, 0, 0], color=[1, 1, 1], material=material, collision=True))
        print(json.dumps(result), flush=True)
        (out / 'generation.json').write_text(json.dumps(result, indent=2) + '\n')
        client.call('set_light', light=dict(direction=[.6, -.75, .4], ambient=.2, direct_intensity=.9, direct_color=[1, .96, .9], shadow=dict(center=[0, 6, 0], extent=22, resolution=2048, bias=.0003, receiver_plane_bias=True)))
        for name, camera in recipe['cameras'].items():
            client.call('set_camera', camera=camera)
            client.call('capture', path=f'{PREFIX}/{name}.png', width=1600, height=1000)
        client.call('edit_scene', edits=[dict(op='mesh', edit=dict(kind='set_material', asset='dual_rock', material={}))])
        neutral_instance=dict(id='dual_rock', asset='dual_rock', position=[0, 0, 0], rotation=[0, 0, 0, 1], scale=1, color=[.45, .48, .5])
        client.call('edit_scene', edits=[dict(op='mesh', edit=dict(kind='put_instance', instance=neutral_instance))])
        client.call('set_camera', camera=recipe['cameras']['close'])
        client.call('capture', path=f'{PREFIX}/neutral.png', width=1600, height=1000)
        client.call('edit_scene', edits=[dict(op='mesh', edit=dict(kind='set_material', asset='dual_rock', material=material))])
        neutral_instance['color']=[1, 1, 1]
        client.call('edit_scene', edits=[dict(op='mesh', edit=dict(kind='put_instance', instance=neutral_instance))])
        client.call('set_camera', camera=recipe['cameras']['overview'])
        client.call('save_scene', path=f'{PREFIX}/world.scene.json')
        print(f'Authored {PREFIX}/world.scene.json', flush=True)
    finally:
        client.close()

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--recipe', type=Path, default=ROOT / 'examples/dual-rock.recipe.json')
    author(parser.parse_args().recipe)
