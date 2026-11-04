# /// script
# requires-python = ">=3.11"
# dependencies = ["numpy>=2,<3", "pillow>=11,<13"]
# ///
"""Report native GPU experiment; uv run scripts/lighting_gallery_report.py."""
from pathlib import Path
import hashlib
import html
import json
import numpy as np
from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "artifacts/lighting-gallery"
data = json.loads((OUT / "results.json").read_text())
names = ["Atual — sem candidato", "Monte Carlo · 16", "Cache isolado", "Correção · peso 1", "Peso aprendido", "Peso validado", "Compartilhado · TTL32", "Revisão explícita", "Sondas pareadas", "Referência · 512"]
font_path = "/System/Library/Fonts/Supplemental/Arial.ttf"
font = ImageFont.truetype(font_path, 23)
small = ImageFont.truetype(font_path, 19)
ref = np.fromfile(OUT / "09-reference512.f32", dtype="<f4")
summary = []
for mode, name in enumerate(names):
    rows = [r for r in data["rows"] if r["mode"] == mode]
    samples = lambda key: np.array([r[key] for r in rows])
    cpu = samples("prepare_cpu_ms") + samples("raster_cpu_ms") + samples("compute_cpu_ms")
    gpu = samples("raster_gpu_ms") + samples("compute_gpu_ms")
    image = sorted(OUT.glob(f"{mode:02}-*.png"))[0]
    ao = np.ones_like(ref) if mode == 0 else np.fromfile(next(OUT.glob(f"{mode:02}-*.f32")), dtype="<f4")
    assert np.all(np.isfinite(ao)) and ao.shape == ref.shape
    summary.append(dict(mode=mode, name=name, image=image.name, samples=len(rows),
                        cpu_ms=float(np.median(cpu)), cpu_p95_ms=float(np.percentile(cpu, 95)),
                        gpu_ms=float(np.median(gpu)), gpu_p95_ms=float(np.percentile(gpu, 95)),
                        raster_gpu_ms=float(np.median(samples("raster_gpu_ms"))),
                        compute_gpu_ms=float(np.median(samples("compute_gpu_ms"))),
                        compute_cpu_ms=float(np.median(samples("compute_cpu_ms"))),
                        setup_gpu_ms=float(np.median(samples("setup_gpu_ms"))),
                        setup_cpu_ms=float(np.median(samples("setup_cpu_ms"))),
                        mse_against_512=float(np.mean((ao-ref)**2)),
                        outside_unit_fraction=float(np.mean((ao<0)|(ao>1)))))
# Workload response audit catches counter attribution failures without an absolute-ms target.
heavy_light_ratio = summary[9]["compute_gpu_ms"] / summary[1]["compute_gpu_ms"]
cache_light_ratio = summary[2]["compute_gpu_ms"] / summary[1]["compute_gpu_ms"]
assert heavy_light_ratio > 8, "512-query workload failed GPU attribution audit"
assert cache_light_ratio < 1, "Zero-query cache failed GPU attribution audit"
# A single native render is a separate light model, not an MC estimator of AO.
# This approximate reference has finite sampling variance; never label it ground truth.
assert data["native_profile_identity"] and data["passthrough_identity"]
assert data["timestamp_resolution_fenced"] and data["compute_timestamps_monotonic"]
assert data["static_ttl_revision_identity"]
for key in ("cpu_ms", "gpu_ms", "compute_gpu_ms"):
    assert all(np.isfinite(row[key]) and row[key] >= 0 for row in summary)

# Numerical oracle independent of WGSL: selected primary hits and cosine rays.
scene = json.loads((OUT / "scene.json").read_text())
boxes = [(np.array(e["transform"]["position"]), np.array(e["transform"]["size"])) for e in scene["entities"]]
eye = np.array(scene["camera"]["position"])
forward = np.array(scene["camera"]["target"]) - eye
forward /= np.linalg.norm(forward)
right = np.cross(forward, [0., 1., 0.]); right /= np.linalg.norm(right)
up = np.cross(right, forward)
scale = np.tan(np.radians(scene["camera"]["fov_degrees"])/2)
w, h = data["width"], data["height"]
def hit(p, d):
    with np.errstate(divide="ignore", invalid="ignore"):
        a = np.array([(c-s/2-p)/d for c,s in boxes]); b = np.array([(c+s/2-p)/d for c,s in boxes])
    return np.max(np.minimum(a,b), axis=1), np.min(np.maximum(a,b), axis=1)
