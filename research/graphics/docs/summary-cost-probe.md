# Custo da avaliação dos resumos

Continuação autorizada do experimento de qualidade: reduzir trabalho repetido preservando seleção, radiância e transição. Worktree gráfica isolada; sem mudança de runtime/MCP/cena.

## Decisão e revisão antes da implementação

O modo gradual avalia32 lobos por vértice e ancestral ativo. A radiância de cada resumo depende apenas de sua distribuição e dos uniformes de luz/visão/material; não da posição de cada vértice neste modelo. Calcular uma vez por nó ativo em compute e ler o resultado no vertex shader preserva a ordem das misturas e a transformação de altura. Alternativa de compor uma transformação afim por nó economizaria mais, mas reordenaria operações/altura e exigiria prova própria de cobertura. Cache entre frames introduziria regras de invalidação; o primeiro recorte proposto recalculava nós ativos a cada frame e media o passe completo, inclusive preparação. A decisão final abaixo incorpora o custo observado dessa sincronização.

Checklist prévio (docs/performance.md): frequência por frame; buffers residentes e limitados por árvore; nenhum clone/alloc na seleção; mesma precisão f32 e índices; remover avaliação redundante de lobos; compute linear nos nós com retorno cedo para inativos; um passe compute + até dois draws, incluindo barreira na medição; nenhuma espera/readback de runtime novo; recalcular a cada frame garante atualização de luz/câmera/perfil sem estado antigo; comparar pixels lineares e cobertura, não só PNG; custo extra de despacho/memória deve aparecer, sem garantia universal de ganho/FPS.

## Implementação final e decisão sobre reutilização

A preparação em compute por frame remove a repetição de32 lobos em cada vértice, mas sua sincronização com o render também custa tempo. A implementação final mantém um buffer de radiância de16bytes por nó (349.520bytes/341,3KiB por árvore), calculado em compute quando a chave muda. Calcula **todos** os nós nessa atualização, de modo que novas ativações durante a aproximação/afastamento ou troca de perfil não leiam valores antigos. A chave exata contém bits de tilt, ângulo de luz, expoente e amplitude especular. As distribuições e materiais agregados são imutáveis por `Uploaded`; novo upload possui novo cache. Matriz de câmera, distância, translação, jitter e resolução não afetam a radiância neste shader de direções uniformes, portanto não invalidam esse buffer.

Isso é uma propriedade do modelo atual, não um contrato genérico de iluminação: visão por posição, luzes locais, sombras, animação/material mutável e GI exigiriam outras entradas e estratégias. Mudança contínua de ângulo/luz pode exigir atualização a cada frame e não recebe o ganho do caminho reutilizado. Não se calcula a média de frames, não se interpola cache antigo e não se atrasa a atualização.

Cada nó também registra seu ancestral ativo mais próximo, calculado em ordem pai→filho na CPU. O shader pula apenas pesos zero, sem reordenar qualquer mistura ou multiplicação de altura. As ligações são validadas antes de upload; pais e saltos inválidos deixam o estado e o cache intactos. Sem nenhum peso ativo, usa o shader direto e dispensa compute. O controle completo também tem shader direto dedicado para não ser prejudicado por ramos dos modos experimentais.

## Benchmark e controles

Cinco métodos: gradual anterior; otimizado com radiância válida; geometria completa; otimizado sem reduzir os65.536 triângulos; otimizado forçando atualização em cada frame. Os métodos são intercalados em31 rodadas, com rotação da ordem, depois de aquecer todos. A atualização forçada recalcula toda a árvore em **cada repetição medida**. Seu intervalo começa antes do compute e termina depois do draw, incluindo a transição entre passes. Compute e draw também são reportados separadamente. O caminho reutilizado tem zero preparação somente porque a chave é válida.

