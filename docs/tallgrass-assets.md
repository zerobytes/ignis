# Mato alto — continuação EAF-200

Após aprovação explícita da coleção baixa pelo usuário, esta coleção acrescenta
8 tipos e24 variações de mato alto. As fontes e imagens baixas aprovadas são
preservadas; não se substituiu o capim baixo por uma escala maior do mesmo tufo.
Há folhas em nós ao longo das hastes, eixos laterais com folhas alternadas,
espigas volumétricas, plumas, sementes e flores em terminais de ramos.

| Tipo | Estrutura | Altura máxima das três variantes na galeria | Triângulos por tufo |
|---|---|---:|---:|
| Capim alto denso |25 lâminas largas ascendentes |2,17m|1600|
| Capim alto recurvado |26 lâminas longas em arco |1,49m|1664|
| Caniço |5 hastes com folhas alternadas ao longo dos nós |2,82m|3180|
| Junco |20 lâminas basais e4 hastes com espigas cilíndricas |2,72m|1760|
| Plumas altas |22 lâminas e4 pendões com filamentos |2,98m|9328|
| Macega seca |27 lâminas ocres, hastes laterais e sementes |2,33m|12496|
| Mato ramificado |4 hastes, ramos laterais e folhas largas |2,12m|7296|
| Mato florido |4 hastes, ramos folhosos e flores amarelas |1,85m|15440|

Os nomes são descrições autorais de silhueta, não identificações científicas.
As alturas são máximos medidos no bake das amostras de galeria, cuja inserção
fica emY−0,025m. Cada tufo de campo recebe escala0,78..1,20; não se promete uma
altura uniforme. A captura próxima tem câmera emY1,70m, mostrando folhagem acima
dos olhos. Não foi adicionada uma figura humana nem uma escala fotográfica.

## Geração e uso

```sh
python3 scripts/tallgrass_assets.py
python3 scripts/demo_groundcover.py --collection tallgrass
```

Saídas correntes em `artifacts/tallgrass`:

- `review.html`: campo misto, vista próxima, campo só de folhagens e oito cartões.
- `field.scene.json`:650 tufos em14×14m; mistura e passagem por receita seeded.
- `grass.scene.json`:650 tufos altos sem flores ou pendões, incluindo mato ramificado.
- `authoring.scene.json`: fontes editáveis, agrupadas visualmente por variação.
- `gallery.scene.json`:24 tufos como instâncias estáticas.
- `library.scene.json`, `templates.json`, `library-build.json`: bake compartilhado
  e hashes; `report.json`, `verification.json` e `placements.json`: evidências.

`examples/tallgrass-field.recipe.json` controla contagem, área, seed e peso de cada
tipo. Os mesmos controles e limites de plantio da [coleção baixa](groundcover-assets.md)
continuam válidos. Por exemplo, acrescentar mato alto a uma cópia do campo baixo:

```sh
python3 scripts/plant_groundcover.py --collection tallgrass \
  --scene artifacts/groundcover/field.scene.json \
  --output artifacts/tallgrass/layered.scene.json \
  --ground-y -0.03 --prefix high
```

A opção de coleção seleciona a biblioteca, os tipos e a receita correspondentes.
O destino precisa diferir da entrada. Assets e instâncias da coleção baixa são
conservados; os novos assets usam prefixo `tg`, independente de `gc`. Dentro de
cada coleção as cópias compartilham geometria. O candidato passa por LoadScene
nativo antes da publicação atômica do destino por SaveScene.

`scripts/tallgrass_assets.py` define curvas e órgãos. `botany_collections.py`
fornece os parâmetros de apresentação às ferramentas comuns. O compilador Rust
existente constrói as superfícies fechadas; Source/Recipe7 e formatos globais não
mudaram. A folha ainda usa4 amostras longitudinais; a silhueta é facetada quando
vista muito perto. Hastes, folhas e componentes de cor são superfícies individuais
em contato/sobreposição; não se acrescentou união vascular nem física acoplada.

## Custos e verificação

No build macOS arm64 release `e46d5879c672aec9`, a biblioteca alta contém39 fontes
com1878342bytes,60 meshes vegetais e um chão. A biblioteca completa tem84154v e
158294tri incluindo o chão. O campo misto de650 tufos usa1333 instâncias e soma
3197854tri antes do descarte. Só folhagens:825 instâncias/1616822tri. Materiais,
flores e pendões ainda custam por cópia; compartilhamento não implica FPS alto.

A composição com o campo baixo foi carregada/salva nativamente:3050tufos,
5401instâncias e249794tri únicos na biblioteca combinada;7841210tri somados sobre
as instâncias. É uma demonstração de coexistência, não um benchmark de gameplay.
Não se mediu FPS, memória total ou GPU; não há LOD próprio para mato alto.

Verificações realizadas:

- 39 fontes compiladas nativamente; incidência fechada e orientação de arestas
  verificadas nos60 meshes vegetais. Geração canônica reproduzida; outra seed
  muda curvas de todos os componentes. Posicionamento seeded reproduzido.
- Composição nativa conserva campos não relacionados, assets, instâncias e grupos
  anteriores. IDs conflitantes e biblioteca desatualizada recusam sem destino.
- Coleção baixa:57 fontes e manifesto iguais;18 PNGs anteriores byte exatos após
  executar novamente a demonstração com as ferramentas compartilhadas.
- No primeiro save/load da vista alta,1 pixel entre1600000 variou em1 nívelRGB8.
  A diferença está registrada em `initial_roundtrip_comparison`, sem alegar
  igualdade dessa captura. Valores da cena foram idênticos; um segundo par de
  save/load da mesma cena nativa produziu PNG byte exato. A causa da diferença
  mínima inicial não foi estabelecida; não foi atribuída a uma correção da engine.
- Python compilado, links próprios e diff verificados; nenhum Rust mudou, portanto
  não se repetiu a suíte da engine por alteração somente de assets/ferramentas.

Revisão visual cobriu os oito cartões, campo amplo, vista próxima e versão sem
flores/pendões. Plumas/sementes finas podem desaparecer ou cintilar à distância;
interseções de folhas e facetas permanecem visíveis de perto. O usuário aprovou
a coleção alta; a continuidade de [arbustos](shrub-assets.md) é separada.
Plantio continua em altura constante, sem
adaptação automática a terreno, vento, colisão ou integração ao runtime planetário.
Foi usado o executável corrente e preservada a janela humana, sem build novo,
publicação ou push.