def hashed(x):
    x &= 0xffffffff; x = ((x^(x>>16))*0x7feb352d)&0xffffffff
    x = ((x^(x>>15))*0x846ca68b)&0xffffffff
    return x^(x>>16)
def uniform_pair(seed):
    seed=hashed(seed+0x9e3779b9); a=(seed>>8)/16777216
    seed=hashed(seed+0x9e3779b9); b=(seed>>8)/16777216
    return seed, (a,b)
mc=np.fromfile(OUT/"01-mc16.f32", dtype="<f4")
cache=np.fromfile(OUT/"02-cache.f32", dtype="<f4")
checked=0
for i in np.random.default_rng(208).choice(w*h, 160, replace=False):
    i=int(i); y,x=divmod(i,w)
    d=forward+right*((x+.5)/w*2-1)*w/h*scale+up*(1-(y+.5)/h*2)*scale
    d/=np.linalg.norm(d); near,far=hit(eye,d)
    valid=(near>0)&(far>=near)
    if not valid.any():
        assert mc[i]==1 and cache[i]==1
        continue
    j=int(np.argmin(np.where(valid,near,np.inf)))
    p=eye+d*near[j]; c,s=boxes[j]; local=(p-c)/(s/2)
    n=np.zeros(3); axis=int(np.argmax(np.abs(local))); n[axis]=np.sign(local[axis])
    t=np.cross([1.,0.,0.] if abs(n[1])>.9 else [0.,1.,0.],n); t/=np.linalg.norm(t)
    def vis(v):
        a,b=v; direction=t*np.sqrt(a)*np.cos(2*np.pi*b)+np.cross(n,t)*np.sqrt(a)*np.sin(2*np.pi*b)+n*np.sqrt(1-a)
        en,ex=hit(p+n*.002,direction)
        return float(not np.any((ex>=np.maximum(en,0))&(en<4)&(ex>.001)))
    g=sum(vis(((k%4+.5)/4,(k//4+.5)/4)) for k in range(16))/16
    seed=hashed(i+15*91237+9923); values=[]
    for _ in range(16):
        seed,v=uniform_pair(seed); values.append(vis(v))
    assert cache[i]==g, (i,cache[i],g)
    assert mc[i]==np.mean(values), (i,mc[i],np.mean(values))
    checked+=1
assert checked>50

# Two compact comparison plates; all tiles retain the same image exposure.
def plate(ids, cols, filename):
    tw,th=512,333; header=68
    canvas=Image.new("RGB",(cols*tw, ((len(ids)+cols-1)//cols)*(th+header)),"#13191f")
    draw=ImageDraw.Draw(canvas)
    for k,idx in enumerate(ids):
        row=summary[idx]; x=(k%cols)*tw; y=(k//cols)*(th+header)
        image=Image.open(OUT/row["image"]).convert("RGB").resize((tw,th),Image.Resampling.LANCZOS)
        canvas.paste(image,(x,y+header))
        draw.text((x+12,y+7),row["name"],font=font,fill="white")
        draw.text((x+12,y+37),f'CPU {row["cpu_ms"]:.3f} ms | GPU {row["gpu_ms"]:.3f} ms',font=small,fill="#bbd8e6")
    canvas.save(OUT/filename)
plate(list(range(9)),3,"candidates.png")
plate([0,6,9],3,"comparison.png")

hashes={str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in [ROOT/"examples/lighting_gallery.rs",ROOT/"examples/lighting_gallery/estimators.wgsl",ROOT/"src/render.rs",ROOT/"src/render/gpu_timing.rs",ROOT/"scripts/lighting_gallery_report.py",OUT/"scene.json"]}
report=dict(summary=summary, independent_cpu_oracle_receivers=checked, source_sha256=hashes,
            timestamp_workload_audit=dict(reference_to_mc=heavy_light_ratio, cache_to_mc=cache_light_ratio),
            timing_scope="CPU frame preparation + encode/upload/submit; GPU shadow+scene render passes plus AO compute. Excludes blocking/readback/PNG. Not FPS.",
            reference="512-sample Monte Carlo; finite variance, not exact ground truth")
(OUT/"summary.json").write_text(json.dumps(report,indent=2))
rows="".join(f'<tr><td>{html.escape(r["name"])}</td><td>{r["cpu_ms"]:.3f}</td><td>{r["gpu_ms"]:.3f}</td><td>{r["compute_gpu_ms"]:.3f}</td><td>{r["setup_gpu_ms"]:.3f}</td><td>{r["mse_against_512"]:.6f}</td></tr>' for r in summary)
cards="".join(f'<section><h2>{html.escape(r["name"])}</h2><a href="{r["image"]}"><img src="{r["image"]}"></a></section>' for r in summary)
(OUT/"review.html").write_text(f'''<!doctype html><meta charset="utf-8"><title>Ignis — lighting candidates</title>
<style>body{{background:#13191f;color:#e5edf4;font:17px system-ui;max-width:1400px;margin:32px auto;padding:16px}}table{{border-collapse:collapse;width:100%}}td,th{{padding:12px;border-bottom:1px solid #40505d;text-align:left}}.grid{{display:grid;grid-template-columns:1fr 1fr;gap:20px}}img{{width:100%}}h2{{font-size:20px}}</style>
<h1>Ignis: iluminação atual × candidatos experimentais</h1><p>{w}×{h}; {html.escape(data["adapter"]["name"])}. Mesma geometria, câmera, luz e sombras. AO de raio 4 m substitui somente o ambiente uniforme. Sem GI multibounce.</p>
<p>128 amostras de custo por candidato: duas rodadas de 64 frames, ordem rotacionada/invertida a cada frame, época de 32. Todos os métodos, inclusive a referência, têm 128 amostras de custo. Referência: 512 raios/pixel. CPU = preparação + submissão; GPU = timestamps dos passes de sombra/cena + compute. Contadores resolvidos após conclusão do passe, com oráculo de monotonicidade. Espera, readback e PNG excluídos; não mede FPS. Máquina compartilhada, amostras serializadas.</p>
<table><tr><th>Candidato</th><th>CPU total ms</th><th>GPU total ms</th><th>GPU adicional ms</th><th>Setup GPU ms</th><th>MSE AO vs 512*</th></tr>{rows}</table>
<p>*Referência ruidosa, MSE aproximado de um frame, incluindo fundo. Métodos buscam o mesmo resultado. Cache isolado apresenta viés. Nenhum denoiser ou acúmulo temporal. Coeficientes de TTL/revisão reutilizados; amostras de correção novas a cada frame. Revisão e TTL idênticos nesta cena estática; não testa invalidação dinâmica.</p>
<p>Protótipo reserva {data["additional_resident_bytes"]/2**20:.2f} MiB adicionais de recursos lógicos GPU, incluindo cache por pixel; staging e overhead do driver não incluídos. Setup de recursos/shader CPU {data["resources_setup_cpu_ms"]:.1f} ms; renderer frio {data["renderer_setup_cpu_ms"]:.1f} ms. Cache frio: 16 consultas por receptor (mais raio primário), além do quadro normal. Não extrapolar para cena arbitrária: interseções analíticas com 11 caixas.</p>
<p>Oráculos: pixels nativos com/sem profiler idênticos; passthrough sRGB idêntico; {checked} receptores confrontados com implementação CPU independente (MC/cache), saídas finitas; TTL/revisão bit-exatos. <a href="summary.json">Resumo e hashes</a> · <a href="results.json">Todos os tempos</a> · <a href="scene.json">Cena editável</a></p>
<p>Validação adicional: cinco testes de sombras passaram, inclusive GPU; um teste legado espera Scene111 e encontra a versão corrente120. Clippy estrito encontra collapsible_if preexistente em src/botany/production.rs:97; validação com esse lint isolado está documentada no contrato. Nenhum desses arquivos foi modificado por este experimento.</p>
<div class="grid">{cards}</div>''')
print(json.dumps(report,indent=2))
