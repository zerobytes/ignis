#!/usr/bin/env python3
"""Generate the measured irregular-mesh report and local comparison viewer."""

import json
from pathlib import Path
from statistics import median

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "artifacts/appearance-probe/irregular-lod"


def main():
    r = json.loads((OUT / "results.json").read_text())
    build = json.loads((OUT / "boulder.json").read_text())
    imported = json.loads((OUT / "import.json").read_text())
    accepted = [
        x
        for x in r["quality"]
        if not x.get("negative_control") and not x.get("geometry_only_control")
    ]
    controls = [x for x in r["quality"] if x.get("geometry_only_control")]
    assert r["failed"] == 0 and all(x["quality"]["pass"] for x in accepted)
    assert len(accepted) == 158 and len(controls) == 60
    assert sum(x.get("return_identical", False) for x in accepted) == 2
    cohorts = []
    for width in (1728, 3456):
        for distance in sorted({x["distance"] for x in r["timings"]}):
            group = [
                x
                for x in r["timings"]
                if x["width"] == width and x["distance"] == distance
            ]
            item = {"width": width, "distance": distance}
            for adaptive, name in ((False, "source"), (True, "adaptive")):
                values = [x["timing"] for x in group if x["adaptive"] == adaptive]
                item[name] = {
                    key: median(x[key] for x in values)
                    for key in ("gpu_ms", "cpu_encode_ms", "total_batch_ms_per_frame")
                }
                # Duplicate controls in each alternating round estimate run noise.
                duplicate = []
                for round_id in range(8):
                    pair = [
                        x["timing"]["gpu_ms"]
                        for x in group
                        if x["adaptive"] == adaptive and x["round"] == round_id
                    ]
                    duplicate.append(abs(pair[0] - pair[1]) / min(pair) * 100)
                item[name]["duplicate_gpu_delta_percent"] = median(duplicate)
            item["gpu_delta_percent"] = (
                item["adaptive"]["gpu_ms"] / item["source"]["gpu_ms"] - 1
            ) * 100
            item["wall_delta_percent"] = (
                item["adaptive"]["total_batch_ms_per_frame"]
                / item["source"]["total_batch_ms_per_frame"]
                - 1
            ) * 100
            cohorts.append(item)
    summary = {
        "accepted_comparisons": len(accepted),
        "identical_comparisons": sum(x["quality"]["max"] == 0 for x in accepted),
        "geometry_only_failures": sum(not x["quality"]["pass"] for x in controls),
        "geometry_only_comparisons": len(controls),
        "minimum_accepted_triangles": min(
            x["selection"]["triangles"] for x in accepted
        ),
        "source_triangles": r["source_triangles"],
        "source_geometry_bytes": imported["vertices"] * 24 + imported["triangles"] * 12,
        "compiled_geometry_bytes": r["gpu_geometry_bytes"],
        "file_bytes": r["file_bytes"],
        "build_ms": build["build_validate_write_ms"],
        "cohorts": cohorts,
    }
    (OUT / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    data = {"summary": summary, "quality": r["quality"]}
    template = r"""<!doctype html><html lang="pt-BR"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Pedra externa • regiões compiladas</title>
<style>body{font:16px system-ui;background:#101319;color:#e5eaf2;margin:24px}h1{font-size:26px}p{max-width:1000px;line-height:1.5}button,select,input{font:inherit;accent-color:#7bd6bb}button,select{background:#273044;color:white;border:1px solid #52617b;padding:6px}label{display:inline-block;margin:8px}section{display:flex;gap:16px}figure{margin:0;flex:1;min-width:0}figcaption{padding:10px 0}.viewport{overflow:auto;background:#07090d;height:440px}img{display:block;max-width:none}body.fit img{width:100%;height:auto}table{border-collapse:collapse;font-size:14px}th,td{border:1px solid #40506a;padding:8px;text-align:right}th:first-child,td:first-child{text-align:left}.note{color:#a6b9cc}a{color:#83dcca}#metrics{white-space:pre-wrap;font:14px monospace}@media(max-width:800px){section{display:block}.viewport{height:300px}}</style>
<body><h1>Pedra externa: reduzir a malha sem apagar sua aparência</h1>
<p>Malha irregular de 66.122 triângulos, sem genealogia. Geometria e normais importadas; material de diagnóstico compartilhado, sem texturas/UV originais. Capturas Metal offscreen com câmera e luz variáveis. Este painel percorre capturas; não executa o render no navegador.</p>
<p id="outcome"></p>
<label>Resolução <select id="resolution"><option value="1728">1728 × 1117</option><option value="3456">3456 × 2234</option></select></label>
<label>Distância <input id="distance" type="range" min="0" max="5" value="0" step="1"><span id="distanceValue"></span></label>
<label>Órbita / luz <input id="frame" type="range" min="0" max="3" value="0" step="1"></label>
<label>Comparar <select id="mode"><option value="adaptive">Adaptativo conservador</option><option value="geometry-only">Controle: só erro geométrico (pode falhar)</option></select></label>
<label><input id="fit" type="checkbox"> Ajustar imagens à coluna</label><button id="animate">Percorrer distâncias</button>
<p class="note" id="scale"></p>
<section><figure><figcaption>Geometria completa</figcaption><div class="viewport"><img id="source" alt="Pedra com geometria completa"></div></figure><figure><figcaption id="candidateTitle">Adaptativo</figcaption><div class="viewport"><img id="candidate" alt="Pedra com seleção regional"></div></figure></section>
<p id="metrics"></p><h2>Custo medido</h2><p class="note">Medianas de 16 lotes por caminho/caso, 128 frames por lote. Janela GPU inclui os desenhos; CPU inclui seleção, codificação e submissão. Parede inclui espera/readback do benchmark, não é FPS de um jogo. Ambos usam o mesmo buffer combinado residente. Δ positivo = mais lento. Sem ganho relevante de triângulos, diferenças pequenas devem ser comparadas à dispersão dos controles duplicados.</p><table><thead><tr><th>Resolução / distância</th><th>GPU completa → adaptativa (ms)</th><th>Δ GPU</th><th>Ruído duplicado GPU</th><th>CPU (ms)</th><th>Parede (ms)</th></tr></thead><tbody id="timings"></tbody></table>
<p>Fonte: <a href="https://polyhaven.com/a/boulder_01">Boulder 01, Rico Cilliers, CC0</a>. Powered by Poly Haven. <a href="results.json">Medições brutas</a> · <a href="summary.json">Resumo</a> · <a href="import.json">Proveniência</a></p>
<script>const data=__DATA__;
const $=id=>document.getElementById(id), distances=[2.7,4,8,16,32,64];let timer=null;
const s=data.summary;
$('outcome').textContent=`${s.accepted_comparisons} comparações conservadoras passaram; ${s.identical_comparisons} foram idênticas. Menor seleção: ${s.minimum_accepted_triangles.toLocaleString('pt-BR')} triângulos. Ignorar o erro das normais falhou em ${s.geometry_only_failures}/${s.geometry_only_comparisons} controles. Buffers: ${(s.source_geometry_bytes/1048576).toFixed(2)} MiB fonte → ${(s.compiled_geometry_bytes/1048576).toFixed(2)} MiB com níveis. Não houve redução útil neste asset.`;
function update(){let w=+$('resolution').value,d=+$('distance').value,f=+$('frame').value*2,mode=$('mode').value;let q=data.quality.find(x=>x.width===w&&Math.abs(x.distance-distances[d])<.001&&x.frame===f&&!x.trajectory&&!x.negative_control&&!!x.geometry_only_control===(mode==='geometry-only'));$('source').src=`${w}-${d}-${f}-source.png`;$('candidate').src=`${w}-${d}-${f}-${mode}.png`;$('distanceValue').textContent=distances[d];$('candidateTitle').textContent=$('mode').selectedOptions[0].text;document.body.classList.toggle('fit',$('fit').checked);for(let id of ['source','candidate'])$(id).style.width=$('fit').checked?'100%':`${w/devicePixelRatio}px`;$('scale').textContent=$('fit').checked?'Visão geral reduzida: não serve para julgar detalhes por pixel.':`1 pixel renderizado = 1 pixel físico nominal (DPR ${devicePixelRatio}). Use as barras de rolagem para detalhes; mantenha o zoom do navegador em 100%.`;$('metrics').textContent=`${q.selection.triangles} triângulos / ${q.selection.ranges.length} desenhos / ${q.selection.reduced} regiões reduzidas\nErro projetado ≤ ${q.selection.max_error_px.toFixed(4)} px; RMSE ${q.quality.rmse.toFixed(6)}; p99 ${q.quality.p99.toFixed(6)}; máximo ${q.quality.max.toFixed(6)}; cobertura ${q.quality.coverage_error.toFixed(6)}; ${q.quality.pass?'PASSOU os limites medidos':'FALHOU'}`;}
for(let id of ['resolution','distance','frame','mode','fit'])$(id).addEventListener('input',update);window.addEventListener('resize',update);
$('animate').onclick=()=>{if(timer){clearInterval(timer);timer=null;$('animate').textContent='Percorrer distâncias';}else{let step=1;timer=setInterval(()=>{let x=+$('distance').value;if(x===5)step=-1;if(x===0)step=1;$('distance').value=x+step;update();},900);$('animate').textContent='Pausar';}};
$('timings').innerHTML=s.cohorts.map(c=>{let a=c.source,b=c.adaptive;return `<tr><td>${c.width} / ${c.distance.toFixed(1)}</td><td>${a.gpu_ms.toFixed(3)} → ${b.gpu_ms.toFixed(3)}</td><td>${c.gpu_delta_percent.toFixed(1)}%</td><td>${a.duplicate_gpu_delta_percent.toFixed(1)}% / ${b.duplicate_gpu_delta_percent.toFixed(1)}%</td><td>${a.cpu_encode_ms.toFixed(3)} → ${b.cpu_encode_ms.toFixed(3)}</td><td>${a.total_batch_ms_per_frame.toFixed(3)} → ${b.total_batch_ms_per_frame.toFixed(3)}</td></tr>`}).join('');update();</script></body></html>"""
    (OUT / "index.html").write_text(template.replace("__DATA__", json.dumps(data)))
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
