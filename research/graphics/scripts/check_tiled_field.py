"""Validate independent inputs, frozen policy, all images and paired timings."""
import hashlib,itertools,json,math,struct,zlib,re
from pathlib import Path
R=Path('artifacts/appearance-probe/tiled-field')
def read(n):return json.loads((R/f'{n}.json').read_text())
def fnv(b):
 h=0xcbf29ce484222325
 for v in b:h=((h^v)*0x100000001b3)&((1<<64)-1)
 return h
for p,h in read('input-manifest').items():assert hashlib.sha256(Path(p).read_bytes()).hexdigest()==h,p
raw=bytearray()
for d in range(4):
 source=Path(f'artifacts/appearance-probe/geometric-field/f1_d{d}.igf').read_bytes()
 b=(Path('artifacts/appearance-probe/hierarchical-field')/f'f1_d{d}.ihf').read_bytes();assert b[:4]==b'IHF1'
 version,nx,nz,w,h,levels,regions=struct.unpack('<7I',b[4:32]);assert version==1 and regions==64
 assert (nx,nz)==struct.unpack('<II',source[8:16])
 assert struct.unpack('<Q',b[32:40])[0]==fnv(source)
 assert struct.unpack('<Q',b[40:48])[0]==fnv(b[56:]) and struct.unpack('<Q',b[48:56])[0]==fnv(b[:48])
 assert len(b)==56+1024+sum(max(1,w>>l)*max(1,h>>l)*8 for l in range(levels))
 raw.extend(source);raw.extend(b)
raw.extend(Path('examples/appearance_probe/geometric_field.wgsl').read_bytes())
raw.extend(Path('examples/appearance_probe/hierarchical_field.wgsl').read_bytes())
raw.extend(Path('examples/appearance_probe/tiled_field.wgsl').read_bytes())
raw.extend(Path('examples/appearance_probe/geometric_field.rs').read_bytes())
rounds=int(re.search(r'const EDGE: u32 = (\d+)',Path('examples/appearance_probe/tiled_field.rs').read_text()).group(1));raw.extend(struct.pack('<I',rounds))
p=(R/'policy.bin').read_bytes();assert len(p)==40 and p[:4]==b'ITP1' and p[5:8]==bytes(3)
assert hashlib.sha256(p).hexdigest()==(R/'frozen-policy.sha256').read_text().strip()
assert struct.unpack('<Q',p[24:32])[0]==fnv(raw) and struct.unpack('<Q',p[32:])[0]==fnv(p[:32])
mask=p[4];assert mask==read('fit')['enabled_mask']
rows=read('quality');expected=set(itertools.product(range(4),range(2),[14.,28.,224.],[2],range(9)))
expected|={(3,r,z,s,f) for r,z,s,f in itertools.product(range(2),[14.,28.],[0,1],range(9))}
assert {(r['density'],r['resolution'],r['distance'],r['scene'],r['frame']) for r in rows}==expected
hashes={}
def png(path,size):
 b=path.read_bytes();assert b[:8]==b'\x89PNG\r\n\x1a\n' and list(struct.unpack('>II',b[16:24]))==size
 off=8;data=bytearray()
 while off<len(b):
  n=struct.unpack('>I',b[off:off+4])[0];tag=b[off+4:off+8];chunk=b[off+8:off+8+n];assert zlib.crc32(tag+chunk)==struct.unpack('>I',b[off+8+n:off+12+n])[0]
  if tag==b'IDAT':data.extend(chunk)
  off+=n+12
  if tag==b'IEND':break
 assert off==len(b) and len(zlib.decompress(data))==(size[0]*3+1)*size[1]
 return hashlib.sha256(b).hexdigest()
for r in rows:
 assert len(r['checks'])==3
 for q in r['checks']+[r['forced_vs_baseline']]:
  assert all(math.isfinite(q[k]) for k in ['foreground_rmse','p99_rgb','max_rgb','coverage_fraction'])
  assert q['quality_gate']==(q['foreground_rmse']<=.005 and q['p99_rgb']<=.02 and q['coverage_fraction']<=.0005)
  # Diagnostic forced failures are reported, never hidden.
 assert r['symbol_selected']==(r['eligible'] and bool(mask&(1<<r['class'])))
 assert r['checks'][0]['quality_gate'] and r['checks'][2]['quality_gate'],r['id']
 assert r['forced_triangles']==(1 if r['eligible'] else r['triangles'])
 assert r['optional_triangles']==(1 if r['symbol_selected'] else r['triangles'])
 chosen=r['checks'][1 if r['symbol_selected'] else 0]
 assert all(r['checks'][2][k]==v for k,v in chosen.items() if k!='temporal_residual_rmse')

 for m in [0,135,395]:hashes[r['id'],m]=png(R/f"{r['id']}_{m}.png",r['size'])
