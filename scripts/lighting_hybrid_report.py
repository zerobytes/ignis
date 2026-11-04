"""Calibration, freeze and independent quality/cost evaluation; imported by runner."""
from pathlib import Path
import hashlib
import html
import json
import numpy as np
from PIL import Image, ImageDraw, ImageFont
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/"artifacts/lighting-hybrid"
NAMES={0:"Direta nativa",1:"Estratificado16",2:"Estratificado64",6:"Probe visível",9:"Geometria0,25",10:"Geometria0,5",11:"Geometria0,75",12:"Geometria1",13:"Discordância0,02",14:"Discordância0,04",15:"Discordância0,08",16:"Discordância0,03",17:"Reconstrução16",18:"Filtro16",19:"Reconstrução32",20:"32 agrupado",21:"20 agrupado",22:"24 agrupado",23:"28 agrupado"}
FILES={0:"native",1:"strat16",2:"strat64",6:"fine-visible",9:"hybrid-025",10:"hybrid-050",11:"hybrid-075",12:"hybrid-100",13:"spread-020",14:"spread-040",15:"spread-080",16:"spread-030",17:"reconstruct16",18:"filter16",19:"reconstruct32",20:"compact32",21:"compact20",22:"compact24",23:"compact28"}
CRITERIA=dict(version=2,baseline_mode=2, metrics=["rgb_mse","near_mse","pixel_p95_mse","worst_tile_mse"], quality_ratio_max=1., calibration_quality_ratio_max=.9, selection_model="measured raster+primary+guide+GPUcompaction+sampling+filter every frame", fallback_mode=20, fresh_heldout=["validation-c","validation-d"])

def hashes():
    return {str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in [ROOT/"examples/lighting_bounce.rs",ROOT/"examples/lighting_bounce/transport.wgsl"]}
def median(rows,key):
    return float(np.median([r[key] for r in rows]))
def geometric_features(recv,scene):
    p=recv[:,:3];n=recv[:,4:7];ids=recv[:,3].astype(int)-1;risk=np.full(len(recv),1e20)
    for j,entity in enumerate(scene["entities"]):
        c=np.array(entity["transform"]["position"]);half=np.array(entity["transform"]["size"])/2
        distance=np.linalg.norm(np.maximum(np.abs(p-c)-half,0),axis=1);distance[ids==j]=1e20;risk=np.minimum(risk,distance)
        own=ids==j;edge=half-np.abs(p[own]-c);edge[np.abs(n[own])>.5]=1e20;risk[own]=np.minimum(risk[own],edge.min(axis=1))
    return np.maximum(risk,0)
def guide_cpu(recv,w,h):
    r=recv.reshape(h,w,12);live=r[:,:,3]>0;minimum=np.full((h,w),np.inf);neighbors=[]
    for dy in [-1,0,1]:
        for dx in [-1,0,1]:
            v=np.roll(r,(-dy,-dx),(0,1));valid=live&(v[:,:,3]==r[:,:,3])&(np.sum(v[:,:,4:7]*r[:,:,4:7],axis=2)>.999)
            if dy==-1:valid[0,:]=False
            if dy==1:valid[-1,:]=False
            if dx==-1:valid[:,0]=False
            if dx==1:valid[:,-1]=False
            distance=np.linalg.norm(v[:,:,:3]-r[:,:,:3],axis=2)
            if abs(dx)+abs(dy)==1:minimum=np.minimum(minimum,np.where(valid&(distance>1e-6),distance,np.inf))
            neighbors.append((dx,dy,valid,distance,1. if dx==dy==0 else .25 if dx and dy else .5))
    mask=np.zeros((h,w),dtype=np.uint32);sw=np.zeros((h,w));sw2=np.zeros((h,w))
    for bit,(dx,dy,valid,distance,weight) in enumerate(neighbors):
        valid=valid&(distance<=2.5*minimum+.00001)&np.isfinite(minimum)
        mask|=valid.astype(np.uint32)<<bit;sw+=valid*weight;sw2+=valid*weight*weight
    effective=np.divide(sw*sw,sw2,out=np.zeros_like(sw),where=sw2>0)
    mask|=(effective>=4).astype(np.uint32)*512;mask|=(effective>=6).astype(np.uint32)*1024
    return mask

