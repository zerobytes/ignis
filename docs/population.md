# População em execução — retirada explícita de NPCs

**Continuidade EAF-70:** nascimento, templates e reutilização pública estão implementados em [spawn-templates.md](spawn-templates.md), cena47/replay51. A descrição abaixo documenta o recorte EAF-67; suas limitações sobre ausência de spawn foram superadas. Aliases históricos agora são resolvidos somente enquanto o indivíduo original está presente; depois retornam null. Encerramentos de contato são congelados na retirada, antes de reutilizar seus metadados.

EAF-67 publica `despawn_actor` e `inspect_population` no dispatcher compartilhado/MCP. Cena46 preservada; replay50 inclui pedidos e recibos. Primeiro incremento da API de população: nascimento/templates, reciclagem pública, ondas e reposição automática ainda não estão disponíveis. A separação em relação a EAF-63 permite validar retirada em todos os sistemas antes de instalar novos indivíduos.

## Contrato

`{"op":"despawn_actor","actor":"npc_id"}` aceita alias autorado ou token de `inspect_instances`. Resolve a referência original ao enfileirar; o pedido nunca resolve novamente o alias. Aceita somente NPC presente, vivo ou cadáver. Jogador e corpos estáticos ficam fora deste contrato. Resposta `accepted:true`, `phase:pending`, `request_id`, `tick`, `instance_id` confirma admissão, não aplicação. `expected_revision` continua disponível para proteger o documento; identidade/escopo protegem o indivíduo em execução.

Pedidos ficam pendentes enquanto pausado e são consumidos antes dos sensores, habilidades e física do próximo tick. O `tick` do recibo é o da fase inicial, antes do incremento físico. Ordem é a sequência dos pedidos de população; retirar antes das habilidades independe da ordem de chamadas MCP entre essas duas filas. Dois pedidos para o mesmo indivíduo podem ser admitidos no mesmo tick: primeiro `despawned`, segundo `instance_unavailable`. Depois de aplicar, um novo comando para o alias/token ausente falha já na admissão. Erros de admissão não consomem request_id nem alteram fila/mundo/revisão. Nenhuma morte, dano ou recompensa é inventada.

`inspect_population` informa atores presentes, pedidos pendentes, recibos, contadores e limites. `authored_id` nos recibos identifica a origem no documento; `instance_id` identifica o indivíduo retirado. `inspect_instances` lista apenas presentes. `inspect_world` e `inspect_instances` declaram `dynamic_population:true`, `despawn_supported:true`, `spawn_supported:false`. `inspect` inclui população; saves da cena continuam exclusivamente autorais.

## Aplicação e continuidade

1. Revalidar referência e tipo sem mutação. Capacidade para origem sobrevivente já foi reservada na admissão das entregas; retirada não tenta alocar uma nova lease.
2. Capturar bases/contribuições, prazos, facção/vida da origem nas reservas existentes; cancelar cast/canal do autor com `source_unavailable`, sem novas cobranças. Pagamentos anteriores continuam pagos.
3. Invalidar referência e remover corpo/collider, vida corrente, navegação, objetivos/motores, patrulha em execução e estado de passos; liberar passagens. Limpar seleção sem recuperar a autoria como posição viva.
4. No restante do tick, referências de alvos/casts pendentes falham por identidade; buffs do owner ausente terminam como `owner_removed`, periódicos no destinatário como `instance_changed`. Auras ancoradas encerram; projéteis, áreas fixas e periódicos em terceiros podem continuar conforme [runtime-world.md](runtime-world.md).
5. Contatos emitem término na fase física; triggers emitem `exit` com `reason:removed` no pós-física, usando a identidade original. Eventos normais têm `reason:overlap`. Observadores conservam pista congelada sujeita aos seus budgets; nenhuma notificação global apaga ou renova memória.

Sobreviventes mantêm instância, vida, saldos, cooldowns, casts e trajetória física — exceto efeitos legítimos da ausência, como colisão/percepção. Não há `Engine::replace`, clonagem de Scene por retirada nem reconstrução dos corpos sobreviventes. O reset/edição continuam reconstruindo a execução original e invalidando tokens anteriores, como antes.

Os arrays compilados e slots numéricos permanecem estáveis. Metadados imutáveis de entidades/contatos/rotas e saldos/cooldowns inativos permanecem em armazenamento limitado à cena inicial; são inacessíveis por comandos/inspeção de atores vivos após a referência ser invalidada. Não há novo indivíduo usando esse armazenamento nesta etapa. Spawn deverá inicializar explicitamente todos os componentes e implementar ordem por InstanceId antes de publicar reuso; preservar bytes inativos não autoriza herança de estado. Esta etapa não conclui o catálogo de templates ou o desacoplamento de todos os índices autorados.

## Limites e replay

Até64 pedidos por tick,4096 por execução e256 recibos no anel. Recibo antigo sai do anel com contador de descarte; log de entradas permanece limitado para reprodução. Preservados9 atores e demais pools atuais. Entradas usam referências numéricas Copy, sem string/UUID por evento interno. Serialização e resolução textual só na fronteira/inspeção/replay. Não há lock, thread ou dependência nova.

Replay50 guarda fila completa, inclusive duplicados, pendentes e aplicações rejeitadas, e compara recibos/contadores mais registro de identidade e sistemas existentes. Valida orçamento/ordem/tick/identidade antes de consumir cada grupo; prefixo inclui pedidos daquele tick ainda pendentes. Grupos futuros não são executados para inspecionar um prefixo. Mutação manual de um replay não autoriza despawn de jogador nem de uma referência inexistente. Replays exigem mesmo build; sem promessa de compatibilidade binária entre versões.

Verificar com `python3 scripts/run_verifications.py --only population`; regressão completa headless obrigatória para concluir. Cenários cobrem captura GPU após retirada, prefixo com fila pendente, imagens idênticas, fontes sobreviventes e reset. Teclado/foco/janela não participam desta validação.
