# Seleção automática do sombreamento — EAF-88

Protocolo definido antes da nova coleta: escolher somente entre híbrido anterior5 e profundidade85; geometria/limiar0,15px/material permanecem iguais. Modo95 estima benefício anterior−profundidade usando seis termos: constante, megapixels da tela, milhões de triângulos selecionados, sqrt(triângulos×área projetada), área projetada e área com normais nativas. Sem identidade/família/densidade/distância como chave, tabela de vencedores ou leitura de qualidade no frame.

Calibrar em16/24/48/96/192, duas famílias e duas resoluções. Ridge normalizado0,001, seis coeficientes; margem conservadora p90 da superestimação em validação deixando uma coorte inteira fora +0,005ms. Exigir benefício acima dessa margem e5% do tempo anterior previsto. Fora da faixa de pixels calibrada, ou carga/área acima de125% do máximo calibrado, usar anterior. Essa margem não é garantia estatística. Congelar modelo/hash antes de medir teste14/21/28/56/84/112/224 e poses novas19/32/128; não reajustar pelos vencedores de teste. Repetir casos críticos em arquivo separado.

Medição inclui seleção, encode, passes e submit/wait; quatro densidades simultâneas,8rodadas,128frames/bloco, duplicatas e GPU separado32passes. Capturas fora do timing com gates RMSE0,005/p99RGB0,02/cobertura0,0005; ida/volta, comparação exata ao caminho escolhido e trocas adicionais localizadas pela decisão (sem consultar qualidade). Qualidade preservada não significa pixels float iguais à fonte. Relatar perdas, indecisões e ruído, mesmo se a meta de eliminar regressões não for atingida.

Checklist prévio: seleção uma vez/frame e reaproveitada; projeção de8cantos, dois produtos escalares de6termos, sem alocação/scan de malha/readback/lock/I/O. Modelo carregado/validado uma vez, ausente implica anterior. Mesmos buffers; pipelines preparados na inicialização. Sem estado persistente de decisão, retorno determinístico. Build/bake correntes, sem nova dependência ou mudança de contratos runtime/MCP. Custo de calibração e dependência deGPU/shader explícitos; regra só para este laboratório heightfield opaco.

## Correção do domínio antes da comparação completa

Após congelar o primeiro modelo e iniciar a primeira coorte, inspeção apenas dos metadados projetados detectou área3,544MP no plano a14, maior que125% do máximo2,681MP calibrado. A restrição rejeitaria a proximidade por extrapolação; não é evidência de custo. Interrompida a coleta parcial sem usar seus tempos para ajuste. Dados/modelo/auditoria iniciais preservados como initial-domain-*.json. Acrescentar distância12 (quatro coortes) para cobrir o domínio próximo, mantendo features/ridge/margem/fator125% e poses de teste inalterados. Novo modelo é congelado antes de reiniciar comparação completa. Isso é uma correção de cobertura do experimento, não ajuste pelos vencedores de teste.

## Reprodução

```sh
cargo run --locked --release --example appearance_probe -- --calibrate-automatic-shading
cargo run --locked --release --example appearance_probe -- --extend-domain-automatic-shading
python3 scripts/fit_automatic_shading.py
cargo run --locked --release --example appearance_probe -- --validate-automatic-shading
cargo run --locked --release --example appearance_probe -- --repeat-automatic-shading
cargo run --locked --release --example appearance_probe -- --capture-automatic-shading
python3 scripts/check_automatic_shading.py
python3 scripts/report_automatic_shading.py
node scripts/verify_triangle_scaling.mjs artifacts/appearance-probe/automatic-shading
```

Reutiliza bake em hybrid-scaling, quatro densidades32.768/65.536/327.680/3.276.800triângulos fonte.100× é relativo ao primeiro asset, não100× o número de pixels. Manter GPU sem outra tarefa própria concorrente durante timings. Captura/verificação/testes executados depois. Fit lê somente calibration.json; auditoria contém hash do conjunto e modelo, e verificação exige hashes idênticos após teste. Recalibrar após alteração do shader, material, GPU, driver ou protocolo; adapter é validado na carga, mas não é um identificador completo de driver/shader. Este modelo local não é cache persistente de produção.

