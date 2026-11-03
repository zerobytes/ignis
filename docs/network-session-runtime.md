# Sessões e réplica do multiplayer — EAF-141

## Decisão antes de código

Continuidade do estudo multiplayer-system-study.md e cargas EAF-140. Guilherme
incluiu transporte real no escopo em30/10. A implementação terá duas unidades:
domínio executável de sessão/réplica e integração QUIC à mesma fronteira. Teste
em memória é instrumento de correção, não substitui1.000conexões reais depois.

Escolha QUIC/Quinn em Rust: streams separados para sessão/operações/resultados,
datagrams substituíveis para movimento e snapshots. TCP único acopla tráfego;
UDP próprio transfere confiabilidade/congestionamento/segurança ao projeto.
Quinn oferece ambas semânticas com TLS; não acoplar gameplay aos tipos da crate.
Tokio/endpoint vivem fora da seção de mutação. Autoridade só recebe caixas de
entrada limitadas, aplica no tick e publica uma visão imutável; sockets/codificação
por conexão não seguram o lock do mundo. Nenhum endpoint/runtime de I/O no modo
offline. Primeiro macOS; biblioteca portável não é gate de outras plataformas.

Fontes oficiais consultadas: https://quinn-rs.github.io/quinn/quinn/data-transfer.html
https://docs.rs/quinn/0.11.12/quinn/ e exemplos oficiais da mesma versão.
RUSTSEC-2026-0342 descreve fila ilimitada de datagrams vazios: backend só será
integrado com quinn-proto>=0.11.17 no Cargo.lock; não ignorar certificados nem
usar exemplo insecure_connection. Certificado efêmero local confiado por pin
explícito no cliente; chave privada nunca vai para logs/inspector/repositório.
https://rustsec.org/advisories/RUSTSEC-2026-0342.html

## Domínio e limites

- ID lógico do ator e época do mundo no protocolo; slots/generation internos
  permanecem privados. Ator remoto vem da sessão atribuída, nunca do pedido.
  Fila não aceita position/damage/edit_scene, arquivo ou dispatcher administrativo.
- Sessões escolhem um Player livre autorado; controle exclusivo via BindControl.
  Desconectar libera/neutraliza input e reserva retomada por prazo, preserva corpo
  vivo e efeitos comprometidos. Conta persistente/spawn de Player é outro contrato.
- Segredo CSPRNG só para retomar a sessão; comparação constante, Debug redigido.
  Retomada sobe geração de conexão, invalida entradas antigas e conserva ledger.
  É autenticação de permissão temporária, sem conta/serviço remoto obrigatório.
- Movimento latest-wins, sequência por conexão e validade curta em relógio de
  admissão; pacote velho/vida errada não renova lease. ACK é do input consumido.
- Operações confiáveis têm sequência por sessão, fila16 e cache64resolvidas;
  pendentes separados e dedup inclusive após retomada. ID antigo nunca executa
  novamente mesmo se resultado saiu do cache. Admissão não é commit/persistência.
- Resultados de invocação não dependem do anel global256: tap opt-in por tick,
  limitado por entradas e atores, sem alocação/retenção desse canal offline.
  Started é progresso; Completed/Launched/ChannelStarted ou rejeição resolvem
  invocação. Efeitos posteriores pertencem ao estado/eventos de gameplay.
- Before/after tick conservam ordem do executor/vidas. Fonte/alvo/life/posse são
  revalidados antes de fila/custo; cancelar só execução do próprio personagem.
- Visão pública: atores/poses/vida/apresentação explicitamente permitidos, nunca
  memória/decisões NPC nem saldos alheios. Recursos/cooldowns do dono são privados.
  Todos1.000visíveis do combate entram; câmera não reduz participantes.
- Full/delta com época/tick/base explícitos, quantização milimétrica só de
  apresentação em i32 mm (autoria continua ±10000m), parse limitado e atomicidade antes de aplicar. Baseline
  perdido solicita full; estado velho não ressuscita indivíduo. Cache pequeno
  compartilhado por tick/baseline e buffers limitados; nada de cena JSON por tick.
