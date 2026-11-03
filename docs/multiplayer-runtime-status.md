# Base do runtime e multiplayer

Estado após EAF-145/148, Scene88/replay98. O núcleo local executa os cenários de
1.000Players abaixo. Single player offline permanece obrigatório, sem rede,
conta, API proprietária ou processo separado de servidor.

| Camada | Evidência atual | Limite / próximo contrato |
|---|---|---|
| Armazenamento/identidade |1000Players,1024slots,4096entidades, referências/gerações/vidas preservadas |8NPCs; não1000NPCs |
| Controle/admissão |1000vínculos, até16pedidos/ator/tick, FIFO/cancel/posse/replay |uma ação efetiva por ator/tick; transporte não pode virar admin |
| Transações |writes locais16recursos/2atores de vida; rollback tardio |não clonar mundo para planos |
| Entregas |projéteis64/periódicos64/áreas16por fonte e tetos reais; origem32/slot |limites de armazenamento, não pior caso sustentado |
| Buffs/bases |quota fonte/dono=max(64,slots), índices, bases65por entidade/template, snapshot reservado |O(M) denso no refresh/composição real; máximo1mcontribuições não medido |
| Ruído/passos |fila completa proporcional, anéis separados, fontes1000, ouvintes8 |Noise só direto/canal; não áudio reproduzido nem1000ouvintes |
| Execução |continuous consome filas e não acumula histórico |recorded finito; checkpoint persistente de mundo ausente |
| Controle terrestre |1000KCC/grounded/passos; contato SAT inicial comum a planner/runtime |não prova gargalos/navmesh/crowd avoidance geral |
| Carga conjunta |local mixed p95sim6,022ms/chefe14,300ms; rede p95total7,449/17,778ms |receitas especificadas; chefe em rede não assegura60Hz |
| MCP/render/replay |1000ações reais,1000commits, replay e PNG iguais offscreen |não FPS, teclado/aceite humano ou jogo artístico |
| Sessões/réplica |1000posses, operação16/ledger64, resume10s, full/delta atômico, owner privado |transitório, sem conta/checkpoint durável |
| Rede QUIC |1000TLS/100mil ações mixed e100mil no chefe,1000KCC/passos; pin/loss/dedup/resume/TLSretry/ACK monotônico |loopback; checar bytes/recursos/lag, sem WAN/produção |
| Cliente |conteúdo público/props/meshes e janela/capture remota; GPU1000atores/1005boxes |hold100ms, picking/seleção/slots/cancel/aim nativos+MCP; sem predição/interpolação |

Contratos e receitas: scalable-actor-runtime.md, continuous-runtime.md,
frame-admission.md, combat-broadphase.md, source-delivery-pools.md,
scheduled-source-pools.md, area-selection-runtime.md, scalable-attributes.md,
combat-noise-runtime.md e joint-runtime-load.md. Cada documento registra seu
build/gates e custos mantidos. Benchmarks são cenários especificados; não são
promessa para todas habilidades/vítimas/corpos/obstáculos possíveis.

Controles remotos em remote-controls.md: posse local escopada, fence de vida e
conexão, auto-ID/admissão e renovação fora do lock da UI. Conteúdo público e
apresentação em remote-presentation.md, sem simular a autoridade no cliente.
Cargas conjuntas/reprodução/diagnósticos em joint-network-load.md: gerador4workers/
32endpoints, todas1000réplicas terminaram no mesmo tick,1tick atrás; durante combate
lag estimado p95250ms mixed/100ms chefe, com custos/limites explícitos.
A base real de transporte está em quic-transport.md; sessões/codec em
network-session-runtime.md. Offline continua requisito permanente.


A base de escala, transporte real e controles deste escopo estão implementados e
validados nos cenários acima. Etapas de produção continuam separadas: checkpoint
persistente/contas/posse durável, WAN e interpolação/predição. Nenhuma dependência
remota foi imposta ao modo single player; nenhum aceite físico/FPS inferido.
