# Gramas e matos — EAF-200

Coleção autoral de cobertura do chão:12 tipos,36 tufos diferentes e57 fontes de
componentes nativas. Variam distribuição, comprimento, largura e curvatura das
folhas; as cópias variam rotação, escala e tonalidade de forma correlacionada.
Os nomes descrevem silhuetas; não identificam espécies nem calibram botânica.

| Tipo | Forma | Altura máxima das três amostras na galeria | Triângulos por tufo |
|---|---|---:|---:|
| Grama baixa | Folhas curtas, abertas junto ao chão |0,16m|1408|
| Grama fina | Touceira estreita e ascendente |0,31m|1600|
| Folha larga | Lâminas largas e abertas |0,36m|832|
| Capim arqueado | Lâminas longas em arco |0,45m|1152|
| Capim alto | Touceira vertical |1,07m|960|
| Capim seco | Folhas ocres |0,50m|1216|
| Capim com espigas | Hastes com espiguetas compactas |0,97m|3316|
| Capim com panículas | Eixos laterais e sementes separadas |1,18m|6024|
| Capim de plumas | Filamentos finos ascendentes |1,32m|6836|
| Mato trifoliado | Três lâminas arredondadas por pecíolo |0,13m|1764|
| Flores brancas | Roseta, folhas caulinares, nove pétalas e miolo |0,52m|3048|
| Flores roxas | Roseta, folhas caulinares, cinco pétalas e miolo |0,77m|2344|

Alturas medidas do bake da galeria, com inserção no solo emY−0,025m, não medidas
botânicas de referência. A escala adicional do povoamento altera o porte.

## Fonte e compilação

`assets/botany/groundcover/library.json` organiza os componentes de cada variação.
As fontes `.ignis-botany` armazenam curvas quadráticas amostradas, perfis de lâmina,
espessura e seções de hastes. O compilador Rust existente constrói as malhas
fechadas com `blade_samples=4`, `axis_samples=4` e `sides=6`. A resolução baixa é
uma escolha de autoria; faces facetadas e estruturas finas permanecem visíveis
em aproximações. Não há alteração de kernel, shader ou formatos globais.

O gerador Python é uma ferramenta de preparação de assets. A gramática desta
coleção não roda nativamente a cada frame. Os materiais diferentes usam fontes
separadas para verde, sementes, pétalas e miolo, sem adicionar material por órgão
ao formato. Cada componente pode ser aberto/editado no painel Vegetação usando
o mesmo Load/MCP existente. As peças são malhas fechadas individuais que podem
se sobrepor; não há união vascular nem solver acoplado entre componentes de cor.

```sh
python3 scripts/groundcover_assets.py
python3 scripts/demo_groundcover.py
```

O primeiro comando regenera explicitamente as fontes canônicas. Salvar alterações
manuais em outro destino antes disso. O segundo confere as fontes com a receita
canônica e usa um processo MCP isolado, sem tocar na janela ou cena humanas.
Não precisa recompilar a engine; usa `target/release/ignis` corrente.

As saídas correntes, substituídas a cada execução, ficam em `artifacts/groundcover`:

- `review.html`: galeria nativa completa, campo misto, campo sem flores e cartões.
- `authoring.scene.json`: fontes editáveis embutidas, distribuídas na galeria;
  cada componente do tufo ocupa a mesma posição. O painel Vegetação edita uma
  fonte por prefixo, não todo o conjunto de cores de uma vez.
- `gallery.scene.json`: os36 tufos, como malhas estáticas compartilháveis.
- `field.scene.json`:2400 tufos mistos em10×10m, com uma passagem central sinuosa.
- `grass.scene.json`:2400 tufos, somente gramas/folhas largas/capim seco, sem flores
  nem pendões. Há chão exposto; não é cobertura integral certificada do solo.
- `library.scene.json`, `templates.json`, `library-build.json`: bake, templates
  e hashes que impedem usar silenciosamente uma biblioteca desatualizada.
- `report.json`, `placements.json`, `composition-verification.json`: contagens,
  posições e evidência de validação. `field.recipe.json` registra a configuração.

As cenas de campo usam cópias estáticas. Os arquivos fonte continuam editáveis;
aplicar uma escultura na cena de autoria não atualiza automaticamente todos os
campos já exportados. Recompilar/replantar é uma etapa explícita.

