#!/usr/bin/env python3
"""Source-only geometric tangent experiment; generate a report even on rejection."""
import hashlib
import json
import math
from pathlib import Path
from statistics import median

OUT = Path(__file__).resolve().parents[1] / 'artifacts/appearance-probe/rich-relief'
MODES = ['implicit', 'geometric-id', 'geometric-fetch', 'geometric-grad', 'geometric-screen', 'geometric-packed']


def main():
    raw = (OUT / 'compact-results.json').read_bytes()
    data = json.loads(raw)
    quality, returns, repeated, timings = (data[k] for k in ('quality', 'returns', 'repeated', 'timings'))
    assert len(quality) == 138 and len(returns) == 60 and len(repeated) == 768 and len(timings) == 960
    assert {(r['width'], r['case'], r['frame']) for r in quality} == {
        (w, c, f) for w in [1728, 3456] for c in range(5) for f in range(33 if c == 4 else 9)}
    assert {(r['width'], r['case'], r['mode']) for r in returns} == {
        (w, c, m) for w in [1728, 3456] for c in range(5) for m in MODES}
    assert all(r['pass'] == (r['first_hash'] == r['return_hash']) for r in returns)
    assert {(r['width'], r['case'], r['turn'], r['mode']) for r in repeated} == {
        (w, c, t, m) for w in [1728, 3456] for c in [0, 3] for t in range(32) for m in MODES}
    for row in quality:
        for m in MODES[1:] + ['projection_only', 'quantization_only']:
            q = row[m]
            assert all(math.isfinite(q[k]) for k in ['rmse', 'p99', 'max', 'coverage_error'])
            assert q['pass'] == (q['rmse'] <= .005 and q['p99'] <= .02 and q['coverage_error'] <= .0005)
    stats = {m: {'passes': sum(r[m]['pass'] for r in quality),
                 'max_rmse': max(r[m]['rmse'] for r in quality),
                 'max_p99': max(r[m]['p99'] for r in quality),
                 'max_channel': max(r[m]['max'] for r in quality),
                 'max_coverage_error': max(r[m]['coverage_error'] for r in quality),
                 'failures': [{k: r[k] for k in ['width', 'case', 'frame']} for r in quality if not r[m]['pass']]}
             for m in MODES[1:] + ['projection_only', 'quantization_only']}
    repetitions = {m: {'unchanged': sum(r['changed_pixels'] == 0 for r in repeated if r['mode'] == m),
                       'max_changed_pixels': max(r['changed_pixels'] for r in repeated if r['mode'] == m)} for m in MODES}
    returned = {m: sum(r['pass'] for r in returns if r['mode'] == m) for m in MODES}
    cohorts = []
    for w in [1728, 3456]:
        for c in range(5):
            sub = [r for r in timings if r['width'] == w and r['case'] == c]
            assert len(sub) == 96 and {(r['round'], r['slot']) for r in sub} == {(i, s) for i in range(8) for s in range(12)}
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
    passed = all(stats[m]['passes'] == 138 for m in MODES[1:]) and all(r['pass'] for r in returns) and all(r['changed_pixels'] == 0 for r in repeated)
    candidates_pass = all(stats[m]['passes']==138 and returned[m]==10 and repetitions[m]['unchanged']==128 for m in MODES[3:])
    summary = {'candidate_gates_pass': candidates_pass,'image_and_return_gates_pass': passed, 'quality': stats, 'returns': returned,
               'repetitions': repetitions, 'cohorts': cohorts, 'input_sha256': hashlib.sha256(raw).hexdigest(),
               'geometry_mib': data['geometry_bytes']/1048576, 'texture_mib': data['texture_bytes']/1048576,
               'extra_slopes_mib': data['slopes_bytes']/1048576, 'compiled_file_bytes': data['compiled_file_bytes'],
               'source_triangles': data['source_triangles'],
               'packed_mib': data['packed_bytes']/1048576,
               'source_with_maps_mib': (data['geometry_bytes']+data['texture_bytes'])/1048576,
               'isolated_raw_mib': (data['geometry_bytes']+data['texture_bytes']+data['slopes_bytes'])/1048576,
               'isolated_packed_mib': (data['geometry_bytes']+data['texture_bytes']+data['packed_bytes'])/1048576,
               'combined_resident_mib': (data['geometry_bytes']+data['texture_bytes']+data['slopes_bytes']+data['packed_bytes'])/1048576,
               'scale': data['scale'], 'quantization_max_error': data['quantization_max_error']}
    for r in cohorts:
        for m in ['geometric-screen','geometric-packed']:
            r[m]['vs_previous_percent']=(r[m]['total_batch_ms_per_frame']/r['geometric-grad']['total_batch_ms_per_frame']-1)*100
            r[m]['rounds_faster_previous']=sum(
                median(x['timing']['total_batch_ms_per_frame'] for x in timings if x['width']==r['width'] and x['case']==r['case'] and x['mode']==m and x['round']==i)
                < median(x['timing']['total_batch_ms_per_frame'] for x in timings if x['width']==r['width'] and x['case']==r['case'] and x['mode']=='geometric-grad' and x['round']==i) for i in range(8))
    summary['repeat_cohorts']=[]
    for w in [1728,3456]:
        for c in [0,3]:
            for m in MODES:
                rs=[r for r in repeated if r['width']==w and r['case']==c and r['mode']==m]
                assert len({r['first_hash'] for r in rs})==1
                assert all((r['changed_pixels']==0)==(r['first_hash']==r['hash']) for r in rs)
                summary['repeat_cohorts'].append({'width':w,'case':c,'mode':m,'unchanged':sum(r['changed_pixels']==0 for r in rs),'unique_later_hashes':len({r['hash'] for r in rs}),'max_delta':max(r['max_delta'] for r in rs)})
    (OUT/'compact-summary.json').write_text(json.dumps(summary, indent=2)+'\n')
    embedded = json.dumps({'quality': quality, 'summary': summary}).replace('<', '\\u003c')
    status = ('Gates numéricos passaram nesta coleta; integração não realizada.' if passed else
              f"REPROVADO no gate global. Original: {repetitions['implicit']['unchanged']}/128 repetições exatas. Gates dos candidatos geométricos: {'passaram' if candidates_pass else 'falharam'}. Não adotado no renderer.")
    page = '''<!doctype html><html lang="pt-BR"><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>Tangente compacta — Ignis</title>
<style>body{background:#171a1e;color:#e5e8eb;font:16px system-ui;margin:24px}select,button,input{font:inherit;margin:6px;padding:6px}table{border-collapse:collapse}td,th{padding:8px;border-bottom:1px solid #46515b;text-align:right}.images{display:flex;gap:12px;overflow:auto}.images figure{margin:0;min-width:0;flex:1}.images img{display:block;max-width:100%;height:auto}body.native .images figure{flex:none}body.native .images img{max-width:none}#status{padding:14px;background:#473829}p{max-width:1100px}</style>
<h1>Tangente compacta e custo de projeção</h1><p id="status">STATUS</p>
<p>Fonte completa, mesmos mapas e iluminação. IGP1 compila duas inclinações em 4 bytes por face, mais uma escala compartilhada de 4 bytes. A coleta mantém ambas as tabelas residentes; os totais isolados são discriminados abaixo. Comparação isolada da filtragem; não é o seletor regional ou uma nova redução de triângulos.</p>
<label>Resolução<select id="resolution"><option>1728</option><option>3456</option></select></label><label>Câmera<select id="case"><option value="0">Perto · 2,7</option><option value="1">6</option><option value="2">16</option><option value="3">Longe · 64</option><option value="4">Aproximar / afastar</option></select></label>
<label>Pose<input id="frame" type="range" min="0" max="8" value="0"></label><label>Filtro<select id="mode"><option value="geometric-grad">Tangente geométrica · gradientes</option><option value="geometric-screen">Projeção pelo fragmento · f32</option><option value="geometric-packed">Projeção pelo fragmento · compacto</option></select></label><button id="scale">Ver pixels nativos</button>
<div class="images"><figure><figcaption>Fonte · filtro original</figcaption><img id="reference"></figure><figure><figcaption id="caption"></figcaption><img id="candidate"></figure></div><pre id="quality"></pre>
<p>Resolução física indicada, exibição ajustável. Igualdade de retorno não prova ausência de cintilação em movimento. Gates agregados não significam pixels idênticos ou aceite humano.</p><h2>Tempo total · ms/frame</h2><p>Oito rodadas alternadas e duplicadas; não somar CPU e GPU sobrepostas. Memória e controles da coleta são específicos deste ensaio. Os controles com guardas não são temporizadores de instruções isoladas. Detalhes GPU, CPU, dispersão e comparações pareadas estão no <a href="compact-summary.json">resumo numérico</a>.</p><table><thead><tr><th>Pixels / câmera</th><th>Original</th><th>Controle índice</th><th>Controle leitura</th><th>Anterior f32</th><th>Projeção f32</th><th>Compacto</th></tr></thead><tbody id="cost"></tbody></table><p id="memory"></p>
<script>const data=DATA;const $=id=>document.getElementById(id);const names={'geometric-grad':'Anterior · f32','geometric-screen':'Projeção pelo fragmento · f32','geometric-packed':'Projeção pelo fragmento · compacto'};
function update(){const w=+$('resolution').value,c=+$('case').value;$('frame').max=c===4?32:8;const f=Math.min(+$('frame').value,+$('frame').max);$('frame').value=f;const m=$('mode').value;const r=data.quality.find(r=>r.width===w&&r.case===c&&r.frame===f);const q=r[m];$('reference').src=`compact-${w}-${c}-${f}-implicit.png`;$('candidate').src=`compact-${w}-${c}-${f}-${m}.png`;$('caption').textContent=names[m];for(const id of ['reference','candidate'])$(id).style.width=document.body.classList.contains('native')?`${w/(window.devicePixelRatio||1)}px`:'100%';$('quality').textContent=`${q.pass?'Passa':'FALHA'} · RMSE ${q.rmse.toFixed(6)} · p99 ${q.p99.toFixed(6)} · cobertura ${q.coverage_error.toExponential(3)}\nRetornos: ${data.summary.returns[m]}/10; referência: ${data.summary.returns.implicit}/10. Repetições estáticas: ${data.summary.repetitions[m].unchanged}/128; referência: ${data.summary.repetitions.implicit.unchanged}/128.\nDistância ${r.distance.toFixed(3)}.`;}
for(const id of ['resolution','case','frame','mode'])$(id).addEventListener('input',update);$('scale').onclick=()=>{document.body.classList.toggle('native');$('scale').textContent=document.body.classList.contains('native')?'Ajustar à página':'Ver pixels nativos';update();};window.addEventListener('resize',update);$('cost').innerHTML=data.summary.cohorts.map(r=>`<tr><td>${r.width} / ${r.case}</td>${['implicit','geometric-id','geometric-fetch','geometric-grad','geometric-screen','geometric-packed'].map(m=>`<td>${r[m].total_batch_ms_per_frame.toFixed(4)}</td>`).join('')}</tr>`).join('');$('memory').textContent=`Fonte: ${data.summary.geometry_mib.toFixed(3)} MiB; mapas compartilhados: ${data.summary.texture_mib.toFixed(3)} MiB; Tabela f32: ${data.summary.extra_slopes_mib.toFixed(3)} MiB; compacta: ${data.summary.packed_mib.toFixed(3)} MiB. Total isolado f32: ${data.summary.isolated_raw_mib.toFixed(3)} MiB; compacto: ${data.summary.isolated_packed_mib.toFixed(3)} MiB; ambas residentes nesta coleta: ${data.summary.combined_resident_mib.toFixed(3)} MiB. Targets, driver, pipelines e cópias CPU excluídos.`;update();</script></html>'''
    (OUT/'compact.html').write_text(page.replace('STATUS', status).replace('DATA', embedded))
    print(json.dumps({'passed': passed, 'quality': {m: v['passes'] for m, v in stats.items()}, 'returns': returned, 'repetitions': repetitions}, indent=2))


if __name__ == '__main__':
    main()
