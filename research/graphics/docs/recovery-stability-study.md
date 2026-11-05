# Estabilidade da recuperação geométrica — EAF-88

## Hipótese e checklist prévio

O ensaio anterior deixou265.105T da fonte100× de6.612.200T;183.795T estão em regiões nativas rejeitadas. Instrumentar causas de rejeição antes de afrouxar qualquer limite. Hipótese: estimar plano e gradiente de normais a partir de um único microtriângulo amplifica arredondamento; usar pontos mais afastados da própria região pode estabilizar o modelo, revalidando todas as faces aceitas contra os mesmos limites. Manter controle legado disponível. Alternativa de aumentar tolerâncias rejeitada neste incremento, pois confundiria estabilidade com perda de qualidade.

1. Trabalho adicional só no build; sem nova seleção, campo ou busca por pixel no runtime.
2. Instrumentação com contadores e scratch regional; refits em tamanhos geométricos para limitar revisitas. Medir tempo/memória, sem prometer complexidade linear universal.
3. Mesmos formatos binários e f32 persistidos; cálculo intermediário f64, nenhum JSON no render.
4. Fonte/controle/candidato residentes no laboratório; distinguir payload útil e buffers comparadores.
5. Malhas embaralhadas1/2/10/100, compilador sem genealogia; regiões incertas permanecem fonte.
6. Mesmos shaders, material, resolução e câmeras para todos os buffers. Sem misturar redução regional/atlas nesta investigação.
7. Capturas e GPU benchmark separados das regressões; lotes alternados/duplicados.
8. Layout/checksum/certificados existentes; refit precisa revalidar a região, não herdar certificados da semente.
9. Gates RMSE≤0,005,p99≤0,02,cobertura≤0,0005; conferir contra fonte subdividida e pedra-base. Falhas explícitas, mais teste de detalhe não afim e fronteira.
10. Manter erro de rasterização/subdivisão separado do erro de compilação; nenhum contrato funcional/demo alterado, sem aprovação visual humana implícita.

## Diagnóstico e correção

Os certificados antigos100× separavam69.859 regiões,4.230 nativas. Contadores de estágio localizaram3.314 rejeições por contenção do contorno projetado,763 sem redução,152 na restauração das costuras e uma na triangulação. A semente única estimava plano/campo usando diferenças entre pontos cada vez mais próximos. Pequenos resíduosf32, extrapolados pelo gradiente calculado no microtriângulo, fragmentavam uma superfície originalmente afim em regiões que depois falhavam nos controles de contorno.

O modo opt-in `compile_traced(..., true)` conserva os três limites anteriores. Quando a região cresce para2,4,8,... faces, busca um ponto distante de uma âncora, outro distante do primeiro e um terceiro que maximize a área dessa base. Constrói o modelo emf64, orienta-o de acordo com o anterior e só o aceita se **todas as faces já admitidas** passarem novamente. O certificado final mede os erros contra esse modelo final. Não há aproximação da superfície por média sem validação, parental oculta, tabela por densidade ou relaxamento do gate fotométrico.

100× passou a66.191 regiões e67.139 triângulos, com apenas926 faces nativas. Rejeições por contenção3314→14; restauração152→7. Isso sustenta a hipótese de instabilidade do modelo, mas não demonstra que todo resíduo tenha a mesma causa. O compilador legado continua disponível e seu binário foi comparado byte a byte com o artefato anterior nas quatro entradas.

O refit bruto acrescentou33 triângulos no10× (66.151→66.184); não foi promovido nesse caso. O build compara as duas saídas e mantém a anterior quando o refit não reduz estritamente triângulos ou aumenta os bytes de vértices/índices. A escolha não consulta câmeras, imagens nem tempos medidos. Nesta entrada somente100× seleciona o refit. Em2×, o excesso85.705T permanece:19.584 regiões não têm redução possível com as fronteiras atuais. O fixture2× divide uma aresta por face independentemente; junções assimétricas e o bloqueio conservador de fronteiras permanecem para investigação, sem reparo topológico implícito.

