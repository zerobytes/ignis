# Controles independentes por personagem — EAF-122

Primeiro incremento executável após [estudo MMO](multiplayer-system-study.md),
autorizado em29/10/2026. Rust; meta futura de1.000 jogadores na mesma área e no
mesmo combate preservada. Base6bfbc5c, Wedge75/replay79. Este recorte remove input
e motor de jogador únicos, sem declarar rede, autenticação remota ou escala pronta.

## Decisões anteriores à implementação

Entrada global transmitida a todos os corpos confundiria controle com população.
Nome de ator em cada input resolveria o destino, mas permitiria que uma liberação
antiga neutralizasse um controle novo. Escolha: vínculo exclusivo explícito por
instância, handle local escopado à execução e serial monotônico não reutilizado.
O MCP é administrativo/confiável; o handle não substitui autenticação de rede.

`bind_control {actor}` aceita alias atual ou token de inspect_instances, somente
Player. Retorna `control`. `set_control_input {control,input}` mantém input por
personagem. `release_control {control}` neutraliza e libera somente esse vínculo.
`inspect_controls` expõe jogadores, vínculos, inputs e limites. Um personagem
admite um vínculo; reset, edição/undo/redo e load invalidam handles. Soltar e
vincular novamente produz outro serial. Não há lease temporal, conta ou sessão
remota neste recorte. Comandos inválidos não alteram log, input, revisão ou mundo.

`set_input` preserva o caminho legado com até um Player sem vínculo explícito;
com vários jogadores ou com esse Player vinculado, rejeita antes de mutar.
Bind/release invalidam a posse nativa anterior e limpam o input legado. Não há
broadcast, escolha silenciosa do primeiro jogador ou transferência visual nova.

Movimento, perfis de locomoção, ruído e amostras de cast são por ator. Morte mantém
vínculo, neutraliza input explícito; retorno não restaura direção antiga. Bind e
input em cadáver são recusados. Liberação em cadáver continua possível. Cada
passo usa buffers de input reutilizados, sem JSON ou clone de Scene no tick.

O storage de atores ainda é o do laboratório:9 slots globais, no máximo8 NPCs.
Este é limite transitório validado antes de criar o catálogo, não capacidade
arquitetural MMO. Templates reservam os slots de NPC que couberem no mesmo
orçamento, até8, preservando8 quando há um jogador. A migração de índices,
atributos e transações locais é unidade seguinte, sem simplesmente ampliar arrays.

Cena77 admite múltiplos Player; versões anteriores continuam recusando essa
capacidade. Replay81 acrescenta eventos de controle por tick/sequência/identidade
e estado esperado dos vínculos. Registrar mudanças evita copiar input de toda a
população em cada tick. O limite de duração/gravação atual permanece explícito.
Prefixos incluem eventos de controle admitidos no tick selecionado; input passa
a afetar física apenas no próximo step. Capturas continuam usando o mundo físico.
Não há alteração de RenderFrame, renderer ou formatos gráficos; ambiente76/80 é
frente independente e não entra implicitamente pela numeração maior.

## Checklist antes de codificar

1. Vínculo/input/liberação sob comando; frame de inputs preparado uma vez por tick.
2. Frame Vec reservado por capacidade atual; vínculos em BTreeMap por handle,
   alocados só em bind. Sem String/JSON/cópia de mundo no tick.
3. Referência geracional do ator, serialu64 e escopo externo; índices internos
   resolvidos pelo catálogo. Não criar ID de jogador público baseado emu8.
4. Acesso direto ao frame por slot; O(P) para preparação/lifecycle, sem busca P².
5. Validar atores+reserva de templates antes de mutar; log4096,64 eventos/tick.
   Bind/input reservam um release por vínculo ativo nos dois budgets, para que
   saturação não impeça neutralizar/liberar controles já admitidos.
6. GPU/I/O somente nos fluxos existentes de captura/persistência; sem rede nova.
7. Mesma fase/solver/lock; controle de transporte futuro não expõe todo dispatcher.
8. Invalidar em reset/edição/load/release; neutralizar na transição de vida/morte.
9. Testar isolamento/movimento/parede/locks/interrupção/passos/morte/retorno,
   exclusividade/handles/erros/limites e replay/prefixos/PNG, regressão completa.
10. Mantidos índices/máscaras e custo de Plan globais; explicitamente pendentes
    para escala. Nenhuma promessa de FPS, rede ou teclado nativo.

## Validação

O build corrente desta worktree usa o caminho externo estável
`/Volumes/Zero/dev/games/.ignis-build/multiplayer-runtime`; `target` é somente um
symlink gerado/ignorado para esse mesmo diretório, sem cópia de executável.

Build `9f0587fa3f842121`, macOS Apple Silicon/Metal, cena77/replay81:

- 760 testes Rust release/all-targets em62 alvos, zero falhas/ignorados;13 testes
 novos de controles e1 teste de posse nativa em handler sem janela.
