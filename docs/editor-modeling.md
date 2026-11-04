# Modelagem por componentes (EAF-130)

Continuidade autorizada de EAF-104 na main0bea5d6. Extrusão/inset/bevel são
operações tipadas sobre seleção escopada por revisão, compartilhadas entre MCP
e UI. Nenhuma dependência ou versão Scene/Replay/layout nova: muda a geometria
existente e mantém Asset/Instance/rig/clipes/canais. A sessão humana fica intacta.

Alternativas: importar um kernel BMesh/boolean geral ou construir uma malha de
meias-arestas persistente ampliaria formatos/dependências e superfície de migração.
Escolha deste incremento: planos transitórios sobre a triangulação atual e
operações com domínio explicitamente validado. Não chamar este recorte de bevel
arbitrário ou modelagem equivalente ao Blender completo.

- Extrusão: região conectada orientada/manifold com fronteira; duplica a tampa,
  remove as faces antigas da região e costura a fronteira. Deslocamento local
  explícito; UI deriva direção normal média e distância. Fronteiras internas
  permanecem furos; sem tampa interna duplicada ou solda por proximidade.
- Inset: disco plano convexo, borda única de até256v. Offset constante por
  interseção das retas internas; vértices internos seguem coordenadas de valor
  médio. Colapso, concavidade, furos e inversão são recusados neste incremento.
- Bevel: chanfro plano (um segmento) de cadeias retas em sólido convexo fechado.
  Clipping por semiespaços compartilha interseções por índices, cria as tampas e
  mantém fechamento. Expansão à cadeia colinear é parâmetro explícito (visível
  na UI); sem expansão, seleção parcial é recusada. Até32cortes/128faces
  geométricas/4096v/8192t de entrada, além dos limites globais existentes. Não
  suporta sólido côncavo, bevel parcial ao longo de uma cadeia ou perfis curvos.

Preservação: coordenadas materiais implícitas são congeladas; cópias mantêm
coordenadas e pesos, interseções interpolam ambos. Mais de4influências positivas
recusa, sem poda. Ossos/clipes permanecem; vértices originais não tocados
conservam dados. Remover apenas vértices que ficaram órfãos pela operação,
preservando isolados preexistentes. Fontes/conexões de plantas exigem detach
explícito. Identidade de componente continua revisão-escopada, sem IDs globais.

A UI oferece operação/valor/prévia no inspetor do modo Editar. Selecionar faces
habilita Extrudar (Ctrl+E) e Inset (I); selecionar arestas habilita Bevel
(Ctrl+B). Na prévia, arrastar na viewport ajusta a distância/largura e soltar
confirma; Shift reduz a sensibilidade. Também é possível arrastar/digitar nos
campos do inspetor e usar Confirmar operação/Enter. Esc cancela. Atalhos só
atuam na viewport sem um campo de texto em edição; mudança de escala HiDPI
também cancela gestos. Nenhum gesto modifica o documento a cada amostra.
Prévia fora do Engine, aplicar uma vez gera um undo e seleciona as faces criadas;
Esc/cancelar/foco/revisão/câmera/seleção/layout descartam. Alterar parâmetros
recalcula desde a base, sem acumular topologia em idle. O MCP também consulta
relatório e prévia pelo mesmo planejador, antes de confirmar.

`model_editor_mesh { instance, reference, operation, preview }` oferece o fluxo
com seleção resultante. `edit_mesh`/`preview_mesh_edit` aceitam
`kind: model_selection`, asset, reference e operation; a consulta inclui
modeling_report. O relatório descreve effective_edges nos índices da malha de
origem; selected_faces são índices do resultado. Edit_scene pode compor o mesmo edit numa transação, mas não
aceita reutilizar uma referência de componentes depois de mudar esse asset.

Validação geométrica cobre orientação/arestas manifold/degeneração e os domínios
explicitados. Não há detecção global de auto-interseção para uma extrusão que
atravesse outra parte da malha ou outro objeto. Não há bevel arredondado, inset
côncavo, operações booleanas ou novo sistema persistente de IDs de componentes.
Essa fronteira descreve as três operações originais. A subtração convexa finita
adicionada em EAF-221 tem contrato estático próprio e entrada MCP-first em
[mesh-chipping.md](mesh-chipping.md); não amplia o domínio de extrusão/inset/bevel.
O bevel pode refazer a triangulação das faces cortadas. Rig/clipes são
preservados e pesos interpolados; isso não substitui revisão artística de pesos.

## Checklist prévio de performance