Este comparador recebe oIGC1 anterior para reproduzir exatamente a mesma entrada já soldada. A função de compilação recebe somente `Mesh` e hash, sem acesso à proxy antiga ou à pedra-base. A escolha offline entre resultados usa a proxy anterior; o verificador usa a pedra-base separadamente. São papéis distintos.

## Controle dos pixels divergentes

Um comando diagnóstico independente (`--ray`) lança raios nos centros dos quatro pixels que apresentaram maior divergência. Usa a matriz de câmera do renderer convertida paraf64 e interseções raio/triângulo em dupla precisão, sem limiar de simplificação. Nos três pixels com interseção, a fonte1× e a densa concordam na posição, normal e cor calculada: maior diferença de cor entre pares cerca4,3e-9. O quarto pixel não tem interseção em nenhuma das duas, embora a rasterização100× o tenha coberto.

Os triângulos atingidos têm área projetada aproximadamente0,00407px² (2×),0,00273px² (10×) e0,000109px² (100×). As normais interpoladas têm comprimento0,917–0,993: a discrepância nesses pontos não é explicada por normal quase nula. A rasterização pode divergir inclusive na pedra-base: no pixel45°, o raio dá a cor escura próxima da densa, enquanto a rasterização1× é clara. Portanto tratar a imagem1× de amostra única como verdade física absoluta também seria incorreto.

Essas evidências apontam para divergência no caminho de projeção/rasterização/visibilidade/amostragem sob precisão finita, em vez de indicar relevo intencional novo. Não identifica isoladamente a regra interna exata doMetal e não calcula a integral de todo o pixel. Um raio no centro não é antialiasing nem referência integrada. Os quatro controles continuam reprovados contra sua fonte densa; não se altera tolerância, referência ou filtro silenciosamente para fazê-los passar.

## Medição e memória

Comparar fonte densa, recuperação anterior, recuperação escolhida no build e pedra-base1×. Todos usam o mesmo shader e um range de desenho, sem campo de normais ou seleção por muitas regiões. A cada frame medido dos dois compilados, o harness verifica erro projetado≤0,15px e normal≤0,000101, monta/codifica o desenho e envia uniforms. O ensaio é limitado a esse domínio elegível; fora dele o renderer de produção precisará escolher a fonte, não abortar como o verificador.

Duas resoluções1728×1117/3456×2234, distâncias2,7/64,32 poses de câmera/luz por lote128frames,8 rodadas alternadas/palindrômicas com duplicatas:16 amostras/caminho/coorte. Aquecimento antes de cada amostra. Capturas em8 ângulos+retorno,144 poses. GPU é janela de timestamps; total inclui CPU/submissão/espera/readback do lote, não FPS de gameplay. Não somar tempos parcialmente sobrepostos. O controle pedra-base também é medido em cada coorte para identificar variação de frequência/carga entre densidades; não presumir que todos os tempos absolutos de coortes distintas são diretamente comparáveis.

Uma primeira coleta serviu à checagem do harness; a coleta final substitui seus timings e inclui explicitamente a verificação dos bounds em cada frame. Rust/Clippy/MCP terminaram antes dessa coleta final. Não executar regressões GPU em paralelo com o benchmark.

Fonte/fallback é mantida noIGC1; o render carrega pacotes binários, sem JSON ou triangulação no frame. Vetores e modelos adicionais do refit existem somente no build. O processo do comparador mantém fonte, proxy anterior, proxy escolhida e base para comparar; estes buffers não devem ser confundidos com uma configuração mínima de gameplay. Em100×, só a proxy passa de7.107 para2.201MiB de buffers, cerca69% menos. A fonte densa continua ocupando163.484MiB se residente; não alegar economia de69% da memória total ou eliminação do fallback. Metadados, cópiasCPU, framebuffer/readback/driver e RSS não estão nesses totais.

