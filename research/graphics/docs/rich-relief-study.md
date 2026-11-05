# Relevo real e materiais ricos — EAF-88

## Protocolo e revisão antes de implementar

Autorar um terreno mineral determinístico com alturas, sulcos e fissuras reais, veios minerais, oxidação e musgo. A referência é a mesma superfície finita com os mesmos mapas de material em todas as comparações. Não aumentar apenas subdivisão redundante. Este incremento é especializado em heightfields cartesianos; não apresentar o compilador de terreno como recuperação de qualquer malha arbitrária.

Comparar fonte completa, melhor compilador anterior com hierarquia de certificados e níveis geométricos regionais. O autor conhece a função de altura, mas a construção dos níveis/certificados recebe somente a malha amostrada e suas coordenadas de grade. Certificar erro vertical pela interseção das triangulações, incluindo cruzamentos interiores; não apenas comparar vértices. Preservar todos os vértices da borda de cada região em todos os níveis para impedir frestas. Normais e cores são campos de material precompilados e compartilhados por todas as vias, incluindo fonte; não comparar referência sem textura contra candidato com textura. Silhuetas recebem proteção pelo cone geométrico.

1. Build: geometria, mapas, mipmaps e certificados offline; nenhum gerador de ruído no shader.
2. Frame: seleção por erro projetado0,15pixel físico, proteção de silhueta; medir seleção/ranges/draws junto com render.
3. Dados: IGM1/IGC1/IGL1/ICH1 existentes e envelope binário versionado para mapasRGBA8, cores lineares/roughness, normal/modulação ambiente. JSON só diagnósticos. Todos os canais têm consumidor.
4. Reuso: mesmos mapas, iluminação e buffers entre caminhos; sampler dehardware com mipmaps. Não armazenar cor final iluminada.
5. Escala: regiões limitadas e múltiplos níveis; fonte residente, metadados/níveis/mapas adicionais explícitos. Não alegar redução de memória total.
6. GPU/I/O: mapas enviados uma vez, frame envia uniforms/seleção. CapturasRGBA32F ePNG fora dos tempos. Nenhuma janela.
7. Medição: duas resoluções, câmera/luz em órbita e aproximação/afastamento, lotes alternados e duplicados; CPU/GPU/total distintos.
8. Invalidação: checksum/versão/tamanho/mapas/codec e certificados, limites de índices, fonte imutável.
9. Correção: relevo interior, bordas de níveis mistos, amostragem/mips/codec, retorno determinístico, RMSE≤0,005,p99≤0,02,cobertura≤0,0005. Não esconder reprovações.
10. Custos mantidos: textura, seleçãoCPU/draws, fonte/níveis; nenhum PBR completo, sombra dinâmica, streaming ou integração na engine funcional. Inspeção visual suplementa métricas, sem substituí-las.

## Controle adicional após o primeiro teste

Smoke mostrou que o cone de normais geométricas veta todas as144regiões, mesmo longe. O controle conservador foi mantido como uma quarta via. A alternativa regional exploratória testa exclusivamente os certificados de deslocamento projetado e as bordas nativas compartilhadas. Ela não consulta a qualidade das capturas para escolher níveis. Preservar bordas impede frestas, mas não prova sozinho a preservação de silhuetas internas/oclusão nem erro fotométrico de texturas; por isso as reprovações de imagem serão expostas e o caminho não será promovido automaticamente. Limite projetado0,15px mantido. Este experimento distingue a limitação do veto por cone da qualidade da aproximação geométrica.

Um primeiro draw de material apresentou diferença em captura repetida com a mesma geometria. A coleta final exige aquecimento seguido de duas referências RGBA32F idênticas antes de medir, e hash idêntico no retorno de cada trajetória. Não aceitar ruído de referência como melhoria ou erro do candidato.

Smoke exploratório0,15px em864×558 passou perto, mas falhou em duas vistas distantes:RMSE0,005072/0,006503 e p990,02583/0,02671. Não foi aprovado. A coleta completa inclui um limite fixo mais rigoroso0,05px, escolhido antes da coleta final, junto do0,15px reprovado e do controle por cone. Não usar tabela de poses vencedoras nem consultar capturas durante seleção. A projeção dos oito cantos é calculada uma vez por região, reaproveitada nos quatro níveis; teste exige igualdade bit a bit com a fórmula anterior. Mapas/material continuam idênticos entre vias.

## Referência, certificados e repetibilidade

Após um abort por retorno, o diagnóstico isolou a divergência no proxy anterior em3456×2234/órbita próxima. Fonte e novos caminhos repetiram o hash10972482749392018248; anterior mudou3662606175950273817→10136201337398758470. Quatro pixelsPNG mudaram, no máximo1/255por canal. Causa ainda não estabelecida; não atribuir ao aquecimento como correção comprovada. `previous-return-diagnostic.json` mantém o achado. A coleta final continua exigindo igualdade estrita na referência completa e registra aprovação/reprovação de retorno de cada candidato separadamente; reprovações aparecem no painel e não são convertidas em aceite peloRMSE. Nenhum teste de igualdade foi relaxado.

A certificação de cada nível recebe uma segunda verificação com os XYf32 físicos usados pelaGPU, e não apenas coordenadas ideais de grade. As interseções de todas as faces da fonte com todas as faces candidatas sobrepostas precisam ficar dentro do bound armazenado (que inclui2e-6de guarda numérica). Todos os pacotes continuaram byte-idênticos após acrescentar essa validação; hashesSHA256 registrados em input-hashes.json. O tempo de build final inclui ambas as passagens. Teste com pico interior omitido exige detectar erro≥0,1, além do teste de cruzamento de diagonais e igualdade de todas as bordas.


## Dados e execução

A malha fonte tem 591.361 vértices e 1.179.648 triângulos. São 144 regiões de 64 × 64 células, com cinco níveis totais de 1.179.648 / 349.056 / 118.656 / 58.752 / 42.624 triângulos. Os níveis mantêm todos os vértices das bordas regionais; não pressupor que o último nível inteiro seja elegível em uma câmera específica.

O pacote IRM1 usa cabeçalho de 24 bytes, versão, dimensão, número de mipmaps e checksum FNV64. Cor linear RGB + rugosidade ocupam RGBA8 em 2048²; normal XYZ + modulação ambiente ocupam RGBA8 em 1024². Ambos incluem cadeia completa de mipmaps: 27.962.024 bytes (26,667 MiB). Não é um codec de textura comprimida por blocos. O shader lê todos os canais; a iluminação direcional continua sendo calculada em runtime. O teste não usa cor final pré-iluminada nem JSON como payload do renderer.

O transporte GPU usa apenas posições f32×3 (12 bytes/vértice) e índices u32. Normais/UV por vértice não são enviados: a normal está no mapa e UV deriva de XY. A mesma otimização é aplicada a todos os controles. Geometria fonte ocupa 21.252.108 bytes (20,268 MiB); fonte + todos os níveis regionais, 28.081.164 bytes (26,780 MiB). Somando mapas: 46,934 e 53,447 MiB, respectivamente. Portanto este incremento aumenta memória residente em 6,513 MiB; não demonstra compressão de memória total. Esses números excluem render targets, driver, cópias CPU e geometria extra do controle anterior mantida simultaneamente no comparador. IGM/IGL continuam guardando normais para compilação/validação: o formato de arquivo não foi declarado mínimo.

`--verify` confirma igualdade dos vértices e faces da fonte entre pacotes. O compilador anterior solda e reorganiza índices, por isso a comparação remapeia pelos seis valores exatos de posição/normal (zeros assinados equivalentes), preservando orientação/material das faces. A ordem no arquivo não é identidade geométrica. Os seis binários continuaram com SHA256 idênticos após reduzir o transporte de 24 para 12 bytes/vértice.

## Reprodução

No worktree `codex/graphics-research`:

```sh
cargo run --locked --release --example rich_relief_probe -- --build artifacts/appearance-probe/rich-relief
cargo run --locked --release --example rich_relief_probe -- --verify artifacts/appearance-probe/rich-relief
cargo run --locked --release --example rich_relief_probe -- --run artifacts/appearance-probe/rich-relief
python3 scripts/report_rich_relief.py
node scripts/check_rich_relief_ui.mjs
```

O relatório aceita a ausência de `previous-return-diagnostic.json` em uma reprodução limpa; esse arquivo registra a coleta diagnóstica anterior, separada dos retornos da coleta corrente. `--smoke` é diagnóstico em 864 × 558 e não substitui a coleta física final. Execução offscreen, sem abrir/controlar a demo. As comparações temporais usam 128 frames por lote e 32 poses, alternância palindrômica e duas amostras por caminho em cada uma das oito rodadas. CPU/codificação, janela GPU e tempo de parede completo são reportados separadamente, sem somá-los.


## Revisão estática após implementar

1. **Fases:** ruído, alturas, mapas, mipmaps e certificados em `build.rs`/`material.rs`; `experiment.rs` só carrega/seleciona/renderiza. Iluminação permanece dinâmica.
2. **Complexidade:** 144 regiões × até quatro alternativas por frame. Oito cantos projetados uma vez por região, reaproveitados. O teste `cached_projection_matches_existing_bound_bitwise` compara com a fórmula anterior.
3. **Dados:** IRM1 valida tamanho/versão/checksum/mipmaps; IGM/IGC/IGL/ICH reutilizados. GPU recebe posição de 12 bytes, índice de 4 bytes e mapas RGBA8; sem normal/UV redundante por vértice.
4. **Alocações:** vetores de escolhas/ranges por frame continuam. Capturas/sort de métricas/PNG/JSON são fora da janela medida. Sem alegar seletor livre de alocações.
5. **Tráfego:** mapas/meshes enviados no carregamento; por frame uniforms e comandos. Sem leitura de imagem para decidir nível. Texturas permanecem amostradas por pixel, custo assumido.
6. **Reuso:** fonte/candidatos compartilham mapas, sampler, iluminação e pipeline. Controle anterior e níveis ficam no mesmo buffer do comparador; memória por caminho contabilizada separadamente, com exclusões explícitas.
7. **Escala:** grade e orçamento fixos neste fixture, bordas nativas completas. A memória guarda fonte e todos os níveis; não há streaming. Ainda não resolve malha arbitrária ou geometria multicamada.
8. **Correção:** interseções entre triangulações, checagem em XY físicos, pico interior, bordas exatas, corrupção, mesma fonte remapeada. Comparação visual mede cor/cobertura; não existe certificado fotométrico/temporal geral.
9. **Regressões:** mudança compartilhada só torna opcional o stream UV quando o shader o deriva. Caminho antigo continua fornecendo UV; contratos Scene/Command/RenderFrame, NPCs e editor não mudaram.
10. **Tradeoff:** manter o veto por cone como controle e expor as alternativas sem veto, com reprovações visíveis. Não promover um candidato porque um ângulo isolado é mais rápido. Próximo avanço precisa considerar detalhe de material/oclusão e custo de seleção, além de erro geométrico.

