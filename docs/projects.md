# Projetos e cenas no editor

Direção: criar um jogo deve começar pelo projeto, depois cenas e objetos.
Área de trabalho significa distribuição de painéis, não projeto.

## Escolher a pasta pela interface

Criar/Abrir projeto usam um navegador integrado de pastas. A coluna Locais tem
Início, Mesa, Documentos, Downloads (quando existem) e, no macOS, Discos e volumes.
Clique em uma subpasta para entrar, em Subir para voltar ao nível anterior, ou
nos níveis do caminho para saltar diretamente. O endereço completo fica visível
como confirmação; digitação manual fica recolhida e opcional.

Busca somente nas subpastas do local atual, acionada por Buscar/Enter. Ocultos
permite acessar diretórios iniciados por ponto. Páginas ordenadas de256pastas
mantêm todas alcançáveis; arquivos não consomem a página. Atualizar reenumera o
local. Criar mostra a nova pasta com o nome do jogo dentro do local escolhido;
Abrir só habilita quando encontra ignis.project.json. Falha de leitura ou caminho
manual ainda não confirmado desabilitam criar/abrir. A navegação não cria arquivos
nem altera a cena; somente a confirmação envia o comando de projeto existente.

O seletor é integrado ao editor, sem nova dependência de diálogo do sistema.
Enumeração/metadados síncronos apenas em ações, com seleção ordenada limitada
em memória a257nomes, histórico de cursores por página visitada. Sem scan no
redraw, novo comando de engine ou alteração dos formatos persistidos.

## Contrato anterior à implementação

Pasta + `ignis.project.json` versão1, nome, cena inicial e lista explícita de
até256 cenas. Cenas usam o formato atual da engine; sem empacotar assets,
layout ou estado da simulação no manifesto. Uma cena ativa por sessão. Layout independente passa à versão2 (painel Projeto),
com carga de layouts1 sem esse painel e migração ao formato corrente.
Alternativa monolítica descartada para não duplicar persistência nem impedir
edição individual/versionamento das cenas. Lista explícita evita scan recursivo
por frame e inclui somente documentos criados/registrados no projeto.

CreateProject/OpenProject são operações explícitas de troca da raiz, com pasta
absoluta. Todos os demais caminhos continuam relativos e confinados à raiz.
Criar recusa destino existente, prepara/valida cena e manifesto em memória, reserva a nova pasta com create_dir
e publica arquivos temporários sem substituir existentes; erros retiram somente
os outputs criados por essa operação. Abrir valida manifesto,
referências e cena inicial antes de substituir a sessão. Cenas novas são
publicadas sem sobrescrever; se o manifesto não puder ser atualizado, a nova
cena criada por essa operação é retirada. Escritas de cena continuam atômicas.

InspectProject expõe projeto/cenas/cena ativa/alterações. NewProjectScene e
OpenProjectScene recusam alterações não salvas salvo `discard_changes:true`.
CreateProject/OpenProject fazem a mesma guarda. LoadScene legado mantém sua
semântica MCP explícita; UI usa os novos comandos guardados. SaveScene limpa
somente a marca de revisão salva; undo de volta à revisão anterior pode continuar
marcado como alterado (conservador, não compara geometria por frame).

Projeto novo começa com piso e jogador, sem obstáculos/NPCs pré-prontos.
Objetos básicos visual/fixo/dinâmico usam Upsert/validação/undo existentes.
Duplicação preserva o documento do objeto com ID novo; exclusão usa Remove.
Colisão/percepção continuam contratos próprios. Isso inicia autoria/teste; não
entrega exportação/distribuição, scripting geral, múltiplas cenas simultâneas,
modelagem completa ou conclusão de um jogo.

## Revisão estática anterior

1. I/O/criação/enumeração apenas em ações; nenhum scan por tick/frame.
2. Manifesto/lista até256, browser de pastas até256 entradas; cenas/Engine somente
   ao abrir/criar, sem clone de Scene na montagem da UI.
3. Caminhos validados e IDs de objetos preservados; layout não entra no manifesto.
4. Mesmo dispatcher/renderer/histórico; helper de cena inicial reutiliza validação.
5. Limites de8MiB e limites de entidades/recursos mantidos, sem cópias de builds.
6. Sem readback rotineiro; GPU/captura só nos fluxos já existentes.
7. I/O síncrono somente sob pedido, sem dependência/thread nova.
8. Validar antes da troca, guardas de revisão/alterações e arquivos sem overwrite.
9. Testar troca/erro/dados não salvos/confinamento/symlinks/colisão/persistência,
   cena independente, UI e regressão headless completa.
