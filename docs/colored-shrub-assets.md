# Arbustos amarelos, vinho e roxos — EAF-204

Extensão da coleção de arbustos aprovada pelo usuário. Quatro perfis guiados por
fotografias e descrições botânicas, com três variações seeded por perfil. As
coleções de gramas, mato alto e arbustos anteriores permanecem preservadas.

| Perfil | Referência real | Forma construída |
|---|---|---|
| Pingo-de-ouro | [Duranta Gold Mound, UF/IFAS](https://gardeningsolutions.ifas.ufl.edu/plants/trees-and-shrubs/shrubs/golden-dewdrop/) | Corpo baixo/denso, folhas pequenas pontudas amarelo-douradas, terminais claros |
| Berberis vinho | [Crimson Pygmy, NC State](https://plants.ces.ncsu.edu/plants/berberis-thunbergii-crimson-pygmy/) | Ramos finos, lâminas pequenas obovadas vinho, brotação vermelha menor |
| Loropétalo roxo | [Loropetalum rubrum, NC State](https://plants.ces.ncsu.edu/plants/loropetalum-chinense-var-rubrum/) | Corpo mais largo/arqueado, folhas ovais pontudas alternadas, roxo-vinho |
| Cotinus roxo | [Royal Purple, NC State](https://plants.ces.ncsu.edu/plants/cotinus-coggygria-royal-purple/) | Corpo mais alto/aberto, folhas largas arredondadas, roxo/ameixa e brotação vinho |

O porte e as cores são escolhas autorais: não são medidas das fotos, alturas
adultas calibradas ou reflectância obtida das imagens. Cotinus representa um
exemplar arbustivo menor, não o porte máximo da espécie. A foto de Loropetalum
não confirma um cultivar específico. Esta é uma coleção vegetativa sem flores;
flores e panículas das referências não foram reproduzidas.

## Estrutura e uso

`scripts/colored_shrub_assets.py` especializa o gerador de arbustos com curvas
reais de hastes, ramos e raminhos. Comprimento/largura, contorno, ocupação, porte,
distribuição e paleta diferem entre perfis. As brotações terminais têm lâminas
menores, enraizadas nos mesmos pontos de inserção, e um componente de cor próprio.
A alternância desloca inserções ao longo do ramo em Loropetalum/Cotinus. Não é
uma simulação de desenvolvimento ou uma reconstrução nodal botânica exata.

Cada variação tem duas fontes Source/Recipe7: corpo com madeira/folhas maduras
nos dois canais nativos e brotação com outra cor. São24 fontes para12 variações,
sem mudar quotas, compilador, renderer ou formatos. Eixos e folhas são fechados
individualmente e se sobrepõem; não há união vascular soldada entre os órgãos.

```sh
python3 scripts/colored_shrub_assets.py
python3 scripts/demo_groundcover.py --collection colored-shrubs
python3 scripts/review_colored_shrubs.py --leaf-details
python3 scripts/plant_groundcover.py --collection colored-shrubs \
  --scene artifacts/groundcover/field.scene.json \
  --output artifacts/colored-shrubs/with-grass.scene.json \
  --ground-y -0.03 --prefix colored
```

Fontes/manifesto: `assets/botany/colored-shrubs`. Seed padrão10601; quantidade,
área e mistura em `examples/colored-shrubs-field.recipe.json`. Regenerar substitui
as fontes: preserve esculturas em outro destino. As fontes usam Vegetação/MCP
existentes; exportação e plantio são preparados pelos scripts. Campos exportados
não se atualizam automaticamente após editar a fonte. Plantio em altura constante,
sem adaptação ao terreno/planeta, vento ou física de gameplay.

Saídas correntes em `artifacts/colored-shrubs`:

- `review.html`, `gallery.png` e capturas de cada família com três variações.
- `authoring.scene.json`: fontes editáveis; `gallery.scene.json`: cópias estáticas.
- `field.scene.json`/`field.png`:48 indivíduos mistos numa região12×12m.
- `library.scene.json`, `templates.json`, `library-build.json`: bake e hashes.
- `structure-{front,back,top}.png`: revisão neutra de volume/vãos.
- `*-leaf.png`: folha madura à esquerda e brotação à direita, mesma escala local;
  transformação rígida para enquadramento, sem alterar o tamanho das lâminas.
- `references.html`: fotos, créditos/links, recortes reais e candidatos nativos.
  Fotos/crops são estudos locais, não texturas nem arquivos incorporados aos assets.
- `report.json`, `structure-report.json`, `verification.json`: evidências.
- `with-grass.scene.json`/`with-grass.png`: composição com o campo de gramas
  aprovado, preservando seu conteúdo anterior.

## Cobertura e diagnóstico visual

A [análise e lista construtiva prévias](references/botany/colored-shrubs-analysis.md)
foram entregues antes de gerar geometria. Os corpos neutros foram revistos em
frente, costas e topo antes de julgar as cores. Depois foram abertos os quatro
conjuntos coloridos, macros e recortes dos candidatos junto às fotos. Câmeras,
escala e luz são independentes; não se calcula percentual de fidelidade por RGB.

| Itens da lista | Resultado e evidência nativa | Limite observado |
|---|---|---|
| D01..D03 | Corpo baixo fino/denso e amarelo com terminais claros; `pingo_ouro.png`/macro | Contorno facetado, sem dentes finos; ausência do verde interno da foto |
| B01..B04 | Porte compacto, folha obovada pequena, vinho/terminal vermelho e madeira avermelhada; `berberis_vinho.png`/macro | Espinhos e nodos precisos omitidos; agrupamento foliar aproximado |
| L01..L03 | Corpo aberto, folha pontuda alongada, inserções alternadas, paleta vinho-roxa; `loropetalo_roxo.png`/macro | Silhueta autoral; sem identificação de cultivar pela foto |
| C01..C03 | Ramos mais altos, folha ampla arredondada, roxo distinto do Berberis; `cotinus_roxo.png`/macro | Folha ainda poligonal; porte/copa não copiam o indivíduo inteiro |
| L04/C04 | Dobra longitudinal discreta da lâmina | Nervuras secundárias e pigmentação rosada das nervuras não implementadas; item apenas parcial |

A primeira paleta de Loropetalum/Cotinus estava excessivamente violeta. Foi
reduzida a saturação antes da captura final, conservando a geometria. Macros
continuam mostrando bordas facetadas e ausência de nervuras finas. As imagens
guiam cor, contorno e organização; não há fotogrametria ou réplica exata.

## Validação e custo

Capturas finais, macros e conferência estrutural: release macOS arm64
`177ca02466faba24`. A consulta inicial do schema usou `e46d5879c672aec9`; houve
substituições do executável por trabalho paralelo durante a entrega. Este trabalho
não compilou nem copiou executáveis. Relatórios identificam seus builds exatos.

As24 fontes ocupam9247645bytes. A biblioteca exportada tem36 meshes vegetais e
um chão,645760v/1243770tri únicos. O campo48 tem145 instâncias e5484790tri somados
antes do descarte. Detalhe por indivíduo:

| Perfil | Triângulos, três variações |
|---|---|
| Pingo-de-ouro |174564 /175084 /172900|
| Berberis |110040 /109104 /110040|
| Loropétalo |82792 /84144 /80920|
| Cotinus |48348 /47916 /47916|

Todas as partes nativas conferidas são fechadas/orientadas. Posicionamento
reproduzível e primeiro save/load final com PNG exato. Conferência adicional de
regeneração, preservação das coleções anteriores e composição em `verification.json`.
As24 fontes regeneraram iguais e outra seed mudou as curvas de todos os
componentes. As57/39/42 fontes anteriores continuaram iguais, assim como seus
18/19/17 PNGs existentes; isso não é uma nova comparação de pixels entre builds.

A composição conserva campos, assets, instâncias e grupos anteriores, acrescenta
48 arbustos e totaliza2448 objetos/4213 instâncias,1335270tri únicos e10128146tri
somados por instância. A captura composta usou `e074e577dfc4f065`, novamente
substituído pelo trabalho paralelo. O SaveScene nativo migrou somente a versão
global109→110; a igualdade dos demais campos anteriores foi conferida. Este
incremento não implementou essa migração nem alterou contratos globais.

Compartilhamento evita duplicar geometria por cópia, mas os assets são detalhados:
não há LOD novo, benchmark de FPS, custo GPU/memória total ou desempenho garantido
para grandes populações. Nenhum Rust foi alterado nesta entrega. Aceite artístico
das novas folhagens permanece separado da validação técnica.
