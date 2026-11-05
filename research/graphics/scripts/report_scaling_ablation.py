"""Current ablation report: physical-pixel crops and both successful/failed interventions."""
import json,statistics as st
from pathlib import Path
from analyze_scaling_ablation import ROOT,pair
q=json.loads((ROOT/'quality.json').read_text())
cs=json.loads((ROOT/'depth.json').read_text())
depth={r['id']:r for r in json.loads((ROOT/'depth-quality.json').read_text())}
assert len(q)==208 and len(cs)==28
assets=[json.loads(p.read_text()) for p in sorted(Path('artifacts/appearance-probe/hybrid-scaling').glob('f*_d*.json'))]
assert len(assets)==8
mapping=[0,1,2,3,4];codes=[0,5,45,75,85];names=['full','hybrid','branchless','rgba32_surface','depth_only']
rows=[]
for r in q:
    c=next(c for c in cs if all(c[k]==r[k]for k in ['family','resolution','distance']))
    n=len(c['codes']);h=len(c['timings'])//2;d=r['density']
    times=[{k:{'median':c['timings'][d*n+m+extra][k]['median']}for k in ['gpu_ms','throughput_ms','cpu_ms']}for extra in [0,h]for m in mapping]
    rows.append({k:r[k]for k in ['id','family','density','resolution','size','distance','frame','triangles','covered_pixels']}|{'factor':[1,2,10,100][d],'checks':[r['old_check']]+[r['checks'][i]['quality']for i in [2,6]]+[depth[r['id']]['quality']],'timings':times,'source_triangles_per_covered_pixel':r['triangles'][0]/r['covered_pixels']})
summary=[]
keys=['throughput_ms','gpu_ms','cpu_ms']
for c in cs:
    modes=[];n=len(c['codes']);h=len(c['timings'])//2
    for mi,code in enumerate(codes):
        curves={k:[pair(c,d,code,k)for d in range(4)]for k in keys}
        noise={k:[c['timings'][d*n+c['codes'].index(code)+h][k]['median']/c['timings'][d*n+c['codes'].index(code)][k]['median']for d in range(4)]for k in keys}
        quality=mi==0 or all(r['checks'][mi-1]['quality_gate']for r in rows if all(r[k]==c[k]for k in ['family','resolution','distance']))
        modes.append({'curve_ms':curves,'growth_100_over_1':{k:v[-1]/v[0]for k,v in curves.items()},'curve_max_over_min':{k:max(v)/min(v)for k,v in curves.items()},'quality_all_densities':quality,'duplicate_ratio_range':{k:[min(v),max(v)]for k,v in noise.items()}})
    summary.append({k:c[k]for k in ['family','resolution']}|{'frame':[14,21,28,56,84,112,224].index(c['distance']),'batch_frames':128,'modes':modes})