## Resultado da coleta física final — Apple M4 Pro / Metal

138 poses/retornos em 1728 × 1117 e 3456 × 2234, 800 amostras temporais. Valores abaixo são medianas do tempo total em ms/frame de lote. Nenhuma redução consistente de tempo foi demonstrada pelo novo caminho regional contra o anterior.

| Resolução / trajetória | Fonte completa | Anterior + hierarquia | Regional 0,05 px | Regional 0,15 px |
|---|---:|---:|---:|---:|
| 1728 / perto 2,7 | 0.52083 | 0.49154 | 0.52000 | 0.51631 |
| 1728 / média 6 | 0.40999 | 0.38881 | 0.40510 | 0.42647 |
| 1728 / afastada 16 | 0.37468 | 0.32913 | 0.38412 | 0.38323 |
| 1728 / distante 64 | 0.38859 | 0.33757 | 0.39783 | 0.36994 |
| 1728 / aproximar/afastar | 0.40255 | 0.36756 | 0.40296 | 0.39633 |
| 3456 / perto 2,7 | 0.83942 | 0.74090 | 0.79890 | 0.80301 |
| 3456 / média 6 | 0.54856 | 0.52322 | 0.54975 | 0.55118 |
| 3456 / afastada 16 | 0.37207 | 0.36213 | 0.38087 | 0.37548 |
| 3456 / distante 64 | 0.37196 | 0.33866 | 0.36663 | 0.38469 |
| 3456 / aproximar/afastar | 0.40978 | 0.40383 | 0.42326 | 0.41555 |

O limite 0,05 px mantém toda a geometria em todos os casos de 3456 e nas órbitas de distância 2,7/6/16 em 1728. Longe em 1728 reduz somente duas regiões: 1.179.648 → 1.168.112 triângulos (−0,98%), cinco desenhos em vez de um. O 0,15 px chega a 1.053.632 triângulos (−10,68%) longe em 1728, mas não vence o caminho anterior e falha em repetibilidade. Não há novo platô demonstrado para relevo real.

A ordem de vértices/índices do controle anterior difere da fonte regional. Ele remove apenas quatro triângulos; a diferença de tempo não pode ser atribuída a uma grande redução geométrica. Variação mediana entre duplicatas chega a 18,27%; em várias coortes a geometria desenhada é idêntica entre controles. Não chamar pequenas diferenças de mediana de otimização. O regional 0,05 venceu o anterior em apenas 0–3 das oito rodadas por coorte.

**Qualidade:** 138/138 passam em cor/cobertura para 0,05, 0,15 e conservador. Isso não significa pixels idênticos. Em 0,05: RMSE máximo 0,000776875, p99 máximo 0,000159711, erro máximo de canal 0,038676, cobertura idêntica. Em 0,15: RMSE máximo 0,00300238, p99 0,00542168, máximo de canal 0,242971 e cobertura 0,000262467. O controle anterior falha em 1728/trajetória de aproximação/frame14: RMSE 0,00516107, máximo 0,34884; não promover esse controle como visualmente aprovado para todo o fixture.

**Retorno:** a fonte repete os pixels em 10/10 trajetórias; 0,05 também em 10/10. Foram 48/50 retornos RGBA32F exatos entre todos os caminhos. Reprovados: 0,15 em 1728/trajetória4 (quatro pixels PNG, máximo 1/255) e conservador em 3456/trajetória4 (dois pixels PNG, máximo 6/255). Este último desenha a geometria nativa. Logo a instabilidade não está isolada ao proxy anterior; causa aberta, sem afirmar que aquecimento ou redução de vértice a resolveu. Os gates continuam falhos, mesmo passando RMSE. `verification.json` registra os deltas PNG; `results.json` mantém hashes RGBA32F.

**Decisão:** manter como laboratório diagnóstico, sem promover os novos níveis ao renderer funcional. Os bons números anteriores eram de redundância recuperável; esta superfície contém informação geométrica nova. Próxima investigação: subdividir/adaptar regiões em torno de fissuras para não fazer um detalhe estreito vetar o restante, medir detalhe fotométrico além do geométrico, e primeiro estabilizar os controles de imagem. Repetir então o mesmo comparador, sem relaxar gates ou usar vencedores por pose.


## Validação e entrega local

712 testes Rust release/all-targets, Clippy com `-D warnings`, fmt (incluindo os dois gpu.rs), diff-check e 13 grupos MCP headless de render/replay_inspection/selection aprovados. O warning de compatibilidade futura pertence à dependência `block 0.1.6`; não houve novo warning de Clippy. A extensão compartilhada foi exercitada pelo smoke do normal_field_probe com UV: 16/16 comparações do campo e do controle conservador passam; a alternativa forçada sem campo mantém oito reprovações expostas, não promovidas.

690 PNGs da coleta final verificados em tamanho e integridade; 2.208 estados de UI/DPR simulados, incluindo referências a arquivos existentes, seleção de poses e escala física. Inspeção de capturas complementa as métricas. Isso não é aceite humano nem execução do painel em navegador nativo. Evidências em `artifacts/appearance-probe/rich-relief/`, com logs, `build.json`, `results.json`, `summary.json`, `verification.json` e painel `index.html`. Cinquenta PNGs do smoke 864 supersedidos foram removidos após confirmar ausência de uso; JSON diagnóstico preservado. Target/release corrente reutilizado, sem outputs de build por tentativa. Nenhum contrato funcional ou demo alterado.

## Continuação: repetibilidade e refinamento local — protocolo antes de implementar

Primeiro isolar a instabilidade com a mesma geometria, câmera, luz e buffers em capturas repetidas, incluindo uma câmera intermediária entre repetições. Comparar shader original, derivadas finas explícitas para filtragem, mip fixo apenas diagnóstico e saída UV sem textura. Alterar somente o laboratório; não chamar uma hipótese de causa comprovada. Eventual mudança de filtragem precisa passar comparação de imagem com o material original, além de repetir exatamente no mesmo build.

Em seguida testar subdivisão local dentro das regiões existentes: preservar os limites externos nativos e aumentar resolução somente onde o erro de relevo exige. Compilar alternativas e certificados offline, conservar o formato binário/144 decisões por frame quando possível. Comparar exatamente a mesma fonte/material, sem reduzir a resolução de renderização. Memória de variantes e CPU/draws entram no custo. Sem tabela de poses vencedoras e sem relaxar RMSE/p99/cobertura/retorno. Repetir perto/longe e transição nas duas resoluções; resultados negativos continuam úteis e explícitos.

Checklist prévio: geração/interseções/mapas no build; frame limitado a projeção/seleção/comandos; payload binário e canais consumidos; reuso de buffers/mapas; medir crescimento de níveis/metadados; readback somente no diagnóstico; sem paralelismo GPU durante benchmark; validação de fronteiras/certificados/codec e controles repetidos; preservar contratos funcionais; custos de alocação/driver/fonte mantidos explicitamente.

### Diagnóstico de filtragem: hipótese não confirmada como correção

`--repeat` conserva os mesmos buffers, câmera, luz e ordem de desenho, intercala outra vista e repete a captura RGBA32F 32 vezes por coorte. Primeira rodada: 16 coortes; original/3456/perto mudou em 2/32 capturas, até 18 pixels, erro máximo de canal 0,01065023. Derivadas finas, mip0 fixo e saída UV repetiram nessa rodada. Uma segunda rodada inclui novamente derivadas finas e original depois dos controles: 24 coortes, 768 capturas. Derivadas finas/1728/perto mudaram em 1/32 capturas (3 pixels, erro 0,002150774), original/1728/perto em 1/32 (2 pixels, erro 0,000813589). As oito capturas de referência fine versus implicit foram bit-idênticas, mas isso não evitou a instabilidade posterior.

Logo derivadas finas explícitas **não corrigiram** o problema. Mip fixo e UV ficaram estáveis nas amostras, mas não provam sozinhos causa raiz. Mip0 descarta a escolha de escala; não foi adotado para disfarçar a falha. O shader original e seus mipmaps permanecem iguais em todas as vias da comparação final. Não há afirmação de determinismo garantido ou atribuição conclusiva ao Metal. Evidências: `repeat.json`, `repeat.log` e `repeat-first-summary.json`.

### Refinamento compilado dentro das regiões

Cada uma das 144 regiões contém uma árvore temporária no build: quadrados de 64 até 2 células. Uma folha maior usa leque triangular com centro e todos os pontos nativos do perímetro; uma folha de 2 células conserva as oito faces fonte. Refinar somente as folhas cujo certificado excede o limite permite manter fissuras e reduzir áreas menos irregulares. Todas as fronteiras de folhas, inclusive vizinhas com escalas diferentes, conservam amostras nativas. Nenhuma costura precisa ser criada no frame.

Quatro limites fixos em unidades do objeto, escolhidos antes da coleta: 0,000008 / 0,000032 / 0,000128 / 0,000512. A árvore escolhe folhas por esses limites e agrega o máximo certificado; o runtime continua escolhendo uma das cinco representações pelo orçamento projetado de 0,05 ou 0,15 pixel. Não consulta capturas, poses vencedoras ou timers na seleção. O custo de decisão permanece 144 regiões, e a árvore temporária não é salva nem transferida para GPU.