O pacote100× completo passa de209.740.472 para203.627.808bytes: apenas cerca2,9% menos, porque contém a fonte. Redução de triângulos da proxy74,7% é diferente de redução do arquivo completo. Build compara dois algoritmos sequencialmente; o tempo das duas compilações é somado no painel, mais codec/escrita fora da medição `compile_ms`. Não foi implementado disparo seletivo do refit apenas nas regiões difíceis nem streaming/liberação da fonte.

## Reprodução

Com osIGC1 do ensaio `coordinated-regions` disponíveis:

```sh
cargo build --locked --release --example recovery_probe
for factor in 1 2 10 100; do
  target/release/examples/recovery_probe artifacts/appearance-probe/coordinated-regions/source-$factor.igc artifacts/appearance-probe/recovery-stability "$factor"
done
target/release/examples/recovery_probe --ray artifacts/appearance-probe/coordinated-regions artifacts/appearance-probe/recovery-stability
target/release/examples/recovery_probe --run artifacts/appearance-probe/coordinated-regions artifacts/appearance-probe/recovery-stability
python3 scripts/report_recovery_stability.py
node scripts/check_recovery_stability_ui.mjs
```

`RECOVERY_QUALITY_ONLY=1` executa capturas sem tempos e grava `quality-only.json`. Reprovação visual fica nos campos `pass` e `failures`; término do processo não equivale a aprovação de qualidade. Falhas de elegibilidade/certificado, identidade da fonte, retorno ou codec interrompem a execução. O painel exibe capturas estáticas com seleção de câmera/luz/referência; não é vídeo contínuo ou renderer no navegador. Escala física nominal porDPR disponível, além da visão geral reduzida.

## Resultado geométrico

| Fator | Fonte T | Anterior T | Refit bruto T | Escolhido T | Compilar anterior+refit |
|---|---:|---:|---:|---:|---:|
| 1× | 66,122 | 66,122 | 66,122 | 66,122 | 0.107s |
| 2× | 132,244 | 85,705 | 85,705 | 85,705 | 0.236s |
| 10× | 661,220 | 66,151 | 66,184 | 66,151 | 1.057s |
| 100× | 6,612,200 | 265,105 | 67,139 | 67,139 | 11.791s |

## Resultado visual e temporal final

O escolhido passou contra a pedra-base nas144 poses. Contra a fonte subdividida,140/144 passam e as quatro falhas distantes anteriores permanecem. Em100×, máximoRMSE contra base2,5202e-6, máximo de canal0,00148684 e cobertura divergentezero. Em2×, máximoRMSE0,00427684: não chamar todos os resultados de pixels idênticos. Não houve mudança de material, relevo intencional, filtro ou limiares para obter aprovação.

Resolução3456×2234, tempos totais emms/frame de lote:

| Entrada / câmera | Fonte | Anterior | Escolhido | Pedra-base na coorte | Escolhido / base |
|---|---:|---:|---:|---:|---:|
| 1× / perto | 0.26704 | 0.26868 | 0.26873 | 0.27201 | 0.988× |
| 1× / longe | 0.18828 | 0.18879 | 0.18252 | 0.18862 | 0.968× |
| 2× / perto | 0.35706 | 0.29513 | 0.28889 | 0.26013 | 1.111× |
| 2× / longe | 0.19341 | 0.19014 | 0.18718 | 0.18739 | 0.999× |
| 10× / perto | 0.98331 | 0.26641 | 0.26268 | 0.26855 | 0.978× |
| 10× / longe | 0.28655 | 0.19198 | 0.19377 | 0.19607 | 0.988× |
| 100× / perto | 7.00728 | 0.33107 | 0.27138 | 0.27513 | 0.986× |
| 100× / longe | 2.54456 | 0.18888 | 0.19230 | 0.19506 | 0.986× |

100× perto nativo:0,33107→0,27138ms, **−18,0% de tempo total**; GPU0,28757→0,22856ms,−20,5%. Sete das oito rodadas tiveram mediana pareada menor; uma ficou0,7% acima. Discrepância mediana das duplicatas do candidato5,1% nessa coorte. O ganho não depende apenas da redução de contagem ou de um timestamp isolado.

