# Sistema multiplayer e MMO — EAF-117

Requisito permanente esclarecido em29/10/2026: single player offline e multiplayer
coexistem. A autoridade em single player é a simulação embarcada no jogo, sem
internet, autenticação remota ou processo separado de servidor. O servidor
multiplayer usa as mesmas regras; rede/serviços não são dependências do núcleo
local. EAF-122 implementa controles independentes; EAF-123 inicia a migração de
[transações e armazenamento indexado](local-transactions.md). O diagnóstico abaixo
é histórico da base indicada, não retrato de todos os incrementos posteriores.


## Requisitos confirmados e estado deste documento

Direção de Guilherme em 28/10/2026: priorizar o sistema multiplayer da Ignis,
sem construir agora um jogo específico. **Suportar MMO e, futuramente, até
1.000 jogadores simultâneos na mesma área e no mesmo combate é uma premissa
arquitetural.** Distribuir os jogadores entre regiões ou partidas isoladas não
satisfaz esse requisito. Infraestrutura de hospedagem não é objeto deste estudo.

Esse é o alvo do sistema, não uma capacidade implementada ou medida. As escolhas
abaixo são recomendações para discussão; não aprovam automaticamente uma
biblioteca, protocolo, ECS, modelo de colisão ou mudança de semântica de combate.
O requisito de escala foi confirmado; workload, budgets e escolhas estruturais
permanecem a alinhar antes de codificar.

Base funcional auditada: `main` local `e16ae24`, cena66/replay70. A auditoria
considera também os contratos de meshes/editor das branches locais, sem incorporar
seus formatos à main. Há trabalho paralelo de geometria no checkout compartilhado;
este incremento é documental e não modifica seus arquivos, builds ou sessão visual.

## 1. O que precisa ser independente

| Conceito | Responsabilidade e duração propostas |
|---|---|
| Identidade persistente de jogador/personagem | Identifica o registro durável; não é endereço de rede, índice de array ou token MCP. Conta e personagem são conceitos separados. |
| Conexão | Transporte atual, negociado e autenticado; pode cair ou ser substituído sem criar outro personagem. |
| Sessão de jogo | Permissões, associação ao mundo e controle; define reconexão e invalidação da conexão anterior. |
| Instância de ator | Indivíduo presente no mundo, com referência geracional; morte não equivale a retirada ou logout. |
| Vida | Encarnação atual do mesmo indivíduo; manter a separação de `life_index` já existente. |
| Autoridade | Dono da mutação de um estado; não se confunde com quem apresenta ou prevê esse estado. |
| Observador | Visão autorizada de uma conexão sobre o mundo; não é o snapshot de inspeção administrativa. |

Não expor slots físicos como identidade de rede ou persistência. Recomenda-se
handles tipados, slots suficientemente amplos e geração validada no acesso;
identidade lógica e época do mundo distinguem reinícios. A largura exata e o
formato binário devem ser escolhidos juntos, sem despejar structs Rust na rede.
Nem todo `u8` atual é um problema: índices pequenos de grants ou tipos podem
continuar limitados. O problema é usar esses índices para a população global.

## 2. Autoridade e consistência

| Alternativa | Tradeoff para o alvo confirmado |
|---|---|
| Cliente autoritativo/P2P | Distribui trabalho, mas aumenta confiança nos clientes, conflitos de posse e coordenação entre muitos participantes. Não recomendado como base do combate MMO. |
| Lockstep de todos os clientes | Reduz distribuição de estado em alguns modelos, mas exige determinismo compatível e política para participantes atrasados; inadequado como pressuposto para 1.000 clientes heterogêneos e o runtime físico atual. |
| Servidor autoritativo com snapshots | Centraliza decisão de gameplay e admite apresentação/predição locais; exige orçamento explícito de simulação, replicação e reconciliação. Recomendado. |

Um serviço lógico autoritativo deve decidir movimento válido, colisões, custos,
cooldowns, alvos, dano, buffs, morte e nascimento. Clientes enviam intenções;
posição enviada pelo cliente não vira prova de deslocamento, alcance ou acerto.
Autenticação do transporte não concede posse de qualquer ator.

Recomenda-se uma única autoridade de escrita por estado em cada instante.
Regiões/workers podem organizar trabalho, sem exigir processos distribuídos agora.
Se houver transferência futura, a época de autoridade deve impedir escrita dupla,
e efeitos que cruzem a fronteira precisam de ordem e identidade de operação.
Dividir o combate denso em regiões não é uma solução assumida para atingir a meta.

