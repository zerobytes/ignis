#!/usr/bin/env python3
"""Summarize current offscreen measurements and create a physical-pixel viewer."""
import json
import statistics
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "artifacts/appearance-probe/geometry-compiler"


def summarize():
    report = json.loads((OUT / "results.json").read_text())
    timing_source = OUT / "results.json"
    repeat = OUT / "timing-repeat.json"
    if repeat.exists() and repeat.stat().st_mtime > timing_source.stat().st_mtime:
        measured = json.loads(repeat.read_text())
        assert [(a["name"], a["factor"], a["width"], a["source_triangles"], a["proxy_triangles"], a["file_bytes"]) for a in measured["assets"]] == [(a["name"], a["factor"], a["width"], a["source_triangles"], a["proxy_triangles"], a["file_bytes"]) for a in report["assets"]]
        report["timings"] = measured["timings"]
        timing_source = repeat
    rows = []
    duplicates = []
    for name in ("stone", "folded"):
        for width in (1728, 3456):
            for distance in ("near", "far"):
                for factor in (1, 2, 10, 100):
                    group = [x for x in report["timings"] if (x["name"], x["width"], x["range"], x["factor"]) == (name, width, distance, factor)]
                    row = dict(name=name, width=width, range=distance, factor=factor)
                    for optional, key in ((False, "source"), (True, "compiled")):
                        times = [x["timing"] for x in group if x["optional"] == optional]
                        for field in ("gpu_ms", "cpu_encode_ms", "total_batch_ms_per_frame"):
                            row[f"{key}_{field}"] = statistics.median(t[field] for t in times)
                        for round_id in range(8):
                            pair = [x["timing"]["gpu_ms"] for x in group if x["optional"] == optional and x["round"] == round_id]
                            assert len(pair) == 2
                            duplicates.append(pair[1] / pair[0])
                    row["speedup"] = row["source_gpu_ms"] / row["compiled_gpu_ms"]
                    q = [x for x in report["quality"] if (x["name"], x["width"], x["range"], x["factor"]) == (name, width, distance, factor)]
                    row["max_rmse"] = max(x["quality"]["rmse"] for x in q)
                    row["max_coverage_error"] = max(x["quality"]["coverage_error"] for x in q)
                    row["all_pass"] = all(x["quality"]["pass"] for x in q)
                    rows.append(row)
    builds = {f"{name}-{factor}": json.loads((OUT / f"{name}-{factor}.igc.json").read_text()) for name in ("stone", "folded") for factor in (1, 2, 10, 100)}
    result = dict(timing_source=timing_source.name,frames_per_batch=report["timings"][0]["timing"].get("frames",32),rows=rows, builds=builds, quality_cases=len(report["quality"]), failed=report["failed"], max_rmse=max(x["quality"]["rmse"] for x in report["quality"]), max_p99=max(x["quality"]["p99"] for x in report["quality"]), max_coverage_error=max(x["quality"]["coverage_error"] for x in report["quality"]), max_projected_error=max(x["projected_error_px"] for x in report["quality"]), duplicates_range=[min(duplicates), max(duplicates)], assets=report["assets"])
    (OUT / "summary.json").write_text(json.dumps(result, indent=2) + "\n")
    html = '''<!doctype html><html lang="pt-BR"><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>Compilador de geometria · Ignis</title>
<style>body{background:#11151b;color:#e1e7ed;font:16px system-ui;margin:24px}h1{font-size:27px}p{max-width:1000px;line-height:1.5;color:#bcc8d4}label{display:inline-block;margin:8px 18px 8px 0}select,button{background:#273442;color:white;padding:8px;border:1px solid #586b7e;border-radius:5px}input{vertical-align:middle}.viewer{overflow:auto;background:black;border:1px solid #364757}.pair{display:flex;gap:2px}.panel{flex-shrink:0}img{display:block;max-width:none}.caption{padding:8px;background:#1f2934}table{border-collapse:collapse;margin:20px 0}td,th{padding:8px 16px;border-bottom:1px solid #344353;text-align:right}td:first-child,th:first-child{text-align:left}a{color:#8fcaf7}.muted{font-size:14px}</style>
<h1>Compilar redundância, preservar a superfície</h1>
<p>O compilador recebeu posições, normais, índices e materiais embaralhados. Não recebeu a malha parental nem a história da subdivisão. Câmera e luz variam no teste; as imagens abaixo são capturas de referência do caso 100×, não uma renderização interativa.</p>
<label>Objeto <select id="asset"><option value="stone">Pedra fechada</option><option value="folded">Superfície com dobras</option></select></label>
<label>Câmera <select id="range"><option value="near">Perto</option><option value="far">Longe</option></select></label>
<label>Captura <select id="resolution"><option value="3456">3456 × 2234</option><option value="1728">1728 × 1117</option></select></label>
<label>Órbita e luz <input id="frame" type="range" min="0" max="4" value="0" step="1"><span id="angle"></span></label>
<label><input id="physical" type="checkbox" checked> 1 pixel capturado = 1 pixel físico da tela</label>
<p id="scale" class="muted"></p><div class="viewer"><div class="pair"><div class="panel"><div class="caption">Geometria completa</div><img id="source" alt="Geometria completa"></div><div class="panel"><div class="caption">Fonte + geometria compilada opcional</div><img id="compiled" alt="Geometria compilada"></div></div></div>
<p id="stats"></p><table><thead><tr><th>Subdivisão</th><th>Triângulos fonte → compilado</th><th>GPU fonte</th><th>GPU opcional</th><th>Razão de tempos</th></tr></thead><tbody id="table"></tbody></table>
<p>1×, 2×, 10× e 100× são multiplicadores de triângulos da mesma superfície. Não são triângulos por pixel. Tempos são medianas de lotes na Apple M4 Pro/Metal; incluem os desenhos e a escolha opcional, mas não representam FPS de um jogo completo.</p>
<p id="quality"></p><p>O limite geométrico é de 0,15 pixel físico. Regiões sem certificado mantêm triângulos; projeção inválida usa a fonte. A fonte permanece residente para fallback. Este recorte cobre geometria rígida opaca, posições/normais e três materiais; não certifica texturas UV, animação, sombras ou qualquer malha arbitrária.</p>
<p><a href="results.json">Medições completas</a> · <a href="summary.json">Resumo numérico</a></p><script>const data=__DATA__;
const ids=['asset','range','resolution','frame','physical'];const e=Object.fromEntries(ids.map(k=>[k,document.getElementById(k)]));
function update(){const name=e.asset.value,range=e.range.value,width=Number(e.resolution.value),frame=Number(e.frame.value)*2;const physical=e.physical.checked;const dpr=window.devicePixelRatio||1;const displayWidth=physical?width/dpr:Math.max(200,(window.innerWidth-54)/2);
for(const kind of ['source','compiled']){const img=document.getElementById(kind);img.src=`${name}-${range}-${width}-${frame}-${kind}.png`;img.style.width=`${displayWidth}px`;img.style.height='auto';img.parentElement.style.width=`${displayWidth}px`;}
document.getElementById('angle').textContent=` ${frame*45}°${frame===8?' (retorno)':''}`;document.getElementById('scale').textContent=physical?`DPR ${dpr}. Imagem a ${displayWidth.toFixed(1)} pixels CSS; role horizontalmente para comparar em escala física. `:'Ajustada à janela: há redução da imagem; use a escala física para avaliar detalhes.';
const build=data.builds[`${name}-100`];document.getElementById('stats').textContent=`100×: ${build.source_triangles.toLocaleString('pt-BR')} → ${build.proxy_triangles.toLocaleString('pt-BR')} triângulos. Compilação, validação e escrita: ${(build.compile_validate_write_ms/1000).toFixed(2)} s. Geometria compilada: ${(build.proxy_geometry_bytes/1048576).toFixed(2)} MiB; arquivo incluindo fonte de fallback: ${(build.compiled_bytes/1048576).toFixed(2)} MiB.`;
document.getElementById('table').innerHTML=data.rows.filter(r=>r.name===name&&r.range===range&&r.width===width).map(r=>{const b=data.builds[`${name}-${r.factor}`];return `<tr><td>${r.factor}×</td><td>${b.source_triangles.toLocaleString('pt-BR')} → ${b.proxy_triangles.toLocaleString('pt-BR')}</td><td>${r.source_gpu_ms.toFixed(3)} ms</td><td>${r.compiled_gpu_ms.toFixed(3)} ms</td><td>${r.speedup.toFixed(2)}×</td></tr>`}).join('');
document.getElementById('quality').textContent=`${data.quality_cases} comparações de imagem; ${data.failed} falhas nos limites definidos. RMSE máximo ${data.max_rmse.toFixed(6)}; cobertura divergente máxima ${(data.max_coverage_error*100).toFixed(4)}%. Há diferenças numéricas; isto não é igualdade de todos os pixels nem aceite visual humano.`;}
ids.forEach(k=>e[k].addEventListener('input',update));window.addEventListener('resize',update);update();</script></html>'''
    (OUT / "index.html").write_text(html.replace("__DATA__", json.dumps(result)), encoding="utf-8")
    print(json.dumps({k: v for k, v in result.items() if k not in ("rows", "builds", "assets")}, indent=2))
    return result


if __name__ == "__main__":
    summarize()