data={'frameCount':13,'modes':names,'rows':rows,'assets':assets,'summary':summary}
# Reuse the established crop/playback implementation; replace experiment-specific labels.
s=Path('scripts/cost_scaling_report.html').read_text()
s=s.replace('Previsão calibrada: testar fora da calibração','Onde está o custo: material por pixel visível')
s=s.replace('<option value="4">Calibrado · experimental</option><option value="3" selected>Híbrido anterior</option><option value="2">Global com contorno</option><option value="1">Regional</option>', '<option value="4" selected>Profundidade + material · experimental</option><option value="3">Dois passes com buffer RGBA32F</option><option value="2">Sem bifurcação</option><option value="1">Híbrido anterior</option>')
s=s.replace("labels=['Completa','Regional','Global contorno','Híbrido anterior','Calibrado']", "labels=['Completa','Anterior','Sem bifurcação','Buffer RGBA32F','Só profundidade']")
s=s.replace('max="8" value="2"','max="12" value="0"').replace('Math.min(r.frame,8-r.frame)','Math.min(r.frame,12-r.frame)').replace("r.frame<=4?'Afastando'","r.frame<=6?'Afastando'").replace('${r.frame+1}/9','${r.frame+1}/13').replace('a.frame===8-r.frame','a.frame===12-r.frame').replace('.value)+1)%9','.value)+1)%13')
start=s.index(' const a=DATA.assets.find');end=s.index('\n const t=',start)
s=s[:start]+''' const a=DATA.assets.find(a=>a.family===r.family&&a.factor===r.factor);$('memory').textContent=`Asset e todos os níveis: ${fmt(a.bytes/1048576,2)}MiB. Segundo passe: buffer RGBA32F adicional de ${fmt(r.size[0]*r.size[1]*16/1048576,2)}MiB no modo de dois passes; profundidade reutiliza o alvo depth32 existente e não acrescenta textura por pixel. O comparador mantém o alvo RGBA32F para o controle; payload não é RSS/VRAM medido. Readback e driver à parte. Mesmo seletor de geometria nos quatro candidatos; sem previsão nova ou escolha por vencedor medido.`;'''+s[end:]
start=s.index(' const total=i=>');end=s.index('\n $(\'status\')',start)
s=s[:start]+''' const total=i=>(t[i].throughput_ms.median+t[i+labels.length].throughput_ms.median)/2;$('decision').textContent=`Caminho selecionado: ${(100*(total(m)/total(1)-1)).toFixed(1)}% de custo em relação ao híbrido anterior. Positivo significa regressão. Inclui todos os passes e seleção, sem captura/readback. Diferenças pequenas exigem consultar controles duplicados.`;'''+s[end:]
start=s.index('<div class="panel"><h2>Calibração e limites</h2>');end=s.index('__TRANSITIONS__',start)
s=s[:start]+'''<div class="panel"><h2>Limites e reprodução</h2><p>Plano e relevo sintéticos com material opaco único; amostra única, sem sombras/MSAA/transparência. Este comparador usa um heightfield cuja normal fonte é recuperável por worldXZ; a variante de profundidade depende desse preparo e não se aplica automaticamente a malhas arbitrárias. A ablação flat muda o trabalho do shader: diferenças não são uma contabilidade aditiva de estágios da GPU. Segundo passe reconstrói posição a partir da profundidade, o que pode mudar arredondamentos; comparar os gates numéricos e os recortes físicos. Novas poses21/84 não participaram do diagnóstico inicial. Esta pesquisa não comprova uma engine AAA ou custo constante de perto.</p><p><a href="diagnostic.json">Diagnóstico bruto</a> · <a href="order.json">Ordem dos triângulos</a> · <a href="depth.json">Tempos finais</a> · <a href="confirmation.json">Primeira rodada comRGBA32F</a> · <a href="quality.json">Qualidade por pose</a> · <a href="verification.json">Verificação</a></p></div>
'''+s[end:]
ratios=[pair(c,d,85)/pair(c,d,5)-1 for c in cs for d in range(4)]
near=next(c for c in cs if c['family']==1 and c['resolution']==0 and c['distance']==14)
far=next(c for c in cs if c['family']==1 and c['resolution']==0 and c['distance']==224)
transitions=json.loads((ROOT/'depth-transitions.json').read_text())
passed=sum(r['quality']['quality_gate']for r in depth.values())+sum(r['quality']['quality_gate']for r in transitions)
findings=f'<div class="panel"><h2>Ganho localizado; substituição geral ainda não aprovada</h2><p>Relevo100× próximo, resolução nativa: híbrido anterior {pair(near,3,5):.4f}ms → profundidade {pair(near,3,85):.4f}ms ({(pair(near,3,85)/pair(near,3,5)-1)*100:+.1f}%). São tempos CPU+GPU sobrepostos, incluindo seleção e os dois passes. O passeRGBA32F foi mais caro em todos os112pontos da sua primeira rodada.</p><p>Profundidade: {sum(v < -.03 for v in ratios)}/112pontos com custo mais de3% menor, {sum(v > .03 for v in ratios)} com custo mais de3% maior. Essa faixa é descritiva, não um teste de significância. Qualidade {passed}/228poses aprovada nos gates originais, cobertura idêntica; não significa identidadefloat ou aceite humano.</p><p>Uma curva mais plana pode esconder custo fixo maior: comparar os tempos absolutos de cada densidade. Mesmo seletor global/regional nos candidatos; nenhuma tabela de vencedores nem nova política automática foi adicionada.</p><div class="scroll"><table><tr><th>Relevo3456×2234</th><th>1×</th><th>2×</th><th>10×</th><th>100×</th></tr>'
for label,c in [('Perto14',near),('Longe224',far)]:
    for code,name in [(0,'Completa'),(5,'Anterior'),(85,'Profundidade')]:
        findings+=f'<tr><td>{label} · {name}</td>'+''.join(f'<td>{pair(c,d,code):.4f}ms</td>'for d in range(4))+'</tr>'
repeat=next(c for c in json.loads((ROOT/'depth-recheck.json').read_text())if c['family']==1 and c['resolution']==0 and c['distance']==14)
findings+=f'</table></div><p>Repetição do relevo100× próximo: {pair(repeat,3,5):.4f}→{pair(repeat,3,85):.4f}ms ({(pair(repeat,3,85)/pair(repeat,3,5)-1)*100:+.1f}%). O ganho persistiu, com magnitude menor; não interpretar a rodada inicial como garantia fixa.</p>'
findings+='<p><a href="depth-recheck.json">Repetição dos casos críticos</a> · <a href="depth.json">Rodada final completa</a> · <a href="depth-transitions.json">Qualidade nas trocas de estratégia</a></p></div>'

s=s.replace('img.src=`${r.id}_${key}.png`','img.src=`${r.id}_${[0,1,2,4,5][key]}.png`')
s=s.replace('__FINDINGS__',findings).replace('__TRANSITIONS__','').replace('__DATA__',json.dumps(data,separators=(',',':')))
(ROOT/'index.html').write_text(s)
(ROOT/'summary.json').write_text(json.dumps(summary,indent=2))
print('report',len(rows),'poses',len(summary),'cohorts')
