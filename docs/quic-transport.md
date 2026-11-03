Atualização EAF-143: protocolo/ALPN2 e apresentação/conteúdo público implementados;
ver remote-presentation.md. Evidências abaixo registram o recorte EAF-142/v1.

# Transporte real QUIC — continuidade EAF-141

## Decisão e checklist antes de implementar

O backend integra o domínio de sessões, não expõe o dispatcher administrativo.
Quinn0.11/Tokio com TLS e certificado público explicitamente confiado por pin;
quinn-proto>=0.11.17 obrigatório. Certificado/chave são efêmeros no host; só DER
público é retornado pela inspeção. Não há conta/API proprietária obrigatória.

Thread de I/O com runtime Tokio limitado; tarefas async por conexão, nunca uma
thread OS por jogador. Core só troca mensagens/Arc por caixas limitadas. Endpoint,
TLS, leitura/escrita, JSON e encoding de réplica ficam fora do lock do mundo.
Abertura é assíncrona para não fazer bind/TLS sob o dispatcher. Offline não cria
runtime, thread, endpoint nem observer. Encerrar/rebuild invalida mundo/conexões.

Um stream bidirecional confiável versionado: hello (join/resume), operações,
resultados e visão privada. Inputs são datagrams latest-wins com sequência/vida;
controle não recebe sessão/geração escolhidos pelo cliente. O transporte atribui
envelope a partir do handshake, com prazo de leitura, bytes/taxa limitados e
orçamentos separados por conexão. Não enviar segredo de retomada em inspeções.

Snapshots públicos são capturados uma vez a10Hz, codificados fora do mundo.
Baseline full confiável usa stream uni independente; deltas datagram fragmentados
com limite de tamanho, remontagem limitada e época/tick/base. ACK só após aplicar
frame inteiro. Ausência de baseline solicita full; duplicação/reordenação/atraso
não altera estado. Cache pequeno compartilhado por baseline, não serializar cena
para cada observador. A saída atual substitui snapshot não iniciado; conexão lenta
é encerrada com controle neutralizado, sem travar simulação nem reter fila infinita.

1. Frequência: pump de filas no tick; snapshot10Hz; sem JSON no motor de atores.
2. Alocações: fila16operações/conexão, resultados64, latest input/owner/frame,
   cache limitado e buffers de streams/QUIC explicitamente reduzidos.
3. Dados: protocolo versionado com IDs lógicos/época/vida; nenhum slot/handle.
4. Repetição: Arc de projeção e cache de codec por base compartilhados.
5. Limites:1000sessões, conexões/handshakes/taxas/bytes/fragmentos/prazos finitos.
6. I/O/GPU: tudo fora da mutação; réplica headless não declara FPS/aceite gráfico.
7. Locks: uma autoridade; bridges curtas sem socket/serde/await sob o lock.
8. Invalidação: geração de conexão, vida, época, timeout e desligamento.
9. Novas provas só críticas: TLS/pin, autorização/dedup/reconexão, perda/resync,
   cliente lento,1.000conexões reais e offline com rede negada.
10. Custos: bytes para1000receptores reais; loopback não prova WAN/infra/MMO pronto.
    Sem persistência durável, previsão/interpolação/compensação de latência implícitas.

A implementação mantém datagrams de até1028B (payload1000+header28), remontagem
máxima de2frames/500ms e teto1MiB. Deltas acima32KiB viram full confiável para
não expulsar os próprios fragmentos do buffer QUIC64KiB; full no máximo1/s por
conexão. Cache12frames e packets por par (tick,base) compartilhados, inclusive
fragmentos Bytes para evitar remontar cabeçalhos para cada receptor. ACK perdido
pode ser repetido com o tick atual; baseline fora do cache força full. Resultados
64 pendentes e owner latest coalescido; janela128/256KiB e envio64KiB por conexão.

TLS usa ALPN ignis/1, valida nome ignis.local e DER explicitamente confiado; 0-RTT
é desabilitado nos dois lados para operações não chegarem como early data
reexecutável. Cada hello passa por prova de endereço QUIC Retry;32handshakes,
64incoming/1MiB total e no máximo capacidade+32conexões. O core não aceita
ID da sessão/geração por mensagem: Link nasce do handshake confirmado.

API compartilhada: start_multiplayer_host/stop_multiplayer_host/
inspect_multiplayer_host; connect_multiplayer/reconnect_multiplayer/
disconnect_multiplayer/inspect_multiplayer_client; send_multiplayer_movement e
send_multiplayer_operation. Start é assíncrono, inspect retorna listening + IP/porta
+ certificado público; client recebe esse DER em base64. Resume secret fica
privado no handle nativo. Submit confirma fila local, resultado final é observado
separadamente. --server IP:port inicia headless, --scene fornece roster autorado;
--mcp pode acompanhar. Sem --server ou start explícito, rede fica ausente.

