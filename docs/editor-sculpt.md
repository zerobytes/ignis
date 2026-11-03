# Modelagem e escultura no editor

## Método e resolução explícitos — EAF-110, 30/10

Direção de Guilherme: terminologia familiar e método/resolução configuráveis.
O painel passa a distinguir **Topologia fixa** de **Dyntopo — subdividir arestas**.
Este último é o subconjunto de refinamento por subdivisão; não implementa collapse,
Voxel Remesh ou Multiresolution. Remesh é uma operação de reconstrução, Multires
é um fluxo por níveis: não serão opções executáveis fictícias neste seletor.

Resolução dinâmica: **Detalhe do pincel** (aresta alvo em percentual do raio) ou
**Detalhe constante** (aresta alvo em metros locais do asset). Menor aresta pede
mais polígonos; tamanho do pincel e força continuam independentes. O estado do
método fica visível e os parâmetros aparecem juntos no painel de Topologia.
Refinamento disponível apenas para Volume; os outros oito pincéis preservam a
topologia. A preferência dinâmica é conservada ao trocar temporariamente de pincel.

Contrato definido antes do código: `SculptDetail` tipado compartilhado por
`SculptSettings`/`SculptStroke`, campo opcional com default Brush12.5% (=raio/8).
`adaptive_detail` legado conserva o significado; desligado preserva índices e
contagens. Preview e commit recebem o mesmo detalhe congelado por gesto.
Defaults mantêm comandos/layouts antigos; Scene/replay/layout3 não mudam porque
preferências são sessão e a edição grava a geometria resultante. Valores inválidos
recusados antes de publicar; undo único inclui detalhe+pintura.

Checklist prévio: (1) resolução calculada uma vez por chamada do kernel, sem novo
trabalho na simulação; (2) enum Copy sem heap novo, alocações de Plan existentes;
(3) unidades explícitas/finito e limites locais antes de usar; (4) kernel único
MCP/UI e defaults compartilhados; (5) quatro passes e budgets existentes mantidos;
(6) nenhum pipeline/upload/readback novo; (7) dispatcher sequencial, sem threads,
locks/dependências novas; (8) configuração congelada no Draft e invalidação pela
revisão existente; (9) estender provas críticas de compatibilidade, atomicidade,
persistência, preview=commit/undo, não testar texto por texto; (10) manter custo
síncrono de refinamento e limitação de resolução explícitos, sem alegar FPS ou
equivalência ao Dyntopo completo. Alternativa rejeitada: só renomear o checkbox
sem deixar escolher a resolução; remesh/multires são incrementos estruturais
separados, com contratos de preservação e custo próprios.

Revisão posterior: campos/defaults compartilhados em mesh_tools/editor/meshes;
`Draft.detail` congela os parâmetros, `interactive` valida e calcula a aresta
antes dos passes, os mesmos parâmetros chegam ao commit. Nenhum contrato de
rig/Frame/avatar foi alterado; só dois hunks de SculptStroke no arquivo meshes
compartilhado. Preferências inválidas e comandos antigos cobertos pelas provas
críticas ampliadas. Custos de Plan/BTreeSet/scan síncrono mantidos explicitamente;
trocar detalhe não altera a geometria até iniciar um novo gesto.

Validação: 28Rust existentes/ampliados em editor(8), editor_mesh(4),
sculpt_brushes(7), sculpt_detail(4), sculpt_usability(5). Clippy release/locked
all-targets -Dwarnings aprovado; build corrente ignis+probe, MCP editor_mesh7 e
sculpt_brushes9, sem janela/raiz própria. Build final `e381846313b219c3` e manifest
em artifacts/sculpt-config. Graphics real operou checkbox Dyntopo, seletor
Brush→Constant e scrub horizontal0.03013→0.06013m; gesto em seguida conservou a
configuração, preview=commit e um undo exato. Relatório/capturas em
artifacts/editor-sculpt-quality/detail-configuration-report.json,
topology-fixed-ui.png e dyntopo-constant-ui.png. Regressões X nas duas ordens e
pincéis existentes também passaram no probe. A precisão visual de5casas foi
validada por Clippy/build/ponteiro depois dos28Rust; não exigiu novo teste unitário.
Gate offscreen não representa aceite humano nem foco/entrada física do desktop.

MCP: enviar em SculptSettings ou Edit::SculptStroke o campo opcional
`"detail":{"mode":"brush","percent":12.5}` ou
`"detail":{"mode":"constant","edge_length":0.05}`. `adaptive_detail:false`
preserva a topologia; sem `detail`, mantém raio/8. Percentual aceito3.125..100,
constante0.00001..100m locais; os mesmos quatro passes e budgets continuam valendo.

O refinamento local conformante ocorre antes da deformação, com os mesmos
limites globais e um desfazer por traço. Ver a reprodução de diagonais opostas.

Atualização29/10: nove pincéis reais e suas preferências/contratos estão em
[sculpt-brushes.md](sculpt-brushes.md); manipulação direta em
[editor-gizmos.md](editor-gizmos.md). Os registros abaixo preservam a evolução
dos três kernels iniciais e as correções de traçado.

Contrato anterior: primitivas editáveis por comando compartilhado; seleção de
instância é sessão, independente de seleção de atores e persistência de layout.
Picking real de triângulos em pose renderizada, câmera do editor/viewport física.
Prévia do traço pertence à UI e usa o mesmo kernel Sculpt; só mouse-up publica
um EditMesh (undo único). Alternativa de publicar cada amostra descartada para
não inundar histórico/reset; Esc/foco perdido/revisão nova descartam a prévia.
Draw/Smooth amostram por distância; Grab usa deslocamento em plano da câmera,
centro inicial congelado. Shift suaviza, Ctrl inverte volume. Raio/intensidade
não alteram o mundo; geometrias inválidas preservam último estado válido.
Primitivas têm resolução limitada, índices compartilhados e normais orientadas.
Escultura inicialmente na base estática sem rig/playback/attachments/fontes
paramétricas ou conexões consumidas: recusar claramente, sem bake/detach implícito.
Transformações/material/subdivisão usam contratos existentes e limites originais.
Sem remesh, dyntopo, gizmo de osso ou cronograma de Blender completo.

