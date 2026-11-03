# Frente gráfica — ponto de entrada

## Integração topográfica atual — EAF-179

Main reúne topografia164/formas166 com atmosfera, representação distante,
instancing,frustum,oclusão e canais consumidos165..178.15pares nativos com
terreno/landforms/cadeias preservaram pixels; receitas de build da paisagem
aprovada foram preservadas por hash e extraídas para reutilização. Não há merge
pendente dessas frentes. Colisão de terreno, residência real, costuras e novoLOD
coletivo continuam separados. [Contrato e evidências](terrain-render-integration.md).

## Consolidação atual — EAF-128

Trabalho estável em main no checkout principal desde29/10/2026. A união Scene86/replay90 preserva autoria/render/editor, controles independentes, transações locais e navegação terrestre; os gates históricos permanecem. Modly e Nanites ficam nas refs/worktrees experimentais, sem probes de pesquisa nos alvos da main. [Escopo e evidências](main-consolidation.md). Os registros abaixo são históricos por incremento.

## Atualização funcional EAF-127

Main790b04684/88 passa a cena85/replay89; reserva registrada em EAF-104 após confirmar editor02b1c2d83/87. Sem RenderFrame/Light/Vertex/Globals novos. Controller+walk-only pode usar local_connections.walk; novo State.ground_planning opcional com probes/steps/limited (replay compara). Integração deve preservar controles por ator, requested_velocity/grounded, cast lock que pausa edge **ou connector**, prazo e comparações controls/controllers/meshes. Queries só fixos, truncamento explícito/sem ótimo global, retry60 permanece para pistas móveis após falha. Nenhuma worktree/janela paralela alterada.


## Atualização funcional EAF-126

Main sobre d049fdf reserva cena84/replay88 após confirmar união editor83/87/ccce577 (comentário EAF-124 com_c35e27d0f4ac42748e). Registrado em EAF-104/EAF-124. Controller+mobility walk-only e links walk XYZ com gate84; QueryGroundTraversal e relatórios de admissão estática na inspeção. Nenhum campo RenderBox/Vertex/Globals/Light novo. Integração deve preservar controles por ator e requested_velocity/grounded de cada motor, cast lock que preserva trecho terrestre, mundo/cache por Simulation e comparação dos estados controls/controllers/meshes já existentes. Nenhuma capacidade da união isolada é importada por usar84. Janelas e worktrees não alteradas.


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

## Atualização funcional EAF-123

multiplayer-runtime desde e473e5a migra Plan para write sets locais e vida por
atores tocados; Contributions encapsula vetor de buffs com índice derivado por
dono/definição e acesso slice somente leitura. JSON continua lista, índice é
reconstruído ao desserializar. Cena77/replay81, Command/RenderFrame/renderer e
semântica preservados; replay exige o build corrente como antes. Não conflitar
numeração com editor/meshes/ambiente; preservar todos os campos/gates na integração.
Editor já integra e473e5a em sua Scene81/replay85 (checkpoint EAF-104 em29/10);
este incremento interno não exige aumentar versões nem perder campos/gates.
Evidência da origem:75693edd70fe6348,769Rust/63alvos,68MCP/280grupos headless,
Clippy/fmt/build aprovados. Single player offline permanece obrigatório e testado isolando a rede do processo;
nenhuma dependência de servidor separado é acrescentada à simulação local.

## Atualização funcional EAF-122

Branch codex/multiplayer-runtime desde main6bfbc5c: cena77/replay81, coordenados
no Kordo EAF-104 após ambiente76/80. Múltiplos Player, vínculos exclusivos e eventos
de controle pelo MCP; físicos/perfis/passos individuais, legado singleplayer.
RenderFrame/Vertex/Globals/shader permanecem iguais. Unificar formatos preservando
campos/gates das frentes: numeração maior não incorpora ambiente/editor/meshes.
Vínculos são de execução e não persistem em Scene; frame/picking continuam físicos.
Não reutilizar tokens de controle após reset/load/edição. Contrato actor-controls.md;
UI de escolha/transferência de controle e teclado multiplayer não estão implícitos.

## Atualização funcional EAF-124

