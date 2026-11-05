"""Check experiment integrity; visual failure is a measured result, not a harness failure."""
import hashlib
import itertools
import json
import math
import struct
import zlib
from pathlib import Path

ROOT = Path('artifacts/appearance-probe/rock-symbol')
SIZES = [[3456, 2234], [1728, 1117]]
DISTANCES = [4, 24, 96]

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
        if tag == b'IDAT': compressed.extend(data)
        off += n + 12
        if tag == b'IEND': ended = True; break
    assert ended and off == len(b) and b[24] == 8
    assert len(zlib.decompress(compressed)) == (size[0] * {2: 3, 6: 4}[b[25]] + 1) * size[1]
    return hashlib.sha256(b).hexdigest()

manifest = read('input-manifest')
for path, digest in manifest.items():
    assert hashlib.sha256(Path(path).read_bytes()).hexdigest() == digest, path
build = read('build')
assert build['source_triangles'] == 81920 and build['source_vertices'] == 40962
assert build['source_bytes'] == 40962*40 + 81920*12
assert build['symbol_stride'] == 48
for level in build['levels']:
    raw = (ROOT / f"grid{level['grid']}.symbols").read_bytes()
    assert len(raw) == level['bytes'] == level['symbols']*48
    for s in struct.iter_unpack('<12f', raw):
        assert all(math.isfinite(x) for x in s) and s[3] > 0 and s[7] > 0 and s[11] > 0
        assert abs(sum(x*x for x in s[4:7])-1) < 1e-5
        assert all(0 <= x <= 1 for x in s[8:11])
rows = read('quality')
assert len(rows) == len({r['id'] for r in rows}) == 162
assert {(r['resolution'], r['distance_id'], r['scene'], r['frame']) for r in rows} == set(itertools.product(range(2), range(3), range(3), range(9)))
failures = [0]*6
worst = [{'rmse': 0, 'p99': 0, 'coverage_fraction': 0} for _ in range(6)]
images = 0
closure = []
hashes = {}
for r in rows:
    res, d, scene, frame = [r[k] for k in ['resolution', 'distance_id', 'scene', 'frame']]
    assert r['id'] == f'r{res}_d{d}_s{scene}_{frame}' and r['size'] == SIZES[res]
    assert r['pose']['distance'] == DISTANCES[d] and len(r['checks']) == 6
    for m, q in enumerate(r['checks']):
        for k in ['rmse', 'interior_rmse', 'p99', 'max_rgb', 'coverage_fraction']:
            assert math.isfinite(q[k]) and q[k] >= 0
        gate = q['rmse'] <= .005 and q['p99'] <= .02 and q['coverage_fraction'] <= .0005
        assert gate == q['pass']
        assert q['reference_covered_pixels'] > 0 and q['union_pixels'] >= q['reference_covered_pixels']
        assert q['coverage_fraction'] == q['coverage_mismatches']/q['union_pixels']
        assert (q['temporal_residual'] is None) == (frame == 0)
        if frame: assert math.isfinite(q['temporal_residual']) and q['temporal_residual'] >= 0
        failures[m] += int(not gate)
        for k in worst[m]: worst[m][k] = max(worst[m][k], q[k])
    for m in range(7):
        hashes[res, d, scene, frame, m] = png(ROOT / f"{r['id']}_{m}.png", r['size']); images += 1
for res, d, scene, m in itertools.product(range(2), range(3), range(3), range(7)):
    # Combined elevation uses f32 sin(2pi), so diagnose rather than assume bit-exact closure.
    if hashes[res,d,scene,0,m] != hashes[res,d,scene,8,m]: closure.append([res,d,scene,m])
checks = read('checks')
assert len(checks) == 7 and all(c['repeat_wrap_light_return'] for c in checks)
integrated = read('integrated')
assert len(integrated) == 6
for r in integrated:
    assert r['reference_spp'] == 64 and len(r['errors_1spp']) == 7
    png(ROOT / f"r{r['resolution']}_d{r['distance_id']}_integrated.png", SIZES[r['resolution']]); images += 1
    for q in r['errors_1spp'] + [r['convergence_16_64']]:
        assert all(math.isfinite(q[k]) and q[k] >= 0 for k in ['rmse_surface', 'alpha_rmse', 'max_abs'])
cohorts = {}
for name, count in [('timings',18), ('repeat',6)]:
    cs = read(name); assert len(cs) == count
    assert len({(c['resolution'],c['distance_id'],c['frame']) for c in cs}) == count
    ratios = []
    for c in cs:
        assert c['scene'] == 2 and c['size'] == SIZES[c['resolution']]
        assert c['pose']['distance'] == DISTANCES[c['distance_id']]
        samples = {(s['method'],s['control']): s for s in c['samples']}
        assert len(samples) == len(c['samples']) == 14
        for m, control in itertools.product(range(7),range(2)):
            s = samples[m,control]; assert s['frames'] == 32
            for key in ['total','cpu','gpu']:
                v = s[f'{key}_samples_ms']
                assert len(v) == 7 and all(math.isfinite(x) and x > 0 for x in v)
                assert s[f'{key}_ms']['median'] == sorted(v)[3]
            if control: ratios.append(s['total_ms']['median']/samples[m,0]['total_ms']['median'])
    cohorts[name] = {'count':count,'duplicate_total_ratio_range':[min(ratios),max(ratios)]}
result = {'poses':len(rows),'comparisons':len(rows)*6,'pngs':images,'failures_per_method':failures,'maxima_per_method':worst,'cohorts':cohorts,'repeat_wrap_light_return':True,'sequence_png_closure_differences':closure,'manifest_verified':True,'native_browser':False,'continuous_motion_tested':False}
(ROOT/'verification.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result,indent=2))
