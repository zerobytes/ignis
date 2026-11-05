# Curva de densidade 1×/2×/10×/100× — protocolo

EAF-84. Experimento isolado: medir custo versus triângulos fonte, sem mudar contratos da engine funcional. Duas famílias de heightfields opacos: plano com tesselação redundante e superfície com relevo real, progressivamente mais fino. Fonte32.768/65.536/327.680/3.276.800triângulos, resolução física3456×2234 e1728×1117, mesma trajetória por densidade.

## Revisão antes de implementar

1. Preparar meshes/níveis/erros/normais fora do frame; runtime percorre somente metadados dos níveis.
2. Buffers e targets persistentes; upload e alocação de assets fora das medidas de regime. Registrar memória e preparação separadas.
3. Grids aninhadas por subdivisão igual nos dois eixos. Comparar erros máximos entre fonte e nível nos vértices fonte: diferença de campos afins atinge extremos nesses vértices quando triangulações são aninhadas. Normais armazenadas antes da normalização por fragmento.
4. Referência completa e candidatos usam material/luz/câmera iguais. Um draw por superfície em todos os modos; mesma preparação/seleção incluída no tempo total.
5. Quatro densidades exatas, um asset visível. Não extrapolar para muitos assets/streaming/árvores/oclusão.
6. Comparar full, LOD por erro de posição, LOD com guarda de normal e LOD com textura da normal fonte. Textura consultada por interpolação triangular exata, três fetches por fragmento; sem mip nesta primeira prova para manter a referência de amostra única. Custo e memória maiores ficam explícitos.
7. Timestamps do passe e CPU+GPU serial/vazão separados; sem readback/captura na medida. Controles duplicados e ordem rotacionada.
8. Seleção depende da projeção/resolução/erro preparado, nunca do multiplicador nominal nem de resultado de benchmark. Câmera perto/longe/volta; níveis podem mudar discretamente, medir erro temporal.
9. Qualidade versus full de cada densidade: RMSE foreground, p99 de maior diferença RGB por pixel, cobertura divergente, máximo e pixels alterados. Metas experimentais prévias: RMSE≤0,005 linear, p99≤0,02, cobertura divergente≤0,05% do foreground. Não equivalem a aceite perceptual humano. Nunca relaxar metas para declarar sucesso; documentar falhas.
10. Custos mantidos: geometria e normal source crescem com densidade, build/storage/memória não são constantes. Sem sombras/GI/transparência/deformação; materiais difusos/rugosos e superfície parametrizada não provam solução geral. Normal mapeada não muda silhueta; erro geométrico ainda é limitado em pixels.

A hipótese central é custo de regime quase independente da densidade fonte, com fidelidade preservada. Subdivisão plana é somente controle. Na família detalhada, as versões densas acrescentam bandas de relevo; cada candidato compara com sua própria referência completa, e não com a imagem1×. O teste não pretende garantir pixels numericamente idênticos ao simplificar geometria.

## Construção e seleção

`triangle_scaling_build.rs` gera grids128×128,128×256,256×640 e1280×1280, respectivamente32.768,65.536,327.680 e3.276.800triângulos. Cada superfície ocupa12×12unidades. O plano tem altura zero. O relevo combina senoides largas com bandas adicionais de frequência maior e amplitude menor conforme a densidade cresce. Normais analíticas unitárias são amostradas nos vértices fonte e interpoladas pelo rasterizador; não há textura de cor ou rugosidade variável escondendo diferenças.

São preparados oito níveis de triangulação, com stride128,64,32,16,8,4,2,1. O mesmo stride nos dois eixos mantém diagonais aninhadas, inclusive nas grids retangulares. O bake mede a maior diferença de altura e de vetor normal contra a fonte em todos os vértices fonte. Esses campos são afins em cada triângulo da subdivisão, por isso extremos estão nos vértices; o teste de interpolação protege a diagonal contra substituição indevida por bilinear. O bound da normal é anterior à normalização pelo shader, não um bound formal de cor para qualquer material.

O asset cabe numa esfera conservadora de raio9. Para a câmera a distância `d`, `z_min=d−9`, focal física `f=altura/(2*tan(FOV/2))`, o seletor usa `2*f*erro/z_min*(1+9/z_min)`. Se o bound cruza o plano próximo, conserva a fonte. É deliberadamente conservador e global por superfície; um relevo difícil pode manter o nível fino no asset todo. O primeiro nível que atende0,15px é escolhido; a variante com guarda exige ainda erro vetorial de normal≤0,005. As escolhas não usam densidade nominal, tempo medido ou resultados de qualidade.

