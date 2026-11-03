# Entrega por projétil (EAF-42)

Extensão EAF-46: [dissipação](dispel.md) também pode ser entregue por projétil, selecionando agendas da vítima física no impacto; falha não reembolsa o lançamento.

Extensão EAF-43: [mira livre](free-aim.md) adiciona seleção espacial por direção/ponto. O contrato abaixo descreve a seleção original por ator; sweep, impacto e limites continuam compartilhados.

O lançamento e o impacto agora são eventos distintos. Habilidades continuam sendo definições de operações, compartilhadas entre jogador e NPC; `delivery` escolhe como seus efeitos chegam ao destinatário. `direct` preserva a resolução imediata no fim do cast. `projectile` lança um volume em trajetória reta XYZ, paga custos/inicia cooldown e resolve o efeito somente no primeiro contato físico elegível.

## Autoria e limites

Em `ability_catalog.abilities[]`, pelo mesmo `set_ability_catalog`/`edit_scene`:

```json
"delivery": {
  "kind": "projectile",
  "speed": 6,
  "half_extent": 0.14,
  "max_distance": 18
}
```

- `speed`:0,1..100 metros por segundo; `half_extent`:0,01..1m, metade da aresta de uma caixa alinhada aos eixos; `max_distance`:0,1..100m. Todos finitos. Não é uma esfera nem uma malha orientada.
- Ausente ou `{"kind":"direct"}` mantém o comportamento anterior e é omitido na gravação. Projétil exige cena30; replay30 mantém exigência de mesmo build/runtime/plataforma. Não há migração de gravações antigas.
- Neste recorte, seleção `actor`, diferente do emissor, e efeitos `damage`, `heal` ou `resource`, todos com `recipient: target`. A mesma entrega pode causar dano ou curar; não há enum de magias nomeadas. Efeitos na origem e ruído exigirão fases/eventos próprios e são rejeitados nessa entrega, evitando semântica ambígua quando o emissor morrer antes do impacto.
- Alcance da habilidade controla autorização do lançamento; distância máxima controla viagem. Podem ser diferentes: selecionar algo além da distância de viagem não garante acerto. Até64 projéteis ativos no mundo,256 recibos de entrega em anel; limites atuais do laboratório.

O catálogo pode ser produzido proceduralmente pelo MCP dentro desses parâmetros. Compila-se uma vez na edição/carga. Editar/undo/reset reconstroem a simulação inteira como anteriormente; não se misturam definições novas com disparos antigos em voo.

## Cast, custo e impacto

No início e no fim do cast, validar alvo, visão/relação/alcance/oclusão, saldo, componentes necessários e capacidade do pool. NPC só mira em ator confirmado visualmente no tick; memória, som e rádio não fornecem uma posição atual para lançar. Posição coincidente gera `invalid_aim`. Slot disponível no início não é reserva para o fim: dois casts podem disputar o último slot, e o segundo falhar sem pagar.

No commit, cobrar os custos uma vez, iniciar cooldown e emitir `launched`. A direção é calculada da origem para a posição observada naquele instante e congelada. Cast termina e libera bloqueio de locomoção; múltiplos disparos do mesmo ator podem coexistir conforme cooldown. Cancelar antes do commit não paga. Cancelar depois retorna ausência de execução ativa e não recolhe o projétil. Morte do emissor também não apaga o disparo lançado.

No impacto, usar o estado atual do ator realmente atingido e o mesmo plano transacional dos efeitos diretos, sem cobrar novamente. Exemplo: uma cura em viagem encontra a vida atual, não restaura a cópia do instante de lançamento. Alvo morto não é curado/ressuscitado. Se faltar um componente para qualquer efeito, nenhum efeito daquele impacto é aplicado. Falha, expiração ou bloqueio após lançamento não reembolsam energia/cooldown.

Uma morte por impacto interrompe casts e os motores/reservas da vítima pelo fluxo existente. Este recorte não adiciona impulso físico, eventos encadeados, roubo de vida ou respawn.

## Colisão e ordem

Projéteis são volumes de gameplay, sem corpo rígido Rapier. A cada passo, o segmento inteiro é varrido contra caixas físicas expandidas pelo `half_extent`; escolhe-se o menor tempo de contato, com empate exato por ID da entidade. Isso impede atravessar uma parede fina parada mesmo quando o deslocamento do projétil no tick é maior que sua espessura.

