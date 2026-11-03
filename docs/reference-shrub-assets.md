# Arbustos de deserto e topiarias — EAF-204

Coleção adicional guiada pelas sete imagens fornecidas pelo usuário, preservando
as quatro bibliotecas anteriores. São oito perfis com três variações seeded cada;
a primeira imagem origina dois hábitos distintos. Nomes são descritivos e não
identificações de espécies. A [análise pré-construtiva](references/botany/reference-shrubs-analysis.md)
contém inventário, recortes reais, revisitas, hipóteses e lista com IDs A..H.

| Perfil | Referência / itens | Diferença construída |
|---|---|---|
| `xerico_cinza` | R1/A1..A3 | Moita mais alta, hastes basais abertas, muitos raminhos nus e folhas muito pequenas esparsas |
| `xerico_baixo` | R1/B1..B2 | Vários lobos baixos sobrepostos e folhagem fina verde-acinzentada |
| `almofada_clara` | R2/C1..C3 | Corpo lobado de ramagem fina, base nua e pequenas cabeças claras em hastes próprias |
| `duna_oval` | R3/D1..D2 | Base multi-haste exposta, copa oval de folhas maiores pontudas e brotação terminal |
| `duna_agulha` | R4/E1..E3 | Lâminas longas estreitas, direção ascendente e textura fina compacta |
| `esfera_miuda` | R5/F1..F3 | Envelope esférico podado, folha pequena e muitas ramificações, pontas claras |
| `esfera_folhosa` | R6/G1..G3 | Folhas maiores, contorno arredondado irregular e brotos claros |
| `vertical_denso` | R7/H1..H2 | Copa alta estreita e muitos eixos ascendentes com folhas pequenas |

## Matemática e fontes editáveis

`scripts/reference_shrub_assets.py` usa o mesmo compilador nativo existente.
Hastes basais sustentam ramos principais; estes sustentam raminhos e seus grupos
de lâminas. Os pontos de inserção são amostrados nas curvas reais, com direção
do ramo, lateral local e viés vertical. Comprimento/largura, contorno, brotação,
ocupação, quantidade de ramificações e distribuição espacial variam por perfil.

Destinos dos ramos ocupam envelopes elipsoidais, com assimetria seeded e centros
próximos para moitas lobadas. Topiaria usa envelope de crescimento limitado:
folhas e curvas ficam próximas do contorno autoral, com pequenas saliências de
largura. Não existe esfera opaca de preenchimento nem clipping pela câmera.
Brotações claras são menores e ficam nos terminais. As cabeças de R2 são volumes
curtos discretos sustentados por pequenos pedúnculos; a imagem não resolve
flor/fruto, pétalas ou anatomia interna, que não foram inventados.

As48 fontes Source/Recipe7 cabem na quota64 existente. Cada variação tem corpo
(madeira + folhas em dois canais nativos) e brotação, ou corpo e cabeças claras.
Superfícies são fechadas individualmente e se tocam/sobrepõem. O grafo usado
pelo gerador para posicionar órgãos não é uma pele vascular soldada nem uma
hierarquia dinâmica de poda entre componentes. Não há Rust/formato novo.

Escalas em metros são escolhas autorais. Não se mediram dimensões pelas fotos,
idade ou crescimento de espécies; o perfil seco não é simulação de seca ou
senescência. Sementes e regras reproduzem curvas no ambiente corrente, sem um
contrato de igualdade entre versões arbitrárias do gerador.

## Preparar e plantar

```sh
python3 scripts/reference_shrub_assets.py
python3 scripts/review_reference_shrubs.py
python3 scripts/demo_groundcover.py --collection reference-shrubs
python3 scripts/present_reference_shrubs.py
python3 scripts/plant_groundcover.py --collection reference-shrubs \
  --scene artifacts/groundcover/field.scene.json \
  --output artifacts/reference-shrubs/with-grass.scene.json \
  --ground-y -0.03 --prefix refbush
```

Fontes/manifesto em `assets/botany/reference-shrubs`; seed padrão11701. Regenerar
substitui explicitamente essas fontes. Preserve esculturas em outro destino.
O painel Vegetação/MCP existentes editam as fontes. O povoamento é preparado
por scripts; campos estáticos exportados não se atualizam automaticamente após
uma edição da fonte. Plantio é plano, sem encaixe em dunas, solo ou planeta.

Receitas em `examples/reference-shrubs-field.recipe.json` (64 indivíduos mistos)
e `examples/reference-shrubs-desert.recipe.json` (56 das cinco formas desérticas).
Pesos0 excluem perfis. São cópias que compartilham geometria da biblioteca; não
se geram folhas novamente por instância. A saída do plantio difere da entrada,
preserva o conteúdo anterior e valida via Load/SaveScene nativos.

Saídas correntes em `artifacts/reference-shrubs`, substituídas nos mesmos caminhos:

- `review.html`, `gallery.png`, oito cartões de família com três variações.
- `authoring.scene.json`:48 fontes editáveis; `library.scene.json` e
  `templates.json`: bake estático compartilhado e seus templates/hashes.
