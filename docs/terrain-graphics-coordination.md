# Coordenação de mundo procedural, colisão e renderização — EAF-180

Direção de Guilherme em31/10/2026: o mundo procedural deve usar o sistema de
renderização otimizado da main e ter colisão normal de terreno. Trabalho estável
no checkout principal; nenhuma engine gráfica separada para o procedural.

Base comum verificada: main a17bbd7, build44dae857d2128e92, Scene103/replay113,
protocolo5, terreno1 e landform1. Conferir versões no fonte antes de reservar
qualquer mudança; os números aqui descrevem a base, não concedem uma reserva.
Inventário e provas em [terrain-render-integration.md](terrain-render-integration.md).

## Responsabilidades acordadas

Divisão aceita pelo chat systems na resposta interchat de31/10/2026. Contrato
EAF-180, implementação físicaEAF-182 e experimento gráficoEAF-181.

| Frente | Responsável | Entrega |
|---|---|---|
| Fonte procedural e ferramentas topográficas | systems | Amostragem global, cristas/encostas/dobras, variedade de relevo e edição; fonte compartilhada entre representação e física |
| Colisão e caminhada | systems | Colisores reais do terreno/landforms no Rapier e controlador existente, apoio/spawn, subidas/descidas/queda e revisão de fonte coerentes |
| Residência e cobertura do mundo | systems, com contrato do renderer acordado | Vida dos recursos físicos/visuais, prontidão real, costuras e publicação atômica de cobertura; conservar fallback disponível |
| Representações gráficas e seleção | nanites | LOD/impostors compilados, cobertura/silhueta/material/luz e erro em pixels físicos; seleção por custo com fallback |
| Renderer nativo | nanites | Frustum, instancing, índices/streams, shaders, oclusão, atmosfera e medição do custo total |
| Integração e aceitação | ambas | Mesma cena gerada, fonte/revisão, recursos e controles; prova física e visual em conjunto |

O primeiro recorte de systems (EAF-182) é um trecho de terreno caminhável. O primeiro
recorte gráfico seguinte é o experimento de representações distantes automáticas
já autorizado, usando a floresta e os assets nativos de terreno/formas como matriz.
O trabalho gráfico está registrado em EAF-181; sua implementação ainda não começou.
Nenhuma frente precisa aguardar a conclusão de todo o mundo contínuo para
entregar seu recorte, mas as garantias devem permanecer explícitas.

## Contrato comum

1. Publicar geometria pelo caminho nativo de Asset/Instance e pacotes derivados.
   Os dados gerados herdam automaticamente os caminhos elegíveis de frustum,
   instancing, indexação, atributos compactos, shader por canais, oclusão e
   atmosfera. Não criar condições por árvore, nome de modelo ou origem procedural.
2. Colisão deriva da superfície física autorada/compilada e sua revisão, incluindo
   transformações e vãos aplicáveis. Nunca usar AABB de render como sólido,
   colisor invisível aproximado sem contrato ou LOD atmosférico como chão.
3. Visibilidade da câmera não altera colisão, simulação, navegação ou vida do
   mundo. Distâncias de residência visual e física podem ser diferentes. A fonte
   física continua pronta ao atravessar bordas, mudar câmera ou reduzir névoa.
4. Substituição de representação exige recurso e costura prontos de fato;
   consulta de planner com ready declarado não prova uploadGPU. Conservar a
   representação anterior até a troca atômica. Budget não autoriza buraco visível.
5. Resumos se referem à fonte/revisão exatas. Edição/regeneração invalida derivados
   gráficos e físicos pertinentes; não misturar malha nova com certificado/colisor
   antigo. Transações, undo, save/replay e referências permanecem coerentes.
6. A mesma aparência atmosférica participa da imagem e do orçamento de detalhe.
   Ar limpo não tem corte finito implícito. Silhueta que ainda contribui permanece
   representada; ocultação na câmera não elimina sombra/reflexo automaticamente.
7. Preservar a cadeia aprovada como referência visual: scripts/mountain_recipe.py
   e hashes em verify_terrain_render_integration.py. Ela é uma receita finita de
   build, ainda não sampler contínuo/tileável. A evolução nativa precisa resolver
   suporte/transições globais e variedade; repetir quadrados não basta.