O emissor é ignorado durante todo o voo. Os demais corpos bloqueiam, incluindo cadáveres e corpos invisíveis. `visible` e `blocks_vision` não controlam colisão. Objetos visuais sem corpo não bloqueiam. O primeiro ator elegível recebe os efeitos, mesmo se não era o alvo pretendido; relação é conferida nesse ator. Aliado que não pode receber uma habilidade hostil ainda a bloqueia. Não há penetração, colisão entre projéteis, ricochete, explosão, gravidade ou homing.

**Limite dinâmico:** caixas dos outros corpos usam o snapshot atual, antes da integração física; não é CCD relativo entre duas trajetórias móveis. Um ator que atravesse inteiramente o trajeto entre dois snapshots pode não ser detectado. O projétil tampouco empurra corpos; o sweep produz somente o resultado de entrega.

Ordem no tick: percepção e intenções usam o estado anterior; cancelamentos explícitos são processados; projéteis já lançados avançam em ordem estável de lançamento; seguem interrupções por movimento, casts vencidos e solicitações novas; motores/física vêm depois. Um disparo recém-lançado avança a partir do passo seguinte. Empates temporais entre disparos usam ordem de lançamento, não uma ordenação global de frações de impacto. Efeitos revalidam vida entre impactos. Percepção não recebe a posição escondida de um alvo por existir um projétil.

## Inspeção, replay e apresentação

`inspect_abilities` e o snapshot `inspect` publicam `projectiles.active` com ID da execução, ator, alvo inicialmente pretendido, slot, tick de lançamento, posição, direção congelada, velocidade, extensão e distância restante. `projectiles.receipts` registra entidade atingida, posição/tick e `hit`, `blocked`, `relation_denied`, `target_dead`, `effect_failed` ou `expired`; contador informa descarte de recibos antigos.

Recibos de habilidade `launched` contêm custos e nenhum efeito; `impacted` contém efeitos e nenhum novo custo. `effect_failed` também tem o motivo específico no recibo do executor. `projectile_budget` informa falta de capacidade antes de cobrar. Registro em anel é diagnóstico limitado, não armazenamento histórico ilimitado.

Projéteis aparecem como pequenos cubos amarelos, com posição e tamanho iguais aos usados no sweep. São visíveis independentemente das guias J; estilo visual é provisório. Renderização usa os buffers/cache existentes. Prefixos de replay reconstroem voos ativos e pixels; não inserem impactos gravados como novos inputs. Nenhuma API ou controle depende da janela.

## Demonstração manual

```sh
./target/release/ignis --scene examples/npc-projectiles.scene.json
```

P inicia/pausa, WASD move, J mostra barras, clique seleciona, X cancela cast e R reinicia.

| Ator/slot | Configuração e comportamento |
|---|---|
| Vermelho | Atirador parado; observa jogador em360°/18m. Cast1s com bloqueio, projétil5m/s, dano10, cooldown2s, retry0,5s. Atira enquanto o vê; não persegue nem prevê a posição futura. |
| Jogador — 1 | Projétil de dano20, cast0,75s livre, velocidade6m/s, cooldown1,5s. Selecione roxo ou vermelho. |
| Jogador — 2 | Cura própria direta20, cast0,5s bloqueando caminhada, cooldown2s. |
| Jogador — 3 | Projétil de cura20, cast0,75s livre, velocidade6m/s, cooldown1s. Selecione roxo para comparar cura/dano em viagem. |
| Roxo | Alvo passivo com60/100 de vida; não ataca nem se move. |
| Parede verde | Bloqueia visão e projéteis. Use como cobertura; o atirador não lança através dela. |

Todos os disparos da demo têm meia extensão0,14m, distância máxima18m e custo5 de energia; jogador/vermelho começam com1000. Jogador começa com100 de vida. Para testar desvio, espere o cubo sair do vermelho e então ande lateralmente: a trajetória continua reta. Para testar bloqueio em viagem, lance ou espere um disparo e passe para trás da cobertura. Esconder-se depois do lançamento não apaga o cubo; ele pode atingir a parede. Alcance nominal/visão não garantem acerto físico.

Regressão: `python3 scripts/run_verifications.py --only projectiles`; suíte completa inclui render/replay/cast/NPC. Captura GPU offscreen e MCP não comprovam teclado nativo; o aceite da demo é separado.