O controle sem redução usa índices residentes na mesma ordem de folhas do seletor, o mesmo shader/pesos e cache, permitindo separar redução de topologia da mudança de implementação. Controlar somente o número de triângulos mantendo caminhos indexed/non-indexed diferentes confundiria os efeitos; ambos são indexed neste controle. O controle de geometria original mantém seu caminho direto usual. Frequência/carga da GPU e efeitos de scheduling ainda variam; comparar razões por pose e faixas, não converter uma medida isolada em FPS.

Capturas:178 poses ×3 perfis, mesmos percursos do EAF-66, com referência2.048spp e candidatos128spp. Verificação antes/depois em RGBA linear; comparação com os PNGs já entregues no estudo de qualidade. O novo relatório mantém câmera, perfis, recorte comum e grade de pixels, e mostra o custo quente/frio e o erro herdado contra a referência. Não trata identidade com o método anterior como prova de fidelidade perfeita à referência.

Os11 ensaios GPU de cache cobrem primeira preparação, distância, translação, perfil, luz, visão, expoente, amplitude, retorno a configuração antiga, malha distinta e resolução/jitter. O ensaio de resolução/jitter congela a seleção para isolar a radiância; não valida LOD adaptativo em512². Cada mudança relevante compara com a avaliação anterior sem cache, e repetir a pose deve reutilizar valores e preservar pixels. Ligação inválida é rejeitada sem alterar cache/pixels. Trocar de malha não reutiliza nem invalida indevidamente o buffer de outra malha.

## Reprodução

```
cargo run --locked --release --example appearance_probe -- --summary-cost
python3 scripts/report_summary_cost.py
node scripts/verify_quality_report.mjs artifacts/appearance-probe/cost
```

`--summary-cost-smoke` executa um recorte exploratório de4 poses brilhantes. A entrega corrente é `artifacts/appearance-probe/cost/index.html`, JSON/CSV e montagem PNG; o relatório de qualidade recebe link para ela. Ambos usam os mesmos controles de perfil/câmera, verificados pelo harness JavaScript. Interação nativa do navegador continua sendo um gate separado.

## Revisão final de performance

1. **Corrigido:**32 lobos avaliados por nó só na atualização, não por vértice/ancestral a cada desenho.
2. **Adequado:** buffers residentes; seleção reutiliza capacidade. Chave pequena em `Cell`, sem cópia/lookup de distribuições por frame. Buffer adicional de índices pertence apenas ao controle sem redução.
3. **Adequado:** f32, mesma ordem de mistura e altura; salto usa campo já disponível de16bytes do estado. Sem representação afim que reordene operações.
4. **Corrigido:** ancestrais de peso zero pulados; invariantes de luz/visão reaproveitados. Atualização completa é custo deliberado para permitir qualquer nova ativação sem scan de validade por nó.
5. **Mantido com motivo:** limite21.845 nós/128² células; update de pesos e saltos é linear, seleção CPU permanece significativa. Não há evidência de bilhões de triângulos.
6. **Corrigido:** um compute somente quando necessário; shader direto sem resumo ativo. GPU refresca o buffer inteiro em mudanças relevantes; essa despesa aparece no cenário frio. Não há download de radiância para CPU.
7. **Adequado:** compute→draw ordenado por dependência de recursos; timestamps resolvidos depois da submissão terminar. Esperas/readbacks pertencem à instrumentação offline, não a nova lógica de gameplay.
8. **Adequado no modelo:** chave explícita + propriedade por malha, publicação da chave após sucesso GPU; erro de estado/atalho não altera cache. Novo upload invalida por construção. Modelo de direções uniformes documentado.
9. **Adequado:** corte original e qualidade preservados; matriz de cache/retorno/PNG e regressões headless. Sem API exclusiva da UI nem mudança de cena/replay/runtime.
10. **Mantido com motivo:** CPU de seleção, upload de índices/estados, distribuições e geometria residentes, atualização quando luz/visão mudam e limitações de cobertura do proxy. Ganhos são GPU sob condições medidas; FPS total da engine não foi estabelecido.

## Resultado medido — Apple M4 Pro / Metal

