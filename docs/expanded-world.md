# Paisagem ampliada e superfície conforme — EAF-185

Pedido humano no chat nanites: personagem com visão ao nível dos olhos,
caminhada normal e maior recorte testável que a demonstração2×2km.
Systems possui fonte/grade/receita/base/colisão; nanites possui câmera de sessão,
launcher contínuo, implantação de floresta e apresentação final. Nenhum novo
Scene/replay/checkpoint/protocolo/terrain2 é introduzido pela grade.

## Escolha e limites

O candidato20×20km ocupa o frame autoral atual[-10000,10000] emXZ. Não é um
máximo mundial, nem prova de streaming ou mudança de origem física. Uma grade
uniforme4m precisaria25milhões de vértices/50milhões de triângulos, cerca900MB
só de posições/índices, antes de física/GPU/estados/serialização. Não provocar
pressão de memória artificial na máquina de trabalho para descobrir um teto.

Quatro quadrantes10km usam a mesma fonte e eixos conformes:4m no miolo
[-1000,1000] e até32m no exterior, com passos terminais menores para atingir
o limite exato. Há1.136.356v/2.264.192tri,40.806.576B de posições/índices e
13.636.272B do canal de coordenadas de material. São contagens de buffersCPU,
não RAM total nem memóriaGPU. O detalhe é estático: ao caminhar para longe,
o terreno continua32m; ainda não há refinamento que acompanha o personagem.
Faixas finas também se estendem pelos eixos, consequência da grade tensorial.

Todas as regiões ficam residentes e são publicadas antes da entrega. Mesmos
vértices/triângulos completos no desenho e no TriMesh físico, sem piso oculto
nem colliderLOD. Bordas têm amostras e segmentos coincidentes; não há
T-junction de grade fina contra grossa. Normais são calculadas porasset no
renderer existente: alturas iguais não provam normais iguais nas junções.
Aceite visual precisa de imagem nativa; não modificar shader para esconder
falha nem alegar normais compartilhadas entre assets.

Nove cadeias orientadas possuem centros, alturas e extensões próprias na
fonte global, mais planície irregular, vale/canal e costa. Não é ampliar a água
ou escalar a ilha anterior. Água continua um plano visualY0,05, sem fluxo,
natação ou hidrologia. Parte da área física pode ficar submersa; área20km não
significa20km de terra seca ou que todas as encostas são escaláveis.

## API

`Region.axis_offsets_meters: {x:[...],z:[...]}` opcional substitui o espaçamento
uniforme quando presente. `cells` precisa coincidir com comprimentos−1; eixos
começam em0, finitos/crescentes, extensão<=10000 e passos0,125..4096m. Campo
ausente mantém exatamente o caminho uniforme anterior. `spacing_meters`
continua obrigatório/válido no contrato legado; no modo explícito quem define
os passos são os arrays. Até8192valores/eixo no decoder,1Mvertices/64Mtrabalho
por publicação mantidos. Erros de grade são recusados antes dos buffers.

`Publication.world_material_coordinates:true` usa posições projetadas no frame
da publicação como canal material_coordinates. O mesmo pattern continua ao
cruzar os quatro assets, em vez de reiniciar em cada origem local. É canal
nativo existente; não altera posições/índices nem física. Padrãofalse conserva
publicações antigas. Coordenadas do canal obedecem à validação±10000m.

A primeira geração descobriu uma guarda acidental: a publicação regional
projetava a origem porraio10000m, recusando o canto[-10000,0,-10000] apesar de
ele caber na caixa da cena. O adaptador agora usa esfera circunscritora20km e
revalida componentes±10000 ANTES de gerar a malha. WorldSpace e o envelope
da cena permanecem iguais. A prova crítica existente inclui o canto diagonal,
coordenada de material correspondente e rejeição de componente10001m.

## Revisão estática antes/depois

1. Validação e geração são frias: preflight de contagem/trabalho antes dos
   buffers, sampler preparado uma vez/região. NenhumJSON/I/O/alocação de grade
   por tick ou frame; arrays explicam a resolução sem funções de câmera ocultas.
2. Eixos são pequenos/densos/limitados; indexação direta, sem busca por ponto
   durante o bake. Triângulos e vértices têm contagens calculadas antes de
   reservar. Geometry emArc; adicionar canal de material antes de compartilhar.
3. Fontef64/células globais, publicaçãof32/u32, mesmos tipos dos caminhos
   existentes. FonteTerrain2 permanece versionada; documento velho não recebe
   Ridge silenciosamente. Cotas e envelopes não ampliados.
4. Gerador envia documentos/eixos peloMCP e publica nativamente; não transmite
   milhões de vértices porJSON. Posição/apoio consultados com inspect_world e
   inspect_character_controllers, sem inspect completo da cena grande.
5. Estado físico/BVH é construído nas transações frias; não reconstruir ao
   mover a câmera ou andar. Câmera de sessão e launcher são de nanites.
   RSS amostrado porprocesso não mede toda memória unificada/GPU nem pico exato.
6. Sem nova dependência, thread/lock no runtime ou renderer. O monitorRSS é
   exclusivamente do script de ensaio e termina com ele. Não há promessaFPS.
