# Roadmap de modelagem nativa

Direção: autoria nativa na Ignis, MCP-first, com comandos compartilhados e
equivalência de ferramentas visuais quando estas forem implementadas. Plantas e
modelagem direta publicam o mesmo tipo de asset; fonte canônica e correspondências
explícitas mantêm as garantias de preservação separadas por detalhe, pesos, rig e
clipes. Resultados automatizados não substituem integração/aceite visual.

| Etapa | Estado e escopo |
|---|---|
| Seção e repetição de traços | Rounded e domínio periódico2D opcionais no mesmo Detail/MCP,32segmentos mantidos. Scene71/replay75; prova na árvore integrada com geometria/neutros preservados. Fibra natural, materiais regionais e realismo seguem abertos. Contrato detail-domains.md. |
| Diagnóstico do domínio material | Consulta MCP pura por face: taxas principais, anisotropia e colapsos3D/projeção, com seleção e relatório limitado. Estudo da mesma árvore identifica142faces de guarda colapsadas; transporte de derivadas disponível no finisher opt-in da geometria final; referencial afim explícito conserva fontes canônicas. Projeção/casca e realismo continuam gates separados. Contrato mesh-material-domain.md. |
| Criação, escultura, rig e keyframes | Primeiro recorte implementado em EAF-76; assets triangulados, pincéis draw/grab/smooth, rig genérico e clipes locais. Ferramentas manuais e associação a NPC continuam gates próprios. |
| Fonte e regeneração | Kernel EAF-78 e plantas EAF-77: perfil retangular/extrusão, dimensões/tesselação, fonte canônica, políticas físicas/proporcionais, erro amostrado e falhas atômicas. Não constitui solver CAD geral. |
| Componentes e subdivide | EAF-79: vértices/arestas/faces, referências de sessão/revisão, subdivisão linear conformante e refinamento uniforme declarado em plantas. Contrato em [mesh-topology.md](mesh-topology.md). |
| Aparência de superfície | Projeção triplanar de detalhes opt-in em70/74, cache por GeometryArc e fallback finito; direção da fibra/realismo ainda abertos. EAF-80: flat/smooth por índices, sombra direcional limitada e material procedural com canal3D de repouso; não é PBR completo. Contratos mesh-shading.md, directional-shadows.md e procedural-materials.md. |
| Junções orgânicas | EAF-85: preview puro MCP de3colares, fronteiras por índices, campo limitado e diagnósticos; primeira forquilha integrada na árvore EAF-80. Sem solver de árvore inteira ou preservação de miolo novo. |
| Curvas de seções editáveis | EAF-87: elipse/offsetXZ e modo suave PCHIP opt-in integrados no mesmo ramo da árvore. Y ordenado/planos paralelos, fonte original conservada; sem sweepXYZ livre/frame transportado. EAF-85 adiciona condição de tangência opcional ao campo de junção; o primeiro cotovelo recebe taper e eixo compatíveis, com comparação geométrica. Outros encaixes e a tesselação adaptativa por curvatura continuam abertos. |
| Trajetória e cortes normais | EAF-97: curva cúbicaXZ/Ylinear, frames normais e inversa única por critério conservador, raio constante demonstrado no cilindro isolado. Editável pelo MCP com fonte canônica, erro lateral amostrado e rejeição de resolução/volume inadmissíveis. Loops, transporte paralelo geral, certificação Hausdorff e aplicação às bifurcações não fazem parte deste recorte. Contrato plant-trajectories.md. |
| Continuidade com materiais | EAF-80/EAF-85: transição de colares e curva/taper local integrados à mesma árvore com reflectance EAF-94 conservado. Primeiro encontro46.10°→10.62°, demais encaixes e facetas ainda abertos; sem novo formato, teto ou gerador botânico. Contrato tree-geometry-material-integration.md. |
| Vínculos entre componentes | EAF-89: instância/ossos nomeados com TRS local e hierarquia acíclica,58/62. Um secundário e copa testados em edição/movimento na árvore salva; demais componentes ainda não convertidos. Não infere tangente da curva ou solda. Contrato mesh-attachments.md. |
| Distribuição e organização | EAF-90 acrescenta inspeção/extração Move/Copy de ilhas com preview genérico e preservação. Scatter por domínio com seed/densidade/máscaras, picking por subobjeto e diagnóstico de vazios/colisões continuam pendentes. Contrato mesh-islands.md. |
| Detalhe de superfície/folhas | EAF-91: traços genéricos no domínio material (tinta/bump), ajuste de domínio por ilha e transferência por topologia explícita; nervuras e sulcos aplicados à árvore. Espessura física, fibra seguindo frames de ramos, materiais por região/face, cores por vértice e UV/PBR completos continuam pendentes. Receita não conclui esse fluxo. |
| Rugosidade/reflexo direto | EAF-94: resposta dielétrica GGX opcional, testada em folhas/casca da mesma árvore, câmera/luz/pose fixas e ablações. Sem IBL/metais/HDR/reflexos de cena ou PBR completo. Alias especular e rugosidade por domínio/máscara continuam pendentes. Contrato dielectric-response.md. |
| Resolução e orçamento de autoria | Parcial: preview/validação limitados existentes. Seleção de resolução por região, preview do custo composto e LOD ainda pendentes; o script calcula a contabilidade. |
| Escultura e vento avançados | Pendente: máscaras/picking de superfície, autoria facilitada de pesos e sinal de vento/hierarquia. Pincéis/rig/clipes básicos já existem. |
| Extrude, inset e bevel | EAF-130: extrusão de região conectada com fronteira, inset plano convexo e bevel plano de cadeias em sólido convexo fechado. MCP/UI no modo Editar, preview/arraste/undo/seleção resultante e preservação de canais/rig/clipes/budgets; plantas exigem detach explícito. Bevel curvo/côncavo e inset arbitrário seguem abertos. [Contrato](editor-modeling.md). |
| Subdivisão suavizada | Posterior: escolher esquema e representação de polígonos, tratamento de bordas/vincos e preservação. Loop e Catmull-Clark não estão implementados pela subdivisão linear. |

