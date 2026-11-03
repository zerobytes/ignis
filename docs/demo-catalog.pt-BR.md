# Catálogo histórico de demonstrações

Este catálogo conserva os roteiros acumulados durante o desenvolvimento. As versões e pendências de cada seção são históricas. Para começar e conferir os limites atuais, veja o [README](../README.md) e o [estado público](status.md). Capturas em `artifacts/` são saídas locais regeneráveis, não arquivos distribuídos.

**NPC v1:** inteligência e personalidade por indivíduo, mobilidade híbrida, navegação gerada, circulação, tática/cooperação, rotinas/diálogo, avatares nativos, checkpoint e escala explícita. [Matriz e limites](npc-v1-closure.md) · [Demonstração integrada e teste MCP](npc-v1-demo.md). Scene 97/replay 107; funcionamento offline e sem janela por padrão. Aceite humano separado dos gates automáticos.

# Ignis Engine

## Perseguição terrestre fora da rede

`./target/release/ignis --scene examples/npc-ground-local.scene.json --window` abre a demo opt-in: roxo percebe e segue o jogador amarelo por rampa/plataforma, com conexões locais e rede. P inicia, WASD move, R reinicia, J guias de rota; ambos usam controlador terrestre. O MCP oferece os mesmos controles e inspeção do orçamento; `python3 scripts/run_verifications.py --only ground_local_connections` testa sem janela. [Limites e contrato](ground-local-connections.md).


[Navegação terrestre com rampas e degraus](ground-navigation.md): NPCs escolhem ligações compatíveis usando o mesmo controlador da execução, com consulta MCP e cache por perfil. Demo `examples/npc-ground-navigation.scene.json`: roxo30° faz o desvio verde, azul45° sobe a vermelha curta, cinza recusa degrau50cm. Destinos fixos no alto para comparar capacidades; P inicia, WASD jogador, R reinicia e J mostra rotas. `python3 scripts/run_verifications.py --only ground_navigation` valida sem janela. Conexões livres entre posição/alvo e rede são etapa seguinte.



## Editor nativo

`./target/release/ignis --editor --root "$PWD" --scene examples/stealth-lab.scene.json`

Menus no topo, workspaces Cena/Inspeção, áreas redimensionáveis, abas arrastáveis e menu de área para dividir/maximizar. Hierarquia e propriedades editam a cena pelo dispatcher; Aplicar reinicia a simulação, com undo/redo. Biblioteca de templates e eventos ocupam abas sob demanda. Uma cena ativa por sessão neste primeiro incremento. Layout separado de cena/replay, consultável/editável/persistido pelo MCP; `--editor --mcp` compartilha a mesma sessão.

[Contrato e limites](editor.md). Validação sem janela: `python3 scripts/run_verifications.py --only editor`. `capture_editor` compõe a mesma interface offscreen; entrada física e aceite humano são gates separados. A demo jogável anterior continua disponível com `--window`.

A branch `codex/editor` reúne também as frentes de autoria e pesquisa: [integração e atualização recorrente](editor-integration.md). Receber uma capacidade no núcleo/MCP não conclui sua ferramenta visual; a paridade de UI continua obrigatória para os recursos existentes e futuros.
Single player offline e multiplayer são requisitos permanentes da engine. O núcleo
local não exige internet, autenticação remota ou processo separado de servidor.

Host opcional headless: `./target/release/ignis --server 127.0.0.1:4433 --scene examples/stealth-lab.scene.json`.
O roster é o da cena. `--mcp` ou API compartilhada inicia/inspeciona host e cliente,
com certificado explicitamente confiado, sessões e réplica limitadas. Sem esse
opt-in, nenhuma thread/endpoint de rede é criada. [Contrato, comandos e limites](quic-transport.md).

[Transações locais e buffs indexados](local-transactions.md): alterações por
campo tocado e consultas por dono/definição, preservando atomicidade e replay.
Verificação: `python3 scripts/run_verifications.py --only local_transactions`;
no macOS, o cenário executa o núcleo com acesso à rede negado ao processo.

[Execução contínua](continuous-runtime.md): `set_execution_mode` com
`mode:"continuous"` no tick0, antes de entradas, permite jogar sem acumular o log
de replay. `recorded` é o padrão limitado e reproduzível; `inspect_execution`
expõe a política/filas, e exportar replay sem histórico é recusado. Ambos os modos
funcionam no mesmo núcleo offline.

[Controles independentes por personagem](actor-controls.md): múltiplos Player
com vínculos exclusivos e inputs próprios pelo MCP, preservando física, habilidades
e replay. O [armazenamento por ator](scalable-actor-runtime.md) admite até
1.000 Players e oito NPCs, com validação crítica de área/vida/identidade/replay.
[Transporte QUIC real](quic-transport.md) já valida1.000conexões e20milhits em loopback; [Apresentação remota](remote-presentation.md) já compõe1000atores no cliente; controles nativos/WAN e cargas combinadas continuam gates próprios. Verificação: `python3 scripts/run_verifications.py --only actor_controls`.
[Controlador terrestre opcional](character-controller.md): rampas/degraus configuráveis, gravidade e inspeção MCP, mantendo motor dinâmico padrão. Demo `examples/npc-character-controller.scene.json`: rampa verde20°, vermelha40° e degraus azuis20cm. P inicia, WASD movimenta jogador, R reinicia; MCP `inspect_character_controllers` mostra correção física. Guarda roxo usa planejamento planar existente; navegação inclinada é a próxima etapa. Automação sem janela: `python3 scripts/run_verifications.py --only character_controller`.


[Geometria compartilhada de caixas e cunhas](geometry.md): rampas estáticas com volume coerente para render, física e consultas; demo `examples/npc-geometry.scene.json`. Motores e planejamento de subida continuam separados.

[Nascimento sobre superfícies de apoio](spawn-support.md): consulta MCP somente leitura e projeção opcional por gerador, considerando piso, corpo e ocupação; demo `examples/npc-spawn-support.scene.json`.

[Composição ponderada de população](weighted-population.md): listas de templates por gerador/onda, escolha reproduzível e estável nas retentativas; demo `examples/npc-weighted-population.scene.json`.

[Nascimento procedural em regiões](spawn-regions.md), com candidatos XYZ limitados, seed reproduzível e demo `examples/npc-spawn-regions.scene.json` pelo MCP.

[Ativação espacial de geradores](generator-activation.md): ocupação de região libera ondas uma vez ou enquanto ocupado, com controle MCP e demo `examples/npc-generator-activation.scene.json`.

Controle de geradores em execução: [ativação, pausa e cancelamento](generator-control.md), com demo `examples/npc-generator-control.scene.json` e MCP compartilhado.

[Ondas finitas de NPCs](population-waves.md): quotas, templates, atrasos e condições de conclusão pelo MCP, com demo de três etapas e replay determinístico.

Geradores de NPCs por template: semente, candidatos XYZ, meta viva, cadência e orçamento, com retirada opcional de cadáver. Controle/inspeção via MCP; [contrato e demo](population-generators.md).

Nascimento e retirada de NPCs pelo MCP: `set_npc_templates`, `inspect_templates`, `spawn_actor`, `despawn_actor` e `inspect_population`. Cada nascimento tem identidade/estado próprios, mesmo reutilizando armazenamento. [Contrato e limites](spawn-templates.md), cena47/replay51. Validação sem janela: `python3 scripts/run_verifications.py --only spawn population`.

