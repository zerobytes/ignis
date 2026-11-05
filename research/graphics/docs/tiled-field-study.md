# Busca compartilhada por blocos — EAF-88

## Protocolo e checklist antes de implementar

Hipótese: consultar a hierarquia uma vez por bloco de tela pode custar menos que repetir o estreitamento por fragmento. O pré-passe calcula uma faixa conservadora de alturas para todo o feixe de raios do bloco. Cada pixel intersecta essa faixa e resolve a superfície fina original. Um bloco não representa uma única cor nem substitui pixels por amostras maiores.

1. Frequência: IHF1 permanece compilado offline; intervalos de tela dependem da câmera e são preparados uma vez por bloco/frame.
2. Alocações: scratch GPU persistente limitado à resolução máxima do laboratório; sem listas por pixel. Buffers binários de alturas/limites preservados.
3. Tipos: faixa de duas alturas f32 por bloco (8 B), sem média ou quantização do relevo. Cores/normais/profundidade iguais ao candidato anterior.
4. Repetição: quatro cantos de um feixe, quatro consultas de nós por rodada; compartilhar entre pixels. Pré-passe apenas no retângulo conservador do objeto.
5. Escala: testar blocos 8/16/32 em calibração independente; quatro refinamentos por bloco e no máximo 32 etapas finas por pixel. Fora do domínio certificado, B.
6. GPU/I/O: compute + barreira + leitura por fragmento devem entrar no tempo total e janela GPU; sem readback na decisão.
7. Sincronização: scratch escrito antes do draw no mesmo encoder; invalidado em cada frame. Sem histórico temporal ou pressuposto sobre a luz.
8. Invalidação: política assinada por shaders/dados/tamanho do bloco/adaptador/modelo. Câmera/resolução atualizam parâmetros; testes de retorno/troca de resolução.
9. Correção: raios dos cantos delimitam o feixe em cada plano horizontal quando todos apontam para baixo. Havendo horizonte no bloco, usar faixa global. Conservar margens f32, bordas e resultado fino; validar qualidade/depth contra full e hierarquia por pixel.
10. Custos mantidos: scratch, passe compute e fonte/B residentes. Não promover redução de leituras a ganho sem comparação pareada e margem sobre dispersão.

Comparar B135, H345 (hierarquia por pixel), T395 (blocos forçados dentro do certificado), C405 (opcional calibrado). Mesmo ambiente, duas resoluções físicas, câmera/luz girando, perto/médio/longe. Os fatores históricos de quatro assets mudam relevo, portanto não provam platô de subdivisão. Controle adicional usa a superfície fixa `f1_d2` e o certificado de construção da subdivisão, medindo o residual B/H/T sobre o mesmo conteúdo; contagens densas só podem ser chamadas de novas medições quando buffers realmente construídos e desenhados.


## Tamanho do bloco e controle de conteúdo

Calibração inicial de blocos 8/16/32 usou quatro coortes (fontes 1×/100×, duas resoluções, distâncias 16/48). Bloco 8 mantido por ter o menor custo relativo à hierarquia por pixel nas coortes pesadas; não houve ganho robusto contra B. Registros em `tuning.json`. Esta escolha precede a validação em 14/28/224.

O controle de conteúdo fixo constrói e desenha buffers densos reais da parental f1_d2 (327.680 triângulos). Metas 1/2/10/100 triângulos submetidos por pixel coberto na pose inicial próxima, denominador medido por captura. O certificado de construção evita examinar a fonte densa no frame; compara o residual anterior B com H/T/C sobre a mesma parental. Fonte densa liberada entre metas, sem retenção das quatro simultaneamente. Buffers gerados não são salvos em disco.

Antes da coleta final, a calibração parcial foi descartada/reiniciada para congelar o harness final com os mesmos quatro ângulos em todos os caminhos do controle fixo: denso usa quatro frames/bloco e otimizados repetem essas quatro poses oito vezes. Isso evita comparar distribuições de câmera diferentes; as medições do controle fixo ainda não tinham iniciado. Câmera e luz mudam entre as poses. Esse controle mede throughput amortizado com lotes diferentes, não latência individual, e sua resolução temporal é menor que os 64 frames do experimento principal.

## Implementação e limites matemáticos

Os arquivos IGF1/IHF1 existentes são os mesmos dados compilados; não se introduziu outra cópia de alturas ou tabela de cores. `tiled_field.wgsl` usa o feixe definido pelos quatro cantos do bloco para construir um retângulo XZ entre duas alturas. Com denominador vertical de sinal constante, os extremos das razões de direção ocorrem nos cantos. Cada rodada consulta a união de até quatro nós da escala adequada e reduz a faixa de alturas, mantendo uma faixa comum a todos os pixels do bloco. Havendo risco de cruzar o horizonte, conserva a faixa global. O fragmento ainda faz AABB, corte contra essa faixa e o mesmo hit triangular fino; o bloco não filtra a imagem.

