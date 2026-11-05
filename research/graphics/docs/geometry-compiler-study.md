# Compilador sem genealogia — EAF-88

## Protocolo anterior à implementação

Entrada pública do compilador: malha indexada com posições/normais f32 e material u32 por face. Nenhum grid, pai, nível de subdivisão, fórmula procedural ou arquivo parental. Gerador de fixture, compilador e consumidor são comandos separados; permutar índices, vértices e orientação cíclica das faces antes de escrever o input. A malha parental é referência exclusiva do gerador/verificador, não um recurso do compilador.

Descobrir adjacência por identidade geométrica exata, crescer regiões compatíveis com um plano e um campo afim de normais. Fronteiras de materiais são barreiras. Cadeias de borda compartilhadas são simplificadas globalmente com limite de desvio acumulado; não tomar decisões independentes em dois lados da mesma borda. Validar região como disco com projeção convexa, orientação consistente e área coberta antes de criar um leque de triângulos. Regiões sem certificado conservam seus triângulos.

Certificados limitam deslocamento de superfície e diferenças do campo de normais; rasterização e iluminação ainda exigem validação por pixels. Primeiro nó global agrega certificados das regiões e habilita a malha compilada quando o erro projetado em pixels físicos cabe em0,15px. Fora desse domínio, fonte integral. Não prometer seleção regional de LOD/múltiplas escalas já neste recorte. Esse teste busca remover redundância conhecida na fixture mas desconhecida do compilador, não descartar detalhes novos.

Testar uma pedra procedural fechada e uma superfície com relevo/dobra/bordas abertas, materiais variados. Fontes1/2/10/100 multiplicam o número de triângulos por subdivisão da mesma superfície, não são metas de triângulos porpixel. Reportar cobertura/densidade efetiva. Câmera perto/longe girando e luz variável; gates anteriores RMSE≤0,005, p99≤0,02, divergência de cobertura≤0,0005. Comparação de custo entre fonte e compilado no MESMO renderer/material; não comparar timings absolutos ao laboratório heightfield anterior.

## Checklist prévio

1. Frequência: adjacência/regiões/bordas/certificados apenas em build time; frame só projeção de bound e escolha de buffers.
2. Alocações: vetores contíguos e ordenação offline; nenhuma árvore de objetos ou mapa por triângulo no frame. Registrar memória lógica/preparo.
3. Tipos: binários versionados LE, f32×6 por vértice, u32 por índice/material, certificados f32 arredondados para fora. Sem JSON no asset de runtime.
4. Repetição: retirar subdivisão pela geometria, não por nomes/metadados/ordem; teste independente com ordem alterada.
5. Escala: compilação pode crescer comT; render deve depender das regiões certificadas. Fallback explícito se não reduzir ou se qualidade/projeção não certificar.
6. GPU/I/O: upload uma vez; sem busca porpixel, compute, leitura de imagem ou upload de geometria por frame.
7. Sincronização: medições em lotes alternados com duplicatas; GPU por janela de lote, sem somar passes sobrepostos.
8. Invalidação: fonte/hash/layout/checksum versionados; consumidor só lê artefato compilado, material compartilhado pelo shader.
9. Correção: topologia, winding, materiais, bordas compartilhadas, bound geométrico e campo normal; corrupção/truncamento, ordem embaralhada, dobra e fallback.
10. Custos mantidos: fonte residente para fallback/medição, compilaçãoCPU e regiões que não reduzem. Sem mudanças em NPC/Scene/Command/MCP ou promessa de assetsAAA arbitrários.

## Implementação e fronteira do certificado

`examples/geometry_compiler_probe/compiler.rs` recebe somente `Mesh`. O comando `--generate` grava IGM1; `--compile input.igm output.igc` lê esse arquivo em outro processo e não consulta o gerador; `--run` consome somente IGC1. O arquivo de entrada não contém IDs parentais, grade, fórmula de relevo, multiplicador ou hierarquia. São fixtures procedurais controladas, não scans importados.

