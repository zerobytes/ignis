# Ablação do custo — EAF-88

Protocolo prévio: reutilizar bake/fonte, projeção0,15px e gates RMSE0,005/p99RGB0,02/cobertura0,0005. Investigar dois casos próximos com ordenação global/regional distinta antes de ajustar qualquer preditor. Entradas diagnósticas flat, normal e material nativo isolam trabalho em relação ao shader original; diferenças são intervenções no pipeline, não decomposição aditiva exata de tempoGPU. Cor simplificada nunca é candidata à qualidade final.

Candidato explícito: substituir bifurcação entre triângulos da textura de normais por seleção de coordenadas/pesos, preservando três leituras e a mesma interpolação triangular, sem bilinear/mip/perda de precisão. Não presumir ganho de compilação/GPU antes de medir. Seleção de geometria e modelo permanecem intactos.

Diagnóstico: quatro densidades simultâneas, plano/relevo, perto14 e longe224, resolução física3456×2234. Validação do candidato nas duas resoluções, cinco distâncias existentes e poses novas21/84. Protocolo128frames/bloco,8rodadas, controle duplicado,32passesGPU, shader/captura/I/O fora da medição. Preservar resultados negativos e repetir casos críticos após coleta. Capturar curvas de ida/volta e novas poses com material completo; exigir gates antigos e comparar candidato diretamente ao shader anterior.

Checklist prévio: inicialização compila variantes uma vez somente no experimento; frame sem scans/alocações novas, mesmos buffers/tiposf32/targets. Sem upload/readback adicional dentro do timing, sem lock novo. Memória extra de pipelines explicitada, sem duplicar bake ou contratos de runtime. Reversibilidade por código de variante; renderer normal conserva caminho anterior. Custos de geometria/draws/normal e variaçãoGPU permanecem até medição, sem promessa de platô próximo ou FPS.

## Intervenções após o diagnóstico inicial

Primeira coleta: plano próximo1×/10× inverte a vantagem no shader completo, mas a diferença praticamente desaparece com cor simples. Relevo100× próximo: regional0,9086ms completo,0,4168ms flat,0,8657ms material com normais nativas diagnósticas. A troca de bifurcação isolada0,8922ms não resolve o crescimento. Flat muda o trabalho/interpolantes otimizados pelo compilador: não interpretar subtrações como contabilidade exata de estágios físicos.

Correção local: intervalos regionais no nível fonte conservam a triangulação original, portanto podem usar normal interpolada nativa sem consultar textura. Intervalo contíguo conferido uma vez no upload; apenas comparação de faixa por draw. Regiões simplificadas mantêm textura.

Experimento adicional fundamentado pelo custo do material: geometria escreve normal bruta e profundidade reversa assinada; sinal indica se a normal deve vir da textura. Segundo passe reconstrói posição e calcula material uma vez por pixel coberto, usando três leituras triangulares somente quando necessário. Mesmo seletor, precisãoRGBA32F, luz/material e amostra única. Reconstrução de posição é uma nova fonte de erro a medir; sem promessa de igualdade float. Sem sombra/transparência/MSAA. Inclui ambos os passes no timing; extra de16bytes por pixel físico,123.531.264bytes em3456×2234 e30.882.816bytes em1728×1117, além de pipeline/uniform64bytes. Testar perdas distantes/cenas leves; não substituir caminho antigo incondicionalmente.

Ordem: inverter submissão dos triângulos dentro de cada nível sem alterar vértices/winding/triângulos/faixas/draws. Buffer de índices adicional somente na ablação de ordem/captura; validação de tempos final usa os buffers originais. Isso testa ordem/localidade, não isola todos os efeitos da forma do triângulo.

## Reprodução e códigos

```sh
cargo run --locked --release --example appearance_probe -- --ablate-scaling
cargo run --locked --release --example appearance_probe -- --capture-ablation
cargo run --locked --release --example appearance_probe -- --order-ablation
cargo run --locked --release --example appearance_probe -- --validate-ablation
cargo run --locked --release --example appearance_probe -- --repeat-ablation
python3 scripts/analyze_scaling_ablation.py
python3 scripts/check_scaling_ablation.py
python3 scripts/report_scaling_ablation.py
node scripts/verify_triangle_scaling.mjs artifacts/appearance-probe/scaling-ablation
```

