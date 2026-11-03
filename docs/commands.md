# Contrato de comandos

## Conectores terrestres — EAF-127

`upsert`/`edit_scene`/templates: `character_controller` + `npc.mobility.walk:true` + `npc.local_connections:{walk:true,fly:false,max_distance:8}` (alcance0.25..8metrosXYZ), cena85. Reutiliza controles existentes; não há API visual exclusiva. `inspect_mobility.states[id].ground_planning` retorna probes/steps/limited do último plano, incluindo replay; ausente antes de planejar/legado. `ground_local_budget` significa busca limitada sem rota encontrada, não prova impossibilidade. Até3entradas/4saídas/direto,8sondagens/2400passos; lançamento separado até600. `query_ground_traversal` continua disponível para diagnóstico explícito. [Contrato](ground-local-connections.md).


## Navegação terrestre por controlador — EAF-126

`query_ground_traversal {entity,start,end}` consulta geometria fixa sem mutar, start/end são pés XYZ explícitos finitos±10000 e distância≤32m, entity precisa estar presente e possuir character_controller. Resposta `{tick,static_only:true,report:{status,ticks,distance,final_feet}}`; ticks são passos virtuais (máximo600), não CPU nem avanço da partida. NPC usa npc.speed, jogador usa locomotion.walk.speed; velocidade vertical inicial zero. O endpoint também precisa caber e ter contato; falhas distinguem apoio, overlap, bloqueio, altura final e orçamento.

`inspect_mobility.ground_admissions` mostra relatórios compilados por ator/índice de link. `set_spatial_network`, `upsert`, `edit_scene` e templates existentes são a autoria: Entity.character_controller + npc.mobility.walk:true sem outros motores/conexões locais. Gate84 permite ligações walk em XYZ quando há consumidor opt-in; consumidores antigos continuam recusando trechos incompatíveis. Dados compilados usam nós mais skin offset; consulta explícita e lançamento usam pés fornecidos. [Contrato](ground-navigation.md).



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
## Controlador terrestre — EAF-124

- `upsert`/`edit_scene`/`set_npc_templates`: `Entity.character_controller` opcional, campos e limites em [character-controller.md](character-controller.md). `{}` habilita motor terrestre padrão; `step` opt-in, omitir controlador conserva motor dinâmico.
- `inspect_character_controllers`: somente leitura; `{tick,controllers,configs}` por ID presente, com identidade, ativo/grounded/sliding, velocidade vertical, intenção XZ e translações desejada/corrigida/colisões do tick anterior. Configuração efetiva por ID em configs, inclusive instâncias nascidas; autoria original permanece em inspect.scene.
- `set_input`/`step` e políticas de NPC usam mesmo caminho. Snapshot e replay incluem estado do motor; cena82/replay86, sem importar ambiente76/80 da frente isolada.


## EAF-115 — geometrias convexas

`upsert`/`edit_scene` aceitam entity.geometry `{kind:wedge,rise:positive_x|negative_x|positive_z|negative_z}` para fixos/visuais; ausência/box conserva legado. `query_geometry {entity,start,end,half_extent?,interior?}` consulta segmento/AABB varrido contra entidade presente sem mutar. `query_spawn_support` retorna sloped_surface no primeiro topo inclinado, sem fallback. Cena75/replay79, [contrato e limites](geometry.md).

## EAF-111 — consulta e projeção de apoio

`query_spawn_support {template,position:[x,y,z],max_drop}` consulta a geometria atual sem alterar partida, autoria ou seed. Retorna report com status/surface/feet/position e surface_id legível; geometry_only explicita que não reserva capacidade/alvo. `set_npc_generators`/SceneEdit aceitam `support_projection:{max_drop}` opcional: candidato passa a ser origem da busca dos pés. Recibos preservam position sorteada e support com posição resolvida; inspect_generators.support_surface_ids decodifica índices de superfície. Cena66/replay70; [contrato e limites](spawn-support.md).

## EAF-108 — templates ponderados

`set_npc_generators`/SceneEdit aceitam template único ou `templates:[{template,weight}]`, com override/herança por onda. `inspect_generators` mostra pending_template/template_draws, template_ids para índices de membros/recibos, PRNGs e fontes compiladas. Pesos positivos, candidatos únicos e dependências atômicas. Ver [contrato](weighted-population.md).

## EAF-105 — região procedural de nascimento

`set_npc_generators`/SceneEdit aceitam `region:{min:[x,y,z],max:[x,y,z]}` como alternativa exclusiva a points. `inspect_generators` retorna configuração, seed/contadores e recibos com position em toda tentativa, point null para região. Mesmas operações de controle/população/replay. Ver [contrato](spawn-regions.md).

## EAF-96 — condição espacial de produção

`set_npc_generators` aceita `activation:{volume,mode:once|while_occupied}`; upsert/edit_scene autoram o trigger e seus targets. `inspect_generators.controls` mostra production_enabled/condition_open/activation, status manual e prazos efetivos; inspect_triggers mostra a última ocupação física. B/O/K mantêm permissão manual, sem ignorar região. Ver [contrato](generator-activation.md).

## EAF-95 — controle de geradores

`control_npc_generator {generator,action:activate|pause|cancel}` enfileira no mesmo log de população. `inspect_population` mostra pendências/recibos; `inspect_generators.controls` mostra status, término e tempos restantes. `initially_active` autoral defaulttrue, runtime não altera save de cena. Em cena de um gerador, B/O/K usam esses comandos; P pausa o mundo. Ver [contrato](generator-control.md).

## Ondas nos geradores

`set_npc_generators` aceita `waves:[{count,template?,delay_ticks?,completion?}]`. `inspect_generators` mostra fase, quota/contagens da onda e motivo de espera; recibos e eventos conservam índice/identidade. Campo opcional, edição atômica também em SceneEdit; cena61/replay65. Ver [contrato e demo](population-waves.md).

## Geradores de população

`set_npc_generators {generators:[...]}` edita autoria atomicamente; disponível também em `edit_scene`. `inspect_generators` consulta definições, contagens e estado/recibos. Replay64/cena60. Campos, limites, ordem, cadáver/ressurreição e exemplo em [population-generators.md](population-generators.md).

## Varredura de perfil e planejamento

`plan_mesh_sweep{sweeps}` é consulta pura: resolução recomendada por peça,
contagens e limite de desvio da linha central. Dobra spans até satisfazer
centerline_tolerance, máximo128, e informa surface_validated:false. Não valida
frames/faces convertidasf32; `preview_mesh_edit` continua sendo a admissão completa.

`edit_mesh`/`preview_mesh_edit` aceitam o edit de
[profile-sweep.edit.json](../examples/profile-sweep.edit.json): perfil convexo2D,
cadeia BézierXYZ, orientação profile_x, spans e escalas em estações. Novo asset
com shading/material neutros; `append:true` acrescenta a asset sem rig preservando
seus dados e aparência. Planta vinculada exige detach. Também disponível nos
lotes de cena; [contrato/limites](mesh-sweep.md). Sem formato persistido novo.

## Conexões de plantas72

`edit_plant` e `preview_plant` aceitam `{kind:"put_connection",connection:...}`
e `{kind:"remove_connection",id:"..."}`; também em edit_scene/preview_scene_edits.
`inspect_plant_connections` retorna connections e reports (retenção, união e
convergência dos pesos). A receita exige parent_instance/child_instance,
parent_sections:[lowerID,upperID], child_section, parent_root/child_root,
appearance:"parent", shape e skin. Schema estrito em tools/list e exemplo em
examples/native-tree.connection.json. [Contrato e limites](plant-connections.md).
Remover antes de rebase/detach; parâmetros de ambas plantas e transformações
reconstroem o asset composto atomicamente. Scene72/replay76 locais.

Uma chamada MCP usa a ferramenta `engine_command`. Argumentos: `{ "expected_revision": 3, "command": { "op": "..." } }`. A revisão é opcional. O schema completo está em `tools/list`; campos desconhecidos são rejeitados.

| Operação | Campos | Resultado/efeito |
|---|---|---|
| `inspect_plant_sampling` | id | Base construtiva corrente, coordenadas materiais, identidade de superfície por vértice, faces e relatório de trajetória; consulta pura com revisão, sem escultura/assembly |
| `inspect_mesh_material_domain` | asset, faces?, options? | Consulta pura do domínio material na geometria local; referência de sessão, taxas/anisotropia/colapsos e amostras limitadas |
| `preview_mesh_material_domain` | source: Geometry, faces?, options? | Mesma análise de geometria explícita, sem publicar; opções/limites em [mesh-material-domain.md](mesh-material-domain.md) |
| `set_ability_catalog` | `catalog`: resources/abilities | Autoria validada, reset e undo; também em edit_scene |
| `activate_ability` | actor, slot (1..16), target? | Enfileira; resultado de gameplay nos recibos |
| `cancel_ability` | actor, execution_id | Cancelamento por instância no próximo passo |
| `inspect_abilities`, `inspect_resources` | — | Saldos, casts, prazos, resultados e fila pendente; [contrato](abilities.md) |
| `inspect_vitality` | — | Vida/máximo/escala/estado e eventos de morte; [contrato](vitality.md) |
| `inspect` | — | Documento, posições, contatos, navegação, conhecimento dos NPCs, revisão e passo |
| `select_entity` | `id`: string ou null | Seleciona entidade existente ou limpa; estado de sessão |
| `select_at` | `x`, `y`, `width`, `height` | Seleciona caixa visível mais próxima no pixel indicado; miss limpa |
| `inspect_selection` | — | Entidade/componentes autorados, posição e estados runtime do selecionado |
| `inspect_navigation` | — | Estado, motivo, caminho e objetivo conhecido de cada NPC |
| `inspect_patrols` | — | Rotas resolvidas, índice, espera, interrupção, visitas e falhas de patrulha |
| `inspect_footsteps` | — | Emissores, distância acumulada, apoio, velocidade, fila e histórico automático |
| `inspect_noises` | — | Log de ruídos, quantidade pendente e limites |
| `emit_noise` | `origin`: position/entity; `radius` | Enfileira impulso para o próximo tick; congela posição imediatamente |
| `set_debug_view` | `npc_vision`, `npc_hearing`, `noise_events`, `npc_routes`: bool opcionais | Atualiza guias visuais da sessão; não altera cena ou simulação |
| `inspect_relations` | — | Atores/facções, regras autoradas e atitudes direcionais resolvidas |
| `inspect_territory` | — | Política territorial, fase/gatilho, rejeições, retorno/espera, posição própria e navegação |
| `inspect_objectives` | — | Política, índice/ação selecionados, elegíveis, motivo, transições, destino e navegação por NPC |
| `inspect_hearing` | — | Política de atenção, memória ativa, intensidade aceita e motivos/contagens da arbitragem sonora |
| `inspect_knowledge` | — | Configuração de confirmação, evidência por ator, progresso, último visto e confirmação |
| `inspect_targeting` | — | Política, motivo, candidatos observados, distâncias² e memória com identidade |
| `set_faction_relations` | `relations`: lista from/to/attitude | Substitui todas as regras; validação/reset/undo; também disponível em edit_scene |
| `inspect_npcs` | — | Percepção, memória, decisão e orientação sensorial de cada NPC |
| `generate` | `recipe`: width, depth, seed, obstacles | Substitui a cena pela sala procedural |
| `upsert` | `entity` | Cria/substitui entidade completa pelo ID |
| `remove` | `id` | Remove entidade existente |
| `set_camera` | `camera`: position, target, fov_degrees | Edita câmera |
| `set_light` | `light`: direction, ambient | Edita luz |
| `undo`, `redo` | — | Navega pelo histórico de edição |
| `play`, `pause`, `reset` | — | Controla a partida; reset retorna ao documento |
| `set_input` | `input`: x, z, mode opcional | Eixos normalizados; walk/sneak/run seleciona velocidade e ruído |
| `step` | `ticks` | Avança 1–3.600 passos quando pausado |
| `events` | — | Contatos, passo e contagem de eventos descartados |
| `save_scenario` | path, scenario | Valida/salva roteiro limitado de teste |
| `inspect_scenario` | path | Lê/valida roteiro contra a cena atual |
| `run_scenario` | path, report_path?, replay_path? | Executa isolado e retorna evidência, com outputs opcionais |
| `save_scene`, `load_scene` | `path` | Salva/carrega documento JSON versionado |
| `save_replay` | `path` | Salva entradas, posições, navegação e conhecimento finais esperados |
| `run_replay` | `path` | Verifica replay isolado, sem alterar a sessão |
| `capture` | `path`, `width`, `height` | PNG real e metadados; requer GPU |
| `exit` | — | Solicita encerramento do processo/janela |

Exemplo de entidade:

```json
{
  "id": "block_01",
  "name": "Bloco de teste",
  "transform": {"position": [0, 1, 0], "size": [1, 2, 1]},
  "color": [0.1, 0.7, 0.4],
  "body": "fixed"
}
```

`body`: `fixed`, `dynamic`, `player`, `npc` ou `null` (apenas visual). Neste laboratório todas as formas são caixas alinhadas aos eixos, e corpos móveis têm rotação bloqueada. Unidades: metros, segundos, Y para cima. Cores são RGB linear em [0,1].

Edição, load, undo e redo reiniciam a simulação e zeram as entradas. Uma edição inválida não altera o estado. `save_scene` nunca salva posições transitórias da partida. Alterar câmera/luz também é edição nesta versão.

Resultados de ferramentas têm `isError`, `content` e, para operações executadas pelo serviço, `structuredContent`. Erros incluem código e mensagem: `invalid_argument`, `revision_conflict`, `not_found`, `unsupported_version`, `io_error`, `graphics_unavailable`, `capture_failed`. Falhas de protocolo usam o campo JSON-RPC `error`.

Todos os caminhos são relativos à raiz definida ao iniciar. Caminhos absolutos, `..` e escapes por symlinks são rejeitados. O processo tem as permissões locais de quem o executa; use uma pasta exclusiva para o projeto. Não há listener de rede nem escrita fora da raiz pelo contrato de ferramentas. A conferência de caminhos não é um sandbox contra outro processo local que troque symlinks simultaneamente.

Protocolo testado: `2025-11-25`. O handshake aceita também versões anteriores indicadas no servidor, mas a interoperabilidade validada é a versão testada. Requests têm limite de 2 MiB e documentos de 8 MiB. stdout é reservado ao protocolo; stderr recebe diagnósticos.

## Perseguição por MCP

Criar ou alterar o NPC usa `upsert`, com a entidade completa:

```json
{"command":{"op":"upsert","entity":{"id":"hunter","name":"Hunter","transform":{"position":[0,0.85,-5],"size":[0.7,1.6,0.7]},"color":[0.75,0.15,0.8],"body":"npc","npc":{"target":"player","surface":"floor","speed":3,"stop_distance":1.2,"enabled":true,"awareness":{"mode":"omniscient"}}}}}
```

