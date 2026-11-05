"""Report automatic decisions, physical captures, paired timings and residual losses."""
import json,statistics as st
from pathlib import Path
ROOT=Path('artifacts/appearance-probe/automatic-shading')
def pair(c,d,m,k='throughput_ms'):
 n=len(c['codes']);i=d*n+c['codes'].index(m);h=len(c['timings'])//2
 return st.mean(c['timings'][j][k]['median'] for j in [i,i+h])
cs=json.loads((ROOT/'validation.json').read_text());qs=json.loads((ROOT/'quality.json').read_text())
repeat=json.loads((ROOT/'repeat.json').read_text());fit=json.loads((ROOT/'fit.json').read_text())
keys=['throughput_ms','gpu_ms','cpu_ms'];codes=[0,5,85,95];rows=[];summary=[]
for r in qs:
 c=next(c for c in cs if all(c[k]==r[k] for k in ['family','resolution','distance']))
 n=4;h=len(c['timings'])//2;d=r['density']
 times=[{k:{'median':c['timings'][d*n+m+extra][k]['median']} for k in keys} for extra in [0,h] for m in range(4)]
 rows.append(r|{'timings':times,'source_triangles_per_covered_pixel':r['triangles'][0]/r['covered_pixels']})
for c in cs:
 modes=[]
 for mi,code in enumerate(codes):
  curves={k:[pair(c,d,code,k) for d in range(4)] for k in keys}
  noise={k:[c['timings'][d*4+mi+16][k]['median']/c['timings'][d*4+mi][k]['median'] for d in range(4)] for k in keys}
  modes.append({'curve_ms':curves,'growth_100_over_1':{k:v[-1]/v[0] for k,v in curves.items()},'curve_max_over_min':{k:max(v)/min(v) for k,v in curves.items()},'quality_all_densities':mi==0 or all(r['checks'][mi-1]['quality_gate'] for r in rows if all(r[k]==c[k] for k in ['family','resolution','distance'])),'duplicate_ratio_range':{k:[min(v),max(v)] for k,v in noise.items()}})
 summary.append({k:c[k] for k in ['family','resolution']}|{'frame':[14,19,21,28,32,56,84,112,128,224].index(c['distance']),'modes':modes})
results=[]
for c in cs:
 for d in range(4):
  old=pair(c,d,5);depth=pair(c,d,85);auto=pair(c,d,95)
  results.append({k:c[k] for k in ['family','resolution','distance']}|{'density':d,'chosen':c['decisions'][d],'old_ms':old,'depth_ms':depth,'auto_ms':auto,'delta_percent':100*(auto/old-1),'chosen_path_delta_percent':100*(auto/pair(c,d,c['decisions'][d])-1),'vs_fastest_percent':100*(auto/min(old,depth)-1),'selector_us':c['selector_us'][d]})
