# Atributos extensíveis e modificadores temporários (EAF-49)

O registro identifica atributos por chave; buffs contribuem operações sobre esses atributos durante um intervalo. O executor de buffs não conhece os nomes criados pelo jogo. Valor base, contribuições e valor efetivo são separados: expirar/remover uma contribuição não restaura uma cópia antiga por cima de outros buffs.

## Alcance e extensibilidade

`ability_catalog.attributes` contém `definitions`, `bases`, `buffs` e `tags`. Os primeiros tipos são booleano e número fixo. Chaves são abertas, até65 incluindo quatro consumidores nativos (61 personalizados). Uma definição personalizada não exige novo caso no executor; um sistema que pretenda usar esse atributo resolve sua chave na compilação e consulta `Registry::evaluate` com contexto apropriado. A identidade numérica é local ao documento compilado e deve ser resolvida novamente após edição/reset/load.

O runtime genérico identifica beneficiários por entidade, independentemente de corpo, e bases podem pertencer a qualquer entidade. `inspect_attribute` consulta esses atributos pelo MCP. Nesta rodada a entrega de habilidades continua selecionando atores, como o contrato existente; o teste Rust também exercita contribuição em entidade não ator pela API genérica. Isso não acrescenta seleção de objetos ao gameplay nem transforma chaves em escrita arbitrária de campos Rust.

Consumidores integrados agora:

| Chave | Base nativa | Consumidor e contexto |
|---|---|---|
| `ability.interruptible` | Propriedade da habilidade | Ator que executa o cast/canal, tags e ID dessa habilidade |
| `effect.dispellable` | Propriedade da habilidade que instalou o DoT/HoT | Autor original do efeito, tags e ID dessa habilidade, consultados na dissipação |
| `combat.damage_received.multiplier` |10000 =1× | Destinatário do dano, tags de cada componente e ID da habilidade causadora; dano direto, impacto e cada pulso |
| `combat.damage_dealt.multiplier` |10000 =1× | Autor original da execução, mesmo contexto por componente e avaliação dinâmica no commit/impacto/pulso |

Velocidade, tamanho de colisor, vida máxima, custos, render, vetores/referências e outros domínios precisam de seus adaptadores/tipos/regras de invalidação. A base está aberta para eles; ainda não são consumidores implementados. Não há ABI de extensões ou linguagem de scripts nova. Transformar uma chave personalizada em um efeito físico exige esse consumo explícito.

## Dados e operações

Valores são tipados:

```json
{"kind":"bool","value":false}
```

```json
{"kind":"number","units":15000}
```

Números usam escala fixa10000;15000 representa1,5. O autor escolhe a unidade do atributo. Limites gerais±10¹² unidades; cada definição numérica pode restringir `minimum`/`maximum`. Bool não admite limites. Os quatro atributos nativos não podem ser redefinidos no registro. O multiplicador de dano limita o resultado a0..1000000 (0..100×).

Exemplo de definição customizada e base por entidade:

```json
{"definitions":[{"key":"craft.temperature","default":{"kind":"number","units":10000},"minimum":0,"maximum":100000}],"bases":[{"entity":"floor","attribute":"craft.temperature","value":{"kind":"number","units":20000}}]}
```

`bases` substitui a base padrão para aquela entidade/chave, inclusive quando a base nativa viria da habilidade consultada. Isso é autoria, não mutação temporária. Cada binding precisa de entidade/chave existentes, valor compatível e par único. Até512 bindings; nenhuma mudança silenciosa de tipo.

Operações: `set {value}` para qualquer tipo; `add {units}`, `multiply {basis_points}`, `min {units}` e `max {units}` para números. Multiplicação aceita±1000000 pontos-base e divide por10000. Intermediários i128, truncamento em direção a zero e clamp depois de cada operação; sem NaN/infinito. `min` limita o teto, `max` limita o piso. Sets precisam respeitar tipo/limites da definição. Ajuste final do dano é limitado a10¹² unidades; vida já possui esse teto. Recibo preserva dano original, multiplicador, valor ajustado e dano efetivo; excess da vitalidade é calculado sobre o valor ajustado.

## Buff e contexto

Exemplo: imunidade a gelo com vulnerabilidade a fogo por10s, no laboratório de60ticks/s:

```json
{
  "id":"polar",
  "duration_ticks":600,
  "reapply":"refresh",
  "category":"enhancement",
  "dispellable":true,
  "modifiers":[
    {"attribute":"combat.damage_received.multiplier","tags_all":["ice"],"operation":{"kind":"multiply","basis_points":0}},
    {"attribute":"combat.damage_received.multiplier","tags_all":["fire"],"operation":{"kind":"multiply","basis_points":15000}}
  ]
}
```

