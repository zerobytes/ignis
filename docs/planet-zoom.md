# Zoom da superfície ao globo — EAF-199

A visão planetária é estado da sessão, acessível pelo MCP e pela janela. Ela
acompanha o mesmo jogador e a mesma fonte Planet1 durante a passagem da primeira
pessoa ao enquadramento global. Não cria uma esfera de referência como substituto
do mundo e não transporta a física para a câmera.

## Controles e API

Na janela de gameplay planetário, roda aproxima/afasta, G alterna globo/superfície,
F retorna à superfície e Tab alterna distância zero/quatro metros. Clique e mouse
continuam controlando orientação; WASD/Shift continuam movendo o jogador. A
rotação da entrada usa o yaw da câmera ativa. G também está disponível como uma
operação FramePlanetView na API, que considera FOV e proporção do viewport; nenhum controle existe só na janela.

```json
{"command":{"op":"set_planet_view","view":{"distance_meters":19113000}}}
{"command":{"op":"frame_planet_view"}}
{"command":{"op":"zoom_planet_view","factor":1.5}}
{"command":{"op":"look_planet_view","yaw_delta_degrees":20,"tilt_delta_degrees":0}}
{"command":{"op":"set_planet_viewport","width":1280,"height":800}}
{"command":{"op":"inspect_planet_view"}}
{"command":{"op":"set_planet_view","view":null}}
```

Settings permite distância, yaw, tilt, FOV, viewport, erro geométrico pretendido,
orçamento de tiles, altura/profundidade óptica atmosférica e cor do espaço.
Campos omitidos usam defaults documentados pelo schema MCP. O helper Rust
`Engine::planet_view_settings` adapta defaults ao raio do planeta atual. Distância
zero significa olhos do jogador; null desliga o novo modo e restaura a câmera
anterior. A roda multiplica distância+1 para atravessar escalas sem passos fixos
em metros. Existe uma partida de 0,5m ao sair da distância zero.

CLI: `--continuous --planet-runtime CONFIG --planet-view-distance METROS`.
O launcher `python3 scripts/open_planet_zoom.py` usa o único executável corrente,
fonte/cena de `artifacts/planet-runtime` e manifesto corrente de `earth-walk`.
Ele recusa substituir silenciosamente outra demonstração ainda aberta.

## Precisão, projeção e cobertura

O olho e as bases são calculados em f64 no frame transportado do ator; a posição
absoluta continua em Position1 com células inteiras. Cada vértice visual guarda
a posição centrada no planeta, normalizada pelo raio, em duas partes f32. A GPU
subtrai o olho (também em duas partes) antes de aplicar raio e base comum do
observador. A mesma Key fornece os mesmos bits em todas as faces e costuras;
nenhuma matriz de anchor individual volta ao cálculo raster desse ponto.
Normais usam o referencial global do planeta e a mesma base do observador.

A Geometry local e as matrizes derivadas continuam úteis para a autoria e o
adapter; seus limites não foram ampliados. O culling planetário usa bounds das
coordenadas compartilhadas, com margem conservadora para as partes baixas e a
transformação CPU menos precisa, falhando aberto perto de limites ambíguos.

Near/far derivam do envelope conservador da fonte inteira e do jogador,
incluindo os cantos do viewport. A visão planetária usa reverse-Z FINITO:
near<far na API, argumentos invertidos na matriz, Greater/clear0 no terreno e
corpo, céu em0/GreaterEqual. Isso resolve a perda de ordenação em distâncias
intermediárias que ainda ocorria com near0,05/far14Mm convencionais. O caminho
legado permanece Less/clear1 e é restaurado por frame. Não se usa logdepth.
Oclusão GPU auxiliar, seletor ATM legado, IGL1, seleção/outline, sombras,
transparência e filtro por depth não têm esse contrato; os incompatíveis são
recusados ou explicitamente ficam OFF neste modo. Frustum, instancing e cache
incremental continuam ativos.

O stream do terreno tem60B por corner (hi/normal/lo/rest/weights), com instance80B.
Isso custa mais que48/24B; mantém precisão e todos os canais consumidos, não é
uma alegação de compressão. O override CPU usa24B por vértice indexado da fonte,
com identidade Arc própria e invalidação de cache; olhar/rebase não o recria.

As seis faces começam cobertas. Uma fila de prioridade refina tiles pelo limite
projetado de curvatura/relevo, até a resolução física canônica ou o orçamento.
Esse limite inclui perspectiva nos cantos e o cast local da fonte; não certifica
cor, normais filtradas, toda aritmética GPU nem igualdade de pixels entre LODs.
Ao esgotar orçamento, a cobertura mais grosseira permanece e a inspeção marca
quality_limited. Não há descarte de regiões que deixaria o céu aparecer no chão.
Uma esfera interna conservadora permite não gastar refinamento no hemisfério
oculto, sem retirar suas raízes de cobertura.

Endereços inteiros comuns às faces identificam arestas. Arestas grosseiras
recebem também todos os pontos dos vizinhos finos, com alturas amostradas na
mesma fonte, coordenadas GPU com os mesmos bits e normais globais com a mesma escala de filtro. Células de borda divididas
recebem triangulação conforme; não são saias nem dois terrenos sobrepostos.
Mudanças de costura entram na chave dos recursos. O candidato termina antes de
substituir a cobertura publicada.

A visão tem um só proprietário visual. A residência anterior do World continua
mantida em CPU para o adapter físico existente; ela não é desenhada por baixo da
malha global. Esse custo duplicado de derivação local é uma limitação explícita
deste incremento, não streaming gráfico/físico desacoplado já otimizado.

## Câmera e atmosfera

