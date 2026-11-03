# Vegetação da paisagem representativa — EAF-184

Correção de aceite: terrain-walk valida física, mas não representa a paisagem
natural solicitada. systems possui o kernel/receita e terrain.scene.json;
nanites possui implantação/renderer e a cena derivada world.scene.json.
Base física acordada2×2km, grade de4m numa malha regional contínua. Manter fonte,
collision_meshes e personagem da base; não alterar kernel/formato/build
reservados para a geração. Cursos d'água começam visuais, com leito físico real.

Revisão prévia: implantação eventual em build time, nunca por frame/tick. Ler
malhas publicadas e preparar índice espacial de grades/triângulos de água uma
vez. Raízes usam interpolação dos triângulos da cena, não altura analítica antes
do bake. Bosques por seed, inclinação máxima e espaçamento configuráveis, água
e spawn protegidos. Reusar IGFT1/mesh compartilhado e variar porte/rotação/tinta;
nenhum caminho específico para árvore no renderer. Dados autorais na borda,
resumos binários nativos compilados antes do uso; sem formato runtime novo.

Verificar igualdade de fonte/luz/colisão/entidades e compatibilidade nativa,
contagens e enraizamento; inspecionar captura antes de abrir. Não escrever teste
por parâmetro visual nem alegar FPS/streaming/geologia/água simulada ou aceite
humano a partir da implantação. Os logs/capturas desta entrega são derivados
correntes em artifacts/representative-world; não criar cópias de executáveis.

Ajuste acordado com systems183: o trecho usa uma malha regional513×513 com
vértices/normais compartilhados (mesma grade2048m/4m), em vez de64assets com
normais unilaterais nas bordas. Ilha/costa abaixoY0 e oceano visual estendido
como acabamento do horizonte; água não física. O atual seletor de oclusores
não escolhe inteiro esse asset de524288tri por seu orçamento de trabalho262144;
medir esse custo e futuramente particionar a representação corretamente. Isso
não é teto autoral. Árvores desta unidade são visuais; colisão física do terreno
da base é preservada. A máscara de água deve considerar altura para não excluir
montanhas apenas porque o plano oceânico passa abaixo delas emXZ.

## Reprodução e apresentação

Depois de gerar a base183, executar `python3 scripts/build_representative_forest.py`.
São12mil árvores por padrão, com `--trees`, `--seed`, `--spacing` e
`--max-slope` configuráveis. O formato IGFT1 da árvore contém casca e copa
compartilhadas: duas instâncias por árvore, sem copiar a geometria por exemplar.
Porte varia12–26m; rotação e tinta variam, mas é um único modelo, não diversidade
de espécies. A implantação é uma receita de build, não um sistema novo de
vegetação no runtime. A apresentação é apenas uma região montanhosa, não o
gerador completo de biomas/topologias solicitado para o mundo futuro.

Systems autorizou ajustar a câmera da cena derivada sem alterar sua base.
A vista inicial `scenic` olha o vale e a cadeia a partir de[100,220,800],
alvo[0,130,-150], FOV60°. `--initial-view panorama` usa a vista geral da base;
`--initial-view ''` conserva a câmera original. Capturas `walk.png`,
`panorama.png` e `scenic.png` são nativas1728×1117; a vista de caminhada da
base é registrada mesmo quando a abertura usa outra câmera. No editor,
botão direito orbita, meio/Shift+direito desloca e roda aproxima.

`forest-verification.json` registra build/adapter, contagens por vista,
raiz na malha e fonte física. O build5c2e797b819983f2 carregou, compilou os
resumos e salvou/recarregou a cena. Erro máximo de enraizamento após roundtrip
nativo<0,00005m. A fonte permanece inalterada e `terrain_surface` é a colisão
real completa; água e árvores são visuais nesta entrega. Caminhada da base
validada por183 não substitui o aceite humano da cena povoada.

Compilar resumos não significa que eles serão escolhidos nesta escala: na
primeira captura com12mil árvores de12–26m numa região de2km, as árvores
ainda ocupam muitos pixels e o seletor mantém a geometria. Há244028290
triângulos lógicos por instanciamento na cena, não244milhões de triângulos
únicos residentes. Os contadores antes da oclusão não provam triângulos
efetivamente rasterizados nem FPS. Não prometer o custo da cena distante
anterior para esta floresta próxima; LOD geométrico automático181 e divisão
da representação do terreno continuam pendentes.

Captura final inspecionada e `world.scene.json` aberta e conferida no editor
existente, sem fechar a sessão do usuário. Região2048×2048m,4assets e24002
instâncias de malha. Fonte-base SHA256
8092e01449cd2f3a1518d7131ebb6d12aeb0c53c16154de0606ac65795e1a6dc;
399628bytes de resumos compilados. Densidade atmosférica calibrada para0,00006
com systems, preservada na derivada. Controles de câmera do editor visíveis.
Água ainda é um plano de cor; encostas/bosques ainda têm padrões regulares e
o horizonte do oceano é perceptível com ar mais limpo. Não equiparar esta
prévia ao realismo da referência179. Aceite humano de topografia, gameplay e
FPS da nova composição permanece aberto;184 continua InProgress.

Fronteira da janela: a GUI preservada éPID1262, iniciada22:33:04 em31/10 no
build182 b13af5530ec3621b. Carregou a cena104 pelo menu, sem reiniciar o app.
O binário corrente5c2e foi produzido23:02:25 e usado nos processos MCP de
compilação/captura/save-load; não é o build comprovado nessa janela. A GUI
não demonstra autoria Ridge2/API regional183. Passeio prolongado deve usar
modo continuous, ainda não confirmado na janela; ela foi deixada pausada.
