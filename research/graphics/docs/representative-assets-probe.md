# Assets representativos — experimento isolado

EAF-72. Comparar rocha detalhada, baú multimaterial e árvore estática autorados proceduralmente no Blender. Fonte editável e exportação indexada restrita ao laboratório; não constitui importador Blender/glTF da engine. Posições normalizadas por raio, câmera/luz e resolução comuns. Materiais explícitos com resposta GGX direta, cores por vértice; sem texturas, sombras projetadas, GI, transmissão ou vento. Essas restrições são iguais na referência e no bake, mas limitam a extrapolação para produção.

Escolha: reutilizar atlas24 (128², 24 vistas × 24 luzes, elevações55°/35°), altura única e compactação sem quantização. Medir limites da técnica atual antes de trocar representação. Alternativas de múltiplas camadas, amostragem de elevação e resposta fatorada ficam para decisão posterior. Malha indexada evita comparar com vértices artificialmente duplicados. Mesma implementação de material usada no bake e na referência; material não depende da cor para escolher rugosidade.

## Checklist antes de implementar

1. Geração/exportação/bake somente offline; consulta por frame reutiliza dados.
2. Buffers GPU persistidos; alocações de captura/JSON só no harness de medição.
3. Vértices40bytes e índicesu32, IDs de material explícitos; limites e finitude validados antes do upload.
4. Reutilizar seleção, reprojeção e compactação existentes; sem cópia de renderer.
5. Três assets, atlas24 fixo, percursos finitos; medir por asset e distância.
6. Timestamp GPU intercalado, frame serial separado; readback somente qualidade/instrumentação.
7. Execução headless serial da GPU, sem threads/locks novos nem disputa entre benchmarks.
8. Assinatura inclui vértices, índices e shader; arquivos por asset em caminhos estáveis.
9. Comparar referência integrada, geometria amostrada igual e atlas original/compacto; determinismo/360°/luz e bordas. Sem mudança de Scene/MCP/runtime.
10. Manter explicitamente custo de memória, filtragem e erro de camada única; redução de triângulos não garante ganho nem fidelidade.

## Reprodução e artefatos

```sh
/Applications/Blender.app/Contents/MacOS/Blender -b --factory-startup --python-exit-code 1 -P scripts/build_probe_assets.py
cargo run --locked --release --example appearance_probe -- --bake-assets
cargo run --locked --release --example appearance_probe -- --assets
# Python com Pillow; o runtime local já instalado foi usado, sem instalar dependências.
python3 scripts/verify_representative_assets.py
python3 scripts/report_representative_assets.py
node scripts/verify_representative_report.mjs
```

`--assets-smoke` usa18 poses por asset; não substitui a varredura. Exportação restrita: vértices position/normal/RGB linear/material (40bytes), índicesu32 little-endian e metadados; materiais de perfis fixos compartilhados com representative.wgsl. Não é importador de materiais arbitrários de arquivos .blend. Direção de observação usada no material é comum ao objeto, inclusive na referência. Rerodar o gerador recria os três modelos; não sobrescrever edições manuais sem preservá-las em fontes próprias.

Paths estáveis em `artifacts/appearance-probe/assets/`: fonte `assets.blend`, revisões front/side/above/low/wire, geometria/atlas/manifests/report.json por asset, `summary.json`, `verification.json`, relatório `index.html`. Os atlas densos permanecem controles científicos da compactação, não versões históricas. A instrumentação conserva as três representações simultaneamente; os números de memória comparam o payload necessário em cada opção, não o RSS do processo. Não incluem targets, driver, uniformes pequenos nem staging temporário de build/upload.

## Método e resultados — 26/10/2026

Blender5.0.1, Apple M4 Pro/Metal. Assets normalizados ao raio1,22 ao redor do centro; nenhuma tesselação artificial no baú para forçar vantagem do atlas. Mesmas poses e materiais em todos os métodos por asset.114 poses por asset:21 por câmera/luz/ambos/próximo/zoom e9 na fronteira360°. Distâncias12 (longe),4 (perto), percurso4→16→4; elevações fixas.342 poses e3.078 PNGs. Referência64spp, imagens candidatas16spp e1spp, timing exclusivamente1spp a256². Dois controles64→256spp por asset quantificam a convergência; referência não é uma integral exata.

A instrumentação inicial de passes muito curtos apresentou variação comparável ao suposto ganho. A rodada final mede9 blocos intercalados de32 desenhos por método, limpando cor/profundidade em cada desenho, dividindo o intervalo GPU do bloco por32. São28 poses cronometradas por asset,84 no total; dois dos quatro métodos são a mesma geometria completa. Warm-cache de um objeto isolado, sem simulação, TAA, streaming ou jogo AAA. Não comparar diretamente estes tempos com a instrumentação anterior para declarar evolução de performance.

| Asset | Triângulos | Malha indexada | Atlas compacto | Compacto/malha | GPU compacto/completo longe | GPU compacto/completo perto |
|---|---:|---:|---:|---:|---:|---:|
| Pedra |81.920|2,500MiB|35,584MiB|14,23×|0,989×|0,520×|
| Baú |3.472|0,312MiB|33,053MiB|106,08×|1,053×|1,017×|
| Árvore |20.292|1,502MiB|18,428MiB|12,27×|0,995×|0,639×|

