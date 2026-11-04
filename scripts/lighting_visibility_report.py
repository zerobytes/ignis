# /// script
# requires-python = ">=3.11"
# dependencies = ["numpy>=2,<3", "pillow>=11,<13"]
# ///
"""Audit visible receivers independently from offscreen lighting influencers."""
from pathlib import Path
import hashlib
import json
import numpy as np
from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "artifacts/lighting-visibility"
reports = {}
for label in ("gallery", "guard"):
    path = OUT / label
    data = json.loads((path / "results.json").read_text())
    scene = json.loads((path / "scene.json").read_text())
    w, h = data["width"], data["height"]
    assert data["visibility_audit"] and data["compute_timestamps_monotonic"]
    raw = (path / "receivers.bin").read_bytes()
    f = np.frombuffer(raw, dtype="<f4").reshape(-1,40)
    u = np.frombuffer(raw, dtype="<u4").reshape(-1,40)
    live = f[:,3]>0
    p, n, masks = f[live,:3].astype(float), f[live,4:7].astype(float), u[live,7]
    ids = f[live,3].astype(int)-1
    count = int(live.sum())
    eye = np.array(scene["camera"]["position"])
    forward = np.array(scene["camera"]["target"])-eye
    forward /= np.linalg.norm(forward)
    right = np.cross(forward,[0.,1.,0.]); right /= np.linalg.norm(right)
    up = np.cross(right,forward)
    v = p-eye; depth = v@forward
    scale = np.tan(np.radians(scene["camera"]["fov_degrees"])/2)
    assert np.all(depth>0)
    assert np.all(np.abs(v@right/depth/scale/(w/h))<=1.0001)
    assert np.all(np.abs(v@up/depth/scale)<=1.0001)
    origin = p+n*.002
    candidate_counts = np.zeros(count, dtype=int)
    influence_counts=[]
    for j, entity in enumerate(scene["entities"]):
        c=np.array(entity["transform"]["position"]); half=np.array(entity["transform"]["size"])/2
        near=np.clip(origin,c-half,c+half)
        distance=np.linalg.norm(origin-near,axis=1)
        reach=np.sum(n*(c-origin),axis=1)+np.sum(np.abs(n)*half,axis=1)
        included=(masks & (1<<j))!=0
        required=(distance<=4)&(reach>=0)
        allowed=(distance<=4.001)&(reach>=-.001)
        assert not np.any(required & ~included), (label,j,"lost necessary box")
        assert not np.any(included & ~allowed), (label,j,"unexpected distant box")
        candidate_counts+=included
        influence_counts.append(int(included.sum()))
    box_count=len(scene["entities"])
    timings=[]
    for mode in range(10):
        pairs=[]
        for cull in (0,1):
            rows=[r for r in data["rows"] if r["mode"]==mode and r["culling"]==cull]
            assert len(rows)==128
            def median(key):
                return float(np.median([r[key] for r in rows]))
            gpu=np.array([r["raster_gpu_ms"]+r["compute_gpu_ms"] for r in rows])
            cpu=np.array([r["prepare_cpu_ms"]+r["raster_cpu_ms"]+r["compute_cpu_ms"] for r in rows])
            pairs.append(dict(culling=cull,cpu_ms=float(np.median(cpu)),gpu_ms=float(np.median(gpu)),
                              compute_gpu_ms=median("compute_gpu_ms"),gpu_p95_ms=float(np.percentile(gpu,95)),
                              setup_gpu_ms=median("setup_gpu_ms")))
        stem=f'{mode:02}-{["native","mc16","cache","cv","pilot","validated","ttl","revision","watched","reference512"][mode]}'
        for ext in (["png"] if mode==0 else ["png","f32"]):
            assert (path/f"{stem}-0.{ext}").read_bytes()==(path/f"{stem}-1.{ext}").read_bytes(), (label,stem,ext)
        timings.append(dict(mode=mode,name=stem,pairs=pairs))
    hit_counts=np.bincount(ids,minlength=box_count).tolist()
    if label=="guard":
        # Explicitly establish the entire blocker lies behind the eye, not only zero raster hits.
        entity=scene["entities"][2]
        c=np.array(entity["transform"]["position"]); half=np.array(entity["transform"]["size"])/2
        assert (c-eye)@forward+np.abs(forward)@half<0
        assert hit_counts[2]==0 and influence_counts[2]>0
        good=np.asarray(Image.open(path/"09-reference512-0.png"))
        bad=np.asarray(Image.open(path/"incorrect-offscreen-removal.png"))
        changed=int(np.any(good!=bad,axis=2).sum())
        assert changed>100
    else:
        changed=0
    paired={}
    for row in data["rows"]:
        if row["mode"]==1:
            paired.setdefault((row["round"],row["frame"]),{})[row["culling"]]=row["compute_gpu_ms"]
    pairs=np.array([[v[0],v[1]] for _,v in sorted(paired.items())]).reshape(16,8,2)
    rng=np.random.default_rng(208)
    effects=[]
    for _ in range(2000):
        sample=pairs[rng.integers(0,16,16)].reshape(-1,2)
        effects.append(np.median(sample[:,0]-sample[:,1]))
    effect=float(np.median(pairs[:,:,0]-pairs[:,:,1]))
    ci=np.percentile(effects,[2.5,97.5]).tolist()
    assert ci[0]>0, "MC compute improvement not supported by paired block bootstrap"
    reports[label]=dict(mc_paired_median_compute_saving_ms=effect, mc_saving_block_bootstrap95_ms=ci, width=w,height=h,visible_receivers=count,background_pixels=w*h-count,
                        receiver_fraction=count/(w*h),primary_hits_by_object=hit_counts,
                        influenced_receivers_by_object=influence_counts,
                        mean_candidate_boxes=float(candidate_counts.mean()),box_count=box_count,
                        candidate_ceiling_fraction=float(candidate_counts.sum()/(count*box_count)),
                        negative_control_changed_pixels=changed,timings=timings)