1. Planejar só por operação/valor alterado; idle desenha resultado cacheado.
2. Arc para snapshots, buffers de geometria limitados; sem Simulation por prévia.
3. Enums/índices e f64 no cálculo geométrico; f32 no documento, sem JSON no kernel.
4. Adjacência/planos transitórios por avaliação; sem reexecutar kernel no redraw.
5. Preflight e validação em cada etapa, seleção até4096faces, inset borda256,
   bevel32cortes/128planos e budgets globais; truncamento nunca é sucesso parcial.
6. Mesma ABI GPU/renderer; nenhum I/O/readback no gesto/kernel.
7. Dispatcher sequencial, sem worker/lock/dependência novos.
8. Referência inclui scope/revision, sessão/câmera/layout invalidam prévia; commit
   revalida, plantas/material/rig protegidos e erro não altera histórico.
9. Testar fechamento/orientação/área/volume/seleção/rig/canais/budgets/refusais,
   preview=commit, undo/redo/save/load/replay/PNG e entrada/UI com foco/HiDPI.
10. Clipping por corte×triângulos e MVC por interior×borda são custos conscientes
    de autoria limitados; sem promessa de FPS, otimização especulativa ou benchmark.

Referências primárias de comportamento e fórmula, sem copiar código:
[Extrude Region](https://docs.blender.org/manual/en/4.5/modeling/meshes/tools/extrude_region.html),
[Inset Faces](https://docs.blender.org/manual/en/5.0/modeling/meshes/editing/face/inset_faces.html),
[Mean value coordinates](https://www.inf.usi.ch/hormann/papers/Hormann.2006.MVC.pdf).

## Revisão estática do incremento

1. Frequência: evaluate apenas ao iniciar/alterar parâmetros ou confirmar. Idle
   mantém Arc de geometria e BVH da prévia. Normal média é derivada uma vez ao
   iniciar extrusão; o commit reavalia desde a cena atual sob guarda de referência.
2. Alocações: Scene clone compartilha assets/rig/clipes em Arc; planos/índices e
   buffers geométricos são temporários limitados. Nenhuma Simulation por preview.
   A visibilidade monta TriMesh uma vez por resultado, não por overlay/idle.
3. Dados: Operation/Report tipados, cálculo geométrico f64 e saída f32. JSON só na
   fronteira MCP/dispatcher; conversão do pequeno Report na confirmação nativa.
4. Repetição: interseções do bevel por edge-index cacheado a cada corte. Subdivisão
   colinear não vira canto falso; centro/limites do gesto fixados no press. Frame
   usa o resultado cacheado; UI não invoca renderer offscreen por gesto.
5. Limites: model_selection calcula teto por asset/storage/draw antes da alocação,
   confere contagens finais. Builder verifica cada push/triangle e compacta órfãos.
   Bevel tem limites intermediários conservadores: pode recusar uma operação que
   caberia só após compactação. MVC por interior×cantos; recorte por até32cortes.
6. GPU/I/O: RenderFrame/pipelines/shaders/formato inalterados; upload segue invalidação
   de geometria existente. BVH usa as caixas/malhas da cena prévia. Sem readback/I/O
   no gesto, apenas nas ferramentas explícitas de captura/harness de validação.
7. Concorrência: nenhum worker/lock/PRNG/rede/dependência novo; planejar e confirmar
   no dispatcher sequencial. O editor humano não é usado por nenhuma regressão.
8. Invalidação: scope/revision/tick/câmera/editor/session/seleção/layout/HiDPI/foco.
   Release após cancelamento é consumido; prévia válida mantém-se até dispatch para
   não piscar para a base. Falha do comando limpa a prévia. Plantas/conexões exigem
   remoção/detach explícito; rig/clipes mantidos e pesos >4 recusados, sem redução.
9. Evidência: testes de integração geométrica e de histórico/UI, probe GPU de
   botões/gesto/seleção/undo/cancelamento/foco/resize/HiDPI e cenário MCP com
   preview/commit/erro/batch/plantas/detach/save/load/reset/replay/prefixo/PNG.
   Totais e build final em validation.md; aceite humano permanece separado.
10. Custos mantidos: cópias de geometria/BVH por valor alterado e scans limitados da
    cena. Não há kernel boolean geral, detecção de auto-interseção global, perfil
    de bevel curvo ou IDs estáveis de componente. Sem ganho de FPS alegado.

A política de testes por risco registrada durante o checkpoint final prevalece
para a continuidade. A rodada completa deste incremento já estava em execução;
os novos testes protegem corrupção de topologia/canais/rig, commits parciais,
referências obsoletas e publicação indevida de prévias canceladas. Não se exige
repetir a suíte inteira para ajustes visuais/documentais futuros.
