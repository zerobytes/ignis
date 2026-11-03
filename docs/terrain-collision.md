# Superfícies físicas de terreno — EAF-182

Recorte implementado e validado automaticamente, coordenado com EAF-180/181. Este chat possui
superfícies, física, consultas terrestres e publicação; nanites possui renderer,
LOD e `terrain-graphics-coordination.md`. Scene104/replay114 adotados;
terrain1/landform1 preservados. Build release corrente usado para esta unidade; janela liberada ao concluir os gates.

## Decisões e revisão antes do código

Uma lista explícita de instâncias estáticas torna suas malhas completas físicas.
Não muda o significado de todos os meshes visuais e não depende da câmera/LOD.
TriMesh conserva superfícies côncavas e vãos; convex hull ou caixa não servem.
Referenciar a geometria corrente evita manter duas cópias autorais divergentes.
Primeiro contrato é estático: rigs, playback e attachments físicos recusados.
Edição publica cena candidata validada e recompõe a simulação pelo caminho
transacional existente, com undo; edição física em runtime não é prometida.

- Preparar BVHs/SharedShape na construção/edição, nunca no tick ou draw.
- Compartilhar geometria preparada entre instâncias de mesmo asset/escala;
  transformação rígida permanece no colisor. Nenhuma serialização no kernel.
- Guardar índices/tipos nativos; memória do BVH físico é custo adicional real.
- Consultas usam broadphase/KCC existentes. Apoio de nascimento precisa cobrir
  o corpo, não apenas um raio central; em dúvida/complexidade, rejeitar.
- Persistência portátil recompõe colisores a partir da fonte canônica; snapshot
  nativo deve validar seus derivados. Não salvar BVH como formato portátil.
- Não adicionar dependências, worker, rede obrigatória nem renderer alternativo.
- Provas novas limitadas aos riscos de integração física, atomia e persistência;
  aparência/foco/teclado humano continuam gates distintos.

Aceite humano da demo continua pendente. Isto não declara streaming, costuras de LOD,
navmesh universal, geologia procedural completa nem aceite humano.

## Contrato implementado

`Scene.collision_meshes: Vec<String>` referencia instâncias estáticas de
`Scene.meshes`. O renderer continua consumindo Asset/Instance normais. Não usa
bounds como sólidos nem pacotes atmosféricos como física. Instâncias do mesmo
asset/escala compartilham um SharedShape/BVH por preparação; posição e rotação
ficam no colisor. TriMesh usa FIX_INTERNAL_EDGES (inclui deduplicação exata de
vértices), sem convexificação, quantização, fechamento de vãos ou remoção de
faces. Paredes, teto e abertura de arcos/túneis conservam os triângulos publicados.
É superfície de triângulos, não CSG ou preenchimento volumétrico universal.

- MCP `edit_scene` → `set_mesh_collision {instance,enabled}`; mesmo SceneEdit no
  checkbox **Colisão estática da malha completa** das propriedades do mesh.
- `publish_terrain` e `publish_landform` aceitam `collision: true|false|null`.
  Ausência/null conserva vínculo existente; true ativa, false remove. Asset,
  instância e vínculo entram numa única cena candidata/undo. O campo não altera
  os formatos de fonte terrain1/landform1.
- `inspect_mesh_collisions` informa vínculos e fronteiras; `inspect` conserva a
  cena completa e o estado do controlador. Não depende do renderer estar aberto.
- Fonte, transform, geometria e vínculos entram em edição/undo/redo/reset/load
  pelo mesmo caminho: simulação nova após validar o candidato. Edição autoral
  reinicia o runtime conforme o contrato existente; não é deformação física ao vivo.
- Não remover uma instância deixando vínculo pendurado. Desativar colisão antes
  de remover, ou usar `set_mesh_collision(false)` + remoção na mesma transação.
- Rig, playback, attachment e filhos consumidos por conexão de plantas são
  recusados neste opt-in estático. Superfícies precisam caber no frame físico
  residente ±10000m; validação recusa perda de área no escalonamento.

A malha publicada é a superfície física: `stride` autoral no terreno participa
explicitamente dessa escolha e do erro amostrado já exposto. Use stride1 para
resolução completa do patch. O LOD que o renderer escolhe depois não substitui
essa fonte física. A fonte f64 procedural permanece separada do mesh f32 publicado.

