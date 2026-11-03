# Folhagens amarelas, vinho e roxas — análise anterior à construção

Pedido: acrescentar arbustos com folhas amarelas, vinho e roxas, usando exemplos
reais. “Rochas” foi interpretado como “roxas” pelo contexto. Não há pedido de
reconstruir jardins, casas, pisos, etiquetas ou uma fotografia integral.

Estado desta entrega de análise: **nenhuma nova curva, fonte ou geometria
construída; verificação dos candidatos não executada**. A lista abaixo precede
a geração e não é uma aprovação visual de modelos futuros.

## Fontes, conjunto e recortes

Fotos de referência ficam apenas em `artifacts/colored-shrubs/references`, com
URLs, resolução e SHA256 em `manifest.json`. Não são texturas nem fontes de
vértices; não serão incorporadas aos assets ou ao Git. Recortes estão em
`crops.json`, origem superior esquerda, X→direita/Y→baixo, sem resize/rotação.

| ID | Fonte visual e identidade | Organização observada | Recortes vistos `(x,y,w,h)` |
|---|---|---|---|
| D | [UF/IFAS, Gold Mound](https://gardeningsolutions.ifas.ufl.edu/plants/trees-and-shrubs/shrubs/golden-dewdrop/), foto Stephen H. Brown,300×250; [Flora Express Nursery](https://www.floraexpressnursery.com/shrubs/duranta), folha2500×1875 | Sebe dourada densa e podada, borda baixa contínua; detalhe mostra pares de folhas ovais pontudas, dentes pequenos, brotos amarelos e folhas internas mais verdes | Conjunto`duranta`:(45,75,240,135); folha`duranta-leaf`:(550,300,850,800) |
| B | [NC State, Berberis Crimson Pygmy](https://plants.ces.ncsu.edu/plants/berberis-thunbergii-crimson-pygmy/), David J. Stang, CC BY-SA4.0; copa1024×682, detalhe682×1024 | Ramos finos ascendentes/arqueados com folhas pequenas obovadas, agrupadas; terminais menores vermelhos/rosados, lâminas maduras vinho/bronze e áreas internas verdes na foto | `berberis-leaf`:(200,350,430,570); conjunto inteiro aberto |
| L | [NC State, Loropetalum rubrum](https://plants.ces.ncsu.edu/plants/loropetalum-chinense-var-rubrum/), Carol Tierny, CC BY4.0,1024×768 | Ramos com folhas alternas ovais/pontudas, roxo-vinho e algumas antigas esverdeadas; nervura central e relevo secundário legíveis; flores rosas de pétalas em fitas | `loropetalum`:(350,15,320,270), com folhas e contexto floral |
| C | [NC State, Cotinus Royal Purple](https://plants.ces.ncsu.edu/plants/cotinus-coggygria-royal-purple/), conjunto Leonora Enking CC BY-SA2.0,1024×768; detalhe Randy Harter CC BY4.0,1024×768 | Folhas maiores, quase arredondadas/obovadas, borda inteira, pecíolos finos e nervuras rosadas; ramos longos arqueados, vãos escuros e panículas; verde serrado vizinho de aparência distinta | Conjunto`cotinus`:(50,40,900,660); folha`cotinus-leaf`:(635,270,385,430) |

As fotos mostram fragmentos e oclusões: não se contam folhas totais, raízes ou
ordens de ramos invisíveis. Em D a foto do conjunto não resolve detalhes foliares;
o detalhe independente é necessário. Em B/L/C não se conhece a silhueta inteira
do indivíduo; o porte completo vem de uma hipótese autoral apoiada nas descrições
botânicas, não de uma medição pela fotografia. Tentativas de outras fotos de
conjunto retornaram403/timeout; não foram usadas como evidência visual.

## Segunda leitura e revisitação

- D: o crop confirma que “amarelo” não é cor plana uniforme: há folhas internas
  verdes, terminais claros, lâminas pontudas e pequenos dentes. Atualiza D02/D03;
  não copiar os dentes como nervuras ou confundir sombra com pigmento preto.
- B: o detalhe mostra terminais menores e mais vermelhos, folhas maduras mais
  largas abaixo e ramos vermelhos, não uma flor vermelha na ponta. Atualiza B02/
  B03/B04. Espinhos nodais são descritos pela fonte, mas o crop não permite contar
  todos; não inventar contagem pela parte desfocada.
- L: a lâmina é mais alongada/pontuda que a de C, e a brotação pende junto ao ramo;
  as estruturas rosas estreitas são flores, distintas da folhagem. Atualiza L02/
  L03/L04 e evita transformar pétalas em folhas roxas.
- C: as folhas roxas grandes têm bordo inteiro e nervura rosada, enquanto o verde
  serrado sobreposto tem outro contorno. Não usar esse verde como “brotação de
  Cotinus” sem evidência. Atualiza C02/C03/C04. As panículas rosas são outro órgão,
  não granulação das folhas.
- Revisita dos conjuntos: confirma contraste de textura fina D/B e folha maior C,
  irregularidade dos vãos e diferenças de porte. Casa/piso/palmeira em D, etiqueta
  em B, flores em L, panículas e vegetação vizinha em C permanecem contextualizados
  e explicitamente fora desta coleção de **folhagem vegetativa**.

Não foi necessária ampliação generativa nem sub-recorte adicional: os crops já
resolvem contorno e brotação; textura microscópica, verso, raiz e bifurcações
ocultas permanecem desconhecidos. Não inferir espessuras métricas pelos pixels.

## Lista de construção consolidada

Agrupadores D/B/L/C são variedades; cada tarefa executável abaixo tem estado
**não construído / comparação não executada** nesta análise. Repetições indicam
famílias de órgãos, não contagens de folhas visíveis.

| ID/pai | Construir e evidência | Relações/ocorrências e estado | Critério de conferência |
|---|---|---|---|
| D01/D | Corpo baixo denso de Duranta dourada, D conjunto | Hastes/raminhos sustentam grupos foliares; contorno podado observado, fundo/raízes desconhecidos | Volume e vãos em frente/lateral/topo; não uma superfície amarela opaca |
| D02/D | Folhas pequenas ovais, ápice pontudo, pares ao longo dos ramos, D folha | Sobrepostas, com pequenas diferenças de orientação; observado | Textura foliar fina e pontas separadas; dentes finos serão limitação declarada do contorno nativo |
| D03/D | Folhagem amarelo-dourada e brotos mais claros, D ambos | Requisito amarelo e observação de variação interna | Dois tons correlacionados com grupos/terminais, madeira não amarela |
| B01/B | Corpo compacto arredondado com ramos ascendentes/arqueados, B conjunto + fonte botânica | Porte total é hipótese autoral; repetir ramos3D com vãos | Ramos e volumes distintos de D/L/C, sem fingir foto inteira calibrada |
| B02/B | Folhas pequenas obovadas, ponta mais arredondada e base estreita, B detalhe | Aglomerados em ramos finos; observado | Lâminas curtas e largas acima da base; separadas de hastes |
| B03/B | Vinho/bronze maduro e brotação menor vermelha, B detalhe | Tonalidade e tamanho dependem do terminal; observado | Brotos menores contrastam com folhas maduras, sem flores falsas |
| B04/B | Ramos finos avermelhados e pecíolos, B detalhe | Suporte de B02/B03 observado; espinhos finos não resolvidos em toda a copa | Cor e suporte presentes; espinhos/nodos precisos ficam fora do primeiro bake |
| L01/L | Corpo multi-haste arqueado, mais aberto/largo, fonte e L conjunto | Hipótese de corpo onde o crop não mostra a planta inteira | Ver volume coerente em mais de um ângulo |
| L02/L | Folhas ovais/pontudas mais alongadas que C, L detalhe | Alternas por descrição botânica, repetidas ao longo dos ramos | Relação comprimento/largura e pontas distinguem L de C |
| L03/L | Roxo-vinho maduro e brotação rosada/ameixa, L detalhe | Observado; requisito roxo | Cor continua foliar, sem pintar hastes e sem confundir com as fitas florais |
| L04/L | Nervura central e discretas secundárias, L detalhe | Observadas, relevo fino desconhecido | Campo procedural de detalhe limitado; não alegar escultura vascular exata |
| C01/C | Corpo mais alto e aberto, ramos arqueados, C conjunto | Hastes grossas/raízes ocultas inferidas | Copa volumétrica aberta sem preencher todos os vãos |
| C02/C | Folhas grandes arredondadas/obovadas, borda inteira, pecíolos, C detalhe | Observado, lâminas em profundidades diversas | Folha significativamente maior/larga que D/B/L, com arredondamento aproximado |
| C03/C | Roxo profundo com brotação vinho mais clara, C conjunto/detalhe | Pigmentação e iluminação não calibradas separadamente | Paleta roxa distinguível do vinho de B, sem copiar brilho/sombra da fotografia |
| C04/C | Nervuras centrais rosadas, C detalhe | Contraste observado, ramificações finas parciais | Detalhe procedural discreto; resolução/precisão inferiores ao macro da foto declaradas |

Contexto excluído: cenários e etiquetas, pétalas/inflorescências/panículas das
fotos, espécies verdes vizinhas e propriedades de cultivo. A escolha é um estado
vegetativo autoral sem floração; não se afirma que as espécies não florescem.

## Plano e ferramentas confirmadas

Coordenadas locais: Y para cima; corpos volumétricos de hastes e folhas, sem
alinhamento à câmera. Alturas/alcances em metros são escolhas autorais, não
estimativas fotográficas ou simulação de idade. Fonte/Recipe7 do compilador
botânico nativo; schema lido do build `e46d5879c672aec9`, com Load/Publish/capture,
cores de madeira/folha, contornos strip, blade_samples4..32 e compare_images.

Fonte canônica será `assets/botany/colored-shrubs`, com três seeds por variedade.
Reusar curvas e parâmetros de estrutura com perfis específicos; brotos terão
componente de cor próprio. Formas e brotação por regras/seed, nunca pixels como
geometria/textura. As coleções aprovadas devem regenerar iguais e manter os PNGs.

Verificação prevista: corpos neutros em ângulos distribuídos antes da revisão de
cor; topologia por componente; quatro candidatos coloridos; crops reais dos
candidatos e revisão local comparada às fotos. Fotos e renders não terão câmera,
escala ou luz alinhadas: não calcular “percentual de fidelidade” por RGB/SSIM.
Save/load e comparação entre estados nativos terão mesma câmera/resolução e
usarão o comparador existente, com a discrepância mínima já conhecida registrada.

Lacunas declaradas antes de construir: contorno nativo não representa serrilhas
finas de D/L, nervuras secundárias exatas, pubescência, espinhos/nodos calibrados,
união vascular de todas as peças, solo, dinâmica de gameplay ou diversidade
biológica universal. São aproximações procedurais guiadas pelas referências;
não cópias fotogramétricas ou botânica calibrada.
