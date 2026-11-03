# Conexões regeneráveis entre plantas

## Contrato anterior ao código

Continuidade autorizada de c305ded: promover o ensaio de forma/skin para receita
nativa, sem enviar malhas pelo MCP. A UI permanece em outra frente.

Alternativas: bake perde os vínculos; contexto filho com uma cópia do pai duplica
autoria; grafo arbitrário amplia ordenação/ciclos e cortes sobre saídas anteriores.
Escolha inicial: até8 conexões de pares disjuntos. Uma planta participa no máximo
de um par, em qualquer papel. Isto recusa cadeias/ciclos/múltiplos filhos antes
de avaliar; não é um grafo geral de árvores. Cada conexão guarda IDs de duas
instâncias, seções nomeadas do pai/filho, raízes de skin e parâmetros físicos.

Ambas as Entry mantêm definition/reference/source/assembly separadas. Avaliar
as bases primeiro, depois recortar por identidade material e compor o asset do
pai. O filho conserva seu asset e instância para inspeção e attachments, mas
seu desenho é consumido pela conexão. Sem duplicar instância, fonte ou rig no
documento. O renderer omite somente o corpo da instância consumida; seus filhos
continuam apresentados. Esta máscara deriva da receita, sem novo campo genérico
de visibilidade ou do estado de gameplay. Draw budget atual continua contando
também o filho consumido, de forma conservadora; não ampliar tetos.

Domínio inicial: cada asset tem uma única instância; o filho é ligado diretamente
a um osso explícito do pai, sem playback/clipes próprios, com todos os seus
vértices rigidamente ponderados a uma raiz declarada. Ossos extras do filho
continuam disponíveis para seus descendentes, mas não deformam seu corpo.
O pai conserva todos os ossos/clipes/pesos retidos. As duas portas do pai devem
usar a raiz declarada; a porta do filho usa o osso de attachment. Guardas copiam
pesos por proveniência exata; somente o iso novo recebe mistura harmônica entre
esses dois ossos. Não inferir skin geral nem reinterpretar clips independentes.

Cortes por IDs de seções elípticas interiores. Se o pai já tiver assembly, o
intervalo deve estar no corpo retido após seu corte principal; mapear células
e prefixos exatos, sem persistir índices avaliados ou usar nearest. Filho não
pode ter assembly neste recorte. Refinos respeitam portas4..32 do kernel; erro
atômico quando uma densidade não for admitida. A superfície cortada não é
transferida para o iso novo; escultura/canais/pesos retidos mantêm origem.

Hub calculado pelo ponto médio das portas do pai mais offset explícito. As
demais opções repetem os limites físicos do kernel de união. A aparência é uma
escolha obrigatória `parent`: a composição usa a receita material do pai, mantendo
os canais das regiões retidas. A fonte/material do filho permanecem intactos e
retornam ao remover a conexão. Shading e presença do canal material devem coincidir. Não prometer continuidade
de fibras, materiais por região ou C1 da malha triangulada.

`edit_plant`/`preview_plant` recebem `put_connection` e `remove_connection`.
`inspect_plant_connections` retorna receitas e relatórios. A edição de qualquer
planta, assembly, aparência ou transformação de instância reavalia o par no
staging antes da publicação. Rebase direto de fonte/rig/clipes de uma planta
conectada exige remover a conexão explicitamente, evitando usar a composição
como fonte. Remover/detach uma dependência deixa o documento inválido e recusa;
um lote pode remover conexão e dependência atomicamente. Undo/redo/load/replay
conservam fonte e receita. Load compara o asset derivado com sua recomposição,
sem aceitar cache adulterado.

Formato local desta frente: Scene72/replay76; Scene<=71 recusa connections,
inclusive um documento escondido sem entries. Compatibilidade entre branches
independentes continua exigindo integração explícita; números não bastam.

## Checklist anterior

1. Apenas staging/validação/consulta; nenhuma união/solver por tick.
2. Fontes Arc existentes; avaliar cada Entry uma vez por reconstrução do conjunto,
   Vecs e correspondências limitados, sem copiar fonte inteira por probe.
3. IDs/enum tipados, índices transitórios, mesma precisão f32/f64.
4. Reusar sample/cortes/união; CG escalar com adjacência esparsa e máximo256passos.
5. Até8 pares disjuntos, budgets existentes, preflight antes de concatenar.
6. Mesmo renderer/layout; máscara de consumo participa da invalidação do Frame.
   Sem readback/I/O por frame. Capturas offscreen e caminhos correntes.
7. Mesmo dispatcher/lock; nenhum worker ou dependência nova.
8. Reconstrução no fim do lote e nos previews; validação recomputa para conferir
   caches. Custo duplicado síncrono de autoria deliberado, não ganho de FPS.