O piso e um alvo explícito precisam existir no resultado da edição. `target` é opcional/null, ou referencia um corpo `player`/`npc` distinto (cena11); `surface` referencia uma caixa `fixed` cujo topo define o plano de navegação. Remover ou alterar uma referência de forma incompatível falha atomicamente. `body: "npc"` exige configuração `npc` e vice-versa.

Use `inspect_navigation` para consultar `npcs[ID]`: `status`, `reason`, `path` (waypoints XYZ), `planned_tick`, `decision_tick`, `target_position` e `effective_stop_distance`. `decision_tick` normalmente representa o estado observado antes do passo físico; o encerramento por distância ocorre depois da física e usa o tick resultante. `tick` externo é o passo atual da simulação. O caminho retornado é o plano restante, não o histórico percorrido.

- `idle / no_known_goal`: sem objetivo conhecido ou varredura local em andamento.
- `pending / awaiting_step`: ainda não houve passo de simulação.
- `following / clear_waypoint`: movimento em direção a um waypoint com folga.
- `recovering / leaving_contact_margin`: saída curta e física da margem de contato (6 cm + até 5 mm de tolerância). Sem teleporte; respeita velocidade/distância restante e não consulta alvo oculto.
- `arrived / within_stop_distance`: próximo ao alvo, com espaço para o corpo e linha de visão livre (sem exigir que o centro do jogador seja navegável).
- `disabled / disabled_by_configuration`: `enabled: false`.
- `unreachable / no_clear_path`: nenhum caminho no domínio discretizado atual; inclui alvo fora da área ou ponto inicial sem saída segura da margem. A grade pode conectar o último nó ao destino por um segmento livre de até 0,5 m, preservando a distância de chegada.
- `unreachable / npc_off_surface` ou `target_off_surface`: altura incompatível com o piso configurado.

Para desativar, faça `upsert` com `npc.enabled: false`; para retomar, use `true`. Para excluir, use `remove`. Essas são edições do documento: reiniciam/pausam a partida, entram no undo/redo e invalidam revisões antigas, como as demais edições. `reset` limpa o plano; `play` e `step` recalculam dentro da simulação, sem chamadas MCP por frame.

Cenas usam versão **7**. Cenas v1 sem NPCs e v2–v6 continuam legíveis. Na migração, NPCs v2 recebem `awareness: {"mode":"omniscient"}`. Em v3+, `awareness` omitido/null recebe a configuração perceptiva padrão, materializada no documento normalizado. Campos `awareness` e `blocks_vision` não são aceitos em arquivos v1/v2. A política perceptiva v3 exige os dois tempos numéricos e não admite limite de distância; v4 permite limites opcionais. `area_search` não nulo exige v5. Audição e transmissão não nulas exigem v6. Passos exigem v7. Leitores anteriores não suportam os componentes novos. Replay v7 registra ruídos e compara posições, navegação, conhecimento, distância e estado da investigação espacial, com restrição ao mesmo build; gere novos replays após recompilar.

## Percepção e memória

`npc.awareness` aceita `{"mode":"omniscient"}` ou este objeto completo (valores padrão):

```json
{
  "mode": "perceptive",
  "vision_range": 12,
  "fov_degrees": 120,
  "initial_facing_degrees": 0,
  "memory_ticks": 600,
  "search_ticks": 180,
  "search_turn_degrees_per_second": 90
}
```

Alcance [0.1,100] m; FOV horizontal [1,360] graus; orientação inicial [-180,180] graus; memória [1,36000] ticks ou null; busca [1,3600] ticks ou null; giro [0,360] graus/s. `max_investigation_distance` opcional: [0.05,10000] metros; omitido/null desativa esse limite. Dentro de uma política explícita, omitir um dos tempos equivale a null. Pelo menos um dos três limites deve estar ativo; zero não significa desativado. Tick fixo = 1/60 s, pausas não consomem tempo. Yaw 0 olha para -Z; +90 para +X. A orientação acompanha instantaneamente a velocidade de navegação; na busca, gira no sentido positivo à velocidade configurada. Sem alvo, ou após desistir, fica parado mantendo a orientação. Zero de giro permite esperar sem varrer.

### Limites de investigação

| Campo | Início e consumo |
|---|---|
| `memory_ticks` | Idade da última observação; inclui deslocamento e busca parada |
| `search_ticks` | Tempo da varredura local, iniciado após chegar à região lembrada |
| `max_investigation_distance` | Soma dos deslocamentos XZ efetivos após perder visão, durante investigação/busca |

Qualquer limite ativo atingido encerra o episódio. Para usar somente distância, defina os dois tempos como null. Para usar somente tempo, omita a distância. Com ambos, o primeiro vence. O tempo é da simulação, não timestamp de parede, preservando pausa, execução headless e replay.

A distância é medida após a física e inclui empurrões; não inclui queda vertical, deslocamento anterior à perda de visão nem distância em linha reta até o alvo. Replanejar não renova o orçamento. Nova observação válida, reset ou edição reiniciam o contador; a desistência preserva o valor para diagnóstico. O movimento solicitado é limitado ao saldo restante, com tolerância de encerramento de 1 mm para evitar que o atrito impeça consumir o resíduo. Deslocamento externo pode ultrapassar o limite no último passo; a física não é desfeita. Não existe teleporte em runtime neste recorte; ao introduzi-lo será necessário distinguir essa descontinuidade de locomoção.

Somente distância **não garante desistência em tempo finito**: NPC bloqueado ou varrendo parado não a consome. Somente `search_ticks` também não limita uma investigação que nunca chega ao destino. Use `memory_ticks` para um prazo global desde a última observação. Esses parâmetros controlam persistência; não acrescentam exploração de caminhos alternativos.

`entity.blocks_vision` aceita bool ou null/omissão: omitido herda `body != null`; false permite enxergar através de um colisor, true permite que uma caixa sem corpo bloqueie visão. Não altera renderização, transparência ou colisões. O teste vai do olho (85% da altura do NPC) ao centro do jogador; alcance usa distância 3D, FOV é horizontal, oclusão usa segmento 3D contra caixas. Ainda não há amostragem de várias partes do corpo, luz ou latência de reação. Audição opcional está descrita abaixo.

`inspect_npcs` retorna `tick` e mapa `npcs[ID]` com `phase`, `reason`, `decision_tick`, `observed_this_tick`, `memory` (position/tick/source), `confidence`, `facing_degrees`, `search_started_tick`, `investigation_distance` e `distance_accounted_tick`. O último campo marca o passo físico contabilizado. Fonte é `vision`, `sound` ou `omniscient`; pistas sonoras incluem `noise_id`. Confiança decai linearmente com a idade da memória quando `memory_ticks` está ativo; sem esse prazo permanece 1 enquanto houver memória. É um peso de recência com expiração opcional, não probabilidade calibrada. Nenhuma observação nova não revela o motivo geométrico ou a posição oculta. No modo perceptivo:

- `unaware / no_observation`: nunca adquiriu o alvo; memória e objetivo vazios.
- `pursuing / visual_observation`: posição conhecida atualizada pelo sensor.
- `investigating / last_known_position`: perdeu visão; objetivo continua na última posição observada, com chegada em raio de 0,4 m.
- `exploring / investigate_search_point`: segue um destino gerado pelo perfil espacial, distinto da observação memorizada.
- `searching / scan_last_known_position`: chegou à região lembrada e gira a visão; não há objetivo de locomoção.
- `searching / scan_search_point`: varre a visão ao chegar a um ponto de busca.
- `lost / memory_expired`, `search_budget_exhausted`, `travel_budget_exhausted` ou `search_points_exhausted`: esquece o alvo e para. O sensor continua ativo para redetecção.
- `disabled / disabled_by_configuration`: sensor e perseguição desligados, memória vazia.

Onisciência produz `pursuing / omniscient_policy`, mesmo com alvo oculto. Navegação ainda pode indicar `unreachable`: conhecer o alvo não garante alcançá-lo. A memória expira desde a última observação; o orçamento de busca começa na chegada, e o primeiro limite atingido encerra a investigação. A mudança para busca ocorre no tick seguinte ao de chegada da navegação. Sensores leem o snapshot anterior à física; orientação por velocidade é aplicada depois da decisão para o próximo sensor.

`inspect` inclui os dois mapas (`npcs` e `navigation`), além das posições reais para diagnóstico privilegiado. `navigation.target_position` representa o objetivo conhecido, não uma consulta ao alvo real. Edições reiniciam a simulação; save_scene preserva configuração, não memória transitória. Replay preserva entradas e verifica o estado final completo. Pistas sonoras explícitas usam o contrato abaixo.

## Investigação espacial opcional

Em `npc.awareness` perceptivo, `area_search` omitido/null preserva a busca parada anterior. Para visitar pontos próximos à última observação, adicione:

```json
{"area_search": {"radius": 3.5, "points": 4, "scan_ticks": 120, "seed": 0}}
```

Esse fragmento integra a política completa enviada por `upsert`. Raio [0.5,20] metros; pontos [1,8]; varredura por ponto [1,3600] ticks; semente inteira u32. Pontos igualmente espaçados num círculo XZ centrado na última observação, à mesma altura. Ângulo inicial `seed % 360` graus; incremento `360 / points`; zero aponta -Z e +90 aponta +X. Sementes congruentes módulo 360 geram o mesmo padrão. Não há aleatoriedade por tick nem leitura da posição escondida.

Após chegar à região memorizada, o NPC visita um ponto por vez e varre a visão durante `scan_ticks`. Se a região inicial for diagnosticada inalcançável, o perfil espacial começa mesmo assim ao redor da memória; pontos com navegação inalcançável são descartados. Os pontos não são projetados para dentro do piso nem para o ponto navegável mais próximo. Ao esgotar a lista, desiste. Uma nova observação visual tem prioridade, cancela a lista e reinicia o episódio.

`search_ticks` passa a limitar toda a busca espacial (trajetos e varreduras) desde a chegada à região lembrada **ou** o diagnóstico de região inicial inalcançável. `memory_ticks` continua contando desde a última observação e a distância inclui também `exploring`. Nenhum ponto renova esses limites. `scan_ticks` controla a permanência em cada ponto; não substitui os limites globais. A configuração ainda exige pelo menos um limite global ativo. Bloqueio físico sem diagnóstico de impossibilidade pode consumir tempo sem consumir distância; mantenha prazo global quando precisar garantir desistência.

`inspect_npcs` acrescenta `search_point` (destino hipotético atual), `search_point_index` (índice zero-based; no esgotamento equivale à quantidade tentada), `skipped_search_points` e `point_scan_started_tick`. `memory` continua sendo somente a observação verdadeira. Desistência limpa o ponto, mas retém os contadores para diagnóstico; observação/reset/edição/desativação zeram o episódio. Troca de destino de memória/busca força replanejamento; perseguição visual mantém a cadência existente.

É uma estratégia espacial simples: não classifica portas, não estima a direção de fuga e não garante reencontrar o alvo. Mapa conhecido, piso plano e limites do A* continuam aplicáveis. `examples/search.scene.json` demonstra um caso de fuga pela ponta da parede e redetecção visual durante a busca.


## Audição e ruídos pontuais

Acrescente `"hearing": {"range": 12}` à política perceptiva completa via `upsert`. Alcance finito [0.1,100] m; omitido/null mantém audição desligada. Não altera a política onisciente. Cada entidade aceita `sound_transmission` em [0,1]: zero bloqueia, um transmite integralmente, omitido/null equivale a um. Independente de corpo físico, aparência e `blocks_vision`.

```json
{"command":{"op":"emit_noise","origin":{"kind":"entity","id":"player"},"radius":12}}
```

```json
{"command":{"op":"emit_noise","origin":{"kind":"position","position":[-4,0.8,2]},"radius":12}}
```

`radius`: [0.1,100] m; posição finita dentro de ±10000 por eixo. Origem por entidade é resolvida no instante do comando usando posição física atual ou posição de autoria para entidade sem corpo. Só a coordenada congelada entra no estímulo: nenhum ID de emissor é entregue ao NPC. Uma posição arbitrária permite iscas/proceduralidade; não há inferência da identidade do jogador. A tecla **N**, sem repetição automática, emite pelo mesmo comando um ruído de raio 12 m na posição do jogador.

A resposta contém `queued` (id u32, tick, position, radius) e `consumed_on_next_step: true`. O tick é o snapshot pré-física no qual será percebido. A emissão não avança a simulação; pausado, espera `step`/`play`. No máximo 16 ruídos pendentes no mesmo tick e 4096 por sessão. Excesso, origem inexistente e valores inválidos são rejeitados sem mutação; não há descarte silencioso. No limite de 36000 ticks, reset é necessário antes de emitir. Reset, edição, load e histórico limpam ruídos; pause preserva a fila. Ruídos não fazem parte do documento salvo por `save_scene`.

Cada impulso é considerado apenas uma vez. Alcance efetivo = `min(radius, hearing.range)` multiplicado pela transmissão de cada caixa cruzada pelo segmento 3D entre centro do NPC e posição emitida. O próprio NPC é excluído; outras caixas com transmissão explícita também atenuam quando contêm a origem. FOV não afeta audição. Não há difração, espessura de material, tempo de voo, volume em decibéis ou reprodução de áudio. Movimento sem componente `footsteps` não produz passos automáticos.

Entre os impulsos audíveis no tick, vence a maior intensidade relativa `1 - distância / alcance_efetivo`; empate preserva o último comando recebido. Um evento exatamente no limite é audível, com intensidade relativa zero. Visão/ onisciência sempre têm prioridade e dispensam seleção acústica naquele tick. A escolha é determinística; não há RNG nem leitura da posição atual de quem emitiu.

Audição selecionada gera `investigating / sound_cue`, `heard_this_tick: true`, `observed_this_tick: false`, `memory.source: sound` e `memory.noise_id`. Depois, `last_heard_position` mantém a coordenada até uma nova pista ou expiração. O NPC usa os mesmos trajetos, chegada, busca espacial e budgets; uma nova pista selecionada reinicia o episódio. Som não ativa `pursuing`: só visão real ou política onisciente o fazem. `heard_this_tick` indica a pista escolhida, não todos os sons potencialmente audíveis quando há visão. Confiança continua sendo recência, não intensidade nem certeza de identidade.

`inspect_noises` retorna `noises`, `pending`, `capacity` e `per_tick_capacity`; `inspect` inclui `recorded_noises`/`pending_noises`. Replay v7 preserva ordem, instante e coordenada dos comandos, inclusive os ainda pendentes no último tick. Valida limites/ordem antes de simular e consome somente eventos anteriores ao tick final; pendentes são dados registrados, não efeitos já ocorridos. A comparação do estado final não é uma assinatura de integridade do arquivo. Navegação continua restrita ao piso plano; ruídos em altura incompatível podem produzir `unreachable`.


