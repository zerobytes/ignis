# Dissipação de efeitos periódicos

Dissipar remove instâncias já aplicadas. Não desfaz dano anterior, não cancela uma preparação nem recolhe um projétil em voo. A operação compartilha executor, custos, alcance, percepção, cancelamento e entrega com as demais habilidades.

## Contrato

A definição que instala um efeito periódico pode declarar `periodic_category: "poison"`, ao lado de `periodic`. A categoria é um nome do jogo, não um enum de venenos embutidos. Ausência conserva o legado e torna a agenda inacessível a este operador de dissipação. Há uma categoria por definição neste recorte; nomes de1..80 caracteres ASCII alfanuméricos ou `_-.`, até64 categorias distintas entre aplicações e dissipadores. Categoria requer agenda e cena34; formatos antigos sem a extensão permanecem válidos.

Uma habilidade direta ou por projétil pode conter:

```json
{"kind":"dispel","recipient":"target","category":"poison","max_instances":2,"require_match":true}
```

- `recipient` pode ser source ou target na entrega direta; projétil exige target, como seus outros efeitos. Self_actor resolve target como o próprio emissor.
- `max_instances`:1..64. Remove até essa quantidade de instâncias dissipáveis da categoria no destinatário, independentemente da origem/definição que as instalou.
- `require_match`: true exige ao menos uma instância compatível, não a quantidade máxima inteira. Sem correspondência, falha com no_matching_effect. False permite resultado zero e consumo normal de energia/cooldown.
- Seleção usa a ordem estável da agenda (instalação; refresh mantém sua posição), ignorando instâncias já selecionadas por operadores anteriores na mesma transação. Não há sorteio ou ordenação por dano/tempo restante.
- Um efeito periódico não pode conter dispel: evita mutação reentrante da agenda e mantém o recorte de pulsos em dano/cura/recursos. Uma habilidade imediata pode combinar dissipação, cura, dano e recursos; se qualquer operador falhar, não remove nada, não aplica outros efeitos nem cobra na entrega direta.

Escolha: classificação única e máscara de remoção no plano atual, em vez de catálogo geral de buffs/tags ou comando especial de antídoto. Nomes são compilados em índices u8; nenhuma comparação de string no caminho de dissipação. Isso permite categorias arbitrárias e dados gerados por MCP, mas múltiplas tags, resistência probabilística, restrição por origem, roubo de efeitos e ordem por severidade são contratos futuros.


### Proteção explícita (cena/replay35)

`dispellable` pertence à definição que instala a agenda, ao lado de `periodic` e `periodic_category`. O padrão é `true`, omitido ao salvar; `false` exige agenda periódica e cena35. A categoria permanece independente:

```json
{"periodic_category":"poison","dispellable":false,"periodic":{"interval_ticks":60,"pulses":4,"reapply":"refresh"}}
```

Fragmento de definição: os demais campos da habilidade continuam obrigatórios. A proteção vale para todas as instâncias dessa definição, incluindo reaplicações. Para venenos comuns e protegidos da mesma categoria, criar definições distintas. Não há override por aplicação, dissipador ou NPC.

O seletor ignora protegidos antes de contar `max_instances` ou satisfazer `require_match`. Com apenas protegidos, `require_match:true` produz `no_matching_effect` sem custo/cooldown na entrega direta; false permite sucesso vazio pago. Um projétil já lançado continua pago e revalida na vítima real. Múltiplos operadores mantêm atomicidade e não podem remover protegidos para satisfazer a transação.

Proteção não muda pulsos/duração/refresh nem impede limpeza por morte, reset ou edição. Não é invulnerabilidade, resistência probabilística ou prevenção da aplicação de venenos. Ausência de categoria ainda torna a agenda inacessível a este operador, mesmo com dispellable true.

Schema de `set_ability_catalog`/`edit_scene`, catálogo persistido e `inspect_abilities` expõem a propriedade; grants e agendas ativas mostram o valor efetivo. NPC/jogador usam o mesmo filtro e retry/percepção existentes. Replay35 registra o catálogo e reproduz a decisão; cenas antigas continuam aceitas e normalizadas, replays seguem exigindo versão/build compatíveis.

## Tempo, entrega e conhecimento

