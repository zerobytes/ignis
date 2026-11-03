# Sistema de NPCs — continuidade e critérios de fechamento

## Extensões futuras — envelhecimento e relacionamentos

Pedido de Guilherme em 01/11/2026: registrar estes dois sistemas para desenvolvimento futuro. Este registro não inicia a implementação nem altera o fechamento técnico do NPC v1.

- **Envelhecimento por indivíduo — EAF-195 (backlog):** idade e evolução configuráveis pelo jogo, com perfis por espécie/tipo e possíveis efeitos em capacidades, atributos, comportamento e apresentação. Relógio, ritmo, fases e consequências serão definidos no contrato técnico; envelhecer não implica obrigatoriamente morrer ou perder inteligência. Separar idade do indivíduo, vida atual, ressurreição e novo nascimento.
- **Relacionamentos entre NPCs — EAF-196 (backlog):** vínculos individuais, além das facções, capazes de influenciar transversalmente as decisões e comportamentos pertinentes: alvos, combate, ajuda, fuga, cooperação, diálogo, rotinas e prioridades. Tipos e dimensões são dados do jogo; confiança, afeição, medo, respeito, rivalidade e parentesco são exemplos. Definir direção/assimetria e evolução por interações e memórias, em conjunto com inteligência e personalidade. Um relacionamento não revela automaticamente posição ou estado oculto de outro NPC.

Ambos devem preservar identidade/persistência sem transferir estado ao novo ocupante de um slot, oferecer controle e inspeção MCP antes ou junto da interface visual e explicar suas influências nas decisões. Avaliar custos e representação antes de implementar, sem exigir atualização por frame nem grafo denso entre todos os NPCs. Não há alteração de código ou formato neste registro.

## Programa EAF-149 — direção30/10

Os nove eixos foram integrados em EAF-150..158, com inteligência e personalidade transversais. Fechamento técnico NPC v1 em Scene 97/replay 107; matriz e limites em [npc-v1-closure.md](npc-v1-closure.md), cenário em [npc-v1-demo.md](npc-v1-demo.md). Aceite humano separado, sem concluir automaticamente M09 ou o roadmap geral. A sequência abaixo registra estados históricos e não reabre o programa concluído.

## EAF-127 — alvos fora dos nós no motor terrestre

Conectores locais walk-only reutilizam corpo/KCC/rede/Dijkstra para entrada/direto/saída e perseguição perceptiva. Orçamento e truncamento observáveis, retentativa sem spam diante de alvo móvel; cast/lifecycle/budgetXYZ/parcial/replay preservados. [Contrato e limites](ground-local-connections.md). Próximo contrato: integração de capacidades terrestres e aéreas por trechos explícitos, com transição/apoio/gravidade e capacidade individual; discutir antes de implementar. Navegação automática, busca incremental completa, multidões, tática e NPCv1/M09 seguem abertos; aceite humano separado.


## EAF-126 — caminhos terrestres inclinados por rede

NPC walk-only com controlador escolhe e executa rota compatível em rede explícita, incluindo rampas/degraus/descida, corpo/teto e desvio mais longo. Mesmo KCC compila ligações dirigidas; cache de perfis compartilhado/preparado para templates, consulta MCP somente leitura e diagnósticos. Vida/retorno/casts/patrulha/memória/budgetXYZ/reuso/replay preservados. Apoio do nascimento continua integral e motores antigos não migram. Próximo: conexões locais limitadas para alvos fora dos nós; depois discutir ground/air e descoberta automática de superfícies. [Contrato](ground-navigation.md). Não conclui navmesh, escala, NPCv1/M09 ou aceite humano.


## EAF-124 — execução terrestre com controlador opcional

Jogador/NPC planar podem usar controlador cinemático configurável: inclinação, autostep, snap, gravidade/queda e push. Estado por indivíduo, morte/ressurreição/reuso, autoria/inspeção MCP e replay. Motor legado permanece padrão e mobilidade espacial atual não migra; contato parcial do motor não muda apoio integral do spawn/planejador. Próximo passo: consultas e planejamento compatíveis com rampas/degraus, antes de prometer perseguição por essas rotas. [Contrato](character-controller.md). Aceite manual separado e NPCv1/M09 permanecem abertos.


## EAF-115 — geometria compartilhada de rampas

Cunha sólida fixa/visual, mesma forma para colisor/render/consultas; espaço vazio não bloqueia visão/som/entregas/voo. Apoio plano e nascimento projetado não aceitam rampa como piso horizontal; motores antigos preservados. Próxima decisão: motor opt-in para jogador/NPC com inclinação, autostep, snap, gravidade e política de empurrões; depois planejamento compatível. Não conclui circulação em rampas/degraus, terreno/navmesh, NPCv1 ou M09. [Contrato](geometry.md), estudo EAF-35 permanece referência de tradeoffs.

## EAF-111 — nascimento com apoio físico

