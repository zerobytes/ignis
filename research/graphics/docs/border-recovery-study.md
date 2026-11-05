# Recuperação de bordas assimétricas — EAF-88

## Protocolo antes da implementação

Hipótese: subdivisão2× cria bordas a–b de um lado e a–v–b do outro. O grafo global recusa retirar v quando a–b já existe, mesmo sendo colinear. Investigar por contador e teste controlado antes de concluir causalidade.

Experimento opt-in: permitir unir caminhos colineares duplicados no grafo, propagando o máximo dos erros; preservar validação topológica por região, materiais, contenção, triangulação e restauração das bordas de regiões nativas. Comparar com o pacote selecionado do experimento recovery-stability. A seleção de pacote ocorre somente no build por redução estrita de faces sem aumentar bytes GPU; não inserir heurísticas por pose no frame.

Checklist prévio: (1) trabalho só no build; (2) reutilizar adjacências, sem nova coleção por vértice; (3) f64 para erro, f32/u32 no pacote existente; (4) não repetir descoberta no frame; (5) custo proporcional às adjacências da entrada, sem promessa de build constante; (6) shader, buffers e IGC1 inalterados; (7) coleta GPU isolada das regressões, com controles e duplicatas; (8) opção anterior preservada e codec validado; (9) testar fechamento, orientação, materiais e detalhe não afim em malhas controladas, além de1/2/10/100× na pedra externa; (10) manter fallback denso e explicitar falhas de rasterização anteriores.

Gates visuais inalterados: RMSE≤0,005, p99≤0,02, cobertura≤0,0005. Câmera perto/longe, duas resoluções físicas, órbita e luz variáveis, retorno. Medir GPU e tempo total; menos triângulos não constitui ganho temporal por si só. Malhas procedurais adicionais validam propriedades geométricas; não equivalem a ampliar o conjunto de assets externos.

## Implementação e reprodução

`compiler::compile_borders` recebe uma opção adicional; `compile` e `compile_traced` mantêm o comportamento anterior. Um vértice de grau2 só pode sair se a distância ao segmento, somada ao erro acumulado dos caminhos, couber emBORDER_TOL=1e-6. Se a aresta direta já existe, seu erro passa a ser o máximo dos dois caminhos; as adjacências duplicadas são reunidas. As validações de disco, área, contenção, materiais, triangulação e restauração por vizinhos nativos continuam posteriores. O contador separa oportunidades e colapsos realizados.

`RECOVERY_BORDERS=1` no laboratório compara contra os quatro pacotes escolhidos emrecovery-stability. O pacote anterior é reutilizado sem recompilar; `baseline_copy_ms` mede a cópia/codec de controle separadamente de`compile_ms`. Candidato sem redução estrita de faces e sem bytesGPU menores ou iguais não substitui o anterior. Isso é seleção de representação offline por geometria/memória, não garantia de que o tempo diminua; os tempos são medidos depois.

```sh
cargo build --locked --release --example recovery_probe
for f in 1 2 10 100; do
  RECOVERY_BORDERS=1 target/release/examples/recovery_probe artifacts/appearance-probe/recovery-stability/stable-$f.igc artifacts/appearance-probe/border-recovery $f
done
RECOVERY_BORDERS=1 target/release/examples/recovery_probe --run artifacts/appearance-probe/recovery-stability artifacts/appearance-probe/border-recovery
python3 scripts/report_recovery_stability.py --borders
node scripts/check_recovery_stability_ui.mjs --borders
```

O benchmark reutiliza o protocolo anterior:1728×1117 e3456×2234, distâncias2,7/64, oito poses mais retorno,32poses temporais,128frames/lote,16lotes medidos/caminho em oito rodadas alternadas com duplicatas. Referências independentes: fonte subdividida completa e superfície1×. Testes de erro não participam da escolha durante a medição. Materiais diagnósticos da pedra inalterados; o shader não foi modificado. Painel de capturas, não renderer interativo contínuo.

## Evidência geométrica

A hipótese foi confirmada:19.582 colapsos de caminhos duplicados retiram exatamente19.582 triângulos em2×. Nenhum colapso novo em1/10/100×. A seleção preserva byte a byte os pacotes anteriores nesses outros três fatores.

| Fator | Anterior selecionado T | Candidato bruto T | Escolhido T |
|---|---:|---:|---:|
| 1× | 66.122 | 66.122 | 66.122 |
| 2× | 85.705 | 66.123 | 66.123 |
| 10× | 66.151 | 66.184 | 66.151 |
| 100× | 67.139 | 67.139 | 67.139 |

A pedra2× passa de2.986.236 para2.281.284 bytes de proxyGPU (−23,6%); arquivoIGC1 de11.580.472 para10.797.192 bytes (−6,8%). Arquivo inclui fonte de fallback e certificados; esses números não sãoRSS nem redução da memória total da engine.100× conserva o ganho anterior; não atribuir a esta correção novamente.

Duas malhas controladas adicionais: pedra fechada de192T e superfície dobrada de128T, ambas com três materiais. Embaralhar antes da subdivisão produz arestas assimétricas de fato:118 e84 colapsos em2×. Para1/2/10/100×, exigir mesmos triângulos orientados, posições, normais e materiais da base, mesma contagem, codec válido e resultado equivalente após novo embaralhamento. Teste separado verifica duas incidências opostas por aresta na pedra fechada. Outro teste introduz relevo/normal não afim na borda assimétrica e exige sua preservação com fallback nativo. São provas geométricas controladas, não novos scans externos ou benchmarksGPU desses dois assets.