Certificados são verificados novamente contra os XYf32 físicos de todas as interseções. Nível com erro exatamente zero só é aceito após confirmar as mesmas faces orientadas da fonte. Variantes sem redução referenciam o intervalo nativo, e variantes idênticas reutilizam o intervalo anterior: não armazenar cópias nunca escolhidas. Teste com um pico isolado de altura 0,1 exige preservação, igualdade das bordas, arestas internas sem ramificação, redução fora do pico, compartilhamento de intervalos e roundtrip binário.

Níveis completos: 1.179.648 / 1.178.256 / 1.101.904 / 750.720 / 449.200 triângulos. IGL1 adaptado: 65.697.568 bytes de arquivo; GPU de geometria: 45.711.564 bytes (43,594 MiB); com os mapas compartilhados, 70,261 MiB. É mais memória que os 53,447 MiB dos níveis uniformes e os 46,934 MiB da fonte, excluindo targets, driver, CPU e controles simultâneos. Build e autovalidação: 12,449 s em uma observação; não é benchmark repetido de build. Sem o compartilhamento de intervalos seriam 63.013.068 bytes de geometria GPU; foram evitados 17.301.504 bytes sem mudar os níveis selecionáveis.

A comparação pareada usa fonte completa, compilador anterior + hierarquia, níveis uniformes anteriores a 0,05px, refinamento local a 0,05px e a 0,15px. O campo histórico `conservative` no JSON dessa coleta significa níveis uniformes anteriores, explicitado por `conservative_mode`; não é mais o veto por cone. Ambos os seletores regionais dispensam o veto nesta investigação, preservando as falhas fotométricas no relatório. A fonte do IGL refinado é comparada byte a byte com a fonte regional anterior antes de acrescentar buffers dos controles.

A igualdade de retorno da referência permanece um gate: na coleta refinada, se falhar, os dados diagnósticos são gravados e o comando termina com erro, mantendo o painel explicitamente reprovado. Isso evita perder o diagnóstico inteiro por um abort antecipado; não converte uma referência instável em aprovação. O teste original conserva seu abort imediato.

Reprodução adicional:

```sh
cargo run --locked --release --example rich_relief_probe -- --repeat artifacts/appearance-probe/rich-relief
cargo run --locked --release --example rich_relief_probe -- --refine artifacts/appearance-probe/rich-relief
cargo run --locked --release --example rich_relief_probe -- --refined-run artifacts/appearance-probe/rich-relief
python3 scripts/report_rich_relief.py --refined
node scripts/check_rich_relief_ui.mjs --refined
```

O painel refinado usa `refined.html`; controles anteriores permanecem em `index.html`. Não usa o mip fixo ou outra filtragem para melhorar artificialmente a comparação. Smoke 864 revelou duas reprovações distantes mesmo em 0,05px; esse achado permanece registrado em `refined-smoke.json`, sem promover o resultado por conter menos triângulos.

### Resultado do refinamento local — coleta final

Apple M4 Pro / Metal, 138 capturas de comparação, 800 amostras temporais, 16 por caminho/coorte. Medianas abaixo em ms/frame de lote, incluindo escolha/codificação/submissão/espera. O shader original é igual em todos os caminhos. A coleta terminou com código 1 porque a referência falhou em um retorno; dados diagnósticos preservados. Não repetir a coleta até obter uma aprovação e apagar a falha.

| Pixels / trajetória | Anterior + hierarquia | Uniforme anterior 0,05px | Local 0,05px | Economia contra anterior |
|---|---:|---:|---:|---:|
| 1728 / perto 2,7 | 0.510510 | 0.539186 | 0.475788 | 6.80% |
| 1728 / média 6 | 0.425741 | 0.442959 | 0.275612 | 35.26% |
| 1728 / afastada 16 | 0.378263 | 0.425024 | 0.156585 | 58.60% |
| 1728 / distante 64 | 0.322933 | 0.382423 | 0.144400 | 55.28% |
| 1728 / aproximar/afastar | 0.363731 | 0.410133 | 0.239794 | 34.07% |
| 3456 / perto 2,7 | 0.776606 | 0.798719 | 0.791177 | -1.88% |
| 3456 / média 6 | 0.548835 | 0.554166 | 0.506412 | 7.73% |
| 3456 / afastada 16 | 0.386628 | 0.395923 | 0.265845 | 31.24% |
| 3456 / distante 64 | 0.346289 | 0.387028 | 0.195118 | 43.65% |
| 3456 / aproximar/afastar | 0.439809 | 0.450436 | 0.372182 | 15.38% |

Em 3456/distante, 0,346289→0,195118ms (−43,65%), 8/8 rodadas favoráveis; GPU0,295614→0,146458ms, CPU0,034040→0,037291ms. Duplicatas medianas 5,53%/5,27%. Em 3456/afastada, −31,24%, também 8/8 rodadas favoráveis. Em 1728/afastada, −58,60%. Perto3456, +1,88% de tempo, somente3/8 rodadas favoráveis; não existe ganho consistente próximo nessa resolução. Não somar CPU/GPU nem traduzir essas medidas isoladas em FPS de gameplay. Aproximar/afastar3456 permanece uma coorte visualmente reprovada por instabilidade da referência, mesmo mostrando mediana menor.

Local0,05 seleciona 449.200T nas duas órbitas mais distantes1728 e na mais distante3456 (−61,92% contra1.179.648T), com um draw. Perto3456 conserva1.128.384–1.141.200T e até33draws. Portanto o ganho vem de simplificar trechos locais com relevo real fora da zona próxima; não demonstra um platô universal perto nem reduz o custo de todos os triângulos invisíveis.

**Imagem:** local0,05, local0,15 e uniforme anterior passam138/138gates de cor/cobertura. Local0,05: RMSEmáximo0,004453874, p990,01796136, cobertura8,84807e-6. Máximo de canal0,46667686 ocorre em3456/perto/frame4; nessa captura há também diferença de cobertura de um pixel em2.557.750cobertos; não chamar a imagem de idêntica nem os erros locais de inexistentes. O anterior+hierarquia continua reprovando uma captura1728/aproximação/frame14. Gates numéricos são necessários, não substituem inspeção humana ou estabilidade temporal.

**Repetibilidade:**48/50retornos RGBA32F iguais. Falham uniforme anterior3456/perto (3pixelsPNG,max7/255) e fonte completa3456/aproximação (4pixelsPNG,max2/255). Ambos refinamentos repetem10/10retornos nesta coleta, mas isso não corrige a referência. A saída do runner é falha e o painel apresenta aviso global e por trajetória. Outros nove retornos da fonte passaram; as coortes estáticas afastadas mostram ganho parcial com gates aprovados.

**Validação de implementação:**713testesRust release/all-targets,Clippy-Dwarnings,fmt e13gruposMCPheadless aprovados. Novo teste do pico isolado cobre refinamento/bordas/arestas/compartilhamento/codec.690novosPNGs íntegros;2.208estadosDOM/DPR incluindo o aviso da referência instável;2.208estados do painel anterior rechecados. Captura próxima local inspecionada, sem aceite humano ou navegador nativo. Nenhuma mudança na engine funcional, NPCs ou contratos; target/release corrente reutilizado.

**Decisão:** avanço parcial de performance validado nas vistas estáticas afastadas, maior memória e ausência de ganho consistente muito perto. Não promover globalmente. Próximo gargalo continua sendo repetibilidade/filtragem com microtriângulos e escolha de caminho próximo que evite custo de muitos desenhos; preservar esse conjunto de evidências ao experimentar.

Cinquenta PNGs do smoke refinado foram removidos após lsof confirmar ausência de uso; JSON diagnóstico e os controles anteriores foram preservados. Revisão posterior: trabalho de árvore/interseções só no build; 144decisões e até33draws nas vistas próximas; posições12B/mapas compartilhados; variantes idênticas sem payload duplicado; arrays de escolhas/ranges e uniform slots do renderer comum ainda têm custo medido; nenhuma nova sincronização, contrato ou dependência; erro/codec/bordas validados; aumento de memória e reprovação temporal mantidos como limites explícitos.

## Continuação: filtragem explícita e seleção por custo — protocolo prévio

Isolar agora a saída das derivadas/LOD, além de amostrar os mesmos mipmaps com LOD escalar explícito. Manter o shader original como controle e registrar a diferença fotométrica; estabilidade de uma alternativa não autoriza mudar o material da comparação principal. Usar arquivo `filter-repeat.json` separado para preservar as observações anteriores. Rodadas em ordem invertida, 32 retornos por coorte, perto/longe e nas duas resoluções; sem repetir até passar.

Para a escolha automática, calibrar uma função pequena de trabalho (triângulos evitados e desenhos adicionais) em vistas distintas das vistas de validação. Ajustar coeficientes não negativos, reservar margem pelo resíduo e ruído das duplicatas, congelar antes da avaliação. Nenhuma tabela de vencedor por pose, captura ou consulta de imagem no frame. Comparar fonte, anterior, automático, local0,05 e local0,15 em nova coleta; custos da decisão dentro do lote e calibração inicial separados. O automático precisa manter residentes também os dados do caminho anterior; contabilizar esse custo, sem esconder os buffers de fallback.

Checklist anterior: build/material/formatos permanecem; seleção limitada a144regiões, poucos coeficientes numéricos; ajuste e readbacks somente no diagnóstico inicial; evitar trabalho por pixel novo no caminho principal; medir alocações/draws já existentes; sem dependências/threads/contratos funcionais novos. Gates de imagem/retorno inalterados, falhas preservadas; testes de monotonicidade e rejeição de modelos inválidos, regressões headless e medição pareada.

### Filtragem escalar explícita — resultado do diagnóstico

