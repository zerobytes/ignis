# Admissão e índice de comandos por ator — EAF-133

## Decisão anterior ao código

Meta:1000Players podem agir no mesmo tick do mesmo mundo, mantendo single player
offline. O antigo limite global16 não reserva vaga para cada participante. Apenas
aumentá-lo deixa o executor filtrar todos os pedidos para cada ator, produzindo
actor_count×entries. Um worker por cliente não resolve a ordem/atomicidade e
adiciona sincronização. Escolha: até16pedidos por ator/tick, orçamento global
limitado pelos slots reais×16, índice de frame reutilizado e ordem por InstanceId
inalterada. Cancelamentos mantêm a fase anterior e a ordem global; pedidos de cada
ator mantêm sua ordem de admissão. Recorded conserva o log total4096.

O índice usa cabeças/caudas por ator e próximo por entrada, índices16bits com
sentinela, no máximo16384pedidos. Sem mapas/alocações por pedido ou sort por tick;
limpeza linear dos buffers e agrupamento em O(atores+pedidos). Recursos, vida e
writes transacionais continuam locais e não são copiados para simular paralelismo.

No contínuo, controles são estado imediato: várias amostras no mesmo tick
substituem o estado e não precisam ocupar várias vagas de histórico inexistente.
O orçamento considera atores tocados, com reserva de liberação; sequência total
continua monotônica para cada comando aceito. Bind/release continuam aplicados em
ordem, com handle novo e neutralização. Limitar artificialmente rebind por tick
foi rejeitado porque impediria retomar controle de um jogo pausado após perda de
foco. A coalescência evita consumir vagas dos outros; limitação de mensagens/CPU
de conexões não confiáveis pertence à camada de sessão/transporte futura.
Modo gravado continua contando/grava cada evento para reprodução exata.

Reutilizar o índice de placements do RuntimeWorld para IDs e a ordem cacheada para
buscar serial de instância; evitar novo mapa redundante e varredura do mundo em
cada pedido. Remoção/reuso continuam exigindo validação da referência generacional.

Scene87 permanece; replay93 registra a política de admissão maior com limites por
ator. Pools de entregas/origens/buffs e orçamento de ruído não são ampliados neste
recorte. A carga será declarada como movimento+ações diretas sem projéteis/buffs/
ruído/NPCs, não como validação de todos os tipos de combate ou clientes de rede.

## Checklist estático prévio

1. Admissão por comando; construir índice uma vez por tick e iterar pedidos uma
   vez na fase de cada ator. Nenhum worker/lock/relógio novo.
2. Scratch dimensionado pelos atores, next limitado por entradas; reutilizar
   capacidade. Mode continuous não cria logs por amostra de controle.
3. Contagem por ator u8 até16, índice de entrada u16 até16384, identidade/vida e
   sequência64bits. Nenhuma mistura com grants/buffs/slots de entidade.
4. Índice de placements e ordem por instância existentes substituem buscas
   repetidas; cabeças/caudas preservam FIFO sem reordenar por alvo/posição.
5. Vaga por ator impede esgotar o orçamento dos demais; log recorded4096 e
   demais pools mantidos. Fonte/slots inválidos rejeitam antes de mutar counters.
6. Nenhuma alteração de GPU/I/O no tick. Workload headless usa a API compartilhada.
7. Mesmas fases e commits sequenciais. Cancelar antes de avançar/começar casts,
   intenção NPC e snapshot de vida preservados; nenhuma semântica paralela nova.
8. Reset/edição/load recriam contadores/índices; step limpa cotas e replay/prefixo
   recompõem cotas de pedidos pendentes. Reuso de slots não adota fila antiga.
9. Somente um novo cenário crítico de justiça/identidade/ordem/rollback/replay1000;
   reaproveitar cobertura de habilidades/canais/controles/lifecycle. Workload
   reproduzível mede simulação e admissão separadamente, com resultado de estado.
10. Mantidos: física real, candidatos/recibos, limites de outros tipos de efeitos,
    tickfixo60Hz e serialização de recibos por comando. Sem ganho de FPS/cliente
    gráfico/rede inferido de CPU headless; saturação de transporte é outro gate.

## Revisão final e evidência

1. Índice FIFO construído uma vez em FrameScratch::index_entries, O(A+E), fase
   cancel global antes de cast/intenção; execução continua por InstanceId.
2. heads/tails pelos slots reais, next reutilizado e limitado; controles contínuos
   aplicam estado imediatamente sem log por amostra. Sem clone de mundo/transação.
3. Quota u8/ator até16, índices u16/sentinela65535, sequences u64; lookup de ID por
   placement e serial por busca binária com referência generacional validada.
4. Removido filtro inteiro de entries por ator; limpeza O(A), sem sort no tick.
5. Global real16×slots e log recorded4096 preservados; rejeição antes de contar;
   prefixo de replay reconstrói quotas pendentes. Bind/release FIFO sem limitador
   artificial de rebind em pausa; rate limiting de transporte segue pendente.
6. Sem GPU/I/O por tick novo; JSON é fronteira de comando já existente.
7. Mesma autoridade sequencial, commits locais, cancel/custo/vida inalterados.
8. Admission::new em reconstrução e fill(0) por step; índice recriado por tick;
   reset/load/edição não carregam contagem antiga nem identidade reutilizada.
9. Um teste crítico novo em scalable_actors:1000ações, spam de input, limite por
   fonte, handles antigos, rejeição atômica e replay/prefixo.121Rust selecionados,
   Clippy all-targets e13cenários MCP/54grupos passaram, incluindo offline com rede
   negada e consumidores da escultura eb8e604. Nenhum teste por detalhe trivial.
10. Mantidos custos de JSON/recibos, física Rapier, pools pequenos e scans de
    visada. A carga mostra que admissão correta não basta para cumprir60Hz.

Carga reproduzível: `cargo run --locked --release --example runtime_load --
--players 1000 --ticks 600 --warmup 60`.1000corpos Player dinâmicos, piso/colisões/
oclusão reais, todos movem;1000ataques diretos em pares adjacentes a cada6ticks.
Todos receberam110impactos (vida999890);110000ações e661000controles admitidos,
zero histórico retido. Sem NPC/projétil/buff/ruído/render/rede. Inicialmente pares
entre linhas eram ocluídos corretamente; a receita corrigiu apenas pareamento.

Medição local sem outra suíte concorrente,60warmup+600ticks sem pacing: dispatch
p95=0,765ms; simulação p50=1,854ms/p95=108,271ms/p99=110,891ms, média19,578ms;
total p95=109,036ms. Máximo200,554ms é amostra local sujeita ao host, não contrato.
As ações ainda percorrem todos os corpos por visada e não cabem em16,7ms nos
picos. EAF-134 trata broadphase mantendo teste estreito e todos participantes.
Não declarar suporte a combate1000 sustentado, FPS gráfico ou multiplayer entregue.
