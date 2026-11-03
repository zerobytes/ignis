# Vitalidade, dano, cura e morte (EAF-39)

EAF-58 acrescenta [ressurreição explícita](resurrection.md), life_index e eventos de retorno. Cura normal continua rejeitando mortos; os limites de morte terminal descritos abaixo documentam o recorte original EAF-39.

Contrato aprovado antes da implementação. Vitalidade é um componente opcional de ator, independente de grants e pools de habilidades. O executor de habilidades aplica dano/cura tipados com os mesmos custos, condições e commits usados por jogador e NPC. Cena/replay27; cenas anteriores continuam carregando sem vitalidade. Nenhuma dependência nova.

## Dados e autoria

`entity.vitality = {initial, maximum, scale}`. `maximum` entre1 e10^12, `initial` entre0 emaximum, `scale` entre1 e1.000.000. Unidades inteiras i64; escala é apresentação (1250/1000 =1,25). Initial0 autoriza um cadáver inicial, sem fabricar um evento de morte. Ausência do componente significa que o ator não participa deste ciclo vital; receber dano/cura falha. Um alvo passivo pode ter vitalidade sem ter qualquer habilidade.

Neste recorte, Player/Npc podem ter um único pool vital. Outros recursos continuam ilimitados semanticamente pelo catálogo, sujeitos aos budgets atuais; energia/foco não são aliases de vida. Destrutíveis de outros tipos, múltiplos órgãos/pools vitais e regras alternativas de mortalidade permanecem extensões a discutir.

Efeitos adicionados ao catálogo:

- `damage {recipient: source | target, amount}`.
- `heal {recipient: source | target, amount}`.

Amount é positivo, até10^12, na unidade inteira da vitalidade do destinatário. Não há conversão automática entre escalas de atores. O projeto deve padronizar escalas quando deseja magnitudes equivalentes entre destinatários. Dano satura em0; cura satura no máximo. Cada recibo mantém requested/applied e `vitality: {kind, before, after, excess}`. Applied e excess são magnitudes positivas; kind distingue operação. Efeitos genéricos de recurso continuam usando deltas com sinal.

Autoria via `upsert` ou `edit_scene`, incluindo `set_ability_catalog` no mesmo lote; validação precede qualquer mudança. Edição/load/reset/undo/redo recompilam estado a partir do documento. Morte durante partida não altera `initial`, configuração enabled, IDs ou o documento salvo.

## Transação e ordem

O plano temporário contém todos os recursos, estados vitais e efeitos. Falha em qualquer efeito, pool, alvo, custo ou orçamento de ruído rejeita tudo. Nenhum callback externo ocorre durante o commit. Início do cast valida; término revalida. Morte do destinatário durante um cast pode invalidar o resultado sem cobrar custo/cooldown.

Vida/morte é decidida **ao final da execução inteira**. Dano que leva a0 seguido de cura positiva na mesma habilidade pode terminar vivo; não há morte intermediária nem ressurreição implícita. Se o estado final for0, uma única transição registra tick, ator, origem, execution_id e índice zero-based do último efeito que o levou de positivo a0. Overkill adicional enquanto já está em0 não substitui essa causa. A habilidade suicida que concluiu aparece como completed.

Preservada a ordem de EAF-38: sensores/intenção; cancelamentos; casts existentes; novas solicitações; motores/física. Em cada fase, IDs de atores em ordem lexical. Uma execução letal anterior cancela imediatamente casts pendentes do morto (`cancelled_by_death`) e impede suas solicitações posteriores (`actor_dead`). Um cast já vencido pode executar antes de uma nova habilidade instantânea, conforme essa ordem. Não são mortes simultâneas por lote.

Cura num vivo cheio é válida: aplica0, paga custos e cooldown. Morto não recebe dano/cura e não ressuscita; alvo morto rejeita a ação sem débito. Vida ausente produz `missing_vitality`. `target_dead` é emitido após restrições de conhecimento/alcance/oclusão; NPC sem evidência pode receber `target_unavailable`. Diagnóstico não fornece localização à decisão do NPC.

## Corpo e conhecimento

Morte desliga motores, objetivos, percepção, patrulha e comunicação do próprio ator. Casts e reservas de passagem são cancelados antes da física do mesmo tick. O corpo continua presente, selecionável, colidível e sujeito a gravidade/inércia; um voador morto cai. Como os corpos atuais têm rotação bloqueada, isto ainda não é ragdoll. Um cadáver pode continuar obstruindo uma passagem, embora não possua mais a reserva. Volumes continuam medindo ocupação física, inclusive cadáveres.

