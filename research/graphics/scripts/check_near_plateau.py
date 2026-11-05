"""Check physical captures, unchanged model, triangle counts and full-frame timing."""
import hashlib,json,math,struct,zlib
from pathlib import Path
ROOT=Path('artifacts/appearance-probe/near-plateau')
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
auto=Path('artifacts/appearance-probe/automatic-shading');fit=json.loads((auto/'fit.json').read_text())
assert hashlib.sha256((auto/'model.json').read_bytes()).hexdigest()==fit['model_sha256']
rows=json.loads((ROOT/'quality.json').read_text());assert len(rows)==112
assert len({r['id'] for r in rows})==len(rows)
failures=[];exact_failures=[];oldcontrols=0;images=0
for r in rows:
 assert r['size']==[[3456,2234],[1728,1117]][r['resolution']]
 assert 0<r['triangles']<=32768*[1,2,10,100][r['density']]
 if not all(r['exact_rgba32f']):exact_failures.append(r['id'])
 assert len(r['checks'])==5
 for m,q in enumerate(r['checks']):
  assert all(math.isfinite(q[k]) and q[k]>=0 for k in ['foreground_rmse','p99_rgb','max_rgb','coverage_fraction'])
  ok=q['foreground_rmse']<=.005 and q['p99_rgb']<=.02 and q['coverage_fraction']<=.0005
  assert q['quality_gate']==ok
  if not ok:failures.append([r['id'],m,q])
 for m in range(6):
  b=png(ROOT/f"{r['id']}_{m}.png",r['size']);images+=1
  if r['frame']>3:
   assert b==(ROOT/f"f{r['family']}_d{r['density']}_r{r['resolution']}_{6-r['frame']}_{m}.png").read_bytes()
  if m in [3,4,5] and all(r['exact_rgba32f']):
   assert b==(ROOT/f"{r['id']}_{1 if m==5 else 2}.png").read_bytes()
  oldframe={14:0,28:3,224:9}.get(r['distance'])
  if oldframe is not None and m<3:
   old=auto/f"f{r['family']}_d{r['density']}_r{r['resolution']}_{oldframe}_{[0,3,2][m]}.png"
   assert b==old.read_bytes(),str(old);oldcontrols+=1
cohorts={}
for name,count in [('validation',16),('repeat',8)]:
 cs=json.loads((ROOT/f'{name}.json').read_text());assert len(cs)==count
 assert len({(c['family'],c['resolution'],c['distance']) for c in cs})==count
 ratios=[]
 for c in cs:
  assert c['codes']==[0,95,85,105,115,135]
  assert len(c['samples'])==len(c['codes'])*8;half=len(c['samples'])//2
  for d in range(4):
   q=next(r for r in rows if all(r[k]==c[k] for k in ['family','resolution','distance']) and r['density']==d)
   assert c['triangles'][d]==q['triangles']
  for i,t in enumerate(c['samples']):
   assert len(t['samples_ms'])==len(t['gpu_samples_ms'])==8
   assert all(math.isfinite(v) and v>0 for v in t['samples_ms']+t['gpu_samples_ms'])
   assert t['variant']==c['codes'][i%len(c['codes'])]//10
  ratios.extend(c['samples'][i+half]['throughput_ms']['median']/c['samples'][i]['throughput_ms']['median'] for i in range(half))
 cohorts[name]={'count':count,'duplicate_ratio_range':[min(ratios),max(ratios)]}
footprints=json.loads((ROOT/'footprints.json').read_text());assert len(footprints)==64
for f in footprints:
 assert f['source_triangles']==sum(f[k] for k in ['fully_inside','outside_common_halfspace','clip_crossing','behind_or_invalid'])
 assert sum(f['counts'])==f['fully_inside'] and f['mean_area_px2']>=0 and math.isfinite(f['mean_area_px2'])
 assert not f['occlusion_tested']
qs=[q for r in rows for q in r['checks']]
result={'poses':len(rows),'comparisons':len(qs),'quality_failures':failures,'exact_rgba32f_failures':exact_failures,'max_rmse':max(q['foreground_rmse'] for q in qs),'max_p99':max(q['p99_rgb'] for q in qs),'max_coverage':max(q['coverage_fraction'] for q in qs),'pngs':images,'old_controls_exact':oldcontrols,'return_exact':True,'cohorts':cohorts,'model_sha256':fit['model_sha256'],'native_browser':False}
(ROOT/'verification.json').write_text(json.dumps(result,indent=2));print(json.dumps(result,indent=2));assert not failures and not exact_failures