## Guias visuais de percepção

```json
{"command":{"op":"set_debug_view","npc_vision":true}}
```

A tecla **V** alterna a mesma opção pelo dispatcher compartilhado. Default desligado; preferência somente da sessão, preservada por reset/load/undo/redo, sem entrar no documento ou replay. Não avança tick, pausa, reinicia física ou altera a revisão do documento. Comando idempotente retorna `debug_view` e `view_revision` (incrementada apenas quando a opção muda); esses campos também aparecem em `inspect`. A janela pausada usa essa revisão visual para invalidar o desenho. Não cria entidades e não participa das colisões.

Seta sobre a caixa aponta na orientação sensorial atual `facing_degrees`: zero aponta -Z, +90 aponta +X. Arco e raios tracejados projetados na altura do piso representam o alcance/FOV **nominal**, sem descontar paredes, diferenças de altura ou outras condições de oclusão. São desenhados sobre a cena para inspeção, podendo aparecer através de objetos; não são uma máscara de área realmente visível. FOV 360 mostra círculo sem raios laterais. O sensor continua usando distância 3D olho→centro e segmento contra oclusores.

| Cor/forma | Estado derivado do NPC |
|---|---|
| Verde / seta | `observed_this_tick`: detectou por visão no último passo executado |
| Amarelo / seta | Tem memória (visual ou sonora), sem visão naquele passo |
| Azul / seta | Sem pista, inclusive antes do primeiro passo ou após desistência |
| Cinza / sem contorno | NPC desativado |
| Rosa / losango, sem contorno | Política onisciente; não representa visão direcional |

A orientação atual é aplicada após decisão/navegação, enquanto a detecção descreve o sensor pré-física do último passo. Em uma transição pode haver diferença de um tick entre direção/posição desenhadas e observação registrada. Pausar conserva essa última observação; não executa uma nova consulta visual. Use `decision_tick`, `memory.tick` e `observed_this_tick` em `inspect_npcs` para análise exata. A cor não deve ser interpretada como sensor consultado novamente no frame renderizado.

`capture` usa a mesma visualização da janela e retorna `debug_view`/`vision_guide_count` nos metadados. Ligue as guias antes de capturar e desligue para imagens sem diagnóstico. As guias não mudam o formato da simulação; replays persistidos continuam restritos ao build que os gerou.


## Guias de audição e ruídos

`set_debug_view` é um patch: campos omitidos ou null preservam o valor atual; um objeto sem campos é no-op. O controle antigo `npc_vision` continua válido. `npc_hearing` e `noise_events` são independentes e começam false. **H** alterna ambos com base no estado de `npc_hearing`, pelo mesmo comando. V continua alterando apenas visão.

```json
{"command":{"op":"set_debug_view","npc_hearing":true,"noise_events":true}}
```

Círculo roxo contínuo: alcance nominal do NPC habilitado com audição perceptiva, projetado na altura do piso. Não aparece para perfil surdo, desativado ou onisciente. Círculo laranja tracejado e cruz: raio/posição congelados do impulso, em um plano horizontal na altura da emissão. São referências nominais: a audição usa distância 3D e transmissão por obstáculos; a interseção visual entre círculos não prova recepção. `heard_this_tick`, memória e `noise_id` são a evidência de percepção.

Mostra até 16 eventos mais recentes com idade menor que 90 ticks (1,5 s de simulação), incluindo pendentes. Pausa conserva o desenho; emitir ruído pausado incrementa a revisão visual quando `noise_events` está ativo. Reset/load/edição limpam impulsos; preferências visuais persistem. Captura retorna `hearing_guide_count` e `noise_guide_count` além da contagem de visão. Os controles não alteram documento, entradas ou replay, nem produzem áudio no dispositivo.


## Passos configuráveis

Componente opcional `entity.footsteps` em corpo player/npc (cena 7), configurado por `upsert`:

```json
{"surface":"floor","stride":0.6,"radius":8.0,"min_speed":0.1}
```

`surface`: ID de corpo fixed que suporta os passos. `stride`: comprimento entre emissões [0.2,10]m; `radius`: alcance nominal [0.1,100]m; `min_speed`: velocidade física mínima [0.01,10]m/s. Null/omissão deixa o personagem silencioso. Configurações/referências inválidas são rejeitadas sem alterar estado.

Passo exige intenção de movimento, contato físico ativo e base do corpo a até 5 cm do topo do piso configurado. Acumula apenas distância XZ resolvida; deslocamento solicitado contra parede não conta. Parar preserva fração do passo; perder apoio zera. No máximo um impulso por emissor/tick, mesmo sob deslocamento excepcional. Emissão usa centro do corpo pós-física, consumida no sensor do próximo passo. NPC ignora os próprios passos, mas pode ouvir os de outros emissores; sua memória continua anônima.

`inspect_footsteps` retorna tick, footsteps (states por ID, pending, recent, total_emitted, dropped_recent) e recent_capacity=256. Cada estado inclui índices numéricos de entidade/superfície normalizados, distance_since_step, grounded, speed e emitted. `inspect` inclui states e generated_footsteps. `inspect_noises` mantém log/limites manuais e adiciona footsteps (anel recente), pending_footsteps e dropped_footsteps. Passos não consomem o orçamento de 4096 ruídos manuais. Até9 emissores, até 9 pendentes, histórico máximo 256 com descarte dos mais antigos. IDs automáticos ≥2³¹; emitter_index serve exclusivamente à proveniência e autoexclusão. Não significa que o ouvinte identificou o jogador.

Replay 7 armazena comandos manuais, regenera passos e compara seu estado final; `footsteps_matched` informa essa comparação. Eventos pendentes no último tick ficam no estado esperado, sem serem ouvidos antecipadamente. Reset/edição/load/undo/redo limpam filas/contadores/acumuladores; configurações permanecem no documento. Guias de ruído mesclam as duas fontes em ordem temporal, mantendo16 mais recentes. Ainda não existe reprodução sonora no dispositivo.

## Locomoção — cena/replay 8

`set_input` aceita `input: {x, z, mode}`; `mode` é `walk`, `sneak` ou `run`, opcional com default `walk`. A chamada substitui a entrada inteira: omitir mode após correr retorna à caminhada. Eixos continuam em [-1,1] com diagonal normalizada. `inspect.input` expõe o modo e `inspect.player_motion` expõe o perfil selecionado; `speed` é a velocidade solicitada, enquanto a velocidade efetiva está no estado dos passos quando o emissor existe. Colisão pode impedir deslocamento.

Em uma entidade `body: player`, `locomotion` é opcional:

```json
"locomotion": {
  "walk": {"speed": 4, "noise_multiplier": 1},
  "sneak": {"speed": 1.5, "noise_multiplier": 0.2},
  "run": {"speed": 6, "noise_multiplier": 1.8}
}
```

Cada velocidade deve ser finita e estar em [0,1;10] m/s; multiplicador finito em [0;4]. Zero silencia passos; positivo multiplica o raio base e é limitado a [0,1;100] m. Não altera `emit_noise` manual nem velocidade dos NPCs. Configuração ausente usa os valores acima; sem `footsteps` o personagem continua silencioso. Referência de piso/stride e validações do emissor permanecem iguais. Todos os três perfis são obrigatórios quando o componente está presente.

Exemplo: `{"command":{"op":"set_input","input":{"x":1,"z":0,"mode":"sneak"}}}`. Shift e Control são os equivalentes nativos de corrida/furtividade, com Control prioritário. Pausar preserva entrada e congela tempo; soltar teclas atualiza entrada, perder foco limpa eixos/modo. Reset/load/edição/undo/redo zeram entradas. Modo não é postura e não muda altura do colisor. Cenas anteriores sem componente são migradas; v1–v7 com `locomotion` explícito são rejeitadas para evitar interpretar um contrato futuro como antigo. Replay 8 requer o mesmo build.

## Patrulha — cena/replay 9

Componente opcional `npc.patrol`, editado pela entidade completa em `upsert`:

```json
"patrol": {
  "route": {"kind": "points", "points": [[-4,2], [-4,-2], [4,-2], [4,2]]},
  "traversal": "loop",
  "wait_ticks": 60,
  "travel_ticks": 1800,
  "retry_ticks": 180,
  "arrival_distance": 0.25
}
```

`route.kind: ring` aceita `center: [x,z]`, `radius`, `points` e `seed` em lugar da lista. Exemplo: `{"kind":"ring","center":[0,3],"radius":1,"points":4,"seed":27}`. Semente gira pontos equidistantes pelo ângulo seed % 360; mesmos parâmetros/build produzem mesma rota. `traversal` é `loop` ou `ping_pong`; um único ponto é permitido e causa espera/revisita do mesmo local.

Limites: 1–16 pontos; coordenadas/centro finitos até ±10.000 m; círculo com raio [0,5;32] m; wait_ticks [0;3.600], travel_ticks [1;36.000], retry_ticks [1;3.600]; arrival_distance [0,1;2] m. Y é derivado do piso configurado e metade da altura do NPC. Pontos bloqueados ou fora do piso são rejeitados pela navegação durante a simulação, não pela autoria. `travel_ticks` limita cada tentativa; após todos os índices distintos falharem sem sucesso, a rota espera retry_ticks antes de tentar de novo.

`inspect_patrols` retorna `tick`, `routes` com pontos XYZ resolvidos e `patrols` com estado por ID. `inspect.patrols` contém apenas o estado; `inspect_navigation` mostra o caminho efetivo. Fases: `ready`, `moving`, `waiting`, `suspended`, `disabled`, `cooldown`. Campos: índice/direção, timers, total de visitas/descarte, falhas consecutivas, bitset `failed_points` (bit i corresponde ao ponto i) e `last_failure` (`unreachable`/`travel_timeout`, preservado como histórico do último erro). Esses estados não afirmam que o NPC sabe onde está o jogador.

Visão/som reais interrompem; toda memória/investigação tem prioridade. Retomada mantém o índice interrompido e reinicia timers; espera anterior não é descontada. Onisciência suspende continuamente. NPC desativado não patrulha. Ausência/null mantém comportamento anterior. Reset/load/edição/undo/redo reiniciam estado; pausa congela timers. Replay compara estado via `patrols_matched` e no resultado global `matched`.

## Patrulha espacial (cena/replay24)

`npc.patrol.route` também aceita `{"kind":"points_3d","points":[[0,2.8,0],[3,4.8,0]]}` ou `{"kind":"ring_3d","center":[0,4,0],"normal":[0,1,1],"radius":2,"points":6,"seed":27}`. Exige `npc.mobility`; pontos representam o **centro do corpo**, enquanto os nós de `spatial_network` representam os pés. `points`/`ring` antigos continuam XZ, derivados do piso; sua combinação com mobilidade também exige versão24.

Mesmos limites de pontos, raio e timers. `ring_3d.normal` é normalizada e deve ter comprimento[0.01,100]; cada componente do centro mais raio deve caber em±10000m. A normal define o plano do círculo. A semente define o ângulo inicial em graus módulo360. Base determinística: projeção de−Z no plano, com fallback para+X perto do paralelismo; segundo eixo é o produto vetorial dessa base pela normal. Normal+Y reproduz a orientação do ringXZ. A receita é resolvida na edição/carga, não por tick. Pontos gerados não prometem conectividade.

Loop/ping-pong, espera, prazo de tentativa, descarte/cooldown e retomada usam a mesma rotina. Chegada só conta com feedback do destino/etapa correspondentes; conversão centro/pés aceita tolerância2mm por componente. Aproximação parcial não conta como visita. Capacidades e colisões continuam no motor: planador não ganha subida, caminhante não ganha salto. A perda de um objetivo preserva o trecho aéreo comprometido até o motor poder trocar de destino. Espera atrasa o próximo ponto; um voador sem hover continua orbitando, não fica artificialmente imóvel.

`inspect_patrols.routes`, seleção, snapshots e replay mantêm XYZ. Guias lilases preservam altura dos pontos espaciais. Reset/edição/load/histórico reiniciam a rotina. Território, recuperação e passagens planas ainda não combinam com mobilidade espacial. Verificação: `scripts/run_verifications.py --only spatial_patrol`.

## Guias de caminhos e patrulhas

`set_debug_view.npc_routes` é bool opcional, false por padrão; omitir preserva valor. Tecla **J** chama o mesmo comando. Preferência de sessão persiste em reset/load e não altera documento, tick, conhecimento ou replay. V/H preservam essa opção.

Linhas brancas contínuas mostram o caminho atual do navegador, a partir da posição do NPC; cruz branca indica seu destino conhecido. Cruz vermelha indica destino diagnosticado como inalcançável. Linhas lilás tracejadas e cruzes menores mostram a ordem/pontos da patrulha autorada, inclusive enquanto suspensa; não garantem passagem livre. Ping-pong não fecha a última aresta de volta ao começo. NPC desativado não desenha rotas.

Overlay projetado no plano do piso, sobre os objetos; não representa colisão ou visibilidade 3D. Usa caminho/destino existentes, nunca a posição oculta do jogador. Até 256 segmentos planejados por NPC; `capture.truncated_navigation_paths` informa quantos caminhos foram cortados, e `route_guide_count` informa quantidade de primitivas de linha (cada cruz tem duas). `inspect_navigation` continua fornecendo o caminho completo. Marcador de destino pode estar além do trecho exibido. Geometria/caches/GPU limitados e compartilhados com as outras guias.

## Cenários de verificação — documento v1

Comandos de serviço:

- `save_scenario {path, scenario}` valida roteiro e referências contra a cena de autoria atual e salva atomicamente.
- `inspect_scenario {path}` lê/valida e retorna `scenario` tipado.
- `run_scenario {path, report_path?, replay_path?}` executa uma cópia isolada da cena de autoria desde tick 0; retorna `report` e caminhos opcionais de outputs. Não altera a partida ao vivo, mesmo se ela já estiver em andamento. Roteiro não embute cena: carregar a cena correspondente antes de executar.

```json
{
  "version": 1,
  "name": "Passos furtivos",
  "budget_ticks": 60,
  "steps": [
    {"op":"step","ticks":30},
    {"op":"set_input","input":{"x":1,"z":0,"mode":"sneak"}},
    {"op":"step","ticks":20},
    {"op":"assert","condition":{"kind":"npc_memory","id":"hunter","source":null}},
    {"op":"assert","condition":{"kind":"footsteps","min":1,"max":3}}
  ]
}
```

