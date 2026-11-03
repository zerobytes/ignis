# Seleção perto/longe e cobertura subpixel — EAF-181

Continuação de [ICB1](collective-geometry-query.md). O pacote continua experimental,
OFF na engine normal. O mundo aprovado e os formatos não mudaram. As APIs novas
são diagnósticas do renderer/driver; não são configurações entregues no MCP/UI.

## O que mudou na seleção

O caminho anterior já continha todas as otimizações nativas. A regressão da rodada
anterior vinha de ativar consultas por pixel onde a rasterização/oclusão existente
era mais barata. A seleção agora tem duas proteções genéricas:

- Limite de tamanho projetado do objeto composto em pixels físicos: entrada8px,
  saída9px para evitar alternância no limiar. Objetos maiores conservam a fonte.
- Proteção da vista quando volumes próximos cruzam a câmera ou quando a soma
  conservadora de suas áreas projetadas ultrapassa metade da área do framebuffer.
  Nesse caso, conserva integralmente o renderer anterior, inclusive os objetos
  pequenos. Isso favorece a oclusão nativa quando geometria próxima domina a vista.

São heurísticas de custo e fallback conservador, não uma previsão universal de
qual caminho será mais rápido. Não há distância fixa, regra por espécie/nome ou
novo teto autoral de objetos. A soma de áreas pode contar sobreposições; isso pode
recusar uma oportunidade de ganho, nunca autoriza apagar conteúdo. Ainda falta
calibração ampla e um certificado visual para habilitar o recurso em produto.

As máscaras CAMERA só são publicadas depois de aprovar o plano inteiro. Uma
recusa tardia não pode deixar peças anteriores sem desenho. Uma prova crítica
na GPU cobre esse risco, incluindo voltar a selecionar após alterar a fonte.
Histerese conserva apenas a escolha do caminho, não altera a geometria/colisão.

## A causa dos pixels de borda

A busca por raio tratava a cobertura como geometria contínua. A reprodução CPU
independente projetou os triângulos originais e variou o arredondamento em tela.
Sem arredondamento reproduziu as duas discrepâncias de1pixel; na grade1/256px,
as18máscaras coincidiram com a referência do M4Pro/Metal. Uma grade1/16px criou
novos erros. O método/script não exige NumPy/Pillow nem receita de subdivisão.