Main sobre6bfbc5c: Entity.character_controller opcional e inspect_character_controllers, cena82/replay86 após consulta às pontas:76/80ambiente,77/81controles,81/85união do editor. Colisão inicial77/81 corrigida antes de commit; coordenação registrada em EAF-122/EAF-104/EAF-124. Nenhuma alteração Light/RenderBox/Vertex/Globals/pipeline. Preservar Entity.geometry+character_controller e validadores ao integrar a frente gráfica; versão maior não incorpora seu ambiente isolado. Janelas e worktrees preservadas; sem materiais/assets novos.


## Atualização funcional EAF-115

Main sobre e16ae24 reserva cena75/replay79 após74/78 gráfico. Coordenação EAF-113/com_f5469c4bd5f74833a4 e confirmação EAF-116/com_1c9c7cb17a33436982: ambiente/Light.direct_intensity ocupa76/80 em worktree isolada. Entity.geometry defaultBox/Wedge fixo/visual; RenderBox ganha discriminante Copy e seleção acompanha arestas. Vertex/Globals/pipelines inalterados nesta frente; a frente gráfica possui suas próprias alterações. QueryGeometry, oclusão/colisor/clearance/spawn compartilham volume e apoio plano recusa rampa. Integração deve reunir campos/validadores/RenderBox/Light de ambas as frentes e unificar versões; nenhuma numeração incorpora automaticamente capacidades isoladas.

## Atualização funcional EAF-111

Main sobre6b1ab5e reserva cena66/replay70, comunicado ao editor EAF-104 no Kordo. support_projection opcional em geradores, consulta query_spawn_support e relatórios numéricos nos recibos. Geometria continua caixa fixa, renderer/RenderFrame/Vertex/Globals inalterados. Integração deve preservar campos/validadores das frentes isoladas e unificar versões; numeração funcional não incorpora recursos gráficos isolados.

## Atualização funcional EAF-108

Main sobreb18c837 reserva cena65/replay69, comunicados ao editor EAF-104 no Kordo. templates ponderados em gerador/onda, fontes compiladas e binding pendente; campos numéricos de template em membros/recibos e tabela de IDs na inspeção. Sem RenderFrame/renderer/material/Vertex/Globals ou comandos visuais novos. Integração futura deve preservar schemas/validadores das frentes isoladas e escolher versão conjunta.

## Atualização funcional EAF-105

Main sobref6b707e: cena64/replay68 reservados no Kordo EAF-105 e comunicados a EAF-94. Region opcional em npc_generators e position nos recibos; mesmo spawn e limites, sem RenderFrame/renderer/Vertex/Globals/material novos. Demo usa caixas sem corpo para marcações e obstáculo físico autorado. Integração deve preservar campos/validadores de ambas as frentes e unificar formatos; numeração maior funcional não incorpora capacidades gráficas isoladas.

## Atualização funcional EAF-96

Main sobre751b4a4: cena63/replay67 reservados no Kordo EAF-96 e comunicados a EAF-94. activation em geradores, estado de permissão e suspensão efetiva; reuso de trigger, sem RenderFrame/Globals/Vertex/renderer novos. Marcações de demo são entidades sem corpo pelas ferramentas existentes. Preservar campos/validadores de ambas as frentes ao integrar, sem inferir suporte gráfico pela numeração funcional.

## Atualização funcional EAF-95

Main sobre685505b reserva cena62/replay66: initially_active e ControlNpcGenerator, enum de operação de população e estado de controle. RenderFrame/Globals/Vertex/renderer preservados. EAF-94 informada no Kordo: sua proposta60/64 coincide com o incremento funcional92 já existente; integração exige preservar os campos/validadores de cada frente e unificar versões, sem assumir interoperabilidade pela numeração. Nenhuma worktree gráfica alterada.

## Atualização funcional EAF-93

Main desde8d4f972 acrescenta ondas opcionais em npc_generators, estado/eventos derivados e inspeção no dispatcher existente, cena61/replay65 (reservados no Kordo EAF-93/EAF-91). Sem RenderFrame/Vertex/renderer novos. Integração futura preserva campos/validadores gráficos59/63 e geradores60/64; a numeração maior não incorpora capacidades gráficas ainda isoladas. Contrato em population-waves.md.

## Atualização funcional EAF-92