8. Configuração/inspeção pelo dispatcherMCP e controles humanos correspondentes.
   O modo offline continua obrigatório. Nenhum JSON/serialização de autoria no
   kernel de amostragem, desenho ou caminhada por frame.

## Arquivos e builds

systems possui terrain/landforms/world_space/world_regions e física/controlador,
mais adapters de residência/cobertura que introduzir. nanites possui render*,
distant*,scene.wgsl e scripts/exemplos gráficos. A lista final de arquivos será
reservada por incremento no Kordo, não interpretada como permissão para sobrescrever
alterações da outra frente.

engine.rs/service.rs/model.rs/lib.rs, formatos Scene/replay/protocolo, Cargo,
AGENTS.md/performance.md/validation.md e o executável corrente são compartilhados.
Coordenar hunks, versões e janela de build antes de alterar. Commits atômicos com
stage seletivo; nenhuma troca/reset de branch ou limpeza de mudanças alheias.
Um único app corrente, sem cópias históricas. Não encerrar/recarregar o app humano
para medir: usar processos próprios e combinar qualquer troca da cena entregue.

Este documento é mantido pelo chat nanites. systems registra suas reservas e
resultados no documento físico/Kordo e informa o identificador da entrega.

## Gates de integração

- Caminhada real pelo controlador no terreno: repouso, encosta, subida/descida,
  borda/costura, queda e apoio. Verificar correspondência com a superfície visual
  e vãos/overhangs quando incluídos no recorte físico; não inferir física de raycast.
- Publicação/regeneração/edit/undo/load sem misturar fontes, revisões ou derivados;
  persistência/replay e identidade conforme o risco material da alteração.
- Renderer otimizado versus referência com mesma câmera/luz/resolução; silhueta,
  cobertura, estabilidade temporal e erro medidos quando houver aproximação.
- Cena procedural caminhável com o renderer atual; medir CPU, GPU/tempo total e
  memória distinguindo build/upload/runtime. FPS somente com medição real.
- Ao trocar residência/representação, manter cobertura e chão físico válidos.
  Não declarar mundo contínuo pronto por um planner, PNG ou trecho local.

## Confirmação