Nome de 1–128 bytes, 1–256 ações, orçamento entre 1 e 3.600 ticks. Pelo menos um `assert` ou `until` obrigatório. Soma dos máximos de `step` e `until` não pode ultrapassar o orçamento; `ticks`/`max_ticks` positivos. Omitir modo em set_input mantém default walk. `emit_noise {origin,radius}` usa o contrato de ruído normal. Não há edição de cena dentro do roteiro.

`assert {condition}` testa imediatamente. `until {condition,max_ticks}` testa antes de avançar e após cada tick até satisfazer ou atingir o limite inclusivo; condição inicialmente verdadeira não consome ticks. Predicados:

| kind | Campos | Critério |
|---|---|---|
| `position_within` | id, center:[x,y,z], radius | Centro físico dentro da esfera inclusiva; corpo obrigatório |
| `npc_observed` | id, expected:bool | Visão/onisciência no último tick; som não é observação visual |
| `npc_memory` | id, source:vision/sound/omniscient/null | Fonte da memória; null exige ausência |
| `npc_phase` | id, phase | Fase da percepção |
| `patrol_phase` | id, phase | Fase da rotina configurada |
| `patrol_visits` | id, at_least | Contador de chegadas maior ou igual |
| `footsteps` | min, max | Total automático da sessão isolada no intervalo inclusivo |

Position center deve ser finito e dentro de ±10.000 m, radius em [0,001;10.000] m; min≤max. NPC/patrulha precisam existir com os componentes correspondentes. Fases disponíveis estão no schema de `tools/list`.

`report` contém version/name, passed, completed_steps, ticks, checks, failure e final_snapshot. Cada check tem step_index (zero-based), tick, condition, passed e actual. `failure` nula significa sucesso; `assertion_failed`/`condition_timeout` ou erro de execução indicam primeiro problema. `completed_steps` conta somente ações concluídas com sucesso. Não se executam ações posteriores à falha.

Roteiro válido que falha num check retorna **isError=false e report.passed=false**: a ferramenta executou e encontrou a falha pedida. Documento/parâmetro/path inválido retorna isError=true. Um limite operacional atingido durante execução, como o 17º ruído no mesmo tick, aparece em report.failure. Report/replay opcionais são exportados também para testes falhos, permitindo reproduzir o prefixo. Cada output é escrito atomicamente, mas os dois não formam uma transação conjunta. Destinos não podem coincidir com o roteiro ou entre si, inclusive via aliases. A chamada é síncrona e pode bloquear temporariamente o serviço/janela; não há deadline de tempo real ou cancelamento nesta versão.

## Inspecionar e capturar um replay

```json
{"command":{"op":"inspect_replay","path":"artifacts/scenario-patrol.replay.json","tick":31}}
{"command":{"op":"capture_replay","replay_path":"artifacts/scenario-patrol.replay.json","path":"artifacts/replay-selected.png","tick":31,"width":1280,"height":900}}
```

`tick` é opcional/null e seleciona o final por padrão. Inteiro no intervalo inclusivo `[0, quantidade de inputs]`; zero representa estado inicial com ruídos manuais emitidos nesse tick ainda pendentes. A reconstrução executa exatamente `tick` passos; ruídos de ticks anteriores foram processados, os do tick selecionado aguardam o próximo passo. Passos automáticos são regenerados e preservam sua fila da mesma forma. Toda a gravação é validada, inclusive o sufixo de uma consulta intermediária. Limites e compatibilidade são os mesmos de `run_replay`: versão10/build atual, até36.000 ticks/4.096 ruídos, arquivo até8MiB.

`inspect_replay` retorna `snapshot`, `noises` (histórico manual, passos recentes/pendentes e limites) e `replay`. O snapshot é de uma instância isolada: revision/view_revision zero, pausada, histórico de edição vazio, preferências `debug_view` da sessão. `input` é o último aplicado, ou padrão em zero. Uma alteração de entrada ainda sem passo não é registrada no formato atual; a inspeção não inventa esse estado de controle.

`replay` informa path, selected_tick, total_ticks, at_end e final_verification. No prefixo, final_verification=null: não há expectativa gravada para esse instante. No final, contém a mesma comparação de `run_replay`, sem repetir a simulação. `matched=false` não impede inspeção ou imagem do estado realmente reconstruído. Isso permite diagnosticar divergências sem tratá-las como falha de transporte.

`capture_replay` retorna PNG inline e arquivo, com os metadados usuais de `capture` e o campo replay. Câmera, luz e entidades vêm da cena gravada; preferências de guias vêm de `set_debug_view` da sessão. Viewport64..2048 por dimensão. Captura e replay de origem precisam ter destinos distintos, incluindo aliases de diretórios/symlinks. Escrever a imagem é atômico; erro preserva o arquivo anterior. O cache/GPU de captura é compartilhado com `capture` e invalidado por conteúdo, sem trocar a cena da janela.

Ambos preservam autoria, entrada, playback, histórico, revisão e replay da partida aberta; `expected_revision` continua protegendo a consulta contra revisão viva divergente. Comandos síncronos sob o lock atual podem fazer a janela aguardar. Cada consulta reconstrói desde zero, sem cache/checkpoints de simulação. Não representam savegame, restauração ou seek da partida.


## Volumes de gameplay e visibilidade (cena/replay10)

Uma entidade pode ter `trigger: {enabled: true, targets: ["player"]}` e `visible: false`. Trigger exige `body: null`: sua caixa de transform é uma região lógica fixa, sem colisão. Alvos são1..32 IDs únicos de corpos físicos existentes; no máximo32 volumes. Criação/configuração/remoção pelo upsert/remove, como qualquer componente; erros de referências/limites preservam toda a partida. Remover um alvo ainda referenciado é rejeitado. Transformações são autoradas; editar pausa/reinicia a simulação, sem evento sintético de saída.

`visible` é opcional/null e significa true quando omitido. false esconde a caixa do render/captura, sem remover o corpo físico nem alterar blocks_vision/sound_transmission. Por isso, um volume invisível pode ter um marcador visual separado, sem forçar a aparência da região lógica. Cenas1..9 sem campos novos migram para10; trigger/visible explícitos exigem versão10. Replay atual usa10 e o mesmo build.

```json
{"command":{"op":"upsert","entity":{"id":"exit_zone","name":"Exit region","body":null,"color":[0.2,0.8,0.4],"visible":false,"transform":{"position":[0,1,2],"size":[2,2,2]},"trigger":{"enabled":true,"targets":["player"]}}}}
{"command":{"op":"inspect_triggers"}}
```

`inspect_triggers` retorna tick e triggers; o mesmo bloco triggers aparece no snapshot. `volumes` mapeia ID para enabled, targets/occupants ordenados, contadores entered/exited e last_change_tick. `events` traz tick, kind enter/exit, volume e target, ordenados por tick/ID de volume/ID de alvo. event_capacity512, total_events e dropped_events tornam o descarte explícito. Cada alvo gera uma transição por mudança; não há evento stay por passo. Contadores somam transições de todos os alvos do volume. Enabled=false ignora amostras; edição dessa política reinicia todo estado como demais edições.

Amostragem ocorre após física e incremento do tick, comparando sobreposição estritamente positiva em X/Y/Z entre caixas alinhadas. Contato exato de faces não é ocupação, sujeito à precisão f32 das coordenadas. Tick0 começa vazio; um corpo já dentro gera enter no primeiro passo. Zonas podem se sobrepor; nenhuma pista é automaticamente entregue ao NPC. Não há sweep: atravessar integralmente uma região entre duas amostras pode não ser detectado. Não há rotação, sensor móvel ou callbacks de gameplay neste recorte.

Cenário versão2 (relatório acompanha a versão) acrescenta predicados `trigger_occupied {id,target,expected}` e `trigger_entries {id,at_least}`. target deve constar na seleção do volume. Cenários1 continuam válidos com os predicados anteriores; usar predicados novos em1 é rejeitado. until/assert consultam estado tipado, sem JSON por passo. Exemplo completo em examples/triggers.scenario.json; primeiro aguarda entrada, confirma ao menos uma transição e depois aguarda saída. Isso evita confundir ausência inicial com conclusão de uma passagem.

## Edição atômica de cena

`edit_scene {edits:[...]}` aceita1..512 operações tipadas `upsert {entity}`, `remove {id}`, `set_camera {camera}` e `set_light {light}`. São as mesmas formas dos comandos individuais, mas dentro de uma transação de autoria. Não aceita comandos de runtime, arquivos ou lotes aninhados. `expected_revision` é o guard da requisição inteira.

Operações são aplicadas em ordem a uma cena temporária; upsert substitui a entidade inteira, sem mesclar campos, e a última escrita vence. Remove exige que o ID exista naquele ponto da sequência; falha informa índice base0 no texto. Referências e demais invariantes são validadas sobre o resultado final: é permitido criar o consumidor antes do alvo, renomear uma entidade e todas suas referências juntas ou corrigir um valor temporariamente inválido. Não é permitido terminar com referência ausente, entidade duplicada ou limite final excedido.

Erro local, cena final inválida ou revisão obsoleta deixam autoria, simulação, entrada, pistas, replay, playback, histórico e revisões intactos. Sucesso substitui a cena uma vez, aumenta revisão uma vez, pausa/reinicia a simulação, limpa redo e adiciona uma única entrada de undo. Até um lote que termine igual à cena original segue esse contrato de edição. O limite final é512 entidades; staging pode temporariamente chegar a1.024 antes de remoções. Transporte MCP continua limitado a2MiB por mensagem.

```json
{"expected_revision":12,"command":{"op":"edit_scene","edits":[{"op":"remove","id":"exit_zone"},{"op":"remove","id":"exit_marker"}]}}
```

Comandos individuais reutilizam o mesmo caminho com uma operação. O lote não é uma transação de save/load nem executa ticks no meio. A cena combinada `examples/stealth-lab.scene.json` foi criada pelo MCP renomeando player→hero, atualizando o alvo do NPC e criando região/marker em um lote com referências futuras.


## Posse da entrada e foco da janela

SetInput válido e reset avançam uma revisão interna de entrada. O teclado registra a revisão do comando que aplicou; ao perder foco, só envia entrada neutra se ainda possuir aquela revisão. Um SetInput posterior pelo MCP prevalece, mesmo quando repete os mesmos valores. Mudanças reais nas teclas podem assumir controle novamente; repetição sem mudança e releases de teclas já removidas não reenviam entrada. Essa revisão não é persistida nem apresentada como estado de replay; serve somente à coordenação entre controle nativo e dispatcher.

## Seleção de sessão

`select_at`: dimensões inteiras1..16.384; x/y finitos no intervalo [0,width) e [0,height), origem no canto superior esquerdo, pixels físicos. Use a mesma câmera e dimensões da imagem inspecionada. Capturas continuam limitadas a64..2.048; o limite maior de picking atende viewports nativos. Resultado: `selection` ID/null, `hit` ponto 3D/distância em metros desde a câmera/null, `view_revision`. A distância mede a superfície atingida, não o centro do objeto. Segmento entre planos0,05m e30.000m em profundidade de câmera. Caixas alinhadas, posição física atual, visíveis; empate de interseção usa ID lexicográfico. Guias e contornos não são selecionáveis.

`select_entity` aceita também entidades invisíveis; ID inválido ou picking inválido preserva estado. Operações idempotentes não avançam `view_revision`. `inspect` inclui `selection` ID/null. `inspect_selection` retorna `{tick,selection}`; selection é null ou contém id, entity, world_position, npc, navigation, footsteps, patrol, trigger. Ausência de componente é null. Trigger inclui ocupantes/contadores/última mudança, sem copiar o log global. Nenhuma inspeção avança o relógio.

A seleção não entra em cena/replay/undo. Reset e edição preservam ID se continuar existindo, recalculando índice; remoção/renomeação limpa. Undo posterior pode restaurar entidade sem selecioná-la. `capture` inclui selection e selection_guide_count0/1. `inspect_replay.snapshot.selection` e `capture_replay.selection` herdam a seleção viva somente se o ID existir na cena gravada; a posição e componentes vêm do tick reconstruído. Estado vivo permanece intacto e verificação final não inclui a seleção. O destaque dourado é overlay sem oclusão e não comprova visibilidade sensorial.

## Autonomia de NPCs e alvo opcional (cena/replay11)

`npc.target` omitido/null: nenhuma observação visual de ator é procurada, mesmo em modo omniscient. O NPC pode ficar parado, patrulhar e investigar ruídos conforme sua configuração. Não há fallback para o jogador nem seleção automática. Assim uma cena com NPCs pode não conter player. Audição continua sem associar a pista a um ator.

Um ID explícito deve apontar para outro corpo player ou npc; self, corpo fixo/dinâmico genérico, objeto visual e referência ausente são inválidos. Dois NPCs podem apontar um para o outro: ambos observam o snapshot pré-física do passo, sem chamada recursiva de decisão. Visão/oclusão/memória permanecem iguais; mover um alvo oculto não atualiza sua última posição conhecida. Seleção automática de ameaças, facções e ordens de aliado são futuras políticas distintas; target configura um objetivo de perseguição existente, não uma relação social.

Cenas1..10 preservam o contrato anterior (NPC exige alvo player); normalize após validação salva como11. Replay11 exige o mesmo build, como antes. Configurar/remover pelo upsert/remove/edit_scene reinicia a sessão; remover um alvo ainda referenciado falha, mas um lote pode remover a referência e entidade atomicamente. Save/load/undo/replay/inspeção usam os contratos existentes. Inspeção autoral de target ausente omite o campo; não é um ID vazio.

Para uma pista sonora, NavGoal usa a altura do ouvinte ao verificar a altura compatível com o piso; não usa dimensões de um alvo cuja identidade a pista não confirma. Para observação/memória visual, dimensões do alvo explícito continuam metadados autorais. Limites de8NPC/128corpos, grade plana e um objetivo visual por NPC permanecem.

A navegação ainda exige destino compatível com a altura da superfície plana. Uma pista audível em outra altura pode ser recebida e permanecer inacessível; resolver andares/links/projeção de pistas faz parte da ampliação de navegação, não é ocultado como sucesso.


## Relações e seleção automática (cena/replay12)

`Entity.faction` é opcional, apenas em player/NPC: identificador ASCII alfanumérico, `_` ou `-`, de1..80 bytes. Até32 rótulos distintos, contando regras. `Scene.faction_relations` aceita até64 pares direcionais únicos `{from,to,attitude}`, com `allied`, `neutral` ou `hostile`. Mesma facção é aliada por padrão; distintas ou sem facção são neutras. Uma regra explícita prevalece, inclusive para a mesma facção. A→B não implica B→A. Regras podem referenciar facções ainda sem atores.

