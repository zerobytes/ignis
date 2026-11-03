# Controles remotos nativos — EAF-144

## Decisão e checklist antes de código

Continuidade7f0495f/EAF-143 e requisitos EAF-117. Mesmos comandos no MCP e na
janela jogável: input persistente/release, seleção/picking público, slot/cancel e
mira espacial por cursor/plano explícito. O servidor continua atribuindo ator,
revalidando vida/identidade/alvo/custo/LOS e executando, sem física do cliente.
Editor autoral continua local; modo offline mantém controles existentes.

Heartbeat100ms na thread de I/O do cliente, não no lock/render loop da janela:
shader/capture pesado não deve vencer o lease250ms por segurar o Service. Caixa
latest única de input (hold/raw/neutral), sem fila de movimentos históricos.
Hold captura mundo/sessão/conexão/vida/estado vivo; morte/reconexão/vida diferente
apagam hold antes de enviar. Nunca restampar automaticamente input antigo no novo
indivíduo. Raw explícito continua disponível e sequência/vida inválida não toma
posse de hold válido; cliente de baixo nível permite probes de transporte.

Handle local de input escopado e serial sem reuso permite release só se ainda
for do teclado. Comando posterior por MCP ganha posse; perder foco com handle
antigo não apaga esse comando. Modificadores/keyup enviam input próprio atualizado;
troca do contexto vivo/conexão limpa teclas até entrada física nova. Neutralização
é intenção0 do mesmo contexto, sem matar/despawn/teleporte nem liberar cast pago.

Metadata estática de grants (slot/id/selection) só no welcome privado do dono;
alive/cast_execution só no owner state. Campos JSON opcionais/default para
compatibilidade aditiva do protocolo2, sem layout binário/Scene/replay novos.
Nenhum saldo/IA/grant alheio é publicado. Gameplay UI não consulta a cena local.

Auto-ID aceita apenas uma operação ainda não admitida por vez (raw mantém janela16).
É a escolha inicial para impedir lacunas/reexecução sem inventar outbox durável.
Resultado Received/Queued/Started reconhece ID consumido; commit final continua
separado. Rejeição antes da admissão não avança contador. Input é latest substituível;
habilidade é intenção explícita, sem repetir ao manter tecla nem auto-retry de dano.

Picking usa raio e volumes da projeção pública/câmera atual; obstáculo próximo
impede selecionar ator oculto pelo raio. Seleção contém id+vida, nunca slot ou
conhecimento NPC. Mira usa o plano explícito/altura atual do dono; não prevê alvo.
Serviço compartilha seleção/slot/cancel/aim com janela e MCP. Erro não move corpo
local e desconectar transporte não faz fallback de controle para single player.

1. Frequência: hold10Hz na I/O, input latest; seleção sob pedido, slot por keydown.
2. Memória: um hold/token/scope, contador monotônico, fila16 operações existente,
   uma auto-op não admitida; sem log por frame/tecla.
3. Identidades: world/session/connection/actor/life/alive tipados, token local
   escopado não é credencial de rede e não sai no protocolo de jogo.
4. Reuso: mesmo executor/dispatcher/ray/geometry/renderer; nenhuma física paralela.
5. Bounds:100ms/250ms,16grants,32recursos, seq/counter checked e buffers existentes.
6. I/O/JSON continuam fora da mutação; UI apenas publica intenção em canal limitado.
7. Autoridade única, heartbeat independe do lock da GPU/Service.
8. Foco/posse/vida/reconexão/reset invalidam stale; estado remoto só sai explicitamente.
9. Só novas provas críticas de posse/vida/foco/dedup/ordem e integração2processos;
   reutilizar regressões offline/controles/QUIC/render afetadas, sem teste por botão.
10. Custos mantidos: owner10Hz, lookup público, RTT limita admissão automática; sem
    predição/interpolação/compensação de latência ou aceite físico/FPS/WAN implícitos.

Decisão de recuperação: rejeição antes da admissão não autoriza reutilizar o ID
com outro payload enquanto respostas antigas ainda podem chegar. O auto-ID fica
fenceado até admissão conhecida, retry explícito do cliente raw ou resume (R),
que fornece ledger/geração novos. Trocar de vida NÃO reinicia essa sequência.
Não repetir dano automaticamente nem adivinhar ACK por observar uma rejeição
antiga no anel. O input continua independente e com token/vida próprios.

## Uso do contrato

