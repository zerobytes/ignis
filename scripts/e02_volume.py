"""Fresh E02 image study and volumetric authorship through the public Ignis MCP."""
import argparse
import hashlib
import html
import json
import math
from pathlib import Path
import random
import subprocess

import numpy as np
from PIL import Image

PROJECT = Path(__file__).resolve().parents[1]
ROOT = PROJECT/'artifacts/house-e02-volume'
SOURCE = Path('/Volumes/Zero/EpicGames/Unreal Projects/LandOfChaos/docs/art/settlements/residential-concepts-v2/E02.png')
BINARY = '/Volumes/Zero/dev/games/.ignis-build/image-to-3d/release/ignis'
DATA = PROJECT/'docs/references/image-to-3d/e02-volume-analysis.json'

REGIONS = [
 ('S01-main-body',(770,160,1380,885)),('S02-wing-porch',(70,280,830,750)),
 ('S03-roof-structure',(75,25,1380,540)),('S04-patio-access',(110,680,1140,1015)),
 ('D01-main-window',(860,365,1140,760)),('D02-upper-gable',(760,165,1045,430)),
 ('D03-dormer',(1090,210,1285,400)),('D04-side-windows',(1110,445,1330,760)),
 ('D05-porch-left',(100,480,400,760)),('D06-porch-center',(385,470,660,740)),
 ('D07-entry',(610,470,835,730)),('D08-gallery',(470,305,825,475)),
 ('D09-rail-left',(135,730,540,955)),('D10-rail-right',(845,755,1135,1010)),
 ('D11-medallion',(420,710,910,945)),('D12-finials',(680,10,835,195)),
 ('D13-eave',(65,330,530,565)),('D14-garden',(1080,710,1395,950)),
 ('D15-ornament',(930,430,1090,675)),('D16-entry-branch',(685,510,790,680)),
 ('D17-tile-sample',(965,160,1110,250)),('D18-porch-capitals',(265,485,610,620))]

# Re-read from the source and these crops; no prior inventory or recipe is loaded.
GROUPS = [
 ('ORG','Organização espacial',['S01-main-body','S02-wing-porch','S04-patio-access'],[
 'Casa composta por corpo alto à direita e ala baixa alongada à esquerda',
 'Corpo alto tem profundidade: fachada de grande janela e face lateral com duas janelas',
 'Ala baixa fechada recua em relação ao alinhamento dianteiro da varanda',
 'Varanda coberta é espaço aberto diante de portas e janelas da ala baixa',
 'Pátio descoberto ocupa a área diante da varanda e do corpo alto',
 'Galeria de janelas fica acima da cobertura baixa da varanda e abaixo do telhado da ala',
 'Porta principal abre da parede posterior da varanda para o espaço coberto',
 'Porta esquerda também pertence à parede posterior, não à linha dos arcos dianteiros',
 'Escada externa passa entre dois pilares de entrada para o pátio',
 'Escada da porta é distinta da escada externa e eleva a soleira',
 'Traseiras e profundidades completas são hipóteses: manter continuidade sem copiar fachadas visíveis',
 'Identificar em planta os volumes, faces hospedeiras e trajetos escada-pátio-varanda-porta']),
 ('COR','Corpos e paredes',['S01-main-body','S02-wing-porch','D01-main-window'],[
 'Base de alvenaria sob a janela grande tem fiadas e espessura aparente',
 'Parede frontal do corpo alto contorna o vão principal e sobe até a cobertura',
 'Parede lateral direita retorna em profundidade e hospeda duas janelas separadas',
 'Cunhal entre fachada grande e lateral é um encontro comum, sem planos afastados',
 'Pilarete alto à esquerda da janela grande se prolonga até a cobertura superior',
 'Ala baixa tem parede contínua atrás de banco, portas e janelas',
 'Piso interior e paredes com espessura constituem hipótese habitável, sem fachada solta',
 'As aberturas atravessam a espessura e têm jambas, soleiras e continuidade interna',
 'Fechar laterais e fundos não observados como hipótese simples coerente',
 'Paredes da galeria têm base, separações entre vãos e ligação com a ala baixa',
 'Alvenaria elevada acompanha os pisos em vez de terminar suspensa',
 'Definir encontro entre ala e corpo alto sem fenda estrutural',
 'Hipótese acrescentada na revisão espacial: passagem interna atravessa as paredes comuns entre ala e corpo alto']),
 ('RM','Cobertura alta',['S03-roof-structure','D02-upper-gable','D12-finials'],[
 'Cobertura alta tem pano largo visível com queda para a lateral direita',
 'Curva do pano tem mudança suave de inclinação: não é uma placa vertical',
 'Borda dianteira diagonal acompanha a fachada alta com espessura e acabamento',
 'Ponta alta dianteira se projeta e sobe além da superfície principal',
 'Cumeeira se prolonga em profundidade até um remate posterior visível',
 'Beiral lateral projeta-se além das paredes e conecta a borda dianteira à traseira',
 'Fáscia clara e filetes dourados percorrem as bordas como peças contínuas',
 'Forro escuro da ponta alta tem travessas sob a cobertura',
 'Escoras sob o beiral conectam parede e cobertura',
 'Remate dianteiro tem lança, contorno foliforme e miolo vazado',
 'Remate posterior repete a família com leitura parcialmente oclusa',
 'Panos não vistos completam uma cobertura plausível e são inferências',
 'Contato entre parede alta e cobertura deve existir em toda a linha hospedeira',
 'A água-furtada emerge do pano lateral e não flutua sobre ele']),
 ('RA','Cobertura da ala',['S03-roof-structure','D08-gallery','D17-tile-sample'],[
 'Telhado da ala é independente da cobertura baixa da varanda',
 'Pano frontal cobre a galeria e sobe para a linha superior posterior',
 'Bordas e linha superior têm curvatura e extremidades elevadas',
 'Remate sobre a extremidade esquerda/posterior marca a separação das coberturas',
 'Beiral frontal se projeta sobre a fileira de janelas altas',
 'Acabamento claro contorna a borda e pequenos suportes aparecem sob ela',
 'Fiadas de telhas acompanham o pano, com juntas desencontradas',
 'Encontro com o corpo alto precisa cobertura contínua, com interseção declarada',
 'Completar pano posterior e espessura sem derivar geometria da câmera',
 'Não preencher o intervalo da galeria com o telhado da varanda']),
 ('RV','Cobertura da varanda',['S02-wing-porch','D13-eave'],[
 'Cobertura baixa avança além da parede e cobre a circulação da varanda',
 'Frente do beiral tem traçado curvo em planta e elevação',
 'Extremidade esquerda retorna em curva e forma um pano de fechamento de cobertura',
 'Espigão curvo no lado esquerdo separa direções das fiadas',
 'Ponta esquerda sobe e tem remate próprio',
 'Faixa clara de beiral tem espessura, bordas e filete interno',
 'Forro fica acima dos arcos e possui espessura visível',
 'Beiral se apoia na linha de colunas e se conecta à parede posterior',
 'Junção com galeria mantém suas janelas visíveis',
 'Extremo junto ao corpo alto encontra parede e suporte sem intervalo vazio',
 'Telhas baixas têm contorno arredondado e faixas de tamanho distinto das bordas',
 'Cobertura deve continuar em 3D ao girar a câmera']),
 ('COL','Suportes e arcos',['D05-porch-left','D06-porch-center','D18-porch-capitals'],[
 'Quatro apoios dianteiros distinguíveis, contando apoio extremo e apoios intermediários',
 'Cada apoio tem base larga escalonada, fuste fino e transição superior',
 'Fustes claros têm perfis e depressões longitudinais',
 'Capitéis abrem em nervuras que sobem para os arcos',
 'Arcos apontados ligam apoios vizinhos e deixam circulação vazia abaixo',
 'Arco de retorno no canto esquerdo liga alinhamentos diferentes da varanda',
 'Nervuras periféricas acompanham o arco e se unem às nervuras de coluna',
 'Miolos dos encontros têm ramos e campos vazados em vez de triângulos maciços',
 'Viga de contorno sustenta a fáscia e recebe o topo dos arcos',
 'Bases tocam o piso da varanda, não o piso externo mais baixo',
 'Pilaretes junto à alvenaria compartilham linguagem de perfil dos apoios livres',
 'Apoios superiores da fachada alta conectam efetivamente parede e beiral']),
 ('POR','Portas',['D07-entry','D16-entry-branch','D05-porch-left'],[
 'Porta principal tem vão alto apontado na parede ao fundo da varanda',
 'Duas folhas de madeira com junta central e painéis inferiores',
 'Moldura de pedra tem camadas em relevo e acompanha o arco até a soleira',
 'Árvore dourada central se abre em dois pares de ramos ascendentes',
 'Ramos secundários das folhas da porta são finos e curvos, com terminações distintas',
 'Montante e ramificações pertencem à superfície das folhas, sem atravessar o vão',
 'Ferragem pequena aparece na metade inferior da folha',
 'Três degraus rasos curvos conduzem à soleira principal',
 'Porta esquerda fica parcialmente encoberta pela coluna/arco dianteiro',
 'Porta esquerda também tem moldura apontada e painéis de madeira',
 'Há motivo ramificado na porta esquerda, menor e parcialmente ocluso',
 'Aberturas, fundos e espessuras das duas portas devem estar ligados às paredes']),
 ('JAN','Janela grande e fachada alta',['D01-main-window','D02-upper-gable','D15-ornament'],[
 'Vão grande apontado domina a fachada direita com moldura externa espessa',
 'Múltiplas faixas de moldura acompanham arco e montantes laterais',
 'Peitoril tem projeção e friso abaixo do vidro',
 'Tronco claro central é contínuo e chega ao topo sem olho oval grande',
 'Primeiro par de ramos claros abre para as laterais e delimita vazios foliformes',
 'Segundo par abre mais alto, em escala menor e conectado ao tronco',
 'Divisões douradas mais finas existem atrás/entre os ramos claros',
 'Vidro ocupa os campos e permite ler objetos no interior; transparência depende de capacidade',
 'Há vasos em posições distintas atrás da janela e prateleiras ao fundo',
 'Campo triangular superior tem fundo verde, arco curvo e motivo dourado próprio',
 'Faixa alta estreita à esquerda tem abertura ramificada e vegetação subindo ao lado',
 'Molduras, campo superior, pilarete e cobertura devem compartilhar os encontros']),
 ('JL','Janelas laterais e baixas',['D04-side-windows','D06-porch-center','D07-entry'],[
 'Duas janelas apontadas estão na face lateral direita em planos de mesma parede',
 'Pilarete entre elas tem base, fuste, relevo e transição para suporte de beiral',
 'Peitoris laterais se projetam além da parede e retornam nas extremidades',
 'Arcos laterais têm contorno exterior e rede interior de ramos dourados',
 'Campos verdes ornamentados e flores ficam acima das janelas laterais',
 'Três janelas estreitas legíveis na parede da varanda entre as portas',
 'Cada janela baixa tem moldura apontada, peitoril e montante ramificado',
 'Vidros/fundos ficam recuados em relação à moldura',
 'Objetos e luz no interior são parciais e não autorizam inventar uma planta observada',
 'Pilares da varanda ocluem parte dos vãos sem transformar estes em arcos da varanda']),
 ('GAL','Galeria',['D08-gallery'],[
 'Fileira alta tem pelo menos cinco vãos distinguíveis, extremo esquerdo parcialmente ocluso',
 'Vãos são apontados e separados por montantes claros',
 'Base/peitoril contínuo acompanha a ala',
 'Molduras de cada vão têm perfis e bordas independentes',
 'Motivo interior repete tronco central e bifurcações simétricas',
 'Painéis escuros/vidros ficam atrás dos ramos',
 'Encontro superior fica sob o beiral da ala e não sob o telhado baixo',
 'Encontro direito tem pilarete alto comum ao corpo principal']),
 ('AF','Água-furtada',['D03-dormer'],[
 'Pequeno corpo sai do pano inclinado da cobertura principal',
 'Fachada apontada tem moldura saliente e abertura ramificada',
 'Cobertura própria tem pano visível e borda oposta parcialmente oclusa',
 'Duas laterais e encontro posterior com o telhado são inferidos para formar volume',
 'Faixa clara contorna a cobertura menor',
 'Pequeno remate foliforme sobe do encontro das bordas',
 'Rede dourada tem haste central e bifurcações',
 'Abertura/fundo se encaixam na espessura da fachada menor']),
 ('BAL','Balaustradas e pilares de acesso',['D09-rail-left','D10-rail-right'],[
 'Balaustrada esquerda liga extremo da varanda ao pilar esquerdo da entrada',
 'Cinco coroas principais completas distinguíveis no trecho esquerdo; extremos parcialmente oclusos',
 'Cada módulo tem dois montantes/perfis laterais e arco apontado',
 'Interior dos módulos é vazado e contém árvore clara',
 'Tronco dos módulos bifurca em ramos curvos ligados ao arco',
 'Base contínua espessa acompanha o traçado dos módulos',
 'Corrimão claro superior tem largura e face lateral',
 'Balaustrada direita tem traçado curvo em planta e pelo menos três módulos legíveis',
 'Largura física dos módulos deve se manter no percurso curvo, com contagem ajustada',
 'Dois pilares altos marcam a abertura da escada externa',
 'Pilares de entrada têm base escalonada, painel verde e relevo ramificado dourado',
 'Capitéis e tampas projetados recebem vasos rasos',
 'Vasos dos pilares têm borda, bojo, terra e flores/folhas',
 'Extremos dos módulos entram nos pilares e nas bases, sem hastes suspensas']),
 ('PAT','Pátio e escadas',['S04-patio-access','D11-medallion'],[
 'Laje do pátio é elevada e tem frente curva, distinta da varanda coberta',
 'Três degraus largos curvos da entrada externa ligam chão ao pátio',
 'Degraus têm piso, espelho e extremidades conectadas à plataforma',
 'Entrada fica entre dois pilares e mantém passagem livre',
 'Piso tem lajes irregulares com juntas, sem grade ortogonal uniforme',
 'Medalhão verde circular fica deslocado para a metade posterior do pátio',
 'Desenho do medalhão tem rede de folhas e ramificação, não só raios',
 'Borda dourada do medalhão separa campo verde e pedra',
 'Faixa verde curva cruza o pátio e se integra ao medalhão',
 'Faixa verde tem filetes finos claros/dourados nas bordas',
 'Borda externa de lajes contorna a construção e termina em chão irregular',
 'Vegetação invade juntas e bases sem substituir a laje']),
 ('MOB','Móveis e objetos',['D05-porch-left','D06-porch-center','D01-main-window'],[
 'Banco sob a varanda tem assento, encosto ripado e apoios',
 'Encosto e braços do banco têm bordas esculpidas',
 'Cadeira à esquerda do banco tem quatro pernas, encosto e braços',
 'Assento e encosto da cadeira têm espessura e vazios distintos',
 'Vaso pequeno ocupa o assento/extremidade direita do banco',
 'Vaso à esquerda do banco fica no chão junto à parede',
 'Vasos ao lado da porta principal ocupam lados e alturas diferentes',
 'Vasos interiores junto à janela grande são parcialmente visíveis',
 'Prateleiras e objetos no interior grande têm profundidades distintas',
 'Objetos parcialmente oclusos só terão a porção legível especificada']),
 ('LUZ','Lanternas',['D05-porch-left','D06-porch-center','D07-entry'],[
 'Par de lanternas ladeia a porta principal na parede posterior',
 'Lanternas menores aparecem perto da porta esquerda e entre janelas da varanda',
 'Cada lanterna tem suporte curvo ligado à parede',
 'Caixilho escuro envolve campos claros quentes e tem tampa superior',
 'Base inferior afunila e tem pequeno remate',
 'Luz localizada quente é observada; ferramenta pode não oferecer emissão/local light']),
 ('VEG','Vegetação',['D14-garden','D05-porch-left','D07-entry','D10-rail-right'],[
 'Trepadeira alta sobe pelo pilarete à esquerda da janela grande',
 'Trepadeiras da parede da varanda têm ramos e grupos de folhas em alturas distintas',
 'Massa junto ao canto direito é vertical e mais alta que as flores baixas',
 'Arbustos cônicos estreitos ladeiam a face lateral direita',
 'Flores brancas baixas fazem grupos no perímetro e ao pé dos pilares',
 'Flores lilases em espigas ocupam grupos no lado direito',
 'Flores brancas também aparecem nos vasos dos pilares de entrada',
 'Vasos junto às portas têm folhas em direções distintas e terra recuada',
 'Folhagem acima das janelas laterais ocupa os campos sob os beirais',
 'Vegetação cresce atrás dos vãos da balaustrada e continua abaixo dela',
 'Pequenas folhas e musgo invadem juntas do piso externo',
 'Escalas de folhas, arbustos, vasos e flores precisam ser distintas',
 'Caule, folhas e flores têm relação de suporte, sem grupos inteiros flutuando',
 'Distribuição oculta é hipótese; preservar espécies aparentes sem nomes botânicos inventados']),
 ('MAT','Aparência e acabamento',['D13-eave','D14-garden','D17-tile-sample'],[
 'Pedra clara quente tem variação, juntas, manchas e envelhecimento',
 'Relevos claros dos pilares e arcos têm sombras próprias, não só pintura',
 'Telhas cinza-esverdeadas têm pequenas variações e bordas sobrepostas',
 'Telhas são numerosas, pequenas e com extremidades suavemente arredondadas',
 'Ornamentos dourados têm brilho moderado e desgaste',
 'Madeira das portas e móveis é castanha, com relevos e painéis',
 'Fundos verdes dos campos ornamentais são distintos das folhas',
 'Reflexo/transparência de vidro precisam capacidade real; declarar aproximação se ausente',
 'Musgo aparece sobretudo em juntas, bases e pontos de retenção',
 'Fundo neutro e luz da referência são condições de comparação, não geometria da casa'])]


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False, allow_nan=False)+'\n')