## Quantidade, mistura e distribuição

`examples/groundcover-field.recipe.json` configura seed, lado do quadrado, número
de tufos, passagem e pesos dos12 tipos. Peso0 exclui o tipo. O peso controla uma
probabilidade, não uma contagem exata nem percentual de área coberta. Uma malha
estratificada com jitter evita alinhamento rígido; campos suaves independentes
por família, com escala2,8m, formam manchas de mistura. A ordem de seleção e os
IDs são reproduzíveis para a mesma receita/biblioteca no ambiente atual.

```sh
python3 scripts/demo_groundcover.py --count 4000 --size 12 --seed 52
```

Para adicionar um trecho a uma **cópia** de uma cena autoral, após compilar a
biblioteca, por exemplo:

```sh
python3 scripts/plant_groundcover.py \
  --scene artifacts/groundcover/gallery.scene.json \
  --output artifacts/groundcover/planted.scene.json \
  --x 12 --z 0 --ground-y -0.03 --prefix meadow
```

O centroX/Z, a alturaY do chão e a receita são explícitos. O comando conserva
conteúdo/câmera/colisão preexistentes, reutiliza assets idênticos e acrescenta
instâncias/grupos `meshes.objects`. IDs ou assets conflitantes recusam; entrada
e saída iguais recusam. O candidato passa por LoadScene nativo antes de SaveScene
publicar o destino. A biblioteca deve corresponder aos hashes de fonte/bake.
Criar uma segunda área na mesma cena exige outro prefixo.

Esta operação planta em um **plano de altura constante**. Não consulta terreno,
água ou encostas, não roda no runtime planetário e não acrescenta colisores,
vento, animação, crescimento ou interação física. Integração de povoamento na
superfície real e controles próprios na UI/MCP são trabalho posterior; aqui se
reutilizam as ferramentas nativas existentes por scripts de preparação.

## Custos e validação

Build nativo usado: `e46d5879c672aec9`, macOS arm64 release. Nenhum novo executável
foi criado; o build/janela planetários correntes foram preservados. Nesta receita:

| Medida | Campo misto | Campo de gramas |
|---|---:|---:|
| Tufos |2400|2400|
| Instâncias, incluindo chão |4069|2401|
| Geometria única da biblioteca completa, incluindo chão |48634v /91502tri|48634v /91502tri|
| Triângulos somados sobre as instâncias antes do descarte |4643358|3251330|

A biblioteca completa fica residente nas cenas de demonstração, inclusive tipos
não usados no campo de gramas. O comando de composição só importa assets usados
no trecho, caso ainda não existam na cena de destino. Fontes canônicas57arquivos:
1106617bytes, além do manifesto. Geometria compartilhada economiza duplicação de
assets; **não elimina o trabalho de desenhar cada cópia**. Pendões e flores ainda
são caros para grandes extensões. Não foi medido FPS, custo GPU ou memória total,
e não foi implementado LOD próprio para gramas nem cobertura distante coletiva.

A execução nativa compilou todas as57 fontes e verificou incidência fechada e
orientação oposta de arestas nos78 meshes vegetais, removendo somente os
placeholders de um triângulo das categorias vazias do compilador. Fontes e
posicionamento regeneraram iguais; outra seed alterou curvas. Campo salvo e
reaberto gerou PNG byte exato. Composição nativa preservou os campos originais,
malhas/instâncias/grupos anteriores e reutilizou geometria. Entrada=saída,
conflitos de identidade, receitas inválidas e biblioteca desatualizada recusaram
sem publicar o destino. Python compilado e diff verificados; nenhum Rust mudou,
portanto não se repetiu a suíte da engine sem necessidade.

Revisão visual das capturas nativas confirmou diversidade de porte/folhas,
sementes, pétalas, miolos e composição. Solo, câmeras e luz são de demonstração;
não houve comparação fotográfica. O viés da sombra foi ajustado para retirar
moire de auto-sombra do plano. Plumas finas ainda apresentam aliasing à distância;
o interior das manchas ainda mostra chão. O usuário aprovou explicitamente esta
coleção baixa; a continuação acrescenta [mato alto](tallgrass-assets.md) em uma
biblioteca separada, usando `--collection tallgrass` nas ferramentas comuns.
