# Previsão de custo calibrada — continuação EAF-88

Protocolo antes de implementar: mesmos assets e erro projetado0,15px; mesmos gatesRMSE0,005/p99RGB0,02/cobertura0,0005. Não alterar geometria/material nem consultar qualidade para escolher. Reutilizar bake híbrido existente e controles, sem contrato de runtime novo.

Calibrar em distâncias18,40,80,160; validar sem refit em14,28,56,112,224, nas duas resoluções e quatro densidades. Famílias/fontes são compartilhadas: holdout de poses, não teste de assets desconhecidos. Modelo de diferença global−regional com seis termos: intercepto, diferença de triângulos, diferença de draws, proxy de perímetro projetado sqrt(triângulos×área), área projetada e economia da normal nativa quando global é full. Coeficientes físicos de triângulos/draws/perímetro/normal não negativos; intercepto e diferença de custo por área livres. Ajuste por mínimos quadrados regularizados apenas na calibração (ridge0,0001 após normalização), ponderado pelo inverso do custo mediano dos candidatos, piso0,05ms. Isso reduz o domínio dos casos caros sobre os erros relativos dos casos rápidos; hiperparâmetros fixados antes do holdout.

Revisão estática: features derivadas de metadados/8cantos, sem scan de geometria, I/O ou alocação por seleção; scratch64 limitado. Monotonicidade permite podar usando mínimos regionais e custo parcial, com teste contra exaustivo. Modelo carregado uma vez, validado finito/monotônico. Custo da seleção entra na vazão; nenhuma tabela de pose/asset no modelo. Não prometer tempo mínimo real: residual do modelo, overhead e variação permanecem mensuráveis.

Medição: quatro fontes residentes, controles duplicados,8rodadas e128frames/bloco; alternar ordem dos candidatos a cada rodada. Conservar amostras em ordem e medianas, não só vencedor. GPU em32passes consecutivos com timestamps por passe, readback fora do intervalo; custo instrumentado separado da vazão sem timestamps. Compilação/captura fora da medição. Controles full/regional/global preservado/híbrido anterior/calibrado. Publicar perdas e resultados de cada densidade, não só melhor caso; não inferir FPS ou garantia estatística.

Validação: identidade do candidato, qualidade da curva/retorno/trocas, controles anteriores intactos, métricas próximas/distantes e custo/memória. Sem alterar Scene/Command/RenderFrame/MCP ou demo. Manter um build corrente e artefatos de pesquisa reproduzíveis. Próxima decisão depende da validação independente, sem ajustar coeficientes olhando vencedores do holdout.


## Implementação e calibração

`cost_model.rs` define seis númerosf64 (48bytes) e a diferença prevista global−regional. Valores≤0 escolhem global. Os termos de contagem, draws e perímetro têm coeficientes não negativos; logo aumentar custo regional só diminui essa diferença. O mínimo regional preparado e o número de draws já selecionados são limites inferiores, permitindo retornar global sem concluir todas as visitas. Testes com modelos de sinais diferentes e capturas com modelo ajustado comparam poda versus exaustivo. A qualidade permanece decidida exclusivamente pela projeção, antes do custo.

Área é a caixa projetada dos oito cantos, limitada à viewport física; crossing de near-plane usa viewport inteira. É uma estimativa de ocupação, não contagem de pixels vencedores/oclusão. Perímetro é proxy isotrópico sqrt(triângulos×área), não medição exata da forma de cada triângulo. Isso mantém o frame limitado a metadados, mas limita a precisão do modelo.

Ajuste por descida coordenada com restrições, normalização das colunas e regularização.64observações em16coortes, somente18/40/80/160. Peso pelo inverso do custo mediano dos candidatos, piso0,05ms; alvo é mediana de16diferenças pareadas global−regional. O modelo não recebe família, ID, distância, resolução nominal nem índice da pose; recebe características projetadas e contagens. A mesma geometria/material está nos dois conjuntos, portanto não alegar generalização para outro asset/GPU.

Coeficientes obtidos antes da validação: intercepto0,002612843ms; delta de milhões de triângulos0,353439991; delta de draws0; delta de perímetro0,172973570; área em megapixels0,009692831; economia de normal nativa0,012097749. Draws com peso ajustado zero não provam drawcall grátis: o ajuste tem correlações entre features e só estes casos. Treino inclui até16draws, intervalo de área0,00484…1,975MP; a câmera14 testa extrapolação de área próxima além do treino. RMSE de treinamento0,007937ms;16/64sinais previstos diferem do sinal medido, incluindo diferenças pequenas. fit.json registra hash dos dados, configuração e coeficientes; nenhum refit após resultados do holdout.

A primeira tentativa de timestamps em vários passes num único encoder/resolve retornou fim anterior ao início e foi descartada antes do ajuste. A coleta final enfileira32commandbuffers sem espera entre passes, aguarda a conclusão do lote e resolve os64timestamps em submissão separada. Soma das durações/32 medeGPU, com instrumentação; vazão usa128frames sem timestamps. Isso reduz a influência de um passe isolado, sem garantir ausência de variação. CPU informada é tempo de preparar/submeter o lote dividido por128, incluindo eventual pressão da fila; não é o antigo tempoCPU isolado de um frame.

