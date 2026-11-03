# Laboratório de autoria: árvore

## Casca integrada — seção e escala dos detalhes

Direção atual: avaliar realismo por referência e crítica própria, sem usar gosto
ou aprovação do observador como critério de avanço. A mesma árvore integrada70
foi copiada para o estudo71; canônica69 SHA89fe1fa2…9400 conservada.
Ablações sem folhagem separaram sombras de defeitos de domínio. Ruído alongado
25:1 parecia madeira polida; detalhes isolados pareciam fitas; celular forte
parecia escamas. Modificar apenas o padrão não resolveu os três problemas.

A ferramenta genérica agora oferece seção rounded e repetição2D de patches
limitados, no mesmo Detail/MCP. Permite testar fissuras curtas/variadas sem
multiplicar32segmentos; métricas, alternativas e limites em detail-domains.md.
O ensaio study_tree_bark.py compara4vistas com geometria/rig/clipes/instâncias
exatos e4neutros byte-idênticos. Fonte, luz, câmera e tick60 ficam explícitos.
A primeira receita de cortes finos em faixas foi rejeitada; a corrente suaviza
amplitude e varia fases/larguras/comprimentos, conservando a textura independente.

Ainda há periodicidade visível, distorção material no taper/forquilhas e aparência
procedural da copa. Não apresentar esta rodada como árvore realista concluída.
As próximas dificuldades são máscaras/materiais por região, escala física dos
traços, camadas de superfície e distribuição/forma das folhas; exigir prova
específica antes de alegar cada capacidade implementada.

## Casca — experimento de projeção de detalhes

O close do encaixe com frame material explícito e finisher ainda tem faixas
fortes e mudanças de direção. O ensaio triplanar usa o mesmo asset final,
material, câmera e luz: altera somente a projeção dos traços no RenderFrame.
Não altera a árvore canônica nem se apresenta como uma nova árvore concluída.

Dificuldade identificada: resolver continuidade do campo3D e escolher um plano
para desenhos2D são problemas diferentes. Colapsos na projeçãoXY podem existir
com campo3D regular; mudar para três planos evita depender de um só plano,
mas mistura fases/traços e não fornece uma direção de fibra contínua.
A ablação de cor deixa claro que as linhas longas ainda seguem o campo
estendido; o padrão3D da casca continua independente desse modo de detalhe.

A ferramenta genérica opt-in, os pesos por vértice, custos e aproximações estão
em [triplanar-details.md](triplanar-details.md). Prova/capturas correntes em
artifacts/triplanar-study/gpu (planar, sharpness1/2/4/8, neutral/color/bump/combined,
quatro vistas). Integração70/74 e avaliação do conjunto permanecem gates próprios.
Não ativar automaticamente no conteúdo como se a mudança de projeção resolvesse
o aspecto: próximo teste deve separar autoria da direção/escala dos traços da
continuidade geométrica, mantendo comparação fixa e opção de retornar ao planar.

## Objetivo e avaliação anterior à autoria

Direção de Guilherme: descobrir dificuldades reais das ferramentas usando um
alvo exigente, uma árvore de alta qualidade e realismo. A demonstração low poly
foi aceita; realismo continua sendo um objetivo, não um resultado presumido.

