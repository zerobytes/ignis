"""Validate fixed-content physical-density captures and raw timing cohorts."""
import hashlib
import itertools
import json
import math
import struct
import zlib
from pathlib import Path

ROOT = Path('artifacts/appearance-probe/pixel-density')
SIZES = [[3456, 2234], [1728, 1117]]
TARGETS = [1, 2, 10, 100]
DISTANCES = [12, 14, 16, 14, 12]


def read(name):
    return json.loads((ROOT / f'{name}.json').read_text())


def png(path, size):
    b = path.read_bytes()
    assert b[:8] == b'\x89PNG\r\n\x1a\n'
    assert list(struct.unpack('>II', b[16:24])) == size
    off, compressed, ended = 8, bytearray(), False
    while off < len(b):
        n = struct.unpack('>I', b[off:off + 4])[0]
        tag, data = b[off + 4:off + 8], b[off + 8:off + 8 + n]
        assert zlib.crc32(tag + data) == struct.unpack('>I', b[off + 8 + n:off + 12 + n])[0]
        if tag == b'IDAT':
            compressed.extend(data)
        off += n + 12
        if tag == b'IEND':
            ended = True
            break
    assert ended and off == len(b) and b[24] == 8
    assert len(zlib.decompress(compressed)) == (size[0] * {2: 3, 6: 4}[b[25]] + 1) * size[1]
    return hashlib.sha256(b).hexdigest()


manifest = read('input-manifest')
for file, expected in manifest.items():
    assert hashlib.sha256(Path(file).read_bytes()).hexdigest() == expected, file
preparations = read('preparation')
assert len(preparations) == 8
prep = {(p['resolution'], p['density']): p for p in preparations}
assert len(prep) == 8
rows = read('quality')
assert len(rows) == len({r['id'] for r in rows}) == 40
hashes = {}
for r in rows:
    res, d, frame = r['resolution'], r['density'], r['frame']
    assert r['id'] == f'r{res}_d{d}_{frame}' and r['distance'] == DISTANCES[frame]
    assert r['size'] == SIZES[res] and r['target_at14'] == TARGETS[d]
    p = prep[res, d]['metadata']
    assert p['target_triangles_per_pixel'] == TARGETS[d]
    assert abs(p['triangles'] - TARGETS[d] * p['covered_pixels_at14']) <= 1
    assert p['triangles'] == p['base_grid_triangles'] + 2 * p['interior_splits']
    assert p['base_grid_triangles'] == p['parent_triangles'] * p['uniform_edge_subdivisions'] ** 2
    assert p['triangles'] == r['source_triangles'] and r['selected_triangles'] == p['parent_triangles'] == 327680
    assert r['source_triangles_per_pixel'] == p['triangles'] / r['covered_pixels']
    if frame in [1, 3]:
        assert r['covered_pixels'] == p['covered_pixels_at14']
        assert r['parent_bound_px'] == prep[res, d]['bound_px_at14']
    assert r['exact_parent_image'] and r['certified'] and r['invalid_certificate_fallback_checked']
    assert 0 <= r['parent_bound_px'] <= .15
    assert len(r['checks']) == 2
    for q in r['checks']:
        assert all(math.isfinite(q[k]) and q[k] >= 0 for k in ['foreground_rmse', 'p99_rgb', 'max_rgb', 'coverage_fraction'])
        assert q['quality_gate'] and q['foreground_rmse'] <= .005 and q['p99_rgb'] <= .02 and q['coverage_fraction'] <= .0005
        assert q['temporal_residual_rmse'] is None  # Discrete poses, no continuous temporal metric.
    assert r['checks'][1]['max_rgb'] == r['checks'][1]['changed_pixels'] == 0
    for m in range(3):
        hashes[res, d, frame, m] = png(ROOT / f"{r['id']}_{m}.png", r['size'])