def analyze():
    ROOT.mkdir(exist_ok=True, parents=True)
    image = Image.open(SOURCE)
    records = []
    for key, box in REGIONS:
        image.crop(box).save(ROOT/(key+'.png'))
        records.append({'id':key,'box_xywh':[box[0],box[1],box[2]-box[0],box[3]-box[1]],'file':key+'.png'})
    result = {'source':str(SOURCE),'sha256':hashlib.sha256(SOURCE.read_bytes()).hexdigest(),
              'size':list(image.size),'coordinate_policy':'pixels, top-left origin; local 3D Y up; X right; +Z courtyard',
              'source_restriction':'Original image, current skill and public MCP. No previous analysis, recipe or geometry loaded.',
              'crops':records,'groups':[],'tasks':[],
              'state_policy':'Requirements snapshot: not_built/not_run records the analysis state. Current execution is tracked separately in e02-volume-progress.json.',
              'history':{'initial_tasks_delivered_before_geometry':188,
                  'addition_during_spatial_review':'VOL-COR-13: sem passagem, os corpos encostados não conectavam os ambientes. Hipótese interna acrescentada, não observação da foto.'},
              'revisits':[
                {'crops':['S01-main-body','D04-side-windows'],'finding':'Duas faces da alvenaria exigem um canto comum e profundidade; a lateral não é um painel decorativo.'},
                {'crops':['S02-wing-porch','D05-porch-left','D07-entry'],'finding':'Portas são posteriores à linha dos arcos; distinguir corpo fechado, varanda coberta e pátio aberto.'},
                {'crops':['S03-roof-structure','D08-gallery'],'finding':'Galeria fica entre duas coberturas diferentes; não fundir seus limites.'},
                {'crops':['D01-main-window','D15-ornament'],'finding':'Tronco claro é contínuo; rede fina dourada é uma segunda escala, não uma árvore genérica única.'},
                {'crops':['D03-dormer','S03-roof-structure'],'finding':'Um pano é amplamente legível e o restante precisa hipótese volumétrica; não tratar todos os lados como observados.'},
                {'crops':['D09-rail-left','D10-rail-right'],'finding':'Contar módulos por trecho e manter largura de percurso, sem copiar espaçamento em pixels para cada plano.'},
                {'crops':['D11-medallion','S04-patio-access'],'finding':'Há duas escadarias e o medalhão tem campos foliformes, além da rede radial.'},
                {'crops':['D12-finials','D13-eave','D18-porch-capitals'],'finding':'Bordas, remates e nervuras têm espessura e conexão; não são linhas flutuantes.'},
                {'crops':['D14-garden','D17-tile-sample'],'finding':'Aparência depende de famílias densas em escalas pequenas; preservar tarefas mesmo se orçamento impedir acabamento.'}],
              'hypotheses':[
                'Não existe escala métrica. Usar unidades de trabalho; medidas posteriores são hipóteses editáveis.',
                'Corpo alto será uma sala alongada em profundidade; ala baixa é adjacente pelo lado esquerdo.',
                'Traseira não observada será simples, fechada e coerente com pisos e coberturas.',
                'Configuração interna de cômodos não é observável; apenas continuidade próxima aos vãos.',
                'Assimetria aparente da cobertura alta admite várias explicações; testar silhueta sem deformar planos independentemente.']}
    for key, title, crops, descriptions in GROUPS:
        result['groups'].append({'id':key,'title':title,'crops':crops})
        for i, description in enumerate(descriptions,1):
            result['tasks'].append({'id':f'VOL-{key}-{i:02d}','group':key,'description':description,
                'evidence':crops,'observation':'hypothesis' if any(w in description.lower() for w in ['hipótes','infer','não observad']) else 'observed',
                'construction':'not_built','validation':'not_run','criterion':'Conferir a característica descrita no contexto e detalhe; coerência espacial também em vistas auxiliares.'})
    write_json(DATA,result)
    lines=['# E02 — nova lista após releitura e recortes','',
           'Fonte original '+result['sha256']+'. Lista inicial:188tarefas entregues antes da geometria. Revisão atual:'+str(len(result['tasks']))+' tarefas, incluindo passagem interna inferida durante a revisão espacial.',
           '', 'Este arquivo registra os requisitos; execução e pendências ficam em e02-volume-progress.json. A adição durante a construção está identificada no histórico do JSON.',
           '', 'Coordenadas locais: Y para cima, X ao longo da fachada, +Z para o pátio. Profundidade/traseira são hipóteses.', '']
    for group in result['groups']:
        lines += ['## '+group['title'],'','Evidência: '+', '.join(group['crops'])+'.','']
        lines += ['- **'+t['id']+'** — '+t['description']+'.' for t in result['tasks'] if t['group']==group['id']]
        lines += ['']
    lines += ['## Revisitações','']+['- '+r['finding'] for r in result['revisits']]
    lines += ['','## Limites e hipóteses','']+['- '+h for h in result['hypotheses']]
    (PROJECT/'docs/e02-volume-list.md').write_text('\n'.join(lines)+'\n')
    print(json.dumps({'tasks':len(result['tasks']),'crops':len(records),'list':str(PROJECT/'docs/e02-volume-list.md')}))


