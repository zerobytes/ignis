# Habilidades diretas compartilhadas (EAF-38)

O recorte A do estudo EAF-37 foi autorizado para implementação. O mesmo executor atende jogador, objetivos de NPC e comandos externos. O catálogo contém operações tipadas de ajuste de recurso e emissão de ruído; nenhum comportamento depende do nome da habilidade. Dano, cura e morte foram acrescentados em EAF-39, com [contrato de vitalidade](vitality.md) independente. Entrega por projétil retilíneo foi acrescentada em EAF-42, com [contrato de lançamento/impacto](projectiles.md). Mira livre por direção/ponto foi acrescentada em EAF-43, com [contrato espacial](free-aim.md); EAF-44 acrescenta [mira automática por observação](npc-aim.md) em política opt-in. Efeitos periódicos e canalização têm os recortes descritos abaixo; ABI de extensão permanece aberta.

## Autoria e comandos

Cena26 acrescenta `ability_catalog: {resources, abilities}` e `entity.abilities: {resources, grants}`. Cenas antigas continuam carregando sem habilidades; novas propriedades exigem26. IDs têm1..80 caracteres ASCII alfanuméricos, `_`, `-` ou `.`. Um recurso declara `id` e `scale` de1..1.000.000; um pool declara `resource`, `initial` e `capacity`, em unidades inteiras. Saldo/capacidade/custo/delta têm magnitude máxima10^12. `scale:1000` faz1250 unidades representarem1,25 para apresentação; o executor nunca converte ou arredonda valores decimais.

Uma definição declara:

- `id`, `selection: self_actor | actor | spatial` (spatial exige cena31/projétil), `range` XYZ em metros0..100 (0 para spatial) e `relation: allied | neutral | hostile | null`.
- `cast_ticks` e `cooldown_ticks`:0..3600, a60 passos/s. Cooldown0 ainda permite somente uma nova execução por ator/tick.
- `costs`: até8 pares `{resource, units}`; custos repetidos são somados antes de comparar saldos.
- `effects`:1..8 operações, em ordem autorada. `resource` recebe `recipient: source | target`, `resource` e `delta`; `noise` recebe `radius`0,1..100m e emite na origem da execução.
- `delivery`: `direct` por padrão ou `projectile` com velocidade, meia extensão da caixa e distância máxima. Cena30; detalhes/limites em [projectiles.md](projectiles.md).
- `movement`: opcional, `free` (padrão), `interrupt` com tolerância de velocidade, ou `lock`. Contrato de preparação/física e demo em [cast-movement.md](cast-movement.md); modos restritivos exigem cena29.
- `requires_below`: opcional `{resource, units}`, limiar estrito sobre um recurso próprio. É uma pré-condição, não uma consulta ao recurso do alvo. A ausência não restringe o saldo além dos custos.

`grants` é uma lista ordenada de IDs de definição. Slots externos começam em1. Há até64 definições,32 recursos no mundo,16 pools e16 grants por ator, respeitando o limite atual de9 atores/8 NPCs. Essas são capacidades limitadas do laboratório, não limites pretendidos para a engine final.

| Comando MCP/dispatcher | Semântica |
|---|---|
| `set_ability_catalog {catalog}` | Substitui catálogo, valida referências, reinicia simulação e participa do undo. |
| `edit_scene {edits}` | Aceita `set_ability_catalog` junto de `upsert`, validando o estado final inteiro. |
| `aim_ability {actor, slot, aim}` / `aim_ability_at {actor, slot, x, y, width, height, plane_y}` | Mira livre XYZ ou cursor em plano explícito; mesmo executor, somente spatial. |
| `activate_ability {actor, slot, target?}` | Enfileira para o tick atual ainda não processado; retorna `request_id`, `tick`, `queued`. |
| `cancel_ability {actor, execution_id}` | Cancela somente a instância identificada; não cancela uma futura ativação no mesmo slot. |
| `inspect_abilities`, `inspect_resources` | Mesmo relatório: pools/saldos/capacidades/escala, grants/prazos, casts, recibos, contadores e entradas pendentes. |
| `step`, `save_replay`, `inspect_replay`, `capture_replay` | Executam/reconstroem o mesmo runtime, inclusive entradas ainda pendentes. |