10. Custos mantidos: parse/Engine síncronos ao abrir e marca conservadora de dirty;
    sem promessa de autosave, jogo exportado, FPS ou aceite humano por teste.

## Revisão posterior e validação

Manifesto1 e layout2 independentes; Scene75/replay79 preservados. Troca de cena
instala Engine validado e revisão monotônica, evitando histórico cruzado e ações
MCP/UI obsoletas. saved_scene_revision marca autoria, não ticks/input/layout;
session_revision invalida apresentação ao salvar ou alterar metadados sem mundo
novo. Câmera de inspeção volta à cena ao trocar documento. Assets/Scene não são
clonados por frame; read_dir somente em eventos do browser, até256 entradas.

CreateProject reserva pasta nova com create_dir, arquivos persist_noclobber;
NewProjectScene recusa colisões e desfaz arquivo próprio se manifesto falhar.
Resolver protege leitura/escrita, incluindo symlinks. Save/continue interrompe
o lote no primeiro erro. Duplicar jogador é desabilitado (um por cena), edição
respeita pausa e nenhum controle visual contorna o dispatcher. Custo mantido:
I/O/parse/Engine síncronos sob pedido; dirty conservador após undo. Sem autosave
ou transação de filesystem contra processos externos alterando a pasta nova.

Build5b74972fb0197436:1092testes Rust/104alvos release/all-targets, Clippy-Dwarnings,
fmt/diff/sintaxePython e95cenários MCP/392grupos headless aprovados. Cinco testes
de projeto cobrem persistência, guarda de dirty, rollback, colisão, symlink e
migração de layout. MCP cria/reabre em outro processo e confere cena/objetos/replay.

Verificação nativa com o mesmo executável: criar projeto via formulário, digitar
pasta persistente, adicionar bloco, criar Floresta usando Salvar e continuar,
reabrir Principal com bloco, executar/W/Esc/pausar, fechar com nova alteração e
salvar antes de encerrar. W levou Z de0a−1,65 na execução (tick98); autoria
permaneceu0. Capturas artifacts/editor-project-native.png e editor-project-close.png.
O seletor de apps escolhia a instância antiga: wrapper temporário apontando ao
mesmo binário permitiu isolar a interação; foi encerrado/removido após verificar.
Sem cópia de binário. A cena antiga do usuário permaneceu no processo12548;
a instância instalada atualizada17351 foi deixada aberta na entrada de projetos.
Aceite humano permanece separado; isto não conclui o restante do editor.

## Verificação do navegador estruturado

Quatro regressões Rust cobrem paginação ordenada depois de300arquivos/270pastas,
busca sem distinguir caixa, ocultos, erro sem confirmação obsoleta e navegação
por cliques (Locais/subpasta/Subir/breadcrumb). O quarto exige destino e botões
de confirmação dentro de1440×900, cobrindo a expansão de separadores encontrada
na primeira conferência nativa e corrigida antes da entrega.

No macOS, o navegador abriu Discos e volumes→Zero e Início→Ocultos. Busca e
cliques levaram até a pasta de evidências sem digitar endereço completo; criar
Teste de pastas publicou manifesto1 e Principal75 com somente floor/player,
conferidos também no disco. Evidências editor-folder-native.jpg e
editor-folder-created.jpg, projeto em artifacts/editor-native-projects/Teste de
pastas. Janela/launcher auxiliar isolados foram encerrados e removidos; sem
copiar binário ou tocar sessões anteriores. Este gate não substitui aceite humano.

Gates finais do build b540b542eab8bf06:1096testes Rust/104alvos release/all-targets,
Clippy-Dwarnings/fmt/diff e95cenários MCP/392grupos headless aprovados, sem falhas
ou testes ignorados. Mesmo fonte/hash conferido; manifesto1/layout2/Scene75/
replay79 mantidos. App instalado aponta ao único release corrente17.387.568bytes,
cache exclusivo~2,0GiB. Nova instância instalada aberta após o teste isolado;
sessões anteriores não controladas. Sem push, cópia de binário ou instalador.
Logs/auditoria editor-folder-* e relatório MCP corrente. Aceite humano separado.