9. Testar curvas de ambos/refino, fonte/rig/descendentes, máscara/remoção/undo,
   erros de domínio/identidade/ciclos/cache/versão, save/load/reset/replay/PNG.
10. Mantidos recorte rígido, material do pai e avaliação síncrona; grafo geral,
    edição de escultura no miolo novo e garantia global de embedding ficam fora.

## Revisão posterior por leitura

1. A união/CG são executados na autoria/validação/consulta, nunca no tick.
2. Fontes permanecem Arc; a avaliação base mantém a transferência existente.
   A composição precisa copiar posições/canais/rig retidos para um novo asset.
   Matching usa células/IDs e vetores ordenados; não procura vizinho espacial.
3. Bone bind usa a mesma Mat4 f32 do renderer; solver e residual usam f64,
   saída limitada a pesos f32 normalizados. Faixas/finito antes da publicação.
4. CSR esparso, Jacobi/CG com arrays reutilizados por iteração e256passos máximos;
   sem matriz densa. Nenhum trabalho do solver por vértice no render.
5. Até8 pares disjuntos,16assets, portas4..32 e budgets existentes. Retidos são
   contados antes de concatenar. O teto de campo/mesh continua recusando refinos.
6. Nenhum layout/shader/buffer novo. Frame guarda até8IDs consumidos, compara a
   máscara sem alocar e recria strings somente quando ela muda. O custo do scan
   é O(instâncias×conexões) no frame alterado; hierarquia continua incluindo filhos.
7. Mesmo dispatcher/transação; sem dependência, thread ou lock novo.
8. No staging/previews, recomposição seguida de validação recomputada deliberada:
   custo síncrono maior de autoria para conferir o cache. Sem cache de solver.
   Load, undo/redo e reset usam o mesmo documento; máscara faz parte da invalidação.
9. Testes isolados/MCP cobrem ambos desenhos, refino longitudinal, fontes,
   dependências, budgets, pureza/atomicidade, consumo e pixels de histórico/replay.
   O ensaio da mesma árvore confere também poses retidas e todos descendentes.
10. Mantidos: material único do pai, filho rígido, pares sem encadeamento,
    solver limitado e custo síncrono de autoria. Não certificar C1, embedding
    global ou realismo por manifold/contagem. Sem ganho de FPS alegado.

## Uso e reprodução

`examples/plant-connection.input.json` contém duas definições pequenas e a
receita de teste. `scripts/verify_plant_connections.py` constrói fontes/rigs e
verifica o contrato no executável MCP. A árvore usa a receita de648bytes de
`examples/native-tree.connection.json`, com as duas fontes anteriormente autoradas:

```
python3 scripts/demo_tree_secondary_plant.py
python3 scripts/demo_tree_plant_connections.py
```

O segundo comando aceita `--input` para o mesmo estudo com aparência refinada.
Cena corrente: `artifacts/plant-connections/examples/connected.scene.json`;
relatório/capturas: `artifacts/plant-connections/artifacts/`. Esses são gerados,
sem substituir a árvore canônica. A receita mantém dois documentos de plantas;
não inclui cópia da malha de junção no pedido MCP. A cena salva contém o cache
avaliado, que o load verifica novamente.

## Resultado inicial do ensaio da mesma árvore (f0dad0c)

Duas Entry/instâncias intactas, receita MCP644bytes; parent13307v/26610t,
retidos10825v do pai e657v do filho. União fechada/conectada, campo27144pontos;
CG96passos/resíduo8,65e-7. Fontes/escultura retidas preservadas, diferenças de
poses retidas em0/60/240≤0,292µm e demais poses/descendentes exatamente iguais.
Mudar separadamente as duas trajetórias reavalia a conexão e A-B-A retorna a
mesma cena; remover restaura o pai original. Undo/repeat/replay/prefixo/load
pixel-idênticos. Fonte com duas plantas e árvore canônica preservadas.

A vista clay é diagnóstico de geometria: mesma cena com material neutro e luz
sem sombra, sem alterar o arquivo conectado persistido. O colar mostra volume
excessivo e uma dobra inferior, ainda visíveis. Pico de dobra interno75,49°,
seam extraído11,72° e source seam0,0594°: nenhuma garantia C1/realismo visual.
Próximo avanço de forma deve reduzir esse volume/dobra e verificar animação;
não ampliar o domínio do contrato só para aceitar uma forma visualmente ruim.

## Refinamento do colar — plano anterior à autoria

Continuidade de f0dad0c com aceite visual do encaixe pelo usuário. Primeiro
comparar parâmetros existentes de densidade, raio/blend do centro e comprimento
do colar. Alternativa posterior seria mudar o campo/retopologia do núcleo;
escolher somente se a autoria atual não atender, sem aumentar budgets.

