# Enxertos estáticos no contexto das plantas

Lista introduzida em Scene68/replay72; opções materiais exigem70/74. `Assembly.context_grafts` é uma lista ordenada de até8
ligações de fontes estáticas ao contexto canônico. É autoria regenerável pelo
mesmo `set_assembly`/`preview_plant`/`preview_scene_edits` do dispatcher/MCP,
sem comando exclusivamente visual. A lista não é ainda um grafo recursivo de
plantas animadas: suas referências apontam ao contexto original, nunca ao miolo
ou ao filho de um enxerto anterior.

Cada `ContextGraft` guarda:

- `source`: Asset estático imutável com ID distinto, sem rig/clipes;
- `source_translation`: translação explícita da fonte ao quadro do contexto,
  finita e limitada a10000m por eixo; rotação/escala não são inferidas;
- `source_faces` e `source_loop`: faces a remover e porta por índices dessa fonte;
- `context_vertices`: máscara por IDs canônicos do contexto;
- `context_loops`: as duas portas no contexto, por IDs canônicos;
- `options`: opções existentes da união tubular de3portas;
- `surface`: opções opcionais de acabamento do novo domínio iso;
- `source_material_frame`: afim opcional (`columns`, `offset`) aplicada ao canal
  material retido da cópia, independente de `source_translation` geométrica;
- `material_finish`: opções do acabamento material na geometria final, depois de
  surface/collar. Desliga o CG material legado nessa união, deixando geometria igual.

A junção principal tem `plant_material_frame` e `material_finish` equivalentes.
Afim exige canal explícito, coeficientes/resultados finitos dentro de±10000;
None conserva o domínio original. Escala/shear/offset e campos singulares são
possíveis por decisão autoral; nenhuma inversa é necessária. O acabamento regular
pode recusar material singular incidente à fronteira. A mudança de fase/frequência
no material avaliado é intencional; o valor original da fonte continua disponível.
Não confundir preservação da fonte com aparência avaliada idêntica.

O plano material não aparece na receita: ele é produzido novamente pelo fit,
com ordem/prefixos/pares exatos da união atual. Um loft com outro número de anéis
não reutiliza índices de uma avaliação anterior. Reporte `material_finish` separa
fontes/guardas/iso/zipper, iterações/resíduo pós-f32 e envelope em unidades materiais.
Contrato do kernel em mesh-material-finish.md; gate de projeção/realismo é separado.

Ordem: contexto imutável → lofts → filtro do contexto → enxertos em ordem →
corte da assembly principal → união com a planta viva → acabamento principal.
A máscara de cada enxerto inclui um ponto novo de loft somente quando AMBOS
endpoints canônicos estão selecionados. Uma face é removida somente quando
TODOS os seus vértices pertencem à máscara expandida. IDs inválidos/duplicados,
portas ausentes e máscaras sobrepostas que consomem referências são erros.

Cada remoção mantém um mapa parcial canônico→atual; removido é `None`. O corte
principal reencontra os3IDs mapeados e o triângulo orientado exato ainda presente.
Isso também vale quando o número de pontos do loft muda. Não se procura um
triângulo próximo nem se escolhe uma filha de uma face substituída arbitrariamente.
Os IDs da fonte filha usam namespace próprio. As fronteiras reais ditam o winding;
a ordem escrita dos IDs de uma porta serve à sua identidade, não a uma busca
espacial. A união existente exige exatamente3ciclos4..32 aproximadamente redondos
e planares, suporte/hub/guardas válidos, grade≤65536 e saída dentro dos tetos.

Fontes integrais, incluindo faces depois cortadas, participam da validação e dos
budgets canônicos (16assets,32768v/65536t). O resultado avaliado e o draw são
validados separadamente. IDs de fontes repetidos são recusados. Não houve aumento
de tetos, mudança de layout/renderer nem trabalho novo por tick/frame.

Os pontos retidos da filha recebem somente a translação f32 explícita; seus
`material_coordinates` permanecem exatos no domínio original. Fonte armazenada,
referência da planta, contexto original, bones e clipes não são rebased. A
montagem existente atribui o contexto estático ao root explícito da planta.
Isso serve ao secundário0 estático; não é transferência multibone. O secundário06
animado requer contrato próprio de pesos/hierarquia. Aparência é propagada a todas
as fontes, preservando suas geometrias e undo.

