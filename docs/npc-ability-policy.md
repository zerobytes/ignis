# Decisão de habilidades dos NPCs

Extensão opt-in do objetivo `use_ability`, cena/replay28 (EAF-40). Usa o seletor de prioridades, percepção, navegação e executor compartilhado existentes. Não introduz behavior tree, triagem entre múltiplos alvos ou nova capacidade física.

## Autoria e responsabilidade

```json
{
  "priority": 2,
  "when": "always",
  "action": {
    "kind": "use_ability",
    "slot": 1,
    "retry_ticks": 30,
    "policy": {
      "vitality_below": { "subject": "target", "basis_points": 7000 },
      "approach": { "margin": 0.4 }
    }
  }
}
```

Configurar via `upsert` ou `edit_scene`, no mesmo documento/MCP da engine visual. Ausência/null de `policy` preserva o comportamento anterior, inclusive tentativas fora do alcance. `policy: {}` habilita filtros de disponibilidade, custo, relação, visão e alcance, sem aproximação nem limiar vital. Não é preciso ter vida para usar uma habilidade sem limiar.

A política pertence ao objetivo do NPC, não à definição de cura/dano. Alternativa considerada: adicionar a condição à habilidade. Isso bloquearia também o uso deliberado pelo jogador e impediria perfis distintos usando a mesma habilidade. O executor continua responsável por validade, custos e efeitos; nenhuma decisão aplica dano ou debita recursos diretamente.

`vitality_below` aceita `source` (próprio NPC) ou `target` (destinatário da habilidade), com1..10000 pontos-base;7000 corresponde a70%. A comparação é estrita:70/100 já não satisfaz “abaixo de70%”. Inteiros evitam arredondamento do saldo e permitem diferentes máximos/escalas. Esse limiar não é histerese de cura: o próximo dano abaixo dele pode tornar a ação elegível novamente. A alternativa de unidades absolutas fica para outro contrato, se necessária.

O NPC conhece sua própria vida. Vida de outro ator só entra após observação visual confirmada daquele ator no tick atual (ou modo onisciente explicitamente autorado). Memória, som e rádio não fornecem vida atual. Sem componente vital, a condição resulta em `missing_vitality`. Esse recorte assume que observação confirmada autoriza avaliar a vida; jogos que ocultem lesões mesmo sob visão precisarão de uma política sensorial adicional. Não há consulta de feridos globais, seleção de aliados por menor vida ou cura de ator lembrado atrás da parede.

## Aproximação, alcance e preparo

`approach` é opcional e somente para seleção `actor`. `margin` deve ser finita, >=0,1m e <=alcance−0,1m. Ao encontrar o alvo fora do alcance, o objetivo permanece ativo na fase `approaching` e entrega um `NavGoal` ao motor existente. A entrada para executar fica em alcance−margem; depois, o NPC pode continuar executando até o alcance externo. Essa faixa evita alternância por oscilações pequenas na borda. Um NPC que já começa dentro do alcance pode executar imediatamente.

Distância de habilidade é centro a centro XYZ. Para navegação terrestre, o raio XZ desconta a diferença de altura; reserva4cm para a tolerância do planejador. Navegação espacial mantém raio XYZ. O corpo pode impedir a aproximação suficiente quando o alcance é pequeno; isso não vira sucesso. Sem voo, apoio ou caminho, o diagnóstico de navegação continua `unreachable`/motivo equivalente. `approaching` expressa intenção, não caminho garantido. Território, colisão e capacidades existentes continuam valendo; não há pathfinder ou raycast adicional no filtro.

Perder visão torna uma ação direcionada inelegível. O próximo objetivo autorado decide investigar, esperar ou patrulhar. Um fallback `respond` usa a pista e os budgets existentes; `wait` permanece parado. Cooldown/retry também libera o seletor para o fallback. Para manter distância durante cooldown e investigar somente após perder visão, colocar `wait/visible_target` antes de `respond/visual_memory`, com mesma prioridade menor que a habilidade.

Preparo em andamento mantém o objetivo elegível apesar da ocupação do executor. Perder a condição (por exemplo, outra cura tornar o limiar desnecessário), perder visão ou trocar o alvo cancela a execução automática e aplica retry. Dentro de um cast já iniciado, alcance/oclusão são revalidados pelo executor no vencimento; esse recorte não move o NPC para salvar um cast. Travessias espaciais já comprometidas mantêm sua semântica física existente.

