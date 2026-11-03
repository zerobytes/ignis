# Nascimento por templates — EAF-70

Cena47 acrescenta `npc_templates`; replay51 registra nascimento e retirada na mesma fila. O template é configuração, não um indivíduo: não recebe identidade, corpo, percepção ou saldo enquanto não for instanciado. Cada nascimento recebe um novo InstanceId, geração válida e `life_index:0`.

## API compartilhada

- `set_npc_templates {templates:[Entity,...]}` substitui o catálogo de templates. Também disponível como edição atômica em `edit_scene`; valida antes de alterar, com undo/redo único. Como as demais edições, reinicia a execução a partir da autoria.
- `inspect_templates` consulta o catálogo e seu limite.
- `spawn_actor {template:"guard",position:[0,0.8,4],target:null}` admite um pedido. `position` é o centro do corpo XYZ. O alvo opcional aceita alias presente ou token escopado de `inspect_instances`. Se omitido, usa o alvo explícito autorado no template, quando houver. Não altera seleção automática de alvos.
- `inspect_population` mostra pedidos pendentes, resultados e limites. `spawned` contém `instance_id` e o alias do novo indivíduo; `inspect_instances` fornece seu token escopado. Não há token válido de nascimento antes da aplicação.
- `despawn_actor` retira o indivíduo original. Remover e criar são operações distintas; ressuscitar preserva indivíduo/recursos/cooldowns, enquanto nascer inicializa tudo novamente.

Templates usam o contrato Entity existente, exigindo body NPC, configuração NPC, ID único e sem trigger. A posição no template é dado autoral validado; o pedido fornece a posição efetiva. Rede, superfícies, passagens, facções e definições de habilidades pertencem à cena. Bases de atributos podem usar o ID do template em `ability_catalog.attributes.bases`; cada nascimento copia apenas as bases desse template. Os prefixos `_ignis_` são reservados para slots e aliases de execução.

## Aplicação transacional

A fila é consumida sequencialmente antes dos sensores e das habilidades. Por pedido, validar referência/alvo, espaço no pool, volume AABB contra os corpos presentes e apoio integral. A tolerância de contato de0,0001m não autoriza sobreposição deliberada. Caminhantes precisam de apoio; nascimento no ar exige voo capaz de hover (reta ou curva com `can_hover:true`). Planadores e voadores sem hover não recebem impulso inicial implícito: esse contrato permanece futuro.

Participantes de passagem exigem pontos de espera suficientes para a população atual após o nascimento. A capacidade física permanece a do template; spawn não garante caminho até um destino nem descobre uma rede de navegação.

Só após essas verificações é consumida uma identidade e instalado o novo indivíduo. A fila não é uma transação de grupo: cada operação tem seu resultado e não desfaz predecessoras. Remover e depois criar pode liberar espaço; criar e depois remover pode primeiro retornar `capacity`. Dois nascimentos no mesmo volume resultam em um sucesso e um `occupied`. Um alvo retirado antes da aplicação resulta em `instance_unavailable`, sem vincular seu substituto.

Resultados possíveis: `spawned`, `despawned`, `instance_unavailable`, `capacity`, `occupied`, `unsupported_position`, `passage_capacity`, `identity_exhausted`. Erros de schema/template/posição não finita/referência inválida na admissão não entram na fila nem consomem request_id. Recusas durante aplicação geram recibos, mas não consomem identidade de nascimento.

## Estado por indivíduo e ordenação

O pool mantém até8 NPCs, além do jogador. Até16 templates compartilham esses slots; não há reserva de slots por template. NPCs inicialmente autorados também podem liberar seu armazenamento. O mundo em execução possui definições por slot e placements próprios; o documento de autoria e os templates permanecem imutáveis durante a partida. A instalação clona somente a configuração do NPC escolhido, sem reconstruir os sobreviventes.

Cada instalação inicializa vida/recursos/capacidades/grants, cooldowns/retry/interrupção, cast, amostra de movimento, percepção/reconhecimento/rádio/objetivos/território, navegação, patrulha, passagem, motor espacial, corpo físico e acumulador de passos. Buffs e entregas destinadas ao indivíduo retirado não passam ao novo ocupante. Memórias dos outros NPCs continuam pistas do indivíduo antigo, sujeitas aos budgets originais; não há broadcast de esquecimento. Uma observação nova pode ser produzida já no primeiro tick de vida.

Ordem de execução de casts/intentos, mortes/retornos e desempates de área, alvo, triagem, rádio e passagens usa InstanceId, não a posição de armazenamento reutilizada. Indivíduos iniciais preservam a ordem anterior por ID autorado; recém-nascidos recebem seriais crescentes. Históricos e comandos guardam referências congeladas: não passam a nomear ou controlar o novo ocupante.

Entregas já pagas conservam definição/contexto do autor removido, conforme [runtime-world.md](runtime-world.md). Bases e contribuições são capturadas antes de substituir a configuração; seus prazos continuam os originais. Novo saldo, facção, vida ou buff não entram no cálculo de um projétil antigo. Os contatos encerrados na retirada são congelados antes de reutilizar metadados; triggers conservam seus bindings originais.

## Reprodução e limites

Mantidos64 pedidos por tick,4096 entradas por execução e256 recibos. Mapas ativos removem entradas aposentadas; slots/componentes limitados são reutilizados. A autoria permite512 entidades, reservando espaço para o máximo8 NPCs quando há templates. Definições/templates são validados na fronteira; não há JSON na execução do nascimento.

Replay51 valida pedidos de habilidades e população no mundo existente em cada tick, antes de consumir a fila de população. Isso preserva a admissão de comandos que eram válidos antes da retirada e permite comandos para indivíduos nascidos em ticks anteriores. Prefixos incluem entradas daquele tick pendentes, sem executar o futuro. Replay continua exigindo o mesmo build.

Render, picking e inspeção consultam as definições presentes em execução. Save de cena salva autoria/templates; reset e undo recompõem a população autorada e invalidam tokens. Spawn não é edição autoral e não recebe undo isolado: use despawn ou replay/reset.

Validar com `python3 scripts/run_verifications.py --only spawn population`, testes Rust e regressão completa headless. Capturas GPU não exigem janela. Ondas, reposição automática, escolha procedural de posição, alteração de templates em execução e aumento da escala populacional são incrementos posteriores. Arrays globais e consultas por varredura existentes continuam limitando escala; este contrato não é prova de601 atores nem medição de FPS.
