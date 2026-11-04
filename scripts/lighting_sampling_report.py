# /// script
# requires-python = ">=3.11"
# dependencies = ["numpy>=2,<3", "pillow>=11,<13"]
# ///
"""H2 sample-distribution study, distinct from cache prediction or GI architecture."""
from pathlib import Path
import hashlib
import json
import numpy as np
from PIL import Image, ImageDraw, ImageFont

ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/"artifacts/lighting-sampling"
MODES=[1,3,4,5,6,10,11,12,13,14,15,16,17]
NAMES={1:"IID16",3:"Correção fixa16",4:"Peso aprendido16",5:"Peso validado16",6:"Compartilhado16",10:"IID4",11:"Estratificado4",12:"Estratificado16",13:"IID64",14:"Estratificado64",15:"Referência4096",16:"IID32",17:"Estratificado32"}
FILES={1:"mc16",3:"cv",4:"pilot",5:"validated",6:"ttl",10:"iid4",11:"stratified4",12:"stratified16",13:"iid64",14:"stratified64",15:"reference4096",16:"iid32",17:"stratified32"}
FRAMES=[7,23,39,55]
reports={}

def hashed(x):
    x &= 0xffffffff; x=((x^(x>>16))*0x7feb352d)&0xffffffff
    x=((x^(x>>15))*0x846ca68b)&0xffffffff
    return x^(x>>16)
def pair(seed):
    seed=hashed(seed+0x9e3779b9);a=(seed>>8)/16777216
    seed=hashed(seed+0x9e3779b9);b=(seed>>8)/16777216
    return seed,(a,b)
def normalized(v):
    return v/np.linalg.norm(v)