A ordem autoritativa precisa ser registrada por tick/sequência de admissão, com
empates definidos e reprodução possível. Recepção de pacote, autorização,
admissão e commit são etapas distintas. Filas devem ter limites e equidade por
sessão: uma conexão não pode consumir todo o orçamento de entrada. Rejeição por
capacidade acontece antes de cobrar recursos; admissão não promete acerto futuro.

Preservar a semântica atual de área: destinatários selecionados de forma definida,
transação atômica por destinatário e ordem explícita, sem rollback coletivo novo.
Uma otimização paralela não pode mudar quais efeitos anteriores uma transação vê.

## 3. Simulação preparada para combate denso

Mudar `MAX_ACTORS` para 1.000 não resolve os contratos atuais. A migração deve
abranger estado, consultas e trabalho por operação, além de capacidade de storage.

- Armazenamento de componentes por ator/handle, com crescimento controlado,
  limites configuráveis e buffers reutilizados. Considerar arrays/páginas e índices
  por componente; adotar um framework ECS não é requisito nem decisão tomada.
- Transações com scratch proporcional aos atores e campos tocados. Cobrar um
  recurso ou acertar um alvo não deve copiar saldos e vida de todos os jogadores.
  Snapshots iniciais de contribuições continuam garantindo a atomicidade atual.
- Atributos/buffs indexados por dono e chave/contexto; agendas por prazo e ator.
  Evitar procurar em todos os buffs do mundo para avaliar cada parcela de dano.
- Broad phase compartilhada para consultas espaciais, seguida dos filtros exatos
  de geometria, relação, elegibilidade e evidência. Não repetir uma varredura total
  do mundo por ator, habilidade e observador quando há candidatos reutilizáveis.
- Pools de projéteis, efeitos, áreas e origens dimensionados para o workload;
  limites por autor/sessão e limites globais explícitos. Um pool mundial de64
  projéteis não é uma política final de combate para1.000 participantes.
- Paralelismo primeiro em preparação/leitura e trabalho comprovadamente
  independente. Commits conflitantes mantêm ordem; física e efeitos exigem análise
  própria. Locks por entidade e threads por conexão não são pressupostos.
- Player e NPC compartilham contratos de ator e execução, mas controle humano e
  decisão de IA são componentes distintos. Não transformar NPC em jogador remoto
  por um alias nem obrigar todos os jogadores a executar percepção de NPC.

O motor precisa permitir política explícita de colisão de personagens, frequência
de decisões de IA e representação de projéteis. Não substituir automaticamente
Rapier, remover colisões entre jogadores ou reduzir acertos para atingir contagem.
Comparar essas políticas por gênero faz parte do contrato, não de um atalho de carga.

## 4. Relevância e replicação no pior caso

Relevância espacial ajuda quando participantes estão separados, mas pode conter
todos os1.000 no combate confirmado. Nesse caso, a seleção não elimina o custo
real de distribuir atualizações aos destinatários interessados.

Recomenda-se separar **relevância de gameplay**, **permissão de informação** e
**fidelidade de apresentação**. Um personagem pode exigir posição/ação pública,
enquanto saldo privado, memória de NPC e intenções ocultas não são replicados.
Grupo, interação e perigos de combate acrescentam dependências semânticas à
proximidade. Sair do campo da câmera não remove alguém do servidor ou do dano.

Manter índices persistentes e conjuntos compartilháveis de candidatos, evitando
recalcular relevância por todos os pares a cada frame. Codificar conteúdo comum
uma vez quando possível, preservando estado de ACK/baseline e autorização por
conexão. Delimitar quantização, deltas, frequência e prioridades por componente;
não retransmitir geometria, definições imutáveis e JSON da cena por tick.

Exemplo de dimensionamento, **não benchmark nem orçamento aprovado**:

`1.000 × 999 × 20 atualizações/s × 32 bytes = 639.360.000 bytes/s`

São aproximadamente639MB/s agregados e639kB/s por cliente para esse payload
hipotético sem headers, perdas, criptografia ou eventos. Não assume multicast;
representa unicast completo de cada ator para os demais. Os19.980.000 registros
de destinatário por segundo não exigem igual número de codificações se conteúdo
for compartilhável, mas bytes por destinatário e filas continuam tendo custo.
Mudar a hospedagem não remove esses custos do sistema.

O mesmo vale para efeitos: se1.000 autores produzirem uma área por segundo com
1.000 destinatários cada, haverá até1.000.000 de entregas por segundo antes de
filtros, cada qual com os efeitos configurados. Um índice espacial não elimina
destinatários que realmente precisam ser afetados. Workload de habilidades deve
ser parte da aceitação; cap de alvos só é válido quando for regra autorada explícita.