O compilador solda apenas posições **e normais** bit a bit iguais, constrói adjacências orientadas e impede crescimento através de materiais distintos. Uma região precisa caber no plano e no campo afim de normais do seu triângulo semente. Tolerâncias de descoberta: posição0,00002, normal0,00005; cadeia de borda com desvio acumulado≤0,000001 nas unidades desta fixture. Valores absolutos fazem parte deste experimento, não de uma configuração universal da engine.

Antes da redução, verifica ciclo de borda único, característica de Euler de disco, orientação, contenção convexa projetada e conservação de área dentro da tolerância. A triangulação de orelhas evita leques degenerados em bordas com vértices colineares. Uma região rejeitada conserva todos os seus vértices de fronteira; regiões vizinhas retriangulam o interior respeitando esses vértices. A rejeição de uma região não obriga todo o componente a voltar à fonte. Testes verificam arestas internas pareadas na superfície aberta e fechamento/orientação na pedra.

Para regiões aceitas, o registro contém AABB, limite de deslocamento `2 × residualPlano + tolerânciaBorda + margemF32`, limite do campo de normais `2 × residualNormal`, contagens, material e estado. Arredondamento dos limites para fora; margem numérica de16 epsilons f32 vezes a escala da AABB. Regiões nativas têm erro zero. Essa construção é um certificado sob as hipóteses geométricas/numericas verificadas, **não uma prova formal com aritmética intervalar para qualquer entrada adversarial**. Normais não são renormalizadas durante a subdivisão; a normalização ocorre no mesmo shader em ambos os caminhos.

Na inicialização, agrega os limites regionais. Por frame, projeta os oito cantos da AABB global; com erro `e`, profundidade mínima `z`, raio transversal máximo `r` e focal em pixels `f`, usa `f × e × (1 + r/z) / (z-e)`. Exige domínio à frente do near plane, limite≤0,15pixel, limite de normal≤0,000101 e redução real de contagem. Caso contrário, usa a fonte. Nenhuma busca por pixel, consulta de adjacência, seleção de milhares de regiões ou upload de geometria no frame. Este recorte gera regiões offline e um nó de seleção global; ainda não é uma hierarquia de LOD com seleção regional em runtime.

## Dados e custos conscientemente mantidos

IGM1/IGC1 são little-endian, versão1, com tamanho e checksum FNV-1a64. O checksum detecta corrupção acidental; não autentica o asset. Vértices:24B (`f32` posição3 + normal3); faces de arquivo:16B (`u32` índices3 + material). IGC1 inclui hash de proveniência, flag de geometria compartilhada, fonte para fallback, geometria reduzida quando existe e certificados48B. Se nenhuma região reduz, não grava uma segunda malha. Consumidor usa índicesGPU12B/triângulo e até três draws por material; não faz upload duplicado se o caminho compilado não reduz. JSON existe apenas em relatórios, nunca como malha de runtime.

Fonte integral permanece residente para fallback e comparação. Assim, reduzir drasticamente o trabalho de render **não reduz automaticamente a memória total residente deste harness**. Bytes de geometria reduzida, bytes totais dos buffersGPU, tamanho do arquivo com fallback e picoRSS da compilação são métricas distintas. Decodificação ainda aloca vetores e faz validação completa no carregamento. A representação ainda não implementa streaming, paginação nem descarte seguro da fonte.

## Medição reproduzível

```sh
python3 scripts/verify_geometry_compiler.py
target/release/examples/geometry_compiler_probe --measure artifacts/appearance-probe/geometry-compiler
python3 scripts/geometry_compiler_report.py
cargo test --locked --release --all-targets
cargo clippy --locked --release --all-targets -- -D warnings
python3 scripts/run_verifications.py --only render replay_inspection selection
```

Saída corrente: `artifacts/appearance-probe/geometry-compiler/`, incluindo IGM1/IGC1, metadados de compilação, `results.json`, `summary.json`, PNGs e `index.html`. `--smoke` no verificador usa malhas menores; não substituir seus números pelos da coleta integral. Um único conjunto corrente, sobrescrito durante repetição.