`--filter-repeat` conserva a coleta anterior e grava `filter-repeat.json`. São768retornos:256original,256LOD explícito,128saída das derivadas UV e128UV. O LOD explícito usa `log2(max(length(dpdxFine(uv)),length(dpdyFine(uv)))*dimensão)` separadamente para cada mapa e continua usando os mipmaps originais. Não é uma garantia de equivalência ao cálculo implícito da GPU.

Nesta coleta, original mudou em1/256retornos (3456/perto,31pixels,maxcanal0,0193105); LOD explícito mudou em33/256 (1retorno1728/perto e32retornos3456/perto na segunda passagem,até33pixels,max0,00104443). UVs e derivadas repetiram128/128cada. Todas as oito comparações de cor do LOD explícito contra original passaram os gates agregados (RMSEmáximo0,00183917), mas isso não corrige a repetibilidade. Comparações RGB dos controles UV/derivadas não são testes de fidelidade do material. Resultado consistente com instabilidade posterior à interpolação/derivadas nos casos amostrados, sem provar causa raiz ou defeito específico do driver. Material original permanece na comparação automática.

### Modelo de custo e rejeição antecipada

Calibração inicial por resolução: seis vistas com distâncias3,3/4,7/9/24/43/80 e ângulos0,37+0,71i; quatro rodadas alternadas, duplicatas,128frames/lote, aquecimento antes de cada amostra. Nenhum desses pares câmera/luz integra as trajetórias de validação. Ajuste por coordenadas de três coeficientes não negativos: ganho estimado = custo por milhão de triângulos evitados − custo por desenho adicional − custo fixo. Na versão corrigida, a margem é a maior superestimação do ganho, acrescida do ruído mediano das duplicatas daquela vista; subestimações não aumentam o risco de uma escolha lenta. A tentativa bilateral e sua correção estão registradas abaixo. É um estimador conservador neste hardware, não prova matemática de performance; não modela diretamente caches, sobreposição, pixels ou custo dos materiais.

Após calibrar, os coeficientes ficam congelados. O frame só lê contagens/certificados e os coeficientes, sem timers, capturas ou tabela de poses. Um limite otimista usa a distância máxima do objeto e os certificados para descobrir o menor número possível de triângulos do seletor. Se nem esse caso, com um único desenho, superar a margem, retorna ao anterior antes de visitar144regiões. A fronteira ordenada de erros/contagens é derivada uma vez no carregamento dos certificados já compilados; não recalcula geometria nem materiais. A projeção usa a mesma resolução física. Teste do bound em336combinações de câmera/resolução, e cada captura final compara a decisão com/sem atalho.

Quando não rejeita cedo, calcula a seleção regional0,05px e a compara ao caminho anterior; nenhuma relaxação para0,15px no automático. Esse caminho exploratório continua somente como controle. Em fallback, permanecem residentes os índices/vértices do anterior, que são contabilizados além de fonte/níveis/mapas. Calibração é custo inicial medido separadamente, sem amortização implícita nos tempos de render.

Reprodução desta continuação:

```sh
cargo run --locked --release --example rich_relief_probe -- --filter-repeat artifacts/appearance-probe/rich-relief
cargo run --locked --release --example rich_relief_probe -- --auto-run artifacts/appearance-probe/rich-relief
python3 scripts/report_rich_relief.py --auto
node scripts/check_rich_relief_ui.mjs --auto
```

Resultados e painel novos usam prefixo `auto-`/`auto.html`, preservando os controles anteriores. O campo histórico `conservative` nesta coleta é explicitamente `calibrated-automatic`; o painel mostra o caminho escolhido em cada captura. A calibração por hardware é inicialização do experimento, não trabalho de compilação offline do asset: níveis, materiais e certificados continuam preparados em build time, mas não chamamos medição no hardware alvo de build time.

### Primeira escolha automática e correção da margem

Primeira coleta preservada em `auto-conservative-diagnostic.json`: automático138/138gates e10/10retornos, referência10/10; um retorno exploratório3456/aproximação falha. Em3456/distante, anterior0,350686→auto0,207275ms(−40,89%,8/8rodadas). Perto0,745313→0,750949ms(+0,76%). Contudo3456/distância16 mantém anterior0,376790→auto0,392074ms(+4,06%,1/8rodadas), enquanto local forçado0,269817ms. Duplicatas do anterior21,63% nessa coorte: não atribuir toda a diferença ao seletor; CPUauto0,029236 versus anterior0,029700ms, diferença aparece na GPU/lote.

A revisão encontrou um erro conceitual na reserva: `abs(predito-observado)` penalizava também SUBestimar benefício. O risco de escolher o caminho lento é SUPERestimar benefício. Corrigir para `max(predito-observado,0)+ruído`, mantendo o maior valor e os coeficientes monotônicos. Isso não escolhe vencedor por pose e não reduz o gate visual. Os resultados anteriores ficam explícitos; nova coleta independente de timings com a mesma suíte de vistas e nova calibração, sem alegar um conjunto de validação jamais visto. A hipótese do protocolo inicial de margem bilateral foi rejeitada; não escolher limiares manualmente para forçar vitórias.

### Resultado final da escolha automática — margem unilateral

Apple M4 Pro/Metal; mesmas138vistas,800amostras,16por caminho/coorte e128frames/lote; material original intacto. Não comparar tempos absolutos entre as duas coletas como se frequência/caches/ruído fossem constantes. A tabela abaixo usa controles da mesma coleta.

| Pixels / trajetória | Anterior + hierarquia | Automático | Local0,05 forçado | Economia auto | Rodadas favoráveis | Capturas locais |
|---|---:|---:|---:|---:|---:|---:|
| 1728 / 0 | 0.573483 | 0.540808 | 0.516497 | 5.70% | 6/8 | 0/9 |
| 1728 / 1 | 0.412627 | 0.309116 | 0.305983 | 25.09% | 8/8 | 9/9 |
| 1728 / 2 | 0.339833 | 0.150774 | 0.153204 | 55.63% | 8/8 | 9/9 |
| 1728 / 3 | 0.334155 | 0.148429 | 0.143542 | 55.58% | 8/8 | 9/9 |
| 1728 / 4 | 0.374716 | 0.239542 | 0.257515 | 36.07% | 8/8 | 29/33 |
| 3456 / 0 | 0.856888 | 0.761991 | 0.853117 | 11.07% | 8/8 | 0/9 |
| 3456 / 1 | 0.537945 | 0.546707 | 0.505986 | -1.63% | 5/8 | 0/9 |
| 3456 / 2 | 0.374183 | 0.382659 | 0.271594 | -2.27% | 4/8 | 0/9 |
| 3456 / 3 | 0.353173 | 0.196798 | 0.191519 | 44.28% | 8/8 | 9/9 |
| 3456 / 4 | 0.421377 | 0.382091 | 0.364487 | 9.32% | 8/8 | 9/33 |

Trajetórias0/1/2/3 são distâncias2,7/6/16/64;4é aproximação/afastamento. Próximo, o automático conserva o anterior: a diferença de timing entre caminhos que desenham a mesma representação **não é ganho atribuível à redução geométrica**. Em3456/perto, aparentes11,07% de ganho estão na escala da variação das duplicatas do anterior(12,95%); preservar como ruído/efeito de ordem ainda não isolado, sem alegar aceleração próxima.

Longe3456: anterior0,353173→auto0,196798ms,−44,28%,8/8rodadas favoráveis, duplicatas6,06%/9,84%. Longe1728:−55,58%,8/8rodadas, duplicatas14,48%/13,54%. As respectivas trajetórias passam qualidade e retorno. Porém3456/distância16: automático0,382659ms versus anterior0,374183ms(+2,27%;4/8rodadas;duplicatas7,96%/3,29%) e local forçado0,271594ms. Não há prova de regressão sistemática pequena, mas há perda clara de oportunidade: a calibração conservadora deixa de usar um local substancialmente mais rápido. Nenhum seletor universalmente ótimo ou platô novo1×–100× é demonstrado.

Calibração final:1728=7,720s,coeficientes[0,27680144;0;0],margem0,05417787ms;3456=10,820s,[0,24764688;0;0],margem0,11317008ms. Nos dois ajustes o termo de draws ficou zero sob a restrição de não negatividade. A reserva unilateral corrigiu a lógica, mas o ruído de calibração ainda domina e não permitiu recuperar o caso intermediário3456. Estes custos iniciais não estão amortizados na tabela. A próxima revisão de custo deve separar a variação do controle equivalente e o trabalho por pixel, sem baixar margem manualmente ou treinar numa tabela de vencedores.

**Qualidade/retorno:**automático138/138limites agregados,RMSEmáximo0,004453874,p990,01796136,cobertura8,84807e-6,maxcanal0,37672484. Fonte10/10retornos; automático9/10. Falha1728/perto, onde estava usando o anterior:1pixelPNG variou1/255. Também falha um retorno exploratório1728/aproximação(1pixel,1/255). Local0,05forçado10/10; anterior mantém1falha de qualidade na captura1728/aproximação/frame14. Isso não torna o resultado visualmente idêntico nem garante estabilidade. Painel destaca automático reprovado globalmente e por trajetória. Auditoria `--audit-auto` termina código1 com os dados preservados; `--auto-run` passa agora por essa auditoria, além do gate da referência. A coleta GPU foi concluída antes de acrescentar esse gate ao runner; a auditoria é posterior sobre seus resultados, sem repetir a coleta até obter aprovação.

**Memória:**automático fonte+níveis+anterior residente+mapas=90,528MiB, contra70,261MiB do local sem esse fallback e46,934MiB de fonte+mapas. São20,268MiB adicionais para a geometria anterior. Targets,driver,cópiasCPU e recursos de infraestrutura não entram. Não houve economia de memória. `auto_geometry_bytes` inclui os buffers efetivamente necessários ao automático, sem esconder a cópia do fallback.

