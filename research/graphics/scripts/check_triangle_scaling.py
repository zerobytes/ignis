"""Verify data integrity and report failures without changing the quality criterion."""
import json,math,struct,zlib
from pathlib import Path
ROOT=Path('artifacts/appearance-probe/triangle-scaling')
d=json.loads((ROOT/'report.json').read_text());assert len(d['rows'])==144 and len(d['assets'])==8
failures=[];pngs=timed=0
for a in d['assets']:
    assert a['levels'][-1]['count']//3==32768*a['factor']
    assert sum((ROOT/f"{a['id']}.{k}").stat().st_size for k in ['vertices','indices','normals'])==a['bytes']
    assert (ROOT/f"{a['id']}.normals").stat().st_size==(a['grid'][0]+1)*(a['grid'][1]+1)*16
    assert a['build_ms']>0
for r in d['rows']:
    assert r['size']==[[3456,2234],[1728,1117]][r['resolution']]
    assert r['triangles'][0]==32768*r['factor']
    assert all(0<t<=r['triangles'][0] for t in r['triangles'])
    assert r['triangles'][1]==r['triangles'][3]
    assert abs(r['source_triangles_per_covered_pixel']-r['triangles'][0]/r['covered_pixels'])<1e-9
    for m,q in enumerate(r['checks'],1):
        assert all(math.isfinite(q[k]) and q[k]>=0 for k in ['foreground_rmse','p99_rgb','max_rgb','coverage_fraction'])
        passed=q['foreground_rmse']<=.005 and q['p99_rgb']<=.02 and q['coverage_fraction']<=.0005
        assert q['quality_gate']==passed
        if not passed:failures.append({'id':r['id'],'mode':m,'rmse':q['foreground_rmse'],'p99':q['p99_rgb'],'coverage_mismatches':q['coverage_mismatches']})
    if r['triangles'][0]==r['triangles'][3]:
        assert r['checks'][2]['foreground_rmse']<1e-5 and r['checks'][2]['coverage_mismatches']==0
    if r['timings']:
        timed+=1;assert len(r['timings'])==8
        for i,t in enumerate(r['timings']):
            assert t['mode']==d['modes'][i%4]
            assert all(math.isfinite(t[k]['median']) and t[k]['median']>0 for k in ['gpu_ms','serial_ms','throughput_ms','cpu_ms'])
    for mode in range(4):
        b=(ROOT/f"{r['id']}_{mode}.png").read_bytes();assert b[:8]==b'\x89PNG\r\n\x1a\n'
        assert list(struct.unpack('>II',b[16:24]))==r['size'];offset=8;compressed=bytearray();ended=False
        while offset<len(b):
            n=struct.unpack('>I',b[offset:offset+4])[0];tag=b[offset+4:offset+8];payload=b[offset+8:offset+8+n]
            assert zlib.crc32(tag+payload)==struct.unpack('>I',b[offset+8+n:offset+12+n])[0]
            if tag==b'IDAT':compressed.extend(payload)
            offset+=n+12
            if tag==b'IEND':ended=True;break
        assert ended and offset==len(b) and b[24]==8;channels={2:3,6:4}[b[25]]
        assert len(zlib.decompress(compressed))==(r['size'][0]*channels+1)*r['size'][1]
        if r['frame']>4:
            reverse=f"f{r['family']}_d{r['density']}_r{r['resolution']}_{8-r['frame']}_{mode}.png"
            assert b==(ROOT/reverse).read_bytes()
        pngs+=1
c=json.loads((ROOT/'confirmation.json').read_text());assert len(c)==7
for case in c:
    assert case['batch_frames']==128 and len(case['timings'])==32
    for i,t in enumerate(case['timings']):
        assert t['mode']==d['modes'][i%4]
        assert all(math.isfinite(t[k]['median']) and t[k]['median']>0 for k in ['gpu_ms','serial_ms','throughput_ms','cpu_ms'])
assert timed==80 and pngs==576
result={'poses':144,'quality_comparisons':432,'timed_poses':80,'confirmation_cohorts':7,'pngs':pngs,'return_pixels_identical':True,'quality_failures':failures,'native_browser':False}
(ROOT/'verification.json').write_text(json.dumps(result,indent=2));print(json.dumps(result))
