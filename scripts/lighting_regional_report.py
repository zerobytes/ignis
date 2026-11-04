"""Independent dependency classifier and full/partial bake experiment report."""
from pathlib import Path
import hashlib
import json
import numpy as np
from PIL import Image,ImageDraw,ImageFont
from lighting_bake_report import hash32,direct_cpu
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'artifacts/lighting-regional'
SOURCE=['examples/lighting_bounce.rs','examples/lighting_bounce/bake.rs','examples/lighting_bounce/bake.wgsl','examples/lighting_bounce/transport.wgsl','examples/lighting_bounce/regional.rs','examples/lighting_bounce/regional.wgsl']
def hashes():return {f:hashlib.sha256((ROOT/f).read_bytes()).hexdigest() for f in SOURCE}
def boxes(scene):
    p=np.array([e['transform']['position'] for e in scene['entities']]);s=np.array([e['transform']['size'] for e in scene['entities']]);return p-s/2,p+s/2

def dirty_cpu(meta,scene,event,side):
    count=len(meta)//48;f=np.frombuffer(meta,dtype='<f4').reshape(-1,12).astype(float);u=np.frombuffer(meta,dtype='<u4').reshape(-1,12);kind=event['kind'];changed=event['object']
    if kind==0:return np.zeros(count,dtype=bool)
    if kind==3:return np.ones(count,dtype=bool)
    bit=1<<changed;dirty=(u[:,3]&bit)!=0
    if kind==1:return dirty
    idx=np.arange(count);obj=idx//(6*side*side);face=(idx//(side*side))%6;axis=face//2;n=np.eye(3)[axis]*np.where(face%2==0,1.,-1.)[:,None]
    lo,hi=boxes(scene);v=np.zeros((count,3));uv=np.stack([(idx%side+.5)/side,((idx%(side*side))//side+.5)/side],axis=1)
    for a in range(3):
        mask=axis==a;other=[k for k in range(3) if k!=a];v[np.ix_(mask,other)]=uv[mask];v[mask,a]=np.where(face[mask]%2==0,1.,0.)
    p=lo[obj]+(hi[obj]-lo[obj])*v;far=np.where(n>=0,hi[changed],lo[changed]);primary=np.sum((far-p)*n,axis=1)>=-.001
    sun=-np.array(scene['light']['direction']);sun/=np.linalg.norm(sun);t=np.cross([1.,0,0] if abs(sun[1])>.9 else [0.,1,0],sun);t/=np.linalg.norm(t);b=np.cross(sun,t)
    l=f[:,:3]-.0021;h=f[:,4:7]+.0021;valid=np.all(f[:,:3]<=f[:,4:7],axis=1)
    def proj(lo,hi,a):
        c=(lo+hi)/2;r=np.sum((hi-lo)/2*np.abs(a),axis=-1);m=np.sum(c*a,axis=-1);return m-r,m+r
    shadow=valid.copy()
    for a in [t,b]:
        lower,upper=proj(l,h,a);bl,bh=proj(lo[changed],hi[changed],a);shadow&=(upper>=bl-.001)&(bh>=lower-.001)
    lower,_=proj(l,h,sun);_,bh=proj(lo[changed],hi[changed],sun);shadow&=lower<=bh+.001
    return dirty|(obj==changed)|(((u[:,7]|u[:,8])&bit)!=0)|primary|shadow

def dependency_sample(scene,index,side,seed):
    lo,hi=boxes(scene);obj=index//(6*side*side);face=(index//(side*side))%6;pixel=index%(side*side);a=face//2
    v=np.zeros(3);v[[k for k in range(3) if k!=a]]=[(pixel%side+.5)/side,(pixel//side+.5)/side];v[a]=1 if face%2==0 else 0;p=lo[obj]+(hi[obj]-lo[obj])*v;n=np.zeros(3);n[a]=1 if face%2==0 else -1
    cover=sum(1<<j for j in range(len(lo)) if j!=obj and np.all(p>lo[j]-.00001) and np.all(p<hi[j]+.00001))
    if cover:return (0,0,cover,None,None),p,n
    sun=-np.array(scene['light']['direction']);sun/=np.linalg.norm(sun);t=np.cross([1.,0,0] if abs(n[1])>.9 else [0.,1,0],n);t/=np.linalg.norm(t)
    rng=hash32(index+seed*91237+519983);uv=[]
    for j in range(4096):
        rng=hash32(rng+0x9e3779b9);x=(rng>>8)/16777216;rng=hash32(rng+0x9e3779b9);y=(rng>>8)/16777216;uv.append([(j%64+x)/64,(j//64+y)/64])
    uv=np.array(uv);d=t[None,:]*np.sqrt(uv[:,0,None])*np.cos(2*np.pi*uv[:,1,None])+np.cross(n,t)[None,:]*np.sqrt(uv[:,0,None])*np.sin(2*np.pi*uv[:,1,None])+n[None,:]*np.sqrt(1-uv[:,0,None]);origin=p+n*.002
    def slab(p,d):
        with np.errstate(divide='ignore',invalid='ignore'):
            x=(lo[None,:,:]-p[:,None,:])/d[:,None,:];y=(hi[None,:,:]-p[:,None,:])/d[:,None,:]
        return np.minimum(x,y).max(axis=2),np.maximum(x,y).min(axis=2)
    en,ex=slab(np.broadcast_to(origin,d.shape),d);dist=np.where((en>.00001)&(ex>=en),en,np.inf);hit=np.argmin(dist,axis=1);valid=np.isfinite(dist[np.arange(4096),hit]);ids=hit[valid];points=origin+d[valid]*dist[np.arange(4096)[valid],ids,None]
    hitmask=int(np.bitwise_or.reduce(np.left_shift(np.uint32(1),ids.astype(np.uint32)),initial=np.uint32(0)))
    q=(points-(lo[ids]+hi[ids])/2)/((hi[ids]-lo[ids])/2);axes=np.full(len(q),2);axes[(abs(q[:,1])>abs(q[:,2]))]=1;axes[(abs(q[:,0])>abs(q[:,1]))&(abs(q[:,0])>abs(q[:,2]))]=0
    normal=np.zeros_like(points);normal[np.arange(len(q)),axes]=np.sign(q[np.arange(len(q)),axes]);lit=np.sum(normal*sun,axis=1)>0;points=points[lit];normal=normal[lit]
    if not len(points):return (hitmask,0,0,None,None),p,n
    en,ex=slab(points+normal*.002,np.broadcast_to(sun,points.shape));block=(ex>=np.maximum(en,0))&(ex>.00001);first=np.argmax(block,axis=1);first=first[block.any(axis=1)];shadow=int(np.bitwise_or.reduce(np.left_shift(np.uint32(1),first.astype(np.uint32)),initial=np.uint32(0)))
    return (hitmask,shadow,0,points.min(axis=0),points.max(axis=0)),p,n

def scene_report(label):
    path=OUT/label;d=json.loads((path/'results.json').read_text());manifest=json.loads((path/'manifest.json').read_text());assert manifest['source_sha256']==hashes();side=d['side'];n=d['texels'];meta=(path/'initial-deps.bin').read_bytes();old=np.fromfile(path/'initial-atlas.bin',dtype='<f4').reshape(-1,4);events=[];cpu_samples=[]
    first_scene=json.loads((path/'00-unchanged'/'scene.json').read_text());mf=np.frombuffer(meta,dtype='<f4').reshape(-1,12);mu=np.frombuffer(meta,dtype='<u4').reshape(-1,12)
    for face in range(6):
        ids=np.flatnonzero(((np.arange(n)//(side*side))%6==face)&(old[:,3]>0));index=int(ids[np.argmax(old[ids,:3].sum(axis=1))]);support,p,normal=dependency_sample(first_scene,index,side,d['seed']);h,s,c,lo,hi=support
        assert tuple(map(int,mu[index,[3,7,8]]))==(h,s,c),(label,'dependency mask mismatch',index)
        bound_error=0. if lo is None else float(max(abs(lo-mf[index,:3]).max(),abs(hi-mf[index,4:7]).max()));assert bound_error<3e-4,(label,'bounds',index,bound_error)
        recv=np.zeros((1,12));recv[0,:3]=p;recv[0,4:7]=normal;recv[0,8:11]=1;expected=direct_cpu(recv,first_scene,0,64,64,d['seed'],519983,index,True);error=float(abs(expected-old[index,:3]).max());assert error<5e-6
        cpu_samples.append(dict(index=index,rgb_error=error,bounds_error=bound_error))
    for e in d['events']:
        loc=path/f'{e["step"]:02}-{e["label"]}';scene=json.loads((loc/'scene.json').read_text());flags=np.fromfile(loc/'dirty.bin',dtype='<u4');expected=dirty_cpu(meta,scene,e,side);assert np.array_equal(flags!=0,expected),(label,e['label'],'CPU classifier mismatch',int(np.sum((flags!=0)!=expected)))
        work=np.fromfile(loc/'work.bin',dtype='<u4');count=int(work[0]);assert count==e['dirty'] and work[1]==0 and np.array_equal(np.sort(work[2:2+count]),np.flatnonzero(flags))
        dispatch=np.fromfile(loc/'dispatch.bin',dtype='<u4');assert np.array_equal(dispatch,[(count+63)//64,1,1,0,1,1])
        current=np.fromfile(loc/'atlas.bin',dtype='<f4').reshape(-1,4);changed=np.any(old!=current,axis=1);assert int(changed.sum())==e['changed_rgb'] and np.all((~changed)|(flags!=0))
        assert all(e[k] for k in ['atlas_exact','dependencies_exact','published_png_exact','pending_composition_exact','restored_actual_partial_output'])
        recv=np.fromfile(loc/'receivers.bin',dtype='<f4').reshape(-1,12);live=recv[:,3]>0;pending=np.fromfile(loc/'pending.rgba32f',dtype='<f4').reshape(-1,4);pub=np.fromfile(loc/'published.rgba32f',dtype='<f4').reshape(-1,4);direct=np.fromfile(loc/'direct.rgba32f',dtype='<f4').reshape(-1,4)
        assert all(np.isfinite(v).all() and (v[:,:3]>=0).all() and (v[~live,:3]==0).all() for v in [pending,pub,direct]);fallback=pending[:,3]==2;assert np.array_equal(pending[fallback,:3],direct[fallback,:3]) and np.array_equal(pending[~fallback,:3],pub[~fallback,:3])
        # Independent all-pixel bilinear footprint test, including zero-weight taps.
        ids=np.flatnonzero(live);obj=recv[ids,3].astype(int)-1;norm=recv[ids,4:7];axis=np.argmax(abs(norm),axis=1);face=axis*2+(norm[np.arange(len(ids)),axis]<0);lo,hi=boxes(scene);v=np.clip((recv[ids,:3]-lo[obj])/(hi[obj]-lo[obj]),0,1);uv=np.stack([np.where(axis==0,v[:,1],v[:,0]),np.where(axis==2,v[:,1],v[:,2])],axis=1);g=np.clip(uv*side-.5,0,side-1);cell=np.minimum(np.floor(g).astype(int),side-2);dirty=np.zeros(len(ids),bool)
        for y in [0,1]:
            for x in [0,1]:dirty|=flags[(obj*6+face)*side*side+(cell[:,1]+y)*side+cell[:,0]+x]!=0
        assert np.all(fallback[ids][dirty]);expected_fraction=float(fallback[live].mean())
        if e['kind']==0:assert e['dirty']==0 and e['changed_rgb']==0
        if e['kind']==3:assert e['dirty']==n
        times={}
        for method in ['partial','full-dependencies','full-plain']:
            rows=[r for r in e['rows'] if r['method']==method];assert len(rows)==16
            times[method]={k:float(np.median([r[k] for r in rows])) for k in ['gpu_ms','cpu_submit_ms','publication_gpu_ms']}
        render={}
        for method in ['baked','direct64']:
            rows=[r for r in e['render_rows'] if r['method']==method];assert len(rows)==16;render[method]={k:float(np.median([r[k] for r in rows])) for k in ['gpu_ms','cpu_ms']}
        events.append(dict(label=e['label'],step=e['step'],kind=e['kind'],dirty_fraction=e['dirty']/n,changed_atlas_fraction=e['changed_rgb']/n,pending_direct64_fraction=expected_fraction,visible_changed_object_pixels=e['visible_changed_object_pixels'],times=times,render=render,partial_vs_plain=times['partial']['gpu_ms']/times['full-plain']['gpu_ms'],partial_vs_deps=times['partial']['gpu_ms']/times['full-dependencies']['gpu_ms']))
        old=current;meta=(loc/'deps.bin').read_bytes()
    return dict(label=label,adapter=d['adapter'],texels=n,dependency_bytes=d['dependency_bytes'],dirty_bytes=d['dirty_bytes'],atlas_bytes=d['atlas_bytes'],initial_dependency_build_gpu_ms=d['initial_dependency_build_gpu_ms'],cpu_samples=cpu_samples,events=events)

def write_report(reports):
    summary=dict(reports=reports,source_sha256=hashes(),analysis_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest());(OUT/'summary.json').write_text(json.dumps(summary,indent=2));rows='';pictures='';font=ImageFont.truetype('/System/Library/Fonts/Supplemental/Arial.ttf',18)
    for label,d in reports.items():
        for e in d['events']:
            visible=e['visible_changed_object_pixels'] if e['kind'] in [1,2] else '—'
            t=e['times'];rows+=f'<tr><td>{label}/{e["label"]}</td><td>{e["dirty_fraction"]:.1%}</td><td>{e["changed_atlas_fraction"]:.1%}</td><td>{t["full-plain"]["gpu_ms"]:.3f}</td><td>{t["full-dependencies"]["gpu_ms"]:.3f}</td><td>{t["partial"]["gpu_ms"]:.3f}</td><td>{e["pending_direct64_fraction"]:.1%}</td><td>{visible}</td></tr>'
            if e['step'] not in [1,3,8,10]:continue
            path=OUT/label/f'{e["step"]:02}-{e["label"]}';canvas=Image.new('RGB',(1440,365),'#13191f');draw=ImageDraw.Draw(canvas)
            for i,(file,title) in enumerate([('pending.png','Pendente: cache limpo +64'),('published.png','Publicado = bake completo'),('direct.png','Dinâmico64')]):canvas.paste(Image.open(path/file).convert('RGB'),(i*480,45));draw.text((i*480+8,10),title,font=font,fill='white')
            name=f'{label}-{e["label"]}.png';canvas.save(OUT/name);pictures+=f'<h2>{label}/{e["label"]}</h2><img src="{name}">'
    (OUT/'review.html').write_text(f'''<!doctype html><meta charset="utf-8"><title>Ignis — reparo regional</title><style>body{{background:#13191f;color:#e5edf4;font:16px system-ui;max-width:1500px;margin:24px auto}}td,th{{padding:8px;border-bottom:1px solid #40505d}}img{{width:100%}}</style><h1>Reparo regional de bake amostrado</h1><p>24transições sequenciais. Atlas e dependências atualizados iguais byte a byte à reconstrução total; RGB/PNG publicado idêntico. Enquanto pendente, suporte sujo usa64, suporte limpo reutiliza o bake. Validação de todos os texels inclui os fora da câmera. GPU de atualização inclui classificação/compactação/reparo/publicação,16amostras/método; restauração do cache de entrada e upload de parâmetros/geometria excluídos. Frame após reparo e CPU no <a href="summary.json">JSON</a>. Não é FPS, scheduler assíncrono ou GI produção; campo4096amostras,32por face, caixas opacas, um rebote.</p><table><tr><th>Cena/evento</th><th>Texels sujos</th><th>Atlas mudou (RGB/validade)</th><th>Full simples GPU ms</th><th>Full+deps GPU ms</th><th>Regional GPU ms</th><th>Pixels64 pendentes</th><th>Pixels objeto alterado</th></tr>{rows}</table>{pictures}''');return summary