Todos os atlas originais:75MiB. Valores GPU são medianas das razões por pose nos percursos ambos/próximo, não razão de medianas nem ganho de FPS. Controle completo/completo: longe1,015/1,053/0,982; perto0,875/0,965/0,823. Logo diferenças pequenas são inconclusivas e mesmo os percentuais maiores têm variação relevante. As razões perto de pedra/árvore ficaram abaixo de1 em todas as cinco poses medidas; isso não estabelece precisão percentual nem ganho no frame de gameplay. Medianas de tempo serial ficaram próximas de1 (0,988/0,993/0,989 perto): nenhum ganho relevante de frame completo demonstrado.

| Qualidade | Pedra | Baú | Árvore |
|---|---:|---:|---:|
| RMSE compacto1spp / longe |0,00709|0,01382|0,01373|
| RMSE compacto1spp / perto |0,02764|0,03983|0,03301|
| RMSE completo1spp / perto |0,00835|0,03688|0,01519|
| Diferença de cobertura relativa16spp / longe |1,43%|2,61%|9,67%|
| Diferença de cobertura relativa16spp / perto |5,83%|9,41%|25,68%|

RMSE é RGB linear sobre a união de cobertura, não uma porcentagem de qualidade visual. A geometria1spp sofre aliasing; o atlas pré-filtrado pode ter RMSE menor longe sem recuperar informação geométrica. A UI permite comparar também16spp e sempre conserva a referência64spp. A diferença de cobertura é a soma do erro absoluto de alpha dividida pela área coberta da referência; não é “percentual de folhas removidas”. Resíduo temporal é diferença de erros em coordenadas de tela, não avaliação perceptual com optical flow.

Inspeção das imagens próximas: perda de microrelevo da pedra; bordas/reflexos das ferragens do baú suavizados/deslocados; folhas e vazios da copa se misturam. Fonte Blender e comparação da engine inspecionadas visualmente; não equivalem a aceite humano. Conclusão: pedra densa opaca é a melhor candidata para continuar a pesquisa, mas este atlas ainda troca muita memória e detalhe por um ganho restrito. Baú deve manter geometria neste recorte. Árvore requer representação melhor de visibilidade/camadas antes de justificar uso próximo. Nenhum dos três comprova bom equilíbrio universal de memória/qualidade/performance.

## Integridade, linhas e validação

Pack verifica todos os texels e halos exatamente antes de persistir. Assinatura agora inclui índices e shader específico, além dos dados e bake anteriores. Malhas inválidas, material misto na mesma face, índices fora do buffer, normais inválidas e valores não finitos são rejeitados antes de upload. Proxy limitado comparado com fullscreen:342 pares idênticos. Repetição, retorno após mudança de luz e equivalência360° passaram em cada asset.

Imagens filtradas não são bit a bit iguais apesar dos texels preservados. RMSE máximo introduzido16spp:0,00020116/0,00005471/0,00015499. Nos PNGs16spp, apenas13/6/17 pixels em toda a varredura de cada asset diferiram mais de1 nível de canal; máximos6/6/8 níveis. A1spp,10/16/12 pixels ultrapassaram1 nível, máximos6/46/54 (descontinuidades/reflexos pontuais). Maior sequência vertical acima de1 nível:1pixel em ambos os modos para todos os assets. Nenhuma faixa vertical introduzida pela compactação detectada neste conjunto; não prometer ausência universal de artefatos da reprojeção.

598 testes Rust release/all-targets, fmt e Clippy com-D warnings passaram. Smoke legado da compactação15 poses e verificador completo dos PNGs anteriores passaram. Runner MCP: render/replay_inspection/selection,3 cenários/13 grupos headless, passou. Descoberta do runner corrigida para separar explicitamente os dois verificadores de relatório offline dos cenários MCP; nenhum cenário de gameplay foi removido.

Interface:684 combinações pose/modo de amostragem,4.362 carregamentos e4.356 desenhos, reprodução, wrap, zoom e descarte de cargas obsoletas passaram no harness que executa o JavaScript real. Inspeção nativa de file:// foi bloqueada pela política do navegador; não houve workaround. Pedido de abertura no painel retornou queued, sem comprovar exibição. Revisão humana permanece pendente. Fonte .blend reaberta headless:três meshes e materiais preservados.

## Checklist após implementação

1. **Adequado:** export/bake/pack só em comandos offline; consulta/render reutilizam pipeline e buffers.
2. **Adequado:** índices reduzem dados da referência; CPU guarda imagens só na instrumentação. Captura continua alocando targets/readback e arrays, fora do timingGPU.
3. **Corrigido:** material explícito, normal normalizada por fragmento e índices verificados. Payload inclui os índicesu32 e descritores compactos.
4. **Adequado:** mesma implementação de Lab, materiais de bake/referência e compactação; shader novo só no laboratório. Sem dependência Rust nova.
5. **Adequado ao ensaio:** limites1M vértices/triângulos na entrada,24×24 atlas,342 poses; não prova população/streaming.
6. **Corrigido:** blocos GPU limitados reduzem efeito de passes curtos; controle duplicado revela variação residual. Render targets e bake não incluídos no payload comparado.
7. **Adequado:** GPU serial/headless; sem threads/locks novos. Benchmarks separados de regressãoGPU e autoriaBlender.
8. **Corrigido:** assinatura incorpora índices/shader específico; paths por asset; fonte alterada exige bake explícito. Metadados de atlas indexado contam índices/triângulos reais.
9. **Validado no escopo:** 342 pares bounded/fullscreen, integridade, erros, invariância, testes598 e MCP13 grupos. Nenhuma mudança de contrato Scene/Command/RenderFrame/MCP.
10. **Mantido com motivo:** camada única, memória alta, domínio angular restrito e materiais sem texturas/sombras/vento; medir o limite antes de decidir mudança estrutural. Tempos individuais, não promessa de FPS.