**Validação:**715testesRust release/all-targets,Clippy-Dwarnings,fmt/diff e13gruposMCPheadless aprovados. Testes incluem fit/monotonicidade/modelo inválido, regressão da margem unilateral,336combinações para bound otimista e decisão com/sem poda nas138capturas.690PNGs íntegros;2.208estadosDOM/DPRdo painel automático, incluindo reprovação de retorno, e2.208em cada um dos dois painéis anteriores. Sem aceite humano/navegador nativo. Build corrente reutilizado, controles anteriores e diagnóstico de primeira tentativa preservados; nenhum novo contrato funcional,NPC,demo ou publicação. Evidências `auto-results.json`, `auto-summary.json`, `auto-verification.json`, `auto-conservative-diagnostic.json`, `filter-repeat.json` e logs correspondentes.

**Decisão:**manter laboratório em investigação. Ganho distante permanece; custo inicial/memória/ruído e um retorno instável impedem aprovação global. Próximas hipóteses: controlar o ruído dos caminhos equivalentes para calibrar melhor, e isolar a filtragem com leituras explícitas de texels apenas como diagnóstico, comparando sempre ao material original. Não tratar retorno estável da referência nesta coleta como resolução das falhas do diagnóstico dedicado.

## Continuação: texels explícitos — protocolo prévio

Prioridade confirmada em 27/10: consolidar estabilidade antes de composição de cena; amostragem atmosférica registrada como etapa posterior em [graphics-roadmap.md](graphics-roadmap.md). Próximo diagnóstico mantém geometria/material e mapas binários; compara original, LOD explícito com filtro de hardware e trilinear manual (textureLoad, quatro texels por nível, dois níveis por mapa, clamp-to-edge). Mesmo cálculo escalar de LOD do diagnóstico anterior, não afirmar equivalência bit a bit com filtragem implícita.

Protocolo fixado antes da coleta: duas resoluções, perto/longe, 32 retornos por coorte e duas passagens em ordem invertida. Resultado separado `texel-repeat.json`; falhas anteriores preservadas e não repetir até passar. Avaliar diferença contra original, igualdade RGBA32F e, se estável, ampliar para trajetórias/tempo total antes de adotar. Não mudar material principal automaticamente.

Checklist prévio: (1) leituras por fragmento apenas na variante diagnóstica; (2) mesmos buffers, readback/alocações só no harness; (3) RGBA8 original e f32, sem novo formato; (4) reutilizar footprint entre mapas, níveis limitados pelos mipmaps; (5) até16textureLoad por fragmento, custo extra explícito; (6) upload por instalação, nunca por pixel, sem payload adicional; (7) GPU serial, sem thread/lock novo; (8) variantes substituem pipeline explicitamente, mesma malha/mapas; (9) retorno estrito, comparação com original e regressões headless; (10) filtro manual pode custar mais e alterar arredondamento, manter original como controle.

O primeiro diagnóstico de texels também encontrou instabilidade no filtro manual; manter seus resultados. Próxima hipótese: o pipeline que usa derivadas pode receber valores diferentes mesmo quando uma variante separada que apenas desenha derivadas repete. Instrumentar dois attachments no MESMO desenho: cor final e dpdxFine/dpdyFine de UV, ambos RGBA32F. Isso evita concluir sobre entradas do material com base em outro shader otimizado. Capturar original/LOD escalar/manual, duas resoluções, perto/longe, duas passagens invertidas e32retornos; manter limites, registrar correlação no pixel e não afirmar que instrumentação é neutra. Target extra só no diagnóstico,16B/pixel, fora do renderer normal. Sem medição de performance deste MRT e sem promover filtro.

### Resultado: filtro manual não corrige; derivadas variam no mesmo desenho

Duas coletas fixas de768retornos cada, sem nova tentativa para obter aprovação. Apple M4 Pro/Metal,1728×1117 e3456×2234, duas poses perto/longe,32retornos por coorte e seis instalações em ordem invertida. Cada variante tem256retornos. Todas usam a mesma fonte de1.179.648triângulos e os mesmos mapas binários. Não houve medição de performance neste recorte.

| Coleta / variante | Retornos que mudaram | Máximo de pixels por retorno | Maior mudança de canal linear |
|---|---:|---:|---:|
| Sem MRT / original | 0/256 | 0 | 0 |
| Sem MRT / LOD explícito | 1/256 | 2 | 0,0010170 |
| Sem MRT / filtro manual | 3/256 | 21 | 0,0087598 |
| Com MRT / original | 4/256 | 10 | 0,0019465 |
| Com MRT / LOD explícito | 33/256 | 2 | 0,0070878 |
| Com MRT / filtro manual | 0/256 | 0 | 0 |

Os dois runners terminaram com código1 após salvar os resultados: há instabilidade. Todas as24comparações agregadas por coleta passam os limites de cor/cobertura; isso não supera a reprovação de retorno. MaiorRMSE contra controle0,00183917 (LOD explícito); manual0,00178658. São somente oito comparações por variante, não toda a trajetória138 do painel anterior. A referência da coleta MRT é o original com MRT; instrumentação pode alterar execução/otimização/ocupação e não é uma correção.

**Evidência nova:** nos37retornos com cor diferente da coleta MRT, cada pixel de cor alterado também tinha derivadas UV diferentes no attachment do MESMO desenho. Zero eventos de cor alterada com derivadas iguais. Contagem de eventos de pixel com derivadas diferentes (soma entre retornos, não pixels únicos):86no original,171noLOD explícito,0no manual. Algumas mudanças de derivadas não mudam a cor. Exemplo original1728/perto/pixel(833,885): dx passa de(0,00000172853;0,00071528554) para(−0,00000071526;0,00071531534), dy de(0,00083041191;0,00001487136) para(0,00085407495;0,00001454353); RGB também muda. Evidências limitadas a oito exemplos por coorte no relatório, sem imagens temporais gigantes persistidas.

O attachment registra dpdxFine/dpdyFine; no controle implícito isso não expõe necessariamente a política interna de derivadas do sampler. No LOD explícito são as mesmas expressões usadas no cálculo escalar. Isso localiza uma fonte possível da oscilação na estimativa do footprint usada pela textura. Não prova que derivadas sejam a única causa, qual etapa produz a variação, nem um defeito do driver. O resultado manual estável COM MRT não autoriza adotá-lo: ele já falhou SEM MRT nesta mesma unidade. A conclusão anterior de entradas estáveis em shaders que apenas mostram UV/derivadas não se estende ao shader completo; a instrumentação conjunta é necessária.

**Decisão:** manter shader e seleção principais intactos, não promover manual, não alegar ganho de FPS nem estabilidade corrigida. Próxima hipótese é comparar cálculo de footprint independente das diferenças entre fragmentos com o original, primeiro isolando bordas/microtriângulos; qualquer alternativa precisa passar fidelidade, retorno e custo total antes de substituir o material. Previsão de custo intermediária e composição de cena continuam pendentes no roadmap. A atmosfera foi registrada e permanece adiada.

Reprodução:

```sh
cargo build --locked --release --example rich_relief_probe
# Ambos retornaram1 nesta coleta, preservando os diagnósticos:
target/release/examples/rich_relief_probe --texel-repeat artifacts/appearance-probe/rich-relief
target/release/examples/rich_relief_probe --inputs-repeat artifacts/appearance-probe/rich-relief
python3 scripts/report_rich_relief_repeat.py
```

Artefatos: `texel-repeat.json`, `inputs-repeat.json`, logs correspondentes e `texel-summary.json` (checksums/coortes/gates, sem promover o resultado). Controles anteriores preservados; painel automático anterior continua reprovado, sem substituir seu relatório por uma coleta favorável.

Checklist posterior: (1) diagnóstico opt-in por fragmento, engine funcional intacta; (2) arrays de attachments fixos sem alocação por draw, recursos/readbacks só do harness; (3) mapasRGBA8 originais/f32 e clamp por nível, sem payload novo; (4) footprint compartilhado entre mapas, máximo16loads no filtro manual; (5) coortes/32retornos limitados, oito exemplos detalhados por coorte; (6) MRT acrescenta16B/pixel,29,452MiB/117,809MiB de target nas duas resoluções, mais readbacksCPU temporários, somente quando habilitado; (7) GPU serial/espera explícita no readback, sem thread/dependência nova; (8) reinstalar material comum libera target, desativar normal field não anexa target incompatível e instalar campo legado limpa diagnóstico; (9) saída1 conserva gates e resultados, compilação WGSL exercitada naGPU, regressões registradas em validation.md; (10) instrumentação altera carga/compilação e não mede velocidade, filtros alternativos não promovidos.

## Continuação: footprint analítico — protocolo prévio

Testar projeção diferencial do plano tangente por pixel, sem dpdx/dpdy na escolha de mip. Usar a orientação do mapa de normais já compilado, amostrado em mip0, e a matriz/viewport físicas. No heightfield UV=0,5·XY+0,5, tangentes (nz,0,−nx) e(0,nz,−ny) projetadas fornecem Jacobiano2×2; sua inversa estima UV por pixel. Singularidade/grazing conserva filtro grosso finito. É aproximação pelo campo de normais suavizado/quantizado, não certificado de equivalência à face geométrica nem suporte a malhas arbitrárias.

Duas variantes: analytic-lod com sampler de hardware em LOD escalar; analytic-manual com textureLoad dos mesmos mipmaps. Mantém geometria/texturas/BRDF e luz; apenas footprint muda. Sem payload novo, nova textura ou alteração de Scene/MCP. Viewport ocupa dois floats antes zerados no uniform do harness, consumidores antigos verificados usam só material.x. Protocolo inicial fixo: original/analytic-lod/analytic-manual, duas resoluções, perto/longe, duas passagens invertidas,32retornos porcoorte. Preservar resultados em analytic-repeat.json; nenhum rerun até passar. Se candidato repetir e passar cor/cobertura, ampliar a138vistas, controles fonte/anterior/regional, retornos e timing pareado incluindo seleção. Se falhar, registrar onde e não promover.

Checklist prévio: (1) Jacobiano porfragmento só opt-in; (2) sem novas coleções/alocações runtime, mesmos buffers; (3) RGBA8/mips/f32 originais, viewport2f32 já cabem no uniform; (4) orientação/footprint reutilizados entre mapas; (5) custo constante porpixel com uma consulta mip0 adicional, até20loads lógicos na variante manual; (6) sem novo arquivo/upload/readback na seleção; (7) mesma filaGPU, testes/bench serializados; (8) variante explícita, resolução acompanha uniform; (9) validar fórmula por projeção/interseção independente e imagem/retorno/custo, manter gates; (10) normal de shading difere da face e custo pode piorar, então original continua padrão.

