"""Integrity, unchanged controls, selected-image identity and unchanged quality gates."""
import hashlib,json,math,struct,zlib
from pathlib import Path
ROOT=Path('artifacts/appearance-probe/cost-scaling')
SOURCE=Path('artifacts/appearance-probe/hybrid-scaling')
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
    assert all(math.isfinite(q[k]) and q[k]>=0 for k in ['foreground_rmse','p99_rgb','max_rgb','coverage_fraction'])
    passed=q['foreground_rmse']<=.005 and q['p99_rgb']<=.02 and q['coverage_fraction']<=.0005
    assert q['quality_gate']==passed
    return passed

d=json.loads((ROOT/'report.json').read_text());assert len(d['assets'])==8 and len(d['rows'])==144
failures=[];images=0;legacy_checks=0
for a in d['assets']:
    assert a['levels'][-1]['count']//3==32768*a['factor']==a['sealed'][-1]['count']//3
    assert len(a['regions'])==64
    assert a['regional_floor']==sum(r['floor']for r in a['regions'])
    assert all(r['floor']==min(l['count']//3 for l in r['levels'])for r in a['regions'])
    assert a['bytes']==sum((SOURCE/f"{a['id']}.{k}").stat().st_size for k in ['vertices','indices','normals'])
for r in d['rows']:
    assert r['size']==[[3456,2234],[1728,1117]][r['resolution']]
    assert r['triangles'][0]==32768*r['factor'] and len(r['checks'])==4
    assert r['chosen_image_identical'] and 0<=r['visited']<=64 and 1<=r['draws']<=64
    chosen=2 if r['strategy']==4 else 1
    assert r['strategy']in[3,4] and r['triangles'][4]==r['triangles'][chosen]
    for m,q in enumerate(r['checks'],1):
        if not gate(q):failures.append({'id':r['id'],'mode':d['modes'][m],'check':q})
        assert q['quality_gate'],(r['id'],d['modes'][m],q)
    pixels=[]
    for m in range(5):
        b=png(ROOT/f"{r['id']}_{m}.png",r['size']);pixels.append(b);images+=1
        if r['frame']>4:
            reverse=f"f{r['family']}_d{r['density']}_r{r['resolution']}_{8-r['frame']}_{m}.png"
            assert b==(ROOT/reverse).read_bytes()
        if m in[0,1,2,3]:
            old=SOURCE/f"{r['id']}_{[0,2,3,4][m]}.png"
            if old.exists():assert b==old.read_bytes();legacy_checks+=1
    assert pixels[4]==pixels[chosen]
transitions=json.loads((ROOT/'transitions.json').read_text());assert transitions
for t in transitions:
    assert t['chosen_image_identical'] and t['strategy']in[3,4]
    assert gate(t['check']),(t['id'],t['check'])
    for m in [0,4]:png(ROOT/f"{t['id']}_{m}.png",t['size']);images+=1
for width in[3456,1728]:assert {t['strategy']for t in transitions if t['size'][0]==width}=={3,4}
c=json.loads((ROOT/'confirmation.json').read_text());assert len(c)==20
for cohort in c:
    assert cohort['batch_frames']==128 and len(cohort['timings'])==40
    for i,t in enumerate(cohort['timings']):
        assert t['mode']==d['modes'][i%5]
        assert len(t['samples_ms'])==len(t['gpu_samples_ms'])==8
        assert all(math.isfinite(v) and v>0 for v in t['samples_ms']+t['gpu_samples_ms'])
        assert all(math.isfinite(t[k]['median']) and t[k]['median']>0 for k in ['gpu_ms','throughput_ms','cpu_ms'])
cal=json.loads((ROOT/'calibration.json').read_text());assert len(cal)==16
fit=json.loads((ROOT/'fit.json').read_text());model=json.loads((ROOT/'model.json').read_text())
assert fit['calibration_sha256']==hashlib.sha256((ROOT/'calibration.json').read_bytes()).hexdigest()
assert all(x['model']==model for x in c) and d['model']==model
assert all(r['strategy']==(4 if sum(a*b for a,b in zip(r['features'],model['coefficients']))<=0 else 3) for r in d['rows'])
assert {x['distance']for x in c}.isdisjoint({x['distance']for x in cal})
assert all(x['rounds']==8 and x['gpu_batch']==32 for x in c+cal)
assert legacy_checks==576
cpu=json.loads((ROOT/'selection-cpu.json').read_text());assert len(cpu)==80
assert all(len(x['milliseconds_per_selection'])==4 and all(t['samples']==8 and math.isfinite(t['median']) and t['median']>0 for t in x['milliseconds_per_selection'])for x in cpu)
recheck=json.loads((ROOT/'recheck.json').read_text());assert len(recheck)==2
assert all(x['model']==model and len(x['timings'])==40 for x in recheck)

result={'recheck_cohorts':2,'selection_cpu_poses':80,'calibration_holdout_disjoint':True,'frozen_model':True,'poses':144,'quality_comparisons':576,'strategy_transition_poses':len(transitions),'pngs':images,'legacy_control_images_identical':legacy_checks,'chosen_image_identical':True,'return_pixels_identical':True,'cohorts':20,'quality_failures':failures,'native_browser':False}
(ROOT/'verification.json').write_text(json.dumps(result,indent=2));print(json.dumps(result))