- `field.scene.json`/`field.png` e `close.png`: plantio misto e aproximação.
- `structure-{front,back,top}.png`: oito primeiros exemplares em material neutro.
- `*-detail.png`/`*-crop.png`: corpo individual e recorte real do candidato.
- `*-macro.png`: folha madura/brotação em orientação local rígida sem escala,
  ou grupo de três cabeças trasladado preservando relações; não é recorte de
  fidelidade da fotografia. A câmera própria facilita ler a geometria nativa.
- `desert.scene.json`/`desert.png`: plantio em piso arenoso plano, sem reconstruir
  as dunas das fotos. `topiary-row.scene.json`/`topiary-row.png`: seis topiarias
  com três variantes compartilhadas em fileira.
- `references.html`: fotos recebidas, crops e candidatos lado a lado; fotos com
  marcas/créditos preservados apenas nos estudos ignorados, fora dos assets/Git.
- `report.json`, `structure-report.json`, `presentation-report.json`,
  `verification.json`: contagens, câmeras, crops e verificações.

## Revisão e limitações visuais

Os corpos neutros foram abertos em frente/costas/topo. A primeira distribuição
de ramos deixava um vazio polar nas copas densas; mover só as pontas não preencheu
toda a faixa. Foram acrescentados raminhos superiores reais distribuídos pelo
disco polar. As novas vistas mostram cobertura também no topo. Não se acrescentou
uma tampa ou um sólido substituto. As moitas baixas ganharam ramos/folhas e R2
ganhou cabeças em dois pontos dos raminhos, pois a primeira versão estava esparsa.

Os primeiros cartões coloridos ainda mostravam A/B/C transparentes demais,
sem a trama de ramos finos das fotos. A passou a ter hastes principais ascendentes
e mais abertas desde a base; A/B/C ganharam uma ordem adicional de divisões finas,
e C recebeu cabeças em parte dessas divisões. A geometria foi novamente conferida
em material neutro e três vistas antes de repetir os cartões finais.

A revisão próxima de F/G/H ainda mostrava bandas transparentes: muitas lâminas
estavam quase de perfil para uma câmera lateral. O campo `leaf_roll` varia a
orientação da largura ao redor do eixo de cada folha, mantendo as curvas e a
quantidade de órgãos. Isso diversifica a projeção real das lâminas e a oclusão
do miolo; não é um ajuste de câmera/material para ocultar a madeira. D também
usa esse campo, e E usa uma amplitude menor e uma paleta menos azulada.

O ensaio próximo de F confirmou melhora com mais raminhos e sítios foliares na
faixa externa da copa. F/G/H receberam esse aumento real de densidade; F/H usam
o contorno nativo mínimo de4 amostras, apropriado à folhagem pequena, para conter
o custo. G conserva6 para suas lâminas maiores. As contagens finais abaixo já
incluem essas alterações; a madeira ainda pode aparecer por vãos entre folhas.

A revisão de cor e detalhe compara fotografias e candidatos em câmeras/luz
independentes. Os crops dos candidatos são reais e registrados; não existe
alinhamento fotográfico, segmentação ou porcentagem de fidelidade por RGB.
R1 não resolve folhas individuais e R2 não resolve anatomia das cabeças. R3/R4
ocultam a base pela areia. Verso, raízes e interior permanecem hipóteses.

Folhas são facetadas e podem se atravessar; hastes se sobrepõem nas inserções.
Nervuras finas de G3, serrilhas e microtextura não foram implementadas; dobra
longitudinal aproxima somente o relevo principal. Topiaria é um snapshot autoral,
sem ferramenta nova de poda/crescimento em tempo real. Não se afirma réplica
exata das fotos, identidade botânica, física de gameplay ou aceite artístico.

## Transporte, validação e custo

O ensaio inicial de publicação embutida excedeu o envelope MCP de2MiB para uma
fonte densa: transporte encerrou corretamente e o cliente recebeu BrokenPipe.
Fontes grandes agora usam `Botany.load` nativo por arquivo temporário estável,
removido após publicação. Fontes pequenas mantêm a rota anterior. A quota não
foi aumentada; não se reduziu a vegetação para contornar o limite. O helper
`publish_recipe` também atende a cena autoral da demonstração genérica.

Os cartões novos exportam só os assets dos três exemplares enquadrados. Isso
evita copiar/serializar as demais famílias em cada arquivo de aproximação. Essa
opção é exclusiva da nova coleção; as anteriores mantêm a rota e fontes originais.
Não é descarte no renderer nem ganho de FPS demonstrado.

Preflight do schema: `e074e577dfc4f065`; ensaios neutros iniciais:
`65ea7ed588263f6d`; gate estrutural final: `4f5236169d22e891`, todos release
macOS arm64. O executável foi substituído pelo trabalho paralelo durante a
entrega; cada relatório registra seu build. Este trabalho não compilou, instalou
ou copiou um executável e não controlou a janela humana.