Primeira coleta analítica: ambos candidatos0/256retornos alterados; original4/256, portanto runner termina1. Comparações8/8porvariante passam, mas RMSEmáximo0,004748 fica perto do limite0,005. Ampliar agora analytic-lod (menos leituras explícitas) para138poses; manual permanece diagnóstico, não promovido. Comparar originalfonte, originalanterior, analíticoanterior, originalregional0,05 e analíticoregional0,05. Mesma hierarquia/seleção, nenhuma correção de modelo de custo nesta rodada. Compartilhar mapas entre pipelines; manter32poses/128frames porlote,8rodadas,duplicatas/ordemalternada nas10coortes. Registrar ganhos e perdas de imagem separadamente, mesmo se candidato for estável. A fonte original conserva seu gate e todos os dados ficam salvos antes de reprovar. Alternância de shader porlote, compilação/upload fora do timing; seleção geométrica dentro. Não anunciar implementação de seletor automático novo.

### Footprint analítico — resultado e decisão

O candidato **não foi promovido**. A hipótese melhorou a repetibilidade nas amostras coletadas, mas não conservou a resposta visual do material nem trouxe ganho consistente de tempo. Shader original e seletor automático permanecem intactos. A atmosfera continua na etapa posterior do roadmap.

`analytic-repeat.json`: protocolo fixo de768retornos,256por variante. Original mudou4retornos (até10pixels); `analytic-lod` e `analytic-manual` repetiram256/256cada. Os oito comparativos de imagem por variante passaram, porém o RMSE máximo do analítico ficou0,004748, próximo do limite0,005. O runner terminou1 por reprovação do original; não houve repetição até conseguir aprovação.

A ampliação pré-definida para138poses, incluindo giro/luz e aproximação/afastamento, reprovou os candidatos em41poses cada. Resultados no Apple M4 Pro/Metal,1728×1117 e3456×2234:

| Caminho | Comparações aprovadas contra fonte | Retornos exatos | RMSE máximo | p99 máximo |
|---|---:|---:|---:|---:|
| Anterior com shader original |137/138|10/10|0,005161|0,001007|
| Anterior com shader analítico |97/138|10/10|0,007168|0,031249|
| Regional0,05px com shader original |138/138|10/10|0,004454|0,017961|
| Regional0,05px com shader analítico |97/138|10/10|0,007171|0,031253|

A fonte original passou8/10retornos; falhou nas trajetórias próximas das duas resoluções. São48/50retornos no conjunto, portanto reprovação global. Nos PNGs da fonte,1728/perto muda12.404pixels (máximo54/255) e3456/perto37.616pixels (máximo81/255); não são apenas as diferenças esparsas de um nível observadas em algumas coletas anteriores. A causa permanece aberta. A falha fotométrica do anterior original já existia no ensaio precedente. Retorno exato mede repetição da mesma pose; não demonstra ausência de cintilação ao mover a câmera. O resultado de256retornos do manual não autoriza promovê-lo: ele não foi ampliado para a trajetória completa.

O controle que mantém exatamente a geometria e troca somente o shader também falha:97/138 no anterior e93/138 no regional. Portanto a divergência não depende da simplificação geométrica. O pior RMSE do candidato aparece em1728/caso1/frame2 (distância6). Fonte e candidato foram inspecionados; semelhança da imagem ajustada à tela não substitui os gates porpixel. Máximo de canal do candidato0,48143; não alegar pixels preservados.

A inversão do Jacobiano foi confrontada com interseções independentes de raios e planos em768combinações, incluindo duas resoluções, quatro distâncias,16ângulos, três normais e dois eixos. O teste passa. Isso valida a fórmula no domínio de um plano: **não valida usar a normal de shading quantizada/suavizada como normal geométrica**, nem reproduz a política interna do filtro implícito. A aproximação da tangente e a redução do footprint a um escalar são limitações a isolar, não uma causa raiz já comprovada.

#### Custo pareado

800amostras,128frames/lote, oito rodadas com ordem alternada e duplicatas. Mesmo payload residente, seleção de geometria incluída no lote; pipeline trocado fora do lote, sem reconstruir mapas. Tempo total de parede porframe, sem somar intervalosCPU/GPU sobrepostos:

| Resolução / distância | Regional original | Regional analítico | Variação de custo | Rodadas analítico mais rápido |
|---|---:|---:|---:|---:|
|1728 /2,7|0,452440ms|0,452152ms|−0,06%|1/8|
|1728 /16|0,150299ms|0,157729ms|+4,94%|0/8|
|3456 /6|0,501186ms|0,541652ms|+8,07%|1/8|
|3456 /64|0,191426ms|0,191926ms|+0,26%|4/8|

Em3456/distância6, GPU regional0,448912→0,495623ms; CPU de codificação0,031997→0,033360ms. Duplicatas totais medianas4,95%/3,57%. No anterior, a melhor diferença aparente foi−4,62% nessa distância, mas só5/8rodadas e dispersão duplicada7,89%/3,59%: não constitui ganho robusto. Tabela completaCPU/GPU/total/dispersão em `analytic-summary.json` e painel `analytic.html`.

Payload adicional do candidato:0B. Geometria residente63,861MiB + mapas compartilhados26,667MiB =90,528MiB, incluindo fonte/níveis/fallback. Targets, pipelines, driver e cópiasCPU não incluídos. Não há economia de memória; custo adicional é Jacobiano e amostra da normal no mip0 porfragmento. A variante manual chega a20leituras lógicas de texel e permanece diagnóstico. Não houve nova comparação1×–100× nesta unidade: trata-se do material rico sobre a mesma fonte de1.179.648triângulos.

#### Reprodução e continuidade

```sh
cargo build --locked --release --example rich_relief_probe
target/release/examples/rich_relief_probe --analytic-repeat artifacts/appearance-probe/rich-relief
target/release/examples/rich_relief_probe --analytic-run artifacts/appearance-probe/rich-relief
python3 scripts/report_rich_relief_analytic.py
node scripts/check_rich_relief_analytic_ui.mjs
```

Ambos runners salvam resultados antes de reprovar; nesta coleta terminam1. O relatório gera o painel mesmo reprovado e não serve como sinal de aprovação do experimento. Os binários de entrada atuais são os do protocolo refinado/anterior descrito acima; não recompilar silenciosamente outro material para repetir a coleta.

Próximo diagnóstico: separar a tangente geométrica efetiva do triângulo da normal de iluminação, comparando ambas sob a mesma política explícita de filtragem. Isso pode exigir metadados compilados ou acesso à geometria; contabilizar bytes, leituras e cálculos antes de adotar. Preservar o controle original, investigar anisotropia/contornos sem relaxar limites e só depois ampliar a cena. A repetibilidade do candidato nesta coleta não resolve a instabilidade da referência.

#### Revisão estática posterior

1. Frequência: Jacobiano/amostra adicional somente no fragmento dos shaders opt-in; construção de pipeline fora dos lotes (`gpu.rs`).
2. Alocações/cópias: `RichMaterial` compartilha bind group/texturas; não duplica uploads. Seleção mantém vetores/ranges já existentes, incluídos no custo.
3. Tipos/acesso: mapasRGBA8/mips, posição12B/índicesu32 intactos; largura/altura usam duas posições antes vazias do uniform. Binários do asset não mudaram.
4. Repetição: resolve footprint uma vez porfragmento, reutilizado nos dois mapas. Amostra mip0 extra é custo conscientemente mantido.
5. Escala/limites: domínioheightfieldUV=XY; singularidade usa último mip. Não generalizar para malhas arbitrárias, polígonos multicamadas ou câmera atravessando superfície.
6. GPU/I/O: pipelines compartilham mapas; capturas/readback/JSON somente no diagnóstico, fora dos timings. JSON não é transporte do asset.
7. Sincronização: sem nova thread, trava ou dependência; benchmarkGPU executado isolado das regressões.
8. Invalidação: escolha explícita dos handles muda shader/bindings; seleção limpa attachmentMRT diagnóstico. Testes legados exercitam bindings normais/UV.
9. Correção:716testesRust, incluindo768casos geométricos independentes; verificações e gates do experimento registrados separadamente em `validation.md`.
10. Custos mantidos: memória dos controles/fallback, alocações da seleção, cálculo adicional e erro da tangente. Resultado negativo preservado; sem promoção, ganho deFPS ou aceite humano alegado.

## Tangente geométrica por face — protocolo prévio

Próximo isolamento autorizado: conservar toda a fonte refinada e comparar quatro shaders sobre os mesmos buffers/mapas: original implícito, analítico pela normal de shading, analítico pela tangente real com LOD escalar e tangente real com gradientes UV vetoriais. A normal de iluminação continua vindo do mesmo mapa em todos. Não alterar seletor automático nem simplificar mais a geometria neste recorte. Usar fonte completa permite acessar o índice da face sem inventar um contrato de offsets por draw/região.

Compilar em build time IGT1: duas inclinações f32 por triângulo de heightfield (8B/face), calculadas em f64 das posições f32 efetivas e verificadas antes de quantizar. Cabeçalho/versionamento, hashes da fonte/payload, tamanho e finitude verificados no load; nenhum JSON no transporte. 1.179.648 faces acrescentam9MiB residentes, explicitados como custo diagnóstico, não formato final econômico. Fragmento consulta a face via primitive_index; recurso opcional do dispositivo apenas neste ensaio, sem elevar requisito do renderer principal. Falha de suporte é explícita. Alternativa de expandir vértices foi evitada por multiplicar transporte e mudar o trabalho geométrico. A generalização para vários draws fica adiada até a hipótese passar.