O modo automático usa o mesmo código de seleção geométrica do modo5, chama o preditor após escolher os draws e então usa exatamente o pipeline0 ou8. O relatório registra a decisão e confere imagemfloat idêntica ao caminho explícito escolhido. Shader não é interpolado/misturado na transição; diferenças pequenas entre os dois caminhos são medidas pelos gates. Sem histerese persistente; mesma câmera produz a mesma escolha. Instabilidade em limites de custo sob movimento precisa ser avaliada, não presumida resolvida por esse determinismo.

Custos e memória: modelo contém21valoresf64 e identificação textual do adaptador, uma vez naCPU. Não cria textura por pixel; usa depth32 existente. O construtor do experimento reutiliza criação de pipelines diagnósticos e descarta os que não usa antes de criar targets; custo de inicialização mantido fora do timing e não recomendado como inicialização final de produto. Geometria/normais/níveis e color/depth/readback continuam residentes. Não houve medição deRSS/VRAM.

## Medição congelada e resultado

Apple M4 Pro/Metal. Modelo final treinado em96pontos/24coortes, teste160pontos/40coortes, repetição32pontos/8coortes. Nenhum tempo de teste entrou no fit. Margem final0.02249ms, além da exigência de5% do anterior previsto. CalibraçãoSHA256 `3adc344907a5076f143ce91461785e8e74bed6ed09a0a8f1d06819c7d78897bb`; modeloSHA256 `878afbe8b8d7ac2342c2e2050b588a7f26f78b3da5d2e3396a6202a45c344bf3`.

Tempo total CPU/GPU sobrepostas, relevo nativo3456×2234, ms/frame:

| Rodada | Distância | Caminho | 1× | 2× | 10× | 100× |
|---|---:|---|---:|---:|---:|---:|
| Teste | 14 | Fonte | 0.24184 | 0.26259 | 0.46146 | 2.21191 |
| Teste | 14 | Anterior | 0.24039 | 0.26641 | 0.45868 | 0.92808 |
| Teste | 14 | Automático | 0.24663 | 0.27373 | 0.43065 | 0.77309 |
| Teste | 224 | Fonte | 0.18205 | 0.19082 | 0.24243 | 0.89333 |
| Teste | 224 | Anterior | 0.17641 | 0.18339 | 0.18536 | 0.18986 |
| Teste | 224 | Automático | 0.18455 | 0.18319 | 0.17935 | 0.17769 |
| Repetição | 14 | Fonte | 0.24931 | 0.27744 | 0.46191 | 2.29951 |
| Repetição | 14 | Anterior | 0.24912 | 0.28674 | 0.46031 | 0.94538 |
| Repetição | 14 | Automático | 0.24381 | 0.27881 | 0.44310 | 0.80515 |
| Repetição | 224 | Fonte | 0.16983 | 0.17780 | 0.22973 | 0.78860 |
| Repetição | 224 | Anterior | 0.15336 | 0.16605 | 0.16630 | 0.16654 |
| Repetição | 224 | Automático | 0.16116 | 0.16750 | 0.16763 | 0.16395 |

Automático escolheu profundidade12/160pontos; nenhuma dessas escolhas teve o caminho explícito85 mais de3% acima do5 na própria coorte. Isso não prova a escolha ótima: em100×/metade da resolução/distâncias19 e21, a vantagem explícita foi só1,0%/1,8%, enquanto o automático mediu9,6%/7,2% mais lento que anterior. Dois casos com benefício incerto continuam abertos; não ajustar coeficientes a esses vencedores.

