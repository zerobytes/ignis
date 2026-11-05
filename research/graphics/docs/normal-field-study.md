# Campo de normais compilado por região — EAF-88

## Hipótese e checklist antes da implementação

Continuação de `irregular-lod-study.md`: conservar o campo de normais da fonte independentemente da triangulação reduzida. Um atlas planar por região é calculado no build, com bordas preenchidas pela amostra mais próxima da fonte. O runtime consulta o campo, normaliza e aplica a mesma luz/material diagnóstico. Não contém cor com iluminação fixa. A forma, profundidade e silhueta continuam na malha.

Alternativas: campo exato por busca de triângulos repõe custo por pixel; normal por vértice reduzido perde a variação interior já observada. Atlas tem acesso direto e filtragem de hardware, mas introduz quantização, resolução finita, memória e deslocamento de amostra quando a superfície muda. O experimento compara estes erros; não presume solução nem altera contratos da engine.

1. Build: projeção das regiões e amostragem somente offline; pacote binário versionado.
2. Frame: reutilizar seleção por erro geométrico e proteção de silhueta, sem consultar métricas de qualidade/câmera pré-gravadas.
3. Dados: RGBA8Snorm, 4 bytes/texel, normal XYZ e máscara de validade A; filtro bilinear. Posições/normais fonte f32, UV f32 e índices u32 neste laboratório. Medir custo antes de tornar o formato contrato.
4. Compartilhamento: deduplicar vértices por região/UV; armazenar apenas regiões com nível reduzido. Memória contabiliza fonte, níveis, coordenadas e atlas.
5. Escala: testar grades 32/64/128 por região; limites de projeção inalterados. Nenhum ganho100× alegado sem fonte com densidade correspondente.
6. GPU: buffers e textura residentes; apenas uniforms/ranges por frame. Comparar mesmo conjunto de triângulos com/sem campo e fonte completa.
7. Medição: capturas RGBA32F/cobertura em pixels físicos; lotes alternados, duplicatas, CPU/GPU/parede separados.
8. Invalidação: checksums/layout/comprimentos no load; fonte vinculada ao IGL1 de entrada. JSON só relatórios.
9. Correção: testar campo constante, quantização, codec e UV por região; câmera perto/longe/órbita e luz variável. Gates RMSE≤0,005,p99≤0,02,cobertura≤0,0005 permanecem; máximo exposto.
10. Custos mantidos: fallback fonte, memória adicional, shader convencional, seleção CPU. Atlas não prova materiais gerais, sombras, visibilidade interna, referência integrada, animação ou aceite humano.


## Resultado medido — 27/10/2026

O campo recuperou qualidade nos critérios deste ensaio, mas **não trouxe ganho líquido de performance**. A variante 64 passou em todas as 122 comparações. Isso viabiliza a hipótese de aparência independente da triangulação, sem demonstrar ainda que esta representação vale seu custo.

Entrada: o mesmo IGL1 de `irregular-lod-study.md`, Boulder 01/Poly Haven, 66.122 triângulos, 450 regiões. Reutilizados os níveis e os limites geométricos; sem recompilar a simplificação para favorecer o campo. A seleção candidata relaxa apenas o limite antigo de diferença de normais, pois consulta a normal da fonte no atlas; conserva limite projetado de 0,15 pixel e proteção de silhueta. As normais interpoladas da malha completa continuam sendo a referência. O atlas não contém luz nem cor final pré-calculada.

### Qualidade

Duas resoluções físicas, 1728×1117 e 3456×2234; distâncias 2,7 / 8 / 32 / 64; oito posições de órbita mais retorno; luz varia junto da sequência. Mais 25 estados de afastamento/órbita/retorno em cada resolução. São 122 comparações por variante, incluindo retornos repetidos, e 366 no total. Os 24 retornos de órbita e seis retornos de trajetória tiveram hashes RGBA32F idênticos aos respectivos inícios tanto na referência quanto no campo. Isso verifica determinismo, não igualdade entre referência e candidato.