## Checklist anterior
1.Preparação só em criação/gesto; raycast apenas com cursor/evento relevante.
2.Payloads Arc, uma cópia da geometria tocada por gesto; amostras até128;
sem Engine/Simulation novo por frame. 3.Path/IDs separados, posição local e
viewport física explícitas;4.reutilizar Sculpt/SceneEdit/renderer/undo;
5.limites de assets/vértices/triângulos e de primitiva antes de alocar;
6.sem readback rotineiro/shader/GPU feature nova, overlay egui;
7.sem dependência/thread/lock novo;8.revisão e seleção invalidam gestos,
prévia não entra em Scene/Replay;9.testes de primitivas/picking/gesto/UI,
undo/save/load/replay/erros, suíte completa e nativo isolado;
10.custo síncrono limitado de raycast/normal por amostra/upload mantido,
sem promessa de FPS, física por mesh ou aceite humano por headless.


## Uso

Adicionar → Malha editável → Esfera/Cubo/Cilindro/Plano; selecionar na superfície
ou na hierarquia, Enquadrar/F e Escultura. Volume/1, Mover/2, Suavizar/3;
Ctrl cava com Volume, Shift inicia suavização. Raio local e intensidade na barra;
Ctrl+roda muda o raio. Clique/arraste atua na superfície; soltar confirma,
Esc/foco perdido cancela; ⌘Z/⌘ShiftZ desfazem/refazem e ⌘S salva.

## Revisão posterior

1. Raycast cacheado por cena/editor/tick/cursor/geometria. Poses são calculadas
uma vez por consulta; scans de instâncias/triângulos continuam limitados pelos
budgets existentes. A caixa de seleção em modo Objeto ainda percorre vértices.
2. Draft clona metadados/Arcs; geometria tocada usa copy-on-write. Grab recalcula
sobre a base congelada; Draw/Smooth sobre a prévia com kernel comum. Normais,
validação e upload da malha editada continuam síncronos, sem alegação de FPS.
3. Raio/amostras são locais; cursor é projetado pela câmera e escala física da
viewport. IDs de asset/instância permanecem separados. Seleção é sessão.
4. Primitive, Sculpt, SceneEdit, materiais, subdivisão, renderer, replay e undo
são reutilizados. Uma edição apenas ao soltar; nenhum reset de simulação por
amostra. Press/release no mesmo frame também deve gerar edição válida.
5. Resolução validada antes de alocar; até128amostras/gesto, até16interpoladas por
evento. Novas primitivas não aumentam tetos globais. Erro cancela só a prévia.
6. Sem shader/pipeline/readback/dependência/thread novos. Cursor é overlay egui;
RenderFrame recebe o documento da prévia sem duplicar a construção comum.
7. Comandos de uma confirmação UI são serializados sob o mesmo lock; primeira
revisão é a apresentada, seguintes acompanham sucessos do mesmo lote. Falha
interrompe o lote. Soltar e ⌘S no mesmo frame confirma primeiro o traço e só
então salva, com regressão de arquivo real. Outros comandos durante um gesto
incompleto continuam cancelando a prévia. Desenho GPU permanece fora do lock.
8. Esc/foco/cena/câmera/seleção/revisão invalidam gesto. Prévia não é Scene nem
Replay. Save/load usa formato existente; atalho ⌘S encaminha SaveScene ou pede
um destino quando a cena ainda não tem caminho.
9. Testes cobrem fechamento/orientação das primitivas, picking transformado,
preview/commit, gesto rápido, idle sem carimbos extras, cancelamento, save/load,
undo/redo, erros atômicos, render e replay. Resultados finais abaixo.
10. Remesh/dyntopo, pincéis adicionais, simetria, máscaras, pressão de tablet,
retopologia e ferramentas visuais de rig/animação permanecem próximos recortes.
Não há bake/detach, colisão de malha, nova capacidade de runtime ou aceite humano
implícitos. Build corrente único da worktree; janela de teste independente.


## Evidências finais

Build5fcf3f9b1b05dc9c:1104testes Rust/105alvos release/all-targets, Clippy
-Dwarnings/fmt/diff/sintaxePython e96cenários MCP/395grupos completos aprovados,
sem falhas/ignorados. Cinco testes de entrada UI e três testes de primitivas/
picking/persistência novos. Mesmo fonte/hash conferido; formatos mantidos.
Conferência nativa dos três pincéis no build final alterou28/34/45vértices,
salvou e comprovou undo/redo por igualdade dos documentos. Criação pelo menu
foi conferida na passagem anterior; seu código permaneceu igual. Seleção de
superfície/hierarquia também conferida. Screenshot e auditoria editor-sculpt-*
em artifacts; resultados detalhados em validation.md.

Build único17.554.944B/cache exclusivo~2,0GiB; app instalado usa o mesmo
executável. Wrapper de teste removido após encerrar a janela própria; sessão
anterior do usuário preservada. Sem push/main. Aceite humano permanece aberto.


## Revisão de usabilidade — antes da implementação

