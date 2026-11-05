#!/usr/bin/env python3
"""Material-rich real relief, separate controls and explicit failures."""
import json
import math
import sys
from pathlib import Path
from statistics import median
OUT=Path(__file__).resolve().parents[1]/'artifacts/appearance-probe/rich-relief'
AUTO='--auto' in sys.argv
REFINED='--refined' in sys.argv or AUTO
PREFIX='auto-' if AUTO else ('refined-' if REFINED else '')
def main():
 d=json.loads((OUT/(PREFIX+'results.json')).read_text()); build=json.loads((OUT/'build.json').read_text()); cohorts=[]
 if REFINED:
  r=json.loads((OUT/'refined-build.json').read_text());build.update(level_triangles=r['level_triangles'],regional_gpu_bytes=r['gpu_geometry_bytes'],regional_file_bytes=r['file_bytes'])
 if AUTO:
  assert d['automatic'] and len(d['calibrations'])==2
  build['regional_gpu_bytes']=d['auto_geometry_bytes']
 assert len(d['quality'])==138 and len(d['timings'])==800
 assert len(d['returns'])==50
 if not REFINED: assert all(x['pass'] for x in d['returns'] if x['mode']=='source')
 for row in d['quality']:
  assert row['source_triangles']==build['source_triangles']
  for name,budget in [('selection',0.05),('exploratory_selection',0.15)]:
   selection=row[name]
   assert selection['max_error_px']<=budget and selection['triangles']<=row['source_triangles']
  for mode in ['previous','conservative','regional','exploratory']:
   q=row[mode]
   assert all(math.isfinite(q[k]) for k in ['rmse','p99','max','coverage_error'])
   assert q['pass']==(q['rmse']<=0.005 and q['p99']<=0.02 and q['coverage_error']<=0.0005)
  for mode in ['source','previous','conservative','regional','exploratory']:
   assert (OUT/f"{PREFIX}{row['width']}-{row['case']}-{row['frame']}-{mode}.png").exists()
 for row in d['timings']:
  assert row['timing']['frames']==128
  assert all(math.isfinite(row['timing'][k]) and row['timing'][k]>0 for k in ['gpu_ms','cpu_encode_ms','total_batch_ms_per_frame'])
 for width in [1728,3456]:
  for case in range(5):
   rows=[x for x in d['quality'] if x['width']==width and x['case']==case]
   c={'width':width,'case':case,'min_triangles':min(x['selection']['triangles'] for x in rows),'max_triangles':max(x['selection']['triangles'] for x in rows),'max_ranges':max(len(x['selection']['ranges']) for x in rows),'max_error_px':max(x['selection']['max_error_px'] for x in rows)}
   samples=[x for x in d['timings'] if x['width']==width and x['case']==case]
   for mode in ['source','previous','conservative','regional','exploratory']:
    a=[x for x in samples if x['mode']==mode];assert len(a)==16
    c[mode]={k:median(x['timing'][k] for x in a) for k in ['gpu_ms','cpu_encode_ms','total_batch_ms_per_frame']}
    pairs=[]
    for r in range(8):
     p=[x['timing']['total_batch_ms_per_frame'] for x in a if x['round']==r];pairs.append(abs(p[0]-p[1])/min(p)*100)
    c[mode]['duplicate_percent']=median(pairs)
   c['favorable_rounds']=sum(median(x['timing']['total_batch_ms_per_frame'] for x in samples if x['round']==r and x['mode']=='regional')<median(x['timing']['total_batch_ms_per_frame'] for x in samples if x['round']==r and x['mode']=='previous') for r in range(8))
   if AUTO:
    c['auto_local_captures']=sum(x['auto_local'] for x in rows)
    c['capture_count']=len(rows)
    c['auto_favorable_rounds']=sum(median(x['timing']['total_batch_ms_per_frame'] for x in samples if x['round']==r and x['mode']=='conservative')<median(x['timing']['total_batch_ms_per_frame'] for x in samples if x['round']==r and x['mode']=='previous') for r in range(8))
   cohorts.append(c)
 failures={m:[dict(width=x['width'],case=x['case'],frame=x['frame'],metrics=x[m]) for x in d['quality'] if not x[m]['pass']] for m in ['previous','conservative','regional','exploratory']}
 summary=dict(calibrations=d.get('calibrations',[]),build=build,cohorts=cohorts,failures=failures,adapters=d['adapters'],returns=d['returns'],prior_instability=json.loads((OUT/'previous-return-diagnostic.json').read_text()) if (OUT/'previous-return-diagnostic.json').exists() else None);(OUT/(PREFIX+'summary.json')).write_text(json.dumps(summary,indent=2)+'\n')
 html='''<!doctype html><html lang="pt-BR"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Relevo real e cores</title><style>body{font:16px system-ui;background:#10151c;color:#eaf2f5;margin:24px}h1{font-size:30px}p{max-width:1120px;line-height:1.5}.note{color:#a8c5c9}a{color:#7fdbd4}label{display:inline-block;margin:8px}select{font:inherit;background:#26373e;color:white;padding:6px}section{display:flex;gap:16px}figure{flex:1;min-width:0;margin:0}figcaption{margin:10px 0}.view{height:500px;overflow:auto;background:black}img{display:block;max-width:none}table{border-collapse:collapse;font-size:14px}td,th{border:1px solid #48616b;padding:8px;text-align:right}#metric{white-space:pre-wrap;font:14px monospace}@media(max-width:800px){section{display:block}.view{height:350px}}</style><body>
<h1>Relevo real, materiais e detalhes em várias escalas</h1><p>Terreno procedural de 1.179.648 triângulos com sulcos, pequenas irregularidades, veios minerais, oxidação e musgo. As cores são uma composição de teste, não um scan fotográfico. Todos os caminhos recebem os mesmos mapas de cor, normal, rugosidade e modulação ambiente, compilados antes da execução. A luz continua dinâmica.</p>
<p>Os níveis regionais preservam todos os vértices das bordas. A seleção regional usa erro geométrico ≤ 0,05 pixel físico; a alternativa exploratória usa 0,15 pixel. Ambas dispensam o veto conservador de silhueta, mantido como controle. As falhas de imagem aparecem abaixo. Não confundir menos triângulos com qualidade aprovada ou performance maior.</p>
<label>Pixels <select id="resolution"><option value="1728">1728 ×1117</option><option value="3456" selected>3456 ×2234</option></select></label><label>Trajetória <select id="case"><option value="0">Órbita perto —2,7</option><option value="1">Órbita média —6</option><option value="2">Órbita afastada —16</option><option value="3">Órbita distante —64</option><option value="4">Afastar e aproximar</option></select></label><label>Posição / luz <input id="frame" type="range" min="0" max="8" value="1"></label><label>Candidato <select id="mode"><option value="regional">Regional — 0,05 pixel</option><option value="exploratory">Exploratório — 0,15 pixel</option><option value="previous">Compilador anterior + hierarquia</option><option value="conservative">Regional com veto de silhueta</option></select></label><label><input id="fit" type="checkbox" checked>Visão geral</label>
<p id="outcome"></p><p id="scale" class="note"></p><section><figure><figcaption>Referência: geometria completa e mesmos materiais</figcaption><div class="view"><img id="source" alt="Superfície completa"></div></figure><figure><figcaption id="candidateTitle"></figcaption><div class="view"><img id="candidate" alt="Caminho selecionado"></div></figure></section><p id="metric"></p>
<h2>Performance na trajetória selecionada</h2><p>Medianas em ms/frame de lote;16 amostras/caminho,128 frames/amostra,32 poses, oito rodadas alternadas com duplicatas. Total inclui seleçãoCPU, codificação, submissão e espera. Capturas são offscreen; este painel não é um renderer em tempo real nem medição de FPS de gameplay.</p><table><thead><tr><th>Caminho</th><th>GPU</th><th>CPU codificação</th><th>Total</th><th>Duplicatas</th></tr></thead><tbody id="timings"></tbody></table>
<h2>Qualidade e dados</h2><p>Gates mantidos: RMSE ≤ 0,005, p99 ≤ 0,02, cobertura ≤ 0,0005. Passar não garante pixels idênticos. Mipmaps e material são compartilhados; o erro geométrico não é um certificado de erro fotométrico.</p><div id="qualityTable"></div><div id="builds"></div><p class="note">Heightfield controlado: não comprova compilação multiescala de qualquer malha importada, sombras dinâmicas, PBR completo, streaming ou ganho em um jogo AAA.</p><p><a href="results.json">Resultados completos</a> · <a href="summary.json">Resumo</a> · <a href="build.json">Build e memória</a></p>
<script>
const data=__DATA__, $=id=>document.getElementById(id), names={source:'Fonte completa',previous:'Anterior + hierarquia',conservative:'Regional com veto de silhueta',regional:'Regional — 0,05 pixel',exploratory:'Exploratório — 0,15 pixel'};
function update(){const w=+$('resolution').value,c=+$('case').value,mode=$('mode').value,fit=$('fit').checked;let frame=+$('frame').value;const maximum=c===4?32:8;$('frame').max=maximum;frame=Math.min(frame,maximum);$('frame').value=frame;const row=data.rows.find(r=>r.width===w&&r.case===c&&r.frame===frame),q=row[mode],cohort=data.summary.cohorts.find(r=>r.width===w&&r.case===c);
 $('source').src=`${w}-${c}-${frame}-source.png`;$('candidate').src=`${w}-${c}-${frame}-${mode}.png`;
 for(const id of ['source','candidate']){const im=$(id);im.style.width=fit?'100%':`${w/devicePixelRatio}px`;im.onload=()=>{const v=im.parentElement;v.scrollLeft=fit?0:Math.max(0,(im.clientWidth-v.clientWidth)/2);v.scrollTop=fit?0:Math.max(0,(im.clientHeight-v.clientHeight)/2);};}
 $('candidateTitle').textContent=names[mode];$('scale').textContent=fit?'Visão geral reduzida; desmarque para inspecionar pixels físicos.':`1 pixel renderizado por pixel físico, DPR ${devicePixelRatio}; role a imagem para inspecionar.`;
 const repeat=data.summary.returns.find(r=>r.width===w&&r.case===c&&r.mode===mode);$('outcome').textContent=!data.summary.returns.find(r=>r.width===w&&r.case===c&&r.mode==='source').pass?'FALHA NA REFERÊNCIA: comparação desta trajetória não aprovada.':!repeat.pass?'FALHA: este caminho não repetiu os mesmos pixels ao retornar à câmera.':q.pass?'Passa nos limites numéricos desta captura.':'FALHA nos limites de qualidade nesta captura — não aprovado.';
 const selection=mode==='exploratory'?row.exploratory_selection:row.selection;const t=mode==='previous'?row.old_triangles:mode==='conservative'?row.conservative_triangles:selection.triangles;
 $('metric').textContent=`Distância ${row.distance.toFixed(3)}; entrada ${row.source_triangles}T; selecionado ${t}T.\n${mode==='regional'||mode==='exploratory'?`Regional: ${selection.reduced}/144 regiões reduzidas; ${selection.ranges.length} draws; erro certificado máximo ${selection.max_error_px.toFixed(6)} px.`:''}\nRMSE ${q.rmse.toExponential(5)}, p99 ${q.p99.toExponential(5)}, cobertura ${q.coverage_error.toExponential(5)}. Máximo canal ${q.max.toExponential(5)}.`;
 $('timings').innerHTML=Object.entries(names).map(([m,n])=>`<tr><td>${n}</td><td>${cohort[m].gpu_ms.toFixed(5)}</td><td>${cohort[m].cpu_encode_ms.toFixed(5)}</td><td>${cohort[m].total_batch_ms_per_frame.toFixed(5)}</td><td>${cohort[m].duplicate_percent.toFixed(1)}%</td></tr>`).join('');
 $('qualityTable').innerHTML='<p>Controle anterior: houve instabilidade de retorno numa coleta diagnóstica anterior (4 pixels PNG, diferença máxima 1/255); veja o estudo. Retornos reprovados nesta coleta: '+data.summary.returns.filter(r=>!r.pass).map(r=>`${r.mode}/${r.width}/trajetória${r.case}`).join('; ').concat(data.summary.returns.every(r=>r.pass)?'nenhum':'')+'</p>'+Object.entries(data.summary.failures).map(([m,f])=>`<p>${names[m]}: ${f.length} falhas de 138. ${f.map(r=>`${r.width}/trajetória${r.case}/pose${r.frame}`).join('; ')}</p>`).join('');}
for(const id of ['resolution','case','frame','mode','fit'])$(id).addEventListener('input',update);window.addEventListener('resize',update);
const b=data.summary.build;$('builds').textContent=`144 regiões. Níveis completos: ${b.level_triangles.join(' / ')}T. Geometria fonte ${(b.source_gpu_bytes/1048576).toFixed(2)} MiB; fonte + níveis ${(b.regional_gpu_bytes/1048576).toFixed(2)} MiB; mapas compartilhados ${(b.texture_gpu_bytes/1048576).toFixed(2)} MiB. Sem contabilizar targets, driver, cópias CPU ou buffers dos controles. Não houve economia de memória total.`;update();
</script></body></html>'''
 if REFINED:
  if not all(x['pass'] for x in d['returns'] if x['mode']=='source'):
   html=html.replace('<h1>', '<p style="border:2px solid #e9a447;padding:14px"><strong>Coleta reprovada no retorno da referência.</strong> Os tempos mostram ganhos parciais, mas a trajetória com referência instável não está aprovada. Veja as falhas abaixo.</p><h1>',1)
  html=html.replace('Relevo real, materiais e detalhes em várias escalas','Relevo real com refinamento local')
  html=html.replace('Regional com veto de silhueta','Níveis uniformes anteriores — 0,05 pixel').replace('Regional — 0,05 pixel','Refinamento local — 0,05 pixel').replace('Exploratório — 0,15 pixel','Refinamento local — 0,15 pixel')
  html=html.replace('Ambas dispensam o veto conservador de silhueta, mantido como controle.','A divisão interna adapta-se ao relevo em build time, mantendo as 144 regiões de seleção e todos os limites nativos. O controle usa os níveis uniformes do experimento anterior, também com 0,05 pixel.')
  html=html.replace('Controle anterior: houve instabilidade de retorno numa coleta diagnóstica anterior (4 pixels PNG, diferença máxima 1/255); veja o estudo.','Instabilidade em capturas repetidas: também ocorreu com derivadas finas explícitas. O material original foi mantido; veja o diagnóstico e o estudo.')
  html=html.replace('`${w}-${c}-${frame}', '`refined-${w}-${c}-${frame}')
  html=html.replace('href="results.json"','href="refined-results.json"').replace('href="summary.json"','href="refined-summary.json"').replace('href="build.json"','href="refined-build.json"')
  html=html.replace('<h2>Qualidade e dados</h2>', '<h2>Qualidade e dados</h2><p>Este é um experimento diagnóstico: instabilidade de retorno continua em investigação. <a href="repeat.json">Repetibilidade e filtragem</a> · <a href="index.html">Experimento anterior</a></p>')
 if AUTO:
  if any(not r['pass'] for r in d['returns'] if r['mode']=='conservative') or failures['conservative']:
   html=html.replace('<h1>', '<p style="border:2px solid #f27b6d;padding:14px"><strong>Automático reprovado.</strong> Há falha de qualidade ou de repetibilidade no caminho automático. Ganhos de tempo em outras trajetórias não eliminam essa falha.</p><h1>',1)
  html=html.replace('${r.mode}/${r.width}', '${names[r.mode]}/${r.width}')
  html=html.replace('<h2>Performance na trajetória selecionada</h2>', '<h2>Performance na trajetória selecionada</h2><p>Quando o automático mantém o anterior, ambos desenham a mesma representação: diferenças de tempo não comprovam ganho do algoritmo. A variação das duplicatas está na última coluna. Na resolução maior, a escolha conservadora ainda perde ganhos em distâncias intermediárias.</p>')
  html=html.replace('<option value="conservative">','<option value="conservative" selected>')
  html=html.replace("$('candidateTitle').textContent=names[mode];", "$('candidateTitle').textContent=names[mode]+(mode==='conservative'?(row.auto_local?' — usando local':' — usando anterior'):'');")
  html=html.replace('fonte + níveis ${(b.regional_gpu_bytes','fonte + níveis + anterior residente ${(b.regional_gpu_bytes')
  html=html.replace('refined-', 'auto-').replace('refined.html','auto.html')
  html=html.replace('Relevo real com refinamento local','Relevo real com escolha automática')
  html=html.replace('Níveis uniformes anteriores — 0,05 pixel','Automático — anterior ou local 0,05 pixel')
  html=html.replace('O controle usa os níveis uniformes do experimento anterior, também com 0,05 pixel.','O automático escolhe entre anterior e local0,05px por custo calibrado e margem conservadora. O ajuste usa seis vistas separadas, antes das vistas de validação; nenhuma imagem é consultada na seleção.')
  html=html.replace('<h2>Qualidade e dados</h2>', '<h2>Calibração e memória adicional</h2><p>'+ '; '.join(str(c['width'])+'px: '+format(c['elapsed_ms']/1000,'.2f')+' s de calibração inicial; margem '+format(c['model']['margin_ms'],'.6f')+' ms' for c in d['calibrations'])+'. Este custo inicial não está amortizado nos tempos por frame. A memória abaixo inclui também a geometria anterior residente para o fallback automático. O material original continua em uso; LOD explícito não corrigiu a repetibilidade.</p><h2>Qualidade e dados</h2>')
  html=html.replace('href="auto-build.json"','href="refined-build.json"').replace('href="repeat.json"','href="filter-repeat.json"')
  for row in d['quality']:
   assert isinstance(row['auto_local'],bool)
   assert row['conservative_triangles']==(row['selection']['triangles'] if row['auto_local'] else row['old_triangles'])
 (OUT/('auto.html' if AUTO else ('refined.html' if REFINED else 'index.html'))).write_text(html.replace('__DATA__',json.dumps({'summary':summary,'rows':d['quality']})))
 print(json.dumps(summary,indent=2))
if __name__=='__main__':main()
