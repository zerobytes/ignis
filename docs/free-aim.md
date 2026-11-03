# Mira livre — EAF-43

Cena/replay31. Uma habilidade com `selection: spatial` e `delivery.kind: projectile` aceita uma direção ou um ponto XYZ, sem ator selecionado. `actor` e `self_actor` preservam os comandos e as regras anteriores. Os três modos usam a mesma fila, cast, movimento, custo, cooldown e executor de efeitos.

## Contrato e escolhas

- `aim_ability { actor, slot, aim: {kind: direction, vector: [x,y,z]} }`: vetor em coordenadas do mundo, congelado no pedido e normalizado no lançamento. Magnitude não altera velocidade.
- `aim_ability { actor, slot, aim: {kind: point, position: [x,y,z]} }`: ponto do mundo congelado no pedido; direção calculada da posição atual do emissor quando o cast termina. Mover durante o cast pode alterar essa direção. O ponto não contém identidade, não acompanha um personagem e não determina o fim do voo.
- `aim_ability_at { actor, slot, x, y, width, height, plane_y }`: desprojeta o cursor da câmera atual sobre um plano horizontal explícito, grava o ponto resultante e devolve `aim_point`. Coordenadas físicas do viewport, origem superior esquerda. Não seleciona entidades nem modifica câmera/documento. Erros de viewport, plano paralelo/atrás da câmera e coordenadas fora dos limites não enfileiram nada.
- `activate_ability` continua exclusivo de seleção por ator/self. Separar os comandos mantém compatibilidade dos clientes, evita combinações ambíguas de target/direction/point e permite schema derivado do dispatcher. Em runtime, uma entrada numérica `Aim` entra no mesmo executor; não há um segundo sistema de habilidades.

Componentes de direção/ponto devem ser finitos e estar em±10000; comprimento da direção deve ser≥0,001. Ponto a menos de1mm da origem falha como `invalid_aim`, no início ou na conclusão, sem custo/cooldown. Na seleção espacial, `range` deve ser0: ele representa aquisição de ator nos outros modos. O alcance físico continua em `delivery.max_distance`, inclusive quando a referência de mira está mais longe/perto. O projétil sai do centro atual do corpo; não há socket de arma neste recorte.

## Lançamento e impacto

Sem vítima definida, lançamento valida disponibilidade/vida do emissor, movimento, condição própria, recursos, mira e capacidade do pool. Custos/cooldown são aplicados apenas no lançamento. Não exige linha de visão para o ponto: atirar contra uma parede é permitido e a colisão bloqueará o voo.

Somente no primeiro contato se avaliam facção, vida e componentes da vítima. Efeitos de alvo (dano/cura/recurso) usam o plano transacional existente; fonte sem vida pode disparar uma habilidade de dano. Corpo sem componente necessário bloqueia e produz falha de efeito, sem reembolso. Não há busca de outra vítima depois do contato. Relação de facção filtra o efeito, nunca torna um corpo transparente. Tiro no vazio também consome energia.

Configuração `movement` permanece por habilidade: livre, interromper ao mover ou bloquear locomoção. Cancelar antes do lançamento é gratuito; após lançar não recolhe o disparo. AABB retilínea, limites64/256, colisão por snapshot e demais limites de [projéteis](projectiles.md) permanecem.

## NPCs e conhecimento

O comando espacial para NPC é uma **ordem explícita de mira**, equivalente a fornecer um ponto de interesse: pode mirar no vazio ou numa pista lembrada sem adquirir identidade/conhecimento. Não resolve ID de alvo nem consulta sua posição. Uma camada de decisão deve escolher o ponto com base no conhecimento disponível.

Desde EAF-44, `use_ability` automático aceita spatial com a política opt-in [observed_point](npc-aim.md); sem essa configuração continua sendo erro de autoria. Pistas anônimas, antecipação de movimento e tática permanecem contratos próprios. O modo por ator, inclusive quando comandado externamente, mantém exigência de percepção confirmada para NPCs. Mira espacial não altera memória/sensores.

## Inspeção e replay

`inspect_abilities` mostra seleção por grant, mira em entradas pendentes/cast/voo, `target`/`intended_target: null` para tiro livre e direção efetiva do voo. Recibos distinguem lançamento/impacto/falha. Log guarda somente direção/ponto XYZ resolvido, sem precisar da câmera ou viewport original para executar novamente. Replay é verificado no mesmo build; formatos anteriores de cena migram e replays antigos continuam sujeitos ao contrato de versão/build.

Mira é um enum Copy, sem strings/coleções por disparo; projeção ocorre só no comando. Ponto de referência não gera entidade. Autoria procedural usa os mesmos parâmetros validados; não existe uma dependência nova ou raycast por frame para acompanhar o mouse.

## Demonstração manual

```sh
./target/release/ignis --scene examples/free-aim.scene.json --window
```

- **P** inicia; **WASD** move; **J** exibe barras, **R** restaura a cena e recursos.
- **1** dispara onde o cursor estava ao apertar a tecla, sem clicar/selecionar: cast0,5s, dano20, velocidade6m/s, alcance18m, custo5 e cooldown0,5s. O plano de mira usa a altura atual do centro do jogador; aponte ao centro do corpo roxo, não aos pés. Cursor fora da janela não gera comando.
- **2** cura própria20, cast0,5s bloqueando locomoção, custo5/cooldown2s.
- Clique no **roxo** e use **3** para comparar com o projétil por alvo: cast0,75s, dano20, velocidade6m/s, alcance18m, custo5/cooldown1,5s.
- **X** cancela cast. Mover o cursor depois de apertar1 não altera o pedido. Atire ao lado do roxo: ele só perde vida se o corpo interceptar o disparo. A parede verde bloqueia tiros.
- **Vermelho** mantém a habilidade por alvo percebido, disparando contra o jogador; **roxo** é passivo e começa com60/100 de vida. Mortos ficam cinza; R reinicia.

A API é XYZ; o mapeamento do mouse neste laboratório é um plano horizontal. Mira de câmera FPS, retículo contínuo, sockets/animação de arma, balística/homing e decisão tática automática não são entregues aqui. MCP/offscreen não comprovam foco/teclado físico; aceite manual é separado.

Verificação: `python3 scripts/run_verifications.py --only free_aim`, testes `tests/free_aim.rs` e regressão headless completa. Revisão estática em [performance](performance.md).