Consulta MCP somente leitura e projeção opcional de geradores sobre primeiro piso fixo abaixo. Apoio modular, tamanho do template e corpos atuais determinam colocação; seed/pending e XYZ legado preservados. Sem descoberta de caminhos ou nova capacidade de movimento. Próximo contrato estrutural: geometria compartilhada e controlador para rampas/degraus, conforme locomotion-controller-study.md; discutir integração antes de migrar motores. Escala, agendas/interações/persistência e apresentação continuam pendentes; NPCv1/M09 não concluídos. Ver [contrato](spawn-support.md).

## EAF-108 — composição de população

Mistura ponderada por gerador/onda com seed própria, pending estável nas falhas e membros/recibos por template. Fontes únicas legadas preservadas; pesos não garantem composição exata. Próximos recortes exigem contrato: seleção/cotas e condições, consulta de apoio/terreno e pré-requisitos de escala. Não conclui NPCv1/M09. Ver [contrato](weighted-population.md).

## EAF-105 — posições de nascimento em região

Geradores sorteiam candidatos dentro de limites de centros XYZ, com altura explícita, seed reproduzível, tentativas limitadas e admissão física existente. Ondas/ativação/manual preservados; seguir com composição de templates e regras conforme contrato, consulta de apoio/terreno e pré-requisitos de escala. Não encerra NPCv1/M09. Ver [regiões](spawn-regions.md).

## EAF-96 — ativação por região

Ocupação de trigger libera produção uma vez ou enquanto ocupado, com suspensão combinada, prazos preservados e identidade de ocupantes validada. EAF-95 recebeu aceite humano. Próximos recortes: seleção procedural de posições/regiões e composição de condições conforme contrato; escala/políticas globais continuam separadas. Não conclui NPCv1/M09. Ver [contrato](generator-activation.md).

## EAF-95 — controle durante a partida

Geradores agora podem iniciar pausados, ativar/retomar, pausar produção e cancelar sem remover sobreviventes. Fila/replay/inspeção e atalhos da demo compartilham contrato. EAF-93 recebeu aceite humano. Próximos recortes: condições de mundo/ativação e seleção espacial procedural; escala exige seus pré-requisitos. Não conclui NPCv1/M09. Ver [contrato](generator-control.md).

## EAF-93 — ondas finitas

Sequência opcional de ondas sobre geradores, com template, quantidade, intervalo e condição de conclusão. Reposição contínua preservada; sem ampliar pool ou concluir NPCv1/M09. Próximos recortes: ativação/pausa/cancelamento em execução e condições de mundo, seguidos de seleção espacial/escala conforme contratos. Ver [ondas](population-waves.md). EAF-92 recebeu confirmação humana antes desta continuação.

## EAF-92 — geradores de população

Geradores por template com pontos candidatos/seed, meta viva, cadência/budget e retirada opcional de cadáver implementados; validação final em validation.md. MCP primeiro e replay derivado, pool8 preservado. Próximos recortes de população: ondas/políticas de ativação e seleção procedural espacial; indexação/planos locais antes de ampliar escala. Isso não conclui NPCv1/M09. Ver [contrato](population-generators.md).

Direção de Guilherme em23/10/2026: manter NPCs como eixo de desenvolvimento até completar o sistema, implementando dependências de outros marcos quando necessárias. A lista geral de render/assets não passa à frente automaticamente. A base atual resolve perseguição, visão, memória, busca, audição, patrulha, objetivos e diagnóstico, com locomoção espacial por rede/conectores e voo curvado. Ainda não conclui M09.

Atualização25/10/2026: frente gráfica paralela autorizada em outro chat; contexto/fronteiras em [graphics-handoff.md](graphics-handoff.md). Mantém aqui a continuidade funcional. EAF-58 recebeu aceite humano (“boa, funcionou”) e foi encerrada. Próximo contrato funcional: instâncias estáveis para spawn/despawn, separando template, indivíduo e vida; discutir geração/armazenamento e retenção da autoria de efeitos antes da implementação, conforme lifecycle-study.md.

Continuidade EAF-70: nascimento por templates e retirada explícita por fila MCP disponíveis, conforme [spawn-templates.md](spawn-templates.md). Estado independente, referências antigas preservadas e ordem por indivíduo; próximo incremento: geradores/reposição de população. Limite8 NPCs+jogador e custos pendentes de escala preservados.

“Completo” precisa de evidência funcional e limites explícitos: uma primeira versão integrada capaz de compor guarda, civil, aliado e criatura, e não uma promessa de cobrir todo comportamento possível. O roadmap abaixo permanece aberto à revisão técnica conjunta; não é uma sequência de dez tarefas pequenas.