A numeração60/64 foi coordenada após a reserva59/63 em EAF-91. Esta branch funcional isolada não incorpora capacidades/campos gráficos; a integração precisa preservar ambos os contratos e seus validadores. Main acrescenta Scene.npc_generators (default vazio), SetNpcGenerators/InspectGenerators, cena60/replay64 e estado de geradores derivado no replay. RenderFrame/pipeline não foram alterados. Preservar campo na integração; geradores usam os mesmos spawn/remove e identidades EAF-70, com captura/picking de presença. Contrato em population-generators.md. A pesquisa gráfica segue isolada; números históricos abaixo refletem seus respectivos incrementos.

Direção de Guilherme em25/10/2026: iniciar em outro chat uma frente de capacidades gráficas, paralela à continuidade funcional de NPCs/habilidades. Este documento transmite contexto e fronteiras; não escolhe uma nova arquitetura de renderização. Base local verificada: main, implementação073200e, cena/replay46. Revalidar HEAD e status antes de trabalhar. Atualização EAF-60: identidade acrescentada ao catálogo e stamps de habilidades, cena46 mantida e replay48 após migração EAF-61 de buffs/percepção/históricos; contrato em instance-identity.md. EAF-63: RuntimeWorld encapsula definições/placements de execução; RenderFrame/picking filtram presença. RenderFrame mantém o formato de caixas. Cena46/replay49, inspect_world/inspect_origins; consultar runtime-world.md. Não usar índice de definição ou slot como identidade gráfica permanente.

Continuidade da pesquisa e hipóteses adiadas: docs/graphics-roadmap.md permanece na branch isolada codex/graphics-research. Consultar a exclusão documentada em [consolidação](main-consolidation.md).

Direção de Guilherme em25/10/2026: iniciar em outro chat uma frente de capacidades gráficas, paralela à continuidade funcional de NPCs/habilidades. Este documento transmite contexto e fronteiras; não escolhe uma nova arquitetura de renderização. Base local verificada: main, implementação073200e, cena/replay46. Revalidar HEAD e status antes de trabalhar. Atualização EAF-60: identidade acrescentada ao catálogo e stamps de habilidades, cena46 mantida e replay47; contrato em instance-identity.md. Scene/RenderFrame e mundo presente ainda não foram separados.

## Contexto e leitura inicial

Ignis é uma engine3D em Rust para agentes criarem, executarem, observarem e corrigirem jogos, com humanos no controle. Qualquer controle deve existir primeiro ou simultaneamente no MCP. Guilherme é principal engineer; discutir alternativas e tradeoffs antes de contratos/dependências difíceis de substituir. A ambição inclui corpos de espécies diversas e proceduralidade transversal, não apenas humanos ou assets de um jogo específico.

Ler [AGENTS.md](../AGENTS.md), [arquitetura](architecture.md), [performance](performance.md), [validação](validation.md), [comandos](commands.md) e [roadmap NPC](npc-roadmap.md). ADRs e registros de validação são históricos por incremento: não interpretar uma limitação antiga como inventário atual. O código atual é a referência para confirmar capacidades.

Kordo: workspace `engine-ai-first-d8634e3eb5`, projeto `prj_8b6f130352a14ff5af`. Consultar o projeto para marcos M02/M03/M05/M07; pesquisar tarefas existentes antes de criar uma tarefa concreta. EAF-58 recebeu aceite humano e foi encerrada; outras pendências de aceite não são encerradas por inferência. A frente gráfica não substitui a prioridade funcional deste chat.

## Base gráfica verificada

- [render.rs](../src/render.rs): wgpu27, RenderFrame compacto, caixas, caches de geometria/guias, buffers reutilizados, profundidade e captura GPU sem janela.
- [scene.wgsl](../src/scene.wgsl): normais e cor por vértice, luz direcional difusa com ambiente escalar, grade procedural e guias sem influência da iluminação. Ainda não é um pipeline PBR com texturas, sombras, malhas importadas ou animação esquelética.
- [model.rs](../src/model.rs): cena versionada, câmera/luz e entidades; [engine.rs](../src/engine.rs): construção/update do RenderFrame, comandos, edição e replay; [service.rs](../src/service.rs): execução e captura; [mcp.rs](../src/mcp.rs): schema/transport; [window.rs](../src/window.rs): apresentação e entrada nativas winit0.30.
- Geometria visual, colisão, oclusão sensorial e identidade do ator têm contratos distintos. Uma malha bonita não deve mudar implicitamente as regras físicas nem o conhecimento do NPC. RenderFrame não deve carregar configurações inteiras de gameplay.
- macOS/Metal é a plataforma validada. wgpu não comprova sozinho Windows/Linux/browser; WebAssembly continua candidato, não entrega testada. Não trocar backend nem ampliar requisitos de GPU sem apresentar tradeoffs.

