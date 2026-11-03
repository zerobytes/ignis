# Autoria no editor

## Seleção e visibilidade — decisão anterior (29/10)

Novo feedback humano: raio X deve ser uma flag independente, clique vazio sem
Shift limpa componentes, arrastar cria seleção retangular; ações da área devem
ocupar ícones no canto direito do cabeçalho. Manter tema/tokens e tooltips/foco.

Escolha: oclusão geométrica por BVH TriMesh do Parry já dependência do Rapier,
construído do RenderFrame real (meshes posed/consumo de plantas e caixas/cunhas).
Cache por revisão/tick na UI; câmera muda só os raios. Evita readback GPU e um
pipeline novo de overlay; não usar normal frontal como substituto de oclusão.
O BVH tem custo síncrono de construção e memória proporcional ao frame limitado;
não há alegação de FPS. Raio X contorna o teste e o padrão é superfície visível.

Novo comando compartilhado seleciona região em pixels da viewport com instância,
referência de revisão, domínio vértices/arestas/faces e modo aditivo. Vértices usam
posição, arestas/faces seu centro; raio X da sessão governa alcance oculto.
Região vazia limpa sem Shift e conserva com Shift; resultado continua seleção
de sessão, sem alterar Scene/Replay/undo. Arraste cancela em foco/Esc/revisão/
seleção/câmera/resize e nunca altera gameplay nem confirma escultura.

Checklist prévio:1.só BVH quando overlay visível ou seleção por pedido;
2.um cache corrente, sem clone Scene e sem captura GPU;3.pixels locais, IDs e
referências separados;4.mesmo projetor/oclusão para UI/MCP;5.tetos de frame e
viewport existentes;6.nenhum shader/recursoGPU/dependência novo;7.sem worker/lock;
8.invalidação de revisão/tick e gesto obsoleto;9.frente/fundo, outra malha/caixa/
cunha, Shift/vazio/retângulo/resize/áreas e render/replay/MCP;10.BVH/poses,
raycasts de pontos/amostras dos segmentos e egui síncronos são custos mantidos.

A edição de nome, transformação, aparência e componentes publica as mudanças
imediatamente enquanto a simulação está pausada. Arrastar um campo é um gesto
com um undo; foco em outro campo/clique inicia outro grupo. Edições inválidas não
mudam cena/histórico. O dispatcher compartilha `edit_scene_live {gesture,edits}`;
só revisões consecutivas do mesmo grupo coalescem, sem persistir o grupo.
Uma tentativa recusada fica em rascunho e só é reenviada após outra mudança;
frames ociosos não repetem validação/comandos. Uma revisão externa atualiza o
rascunho. Autoria de componentes/rig fica desabilitada durante a simulação.

## Caminhos principais

- Adicionar → NPC / personagem, ou o atalho no inspetor vazio. O novo personagem
  fica selecionado; alvo, velocidade, distância, percepção e patrulha ficam em
  Propriedades → Componentes e comportamento. Configurações adicionais usam
  formulários, listas, escolhas e referências aos objetos existentes. Alvo é
  explícito; criar NPC não escolhe combate/onisciência automaticamente.
- Adicionar → Malha editável. Objeto edita instância/material; Escultura mantém
  os nove pincéis reais ([contrato](sculpt-brushes.md)); Editar escolhe vértices, arestas ou faces pela superfície.
  Shift acrescenta componentes. Centro local move a seleção imediatamente;
  subdivisão e suavização são operações explícitas. Faces oferecem Extrudar e
  Inset; arestas oferecem Bevel convexo plano. A prévia aceita arraste na vista
  (soltar confirma) ou campos/Confirmar no inspetor, com Esc para cancelar.
  Edição altera o asset e
  afeta suas instâncias; cortes/bridges/geradores avançados estão no catálogo.
  Selecionar um personagem retorna ao modo Objeto; Editar/Rig ficam indisponíveis
  sem uma malha, com a pré-condição visível no tooltip.
- Vista mostra arestas, vértices e esqueleto da malha selecionada. Sobreposições
  de malha mostram só superfície visível por padrão; **Raio X da malha** inclui
  estrutura oculta, inclusive na seleção retangular. Ossos são guias independentes.
  Nada participa de colisão/sensores ou muda render de gameplay.
  `set_editor_mesh_view`/inspect_editor compartilham flags de sessão, incluindo
  `xray:false` por omissão; Scene/replay/layout permanecem83/87/3.