| Campo por região | Falhas sem campo | Falhas com campo | RMSE máximo | p99 máximo | Erro máximo em um canal |
|---|---:|---:|---:|---:|---:|
| 32 | 30/122 | 1/122 | 0.005061 | 0.016237 | 0.165534 |
| 64 | 30/122 | 0/122 | 0.002733 | 0.007690 | 0.083448 |
| 128 | 30/122 | 0/122 | 0.001047 | 0.003343 | 0.042378 |

Todos os candidatos mantiveram a mesma cobertura de pixels da referência nas capturas. Os 122 controles conservadores por variante ficaram pixel-idênticos. O controle sem campo usa exatamente os mesmos triângulos do candidato com campo; portanto a melhoria de cor medida vem da mudança de leitura das normais, não de selecionar mais geometria.

A grade 32 falhou em 1728×1117, distância 64, órbita 135°: RMSE 0,0050606, acima de 0,005. A grade 64 teve RMSE máximo 0,0027325, mas erro máximo de canal 0,0834484; **não são pixels idênticos nem prova de que todas as diferenças sejam imperceptíveis**. Mantidos p99≤0,02 e cobertura≤0,0005; picos locais expostos no painel. Aumentar a resolução do campo reduz o erro, com forte aumento de memória.

Amostragem offline: cada região usa o plano do certificado geométrico, interpola baricentricamente o campo da fonte dentro dos triângulos e usa o ponto mais próximo nas bordas para preencher o exterior/gutter. No runtime, UVs são interpoladas na superfície simplificada. O deslocamento da superfície muda onde o raio encontra o campo; quantização e filtro bilinear também introduzem erro. Não foi construído um certificado formal de erro fotométrico do atlas. Grades 32/64/128 são alternativas de build, não uma hierarquia de mipmaps selecionada em runtime. O teste ainda usa um único material diagnóstico, sem sombras, UVs/texturas originais, normal map original ou referência integrada supersampled.

### Memória e compilação

INF1 é um envelope binário little-endian com versão, tamanho e checksum. Contém o IGL1 de geometria/regiões, coordenadas UV e atlas RGBA8Snorm. Arquivo é decodificado uma vez; buffers e textura são enviados uma vez. O frame atualiza uniforms e escolhe ranges, sem JSON, busca da fonte por pixel, compilação ou upload de geometria.

Os três primeiros canais guardam a normal antes da normalização por pixel, quantizada de −1 a 1 em 8 bits com sinal. O canal A guarda validade para fallback. Há uma borda de uma amostra por tile e slots vazios necessários ao atlas retangular. O shader usa uma consulta bilinear de nível zero somente para vértices de níveis reduzidos, seguida de normalização e a mesma iluminação dinâmica. RGB não é cor nesta textura.

| Campo | Atlas GPU | Geometria + UV + atlas GPU | Arquivo completo |
|---|---:|---:|---:|
| 32 | 2.037 MiB | 5.658 MiB | 6.047 MiB |
| 64 | 7.677 MiB | 11.297 MiB | 11.687 MiB |
| 128 | 29.784 MiB | 33.405 MiB | 33.794 MiB |

Fonte isolada do estudo anterior: 2,176 MiB de buffers. Fonte+níveis anteriores: 2,876 MiB. Geometria+UV do campo: 3,620 MiB (83.597 vértices, incluindo separações por região; 8 bytes de UV por vértice). As variantes são carregadas e medidas uma de cada vez; não somar seus tamanhos como requisito do jogo. Memória exclui targets RGBA32F/depth, staging, readback, cópias CPU, metadados em RAM e driver. Pico RSS não medido. O Mac usa memória unificada; isso não torna essas alocações gratuitas.

