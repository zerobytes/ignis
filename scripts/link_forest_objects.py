#!/usr/bin/env python3
"""Upgrade the known forest build recipe with explicit visual object membership.

This is a demo migration, never a renderer rule. Runtime accepts any authored
membership and does not infer objects from names, species or proximity.
"""
import argparse
import hashlib
import json
from pathlib import Path
from verify_lab import SCENE_VERSION


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--scene', type=Path, default=Path('artifacts/expanded-world/world.scene.json'))
    p.add_argument('--population', type=Path, default=Path('artifacts/expanded-world/forest-population.json'))
    p.add_argument('--output', type=Path, default=Path('artifacts/object-culling/grouped.scene.json'))
    a = p.parse_args()
    original = a.scene.read_bytes()
    scene = json.loads(original)
    population = json.loads(a.population.read_bytes())
    by_id = {i['id']: i for i in scene['meshes']['instances']}
    assert len(by_id) == len(scene['meshes']['instances'])
    assert population['trees'] == len(population['records'])
    objects = []
    for n, record in enumerate(population['records']):
        members = [f'grove-{n:05d}-0', f'grove-{n:05d}-1']
        for member in members:
            # Published f32 values may differ from the original Python f64 recipe.
            assert max(abs(x-y) for x,y in zip(by_id[member]['position'],record['position'])) < .002
        assert all(by_id[members[0]][field] == by_id[members[1]][field]
                   for field in ['position', 'rotation', 'scale'])
        objects.append(dict(id=f'grove-{n:05d}', members=members))
    old = scene['meshes'].get('objects', [])
    assert not old or old == objects, 'Refuse to overwrite other authored membership'
    scene['meshes']['objects'] = objects
    scene['version'] = SCENE_VERSION
    a.output.parent.mkdir(parents=True, exist_ok=True)
    candidate = a.output.with_suffix(a.output.suffix + '.candidate')
    encoded = json.dumps(scene, separators=(',', ':')).encode()
    candidate.write_bytes(encoded)
    candidate.replace(a.output)
    report = dict(source=str(a.scene), output=str(a.output),
                  source_sha256=hashlib.sha256(original).hexdigest(),
                  output_sha256=hashlib.sha256(encoded).hexdigest(),
                  objects=len(objects), grouped_instances=2*len(objects),
                  geometry_and_transforms_unchanged=True, physics_unchanged=True,
                  bytes_before=len(original), bytes_after=len(encoded))
    (a.output.parent/'source.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps(report))


if __name__ == '__main__':
    main()