# Directly show why deleting all geometry outside the camera breaks lighting.
font=ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf",21)
canvas=Image.new("RGB",(1280,466),"#13191f");draw=ImageDraw.Draw(canvas)
for k,(name,title) in enumerate([("09-reference512-1.png","Correto: obstáculo atrás da câmera mantido"),("incorrect-offscreen-removal.png","Incorreto: obstáculo fora da tela removido")]):
    canvas.paste(Image.open(OUT/"guard"/name).convert("RGB"),(k*640,50))
    draw.text((k*640+12,12),title,font=font,fill="white")
canvas.save(OUT/"offscreen-comparison.png")
hashes={str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in [ROOT/"examples/lighting_gallery.rs",ROOT/"examples/lighting_gallery/estimators.wgsl",Path(__file__)]}
(OUT/"summary.json").write_text(json.dumps(dict(reports=reports,source_sha256=hashes),indent=2))
rows=""
for label, report in reports.items():
    for timing in report["timings"]:
        a,b=timing["pairs"]
        rows+=f'<tr><td>{label}</td><td>{timing["name"]}</td><td>{a["compute_gpu_ms"]:.3f}</td><td>{b["compute_gpu_ms"]:.3f}</td><td>{a["gpu_ms"]:.3f}</td><td>{b["gpu_ms"]:.3f}</td></tr>'
(OUT/"review.html").write_text(f'''<!doctype html><meta charset="utf-8"><title>Ignis — visibilidade e iluminação</title>
<style>body{{background:#13191f;color:#e5edf4;font:17px system-ui;max-width:1400px;margin:32px auto;padding:16px}}table{{border-collapse:collapse;width:100%}}td,th{{padding:10px;border-bottom:1px solid #40505d;text-align:left}}img{{width:100%}}</style>
<h1>Iluminar somente o que aparece, preservar o que influencia</h1>
<p>Receptores sempre são os primeiros pontos visíveis por pixel. Objetos fora da câmera continuam participando de oclusão/sombra quando necessário. O filtro experimental remove somente caixas fora do raio4m ou inteiramente sob o hemisfério do receptor; não usa o frustum da câmera para esses obstáculos.</p>
<p>Galeria: {reports["gallery"]["visible_receivers"]} receptores de266240pixels. Média de caixas candidatas por receptor {reports["gallery"]["box_count"]}→{reports["gallery"]["mean_candidate_boxes"]:.2f}. Isso reduz o teto de candidatos, não é contagem de interseções executadas nem redução de raios. PNGs e mapas AO de todos os métodos são byte-exatos entre variantes.</p>
<p>Controle: o obstáculo inteiramente atrás da câmera recebe zero hits primários e influencia {reports["guard"]["influenced_receivers_by_object"][2]} receptores. Removê-lo incorretamente altera {reports["guard"]["negative_control_changed_pixels"]} pixels, apesar de não aparecer.</p>
<img src="offscreen-comparison.png">
<p>M4Pro/Metal640×416,128 amostras por método/variante, pares intercalados. Medianas em ms. CPU, p95 e setup no <a href="summary.json">JSON</a>. GPUtotal cobre sombra/cena+compute; bloqueio/readback/PNG excluídos. Cache/máscara estáticos compartilhados; construção da máscara fora dos quadros medidos, mesmos160bytes/receptor. Não é benchmark de mundo dinâmico ou GI multibounce.</p>
<table><tr><th>Cena</th><th>Método</th><th>Compute sem filtro</th><th>Compute filtrado</th><th>GPUtotal sem filtro</th><th>GPUtotal filtrado</th></tr>{rows}</table>''')
print(json.dumps(reports,indent=2))