class MCP:
    def __init__(self):
        ROOT.mkdir(exist_ok=True, parents=True)
        self.log=(ROOT/'mcp.log').open('w')
        self.process=subprocess.Popen([BINARY,'--root',str(ROOT),'--mcp'],stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,stderr=self.log,text=True)
        self.seq=0
        self.revision=None
        self.rpc('initialize',{'protocolVersion':'2024-11-05','capabilities':{},'clientInfo':{'name':'e02-volume','version':'1'}})
        self.process.stdin.write(json.dumps({'jsonrpc':'2.0','method':'notifications/initialized'})+'\n')
        self.process.stdin.flush()

    def rpc(self, method, params):
        self.seq+=1
        self.process.stdin.write(json.dumps({'jsonrpc':'2.0','id':self.seq,'method':method,'params':params},allow_nan=False)+'\n')
        self.process.stdin.flush()
        for line in self.process.stdout:
            response=json.loads(line)
            if response.get('id')!=self.seq:
                continue
            if 'error' in response:
                raise RuntimeError(response['error'])
            return response['result']
        raise RuntimeError('MCP process ended; see mcp.log')

    def command(self, op, **arguments):
        payload={'command':{'op':op,**arguments}}
        if self.revision is not None:
            payload['expected_revision']=self.revision
        response=self.rpc('tools/call',{'name':'engine_command','arguments':payload})
        if response.get('isError'):
            raise RuntimeError(response)
        data=response.get('structuredContent')
        if data is None:
            data=json.loads(next(c['text'] for c in response['content'] if c['type']=='text'))
        if 'revision' in data and op not in ['capture_replay','run_replay','inspect_replay']:
            self.revision=data['revision']
        return data

    def close(self):
        self.process.stdin.close()
        self.process.wait(timeout=30)
        self.log.close()


def probe():
    client=MCP()
    try:
        schema=client.rpc('tools/list',{})
        write_json(ROOT/'schema.json',schema)
        result=client.command('inspect_meshes')
        write_json(ROOT/'limits.json',result['limits'])
        print(json.dumps(result['limits']))
    finally:
        client.close()


class Mesh:
    """Indexed world-coordinate surfaces, cooked once at explicit 1e-6 precision."""
    def __init__(self, key, color, groups=()):
        self.key, self.color = key, color
        self.p, self.t, self.indices = [], [], {}
        self.groups = list(groups)

    def vertex(self, p):
        value=tuple(float(np.float32(round(float(x),6))) for x in p)
        if value not in self.indices:
            self.indices[value]=len(self.p)
            self.p.append(list(value))
        return self.indices[value]

    def polygon(self, points, normal):
        points=[np.array(p,float) for p in points]
        if len(points)<3:
            return
        if len(points)==3:
            ids=[self.vertex(p) for p in points]
            n=np.cross(points[1]-points[0],points[2]-points[0])
            if np.dot(n,normal)<0:ids.reverse()
            if len(set(ids))==3 and np.linalg.norm(n)>1e-12:self.t.append(ids)
            return
        if len(points)==4:
            self.quad(*points,normal)
            return
        direction=np.array(normal,float);direction/=np.linalg.norm(direction)
        turns=[float(np.dot(np.cross(points[i]-points[i-1],points[(i+1)%len(points)]-points[i]),direction)) for i in range(len(points))]
        if ((all(v>1e-12 for v in turns) or all(v<-1e-12 for v in turns)) and
                max(abs(np.dot(p-points[0],direction)) for p in points)<1e-7):
            for i in range(1,len(points)-1):self.polygon([points[0],points[i],points[i+1]],normal)
            return
        # Convex bands may contain collinear edge splits. A center fan retains them.
        center=np.mean(points,axis=0)
        for a,b in zip(points,points[1:]+points[:1]):
            n=np.cross(a-center,b-center)
            if np.linalg.norm(n)<1e-12:
                continue
            triangle=[self.vertex(center),self.vertex(a),self.vertex(b)]
            if np.dot(n,normal)<0:
                triangle.reverse()
            if len(set(triangle))==3:
                self.t.append(triangle)

    def quad(self, a,b,c,d,normal):
        # Four-corner faces do not need a center vertex.
        p=[np.array(x,float) for x in (a,b,c,d)]
        for tri in ((0,1,2),(0,2,3)):
            n=np.cross(p[tri[1]]-p[tri[0]],p[tri[2]]-p[tri[0]])
            if np.linalg.norm(n)<1e-12:
                continue
            ids=[self.vertex(p[i]) for i in tri]
            if np.dot(n,normal)<0:
                ids.reverse()
            if len(set(ids))==3:
                self.t.append(ids)

    def box(self, lo, hi):
        x,y,z=lo;X,Y,Z=hi
        self.quad([x,y,z],[x,Y,z],[X,Y,z],[X,y,z],[0,0,-1])
        self.quad([x,y,Z],[X,y,Z],[X,Y,Z],[x,Y,Z],[0,0,1])
        self.quad([x,y,z],[x,y,Z],[x,Y,Z],[x,Y,z],[-1,0,0])
        self.quad([X,y,z],[X,Y,z],[X,Y,Z],[X,y,Z],[1,0,0])
        self.quad([x,y,z],[X,y,z],[X,y,Z],[x,y,Z],[0,-1,0])
        self.quad([x,Y,z],[x,Y,Z],[X,Y,Z],[X,Y,z],[0,1,0])

    def tube(self, points, radius, sides=4):
        points=np.array(points,float)
        closed=len(points)>3 and np.linalg.norm(points[0]-points[-1])<1e-10
        if closed:points=points[:-1]
        rings=[]
        for i,p in enumerate(points):
            tangent=points[(i+1)%len(points)]-points[i-1] if closed else points[min(i+1,len(points)-1)]-points[max(0,i-1)]
            tangent/=np.linalg.norm(tangent)
            other=np.array([0,0,1]) if abs(tangent[2])<.85 else np.array([0,1,0])
            a=np.cross(tangent,other);a/=np.linalg.norm(a)
            b=np.cross(tangent,a)
            rings.append([p+radius*(math.cos(j*math.tau/sides)*a+math.sin(j*math.tau/sides)*b) for j in range(sides)])
        for i in range(len(rings) if closed else len(rings)-1):
            nxt=(i+1)%len(rings)
            for j in range(sides):
                q=(j+1)%sides
                n=(rings[i][j]+rings[i][q])/2-points[i]
                self.quad(rings[i][j],rings[i][q],rings[nxt][q],rings[nxt][j],n)
        if not closed:
            self.polygon(rings[0],points[0]-points[1])
            self.polygon(rings[-1],points[-1]-points[-2])

    def prism(self, outline, y0, y1):
        lower=[[x,y0,z] for x,z in outline]
        upper=[[x,y1,z] for x,z in outline]
        self.polygon(lower,[0,-1,0]);self.polygon(upper,[0,1,0])
        for i,(x,z) in enumerate(outline):
            j=(i+1)%len(outline);X,Z=outline[j]
            self.quad(lower[i],lower[j],upper[j],upper[i],[Z-z,0,x-X])

    def geometry(self):
        return {'positions':self.p,'triangles':self.t}


class Opening:
    def __init__(self, key, face, center, width, sill, spring, apex, kind='window'):
        self.key,self.face,self.center,self.width,self.sill,self.spring,self.apex,self.kind=key,face,center,width,sill,spring,apex,kind
        self.left,self.right=center-width/2,center+width/2
        self.grid=([self.left,center,self.right] if kind=='passage' else
                   [self.left]+[self.left+i*width/8 for i in range(1,8)]+[self.right])

    def head(self, u):
        if self.kind=='passage':return self.apex
        t=abs((u-self.center)/(self.width/2))
        # Two meeting circular arcs, vertically scaled; a declared pointed-arch approximation.
        return self.spring+(self.apex-self.spring)*math.sqrt(max(0,(4-(1+t)**2)/3))


def panel(mesh, grid, bottom, top, openings, point, normal, extra_breaks=None):
    """Trapezoid strips with shared vertical edge splits; actual through holes."""
    def bands(u, probe):
        intervals=[(bottom(u),top(u))]
        for o in openings:
            if o.left < probe < o.right:
                lo,hi=o.sill,o.head(u)
                next_intervals=[]
                for a,b in intervals:
                    if hi<=a or lo>=b:
                        next_intervals.append((a,b))
                    else:
                        if lo>a: next_intervals.append((a,min(lo,b)))
                        if hi<b: next_intervals.append((max(hi,a),b))
                intervals=next_intervals
        return intervals
    strips=[]
    breaks={u:set() for u in grid}
    for a,b in zip(grid,grid[1:]):
        left,right=bands(a,(a+b)/2),bands(b,(a+b)/2)
        if len(left)!=len(right):
            raise ValueError('Opening crosses an exterior boundary: '+str((a,b,left,right)))
        strips.append((a,b,left,right))
        for lo,hi in left: breaks[a].update([lo,hi])
        for lo,hi in right: breaks[b].update([lo,hi])
    if extra_breaks:
        for u in grid:
            breaks[u].update(extra_breaks.get(u,()))
    for a,b,left,right in strips:
        for (lo,hi),(LO,HI) in zip(left,right):
            if min(hi-lo,HI-LO)<-1e-7:
                raise ValueError('Negative wall interval')
            points=[point(a,lo),point(b,LO)]
            points += [point(b,v) for v in sorted(breaks[b]) if LO<v<HI]
            points += [point(b,HI),point(a,hi)]
            points += [point(a,v) for v in sorted(breaks[a],reverse=True) if lo<v<hi]
            mesh.polygon(points,normal)
    return breaks