Duas famílias, quatro multiplicadores, duas distâncias e duas resoluções físicas1728×1117/3456×2234. A qualidade compara RGBA32F linear sem resize, com nove poses por caso (oito orientações e retorno exato); materiais, luz e renderer iguais. Métricas na união dos pixels cobertos: RMSE RGB, p99 dos erros absolutos de canais e divergência de cobertura. Valores máximos por canal também ficam no relatório. A aceitação é numérica dentro de limites, não igualdade de todos os pixels nem aceite humano.

Performance final:128frames por lote (32poses repetidas quatro vezes), oito rodadas, dois controles duplicados por caminho em cada rodada; ordem fonte/compilado alternada. Câmera e luz mudam a cada frame. GPU usa a janela do primeiro início ao último fim, incluindo todos os draws do lote; não soma timestamps sobrepostos. CPU registra seleção, uniforms, encoding e submit; parede por frame inclui uploads de uniforms, espera e leitura de timestamps. Readback de imagens, PNG e compilação ficam fora do timing. A métricaGPU exclui cópias de uniforms anteriores ao primeiro timestamp; a parede inclui essa preparação. Nenhum desses números é FPS de um jogo completo. Vertex/index buffers de ambas as variantes recebem a mesma ordenação espacial simples por IDs canônicos antes de upload, evitando punir a fonte apenas pelo embaralhamento do teste.

O visualizador mostra capturas, não um renderer interativo. Em escala física usa larguraCSS=larguraCapturada/DPR; modo ajustado à janela é explicitamente identificado como redução. O painel permite comparar objeto, perto/longe, resolução, orientação/luz e tabela1×–100×.

## Revisão estática posterior

1. **Frequência:** `compile` concentra adjacência, regiões e triangulação no build; `Bound::choose` é constante no frame.
2. **Alocação:** sem vetores/mapas no seletor; mapas, ordenações, cópias de binários e validação mantidos no build/load. PicoRSS medido com `/usr/bin/time -l`.
3. **Tipos:** payloads tipados LE; normas/posições f32, predicados offline f64; sem parsing textual de geometria.
4. **Repetição:** shader, renderer, materiais e ordem de índices comuns; parâmetros de três materiais enviados por frame, custo incluído.
5. **Escala:** construção cresce com triângulos de entrada; runtime depende da malha certificada. Falha de redução preserva fonte, não promete platô em toda malha.
6. **GPU/I/O:** vertex/index upload uma vez; até três draws, sem compute, texture lookup de símbolo ou readback no caminho normal.
7. **Sincronização:** único encoder por lote, timestamp de janela e parede medidos separadamente; captura/benchmark separados da regressão MCP.
8. **Invalidação:** versão/layout/checksum e limites conferidos no decode; alteração autoral exige recompilar. Fonte torna fallback independente do gerador.
9. **Correção:** testes de relevo real, normal não afim, material, costuras, orientação/fechamento, entrada inválida, ordem embaralhada, corrupção e projeção; imagens e retornos complementam a geometria.
10. **Custos mantidos:** fonte residente, loadCPU e sorting offline, shader/material convencional, nenhuma nova dependência ou contrato funcional. Não há economia gratuita de iluminação nem certificado universal.


## Resultado medido — 27/10/2026

Apple M4 Pro / Metal, câmera perto,3456×2234, mesma pedra fechada/material/luz. Os multiplicadores abaixo são de **subdivisão**, não de triângulos por pixel.

| Fonte | Triângulos fonte | Triângulos opcionais | GPU fonte (ms) | GPU opcional (ms) |
|---|---:|---:|---:|---:|
| 1× | 49.152 | 49.152 | 0,24824 | 0,24333 |
| 2× | 98.304 | 49.152 | 0,28136 | 0,23945 |
| 10× | 491.520 | 49.152 | 0,69419 | 0,23501 |
| 100× | 4.915.200 | 49.152 | 6,86199 | 0,23924 |

