"""Verify frozen model, physical captures, chosen path, quality and paired timings."""
import hashlib,json,math,struct,zlib
from pathlib import Path
ROOT=Path('artifacts/appearance-probe/automatic-shading')
def png(path,size):
 b=path.read_bytes();assert b[:8]==b'\x89PNG\r\n\x1a\n' and list(struct.unpack('>II',b[16:24]))==size
 off=8;compressed=bytearray();ended=False
 while off<len(b):
  n=struct.unpack('>I',b[off:off+4])[0];tag=b[off+4:off+8];data=b[off+8:off+8+n]
  assert zlib.crc32(tag+data)==struct.unpack('>I',b[off+8+n:off+12+n])[0]
  if tag==b'IDAT':compressed.extend(data)
  off+=n+12
  if tag==b'IEND':ended=True;break
 assert ended and off==len(b) and b[24]==8
 assert len(zlib.decompress(compressed))==(size[0]*{2:3,6:4}[b[25]]+1)*size[1]
 return b
def gate(q):
 assert all(math.isfinite(q[k]) and q[k]>=0 for k in ['foreground_rmse','p99_rgb','max_rgb','coverage_fraction'])
 ok=q['foreground_rmse']<=.005 and q['p99_rgb']<=.02 and q['coverage_fraction']<=.0005
 assert q['quality_gate']==ok
 return ok
fit=json.loads((ROOT/'fit.json').read_text())
for file,key in [('model.json','model_sha256'),('calibration.json','calibration_sha256')]:
 assert hashlib.sha256((ROOT/file).read_bytes()).hexdigest()==fit[key]
rows=json.loads((ROOT/'quality.json').read_text());assert len(rows)==304
assert len({r['id'] for r in rows})==304
transitions=json.loads((ROOT/'transitions.json').read_text())
failures=[];oldcontrols=0;images=0
for r in rows:
 assert r['size']==[[3456,2234],[1728,1117]][r['resolution']]
 assert r['chosen_image_identical'] and r['chosen'] in [5,85]
 assert r['triangles'][0]==32768*[1,2,10,100][r['density']]
 assert len(set(r['triangles'][1:]))==1
 for m,q in enumerate(r['checks']):
  if not gate(q):failures.append([r['id'],m,q])
 for m in range(4):
  b=png(ROOT/f"{r['id']}_{m}.png",r['size']);images+=1
  if r['frame']>9:
   assert b==(ROOT/f"f{r['family']}_d{r['density']}_r{r['resolution']}_{18-r['frame']}_{m}.png").read_bytes()
  if m==3:
   assert b==(ROOT/f"{r['id']}_{2 if r['chosen']==85 else 1}.png").read_bytes()
  oldframe={14:0,21:1,28:2,56:3,84:4,112:5,224:6}.get(r['distance'])
  if oldframe is not None and m<3:
   old=Path('artifacts/appearance-probe/scaling-ablation')/f"f{r['family']}_d{r['density']}_r{r['resolution']}_{oldframe}_{[0,1,5][m]}.png"
   assert b==old.read_bytes(),str(old);oldcontrols+=1
for t in transitions:
 assert t['chosen_image_identical']
 if not gate(t['quality']):failures.append(t)
cohorts={}
for name,count in [('calibration',24),('validation',40),('repeat',8)]:
 cs=json.loads((ROOT/f'{name}.json').read_text());assert len(cs)==count
 ratios=[]
 for c in cs:
  assert len(c['timings'])==len(c['codes'])*8;half=len(c['timings'])//2
  if name!='calibration':
   assert c['distance'] not in fit['training_distances']
   for d in range(4):
    q=next(r for r in rows if all(r[k]==c[k] for k in ['family','resolution','distance']) and r['density']==d)
    assert c['decisions'][d]==q['chosen']
  for i,t in enumerate(c['timings']):
   assert len(t['samples_ms'])==len(t['gpu_samples_ms'])==8
   assert all(math.isfinite(v) and v>0 for v in t['samples_ms']+t['gpu_samples_ms'])
   assert t['variant']==c['codes'][i%len(c['codes'])]//10
  ratios.extend(c['timings'][i+half]['throughput_ms']['median']/c['timings'][i]['throughput_ms']['median'] for i in range(half))
 cohorts[name]={'count':count,'duplicate_ratio_range':[min(ratios),max(ratios)]}
auto=[r['checks'][2] for r in rows]+[t['quality'] for t in transitions]
result={'poses':len(rows),'transitions':len(transitions),'quality_failures':failures,'auto_max_rmse':max(q['foreground_rmse'] for q in auto),'auto_max_p99':max(q['p99_rgb'] for q in auto),'auto_max_coverage':max(q['coverage_fraction'] for q in auto),'pngs':images,'old_controls_exact':oldcontrols,'chosen_path_exact':True,'return_exact':True,'cohorts':cohorts,'native_browser':False}
(ROOT/'verification.json').write_text(json.dumps(result,indent=2));print(json.dumps(result,indent=2));assert not failures