## Reprodução

Requer bake de EAF-88 (`--bake-hybrid-scaling`). Não duplica buffers de assets no disco.

```sh
cargo run --locked --release --example appearance_probe -- --calibrate-cost-scaling
python3 scripts/fit_cost_model.py
cargo run --locked --release --example appearance_probe -- --capture-cost-scaling
cargo run --locked --release --example appearance_probe -- --validate-cost-scaling
cargo run --locked --release --example appearance_probe -- --inspect-cost-scaling
cargo run --locked --release --example appearance_probe -- --repeat-cost-scaling
python3 scripts/check_cost_scaling.py
python3 scripts/report_cost_scaling.py
node scripts/verify_triangle_scaling.mjs artifacts/appearance-probe/cost-scaling
```

Preservar model.json após a calibração até completar o holdout. Cada coorte registra o modelo utilizado; verificador compara com o modelo das capturas, hash do treinamento e separação das distâncias. Dados/report/PNGs correntes ficam em artifacts/appearance-probe/cost-scaling. O relatório anterior é preservado; cada controle novo é recapturado e comparado byte a byte com seu arquivo anterior.


## Resultado e decisão

**Não aprovado como substituto geral do híbrido anterior.** Houve correções locais, mas o modelo não reduziu o erro geral de escolha e introduziu regressões reproduzíveis. Permanece caminho experimental explícito; o relatório abre o híbrido anterior como referência. O objetivo de eliminar regressões continua aberto. Resultados originais não foram substituídos pelos da repetição, e não houve refit com o holdout.

Apple M4 Pro/Metal,20coortes/80pontos cronometrados. Menor candidato medido escolhido em57/80pontos pelo modelo e59/80pelo anterior; corrigiu13escolhas e introduziu15erros. Essa contagem inclui diferenças pequenas sujeitas a ruído. Penalidade média somente da escolha (custo do candidato selecionado/min dos candidatos, sem overhead do seletor) ficou0,887% versus0,880% do anterior;9pontos acima de3% em ambos. Não demonstrou ganho geral de previsão.

Incluindo a execução do seletor, mediana do excesso sobre o menor candidato medido:−0,14% no novo e+1,25% no anterior. Piores excessos15,64% e12,35%, respectivamente. Valores negativos não significam que o modelo sabe vencer ambos os candidatos: são medições separadas de caminhos com os mesmos comandos, sob variabilidade. Razão novo/anterior na vazão: mediana0,9945, mínimo0,8320, máximo1,1501. Esses extremos não são ganho/perda garantidos.

Custo do caminho calibrado CPU+GPU sobrepostas, relevo, ms/frame:

| Resolução | Distância | 1× | 2× | 10× | 100× | 100/1 | Maior/menor | 100/10 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| 3456×2234 | 14 | 0.2358 | 0.2566 | 0.4486 | 0.8932 | 3.788× | 3.788× | 1.991× |
| 3456×2234 | 28 | 0.1999 | 0.2099 | 0.4008 | 0.3902 | 1.951× | 2.005× | 0.973× |
| 3456×2234 | 56 | 0.1912 | 0.2088 | 0.2218 | 0.2802 | 1.466× | 1.466× | 1.263× |
| 3456×2234 | 112 | 0.1674 | 0.1758 | 0.1846 | 0.1992 | 1.190× | 1.190× | 1.079× |
| 3456×2234 | 224 | 0.1657 | 0.1671 | 0.1754 | 0.1668 | 1.007× | 1.059× | 0.951× |
| 1728×1117 | 14 | 0.0942 | 0.1108 | 0.3066 | 0.2843 | 3.017× | 3.253× | 0.927× |
| 1728×1117 | 28 | 0.0774 | 0.0912 | 0.1053 | 0.1626 | 2.100× | 2.100× | 1.544× |
| 1728×1117 | 56 | 0.0646 | 0.0663 | 0.0696 | 0.0850 | 1.315× | 1.315× | 1.220× |
| 1728×1117 | 112 | 0.0592 | 0.0683 | 0.0613 | 0.0630 | 1.064× | 1.154× | 1.029× |
| 1728×1117 | 224 | 0.0602 | 0.0548 | 0.0516 | 0.0615 | 1.021× | 1.190× | 1.190× |

Em3456×2234/perto14,100×full2,1304ms, híbrido anterior0,8879ms, calibrado0,8932ms: mantém a redução geométrica já existente, sem melhora nova demonstrada. Na distância224, calibrado1×→100× varia0,1657→0,1668ms, mas o máximo intermediário é0,1754ms. O platô distante continua; perto a curva cresce3,79×, sem resolver independência de densidade.

## Separação entre erro de escolha e instabilidade

