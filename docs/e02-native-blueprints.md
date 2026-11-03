# E02 — fontes paramétricas e desenhos construtivos

Nova leitura para o pedido de mostrar as plantas/blueprints. Fontes construtivas:
imagem E02 fornecida, skill ignis-image-to-3d e schema/respostas MCP do build.
Não carregar cena, receita, inventário ou parâmetros anteriores. O contexto da
conversa permanece; não apresentar como teste cego independente.

## Análise antes da geração

Casa com corpo alto à direita e ala baixa à esquerda, coberturas em dois níveis
na ala e pátio aberto dianteiro com balaustrada curva. Fachadas apresentam vãos
apontados, colunas perfiladas e motivos ramificados. Duas entradas são visíveis:
uma sob o extremo esquerdo da varanda e outra perto do corpo alto. Há escada
no acesso do pátio e degraus próprios junto à entrada central. A grande janela
pertence à face do corpo alto voltada ao pátio; janelas menores estão na lateral.
Água-furtada se projeta do pano principal. Vegetação, vidro/interior, revestimento
e ornamentos são evidências distintas da planta arquitetônica.

| ID | Evidência / leitura local | Consequência e incerteza |
|---|---|---|
| B01 | Conjunto inteiro: volumes fechados atrás, espaço aberto dianteiro | Corpo alto, ala e pátio separados; profundidades e fundos não são observáveis |
| B02 | Cobertura principal: cumeeira alta à esquerda do vão grande, pano descendo e recurvando no beiral direito | Dois panos assimétricos; trajetória curva de cada pano, encontro longitudinal ainda hipotético |
| B03 | Ala esquerda: cobertura superior e varanda mais baixa; janelas na faixa entre níveis | Duas fontes de cobertura, faixa de galeria deve permanecer livre; telhados não são uma única caixa |
| B04 | Varanda: apoios perfilados e arcos diante de portas/janelas da parede posterior | Colunas afastadas da parede; pelo menos quatro apoios legíveis, apoio extremo/oculto incerto |
| B05 | Entrada central e entrada esquerda: vãos distintos; central tem pequeno acesso elevado | Duas portas na face voltada ao pátio como hipótese; plano/recesso exato incerto |
| B06 | Janela maior: duas laterais curvas convergem ao vértice, moldura e árvore interna | Perfil externo pode ser fonte curva; árvore e vidro não equivalem a esse perfil |
| B07 | Pátio: cerca envolve arco, tem abertura frontal; escada larga e curva | Abertura livre entre os retornos; não supor circunferência exata ou escala métrica da foto |
| B08 | Plantas/vasos e detalhes de pedra/motivos | Não fazem parte da verificação do blueprint estrutural; permanecem pendentes nesta etapa |

Eixos desta hipótese: X ao longo da ala, Y vertical, Z para o pátio; corpo alto
à direita e atrás do pátio. Portas na face +Z da ala; grande janela na face +Z
do corpo alto; janelas menores esperadas na face +X. Piso, colunas e coberturas
têm fontes/instâncias separadas. A hipótese inicial usa faces ortogonais;
o ângulo real da ala e profundidades não são conhecidos. Não inventar cômodos,
paredes internas ou cotas métricas inferidas da imagem.

## Ferramenta e escopo

No build consultado não existe operação chamada blueprint nem exportador de
desenhos2D. O recurso nativo é `preview_plant`/`edit_plant`/`inspect_plant`:
perfis, dimensões, seções e trajetórias geram malhas com fonte preservada.
Usar essas fontes efetivamente; não chamar um desenho externo de ferramenta
nativa de blueprints. O cliente desenha projeções das malhas retornadas pelo
MCP, acompanhadas das definições que realmente as geraram.

Esta unidade gera e mostra primeiro as fontes estruturais, implantação e
elevações. Não declarar a casa completa ou fiel: arcos da varanda, galeria,
motivos internos, janelas laterais, vidro, revestimento e vegetação ainda
precisam de suas próprias fontes/revisão. O desenho estrutural expõe essas
ausências; nenhum desses itens deve ser aprovado por existir uma caixa próxima.

## Revisão estática antes da autoria

1.Comandos de autoria sob demanda, nenhum tick/frame novo. 2–3.Definições e
buffers JSON limitados pelo preflight MCP; projeções Pillow no cliente, sem
nova dependência Rust. 4.Uma fonte por tipo construtivo, instâncias quando
iguais. 5.Até16fontes/32instâncias e16384v totais conforme inspect_meshes;
caminhos estáveis. 6.Capturas GPU offscreen explícitas, sem janela; imagens2D
somente de malhas realmente inspecionadas, sem geração raster por IA.
7.Processo MCP próprio/sequencial, revisão esperada. 8.Preview sem mutação,
persistência sem alterar a cena anterior. 9.Conferir limites, fontes,
capturas repetidas, save/load, AST, referências e desenhos. 10.Câmera manual,
projeções ortográficas no cliente e hipótese espacial continuam limitações.

## Resultado e diferença entre fonte e blueprint desenhado

Execução: `scripts/e02_native_blueprints.py`, processo stdio MCP próprio,
build9860a09115365ad3. Foram criadas16plantas nativas e31instâncias,
2278vértices/2620triângulos armazenados. Nenhuma malha foi enviada por
`put_asset`: toda geometria vem de `edit_plant`. `inspect_plant` confirma fontes
e definições; `inspect_mesh` fornece a malha avaliada usada nos desenhos.
As16fontes canônicas coincidem com as16malhas avaliadas nesta configuração.