Alvo `actor` exige ID de ator conhecido e pode selecionar a origem; a relação autorada também é aplicada nesse caso. `self_actor` aceita omissão ou o próprio ID e não avalia relação/distância/oclusão consigo mesmo. Em outro ator, alcance e segmento centro-a-centro contra bloqueadores visuais são obrigatórios. Para uma origem NPC, visão confirmada daquele alvo no tick atual é verificada antes de consultar sua posição. Rádio, som e memória antiga não autorizam rastrear um alvo oculto. NPC desabilitado não executa habilidades.

## Ordem e atomicidade

A cada passo: sensores consomem pistas anteriores e escolhem objetivos; executor reúne intenções; cancelamentos explícitos são processados; projéteis existentes avançam e resolvem impactos; interrupções por movimento são processadas; casts existentes cancelam por perda de objetivo ou vencem; novas solicitações são processadas; seguem motores/física. Em cada fase a ordem é lexical por ID de ator; solicitações do mesmo ator preservam sequência. Solicitação externa prevalece sobre a intenção automática daquele tick, inclusive se rejeitada. Um cancelamento ocupa a ação externa do ator naquele tick; uma ativação simultânea retorna `busy`. Instância com alvo congelado por ID não troca silenciosamente de destinatário.

Início e término revalidam condições, saldo, seleção, visão/alcance/oclusão e orçamento de efeitos. Um plano temporário calcula todos os saldos e efeitos; somente um plano integralmente válido muda estado. Efeitos da própria habilidade não financiam seus custos. Efeitos de outros atores já concluídos podem mudar a elegibilidade no mesmo passo. Receber comando significa estar enfileirado, não executado.

Custos e cooldown são aplicados no commit. Cancelamento ou falha não os consomem. Cast iniciado em t com duração30 conclui em t+30; um cancelamento recebido antes de processar esse passo vence o commit. Cancelar novamente/depois do término retorna `no_active_execution`. Efeitos de recurso saturam entre0 e capacidade; recibos distinguem `requested` de `applied`, inclusive ganho zero num pool cheio. Um pool cheio não torna a habilidade inválida: use condições explícitas quando necessário. Uma instância por ator; cooldown por grant.

Recibos incluem início, sucesso, cancelamento e falhas de gameplay: `insufficient_resource`, `condition_failed`, `cooldown`, `busy`, `target_unavailable`, `actor_unavailable`, `out_of_range`, `occluded`, `relation_denied`, `missing_resource`, `noise_budget`, `no_active_execution`. Erros de autoria/fila são erros atômicos de comando, distintos desses resultados. Anel mundial de256 recibos com contador de sobrescrita; não é um histórico ilimitado.

Ruídos de habilidades usam namespace de IDs próprio,16 eventos/tick e histórico visual128. Saturar orçamento recusa o plano inteiro antes do débito. O som é consumido no próximo tick, não identifica o emissor para o ouvinte e ignora autoemissão. Não reproduz áudio nos alto-falantes. O replay regenera ruídos a partir das ativações; não os duplica como entradas manuais. Há no máximo16 entradas externas/tick e4096 por gravação de até36.000 passos.

## Decisão de NPC e locomoção

Objetivo `action: {kind: use_ability, slot, retry_ticks}` participa das prioridades e condições existentes. Cooldown, retry e condição de recurso próprio removem sua elegibilidade; durante um cast automático o slot continua elegível para não se autocancelar. Outro objetivo prioritário, território ou perda da condição cancela a instância. `retry_ticks:1..3600` limita novas tentativas; falha/cancelamento de cast reinicia esse prazo. O seletor reúne intenções antes dos commits e não lê saldos de terceiros. Sem alvo visível apropriado a tentativa falha com recibo e retry.

Usar habilidade não congela um salto/voo já comprometido nem teleporta o ator. Sem política explícita não há perseguição para entrar no alcance. EAF-40 acrescenta `use_ability.policy` opcional, com limiar vital, custo/alcance e aproximação configurável; ver [decisão de habilidades](npc-ability-policy.md). O comportamento descrito nesta demo permanece legado.

## Demonstração

