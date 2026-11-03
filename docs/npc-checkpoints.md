# Checkpoint local do mundo — NPC v1

Decisão antes do código. Replay verifica uma história finita; não é save game de
execução contínua. O checkpoint precisa salvar indivíduos/gerações/vidas, poses e
movimento físico, memórias, rotinas/agenda, ordens, sessões internas de diálogo,
recursos/cooldowns, entregas/origens/buffs e relógios absolutos do mesmo tick.
Não reexecutar a história do início para carregar nem pagar efeitos novamente.

Alternativas: reconstruir apenas transforms/velocidades desacopla o save do backend,
mas perde contatos/warm start/trechos em voo e exige um novo contrato de retomada
física. Primeiro recorte usa snapshot nativo Rapier com formato/versão/plataforma
explicitamente fechados, mais estado de gameplay próprio. Não é um formato estável
para migrar entre versões do Rapier. Migração futura deve importar primárias e
reconstruir física com regras próprias; não aceitar silenciosamente outro layout.
A decisão troca portabilidade inicial por continuidade física verificável agora.

Arquivo binário CBOR local, envelope com versão/comprimento/integridade, limite de
bytes/profundidade e escrita temporária+fsync+replace. O arquivo não é protocolo de
rede nem mídia arbitrária de terceiros. Parse/checagens/construção em candidato
separado antes de substituir mundo. Erro preserva mundo e save anterior.

Caches de lookup/broadphase/planos compilados recompõem-se de definições de execução
e perfil de navegação congelado; identidade e relógios não são regenerados. Caches
Rapier de contatos necessários à continuidade permanecem no snapshot físico. Não
serializar GPU/renderer, sockets/TLS, threads, locks, ponteiros, UI ou ownership externo.

Load renova escopos externos de controle/diálogo/world. Preservar o progresso interno
da conversa e permitir consultar um token novo; token antigo não governa mundo novo.
Entradas físicas de jogador/posse externa neutralizadas, sem repetir tecla segurada.
Host/cliente ativos exigem encerramento explícito antes de load; nenhum cliente
silenciosamente controla a pessoa restaurada. Single player não exige conta/rede.

Checkpoint é baseline de replay seguinte. Depois do load, replay guarda a baseline
e só os eventos posteriores, com ticks absolutos e prefixos relativos claros. Reset
retorna à autoria da cena; nova edição também segue fronteira autoral existente.
Execução continuous continua sem histórico, mas permite salvar/carregar checkpoint.

## Checklist estático prévio

1. Captura fora do tick no ponto de comando; uma cópia limitada de primárias, não log infinito.
2. Formato binário tipado, sem JSON em simulação; persistência não altera ABI dos componentes.
3. Ticks/serial/geração sem wrap; dimensões/referências/índices e versão validados.
4. Reusar serde/State/pools, reconstruir catálogos/caches com identidade restaurada.
5. Orçamento explícito do arquivo e alocação; parse recursivo limitado e candidato isolado.
6. Contatos/voo não teleportam ao retomar, mesmo após objeto aberto e origem retirada.
7. Sem rede obrigatória; handles antigos e reconexão não sobrevivem implicitamente ao load.
8. Escrita atômica e rollback de load; formato incompatível/corrompido não publica nada.
9. Uma prova crítica salva/carrega ativo, reusa slots, retoma e faz replay pós-baseline.
10. Registrar custo de cópia/off-tick, compatibilidade inicial e ausência de migração universal.