Uma habilidade aplica esse buff com `{"kind":"apply_buff","recipient":"target","buff":"polar"}`. Source/target na entrega direta, apenas target em projétil. Channel pode aplicar buff nos seus pulsos, conforme o mesmo contrato transacional; agenda periódica independente continua restrita a dano/cura/recurso. O ID precisa existir antes da edição ser aceita.

Desde a cena39, tags precisam ser declaradas em `attributes.tags`; referências desconhecidas/duplicadas são rejeitadas. Cenas antigas são migradas preservando classificações. Tags são classificações autoradas, sensíveis a maiúsculas: fire/ice/poison são nomes da demo, não enums Rust. `Definition.tags` descreve o contexto da habilidade para interrupção/dissipação. Dano usa tags próprias por componente, conforme [dano composto](damage-components.md). `tags_all` exige todas as tags listadas; lista vazia aceita qualquer contexto. Filtro `abilities` aceita qualquer ID listado; vazio aceita qualquer habilidade. Tags e filtro de IDs se combinam por E. Até64 tags distintas, compiladas em máscara u64; tags usadas nos filtros/definições/componentes referenciam o registro explícito `attributes.tags`. Uma habilidade marcada fire e ice satisfaz ambos os filtros; as operações compõem, podendo resultar em imunidade. Não há inferência automática de elementos, equipe, proximidade ou origem a partir de nomes.

Proteção de cast usa set false em ability.interruptible; proteção de venenos usa set false em effect.dispellable filtrado por poison. O buff fica no autor, mas protege suas agendas em outros atores. O filtro não protege efeitos de outro autor. Nesta rodada a avaliação é dinâmica: protege inclusive efeitos previamente instalados e deixa de proteger ao expirar/remover o buff. Não existe captura permanente da proteção no momento de aplicar o DoT.

## Ordem, duração e transação

Regras são compiladas por atributo, em ordem crescente de `(priority, posição da definição de buff, posição do modificador)`. Prioridade é i16, padrão0. Cada regra se aplica às instâncias correspondentes em ordem de aplicação. Operações posteriores veem o resultado das anteriores; set posterior substitui resultados anteriores. Empates entre definições são resolvidos pela ordem autorada, não pelo nome. `independent` aplica a contribuição de cada instância, inclusive multiplicações repetidas; duas proteções false continuam protegendo enquanto pelo menos uma contribuição persistir, na ausência de override conflitante. Prioridades permitem explicitamente sobrepor uma proteção com true. Não há uma regra oculta de deny-wins.

Duration1..216000ticks. Buff aplicado em t vale em `[t,t+duration)`. Expiração é processada antes das ações do tick; consultas já desconsideram contribuições vencidas mesmo na fronteira anterior ao próximo step. Recibo de expiração surge ao processar esse step. `refresh` substitui instância da mesma definição/origem/beneficiário, reinicia prazo e ganha novo ID; `independent` acrescenta instância. Pool64, definições64, até8 modificadores por definição, recibos256 com contador de descarte. Morte do beneficiário remove buffs; morrer a origem não remove buffs entregues a outro beneficiário. Sem persistência entre mortes/respawn nesta etapa.

Plano transacional guarda até8 instalações e uma máscara de remoção. Conta instalações pendentes sem duplicar refresh e permite reutilizar vagas removidas por operadores anteriores. Falha posterior preserva buffs, vida, recursos e custos. Direto cobra no commit; projétil cobra lançamento, instala na vítima física no impacto e inicia duração nesse instante. Falha no impacto não reembolsa. Imunidade é consultada no impacto/pulso atual, não congelada no lançamento/aplicação.

**As contribuições consultadas em uma execução são as do início da transação.** Aplicar/remover um buff e causar dano ou tentar interromper na mesma lista não muda retroativamente a avaliação dessa lista. Buffs tornam-se visíveis às execuções seguintes do tick após commit. Assim, aplicar imunidade e dano na mesma habilidade ainda pode causar esse dano; dissipar uma proteção e tentar a interrupção requerida no mesmo plano ainda pode falhar e preservar tudo. Estados de vida/recursos continuam seguindo a composição existente da lista de efeitos.