## Qualidade e tempos medidos

144/144 poses passam contra a superfície1×. As mesmas quatro poses distantes continuam falhando contra a fonte subdividida completa, sem nova falha. Em2×, máximoRMSE contra base1,1599e-7 (antes0,00427684), máximo de canal0,00013307, p99zero e cobertura divergentezero. Não equivale a igualdade de todos os pixels: a captura próxima2×/1728/frame0 inspecionada temPNG idêntico à base, mas nem todas as poses são idênticas. O problema de referência/rasterização distante descrito emrecovery-stability-study.md continua aberto.

Resolução3456×2234; tempos totais emms/frame de lote:

| Fator / câmera | Fonte | Melhor anterior | Escolhido | Base1× na coorte | Escolhido / base |
|---|---:|---:|---:|---:|---:|
| 1× / perto | 0.28462 | 0.28400 | 0.29106 | 0.30085 | 0.967× |
| 1× / longe | 0.21228 | 0.22416 | 0.21078 | 0.20901 | 1.008× |
| 2× / perto | 0.38974 | 0.31840 | 0.28693 | 0.28040 | 1.023× |
| 2× / longe | 0.20082 | 0.19888 | 0.19168 | 0.19048 | 1.006× |
| 10× / perto | 1.07964 | 0.31261 | 0.30508 | 0.29542 | 1.033× |
| 10× / longe | 0.30585 | 0.20633 | 0.20323 | 0.20169 | 1.008× |
| 100× / perto | 6.70306 | 0.26752 | 0.27226 | 0.27356 | 0.995× |
| 100× / longe | 2.51758 | 0.19026 | 0.18780 | 0.19051 | 0.986× |

2× perto nativo:0,31840→0,28693ms, **−9,9% de tempo total**;GPU0,27911→0,24891ms,−10,8%. Sete das oito rodadas favorecem o candidato; a outra fica0,69% acima. Duplicatas do candidato variam6,6% na mediana. Em1728 perto:0,19427→0,16910ms,−13,0%, oito rodadas favoráveis. Perto há evidência temporal além da redução geométrica.

Longe2×:1728 passa0,08365→0,08475ms (+1,3%),3456 passa0,19888→0,19168ms (−3,6%), frente a duplicatas11,1%/4,9%. Não afirmar ganho distante consistente. Os outros fatores usam arquivos byte-idênticos ao anterior; diferenças temporais entre esses caminhos são ruído/instrumentação, não melhorias novas.

O platô próximo agora apresenta0,29106/0,28693/0,30508/0,27226ms em1/2/10/100×. Normalizando pela base medida dentro de cada coorte:0,967/1,023/1,033/0,995×. Não interpretar100× mais rápido que1× como vantagem causal: as bases das coortes também variam. Em100×,6,70306ms da fonte densa contra0,27226ms do escolhido (24,62×) preserva o ganho já existente. A novidade é eliminar a sobra2×; não demonstra custo independente de novo detalhe visível.

## Checklist posterior e validação

1. Frequência: reconstrução de bordas e seleção somente offline; frame continua usando seleção/projeção e desenho anteriores.
2. Alocações: união usa retain/find nos Vecs existentes; sem novo vetor por colapso. Harness conserva cópia do pacote anterior para controle e fonte parafallback, declarados.
3. Tipos: erros acumuladosf64, geometriaf32/índicesu32, IGC1 intacto. JSON somente métricas/relatório, nunca representação de render.
4. Repetição: pacotes compilados antes da medição, sem descoberta/correção no runtime. Recompilar candidato em todos os fatores é custo de pesquisa/build, contabilizado separadamente da cópia do anterior.
5. Escala: grau de adjacência determina custo local de retain/find; sem promessa de build constante. Grafo e certificados continuam proporcionais à entrada.
6. GPU/I/O: mesmo shader e buffers tipados; nenhum atlas, readback ou cálculo novo no frame de produção proposto. Capturas/readback existem só na instrumentação.
7. Sincronização: coleta isolada de regressõesRust/MCP; aquecimento, oito rodadas, ordem alternada, duplicatas e base por coorte.
8. Invalidação: opções anteriores preservadas; três pacotes escolhidos comprovadamente byte-idênticos, fonte de cada comparação idêntica, decode/checksum e retorno validados.
9. Correção:690 testesRust release/all-targets, Clippy-Dwarnings, fmt,13gruposMCPheadless.320PNGs íntegros,64pares de retorno idênticos,144comparações,1.024amostras e1.024estadosDOM/DPRsimulados. Inspeção de captura próxima; sem aceite visual humano ou navegador nativo.
10. Custos/limites: fallback denso, certificados, memória de compilação e custo de preparaçãoGPU mantidos;RSSnão medido. Ganho desta correção restrito a2× perto. As outras malhas são fixtures procedurais com provas geométricas, não novas evidências deGPU/AAA.

Decisão: manter a correção como opção do compilador experimental; próximo passo é importar outra malha externa com topologia/material diferentes e comparar contra referência com integração de pixel para separar erro geométrico de aliasing. Ainda não promover ao renderer funcional. Scene/Command/RenderFrame/MCP e demo preservados.

Artefato corrente:`artifacts/appearance-probe/border-recovery/` contém painel, quatro pacotes selecionados, resultados, resumo, logs e verification.json. O experimento anterior permanece como controle necessário;target/release reutilizado, sem cópia de app, build histórico, push ou publicação.
