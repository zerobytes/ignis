#!/usr/bin/env python3
"""Paired quality, repeatability and cost of analytic texture footprints."""
import html
import json
import math
from pathlib import Path
from statistics import median

OUT = Path(__file__).resolve().parents[1] / "artifacts/appearance-probe/rich-relief"
MODES = ["source", "previous", "analytic-previous", "regional", "analytic-regional"]


def main():
    data = json.loads((OUT / "analytic-results.json").read_text())
    assert data["sampling"] == "analytic-lod-normal-field" and not data["automatic"]
    quality, timings, returns = (data[k] for k in ("quality", "timings", "returns"))
    assert len(quality) == 138 and len(timings) == 800 and len(returns) == 50
    expected = {(w, c, f) for w in (1728, 3456) for c in range(5)
                for f in range(33 if c == 4 else 9)}
    assert {(r["width"], r["case"], r["frame"]) for r in quality} == expected
    assert {(r["width"], r["case"], r["mode"]) for r in returns} == {
        (w, c, m) for w in (1728, 3456) for c in range(5) for m in MODES}
    assert all(r["pass"] == (r["first_hash"] == r["return_hash"]) for r in returns)
    for row in quality:
        for mode in MODES[1:] + ["material_previous", "material_regional"]:
            q = row[mode]
            assert all(math.isfinite(q[k]) for k in ("rmse", "p99", "max", "coverage_error"))
            assert q["pass"] == (q["rmse"] <= .005 and q["p99"] <= .02 and q["coverage_error"] <= .0005)
        assert row["selection"]["max_error_px"] <= .05
    cohorts = []
    for w in (1728, 3456):
        for case in range(5):
            subset = [r for r in timings if r["width"] == w and r["case"] == case]
            assert len(subset) == 80
            assert {(r["round"], r["slot"]) for r in subset} == {
                (i, s) for i in range(8) for s in range(10)}
            assert all(math.isfinite(r["timing"][k]) and r["timing"][k] >= 0
                       for r in subset for k in ("gpu_ms", "cpu_encode_ms", "total_batch_ms_per_frame"))
            row = {"width": w, "case": case}
            for mode in MODES:
                group = [r for r in subset if r["mode"] == mode]
                assert len(group) == 16 and all(r["timing"]["frames"] == 128 for r in group)
                stats = {k: median(r["timing"][k] for r in group)
                         for k in ("gpu_ms", "cpu_encode_ms", "total_batch_ms_per_frame")}
                duplicates = []
                for round_ in range(8):
                    pair = [r["timing"]["total_batch_ms_per_frame"] for r in group if r["round"] == round_]
                    assert len(pair) == 2
                    duplicates.append(abs(pair[0] - pair[1]) / median(pair) * 100)
                stats["duplicate_percent"] = median(duplicates)
                row[mode] = stats
            for base, candidate in [("previous", "analytic-previous"), ("regional", "analytic-regional")]:
                row[candidate]["relative_cost_percent"] = (
                    row[candidate]["total_batch_ms_per_frame"] / row[base]["total_batch_ms_per_frame"] - 1) * 100
                row[candidate]["rounds_faster"] = sum(
                    median(r["timing"]["total_batch_ms_per_frame"] for r in subset if r["round"] == i and r["mode"] == candidate)
                    < median(r["timing"]["total_batch_ms_per_frame"] for r in subset if r["round"] == i and r["mode"] == base)
                    for i in range(8))
            cohorts.append(row)
    stats = {}
    for mode in MODES[1:] + ["material_previous", "material_regional"]:
        stats[mode] = {"passes": sum(r[mode]["pass"] for r in quality),
                       "max_rmse": max(r[mode]["rmse"] for r in quality),
                       "max_p99": max(r[mode]["p99"] for r in quality),
                       "max_channel": max(r[mode]["max"] for r in quality),
                       "max_coverage_error": max(r[mode]["coverage_error"] for r in quality),
                       "failures": [{k: r[k] for k in ("width", "case", "frame")} for r in quality if not r[mode]["pass"]]}
    return_stats = {mode: sum(r["pass"] for r in returns if r["mode"] == mode) for mode in MODES}
    passes = all(stats[m]["passes"] == 138 for m in ("analytic-previous", "analytic-regional")) and all(r["pass"] for r in returns)
    summary = {"image_and_return_gates_pass": passes, "automatic_policy": False,
               "cohorts": cohorts, "quality": stats, "returns": return_stats,
               "resident_geometry_mib": data["geometry_bytes"] / 1048576,
               "shared_material_mib": data["texture_bytes"] / 1048576,
               "extra_asset_payload_bytes": 0}
    (OUT / "analytic-summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    embedded = json.dumps({"quality": quality, "summary": summary}).replace("<", "\\u003c")
    status = "Gates numéricos e de retorno passaram nesta coleta." if passes else "REPROVADO: existem falhas de imagem ou de retorno. Não promovido."
    document = '''<!doctype html><html lang="pt-BR"><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>Filtragem analítica — Ignis</title>
<style>body{background:#171a1e;color:#e5e8eb;font:16px system-ui;margin:24px}select,button{font:inherit;margin:6px;padding:6px}table{border-collapse:collapse}td,th{padding:8px;border-bottom:1px solid #46515b;text-align:right}th:first-child,td:first-child{text-align:left}.images{display:flex;gap:12px;overflow:auto}.images figure{margin:0;min-width:0;flex:1}.images img{display:block;max-width:100%;height:auto}body.native .images figure{flex:none}body.native .images img{max-width:none}#status{padding:14px;background:#473829}p{max-width:1100px}</style>
<h1>Filtragem analítica da textura</h1><p id="status">STATUS</p>
<p>Mesma geometria, mapas e iluminação. O candidato estima a área da textura coberta pelo pixel usando câmera e orientação compilada da superfície. Não é uma nova política automática. Nuvens/atmosfera não participam.</p>
<label>Resolução <select id="resolution"><option>1728</option><option>3456</option></select></label>
<label>Câmera <select id="case"><option value="0">Perto · 2,7</option><option value="1">6</option><option value="2">16</option><option value="3">Longe · 64</option><option value="4">Aproximar / afastar</option></select></label>
<label>Pose <input id="frame" type="range" min="0" max="8" value="0"></label>
<label>Caminho <select id="mode"><option value="analytic-previous">Anterior analítico</option><option value="analytic-regional">Regional analítico</option><option value="previous">Anterior original</option><option value="regional">Regional original</option></select></label>
<button id="scale">Ver pixels nativos</button><p>As capturas usam a resolução física indicada. Ajustar à largura da página reduz apenas a exibição; não altera o teste.</p>
<div class="images"><figure><figcaption>Fonte completa · shader original</figcaption><img id="reference"></figure><figure><figcaption id="caption"></figcaption><img id="candidate"></figure></div>
<pre id="quality"></pre><h2>Tempo total pareado · ms/frame</h2><p>Mesma coleta, 8 rodadas e controles duplicados. Sinal positivo na variação significa maior custo. GPU e CPU podem se sobrepor; não somá-los.</p>
<table><thead><tr><th>Pixels / câmera</th><th>Anterior</th><th>Analítico</th><th>Variação</th><th>Regional</th><th>Analítico</th><th>Variação</th></tr></thead><tbody id="cost"></tbody></table>
<p id="memory"></p><p>Resultados experimentais neste hardware. Aprovação numérica não é identidade de pixels, aceite humano ou prova de qualidade temporal geral. A fonte original continua como controle; nenhum material foi promovido.</p>
<script>const data=DATA;const $=id=>document.getElementById(id);const names={'previous':'Anterior original','analytic-previous':'Anterior analítico','regional':'Regional original','analytic-regional':'Regional analítico'};
function update(){const w=+$('resolution').value,c=+$('case').value;$('frame').max=c===4?32:8;const f=Math.min(+$('frame').value,+$('frame').max);$('frame').value=f;const m=$('mode').value;const r=data.quality.find(r=>r.width===w&&r.case===c&&r.frame===f);const q=r[m];$('reference').src=`analytic-${w}-${c}-${f}-source.png`;$('candidate').src=`analytic-${w}-${c}-${f}-${m}.png`;$('caption').textContent=names[m];for(const id of ['reference','candidate'])$(id).style.width=document.body.classList.contains('native')?`${w/(window.devicePixelRatio||1)}px`:'100%';$('quality').textContent=`${q.pass?'Passa':'FALHA'} · RMSE ${q.rmse.toFixed(6)} · p99 ${q.p99.toFixed(6)} · cobertura ${q.coverage_error.toExponential(3)}\nRetornos exatos: ${data.summary.returns[m]}/10; fonte: ${data.summary.returns.source}/10.\nDistância ${r.distance.toFixed(3)}, regional ${r.selection.triangles}T em ${r.selection.ranges.length} draws.`;}
for(const id of ['resolution','case','frame','mode'])$(id).addEventListener('input',update);$('scale').onclick=()=>{document.body.classList.toggle('native');$('scale').textContent=document.body.classList.contains('native')?'Ajustar à página':'Ver pixels nativos';update();};window.addEventListener('resize',update);
$('cost').innerHTML=data.summary.cohorts.map(r=>`<tr><td>${r.width} / ${r.case}</td><td>${r.previous.total_batch_ms_per_frame.toFixed(4)}</td><td>${r['analytic-previous'].total_batch_ms_per_frame.toFixed(4)}</td><td>${r['analytic-previous'].relative_cost_percent.toFixed(1)}%</td><td>${r.regional.total_batch_ms_per_frame.toFixed(4)}</td><td>${r['analytic-regional'].total_batch_ms_per_frame.toFixed(4)}</td><td>${r['analytic-regional'].relative_cost_percent.toFixed(1)}%</td></tr>`).join('');$('memory').textContent=`Geometria residente: ${data.summary.resident_geometry_mib.toFixed(3)} MiB; mapas compartilhados: ${data.summary.shared_material_mib.toFixed(3)} MiB. Payload adicional do candidato: 0 B. Targets, pipelines, driver e cópias CPU excluídos.`;update();</script></html>'''
    document = document.replace("STATUS", html.escape(status)).replace("DATA", embedded)
    (OUT / "analytic.html").write_text(document)
    print(json.dumps({"image_and_return_gates_pass": passes,
                      "quality_passes": {m: q["passes"] for m, q in stats.items()},
                      "returns": return_stats, "report": str(OUT / "analytic.html")}, indent=2))


if __name__ == "__main__":
    main()
