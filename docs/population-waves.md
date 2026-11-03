# Ondas de população — EAF-93

Continuidade EAF-108: [templates ponderados](weighted-population.md), cena65/replay69. Fonte templates alternativa ao template único, com herança por onda; comportamento de fonte única abaixo preservado.

Continuidade EAF-95: [controle em execução](generator-control.md), cena62/replay66. `initially_active` e ativar/pausar/cancelar pelo MCP; contratos anteriores abaixo preservados quando ativo.

Cena61/replay65 acrescentam `waves` opcional a cada gerador. A lista vazia/ausente mantém a reposição EAF-92. Uma lista não vazia descreve uma sequência finita: cada onda tem quantidade de nascimentos, template opcional, atraso inicial e condição de conclusão. Não há dependência de classe, facção hostil ou formato de combate.

## Autoria e controle compartilhado

Usar `set_npc_generators` ou a mesma operação em `edit_scene`; consultar `inspect_generators`. Configuração validada antes de alterar, com undo/redo; editar reinicia a execução a partir da autoria, conforme as outras edições. Não é uma alteração silenciosa de uma sequência em andamento.

```json
{
  "id": "encounter",
  "template": "guard",
  "points": [[0, 0.8, 0], [3, 0.8, 0], [-3, 0.8, 0]],
  "seed": 42,
  "target_alive": 2,
  "interval_ticks": 30,
  "attempts_per_cycle": 3,
  "max_births": 5,
  "retire_dead_after_ticks": 120,
  "waves": [
    {"count": 2},
    {"count": 2, "template": "elite", "delay_ticks": 120},
    {"count": 1, "template": "boss", "delay_ticks": 120}
  ]
}
```

- `count`: total de nascimentos bem-sucedidos daquela onda,1..36000. Não é quantidade simultânea nem número de mortes. Pode exceder o pool se os indivíduos anteriores forem retirados ao longo da onda.
- `template`: quando omitido/null, usa o template base do gerador. Cada onda escolhe uma configuração; misturas no mesmo grupo, pesos e sorteio de template ficam futuros. Alvos explícitos de todos os templates da sequência são vinculados aos indivíduos originais na construção do runtime.
- `delay_ticks`: padrão0, até36000; espera antes daquela onda. Na primeira, soma a `initial_delay_ticks`. Nas seguintes, conta desde o tick que confirmou a conclusão da anterior.
- `completion`: `defeated_or_removed` (padrão) confirma quando nenhum membro da onda atual está vivo/presente; `removed` exige que todos estejam ausentes do mundo. Em ambos os casos a quota de nascimentos precisa estar completa.

Até16 ondas por gerador,8 geradores,8 NPCs físicos mais jogador. A soma dos `count` precisa caber no `max_births` do gerador, evitando uma quota impossível por orçamento. Budget extra não cria ondas extras. Pontos, seed, cadência, tentativas, retirada de cadáver e validação geométrica continuam os de [population-generators.md](population-generators.md). Não há garantia de terminar dentro dos36000 ticks do laboratório, de ter apoio/rota ou de conseguir vagas: falhas ficam observáveis.

## Fases e ordenação

`delay → spawning → waiting_for_clear → delay` para a próxima onda ou `completed` na última.

A fase de geradores permanece depois de comandos externos de população e antes dos sensores/habilidades/física. Cada gerador confirma no máximo uma transição com evento por tick. A conclusão de uma onda nunca cria a seguinte no mesmo tick, mesmo com atraso0; o primeiro tick elegível posterior inicia a próxima. Ao iniciar, pode nascer mais de um NPC até os limites de tentativas/quota/população.

Durante `spawning`, o gerador tenta completar a quota, respeitando `target_alive` como limite de vivos de **todos os seus membros**, inclusive indivíduos de ondas anteriores que ressuscitaram. Matar alguém antes de terminar a quota pode permitir outros nascimentos; após a quota completa não há reposição extra naquela onda. A cadência é interna à onda; o começo de uma nova usa seu próprio prazo. A semente continua a sequência anterior, sem reinicialização por onda.

`waiting_for_clear` observa apenas membros da onda atual. `defeated_or_removed` não distingue retirada externa de morte para concluir; uma entidade sem vitalidade exige retirada. A saúde/ausência é observada na fase seguinte ao efeito que a alterou. Remoções externas no começo do tick já são visíveis; tentativas de ressuscitar em habilidade no mesmo tick de uma conclusão ocorrem depois dela.