Cada incremento precisa validar consultas/edições pelo MCP, atomicidade, reversão,
persistência e observação do resultado. Recursos que excedem a representação da
fonte devem falhar com diagnóstico ou exigir desligamento explícito do vínculo;
não substituir trabalho autoral por um bake incompatível silenciosamente.

O estado de cada entrega e seus gates estão em [validation.md](validation.md).
Este roteiro ordena a modelagem já discutida; não conclui os sistemas completos de
escultura, rigging e animação nem agenda suas extensões avançadas.

## Exercício de autoria — árvore, EAF-80

Guilherme priorizou descobrir dificuldades das ferramentas usando como alvo uma
árvore de alta qualidade e realismo. A demo low poly foi aceita como ponto de
partida. A primeira rodada ampliada usa2100 folhas, vistas fixas e detalhes;
o alvo artístico continua aberto. Sintomas, tentativas que não funcionaram,
workarounds, custos e capturas estão em [tree-authoring-lab.md](tree-authoring-lab.md).

O exercício aponta para sweep por curva/perfil com raio variável e conexões
editáveis, organização/scatter por subobjetos, controle de normais e materiais.
São incrementos candidatos derivados de uso, não um subsistema exclusivo de
árvores. Alinhar curvas ao contrato de perfis/loft das plantas paramétricas e
as capacidades de aparência à frente gráfica antes de implementar contratos.
Extrude/inset/bevel continuam no roteiro; a sequência imediata deve considerar
as fricções reais e a prioridade atual do exercício da árvore.


Segunda rodada EAF-80:1782folhas26v/32t, primeira forquilha compartilhando
bordas, colo basal contínuo, material e sombra;64070t desenhados sob o teto
revisado65536, demais budgets mantidos. Alvo artístico segue reprovado no close.
Ordem imediata derivada: curvas/conexões editáveis e controles de domínio da
superfície; folhas/material/densidade com inspeção de distribuição; hierarquia
entre madeira e folhagem antes de vento do conjunto. Não tratar esses itens
como features já entregues ou criar um tipo especial de árvore na cena.
EAF-87 acrescenta loft elíptico de estações com offsets XZ, domínio material
inversível e superfície fechada compartilhada. Escultura/pesos/rig/clipes/canal
de material seguem a fonte canônica; planos paralelos e interpolação linear
são explícitos, sem frame transportado ou solver de árvore inteira. Contrato
em [curved-plants.md](curved-plants.md); receita do mesmo tronco em
`scripts/tree_plant_recipe.py`. Integrar contornos com o primeiro fork exige
correspondência explícita; contar32v não cria um colar circular compatível.

Primeira integração EAF-80/EAF-87: um ramo da árvore é planta vinculada reautorável,
com fonte/canais e seis inserções preservados; primeira junção/base/líder
intactos. O teste exagerado revela quinas da interpolação linear. Priorizar
curvas suaves com domínio observável e vínculos por parâmetro/frame antes de
animar madeira com secundários/folhas; comparar preservação e close visual.
Vínculos e orientação continuam pendentes; suavização foi integrada depois,
como descrito abaixo. Relatório e limites desta primeira integração
em [tree-plant-integration.md](tree-plant-integration.md);64346t sem subir teto.