for label in ["gallery","guard"]:
    path=OUT/label
    data=json.loads((path/"results.json").read_text())
    assert data["sampling_audit"] and data["compute_timestamps_monotonic"]
    w,h=data["width"],data["height"]
    receivers=np.fromfile(path/"receivers.bin",dtype="<f4").reshape(-1,40)
    active=receivers[:,3]>0
    ref=np.fromfile(path/"15-reference4096.f32",dtype="<f4").astype(float)
    assert ref.size==w*h and np.all(np.isfinite(ref)) and np.all((ref>=0)&(ref<=1))
    scene=json.loads((path/"scene.json").read_text())
    boxes=[(np.array(e["transform"]["position"]),np.array(e["transform"]["size"])/2) for e in scene["entities"]]
    def visible(index,v):
        p=receivers[index,:3].astype(float);n=receivers[index,4:7].astype(float)
        t=normalized(np.cross([1.,0.,0.] if abs(n[1])>.9 else [0.,1.,0.],n))
        a,b=v;d=t*np.sqrt(a)*np.cos(2*np.pi*b)+np.cross(n,t)*np.sqrt(a)*np.sin(2*np.pi*b)+n*np.sqrt(1-a)
        origin=p+n*.002
        with np.errstate(divide="ignore",invalid="ignore"):
            for c,half in boxes:
                u=(c-half-origin)/d;z=(c+half-origin)/d
                enter=np.min([u,z],axis=0).max();exit=np.max([u,z],axis=0).min()
                if exit>=max(enter,0) and enter<4 and exit>.001:
                    return 0.
        return 1.
    # Independent scalar CPU oracle for all six plain/stratified variants.
    indices=np.random.default_rng(208).choice(np.flatnonzero(active),80,replace=False)
    for mode,side in [(10,0),(11,2),(1,0),(12,4),(13,0),(14,8),(16,0),(17,4)]:
        budget={10:4,11:4,1:16,12:16,13:64,14:64,16:32,17:32}[mode]
        saved=np.fromfile(path/f"quality-{mode}-7.f32",dtype="<f4")
        for index in indices:
            index=int(index);seed=hashed(index+7*91237+9923);total=0.;cells=[]
            for j in range(budget):
                seed,v=pair(seed)
                if side:
                    rows=8 if mode==17 else side
                    v=((j%side+v[0])/side,(j//side+v[1])/rows)
                    assert 0<=v[0]<1 and 0<=v[1]<1
                    cells.append((int(v[0]*side),int(v[1]*rows)))
                total+=visible(index,v)
            assert total/budget==saved[index],(label,mode,index,total/budget,saved[index])
            if side:
                assert len(set(cells))==budget
    timings=[];errors={};tiles={}
    for mode in MODES:
        rows=[r for r in data["rows"] if r["mode"]==mode]
        assert len(rows)==128 and all(r["culling"]==0 for r in rows)
        cpu=[r["prepare_cpu_ms"]+r["raster_cpu_ms"]+r["compute_cpu_ms"] for r in rows]
        gpu=[r["raster_gpu_ms"]+r["compute_gpu_ms"] for r in rows]
        timing=dict(mode=mode,name=NAMES[mode],cpu_ms=float(np.median(cpu)),gpu_ms=float(np.median(gpu)),compute_gpu_ms=float(np.median([r["compute_gpu_ms"] for r in rows])),gpu_p95_ms=float(np.percentile(gpu,95)))
        if mode!=15:
            maps=np.stack([np.fromfile(path/f"quality-{mode}-{frame}.f32",dtype="<f4").astype(float) for frame in FRAMES])
            assert np.all(np.isfinite(maps)) and np.all(maps[:,~active]==1)
            delta=maps-ref[None,:]
            per_frame=np.mean(delta[:,active]**2,axis=1)
            timing.update(mse_vs4096=float(per_frame.mean()),mse_per_frame=per_frame.tolist(),signed_error=float(delta[:,active].mean()),unit_range_violation_fraction=float(np.mean((maps[:,active]<0)|(maps[:,active]>1))))
            errors[mode]=timing["mse_vs4096"]
            # Keep spatial tiles intact across all four seeds when bootstrapping.
            mse_image=np.mean(delta**2,axis=0).reshape(h,w)
            mask=active.reshape(h,w)
            tiles[mode]=[(float(mse_image[y:y+h//8,x:x+w//8][mask[y:y+h//8,x:x+w//8]].sum()),int(mask[y:y+h//8,x:x+w//8].sum())) for y in range(0,h,h//8) for x in range(0,w,w//8)]
        timings.append(timing)
    comparisons=[]
    for budget,iid,strat in [(4,10,11),(16,1,12),(64,13,14),(32,16,17)]:
        rng=np.random.default_rng(208)
        a=np.array(tiles[iid]);b=np.array(tiles[strat]);ratios=[]
        for _ in range(2000):
            selected=rng.integers(0,len(a),len(a))
            ratios.append((a[selected,0].sum()/a[selected,1].sum())/(b[selected,0].sum()/b[selected,1].sum()))
        comparisons.append(dict(budget=budget,mse_gain=errors[iid]/errors[strat],mse_gain_spatial_bootstrap95=np.percentile(ratios,[2.5,97.5]).tolist()))
    rng=np.random.default_rng(208)
    a=np.array(tiles[13]);b=np.array(tiles[17]);ratios=[]
    for _ in range(2000):
        selected=rng.integers(0,len(a),len(a))
        ratios.append((a[selected,0].sum()/a[selected,1].sum())/(b[selected,0].sum()/b[selected,1].sum()))
    cross=dict(iid64_to_strat32_mse_gain=errors[13]/errors[17],mse_gain_spatial_bootstrap95=np.percentile(ratios,[2.5,97.5]).tolist())
    reports[label]=dict(half_query_comparison=cross,visible_receivers=int(active.sum()),cpu_oracle_checks=80*8,timings=timings,equal_budget=comparisons)

font=ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf",22)
small=ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf",18)
canvas=Image.new("RGB",(1536,802),"#13191f");draw=ImageDraw.Draw(canvas)
for k,mode in enumerate([1,16,13,12,17,14]):
    x=(k%3)*512;y=(k//3)*401
    canvas.paste(Image.open(OUT/"gallery"/f"{mode:02}-{FILES[mode]}.png").convert("RGB").resize((512,333),Image.Resampling.LANCZOS),(x,y+68))
    t=next(t for t in reports["gallery"]["timings"] if t["mode"]==mode)
    draw.text((x+12,y+8),NAMES[mode],font=font,fill="white")
    draw.text((x+12,y+38),f'GPU total {t["gpu_ms"]:.3f} ms · compute {t["compute_gpu_ms"]:.3f}',font=small,fill="#bbd8e6")
canvas.save(OUT/"comparison.png")
hashes={str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in [ROOT/"examples/lighting_gallery.rs",ROOT/"examples/lighting_gallery/estimators.wgsl",Path(__file__)]}
(OUT/"summary.json").write_text(json.dumps(dict(reports=reports,source_sha256=hashes,executable_sha256=hashlib.sha256((ROOT/"target/release/examples/lighting_gallery").read_bytes()).hexdigest()),indent=2))
rows=""
for label,r in reports.items():
    for t in r["timings"]:
        error="—" if t["mode"]==15 else f'{t["mse_vs4096"]:.6f}'
        rows+=f'<tr><td>{label}</td><td>{t["name"]}</td><td>{t["cpu_ms"]:.3f}</td><td>{t["gpu_ms"]:.3f}</td><td>{t["compute_gpu_ms"]:.3f}</td><td>{error}</td></tr>'
(OUT/"review.html").write_text(f'''<!doctype html><meta charset="utf-8"><title>Ignis — H2 amostragem</title><style>body{{background:#13191f;color:#e5edf4;font:17px system-ui;max-width:1500px;margin:32px auto;padding:16px}}td,th{{padding:10px;border-bottom:1px solid #40505d;text-align:left}}img{{width:100%}}</style>
<h1>H2: distribuir amostras, sem cache ou peso aprendido</h1><p>IID versus estratificação com jitter,4/16/32/64raios, mesma visibilidade/AO4m/câmera/geometria/luz/resolução. Comparadores da H1 também incluídos. Não é GI multibounce, ReSTIR, radiance cascades ou denoising; não escolhe a arquitetura de GI.</p><img src="comparison.png">
<p>M4Pro/Metal640×416;128tempos por método com ordem rotacionada/invertida. CPU preparação+submissão, GPU sombra/cena+compute; espera/readback/PNG excluídos. Máscara de influência/cache imutável preparado fora dos tempos de quadro; sem custo de atualização dinâmica incluído.</p>
<p>MSE de AO foreground contra referência independente4096amostras, média de quatro seeds de quadro. Referência tem variância finita; não é verdade exata nem medição de viés. CI do ganho no <a href="summary.json">JSON</a> reamostra64blocos espaciais, mantendo quatro seeds juntos; não implica generalização para outras cenas.</p>
<table><tr><th>Cena</th><th>Método</th><th>CPU ms</th><th>GPU total ms</th><th>GPU compute ms</th><th>MSE vs4096</th></tr>{rows}</table>
<p>Oráculo CPU independente:80receptores por método,640verificações por cena, com verificações de estratos/cobertura/budget, fundo sem raios e saídas finitas. Sem recursos GPU por-frame adicionais para a estratificação;4/16/64consultas explícitas, cache/pilotos ausentes nesses oito métodos. Build e execuções GPU passaram; Clippy final do exemplo passou com o lint preexistente collapsible_if isolado, sem allowance no código. Código de diagnóstico e fonte conceitual: <a href="https://www.pbr-book.org/4ed/Sampling_and_Reconstruction/Stratified_Sampler">PBRT Stratified Sampler</a>.</p>''')
print(json.dumps(reports,indent=2))
