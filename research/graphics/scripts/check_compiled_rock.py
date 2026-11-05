"""Verify binary layout, source integrity, image evidence and complete timing coverage."""
import hashlib,itertools,json,math,struct,zlib
from pathlib import Path
R=Path('artifacts/appearance-probe/compiled-rock')
def read(n): return json.loads((R/f'{n}.json').read_text())
def fnv(b):
    h=14695981039346656037
    for x in b:h=((h^x)*1099511628211)&((1<<64)-1)
    return h
# Hash implementation in the Rust experiment uses the same standard FNV-1a seed.
def png(p,size):
    b=p.read_bytes();assert b[:8]==b'\x89PNG\r\n\x1a\n';assert list(struct.unpack('>II',b[16:24]))==size
    off=8;data=bytearray()
    while off<len(b):
        n=struct.unpack('>I',b[off:off+4])[0];tag=b[off+4:off+8];payload=b[off+8:off+8+n]
        assert zlib.crc32(tag+payload)==struct.unpack('>I',b[off+8+n:off+12+n])[0]
        if tag==b'IDAT':data.extend(payload)
        off+=n+12
        if tag==b'IEND':break
    assert off==len(b) and b[24:26]==bytes([8,2])
    assert len(zlib.decompress(data))==(size[0]*3+1)*size[1]
    return hashlib.sha256(b).hexdigest()
build=read('build');assert build['source_triangles']==81920
for path,digest in read('input-manifest').items():assert hashlib.sha256(Path(path).read_bytes()).hexdigest()==digest,path
for l in build['levels']:
    raw=(R/f"rock{l['level']}.irk").read_bytes();h=struct.unpack('<17I',raw[:68]);assert h[:2]==(int.from_bytes(b'IRK1','little'),1)
    assert len(raw)==l['disk_bytes']==68+l['gpu_payload_bytes'];assert fnv(raw)==l['checksum'];assert fnv(raw[:60]+raw[68:])==h[15]|(h[16]<<32)
    words=struct.unpack('<'+'I'*((len(raw)-68)//4),raw[68:]);d=words[:h[12]];maps=words[h[12]:h[12]+h[13]];responses=words[h[12]+h[13]:]
    assert len(responses)==h[14]==h[3]*h[4]*(h[5]-1)
    assert sorted(x for x in d if x!=0xffffffff)==list(range(0,len(maps),64))
    assert all((x&65535)<h[5] for x in maps)
    for view in range(h[3]):
        # Every code distributed in the dictionary is actually referenced by this view.
        ids=set()
        for offset in d[view*(h[2]//8)**2:(view+1)*(h[2]//8)**2]:
            if offset!=0xffffffff:ids.update(x&65535 for x in maps[offset:offset+64])
        assert set(range(1,h[5]))<=ids
rows=read('quality');assert len(rows)==162
assert {(r['resolution'],r['distance_id'],r['scene'],r['frame']) for r in rows}==set(itertools.product(range(2),range(3),range(3),range(9)))
hashes={};failed=[0,0,0,0];count=0
for r in rows:
    for m in [0,7,8,9,10]:hashes[r['id'],m]=png(R/f"{r['id']}_{m}.png",r['size']);count+=1
    for i,q in enumerate(r['checks']):
        assert all(math.isfinite(q[k]) for k in ['rmse','p99','coverage_fraction'])
        assert q['pass']==(q['rmse']<=.005 and q['p99']<=.02 and q['coverage_fraction']<=.0005)
        failed[i]+=not q['pass']
for res,di,scene,m in itertools.product(range(2),range(3),range(3),[0,7,8,9,10]):assert hashes[f'r{res}_d{di}_s{scene}_0',m]==hashes[f'r{res}_d{di}_s{scene}_8',m]
checks=read('checks');load=read('load');assert not load['source_mesh_read'] and not load['json_read']
assert [c['hash'] for c in checks if c['mode']!=0]==load['hashes']
for m in [7,8,9,10]:assert png(R/f'standalone{m}.png',[1728,1117])==hashes['r1_d0_s2_0',m];count+=1
integrated=read('integrated');assert len(integrated)==6
for r in integrated:png(R/f"integrated_r{r['resolution']}_d{r['distance_id']}.png",[[3456,2234],[1728,1117]][r['resolution']]);count+=1
for r in read('timings'):
    assert len(r['samples'])==10
    for s in r['samples']:
        assert s['frames']==32
        for metric in ['total','cpu','gpu']:
            v=s[f'{metric}_samples_ms'];assert len(v)==7 and all(math.isfinite(x) and x>0 for x in v)
times=read('timings');assert len(times)==24
expected={(0,r,d,f) for r,d,f in itertools.product(range(2),range(3),[0,3,6])}|{(1,r,d,0) for r,d in itertools.product(range(2),range(3))}
assert {(r['repeat'],r['resolution'],r['distance_id'],r['frame']) for r in times}==expected
out={'poses':len(rows),'comparisons':len(rows)*4,'visual_failures':failed,'pngs_checked':count,'roundtrip_image_pairs':90,'standalone_binary_only_matches':True,'integrated_poses':6,'timing_cohorts':len(times),'binary_fields_and_codes_referenced':True,'native_browser':False}
(R/'verification.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out,indent=2))
