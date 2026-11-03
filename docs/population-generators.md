# Geradores de população — EAF-92

Continuidade EAF-108: [templates ponderados](weighted-population.md), cena65/replay69. Fonte templates alternativa ao template único, com herança por onda; comportamento de fonte única abaixo preservado.

Continuidade EAF-105: [regiões de nascimento](spawn-regions.md), cena64/replay68; fonte region alternativa a points e posição tentada em recibos. Com points, a sequência abaixo continua a mesma.

Continuidade EAF-95: [controle em execução](generator-control.md), cena62/replay66. `initially_active` e ativar/pausar/cancelar pelo MCP; contratos anteriores abaixo preservados quando ativo.

Continuidade EAF-93: [ondas finitas](population-waves.md) opcionais em `waves`, cena61/replay65. Lista vazia preserva o modo de reposição descrito abaixo; os limites históricos de EAF-92 continuam válidos.

Cena60 acrescenta `npc_generators`; replay64 regenera e compara seu estado. Geradores mantêm uma meta de indivíduos vivos provenientes de um template, com candidatos XYZ, semente, cadência e orçamento. Não são entidades físicas nem adquirem conhecimento para os NPCs.

## Contrato compartilhado

`set_npc_generators {generators:[...]}` substitui as definições, também como `SceneEdit` em `edit_scene`. Edição valida tudo antes de mutar, reinicia a execução a partir da autoria e participa de undo/redo. `inspect_generators` retorna definições, contagens atuais por gerador, membros/referências, PRNG, próximo tick elegível, tentativas, nascimentos e recibos. Ambos passam pelo dispatcher usado pelo MCP e pelos controles nativos. O editor visual dedicado de geradores não faz parte deste recorte.

```json
{
  "id": "camp",
  "template": "guard",
  "points": [[0, 0.8, 0], [3, 0.8, 0], [-3, 0.8, 0]],
  "seed": 42,
  "target_alive": 2,
  "interval_ticks": 60,
  "attempts_per_cycle": 3,
  "max_births": 20,
  "initial_delay_ticks": 0,
  "retire_dead_after_ticks": 180
}
```

Posições representam o centro do corpo, como `spawn_actor`. Omitir `retire_dead_after_ticks` (ou null) preserva cadáveres; omitir `initial_delay_ticks` começa no primeiro step. `interval_ticks` é o intervalo mínimo entre ciclos de tentativa, não um prazo individual contado desde cada morte. Ao surgir déficit depois de um período cheio, o próximo step já pode tentar se a cadência anterior venceu. `max_births` conta sucessos cumulativos, inclusive NPCs cujo template inicia morto; falhas não o consomem. Ao esgotar o orçamento, a política de retirada de cadáveres continua funcionando.

Limites:8 geradores,32 pontos por gerador,1..8 vivos desejados e1..8 tentativas por ciclo, orçamento1..36000 nascimentos. Intervalo1..36000 ticks; atraso inicial e prazo de retirada0..36000. Identificador único por gerador,1..64 caracteres ASCII alfanuméricos/`_`/`-`. A população física continua limitada a8 NPCs mais o jogador; meta não reserva capacidade. Template inválido, pontos não finitos, limites inválidos e remoção de template referenciado falham atomicamente.

## Fase, candidatos e concorrência

1. Aplicar pedidos externos de população, na ordem existente.
2. Em ordem autoral dos geradores, limpar referências ausentes, observar vida/morte e retirar somente os próprios cadáveres quando a política vencer.
3. Se houver déficit, budget e cadência vencida, escolher um início na lista de candidatos pela semente. Tentar os candidatos seguintes circularmente, sem repetir um ponto naquele ciclo, até o menor entre a quantidade de pontos e `attempts_per_cycle`. Encerrar antes ao atingir meta, budget ou capacidade de acompanhamento.
4. Executar sensores, habilidades e física normalmente.

A sequência pseudoaleatória é explícita: LCG u32 com multiplicador1664525, incremento1013904223 e aritmética wrapping; índice inicial = parte alta de `rng * quantidade_de_pontos`. Seed zero é válida. Não é distribuição uniforme em volume nem amostragem de navmesh. A sequência é reproduzível com a mesma autoria, comandos, build e ticks; outra semente pode escolher outro início, mas não garante cenário diferente se os mesmos pontos forem os únicos válidos.