Bounds em f32 têm expansão nas folhas e margens no cálculo do feixe e do intervalo. O argumento geométrico exato não é uma prova de arredondamento universal em todas as GPUs. Os testes de inclusão do feixe, indexação parcial, qualidade e profundidade verificam o domínio amostrado; limites/ressalvas do heightfield anterior continuam válidos.

Um buffer `Storage` de 967.680 B (0,923 MiB) é reservado uma vez para até 3456×2234 em blocos 8×8. A resolução menor usa 241.920 B lógicos desse scratch; a reserva continua sendo a máxima. Parâmetros compute: 128 B. A faixa em tela depende da câmera, portanto é calculada em runtime; a hierarquia espacial e as alturas continuam preparadas em build time. Câmera/resolução mudam uniformes, não provocam alocações por frame. A mesma faixa serve a qualquer luz deste material; nenhuma iluminação fica congelada pelo bloco.

Compute só despacha o retângulo conservador do objeto, incluindo borda de rasterização. O draw usa o mesmo scissor, e o scratch só é lido dentro dessa região preparada. A transição storage-write → storage-read ocorre entre compute e render no mesmo encoder. Timestamps começam no compute e terminam no render; a janela por lote inclui ambos e a dependência. Este recorte não separa fisicamente custo de banda, ALU, busca residual ou sincronização; uma piora não identifica sozinha qual deles domina.

O relatório mantém dois conjuntos separados: quatro fontes históricas com relevo diferente e o controle de uma superfície fixa subdividida. O segundo reutiliza o certificado de genealogia conhecido da construção. Ainda não extrai equivalência de uma malha arbitrária recebida pelo compilador.

## Resultados — 27/10/2026, Apple M4 Pro / Metal

A hipótese não demonstrou ganho incremental. A política congelada manteve B em todos os casos. Quarenta coortes de calibração, suporte 3/5/3/1 por classe; classes com suporte também tiveram regressões. Não há ajuste por vencedor da validação. H e T foram medidos na mesma execução, com o mesmo domínio de elegibilidade.

100× histórico, resolução física 3456×2234, tempo total amortizado por frame:

| Distância | B anterior | H por pixel | T blocos | C opcional | T/B | T/H |
|---|---:|---:|---:|---:|---:|---:|
| Perto 14 | 0,63641 ms | 1,02791 ms | 1,01833 ms | 0,64605 ms | +60,0% | −0,9% |
| Médio 28 | 0,38675 ms | 0,51205 ms | 0,54527 ms | 0,39948 ms | +41,0% | +6,5% |
| Longe 224 | 0,19200 ms | 0,18732 ms | 0,24397 ms | 0,19615 ms | +27,1% | +30,2% |

Repetição próxima B/H/T = 0,63932/1,02614/1,02737 ms; longe = 0,18685/0,18856/0,24821 ms. O aparente ganho de 0,9% contra H perto não se repetiu; não interpretá-lo como melhoria. Mediana T/H das 24 coortes = 1,06437; nas 16 repetições = 1,03707. T/B = 1,41919/1,55839. C/B = 1,00865/0,97535 é variação de controle com o mesmo caminho, não ganho. Razão entre duplicatas B/T/C variou de 0,7778 a 1,3841.

GPU próxima B/T = 0,62146/1,03157 ms; média distância = 0,37803/0,53588; longe = 0,19568/0,21044. CPU_submit próxima B/T = 0,04315/0,03952 ms; longe = 0,02685/0,04356. Métricas coletadas separadamente; não somar nem converter em FPS de jogo.

A auditoria desta execução não registrou sobreposição entre frames, inclusive nos controles B/C. O verificador herdado exigia alguma sobreposição e rejeitou a coleta por esse motivo. Corrigido: sobreposição é observação, não requisito de correção; manter monotonicidade, faixa válida e janela GPU compatível com a parede do mesmo lote. Nenhum timestamp ou medição foi alterado. A mudança de comportamento em relação ao estudo anterior não autoriza atribuir toda a regressão à barreira do scratch.

## Controle de superfície fixa

Parental f1_d2, mesma superfície/material/normais, câmera/luz em quatro posições por bloco. Em resolução nativa, 2.544.674 pixels cobertos na pose inicial a14. Metas referentes a triângulos fonte submetidos/pixel coberto, não triângulos que vencem o teste de profundidade.

| Meta T/pixel | Triângulos fonte | Fonte densa | B anterior | H por pixel | T blocos | C opcional |
|---|---:|---:|---:|---:|---:|---:|
| 1 | 2.544.674 | 1,56758 ms | 0,47930 ms | 1,19898 ms | 1,14625 ms | 0,44107 ms |
| 2 | 5.089.348 | 2,67001 ms | 0,45344 ms | 1,22503 ms | 1,13836 ms | 0,44241 ms |
| 10 | 25.446.740 | 7,22815 ms | 0,46877 ms | 1,27404 ms | 1,22947 ms | 0,45255 ms |
| 100 | 254.467.400 | 53,64820 ms | 0,45789 ms | 1,22983 ms | 1,19714 ms | 0,44624 ms |