O shader diagnóstico pode usar a mesma grade para cobertura e a regra top-left.
A busca infla seus volumes conservadoramente para não perder contribuições que o
arredondamento desloca. A interpolação permanece em coordenadas não arredondadas.
Isso corrigiu as18máscaras no ensaio GPU inicial, mas ainda não resolveu a cor.
A precisão de rasterização depende da implementação: a grade medida neste Mac
não vira automaticamente uma garantia para outros dispositivos. O modo padrão
do diagnóstico sem flag continua sendo o raio contínuo; a engine normal sequer
instala o pacote. [Contrato de rasterização WebGPU](https://gpuweb.github.io/gpuweb/#rasterization).

Um segundo diagnóstico compara ordem pela superfície mais próxima com ordem pelo
Z convencional aproximado. Superfícies sobrepostas podem trocar de vencedor
quando profundidades distintas caem no mesmo valor representável. A tentativa
por Z não reproduziu todas as escolhas nativas e piorou alguns casos de4px;
não é uma solução aprovada. Ganho na máscara não significa fidelidade completa.

## Reprodução

`target/release/examples/collective_lod_cost` recebe a cena como primeiro argumento.
O pacote é o mesmo `artifacts/collective-lod/object.igcb`.

- Padrão: seleção8/9px com proteção da vista; `--raster-grid 256` ativa cobertura
  medida em tela. Ausência da flag mantém interseções por raio.
- `--max-pixels 0 --no-cost-guard` força a política anterior para investigação.
- `--isolated --quality-only --max-pixels 0 --no-cost-guard --raster-grid 256`
  testa todas as18poses sem mascarar falhas com fallback dos casos de16px.
- `--native-depth-order` escolhe o diagnóstico de Z aproximado, sem garantia nativa.
- `--quality-only` não mede tempos. `--pose N` isola uma vista.
- `python3 scripts/diagnose_collective_coverage.py` reproduz a análise de máscaras
  das capturas isoladas antigas. É um oráculo aproximado independente, não
  certificado geométrico ou contrato portátil da GPU.

## Medições finais — 01/11/2026

Apple M4Pro/Metal, snapshot do renderer/driver `e4b13512b154a71b`, mesmo mundo
20×20km/30milobjetos, 32amostras alternadas aquecidas por variante. Sem
compilação/capturas concorrentes no intervalo medido. Cada par compara o renderer
anterior já otimizado contra ele mesmo com a nova seleção. Valores são draw+espera
GPU, incluindo submissão, não FPS de gameplay nem timestamps GPU. Não comparar
os tempos absolutos entre rodadas diferentes como prova de causa.

| Vista | Pixels físicos | Anterior → seleção com raio ms | Resultado da seleção |
|---|---:|---:|---|

| Solo | 1280×800 | 48,368 → 48,379 | Fonte integral, RGB8exato |
| Solo | 1728×1117 | 52,649 → 52,978 | Fonte integral, RGB8exato |
| Aérea média | 1280×800 | 159,264 → 51,444 | Símbolos ativos, cor ainda falha |
| Aérea média | 1728×1117 | 164,492 → 69,040 | Símbolos ativos, cor ainda falha |
| Aérea ampla | 1280×800 | 225,262 → 46,280 | Símbolos ativos, cor ainda falha |
| Aérea ampla | 1728×1117 | 228,135 → 62,447 | Símbolos ativos, cor ainda falha |

No solo, a diferença ficou em+0,02%/+0,62%, contra a regressão anterior de12–21%.
Isso elimina aquela regressão grande nestas duas amostras; não prova ganho ou
regressão zero em toda cena/hardware. Nas aéreas, o raio custou58–79%menos tempo,
mas conserva suas falhas de cobertura/cor: não é uma opção visual aprovada.

| Vista | Pixels físicos | Anterior → grade256 com seleção ms | RGB8máx |
|---|---:|---:|---:|
| Solo | 1280×800 | 54,103 → 53,816 | 0 |
| Solo | 1728×1117 | 58,789 → 59,148 | 0 |
| Aérea média | 1280×800 | 182,303 → 88,318 | 24 |
| Aérea média | 1728×1117 | 168,751 → 129,690 | 24 |
| Aérea ampla | 1280×800 | 225,400 → 115,204 | 15 |
| Aérea ampla | 1728×1117 | 228,995 → 123,895 | 16 |

A grade corrigiu as18máscaras isoladas, inclusive os casos não vazios de16/4px;
nenhuma troca de caminho foi usada para esconder os dois erros de borda. Cores
continuam falhando: máximo109RGB8 isolado e15–24RGB8 nas aéreas. A tentativa de Z
aproximado também falhou (até108RGB8 em4px). O diagnóstico confirmou trocas entre
superfícies/material de madeira e folhagem, além de diferenças de iluminação;
a precisão específica que determina cada vencedor ainda não foi certificada.

O custo da grade é maior: nas aéreas o ganho ficou em23–52%, em vez de58–79% no
raio. Não atribuímos o ganho maior do raio ao modo de borda corrigida. A variante
de gradeprojeta candidatos em tela durante a busca, acrescentando operações;
ainda não demonstramos um caminho que junte custo mínimo e imagem aprovada.

Oito frames adicionais com câmera/luz/64objetos em movimento e resize alternado
ficaram RGB8exatos **entre a grade com e sem o descarte por depthseed**, mantendo
cerca de10mil símbolos ativos. Esse ensaio forçou `--max-pixels0 --no-cost-guard`
para não contar fallback de solo como prova da consulta. Não é equivalência com
geometria nativa nem certificado de estabilidade temporal contra a referência.

## Dados, validação e limites

O pacote ICB1 não mudou:1.885.604B no disco,1.885.472B em nós/faces GPU. As novas
escolhas ficam em cache CPU por objeto; o controle de grade/ordem ocupa slots do
registro GPU existente de80B/instância, sem nova imagem ou readback no frame.
O relatório `selection_cache_bytes` expõe capacidades desse cache. FonteCPU,
representação GPU nativa aquecida e buffers de símbolos continuam residentes;
não há redução nova de RAM total demonstrada por esta seleção.

Build release, Clippyalltargets-Dwarnings, formatação própria e os dois testes
críticos de collective_lod passaram. O teste GPU exerceu raio e grade realmente
selecionados, recusa tardia do plano sem perder fonte, instalação inválida e
edição de fonte. Os18ensaios isolados e oito de movimento são validação direta
do driver. Os testes/arte posteriores de personagens e botânica pertencem às
outras frentes; não são inferidos deste snapshot. Fonte181 foi congelada antes
da devolução de Cargo/GPU. Janela humana e cena aprovada preservadas.

Dados locais: `artifacts/collective-lod/coverage-probe/projection.json`,
`raster-isolated/isolated.json` (Zaproximado inicial), `nearest-isolated/isolated.json`,
`selected-final/world.json`, `selected-ray/world.json` e `raster-motion/motion.json`.
A rodada preliminar `raster-native-order` ocorreu com compilação concorrente e
seus tempos não sustentam conclusão de performance. PNGs/logs seguem nas mesmas
pastas ignoradas. `--pose`/`--quality-only` permitem repetir diagnósticos menores.

EAF-181 segue em andamento e desligado no mundo aprovado. Próximo contrato:
resolver a escolha de superfície/cor sem reintroduzir os erros de cobertura e
reduzir as projeções repetidas da busca; depois ampliar a seleção por custo em
outros objetos/dispositivos e testar a transição temporal contra a referência.
