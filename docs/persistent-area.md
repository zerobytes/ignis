# Áreas persistentes 3D — EAF-53

## Contrato

`Definition.area.persistence` opcional (cena/replay42). Ausência mantém a área instantânea. Nenhum elemento, classe ou temática é embutido: reutiliza dano composto, cura, recursos, buffs, dissipação e interrupção já definidos pelo jogo. `periodic` e `channel` continuam incompatíveis com `area`; o scheduler pertence ao volume, não a cada destinatário.

| Campo obrigatório | Semântica |
|---|---|
| `duration_ticks` | 1..216000; vida do volume desde a entrega até o prazo exclusivo |
| `interval_ticks` | 1..3600; espaçamento entre pulsos |
| `apply_on_spawn` | Aplica um pulso no momento de criação; os seguintes mantêm o intervalo |
| `reapply` | `refresh` substitui área do mesmo autor/definição; `independent` cria outra |
| `end_on_source_death` | Termina o volume quando o autor morre; false permite persistência póstuma |

Tempo de simulação em passos de1/60s. O intervalo de vida é `[created_tick, expires_at)`: expiração precede pulso no mesmo tick. Sem pulso inicial, deve haver pelo menos um pulso antes da expiração (`interval_ticks < duration_ticks`). Com pulso inicial, duração menor que intervalo é válida. Exemplo duração360/intervalo60/inicialtrue aplica em0,60,120,180,240,300 e expira em360.

Entrega direta resolve o centro no commit (posição do autor/alvo ou ponto espacial congelado). Projétil mantém deslocamento e cria o volume somente no contato; duração começa nesse instante. Expirar sem contato não cria volume. No modo padrão `anchor:fixed`, após a entrega o centro permanece **fixo em XYZ**, mesmo que o autor ou a âncora se mova/morra. A extensão [auras pulsantes](auras.md) acrescenta `anchor:source` em cena43 para entrega direta self_actor.

## Ocupação, efeitos e ordem

Cada pulso refaz a seleção de centros físicos atuais, inclusive raio XYZ, vivos, relação, inclusão da origem, oclusão e limite. Usa o [contrato de área instantânea](instant-area.md): lista congelada para aquele pulso, distância²/ID e transação independente por destinatário. Uma falha não termina o volume nem impede as próximas tentativas; a vaga falha não é substituída naquele pulso.

Entrar não concede pulso adicional. Sair impede os próximos pulsos, sem desfazer dano já recebido. Atravessar entre dois pulsos não recebe efeito. Buff instalado por um pulso mantém seu próprio prazo mesmo após sair; não há aplicação/remoção automática por ocupação contínua. Isso é uma escolha de amostragem explícita, não detecção contínua de entrada/saída.

Custos e cooldown uma vez na entrega direta/lançamento. Pulsos avaliam atributos vigentes por transação (dano causado/recebido, proteção etc.), sem rastrear percepção da origem ou cobrar novamente. Morte/dissipação dentro de um pulso pode mudar contexto dos destinatários seguintes, mas o pulso já comprometido termina. `end_on_source_death:true` remove depois esse volume e suprime pulsos ainda não iniciados. Buffs da origem morta desaparecem; bases permanecem, conforme contrato de dano causado.

Ordem na fase de habilidades: limpar buffs, coletar intenções e cancelar casts; efeitos periódicos existentes; expiração/morte e pulsos de áreas; impactos de projéteis; casts/canais e novas solicitações; limpeza final de mortes. Pulsos consultam posições anteriores à física desse passo, como as outras habilidades. Áreas na ordem estável do pool (refresh preserva posição). Expiração tem precedência sobre motivo de morte quando ambos coincidem. Sem interrupções assíncronas, relógio real ou callback recursivo.

## Renovação, capacidade e morte

Refresh usa **autor + definição**, não slot, alvo, centro ou distância aproximada. Substitui execução, centro e relógios; recibo liga execution_id anterior ao novo. A última aplicação pode retirar uma zona de um lugar para colocá-la em outro. Independent mantém todas e seus próprios prazos; sobreposições aplicam separadamente. Autores diferentes nunca renovam a área um do outro.