B já apresenta platô para redundância conhecida: 0,45–0,48 ms, sem crescer junto com a fonte. C preserva B; as pequenas diferenças entre ambos são controles. T não reduz esse patamar e custa aproximadamente 2,4–2,6 vezes B perto. Não apresentar a economia frente ao denso como ganho novo dos símbolos. Longe, resultados completos constam no relatório: as diferenças pequenas B/H/T são instáveis; não foram usadas para mudar a política. Duplicatas deste controle variaram de 0,7194 a 1,3487.

O maior buffer denso lógico agregado tem 4.743.597.840 B (4,418 GiB), 32 draws; preparo/upload 2.656,44 ms, fora do tempo de frame. Scratch CPU máximo 224 MiB, buffers individuais até128 MiB. Uma fonte densa por vez, liberada entre metas e ao terminar; nada persistido em disco. Isso não é memória exigida pelo símbolo. O comparador mantém a fonte para desenhá-la e medi-la; B e os símbolos continuam residentes. Sem medição de RSS/VRAM física.

## Qualidade e validação

- Principal: 288 poses, 864/864 gates aprovados. T ativo em268 poses; próximo100× ativo35/64 frames temporizados, preservandoB nos demais. C=B em RGBA32F em todas as poses.
- T contra full: RMSEmáximo0,00284727, p99máximo0,000594735, cobertura divergente máxima0,000137155; não pixels idênticos. Resíduo temporal máximo0,00265885. Mesmos gates do estudo anterior, sem relaxamento.
- Dezoito checks depth ativos, erro máximo em pixels comuns1,77e−8 e cobertura depth=alpha. 96 fechamentos PNG idênticos, oito baselines legados idênticos.
- Controle fixo:48 poses,240/240 comparações aprovadas; todos os certificados parentais válidos, erro projetado máximo0,00268543px. Subdivisão/full parental RMSEmáximo0,00151120; B/C contra denso0,00284187, H/T0,00151019. Cobertura divergente máxima0,000143431. Forçar certificado inválido conservou pixels/contagem da fonte densa em todas as oito fontes.
- 653 testes Rust release/all-targets, Clippy com `-D warnings`, fmt/diff;13 grupos MCP headless (render, replay_inspection, selection). Dois novos testes cobrem inclusão de raios interiores pelo feixe e cobertura/indexação de blocos parciais em diferentes resoluções.
- 872 PNGs principais mais16 fixos íntegros. Seis PNGs de smoke dos consumidores geométrico/hierárquico anteriores permaneceram byte-idênticos. Fontes/política congeladas;40+24+16 coortes, quatro auditorias.
- Harness DOM:576 estados, DPR1/1,5/2/3 e recortes físicos; não browser nativo nem aceite humano. Inspeção visual de uma captura próxima foi complementar aos dados físicos, sem usar imagem redimensionada como certificado.

## Checklist posterior e decisão

1. Build time preservado em IGF1/IHF1; novo trabalho por bloco depende da câmera e pertence ao frame.
2. Scratch persistente e uniformes fixos; sem alocações no loop de decisão/compute. Capturas/benchmarks alocam fora do caminho de jogo medido.
3. Dados binários R32/RG32 e `vec2<f32>`; nenhum JSON no shader/seletor. JSON continua restrito ao harness/manifestos do laboratório.
4. Consultas hierárquicas compartilhadas, porém cada pixel mantém o hit fino. Não houve contagem de etapas finas neste incremento; não afirmar que sua variação foi medida.
5. Limites explícitos3456×2234, bloco8/16/32,64certificados, quatro estreitamentos e32iterações finas. Sem promessa para telas maiores ou malhas fechadas.
6. Compute, dependência de storage e draw incluídos no custo; CPU_submit contém driver/backpressure.
7. Sem readback de decisão, estado temporal ou trabalho omitido do timing. Lotes finais fixos usam as mesmas quatro poses em todos os caminhos.
8. Scratch refeito no retângulo lido a cada frame; troca de resolução e retorno validados. Política ligada aos dados/shaders/factory/tamanho/adaptador/modelo; manifesto congela fontes.
9. Cor/cobertura/depth/retorno/legado e fallback inválido verificados; margens f32 não são prova universal.
10. Sem mudanças de runtime, NPC, Scene, Command ou MCP. Experimento não promovido à engine; caminho anterior permanece padrão, memória extra só no comparador.

Decisão de pesquisa: não perseguir esta implementação por blocos como ganho demonstrado. A evidência reforça o valor da eliminação de redundância compilada que já estabiliza B. O próximo problema mais útil é generalizar certificados/regiões a uma malha sem genealogia conhecida, preservando contorno/material, ou demonstrar uma representação cujo custo residual seja menor que a rasterização otimizada. Adicionar outro estágio de busca por pixel precisa justificar seu custo completo; redução de triângulos/leituras isoladamente não basta.

Reprodução: `python3 scripts/run_tiled_field.py`, com inputs do estudo hierárquico existentes; `--tune-tiled-field` para8/16/32. Evidências em `artifacts/appearance-probe/tiled-field/`. O relatório contém a comparação principal e uma tabela separada do controle fixo.