As pranchas PNG são um visualizador no cliente: projeções ortográficas das
malhas inspecionadas, silhueta rasterizada e composição por instância com a
mesma transformação/quaternion do documento. Não são um novo comando MCP nem
um exportador CAD nativo. As escalas dos quadros de fontes são independentes;
W/H/D são parâmetros da fonte, não cotas recuperadas da fotografia. A prancha
de implantação usa escala comum dentro de cada vista. Vista superior não é
um corte arquitetônico, não acrescenta paredes/cômodos e mostra também os
degraus elípticos completos parcialmente sobrepostos ao piso.

Arquivos correntes em `artifacts/house-e02-blueprints/`:

- `sources.png`:16desenhos das fontes nativas, IDs e dimensões de autoria;
- `blueprints.png`: implantação sem telhados, elevação frontal/lateral e coberturas;
- `render.png`: captura nativa real do estudo estrutural;
- `house.scene.json`: cena nativa com fontes regeneráveis;
- `definitions.json`, `plants.json`, `meshes.json`, `counts.json`: entrada e inspeções;
- `schema.json`, `limits.json`, `initialize.json`: interface/build usados;
- `regions.json`, `*-reference.png`, `*-candidate.png`, `comparison.json`:
  quatro recortes fixos e comparação;
- `verification.json`: checks de repetição, carga e correspondência das fontes;
- `refused-source.json`, `capability-probes.json`, `curve-probes.json`: recusas
  dos previews iniciais por trajetória, antes de escolher outra parametrização.

Manifesto construtivo versionado:
[e02-native-blueprints.json](references/image-to-3d/e02-native-blueprints.json).

## O que o uso da ferramenta revelou

O primeiro pano de cobertura por trajetória cúbica foi recusado com
`Trajectory/section volume cannot certify a unique inverse; reduce bend or
thickness, or increase height`. Reduzir só espessura ou largura não resolveu
os casos sondados; uma trajetória quase reta foi aceita. Isso não prova que
toda cobertura curva seja impossível: é uma recusa das configurações testadas.
Não consultar a implementação Rust para contornar a validação.

A fonte final usa13estações com deslocamentos XZ e interpolação `pchip`,
amostradas da curva proposta. São seções paralelas ao plano XZ local, ao
contrário dos cortes perpendiculares à tangente da trajetória. Essa escolha
mantém uma construção nativa editável para a extrusão fina da cobertura,
mas não equivale ao mesmo transporte de seção. A curva não foi achatada nem
substituída silenciosamente por uma malha direta.

Os recortes da referência acrescentaram distinções importantes: a entrada
central tem degraus próprios, a varanda tem nervuras entre arcos/apoios e a
grande janela contém duas camadas de ramificação. Essas observações não foram
implementadas nesta etapa de fontes estruturais; continuam pendências explícitas.

## Revisão visual e limites

Foram abertos referência inteira, quatro recortes reais e captura do candidato,
além das duas pranchas finais. Diagnóstico dos pares:

| Região | O candidato realmente mostra | Pendência |
|---|---|---|
| Telhados | Dois níveis baixos e dois panos assimétricos altos | Formas longitudinais, encontros, suportes e espessuras não explicam a referência; cobertura alta cobre área insuficiente da fachada |
| Varanda | Quatro colunas perfiladas e duas portas em parede lisa | Faltam arcos/nervuras, galeria, bancos, janelas baixas e degraus próprios da porta; posições/profundidades são hipóteses |
| Janela grande | Somente duas meias molduras curvas, abertas | Contorno projetado deslocado; faltam fechamento de fachada ao redor, montantes, grafo ramificado, vidro e interior |
| Pátio | Piso elíptico, duas travessas curvas, apoios e três degraus | Cerca sem painéis em arco, piso/vegetação ausentes; elipse e proporções ainda não ajustadas à imagem |

SSIM global0.198308, MAE0.362834 nesta captura; regiões:telhados0.209955,
varanda0.051430,janela0.043032,pátio0.092739. Câmera/aparência diferentes do
ensaio anterior: não usar essas notas para um ranking A/B ou porcentagem de
fidelidade. São medidas do par desta condição e não aprovam a construção.
Este resultado não é uma nova casa concluída nem melhora de fidelidade. Expõe
as fontes estruturais solicitadas para inspeção, inclusive o que falta nelas.

## Validação posterior

Captura repetida e save/load reproduzem exatamente os pixels. Todas as fontes
foram confirmadas pelo MCP, fontes/malhas avaliadas coincidem e os desenhos
usam essas mesmas malhas. Conferir AST, índices/bounds finitos, quaternions,
limites e recursos dos desenhos. Sem Rust/schema/kernel novo, sem regressão
Rust ou novo teste de replay/undo, sem interação de janela/aceite humano.
Não houve mudança na cena E02 anterior ou conteúdo Unreal.

Custos mantidos: cópias JSON/PNG e projeção/rasterização no cliente; capturas
GPU somente por comando explícito. Nenhuma mudança por tick/frame ou ganho de
performance alegado. Fontes16/16 e instâncias31/32 tornam o orçamento de
componentes uma restrição, mesmo com vértices ainda abaixo do teto.