`examples/npc-abilities.scene.json` é autorada pelo MCP em `scripts/verify_abilities.py`. P inicia/pausa, R reinicia, J mostra barras de recurso e cast, V visão, H audição/ruído. Azul da barra é energia; verde é foco; amarelo é progresso do cast. Valores iniciais100 energia/0 foco, capacidades100. Sem regeneração automática; R restaura os recursos.

- Jogador laranja: **1** converte20 energia em30 foco, cast1,5s; **2** concede25 foco ao ator selecionado por clique, custo15 energia, cast1s e alcance8m; **3** emite sinal de raio12m, custo5 energia, instantâneo. **X** cancela o cast atual. Cooldown2s após sucesso em cada slot.
- Roxo: usa a mesma operação de conversão com condição própria `focus < 60`; para após duas conversões e espera. Não persegue ninguém.
- Azul: concede25 foco ao jogador enquanto consegue vê-lo, com o mesmo executor de apoio. Fica no posto; visão150°/10m e habilidade8m. Olhar não garante alcance; perder a visão interrompe o cast. Pode gastar energia mesmo se o foco do jogador já estiver cheio, conforme a política descrita.
- Cinza: não tem alvo visual; investiga o sinal de3 ou o ruído manual deN quando dentro do raio12m. Não há passos automáticos nesta cena. N é emissão de diagnóstico independente do custo/cooldown da habilidade3.

`python3 scripts/run_verifications.py --only abilities` valida autoria, decisões, recursos, cancelamento no prazo, som, undo/reset/load, replay final/prefixo/pendente e capturas fora de janela. Atalhos físicos e aceite da experiência permanecem gates humanos separados. As cenas antigas continuam acessíveis; testes não abrem nem fecham a sessão visual do usuário.

Extensão EAF-45: [efeitos periódicos](periodic-effects.md) instala agendas target-only com custo único, atraso inicial e refresh/independent via entrega direta ou projétil.

Extensão EAF-46: [dissipação](dispel.md), operador transacional por categoria/limite/require_match, direto ou por projétil, sem nova cobrança no impacto.

Proteção contra dissipação: `dispellable:false` na definição periódica preserva categoria e impede remoção pelo operador; padrão true. Contrato e limites em [dispel.md](dispel.md).

Extensão EAF-47: [canalização](channeling.md), fase sustentada após preparação, cobrança on_start/per_pulse, interrupção contínua e barra ciano; mesmo executor/MCP.

Extensão EAF-48: [interrupção externa](interrupts.md), efeito target-only direto/projétil, require_active e proteção interruptible; preserva pagamentos/efeitos anteriores e usa o mesmo executor/MCP.

Extensão EAF-49: [atributos e buffs](attributes.md), registro aberto por chave, operações tipadas, contexto por tags/habilidade e contribuições temporárias. Apply_buff direto/projétil/pulso de canal, proteções dinâmicas e dano condicionado; bases imutáveis, autoria/inspeção MCP.

Dano composto, registro explícito de tags e migração para cena39: [contrato](damage-components.md).

Extensão EAF-51: [dano causado](outgoing-damage.md), fator por componente no autor original e consulta dinâmica de buffs no impacto/pulso; executor compartilhado.

## Área instantânea

EAF-52 acrescenta `area` opcional (cena/replay41), autoria pelo mesmo catálogo, Point direto via aim_ability/aim_ability_at e centro de impacto para projéteis. Efeitos target-only, custo único, transação por destinatário e inspeção em `areas`/`area_applied`. Ver [contrato e limites](instant-area.md); não é agenda periódica ou área persistente.

Extensão EAF-53: [áreas persistentes](persistent-area.md), zonas fixas com pulsos/ciclo de vida e ocupantes atuais, custo único e política de morte.

Extensão EAF-54: [auras pulsantes](auras.md), anchor source em área persistente direta/self_actor. Centro segue física XYZ, recibo preserva ponto pré-física do pulso e guia/inspeção mostram centro atual. Mesmo executor para jogador/NPC.

Extensão EAF-58: [ressurreição](resurrection.md), efeito direto no mesmo indivíduo, referências por vida e saldo/cooldowns preservados; MCP e replay46.