closure=0
for r in rows:
 if r['frame']==8:
  for m in [0,135,395]:assert hashes[r['id'],m]==hashes[r['id'][:-1]+'0',m];closure+=1
for d,r in itertools.product(range(4),range(2)):
 p=R/f'legacy_d{d}_r{r}.png';assert p.read_bytes()==Path(f'artifacts/appearance-probe/near-plateau/f1_d{d}_r{r}_1_5.png').read_bytes()
 png(p,[[3456,2234],[1728,1117]][r])
for phase,distances in [('calibration',[16.,24.,48.,96.,192.]),('timings',[14.,28.,224.]),('repeat',[14.,224.])]:
 times=read(phase)
 assert {(r['density'],r['resolution'],r['distance']) for r in times}==set(itertools.product(range(4),range(2),distances))
 for r in times:
  assert len(r['classes'])==64
  assert 0<=r['selected']<=r['eligible']<=64
  if mask==0:assert r['selected']==0
  modes=[135,395,345] if phase=='calibration' else [135,395,405,345]
  assert {(s['mode'],s['duplicate']) for s in r['samples']}==set(itertools.product(modes,range(2)))
  for s in r['samples']:
   for k in ['total','gpu','cpu_submit','instrumented_total','pass_latency']:assert len(s[k])==8 and all(math.isfinite(x) and x>0 for x in s[k])
  for s in r['samples']:
   assert len(s['gpu_overlap_count'])==8 and all(0<=n<=31 for n in s['gpu_overlap_count'])
   assert all(g<=w*1.05+.01 for g,w in zip(s['gpu'],s['instrumented_total'])), 'GPU envelope cannot exceed same batch wall time'
audit=read('timestamp-audit');assert len(audit)==4
# Compute/read dependency can serialize batches; overlap is measured, never required.
for r in audit:
 for s in r['samples']:
  assert all(0<=n<=31 for n in s['gpu_overlap_count'])
  assert all(g<=w*1.05+.01 for g,w in zip(s['gpu'],s['instrumented_total']))
# Ensure the path actually moves the requested inputs independently.
for scene in [0,1,2]:
 rs=[r for r in rows if r['density']==3 and r['resolution']==0 and r['distance']==28. and r['scene']==scene]
 assert any(r['eligible'] for r in rs), 'independent motion must exercise geometric symbol'
 eyes={tuple(r['eye']) for r in rs};lights={tuple(r['light']) for r in rs}
 assert (len(eyes)>1)==(scene!=1) and (len(lights)>1)==(scene!=0)
depth=read('depth');assert len(depth)==18 and sum(r['symbol_active'] for r in depth)>=16 and all(r['max_depth_error']<.00001 and r['depth_coverage_matches_color'] for r in depth)
fixed=read('fixed-quality');assert len(fixed)==48
assert all(r['invalid_certificate_fallback'] and r['bound_px']<=.15 and all(q['quality_gate'] for q in r['checks']) for r in fixed)
prep=read('fixed-preparation');assert len(prep)==8
for r in prep:
 m=r['metadata'];assert abs(m['actual_triangles_per_pixel']-r['target'])<=1/m['covered_pixels_at14']+1e-10
for resolution,distance,mode in itertools.product(range(2),[14,224],[135,345,395,405]):
 png(R/f'fixed_r{resolution}_z{distance}_m{mode}.png',[[3456,2234],[1728,1117]][resolution])
 if mask==0 and mode==405:assert (R/f'fixed_r{resolution}_z{distance}_m405.png').read_bytes()==(R/f'fixed_r{resolution}_z{distance}_m135.png').read_bytes()
ft=read('fixed-timings');assert len(ft)==16
for r in ft:
 assert {(s['mode'],s['duplicate']) for s in r['samples']}==set(itertools.product([1140,135,345,395,405],range(2)))
 assert all(len(s['total'])==4 and all(math.isfinite(v) and v>0 for v in s['total']) for s in r['samples'])
summary=dict(fixed_poses=len(fixed),fixed_comparisons=len(fixed)*5,fixed_pngs=16,depth_cases=len(depth),poses=len(rows),quality_comparisons=len(rows)*3,passed=sum(q['quality_gate'] for r in rows for q in r['checks']),pngs=len(hashes)+8,closed_image_pairs=closure,legacy_baselines_identical=8,optional_equals_baseline=not any(r['symbol_selected'] for r in rows),symbol_selections=sum(r['symbol_selected'] for r in rows),forced_symbol_coverage_exact=all(r['checks'][1]['coverage_mismatches']==0 for r in rows),eligible_poses=sum(r['eligible'] for r in rows),frozen_policy=True,calibration_cohorts=40,validation_cohorts=24,repeat_cohorts=16,native_browser=False,human_visual_acceptance=False)
(R/'verification.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary,indent=2))