## Caminhada, consultas e nascimento

Simulation insere corpos fixos e colisores reais; KCC e solver dinâmico existentes
os encontram no broadphase. GroundNavigation usa as mesmas formas completas e
transformações, inclusive em consultas no tick0. Nenhuma autoridade de movimento
nova: aceleração/gravidade/limites de inclinação, offset e snap continuam no motor.
Guia de locomoção/render não serve como prova de contato.

Spawn projeta a base inteira do corpo sobre triângulos próximos por BVH, escolhe
altura suficiente para não enterrar o corpo numa encosta e verifica cobertura por
subtração de polígonos em XZ/f64. Isso detecta ponte estreita e buraco interno que
raio central/cantos isolados perderiam. Limite de inclinação vem do controlador;
clearance usa a forma real da superfície e as entidades físicas existentes. O
resultado `mesh_surface` referencia `collision_meshes`, enquanto `surface` antigo
continua referenciando entidades; `surface_id` resolve ambos. Nenhuma identidade
é consumida numa projeção recusada. Suporte de caixas legado permanece sem este
caminho adicional quando não há meshes físicos.

Envelopes de consulta: até4096 candidatos por superfície e4096 triângulos por
consulta; cobertura até512 fragmentos/65536 visitas a vértices. Saturação rejeita
com `support_limit`, não aceita resultado truncado. Apoio usa margem de contato
horizontal1mm, tolerância vertical6cm e área residual1e-12m²; cálculo conservador
pode recusar casos degenerados. Não é certificado matemático de predicados exatos.

Recorte de navegação: controlador terrestre e seu planejador KCC. Planejadores
legados de voo/salto/planeio, geração automática de rede, visão/audição e passos de gameplay,
projéteis e demais consultas semânticas ainda usam seus contratos de primitivas;
esta entrega não os converte silenciosamente em consultas de terreno. NPCs ainda
possuem a âncora `npc.surface` do esquema existente; a presença dela não substitui
colisão real sob o corpo. Expandir essas integrações exige gates próprios.

Superfícies não são atores nem consomem suas identidades. O estado grounded/KCC
registra apoio físico; o anel de contatos entre entidades não inventa uma
Reference de ator para um mesh. Eventos de superfície exigirão contrato próprio.

## Persistência e versões

Scene104/replay114 e checkpoint nativo3; conteúdo público5/protocolo5, PSV1,
terrain1/landform1 preservados. O registro nativo `surface_bodies` contém handles
Rapier; pertence apenas ao snapshot nativo. PortableSave conserva os vínculos e
as definições canônicas e recompõe corpos/colisores a partir dos assets, sem
serializar esse mapa ou BVH de backend. RuntimeWorld valida que fonte/vínculos
correspondem à cena. Snapshot nativo valida cobertura, transform e triângulos antes
de publicar. Alterar a cena do save sem seus derivados é recusado sem mutar o
mundo corrente. Saves com versão antiga de cena/checkpoint continuam exigindo
migração explícita compatível; não baixar versão à mão. Trajetórias entre backends
não passam a ter garantia bit-exata por esta integração.

## Revisão estática depois do código

1. **Alocações/cópias:** caminho vazio retorna antes de criar índices. Mapas frios
   por ID evitam procura linear de asset por instância; área por asset/escala só
   validada uma vez. BVH/vertices/índices físicos preparados na construção, sem
   cópia por tick; SharedShape clonado por Arc. Consultas de spawn alocam scratch
   limitado de candidatos/polígonos; não são zero-allocation.
2. **Tipos/acesso:** f32/u32 do mesh físico, handles Rapier e IDs compilados;
   autoria global/f64 continua no sampler. Cobertura local usa f64, sem números
   absolutos globais convertidos para float.
3. **Trabalho repetido:** nenhum hash/JSON/CBOR no motor de caminhada.
   Clearance de spawn usa o broadphase Rapier já atualizado das superfícies fixas,
   evitando um intersection_test contra cada malha e construção repetida do corpo. Preparação
   separada do GroundNavigation pode manter outro BVH para o mundo fixo; custo
   frio/residente consciente, não promessa de RAM compartilhada entre todos os
   adapters. Escalas diferentes exigem formas/BVHs próprios neste backend.
