"""Extract original-pixel evidence and publish the E02 pre-construction inventory.

No engine process, model generation or previous house recipe is used here.
The semantic observations live in the reviewed JSON; this script does not infer them.
"""
import argparse
import hashlib
import json
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

PROJECT = Path(__file__).resolve().parents[1]
ROOT = PROJECT / 'artifacts/house-e02-analysis'
SOURCE = Path('/Volumes/Zero/EpicGames/Unreal Projects/LandOfChaos/docs/art/settlements/residential-concepts-v2/E02.png')
EXPECTED = '4153a456f9fdf2e9e6f43328171692156db7563c2062654199e66c0c540882ad'
DATA = PROJECT / 'docs/references/image-to-3d/e02-construction-list.json'
REPORT = PROJECT / 'docs/e02-construction-list.md'
CONTEXTS = [
    ('C01', 'Cobertura principal', [670, 10, 710, 520]),
    ('C02', 'Coberturas da ala', [65, 170, 790, 405]),
    ('C03', 'Galeria superior', [470, 315, 390, 150]),
    ('C04', 'Varanda e apoios', [90, 455, 565, 330]),
    ('C05', 'Entrada central', [565, 465, 285, 285]),
    ('C06', 'Janela principal', [825, 345, 350, 450]),
    ('C07', 'Fachada lateral direita', [1110, 430, 250, 440]),
    ('C08', 'Água-furtada', [1080, 195, 200, 260]),
    ('C09', 'Balaustrada esquerda', [120, 690, 450, 330]),
    ('C10', 'Balaustrada direita', [850, 700, 300, 320]),
    ('C11', 'Piso e acesso', [400, 665, 600, 375]),
    ('C12', 'Vegetação esquerda', [10, 565, 345, 390]),
    ('C13', 'Vegetação direita', [1070, 620, 325, 365]),
    ('C14', 'Frontão, janela alta e trepadeira', [735, 115, 365, 610]),
    ('C15', 'Entrada esquerda e mobiliário', [150, 525, 430, 220]),
    ('C16', 'Pavimentação externa', [20, 900, 1355, 190]),
]
DETAILS = [
    ('D01', 'Perfil da coluna', [387, 485, 85, 250], 'C04'),
    ('D02', 'Pilar esquerdo de acesso', [450, 770, 115, 235], 'C09'),
    ('D03', 'Motivo da balaustrada', [275, 765, 170, 165], 'C09'),
    ('D04', 'Ramificação da grande janela', [905, 405, 185, 265], 'C06'),
    ('D05', 'Folhas da porta central', [683, 500, 112, 190], 'C05'),
    ('D06', 'Lanterna e suporte', [635, 510, 58, 98], 'C05'),
    ('D07', 'Banco e cadeira', [320, 580, 220, 140], 'C15'),
    ('D08', 'Trepadeira da fachada alta', [810, 250, 83, 455], 'C14'),
    ('D09', 'Telhas e sobreposição', [890, 145, 205, 170], 'C01'),
    ('D10', 'Flores e folhagem baixa', [1090, 780, 210, 160], 'C13'),
    ('D11', 'Medalhão e faixa sinuosa', [560, 730, 235, 120], 'C11'),
    ('D12', 'Arremate da cumeeira', [680, 15, 95, 150], 'C01'),
]


def prepare():
    ROOT.mkdir(parents=True, exist_ok=True)
    im = Image.open(SOURCE)
    if im.size != (1402, 1122) or hashlib.sha256(SOURCE.read_bytes()).hexdigest() != EXPECTED:
        raise ValueError('Reference does not match the supplied E02 image')
    crops = []
    for row, (y0, y1) in enumerate([(0, 420), (360, 800), (740, 1122)]):
        for col, (x0, x1) in enumerate([(0, 400), (340, 740), (680, 1080), (1020, 1402)]):
            crops.append({'id': f'G{row*4+col+1:02}', 'label': f'Varredura {row+1}/{col+1}',
                          'kind': 'coverage', 'box': [x0, y0, x1-x0, y1-y0], 'parent': None})
    crops += [{'id': id, 'label': label, 'box': box, 'kind': 'context', 'parent': None}
              for id, label, box in CONTEXTS]
    crops += [{'id': id, 'label': label, 'box': box, 'kind': 'detail', 'parent': parent}
              for id, label, box, parent in DETAILS]
    mask = Image.new('L', im.size, 0)
    md = ImageDraw.Draw(mask)
    for c in crops:
        x, y, w, h = c['box']
        if min(x, y) < 0 or min(w, h) <= 0 or x+w > im.width or y+h > im.height:
            raise ValueError(c)
        path = ROOT / (c['id']+'.png')
        im.crop((x, y, x+w, y+h)).save(path)
        c['path'] = str(path)
        if c['kind'] == 'coverage': md.rectangle((x, y, x+w-1, y+h-1), fill=255)
    if mask.getextrema() != (255, 255): raise AssertionError('Pixel coverage has gaps')
    manifest = {'source': str(SOURCE), 'source_sha256': EXPECTED, 'dimensions': list(im.size),
                'coordinates': 'top-left; x right; y down; x,y,width,height; original pixels',
                'coverage': '12 overlapping rectangles cover every source pixel; this is not semantic completeness',
                'crops': crops}
    (ROOT/'crops.json').write_text(json.dumps(manifest, indent=2, ensure_ascii=False)+'\n')
    overview = im.copy()
    draw = ImageDraw.Draw(overview)
    f = ImageFont.truetype('/System/Library/Fonts/Supplemental/Arial.ttf', 19)
    for c in crops:
        if c['kind'] != 'context': continue
        x,y,w,h = c['box'];draw.rectangle((x,y,x+w-1,y+h-1), outline='#c22235', width=2)
        draw.rectangle((x,y,x+43,y+25), fill='#fff1df');draw.text((x+3,y+2),c['id'],fill='#8b1224',font=f)
    overview.save(ROOT/'crop-map.png')
    print(json.dumps({'crops': len(crops), 'coverage': manifest['coverage'], 'source': EXPECTED}))