O quarto caminho usa a mesma geometria escolhida pelo erro de posição, mas consulta a normal da fonte numa textura RGBA32F. A coordenada XZ do fragmento localiza seu triângulo fonte e três texels reproduzem a interpolação triangular do campo. Não há loops por triângulo fonte no fragmento, mip ou filtragem adicional. A geometria simplificada ainda pode deslocar a posição amostrada, alterar silhueta e perder auto-oclusão; a textura não resolve isso. Quando o nível escolhido é a própria fonte, a comparação valida a consulta em textura contra a interpolação nativa, com diferença residual de ponto flutuante.

Somente CPU percorre oito metadados por seleção. Upload/geração dos assets ocorrem antes da medição. Um draw por superfície em todos os caminhos, mesma shader com ramo uniforme para consultar a textura, mesmo material difuso+especular de expoente24 e mesma luz. Não é uma nova arquitetura integrada à engine; a parametrização explícita de heightfield torna essa prova mais simples que uma malha arbitrária.

## Medição e limites de interpretação

- Adapter Apple M4 Pro/Metal.144poses: duas famílias × quatro densidades × duas resoluções × nove passos. Distâncias14,28,56,112,224,112,56,28,14, mantendo orientação. Câmera/luz/resolução iguais entre fonte e candidatos.
-432comparações foreground entre completa e três candidatos.576PNGs no tamanho físico, sem reduzir a resolução para comparar. Ida/volta precisa reproduzir os mesmos pixels por representação. A curva de erros não descarta as falhas.
- A razão “triângulos fonte/pixels cobertos” mede toda a fonte dividida pela cobertura do objeto, não quantos triângulos ganharam cada amostra. Mesmo acima de1, algumas arestas/detalhes podem ser visíveis. Área total da tela não é substituto de cobertura local.
-80poses iniciais cronometradas (cinco distâncias únicas).12ciclos de aquecimento e12rodadas de oito slots: quatro caminhos duplicados, ordem rotacionada. Medianas dos controles combinadas por média. GPU por timestamps do passe; ciclo serial e preparação/codificaçãoCPU separados. Vazão inicial usa blocos de oito frames na mesma pose, refazendo câmera/seleção/upload/codificação/submissão.
- Medições iniciais mostraram variação e possível peso da espera em blocos curtos. A confirmação carrega as quatro densidades da família simultaneamente e alterna32slots (quatro densidades × quatro caminhos × dois controles),12rodadas, blocos de128frames. Sete coortes: plano na resolução maior a14/112; relevo na maior a14/112 e na menor a14/56/112. As demais distâncias continuam explicitamente identificadas como rodada inicial. Sem compilação/testes concorrentes durante benchmarks; desktop externo não controlado.
- `report.json` conserva a primeira rodada. `confirmation.json` conserva todos os controles novos; o relatório/summary usam os novos tempos apenas nos pontos confirmados. Capturas/erros não são alterados. A comparação entre densidades usa controles sob condições mais próximas, mas não é intervalo de confiança estatístico nem prova assintótica.
- A razão100×/1× é insuficiente sozinha: publicar também maior/menor custo entre os quatro pontos, controles duplicados e todos os valores. Uma curva com pico em10× não se torna plana porque suas extremidades coincidem.
- O residual temporal é a diferença entre os erros RGB de frames consecutivos nas mesmas coordenadas de tela, sobre o foreground de qualquer um dos dois. Não usa reprojeção e não isola cintilação de borda de movimento legítimo; é diagnóstico complementar. Trocas discretas de LOD não têm geomorph/histerese nesta rodada.
- Referência de amostra única, sem MSAA/TAA ou integração supersampled. A normal em textura conserva inclusive aliasing da fonte; reproduzir a referência não demonstra a média física correta. Iluminação e material fixos; novos ângulos/luzes/materiais precisam de outros testes.

## Reprodução

Na worktree `codex/graphics-research`:

```sh
cargo run --locked --release --example appearance_probe -- --bake-triangle-scaling
cargo run --locked --release --example appearance_probe -- --triangle-scaling
cargo run --locked --release --example appearance_probe -- --confirm-triangle-scaling
python3 scripts/check_triangle_scaling.py
python3 scripts/report_triangle_scaling.py
node scripts/verify_triangle_scaling.mjs
```

