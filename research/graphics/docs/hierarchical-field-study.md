# Hierarquia geométrica compilada — protocolo EAF-88

Comparar A=geometria completa, B=135 anterior otimizado, D=símbolo hierárquico elegível, C=B+símbolo opcional. Controle adicional: símbolo plano anterior, medido na mesma execução. Fontes, materiais, câmeras, resolução física e gates do estudo geométrico preservados. Não tratar fatores históricos1/2/10/100 como subdivisão redundante. Domínio continua heightfield; nenhuma integração funcional ou alegação sobre pedra fechada.

## Checklist antes de implementar

1. Frequência: hierarquia/certificados em build time; upload uma vez; decisão por objeto e estreitamento do intervalo por pixel.
2. Alocações: limites contíguos e mipmaps persistentes; sem alocar listas/mapas nem reconstruir malha no frame.
3. Tipos: alturas originaisR32Float; limites conservadoresRG32Float com arredondamento para fora; binárioLE versionado/checksummed. Não quantizar detalhe.
4. Repetição: reduzir acessos à superfície fina com limites regionais; guardar alturas somente uma vez e compartilhar material/normais. Benchmark contraB e controleplano.
5. Escala: folhas8×8células, pais2×2; árvore limitada pelo grid. CertificadoCPU de64regiões, sem scan de triângulos no frame. Busca residual mantém limite32iterações.
6. GPU/I/O: upload binário único, sem buildGPU/readback na seleção; avaliar custo de textura/memória adicional e consultas hierárquicas.
7. Sincronização: calibração congelada antes do teste, lotes separados de captura; GPU pela janela do lote, latências sobrepostas só como diagnóstico.
8. Invalidação: header/hash/dimensões e vínculo ao campo fonte; mudanças no shader/dados/modelo/adaptador invalidam política.
9. Correção: nós contêm todos os filhos/triângulos; limite regional de derivadas garante raiz única. Testes de corrupção, limites, clipping conservador, gatesvisuais/depth e retorno; geometria em vistas não certificadas.
10. Custos mantidos: fonte/B residentes para fallback, campo fino para hit/material, certificado conservador. Hierarquia pode custar mais que economiza: medir e rejeitar classes sem benefício.

## Hipótese matemática

No intervalo candidato do raio, as coordenadasXZ formam um retângulo. Escolher a escala cuja célula contém a extensão desse retângulo permite cobri-lo por até quatro nós. A união de seus mínimos/máximos de altura contém qualquer interseção possível nesse intervalo; recortar o raio contra essa faixa não remove soluções. Repetir em escalas menores, depois resolver a mesma superfície triangular original. O símbolo não substitui relevo por uma média.

Certificação local: em cada uma de64regiões, compilar mínimo/máximo das derivadasX/Z. Limitar `dy - gx*dx - gz*dz` usando os extremos de posição/derivada e o máximo global de altura; exigir negativo com margem em todas as regiões. Isso certifica monotonicidade de todo raio possível no volume, com limites mais precisos que máximos globais absolutos. Se qualquer região falhar, manterB. O certificado não depende de consultar pixels renderizados.

## Implementação

`IHF1` guarda header56, vínculo porhash aoIGF1,64certificados com quatro derivadasf32 e mipmaps de intervalosRG32Float. Folhas abrangem8×8células e todos os seus vértices; alturas interpoladas linearmente ficam entre os extremos. Expansão de2e-6 e arredondamento para fora nas folhas, pais por uniãoexata. Padding fora do grid usa sentinelas finitas vazias; consultas são limitadas às regiões que contêm células reais. Compilador requer grid divisível por8 para alinhar as64regiões de certificação. Nada é reconstruído naGPU ou no frame.

O intervaloXZ do raio seleciona uma escala com célula tão larga quanto sua extensão. Consultar os quatro cantos cobre todos os nós intersectados nessa escala. A união de alturas corta o intervalo emt, com folga proporcional a|t| para aritméticaf32. Repetir até quatro rodadas ou até chegar àfolha/contração pequena. A busca fina32máximo e a solução analítica do triângulo local são preservadas. Mudar a escala não muda a superfície renderizada; não há filtragem de altura nem média de cor. Cor/normais/material/profundidade permanecem os originais.

CertificadoCPU usa limites assinados por região: `max(-gx*dx)+max(-gz*dz)` em cada retângulo, dividido por `eye.y-hmax_global`; exige pior valor<.95 eeye.y>hmax+.1. Isso custa até64regiões por objeto ativo, incluídas no tempo total. Máscara de custo0 evita esse scan no caminho opcional. A escolha continua por objeto inteiro, não por fragmento; não declarar mistura regional de malha/símbolo entregue.

A=full0; B=135; D=345hierarquia forçada dentro do certificado; C=355opcional; E=385controle sem hierarquia com o **mesmo certificado regional** deD. E permite atribuir diferenças à busca hierárquica, inclusive onde o símbolo antigo rejeitava a vista. E não é a política de elegibilidade antiga. Modo325 continua disponível com certificado global original. Diagnósticos365/375 usam pipelines separados para contar etapas, não contam como medição de tempo. A captura de trabalho também compara as imagens E/D.