Fontes de contrato: [snapshot oficial Rapier](https://github.com/dimforge/rapier/blob/master/website/docs-examples/2d/rust/examples/rs_serialization.rs),
[CBOR/Serde](https://docs.rs/ciborium/0.2.2/ciborium/),
[limite de recursão](https://docs.rs/ciborium/0.2.2/ciborium/de/index.html).


## Implementação: Scene 97 / replay 107 / checkpoint 1

Comandos no dispatcher compartilhado/MCP:

- `save_checkpoint {"path":"artifacts/world.ignis-save"}`: captura o mundo pausado, sem pedidos de gameplay pendentes. Casts, efeitos, diálogo e voo em andamento são permitidos; um pedido aceito ainda não processado exige `step` antes de salvar.
- `load_checkpoint {"path":"artifacts/world.ignis-save","mode":"recorded"}`: valida/constrói um candidato e publica pausado. `mode` opcional conserva o modo salvo; pode trocar continuous por recorded para começar uma nova história verificável.
- `inspect_checkpoint {}`: versão, limite, física/plataforma, tick absoluto, tick da baseline, bytes da baseline e modo. Somente leitura.

Service restringe o caminho ao projeto e extensão `.ignis-save`, usa escrita atômica existente e só limpa seleção após load bem-sucedido. Preferências de debug permanecem. Host ou cliente ativos recusam load; cliente remoto também recusa save e captura PCM local, pois a projeção pública não contém o estado autoritativo nem o banco sonoro. Encerrar host/desconectar é uma ação explícita anterior; load não encerra conexão por conta própria.

Envelope: `IGNISCPT` (8 bytes), versão u16, tamanho u64, SHA-256 (32 bytes), payload CBOR tipado. Cabeçalho de 50 bytes, total até 128 MiB, profundidade de decode 128 e EOF exato. O hash detecta corrupção; não autentica um autor. Não se oferece importação de arquivos hostis arbitrários. Antes da publicação são conferidos versão/plataforma, dimensões, registry/catálogo, referências/vidas, definições/templates, recursos, índices de efeitos, sessões, props, corpos/handles e estado correlato.

Rapier fica fixado em `=0.30.1`, com `serde-serialize`; CBOR em `=0.2.2`. A compatibilidade inicial exige Scene 97, checkpoint 1, a mesma física e OS/arquitetura. Build ID é registrado para auditoria; não é promessa de migração entre qualquer build. Não há migração universal de saves nem portabilidade entre versões de física/plataformas. Protocolo remoto 3 e conteúdo 1 permanecem.

O estado inclui registry/gerações, catálogo/mundo em execução, bases de atributos, recursos/casts/entregas/origens/buffs, percepção/memória/objetivos, cooperação/leases, rotinas/agenda/diálogo/props, geradores/população, controladores/voo, apresentação/vozes, passos, triggers e contatos nativos. Caches derivados recompõem-se com as definições de execução e a fonte física congelada, inclusive se o autor daquela fonte foi retirado e o slot reutilizado. Nenhum replay anterior é executado para carregar.

Novos escopos invalidam tokens externos de controle e diálogo. A conversa interna continua e `inspect_dialogue` emite um token novo. Teclas mantidas pelo jogador são neutralizadas; ordens internas do NPC e seus prazos continuam. Um indivíduo novo não recebe o estado do anterior. Captura/restauração em continuous não cria histórico de entrada.

## Baseline de replay

Replay 107 incorpora o checkpoint binário em base64 e grava somente entradas posteriores. Contatos e sentinelas numéricas do Rapier não são convertidos em objetos JSON. Não há baselines aninhadas. Ticks de simulação, deadlines e eventos continuam absolutos; seleção de prefixo usa 0..quantidade de novas entradas. Prefixo 0 é o instante restaurado. IDs de admissões/ruídos continuam dos seus watermarks e conservam o domínio do ID, inclusive após serial manual 2^30.

O limite existente de **8 MiB do documento de replay** continua aplicável. Um checkpoint de até 128 MiB não implica que um replay com essa baseline caberá nesse limite; a exportação pode recusar antes de escrever. Checkpoint e replay têm propósitos e orçamentos diferentes. Reset/edição retorna ao documento autoral e descarta a baseline.

## Revisão estática após implementação

1. Serialização só sob comando pausado, fora do tick; não há CBOR/hash/fsync na decisão do NPC.
2. Captura clona primárias uma vez; buffer de encode limitado, não há crescimento ilimitado de história. Manual noise mantém só anel recente de 128.
3. Parser limita bytes/profundidade e exige payload consumido; candidato separado protege atomicidade. Buffer original, estado desserializado e candidato coexistem: pico de memória de load é custo explícito, não zero-copy.
4. IDs, gerações, vidas e sequências preservados; buffers/cache recompostos sem gerar uma nova população.
5. Estado físico nativo evita reinstanciar apenas transforms e perder contatos. Custo e dependência do backend ficam explícitos.
6. Caches de geração recompostos ignoram o campo autoral derivado `cached`; igualdade de autoria/replay também ignora esse cache.
7. Sem novos workers, locks, GPU, dispositivo de áudio, conta ou servidor obrigatório. Ring já usado pelo TLS fornece SHA-256.
8. Arquivo/candidato falho não substitui mundo nem save anterior; tokens antigos não adquirem posse no mundo novo.
9. Uma prova Rust crítica ampliada protege perdas/duplicação/identidade/formato/replay; cobertura existente selecionada por impacto. Teste antigo de retirada em massa passou a enviar lotes de 64, preservando a admissão real.
10. Custos conscientemente mantidos: snapshot físico nativo, cópia off-tick e recompilação de navegação/atores ao carregar. Não se promete load sem pausa, migração universal nem ganho de FPS.

## Evidência final

Build `c316e5eeb5acde56`, macOS arm64. **79 testes Rust distintos em 12 alvos**, **26 cenários MCP / 92 grupos**, check e Clippy release all-targets `-D warnings`, fmt/Python/diff aprovados. A prova crítica final e o cenário integrado foram repetidos após estender a cura a 90 ticks, garantindo uma lease ativa ao salvar no tick 60.

A prova Rust inclui DOT/buff/cura/voo ativos, retirada/reuso e origem antiga ainda válida, tokens expirados, novo diálogo sem custo duplicado, replay/prefixos, checksum e referência de catálogo forjada recusados atomicamente, domínio de ruído após 2^30, continuous sem histórico e roundtrip de 100 NPCs + 500 jogadores. O cenário MCP compara estado ao carregar e pixels do tick salvo com o prefixo 0; também recusa load com host ativo sem alterar o mundo pausado.

Receita: `python3 scripts/run_verifications.py --only npc_checkpoint`. Logs em `artifacts/npc-checkpoint-*.log`, relatório amplo preservado em `artifacts/npc-checkpoint-suite-report.json`, prova atual em `artifacts/mcp-validation/artifacts/npc-checkpoint-verification.json`. Apenas um executável corrente em `target/release/ignis`; nenhuma janela foi aberta para validar. Aceite humano da demo é separado.