| Frente | Entrega verificável | Dependências a trazer quando necessárias |
|---|---|---|
| 1. Autonomia e identidade de alvos | NPC funciona sem jogador/alvo; observa outro NPC sem onisciência implícita. | Referências, ciclo de vida, snapshot comum. |
| 2. Relações e escolha de alvos | Facções/atitudes, elegibilidade, múltiplos candidatos, prioridade e estabilidade da escolha explicáveis. | Identidade de atores e memória por evidência; política configurável. |
| 3. Percepção e conhecimento | Pistas concorrentes, reação/suspeita, memória limitada, origem/incerteza, comunicação explícita entre NPCs. | Eventos com proveniência; privacidade de informações ocultas. |
| 4. Composição de decisões | Guardar, seguir, fugir, investigar, interagir, esperar; prioridades, interrupção, retomada, falha e diagnóstico. | Discutir FSM/árvore de comportamento/pontuação de ações antes de escolher framework; blackboard não vira acesso irrestrito ao mundo. |
| 5. Rotinas e território | Agendas, pontos de interesse, área de atuação, retorno, necessidades e variações procedurais. | Relógio de simulação, regras de interação e geração. |
| 6. Navegação e locomoção ampliadas | Rampas/degraus, portas/passagens, obstáculos móveis, agentes sem bloqueio permanente, recuperação e diferentes capacidades. | Controlador de personagem, consultas espaciais, transformações; discutir substituição da grade plana por representação adequada. |
| 7. Ações, habilidades e combate | NPC executa as mesmas habilidades do jogador: custos, recursos, alcance, cooldown, cancelamento, efeitos e morte. | Estudar runtime do LandOfChaos; habilidades/vida/múltiplas energias/equipamentos. |
| 8. Corpo e apresentação | Movimento/ações perceptíveis em avatar animado, orientação, equipamentos e sons coerentes com o estado. | Assets, hierarquia, animação, avatares e áudio; visual não governa sozinho as regras. |
| 9. Interação social e população | Interações/diálogo com condições, objetivos/quests, spawn/respawn, save/load e reconstrução de estado. | Contratos de interação, progressão e persistência; LLM opcional, não requisito do NPC. |
| 10. Integração, escala e entrega | Cenários completos com guarda/civil/aliado/criatura, combinações e budgets; medir capacidade quando robusto e declarar limites. | Replay, testes, inspeção visual/MCP, controle de custo e aceite humano. |

## Critérios transversais

- Configuração e controle pelo MCP antes ou junto da UI, com inspeção de razões/fontes e estado de cada etapa.
- Proceduralidade para parâmetros/rotinas/distribuições pertinentes, com sementes e proveniência, preservando conteúdo manual.
- Conhecimento limitado por padrão; política onisciente explícita. Mudar posição oculta sem pista não pode mudar crenças/decisões correspondentes.
- Comportamentos reutilizáveis e independentes da espécie/aparência, plataforma de IA e presença de um jogador humano.
- Persistência, mudanças de cena, desativação, remoção, interrupção e retomada têm semântica documentada e testes.
- Orçamentos limitados, revisão estática de performance em cada incremento, sem alegar escala/FPS não medidos.
- M09 só será considerado concluído após os critérios integrados e aceite acordado; completar uma linha não conclui todos os marcos que ela toca.

## Incremento atual

EAF-19 implementou alvo explícito opcional e NPC-NPC. EAF-20 acrescenta facções, atitudes direcionais e escolha estável entre hostis percebidos, opcional por NPC. Nenhuma dessas entregas conclui M09. EAF-21 acrescenta confirmação visual gradual e exposição individual limitada, separada da memória ativa de movimento. EAF-22 acrescenta arbitragem acústica limitada com compromisso, margem, supressão espacial e proteção de memória visual. EAF-23 acrescenta seletor limitado de objetivos, prioridade/condições, resposta à pista, fuga, guarda, espera e patrulha com interrupção/retomada. Sem árvore geral ou utility framework. EAF-23 recebeu aceite humano explícito. EAF-24 acrescenta território circular/retorno comprometido, filtros de pistas e guias, com aceite próprio pendente. EAF-25 acrescentou recuperação opcional de navegação e recebeu aceite humano. EAF-26 acrescenta reserva cooperativa de passagens explícitas: opt-in, prioridade+idade/ID, posse não preemptiva, vagas de espera, prazo/ocupação/gate, retomada do destino e inspeção/replay. EAF-26 recebeu aceite humano explícito. O navegador legado continua plano: encadeamento automático de gargalos, desvios gerais entre agentes, rampas/degraus e alternativas táticas permanecem abertos. Próximo recorte a discutir: ampliar representação/capacidades de navegação antes de prometer circulação geral; comunicação de pistas e ações/habilidades também permanecem no roadmap. Composição de ações e navegação ampliada continuam dependências abertas.


EAF-27 inicia navegação espacial real: rede explícita XYZ, capacidades walk/jump/glide/fly, motores físicos distintos, escolha determinística de caminho, inspeção e replay. Primeira demo com plataformas, voo acima de obstáculo e planeio descendente; aceite próprio pendente. Ainda abertos: geração automática/conectores livres, navmesh/rampas, coordenação de passagens no espaço, rotinas/território/footsteps em múltiplas superfícies, orientação tridimensional de visão/corpo e integração com habilidades. Não conclui a frente6 nem M09.