Até16 volumes. Validação direta verifica capacidade no pedido e novamente no commit, antes de pagar. Áreas expiradas ou encerradas por morte liberam vagas no mesmo tick. Refresh válido funciona com pool cheio. Projéteis não reservam vaga durante voo: impacto pode retornar `area_budget`, `effect_failed` no recibo de projétil, sem reembolso. Se autor morreu e a configuração exige sua vida, impacto é recusado com `actor_dead`; false permite instalar. Essa escolha evita reserva oculta mantida por disparos que nunca chegam.

Cancelar/interromper antes do commit impede criação e custo. Após a entrega, `cancel_ability` só cancela casts ativos; não recolhe zonas, assim como não recolhe projéteis. Não há dissipação manual do volume neste recorte; duração, morte configurada ou reset/edição encerram. Efeitos `dispel` continuam dirigidos a efeitos pessoais, não removem áreas do mundo.

## MCP, replay e guias

Autoria via `set_ability_catalog`/`edit_scene`; mesmos `activate_ability`, `aim_ability` e `aim_ability_at`. Proceduralidade usa o mesmo dado validado, sem IDs especiais de zona na autoria. Inspeção `inspect_abilities.persistent_areas`: ativos com definição/autor/slot/execução, centro/configuração, criação/expiração/próximo tick e quantidade de pulsos; ring128 de ciclo de vida com `created`, `refreshed`, `expired`, `source_dead`, vínculo de substituição e descarte explícito.

`inspect_abilities.areas.receipts` mantém resultados dos últimos64 pulsos/áreas, com `pulse` numerado desde1 (null para instantânea), execução e candidatos/motivos. Recibos individuais `area_applied` compartilham execução, sem custo; rings são independentes. Esses dados são diagnósticos privilegiados: nenhuma posição secundária ou sucesso de pulso alimenta memória/decisão do NPC.

J / `set_debug_view(npc_routes:true)` mostra centro/raio nominal em três círculos verde-lima durante toda a vida. Ciano continua sendo preparo direto; laranja é entrega instantânea. Até16 áreas ativas; pulsos não acumulam cópias visuais do mesmo volume. Guias não indicam destinatários garantidos nem VFX. Reset/load/edição/undo recompilam e removem runtime; replay guarda estado, entrada pendente e ordem para inspeção/captura de prefixo.

## Limites e alternativas

Pool16, ring128, array9 de candidatos e executor de até8 efeitos por destinatário. Pior tick pode ter16×9 transações, além de consultas de oclusão/atributos. Sem alocação de coleção por pulso, strings ou JSON no gameplay; diagnóstico aloca sob demanda. Scan linear é deliberado para limites do laboratório, não promessa de escala MMO. Sem scheduler global, broadphase nova, persistência entre sessões ou nova dependência.

Alternativas adiadas: membros continuamente acompanhados com eventos enter/exit, aura em terceiros, dispel de entidades de área, volumes arbitrários, VFX e escolha tática de agrupamentos pelo NPC. Cada uma exige contrato próprio; não são automaticamente entregues por pulsos persistentes.

## Demonstração

`examples/npc-persistent-area.scene.json`: P inicia/pausa; WASD move; J guias; R reinicia; clique seleciona alvo; X cancela preparo.

- 1: projétil no selecionado; contato cria zona independente6s, pulso inicial e a cada1s.
- 2: ponto sob cursor; zona6s, mesmo ritmo; nova aplicação renova/reposiciona a anterior.
- 3: zona independente2m no local atual do jogador, com dano nele também; caminhar para fora evita pulsos seguintes. Encerra se jogador morrer.
- 4: buff próprio +25% no componente fire por10s; altera próximos pulsos já ativos.
- 5: cura em área500, incluindo jogador.
- 6: dissipação em área das resistências pessoais dos alvos, não dos volumes.

Verificações: `python3 scripts/run_verifications.py --only persistent_area` e regressão completa headless. Entrada física e aceite humano permanecem gates separados.
