# ADR 0001 — Laboratório 3D com comandos compartilhados

## EAF-137 — diagnóstico de área congelado fora da ordenação do tick

Scene87/replay95/JSON mantidos. Snapshot denso por Entity elimina BTree por
candidato; seleção classifica todos e ordena só elegíveis para executar. Recibo
possui todos rows/resultados congelados; cache ordenado é materializado sob
leitura/serde, sem consultar estado futuro. [Contrato e custos](area-selection-runtime.md).

## EAF-136 — agendas por fonte e remoções locais

Scene87/replay95 estende Pool<T> a periódicos64/áreas16por slot, índice de alvo
para dissipação e IdSet local512por transação. Refresh/compactação preservam
ordem densa; cleanup de áreas só repete após tick/morte nova nos mesmos pontos.
[Contrato, prova e gargalo de diagnóstico de área](scheduled-source-pools.md).

## EAF-135 — entregas e contextos por fonte

Scene87/replay94. Projéteis usam pool denso encapsulado/indexado por slot, quota64
por fonte e teto real slots×64. Origem retida quota32por slot, leaseu16, lookup
por referência, alocação sob demanda e captura pre-reservada65/64; coletar valida
lease+referência e mantém buffers para reuso. Sweep reutiliza snapshot espacial e
preserva narrowphase/TOI/ID. [Contrato, custos e limites](source-delivery-pools.md).

## EAF-134 — índice espacial do snapshot de combate

BVH por mediana reconstruída em buffers reutilizados após população/posição e
antes da física, com teste AABB conservador e box/wedge exato inalterado. Visada
de habilidades usa candidatos/centros do mesmo snapshot; booleano não depende da
ordem da árvore. Sem formato novo ou dependência; fallback linear explícito para
World sem índice. [Tradeoffs, checklist e medição](combat-broadphase.md).

## EAF-133 — admissão e execução por ator

Scene87/replay93. Quota16por ator e global slots×16; scratch FIFO agrupa pedidos
em O(atores+entradas), sem alterar cancel/fases/InstanceId. Control input contínuo
atualiza estado e conta atores tocados; gravado mantém log exato. IDs consultam
placement/ordem de instância cacheados com geração validada. Somente ampliar teto
foi rejeitado por manter filtro quadrático. [Contrato e carga](frame-admission.md).

## EAF-132 — política de execução independente do documento

Scene87/replay92. Recorded padrão mantém replay completo limitado; continuous
usa a mesma Simulation local/offline e descarta entradas consumidas, com recibos
limitados e anel de diagnóstico128/90ticks. Ampliar o log apenas adiaria exaustão;
descartar um prefixo sem checkpoint produziria replay impossível de reproduzir.
Sequências64bits não dependem de Vec::len; política muda somente antes de entradas,
reset/load/projeto a conservam com escopo novo. Replay completo e exportação são
recusados em contínuo antes da escrita. [Contrato e custos](continuous-runtime.md).

## EAF-131 — componentes de atores dimensionados pelo mundo

Scene87/replay91 migram slots para16bits e componentes de vida/recursos/casts/
prontidão/percepção para armazenamento denso pelo número de atores/reservas.
Conjuntos de atores usam16palavras64bits; máscaras de grants permanecem16bits.
Ordenação por InstanceId é cacheada no lifecycle; transações preservam writes
locais e buffs conservam índice dono/definição. Até1000Players/8NPCs,1024slots
internos e4096entidades; sem ECS/dependência/rede nova e sem requisito remoto
para single player. Ampliação apenas de constantes foi rejeitada porque mantém
cópias globais, truncamentos e shifts inválidos. [Decisão, custos e gates](scalable-actor-runtime.md).

## EAF-128 — consolidação estável na main

A união Scene86/replay90 combina ferramentas nativas/render/editor e runtime de controles/transações/navegação terrestre, conservando os gates históricos por campo. Trabalho estável no checkout principal; Modly e Nanites permanecem isolados. O editor herdou pesquisa por ancestralidade: integração squash filtrada exclui seus244arquivos exclusivos e sua ancestralidade da main, conservando refs originais para reprodução. Biblioteca de projetos é preferência separada do documento/simulação, com limites e persistência própria; não é catálogo global de assets. [Escopo, fontes e evidências](main-consolidation.md). Registros seguintes são históricos por incremento.

## EAF-127 — conectores terrestres limitados

Cena85/replay89 estende controller+walk-only com local_connections, reutilizando Dijkstra/KCC sem outro planejador. Três entradas e quatro saídas próximas (XYZ/índice), direto e cache8pares no stack;2400passos virtuais/plano+600para lançamento. Truncamento explícito em ground_planning, falha ground_local_budget; menor custo entre sondados, sem ótimo global quando limitado. Ordenação por distância é heurística de seleção, não limite inferior físico em rampa; custo acumulado não negativo permite evitar saída já mais cara. Alternativa de sondar128conectores×600passos por replan recusada; busca incremental/global completa permanece alternativa futura com estado/custo de invalidação próprios. Retry60 também cobre pistas móveis após falha. Cast lock mantém conector/prazo; âncora só é aceita após probe físico. [Contrato](ground-local-connections.md).


## EAF-126 — rede terrestre com admissão pelo motor

Reutiliza mobility/rede/Dijkstra, controller+walk-only, cena84/replay88 após união editor83/87. Links inclinados são sondados pelo mesmo KCC sobre mundo fixo compartilhado; Arc de relatórios por perfil de caixa/velocidade/Config, templates aquecem antes do spawn. Alternativa de outro planejador ou fórmula de ângulo isolada rejeitada por divergência de clearance/autostep. Limite600passos virtuais e2checagens de contato,3ticks de transição após autostep sem renovar em voo, chegada com contato recuperado; custos estáticos dirigidos e revalidação somente da ligação lançada. Contato do último passo do controlador alimenta confirmação física; cast lock preserva trecho/prazo sem congelar percepção. Dinâmicos não são previstos/reservados pela consulta; solver/timeout preservam execução. Conectores locais para este motor e transições aéreas precisam contrato próprio. Nenhuma mudança em renderer/Light/Vertex/Globals, transporte/ABI/ECS ou alegação de escala. [Contrato e limites](ground-navigation.md).



## União atual do editor — Scene83/replay87

Conjunto de c1e3b13 (writes locais/buffs indexados), d049fdf (controlador terrestre)
e c5f2419/7f98b33 (transformação de ilhas e composição), sobre a UI5e06a61 e a
escultura167d8de. Layout3/manifesto1 preservados. Gates históricos82/controlador,
77/controles,76/ambiente e75/cunhas mantidos. Os estados de controles, motores
terrestres e poses de meshes participam juntos de snapshot/replay; cada Player
alimenta sua intenção XZ a partir do próprio handle/perfil/lock. Contribuições
mantêm índice derivado encapsulado e JSON em lista. transform_islands usa a
revisão de seleção existente, congela domínio material e não cria formato novo.
Single player offline permanece obrigatório, sem rede/escala1000 implícita.
A união preserva as ferramentas visuais atuais; UI específica de controlador,
ilhas, transferência de jogador, Sweep por trecho e chanfros ainda está pendente.
[Pontas, validação e limitações](editor-integration.md).

## União anterior do editor — Scene81/replay85

codex/editor usa Scene81/replay85, layout3/manifesto1. Múltiplos Player e
controles escopados (gate histórico77) convivem com ambiente76, cunha75 e
malhas/plantas. Snapshot conserva mesh_selection+controls; verificação de replay
exige controles E poses de meshes além dos estados anteriores. Sweep.spans
escalar/lista e ChamferBoxes geram Geometry comum sem formato persistido novo.
Single player local/offline permanece disponível; não há rede/autenticação
remota ou escala1000 entregue. Controles visuais de transferência de jogador,
Sweep por trecho e chanfros continuam pendentes.
[Pontas, conflitos e evidências](editor-integration.md).


## União anterior do editor — Scene80/replay84

A saída conjunta de codex/editor é Scene80/replay84, layout3/manifesto1.
Inclui Wedge, ambiente procedural, intensidade direta e normais avaliadas junto
às capacidades anteriores. Os números75/79 e76/80 abaixo são históricos das
frentes isoladas; os gates de entrada75cunha/76ambiente continuam preservados.
Light::validate é comum à cena e ao work_light; defaults antigos omitem os
novos campos, e o override do editor não altera a autoria. RenderBox.geometry
convive com Vertex64/material neutro nas cunhas e Globals192 do ambiente.
Normais usam a rotina compartilhada do render, sem novo canal ou cache.
Integração de núcleo/MCP não conclui os controles de UI para estes recursos.
[Protocolo e evidências](editor-integration.md).


## Integração do editor — EAF-104

`codex/editor` passa a reunir as frentes commitadas de autoria funcional, imagem→3D, malhas, plantas e pesquisa gráfica. Cena73/replay77 unificam os contratos mantendo guardas históricas por capacidade; estado de editor continua separado. Pesquisa permanece explícita/isolada dos caminhos funcionais. Pontas, recorrência, checklist e fronteiras em [editor-integration.md](editor-integration.md). A integração de código não conclui a UI de cada operação; toda capacidade MCP existente ou futura exige ferramenta visual equivalente.

## EAF-104 — refinamento visual do editor

Tokens e componentes locais em `editor_design_ui` padronizam espaçamento,
cores, abas, campos e ações no egui existente. Popover de iluminação usa estilo
de formulário para manter trilhos/valores visíveis; propriedades têm seções e
adaptação à largura. Alternativa de novo framework/dependência foi descartada:
o problema está na composição da interface, e a renderização/comandos atuais
já permitem corrigir a experiência sem mudar contratos. Layout 3, Scene 75,
Replay 79 e manifesto 1 permanecem iguais. Critérios, auditoria antes/depois,
tradeoffs e reprodução em [editor-ui.md](editor-ui.md).

## EAF-110 — escultura direta com confirmação por gesto

Primitivas e picking triangular pertencem a `mesh_tools`, acessíveis pelo Service
no dispatcher comum. Seleção de instância fica na sessão e não no documento.
A UI mantém um draft com o mesmo kernel Sculpt e apresenta esse documento no
RenderFrame; soltar publica um EditMesh com as amostras registradas. Isso evita
reset/histórico por amostra e permite cancelar sem transação compensatória.
Alternativa de execução visual própria foi descartada para conservar equivalência
com MCP. Custos são cópias limitadas da geometria tocada, raycast e normais
síncronos durante entrada; sem novo backend ou dependência. Base estática apenas,
sem bake/detach/colisão implícitos; contratos e checklist em [editor-sculpt.md](editor-sculpt.md).
Scene75/replay79/manifesto1 mantidos. Layout3 acrescenta configuração global
SculptSettings e work_light opcional; LoadEditor migra layouts1/2 com padrões.
EditEditor::Sculpt/Lighting valida a cópia e avança apenas revisão de editor,
sem histórico autoral/reset. Força Volume corresponde diretamente ao Stroke,
sem fator oculto; Mover converte proporção do arraste para metros do kernel.
Light::validate é compartilhado entre Scene e editor. O compositor substitui
Light apenas no RenderFrame da viewport, reutilizando o passe de sombras e
preservando luz, histórico e replay da cena. Configuração de trabalho foi
escolhida em lugar de alterar cenas existentes. Sombra padrão custa um passe
GPU existente; não há promessa de FPS, shader/backend ou dependência nova.

## EAF-104 — editor e layout de sessão

Direção autorizada: editor de ferramenta com menus suspensos, workspaces e áreas configuráveis inspirado na organização de Blender/Unreal. Widgets/texto/entrada por egui0.33.3 sobre winit0.30/wgpu27 existentes, sem novo backend ou processo web. Árvore de áreas própria e tipada mantém persistência/validação/MCP independentes da biblioteca visual; alternativa de docking externo foi rejeitada neste recorte para evitar seu formato como contrato público. Custos: manter interações pequenas de split/movimento e limite explícito de layouts, em troca de controle do contrato.

`editor::State` pertence ao Service: versão1, até8workspaces/16áreas/32abas/profundidade8, exatamente uma viewport por workspace; revisão de editor independente da cena. `EditEditor` valida cópia limitada antes de aplicar; `InspectEditor`, `SaveEditor`, `LoadEditor` e `CaptureEditor` compartilham dispatcher/MCP. Layout não entra em Scene/Replay nem altera simulação. Viewport compõe textura GPU reutilizada com controles no mesmo dispositivo; captura UI é offscreen e separada da captura da cena. Headless não inicializa widgets/GPU sem pedido de captura UI. Primeiro incremento mantém uma cena ativa e não integra autoria gráfica de outras branches. Contrato/limites/checklist em [editor.md](editor.md).

## Painéis com vãos — construção reutilizável

MeshEdit::Panel compila dimensões/retângulos locais numa Geometry comum via
mesh_panel::build. Usa staging/preview/MCP/undo existentes; índices de grade
compartilhados, superfícies externas e jambas sem faces internas redundantes.
Escolha frente a receitas por imagem e CSG geral, limites e custos em
[mesh-panels.md](mesh-panels.md). Como Grid, fonte não é novo documento
persistente e substituir não transfere escultura/rig. Cena67/replay71 e
renderer mantidos. Sem inferência de imagem, certificado de casa ou colisão.

## Análise anterior à construção por imagem — EAF-100

Guia versionado com o projeto em
[ignis-image-to-3d](../.agents/skills/ignis-image-to-3d/SKILL.md), exemplo
[casa](references/image-to-3d/elven-house-analysis.md). A falha do ensaio inicial
foi modelar volumes sem leitura estrutural suficiente; correção começa pelo
inventário de peças/relações e regras dos motivos, não por trocar a métrica.

Alternativas: prompt curto é fácil de distribuir, mas não define entregas;
parser/solver semântico no núcleo permitiria verificações de campos/medidas,
mas exige contrato e não comprova a interpretação. Recorte atual: skill e
registro documental com evidências, critérios por etapa e lacunas explícitas.
Não é novo formato Scene/ABI nem escolhe fornecedor de IA. Imagem única continua
ambígua; guia não garante qualidade uniforme entre modelos.

AGENTS aponta para o guia nesta árvore. Sua distribuição atual é pelo repositório;
não foi adicionado recurso MCP nem carregamento automático em clientes externos.
Uma futura entrega via protocolo precisará descoberta/versionamento explícitos.
Contagens, silhuetas e correspondências podem fundamentar verificações futuras;
o comparador RGB/SSIM atual não as implementa. Sem mudança no runtime/renderer.

## Avaliador visual determinístico — EAF-99

CompareImages é consulta de Service pelo Command/MCP comum: decodificação PNG
limitada, RGB8 e SSIM11×11 de luminância, máscara/regiões explícitas e mapa de
erro opcional. Núcleo CPU puro separado de arquivos/render; não muda cena67,
replay71, renderer, tick, câmera ou geometria. Sem alinhamento/segmentação
implícitos ou score de verdade3D. Comparação permanece síncrona no serviço
existente; fontes/alternativas/custos e política de cor em
[image-similarity.md](image-similarity.md). Métricas locais antes de um solver
de câmera/geometria; modelos perceptuais externos continuam alternativa futura.

## Integração da base de autoria — EAF-98

A branch `codex/image-to-3d` reúne as duas frentes de autoria e a main local
sem introduzir nova técnica de reconstrução. Cena67 e replay71 preservam
campos e guardas de ambas as frentes; validação do replay exige tanto meshes
quanto geradores. Fontes, tradeoffs de integração, referência da casa e
evidências: [image-to-3d-integration.md](image-to-3d-integration.md).

## EAF-122 — personagem independente da origem de controle

Primeiro incremento autorizado da direção MMO: múltiplos Player recebem input
por vínculo exclusivo de instância, com handle escopado/serial monotônico.
Escolhido em vez de broadcast global ou alias em cada input para impedir que
release antigo neutralize vínculo novo. SetInput legado permanece somente quando
não é ambíguo; bind/release invalidam a posse nativa anterior. Perfis, movimento,
cast e passos são individuais; morte mantém vínculo e neutraliza input explícito,
inclusive retorno no mesmo tick. Eventos de controle evitam copiar todos os inputs
por tick; log/tick reservam releases. Frame reutilizado por slot e mapa ordenado
por handle oferecem preparação O(P) e consulta de input O(log P), com alocação
somente no vínculo, sem novo scheduler/lock/renderer. Inspeção/exclusividade por
ator ainda percorrem população pequena sob comando; storage/máscaras/Plan globais
seguem transitórios. Cena77/replay81, coordenados após ambiente76/80 sem incorporá-lo.
[Contrato e gates](actor-controls.md). MCP confiável não é autenticação remota;
capacidade atual não é comprovação da meta de1.000 no mesmo combate.

## EAF-124 — motor terrestre cinemático opcional

Entity.character_controller escolhe Rapier KinematicCharacterController0.30.1 apenas para ator terrestre, cena82/replay86. Motor dinâmico permanece padrão; intenção XZ separada de velocidade física porque set_linvel não dirige um cinemático de posição. Configuração compilada por nascimento, estado por identidade e scratch20colisões compartilhado. Gravidade acumulada/limitada, degrau/snap/inclinação e push opt-in; cadáver dinâmico, revive reinicia controlador. Grounded por contato Rapier é política distinta de apoio integral no spawn/planejador; não relaxa walk_support. Mobilidade aérea e planejamento inclinado ficam em contratos separados. Consultas usam AABBs pré-física incluindo corpos recém-instalados, sem passo escondido. Vertex/Globals/RenderBox inalterados. [Contrato e alternativas](character-controller.md). Requisito MMO permanece alvo arquitetural, sem comprovação de escala deste motor.


## EAF-115 — primitiva convexa antes do controlador

Entity.geometry tipada, Box padrão omitido e Wedge alinhada em±X/±Z, cena75/replay79. Compartilhar seis vertices e volume convexo entre collider, visual e consultas evita tratar o vazio da caixa envolvente como parede. Cunhas só fixas/visuais; atores/volumes/surfaces planas preservados. Fast paths/tolerâncias das caixas continuam iguais; apoio plano recusa inclinado e o planejador planar projeta apenas o volume na faixa vertical relevante. RenderBox inclui Geometry Copy; cache mantém36v/slot com12v degenerados limitados por cunha, sem novos shader/Vertex/Globals. Alternativas (rotação geral, malha/heightfield e migração de controlador) adiadas por contratos maiores. QueryGeometry read-only e autoria MCP precedem controles visuais. [Contrato/tradeoffs](geometry.md). Rampas/degraus caminháveis exigem próxima decisão de motor e planejamento.

## EAF-117 — premissa de multiplayer MMO

Guilherme definiu em28/10/2026 a meta futura de1.000 jogadores simultâneos na
mesma área e no mesmo combate. Essa premissa orienta identidade, autoridade,
storage, transações, tempo, replicação e cliente; separar os participantes em
instâncias/regiões não satisfaz a meta. Infraestrutura de hospedagem fica fora
deste estudo. [Auditoria, alternativas e proposta](multiplayer-system-study.md)
distinguem requisitos confirmados, recomendações e decisões pendentes. Servidor
autoritativo, planos locais e visão replicada por componente são recomendações;
transporte/ECS/budgets e políticas físicas não foram escolhidos implicitamente.
O estudo não implementa multiplayer nem comprova escala. Limites históricos
abaixo descrevem os respectivos incrementos, não substituem essa direção.

## EAF-111 — colocação sobre apoio sem mudar locomoção

Consulta de colocação vertical compartilhada entre MCP e geradores opt-in, cena66/replay70. Maior topo fixo abaixo do centro XZ dentro de max_drop; empate por ID, sem procurar outro andar quando o primeiro não couber. Apoio modular da pegada inteira e overlap AABB com corpos presentes reutilizam contratos atuais; spawn ainda revalida admissão completa. Preferido a um raycast pontual que ignoraria a largura do corpo, a ajuste silencioso do legado ou a navmesh/controlador novo. A região limita origens da busca; altura final depende do piso e template. Custos esparsos limitados sem cache espacial especulativo; consulta não reserva posição nem prova caminho/mobilidade. [Contrato](spawn-support.md).

## EAF-108 — composição ponderada e escolha pendente

Template único ou lista ponderada por gerador/onda, cena65/replay69. Tabelas cumulativas e bindings compilados; fontes herdadas referenciam base. Stream template seed XOR0x9e3779b9 separado da posição. Escolher uma vez por nascimento pendente e manter em falhas evita fallback enviesado para quem cabe; pode bloquear se o escolhido nunca for viável. Tradeoffs e contrato em [weighted-population.md](weighted-population.md).

## EAF-105 — posições regionais com admissão física

Fonte regional min/max exclusiva de points, cena64/replay68. Sortear centros XYZ por3draws do LCG por tentativa, sem lista temporária, projeção/fallback ou alteração de capacidades. Reusar transação spawn; falha consome apenas tentativa/seed. Pontos legados mantêm consumo e ordem. Recibos guardam posição tentada; contrato/tradeoffs em [spawn-regions.md](spawn-regions.md).

## EAF-96 — permissão espacial independente

Activation referencia um trigger existente; modos once/while_occupied usam ocupação pós-física anterior com referências ainda presentes. Permissão manual e condição espacial se combinam antes de atualizar uma única suspensão efetiva; sem callbacks de percepção ou geometria duplicada. Scene63/replay67. Tradeoffs/fases em [generator-activation.md](generator-activation.md).

## EAF-95 — controle do produtor em execução

Pausa/ativação/cancelamento usam a fila de população, com ordem e limites compartilhados. Pausa congela produção/avanço de ondas e desloca prazos na retomada; indivíduos e retirada de cadáveres continuam no relógio do mundo. Cancelamento terminal sem despawn implícito, cena62/replay66. Alternativas, recibos e persistência em [generator-control.md](generator-control.md).

## EAF-93 — sequência finita derivada

Ondas opcionais nos geradores, cena61/replay65, até16 por gerador. Quotas contam nascimentos; conclusão por morte/ausência ou ausência completa. Confirmação histórica definitiva, no máximo uma transição com evento por tick; ressuscitados não reabrem ondas nem são expulsos. Bindings de todos os templates/alvos compilados na construção, estado separado Copy e membros por Reference/onda/life_index. Reusar spawn/remove e replay derivado. Tradeoffs, limites e contrato em [population-waves.md](population-waves.md).

## EAF-92 — geradores derivados e identidade

Cena60/replay64. Política de população separada das transações de nascimento/retirada. Geradores compilam template/alvo e mantêm referências de indivíduos; executam depois da fila externa e antes dos sensores. Semente/cadência/budget finitos e recibos próprios regenerados no replay, evitando encher o log de inputs com efeitos derivados. Retirada de cadáver opt-in; ressurreição pode exceder a meta sem expulsão. Competição em ordem autoral, sem garantia de equidade. RenderFrame preservado. Contrato e tradeoffs em [population-generators.md](population-generators.md).

