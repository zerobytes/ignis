# Inspeção do domínio material

Decisão anterior ao código, EAF-80/EAF-85: medir o domínio antes de mudar a
casca. A consulta é genérica e pura. Em cada triângulo, construir uma base
ortonormal da superfície local e calcular o Jacobiano2→3 das coordenadas
materiais interpoladas. Seus dois valores singulares medem unidades materiais
por metro da geometria local; razão maior/menor mede anisotropia. Não são
alongamentos relativos a uma mesh original. Taxas grandes significam mais
variação material por metro, não alongamento físico da madeira.

Opções: domínio espacial ou projeçãoXY/XZ/YZ (como os detalhes do shader),
escala positiva explícita por eixo, até32amostras. A escala é aplicada antes da
projeção; não é inferida da receita. Coordenadas ausentes usam posições locais
de repouso, igual ao renderer, e isso é rotulado no relatório. Sem aplicar
skinning, transform de instância, warp/noise, iluminação, frequência implícita
ou pixel footprint. Projeção planar pode colapsar em faces laterais mesmo com
domínio3D regular; não confundir isso com defeito topológico.

`inspect_mesh_material_domain` consulta um asset presente e retorna referência
de sessão; `preview_mesh_material_domain` recebe Geometry explícita, sem asset,
rig ou publicação. Faces opcionais: ausente significa todas; lista vazia,
repetida ou fora do domínio é erro. A seleção é ordenada para determinismo.
Estatísticas cobrem TODA a seleção; amostras são só as piores por anisotropia,
com colapsos primeiro e desempate por face. Índices são da geometria consultada,
não correspondências persistentes. Reconsultar após regenerar.

Colapsos exatos de posto0/1 são contados. Posto vem da aritmética finita e não
é certificação algébrica. Taxa mínima<=1e-10da máxima sinaliza singularidade
numérica: anisotropia null, excluída também de máximo/média de anisotropia.
Não apresentar razões instáveis de1e20como melhora/piora finita de material.
Média de anisotropia ponderada por área local exclui singulares; área elegível
é explícita. Não certifica orientação das fibras, ausência de sobreposição,
qualidade visual, texel density em pixels, preservação de volume ou injetividade.
Faces com dobro da área<=1e-12da maior aresta ao quadrado são recusadas com ID:
a inversa é numericamente instável; não confundir isso com colapso do material.
Alta anisotropia pode ser intencional. Sem mapa de calor/novo renderer neste recorte.

Alternativas: usar somente comprimentos de arestas depende da triangulação e
pode perder a direção extrema dentro da face. UV unwrap/transporte de frames
resolveria outra etapa e muda o contrato de autoria; adiado até o diagnóstico.

Checklist de performance anterior:1.somente consulta, zero tick/frame;
2.sem cópia de geometria, lista de faces opcional e top32limitado;
3.f64 temporário, índices/finito validados;4.um passe nas faces selecionadas,
validação básica O(V+T), sem adjacência ou solver;5.mesmos tetos de Geometry,
top32e transporte MCP2MiB;6.nenhum GPU/layout/readback;7.sem dependência/thread/
lock;8.sem cache/Scene/replay novos;9.testes analíticos de rotação/escala/shear,
projeção/colapso/determinismo/limites e pureza MCP, depois snapshot da árvore;
10.custo mantido de validação integral e topK por inserção O(32T), sem FPS alegado.

## Estudo na árvore existente

Repro: `scripts/study_tree_material_domain.py --binary target/release/ignis
--before <tree-collar/examples/before.scene.json> --after
<tree-collar/examples/after.scene.json> --junction-before
<collar-study/artifacts/baseline.json> --junction-after
<collar-study/artifacts/filtered.json> --junction-source
<local-graft/artifacts/input.json>`. Caminhos são snapshots da integração
native-plants. Saída estável `artifacts/tree-material-domain/report.json`,
com hashes dos inputs e build_id. Recebe somente Geometry explícita por MCP:
o binário desta unidade é Scene68/replay72, não valida o load da receita69.
Fontes/cena/material não são publicadas nem alteradas pelo estudo.

Snapshot completo anterior SHA31ffe970…a4cb0; atual SHA89fe1fa2…89400.
As765posições do collar mudaram; triângulos e coordenadas materiais continuam
iguais. Na união local isolada, classificar faces por prefixo da fonte de3242v
e últimos71v de guarda, não por proximidade:6407faces fonte,4865iso,142primeira
guarda,220zipper. A primeira guarda usa material[ma,mb,ma]/[mb,mb,ma]:
**142/142faces de posto1 em3D antes e depois**. EmXY são2posto0+140posto1.
Sua área geométrica cresce0.01493344→0.03530077m² enquanto o domínio continua
sem uma segunda direção. Isso é preservação de valores, não de derivadas.

Anisotropia3D média ponderada por área: fonte1.04301→1.04244,iso6.78670→6.79458,
zipper5.08005→5.80355; primeira guarda sem média regular(null). Máximo iso
1626.695,face7274, nos dois estados. São regiões e métricas diferentes dos
ângulos usados para aceitar o filtro geométrico; não declarar regressão da
forma ou melhora material global com esses números. XY isolado acrescenta
59singulares na fonte e1no zipper, e dá média iso154.54depois: projeção tem
efeito próprio. A consulta não inclui warp/luz/aliasing.

No asset composto completo branch-editable-1:334faces de posto1 em3D nos dois
snapshots,522singulares emXY. Wood tem0singulares3D/1420XY;secondary-06 tem0/42.
Não atribuir todo o sintoma ao novo collar: outras junções/projeções já existem.

Próxima prova proposta: extrapolar derivada tangencial de material da faixa
retida à guarda, mantendo sourcecoords exatas e continuidade na borda original.
Isso muda o contrato material e a antiga expectativa guardcoords==sourcecoords;
deve ser opt-in, com legado exato e validação de iso/zipper para não deslocar
a distorção ao próximo anel. Gradiente de coordenadas não define fibra física.
Nenhuma correção de material foi implementada por esta consulta.

Coordenação do estágio: medir/terminar o domínio na GEOMETRIA FINAL, após iso e
collar. O collar moveu64guardas e320pontos retidos; extrapolar antes e depois
conservar coordenadas pode reintroduzir distorção. Candidato a debater: finisher
material puro separado, source faces/pares source→guard/intervaloiso explícitos,
sourcecoords fixas e política CopyGuard legada. Nova política só no union é
alternativa, mas exige recomposição pós-collar e cuidado com dupla difusão.
Este parágrafo registra uma proposta; não é contrato nem API implementada.

## Integração na frente de plantas69

A consulta também foi integrada sobre e1fef8f, mantendo Scene69/replay73.
O script aceita `--load-after` opcional: copia o snapshot para o processo isolado,
valida seu load com o binário usado e compara inspeção por asset à query por
Geometry. Sem essa flag, mantém o estudo por dados explícitos compatível com a
unidade anterior. Essa validação adicional não publica nem edita a cena original.
A referência afim/finisher da proposta continua uma unidade separada.