Artefatos em `artifacts/appearance-probe/triangle-scaling/`. O relatório controla família, densidade, resolução, caminho, métrica e câmera, com curvas e recortes512² que compensam DPR. Imagens gerais são reduzidas só para caber na página; reprodução offline não é engine rodando no navegador. Automação nativa file:// continuou indisponível, sem contorno; harness de DOM e inspeção de PNGs não substituem aceite humano. O build corrente permanece na localização externa existente, sem nova worktree ou versão instalada.

## Resultado confirmado

A confirmação conjunta **não sustenta custo constante em toda a curva1×→100×** para relevo real. Encontrou um platô parcial nas densidades altas e distâncias subpixel. Os blocos iniciais de oito frames sugeriam diferenças menores; a confirmação de128frames alterou a conclusão. A evidência abaixo usa exclusivamente os tempos confirmados.

Custo de vazão CPU+GPU, em ms/frame, para relevo a distância112, com normais em textura:

| Resolução | 1× | 2× | 10× | 100× | 100×/1× | Maior/menor da curva | 100×/10× |
|---|---:|---:|---:|---:|---:|---:|---:|
| 3456×2234 | 0.219 | 0.237 | 0.295 | 0.307 | 1.401× | 1.401× | 1.039× |
| 1728×1117 | 0.073 | 0.089 | 0.095 | 0.093 | 1.263× | 1.297× | 0.974× |

Entre10× e100×, a variação é+3,9%/−2,6%, compatível com um platô dentro da dispersão dos controles; não prova diferença matematicamente nula. Todas as quatro densidades desses dois pontos passaram nas metas de qualidade. Na fonte100×, a referência completa custou1,081ms/0,867ms, contra0,307ms/0,093ms da seleção com textura: redução local de71,6%/89,3%. A fonte100× tem3,2768milhões de triângulos; o desenho escolhido tem204.800/51.200. Custo ainda não é estritamente proporcional só ao número de pixels.

O intervalo GPU isolado na mesma comparação1×→100× cresceu54%/47%, apesar do crescimento menor de40%/26% na vazão total. Nos quatro pontos a maior/menor medida GPU foi1,55×/1,52×. Isso impede declarar independência geral da geometria, mesmo quando CPU/submissão escondem parte da diferença. Entre10× e100×, porém, GPU também ficou próxima:0,2295→0,2284ms e0,0681→0,0656ms.

Na distância56 da resolução menor, a confirmação de128frames encontrou crescimento de76% na vazão1×→100× (e pico maior em10×). Essa posição parecia praticamente constante na medição inicial e **não foi confirmada como tal**. Perto, distância14, o seletor conserva a fonte inteira e o caminho com textura cresce9,1×/15,3×. Não afirmar que todo triângulo mantido era necessário: o bound global é conservador e ainda não há seleção local por regiões neste teste.

No plano redundante, a confirmação encontrou apenas0,8%/8,7% de crescimento de vazão entre1× e100× nas distâncias112/14 da resolução maior. Essa família controla a capacidade de eliminar subdivisão inútil, mas não valida aparência complexa. A diferença entre famílias é o resultado científico relevante.

### Fidelidade e falhas preservadas

O caminho com textura passou141/144poses. As três falhas ficam na distância224, resolução1728×1117: plano10× e relevo10×/100×. Cada uma tem um pixel de cobertura divergente; com~1.774pixels de objeto, esse pixel representa~0,056%, acima do limite0,05% definido antes do teste. RMSE e p99 nesses casos passam; o gate agregado continua falho. A causa exata entre rasterização de borda e aproximação geométrica não foi isolada para todos os casos. Não arredondar essas falhas para zero, mudar a meta ou declarar pixels idênticos.

Somente forma falhou6/144poses: além da borda do plano, há perda de orientação/iluminação no relevo. Guarda de normal passou143/144, mas conservou toda a fonte detalhada e não entregou o platô desejado. Em superfície próxima, consultar a normal fonte na textura reproduz a interpolação de referência com RMSE abaixo de1e−5; o cache não está substituindo detalhe visível por uma cor média.

### Memória, preparação e validação

| Fonte | Malhas + todos os níveis + textura residente | Bake relevo |
|---|---:|---:|
| 1× | 1.27MiB | 1.71ms |
| 2× | 2.52MiB | 2.08ms |
| 10× | 12.55MiB | 18.37ms |
| 100× | 125.15MiB | 207.54ms |

