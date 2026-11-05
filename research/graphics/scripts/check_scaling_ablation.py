"""Validate current artifacts, prior controls, physical resolution and all quality gates."""
import json,math,struct,zlib
from pathlib import Path
ROOT=Path('artifacts/appearance-probe/scaling-ablation')
def png(path,size):
    b=path.read_bytes();assert b[:8]==b'\x89PNG\r\n\x1a\n' and list(struct.unpack('>II',b[16:24]))==size
    offset=8;compressed=bytearray();ended=False
    while offset<len(b):
        n=struct.unpack('>I',b[offset:offset+4])[0];tag=b[offset+4:offset+8];payload=b[offset+8:offset+8+n]
        assert zlib.crc32(tag+payload)==struct.unpack('>I',b[offset+8+n:offset+12+n])[0]
        if tag==b'IDAT':compressed.extend(payload)
        offset+=n+12
        if tag==b'IEND':ended=True;break
    assert ended and offset==len(b) and b[24]==8
    assert len(zlib.decompress(compressed))==(size[0]*{2:3,6:4}[b[25]]+1)*size[1]
    return b

def gate(q):
    assert all(math.isfinite(q[k])and q[k]>=0 for k in ['foreground_rmse','p99_rgb','max_rgb','coverage_fraction'])
    passed=q['foreground_rmse']<=.005 and q['p99_rgb']<=.02 and q['coverage_fraction']<=.0005
    assert q['quality_gate']==passed
    return passed

rows=json.loads((ROOT/'quality.json').read_text());assert len(rows)==208
depth={r['id']:r for r in json.loads((ROOT/'depth-quality.json').read_text())};assert len(depth)==208
assert len({r['id']for r in rows})==208
failures=[];maxima={};images=0;old_images=0;order_exact=0
for r in rows:
    assert r['size']==[[3456,2234],[1728,1117]][r['resolution']]
    assert r['triangles'][0]==32768*[1,2,10,100][r['density']]
    assert len(set(r['triangles'][1:]))==1
    assert gate(r['old_check'])
    assert [c['code']for c in r['checks']]==[43,44,45,63,64,65,75]
    for c in r['checks']:
        if not gate(c['quality']):failures.append({'id':r['id'],'code':c['code'],'quality':c['quality']})
        assert c['quality']['coverage_mismatches']==0,(r['id'],c)
        m=maxima.setdefault(c['code'],{'rmse':0,'p99':0,'change':0,'exact':0})
        m['rmse']=max(m['rmse'],c['quality']['foreground_rmse']);m['p99']=max(m['p99'],c['quality']['p99_rgb']);m['change']=max(m['change'],c['max_change']);m['exact']+=int(c['exact'])
    for c in r['order_checks']:
        assert gate(c['quality']);order_exact+=int(c['exact'])
    c=depth[r['id']];assert c['controls_identical']
    if not gate(c['quality']):failures.append({'id':r['id'],'code':85,'quality':c['quality']})
    assert c['quality']['coverage_mismatches']==0
    m=maxima.setdefault(85,{'rmse':0,'p99':0,'change':0})
    m['rmse']=max(m['rmse'],c['quality']['foreground_rmse']);m['p99']=max(m['p99'],c['quality']['p99_rgb']);m['change']=max(m['change'],c['max_change'])
    for m in range(6):
        b=png(ROOT/f"{r['id']}_{m}.png",r['size']);images+=1
        if r['frame']>6:
            reverse=f"f{r['family']}_d{r['density']}_r{r['resolution']}_{12-r['frame']}_{m}.png"
            assert b==(ROOT/reverse).read_bytes()
        old_frame={0:0,2:1,3:2,5:3,6:4,7:5,9:6,10:7,12:8}.get(r['frame'])
        if old_frame is not None and m in [0,1]:
            path=Path('artifacts/appearance-probe/cost-scaling')/f"f{r['family']}_d{r['density']}_r{r['resolution']}_{old_frame}_{0 if m==0 else 3}.png"
            assert b==path.read_bytes(),str(path);old_images+=1
transitions=json.loads((ROOT/'depth-transitions.json').read_text());assert len(transitions)==20
for t in transitions:
    if not gate(t['quality']):failures.append({'id':t['id'],'code':85,'quality':t['quality']})
    assert t['quality']['coverage_mismatches']==0
    for m in [0,5]:png(ROOT/f"{t['id']}_{m}.png",t['size']);images+=1
for w in [3456,1728]:assert {t['strategy']for t in transitions if t['size'][0]==w}=={3,4}
cohorts={}
for name,count in [('diagnostic.json',4),('order.json',4),('confirmation.json',28),('recheck.json',3),('depth.json',28),('depth-recheck.json',3)]:
    cs=json.loads((ROOT/name).read_text());assert len(cs)==count
    ratios=[]
    for c in cs:
        assert len(c['timings'])==len(c['codes'])*8
        half=len(c['timings'])//2
        for i,t in enumerate(c['timings']):
            assert len(t['samples_ms'])==len(t['gpu_samples_ms'])==8
            assert all(math.isfinite(v)and v>0 for v in t['samples_ms']+t['gpu_samples_ms'])
            assert t['variant']==c['codes'][i%len(c['codes'])]//10
        ratios=[c['timings'][i+half]['throughput_ms']['median']/c['timings'][i]['throughput_ms']['median']for i in range(half)]+ratios
    cohorts[name]={'count':count,'duplicate_total_ratio':[min(ratios),max(ratios)]}
result={'poses':len(rows),'candidate_quality_comparisons':len(rows)*8+20,'transition_poses':20,'depth_controls_identical':416,'quality_failures':failures,'maxima':maxima,'pngs':images,'old_controls_identical':old_images,'return_images_identical':True,'order_comparisons':len(rows)*2,'order_exact':order_exact,'cohorts':cohorts,'native_browser':False}
(ROOT/'verification.json').write_text(json.dumps(result,indent=2));print(json.dumps(result))
assert not failures,failures