Protocolo fixo:138poses nas duas resoluções físicas, quatro órbitas/distâncias e aproximação/afastamento com luz móvel; quatro modos, retornos exatos. Um bloco inicial de32retornos por modo/resolução/perto-longe testa repetibilidade sob alternância de pipelines. Salvar resultados antes de reprovar; não repetir coleta até passar. Capturas/compilação/upload fora do timing; oito rodadas alternadas com duplicatas,128frames/lote. Comparar candidatos com original e geometric-lod versus shading-lod; imagem tem os mesmos gates RMSE0,005/p990,02/cobertura0,0005. Se os limites falharem, manter diagnóstico e não integrar. A coleta original permanece controlada mesmo se instável. Reportar memória,CPU/GPU/tempo total e dispersão; nenhuma nova promessa1×–100× ou FPS.

Checklist prévio: (1) derivação de inclinações offline, Jacobiano/leitura porfragmento só no candidato; (2) um buffer8B/face, mapas e posições compartilhados, sem expansão de vértices; (3) f64 no build/f32 binário no runtime, faces ligadas ao hash da fonte; (4) gradientes calculados uma vez e usados pelos dois mapas; (5) limite explícito heightfield/nz não degenerado, fonte única no draw; (6) payload carregado uma vez, readbacks só diagnóstico; (7) GPU serial e timings sem builds concorrentes, sem novas threads; (8) binding/feature opt-in, shader original intacto; (9) testar codec/corrupção/ligação à fonte/inclinação e projeção independente, depois imagem/retorno/custo; (10) +9MiB e leitura porfragmento são custos aceitos para isolamento, não proposta de integração definitiva.

Implementação do protocolo: a primeira compilação rejeitou o limite absoluto de arredondamento baseado no comprimento da aresta. Esse limite ignorava cancelamento entre os termos XY do plano. O certificado agora limita o erro por EPSILON_f32 vezes a soma das magnitudes dos termos avaliados, com teste de cancelamento; gates de imagem inalterados. O binário de produção do ensaio compilou/verificou1.179.648faces em0,098s,9.437.224B (9MiB de payload +40B de cabeçalho). A tentativa inicial de shader usou `enable primitive_index`, ainda não implementado pelo parser Naga27; o builtin nativo funciona com a feature opt-in do dispositivo sem essa diretiva. Essa falha ocorreu antes da coleta, registrada em tangent-startup.log. A feature pode acrescentar processamento geométrico em alguns backends; seu custo faz parte do timing.

Reprodução deste recorte (os comandos de build e execução são separados):

```sh
cargo build --locked --release --example rich_relief_probe
target/release/examples/rich_relief_probe --tangent-build artifacts/appearance-probe/rich-relief
target/release/examples/rich_relief_probe --tangent-run artifacts/appearance-probe/rich-relief
python3 scripts/report_rich_relief_tangent.py
node scripts/check_rich_relief_tangent_ui.mjs
```

O runner testa512repetições estáticas,138poses/40fechamentos de trajetória e640lotes medidos. São552PNGs e1.656estados esperados do painel para3candidatos,duas escalas de exibição eDPR1/2. Arquivos correntes:source.igt,tangent-results.json,tangent-summary.json,tangent.html. O JSON é relatório; a GPU lê o buffer binário de inclinações. IGT1 não é integrado ao pacote/seleção regional, e a guarda do harness exige um único draw da fonte completa começando no índice0 para evitar metadados da face errada.

### Tangente geométrica — resultado

**Avanço de fidelidade/repetibilidade do candidato, sem adoção por custo.** As duas variantes geométricas passam138/138comparações contra o original,128/128repetições estáticas e10/10retornos de trajetória cada. A normal de shading com a mesma política deLOD escalar passa98/138comparações. Alterar apenas a origem da tangente, mantendo oLOD escalar, elimina as40reprovações desta coleta; portanto o sucesso não depende de trocar para gradientes vetoriais. Isso sustenta separar normal de iluminação e plano geométrico no footprint deste fixture, não uma solução geral de materiais.

| Variante | Imagens aprovadas | Repetições estáticas iguais | Retornos | RMSE máximo | p99 máximo |
|---|---:|---:|---:|---:|---:|
| Normal do material / LOD |98/138|128/128|10/10|0,007168|0,031249|
| Tangente geométrica / LOD |138/138|128/128|10/10|0,004304|0,017363|
| Tangente geométrica / gradientes |138/138|128/128|10/10|0,004259|0,017008|

Cobertura idêntica à fonte nas138vistas de ambos os candidatos geométricos; máximo absoluto de canal0,376984, portanto os pixels não são idênticos. PiorRMSE em1728/distância64/frame7,961pixels de cobertura. Entre gradientes eLOD geométricos:138/138gates,RMSEmáximo0,000625/p990,002332. A diferença entre políticas é pequena nesta amostra, sem demonstrar equivalência geral ou filtragem anisotrópica completa (sampler continua o anterior).

A **referência original falha64das128repetições estáticas**, todas as32próximas de cada resolução em comparação à primeira captura. Até160.851pixels float alterados,RMSEmáximo0,000743 ecanal0,376984. Essa contagem é sobre floats e não equivale ao número de diferençasPNG visíveis. Depois desse bloco, a referência passa10/10retornos de trajetória; candidatos e original somam40/40retornos. A dependência da fase da coleta merece isolamento; não concluir que aquecimento corrige nem atribuir ao driver. O runner termina1 pelo bloco estático e mantém os dados. A aprovação individual do candidato não elimina a reprovação geral ou comprova qualidade temporal durante movimento.

#### Performance e memória

Apple M4 Pro/Metal,640amostras,128frames/lote, oito rodadas alternadas/duplicadas, toda a fonte1.179.648T. Nenhuma seleção regional participa. Total de parede porframe:

| Pixels / distância | Original | Geom. LOD | Variação | Geom. gradientes | Variação |
|---|---:|---:|---:|---:|---:|
|1728 /2,7|0,523880ms|0,591833ms|+12,97%|0,585983ms|+11,85%|
|3456 /2,7|0,779658ms|0,876629ms|+12,44%|0,875909ms|+12,35%|
|3456 /6|0,549363ms|0,618227ms|+12,54%|0,603970ms|+9,94%|
|3456 /64|0,375444ms|0,380383ms|+1,32%|0,382149ms|+1,79%|
|3456 /aproximar-afastar|0,410745ms|0,433984ms|+5,66%|0,440652ms|+7,28%|

Perto3456, gradientes perdem8/8rodadas; duplicatas medianas6,28%original/2,96%candidato. GPU0,736732→0,830669ms,CPUcodificação0,030049→0,029851ms: regressão concentrada naGPU, sem isolarmos ainda a parcela de consulta da face/Jacobiano/recurso primitive_index. O pequeno ganho geom.LOD1728/distância16 (0,409441→0,389102ms,−4,97%,8/8rodadas) fica próximo da dispersão duplicada4,09%/4,01%; registrar sinal localizado, não ganho global nem justificativa para regressão próxima. Tabela completa deGPU/CPU/total,dispersão e rodadas emtangent-summary.json.

Fonte20,268MiB + mapas26,667MiB =46,934MiB; candidatos acrescentam9MiBdeinclinações, total55,934MiB (+19,18%depayloadresidente). ArquivoIGT1 tem9.437.224B; payloadGPU9.437.184B. Targets,driver,pipelines e cópiasCPU excluídos. Este comparador não mantém todos os níveis/fallbacks do seletor anterior; **não comparar55,934 com90,528MiB como economia**. Mantida a mesma malha nos quatro modos, logo não houve redução adicional de triângulos ou nova suíte1×–100×. Dados de build utilizados porconsulta de face, sem recalcular inclinações noframe.

Decisão: conservar material/shader/seletor originais. Tangente real vira candidato de pesquisa com fidelidade validada neste recorte, não padrão. Próximo passo é separar o custo de acesso porface e cálculo dofootprint e estudar compartilhamento/compactação de dados com erro verificável; integrar ao caminho regional somente contando todos os níveis e offsets, mantendo os mesmos gates. Em paralelo lógico, a referência precisa de um teste controlado de primeira captura versus sequência já executada, sem descartar a primeira para obter aprovação.

Checklist posterior: (1) inclinações offline, Jacobiano/leitura somente nosfragmentos candidatos; (2) umbuffer9MiB, mapas/vértices compartilhados, sem expansão de vértices; (3) IGT1validado e teste de cancelamento/corrupção/fonteobsoleta, f32 consumido diretamente; (4) gradientes reutilizados entre mapas, sem consulta mip0extra do shading; (5) heightfield/drawúnicofonte explicitamente guardados, sem suporte regional implícito; (6) uploads/pipelines/capturas fora doslotes, leitura dobuffer ebind incluídos, relatórios fora doruntime; (7) feature opcional no dispositivo diagnóstico, filaGPU isolada dasregressões, semthread/dependência nova; (8) instalação legado desativa bindingdefaces, variantes alternam handles sem remontar mapas; (9)719testesRust/Clippy/fmt e13gruposMCPheadless, métricas doexperimento separadas e falha preservada; (10) +9MiB e regressão próxima~12%mantidos apenas como diagnóstico, sem ganhoFPS/aceitehumano/integração alegados.

## Compactação e custo da tangente — protocolo prévio

Investigar separadamente acesso por face, projeção e transporte de inclinações. Manter a fonte completa de 1.179.648 triângulos, mesmos mapas, shader original e política geométrica por gradientes como controles. Novos caminhos opt-in: (a) índice de primitiva mais guarda de limite, sem ler inclinações; (b) leitura f32 mais guarda de validade, sem Jacobiano; (c) gradientes f32 usando posição de fragmento fornecida pela rasterização; (d) o mesmo cálculo com duas inclinações signed-normalized16 em um u32. Guardas são satisfeitas por toda a fonte validada; ainda assim podem mudar compilação/execução e não são medidas puras de uma instrução. Não somar/subtrair tempos para inventar decomposição exata.

