# Interrupção externa de habilidades (EAF-48)

Jogador e NPC podem interromper a preparação ou canalização de outro ator pelo mesmo executor. `interrupt` é um efeito; `interruptible` é uma propriedade da habilidade interrompida. São independentes de `dispel`/`dispellable`, que operam sobre agendas periódicas já entregues.

## Contrato

Cena/replay37. Exemplo de operador dentro de `effects`:

```json
{"kind":"interrupt","recipient":"target","require_active":true}
```

`recipient` precisa ser target; a definição usa seleção actor ou spatial/projétil. Self_actor, source, canal que entrega interrupções e agenda periódica de interrupções são recusados neste recorte. Alvo igual à origem não é elegível. `interruptible` tem padrão true, omitido na serialização quando true; false exige versão37. Cenas antigas preservam o comportamento anterior. Não há probabilidade de resistência ou override por execução.

- `require_active:true`: exige execução atual elegível. Ausência/proteção gera `no_interruptible_execution`. Entrega direta falha sem custo/cooldown.
- `require_active:false`: permite sucesso pago sem interrupção. Recibo distingue `not_casting`, `protected`, `already_selected` e `interrupted`.
- `interruptible:false`: bloqueia apenas este efeito externo. Cancelamento voluntário, morte, movimento incompatível, perda de visão/alcance/política e reset continuam encerrando execuções.

Validação de fonte, percepção, alcance, relação, oclusão e custos permanece no executor comum. Na escolha/preparação, NPC só consulta execução/proteção de alvo após confirmar visão atual; não aprende posição pelo cast. Um projétil já lançado resolve o contato físico independentemente da visão posterior da origem; seu recibo não atualiza a memória perceptiva. Inspeção de ferramentas é observabilidade global, não percepção NPC. Não há silêncio ou bloqueio por duração, escolha tática de melhor momento para interromper, leitura de cast pelo rádio ou triagem automática.

## Transação e tempo

O plano seleciona o ID atual de execução do destinatário, sem copiar todo seu Cast. Seleção e efeitos de vida/recursos/remoção/custo são transacionais: falhar um operador preserva todos. Operadores repetidos não selecionam duas vezes a mesma execução; um segundo obrigatório falha o plano, opcional vira no-op. Interrupção é aplicada antes da notificação de morte; uma composição com dano letal registra a interrupção explícita uma vez, além da morte normal.

A pré-validação no início da preparação não reserva o cast do alvo. No commit consulta-se a execução atual: se o alvo tiver cancelado e iniciado outra, esta será a candidata. Alternativa de travar o ID no pedido exigiria API de seleção de execução e faria parte de outro contrato. Aqui seleciona-se um ator, coerente com dano/cura/dispel existentes.

Entrega por projétil cobra no lançamento e revalida no impacto contra o ator fisicamente atingido. Actor exige candidato elegível também antes de lançar, conforme a pré-validação comum; spatial não tem vítima no lançamento. Ausência/proteção no impacto não reembolsa. Interromper a origem não remove disparos ou agendas anteriores.

A execução interrompida preserva pulsos, custos e cooldown já efetivados. Preparação ainda não paga continua sem cobrança/cooldown. O ator não inicia outra execução no mesmo tick; no seguinte pode receber novo comando, sem silêncio implícito. Execuções automáticas aplicam também o retry do objetivo.

Ordem preservada: cancelamentos explícitos → pulsos periódicos → impactos → movimento → casts/canais existentes em ordem de ID de ator → novas ordens. Uma interrupção instantânea pedida no tick de um pulso acontece depois desse pulso. Uma interrupção preparada vencendo no mesmo tick disputa pela ordem de ator; projéteis impactam antes dos canais. Não há rollback de fases anteriores nem simultaneidade arbitrária.

## MCP, persistência e recibos

Autoria via `set_ability_catalog`/`edit_scene`; execução por `activate_ability`, `aim_ability` ou `aim_ability_at`; inspeção via `inspect_abilities`. Nenhuma entrada exclusivamente visual. Grants/cast expõem `interruptible`; estado expõe `interrupted_until`, barreira interna de um tick, não duração de debuff.

O efeito aplicado registra status, ID da execução candidata e `channeling`. O recibo da vítima usa `interrupted_by_ability` com `interrupted_by:{actor,execution_id}`; assim atacante e vítima podem ser correlacionados. Anel de recibos permanece limitado256, com descarte explícito. A barra amarela/ciano desaparece ao terminar a execução; não há novo VFX de impacto. Replay37 reconstrói decisões, recibos e pixels; replay de build/versão antigos não é aceito como atual.

## Demo

```sh
./target/release/ignis --scene examples/npc-interrupt.scene.json --window
```

P inicia/pausa, WASD move, clique seleciona, J mostra barras, X cancela sua execução, R restaura.

- Roxo prepara cura própria por2s; use **1** para interrupção direta ou **2** para projétil no selecionado. Alcance12m; custo5; cooldown0,5s. Projétil6m/s: o cast precisa ainda existir quando ele chegar.
- Azul canaliza cura protegida: `interruptible:false`. **1** falha sem cobrar; **2** com alvo protegido não lança. Continua sujeito a cancelamento/morte/regras do canal.
- **3** canaliza cura própria normal; **4** canaliza a mesma cura protegida. Preparação0,5s,10 pulsos de2 a cada0,5s, custo5 na entrada e cooldown1s. Ambos permitem andar.
- Vermelho fica parado à direita do jogador. Aproxime-se a menos de4m e use3: sob visão atual, ele tenta interromper a cada0,5s, cooldown2s quando consegue. Use4 para comparar a proteção; X ainda cancela voluntariamente. Ele não causa dano nem persegue nesta demo.

Jogador/roxo/azul começam com50 de vida; energias1000 para testes repetidos. Roxo tem cooldown2s após completar e retry0,5s após interrupção. Azul usa o mesmo canal protegido do jogador e repete após cooldown, inclusive com vida cheia; objetivo aqui é disponibilizar uma execução para teste, não triagem de cura.

Validação automatizada: testes Rust de transação/ordem/privacidade e `scripts/run_verifications.py --only interrupt`; suíte completa headless inclui render/replay. Aceite manual/teclado nativo permanecem separados. Não conclui área, modificadores, resistência, silêncio, respawn, NPCv1 ou M08/M09.
