#!/usr/bin/env python3
"""Summarize measured normal-field variants; no per-pose winner selection."""

import json
from pathlib import Path
from statistics import median

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "artifacts/appearance-probe/normal-field"


def main():
    r = json.loads((OUT / "results.json").read_text())
    summaries = []
    timings = []
    for memory in r["memory"]:
        name = memory["name"]
        rows = [x for x in r["quality"] if x["name"] == name]
        assert len(rows) == 122 and all(x["prior"]["pass"] for x in rows)
        summary = dict(memory)
        summary["comparisons"] = len(rows)
        summary["minimum_triangles"] = min(x["selection"]["triangles"] for x in rows)
        summary["max_draws"] = max(len(x["selection"]["ranges"]) for x in rows)
        for mode in ["prior", "reduced", "field"]:
            summary[mode] = {
                "failures": sum(not x[mode]["pass"] for x in rows),
                **{
                    key: max(x[mode][key] for x in rows)
                    for key in ["rmse", "p99", "max", "coverage_error"]
                },
            }
        summaries.append(summary)
        for width in [1728, 3456]:
            for distance in sorted({x["distance"] for x in r["timings"]}):
                cohort = {"name": name, "width": width, "distance": distance}
                for mode in ["source", "prior", "reduced", "field"]:
                    group = [
                        x
                        for x in r["timings"]
                        if x["name"] == name
                        and x["width"] == width
                        and x["distance"] == distance
                        and x["mode"] == mode
                    ]
                    assert len(group) == 16
                    cohort[mode] = {
                        key: median(x["timing"][key] for x in group)
                        for key in [
                            "gpu_ms",
                            "cpu_encode_ms",
                            "total_batch_ms_per_frame",
                        ]
                    }
                    duplicate = []
                    for round_id in range(8):
                        pair = [
                            x["timing"]["total_batch_ms_per_frame"]
                            for x in group
                            if x["round"] == round_id
                        ]
                        duplicate.append(abs(pair[0] - pair[1]) / min(pair) * 100)
                    cohort[mode]["duplicate_wall_percent"] = median(duplicate)
                timings.append(cohort)
    result = {"variants": summaries, "cohorts": timings}
    (OUT / "summary.json").write_text(json.dumps(result, indent=2) + "\n")
    data = {"summary": result, "quality": r["quality"]}
    html = r"""<!doctype html><html lang="pt-BR"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Campo de normais • compilação offline</title>
<style>body{margin:24px;background:#10151d;color:#e5edf5;font:16px system-ui}h1{font-size:27px}p{line-height:1.5;max-width:1100px}a{color:#70d6b6}label{display:inline-block;margin:8px}select,button{font:inherit;background:#28374b;color:white;padding:5px;border:1px solid #60738d}input{accent-color:#6edbbb}.note{color:#adbed0}section{display:flex;gap:16px}figure{margin:0;min-width:0;flex:1}figcaption{margin:10px 0}.view{height:460px;overflow:auto;background:black}img{display:block;max-width:none}body.fit img{width:100%}table{border-collapse:collapse;font-size:14px}th,td{border:1px solid #45546a;padding:8px;text-align:right}td:first-child,th:first-child{text-align:left}#metric{white-space:pre-wrap;font:14px monospace}@media(max-width:800px){section{display:block}.view{height:300px}}</style>
<body class="fit"><h1>Separar a forma da resposta fina à luz</h1>
<p>O campo de normais foi compilado offline da pedra original. A malha continua definindo forma, profundidade e contorno; o campo fornece a direção da superfície para a iluminação dinâmica. Mesma seleção de triângulos com e sem campo. Capturas Metal pré-renderizadas: este painel não executa o renderer no navegador.</p>
<p id="summary"></p><label>Campo <select id="variant"><option value="boulder-32">32 × 32 / região</option><option value="boulder-64" selected>64 × 64 / região</option><option value="boulder-128">128 × 128 / região</option></select></label>
<label>Resolução <select id="resolution"><option value="1728">1728 × 1117</option><option value="3456">3456 × 2234</option></select></label>
<label>Distância <input id="distance" type="range" min="0" max="3" value="2"><span id="dvalue"></span></label>
<label>Órbita / luz <input id="frame" type="range" min="0" max="3" value="1"></label>
<label>Candidato <select id="mode"><option value="field">Malha reduzida + campo</option><option value="reduced">Mesma malha sem campo</option></select></label>
<label><input id="fit" type="checkbox" checked>Visão geral</label><button id="animate">Percorrer distâncias</button>
<p class="note" id="scale"></p><section><figure><figcaption>Geometria completa</figcaption><div class="view"><img id="source" alt="Pedra original"></div></figure><figure><figcaption id="candidateTitle">Malha reduzida + campo</figcaption><div class="view"><img id="candidate" alt="Pedra com representação candidata"></div></figure></section><p id="metric"></p>
<h2>Qualidade e memória</h2><p class="note">122 comparações por campo, incluindo órbitas, câmera perto/longe e trajetória de ida e volta. Gates RMSE≤0,005, p99≤0,02, cobertura≤0,0005. Passar nesses limites não significa pixels idênticos. Cada grade é uma variante completa; não somar sua memória como se todas fossem obrigatórias no jogo.</p><table><thead><tr><th>Campo</th><th>Falhas sem → com campo</th><th>RMSE máximo com campo</th><th>p99 máximo</th><th>Textura</th><th>Geometria + UV + textura</th></tr></thead><tbody id="qualityTable"></tbody></table>
<h2>Custo no experimento</h2><p class="note">Medianas de 16 lotes por caminho/caso, 128 frames/lote. Parede inclui seleção, codificação, GPU e espera/readback em lote. O anterior usa o limite de normais conservador. O candidato pode falhar na qualidade; nenhuma linha é escolhida como vencedora automaticamente. Memória exclui targets, staging, cópias CPU e driver.</p><table><thead><tr><th>Resolução / distância</th><th>GPU anterior → campo (ms)</th><th>CPU anterior → campo (ms)</th><th>Parede anterior → campo (ms)</th><th>Ruído duplicado de parede</th></tr></thead><tbody id="timings"></tbody></table>
<p>Fonte <a href="https://polyhaven.com/a/boulder_01">Boulder 01</a>, Rico Cilliers, CC0. Powered by Poly Haven. Apenas geometria/normais importadas; mesmo material diagnóstico, sem texturas originais. <a href="results.json">Dados brutos</a> · <a href="summary.json">Resumo</a></p>
<script>const data=__DATA__;const $=id=>document.getElementById(id),distances=[2.7,8,32,64];let timer=null;
function update(){const name=$('variant').value,w=+$('resolution').value,d=+$('distance').value,f=+$('frame').value*2,mode=$('mode').value;const row=data.quality.find(x=>x.name===name&&x.width===w&&!x.trajectory&&Math.abs(x.distance-distances[d])<.001&&x.frame===f);const q=row[mode],s=data.summary.variants.find(x=>x.name===name);$('source').src=`${name}-${w}-${d}-${f}-source.png`;$('candidate').src=`${name}-${w}-${d}-${f}-${mode}.png`;$('dvalue').textContent=distances[d];$('candidateTitle').textContent=$('mode').selectedOptions[0].text;document.body.classList.toggle('fit',$('fit').checked);for(const id of ['source','candidate'])$(id).style.width=$('fit').checked?'100%':`${w/devicePixelRatio}px`;$('scale').textContent=$('fit').checked?'Visão geral reduzida. Desative para inspecionar pixels físicos.':`Escala nominal 1 pixel renderizado/pixel físico, DPR ${devicePixelRatio}. Use zoom do navegador em 100% e role para explorar.`;$('metric').textContent=`${row.selection.triangles} triângulos / ${row.selection.ranges.length} desenhos / ${row.selection.reduced} regiões reduzidas\nRMSE ${q.rmse.toFixed(6)}; p99 ${q.p99.toFixed(6)}; máximo ${q.max.toFixed(6)}; cobertura ${q.coverage_error.toFixed(6)}; ${q.pass?'PASSOU os gates medidos':'FALHOU'}`;$('summary').textContent=`Nesta grade: ${s.field.failures}/${s.comparisons} falhas com campo, contra ${s.reduced.failures} sem ele. Redução máxima de ${(100*(1-s.minimum_triangles/66122)).toFixed(2)}% dos triângulos; até ${s.max_draws} desenhos. A qualidade recuperada deve ser avaliada junto com o custo total.`;$('timings').innerHTML=data.summary.cohorts.filter(c=>c.name===name).map(c=>{let a=c.prior,b=c.field;return `<tr><td>${c.width} / ${c.distance.toFixed(1)}</td><td>${a.gpu_ms.toFixed(4)} → ${b.gpu_ms.toFixed(4)}</td><td>${a.cpu_encode_ms.toFixed(4)} → ${b.cpu_encode_ms.toFixed(4)}</td><td>${a.total_batch_ms_per_frame.toFixed(4)} → ${b.total_batch_ms_per_frame.toFixed(4)}</td><td>${a.duplicate_wall_percent.toFixed(1)}% / ${b.duplicate_wall_percent.toFixed(1)}%</td></tr>`}).join('');}
for(const id of ['variant','resolution','distance','frame','mode','fit'])$(id).addEventListener('input',update);window.addEventListener('resize',update);
for(const id of ['source','candidate'])$(id).onload=()=>{if(!$('fit').checked){const el=$(id),view=el.parentElement;view.scrollLeft=(el.clientWidth-view.clientWidth)/2;view.scrollTop=(el.clientHeight-view.clientHeight)/2;}};
$('animate').onclick=()=>{if(timer){clearInterval(timer);timer=null;$('animate').textContent='Percorrer distâncias';}else{let step=1;timer=setInterval(()=>{let d=+$('distance').value;if(d===3)step=-1;if(d===0)step=1;$('distance').value=d+step;update();},1000);$('animate').textContent='Pausar';}};
$('qualityTable').innerHTML=data.summary.variants.map(s=>`<tr><td>${s.name}</td><td>${s.reduced.failures} → ${s.field.failures} / ${s.comparisons}</td><td>${s.field.rmse.toFixed(6)}</td><td>${s.field.p99.toFixed(6)}</td><td>${(s.texture_bytes/1048576).toFixed(2)} MiB</td><td>${((s.texture_bytes+s.gpu_geometry_uv_bytes)/1048576).toFixed(2)} MiB</td></tr>`).join('');update();</script></body></html>"""
    (OUT / "index.html").write_text(html.replace("__DATA__", json.dumps(data)))
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
