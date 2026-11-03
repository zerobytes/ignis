# Canalização sustentada (EAF-47)

Uma habilidade pode ter preparação, seguida de um canal que mantém o ator ocupado e entrega pulsos enquanto permanece válido. Preparar não entrega efeitos; canalizar exige manter a execução; um efeito periódico já entregue tem vida independente da origem. O mesmo executor atende jogador, NPC e MCP.

## Contrato e alternativas

Cena/replay36 acrescenta `channel` opcional à definição. Ausência preserva o comportamento antigo. Fragmento de configuração:

```json
{"channel":{"interval_ticks":30,"pulses":6,"cost_mode":"per_pulse"}}
```

Intervalo/pulsos positivos de1..3600; produto até216000 ticks. `cost_mode` obrigatório: `on_start` ou `per_pulse`. `cast_ticks` continua sendo a preparação anterior ao canal. Ao terminar a preparação no tick t, o primeiro pulso vence em t+intervalo; o último em t+intervalo×pulsos. Não há pulso imediato nem efeito extra ao terminar.

Escolha: progresso tipado dentro da execução já existente, com a mesma lista de efeitos repetida. Alternativas seriam simular uma agenda independente ligada ao emissor ou criar um scheduler de callbacks. Ambas exigiriam outra fonte de cancelamento/ocupação/custo. Manter a execução simplifica cancelamento e movimento, ao custo de aumentar o tamanho fixo do Cast também usado por agendas/projéteis.

Neste recorte canal exige entrega direta e seleção self_actor/actor, sem `periodic` independente. Dano, cura, recurso, ruído e dispel continuam compondo a lista transacional. Categoria/proteção de agendas não tornam o canal dissipável: canal termina por cancelamento/falha, não pelo operador de dissipar. Sem rajadas de projéteis, canal por direção, área, carga/liberação ou duração infinita.

## Cobrança e resultados

Antes da preparação e novamente ao entrar no canal, valida-se o primeiro efeito e a capacidade de pagar um custo completo. Essa checagem não reserva recursos. O cooldown começa na entrada do canal nas duas políticas. Mesmo que expire antes do último pulso, o ator continua ocupado.

- **on_start:** cobra a lista de custos uma vez na entrada, antes de qualquer pulso. Falha/cancelamento posterior não reembolsa. Os pulsos não precisam de saldo para outra ativação.
- **per_pulse:** entrada não debita; cada pulso revalida e cobra seus custos junto dos efeitos. Se faltar energia ou falhar qualquer operador, esse pulso inteiro não acontece e o canal termina. Efeitos de um pulso não financiam seus próprios custos. Pagamentos e efeitos anteriores permanecem.

Cancelar durante preparação, inclusive no tick em que ela venceria, não cobra nem inicia cooldown. Cancelar depois da entrada mantém o cooldown; per_pulse antes do primeiro pulso não paga energia, mas mantém esse prazo. Um novo cast não pode substituir outro sem cancelamento explícito e avanço do tick.

Recibos: `started` marca preparação; `channel_started` registra entrada; `channel_pulse` registra cada transação; `completed` registra término normal sem duplicar efeito/custo. Falhas usam os motivos existentes, como `insufficient_resource`, `out_of_range`, `target_unavailable`, `no_matching_effect` ou `noise_budget`. Inspeção de custos distingue entrada paga, pulsos pagos e término sem débito. Anel limitado256.

## Interrupção, percepção e ordem

Durante canal ativo, cada tick revalida fonte/destinatário vivos, fonte habilitada, visão atual de NPC, relação, alcance XYZ, oclusão e condição de recurso próprio. Objetivo/política NPC continuam podendo cancelar por troca de prioridade, alvo, território ou limiar vital. Alvo congelado por identidade não autoriza rastreamento através de paredes. Estado do alvo só é consultado depois de confirmar conhecimento visual, salvo controle explicitamente onisciente existente.

Para canal já ativo, saldo insuficiente não elimina a intenção antes do instante de cobrança. Isso evita que um NPC cancele o canal pré-pago por não ter dinheiro para uma segunda ativação; per_pulse encerra no próximo pulso impagável. Não acrescenta triagem de aliados ou tática.

`movement` vale na preparação e no canal: free permite mover; interrupt cancela com entrada/movimento físico acima da tolerância; lock impede locomoção comandada, mantendo gravidade e colisão. X/cancel_ability interrompe ambos pelo ID de execução. Cancelar ID antigo não interrompe a próxima execução.

Ordem por passo preservada: cancelamentos explícitos; pulsos periódicos independentes; impactos de projéteis; interrupção por movimento; execuções existentes por ID de ator (término de preparação ou pulso de canal); novas ordens; limpeza final de agendas de alvos mortos. Pulso de canal com morte cancela as execuções pertinentes e não restaura um cast morto. Sensores/posições são os snapshots existentes pré-física; interrupção física também é checada após movimento. Não promete detecção sub-tick.

## Inspeção, apresentação e persistência

`set_ability_catalog` e `edit_scene` autoram channel pelo schema compartilhado. `activate_ability` e `cancel_ability` existentes controlam a execução. `inspect_abilities` mostra channel nos grants e, no cast, `phase: preparing|channeling`, progresso `{started,next_tick,finishes_at,remaining}`, alvo, custos e recibos. `execute_at` continua sendo o prazo de preparação, não o próximo pulso.

A barra de preparação é amarela crescente. Durante canal é ciano decrescente, mostrando tempo restante total. Usa o mesmo J/set_debug_view(npc_routes) e cache visual; não é um feixe visual nem fornece informação à decisão do NPC. Replay36 reconstrói estado, recibos, entradas pendentes, prefixos e pixels. Edição/reset/load/undo recompilam e reiniciam como antes; cenas antigas permanecem aceitas, replay exige build/versão compatíveis.

## Demo

```sh
./target/release/ignis --scene examples/npc-channel.scene.json --window
```

P inicia/pausa; WASD move; clique seleciona; J mostra barras; X cancela; R restaura.

- **1:** dano canalizado no selecionado,6 pulsos de5 a cada0,5s, custo5 por pulso. Permite andar; perder alcance/linha de visão interrompe.
- **2:** cura própria,6 pulsos de5 a cada0,5s, custo15 ao iniciar. Mover cancela. Jogador começa com70 de vida.
- **3:** cura canalizada no selecionado,6 pulsos de5 a cada0,5s, custo5 por pulso. Bloqueia locomoção durante preparação/canal; X libera.
- **4:** veneno periódico independente,4 pulsos de3 a cada1s, custo5. Depois de entregue continua ao cancelar/mover, permitindo comparar com canalização.
- **Roxo:** alvo passivo para dano/cura/veneno.
- **Azul:** começa com50 de vida, canaliza cura própria em6 pulsos de5, paga10 ao iniciar; usa a mesma política/executor e repete após cooldown enquanto ferido.
- **Vermelho:** aproxima para alcance8m e canaliza dano no jogador observado,4 pulsos de3 a cada0,75s. Cobertura/perda de visão interrompe; sem feixe gráfico nesta etapa.

Todas as preparações duram0,5s. Cooldowns do jogador1s e NPCs4s a partir da entrada no canal. O botão4 aplica um efeito independente; X não o dissipa. Aceite visual e teclado nativo são separados das verificações MCP/offscreen.

Validação: `scripts/run_verifications.py --only channel`, suíte completa headless, testes Rust/all-targets e Clippy. Resultados em validation.md; checklist estático em performance.md.