Validação ao iniciar não reserva instâncias. No fim do cast, reconstrói o plano com a agenda atual: se o veneno expirou ou outro curador o removeu, require_match pode falhar sem cobrança. Não há remoção no início da preparação. Cancelar antes do commit mantém os efeitos.

Projétil paga no lançamento e resolve a dissipação na vítima física real no impacto, consultando a agenda daquele instante. Pode interceptar outro ator elegível; se o efeito acabou durante a viagem ou não existe no interceptor, falha sem reembolso. Um tiro espacial não precisa conhecer previamente a vítima. Efeitos requeridos no alvo autorado são pré-validados no lançamento por ator, como os demais efeitos dessa entrega.

Ordem por tick preservada: cancelamentos explícitos, pulsos já devidos, impactos, casts vencidos, pedidos novos, limpeza de morte. Dispel no tick de um pulso não desfaz o pulso já ocorrido. Cura e remoção na mesma transação são atômicas; morte é confirmada ao final da lista. Remoções explícitas recebem motivo dispelled, inclusive se a mesma transação também mata o destinatário; demais agendas encerram por target_dead.

NPC pode usar antídoto próprio e tentar novamente pelo retry existente. Require_match impede cobrança e início do cast quando não há efeito; não adiciona um novo seletor sensorial. Para atuar em outro ator, mantém visão confirmada/alcance/relação antes de consultar sua agenda. Pista sonora, rádio e memória antiga não autorizam essa consulta. Esse recorte não adiciona triagem de aliados, leitura remota de estados ou um sistema de reconhecimento de doenças. Como a consulta de vida observada existente, permite consultar efeitos após confirmação visual; regras de ocultação de status requerem extensão própria.

## Inspeção e apresentação

`inspect_abilities.periodic.active` inclui category e dispellable. Cada efeito aplicado de dispel informa categoria, limite pedido e número removido. Recibos de ciclo de vida das instâncias removidas têm outcome dispelled e removed_by com ator/ID da execução responsável; continuam identificando origem/destinatário/execução do efeito original. Falhas deixam o estado intacto e têm motivo no executor. Todos os anéis continuam limitados256.

A faixa lilás existente some quando a última agenda daquele ator termina ou é removida. Não indica categoria nem distingue veneno de regeneração. J/set_debug_view controla o mesmo overlay pelo dispatcher; nenhuma nova interação exclusivamente visual. Replay registra as ordens originais e reconstrói remoções, agendas e pixels; edição/reset/undo/load seguem reconstruindo estado.

## Demo

```sh
./target/release/ignis --scene examples/npc-dispel.scene.json --window
```

P inicia, WASD move, J mostra barras/faixa de duração, X cancela cast, R reinicia. Clique seleciona.

- **1:** veneno no selecionado,5 de dano/s por6 pulsos, reaplicações independentes.
- **2:** antídoto próprio, remove até2 instâncias poison, exige correspondência.
- **3:** antídoto no selecionado, remove até2 instâncias poison, exige correspondência.
- **4:** regeneração no selecionado,5 de cura/s por6 pulsos, categoria regeneration e reaplicação independente. Antídoto não remove essa categoria.
- **Roxo:** alvo passivo; permite comparar duas instâncias removidas com outras que permanecem.
- **Azul:** usa antídoto próprio automaticamente, com preparação1,5s, cooldown3s e retry0,5s. Envenene-o para observar a reação.
- **Vermelho:** lança veneno no jogador observado, preparando1s, cooldown8s, projétil6m/s. O veneno entregue continua se você sair da visão; use2 depois do impacto para confirmar que o veneno protegido permanece.

Habilidades do jogador preparam0,5s, cooldown0,5s e custam5 de energia; NPCs também pagam5 por execução bem-sucedida. O veneno lançado pelo vermelho usa `dispellable:false`: o antídoto próprio (2) não o remove nem cobra quando só houver esse veneno. O veneno do jogador (1) continua removível, inclusive pelo NPC azul. Antídoto não cura dano anterior. Use4 para recuperar o alvo e comparar categorias. A faixa pode continuar após antídoto se restarem outros venenos ou regeneração.

Regressão: `python3 scripts/run_verifications.py --only dispel` e testes `tests/dispel.rs`, além da suíte headless completa/Rust/clippy. Aceite manual de controles e comportamento permanece separado da automação.