## Conexões regeneráveis de plantas72

[plant-connections.md](plant-connections.md) define o recorte nativo: até8 pares
disjuntos de plantas por IDs de instâncias/seções, filho rígido preso a osso do
pai e iso novo com pesos harmônicos de dois ossos. Ambas fontes continuam no
documento; somente o asset do pai recebe a composição. O Frame omite o desenho
do corpo do filho consumido, conservando sua hierarquia para folhas/rametas.
Regeneração/validação ocorrem no staging; Scene72/replay76 locais, sem novo
shader/teto/dependência. Não é grafo arbitrário de árvores ou skin independente.

## Amostragem observável de plantas e estudo de conexão

`inspect_plant_sampling` expõe a amostragem real de Definition.sample pelo mesmo
dispatcher; desenho/recorte não precisam implementar outro avaliador de curvas.
A consulta de união exporta o material_plan transitório existente, incluindo os
pares fonte→guarda, para atribuição explícita de canais. Nenhum formato novo.

[live-plant-junction-study.md](live-plant-junction-study.md) separa a prova da
junção/skin da futura montagem persistida por referências. O estudo visual ainda
usa bake isolado; não é uma conexão nativa regenerável entre plantas entregue.

## Espaço de trabalho de plantas — direção proposta

Plantas devem ter coleção e apresentação cotada dentro da Ignis, independentes
do agente autor. [plant-workspace.md](plant-workspace.md) distingue o primeiro
visualizador da cena, edição visual posterior e catálogo entre arquivos. O mesmo
documento paramétrico alimentará prancheta e geometria; SVGs exportados não serão
uma segunda fonte de autoria. Toolkit e persistência do catálogo ainda não foram
escolhidos; esta direção não representa editor implementado ou migração de formato.

## Projeção opcional de detalhes por domínio material (EAF-80/EAF-85)

Scene70/replay74 expõem Detail.triplanar no dispatcher/MCP normal. Pesos unsigned
suavizados por vértice são atributo próprio64B, calculados no repouso/material e
compartilhados por GeometryArc no Frame persistente. A projeção planar segue como
padrão; a árvore mantida não recebe ativação automática. Trocar projeção não altera
posições, escultura, skin ou fonte de coordenadas. Bump mistura covetores com
pesos congelados, aproximação explícita distinta da derivada da tinta misturada.
Cache por snapshot MCP ainda é reconstruído. Mistura de projeções não determina
fibra botânica nem conclui casca realista; contrato/custos em triplanar-details.md.

## Referenciais e acabamento materiais regeneráveis (EAF-80/EAF-85)

Scene70/replay74 introduzem opções afins explícitas na avaliação das fontes de
plantas/enxertos. A cópia avaliada recebe a transformação material; fonte canônica,
correspondência de escultura e rig ficam preservados. A transformação geométrica
não altera silenciosamente o domínio material. A referência pode ser singular;
o finisher regular opt-in recusa suporte de fronteira degenerado.

`material_finish` usa a geometria final de cada estágio e a proveniência exata
transitória produzida pela união: prefixos retidos, intervalo iso e pares
fonte→guarda. Substitui a difusão material legada naquele estágio, não a soma a
ela. Sem a opção, mantém CopyGuard e saída legados. Índices avaliados não são
persistidos nem reencontrados por proximidade. Ordem de enxertos continua autoria;
não é um unwrap global ou um modelo físico de fibras. Contratos, alternativas e
checklist em material-continuity-study.md e mesh-material-finish.md.

## Diagnóstico do domínio material (EAF-80/EAF-85)

Consultas puras por triângulo no dispatcher compartilhado medem o Jacobiano do
canal de material sobre a superfície local. Coordenadas3D/projeções2D e escala
explícita separam compressão do domínio de compressão por projeção. Não avaliam
warp, iluminação ou pose e não inferem orientação de fibra. Metadados de autoria,
Scene/replay e renderer não mudam. Contrato/custos em mesh-material-domain.md.

## Junção geométrica com fronteiras explícitas (EAF-85)

preview_mesh_junction é consulta pura do dispatcher/MCP: três colares com
frames/IDs, refino/envelopes e campo de fusão opcional retornam Geometry,
boundaries e diagnósticos. Publicação usa put_asset; a query não entra no
histórico nem introduz um documento especial de árvore. O miolo é novo e não
possui correspondência implícita com escultura/rig anteriores. Domínios agudos
ou sem separação recusam, em vez de ajustar o objeto silenciosamente. Material
pode usar coordenadas novas explícitas depois da composição por índices.
Contrato e tradeoffs em mesh-organic.md; fonte55/replay59 permanecem porque
não há formato persistente novo para essa operação.

## Receitas de superfície e coordenadas de material (EAF-80)

Asset recebe material neutro por padrão e Geometry um canal3D opcional por
vértice; fallback é posição de repouso. Receita genérica limitada substitui o
workaround de dividir assets para variar apenas cor. Texturas UV/PBR completas
adiadas por exigirem recursos/tangentes/transferências adicionais. Posição
material anterior ao skinning conserva o campo durante animação; bump perturba
apenas a normal. Material é metadado de plantas, canal é edição que estabelece
fonte canônica; regeneração interpola por correspondência explícita. Cena54/
replay58. Tabela17slots evita receita replicada por vértice; Vertex cresce16bytes,
shader tem ruído opt-in. Custos/limitações em procedural-materials.md.

## Sombra direcional explícita (EAF-80)

`Light.shadow` opt-in define centro/extensão/resolução/bias no MCP. Um mapa
ortográfico finito recebe caixas e malhas opacas presentes, exclui guias, e
atenua somente a luz direcional com PCF3×3. Ambiente continua escalar. Sem
dependência de câmera, cascatas, GI ou interpretação de identidade de gameplay.
Cena53/replay57, fallback legado desativado; Vertex permanece inalterado.
Buffers de geometria existentes são reutilizados; mapa é redesenhado por frame
com custo adicional explícito. Contrato/limitações em directional-shadows.md.

## Sombreado de superfície (EAF-80)

`Asset.shading` flat/smooth é metadado autoral, com setter no dispatcher/MCP.
Smooth deriva normais por área da superfície já deformada/transformada, usando
índices compartilhados como regra de continuidade. Não armazena normais autorais
que poderiam ficar obsoletas após escultura/regeneração; não solda superfícies.
Seams/vincos por índice continuam responsabilidade da autoria neste recorte.
Fonte de planta muda apenas esse metadado sem rebase; kernel de subdivisão e
preservação conservam o modo. Scene52/replay56; contratos e custos em
[mesh-shading.md](mesh-shading.md). Materiais/sombras permanecem capacidades distintas.

## Topologia e refinamento linear (EAF-79)

Contrato em [mesh-topology.md](mesh-topology.md). Faces trianguladas e incidência
derivada são mantidas neste recorte. Referências de vértices/arestas/faces expiram
por escopo/revisão; não fixar identidade permanente aos índices. Alternativa de
half-edge/polígonos autorais fica para operações que a justifiquem, com migração
explícita. Subdivisão linear é distinta de Loop/Catmull-Clark e não suaviza a base.

Um kernel produz midpoints compartilhados por aresta e vizinhos conformantes,
com preflight de contagens, preservação de ossos/clipes e interpolação de pesos
sem poda. Seleção é estado de sessão, com dispatcher/MCP e sem efeito no gameplay.
Erros conservam histórico/execução; undo existente reverte meshes livres. Não há
stack genérica de modificadores nem ferramentas manuais de componentes entregues.

Plantas usam `subdivision_levels` na amostragem declarada e o mesmo kernel, em vez
de anexar uma malha incompatível à referência de grade antiga. Fonte canônica só
rebasa em edição direta explícita; proveniência cobre todos os novos vértices e
correspondências descendem nos filhos baricêntricos. Local vinculado é recusado
até haver representação autoral própria; detach permanece explícito. Cena50 valida
níveis da definição e referência; replay54 mantém resultado/fonte autocontidos.
As garantias de detalhe, pesos, bind e animação continuam separadas: LBS das novas
amostras não garante igualdade com a antiga superfície animada nem contatos.

## Nascimento por templates (EAF-70)

Contrato aprovado: [spawn-templates.md](spawn-templates.md), cena47/replay51. Templates separados da presença; pool estável de8 NPCs compartilhado por até16 templates. Escolha: instalar uma configuração por nascimento e inicializar todos os componentes do slot, preservando arrays e sobreviventes. Alternativas descartadas neste recorte: reconstruir Simulation (perde estado dos sobreviventes), reservar slots por template (fragmenta o limite), migrar para ECS/containers dinâmicos agora (amplia a mudança estrutural sem necessidade para o contrato). Templates/catálogo/rede são imutáveis durante execução; RuntimeWorld substitui somente a definição do slot que recebe um novo indivíduo.

O custo de clonar configuração e preparar rotas/links ocorre por nascimento; o custo de ordenar até9 índices por InstanceId é limitado e evita que um slot reciclado altere prioridades lógicas. Bases são compiladas por template e copiadas somente para o dono novo. Inspeção/render/picking usam o mundo em execução, saves/undo usam o documento. Fila de população sequencial antes dos sensores, validação antes da identidade, referências preservadas em históricos e contextos de origem. Capacidade atual mantida; planos transacionais locais e índice de atributos continuam pendentes antes de ampliar escala.


Direção estrutural de população aprovada em [population-runtime-study.md](population-runtime-study.md). EAF-60/61 migraram identidade e vínculos; EAF-63 separou presença/configuração e integrou contexto compartilhado de origem. EAF-67 publica retirada transacional de NPCs, fila/recibos e replay50: [population.md](population.md). Cena46 preservada. Nascimento/templates e instalação dinâmica de novos componentes continuam pendentes; não implicar ECS ou escala ampliada.

Status: adotado para EAF-1. Núcleo em Rust; primeira validação em macOS. Esta decisão cobre um recorte dos marcos M01/M02/M03/M04/M06/M15/M16, sem declarar esses marcos completos.

## Organização

| Módulo | Responsabilidade |
|---|---|
| `model` | Documento versionado, entidades/componentes básicos, validação e receita procedural |
| `engine` | Comandos, revisão, histórico, edição versus partida e replay |
| `navigation` | Planejamento XZ com folga até um objetivo conhecido |
| `perception` | Sensores, memória e decisão limitada dos NPCs |
| `runtime_world` / `origins` | Definições e placements da execução; retenção limitada da autoria de entregas |
| `population` | Pedidos numéricos de retirada, limites e recibos; aplicação no começo do tick |
| `physics` | Adaptador Rapier, passos fixos, posições e contatos identificados por IDs de domínio |
| `render` | Adaptador wgpu, geometria de caixas, iluminação e captura GPU |
| `service` | Operações de projeto, limites de arquivos, escrita atômica e captura |
| `mcp` | Protocolo JSON-RPC/MCP stdio; descoberta de contrato e resultados estruturados |
| `window` | Janela winit, apresentação e mapeamento de teclas para comandos |

As entidades são estruturas serializáveis com transformação, visual e corpo opcional. Não criamos ainda um ECS genérico, scheduler de sistemas, ABI de plugins ou linguagem de gameplay. Essas escolhas deverão responder a casos reais e preservar o contrato de projeto.

## Regra absoluta de paridade

Toda capacidade controlável da engine deve existir no MCP antes ou simultaneamente na UI. Os controles de jogo da janela chamam `Service::execute`, usando o mesmo enum `Command` que o MCP. O schema é derivado desse enum, para que não exista um catálogo MCP mantido manualmente em separado.

`project_landmarks` acrescenta diagnóstico puro de projeção3D→pixels, com câmera
candidata e correspondências explícitas opcionais. Reutiliza Projection da
seleção e a matriz comum ao picking/render; não altera Scene/replay, câmera,
histórico ou runtime. Clipping e cobertura são explícitos; erro2D não é
aceite3D ou visibilidade. Contrato e decisões em [landmark-projection.md](landmark-projection.md).

P → play/pause; R → reset; WASD/setas → set_input; Esc/fechamento → exit. O gerenciador de janelas cuida de tamanho/foco da janela hospedeira; foco perdido libera somente a entrada ainda pertencente ao teclado pelo mesmo contrato. Capturas têm resolução explícita independente da janela. Futuras ferramentas de edição devem seguir esse dispatcher.

MCP é plano de controle, nunca relógio por frame. `step` avança ticks internamente; `play` habilita o relógio local. `pause` é necessário antes de passos explícitos. O relógio acumula tempo e limita catch-up para não entrar em espiral de atraso. Não há multiplayer nesta etapa.

## Consistência e identidade

O documento `Scene` é independente das posições físicas. Saves e undo operam no documento. Edições são construídas e validadas antes do commit; uma falha preserva documento, revisão, histórico e partida. Alterações bem-sucedidas incrementam a revisão e reconstroem a simulação pausada.

IDs de domínio são strings estáveis; handles Rapier não escapam do adaptador. Corpos são inseridos em ordem de ID para evitar dependência da ordem de arquivo. `expected_revision` permite rejeitar edições obsoletas. Captura e snapshot observam um estado consistente sob o mesmo lock.

## Proceduralidade

`RoomRecipe` é uma primeira receita, não o framework procedural completo. Dimensões e quantidade definem a distribuição em grade; a semente controla alturas. Um PRNG inteiro versionado torna a receita reproduzível. Os IDs gerados são estáveis. Edições manuais ficam no documento junto da receita original como proveniência; regenerar substitui a cena inteira e pode ser desfeito. Regeneração parcial com preservação de overrides é trabalho futuro.

## Bibliotecas e portabilidade

O experimento inicial usa wgpu 27, winit 0.30 e Rapier 0.30, com resoluções exatas no Cargo.lock. São bibliotecas de infraestrutura sob módulos próprios. Não estamos construindo uma extensão de outra engine.

wgpu escolhe o backend disponível; nenhuma API Metal específica está espalhada pelo domínio. winit trata janela/entradas. Rapier está configurado com `enhanced-determinism`; isso não torna o sistema completo automaticamente determinístico. A evidência atual é reprodução de posições no mesmo build/ambiente.

WebAssembly é apenas candidato à futura fronteira de extensões. A arquitetura não promete que toda linguagem ou recurso estará disponível em toda plataforma. A discussão de builds ocorre separadamente e não amplia o aceite de EAF-1.

## Observação e reprodução

Cada snapshot informa revisão, passo, documento, posições, contatos, entradas e histórico. Eventos identificam os dois objetos e o passo. Capturas incluem pixels e câmera/adapter/passo/revisão. Replay registra cena inicial, entradas por tick, versão e identificação do build; uma reexecução isolada compara posições exatas ao final. Não compara automaticamente áudio, imagens ou todos os estados internos da física.

O verificador de integração injeta uma parede ausente, confirma fuga, desfaz a mudança e repete a trajetória. Isso comprova a capacidade de detectar e corrigir uma falha conhecida via MCP. Não comprova raciocínio autônomo generalizado nem cobertura de todos os bugs de gameplay.

## Limites e próximos passos

A janela atual é um visualizador jogável, não um editor visual completo. A perseguição de NPCs em piso plano foi adicionada em EAF-2. Habilidades, combate, multiplayer, importação de assets e um sistema universal de plugins continuam fora deste recorte.

