"""Report all controls, quality failures, and measured regret of the cost model."""
import json
import statistics as st
from pathlib import Path
import report_triangle_scaling as base
ROOT=Path('artifacts/appearance-probe/hybrid-scaling')

def main():
    base.ROOT=ROOT
    data=json.loads((ROOT/'report.json').read_text());assert len(data['rows'])==144
    base.apply_confirmation(data);assert len(data['confirmation'])==20
    data['summary']=base.summarize(data)
    (ROOT/'summary.json').write_text(json.dumps(data['summary'],indent=2))
    regrets=[]
    for r in data['rows']:
        if not r['timings']:continue
        costs={k:[base.pair(r,m,k)for m in range(5)]for k in base.KEYS}
        eligible=[m for m in [2,3] if r['checks'][m-1]['quality_gate']]
        regrets.append({'id':r['id'],'family':r['family'],'factor':r['factor'],'resolution':r['resolution'],'distance':r['distance'],'strategy':r['strategy'],'visited':r['visited'],'quality':r['checks'][3]['quality_gate'],'costs':costs,'regret_vs_eligible':{k:v[4]/min(v[m]for m in eligible)-1 if eligible else None for k,v in costs.items()},'ratio_vs_regional':{k:v[4]/v[2]for k,v in costs.items()}})
    (ROOT/'regret.json').write_text(json.dumps(regrets,indent=2))
    worst=max(r['regret_vs_eligible']['throughput_ms']for r in regrets)
    typical=st.median(r['regret_vs_eligible']['throughput_ms']for r in regrets)
    ratios=[c['timings'][i+20]['gpu_ms']['median']/c['timings'][i]['gpu_ms']['median']for c in data['confirmation']for i in range(20)]
    findings=f'<div class="panel"><h2>Resultado e limite da escolha</h2><p>O híbrido preservou os critérios de qualidade nas144poses da curva e nas20poses de troca verificadas. Ainda não escolhe sempre o menor tempo real: excesso mediano de {typical*100:.1f}% e máximo de {worst*100:.1f}% em relação ao melhor candidato elegível medido. O global antigo continua como controle, com sua falha conhecida de cobertura.</p><p class="note">TemposGPU isolados tiveram grande dispersão: razão entre controles duplicados {min(ratios):.2f}–{max(ratios):.2f}. A curva padrão usa vazão em blocos, incluindo o seletor; diferenças pequenas e picosGPU não demonstram por si só ganho/regressão estável. Dados completos: <a href="regret.json">comparações</a> e <a href="confirmation.json">medições</a>.</p></div>'

    for r in data['rows']:
        if r['timings']:
            for t in r['timings']:
                for k in base.KEYS:t[k]={'median':t[k]['median']}
    data.pop('confirmation',None)
    html=Path(__file__).with_name('hybrid_scaling_report.html').read_text()
    transitions=json.loads((ROOT/'transitions.json').read_text())
    gallery='<div class="panel"><h2>Trocas entre global e regional</h2><p>Fonte100×; poses dos dois lados de cada troca detectada entre14 e224. Referência à esquerda, híbrido à direita. Capturas de qualidade, fora da curva cronometrada.</p>'
    for t in transitions:
        q=t['check'];strategy='global com contorno'if t['strategy']==4 else'regional'
        gallery+=f'<p>{t["size"][0]}×{t["size"][1]}, distância {t["distance"]}: {strategy}, {t["triangles"]:,}triângulos, {t["draws"]}draws. RMSE {q["foreground_rmse"]:.6f}; {"passou"if q["quality_gate"]else"falhou"}.</p><div class="overview">'
        for m,label in [(0,'Completa'),(4,'Híbrido')]:
            gallery+=f'<figure><figcaption>{label}</figcaption><a href="{t["id"]}_{m}.png"><img loading="lazy" style="width:100%" src="{t["id"]}_{m}.png" alt="{label}"></a></figure>'
        gallery+='</div>'
    gallery+='</div>'
    (ROOT/'index.html').write_text(html.replace('__FINDINGS__',findings).replace('__TRANSITIONS__',gallery).replace('__DATA__',json.dumps(data,separators=(',',':'))))
    for s in data['summary']:
        m=s['modes'][4];print(s['family'],s['resolution'],round(s['distance']),[round(v,4)for v in m['curve_ms']['throughput_ms']],m['quality_all_densities'])
if __name__=='__main__':main()