for res, d, frame in itertools.product(range(2), range(4), range(5)):
    assert hashes[res, d, frame, 2] == hashes[res, d, frame, 0] == hashes[res, 0, frame, 0]
    if frame >= 3:
        for m in range(3):
            assert hashes[res, d, frame, m] == hashes[res, d, 4-frame, m]
cohorts = {}
for name in ['validation', 'repeat']:
    cs = read(name)
    assert len(cs) == 2 and {c['resolution'] for c in cs} == {0, 1}
    ratios = []
    for c in cs:
        assert c['distance'] == 14 and c['size'] == SIZES[c['resolution']]
        ss = {(s['density'], s['method'], s['control']): s for s in c['samples']}
        assert len(ss) == len(c['samples']) == 24
        for d, m, control in itertools.product(range(4), range(3), range(2)):
            s = ss[d, m, control]
            expected = max(2, min(128, 32000000 // prep[c['resolution'], d]['metadata']['triangles'])) if m == 1 else 128
            assert s['frames'] == expected and s['gpu_frames'] == min(32, expected)
            for metric in ['total', 'gpu', 'cpu']:
                values = s[f'{metric}_samples_ms']
                assert len(values) == 8 and all(math.isfinite(x) and x > 0 for x in values)
                assert s[f'{metric}_ms']['samples'] == 8
                assert s[f'{metric}_ms']['median'] == sorted(values)[len(values)//2]
            if control == 1:
                ratios.append(s['total_ms']['median'] / ss[d, m, 0]['total_ms']['median'])
    cohorts[name] = {'duplicate_total_ratio_range': [min(ratios), max(ratios)], 'cohorts': len(cs)}

# Conservative box visibility check, independently reconstructed in double precision.
# This says the box lies in the frustum, not that every triangle wins a depth sample.
a = json.loads(Path('artifacts/appearance-probe/hybrid-scaling/f1_d2.json').read_text())
def dot(a, b):
    return sum(x*y for x, y in zip(a, b))
def norm(a):
    return [x/math.sqrt(dot(a, a)) for x in a]
z = norm([.15, .66, .736])
x = norm([z[2], 0, -z[0]])
y = [z[1]*x[2]-z[2]*x[1], z[2]*x[0]-z[0]*x[2], z[0]*x[1]-z[1]*x[0]]
frusta = []
for res, size in enumerate(SIZES):
    e = max(p['metadata']['guarded_error_world'] for p in preparations if p['resolution'] == res)
    corners = list(itertools.product(*[(a['bounds'][0][i]-e, a['bounds'][1][i]+e) for i in range(3)]))
    depths = [14-dot(p, z) for p in corners]
    ndcx = [abs(dot(p, x)/(depth*math.tan(math.pi/6)*size[0]/size[1])) for p, depth in zip(corners, depths)]
    ndcy = [abs(dot(p, y)/(depth*math.tan(math.pi/6))) for p, depth in zip(corners, depths)]
    assert min(depths) > .1 and max(ndcx) < 1 and max(ndcy) < 1
    frusta.append({'resolution': res, 'distance': 14, 'min_depth': min(depths), 'max_abs_ndc': [max(ndcx), max(ndcy)], 'fully_inside': True, 'occlusion_tested': False})
qs = [r['checks'][0] for r in rows]
result = {'poses': len(rows), 'comparisons': len(rows)*2, 'pngs': len(hashes), 'quality_failures': 0,
          'certified_parent_exact_rgba32f': True, 'same_reference_across_densities': True, 'return_png_exact': True,
          'full_max_rmse': max(q['foreground_rmse'] for q in qs), 'full_max_p99': max(q['p99_rgb'] for q in qs),
          'full_max_rgb': max(q['max_rgb'] for q in qs), 'full_max_coverage_fraction': max(q['coverage_fraction'] for q in qs),
          'max_bound_px': max(r['parent_bound_px'] for r in rows), 'frusta_at14': frusta, 'cohorts': cohorts,
          'native_browser': False, 'continuous_motion_tested': False, 'manifest_verified': True}
(ROOT / 'verification.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps(result, indent=2))