Automático contra anterior em todos160pontos:51mais de3% rápidos,36mais de3% lentos,73dentro da faixa; mediana−0,83%, extremos−26,37%/+20,02%. Na repetição32pontos:18mais rápidos,4mais lentos,10na faixa. As quatro perdas repetidas ocorreram no fallback que usa o mesmo pipeline anterior; não atribuir diferenças de dezenas de microssegundos ao cálculo isolado da decisão. Custo isolado do seletor mediano0,0155µs/max0,0254µs no teste,0,0168µs/max0,0538µs na repetição. Essa micro medição separa a decisão da seleção geométrica já existente; os tempos totais incluem ambas. Não somar mecanicamente CPU e GPU de fases separadas.

Duplicatas de tempo total:0,757…1,319 na validação;0,642…1,553 na repetição. Há ruído substancial, inclusive entre controles com trabalho idêntico. A faixa3% é descritiva, não intervalo de confiança nem margem que garanta ausência de regressão. Menor tempo entre controles explícitos é referência posterior de análise, jamais entrada da decisão. Relatório conserva amostras e perdas, sem substituir curva automática por mínimo artificial.

Ganho principal repetido: relevo100×/perto/nativo−16,70% no teste e−14,83% na repetição; GPU−17,50%/−15,55%. Em10×/perto/nativo,total−6,11%/−3,74%, masGPU+4,26%/−6,86% — evidência menos estável, não alegar ganhoGPU constante ali. Metade da resolução10×/perto:total−26,37%/−26,25%.100×/metade:−13,02%/−4,28%, magnitude variável.1×/2× próximos e todas as densidades distantes mantêm o anterior. Nenhum passe extra é executado nesses fallbacks.

A fonte completa100×/perto/nativo custa2,21191ms no teste e2,29951ms na repetição; o ganho do automático sobre ela inclui a seleção geométrica prévia. Não atribuir tudo ao novo seletor/passe. O platô próximo não está resolvido: no teste, automático0,24663→0,77309ms de1× a100×. Longe, continua aproximadamente estável, com o piso do caminho anterior. Só este laboratório de heightfield/material/GPU foi validado, não uma cenaAAA geral.

## Qualidade e artefato

304poses de ida/volta (duas famílias×quatro densidades×duas resoluções×19frames) e12poses adicionais nas quatro trocas encontradas pelo seletor entre14 e224. A busca de trocas usa passo0,5 e captura antes/depois do intervalo detectado; não é validação contínua de todas as posições. Todas316poses automáticas passam: RMSEmax0,00358144/p99max0,01664484/cobertura0. As três variantes de cada pose principal e o automático nas trocas totalizam924comparações de qualidade aprovadas. Mesmo conjunto de triângulos entre modos5/85/95; a escolha não relaxa erro projetado.

1216PNGs validados em CRC/dimensão/descompressão,624controles byte-idênticos ao estudo anterior. Retorno de câmera byte-idêntico; modo95 igual emRGBA32F ao modo escolhido explicitamente, inclusive nas trocas. Recortes físicos512² do caso relevo100×/perto inspecionados em inspection-full.png/inspection-auto.png: sem linha/fresta nova observada no recorte. Não equivale a inspeção humana de todo percurso ou prova de ausência de qualquer artefato.

Relatório atual: artifacts/appearance-probe/automatic-shading/index.html. Controles de câmera/densidade/resolução, fonte completa versus anterior/profundidade/automático, curvas e temposGPU/CPU/total, decisões, perdas e links aos dados.912casos do JavaScript real passaram no harnessDOM, incluindoDPR/recorte físico/playback/carga obsoleta. Capturas são offline; não é render ao vivo nem aceite de browser nativo.

Validação final do incremento:632testes Rust com `cargo test --locked --release --all-targets`, Clippy release/all-targets `-D warnings`, fmt e diff-check aprovados. Render/replay_inspection/selection headless:13grupos aprovados; sem janela ou controle da demo. Build corrente reutilizado em `/Volumes/Zero/build-cache/ignis-graphics/release`, sem nova cópia de bake/binário histórico. Runtime funcional preservado. Pesquisa permanece aberta quanto a regressãozero, casos intermediários incertos, platô próximo, generalização e aceite humano.
