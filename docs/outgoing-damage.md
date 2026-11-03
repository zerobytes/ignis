# Dano causado por componente — EAF-51

O atributo nativo `combat.damage_dealt.multiplier` pertence ao autor da execução. Complementa `combat.damage_received.multiplier`, consultado no destinatário real de cada efeito. Ambos usam números em escala10000, default10000 (neutro), limites0..1000000 (0..100×). A engine não interpreta nomes como fogo ou gelo: são tags registradas pelo jogo.

## Resolução e precisão

Para cada componente, o executor consulta os dois atributos com as tags daquele componente e a identidade da habilidade original:

`ajustado = floor(original × causado × recebido / 10000²)`

O produto usa i128 e é dividido uma única vez. Por exemplo,3 unidades ×0,5×2 =3; arredondar após o primeiro fator produziria2 incorretamente. As operações internas de composição de cada atributo mantêm seu próprio arredondamento já documentado. Parcelas diferentes continuam arredondadas individualmente.

As parcelas são somadas e limitadas a10¹² antes da única alteração de vida daquele efeito. Quantidade original total por pacote<=10¹² e dois fatores<=100× limitam a soma bruta por efeito a10¹⁶, dentro de i64. Teto numérico e overkill permanecem separados no recibo. Dano zero ainda conclui a habilidade/cobra custo; não bloqueia cura ou outros efeitos da transação. Cura não usa estes multiplicadores.

Exemplo:40fire+60ice, atacante com+25%fire e destinatário com−50%ice resulta50+30=80. Tag temática da habilidade não classifica seus componentes; dano escalar continua sem tags. Filters `tags_all` e `abilities` seguem o avaliador compartilhado, inclusive para bases, contribuições sobrepostas e prioridades.

## Momento da avaliação

Cada execução avalia os atributos vigentes no início da sua transação. Isso inclui commit direto, impacto de projétil e cada pulso periódico/canalizado. A identidade da origem é preservada, mas o fator não fica congelado no lançamento. Dois atores agindo no mesmo tick continuam em ordem determinística: não há snapshot global de todos os commits do tick.

- Ganhar um buff durante o voo pode aumentar o impacto; expirar/dissipar antes do impacto remove esse aumento.
- Pulsos podem mudar de intensidade conforme os buffs atuais. Não há novo pagamento em impactos/pulsos periódicos; canal mantém sua política de cobrança.
- Buff aplicado ou dissipado pela própria transação só afeta execuções seguintes. Todas as parcelas/efeitos veem o mesmo estado inicial de buffs; vida/recursos seguem a composição sequencial existente.
- Morte da origem remove buffs cujo beneficiário é ela. Projéteis/agendas já entregues continuam; usam suas bases remanescentes, não o antigo buff. Morte do emissor de um buff em outro beneficiário não o remove.
- No dano contra si, a mesma entidade fornece os dois atributos. É intencional aplicar ambos, pois são papéis distintos.

Escolha: avaliação dinâmica permite efeitos de enfraquecimento/poder repercutirem nas próximas entregas usando o contrato atual de buffs. Congelar ofensiva por lançamento poderia ser útil para outros jogos, mas requer política explícita e armazenamento por execução/agenda; este incremento não introduz esse modo silenciosamente. Não adiciona efeitos on-hit, crítico, armadura ou seleção tática de elementos.

## MCP, compatibilidade e limites

Autoria via `set_ability_catalog`/`edit_scene`, aplicação pelo executor comum, consulta via `inspect_attribute`; `inspect_abilities` mostra `dealt_multiplier` e `received_multiplier` por componente. `multiplier` por componente e `damage_multiplier` escalar continuam aliases do fator **recebido**, não do produto. O valor final é `adjusted`, e os totais permanecem em `adjusted_amount`/`damage_before_limit`/`damage_limit_excess`. Recibos preservam origem, destinatário, habilidade e execução.

Cena/replay40. Cenas anteriores sem colisão migram com fator causado neutro; IDs e dados customizados são preservados. Os61 atributos personalizados continuam disponíveis, agora com4 nativos (65 totais); índices u8 continuam suficientes. Se uma cena antiga declarou `combat.damage_dealt.multiplier` como chave customizada, a carga falha com diagnóstico para renomear essa chave e referências antes da atualização: ativá-la automaticamente alteraria dados antes inertes. Referências novas exigem versão40. Replays permanecem específicos da versão/build, conforme contrato existente.

Até8 componentes de dano por habilidade,64 buffs vivos e256 recibos. Mais um i64 por componente no plano/recibo; sem novas alocações, strings ou JSON no cálculo. Uma avaliação adicional por parcela/preflight é custo consciente, sem alegação de FPS.

## Demo e validação

`./target/release/ignis --scene examples/npc-outgoing.scene.json --window`

P inicia/pausa; WASD movimenta; clique seleciona; J exibe barras; X cancela; R reinicia. Não há necessidade de abrir a demo durante regressões.

| Tecla | Ação |
|---|---|
|1|Dano direto40fire+60ice no selecionado|
|2|Mesmo dano por projétil|
|3|Três pulsos do mesmo dano, separados por60ticks|
|4|Buff em si: +25%fire por600ticks (10s de simulação), reaplicação renova|
|5|Dissipa a categoria power no selecionado; selecionar o jogador para remover seu buff|
|6|Cura500 no selecionado|

| Destinatário | Sem buff ofensivo | Com buff ofensivo |
|---|---:|---:|
|Vermelho, sem proteção|100|110|
|Azul, metade do gelo|70|80|
|Roxo, imune ao gelo|40|50|

NPCs aplicam a própria proteção; não fazem nova escolha tática. Barra verde-amarela de buff indica duração, e barras de vida exibem o resultado; não há VFX elemental novo. Tempo pausado não consome buff.

`scripts/run_verifications.py --only damage outgoing` cobre contrato MCP, fatores, matriz, dissipação, expiração exata, projétil/pulsos, erros atômicos/undo/redo, autoria/save/load/reset e replay/prefixo/capturas idênticas. Testes Rust cobrem precisão/limites/legado, autor morto, filtros, executor NPC e snapshot transacional. Resultados finais em validation.md; entrada física/aceite humano são gates separados.
