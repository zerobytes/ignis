# Estabilidade do contato inicial de caixas — EAF-146

## Preflight e decisão antes de código

Descoberto durante EAF-145. Cena simples1Player sem rede reproduz afundamento no
piso em X=Z depois de repouso, tanto tocando quanto, raramente, com skin inicial.
Carga1000 tem ACKs corretos e saúde correta, mas33fontes sem passos; adicionar
1cm deixou2, então não é correção suficiente nem gate aprovado. Conservar prova
física; não ampliar tolerância de passos/grounded nem remover atores da receita.

No código local Parry0.25.3, cast_shapes_support_map_support_map usa contato
support-map genérico no início dentro de target_distance; cuboid/cuboid tem
contato SAT especializado já disponível. Hipótese a provar: normal degenerada
no contato genérico permite queda e grounded sem apoio geométrico num piso plano.

Decisão limitada: dispatcher readonly somente das consultas KCC, que obtém
contato inicial cuboid/cuboid por SAT quando já estão dentro do offset. Respeitar
normal/velocidade, stop_at_penetration e testemunhas; demais casos/movimento
separado/tipos usam dispatcher original. Mesmo adapter em runtime e probes
terrestres. Não implementar depenetração arbitrária, alterar solver dinâmico,
normal de rampas/cunhas, teletransportar corpos ou somar altura escondida no spawn.
Replay98 distingue a mudança física; Scene88/protocolo2 inalterados.

1. Uma consulta de contato especializada só para par de caixas no KCC;
   delegates sem Scene clone/JSON e sem alteração da física legada.
2. Adapter no stack, sem heap/cache/estado por tick. Custos SAT medidos pela carga.
3. Mesmo Rapier/Parry e geometria/configuração existentes, sem dependência/ABI nova.
4. Usar mesma regra no planner/runtime para não prometer caminhos divergentes.
5. Contatos/iterações KCC e budget de planejador mantidos; não varrer todo mundo.
6. Sem GPU/socket/arquivo no tick, inspeções sob pedido.
7. Read-only sobre snapshot físico, uma autoridade de mutação.
8. Sem cache novo; formato de replay sobe por semântica física, cena preservada.
9. Uma nova prova crítica em posições diagonais/repouso/movimento/apoio/passos/
   replay, reusar controladores/ground navigation/conexões/QUIC/offline conforme
   impacto. Não criar teste por botão/contador.
10. Recuperação de penetração profunda, plataformas e CCD relativo permanecem
    fora do contrato; não alegar ganho de FPS ou solver completo de multidões.

Nota de diagnóstico: sliding é o flag bruto de handle_slopes do Rapier, cujo else
também o ativa em contato plano sem tangente vertical. Não o usar como prova de
ângulo do piso; a prova exige altura, contato e passos físicos, não sliding=false.


## Revisão e evidência

Build09c4088aba9675cc/130inputs, Scene88/protocolo2/replay98. StableContacts é
readonly/no stack: usa SAT existente para contato inicial cuboid/cuboid dentro
do offset, conserva testemunhas/normais e recusa contato separando quando
stop_at_penetration=false. Restante do cast e demais5métodos delegam ao original.
Runtime e ground_navigation usam o mesmo adapter; solver dinâmico permanece.
Não há correção de autoria/teleporte/cache/estado/alocação ou dependência novos.

Uma prova crítica nova,6combinações (X=Z=-23,25/9,75/11,25 e Y0,8/0,81), cada uma
com240ticks parado+600movendo e reversões: pés nunca abaixo de-3mm, contato
mantido, pelo menos15passos físicos e replay exato. Falha reproduzida previamente
sem rede e no ensaio1000. A hipótese do contato inicial genérico é sustentada
pela correção localizada e essas execuções; sem alegar solver de penetração geral.

77Rust selecionados (71integrações incluindo essa prova,3teclado,1controle remoto,
2QUIC), Clippy all-targets -Dwarnings e11MCP44grupos aprovados. Rampas/degraus/
teto/corpo/push/casts/lifecycle, navegação/conexões, offline com rede negada,
controles, passos, replay e capturas existentes preservados. Testes por risco,
sem repetir suíte inteira nem criar casos para cada delegate.

Gate conjunto EAF-145 no mesmo build (exemplos auditados por SHA separado):
1000TLS×100operações tanto mixed como boss; todos1000grounded e1000fontes de
passos. Mixed:100.000operações/vida999900 uniforme,24000passos/8ouvintes, p95sim
6,530ms/total7,773ms. Boss:100.000completed/0occluded/vida900000 no chefe,
1000buffs,17933passos/7ouvintes, p95sim16,301ms/total18,023ms. Mesma receita
originalY0,8, sem skin extra para aprovar. Nenhuma física/colisão/visão foi omitida.
CPU da máquina compartilhada com gerador de1000clientes; não equivale a FPS/WAN
ou60Hz universal. Um timeoutTLS anterior está registrado e ainda requer tratamento
separado; não é corrigido por inferência desta alteração física.

Revisão10itens: consultas no movimento existente; adapter sem heap; IDs/config
preservados; mesmo caminho planner/runtime; budgets/limites mantidos; sem I/O/GPU;
um writer/read-only; sem invalidação nova; prova crítica+regressão proporcional;
SAT em pares de caixas e manifolds internos do Rapier são custos conscientemente
mantidos. Artefatos kcc-contact-{audit,report,critical,integration,clippy,mcp} e
network-load-{mixed,boss}.json; detalhes/limites finais do ensaio em EAF-145.