def hollow_body(mesh, bounds, ceiling, openings, thickness=.18):
    """Outer and inner closed skins joined around real windows/door openings."""
    xl,xh,zl,zh,y0=bounds
    xgrid=sorted(set([xl,xh,xl+thickness,xh-thickness]+[o.center for o in openings if o.face in ('front','back')]+
                     [v for o in openings if o.face in ('front','back') for v in o.grid]))
    zgrid=sorted(set([zl,zh,zl+thickness,zh-thickness]+[v for o in openings if o.face in ('left','right') for v in o.grid]))
    face_info={
        'front':(xgrid,lambda u,y,d:[u,y,zh-d],lambda u,d:ceiling(u,zh-d),[0,0,1]),
        'back':(xgrid,lambda u,y,d:[u,y,zl+d],lambda u,d:ceiling(u,zl+d),[0,0,-1]),
        'right':(zgrid,lambda u,y,d:[xh-d,y,u],lambda u,d:ceiling(xh-d,u),[1,0,0]),
        'left':(zgrid,lambda u,y,d:[xl+d,y,u],lambda u,d:ceiling(xl+d,u),[-1,0,0])}
    for face,(grid,point,top,normal) in face_info.items():
        holes=[o for o in openings if o.face==face]
        inner=grid[1:-1]
        if len(inner)<2:
            raise ValueError('Inner grid has no span')
        # Exterior contour cuts shared by the inner and outer hole loops.
        union={u:set() for u in grid}
        for o in holes:
            for u in grid:
                if o.left<=u<=o.right:
                    union[u].update([o.sill,o.head(u)])
        panel(mesh,grid,lambda u:y0,lambda u:top(u,0),holes,
              lambda u,y:point(u,y,0),normal,union)
        panel(mesh,inner,lambda u:y0+thickness,lambda u:top(u,thickness)-thickness,holes,
              lambda u,y:point(u,y,thickness),[-n for n in normal],union)
        for o in holes:
            us=[u for u in grid if o.left<=u<=o.right]
            loop=[(u,o.sill) for u in us]
            loop += [(o.right,v) for v in sorted(union[o.right]) if o.sill<v<o.spring]
            loop += [(u,o.head(u)) for u in reversed(us)]
            loop += [(o.left,v) for v in sorted(union[o.left],reverse=True) if o.sill<v<o.spring]
            for (u,y),(v,Y) in zip(loop,loop[1:]+loop[:1]):
                # Outward of wall material points into the hole; choose by UV edge.
                a,b=point(u,y,0),point(v,Y,0)
                c,d=point(v,Y,thickness),point(u,y,thickness)
                n=np.cross(np.array(b)-a,np.array(d)-a)
                if face in ('back','right'): n=-n
                mesh.quad(a,b,c,d,n)
    for inset,sign in [(0,1),(thickness,-1)]:
        xs=xgrid if inset==0 else xgrid[1:-1]
        zs=zgrid if inset==0 else zgrid[1:-1]
        for a,b in zip(xs,xs[1:]):
            for c,d in zip(zs,zs[1:]):
                pts=[[a,ceiling(a,c)-inset,c],[b,ceiling(b,c)-inset,c],
                     [b,ceiling(b,d)-inset,d],[a,ceiling(a,d)-inset,d]]
                mesh.quad(*pts,[0,sign,0])
        perimeter=[[x,y0+inset,zs[0]] for x in xs]
        perimeter += [[xs[-1],y0+inset,z] for z in zs[1:]]
        perimeter += [[x,y0+inset,zs[-1]] for x in reversed(xs[:-1])]
        perimeter += [[xs[0],y0+inset,z] for z in reversed(zs[1:-1])]
        mesh.polygon(perimeter,[0,-sign,0])
    return face_info


def patch_solid(mesh, sample, nu, nv, thickness=.16, uvalues=None):
    us=list(np.linspace(0,1,nu+1)) if uvalues is None else uvalues
    nu=len(us)-1
    grid=[[np.array(sample(u,j/nv),float) for j in range(nv+1)] for u in us]
    for i in range(nu):
        for j in range(nv):
            pts=[grid[i][j],grid[i+1][j],grid[i+1][j+1],grid[i][j+1]]
            mesh.quad(*pts,[0,1,0]);mesh.quad(*[p-[0,thickness,0] for p in pts],[0,-1,0])
    center=np.mean(np.array(grid),axis=(0,1))
    edges=[grid[0],grid[-1],[row[0] for row in grid],[row[-1] for row in grid]]
    for edge in edges:
        for a,b in zip(edge,edge[1:]):
            normal=(a+b)/2-center;normal[1]=0
            mesh.quad(a,b,b-[0,thickness,0],a-[0,thickness,0],normal)


def main_roof_height(x,z):
    ridge=2.15
    if x<ridge:
        y=5.0+3.0*((x-1.15)/(ridge-1.15))
    else:
        t=(x-ridge)/(6.35-ridge)
        y=4.1+3.9*(max(0,1-t)**1.45)+.20*t**10
    return y+.50*(abs((z+.7)/2.8)**8)


def wing_roof_height(x,z):
    t=abs((z+1.35)/1.95)
    return 4.90+.90*max(0,1-t)**1.3+.34*(abs((x+2.25)/4.25)**10)


def porch_front(x):
    return 2.12+.28*math.sin(math.pi*(x+6.4)/7.9)


def porch_sample(u,v):
    x=-6.4+7.9*u
    z=.28+(porch_front(x)-.28)*v
    y=3.84-.50*v+.14*math.sin(math.pi*u)+1.25*((1-u)**10)*((1-v)**3)
    return [x,y,z]


def make_structure():
    main=Mesh('volume-main-body',[.68,.62,.50],['COR','ORG'])
    wing=Mesh('volume-wing-body',[.68,.62,.50],['COR','ORG','GAL'])
    roof=Mesh('volume-roof',[.23,.30,.28],['RM','RA','RV'])
    floor=Mesh('volume-groundworks',[.66,.60,.49],['PAT'])
    trim=Mesh('volume-ivory',[.83,.76,.62],['COL','RM','RA','RV','BAL'])
    openings_main=[Opening('great','front',4.25,2.35,1.15,3.20,4.85),
                   Opening('high-left','front',1.98,.56,1.4,5.8,6.60),
                   Opening('upper','front',3.25,1.25,5.02,5.18,6.10),
                   Opening('side-front','right',.33,1.02,1.35,2.95,3.75),
                   Opening('side-back','right',-1.50,1.02,1.35,2.95,3.75),
                   Opening('passage-main','left',-1.50,1.02,.73,2.60,2.60,'passage')]
    openings_wing=[Opening('door-main','front',.55,1.22,.85,2.40,3.22,'door'),
                   Opening('door-left','front',-5.25,.95,.75,2.10,2.92,'door')]
    openings_wing += [Opening('porch-window-'+str(i),'front',x,.62,1.26,2.32,2.94) for i,x in enumerate([-3.96,-2.84,-1.70])]
    openings_wing += [Opening('gallery-'+str(i),'front',x,.70,3.60,4.20,4.54) for i,x in enumerate([-3.65,-2.65,-1.65,-.65,.35])]
    openings_wing += [Opening('passage-wing','right',-1.50,1.02,.73,2.60,2.60,'passage')]
    # Room bodies, not facade proxies. Skins and jambs share vertices in 3D.
    main_faces=hollow_body(main,[1.50,5.90,-3.0,1.14,.55],lambda x,z:main_roof_height(x,z)-.18,openings_main)
    wing_faces=hollow_body(wing,[-6.10,1.50,-3.0,.28,.55],lambda x,z:wing_roof_height(x,z)-.18,openings_wing)
    roof_us=sorted(set([i/14 for i in range(15)]+[(2.15-1.15)/5.2]))
    patch_solid(roof,lambda u,v:[1.15+5.20*u,main_roof_height(1.15+5.20*u,-3.35+5.18*v),-3.35+5.18*v],14,10,uvalues=roof_us)
    patch_solid(roof,lambda u,v:[-6.45+8.05*u,wing_roof_height(-6.45+8.05*u,-3.4+4.02*v),-3.4+4.02*v],12,8)
    patch_solid(roof,porch_sample,16,5)
    outline=[[-6.6,-.1],[5.9,-.1]]+[[ -.45+6.9*math.cos(t),1.0+4.65*math.sin(t)] for t in np.linspace(0,math.pi,33)]
    floor.prism(outline,.12,.55)
    # Steps occupy the entrance sector and have full risers/bottoms.
    for i in range(3):
        scale=1.18-i*.06
        def step(u,v,scale=scale,i=i):
            angle=1.05+.67*u;r=.96+(scale-.96)*v
            return [-.45+6.9*r*math.cos(angle),.19+i*.12,1+4.65*r*math.sin(angle)]
        patch_solid(floor,step,14,1,.17+i*.12)
    for i in range(3):
        rx=.95-i*.10;rz=.55-i*.07
        outline=[[.55+rx*math.cos(t),.28+rz*math.sin(t)] for t in np.linspace(0,math.pi,17)]
        floor.prism(outline,.55,.65+i*.10)
    # Native plant columns will be instantiated at these grounded feet.
    supports=[]
    for x in [-5.90,-4.18,-2.46,-.74]:
        z=porch_front(x)-.12
        u=(x+6.4)/7.9;v=(z-.28)/(porch_front(x)-.28)
        supports.append({'x':x,'z':z,'bottom':.55,'top':porch_sample(u,v)[1]-.16})
    # Roof edge frames and beam are finite structural members, independent of camera.
    for coords in [lambda t:[1.15+5.2*t,main_roof_height(1.15+5.2*t,1.83),1.83],
                   lambda t:[1.15+5.2*t,main_roof_height(1.15+5.2*t,-3.35),-3.35],
                   lambda t:[6.35,main_roof_height(6.35,-3.35+5.18*t),-3.35+5.18*t],
                   lambda t:[-6.45+8.05*t,wing_roof_height(-6.45+8.05*t,.62),.62],
                   lambda t:porch_sample(t,1),lambda t:porch_sample(0,t),lambda t:porch_sample(1,t)]:
        trim.tube([coords(t) for t in np.linspace(0,1,20)],.085,4)
    # Façade arch supports between columns; terminals are on grounded supports/wall.
    ends=[(s['x'],s['z']) for s in supports]+[(1.5,1.14)]
    for (x,z),(X,Z) in zip(ends,ends[1:]):
        path=[]
        for t in np.linspace(0,1,13):
            path.append([x+(X-x)*t,2.30+.90*math.sqrt(max(0,(4-(1+abs(2*t-1))**2)/3)),z+(Z-z)*t])
        trim.tube(path,.065,5)
    trim.tube([[-5.9,2.3,2.0-1.72*t] for t in [0]]+
              [[-5.9,2.3+.90*(1-abs(2*t-1)**.7),2.0-1.72*t] for t in np.linspace(.08,1,12)],.065,5)
    meshes={m.key:m for m in [main,wing,roof,floor,trim]}
    return meshes,{'main_openings':openings_main,'wing_openings':openings_wing,
                   'main_faces':main_faces,'wing_faces':wing_faces,'supports':supports}


