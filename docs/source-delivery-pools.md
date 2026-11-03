# Entregas e origens por ator — EAF-135

## Decisão anterior ao código

1000fontes precisam lançar projéteis no mesmo tick. Teto global64/32origens não
satisfaz essa premissa. Escolha:64projéteis por slot de ator e teto slots reais×64;
32origens retidas por slot e teto slots×32. Cobrar o slot mesmo após nova geração
impede que churn contorne quota; referência completa continua separando indivíduos.
Periodic/áreas/buffs/ruídos não mudam neste recorte. Single player offline mantém
mesmo núcleo e quota individual anterior, rede permanece opcional/futura.

Pool denso encapsulado e somente leitura fora dos mutadores; índice de posições
por fonte limita busca/contagem ao orçamento do autor. Push/clear/refresh/serde
reparam índice. Compactação estável escreve retidos sem expor índice parcial e
recompõe uma vez se houve remoção, preservando ordem de efeitos. Vec por fonte
foi preferido a árvore por entrada; domínio1024 é limitado, buckets crescem só
quando usados. Sem divisão em mundos/threads/ECS, sem apenas ampliar constantes.

Origem usa leaseu16, referência validada e mapa derivado para lookup, free-set
ordenado para reuso determinístico. Slots e buffers de snapshot são alocados
somente ao reservar nova origem, antes de pagar/cooldown. Cada slot reserva65bases
e64contribuições (tetos atuais); detach não aloca. Slots livres preservam buffers
para reuso, quota/global limitam memória. Coleta usa scratch por lease alocado,
sem máscara32bits. JSON registra capacidade real e dono do slot; serde reconstrói
índices e rejeita domínio/duplicata/quota inválidos. Replay94 explicita nova
política/estado; replay só aceita versão corrente como já era, Scene87 permanece.

Sweep pode usar o BVH do mesmo snapshot, com padding e limiar paralelos próprios,
sem alterar TOI estreito nem desempate por ID. O broadphase é conservador inclusive
com arredondamento f32; oráculo/limites cobrem tangências e trajetória degenerada.
Fallback linear permanece para API avulsa. Nenhum candidato real é retirado por
relação/visibilidade, e impactos continuam sequenciais.

## Checklist estático prévio

1. Admissão O(1)/quota por fonte e origem O(logorigens); índice espacial uma vez
   por tick. Compactação O(ativos), reindexar só quando houver mudança estrutural.
2. Dense Vec e buckets reutilizados, scratch de coleta reutilizado; sem nova cópia
   de mundo/Vec por projétil/frame. Snapshot reservado antes do commit.
3. Actoru16, leaseu16 até32768, referência64bits e origem de geração distinta;
   projéteis totais<=65536, não confundir índice de entidade/autor/lease.
4. Lookup por fonte/referência substitui scan global em capacidade/refresh/detach;
   não duplicar dados autorados. Futuras agendas reutilizam o mesmo pool.
5.64por fonte,32origens por slot e tetos reais impedem monopolização/churn;
   rejeição não paga nem renova cooldown. Buff64 ainda limita snapshot atual.
6. Sem I/O/GPU novo no tick, recibos existentes limitados; JSON só na fronteira.
7. Mesmas fases, ordem densa, TOI/ID; índice não escolhe autoridade ou elegibilidade.
8. Push/compactação/clear/serde e reserve/collect/reuse mantêm derivados; referência
   antiga nunca valida novo ocupante. Reset/edit/load recriam estado.
9. Somente prova crítica nova de1000lançamentos/leases altos, quota/rollback,
   impacto/lifecycle/serde/replay; adaptar cobertura existente à política por fonte
   e repetir carga reproduzível, sem testes por detalhe.
10. Mantidos custos de snapshot por origem, física real/TOI e demais pools/ruído;
   BTree aloca/libera nós na reserva/coleta, JSON de comandos e Rapier mantêm heap;
    não se promete zero alocação.64mil projéteis sobrepostos têm custo real,
    sem promessa de FPS/MMO/latência.

