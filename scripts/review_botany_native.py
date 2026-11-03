#!/usr/bin/env python3
"""Build a local comparison page with native renders and original reference links."""
from pathlib import Path
from verify_lab import ROOT

def main():
    out=ROOT/'artifacts/botany-native'
    refs={
        'palmeirinha':('Areca · touceira pinada','https://www.windyridgegardencentre.ie/products/dypsis-lutescens-areca-160-cm'),
        'platano':('Plátano · copa e ramificação','https://trees.stanford.edu/ENCYC/PLATace.htm'),
        'eucalipto':('Eucalipto · copa aberta','https://www.gleneira.vic.gov.au/our-city/classified-trees/classified-tree-register/357ctr2022-hawthorn-road'),
        'figueira':('Figueira · copa ornamental ampla','https://www.gardenershq.com/Ficus-benjamina.php')}
    html="""<!doctype html><meta charset="utf-8"><title>Botânica · referências e iteração nativa</title><style>body{font:16px system-ui;background:#18202b;color:#edf1e9;margin:32px auto;max-width:1500px;padding:0 24px}a{color:#a3d8be}.row{display:grid;grid-template-columns:repeat(3,1fr);gap:14px}img{width:100%;background:#222c38}figure{margin:0}figcaption{padding:8px 0;color:#b6c7bc}.curves{display:grid;grid-template-columns:repeat(4,1fr);gap:12px}h2{margin-top:48px}p{max-width:900px;line-height:1.6}summary{cursor:pointer}</style><h1>Botânica · referências e iteração nativa</h1><p>Capturas do renderer Ignis, sem pós-produção ou imagens geradas. Referências externas abrem no site original. Comparativo anterior/depois usa a mesma espécie; enquadramentos foram ajustados para a nova copa, portanto não mede proporção por pixel. Estrutura sem folhas usa a mesma câmera do exemplar novo.</p>"""
    for key,(label,url) in refs.items():
        html+=f'<h2>{label}</h2><p><a href="{url}" target="_blank">Abrir referência fotográfica</a></p><div class="row">'
        for src,text in [(f'baseline/{key}.png','Antes · V3'),(f'{key}.png','Revisão atual'),(f'{key}-structure.png','Estrutura sem folhas')]:
            html+=f'<figure><img src="{src}"><figcaption>{text}</figcaption></figure>'
        html+='</div>'
    html+='<h2>Uma fonte, quatro curvas</h2><p>Mesmos órgãos, folhas, material e câmera. O campo altera a curva de repouso e transporta os descendentes. A resposta ao vento usa depois o solver material existente.</p><div class="curves">'
    for key,label in [('straight','Campo zero'),('arch','Curvatura base → ponta'),('sag','Pendência'),('irregular','Torção e irregularidade 3D')]:
        html+=f'<figure><img src="curve-{key}.png"><figcaption>{label}</figcaption></figure>'
    html+='</div><details><summary>Resposta dinâmica da curva irregular ao vento</summary><img style="max-width:900px" src="curve-wind.png"></details><p>Ensaio físico autoral local; não representa colisão de gameplay Rapier ou tecido vascular calibrado. As receitas são aproximações configuráveis de arquitetura e folhagem; sem aceite artístico humano presumido.</p>'
    (out/'review.html').write_text(html)
if __name__=='__main__':main()