Perda de baseline exige ressincronização, não aplicar deltas em estado desconhecido.
Spawn/retirada/vida/época, ações e correções precisam de ordem compatível: pacote
atrasado não pode ressuscitar uma réplica ou atuar sobre o ocupante novo do slot.
Entrar na relevância requer base suficiente; sair exige descarte/versionamento
compatível com eventos atrasados. Um cliente lento não pode acumular fila infinita
nem atrasar o mundo: substituir estados supersedidos e limitar ressincronizações,
com política explícita para eventos críticos e desconexão.

## 5. Tempo, comandos e transporte

Separar frequência da simulação, envio de input, replicação e renderização.
O tick atual de60Hz é baseline histórico, não compromisso de desempenho para MMO.
Não reduzir esse tick silenciosamente: velocidades, prazos, física e replay
dependem dele. Uma decisão de frequência requer workload e erro aceitável.

| Tráfego | Semântica proposta |
|---|---|
| Movimento contínuo | Sequência/tick, validade limitada, ACK e redundância/reenvio limitado; não aplicar input velho depois de novo nem repetir deslocamento por duplicata. |
| Intenção discreta | ID de operação, validação de posse/vida, admissão e resultado; deduplicação e retry limitados, sem cobrança duplicada. |
| Snapshot substituível | Tick, época, base identificada e descarte de versões antigas; retransmitir o estado atual quando apropriado. |
| Ciclo de vida e resultados críticos | Entrega/recuperação definida e dependências verificadas; não confundir confiabilidade de pacote com commit exatamente uma vez. |
| Conteúdo e controle de sessão | Fluxo separado do tráfego temporal; limites de tamanho, compatibilidade e backpressure. |

Comparar transporte UDP com biblioteca de confiabilidade/segurança, QUIC com
streams+datagrams e alternativas de plataforma. Um único stream confiável ordenado
é simples, mas acopla tráfego que poderia avançar independentemente. UDP próprio
transfere recuperação, congestionamento e segurança para nós. QUIC fornece streams
e proteção; datagrams dependem da extensão correspondente. Não escolher crate,
serializer ou transporte de browser antes de avaliar APIs, manutenção, plataformas,
backpressure e integração com o relógio. O domínio deve depender da semântica de
entrega, não de detalhes de uma biblioteca.

O cliente pode prever o movimento do próprio personagem e interpolar os demais;
recomenda-se projetar ACK de inputs e correções desde o contrato, mesmo que uma
etapa de verificação ainda use apresentação atrasada. Predição não confirma dano,
recompensas ou custos. Compensação de latência de acertos exige política por tipo
de habilidade e janela histórica limitada; não implica rewind integral do mundo
para qualquer pedido atrasado. Sem relógio do cliente autoritativo.

## 6. Sessões longas, reconexão e estado durável

O servidor não pode exigir reset porque um log de diagnóstico encheu. Separar
relógio/tick e vida da simulação, gravação opcional de replay em segmentos limitados,
checkpoint e persistência durável. O checkpoint precisa incluir estado suficiente
de física, comandos pendentes e entregas; um snapshot de apresentação não basta
para restaurar a simulação determinística. Isso precisa ser implementado e testado.

Reconectar envolve autenticar uma sessão permitida, invalidar o controle anterior,
reconstruir visão/base atuais e resolver comandos sem resposta pelo ID da operação.
Timeout deve neutralizar input preso. Logout/desconexão não significam morte ou
despawn automáticos: permanência do corpo em combate, expiração e retorno são
políticas explícitas para evitar vantagem por desconexão. Respawn continua distinto
de ressurreição. Migração de autoridade, quando existir, precisa da mesma proteção.

Estado persistente de personagem e futuras operações de inventário/recompensa
exigem um contrato durável com revisão, identidade e deduplicação gravadas com o
efeito; um ACK de rede não prova persistência. Banco, serviço de contas e operação
de hospedagem ficam fora da escolha atual, sem eliminar essa fronteira do sistema.

## 7. Cliente, render e MCP

Servidor sem dependência de janela, GPU ou captura para executar gameplay.
Cliente apresenta uma réplica autorizada, separada do documento autoral e da
simulação autoritativa. Assets locais são negociados por manifesto/identidade;
RPC de gameplay não transporta livremente edição, arquivos ou comandos MCP.

O cliente precisa ser avaliado com1.000 personagens no mesmo cenário, além de
efeitos/ambiente do workload. Culling, instancing, skinning e LOD de animação são
frentes gráficas relacionadas; não presumir que CPU skinning e os budgets autorais
atuais já atendam. Simplificação visual não pode alterar hitboxes ou decisões do
servidor. Evidência headless de carga não comprova legibilidade ou fluidez no cliente.