## Calibração e controles

Antes da validação, comparação de limites1/2/4rodadas em quatro coortes de calibração (fontes1×/100×, duas resoluções, distâncias16/48). Quatro rodadas mantidas: melhor normalização contra controle plano no100× pesado nas duas resoluções; não é ótimo universal. Valores e dispersão em`tuning.json`; essa escolha não usa as poses de validação. A contagem efetiva pode parar antes de quatro.

Calibração de seleção ampliada para40coortes, distâncias16/24/48/96/192, mantendo três coortes integralmente na classe e benefício>5%acima da dispersão como exigência. Maior cobertura evita repetir o suporte insuficiente de todas as classes do estudo anterior; classes que ainda não tenham suporte continuam rejeitadas. Política congelada antes dos testes14/28/224. Quatro classes fixas pela razão de triângulosB/área projetada; não há tabela porasset/pose nem leitura de pixels no frame.

Mantidos: oito rodadas, duas duplicatas por modo, ordem alternada,64frames com câmera/luz atualizadas e32frames para timestamps. GPU é a janela entre primeiro início e último fim/32; `pass_latency` registra latências sobrepostas separadamente. CPU_submit inclui driver/backpressure e não é somável àGPU. Repetição perto/longe e auditoria de timestamps. Calibração mede B/D/E; validação B/D/C/E.

## Memória e build

| Fonte | HierarquiaGPU | Alturas+hierarquiaGPU | CertificadoCPU | Escalas |
|---|---:|---:|---:|---:|
|1×|2.728B|69.292B|1.024B|5|
|2×|5.464B|138.076B|1.024B|6|
|10×|43.704B|702.652B|1.024B|8|
|100×|699.048B|7.262.892B|1.024B|9|

No100×, a hierarquia acrescenta0,667MiB ao campo de6,260MiB; total lógico6,926MiB além deB. ArquivoIHF1 tem700.128B. Sem duplicar alturas/normais no controle plano; ambos compartilham dados. Mipmaps e vetores de alturasCPU são liberados após upload, certificado permanece. Contagens porlayout, não medição deRSS/VRAM física ou objetos de driver. B, posições compactadas, índices e normais ainda residentes para fallback; não alegar economia líquida de memória. Build da hierarquia100× medido em cerca de36ms nesta execução, separado do build original do campo.

## Reprodução e fronteiras

`python3 scripts/run_hierarchical_field.py`; inputs anteriores dehybrid-scaling,near-plateau,automatic-shading egeometric-field. `--tune-hierarchical-field` reproduz a comparação1/2/4emcoortes de calibração; não muda a política compilada. Saídas correntes em`artifacts/appearance-probe/hierarchical-field/`, sem substituir o relatório anterior aberto. UI conserva resolução física, recortesDPR e playback das capturas. Testes nativos de apresentação/aceite humano separados.

Câmera360°, oscilação de distância±12%nas faixas14/28/224 e luz360°; controles independentes no100×a14/28. Fontes históricas acrescentam detalhe real; não são1/2/10/100triângulos porpixel nem o teste de226milhões com superfície fixa. Domínioheightfield visto de cima, sem pedra fechada/multicamada, deformação,GI,sombras, composição com obstáculos ou contrato de renderização daengine.

## Resultado medido em 27/10/2026

A hierarquia reduziu a busca fina e o custo do símbolo, mas ainda não demonstrou benefício incremental robusto sobre B. Calibração congelou máscara zero: C usa B em todas as poses, sem consultar certificados ou texturas adicionais por frame. As quatro classes tiveram 3/5/3/1 coortes válidas; além de suporte insuficiente na última, houve regressões nas classes com suporte. Não ajustar essa política pelos vencedores da validação.

Tempo total de throughput por frame, 100×, resolução física 3456×2234; média das medianas de duas duplicatas, oito rodadas. E e D usam o mesmo certificado regional, portanto a comparação mede a mudança de busca. Próximo: símbolo ativo em 35/64 frames, B nos demais; médio/longe: 64/64 ativos.

| Distância | B anterior | E símbolo plano | D hierarquia | C opcional | D versus E | D versus B |
|---|---:|---:|---:|---:|---:|---:|
| Perto 14 | 0,60877 ms | 1,19701 ms | 0,95527 ms | 0,60611 ms | −20,2% | +56,9% |
| Médio 28 | 0,38011 ms | 0,59173 ms | 0,47644 ms | 0,38052 ms | −19,5% | +25,3% |
| Longe 224 | 0,18620 ms | 0,22630 ms | 0,18300 ms | 0,19130 ms | −19,1% | −1,7% |

Repetição próxima: B/E/D = 0,59885/1,20670/0,96408 ms; longe: 0,18352/0,21717/0,17455 ms. A melhora contra E se repetiu, mas as pequenas vantagens distantes contra B não satisfazem a calibração nem justificam promoção. Na resolução 1728×1117, longe, D ainda custou 0,12673 ms contra B 0,06038 ms (+109,9%); reduzir trabalho fino não elimina custo fixo/leitura/seleção.