- Clippy release/all-targets `-D warnings`, fmt e build release aprovados.
- 67 cenários MCP/277 grupos completos headless passaram; o cenário actor_controls
 tem5 grupos, zero pedidos lentos e capturas de execução/replay byte-exatas.
 A imagem real foi inspecionada: ambos os corpos separados, azul e laranja,
 presentes no mesmo piso; a captura estática não é prova de movimento/teclado.
- Replay passa pela mesma admissão de controles do vivo, incluindo reservas de
 release. Erros de sequência/tick/vida, geração/serial esgotado e estado esperado
 adulterado são cobertos. O frame conserva o ponteiro entre inputs.

Um primeiro teste de regressão identificou inspeção de Player ausente por seam
cfg(test); corrigida para consultar referência presente antes de formar a linha,
preservando o teste de ausência em world_origins. Não se relaxaram assertivas.
As versões correntes nas verificações legadas foram atualizadas; guardas históricas
de capacidades, geometria e pixels permaneceram. Aviso transitivo preexistente
de futura incompatibilidade de block0.1.6 não é falha do Clippy.

Evidências em `artifacts/controls-{rust-tests,clippy,build,fmt,mcp}.log`,
`artifacts/mcp-validation/report.json` e
`artifacts/mcp-validation/artifacts/actor-controls-verification.json`/PNGs/replay.
O source hash do executável foi conferido contra src/Cargo.lock/build.rs.
Um release corrente11.401.728bytes, cache962MiB e artifacts44MiB no checkpoint;
sem instalador/cópia histórica ou interação com a sessão visual.
Aceite humano, foco/teclado físico, integração ao editor, rede e capacidade1000
continuam gates próprios. Testes pequenos comprovam contratos; não substituem
o alvo de1.000 no mesmo combate.

## Reproduzir o recorte

Cena `examples/actor-controls.scene.json` criada/salva pelo MCP real, copiada sem
alteração da saída nativa e carregada pelo CLI final: dois Player, zero vínculos
persistidos. Laranja=`player`, azul=`partner`. Carregar pelo MCP e executar:

1. `bind_control {actor:"player"}` e `bind_control {actor:"partner"}`; guardar os
   dois handles retornados.
2. `set_control_input {control:handleLaranja,input:{x:1,z:0,mode:"run"}}` e
   `set_control_input {control:handleAzul,input:{x:1,z:0,mode:"sneak"}}`.
3. `step {ticks:60}`; `inspect_controls` mostra vínculos/perfis e `inspect` mostra
   posições/passos. O campo player_motion legado fica null com vários jogadores.
4. `release_control {control:handleLaranja}` neutraliza só o laranja. Uma nova
   vinculação retorna handle diferente; o anterior não pode liberar o novo.

Habilidades por ator usam os comandos administrativos existentes: slot1 mantém
cast lock12ticks, slot2 causa100 de dano, slot3 ressuscita com50. O cenário MCP
automatiza essas verificações. Vínculo não é conta/sessão remota e não restringe
outros comandos administrativos do MCP. Janela e UI de escolha de jogador não
foram acrescentadas neste incremento.

## Revisão final de performance

| Pergunta | Resposta/evidência |
|---|---|
|1. Frequência|Adequado: bind/input/release sob comando; inputs e lifecycle uma vez por tick em controls.rs/Engine::advance.|
|2. Alocações|Adequado: frame Vec reservado e reutilizado; mapa só aloca em bind e log cresce sob comando. Nenhum clone de Scene ou JSON no tick vivo.|
|3. Tipos|Adequado: serialu64/escopo, referência geracional e vida. Catálogo resolve referência→slot diretamente; storage u8/máscaras atuais explicitamente transitórios.|
|4. Repetição|Corrigido: lookup por handle em BTreeMap O(log P), frame por slot O(1), preparação/lifecycle O(P). Admissão compartilhada vivo/replay sem JSON intermediário.|
|5. Limites|Adequado: atores totais9, até8NPCs; reserva de templates reduzida por Player antes do catálogo. Log4096/tick64 reserva releases, inclusive saturação total testada.|
|6. GPU/I/O|Adequado: nenhum renderer/RenderFrame/Vertex/shader novo; PNGs puros e byte-exatos. JSON, save e readback sob demanda.|
|7. Sincronização|Adequado: mesmos locks/fases; inputs individuais alimentam cast/motor/passos sem mudar ordem de NPCs/efeitos. Sem thread/dependência nova.|
|8. Invalidação|Corrigido: reset/edit/undo/load refazem Runtime/escopo. Release remove vínculo; morte/retorno neutralizam input explícito. Inspeção filtra Player ausente.|
|9. Contrato|Validado:760Rust/67MCP/277grupos, Clippy/fmt/build, replay/prefixos/erros/capacidade/colisão/PNG. Gate nativo físico/humano separado.|
|10. Custos|Mantidos com motivo: pequenos scans na inspeção/exclusividade sob comando, mapa aloca por bind, log/duração e Plan globais do laboratório. Migração para escala segue pendente; nenhum ganho de FPS alegado.|
