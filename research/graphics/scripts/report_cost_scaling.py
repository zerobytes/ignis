"""Held-out results; compare calibrated selector against both candidates and old policy."""
import json,statistics as st
from pathlib import Path
import report_triangle_scaling as base
ROOT=Path('artifacts/appearance-probe/cost-scaling')
base.ROOT=ROOT;base.KEYS=['gpu_ms','throughput_ms','cpu_ms']
d=json.loads((ROOT/'report.json').read_text());assert len(d['rows'])==144
base.apply_confirmation(d);assert len(d['confirmation'])==20
d['summary']=base.summarize(d);(ROOT/'summary.json').write_text(json.dumps(d['summary'],indent=2))
old_rows={r['id']:r for r in json.loads(Path('artifacts/appearance-probe/hybrid-scaling/report.json').read_text())['rows']}
regrets=[]
for r in d['rows']:
    if not r['timings']:continue
    v={k:[base.pair(r,m,k)for m in range(5)]for k in base.KEYS}
    regrets.append({'id':r['id'],'family':r['family'],'factor':r['factor'],'resolution':r['resolution'],'distance':r['distance'],'strategy':r['strategy'],'visited':r['visited'],'prediction_only_regret':v['throughput_ms'][2 if r['strategy']==4 else 1]/min(v['throughput_ms'][1:3])-1,'old_choice_only_regret':v['throughput_ms'][2 if old_rows[r['id']]['strategy']==4 else 1]/min(v['throughput_ms'][1:3])-1,'predicted_delta_ms':r['predicted_delta_ms'],'measured_delta_ms':v['throughput_ms'][2]-v['throughput_ms'][1],'costs':v,'regret_vs_eligible':{k:a[4]/min(a[1:3])-1 for k,a in v.items()},'old_regret':{k:a[3]/min(a[1:3])-1 for k,a in v.items()},'ratio_vs_old':{k:a[4]/a[3]for k,a in v.items()}})
(ROOT/'regret.json').write_text(json.dumps(regrets,indent=2))
transitions=json.loads((ROOT/'transitions.json').read_text());passed=sum(r['checks'][3]['quality_gate']for r in d['rows'])+sum(t['check']['quality_gate']for t in transitions)
worst=max(r['regret_vs_eligible']['throughput_ms']for r in regrets);typical=st.median(r['regret_vs_eligible']['throughput_ms']for r in regrets)
ratios=[c['timings'][i+20]['gpu_ms']['median']/c['timings'][i]['gpu_ms']['median']for c in d['confirmation']for i in range(20)]
old_worst=max(r['old_regret']['throughput_ms']for r in regrets)
old_choices=sum(r['old_choice_only_regret']==0 for r in regrets);new_choices=sum(r['prediction_only_regret']==0 for r in regrets)
findings=f'<div class="panel"><h2>Resultado: ainda não aprovado como substituto geral</h2><p>O modelo escolheu o menor tempo medido em {new_choices}/80casos; o anterior em {old_choices}/80. O pior excesso do anterior nesta mesma rodada foi {old_worst*100:.1f}%. Diferenças pequenas incluem ruído e não comprovam um vencedor estável.</p><p>Qualidade: {passed}/{144+len(transitions)}poses aprovadas. Excesso mediano de {typical*100:.1f}% e máximo de {worst*100:.1f}% sobre o menor tempo dos candidatos elegíveis. Inclui custo de seleção e variação da medição. Valores positivos são regressões.</p><p class="note">GPU sustentada: razão entre controles duplicados {min(ratios):.2f}–{max(ratios):.2f}. <a href="fit.json">Calibração</a> · <a href="regret.json">Resultados independentes</a> · <a href="confirmation.json">Amostras</a> · <a href="recheck.json">Repetição dos casos críticos</a> · <a href="selection-cpu.json">Custo do seletor</a>. Na repetição, o maior excesso inicial (relevo100× distante) não persistiu; a escolha errada do plano10× próximo persistiu. Os resultados originais continuam publicados. Menor tempo medido não equivale a garantia universal de vencedor.</p></div>'
gallery='<div class="panel"><h2>Trocas de estratégia fora da curva</h2><p>Poses dos dois lados de cada troca encontrada em14…224, passo0,5. Fora da calibração e do timing.</p>'
for t in transitions:
    gallery+=f'<p>{t["size"][0]}×{t["size"][1]}, distância {t["distance"]}, {"global"if t["strategy"]==4 else"regional"}; RMSE {t["check"]["foreground_rmse"]:.6f}.</p><div class="overview">'
    for m,label in [(0,'Completa'),(4,'Calibrado')]:gallery+=f'<figure><figcaption>{label}</figcaption><a href="{t["id"]}_{m}.png"><img loading="lazy" style="width:100%" src="{t["id"]}_{m}.png" alt="{label}"></a></figure>'
    gallery+='</div>'
gallery+='</div>'
for r in d['rows']:
    if r['timings']:
        for t in r['timings']:
            for k in base.KEYS:t[k]={'median':t[k]['median']}
            t.pop('samples_ms',None);t.pop('gpu_samples_ms',None)
d.pop('confirmation',None)
html=Path(__file__).with_name('cost_scaling_report.html').read_text().replace('__FINDINGS__',findings).replace('__TRANSITIONS__',gallery).replace('__DATA__',json.dumps(d,separators=(',',':')))
(ROOT/'index.html').write_text(html)
print('Median/worst regret',typical,worst,'quality',passed,'transitions',len(transitions))