`npc.targeting: {"kind":"hostile_nearest","switch_margin":0.2}` habilita seleção automática. É mutuamente exclusivo com `npc.target`; ambos ausentes preservam rotina/audição sem objetivo visual. Alvo explícito continua uma ordem direta, independente de relações. `switch_margin` deve ser finita em[0,0.9]; é uma fração de distância, não segundos ou probabilidade.

A elegibilidade usa hostilidade, excluindo o próprio NPC. Somente observações aceitas pelo sensor participam da comparação: alcance, FOV e oclusão na política perceptiva; onisciência apenas quando configurada. Aquisição escolhe a menor distância3D entre centros; empate exato usa ID lexicográfico. Com incumbente ainda observado, troca somente se `nova_distância² < distância_incumbente² × (1 − margem)²`. Uma margem0.2 exige mais de20% de aproximação relativa; comparações usam f32. Se o incumbente deixa de ser observado, um hostil observado pode substituí-lo imediatamente. Se nenhum é observado, mantém a última pista até expirar, sem atualizar coordenadas ocultas.

`inspect_targeting` expõe `decision_tick`, política, facção, `selection` e `memory`. Selection informa `reason`, contagens elegíveis/observadas, `observed_actor` e distâncias² do selecionado, mais próximo e incumbente. Valores são da decisão pré-física; posições de `inspect.runtime_positions` são pós-física. `memory.actor_id` identifica observações visuais/oniscientes; som permanece anônimo, inclusive se emitido por entidade conhecida. Relação não filtra ruído anônimo. A inspeção geral, de NPC e de seleção usa os mesmos IDs; índices internos de replay não são IDs públicos.

Alterar facção/política usa upsert; relações usam set_faction_relations individual ou em edit_scene. Validação final atômica, um reset/undo por lote. Reset/load/undo/redo recompilam catálogo e apagam memória runtime. Cenas1..11 são migradas para12; novos campos em documentos antigos são rejeitados. Replay12 exige o mesmo build; cenários continuam versões1/2.

Limites atuais:8NPCs +1player,128 corpos com navegação, uma memória ativa, sem dano/ameaça/reputação/amizade individual. O mapa de navegação permanece global e plano: outros corpos podem alterar caminhos mesmo sem observação. O ator lembrado é excluído como obstáculo do planejamento sem consultar sua posição atual como destino; isso não torna o mapa conhecido apenas por percepção.


## Confirmação visual e suspeita individual (cena/replay13)

`npc.recognition: {"acquire_ticks":120,"decay_ticks":240}` habilita exposição gradual antes da reação visual. Campos inteiros obrigatórios em1..3600, passos fixos de1/60s. Ausência/null preserva reação imediata e nenhum registro de reconhecimento. A configuração está no upsert/edit_scene e no schema MCP, com validação atômica/reset/undo. Cenas1..12 migram para13 sem ativar a opção; usar recognition numa versão anterior é erro. Replay13 requer o mesmo build; cenários continuam1/2.

Cada candidato elegível tem um slot independente: alvo explícito ou hostil da seleção automática. Uma observação visual aceita acrescenta `decay_ticks` unidades; um passo sem observação retira `acquire_ticks`, até zero. O limiar é o produto dos dois. Partindo de zero, acquire_ticks observações contínuas confirmam o candidato; partindo do máximo, decay_ticks ausências apagam o registro. Exposição intermitente pode acumular se os ganhos excederem as perdas. Progresso é uma fração determinística, não uma probabilidade. Confirmação permanece até o registro chegar a zero; reencontro antes disso permite reação imediata. A política onisciente explícita confirma imediatamente.

A seleção só compara candidatos confirmados E observados no passo atual. Um candidato próximo ainda suspeito não substitui um alvo confirmado; margem e desempate continuam iguais. `selection.observed_candidates` conta observações brutas; `confirmed_candidates` conta as atualmente observadas que passaram pelo limiar. `awaiting_confirmation` explica a ausência de seleção apesar de haver candidatos vistos. `observed_actor` e `observed_this_tick` correspondem ao selecionado confirmado, não à mera exposição sensorial.

`inspect_knowledge.npcs[id]` informa config, decision_tick e tracks: actor_id, evidence, progress, confirmed, visible, last_seen_tick, last_seen_position e source. Apenas sensores atualizam a última coordenada; fora de visão ela fica congelada. A mesma lista aparece em inspect/inspect_npcs/inspect_selection e inspect_targeting. ID aqui é a referência técnica ao indivíduo para acumular exposição; não implementa identificação facial, disfarce ou descoberta de nome/facção.

O registro de reconhecimento não substitui a memória ativa de perseguição. Um alvo já confirmado pode continuar sendo procurado na última posição conhecida enquanto memory_ticks/budgets existentes permitirem, mesmo depois de sua confirmação decair. Sem alvo ativo, suspeita sozinha mantém a rotina; patrulha só é interrompida por confirmação ou pista acústica. Sons continuam anônimos, não somam confirmação de nenhum indivíduo e podem provocar investigação antes da confirmação visual. Visão confirmada tem precedência sobre som.

Guias existentes: amarelo vivo quando existe suspeita não confirmada e não há memória ativa; verde quando um alvo confirmado é observado; âmbar durante memória/investigação; azul sem pista; magenta onisciente, cinza desativado. Cor não controla decisão. Configuração/reset/desativação limpam todos os slots. Limite fixo9 por NPC (até8 outros atores); sem crescimento, fila de sons ou transferência entre indivíduos. Mapa de navegação continua global/plano.


## Prioridade de investigação sonora (cena/replay14)

`npc.sound_attention` é opcional, válido somente com awareness perceptivo e hearing habilitado. Exemplo: `{"commitment_ticks":90,"switch_margin":0.15,"repeat_radius":1.25,"visual_priority_ticks":120}`. Ticks inteiros0..3600, margem finita0..1, raio finito0..20m. Upsert/edit_scene pelo MCP com validação atômica/reset/undo. Ausência/null preserva a seleção legada do ruído mais forte de cada passo. Cenas1..13 migram sem ativar política; novo componente numa versão anterior é erro. Replay14 exige mesmo build.

A intensidade continua o escore de gameplay existente: `1 − distância3D / raio_efetivo`, após limitar pelo alcance auditivo e atenuar nas caixas. Não são decibéis, probabilidade ou volume de áudio reproduzido. O escore é congelado ao aceitar a pista; o ouvinte andar não recalcula o som antigo.

Prioridades:

1. Observação visual/onisciente confirmada atual prevalece; nesse passo o scan acústico é pulado. Suspeita visual sem confirmação não bloqueia som por si só.
2. Memória visual ainda válida bloqueia som enquanto sua idade for menor que visual_priority_ticks. Depois desse prazo, um som audível pode substituí-la.
3. Para memória sonora válida, ruídos a distância menor ou igual a repeat_radius do ponto lembrado são rejeitados. É agrupamento espacial, não reconhecimento do emissor.
4. Outros pontos são rejeitados enquanto a idade da pista ativa for menor que commitment_ticks.
5. Depois, troca exige `intensidade_nova > intensidade_aceita + switch_margin`. É margem **aditiva** no escore0..1, diferente da margem relativa de distância da seleção visual. Ex.:0.4 com margem0.15 exige mais de0.55. Igualdade rejeita.

Filtragem ocorre antes do ranking: um ruído forte descartado não esconde uma alternativa elegível. Entre elegíveis, maior escore vence; empate mantém a ordem anterior(automáticos antes dos manuais, último vence). Todos são comparados contra a mesma pista ativa anterior ao passo. Rejeição não modifica destino, tick, distância percorrida ou orçamento de busca e não cria fila futura. Se a memória expirou por tempo/busca, ou já foi esquecida por distância, deixa de bloquear a próxima pista. Compromisso não prolonga a vida da memória.

`inspect_hearing.npcs[id]` informa policy, decision_tick, heard_this_tick, memory e attention. `attention` contém reason; audible_candidates/selectable_candidates; repeated_rejections/commitment_rejections/margin_rejections/visual_memory_rejections; strongest_audible_id/strength; selected_noise_id e active_noise_id/strength. Contagens cobrem somente estímulos audíveis não autoemitidos; cada rejeição recebe a primeira regra aplicável na ordem acima. strongest_audible pode ser diferente do selecionado. reason rejected exige olhar as contagens; no_stimuli não afirma ausência de capacidade auditiva. selected_noise_id é do passo atual, active_* permanece enquanto sua memória sonora existir. heard_this_tick significa pista aceita; um som audível rejeitado aparece nas contagens.

Nenhum dado identifica o emissor no conhecimento do NPC. Reaquisição visual, reset/edição/load/undo/redo/desativação/esquecimento limpam o recibo acústico correspondente. Guias/ruídos/rotas existentes continuam disponíveis; a política não adiciona controles apenas visuais nem emissão de áudio por hardware.


## Objetivos componíveis — cena/replay15

`npc.objectives` é opcional:1..8 entradas `{priority, when, action}`. Configurar por upsert/edit_scene; validação é atômica e edição reinicia a simulação/histórico conforme os demais componentes. Omitir/null conserva o fluxo legado. Prioridade0..255, maior vence; empate fica com a primeira entrada autorada. Índices públicos são zero-based. Não há ordenação por ID, aleatoriedade ou seleção baseada em alvo oculto.

| when | Elegibilidade |
|---|---|
| always | Sempre, respeitando requisitos intrínsecos da ação |
| evidence | Existe observação aceita ou memória não expirada |
| visual_memory | Evidência ativa visual/onisciente; pode ser última posição conhecida |
| sound_memory | Evidência ativa sonora anônima |
| visible_target | Alvo visual/onisciente confirmado observado neste passo |

| action.kind | Parâmetros e comportamento |
|---|---|
| respond | Exige evidência; usa perseguição/último conhecido/busca existentes e npc.stop_distance. Seguir não confere conhecimento extra. |
| flee | distance:[0.5,32]m; exige evidência. Destino radial oposto à pista, congelado até nova evidência/ativação; já distante mantém posição. Coincidência usa +X. |
| patrol | Exige npc.patrol; retoma ponto/direção da rota com novo orçamento da etapa. Pode continuar mesmo conhecendo pista, se tiver maior prioridade. |
| guard | point:[x,z] finitos até±10000m; arrival_distance:[0.1,2]m. Retorna ao ponto e mantém posição. |
| wait | Sem deslocamento próprio; percepção continua. |

`respond` e `flee` nunca são elegíveis sem pista, inclusive com when:always. Sem entrada elegível, o seletor não produz destino; uma patrulha configurada só atua se tiver objetivo selecionado. Fuga/guarda não escolhem alternativas quando o destino é inalcançável: navigation.status/reason explicam falha. Chegada mantém o objetivo selecionado; são políticas contínuas, não uma fila de tarefas consumíveis.

`inspect_objectives` retorna tick e npcs por ID, com policy, decision_tick, selection, navigation e memory decodificada. selection contém active/action, eligible_mask (bit i = entrada i elegível), reason (legacy/activated/higher_priority/condition_lost/retained/no_eligible/disabled), previous, started_tick, transitions, activated_mask, resumed, changed_this_tick, destination e evidence_tick. resumed permanece durante a ativação que retornou a um índice já escolhido; changed_this_tick descreve só a última decisão. destination é o ponto das ações flee/guard; para respond/patrol consulte navigation.target_position. Antes do primeiro passo, o estado default tem reason:legacy e nenhum índice mesmo com configuração; policy distingue configuração pendente de modo legado.

Nova observação pode alterar memória enquanto outro objetivo controla movimento. Fase perceptiva suspended significa pista ativa sem execução de investigação; observed_this_tick/heard_this_tick continuam reportando recepção. Memória e busca expiram por tempo de simulação durante suspensão; viagem de outra ação não é cobrada como investigação. Busca mantém cursor, sem avançá-lo durante suspensão; seus relógios não pausam. Patrulha conserva ponto/direção, reinicia espera/viagem/cooldown. Troca de objetivo descarta caminho/feedback anterior. Reset, load, upsert, undo/redo e desativação limpam seletor. Replay exige mesmo build e inclui estado/decisões; formatos de cenário1/2 permanecem inalterados.


## Território e retorno — cena/replay16

`npc.territory` opcional `{home:[x,z], radius, arrival_distance, rearm_ticks}`. Home finito até±10000m; radius:[0.5,32]m; arrival_distance:[0.1,2]m e estritamente menor que radius−0.02; rearm_ticks:0..3600. Pontos de patrol (inclusive ring resolvido) e guard precisam pertencer ao círculo. Spawn externo é permitido e inicia retorno no primeiro passo; origem geometricamente bloqueada pode ser autorada, mas gera falha de navegação. Configuração por upsert/edit_scene com erro atômico, reset/histórico iguais aos demais componentes.

Raio compara centros no plano XZ, inclusive a fronteira; não inclui extensão do corpo, não usa altitude ou comprimento da rota. Sensores filtram pistas externas antes de selecionar. Motivo outside_territory nas inspeções de alvo/audição distingue rejeição territorial de ausência de observação/audibilidade. Alvo ativo visto saindo inicia retorno se não houver alternativa interna confirmada; desaparecimento atrás da parede não informa saída e preserva último conhecido/budgets anteriores. Som não informa identidade e sua rejeição externa isolada não causa retorno.

Retorno também começa se posição própria/destino conhecido estiver fora. Fase returning tem precedência sobre objetivos/patrulha; memória/suspeita/recibo são apagados e sensores ficam suspensos. Objetivos/percepção expõem territory_return; não se cobra distância de investigação. Ao alcançar a origem dentro de arrival_distance+0.02, entra em cooling_down; ready_at_tick marca o rearme. Deslocamento para além dessa chegada durante espera recomeça retorno. Com0ticks, rearma no próprio passo de chegada. Em active, volta a perceber e retoma rotina; evidências externas não podem reativar perseguição. Inalcançável mantém returning e navigation.status/reason, sem teleporte ou timeout automático.

`inspect_territory` retorna tick e npcs por ID: policy, decision_tick, territory, position e navigation. territory contém phase (unrestricted/active/returning/cooling_down/disabled), last_trigger (own_outside/target_left/goal_outside/displaced_from_home), return_started_tick, ready_at_tick, returns, changed_this_tick, rejected_visual/rejected_sound e active_target_departed. last_trigger/return_started_tick preservam o último episódio; rejeições/departed são do passo atual. returns conta episódios iniciados fora de retorno/espera; deslocamento durante espera reinicia tentativa sem aumentar o contador. Antes do primeiro passo, phase:unrestricted com policy não nula significa configuração ainda não executada. Reset/load/edição/undo/redo/desativação zeram estado; replay inclui-o.

`set_debug_view {npc_routes:true}`/J mostra círculo laranja e X da origem; vermelho indica retorno/espera. Visibilidade da guia não altera política. Território é comportamental: rotas de desvio e forças externas ainda podem atravessar a fronteira. O motor usa posição própria no começo do passo seguinte para detectar cruzamento. Retorno bloqueado não é resolvido por esta política.