O miolo gerado é novo: não recebe implicitamente escultura de faces removidas.
`surface` seleciona somente vértices iso, mantendo fonte e primeira guarda fixas.
O relatório `junction` é anterior a esse filtro; `surface` mede sua própria
região e volume algébrico de componente fechado, sem prometer volume físico
conservado, continuidade analítica C1/C2 ou ausência global de interseções.

## Mesmo objeto, primeiro enxerto

`scripts/demo_tree_graft.py` parte do snapshot fixo7ee07c6 e usa extração nativa
`ExtractIslands` em preview para mover o secundário0 de wood à receita. O lote
publica simultaneamente wood sem essa ilha e a receita, evitando desenho duplicado.
A fonte filha119v/234t fica inteira no documento; retém91v após o corte da raiz.
Os cortes do pai são os anéis1903/1999. Portas32/32/7, razão de raios2.93,
grade39527. A composição final é um componente fechado; wood passa177→176ilhas.
Outros galhos continuam atravessados e estáticos. A ponta anteriormente protegida
e a transição visual em anel/casca continuam pendentes. Um limite de recusa90°
não é critério de realismo.

Capturas nativas antes/depois, com casca e neutras, mantêm câmera/luz e tick60 em
`artifacts/tree-graft`. O resultado não é um render externo nem um novo objeto.

## Acabamento do colar (Scene69/replay73)

`collar_surface` opcional roda DEPOIS de `surface`. Exemplo do primeiro encaixe:

```json
{"ports":[0,1],"rings":6,"fixed_rings":2,
 "options":{"iterations":4,"strength":0.3,"max_displacement":0.01}}
```

`ports` é um subconjunto único não vazio dos ordinais0/1 (contexto) e2 (filha).
São identidades das portas canônicas, resolvidas na avaliação atual. Uma BFS
multi-source de arestas únicas calcula a menor distância a QUALQUER seed das
portas escolhidas. Seleção inclusiva `d <= rings`; âncoras externas
`rings-fixed_rings < d <= rings`. Exige1..16anéis e1..rings camadas fixas,
além de alcançar a última camada solicitada; não trunca o alcance silenciosamente.
Nenhum índice do miolo é persistido. Anéis são distância topológica, não metros,
e mudam com a tesselação. A receita é reavaliada nesse caso.

A camada PODE mover vértices retidos e guardas selecionados de forma intencional.
Ela não muda a fonte armazenada, mas pode reduzir detalhe avaliado dentro da
região; não é preservação visual exata da escultura que se pediu para suavizar.
Fora da seleção e nas âncoras os pontos ficam exatos. Portas não escolhidas e
suas primeiras guardas recebem pins quando alcançadas. Portas da assembly
principal são comparadas ao contexto original APÓS todos os enxertos/filtros;
se deslocadas/removidas, a operação é recusada. Enxertos sucessivos seguem a
ordem autoral: uma região pode alcançar geometria de um enxerto anterior; só
as interfaces explicitamente protegidas recebem essa garantia.

`collar_surface.filter` contém o relatório regional do filtro existente.
`moved_retained_context`, `moved_retained_source` e `moved_generated` distinguem
as origens. `junction` continua PRÉ-`surface`/PRÉ-`collar_surface`, e `surface`
continua PRÉ-`collar_surface`. Logo `retained_vertices_exact` e os ângulos da
costura crua não descrevem o resultado posterior intencional. Triângulos,
domínios materiais, pesos, bones e clipes permanecem exatos. As poses dos
pontos remodelados mudam; descendentes e pontos não remodelados são outro gate.
Sem o campo, os resultados antigos são mantidos exatamente.

A prova na mesma árvore move765pontos:320retidos do pai e445gerados (381iso+
64guardas); a filha não é alcançada. Região1154v,389pins. Pico regional
41.2277→23.9936°,média6.0103→5.8012°;não são métricas da árvore inteira.
As guardas de inversão/dobra/qualidade e o envelope10mm(+tol.f32) permanecem.
A ponta antiga94° fica fora desse recorte. Casca/demais interseções continuam
pendentes. Repro `scripts/demo_tree_collar.py`, imagens em artifacts/tree-collar.