Executar medições sem outra captura/build/regressão concorrente. Reusa bake híbrido corrente, sem regeneração de assets. Resultado e PNGs em artifacts/appearance-probe/scaling-ablation, logs estáveis no diretório pai. Os dados brutos conservam oito amostras de cada controle, não apenas medianas. Repetição dirigida: plano/perto, relevo/perto e relevo/longe na resolução nativa, após rodada completa, sem sobrescrever resultados iniciais.

Códigos: unidade0=fonte,3=regional,4=global preservado,5=híbrido anterior. Dezena0=shader original,1=flat diagnóstico,2=normal diagnóstico,3=material com normal nativa diagnóstico,4=interpolação triangular sem bifurcação,5=ordem invertida,6=sem bifurcação+normal nativa em regiões completas,7=material após visibilidade. Somente4/6/7 são candidatos visuais;1/2/3 não podem substituir a referência. Nenhum preditor novo é ajustado neste experimento.

## Qualidade e ablação de ordem

208poses: duas famílias, quatro densidades, duas resoluções,13posições de ida/volta (14/21/28/56/84/112/224). Sete candidatos por pose,1456comparações, todos passam os gates originais e todos têm zero divergência de cobertura. As posições21/84 não participaram do diagnóstico inicial.

- Sem bifurcação: imagensRGBA32F exatamente iguais ao shader anterior em208/208poses para cada caminho global/regional/híbrido.
- Normal nativa em regiões completas: regional168/208exatas, maior diferença3,04e−6; global/híbrido208/208exatas. Neste percurso o híbrido não ganha uma mudança de imagem com esse atalho: é importante não atribuir a ele ganhos que já vêm da variante sem bifurcação.
- Material por pixel: máximo absoluto de mudança frente ao híbrido anterior0,00056028linear; RMSEmax contra fonte0,00357672, p99max0,01667795. Pequenas diferenças por reconstrução de posição; não é identidadefloat. O máximoRMSE do híbrido anterior é0,00358144. Essa redução minúscula não prova melhora perceptual.
- Ordem invertida:416/416comparaçõesRGBA32F exatas. Razão de tempo invertido/original nos32pontos: mediana1,0026, mínimo0,9707, máximo1,0658. Não sustenta otimização consistente. No plano próximo, regional continua melhor em1× e global em10× nas duas ordens. Isso não exclui toda otimização de localidade nem isola a forma geométrica.

Revisão final proporcional: inicialização prepara pipelines, intervalo fonte e buffers; nenhuma alocação/scan de malha novo por seleção. Desenho usa comparação de faixa e troca limitada de pipeline; shader sem bifurcação conserva três leituras. Segundo passe acrescenta uniform64bytes, drawfullscreen e um alvoRGBA32F por viewport; resize substitui recursos, lifetime por renderer. Scratch/seleção/projeção/bake permanecem iguais. Buffers de índices invertidos só existem no diagnóstico/captura. Sem dependência, lock, runtime, Scene/Command/RenderFrame ou MCP novo. Custo fixo do segundo passe e memória são tradeoffs medidos, não escondidos pela curva100/1.

## Correção após medir o buffer intermediário

Primeira confirmação nativa/relevo/perto mostrou que o passe comRGBA32F ainda perde:100×1,0997ms versus híbrido0,9085ms;1×0,5232ms versus0,2313ms. Não adotar esse caminho como melhoria. A separação do material precisa pagar seu custo extra de memória/passe.

Neste heightfield, as normais já estão no bake e são acessíveis por worldXZ. Logo testar uma variante de profundidade apenas, seguida de reconstrução da posição e leitura triangular da normal fonte por pixel. Reutiliza o depth32 existente como textura, sem alvo extra de normal/cor, sem quantização nova nem alteração dos limites. Usa textureLoad também nos níveis fonte que antes usavam normal nativa; isso exige nova comparação de qualidade. Mais um bindgroup/pipeline/uniform64bytes; nenhuma textura adicional por pixel. A técnica depende da parametrização deste laboratório e não equivale a um visibility buffer genérico para múltiplos assets, UVs, materiais e animação.

