# Refinamento da interface do editor

## Direção e critérios

Usuário confirmou escultura funcional e pediu alinhamento, padding, organização,
hierarquia e UX. Direção existente: Cena primeiro, ferramenta densa, menus no
topo, painéis configuráveis, grafite e laranja. Público: autoria/teste de jogos
com mouse/teclado. A viewport é o foco; controles precisam ser legíveis sem
competir com o modelo. Interações frequentes sem animação. Manter Rust/egui,
comandos compartilhados, capacidades e formatos existentes.

Referências de revisão: [artigo Kimi](https://www.kimi.ai/pt-br/resources/ui-ux-design-skills-for-agents) e [ui-craft](https://github.com/educlopez/ui-craft), sobretudo
layout/grouping, review e formulários. Aplicar agrupamento por proximidade,
contraste/estados explícitos e hierarquia de ações ao editor nativo. Receitas de
landing pages/dashboards web não definem esta ferramenta. Não instalar CLI,
framework, fonte ou serviço externo para realizar o refinamento.

## Auditoria do estado apresentado

| Estado observado | Alteração | Efeito esperado |
|---|---|---|
| Ferramentas, restauração e instruções em uma linha contínua | Grupos de modo, pincel e parâmetros; ajuda/restauração no menu contextual | Encontrar o controle sem percorrer uma frase |
| Sliders da iluminação sem trilho no menu | Janela de ajustes com controles de formulário e colunas consistentes | Ler direção, valor e faixa visualmente |
| Propriedades sem seções, ações repetidas empilhadas | Transformação, aparência, geometria e instância; ação aplicar destacada | Separar editar valores de executar operações |
| Cabeçalhos/seleções todos alaranjados e cinzas muito próximos | Abas neutras, hierarquia de superfícies/textos e seleção contida | A cena e a ferramenta ativa mantêm prioridade |
| Linha de projeto vazia ocupa espaço permanente | Contexto de projeto compacto junto às áreas de trabalho | Recuperar altura útil sem esconder criar/abrir |

## Tokens e componentes

Espaçamento lógico 4/8/12/16; intervalo interno menor que entre grupos/seções.
Altura de controle 24, cabeçalho 32, inputs numéricos monoespaçados e valores
alinhados. Corpo 13, metadados 11, título de seleção 16. Fundo RGB 24/27/31,
painel 31/35/40, campos 40/46/54, linha 60/66/75, texto principal 224/228/234
e secundário 167/176/189; acento laranja 239/151/91. Raio 3 para controles e 6
para popovers. Foco/hover/seleção têm estados distintos.
Sliders com trilho e preenchimento legíveis; colunas de rótulos/valores constantes.

Componentes locais compartilhados para linha de campo, vetor XYZ, seção e ação.
Grupos da barra de ferramentas quebram juntos em janela estreita, sem separar rótulo/valor.
Menus/abas/layout permanecem disponíveis. O refinamento não altera autoria,
simulação, kernel dos pincéis, Scene 75/Replay 79/layout 3/manifesto 1.

## Checklist anterior de performance

1. Eventos/controles locais; nada no tick.
2. Estilo e pequenos helpers sem clones de Engine/Scene/geometria.
3. Pontos lógicos/HiDPI, IDs estáveis e label/value unidos.
4. Reuso de widgets, dispatcher e renderer.
5. Mesmos limites de documentos, listas e amostras.
6. Sem shader/pass/readback ou recursos GPU adicionais no produto.
7. Sem dependência/thread/lock/fontes novos.
8. Revisões/drafts/undo preservados; popovers continuam consumindo o clique.
9. Capturas reais normal/compacta, interação de parâmetros/iluminação e regressões de editor/projeto/seleção/replay, Rust release/all-targets/Clippy.
10. Layout imediato e formulários síncronos mantidos; nenhuma alegação de FPS nem aceite humano por captura.

## Revisão posterior e reprodução

`editor_design_ui` concentra tokens, abas, campos, vetor XYZ, ações e seções.
Não há novo estado de simulação, formato, dispatcher ou framework. Campos
empilham rótulo/valor em painéis abaixo de 270 pontos; XYZ passa a linhas abaixo
de 190. Abas podem rolar horizontalmente com menu de área reservado à direita.
A hierarquia mantém seleção em toda a largura e também filtra malhas pelo identificador.
O projeto ocupa um menu de contexto junto às áreas de trabalho, além do menu
Arquivo e painel Projeto. Criar/abrir continua usando o navegador de pastas.

A iluminação usa uma janela de formulário independente do pop-up de cor, sem
o estilo de menu de ações que removia o trilho dos sliders. A janela fecha pelo
X ou pelo botão Iluminação; clicar fora fecha apenas o seletor de cor. Alterar
a luz da cena ativa sua prévia automaticamente. Campos de luz/sombra ficam
alinhados e ajustes avançados
são recolhíveis; altura limitada com scroll. Pincéis, raio/força e modo permanecem
na barra contextual; restauração/ajuda estão em Opções e tooltips. Parâmetros
mantêm unidades reais e percentuais do contrato anterior.

Inspeção de cor usa cópia local e só publica no draft após `Response::changed`,
preservando RGB exato em repouso. Isso evita botão Aplicar ativo por conversão
RGB/HSV do widget. Sem mudança no kernel da escultura ou nas cores autorais.

Custos mantidos: layout egui imediato, strings curtas por frame, filtragem linear
das listas limitadas, picking/normais/preview síncronos anteriores. O probe cria
GPU offscreen/readbacks somente para verificação, sem controles no app aberto.
Não há claim de FPS, compatibilidade nova ou promoção de estudos gráficos.

Reproduzir UI real, interações e capturas sem janela:

```sh
cargo run --locked --release --example editor_ui_probe
```

O probe usa Service temporário, comandos compartilhados e a composição Graphics
real; clica Escultura, ajusta Ambiente no popover e restaura contraste, abre o
formulário Novo projeto sem criar arquivos pessoais. Verifica toolbar/campos em
1440/1000/800 pontos, formulário em 900×600, mundo inalterado e ausência de edição
pendente em repouso. Capturas estáveis `artifacts/editor-design-*.png`; saída JSON
em `artifacts/editor-design-probe.log`. Cliques são sintéticos/offscreen: não
comprovam teclado físico, foco nativo, leitor de tela ou aceite humano.

## Relatório de revisão

- **Verificado:** toolbar, hierarquia, propriedades de malha/objeto, iluminação,
  contexto de projeto, criação de projeto, barras/abas, estados ativo/desabilitado
  e largura compacta na composição real.
- **Alterado:** medidas e cores compartilhadas; agrupamento, alinhamento e
  hierarquia; correção dos sliders, overflow numérico e falsa edição de cor.
- **Mantido:** arquitetura Cena primeiro, menus superiores, áreas configuráveis,
  MCP, autoria/undo/replay, navegador de pastas, pincéis e luz de trabalho.
- **Veredito:** pronto para avaliação visual após gates automatizados abaixo.
  Leitor de tela, fluxo integral por teclado e aceite humano desta apresentação
  continuam sem comprovação. Demais ferramentas do roadmap não fazem parte deste
  refinamento.

## Evidências finais

Build `adea604e9a6ffdd4`, fonte/hash do executável conferidos. `cargo test
--locked --release --all-targets`: 1.109 testes/106 alvos, zero falhas ou ignorados.
`cargo clippy --locked --release --all-targets -- -D warnings`, fmt/diff e build
release aprovados. MCP headless completo: 96 cenários/397 grupos, completed/passed.
Probe final passou em quatro larguras e renderizou as capturas acima; nenhuma
janela desktop foi aberta ou controlada. Logs `artifacts/editor-design-*.log` e
consolidado `artifacts/editor-design-validation.json`. Aviso futuro block 0.1.6
preexistente, separado de Clippy.

Um executável corrente de 17.705.216 bytes, cache release ~2 GiB; app instalado e
wrappers em uso apontam ao mesmo arquivo por symlink. Sessões, cenas pessoais e
processos anteriores preservados. Esta nova apresentação carrega em uma nova
abertura do app; a sessão em uso não foi reiniciada. Sem main/push ou instalador.
Guilherme confirmou funcionamento da escultura anterior; este passe visual ainda
aguarda sua avaliação no aplicativo.

## Segunda direção — reorganização nativa (29/10)

Feedback humano: refinamento anterior insuficiente; retomar após correção do
traçado167d8de. Referências ui-craft/layout e review aplicadas como orientação de
agrupamento, hierarquia e interação ao Rust/egui, sem importar receitas web.
Brief/tokens locais em .ui-craft. Antes do código: viewport dominante; ferramentas
na lateral, cabeçalho contextual local e inspetor Pincel/Objeto. A faixa global de
parâmetros desaparece; menus/workspaces/documentos/áreas continuam funcionais.
Campos usam componente horizontal com Shift fino e entrada textual/teclado nativos;
sem alteração global da entrada que possa interferir no viewport ou simulação.

Alternativas: framework/ícones/fontes externos acrescentariam recursos e superfície
de manutenção sem necessidade neste recorte; manter a longa toolbar não resolveria
organização. Escolha: vetores nativos para miniaturas/controles e egui para edição,
foco e acessibilidade. Nenhum controle de pincel futuro inerte; catálogo ampliado
precisa primeiro de kernels/contratos compartilhados. Formatos e kernel preservados.

Checklist anterior:1.layout/eventos da UI, nada no tick;2.sem clone de cena/mesh,
miniaturas vetoriais pequenas, contexto e textura reutilizados;3.IDs estáveis e
coordenadas lógicas/físicas separadas;4.widget numérico compartilhado em vetores,
pincéis e ajustes;5.mesmos budgets e128amostras/gesto;6.sem shader/readback/I/O
novo no produto;7.sem thread/lock/dependência/fonte;8.estado de edição/IDs/foco e
invalidação de gesto preservados, novo rect físico exclui ferramentas;9.testar
arraste horizontal/vertical/Shift/entrada textual, cliques sem atravessar, qualidade
de escultura, layout compacto e MCP/editor/render/replay/histórico;10.layout egui,
raycast/preview síncronos e trabalho por widget mantidos, sem alegação de FPS.

### Revisão do diff e relatório de design

| Anterior | Agora | Motivo |
|---|---|---|
| Barra global de modo/pincéis/parâmetros abaixo de menus e workspaces | Modo/luz incorporados ao cabeçalho da viewport; ferramentas em faixa lateral | Recuperar altura e aproximar cada controle de sua área |
| Propriedades do objeto dominam a lateral durante escultura | Inspetor Pincel/Objeto, com força/raio e contexto do pincel | Priorizar a tarefa ativa sem perder acesso à autoria |
| Campos com alvos pequenos e comportamento diagonal nativo | Scrub compartilhado horizontal, Shift fino, edição textual/teclado nativos | Previsibilidade, alvo amplo e precisão |
| Transformações comprimidas ou aparências empilhadas | Vetores XYZ em colunas, fallback para linhas em painel estreito; escalares alinhados | Não cortar números nem desperdiçar altura |
| Abas e divisores competem com a seleção | Acento concentrado em ferramenta/modo, divisores finos com área de interação preservada | Reduzir ruído mantendo affordances |

1.Frequência adequada: widgets em layout/eventos; nada em tick/NPC/física.
2.Sem clone de Scene/Engine/mesh. Miniaturas usam poucos pontos vetoriais por
pincel; widgets guardam cópia f32 e cache temporário (f64,último f32 publicado).
Strings/Shapes pequenos de UI imediata continuam custos explícitos.
3.IDs por região preservados. A viewport física exclui a faixa64pontos antes do
picking/projeção; HiDPI continua convertido pelo caminho compartilhado.
4.Componentes numéricos reutilizados em raio/forças, vetores/escala e bias.
O wrapper corrige só seu draft, sem alterar entrada global. Resposta changed é
reconciliada com o valor real para impedir edição por movimento vertical.
5.Três pincéis existentes em ScrollArea; mesmos budgets documentais e128amostras.
Novo catálogo permanece dependente de kernels/contratos, sem placeholders inertes.
6.Mesmo contexto/textura/GPU, sem shader/feature/readback por interação. Capturas
ocorrem apenas nos probes explicitamente executados.
7.Sem dependência/fonte/thread/lock novos; comando de configuração compartilhado.
8.Cache numérico rebasa ao detectar edição externa/undo; draft autoral, revisões,
seleção, foco, cancelamento de gesto e bloqueio de popup mantêm guardas existentes.
9.Testes dedicados cobrem alvo inteiro, X/Y/diagonal/Shift, precisão/texto/Enter e
edição externa. Probe usa compositor/GPU reais para alternar contexto/pincel,
arrastar raio, impedir gesto atravessado, iluminação e formulário de projeto em
quatro larguras. Primeiro probe encontrou formulário alto demais para seu gate;
compactação corrigiu o layout sem remover a asserção de ausência de edição pendente.
10.Layout egui, geometria de curvas simples, scans/preview síncronos mantidos.
Sem claim de FPS. Leitor de tela, teclado/foco físicos e aceite humano não são
inferidos de input sintético/offscreen.

Verificado: shell nativo, ferramentas, inspetor/contextos, transformações,
iluminação, hierarquia, status, larguras compactas e diálogo de projeto.
Mantido deliberadamente: MCP/kernel/histórico, projetos e navegador de pastas,
formatos Scene81/replay85/layout3/manifesto1 e identidade dos documentos.
O resultado requer avaliação humana; expansão de pincéis é o próximo incremento,
com desenho de kernels e MCP, além da apresentação visual do catálogo.

## 29/10 — edição direta e descoberta de ferramentas, decisão anterior

Feedback: Aplicar interrompe o fluxo, capacidades MCP não têm caminho humano e
barra de título desperdiça altura. Adotar publicação automática de propriedades,
um undo por gesto/campo e nome de grupo compartilhado pelo dispatcher/MCP;
edits continuam validados atomicamente e pausam/reconstroem runtime. Grupos só
coalescem updates consecutivos na mesma revisão; edição externa/undo/reset
quebra o grupo. Nenhum estado de gesto entra em Scene/Replay.

No macOS, conteúdo ocupa título transparente com botões nativos e menu próprio;
drag/duplo clique preservam gestos de janela. Contexto da criação deve oferecer
NPCs e edição dos componentes reais, sem editar JSON como fluxo principal.
Malhas precisam de modo de componentes e visualização estrutural; rig deve
expor ossos/pesos e animação com o mesmo EditMesh. Operações avançadas são
agrupadas por tarefa e mostram pré-condições/efeito, sem alegar paridade só por
listar nomes MCP. Cada incremento deve registrar o que falta.

Checklist anterior:1.publicar apenas alterações de controles, nunca frames idle;
2.COW de Scene e histórico limitado32, coalescer sem reter32cópias por arraste;
3.grupo de sessão≠ID autoral;4.dispatcher compartilhado, formulários reutilizados;
5.limites de domínio mantidos;6.estrutura/wireframe como overlay, sem mudar shader
padrão;7.sem dependência/worker;8.erro atômico/revisão externa/foco/undo;
9.testes UI+MCP/serde/histórico e regressões headless/nativo separado;
10.reconstrução síncrona de runtime por alteração permanece custo explícito.

Fluxos/limites e revisão de performance deste incremento em
[editor-workflows.md](editor-workflows.md). A cobertura de controles existente não
é declarada equivalente à UX inteira do Blender; o catálogo compartilha tipos,
enquanto os fluxos diretos de personagem/geometria/esqueleto são ferramentas próprias.

## Modelagem por componentes

EAF-130 acrescenta Extrudar/Inset/Bevel contextuais no inspetor de Editar e prévia
com arraste/confirmar/cancelar. Contagens legíveis, abas de componentes sem quebrar
nomes em painéis estreitos e mesmos scrub/flags de oclusão/raio X. Contratos,
limites e gates em [editor-modeling.md](editor-modeling.md).


### Correção do painel de iluminação (2026-11-05)

A janela tem estado próprio, sem disputar o único slot de pop-up da memória
do egui com o color picker. Controles da cena publicam SetLight e uma única
mudança de prévia quando necessário; a luz de trabalho não mascara o resultado.
Controles de trabalho desabilitados não publicam mudanças por clamp de drafts
locais. Abrir o painel não altera a cena nem gera revisões continuamente.

Regressões em `editor_lighting_ui_tests.rs` usam eventos reais de ponteiro:
slider→comando→frame→pixels GPU, edição RGB com ambos os painéis mantidos,
fechamento apenas do picker ao clicar fora, luz de trabalho separada da cena
e ausência de edições em idle com luz vertical. Sem mudança de shader ou formato.