7. Uma prova crítica agregada acrescentada ao teste terrain_region protege
   grades malformadas, limites do decoder e identidade de bordas; cobertura
   prévia de atomicidade/formato e física selecionada conforme impacto.

## Reprodução e evidências

```sh
python3 scripts/build_representative_terrain.py --plan-expanded
python3 scripts/build_representative_terrain.py --expanded
```

Receita/fonte: examples/expanded-world.recipe.json e .ignis-terrain. Saída
corrente artifacts/expanded-world/terrain.scene.json e placement.json. A cena
final world.scene.json pertence a nanites, gerada somente após congelar a base.
Placement fornece regiões/eixos, bounds, vistas, spawn/clareiras e forest_zones:
12mil árvores no miolo2km e18mil no exterior, não densidade uniforme por400km².

O script mede cada operação fria, RSS próprio a cada0,25s, carregamento,
capturas nativas e quatro caminhadas: spawn, cruzamentoX, cruzamentoZ e ponto
distante. Probes usam altura interpolada dos quatro vérticesf32/mesma diagonal
b-c, posição real/grounded e progresso, não movimento de animação. A região
deve ser avaliada por imagem e caminhada humana após os gates automatizados.

## Resultados nativos

Build conjunto6dda7b1233983522 (release/aarch64/macOS). Gates conjuntos da
câmera/controles e terreno:155Rust/7alvos; após a correção diagonal,3provas
terrain_region repetidas, Clippyalltargets-Dwarnings e buildrelease aprovados.
MCP verify_terrain_region passou2grupos no executável final: legado1/2,
arquivos/metadados, publicação/região grande/undo/save-load/replay pequeno.
Não alegar execução da suíte inteira. Fmt/Python/diff verificados separadamente.

Geração20km real confirmou as contagens e o mesmo digest de fonte nas quatro
regiões. Publicações:0,481/0,855/1,235/1,654s, total4,224s nesta execução com
Mac em uso. RSS após publicar quarto quadrante935.477.248B (~892MiB), após
salvar base com player/luz1.146.142.720B (~1093MiB). Scene107.616.361B
(~102,63MiB), SHA2568915ada3fc01c6abfe373f3e84bcacacacc5473fa6fd364582e39466c81b68be.
Digest da fonte2ecf9df7ee781e10ac1c94612c2fdec493bf1d3d590d2f1664c6c3c7c4a00999.

Caminhadas: spawn11,596m, junçãoX12,158m, junçãoZ11,986m e ponto distante
[-8000,-2000]11,992m, todosgrounded. Os cruzamentos exigem posição inicial
negativa e final positiva no eixo correspondente. Nenhuma animação usada como
prova. Capturas nativas1728×1117 terrain-play/panorama foram inspecionadas.
PicoRSS amostrado4.236.902.400B (~3,95GiB) após capturas e recargas repetidas;
isso inclui custos frios/undo/driver e não representa um launcher recém-aberto.

Ensaio adicional em processo novo, continuous, semGPU e sem histórico de edição:
load2,068s, RSS616.529.920B após load e617.349.120B (~589MiB) após640ticks;
600ticks de caminhada avançaram39,504m, grounded, retençãoreplay0. Passo em
lote4,16ms não éFPS ou frame budget completo. Não inclui floresta/desenho,
eRSS não mede toda memória unificada nem prova ausência de picos transitórios.
Gerador também chama esse ensaio ao final; relatório separado
terrain-residency-verification.json. O processo próprio encerrou limpo.

Nanites integrou30mil árvores/60mil componentes:7assets/60005instâncias com
terreno/oceano. Checagem das quatro bordas533v/aresta no arquivo real: erro
máximo posição mundial0 e material0; enraizamento após save/load<=0,000675m.
Fonte física preservada; árvores seguem visuais. Captura humana walk.png
inspecionada: clareira/chão/árvores/cristas coerentes, personagem oculto pela
visão de sessão. Uma espécie/modelo e solo sem acabamento final permanecem.

Panorama apresenta moiré/pontilhado sobre água/áreas baixas e limite perceptível
do plano oceânico. Não há duplicação coplanar óbvia na autoria:1plano de água e
4regiões disjuntas. A projeção principal usa perspective_rh NEAR0,05/FAR30000
eDepth32Float/Less; perda de separação de profundidade a quilômetros é hipótese
de diagnóstico comunicada a nanites, ainda não causa comprovada por oráculo.
Não elevar mar/terreno para esconder artefato; renderer/diagnóstico gráfico e
abertura final são da outra frente. Normais entre assets não foram certificadas.

Relatórios/logs correntes: terrain-verification.json, terrain-residency-verification.json,
region-mcp.log, terrain-runtime.log e terrain-residency-runtime.log em
artifacts/expanded-world; generation.log em artifacts/expanded-terrain-generation.log.
GatesRust/build/projeção pertencem ao ciclo comum em artifacts/representative-world.
Receita/fonte/base congeladas para a derivação de nanites. Nenhuma janela humana
foi aberta/fechada/focada por systems. Aceite humano da arte e do passeio final
segue aberto; não declarar máximo universal, streaming, replay/rede da cena
grande ou qualidade artística concluídos. Uma versão corrente do executável,
nenhuma cópia histórica/novo worktree ou push produzido por esta frente.