Execução completa:178 poses ×3 perfis,31 repetições por método/pose,256² e uma amostra por pixel nos timestamps. A tabela contém a **mediana das razões pareadas por pose**, perfil “Mais resumos”; não é razão entre tempos médios nem ganho de FPS.

| Percurso | Otimizado/anterior | Otimizado/geometria completa | Reduzido/mesmo shader sem redução |
| --- | ---: | ---: | ---: |
| Difuso35° |0,336|0,884|0,779|
| Brilhante35° |0,334|0,905|0,759|
| Rasante75° |0,329|0,825|0,820|
| Transição fina10–11 |0,386|0,959|0,813|

A reutilização reduz bastante o custo anterior; o benefício sobre geometria completa é menor. Os perfis conservadores ficam próximos da geometria completa: razões medianas de0,977 a1,012. Não há promessa de ganho em toda pose/perfil. Em particular, diferenças de poucos pontos percentuais não são conclusivas nesta execução: há dispersão elevada e picos por carga/frequência/scheduling, apresentados como p10–p90 no relatório. Mesmo controles sem redução exibem variação. As razões sugerem benefício, mas não isolam uma constante de custo por triângulo nem substituem profiling sustentado do gameplay.

Exemplo brilhante a distância64, perfil “Mais resumos”:65.536→28.090 triângulos. Medianas GPU: anterior0,28250ms; otimizado reutilizado0,04983ms; mesmo shader sem redução0,08746ms; geometria completa0,08350ms. Recalcular a árvore nesse mesmo frame custa0,18267ms no total (compute0,11792ms), portanto supera o custo da geometria completa. O p10–p90 do reutilizado vai de0,03446 a0,85650ms, explicitando a variabilidade; esse exemplo não deve virar previsão de FPS.

Seleção CPU por percurso/perfil: medianas entre617 e712µs, fora dos timestamps GPU; validação/envio também é reportada separadamente. O próximo gargalo é a seleção/upload, além de tornar a atualização da radiância barata quando as entradas mudam. Os534 casos de captura tiveram apenas4 atualizações necessárias porque cada percurso preserva direção de visão/luz/material; o benchmark frio continua recalculando em toda repetição medida com resumos ativos.

## Validação e entrega

- 594 testes Rust passaram (`cargo test --locked --release --all-targets`); fmt e clippy com `-D warnings` passaram. Permanece aviso de incompatibilidade futura da dependência `block0.1.6`.
- 11 ensaios de cache GPU,6 verificações GPU de extremos/cobertura e os3 cenários MCP headless (`render`, `replay_inspection`, `selection`) passaram.
- 534 pares com RGB linear e cobertura idênticos (diferença máxima0),534 PNGs idênticos ao modo gradual entregue anteriormente e178 referências idênticas. Nenhuma linha adicional pode ter sido introduzida nessas imagens; isso não remove o erro já existente contra a referência.
- 870 imagens de poses correspondentes na ida/volta idênticas;277 PNGs do experimento inicial preservados.1.780 capturas de256² e montagem de comparação atualizadas em caminhos estáveis.
- Harness JavaScript do relatório:40 verificações,140 carregamentos e192 recortes válidos. Montagem PNG inspecionada. Interação nativa do navegador e aceite visual do usuário permanecem separados.

Evidência corrente: `artifacts/appearance-probe/cost/{report.json,results.csv,validation.json,ui-validation.json,comparison.png,index.html}` e logs no mesmo diretório. Artefatos são gerados/ignorados; os comandos acima os reproduzem. EAF-68 permanece em acompanhamento para aceite visual. Runtime/MCP/cena da engine não foram alterados; integração futura deve manter paridade MCP/UI.

## Continuação

O [experimento de vistas e respostas preparadas no build](precomputed-views-probe.md) mede uma tabela persistida com câmera e luz móveis, incluindo memória, erro de reconstrução e custo de frame serial. Seus resultados pertencem a outra representação/cena; não substituem as medições deste estudo.
