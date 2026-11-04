"""Build-time atlas calibration and independent verification."""
from pathlib import Path
import hashlib
import json
import struct
import html
import numpy as np
from PIL import Image, ImageDraw, ImageFont
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/"artifacts/lighting-bake"
METRICS=["rgb_mse","near_mse","pixel_p95_mse","worst_region_mse"]
SOURCE=["examples/lighting_bounce.rs","examples/lighting_bounce/transport.wgsl","examples/lighting_bounce/bake.rs","examples/lighting_bounce/bake.wgsl"]
def hashes():
    return {f:hashlib.sha256((ROOT/f).read_bytes()).hexdigest() for f in SOURCE}
def hash32(x):
    x &= 0xffffffff;x=((x^(x>>16))*0x7feb352d)&0xffffffff;x=((x^(x>>15))*0x846ca68b)&0xffffffff;return x^(x>>16)
def direct_cpu(recv,scene,index,cols,rows,frame,seed_salt=9923,seed_index=None,accumulate32=False):
    lo=np.array([np.array(e["transform"]["position"])-np.array(e["transform"]["size"])/2 for e in scene["entities"]]);hi=np.array([np.array(e["transform"]["position"])+np.array(e["transform"]["size"])/2 for e in scene["entities"]]);colors=np.array([e["color"] for e in scene["entities"]])
    sun=-np.array(scene["light"]["direction"]);sun/=np.linalg.norm(sun);energy=np.array(scene["light"]["direct_color"])*scene["light"]["direct_intensity"]
    p=recv[index,:3];n=recv[index,4:7];t=np.cross([1.,0.,0.] if abs(n[1])>.9 else [0.,1.,0.],n);t/=np.linalg.norm(t);origin=p+n*.002
    def slabs(p,d):
        with np.errstate(divide="ignore",invalid="ignore"):
            a=(lo-p)/d;b=(hi-p)/d
        return np.minimum(a,b).max(axis=1),np.maximum(a,b).min(axis=1)
    seed=hash32((index if seed_index is None else seed_index)+frame*91237+seed_salt);total=np.zeros(3)
    for j in range(cols*rows):
        seed=hash32(seed+0x9e3779b9);a=(seed>>8)/16777216;seed=hash32(seed+0x9e3779b9);b=(seed>>8)/16777216
        a=(j%cols+a)/cols;b=(j//cols+b)/rows
        d=t*np.sqrt(a)*np.cos(2*np.pi*b)+np.cross(n,t)*np.sqrt(a)*np.sin(2*np.pi*b)+n*np.sqrt(1-a)
        en,ex=slabs(origin,d);valid=(en>.00001)&(ex>=en)
        if not valid.any():continue
        k=int(np.argmin(np.where(valid,en,np.inf)));hit=origin+d*en[k];v=(hit-(lo[k]+hi[k])/2)/((hi[k]-lo[k])/2);axis=2
        if abs(v[0])>abs(v[1]) and abs(v[0])>abs(v[2]):axis=0
        elif abs(v[1])>abs(v[2]):axis=1
        normal=np.zeros(3);normal[axis]=np.sign(v[axis]);cosine=max(float(normal@sun),0.)
        if cosine==0:continue
        en,ex=slabs(hit+normal*.002,sun)
        if np.any((ex>=np.maximum(en,0))&(ex>.00001)):continue
        total+=colors[k]*energy*cosine
        if accumulate32:total=total.astype(np.float32)
    return recv[index,8:11]*total/(cols*rows)

def package(path):
    raw=path.read_bytes();assert raw[:8]==b'IGLBAKE1';n=struct.unpack('<I',raw[8:12])[0]
    header=json.loads(raw[12:12+n]);payload=raw[12+n:];assert hashlib.sha256(payload).hexdigest()==header['payload_sha256']
    data=np.frombuffer(payload,dtype='<f4').reshape(header['objects'],6,header['side'],header['side'],4).astype(float)
    assert np.isfinite(data).all() and (data>=0).all();assert np.isin(data[:,:,:,:,3],[0,1]).all()
    return header,data

def atlas_checks(scene,recv,header,atlas,values,method,scale,seed):
    live=np.flatnonzero(recv[:,3]>0);side=header['side'];p=recv[live,:3];n=recv[live,4:7];obj=recv[live,3].astype(int)-1
    axis=np.argmax(abs(n),axis=1);face=axis*2+(n[np.arange(len(n)),axis]<0)
    lo=np.array([np.array(e['transform']['position'])-np.array(e['transform']['size'])/2 for e in scene['entities']]);hi=np.array([np.array(e['transform']['position'])+np.array(e['transform']['size'])/2 for e in scene['entities']])
    uvw=np.clip((p-lo[obj])/(hi[obj]-lo[obj]),0,1);u=np.where(axis==0,uvw[:,1],uvw[:,0]);v=np.where(axis==2,uvw[:,1],uvw[:,2])
    g=np.clip(np.stack([u,v],axis=1)*side-.5,0,side-1);cell=np.minimum(np.floor(g).astype(int),side-2);f=g-cell;total=np.zeros((len(live),3));sw=np.zeros(len(live))
    for y in [0,1]:
        for x in [0,1]:
            val=atlas[obj,face,cell[:,1]+y,cell[:,0]+x];w=(f[:,0] if x else 1-f[:,0])*(f[:,1] if y else 1-f[:,1])*(val[:,3]>0)
            total+=val[:,:3]*w[:,None];sw+=w
    expected=np.divide(total,sw[:,None],out=np.zeros_like(total),where=sw[:,None]>0)*np.array(scale)*recv[live,8:11]
    direct=sw<=.00001
    if method=='hybrid':
        edges=np.minimum(p-lo[obj],hi[obj]-p);edges[np.arange(len(axis)),axis]=1e20;risk=edges.min(axis=1)
        for j in range(len(lo)):
            d=np.linalg.norm(np.maximum(np.maximum(lo[j]-p,p-hi[j]),0),axis=1);d[obj==j]=1e20;risk=np.minimum(risk,d)
        size=hi[obj]-lo[obj];size[np.arange(len(axis)),axis]=0;cell_size=size.max(axis=1)/side
        direct|=risk<=1.5*cell_size
    actual_direct=values[live,3]==2
    # f32 boundary tolerance only; exact flags elsewhere.
    if method=='hybrid':
        stable=abs(risk-1.5*cell_size)>2e-5
        assert np.array_equal(direct[stable],actual_direct[stable])
    else:assert np.array_equal(direct,actual_direct)
    error=float(np.max(abs(expected[~actual_direct]-values[live[~actual_direct],:3]),initial=0));assert error<3e-6,(method,error)
    indices=live[actual_direct]
    direct_error=0.
    if len(indices):
        for i in indices[np.linspace(0,len(indices)-1,min(6,len(indices)),dtype=int)]:
            v=direct_cpu(recv,scene,int(i),8,8,seed);direct_error=max(direct_error,float(np.max(abs(v-values[i,:3]))))
        assert direct_error<3e-6,direct_error
    return dict(lookup_max_error=error,direct_max_error=direct_error,direct_checks=min(6,len(indices)))

def report_scene(label,validation=False,final=False):
    path=OUT/label;data=json.loads((path/('final.json' if final else 'validation.json' if validation else 'calibration.json')).read_text());w,h=data['width'],data['height'];assert all(data[k] for k in ['round_trip','corrupt_rejected','truncated_rejected','native_profile_identity','full_invalidation_exact'])
    reports=[];bake_cpu=[];bake_cpu_nonzero=0
    for case in data['cases']:
        loc=path/case['label'];scene=json.loads((loc/'scene.json').read_text());recv=np.fromfile(loc/'receivers.bin',dtype='<f4').reshape(-1,12).astype(float);live=recv[:,3]>0
        ref=np.mean([np.fromfile(loc/f'reference-{m}.rgba32f',dtype='<f4').reshape(-1,4)[:,:3] for m in [7,8]],axis=0)
        assert np.isfinite(ref).all() and (ref>=0).all() and (ref[~live]==0).all()
        # Fixed spatial reporting band, independent of allocation.
        band=live&(abs(recv[:,0]-.27)<.75)&(recv[:,2]>-1.5)&(recv[:,2]<3.5)
        if not band.any():band=live
        for variant in case['variants']:
            side=variant['side'];header,atlas=package(path/f'atlas-{side}.bin');timings=[]
            if not validation and not final:
                # Independent MC checks on valid samples across all six chart normals.
                for face in range(6):
                    valid=np.argwhere(atlas[:,face,:,:,3]>0);scores=atlas[valid[:,0],face,valid[:,1],valid[:,2],:3].sum(axis=1);obj,y,x=valid[int(np.argmax(scores))];e=scene['entities'][obj];lo=np.array(e['transform']['position'])-np.array(e['transform']['size'])/2;hi=np.array(e['transform']['position'])+np.array(e['transform']['size'])/2
                    uv=np.array([(x+.5)/side,(y+.5)/side]);v=np.zeros(3);axis=face//2;v[axis]=1 if face%2==0 else 0
                    v[[a for a in range(3) if a!=axis]]=uv;p=lo+(hi-lo)*v;n=np.zeros(3);n[axis]=1 if face%2==0 else -1
                    synthetic=np.zeros((1,12));synthetic[0,:3]=p;synthetic[0,4:7]=n;synthetic[0,8:11]=1
                    idx=(int(obj)*6+face)*side*side+int(y)*side+int(x)
                    expected=direct_cpu(synthetic,scene,0,64,64,317,519983,idx,True);error=float(np.max(abs(expected-atlas[obj,face,y,x,:3])));assert error<5e-6,(label,side,face,error)
                    bake_cpu.append(error);bake_cpu_nonzero+=bool(np.max(expected)>0)
            methods=sorted({row['method'] for row in variant['rows']})
            for method in methods:
                values=np.stack([np.fromfile(loc/f'{side}-{method}-{seed}.rgba32f',dtype='<f4').reshape(-1,4) for seed in range(4)]).astype(float)
                assert np.isfinite(values).all() and (values[:,:,:3]>=0).all() and (values[:,~live,:3]==0).all()
                err=np.mean((values[:,:,:3]-ref[None,:,:])**2,axis=2);regions=[]
                for y in range(0,h,h//8):
                    for x in range(0,w,w//8):
                        mask=live.reshape(h,w)[y:y+h//8,x:x+w//8]
                        if mask.sum()>=32:regions.append(float(err.reshape(4,h,w)[:,y:y+h//8,x:x+w//8][:,mask].mean()))
                rows=[r for r in variant['rows'] if r['method']==method];assert len(rows)==64
                checks=None
                if method in ['baked','stale'] or (method=='hybrid' and variant['reused']):checks=atlas_checks(scene,recv,header,atlas,values[3],method,variant['scale'] or [1,1,1],55)
                if method=='hybrid' and not variant['reused']:
                    baseline=np.stack([np.fromfile(loc/f'{side}-direct64-{seed}.rgba32f',dtype='<f4').reshape(-1,4) for seed in range(4)])
                    assert np.array_equal(values[:,:,:3],baseline[:,:,:3])
                timings.append(dict(method=method,gpu_ms=float(np.median([r['gpu_ms'] for r in rows])),gpu_p95_ms=float(np.percentile([r['gpu_ms'] for r in rows],95)),cpu_ms=float(np.median([r['cpu_ms'] for r in rows])),rgb_mse=float(err[:,live].mean()),near_mse=float(err[:,band].mean()),pixel_p95_mse=float(np.percentile(err[:,live],95)),worst_region_mse=max(regions),mse_per_seed=err[:,live].mean(axis=1).tolist(),direct64_fraction=float((values[:,live,3]==2).mean()),cpu_checks=checks))
            baseline=next(t for t in timings if t['method']=='direct64')
            for t in timings:t['ratios']={k:t[k]/baseline[k] for k in METRICS};t['eligible']=all(x<=1.+1e-9 for x in t['ratios'].values())
            reports.append(dict(label=label,case=case['label'],side=side,reused=variant['reused'],load_ms=variant['load_ms'],upload_wall_ms=variant['upload_wall_ms'],timings=timings))
    return dict(label=label,builds=data['builds'],reports=reports,bake_cpu_checks=len(bake_cpu),bake_cpu_nonzero_checks=bake_cpu_nonzero,bake_cpu_max_error=max(bake_cpu,default=0),adapter=data['adapter'])

def freeze(calibration):
    candidates=[]
    for side in [16,32,64]:
        rows=[next(r for r in x['reports'] if r['side']==side) for x in calibration.values()]
        hybrid=[next(t for t in r['timings'] if t['method']=='hybrid') for r in rows];baseline=[next(t for t in r['timings'] if t['method']=='direct64') for r in rows]
        candidates.append(dict(side=side,eligible=all(t['eligible'] for t in hybrid),worst_gpu_ratio=max(t['gpu_ms']/b['gpu_ms'] for t,b in zip(hybrid,baseline)),quality=[t['ratios'] for t in hybrid]))
    allowed=[c for c in candidates if c['eligible'] and c['worst_gpu_ratio']<1]
    selection=dict(side=min(allowed,key=lambda c:c['worst_gpu_ratio'])['side'] if allowed else None,candidates=candidates,source_sha256=hashes(),criteria=METRICS,quality_ratio_max=1.,guard_texels=1.5)
    (OUT/'selection.json').write_text(json.dumps(selection,indent=2));return selection

def write_report(calibration,validation,selection):
    cases=[r for d in [*calibration.values(),*validation.values()] for r in d['reports']]
    heldout=[r for r in cases if r['case'] in ['reveal-e','reveal-f']]
    chosen=selection.get('method','hybrid')
    passes=bool(heldout) and all(r['side']==selection['side'] and next(t for t in r['timings'] if t['method']==chosen)['eligible'] for r in heldout)
    summary=dict(selection=selection,heldout_quality_pass=passes,calibration=calibration,validation=validation,source_sha256=hashes(),analysis_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest())
    (OUT/'summary.json').write_text(json.dumps(summary,indent=2));rows='';pictures='';font=ImageFont.truetype('/System/Library/Fonts/Supplemental/Arial.ttf',19)
    for r in cases:
        for t in r['timings']:
            ratios=' / '.join(f'{t["ratios"][k]:.3f}' for k in METRICS)
            rows+=f'<tr><td>{r["label"]}/{r["case"]}</td><td>{r["side"]}</td><td>{t["method"]}</td><td>{t["gpu_ms"]:.3f}</td><td>{t["cpu_ms"]:.3f}</td><td>{ratios}</td><td>{t["direct64_fraction"]:.1%}</td><td>{t["eligible"]}</td></tr>'
        if r['side']!=selection['side']:continue
        loc=OUT/r['label']/r['case'];policy=chosen if r['reused'] else 'hybrid';canvas=Image.new('RGB',(1920,365),'#13191f');draw=ImageDraw.Draw(canvas)
        for i,(file,title) in enumerate([(f'{r["side"]}-direct64.png','Dinâmico64'),(f'{r["side"]}-{policy}.png','Bake selecionado' if r['reused'] else 'Fallback dinâmico64'),('reference.png','Referência4096'),(f'{r["side"]}-'+('compact20' if r['reused'] else 'stale')+'.png','Dinâmico20 anterior' if r['reused'] else 'Bake desatualizado — controle')]):
            canvas.paste(Image.open(loc/file).convert('RGB'),(i*480,45));draw.text((i*480+8,10),title,font=font,fill='white')
        name=f'{r["label"]}-{r["case"]}.png';canvas.save(OUT/name);pictures+=f'<h2>{html.escape(r["label"]+"/"+r["case"])}</h2><img src="{name}">'
    (OUT/'review.html').write_text(f'''<!doctype html><meta charset="utf-8"><title>Ignis — pré-cálculo</title><style>body{{background:#13191f;color:#e5edf4;font:16px system-ui;max-width:1920px;margin:24px auto}}td,th{{padding:8px;border-bottom:1px solid #40505d}}img{{width:100%}}</style><h1>Pré-cálculo de iluminação por superfície</h1><p>Política selecionada: {chosen}, resolução {selection['side']}. Validação reservada: {passes}. Qualidade relativa ao estimador dinâmico64; quatro sementes, média de duas referências4096. Erros: RGB global / faixa da parede / p95 / pior região de uma partição8×8. CPU preparação/submissão; GPU raster+mapa primário+consulta/correção,64amostras. Bake/load/upload separados no <a href="summary.json">JSON</a>. Mudanças de direção/geometria/material invalidam o bake inteiro; híbrido deve igualar64 exatamente. Consulta do bake desatualizado é controle incorreto. Sem FPS de jogo, movimento contínuo, GI produção ou mínimo universal inferido.</p><table><tr><th>Cena</th><th>Resolução</th><th>Método</th><th>GPU ms</th><th>CPU ms</th><th>Razões de erro</th><th>Fração64</th><th>Qualidade elegível</th></tr>{rows}</table>{pictures}''')
    return summary

def freeze_final(calibration):
    candidates=[]
    for side in [16,32,64]:
        rows=[next(r for r in x['reports'] if r['side']==side) for x in calibration.values()]
        for method in ['baked','hybrid']:
            chosen=[next(t for t in r['timings'] if t['method']==method) for r in rows];base=[next(t for t in r['timings'] if t['method']=='direct64') for r in rows]
            candidates.append(dict(side=side,method=method,eligible=all(t['eligible'] for t in chosen),worst_gpu_ratio=max(t['gpu_ms']/b['gpu_ms'] for t,b in zip(chosen,base)),quality=[t['ratios'] for t in chosen]))
    eligible=[c for c in candidates if c['eligible'] and c['worst_gpu_ratio']<1];assert eligible
    best=min(c['worst_gpu_ratio'] for c in eligible)
    # Predeclared practical timing tie: prefer smaller atlas within5%of fastest ratio.
    tied=[c for c in eligible if c['worst_gpu_ratio']<=best*1.05];winner=min(tied,key=lambda c:(c['side'],c['worst_gpu_ratio']))
    selection=dict(**winner,candidates=candidates,source_sha256=hashes(),criteria=METRICS,quality_ratio_max=1.,timing_tie=1.05,heldout=['reveal-e','reveal-f'],phase='second selection; earlier camera/scaling states exploratory')
    (OUT/'selection.json').write_text(json.dumps(selection,indent=2));return selection
