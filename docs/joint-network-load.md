# Carga conjunta pela rede — EAF-145

## Decisão e checklist antes de código

Após e68aa8f/EAF-144: runtime local conjunto e QUIC pareado existem, mas sua soma
não prova coexistência. Reutilizar a receita EAF-140 em módulo compartilhado de
exemplos, sem trocar motor, gerar peers falsos ou duplicar regras de combate.
Novo `network_load` executa autoridade sequencial e clientes QUIC/TLS reais no
loopback, mesmo processo para o gerador de carga; declarar competição por CPU.
Todos os clientes recebem todos atores, sem culling/AOI para cumprir o número.

Mixed usa quatro entregas em pares com KCC/passos/8ouvintes; boss usa chefe
central, arena modular/ring90m e7ouvintes (corpos/oclusões preservados). Movimento
renovado10Hz, intenção explícita com ID/vida por operação e resultado por jogador.
Separar tempo de conexão, janela de combate, drain e convergência. Medir resultado
observado pelo cliente e custos before/sim/after da autoridade, não chamar isso
latência de rede pura ou FPS. Contar recusas legítimas, nunca esconder perda
como oclusão. Toda saúde precisa concordar com transações/entregas reais.

1. Receita e catálogo compilam na preparação; nenhum Scene clone por tick.
2. Buffers de métricas têm prazo/teto; peers e operações limitados por CLI.
3. Identidades vêm de welcome/replica e rótulos públicos, nunca slot presumido.
4. Mesmos definições/Engine/Authority/Host/Client dos caminhos validados.
5. Até1000Players/8NPCs; queues/lease/pools/codec/protocolo2 mantidos.
6. I/O já fora do mundo; métricas por fase sem inspect1000sessions por tick.
7. Uma autoridade, gerador recebe estado sem executar física de cliente.
8. Drain neutraliza movimento, termina agendas; convergência não exige tick
   idêntico enquanto relógio corre, mas sim roster/saúde/vida e progresso atual.
9. Sem novo teste por flag/probe: asserções materiais de resultado/identidade/
   estado/filas do ensaio e regressões existentes conforme alterações efetivas.
10. CPU/bytes/latências observadas do loopback são receita concreta; sem promessa
    universal60Hz/WAN/FPS/MMO de produção, contas ou checkpoint durável. Offline
    continua independente e sem novo requisito de runtime/transporte.


## Diagnóstico da primeira carga

Primeiro mixed1000/100rounds concluiu100.000operações, saúde correta e1000réplicas,
sem desconexões/pendências, mas falhou passos:967/1000fontes. Repetição com
observação confirmou33sem emissão, quase toda diagonal p0000/p0033/...; ACK145
em todos, então não inferir perda de input/rede. KCC grounded=true podia coexistir
com pés penetrados até~0,2m no piso, impedindo o critério físico de passos.

Reprodução MCP1Player sem rede, piso63×63 e spawn(-23,25;Y;-23,25): após120ticks
parado+240movendo, Y inicial0,8 terminou0,61866/passos0; Y0,81/0,82 terminou0,80328/
passos6. Rapier0.30.1 preserva offset, mas check_and_fix_penetrations é TODO no
código local da dependência; não realiza recuperação automática de penetração.
A receita iniciava tocando o piso sem offset, ao contrário dos probes estáveis
do controlador. Hipótese inicial de correção de autoria: somar Config.offset no spawn. A
repetição ainda falhou em2/1000atores, portanto foi rejeitada e revertida. EAF-146
corrigiu contato inicial SAT no KCC comum a planner/runtime; receita originalY0,8
foi mantida e agora passa altura/apoio/passos. Não é recuperação de penetração
profunda nem licença para remover bloqueios físicos/tolerâncias.
Artefatos de diagnóstico: joint-network-diagnosis.json, kcc-start-diagnosis.json.

O smoke anterior esperava AreaApplied como resposta à criação de área persistente.
O executor registra Completed para admissão e AreaApplied somente no pulso,
excluído do request tap; corrigida a expectativa exata, sem aceitar qualquer
resultado nem relaxar vida/contagem/drain. Sem alteração funcional do executor.


## Resultado final e reprodução

Build `f75f4a7ea9154d40`,130inputs; exemplos/helpers têm SHA256 separado em
joint-network-audit.json, pois build.rs identifica Cargo.lock/build.rs/src.
Scene88/replay98/protocolo2. Mac Apple M4 Pro14CPU/48GiB; autoridade sequencial,
backend2workers, gerador4workers/32endpoints UDP para1000TLS reais. Todos os
clientes recebem1000Players+8NPCs, sem culling/AOI/substituição por peers falsos.
Mesma receita local/rede em examples/support/load_scene.rs, sem kernel alternativo.

```
cargo build --locked --release --example runtime_load --example network_load

target/release/examples/network_load --players 1000 --rounds 100 --workers 4 --endpoints 32 --delivery mixed --output artifacts/network-load-mixed.json
target/release/examples/network_load --players 1000 --rounds 100 --workers 4 --endpoints 32 --delivery boss --output artifacts/network-load-boss.json
```

