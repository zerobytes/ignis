# Arbustos procedurais — EAF-204

Esta coleção amplia as gramas e matos aprovados de EAF-200 com oito famílias de
arbustos e três variações por família. As estruturas lenhosas, o volume da copa,
o porte, o tamanho das folhas e a ocupação dos sítios foliares variam por perfil.
Os nomes descrevem formas autorais; não identificam espécies científicas.

| Família | Porte autoral de referência | Forma e folhagem |
|---|---:|---|
| Compacto |0,85m|Múltiplas hastes, muitos ramos curtos e copa densa |
| Aberto |1,10m|Menos ramos e folhas, vãos e madeira aparente |
| Espalhado |0,65m|Copa baixa, larga e arqueada |
| Vertical |1,85m|Copa estreita, ramos ascendentes, folhas alongadas |
| Folhas grandes |1,10m|Lâminas largas, mais espaço entre grupos |
| Flores brancas |1,10m|Folhas ovais e grupos de três flores pequenas |
| Flores rosas |0,95m|Folhagem densa e flores maiores de cinco pétalas |
| Flores amarelas |1,35m|Folhas estreitas e flores de oito pétalas |

Esses portes são parâmetros do esqueleto, não máximos geométricos exatos: folhas,
flores, variação de altura e escala dos indivíduos alteram o contorno final.
Compacto e aberto diferem em quantidade de hastes, ramificações, sítios e ocupação
foliar. A ocupação probabilística é0,96 no compacto e0,52 no aberto; não é uma
medida de cobertura visual. O comparativo `density.png` usa escala1 e a mesma
câmera, com compacto à esquerda e aberto à direita.

No primeiro exemplar, o compacto tem941 folhas/77 eixos; o aberto tem216 folhas/
45 eixos. Essa é uma contagem dos órgãos gerados, não uma estimativa pela imagem.

## Estrutura procedural

`scripts/shrub_assets.py` gera hastes basais, ramos laterais em diferentes alturas
e azimutes, e raminhos terminais. Perfis elípticos da copa, assimetria seeded e
proporção altura/largura orientam os destinos. Folhas nascem ao longo das curvas
reais desses eixos; sua orientação varia em torno do ramo e do eixo vertical.
Não são folhas distribuídas por uma esfera nem uma superfície opaca da copa.

As curvas são amostras de trajetórias quadráticas. Lâminas têm contorno, dobra,
espessura e forma fechada gerados pelo compilador Rust existente. Pedúnculos
afastam as flores das folhas terminais, sem recortar geometria pela câmera.
Cada folha, eixo, pétala e miolo continua sendo geometria3D real.

O corpo de cada variação reúne hastes e folhas na mesma receita: o canal de madeira
usa marrom e o canal foliar usa verde. Pétalas e miolos são fontes adicionais.
Assim,42 fontes cabem na quota nativa de64 receitas por cena, sem elevar limites.
A primeira organização com fontes separadas de madeira/folhas teria66 receitas
e foi recusada pela guarda; foi substituída antes da entrega. Não mudou Source7,
Recipe7, Scene ou renderer. As coleções anteriores permanecem iguais.

As superfícies são fechadas individualmente e se encontram por contato ou
sobreposição; os eixos desta coleção não usam junções soldadas nem hierarquia
vascular acoplada. Não se infere pele única, poda física coerente entre fontes,
animação, vento, colisão ou dinâmica de gameplay a partir desses assets estáticos.

## Uso

```sh
python3 scripts/shrub_assets.py
python3 scripts/demo_groundcover.py --collection shrubs
```

Fontes e manifesto: `assets/botany/shrubs`. Regenerar explicitamente substitui os
arquivos gerados; preservar esculturas em outro destino. A mesma seed e regra
reproduzem as curvas no ambiente atual; não há contrato entre versões arbitrárias.

Saídas em `artifacts/shrubs`, substituídas nos mesmos caminhos:

- `review.html`: oito tipos, campo misto, aproximação e comparação de densidades.
- `authoring.scene.json`:42 fontes editáveis, distribuídas por variação.
- `gallery.scene.json`:24 exemplares como instâncias estáticas.
- `density.scene.json` e `density.png`: compacto e aberto lado a lado.
- `field.scene.json`:90 arbustos mistos em16×16m; receita seeded com passagem.
- `grass.scene.json`: demonstração somente com arbustos sem flores.
- `library.scene.json`, `templates.json`, `library-build.json`: bake compartilhado
  e hashes; `report.json` e `verification.json`: contagens e evidências.

Quantidade, área, seed e mistura estão em `examples/shrubs-field.recipe.json`.
Peso0 exclui uma família. O plantio usa o mesmo fluxo das outras coleções:

```sh
python3 scripts/plant_groundcover.py --collection shrubs \
  --scene artifacts/groundcover/field.scene.json \
  --output artifacts/shrubs/with-grass.scene.json \
  --ground-y -0.03 --prefix bushes
```

A saída deve diferir da entrada. O comando conserva conteúdo anterior e importa
somente os assets necessários que ainda não existem. Os novos usam prefixo `sh`,
independente de `gc` e `tg`; cópias compartilham geometria. Biblioteca desatualizada,
IDs/assets conflitantes e receita inválida recusam sem publicar o destino;
validação e gravação usam Load/SaveScene nativos em processo isolado.

O plantio continua em altura constante, sem adaptação a encostas/água/planeta.
Não há controle novo de arbustos no painel; as fontes usam o painel Vegetação e
MCP existentes, enquanto a biblioteca e o povoamento são preparados por scripts.
Campos já exportados não recebem automaticamente uma escultura da cena autoral.

## Conferência e limites

Build usado: macOS arm64 release `e46d5879c672aec9`. As42 fontes ocupam13432710
bytes. A biblioteca completa tem66 meshes vegetais e um chão,592448v/1114758tri.
O campo de90 arbustos tem233 instâncias e4094778tri somados antes do descarte;
somente arbustos sem flores:181 instâncias/3929846tri. A biblioteca completa fica
residente nas demonstrações, inclusive variantes não usadas em uma mistura.

O exemplo composto com gramas conserva o campo anterior e acrescenta os90
arbustos:2490 objetos/4301 instâncias,1153658tri únicos e8738134tri somados sobre
as instâncias. O plantio importa apenas os assets usados e ausentes na entrada.
`with-grass.png` é uma captura nativa adicional dessa composição.

Verificações incluem compilação nativa de todas as fontes, incidência fechada e
orientação das arestas, reprodução de curvas/posicionamento, salvamento e composição
com as coleções anteriores. Capturas isoladas e em conjunto permitem avaliar
folhagem, vãos, madeira, flores e coerência espacial. Não houve comparação com
fotografias, identificação de espécie ou calibração de crescimento/idade.

As42 fontes canônicas regeneraram iguais; outra seed mudou as curvas de todos
os componentes. As fontes das duas coleções anteriores ficaram iguais, assim
como18 PNGs de gramas e19 de mato alto. A composição nativa preservou campos,
assets, instâncias e grupos anteriores. Conflitos de identidade e biblioteca
desatualizada recusaram sem publicar um destino. Relatório em `verification.json`.

No primeiro save/load final houve diferença de1 pixel em1600000, de1 nívelRGB8;
o comparador registra essa diferença em vez de afirmar igualdade. Os valores da
cena foram iguais, e o segundo par de save/load nativo produziu PNG byte exato,
como exige a guarda já existente na demonstração. A causa da pequena diferença
inicial não foi estabelecida. Não se alterou nem relaxou a guarda nesta entrega.

Python compilado, links próprios e diff verificados. Nenhum Rust foi alterado
por esta entrega; não houve novo build, instalação, push ou mudança na janela
humana. Trabalho paralelo no website, materiais e planeta foi preservado.

Folhas e pétalas ainda são facetadas e podem se atravessar; estruturas finas têm
aliasing à distância. Compartilhamento reduz duplicação de geometria, mas cada
cópia continua tendo custo de renderização. Arbustos densos e floridos são assets
detalhados; não há LOD próprio, benchmark de FPS ou garantia de desempenho em
florestas extensas. Aceite artístico desta coleção permanece com o usuário.
