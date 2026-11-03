# Tática e cooperação — contrato EAF-154

Scene93/replay103, protocolo remoto2 preservado. Objetivo `tactical` no seletor;
`order_npc`, `order_npc_group`, `inspect_npc_tactics` e
`inspect_npc_cooperation` no dispatcher compartilhado. `npc.cooperation`
configura group/role numéricos definidos pelo jogo, formação e reserva opt-in.

Decisão antes da implementação: continuar com o seletor de objetivos e os
executores físicos/de habilidades atuais. Um objetivo tático produz um destino;
uma ordem produz intenção de movimento; uma reserva de apoio regula intenção
automática. Nenhuma dessas camadas pode aplicar dano, curar, teleportar ou
consultar a posição atual de um inimigo escondido por conta própria.

Alternativas: behavior tree separado duplicaria vida/percepção/cancelamento;
planejador global de esquadrão teria conhecimento implícito excessivo. Preferimos
decisões locais sobre recibos de percepção e comandos explicitamente entregues.
Mapa de navegação é conhecimento autorado, não posição de inimigo.

Posicionamento: alcance desejado/tolerância/cobertura/flanco configuráveis por
objetivo. Candidatos vêm de pontos do mapa e de um anel em torno da pista
congelada; nenhum ajuste da câmera/debug participa. Inteligência limita esforço
e cadência; cautela/medo/vida própria preferem distância/cobertura, coragem e
agressividade aceitam aproximação, curiosidade flanco. Flanco é desvio do eixo
de aproximação conhecido, não leitura da direção oculta do adversário. Resultado
é sugestão sujeita a capacidade/apoio/rota/território/budget existentes.

Cooperação: grupos e papéis são dados do jogo, com ordens de ponto congelado,
formação e prazo. Cada destinatário fica congelado por Reference+vida na admissão;
despawn/reuso/ressurreição não transfere a ordem. Formação tem posição estável por
recibo, sem rastreamento invisível de líder ou redistribuição secreta ao morrer
um aliado. Controle é administrativo/local primeiro, sem ampliar autoridade do
protocolo remoto2 implicitamente.

Reservas de apoio serão opt-in por grupo, limitadas aos alvos que o executor já
aceitou mediante observação. Coordenam cura automática, sem bloquear comandos
humanos explícitos ou transformar o grupo em sensor global. Cancelamento, morte,
troca de vida/alvo e término liberam a reserva; inspeção distingue escolha,
reserva e execução real.

Refinamento da reserva: no máximo uma entrega de cura reservada por fonte;
prazo `support_lease_ticks`1..3600 (padrão120). Cast/canal/projétil/periódico/área
com alvo individual conservam a reserva até término ou prazo. Consultas usam
índices por fonte dos pools existentes; áreas espaciais sem alvo individual não
reservam personagens que ainda não selecionaram. Cura explícita humana ignora
esta política de coordenação automática. Concern/sociability/intelligence e
discipline priorizam intenções concorrentes antes do executor; InstanceId
desempata. Reserva existente precede nova eleição. Novo nascimento/vida ou
remoção da fonte libera a coordenação; uma entrega destacada ainda pode chegar,
pois reserva não cancela efeito. Uma conclusão no tick mantém a preferência
desse frame para evitar dupla cura baseada na mesma leitura; limpeza final
libera para o próximo tick. Essa é uma lease limitada, não promessa de ausência
de overheal em todo cenário.

## Revisão estática antes do código

1. Pontos do mapa até64 e amostras de anel limitadas, cadência reaproveita perfil.
2. Diagnóstico compacto no indivíduo; sem Scene clone/listas de mundo por score.
3. F32 de autoria validado, scores/vetores limitados; sem JSON/reflexão no tick.
4. Mesmos objetivos/navegação/abilities/population, evitando novo log/relógio.
5. Ordens usam fila finita existente; formação e reservas limitadas por atores.
6. Sem pipeline GPU novo; capturas offscreen e replay de estado compartilhado.
7. Snapshot de evidência e desempate por indivíduo; sem locks/threads/LLM.
8. Identidade+vida nas ordens/reservas, limpeza em lifecycle/reset/reuso.
9. Uma prova crítica integrada e regressões existentes por impacto, sem teste por trait.
10. Explicitar custo conscientemente mantido e fronteiras de risco/rotas/escala;
    não inferir FPS, formação perfeita ou comportamento coletivo universal.

