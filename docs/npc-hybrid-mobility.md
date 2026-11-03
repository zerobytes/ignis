# Mobilidade híbrida dos NPCs — EAF-149

Scene90/replay100, após cognição89/99; protocolo2 preservado. Opt-in continua pela presença de character_controller e capacidades npc.mobility. Walk obrigatório para esse controlador; jump/fly/glide adicionais aceitos só na90. Local fly exige capacidade correspondente. Sem flag que habilite poder ausente.

## Decisão estrutural antes do código

Reutilizar KCC para walk e motores dinâmicos existentes para jump/fly/glide. Alternativa de tornar KCC um controlador aéreo universal criaria nova integração/colisão/balística; trocar todo movimento para dinâmico perderia rampas/autostep. Trocar tipo do mesmo corpo por modo conserva pose/colisor/identidade, sem substituir indivíduo ou reconstruir mundo. Velocidade é aplicada após mudar tipo. Caminhada retomada usa velocidade vertical atual e solver KCC, não teleporte ao piso.

Mobility expõe último motor e controlador expõe airborne. Decolar limpa grounded/contato obsoleto; corpo aéreo não passa pelo movimento KCC. Pouso exige contato/aproximação do executor; hover/orbit explícitos continuam no ar. Ausência de hover não é chegada estacionária. Morte mantém corpo dinâmico, retorno reinicia motor e novas evidências conforme contratos anteriores. Lock durante trecho aéreo comprometido segue recusa atual; cast em chão mantém trecho/prazo. Gravidade/solver continuam ativos segundo modo.

Walk em rede é compilado e lançado pelo mesmo KCC. Outros links conservam validação atual de capacidade/volume/apoio e trajetória. Conector local avalia walk e fly habilitados, sem confundir falha de apoio com autorização para voar. Inteligência limita sondagens; cautela/medo/coragem influenciam custo preferencial aéreo, separado de duração/distância física/custo pago. Preferência não pode criar ligação nem atravessar sólido.

## Performance antes da implementação

1. Troca de modo só ao mudar motor, sem criar corpos/mapas/Scene por tick.
2. State recebe enums/bool pequenos, sem novo pool/worker/lock; reutilizar scratch KCC e BVH atuais.
3. Queries estáticas/preparação por perfil mantidas; apenas o link lançado é revalidado.
4. Custos de preferência inteiros e separados dos Motion físicos; Dijkstra limitado64/128, mesma ordem/desempates.
5. Tetos de probes600/2400 e cache8mantidos; voo usa validação já limitada, sem pesquisa global extra.
6. Render/GPU/I/O não mudam; inspeção/replay/captura sob demanda.
7. Não inventar percepção/teleporte/apoio; estados de morte/cast/identidade são fences imediatos.
8. Spawn/retirada/revive/reset/load recompõem modo, sem estado aéreo herdado.
9. Nova prova somente para integração física/identidade/replay; verificação existente por impacto.
10. Contato parcial KCC e custos internos Rapier mantidos, sem FPS/navmesh/multidões/escala alegados.

## Fronteiras físicas

Planeio não fornece sustentação para decolar sozinho: o trecho começa já livre da borda/no ar, conforme motor anterior, e só desce. Pousos de jump/glide continuam exigindo apoio completo plano admitido pelo validador desses motores; rampas são percorridas pelo KCC. A sequência integrada walk→fly→walk e walk→jump→walk, mais glide→walk, não é uma promessa de decolagem arbitrária de qualquer borda. Conectores híbridos aceitam nó aéreo só na posição física precisa, com clearance e capacidade fly; sem esse caso o antigo requisito de apoio criava um ciclo descer/subir. Estados/custos físicos não são substituídos por pontuação mental.