def preflight(meshes):
    from reconstruction_structure import inspect_surface
    reports={key:inspect_surface(mesh.geometry()) for key,mesh in meshes.items() if key.endswith('-body')}
    write_json(ROOT/'solid-preflight.json',reports)
    if not reports or not all(r['solid_preflight_passed'] for r in reports.values()):
        print(json.dumps({k:{'defects':v['defects'],'boundary':v['boundary_edges'],'orientation':v['inconsistent_edges'],
            'components':len(v['components'])} for k,v in reports.items()}))
        raise ValueError('Actual room shell failed solid preflight')
    counts={'assets':len(meshes)+1,'vertices':sum(len(m.p) for m in meshes.values())+128,
            'triangles':sum(len(m.t) for m in meshes.values())+128,'instances':len(meshes)+4}
    write_json(ROOT/'preflight-counts.json',counts)
    print(json.dumps(counts))
    return reports


def geometry_probe():
    meshes,layout=make_structure()
    preflight(meshes)
    write_json(ROOT/'structure-geometry.json',{k:m.geometry() for k,m in meshes.items()})
    print('closed room bodies ready')


def curve(points, steps=9):
    p=np.array(points,float)
    if len(p)==3:
        return [(1-t)**2*p[0]+2*(1-t)*t*p[1]+t*t*p[2] for t in np.linspace(0,1,steps)]
    return [(1-t)**3*p[0]+3*(1-t)**2*t*p[1]+3*(1-t)*t*t*p[2]+t**3*p[3] for t in np.linspace(0,1,steps)]


def opening_loop(o):
    return [(o.left,o.sill),(o.right,o.sill)]+[(u,o.head(u)) for u in reversed(o.grid)]


def surface_slab(mesh, outline, host, depth=.025):
    front=[np.array(host(u,y),float) for u,y in outline]
    n=np.cross(front[1]-front[0],front[2]-front[1]);n/=np.linalg.norm(n)
    back=[p-n*depth for p in front]
    mesh.polygon(front,n);mesh.polygon(back,-n)
    center=np.mean(front,axis=0)
    for a,b,c,d in zip(front,front[1:]+front[:1],back[1:]+back[:1],back):
        normal=(a+b)/2-center
        mesh.quad(a,b,c,d,normal)


def motif(mesh, o, host, radius=.016, levels=2, fine=False):
    c,h,w=o.center,o.apex-o.sill,o.width/2
    stem=[host(c,o.sill+.025),host(c,o.sill+.48*h),host(c,o.apex-.07)]
    mesh.tube(curve(stem,8),radius,3)
    for level in range(levels):
        start=o.sill+h*(.24+.22*level)
        reach=w*(.86-.21*level)
        for sign in (-1,1):
            endx=c+sign*reach
            endy=min(o.head(endx)-.045,o.sill+h*(.74+.1*level))
            path=curve([host(c,start),host(c+sign*reach*.15,endy-.22*h),host(endx,endy)],7)
            mesh.tube(path,radius*.8,3)
            if fine:
                for f in [-1,1]:
                    targetx=endx+f*reach*.12
                    targety=min(o.head(targetx)-.04,endy+.13*h)
                    mesh.tube([path[-3],host(targetx,targety)],radius*.44,3)


def pot(mesh, x,y,z, scale=1):
    profile=[(.11,0),(.15,.04),(.22,.29),(.24,.34),(.20,.34),(.18,.27),(.12,.08),(0,.08),(0,0)]
    for (r,a),(R,b) in zip(profile,profile[1:]+profile[:1]):
        for j in range(8):
            t,T=j*math.tau/8,(j+1)*math.tau/8
            pts=[[x+scale*r*math.cos(t),y+scale*a,z+scale*r*math.sin(t)],
                 [x+scale*r*math.cos(T),y+scale*a,z+scale*r*math.sin(T)],
                 [x+scale*R*math.cos(T),y+scale*b,z+scale*R*math.sin(T)],
                 [x+scale*R*math.cos(t),y+scale*b,z+scale*R*math.sin(t)]]
            radial=np.array([math.cos((t+T)/2),0,math.sin((t+T)/2)])
            normal=radial if r+R>.25 else np.array([0,-1,0])
            mesh.quad(*pts,normal)
    return [x,y+scale*.30,z]


