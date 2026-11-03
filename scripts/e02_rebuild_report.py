"""Publish fixed-window comparisons and an execution ledger for the E02 study."""
from collections import Counter
import html
import json
from pathlib import Path
import shutil

import numpy as np
from PIL import Image, ImageDraw

from e02_rebuild import ROOT, PROJECT

DIAGNOSES={
'A':'Organização principal, ala, varanda e pátio presentes. Profundidades hipotéticas; fechamento posterior e espessuras incompletos.',
'B':'Curva principal e bordas aproximadas; aba/forro esquemáticos, encontros e pano posterior pendentes. Telhas ainda largas e com interferências.',
'C':'Pano superior curvo presente; profundidade, sobreposição da galeria e fáscia divergem.',
'D':'Curva baixa/retorno presentes; forma ainda suave demais, espessura e nervuras incompletas.',
'E':'Cinco remates e fiadas desencontradas presentes. Telhas quadrangulares, distribuição muito menos densa, desgaste e lâminas simplificados.',
'F':'Vão alto, campo verde e pilares presentes; grande área lisa indevida no frontão e encontros do arco não reproduzidos.',
'G':'Cinco vãos pequenos construídos. Perfil, bases, quantidade visível parcial e relação entre coberturas ainda divergentes.',
'H':'Quatro colunas nativas e arcos/ramificações dianteiros. Fuste retangular sem sulcos esculpidos; nervuras e retorno incompletos.',
'I':'Três janelas pequenas e juntas de parede aproximadas. Blocos, peitoris, vidro e manchas sem correspondência fina.',
'J':'Porta dupla, motivo próprio, ferragens, dois vasos e três degraus próprios. Ramos, perfil de moldura e encaixes simplificados.',
'K':'Porta esquerda, ornamento e vaso presentes. Orientação do retorno e espessuras/folhas ocultas permanecem hipóteses.',
'L':'Duas redes distintas, ombreiras e objetos internos presentes. Grafo ainda divergente; vidro transparente e reflexos não construídos.',
'M':'Duas janelas laterais em plano próprio, árvores e pilares. Embasamento, peitoris e coroas ainda genéricos; sem vidro real.',
'N':'Abertura da água-furtada e faces de cobertura próprias presentes. Volume muito raso, fiadas/encaixe e proporções divergentes.',
'O':'Dois percursos vazados, sete módulos à esquerda e três à direita. Percurso, espessuras, curvas e motivos aproximados; retorno direito irregular.',
'P':'Dois pilares perfilados com campos verdes frontais, relevo claro lateral e vasos. Seções, ornamentos, vasos e arranjos ainda grosseiros.',
'Q':'Escadas independentes, paginação e inlays planos presentes. Medalhão sem folhas internas corretas; pavimentação/bordas/cotas e cobertura de lajes divergem.',
'R':'Banco e cadeira separados, vasos, prateleiras/livros. Apoios e formas internas muito simplificados; mobiliário parcialmente oculto ainda pendente.',
'S':'Quatro corpos externos e um interior, com tampa/base/suporte. São materiais coloridos, não fontes locais emissivas; manchas de luz quente ausentes.',
'T':'Arbustos, maciços, plantas em vasos, roxas e trepadeiras presentes. Densidade, ramificação, flores e oclusões muito inferiores à referência; musgo de juntas ausente.',
'U':'Câmera declarada e luz/sombra configuradas. Materiais procedurais aproximados, sem desgaste esculpido nem luz local; fundo e contrastes divergem.'}