Build + roundtrip + escrita observados: aproximadamente 0,395 / 1,112 / 3,727 segundos para 32 / 64 / 128. Esses tempos excluem download e a compilação geométrica anterior. Dados são preparados offline, mas o layout ainda é experimental: UV f32, índices u32, fonte residente e fonte/normais de fallback. Compactação adicional, tiles variáveis, mipmaps e streaming não implementados.

### Performance: comparação pareada no mesmo harness

Apple M4 Pro/Metal. Quatro caminhos: fonte completa, anterior conservador, seleção geométrica sem campo, a mesma seleção com campo. Todos usam o mesmo buffer geométrico residente nesta rodada; fonte e controles sem campo não ligam o pipeline de textura. Semântica/material da referência mantidos. O renderer anterior continua disponível sem atlas.

128 frames por lote, 32 poses por distância; aquecimento por caminho e oito rodadas alternadas/palindrômicas com duas duplicatas por caminho/rodada, 16 lotes por caminho/caso. Tempo GPU é a janela dos timestamps; CPU inclui seleção, codificação, uniforms e submissão; parede inclui espera/readback do lote. Esses números não são FPS de gameplay. Não comparar valores absolutos desta rodada com a rodada anterior em outra condição térmica; usar os pares abaixo.

Grade 64, que passou em todos os casos medidos:

| Resolução / distância | GPU anterior→campo (ms) | CPU anterior→campo (ms) | Parede anterior→campo (ms) |
|---|---:|---:|---:|
| 1728 / 2.7 | 0.08160→0.08157 | 0.03093→0.03780 | 0.13003→0.13135 |
| 1728 / 8.0 | 0.07691→0.07536 | 0.02903→0.03744 | 0.12017→0.12774 |
| 1728 / 32.0 | 0.05664→0.05775 | 0.03073→0.04439 | 0.10009→0.10840 |
| 1728 / 64.0 | 0.04608→0.04666 | 0.02979→0.04498 | 0.09043→0.10220 |
| 3456 / 2.7 | 0.22699→0.23102 | 0.03093→0.03744 | 0.26895→0.28197 |
| 3456 / 8.0 | 0.20159→0.20101 | 0.03018→0.03700 | 0.24097→0.25163 |
| 3456 / 32.0 | 0.16788→0.17086 | 0.03073→0.04311 | 0.20912→0.22686 |
| 3456 / 64.0 | 0.15494→0.14988 | 0.02999→0.04545 | 0.19496→0.20531 |

Parede ficou 1,0–13,0% maior nas medianas desta grade. Dispersão mediana dos pares duplicados de parede: aproximadamente 1,3–9,1% nessas coortes. Não tratar diferenças pequenas como regressões estatisticamente estabelecidas; não apareceu ganho líquido demonstrado. O aumento de CPU é consistente e relevante frente à economia pequena de triângulos. As métricas de todos os quatro caminhos e três grades ficam em `results.json` e `summary.json`.

### Onde está o limite agora

- A seleção candidata desenhou no mínimo 64.314 triângulos, contra 66.122 na fonte: economia máxima de **2,734%**. O campo sozinho não produz uma malha mais reduzida.
- O seletor chegou a **193 desenhos** para uma pedra, mesmo unindo ranges adjacentes. O custo inclui a seleção de 450 regiões e submissão desses desenhos.
- No pacote geométrico, 30.966 triângulos (46,83% da fonte) não têm redução. As 449 regiões com alternativas têm mediana de apenas 24 triângulos; fronteiras fixas e cones conservadores restringem reduções adicionais. Mesmo forçando todos os níveis mínimos, haveria 56.018 triângulos (−15,28%), mas isso já falhou na qualidade e não é resultado aceito.
- Quando perto, a fonte tem densidade total de apenas 0,128 ou 0,032 triângulo por pixel coberto, incluindo faces ocultas. Este asset próximo não é prova do platô de microtriângulos. O resultado anterior com subdivisão redundante permanece um teste distinto.