Primeira hipótese artística: corrigir escolhas minhas antes de atribuir o
resultado à engine. A árvore anterior tinha folhas grandes, ramos em andares,
eixos retos, repetição de conjuntos e pouca hierarquia de espessuras. Este
incremento experimenta copa irregular, ramificação hierárquica, folhas menores,
curvas contínuas e variação controlada. Referência de porte/ramificação, não
réplica botânica: [carvalho em Kew](https://www.kew.org/plants/oak-tree) e
[atlas florestal JRC](https://forest.jrc.ec.europa.eu/media/atlas/Quercus_robur_petraea.pdf).

Checklist estático proporcional, antes: sem mudar núcleo, Scene/Command/replay,
renderer ou contratos de plantas em desenvolvimento paralelo. Python descreve
geometria base e orquestra MCP; geração só na autoria, com seed fixa. Contar
vértices/triângulos por asset, armazenamento, instâncias, desenho, ossos/chaves
antes de publicar. Reusar validação/undo/persistência/rig/captura existentes.
Capturas offscreen explícitas e sequência finita em `artifacts/tree-study`, sem
novo build, janela ou I/O no tick. Memória finita dos frames da prévia; manter
paths correntes. Custos existentes de skinning CPU/upload/normais flat não
mudam; mais geometria não é alegação de FPS ou realismo.

## Fricções confirmadas antes da nova árvore

1. `meshes::Geometry` só contém posições/triângulos; `Instance` fornece uma cor
   única. Não consigo pintar nervuras, casca e face inferior da folha no mesmo
   asset. Separar assets/instâncias para cores gasta os limites16/32.
2. `Document::append_vertices` recalcula uma normal por triângulo. Aumentar
   tesselação reduz facetas; não equivale a controlar normais/vincos.
3. `scene.wgsl` tem luz difusa + ambiente, sem texturas, sombras de oclusores,
   transmissão ou rugosidade. O interior da copa não recebe sombra das folhas;
   não vou pintar iluminação falsa e apresentá-la como capacidade do renderer.
4. Limite32768 triângulos desenhados: a folha anterior152t repetida2100 vezes
   custaria319200t, antes da madeira. Tenho de distribuir o budget e simplificar
   folhas manualmente; não existe LOD de autoria/render integrado nesta frente.
5. Não há sweep de curva, junção de ramos, scatter/hierarquia de instâncias ou
   máscara/picking de superfície do pincel. Os cálculos estão no script; isso é
   trabalho real do autor fora da biblioteca nativa de operações, embora o
   transporte/publicação/render sejam MCP. Não chamar o script de novo editor.

## Critérios da rodada

Guardar uma fonte reproduzível, capturar antes/depois em câmera igual e mostrar
frente/costas/detalhes da árvore nova. Validar poses distintas, ciclo, save/load,
replay e pixels. Avaliar silhueta, espessuras, junções, escala/densidade das folhas,
repetição e comportamento do vento. Registrar falhas e workarounds observados,
não só os checks que passam. Arte/realismo exigem avaliação visual separada.

## Rodada executada — EAF-80

`python3 scripts/demo_tree_authoring.py --frames 0` produz a cena, as vistas e
o relatório com o binário corrente. `--frames 48`, com Pillow disponível, também
codifica um GIF de4s a partir de48 PNGs nativos840×770. Helpers geométricos estão
em `scripts/tree_study_geometry.py`; não são comandos implementados na engine.

Fonte `examples/native-tree-study.scene.json`, salva pelo MCP e conferida byte a
byte com a cópia no root isolado. Build3077e97f88c54625, cena50/replay54. Seed
20261026. Cena3.462.731bytes:8 assets,28 instâncias,10543 vértices/10068 triângulos
armazenados,31224 triângulos desenhados. As2100 folhas têm11v/12t cada. Quatro
variantes de conjunto,25 ossos/até2 influências,216 chaves por clipe; grupos de
folhas compartilham juntas para respeitar32 ossos. Tronco/madeira estáticos.

Capturas em `artifacts/tree-study/artifacts/`:

- `before.png` e `after.png`: mesma câmera, luz, viewport1200×1100 e tick60.
- `back.png`: lado oposto; `structure.png`: madeira sem folhagem, removida em um
  lote MCP reversível; `leaf-detail.png` e `bark-detail.png`: aproximações nativas.
- `tree.gif`:48 frames distintos/4000ms/2366931bytes; `report.json`: operações,
  contagens, vínculos de ramificação, seed, câmera/luz, checks e recusa do budget.

Não há imagem gerada para embelezar o resultado nem pós-processamento dos PNGs.
Pillow somente quantiza/codifica a sequência animada. Capturas de frente,
costas, estrutura e detalhes foram inspecionadas visualmente.

### O que precisei corrigir como autor

- Primeiro aumentei a quantidade de folhas mantendo ramos superiores grossos:
 1152 folhas não corrigiram uma estrutura visualmente pesada e uma copa vazia.
  Passei a derivar espessura do pai, reduzir a espessura nas pontas e terminar
  eixos principais dentro da folhagem. A engine não causou essas proporções.
- O frame radial anterior escolhia um eixo auxiliar pela direção da curva,
  sujeito a saltos de orientação. No helper desta rodada transporto o radial
  projetando o anterior no plano da nova tangente; isso pertence ao script,
  não é um novo sweep nativo ou garantia para curvas degeneradas arbitrárias.
- Eu colocava pecíolos no parâmetro linear, mas a curva quadrática da rameta
  usa0,9t+0,1t². Corrigi o cálculo para a mesma curva. Este erro de vínculo
  mostra o valor de anexação por parâmetro/superfície, em vez de posições soltas.
- A variação de casca com alto contraste por triângulo parecia camuflagem.
  Reduzi o contraste; cor por pedaço de malha não substituiu um material.
- Reduzi manualmente a folha de22 para12 triângulos e aumentei a densidade de
  1200 para2100 folhas, passando de32424 para31224 triângulos desenhados.
  É uma escolha de detalhe para esta vista,
  sem LOD automático ou garantia de qualidade no close. Uma orientação invertida
  nas duas faces da base da folha também foi corrigida antes da entrega.

### Dificuldades observadas e próximos incrementos candidatos

| Fricção | Evidência desta rodada | Workaround/custo | Melhoria genérica a avaliar |
|---|---|---|---|
| Galhos exigem modelagem matemática externa | Béziers, raio e frames calculados nos helpers;45 edições mesh publicam o resultado completo | Regenerar arrays fora da engine; pontos de controle não ficam editáveis na cena | Sweep de perfil por curva com raio variável e correspondência material, alinhado ao loft/perfis do outro chat; não criar um formato especial de árvore |
| Junções não são uma superfície contínua | Em `bark-detail.png`, bifurcações revelam tubos sobrepostos e mudanças abruptas | Introduzir ramo dentro do pai; ainda sem solda, continuidade ou retopologia | Conexão de contornos/weld e diagnósticos de superfície; não prometer boolean/remesh como parte do sweep |
| Variação de material consome instâncias |3 assets/instâncias de madeira para variar apenas a cor; folhas não têm frente/verso distintos | Divisão por triângulos duplica vértices entre assets e prejudica futura suavização nas fronteiras | Material por região/face e atributo de cor, com proveniência definida na regeneração |
| Normais flat e iluminação limitada | Casca em facetas; close das folhas parece recorte; copa não projeta sombra em si | Mais tesselação seria cara e não criaria transmissão/textura/sombras | Normais/vincos primeiro; materiais/UV e iluminação em incrementos coordenados com a frente gráfica |
| Detalhe e densidade disputam budget |31224t atuais; lote de2 cópias de conjuntos iria a33240t e foi recusado com `Mesh instances exceed32768 rendered triangles` | Simplificação manual para12t/folha; recusa preservou inspect/cena/execução integralmente | Inspeção prévia de custos por componente, LOD e agrupamento; não ampliar constantes sem rever custos |
| Vento é editado como chaves/índices |4 clipes, grupos de folhas compartilham juntas; madeira não responde ao vento | Tabelas de pesos e216 chaves por variante produzidas no script | Pesos por domínio/seleção, hierarquia de autoria e sinal de vento explícito; preservar executor genérico de rig |
| Repetição e oclusão dificultam julgamento | Ramos e conjuntos ainda mostram padrões no detalhe e vazios no centro da copa |4 variantes/seed e vistas fixas; inspecionar a estrutura separadamente | Scatter por regras/seleção e organização por subobjetos com picking, sem posições opacas em arrays |

Prioridade derivada do exercício: modelagem por curvas/perfis e conexões para
reduzir os cálculos manuais; controle de normais/materiais para que o detalhe
possa aparecer sem aumentar densidade indiscriminadamente. As duas frentes
compartilham o asset nativo atual. Isso é fila de decisões pequenas a especificar,
não autorização implícita de um novo stack gráfico/vegetação ou contrato já entregue.

### Julgamento artístico atual

Silhueta e escala relativa das folhas avançaram; as vistas opostas mostram uma
árvore mais irregular. Ainda reprovo o alvo de alta qualidade/realismo: junções
e raízes duras, casca sem estrutura de superfície convincente, folhas poligonais
uniformes sem nervuras, ausência de transmissão/sombra e repetição de rametas.
O close piora com a simplificação usada para preencher a copa. O vento move
folhas, mas não comunica flexibilidade da árvore inteira. Contagem maior de
folhas e checks aprovados não resolvem esses problemas. EAF-80 segue In Progress.

## Validação e revisão posterior

Transporte MCP real: undo/redo exato da escultura; recusa atômica por budget de
desenho; poses distintas; save/load restaura a pose; pixels vivos e do replay
coincidem byte a byte; após240ticks o loop retorna exatamente à pose inicial.
48 capturas distintas e duração do GIF conferidas. Comparação de imagens usa o
mesmo build; não é benchmark nem prova de fidelidade fotográfica.

Frequência/tipos: nenhum código de runtime alterado, listas autorais finitas;
seed e relações explícitas apenas no script/relatório. Alocações/cópias: fonte
3,46MB e snapshots de inspeção são custos deliberados de autoria; até31MiB de
pixels paletizados da prévia mais conversões/overhead, fora do tick. Repetição:
cálculos de curva e skinning atuais mantidos; variantes reduzidas a quatro para
compartilhar assets/clipes. GPU/I/O: capturas explícitas e paths estáveis, nenhum
novo build. Sem nova thread/lock/cache/dependência da engine, sem mudança de ABI
ou contrato de invalidação. Testes Rust/Clippy do mesmo núcleo não repetidos;
execução integral do novo script/cena verifica esta unidade de conteúdo.

## Segunda rodada — forma, aparência e junção nativa

O alvo permanece a mesma árvore. `native-tree-study-before.scene.json` conserva
como fixture o conteúdo17fb9ed; o script captura essa fonte com a mesma câmera e
luz usadas no conteúdo novo. A luz agora inclui sombra; esta comparação de
conteúdo não isola causalmente cada recurso do renderer. O relatório corrente
é `artifacts/tree-study/artifacts/report.json`, build75a671e2f5f40d99,55/59.

O núcleo ganhou recursos genéricos compartilhados por autoria direta e plantas:
shading flat/smooth por índices, sombra direcional opt-in, material procedural
com coordenadas3D de repouso e preview_mesh_junction. O último retorna uma
Geometry com fronteiras, não uma cena ou tipo especial de árvore. Todo o
conteúdo usa os mesmos assets/instâncias/rig/clipes/undo/save/replay existentes.
Sweeps, organização e distribuição continuam cálculos do autor no script;
não são um editor de curvas nativo já entregue.

### Decisões de forma e superfície

- Base: removidos tubos em espinhos. Colo enterrado e raízes largas são partes
  da mesma superfície; amplitudes/ângulos diferentes evitam estrela regular.
  Ainda há transições largas e lisas demais no detalhe, sem alegar raízes reais.
- Primeira forquilha: recorte do tronco t0.24..0.42 e pescoço inferior reautorado.
  A curva original do primeiro ramo desde t0.30 é retida por split Bézier;
  seus seis pontos de inserção mantêm erro matemático≤9.42e-16m. Isso não é
  igualdade de toda superfície f32 antiga nem transferência de IDs/escultura.
  A sonda tangente original22.8815° permanece recusada por separação insuficiente.
- Patch nativo32amostras/refino2, blend0.06/raio0.29,687v/1280t. Montagem dos
  três tubos compartilha os96 índices de borda exatos, sem nearest-weld. Com
  tronco/líder/primeiro ramo:1839v/3674t,1componente fechado orientado/χ2.
  Outros encaixes ainda são superfícies sobrepostas; não afirmar árvore inteira
  conectada ou autointerseções globalmente resolvidas.
- Realocação: microgalhos collineares usam2anéis em vez3, liberando450v para o
  miolo mais refinado. Uma receita blend0.08/raio0.29 foi recusada corretamente;
  não mover colares nem relaxar limites para fazê-la passar.
- Folhas:26v/32t substituem11v/12t, com8estações no contorno, curvatura e leve
  assimetria.1650folhas iniciais foram seguidas de dois conjuntos no novo líder
  central; total1782. Ramos secundários alternam azimutes, rametas têm extensão
  vertical maior; são novas escolhas de copa, distintas dos pontos preservados
  pela operação de junção. Não há promessa de morfologia botânica exata.
- Material: receitas muito contrastadas pareciam arame/rede ou pele de réptil;
  ruído anisotrópico liso parecia plástico. Interpolação de coordenadas entre
  bordas criou cintos no tronco, apesar da conexão geométrica. Foi retirada;
  todo o componente conectado usa um campo contínuo no repouso do objeto.
  Grão perfeitamente alinhado a cada ramo exige outro mapa de domínio; não
  atribuir essa garantia à receita atual. O canal genérico continua disponível.

### Contagens e verificação

7assets/29instâncias,10810v/15526t armazenados,64070t desenhados; madeira3871v
abaixo4096. Quatro variantes de folhagem com19ossos/162chaves cada, até2pesos por
vértice. Fonte5.192.880bytes, salva pelo MCP e conferida byte a byte. Não é uma
migração automática do trabalho artístico anterior: o script reautora a cena.

O teto de desenho foi revisto explicitamente32768→65536 em55/59 após avaliar
custos; armazenamento/4096v por asset/32instâncias/16assets mantidos. Ver
mesh-draw-budget.md. Três conjuntos adicionais manteriam32instâncias, mas
excederiam o teto: o teste exige a mensagem específica de65536t e
confere `inspect` integralmente inalterado. A cena publicada contém64070t;
as três cópias acrescentariam6336t, total70406t.

Capturas nativas: before/after1200×1100, back, structure, root-detail,
fork-detail, bark-detail e leaf-detail.48PNGs840×770 distintos geram GIF4s de
2.482.138bytes; Pillow somente codifica a sequência, sem retocar a árvore.
Query pura, topologia/arestas/orientação/conexão da primeira montagem, pontos
protegidos, escultura/undo/redo, rejeição de budget, posições animadas,
save/load/loop e pixels vivos/replay foram verificados. Núcleo completo:
732Rust/66cenários MCP, resultados e limites em validation.md.

### Julgamento e dificuldades que permanecem

O resultado melhorou a superfície e o contorno, mas ainda reprovo alta qualidade/
realismo. A copa continua mostrando agrupamentos e regiões vazias; folhas não
possuem nervuras/espessura/bordas convincentes, casca tem padrão procedural
reconhecível, raízes e outros encaixes ainda denunciam a construção. Não chamar
esse resultado de fotorrealista nem confundir os testes passados com aceite.

A animação move apenas folhagem. Animar madeira separadamente descolaria os
conjuntos, pois instâncias ainda não têm vínculo de transformação com ossos de
outra malha. Isso pede hierarquia/attachments explícitos, não mais chaves sem
contrato. Rest-space material não equivale a UV/PBR nem preserva qualquer textura
em uma junção nova sem correspondência. O orçamento ainda obriga escolher
contorno/densidade; mais constantes não substituem LOD/instancing/organização.

Prioridade derivada: autoria de curvas e conexões editáveis com diagnóstico,
controle de domínios/materiais e detalhes da folha, distribuição observável e
hierarquia de movimento. São recursos genéricos sobre o mesmo asset; nenhuma
biblioteca exclusiva de árvores foi colocada no runtime.

### Checklist estático antes/depois desta unidade de conteúdo

Frequência/tipos: aritmética de autoria Python + query Rust limitada; nada novo
no tick. Alocações: listas finitas para10810v e48frames; Geometry/rig/clipes do
runtime continuam Arc. Repetição: sem subdivisão arbitrária global; refino só na
junção e tesselação realocada de segmentos collineares. Limites: contagens antes
de put_asset/instâncias, validação nativa e recusa integral acima do teto. GPU/I/O:
mesmo renderer/binário corrente, capturas explícitas e paths estáveis, sem
janela. Sincronização: sem thread/lock/dependência nova. Invalidação: setters,
undo/load/replay existentes. Correção: script inteiro e prova topológica local,
mais regressão do núcleo integrado. Custos mantidos: expansão/skinning/upload
CPU por instância, materiais por fragmento, sombreamento extra e construção
síncrona do patch; sem alegar ganho de FPS ou suporte multiplataforma testado.

## Integração de um ramo editável — EAF-87/EAF-80

O segundo ramo primário agora é uma planta curva vinculada,258v/512t, sem
substituir a primeira junção por geometria desconectada. A receita/fonte está
na cena e suporta reautoria nativa de seções. Árvore8assets/30instâncias,
10948v/15802t armazenados,64346t de desenho; fonte5.358.019bytes em56/60.
`demo_tree_authoring.py --compare-branch --frames 48` reproduz o baseline e
o teste. Mesmo núcleo e3d1731ed06f70ab; novo GIF48frames/4s/2.481.820bytes.

Forma A-B-A, fonte/material/canal,6inserções≤1.174e-7m, undo/redo e persistência
passam; primeira montagem1839v/3674t e demais assets/instâncias permanecem
exatos. Close A/B revela quinas de interpolação linear e o encaixe sobreposto.
Realismo permanece reprovado. Fricções novas, custos e gates completos em
[tree-plant-integration.md](tree-plant-integration.md). Próxima hipótese é curva
suave com correspondência de domínio e anexação dos secundários; aumentar
folhagem isoladamente não resolve essas dependências.

## Suavização sobre a árvore salva — EAF-87/EAF-80

`demo_tree_smoothing.py --frames 48 --segments-y 4` edita incrementalmente a
cena atual pelo MCP. A prova desta unidade carregou a fonte56 de48aa4d4, sem
reconstrução; source/reference originais lineares mantidos exatamente em todos
os12casos Linear/PCHIP × A/B × Y2/4/6. Outros assets (wood inteiro incluído),
instâncias, rig/clipes da folhagem e sua pose no tick de edição permanecem iguais.
O ramo permanece estático; não há novo vínculo de movimento com a folhagem.

Forma final A/PCHIP[4,4,4]:482v960t, árvore8assets/30instâncias11172v16250t
armazenados64794t desenhados,742t livres. A opção6 custa65242t e restam294t;
o close de A justificou4 nesta vista. A/B e resoluções comparados com câmera,
luz,tick e estações iguais: Linear denso mantém cotovelo, PCHIP esparso ainda
tem segmentos, PCHIP com resolução suficiente arredonda. Não confundir C1 com
malha automaticamente suave nem com orientação/frame/attachments preservados.

Fonte5.443.474B em57/61, build6758029880829ea2, GIF48frames/4s/2.481.539B.
Seis centros≤6.59e-8m; pure preview, repetição, undo/redo, erro atômico por draw,
save/load meshes+plants, PNG vivo/replay e ciclo exatos. Capturas correntes
foram inspecionadas. Realismo continua aberto: casca/folhas/distribuição e os
outros encaixes não melhoram só com esse modo. Checklist, tabela de custo e
prova histórica em [tree-plant-integration.md](tree-plant-integration.md).

## Vínculo de um secundário e folhagem — EAF-89

Agora uma instância pode seguir outra ou um osso nomeado, com TRS local explícito.
Na árvore salva, o secundário de crown-06 e suas rametas foram extraídos por
índices conferidos e vinculados ao socket de um rig no galho editável. A copa
segue o secundário com seu playback próprio. Um teste move a inserção e observa
ambos; outro compara movimento do osso1.5graus com a animação foliar antiga.
Madeira principal permanece estática, e os outros secundários ainda não foram
convertidos. Não é hierarquia completa da árvore ou física de vento.

Mesma geometria64794t,9assets/31instâncias; extração71v114t, erro de repouso
2.98e-8m e copa idêntica no tick0. Primeirofork preservado por índices. Fonte
5.663.332B/GIF2.481.054B48frames/4s em58/62, buildb0aba8aa96ad42cb.
748Rust/69MCP passaram; contrato/custos/checklist e dificuldades em
[mesh-attachments.md](mesh-attachments.md). Setters de rig/clipe rebaseiam e
podem arredondar canal material; restauração explícita do canal original foi
workaround medido, não correção nativa. Extração ainda exigiu helper e origem
local manual; ferramentas de organização/seleção continuam necessárias.

Capturas mostram vínculo funcionando e os mesmos problemas de aparência da
árvore: casca em rede, folhas planas/repetição e encaixes sobrepostos. A feature
passou nos gates técnicos sem concluir o alvo artístico ou todas as pendências.

## EAF-90 — organização por ilhas nativas

Foi removida uma fricção geral concreta: separar partes soltas exigia reproduzir
receita/arrays/índices e construir manualmente os subsets. Agora a consulta MCP
expõe ilhas, bounds/contagens e referência de revisão; Move/Copy com preview
preserva espaço local/material/canais/rig/clipes. Não existe tipo especial de
árvore. Costuras duplicadas continuam desconectadas; seleção por proximidade
ou solda implícita não foi adicionada.

O novo `demo_tree_islands.py` carrega a árvore de f810466, seleciona as seis ilhas
menores do secundário pela inspeção e as move pelo comando nativo para
secondary-twigs. Instância explícita tree-secondary-twigs segue o secundário em
TRS local identidade. Compara canais/faces da união, poses de todos os outros
componentes, planta/source/reference e madeira/primeirofork; captura GPU exata
antes/depois em repouso e movimento. Rigged Copy é exercitado na mesma cena.
Reusa12casos de retesselação/edição, save/load/replay/loop em48frames distintos.

Final10assets/32instâncias, mesmos11172v16250t armazenados64794t desenhados;
fonte5.664.534B/GIF2.481.054B48frames/4s. A separação deixa35v66t no secundário e
36v48t nas rametas. O novo probe de draw conserva32instâncias e substitui uma
copa por madeira pesada somente no pedido rejeitado. Fonte salva por MCP e
copiada para examples/native-tree-study.scene.json; paths correntes.

A dificuldade seguinte permanece de autoria geral: controle de detalhe por
região, materiais/superfície, distribuição observável e organização em escala.
Todos os slots estão ocupados, mas isso não autoriza orientar a engine só por
uma árvore. Folhas e casca continuam insuficientes no close. Extração é avanço
de editabilidade/preservação; não muda a aparência nem conclui o alvo artístico.
Contrato/checklist em mesh-islands.md, evidências em validation.md.

## EAF-91 — folhas e casca como teste de ferramentas de superfície

UI explicitamente adiada por Guilherme. Camada geral de traços afunilados no
material: tinta e bump independentes do ruído, plano de repouso escolhido,
limite32segmentos e fade subpixel. Serve também costuras/fissuras/painéis;
sem shader botânico. fit_material_coordinates por asset/ilha foi acompanhado
por copy_material_coordinates com correspondência explícita de topologia.

O uso encontrou um erro de autoria antes de fechar: as variantes antigas não
tinham canal local; AABB de posições já rotacionadas não orientava nervuras.
A folha-fonte correta não bastava como evidência. Correção nativa: fit do domínio
na fonte e Copy validado pelas sequências de triângulos/índices para cada ilha.
66ilhas por variante,4variantes, domínio exato em todas, sem nearest/PCA ou
arrays da receita recompostos pelo script. Incompatíveis falham integralmente.
Também foi extraída uma folha da copa pelo MCP para captura isolada com rig/
clipe, câmera da pose e undo, além da folha-fonte. Dessa vez o teste observa o
conteúdo que realmente está instanciado na árvore.

Nova receita31segmentos de nervuras, refinada após os traços retos iniciais
parecerem diagrama. Casca usa ridged anisotrópico +32segmentos de fissuras,
reduzindo a aparência de rede. Ablação sem detail tanto na folha quanto casca,
mesma câmera/luz/tick60 antes/depois, poses0/60 e geometria/rig/clipes/instâncias
exatos. Fonte/reference da planta e geometria do primeirofork preservadas;
aparência muda explicitamente.12casos Linear/PCHIP×A/B×Y2/4/6, persistência,
replay e48frames distintos continuam passando no conteúdo de superfície.

Fricções restantes: limite de31/32traços, transferência exige mesma topologia,
orientação do canal precisa autoria, fibras não seguem todos os frames dos
ramos. Bump sem espessura/silhueta/sombra, shader difuso sem reflexão/roughness
ou PBR completo; folhas poligonais ainda visíveis no close.742triângulos livres
não permitem acrescentar sequer1triângulo a cada uma das1782folhas sem
realocação/representação por escala. A árvore inteira não ganha realismo só
porque nervuras aparecem no close; alvo artístico continua aberto. Próximos
recursos devem continuar gerais, sem subsistema dedicado de árvores.

Contrato/custos e revisões antes/depois em surface-details.md/performance.md.
Capturas surface-before/after-leaf,crown-leaf,bark,fork,whole, ablações e relatórios
surface-report.json/report.json em artifacts/tree-study/artifacts. Fonte corrente
mantém10assets/32instâncias11172v16250t armazenados64794t de desenho.

## EAF-94 — resposta diferente à luz, mesma árvore

Baseline db3b3de carregado, sem gerar nova árvore. Material.reflectance é geral,
MCP primeiro: rugosidade e F0 mudam o reflexo da luz direta, não a geometria.
Folhas0.58 e casca0.92, ambasF0=0.035. Não pintar brilho na cor difusa: ele
acompanha a câmera/luz e desaparece na sombra, com ambiente legado preservado.
Câmeras/luz/ticks fixos no antes/depois inteiro/casca/forquilha; folha extraída
da copa real com rig/clipe, vista e luz alinhadas ao reflexo para diagnóstico.
Controles0.3/0.9/F0zero sob a MESMA montagem, inteiramente desfeita depois.

A inspeção revelou resposta muito plástica e serrilhado das nervuras no controle
0.3. A receita final tem brilho discreto; casca continua fosca. Isso acrescenta
controle útil mas não estabelece realismo: contorno/espessura da folha, casca
estriada demais e domínio de fibras nas curvas permanecem gargalos. Um escalar
por asset não distingue regiões da superfície; máscara/domínio de rugosidade,
representação de detalhe mais densa e antialiasing especular devem ser avaliados
como recursos gerais. Não esconder esses limites com troca de iluminação entre
antes/depois ou apresentar controle brilhante como folha pronta.

scripts/demo_tree_reflectance.py --frames48 --input
artifacts/tree-study/examples/reflectance-input.scene.json reproduz o ensaio;
reflectance-report.json e report.json no root corrente registram build e hashes.
Todos os assets/canais/rigs/clipes/instâncias/poses0/60 conservados, só materiais
alterados. Fonte geométrica/reference/definition da planta exatas, metadado de
material atualiza a fonte sem rebase. Mesmo11172v16250t armazenados/64794t desenho,
10assets/32instâncias. Loop48frames/4s, save/load,12casos de retesselação/edição e
PNG vivo/replay exatos. A frente native-plants trata continuidade geométrica em
paralelo, sem misturá-la a este ensaio de superfície. EAF-80 continua aberto.

## Continuidade e materiais juntos — integração EAF-80/EAF-85

Nova entrada435742d: árvore com rugosidade/nervuras/casca preservadas, somente
primeira montagem de wood corrigida. Cotovelo46.10°→10.62° no encontro e curva
menos abrupta.9materiais, restantes assets/plantas/source/reference/instâncias/
poses0/60 exatos; madeira restante e domínio comum conservados. Capturas
before/after-bark/fork/whole em câmera/luz/tick60 fixos; antes byte-exato às
capturas após o material anterior. O encaixe oposto continua sobreposto e a
parte superior do pescoço ainda mostra facetas; não chamar tudo de seamless.

10assets/32instâncias11300v16506t armazenados/65050t desenho486livres. Sem ampliar
budget ou mudar shader/material/formatos. Loop48frames/4s e12casos de edição/
retesselação/replay/prefixo/undo/persistência passam. Houve correção do próprio
ensaio: mudar câmera reseta execução; capturar após reaplicar câmera anulava o
step60. Agora o tick é verificado e o prefixo0 testado separadamente. Essa
observabilidade é parte do exercício, não ganho artístico presumido.

O prefixo geométrico é conferido antes de substituir região; usar entrada anterior
à correção para reproduzir demo_tree_integrated.py. Checklist, custos e evidências
completos em tree-geometry-material-integration.md/validation.md. Fonte central
é a cena integrada com materiais novos; EAF-80 e aceite de realismo permanecem
abertos. Próximos ensaios continuam visando ferramentas gerais, não gerador de
árvores. Trajetória/seções em outra frente seguem experimento próprio, sem serem
presumidas entregues por esta integração.

## Contornos reais e escultura protegida — EAF-85/EAF-80

A árvore exercita inspect_mesh_boundaries/remove_faces/bridge_boundary_loops.
A hipótese radial e as pontes superiores foram reprovadas; topologia/costuras
não bastaram para reconhecer dobras internas. A guarda genérica agora compara
faces adjacentes envolvendo a ponte e recusa>=90°. Somente o pescoço central
foi promovido, com ponte32×10 seguida por duas passagens do Smooth nativo,
preservando cada ponto antigo/material/source/rig/attachment. Medidas completas,
limites e reprodução em mesh-bridges.md/validation.md. A diferença visual é
incremental; não é uma entrega de árvore realista nem de todas as junções.

Dificuldades observadas: recorte/fase/alças ainda exigem autoria explícita;
o preview devolve o documento completo; máscaras de componentes nos pincéis
faltam, obrigando seis strokes com centros pequenos para proteger os pontos
antigos. Distribuição angular/erro de tesselação e domínio de casca transportado
continuam necessários. Esses são aprimoramentos genéricos das ferramentas.

## Prova isolada de união volumétrica — EAF-85

A ponte por pares falhava no encontro agudo. O novo kernel usa três volumes
tubulares guiados para fornecer uma forma alvo e conecta a superfície extraída
aos contornos reais32/24/32. O recorte nativo congelado produz3995v7986t, sem
bordas abertas, preservando exatamente fonte, canais, rig, clipe e filho animado
no socket. As vistas front/profile/back sem textura/sombras revelam um colar
remanescente:1.15° na costura fonte,59.74° na conexão interna,53.60° no miolo.
Portanto a base resolve uma hipótese topológica/geometricamente válida, ainda
não o acabamento natural ou integração na árvore inteira.

Outra dificuldade encontrada: domínio posição_local+offset nos pontos novos
não corresponde necessariamente ao domínio autorado da planta/contexto.
Preservar os valores antigos não garante continuidade da casca. Próximo ensaio
deve tratar geometria e transporte de domínio como gates distintos; suavização
restrita a pontos novos e primeira guarda fixa, sem mascarar defeitos por luz.
Contrato e custos em implicit-junctions.md; fonte central desta branch intacta.

## Refinamento finito da conexão e domínio

Na mesma prova isolada,4passos locais em352pontos extraídos reduzem pico
59.7362°→43.4687°, conexão interna41.7063°, média5.01344°→4.96777°. Fonte e
primeira guarda88pontos exatas, nenhuma geometria adicional. Deslocamento máximo
25mm; volume fechado−0.021496%, não conservação local.8passos pioram a média e
são recusados. O erro final em relação ao campo sobe a6.9267mm porque a faixa
foi deliberadamente refinada fora da iso-superfície; o erro5.833e-8m da extração
não deve ser apresentado como erro da superfície final.

Domínio: diferença de coordenadas entre ponto fonte e guarda adjacente
3.59369→0, correção interna limitada64passos com resíduo pós-f326.05e-5unidades.
A malha compartilhada já era contínua por interpolação; o problema era a
variação forte na faixa8mm. Render authored mostra menos linhas abruptas, mas
ainda estiramento/direção artificial dos veios. Harmônico de valores não impõe
derivadas ou orientação física: registrar esse limite, não chamar de casca
resolvida. Próxima melhoria genérica precisa observar frames/gradientes do
domínio e distorção, além de coordenadas. Não inferir fibras por nearest-source.

Opções default0 preservam base; nenhuma mudança da árvore salva/tetos/UI nesta
unidade. Comparação0/1/2/4, erro8e poses/socket/persistência/replay nativos em
mesh_junction_union. Vistas neutras separam avanço geométrico de aparência;
composição com planta/contexto e densidade seguem gates próprios em paralelo.



## Fricção de composição — árvore local66

O corte se refere a uma SEÇÃO pelo ID030, não à altura aproximada.044 deixaria o
socket sem região de apoio. Um preview singular de planta não prevê a retirada
do contexto do asset wood: duplicaria o tronco provisoriamente e ultrapassaria
o orçamento embora o documento final coubesse. A melhoria genérica foi
preview_scene_edits compartilhando o mesmo staging/guardas da publicação; lote
extrai, remove temporário e instala receita com fonte própria. Não foi necessário
duplicar limites totais nem criar uma nova árvore. Asset maior e amostragem mais
fina reduziram facetas; não resolveram material/gradientes ou todos os galhos.

O domínio da fonte da planta usa outro referencial que o tronco. Copiar posições
como novo domínio cria variação abrupta na guarda. A correção finita do kernel
mantém domínio retido e copia fonte na guarda, porém ainda estica os veios.
Amostragem fina64passos dá resíduo maior que coarse; reportar, sem fingir solução
harmônica exata. Nova escultura no miolo exige camada com âncoras estáveis; rebase
é recusado. Evidências/custos e reprodução em plant-assemblies.md/validation.md.

## Primeiro secundário conectado por receita68

O ressalto no pescoço esquerdo foi identificado como raiz do secundário0; era uma
ilha fechada atravessando o pai. `demo_tree_graft.py` agora reproduz uma união
nativa3portas com fontes estáticas integrais, extraçãoMove atômica e recipe
regenerável. Fontes anteriores/contexto/ref/ossos/clipes permanecem; demais
assets/instâncias/poses0/60/240 exatos. Comparações nas mesmas vistas com e sem
casca em artifacts/tree-graft. Contrato/limites em context-grafts.md.
A tampa atravessada desapareceu, mas uma transição de curvatura/iluminação em
anel e distorção do domínio de casca permanecem visíveis. Não é árvore inteira
realista/conectada; wood ainda tem176ilhas e o secundário06 animado tem outro
contrato de pesos a resolver. Não confundir conectividade/topologia com aceite
artístico ou continuidade analítica perfeita.

## Colar suavizado de forma regenerável69

O modificador `collar_surface` permite redistribuir curvatura entre pai e miolo,
com seleção por portas/anéis de arestas e camadas externas fixas. Na mesma vista,
o anel ficou atenuado; fonte anterior/ref/rig/canais permanecem, posições dentro
da região mudam por intenção. Repro demo_tree_collar.py e imagens tree-collar,
sem novas faces/triângulos. Não resolve casca/demais junções/folhas: esses seguem
gates da árvore realista. A frente complementar de meshes segue o diagnóstico
de estiramento do domínio da casca; a cena continua sob integração de plantas.