1×→100× próximo nativo:0,26873→0,27138ms, crescimento de aproximadamente1,0%;2× fica0,28889ms. O controlebase dentro das coortes dá razões0,988/1,111/0,978/0,986 para1/2/10/100×. Isso é evidência de custo quase estável para redundância da mesma pedra, com exceção residual no2×; não é garantia para100× mais detalhe visível. Contra desenhar a fonte100× inteira7,00728ms, o escolhido é25,82× mais rápido neste benchmark. A maior parte desse ganho já vinha do compilador anterior; ganho incremental deste trabalho é18%.

100× longe nativo:0,18888→0,19230ms (+1,8%), dentro de duplicatas8,8%, sem ganho total demonstrado. Em1728,100× perto0,16786→0,15288ms; longe0,15256→0,08878ms. Coortes leves escolhem o mesmo pacote anterior; diferenças temporais entre caminhos idênticos são ruído/instrumentação, não melhorias do refit. Duplicatas chegam a17,6% em uma coorte distante. Tempos absolutos mudaram entre a coleta preliminar e a final: conservar só os finais e usar pares/controlebase, sem extrapolação aFPS.

Decisão: conservar refit como opção de build comprovada neste domínio, com escolha anterior quando não há redução de geometria/bytes. Não integrar automaticamente ao runtime geral. Próximo foco: explicar/reparar as restrições topológicas do2×, validar contra referência integrada e ampliar para outras malhas externas antes de prometer platô universal.

## Checklist posterior e validação

1. Frequência: refit e escolha entre compilações exclusivamente offline; frame conserva matriz/luz/bounds e desenho convencional.
2. Alocações: refit usa iteradores sobre faces existentes para buscar três pontos e revalidar; sem coleção extra por refit. Compilações sequenciais e buffers comparadores ainda consomem memória; RSS não medido.
3. Tipos: f64 para modelo/interseções diagnósticas, f32/u32 emIGC1/GPU. Contadores/JSON somente relatório, sem novo formato público/dependência.
4. Reuso: pacote/buffers residentes; aumento de trabalho de build em troca de proxy menor. Build completo duplo mantido e contabilizado.
5. Escala: refit em potências de dois limita revisitas a uma região; adjacência/sort/topologia continuam proporcionais à entrada com seus custos anteriores. Não é compilador de custo constante.
6. GPU/I/O: shader inalterado, sem textura nova ou raio no runtime. Fonte continua disponível parafallback, comandosbinários consumidos uma vez.
7. Sincronização: coleta final após Rust/Clippy/MCP; aquecimento, alternância, duplicatas e controlebase por coorte. GPU/CPU/total não somados.
8. Invalidação: quatro controles legados byte-idênticos ao recompilar; código opt-in e pacote validado porcodec/checksum. Certificados são medidos novamente após refit.
9. Correção:687 testesRust release/all-targets, Clippy-Dwarnings, fmt/diff e13gruposMCP headless passaram. Fixture numérico reduz98→1T com mesmos limites; testes preservam relevo/normais não afins, materiais, fechamento e contagens após embaralhamento.144 poses,1.024 amostras temporais,320PNGs íntegros,64pares de retornoPNG idênticos e1.024estadosDOM/DPRsimulados. PNG próximo inspecionado; sem navegador nativo ou aceite humano.
10. Custos/limites: fonte de fallback, certificados, cópiasCPU/targets/instrumentação mantidos. Quatro diferenças contra fonte densa continuam abertas; referência por raio central não é integral do pixel. Sem modificação funcional/NPC/Scene/Command/RenderFrame/MCP/demo.

Artefato corrente em `artifacts/appearance-probe/recovery-stability/`: painel, resultados, resumo, diagnóstico de raios, quatro pacotes, logs e verificação com hashes. Dois outputs de qualidade preliminar supersedidos foram removidos após confirmar ausência de uso comlsof. `target/release` corrente reutilizado; nenhum build histórico, push ou publicação novo.
