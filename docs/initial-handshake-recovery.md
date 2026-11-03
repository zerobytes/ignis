# Recuperação limitada do handshake inicial

## Preflight e decisão antes de código

EAF-145 registrou timeout intermitente especificamente em QUIC handshake, antes
de Join, com815peers aceitos numa tentativa. Causa do evento original ainda não
estabelecida. README local Quinn0.11.12 discute buffers UDP de endpoint sob carga;
não inferir que isso foi provado nem mudar sysctl/buffers/dependências por hipótese.

Decisão de produto: até3tentativas TLS somente quando o deadline local do handshake
expira. Manter3s por tentativa; backoff100/200ms, orçamento TLS total aproximado9,3s
mais agendamento. Erro QUIC/criptográfico/pin falha imediatamente. Join/Resume,
welcome, input/operações permanecem de execução única e com deadlines existentes.
O retry ocorre antes de abrir stream/enviar Hello, então não repete atribuição ou
consumo de ID/custo. Não usar isso como alegação de correção da causa original.

1. Rede só quando conectar explícito; offline não cria timer/socket.
2. Loop de3tentativas, sem fila/estado durável/heap adicional por frame.
3. Mesmo endpoint/pin/ALPN/TLS0RTTdesabilitado; nova tentativa não relaxa segurança.
4. Reusar Connector de native/probes; metadata local attempts observável.
5. Deadline3s e semáforos/limites do servidor mantidos; max9,3s total explícito.
6. I/O async fora do Service/world lock; nenhum trabalho GPU/Engine novo.
7. Uma autoridade; nenhuma repetição de Hello/Join/Resume/habilidade/consumo.
8. Desistência final é erro, sem fallback offline e sem tentar outro certificado.
9. Só prova crítica com proxyUDP descartando tentativa inteira e depois liberando:
   uma sessão/um dano, pin inválido sem retry; gates QUIC/offline existentes.
10. Maior tempo de conexão sob falha é tradeoff assumido; não garante disponibilidade
    ou WAN/produção. Investigar causas de saturação separadamente quando observáveis.

Native também observa disconnect a cada25ms durante connect/retries, antes de
publicar ticket/estado. Cancelar o future impede tentativas tardias após sair;
race após Hello segue a limpeza/reserva de sessão já existente, não nova operação.

## Revisão e evidência — EAF-147

Buildefc7c952c827e327/130inputs, Scene88/replay98/protocolo2.27Rust selecionados
(20integrações,3teclado,1controle remoto,3QUIC), Clippy all-targets -Dwarnings e
7MCP27grupos passaram. Uma prova crítica nova usa proxyUDP real: descarta a
primeira tentativa inteira, segunda conecta, só1sessão/ator e HP[1000,999]/energia99
após operação e duplicate com payloadkill. Pin inválido inicia só1handshake.
Native desconectado enquanto blackholed não reenvia após o prazo original:
observação200ms após cancel e mais3200ms sem novos pacotes. Nenhum teste por flag.

A política só captura Elapsed local antes de abrir stream/enviar Hello; erro
QUIC/crypto retorna imediatamente. Máximo3×3s+100/200msbackoff; Inspector/transport
stats expõem tentativas locais, sem alterar protocolo/owner/secret. Native observa
stop25ms também durante o connect e antes de publicar estado/ticket; socket/async
continuam fora da mutação. Sem heap/cache/timer novo offline, worker count e
buffers/semáforos do backend preservados. Custos sob falha/até9,3sTLS são explícitos;
Join/Resume/welcome/habilidades não são repetidos por esta política.

QUIC pareado1000×20hits, loss/resume/dedup/slowpeer, mais mixed/boss1000×100ações
passaram no build final. Todos os2000connects das duas cargas conjuntas precisaram
só1tentativa; isso NÃO prova causa do timeout anterior eliminada. O proxy prova
recuperação delimitada/posse/at-most-once, não disponibilidade geral de WAN.

Artefatos handshake-recovery-{audit,report,critical,integration,clippy,mcp}.json/logs
e handshake-recovery-verification.json. Regressões offline/rede negada mantidas.
Release único/current examples, nenhuma janela humana/app instalado/push. O ensaio
conjunto revelou lag residual em algumas réplicas; EAF-145 registra bytes/lag.
O codec já usa diferenças por campo e a maioria recebe deltas10Hz; não inferir
que o fallback full1Hz causou esse atraso sem medir gerador e ACKs.
