#!/usr/bin/env python3
"""Report coordinated regions, redundant density and grouped submission separately."""

import json
from pathlib import Path
from statistics import median

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "artifacts/appearance-probe/coordinated-regions"


def main():
    data = json.loads((OUT / "results.json").read_text())
    builds = [
        json.loads((OUT / f"build-{f}.json").read_text()) for f in [1, 2, 10, 100]
    ]
    quality = []
    cohorts = []
    for factor in [1, 2, 10, 100]:
        rows = [x for x in data["quality"] if x["factor"] == factor]
        assert len(rows) == 36 and all(
            x["indirect_identical"] for x in rows
        )
        item = {
            "factor": factor,
            "comparisons": len(rows),
            "minimum_triangles": min(x["selection"]["triangles"] for x in rows),
            "max_ranges": max(len(x["selection"]["ranges"]) for x in rows),
            "old_minimum_triangles": min(x["old_selection"]["triangles"] for x in rows),
        }
        for mode in [
            "density_reference",
            "recovered",
            "old",
            "geometry",
            "coordinated",
        ]:
            item[mode] = {
                "failures": sum(not x[mode]["pass"] for x in rows),
                **{
                    key: max(x[mode][key] for x in rows)
                    for key in ["rmse", "p99", "max", "coverage_error"]
                },
            }
        quality.append(item)
        for width in [1728, 3456]:
            for distance in sorted({x["distance"] for x in data["timings"]}):
                cohort_rows = [x for x in rows if x["width"] == width and abs(x["distance"] - distance) < 0.001]
                c = {"factor": factor, "width": width, "distance": distance,
                     "density_control_pass": all(x["density_reference"]["pass"] for x in cohort_rows),
                     "coordinated_quality_pass": all(x["coordinated"]["pass"] for x in cohort_rows)}
                for mode in ["source", "recovered", "old", "coordinated", "indirect"]:
                    group = [
                        x
                        for x in data["timings"]
                        if x["factor"] == factor
                        and x["width"] == width
                        and x["distance"] == distance
                        and x["mode"] == mode
                    ]
                    assert len(group) == 16
                    c[mode] = {
                        key: median(x["timing"][key] for x in group)
                        for key in [
                            "gpu_ms",
                            "cpu_encode_ms",
                            "total_batch_ms_per_frame",
                        ]
                    }
                    duplicates = []
                    for round_id in range(8):
                        pair = [
                            x["timing"]["total_batch_ms_per_frame"]
                            for x in group
                            if x["round"] == round_id
                        ]
                        duplicates.append(abs(pair[0] - pair[1]) / min(pair) * 100)
                    c[mode]["duplicate_wall_percent"] = median(duplicates)
                cohorts.append(c)
    summary = {"builds": builds, "quality": quality, "cohorts": cohorts}
    (OUT / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    embedded = {"summary": summary, "rows": data["quality"]}
    html = r"""<!doctype html><html lang="pt-BR"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Fronteiras coordenadas • densidade 1–100×</title><style>body{font:16px system-ui;margin:24px;background:#111722;color:#e9eff7}h1{font-size:26px}p{line-height:1.5;max-width:1100px}.note{color:#b0c2d7}label{display:inline-block;margin:8px}select,button{font:inherit;padding:6px;background:#26374e;color:white;border:1px solid #60748d}input{accent-color:#77dabb}a{color:#77dabb}section{display:flex;gap:16px}figure{margin:0;min-width:0;flex:1}.view{height:460px;overflow:auto;background:black}img{display:block;max-width:none}figcaption{margin:10px 0}table{border-collapse:collapse;font-size:14px}td,th{border:1px solid #455670;padding:8px;text-align:right}td:first-child,th:first-child{text-align:left}#metric{white-space:pre-wrap;font:14px monospace}@media(max-width:800px){section{display:block}.view{height:300px}}</style>
<body><h1>Fronteiras coordenadas, mesma superfície, mais subdivisões</h1><p>A pedra recebe 1×, 2×, 10× e 100× triângulos por subdivisão baricêntrica, sem relevo intencional novo. Isso não garante igualdade de rasterização: controles reprovados são destacados. O compilador recebe a malha embaralhada e remove redundância antes de preparar regiões e campos de normais. Regiões coordenadas liberam bordas internas de uniões; o contorno externo continua protegido. O painel exibe capturas offscreen, não render em tempo real no navegador.</p>
<label>Subdivisão <select id="factor"><option>1</option><option>2</option><option>10</option><option selected>100</option></select></label><label>Pixels <select id="resolution"><option value="1728">1728 × 1117</option><option value="3456" selected>3456 × 2234</option></select></label><label>Câmera <select id="distance"><option value="0">Perto (2,7)</option><option value="1">Longe (64)</option></select></label><label>Órbita / luz <input id="frame" type="range" min="0" max="3" value="0"></label>
<label>Candidato <select id="mode"><option value="recovered">Compilador anterior: remover redundância</option><option value="old">Regiões anteriores + campo</option><option value="coordinated" selected>Regiões coordenadas + campo</option><option value="geometry">Coordenadas sem campo (controle)</option></select></label><label><input id="fit" type="checkbox" checked>Visão geral</label>
<p id="outcome"></p><p class="note" id="scale"></p><section><figure><figcaption>Todos os triângulos da entrada</figcaption><div class="view"><img id="source" alt="Geometria completa"></div></figure><figure><figcaption id="candidateTitle">Candidato</figcaption><div class="view"><img id="candidate" alt="Caminho compilado"></div></figure></section><p id="metric"></p>
<h2>O custo cresce com os triângulos de entrada?</h2><p class="note">Tabela acompanha resolução e distância selecionadas. Medianas de 16 lotes por caminho, 128 frames/lote, câmera e luz variáveis. Parede inclui CPU, GPU, submissão/upload de argumentos e espera/readback do benchmark; não é FPS de gameplay. O multi-draw usa vários desenhos no backend Metal, apesar de uma chamada agrupada. Os shaders e a seleção direta/indireta produziram pixels idênticos.</p><table><thead><tr><th>Entrada</th><th>GPU fonte</th><th>GPU anterior</th><th>GPU coordenado</th><th>GPU indireto</th><th>Parede anterior → coord. → indireto</th></tr></thead><tbody id="timings"></tbody></table>
<h2>Qualidade e tamanho</h2><p class="note">Limites mantidos: RMSE≤0,005, p99≤0,02, cobertura≤0,0005. Passar não significa pixels idênticos. Controle de subdivisão conferido contra a pedra 1×; as reprovações invalidam a comparação de densidade da respectiva coorte, mesmo quando o candidato passa contra sua própria fonte. Mesmo um candidato que falhe continua na tabela para diagnóstico; não foi promovido automaticamente.</p><table><thead><tr><th>Fator</th><th>Triângulos de entrada</th><th>Após compilador anterior</th><th>Menor seleção coordenada</th><th>Falhas controle densidade / anterior / campo antigo / coordenado</th><th>Geometria+UV+campo coordenado</th></tr></thead><tbody id="qualityTable"></tbody></table>
<p><a href="summary.json">Resumo completo</a> · <a href="results.json">Medições brutas</a> · Fonte: <a href="https://polyhaven.com/a/boulder_01">Boulder 01</a>, Rico Cilliers, CC0. Powered by Poly Haven. Apenas geometria/normais; material diagnóstico compartilhado.</p>
<script>const data=__DATA__;const $=id=>document.getElementById(id),distances=[2.7,64];
function update(){const f=+$('factor').value,w=+$('resolution').value,d=+$('distance').value,frame=+$('frame').value*2,mode=$('mode').value;const row=data.rows.find(x=>x.factor===f&&x.width===w&&Math.abs(x.distance-distances[d])<.001&&x.frame===frame),q=row[mode];$('source').src=`${f}-${w}-${d}-${frame}-source.png`;$('candidate').src=`${f}-${w}-${d}-${frame}-${mode}.png`;$('candidateTitle').textContent=$('mode').selectedOptions[0].text;for(const id of ['source','candidate'])$(id).style.width=$('fit').checked?'100%':`${w/devicePixelRatio}px`;$('scale').textContent=$('fit').checked?'Visão geral reduzida. Desative para escala nominal de pixel físico.':`1 pixel renderizado por pixel físico nominal (DPR ${devicePixelRatio}); zoom do navegador em 100%. Role para explorar.`;const s=data.summary.quality.find(x=>x.factor===f);$('outcome').textContent=`${s.density_reference.failures}/${s.comparisons} falhas no controle de densidade; ${s.coordinated.failures}/${s.comparisons} falhas no caminho coordenado; até ${s.max_ranges} ranges de desenho. Esses resultados medem redundância geométrica, não 100 vezes mais detalhe visível.`;$('metric').textContent=`Entrada ${row.source_triangles} T; recuperado ${row.recovered_triangles} T; regiões anteriores ${row.old_selection.triangles} T; coordenadas ${row.selection.triangles} T / ${row.selection.ranges.length} ranges\nRMSE ${q.rmse.toFixed(6)}; p99 ${q.p99.toFixed(6)}; máximo ${q.max.toFixed(6)}; cobertura ${q.coverage_error.toFixed(6)}; ${q.pass?'PASSOU os limites':'FALHOU'}\nSubdivisão vs pedra-base: RMSE ${row.density_reference.rmse.toFixed(6)}. Direto/indireto idênticos: ${row.indirect_identical}.`;$('timings').innerHTML=data.summary.cohorts.filter(c=>c.width===w&&Math.abs(c.distance-distances[d])<.001).map(c=>`<tr><td>${c.factor}×${c.density_control_pass?'':' — controle INVÁLIDO'}</td><td>${c.source.gpu_ms.toFixed(4)}</td><td>${c.recovered.gpu_ms.toFixed(4)}</td><td>${c.coordinated.gpu_ms.toFixed(4)}</td><td>${c.indirect.gpu_ms.toFixed(4)}</td><td>${c.recovered.total_batch_ms_per_frame.toFixed(4)} → ${c.coordinated.total_batch_ms_per_frame.toFixed(4)} → ${c.indirect.total_batch_ms_per_frame.toFixed(4)}</td></tr>`).join('');}
for(const id of ['factor','resolution','distance','frame','mode','fit'])$(id).addEventListener('input',update);window.addEventListener('resize',update);for(const id of ['source','candidate'])$(id).onload=()=>{if(!$('fit').checked){const el=$(id),view=el.parentElement;view.scrollLeft=(el.clientWidth-view.clientWidth)/2;view.scrollTop=(el.clientHeight-view.clientHeight)/2;}};
$('qualityTable').innerHTML=data.summary.builds.map(b=>{const q=data.summary.quality.find(x=>x.factor===b.factor),v=b.variants.find(x=>x.name==='coordinated');return `<tr><td>${b.factor}×</td><td>${b.source_triangles}</td><td>${b.recovered_triangles}</td><td>${q.minimum_triangles}</td><td>${q.density_reference.failures} / ${q.recovered.failures} / ${q.old.failures} / ${q.coordinated.failures}</td><td>${((v.geometry_uv_bytes+v.texture_bytes)/1048576).toFixed(2)} MiB</td></tr>`}).join('');update();</script></body></html>"""
    (OUT / "index.html").write_text(html.replace("__DATA__", json.dumps(embedded)))
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
