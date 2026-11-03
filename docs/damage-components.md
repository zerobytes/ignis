# Dano composto e classificação do jogo (EAF-50)

O jogo registra suas próprias tags. A engine não conhece fogo, gelo, magia ou tipos enumerados de dano. Um único efeito pode conter quantidades independentes; cada componente consulta os modificadores aplicáveis antes de somar o resultado e alterar a vida uma vez.

## Autoria

Cena 39 exige `ability_catalog.attributes.tags` como registro explícito de até 64 nomes distintos. Tags da habilidade, dos componentes e dos filtros de buffs devem referenciar esse registro. Nomes desconhecidos e referências duplicadas são erros antes de mutar documento/partida/histórico. Não há inferência de hierarquia, exclusividade ou significado pelo nome.

```json
{
  "kind": "damage_components",
  "recipient": "target",
  "components": [
    {"amount": 40, "tags": ["fire", "magical"]},
    {"amount": 60, "tags": ["ice", "magical"]}
  ]
}
```

Cada quantidade usa a escala de vida do jogo, como o dano escalar existente. `recipient` admite source/target na entrega direta; projéteis e agendas periódicas continuam target-only. Até oito componentes de dano **somando todos os efeitos da habilidade**. Dano escalar conta como um componente. Cada efeito composto precisa de 1..8 componentes positivos, e a soma original do pacote não pode exceder 10¹² unidades. Listas de tags podem ser vazias: regras sem filtro continuam aplicáveis. Campo `amount` no nível do pacote é rejeitado; o total deriva das parcelas.

`damage {recipient, amount}` permanece como forma escalar sem classificação. Tags da habilidade não são herdadas pelo dano em cena 39. Para um dano único classificado, use um pacote de um componente. Tags da habilidade continuam sendo o contexto de interrupção/dissipação dos efeitos periódicos; tags do componente são o contexto do multiplicador de dano recebido. O filtro por ID de habilidade continua disponível nos dois contextos.

## Classificação versus composição

100 com tags `[fire, ice]` é um único componente que satisfaz os dois filtros. 40 com `[fire]` e 60 com `[ice]` são parcelas distintas. Contra redução de 50% para ice, o primeiro resulta em 50 e o segundo em 70. Uma regra sobre magical pode afetar ambas as parcelas sem tornar seus elementos iguais.

| Componente | Original | Multiplicador recebido | Ajustado |
|---|---:|---:|---:|
| fire, magical |40|10000 (1×)|40|
| ice, magical |60|5000 (0,5×)|30|
| Total |100||70|

O avaliador de atributos existente determina o multiplicador por componente. Mantém prioridades, filtros `tags_all` e composição definidos em attributes.md. Não há novo tipo nativo de elemento, deny-wins ou regra automática de exclusividade. EAF-50 integrou dano **recebido**; EAF-51 acrescenta dano **causado** no autor original, conforme outgoing-damage.md.

## Cálculo, limites e transação

Para cada componente: `adjusted = amount * dealt_multiplier * received_multiplier / 10000²`, com intermediário i128 e truncamento único em direção a zero, na unidade mínima de vida do jogo. Depois somamos em i64 e limitamos o total do efeito a10¹². Quantidade original total limitada a10¹² e ambos multiplicadores limitados a100× garantem soma ajustada <=10¹⁶, sem overflow. Com dano causado neutro, preserva a fórmula original de EAF-50. A soma independe da ordem das parcelas; a ordem dos modificadores de cada atributo continua a ordem determinística existente.

Arredondamento por componente é deliberado: duas parcelas de 1 unidade com multiplicador 0,5 resultam em 0 + 0, enquanto uma parcela de 2 resultaria em 1. Jogos que precisem dessa precisão devem escolher uma escala de vida mais fina. Não há promessa de invariância ao subdividir artificialmente uma quantidade.

Todos os componentes consultam o estado de buffs do início da transação. Aplicar/remover proteção na mesma lista de efeitos não muda suas resistências durante aquele plano. Só depois da soma há uma mudança de vida para o efeito composto, um recibo de efeito e eventual causa de morte. Componentes não geram acertos ou execuções separados. A engine ainda não oferece callbacks gerais de on-hit; este contrato evita multiplicar os eventos que já existem.