## Recuperação de navegação — cena/replay17

`npc.navigation_recovery` é opcional/null. Configuração via upsert/edit_scene, inclusive em conteúdo gerado; omissão preserva cadência/status anteriores. Campos obrigatórios quando presente:

| Campo | Limites/semântica |
|---|---|
| retry_ticks |1..3600; espera após primeira falha. |
| max_retry_ticks |retry_ticks..3600; teto do dobro exponencial por falha consecutiva. |
| stalled_ticks |12..3600; janela sem deslocamento mínimo após comandos efetivos de movimento. |
| min_progress |0,01..1m finitos, até metade do deslocamento nominal speed*stalled_ticks/60. |

Falhas no caminho, superfície do NPC/alvo ou ausência de progresso entram em `navigation.status:retry_wait`, velocidade zero e caminho vazio. `reason` preserva no_clear_path/npc_off_surface/target_off_surface ou no_physical_progress. Durante espera não há reconstrução do mapa/A*. No prazo, tenta novamente o destino conhecido mais recente. Alterar apenas a posição de uma pista dentro do mesmo objetivo não fura a espera; uma troca de objetivo/território, ausência de destino ou disabled limpa a recuperação. Percepção e budgets continuam contando normalmente. Patrulha e investigação mantêm seus próprios prazos; podem desistir antes da próxima tentativa. Com política, retry_wait não é o unreachable imediato que antes descartava pontos de patrulha/busca; o orçamento da etapa continua limitando a tentativa.

`inspect_navigation.npcs[id].recovery` (também em inspect/replay/seleção) contém consecutive_failures, failures, path_attempts, retry_at_tick, last_failure, last_failure_tick, progress_anchor, progress_since_tick e movement_requested. last_failure é no_clear_path/npc_off_surface/target_off_surface/no_progress; reason usa no_physical_progress no último caso. Contagens saturantes e histórico do último erro persistem até idle/disabled/reset/troca de objetivo; sucesso físico/chegada limpa apenas sequência/prazo. path_attempts conta chamadas ao planejador, não consultas locais de contato. Campos de progresso refletem observação pré-física; movement_requested descreve o comando do passo. Estado inicial zerado não indica falha. Sem política, recovery permanece default.

Progresso mede deslocamento XZ desde a âncora, não distância percorrida nem aproximação do destino. Um corpo que oscila abaixo do limiar pode ser considerado parado; um corpo empurrado além dele conta como progresso. Chegada/liberação durante espera só é reavaliada no próximo prazo, exceto transições dos sistemas superiores. Não há teleporte, consulta adicional ao alvo oculto, escolha de destino alternativo, prioridade ou garantia de passagem entre NPCs. A guia npc_routes/J colore o X de retry_wait em vermelho pelo mesmo estado inspecionável no MCP. Cena17 aceita versões anteriores sem a política; replay17 exige o mesmo build/runtime.


## Reserva cooperativa de passagens — cena/replay18

`set_passages {passages:[...]}` substitui a lista autoral; a mesma operação existe em edit_scene. Validação final atômica, reset/pausa e um undo seguem o dispatcher comum. `npc.passage_priority` u8 opcional habilita participação explícita:0 também participa; ausente não coopera e permanece obstáculo físico. Facção não ativa cooperação automaticamente. Todos os participantes usam a infraestrutura de passagens declarada para sua superfície. Criadores/geradores podem produzir a lista usando o mesmo contrato.

Até8 passagens, com id único1..64bytes, surface referindo corpo fixo, a/b pontos XZ finitos alinhados a X ou Z e separados2..16m, width0,8..8m. a/b delimitam o corredor protegido, não precisam coincidir com a espessura visual da parede. width deve acomodar maior corpo participante+margem0,12m. wait_a/wait_b têm1..8 pontos por lado, pelo menos o número total de participantes na superfície. Pontos ficam além da entrada/saída no próprio lado e lateralmente fora da faixa, separados pelo maior corpo+margem. Saídas/esperas cabem no piso e territórios dos participantes; envelopes de passagens na mesma superfície não se sobrepõem. Não há validação de conectividade/obstáculos estáticos nesses pontos: falhas físicas continuam inspecionáveis no navegador.

lease_ticks30..3600 é o prazo máximo de cada concessão, retry_ticks1..3600 a espera após vencimento, aging_ticks1..3600 a cadência de envelhecimento. Pontos de espera são reservados uma vez por solicitante/lado, em ordem de ID, até cancelar/concluir. Nenhuma fila/histórico cresce com tempo.

Pedido considera apenas posição própria e destino já conhecido: o segmento de intenção deve cruzar o meio da faixa dentro da largura, com destino além da saída e folga do corpo. Escolhe a passagem candidata mais próxima (empate id). Este recorte não descobre gargalos nem deriva uma cadeia de passagens do caminho A*. Uma intenção cujo segmento não cruza a faixa pode ter desvio planejado passando nela; o gate bloqueia a entrada sem reserva e expõe waiting_passage/reservation_required. Esse caso exige ajustar autoria/rotina; não há autoencadeamento geral.

Sem dono/ocupação dinâmica adicional, vence maior `prioridade_base + floor(tempo_na_fila / aging_ticks)`, empate por ID. Reserva já concedida não é preemptada. Uma reserva por NPC; em espera ele navega ao ponto lateral, com colisões/recuperação normais. Cruzamento navega até saída com folga de corpo; liberação exige posição física além da face+meio corpo+0,05m. Mudança de objetivo, ausência de destino ou disabled cancela; vencimento libera reserva, mantém idade do pedido e impõe cooldown para recuar/ceder. Se um corpo continua dentro, nova concessão fica bloqueada. Chegar ao waypoint de espera não conclui o objetivo original.

Percepção e relógios da investigação/rotina continuam. Movimento de espera/cooldown não consome distância investigativa; travessia respeita o orçamento restante. Alteração XZ do waypoint invalida caminho/feedback e retentativa anterior; acomodação vertical da física não provoca replano por si. Gate barra comando de movimento para faixa não reservada, inclusive desvios do planejador; corpo já dentro pode recuar ao lado de entrada/mais próximo. Não é colisor invisível: empurrões físicos continuam possíveis. Jogador/NPC não participante não obedece ao gate e continua bloqueando por colisão/ocupação.

`inspect_passages` retorna tick, policy (lista), runtime e owners (IDs/null na ordem da lista). runtime.slots expõe owner (índice de entidade), granted_tick, occupied (corpo dinâmico além do dono), grants/timeouts. runtime.agents por ID contém entity_index, passage (índice/null), side (-1:a,+1:b), slot, phase idle/waiting/crossing/cooldown, reason no_request/queue/granted/occupied/lease_expired/completed/cancelled, requested_tick/retry_at_tick, effective_priority, original_goal, waypoint e contadores grants/completions/timeouts. Sem pedido ativo, timers/prioridade/slot preservam último valor; phase/passage determinam validade. Estado é pré-física, como navegação; posição atual está no inspect. Snapshots/replay incluem runtime; run_replay compara passages_matched. Reset/load/edição/undo/redo recriam estado; replay18 exige build correspondente. Campo optional ausente preserva cenas antigas.

Guias por npc_routes/J: retângulo tracejado cinza livre, verde reservado, vermelho com ocupação adicional; X amarelos de espera. Guias nunca decidem gameplay. Reservas globais de infraestrutura não entregam pistas de jogador nem alteram memória sensorial. Prioridade de tráfego é distinta da prioridade de objetivos. Aumento por espera reduz preferência recorrente por recém-chegados, mas não garante progresso sob bloqueio físico persistente, cancelamento dentro do corredor ou rede de passagens mal autorada.


## Rede espacial — cena/replay19

- `set_spatial_network {network:{nodes:[[x,y,z],...],links:[{from:0,to:1,mode:"jump"},...]}}`, inclusive dentro de `edit_scene`. Nós representam **pés**, não centro do corpo; links dirigidos (reverso precisa outra entrada), modos walk/jump/glide/fly, até64 nós/128 links, extensão0,1..32m. Nenhuma rede é inferida das caixas.
- `npc.mobility:{walk:true,jump_speed:7,fly_speed:null,glide:null}` opt-in. Valores opcionais em m/s,0,1..10; glide `{speed:3,sink_speed:1}` limita velocidade horizontal e fixa descida. Capacidades ausentes proíbem links. Scene npc.surface permanece referência legada válida; rede espacial não fica confinada a ela.
- `objectives:[{priority:1,when:"always",action:{kind:"guard_3d",point:[3,2.8,-4],arrival_distance:0.2}}]`: point é **centro desejado**; motor converte usando altura do NPC. respond/wait podem usar conhecimento usual. Combinações ainda planas são rejeitadas explicitamente, sem conversão silenciosa.
- `inspect_mobility`: network, states, compiled_links e rejection_reasons por NPC/conexão. None na compilação indica missing_capability ou static_geometry_or_motion_constraint. Runtime: phase, reason, edge, route, launch_position/motion, started_tick/retry_tick, completed/plans, destination (pés), committed. snapshot geral inclui mobility; replay inclui expected_mobility e mobility_matched.
- Falhas: not_anchored_to_network (>0,35m), target_not_on_network, no_compatible_route, launch_geometry_blocked, insufficient_travel_budget, traversal_timeout. Geometria nominalmente inválida pode ser rejeitada por corpo/capacidade na compilação; documento válido não promete conectividade. Timeout pode evoluir para falta de ancoragem após nova tentativa.
- `set_debug_view {npc_routes:true}`/J mostra links coloridos: walk cinza, jump laranja, glide verde, fly azul; branco é rota escolhida, arco laranja é trajetória balística nominal ativa. Não são linhas de visão. Camada de decisão não consulta esses guias.
- P inicia/pausa e R reinicia como antes. A demo de capacidades usa objetivos fixos; jogador continua caminhada WASD. Esses NPCs não estão configurados para perseguir o jogador.


## Aproximação parcial — cena/replay20

`npc.unreachable_policy` via `upsert`/`edit_scene`:

- Ausente ou `{ "mode": "stop" }`: exige caminho completo, comportamento anterior.
- `{ "mode": "approach_reachable", "min_progress": 0.25, "max_path_ticks": 300 }`: quando não existe caminho completo compatível, aproxima até um nó alcançável. Exige `npc.mobility`; cena antiga com a política é rejeitada.
- min_progress finito de0,05 a10m: redução mínima de distância XYZ ao objetivo conhecido, com margem adicional de0,12m para tolerância de chegada. Não é distância percorrida.
- max_path_ticks inteiro de1 a3600: limite da soma de custos nominais `ceil(segundos/FIXED_DT)` das conexões parciais. 300ticks equivalem a5s estimados a60Hz. Não limita uma rota completa nem substitui memória/budget investigativo ou timeout físico.

`inspect_mobility.policies[id]` mostra a política efetiva (stop inclusive quando omitida). `states[id].partial_target` é índice de nó ou null; `destination` continua sendo o objetivo original em coordenadas dos pés. `planned_stop_distance` é o raio associado ao plano; `route` conserva as conexões restantes da aproximação parcial. `phase: partial_reached`, `reason: partial_route_end` e `navigation.status: unreachable` distinguem aproximação terminada de chegada real. Sem candidato aceitável: no_partial_progress. Contador plans não cresce parado no endpoint; novas pistas/raio/deslocamento/reset invalidam o plano conforme arquitetura.

`set_debug_view {npc_routes:true}`/J: X laranja adicional marca o endpoint parcial, destino original continua presente (vermelho quando inalcançável); rota atual branca. Nenhum controle novo exclusivo de UI. `python3 scripts/verify_partial_routes.py` atualiza a demo pelo MCP e valida política, histórico, limites, erro atômico, replay/prefixo e guias. Se executar verify_mobility isoladamente, ele recria a demo básica sem fallback; execute verify_partial_routes em seguida para restaurar a demonstração de aproximação.


## Conexões locais — cena/replay21

`npc.local_connections: {"walk":true,"fly":false,"max_distance":4.5}` via upsert/edit_scene. Exige mobility, pelo menos um modo ativo com capacidade correspondente e alcance finito0,25..8m. A comparação física admite2mm de tolerância de contato, igual à chegada ao nó; colisões não são relaxadas. Campos obrigatórios; ausência desativa conexão automática. Rede pode estar vazia quando a política existe: só destino direto alcançável será resolvido. Não habilita salto/planeio automaticamente nem amplia suporte de footsteps/território/patrulha planares.

`inspect_mobility.local_connections[id]` mostra config/null. `states[id].connector` é null ou `{kind:"entry"|"exit"|"direct",mode:"walk"|"fly",endpoint:[x,y,z]}`; endpoint representa pés. Durante conector edge é null; route contém links autorados restantes do plano. destination continua objetivo conhecido original. committed informa conclusão do trecho após mudança/perda da intenção. completed inclui trechos locais e autorados. Sem caminho: no_local_route; lançamento inválido: local_geometry_blocked; insufficient_travel_budget/traversal_timeout mantêm semântica anterior. `npc_routes`/J mostra X ciano no endpoint local; path e destino permanecem inspecionáveis. Estado e configuração entram em replay e histórico.

`python3 scripts/verify_local_connections.py` cria a demo `examples/npc-connectors.scene.json` pelo MCP. P inicia, R reinicia, J alterna rede/caminhos, V alterna visão. Roxo persegue o jogador laranja com percepção360°, distância16m e memória3s; audição16m/N e busca1s. O FOV360° isola a navegação, não implica visão através de paredes. Conectores de caminhada até4,5m e rede explícita contornam a parede. Azul tem objetivo fixo no ar, voo3m/s, conexões locais até6m e rede acima da parede. Player não emite passos automaticamente nessa cena. Corpos/destinos fora do alcance conectado ainda podem ficar inalcançáveis; isso não é navegação livre global.

## Apoio contínuo de caminhada (EAF-34)

Links `walk` e conexões locais usam a união das faces superiores de caixas fixas. O trecho pode atravessar peças encostadas na mesma altura: não precisa caber em uma única entidade. A projeção varrida de toda a largura/profundidade do corpo deve ter apoio; testes de colisão do corpo, alcance, capacidade e tempo de movimento continuam separados. Não introduz rampa, escalada de degrau ou uso de corpos móveis como chão. `npc.surface` continua exigido pelos contratos legados; mobilidade espacial pode caminhar sobre outras caixas fixas.

Tolerância vertical de contato6cm e margem de borda1cm preservadas. Na união composta a margem reduz o footprint, sem expandir cada piso e fechar buracos. Fast path aceita uma caixa suficiente; caso contrário, até64 superfícies potencialmente relevantes. Exceder o limite recusa conservadoramente, mesmo se uma união maior pudesse fornecer apoio. A geometria continua limitada a512 entidades por cena.