Operações de configuração, execução, inspeção de sessões, posse, filas, rejeições,
ACKs, replicação e falhas simuladas devem existir no dispatcher/MCP antes ou junto
da UI. MCP é plano de controle confiável, não protocolo público por frame. Inspeção
deve aceitar filtros/paginação e limites: não gerar o mundo inteiro sob um mutex
por polling. Captura GPU, foco e entrada nativa continuam gates distintos.

## 8. Lacunas verificadas no código atual

| Evidência na base funcional | Consequência para a migração |
|---|---|
| [Scene::validate_inner](../src/model.rs): no máximo um Player,512 entidades e8 NPCs. | Separar autoria de capacidade runtime e de controles; dimensionar jogadores, NPCs, efeitos e entidades gerais separadamente. |
| [Simulation](../src/physics.rs): `player`, `player_actor`, perfil de locomoção único; [Engine](../src/engine.rs): um `input`. | Roteamento por jogador/ator e validação de posse; movimento/cast/passos por ator. |
| [targeting](../src/targeting.rs): `MAX_ACTORS=9`, slots de ator `u8`; [identity](../src/identity.rs): registro limitado pelo MAX_ENTITIES atual. | Handles/armazenamento e consultas sem o teto global do laboratório. |
| [vitality](../src/vitality.rs): arrays globais e máscaras `u16`; [abilities](../src/abilities.rs): arrays/casts/saldos por MAX_ACTORS. | Estados por ator e conjuntos dinâmicos limitados onde necessários, sem apenas ampliar arrays globais. |
| `abilities::Plan` copia `w.vitality.state.actors` e `self.state.balances` em `plan_effects`. | Transações locais antes de escalar número de efeitos e destinatários. |
| [attributes::Registry::resolve_contributions](../src/attributes.rs) filtra contribuições pelo dono; pools globais pequenos. | Índices por dono/chave e reserva de capacidade dimensionada por workload. |
| [projectiles](../src/projectiles.rs): pool64; [abilities](../src/abilities.rs):16 pedidos/tick e4096 entradas; [population](../src/population.rs): filas/histórico limitados. | Admission control, equidade e ciclo de vida de buffers próprios de serviço longo. |
| [Engine::advance](../src/engine.rs):36.000 ticks e input gravado por tick, a60Hz. | Separar execução contínua de gravação;10min não é duração admissível imposta a um MMO. |
| [Service](../src/service.rs), [MCP](../src/mcp.rs), [main](../src/main.rs): transporte de controle e lock compartilhado, sem protocolo multiplayer. | Frontend de rede autorizado, fronteira de tick e snapshots; sem publicar o dispatcher inteiro aos clientes. |
| [Frente gráfica](graphics-handoff.md) e contratos isolados de meshes/editor. | Coordenar representação da réplica/RenderFrame e versões; contagem de participantes simulados não prova render de1.000 personagens. |

## 9. Sequência de implementação proposta

As etapas constroem o sistema orientado à meta; testes pequenos são instrumentos
de correção e não redefinem o requisito como coop para poucos jogadores.

1. Alinhar workload e budgets; registrar decisões de autoridade, identidade,
   propriedade, tempo e semântica de entrega. Nenhuma dependência escolhida ainda.
2. Migrar storage/handles e transações locais preservando identidade, vida,
   efeitos, percepção e replay existentes. Medir operações com população crescente;
   não acrescentar TCP ao singleton atual como arquitetura definitiva.
3. Separar controle por sessão/ator, entrada autoritativa por tick e execução
   contínua. Testes de apropriação indevida, duplicata, desconexão e replay.
4. Implementar visão replicada e scheduler, primeiro com transporte em memória
   determinístico; manter os requisitos reais de ACK, perda, ordem e resync.
5. Selecionar/integrar transporte real e clientes em processos independentes;
   validar rede, reconexão, predição/interpolação e combate compartilhado.
6. Completar gates de carga densa e cliente gráfico; otimizações e paralelismo
   orientados às medições, sem alterar regras para aprovar a contagem.
7. Integrar persistência/checkpoints e recuperação durável segundo os contratos
   acordados; checkpoint/replay não podem bloquear ou reiniciar a sessão viva.

## 10. Gates para declarar a meta atendida

Definir antes da medição: hardware de referência, versões/build, workload,
frequências, limites de CPU/memória/bytes, RTT/jitter/perdas e metas de correção
visual/latência. Essas escolhas de dimensionamento estão abertas; não são promessa
de atingir60Hz ou uma classe arbitrária de dispositivo. Profiling não bloqueia
incrementos de correção, mas capacidade sustentada precisa de evidência.