Dissipação existente também remove buffs por categoria, respeitando dispellable do próprio buff. A categoria compartilha o namespace existente. Ordem: agendas periódicas elegíveis primeiro, depois buffs em ordem de aplicação, compartilhando max_instances/require_match. A propriedade do próprio buff é estática nesta etapa; effect.dispellable dinâmico é o adaptador das agendas DoT/HoT. Remoção de buff e expiração só retiram suas contribuições. Recibos identificam a instância, origem, beneficiário, execução aplicadora e responsável por remoção/refresh.

## MCP, inspeção e replay

Autoria via set_ability_catalog/edit_scene; execução via activate_ability/aim_ability; inspeção de instâncias/recibos em inspect_abilities.buffs. Grants e agendas expõem base e valor efetivo das proteções. Novo `inspect_attribute {entity,attribute,ability?,tags?}` retorna base/efetivo/contribuições com operações, prioridade, origem, execução e prazo. Contexto ability inclui suas tags para atributos gerais; tags adicionais explícitas se somam. Para combat.damage_received.multiplier e combat.damage_dealt.multiplier, ability identifica a habilidade e apenas as tags explícitas da consulta classificam o dano. Queries desconhecidas são erros sem mutação. É uma ferramenta de inspeção, não conhecimento concedido ao NPC.

NPC mantém validação de visão atual/relação/alcance antes de consultar/aplicar efeitos externos. Projéteis já lançados continuam resolvendo colisão física sem conceder posição à memória perceptiva. Estado autorado e estado da partida continuam separados; cena/replay39, edição/undo/reset/load recompilam registry e limpam instâncias, replay compara estado e recibos. Cenas anteriores preservam defaults; replay exige versão/build compatíveis.

J/set_debug_view(npc_routes) mostra uma barra verde-lima com o prazo do buff ativo que expira primeiro no ator. Ela não representa todos os buffs nem identifica seu efeito; o MCP lista todos. Corpos invisíveis mantêm regra anterior; guia não altera conhecimento/decisão. Sem pipeline GPU novo. Teclas5/6 usam slots previamente disponíveis pelo MCP; entrada nativa tem aceite separado.

## Demo

```sh
./target/release/ignis --scene examples/npc-attributes.scene.json --window
```

P inicia/pausa; WASD move, J guias, clique seleciona, X cancela, R restaura. Jogador começa com100/200 de vida e1000 de energia.

| Tecla | Habilidade |
|---|---|
|1|Concentração por10s: seus casts não são interrompíveis e seus DoTs poison não podem ser dissipados. Também modifica craft.temperature para demonstrar chave customizada no MCP.|
|2|Por10s: imunidade a gelo e1,5× dano de fogo recebido.|
|3|Por10s: imunidade a todo dano recebido.|
|4|Cura própria canalizada, preparo0,5s,10 pulsos de10 a cada1s; custo5 na entrada, cooldown1s. Permite andar.|
|5|Veneno no selecionado:12 pulsos de5 a cada1s, custo5 e cooldown0,5s.|
|6|Dissipa até8 buffs enhancement próprios, custo5 quando houver correspondência.|

Buffs1–3 são instantâneos, custo5, reaplicação refresh, cooldown0. Roxo tenta interromper o jogador, retry0,5s/cooldown2s: use4 sozinho, depois1 e4 para comparar. A proteção expira antes do fim do canal iniciado depois dela, permitindo nova interrupção no final.

Vermelho lança fogo; azul lança gelo; ambos causam3 de dano base, preparo0,5s/cooldown1,5s e projétil6m/s. Com2, gelo causa0 e fogo causa4 (4,5 truncado para unidades inteiras). Com3, ambos causam0.6 permite remover as proteções antes do prazo.

Verde recebe veneno e usa antídoto próprio pelo executor comum: selecione-o e use5. Sem1, ele remove o veneno; com1 ativo, o veneno permanece até a proteção do autor expirar, quando o verde pode removê-lo. Roxo/vermelho/azul/verde ficam parados para isolar regras. Sem áudio/VFX específico de elementos; cores e barras são diagnóstico.

Testes Rust e scripts/run_verifications.py --only attributes; regressão completa headless/render/replay e checklist em performance.md. Não conclui todos os adaptadores/tipos de atributos, tática de buff de NPC, área, equipamentos, respawn ou M08/M09.

## Dano causado (EAF-51)

Quarto atributo nativo: `combat.damage_dealt.multiplier`, base10000, limites0..1000000, beneficiário autor da execução, tags dos componentes. Mantidos61 customizados (65 chaves totais). Consulta dinâmica no commit/impacto/pulso, junto do recebido; [contrato, compatibilidade e demo](outgoing-damage.md). Cena/replay40.
