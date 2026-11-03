# Navegação terrestre por controlador — EAF-126

Reutilizar a rede espacial64nós/128links e o Dijkstra existentes. Entity.character_controller + npc.mobility.walk:true habilitam motor terrestre na rede; jump_speed/fly_speed/glide e local_connections recusados juntos neste recorte. Omissão mantém todos os motores e validadores antigos. Cena84/replay88 após união do editor83/87, sem importar capacidades isoladas.

A extensão cena85/replay89 habilita conectores locais walk-only: consultar [contrato EAF-127](ground-local-connections.md). As exclusões de conexões locais abaixo descrevem o recorte84 original.

## Admissão e execução

Links walk podem variar em XYZ na cena84 quando há consumidor terrestre opt-in. Nós são posições dos pés; corpo permanece caixa alinhada. Geometria fixa e parâmetros do mesmo Rapier0.30.1 KinematicCharacterController validam cada ligação dirigida; até600passos fixos (10s), sem impulso, RNG ou avanço da partida. Velocidade vem de npc.speed. Compilação usa pés do nó mais offset do controlador, representando a pose com skin; consultas explícitas e revalidação de lançamento usam os pés físicos fornecidos, sem teleporte ou alteração do nó. Caminhar requer avanço horizontal, contato e chegada física próxima ao nó; autostep pode levantar o corpo antes do apoio do topo e recebe tolerância máxima de3ticks exclusivamente após subida de degrau, devendo recuperar contato antes da chegada; não permite saltar/fazer queda livre para concluir um trecho. Descida de degrau precisa snap compatível; gaps que o contato parcial consegue cobrir seguem política física do controlador, não apoio integral do spawn. Posição inicial dentro de sólido ou sem chão é recusada. Autoria no meio de rampa deve acomodar a caixa, não apenas tocar seu centro na superfície.

Mundo estático de consultas e cache por tamanho/velocidade/configuração são criados na sessão; templates aquecem o cache antes do nascimento. Relatório de custo/distância/motivo por link alimenta a busca existente. Ao lançar, repetir só o link escolhido da posição física atual. Durante execução, intenção XZ vai ao controlador comum, gravidade/colisão resolvem Y; confirmação usa corpo e contato, nunca contador de tempo como sucesso. Pausa por cast preserva trecho e congela seu prazo; percepção/vida/memória continuam.

query_ground_traversal recebe entity/start/end explícitos em coordenadas de pés, é somente leitura e retorna status/ticks/distância/final_feet. É consulta de geometria fixa, não garante passagem diante de outros corpos, reserva ou conhecimento do alvo. inspect_mobility mantém estados/custos e inclui admissões estáticas. Probes não usam posição oculta de player/NPC; objetivos continuam recebidos da percepção e congelamento existentes.

## Fronteiras e alternativas

Preferido a acrescentar outro planejador ou aprovar rampa apenas por ângulo. Usa o mesmo controlador da execução, mas seu mundo de consulta contém apenas geometria fixa: corpos dinâmicos podem bloquear/reter o trecho e levar a timeout/retry. A validação estática não é promessa de circulação. Cache compartilhado reduz custo de nascimento ao preço de preparação finita mais cara; Rapier ainda aloca manifolds internamente. Não certificar escala1000 ou FPS.

Conectores livres, navegação automática/navmesh, alternância ground/air e plataformas móveis são contratos seguintes. NPC usa inicialmente nós autorados/gerados via MCP, com alvo conhecido próximo a um nó. Apoio integral de nascimento/projeção e caminhada dinâmica legada não é relaxado. Render/Light/Vertex/Globals preservados. Morte/ressurreição/despawn/reset/undo/replay conservam contratos existentes.

## Observabilidade e custo

NPC usa npc.speed; jogador sondado usa locomotion.walk.speed. Velocidade vertical inicial zero; ticks são passos virtuais e não tempo de CPU. Até600passos de movimento mais2checagens de contato por probe; árvore e geometria não mudam. Forma/caixa de consulta ficam no stack, mas Rapier usa manifolds internos. No máximo24perfis autorais (8NPC+16templates), com128relatórios por perfil; nascimento só encontra perfil preparado e copia os bindings/Arc. Preparação pode ser cara em cenas no teto e deve receber budgets/medição próprios antes de escala maior. Não transferir esse custo para toda a rede por tick.

Links não autorizados para cada indivíduo não entram na busca. A ligação lançada é revalidada da posição física atual; a consulta não assegura o comportamento de outro perfil/velocidade inicial ou de corpos móveis. Percepção fornece o destino, mapa fornece travessias, nenhum dos dois inventa observação. A demo usa destinos fixos: roxo30graus toma o desvio verde (~20graus), azul45graus usa vermelha curta (~40graus); cinza com autostep30cm recusa degrau50cm. P inicia, WASD jogador, R reinicia; guias J são diagnósticos. Não é perseguição livre fora da rede.