- Rig abre criação de raiz/filhos, edição de repouso e pesos da seleção. Escolha
  vértices em Editar, volte a Rig e atribua o peso ao osso; o restante vai à raiz.
  Ossos/clipes usam EditMesh e são persistidos normalmente. Criar clipe,
  inserir/substituir chave por osso e atribuir o clipe permitem autoria/reprodução
  inicial. Timeline gráfica, IK, retargeting e pintura de pesos não estão concluídos.
  O modo Rig abre seus controles mesmo se a seção estava recolhida no modo Objeto.
- Ferramentas oferece busca/categorias e formulários derivados do Command usado
  pelo MCP. Tipos opcionais, enums, listas paginadas, mapas e resultados estruturados
  usam o mesmo contrato; referências de malha vêm do contexto selecionado. Operações
  destrutivas de troca de documento passam pela confirmação de cena suja. O
  catálogo é acesso avançado, não prova que todos os sistemas já tenham editores
  especializados ou uma experiência visual completa.

## Contratos e limites

TransformVertices usa referência de revisão, seleção não vazia sem duplicatas e
transformação finita com rotação unitária/escala uniforme positiva. Índices,
material, rig, pesos e clipes são preservados. Coordenadas de material implícitas
são congeladas; colapso/inversão f32 e plantas vinculadas exigem recusa/detach
explícito. A UI reinspeciona sua seleção após alterações próprias sem topologia;
revisões externas a invalidam. Arestas e faces conservam seu domínio no dispatcher
e na subdivisão; movimento, suavização e pesos usam seus vértices associados.
Arestas são escolhidas pela distância ao segmento projetado, faces pelo primeiro
triângulo atingido. Extrusão/inset/bevel têm operações próprias e prévia no
inspetor ([contrato e limites](editor-modeling.md)); dissolução e remesh
continuam ausentes.

No macOS, fullsize content/título transparente conservam os botões nativos e
colocam o menu no cabeçalho da aplicação. Espaço livre permite drag/duplo clique.
A compilação e o compositor offscreen não comprovam hit targets nativos,
acessibilidade ou foco físico; esses gates e aceite humano ficam separados.
A janela/cena pessoal não é reiniciada por regressões.

## Revisão estática

1. Alterações apenas por gesto/controle/operação; frames idle não publicam edits.
2. Schemas/formulários base cacheados em OnceLock; nenhuma reflexão por tick.
   COW de Scene/Geometry e histórico32 mantidos; só o primeiro update de um grupo
   guarda sua cena anterior. O rascunho pequeno do osso é copiado por frame;
   rig/pesos e cache da última tentativa recusável somente ao editar.
3. Referências opacas e índices separados; seleção usa revisão e mantém limites.
4. Operações reutilizam dispatcher, kernel Sculpt e transformador comum a ilhas
   e vértices. Preview e commit concordam; Draw congela sua base inteira.
5. Catálogos/pools não foram ampliados. Arrays em formulários são paginados16,
   inspeção avançada limitada por profundidade; limites do domínio continuam finais.
6. Nenhum shader/recursoGPU/dependência novo. Overlay só quando solicitado;
   arestas deduplicadas são cacheadas por geometria e seleção busca por índice.
7. Mesma execução serial, sem workers/locks novos. Capturas/arquivos sob pedido.
8. Erro valida antes de publicar inclusive em updates coalescidos; undo/revisão
   externa/reset quebram grupo. Caches de campos rebasam em alterações externas.
9. Rust release/all-targets, Clippy, MCP completo, probes GPU/entrada e teste dos
   dois sentidos da escultura; teclado/foco/título nativos continuam gate separado.
10. Reconstruir runtime por alteração e poses/projeções do overlay selecionado
    são custos síncronos mantidos. Sem afirmação de FPS, multidões ou paridade
    de UX integral por contar comandos no catálogo.

Passos de uso sem os detalhes do contrato: [Primeira cena](editor-first-scene.md).

## Retângulo e ações de área

