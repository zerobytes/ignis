# Continuidade orgânica intencional

Os quatro recortes de revisão da árvore expõem problemas de autoria distintos:
cotovelos de uma varredura pouco amostrada, junções com variação brusca de
normais e peças desconectadas que se atravessam. Normais interpoladas não
corrigem a silhueta. Subdivisão linear não muda a forma. Um filtro de superfície
não une ilhas nem elimina suas faces internas.

## Escolhas anteriores ao código

1. Não aplicar Smooth radial indiscriminado: encolhe ramos, pode deslocar
   sockets e não resolve conexão. Preservar seu resultado legado.
2. Acrescentar um loft cúbico entre anéis **nomeados por índices de fonte**.
   Todos os vértices da fonte ficam exatos; pontos novos usam correspondência
   longitudinal explícita para coordenadas de material e pesos. Interpolação
   linear continua disponível para intenção angular. A cúbica interpola as
   colunas da seção; não promete seção circular exata, isometria ou volume
   conservado entre anéis. Não é ajuste por superfície mais próxima.
3. Filtro de superfície geral usa seleção e âncoras explícitas, passos finitos
   e envelope de deslocamento. Opera sobre superfícies conectadas; ferramenta
   complementar ao eixo e às operações de recorte/junção existentes.
4. Antes de publicar a árvore, comparar silhueta e encaixes em vistas neutras
   e com casca, medir fonte/rig e testar regeneração. Não classificar como
   árvore inteiramente resolvida uma prova de somente um desses recortes.

O loft recebe 2..128 anéis de 3..32 vértices, orientação e colunas consistentes,
1..16 subdivisões por intervalo. Cada célula deve corresponder a exatamente
dois triângulos existentes. Não inferir a ordem dos anéis pela posição. Rejeitar
índices/duplicatas/células ausentes e sobreposição de patches. O limite de desvio
da cúbica em relação à interpolação linear é certificado pelo casco convexo dos
controles de Bézier da diferença, além de medir os pontos produzidos. Guardas
geométricas recusam colapso e inversão local; não são prova global de embedding.

O bound é relativo ao loft linear das mesmas colunas. Ele não é distância
Hausdorff à triangulação original nem tolerância de discretização da cúbica.
Mesmo o modo linear pode retesselar quads não coplanares de forma
diferente da diagonal antiga; para subdivisão conformante sem mudar as faces,
usar Subdivide. Continuidade entre dois patches distintos não é inferida.

Fonte, material e rig continuam explícitos. Vértices novos interpolam os pesos
dos dois pontos correspondentes, recusando mais de quatro influências. Preservar
pesos/bones/clips não significa que a forma nova tenha poses idênticas à antiga.
Fora do patch a geometria fica exata. Plantas compostas não podem ser desligadas
ou rebased implicitamente para contornar as proteções da receita.

## Referência de ferramentas