As intenções leem o snapshot anterior aos efeitos do tick. Outra execução pode curar/matar/mover condições de recursos depois da decisão; a revalidação transacional continua definitiva. Portanto, duas curas admitidas no mesmo tick podem produzir excesso, conforme o contrato de EAF-39. Não há reserva de cura nem promessa de ausência de toda tentativa rejeitada: o filtro não duplica validação geométrica/efeitos do executor.

## Inspeção e limites

`inspect_npcs`/`inspect` expõem `objectives.ability_decisions`: array fixo de8 entradas por NPC, alinhado aos objetivos autorados, com `phase`, índice de ator `target` e `target_id` resolvido. `eligible_mask` informa candidatos e `active` informa o escolhido. Uma entrada `ready` não garante seleção; prioridade/condição geral ainda se aplicam. Entradas não configuradas são `unconfigured`.

Motivos: `unavailable` (cast externo, cooldown/retry ou condição própria de recurso), `insufficient_resources`, `no_visible_target`, `relation_mismatch`, `missing_vitality`, `vitality_satisfied`, `out_of_range`, `target_changed`, `approaching`, `ready`, `casting`. A primeira barreira determina o motivo; custos/saúde são reavaliados quando a disponibilidade permitir. Recibos em `inspect_abilities` continuam sendo evidência da execução real. Estado após um passo descreve a decisão pré-física/pré-commit daquele tick, podendo ainda mostrar `ready` no tick em que o efeito acabou de concluir.

Políticas podem ser geradas proceduralmente como qualquer objetivo em lote. Até8 objetivos/NPC e9 atores no laboratório, sem novo limite ou dependência. Decisões usam enums/arrays; edição/undo/reset reconstroem o estado; replay inclui decisões, recursos, vida e entradas pendentes. Formatos anteriores sem política continuam carregáveis; replay requer versão28/build compatível.

## Demonstração

```sh
./target/release/ignis --scene examples/npc-ability-policy.scene.json
```

Começa pausada: P inicia/pausa, WASD move, J barras/rotas, V visão, clique seleciona alvo, X cancela seu cast, R reinicia. Os testes não abrem janela.

- Vermelho: visão360°/14m, memória90ticks (1,5s) e busca120ticks (2s); aproxima para atacar por15, alcance2,5m/margem0,4m, preparo0,5s e cooldown2s. Espera durante cooldown enquanto vê; investiga a memória quando perde visão.
- Azul: visão360°/14m; observa o jogador, começa a aproximar/curar somente abaixo de70% de vida. Cura20, alcance4m/margem0,4m, preparo0,75s, cooldown3s. Sem necessidade, espera.
- Roxo: parado; cura30 em si mesmo somente abaixo de50%, preparo0,75s e cooldown2s. Sua decisão não depende de ver o jogador.
- Jogador laranja:1 causa35 no selecionado (alcance6m, preparo0,25s/cooldown0,5s);2 cura35 em si (0,5s/1s);3 cura35 no selecionado (6m,0,5s/1s). Custo5, energia1000 sem regeneração. Todos começam com100 de vida.

Deixe o vermelho chegar e observe o azul esperar até haver necessidade. Fuja para trás da parede verde para testar a perda de visão. No roxo, o primeiro ataque deixa65 e não dispara cura; o segundo deixa30 e dispara recuperação para60. Um terceiro ataque durante o preparo pode matá-lo antes da cura: morte permanece definitiva, R restaura a cena. Matar o vermelho continua interrompendo seu comportamento.

Validação: `python3 scripts/run_verifications.py --only ability_policy`, testes Rust `ability_policy` e regressão completa sem janela. Demonstração/teclado nativo dependem do aceite humano; MCP/offscreen não comprovam foco ou entrada física.

## Extensão EAF-55

`policy.allies` habilita triagem de aliados atualmente observados, independente da memória de combate; `allies` ausente preserva este contrato. Ranking, inclusão própria, compromisso de cast e revalidação no commit em [ally-triage.md](ally-triage.md).