def add_details(meshes, layout):
    ivory=meshes['volume-ivory'];stone=meshes['volume-groundworks']
    gold=Mesh('volume-gold',[.57,.39,.15],['JAN','POR','GAL','AF','PAT','BAL','RM','RA','RV'])
    wood=Mesh('volume-wood',[.25,.14,.065],['POR','MOB','LUZ','RM'])
    inlay=Mesh('volume-inlay',[.10,.25,.19],['JAN','JL','GAL','BAL','PAT'])
    tiles=Mesh('volume-tiles',[.31,.37,.32],['RM','RA','RV','MAT'])
    leaf=Mesh('volume-leaf',[.12,.25,.055],['VEG'])
    white=Mesh('volume-white-flowers',[.90,.86,.64],['VEG'])
    purple=Mesh('volume-purple-flowers',[.44,.29,.54],['VEG'])
    light=Mesh('volume-lantern-fields',[.98,.74,.31],['LUZ'])
    meshes.update({m.key:m for m in [gold,wood,inlay,tiles,leaf,white,purple,light]})
    plants=[]
    # Window geometry is hosted on named faces of the actual room bodies.
    for openings,faces in [(layout['main_openings'],layout['main_faces']),
                           (layout['wing_openings'],layout['wing_faces'])]:
        for o in openings:
            if o.kind=='passage':continue
            point=faces[o.face][1]
            host=lambda u,y,p=point:p(u,y,-.035)
            outline=opening_loop(o)
            ivory.tube([host(u,y) for u,y in outline+[outline[0]]],.058 if o.key=='great' else .038,4)
            inner_host=lambda u,y,p=point:p(u,y,-.058)
            if o.kind=='door':
                surface_slab(wood,outline,lambda u,y,p=point:p(u,y,.055),.04)
                motif(gold,o,inner_host,.014,3,True)
                gold.tube([host(o.center,o.sill+.15),host(o.center,o.apex-.07)],.012,3)
                for sign in (-1,1):
                    u=o.center+sign*.11
                    gold.tube([host(u,o.sill+.53),host(u,o.sill+.66)],.012,4)
                continue
            if o.key=='great':
                motif(ivory,o,inner_host,.055,3,False)
                motif(gold,o,lambda u,y,p=point:p(u,y,.01),.012,3,True)
                # Shelves and pots are inside the room, with separate depths.
                for y in [1.35,2.1,2.8]:
                    wood.box([3.15,y,-.65],[5.15,y+.055,-.30])
                for x,z in [(3.35,.70),(5.08,.70)]:
                    plants.append(pot(stone,x,.73,z,.85))
            else:
                surface_slab(inlay,outline,lambda u,y,p=point:p(u,y,.12),.012)
                motif(gold,o,inner_host,.012,1 if o.key.startswith('gallery') else 2,False)
            # Peitoril is a real projection from the host wall.
            sillhost=lambda u,y,p=point:p(u,y,-.09)
            ivory.tube([sillhost(o.left-.07,o.sill-.025),sillhost(o.right+.07,o.sill-.025)],.045,4)
    # Piers that meet the common wall corner rather than independent facade planes.
    for x,z in [(1.53,1.18),(5.87,1.18),(5.94,-.58),(5.94,-2.83)]:
        height=main_roof_height(x,z)-.30
        ivory.box([x-.095,.55,z-.09],[x+.095,height,z+.09])
        ivory.box([x-.20,.55,z-.18],[x+.20,.78,z+.18])
        ivory.box([x-.16,.78,z-.14],[x+.16,.86,z+.14])
    # Small dormer has a separate, inspected hollow body and a solid gable roof.
    dormer=Mesh('volume-dormer-body',[.70,.63,.50],['AF'])
    dormer_top=lambda x,z:5.46+.84*(1-abs((z+.85)/.65))
    window=Opening('dormer','right',-.85,.84,4.92,5.39,5.94)
    faces=hollow_body(dormer,[4.64,5.93,-1.50,-.20,4.60],dormer_top,[window],.12)
    meshes[dormer.key]=dormer
    host=lambda u,y:faces['right'][1](u,y,-.03)
    points=opening_loop(window)
    ivory.tube([host(u,y) for u,y in points+[points[0]]],.045,4)
    surface_slab(inlay,points,lambda u,y:faces['right'][1](u,y,.06),.015)
    motif(gold,window,host,.013,2,False)
    patch_solid(meshes['volume-roof'],lambda u,v:[4.55+1.49*u,5.46+.90*(1-abs(2*v-1)),-1.60+1.5*v],4,6,.10)
    # Staggered tile fields: geometry follows roof samples in object coordinates.
    def main_tile(u,v):
        split=2/14
        x=1.15+u/split if u<=split else 2.15+(u-split)/(1-split)*4.2
        return [x,main_roof_height(x,-3.35+5.18*v)+.023,-3.35+5.18*v]
    surfaces=[(main_tile,14,14,'v'),
              (lambda u,v:[-6.45+8.05*u,wing_roof_height(-6.45+8.05*u,-3.4+4.02*v)+.023,-3.4+4.02*v],16,8,'u'),
              (lambda u,v:np.array(porch_sample(u,v))+[0,.023,0],18,6,'u')]
    for sample,nu,nv,stagger in surfaces:
        for j in range(nv):
            for i in range(nu):
                a=max(0,(i+.02-(j%2)*.5*(stagger=='u'))/nu);b=min(1,(i+.98-(j%2)*.5*(stagger=='u'))/nu)
                c=max(0,(j+.015-(i%2)*.5*(stagger=='v'))/nv);d=min(1,(j+.985-(i%2)*.5*(stagger=='v'))/nv)
                if b>a:
                    pts=[sample(a,c),sample(b,c),sample(b,d),sample(a,d)]
                    tiles.quad(*pts,[0,1,0])
    # Five finials, leaf outlines and a central lance, each attached to a roof.
    finials=[[2.15,main_roof_height(2.15,1.83),1.83],[2.15,main_roof_height(2.15,-3.35),-3.35],
             [-6.45,wing_roof_height(-6.45,-1.35),-1.35],porch_sample(0,0),[5.96,6.36,-.85]]
    for x,y,z in finials:
        gold.tube([[x,y-.03,z],[x,y+.58,z]],.024,4)
        leafline=[[x,y+.06,z],[x-.13,y+.25,z],[x,y+.51,z],[x+.13,y+.25,z],[x,y+.06,z]]
        gold.tube(leafline,.019,3)
    # Roof brackets and shaded under-eave ribs have endpoints on wall and roof.
    for z in np.linspace(-2.65,.95,5):
        wood.tube([[5.91,3.70,z],[6.28,main_roof_height(6.28,z)-.16,z]],.045,4)
    for z in np.linspace(.60,1.55,5):
        wood.tube([[2.0,7.18,z],[1.48,main_roof_height(1.48,z)-.17,z]],.04,4)
    # Balustrades are laid out by approximately equal arc distance, not pixels.
    def border(theta):
        return np.array([-.45+6.9*math.cos(theta),1+4.65*math.sin(theta)])
    def arc_points(a,b,n):
        samples=[border(t) for t in np.linspace(a,b,200)]
        lengths=[0]
        for p,q in zip(samples,samples[1:]):lengths.append(lengths[-1]+np.linalg.norm(q-p))
        return [np.array([np.interp(s,lengths,[p[k] for p in samples]) for k in range(2)]) for s in np.linspace(0,lengths[-1],n+1)]
    for a,b,n in [(2.85,1.72,5),(1.05,.40,3)]:
        path=arc_points(a,b,n)
        for y,r in [(.63,.11),(1.58,.09)]:
            ivory.tube([[p[0],y,p[1]] for p in arc_points(a,b,18)],r,5)
        for p,q in zip(path,path[1:]):
            mid=(p+q)/2;delta=q-p;length=np.linalg.norm(delta);axis=delta/length
            host=lambda u,y,mid=mid,axis=axis:[mid[0]+u*axis[0],y,mid[1]+u*axis[1]]
            o=Opening('rail','rail',0,length*.92,.72,1.18,1.55)
            pts=opening_loop(o)
            ivory.tube([host(u,y) for u,y in pts+[pts[0]]],.032,4)
            motif(ivory,o,host,.034,1,False)
    for t in [1.72,1.05]:
        x,z=border(t)
        for w,lo,hi in [(.34,.55,.68),(.29,.68,.78),(.24,.78,1.85),(.30,1.85,1.94),(.35,1.94,2.01)]:
            stone.box([x-w,lo,z-w],[x+w,hi,z+w])
        panel_outline=[(-.15,.92),(.15,.92),(.15,1.64),(0,1.81),(-.15,1.64)]
        panelhost=lambda u,y,x=x,z=z:[x+u,y,z+.245]
        surface_slab(inlay,panel_outline,panelhost,.012)
        motif(gold,Opening('post','front',0,.28,.94,1.59,1.80),lambda u,y,x=x,z=z:[x+u,y,z+.27],.014,2)
        plants.append(pot(stone,x,2.01,z,1.15))
    # Paving: bounded vector grooves, distinct from changing the slab's outline.
    # Medallion and ribbon are flat inset-like geometry with leaf-shaped networks.
    cx,cz=-1.10,3.35
    inlay.prism([[cx+.80*math.cos(t),cz+.80*math.sin(t)] for t in np.linspace(0,math.tau,33)[:-1]],.551,.557)
    gold.tube([[cx+.81*math.cos(t),.565,cz+.81*math.sin(t)] for t in np.linspace(0,math.tau,33)],.012,3)
    for i in range(8):
        a=i*math.tau/8
        points=[[cx,.565,cz],[cx+.43*math.cos(a-.26),.565,cz+.43*math.sin(a-.26)],
                [cx+.72*math.cos(a),.565,cz+.72*math.sin(a)],
                [cx+.43*math.cos(a+.26),.565,cz+.43*math.sin(a+.26)],[cx,.565,cz]]
        gold.tube(points,.010,3)
    ribbon=curve([[-2.8,1.1],[-2.5,2.5],[-.10,3.5],[-.1,4.65]],28)
    left=[];right=[]
    for i,p in enumerate(ribbon):
        tangent=ribbon[min(i+1,len(ribbon)-1)]-ribbon[max(0,i-1)]
        normal=np.array([-tangent[1],tangent[0]])/np.linalg.norm(tangent)
        left.append(p+normal*.09);right.append(p-normal*.09)
    # Piecewise ribbon quads preserve the curve without a concave polygon fan.
    for i in range(len(ribbon)-1):
        inlay.quad([left[i][0],.558,left[i][1]],[right[i][0],.558,right[i][1]],
                   [right[i+1][0],.558,right[i+1][1]],[left[i+1][0],.558,left[i+1][1]],[0,1,0])
    for edge in [left,right]:gold.tube([[p[0],.566,p[1]] for p in edge],.009,3)
    # Bench and armchair are full depth furniture below the canopy.
    wood.box([-4.10,.91,.63],[-2.36,1.04,1.12])
    for x in [-3.99,-2.48]:
        for z in [.72,1.01]:wood.box([x-.045,.55,z-.045],[x+.045,.94,z+.045])
        wood.box([x-.04,1.0,.68],[x+.04,1.50,.77])
        wood.box([x-.04,1.18,.74],[x+.04,1.28,1.10])
    for x in np.linspace(-3.93,-2.55,9):wood.box([x-.025,1.05,.68],[x+.025,1.45,.74])
    for z in [.71,1.18]:
        for x in [-4.88,-4.40]:wood.box([x-.035,.55,z-.035],[x+.035,1.02,z+.035])
    wood.box([-4.95,.94,.65],[-4.34,1.04,1.23])
    for x in [-4.88,-4.40]:
        wood.box([x-.03,1.0,.70],[x+.03,1.65,.77])
        wood.box([x-.03,1.22,.70],[x+.03,1.29,1.20])
    wood.box([-4.90,1.57,.70],[-4.38,1.65,.77])
    for x,y,z in [(-5.70,.55,.75),(-4.25,.55,.62),(-2.48,1.04,.88),(-.27,.55,.55),(1.20,.55,.60)]:
        plants.append(pot(stone,x,y,z,.85))
    # Lanterns have wall brackets, a frame and warm fields; no invented local light API.
    for x in [-5.85,-4.62,-3.28,-.23,1.26]:
        y=2.25;z=.40
        wood.tube([[x,y+.31,.29],[x,y+.35,.60],[x,y+.18,.66]],.022,3)
        light.box([x-.065,y-.12,.60],[x+.065,y+.15,.72])
        for dx in [-.08,.08]:
            for zz in [.59,.73]:gold.tube([[x+dx,y-.15,zz],[x+dx,y+.18,zz]],.012,3)
        gold.box([x-.10,y+.15,.57],[x+.10,y+.20,.75])
    # Leaves are explicitly oriented in 3D; the stem anchors each cluster to floor/pot/wall.
    rng=random.Random(271026)
    def foliage(base, height=.50, count=12, spread=.25, stem=True):
        base=np.array(base,float)
        tip=base+[0,height,0]
        if stem:leaf.tube([base,tip],.012,3)
        for i in range(count):
            t=.16+.80*(i/max(1,count-1));a=i*2.399+rng.uniform(-.2,.2)
            root=base+[0,t*height,0]
            reach=spread*(1-.50*t)
            end=root+np.array([math.cos(a)*reach,.12*height,math.sin(a)*reach])
            side=np.array([-math.sin(a),0,math.cos(a)])*reach*.32
            mid=(root+end)/2+[0,.035,0]
            leaf.quad(root,mid+side,end,mid-side,[math.cos(a),1,math.sin(a)])
    for base in plants:foliage(base,.42,9,.26)
    for i in range(15):
        a=.12+i*math.pi/14
        p=np.array([-.45+7.15*math.cos(a),.04,1+4.85*math.sin(a)])
        foliage(p,.28+rng.random()*.30,8,.30)
        for j in range(2):
            x=p[0]+rng.uniform(-.18,.18);z=p[2]+rng.uniform(-.18,.18);y=.34+rng.random()*.20
            flower=white if i%3 else purple
            leaf.tube([p,[x,y,z]],.006,3)
            flower.polygon([[x+.06*math.cos(k*math.tau/5),y,z+.06*math.sin(k*math.tau/5)] for k in range(5)],[0,1,0])
    for x,z,h in [(6.13,-2.30,1.55),(6.25,-.8,1.20),(5.97,1.40,.90)]:foliage([x,.05,z],h,16,.30)
    for x,z,h in [(1.70,1.20,5.8),(-2.16,.34,2.5),(-4.47,.34,2.3)]:
        path=[[x+.07*math.sin(i),.55+h*i/12,z+.045] for i in range(13)]
        leaf.tube(path,.014,3)
        for i in range(12):foliage(path[i],.16,2,.16,False)


CAMERA={'position':[14,9.0,19],'target':[.2,3.4,1.0],'fov_degrees':32}
VIEWS={
    'reference':CAMERA,
    'front':{'position':[-.2,5.5,24],'target':[-.2,3.0,.8],'fov_degrees':40},
    'right':{'position':[23,8,0],'target':[-.2,3.0,.8],'fov_degrees':40},
    'back':{'position':[-11,10,-20],'target':[-.2,3.0,.8],'fov_degrees':40},
    'left':{'position':[-23,8,1],'target':[-.2,3.0,.8],'fov_degrees':40},
    'top':{'position':[-.2,26,3.5],'target':[-.2,0,1.0],'fov_degrees':40}}


def material_for(key):
    material={'pattern':{'frequency':[3.1,4.0,3.1],'low':[.86,.86,.81],'high':[1.05,1.04,1.01],
                          'bump':.016,'warp':.22,'octaves':3,'seed':117,'contrast':.5}}
    if key in ['volume-main-body','volume-wing-body']:
        segments=[{'start':[-8,i*.43],'end':[8,i*.43],'widths':[.011,.011]} for i in range(17)]
        segments += [{'start':[-5.5+i*.86,.43*(i%5)],'end':[-5.5+i*.86,.43*(i%5+1)],'widths':[.008,.008]} for i in range(12)]
        material['detail']={'plane':'xy','segments':segments,'tint':[.38,.34,.26],'opacity':.55,'bump':.018}
    if key=='volume-groundworks':
        segments=[]
        for row in range(5):
            zs=[.95+row*.78+.12*math.sin(row+i*1.8) for i in range(5)]
            for i in range(4):segments.append({'start':[-6+i*3,zs[i]],'end':[-3+i*3,zs[i+1]],'widths':[.014,.014]})
        for row in range(3):
            for col in range(4):
                x=-4.6+col*2.9+.25*(row%2)
                segments.append({'start':[x,1+row*.8],'end':[x+.18*math.sin(col),1.8+row*.8],'widths':[.012,.012]})
        material['detail']={'plane':'xz','segments':segments,'tint':[.34,.31,.25],'opacity':.57,'bump':.016}
    if key in ['volume-roof','volume-tiles','volume-gold']:
        material['reflectance']={'f0':.045,'roughness':.7}
    return material


