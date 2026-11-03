# Controle de geradores em execução — EAF-95

Continuidade EAF-96: [ativação espacial](generator-activation.md), cena63/replay67. `paused_since` agora marca suspensão efetiva por permissão manual ou condição espacial; sem activation o comportamento abaixo permanece.

Cena62/replay66. `initially_active` é autoria, booleano com padrão true. False inicia o gerador pausado no tick0; a primeira ativação conserva o atraso inicial inteiro. Cenas antigas sem o campo continuam ativas. Não migra automaticamente campos gráficos de outras branches.

## Contrato MCP e ordem

`control_npc_generator {generator:"encounter",action:"activate"|"pause"|"cancel"}` entra na fila existente de população. Resposta informa accepted/request_id/tick/phase pending. Efeito no próximo step; pausar o mundo com P ou pause deixa os pedidos pendentes. `inspect_population` mostra fila e recibos finais; `inspect_generators` mostra controles, estado, membros, ondas e prazos.

ID é resolvido para índice na admissão. ID desconhecido, ação inválida e limite excedido falham antes de mutar. A fila inteira tem64 pedidos/tick,4096 por execução e recibos em anel256, compartilhados com spawn/despawn. Repetições aceitas contam no orçamento. Aplicação sequencial antes da fase de geradores, sensores, habilidades e física. Nenhum nascimento automático ocorre entre dois comandos dessa fila: pause→activate no mesmo tick termina ativo; activate→cancel termina cancelado sem um nascimento intermediário. Operações manuais de spawn/despawn continuam independentes.

- `activate`: ativa ou retoma. Conserva seed, contadores, membros, identidade e onda. Acrescenta a duração da pausa aos prazos de próxima tentativa e de atraso da onda, sem recriar configuração ou consultar relógio de parede.
- `pause`: congela nascimentos e avanço das ondas. Marca o tick da suspensão; comandos repetidos não mudam esse início. Um prazo que já venceu continua elegível imediatamente na retomada, sem acumular nascimentos atrasados.
- `cancel`: termina a produção nesta execução. Não mata nem retira membros. Activate/pause posteriores recebem `control_terminal`; cancel repetido recebe `control_unchanged`.

Recibos: `control_applied`, `control_unchanged` ou `control_terminal`. A admissão só confirma estrutura/identificador; estado é avaliado na ordem de aplicação, permitindo sequências no mesmo tick. Uma sequência de ondas concluída ou um gerador contínuo com orçamento esgotado também retorna terminal, inclusive para cancel; término histórico preservado. Reinício exige reset da execução ou edição explícita da autoria, não activate.

## O mundo continua

Pausa/cancelamento pertencem ao produtor, não aos indivíduos. NPCs seguem sensores, locomoção, combate, efeitos, morte e ressurreição. O gerador segue limpando referências ausentes e observando life_index/morte; sua retirada opcional de cadáveres usa o tempo normal da simulação, inclusive durante pausa/cancelamento e após conclusão. Cadáver preservado pode ser ressuscitado; pause não congela seu prazo de retirada. Para preservação indefinida, autoria `retire_dead_after_ticks:null`.

A condição de conclusão de onda não avança pausada. Se todos morrerem/forem retirados, ela confirma no primeiro tick ativo; o intervalo da próxima onda começa nessa confirmação. Se alguém ressuscitar antes da retomada, a condição usa o mundo atual. Cancelamento não gera evento de onda completed nem simula derrota: estado da onda permanece histórico e o controle cancelled explica por que não progride.

## Inspeção e persistência

`controls` tem uma entrada por definição: id, status active/paused/cancelled, terminal null/cancelled/completed/budget_exhausted, paused_since/cancelled_tick, remaining_delay_ticks e remaining_cadence_ticks. Pausado, restante é calculado contra paused_since, não contra tick atual. `due_tick` e `next_attempt_tick` brutos ficam na coordenada anterior até activate deslocá-los; não interpretá-los como previsão durante pausa. Cancelado/concluído não há promessa de próxima tentativa. `waves.waiting_for` continua descrevendo a fase interna, subordinada ao estado de controle.

Save de cena guarda initially_active, nunca estado de controle em execução. Replay66 guarda pedidos, inclusive pendentes, estado e recibos esperados; reproduz a ordem, prefixos e pixels. Reset/edição/undo/load recompõem o estado autorado e esvaziam fila/histórico de execução. Não há restart parcial, exclusão dinâmica de definição ou referência de gerador sobrevivendo a reset como objeto autônomo.

## Interface e tradeoffs

Em cenas com exatamente um gerador, B ativa/retoma, O pausa, K cancela pelo mesmo Command do MCP. P segue play/pause do mundo; R reset. O título mostra ID/status; com múltiplos geradores não se escolhe um arbitrariamente, use o ID no MCP até haver seletor visual dedicado. Controles de teclado/foco requerem teste humano separado.

Escolhida fila de população compartilhada para ter ordem única e orçamento comum, em vez de mais uma fila com prioridade implícita. Escolhido deslocar prazos na retomada em vez de manter relógio por gerador ou atualizar deadlines a cada tick: estado adicional pequeno e trabalho apenas no comando. Cadáveres mantêm a semântica de tempo do mundo; congelar indivíduos seria outro controle. Registro e enum tipados no runtime, sem JSON/strings/alocação nova por tick. Pool8 NPCs, geradores8, ondas16 e buffers existentes mantidos. Sem ganhos de FPS ou escala alegados.

Demo `examples/npc-generator-control.scene.json`, criada por MCP: mesma sequência roxo/laranja/azul, inicialmente pausada. P inicia o mundo e B libera a primeira onda; O impede progresso enquanto você pode continuar movendo/atacando. B retoma, K cancela novos nascimentos.1 causa100dano,2 ressuscita enquanto cadáver presente. Azul200vida; dois segundos entre ondas. R restaura autoria pausada. Testes em `tests/generator_control.rs` e `scripts/verify_generator_control.py`; evidências finais em validation.md.