## Evidência EAF-142

Build `a57ddc00a2a686ce`,125inputs, Scene88/replay97, macOS Apple Silicon.
Quinn0.11.12/proto0.11.19, Tokio1.54.1, Rustls0.23.46, rcgen0.14.10 no lock.
79Rust afetados (incluem3provas de domínio) +2provas críticas QUIC; Clippy
all-targets -Dwarnings;11MCP48grupos headless. `continuous_runtime` e
`local_transactions` mantêm prova offline com rede negada. CLI `--server
127.0.0.1:0` foi observado listening sem janela. MCP testa2processos reais,
private owner/public roster, resultado final, sequência recusada sem falso ACK,
retomada sem cobrança repetida, reset/epoch/encerramento e volta ao recorded offline.

`IGNIS_QUIC_PLAYERS=1000 IGNIS_QUIC_ROUNDS=20 cargo test --locked --release
--test network_transport real_quic -- --nocapture`:1000QUIC/TLS distintos sobre
endpoint UDP compartilhado no gerador de carga,1000rosters completos e20.000hits.
Todos HP980/energia80; o retry do ID1 com outro payload não causa novo efeito.
Conexões2050,067ms;19rodadas seguintes3550,616ms;621ticks no servidor, réplica619.
Passo inclui admission+simulação+projeção/publicação, sem socket/codec:
p502,713/p953,634/p994,679/max7,151ms. Cliente que não consome streams encerra
em3001,98ms; outro avança enquanto isso. Perda de fragmentos é injetada na
recepção da aplicação, não representa ensaio de WAN/roteador/perda UDP física.

Bytes UDP reais somados dos clientes, incluindo handshake/ACK/QUIC:23.153.309B
enviados e400.709.075B recebidos. Counter do servidor mede payload de snapshot
enfileirado435.450.540B (não equivale a bytes entregues),92.969deltas/1086full e
512encodes compartilhados. Sem câmeras/AOI removendo participantes do combate.
Este cenário usa corpos dinâmicos/hits pareados; KCC/mixed/chefe comum têm suas
provas locais em joint-runtime-load.md e ainda exigem ensaio conjunto pela rede.
Não extrapolar esse p95 para todos combates, GPU, WAN ou infraestrutura MMO.

Artefatos correntes: quic-{audit,report,real-load,retention,debug-retention}.json,
quic-{critical,thousand,integration,clippy,build,mcp,network-mcp}.log. Dois ajustes
foram no harness: seed obrigatório e summary checks inteiro do runner; não houve
relaxamento de asserção. Codec fragmentado usa poses explícitas no fixture para
exercitar múltiplos fragmentos sem depender de movimento residual da física.

## Revisão estática final e custos mantidos

Tick aplica caixas16/source e drena resultados64; scratch16 é reutilizado entre
sessões. Captura pública e privados10Hz, appearance Arc, codec fora da mutação
com cache12frames×baseline e fragmentos Bytes comuns. Bound de frame1MiB, duas
remontagens/500ms, streams/janelas/taxas/prazos finitos; parsing completo antes de
publicar réplica. Seq/life/connection/world cercam reuso; refresh/reset/stop removem
handles e observador. Duplicate retorna resultado existente, não executa outra vez.

São mantidos Vec/BTreeMap, JSON no controle confiável/owner (fora do mundo),
alocações de owner/patches, um notify por sessão/tick e timer50ms por writer para
encerramento; são custos limitados, não promessa de zero heap. TLS/CSPRNG só em
abertura/sessão;2workers de I/O por host/cliente nativo, tasks async por conexão,
nenhuma thread OS por Player. Quotas16/64 e filas não substituem proteção
operacional contra abuso distribuído. Guest ocupa Player livre autorado; conta,
spawn de Player, saves de mundo e ACK durável não foram implementados.

Cliente nativo/API já recebe e aplica réplica; a janela jogável ainda apresenta o
mundo local. Apresentação/controles remotos na janela, conteúdo estático/projeto,
interpolação/predição e aceite gráfico têm gates próprios. Não declarar entrega
de jogo multiplayer gráfico ou do MMO inteiro a partir deste backend headless.

Retenção: único release corrente ignis24.375.264B e exemplos atuais;8artefatos
first-party superados200.287.576B removidos com fingerprint/lock/stat/lsof.
Debug parcial inativo279.755.587B removido após conferir locks e processos;
compiler state corrente e app instalado preservados. Sem cópias históricas de app,
sem instalar/lançar/tocar janela do usuário, sem push.
