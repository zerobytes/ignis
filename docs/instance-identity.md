# Identidade de instâncias — EAF-60

Primeiro incremento executável da direção aprovada em [EAF-59](population-runtime-study.md). Cena46 permanece; replay47 acrescenta referências numéricas e o estado esperado do registro. O catálogo de atores ainda é fixo. Não há spawn/despawn publicado nem reciclagem de slots na simulação.

## Três conceitos separados

- `InstanceId`: serial u64 único dentro da execução. Identifica o indivíduo, independentemente de sua vida ou posição de armazenamento. Atribuição inicial por ID autorado ordenado; sequências iguais produzem IDs lógicos iguais.
- `Reference`: slot u16 + geração u64 + InstanceId. O registro confere os três antes de resolver a entidade. Identidade não contém posição, estado físico ou conhecimento sensorial.
- `life_index`: contador existente de ressurreições. Morte/ressurreição preservam instância, geração, corpo, recursos e cooldowns; a vida muda apenas no retorno.

Registro limitado a512 entidades, incluindo objetos sem ator. Inserção falha sem consumir identidade quando cheia/duplicada ou quando o serial esgota. Remoção invalida a geração; slot cuja geração alcança u64::MAX fica aposentado. Reuso é coberto por testes isolados, mas não é exposto pelo catálogo vivo: os consumidores restantes ainda guardam índices legados. O limite atual continua9 atores (um jogador e até8 NPCs).

## Interface compartilhada e MCP

`inspect_instances` retorna `execution_scope`, `tick`, `dynamic_population:false` e `instances`. Cada entrada contém `authored_id`, `instance_id`, `generation`, `reference`, `actor` e `life_index` (null para entidade sem ator). Seriais/gerações externos são strings hexadecimais de16 caracteres para não perder precisão em clientes JavaScript.

`reference` é um token opaco `ignis:<execution_scope>:<instance_id>`. Use o valor retornado sem reconstruí-lo. `activate_ability` aceita token em `actor` e `target`; `aim_ability`, `aim_ability_at` e `cancel_ability` aceitam token em `actor`. IDs autorados continuam funcionando para o catálogo inicial. Os outros comandos de entidade continuam usando IDs autorados; não foi acrescentada resolução de tokens implicitamente a todas as APIs.

Reset, edição bem-sucedida, undo/redo e load geram novo escopo e rejeitam tokens anteriores. Edição inválida não altera execução nem escopo. Ressurreição não troca token. Tokens de outra Engine, entidades sem ator e valores malformados são rejeitados antes de enfileirar ações. IDs autorados são aliases, não garantias de continuidade de um indivíduo entre resets.

Escopo de transporte usa início do processo, PID e nonce atômico monotônico. Não é credencial, UUID criptográfico ou identificador global persistente. Não integra estado determinístico nem replay; reproduzir cria outro escopo externo e recompõe os mesmos IDs lógicos. Slots/gerações internos não são argumentos públicos de comandos.

## Execução e replay

`LifeStamp` acrescenta referências da origem/destinatário em entradas e casts; entregas já conservam o cast. Fila/replay validam os stamps contra o registro, e o executor revalida antes do plano/custo/efeito. Cast obsoleto produz `instance_changed`, distinto de `life_changed`. Cancelamento também confere a origem antes de tocar a execução. A resolução numérica é indexada e não interpreta strings por pulso.

Contato novo de projétil e candidato novo de área recebem o stamp do destinatário corrente, preservando o stamp original da origem. Isso mantém a distinção entre uma intenção dirigida a alguém e um contato físico novo. O catálogo fixo ainda impede testar substituição real no mundo; esta etapa não conclui o contexto de autor removido.

`inspect_abilities.pending_inputs[].life` expõe os stamps junto da operação. Replay47 grava `expected_instances`; `run_replay` compara também `instances_matched`. Referências adulteradas em comandos gravados falham inclusive na inspeção de um prefixo; adulterar o registro esperado produz comparação final negativa. Nenhuma migração de replay antigo é prometida: o contrato existente exige build compatível. Scene permanece46 porque autoria não mudou.

## Continuidade necessária

Atualização EAF-61: [buffs, percepção, ruído e históricos](identity-bindings.md) receberam referências e validação; replay atual48, cena46. O texto acima descreve a entrega EAF-60/replay47. Ainda separar a visão do mundo presente e integrar bindings/componentes antes de liberar reciclagem real. Em seguida, integrar contexto de origem e spawn/despawn transacionais; só depois geradores de população. Atributos indexados por dono/chave, transações com apenas campos tocados e reserva da retenção de origem na admissão das entregas são requisitos aprovados para essa evolução. Não foram implementados neste incremento.

Validação: `cargo test --locked --release --all-targets`, Clippy release/all-targets e `python3 scripts/run_verifications.py` (inclui `identity`). Testes cobrem reuso/overflow isolados, referências externas, erros atômicos, identidade versus vida, casts adulterados, replay pendente/completo/prefixo e igualdade de captura GPU offscreen. Não há nova interação visual para demonstrar nesta etapa; comportamento da demo de ressurreição permanece o anterior.
