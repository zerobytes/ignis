"""Regional experiment reuses the EAF-84 charts and unchanged quality thresholds."""
import json
from pathlib import Path
import report_triangle_scaling as base
ROOT=Path('artifacts/appearance-probe/regional-scaling')

def main():
    base.ROOT=ROOT
    data=json.loads((ROOT/'report.json').read_text())
    assert len(data['rows'])==144
    base.apply_confirmation(data)
    assert len(data['confirmation'])==20
    data['summary']=base.summarize(data)
    (ROOT/'summary.json').write_text(json.dumps(data['summary'],indent=2))
    for r in data['rows']:
        if r['timings']:
            for t in r['timings']:
                for k in base.KEYS:t[k]={'median':t[k]['median']}
    data.pop('confirmation',None)
    html=Path(__file__).with_name('triangle_scaling_report.html').read_text()
    html=html.replace('EAF-84','EAF-86').replace('100× mais triângulos precisa custar 100× mais?','Regiões menores: até onde o custo estabiliza?')
    html=html.replace('Forma + normais em textura','64 regiões · projeção precisa').replace('Forma + guarda das normais','Global · projeção precisa').replace('Somente forma','Global · limite anterior')
    html=html.replace("labels=['Completa','Global · limite anterior','Guarda das normais','Normais em textura']","labels=['Completa','Global anterior','Global preciso','Regional preciso']")
    html=html.replace('Vazão mede blocos de8frames; nos pontos confirmados,128frames','Vazão mede blocos de128frames')
    html=html.replace('com assets carregados separadamente','com quatro densidades residentes e alternadas')
    start=html.index('Somente forma considera') if 'Somente forma considera' in html else html.index('Global · limite anterior considera')
    end=html.index('</p><p>O plano',start)
    html=html[:start]+('Todos os candidatos usam o mesmo campo de normais fonte e erro de posição ≤0,15pixel. O controle global anterior usa uma esfera. O global preciso usa a caixa e a direção do deslocamento. O regional seleciona64partes com caixas locais e preserva as polilinhas fonte compartilhadas, com até64draws. O custo de seleção e das fronteiras está incluído. Regiões não garantem custo menor: veja os controles e as falhas de qualidade.')+html[end:]
    old='Nível escolhido: ${r.levels[m]}; erro de posição preparado ${fmt(a.levels[r.levels[m]].position_error,6)} unidades.'
    assert old in html
    html=html.replace(old,'64regiões preparadas; decisões locais por erro projetado, com bordas fonte conservadas.')
    transitions=json.loads((ROOT/'transitions.json').read_text())
    gallery='<div class="panel"><h2>Posições intermediárias: regiões em níveis diferentes</h2><p>Fonte100×. Cada par mostra referência completa e regional na resolução indicada. Imagens gerais reduzidas para caber; abra a captura para inspecionar pixels físicos. Estes pontos verificam qualidade nas transições; não entram na curva cronometrada.</p>'
    for t in transitions:
        q=t['check']
        gallery+=f'<p>Distância {t["distance"]} · {t["size"][0]} × {t["size"][1]} · {t["draws"]} trechos · {t["triangles"]:,} triângulos · RMSE {q["foreground_rmse"]:.6f} · {"Passou" if q["quality_gate"] else "Falhou"}</p><div class="overview">'
        for m,label in [(0,'Completa'),(3,'Regional')]:
            gallery+=f'<figure><figcaption>{label}</figcaption><a href="{t["id"]}_{m}.png"><img loading="lazy" style="width:100%" src="{t["id"]}_{m}.png" alt="{label}, distância {t["distance"]}"></a></figure>'
        gallery+='</div>'
    html=html.replace('<script>',gallery+'</div><script>',1)
    html=html.replace('__DATA__',json.dumps(data,separators=(',',':')))
    (ROOT/'index.html').write_text(html)
    for s in data['summary']:
        print(s['family'],s['resolution'],round(s['distance']),[(m['mode'],round(m['growth_100_over_1']['throughput_ms'],2),m['quality_all_densities']) for m in s['modes']])
if __name__=='__main__':main()
