# Densidade física com conteúdo fixo — EAF-88

## Pergunta e protocolo

Correção solicitada do experimento anterior: aumentar apenas a subdivisão, mantendo a mesma superfície detalhada e o mesmo material. O relevo anterior acrescentava frequências conforme aumentava a fonte; isso misturava complexidade visual com redundância geométrica. Este experimento fixa a superfície piecewise-linear `f1_d2` do bake híbrido: 327.680 triângulos parentais, quatro bandas de relevo, mesmo campo de normais e material. Não reavalia a função de relevo nos novos vértices.

O protocolo registrado antes da coleta mantém câmera próxima na distância 14, resoluções físicas 3456×2234 e 1728×1117 e metas de 1, 2, 10 e 100 triângulos fonte por pixel **coberto pelo objeto**. O denominador vem da captura parental na mesma câmera; não é pixel CSS nem total da tela. É uma média de triângulos submetidos/pixels cobertos, não uma contagem de vencedores por amostra. A caixa expandida pelo erro numérico cabe inteira no frustum medido a 14 (máximos NDC 0,787 horizontal/0,939 vertical); não foi calculada oclusão por triângulo.

Subdivisão baricêntrica uniforme `n` nas bordas de todos os triângulos parentais; refinamentos adicionais por centroide só no interior. Cada divisão interna acrescenta dois triângulos, atingindo o alvo com diferença máxima de um triângulo. O smoke inicial usava divisões diferentes nas bordas; embora passasse nos gates, foi corrigido **antes da coleta completa** para evitar T-junctions. Testes cobrem área, winding, bordas compartilhadas bit-idênticas e contagens.

Três controles usam o mesmo depth + material por pixel:

- Referência: superfície parental completa e detalhada.
- Subdividida completa: novos buffers indexados reais, sem geração procedural no shader ou extrapolação de tempo.
- Seleção certificada: usa a parental se o erro numérico projetado cabe em 0,15 pixel; conserva a subdividida caso contrário. Nenhum LOD aproximado adicional.

O certificado vem da construção, **não da descoberta de equivalência em uma malha arbitrária**. Acumula arredondamento dos pontos da grade e dos centroides, com margem explícita para coordenadas/transforms f32. O limite conservador de projeção para deslocamento de norma `e` é `focal * e * (1 + raio_xy / z_min) / (z_min - e)`, usando oito cantos da caixa; entrada inválida ou próxima do plano near rejeita a parental. Essa margem não é uma prova de equivalência raster em qualquer GPU: cobertura e cor são verificadas separadamente. Foi forçado certificado inválido em uma fonte de cada resolução; o fallback produziu os mesmos pixels e triângulos do caminho denso.

## Resultado medido

Apple M4 Pro / Metal. Display consultado: 3456×2234 pixels físicos, 1728×1117 pontos, 120 Hz. Mesma luz e materiais em todas as densidades. Na pose 14, referência cobre 2.263.907 pixels nativos ou 565.977 na resolução menor. Todas as quatro fontes de uma resolução ficam residentes durante a respectiva coleta.

Tempos abaixo são a média de quatro medianas: dois controles × duas rodadas, oito blocos por controle. “Total” é tempo amortizado por frame de CPU/GPU sobrepostas, com espera ao final do bloco. Câmera, decisão, codificação e submissão estão incluídas. Não é latência individual nem FPS de um jogo. Timestamp GPU é uma coleta separada.

| Triângulos/pixel a 14 | Triângulos fonte nativos | Completa total ms | Certificada total ms | Completa GPU ms | Certificada GPU ms |
|---|---:|---:|---:|---:|---:|
| 1 | 2.263.908 | 1,2488 | 0,4259 | 1,1682 | 0,4037 |
| 2 | 4.527.814 | 2,0056 | 0,4247 | 1,8771 | 0,3989 |
| 10 | 22.639.070 | 6,5441 | 0,4290 | 5,8252 | 0,4053 |
| 100 | 226.390.700 | 46,1012 | 0,4259 | 45,4957 | 0,3997 |

