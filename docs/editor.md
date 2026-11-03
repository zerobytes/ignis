# Editor nativo

## Começar um jogo

Na tela inicial, **Criar projeto** pede nome e pasta (Documentos por padrão).
A Ignis cria uma pasta nova contendo o manifesto e a cena **Principal**, com piso
e jogador. **Abrir projeto** navega até uma pasta com ignis.project.json.

No painel **Projeto**, **Nova cena** cria outra cena; clicar em um nome abre a
cena, e **Usar como cena inicial do jogo** escolhe a entrada. Barra superior
mostra projeto, cena e asterisco de alterações; **Salvar/⌘S** grava a cena ativa.
Trocas e fechamento pedem salvar/descartar/cancelar quando há alterações.

**Adicionar** cria cubo visual, bloco sólido ou caixa física. O objeto nasce
selecionado; Propriedades edita nome/posição/dimensões/cor/visibilidade, duplica
ou exclui. Alterações são imediatas e usam o histórico compartilhado. Executar inicia a
simulação; clicar na viewport dá entrada WASD/setas, Esc libera.

**Área de trabalho** é a distribuição de painéis. Pode haver várias áreas de
trabalho dentro da sessão do mesmo projeto. Uma cena ativa por vez;
exportação, scripting e ferramentas avançadas de modelagem seguem pendentes.
Contrato, persistência e limites em [projects.md](projects.md).


Continuidade: a integração das branches e sua atualização horária estão em [editor-integration.md](editor-integration.md). As limitações abaixo descrevem o primeiro incremento; autoria gráfica recebida no núcleo ainda exige suas ferramentas visuais.

Direção aprovada em27/10/2026: ferramenta densa com menus suspensos, áreas de trabalho e regiões configuráveis, tomando Blender/Unreal como referências de organização. Grafite neutro, seleção laranja, sem área permanente para assistência não conectada. Implementação isolada desde main f6b707e; meshes/plantas/pesquisa continuam em suas frentes.

## Criar e esculpir malhas

**Adicionar → Malha editável** cria esfera, cubo, cilindro ou plano com tamanho e
resolução configuráveis. A nova malha fica selecionada; também é possível
selecioná-la clicando na superfície ou em **Malhas** na hierarquia. **Enquadrar/F**
centraliza a câmera. Em **Escultura**, arraste sobre a superfície:

- **Volume/1** acrescenta volume; **Ctrl** inverte para cavar.
- **Mover/2** puxa a região com queda suave de influência.
- **Suavizar/3**, ou **Shift** ao iniciar o gesto, suaviza a região.
- **Achatar/4**, **Pinçar/5**, **Vinco/6**, **Raspar/7**, **Preencher/8** e
  **Relaxar/9** são operações próprias de forma/acabamento; Ctrl inverte as
  operações que têm inverso. [Contrato e limites](sculpt-brushes.md).
- **Raio (m)** e força ficam no inspetor do pincel; **Ctrl + roda** ajusta o raio.
  Volume usa metros por amostra; os outros mostram porcentagem.
  Cada pincel conserva sua própria força.
- Soltar confirma uma edição; **Esc**, perda de foco ou mudança concorrente
  descartam a prévia. **⌘Z/⌘ShiftZ** desfazem/refazem o gesto inteiro; **⌘S** salva.

Propriedades permite transformar, colorir, aplicar material procedural, trocar
sombreamento, subdividir, duplicar instância e excluir. Instâncias do mesmo
asset compartilham escultura/material. Malhas visuais não geram colisão
implicitamente. Este recorte trabalha na base estática; rig, animação,
attachments e fontes paramétricas/conexões exigem suas ferramentas próprias.
Na barra da viewport, **Iluminação** configura direção/altura da luz, ambiente,
sombras, alcance e resolução. O padrão **Escultura contrastada** usa ambiente
baixo e sombras; **Luz da cena** volta à iluminação autoral. Centro e bias da
sombra ficam em ajustes avançados. A luz de trabalho afeta somente a viewport.
**Arquivo → Salvar layout** guarda também pincel, raio, forças e luz;
carregar o layout restaura esses valores. Layouts antigos recebem os padrões.

O MCP oferece os mesmos nove pincéis da UI, pelo comando Sculpt compartilhado.
Outros operadores de malha não equivalem a pincéis interativos. Pressão de
tablet, máscaras, simetria e remesh continuam pendentes.
Contratos, custos e evidências em [editor-sculpt.md](editor-sculpt.md).

