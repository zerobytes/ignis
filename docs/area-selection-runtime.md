# Seleção de área e diagnóstico congelado — EAF-137

## Decisão anterior ao código

Receita EAF-136:1000auras pequenas,1000pulsos por janela, apenas o próprio autor
atingido por aura; p95simulação78,607ms. Cada pulso faz lookup BTree e ordena os
1000participantes, inclusive Outside/Dead/RelationDenied. Não omitir rows nem
retirar participantes para melhorar números.

Escolha: posições por índice de Entity no mesmo snapshot espacial já construído;
classificar todos candidatos e congelar dados/status na hora da seleção; ordenar
apenas elegíveis para executar distância/InstanceId e aplicar limite. Resultado
completo dos recibos guarda rows por slot, com ordenação materializada sob leitura
em OnceLock. JSON e consultas públicas continuam devolvendo a lista completa
ordenada; clone/igualdade/serde preservam dados congelados, cache não é estado de
jogo. Uma consulta/inspeção não muda tick/histórico/efeitos/revisão.

O hotpath processa os selecionados na mesma ordem e copia seus resultados de volta
antes de registrar recibo. Outside/Dead/SourceExcluded/RelationDenied/Occluded/
TargetLimit permanecem disponíveis e não são recalculados de um mundo posterior.
Consultas select/select_from públicas continuam retornando Vec ordenado; somente
execução/armazenamento internos usam a representação adiada. Não usar snapshot
vivo/Arc sem congelar morte/relação/status por transação: transações anteriores
podem alterar o estado no mesmo tick. Preferimos Vec congelado simples à maior
complexidade de reconstruir candidatos de snapshots compartilhados.

Occlusão pode usar candidatos BVH fechados com limiar0 e teste estreito interior
original, somente Fixed/Dynamic. Fallback linear permanece; não usar limiar de
visada1e-7 que poderia perder movimento muito pequeno. Scene87/replay95 e JSON
inalterados porque resultados devem coincidir exatamente com o caminho anterior.

## Checklist estático prévio

1. Snapshot de posição uma vez por tick; por área O(atores+K logK) com K elegíveis,
   diagnóstico completo O(A logA) sob leitura. Pior caso K=A permanece caro.
2. Vec de rows congelado e Vec de selecionados; OnceLock só aloca cópia ordenada
   se lido. Memória extra limitada aos64recibos, custo de leitura declarado.
3. Entity index distingue actor slot; Option<Vec3> preserva presença/ausência.
   IDs/dados f32 e comparador distância/InstanceId idênticos, sem nova dependência.
4. Reutilizar Index.rebuild e oclusão estreita, sem BTree por candidato normal;
   query avulsa sem índice conserva fallback e resultado antigo.
5. Todos participantes/status/limites continuam; nenhum truncamento, redução de
   max_targets ou omissão de falhas para passar a carga.
6. Sem GPU/I/O novo; JSON/inspeção materializam diagnóstico completo sob demanda.
   Debug/inspector que pedir todos rows continua pagando esse custo, uma vez.
7. Mesma ordem de transação e efeitos, seleção congelada antes de aplicar; mortes
   e mudanças posteriores não reescrevem histórico/cache.
8. Índice recompõe após população/posição; reset/load/edit refazem. Cache de recibo
   pertence aos dados congelados, clone/serde não dependem de Scene corrente.
9. Só novo oráculo crítico: caminho linear anterior versus selecionados/rows,
   ordem, oclusão/limites/ausência, resultado congelado após mutação e serde/replay;
   reaproveitar cobertura existente afetada e mesma carga1000.
10. Mantidos: classificar todos, Vec por pulso, memória de rows/cache se observado,
    trabalho real de vítimas. Sem promessa universal de60Hz/render/rede.

## Revisão final e resultado

1. Classificação completa O(A), ordenar apenas elegíveis K; consulta pública ou
   leitura de recibo ordena todos rows como antes. Nenhuma vítima/candidatura omitida.
2. Index.positions é Vec<Option<Vec3>> reutilizado por Entity, recomposto no mesmo
   rebuild; rows/selecionados são possuídos pelo pulso. Cache OnceLock apenas se
   observado, limitado aos64recibos; clone copia dados e não copia cache derivado.
3. Candidates guarda por actor slot e normaliza serde, recusando duplicata/domínio
   inválido; serialize e Deref entregam lista completa ordenada. JSON/tipos autorados
   de Scene87/replay95 não mudaram; igualdade compara dados congelados, não cache.
4. World.position lê índice pelo Entity; fallback mantém mapa. Oclusão BVH fechada
   com limiar0 filtra Fixed/Dynamic e usa os mesmos kernels interiores; delta0 é
   tratado explicitamente. Não reutiliza política1e-7 da visada indevidamente.
5. Distância/InstanceId e max_targets são aplicados aos elegíveis; TargetLimit,
   SourceExcluded,Dead,RelationDenied,Outside,Occluded e falhas ficam em todos rows.
6. Nenhum renderer/I/O novo; inspector/serde pagam ordenação completa quando pedem
   dados. Não prometer melhora de custo de dump completo ou FPS a partir de CPU.
7. Selection.selected conserva a ordem de transações antiga, resultados Applied/
   EffectFailed retornam aos rows antes de registrar. Nada consulta mundo futuro.
8. Presença/posição vêm do mesmo snapshot; reset/load/edit recompõem. Cache do
   recibo é autônomo, permanece correto após esvaziar posições/vitalidade/índice.
9. Novo oráculo crítico compara a versão linear anterior com fallback/índice,
  12combinações de raio/limite/fonte/relação/oclusão, bordas/cunha/ausência/empate,
   selecionados/resultados, primeira leitura após mundo mudar, clone/serde/erro.
  102Rust existentes+2oráculos (104total),Clippy all-targets e16MCP/66grupos passaram.
10. Mantidos Vec de todos rows por pulso e custo linear de classificação, memória
    duplicada se diagnóstico observado, trabalho real de todos alvos quando K=A.
    Sem promessa universal de60Hz, capacidade de render ou clientes de rede.

Main após a58b6e3, build `8d0d1805caf7d57e` auditado fonte/binário/example.
Mesma receita1000Players movendo/1000auras por janela/pico2000volumes,60warmup+
600medidos:110000lançamentos e110impactos/vida999890 por jogador, zero histórico.
Simulação p50=2,060ms/p95=7,683ms/p99=7,908ms/máximo8,178ms; total p95=8,355ms/
p99=8,637ms/máximo8,873ms. Antes p95simulação78,607ms, mesmos participantes/
vítimas/diagnósticos.2ticks de drain separados (máximo7,323ms) confirmam último
pulso/expiração. Sem buffs/ruído/NPC/render/rede na carga; todos rows continuam
visíveis/serializados, sua leitura completa tem custo sob demanda.

Logs/relatórios correntes:artifacts/area-runtime-{integration,oracle,spatial,
clippy,build,mcp}.log, area-runtime-report.json e runtime-load-area.json.
Próximo recorte: buff/atributos ainda têm64contribuições/512bases globais.