Avaliação humana: deformações pouco legíveis; priorizar qualidade deste recorte,
sem iniciar simetria/máscaras/remesh. O backend tem somente Draw/Grab/Smooth;
Stroke já admite radius/strength/direction/centers. Expor a força efetiva em
metros por amostra para Volume, proporção do arraste para Mover e fator de
suavização para Suavizar, com parâmetros por pincel. Eliminar fator0.08 oculto.
Raio/intensidade e escolha de pincel passam a configuração tipada via EditEditor.

Iluminação de trabalho global no layout3, independente da luz autoral, com
padrão direcional lateral, ambiente0.08 e sombras1024. Controles de direção,
ambiente, sombras/alcance/resolução/bias e opção Luz da cena reutilizam Light e
renderer. Alternativa de alterar luz de todas as cenas descartada para preservar
a autoria e replay; nenhum shader novo. Layout1/2 carrega valores padrão e migra
para3; Scene75/Replay79/manifesto1 permanecem. Configuração valida cópia antes de
aplicar, sem histórico autoral ou mudança da simulação.

Checklist anterior:1.eventos de widgets publicam configuração, nada no tick;
2.configurações pequenas Copy/clone, Light/Frame reutilizados;3.raio/forças com
unidades explícitas, limites do kernel;4.validadores/dispatcher/renderer comuns;
5.resolução512/1024/2048 e mesmos budgets;6.mapa só recria ao mudar resolução,
sem captura/readback por frame;7.sem dependência/thread/lock novo;
8.revisão de editor invalida gesto e captura;9.testes de migração, MCP/arquivo,
parâmetros/atomicidade/pixels/sombras/undo, Rust/Clippy e regressão completa;
10.sombra adiciona passeGPU já existente por escolha explícita, sem prometer FPS.

## Revisão posterior da correção de usabilidade

1. Controles publicam EditEditor por evento; não entram no tick. Picking e
normais mantêm os mesmos caches e limites. 2. SculptSettings/Light são pequenos;
nenhuma cópia extra de geometria por frame. 3. Volume usa diretamente metros
por amostra; Mover aplica proporção ao arraste local com teto1m do kernel;
Suavizar usa blend. Raio permanece local, com escala visível no anel.
4. Light::validate extraído do validador da cena e reutilizado no editor, sem
relaxar guardas históricas. Scene/Replay/undo permanecem separados.
5. Mesmos limites de Stroke, resolução de sombra e áreas; migração de layout1/2
validada antes de substituir. 6. Work light altera apenas o RenderFrame da
viewport; shader, pipelines e mapa de sombras existentes são reutilizados.
7. Sem dependência/thread/lock novos. 8. Mudança de configurações invalida
rascunho por revisão do editor; gravação em layout explícita, não na cena.
9. Regressões novas medem deslocamento real por gesto, footprint por raio,
persistência/atomicidade e pixels da superfície sob luz/sombra distintas.
10. Custo conscientemente mantido: padrão com um passe de sombras existente,
mapa1024 e validação síncrona limitada. Nenhum ganho de FPS ou aceite humano
é inferido das verificações. Não foram acrescentados novos pincéis neste ajuste.

### Comandos compartilhados de configuração

`inspect_editor` retorna `editor.sculpt` e `editor.work_light`. Configuração
nativa e MCP usa `edit_editor`, com revisão opcional de editor. Exemplos:

```json
{"op":"edit_editor","action":{"action":"sculpt","settings":{"brush":"draw","radius":0.65,"draw_strength":0.12,"grab_strength":0.7,"smooth_strength":0.5}}}
```

```json
{"op":"edit_editor","action":{"action":"lighting","light":{"direction":[0.7,-1,-0.4],"ambient":0.08,"shadow":{"center":[2,1,0],"extent":8,"resolution":1024,"bias":0.002}}}}
```

`light:null` restaura iluminação da cena. `edit_mesh` com Sculpt continua
recebendo Stroke explícito, independente das preferências do editor: o cliente
MCP escolhe suas amostras e força. `save_editor`/`load_editor` persistem o layout3;
`save_scene`/replay não incluem luz de trabalho nem preferências de pincel.

A inspeção GPU encontrou acne de sombra com bias0.0004; padrão corrigido para
0.002 após comparar capturas com0.0004/0.001/0.002/0.003. Valores avançados
continuam configuráveis. O teste nativo revelou também fechamento prematuro do
menu ao clicar em controles e possível traço por clique atravessado. Menu de
iluminação agora fecha apenas fora; a UI conserva a presença de popup no início
do passe e bloqueia escultura/seleção/câmera no clique de fechamento. Regressão
de entrada cobre popup aberto sobre a superfície, incluindo press/release rápido.


### Evidências da correção

Build9c80622307cd2659:1108Rust/105alvos release/all-targets e Clippy-Dwarnings
aprovados;96cenários MCP/397grupos completos, sem janela, erros ou ignorados.
Quatro regressões Rust novas e dois grupos MCP de configuração/pixels. Formatos
Scene75/Replay79/manifesto1 mantidos, layout3 migra1/2. Conferência nativa final:
campos raio0.6/força0.12,87vértices alterados por um gesto, undo/redo com documentos
exatos; luz/sombra alternáveis e fechamento de popup sem atravessar geometria.
Artefatos editor-light-* e detalhes em validation.md. Usuário retomou a janela;
nenhuma sessão anterior foi encerrada. Um release corrente17.655.504B, wrappers
por symlink em uso preservados. Aceite humano continua aberto.

## Correção do traçado — decisão antes do código

Relato humano: arrastar Volume sobre esfera padrão cria espinhos caóticos.
Reprodução isolada, resolução24/1106v, raio0.4, força0.08m/amostra, arraste
160px: referência móvel desloca até1.11m e estica arestas8.65×; congelar apenas
picking reduz para0.345m/2.15×. A ferramenta estava perseguindo o próprio relevo;
redistribuir carimbos por evento também mudava a densidade conforme os eventos.