No caso100× perto:28,68× na janelaGPU (96,51% menos tempo), parede6,93122→0,27892ms por frame em lote. Longe:2,49918→0,18430ms GPU (13,56×), parede2,54283→0,22492ms. O caminho opcional usa a própria fonte em1×; pequenas diferenças entre esses controles não são otimização. As medianas compiladas da pedra perto variam cerca de3,5% entre o mínimo e máximo, enquanto a fonte cresce fortemente.

Controle de dobras100×:819.200→9.145triângulos,0,54249→0,18838ms perto e0,31632→0,15499ms longe (2,88×/2,04×). A primeira proteção de fronteiras fazia falhas locais contagiarem quase o objeto inteiro; o reparo retém vértices apenas na costura com a região nativa. Não remove o relevo genuíno para obter os números.

Coleta inicial32frames e confirmação128frames mantidas como dados deste experimento em `results.json` e `timing-repeat.json`, respectivamente. Resumo/visualizador usam a confirmação mais recente. Duplicatas individuais na confirmação variaram0,6228–1,5293; na coleta curta,0,1603–4,7296. A mediana da pedra100× perto foi7,2160→0,2406ms inicialmente e6,8620→0,2392ms na confirmação. Isso sustenta o ganho grande e o platô deste caso; não sustenta alegações de ganhos pequenos ou ausência universal de regressões nos casos leves. Sem p-value/confiança formal ou isolamento de clocks/preempção do desktop.

Qualidade:288comparações/864gates passaram; RMSEmáximo0,00152995, p99máximo0,000343263, cobertura divergente máxima0,000050822 (0,0050822%).32retornos reproduziram RGBA32F byte a byte para fonte e opcional. Erro projetado certificado máximo0,118811px. Houve diferenças em pixels; não é renderização pixel-idêntica. Capturas em resolução integral, sem resize na métrica; inspeção visual das imagens da pedra complementou essas métricas, mas não substitui aceite humano.320estados do visualizador verificados em DOM simulado, incluindoDPR1/1,5/2/3; não foi validação de navegador nativo.

Na pedra100× nativa, a razão **triângulos totais da fonte/pixels cobertos** foi1,731–1,794 perto e18,709–19,315 longe. Esse numerador inclui faces ocultas; não é contagem de triângulos visíveis. Portanto este teste não demonstra100triângulos visíveis por pixel perto nem substitui o teste anterior de fontes densas com genealogia conhecida.

Memória da pedra100×: geometria reduzidaGPU1,125MiB versus fonte112,5MiB; ambas residentes neste harness, total113,625MiB de buffers de geometria. CertificadosCPU/arquivo2,25MiB; IGC1 com fonte de fallback134,813MiB. Geometria reduzida em arquivo1,313MiB, pois faces incluem material. Build+decode/validação/encode/escrita4,913s; picoRSS do processo980,44MiB. Esses valores não são VRAM física nem ganho de memória total do jogo. A versão1× compartilha geometria no arquivo e não duplica upload. Fonte/dadosCPU ainda são mantidos pelo harness durante a comparação; streaming e liberação após upload são próximos problemas separados.

Validação final:660testesRust release/all-targets, Clippy-Dwarnings, fmt/diff;13gruposMCP headless em render/replay_inspection/selection. Nenhuma mudança em contratos funcionais, NPCs, Scene/Command/RenderFrame ou demo aberta. Experimento isolado, não integração de renderer na engine nem aceite humano.

### Continuação útil

O compilador já reconhece redundância de subdivisão em uma superfície fechada sem receber sua genealogia. Próximo recorte: entrada de malha externa com triangulação irregular e geometria não coplanar, medir quanto continua certificável; então construir níveis regionais com erro de superfície/normal e preservação explícita de UVs/material. Usar estas fixtures como controles de não regressão. Não extrapolar o platô para triângulos que acrescentam detalhe realmente visível, animação, sombras/GI ou cenasAAA completas.