Esses bytes são payloads GPU explícitos, não memória medida do driver. A fonte100× usa25,04MiB só para o mapa de normais, além de malhas/índices/níveis; o comparador deixa tudo residente mesmo nos modos sem consulta. Textura tem três componentes úteis em RGBA32F, sem compressão/mips/streaming. Alvos RGBA32F+Depth32F:147,26/36,82MiB; readback/cópiasCPU, buffers de timestamps, allocator e driver separados. A confirmação mantém as quatro fontes da família residentes ao mesmo tempo. O tempo de bake inclui geração/cálculo de erros e escrita no cache do sistema operacional, não fsync nem importação de asset externo.

622testes Rust release/all-targets, Clippy com `-D warnings`, fmt e13grupos MCP headless de render/replay/selection aprovados. Verificador offline:144poses,432comparações,80poses cronometradas iniciais, sete coortes de confirmação,576PNGs íntegros e ida/volta com pixels idênticos dentro de cada representação. Esse determinismo não significa igualdade entre representação simplificada e completa. Harness do relatório:432combinações, três métricas de curva, recortes/DPR/playback/carregamento obsoleto aprovados. Aceite visual humano e apresentação nativa pendentes.

## Revisão final

1. Frequência: geração de vértices/índices/normais e varredura de erro somente no bake. Seletor lê oito níveis por frame; custo CPU inclui câmera e escolha.
2. Alocação: buffers/textura/targets reutilizados, sem geometria reconstruída no frame. Encoder/submissão e staging interno de write_buffer permanecem. Captura/JSON/alocação de residual temporal só fora das medidas.
3. Layout: vértice24B, índiceu32, normalRGBA32F16B. Triangulação diagonal aninhada e teste afim; erro de normal antes da normalização, validado visualmente por comparação numérica e não tratado como prova de todo material.
4. Repetição: um draw por caminho. Leitura triangular da normal tem três fetches fixos por fragmento, sem consulta dos triângulos fonte. Cache de normal continua usado mesmo quando a geometria escolhida é a fonte; eliminar essa redundância é melhoria posterior.
5. Escala: exatos32.768–3.276.800triângulos, um heightfield. Oito níveis, escolha global e bounds conservadores limitam a extrapolação; sem generalização para malhas não parametrizadas ou múltiplas instâncias.
6. GPU/I/O: mesmos targets e material; medir tanto GPU quanto serial/vazão. Fonte/níveis/normais residentes crescem~100×. Platô de tempo não implica platô de memória ou de custo de build.
7. Sincronização: sem readback nas escolhas/timers. Confirmação de128frames e fontes intercaladas corrigiu conclusões otimistas de blocos curtos. CPU/submissão ainda pode mascarar crescimento de GPU; ambos publicados.
8. Invalidação: viewport físico altera projeção; metadados e binários preparados novamente pelo comando de bake quando a fonte muda. Cache experimental não é formato público versionado de asset, assinatura criptográfica ou pipeline incremental de produção. Sem morph/histerese temporal; retorno determinístico testado.
9. Correção: gates prévios preservados, inclusive falhas por um pixel; cinco novos testes de proporções/interpolação/projeção/guarda/foreground. Rust/clippy/fmt,13grupos MCP,576PNGs e432casos JS. Nenhuma alteração em Scene/Command/RenderFrame/MCP ou demo de NPCs.
10. Custos restantes: erro subpixel não garante equivalência exata, normal map sem mip pode manter aliasing, materiais/ângulos/luzes fixos, memória elevada e bound global demasiado conservador perto. Interface CLI e relatório são do laboratório; integração futura precisa dispatcher/MCP e UI correspondentes.

Próximo experimento recomendado: seleção regional usando limites de projeção mais apertados e preservação explícita de fronteiras, seguida de compressão/mips/streaming do campo de normais com critérios próprios de fidelidade. Medir se a curva inteira achata sem sacrificar a borda ou trocar qualidade por performance implicitamente. Não implementado nesta rodada.

## Continuação

[EAF-86 — regiões menores e projeção mais precisa](regional-scaling-study.md) compara o seletor global anterior, projeção pela caixa e seleção em64regiões com fronteiras fonte preservadas. O relatório e as medições de EAF-84 permanecem neste diretório; não substituir a evidência anterior pela rodada seguinte.
