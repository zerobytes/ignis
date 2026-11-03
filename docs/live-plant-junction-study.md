# Junção entre plantas vivas — estudo incremental

## Avaliação anterior ao código

Continuidade de ed54089: conservar as duas plantas e provar o encaixe antes de
introduzir uma montagem persistida. A primeira planta já contém uma assembly
com tronco e enxerto estáticos; a segunda é um filho rígido do osso secondary-06.
O novo encaixe precisa de recortes sobre seções reais, correspondência exata,
pesos e transforms compatíveis. Copiar o pai como contexto congelado não cria
uma dependência viva.

Alternativas: bake encerra edição; replicar fontes em cada filho duplica autoria
e tronco; montagem por referências mantém os documentos separados, mas precisa
resolver ordem, ciclos, recursos de desenho consumidos e preservação do rig.
A direção é montagem por referências. Primeiro provar geometricamente o caso
com recortes de seções e união nativa; somente depois definir o formato de
dependências. O ensaio não será apresentado como regeneração nativa entregue.

O kernel atual de união aceita uma única raiz rígida nos três contornos. Aqui,
o pai permanece na base e o filho acompanha outro osso. Aplicar uma raiz única
mudaria a animação. Precisamos distinguir prova de forma, atribuição explícita
de skin e montagem persistida. Geometria removida não ganha correspondência
implícita com o interior novo.

Primeiro incremento compartilhado: consulta pura inspect_plant_sampling retorna
a amostragem construtiva corrente, material e identidade de superfície diretamente
de Definition.sample. Isto evita reencontrar cortes por proximidade ou duplicar
a fórmula da trajetória em Python/UI. Resultado é base sem escultura/assembly,
com índices locais ao documento/revisão corrente, e não uma identidade permanente
de vértice. Fonte/referência e asset final continuam em inspect_plant/inspect_mesh.

Checklist anterior:1.query explícita, nada por tick;2.um Sampling limitado já
gerado pelo núcleo, serialização somente na consulta;3.mesmos f32/IDs e faces;
4.sem segundo avaliador de curvas;5.mesmos limites de Definition, resposta pode
ser grande como preview, nenhum aumento de budget;6.sem GPU/I/O novo;7.dispatcher
existente, sem lock/thread/dependência;8.sem cache novo, consulta da cena corrente;
9.pureza/identidades/erro atômico e uso em recortes;10.custo síncrono de amostragem
e serialização mantido. Não reservar formato de cena/replay por uma consulta.

A consulta de união passa também a retornar material_plan, a proveniência
transitória já produzida pelo kernel: prefixos retidos, faixa iso e pares exatos
fonte→guarda. Isso permite atribuir pesos às guardas sem heurística espacial.
Não muda a geometria, os budgets ou o formato persistido; custo adicional é
serializar a lista limitada de pares numa consulta explícita.

## Prova de forma e skin

Reprodução, após `demo_tree_secondary_plant.py` gerar a cena de duas plantas:

```sh
python3 scripts/study_tree_connection_geometry.py --input artifacts/tree-secondary-plant/examples/after.scene.json
python3 scripts/study_tree_connection_skin.py
```

Primeiro script consulta amostragem/material nativos e a assembly para localizar
o intervalo030→044 do pai e o corte1 do filho por índices e células, sem nearest.
Compacta apenas dados retidos e transforma o filho para o frame do pai. O kernel
native preview_mesh_junction_union cria a superfície nova:27144pontos de campo,
1729v3303t iso,1825v3748t adicionais ao todo. Contornos32/32/32v e prefixo
retido11482v; saída13307v26610t. Restante da árvore não é reconstruído.

A primeira grade18mm excedeu65536pontos; resolução28mm respeita o limite. O
relaxamento de costura4 foi recusado por piorar o máximo/média dos ângulos; a
receita de prova usa0, sem desabilitar a guarda90°. Pico interno75,49°, média
5,22°,costura fonte0,0594°,costura iso11,72°; continua uma malha finita com
triângulos locais de qualidade insuficiente. O erro do campo3,15e−8m não é
distância Hausdorff nem garantia artística/C1.