| Gate | Evidência necessária |
|---|---|
| Correção do núcleo | Legado, atomicidade, identidade/vida/reuso, filas, capacidade e replay/prefixos; testes de regressão headless da main e integração gráfica quando afetada. |
| Densidade real |1.000 jogadores na mesma área com input simultâneo, alvos cruzados, colisões conforme política, áreas, projéteis, buffs, morte e retorno. NPCs/efeitos extras com budgets próprios. Sem reduzir participantes ativos para passar. |
| Rede |1.000 clientes de carga com sessões próprias em processos reais de rede; perda/duplicata/reordenação/atraso/cliente lento, baseline perdido, tempestade de entradas e reconexão. Replay de pacotes/intenções e resistência a posse inválida. |
| Custo sustentado | Tempo de tick p50/p95/p99/máximo, tick debt, bytes/cliente/agregados, filas/descartes/rejeições, CPU/alocações/memória e fairness. Duração contínua acordada, incluindo além do limite atual de10min, sem crescimento histórico ilimitado. |
| Cliente gráfico | Vista real do combate denso, personagens/ações/efeitos e input; medir frame time, memória e latência de apresentação no alvo acordado. Bots sem render não cumprem esse gate. |
| Recuperação | Conexão substituída não controla ator; timeout neutraliza input; resync não duplica efeitos, respawn ou recompensas; checkpoint reinicia sem operação durável duplicada ou perdida conforme contrato. |
| Aceite | Resultados e limites apresentados a Guilherme; aceite humano e validação multiplataforma continuam separados. |

## 11. Revisão estática proporcional deste estudo

Aplicado o checklist de [performance](performance.md) antes de qualquer incremento
de código. Apenas estudo/links de arquitetura neste diff; não altera runtime.

| Pergunta | Resposta e implicação para a implementação futura |
|---|---|
|1. Frequência | Não se aplica à execução do estudo. Separar tick, envio, replicação, render e persistência antes de implementar. |
|2. Cópias/alocações | Identificados Plan global, snapshots e histórico de input; recomendar scratch local, índices e buffers limitados. |
|3. Tipos/acesso | Identificados singleton, `u8`, máscaras `u16` e teto512; identidade lógica não é slot. Não migrar todos os índices pequenos indiscriminadamente. |
|4. Repetição | Reutilizar candidatos/codificação; caso totalmente relevante conserva fanout e interações reais, sem promessa de eliminar esse custo. |
|5. Escala | Meta1.000 no mesmo combate; dimensões de NPCs/efeitos e budgets abertas. Fórmulas são dimensionamento hipotético, não benchmark. |
|6. GPU/I/O | Nenhum build/GPU/rede/arquivo de gameplay novo; contrato distingue servidor headless e cliente gráfico. |
|7. Sincronização | Dono de escrita e fronteira de tick propostos; transporte/persistência fora de seção crítica, sem congelar scheduler/ECS. |
|8. Invalidação | Época/geração/vida, ACK/baseline, reset/load/reconexão e conteúdo precisam de contratos próprios. |
|9. Correção | Matriz de gates acima; só referências/símbolos/diff verificados neste estudo. Nenhum teste de multiplayer executado. |
|10. Custos mantidos | Serialização autorizada, distribuição densa e commits conflitantes continuam reais; decisões dependem de workload e medições. Sem ganho de FPS ou escala entregue alegados. |

## 12. Referências e validação do estudo

O [Replication Graph da Epic](https://dev.epicgames.com/documentation/en-us/unreal-engine/replication-graph-in-unreal-engine)
é referência de organização persistente/compartilhada de candidatos à replicação,
evitando a avaliação ingênua de cada ator contra cada cliente. Não comprova a meta
da Ignis nem resolve a distribuição quando todos realmente são relevantes.
[RFC9000](https://www.rfc-editor.org/rfc/rfc9000.html) documenta streams e transporte
QUIC; [RFC9221](https://www.rfc-editor.org/rfc/rfc9221.html) define datagrams sem
retransmissão obrigatória sobre a conexão. Referências consultadas em28/10/2026;
não constituem seleção de biblioteca, transporte ou plataforma.

Revisão documental executada:18 links locais,12 verificações de símbolos/constantes
contra `e16ae24`, dois cálculos de dimensionamento e `git diff --check` aprovados.
Sem alterações de
fonte, formato, dependência, target ou artefatos de build. Testes Rust/Clippy/MCP
não são reexecutados para este diff documental, e resultados anteriores não são
apresentados como prova de multiplayer/MMO.
