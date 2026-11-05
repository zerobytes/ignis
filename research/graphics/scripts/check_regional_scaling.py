"""Verify data integrity and report failures without changing the quality criterion."""
import json,math,struct,zlib
from pathlib import Path
ROOT=Path('artifacts/appearance-probe/regional-scaling')
def verify_png(path,size):
    b=path.read_bytes();assert b[:8]==b'\x89PNG\r\n\x1a\n'
    assert list(struct.unpack('>II',b[16:24]))==size;offset=8;compressed=bytearray();ended=False
    while offset<len(b):
        n=struct.unpack('>I',b[offset:offset+4])[0];tag=b[offset+4:offset+8];payload=b[offset+8:offset+8+n]
        assert zlib.crc32(tag+payload)==struct.unpack('>I',b[offset+8+n:offset+12+n])[0]
        if tag==b'IDAT':compressed.extend(payload)
        offset+=n+12
        if tag==b'IEND':ended=True;break
    assert ended and offset==len(b) and b[24]==8;channels={2:3,6:4}[b[25]]
    assert len(zlib.decompress(compressed))==(size[0]*channels+1)*size[1]
    return b

d=json.loads((ROOT/'report.json').read_text());assert len(d['rows'])==144 and len(d['assets'])==8
failures=[];pngs=timed=0
for a in d['assets']:
    assert a['levels'][-1]['count']//3==32768*a['factor']
    assert sum((ROOT/f"{a['id']}.{k}").stat().st_size for k in ['vertices','indices','normals'])==a['bytes']
    assert (ROOT/f"{a['id']}.normals").stat().st_size==(a['grid'][0]+1)*(a['grid'][1]+1)*16
    assert a['build_ms']>0
    assert len(a['regions'])==64
    assert sum(r['levels'][-1]['count'] for r in a['regions'])==a['levels'][-1]['count']
for r in d['rows']:
    assert r['size']==[[3456,2234],[1728,1117]][r['resolution']]
    assert r['triangles'][0]==32768*r['factor']
    assert sum(r['draw_triangles'])==r['triangles'][3] and 1<=len(r['draw_triangles'])<=64
    assert all(0<t<=r['triangles'][0] for t in r['triangles'])
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
        b=verify_png(ROOT/f"{r['id']}_{mode}.png",r['size'])
        if r['frame']>4:
            reverse=f"f{r['family']}_d{r['density']}_r{r['resolution']}_{8-r['frame']}_{mode}.png"
            assert b==(ROOT/reverse).read_bytes()
        pngs+=1
transitions=json.loads((ROOT/'transitions.json').read_text());assert len(transitions)==6
for t in transitions:
    assert 1<t['draws']<=64 and 0<t['triangles']<=3276800
    q=t['check'];passed=q['foreground_rmse']<=.005 and q['p99_rgb']<=.02 and q['coverage_fraction']<=.0005
    assert q['quality_gate']==passed
    if not passed:failures.append({'id':t['id'],'mode':3,'rmse':q['foreground_rmse'],'coverage_mismatches':q['coverage_mismatches']})
    for mode in [0,3]:
        verify_png(ROOT/f"{t['id']}_{mode}.png",t['size'])
c=json.loads((ROOT/'confirmation.json').read_text());assert len(c)==20
for case in c:
    assert case['batch_frames']==128 and len(case['timings'])==32
    for i,t in enumerate(case['timings']):
        assert t['mode']==d['modes'][i%4]
        assert all(math.isfinite(t[k]['median']) and t[k]['median']>0 for k in ['gpu_ms','serial_ms','throughput_ms','cpu_ms'])
assert timed==0 and pngs==576
result={'poses':144,'quality_comparisons':432,'timed_poses':80,'confirmation_cohorts':20,'pngs':pngs+2*len(transitions),'return_pixels_identical':True,'quality_failures':failures,'mixed_level_poses':len(transitions),'native_browser':False}
(ROOT/'verification.json').write_text(json.dumps(result,indent=2));print(json.dumps(result))