## Contrato implementado e revisão

Tática amostra posição atual, mapa conhecido e até16 posições de anel; esforço
total8..64 conforme configuração/inteligência, amostragem distribuída do mapa e
truncamento explícito. Diagnóstico contém evidência congelada, melhor candidato,
score/erro de alcance/cobertura/flanco/contagens/próxima avaliação. Fonte/vida/
identidade da pista invalidam retenção; sem pista não há objetivo tático. KCC
admite apoio em posições terrestres, fly admite espaço livre, legado conserva
superfície; executor decide rota real. Território/lock/budget de investigação
continuam filtros. Cobertura é oclusão em relação à pista, não conhecimento de
adversário escondido, direção de olhar ou todos os ataques de área do jogo.

Ordens move/hold/cancel com prazo1..36000ticks; move usa offsets/line/wedge/circle,
spacing0.1..8 e orientação explícita. Role ordena formação e risco pessoal
coragem/medo desempata antes de InstanceId. Posição/ordinal/membros congelados
na admissão; lote só entra se todos destinos/recipientes e orçamento forem
válidos. Limite atual da fila64entradas/tick inclui todos destinatários;
não há fracionamento invisível de grupo. Reação deriva inteligência/disciplina;
prazo corre desde emissão. Ordem expirada/cancelada devolve controle aos
objetivos; cast/retorno/disabled suspendem locomoção sem estender prazo. Hold
nunca corta voo comprometido no ar; motor conclui trecho seguro existente.
Ordens são administrativas locais, sem nova autorização remota implícita.

Cooperação não introduz sensor global. Intenções de cura já elegíveis competem
por grupo/alvo/vida. Concern/sociability/intelligence/discipline ordenam eleição,
reserva existente tem precedência e fonte só mantém1lease. Máscaras de prontidão
automática suspendem novas curas da fonte enquanto entrega reservada está
pendente, conservando slot de cast ativo; comandos explícitos não sofrem esse
filtro. Lease cobre canal/projétil/periódico/área com alvo individual via buckets
por fonte; identidade/vida/prazo e existência da execução são revalidados. Decisão
do frame e leases são inspecionáveis e comparados no replay. Não coordena áreas
espaciais sem destinatário, nem promete ausência universal de overheal.

Custos mantidos: dois loops limitados do catálogo de atores para tática/ordens,
queries estáticas limitadas por candidato, Vecs de coordenação reutilizados e
reservados antes de commits só quando feature existe; sort_unstable sem heap,
ActorSet para bloqueio O(1), busca de leases por buckets limitados da fonte.
Sem map/JSON/Scene clone em pontuação, sem workers ou LLM/rede obrigatórios.
Serde limita coordenação e recusa dono duplicado; um quadro de preferência após
conclusão evita dupla cura sobre a mesma leitura. Autoria/erros/reset/reuso
continuam preservando documento e identidade dos sobreviventes.

27Rust/3alvos (uma prova crítica nova ampliada) e8MCP25grupos headless;
check releasealltargets/Clippy lib/bin/prova-Dwarnings/fmt/Python/diff passaram.
Prova cobre privacidade, cobertura, formação física, admissão de lote/autoridade,
ordem pendente obsoleta após reuso, morte/revive, eleição de suporte/custos,
cast/projétil, serde malformado, comando humano e replay. Prefixo PNG vivo/replay
byteexato. Build2651a50e94e99cf1, demo de apoio em
examples/npc-tactics-cooperation.scene.json salva pelo MCP. Sem janela/foco,
aceite humano, MMO/FPS ou fechamento de rotinas/avatar/checkpoint/escala.