Escolha: picking do documento autoral congelado durante todo gesto; continuidade
por comprimento percorrido, com resto transportado entre eventos e passo0.15raio.
Primeiro carimbo na posição real do press. Sem interpolar através de região sem
superfície selecionada. Limite128 carimbos continua explícito; remover teto16 por
evento que criava lacunas em movimentos rápidos. Preview incremental continua
usando o kernel público e o commit publica a sequência inteira idêntica.

Alternativas: baixar força esconderia feedback sem corrigir sua causa; remesh
implícito mudaria topologia/autoria; alterar Draw globalmente ou limitar altura
mudaria o contrato MCP de metros por amostra. Não adotados neste incremento.
Acúmulo intencional e normais por amostra permanecem; não promete pincel de camada,
dyntopo ou equivalência completa ao Blender. Avaliar qualidade por traço contínuo,
sem runaway, invariância de eventos e orientação das faces, além de preview/undo.

Checklist anterior:1.picking por cursor/evento, não tick;2.reutilizar cache do
original e geometria COW da prévia, apenas dois vetores/um resto por gesto;
3.distância local f32, força/raio e formatos intactos;4.cada segmento consumido
uma vez, resto preservado;5.até128 aplicações/gesto, sem novo limite global;
6.sem shader/readback/I/O no gesto;7.sem thread/lock/dependência;
8.revisão/câmera/seleção/foco/Esc continuam invalidando;9.entrada real egui com
esfera padrão, eventos densos/esparsos/coalescidos e lacunas, preview=commit,
undo/redo/cancel e regressão MCP/render/replay;10.raycast/validação/normais/upload
síncronos e normais recalculadas por carimbo mantidos, sem alegação de FPS.

### Revisão do diff do traçado

1. Corrigido: referência do picking fica no documento autoral; geometria da
prévia não retroalimenta hit_cache. Trabalho em eventos de ponteiro, sem tick.
2. Adequado: StrokePath guarda previous/remainder, duas coleções temporárias
por lote de eventos e a geometria COW existente. Clone limitado do Stroke no
preview incremental permanece neste recorte (até128centros).
3. Adequado: distâncias locais f32 e eventos Pos2; metros por amostra e fatores
Smooth/Grab preservados, sem campo/enum público novo.
4. Corrigido: comprimento transporta o resto entre segmentos; eventos entregues
na mesma passagem são conservados, inclusive caminho fechado que termina na
posição anterior. Duplicatas não produzem aplicações adicionais.
5. Mantido com motivo:128 aplicações/gesto; interpolar mais de16 num evento é
permitido até esse teto para não truncar um arraste rápido. Picking percorre os
eventos entregues e o kernel continua O(amostras×(V+T)), sem alegação de FPS.
6. Adequado: mesma prévia/upload/renderização, sem I/O/readback por movimento;
capturas ocorrem somente no probe explicitamente executado.
7. Adequado: nenhum lock/dependência/thread novo; confirmação única preservada.
8. Corrigido: amostragem reinicia após falha de superfície/saída da viewport;
revisão/foco/Esc/seleção/câmera continuam descartando o rascunho inteiro.
9. Regressões de entrada reais do egui: esfera padrão com eventos1/8px, caminho
curvo coalescido, faltas de superfície e teto de amostras; faces orientadas,
relevo limitado e preview=commit/undo/redo. Probe GPU independente conserva o
comando serializado exato produzido pela UI e captura antes/depois.
10. Mantido: normais recalculadas sobre predecessor por aplicação, acúmulo
intencional, resolução/topologia fixa e custo síncrono. A mudança corrige
posicionamento/densidade do traço; novos pincéis/remesh e redesenho da UI ainda
são incrementos distintos. Evidências finais em validation.md.

## 29/10 — traços inversos e normais congeladas, decisão anterior

Novo feedback humano mostra espinhos dependentes do sentido mesmo com picking
congelado. O kernel Draw ainda recalcula normais e alcance a cada amostra: a
amostra seguinte inclina a deformação anterior e pode dobrar triângulos. Testar
mesmo percurso nos dois sentidos em plano e esfera, inclusive diagonais e
cadências diferentes. Escolha: Draw lê posições/normais da base do gesto e
acumula relevo, sem perseguir a própria deformação. Preview usa o gesto inteiro
sobre a mesma base; commit/MCP usam exatamente o kernel público. Smooth/Grab
conservam seus comportamentos. Formatos, força por amostra e topologia mantidos.

Checklist:1.apenas gesto/comando;2.normais uma vez por gesto avaliado, somas
limitadas128×V, Arcs/COW existentes;3.índices e posições locais separados do cursor;
4.preview/commit compartilham kernel, sem algoritmo só visual;5.tetos existentes;
6.sem shader/readback/IO novo;7.sem lock/worker/dependência;8.base por revisão,
Esc/foco/undo invalidam;9.ordem/cadência/face/erro/histórico e MCP/render/replay;
10.preview completo O(T+V×amostras) é custo limitado explícito, sem claim FPS.

A superfície congelada mantém mais influência que o alcance que encolhia na
prévia antiga. O padrão de novos layouts passa de0.08 para0.04m/amostra; valores
já salvos e a unidade pública de Stroke permanecem intactos. A regressão do
compositor mantém o limite anterior de relevo0.1..0.5m com o novo padrão, em vez
de ampliar esse limite para aceitar um preset excessivo. Draw explícito0.08 é
exercitado nos testes de direção positivos/negativos.

## Reincidência29/10 — direção do pincel e proteção de deformação (EAF-110)