| Comando MCP/dispatcher | Resultado |
|---|---|
| `set_multiplayer_input {input:{x,z,mode}}` | devolve `control`; mantém intenção a10Hz até substituição/release/invalidação |
| `release_multiplayer_input {control}` | neutraliza somente se o token ainda possui o input; token antigo retorna `released:false` |
| `inspect_multiplayer_controls` | seleção, revisão e operação emitida; não expõe token/segredo |
| `select_multiplayer_target {target:{actor,life}}` | seleciona ator público atual; `null` limpa |
| `pick_multiplayer {x,y,width,height}` | resolve o primeiro volume público no raio; chão/prop à frente impede selecionar ator |
| `use_multiplayer_ability {slot,target?,aim?}` | usa grant do próprio dono e auto-ID; actor aceita seleção corrente, self recusa alvo externo, spatial exige aim |
| `use_multiplayer_ability_at {slot,x,y,width,height,plane_y?}` | spatial no plano explícito ou altura atual do dono, câmera remota corrente |
| `cancel_multiplayer_ability` | solicita cancel do execution informado pelo owner; servidor revalida |

A janela jogável (`--window --mcp`) usa WASD/setas, Shift/Ctrl, clique esquerdo,
1..6, X e R para os mesmos comandos. Habilidades espaciais usam cursor/plano na
altura do dono; grants16 continuam disponíveis por MCP/API. R faz resume e
ressincroniza o ledger, sem repetir automaticamente ações. O editor `--editor`
continua autoral/local e não encaminha suas teclas ao personagem remoto.
Iniciar host/conectar/certificado seguem quic-transport.md; este incremento não
adiciona lobby ou contas. `disconnect_multiplayer` sai explicitamente da réplica
para o mundo local; queda de conexão mantém modo remoto e não assume corpo local.

Sem mistura automática de pipelines: clientes que usam a API raw devem acompanhar
admissão/IDs, sobretudo antes de voltar à API de auto-ID. Resultado de enqueue
local (`submitted:true,committed:false`) não confirma execução/pagamento. Rejeição
pré-admissão pode exigir resume ou retry raw explícito do mesmo ID; a UI nunca
substitui silenciosamente uma intenção cujo estado ainda é desconhecido.

## Revisão e evidências

Build `eb4f37f3b7a013c0`,129inputs, protocolo/ALPN2 e Scene88/replay97 mantidos.
26Rust selecionados (20integrações,3teclado existentes,1novo crítico de
posse/vida/geração/admissão e2QUIC), Clippy all-targets -Dwarnings e10MCP/40grupos
passaram. O único novo Rust e o cenário integrado protegem comando aplicado ao
indivíduo errado, release de outro dono e repetição de cobrança/dano; nenhum
teste novo por botão/slot/campo. Regressões offline/replay/render existentes
selecionadas por impacto, incluindo execução com rede negada. Fmt dos arquivos
alterados segue estilo existente; o módulo de exports mantém sua ordem histórica.

MCP2processos: hold650ms supera lease250ms, renovação continua durante primeira
captura GPU, token antigo não apaga input posterior, release atual neutraliza.
Picking e outline remoto, recusa atômica de vida velha, actor/self/spatial,
cast/cancel sem pagar, duplicate de hit com payload kill não mata/cobra novamente.
Morte/revive e resume invalidam hold antigo, input novo funciona, antigo release
não o apaga. Cliente fica tick0 até disconnect explícito, então offline6ticks.
A captura selecionada foi inspecionada: dois corpos, alvo com outline amarelo.
Não equivale a pressionar teclas/foco físicos ou aceite humano da janela.

QUIC1000/20.000hits: todos players HP980/energy80, história0,603ticks e réplica601.
Conexão2066,113ms, combate3061,856ms, CPU step p953,631ms; clientes UDP22.732.338B
subindo/375.544.548B recebidos,470pacotes codificados compartilhados. Cliente lento
encerrou em3001,399ms, sem bloquear mundo. Loopback/corpos dinâmicos/pares; não
extrapolar para WAN, FPS, controlador misto ou chefe compartilhado pela rede.

Revisão10itens: (1)hold10Hz independente de desenho; (2)um mailbox latest/hold,
serial e auto-op limitada, sem histórico; (3)Context tipado conserva vida e
conn, token local não sai na rede; (4)mesmos Input/dispatcher/geometry/renderer;
(5)16grants e budgets do transporte preservados; (6)apenas publicação em canal
sob Service, socket/codec na I/O e GPU fora do lock da réplica; (7)autoridade
única, sem simulação local remota; (8)sync limpa contexto/seleção, OS-repeat não
restaura teclas após mudança e editor bloqueia roteamento remoto; (9)provas
críticas e gates proporcionais acima; (10)lookup público sob demanda, cópia de
token por comando e RTT da admissão mantidos. Sem interpolação/predição/lobby/
conta/checkpoint durável ou garantia60Hz. Não mudar ID desconhecido após rejeição.

Artefatos `remote-controls-{audit,report,retention}.json` e logs, MCP
`remote-controls-verification.json`, `controls-{held,selected}.png` no workspace
isolado. Release corrente e exemplos preservados;4binários próprios supersedidos
sem uso removidos (62.740.848B), sem limpar dependências compartilhadas. Nenhuma
janela humana, app instalado, push ou publicação alterados. Próximo gate: cargas
KCC/mixed/chefe compartilhado com clientes reais e métricas de convergência/bytes.