## Primeiro incremento — histórico das decisões

egui0.33.3/egui-winit/egui-wgpu usam Rust/winit0.30/wgpu27 existentes. Alternativa: frontend web com transporte e textura externos, que acrescentaria sincronização e outro ambiente. Alternativa: widgets e texto próprios, com custo de manutenção de foco, fontes e edição de campos. Escolhida infraestrutura egui; sem eframe/novo backend. A árvore de áreas é um contrato pequeno próprio, tipado e limitado, independente dos widgets. Biblioteca de docking economizaria interações mas acoplaria persistência/MCP a tipos externos; divisão/movimento explícitos e arraste de abas usam a árvore comum.

Estado de editor pertence ao Service, fora de Scene/Replay. Workspaces guardam árvore de splits e áreas com abas, seleção e maximização; todos os controles de layout passam pelo dispatcher. Erros validam cópia limitada antes de substituir. Arquivo de layout versionado separado, caminhos pelo confinamento do Service. Nenhum layout muda o tempo da simulação ou fornece conhecimento ao NPC.

Recorte: uma cena ativa por sessão, documento identificado na viewport, dois workspaces úteis (Cena/Inspeção), áreas viewport/hierarquia/propriedades/templates/eventos, menus de arquivo e simulação, seleção, transformação autoral com undo/redo. Não há editores de materiais/modelagem/animação integrados nesta base nem chat IA fictício. Múltiplos documentos simultâneos e janelas nativas destacadas são incrementos posteriores; não confundir abas de painéis/workspaces com suporte já entregue a esses recursos.

Viewport real renderizado em textura GPU reutilizada e composto com os controles, no mesmo dispositivo. Captura de editor offscreen usa a mesma composição, sem abrir janela, separada da captura de cena e dos testes de entrada nativa. Coordenadas de picking relativas à região física; campos/menus não enviam gameplay. Simulação é a mesma do MCP. Edição autoral conserva contrato existente de reset, explicitado ao usuário.

Câmera de inspeção opcional por workspace, independente da câmera autorada: botão direito orbita; botão do meio ou Shift+direito desloca; roda aproxima. Exibir → Restaurar câmera da cena remove o override. `SelectEditorAt` usa a mesma câmera e corpos atuais, preservando identidade e visibilidade do picking existente. Nenhum clone de Scene para navegar ou selecionar; `Camera::validate` e kernel `pick_camera` são compartilhados. Transformações são posição/dimensões de caixas; rotação não foi inventada onde a autoria não a suporta.

## Uso e fronteiras do primeiro incremento

App local para Finder: `python3 scripts/package_macos.py --editor --install` instala `~/Applications/Ignis Editor.app`, com a tela inicial de projetos. O launcher fixa a raiz do projeto, independentemente da pasta de trabalho do Finder; usa hardlink ou symlink ao build corrente entre volumes, sem cópia histórica. Não precisa de terminal para abrir. Reempacotar após substituir o build; projeto e volume precisam continuar disponíveis.

`./target/release/ignis --editor --root "$PWD" --scene examples/stealth-lab.scene.json`

Adicionar `--mcp` compartilha editor e transporte no mesmo processo. `--window` continua sendo a demo jogável anterior. Menus de arquivo pedem caminho relativo à raiz do projeto, com o confinamento existente. Layout é salvo/carregado explicitamente pelo menu, sem autosave nesta etapa. Arrastar abas move entre áreas; o menu Área divide, abre/fecha abas e maximiza. A viewport única é preservada; múltiplas vistas independentes, múltiplos documentos e janelas destacadas ainda não estão implementados.

Exemplos de comandos para `engine_command`:

```json
{"command":{"op":"inspect_editor"}}
{"command":{"op":"edit_editor","expected_editor_revision":0,"action":{"action":"resize","area":0,"ratio":0.7}}}
{"command":{"op":"edit_editor","action":{"action":"switch","workspace":"inspect"}}}
{"command":{"op":"save_editor","path":"editor.layout.json"}}
{"command":{"op":"capture_editor","path":"artifacts/editor-current.png","width":1440,"height":900}}
```