systems confirmou o contrato e assumiu EAF-182 (`iss_868e0104d7e942ec8b`).
Reserva: surfaces.rs novo, physics.rs/physics/*, ground_navigation.rs,
spawn_support.rs/walk_support.rs conforme integração, terrain.rs/landforms.rs
para publicação opt-in. Compartilhados somente seus hunks em model.rs,
engine.rs,service.rs,lib.rs e codec de conteúdo/rede se necessário, além de
AGENTS e docs/terrain-collision.md. nanites não altera esses campos/versões
durante182. Reserva planejada: Scene104/replay114; conteúdo/protocolo só avançam
se o codec exigir. Terreno1/landform1 preservados. São reservas de implementação,
não formatos já entregues; conferir contrato físico antes de integrar.

Durante182, a janela do buildrelease corrente ficou reservada para a física,
sem encerrar o app humano; liberação final registrada abaixo.
Renderer/distant/shaders eLOD181 ficam com nanites; mudanças futuras nos arquivos
compartilhados ou nova janela de build exigem aviso e reconciliação de hunks.
O documento presente e seu link em terrain-render-integration.md são mantidos
por nanites, com stage seletivo independente das fontes182 em edição.

Primeiro recorte físico confirmado: trecho local caminhável com superfície
completa, vãos e transforms, persistência e consultas físicas coerentes. O pedido
de personagem/câmera para teste humano faz parte do objetivo de integração;
prova técnica de colisão não equivale ao aceite humano dessa navegação.
Geração mundial e residência são etapas seguintes de systems, com contrato de
prontidão visual acordado com nanites. O aceite desta divisão não declara
colisão, streaming ou mundo contínuo implementados.

Atualização182 após início da implementação: acrescentados à reserva física
surfaces/support.rs e editor_mesh_ui.rs (controle humano pelo mesmo EditScene),
além de checkpoint nativo3 para o mapa de corpos estáticos. Scene104/replay114
mantidos. Após inspecionar o codec, systems confirmou que conteúdo5/protocolo5
permanecem: a lista física é administrativa e não integra o pacote visual público.
PSV1/terrain1/landform1 preservados. São decisões/reservas da implementação em
andamento, ainda sujeitas aos gates de182. Janela própria de cargo check release;
a janela humana permanece independente.

## Entrega técnica182 e liberação

systems confirmou commit local86b6561 na main e build correnteb13af5530ec3621b.
Conferidos o commit no checkout e o contrato docs/terrain-collision.md.
Scene104/replay114/checkpoint3 implementados; conteúdo5/protocolo5 e
PSV1/terrain1/landform1 preservados. Renderer intacto. Janela de build e hunks
compartilhados liberados para o próximo incremento, com nova reserva quando usada.

Gates informados por systems no mesmo build:79Rust/9alvos,9MCP/31grupos,
Clippyrelease all-targets-Dwarnings,fmt/Python/build/diff. Repetidos após trocar
clearance de spawn para broadphase; fixture assimétrica verifica rotação/escala.
Fonte completa usa TriMesh/SharedShape e conserva vãos/transforms.

Demo local: `target/release/ignis --root . --scene examples/terrain-walk.scene.json --window`.
P inicia/pausa, clique na viewport e WASD/setas movem, R reinicia. Dois patches
procedurais seeded com subida/descida/junção/borda e arco com vão físico, sem
piso oculto. Imagem artifacts/terrain-walk.png, relatório consolidado
artifacts/terrain-collision-validation.json. Não é ainda a floresta30mil
convertida para gameplay nem o mundo contínuo inteiro. Aceite humano da
caminhada pendente; EAF-182 permanece InProgress até esse gate.

Fronteira confirmada: KCC/solver e planejador terrestre usam malhas reais.
Visão/áudio/projéteis semânticos e planejadores legados aéreos ainda não foram
migrados. Geração/cobertura/residência contínuas seguem próximas etapas.
O experimento gráfico181 pode usar essa superfície sem mudar sua fonte física.

## Correção do aceite visual e paisagem integrada183/184

Ao abrir terrain-walk, Guilherme explicitou que esperava montanhas, vales,
árvores e cursos d'água. A demo do arco não recebeu aceite como paisagem
representativa. Os gates técnicos182 continuam válidos, mas não equivalem ao
resultado visual/gameplay solicitado. EAF-182 permanece com aceite humano aberto.

Nova divisão confirmada: systems183 possui Sampler/cristas nativas editáveis,
publicação regional, receita examples/representative-world.recipe.json e base
artifacts/representative-world/terrain.scene.json + placement.json. nanites184
possui scripts/build_representative_forest.py, implantação e cena derivada
world.scene.json. Não sobrescrever a fonte/base de183; preservar sua física,
personagem, câmera e luz. ClientMCP aceita log_path opcional desde0735903 para
os verificadores paralelos terem logs próprios, com default legado preservado.

Primeiro recorte explorável2×2km, grade4m em um asset regional513×513, evitando
emendas de normais nesse estágio. Costa submersa e oceano/curso d'água visuais;
leito, margens e terreno físicos da mesma geometria. Ainda não é mundo contínuo
ou fluxo simulado. Terrain2 é reservado por183 com leitura/re-encode1 preservados;
Scene104/replay114/checkpoint3 e conteúdo/protocolo5 ficam intactos neste recorte.

Previews são avaliados antes de povoar/abrir: a primeira composição183 foi
considerada íngreme e regular demais, solicitando encostas mais largas, alturas
proporcionais e vale/margens naturais na direção179. Implantação184 usa a
superfície triangular publicada, seed/bosques, exclusões de água/spawn/inclinação
e variação do asset existente. Árvores ainda são visuais, sem novos colisores.
A cena rica anterior foi reaberta apenas como referência, não como entrega183.

Entrega derivada184 capturada/inspecionada e aberta na GUI: world.scene.json,
12mil árvores/24mil componentes,4assets/24002instâncias. A base física e o
personagem permanecem iguais; systems autorizou câmera inicial derivada
[100,220,800]→[0,130,-150],FOV60 e calibragem da base density0,00006.
Erro máximo de raiz no triângulo após save/load<0,00005m, fonte preservada,
resumos399628B compilados no build5c2e797b819983f2. Não há medição deFPS ou
aceite humano inferidos. Água plana/vegetação visual e região finita continuam
limites explícitos. detalhes em representative-forest.md;181 não iniciado.