`inspect_mobility.rejection_reasons[id][índice_do_link]` distingue `walk_support_gap`, `walk_height_mismatch` e `walk_support_limit`. Falha restante de colisão/movimento continua `static_geometry_or_motion_constraint`. Conexões locais conservam `no_local_route` quando nenhuma alternativa é viável; não expõem todos os motivos de cada candidato. Posições consultadas são pés; destino do objetivo/patrulha continua centro do corpo. Autoria existente `upsert`/`edit_scene`/`set_spatial_network`, inspeção, histórico e replay pelo MCP. Cena/replay24 preservados; replay requer o mesmo build. Demo procedural por parâmetros no roteiro `verify_walk_support.py`.

## Voo curvado — cena/replay22

Configurar `npc.flight` via upsert/edit_scene. Ausente ou `{"mode":"straight"}` preserva voo reto anterior. Exemplo curved:

```json
{"mode":"curved","acceleration":3,"deceleration":3,"turn_degrees_per_second":45,"smoothing_seconds":0.4,"minimum_speed":1,"can_hover":false,"initial_direction":[0,0,-1]}
```

Exige mobility.fly_speed (máximo0,1..10m/s). acceleration/deceleration:0,1..30m/s²; giro15..360°/s; suavidade0..2s; minimum_speed0..fly_speed, mínimo0,1 quando can_hover=false; initial_direction finita/não nula, módulo0,01..100, normalizada no lançamento. Campos curved obrigatórios, validação antes de mutar. Velocidade mínima é de cruzeiro: partida e manobras de pairadores podem ficar abaixo dela. Straight não aplica aceleração/filtro novos.

inspect_mobility.flight_profiles mostra perfil efetivo; states[id].flight/flight_path mostram controlador/última previsão reduzida. flight.velocity é comando do motor, não medição de velocidade pós-colisão. holding=true significa órbita após passagem pelo destino; phase:arrived nesse caso não exige permanecer no raio. Sem pista: holding_without_known_goal; rota indisponível: holding_while_route_unavailable. Motivos de falha incluem flight_path_blocked, flight_preview_limit, flight_holding_blocked, flight_holding_preview_limit, flight_initial_speed_exceeds_limit, flight_tracking_interrupted, flight_path_interrupted e flight_traversal_timeout. Perfil impossível para o espaço pode falhar mesmo que o segmento reto esteja livre.

`python3 scripts/verify_flight.py` cria examples/npc-flight.scene.json pelo MCP. Azul reto, roxo curvado com parada/pairar, verde curvado sem pairar com órbita; todos têm destinos fixos no ar. Não perseguem o jogador laranja. P inicia, R reset, J guias de curva violeta/direção amarela. Curvos: máximo3m/s, aceleração/frenagem3m/s², giro45°/s, suavidade0,4s; verde mínimo1m/s. Straight chega antes porque mantém seu controle anterior de velocidade imediata. Guia não muda simulação; replay e histórico incluem configuração e estado de voo.


## Visão volumétrica de NPC (cena/replay23)

`upsert`/`edit_scene` aceitam `npc.spatial_vision: {"initial_pitch_degrees":35,"follow_movement":false}`. Campo ausente preserva visão horizontal; pitch deve ser finito em[-90,90]. Yaw inicial, alcance e abertura total continuam em awareness.initial_facing_degrees, vision_range e fov_degrees. Positivo olha acima; cone360° equivale a sensor angular esférico, com alcance e oclusão ativos. Omniscient continua ignorando limites sensoriais por escolha explícita.

follow_movement true acompanha direção comandada pelo motor emXYZ; false mantém orientação durante locomoção. A busca pode girar yaw mantendo pitch em ambos os casos. inspect_npcs/inspect trazem pitch, flag de acompanhamento e vision_direction; não confundir direção sensorial com rotação física/visual do corpo. `set_debug_view {npc_vision:true}` / V mostra seta e volume nominalXYZ; mesma convenção de cores dos guias legados. Toggle não modifica sensor/simulação.

`python3 scripts/run_verifications.py --only spatial_vision` valida sem janela no workspace isolado. `examples/npc-spatial-vision.scene.json`: jogador laranja sobre plataforma, observador azul horizontal e roxo com pitch35°, FOV45°, alcance9m, yaw apontado para a plataforma. Os dois usam objetivo wait e ficam parados intencionalmente; P executa percepção, V mostra guias. Roxo detecta o jogador no início (verde); azul não. R reinicia. Esse exemplo isola visão, não perseguição/combate.

## Comunicação por rádio (cena/replay25)

Componente opcional em npc, disponível nos schemas de upsert/edit_scene; ausência/null desliga. Exemplo:

```json
{
  "communication": {
    "mode": "radio",
    "channel": 1,
    "range": 14,
    "cooldown_ticks": 60,
    "max_age_ticks": 180,
    "send": true,
    "receive": true
  }
}
```

Canal inteiro1..65535, alcance finito0,1..100m, cooldown e idade máxima inteiros1..3600 ticks (60 ticks/s). Booleans obrigatórios. Awareness deve ser perceptivo (padrão quando ausente). O relato sai um tick após visão confirmada; max_age_ticks=1 expira antes da entrega. Não use esse valor para uma comunicação efetiva. Configuração nova exige cena25; cenas anteriores carregam sem rádio e são normalizadas para25. Replay25 requer mesmo build.

Alianças mútuas, mesmo canal e distância XYZ até min(alcance do emissor, alcance do receptor) são obrigatórios. Não exige linha de visão nem hearing; não transmite sons, onisciência ou outros relatos. Alvo explícito restringe a identidade recebida; hostile_nearest requer alvo hostil. Sem seleção de alvo, um NPC pode investigar a identidade reportada. Território limita o ponto transmitido aceito e impede comunicação durante retorno/cooldown.

inspect_npcs e inspect expõem communication: sent_this_tick, received_this_tick, next_send_tick, sent, accepted, rejected (neste tick), reason e last_received. O recibo traz sender_id, target_id, position, observed_tick, received_tick, expires_at e source:vision (origem do emissor); memory.source:report indica conhecimento indireto no receptor. O último recibo permanece como diagnóstico após expirar; memory é a evidência ativa. Reset/edição/load/undo o limpam.

Razões: disabled, no_report, direct_evidence, existing_evidence, filtered, accepted. sent conta tentativas de transmissão válidas, mesmo sem ouvintes; rejected conta relatos de outros emissores recusados pelos filtros, não ausência de rádio ou prioridade de evidência direta. Expiração é exclusiva: tick >= expires_at. Novos relatos com instante maior podem substituir relato ativo; nunca memória direta ainda válida. Investigar rádio não conta como visão/audição nem como confirmação visual. O objetivo evidence pode reagir; visual_memory/sound_memory não se tornam verdadeiros por rádio.

Demo examples/npc-radio.scene.json: roxo observa parado e transmite no canal1, azul investiga atrás da parede, cinza usa canal2 e só reage à própria visão. Tudo gerado pelo MCP em scripts/verify_communication.py. Sem áudio de dispositivo, gritos, retransmissão, reputação ou protocolos de rede.


## Habilidades compartilhadas (cena/replay26)

Contrato completo, limites, ordenação, erros e demo em [abilities.md](abilities.md). Cena26 normaliza cenas antigas sem adicionar habilidades; replay26 exige o mesmo build. `npc_routes` também controla barras de recursos/cast; `noise_events` inclui sinais gerados por habilidades. O formato JSON continua na autoria/inspeção/transporte MCP; o executor usa índices/arrays/enums compilados.


## Movimento de cast (cena/replay29)

`set_ability_catalog`/`edit_scene` aceitam `abilities[].movement`: `{"mode":"free"}` (omitido por padrão), `{"mode":"interrupt","speed_tolerance":0.2}` ou `{"mode":"lock"}`. Tolerância0,01..2m/s; rejeição atômica de versão/valor/campos inválidos. `inspect_abilities` publica política dos grants, `movement_locked`, `motion` e resultados `moving`, `interrupted_by_movement`, `movement_lock_unavailable`. Entrada, ativação e cancelamento existentes atendem os três modos; nenhuma tecla ou API exclusiva. [Contrato, ordem física e demonstração](cast-movement.md).


## Entrega por projétil (cena/replay30)

`set_ability_catalog`/`edit_scene` aceitam `abilities[].delivery`: `{"kind":"direct"}` (default) ou `{"kind":"projectile","speed":6,"half_extent":0.14,"max_distance":18}`. Mesmos `activate_ability`, `cancel_ability`, `inspect_abilities`, reset e replay. Inspeção inclui `projectiles.active`, `projectiles.receipts` e descarte/capacidade, com IDs públicos resolvidos. Recibos distinguem launched/impacted e projectile_budget/invalid_aim. [Limites, cobrança e colisão](projectiles.md). Cubos renderizados são observáveis também por capture/capture_replay offscreen; não há configuração exclusivamente visual.

## Autoria nativa de malhas

`edit_mesh`, `inspect_meshes`, `inspect_mesh` e `inspect_mesh_pose` compartilham o dispatcher com o MCP. Operações, esquemas, limites, reversão e demo em [mesh-authoring.md](mesh-authoring.md). A operação `mesh` também participa de `edit_scene`; autoria reseta execução e animação segue ticks fixos.

`inspect_mesh_islands {asset}` retorna ilhas conectadas por índices, bounds,
contagens, vértices isolados e referência `scope/revision`. Para separar peças:
`edit_mesh {edit:{kind:"chamfer_boxes",id,boxes:[{center:[x,y,z],size:[w,h,d],rotation:[x,y,z,w],bevel}],append:false}}`
gera caixas com chanfro plano nativo. Quaternion default identidade; bevel0
gera8v/12t e positivo24v/44t. Até512peças, parâmetros/contagens antes da expansão;
preview_mesh_edit/lotes/undo/erros têm o contrato comum. Append preserva fonte
sem rig/clips e exige asset existente; planta vinculada exige detach. Sem receita
persistida nem bevel genérico de região. [Contrato](chamfer-boxes.md) e exemplo
`examples/chamfer-boxes.edit.json`.

`edit_mesh {edit:{kind:"extract_islands",asset,id,reference,islands,mode:"move"|"copy"}}`.
Destino novo; Move deixa faces na origem, Copy mantém a origem. Espaço local,
canais, pesos e todos os ossos/clipes permanecem; nenhuma instância é criada.
`preview_mesh_edit {edit}` prevê qualquer mesh edit com a mesma validação sem
alterar cena, seleção ou execução. Regras completas em [mesh-islands.md](mesh-islands.md).

`set_material` aceita `detail:{plane:"xy"|"xz"|"yz",tint,opacity,bump,segments}`,
com até32segmentos `{start:[u,v],end:[u,v],widths:[raioInicial,raioFinal]}` no
domínio de repouso. Cor/relevo são independentes do ruído. Ausência é neutra;
bump assinado muda normal, nunca geometria. `fit_material_coordinates` em
`edit_mesh` recebe `asset`, `per_island` (false padrão) e `bounds:[[x,y,z],[x,y,z]]`.
Ajusta o canal existente (fallback posição) por AABB; não altera rig/mesh nem
infere orientação. Preview genérico e undo existentes. Formato59/63 e contrato
completo em [surface-details.md](surface-details.md).

`copy_material_coordinates {asset,source,per_island}` copia o canal explícito
de outro asset. Modo inteiro exige mesma topologia ordenada; por ilha exige
uma ilha na fonte e correspondência exata em todas as ilhas destino, sem
isolados. Contagens/triângulos incompatíveis falham. Não transforma geometria,
infere orientação, faz nearest ou cria vínculo vivo com a fonte. Preview e
undo existentes; em planta é edição de canal com rebase declarado.

`set_material` também aceita `reflectance:{roughness:0.58,f0:0.035}` opcional.
Rugosidade perceptual finita0.2..1 e F0 dielétrico escalar0..0.16; F0zero ou
campo ausente mantém resposta anterior. Reflexo da luz direta depende da câmera,
normal, luz e sombra existentes; não é reflexo de objetos do cenário. Mesmo
preview/inspect/undo, sem geometria nova. Scene60/replay64 locais à frente de
mesh; integrar com versões funcionais concorrentes exige unificação explícita.
[Contrato, custos e limites](dielectric-response.md).

- `inspect_mesh_boundaries {asset}`: consulta pura de ciclos de borda orientados,
  centros e referência de sessão/revisão. Bordas ambíguas/não manifold falham.
- `edit_mesh`/`preview_mesh_edit`, `kind:"remove_faces"`, `asset`, `reference`,
  `faces:[u32]`: faces atuais únicas/não vazias; compacta vértices/pesos/canais
  usados em ordem e deixa ao menos uma face.
- `kind:"bridge_boundary_loops"`, `asset`, `reference`, `loops:[Vec<u32>;2]`,
  `spans:2..32`, `handles:[f32;2]` em0.0001..2m: ciclos disjuntos de3..64pontos,
  mesma contagem. Primeiro reverso da borda, segundo segue a borda; fase explícita.
  Pontes compartilham os índices antigos. Recusar winding/dobras/budget/rig>4
  e ângulos>=90° entre faces adjacentes envolvendo a ponte;
  plantas vinculadas exigem detach explícito. Contrato em mesh-bridges.md.


## Adoção de trajetória

`edit_plant`/`preview_plant` aceitam edit.kind `adopt_trajectory`, com definition,
detail_policy e animation_translation_policy. Exige vínculo legado sem eixo,
conserva source/reference e mantém IDs. Put posterior exige o mesmo ID de eixo.
`inspect_meshes` retorna limits com budgets de assets/instâncias/armazenamento/desenho.
[Contrato e versões](tree-trajectory-integration.md).

- `preview_mesh_junction_union {source,ports,root,options}`: consulta pura de
  união tubular com três contornos reais em um Asset recortado. Retorna asset
  e diagnósticos separados de costura fonte/conexão/interior; publicação explícita
  por `put_asset` valida orçamento global. Prefixo antigo exato, miolo novo,
  raiz rígida nominal nas bordas ou sem rig. Contrato e domínio recusado em
  [implicit-junctions.md](implicit-junctions.md); não é boolean geral.
  Refinamento opt-in em options: `seam_relaxation_steps`0..8 com
  `seam_max_displacement` positivo até min(cell_size/2,collar_length), e
  `material_relaxation_steps`0..64 com canal existente. Defaults0 conservam
  chamadas anteriores. Apenas pontos iso próximos ao zipper se movem; fonte e
  primeira guarda geométrica fixas. Domínio das guardas copia fonte e correção interna é
  finita, com resíduo reportado; não certifica fibras físicas ou realismo.



