# Visibilidade conservadora na GPU — experimento

Recorte autorizado: geometria completa, sem resumos de aparência, câmera passando por obstáculos e objetos/oclusor em movimento. Exigir pixels e IDs idênticos à referência agrupada e medir custo total. Isolado da engine funcional e dos contratos Scene/MCP.

## Revisão antes de implementar

1. Frequência: bounds e hierarquia objeto→regiões preparados antes do percurso; transforms e profundidade dos oclusores do frame corrente atualizados por frame.
2. Alocação: buffers GPU e comandos indiretos persistentes, planos CPU reutilizados; sem readback no caminho de decisão. Captura/contadores só para instrumentação externa ao timing.
3. Dados: layout POD, IDs mantidos ao compactar instâncias. Regiões inteiras e caixa conservadora; nenhuma remoção por tamanho subpixel ou material.
4. Repetição: grupos de instâncias por asset; oclusão da raiz evita testar filhos. Comparar objeto inteiro versus regiões para medir o custo da granularidade.
5. Limites:96 instâncias de três assets; objetos e regiões opacos. Oclusores explícitos baratos (terreno/caixas), nenhum teste temporal com máscara antiga. Não prometer todas as oclusões possíveis.
6. GPU/I/O: profundidade invertida; pirâmide armazena o mínimo (profundidade mais distante/zero onde vazio). Bounds ampliados e margem depth preservam casos incertos. Comandos indiretos consumidos sem CPU.
7. Sincronização: timestamps incluem todos os passes; serial e blocos com sobreposição separados. Não esconder custo de construir pirâmide ou preparar draws. Indirect first instance solicitado só no experimento e validado no adapter.
8. Invalidação: reconstruir profundidade/pirâmide para toda câmera/posição corrente. Testar novo objeto visível, obstáculo móvel, near plane e bordas; nunca esperar frame futuro para reparar imagem.
9. Correção: comparar RGB float/IDs e PNGs; referência de geometria agrupada, controle indireto/frustum, oclusão por objeto, oclusão por região; testes de min reduction/projeção/compactação e regressão headless.
10. Custos mantidos: GPU ainda paga a prova de visibilidade; sombras/reflexos/GI/transparência/deformação fora deste recorte. Features do laboratório não mudam contratos ou requisitos da engine principal.

Alternativas: Hi-Z temporal exige recuperação segura de desoclusões; prepass de toda a geometria paga grande parte do custo que se quer remover. Neste primeiro recorte, os oclusores explícitos são desenhados com sua geometria real e retidos no framebuffer, formando evidência corrente; os assets detalhados podem ser ocluídos, mas ainda não servem como oclusores para outros assets.

## Implementação e prova de visibilidade

`--bake-gpu-visibility` prepara bounds de479 regiões e três raízes por asset, com assinatura dos vértices/índices. As96 instâncias compartilham geometria e bounds. Runtime atualiza transforms, testa raízes e, na variante regional, somente os filhos das raízes sobreviventes. Compute escreve instâncias compactadas e comandos indiretos; render bundles reutilizam três comandos por objeto/asset ou479 por região. Comandos vazios ainda existem: não há multi-draw-count nem custo zero de dispatch. Ordem de compactação determinística conserva IDs.

**Hi-Z corrente:** desenhar terreno e paredes reais, construir pirâmide mínima da profundidade invertida e depois desenhar a geometria sobrevivente. O primeiro nível reúne4×4 pixels físicos; os seguintes2×2. Padding e amostras descobertas valem zero e impedem a remoção. Projetar oito cantos, expandir retângulo em dois pixels e só descartar se a profundidade mais próxima do bound, acrescida de0,000002, estiver atrás do mínimo de todos os texels tocados. Bounds que cruzam o plano próximo permanecem. A mesma cor/depth dos obstáculos é mantida, mas há custo de encerrar e retomar o passe.

**Prova direta para caixas:** a primeira rodada mostrou que construir/consumir Hi-Z pode absorver o ganho. Foi adicionada uma especialização para as quatro caixas opacas deste cenário. Para uma câmera fora de uma caixa convexa, o volume encoberto por ela é convexo. Se todos os oito vértices de uma AABB estão estritamente nesse volume, a caixa inteira também está; qualquer triângulo contido nela está oculto. O shader intersecta os segmentos câmera→canto com uma mesma caixa oclusora. Nunca junta cantos ocultos por caixas diferentes, o que poderia fechar falsamente um vão.

