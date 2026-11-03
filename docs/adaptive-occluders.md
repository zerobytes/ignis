# Orçamento adaptativo de oclusores — EAF-181

O terreno do mundo de 20×20 km tem peças de 566.048 triângulos. O orçamento fixo de 262.144 triângulos do pré-passe impedia essas peças de servirem como oclusores, mesmo com centenas de milhões de triângulos de instâncias por frame. Isso era uma decisão de custo do pré-passe, não um limite de objetos ou de renderização.

A política nova usa `B = max(262144, W/64)`, sendo `W` o trabalho de triângulos instanciados selecionado antes da oclusão GPU. Quando W supera 16.777.216, um oclusor também pode passar pelo critério de custo se tiver até W/256 triângulos. Permanecem os critérios de área e tamanho em pixels físicos, material opaco, buffers do dispositivo e orçamento de 128 oclusores. Casos menores mantêm o critério anterior. São heurísticas de rentabilidade, não limites da cena ou garantia universal de ganho.

Apenas a escolha dos oclusores muda. Usamos os vértices e índices já residentes da geometria do frame atual, com os mesmos passes conservadores. Não simplificamos a superfície de profundidade nem usamos profundidade histórica. Colisão, fonte, materiais e luz permanecem iguais. Não há novo formato ou configuração de gameplay. `set_adaptive_occluders` é um controle diagnóstico A/B do renderer; orçamento e custo aparecem em `renderer.info` e `InspectViewportPerformance`.

## Comparação

macOS / Metal / Apple M4 Pro, build `aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|4ba6d5888f239b6f`. Mesma cena de 30 mil árvores, câmera e luz por caso, 16 amostras por variante em blocos alternados aquecidos. Tempos incluem CPU submit e espera de término da GPU; não incluem PNG, UI ou simulação e não são FPS de gameplay. A GPU pode executar durante o submit; suas duas parcelas não são timestamps GPU.

| Vista | Pixels físicos | Fixo ms | Adaptativo ms | Ganho | Tri compartilhados GPU antes→depois | RGB8 máx |
|---|---:|---:|---:|---:|---:|---:|
| Solo / primeira pessoa | 1280×800 | 90.974 | 48.463 | 1.88× | 209,969,688→74,305,776 | 0 |
| Solo / primeira pessoa | 1728×1117 | 92.530 | 48.724 | 1.90× | 203,356,896→64,488,836 | 0 |
| Aérea média | 1280×800 | 170.920 | 154.099 | 1.11× | 446,737,532→386,578,128 | 0 |
| Aérea média | 1728×1117 | 175.286 | 155.539 | 1.13× | 442,902,344→374,581,580 | 0 |
| Aérea ampla | 1280×800 | 217.475 | 214.280 | 1.01× | 589,137,636→575,729,800 | 0 |
| Aérea ampla | 1728×1117 | 220.148 | 217.289 | 1.01× | 586,885,224→570,385,052 | 0 |

As seis imagens estáticas são idênticas. A redução de tempo no solo foi cerca de 47%; a vista aérea ampla ganhou apenas 1–2%. Não há promessa de 60 FPS: ainda sobra muita geometria visível ou conservadoramente mantida. O readback de sobreviventes foi diagnóstico separado dos tempos e conta triângulos instanciados submetidos. Geometria legada e pré-passe entram no tempo total, mas não nesse contador. Os buffers de oclusão são os mesmos, sem cópia adicional de meshes; no solo o trabalho extra de profundidade passa de aproximadamente 262 mil para 2,13 milhões de triângulos.

As câmeras aéreas atuais permanecem na área visível do renderer. Os panoramas 3×/5× do primeiro ensaio de LOD esbarravam no far plane de 30 km; não servem como prova de detalhe distante. Remover esse limite e avaliar profundidade reversa no passe principal continua uma etapa separada. O novo orçamento preserva os guardas conservadores de empate do depth convencional.

## Movimento e repetibilidade

`artifacts/adaptive-occluders/motion.json` compara oito poses contra oclusão GPU desligada: câmera e luz mudando, um oclusor e 16 instâncias movidos no próprio frame. A primeira pose fica cerca de 20 cm abaixo da superfície, exercitando corte e câmera dentro de bounds. Movimento é entrada de renderer, não teste de física de terreno móvel.

Nesse caso extremo apareceu inicialmente uma diferença de 1 unidade RGB8 em um único pixel. Repetir a própria referência sem oclusão também produz essa mesma variação. O harness guarda seis repetições da referência, antes e depois do caminho otimizado, e exige que este permaneça dentro do intervalo observado por pixel/canal. Registra separadamente igualdade exata, pixels instáveis e erro; não transforma repetibilidade em certificado visual. Na execução isolada, sete poses foram exatas e a pose rente ao terreno teve um único pixel / 1 unidade de variação na própria referência. Na repetição final completa, as oito poses foram exatas e estáveis; nenhum pixel otimizado ficou fora do intervalo observado em nenhuma das duas execuções. Uma tentativa de excluir oclusores com bounds cruzando o plano próximo não eliminou essa variação da referência e foi retirada; não se atribui causa ao clipping sem evidência.

Evidências: `comparison.json`, `motion.json` e PNGs em `artifacts/adaptive-occluders/`. Reproduzir com:

```sh
cargo run --locked --release --example automatic_lod_cost -- artifacts/expanded-world/world.scene.json --world-player-view --adaptive-occluders
```

## Revisão de custo e limites

Poucas operações inteiras por frame decidem o orçamento; mesmos buffers GPU, sem duplicar meshes, serializar dados, fazer hash ou criar cópias por pixel. Vetores de oclusores reutilizados e limitados pelo orçamento de trabalho. Os oráculos de imagem cobrem a falha de integração material relevante; não foram criados testes que apenas repetem as constantes da heurística. Clippy release de todos os alvos e formatação dos arquivos deste incremento aprovados. O fmt global ainda aponta duas formatações anteriores de EAF-187, comunicadas ao chat responsável.

O próximo ganho precisa atacar as copas visíveis, com cobertura coletiva / impostors preparados em build time, e perfilar a parcela GPU. O protótipo IGL1 permanece opcional e desligado por padrão; não houve ganho próprio dele na floresta. EAF-181 continua em andamento.
