#!/usr/bin/env python3
"""Check the actual scene/field bundle, sampled terrain contact and native walk."""
import hashlib
import json
import math
import struct

from build_emerald_meadow import OUT, grid, hash32
from groundcover_assets import atomic_json


def main():
    bundle = json.loads((OUT / 'bundle.json').read_text())
    for key in ['scene', 'field']:
        assert hashlib.sha256((OUT / bundle[key]).read_bytes()).hexdigest() == bundle[key + '_sha256']
    scene = json.loads((OUT / bundle['scene']).read_text())
    surface = grid(scene)
    assert surface.bounds == [-500., -500., 500., 500.]
    report = json.loads((OUT / 'compile.json').read_text())
    assert report['exclusions'] == [] and len(report['coverage_50m_cells']) == 400
    assert all(c == 160000 for c in report['coverage_50m_cells'])
    effective_radius = 0.
    triangles = []
    with (OUT / bundle['field']).open('rb') as f:
        assert f.read(8) == b'IGFLD1\0\0'
        ox, oz, spacing, _, _, _, nx, nz = struct.unpack('<6f2I', f.read(32))
        assert (ox, oz, nx, nz, spacing) == (-500., -500., 500, 500, .125)
        assert nx * 16 * spacing == 1000 and nz * 16 * spacing == 1000
        for _ in range(6):
            stride, count = struct.unpack('<2I', f.read(8))
            assert count > 0 and count % 3 == 0
            triangles.append(count // 3)
            values = struct.unpack('<' + 'f' * count * 8, f.read(count * 32))
            assert all(math.isfinite(v) for v in values)
            for i in range(0, len(values), 8):
                effective_radius = max(effective_radius, math.hypot(values[i], values[i+2]) * stride)
        assert effective_radius < .25, 'Distant templates inflate near block bounds'
        offset = f.tell()
        assert (OUT / bundle['field']).stat().st_size - offset == nx * nz * 256 * 4
        payload = hashlib.sha256()
        while block := f.read(4 * 1024 * 1024):
            payload.update(block)
        errors = []
        for i in range(4096):
            index = hash32(i + 218) % (nx * nz * 256)
            tile, local = divmod(index, 256)
            tx, tz = tile % nx, tile // nx
            lx, lz = local % 16, local // 16
            h = hash32(index)
            jx, jz = (h & 65535) / 65535 - .5, (h >> 16) / 65535 - .5
            x = ox + (tx * 16 + lx + .5 + jx * .9) * spacing
            z = oz + (tz * 16 + lz + .5 + jz * .9) * spacing
            f.seek(offset + index * 4)
            y, = struct.unpack('<f', f.read(4))
            # Independent Grid interpolation of the persisted source triangles.
            errors.append(abs(y - (surface.sample(x, z)[0] - .004)))
        assert max(errors) < .00002
    assert triangles == report['triangle_templates']
    walk = json.loads((OUT / 'native-validation.json').read_text())
    assert walk['walk_meters'] > 3 and walk['controllers']['controllers']['player']['grounded']
    assert walk['save_load_png_exact_in_bound_session']
    atomic_json(OUT / 'verification.json', dict(bounds=surface.bounds, area_m2=1_000_000,
        occupied_50m_cells=400, all_cells_full=True, sample_count=len(errors),
        maximum_root_datum_error_m=max(errors), effective_template_radius_m=effective_radius,
        triangles_per_level=triangles, height_payload_sha256=payload.hexdigest(),
        walk_meters=walk['walk_meters'], native_build_id=walk['build_id'],
        save_load_png_exact_in_bound_session=True,
        individual_blade_terrain_warp=False, artistic_acceptance=False))
    print('Bundle, whole-area planting,4096 root datums, template bounds and native walk verified')


if __name__ == '__main__':
    main()