def main():
    data=json.loads((PROJECT/'docs/references/image-to-3d/e02-construction-list.json').read_text())
    scene=json.loads((ROOT/'house.scene.json').read_text())
    parts=json.loads((ROOT/'components.json').read_text())
    mapping={}
    for asset,entries in parts.items():
        for part in entries:
            for item in part['items']:mapping.setdefault(item,[]).append({'asset':'rebuild-'+asset,'component':part['name'],'vertices':len(part['vertex_indices'])})
    for i in range(1,7):mapping.setdefault(f'E02-H{i:02}',[]).append({'plant':'rebuild-column','instances':[f'column-{j}' for j in range(4)]})
    for item,source in [('U01','stone.pattern'),('U02','stone.detail'),('U05','wood.pattern'),('U06','gold.reflectance'),('U07','directional-shadow'),('U08','camera'),('U09','directional-light'),('U10','studio-ground/background')]:
        mapping.setdefault('E02-'+item,[]).append({'configuration':source})
    for i in [4,5,9,10,12]:mapping.setdefault(f'E02-T{i:02}',[]).append({'asset':'rebuild-leaf-cluster','instances':12})
    # Relations reference the actual components on both sides of the relation.
    relations={
        'A05':['M11'],'A06':['A02','Q01'],'A07':['Q10','Q01','J01'],'A10':['Q01','Q12'],
        'B11':['B01','C01'],'C07':['C01','F03'],'C08':['C01','D01','G01'],
        'D09':['D01','A01'],'D11':['D01','G01'], 'F06':['F07'],
        'J13':['J11','J14','J15'],'K06':['K01','Q01'],'K07':['I08','K03'],
        'L01':['L02'],'L16':['L05','R11'],'M07':['L18']}
    for item,linked in relations.items():
        for other in linked:mapping.setdefault('E02-'+item,[]).extend(mapping.get('E02-'+other,[]))
    mapping['E02-A08']=[{'configuration':'spatial-plan.json: main/wing/side host planes'}]
    items=[]
    for group in data['groups']:
        for task in group['items']:
            refs=mapping.get(task['id'],[])
            items.append({**task,'construction':'parcial' if refs else 'não construído','validation':'divergente' if refs else 'não executada','sources':refs,'diagnosis':DIAGNOSES[group['id']],'capture':'artifacts/house-e02-rebuild/detail.png'})
    valid_ids={t['id'] for t in items}
    assert len(items)==248 and set(mapping)<=valid_ids
    ledger={'version':1,'baseline':'763ba8d','fidelity_accepted':False,'policy':'Parcial = existe alguma representação vinculada; não certifica todas as propriedades do item. Comparação por grupo não valida individualmente seus subitens.','counts':dict(Counter(t['construction'] for t in items)),'items':items}
    out=PROJECT/'docs/references/image-to-3d/e02-rebuild-progress.json';out.write_text(json.dumps(ledger,ensure_ascii=False,indent=2)+'\n')
    (ROOT/'progress.json').write_text(json.dumps(ledger,ensure_ascii=False,indent=2)+'\n')
    shutil.copyfile(ROOT/'house.scene.json',PROJECT/'examples/house-e02-rebuild.scene.json')
    crops=json.loads((PROJECT/'artifacts/house-e02-analysis/crops.json').read_text())['crops']
    ref=Image.open(ROOT/'reference.png').convert('RGB');render=Image.open(ROOT/'detail.png').convert('RGB')
    compared=[c for c in crops if c['kind']!='coverage']
    for c in compared:
        x,y,w,hh=c['box'];box=(x,y,x+w,y+hh)
        ref.crop(box).save(ROOT/(c['id']+'-ref.png'));render.crop(box).save(ROOT/(c['id']+'-render.png'))
    # Contact sheets are display thumbnails only. The saved crops above retain native pixels.
    for batch in range(0,len(compared),4):
        page=Image.new('RGB',(1400,950),'#e6e0d3');draw=ImageDraw.Draw(page)
        for j,c in enumerate(compared[batch:batch+4]):
            for col,kind in enumerate(['ref','render']):
                im=Image.open(ROOT/(c['id']+'-'+kind+'.png'));im.thumbnail((675,205))
                page.paste(im,(col*700+(700-im.width)//2,j*237+26));draw.text((col*700+15,j*237+6),c['id']+' '+kind,fill='#171717')
        page.save(ROOT/f'pairs-{batch//4}.png')
    # These orthographic client projections use actual MCP-saved geometry, not a blueprint API.
    assets={a['id']:a['geometry'] for a in scene['meshes']['assets']}
    world=[]
    for inst in scene['meshes']['instances']:
        assert inst['rotation']==[0.0,0.0,0.0,1.0]
        g=assets[inst['asset']];p=np.array(g['positions'])*inst['scale']+inst['position']
        world.append((inst['asset'],p,g['triangles']))
    page=Image.new('RGB',(1500,850),'#edf2f0');draw=ImageDraw.Draw(page)
    for panel,(axes,title) in enumerate([((0,2),'Vista superior — hipotese espacial'),((0,1),'Elevacao X/Y — geometria salva')]):
        allp=np.concatenate([p[:,axes] for _,p,_ in world]);lo=allp.min(0);hi=allp.max(0);scale=min(680/(hi[0]-lo[0]),730/(hi[1]-lo[1]))
        for name,p,triangles in world:
            if name.startswith(('rebuild-leaf','rebuild-white','rebuild-purple')):continue
            q=(p[:,axes]-(lo+hi)/2)*scale+[panel*750+375,440]
            if axes==(0,1):q[:,1]=880-q[:,1]
            color='#7e908c' if 'roof' in name else '#697579' if 'column' in name else '#bac3c0'
            for t in triangles:draw.line([tuple(q[i]) for i in t+[t[0]]],fill=color,width=1)
        draw.text((panel*750+25,20),title,fill='#1d3537')
    draw.text((25,820),'Projecoes no cliente. Nao ha exportador blueprint nativo. Apenas as colunas usam fonte de planta parametrica.',fill='#26383b')
    page.save(ROOT/'blueprints.png')
    # Native column source, in its own coordinates, projected from inspect_mesh.
    g=json.loads((ROOT/'column-mesh.json').read_text())['asset']['geometry'];p=np.array(g['positions'])
    im=Image.new('RGB',(520,650),'#edf2f0');d=ImageDraw.Draw(im)
    for t in g['triangles']:
        ps=[(260+float(p[i,0])*180,330-float(p[i,1])*180) for i in t+[t[0]]];d.line(ps,fill='#607878',width=1)
    d.text((20,20),'Planta nativa da coluna — inspect_mesh',fill='#203b3b');im.save(ROOT/'column-source.png')
    metrics=json.loads((ROOT/'comparison.json').read_text())[0]['report']['metrics']
    cards=''.join('<details><summary>'+html.escape(c['id']+' — '+c['label'])+'</summary><div class="pair"><img src="'+c['id']+'-ref.png"><img src="'+c['id']+'-render.png"></div></details>' for c in compared)
    rows=''.join('<tr><td>'+t['id']+'</td><td>'+html.escape(t['build'])+'</td><td>'+t['construction']+'</td></tr>' for t in items)
    diagnoses=''.join('<p><b>'+g+'</b> '+html.escape(d)+'</p>' for g,d in DIAGNOSES.items())
    page='''<!doctype html><html lang="pt-BR"><meta charset="utf-8"><title>E02 — reconstrução por lista</title><style>body{margin:30px;background:#202523;color:#e7e4da;font:16px system-ui}h1{font-size:25px}.pair{display:grid;grid-template-columns:1fr 1fr;gap:14px}img{width:100%;object-fit:contain}summary{padding:15px;cursor:pointer}details{border-top:1px solid #56605b}td{padding:8px;border-bottom:1px solid #45514b}td:first-child{white-space:nowrap}a{color:#b5d8c5}p{max-width:1100px}table{border-collapse:collapse}</style><h1>E02 — tentativa a partir dos 248 itens</h1><p>Referência à esquerda; captura Ignis à direita. Reconstrução parcial e divergente. Presença de uma peça não significa validação. Profundidade, fundos e interior continuam hipóteses.</p><div class="pair"><img src="reference.png"><img src="detail.png"></div><p>MAE '''+f'{metrics["rgb_mae"]:.6f}'+', SSIM '+f'{metrics["ssim_luma"]:.6f}'+''' — não são percentuais de fidelidade.</p><details><summary>Vista auxiliar e projeções da geometria salva</summary><div class="pair"><img src="auxiliary.png"><img src="blueprints.png"></div><img style="width:400px" src="column-source.png"></details>'''+cards+'<h2>Revisão visual por grupo</h2>'+diagnoses+'<h2>Estado dos 248 itens</h2><p>'+html.escape(str(ledger['counts']))+'</p><table>'+rows+'</table><p><a href="house.scene.json">Cena Ignis editável</a> · <a href="progress.json">Registro por item</a> · <a href="verification.json">Integridade</a></p></html>'
    (ROOT/'index.html').write_text(page)
    print(json.dumps({'counts':ledger['counts'],'metrics':metrics,'assets':len(assets),'instances':len(world),'vertices':sum(len(g['positions']) for g in assets.values()),'triangles':sum(len(g['triangles']) for g in assets.values())},ensure_ascii=False))


if __name__=='__main__':main()