Continuidade EAF-87: PCHIP opt-in de canais de seção,57/61, suavidadeC1 emY
com dimensões positivas e sem overshoot escalar. Linear legado conservado;
resolução continua explícita. Prova controlada no mesmo ramo separa função
de tesselação; não conclui a casca, encaixes, frames livres ou árvore realista.

Suavização aplicada incrementalmente na cena salva, source/reference originais
exatos:12casos em2/4/6amostrasY com mesmasestações/câmera/luz/tick. Final PCHIP-A
[4,4,4],482v960t no ramo/64794t na árvore,742t livres;6 custaria65242t/294livres
e ficou diagnóstico. Wood inteiro e outros assets/instâncias preservados.
Dados e checklist em [tree-plant-integration.md](tree-plant-integration.md).

EAF-89 aplica hierarquia nativa à MESMA árvore: secundário com rametas extraído
por índices conferidos, sem geometria extra;9assets/31instâncias64794t. Copa
mantém clipe próprio e acompanha osso de inserção por duas relações explícitas.
Outros secundários ainda fundidos em wood, fonte/seleção/extração/distribuição,
materiais avançados e LOD seguem parciais/pendentes. Não converter esse ensaio
de um componente em alegação de árvore inteira vinculada ou realista.

### Continuidade por contornos reais — EAF-85/EAF-80

Disponíveis via MCP: inspeção de bordas, recorte explícito por faces e ponte
cúbica tangente entre ciclos iguais; preview/undo/rig/canais protegidos. A mesma
árvore suaviza a transição central. Pontes superiores foram reprovadas por
dobras internas; componentes anteriores preservados.
Não há ferramenta específica de árvore. Limites observados: seleção manual de
recortes/fases/alças, bordas retangulares ainda facetadas, ciclos de contagens
iguais, sem prova global de autointerseção. Próximos recortes: composição que
preserve fonte do ramo paramétrico oposto, encaixes secundários e distribuição
angular/erro geométrico da tesselação; domínio de casca transportado sem emendas.
Adicionar máscara/seleção explícita nos pincéis e diagnósticos geométricos
compactos no preview. Não aumentar orçamento ou perder autoria para acomodar o teste. UI continua
adiada. Contrato/evidências em mesh-bridges.md/validation.md.


Checkpoint trajetória da árvore: adoção explícita sem bake, fonte/referência e
poses0/60 preservadas; ramo[6,6,6] com erro lateral amostrado3.10mm<4mm.
Draw131072, outros budgets mantidos/consultáveis. EAF-80/85 continuam abertos:
a junção antiga permanece no exemplo e outros galhos estáticos ainda não são
plantas. Próximo é integrar kernel de3portas com fontes imutáveis e recorte030,
sem remover a região do socket nem promover candidata só por ser manifold.
Contrato/checklist: tree-trajectory-integration.md; proposta: plant-assemblies.md.

### União volumétrica limitada — base EAF-85

`preview_mesh_junction_union` acrescenta uma alternativa genérica às pontes:
três contornos reais de contagens diferentes, volumes tubulares guiados,
extração limitada e conexão com prefixo fonte exato. Prova nativa isolada no
recorte do tronco/ramo:3995v7986t, superfície fechada e rig/socket preservados.
Não foi promovida à árvore inteira nem resolve acabamento: fonte/costura1.15°,
conexão interna59.74°, colar ainda perceptível no neutro. Próximo incremento
finito: suavizar só pontos novos perto dessa conexão, primeira guarda e fonte
fixas; transportar domínio de material nas novas faixas. A receita regenerável
com contexto e planta imutáveis pertence à composição de plantas em paralelo.
Contrato, alternativas, custos e limites em implicit-junctions.md. Sem tipo de
árvore, UI ou aumento de orçamento nesta base.

Refinamento opt-in disponível depois da base: três anéis de pontos novos,
envelope finito e não piora angular, fonte/primeira guarda fixas. Na prova
isolada o máximo cai59.74°→43.47° sem geometria extra; ainda há colar. Correção
do domínio elimina a forte diferença fonte/guarda mas veios continuam com
distorção: preservar orientação/derivadas de material e diagnosticar stretching
é outro requisito genérico, não entregue por interpolação harmônica. Opções0
conservam chamadas anteriores; detalhes e gates em implicit-junctions.md.



### Composição corrente local66