EAF-28 acrescenta política de aproximação parcial à navegação espacial: stop padrão ou approach_reachable, progresso mínimo e custo estimado limitado, prioridade para rota completa, estado parcial sem sucesso falso e retenção de desvios. Demo cinza caminha até a base da plataforma sem adquirir salto. Aceite próprio pendente. Próximo recorte proposto para discussão: ampliar conexões/superfícies e orientação espacial antes de chamar esse laboratório de navegação 3D geral; o mapa explícito ainda limita destinos alcançáveis. Comunicação, habilidades e integração de rotinas espaciais permanecem abertas.


EAF-29 acrescenta conexões locais walk/fly para entrada, saída e destino livre, com custo completo, colisão/apoio/capacidade e alcance limitado. Rede conserva desvios maiores e saltos/planeios; roxo pode perseguir jogador em posições livres no chão e azul atravessa acima de obstáculo até destino aéreo fora dos nós. Não conclui geração automática de navegação, rampas, controladores gerais ou orientação volumétrica. Aceite próprio pendente. Próximos recortes devem tratar superfícies e orientação/percepção3D antes da integração mais ampla com habilidades e rotinas.


EAF-30 acrescenta voo curvado opt-in por controle limitado, previsão física do trecho, parâmetros MCP de aceleração/giro/suavidade/mínimo/hover e órbita validada para não pairadores. Straight e cenas antigas preservados; demo comparativa de três perfis com objetivos fixos. EAF-29 recebeu sinalização de aceite do usuário; EAF-30 recebeu aceite humano explícito (“ficou perfeito”). Ainda faltam orientação visual/volumétrica, superfícies/rampas, rotinas espaciais e navegação global com estado dinâmico de voo.


## Situação consolidada e próximo fechamento (EAF-32)

A quantidade de incrementos não mede a fração concluída: habilidades, interação e apresentação ainda são dependências grandes. Não há estimativa percentual ou prazo defensável antes de detalhar esses contratos. O núcleo de percepção/decisão/locomoção já é utilizável; a primeira versão integrada do sistema de NPCs ainda não está na fase de últimos ajustes.

| Frente | Situação atual | Evidência que ainda falta para fechar a v1 |
|---|---|---|
| Autonomia/identidade | Implementada no recorte atual | Revalidar remoção, morte e respawn quando ciclo de vida chegar. |
| Relações/alvos | Implementada no recorte atual | Integrar elegibilidade a vida/ações, mantendo conhecimento limitado. |
| Percepção/conhecimento | Cone volumétrico e rádio opt-in com origem/idade (EAF-36) | Aceite do rádio; integrar sensores aos diferentes corpos e definir outros meios de comunicação quando necessários. |
| Decisões | Seletor com interrupção/retomada funcional | Executar ações/interações com sucesso, falha e cancelamento, além de locomoção. |
| Rotinas/território | Patrulhas XYZ em EAF-33; território permanece plano | Território espacial, pontos de interesse e necessidades com orçamento; acordar quais entram na v1. |
| Navegação/locomoção | Rede XYZ, conexões locais, salto/planeio/voo curvado e recuperação | Superfícies/rampas/degraus, integração espacial das passagens e circulação combinada. |
| Ações/habilidades | Executor compartilhado, custos/cooldown, cast/cancelamento e vitalidade/dano/cura/morte (EAF-38/39) | Ressurreição/respawn, entregas físicas/área, efeitos persistentes e política composta de aproximação/ação. |
| Corpo/apresentação | Caixas e guias de diagnóstico | Avatar/orientação/animação e sons coerentes com estados e ações. |
| Interação/população | Ainda não implementada | Interação condicionada, spawn/respawn e persistência de estado; não exige engine completa de quests/diálogo. |
| Integração/escala | Regressões determinísticas e headless | Guarda, aliado, civil e criatura em cenários integrados; budgets medidos e aceite humano. |

Sequência de engenharia: (1) visão/orientação sensorial3D; (2) superfícies e rotinas espaciais; (3) ações/habilidades e ciclo de vida; (4) comunicação/interação/população; (5) corpo/apresentação e integração final, antecipando dependências quando necessárias. Cada bloco exige mais de um incremento; a ordem fina e os contratos estruturais serão discutidos antes de implementar.

EAF-31 tornou operação MCP sem janela o padrão e isolou os outputs de regressão. EAF-32 mantém essa regra: cone circular XYZ opcional, yaw/pitch, orientação independente ou acompanhando velocidade comandada, alcance/oclusão existentes e guia volumétrica. Preserva sensor horizontal legado. Não gira mesh/corpo, não fornece varredura vertical automática nem visão de partes do alvo; a amostra continua sendo o centro do corpo. Aceite visual próprio permanece separado.

O fechamento de NPC v1 exige demonstrar os quatro papéis usando os mesmos módulos: percepção limitada e pistas compartilhadas explicitamente; rotinas interrompidas/retomadas; movimento e ações sem sucesso falso; remoção/morte/respawn persistentes; apresentação legível; autoria/inspeção/replay pelo MCP e execução sem agente. Suporte universal a qualquer jogo permanece visão de longo prazo, não critério verificável deste marco.