Os bounds da fonte são ampliados segundo escala/magnitude da posição, as caixas oclusoras encolhem0,002 unidades e a interseção precisa ocorrer antes de0,99999do segmento. Paralelismo, câmera dentro do oclusor e plano próximo têm fallback conservador. Essas margens são proteção numérica para a escala deste laboratório, não uma prova universal de precisão IEEE754. As caixas têm espessura maior que o near plane e geometria opaca real, sem faces ausentes ou alpha. O seletor usa suas posições atuais e não a imagem anterior. Compute antecede um único passe de geometria, eliminando a construção da pirâmide e o segundo passe neste caminho.

Essa alternativa não é Hi-Z generalizado: só usa caixas sólidas alinhadas aos eixos. Terreno e assets detalhados não se tornam automaticamente oclusores analíticos. Aproximar uma árvore por sua caixa externa seria incorreto; uma futura preparação precisaria provar que qualquer oclusor simplificado fica dentro de material realmente opaco e tratar recorte pelo plano próximo. Não foi implementada essa generalização.

## Protocolo experimental

- Apple M4 Pro/Metal, wgpu27, render offscreen RGBA32F+Depth32F, amostra única, shading GGX compartilhado e inalterado.
- Duas resoluções físicas:3456×2234 e1728×1117. Não decidir qualidade por pixels CSS ou pelo preview reduzido.
- Três cenários: aberto; quatro paredes fixas; terceira parede móvel.96 assets com rochas, baús e árvores; parte dos assets também se move. A câmera atravessa/revela paredes e vãos. Paredes grandes favorecem oclusão forte: o resultado não é média representativa de um jogo.
- Cada percurso tem13 poses de ida e13de volta, total156 capturas de referência. Os cinco seletores GPU são comparados ao mesmo frame da geometria completa agrupada. Igualdade de todos os valores RGB float e IDs, sem limiar de RMSE. Cada variante salva mantém os PNGs iguais ao PNG completo.
- 16 poses extras cobrem câmera dentro de parede/asset, cruzamento do near plane, borda de passagem, remoção da parede sem mudar câmera e asset deslocado de trás para a frente. A CPU de referência desenha todos os96 objetos nesses casos; nenhum frustum antigo pode esconder uma falha.
- Dez poses cronometradas por cenário/resolução, cinco estados únicos e sua volta, total60. Incluem os extremos laterais da parede móvel. As poses intermediárias foram medidas depois da primeira rodada para evitar amostrar somente fases centrais do movimento; os controles são pareados dentro de cada medição.
- 20 ciclos de aquecimento;12 rodadas de controles duplicados com ordem rotacionada. GPU total abrange todos os passes do caminho. Pirâmide e seleção também têm intervalos próprios; captura/PNG/contadores/readback ficam fora.
- Ciclo serial inclui preparar cena, upload, codificar/submeter e esperar. Vazão inclui blocos de oito poses consecutivas, com submissões ordenadas e CPU/GPU sobrepostas;12 rodadas e controles duplicados. As poses continuam mudando em cada bloco. Não é FPS de gameplay, latência de input ou frame com apresentação na tela.
- Para agregar, calcular a média das duas medianas de cada modo na mesma pose; depois a razão modo/completa por pose; finalmente a mediana dessas razões. Não dividir medianas globais independentes. Variação dos controles aparece no relatório, incluindo outliers grandes no teste serial/timestamps.

O modo GPU ainda prepara a lista CPU usada pela referência e faz o mesmo upload comum. Isso torna a comparação equivalente, mas mantém trabalho redundante do comparador. Não há oracle ou readback guiando o desenho; o diagnóstico de quantos comandos/triângulos sobreviveram vem somente após capturas fora do timing.

## Reprodução

Na worktree gráfica, com meshes exportadas e cache regional existente conforme `representative-assets-probe.md` e `regional-scene-probe.md`:

```sh
cargo run --locked --release --example appearance_probe -- --bake-gpu-visibility
cargo run --locked --release --example appearance_probe -- --gpu-visibility
cargo run --locked --release --example appearance_probe -- --verify-gpu-visibility
python3 scripts/check_gpu_visibility.py
python3 scripts/report_gpu_visibility.py
node scripts/verify_gpu_visibility.mjs
```

`--benchmark-gpu-visibility` completa medições intermediárias ausentes em relatórios da primeira execução; a execução normal já mede as dez poses. Não muda capturas. `report.json` mantém amostras brutas; `summary.json`, `edge-checks.json`, `verification.json` e `index.html` estão em `artifacts/appearance-probe/gpu-visibility/`. Um campo diagnóstico do primeiro relatório de seis modos foi corrigido: a variante analítica por objetos envia três comandos, não479. Nenhuma imagem, contagem de triângulos ou amostra de tempo foi alterada nessa correção; reprodução já emite o valor correto.

O relatório oferece seleção de cena/resolução/algoritmo, ida/volta e recorte512² com compensação de DPR. É reprodução de capturas. Apresentação nativa não foi validada: o acesso automatizado a file:// foi bloqueado e não foi contornado. O harness executa o JavaScript, mas não substitui o navegador ou o aceite humano.

## Resultados medidos

Variante de menor custo de vazão neste recorte: **prova por caixas, seleção por objeto**. Percentuais abaixo são custo relativo à geometria completa agrupada (100% = empate), não ganho de FPS. Cada célula temporal é mediana de razões pareadas. Os valores absolutos do relatório não devem ser usados para recalcular essa razão por divisão de medianas globais.

| Resolução | Cena | Triângulos restantes | GPU total | Ciclo serial | Vazão em movimento | Vazão Hi-Z objetos | Vazão Hi-Z regiões |
|---|---|---:|---:|---:|---:|---:|---:|
| 3456×2234 | Aberta | 100.0% | 113.3% | 103.8% | 100.6% | 135.1% | 154.3% |
| 3456×2234 | Paredes fixas | 20.8% | 41.1% | 61.8% | 57.0% | 100.6% | 102.8% |
| 3456×2234 | Parede móvel | 17.7% | 41.9% | 86.6% | 58.9% | 105.1% | 104.0% |
| 1728×1117 | Aberta | 100.0% | 115.4% | 103.4% | 102.2% | 122.2% | 146.1% |
| 1728×1117 | Paredes fixas | 20.8% | 46.6% | 102.9% | 44.9% | 84.0% | 82.1% |
| 1728×1117 | Parede móvel | 17.7% | 37.8% | 101.1% | 51.1% | 86.3% | 96.9% |

Com paredes, a mediana de triângulos removidos é79–82%, enquanto a economia de custo por frame em sequência é41–43% na resolução maior e49–55% na menor. A variante regional remove ainda mais triângulos (85–86%), mas seus479 comandos e seleção adicional ficam mais caros que os três comandos por asset da variante de objetos. Mais triângulos removidos não é o objetivo final.

Na cena aberta, a prova por objetos não remove mais geometria que o frustum e custa13–15% mais no intervalo GPU; a vazão varia perto do empate (+0,6/+2,2%). Não há ganho demonstrado ali. Hi-Z custa22–35% mais na vazão aberta; com paredes, varia de regressão de~5% a economia de~16%, conforme resolução/cena/variante. A economia bruta de geometria não paga sempre a pirâmide e a retomada de passe.

O ciclo serial menor-resolução fica1–3% mais caro na variante vencedora, apesar do ganho em vazão. Isso registra o custo de submissão/espera e a diferença de sobreposição; não é correto anunciar melhoria equivalente de latência. Alguns controles duplicados de GPU/serial variaram fortemente (razões menores que0,5 ou maiores que1,7); a dispersão completa por cenário/método permanece publicada. A medição indica oportunidade local, não garantia estatística para outros dispositivos, cargas ou jogos. Não houve compilação/teste concorrente durante os benchmarks; o estado global do desktop não foi controlado.

## Memória e validação