Spawn usa a mesma transação de nascimento: capacidade, volume, apoio/hover, passagem e alvo antes de consumir identidade. O alvo explícito do template é vinculado ao indivíduo original na construção do runtime. Sua retirada gera `instance_unavailable`; nunca vincula um substituto silenciosamente. Templates sem alvo permanecem sem alvo explícito; seleção automática/percepção continua nos sistemas existentes.

Os comandos externos têm prioridade de capacidade. Geradores competem em ordem autoral, sem reserva/equidade implícitas. Uma origem pode monopolizar o pool; o último recibo mostra `capacity`, `occupied`, `unsupported_position` etc. Um gerador que já acompanha8 corpos, inclusive mortos, espera liberar um deles; não rouba armazenamento de outro. Não há loop buscando indefinidamente um espaço nem fallback para uma posição insegura.

## Morte, retirada e ressurreição

Cada membro guarda actor+Reference, nunca apenas slot. Reuso por um comando externo não vira membro do gerador antigo. NPCs autorados ou criados manualmente não são adotados nem retirados automaticamente. Não há transmissão global de esquecimento para os sobreviventes.

Morte é observada na fase de geradores do tick seguinte ao dano. O prazo de retirada começa nessa primeira observação. Prazo zero retira nessa fase, antes das habilidades; ressuscitar no mesmo tick da retirada não ganha prioridade. Ressurreição anterior ao prazo limpa o timer quando observada; uma morte posterior recebe um prazo novo. O membro registra também o life_index observado, para reiniciar o prazo mesmo se ressuscitar e morrer novamente entre duas verificações. Remoção usa o mesmo caminho de despawn: origem de entregas paga preservada, corpo/componentes liberados e referências antigas invalidadas.

A meta conta vivos, enquanto cadáveres ainda ocupam o pool. Havendo outro ponto livre, pode nascer um substituto antes da retirada do cadáver. Se esse cadáver ressuscitar, o número de vivos pode superar a meta: ninguém é removido para corrigir isso e novos nascimentos ficam suspensos. Um template sem vitalidade conta como vivo. A ressurreição preserva indivíduo, energia/cooldowns; a reposição produz outro indivíduo com estado inicial.

## Replay, observação e desempenho

Eventos automáticos são derivados; não ocupam o log limitado4096 de inputs externos. Replay64 salva estado esperado de membros/PRNG/contadores/recibos e o reconstrói ao executar os mesmos ticks. Prefixos anteriores ao primeiro step não contêm nascimentos futuros. Recibos separados em anel256 guardam tick, índice do gerador/ponto, resultado e Reference. Contagens consultam presença atual; os timers de morte são observações da fase anterior às habilidades.

Configuração permanece no documento; estado pertence à simulação. Reset/edição/undo recompõem ambos e invalidam tokens externos. Save de cena não salva população gerada como autoria. Nenhuma mudança no formato `RenderFrame`; captura/picking usam os indivíduos presentes existentes.

Estado por gerador é numérico e limitado; tick copia somente esse estado, sem heap/Scene/String na política. Catálogo de template e alvo é compilado na construção. Nascimento/retirada mantém as alocações esparsas de configuração e Rapier já existentes. Inspeção/replay serializa sob demanda. Até64 tentativas automáticas por tick (8×8), além dos64 pedidos externos; não é promessa de throughput ou escala601. Perfis/índices globais ainda limitam expansão da população.

## Verificação e demo

`python3 scripts/run_verifications.py --only generators` valida autoria, reposição, cadáver/ressurreição, identidade, erros, tentativas/anel, prefixos e pixels via MCP headless. Rust cobre também competição, prioridade externa, alvo original, prazo no mesmo tick, seed, budget e configuração. Executar regressão completa para integração.

Cena `examples/npc-generators.scene.json`, salva via MCP: P inicia, aparecem dois roxos imóveis; clique para selecionar e1 aplica dano letal,2 ressuscita um cadáver ainda presente. Gerador tenta manter2 vivos a cada60 ticks, retira cadáveres após180 ticks observados e permite20 nascimentos no total. A terceira posição permite reposição enquanto um cadáver permanece. R reinicia. Os NPCs imóveis isolam o comportamento da população; isso não reduz suas capacidades gerais. Nenhum teste de teclado/foco nativo é inferido da validação MCP.

Ondas, região contínua de amostragem, pesos de templates, reservas entre geradores, ativação por distância/bioma, orçamento global de processamento e aumento de escala são incrementos futuros.
