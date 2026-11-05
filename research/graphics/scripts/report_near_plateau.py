"""Build the near-camera report without changing eligibility or frozen cost model."""
import json, statistics as st
from pathlib import Path
ROOT=Path('artifacts/appearance-probe/near-plateau')
CODES=[0,95,85,105,115,135]
DISTANCES=[10,14,28,224]
def read(name): return json.loads((ROOT/name).read_text())
def pair(c,d,code,k='throughput_ms'):
 n=len(c['codes']);i=d*n+c['codes'].index(code);h=len(c['samples'])//2
 return st.mean(c['samples'][j][k]['median'] for j in [i,i+h])
cs=read('validation.json');repeat=read('repeat.json');qs=read('quality.json');bake=read('bake.json')
footprints=read('footprints.json')
assets=[json.loads(p.read_text()) for p in sorted(Path('artifacts/appearance-probe/hybrid-scaling').glob('f*_d*.json'))]
keys=['throughput_ms','gpu_ms','cpu_ms'];rows=[];summary=[]
for r in qs:
 c=next(c for c in cs if all(c[k]==r[k] for k in ['family','resolution','distance']))
 a=next(a for a in assets if a['family']==r['family'] and a['factor']==[1,2,10,100][r['density']])
 d=r['density'];h=len(c['samples'])//2
 times=[{k:{'median':c['samples'][d*6+m+extra][k]['median']} for k in keys} for extra in [0,h] for m in range(6)]
 rows.append(r|{'timings':times,'factor':a['factor'],'triangles':[a['levels'][-1]['count']//3]+[r['triangles']]*5,'source_triangles_per_covered_pixel':a['levels'][-1]['count']/3/r['covered_pixels']})
for c in cs:
 modes=[]
 for mi,code in enumerate(CODES):
  curves={k:[pair(c,d,code,k) for d in range(4)] for k in keys}
  noise={k:[c['samples'][d*6+mi+24][k]['median']/c['samples'][d*6+mi][k]['median'] for d in range(4)] for k in keys}
  modes.append({'curve_ms':curves,'growth_100_over_1':{k:v[-1]/v[0] for k,v in curves.items()},'curve_max_over_min':{k:max(v)/min(v) for k,v in curves.items()},'quality_all_densities':mi==0 or all(r['checks'][mi-1]['quality_gate'] for r in rows if all(r[k]==c[k] for k in ['family','resolution','distance'])),'duplicate_ratio_range':{k:[min(v),max(v)] for k,v in noise.items()}})
 summary.append({k:c[k] for k in ['family','resolution']}|{'frame':DISTANCES.index(c['distance']),'batch_frames':128,'modes':modes})
analysis={}
for dataset,label in [(cs,'validation'),(repeat,'repeat')]:
 results=[]
 for c in dataset:
  for d in range(4):
   results.append({k:c[k] for k in ['family','resolution','distance']}|{'density':d,'auto_old_ms':pair(c,d,95),'auto_candidate_ms':pair(c,d,135),'delta_percent':100*(pair(c,d,135)/pair(c,d,95)-1),'packed_delta_percent':100*(pair(c,d,105)/pair(c,d,85)-1),'spatial_delta_percent':100*(pair(c,d,115)/pair(c,d,85)-1)})
 ratios=[r['delta_percent'] for r in results]
 duplicates=[c['samples'][i+len(c['samples'])//2]['throughput_ms']['median']/c['samples'][i]['throughput_ms']['median'] for c in dataset for i in range(len(c['samples'])//2)]
 analysis[label]={'points':len(results),'faster_over3pct':sum(v< -3 for v in ratios),'slower_over3pct':sum(v>3 for v in ratios),'median_delta_percent':st.median(ratios),'range_delta_percent':[min(ratios),max(ratios)],'duplicate_ratio_range':[min(duplicates),max(duplicates)],'points_detail':results}
(ROOT/'analysis.json').write_text(json.dumps(analysis,indent=2))
s=Path('scripts/cost_scaling_report.html').read_text()
s=s.replace('Previsão calibrada: testar fora da calibração','Câmera perto: mesmos detalhes, outra organização')
start=s.index('<select id="algorithm">');end=s.index('</select>',start)
s=s[:start]+'''<select id="algorithm"><option value="1">Automático anterior</option><option value="2">Profundidade anterior</option><option value="3">Posições compactas</option><option value="4">Compactas + ordem espacial</option><option value="5" selected>Automático + ordem espacial · experimento</option>'''+s[end:]
s=s.replace("labels=['Completa','Regional','Global contorno','Híbrido anterior','Calibrado']", "labels=['Completa','Auto anterior','Profundidade','Compactas','Espacial','Auto espacial']")
s=s.replace("'#e0a5fa']","'#e0a5fa','#60d9e8']").replace('${65+m*160}','${65+m*135}')
s=s.replace('max="8" value="2"','max="6" value="1"').replace('Math.min(r.frame,8-r.frame)','Math.min(r.frame,6-r.frame)').replace("r.frame<=4?'Afastando'","r.frame<=3?'Afastando'").replace('${r.frame+1}/9','${r.frame+1}/7').replace('a.frame===8-r.frame','a.frame===6-r.frame').replace('.value)+1)%9','.value)+1)%7')
start=s.index(' const a=DATA.assets.find');end=s.index('\n const t=',start)
s=s[:start]+''' const a=DATA.assets.find(a=>a.family===r.family&&a.factor===r.factor),b=DATA.bake.filter(b=>b.id===a.id);$('memory').textContent=`Fonte e níveis ${fmt(a.bytes/1048576,2)}MiB. Buffers extras do comparador: ${b.map(b=>b.variant+' '+fmt((b.positions_bytes+b.index_bytes)/1048576,2)+'MiB').join('; ')}. Não é economia de memória. Igualdade RGBA32F: compacto/depth ${r.exact_rgba32f[0]}, espacial/depth ${r.exact_rgba32f[1]}, auto novo/anterior ${r.exact_rgba32f[2]}.`;'''+s[end:]
start=s.index(' const total=i=>');end=s.index('\n $(\'status\')',start)
s=s[:start]+''' const total=i=>(t[i].throughput_ms.median+t[i+labels.length].throughput_ms.median)/2;$('decision').textContent=`Automático com ordem espacial: ${(100*(total(5)/total(1)-1)).toFixed(1)}% contra automático anterior. Mesmo seletor e modelo congelado. Positivo significa custo maior. Consulte também profundidade/compacto/espacial para separar as intervenções e as duplicatas para avaliar a variação.`;'''+s[end:]
start=s.index('<div class="panel"><h2>Calibração e limites</h2>');end=s.index('__TRANSITIONS__',start)
s=s[:start]+'''<div class="panel"><h2>Protocolo e limites</h2><p>Dois experimentos sem simplificar: posições f32 compactadas por intervalo de LOD; depois ordenação espacial dos mesmos triângulos dentro do intervalo. Mesmos índices de seleção, coordenadas, winding, normais, material e limite0,15px. Modelo automático anterior congelado. Preparação fora do frame; buffers adicionais explicitados. Plano redundante e relevo com detalhes adicionais não são assets AAA. O teste pode reduzir constantes, mas não comprova independência da densidade.</p><p>Diagnóstico dos passes: timestamps Metal se sobrepõem; não somar intervalos. Ablação185/285 remove dependências e muda cache; os timestamps isolados não sustentam atribuição física. Dados brutos preservados; conclusão usa vazão sem timestamps e comparações completas.</p><p><a href="validation.json">Tempos brutos</a> · <a href="repeat.json">Repetição</a> · <a href="quality.json">Qualidade</a> · <a href="bake.json">Memória/preparo</a> · <a href="footprints.json">Áreas projetadas</a> · <a href="stages.json">Timestamps</a> · <a href="isolation.json">Ablações</a> · <a href="verification.json">Verificação</a></p></div>'''+s[end:]
findings='<div class="panel"><h2>Comparação medida</h2><p>Intervenção experimental; o objetivo de custo constante próximo continua em investigação. As tabelas mostram tempo total, incluindo seleção e ambos os passes, no Apple M4 Pro/Metal. Não são FPS de jogo.</p><div class="scroll"><table><tr><th>Relevo nativo · ms/frame</th><th>1×</th><th>2×</th><th>10×</th><th>100×</th></tr>'
for dataset,label in [(cs,'Validação'),(repeat,'Repetição')]:
 for distance in ([10,14,224] if label=='Validação' else [14,224]):
  c=next(c for c in dataset if c['family']==1 and c['resolution']==0 and c['distance']==distance)
  for code,name in [(0,'Completa'),(95,'Auto anterior'),(135,'Auto espacial')]:
   findings+=f'<tr><td>{label} · distância{distance} · {name}</td>'+''.join(f'<td>{pair(c,d,code):.4f}</td>' for d in range(4))+'</tr>'
findings+='</table></div>'
for label,a in analysis.items():
 findings+=f'<p>{label}: {a["faster_over3pct"]}/{a["points"]} pontos mais de3% rápidos; {a["slower_over3pct"]} mais de3% lentos. Mediana {a["median_delta_percent"]:+.2f}%. Razão de duplicatas {a["duplicate_ratio_range"][0]:.2f}–{a["duplicate_ratio_range"][1]:.2f}. Faixa descritiva, sem significância estatística estabelecida.</p>'
findings+='<h2>O que100× significa neste teste</h2><p>100× é relativo a32.768triângulos, não ao número de pixels. A área projetada abaixo usa triângulos fonte inteiramente dentro do frustum; não determina oclusão nem invisibilidade. Área menor que1pixel² não garante erro visual abaixo de0,15px.</p><div class="scroll"><table><tr><th>Relevo100× nativo</th><th>Triângulos fonte / pixels cobertos</th><th>Área média por triângulo (px²)</th><th>Triângulos com área &lt;1px²</th></tr>'
for distance in DISTANCES:
 f=next(f for f in footprints if f['family']==1 and f['density']==3 and f['resolution']==0 and f['distance']==distance)
 r=next(r for r in rows if r['family']==1 and r['density']==3 and r['resolution']==0 and r['distance']==distance)
 findings+=f'<tr><td>Distância{distance}</td><td>{r["source_triangles_per_covered_pixel"]:.2f}</td><td>{f["mean_area_px2"]:.3f}</td><td>{100*sum(f["counts"][:2])/f["fully_inside"]:.1f}%</td></tr>'
findings+='</table></div><p>Manter amostras visíveis pode exigir trabalho adicional. O controle plano mede subdivisão redundante; o relevo acrescenta frequências com as densidades. Esta bateria ainda não demonstra centenas de triângulos fonte por pixel com câmera próxima.</p></div>' 
data={'frameCount':7,'modes':['full','auto_old','depth','packed','spatial','auto_spatial'],'rows':rows,'summary':summary,'assets':assets,'bake':bake}
s=s.replace('__FINDINGS__',findings).replace('__TRANSITIONS__','').replace('__DATA__',json.dumps(data,separators=(',',':')))
(ROOT/'index.html').write_text(s);(ROOT/'summary.json').write_text(json.dumps(summary,indent=2))
print(json.dumps({k:{a:b for a,b in v.items() if a!='points_detail'} for k,v in analysis.items()},indent=2))