Buffers estáticos GPU:7.107.552 bytes (6,78 MiB), incluindo meshes/índices compartilhados, instâncias, bounds, compactação e comandos. Não é custo de cache por asset. A saída regional reserva96 slots por comando apesar de só32 instâncias por asset — capacidade conservadora ainda excessiva, mantida explícita. A pirâmide R32Float custa5,33 MiB na resolução maior e1,33 MiB na menor. O comparador conserva essa alocação em todas as variantes, mesmo quando não a usa; não há economia de memória residente medida pela escolha analítica.

Cor RGBA32F+depth custa147,26 MiB ou36,82 MiB; readback de cor, contadores, timestamps, cópias CPU de diagnóstico e metadados/driver ficam à parte. Esses alvos de alta precisão servem à comparação e não equivalem ao formato/custo de um framebuffer de produção. Cache de aparência não é enviado à GPU.

- 156 poses principais +16extremas,860 comparações de RGB float/IDs sem divergência;780 PNGs completos, CRC/descompressão/dimensões válidos e bytes iguais às referências de cada pose.
- 617 testes Rust release/all-targets aprovados;34 testes do exemplo repetidos após acrescentar o comando de medições intermediárias. Clippy all-targets com `-D warnings` e fmt aprovados.
- Render, replay_inspection e selection headless:13 grupos aprovados. Nenhuma janela da demo funcional controlada.
- Harness do relatório:624 combinações de resolução/cena/algoritmo/pose; DPR1/1,5/2/3, zoom1/2/4, recorte limitado às bordas, ida/volta e descarte de carregamento obsoleto aprovados. Inspeção de PNGs reduzidos não é aceite visual físico; navegador nativo continua separado.

## Revisão final de performance

1. Frequência: `gpu_visibility_scene.rs` prepara regiões/raízes antes da sequência; câmera/transforms atuais atualizados por frame. Modo analítico dispensa reconstruir profundidade auxiliar; Hi-Z mantém geração corrente.
2. Alocação: buffers/bundles persistem; `Pose` reutiliza capacidade. Encoder/pass/submissão por frame e reconstrução dos targets no resize permanecem. PNG/JSON e readbacks somente na instrumentação.
3. Dados: Node48 B, instância32 B, comando20 B; IDs originais seguem compactação estável. Mesmos índices/material/precisão da referência. Nenhum resampling/subpixel/LOD nesta rodada.
4. Repetição: raiz oculta evita os filhos; comandos por faixa compartilham as32 instâncias de cada asset. Variante de objeto vence apesar de seleção menos precisa. CPU de referência ainda executada como custo comum do harness.
5. Escala:96 instâncias,479 regiões, quatro oclusores explícitos. Teste analítico limitado a quatro caixas; não há descoberta/seleção espacial de milhares de oclusores ou prova de escala AAA.
6. GPU/I/O: Hi-Z paga store/load e pirâmide; prova por caixas usa um passe. Indireção/compactação não exigem readback. Output reservado e479 comandos, inclusive vazios, continuam custos da variante regional.
7. Sincronização: intervalo total inclui todos os passes; CPU+GPU+espera e sobreposição em blocos separados. Não transferir ganho de vazão para latência/FPS. Captura/contadores não participam do caminho medido.
8. Invalidação: bounds locais têm assinatura da geometria; transforms/câmera/luz são atuais. Sem máscara temporal, sem reparo em frame futuro;16 poses extremas verificam mudança brusca e clipping. Generalizar caixas ou deformação exige novos bounds/provas.
9. Correção:860 comparações exatas, Rust/clippy/fmt e13 grupos MCP. Nenhuma alteração em Scene/Command/RenderFrame/replay/MCP ou no escopo funcional de NPCs. Interface do experimento é CLI e relatório; integração futura na engine requer dispatcher e UI equivalentes.
10. Custos mantidos: margens específicas da escala do teste, buffers folgados, targets de alta precisão, overhead de comandos/culling em cena aberta, variabilidade do desktop, geometria completa de superfícies parcialmente visíveis. Sem sombras/reflexos/GI/transparência/deformação e sem afirmação de superioridade sobre outras engines.

Próximo recorte proposto: gerar em build time oclusores internos conservadores de meshes sólidas reais, comparar com Hi-Z e escolher o caminho quando sua economia esperada pagar a prova. Essa escolha pode afetar desempenho, mas nunca deve autorizar remover geometria sem prova de invisibilidade. Ainda não implementado.