Checklist proporcional:1.só autoria/consultas, sem tick novo;2.fontes Arc e
mesmo prefixo retido, estudos guardam métricas e PNGs, não cópias de builds;
3.receitas tipadas existentes, análise de área/volume em f64 Python;
4.máximo12candidatas explícitas por rodada (duas rodadas limitadas), sem busca/otimização escondida;5.mesmos tetos,
falhas preservam cena/revisão;6.capturas nativas offscreen, câmera/luz fixas;
7.sem dependência/thread/lock;8.mesma invalidação/regeneração do documento;
9.comparar topologia, retenção, poses/descendentes, A-B-A e replay na escolhida;
10.manter limites de pares rígidos e material único. Métrica de dihedral máximo
isolado pode refletir slivers; registrar também distribuição ponderada por área,
volume e vistas, sem reduzir qualidade à contagem ou ao solver.

### Resultado do refinamento

Estudo nativo em study_plant_connection_shape.py com controle congelado de
f0dad0c e19receitas explícitas em duas rodadas de12/7 (a referência se repete).
Inputs em examples/native-tree-connection-shape.input.json; relatórios e vistas
em artifacts/plant-connection-shape. Escolhida down080: centro0,08m (era0,09),
blend0,05m (era0,025), offsetY−0,025m e cell0,024m (era0,028).

| Medida do mesmo encaixe | Controle f0dad0c | Receita refinada |
|---|---:|---:|
| Dobra máxima entre faces novas/adjacentes |75,49°|40,60°|
| Dihedral p90 ponderado por área |11,88°|10,54°|
| Dihedral p99 ponderado por área |24,67°|18,65°|
| Área da superfície nova |0,36476m²|0,35428m²|
| Volume fechado do asset |1,75328m³|1,75231m³|
| Costura fonte/guarda máxima |0,0594°|0,0594°|
| Costura extraído/zipper máxima |11,72°|16,14°|
| Vértices/triângulos do asset |13307/26610|13829/27654|
| Pontos de campo / CG |27144/96|48160/108|

A variação densamente amostrada soft060dense chegou a65520/65536pontos e foi
preterida apesar de boa distribuição de dobras: somente16amostras de folga
fragilizavam novas edições. Densidade020 e colar050 foram recusados pelo kernel,
sem mudar cena/revisão. Centro065 também recusou dobra≥90°. Nenhuma guarda ou
teto foi relaxado. Tradeoff da escolhida: +522v/+1044t e aumento da costura no
lado extraído, compensados por menor pico/distribuição de dobras e menor volume.
O colar ainda tem relevo/concavidade; não declarar que todas as dobras sumiram.

Demo agora aceita --recipe e --compare-recipe para comparação conectada com o
controle, mantendo vistas frontais/laterais. O ensaio confirma mesmos10825v/657v
retidos, fonte/rig/clipes/instâncias intactos, poses retidas≤0,292µm, demais e
folhas/rametas exatas; oito poses novas com cos mínimo0,999511 e sem inversão
observada. Não é prova global de embedding. A-B-A, unlink, undo e pixels
repeat/replay/prefixo/load exatos.92002t desenhados, abaixo do teto131072.

Reprodução:
```
python3 scripts/study_plant_connection_shape.py
python3 scripts/study_plant_connection_shape.py --phase refined
python3 scripts/demo_tree_plant_connections.py --compare-recipe artifacts/plant-connection-shape/examples/baseline.recipe.json
```

Revisão posterior:1.só autoria/MCP, sem tick novo;2.retenção exata, arrays de
normais/arestas temporários limitados à mesh;3.f64 na análise e receita nativa
f32 existente;4.O(TlogT) para percentis/arestas, sem otimizador/loop ilimitado;
5.mesmos limites, recusas preservadas;6.só capturas explícitas, mais1044t no
resultado é custo mantido, sem FPS medido;7.sem concorrência/dependência nova;
8.mesmo cache/undo/receita;9.demo/poses/pixels e4cenários MCP/20grupos aprovados;
10.formato/engine Rust intactos, materiais/árvore completa/pares encadeados
seguem fora desta unidade. O campo passa de27144 para48160amostras na autoria;
maior custo síncrono é deliberado, não uma otimização de runtime.

## Segundo enxerto estático — plano anterior à autoria

Continuidade de8eb4a9c. O próximo componente escolhido é secondary02, no braço
estático esquerdo. A fonte já contém a curva gerada pelo loft nativo; extrair
por ilha autorada incidenta no vértice35 da wood atual, validando119v/234t e
canais. A montagem existente aceita outro ContextGraft com cortes canônicos
separados: anéis contendo1647 e1711, fora do primeiro enxerto. Não converter a
fonte geométrica estática em promessa de uma nova Entry paramétrica animada.