O [manual do Blender](https://docs.blender.org/manual/en/4.2/sculpt_paint/sculpting/tools/mesh_filter.html)
separa Smooth, Surface Smooth, Relax Topology e Sharpen. A distinção é útil:
regularizar a forma, redistribuir amostras e preservar uma quina são intenções
diferentes. No MCP, expressá-las por seleção, referências, restrições e relatório,
em vez de exigir simulação de movimentos de mouse. A lista de pincéis não
substitui o contrato geométrico da junção ou a autoria da trajetória.

Inventário por finalidade, baseado no [catálogo de pincéis](https://docs.blender.org/manual/en/5.1/sculpt_paint/sculpting/brushes/brushes.html)
e no [manual de filtros](https://docs.blender.org/manual/en/4.4/sculpt_paint/sculpting/editing/sculpt.html):

| Intenção | Referência Blender | Ignis nesta unidade |
|---|---|---|
| Acrescentar/modelar massa | Draw, Clay, Inflate | Draw existente; demais não implementados aqui |
| Deslocar forma | Grab, Elastic Deform, Pose | Grab existente; rig/poses existentes não equivalem ao pincel Pose |
| Suavizar forma | Smooth, Surface Smooth | Novo SmoothRegion com seleção/âncoras/envelope; laplaciano, sem conservação de volume |
| Redistribuir topologia | Relax Slide, Relax Topology | Loft entre seções e subdivisão existentes têm propósitos distintos; relax tangencial ainda pendente |
| Proteger regiões | Mask, Face Sets | Seleção explícita, referências e pontos fixos; sem máscara pintada/Face Sets completos |
| Quina/dobra intencional | Crease, Pinch, Plane | Interpolação linear/pontos fixos disponíveis; esses pincéis não foram copiados |
| Unificar superfície | Remesh, operações de malha | Recortes/bridge/união de três portas; filtro não une ilhas |
| Detalhe reversível | Layer, multires | Fontes canônicas e acabamento persistido por origem; não é um sistema multires geral |

## Ordem e persistência do acabamento

`context_lofts` opera sobre anéis da fonte canônica, depois `context_surface`
atua com uma máscara também canônica. Um ponto novo herda seleção ou âncora
**somente se ambos os endpoints** de sua correspondência longitudinal a possuem.
Assim a receita não guarda índices efêmeros do resultado. Portas da junção
precisam permanecer exatas. Recorte de uma face substituída pelo loft é recusado,
sem escolher uma filha arbitrária.

`junction_surface` recebe somente opções: sua seleção é o intervalo de vértices
iso recém-gerados da avaliação atual. Fontes retidas e primeiras guardas ficam
fixas. Relatórios da união referem-se à etapa anterior ao filtro; o relatório
de acabamento informa os ângulos e o volume finais, com a sua região explícita.
Manter pesos/bones/clips não implica poses idênticas dos pontos remodelados.
O documento guarda as fontes originais intactas e as operações intencionais.

## Diagnóstico que mudou a seleção

O ensaio da máscara completa do pescoço recusava até2×.03/.003. A causa foi
localizada na tampa distal:14arestas já tinham90..94.44°, e mover o terceiro
vértice de suas faces laterais piorava a dobra apesar de os endpoints estarem
fixos. Isso não demonstrava falha do cotovelo. Proteger os dois últimos anéis
canônicos também protege o intervalo novo entre eles e isola esse problema.
Os erros agora identificam etapa, face/aresta e ângulos para permitir diagnóstico
MCP, mantendo todas as guardas. A tampa protegida continua existindo; não contar
sua exclusão da região como melhoria de94° para44°.

Uma prova separada de abertura7→ramo7 com o bridge existente recusou21 propostas
finitas (fase/handles) por dobra. O estudo está em artifacts/graft-probe e não
foi instalado como junção. As interseções restantes precisam de uma operação
de união adequada à sua geometria; nenhuma guarda foi relaxada.

## Gate restante da árvore

As varreduras antigas refinadas continuam em assets estáticos, além da única
planta composta regenerável. Não são22novas plantas independentes. A composição
retém contexto e rig, porém ainda não é um grafo paramétrico de toda a árvore.
O ressalto de uma peça atravessada e as outras interseções não foram resolvidos
pelo filtro. Não marcar EAF-80/EAF-85 concluídos com base neste incremento.

Alternativas para esse próximo recorte: uma união/remalha geral elimina faces
internas mas troca a topologia e exige novo contrato de correspondência do
detalhe/skin; um grafo de ramificações explicita pai, estação e continuidade de
cada porta, preservando o domínio das plantas e regenerando só o encontro.
Recomendação é o grafo e suas junções locais, mantendo união geral como ferramenta
separada. É preciso ainda estabelecer tolerância de continuidade/discretização
e quinas autoradas por região: um limite de recusa90° não é critério de realismo.

## Revisão estática anterior

Ensaio nativo anterior à mudança de armazenamento: interpolar os21 tubos antigos
leva wood de1841 a4085 vértices e2974 a7462 triângulos; o mesmo pescoço esquerdo
no contexto passa de2127 a3407 vértices, sem mover os originais. A composição
atual de7357 passaria a8637, acima8192. O documento previsto fica19732v33374t,
acima16384v32768t; draw previsto81918t continua abaixo131072. Portanto ampliar
apenas armazenamento: asset16384v32768t, total32768v65536t, com gateScene67.
Draw, grade de união, intermediários da união, ossos/instâncias continuam iguais.
Isso dá espaço à discretização da curva, sem tratar teto maior como qualidade.
Os números são previsão da edição, a confirmar na cena publicada.

1. Apenas comando de autoria; nada por tick/frame.
2. Preparar tabelas e capacidades uma vez; sem coleções por ponto amostrado.
3. Índices u32 e cálculo f64, publicação f32; fonte compartilhada imutável.
4. Indexar faces uma vez; não procurar cada célula varrendo toda a malha.
5. Tetos de anéis, lados, subdivisões e saída antes de alocar a saída expandida.
6. Sem GPU/I/O/dependência novos no kernel; captura continua sob demanda.
7. Mesmo dispatcher e transação existentes.
8. Resultado publicado pelos caminhos existentes invalida cache/undo corretamente.
9. Preview puro, recusas atômicas, topologia e canais testados antes do uso artístico.
10. Cópias limitadas de autoria e triangulação explícita mantidas; sem alegação de FPS.

## Encaixe lateral persistido — decisão anterior ao incremento

O ressalto no pescoço esquerdo é a tampa do secundário0 (wood0..34 na
fonte anterior ao loft), não uma dobra conectada. Uma prova pelo kernel nativo
usa dois cortes no ramo pai (anéis canônicos1903 e1999), remove a raiz do filho
até seu segundo anel e une as três portas32/32/7. A grade39527 e a guarda
existente aceitaram o caso; o recorte não muda dados retidos. Captura isolada
em artifacts/local-graft; prova local não é ainda receita regenerável nem árvore
inteira aprovada.

Escolha: acrescentar enxertos estáticos explícitos ao contexto da assembly,
depois de loft/filtro e antes da bifurcação viva. Cada receita guarda fonte
filha imutável, faces cortadas nela, sua porta, máscara de corte e duas portas
canônicas do contexto, opções de união e acabamento opcional. A máscara se
expande a vértices novos somente quando ambos endpoints canônicos pertencem a
ela. Remove-se uma face somente se todos os seus vértices pertencem à máscara.
Mapas exatos de compactação mantêm referências canônicas dos enxertos seguintes
e da bifurcação principal; referência removida ou corte sem as portas declaradas
é erro. Sem coordenadas próximas, rebake, novos pesos inferidos ou índices
persistidos do miolo transitório. Fontes estáticas recebem o root explícito já
exigido pela assembly; fonte com rig/clipe é recusada neste recorte. Não é ainda
um grafo de plantas animadas independentes nem enxerto com skin multibone.

União/remalha geral é alternativa útil mas exige novo contrato de correspondência
de detalhe/skin em toda a região. A operação local preserva as fontes, tem custos
limitados de autoria e pode recusar suportes/raios incompatíveis. Mesmo conectado,
o campo pode arredondar demais o encontro e o domínio de casca precisa avaliação
visual separada. Quinas intencionais continuam possíveis sem filtro e por fontes
não substituídas pelo recorte.

Checklist prévio: (1) apenas avaliação de edição; (2) um mapa de compactação por
corte e capacidades calculadas; (3) IDs u32/f64 no kernel/f32 publicado;
(4) tabelas de triângulos e máscaras uma vez por enxerto; (5) até8 enxertos e
mesmos tetos de união/saída, sem ampliá-los; (6) sem GPU/I/O no kernel;
(7) dispatcher/transação existentes; (8) reconstrução determinística/undo sem
cache novo; (9) fontes imutáveis, recusas atômicas e replay versionado;
(10) cópias de autoria e grade limitada conscientemente mantidas, sem alegar FPS.

## Continuidade do colar — decisão antes do incremento69

O anel remanescente não é tampa atravessada: a guarda fixa da união impede o
filtro iso de redistribuir curvatura entre superfície retida e miolo. Provas
nativas em artifacts/collar-study mantiveram todas as guardas: seleção ampla
32×.3/30mm piorou pico51.86→63.05° e foi recusada; seleção só dos dois colares
parent com32passos baixou pico mas piorou média6.0103→6.1710° e foi recusada.
A mesma região com4×.3/10mm passou: pico regional41.2277→23.9936°,
média6.0103→5.8012°,765pontos movidos/1154selecionados/389pinos.
Capturas neutras na mesma vista mostram o anel suavizado; isso não fecha os
outros encontros/casca/realismo completo.

Escolha: `ContextGraft.collar_surface` opcional, posterior ao filtro iso, com
portas por ordinal0..2 (as2do contexto e a filha), alcance em anéis topológicos,
número de camadas externas fixas e opções do filtro já existente. Resolver as
portas canônicas e então derivar seleção/âncoras por BFS de arestas únicas a
cada avaliação. Fonte/miolo/guardas têm índices transitórios, não persistidos.
Essa camada explicitamente PODE remodelar pontos retidos e primeira guarda;
o relatório da união continua anterior a ela. Fontes originais, canais materiais,
triângulos/IDs/bones/clips ficam intactos. A afirmação de vértices retidos exatos
é do kernel cru, não da avaliação após esse acabamento intencional. Portas da
assembly principal não podem ser deslocadas por esse acabamento: recusar caso
ocorra. Fora da seleção e camadas fixas permanecem bit-exatos.

Alternativas: novas normais de shading esconderiam parte do sintoma sem corrigir
geometria; estender o campo implícito para reproduzir derivadas completas das
seções é uma alternativa mais geral, mas amplia seu contrato e não é necessária
para a prova aceita. Não alterar algoritmo/guardas de SmoothRegion nem usar uma
busca de parâmetros que publique candidatos recusados. Alcance topológico depende
da tesselação; reavaliar quando ela muda, sem fingir raio físico ou C1/C2 analítico.

Checklist prévio69: (1) só autoria; (2) um CSR e fila limitada ao número de
vértices, sem alocações por camada; (3) u32 IDs/u8 distâncias com limites;
(4) topologia preparada uma vez por seleção; (5) alcance1..16, fontes/saída com
mesmos tetos; (6) sem GPU/I/O novo; (7) mesmo staging atômico;
(8) receita reavaliada, sem cache novo; (9) defaults legados e gate69, preview/
publicação/undo/replay/seleção exata e recusas; (10) inspeção topológica adicional
por edição mantida, sem alegar FPS/volume conservado.