def publish():
    data = json.loads(DATA.read_text())
    manifest = json.loads((ROOT/'crops.json').read_text())
    crops = {c['id']: c for c in manifest['crops']}
    all_items = [item for group in data['groups'] for item in group['items']]
    ids = {i['id'] for i in all_items}
    assert len(ids) == len(all_items)
    for group in data['groups']:
        assert group['evidence'] and all(e in crops for e in group['evidence'])
        for i in group['items']:
            assert i['build'] and i['check'] and i['state'] in ['observado', 'hipótese', 'desconhecido']
            assert all(e in crops for e in i.get('evidence', group['evidence']))
            assert i['construction'] == 'não construído' and i['validation'] == 'não executada'
    reviewed = set(data['crop_review'])
    assert reviewed == set(crops)
    for c in data['revisits']:
        assert all(i in ids for i in c['affected_items'])
    lines = ['# E02 — lista consolidada de coisas para construir', '',
             '**Entrega anterior à construção. Todos os itens estão não construídos; validação do modelo não executada.**', '',
             data['scope'], '', f"Fonte: {SOURCE}. Resolução1402×1122. SHA256 `{EXPECTED}`.", '',
             f"{len(data['groups'])} conjuntos; **{len(all_items)} tarefas construtivas**;40recortes reais (12de cobertura,16de contexto,12de detalhe).", '',
             'Quantidade de tarefas não mede completude nem fidelidade. Famílias repetidas preservam distribuição/variações; contagens ilegíveis ficam explícitas.', '',
             '## Leitura global e inventário inicial', '', *['- '+s for s in data['initial_inventory']], '',
             '## Organização espacial observada e hipóteses', '', *['- '+s for s in data['spatial_reading']], '',
             '## Revisitações após os recortes', '']
    for r in data['revisits']:
        lines += [f"- **{r['id']}** — {r['finding']} Itens: {', '.join(r['affected_items'])}."]
    lines += ['', '## Lista completa consolidada', '',
              'Cada grupo identifica contexto, ocorrência e relações. Essas informações são herdadas pelos itens, com exceções registradas na própria linha. O critério de cada linha será verificado na referência e numa captura futura; não é uma aprovação atual.', '']
    for g in data['groups']:
        lines += [f"### {g['id']} — {g['title']}", '',
                  f"**Contexto/evidência:** {', '.join('['+e+']('+crops[e]['path']+')' for e in g['evidence'])}.", '',
                  f"**Ocorrência:** {g['occurrence']}", '', f"**Relações/dependências:** {g['relations']}", '',
                  '| ID | O que construir / descrição observada | Conferir depois | Estado da leitura |',
                  '|---|---|---|---|']
        for i in g['items']:
            extra = ''
            if i.get('evidence'):extra = ' Evidência: '+', '.join(i['evidence'])+'.'
            if i.get('uncertainty'):extra += ' Dúvida: '+i['uncertainty']
            lines.append(f"| {i['id']} | {i['build']}{extra} | {i['check']} | {i['state']} |")
        lines.append('')
    lines += ['## Leitura de cada recorte e rastreabilidade', '', '| Recorte | Caixa na fonte | Leitura / refinamento |', '|---|---|---|']
    for id,c in crops.items():
        lines.append(f"| [{id} — {c['label']}]({c['path']}) | {c['box']} | {data['crop_review'][id]} |")
    lines += ['', '## Pendências e limites anteriores à autoria', '', *['- '+x for x in data['unknowns']], '',
              '## Fronteira desta entrega', '',
              'Nenhuma geometria, planta paramétrica, cena ou render foi gerado nesta aplicação. A próxima fase só começa após o usuário examinar esta lista, conforme seu pedido. A lista é a especificação visual atual, aberta a correções; não uma prova automática de percepção completa.', '',
              f"Dados estruturados: [{DATA.name}]({DATA}). Extração e publicação: [script]({PROJECT/'scripts/e02_construction_inventory.py'}).", '']
    REPORT.write_text('\n'.join(lines))
    print(json.dumps({'report': str(REPORT), 'groups': len(data['groups']), 'items': len(all_items),
                      'crops_reviewed': len(reviewed), 'all_not_built': True}, ensure_ascii=False))


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--publish', action='store_true')
    args = parser.parse_args()
    publish() if args.publish else prepare()