`CaptureEditor` usa o mesmo compositor com estado de UI isolado: reproduz layout/câmera/seleção/cena, não o foco, os menus abertos ou rascunhos ainda não aplicados da janela. Captura repetida e retorno de layout/câmera devem produzir os mesmos pixels dentro do build. A captura da cena continua livre de UI. Entrada física, integração com apresentação nativa e aceite humano ficam separados. Comandos da UI carregam revisões de cena/editor do snapshot para recusar alterações obsoletas durante controle MCP concorrente.

## Revisão estática antes de codificar

1. UI sob redraw/evento; tick de simulação não recebe layout, JSON ou widgets. Pausa sem mudanças não mantém animação contínua.
2. Contexto egui, GPU, textura da viewport e RenderFrame reutilizados. Clonar árvore pequena somente ao editar layout; entidade só em confirmação de edição.
3. Tipos próprios para painel/eixo/IDs/ações; limites de16áreas/32abas/8workspaces/profundidade8. Sem unsafe ou dependência de índices de NPC.
4. Dimensões da viewport medidas no layout; renderer conserva invalidação própria. Sem captura/readback para apresentar a cena nativa.
5. Hierarquia limitada pelas entidades do runtime, biblioteca pelos templates; logs de UI em anel64. Scans pequenos permanecem sem promessa de escala.
6. Texturas recriam apenas em resize; leitura GPU/PNG e persistência só por pedido. Build corrente desta worktree em /Volumes/Zero/dev/build-cache/ignis-editor/target.
7. Comandos antes/depois da construção da UI em fase serial; lock do Service não permanece durante desenho GPU, apresentação ou eventos do SO.
8. Layout valida IDs/ratios/abas e viewport presente; restore/load/resize invalidam visual. Selection/runtime continuam guardados pela engine.
9. Testar erros atômicos, split/fechar/mover/maximizar, workspace e persistência, limites, invariância da cena/replay, picking/edição e captura repetida. Rust/Clippy/MCP headless e gate nativo separado.
10. Custos mantidos: widgets imediatos, tesselação de texto e uma composição de textura por viewport. Sem ganho de FPS alegado; múltiplos documentos não são simulados com clones por frame.

## Revisão do incremento

1. Adequado: `editor_window::about_to_wait` agenda desenho por mudança/entrada/repaint ou simulação; layout não participa de `Simulation::step`.
2. Adequado: `Graphics` retém contexto, fonte, RenderFrame e textura; rascunho de entidade só recomposto por seleção/revisão. `State::apply` clona estado limitado em ações, inclusive navegação de câmera, para rollback integral.
3. Adequado: Panel/Axis/Area/Action tipados, IDs locais do layout separados dos IDs de mundo; nenhuma mudança de Scene63/Replay67.
4. Adequado: mesma câmera efetiva no frame/picking; `Camera::validate` e `pick_camera` reutilizados. Filtro textual de hierarquia permanece sob demanda com scans limitados.
5. Adequado: validação de8workspaces/16áreas/32abas/profundidade8; anel64 de mensagens e últimos64 eventos apresentados; limites do runtime intactos.
6. Corrigido: textura da cena só recriada por mudança de tamanho; egui atualiza vínculo existente. Apresentação sem PNG/readback. Leitura GPU e arquivos apenas por comando.
7. Adequado: construção UI/snapshot sob lock; tesselação, GPU e apresentação nativa fora. Captura MCP/persistência continuam síncronas no Service, como no contrato anterior.
8. Adequado: validar cópia antes de substituir, revisão independente, guardas da UI contra snapshots obsoletos; rascunhos invalidados por revisão. Load de layout inválido preserva sessão.
9. Adequado: testes de invariância/atomicidade/limites/persistência, projeção HiDPI/câmera e eventos UI; cenário MCP cobre compositor real e pixels repetidos/retorno. Evidências finais em validation.md.
10. Mantido com motivo: composição imediata e scans pequenos de autoria/diagnóstico, cópia limitada por ação, captura síncrona, uma cena/viewport ativa. Sem promessa de ganho de FPS, escala ou aceite nativo por inferência.

## Transformar na viewport

Em Objeto ou Editar, a faixa lateral oferece Selecionar/Q, Mover/W, Girar/E e
Escala uniforme/R. Arraste eixo, plano, anel ou alça; ··· configura global/local,
pivô e snap. Ctrl/Cmd ajusta a incrementos, Shift dá precisão, Esc cancela.
Malhas/componentes aceitam as três operações; caixas físicas aceitam movimento
e escala uniforme, sem rotação apenas visual. [Contrato](editor-gizmos.md).