Segundo script é pesquisa de skin, em Python no lado autor. Preserva pesos e
clipes antigos, mapeia a parte retida do filho rígido ao osso secondary-06 e
adiciona um osso de ponta explícito. Guardas copiam os pesos das fontes pelos
pares exportados; apenas os vértices iso recebem um campo harmônico escalar
entre base e socket. CG com precondicionador diagonal converge em96passos,
resíduo máximo8,65e−7, valores0,000792..0,989073 antes de quantização. Não é
autorig nem algoritmo já incorporado ao runtime/MCP como operação de skin.

Prova de poses0/60/240: partes retidas de ambos os ramos diferem no máximo
0,343µm; copa/rametas reparentadas à ponta diferem0,119µm; demais objetos exatos.
Novos triângulos têm produto de normais positivo em oito poses durante o ciclo
(mínimo cos0,999271); isso verifica esta animação, não embedding global ou
animação arbitrária. O objeto final tem um componente, sem bordas abertas,
arestas não manifold ou vértices isolados. Undo/redo da entrada da cena,
save/load, reset e pixels repeat/replay/prefixo passaram.

O asset final com skin ultrapassa2MiB quando embutido no lote JSON, encerrando
o transporte conforme limite existente. O estudo usa uma cena proposta local
pelo load_scene validado, sem ampliar o limite. Esta fricção deve informar o
formato de receita futura, em vez de enviar meshes completas a cada edição.

Capturas antes/depois e cena em artifacts/live-plant-junction. A fonte de duas
plantas fica no input e na frente ed54089; shape-study.scene.json é um bake de
pesquisa que retira o vínculo paramétrico do pai para testar a união/skin.
Não promovida à árvore mantida, não altera a demo, e não deve ser confundida
com montagem viva. Na cena de estudo os materiais do corpo composto usam a
receita do pai; as capturas neutras isolam a forma.

## Próximo contrato e custos

Montagem por referências ainda precisa: resolver dependências/ciclos; declarar
qual instância desenha a composição sem duplicar o filho; conservar instâncias
de origem para seus vínculos; transportar rigs e clips admissíveis; preservar
contornos por identidade após retesselação; atualizar derivados após editar
qualquer planta, undo/load/reset. Não armazenar a malha do pai como fonte filha.
O primeiro domínio proposto é filho rígido preso a um osso do pai, com portas
nomeadas e políticas explícitas. Skin com múltiplas influências arbitrárias e
clips independentes exige outro contrato. Nenhum schema foi reservado aqui.

Revisão posterior:1.query só por pedido;2.Sampling serializado apenas na consulta,
estudos mantêm snapshots finitos;3.identidades de material/célula/índice e f32
existentes, sem depender de proximidade;4.um único avaliador de curvas;5.budgets
de kernel e transporte mantidos, prova usa arquivo de cena no input grande;
6.nenhum shader/buffer/readback rotineiro novo;7.mesmo dispatcher e locks;
8.sem cache persistido novo;9.pureza/schema/domínio/base versus escultura,
proveniência fonte→guarda, poses/topologia/replay e regressões;10.mantidos custo
síncrono de consulta e qualidade limitada da prova, sem FPS/realismo alegado.

Validação final:848Rust/84grupos release/all-targets, Clippy-Dwarnings/fmt,
84cenários MCP/340grupos completos headless e Python/diff/links aprovados.
Build24285b4b58acb464;66summaries com ID coincidente,18legados sem ID e hash da
fonte conferido. Provas de geometria/skin/replay executadas pelos dois scripts
mantidos, não apenas pelos protótipos temporários. Sem mudança de formato,
shader ou UI; integração regenerável permanece aberta.