A seleção envia **327.680 triângulos** em todos os casos. Sua curva nativa varia 1,025% entre mínimo e máximo; razão 100/1 = 1,00014. A referência custa 0,4252–0,4305 ms, compatível com esse piso. O caminho denso cresce 36,92× entre extremos. No caso 100, a redução de tempo medida é 99,08% contra processar toda a subdivisão redundante. Esse ganho não é extrapolável a detalhes geométricos novos.

| Triângulos/pixel a 14 | Fonte em 1728×1117 | Completa total ms | Certificada total ms |
|---|---:|---:|---:|
| 1 | 565.978 | 0,4328 | 0,2311 |
| 2 | 1.131.954 | 0,6162 | 0,2216 |
| 10 | 5.659.770 | 1,8310 | 0,2227 |
| 100 | 56.597.700 | 12,3261 | 0,2211 |

A queda de 4,32% na curva certificada menor não prova que mais triângulos aceleram o render: o trabalho selecionado é igual. Razões das medianas duplicadas de todos os métodos: 0,903–1,040 na primeira rodada e 0,928–1,069 na repetição. As rodadas pertencem ao mesmo processo e repetem a sequência de ordens alternadas/rotacionadas; não são execuções independentes nem intervalos de confiança. Não prometer regressão zero por diferenças dentro desse ruído.

Blocos: referência/seleção 128 frames; completa nativa 14/7/2/2 frames e menor 56/28/5/2, definidos por orçamento de trabalho antes da coleta. Timestamp usa até 32 frames de cada bloco. Ambas as passagens do render estão incluídas; CPU e GPU não são parcelas aditivas. Captura, leitura de timestamps, preparo e uploads iniciais ficam fora. Timings somente na pose 14; poses 12 e 16 têm validação visual, não medição própria de performance.

## Qualidade

40 poses: duas resoluções × quatro densidades × cinco posições `12→14→16→14→12`. Fonte e subdivisão ficam fixas durante o trajeto; a razão triângulos/pixel muda com a cobertura. Uma mesma referência por pose para todas as fontes.

- 80 comparações passaram nos gates anteriores: RMSE foreground ≤0,005 linear; p99 RGB ≤0,02; cobertura divergente ≤0,0005.
- Seleção certificada é **exatamente igual à referência RGBA32F** nas 40 capturas: erro, cobertura e pixels alterados iguais a zero.
- Subdividida completa: RMSE máximo 0,00060947; p99 máximo 0,00001854; cobertura divergente máxima 0,000009747 (0,0009747%). Máximo RGB isolado 0,326505: existem diferenças de borda/rasterização mesmo com mesma superfície matemática; não chamar esse comparador de pixel-idêntico.
- Maior limite projetado da parental: 0,00476473 pixel, abaixo do limite 0,15 mantido.
- 120 PNGs verificados por CRC, dimensões e descompressão. Referências idênticas entre densidades e retorno byte-idêntico nos três métodos. Hashes dos inputs/shaders preservados.
- Recortes centrais 512² nativos da referência e da subdividida 100 inspecionados; sem diferença visível identificada nesse recorte. O relatório permite escolher recortes e respeita DPR, com ampliações 1/2/4. Visões gerais são reduzidas e rotuladas.

Cinco poses discretas não comprovam ausência de shimmer em movimento contínuo. Não há métrica temporal reprojetada, browser nativo validado ou aceite humano. Igualdade exata se refere à referência parental fixa; não à integração analítica de subpixels nem a uma imagem supersampled nova.

## Preparo e memória

| Meta nativa | Posições + índices | Draws fonte | Preparo + upload |
|---|---:|---:|---:|
| 1 | 53,86 MiB | 1 | 32,52 ms |
| 2 | 98,35 MiB | 1 | 51,23 ms |
| 10 | 437,38 MiB | 3 | 246,96 ms |
| 100 | 4.036,25 MiB (3,94 GiB) | 29 | 2.445,20 ms |

