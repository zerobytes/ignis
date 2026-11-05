#!/usr/bin/env python3
"""Source-only geometric tangent experiment; generate a report even on rejection."""
import hashlib
import json
import math
from pathlib import Path
from statistics import median

OUT = Path(__file__).resolve().parents[1] / 'artifacts/appearance-probe/rich-relief'
MODES = ['implicit', 'analytic-lod', 'geometric-lod', 'geometric-grad']


def main():
    raw = (OUT / 'tangent-results.json').read_bytes()
    data = json.loads(raw)
    quality, returns, repeated, timings = (data[k] for k in ('quality', 'returns', 'repeated', 'timings'))
    assert len(quality) == 138 and len(returns) == 40 and len(repeated) == 512 and len(timings) == 640
    assert {(r['width'], r['case'], r['frame']) for r in quality} == {
        (w, c, f) for w in [1728, 3456] for c in range(5) for f in range(33 if c == 4 else 9)}
    assert {(r['width'], r['case'], r['mode']) for r in returns} == {
        (w, c, m) for w in [1728, 3456] for c in range(5) for m in MODES}
    assert all(r['pass'] == (r['first_hash'] == r['return_hash']) for r in returns)
    assert {(r['width'], r['case'], r['turn'], r['mode']) for r in repeated} == {
        (w, c, t, m) for w in [1728, 3456] for c in [0, 3] for t in range(32) for m in MODES}
    for row in quality:
        for m in MODES[1:] + ['geometry_vs_shading', 'gradient_vs_scalar']:
            q = row[m]
            assert all(math.isfinite(q[k]) for k in ['rmse', 'p99', 'max', 'coverage_error'])
            assert q['pass'] == (q['rmse'] <= .005 and q['p99'] <= .02 and q['coverage_error'] <= .0005)
    stats = {m: {'passes': sum(r[m]['pass'] for r in quality),
                 'max_rmse': max(r[m]['rmse'] for r in quality),
                 'max_p99': max(r[m]['p99'] for r in quality),
                 'max_channel': max(r[m]['max'] for r in quality),
                 'max_coverage_error': max(r[m]['coverage_error'] for r in quality),
                 'failures': [{k: r[k] for k in ['width', 'case', 'frame']} for r in quality if not r[m]['pass']]}
             for m in MODES[1:] + ['geometry_vs_shading', 'gradient_vs_scalar']}
    repetitions = {m: {'unchanged': sum(r['changed_pixels'] == 0 for r in repeated if r['mode'] == m),
                       'max_changed_pixels': max(r['changed_pixels'] for r in repeated if r['mode'] == m)} for m in MODES}
    returned = {m: sum(r['pass'] for r in returns if r['mode'] == m) for m in MODES}
    cohorts = []
    for w in [1728, 3456]:
        for c in range(5):
            sub = [r for r in timings if r['width'] == w and r['case'] == c]
            assert len(sub) == 64 and {(r['round'], r['slot']) for r in sub} == {(i, s) for i in range(8) for s in range(8)}
            row = {'width': w, 'case': c}
            for m in MODES:
                group = [r for r in sub if r['mode'] == m]
                assert len(group) == 16 and all(r['timing']['frames'] == 128 for r in group)
                assert all(math.isfinite(r['timing'][k]) and r['timing'][k] >= 0 for r in group for k in ['gpu_ms', 'cpu_encode_ms', 'total_batch_ms_per_frame'])
                row[m] = {k: median(r['timing'][k] for r in group) for k in ['gpu_ms', 'cpu_encode_ms', 'total_batch_ms_per_frame']}
                duplicates = []
                for i in range(8):
                    pair = [r['timing']['total_batch_ms_per_frame'] for r in group if r['round'] == i]
                    assert len(pair) == 2
                    duplicates.append(abs(pair[0]-pair[1]) / median(pair)*100)
                row[m]['duplicate_percent'] = median(duplicates)
                if m != 'implicit':
                    row[m]['relative_cost_percent'] = (row[m]['total_batch_ms_per_frame']/row['implicit']['total_batch_ms_per_frame']-1)*100
                    row[m]['rounds_faster'] = sum(
                        median(r['timing']['total_batch_ms_per_frame'] for r in sub if r['mode'] == m and r['round'] == i)
                        < median(r['timing']['total_batch_ms_per_frame'] for r in sub if r['mode'] == 'implicit' and r['round'] == i) for i in range(8))
            cohorts.append(row)
    passed = all(stats[m]['passes'] == 138 for m in MODES[2:]) and all(r['pass'] for r in returns) and all(r['changed_pixels'] == 0 for r in repeated)
    summary = {'image_and_return_gates_pass': passed, 'quality': stats, 'returns': returned,
               'repetitions': repetitions, 'cohorts': cohorts, 'input_sha256': hashlib.sha256(raw).hexdigest(),
               'geometry_mib': data['geometry_bytes']/1048576, 'texture_mib': data['texture_bytes']/1048576,
               'extra_slopes_mib': data['slopes_bytes']/1048576, 'compiled_file_bytes': data['compiled_file_bytes'],
               'source_triangles': data['source_triangles']}
    (OUT/'tangent-summary.json').write_text(json.dumps(summary, indent=2)+'\n')
    embedded = json.dumps({'quality': quality, 'summary': summary}).replace('<', '\\u003c')
    status = 'Gates numéricos passaram nesta coleta; integração não realizada.' if passed else 'REPROVADO: falhas de imagem ou repetibilidade; material original preservado.'
    page = '''<!doctype html><html lang="pt-BR"><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>Tangente geométrica — Ignis</title>
<style>body{background:#171a1e;color:#e5e8eb;font:16px system-ui;margin:24px}select,button,input{font:inherit;margin:6px;padding:6px}table{border-collapse:collapse}td,th{padding:8px;border-bottom:1px solid #46515b;text-align:right}.images{display:flex;gap:12px;overflow:auto}.images figure{margin:0;min-width:0;flex:1}.images img{display:block;max-width:100%;height:auto}body.native .images figure{flex:none}body.native .images img{max-width:none}#status{padding:14px;background:#473829}p{max-width:1100px}</style>
<h1>Tangente real do triângulo</h1><p id="status">STATUS</p>
<p>Fonte completa, mesmos mapas e iluminação. IGT1 compila a inclinação das faces em build time. Comparação isolada da filtragem; não é o seletor regional ou uma nova redução de triângulos.</p>
<label>Resolução<select id="resolution"><option>1728</option><option>3456</option></select></label><label>Câmera<select id="case"><option value="0">Perto · 2,7</option><option value="1">6</option><option value="2">16</option><option value="3">Longe · 64</option><option value="4">Aproximar / afastar</option></select></label>
<label>Pose<input id="frame" type="range" min="0" max="8" value="0"></label><label>Filtro<select id="mode"><option value="geometric-grad">Tangente geométrica · gradientes</option><option value="geometric-lod">Tangente geométrica · LOD</option><option value="analytic-lod">Normal do material · LOD</option></select></label><button id="scale">Ver pixels nativos</button>
<div class="images"><figure><figcaption>Fonte · filtro original</figcaption><img id="reference"></figure><figure><figcaption id="caption"></figcaption><img id="candidate"></figure></div><pre id="quality"></pre>
<p>Resolução física indicada, exibição ajustável. Igualdade de retorno não prova ausência de cintilação em movimento. Gates agregados não significam pixels idênticos ou aceite humano.</p><h2>Tempo total · ms/frame</h2><p>Oito rodadas alternadas e duplicadas; não somar CPU e GPU sobrepostas. Memória e controles da coleta são específicos deste ensaio.</p><table><thead><tr><th>Pixels / câmera</th><th>Original</th><th>Material LOD</th><th>Geom. LOD</th><th>Geom. gradientes</th></tr></thead><tbody id="cost"></tbody></table><p id="memory"></p>
<script>const data=DATA;const $=id=>document.getElementById(id);const names={'analytic-lod':'Normal do material · LOD','geometric-lod':'Tangente geométrica · LOD','geometric-grad':'Tangente geométrica · gradientes'};
function update(){const w=+$('resolution').value,c=+$('case').value;$('frame').max=c===4?32:8;const f=Math.min(+$('frame').value,+$('frame').max);$('frame').value=f;const m=$('mode').value;const r=data.quality.find(r=>r.width===w&&r.case===c&&r.frame===f);const q=r[m];$('reference').src=`tangent-${w}-${c}-${f}-implicit.png`;$('candidate').src=`tangent-${w}-${c}-${f}-${m}.png`;$('caption').textContent=names[m];for(const id of ['reference','candidate'])$(id).style.width=document.body.classList.contains('native')?`${w/(window.devicePixelRatio||1)}px`:'100%';$('quality').textContent=`${q.pass?'Passa':'FALHA'} · RMSE ${q.rmse.toFixed(6)} · p99 ${q.p99.toFixed(6)} · cobertura ${q.coverage_error.toExponential(3)}\nRetornos: ${data.summary.returns[m]}/10; referência: ${data.summary.returns.implicit}/10. Repetições estáticas: ${data.summary.repetitions[m].unchanged}/128; referência: ${data.summary.repetitions.implicit.unchanged}/128.\nDistância ${r.distance.toFixed(3)}.`;}
for(const id of ['resolution','case','frame','mode'])$(id).addEventListener('input',update);$('scale').onclick=()=>{document.body.classList.toggle('native');$('scale').textContent=document.body.classList.contains('native')?'Ajustar à página':'Ver pixels nativos';update();};window.addEventListener('resize',update);$('cost').innerHTML=data.summary.cohorts.map(r=>`<tr><td>${r.width} / ${r.case}</td>${['implicit','analytic-lod','geometric-lod','geometric-grad'].map(m=>`<td>${r[m].total_batch_ms_per_frame.toFixed(4)}</td>`).join('')}</tr>`).join('');$('memory').textContent=`Fonte: ${data.summary.geometry_mib.toFixed(3)} MiB; mapas compartilhados: ${data.summary.texture_mib.toFixed(3)} MiB; inclinações extras: ${data.summary.extra_slopes_mib.toFixed(3)} MiB. Targets, driver, pipelines e cópias CPU excluídos.`;update();</script></html>'''
    (OUT/'tangent.html').write_text(page.replace('STATUS', status).replace('DATA', embedded))
    print(json.dumps({'passed': passed, 'quality': {m: v['passes'] for m, v in stats.items()}, 'returns': returned, 'repetitions': repetitions}, indent=2))


if __name__ == '__main__':
    main()