def publish(stage):
    meshes,layout=make_structure()
    preflight(meshes)
    if stage=='detail':
        add_details(meshes,layout)
        preflight(meshes)
    client=MCP()
    try:
        limits=client.command('inspect_meshes')['limits']
        counts={'assets':len(meshes)+1,'instances':len(meshes)+len(layout['supports']),
                'vertices':sum(len(m.p) for m in meshes.values())+128,
                'triangles':sum(len(m.t) for m in meshes.values())+128}
        assert counts['vertices']<=limits['total_vertices'],counts
        assert counts['triangles']<=limits['total_triangles'],counts
        assert counts['assets']<=limits['assets'] and counts['instances']<=limits['instances'],counts
        assert all(len(m.p)<=limits['asset_vertices'] and len(m.t)<=limits['asset_triangles'] for m in meshes.values())
        state=client.command('inspect')
        edits=[{'op':'remove','id':entity['id']} for entity in state['scene']['entities'] if entity['id']!='player']
        edits.append({'op':'upsert','entity':{'id':'player','name':'outside model','visible':False,'body':'player',
            'transform':{'position':[35,1,35],'size':[.7,1.6,.7]},'color':[.5,.5,.5]}})
        edits.append({'op':'upsert','entity':{'id':'volume-studio-ground','name':'ground',
            'transform':{'position':[0,-.12,0],'size':[100,.20,100]},'color':[.64,.61,.55]}})
        client.command('edit_scene',edits=edits)
        for key,mesh in meshes.items():
            client.command('edit_mesh',edit={'kind':'put_asset','asset':{'id':key,'geometry':mesh.geometry(),
                'shading':'flat' if key.endswith('-body') else 'smooth','material':material_for(key)}})
            client.command('edit_mesh',edit={'kind':'put_instance','instance':{'id':key+'-instance','asset':key,
                'position':[0,0,0],'rotation':[0,0,0,1],'scale':1,'color':mesh.color}})
        definition={'id':'volume-column','profile':{'id':'column-profile','width':.40,'height':2.47},'depth':.40,'segments':[1,1,1],
            'sections':{'id':'column-sections','cross_section':'rectangle','stations':[
                {'id':'section-'+str(i),'at':at,'width':w,'depth':w} for i,(at,w) in enumerate(
                    [(0,1),(.045,1),(.065,.84),(.10,.84),(.12,.94),(.15,.55),(.79,.55),(.84,.64),(.88,.78),(.93,1),(1,1)])]}}
        edit={'kind':'put','definition':definition,'detail_policy':'physical','animation_translation_policy':'physical'}
        write_json(ROOT/'column-preview.json',client.command('preview_plant',edit=edit))
        client.command('edit_plant',edit=edit)
        for i,s in enumerate(layout['supports']):
            client.command('edit_mesh',edit={'kind':'put_instance','instance':{'id':'volume-column-'+str(i),'asset':'volume-column',
                'position':[s['x'],(s['bottom']+s['top'])/2,s['z']],'rotation':[0,0,0,1],
                'scale':(s['top']-s['bottom'])/2.47,'color':[.80,.73,.60]}})
        client.command('set_light',light={'direction':[-.65,-1,-.65],'ambient':.38,
            'shadow':{'center':[0,3,0],'extent':16,'resolution':2048,'bias':.001}})
        native={}
        for key in [k for k in meshes if k.endswith('-body')]+['volume-column']:
            native[key]=client.command('inspect_mesh',asset=key)
        write_json(ROOT/'native-geometry.json',native)
        for name,camera in VIEWS.items():
            client.command('set_camera',camera=camera)
            capture=client.command('capture',path=name+'.png',width=1402,height=1122)
            write_json(ROOT/(name+'-capture.json'),capture)
        client.command('set_camera',camera=CAMERA)
        client.command('save_scene',path='house.scene.json')
        scene=json.loads((ROOT/'house.scene.json').read_text())
        (PROJECT/'examples/house-e02-volume.scene.json').write_bytes((ROOT/'house.scene.json').read_bytes())
        write_json(ROOT/'native-counts.json',client.command('inspect_meshes'))
        write_json(ROOT/'bindings.json',{key:mesh.groups for key,mesh in meshes.items()})
        print(json.dumps({'stage':stage,'native_keys':{k:list(v) for k,v in native.items()},'counts':counts}))
    finally:
        client.close()


def segment_hits(geometry, start, end):
    """Vectorized segment/triangle diagnostic on the actual exported wall geometry."""
    p=np.array(geometry['positions'],float);t=np.array(geometry['triangles'],int)
    a,b,c=(p[t[:,i]] for i in range(3))
    direction=np.array(end)-start;e1=b-a;e2=c-a
    h=np.cross(np.broadcast_to(direction,e2.shape),e2);det=np.sum(e1*h,axis=1)
    valid=np.abs(det)>1e-10
    inverse=np.zeros(len(det));inverse[valid]=1/det[valid]
    s=np.array(start)-a;u=np.sum(s*h,axis=1)*inverse
    q=np.cross(s,e1);v=np.sum(q*direction,axis=1)*inverse
    along=np.sum(e2*q,axis=1)*inverse
    return int(np.count_nonzero(valid & (u>=-1e-7) & (v>=-1e-7) & (u+v<=1+1e-7) & (along>=0) & (along<=1)))


def verify():
    from reconstruction_structure import inspect_surface
    source_hash=hashlib.sha256(SOURCE.read_bytes()).hexdigest()
    scene=json.loads((ROOT/'house.scene.json').read_text())
    assets={a['id']:a for a in scene['meshes']['assets']}
    checks={};structure={};topology={}
    for key,asset in assets.items():
        if key.endswith('-body') or key=='volume-column':
            structure[key]=inspect_surface(asset['geometry'])
            checks[key+'-closed']=structure[key]['solid_preflight_passed']
    roof=inspect_surface(assets['volume-roof']['geometry'])
    checks['roof-surfaces-closed']=roof['boundary_edges']==0 and roof['non_manifold_edges']==0 and roof['inconsistent_edges']==0
    # Openings are tested against walls only; door leaves and glazing have separate roles.
    _,layout=make_structure()
    aperture_checks={}
    for key,openings,faces in [('volume-main-body',layout['main_openings'],layout['main_faces']),
                              ('volume-wing-body',layout['wing_openings'],layout['wing_faces'])]:
        for o in openings:
            point=faces[o.face][1];y=(o.sill+o.apex)/2
            hits=segment_hits(assets[key]['geometry'],point(o.center,y,-.06),point(o.center,y,.26))
            aperture_checks[o.key]={'wall_triangle_intersections':hits,'passed':hits==0}
    checks['17-body-apertures-through-wall']=all(v['passed'] for v in aperture_checks.values()) and len(aperture_checks)==17
    checks['internal-passage-connects-rooms']=all(segment_hits(assets[key]['geometry'],[1.23,1.60,-1.50],[1.77,1.60,-1.50])==0
        for key in ['volume-main-body','volume-wing-body'])
    original=(ROOT/'reference.png').read_bytes()
    (ROOT/'source.png').write_bytes(SOURCE.read_bytes())
    client=MCP()
    try:
        client.command('load_scene',path='house.scene.json')
        client.command('capture',path='repeat.png',width=1402,height=1122)
        checks['fresh-load-exact']=(ROOT/'repeat.png').read_bytes()==original
        client.command('capture',path='repeat.png',width=1402,height=1122)
        checks['repeated-capture-exact']=(ROOT/'repeat.png').read_bytes()==original
        client.command('set_camera',camera=VIEWS['back'])
        client.command('undo')
        client.command('capture',path='undo.png',width=1402,height=1122)
        checks['undo-camera-exact']=(ROOT/'undo.png').read_bytes()==original
        for key in structure:
            topology[key]=client.command('inspect_mesh_topology',asset=key)
        write_json(ROOT/'native-topology.json',topology)
        client.command('save_replay',path='static.replay.json')
        replay=client.command('run_replay',path='static.replay.json')
        write_json(ROOT/'replay-report.json',replay)
        checks['static-replay-matched']=replay['matched']
        client.command('capture_replay',replay_path='static.replay.json',path='replay.png',tick=0,width=1402,height=1122)
        checks['static-replay-pixels-exact']=(ROOT/'replay.png').read_bytes()==original
        regions=[{'id':key,'x':box[0],'y':box[1],'width':box[2]-box[0],'height':box[3]-box[1]} for key,box in REGIONS]
        comparisons=[]
        for i in range(0,len(regions),16):
            comparisons.append(client.command('compare_images',reference_path='source.png',candidate_path='reference.png',
                regions=regions[i:i+16],difference_path='difference.png'))
        write_json(ROOT/'comparison.json',comparisons)
        checks['source-preserved']=hashlib.sha256(SOURCE.read_bytes()).hexdigest()==source_hash=='4153a456f9fdf2e9e6f43328171692156db7563c2062654199e66c0c540882ad'
        report={'checks':checks,'closed_bodies':structure,'roof':roof,'apertures':aperture_checks,
                'native_counts':json.loads((ROOT/'native-counts.json').read_text()),
                'render_sha256':hashlib.sha256(original).hexdigest(),
                'scene_sha256':hashlib.sha256((ROOT/'house.scene.json').read_bytes()).hexdigest(),
                'fidelity_accepted':False,'note':'Topology and wall apertures, not self-intersection/physical collision/architectural or artistic acceptance. Replay is static tick0.'}
        write_json(ROOT/'verification.json',report)
        assert all(checks.values()),checks
        print(json.dumps(checks))
    finally:
        client.close()


REVIEWS={
 'S01-main-body':'Há corpo fechado, canto comum e profundidade; proporções do frontão, posição relativa dos vãos e riqueza dos perfis continuam diferentes.',
 'S02-wing-porch':'Portas e janelas estão atrás da linha de apoios e há espaço de circulação. Cobertura e arcos ainda são simplificações visíveis.',
 'S03-roof-structure':'As três coberturas têm espessura e continuação; a silhueta, os espigões e o tamanho das telhas divergem.',
 'S04-patio-access':'Pátio, dois pilares, escada externa e escada da porta existem. Traçado em planta e posição projetada do acesso ainda diferem.',
 'D01-main-window':'Há vão através da parede, rede clara separada da dourada e profundidade com prateleiras. Rede é mais simples e vidro/reflexos estão ausentes.',
 'D02-upper-gable':'Campos e pilarete estão ligados ao corpo. A configuração superior e os relevos não reproduzem o recorte.',
 'D03-dormer':'Água-furtada tem corpo próprio fechado e cobertura. Está maior/mais quadrada e o encontro de telhas não foi recortado no pano hospedeiro.',
 'D04-side-windows':'Dois vãos compartilham a parede lateral real. Molduras e ramificações são mais simples; painéis verdes são aproximação opaca.',
 'D05-porch-left':'Porta, arco dianteiro, apoio, cadeira e vasos estão em profundidades diferentes. Falta a riqueza de nervuras e escultura.',
 'D06-porch-center':'Banco, apoios e janelas têm corpos e posições espaciais. Perfis, espaçamentos e oclusões não coincidem com a foto.',
 'D07-entry':'Porta da parede posterior, moldura e três degraus são distintos da varanda. Ramos, camadas e lanternas ainda são esquemáticos.',
 'D08-gallery':'Cinco vãos altos visíveis estão entre coberturas. Forma dos montantes e proporções da galeria divergem.',
 'D09-rail-left':'Módulos foram distribuídos num percurso 3D com base e corrimão. Traçado e árvore de cada módulo precisam refinamento.',
 'D10-rail-right':'Trecho curvo, pilares e módulos existem em volume. Curvatura e espessuras não reproduzem o detalhe da referência.',
 'D11-medallion':'Campo verde, oito contornos foliformes e faixa curva estão no piso. Rede, dimensão e paginação continuam diferentes.',
 'D12-finials':'Remates têm haste e contorno foliforme presos à cobertura. Desenho, espessura e quantidade de subpartes são simplificados.',
 'D13-eave':'Faixa e cobertura são espessas e contínuas. Espigão esquerdo e subdivisão dos panos não foram reproduzidos fielmente.',
 'D14-garden':'Há vegetação em 3D com caules e grupos de flores. Densidade, distribuição, espécies aparentes e escala são insuficientes.',
 'D15-ornament':'A árvore clara tem haste contínua e três pares de ramos, distinta da rede dourada. Curvas, ligações e vazios seguem diferentes.',
 'D16-entry-branch':'Árvore dourada está hospedada nas folhas da porta. O padrão não reproduz todas as bifurcações e espessuras observadas.',
 'D17-tile-sample':'Centenas de telhas geométricas seguem os panos e têm juntas desencontradas. São grandes, retangulares e pouco envelhecidas.',
 'D18-porch-capitals':'Apoios nativos encontram a cobertura e os arcos são 3D. Capitéis, nervuras e vazios ornamentais continuam grosseiros.'}