EAF-33 integra patrulhas XYZ: pontos e anéis em plano configurável, geração na edição, motores físicos/capacidades existentes, espera/loop/ping-pong, falha/cooldown e retomada após pista. Corrige confirmação prematura de aterrissagem durante acomodação do contato. Caminhante não adquire salto e planador não adquire subida; voador sem hover espera orbitando. Guias/inspeção/replay/MCP mantêm altura. Implementação e regressão são gates separados do aceite humano.

Próximo recorte proposto: superfícies caminháveis e transições entre alturas (rampas/degraus), com decisão explícita sobre representação de geometria/colisão e controlador antes de codificar. Atualmente a representação por caixas sem rotação não descreve uma rampa. Depois, integrar território/coordenação às superfícies e avançar para ações/habilidades e ciclo de vida. Patrulha espacial conclui este recorte de rotina, não todo o sistema de NPCs.


EAF-34 remove a exigência de uma só caixa suportar um trecho de caminhada: união contínua de topos fixos, área varrida completa, junções/diagonal/largura/gaps pequenos, limite64 peças relevantes e diagnóstico. Links/conectores/apoio pontual usam a mesma consulta; demo parametrizada, remoção/undo e replay pelo MCP. Esta é a base de pavimentos modulares planos, não rampas/degraus ou navmesh. Próxima decisão estrutural de geometria e controlador detalhada em architecture.md; não foi tomada silenciosamente. Aceite humano próprio permanece separado.

EAF-35 concluiu um experimento físico isolado de11cenários com33combinações de motor, repetidas: rampas20°/40°, degraus20/50cm, teto, parede, queda e impulsos. Evidencia viabilidade funcional do controlador cinemático e custos de integração; não acrescenta rampas à cena/NPC. Recomendação detalhada em [locomotion-controller-study.md](locomotion-controller-study.md): geometria compartilhada caixa/cunha, depois motor de personagem opt-in e consultas de navegação compatíveis. Escolha de produção permanece proposta para discussão.207testes Rust e14grupos MCP do runtime existente aprovados; M09 e aceites visuais anteriores permanecem abertos.


## Comunicação e continuidade (EAF-36)

Integração de rampas/controlador ficou para outro momento por direcionamento do usuário. Rádio opcional agora transmite visão confirmada entre aliados mútuos, por canal/alcanceXYZ/intervalo/idade, com um tick de latência, origem preservada e nenhuma retransmissão. Receptor investiga uma posição congelada; visão, audição e reconhecimento continuam independentes. Tudo autorado/inspecionado via MCP e testado sem janela. EAF-36 recebeu aceite humano explícito (“excelente. Comportamento fantastico”) e foi encerrado; não conclui a frente3 nem NPCv1.

Próximo contrato a detalhar: ações/habilidades compartilhadas entre jogador e NPC, com resultados, custos, cancelamento e ciclo de vida. Antes de codificar, estudar o runtime do LandOfChaos e discutir o recorte, conforme M08; não deduzir uma abstração completa a partir do catálogo/HUD. Gritos, fusão/reputação, território espacial, rampas e apresentação continuam abertos.


## Contrato de ações/habilidades preparado (EAF-37)

[Estudo do runtime de referência e proposta](ability-runtime-study.md) concluído a partir de16 arquivos identificados de LandOfChaos7b97893, sem mutar aquele projeto ou incorporar GAS. Distingue autoria de294 programas de execução efetiva. Recomenda definição tipada/instância, recursos genéricos por unidades inteiras, commit atômico, cast/cancelamento/cooldown e entrega direta antes de grafo/eventos/projéteis.

Primeiro incremento proposto: converter recursos próprios, energizar aliado observado e sinalizar por ruído, mesmos operadores/definições para jogador e NPC, configuração/inspeção/replay pelo MCP. Matriz de aceitação e limites no estudo. Números, cobrança e concorrência são contratos para alinhamento técnico; runtime/cena/replay25 permanecem inalterados. Habilidades não estão implementadas por concluir este estudo, e M08/M09 continuam parciais.


## Primeiro runtime de habilidades (EAF-38)

Autorizado recorte A após o estudo: executor comum jogador/NPC, recursos inteiros/escala, custos/efeitos atômicos, cast/cancelamento/cooldown, condição de recurso próprio, retry e visão confirmada para alvo de NPC. Demo converter/aliado/sinal em `npc-abilities.scene.json`; comandos, limites e teste humano em [abilities.md](abilities.md). Implementação/validação automatizada e aceite humano são gates distintos. M08/M09 continuam parciais; próximo contrato é vida/morte e resolução de dano/cura, antes de projéteis/efeitos persistentes.


## Vitalidade integrada às ações (EAF-39)