## Composição regenerável e preview de lote — local66

`set_assembly` em edit_plant/preview_plant recebe id e assembly (ou null).
Assembly contém context estático canônico,context_faces,context_loops[2],station,
context_bone opcional e options do kernel tubular. Retém o corpo após uma seção
nomeada, conecta três portas e conserva fontes; nenhum bake automático.
`preview_scene_edits {edits}` compartilha staging/guardas do edit_scene, valida
até512operações e retorna `{scene}` sem alterar revisão/execução/histórico.
Recorte+remoção do temporário+instalação podem ser previstos e publicados como
um único lote. Contrato/custos/limites em [plant-assemblies.md](plant-assemblies.md).

Asset passa a8192v16384t; totais16384v32768t e draw131072 permanecem.
Scene65 rejeita nova receita ou asset maior, inclusive fonte canônica;66 admite,
replay70 local. Fonte de formato funcional/main ainda exige integração separada.


## Comparação visual de referência

`compare_images {reference_path,candidate_path,mask_path?,regions?,difference_path?}`
compara PNGs alinhados no mesmo grid sem alterar cena ou execução. Retorna
RGB MAE/RMSE, igualdade exata e SSIM de luminância com cobertura por região;
mapa de erro opcional em PNG. Máscara binária, paths relativos e saídas que
não substituem entradas. Não ajusta câmera/escala/cor nem fornece porcentagem
de fidelidade3D. Contrato, limites e reprodução em [image-similarity.md](image-similarity.md).

## Curvas de seções e acabamento de superfície

`preview_mesh_ring_loft {source,lofts}` é uma consulta pura. Cada loft declara
anéis da fonte por índices/colunas consistentes, spans1..16,
`interpolation:linear|cubic` e `max_deviation` em metros. A cúbica usa parâmetros
centrípetos entre centros e tangentes compartilhadas; interpola as colunas das
seções, não promete círculos exatos entre elas. Relatório distingue bound do
polinômio e erro amostrado após f32. Vértices antigos/domínio/pesos ficam exatos;
novos pesos são interpolados pelos endpoints, com recusa de mais4influências.

`preview_mesh_surface {source,vertices,anchors,options}` é puro e retorna asset
e relatório. `edit_mesh {edit:{kind:smooth_region,asset,reference,vertices,anchors,options}}`
publica com referência de sessão. `preview_mesh_edit` e `preview_scene_edits`
servem para prever a publicação. Options declara iterations1..32, strength(0,1]
e max_displacement1e-6..100m; fora da seleção, âncoras e bordas reais ficam fixos.
O resultado pode encolher. Erros identificam aresta/face e ângulos quando há
dobra/inversão; não existe fallback para relaxar as guardas.

Em assembly, `context_lofts` e `context_surface` usam IDs da fonte canônica;
`junction_surface` recebe só options e deriva seu domínio novo por origem.
Novos pontos do loft herdam seleção/pino se ambos os endpoints o possuem.
São campos opt-in Scene67. Escultura direta na composição continua recusada;
use a receita para preservar regeneração. Ver [organic-continuity.md](organic-continuity.md).

### Enxerto lateral persistido na planta

`edit_plant {edit:{kind:"set_assembly",id,assembly}}` e os previews existentes
aceitam `assembly.context_grafts`. Cada entrada contém `source`,
`source_translation`, `source_faces`, `source_loop`, `context_vertices`,
`context_loops`, `options` e `surface` opcional. Os índices do contexto são
canônicos; IDs do miolo avaliado não são aceitos. Ver
[contrato completo](context-grafts.md). Remover a fonte filha do asset original
no mesmo `edit_scene` evita duplicação; `preview_scene_edits` permite revisar o
lote inteiro antes da publicação, que tem um undo único.

`assembly.context_grafts[].collar_surface` (Scene69) aceita `ports`, `rings`,
`fixed_rings`, `options` por `set_assembly` e seus previews. O alcance é por
arestas, com camadas externas fixas e proteção das outras portas; opções de
acabamento são as de SmoothRegion. A seleção é reconstruída pela engine após
cada união/loft. Campo ausente mantém comportamento anterior. Esse acabamento
muda intencionalmente pontos avaliados na região, preservando fontes/canais/rig.

### Acabamento material das plantas

`preview_plant` e `edit_plant`/`set_assembly` compartilham
`Assembly.plant_material_frame`, `Assembly.material_finish` e, por enxerto,
`source_material_frame`/`material_finish`. Campos opcionais requerem Scene70.
Transformam uma cópia do domínio e finalizam depois da geometria, mantendo
fontes/posições/rig e reversão explícita; contrato em context-grafts.md.
`preview_mesh_material_finish {source:Asset,plan,options}` oferece o mesmo kernel
puro com proveniência explícita para assets livres, sem alterar cena/histórico.

`set_material`/`preview_mesh_edit` aceitam `detail.triplanar:{}` para ativar
sharpness2 (ou `{sharpness:1..8}`); ausência mantém planar. Persistência70 e
replay74. Campo declarado em formatos anteriores recusa inclusive nas fontes
ocultas. Troca de aparência conserva geometria/rig e pode ser desfeita; misturar
projeções não certifica continuidade de fibras ou realismo da árvore.


### Perfis e repetição de traços — Scene71/replay75

`set_material` e seus previews aceitam `detail.profile:"band"|"rounded"` e
`detail.repeat:[period_x,period_y]` opcional. Band/None preservam o comportamento
anterior e são omitidos ao salvar. Rounded oferece seção suave para tinta/bump;
repeat reutiliza um patch2D no plano escolhido ou em cada plano triplanar.
Períodos finitos0.001..10000; pontas com margem2×seu raio dentro da célula.
Traços cruzando a célula são recusados, não aparados silenciosamente. Relevo
assinado/limites32segmentos permanecem; não muda geometria/sombra/colisão.
Declarações habilitadas são recusadas em cenas≤70, inclusive fontes ocultas.
Contrato/limitações em [detail-domains.md](detail-domains.md).


## Projetos — EAF-109

- create_project {path absoluto, name, discard_changes:false} reserva pasta nova
  e cria manifesto/cena Principal; não sobrescreve pasta existente.
- open_project {path absoluto, discard_changes:false} valida antes de trocar raiz.
- inspect_project mostra manifesto/raiz/cena ativa/modified/limites.
- new_project_scene {name, discard_changes:false} cria arquivo e registra no manifesto.
- open_project_scene {path relativo, discard_changes:false} abre cena registrada.
- set_project_entry {path relativo} seleciona cena inicial; save_scene salva a ativa.
- create_scene_object {kind:visual|fixed|dynamic}, duplicate_scene_object {id};
  upsert/remove/undo/redo continuam compartilhados.

Trocas novas recusam alterações não salvas; discard_changes precisa ser explícito.
Manifesto1 e layout2 independentes de Scene75/replay79. Apenas create/open_project
aceitam caminhos absolutos; demais arquivos continuam confinados à raiz ativa.


## Primitivas e seleção de malhas no editor

`create_editor_mesh {primitive:"sphere"|"cube"|"cylinder"|"plane",resolution:4..32,size:0.01..100}`
cria asset e instância atomicamente, seleciona a instância e exige simulação
pausada. `size` é diâmetro/lado/altura em metros; plano usa largura/profundidade.
A UI oferece um intervalo menor de tamanho, sem alterar o contrato do comando.

`select_editor_mesh {instance:ID|null}` altera somente a seleção de sessão,
mutuamente exclusiva com a seleção de entidades. `inspect_editor.selected_mesh`
expõe essa seleção; load/troca de cena não a persiste. `pick_editor_mesh
{x,y,width,height}` consulta o triângulo mais próximo de uma malha renderizada,
com câmera do editor, coordenadas físicas relativas à viewport, ponto mundial,
ponto local, normal e índice de face. É consulta somente de malhas; a seleção
`select_editor_at` compara também entidades. Conexões de planta consumidas não
são duplicadas no picking. Escultura continua usando `edit_mesh/preview_mesh_edit`
com `kind:"sculpt"`; não há executor visual paralelo nem mudança de Scene/Replay.

### Segundo padrão procedural — Scene73/replay77

`set_material`/`put_asset` e seus previews aceitam `secondary_pattern` opcional,
com os mesmos campos/bounds de `pattern`. Também funciona sem o primeiro.
Frequência, seed, kind, warp, octaves, contrast, paleta e bump independentes;
ambos usam o M de repouso. Tintas multiplicam e alturas somam; bumps dos padrões
somados não podem exceder0.05m (Detail mantém o contrato próprio). None preserva
o caminho anterior, inclusive pixels; declarações em cenas≤72 recusam antes da
avaliação das receitas, inclusive fontes ocultas.

Exemplo: `{"kind":"set_material","asset":"stone","material":{"secondary_pattern":{"frequency":[1,1,1],"low":[0.7,0.75,0.65],"high":[1.15,1.1,1.05],"bump":0,"warp":0.4,"octaves":2,"seed":9,"contrast":2}}}`.
Lote/preview/inspeção/undo/load/replay usam as operações existentes. Contrato em
[layered-patterns.md](layered-patterns.md); teste na casa em
[cottage-materials.md](cottage-materials.md).

### Metalness — Scene74/replay78

`material.reflectance` aceita `metallic:0..1` finito, default0 omitido ao salvar.
Exemplo: `{"kind":"set_material","asset":"bronze","material":{"reflectance":{"roughness":0.4,"f0":0.12,"metallic":1}}}`.
Funciona em set_material/put_asset, previews/lotes e inspeções existentes.
Metallic>0 com transmission>0 recusa sem publicar; transmission não é alpha de
vidro. Metal puro usa baseColor*tints como refletância linear colorida (clamp0..1)
e não depende de f0 dielétrico; roughness permanece0.2..1, f00..0.16. Default0
preserva pixels legados e convenção de f0=0 que desliga o lobo dielétrico.
Metallic>0 em cena≤73 recusa inclusive fontes ocultas. Save/load/undo/replay
permanecem nativos; ambiente constante/ausência de IBL explícitos no contrato
[metallic-response.md](metallic-response.md).

### Ambiente distante e intensidade direta — Scene76/replay80

`set_light` e preview/lote aceitam `environment` opcional:
`{"direction":[-0.7,-1,-0.65],"ambient":0,"direct_intensity":0.68,"environment":{"sky":[0.6,0.75,0.95],"horizon":[0.8,0.75,0.6],"ground":[0.05,0.04,0.025],"axis":[0,1,0],"intensity":0.55}}`.
RGB linear0..1, axis componentes±1/norma≥0.01 normalizada, intensity0..4.
direct_intensity0..4 default1 omitido. None/I0/direct1 preservam pixels;
configuração environment, mesmoI0, ou direct!=1 em≤75 recusa atomicamente.
Ambiente soma à constante ambient existente; para só ambiente: ambient0/direct0.
Não configura fundo/oclusão/GI/reflexos de objetos nem importa HDR.

Capturas retornam `adapter.environment_preparation` com `probe_bakes`,
`lut_bakes` e `logical_texture_bytes`: diagnóstico do Renderer, não do estado da
simulação/replay. Intensity/câmera/materiais/animação não devem rebakear; trocar
cores/eixo muda a chave. Layouts/limites e reprodução em
[procedural-environment.md](procedural-environment.md).

### Inspeção de normais avaliadas

`inspect_mesh_normals{instance,tick?,faces?,sample_limit?}` consulta normais de
face/cantos efetivamente usadas pelo renderer em mundo, antes dos materiais.
Filtro refere índices do asset; todas as incidências participam do smooth.
`sample_limit`0..32/default8, tick0..36000/default atual; amostras por maior desvio,
empate por face. Não altera execução/histórico nem corrige junções. Retorna
reference de sessão, fallback/cantos opostos/área e normais. Contrato e limitações
em [mesh-normal-inspection.md](mesh-normal-inspection.md).

## Controles de personagens — EAF-122

`bind_control {actor}` vincula Player vivo por alias ou token de instância e retorna
`control`. `set_control_input {control,input:{x,z,mode?}}` altera somente seu input;
`release_control {control}` neutraliza/libera o vínculo; `inspect_controls` lista
jogadores, handles, input/perfil, vida e limites. Vínculo exclusivo, handles
escopados à execução e não reutilizados; nenhum comando grava controle na Scene.
Morte neutraliza input explícito sem soltar vínculo. `set_input` legado é recusado
com vários Player ou vínculo explícito. São controles locais administrativos,
sem autenticação/transporte remoto ou UI nova implícitos. Cena77/replay81;
[contrato, exemplos e gates](actor-controls.md).

Sweep.spans/plan_mesh_sweep aceitam inteiro ou lista por cúbica (valores1..128).
Plano conserva a forma de entrada e acrescenta curve_centerline_bounds. Lista
refina trechos independentemente; escalar mantém semântica anterior. Contagens
pela soma, seções independem do índice dos anéis. Sem novo tipo de cena/replay.
[Contrato](sweep-curve-sampling.md).

## Navegador e biblioteca de projetos — EAF-109/EAF-128

Todos os comandos passam pelo mesmo Service/dispatcher usado pela UI. Biblioteca e navegação são de sessão/filesystem; não editam Scene nem participam do undo de gameplay.

- browse_project_directory {query:{path absoluto, search:"", show_hidden:false, sort:"name"|"kind"|"modified", descending:false, after:null|cursor}}: página ordenada de até256entradas, next explícito, tipos e metadados. Cursor pertence à ordenação solicitada; diretório pode mudar entre consultas.
- create_project_folder {parent absoluto, name}: criar pasta exclusiva; colisão ou nome inválido não sobrescreve dados.
- inspect_project_library: recentes/fixados/favoritos, revision, warning e arquivo de preferências configurado.
- use_project_library {path absoluto}: carregar/configurar JSON limitado e validado; erro preserva a biblioteca anterior. MCP começa em memória; o editor configura o arquivo pessoal no startup.
- register_project {path absoluto}: validar manifesto e adicionar à biblioteca, deduplicando aliases canônicos.
- pin_project {path, pinned}: fixar/desafixar metadados, até16fixados dentro de32projetos.
- forget_project {path}: retirar somente o registro; os arquivos do projeto permanecem.
- set_project_favorite {path, favorite}: adicionar/remover pasta favorita, até16.
- discover_projects {paths}: descoberta superficial limitada a8raízes, quatro páginas por raiz e64candidatos, com found/errors/truncated explícitos.

Preferências1 e manifesto1 permanecem distintos de Scene86/replay90. Persistência por arquivo temporário/rename; falha ao salvar recentes não desfaz um projeto que abriu corretamente, retornando aviso. Ver [navegador](editor-file-browser.md) e [consolidação](main-consolidation.md).