def report():
    from PIL import ImageDraw
    source=Image.open(SOURCE).convert('RGB');candidate=Image.open(ROOT/'reference.png').convert('RGB')
    for key,box in REGIONS:
        a,b=source.crop(box),candidate.crop(box)
        pair=Image.new('RGB',(a.width+b.width+16,max(a.height,b.height)+32),'#eee8dc')
        pair.paste(a,(0,32));pair.paste(b,(a.width+16,32))
        draw=ImageDraw.Draw(pair);draw.text((8,8),'Fonte original',fill='#252b29');draw.text((a.width+24,8),'Render: mesma janela de pixels',fill='#252b29')
        pair.save(ROOT/('pair-'+key+'.png'))
    scene=json.loads((ROOT/'house.scene.json').read_text())
    # Horizontal section of exported triangles; this is a client drawing, not a native blueprint API.
    plan=Image.new('RGB',(1050,820),'#f5f0e5');draw=ImageDraw.Draw(plan)
    project=lambda p:(70+(float(p[0])+7.5)*57,90+(float(p[2])+3.5)*57)
    for x in range(-7,8):draw.line([project([x,0,-3.5]),project([x,0,7.2])],fill='#e1ded5')
    for z in range(-3,8):draw.line([project([-7.5,0,z]),project([7.5,0,z])],fill='#e1ded5')
    for asset in sorted(scene['meshes']['assets'],key=lambda a:a['id']!='volume-groundworks'):
        key=asset['id'];points=np.array(asset['geometry']['positions'])
        if key=='volume-groundworks':
            for tri in asset['geometry']['triangles']:
                ps=points[tri]
                if np.max(np.abs(ps[:,1]-.55))<1e-5 and np.cross(ps[1]-ps[0],ps[2]-ps[0])[1]>0:
                    draw.polygon([project(p) for p in ps],fill='#d8d3c6')
        if key not in ['volume-main-body','volume-wing-body','volume-wood']:
            continue
        for tri in asset['geometry']['triangles']:
            ps=points[tri];hits=[]
            for a,b in zip(ps,np.roll(ps,-1,axis=0)):
                if (a[1]-1.63)*(b[1]-1.63)<0:
                    hits.append(a+(b-a)*(1.63-a[1])/(b[1]-a[1]))
            if len(hits)==2:
                draw.line([project(p) for p in hits],fill='#3d655c' if key.endswith('-body') else '#8c6039',width=3)
    for instance in scene['meshes']['instances']:
        if instance['asset']=='volume-column':
            x,y=project(instance['position']);draw.rectangle([x-7,y-7,x+7,y+7],fill='#a48245')
    draw.text((35,25),'E02: corte horizontal real da cena exportada, y=1.63',fill='#25372e')
    draw.text(project([-4,0,-1.5]),'Ala fechada',fill='#25372e')
    draw.text(project([2.8,0,-1.0]),'Corpo alto',fill='#25372e')
    draw.text(project([-4,0,1.3]),'Varanda aberta',fill='#25372e')
    draw.text(project([-.6,0,3.8]),'Patio descoberto',fill='#25372e')
    draw.text((35,730),'Verde: paredes em corte. Castanho: madeira em corte. Quadrados: apoios nativos.',fill='#25372e')
    draw.text((35,750),'Eixos: +X para direita; +Z para baixo. Profundidades e interior sao hipoteses.',fill='#25372e')
    draw.text((35,770),'Desenho do cliente a partir dos triangulos salvos; nao e exportador nativo de blueprints.',fill='#25372e')
    plan.save(ROOT/'plan.png')
    payload={'assets':scene['meshes']['assets'],'instances':scene['meshes']['instances'],'camera':scene['camera']}
    template=(PROJECT/'scripts/viewers/scene_orbit.html').read_text()
    template=template.replace('__SCENE_JSON__',json.dumps(payload,separators=(',',':')))
    template=template.replace('src="reference.png"','src="source.png"').replace('href="differences.html"','href="index.html"')
    template=template.replace('Jogo das diferenças','Capturas e revisão').replace('Fundos e fechamentos do modelo ainda estão incompletos.',
        'Corpos fechados inspecionados; profundidades e traseira são hipóteses. Detalhes e aparência ainda divergem da referência.')
    template=template.replace('E02 · cena exportada da Ignis','E02 · nova construção volumétrica')
    (ROOT/'orbit.html').write_text(template)
    verification=json.loads((ROOT/'verification.json').read_text())
    totals={'assets':len(scene['meshes']['assets']),'instances':len(scene['meshes']['instances']),
            'vertices':sum(len(a['geometry']['positions']) for a in scene['meshes']['assets']),
            'triangles':sum(len(a['geometry']['triangles']) for a in scene['meshes']['assets'])}
    compact={k:verification[k] for k in ['checks','scene_sha256','render_sha256','fidelity_accepted','note']}
    compact['counts']=totals
    compact['bodies']={key:{k:v[k] for k in ['vertices','faces','boundary_edges','non_manifold_edges','inconsistent_edges','solid_preflight_passed','components']}
                       for key,v in verification['closed_bodies'].items()}
    compact['apertures']=verification['apertures']
    compact['pixel_metrics']=json.loads((ROOT/'comparison.json').read_text())[0]['report']['metrics']
    write_json(PROJECT/'docs/references/image-to-3d/e02-volume-verification.json',compact)
    review={'source_sha256':hashlib.sha256(SOURCE.read_bytes()).hexdigest(),'candidate_sha256':verification['render_sha256'],
            'policy':'Same fixed crop in both full 1402x1122 images. No independent alignment/resizing.',
            'camera':scene['camera'],'reviews':[{'crop':key,'diagnosis':REVIEWS[key],'accepted':False} for key,_ in REGIONS]}
    write_json(PROJECT/'docs/references/image-to-3d/e02-volume-review.json',review)
    analysis=json.loads(DATA.read_text())
    bindings=json.loads((ROOT/'bindings.json').read_text())
    task_progress=[]
    for task in analysis['tasks']:
        objects=[key for key,groups in bindings.items() if task['group'] in groups]
        if task['group']=='COL':objects.append('volume-column')
        task_progress.append({'id':task['id'],'candidate_objects':objects,
            'status':'requires_item_review','fidelity_accepted':False,
            'note':'Family binding is not evidence that this individual feature was reproduced.'})
    write_json(PROJECT/'docs/references/image-to-3d/e02-volume-progress.json',{
        'scene_sha256':verification['scene_sha256'],'tasks':task_progress,
        'scope':'189 entries retained; family bindings do not mark individual tasks built/accepted.',
        'known_missing_or_incomplete':['Glass transparency/reflections','Local warm lights',
            'Fine carved profiles and grooves','Rounded small tiles','Exact arch/motif graphs',
            'Left roof hip/valley layout','Dense vegetation and plant varieties','Detailed irregular paving and wear']})
    head='<!doctype html><html lang="pt-BR"><meta charset="utf-8"><title>E02 — nova construção</title><style>body{font:16px system-ui;background:#ece7dd;color:#25332e;max-width:1350px;margin:auto;padding:24px}img{max-width:100%;height:auto}section{margin:36px 0}a{color:#285f4b}.pair{display:grid;grid-template-columns:1fr 1fr;gap:12px}h1{font-size:26px}pre{white-space:pre-wrap}</style>'
    content=['<h1>E02 — nova construção volumétrica</h1><p><a href="orbit.html">Abrir modelo 3D: girar, aproximar e inspecionar</a></p>',
        '<p>Nova análise:188tarefas iniciais e uma hipótese de passagem interna acrescentada na revisão espacial, com22recortes. Corpos da casa e água-furtada são fechados; pisos, paredes internas e vãos fazem parte da geometria. A semelhança artística continua insuficiente.</p>',
        '<div class="pair"><figure><img src="source.png"><figcaption>Fonte original</figcaption></figure><figure><img src="reference.png"><figcaption>Render nativo</figcaption></figure></div>',
        '<h2>Outros ângulos nativos</h2>']
    for name in ['front','right','left','back','top']:
        content += ['<details><summary>'+name+'</summary><img src="'+name+'.png"></details>']
    content += ['<h2>Verificação técnica</h2><pre>'+html.escape(json.dumps(verification['checks'],indent=2))+'</pre>',
                '<h2>Planta de inspeção</h2><p>Corte horizontal dos triângulos reais exportados; desenho no cliente, com hipóteses de interior.</p><img src="plan.png">',
                '<p>Testes geométricos e replay estático não certificam fidelidade, ausência de interseções ou colisão física. Visualizador tem iluminação própria.</p>',
                '<h2>Comparações locais — janelas fixas</h2><p>Ângulo ainda aproximado: discrepâncias de posição e escala permanecem visíveis; os recortes não foram recentralizados.</p>']
    for key,_ in REGIONS:
        content += ['<section><h3>'+key+'</h3><p>'+html.escape(REVIEWS[key])+'</p><img src="pair-'+key+'.png"></section>']
    (ROOT/'index.html').write_text(head+'\n'.join(content)+'</html>')
    print('Published native captures, fixed-window review and generic orbit viewer')


if __name__=='__main__':
    parser=argparse.ArgumentParser()
    parser.add_argument('--analyze',action='store_true')
    parser.add_argument('--probe',action='store_true')
    parser.add_argument('--geometry',action='store_true')
    parser.add_argument('--stage',choices=['structure','detail'])
    parser.add_argument('--verify',action='store_true')
    parser.add_argument('--report',action='store_true')
    args=parser.parse_args()
    if args.analyze:
        analyze()
    if args.probe:
        probe()
    if args.geometry:
        geometry_probe()
    if args.stage:
        publish(args.stage)
    if args.verify:
        verify()
    if args.report:
        report()