As fontes são detalhadas; compartilhamento evita duplicação de geometria por
cópia, mas cada indivíduo ainda custa para renderizar. Não há novo LOD,
benchmark de FPS ou estimativa de memória total/GPU. Contagens finais e
diagnóstico por característica são registrados abaixo a partir dos relatórios.


### Contagens finais

Build final da galeria: `4f5236169d22e891`. As48 fontes ocupam132709382bytes. A biblioteca tem72 meshes vegetais e um chão, 6344754v/11988938tri únicos. O campo64 tem193 instâncias e30999162tri somados antes do descarte; a mistura alternativa soma32997846tri. Variantes não utilizadas ainda ficam residentes na biblioteca completa dessas demonstrações.

| Perfil | Triângulos por indivíduo, três variações |
|---|---|
| `xerico_cinza` | 137360 / 134416 / 134800 |
| `xerico_baixo` | 346208 / 346336 / 345696 |
| `almofada_clara` | 773904 / 776464 / 777040 |
| `duna_oval` | 395452 / 397844 / 396596 |
| `duna_agulha` | 211764 / 213300 / 213556 |
| `esfera_miuda` | 916384 / 916896 / 913568 |
| `esfera_folhosa` | 508264 / 507848 / 506912 |
| `vertical_denso` | 702120 / 708904 / 707304 |

Todas as partes nativas exportadas passaram a conferência de fechamento/orientação. Posicionamento repetido igual e primeiro save/load final com PNG byte exato, sem recorrer à tolerância mínima da demonstração. As48 fontes regeneram iguais, outra seed muda as curvas de todos os componentes e os IDs são únicos por fonte. As57/39/42/24 fontes anteriores continuam iguais, assim como18/19/17/25 PNGs locais existentes. Isso não é uma nova comparação de imagens entre builds distintos. Python compilado e alterações próprias verificadas; não houve suite Rust por não haver alterações Rust nesta entrega.


### Cobertura visual e composição

As oito aproximações e crops dos candidatos foram abertos junto dos contextos de referência. A fileira de topiarias e o campo desértico também foram revistos no renderer. Diagnóstico final por item:

| Itens | Capturas correspondentes | Resultado e divergência restante |
|---|---|---|
| A1..A3 | `xerico_cinza-detail.png`, `-crop.png`, `-macro.png` | Rede ascendente e ramagem nua presentes; distribuição ainda mais regular/aberta que a moita fotografada. |
| B1..B2 | `xerico_baixo-detail.png`, `-crop.png`, `-macro.png` | Hábito baixo lobado e folha fina presentes; camadas dos ramos mais legíveis/geométricas que na foto. |
| C1..C3 | `almofada_clara-detail.png`, `-crop.png`, `-macro.png` | Lobos, ramagem nua e cabeças claras sustentadas presentes; cabeças são pequenos prismas facetados, não flores anatomicamente reproduzidas, e há mais vãos que na referência. |
| D1..D2 | `duna_oval-detail.png`, `-crop.png`, `-macro.png` | Base multi-haste aberta e lâminas ovais/pontudas presentes; madeira ainda mais aparente na copa que na foto. |
| E1..E3 | `duna_agulha-detail.png`, `-crop.png`, `-macro.png` | Lâminas estreitas ascendentes e cor verde acinzentada presentes; sem brilho/iluminação fotográficos calibrados. |
| F1..F3 | `esfera_miuda-detail.png`, `-crop.png`, `-macro.png` | Contorno esférico e folhagem densa com brotos menores claros presentes; ainda há vãos e madeira aparente, sobretudo de perto. |
| G1..G3 | `esfera_folhosa-detail.png`, `-crop.png`, `-macro.png` | Copa irregular e folha maior com brotação presentes; nervuras finas ausentes, dobra apenas parcial, ramos centrais mais visíveis. |
| H1..H2 | `vertical_denso-detail.png`, `-crop.png`, `-macro.png` | Copa alta/estreita e grupos pequenos em eixos ascendentes presentes; base e madeira mais expostas que no viveiro da referência. |

Aproximações/macros e plantios adicionais usaram `38af790d4fa7214f`. O deserto56 tem169 instâncias/22887494tri somados; a fileira6 compartilha três variantes,19 instâncias/5493698tri somados. O piso arenoso é plano e a fileira usa piso verde simples. Não se reconstruíram os cenários.

A composição nativa com grama acrescentou64 arbustos: 2464 objetos/4261 instâncias, 11373134tri únicos/35642518tri somados. Campos, assets, instâncias e grupos prévios foram preservados. SaveScene migrou a versão global109→110 no build do trabalho paralelo; este incremento não implementou essa migração. Cena e captura em `with-grass.scene.json`/`with-grass.png`.

Validação técnica não encerra a comparação artística. As aproximações preservam os hábitos principais e permitem ajustar parâmetros; não há alegação de cópia exata de todas as características das fotos. EAF-204 permanece em avaliação artística desta extensão.

A captura composta usou `8784ca62e3513867`; as substituições do executável compartilhado vieram do trabalho paralelo, sem builds por esta entrega.
