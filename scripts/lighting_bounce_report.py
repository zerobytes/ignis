# /// script
# requires-python = ">=3.11"
# dependencies = ["numpy>=2,<3", "pillow>=11,<13"]
# ///
"""Independent checks and report for one diffuse indirect bounce, not full GI."""
from pathlib import Path
import hashlib
import html
import json
import numpy as np
from PIL import Image, ImageDraw, ImageFont

ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/"artifacts/lighting-bounce"
NAMES=["Somente direta", "RGB · 16 raios", "RGB · 64 raios", "Probes 1 m", "Probes 1 m + visibilidade", "Probes 0,5 m", "Probes 0,5 m + visibilidade"]
FILES=["native", "strat16", "strat64", "coarse", "coarse-visible", "fine", "fine-visible"]
reports={}

def hashed(x):
    x &= 0xffffffff; x=((x^(x>>16))*0x7feb352d)&0xffffffff
    x=((x^(x>>15))*0x846ca68b)&0xffffffff
    return x^(x>>16)
def pair(seed):
    seed=hashed(seed+0x9e3779b9);a=(seed>>8)/16777216
    seed=hashed(seed+0x9e3779b9);b=(seed>>8)/16777216
    return seed,(a,b)

for label in ["open","thin-wall"]:
    path=OUT/label
    data=json.loads((path/"results.json").read_text())
    for flag in ["native_profile_identity","passthrough_identity","zero_sun_direct","zero_sun_rebuilt_probe","channel_isolation","monotonic_gpu_timestamps"]:
        assert data[flag],(label,flag)
    scene=json.loads((path/"scene.json").read_text())
    w,h=data["width"],data["height"]
    recv=np.fromfile(path/"receivers.bin",dtype="<f4").reshape(-1,12).astype(float)
    active=recv[:,3]>0
    refs=[np.fromfile(path/f"reference-{k}.rgba32f",dtype="<f4").reshape(-1,4).astype(float) for k in [7,8]]
    ref=(refs[0][:,:3]+refs[1][:,:3])*.5
    for v in refs:
        assert np.all(np.isfinite(v)) and np.all(v[:,:3]>=0) and np.all(v[~active,:3]==0)
    assert ref[active].mean()>1e-5, "No one-bounce contribution"
    rho=recv[:,8:11]
    direct=np.array(scene["light"]["direct_color"])*scene["light"]["direct_intensity"]
    sun=-np.array(scene["light"]["direction"]);sun/=np.linalg.norm(sun)
    boxes=[(np.array(e["transform"]["position"]),np.array(e["transform"]["size"])/2,np.array(e["color"])) for e in scene["entities"]]
    lo=np.array([c-half for c,half,_ in boxes]);hi=np.array([c+half for c,half,_ in boxes]);colors=np.array([c for _,_,c in boxes])
    # Scalar CPU transport uses all boxes, its own slab/normal/shadow evaluation.
    def interval(p,d):
        with np.errstate(divide="ignore",invalid="ignore"):
            a=(lo-p)/d;b=(hi-p)/d
        return np.minimum(a,b).max(axis=1),np.maximum(a,b).min(axis=1)
    def endpoint(p,d):
        en,ex=interval(p,d)
        ok=(en>.00001)&(ex>=en)
        if not ok.any():
            return np.zeros(3)
        j=int(np.argmin(np.where(ok,en,np.inf)))
        hit=p+d*en[j];c,half,color=boxes[j]
        local=(hit-c)/half
        # Match tie handling of the box geometry's face convention.
        axis=2
        if abs(local[0])>abs(local[1]) and abs(local[0])>abs(local[2]):
            axis=0
        elif abs(local[1])>abs(local[2]):
            axis=1
        n=np.zeros(3);n[axis]=np.sign(local[axis])
        cosine=max(float(n@sun),0.)
        if cosine==0:
            return np.zeros(3)
        a,b=interval(hit+n*.002,sun)
        if np.any((b>=np.maximum(a,0))&(b>.00001)):
            return np.zeros(3)
        return color*direct*cosine
    def estimate_cpu(i,side,frame):
        p=recv[i,:3];n=recv[i,4:7]
        t=np.cross([1.,0.,0.] if abs(n[1])>.9 else [0.,1.,0.],n);t/=np.linalg.norm(t)
        seed=hashed(i+frame*91237+9923);total=np.zeros(3)
        for j in range(side*side):
            seed,v=pair(seed);a=(j%side+v[0])/side;b=(j//side+v[1])/side
            d=t*np.sqrt(a)*np.cos(2*np.pi*b)+np.cross(n,t)*np.sqrt(a)*np.sin(2*np.pi*b)+n*np.sqrt(1-a)
            total+=endpoint(p+n*.002,d)
        return rho[i]*total/(side*side)
    rng=np.random.default_rng(208)
    indices=rng.choice(np.flatnonzero(active),64,replace=False)
    worst=0.
    for mode,side in [(1,4),(2,8)]:
        actual=np.fromfile(path/f"quality-{mode}-0.rgba32f",dtype="<f4").reshape(-1,4)
        for i in indices:
            i=int(i);expected=estimate_cpu(i,side,7)
            error=float(np.max(np.abs(expected-actual[i,:3])))
            worst=max(worst,error)
            assert error<2e-5,(label,mode,i,error,expected,actual[i,:3])
    probe_worst=0.;probe_checks=0;occupancy=[];build_worst=0.;build_checks=0
    for fine in [False,True]:
        dims=np.array([21,9,21] if fine else [11,5,11]);spacing=.5 if fine else 1.
        field=np.fromfile(path/f"probe-field-{'fine' if fine else 'coarse'}.rgba32f",dtype="<f4").reshape(-1,6,4).astype(float)
        assert np.all(np.isfinite(field)) and np.all(field[:,:,:3]>=0)
        grid_indices=np.arange(len(field))
        coords=np.stack([grid_indices%dims[0],(grid_indices//dims[0])%dims[1],grid_indices//(dims[0]*dims[1])],axis=1)
        positions=np.array([-5.,.125,-5.])+coords*spacing
        inside=np.any(np.all(positions[:,None,:]>lo[None,:,:]-.0001,axis=2)&np.all(positions[:,None,:]<hi[None,:,:]+.0001,axis=2),axis=1)
        assert np.all(field[:,:,3]==(~inside)[:,None])
        occupancy.append(dict(nodes=len(field),invalid=int(inside.sum()),bytes=len(field)*96))
        normals_for_field=np.array([[1.,0.,0.],[-1.,0.,0.],[0.,1.,0.],[0.,-1.,0.],[0.,0.,1.],[0.,0.,-1.]])
        for node in rng.choice(np.flatnonzero(~inside),4,replace=False):
            node=int(node)
            for kind,n in enumerate(normals_for_field):
                t=np.cross([1.,0.,0.] if abs(n[1])>.9 else [0.,1.,0.],n);t/=np.linalg.norm(t)
                seed=hashed(node*737+kind*223+3*91237+113);total=np.zeros(3)
                for j in range(256):
                    seed,v=pair(seed);a=(j%16+v[0])/16;b=(j//16+v[1])/16
                    direction=t*np.sqrt(a)*np.cos(2*np.pi*b)+np.cross(n,t)*np.sqrt(a)*np.sin(2*np.pi*b)+n*np.sqrt(1-a)
                    total+=endpoint(positions[node],direction)
                error=float(np.abs(total/256-field[node,kind,:3]).max())
                build_worst=max(build_worst,error);build_checks+=1
                assert error<2e-5,(label,"probe build",fine,node,kind,error)
        for mode in ([5,6] if fine else [3,4]):
            actual=np.fromfile(path/f"quality-{mode}-3.rgba32f",dtype="<f4").reshape(-1,4)
            fallback_indices=np.flatnonzero(active & (actual[:,3]>0))[:8]
            selected=np.unique(np.concatenate([indices,fallback_indices]))
            for i in selected:
                i=int(i);n=recv[i,4:7];origin=recv[i,:3]+n*.002
                axis=int(np.argmax(np.abs(n)));kind=axis*2+int(n[axis]<0)
                g=np.clip((origin-np.array([-5.,.125,-5.]))/spacing,0,dims-1)
                cell=np.minimum(np.floor(g).astype(int),dims-2);f=g-cell;total=np.zeros(3);weight=0.
                for j in range(8):
                    bits=np.array([j&1,(j>>1)&1,(j>>2)&1]);node=cell+bits
                    index=int(node[0]+dims[0]*(node[1]+dims[1]*node[2]));value=field[index,kind]
                    if value[3]==0:
                        continue
                    factors=np.where(bits,f,1-f);wgt=float(factors.prod())
                    if wgt<=0:
                        continue
                    if mode in [4,6]:
                        delta=positions[index]-origin;distance=np.linalg.norm(delta)
                        if distance>.0001:
                            en,ex=interval(origin,delta/distance)
                            if np.any((ex>=np.maximum(en,0))&(ex>.00001)&(en<max(0,distance-.001))):
                                continue
                    total+=wgt*value[:3];weight+=wgt
                expected=rho[i]*total/weight if weight>.00001 else estimate_cpu(i,4,55)
                error=float(np.abs(expected-actual[i,:3]).max());probe_worst=max(probe_worst,error);probe_checks+=1
                assert error<2e-5,(label,"probe",mode,i,error,expected,actual[i,:3])
                assert actual[i,3]==float(weight<=.00001)
    # Constant incident-field response, plus six synthetic ±axis receivers.
    const=np.fromfile(path/"constant-field.rgba32f",dtype="<f4").reshape(-1,4).astype(float)
    normals=recv[:,4:7];axis=np.argmax(np.abs(normals),axis=1)
    kinds=axis*2+(np.take_along_axis(normals,axis[:,None],axis=1)[:,0]<0)
    expected=np.zeros_like(const[:,:3])
    expected[active]=rho[active]*np.stack([.01*(kinds[active]+1),np.full(active.sum(),.02),np.full(active.sum(),.03)],axis=1)
    expected[:6]=np.array([.2,.3,.4])*np.stack([np.arange(1,7)*.01,np.full(6,.02),np.full(6,.03)],axis=1)
    constant_error=float(np.abs(expected-const[:,:3]).max())
    assert constant_error<2e-7,(label,"constant",constant_error)
    max_rho=colors.max(axis=0)
    bound=rho*max_rho*direct
    band=active & (np.abs(recv[:,0]-.27)<.65)&(recv[:,1]<2.)&(recv[:,2]>-1.1)&(recv[:,2]<3.1)
    # Frame RGB uses quantized native direct, shared by all candidates.
    native=np.asarray(Image.open(path/"native.png")).astype(float)[:,:,:3].reshape(-1,3)/255
    native=np.where(native<=.04045,native/12.92,((native+.055)/1.055)**2.4)
    timings=[]
    for mode in range(7):
        rows=[r for r in data["rows"] if r["mode"]==mode]
        assert len(rows)==128
        gpu=np.array([r["raster_gpu_ms"]+r["compute_gpu_ms"] for r in rows])
        cpu=np.array([r["prepare_cpu_ms"]+r["raster_cpu_ms"]+r["compute_cpu_ms"] for r in rows])
        maps=np.zeros((4,w*h,4)) if mode==0 else np.stack([np.fromfile(path/f"quality-{mode}-{seed}.rgba32f",dtype="<f4").reshape(-1,4) for seed in range(4)]).astype(float)
        assert np.all(np.isfinite(maps)) and np.all(maps[:,:,:3]>=0) and np.all(maps[:,~active,:3]==0)
        assert np.all(maps[:,:,:3]<=bound[None,:,:]+2e-5), (label,mode,"energy bound")
        assert np.all((maps[:,:,3]==0)|(maps[:,:,3]==1))
        error=maps[:,:,:3]-ref[None,:,:]
        per_seed=np.mean(error[:,active,:]**2,axis=(1,2))
        build=[r["gpu_ms"] for r in data["builds"] if r["fine"]==(mode>=5) and r.get("calibration",False)] if mode>=3 else []
        cold=[r["gpu_ms"] for r in data["builds"] if mode>=3 and r["fine"]==(mode>=5) and r["seed"]==0]
        interleaved=[r["gpu_ms"] for r in data["builds"] if mode>=3 and r["fine"]==(mode>=5) and not r.get("calibration",False) and r["seed"]>0]
        timings.append(dict(build_first_gpu_ms=cold[0] if cold else 0.,build_interleaved_gpu_median_ms=float(np.median(interleaved)) if interleaved else 0.,build_interleaved_gpu_range_ms=[min(interleaved),max(interleaved)] if interleaved else [0.,0.],mode=mode,name=NAMES[mode],cpu_ms=float(np.median(cpu)),gpu_ms=float(np.median(gpu)),gpu_p95_ms=float(np.percentile(gpu,95)),compute_gpu_ms=float(np.median([r["compute_gpu_ms"] for r in rows])),rgb_mse=float(per_seed.mean()),rgb_mse_per_seed=per_seed.tolist(),signed_rgb_error=np.mean(error[:,active,:],axis=(0,1)).tolist(),seed_rgb_variance=float(np.var(maps[:,active,:3],axis=0,ddof=1).mean()),near_partition_mse=float(np.mean(error[:,band,:]**2)) if band.any() else None,near_partition_positive_rgb_excess=float(np.maximum(error[:,band,:],0).mean()) if band.any() else None,fallback_fraction=float(maps[:,active,3].mean()),display_clipped_pixel_fraction=float(np.any(native[None,:,:]+maps[:,:,:3]>1,axis=2)[:,active].mean()),build_gpu_median_ms=float(np.median(build)) if build else 0.,build_gpu_p95_ms=float(np.percentile(build,95)) if build else 0.,build_gpu_range_ms=[float(min(build)),float(max(build))] if build else [0.,0.]))
    for t in timings:
        saving=timings[2]["compute_gpu_ms"]-t["compute_gpu_ms"]
        t["build_break_even_frames_vs64"]=t["build_gpu_median_ms"]/saving if t["mode"]>=3 and saving>0 else None
        t["build_break_even_range_vs64"]=[value/saving for value in t["build_gpu_range_ms"]] if t["mode"]>=3 and saving>0 else None
        t["modeled_gpu_total_if_rebuilt_every_frame_ms"]=t["gpu_ms"]+t["build_gpu_median_ms"]
        t["modeled_gpu_total_with_interleaved_rebuild_ms"]=t["gpu_ms"]+t["build_interleaved_gpu_median_ms"]
        t["break_even_frames_interleaved_vs64"]=t["build_interleaved_gpu_median_ms"]/saving if t["mode"]>=3 and saving>0 else None
    reports[label]=dict(visible_receivers=int(active.sum()),cpu_transport_checks=128,cpu_probe_build_lobe_checks=build_checks,cpu_probe_build_max_rgb_error=build_worst,cpu_probe_checks=probe_checks,cpu_probe_max_rgb_error=probe_worst,probe_occupancy=occupancy,cpu_max_rgb_error=worst,constant_field_max_rgb_error=constant_error,reference_pair_rgb_mse=float(np.mean((refs[0][active,:3]-refs[1][active,:3])**2)),estimated_mean_reference_noise_mse=float(np.mean((refs[0][active,:3]-refs[1][active,:3])**2)/4),probe_bytes=data["probe_bytes"],builds=data["builds"],timings=timings)

font=ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf",20)
small=ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf",17)
for label in ["open","thin-wall"]:
    canvas=Image.new("RGB",(1440,760),"#13191f");draw=ImageDraw.Draw(canvas)
    ids=[0,2,7,3,4,6]
    for k,mode in enumerate(ids):
        x=(k%3)*480;y=(k//3)*380
        path=OUT/label/("native.png" if mode==0 else "7-reference-a.png" if mode==7 else f"{mode}-{FILES[mode]}.png")
        canvas.paste(Image.open(path).convert("RGB"),(x,y+60))
        title="Referência · 4096" if mode==7 else NAMES[mode]
        draw.text((x+10,y+7),title,font=font,fill="white")
        if mode!=7:
            t=reports[label]["timings"][mode]
            draw.text((x+10,y+34),f'GPU {t["gpu_ms"]:.3f} ms · MSE {t["rgb_mse"]:.6f}',font=small,fill="#bbd8e6")
    canvas.save(OUT/f"{label}-comparison.png")
    # Difference map is a diagnostic, never a replacement for native appearance.
    values=[]
    ref=(np.fromfile(OUT/label/"reference-7.rgba32f",dtype="<f4").reshape(-1,4)[:,:3]+np.fromfile(OUT/label/"reference-8.rgba32f",dtype="<f4").reshape(-1,4)[:,:3])/2
    for mode in [3,4,6]:
        v=np.fromfile(OUT/label/f"quality-{mode}-0.rgba32f",dtype="<f4").reshape(-1,4)[:,:3]
        values.append(np.uint8(np.clip(np.abs(v-ref).reshape(h,w,3)*12,0,1)*255))
    Image.fromarray(np.concatenate(values,axis=1)).save(OUT/f"{label}-error-x12.png")
hashes={str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in [ROOT/"examples/lighting_bounce.rs",ROOT/"examples/lighting_bounce/transport.wgsl",Path(__file__)]}
(OUT/"summary.json").write_text(json.dumps(dict(reports=reports,source_sha256=hashes,executable_sha256=hashlib.sha256((ROOT/"target/release/examples/lighting_bounce").read_bytes()).hexdigest()),indent=2))
rows=""
for label,r in reports.items():
    for t in r["timings"]:
        build="—" if t["mode"]<3 else f'{t["build_gpu_median_ms"]:.3f} / {t["build_interleaved_gpu_median_ms"]:.3f}'
        rows+=f'<tr><td>{label}</td><td>{html.escape(t["name"])}</td><td>{t["cpu_ms"]:.3f}</td><td>{t["gpu_ms"]:.3f}</td><td>{t["compute_gpu_ms"]:.3f}</td><td>{t["rgb_mse"]:.6f}</td><td>{build}</td><td>{t["fallback_fraction"]:.1%}</td></tr>'
(OUT/"review.html").write_text(f'''<!doctype html><meta charset="utf-8"><title>Ignis — RGB um rebote</title><style>body{{background:#13191f;color:#e5edf4;font:17px system-ui;max-width:1500px;margin:32px auto;padding:16px}}td,th{{padding:10px;border-bottom:1px solid #40505d;text-align:left}}img{{width:100%}}</style>
<h1>RGB indireto: avaliação direta × probes espaciais</h1><p>Mesmo transporte sol→superfície difusa→receptor→câmera, mesmas caixas e visibilidade sem cutoffAO4m. Diretamente iluminado usa renderer nativo com ambiente0; RGB indireto analítico é composto sobre sua imagem linearizada. O termo secundário usa sombra analítica em todos os métodos. Sem emissão, ambiente, especular, transmissão ou rebotes adicionais.</p>
<img src="open-comparison.png"><img src="thin-wall-comparison.png">
<p>128tempos por método no M4Pro/Metal480×320; CPU preparação+submissão, GPU sombra/cena+consulta. Bloqueio/readback/PNG fora da medição. Probes estáticos: construção separada, sem atualização dinâmica. Construção é dependente do contexto:32rebuilds alternados consecutivos por grid e3rebuilds intercalados com consultas são reportados separadamente; não usar o caso mais rápido como garantia de atualização porframe. O primeiro build também consta noJSON. RGBMSE foreground contra média de2referências independentes4096; quatro seeds de query/campo. Diferença entre referências e incerteza estimada no <a href="summary.json">JSON</a>; não é verdade exata.</p>
<table><tr><th>Cena</th><th>Método</th><th>CPU ms</th><th>GPU total ms</th><th>GPU consulta ms</th><th>RGBMSE</th><th>Build repetido / intercalado ms</th><th>Fallback16</th></tr>{rows}</table>
<p>Probes guardam seis integrais para normais±XYZ: representação adequada somente às caixas deste laboratório. Grids1m e0,5m; fase vertical0,125m, consulta trilinear com clamp ao domínio, probes interiores inválidos. Visibilidade custa até8segmentos/consulta; sem peso válido usa avaliação direta16 e esse custo está incluído. Campos não foram relocalizados nem reutilizados após mudança de luz/geometria.</p>
<p>Erros absolutos RGB×12: coarse,coarse-visible,fine-visible (não são imagens finais):</p><img src="thin-wall-error-x12.png">
<p>Checks:128checks de transporte CPU por cena, mais48lobos de construção por cena, interpolação/visibilidade independente e ocupação dos campos, campo constante com seis receptores sintéticos±XYZ, energia finita/não-negativa/limitada, luz0 após reconstruir campo, isolamento de canais, pixels nativos com/sem profiler e passthrough iguais, timestamps monotônicos. Custos cold,32reconstruções alternadas por grid, p95/range e modelo break-even (soma de passes, não FPS dinâmico medido), faixa de erro porseed e erro perto da parede noJSON. Nenhum resultado é adoção em produção, solução de GI completa ou comparação com Lumen.</p>''')
print(json.dumps(reports,indent=2))