Alternativas: ampliar Connections para irmãos/árvores exigiria novo contrato de
ordenação, skin e orçamento; adicionar instâncias também encontra o limite32.
Esta unidade usa o enxerto estático já suportado, sem contornar essas guardas.
A fonte extraída fica na receita da montagem e deixa a wood em transação única;
sem duplicar desenho ou apagar o vínculo das duas plantas animadas aprovadas.

Checklist prévio:1.só autoria/MCP, sem tick;2.fontes existentes e extração exata,
sem cópia de composição usada como fonte;3.IDs/células canônicos e frame explícito;
4.até8 opções declaradas de forma, sem busca ilimitada;5.mesmos limites de mesh,
campo e32instâncias, recusar sem mutação;6.capturas offscreen;7.sem dependência/
thread;8.previews e lote único, histórico reversível;9.manifold, fonte/rig/poses,
encaixe aprovado, A-B-A e replay/prefixo;10.estático na montagem, não grafo geral
ou rig novo. Medir custos finais e preservar canônica/main/demo.

### Segundo enxerto estático — resultado

A receita examples/native-tree-secondary02.graft.json integra secondary02 ao
braço esquerdo por ContextGraft, preservando as duas Entry e a Connection
animada de8eb4a9c. Fonte extraída119v/234t,91v retidos; recorte de160v no contexto
avaliado é intencional. A fonte/contexto canônicos continuam guardados intactos,
com o segundo enxerto declarado na assembly. Não é uma terceira planta com
trajetória/rig independentes; a curva da fonte já veio do loft nativo anterior.

Oito opções declaradas (6iniciais+2refinos),5recusadas por orçamento de mesh ou
seam. Escolhida round065: raio0,065m/blend0,065m/célula0,032m,5relaxamentos;
21204pontos de campo,1822v iso, dobra máxima51,90°, source seam4,34°/extraído22,78°.
A fonte de7lados conserva sua resolução; conexão fechada/manifold não prova C1.
Não se aumentou limite para fazer uma variante caber.

Resultado total do asset15653v/31302t e95416t desenhados,32instâncias mantidas.
Dos13829v anteriores,13669permanecem com posições/pesos exatos,160removidos no
recorte. Os3004pontos da região do encaixe aprovado mantêm também material e
triângulos, e sua captura clay é byte-idêntica. Poses retidas anteriores e dos
outros objetos/folhas/rametas exatas em0/60/240. A fonte estática recolocada tem
erro máximo0,119µm no frame f32; oito poses novas com cos mínimo1, sem inversão
observada. Sem prova global de embedding. Fontes/ref/def/contexto/clipes/ossos e
primeiro enxerto preservados. Extração conserva exatamente posições/canais/
triângulos em wood+fonte, sem duplicar o galho.

Previews/lote, remoção+restauração da ilha, undo/redo, A-B-A das duas plantas e
parâmetro do novo enxerto, save/load e pixels repeat/replay/prefixo/load passam.
Regressão atual4MCP/19grupos (render,plant_assemblies,plant_material_frames,
plant_connections); Python/fmt/diff aprovados. Mesmo binário ec96a3441fef43e2,
sem Rust/formato novo. Não repetir os850Rust como se fossem desta unidade.

Reprodução com a cena de duas plantas já conectada:
```
python3 scripts/demo_tree_second_graft.py --study
python3 scripts/demo_tree_second_graft.py --study --refine
python3 scripts/demo_tree_second_graft.py
```

--input seleciona a cena de origem; --recipe seleciona o ContextGraft.
Geometria/frame da fonte são conferidos contra a extração; o material atual da
fonte extraída é preservado para permitir o mesmo desenho com outra aparência.
A cena/capturas/relatórios correntes ficam em artifacts/tree-second-graft;
nenhuma alteração na canônica/main/demo. A geometria estática unida passou a
ser parte da montagem do documento da planta; fontes separadas são reversíveis.

Revisão posterior:1.só autoria;2.extração/correspondências exatas, fontes
imutáveis, nenhuma composição usada como fonte;3.frames f32 nativos, análise
f64 limitada;4.oito opções finitas e correspondência por índices/dados exatos,
sem nearest;5.tetos mantidos e5recusas, sem instâncias novas;6.mais1824v/3648t no
asset e custo de nova união síncrona na autoria, sem ganho de FPS;7.sem thread/
dependência;8.mesmo staging/cache/undo;9.verificações acima e auditoria de fonte;
10.estático e fonte de7lados mantidos, restante da árvore/realismo global abertos.

Fricção seguinte:731v livres no teto16384deste asset e nenhuma instância livre.
Outro enxerto deste porte não cabe no mesmo asset. Antes de ampliar a árvore,
avaliar particionamento da saída gerada com fronteiras/skin compartilhados ou
assets de montagem separados; elevar o teto é alternativa com custo de arrays,
expansão/upload e regressão. Isso é decisão estrutural futura, não capacidade
implementada por esta unidade.