No modo Editar, arraste com o botão esquerdo. O retângulo fica visível enquanto
o mouse está pressionado; a seleção só é publicada ao soltar. Shift acrescenta
sem duplicar componentes; sem Shift substitui. Clique vazio sem Shift limpa e
não troca a malha em edição. Esc/perda de foco, câmera, resize, revisão ou troca
de seleção cancelam o gesto inteiro. Limpeza externa/MCP não é restaurada pelo
rascunho do inspetor; só a transformação própria sem topologia reinspeciona.

`select_editor_mesh_region` recebe `instance`, `reference`, `component`
(`vertices`, `edges`, `faces`), `min/max` em pixels locais, `width/height` e
`additive`. Usa a câmera e `mesh_view.xray` da sessão. Vértices usam sua posição;
arestas/faces, o centro. Retângulos invertidos/fora da viewport, referência
obsoleta e fonte consumida de planta são recusados antes de mudar a seleção.

A visibilidade reutiliza o near plane do picking e a geometria posed do
RenderFrame, incluindo volume real de caixas/cunhas e consumo de conexões.
TriMesh/BVH é reconstruído só após revisão/tick; câmera/resize/flag não retêm
outros históricos. Um viés de poucos ULPs evita auto-oclusão. Arestas visíveis
são amostradas em trechos de aproximadamente2pontos lógicos, até2048por aresta,
coalescendo trechos consecutivos; não é um novo pass GPU de wireframe.

Cabeçalhos oferecem ícones vetoriais para maximizar/restaurar, dividir lado a
lado/acima/abaixo, abrir e fechar abas. Tooltips e nomes acessíveis conservam
os rótulos; popups mantêm texto para escolher o painel. Área de cena não oferece
fechar sua última viewport. Cabeçalho estreito agrupa modos num menu para manter
Vista/Iluminação e as ações de área disponíveis. Nenhuma mudança em layout3.


## Campos do centro da seleção — diagnóstico e revisão prévia (29/10)

Feedback humano: seleção de47vértices pisca ao arrastar posição e não publica
valor/geometria. Reproduzir entrada egui prolongada e recusa de uma transformação,
sem tocar a sessão humana. Hipótese: revisão prevista é instalada antes do
resultado; erro mantém revisão real e a UI invalida/reconstitui a seleção.
Preservar validação geométrica, referências e último documento válido; recusa
precisa manter campos/seleção estáveis e mostrar motivo junto do controle.

Checklist anterior:1.apenas evento/resultado do campo, sem novo trabalho no tick;
2.rascunho pequeno por seleção, COW/histórico existentes;3.revisão real difere da
prevista e centro local difere de posição da instância;4.reutilizar TransformVertices,
EditSceneLive e Scrub;5.tetos/validadores mantidos;6.sem GPU/readback/dependência;
7.mesmo dispatcher serial;8.erro não avança revisão local nem remove seleção,
revisão externa/undo limpam gesto;9.arraste contínuo47v, eixos/fino/digitação,
recusa/idle/recuperação/undo e probes/MCP;10.custos síncronos de validação e
reconstrução existentes, sem ganho de FPS ou aceite físico inferido.


Correção: revisão local só acompanha edição aceita; seleção canônica sincroniza
sua revisão ao reidratar inclusive após undo+nova seleção. Erros de comando do
host são encaminhados ao contexto de geometria, com aviso em português e motivo
original no tooltip, sem remover os campos. Arraste passa a0.001m/pixel; Shift
mantém ajuste dez vezes mais fino. O kernel continua recusando colapso/inversão.
Teste de47v reproduziu o desaparecimento antes da correção e agora cobre
recusa/idle/recuperação/updates/undo. Probe GPU cobre XYZ, cada vértice selecionado,
vértices não selecionados intactos, erro junto ao campo e undo exato por gesto.
Gates completos e evidências finais em validation.md.

## Continuidade: manipulação direta e acabamento

Gizmos Q/W/E/R, eixos/planos/anéis, orientação/pivô/snap e preview/commit/undo
compartilhados: [editor-gizmos.md](editor-gizmos.md). Seis kernels adicionais
completam nove pincéis, com forças independentes e base congelada:
[sculpt-brushes.md](sculpt-brushes.md). Essas entregas não concluem extrusão/inset/
bevel genéricos, máscaras/simetria/remesh/tablet, timeline/IK/pintura de pesos,
editores visuais especializados nem múltiplas cenas/viewports independentes.