EAF-38 recebeu aceite humano e foi encerrado. Contrato de vitalidade aprovado antes da implementação: um componente vital independente, dano/cura atômicos, morte ao final de cada execução e ordem sequencial existente. Morte interrompe ações/motores/reservas, mantém corpo físico e não revela morte oculta a outros NPCs. Demo atacante/curador/alvo passivo e [contrato](vitality.md) disponíveis via MCP. Aceite humano deste incremento permanece separado da automação.

M08/M09 continuam parciais. A continuidade de condições vitais e aproximação foi recortada em EAF-40 abaixo. Entrega física/área, efeitos persistentes e ressurreição/respawn permanecem posteriores; não introduzir esses contratos implicitamente.

## Política de habilidades e aproximação (EAF-40)

EAF-39 recebeu aceite humano explícito e foi encerrado. Continuidade: `use_ability.policy` opt-in, condições vitais próprias/alvo visual confirmado, custos/alcance e aproximação pelos motores existentes, com margem configurável e decisões por objetivo no MCP/replay28. Contrato, alternativas e demo em `npc-ability-policy.md`. Ausência de política preserva legado; não fornece vida/posição oculta, capacidade de voo ou caminho garantido.

Recorte mantém o seletor de prioridades e o alvo sensorial existente. Ainda falta triagem de múltiplos candidatos, conhecimento de lesões configurável, posicionamento tático/linha de tiro, entrega física/projéteis/área, efeitos persistentes e ciclo de respawn. Esses contratos não são inferidos de cura/aproximação; M08/M09 seguem parciais. A demo própria precisa de aceite humano após validação automatizada.


## Movimento durante habilidades (EAF-41)

EAF-40 recebeu aceite humano explícito ("ficou muito bom"). Novo recorte autorizado: free, interrupt e lock configuráveis por habilidade, executor comum, suspensão/retomada de patrulha e bloqueio aéreo incompatível diagnosticado. [Contrato e demo](cast-movement.md). Automação headless e aceite humano da demo permanecem gates separados. M08/M09 continuam parciais; não fecha canalização, entrega física/área, efeitos persistentes, triagem ou respawn.


## Entrega por projétil (EAF-42)

EAF-41 recebeu aceite (“boa, pode seguir”) e foi encerrado. Próximo recorte implementado: lançamento pago, voo XYZ retilíneo sem homing e impacto posterior, compartilhados por jogador/NPC. Corpo bloqueia independentemente da elegibilidade; erro não reembolsa e morte do emissor não apaga disparos. [Contrato e limites](projectiles.md). Atirador da demo reage à visão atual para lançar, sem prever desvio nem consultar posição oculta em voo. Não acrescenta decisão de esquiva/tática do NPC. Aceite humano próprio permanece separado de automação; M08/M09 parciais, sem área/efeitos persistentes/canalização/respawn.


## Mira livre (EAF-43)

EAF-42 recebeu aceite humano. Selection spatial permite ordens por direção/ponto XYZ no executor compartilhado, sem ator selecionado, mantendo NPC automático por alvo confirmado. Mira congelada no pedido, origem atual no lançamento e efeitos apenas no contato; API de cursor/plano disponível no MCP junto do teclado. Contrato/teste em [free-aim.md](free-aim.md). A habilidade espacial pode ser concedida a NPC, mas sua decisão automática baseada em pistas/antecipação requer contrato posterior; objetivos use_ability espaciais são rejeitados explicitamente. Este incremento não conclui tática, triagem, efeitos persistentes, área ou M08/M09.


## Mira automática por observação (EAF-44)

Continuidade autorizada após EAF-43, sem inferir confirmação de teste manual da demo anterior. `use_ability.policy.spatial_aim: observed_point` liga habilidades spatial à evidência atual, congela ponto/proveniência ao iniciar e cancela ao perder visão/identidade/alcance/condição. Aproximação opt-in usa alcance próprio. NPC vermelho e azul com modos distintos em npc-aim.scene.json; marcador pelo MCP/J. Contrato em [npc-aim.md](npc-aim.md). Não conclui antecipação, tática de esquiva/cobertura, pistas anônimas, triagem, efeitos persistentes, área ou M08/M09.


## Efeitos periódicos (EAF-45)

Continuidade após EAF-44, sem inferir novo aceite manual. Agenda opcional de dano/cura/recursos, mesma entrega/executor para NPC/jogador, primeiro pulso atrasado, custo único e reaplicação refresh/independent. Morte do destinatário encerra; perder visão ou morrer a origem não remove o efeito entregue nem revela posição à IA. Demo npc-periodic, contrato [periodic-effects.md](periodic-effects.md). Automação e aceite humano separados. Efeitos persistentes agora têm este recorte; modificadores, dissipação, canalização, área, respawn e M08/M09 permanecem abertos.


## Dissipação e resposta por habilidade (EAF-46)

