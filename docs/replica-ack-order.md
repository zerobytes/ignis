# Ordem de ACKs de réplica

## Preflight antes de código

Na consolidação EAF-145: lag de algumas réplicas chegou a~1s. Correção da hipótese
inicial: codec JÁ é por campo/varints/Arc, apply JÁ valida patches antes de commit
sem clone do mapa. Contadores mostram maioria dos frames em delta perto10Hz, não
fallback geral full1Hz. Não subir protocolo3 nem recriar mecanismos existentes.

Aumentar gerador2→8workers com endpoint UDP único piorou tempo/lag; isolar gerador
com vários endpoints/4workers e medir. Isso modela recursos de recepção separados
dos clientes reais; todas1000conexões TLS continuam reais e recebem todos atores.

Bug confirmado por leitura: receiver UDP escreve link.ack=tick sem guardar ordem.
ACK antigo ou None atrasado pode substituir um Some mais novo; o próximo delta
usa base anterior à réplica atual e é rejeitado NeedBaseline. É contrato real de
rede não ordenada, independentemente da causa observada do lag agregado.

Decisão: ACK monotônico por Link/conexão, None só no início, Some avança e velhos/
None tardio são ignorados. Novo Link no resume já começaNone. Mesmo mundo checado
antes, sem limpar Replica/ignorar validação/reatribuir ID. Counter agregado readonly
para ACK regressivo (duplicata igual não é regressão), sem log por evento.

1. O(1) no receiver UDP sob mutex já existente; tick/world mutável não consultado.
2. Sem fila/cache/heap/estado por tick; uma AtomicU64 por backend para diagnóstico.
3. Mesmo Option<u64>/world/Link, Scene88/replay98/protocolo2 preservados.
4. Não repetir encode/parse existente; baselines/cache12/32KiB/fallback1Hz mantidos.
5. Mesmo limite128datagrams/s, ciclos/TTLs e1MiB; não ampliar orçamento para passar.
6. I/O fora do mundo; sem GPU/arquivo/serialização no processamento de ACK.
7. Uma autoridade; a confirmação de apresentação não confirma gameplay/custo.
8. Nova conexão/Link reinicia ack; antigo datagramworld/generation não ganha link.
9. Uma nova prova crítica com Hub/bytes/Replica reais e ACKs reordenados entre
   frames0→6→12: próximo delta precisa aplicar sem perder base. Reusar QUIC/loss/
   resync/offline e cargas1000, sem teste por contador/flag.
10. ACK monotônico corrige esse caso; medir antes de atribuir toda latência a ele.
    Recursos do gerador e perda/burst UDP continuam limites medidos, sem FPS/WAN.

## Revisão e evidência — EAF-148

Buildf75f4a7ea9154d40/130inputs, Scene88/replay98/protocolo2. ACK regressivo é
ignorado por Link::acknowledge em O(1) sob mutex existente; duplicata igual não
conta como regressão e Some avança. Host inspector expõe reordered_acks agregado.
Nova conexão cria outroLink/None, world check anterior permanece. Não é ACK de
operação/pagamento. Sem cache/fila/heap/worker adicional ou mudança de codec.

Uma nova prova crítica usa Hub/Frame/codec/Replica de verdade: full0, delta6,
ACK6 e depois ACK0/None atrasados, delta12 aplica Current e conserva posição.
28Rust selecionados (inclui esse caso e proxyTLS/loss/1000×20hits), Clippy
all-targets e7MCP27grupos aprovados; offline/rede negada/ownership/replay/render
preservados. Sem teste para cada counter/flag. Retenção/artefatos em replica-ack-*.

Cargas com1000TLS/100.000ações cada,4workers de gerador/32endpoints, verificam
saúde/grounding/passos/entregas e todas1000réplicas no mesmo tick,1tick atrás da
autoridade após drain. Gate exige até12ticks (2períodos de snapshot) nesse ponto.
Durante combate, lag em ticks convertido a60Hz é estimativa, não latência pura:
mixedp95250ms/p99400/máximo1033,333; bossp95100/p99100/máximo150ms. Resultado
observado inclui cliente/IO/tick/polling: p95130,022/57,984ms respectivamente.
Métricas completas e recursos em joint-network-load.md, sem FPS/WAN/60Hz universal.

O contador de ACK regressivo ficou0 nesses ensaios. Logo, o caso determinístico
foi corrigido, mas não explica por si só o lag agregado observado. Perfil8workers/
1endpoint teve mixedp95650ms/bossp95900ms de lag, finais32/68ticks. Perfil4/32 teve
finais1/1ticks e apenas1000fulls iniciais por ensaio. Dois parâmetros mudaram
juntos; não é experimento isolado de ganho de throughput ou causalidade. Sem
atribuir resync ao tamanho de frame por inferência: o codec já é por campo e
parse-validate-commit. No mixed há23204NeedBaseline, refletindo também deltas em
voo contra base anterior; cache de baselines no cliente/predição/interpolação são
contratos futuros, não requisitos concluídos aqui. Dados/limites permanecem visíveis.

Revisão10itens: frequência no ACK; custo O(1); Option/world/Link preservados;
encoder/parser reutilizados; budgets originais; I/O fora do mundo; autoridade
única; reset porLink novo; prova de causalidade de base+gates proporcionais;
latência/memória/recursos do gerador explícitos. Nenhuma troca de formato por hipótese.