## Revisão final e evidência

1. Quota por fonte consulta bucket O(1); sweep usa BVH pré-física, origem usa
   lookup BTree por referência e reuso mínimo por free-set. Sem varredura global
   de projéteis para admitir um autor ou scan de slots para detach.
2. Pool<T> expõe slice somente leitura, push/clear/serde mantém buckets; write/
   finish_retained recompõem uma vez após remoção e preservam ordem. Sem rebuild
   de buckets quando só posição muda. Slots/snapshots e scratch de coleta retêm
   capacidade; BTree ainda aloca nós na reserva/coleta, custo explícito mantido.
3. Leaseu16, actoru16 e referência completa; até32768origens e65536projéteis em
   domínio1024, teto real pelo mundo. JSON de active continua lista, índice é
   derivado. Origens serializam dono/capacidade real e reconstrução valida quotas.
4. Detach usa source_mut pelo mapa, captura em buffers65bases/64contribuições
   reservados antes do custo; corrigida pré-reserva antiga64para65bases. Coleta
   marca só leases com referência completa e não retém histórico/recibos.
5. Quota64inclui projéteis de gerações anteriores no mesmo slot; quota32contextos
   também. SourcePool e origem rejeitam antes de pagar/cooldown. Fonte que preenche
   sua quota não impede outra; fixture concorrente foi adaptada a essa política.
6. Sem GPU/I/O ou novos comandos; inspector mostra capacidade total e por ator.
7. Sweep passa delta original (sem somar/subtrair origem novamente), margem8eps
   conservadora de arredondamento e padding, limiar1e-8; narrowphase e TOI/ID
   idênticos ao caminho linear. Não filtrar relações/visibilidade na colisão.
8. serde recompõe índices/reservas, coletar/reusar valida referência completa;
   cenário1000mantém lease1000após NPC removido e rejeita lease antigo reutilizado.
9. Um novo cenário crítico Rust em scalable_actors e expansão do oráculo existente
   para sweep/TOI/cunhas/bordas/9000m.136Rust selecionados (16origins,9bindings,
   110integrações,1oráculo), Clippy all-targets e14MCP/60grupos passaram. A correção
   final apontada por Clippy apenas deriva Default equivalente; prova crítica e
   oráculo foram repetidos no build final, sem repetir toda suíte existente.
10. Buff64/periodic64/áreas16/ruídos16globais continuam; pools ampliados não provam
    todos combates nem rede. Campos/slots/grants/vida mantêm seus próprios domínios.

Main local após dc1e713, Scene87/replay94; build `28cbb8961e9a9fdb` auditado
fonte/binário/example. Documento de cena/renderer/editor não mudaram. Testes críticos cobrem
1001lançamentos,64de uma fonte, rejeição sem cobrança/renovação e outra admitida;
origem alta removida, serde/índice, coleta/reuso, corrupção recusada e replay.
Outra rodada de1000impactos remove pool/recompõe índices e confirma vida99.

Workload reproduzível: `cargo run --locked --release --example runtime_load --
--players 1000 --ticks 600 --warmup 60 --delivery projectile` (pares exigem número
par nesta receita).1000Players reais movendo com colisão/oclusão,1000lançamentos a
cada6ticks;110000lançamentos,661000controles, todos110impactos/vida999890,
zero histórico e zero drain fora da medição. Sem NPC/buff/periódico/área/ruído/
render/rede. Simulação p50=1,969ms/p95=2,838ms/p99=2,891ms/máximo2,946ms;
total p95=3,670ms/p99=3,754ms/máximo3,846ms. Receita direta repetida isoladamente:
p95simulação2,519ms/total3,338ms, resultado idêntico. Sem promessa universal de
latência/FPS;64mil projéteis simultâneos não foram medidos. Logs/relatórios em
artifacts/source-pools-*, runtime-load.json e runtime-load-projectile.json.