ratios=[r['delta_percent'] for r in results]
analysis={'points':len(results),'depth_selected':sum(r['chosen']==85 for r in results),'faster_over3pct':sum(v< -3 for v in ratios),'slower_over3pct':sum(v>3 for v in ratios),'median_delta_percent':st.median(ratios),'range_delta_percent':[min(ratios),max(ratios)],'selector_us_median':st.median(r['selector_us'] for r in results),'selector_us_max':max(r['selector_us'] for r in results),'depth_choice_slower_over3pct':sum(r['chosen']==85 and r['depth_ms']>r['old_ms']*1.03 for r in results),'fallback_delta_range':[min(r['delta_percent'] for r in results if r['chosen']==5),max(r['delta_percent'] for r in results if r['chosen']==5)],'points_detail':results}
(ROOT/'analysis.json').write_text(json.dumps(analysis,indent=2))
s=Path('scripts/cost_scaling_report.html').read_text()
s=s.replace('Previsão calibrada: testar fora da calibração','Escolha automática: anterior ou material por pixel')
s=s.replace('<option value="4">Calibrado · experimental</option><option value="3" selected>Híbrido anterior</option><option value="2">Global com contorno</option><option value="1">Regional</option>', '<option value="3" selected>Automático · experimental</option><option value="2">Sempre profundidade</option><option value="1">Anterior</option>')
s=s.replace("labels=['Completa','Regional','Global contorno','Híbrido anterior','Calibrado']", "labels=['Completa','Anterior','Profundidade','Automático']")
s=s.replace('max="8" value="2"','max="18" value="0"').replace('Math.min(r.frame,8-r.frame)','Math.min(r.frame,18-r.frame)').replace("r.frame<=4?'Afastando'","r.frame<=9?'Afastando'").replace('${r.frame+1}/9','${r.frame+1}/19').replace('a.frame===8-r.frame','a.frame===18-r.frame').replace('.value)+1)%9','.value)+1)%19')
start=s.index(' const a=DATA.assets.find');end=s.index('\n const t=',start)
s=s[:start]+''' const a=DATA.assets.find(a=>a.family===r.family&&a.factor===r.factor);$('memory').textContent=`Asset e níveis ${fmt(a.bytes/1048576,2)}MiB; modelo pequeno na CPU. Automático escolheu ${r.chosen===85?'profundidade + material':'anterior'}. Resultado float idêntico ao caminho escolhido: ${r.chosen_image_identical?'sim':'não'}. Sem alvo RGBA32F intermediário; depth existente é amostrado quando necessário. Payload não é RSS/VRAM.`;'''+s[end:]
start=s.index(' const total=i=>');end=s.index('\n $(\'status\')',start)
s=s[:start]+''' const total=i=>(t[i].throughput_ms.median+t[i+labels.length].throughput_ms.median)/2;$('decision').textContent=`Automático: ${(100*(total(3)/total(1)-1)).toFixed(1)}% contra anterior; ${(100*(total(3)/Math.min(total(1),total(2))-1)).toFixed(1)}% contra o mais rápido medido. Positivo significa custo maior. Timing inclui decisão e todos os passes; pequenas diferenças exigem consultar as duplicatas.`;'''+s[end:]
start=s.index('<div class="panel"><h2>Calibração e limites</h2>');end=s.index('__TRANSITIONS__',start)
s=s[:start]+'''<div class="panel"><h2>Protocolo e limites</h2><p>Calibração12/16/24/48/96/192; modelo congelado antes das dez poses de validação. Regra usa carga selecionada e projeção em pixels, nunca família/densidade/distância como chave. Exige benefício acima da margem de erro e5% do anterior previsto; fora do domínio usa anterior. Mantém0,15px e os gates antigos. Duas superfícies sintéticas, material único, sem sombras/MSAA/transparência. Não é generalização para qualquer asset/GPU nem custo constante perto. As duas escolhas têm pequenas diferençasfloat, portanto trocas foram verificadas separadamente. Ruído e previsões erradas continuam possíveis.</p><p><a href="fit.json">Calibração e hash</a> · <a href="validation.json">Tempos e amostras brutas</a> · <a href="repeat.json">Repetição</a> · <a href="quality.json">Qualidade</a> · <a href="transitions.json">Trocas</a> · <a href="verification.json">Verificação</a></p></div>'''+s[end:]
findings=f'<div class="panel"><h2>Seleção conservadora, medida fora da calibração</h2><p>Profundidade escolhida em{analysis["depth_selected"]}/{len(results)}pontos. Automático: {analysis["faster_over3pct"]}pontos mais de3% mais rápidos e{analysis["slower_over3pct"]}mais de3% mais lentos que anterior. Em{analysis["depth_choice_slower_over3pct"]}pontos, o modelo escolheu profundidade apesar de esse caminho medir mais de3% acima do anterior. Nos demais, diferenças também incluem variação entre execuções do mesmo pipeline. Faixa descritiva, sem promessa estatística. Margem calibrada{fit["margin_ms"]:.4f}ms; custo isolado do seletor mediano{analysis["selector_us_median"]:.3f}µs.</p><div class="scroll"><table><tr><th>Relevo nativo · ms/frame</th><th>1×</th><th>2×</th><th>10×</th><th>100×</th></tr>'
for dataset,label in [(cs,'Validação'),(repeat,'Repetição')]:
 for distance in [14,224]:
  c=next(c for c in dataset if c['family']==1 and c['resolution']==0 and c['distance']==distance)
  for code,name in [(0,'Completa'),(5,'Anterior'),(95,'Automático')]:
   findings+=f'<tr><td>{label} · distância{distance} · {name}</td>'+''.join(f'<td>{pair(c,d,code):.4f}</td>' for d in range(4))+'</tr>'
findings+='</table></div><p>As curvas usam tempos reais do automático, incluindo fallback; não são o mínimo dos tempos dos dois candidatos. Evidência automatizada não substitui aceite visual humano.</p></div>'
assets=[json.loads(p.read_text()) for p in sorted(Path('artifacts/appearance-probe/hybrid-scaling').glob('f*_d*.json'))]
data={'frameCount':19,'modes':['full','old','depth','automatic'],'rows':rows,'summary':summary,'assets':assets}
s=s.replace('__FINDINGS__',findings).replace('__TRANSITIONS__','').replace('__DATA__',json.dumps(data,separators=(',',':')))
(ROOT/'index.html').write_text(s);(ROOT/'summary.json').write_text(json.dumps(summary,indent=2))
print(json.dumps({k:v for k,v in analysis.items() if k!='points_detail'},indent=2))