Até125m a câmera consulta a colisão já residente por sweep de esfera, excluindo
seu jogador. Mais longe usa afastamento radial da fonte analítica; não é uma
varredura física do percurso orbital. Somente o olho é ajustado, nunca o ator.
O corpo primitivo reaparece quando a distância efetiva permite vê-lo.

A atmosfera é um volume esférico homogêneo finito, recortado pelo primeiro
terreno. Raios que não atravessam ar mostram a cor do espaço. A espessura não é a
distância inteira da câmera até o chão. Na superfície, o cálculo usa a altura
radial interpolada recuperada do domínio canônico do terreno para não converter
a sagitta dos triângulos grosseiros em quilômetros artificiais de ar. Isso é
uma aproximação óptica vinculada ao relevo amostrado, sem alterar depth, malha ou
colisão; não simula espalhamento físico, nuvens, oceanos ou refração.

## Contratos e limites

Permanece o primeiro adapter offline: um Player/KCC e terreno opaco, sem
multiplayer/NPCs/assets arbitrários transportados. Cena, replay, save e formatos
de fonte não mudam por causa da câmera. Substituir a fonte/câmera autoral exige
primeiro desligar a visão; picking autoral pelo viewport orbital é recusado.
Reset limpa a visão da sessão. Captura e resize usam o mesmo comando de viewport.

A distância aceita0..1e18m e respeita o domínio Position1; não é infinito numérico.
O planeta fica subpixel antes do limite. Ainda não há galáxia, seleção entre
planetas ou representação de estrelas. O diâmetro continua sendo o da fonte,
independente da distância da câmera. A família artística de caps da demonstração
segue reprovada: zoom e cobertura global não encerram a correção Ridge pendente.

## Checklist obrigatório de performance (código)

- Há serialização no kernel/frame? Não. Settings entram pelo MCP na fronteira;
  amostragem, decisões e matrizes são tipadas em Rust. JSON não trafega vértices.
- O algoritmo reavalia todas as folhas para cada divisão? Não. BinaryHeap faz a
  seleção; bounds e erro de fonte são pré-calculados e guardados nos recursos.
- Movimento reconstrói tudo? Não. Atualizações pequenas alteram matrizes e o
  limite atual, mantendo Arcs; replanejamento tem limiar espacial/angular/zoom
  referido ao último plano, não ao último tick. Atualiza uma vez por advance.
- Há alocação por célula no gerador? Scratch principal de polígonos/índices é
  reutilizado; células de borda ainda recebem pequenos vetores temporários de
  costura. Mapas/registro são trabalho de troca de residência, fora do draw.
- Memória pode crescer com a área do planeta ou histórico de zoom? Não. Até1536
  tiles e2M vértices por atlas; atual/novo podem coexistir na transação. O World
  local conserva seu orçamento separado de2M. Recursos GPU obsoletos seguem a
  aposentadoria do cache existente, sem colecionar versões históricas.
- Há upload de geometria por olhar/rebase? Só placements se a fonte/costura não
  mudar. Mudanças reais de LOD/costura geram novos recursos e upload necessário.
- Há custo oculto nos fallbacks? Oclusão, ATM legado e IGL1 ficam OFF nesse modo;
  refinar/gerar é síncrono e pode causar pausa ao atravessar uma escala. Não há
  promessa de FPS nem geração assíncrona implementada neste incremento.
- Há cálculo caro desnecessário por frame? Envelope global, coeficientes do
  sampler e bounds/erro dos recursos são cacheados; o caminho aquecido não refaz
  preparação/trigonometria dos operadores. As folhas ainda recebem testes e
  matrizes. Oportunidade futura:
  geração assíncrona e desligar manutenção visual local do adapter físico.

## Validação

Evidência corrente: `artifacts/planet-zoom/verification.json` e capturas nativas,
produzidas por `scripts/verify_planet_zoom.py`, sem janela/foreground. A prova GPU
adicional está em `artifacts/planet-zoom/precision.json`:6 vistas de aresta/canto,
a poucos metros, com6/48tiles, RGB8 exato contra a malha fundida equivalente e
frustumOFF×ON exato;12 casos de profundidade com cores/ordens alternadas em
100/101km,200/205km e0,9/1m; e cache A→B→None→A preservando Geometry/normalArc,
mais restauração do legado no mesmo Frame/Renderer. Os testes
cobrem ida/volta, preservação da física/cena, rejeição atômica, cache aquecido e
silhueta de uma fixture plana independente. Os testes Rust verificam costuras
fechadas/orientadas entre três níveis/faces/cantos, limites da fonte, clipping e
bases em polos, configuração inválida e reuso de geometria. Aceite humano e FPS
apresentado são gates separados.

Evidência executada em03/11/2026:7 testes orbitais (incluindo a prova GPU acima),
layout de Globals448B e10 regressões de residência/runtime/visão aprovados.
Clippy release lib/bin/tests e fmt aprovados. O ensaio longo de circunavegação da
Terra foi mantido ignorado nesta repetição; não é uma nova prova de percurso.
Release corrente8784ca62e3513867; o ensaio GPU foi produzido em38af790d4fa7214f,
com o mesmo renderer, antes do ajuste final do getter da distância da3ªpessoa.
No MCP final:9 escalas, ida/volta com pixels idênticos e física preservada,
3 baselines opacos anteriores RGB8exatos, duas máscaras globais sem pixels
faltantes/externos além da faixa2px, atmosfera coarse×fine máximo1RGB8 no
interior e cache aquecido sem novos uploads. O orçamento768tiles foi atingido
na superfície e em100km; quality_limited permaneceu explícito nesses casos.
A validação foi em segundo plano. O launcher fica disponível para o teste
humano dos controles nativos; nenhuma janela precisou ser ativada para os gates.