Categorias periódicas opcionais, antídotos/remoções como efeitos transacionais, entrega direta/projétil, limites/require_match e proveniência pelo MCP. NPC azul usa antídoto próprio pelo mesmo executor/retry que o jogador; não é triagem de aliados, reconhecimento geral de status ou imunidade. Contrato/demo em [dispel.md](dispel.md). Continuidade após EAF-45 não comprova novo teste manual; automação e aceite separados. Modificadores, canalização, área, respawn e integração de NPCs seguem abertos; M08/M09 parciais.

EAF-46, extensão aprovada: proteção explícita `dispellable:false` por definição periódica, independente de categoria. Executor comum ignora protegidos no limite/require_match; self-antídoto NPC não inicia/paga por veneno apenas protegido. Demonstração: vermelho aplica veneno protegido, jogador aplica veneno removível ao azul. Não introduz classificação tática/triagem de aliados ou novos sentidos; aceite manual próprio continua separado.

## Canalização sustentada (EAF-47)

Continuidade autorizada após proteção de efeitos, sem inferir novo aceite manual. Execução mantém o ator ocupado e revalida condições durante pulsos diretos; custo na entrada ou por pulso, cancelamento e políticas de movimento existentes. NPC pré-pago continua com saldo zero, perda visual encerra e guia ciano é diagnóstico. Contrato/demo em [channeling.md](channeling.md). Não conclui modificadores, área, triagem, respawn, corpo/apresentação ou M08/M09. Aceite manual próprio permanece separado.

## Interrupção externa (EAF-48)

Execução atual de outro ator pode ser interrompida transacionalmente, com proteção por definição e entrega direta/projétil. NPC só consulta candidato sob visão atual e usa retry existente; sem silêncio por duração ou nova tática. Demo npc-interrupt permite interromper o roxo, comparar azul protegido e receber interrupção do vermelho. Contrato em [interrupts.md](interrupts.md). Continuidade não implica aceite manual da canalização; automação e aceite próprio separados. Área, modificadores/controle, triagem, respawn e integração permanecem abertos; M08/M09 parciais.

## Atributos e buffs temporários (EAF-49)

Guilherme ampliou explicitamente a abstração para atributos arbitrários por chave/operação/valor. Registro extensível bool/número, filtros/tags, duração/reaplicação e avaliação efetiva; primeiros consumidores: interrupção, dissipação de DoTs por autoria e dano recebido. NPC compartilha executor, percepção e retry; não ganhou tática de escolher buffs por contexto. Demo npc-attributes e contrato [attributes.md](attributes.md). Outros adaptadores/tipos, área, triagem, equipamentos e respawn continuam abertos. Continuidade não implica aceite manual de EAF-48; EAF-49 também tem gate humano separado. M08/M09 parciais.

EAF-50: dano composto compartilha executor de jogador/NPC, contexto perceptivo, projétil/pulso/canal. Não adiciona seleção tática de elementos ou buff; validação headless e aceite humano da demo continuam separados. Contrato em damage-components.md.

EAF-51: dano causado consulta atributos do autor original no mesmo executor, inclusive NPC, com contexto por componente e momento dinâmico de impacto/pulso. Não acrescenta escolha tática de buffs/elementos; triagem, área, respawn e M08/M09 seguem parciais. Contrato em [outgoing-damage.md](outgoing-damage.md); automação e aceite físico separados.

## EAF-52 — área instantânea no executor compartilhado

Acrescenta aplicação coletiva física com seleção limitada/explicável e efeitos por destinatário. NPC usa as mesmas habilidades e continua mirando evidência visual congelada; atingir secundários não revela sua localização à decisão. Não entrega seleção tática de grupos, triagem de aliados, áreas persistentes ou conclusão M08/M09. Contrato em instant-area.md; evidências e gates em validation.md.

## EAF-53/EAF-54 — zonas persistentes e auras móveis

EAF-53 entregou zonas fixas com pulsos e recebeu aceite humano explícito. EAF-54 acrescenta âncora source: NPC pode ativar aura própria pelo mesmo executor e retomar locomoção/patrulha durante cooldown; esfera acompanha corpo XYZ sem revelar destinatários secundários. Demo npc-aura combina portador jogador, NPC voador aliado e alvos resistentes. Contrato em auras.md. Não acrescenta triagem/seleção tática de habilidades, buff contínuo por ocupação, respawn ou fechamento M08/M09; aceite humano próprio separado.

## Apoio observado (EAF-55)

EAF-54 recebeu aceite humano. Triagem opt-in por objetivo actor: menor fração vital/proximidade, inclusão própria, filtros antes de ranking, compromisso de cast e evidência independente de combate. Demo npc-triage combina curandeiro, aliados feridos e aliado oculto. Contrato em ally-triage.md. Não fecha tática geral, reserva coletiva, triagem por status, respawn ou NPCv1/M08/M09; aceite humano próprio separado.

## Apoio por efeito removível (EAF-56)

