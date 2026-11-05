"""Verify exact-pixel gates, moving controls, and every saved PNG payload."""
import json
import math
import struct
import zlib
from pathlib import Path

ROOT = Path('artifacts/appearance-probe/gpu-visibility')
d = json.loads((ROOT/'report.json').read_text())
edges = json.loads((ROOT/'edge-checks.json').read_text())
assert len(d['rows']) == 156 and len(d['modes']) == 6
assert d['appearance_summaries'] is False
assert d['visibility_readback_in_render_loop'] is False
assert d['region_count'] == 479
pngs = timed = comparisons = 0
for r in d['rows']:
    assert r['size'] == [[3456, 2234], [1728, 1117]][r['resolution']]
    assert len(r['checks']) == 5 and len(r['counts']) == 6
    for c in r['checks']:
        assert c['changed_pixels'] == c['id_mismatches'] == c['max_rgb_difference'] == 0
        comparisons += 1
    for mode, c in enumerate(r['counts']):
        assert 0 < c['triangles'] <= r['counts'][0]['triangles']
        assert c['submitted_geometry_commands'] == (3 if mode in [0, 2, 4] else 479)
        assert c['active_geometry_commands'] <= c['submitted_geometry_commands']
    if r['timings']:
        timed += 1
        assert len(r['timings']) == 12
        for i, t in enumerate(r['timings']):
            assert t['mode'] == d['modes'][i % 6]
            for k in ['gpu_ms', 'serial_ms', 'moving_throughput_ms', 'cpu_prepare_encode_ms']:
                assert math.isfinite(t[k]['median']) and t[k]['median'] > 0
            assert (t['pyramid_ms'] is not None) == (i % 6 in [2, 3])
            assert (t['cull_ms'] is not None) == (i % 6 > 0)
    reference_png = (ROOT/f"{r['id']}_full.png").read_bytes()
    for key in ['full', '2', '3', '4', '5']:
        b = (ROOT/f"{r['id']}_{key}.png").read_bytes()
        assert b == reference_png
        assert b[:8] == b'\x89PNG\r\n\x1a\n'
        assert list(struct.unpack('>II', b[16:24])) == r['size']
        offset = 8
        compressed = bytearray()
        ended = False
        while offset < len(b):
            n = struct.unpack('>I', b[offset:offset+4])[0]
            tag = b[offset+4:offset+8]
            payload = b[offset+8:offset+8+n]
            assert zlib.crc32(tag+payload) == struct.unpack('>I', b[offset+8+n:offset+12+n])[0]
            if tag == b'IDAT':
                compressed.extend(payload)
            offset += n+12
            if tag == b'IEND':
                ended = True
                break
        assert ended and offset == len(b) and b[24] == 8
        channels = {2: 3, 6: 4}[b[25]]
        assert len(zlib.decompress(compressed)) == (r['size'][0]*channels+1)*r['size'][1]
        pngs += 1
assert timed == 60 and pngs == 780 and len(edges) == 16
for edge in edges:
    assert len(edge['checks']) == 5
    for c in edge['checks']:
        assert c['changed_pixels'] == c['id_mismatches'] == c['max_rgb_difference'] == 0
        comparisons += 1
for resolution in [0, 1]:
    for scenario in d['scenarios']:
        rows = [r for r in d['rows'] if r['resolution'] == resolution and r['scenario'] == scenario]
        assert [r['step'] for r in rows] == list(range(13))+list(reversed(range(13)))
        for a, b in zip(rows, reversed(rows)):
            assert a['eye'] == b['eye'] and a['counts'] == b['counts']
result = {'poses': 156, 'edge_poses': 16, 'timed_poses': timed, 'exact_float_rgb_and_id_comparisons': comparisons, 'pngs': pngs, 'native_browser': False}
(ROOT/'verification.json').write_text(json.dumps(result, indent=2))
print(json.dumps(result))