Código85 usa o mesmo híbrido geométrico. Nova rodada pareada inclui full0, híbrido5, branchless45, dois passesRGBA32F75 e profundidade85; mesmas112combinações de família/densidade/resolução/distância, mais repetição das três coortes críticas. Dados anteriores preservados, sem escolher vencedor a posteriori no renderer. Novas capturas208poses com comparação aos controles PNG anteriores, qualidadefloat, cobertura e retorno. Comandos adicionais: --capture-depth-ablation, --depth-ablation, --repeat-depth-ablation; arquivos depth-quality.json/depth.json/depth-recheck.json.

A validação de profundidade inclui também as20poses adjacentes às trocas do híbrido anterior, reaproveitadas de hybrid-scaling/transitions.json (família relevo100×). Não altera as distâncias após observar erros. Comparação float/cobertura e imagens em depth-transitions.json; teste distinto dos112pontos de timing.

Memória: a variante85 não requer textura intermediária além do depth32 já existente e da textura de normais preparada. O comparador ainda mantém o alvoRGBA32F residente para medir o controle75 lado a lado; isso não equivale a uma medição de redução da memória do processo. O número117,81MiB é payload calculado do alvo dispensável, não leitura de RSS/VRAM nem memória por asset. Pipeline/driver, fonte/bake, color/depth e readback continuam separados.

O teste com profundidade altera também os interpolantes/trabalho de geometria e o acesso à normal nos níveis completos. Portanto um ganho dele não identifica isoladamente largura de banda, cache ou número de invocações de shader. Os contadores de hardware não foram coletados; a conclusão causal fica restrita às intervenções de pipeline medidas.

## Resultado da rodada final

Apple M4 Pro/Metal;28coortes,112pontos, cinco modos com controles duplicados,8rodadas×128frames e GPU em32passes instrumentados. Inclui ambos os passes da variante85, uniform, seleção e submissão no total. Não é FPS de jogo.

Custo CPU+GPU sobrepostas, relevo, ms/frame:

| Resolução | Distância | Caminho | 1× | 2× | 10× | 100× |
|---|---:|---|---:|---:|---:|---:|
| 3456×2234 | 14 | Fonte completa | 0.2431 | 0.2657 | 0.4495 | 2.1384 |
| 3456×2234 | 14 | Híbrido anterior | 0.2468 | 0.2673 | 0.4495 | 0.8823 |
| 3456×2234 | 14 | Profundidade+material | 0.2652 | 0.2801 | 0.4162 | 0.7498 |
| 3456×2234 | 224 | Fonte completa | 0.1854 | 0.1899 | 0.2390 | 0.8210 |
| 3456×2234 | 224 | Híbrido anterior | 0.1760 | 0.1817 | 0.1716 | 0.1813 |
| 3456×2234 | 224 | Profundidade+material | 0.2267 | 0.2170 | 0.2285 | 0.2286 |
| 1728×1117 | 14 | Fonte completa | 0.0885 | 0.1141 | 0.2845 | 1.4706 |
| 1728×1117 | 14 | Híbrido anterior | 0.0924 | 0.1127 | 0.2873 | 0.2836 |
| 1728×1117 | 14 | Profundidade+material | 0.0934 | 0.1133 | 0.2196 | 0.2719 |
| 1728×1117 | 224 | Fonte completa | 0.0660 | 0.0745 | 0.1264 | 0.6993 |
| 1728×1117 | 224 | Híbrido anterior | 0.0570 | 0.0545 | 0.0540 | 0.0589 |
| 1728×1117 | 224 | Profundidade+material | 0.0756 | 0.0811 | 0.0823 | 0.0848 |

Candidato85 contra híbrido anterior: 9/112pontos com custo mais de3% menor, 97/112 com custo mais de3% maior; mediana +22.58%, melhor -23.56%, pior +52.41%. Faixa3% é descritiva, não significância estatística nem garantia. Não aprovado como substituto geral; conservar escolha explícita e baseline. Sem tabela de vencedores ou ajuste novo de preditor.