A confirmação é definitiva. Ressuscitar alguém de uma onda já concluída não reabre a onda nem retorna o índice. O indivíduo permanece vivo e pode bloquear o limite de novos nascimentos. Ressuscitar enquanto outro membro ainda mantém a onda aberta pode impedir sua conclusão. Não remover vivos excedentes nem limpar cadáveres para fazer o roteiro avançar.

`completed` indica conclusão histórica da sequência; pode haver corpos ou indivíduos ressuscitados depois. Não cria mais NPCs, mesmo com budget sobrando. A política de retirada dos próprios cadáveres continua executando. Membros guardam Reference, life_index observado e índice da onda: reciclar o slot com um NPC externo não o incorpora ao grupo nem permite que a limpeza o retire.

## Bloqueios e inspeção

`inspect_generators` mantém as contagens gerais e acrescenta `waves`, uma entrada por gerador (null na reposição contínua). Mostra índice zero-based e ordinal humano, total de ondas, fase, nascimentos/quota, membros presentes/vivos da onda, prazo e tick de conclusão. `waiting_for` distingue atraso, condição de conclusão, limite de vivos, capacidade de acompanhamento, cadência e tentativa de candidato. É diagnóstico da fase; motivo geométrico/global da última recusa está nos recibos de população do gerador.

Exemplos de bloqueio legítimo:8 cadáveres preservados ocupam os slots; um ressuscitado antigo mantém o limite vivo; o alvo original de um template de onda futura foi retirado; todos os pontos estão ocupados ou sem apoio. Não criar indivíduos em posição inválida, roubar NPCs de outros geradores ou declarar sucesso para sair dessas situações.

Recibos de nascimento/retirada incluem `wave` (null no modo contínuo). `wave_events` tem tick, gerador, onda e tipo `started`, `cleared` ou `completed` — este último é também a confirmação final. Há até256 eventos para8×16 ondas; anel defensivo de256 com contador de descartes. O evento `cleared` identifica a onda anterior, mesmo quando o estado já aponta para a próxima em `delay`.

## Persistência, desempenho e limites

Scene61 salva apenas autoria. Replay65 reconstrói bindings, fase, quotas, seed, membros e eventos a partir dos mesmos ticks/comandos, e compara o estado esperado; eventos automáticos não enchem o log de entradas externas. Prefixos mantêm o limite temporal e comandos pendentes. Formatos61/65 foram coordenados no Kordo após a reserva gráfica59/63 e geradores60/64; a branch funcional não incorpora implicitamente as capacidades gráficas isoladas.

Templates/alvos das ondas são compilados uma vez, em vetores limitados, separados do estado Copy por gerador. Nenhum clone de lista de ondas/configuração/cena ou resolução textual por tick. A varredura existente dos membros também conta a onda corrente. Nascimento/retirada reutilizam transações e custos esparsos existentes. Inspeção agrega contagens sem vetor intermediário de membros; JSON permanece na fronteira. RenderFrame, shaders, concorrência e dependências preservados. Sem profiling ou promessa de FPS/escala601.

Próximos recortes possíveis: ativação/pausa/cancelamento em execução, condições de mundo/bioma, composições de templates por onda, amostragem de região e políticas globais de capacidade. Este incremento não adiciona quests, sistema geral de encontros, reservas/equidade, limpeza automática de sobreviventes ou reinício cíclico de ondas.

## Demo e validação

`examples/npc-waves.scene.json`, criada/salva/carregada pelo MCP: P inicia; selecione com clique e use1 para dano100,2 para ressuscitar com40vida enquanto o cadáver está presente. Duas unidades roxas, duas laranjas, uma azul com200vida; dois segundos de espera entre ondas, retirada opcional da demo após120ticks observados e orçamento5. O azul precisa de dois golpes sem cura intermediária. Não há reposição infinita; R reinicia.

NPCs imóveis isolam o teste populacional. O MCP completou as três ondas, verificou vida200→100→0 e replay. `python3 scripts/run_verifications.py --only waves` testa o contrato; regressão completa inclui geradores, população, habilidades/NPCs/render. Captura offscreen não comprova teclado/foco nativo. Resultados finais em [validation.md](validation.md).