def hash32(x):
    x &= 0xffffffff;x=((x^(x>>16))*0x7feb352d)&0xffffffff;x=((x^(x>>15))*0x846ca68b)&0xffffffff;return x^(x>>16)
def direct_cpu(recv,scene,index,cols,rows,frame):
    lo=np.array([np.array(e["transform"]["position"])-np.array(e["transform"]["size"])/2 for e in scene["entities"]]);hi=np.array([np.array(e["transform"]["position"])+np.array(e["transform"]["size"])/2 for e in scene["entities"]]);colors=np.array([e["color"] for e in scene["entities"]])
    sun=-np.array(scene["light"]["direction"]);sun/=np.linalg.norm(sun);energy=np.array(scene["light"]["direct_color"])*scene["light"]["direct_intensity"]
    p=recv[index,:3];n=recv[index,4:7];t=np.cross([1.,0.,0.] if abs(n[1])>.9 else [0.,1.,0.],n);t/=np.linalg.norm(t);origin=p+n*.002
    def slabs(p,d):
        with np.errstate(divide="ignore",invalid="ignore"):
            a=(lo-p)/d;b=(hi-p)/d
        return np.minimum(a,b).max(axis=1),np.maximum(a,b).min(axis=1)
    seed=hash32(index+frame*91237+9923);total=np.zeros(3)
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
    return recv[index,8:11]*total/(cols*rows)