GPU pela janela do lote, nativo próximo B/D = 0,60214/0,94046 ms; médio 0,36295/0,46558; longe 0,15733/0,15271. CPU de submissão próxima B/D = 0,04158/0,03256 ms, inclui driver/backpressure. Não somar estas métricas nem convertê-las em ganho de FPS de um jogo. Nas 24 coortes, mediana D/E = 0,87690 e D/B = 1,32742; repetição 16 coortes = 0,88098 e 1,37473. C/B = 1,00417 e 1,01471 é controle sem símbolo, não ganho. Razão entre duplicatas de B/D/C variou de 0,7199 a 1,2972.

Contadores em pipelines separados: no 100×, etapas finas caíram aproximadamente de 5,4–5,8 para 1,6–1,8 por hit (redução de 68,0–70,2%), acrescentando cerca de 2,7–3,1 rodadas hierárquicas. Há até quatro leituras RG por rodada, operações de escala/intervalo e busca fina residual. Não é custo zero nem uma consulta única. A contagem é por hit coberto, não por todo fragmento executado. Sem contadores de hardware, não atribuir o gargalo exclusivamente à banda ou à ALU.

## Fidelidade e cobertura

864/864 gates passaram em 288 poses, com limites anteriores RMSE ≤0,005, p99 ≤0,02 e divergência de cobertura ≤0,0005. D foi elegível em 268 poses (20 fallbacks), contra 196 no experimento anterior. No 100× próximo, 38/54 capturas elegíveis e 35/64 frames dos blocos temporizados; o certificado global antigo rejeitava todos os próximos. Isso amplia o domínio demonstrado, não garante todos os ângulos.

D contra geometria completa: RMSE máximo 0,00284727, p99 máximo 0,000594735, divergência máxima de cobertura 0,000137155. Não são pixels idênticos nem contornos matematicamente idênticos à rasterização. B e C: RMSE máximo 0,00319861, p99 0,0148781, cobertura sem divergência. C = B em RGBA32F em todas as poses. Resíduo temporal máximo D 0,00265885 e B/C 0,00361047; nove capturas por percurso não comprovam ausência de popping entre amostras.

24 comparações E/D aprovadas, sem divergência de cobertura nas poses verificadas; RMSE máximo no 100× 2,67e−7. Dezoito verificações de profundidade, todas com símbolo ativo; erro máximo em pixels comuns 1,77e−8, cobertura depth igual à alpha. Não comprova composição com objetos móveis. Retorno +360°: 96 pares de PNGs idênticos. Oito baselines legados e três PNGs de smoke do consumidor geométrico anterior permaneceram byte-idênticos.

## Validação e checklist posterior

651 testes Rust release/all-targets aprovados, Clippy com `-D warnings`, fmt e diff-check. Treze grupos MCP headless (render, replay_inspection, selection) aprovados. Verificação de 872 PNGs, fonte/checksum/política congelados, 40+24+16 coortes, quatro auditorias de timestamps e 24 contadores de trabalho. Harness DOM validou 576 estados e recortes físicos em DPR 1/1,5/2/3. Inspeção de duas capturas próximas foi complementar; métricas operam nas imagens físicas originais. Não houve teste de apresentação nativa nem aceite humano.

O verificador Python tinha um dígito errado no offset FNV; corrigido para o mesmo FNV-1a usado pelo compilador/decoder, e todas as assinaturas foram revalidadas. Nenhum dado medido ou binário precisou ser alterado para passar essa checagem.

1. Frequência: `hierarchical_field::compile` constrói limites/certificados offline; frame só consulta dados persistentes.
2. Alocações: vetores de build/load liberados após upload; sem heap novo no certificado ou no shader por frame.
3. Tipos/acesso: R32Float mantém detalhe, RG32Float mantém bounds; layout IHF1 validado. Quantização dos limites ficou adiada para preservar conservadorismo.
4. Repetição: folhas e pais calculados uma vez; quatro consultas por escala incluem duplicatas possíveis. Esse custo foi conscientemente mantido e medido.
5. Escala: até 64 regiões CPU, quatro rodadas GPU e 32 etapas finas; sem varrer todos os triângulos. Ainda seleção por objeto inteiro.
6. GPU/I/O: upload único; sem readback/I/O na decisão. Memória adicional de 0,667 MiB no 100×, sem economia líquida residente.
7. Sincronização: timestamps/readbacks restritos ao laboratório; GPU usa janela de lote, controles duplicados evidenciam variância.
8. Invalidação: IHF1 ligado ao IGF1; política ligada a dados/shaders/rodadas/adaptador/modelo. Manifesto de fontes congelado antes da calibração.
9. Correção: três testes novos de contenção/hierarquia/codec/certificado; gates de cor/cobertura/depth/retorno/legado; checksum não substitui prova de bounds.
10. Contratos/custos: nenhum runtime, NPC, Scene, Command ou MCP alterado. Manter B e recursos de fallback; D diagnóstico, C conservador. Próximo recorte justificável é reduzir o custo por pixel ainda presente e testar seleção de malha/símbolo por região; não promover esta versão como platô próximo resolvido.