Uma habilidade com vários efeitos mantém a composição ordenada existente (por exemplo, dano seguido de cura). O agrupamento não funde automaticamente efeitos separados. O commit de vida/recursos/buffs continua único para a execução: falha em operador posterior preserva vida, custo e componentes, sem recibo parcial de dano. A limitação da vida disponível é aplicada ao total; não inventamos atribuição de overkill entre componentes.

Direto revalida no commit; projétil continua pago no lançamento e calcula contra a resistência atual da vítima física no impacto. DoT/HoT e canal mantêm seus prazos/custos; cada pulso de dano calcula um novo pacote contra a proteção atual. Percepção e elegibilidade de NPCs permanecem anteriores à aplicação.

## Inspeção, compatibilidade e performance

`inspect_abilities.receipts[].effects[]` mostra:

- `effect_index`, `requested`, `adjusted_amount`, `applied` e a mudança única de vitalidade;
- `damage_components`: índice, tags do componente, quantidade original, multiplicador e resultado ajustado;
- `damage_before_limit` e `damage_limit_excess`: soma calculada e redução pelo teto numérico, separados de `vitality.excess` (overkill);
- `damage_multiplier`: fator recebido quando há um componente; null quando há vários. Em cada componente, `received_multiplier`/`dealt_multiplier` explicitam ambos fatores; `multiplier` permanece alias do recebido.

`inspect_attribute` com `combat.damage_received.multiplier` ou `combat.damage_dealt.multiplier` usa as tags explícitas da consulta. `ability` seleciona o ID para filtros, sem injetar tags temáticas da habilidade. Para outros atributos, a consulta mantém o contexto anterior da habilidade. Inspeção não fornece conhecimento ao NPC.

Cenas anteriores passam por validação no contrato antigo. Normalização coleta o registro implícito de tags e converte dano escalar com tags de habilidade em um pacote de um componente com essas mesmas tags. Isso preserva o comportamento anterior, inclusive um único dano com várias classificações. Duplicatas semânticas antigas são normalizadas. Depois de salvar, o documento é cena 39 com registro explícito; referências novas desconhecidas passam a falhar. IDs de entidades/habilidades/buffs são preservados. Replay 39 exige build compatível; não prometemos reproduzir arquivos de builds anteriores.

Componentes ficam em pacotes compilados com quantidades e máscaras numéricas. O plano/recibo contém um único array fixo de até oito decomposições, compartilhado entre os efeitos. Evita ampliar cada um dos oito slots de efeito por mais oito componentes. Consultas não alocam nem interpretam nomes no tick; registros e JSON pertencem à compilação/inspeção. O aumento fixo do anel de recibos e o scan limitado do avaliador são custos deliberados; sem ganho de FPS alegado.

## Demo

```sh
./target/release/ignis --scene examples/npc-damage.scene.json --window
```

P inicia/pausa; clique seleciona; WASD move; J alterna guias; X cancela; R restaura. Jogador e alvos começam com 1000 de vida. Os NPCs ficam parados para permitir comparar as regras.

- Vermelho: sem proteção; ataque misto causa 100.
- Azul: aplica em si redução de 50% do gelo; ataque misto causa 70.
- Roxo: aplica em si imunidade a gelo; ataque misto causa 40.

As proteções dos NPCs duram 216000 ticks (uma hora a 60Hz), têm reaplicação refresh e cooldown de 3600 ticks. A primeira aplicação ocorre no início da simulação; dissipar permite testar sem proteção até a próxima reaplicação. Não é uma tática adaptativa de buff.

| Tecla | Habilidade no selecionado |
|---|---|
|1|Dano direto: 40 fire + 60 ice, ambos magical|
|2|Mesmo pacote por projétil a 8m/s|
|3|Mesmo pacote por pulso, três pulsos com intervalo de um segundo|
|4|Aplica a redução de 50% contra gelo|
|5|Dissipa até oito buffs da categoria resistance|
|6|Cura 500 unidades, limitada à vida máxima|

Habilidades do jogador custam 5 de energia, sem preparo/cooldown nesta comparação. Depois de três pulsos, os totais esperados são 300/210/120 nos três alvos. Evite matar os alvos para continuar comparando; cura não ressuscita, R restaura. Não há VFX/áudio elemental; as tags definem regras de gameplay, não aparência. Entrada física e aceite humano são separados da validação MCP/offscreen.

Extensão EAF-51: [dano causado](outgoing-damage.md) acrescenta o fator do autor por componente, divisão única por SCALE² e recibos dos dois fatores. Cena/replay40; dano recebido mantém seu significado.