A decisão é manter esta representação no laboratório. A qualidade da resposta à luz melhorou, mas o benefício geométrico é insuficiente para pagar atlas e seleção. Próximo recorte recomendado: melhorar o particionamento/simplificação e a representação de fronteiras, verificando primeiro quanto se consegue reduzir sob o mesmo limite geométrico; depois comparar submissão agrupada. Encolher o atlas sem resolver a redução de apenas 2,73% provavelmente não basta. Um certificado do erro do próprio campo também será necessário antes de ativá-lo automaticamente para assets/câmeras arbitrários. Nenhuma integração ou nova arquitetura de runtime foi decidida aqui.

## Checklist posterior e validação

1. Build em `field.rs`: projeção, interpolação e padding offline; duas granularidades de malha continuam no IGL1, campo em INF1.
2. Frame: seletor anterior reutilizado, mesmos gates geométricos; limite de normais relaxado apenas no candidato explicitamente experimental. Nenhum resultado de qualidade ou tabela de poses no seletor.
3. Tipos: normal RGBA8Snorm filtrável, UV f32; payload tipado e arrays contíguos. Alpha usado para validade; padding e atlas retangular são custos declarados, não compressão ótima.
4. Compartilhamento: vértices fonte deduplicados, candidatos deduplicados por região. UV adicional e fronteiras duplicadas contabilizadas.
5. Escala: grades limitadas a 32/64/128, textura até 4096 em cada dimensão no decoder; sem hierarquia de amostragem ou streaming.
6. GPU: extensão opt-in no harness compartilhado, não em `src/render.rs`; fonte usa pipeline original, candidato tem textura/UV adicionais. Readback só no experimento.
7. Medição: benchmark concluído antes da regressão GPU/MCP, sem janela nem controle da demo. Alocações temporárias do seletor e draws são contabilizados.
8. Invalidação: INF1 valida versão/checksum/tamanho/UV e decodifica IGL1 validado. Entrada é a saída confiável do compilador, não formato público adversarial.
9. Correção: 670 testes Rust release/all-targets, Clippy `-D warnings`, fmt (incluindo arquivo de extensão GPU), 13 grupos MCP headless (render/replay_inspection/selection) passaram. Dois testes novos de interpolação, padding, quantização, UV, roundtrip/truncamento/corrupção; quatro testes regionais compartilhados também executados no novo exemplo. Painel: 768 estados DOM/DPR simulados, imagens presentes; PNG inspecionado. Sem validação de navegador nativo ou aceite humano.
10. Custos mantidos: textura, UV, fonte residente, certificado conservador, até 193 draws; sem alegação de economia de RAM total, FPS ou qualidade universal. Contratos funcionais de NPC/Scene/Command/MCP intactos.

## Reprodução

Preparar o IGL1 conforme `irregular-lod-study.md`, então na worktree `codex/graphics-research`:

```sh
cargo build --locked --release --example normal_field_probe
mkdir -p artifacts/appearance-probe/normal-field
for size in 32 64 128; do
  target/release/examples/normal_field_probe --build artifacts/appearance-probe/irregular-lod/boulder.igl artifacts/appearance-probe/normal-field/boulder-$size.inf $size
done
target/release/examples/normal_field_probe --run artifacts/appearance-probe/normal-field artifacts/appearance-probe/normal-field/boulder-32.inf artifacts/appearance-probe/normal-field/boulder-64.inf artifacts/appearance-probe/normal-field/boulder-128.inf
python3 scripts/report_normal_field.py
node scripts/check_normal_field_ui.mjs
```

O comando de medição falha se o caminho conservador regredir. Falhas dos candidatos investigativos são resultados explícitos, não erro de transporte: consultar os campos `pass`/`failures`. O painel local `artifacts/appearance-probe/normal-field/index.html` compara capturas e tabelas, não executa o renderer. Escala nominal por DPR disponível além da visão geral; não é leitura da resolução atual da tela do usuário. Artefatos em caminho estável, ignorados pelo Git; os três pacotes são variantes necessárias à comparação corrente, não versões históricas.