Novo relato: espinhos já no primeiro traço de uma malha nova. Inspeção somente
leitura da janela mostrou também Mover ativo, raio0.241m/70%, Cube24. O pincel
usado no print está sendo confirmado; o reparo cobre explicitamente ambos os
caminhos, sem assumir que seja apenas o caso antigo de amostras incrementais.

Decisão antes do código: Draw usa uma direção média da área por carimbo (base
congelada), não direções individuais por vértice equivalentes a Inflate. Mantém
metros/amostra, raio, pesos e campos públicos. Mover continua sendo Grab da
região inicial; não vira Draw nem passa a carimbar um caminho. Deformações que
dobrem/colapsem faces devem ser recusadas antes de alterar posições/histórico.
No arraste Grab recusado, manter última prévia válida, permitir recuar o cursor
e confirmar somente um resultado válido; soltar ainda inválido cancela tudo.
A UI descreve essa distinção e não anuncia espaçamento contínuo para Grab.

Alternativas descartadas: baixar força silenciosamente ou limitar por fator
oculto; remesh/reparo automático; converter Grab em outro pincel; mexer na cena
pessoal. Malhas antigas já deformadas não são reescritas pelo reparo.

Checklist prévio:1.só autoria/gesto, sem tick;2.normais V, direções128 e vetor
candidato V limitados, sem matrizV×128;3.tipos/campos/formatos atuais;4.direções
por carimbo calculadas uma vez, preview/commit usam o mesmo kernel;5.tetos
existentes e recusa explícita, sem poda/topologia nova;6.mesma GPU/I/O por gesto;
7.sem workers/locks/deps;8.validade da prévia explícita, erro/recuo/foco/undo;
9.apenas regressões críticas de corrupção geométrica/commit inválido e probes
existentes pertinentes, sem suíte ampla automática;10.custo síncrono
O(T+V×amostras), cópia de posições e guardas T aceitos, sem claim de FPS.