IGP1: cabeçalho de versão, contagem, hashes de fonte/payload; payload contém escala f32 comum seguida de um u32 por face. Escala em potência de dois cobre o máximo absoluto; quantização simétrica [-32767,32767], erro de coeficiente limitado por meia célula mais arredondamento f32. GPU usa unpack2x16snorm e escala. Alvo: 4,5MiB +4B contra9MiB, sem dependência nova e sem expansão em runtime. Este limite não certifica erro de imagem em vistas rasantes; manter gates anteriores. Normal de iluminação continua separada e intacta.

Cálculo novo usa @builtin(position) do fragmento (centro do pixel e inverso de clip.w), evitando reconstruir clip pela posição mundial; colunas da matriz projetam as duas tangentes. Comparar tanto com original implícito quanto com tangente f32 anterior, isolando erro da quantização contra o mesmo shader f32. Nenhum novo contrato na engine, seleção regional ou recurso de plataforma além do primitive_index já opt-in.

Coleta fixada antes de implementar: seis modos em138poses, duas resoluções físicas e câmera/luz variando;32repetições estáticas por modo/coorte perto-longe (768 no total), incluindo a primeira captura e hashes de todas as seguintes.10retornos por modo. Oito rodadas alternadas e duplicadas,128frames/lote em10coortes (960amostras). Capturas dos quatro caminhos de produto, controles de acesso apenas métricas;552PNGs. Não repetir até passar, salvar dados antes de reprovar. Mesmos limites RMSE0,005/p990,02/cobertura0,0005; retorno exato separado de qualidade temporal geral. Compilação/upload/captura fora dos timings e cálculo/consulta dentro.

Checklist prévio: (1) quantização no build, leitura/projeção porfragmento opt-in; (2) controles compartilham mapas/malha, bindings próprios preservam buffer bruto/compacto durante alternância; (3) u32 compilado sem descompressão CPU no frame, escala comum utilizada e hash vinculado à fonte; (4) reaproveitar posição/recíproco da rasterização e gradientes entre mapas; (5) guarda de draw único/fonte, limite de coeficiente e degeneração, sem extrapolar para malhas arbitrárias; (6) I/O/readbacks só no laboratório, tabelas carregadas uma vez; (7) benchmarks sem builds/GPU concorrente, nenhuma thread/dependência nova; (8) handle de variante conserva seu binding, legado desativado corretamente; (9) testar arredondamento/extremos/corrupção/fonte e projeção independente, depois imagem/custo/repetição; (10) manter +4,5MiB e custo restante explícitos, sem promoção automática ou promessa de ganho.

## Compactação e custo da tangente — resultados

**Redução de memória validada; ganho consistente de tempo ainda não demonstrado.** IGP1 usa 4.718.636B em disco (40B de cabeçalho,4B de escala e4B por face), contra9.437.224B do IGT1. Payload de GPU cai de9MiB para4,500004MiB. Compilar/verificar/gravar o compacto, após leitura e validação da fonte, levou0,078s nesta execução. Escala8, erro máximo de coeficiente0,000122074, dentro de meia célula mais arredondamento. Os dados permanecem compactos no upload; `unpack2x16snorm` e multiplicação são custos explícitos do fragmento. JSON aparece apenas em relatório offline.

Os três candidatos geométricos passam138/138comparações,128/128repetições estáticas e10/10retornos cada. Compacto: RMSEmáximo0,00425831,p990,017008,cobertura idêntica. Comparando compacto ao mesmo shader f32, RMSEmáximo0,0000292674,p990,000182681,máximo de canal0,0124937; portanto não é compressão sem perdas ou igualdade de pixels. A projeção pelo fragmento contra a anterior f32 tem RMSEmáximo0,00000445244,p99zero,máximo0,00107667. Nenhum limite foi relaxado. Fórmula de posição do fragmento segue a [especificação WGSL](https://www.w3.org/TR/WGSL/#position-builtin-value); teste numérico confronta a nova expressão com o Jacobiano anterior, complementando o teste pré-existente de raios/planos.

Original:64/128repetições exatas. Nas duas coortes próximas, todas as32capturas posteriores diferem da primeira e ainda há **dois hashes distintos entre essas32**: uma ocorrência divergente na repetição21 de1728 e11 de3456 (índices começam em0). Até162.349pixels float alterados; máximo de canal0,376984. Isso descarta a explicação simples de apenas uma primeira captura diferente nesta sequência. Todos os60retornos de trajetória passam. Runner terminou1, salvou os dados e não foi reexecutado até passar. Os controles índice/guarda e leitura/guarda passam138/138vistas e128/128repetições cada, mas usam `discard`: podem alterar compilação, execução de fragmentos e testes de profundidade. Não isolam o custo puro de `primitive_index`, nem demonstram a causa raiz da instabilidade.

Medições no Apple M4 Pro/Metal:960lotes de128frames,8rodadas alternadas com duplicatas, mesma fonte/mapas/luz. Tempos abaixo são medianas do total porframe, não FPS de gameplay:

| Resolução / câmera | Original | Geom. anterior f32 | Projeção fragmento f32 | Compacto |
|---|---:|---:|---:|---:|
|1728 / perto2,7|0,507893ms|0,563250ms|0,565298ms|0,557174ms|
|1728 / distância6|0,391567ms|0,410363ms|0,421265ms|0,425889ms|
|1728 / distância16|0,380976ms|0,380531ms|0,377956ms|0,374830ms|
|1728 / longe64|0,379265ms|0,371021ms|0,378303ms|0,382002ms|
|1728 / aproxima-afasta|0,407327ms|0,411165ms|0,404904ms|0,419415ms|
|3456 / perto2,7|0,768841ms|0,875541ms|0,875116ms|0,855181ms|
|3456 / distância6|0,536693ms|0,598473ms|0,590925ms|0,582982ms|
|3456 / distância16|0,380952ms|0,379379ms|0,384196ms|0,387286ms|
|3456 / longe64|0,367652ms|0,364851ms|0,370119ms|0,365473ms|
|3456 / aproxima-afasta|0,415536ms|0,439605ms|0,431908ms|0,425937ms|

Perto3456: compacto reduz a mediana2,33% contra geométrico anterior, mas vence só5/8rodadas e as duplicatas variam1,45%/2,92%. Tratar como sinal pequeno, **não ganho robusto**. Continua11,23% mais caro que original, perde8/8rodadas. GPUoriginal/anterior/compacto0,728664/0,832458/0,812625ms; CPU~0,030ms. A projeção mais curta sozinha muda−0,05%,4/8rodadas: a retirada dessas operações não resolve o gargalo neste ensaio. Compacto tem regressões em outras coortes; não adotar globalmente. Controles com guarda perto3456 custam1,104944/1,125945ms (+43,72%/+46,45% contra original), mais que o shader geométrico completo. Não subtrair essas medidas para calcular parcelas de acesso/Jacobiano: os controles perturbam o pipeline.

Fonte+mapas46,934254MiB; com tabela f32 isolada55,934254MiB; com compacta isolada51,434258MiB (**8,05% menos no conjunto isolado**). Durante a comparação ambas as tabelas estão residentes:60,434258MiB, além de targets, pipelines/driver e cópias CPU, que não integram esses totais. Não comparar com90,528MiB do seletor automático como economia. Mesmo número de triângulos nos seis modos; nenhuma nova suíte1×–100× ou economia geométrica atribuída a este passo.

Decisão: manter o compacto como opção de pesquisa, sem mudar shader original ou seletor regional. Próximo isolamento deve observar índice/leitura em um canal diagnóstico sem `discard`, preservando RGB e cobertura e contabilizando a instrumentação, para separar a perturbação da guarda do acesso por face. Confrontar também sequências sem alternar pipeline; não esconder a primeira captura. A compactação fica disponível para futura integração regional, que precisa contabilizar todos os níveis/offsets e passar fidelidade/custo conjuntos. Atmosfera continua adiada.

Reprodução, sem outros benchmarks/builds simultâneos:

```sh
cargo build --locked --release --example rich_relief_probe
# Requer refined.igl, color.irm, normal.irm e source.igt verificados.
target/release/examples/rich_relief_probe --compact-build artifacts/appearance-probe/rich-relief
target/release/examples/rich_relief_probe --compact-run artifacts/appearance-probe/rich-relief
# A execução pode reprovar e ainda assim deve gerar o relatório:
python3 scripts/report_rich_relief_compact.py
python3 scripts/check_rich_relief_compact_images.py # Python com Pillow
node scripts/check_rich_relief_compact_ui.mjs
```

Relatório em `artifacts/appearance-probe/rich-relief/compact.html`; dados completos em `compact-results.json`, resumoGPU/CPU/total/duplicatas/rodadas em `compact-summary.json`, integridade em `compact-verification.json`. Imagens em resolução física, visualização ajustada ou1pixel físico;552PNGs decodificados e40paresPNG de retorno iguais;1.656estadosDOM/DPRsimulados. Inspeção de capturas próximas é complementar aos gates; não equivale a aceite humano/navegador nativo ou prova de estabilidade temporal contínua.

Checklist posterior: (1) quantização offline e projeção/decodificação opt-in porfragmento; (2) mapas/malha únicos, doisbuffers residentes apenas para alternar controles, sem expansão de vértices; (3) codec versionado testa extremos/zero/corrupção/fonte, u32 direto naGPU; (4) posição da rasterização reutilizada, sem ganho robusto por esse ajuste; (5) domínio heightfield/fonte completa/drawúnico explícito, quantização não certifica imagem arbitrária; (6) builds/uploads/readbacks fora dos timings, JSON só diagnóstico; (7) sem dependência/thread nova, benchmark concluído antes dasregressões; (8) variantes conservam seu binding bruto/compacto e smoke legado aprovado; (9)722testesRust/Clippy/fmt e13gruposMCPheadless,552PNGs e1.656estadosDOM/DPRsimulados, falha da referência mantida; (10)4,5MiB extras e regressão próxima11,23% contra original conservados só no laboratório, sem promoção/FPS/aceite humano alegados. O prefixo inicial verify_ do checador offline conflitou com a descoberta MCP; corrigido para check_, sem mudar cenários ou esconder falha.