Relevo nativo próximo100×:0,8823→0,7498ms (−15,0%) contra anterior; fonte completa2,1384ms. A maior parte da redução frente à fonte já vinha da geometria adaptativa anterior; não atribuir todo o ganho ao passe novo. Em1×:0,2468→0,2652ms (+7,5%). Crescimento100/1 diminui3,57×→2,83× em parte por aumentar o custo mínimo; ainda não há platô próximo. Longe224: anterior0,1760…0,1813ms, candidato0,2170…0,2286ms: platô mais caro. Em1728×1117/perto10×:0,2873→0,2196ms (−23,6%), mas100× melhora só4,1%; sem monotonicidade universal da vantagem.

Hipótese do bufferRGBA32F rejeitada: primeira rodada112/112pontos mais caros que o híbrido (mediana+76,18%); repetição12/12mais caros. Em100× próximo nativo,1,0997ms e repetição1,1441ms, versus anterior0,9085/0,8984ms. Remover esse buffer e reconstruir a normal reduz o custo, mas não elimina o custo fixo de um passe adicional. Os tempos de protocolos/rodadas diferentes não são comparados como ganho; todas as porcentagens usam controles da própria coorte.

## Repetição e conclusão

Repetição direcionada, dados separados em depth-recheck.json: relevo100× perto nativo, anterior0,88077ms versus profundidade0,79708ms (−9,50%), frente aos−15,02% da rodada inicial. Os16pares de amostras por rodada/controle foram mais rápidos no candidato; razão mediana0,8636, intervalo0,8023…0,9585. Não equivale a16observações independentes nem intervalo estatístico de garantia. GPU:0,87210→0,74302ms na rodada inicial e0,87969→0,74991ms na repetição, aproximadamente−15% em ambas. CPU preparar/submeter aumenta0,02867→0,04466ms e0,02907→0,04575ms; inclui pressão de fila, não somar mecanicamente ao tempoGPU coletado em outra etapa.

Relevo10× perto na repetição:0,44238→0,41796ms (−5,52%),13/16pares mais rápidos. Relevo1× perto continua pior:0,24565→0,26502ms (+7,9%). Longe100× repete a perda:0,17439→0,22801ms (+30,7%). Plano próximo também continua mais caro. O candidato tem utilidade localizada, mas não elimina regressões nem resolve o platô próximo. Ganhos de9,5–15% no ponto pesado são evidência deste laboratório, não promessa para outrosassets/GPU/jogos.

Duplicatas de vazão na rodada final variaram0,706…1,387; na repetição0,921…1,094. Manter resultados originais, repetições e amostras disponíveis. O resultado robusto é a direção repetida de ganho/perda dos casos críticos; pequenas diferenças do restante não têm significância estabelecida. Preditor anterior não foi substituído por ajuste posterior nem tabela de vencedores. A próxima decisão de seleção ainda precisa considerar o custo fixo do passe e validar em novas cenas/poses, antes de integração.

## Validação final

Profundidade:228/228poses aprovadas (208curva/novas posições/retorno+20adjacentes às trocas). RMSEmax0,00357674/p99max0,01667795, zero divergência de cobertura, maior mudança frente ao shader híbrido anterior0,00056028linear. Nenhuma quantização de norma foi introduzida. As normais nativas versus recuperação por textura/reconstrução podem arredondar de forma diferente; preservar gate não significa igualdadefloat ao renderer anterior.

Verificador:1684comparações de candidatos aprovadas,416comparações de ordem comRGBA32F exatamente iguais;1288PNGs íntegros em CRC/dimensões/descompressão, retorno byte-idêntico,288controles byte-idênticos ao estudo anterior. A captura de profundidade também conferiu416controles PNG contra esta primeira rodada antes de remover seu arquivo temporário. Recortes de perto, distância56 e troca30,5 inspecionados em inspection.png: sem fresta ou padrão vertical novo observado. Isso não substitui aceite visual humano.

631testes Rust release/all-targets, Clippy -D warnings e fmt aprovados. Render/replay_inspection/selection headless aprovados; sem manipular demo.832casos do JavaScript real do novo relatório e576do relatório anterior passaram no harnessDOM, incluindoDPR/recorte/playback/carga obsoleta. Não é browser nativo nem render ao vivo. Dados e relatório regeneráveis pelos scripts, sem enginefuncional/contratos/MCP novos. Build corrente externo mantido; sem novo bake, cópia histórica de binário ou publicação.