Referência de semântica: [Draw](https://docs.blender.org/manual/en/4.0/sculpt_paint/sculpting/tools/draw.html)
e [Inflate](https://docs.blender.org/manual/en/5.0/sculpt_paint/sculpting/brushes/inflate.html).
Não promete equivalência completa de kernels/força/pressão com Blender.

### Revisão do reparo

1. Frequência: apenas mudança de gesto/centros, sem trabalho no tick. 2. Draw usa
normais V, direções128 no stack e posições candidatas V; Grab copia posições V
para rollback. 3. Mesmos Stroke/BrushKind e unidades, somas/direções médias f64,
saída f32. 4. Direção da área calculada uma vez por dab, reaproveitada no passe
por vértices; preview/commit continuam o mesmo kernel. 5. Sem crescer topologia
ou elevar tetos; dobramento/colapso relativo à face original é recusado, não
atenuado por força oculta. 6. Sem shader/readback/I/O no gesto. 7. Sem worker,
lock ou dependência novos. 8. Grab retém prévia válida mas marca candidato
inválido; soltar inválido não envia comando, recuar recupera e produz undo único;
revisões/foco/cancelamentos anteriores preservados. 9. Teste crítico falhou antes
do reparo: um único Grab no cubo novo, raio0.241/deslocamento0.35 em ambos os
sentidos, era aceito com faces invertidas. Regressões focadas cobrem recusa/
atomicidade/recuperação, normal de área de Draw e probes de ponteiro existentes.
10. Draw mantém custo limitado O(T+V×amostras), com dois passes de pesos; guarda
T e cópia V são custos conscientes de integridade na autoria. Sem medição ou
alegação de ganho de FPS, remesh/auto-reparo ou aceite humano implícitos.

A guarda é conservadora em relação à orientação anterior de cada face; não é
um detector global de auto-interseções. Puxar uma dobra através de outra parte
da superfície pode requerer outro fluxo de modelagem. A recusa não conserta
malhas que já foram dobradas por versões anteriores.

## Diagonais opostas — refinamento local antes de Volume (EAF-110)

Reprodução exata após novo ângulo humano: Cube24/3458v/6912t, raio0.241,
força0.04, diagonais / versus \\, com sombra desligada. Uma acompanha as arestas
da grade; a outra cruza a triangulação e produz dentes. Testes anteriores
invertiam o sentido na MESMA diagonal e não cobriam essa anisotropia. Subdividir
ANTES de deformar reduz a serrilha; subdividir o relevo já serrilhado apenas
interpola os dentes existentes. Capturas artifacts/sculpt-diagonals/.

Decisão anterior ao código: novo Edit::SculptDetailed com referência scope/revision,
Stroke e max_edge explícito. Usa Plan de subdivisão conformante existente para
refinar somente triângulos próximos do traço; depois aplica Draw no resultado.
Até4passes, edge<=raio/8 por padrão nativo, tetos atuais checados em cada plano;
se detalhe não couber, recusa sem pintura parcial. Rig/clipes/canais passam pelo
transferidor existente, sem poda; plantas/conexões exigem detach/remoção explícita.
Scene/replay não recebem novo formato: o resultado é a geometria nativa existente.

Alternativas: refinar globalmente consome quase todo orçamento do cubo; girar
arestas só esconderia o caso alinhado à grade; remesh geral trocaria topologia/
preservação/contratos. Escolhido refinamento local por reutilizar EAF-79 e evitar
novas dependências. Preferência visível/persistida "Detalhar sob o pincel", default
ativo para Volume; pode desligar para conservar topologia. MCP recebe a mesma
operação e prévia; confirmar produz uma transação e um undo para detalhe+pintura.
Prévias partem sempre da base, sem crescer a malha por redraw ou perseguir relevo.

Checklist prévio:1.autoria só com valor/centro novo;2.COW e Plan limitado, sem
matrizV×amostras, sem Simulation por preview;3.refs/índices/sizes tipados, f32 na
geometria;4.planos/detalhe completos a cada preview para igualdade commit, sem
acúmulo em idle;5.4passes/128centros/16384v/32768t e budgets globais, nunca parcial
silencioso;6.mesmo renderer/upload, sem readback no gesto;7.sem worker/lock/deps;
8.scope/revision, undo/foco/erro e fontes preservados;9.regressões críticas de
qualidade / versus \\, rig/canais/budget/stale/atomicidade/undo e probe existente;
10.custo O(passes×T×centros), com rejeição AABB antes da distância triângulo,
mais Plan/Draw/validação síncronos; sem claim FPS ou remesh completo.


### Revisão posterior e evidência do refinamento

Build `bab348c00e11cd59`, sobre main `5cf41aa`, sem Scene87/replay92/layout3/
manifesto1/deps novos.4regressões críticas novas protegem perda de qualidade
por orientação da grade, atomicidade/orçamento/escopo, canais de rig/material/
clipes e PutAsset inválido seguido de escultura no mesmo lote (recusa sem panic).
14testes kernel/contrato e17UI existentes passaram; Clippy focado-Dwarnings,
fmt/diff/Python e4MCP/28grupos (editor_mesh/sculpt_brushes/editor/mesh_topology)
aprovados. Preview/commit, um undo/redo, salvar/reabrir e replay exatos.

Régua de qualidade: crista amostrada na superfície triangular real, sem usar
normais ou pixels de sombra. Cube24/raio0.241, forças0.04 e0.12: variação relativa
(máximo−mínimo)/média cai de15.4097% para0.9677% na diagonal adversa; outra diagonal
0.0720%.6637v/13270t e6803v/13602t, dentro dos tetos. Face oposta mantém os
vértices originais. Capturas comparáveis com sombra desligada mostram o caso
antigo e o relevo contínuo, sem ocultar erro por iluminação. Probe Graphics com
ponteiro real em ambas as diagonais confirma refinamento, comando compartilhado,
prévia=commit e um undo:5681v/11358t e5589v/11174t. PNGs com/sem sombra revisados;
execução offscreen, sem entrada/reinício na sessão humana.

Checklist posterior:1.preview só em centro novo;2.COW/base imutável e planos
limitados, buffers temporários por passagem, sem clone de Simulation;
3.triângulos/canais validados antes de indexar, inclusive candidato intermediário
de lote;4.preview recalcula da base para coincidir com commit, sem acumular
subdivisão em redraw;5.preflight de contagens antes de alocar posições/pesos,
4passes/128centros/tetos existentes, recusa integral;6.mesmo renderer/upload,
nenhum readback no gesto;7.mesmo dispatcher/undo, sem lock/thread/dependência;
8.referências de sessão e guards de fontes/conexões, cancelamento/revisão/foco
preservados;9.gates críticos acima e relatório em artifacts/sculpt-diagonals/;
10.O(passes×T×centros), AABB antes de distância exata, Plan/Draw/validação e
clone de metadados de Scene por preview são custos síncronos conscientemente
mantidos. Sem medição/alegação de FPS ou remesh geral.

Detalhar consome orçamento e pode exigir traços menores/raio maior se recusado.
Não apaga dentes já gravados: subdivisão antes da pintura melhora sua amostragem;
subdivisão depois conserva a deformação antiga. Default pode ser desligado
explicitamente no painel; Sculpt continua com topologia fixa. A janela humana
aberta precisa ser salva/reaberta para usar novo código; nenhum aceite humano é
inferido desses gates. EAF-110 permanece In Progress.


## EAF-110 — força contínua e saturação sem descartar gesto

Diagnóstico30/10: fonte16384v/32768t, print16046v/32088t. SculptDetailed exige toda densidade raio/8 em até4passes; qualquer falta de orçamento descarta o gesto. Draw legado soma metros/amostra até128vezes. Grab estrito protege orientação, mas a UI mostra recusa binária em arraste comum.

Decisão anterior ao código: manter Sculpt e SculptDetailed estritos para contratos existentes. Novo Edit::SculptStroke {asset,reference,stroke,adaptive_detail}, Draw/Grab. Volume usa força normalizada0..1, altura máxima raio×força×0.5 por gesto e envelope máximo dos carimbos na base congelada (sem acumulação pelo comprimento/cadência). Ctrl negativo mantém semântica. Preferências incluem unidade relativa; layouts legados migram m/amostra→percentual preservando altura de um carimbo, sem alterar cenas. UI mostra percentual para Volume e os outros pincéis.

Detalhe automático tenta até4passes conformantes com preflight de contagens antes de Plan/asset. Se orçamento ou4passes terminarem, retém o detalhe alcançado e aplica Volume na resolução disponível; feedback discreto de limite, sem erro nem aumentar tetos. Rig/material/clipes transferidos pelo Plan existente, sem poda; fontes/conexões protegidas como SculptDetailed. Nenhum remesh ou ganho de capacidade alegado. Topologia fixa quando adaptive_detailfalse.

Guard de superfície permanece. Nova operação limita deslocamento linear ao primeiro ponto seguro ao longo do gesto, usando raízes do polinômio de área projetada de cada face; mantém margem de5% da orientação original. Mesmo cálculo determinístico em preview/MCP/commit. Grab usa um único centro congelado. API estrita antiga continua recusando faces dobradas; novos limites são explícitos na operação interativa. Isto não detecta auto-interseções globais.

Se o pincel não alcançar3vértices na superfície disponível, amplia somente o footprint até suportar3vértices (círculo/raio efetivo e aviso), mantendo a altura limitada pelo raio solicitado. Sem snap a um único vértice ou força artificial maior.

Feedback tipado do kernel (detalhe limitado/fração aplicada) é somente sessão na UI; não entra em Asset/Scene/Replay. Kernel compartilhado retorna o mesmo Asset ao dispatcher. Preview usa o mesmo kernel+preflight e guards de elegibilidade nativos, commit mantém guard scope/revision e Scene.validated. Um desfazer por gesto, cancelar não muta.

Checklist prévio:1.só gesto/amostra nova;2.bufferslimitados por V/T/centros, Arcs nos metadados;3.força/raio/ref tipados sem JSON no kernel;4.base congelada e mesmo kernel para prévia/commit;5.4passes/128centros/tetos inalterados;6.renderer/upload existentes, sem readback/I/O no gesto;7.sem thread/lock/deps;8.scope/revisão/foco cancelam e fontes/rig/canais preservados;9.gates críticos budget saturado+pequeno raio+força monotônica+limite seguro/undo/persistência e probes de UI existentes;10.O(T×centros×passes)+normal/validação/áreaquadrática porface, sem alegar FPS. Alternativas: levantar limites adia esgotamento; remover guard causa corrupção; só esconder mensagens mantém zeroefeito; remesh geral exige outro contrato de preservação.


### Revisão posterior — gesto interativo sem cancelamento por saturação

Build congelado `f18dd4ad81236106`, base EAF-141/d2e37ee, Scene88/replay97/
layout3 preservados por esta unidade. Native usa SculptStroke; APIs estritas
continuam recusando seus casos antigos. Força percentual mede altura por gesto,
não metros por amostra. Default60%; layouts antigos têm marcador de unidade
serde defaultfalse e migram uma vez preservando altura de um dab (limitada ao
novo intervalo). Sobrepor128centros no mesmo ponto gera a mesma altura que1.
Repassar em novo gesto permite acumular camadas explicitamente.

Detalhe automático conserva passagens completas que couberam até4; preflight
conta incidências geradas antes de Plan/posições/pesos. Falta de orçamento é
feedback, não falha do gesto. Se nenhum suporte de3vértices couber no círculo,
footprint aumenta até resolução disponível; círculo/aviso mostram raio efetivo,
a altura permanece limitada pelo raio solicitado. Não é remesh nem ampliação
de capacidade. Rig/clipes/material/procedência seguem Plan/guards existentes.
Limite de movimento usa primeira raiz positiva da área projetada de cada face,
incluindo dobra que voltaria a ter orientação válida no fim; margem5% e recuo
numérico conservador de2%. Até8reduções extras por arredondamento f32; não publica
geometria inválida. Não detecta auto-interseções globais.

Validação por risco:18testes kernel/contrato (4novos críticos),17UI existentes,
Clippy focado-Dwarnings,fmt/diff/Python e4MCP/29grupos passaram. Asserções antigas
de integridade/qualidade não foram afrouxadas. Probe reutilizado: renova referência
após undo antes de reaplicar parâmetros serializados; mesmo preview/commit/undo.
Lib/binário congelados têm121inputs no manifesto. Por coordenação, backendQUIC
foi escrito depois em fontes compartilhados; só o probe foi recompilado contra
esta lib usando rustc com externs exatos do fingerprint Cargo, sem rebuild de
core incompleto. Comando em artifacts/sculpt-usability/probe-compile.json.

Probe Graphics/GPU sem janela:16129v/31752t, raio0.08, forças10/30/60%→alturas
0.00399995/0.01199996/0.02400005m, sem perder gesto nem exceder teto, um undo por
traço. Esfera95v alterados/máximo0.119918585m, nove pincéis com paridade, diagonais
opostas e Grab limitado/recuperado passaram. PNGs saturation-10/30/60,grab-limited
e detail-diagonal-* revisados. MCP isolado também clonou read-only o asset salvo
humano16046v/32088t:9casos(raios0.03/0.1/0.241×forças10/30/60%) todos aplicaram,
preview puro/commit igual/undo exato e tetos mantidos. Arquivo humano não foi salvo
ou alterado. Logs/manifesto/validation/provas em artifacts/sculpt-usability/.

Checklist posterior:1.só novos centros/arraste;2.Arc nos canais/metadados,
buffersV de normais/candidato/original, sem Scene/Simulation por prévia;
3.raiz/normalização em f64 e saída f32, pressão/unidade/ref tipadas;
4.envelope máximo na base evita dependência de sobreposição/cadência/comprimento;
5.4passes/128centros/tetos originais, preflight e guarda f32 finitos;
6.renderer/upload existentes, sem readback/I/O no gesto;7.mesmo dispatcher e
transação, sem nova dependência/worker/lock;8.cancelamento/foco/câmera/revisão e
scope preservados, aviso de sessão não é estado de jogo;9.gates acima;
10.custo síncrono O(passes×T×centros+V×centros), scans extras só quando footprint
não alcança3vértices, raiz por face e até8candidatos numéricos mantidos. Sem FPS,
remesh geral, suporte maior ou aceite humano inferidos.

App aponta ao binário corrente21388688B, SHA256
38ab194b8ef08678420912327e23560ab988a44d5c1fd37044386b8bd93d73c1.
A janela humana precisa salvar/reabrir para usar esta versão. Deformações antigas
continuam no documento, sem reparo automático; EAF-110 segue In Progress.

Retenção desta unidade: 45outputs anteriores confirmados/inativos removidos (432900960B) com lock/stat/lsof. Sem cópia histórica de app/instalador. Debugmetadata de check inicial abortado fica para reconciliação com os checks paralelos que passaram a usar o checkout; não foi apagado cache de dono incerto.


## EAF-110 — seleção de superfície nas diagonais do X

Repro30/10 em main/c316e5eeb5acde56, sem janela humana: Cube24, câmera[-7,1.05,0]→[-3,1.05,0],800²/fov48, diagonais±180px. PickMCP em uma aresta retorna localx+1 em vez de−1; no segundo traço cresce48→160amostras por saltos de2m. APIs atuais rejeitam>128; nativo trunca. Mesmo numa diagonal do cubo novo há falhas. Evidências points-*.json/asset-*.json/PNG em artifacts/sculpt-x, geometria e luz separadas.

Causa identificada no código: Möller-Trumbore f32, testes independentes u/v estritos rejeitam ambos triângulos na borda por arredondamento. Medir crista usando centros 3D fornecidos diretamente não exercitava isso.

Decisão antes do código: interseção estanque por projeção cisalhada no eixo dominante do raio, funções de aresta idênticas/sinal oposto para vértices compartilhados. Cálculo temporário f64, geometria/formatos continuamf32. Coeficientes por raio, sem tolerância que engorde faces ou feche buracos reais. Sem mudança de brush, budgets, força, topologia, shaders, Scene/Replay/Checkpoint/ABI/deps. Preservar segmento0..1, dupla face, menor distância e desempate por ID/face. Checar colapso real de face posada e raio paralelo; baricêntricas corretas mapeiam ao repouso como antes. Referência primária: https://pbr-book.org/4ed/Shapes/Triangle_Meshes (interseção ray/triangle).

Checklist prévio:1.só picking por movimento/consulta;2.structde coeficientes Copy e3DVec temporários, sem novoVec/pose/heap;3.f64somente interseção para precisão necessária, Hit/Scene f32;4.eixo/shear calculados uma vez por raio;5.mesmo scan limitado instâncias32/131072tri e O(T);6.sem upload/readback/I/O novo;7.sem threads/locks;8.cacheHit e revisões existentes, capturas independentes;9.prova crítica deve falhar antes em ambas diagonais e passar sem selecionar fundo, com controle de buraco verdadeiro, picking transformado/nearest/undo/MCP/native X cruzado;10.custo double portri mantido explicitamente, sem alegarFPS/BVH. Alternativasdescartadas:mais subdivisão/força/sombra não corrigem cursor na face errada; epsilon amplo esconderia gaps; impedir face traseira por winding falha em superfícies abertas/doislados.

Prova expandida: X nas duas ordens, cursor real pela seleção de tela, não centros3D fornecidos. Registrar cada hit/centro e exigir nenhuma troca para x+1 ou ponte dentro do cubo; snapshots/undo duplo, comparar perfil completo e imagens com worklight e sem sombra. Screenshot de antes preservado; não atribuir todo resíduo visual à seleção até depois de revisar a geometria nova.


### Correção de seleção nas arestas compartilhadas

A falha é anterior ao pincel: Möller-Trumbore f32 podia rejeitar ambos triângulos
na aresta frontal, fazendo pick retornar a face traseira. Interpolar esse salto
inventa caminho dentro do cubo e esgota128centros. Os gates anteriores por centros
3D ou só altura da crista não exercitavam esse erro do cursor.

TriangleRay transforma o segmento no eixo dominante, usa funções de aresta
consistentes entre vizinhos e f64local, sem epsilon espacial que feche buracos.
Face de ambos os lados/segmento0..1/nearest/desempate e baricêntricas de repouso
preservados. Nenhuma topologia/força/pincel/shader/budget/formato novo.
Referência primária: [PBRT4](https://pbr-book.org/4ed/Shapes/Triangle_Meshes).

Gate crítico novo em editor_mesh:162consultas nas duas diagonais não escolhem
fundo, e uma fenda real de0.00002m continua revelando fundo.3testes existentes
cobrem primitivas, nearest transformado e undo/save/load/replay.3MCP/15grupos
(editor_mesh/editor_mesh_selection/mesh_topology) passaram em root exclusivo,
sem substituir mcp-validation de circulação. ProbeGraphics faz os dois traços
consecutivos em ambas ordens (não undo entre eles):49+49centros/frontonly em cada
ordem, prévia=commit/undo único por traço/undo duplo exato. ReproduçãoMCP:48+48em
ambas ordens, com e sem detalhe, zero saltos para trás; antes saturava128.
Capturas frontais/oblíquas, worklight/sombra desligada revisadas. Densidade e
luz podem ainda revelar aproximação poligonal; não inferir aceite humano/artístico
ou capacidade ilimitada dessa correção de seleção.

Provas nos builds2a702a013f64c890 e62a7a16a267eb016 (IDs por relatório). Fontes globais/target avançaram por outras frentes
durante esta janela; validation.json separa build testado e binário observado,
sem afirmar validação dos recursos de circulação/apresentação/world-space.
Clippy focado passou depois de simplificar retorno Ok+? redundante num hunk
QueryWorldSpace da outra frente; essa linha não pertence ao commit da escultura.
Artefatos before/after/points/asset/logs em artifacts/sculpt-x e Xnativo em
editor-sculpt-quality/cross-*. Sem cópia histórica de app, sem interação na sessão
humana. Defeitos já gravados não são reconstruídos; EAF110 mantém In Progress.

Revisão posterior:1.só picking/movimento/comando, sem ticknovo;2.coeficientes e
3vértices no stack, sem coleções/poses extras;3.f64necessário local, Hit/Scene
f32;4.eixo/shear uma vez por raio;5.O(T)/tetos existentes, sem BVH ou claimFPS;
6.sem GPU/I/O/readback novos;7.dispatcher/locks iguais;8.cacheHit/revisão mantém
invalidação;9.arestas reais/fenda/nearest/X/undo/replay acima;10.custo double por
triângulo mantido conscientemente para classificação correta de bordas.