Repositório público: [zerobytes/ignis](https://github.com/zerobytes/ignis). A menção Apache-2.0 pertence à publicação histórica. Para esta distribuição, consulte a licença comercial em LICENSE e NOTICE na raiz.

Ressurreição do mesmo indivíduo: [contrato e teste](resurrection.md). Identidade de instâncias e tokens MCP: [contrato](instance-identity.md), cena47/replay51. Buffs, percepção e históricos: [vínculos persistentes](identity-bindings.md). Mundo em execução e contextos de origem: [contrato](runtime-world.md), `inspect_world` e `inspect_origins` no MCP. Spawn/despawn reutilizam slots com segurança, conforme o [contrato de templates](spawn-templates.md). Geradores e reposição automática são a próxima etapa.

## Demo: ressurreição

`./target/release/ignis --scene examples/npc-resurrection.scene.json --window`

P inicia; roxo gasta energia/entra em cooldown. Selecione-o e use1 para matá-lo: verde observa e prepara a ressurreição por2s; roxo retorna com40 de vida, preservando energia/prazos. 2 ressuscita o selecionado,3 cura vivo,4 emite sinal; J barras, X cancela, R reset. Azul observa; cinza atrás da parede começa morto. Automação sempre headless; teste manual separado.


## Demo: apoio por efeitos removíveis

`./target/release/ignis --scene examples/npc-dispel-triage.scene.json --window`

Verde dissipa efeitos removíveis de aliados observados; azul protegido e cinza oculto não motivam a ação. P inicia, clique seleciona,1 aplica removível,2 protegido,3 dissipa selecionado,4 buff removível,5 aplica em si,6 dissipa em si. WASD/J/X/R. Contrato e resultados esperados em [docs/dispel-triage.md](dispel-triage.md).


Demo atual: [triagem de aliados observados](ally-triage.md). Verde escolhe feridos visíveis sem perder a memória do inimigo; roxo/azul recebem apoio e cinza oculto não recebe cura.

```sh
./target/release/ignis --scene examples/npc-triage.scene.json --window
```

P inicia; WASD move; clique seleciona;1 dano20,2 cura30 no selecionado,3 cura30 própria; J guias/barras, R reinicia.


Demo atual de auras móveis: `./target/release/ignis --scene examples/npc-aura.scene.json --window`. **P** inicia, **WASD** move, **J** guias. **1** aura de dano em hostis; **2** aura de cura em aliados; **3** zona fixa para comparação; **4** buff; **5** cura instantânea; **6** dissipa resistências. Auras10s/pulsos1s. NPC verde voa com aura própria. Azul-claro acompanha o corpo, verde-lima permanece no mundo. [Contrato e roteiro](auras.md). Teste headless: `python3 scripts/run_verifications.py --only aura`.

Demo de áreas persistentes: `./target/release/ignis --scene examples/npc-persistent-area.scene.json --window`. P inicia, J guias, WASD move. 1 projétil cria zona no impacto; 2 zona sob cursor, renova/reposiciona; 3 zona no chão que também afeta jogador (saia do raio); 4 buff de dano; 5 cura; 6 dissipa resistências. Zonas duram6s com pulsos a cada1s; guia verde-lima permanece durante a vida. [Contrato e controles](persistent-area.md). Validação headless: `python3 scripts/run_verifications.py --only persistent_area`.

## Laboratório de áreas instantâneas 3D

`./target/release/ignis --scene examples/npc-area.scene.json --window`

**P** inicia; **WASD** move; clique seleciona; **J** mostra guias/barras; **X** cancela e **R** reinicia. **1** projétil explosivo no alvo selecionado; **2** explosão no ponto do cursor (plano na altura do jogador); **3** explosão ao redor do jogador; **4** poder próprio +25%fire/10s; **5** cura500 em área ao redor do jogador; **6** dissipa resistências na área do cursor.

Mire2 no azul central: vermelho/azul/roxo recebem100/70/40, cinza atrás da parede permanece protegido. Com4, passam a110/80/50. A esfera ciano é a preparação; laranja é a área entregue, visível por1s de simulação. Raio4.5m; cura8m; alturas e paredes contam. NPCs são alvos passivos com suas resistências. Custo único e falhas por destinatário. [Contrato](instant-area.md). Teste sem janela: `python3 scripts/run_verifications.py --only area`.


Demo de dano causado: `--scene examples/npc-outgoing.scene.json --window`. Clique seleciona; **1** dano direto (40fire+60ice), **2** projétil, **3** três pulsos, **4** buff próprio de +25%fire por10s, **5** dissipação de poder no selecionado, **6** cura. Vermelho/azul/roxo recebem100/70/40 sem buff e110/80/50 com buff. **P/WASD/J/X/R**. [Contrato, limites e roteiro](outgoing-damage.md).

**Da centelha da ideia a um mundo jogável.**

Engine 3D em Rust com controle por MCP e uma janela nativa sobre a mesma simulação. Este repositório contém o **primeiro laboratório executável**, não a engine completa do roadmap.

## Laboratório de dissipação

`--scene examples/npc-dispel.scene.json --window`: **1** envenena o selecionado, **2** antídoto próprio, **3** antídoto no selecionado, **4** regeneração. Antídoto remove até duas instâncias poison e preserva regeneration; sem veneno removível, falha sem custo. Roxo passivo, azul usa antídoto próprio, vermelho lança veneno protegido (`dispellable:false`) no jogador; o antídoto não o remove. **P/WASD/J/X/R** e clique; faixa lilás indica qualquer agenda restante. [Contrato e roteiro](dispel.md). Teste MCP headless: `python3 scripts/run_verifications.py --only dispel`.

## Laboratório de efeitos periódicos

`--scene examples/npc-periodic.scene.json --window`: clique no roxo e use **1** para veneno direto, **2** para regeneração própria e **3** para projétil de veneno. **Vermelho** lança veneno no jogador observado. **P/WASD/J/R** iniciam, movimentam, mostram barras/faixa lilás e reiniciam. Vida muda em pulsos; custo é único, refresh renova e independent acumula agendas. [Contrato, limites e roteiro](periodic-effects.md). Verificação MCP sem janela: `python3 scripts/run_verifications.py --only periodic`.

## Laboratório de mira dos NPCs

`--scene examples/npc-aim.scene.json`: **vermelho** congela o ponto observado no início do cast; **azul** mira no personagem até lançar. Mova durante a barra de preparo e compare os disparos. **J** mostra o marcador amarelo do ponto congelado. **P/WASD/R** iniciam, movimentam e reiniciam; jogador **1** tiro livre, **2** cura, **3** tiro por alvo. [Contrato, parâmetros e teste manual](npc-aim.md). Regra perceptiva e colisão são compartilhadas; `python3 scripts/run_verifications.py --only npc_aim` testa sem abrir janela.

## Laboratório de mira livre

`--scene examples/free-aim.scene.json`: **1** dispara na direção do cursor, sem selecionar personagem; aponte ao centro do corpo roxo. **2** cura própria; **3** projétil por alvo selecionado, para comparar. **P** inicia, **WASD** move, **X** cancela, **R** reinicia e **J** mostra barras. Vermelho mantém disparos contra o jogador, parede bloqueia e tiro no vazio consome energia. [Contrato XYZ, parâmetros e teste manual](free-aim.md). MCP e mouse usam o mesmo dispatcher; `python3 scripts/run_verifications.py --only free_aim` verifica sem janela.

## Laboratório de projéteis

`--scene examples/npc-projectiles.scene.json`: **vermelho** dispara contra o jogador; os cubos amarelos viajam em linha reta e podem ser desviados ou bloqueados pela parede verde. **P** inicia, **WASD** move, **J** barras e clique seleciona. Jogador: **1** projétil de dano, **2** cura própria, **3** projétil de cura; **X** cancela cast e **R** reinicia. **Roxo** é alvo passivo. Energia é cobrada no lançamento; efeito só no impacto. [Contrato e parâmetros](projectiles.md). Regressão MCP sem janela: `python3 scripts/run_verifications.py --only projectiles`.

## Laboratório de cast e movimento

`--scene examples/cast-movement.scene.json`: **1** ataque que permite andar, **2** cura que cancela ao andar, **3** cura que bloqueia a locomoção durante o cast. **P** inicia, **J** barras, clique no roxo para selecionar o alvo do ataque, **X** cancela e **R** reinicia. Cada preparação dura1,5s; segurar a direção no modo3 retoma o movimento ao concluir. [Contrato e teste manual](cast-movement.md). Tudo configurável por habilidade no MCP; regressão headless `python3 scripts/run_verifications.py --only cast_movement`.

## Laboratório de vitalidade

`--scene examples/npc-ability-policy.scene.json`: **vermelho** aproxima e ataca, **azul** cura o jogador somente abaixo de70% de vida e **roxo** se cura abaixo de50%. Políticas opcionais com limiar, alcance/margem e decisões inspecionáveis pelo MCP. **P** inicia; **J** barras; clique seleciona; **1** dano, **2** cura própria, **3** cura do selecionado, **R** reinicia. [Contrato e teste manual](npc-ability-policy.md). Regressão: `python3 scripts/run_verifications.py --only ability_policy`.

`--scene examples/npc-vitality.scene.json`: **vermelho** ataca o jogador, **azul** cura e **roxo** é um alvo passivo. **P** inicia, **J** mostra vida/energia/cast; clique seleciona alvo. Jogador: **1** dano, **2** cura própria, **3** cura do selecionado, **X** cancela e **R** reinicia. Mortos ficam cinza, param de agir e mantêm o corpo físico. [Contrato e parâmetros do teste](vitality.md). Verificação via MCP sem janela: `python3 scripts/run_verifications.py --only vitality`.

## Laboratório de habilidades

`--scene examples/npc-abilities.scene.json`: **roxo** converte energia em foco até atingir60; **azul** fornece foco ao jogador enquanto o vê; **cinza** investiga sinais sonoros. **P** inicia, **J** mostra barras (energia azul, foco verde, cast amarelo), **V/H** mostram sensores. Jogador: **1** converte, **2** energiza o ator selecionado por clique, **3** emite sinal e **X** cancela o cast. **R** restaura recursos. Não há dano ou combate neste recorte.

Custos, tempos, comportamento esperado e limites em [habilidades compartilhadas](abilities.md). Tudo autorado/inspecionado pelo MCP; `python3 scripts/run_verifications.py --only abilities` verifica sem abrir janela.

## Laboratório de comunicação entre NPCs

`--scene examples/npc-radio.scene.json`: o **roxo** permanece no posto, vê o jogador laranja e avisa pelo rádio. O **azul**, atrás da parede, recebe a última posição observada e começa a contorná-la para investigar. O **cinza** está em outro canal e não recebe o aviso; ainda pode reagir se enxergar você diretamente. **P** inicia/pausa, **R** reinicia, **V** mostra visão, **J** rotas e **WASD** move o jogador.

Roxo confirma a visão após0,2s, transmite no máximo uma vez por segundo e usa canal1/alcance14m. Azul recebe no canal1; cinza no canal2. Relatos expiram3s após a observação original. Azul/cinza andam2m/s, enxergam5m num cone120° e guardam observações diretas por4s. Não há passos automáticos nem audição nesta demo; **N não provoca reação**. Rádio atravessa paredes e não é um som de gameplay.

Para observar perda de pista, saia da visão do roxo: o azul pode investigar o último ponto recebido, mas só acompanha sua posição atual se conseguir vê-lo. Configuração e origem/idade das mensagens são acessíveis pelo MCP; nenhuma janela é necessária para a verificação automática.

## Laboratório de superfícies modulares

`--scene examples/npc-walk-surfaces.scene.json`: **azul** anda de uma ponta à outra sobre três peças contíguas; **roxo** não atravessa os vãos da segunda passagem; **cinza** recusa a ponte estreita demais para seu corpo. **P** inicia, **R** reinicia e **J** mostra rotas. Esta demo isola apoio e caminhada; N não altera esses NPCs. O jogador laranja permanece no piso inferior.

As plataformas têm topo a2m e são entidades independentes. Azul caminha2m/s, espera0,5s em cada extremidade e usa a mesma consulta de apoio em links autorados e conexões locais. Remover a peça central pelo MCP torna a travessia inviável; undo a restaura. Rota desenhada é intenção, não prova de passagem. A verificação considera toda a área do corpo, inclusive junções e buracos pequenos; superfícies inclinadas, subida automática de degraus e navmesh permanecem futuros.

## Laboratório de patrulhas 3D

`--scene examples/npc-spatial-patrol.scene.json`: azul percorre seis pontos num anel inclinado; roxo salta para uma plataforma e volta; verde plana até o chão e falha ao tentar subir de volta; cinza só caminha e não alcança o ponto elevado. Falhas levam a novas tentativas com espera, sem conceder capacidades inexistentes. **P** inicia/pausa, **R** reinicia e **J** mostra rotas. O jogador laranja usa WASD.

O azul usa voo retilíneo3m/s e espera0,5s nos pontos; a rota procedural é um círculo inclinado de raio2,5m com centroY4,5m. Roxo usa velocidade inicial de salto10m/s. Verde desce2m/s com limite horizontal4m/s. Todos têm prazo20s por ponto e cooldown2s se a rota inteira falhar. O azul ouve até12m, guarda a pista0,5s e retoma a patrulha; **N** só o interrompe quando o impulso sonoro também o alcança. Os demais isolam suas rotinas, sem perseguição. Passos automáticos desligados. O teste também cobre espera em órbita para voadores sem hover; a demo azul usa paradas deliberadas para tornar os pontos legíveis.

Pontos XYZ e anéis em plano configurável são autorados/inspecionados pelo MCP. Rede usa posição dos pés; rotina usa centro do corpo. `scripts/run_verifications.py --only spatial_patrol` valida sem abrir janela.

**Para testar N:** pressione **H** para ligar as guias de audição/ruídos (desligadas ao abrir a janela), aproxime o jogador do azul e use **P** para deixar a simulação rodando. Cada toque em **N** mostra círculo e cruz laranja no local da emissão por1,5s de simulação; isso confirma emissão, não recepção. O azul deve investigar a posição emitida e depois retomar a patrulha, podendo terminar antes um trecho aéreo já comprometido. O jogador começa a cerca de17m do azul, fora do alcance12m, que considera também a altura. Pausado, o ruído fica pendente até avançar a simulação. Roxo/verde/cinza não têm audição nesta demo; a demo antiga `npc-flight` também não reage a ruídos.

## Laboratório de voo curvado

`--scene examples/npc-flight.scene.json`: azul voa em linha reta; roxo faz uma curva e desacelera até pairar; verde faz uma curva e continua orbitando após passar pelo destino. **P** inicia/pausa, **R** reinicia e **J** alterna trajetória violeta/direção amarela. Os três têm destinos fixos; o jogador laranja pode andar com WASD.

Perfil de voo pelo MCP: velocidade máxima, aceleração/frenagem, giro, suavidade, velocidade mínima de cruzeiro, capacidade de pairar e direção inicial. Curvas são previstas e verificadas contra obstáculos antes da execução física. Configurações que não cabem no espaço falham explicitamente. Voo retilíneo continua disponível. Este recorte controla locomoção; rotação visual do corpo e aerodinâmica ainda não estão implementadas.

## Laboratório de conexões locais

`--scene examples/npc-connectors.scene.json`: roxo persegue o jogador laranja usando posições livres no chão e a rede para contornar a parede. Azul sobe, cruza acima dela e segue até um ponto fixo no ar. **P** inicia/pausa, **WASD** move o jogador, **N** emite ruído, **R** reinicia, **J** mostra rede/caminhos e **V** visão. X ciano marca a conexão local ativa.

Roxo tem visão360°/16m, memória3s, audição16m e caminhada3m/s com conexões locais até4,5m. Azul voa3m/s e conecta até6m. Conexões automáticas são segmentos livres de caminhada/voo; saltos e planeios continuam explícitos. Perder uma pista não cancela um trecho já comprometido. A rede ainda limita os desvios possíveis; posição arbitrária não implica alcance garantido.

## Laboratório de locomoção 3D

`examples/npc-mobility.scene.json`: roxo salta duas plataformas; azul sobe, cruza acima da parede e pousa; verde começa no ar junto à plataforma alta e plana até o chão; cinza tem apenas caminhada e se aproxima até a base da plataforma, onde para sem concluir o objetivo elevado. O laranja é o jogador. P inicia/pausa, R reinicia, J mostra rede/rota/arco, WASD move o jogador.

O cinza usa `unreachable_policy: approach_reachable`, progresso mínimo de 0,25 m e custo estimado de rota parcial limitado a 300 ticks (5 s). Sem essa política, o padrão exige caminho completo. O X laranja indica o ponto parcial; o destino original permanece indicado acima da plataforma.

A rede é explícita e configurável pelo MCP. O planejador seleciona conexões compatíveis; o motor move corpos físicos, sem teleporte. Esta demo usa destinos fixos, não perseguição. Planeio usa descida constante, não aerodinâmica completa. Veja `docs/architecture.md` e `docs/commands.md` para limites e contratos.

## Executar

Requer Rust/Cargo. A plataforma validada inicialmente é macOS Apple Silicon.

```sh
cargo build --locked --release
./target/release/ignis --root . --window
```

A janela começa pausada. **P** inicia/pausa; **WASD ou setas** movem o personagem laranja; **Shift** corre, **Control** anda furtivamente; **N** emite um ruído de gameplay; **V** alterna guias de visão; **H** alterna alcance auditivo e ruídos recentes; **J** alterna caminhos e patrulha; **clique esquerdo** seleciona uma caixa; **R** reinicia; **Esc** fecha. O eixo Y aponta para cima; W/seta para cima move em -Z. A câmera é fixa, em perspectiva. O título informa estado, passo, perfil de movimento e entidade selecionada.

O executável funciona sem Cargo, agente ou conexão de rede. `--root` identifica a pasta de projeto; a sala padrão pode ser criada sem arquivos externos. Para abrir uma cena salva:

```sh
./target/release/ignis --root . --scene examples/lab.scene.json --window
```

No macOS, `python3 scripts/package_macos.py --editor --install` instala **Ignis Editor.app** em `~/Applications`, para abrir com dois cliques. Abre o editor com a cena de laboratório. Sem flags, o script mantém a demo em `artifacts/Ignis.app`. O app usa o build corrente por hardlink ou link simbólico entre volumes; reexecute após substituir o build. É um app local dependente desta pasta de projeto e do volume do build, sem cópia do binário nem instalador de distribuição.

## Laboratório de território

`--scene examples/npc-passages.scene.json`: dois NPCs querem trocar de lado pela mesma porta. **P** inicia; roxo tem prioridade10 e azul0. O azul se desloca para o X amarelo lateral, o roxo atravessa e libera a reserva, depois o azul atravessa. Ambos continuam até seus postos. **R** reinicia; **J** mostra passagem, espera e caminhos. Você é laranja e pode bloquear fisicamente o vão; colisões continuam valendo.

Passagem explícita de2m por1,4m, duas vagas de espera por lado, reserva de até10s/cooldown1s, envelhecimento de prioridade1ponto/s. Velocidade2m/s. A faixa tracejada fica verde com dono, vermelha com ocupante adicional e cinza livre; cruzes amarelas são vagas autoradas, cruzes brancas destinos atuais. `inspect_passages` explica pedidos, dono e razões. Configuração via `set_passages`/`edit_scene` e `npc.passage_priority`; ausência mantém navegação legada. São regras determinísticas de coordenação, independentes da escolha de objetivos e da percepção. `python3 scripts/verify_passages.py` valida travessia, prazo, ocupação, persistência e replay.

`--scene examples/npc-recovery.scene.json`: demonstração de passagem bloqueada. Você (laranja) começa no vão da parede; o guarda roxo quer chegar ao X do outro lado. **P** inicia: ele fica aguardando porque seu corpo impede a passagem. Mova-se na diagonal com **D+S** por cerca de um segundo para liberar o vão e depois pare; o guarda deve atravessar e chegar ao posto. **R** reinicia. O guarda não está perseguindo você: seu objetivo é o posto fixo. **J** mostra caminho/destino; X vermelho indica falha/espera, branco indica caminho disponível/chegada.

Configuração da demo: retry15ticks/0,25s, máximo60ticks/1s, detecção sem progresso após30ticks/0,5s com deslocamento mínimo0,1m; velocidade2m/s. A espera dobra após falhas sucessivas e limita a frequência de tentativas; liberar a passagem não provoca resposta instantânea. Progresso físico/chegada reiniciam a sequência. `inspect_navigation` expõe prazos, contagens e motivo. `python3 scripts/verify_nav_recovery.py` verifica a liberação física sem editar/resetar a cena. Não há negociação de passagem entre agentes nesta etapa.

`--scene examples/npc-territory.scene.json`: você laranja e guarda roxo, círculo territorial de5m centrado no X. **P** inicia; **WASD/setas** movem; **N** emite ruído; **R** reinicia; **J** alterna território/rotas. Passos automáticos desligados. Visão e audição podem ser mostradas com **V/H**; inicialmente ficam ocultas para destacar a fronteira.

Você começa dentro do território e o guarda pode vê-lo. Saia do círculo pelo lado aberto, à direita: ele deve desistir ao ver a saída, o círculo fica vermelho e ele volta ao X. Enquanto retorna e espera2s na origem, aproximar-se/fazer barulho não interrompe. Depois o círculo volta a laranja: fora dele você não reativa perseguição; entrando e sendo percebido, sim. Se sair escondido atrás da parede, ele não sabe que saiu: investiga a última pista até o orçamento acabar e retoma a patrulha.

Configuração: home[-3,0], raio5m, chegada0.4m (+0.02 de tolerância), rearme120ticks/2s. Guarda1.5m/s, visão360°/12m para isolar o limite territorial, audição10m, memória3s e busca1.5s. O raio é regra de comportamento, sem parede invisível; desvio/empurrão pode ultrapassá-lo. Origem inacessível mantém tentativa e falha explícita. `inspect_territory` mostra o motivo; `python3 scripts/verify_territory.py` verifica o fluxo/MCP/replay/guias.

## Laboratório de objetivos

`--scene examples/npc-objectives.scene.json` demonstra três papéis: **roxo guarda**, patrulha e reage a visão/som; **ciano civil**, foge da posição percebida do jogador e retorna ao ponto inicial quando esquece; **azul aliado**, acompanha o jogador percebido e espera quando perde a pista. **Laranja é você.** Todos usam os mesmos sensores/navegador/seletor, sem combate.

**P** inicia, **WASD/setas** movem, **N** emite ruído e **R** reinicia. Passos automáticos desligados para isolar o som manual. Inicialmente a parede oculta você do guarda: N interrompe a patrulha. Civil e aliado têm visão360° nesta demonstração; o guarda120°. **V/H/J** alternam guias de visão, audição/ruído e rotas. O contorno dourado indica seleção, não hostilidade.

Guarda: velocidade1.5m/s, visão12m, audição10m, memória3s, busca1.5s. Civil:2m/s, visão8m, memória2s, fuga até6m da pista. Aliado:1.5m/s, visão12m, memória3s, distância de acompanhamento2m. Civil/aliado sem audição neste teste. Ao perder visão, usam a posição lembrada até expirar; não rastreiam posição oculta. A fuga é radial e pode ficar bloqueada; `inspect_objectives`/`inspect_navigation` mostram a causa.

Objetivos por prioridade e condição são autorados via MCP, assim como esta cena. `python3 scripts/verify_objectives.py` verifica os papéis, interrupção/retomada, save/load, replay e erros atômicos.

## Laboratório de atenção sonora

`--scene examples/npc-sound-attention.scene.json`: laranja jogador, roxo ouvinte, parede para desvio. **Este teste isola audição:** roxo sem alvo visual; passos do jogador desativados. **P** inicia e **N** emite uma pista no seu ponto atual; aproximar-se sem N não dá informação nova ao NPC. **H/J** mostram audição/ruídos e caminhos; **R** reinicia.

O ouvinte mantém a pista por pelo menos1.5s enquanto sua memória durar. Outro N até1.25m do ponto lembrado é repetição e não renova a busca. Em outro ponto, após o compromisso, só troca se o escore recebido superar o anterior em mais de0.15. Ande para um ponto distinto e mais próximo do ouvinte para produzir uma pista mais forte. Alcance10m, memória10s, busca3s, velocidade1.5m/s. `inspect_hearing` explica cada aceitação/rejeição. O NPC investiga pontos; não conhece sua localização continuamente.

## Laboratório de confirmação visual

`--scene examples/npc-recognition.scene.json`: somente você (laranja) e o guarda (roxo), com uma parede para ocultação. **P** inicia. Fique parado: o guia fica amarelo enquanto o guarda acumula2s de exposição; depois fica verde e ele começa a perseguir. **R** reinicia. `inspect_knowledge` mostra o progresso por indivíduo pelo MCP.

Acquire120ticks/2s, decay240ticks/4s do máximo a zero, memória de perseguição180ticks/3s. Visão120°/12m e audição10m. Mover-se emite passos e **N** emite um ruído: isso pode iniciar investigação antes da confirmação visual, sem identificar o emissor. Reencontro antes de a confirmação zerar é imediato. Não há combate nesta demonstração.

## Laboratório de relações entre NPCs

`--scene examples/npc-relations.scene.json` abre a demonstração: guarda roxo seleciona entre jogador laranja e patrulheiro vermelho; aliado ciano é excluído. Todos são caixas de teste. O guarda exige visão ou pista sonora, com visão120°/12m, audição10m, memória3s e margem20% para trocar entre hostis ainda visíveis. P inicia; WASD move; V/H/J mostram sensores e rotas; R reinicia. A seleção e suas razões estão em `inspect_targeting` pelo MCP. Não há ataque/dano nesta cena.

## Identidade e migração

O nome do produto é **Ignis Engine**; pacote Rust `ignis-engine`, biblioteca e executável `ignis`. O nome anterior era Engine AI First. Configurações MCP existentes devem apontar para `target/release/ignis`. A ferramenta `engine_command`, seus comandos e o formato de cenas permanecem iguais. Replays continuam restritos ao build que os gerou; gere um novo replay após recompilar.

No Kordo, os nomes visíveis foram atualizados; o prefixo `EAF`, os IDs e os slugs existentes são preservados para manter o histórico e as integrações. O diretório do repositório continua `custom-engine`.

## MCP

O transporte implementado é **stdio**, com mensagens JSON-RPC delimitadas por linha. Logs vão para stderr. Configure seu cliente MCP com o caminho absoluto do executável e da pasta do projeto:

```json
{
  "mcpServers": {
    "ignis-engine": {
      "command": "/CAMINHO/custom-engine/target/release/ignis",
      "args": ["--root", "/CAMINHO/custom-engine", "--mcp"]
    }
  }
}
```

A configuração acima executa sem janela e não toma foco do desktop. Acrescentar `--window` é uma escolha explícita para acompanhar visualmente a sessão. A simulação não inicializa GPU; a captura inicializa um renderizador offscreen quando solicitada. Janela e MCP no mesmo processo compartilham o mesmo estado. Processos diferentes são sessões independentes.

Após o handshake MCP, `tools/list` fornece a ferramenta `engine_command` com o JSON Schema completo derivado dos tipos Rust. Exemplo de argumentos:

```json
{"command":{"op":"generate","recipe":{"width":16,"depth":14,"seed":42,"obstacles":4}}}
```

```json
{"command":{"op":"set_input","input":{"x":0,"z":-1}}}
```

```json
{"command":{"op":"step","ticks":120}}
```

```json
{"command":{"op":"capture","path":"artifacts/view.png","width":1280,"height":900}}
```

`capture` devolve pixels PNG em um bloco de imagem MCP, metadados da câmera, revisão, passo e adapter gráfico. `inspect` fornece documento de edição, posições da partida e contatos. Para evitar edições com estado desatualizado, envie `expected_revision` junto de `command`.

Consulte [o contrato de comandos](commands.md) e [a arquitetura](architecture.md).

Todo incremento de código passa pelo [checklist de performance](performance.md) antes da implementação e na revisão final: alocações, tipos, acesso à memória, trabalho repetido, GPU/I/O, locks e invalidação de caches. É revisão estática obrigatória, sem profiling como requisito nesta fase. As otimizações locais não representam uma promessa de ganho de FPS medido.

## Validação

```sh
cargo test --locked --release --all-targets
cargo clippy --locked --release --all-targets -- -D warnings
python3 scripts/run_verifications.py --binary target/release/ignis
```

A regressão completa executa os cenários sequencialmente, sem janela, por MCP real. `--only flight render` seleciona um recorte. Exemplos são copiados para `artifacts/mcp-validation/examples`; capturas, replays e relatórios ficam em `artifacts/mcp-validation/artifacts`, logs por cenário em `logs` e resumo incremental em `report.json`. Uma falha interrompe a suíte, retorna status não zero e permanece no relatório sem repetição automática. Cada nova execução substitui o resumo corrente; confira `selected`/`results`, pois arquivos de cenários não executados podem pertencer à execução anterior. A suíte não recompila nem modifica o binário, os exemplos versionados ou a sessão visual aberta. Uma trava impede suítes concorrentes no mesmo diretório.

Integração desktop é separada e opt-in: `python3 scripts/run_verifications.py --only lab --window`. Pode abrir e tomar foco; não faz parte da regressão habitual. Capturas offscreen validam desenho na GPU, mas não apresentação nativa, foco ou entrada física. Esses últimos exigem teste específico/manual quando alterados.

Os scripts individuais continuam disponíveis para autoria explícita de demos: por exemplo, `python3 scripts/verify_flight.py` regrava seu exemplo no projeto. O executor usa `IGNIS_VERIFY_ROOT` para isolar essas gravações. O verificador individual do laboratório inicia um processo real e conversa exclusivamente pelo MCP. Remove uma parede, detecta a fuga, desfaz a edição e repete as entradas; verifica eventos, persistência, replay e capturas. Os resultados ficam em `artifacts/verification.json`, `artifacts/lab.png` e `artifacts/collision.png`. Esses arquivos são substituídos a cada execução, não versionados por tentativa.

No verificador individual `verify_lab.py`, `--window --hold` mantém a janela aberta para inspeção. Criar `artifacts/close-window` solicita fechamento pelo MCP. A inspeção automatizada não substitui o aceite humano.

## Demonstração de visão volumétrica

`examples/npc-spatial-vision.scene.json` compara dois observadores parados: azul olha horizontalmente, roxo olha35° para cima. Jogador laranja começa sobre a plataforma. **P** inicia a percepção, **V** mostra os volumes, **R** reinicia. Roxo detecta o jogador e fica com guia verde; azul permanece sem detecção. O objetivo `wait` é intencional para isolar o sensor. Configuração, direçãoXYZ, capturas e replay disponíveis pelo MCP.

Teste sem janela: `python3 scripts/run_verifications.py --only spatial_vision`. Para abrir a demo explicitamente, use o binário corrente com `--scene examples/npc-spatial-vision.scene.json --window`. O executor não abre/reinicia demonstrações.

## Demonstração de perseguição

```sh
./target/release/ignis --root . --scene examples/npc.scene.json --window
```

O NPC roxo persegue o jogador laranja e contorna a barreira verde. **P** inicia/pausa; **WASD** move o jogador; **R** reinicia. Adicione `--mcp` para controlar e observar essa mesma sessão pelo protocolo. A cena foi construída e validada pelo MCP; `upsert` configura alvo, piso, velocidade e distância de parada, e `inspect_navigation` informa a decisão e o caminho.

Esta cena histórica preserva perseguição **onisciente** por migração explícita. NPCs novos usam percepção visual e memória por padrão. A visibilidade geométrica da navegação e o sensor visual são sistemas distintos; veja a decisão ADR 0003 na [arquitetura](architecture.md).

```sh
python3 scripts/verify_npc.py --binary target/release/ignis
```

O roteiro verifica desvio, alvo móvel, bloqueio total, undo/redo, desativação, save/load e replay de posições e decisões. Evidências em `artifacts/npc-verification.json` e capturas `npc-before.png`, `npc-detour.png`, `npc-arrived.png`. A navegação deste recorte ocorre em um piso plano; veja os limites na [arquitetura](architecture.md).

## Demonstração de percepção e fuga

```sh
./target/release/ignis --root . --scene examples/perception.scene.json --mcp --window
```

O NPC roxo começa à esquerda do jogador, olhando para ele. **P** inicia; segure **D** por cerca de dois segundos para sair do alcance de visão. O NPC segue até a última posição vista, procura girando sua orientação de visão e desiste. A caixa visual não gira: a orientação sensorial é exposta por `inspect_npcs`. Para detectar novamente, o jogador precisa entrar no alcance e campo visual atuais; **R** reinicia a demonstração.

`inspect_npcs` informa fase, motivo, observação memorizada, confiança por recência, orientação e distância investigada. `inspect_navigation` informa o objetivo conhecido e a rota. Configure alcance, FOV, orientação inicial, memória, duração/velocidade de busca, distância máxima investigada ou modo onisciente via `upsert`; `blocks_vision` separa oclusão de colisão. Tempo e distância podem ser usados separadamente ou juntos: vence o primeiro limite ativo. Veja a semântica e as tolerâncias no [contrato](commands.md). A demonstração padrão mantém memória de 600 ticks, busca de 180 ticks e distância desativada. Busca espacial opcional está descrita abaixo; audição opcional está descrita abaixo; planejamento semântico de rotas de fuga continua futuro.

```sh
python3 scripts/verify_perception.py --binary target/release/ignis
```

A cena acima é salva pelo roteiro através do MCP. Relatório em `artifacts/perception-verification.json`; capturas `perception-before.png`, `perception-searching.png` e `perception-lost.png`. Os replays comparam também o estado de percepção/memória/decisão.

## Demonstração de investigação espacial

```sh
./target/release/ignis --root . --scene examples/search.scene.json --mcp --window
python3 scripts/verify_search.py --binary target/release/ignis
```

O jogador laranja começa à esquerda da parede e o NPC roxo à sua direita, ambos no lado frontal. Aperte **P**, avance com **W** pela ponta esquerda da barreira e depois **D** para se esconder atrás dela. Pare e acompanhe: ao perder visão, o NPC visita a região lembrada e pode continuar por pontos próximos, abandonando a lista se voltar a enxergar o jogador. **R** reinicia. O roteiro automatizado usa 100 ticks de W e 52 de D.

Perfil da cena: velocidade 1,5 m/s; visão 4 m/FOV 120°; orientação inicial -90°; memória 30 s; busca espacial até 20 s ou 40 m; quatro pontos a 3,5 m da memória, semente 0, varredura de 2 s por ponto. Destinos hipotéticos são expostos separadamente da memória. O perfil anterior continua em `examples/perception.scene.json`.

Os pontos seguem um círculo determinístico, sem consultar a posição escondida. O NPC ignora destinos diagnosticados inalcançáveis, pode falhar na busca e desiste ao esgotar pontos ou orçamento. Não identifica portas nem prevê a direção de fuga. Evidências em `artifacts/search-verification.json` e capturas `search-before.png`, `search-exploring.png`, `search-reacquired.png`; cena construída/salva via MCP.

## Demonstração de audição

```sh
./target/release/ignis --root . --scene examples/hearing.scene.json --mcp --window
python3 scripts/verify_hearing.py --binary target/release/ignis
```

O jogador começa escondido atrás da barreira; o NPC, do outro lado, não sabe onde ele está. **P** inicia e **N** emite uma pista sonora na posição atual do jogador. O NPC investiga aquele ponto; mover-se silenciosamente depois não atualiza a pista. Se enxergar o jogador durante o percurso, passa a persegui-lo. **R** reinicia. Este incremento simula percepção de ruído, ainda sem som nos alto-falantes nem passos automáticos.

Perfil: audição 12 m; emissão de N com raio 12 m; transmissão da barreira 0,65 (alcance efetivo 7,8 m atravessando-a); visão 6 m/FOV 120°; velocidade 1,5 m/s; memória 30 s, busca 20 s ou 40 m; quatro pontos a 2 m, varredura de 2 s. `emit_noise` aceita posição explícita ou entidade; `inspect_noises` expõe o log. `sound_transmission` e `hearing` são configuráveis pelo MCP, independentes da colisão/visão. Veja regras de seleção e limites no [contrato](commands.md).

Cena construída/salva pelo MCP. Evidências em `artifacts/hearing-verification.json`, `hearing-before.png`, `hearing-investigating.png` e `hearing-acquired.png`.

## Ver para onde o NPC olha

Pressione **V** ou envie `{"command":{"op":"set_debug_view","npc_vision":true}}` pelo MCP. A seta sobre o NPC acompanha sua orientação atual; o contorno tracejado indica alcance e FOV nominais, sem descontar obstáculos.

**Verde:** viu o jogador no último passo. **Amarelo:** segue uma memória visual/sonora. **Azul:** sem pista. NPC desativado fica cinza; onisciente usa losango rosa sem cone. A legenda principal aparece no título da janela. A cor reflete a última amostra do sensor; a direção já incorpora a navegação daquele passo, podendo diferir por um tick. Estar dentro do contorno não garante visibilidade através de paredes.

A opção é da sessão, não altera gameplay/replay e funciona também nas capturas. Reset/load preservam a preferência. Veja o [contrato](commands.md) para detalhes. Validação: `python3 scripts/verify_vision_guides.py`; evidências em `artifacts/vision-verification.json` e `vision-before.png`, `vision-memory.png`, `vision-seeing.png`.

## O que esta versão oferece

- Cena JSON versionada, IDs estáveis e caixas com cor e corpo físico opcional.
- Sala procedural com dimensões, semente, obstáculos e IDs determinísticos.
- NPCs com visão, memória, audição opcional, busca espacial, desvio em piso plano e diagnóstico pelo MCP.
- Física Rapier, gravidade, colisões e um personagem simples com rotação bloqueada.
- Iluminação direcional/ambiente, câmera em perspectiva e grade procedural.
- Edição validada, undo/redo, conflito de revisão e salvamento atômico.
- Simulação a 60 passos por segundo, controle explícito de passos, eventos e replay.
- Paridade dos controles de jogo da janela com os comandos MCP.
- Execução nativa e captura offscreen pelo mesmo código de renderização.

## Autoria nativa de malhas pelo MCP

A Ignis cria geometria triangulada explícita ou grades procedurais, esculpe com draw/grab/smooth, recebe rigs genéricos com pesos e anima clipes por keyframes. Assets são compartilhados por instâncias visuais; a autoria usa o mesmo dispatcher transacional e undo/redo da cena. `inspect_meshes`, `inspect_mesh` e `inspect_mesh_pose` expõem dados e poses; `play`, `pause`, `step` e `reset` controlam o relógio. Contrato, limites e exemplos em [docs/mesh-authoring.md](mesh-authoring.md).

O [roadmap de modelagem](mesh-roadmap.md) continua com [seleção de componentes e subdivisão linear](mesh-topology.md). `inspect_mesh_topology`/`select_mesh_components` expõem vértices, arestas e faces; `edit_mesh` com `kind:subdivide` divide a superfície inteira ou componentes explícitos com undo/redo. Plantas vinculadas suportam refinamento uniforme declarado, preservando a fonte. Demo [native-mesh-topology.scene.json](../examples/native-mesh-topology.scene.json), verificação `python3 scripts/run_verifications.py --only mesh_topology`. Subdivisão suavizada e editor manual de componentes seguem futuros.

A cena [examples/native-meshes.scene.json](../examples/native-meshes.scene.json) contém uma grade esculpida com três ossos e duas instâncias animadas em fases distintas. `python3 scripts/run_verifications.py --only meshes` reconstrói o pipeline pelo MCP e compara capturas/replay sem abrir janela. Autoria manual por pincel/gizmos/timeline, remesh, IK, anexação a NPC e troca de animação ao vivo ainda não estão implementados.

## Limites explícitos

Corpos e consultas de gameplay usam as caixas/cunhas autorizadas e o controlador terrestre opt-in. Malhas visuais têm transformação e animação próprias; sua topologia não cria colisão de triângulos automaticamente. Transporte e replicação multiplayer ainda são etapas futuras do núcleo local.

O armazenamento da simulação admite4.096entidades, até1.000Players e oito NPCs. O modo gravado limita a sessão a36.000passos (10minutos); o contínuo descarta entradas consumidas e não usa esse teto de gravação. Cada chamada `step` aceita 1–3.600 passos; capturas aceitam 64–2.048 pixels por dimensão. O histórico de edição retém 32 cenas; o registro de contatos retém 2.048 eventos e informa quantos foram descartados. Esses limites são deliberados para este laboratório.

Edições de cena, câmera e luz reiniciam e pausam a partida; undo/redo afetam o documento, não rebobinam a física. `save_scene` salva o documento de edição. Replay reexecuta entradas desde o estado inicial e compara posições e estado de navegação em uma simulação isolada: não é um sistema geral de savegame. É exigida a mesma identificação de build (código, lockfile, compilador, perfil e target). Não há afirmação de determinismo entre plataformas ou builds diferentes.

macOS/Metal é o primeiro ambiente de validação. Os backends de outras plataformas não representam suporte de produto já testado. A estratégia de builds multiplataforma será definida separadamente.

A licença permissiva ainda será escolhida. O repositório não está publicado e nenhuma licença open source foi concedida por este primeiro commit.


Guias auditivos: **H** ou `set_debug_view` com `npc_hearing`/`noise_events`. Roxo representa o alcance nominal do ouvinte; laranja tracejado com cruz representa a origem e o raio do ruído. Até 16 impulsos durante 1,5 s de simulação; pausa conserva a evidência. Alcance nominal não desconta paredes. Verificação: `python3 scripts/verify_hearing_guides.py`.


## Passos físicos — EAF-10

```sh
cargo run --release -- --root . --scene examples/footsteps.scene.json --mcp --window
```

**P** inicia, **WASD** caminha, **V/H** mostram visão/audição. O jogador emite automaticamente a cada 0,6 m efetivamente percorrido no piso, com raio 8 m e velocidade mínima 0,1 m/s; não precisa de N. Ficar parado, segurar contra a parede ou mover-se no ar não gera sequência de passos. Cada personagem player/npc pode receber seu componente opcional via `upsert`; ausência significa silêncio. NPC ignora passos próprios. Esses são estímulos de gameplay; ainda não há áudio no dispositivo.

`inspect_footsteps` mostra apoio, velocidade, fração do passo e histórico limitado. Passos não consomem o log manual; replay regenera a sequência. Componente introduzido na cena/replay 7; formato corrente 18, cenas anteriores sem emissor preservadas. `python3 scripts/verify_footsteps.py` reproduz o cenário e valida caminhada longa, parede, ar, histórico e replay.


## Caminhada, corrida e furtividade — EAF-11

```sh
./target/release/ignis --root . --scene examples/stealth.scene.json --mcp --window
```

**P** inicia; **WASD/setas** caminham; segure **Shift** para correr ou **Control** para mover-se furtivamente. **V/H** mostram os sensores e ruídos, **R** reinicia. O título mostra o modo atual. Perfis desta cena: caminhada 4 m/s/raio 8 m, furtivo 1,5 m/s/raio 1,6 m, corrida 6 m/s/raio 14,4 m. Passos a cada 0,3 m percorridos, piso `floor`; o raio efetivo ainda depende da audição e da transmissão da parede. Furtividade reduz ruído, sem alterar altura/postura ou visão.

Os mesmos modos estão em `set_input.input.mode`; perfis podem ser editados em `player.locomotion` pelo MCP. `python3 scripts/verify_locomotion.py` recria a cena, compara os três modos atrás da parede e verifica silêncio, erros e replay. Detalhes e limites no [contrato](commands.md).


## Patrulha, investigação e retomada — EAF-12

```sh
./target/release/ignis --root . --scene examples/patrol.scene.json --mcp --window
```

O NPC segue quatro pontos ao redor da parede, espera um segundo em cada ponto e usa seus sentidos durante todo o percurso. Ao ver ou ouvir uma pista, interrompe a patrulha para perseguir/investigar; quando perde a pista, retoma o ponto interrompido. Caminhar/correr gera passos; **Control** reduz o ruído. **N** emite um impulso manual. A patrulha não concede conhecimento sobre o jogador.

`npc.patrol` admite pontos explícitos ou um círculo procedural com semente, percurso em loop ou ida e volta, espera, prazo de deslocamento e intervalo entre tentativas de rota bloqueada. `inspect_patrols` mostra rota resolvida e estado; `inspect_navigation` mostra o objetivo/caminho. `python3 scripts/verify_patrol.py` valida a demonstração, interrupção/retomada, geração, replay e falhas. Cena/replay correntes: 10.


**J — caminhos e destinos:** branco contínuo é o caminho calculado; cruz branca é o destino conhecido e vermelha é destino inacessível. Lilás tracejado mostra os pontos/ordem da patrulha autorada. Não é promessa de passagem livre. MCP: `set_debug_view.npc_routes`; captura informa eventual truncamento acima de 256 segmentos por NPC. `python3 scripts/verify_route_guides.py` valida composição/cache, edição, invariância do replay e capturas.


## Cenários verificáveis numa chamada — EAF-14

Após carregar `examples/stealth.scene.json` pelo MCP, execute:

```json
{"command":{"op":"run_scenario","path":"examples/quiet.scenario.json","report_path":"artifacts/quiet-report.json","replay_path":"artifacts/quiet.replay.json"}}
```

O roteiro faz o jogador andar furtivamente, confirma emissão de passos e ausência de memória no NPC. `examples/expected-failure.scenario.json` contém uma expectativa errada proposital: o relatório identifica a falha. Para investigar/retomar patrulha, carregue `examples/patrol.scene.json` e execute `examples/patrol.scenario.json`. Todos foram criados por `save_scenario` pelo MCP.

`run_scenario` parte da autoria em uma simulação isolada e preserva a partida aberta. Até 256 ações e 3.600 ticks; condições tipadas, primeiro erro/timeout e snapshot final. `report.passed` indica o resultado do teste; uma falha de assertion é um resultado válido da ferramenta. Contratos/limites em [commands.md](commands.md), tradeoffs na ADR 0009. Verificação: `python3 scripts/verify_scenarios.py`.

### Diagnosticar um passo gravado

`inspect_replay` consulta o estado de um tick e `capture_replay` produz PNG real desse instante, sem alterar a partida aberta. Por exemplo, após executar o cenário de patrulha e exportar seu replay, consultar tick31 mostra a interrupção por som. O tick final traz a comparação completa do resultado; um prefixo informa explicitamente que não verificou o final. Veja os contratos e limites em `docs/commands.md`.

```sh
python3 scripts/verify_replay_inspection.py
```

### Volumes de gameplay

`examples/triggers.scene.json` demonstra uma região invisível com marcador verde: atravessá-la gera enter/exit sem bloquear movimento. `inspect_triggers` mostra ocupantes e histórico; `examples/triggers.scenario.json` verifica a passagem. A configuração usa upsert e pode ser construída programaticamente; nenhuma regra de quest/vitória é implícita.

```sh
./target/release/ignis --root . --scene examples/triggers.scene.json --window --mcp
python3 scripts/verify_triggers.py
```

### Autoria em lote

`edit_scene` aplica até512 operações como uma única edição reversível. Isso permite criar entidades relacionadas ou renomear um alvo junto de suas referências, sem reiniciar o mundo entre cada operação. Uma falha preserva a partida; um undo desfaz o lote inteiro. Exemplo combinado de furtividade, patrulha e região:

```sh
./target/release/ignis --root . --scene examples/stealth-lab.scene.json --window --mcp
python3 scripts/verify_scene_edits.py
```

O marcador verde delimita uma região observável; ainda não representa vitória/quest. O jogador se chama hero nesse exemplo para demonstrar que controle/percepção dependem dos componentes e referências, não do nome player.

## Seleção e inspeção de entidades

Clique numa caixa para selecioná-la; o contorno dourado acompanha sua posição atual e o título mostra o ID. Clicar no fundo limpa a seleção. O contorno é uma ajuda de inspeção, desenhada sobre a cena, inclusive através de obstáculos; não indica o que um NPC enxerga.

Pelo MCP, `select_entity` recebe `id` ou `null`; `select_at` recebe `x`, `y`, `width`, `height` em pixels físicos da imagem/viewport; `inspect_selection` retorna a entidade autorada, posição atual e seus estados de NPC, navegação, passos, patrulha e volume quando presentes. Uma entidade invisível pode ser selecionada por ID, mas não por clique. Alterar seleção não altera gameplay, autoria, histórico ou replay. Reset preserva um ID ainda existente; remover/renomear limpa, e undo não restaura uma seleção já perdida.

`python3 scripts/verify_selection.py` verifica o contrato e as capturas com janela nativa. O teste automatizado de coordenadas MCP é separado do aceite humano do clique físico.

## NPCs independentes do jogador

`examples/npc-autonomy.scene.json` demonstra um NPC roxo cujo alvo explícito é o NPC azul. O azul não tem alvo e segue sua rotina; ambos podem investigar sons conforme suas configurações. O jogador laranja não é o objetivo visual desses NPCs. Não são ainda facções, amizade ou seleção automática de ameaças.

Uma cena pode conter apenas NPCs: `npc.target` omitido/null mantém rotina/audição e não adquire um jogador implicitamente. Configure um ID de player ou npc distinto para a perseguição sensorial existente. `python3 scripts/verify_npc_autonomy.py` verifica a ausência de player, a investigação sonora, NPC-NPC, referências e replay pelo MCP.

A continuidade prioriza concluir o sistema de NPCs, incorporando dependências necessárias: [frentes e critérios de fechamento](npc-roadmap.md).

Autoria construtiva: [plantas paramétricas](plants.md) geram meshes por perfil retangular cotado e extrusão, preservando escultura e rig/clipes conforme [políticas explícitas](mesh-preservation.md). Edição, preview e inspeção estão no dispatcher/MCP; ferramentas visuais de desenho e modelagem geral são incrementos posteriores.


### Criar um jogo no editor

Ignis Editor abre a tela de projetos. Criar projeto → escolher nome/pasta →
Principal. Painel Projeto cria/abre cenas; Adicionar cria objetos visuais ou
físicos; Propriedades edita/duplica/exclui; Salvar/⌘S persiste. Projeto é a pasta
do jogo; área de trabalho organiza painéis. Veja docs/projects.md e docs/editor.md.