Blender é a direção desejada para autoria/intercâmbio; suporte foi adiado pelo usuário e não está entregue. Avaliar glTF/GLB quando tratar importação, com escopo explícito de malhas, materiais, rigs e animações. Não prometer compatibilidade completa com tudo do Blender nem com assets proprietários de outra engine.

## Primeiro trabalho recomendado

Inventariar renderer e propor um primeiro recorte visual demonstrável, antes de implementar contratos estruturais. Comparar representação de malhas/instâncias, materiais/texturas, iluminação/sombras e espaço de cor/HDR. Explicitar requisitos de GPU, limites, custo de memória e impacto em autoria/MCP/replay. A sequência é proposta para discutir com Guilherme; não é autorização para implementar todos esses sistemas de uma vez.

O primeiro marco gráfico deve incluir cena pequena autorada via MCP, inspeção dos parâmetros, captura offscreen e demonstração visual humana. Efeitos de apresentação nunca viram a única evidência de que uma habilidade ou percepção funcionou.

## Trabalho paralelo e integração

Preferir worktree e branch próprias para a frente gráfica, partindo do main local que contenha este documento e073200e. Um clone do GitHub pode estar atrás: este incremento não foi enviado. Confirmar o ponto de partida em vez de usar automaticamente origin/main.

Fronteiras que ambas as frentes podem alterar: Scene/versões/migrações, Command/schema MCP, Engine::update_render_frame, RenderFrame, captura/replay e runner de verificações. Registrar a mudança proposta no Kordo antes de desenvolver contratos conflitantes. Worktrees isolam arquivos, mas não integram commits nem resolvem conflitos sem revisão. Não editar a árvore do outro chat nem trocar sua branch.

Preferir documentos e scripts específicos de cada frente; atualizar arquivos centrais em diffs pequenos. A frente funcional seguinte trata identidade de instâncias e spawn/despawn, com referências geracionais já implementadas e retenção de autoria integrada em EAF-63 com reserva antecipada e hook interno de retirada; spawn/despawn público ainda pendente: não assumir que slots atuais serão IDs gráficos permanentes. IDs autorados, identidade de runtime e recursos GPU devem permanecer distintos.

Cada worktree ativa pode manter um build corrente; não compartilhar binário mutável ou diretório artifacts entre validações concorrentes. Não abrir/focar/fechar a demo do usuário. Testes normais são headless, inclusive GPU; janela somente por pedido de demonstração ou integração desktop. Commits atômicos; publicação/push dependem de autorização própria.

## Gates

Aplicar checklist estático de performance antes/depois de cada incremento. Não declarar FPS sem medição. Reusar caminhos estáveis e preservar outputs em uso.

Para mudanças de render/cache: `python3 scripts/run_verifications.py --only render replay_inspection selection` e demais cenários afetados; `cargo test --locked --release --all-targets`, Clippy release/all-targets com `-D warnings`, fmt e regressão completa conforme AGENTS.md. Não eliminar testes de igualdade de pixels para esconder uma regressão: distinguir determinismo dentro do mesmo build de mudança visual esperada entre builds.

Base precedente:573 testes Rust e53 cenários/218 grupos MCP aprovados no build9e734a8d82ffc860. Esses resultados não validam código futuro da frente gráfica. EAF-58 já recebeu aceite humano. Não refazer builds apenas para iniciar esta leitura/planejamento.


## Autoria nativa EAF-76

Frente separada desde main c1ada9c, worktree native-meshes/custom-engine. Guilherme escolheu autoria inteiramente nativa. Scene48 acrescenta assets/instâncias visuais autocontidos, Command/SceneEdit expõem mesh edits/inspeções pelo mesmo MCP e replay52 compara poses. RenderFrame acrescenta cache de vértices de malhas; caixas e guias existentes preservados. CPU skinning com orçamento explícito, não integração do experimento de resumos. Contrato/alternativas/checklist em [mesh-authoring.md](mesh-authoring.md). Geometria visual continua separada de atores, colisão e oclusão sensorial. Integração de branches deve revisar estes contratos e versões, sem trazer caches/binários de outra worktree.
