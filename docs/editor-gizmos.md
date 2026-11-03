# Transformação direta no editor

Incremento autorizado na main, depois de a74ddb4/1efe71f. Mover, girar e escalar
usam operações autorais existentes, expostas simultaneamente no dispatcher/MCP.
Não há nova representação de geometria, dependência ou versão Scene/Replay.

Decisão: prévia validada fora do Engine durante o gesto; somente soltar confirma
uma transação. Esc, perda de foco, seleção/câmera/revisão diferentes cancelam a
prévia sem consumir undo/redo. O mesmo planejador de transformação em coordenadas
mundiais atende UI e MCP. Referência inclui escopo/revisão para evitar reuso após
load/reset. Componentes mantêm seleção após a confirmação, com referência nova.

Limites de contrato: instâncias de malha e componentes aceitam rotação e escala
uniforme; caixas físicas aceitam movimento e escala uniforme, sem rotação livre.
Transformações de instâncias vinculadas/animação ativa são recusadas neste
incremento; nenhuma rotação apenas visual de colisor. Fontes paramétricas e
validação de faces continuam protegidas pelos validadores existentes.

## Checklist antes da implementação

1. Desenhar poucos segmentos por frame; planejar/validar somente quando o ponteiro muda.
2. Snapshot autoral por atualização efetiva; geometria Arc compartilhada até mutação. Sem Simulation clonada/recriada na prévia.
3. Tipos explícitos para ferramenta/espaço/pivô/alvo; sem reflexão/JSON no cálculo de arraste.
4. Congelar início, câmera, alvo e seleção; recalcular deltas totais, sem acúmulo incremental.
5. Respeitar os budgets de malha/cena existentes; nenhuma ampliação de limites.
6. Sem I/O/readback por gesto; renderer recebe documento da prévia e caixas ajustadas, sem nova ABI GPU.
7. Dispatcher sequencial; sem worker/lock/dependência novos.
8. Cancelar por revisão/seleção/câmera/foco/resize; confirmação revalida o mesmo comando.
9. Testar preview/commit/cancel/undo único, pivô/espaços/snap, alvos/ref inválidos, seleção de componentes e equivalência MCP.
10. Cópia/validação da cena por movimento efetivo é custo consciente de autoria, fora do tick. Otimização posterior depende de medição; nenhum ganho de FPS alegado.

## Uso e paridade

Na faixa lateral: Selecionar Q, Mover W, Girar E, Escala uniforme R. Mover
oferece eixos XYZ, planos XY/XZ/YZ e alça livre no plano da vista. Girar usa anéis
XYZ; anéis quase de perfil ficam sem picking para evitar saltos numéricos. A
alça diagonal/central escala uniformemente. O menu ··· controla orientação,
pivô e incrementos; Ctrl/Cmd ativa snap temporário e Shift reduz o deslocamento.
Ao sair do viewport o arraste continua no mesmo plano, sem prender ao limite da
janela. Arrastes entregues como press/move/release em uma única passagem usam o
ponto original de pressão, não o ponto final.

`set_editor_transform` configura sessão; `inspect_editor_transform` retorna alvo,
pivô e referência, aceitando components:true. `preview_editor_transform` e
`transform_editor` recebem o mesmo target/reference/transform (IslandTransform em
coordenadas mundiais). A consulta prévia não muda cena, simulação ou histórico.
Campos numéricos existentes continuam usando as operações autorais compartilhadas.

## Revisão estática e validação do incremento

1. Overlay limitado a3×65 pontos dos anéis e poucos segmentos; kernel só em
mudança de posição/modificadores. 2. Seleção/topologia em cache Arc por referência,
view/session revision e domínio; teste ptr_eq comprova reuso em idle. Snapshot
Scene por movimento efetivo permanece custo declarado. 3. Enums/quat/vetores
f32; soma/correção de faces usa kernel existente. 4. Deltas desde o início,
prévia mantida até confirmação para evitar flicker, sem scene clone no frame
parado. 5. Budgets e validadores mantidos. 6. Sem ABI GPU/readback novo;
RenderFrame é atualizado por documento de prévia e ordem filtrada de entidades.
7. Nenhuma thread/lock novo. 8. Referência inclui escopo, cache/gesto invalidam por
revisão/sessão/câmera/foco/layout; confirmação revalida. 9. Sete testes de entrada
egui e quatro testes de contrato cobrem três ferramentas, pivô/local/global/snap,
fino, arraste coalescido, erros, undo/redo, cancelamentos e componentes. Probe
GPU editor_gizmos_probe confirmou mouse→prévia→commit→undo e capturas reais em
1440/1000/900/800, sem janela. 10. Stage/Scene::validate por movimento e projeções
pequenas por frame são custos mantidos; sem profiling ou promessa de FPS.

Logs/capturas em artifacts/editor-gizmos*.log e artifacts/editor-gizmos/. Gates
completos do conjunto final são registrados em docs/validation.md; confirmação
nativa/humana permanece separada. Não declara modelagem completa, pesos/IK,
novos pincéis, múltiplos documentos ou editores especializados como entregues.

### HiDPI e viewports grandes

A projeção/ray de gizmos usa exatamente as dimensões físicas arredondadas e
limitadas a4096 da textura da viewport, mapeadas de volta para coordenadas
lógicas. Não pressupõe que o aspecto físico permaneça igual ao retângulo quando
somente uma dimensão satura. Mudança dessas dimensões invalida o gesto.
Regressão compara projeção e ray com picking compartilhado em2× e1.5×, inclusive
3000×2000 lógicos→4096×4000 físicos. Correção identificada durante revisão antes
da entrega; somente a compilação de testes desta execução foi interrompida para
refazer o gate com a fonte corrigida. Nenhum processo/janela do usuário alterado.

Atalhos aceitam foco do widget viewport_image e recusam foco de campos de texto
e combinações Ctrl/Cmd/Alt. Um teste próprio cobre viewport focada, campo ativo
e simulação, além da separação de gameplay no host nativo. O mesmo predicado é
compartilhado com F/1..9 na escultura; números digitados em campos não trocam pincel.