def scene_report(label):
    path=OUT/label;data=json.loads((path/"results.json").read_text());scene=json.loads((path/"scene.json").read_text())
    assert data["hybrid_audit"] and data["hybrid_oracles"] and data["reconstruction_oracles"] and data["compact_empty_lists"]
    assert all(data[k] for k in ["native_profile_identity","passthrough_identity","zero_sun_direct","zero_sun_rebuilt_probe","channel_isolation","monotonic_gpu_timestamps"])
    w,h=data["width"],data["height"];recv=np.fromfile(path/"receivers.bin",dtype="<f4").reshape(-1,12).astype(float);live=recv[:,3]>0
    expected_risk=geometric_features(recv,scene);risk_error=float(np.max(abs(expected_risk[live]-recv[live,11])));assert risk_error<2e-4
    guides=np.fromfile(path/"guides.bin",dtype="<u4").reshape(h,w);expected_guides=guide_cpu(recv,w,h)
    assert np.array_equal(guides,expected_guides),(label,"guide mask mismatch",int((guides!=expected_guides).sum()))
    work=np.fromfile(path/"work.bin",dtype="<u4");count_low,count_high=map(int,work[:2]);pixels=w*h
    low=work[2:2+count_low];high=work[2+pixels:2+pixels+count_high]
    expected_low=np.flatnonzero(live&((guides.reshape(-1)&512)!=0));expected_high=np.flatnonzero(live&((guides.reshape(-1)&512)==0))
    assert count_low+count_high==int(live.sum())
    assert np.array_equal(np.sort(low),expected_low) and np.array_equal(np.sort(high),expected_high)
    dispatch=np.fromfile(path/"dispatch.bin",dtype="<u4");assert np.array_equal(dispatch,[(count_low+63)//64,1,1,(count_high+63)//64,1,1])
    constant=np.fromfile(path/"constant-filter.rgba32f",dtype="<f4").reshape(-1,4)
    expected=np.zeros_like(constant[:,:3]);expected[live]=[.001,.002,.003];assert np.abs(constant[:,:3]-expected).max()<1e-7
    ref=np.mean([np.fromfile(path/f"reference-{k}.rgba32f",dtype="<f4").reshape(-1,4)[:,:3].astype(float) for k in [7,8]],axis=0)
    assert np.all(np.isfinite(ref)) and np.all(ref[~live]==0)
    if label.startswith("validation"):
        wall=scene["entities"][-1];center=np.array(wall["transform"]["position"]);half=np.array(wall["transform"]["size"])/2
        axis=int(np.argmin(half));other=2 if axis==0 else 0
        band=live&(abs(recv[:,axis]-center[axis])<.65)&(recv[:,1]<center[1]+half[1]+.2)&(abs(recv[:,other]-center[other])<half[other]+.1)
    else:band=live&(abs(recv[:,0]-.27)<.65)&(recv[:,1]<2)&(recv[:,2]>-1.1)&(recv[:,2]<3.1)
    assert band.sum()>32
    modes=sorted({r["mode"] for r in data["rows"]});maps={mode:np.stack([np.fromfile(path/f"quality-{mode}-{seed}.rgba32f",dtype="<f4").reshape(-1,4) for seed in range(4)]).astype(float) for mode in modes if mode}
    cpu_checks=0;cpu_error=0.;rng=np.random.default_rng(208)
    for i in rng.choice(np.flatnonzero(live),24,replace=False):
        expected=direct_cpu(recv,scene,int(i),8,8,7);error=float(np.max(abs(expected-maps[2][0,i,:3])));cpu_error=max(cpu_error,error);cpu_checks+=1
        assert error<2e-5,(label,"CPU direct",int(i),error)
    direct=maps[2];probe=maps[6]
    # Branch recomposition checks all pixels/all seeds, independently of the quality reference.
    composition_error=0.
    for mode in modes:
        if 9<=mode<=16:
            mask=maps[mode][:,:,3]==2
            assert np.all((maps[mode][:,:,3]==0)|mask)
            expected=np.where(mask[:,:,None],direct[:,:,:3],probe[:,:,:3])
            error=float(np.max(np.abs(expected-maps[mode][:,:,:3])));composition_error=max(composition_error,error);assert error<2e-7,(label,mode,"composition",error)
            distance={9:.25,10:.5,11:.75,12:1.}.get(mode,.25)
            must_trace=live&(recv[:,11]<=distance+.00001)
            assert np.all(mask[:,must_trace]),(label,mode,"geometry gate")
            if mode>=13:
                field=np.fromfile(path/"probe-field-fine.rgba32f",dtype="<f4").reshape(-1,6,4)
                origin=recv[:,:3]+recv[:,4:7]*.002;g=np.clip((origin-[-5,.125,-5])/.5,0,[20,8,20]);cell=np.minimum(np.floor(g).astype(int),[19,7,19])
                axis=np.argmax(abs(recv[:,4:7]),axis=1);kind=axis*2+(recv[np.arange(len(recv)),axis+4]<0)
                low=np.full((len(recv),3),1e20);high=-low.copy();valid_any=np.zeros(len(recv),bool)
                for j in range(8):
                    q=cell+[j&1,(j>>1)&1,(j>>2)&1];idx=q[:,0]+21*(q[:,1]+9*q[:,2]);v=field[idx,kind];ok=v[:,3]>0
                    low[ok]=np.minimum(low[ok],v[ok,:3]);high[ok]=np.maximum(high[ok],v[ok,:3]);valid_any|=ok
                scale=max(np.array(scene["light"]["direct_color"])*scene["light"]["direct_intensity"])
                spread=(high-low).max(axis=1)/max(scale,1e-6);spread[~valid_any]=1e20;limit={13:.02,14:.04,15:.08,16:.03}[mode]
                predicted=must_trace|(spread>limit-.000001)|(probe[3,:,3]>0)
                decisive=live&(abs(spread-limit)>2e-6)
                assert np.all(mask[3,decisive]==predicted[decisive]),(label,mode,"spread gate")
    filter_error=0.
    for mode in modes:
        if mode<17:continue
        inputs=np.fromfile(path/f"input-{mode}.rgba32f",dtype="<f4").reshape(h,w,4).astype(float);flag=1024 if mode==17 else 512;eligible=(guides&flag)!=0
        expected=inputs.copy();sums=np.zeros((h,w,3));weights=np.zeros((h,w))
        for dy in [-1,0,1]:
            for dx in [-1,0,1]:
                bit=(dy+1)*3+dx+1;weight=1. if dx==dy==0 else .25 if dx and dy else .5;ok=(guides&(1<<bit))!=0
                sums+=np.roll(inputs[:,:,:3],(-dy,-dx),(0,1))*(weight*ok)[:,:,None];weights+=weight*ok
        expected[eligible,:3]=sums[eligible]/weights[eligible,None];expected[~live.reshape(h,w)]=0
        error=float(np.abs(expected.reshape(-1,4)-maps[mode][3]).max());filter_error=max(filter_error,error);assert error<2e-7,(label,mode,"CPU filter",error)
        for i in rng.choice(np.flatnonzero(live),8,replace=False):
            i=int(i);edge=not eligible.reshape(-1)[i]
            cols,rows=(8,8) if edge and mode!=18 else ({19:8,20:8,21:5,22:6,23:7}[mode],4) if mode>=19 else (4,4)
            expected=direct_cpu(recv,scene,i,cols,rows,55);error=float(np.max(abs(expected-inputs.reshape(-1,4)[i,:3])));cpu_error=max(cpu_error,error);cpu_checks+=1
            assert error<2e-5,(label,"CPU reconstruction source",mode,i,error)
        wanted=np.where((~eligible).reshape(-1)&live,2.,0.) if mode!=18 else np.zeros(len(recv))
        assert np.all(maps[mode][:,:,3]==wanted[None,:]),(label,mode,"sample allocation")
    equivalence=None
    if 19 in modes and 20 in modes:
        equivalence=float(np.max(np.abs(maps[19]-maps[20])))
        assert equivalence<2e-7,(label,"compact32 policy changed",equivalence)
    p0=[r for r in data["primary_study"] if not r["feature"]];p1=[r for r in data["primary_study"] if r["feature"]]
    assert len(p0)==len(p1)==32 and len(data["guide_study"])==32
    primary_plain=median(p0,"gpu_ms");primary_risk=median(p1,"gpu_ms");guide_cost=median(data["guide_study"],"gpu_ms");assert len(data["compact_study"])==32;compact_cost=median(data["compact_study"],"gpu_ms")
    builds=[r["gpu_ms"] for r in data["builds"] if r["fine"] and not r.get("calibration",False) and r["seed"]>0];build_cost=float(np.median(builds))
    timings=[]
    for mode in modes:
        rows=[r for r in data["rows"] if r["mode"]==mode];assert len(rows)==128
        values=maps[mode] if mode else np.zeros((4,w*h,4))
        assert np.all(np.isfinite(values)) and np.all(values[:,:,:3]>=0) and np.all(values[:,~live,:3]==0)
        e=np.mean((values[:,:,:3]-ref[None,:,:])**2,axis=2)
        tiles=[]
        for y in range(0,h,h//8):
            for x in range(0,w,w//8):
                active=live.reshape(h,w)[y:y+h//8,x:x+w//8]
                if active.sum()>=32:tiles.append(float(e.reshape(4,h,w)[:,y:y+h//8,x:x+w//8][:,active].mean()))
        cpu=[r["prepare_cpu_ms"]+r["raster_cpu_ms"]+r["compute_cpu_ms"] for r in rows];gpu=[r["raster_gpu_ms"]+r["compute_gpu_ms"] for r in rows]
        f64=float((values[:,live,3]==2).mean());f16=float((values[:,live,3]==1).mean());needs_probes=mode==6 or 9<=mode<=16
        setup=primary_risk if 9<=mode<=16 else primary_plain+(guide_cost if mode>=17 else 0)+(compact_cost if mode>=20 else 0)
        total=float(np.median(gpu));build=build_cost if needs_probes else 0.
        full=[r for r in data["full_rows"] if r["mode"]==mode]
        if mode==2 or mode>=20:assert len(full)==32
        complete_gpu=median(full,"gpu_ms") if full else None
        complete_cpu=median(full,"cpu_ms") if full else None
        timings.append(dict(complete_gpu_ms=complete_gpu,complete_cpu_ms=complete_cpu,mode=mode,name=NAMES[mode],cpu_ms=float(np.median(cpu)),gpu_ms=total,compute_gpu_ms=median(rows,"compute_gpu_ms"),gpu_p95_ms=float(np.percentile(gpu,95)),rgb_mse=float(e[:,live].mean()),near_mse=float(e[:,band].mean()),pixel_p95_mse=float(np.percentile(e[:,live],95)),worst_tile_mse=max(tiles),mse_per_seed=e[:,live].mean(axis=1).tolist(),near_mse_per_seed=e[:,band].mean(axis=1).tolist(),direct64_fraction=f64,fallback16_fraction=f16,primary_guide_gpu_ms=setup,probe_interleaved_build_gpu_ms=build,moving_static_gpu_model_ms=total+setup,moving_reuse16_gpu_model_ms=total+setup+build/16,needs_probe_field=needs_probes))
    for t in timings:
        t["cpu_p95_ms"]=float(np.percentile([r["prepare_cpu_ms"]+r["raster_cpu_ms"]+r["compute_cpu_ms"] for r in data["rows"] if r["mode"]==t["mode"]],95))
    baseline=next(t for t in timings if t["mode"]==2)
    for t in timings:
        t["quality_ratios"]={key:t[key]/baseline[key] for key in CRITERIA["metrics"]}
        t["eligible_quality"]=all(r<=1.+1e-10 for r in t["quality_ratios"].values())
        t["calibration_margin_pass"]=all(r<=CRITERIA["calibration_quality_ratio_max"] for r in t["quality_ratios"].values())
    return dict(label=label,width=w,height=h,visible_receivers=int(live.sum()),compact32_policy_max_error=equivalence,compact_gpu_ms=compact_cost,compact_counts=[count_low,count_high],risk_cpu_max_error=risk_error,cpu_transport_checks=cpu_checks,cpu_transport_max_error=cpu_error,guide_cpu_exact=True,branch_cpu_max_error=composition_error,filter_cpu_max_error=filter_error,primary_plain_gpu_ms=primary_plain,primary_risk_gpu_ms=primary_risk,guide_gpu_ms=guide_cost,incremental_filter_bytes=w*h*20,incremental_compaction_bytes=w*h*8+56,probe_build_context_gpu_ms=build_cost,timings=timings)

def freeze_selection(calibration):
    candidates=[]
    for mode in range(20,24):
        rows=[next(t for t in calibration[label]["timings"] if t["mode"]==mode) for label in ["open","thin-wall"]]
        baseline=[next(t for t in calibration[label]["timings"] if t["mode"]==2) for label in ["open","thin-wall"]]
        eligible=all(all(v<=CRITERIA["calibration_quality_ratio_max"] for v in t["quality_ratios"].values()) for t in rows)
        cost=max(t["complete_gpu_ms"]/b["complete_gpu_ms"] for t,b in zip(rows,baseline))
        candidates.append(dict(mode=mode,eligible=eligible,worst_cost_ratio=cost,quality=[t["quality_ratios"] for t in rows]))
    eligible=[c for c in candidates if c["eligible"] and c["worst_cost_ratio"]<1]
    if not eligible:raise RuntimeError("No cheaper candidate meets all four quality gates; revise calibration before validation")
    winner=min(eligible,key=lambda c:c["worst_cost_ratio"])
    selection=dict(selected_mode=winner["mode"],fallback_mode=20,selected_name=NAMES[winner["mode"]],criteria=CRITERIA,source_sha256=hashes(),calibration_candidates=candidates,calibration_results_sha256={label:hashlib.sha256((OUT/label/"results.json").read_bytes()).hexdigest() for label in ["open","thin-wall"]})
    (OUT/"selection.json").write_text(json.dumps(selection,indent=2));return selection

def write_report(reports,selection):
    mode=selection["selected_mode"];validation=[label for label in reports if label.startswith("validation")]
    assert selection["criteria"]==CRITERIA and selection["source_sha256"]==hashes()
    outcome={label:next(t for t in reports[label]["timings"] if t["mode"]==mode)["eligible_quality"] for label in validation}
    fallback={label:next(t for t in reports[label]["timings"] if t["mode"]==20)["eligible_quality"] for label in validation}
    effective=mode if all(outcome.values()) else 20 if all(fallback.values()) else None
    summary=dict(criteria=CRITERIA,selection=selection,effective_mode=effective,heldout_quality_pass=outcome,heldout_fallback_pass=fallback,reports=reports,source_sha256=hashes(),analysis_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),executable_sha256=hashlib.sha256((ROOT/"target/release/examples/lighting_bounce").read_bytes()).hexdigest())
    (OUT/"summary.json").write_text(json.dumps(summary,indent=2))
    font=ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf",20)
    for label in reports:
        canvas=Image.new("RGB",(1440,380),"#13191f");draw=ImageDraw.Draw(canvas)
        for k,m in enumerate([2,effective or mode,7]):
            name="reference-a" if m==7 else FILES[m];im=Image.open(OUT/label/f"{m}-{name}.png").convert("RGB");canvas.paste(im,(k*480,60));draw.text((k*480+10,10),"Referência4096" if m==7 else NAMES[m],font=font,fill="white")
        canvas.save(OUT/f"{label}-comparison.png")
        # Selected input uses no cross-instance history; show absolute RGB error separately.

    rows=""
    for label,r in reports.items():
        for t in r["timings"]:
            ratios=" / ".join(f"{t['quality_ratios'][k]:.3f}" for k in CRITERIA["metrics"])
            complete="—" if t["complete_gpu_ms"] is None else f'{t["complete_gpu_ms"]:.3f}'
            rows+=f'<tr><td>{label}</td><td>{html.escape(t["name"])}</td><td>{t["cpu_ms"]:.3f}</td><td>{t["gpu_ms"]:.3f}</td><td>{complete}</td><td>{ratios}</td><td>{t["direct64_fraction"]:.1%}</td><td>{t["eligible_quality"]}</td></tr>'
    pictures="".join(f'<h2>{label}</h2><img src="{label}-comparison.png">' for label in reports)
    (OUT/"review.html").write_text(f'''<!doctype html><meta charset="utf-8"><title>Ignis — qualidade/custo</title><style>body{{background:#13191f;color:#e5edf4;font:17px system-ui;max-width:1500px;margin:32px auto;padding:16px}}td,th{{padding:9px;border-bottom:1px solid #40505d;text-align:left}}img{{width:100%}}</style><h1>Alocação híbrida × reconstrução espacial</h1><p>Política congelada: {NAMES[mode]}. Critérios: MSE global/região da parede/p95/pior bloco com margem10% nas cenas de calibração e não piores que64raios nas novas cenas reservadas. Orçamento32 anterior é fallback pré-declarado; equivalência com implementação agrupada é verificada. Validação reservada: {html.escape(str(outcome))}; fallback: {html.escape(str(fallback))}; política efetiva: {effective}. Nenhum mínimo global ou aceite artístico inferido.</p>
<p>CPU preparação+submissão, GPU sombra+estimativa+eventual filtro,128amostras/método, bloqueio/readback/PNG excluídos. Coluna completa mede raster+reconstrução do mapa primário+guia+compactação GPU+amostragem+filtro em todo frame,32amostras/variante. Geometria/câmera ficam fixas durante cada caso, mas os mapas são reconstruídos, evitando esconder seu custo. Outras colunas são consultas estáticas128amostras. Não é FPS de jogo nem teste de movimento contínuo. Custos de probes e modelos comparativos continuam no <a href="summary.json">JSON</a>. Campos e filtros são aproximações com viés; sem acúmulo temporal.</p>
<table><tr><th>Cena</th><th>Método</th><th>CPU ms</th><th>GPU ms</th><th>GPU completa medida ms</th><th>Razões erro global / parede / p95 / pior bloco</th><th>Fração64</th><th>Qualidade elegível</th></tr>{rows}</table>{pictures}''')
    return summary