4. **GPU/I/O:** nenhum passe, stream, readback ou shader novo. I/O só em autoria,
   saves e demonstração. Renderer anterior permanece integralmente responsável
   por frustum/oclusão/atmosfera e representação distante.
5. **Locks/cache:** nenhum worker, lock, transporte ou cache global novo. Edição
   recompõe os caches a partir do candidato validado; fonte CPU e física residentes.
6. **Limites/rollback:** queries recusam complexidade, publicações são atômicas,
   checkpoint e portable mantêm fronteiras distintas; testes críticos cobrem risco
   de chão ausente, vãos fechados, nascimento inválido e save divergente.
7. **Custos mantidos:** triângulos/BVH físicos, broadphase e solver, reconstrução
   em edição/load, fontes duplicadas nos snapshots nativos conforme serde/backend.
   Não foi medido FPS nem sustentação de floresta física/multiplayer massivo.

## Validação e demonstração

Build corrente `b13af5530ec3621b`, aarch64/macOS, Rust1.97.1/release.
79 testes Rust em9 alvos selecionados: terrain_collision(2),
character_controller(19), ground_navigation(17), ground_local_connections(15),
spawn_support(19), npc_checkpoint(1), portable_save(1), terrain(3), landforms(2).
Clippy release/all-targets com warnings negados, fmt, Python e diff aprovados.
A primeira regressão de portable_save sinalizou o novo mapa nativo como campo
não classificado; ele foi explicitamente excluído dos primários portáteis e
adicionado à proibição de dados de backend no arquivo portátil. A prova completa
portátil foi repetida e passou; não se removeu proteção de estado de gameplay.

9 roteirosMCP/31 grupos: terrain_collision(3) mais8 existentes/28 grupos em
workspace isolado. Renderer nativo offscreen, sem janela/áudio: subida/descida e
junção, vão do arco, saves nativo/portátil e continuação; PNG de execução e replay
bit-exatos. No Rust também: inclinação/queda, bloqueio da perna do arco, corpo
dinâmico, rotação/escala, ponte estreita e buraco interno, nascimento real,
erro tardio atômico, fonte adulterada recusada, edição de geometria e undo.

O agregador run_verifications já possuía5 scripts gráficos sem registro na lista
SCENARIOS; não o declaramos como suíte completa. Esta unidade registra seu novo
roteiro e executou os8 selecionados diretamente em workspace isolado, sem alterar
arquivos de exemplos alheios. Registro em
`artifacts/terrain-collision-regression/report.json` e logs por cenário. Outras
provas: terrain-collision-verification.json, terrain-collision-mcp.log,
terrain-collision-tests.log (inclui a falha de classificação histórica),
terrain-collision-remaining-tests.log, terrain-collision-portable-final.log,
terrain-collision-tests-final.log (79 aprovados no runtime final),
terrain-collision-transform-final.log (fixture assimétrica de rotação/escala),
terrain-collision-clippy.log e terrain-collision-build.log.

Demo técnica: `examples/terrain-walk.scene.json`, receita e fontes binárias
`terrain-walk.recipe.json/.ignis-terrain/.ignis-landform`. Dois patches do mesmo
sampler seeded e um arco nativo, sem piso oculto. A imagem não pretende ser a
paisagem realista final. Abrir manualmente:

```sh
./target/release/ignis --scene examples/terrain-walk.scene.json --window
```

P inicia/pausa; clique na viewport e WASD/setas para mover; R reinicia. No editor,
botão direito orbita, meio/Shift+direito desloca e roda aproxima. O percurso inicial
passa à frente do arco; suas pernas bloqueiam, o vão permite passagem. Sair da
borda faz cair. Uma só versão corrente em target/release/ignis; nenhum app aberto,
fechado ou focado, nem instalador/cópia de executável adicional. Captura atual:
`artifacts/terrain-walk.png`. Aceite artístico, teclado/câmera humanos, terreno
mundial contínuo e demais integrações semânticas seguem gates distintos.