Set_assembly e preview_scene_edits integram a primeira bifurcação à trajetória,
com duas fontes canônicas e vínculo vivo. Estudo7357v14710t,74870draw, rig/poses
preservados e ciclo de edição reproduzível. Apenas asset8192v16384t; totais e
layout/render mantidos. Native mesh_storage/plant_assemblies e regressão completa
passaram (validation.md). Restam outros galhos, camada de escultura ancorada no
miolo, distorção/gradientes do domínio, acabamento do colar e aceite humano;
isto não encerra EAF-80/85. Não usar saída avaliada como fonte para contornar.

### Continuidade68 — primeiro enxerto lateral persistido

O secundário0 estático agora se une ao contexto por3portas e fontes imutáveis,
com cortes/máscaras canônicas e mapas de compactação. Contrato em
[context-grafts.md](context-grafts.md), fonte/repro em demo_tree_graft.py.
A árvore completa não está conectada: transição em anel, casca, demais interseções
e pesos do secundário06 animado continuam próximos gates. Não transformar esta
lista de enxertos em grafo recursivo ou suporte multibone declarado implicitamente.

O acabamento69 permite suavizar o colar por região derivada de portas, mantendo
fontes, âncoras e interfaces protegidas. O primeiro anel foi atenuado nas vistas
fixas, sem aumentar triângulos. A próxima frente complementar é medir distorção
do domínio material na casca; demais encontros e rig de junções animadas seguem
abertos. Contrato em context-grafts.md e evidências em validation.md.

### Casa como novo modelo de teste — EAF-101/EAF-112

Direcionamento atual substitui a árvore pela casa da referência, preservando
seus artefatos e lacunas pendentes. Reutilizar a mesma casa ea8c6b8; revisar
[arquitetura observada](elven-cottage-review.md) antes de tratar detalhe como
fidelidade. Primeira melhoria genérica é separar grão fino/relevo de manchas de
cor em dois Pattern independentes (layered-patterns.md). Prova material-only em
cottage-materials.md, mesmos32172v/15assets/19instâncias/61312draw.

Próximos casos finitos: validar implantação/vazios/entrada e relação bay/empena/
coberturas em neutro; depois um módulo com seção de pedra e espessuras variáveis.
Separar erros de autoria de capacidade ausente com um caso mínimo. Aparência
futura ainda precisa de domínio orientado por peça, vidro/transparência, resposta
metálica e luz de lanterna; não considerar essas capacidades entregues por dois
ruídos. Nenhum CAD/gerador de casa ou aumento de teto aprovado implicitamente.

### Resposta metálica e fila de casos — EAF-114

O ensaio passou a perguntar qual capacidade ajudaria o resultado e provar essa
capacidade separadamente. Metalness nativa corrige a limitação do brilho branco
no bronze; ambiente/reflexos, parâmetros físicos por região, vidro e aliasing
continuam gaps. A forma da casa fica na frente de composição/Sweep. Fila atual,
com casos mínimos e distinção entre erro de autoria e capacidade ausente:
[authoring-gaps.md](authoring-gaps.md). Não declarar esses itens todos entregues.

### Ambiente procedural — EAF-116

Probe distante pré-filtrada e controle independente da luz direta passam a
permitir testar forma pelo reflexo em vez de depender só do realce do sol.
Recursos finitos/cache por conteúdo/counters, None preservado, MCP compartilhado,
Scene76/replay80 coordenados; sem importar a Wedge75/79 funcional implicitamente.
A casa mantém geometria/fontes; fontes novas da revisão estrutural devem ser
integradas com baseline anterior explícito para não invalidar os probes antigos.
Reflexos locais/AO, vidro, máscaras, HDR/specAA e revisão de normais/sidedness das
peças finas continuam casos distintos. Contrato em procedural-environment.md.

Inspeção de normais avaliadas(EAF-119): query MCP de pose em mundo/face/cantos,
mesmo algoritmo do renderer, pré-material e respostas limitadas. Ensaio local na
casa elimina um contato artificial entre reta/curva usando cadeia Sweep; topo
pontudo/miter continua aberto. A amostragem uniforme por cúbica gasta vértices
na reta: resolução por trecho foi tratada em EAF-120; erro de seção/superfície segue aberto.
[Diagnóstico e evidência](mesh-normal-inspection.md).


EAF-120: Sweep.spans escalar|lista por cúbica, planner independente e parâmetro
de seção preservado, formas uniformes iguais. Mola usa[1,9], economiza80v ante
variante uniforme, salto0no contato. Integração dos relevos/materiais/ambiente
em31948v, com friso antes encoberto corrigido por publicação MCP. Próximas
fricções: consultar apoio/folga em mesh visual, erro por seção/superfície e
encaixe angular. Contratos em sweep-curve-sampling.md/cottage-relief-integration.md.