As quatro fontes somam 4,517 GiB na coorte nativa e 1,191 GiB na menor, além dos recursos comuns parentais/normais/targets. Não é medição de RSS ou memória total do processo. O scratch de posições/índices atinge capacidade de 224 MiB no maior caso, além de leitura da fonte e scratch de um triângulo parental. Buffers individuais limitados a 128 MiB e blocos a oito milhões de triângulos. Posições f32×3 e índices u32; nenhuma quantização. As fontes densas não são gravadas em disco e são liberadas antes da resolução seguinte e ao término.

O preparo é O(T), executado antes do frame nesta rotina offline; não é uma integração com o pipeline de build da engine. O comparador mantém os dados densos para poder medi-los: este incremento **não entrega streaming ou redução da memória residente**. Um pipeline de produção poderia conservar apenas os níveis necessários, mas isso ainda requer projeto/validação.

## Conclusão e próximo experimento

O teste corrigido confirma um platô **para subdivisão redundante conhecida**, conservando o conteúdo visual. Corrige a confusão anterior entre acrescentar triângulos e acrescentar detalhe. A decisão usa erro projetado, não tabela de tempos por densidade, e evita examinar a fonte densa no frame.

O próximo problema é construir esse certificado a partir do asset: partir da malha detalhada sem usar sua genealogia conhecida, produzir regiões/pais com erro limitado de forma e aparência e comparar contra a fonte. Isso deve preservar detalhes realmente novos, contornos, mudanças de material e visibilidade; não basta reduzir contagem. O caso atual é um heightfield opaco, luz fixa e material específico, não um importador de assets arbitrários, árvore animada, GI, sombras, transparência ou integração com os contratos da engine.

## Reprodução e validação

Após o bake existente `--bake-hybrid-scaling`, a partir da worktree `codex/graphics-research`:

```sh
python3 scripts/run_pixel_density.py
cargo test --locked --release --all-targets
cargo clippy --locked --release --all-targets -- -D warnings
cargo fmt --all --check
python3 scripts/run_verifications.py --only render replay_inspection selection
```

O runner congela hashes, executa `--pixel-density`, valida capturas e gera o relatório. `--pixel-density-smoke` limita a menor resolução/meta 1 para diagnóstico. Usar GPU sem tarefas próprias concorrentes na coleta. Para revisar dados existentes sem recalcular GPU: `check_pixel_density.py`, `report_pixel_density.py`, `verify_pixel_density.mjs` em `scripts/`.

Resultados: 636 testes Rust (três novos), Clippy com warnings negados, fmt/diff-check e 13 grupos headless render/replay_inspection/selection passaram. 80 casos JS com DOM simulado cobrem resoluções, densidades, caminhos, poses, DPR, recortes, reprodução e carregamento fora de ordem. A dependência `block 0.1.6` mantém aviso pré-existente de incompatibilidade futura.

Evidência corrente em `artifacts/appearance-probe/pixel-density/{index.html,preparation.json,quality.json,validation.json,repeat.json,summary.json,verification.json,ui-verification.json,input-manifest.json}`; logs no diretório pai. Fonte/documentação/scripts versionados, outputs gerados ignorados. Build corrente reaproveitado em `/Volumes/Zero/build-cache/ignis-graphics/release`; nenhum binário histórico novo. Runtime/NPC/Scene/Command/RenderFrame/MCP e demo não alterados.

Checklist prévio: somente laboratório; sem contratos, dependências ou threads novos; GPU preparada antes dos frames; decisão por certificado/bounds sem varrer triângulos; alocações/uploads somente no preparo; testes de cobertura/contagem/winding; tolerância mantida; memória e preparo explícitos. Revisão final em `docs/performance.md`.