Entrada de movimento não aplica velocidade ao jogador morto. Passos automáticos não são gerados pelo deslizamento do cadáver. Ruídos legítimos anteriores à morte ainda podem ser consumidos no tick seguinte. `emit_noise` com origem num ator morto é rejeitado; emissão diagnóstica numa posição continua disponível. Não há som de morte automático.

Outros NPCs não consultam o estado global de morte para descartar memória. Enquanto o alvo estiver oculto, continuam com a pista congelada e seus budgets normais. Uma observação visual aceita, inclusive confirmação acumulada quando configurada, permite registrar o alvo como morto; a política omnisciente tem essa informação imediatamente na próxima fase sensorial. Um conjunto limitado de identidades mortas conhecidas impede reaquisição por relatos antigos. Não há transmissão de mortes pelo rádio neste recorte. O diagnóstico contém proveniência do dano, mas não gera pista de posição ou aggro automaticamente.

## Inspeção, replay e performance

`inspect_vitality` retorna atores com units/maximum/scale/life e eventos de morte. `inspect` inclui o mesmo estado; `inspect_abilities` inclui recibos de dano/cura e causas de falha/cancelamento. `inspect_npcs` mostra phase dead e IDs em known_dead quando observados. Inspeção global é ferramenta de diagnóstico, distinta do conhecimento individual dos NPCs.

Runtime usa arrays de9 atores, IDs compilados, inteiros e enums. A transação copia9 estados vitais além da matriz de recursos existente. Sem JSON/parsing por efeito. Anel de256 mortes pré-alocado e recibos de habilidades limitados; inspeção fria serializa JSON. Desativação física acontece uma vez por ator, evitando acordar cadáveres a cada tick. Custos mantidos e checklist em [performance.md](performance.md).

Replay27 inclui estado vital/eventos, habilidades, conhecimento e entradas pendentes. Prefixos reconstruídos têm a mesma vida e imagem do instante correspondente; reset restaura o documento. Replays exigem build compatível, como antes. Isso não é savegame de campanha.

J (`set_debug_view.npc_routes`) mostra vida na primeira barra, com fundo vermelho/parte verde, energia azul abaixo e cast amarelo. Cadáveres ficam cinza, barra vazia e sem guias sensoriais. Tudo deriva do mesmo estado inspecionável pelo MCP.

## Demonstração e teste manual

Cena autorada pelo MCP: `examples/npc-vitality.scene.json`.

```sh
./target/release/ignis --scene examples/npc-vitality.scene.json
```

Começa pausada. P inicia/pausa; WASD move; J mostra barras; clique seleciona destinatário; X cancela seu cast; R reinicia a cena inteira. Nenhuma janela é aberta pelos roteiros de verificação.

- Jogador laranja, vida100: **1** causa35 de dano ao selecionado (cast0,5s, cooldown0,75s); **2** cura35 em si mesmo (0,75s/1,5s); **3** cura35 no selecionado (0,5s/1s). Alcance12m e custo5 de energia em todas; energia inicial1000, sem regeneração.
- Vermelho à direita: ataca o jogador observado por20, alcance6m, cast0,75s, cooldown2s. Permanece no posto; fora do alcance falha e retenta após1s. Não se aproxima automaticamente para atacar.
- Azul à esquerda: cura o jogador observado por12, alcance8m, cast1s, cooldown3s. Permanece no posto. Pode gastar energia com vida cheia, conforme o contrato; a política de necessidade/triagem será outro incremento.
- Roxo ao fundo: alvo passivo com vida100 e sem grants. Três ataques do jogador o matam. Pode ser curado antes disso; após a morte, cura falha. Todos têm corpo físico.

Teste sugerido: ataque/cure o roxo; aproxime-se do vermelho e observe a sua vida baixar; use2 ou aproxime-se do azul para recuperar. Mate o vermelho e veja que seus ataques cessam. Se o jogador morrer, movimento e habilidades param; R restaura todos. Barras/recibos permitem distinguir alvo inválido, falta de alcance e cast em andamento. Foco/teclado nativo e aceite humano são gates separados de MCP/offscreen.

`python3 scripts/run_verifications.py --only vitality` valida autoria, vida/dano/cura/morte, atomicidade, histórico, replay e pixels. Testes Rust também cobrem ordem, morte oculta/rádio, passagens e queda física. Ressurreição/respawn, resistência/armadura/crítico, efeitos persistentes, projéteis/área, loot/gameover e animação ainda não fazem parte deste contrato.
