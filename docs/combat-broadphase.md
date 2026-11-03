# Broadphase do snapshot de combate — EAF-134

## Decisão antes do código

A carga EAF-133 mantém1000corpos reais e executa1000ataques a cada6ticks, com
p95de simulação108,271ms. A visada de cada ataque percorre todos os corpos.
Escolha: BVH balanceada por mediana, AABB XYZ fechada como filtro conservador,
reutilizada por tick, sem dependência nova. Primeiro consumidor é visada de
habilidades; teste estreito box/wedge e flags blocks_vision permanecem idênticos.
Não retirar players/colisões/oclusores nem mudar tick60Hz. Grid foi preterido por
replicar corpos grandes e complicar segmentos/candidatos duplicados; usar o BVH
físico sem sincronização própria arriscaria estado antigo após spawn/reuso.

O snapshot nasce após population/generators e atualização das posições, antes
sensores/combate/física. Somente entidades presentes entram, incluindo visuais
que optam por bloquear visão. Leaf contém índice de Entity e centro do snapshot;
nenhuma consulta muda conhecimento/posição ou fase. Simulation recompila buffers
reutilizados cada tick; edição/reset/load recompõem Simulation; lifecycle aparece
no rebuild seguinte à admissão da fase. World admite índice opcional: consumidores
avulsos/testes usam caminho linear equivalente, sem cache implícito obsoleto.

Slabs fechados com o mesmo limiar paralelo1e-7 cortam nós; folhas
mantêm candidato exato fechado. A ordem da BVH não é autoridade: somente consulta
booleana anyhit neste recorte. Projétil precisa preservar TOI/ID, áudio ordem de
produto float e área recibos de todos candidatos; não migrá-los implicitamente.
Scene87/replay93 sem novo formato porque resultados devem ser equivalentes.

## Checklist estático prévio

1. Rebuild uma vez/tick O(E logE), consulta O(logE+candidatos) em distribuição
   favorável; pior caso ainda linear quando todos volumes sobrepõem o segmento.
2. Vecs nodes/leaves reutilizados, stack pequeno fixo por iterador; nenhum Vec,
   sort ou mapa por raio. Scene/estado não são clonados.
3. Leaf índice/centro e AABB f32, nós tipados. Sem IDs de rede/ECS/ABI novos.
4. Reutilizar snapshot de posições existente; broadphase elimina testes/lookup
   global por ataque, narrowphase existente continua dono da semântica.
5. Teto de entidades herdado, árvore balanceada e stack dimensionada para todo
   domínio u16; nenhuma truncagem silenciosa nem limitação de vítimas.
6. Sem GPU/upload/arquivo no tick; medição headless usa workload já existente.
7. Mesmo dispatcher/fases sequenciais; índice imutável em consultas, sem locks.
8. Rebuild após mudanças de população/posições; reset/load/edit não persistem
   índice. Fallback não consulta índice de outro mundo nem estado pós-física.
9. Novo teste somente crítico: oráculo linear versus índice, fronteiras/segmentos
   degenerados/oclusor visual/wedge/lifecycle e replay por cobertura existente.
10. Custos mantidos: build BVH por tick, física Rapier, JSON/recibos e pools atuais;
    combate arbitrariamente denso pode exigir trabalho real quadrático. Medir
    mesma receita isolada; não inferir FPS, rede ou outros tipos de efeitos.

## Revisão final e resultado

1. Simulation reconstrói após population/generators e atualização das posições;
   consultas de habilidade usam uma BVH imutável antes da física.
2. Leaves/nodes Vec retêm capacidade; cada iterador usa32u32 no stack. Build por
   mediana/maior dispersão de centros, sem sort/alocação por raio ou clone de Scene.
3. Leaf u16 validado, links u32; centro/half f32 idênticos ao teste antigo.
4. Reutiliza positions_before_step. Fallback World.spatial=None faz scan presente
   linear; caminho normal passa Some do mesmo snapshot e evita BTree por candidato.
5. Até4096entidades autoradas mais reservas existentes, sem truncar; mediana
   balanceada limita profundidade a17 em todo domínio u16 e stack32 é suficiente.
6. Nenhum recurso de render/I/O novo, índice não é documento/replay persistido.
7. Consulta booleana anyhit pode mudar ordem de visitas sem mudar resultado;
   fase/vida/custos/InstanceId e geometria estreita permanecem idênticos.
8. Rebuild exclui ausência, move centro e troca índice de slots com população;
   edição/reset/load recompõem estado. Morte mantém corpo/oclusão como antes.
9. Somente1teste crítico novo:160raios/4096entidades, quatro orientações de cunha,
   caixas/visuais/flags, degenerados/quase paralelos/tangências, mover/remover/
   esvaziar/repor índice alto versus oráculo linear.111Rust existentes em8alvos
   e11MCP/42grupos selecionados passaram, com replay/lifecycle/offline rede negada.
   Clippy all-targets também passou. Nenhum teste novo por detalhe trivial.
10. Custo de rebuild por tick conscientemente mantido (inclusive ticks sem ataque),
    assim como scans de projéteis/área/áudio e pools antigos. Pior caso de volumes
    sobrepostos segue linear por consulta; não promete custo sublinear universal.

Build auditado `fc15e84187cbfd78`, main local após e193390, Scene87/replay93 sem
mudança de formato/dependência/renderer. Mesma receita runtime_load isolada:
1000Players movendo+1000ataques em pares a cada6ticks,60warmup/600medidos; todos
110impactos/vida999890 e zero histórico. Simulação p50=1,960ms/p95=2,515ms/
p99=2,588ms/máximo2,760ms; dispatch p95=0,739ms e total p95=3,252ms/máximo3,528ms.
Antes p95=108,271ms de simulação, mesma receita/host. Este workload coube no
orçamento local16,7ms das amostras; não é garantia de latência, renderFPS, MMO,
projéteis/buffs/ruído/NPCs ou clientes de rede. Pools são o próximo recorte.