EAF-55 recebeu aceite humano. Filtro require_dispel_match por objetivo permite priorizar aliados observados com instâncias removíveis pela habilidade, incluindo proteção efetiva e buffs; vida cheia não impede apoio. Contrato em dispel-triage.md. Preserva ranking/cast/executor e revalida no commit. Não implementa doenças ocultas, severidade, reserva coletiva ou tática de área; NPCv1/M08/M09 seguem parciais. Aceite humano próprio separado.
## Próximo contrato — ciclo de vida (EAF-57)

[Estudo de ressurreição e respawn](lifecycle-study.md) audita identidade, morte, conhecimento, ações persistentes e replay. Recomenda começar por ressurreição explícita do mesmo indivíduo, preservando recursos/cooldowns e exigindo nova evidência dos observadores; contador de vida evita reaproveitar intenções antigas. Spawn de população exige identidade de instância e geração antes de reutilizar slots. Estudo concluído não significa implementação ou aprovação dessas escolhas. EAF-56 continua com aceite próprio pendente; NPCv1/M08/M09 permanecem parciais.

## Ressurreição explícita — EAF-58

Recorte de EAF-57 aprovado: mesmo indivíduo/corpo, recursos e cooldowns preservados, contador de vida e reativação transitória. NPC usa habilidade no cadáver do alvo explicitamente configurado com visão confirmada; observadores redescobrem por evidência, sem aviso global. [Contrato/demo](resurrection.md). Não conclui triagem/aproximação de cadáveres, spawn/despawn, savegame, NPCv1 ou M08/M09; aceite humano separado da automação.

## Instâncias e população — estudo EAF-59

EAF-58 recebeu aceite humano. [Proposta detalhada](population-runtime-study.md) inventaria dependências de slots/índices, inclusive atributos, ruídos, históricos e consultas/render. Recomenda pool com geração, identidade pública por execução e vida separada; contexto de origem limitado para entregas sobreviventes; fila de população antes dos sensores e spawn inicial apenas de NPCs. Quatro fases: referências, mundo presente separado da autoria, operações explícitas, gerador procedural. Ainda exige alinhamento das políticas de origem/remoção; estudo concluído não significa implementação de spawn ou aprovação de ECS/ABI/limites novos. Frente gráfica deve coordenar Scene/RenderFrame e versões, conforme graphics-handoff.md.


## Identidade — primeiro incremento EAF-60

Direção EAF-59 aprovada após discussão de custo/carga. Registro determinístico com geração e instância, vida separada, inspect_instances e tokens escopados nos controles de habilidades pelo MCP. Fila/cast revalidam referências; contato físico novo usa destinatário corrente. Ressurreição preserva o indivíduo. [Contrato](instance-identity.md), cena46/replay47. O catálogo não recicla slots ainda; continuar migração de atributos, percepção/rádio, ruídos/históricos/contatos/trigger e visão do mundo antes do spawn/despawn. Contexto de origem compartilhado requer reserva na admissão de entregas; escala exige atributos por dono/chave e planos transacionais locais. Não conclui população, capacidade601, NPCv1/M08/M09.


## Vínculos persistentes — EAF-61

Buffs, observações/reconhecimento/rádio e históricos receberam identidade de instância; pista antiga permanece congelada sem virar alvo novo. Som anônimo preservado. [Contrato](identity-bindings.md), cena46/replay48; reuso isolado testado, sem reciclagem pública do mundo. Próximo passo: separar consultas do mundo presente da autoria e integrar contexto de origem/bindings antes das operações transacionais de spawn/despawn. Indexação por dono/chave e planos locais continuam pendentes para escala. NPCv1/M08/M09 permanecem parciais.


## Mundo presente e origem de entregas — EAF-63

RuntimeWorld separa definições/placements da autoria; ausência não recupera posição autorada. Entregas congelam definição e compartilham contexto reservado antes de cobrança. Origem permanece dinâmica enquanto presente; retirada conserva bases/contribuições e prazos, identidade/facção/vida. Áreas fixas podem sobreviver, auras encerram, histórico não retém reserva. [Contrato e limites](runtime-world.md), MCP inspect_world/inspect_origins, cena46/replay49.

Próximo: transação pública de spawn/despawn e instalação dos componentes/placements com fila/replay e invalidação. A seam de testes já retira corpos; não é API pública de população. Atributos por dono/chave e planos locais continuam necessários antes de aumentar o limite9. Sem fechamento de NPCv1/M09.


## Nascimento explícito por templates (EAF-70)

`set_npc_templates`/`inspect_templates`/`spawn_actor` complementam despawn, com identidade nova e inicialização independente, inspeção e replay pelo dispatcher/MCP. Contrato em [spawn-templates.md](spawn-templates.md); vida0 e recursos/cooldowns novos distinguem nascimento de ressurreição. Até16 templates compartilhando8 slots de NPC; autoria e sobreviventes preservados. Validação de volume/apoio e ordem por indivíduo precedem geradores. Próxima etapa funcional: política de geração/reposição procedural com seed, posições e budgets explícitos, construída sobre esta API. NPCv1/M09 continuam parciais; não fechar escala/multiplayer, geradores ou decisões de rampas por inferência.