Cada jogador usa100operações com ID/vida e movimento renovado10Hz. Warmup1s;
rounds esperam resultado de todos antes da próxima, cadência mínima100ms. Pacing
não é promessa de100rounds em10s se cliente/IO/autoridade atrasarem. Prazo porround
15s e gerador120s total, sem histórico ilimitado. Métricas before/sim/after são
medidas no writer; collect/inspect/JSON ficam fora do intervalo cronometrado.
Latência observada é desde enfileirar rodada até polling do resultado, inclui
cliente/IO/tick e não é RTT puro. Lag em ticks convertido a60Hz é estimativa.

| Medida | Mixed | Chefe compartilhado |
|---|---:|---:|
| Operações classificadas |100000 (75000Completed+25000Launched)|100000Completed/0Occluded |
| Saúde de todos |1000×999900 |1000×1000000; chefe900000 |
| Grounded/fontes de passos |1000/1000 |1000/1000 |
| Passos/ouvintes |21000/8 |18019/7 |
| Pés mínimos sobre piso |0,002715m |0,000964m |
| Picos projéteis/periódicos/áreas/buffs |250/500/500/750 |0/0/0/1000 |
| Simulação p95/p99/máximo ms |6,302/7,090/7,918 |16,429/17,635/18,771 |
| Total autoridade p95/p99/máximo ms |7,449/8,563/8,744 |17,778/19,205/20,521 |
| Ticks medidos acima16,667ms |0/730 |57/611 |
| Resultado observado p95/p99/máximo ms |130,022/188,016/330,037 |57,984/64,872/74,514 |
| Lag réplica estimado p95/p99/máximo ms |250/400/1033,333 |100/100/150 |
| Conectar1000/combate ms |1050,134/12168,607 |7250,053/10171,198 |
| Drain/progresso1000réplicas ms |168,463/52,042 |46,635/76,605 |
| Tick final réplicas/autoridade |865/866 em todos |1111/1112 em todos |
| UDP clientes enviados/recebidos bytes |64311726/1886483457 |53928336/758046930 |
| Tentativas TLS |1000×1 |998×1+2×2 |

Saúde da autoridade e de todas as réplicas coincide; entregas drenadas e filas/
história0. Gate final exige todas réplicas até12ticks (2períodos) atrás apósdrain,
alcançou1 em ambos. Não confundir esse gate com lag durante combate: mixed pode
atingir1,033s nessa máquina compartilhada. Chefe acima do prazo60Hz em57ticks;
capacidade validada é a receita, não qualquer combinação de skills/corpos/pools.

A recuperação TLS do EAF-147 foi usada em2conexões do chefe, sem atribuição/ação
repetida. Causa original de timeout continua desconhecida. Counter reordered_acks
ficou0; caso de reordenação foi provado isoladamente no EAF-148 e não explica por
si só a diferença entre perfis de gerador. Codec já tem diferenças por campo/
varints/Arc e validate-commit, maioria recebe deltas10Hz. Hipótese de fallback
full1Hz generalizado foi descartada por contadores. Mixed teve23204NeedBaseline,
115983deltas aplicados/1000fulls iniciais; chefe154819deltas/1000fulls/0NeedBaseline.
Deltas em voo contra base anterior são recusados, não aplicados parcialmente.
Não subir protocolo3 ou culpar todo lag sem outro estudo. Interpolação/predição,
cache de baselines no cliente e WAN são evoluções futuras.

## Validação e revisão

As falhas encontradas geraram3correções independentes: contato inicial KCC
(EAF-146/5ed48e7), recuperação inicial TLS (EAF-147/2a14273), ACK monotônico
(EAF-148/c9bfc6d). Novas provas só para esses riscos críticos, sem teste por flag/
contador do exemplo.77Rust/11MCP44grupos na física,27Rust/7MCP27grupos no TLS e
28Rust/7MCP27grupos no ACK, com Clippy all-targets e exemplos aprovados. Os conjuntos
se sobrepõem; não somar como testes distintos. Offline com rede negada preservado.

A receita compartilhada local foi repetida no mesmo build,60warmup+600medidos,
sem transporte: mixed saúde999890×1000, p95sim6,022/total6,644ms; boss110000commits,
chefe890000/playersintactos, p95sim14,300/total14,906ms. Todosgrounded/passos e
história0. Sem declaração de FPS/60Hz/WAN universal, aceite físico ou MMO de produção.

Checklist final10itens: receita só prepara; métricas finitas120s; identidades
welcome/InstanceId sem slots na rede; kernel compartilhado; limites mantidos;
I/O fora do mundo; um writer; neutral/drain/progresso; provas críticas e regressão
proporcional; recursos/latência/bytes/custos mantidos explícitos. Autoridade não
faz serde/socket/GPU no tick. Probes e relatório não são checkpoint persistente.

Artefatos current joint-network-{audit,report,diagnosis}.json, network-load-{mixed,
boss,state}.json, runtime-load-{mixed,boss}.json e logs. Manter release/exemplos
correntes, sem cópias por tentativa. Nenhum app instalado/janela humana foi usado,
nenhum push/publicação. Próximos contratos opcionais de produção: contas/posse
durável/checkpoints, WAN/simulação de latência, interpolação/predição e redução de
lag em cargas ricas. Single player offline continua requisito permanente.