Plano1×/3456×2234/distância14: regional0,2176ms versus global0,2298ms. O modelo passou a escolher regional. Na repetição, regional0,2260ms versus global0,2370ms: a correção manteve a ordem observada. Em contraste, plano10× na mesma câmera: regional0,2247ms versus global0,2099ms, mas modelo escolhe regional. Novo0,2269ms (+8,1% sobre global); repetição regional0,2431ms/global0,2142ms/novo0,2402ms (+12,1%). **Erro de previsão reproduzível.** É uma câmera com área maior que as usadas no treino; correlação não prova que extrapolação seja a única causa. Não atribuir causalmente a formato dos triângulos/cache sem ablação própria.

Pior excesso inicial: relevo100×/1728×1117/distância224. Modelo escolheu corretamente global; global0,05314ms, novo0,06145ms (+15,64%), apesar de imagem/comandos de desenho iguais. Repetição com as quatro densidades e todos os controles: global0,06117ms, novo0,05981ms (−2,21%). Esse pico não se reproduziu; não tratá-lo como erro de previsão nem apagar a primeira observação.

DiagnósticoCPU separado:80poses,8rodadas×1000chamadas, black_box, ordem alternada, somente seleção e metadados quentes. Acréscimo novo−híbrido anterior: mediana0,086µs, máximo0,936µs. No pior ponto de vazão, global0,0211µs, anterior0,0515µs, novo0,0581µs; a diferença para global é0,037µs, muito menor que os8,31µs do pico de vazão. Não comprova causa única do restante nem equivale a desempenho de um frame com cache frio. No plano10× próximo, o novo seletor custa0,705µs, anterior0,050µs; há overhead real, além de escolher a geometria mais lenta.

A coleta sustentada reduziu os extremos observados de dispersãoGPU frente à rodada anterior, mas continua variável: controles duplicadosGPU0,533…2,169 (mediana1,0003); vazão0,784…1,273 (mediana0,9993). Pior excessoGPU bruto:58,5% novo e71,3% anterior nesta rodada. Nenhum desses extremos estabelece limite confiável de regressãoGPU. Não comparar números isolados de protocolos diferentes como ganho de renderer; o protocolo novo é aplicado simultaneamente a todos os controles.

Duas coortes críticas foram repetidas após identificar problemas, sem mexer no modelo: plano/3456×2234/perto e relevo/1728×1117/longe. recheck.json guarda os40slots por coorte, sem substituir confirmation.json. Essa repetição é diagnóstico dirigido pelos resultados, não novo conjunto de calibração nem teste cego adicional de todo o espaço.

## Qualidade e validação

Qualidade164/164poses aprovada,144da curva+20trocas. Zero divergência de cobertura; RMSEmax0,00358144 e p99max0,01664484, com gates anteriores intactos. Cada imagem calibrada é exatamente igual ao candidato escolhido emRGBA32F; não é igualdade numérica à fonte full. Poda/exaustivo coincidem nas164poses.68escolhas globais e76regionais na curva;56encerram sem regiões,76visitam64,12encerram parcialmente. Mudaram50decisões das144poses em relação ao anterior.

760PNGs verificados em dimensões/CRC/descompressão; retorno idêntico e576controles byte a byte iguais ao experimento anterior. Modelo congelado confere entre todas as coortes e capturas; distâncias de treino/holdout disjuntas, hash de calibração conferido.630testes Rust release/all-targets, Clippy/fmt e regressões render/replay_inspection/selection headless aprovados.576casos JavaScript novos e576legados comDPR/recortes/playback/carga obsoleta; harnessDOM não equivale a browser nativo. Recortes do plano10×, relevo100× próximos e uma troca inspecionados sem fresta nova; imagem inspection.png. Aceite humano/vídeo contínuo separados.

Sem novo payload de mesh/normal: reusa266,96MiB do comparador100×, todos os controles residentes,303,40MiB por família com quatro fontes. Modelo6f64=48bytes; JSON de dados/fit e capturas são artefatos, não memória por frame. Não inclui overhead de targets/readback/driver. Build corrente externo preservado, sem nova cópia de app ou integração na engine funcional/MCP.

Limite identificado: seis features e este conjunto de calibração não distinguem suficientemente os custos reais. Próxima hipótese exige ablação que isole forma/distribuição dos triângulos e acesso à normal, com dados mais estáveis e novas poses reservadas; não basta reponderar este modelo olhando os vencedores já publicados. A calibração é infraestrutura de pesquisa utilizável; sua hipótese de melhora geral foi rejeitada pelos resultados atuais.

Continuidade executada: [ablação de material e profundidade](scaling-ablation-study.md). Ordem de triângulos não corrigiu a inversão; simplificação de shader muda a relação global/regional. Reutilizar profundidade e normais preparadas trouxe ganho localizado9,5–15% no ponto100× próximo, com perdas em outros pontos; nenhum novo preditor adotado. O estudo registra também a hipótese de passeRGBA32F rejeitada, controles e qualidade. Não confundir avanço localizado com eliminação geral de regressões.
