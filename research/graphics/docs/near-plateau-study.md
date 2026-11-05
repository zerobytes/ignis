# Platô próximo — diagnóstico e intervenção

Incremento EAF-88: focar o crescimento próximo1×→100×, preservando0,15px e gates RMSE0,005/p99RGB0,02/cobertura0,0005. Não alterar modelo automático pelo resultado do teste. Primeiro medir, no mesmo command buffer, timestamps de início/fim de geometria e início/fim do material do modo85.16frames instrumentados (64queries existentes),8rodadas, quatro densidades, duas resoluções, plano/relevo, distância14. Reportar também intervalo entre passes; não equiparar soma de estágiosGPU a tempoCPU/GPU sobrepostas ou contadores físicos de invocações.

Checklist prévio: instrumentação só no laboratório, nenhum contrato/runtime/MCP; queryset/readback existentes, sem recurso porframe novo. Flag temporária somente no profiler, restaurada antes do benchmark normal. Seleção e shader intactos no diagnóstico. Coleta/captura/compilação separadas; mesmo build e bake correntes. Depois do diagnóstico, documentar intervenção específica e seu custoCPU/memória antes de medir qualidade/performance; preservar controles e resultados negativos.

## Timestamps sobrepostos e ablação

A hipótese de ordenação estrita dos quatrotimestamps falhou. Backend Metal local do wgpu27 usa início de vértices/fim de fragmentos; os intervalos dos passes se sobrepõem. Evidência: o intervalo intermediário foi negativo, enquanto cada duração e o intervalo global foram positivos. Preservar stamps brutos e diferença assinada; não somar as medianas como estágios exclusivos. A referência de [timestamp writes do wgpu](https://wgpu.rs/doc/wgpu/api/render_pass/struct.RenderPassTimestampWrites.html) define as amostras, não uma decomposição aditiva de custos físicos.

Nova ablação dirigida:85executa ambos;185executa somente geometria/depth;285executa somente material sobre depth preparado pelo85 do MESMO asset/câmera imediatamente antes de cada bloco, fora do timing. Mesmas densidades/resoluções/pose14,8rodadas/128frames, duplicatas. Não usar185/285 como candidatos visuais completos nem somar seus tempos isolados como se preservassem overlap/cache/dependências da execução completa.

Intervenção a testar após o diagnóstico: preparar buffers exclusivos de posiçõesf32 para depth, reindexados por intervaloLOD, e uma segunda variante com ordenação espacial dos mesmos triângulos. Mantém coordenadas/winding/triângulos/níveis/draws/seleção0,15px/normais/material. Remap e ordenação no preparo, apenas bind/troca de pipeline no frame. Medir posição compacta separadamente de compacta+ordem espacial, memória adicional dos controles e igualdadeRGBA32F. Não afirmar que reorganizar trabalho elimina sua dependência do número de triângulos.

## Comparação implementada

Modos0/95/85/105/115/135: fonte completa, automático anterior, profundidade anterior, posições compactas, compactas+ordem espacial, automático com o mesmo modelo que substitui somente85 por115. Não reajustar o modelo com estes tempos. Hash do modelo anterior `878afbe8b8d7ac2342c2e2050b588a7f26f78b3da5d2e3396a6202a45c344bf3`. Poses10/14/28/224, ambas famílias/resoluções/quatro densidades; repetir14/224.128frames por bloco sem timestamps,32frames instrumentados separados,8rodadas e duplicatas. Qualidade: ida/volta10→14→28→224→28→14→10, seis imagens por pose; comparar diretamente compactos/espacial contra85 e automático novo contra95, além da referência completa e gates anteriores.

Compactação por intervalo preserva offsets/counts e duplica vértices compartilhados entre níveis; somente posição é carregada pelo vertex shader do depth. Ordenação Morton de centroidesXZ é específica deste laboratório heightfield e acontece no preparo, dentro de cada intervalo; preserva winding. Não é culling, novo resumo ou mudança do orçamento visual. O comparador carrega ambos os controles extras;100× adiciona231,56MiB por variante. Isso NÃO é economia de memória do experimento. Fonte e todos os níveis continuam residentes; investigar streaming/compartilhamento antes de integração. Nenhum artefato ou contrato do runtime foi alterado.

A ablação isolada apresentou timestampsGPU muito discrepantes da vazão e sobreposição entre estágios. Mantidos os dados brutos, mas não usados para estimar porcentagens físicas de custo. A vazão sem timestamps da ablação é evidência de intervenção: remover um passe muda também dependências, cache e sobreposição; não subtrair ou somar para atribuir exclusivamente o restante ao outro passe. Confirmar intervenções pelos tempos completos.

## Unidade de densidade e diagnóstico de projeção

Inspeção das capturas destacou uma limitação central:100× quer dizer3.276.800triângulos fonte, não100triângulos/pixel. Normalizar pela cobertura física real. Adicionado diagnósticoCPU offline de área projetada de cada triângulo fonte, usando a mesma matriz/câmera/resolução. Bins <0,25/0,25..1/1..4/≥4pixel²; separar triângulos inteiramente dentro do frustum, fora de um semiespaço comum e cruzando clipping. Estes últimos não entram na média para evitar área sem clipping tratada como área na tela. Sem teste de oclusão; área subpixel não prova erro visual pequeno nem substitui o certificado0,15px. Contagens/transformações apenas nesta ferramenta, nunca por frame.

O plano mantém a mesma superfície com tesselação redundante. O relevo acrescenta frequências ao aumentar a densidade. Portanto o crescimento próximo mistura representação mais densa e conteúdo diferente; não é um teste de centenas de triângulos redundantes por pixel na mesma superfície próxima. Relatar explicitamente essa limitação, além da organização de memória testada.

## Resultados de tempo e interpretação

Apple M4 Pro/Metal;64pontos/16coortes de teste,32pontos/8coortes de repetição. Tempos totais CPU/GPU sobrepostas emms/frame; todos os passes incluídos.

| Rodada | Distância | Caminho | 1× | 2× | 10× | 100× |
|---|---:|---|---:|---:|---:|---:|
| Teste | 10 | Fonte | 0.31867 | 0.35030 | 0.52697 | 2.58939 |
| Teste | 10 | Auto anterior | 0.31481 | 0.34992 | 0.53610 | 0.92192 |
| Teste | 10 | Auto espacial | 0.31890 | 0.35035 | 0.50020 | 0.87176 |
| Teste | 14 | Fonte | 0.26799 | 0.29506 | 0.50314 | 2.41889 |
| Teste | 14 | Auto anterior | 0.26942 | 0.29723 | 0.46960 | 0.85787 |
| Teste | 14 | Auto espacial | 0.26813 | 0.29659 | 0.45036 | 0.79844 |
| Teste | 224 | Fonte | 0.20623 | 0.21535 | 0.27343 | 1.00298 |
| Teste | 224 | Auto anterior | 0.19145 | 0.20487 | 0.19556 | 0.19183 |
| Teste | 224 | Auto espacial | 0.19654 | 0.20623 | 0.19592 | 0.21022 |
| Repetição | 14 | Fonte | 0.25874 | 0.28653 | 0.50184 | 2.40345 |
| Repetição | 14 | Auto anterior | 0.25973 | 0.29087 | 0.47033 | 0.84839 |
| Repetição | 14 | Auto espacial | 0.26115 | 0.28532 | 0.44231 | 0.79834 |
| Repetição | 224 | Fonte | 0.19689 | 0.20697 | 0.26545 | 0.91939 |
| Repetição | 224 | Auto anterior | 0.19637 | 0.19646 | 0.18730 | 0.19233 |
| Repetição | 224 | Auto espacial | 0.18846 | 0.19299 | 0.19575 | 0.19468 |

No relevo100×/perto14/nativo, ganho total6,93% no teste e5,90% na repetição. A curva1×→100× ainda cresce2,98×/3,06× com o candidato, contra3,18×/3,27× com o automático anterior. Na câmera ainda mais perto10, ganho5,44% no100× (sem repetição própria dessa pose). O resultado é redução de constante, não platô resolvido. Comparador continua experimental, não novo padrão do runtime ou substituição do relatório anterior.

Diagnóstico de carga: perto14 mantém32.768/65.536/327.680/880.384triângulos. O100× usa64regiões no nívelstride2, fundidas em1draw; o crescimento não vem de multiplicar64draws nesse ponto. Organização espacial muda acesso/ordem/reuso, mas não essa contagem. O buffercompacto sozinho deve ser comparado ao85 explícito para separar seu efeito do sort; nenhuma atribuição a cache/banda física foi demonstrada por contadores dehardware.

Na ablação isolada próxima nativa, geometria/depth sem material cresce0,0824→0,3708ms de1× a100×; material sobredepth pronto cresce0,2346→0,3343ms. As execuções completas desse diagnóstico crescem0,3253→0,9408ms. Não somar/subtrair esses números como parcelas físicas: o isolamento muda dependências/overlap/cache e o timingGPU isolado foi instável. Usar tempos completos para validar ganho.

Limitação central: fonte100× tem3.276.800triângulos, somente0,424triângulo por pixel da tela3456×2234. Pela cobertura do objeto, distância10/14/28/224 corresponde a0,750/1,447/6,851/461,781triângulos fonte por pixel coberto. Isso é média, sem contar quais triângulos vencem cada amostra. A câmera próxima ainda não testa100triângulos/pixel. Além disso, o relevo acrescenta bandas de frequência entre densidades; o plano redundante já apresenta curva aproximadamente plana. Separar densidade de conteúdo antes de inferir independência de complexidade.

Próximo experimento fundamentado: uma superfície detalhada fixa e o mesmo campo de normais/material em todas as densidades, subdivisão que preserve sua forma, densidade expressa em triângulos por pixel físico e memória limitada por preparo/streaming. Medir quanto o certificado conserva por conservadorismo antes de alterar0,15px. Manter também o caso com conteúdo novo como teste distinto de fidelidade. Não assumir que área<1pixel² garante invisibilidade, nem que detalhes visíveis possam ser descartados para produzir uma curva plana.

Reprodução após o bakehybrid e o modelo automático anteriores:

```sh
cargo run --locked --release --example appearance_probe -- --profile-near-plateau
cargo run --locked --release --example appearance_probe -- --isolate-near-plateau
cargo run --locked --release --example appearance_probe -- --bake-near-plateau
cargo run --locked --release --example appearance_probe -- --bench-near-plateau
cargo run --locked --release --example appearance_probe -- --capture-near-plateau
cargo run --locked --release --example appearance_probe -- --repeat-near-plateau
cargo run --locked --release --example appearance_probe -- --footprint-near-plateau
python3 scripts/check_near_plateau.py
python3 scripts/report_near_plateau.py
node scripts/verify_triangle_scaling.mjs artifacts/appearance-probe/near-plateau
```

GPU sem outra tarefa própria concorrente durante cada coleta; capturas/regressões/preparo separados. Reutiliza o build corrente externo e o bakefonte; derivados ativos do experimento ficam no diretório estávelnear-plateau. Não há cópias históricas de executável.

## Validação final e custos mantidos

112poses/560comparações nos gates,336igualdadesRGBA32F entre variantes reorganizadas e seus controles.672PNGs íntegros,240controles anteriores byte-idênticos, retorno idêntico. RMSEmax0.00358144, p99max0.01667795; cobertura divergente máxima0.0000002288, equivalente a1pixel na nova pose10. A igualdade entre candidato e anterior é exata; igualdade com a fonte completa não é prometida. Modelo congelado/hash preservado.

validation: 14/64pontos mais de3% rápidos, 6mais de3% lentos, mediana-0.52%; faixa-9.37%…+13.58%. Duplicatas0.877…1.160. A faixa3% é descritiva, sem significância estabelecida; perdas e fallbacks do mesmo pipeline permanecem no relatório.

repeat: 8/32pontos mais de3% rápidos, 5mais de3% lentos, mediana-1.10%; faixa-11.23%…+9.81%. Duplicatas0.873…1.141. A faixa3% é descritiva, sem significância estabelecida; perdas e fallbacks do mesmo pipeline permanecem no relatório.

Relevo100×/perto14: o nível regional seguinte (stride4) tem certificado entre0,1575e0,2460px nas64regiões, acima do limite0,15px. Esse certificado impede usar o nível mais barato; não prova que todo pixel teria erro visível nem que relaxar o limite seria seguro. O globalstride2 também é rejeitado por0,2377px. Esta é uma oportunidade concreta para certificados locais mais precisos, mantendo a mesma tolerância e a comparação de fidelidade.

Projeção da fonte100×/nativa: distância10 tem51,88% dos triângulos inteiramente dentro do frustum com área<1pixel²; distância14 tem76,21%; distância28/224 têm100%. Média de área1,423/0,691/0,146/0,00217pixel², respectivamente. Não há teste de oclusão nesse diagnóstico e área não é extensão máxima de aresta nem certificado de erro. Contabilidade de64coortes passou. A pose10 separa207.911triângulos fora de um semiespaço comum e2.567cruzando clipping.

633testes Rust release/all-targets, Clippy-Dwarnings, fmt/diff-check e regressões headless render/replay_inspection/selection aprovados.560casos JS de escolha/reprodução/recorteDPR passaram. Recortes512² fonte/candidato inspecionados sem artefato novo observado; nenhuma comprovação de browser nativo, vídeo contínuo ou aceite humano. Relatório offline `artifacts/appearance-probe/near-plateau/index.html` e dados raw no mesmo diretório; documento é a evidência versionada.

Preparo100×:compacto0,27–0,35s, espacial2,16–2,42s nas duas famílias; inclui escrita dos derivados. Cada controle tem78,32MiB de posições+153,24MiB de índices. O comparador adiciona463,12MiB aos266,96MiB fonte/níveis/normais porasset100×; sem mediçãoRSS/VRAM, compressão ou streaming. Melhora modesta pelo custo de memória: manter experimental, sem promover a formato da engine.

Incremento concluído como diagnóstico/intervenção controlada. Meta de platô próximo, generalização, eliminação de regressões e aceite humano continuam abertas em EAF-88. Contratos funcionais e demo preservados; sem push.