Referências primárias: [wgpu](https://github.com/gfx-rs/wgpu), [winit](https://docs.rs/winit/0.30.13/winit/), [Rapier](https://rapier.rs/docs/), [MCP stdio](https://modelcontextprotocol.io/specification/2025-11-25/basic/transports).

## ADR 0002 — Perseguição em superfície plana (EAF-2)

A configuração opcional `NpcConfig` vive na entidade; as decisões e waypoints vivem na simulação. A cena é o documento editável e a referência para reset/replay. A física mantém os corpos dinâmicos, colisões, gravidade e CCD; navegação define apenas velocidade XZ. O controle vertical permanece físico.

Cada NPC escolhe explicitamente um jogador e uma caixa de piso por IDs. O topo dessa caixa define uma única altura navegável. Obstáculos físicos que interceptam a altura do NPC são projetados em XZ e expandidos pelas meias dimensões do corpo mais 6 cm. O centro do NPC também fica dentro do piso com essa folga. Caixas apenas visuais não bloqueiam.

Primeiro tenta-se uma trajetória direta por teste de segmento contra essas caixas expandidas. Caso bloqueada, uma grade de 0,5 m e A* com quatro vizinhos encontra uma rota; os segmentos entre células também são verificados, incluindo obstáculos mais finos que uma célula. O início conecta-se a uma célula próxima com segmento livre. O destino é uma região ao redor do jogador, dentro da distância de parada: a busca aceita qualquer célula livre nessa região com linha de visão desobstruída. Uma aproximação direta também termina nessa região, sem tentar ocupar o centro do jogador. O NPC segue o waypoint mais distante visível, sem cortar cantos. Velocidade é limitada pela distância restante por passo para não ultrapassar o waypoint.

A rota é recalculada a cada 12 passos (0,2 s), e sua visibilidade é reavaliada a cada passo contra as posições físicas atuais, inclusive caixas dinâmicas. Os NPCs leem o mesmo snapshot anterior à física; empates da busca e ordem das decisões são estáveis. Chegar exige distância de parada, posição livre para o NPC e linha de visão até o alvo. A visibilidade usa caixas físicas sem expansão; a locomoção continua usando caixas expandidas pelo volume do NPC. Assim, um jogador encostado na parede pode ser alcançado sem liberar trajetórias através dela. A distância efetiva considera os raios circunscritos de ambas as caixas; há tolerância de 2 cm na detecção de chegada. O raio informado pode ser ampliado para evitar empurrar o jogador.

Limites impostos: 8 NPCs por cena, 128 corpos físicos quando houver NPCs, piso até 64x64 m, no máximo 16.641 células por busca, dimensões XZ do NPC até 4x4 m, velocidade de 0,1 a 10 m/s, distância configurada de 0,2 a 5 m. São limites de custo, não uma promessa de desempenho máximo. A demonstração medida usa um NPC, uma barreira e sala de 16x14 m.

A busca é conservadora: uma passagem geometricamente possível pode não aparecer na grade; precisa existir uma posição livre na região de parada com visão do alvo. `no_clear_path` significa ausência de rota neste planejador, não prova matemática de impossibilidade no mundo. Múltiplos NPCs são obstáculos móveis uns para os outros; não existe ainda negociação de passagem, prevenção de deadlock ou coordenação de multidões. Obstáculos em movimento podem alcançar o NPC; a física protege colisões, sem promessa de previsão/evitação contínua.

Não suporta rampas, degraus, salto, empilhamento de pisos, navegação vertical ou navmesh. Alturas fora da tolerância de 30 cm são diagnosticadas. Esse recorte original não incluía percepção ou memória; EAF-3 adiciona a camada descrita abaixo. Combate permanece futuro.

## ADR 0003 — Conhecimento limitado e onisciência explícita

Status: direção adotada para M09; primeiro recorte visual implementado em EAF-3. Os contratos abaixo orientam a evolução completa; a seção de implementação delimita o que existe hoje. O laboratório histórico EAF-2 é preservado como onisciente por migração. O teste geométrico da navegação serve à chegada e não substitui o sensor.

### Contrato de conhecimento

O NPC padrão decidirá com base em observações e memória. Não receberá a posição atual de um jogador escondido apenas porque existe uma referência à entidade. Onisciência será uma política explícita, configurável para jogos ou personagens que precisem dela. A migração das cenas existentes deverá declarar a política para preservar intencionalmente o comportamento da demonstração; novos NPCs terão percepção limitada como padrão.

Separar quatro responsabilidades:

1. **Percepção:** produzir observações por visão, audição e outros sentidos autorizados pelo gameplay. Visão considera alcance, orientação/campo visual e oclusão. Audição considera eventos, alcance, intensidade e propagação/atenuação conforme o modelo suportado. Uma pista informa origem conhecida ou estimada, instante, precisão e confiança; não revela automaticamente identidade ou coordenadas exatas de quem a causou.
2. **Memória:** guardar última posição observada, instante, fonte e confiança, com expiração configurável. Direção ou velocidade estimadas devem derivar das observações anteriores. Perder visão interrompe a atualização visual; novas informações exigem outra observação válida.
3. **Decisão:** escolher perseguir, ir à última posição conhecida, investigar pistas, explorar caminhos plausíveis, desistir e retomar uma rotina. Previsões e hipóteses permanecem distinguíveis de fatos observados. Uma busca pode falhar mesmo quando o jogador está perto.
4. **Navegação:** executar o destino conhecido ou estimado escolhido pela decisão. No modo padrão, não atualizar esse destino consultando silenciosamente a posição real do alvo. Conhecimento do mapa e capacidade de encontrar caminhos são políticas separadas do conhecimento sobre o jogador.

Uma janela ou outro material deve ter regras explícitas de passagem/bloqueio de visão e som; aparência transparente, colisão física e oclusão sensorial não são equivalentes. Rastreamento mágico, marcações e comunicação entre aliados podem fornecer pistas adicionais, com fonte e alcance próprios, se o jogo os habilitar. Eventos sonoros da simulação devem funcionar em testes sem janela e sem depender da reprodução de áudio no dispositivo.

### Configuração e proceduralidade

“Inteligência” será representada por perfis componíveis, não por uma permissão implícita para conhecer o mundo inteiro. Alcance dos sentidos, tempo de reação, duração da memória, confiança mínima, persistência, raio/orçamento da busca e estratégia para escolher caminhos serão parâmetros separados. Um perfil simples pode procurar brevemente na última posição vista e desistir; outro pode estimar uma rota de fuga e investigar saídas sem saber qual foi usada.

Perfis poderão ser definidos manualmente ou gerados por receitas com parâmetros, restrições e semente. Escolhas probabilísticas de investigação precisam ser reproduzíveis no contrato de replay suportado. Maior persistência ou capacidade de planejamento não garante detectar ou reencontrar o jogador.

Toda configuração, seleção de política, emissão de pistas para teste e inspeção de percepção/memória/decisão deverá existir no MCP antes ou junto da interface visual. O diagnóstico deverá explicar o que foi percebido, o que foi lembrado, a hipótese atual e por que houve mudança de comportamento. A inspeção privilegiada do mundo pelo criador não deve alimentar inadvertidamente o comportamento de um NPC com conhecimento limitado.

### Próximo recorte recomendado e critérios

Começar com visão configurável, memória da última posição vista, investigação limitada, desistência e retomada ao detectar novamente o jogador; incluir a política onisciente explícita para comparação. Audição e estratégias mais elaboradas serão incrementos posteriores sobre o mesmo contrato de observações. Esse primeiro recorte foi implementado em EAF-3; EAF-5 acrescenta busca espacial limitada. Audição e planejamento semântico de rotas permanecem futuros.

Validar pelo MCP e por cenários reproduzíveis:

- Ao perder visão, seguir a última posição conhecida; deslocamentos ocultos do jogador não atualizam essa posição nem o caminho por acesso direto ao alvo.
- Com observações, mapa conhecido e semente iguais, movimentos ocultos sem novas pistas não alteram as decisões de busca. Colisões, sons ou outra informação efetivamente percebida contam como novas observações.
- Expirar memória/orçamento causa desistência; uma observação válida posterior permite retomar a perseguição.
- Perfis de busca distintos produzem estratégias distintas sem consultar a verdade oculta; pistas imprecisas preservam a incerteza e não se tornam rastreamento exato por referência à entidade.
- O modo onisciente continua rastreando o alvo oculto por escolha explícita do criador.
- Save/load da configuração, reset, undo/redo e replay preservam os contratos aplicáveis; replay compara também memória, pistas relevantes e decisões, além de posições.

### Implementação e tradeoffs de EAF-3

Pipeline por tick: snapshot físico → observação → memória/decisão → `NavGoal` → navegação → orientação por velocidade → passo físico. `decide` recebe somente a observação filtrada e o estado anterior de navegação; não recebe cena nem posições reais. A navegação recebe posição conhecida, raio de chegada e altura do alvo como metadado, sem consultar sua posição atual. Continua conhecendo a geometria do cenário e dos outros obstáculos móveis; não implementamos mapa aprendido/fog of war. O alvo é excluído das consultas de obstáculos para evitar influência de sua posição escondida. A física continua resolvendo contatos reais.

Foi escolhida uma FSM explícita, sem framework de behavior trees/utility AI: menor contrato e estados reproduzíveis para os comportamentos existentes, ao custo de transições codificadas em Rust e menor composição. A fronteira entre observação e decisão permite substituir essa política depois. Não é ainda o sistema genérico de decisão de M09.

Sensor: um segmento 3D entre olho e centro do alvo, alcance esférico e FOV horizontal. AABB próprio é suficiente para as caixas sem rotação atuais e permite `blocks_vision` independente de corpo físico. Custo linear por NPC nos possíveis oclusores, até os limites existentes de entidades/NPCs; sem índice espacial nem evidência de escala máxima. A alternativa de consultas Rapier reaproveitaria broad phase, mas exigiria representar também oclusores sem collider; esta escolha deve ser revista com malhas/rotações e escala. Um alvo parcialmente exposto pode não ser detectado se o centro estiver oculto. A orientação sensorial segue velocidade instantaneamente; a caixa visual não gira. Não há FOV vertical, múltiplos pontos de amostra, iluminação ou latência de reação. Audição posterior: ADR 0005.

Memória retém uma observação exata do sensor visual; confiança é recência linear, não estimador probabilístico. Sem prazo de memória, o peso permanece 1 até esquecer. A primeira investigação visita a região da última posição (raio 0,4 m), depois varre a visão até atingir um limite ativo. Esse raio cobre a discretização da grade; corpos grandes/perto de paredes ainda podem não alcançar a região. Após desistir, o NPC permanece parado com sensor ativo; patrulha/retorno à rotina ainda não existem. Perfis são parâmetros serializáveis geráveis via MCP; EAF-5 acrescenta geração espacial com semente. Receitas dedicadas de perfis, pistas adicionais e planejamento semântico de rotas continuam futuros.

Desde v5, a cena materializa a política: ausência em v3+ significa perceptiva; ausência em v2 é migrada para onisciente, preservando exemplos existentes. v3 preserva os dois prazos numéricos, sem limite de distância; v4 permite desativar tempos e combinar distância. Formatos antigos não aceitam as novas configurações. Replay v5 inclui estado de conhecimento, contagem de distância e busca espacial. `inspect_npcs` e `inspect` expõem memória, fonte, fase, motivo, recência, orientação e ticks pelo mesmo dispatcher. Configuração continua por `upsert`, com validação atômica, save/load e histórico. Nenhuma dependência nova foi adicionada.

### Persistência por tempo e distância

Os relógios existentes permanecem distintos: `memory_ticks` desde a última observação e `search_ticks` desde a chegada à região lembrada. Ambos são opcionais; `max_investigation_distance` limita o deslocamento horizontal efetivo durante investigação/busca. Pelo menos um limite deve estar ativo; encerra o primeiro atingido. Tempo usa ticks da simulação, respeitando pausa e replay. Distância exige uma subtração vetorial, norma e acumulação por NPC ativo; reutiliza o snapshot anterior e os corpos após a física, sem snapshot adicional ou serialização por tick.

A navegação limita a velocidade solicitada ao saldo. A contagem posterior à física incorpora empurrões e exclui queda vertical. Tolerância de 1 mm encerra resíduos que o atrito pode impedir de percorrer; o diagnóstico mantém o valor medido. Forças externas podem ultrapassar o limite em um passo, sem desfazer física. Expiração por distância limpa objetivo/rota e velocidade horizontal após o passo, registrando o tick resultante. Nova observação zera o episódio, replanejamento não; desistência retém a medição. Reset/edição reiniciam a simulação. Teleportes futuros exigirão uma marca de descontinuidade, ainda sem API neste recorte.

Tradeoff: somente distância não limita busca estacionária ou bloqueio; somente busca local não limita o trajeto antes da chegada. `memory_ticks` fornece o prazo global quando desejado. Configurar persistência não muda a estratégia; o perfil espacial opcional descrito abaixo acrescenta destinos de busca. Defaults continuam 600/180 ticks e distância desativada. Contrato completo em [commands.md](commands.md).

### EAF-5 — investigação espacial limitada

Direção escolhida para o próximo incremento: uma política opcional de pontos próximos à última observação, preservando varredura parada quando `area_search` estiver ausente. Alternativa mais sofisticada seria inferir rotas de fuga/portas a partir da geometria e observações de movimento, mas ampliaria o domínio do planejador e o contrato de hipóteses. O círculo determinístico é pequeno, testável e oferece persistência espacial sem onisciência; não equivale a compreensão semântica do mapa.

`AreaSearch` contém raio, número de pontos (1–8), tempo de varredura por ponto e semente. A semente fornece a orientação inicial módulo 360; os pontos são igualmente espaçados no plano XZ da memória. Um único ponto é gerado por transição, sem vetor de candidatos nem trigonometria adicional por tick estacionário. Estado inclui ponto atual, índice, contagem de descartados e instante da varredura, todos inspecionáveis/reproduzíveis. Não são observações e não substituem `memory`.

Após chegada à região lembrada, ou diagnóstico de região inicial inalcançável, começa o orçamento de busca e a fase `exploring`. Navegação continua responsável por encontrar rota. Ponto inalcançável é descartado; chegada inicia a varredura; ao terminar, tenta o próximo. Lista esgotada produz `search_points_exhausted`. Observação visual tem prioridade e cancela o episódio; limites de memória/tempo/distância continuam compartilhados por todos os pontos, sem renovação entre destinos. Só distância não limita espera por bloqueio físico; a configuração mantém ao menos um orçamento global obrigatório. Sem retorno à patrulha neste recorte.

Trocar um destino de memória/busca invalida imediatamente o plano anterior; destinos de perseguição visual mantêm a cadência de 12 ticks. Isso evita usar caminho antigo ao mudar de ponto. `decide` continua recebendo somente observação filtrada, política e estado anterior da navegação; não recebe a cena nem a posição oculta do jogador. Geometria conhecida e sensor de um ponto permanecem limitações do laboratório.

Cena v5 admite `area_search`; v1–v4 continuam legíveis, sem ativação implícita do perfil novo. Replay v5 compara o estado adicional e mantém restrição ao mesmo build. Configuração pelo `upsert` e diagnóstico por `inspect_npcs`/`inspect_navigation`, sem controles exclusivamente visuais. O checklist antes/depois consta em `performance.md`; nenhuma dependência nova ou profiling necessário.

## ADR 0004 — Representações de dados e desempenho

Status: diagnóstico do laboratório e proposta de evolução para discussão técnica. Separação documento/simulação e MCP como plano de controle já existem; pacote binário, novo layout de runtime e desacoplamento assíncrono abaixo ainda não estão implementados. Nenhum codec/dependência novo foi escolhido e nenhum benchmark de desempenho máximo foi realizado.

### Onde JSON existe hoje

JSON representa o documento de edição, replays persistidos, mensagens MCP e resultados de inspeção. `model` converte os documentos para structs/enums Rust; `Engine::advance` executa percepção, navegação e Rapier diretamente, sem JSON entre sistemas a cada tick. GPU já recebe buffers binários pelo adaptador wgpu. Paridade MCP significa acesso às mesmas capacidades, não trafegar cada entidade/frame pelo protocolo.

O contrato MCP exige JSON-RPC 2.0. Trocar esse envelope por um protocolo binário próprio quebraria interoperabilidade. O transporte de controle pode coexistir com referências a arquivos/payloads binários; a imagem atual usa PNG em base64 conforme o protocolo. Formato de autoria legível continua útil para diffs, revisão, migrations e ferramentas, mas não precisa ser o formato distribuído nem o layout usado durante o jogo.

Isso não torna o custo do JSON irrelevante: operações de serviço ainda ocorrem sob o lock compartilhado. EAF-4 retirou dele a montagem textual/base64 do envelope MCP, validação do protocolo e descoberta do schema. Captura, persistência e materialização de inspeções volumosas ainda podem bloquear a simulação; parse inicial e escrita no transporte ficam fora. Uma representação binária tampouco eliminaria esse bloqueio por si só.

### Custos identificados por leitura de código e correções EAF-4

- Snapshot físico, objetivos e velocidades agora reutilizam mapas e suas chaves. Strings de estado reutilizam capacidade. Contatos usam pares de índices ordenados internamente e nomes somente em eventos/inspeções; a ordem externa permanece lexicográfica.
- `Engine::update_render_frame` substitui a cópia integral do documento por dados visuais compactos em um vetor reutilizável. `Renderer` mantém vértices e buffer GPU, recalcula caixas alteradas e envia a faixa entre a primeira e a última alteração; crescimento de capacidade é geométrico. Remoção reduz o draw count; câmera/luz são independentes da geometria.
- Navegação reutiliza vetores de obstáculos, grade, custos, predecessores, heap e caminho. A conexão inicial escolhe o mínimo com o mesmo desempate por índice, sem alocar/ordenar candidatos. A grade continua sendo recalculada para respeitar obstáculos móveis.
- A janela não solicita novos desenhos quando pausada e inalterada; consulta o estado MCP a cada 16 ms. Resize e pedidos de redraw do SO continuam atendidos. Atualização de título ocorre somente quando muda o estado exibido, fora do lock.

Essas correções removem trabalho identificável no código, sem ganho de FPS medido. Sensores/navegação ainda procuram entidades por ID e percorrem obstáculos; mapas, coleta de diagnóstico, readback de captura e sincronização de serviço permanecem candidatos futuros. O laboratório priorizou validar contratos e o ciclo observável; os limites pequenos atuais não demonstram eficiência para mundos grandes. O checklist respondido está em [performance.md](performance.md).

### Direção proposta por fronteira

| Fronteira | Representação e propósito |
|---|---|
| Autoria | Documento versionado com IDs estáveis, receitas, referências e overrides; JSON pode continuar aqui |
| Build de conteúdo (cooking) | Validar, resolver referências, pré-processar malhas/texturas/navegação/shaders conforme o alvo e gerar pacotes binários em chunks |
| Carregamento/streaming | Metadados versionados, offsets/tamanhos/alinhamentos explícitos, integridade, dependências e compressão por chunk quando vantajosa |
| Runtime CPU | Handles compactos com geração, referências resolvidas, arrays contíguos e buffers reutilizados; AoS/SoA por padrão de acesso, não regra universal |
| GPU | Meshes/buffers persistentes, dados de instâncias atualizados e formatos de textura adequados ao alvo; evitar reconstruir geometria estática por frame |
| MCP/editor | Comandos tipados aplicados em fronteira definida do tick; snapshots consistentes e limitados, serialização e I/O fora do lock da simulação |
| Multiplayer futuro | Protocolo de replicação dedicado, binário, com quantização/deltas e seleção de relevância; contrato separado do MCP |

Comandos assíncronos exigirão definir confirmação, ordenação, revisão/tick, backpressure e erros. Snapshots/capturas precisarão manter consistência mesmo fora do lock; não basta mover trabalho para uma thread. Bulk data poderá usar arquivos/blobs ou memória compartilhada com contratos de propriedade/vida útil, conforme a fronteira efetivamente necessária. Plugins/Wasm têm ABI própria a decidir; a engine não pode prometer passar structs Rust arbitrárias entre linguagens.

No mesmo processo, sistemas devem consumir dados tipados/visões do mundo diretamente: não há necessidade de serializar para se comunicarem. ECS é uma opção de organização, não pré-requisito automático para cache locality; complexidade de scheduler e paralelismo precisa compensar sua sobrecarga. Preparação de assets em build e organização em memória normalmente tratam problemas diferentes do tamanho do envelope de controle.

### Seleção de formato binário

FlatBuffers é candidato para dados interoperáveis entre linguagens, com acesso ao buffer serializado e evolução de schema sob regras explícitas; indirections e layout de arquivo não o tornam automaticamente ideal para loops mutáveis. rkyv é candidato a arquivos de acesso sem desserialização convencional no ecossistema Rust; requer considerar validação, alinhamento e política de compatibilidade. Arrays binários com layout definido podem servir a geometria/chunks especializados. Não escolher um único serializer para cenas, rede, GPU e runtime apenas por um benchmark genérico.

Não persistir o dump cru de uma struct Rust: layout padrão não é um ABI estável, pointers/Vec/String não são dados portáveis, e endianness/alinhamento/versão devem ser contratos explícitos. “Zero-copy” descreve uma fronteira específica: não elimina validação, page faults, descompressão, indireções nem upload à GPU. Compressão pode reduzir I/O e aumentar CPU; o melhor compromisso depende do asset, plataforma e acesso.

### Medição futura, sem bloquear a revisão de código

Decisão de Guilherme em 23/10/2026: revisão estática de performance obrigatória antes de codificar cada incremento e respondida ao concluir; profiling será considerado quando a ferramenta estiver mais robusta. Corrigir desperdícios claros agora, sem exigir baseline de tempo para cada alteração. Não extrapolar ganhos de FPS da leitura de código. O checklist em `docs/performance.md` e a regra em `AGENTS.md` formalizam essa rotina.

Separar medições: tempo CPU de simulação/render submission, tempo GPU, p50/p95/p99 de frame, alocações/bytes por tick/frame, memória residente e picos. CPU/GPU podem se sobrepor; não somar tempos sem considerar o pipeline. Carregamento frio/quente deve discriminar leitura, parse, validação, resolução, preparação e upload. Streaming precisa medir pausas e orçamento por frame, além do throughput.

Quando a medição entrar no escopo, usar cenas fixas e reprodutíveis nos limites suportados; só ampliar escala após rever os limites. Criar baseline para comparar alternativas estruturais e custo de inspeção ativa versus inativa. Comparar formatos de pacote com dados representativos e verificar roundtrip/versão; escolher com Guilherme o contrato antes de cristalizar o cooker/ABI. FPS alto na sala pequena não comprova arquitetura de escala.

Fontes primárias: [MCP básico/JSON-RPC](https://modelcontextprotocol.io/specification/2025-11-25/basic), [FlatBuffers](https://flatbuffers.dev/), [evolução do schema](https://flatbuffers.dev/evolution/), [rkyv](https://rkyv.org/), [layout de tipos Rust](https://doc.rust-lang.org/reference/type-layout.html).


## ADR 0005 — Pistas sonoras de gameplay (EAF-6)

Escolha: ruídos pontuais tipados, consumo na fronteira do próximo tick e registro no replay. O dispatcher resolve a origem uma vez. Percepção recebe uma slice de `Noise` (id/tick/posição/raio), sem entidade emissora; `hear` considera listener e oclusores. Decisão recebe só a observação selecionada. O NPC investiga som anônimo; apenas visão/ onisciência confirma a posição do alvo. Isso também permite iscas e emissores procedurais sem acoplamento a assets de áudio.

Alternativas: derivar percepção do áudio reproduzido impediria equivalência headless e acoplaria gameplay ao dispositivo/mixer. Um barramento genérico de eventos anteciparia contratos de prioridade, assinaturas e ciclo de vida para domínios ainda não implementados. Este recorte mantém um log acústico delimitado e específico; poderá se conectar a passos, impactos e áudio posicional posteriormente. Não introduz dependência, ABI ou thread.

Transmissão por segmento entre centros com multiplicadores explícitos de alcance é uma aproximação barata e verificável. Não é propagação acústica física nem estima identidade/direção com incerteza. Posições das pistas são exatas; difração, portas semânticas, erro de localização e múltiplas hipóteses exigem decisões futuras. Default de transmissão 1 evita derivar acústica implicitamente da colisão; cenas precisam configurar materiais/caixas relevantes. Listener opt-in preserva todos os perfis existentes.

Visão tem prioridade. Ruídos simultâneos escolhem maior intensidade relativa e último comando no empate. Nova pista renova memória/budgets; sons repetidos podem sustentar uma investigação, mas há limite explícito de 16/tick e 4096/sessão. Não criar passos a cada frame; um sistema de locomoção futuro deve emitir eventos com cadência/limites próprios.

`Engine` armazena log e cursor, emprestando apenas eventos novos a `Simulation::step_with_noises`. Sem duplicar filas por NPC ou reexaminar o log por tick. Memória/estado mantêm tamanho fixo. O log cresce sob demanda e retém capacidade após reset; limite numérico impede crescimento ilimitado. Em ticks sem ruído o sensor acústico retorna imediatamente. Com ruído, o pior caso continua limitado a NPCs × eventos × entidades; índice espacial será avaliado com escala efetivamente necessária.

Cena v6 exige versão explícita para `hearing`/`sound_transmission`; v1–v5 migram sem ativá-los. Replay v6 adiciona log ordenado, incluindo pendentes ainda não consumidos; validado antes da simulação e restrito ao mesmo build. Reset/edição/histórico descartam fila/episódio. Contratos de ruídos e snapshots estão em `commands.md`; teste de uma pista não prova reprodução de áudio ou acústica realista.


## Guias de percepção para diagnóstico (EAF-7)

`Engine::debug_view` pertence à sessão de visualização; `view_revision` invalida janela pausada sem tocar no documento ou tick. `set_debug_view` é usado pelo MCP e pela tecla V. Sem campos novos em cenas/replays e sem mudanças em sensores/navegação. O frame visual recebe até oito structs compactas com posição/yaw/alcance/FOV/cor, derivadas dos dados existentes. Não consulta a posição escondida nem recalcula raycasts.

Seta indica orientação corrente; cor indica detecção/memória do último passo. A ordem sensor → decisão → navegação → orientação → física continua igual, portanto a pose exibida pode estar um tick à frente da amostra sensorial. O contorno nominal é uma projeção horizontal no piso; não é uma visualização de oclusão 3D. Linhas tracejadas desenhadas sobre a cena tornam essa intenção explícita. Onisciência usa losango, sem cone direcional; desligado usa cinza sem alcance.

Renderer mantém cache e buffer específicos para guias, separados das caixas. Compara conteúdo, regenera somente quando muda, retém capacidade e faz um draw adicional apenas com guias presentes. Shader não iluminado mantém as cores legíveis. Recursos mínimos da pipeline são criados na inicialização, mesmo com a opção desligada; geração/upload/draw de guias só ocorrem quando necessários. Captura e janela compartilham implementação. Sem dependências/fontes novas, sem prometer oclusão exata ou melhoria de FPS.

## ADR 0006 — passos como emissores físicos determinísticos

**Decisão:** componente `footsteps` opcional em jogador/NPC, referenciando um piso fixo e parametrizado por stride, raio e velocidade mínima. A implementação mede deslocamento XZ resolvido, exige intenção de locomoção e contato ativo no topo do piso. Somente a física pode transformar movimento solicitado em deslocamento elegível. Segurar contra a parede, cair ou ser empurrado parado não produz uma sequência de passos.

Eventos são gerados depois da física e consumidos pela percepção no próximo tick, com coordenada congelada do centro do corpo. Isso evita consultar posição futura ou depender da taxa de renderização. O emissor armazena apenas um acumulador de distância; paradas preservam a fração, perda de apoio zera a fração. Impulsos anormalmente grandes são coalescidos em um evento por emissor/tick. Esse recorte não sincroniza animações de pés nem reproduz áudio no dispositivo.

**Memória e replay:** comandos manuais continuam no log limitado de 4096. Passos são consequência da simulação e são regenerados por cena + entradas; replay 7 compara estado dos emissores, fila pendente e histórico recente além de física/percepção/navegação. Até9 emissores (jogador+8NPCs), fila reutilizada, anel de 256 eventos com contador de descarte. Evita crescimento por tempo de caminhada e parada abrupta ao alcançar o orçamento de comandos manuais. IDs automáticos começam em 2³¹; IDs manuais permanecem sequenciais desde zero.

**Identidade:** evento automático contém índice u16 na cena normalizada para excluir o próprio NPC ouvinte. O índice é estável durante a sessão; edição reinicia a simulação. Isso é proveniência para diagnóstico/autoexclusão, nunca conhecimento concedido ao NPC: sua memória recebe somente posição/tick/source=sound/noise_id. Outros NPCs podem ouvir o passo. Em intensidade igual no mesmo tick, comando manual ganha do passo gerado pela física anterior, preservando ordem causal. O log de comandos do replay rejeita eventos com índice de emissor.

**Compatibilidade:** cena 7 exige versão explícita para footsteps; cenas v1–v6 sem componente continuam silenciosas. Configurar por upsert valida corpo, superfície e limites antes de editar. `inspect_footsteps`, `inspect_noises`, captura e guias acústicas expõem estado e impulsos. Nenhum controle exclusivo da janela. A representação de `Upsert` em Rust usa Box<Entity> para não ampliar o enum de comandos frequentes; a alocação acontece em edição e não muda o contrato JSON.

**Custos/limites:** referências de entidade e piso resolvidas uma vez por cena/reset; por emissor consulta corpos/contato existente, calcula distância e no máximo adiciona um estímulo. Sem raycast acústico extra, fila por ouvinte, log infinito, novas dependências ou thread. Deslocamento e intenção são uma aproximação de locomoção; gait, animação, múltiplos materiais/pisos, rampas e efeitos sonoros renderizados são extensões separadas.

## ADR 0007 — perfis de locomoção e ruído

**Decisão:** entrada tipada com enum `walk/sneak/run`, perfis opcionais no jogador e seleção por tick. Cada perfil contém velocidade horizontal e multiplicador do raio de passos. Ausência de configuração usa 4/1,5/6 m/s e multiplicadores 1/0,2/1,8, respectivamente; ausência de modo na entrada mantém caminhada. Passos continuam opt-in. Cena/replay 8 incluem esse contrato; versões anteriores de cena sem perfis continuam legíveis.

MCP e teclado usam o mesmo `SetInput`. Shift solicita corrida, Control solicita furtividade e tem prioridade sobre Shift. Soltar modificadores preserva eixos ainda pressionados; perder foco limpa teclas e entrada. Pausa conserva a entrada e congela o tick; mudanças de modo pausadas aparecem na inspeção/título. Reset/edição/histórico voltam a walk. Furtividade aqui não muda colisor, postura ou animação, nem garante invisibilidade ou silêncio absoluto.

**Ruído:** a cadência continua determinada pelo deslocamento físico apoiado. Aplicar multiplicador apenas ao emitir, com zero significando silêncio e raio positivo limitado a [0,1;100] m. Mesmo em silêncio a distância do passo é consumida, impedindo acumular um estouro ao voltar a caminhar. A fórmula de audição/atenuação continua igual. Perfis alteram o estímulo, não concedem conhecimento do emissor ao NPC.

**Alternativas/tradeoffs:** um sistema geral de animação/gaits/estamina anteciparia dependências que este teste de furtividade não precisa. Limitar o componente ao jogador preserva o controlador de navegação do NPC, cuja velocidade permanece em seu próprio componente. Isso é uma ponte pequena e explícita, não a arquitetura final de avatares; posteriormente ambos podem consumir um controlador de movimento comum sem mudar o contrato de percepção. Seleção por enum evita strings/lookup por tick; perfis são Copy resolvidos no reset, nenhum buffer, thread, dependência ou I/O novo. O replay grava o modo junto aos eixos e regenera passos. Nenhum ganho de FPS foi medido.

## ADR 0008 — patrulha como rotina separada da percepção

**Decisão:** `npc.patrol` opcional fornece objetivos autorados somente quando a percepção não tem memória ativa. Pipeline: sensores/decisão → rotina se elegível → navegação → física → passos. Uma pista visual ou sonora suspende a rotina imediatamente; ao esgotar a investigação, retoma o índice interrompido, reiniciando prazo/espera. Estado de patrulha separado de `NpcState`: um NPC pode estar `unaware/lost` sobre o jogador e `moving/waiting` em sua rotina. A rotina nunca recebe posições vivas do mundo e não inventa observação.

Rotas por lista de até 16 pontos XZ ou receita circular (centro, raio, quantidade, semente que gira o círculo por seed % 360 graus). Altura derivada do piso e corpo na criação/reset; pontos resolvidos uma vez. Percurso circular ou ida e volta, espera por ponto e prazo máximo por tentativa. Não há grafo de behavior trees, utility AI, scheduler, novo pathfinder nem movimentação por teleporte. Usa NavGoal/A*/física existentes. Esses mecanismos maiores ficam para uma decisão com casos de comportamento que os justifiquem.

**Falhas e custo:** um ponto inalcançável ou cujo prazo terminou é descartado, no máximo um por tick. Um bitset u16 registra índices distintos que falharam desde o último sucesso/interrupção; somente após todos falharem a rotina aguarda retry_ticks. Assim, voltar por um ponto já falho em ping-pong não equivale a testar a rota inteira. Cooldown não fornece objetivo e evita repetição de A*. Os timers e contadores têm tamanho fixo, mapas/pontos são alocados na criação. Lookups por ID/scans de metadados seguem os limites existentes de oito NPCs/512 entidades.

**Contratos:** configuração, inspeção e edição pelo MCP; snapshot e replay incluem estado da rotina. Cena/replay 9, cenas anteriores sem patrulha mantêm comportamento. Undo/edição/load/reset recriam rotas e índices. O resultado anterior da navegação só é consumido se destino e tick pertencem à tentativa atual, evitando confundir chegada de uma perseguição com a da rotina. Pontos fora do piso ou bloqueados são permitidos como autoria e produzem falha observável; o gerador não promete conectividade. `wait_ticks` não é varredura sensorial: NPC parado conserva orientação de chegada e continua percebendo normalmente.

**Limites:** NPC ainda exige um jogador-alvo e piso único no componente existente; onisciente sempre tem pista e suspende a patrulha. Não há escolha de waypoint mais próximo, agendas, facções ou memória de rota aprendida. Cadência de navegação continua a mesma; sem profiling ou alegação de escala/FPS. Estados transitórios são comparados no replay do mesmo build; receita circular determinística não é promessa de RNG uniforme.

## Guias de rotas para diagnóstico (EAF-13)

`npc_routes` é opção de sessão do mesmo patch de debug, com J no dispatcher. `debug_routes::fill` projeta rota autorada e resultado existente da navegação em LineGuide numérico; nenhum sensor/pathfinding é repetido. Linha contínua branca/cross de destino representa conhecimento operacional; tracejado lilás representa intenção autorada de patrulha. Mantém essa diferença quando uma pista suspende a rotina. Sem novo campo em cena/replay.

Até 256 segmentos de navegação e 16 pontos/arestas por NPC; truncamento explícito nos metadados de captura. Até 2.448 linhas/20.064 vértices adicionais com oito NPCs. Traços usam subdivisão fixa por aresta, não por metro, e compartilham o cache/buffer/draw de guias. Alterações de cor/geometria/quantidade invalidam por conteúdo; lista vazia limpa desenho sem descartar capacidade. O overlay não confirma conectividade, área visível nem detecção de jogador. Índice, timers e motivos continuam no MCP; texto/rótulos no espaço 3D ficam fora deste recorte.

## ADR 0009 — cenários de verificação limitados e isolados

**Decisão:** documento de teste v1 com ações tipadas (`set_input`, `step`, `emit_noise`, `assert`, `until`), condições de posição/percepção/patrulha/passos e orçamento finito. Executa em uma Engine isolada a partir da cena de autoria atual, sempre do início. Não copia a partida viva nem altera sua revisão, estado, entradas, replay ou preferências. Isso reduz o protocolo necessário para verificar um comportamento de vários segundos: o cliente envia um roteiro e recebe evidência dos checkpoints.

**Limites/validação:** 1–256 ações, orçamento 1–3.600 ticks e pelo menos um check. Soma conservadora de todos os `step.ticks` e `until.max_ticks` precisa caber antes da execução, ainda que algumas condições terminem cedo. Valores e referências são validados por completo; o runtime mantém limites operacionais de ruído (uma sequência de 17 emissões sem tick, por exemplo, produz falha de execução no relatório). Não há loops/recursão arbitrários, código fornecido pelo usuário, JSONPath ou interpretação de JSON por tick.

`until` consulta structs Rust antes do primeiro passo e após cada passo, inclusive no limite final. Condição já satisfeita consome zero ticks. A primeira falha encerra o roteiro; relatório indica índice zero-based, tick, condição e valor real. Índice/tick são evidência do ponto de observação, não o instante anterior em que um sensor foi amostrado. Checkpoints serializam valores sob demanda; apenas um snapshot completo no final. Documento/relatório podem usar JSON porque são autoria/diagnóstico, enquanto simulação e predicados permanecem tipados.

**Persistência:** save/inspect/run pelo serviço/MCP, reaproveitando root, limites de arquivo, proteção de caminhos e escrita atômica. Run pode salvar relatório e replay da execução isolada, inclusive de um prefixo que falhou. Destinos precisam ser distintos do roteiro e entre si, considerando aliases de diretórios e origem via symlink. Relatório salvo e devolvido usam a mesma árvore Value, evitando representações distintas de f32. Cada arquivo é substituído atomicamente; não há transação que agrupe os dois outputs caso o disco falhe entre escritas.

**Tradeoff:** execução síncrona sob o lock existente do serviço, como replay. O limite é 3.600 ticks, dez vezes menor que a sessão, mas não representa garantia de latência de parede; a janela pode aguardar a chamada terminar. Um serviço assíncrono/cancelável exigirá ordenação, jobs, captura e consistência de resultados; não antecipamos esse contrato. Inicialização/cópia da cena, buscas por ID e serialização final são custos de ferramenta, não do frame. Runner não é scripting de gameplay, savegame, suite genérica nem prova de determinismo entre builds/plataformas. Assertions ampliadas e composição de suítes podem evoluir separadamente.

## Inspeção de replay por tick (EAF-15)

`Engine::replay_at` centraliza validação/reconstrução para run_replay, inspect_replay e capture_replay. Cria Engine isolada, aplica entradas/ruídos por cursor monotônico, retém prefixo uma vez e inclui impulsos do tick escolhido como pendentes. Passos automáticos e estados de percepção/navegação/patrulha vêm da simulação normal. Nenhum snapshot/JSON é produzido por tick. Comparação do final usa a instância pronta; prefixo não recebe promessa de correspondência não gravada.

Escolha incremental: reexecutar O(tick) a cada consulta em vez de persistir checkpoints de Rapier e estados próprios. Reutiliza o contrato v9 e não introduz formato/ABI de savegame. Limites atuais tornam o custo finito, mas não estabelecem SLA; operação síncrona pode bloquear a janela. Jobs/cancelamento/checkpoints deverão ter contratos próprios se a escala exigir. Uma cópia limitada do prefixo de inputs/ruídos permite reusar Engine/render/diagnósticos sem outro modelo de mundo.

Service compartilha Renderer/cache/PNG entre captura viva e reconstruída. Cena/câmera gravadas e guias de sessão são fontes explícitas. Validação de paths/viewport precede reconstrução; output não substitui a origem. Estado de controle não gravado (playback, entrada apenas pendente, revisão/histórico de edição) não é apresentado como reconstituído. Capturar não troca a partida nem valida gameplay por aparência.


## ADR 0010 — volumes discretos e apresentação independente

Componente trigger opcional em entidade sem corpo, configurado por caixa fixa e IDs explícitos. Módulo `trigger` recebe posições físicas e produz dados de ocupação/transição; não controla movimento, percepção, quests ou ações. `visible` controla somente geometria apresentada. Marcadores visuais permanecem entidades independentes. Cena/replay10, cenário2 para predicados novos; migração das versões anteriores preserva ausência de volumes/visibilidade padrão.

Alternativas: colliders sensores do Rapier poderiam estender formas e eventos físicos; teste simples de caixas resolve o contrato atual com independência do backend e ordem explícita pós-física. Foi escolhido AABB discreto limitado, com sobreposição positiva nos três eixos. Isso não substitui a colisão do Rapier nem promete detecção contínua. Uma passagem inteira entre amostras pode escapar; rotação/formas/sweep deverão pressionar esse contrato antes de ampliar fidelidade.

Runtime prepara atores únicos/volumes/alvos em ordem lexicográfica. Cada ator é consultado uma vez por passo; relações usam índices u16, ocupação em u32 e eventos Copy com índices u8. Até32×32 relações; ring512 pré-reservado, descarte antes de push, sem strings/JSON ou alocação por transição. IDs são decodificados na inspeção. Não há índice espacial novo: limite pequeno explícito; buffers/configuração refeitos somente quando a cena reinicia.

Snapshot e replay incluem contadores/ocupação/histórico, permitindo detectar adulteração e inspecionar prefixos. O cenário2 pode verificar entrada/saída sem introduzir lógica de vitória embutida. Reset/edição não emitem saídas sintéticas; primeiro passo observa ocupantes iniciais. Zero trabalho de amostragem quando não há volumes; eventos são estado limitado, não um barramento de callbacks arbitrários.

## Transações de autoria (EAF-17)

SceneEdit é um enum restrito a entidades/câmera/luz; Command::EditScene contém vetor limitado. Engine aplica operações a uma cópia da cena, valida apenas o resultado e chama replace uma vez. Comandos individuais usam iterador de um item no mesmo caminho, sem alocar um vetor. Replace move a cena anterior ao histórico, descartando o item mais antigo antes de push quando cheio; evita clone redundante e crescimento temporário do deque para33.

O isolamento é de autoria dentro do dispatcher síncrono. Nenhum estado parcial chega a física/render/percepção; erro não toca no runtime ou histórico. A ordem é significativa para remove/upsert, mas referências só precisam ser consistentes no commit. Scans lineares frios O(edições×entidades) foram mantidos sob limites512/1.024 staging, evitando um índice adicional com invalidação própria. Formatos de cena/replay permanecem10. Não é scripting/runtime mutation, edição colaborativa distribuída ou transação de arquivos.


## Posse local da entrada (correção de EAF-11)

Perder foco é uma limpeza de controle nativo, não uma intenção de sobrescrever o MCP. KeyboardControl mantém a revisão do último SetInput que aplicou, registrada dentro do lock do serviço. Engine avança input_revision em SetInput válido/reset. Release confere e consome a posse sob o mesmo lock; comparação de valores seria insuficiente para um comando MCP posterior idêntico. Eventos de tecla sem mudança não reenviam estado. São dois escalares internos, sem alterações de física, replay ou contratos de autoria.

## Seleção e picking de caixas (EAF-18)

Estado de edição guarda um ID estável e índice u16 resolvido no comando/reset. Não altera Scene, simulação, entrada ou replay. Frame carrega somente Option<RenderBox>; índice evita scan autoral por frame, com a consulta física já usada pelo renderer. Seleção explícita admite invisíveis para diagnóstico; picking os omite. Clique esquerdo traduz coordenadas físicas pelo mesmo SelectAt do MCP, sem caminho de lógica exclusivo da janela.

O módulo picking compartilha projeção/view com renderer. Desprojeta o pixel no plano próximo em espaço de câmera e escala a direção até o plano distante antes de transformar para mundo; evita cancelar grandes translações ao inverter/desprojetar o plano distante em f32. Interseção slabs percorre até512 caixas, guarda apenas melhor parâmetro e resolve empate por ID. Sem lista de hits, triângulos, nova aceleração espacial ou raycast de gameplay. É uma escolha proporcional ao laboratório de caixas; importação de meshes exigirá discutir picking de geometria ou proxies.

Contorno usa12 arestas, duas fitas cruzadas por aresta,144 vértices no mesmo cache/buffer/pipeline dos guias. A segunda fita evita degeneração nas verticais e reduz dependência do ângulo de câmera; largura fixa de0,036m, sem promessa de espessura constante em pixels. Overlay sem depth test é intencional para inspeção através de objetos; não participa de percepção. Movimento do cursor só guarda dois números e não dispara desenho. Título usa o ID emprestado durante a formatação já existente, com a chamada nativa após liberar o lock.

## Autonomia inicial de NPCs (EAF-19)

NpcConfig.target passa de String obrigatório para Option<String>, versionado em cena/replay11. Esta é a menor mudança que remove a dependência de player sem antecipar seleção automática ou relações sociais. Ausência suspende apenas aquisição visual; percepção acústica e rotina não precisam de um alvo. Onisciência explicita como observar um objetivo existente, não inventa um. A validação de versões anteriores mantém target player obrigatório, evitando reinterpretar arquivos antigos malformados.

Observe resolve target com retorno antecipado paraNone; a decisão recebe a mesma observação numérica. Objetivos sonoros usam dimensões do próprio NPC porque a pista não identifica ator; objetivos visuais usam metadados do alvo. Navegação exclui da lista de obstáculos apenas um alvo explicitamente configurado. Nenhuma outra consulta implícita ao jogador foi adicionada.

NPC-NPC usa o snapshot de posições existente antes da atualização física de todos os corpos. Ciclos de referência são dados, não recursão, e não exigem ordenar agentes por dependência. Ordem de entidades é normalizada; testes com referências mútuas verificam observação simultânea, movimento e replay. Relações/seleção entre múltiplos candidatos precisarão manter esta fronteira sensorial.

A prioridade de continuidade e os critérios integrados estão em npc-roadmap.md. Frameworks de decisão, memória de múltiplos atores, facções, navegação ampliada e habilidades continuam decisões a apresentar antes de implementações estruturais.


## ADR: relações direcionais e seleção baseada em evidência — EAF-20

Relações e sensores são etapas distintas. Facção determina elegibilidade, jamais fornece conhecimento de localização. Mantemos `target` explícito e acrescentamos política opt-in `hostile_nearest`; não introduzimos framework de decisões, tabela de ameaça ou combate nesta etapa. Margem relativa estabiliza escolha enquanto o incumbente é observado; perder observação permite trocar para outro candidato observado. Uma memória conserva identidade e última coordenada, sem histórico multiator.

Catálogo compila atores ordenados por ID e matriz9×9 de atitudes ao construir/resetar a simulação. Índice de ator u8 estável por ID referencia memória; índice u16 referencia a entidade no documento. A ordem autoral do documento não altera desempates ou resultados. Matriz evita resolução de strings/regras no tick; ranking usa dois acumuladores na pilha e distâncias². Apenas inspeção traduz índices para IDs e constrói JSON. Replay12 retém estado tipado e índices, vinculado a cena/build.

Simulation chama `update_with_context` com o catálogo cacheado e snapshot pré-física comum. As APIs auxiliares `update`/`update_with_stimuli` constroem catálogo por chamada para conveniência/testes; não são o caminho quente da Simulation. Sensores ainda fazem scans limitados de entidades/oclusores e consultas no mapa de posições existente. Pior caso8NPCs×8candidatos×oclusores: limite deliberado, ainda sem índice espacial nem promessa de escala.

Navegação recebe a identidade do ator lembrado separada da coordenada conhecida, excluindo seu corpo do planejamento. Continua usando geometria global/plana, inclusive outros atores físicos: a garantia de privacidade desta etapa é de seleção/memória, não de conhecimento local de obstáculos. Este tradeoff já existia na navegação do laboratório e exige trabalho próprio no roadmap.


## ADR: exposição visual individual antes da seleção — EAF-21

Introduzimos recognition opcional por NPC, separado de awareness (o que o sensor recebe), targeting (qual confirmado escolher) e memory (qual pista movimenta a investigação). Isso evita converter o tempo de perseguição em atraso de reconhecimento ou transferir exposição ao mudar o melhor candidato. Não introduzimos FSM/árvore de comportamento/utility: a etapa apenas filtra observações antes da seleção já existente.

Knowledge mantém nove Track fixos indexados pelo catálogo ordenado de atores. Track contém escore u32, confirmação, visibilidade do passo e última observação numérica. Observação passou a Copy por conter somente escalares; cópias não alocam. O mesmo teste de visão alimenta o registro e a seleção, sem repetir ray/box queries. Sem configuração, Simulation pula os scans de conhecimento e mantém resposta imediata. Estado fixo por NPC/replay é o custo deliberado desta representação, adequado ao limite atual; não prometemos escala sem medi-la.

Usamos unidades inteiras com limiar acquire_ticks×decay_ticks, incremento decay_ticks e decremento acquire_ticks. Com ambos limitados a3600, o máximo12.960.000 cabe em u32 e evita deriva de integração. Confirmação é latched até zero, permitindo reencontro rápido; adquirir evidência não fornece objetivo de movimento antes da confirmação. Patrol continua, audição anônima continua capaz de interromper. Regras de prioridade mais amplas e pistas acústicas concorrentes permanecem trabalho futuro.

Knowledge registra apenas candidatos visuais elegíveis, não todas as pessoas da cena. Não é memória espacial multiobjetivo: slots ocultos não são escolhidos como novos destinos. A memória ativa anterior conserva seu próprio orçamento. Ordem de atualização e snapshot pré-física comuns preservam determinismo; edições/load/undo/redo resetam o catálogo e o conhecimento juntos. Replay13 serializa slots e snapshot/inspeção traduzem IDs/progresso na fronteira. Guias existentes apenas mudam de cor, sem geometria nova.


## ADR: arbitragem limitada de pistas acústicas — EAF-22

Mantemos um objetivo de investigação ativo, com sound_attention opcional. Escolhemos compromisso temporal, margem aditiva de escore, supressão espacial e proteção breve da memória visual, em vez de introduzir fila persistente, sistema de ameaça ou framework de comportamento. O contrato é local ao ouvinte e usa apenas observações recebidas. Supressão espacial agrupa pontos sem afirmar que o mesmo indivíduo os emitiu; pistas rejeitadas são descartadas.

hear reaproveita o scan existente de impulsos/atenuação. Cada audível passa pelo filtro e ranking em uma só passagem; dois melhores escalares (audível e elegível) permitem explicar por que o mais forte foi ignorado. A comparação usa o recibo anterior ao passo, nunca o candidato provisório, evitando dependência indevida da ordem. O estado persiste apenas ID/intensidade da pista ativa; contagens e enum são diagnóstico do passo. Sem alocações/listas/strings novas no tick, limites originais de ruído/NPC preservados.

O escore antigo fica congelado na recepção, separado do movimento atual do ouvinte. Isso sacrifica uma simulação acústica contínua em favor de semântica reproduzível para impulsos. Semântica semelhante a atenção, não pressão sonora. Memória expirada por tempo ou busca é excluída antes da arbitragem; orçamento de viagem já esquece durante pós-física. Rejeições não reiniciam budgets. Visão confirmada atual mantém precedência e pula o sensor acústico como antes.

MCP inspect_hearing expõe config/decisão/memória; inspect_npcs e replay14 contêm State tipado. Nenhuma troca no renderer, event bus, física, ordem de passos ou framework de decisão. Ausência da política preserva substituição legada imediata pelo ruído mais forte. Próximas decisões de composição de ações/território permanecem próprias, sem extrapolar esta política para combate ou comunicação.


## ADR: seletor limitado de objetivos sobre executores existentes — EAF-23

Escolhemos um seletor declarativo de prioridades estáticas sobre as FSMs existentes de percepção/investigação e patrulha. Uma FSM única exigiria transições entre todas as combinações; uma árvore de comportamento exigiria contratos de nós/sequências/abort/sucesso ainda não exercitados; utility exigiria normalização e estabilidade de escores de ações. O seletor de até8 entradas atende interrupção/retomada verificáveis agora, com ações tipadas, sem prometer substituição de um framework geral. Futuras árvores/políticas podem gerar intenções no mesmo limite de navegação; ainda não existe ABI de extensão.

A configuração opcional objectives tem condição, prioridade u8 e ação. Um scan mantém o primeiro candidato de maior prioridade; ordem autoral desempata. Respond reutiliza perseguição/investigação existente (seguir um ator percebido é uma configuração de target/stop_distance). Flee escolhe ponto radial a uma distância da pista; guard usa ponto autorado; patrol reutiliza sua rota; wait retém controle sem destino. São objetivos contínuos enquanto elegíveis: chegada significa manter posição, não remover tarefa. Inalcançável permanece selecionado e expõe falha da navegação; não há fallback silencioso ou busca tática de saída.

Sensores continuam ativos. Seleção recebe observação aceita ou memória ainda válida e flag de visão confirmada atual; nunca recebe mapa/posição do alvo. Ações de movimento recebem só posição própria, altura e recibo. Fuga congela destino até nova evidência/ativação; se já estiver além da distância, mantém posição em vez de se aproximar. Direção coincidente usa +X determinístico. Navegador atual continua consultando geometria global/plana: o limite de conhecimento do mapa não mudou.

Uma ação controla NavGoal. Troca de índice invalida feedback anterior antes da investigação e limpa caminho antes de patrulha/navegação, mesmo se destinos coincidirem. Percepção recebe execute=false para atualizar evidência/expiração sem avançar investigação; fase suspended distingue pista conhecida de movimento investigativo, evitando cobrar distância de fuga/guarda/patrulha. Memória e busca usam relógio de simulação e continuam expirando durante suspensão. Cursor/ponto de busca são conservados, mas não avançam enquanto outra ação controla movimento; uma observação nova reinicia a investigação como antes. Retomada nunca reutiliza o resultado de navegação da outra ação. Na patrulha, ponto/direção permanecem; espera, viagem e cooldown reiniciam segundo o contrato anterior de interrupção.

Estado do seletor é escalar: índices/máscaras u8, enum de ação/motivo, contador saturante, ticks e destino. Sem fila, alocação por candidato, strings no seletor ou nova consulta sensorial. Dois scans limitados adicionais: seleção até8 por NPC e invalidação até8 NPCs. Cena/replay15; inspeção traduz configuração somente sob demanda. Ausência de objectives mantém fluxo legado percepção→patrulha; lista vazia é erro, lista sem elegível deixa o NPC sem movimento e não ativa patrulha implícita.

Limites: prioridades estáticas, preempção imediata sem histerese de objetivo, uma evidência ativa, navegação plana, nenhuma habilidade/combate/interação/agendamento. Compromisso acústico continua anterior à seleção de objetivos. Retomada observável não significa congelar todos os relógios nem garantir que a pista ainda exista. M09 permanece parcial.


## ADR: território comportamental e retorno comprometido — EAF-24

Territory é uma política opcional independente de objectives. Define círculo XZ (home/radius), chegada e espera de rearme. Uma restrição geométrica dura exigiria recortar navegação, recuperação física e regras de fronteira; este incremento limita aquisição/destinos e reage ao deslocamento próprio, reutilizando o navegador existente. Desvios e empurrões podem cruzar o círculo e iniciar retorno. O raio é euclidiano desde home, não distância acumulada percorrida. Não há teleporte, barreira invisível ou garantia de rota interna.

O filtro visual opera após o sensor, antes de reconhecimento/ranking: jamais consulta localização oculta. Pista externa não adquire alvo. Se uma observação externa corresponde ao ator da memória ativa não expirada, marca sua saída; sem alternativa interna confirmada, inicia retorno. Uma alternativa válida vence normalmente. Som externo é filtrado após alcance/atenuação e antes do ranking acústico, sem identificar emissor. Conhecer posição externa de um candidato não revela candidatos ocultos; onisciência explícita também respeita a área de atuação.

Retorno começa por posição própria externa, saída observada do alvo ativo ou destino conhecido externo (por exemplo fuga/busca geradas). Rotas e pontos de guarda autorados devem ficar dentro da área, verificados na validação. Um destino de navegação é conhecido, não a posição real oculta. Iniciar retorno limpa memória, suspeita visual e recibo sonoro; suspende objetivos e patrulha. Sensores e investigação ficam suspensos durante retorno/cooldown: é compromisso deliberado de comportamento, distinto da suspensão comum entre objetivos de EAF-23, que mantém percepção ativa. Não introduzimos imunidade ou regras de combate.

Chegada usa posição física XZ a arrival_distance+0.02m, a tolerância do navegador. Em seguida espera rearm_ticks;0 permite rearmar no mesmo passo de chegada. Se deslocado da origem durante espera, volta a retornar e reinicia a espera na próxima chegada. Retorno bloqueado permanece Returning com motivo de navegação, sem sucesso fictício ou fallback. Após rearmar, sensores recomeçam sem memória antiga e patrulha retoma seu ponto segundo a semântica anterior. Retorno tem precedência sobre qualquer objetivo, inclusive wait/flee/guard; omitir territory conserva o comportamento anterior.

State fixo armazena fase, último gatilho, ticks, contador saturante e contagens de rejeição do passo. Mudanças de fase invalidam navegação e feedback, aproveitando o passo de invalidação existente. Guia territorial usa64 segmentos e um X, junto de npc_routes/J; círculo unitário LazyLock calculado uma vez, transformações/cache existentes, sem influência no gameplay. Laranja ativo, vermelho no retorno/cooldown. Cena/replay16, inspect_territory pelo dispatcher/MCP. Limites atuais: um círculo fixo por NPC, navegação plana/global, sem origem móvel, volumes arbitrários, exceções de emergência ou recuperação tática de caminhos bloqueados.


## Recuperação de navegação limitada (EAF-25)

A política opcional navigation_recovery controla o custo/ritmo de insistência sem substituir o planejador plano. Configuração na autoria, estado escalar por NPC no NavState; conhecimento/destino continuam fornecidos pela percepção/objetivo. Falha produz retry_wait, limpa caminho e marca deadline; espera retorna antes de World.refresh/A*. Backoff exponencial limitado evita repetição a cada12ticks em bloqueios persistentes. Movimento normal mantém cadência existente.

Detecção de stall usa posição física própria antes do passo e somente períodos com velocidade solicitada positiva. Deslocamento XZ maior que min_progress reinicia janela/sequência; ausência dele durante stalled_ticks gera falha. Contato conservador existente continua, mas também é monitorado. Sem acumular trajeto ou guardar histórico. Retentativa reinicia âncora, chegada limpa sequência, idle/disabled/troca de objetivo limpam tudo. Mesmo snapshot/ordem determinística/replay.

Tradeoff: uma passagem recém-aberta pode aguardar até max_retry_ticks; posição atualizada da mesma pista não cancela espera para não permitir que alvo móvel provoque A* a cada tick. Percepção/budgets continuam; patrulha/busca usam prazo próprio quando recebem retry_wait. O estado evita declarar inalcançável definitivo enquanto há retentativa, sem prender rotinas para sempre além de seus budgets. Deslocamento significativo não prova avanço topológico: oscilações amplas ou empurrões podem reiniciar janela. Destino alternativo, prioridade de passagem e coordenação multiagente ficam fora; demandam política própria e testes de starvation/deadlock. Nenhum navmesh/framework/dependência novo.


## Coordenação de passagens explícitas (EAF-26)

Pipeline: sensores/conhecimento → seleção de objetivo → patrulha quando aplicável → reserva de passagem/waypoint temporário → navegação/recuperação → gate de entrada sem reserva → física. MCP configura/inspeciona fora do loop; regras determinísticas Rust executam no tick. Tráfego não consulta identidade/localização de alvo oculto, nem escreve percepção. O coordenador conhece pedidos de movimento e ocupação física da infraestrutura declarada; isso não representa inteligência/onisciência do NPC.

Escolha: reserva exclusiva sobre trecho autorado X/Z, em vez de inferir gargalos ou introduzir um solucionador geral de multidões. Lista pode ser gerada por código/MCP e validada como conteúdo manual. Até8 trechos sem envelope sobreposto e8 participantes; slots de espera pré-autorizados por lado. Uma concessão por agente, prioridade u8+idade, empate ID, posse não preemptiva. Liberação física ou cancelamento/timeout; ocupação impede nova concessão mesmo se antigo dono perdeu a vez. Sem eliminar corpos/colisões. `passage_priority` opt-in é política locomotora separada de facção/urgência do objetivo.

Pedido considera segmento da intenção conhecida cruzando meio da faixa e saída do outro lado. Escolhe candidato mais próximo; não substitui topologia da navegação nem resolve redes automaticamente. Gate conservador sobre corpo varrido impede entrada comandada sem posse, incluindo desvios imprevistos do A*. Já dentro, permite recuar. Falha de conectividade ou ausência de pedido compatível é explícita; nenhum teleporte. Detecção automática/encadeamento de links, evitação contínua e prioridades sociais/táticas exigem recortes próprios.

Estado por NPC preserva original_goal, waypoint, idade, vaga e contadores; estado por trecho guarda dono, ocupação e prazo. Pré-alocação no reset, sem fila dinâmica no tick. Passo usa mesmo snapshot para todos, desempate independente da ordem de entidades. Troca XZ de waypoint invalida NavState; relógios dos sensores continuam, mas desvio de espera não consome budget de distância de investigação. Travessia mantém remaining_travel. No cancelamento dentro da faixa, corpo ainda pode impedir outros: ceder/evacuar sem objetivo válido é política futura, não promessa do scheduler.

Tradeoffs: mais autoria explícita e latência de espera por exclusão total, em troca de diagnóstico/replay e comportamento previsível. Envelhecimento não elimina bloqueios físicos, reservas longas nem geometria inadequada. Faixa/plano e corpo conservador usam maior meia dimensão XZ. Gate atua em comandos; colisões/empurrões continuam físicos. Não há garantia geral de deadlock/starvation ou escala de multidões.


## EAF-27 — rede espacial e motores de travessia

Decisão apresentada antes de codificar: manter o navegador XZ legado e acrescentar uma rede XYZ explícita, por cena, com capacidades por NPC. Alternativas consideradas: estender a grade para voxels (custo cúbico/resolução), navmesh com links externos (adequada a superfícies, insuficiente sozinha para voo), ou rede limitada de conexões tipadas. A rede permite validar primeiro o contrato decisão → caminho → execução física, sem dependência nova nem compromisso com biblioteca de navmesh. Um gerador procedural pode emitir exatamente o mesmo documento pelo MCP; detecção/geração automática da rede a partir da geometria ainda não existe.

Nós são posições dos pés em metros, independentes da altura do corpo. Até64 nós e128 conexões dirigidas walk/jump/glide/fly. Cada NPC opt-in declara walk e velocidades opcionais de salto, voo e planeio. Compilação por corpo/capacidade verifica alcance, suporte estático e varredura de caixas expandida pelo corpo. Caminhada exige uma mesma caixa de apoio em todo o segmento; saltos/planeios exigem destino apoiado. Caminhos são escolhidos por Dijkstra com custo em ticks estimados, desempate por índice autorado. Arrays fixos; nenhuma heurística acessa alvo oculto.

Ancoragem inicial até0,35m de um nó; destino conhecido precisa ter nó dentro da distância de chegada. Isso é navegação sobre rede explícita, não navegação aérea livre entre quaisquer pontos. Não existe conector automático para uma posição arbitrária nem replano contornando novo obstáculo móvel. Falhas têm motivo e retentativa a cada60ticks. Conexões custam no máximo10s estimados e têm prazo extra de120ticks. Colisões dinâmicas permanecem no Rapier e podem impedir a travessia.

Motores: walk controla XZ e preserva gravidade; jump aplica uma única velocidade vertical inicial, calcula tempo balístico e mantém gravidade real, com direção horizontal para o endpoint fixo; fly controla XYZ e compensa gravidade; glide controla descida constante e deslocamento horizontal limitado, compensando gravidade para representar sustentação terminal. Este planeio é um modelo de gameplay sem aerodinâmica/energia/estol, exige início já livre da borda e nunca sobe. Corpos continuam caixas verticais, sem pitch/roll/animação; visão mantém FOV de azimute, distância/oclusão 3D.

Cada trecho iniciado fica comprometido até chegada física ou timeout; ao perder/mudar objetivo, não se congela salto no ar nem se inicia a próxima conexão antiga. Sensores/memórias continuam; committed explica a conclusão do trecho pendente. Fly pode pairar sem objetivo; corpos sem voo caem sob gravidade fora de travessia. Chegada de salto/planeio/caminhada exige apoio e baixa velocidade vertical; voo com endpoint apoiado também precisa pousar antes de encadear. Não há teleporte ou desativação de colisão.

O motor só recebe NavGoal derivado de evidência/objetivo, própria posição/velocidade e geometria estática. Alvo oculto não é consultado. Deslocamento investigativo agora pode contabilizar XYZ; antes de iniciar trecho exige orçamento estimado suficiente mais margem. Uma travessia física comprometida pode concluir após expiração do orçamento/tempo ou pista; budget não é freio impossível no ar, e empurrões externos não são distância comandada garantida.

Guard3d fornece objetivo XYZ explícito (centro do NPC); respond/wait continuam componíveis. Políticas cujo contrato ainda é plano — patrol, guard/flee XZ, territory, passage_priority, navigation_recovery e footsteps de superfície única — são rejeitadas no modo espacial. O legado permanece funcional nos outros NPCs. Busca de memória pode produzir ponto fora da rede e então falha explicitamente. Próximas decisões: conectores/geração de rede, superfícies/rampas, coordenação volumétrica, percepção orientada e controladores/ações de personagens.

Cena/replay19; set_spatial_network (também edit_scene), upsert de capacidades, inspect_mobility; estados e compilação inspecionáveis, replay compara estados de execução. Reset/edição/undo/load reconstroem caches. Guias J preservam altura real, cores por conexão e arco balístico do salto ativo, nunca controlam simulação. Primeiro alvo validado continua macOS, sem mudar renderizador/dependências/plataformas.


## ADR: política de aproximação parcial independente das capacidades — EAF-28

`npc.unreachable_policy` é opcional: ausência/stop exige caminho completo; approach_reachable permite aproximação a um destino conhecido que não tem caminho completo compatível. Separar política de capacidades permite que dois corpos idênticos reajam de modo distinto, sem um escalar de “inteligência” alterar salto/voo ou conhecimento. Não há novas dependências nem troca do planejador.

O Dijkstra existente procura primeiro a solução completa, sem aplicar o limite exclusivo do fallback. Somente se falhar, seleciona entre nós já visitados um ponto que melhora distância euclidiana XYZ ao destino por min_progress e cujo custo acumulado estimado cabe em max_path_ticks. Desempate: menor distância², menor custo, menor índice do nó. Exclui o nó inicial e reserva 0,12 m da tolerância de chegada no requisito de progresso; é conservador perto do limiar. min_progress mede melhora em relação à posição inicial real, não comprimento de rota. max_path_ticks estima esforço por duração nominal das conexões, não é timeout de relógio nem orçamento físico da investigação.

A rota parcial é mantida através dos nós intermediários: um caminho válido pode primeiro se afastar do objetivo. Replanejar por proximidade a cada nó permitiria retrocesso/oscilação; reusar o Vec limitado existente evita isso. Ao alcançar o endpoint, partial_reached/partial_route_end mantém destino original e publica navegação unreachable. Não é sucesso da guarda/investigação. Enquanto destino, raio e posição permanecem estáveis, não roda Dijkstra novamente. Deslocamento físico para fora da tolerância invalida o resultado estacionário; mudar/perder destino termina somente a conexão física comprometida e descarta o restante. Edição/reset/load/undo recompilam como antes.

Capacidades, validação de geometria/apoio, colisões e orçamento investigativo real continuam obrigatórios. Falha de lançamento/budget/timeout limpa o plano parcial e aplica a cadência existente, evitando repetir validação de geometria a cada tick. Nenhuma posição de alvo oculto é consultada. Política só aceita motor espacial: navegador XZ legado permanece inalterado. inspect_mobility expõe política efetiva e estado; guias existentes mostram endpoint parcial laranja e objetivo original. Cena/replay20; versões antigas sem política preservam comportamento, replay exige build correspondente.

Limites: só considera nós da rede explícita, não qualquer ponto livre da geometria. Proximidade não é avaliação tática de linha de tiro, cobertura, risco ou melhor lado de parede. Obstáculos dinâmicos ainda podem bloquear a rota e produzir timeout. Não há navmesh, geração automática de conexões, inferência de capacidades, negociação3D ou retomada automática quando o mundo muda sem invalidar o plano. O contrato permite configurações procedurais pelo mesmo upsert/edit_scene, sem heurística escondida por espécie.


## ADR: conexões locais lineares sobre a rede — EAF-29

A rede explícita continua descrevendo percursos maiores e travessias especiais. `npc.local_connections` opt-in acrescenta segmentos temporários entre posição física, nós da rede e destino conhecido; caminhada e voo apenas. Um navmesh seria adequado a superfícies, mas não resolveria sozinho voo; uma grade volumétrica introduziria resolução/custo de volume. Este recorte mantém o contrato atual, sem dependência nova, e permite destinos fora dos nós antes de escolher uma representação mais geral.

Configuração independente das capacidades: walk/fly habilitam somente modos que o corpo possui; max_distance limita cada conector a0,25..8m; alcance e chegada ao nó compartilham tolerância física explícita de0,002m para o resíduo de contato do Rapier. Esse pequeno limite não relaxa a varredura contra obstáculos nem usa a tolerância genérica de chegada0,12m. Caminhada exige mesma caixa de apoio ao longo do segmento e altura compatível; voo permite XYZ. Segmento inteiro é varrido contra caixas fixas expandidas pelo corpo. Obstáculos dinâmicos seguem no Rapier, podendo causar timeout. Saltos/planeios continuam conexões autoradas; não são inferidos por proximidade. Sem rede, uma política local permite apenas deslocamento direto dentro do alcance, sem descoberta automática de desvios.

Planejamento multi-origem: para cada nó próximo e acessível, custo de entrada desde posição real; Dijkstra usa conexões compiladas e avalia saídas para o destino. Custo completo inclui entrada+rede+saída em ticks nominais; direto compete com esse custo. Empate favorece direto, depois índice do nó final, e ordem estável das relaxações; modos locais empatados preferem caminhada. Ancoragem dentro da tolerância0,12m aceita custo zero se geometria/apoio permitirem. Se a saída falha a partir da posição real, o corpo primeiro alinha fisicamente com o nó. No modo com conexões locais, entrada/links autorados chegam com precisão horizontal0,002m (também vertical para voo sem apoio); altura de apoio mantém a tolerância física anterior. Isso evita encerrar antes do nó e bloquear uma saída perto do limite de alcance, com a mesma tolerância física explícita do alcance. Se nenhum percurso completo existe, a política parcial escolhe nó com melhora/custo suficientes incluindo entrada. Rota parcial conserva desvios e entrada, sem recálculo em cada nó.

`connector` no State contém kind entry/exit/direct, mode e endpoint (pés). Ele é alternativo ao edge autorado e usa o mesmo motor físico, arrival/apoio/velocidade vertical, compromisso e timeout. completed conta trechos físicos, incluindo conectores. Conexões direct/exit chegam ao raio de parada da intenção congelada, preservando linha livre e apoio; não tentam ocupar o centro do ator perseguido. Mesmo após perder ou trocar pista, termina o trecho congelado antes de planejar nova intenção. Isso pode criar atraso perceptível em perseguição: não há steering contínuo nem preempção de caminhada nesta etapa. Destino novo não consulta o ator oculto, só NavGoal já emitido por percepção/objetivos. remaining_travel continua sendo exigido antes de lançar um trecho, com contabilidade física XYZ existente.

Percurso completo é reavaliado ao fim de cada trecho; uma saída direta desde um nó é classificada exit, mesmo quando vence a comparação de custo. Aproximação parcial mantém toda a sequência até endpoint. Falha local sem opção é no_local_route; erro geométrico no lançamento é local_geometry_blocked. Sem mudanças, retentativa espera60ticks; mudança de destino conhecido pode antecipar a busca, como no contrato anterior. Cenários com muitos destinos móveis inalcançáveis podem aumentar esse custo, ainda limitado ao laboratório; não há promessa de escala/FPS.

Caminhos lineares antigos passaram a validar o segmento inteiro uma vez, em vez de centenas de partes colineares; somente salto conserva amostragem balística. Planejamento local usa até129 tentativas de conector (direto,64 entradas,64 saídas), além das consultas limitadas de ancoragem/chegada, arrays fixos e a rota Vec já reservada. Não modifica a rede, não aloca nós temporários, não cresce histórico. Inspeção converte políticas sob demanda; guias marcam endpoint ativo em ciano. Cena/replay21, ausência da opção mantém o planejador explícito. Editar/reset/load/undo/redo invalidam estado/cache atomicamente.

Continuam abertos: geração da rede a partir da geometria, navmesh/rampas/degraus, múltiplas superfícies para caminhada, obstáculos móveis com desvio, percepção orientada em pitch/roll, coordenação espacial e conexão de saltos/planeios em posições arbitrárias. O criador/gerador pode fornecer rede e parâmetros pelo mesmo MCP, com limites idênticos aos da autoria manual.

## ADR: voo curvado por controle limitado — EAF-30

`npc.flight` é opcional: ausente ou `{mode:straight}` mantém o motor retilíneo anterior. O perfil curved exige capacidade fly_speed e acrescenta aceleração, desaceleração, giro, suavidade de direção, velocidade mínima de cruzeiro, capacidade de pairar e direção inicial XYZ. Velocidade máxima continua em mobility.fly_speed; não há dois limites concorrentes. Cena/replay22, comandos compartilhados upsert/edit_scene/inspect_mobility, sem dependência nova.

Escolha: trajetória discreta gerada pelo próprio controlador em passo fixo, em vez de uma spline decorativa seguida por outro motor. Direção desejada é filtrada por tempo de resposta; heading muda no máximo pelo giro configurado. Velocidade escalar acelera/freia e a mudança vetorial reserva orçamento para a curva, limitada pelo módulo de aceleração/desaceleração. O limite de giro vale entre direções; velocidades próximas de zero mantêm heading inspecionável. Ao pairar, a velocidade cai a zero dentro do raio de chegada; a direção final permanece disponível para a próxima partida. Conexões intermediárias também exigem chegada; perfil pairador pode parar em um nó antes do próximo trecho. Ainda não é suavização global de uma cadeia de splines ou passagem contínua garantida em todas as junções.

Suavidade é a constante de resposta do filtro (alpha=DT/(smoothing_seconds+DT)), não um multiplicador de giro; zero responde imediatamente, sem retirar os limites físicos. O caso antiparalelo usa eixo determinístico. initial_direction é normalizada somente no lançamento, independente da orientação sensorial horizontal. minimum_speed vale no cruzeiro: partida parte do repouso, pairadores podem desacelerar abaixo do mínimo para virar/chegar, e não pairadores mantêm o mínimo depois de acelerar. Mudanças por colisão, desativação ou parada de segurança podem impor velocidades fora desse envelope nominal. Transferência de outro motor acima de fly_speed é recusada explicitamente, sem redução instantânea disfarçada de aceleração limitada.

Antes de lançar, simular o mesmo controlador por no máximo1800ticks/30s e varrer o corpo inteiro em cada segmento. Esses segmentos são o próprio movimento por tick, não amostras de uma curva analítica que poderia sair entre pontos. Preview usa somente destino conhecido congelado, geometria fixa e estado próprio. Resultado fornece duração/distância reais previstas ao timeout e à checagem de budget; a contabilidade da investigação continua medindo deslocamento físico XYZ. O grafo conserva custos nominais retilíneos como estimativa: selecionar uma rota não garante que a curva dependente da direção inicial caiba. Recusa informa flight_path_blocked/flight_preview_limit; não há fallback silencioso para reta nem busca completa no espaço de orientação/velocidade. Esse custo ocorre no lançamento selecionado, não para cada candidato de Dijkstra.

Não pairadores passam pelo raio do destino e continuam em órbita: `phase:arrived` significa passagem concluída, não permanência no ponto. A órbita preserva velocidade de chegada, usa metade dos limites de giro/aceleração disponíveis e plano perpendicular à direção de chegada, próximo de horizontal quando possível. A volta inteira precisa caber e ser validada em até3600ticks/60s; sem espaço, o lançamento é recusado como flight_holding_blocked ou flight_holding_preview_limit. Não se acrescenta uma pista fictícia. Perda de objetivo mantém a órbita segura; objetivo novo inicia planejamento desde posição/velocidade atuais, e uma rota indisponível preserva a órbita anterior. Direção/rotação/velocidade da órbita ficam em estado limitado; rotação é calculada uma vez, normalização evita crescimento de velocidade por erro acumulado.

Execução usa Rapier, sem alterar posições diretamente. Verificar próximo segmento contra caixas fixas e comparar posição prevista com física: erro instantâneo acima0,15m ou seis ticks consecutivos acima0,005m interrompem a trajetória; timeout também permanece. Corpos dinâmicos continuam colidindo e podem impor parada de segurança. No início da órbita a volta é revalidada da posição física efetiva; durante holding, próximo passo e tracking continuam ativos. Comandos da curva obedecem limites; impulsos/contatos do solver não são uma promessa de aceleração limitada. Edição/reset/load/undo limpa controlador/preview e recompila.

`State.flight` expõe heading, aim, velocidade comandada, holding, parâmetros da órbita, posição esperada, ticks e falhas consecutivas de tracking. O estado parado pode reter heading com velocidade zero. `flight_path` guarda até128 pontos da última previsão para inspeção/guias, não todos os passos nem histórico crescente. A previsão completa de colisão usa todos os ticks; a guia é reduzida. J/npc_routes mostra curva violeta e direção amarela XYZ, além da rede/caminho nominal existente. Caixas visuais/físicas continuam sem rotação; inclinação, aerodinâmica, animações de asas e campo visual volumétrico são incrementos separados.


## Operação sem interferência no desktop

Execução por agentes usa `--mcp` sem janela como padrão. A simulação usa o relógio headless ou comandos `step`; o renderer de captura é inicializado sob demanda, sem Surface, e desenha em textura usando `Renderer::draw`, também utilizado pela janela. Captura não depende da tela, visibilidade ou foco de outra aplicação. Nenhuma nova API de gameplay foi necessária.

O executor `scripts/run_verifications.py` roda cenários em sequência e processos independentes para isolamento, sobre workspace gerado estável. Não abre a demo após concluir. `--window` é opt-in com seleção explícita de cenários. Janela visível e MCP no mesmo processo continuam compartilhando o Service, mas processos distintos não compartilham sessão; conectar/desconectar um editor de um processo existente fica para trabalho futuro.

Custo restante: simulação, inicialização de processo/GPU e readback de PNG continuam consumindo recursos. Execução sem janela remove a necessidade de apresentação nativa, não garante impacto zero ou ganho de FPS. Testes de janela/foco/entrada física continuam separados; a suíte sem janela não substitui esse gate.


## Visão volumétrica opcional (EAF-32)

`npc.spatial_vision` habilita cone circular em XYZ; sem esse campo permanece o sensor horizontal legado. O alcance já era XYZ e a oclusão já testava segmento3D: esta mudança acrescenta o componente vertical ao FOV e à orientação, sem dar acesso ao alvo oculto. Reutilizar fov_degrees e initial_facing_degrees evita dois contratos concorrentes. Abertura total pode ir de1° a360°;360° ignora ângulo, mantendo alcance/oclusão. A amostra é o centro do alvo, a origem é85% da altura do observador. Não há múltiplas amostras por membro/cabeça.

Yaw/pitch escalares foram escolhidos para a orientação do sensor: suficientes para um cone circular, sem roll e independentes da orientação futura de corpo/esqueleto. initial_pitch_degrees[-90,90], positivo para cima. follow_movement true acompanha velocidade comandada pelo motor XYZ (inclui walk/jump/glide/fly); false conserva orientação enquanto locomove. Busca existente ainda gira yaw, mantendo pitch. Não modela cabeça, velocidade de rotação sensorial ou varredura vertical automática. Direção comandada não implica deslocamento físico após colisão; gravidade/empurrões não governam o olhar. Parada conserva direção. As observações do tick usam a orientação anterior ao comando do motor daquele tick, como no caminho horizontal existente.

O controle não consulta alvos: navegação/motores continuam recebendo somente destinos conhecidos. inspect/inspect_npcs expõem facing_pitch_degrees, vision_follows_movement e vision_direction XYZ nos NPCs volumétricos. Orientação do sensor não rotaciona colisor/mesh. Guias mostram direção atual e resultado da última atualização sensorial; volume nominal não recorta obstáculos nem representa certeza de visão. SetDebugView.npc_vision permanece o único toggle; sem controle exclusivo visual.

Guia usa superfície esférica limitada pelo ângulo, com arco final, quatro meridianos e raios tracejados, até594 vértices por NPC (306 em360°). Ribbon ganhou base alternativa para segmentos verticais, mantendo geometria horizontal. Cena/replay23; cenas1..22 sem spatial_vision preservadas, declarar esse campo em versão antiga é erro. Replays continuam exigindo mesmo build/versão. Sem dependências novas.


## Patrulhas XYZ sobre motores existentes (EAF-33)

Decisão: ampliar o enum de rotas com points_3d/ring_3d e reutilizar a FSM de patrulha, o seletor de objetivos e os motores de mobilidade. Alternativa de uma segunda rotina espacial duplicaria timers/interrupções/falhas. Coordenadas de destino são centros dos corpos, como NavGoal/guard_3d; nós da rede continuam nos pés. Essa diferença permanece explícita no contrato. Até16 pontos, em lista ou círculo definido por centro/normal/raio/quantidade/semente. Normalização/base/trigonometria ocorrem apenas ao resolver a cena; pontos ficam no Vec limitado já existente. Nenhuma dependência ou fila nova.

O consumidor de feedback aceita erro de conversão centro/pés de até2mm por componente quando mobility está ativo, mantendo comparação exata no legado. Tick da decisão também deve pertencer à tentativa atual. Não relaxa sweep físico nem interpreta partial_reached como sucesso. Mantém contagem de visitas/falhas, prazo por etapa e cooldown quando todos os índices falharam. A receita descreve intenção, sem prometer que o motor encontra rota. Território, recuperação e passagens planas continuam incompatíveis com mobility.

Uma pista ou troca de objetivo suspende a patrulha e conserva o índice. O motor pode terminar o trecho aéreo comprometido antes de trocar o destino; isso não conta como visita se a rotina já foi suspensa. Retomada inicia novo prazo. wait_ticks é tempo até o próximo ponto, não imposição de velocidade zero: voador sem hover conserva órbita validada. Para saltos/planeios, aterrissagem exige apoio, velocidade vertical baixa e segmento corpo→endpoint livre antes de encerrar o trecho. O solver pode precisar de alguns ticks para resolver penetração rasa no contato; publicar chegada antes disso causava falso unreachable no planejamento seguinte. Sem teleporte ou tolerância de colisão ampliada.

Cena/replay24 habilita patrulha com mobilidade espacial; cenas antigas permanecem válidas sem essa combinação. Schema MCP deriva do mesmo enum, upsert/edit_scene validam antes de mutar e inspect_patrols expõe pontosXYZ. Guias preservam Y para as novas rotas; reset/load/undo recompilam estado/rotas. Replay requer mesmo build. Não inclui rampas/navmesh, agendas, necessidades, território espacial ou orientação de avatar.


## Apoio contínuo em superfícies modulares (EAF-34)

O motor espacial exigia que uma única caixa fixa contivesse o footprint inteiro nas duas extremidades de cada link/conector walk. Esse fast path é correto porque a caixa é convexa, mas rejeitava pavimentos contínuos construídos por peças distintas. A nova consulta walk_support é comum à compilação/lançamento de links, conexões locais e apoio pontual/aterrissagem. Não altera o navegador plano legado nem o controlador dinâmico Rapier.

Decisão: cobertura da área varrida do footprint pela união de retângulos de apoio, sem malha/navmesh nova. Alternativas rejeitadas neste recorte: amostrar centro/cantos ou passos discretos pode ignorar buracos; usar apenas a AABB do trajeto é conservador demais para caminhos diagonais. Uma translação linear de um retângulo produz um polígono convexo. Particionamos X nos limites dos retângulos de apoio e nas quatro mudanças de inclinação do footprint varrido. Dentro de cada faixa, superfícies ativas são constantes e os limitesZ do polígono são lineares. Sua extensãoZ deve caber na união contínua dos intervalos de apoio; uma lacuna reprova o trecho. Sem resolução de grade que engula pequenos buracos.

Fast path conserva teste de uma caixa. Na consulta composta, o footprint encolhe pela margem de borda existente1cm; expandir cada piso por essa margem fecharia gaps de até2cm, portanto foi evitado. Topos devem estar dentro da tolerância de contato6cm em relação às duas extremidades; clearance físico permanece obrigatório. Isso não autoriza subir diferenças de altura: obstáculos ainda bloqueiam o sweep, e planos inclinados ainda não existem na autoria. Corpo dinâmico/visual não sustenta a consulta. Exigir todo o footprint é uma política conservadora existente, não uma simulação biomecânica de pés/centro de massa.

Buffers fixos para64 superfícies potencialmente relevantes e132 cortes, sort_unstable in-place. Custo composto O(S log S + S²) com S≤64, mais scans das entidades existentes; nenhuma alocação por consulta. O filtro considera a AABB do trecho antes da análise exata, podendo atingir o limite por superfícies fora do polígono. Exceder64 rejeita conservadoramente com walk_support_limit nos links; não tenta escolher subset dependente da ordem. Fast path continua disponível em cenas com mais peças quando uma delas basta. Sem cache novo e sem prometer escala/FPS medidos.

MCP existente fornece entidades, rede, conectores, inspeção de rejeições e histórico. walk_support_gap/height_mismatch/limit nos links distinguem suporte de outras restrições físicas; no_local_route permanece agregado para conexões locais. Cena/replay24 não mudam de formato; comportamento é identificado pelo build. Demo de passagens gerada por parâmetros no cliente MCP; resultado permanece editável sem o gerador.

### Decisão estrutural ainda aberta para rampas/degraus

Apoio modular remove a dependência de uma única peça, mas não representa inclinação. Hoje renderização, ColliderBuilder::cuboid, picking, oclusão visual/acústica e clearance assumem caixas alinhadas aos eixos. Acrescentar somente altura interpolada à navegação faria o planejador discordar da física e da imagem. A próxima mudança deve definir conjuntamente geometria visível, colisor e consultas sobre forma/normal.

| Alternativa | Vantagem | Custo/limite a discutir |
|---|---|---|
| Caixas orientadas | Reutiliza primitiva simples e permite inclinação | Rotação precisa entrar em todos os subsistemas; topo de uma caixa inclinada não é uma rampa maciça com base horizontal. |
| Formas convexas, começando por cunha | Rampa volumétrica explícita; colisão e consultas podem compartilhar forma | Exige contrato de geometria, render/cache/picking e revisão do clearance; não resolve malhas arbitrárias. |
| Malha geral/heightfield | Base mais ampla para terreno e conteúdo importado | Escopo maior de assets, colisão, consultas, validação e navegação; não deve entrar como dependência implícita de um NPC. |

O controlador também precisa de decisão própria: preservar corpos dinâmicos com regras de chão/inclinação ou introduzir controlador cinemático com shape casts, limites de inclinação e degraus. Isso afeta gravidade, impulsos, empurrões, transições aéreas e replay. Nenhuma dessas alternativas foi incorporada neste incremento. Discutir contrato e experimento técnico antes de escolhê-la; não apresentar a composição plana como rampas concluídas.

### Experimento EAF-35 concluído

[Estudo reproduzível de controlador](locomotion-controller-study.md):11casos×3motores, incluindo cunha20°/40°, degraus20/50cm, teto, parede, queda e interação dinâmica. O controlador cinemático do Rapier0.30.1 passou os limites configurados; o dinâmico atual sobe40° mas não supera20cm e sobrescreve impulso horizontal no comando seguinte. Empurrão cinemático exige transferência explícita; a implementação da biblioteca aloca manifolds nas consultas de chão/impulso. Recomendação: geometria tipada compartilhada caixa/cunha e motor de personagem opt-in; preservar legado. Ainda proposta, sem mudança de src/formato/replay ou promessa de navegação integrada. A decisão deve incluir apoio parcial, projeção de velocidade e transições aéreas.

## Comunicação de pistas por rádio (EAF-36)

Rádio é um componente opcional do NPC, separado da audição e da escolha de objetivos. Um relato contém a identidade do emissor e do alvo, posição observada e instante original. Todos os relatos são coletados a partir da visão confirmada do tick anterior, antes de atualizar qualquer receptor. Essa latência de um passo elimina dependência da ordem de atualização e impede propagação instantânea. Alternativa de entrega imediata exigiria uma segunda fase global de sensores; foi evitada neste recorte. Não há consulta à posição atual do alvo para montar a mensagem.

Canal, alcance XYZ, intervalo, idade máxima e permissões de envio/recepção são configuráveis. Alcance efetivo é o menor entre os dois rádios; aliança deve existir nos dois sentidos. Rádio atravessa paredes por definição desta política. Gritos serão outro meio, com regras acústicas próprias; este componente não emite áudio nem simula rede multiplayer. Som, relato recebido e conhecimento onisciente nunca alimentam a transmissão. Não há retransmissão, fila pendente nem mensagens guardadas para um aliado que entre depois no alcance.

A memória recebida tem fonte report, diferente de vision e sound. Não avança confirmação visual, observed_this_tick ou heard_this_tick; a condição evidence a reconhece, visual_memory e sound_memory não. O seletor de objetivos pode investigar, fugir ou continuar esperando conforme suas regras. A identidade transmitida habilita metadados estáticos de corpo já presentes no catálogo, não rastreamento oculto. Receptor sem alvo explícito aceita identidades reportadas; hostile_nearest exige relação hostil com esse alvo. Alvo explícito e território também filtram antes da seleção.

Política inicial conservadora: visão/som direto e memória direta ainda válida têm prioridade sobre rádio, mesmo se o relato for mais novo. Entre relatos, vence a observação mais recente; empate usa ID lexical do emissor. Um relato igual/mais antigo que a memória ativa não a renova. Expiração usa o instante original e o menor limite de idade entre emissor/receptor, além dos budgets de memória/busca/viagem existentes. Um novo avistamento confirmado pode renovar a investigação; repetir a mesma pista não pode. Confiança é recência limitada, não probabilidade calibrada. Mantemos uma memória ativa, sem fusão probabilística ou reputação.

Até oito NPCs e nove slots de atores, array fixo por tick e um recibo por NPC. O(N²) limitado, distâncias ao quadrado e matriz de relações já compilada; nenhuma alocação por mensagem, serialização ou fila dinâmica no caminho de entrega. Inspeção converte índices para IDs e expõe origem, idade, prazos, razões e contadores. Reset/edição/load/undo reconstruem o estado; replay inclui os recibos e exige mesmo build. Cena/replay25, leitura de cenas antigas sem ativar rádio; não há dependências novas.

A integração de rampas/controlador estudada em EAF-35 ficou para retomada posterior por direcionamento do usuário. Este incremento avança percepção/conhecimento e não encerra NPCv1/M09.


## Habilidades — proposta anterior à implementação (EAF-37)

O [estudo de habilidades](ability-runtime-study.md) compara o runtime real do LandOfChaos com as necessidades da Ignis. Recomendação: definição tipada compilada, solicitações compartilhadas jogador/NPC/MCP, execução por ticks, custos múltiplos planejados antes do débito, entrega separada dos efeitos e recibos com resultado aplicado. A referência está identificada no inventário; sua auditoria de autoria não comprova execução de gameplay.

Inteiros com escala para recursos, cobrança no commit e uma execução em cast por ator são propostas com tradeoffs explícitos, não decisões incorporadas ao schema. O primeiro recorte recomendado usa ações diretas não combativas, sem assumir vida/morte, grafo/eventos, projéteis ou ABI externa. Preservar privacidade dos sensores e ordem de execução/replay faz parte do contrato. Consultar o estudo antes da implementação; nenhum módulo novo ou migração de formato nesta unidade.


## Habilidades diretas compartilhadas (EAF-38)

Após autorização para prosseguir com o recorte A, adotamos definições tipadas compiladas, i64/escala para saldos, cast único por ator e débito/cooldown no commit. [Contrato implementado](abilities.md) detalha as decisões propostas em EAF-37. Cena/replay26. Nenhuma dependência, VM, ABI pública, vida/morte ou entrega física adicionada.

Catálogo e grants são compilados na edição; executor recebe intenções e entradas tipadas. Plano limitado calcula débitos/efeitos antes de mutar, permitindo falha atômica e recibos com deltas reais. A cópia de uma matriz9×32 i64 por tentativa é um custo consciente para atomicidade simples neste limite. Não transportar esse limite/layout como ABI. Sensores e intenção precedem commits; fases e IDs ordenados definem disputa determinística. Ruído gerado só é percebido no próximo passo. Dados de recursos não são JSON no tick; geometria ainda consulta os BTreeMaps existentes. Extensões futuras precisarão respeitar o mesmo orçamento/commit/resultado.

Comandos de autoria, ativação, cancelamento e inspeção passam pelo dispatcher MCP; teclas1/2/3/X chamam esses comandos. Barras usam o cache visual existente e toggleJ já exposto no MCP. Replay preserva fila pendente, casts, saldos, cooldown/retry, recibos e sinais regenerados. Edição/reset recompilam; não existe savegame ainda. Definição imutável durante partida evita versionar grants ativos neste primeiro incremento.


## Vitalidade e resolução sequencial (EAF-39)

Após alinhamento explícito, adotado componente vital opcional independente de grants, com um pool i64/escala e morte terminal no recorte atual. [Contrato](vitality.md) define Damage/Heal positivos, saturação/recibos e planejamento junto dos custos. Tradeoff: copiar9 estados vitais por execução facilita atomicidade e causa auditável; não há abstração ABI nem callbacks reentrantes. Inteiros preservam semântica de unidades, sem prometer determinismo binário da física entre plataformas.

Morte avaliada ao final do plano, preservando dano seguido de cura dentro da mesma execução. Entre execuções, ordem por fase/ID já existente resolve conflitos; rejeitada a alternativa de mortes simultâneas por lote neste incremento. Casts do morto cancelam imediatamente, reservas antes da física, motores desativados e gravidade/inércia mantidas. Morte não altera documento nem remove corpo. Estado terminal simples deixa ressurreição/respawn como contrato futuro explícito.

Estado global do mundo e conhecimento são separados: morte oculta não apaga pista; visão confirmada registra identidade morta e bloqueia relato antigo. Diagnóstico causal não vira percepção do agressor. Cena/replay27; autoria/inspeção/efeitos pelo dispatcher MCP, barras/cor e capturas pelo mesmo runtime. Sem dependência nova, mudança de controlador ou GUI exclusiva.

## Intenção de habilidade e aproximação (EAF-40)

`ability_policy` filtra objetivos a partir de dados próprios e de uma observação confirmada corrente; não aplica efeitos nem consulta coordenadas globais. Política opcional no objetivo permite perfis distintos com o mesmo grant. Limiar vital em pontos-base e comparação inteira, custo no catálogo compilado, alvo único da percepção e máscara por objetivo (não por grant) preservam decisões independentes para dois objetivos da mesma habilidade.

Alternativas discutidas: condição na definição impediria uso deliberado pelo jogador; condição só por grant misturaria prioridades/limiares distintos; nova árvore/planejador duplicaria contratos já existentes. Adotada extensão opt-in, mantendo o executor transacional como última autoridade. Observação visual autoriza avaliar saúde neste recorte, uma escolha explícita que jogos com lesões ocultas precisarão estender. Nenhuma consulta de saúde através de som/rádio/memória.

A intenção `approaching` entrega NavGoal conhecido, alcance XYZ e margem de entrada para os motores atuais. Ao perder visão a elegibilidade cessa e o fallback autorado pode investigar a memória. Diagnóstico separa intenção de caminho executável. Cast em andamento mantém elegibilidade, mas mudança de condição/alvo cancela; range/oclusão revalidam no vencimento. Custos e saúde da decisão precedem os commits do tick, portanto cura concorrente pode exceder o necessário; não houve reserva implícita de cura. Detalhes em npc-ability-policy.md.

Cena/replay28; campos tipados/Copy, array8 decisões por NPC e inspeção de IDs na fronteira fria. Sem dependência, thread, callback, varredura de novos alvos ou raycast duplicado. Limites existentes e contratos físicos de travessia permanecem. Testes determinísticos/MCP e performance estática acompanham o recorte; não constitui framework geral de comportamento, combate ou escala de multidões.


## Política de movimento por habilidade (EAF-41)

Decisão autorizada: três modos na definição compilada, free/interrupt/lock. Dois booleanos independentes permitiriam combinações contraditórias; enum expressa a política válida. A configuração é da habilidade, não uma regra global do jogador. Mesma instância/executor aplica política a comandos externos e objetivos NPC. Cena/replay29; default free mantém cenas anteriores. Contrato detalhado em cast-movement.md.

Lock deriva da máscara de casts ativos: não há contador persistente de bloqueios para vazar após cancelamento/falha/morte. Entrada física gravada permanece intacta, motores recebem a restrição no tick; sensores e solver continuam. Patrulha suspende e navegação de caminhada replaneja após liberação. Travessias aéreas/órbitas comprometidas recusam lock, pois congelá-las violaria o contrato de trajetória; implementar frenagem/aterrissagem exige decisão própria. Hover estacionário usa o comportamento existente.

Interrupt combina intenção e movimento XYZ resolvido, com tolerância explícita para jitter. Apenas velocidade física permitiria iniciar contra uma parede e só cancelar depois de andar; apenas input ignoraria quedas/empurrões. Custo: amostras fixas de9 atores e três checagens limitadas por tick. Entrada cancela antes do commit, motor antes do solver e deslocamento depois; efeito já comprometido não sofre rollback retroativo. Não se mudou a ordem global de física/efeitos por esta política. Sem novos callbacks, dependências ou serialização no tick.


## Entrega independente da execução — EAF-42

Continuidade autorizada após EAF-41. Alternativas apresentadas: corpo rígido por disparo versus volume de gameplay com sweep. Escolhido sweep de caixa alinhada por tick, sem corpo/impulso Rapier novo, com direção XYZ congelada e pool64. Benefício: custo/memória limitados, proteção contra atravessar obstáculos finos estáticos e ausência de acoplamento entre vida do cast e vida do projétil. Limite: outros corpos usam snapshot atual; não é CCD relativo, aerodinâmica ou interação física entre projéteis. Gravidade/homing/ricochete/área exigem contratos posteriores.

`Delivery` opt-in na definição; direct preserva legado. Launch valida/cobra/cooldown, Impact reutiliza plano de efeitos sem nova cobrança/consulta sensorial ao alvo pretendido. Primeiro corpo bloqueia; relação/elegibilidade separadas da geometria. Recorte aceita operações de alvo (vida/recurso); efeitos de origem/ruído por fases foram adiados explicitamente para não inferir comportamento após morte do emissor. Previsão do efeito feita no lançamento valida componentes, mas seus resultados não são gravados no mundo nem reutilizados no impacto. Energia paga não é reembolsada por erro posterior.

Estado em Vec pré-alocado64, recibos256 em anel, compactação estável em uma passagem; IDs/slots tipados. Sweep linear nos corpos atuais, desempate por ID; sem dependência nova/BVH especulativa. Avanço após cancelamentos explícitos e antes de casts/intentos novos, em ordem de lançamento; não reordena disparos por fração de contato. Morte por impacto usa cancelamento/motores existentes. Cena/replay30 inclui voos; renderer compartilha posições/extensões com inspeção MCP. Contrato e demonstração em projectiles.md.


## ADR — Mira espacial explícita (EAF-43)

Adotado no recorte de habilidades após EAF-42. `Selection::Spatial` e `Aim::{Direction,Point}` distinguem intenção de vítima efetiva; cast não precisa de um ator alvo fictício. `aim_ability`/`aim_ability_at` preservam API antiga e convergem no executor existente. Cursor resolve em comando para ponto XYZ congelado, log independente de viewport. Origem atual no commit, alvo ausente até contato; custo valida fonte, efeitos validam vítima no impacto. `range=0` para spatial, viagem limitada por max_distance.

Alternativas: sobrecarregar target com identidade/posição produziria combinações ambíguas e quebraria consumidores; interpretar ponto como endpoint alteraria a entrega retilínea. Um novo executor duplicaria atomicidade/cancel/replay. A opção adotada aumenta o payload fixo do cast/log, mas conserva buffers limitados, sem entidades/alocações por mira. Objetivo automático espacial rejeitado até definir sua fonte de evidência; comando explícito não lê ator oculto. Contrato/limites e demo em docs/free-aim.md.


## ADR — Ponto observado como origem da mira automática (EAF-44)

Política de objetivo `spatial_aim: observed_point` fornece alcance de decisão separado do alcance físico. O executor captura observação confirmada atual em Aim::Point e proveniência Copy (ator/tick/fonte), sem transformar identidade em rastreamento. Perda de visão/identidade/alcance/condição cancela; commit revalida fonte e candidato antes de pagar. Aproximação usa alcance efetivo da política e motores existentes. Comando espacial explícito segue independente de percepção.

Recalcular até lançar já é o modo actor; previsão exigiria histórico perceptivo e modelo temporal; memória/som exigiriam política de confiança/idade/compromisso. Optamos pelo ponto confirmado congelado para distinguir decisão, evidência e entrega sem criar outro executor. Um enum de estratégia na política permite extensão futura sem reservar nomes mágicos de habilidade. Fonte onisciente autorada é identificada, não apresentada como visão normal. Detalhes e limites em docs/npc-aim.md; sem dependências novas.


## Agenda de efeitos periódicos (EAF-45)

Entrega e duração são fases distintas. Uma agenda opcional na definição reutiliza a lista de efeitos target-only; instanciação direta/impacto, pulsos transacionais, refresh por origem/definição/destino ou instâncias independentes. Primeiro pulso atrasado; custo único e vida da origem desacoplada. Escolha mantém executor tipado em vez de callbacks/timers livres; limita o primeiro recorte a listas uniformes, sem buff catalog, dissipação, canalização ou modificadores. Pool64 e anéis256, ordem estável e clocks da simulação; nenhuma identidade/localização oculta fornecida à decisão NPC. Alternativas, semântica de morte/reaplicação e controles em [periodic-effects.md](periodic-effects.md). Evolução de efeitos mistos/atributos exige nova decisão explícita.


## Dissipação transacional por categoria (EAF-46)

Uma categoria opcional por definição periódica, nomes autorados compilados em índice u8 (até64), e operador dispel com destinatário/limite/require_match. Alternativa de catálogo geral de buffs/tags adiada: classificação única e máscara u64 no plano existente cobrem antídotos/remoção de regeneração sem callbacks ou novo scheduler. Máscara seleciona ordem estável, exclui duplicações entre operadores e só compacta no commit; custo/vida/recursos/remoção são atômicos. Pulsos não contêm dispel, evitando mutação reentrante da agenda. Revalidação em fim de cast e vítima real no impacto; ordem dos pulsos precede dissipação, cancelamento não remove efeitos instalados. Percepção/controle compartilhados, nenhum conhecimento oculto novo; detalhes e limites em [dispel.md](dispel.md).

## EAF-46 — proteção explícita contra dissipação

Decisão conjunta: booleano `dispellable` na definição periódica, default true; false requer cena35. Categoria não codifica imunidade. Preferido a categorias artificiais de veneno protegido (misturariam classificação e regra), tags/resistências gerais (contrato maior) ou flag por instância (estado redundante neste recorte). Todas as aplicações de uma definição compartilham a proteção; variações exigem definições distintas.

Runtime compila um bool por definição e consulta no scan existente antes de contar/selecionar instâncias. Sem nova coleção ou estado por instância, sem mudança nos pulsos ou cleanup. Require_match considera apenas removíveis; atomicidade/custos/casts/impactos/NPC usam o mesmo caminho. Inspeção de grants/agendas expõe o bool; autoria MCP e persistência compartilham schema. Cena/replay35; cenas antigas assumem true e normalizam, replay exige versão/build exatos como antes. Proteção só impede dispel, não morte/expiração/refresh/reset.

## EAF-47 — canalização como fase da execução

Continuidade de habilidades: channel opcional na definição, com intervalo/pulsos/cost_mode explícitos, validação antes de mutar e cena/replay36. Preparação → entrada → pulsos → término/cancelamento usa o Cast existente; nenhuma agenda independente disfarçada de execução sustentada. Estado Copy com prazos/contador, mesma ocupação por ator/cancelamento/movimento/percepção/retry.

Alternativas/tradeoffs em [channeling.md](channeling.md): custo único on_start ou per_pulse, cooldown na entrada, atraso inicial e falhas não desfazem transações anteriores. Sem nova dependência/ABI/grafo/scheduler. Separar validação de execução do planejamento evita copiar saldos/vida a cada tick entre pulsos; visibilidade/alcance continuam consultados a cada tick para interromper prontamente. Barra ciano deriva de prazos numéricos do Cast, sem busca de nomes no render. Limite de execução/pulsos e tamanho fixo maior documentados em performance.md.

## EAF-48 — interrupção externa como efeito transacional

Operador interrupt target-only no executor existente, direto/projétil, require_active explícito; interruptible defaulttrue compilado por definição. Não reutilizar cancel_ability de ferramentas como ação de gameplay: ele não valida custo/percepção/alcance do responsável. Não reutilizar dispel, pois agendas entregues e execuções ativas têm ciclos diferentes.

Plano guarda um ID opcional de execução do único destinatário, exclui duplicação e só muta no commit conjunto. ID é reconsultado ao concluir preparação/impacto, coerente com seleção por ator, sem reserva de alvo. Recibos correlacionam ambos os lados; barreira de um tick impede reinício em intenção já coletada, com retry NPC separado. Alternativa de status geral de silêncio/controle ampliaria indevidamente o recorte; não adicionamos scheduler, dependência ou ABI. Tamanho fixo maior dos recibos é custo consciente documentado. Sem reversão de pulsos/lançamentos passados; ordem sequencial anterior preservada. [Contrato, tradeoffs e demo](interrupts.md).

## EAF-49 — registro extensível de atributos e contribuições temporárias

Decisão alinhada com Guilherme: chave/operação/valor, sem enum de nomes de atributos. Primeiro registro bool/número fixo, base por entidade, modifiers genéricos com prioridade/tags/ID de habilidade; consumidores iniciais de interrupção, dissipação de agendas por autoria e dano recebido. Registro aberto não é reflexão sobre todos os campos Rust: cada sistema deve consumir seu valor efetivo e tratar invariantes/invalidação. Outros tipos/adaptadores permanecem posteriores.

Alternativas: mutar/restaurar configuração produz conflito entre buffs sobrepostos; callbacks/JSON por frame dispersariam validação e replay; cache de resultados condicionado por todas as tags/IDs exigiria invalidação mais complexa. Escolha mantém bases imutáveis e regras compiladas por chave, com avaliação no evento, limites explícitos e contribuições tipadas. Prioridade/ordem autorada/instância resolvem composição; números fixos arredondam em direção a zero, clamp por operação. Estado do início da transação define efeitos atuais; buff aplicado/removido afeta execuções posteriores. Expiração antes das ações, pool64/anel256, sem scheduler/dependência nova. Contrato completo, limites de custo e demo em [attributes.md](attributes.md).


## EAF-50 — classificação e composição de dano

Contrato aprovado: o jogo registra tags, cada componente tem sua quantidade/contexto e o efeito soma resultados antes de alterar vida uma vez. Escolhidos pacotes `damage_components` com até8 parcelas totais por habilidade, em vez de transformar parcelas em efeitos independentes ou embutir enum de elementos na engine. Tags de habilidade continuam para proteções de cast/agenda; dano usa tags próprias. Registro explícito em cena39 permite rejeitar erros de autoria; migração do contrato38 preserva o escalar classificado como um componente.

Arredondamento por componente, teto do total e overkill separados são explícitos. Mantidos snapshot inicial de buffs e commit transacional existente. Decomposição cabe em array fixo compartilhado por recibo; custo fixo adicional do anel em troca de observabilidade, sem alocação no cálculo. Sem callbacks novos por componente nem consumidor de dano causado nesta rodada. Contrato/limites/roteiro em docs/damage-components.md.

## Dano causado e recebido (EAF-51)

Quarto consumidor nativo por chave: combat.damage_dealt.multiplier no autor original. Mesmas regras compiladas e snapshot de buffs por transação. Fórmula i128 original×dealt×received/SCALE² com arredondamento único por componente; soma bruta<=10¹⁶, teto vital preservado. Avaliar dinamicamente no impacto/pulso permite buffs recentes e sua expiração afetarem entregas; congelamento por lançamento é alternativa futura explícita. Sem armazenamento ofensivo extra por projétil/agenda, sem cache no tick. Recibos fixos ganham um i64 por componente. Cena/replay40,61 customizados preservados (65 totais), colisão legada com nova chave reservada falha exigindo renomeação para evitar ativação silenciosa. Contrato em outgoing-damage.md.

## EAF-52 — seleção de área e transações individuais

Decisão: `area` opcional no contrato de habilidades, com seleção física no instante da entrega. A seleção não vira percepção de NPC. Centros XYZ, raio/limite/relação/oclusão, ordenação distância²/ID; custos únicos e executor existente por destinatário. A alternativa de um plano coletivo com rollback de todos os alvos foi preterida: faria uma ausência de recurso ou pool cheio em um alvo cancelar efeitos válidos nos demais e ampliaria o plano/coordenação de reservas. Falhas individuais e efeitos sequenciais são explícitos no MCP; buff/morte de uma transação pode alterar a próxima.

Oclusão radial consulta interiores de corpos fixed/dynamic, independente de visão/visibilidade e sem atores como escudos. Coordenada espacial explícita não implica teste de visão para colocação; NPC automático continua exigindo evidência. Custo deliberado: até9 seleções/transações, scan de corpos e ring64 extra; não introduz índice espacial/ECS/ABI. Guias reutilizam linhas/cache e círculo unitário. Cena/replay41, sem dependências novas. Contrato completo em [instant-area.md](instant-area.md).

## EAF-53 — volumes fixos com amostragem por pulso

`area.persistence` separa a vida de um volume da agenda por destinatário. Centro congelado na entrega, ocupantes consultados a cada pulso, duração exclusiva e custo único. Alternativa de manter conjunto contínuo de ocupantes/enter/exit foi adiada: adicionaria semântica de travessia e revogação de efeitos que não é necessária para zonas pulsantes. Auras móveis permanecem outra capacidade. Reutilizar a seleção/executor preserva transação individual, atributos atuais e privacidade dos NPCs.

Refresh por autor/definição substitui centro e prazo; independent permite sobreposição. Rejeita-se chave espacial aproximada: introduziria tolerância implícita e agrupamento difícil de inspecionar. Pool16, ring128 e prazos u64; capacidade direta antes de pagar e no impacto do projétil sem reserva durante voo. Isso aceita impacto pago sem zona quando o pool enche, com falha explícita. Morte do autor configurável; pulso comprometido termina, próximos podem ser suprimidos. Cena/replay42, mesmo dispatcher/clock/render, sem dependências. Contrato e ordem em [persistent-area.md](persistent-area.md).

## EAF-54 — âncora física e fases de auras pulsantes

Área persistente admite anchor fixed/source, default fixed. Source se limita à entrega direta self_actor: aceitar Point/projétil/terceiro sem contrato de identidade faria argumentos perderem significado. A extensão reaproveita pool, relógio e executor; não acompanha membros continuamente nem revoga buffs ao sair. Política de morte existente decide encerrar ou acompanhar corpo morto.

Centro usado no pulso vem do snapshot pré-física e permanece no recibo. Centro vivo é atualizado após física para inspeção/guia, sem selecionar ocupantes novamente. Alternativa de atualizar apenas a cada pulso deixaria visual/diagnóstico atrasados; callback contínuo de membership adicionaria custo e semântica desnecessários. Até16 lookups por handles já existentes, nenhuma coleção/snapshot adicional; cache de linhas anterior. Cena/replay43 e MCP antes da UI. Contrato em [auras.md](auras.md); EAF-53 recebeu aceite humano.

## EAF-55: apoio independente da memória de combate

Política opcional por objetivo actor, evidência visual/reconhecimento de aliados em arrays limitados e ranking determinístico após elegibilidade. Alternativa de reutilizar/substituir alvo de perseguição rejeitada: curar não deve apagar memória do inimigo ou publicar aliado por rádio. Trilhas independentes evitam dupla confirmação; aceitamos até9 consultas sensoriais adicionais por NPC opt-in em troca de isolamento, sem cache espacial novo. Cast fixa destinatário enquanto válido; revalidação transacional de limiar precede custo. Cena/replay44; contrato/limites em [ally-triage.md](ally-triage.md).

## EAF-56 — filtro de necessidade por dissipação

Política require_dispel_match opt-in reutiliza o seletor do plano transacional, após visão própria/confirmada e antes do ranking. Duas máscaras u64 mantêm limite e ordem de operadores/periódicos/buffs; proteção efetiva não é duplicada. Alternativa de simular toda habilidade na decisão rejeitada: exigiria prever mutações/capacidades de operadores sem necessidade para este filtro. Sem cache/reserva coletiva; scans limitados aceitos e revalidação no commit obrigatória. Actor/self_actor, direto/projétil sem área/channel/periodic; cena/replay45. Contrato em [dispel-triage.md](dispel-triage.md).
# Proposta de ciclo de vida — EAF-57

Estudo em [lifecycle-study.md](lifecycle-study.md), baseado no runtime de d590930. Ressurreição do mesmo indivíduo, nova instância de população e reset do laboratório têm contratos distintos. Recomendação: contador de vida para ressuscitar atores existentes; handle com geração antes de reciclar slots em spawn/despawn. Memória de morte precisa de nova evidência, e entregas persistentes mantêm autoria própria. Alternativas, fronteiras de transação/limpeza, custos e matriz de aceitação documentados. São propostas para alinhamento técnico, não schema ou migração aprovados; runtime/cena/replay45 inalterados.

# Ressurreição do mesmo indivíduo — EAF-58

Aprovado o primeiro recorte de EAF-57: efeito direto explícito no mesmo corpo, energia/cooldowns preservados e reativação sem teleporte. Contrato em [resurrection.md](resurrection.md). Catálogo fixo mantido; life_index u64 e stamps em entradas/casts/evidências distinguem vidas sem introduzir handles de população. Fonte/custos/efeitos atômicos; morte/retorno no mesmo tick consomem limpeza e reativação. Ressurreição compõe apenas recurso próprio/ruído além do efeito no alvo neste recorte.

Sondar corpos conhecidos mortos acrescenta consultas limitadas, mas evita o atalho incorreto de limpar crenças globalmente. Nova vida reinicia reconhecimento; relatos antigos não superam evidência mais recente. Corpse_visual separado permite alvo explícito de NPC sem transformar cadáver em alvo de perseguição/triagem de vivos. Projéteis/zonas sobreviventes selecionam contato/ocupantes correntes e preservam autoria dinâmica. Contadores/stamps e anel256 aumentam memória limitada; sem ECS/dependências/threads novos. Cena/replay46. Spawn/despawn/teleporte/triagem de cadáveres continuam posteriores.


## Identidade anterior à população dinâmica — EAF-60

Registro limitado com slot/geração e serial lógico; vida permanece separada. Referências numéricas Copy nos pedidos/casts, validação indexada antes de efeitos/custos e tokens escopados nos controles MCP existentes. Usar apenas slot ou ID autorado permitiria referência antiga atingir substituto; usar UUID/string em cada pulso ampliaria armazenamento/comparação sem necessidade. Escopo de transporte fica fora do replay, que usa identidades determinísticas.

A migração é incremental: catálogo ainda não recicla slots, embora o registro isolado teste reuso. Atributos/percepção/históricos e visão do mundo precisam migrar antes de spawn/despawn. Sem ECS/ABI/dependência nova; arrays/pools e limite9 atores preservados. Stamps ficam maiores em troca de validade temporal explícita, e catálogo inicial faz trabalho limitado de ordenação/inserção fora do tick. Contrato em [instance-identity.md](instance-identity.md), cena46/replay47. Crescimento exige atributos por dono/chave, planos locais e reserva antecipada de contexto de origem conforme [population-runtime-study.md](population-runtime-study.md).


## Vínculos persistentes — EAF-61

Referências Copy junto dos índices conferem identidade antes do uso. Buffs conservam dono/origem, observações e reconhecimento distinguem indivíduo/vida, rádio e autoexclusão sonora usam referências congeladas, históricos não resolvem aliases pelo ocupante novo. Alternativa de limpar toda memória na remoção rejeitada por revelar ausência oculta; alternativa de guardar strings/Scene em cada evento rejeitada pelo custo e por não validar geração. Alias ausente pode ser null com ID lógico preservado. Recibos conservam definição própria. Pools/aneis continuam limitados; structs maiores e comparações escalares são custos explícitos.

Contrato em [identity-bindings.md](identity-bindings.md), cena46/replay48. Testes usam reuso real do registro isolado; mundo vivo permanece fixo. Contextos de autor removido, reserva na admissão e integração de componentes/bindings ainda antecedem spawn/despawn; limites atuais não provam escala de população.


## ADR — autoria nativa de malhas (EAF-76)

Guilherme escolheu criação, escultura, rigging e animação nativos na Ignis em26/10/2026. O recorte usa assets triangulados/instâncias visuais separados de atores e corpos de gameplay, payloads imutáveis compartilhados por Arc, pincéis que preservam topologia, esqueleto genérico e clipes locais TR em ticks. CPU linear blend skinning e normais flat permitem verificação numérica; GPU skinning, remesh, IK, retargeting e root motion exigem incrementos próprios. O custo de bind/skin/expansão/upload por frame animado permanece limitado e explícito, sem afirmação de FPS.

Command/SceneEdit/MCP usam a mesma validação/edição atômica/histórico. Scene48 contém assets autocontidos e playback autoral; replay52 compara poses e prefixos. Edição reinicia simulação conforme contrato existente; não há comando oculto de troca de animação em runtime nem deformação física/sensorial implícita. RenderFrame mantém geometria preparada/cacheada e nenhum dado de gameplay adicional. Alternativas, coordenadas, operações, budgets e gates em [mesh-authoring.md](mesh-authoring.md). A pesquisa gráfica EAF-75 permanece independente.


## Autoria por plantas e preservação de superfície

Escolha do primeiro recorte: [plants.md](plants.md), perfil retangular cotado/extrusão e seis faces com coordenadas materiais canônicas. Documento construtivo separado de meshes::Asset e do runtime; geometria avaliada alimenta o renderer atual. Fonte canônica imutável compartilhada no histórico, reavaliada em cada mudança dimensional/amostragem. Edição direta explícita rebasa a fonte; regeneração não acumula transferências. Kernel puro [mesh-preservation.md](mesh-preservation.md) recebe correspondências fornecidas pelo gerador e adapta vetores/pesos/repouso/clipes por política.

Alternativas: um snapshot somente conserva trabalho; reprojeção por proximidade não distingue regiões próximas; cage demanda novos vínculos volumétricos; BRep/solver geral seria um contrato maior que o necessário neste ensaio. Reutilizamos Asset/dispatcher/revisão/lotes/undo/replay e mantemos formatos próprios de construção. Detach é explícito; reconstrução incompatível falha antes de substituir autoria/runtime. Cena49/replay53.

Custos mantidos: fonte e avaliado persistidos, reconstrução/validação síncronas limitadas na autoria, reamostragem linear por vértices e aproximação da escultura por malha canônica. Erro amostrado com tolerância não comprova fidelidade contínua. Nenhuma nova biblioteca, ABI, fase de gameplay ou ganho de FPS presumido.

## Perfis e seções no domínio material — EAF-81

O loft reto mantém IDs/ordem das estações, seção transversal e superfícies como
identidade; as posições físicas são resultado do mapa de volume. O kernel recebe
coordenadas materiais explícitas em vez de deduzi-las da posição/dimensão.
Isso permite alterar a silhueta e os cortes sem reinterpretar o relevo canônico.
Correspondência por proximidade foi descartada para evitar troca de lado na lâmina.
Um solver geral, frames por superfície e IK seriam contratos adicionais.

Origens de bind são invertidas no volume fonte e mapeadas no destino, com
orientação global conservada e chaves reconstituídas a partir dos deltas de
repouso. Não existe escolha implícita de twist; ossos fora do volume recusam.
Medição de retesselação separa o detalhe da alteração intencional da base.
O custo permanece na autoria; GPU/runtime continuam recebendo o Asset existente.
Contrato e limites em [plants.md](plants.md), cena51/replay55. Fonte/avaliado e
validação síncrona permanecem custos explícitos; sem promessa de FPS/CAD geral.
## Plantas curvas de seções — EAF-87

Cena56/replay60 acrescentam `cross_section: ellipse` e `Station.offset: [x,z]`
em metros locais, default zero. Contrato e alternativas em
[curved-plants.md](curved-plants.md). Y ordenado e escalas positivas permitem
inversa material analítica; planos XZ paralelos, interpolação linear entre
estações. Não é sweep XYZ com frame transportado. Elipses compartilham índices
nas grades materiais, sem seams por proximidade. Topologias retangular/diamante
legadas ficam idênticas. Definição/reference com dados novos exigem versão56.
O domínio explícito de preservação valida coordenadas normalizadas; apenas
o caminho affine deriva esse domínio da caixa física centrada. Fonte, material,
pesos e deltas TR usam os mesmos comandos/MCP/validação/transações existentes.

## Interpolação de seções (EAF-87)

Cena57/replay61 acrescentam sections.interpolation:pchip opt-in, Linear
default/serialização omitida. PCHIP interpola quatro canais em Yfísico por
cúbicas Hermite limitadas, coeficientes compilados uma vez por sample; nenhuma
dependência ou fase de runtime nova. Y e IDs/intervalos materiais continuam
lineares/ordenados; a inversa avalia as mesmas funções em Yconhecido. Target
ou referência PCHIP exige57, enquanto Linear conserva operações/resultado
legados. Trocar interpolação mantém fonte canônica/escultura/pesos/canais de
material e adaptação de bind/TR. Planos paralelos/orientações existentes são
preservados; C1 da função não garante suavidade com malha esparsa. Alternativas
e custos em curved-plants.md/performance.md; fonte/versões/capturas em
validation.md. Sem atribuir realismo ou ganho de FPS aos testes.

## Hierarquia visual de meshes (EAF-89)

Cena58/replay62 permitem Instance.attachment opcional por parentInstance e
osso nomeado. Sem vínculo, TRS mundial e resultados legados mantidos; com
vínculo, TRS local ao pai/osso. Instâncias de armazenamento arbitrário formam
grafo acíclico, escala composta0.001..100 e dependências validadas no commit.
Rig/clipe/material do filho independem dos do pai. Não há parentesco de atores,
colisão/percepção ou solda geométrica implícitos.

Hierarchy compila IDs/nomes/ordem em arrays fixos ao invalidar Frame. Prepared
amostra skeleton uma vez por instância e compõe worlds compartilhados entre
vértices, pose e replay. Inspeção expõe world_transform, mantendo joints como
globais do asset. Materiais seguem coordenadas de repouso, renderer/sombra
consomem os vértices mundiais existentes. Consulta explícita compila preparação
limitada; Frame estático mantém retorno rápido. Nenhum Vertex/shader/dependência
novo. Contrato, alternativas e custos em mesh-attachments.md/performance.md.

## Ilhas e extração de meshes (EAF-90)

Consultas/autoria usam conectividade por índice de vértice, sem inferência
espacial. IDs de ilhas são o menor índice de face e seguem o mesmo scope/revisão
dos componentes. Não são identidade persistente. `meshes/extraction.rs` gera
partição e subsets ordenados, preservando espaço local/canais/pesos; não poda
ossos/clipes nem recentra o resultado. Move conserva geometria armazenada,
Copy aumenta seu custo. Ambos criam asset, nunca instância automaticamente.

Assets existentes já representam o resultado: Scene58/replay62 permanecem,
sem novo grafo/formato intermediário ou tipo de árvore. `preview_mesh_edit`
usa o mesmo `plants.apply_mesh` e validação integral sobre staging, sem publicar.
Move exige detach explícito de planta; Copy é snapshot independente sem rebase.
Referências de subdivisão/extração rejeitam fonte ou destino alterados antes
no lote. Contagens globais, inclusive clipes/chaves mantidos, precedem cópias
de payloads; transação publicada/undo único continuam pelo dispatcher existente.
Contrato e custos em [mesh-islands.md](mesh-islands.md).
Rig/socket da planta aproveita seu mapa de origens existente, sem inferir
orientação de tangente. O ensaio vincula somente um secundário/coroa da árvore.

## Detalhes vetoriais de superfície (EAF-91)

Material.detail opcional usa até32segmentos analíticos afunilados no planoXY/XZ/YZ
do canal3D de repouso. União por máximo, tinta multiplicadora e bump assinado
independentes do ruído existente. Campo filtrado pelo footprint, sem afetar
silhueta/colisão/sombra. Não há tipo de folha/árvore na engine. Material deixa
Copy e compartilha Detail por Arc; igualdade usa atalho por identidade antes
de conteúdo. Histórico/frame/extração/subdivisão/regeneração conservam camada.

No recorte original, Vertex52bytes, uniforme por slot1136bytes e tabela17slots19312bytes;
renderer converte só quando receitas mudam, além de manter cache do upload.
Até32distâncias por fragmento habilitado; ausência não avalia os segmentos.
Scene59/replay63 explicitam campo novo; formatos≤58 com camada são rejeitados
antes da migração. Sem dependência/textura externa/ABI de gameplay nova.

fit_material_coordinates ajusta AABB do canal existente (fallback repouso) ao
domínio alvo, por asset ou ilhas de índices compartilhados. Eixo constante e
isolados usam centro. Somente coordenadas materiais mudam; orientação livre ou
frame de fibra não é inferido. Em planta é edição de canal com rebase explícito
existente; set_material continua metadado e preserva fonte geométrica. Mesmo
dispatcher/preview/validação/undo. Contrato/tradeoffs em surface-details.md.

A aplicação às variantes foliares sem canal mostrou que fit não orienta peças.
copy_material_coordinates acrescenta transferência explícita por sequência de
vértices/triângulos local idêntica, inteira ou por ilha. Valida todas antes de
copiar o canal, sem nearest/inferência, posições/rig/clipes imutados. Não cria
vínculo de template persistente; resultado é canal já expressável no formato.


## Trajetória + seções normais — EAF-97

Definition.trajectory opcional descreve Bézier cúbicaXZ emYlinear crescente.
A seção existente é avaliada num frame normal à tangente, por rotação de menor
ângulo de+Y. Sem trajetória, geometria e serialização antigas são conservadas.
Escolha/alternativas e prova em plant-trajectories.md: margem analítica
conservadora faz F=(P-C)·C' estritamente decrescente em todo o domínio,
permitindo inversa única limitada sem nearest e recusando combinações
não certificadas. Não se trata de BézierXYZ livre/loops ou Frenet/transportegeral.

Correspondência/fonte canônica mantidas; bind origins mapeiam pelo volume e
vetoresXYZ/rotações globais/clipes seguem políticas existentes. Critério
surface_tolerance e relatório de erro lateral AMOSTRADO separados do erro de
reconstrução canônico. Controle/preview/inspeção via dispatcher/MCP existente,
sem renderer/Vertex/dependência nova. Fonte: native-curved-cylinder.plant.json.

Scene64/replay68 nesta linha local, reserva registrada após63/67funcionais.
Os schemas das linhas não foram unificados: números não atestam compatibilidade.
Main e cena da árvore não são modificados; demonstração isolada precede retorno
à árvore por pedido direto de Guilherme.
## Rugosidade e reflexo da luz direta (EAF-94)

Material.reflectance opcional {roughness,f0} no mesmo asset/set_material/preview.
Modelo dielétrico isotrópico GGX/Smith/Schlick, câmera atual e normal perturbada
existentes; difuso frontal atenuado pelo Fresnel e lobo branco sob a mesma sombra.
Ausência/F0zero conserva pixels legados. Sem introduzir metais, ambiente refletido,
IBL, HDR, texturas ou um domínio específico de árvore. Contrato/alternativas,
unidades legadas e limites em [dielectric-response.md](dielectric-response.md).

Vertex52B e tabela19312B mantidos, dois escalares antes ociosos recebem parâmetros;
Globals176B inclui posição de câmera. Igualdade de Material invalida cache quando
parâmetros mudam, sem conversão repetida por animação. Scene60/replay64 são locais
à branch de autoria; colidem com numeração funcional EAF-92/93/95 e precisam de
unificação explícita na integração. Não declarar compatibilidade entre branches
apenas pelo número, nem replay entre builds. UI segue adiada por Guilherme.

## Recortes e pontes de contornos reais (EAF-85/EAF-80)

`mesh_bridge` inspeciona ciclos orientados por índices reais, remove faces com
compactação explícita e constrói pontes cúbicas entre dois ciclos de mesma
contagem. Reutiliza os vértices das bordas, sem weld/nearest ou hipótese radial.
Condição tangente pelas normais de área e direções do loop; distribuição de
amostras concentra resolução nas extremidades. Jacobiano/orientação amostrados
e limite conservador de90° entre faces novas/adjacentes recusam dobras locais,
sem provar ausência global de autointerseção. Novos
pontos interpolam canais/pesos; antigos conservam valores exatos. Limites e
preflight antes da alocação, staging/referências/undo/preview compartilhados.
Plantas vinculadas recusam cortes/ponte sem detach explícito. Não é boolean,
remesh, solver automático de junções ou formato novo. Alternativas, hipótese
radial rejeitada e custos em [mesh-bridges.md](mesh-bridges.md).


## Adoção de trajetória na árvore

Contrato, alternativas, limites e revisão estática em
[tree-trajectory-integration.md](tree-trajectory-integration.md). Adoção explícita
conserva fonte/referência históricas; Scene65/replay69 locais. Draw131072, demais
tetos mantidos e consultáveis via inspect_meshes.limits. Junção regenerável
permanece uma etapa distinta; protótipos reprovados não fazem parte da API.

## União implícita de três portas (EAF-85)

Kernel puro mesh_junction_union e dispatcher/MCP recebem Asset recortado e
contornos reais, extraem união tubular guiada e conectam contagens desiguais.
Prefixo de geometria/canais/rig/clipes conservado; miolo explicitamente novo e
raiz única nas bordas. Query não muda documento ou simulação; Asset pronto
passa pela admissão existente na publicação. Sem tipo de árvore, Scene/replay,
dependência ou renderer novo. Forma alvo volumétrica substitui tentativas de
embedding por energia reprovadas, com domínio/grade/tetos explícitos. Métricas
de convergência não substituem orientação/topologia/limites/arte. Contrato,
tradeoffs e custos em [implicit-junctions.md](implicit-junctions.md).

## Filtro geral de superfície com pontos fixos (EAF-85)

`mesh_surface::smooth_region` é um kernel puro de acabamento por posição,
independente da união tubular e do pincel radial legado. Seleção/âncoras são
explícitas; fora da seleção e bordas reais ficam fixos. CSR de arestas únicas,
Jacobi limitado, envelope pós-f32 e guardas incidentes mantêm topologia/canais/
rig/clipes. Laplaciano foi escolhido como primeiro recorte verificável, com
encolhimento observado; não se adotou solve de embedding, remesh ou promessa
de volume preservado. Não une peças desconectadas nem corrige fonte curva por
inferência. Kernel/tipos exportados primeiro para integração coordenada;
dispatcher/MCP e persistência da operação continuam gates separados. A futura
integração deve preservar referências/rebase e autoria canônica de plantas,
sem gravar seleção efêmera como receita. Contrato/tradeoffs/checklist em
[mesh-surface-filter.md](mesh-surface-filter.md).


## Composição paramétrica com contexto — local66

Receita de3portas reavaliada de fontes imutáveis, sem usar saída como origem.
IDs de seção/contornos reais, raiz rígida explícita e aparência compatível;
metadados atualizam fontes e geometria direta recusa rebase. Preview de lote
compartilha staging do editor. Apenas asset8192v16384t; totais e draw mantidos.
Escolhas/revisão10pontos, custos e limitações: [plant-assemblies.md](plant-assemblies.md).
Scene66/replay70 locais; dados canônicos cortados ficam na fonte, sem falsa
transferência de escultura para miolo novo. Artgate e demais galhos continuam.

## Filtro geral de superfície com pontos fixos (EAF-85)

`mesh_surface::smooth_region` é um kernel puro de acabamento por posição,
independente da união tubular e do pincel radial legado. Seleção/âncoras são
explícitas; fora da seleção e bordas reais ficam fixos. CSR de arestas únicas,
Jacobi limitado, envelope pós-f32 e guardas incidentes mantêm topologia/canais/
rig/clipes. Laplaciano foi escolhido como primeiro recorte verificável, com
encolhimento observado; não se adotou solve de embedding, remesh ou promessa
de volume preservado. Não une peças desconectadas nem corrige fonte curva por
inferência. Kernel/tipos exportados primeiro para integração coordenada;
dispatcher/MCP e persistência da operação continuam gates separados. A futura
integração deve preservar referências/rebase e autoria canônica de plantas,
sem gravar seleção efêmera como receita. Contrato/tradeoffs/checklist em
[mesh-surface-filter.md](mesh-surface-filter.md).
## Acabamento por origem em plantas compostas

Scene67/replay71 locais de autoria acrescentam `context_lofts`,
`context_surface` e `junction_surface` opcionais. A ordem é fonte canônica,
loft de anéis, filtro de contexto, recorte, corpo da planta/união e filtro iso.
Máscaras de contexto referem-se somente à fonte; novos pontos herdam seleção
e pinos quando ambos os endpoints da correspondência os possuem. O filtro da
junção seleciona somente o domínio iso da avaliação atual. Nenhum índice da
malha composta avaliada é persistido como máscara. Recortar uma face refinada
exige outro domínio explícito e é recusado nesta entrega.

Fontes e referência continuam intactas. O acabamento é uma edição intencional
da forma avaliada, sem prometer conservação de volume ou poses idênticas dos
pontos movidos. Pesos/bones/clips e sockets retidos mantêm o contrato existente.
Escultura direta na composição continua recusada; SmoothRegion usa a mesma
proteção. Para malhas independentes, seleção escopada por revisão, preview e
publicação compartilham o dispatcher.

Armazenamento medido da árvore exige asset16384v32768t e total32768v65536t;
draw131072, grade/extração da união, RenderFrame e layout de vértice mantidos.
Formatos anteriores recusam camadas novas e orçamento expandido, também em fontes
canônicas. A integração da numeração com a linha funcional de main permanece
separada desta frente. Contrato, alternativas e limitações em
[organic-continuity.md](organic-continuity.md).

## Contexto com enxertos locais — Scene68/replay72

Escolhida composição explícita de fontes estáticas antes da bifurcação viva,
reutilizando a união3portas existente. Fonte filha integral e translação ao quadro
do contexto são persistidas; máscaras ALL expandidas pororigins BOTH, mapas
parciais canônico→retido e lookup do triângulo orientado sobrevivente evitam
referências efêmeras/nearest e rebake. Até8enxertos, mesmos tetos de geometria,
união e draw. Canonical sources incluem filhas nos budgets/IDs; aparência se
propaga sem rebase. Contrato em [context-grafts.md](context-grafts.md).
União/remalha geral é alternativa com outro contrato de detalhe/skin; este recorte
não implementa grafo recursivo nem ligação entre ossos diferentes. O primeiro
secundário estático cabe no domínio atual; transição de curvatura/casca ainda é
gate artístico. Nenhuma guarda foi relaxada para aceitá-lo.

## Continuidade local explícita — Scene69/replay73

Acrescentado `ContextGraft.collar_surface` posterior ao acabamento iso. A fonte
não é substituída; o novo modificador tem autorização explícita para remodelar
pontos retidos/guardas na sua região. Portas por ordinal e BFSmulti-source de
arestas únicas derivam seleção/pins; nenhuma máscara efêmera vai para o documento.
Defaults sem campo mantêm resultado anterior. Pins de portas não escolhidas e
verificação final das portas da assembly principal evitam deslocamento implícito
das interfaces protegidas. Relatórios separam união crua,iso e colar. Tetos,
renderer e guardas existentes mantidos. Alternativa de continuidade analítica do
campo fica aberta,sem substituí-la por normais de shading. Contrato em
[context-grafts.md](context-grafts.md).


## Primitivas de caixas chanfradas — EAF-121

Adotado gerador limitado `MeshEdit.chamfer_boxes`: centro/dimensões/orientação e
chanfro plano produzem Geometry comum; zero8v/12t, positivo24v/44t. Alternativas
de bevel por região e hull geral exigem contratos separados. Append sem rig
conserva canais/aparência; fonte de planta vinculada continua protegida. Sem
novo formato persistido/renderer/Vertex/Globals ou alteração no Sweep paralelo.
[Contrato/checklist](chamfer-boxes.md). Caixas visuais não alteram colisão.

## Varredura nativa de perfil3D — EAF-113

Adotado construtor `MeshEdit.sweep` e consulta pura `plan_mesh_sweep`, sem campo
persistente novo. Perfil convexo, cadeia Bézier aberta regular e estações de
escala produzem Geometry comum com canais materiais. O plano certifica erro da
linha central e contagens; preview admite também faces e conversão f32. Append
conserva asset antigo sem rig e evita reconstruir arrays no cliente. Loft por
anéis e plantas continuam ferramentas distintas; não generalizamos o contrato
de preservação de escultura/rig implicitamente. Alternativas de spline patch e
montagem persistente ficam abertas. O incremento original conservou Scene72/replay76;
a integração com ambiente usa Scene76/replay80, sem incluir a Wedge75/79.
Sweep não altera renderer/Vertex/ABI. [Contrato e custos](mesh-sweep.md).

## Seção e domínio dos traços — EAF-80/EAF-85

O estudo da árvore separou continuidade material, direção/escala do padrão e
seção da ranhura. `Detail.profile` escolhe banda legada ou seção radial suave;
`Detail.repeat` é domínio periódico explícito com suporte contido e footprint
anterior ao wrap. São controles genéricos da mesma receita, sem tipo de árvore
ou novo sistema de recursos. Renderer/uniforme usam detail_domain vec4:
1152 B/slot,19584 B por tabela17slots; Vertex64 B e Globals176 B mantidos.
Schema/preview/edição/undo/replay compartilham contratos MCP existentes.
Scene71/replay75 protegem os campos novos; frames/finisher/triplanar continuam
os contratos70 anteriores. Limites/custos/alternativas em detail-domains.md.


## Projetos e cenas no editor — EAF-109

Pasta com manifesto ignis.project.json1 e cenas do formato existente. Projeto
contém autoria e cena inicial, não estado do runtime/replay/layout. Service
conserva uma cena ativa e a raiz explícita; Create/OpenProject são comandos de
troca de raiz absoluta. New/OpenProjectScene protegem alterações não salvas e
instalam Engine validado com revisão monotônica e histórico próprio por cena.
Resolver existente continua confinando todos os arquivos de cena/artefatos.
Alternativas/checklist/limites em projects.md; não é empacotador/distribuidor.
Layout2 acrescenta painel Projeto, lê1 sem esse painel e migra ao carregar.
Session_revision invalida apresentação quando nome/caminho/estado salvo muda
sem edição do mundo. Nenhum documento/catálogo é clonado para montar widgets.

## Duas camadas de padrão — EAF-112

`Material.secondary_pattern:Option<Pattern>` reutiliza o contrato limitado de
Pattern em um segundo campo independente no mesmo domínio material de repouso.
Tintas multiplicam, alturas somam antes da reconstrução mundial do gradiente;
Detail/transmission/reflectance continuam independentes. Soma dos dois bumps
limitada a0.05m; não é deslocamento geométrico nem um grafo de materiais.
Defaults preservam pixels e omitem o novo campo. Scene73/replay77, inclusive
fontes ocultas de plantas/assemblies/enxertos. MCP/schema e dispatcher existentes
oferecem preview, lote, edição, inspeção, undo/save/load/replay, sem UI nova.

GpuMaterial1232B/tabela17slots20944B (+1360B), Vertex64/Globals176 mantidos.
Alternativas/custos em [layered-patterns.md](layered-patterns.md). A casa é o novo
modelo de teste; materiais não corrigem sua arquitetura. Árvore preservada.

## Resposta metálica direta — EAF-114

Reflectance.metallic default0/omitido mistura BRDFs dielétrica e metálica no
caminho opt-in, com Fresnel colorido pelo base linear e GGX/Smith existente.
Metal puro não tem difuso direto; ambiente constante legado é aproximação
mantida explicitamente, não IBL. Sem texturas, transparência, HDR ou grafo novo.
Reutiliza high.w: GpuMaterial1232/tabela20944/Vertex64/Globals176 mantidos;
default high.w explicitamente0 para materiais sem pattern/reflectance.
Same MCP/dispatcher/preview/lote/undo/replay, Scene74/replay78 e gates das fontes
ocultas. Contrato, alternativas e custos em [metallic-response.md](metallic-response.md).
Gaps observados e próximos casos em [authoring-gaps.md](authoring-gaps.md).

## Ambiente distante procedural — EAF-116

Light.environment opcional fornece radiância sky/horizon/ground em eixo explícito.
Cubemap64GGX/mips, irradiância8 e DFG64² lazy/preparados por conteúdo substituem
integração por pixel; mesma infraestrutura para todos os materiais, sem objeto
específico de casa. Intensidade fora da chave; uma chave/texturas reutilizadas,
LUT uma vez. None/intensidade0 preservam legado; ambiente é aditivo e independente
da constante ambient. direct_intensity default1 separa a luz direcional para
autoria/controles. Sombras direcionais não representam oclusão de um ambiente.

Recursos fixos148984B lógicos (3texturas/sampler), Globals176→192B; Vertex64,
Material1232/tabela20944 e caps intactos. Counters de preparo por Renderer estão
no adapter.environment_preparation das capturas, sem participar do replay/estado
autorado. Scene76/replay80 coordenados após reserva75/79 da Wedge funcional;
este incremento não integra Wedge/RenderBox. MCP usa set_light/lote/preview/
inspect/undo/load/replay existentes. Contrato/alternativas/limites em
[procedural-environment.md](procedural-environment.md).

## Observabilidade de normais de mesh — EAF-119

Consulta explícita por instância/tick/faces usa `mesh_normals` compartilhado com
`Document::append_vertices`: mesma pose e operaçõesf32, relatório angularf64.
Não persiste normais nem acrescenta trabalho por tick/frame, shader ou layout.
Alternativas de normais autorais/weld/crease ficaram adiadas até diagnosticar
as junções. Scene76/replay80 mantidos. A casa revela salto de normais em caps
separados e uso de cadeia Sweep já existente para um contato tangente; não há
solver arquitetônico. [Contrato](mesh-normal-inspection.md).

## Resolução explícita por curva — EAF-120

Sweep.spans usa enum JSON não etiquetado escalar|lista, evitando precedência
ambígua entre dois campos. Geometry comum persiste,76/80 sem mudança. Planner
preserva lockstep escalar e refina individualmente a lista; estações seguem
parâmetro da cadeia, não densidade amostral. Formas uniformes têm Geometry/canais
idênticos. Rust consumers adaptam Spans; não prometemos ABI estável. Refinamento
por seção/erro de superfície e comprimento de arco são contratos separados.
[Alternativas/custos/evidência](sweep-curve-sampling.md).
## Transações locais e modos de execução — EAF-123

Single player offline e multiplayer são requisitos permanentes; o núcleo Rust
compartilhado não depende de internet/autenticação/servidor separado para jogar
localmente. [Contrato e alternativas de transações e índices](local-transactions.md):
write sets por campo tocado e índice secundário de buffs, sem ECS/dependência nova.
A migração não altera ordem/atomicidade ou torna rede obrigatória.