- Protocolo próprio versionado, não dump de structs Rust. Backend deve limitar
  frame/streams/datagrams/filas/ressync; latest substitui snapshot não iniciado.
  Cliente lento não pausa simulação. Cripto não substitui autorização de ator.
- World rebuild invalida domínio/conexões antes de novas intenções. Scene88/
  replay97 preservados se só adicionar dados transitórios fora de State salvo.

## Checklist estático prévio

1. Aplicar por tick/ator, coalescer movimento; não varrer todos pacotes para cada
   ator nem serializar snapshot administrativo para replicar.
2. Filas16/ledger64/pendentes por sessão e tap somente quando ativo, buffers de
   codec reutilizáveis. Snapshot comum capturado uma vez; I/O fora do mundo.
3. Tipos de mundo/ator/sessão/conexão/vida distintos; sem slots físicos como ID.
4. Mesmo dispatcher/executor/cancel/atomicidade; ACK e resultado distinguíveis.
5. Orçamentos independentes por sessão e limite global; retry não duplica custo.
6. Nenhum socket/GPU/conta no offline; inspeção MCP/native antes/junto de UI.
7. Uma autoridade de escrita, sem thread por personagem/conexão no domínio.
8. Timeout/retomada/rebuild/vida e alvo trocado invalidam entradas, não reusam dono.
9. Só provas críticas de apropriação, dedup/rollback/vida/timeout, codec atômico,
   baseline e1.000sessões. Depois1.000QUIC reais/perda/cliente lento/MCP.
10. Custos mantidos e limites declarados: projeção, bytes/observador, fonte de
    carga e falta de persistência/predição/cliente gráfico até seus próprios gates.


O tap é opt-in e observa somente IDs de invocação registrados pela autoridade;
remove watches terminais e ignora aftermath de entrega. Eventos carregam dados
nativos só dentro do processo; correlação request/slot/generation não vai ao wire.
Cancel também resolve invocação original pendente. Estado de jogo/replay não
serializa o observador. Fila de resultados pode falhar fechada se exceder limite.

Métodos de domínio recebem sessão/conexão de envelope confiável do backend, nunca
campos que o cliente possa escolher. Operações já admitidas na sessão antes da
retomada continuam uma vez; novas entradas da geração antiga são recusadas.

## Revisão implementada

Observer transitório privado não entra em State/replay e fica None offline.
Pending16 por sessão limita watches16×atores; recent/outgoing64 não crescem.
Latest input e reserva10s usam relógio confiável recebido do backend; cliente não
escolhe timestamp nem identidade do envelope. Resume invalida conexão antes de
liberar/revincular controle: falha inesperada deixa sessão desconectada, nunca
conexão antiga ativa com handle inválido. Nenhuma chave/segredo no inspect.

Frame captura ordem lógica uma vez; appearance compartilhada por Arc. Decoder
limita1MiB/1024atores, valida inclusive o baseline do encoder, parseia inteiro em
patches antes de mutar, recusa overflow/vida regressiva/trailing bytes. Cache,
fragmentação e publicação compartilhada pertencem ao backend seguinte. Alocação
de Vec/BTreeMap por sessão, watches e patches é custo limitado conscientemente
mantido; não há cópia de Scene, acesso GPU, socket ou lock novo no domínio.

Três provas críticas novas cobrem1.000posses, fairness com fonte cheia, recursos/
resultados além do anel256, segredo/geração, lease/vida/cancel alheio/retentativa,
cache expirado sem executar novamente e full/delta malformados/ressync atômicos.
Elas protegem autorização/atomicidade/formato, não detalhes visuais ou cada campo.

Gates:68Rust release, Clippy all-targets -D warnings, buildf8b11dd7370eac27,
9MCP headless/33grupos (report/audit em artifacts/network-domain-*).
Scene88/replay97 preservados. Domínio implementado não é transporte real,
que continua em EAF-142; sem aceite gráfico/WAN inferido.
