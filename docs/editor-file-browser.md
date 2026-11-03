# Navegador de projetos e arquivos

## Decisão anterior — EAF-109, 29/10

Pedido: navegação inspirada no Finder, lista de projetos pronta e criar pastas.
Worktree própria codex/file-browser desde02b1c2d; editor original está em uso
por outra execução. Não integrar branches nem trocar sua instalação/sessão.

Barra lateral com Projetos/Favoritos/Locais, histórico voltar/avançar, caminho
clicável/editável, busca local, lista ordenável com tipos/metadados, seleção,
duplo clique/teclado e nova pasta. Criar/abrir separados e destino explícito;
proteção existente da cena não salva reutilizada.

Biblioteca limitada de recentes/fixados e favoritos fora de Scene/replay/manifesto.
Editor nativo carrega arquivo por usuário; serviço de teste/MCP inicia em memória
e pode configurar arquivo explícito pelo dispatcher. Descoberta só superficial,
sem indexador global ou scans recursivos. Kernels compartilhados para UI/MCP.

Alternativas: diálogo nativo isolado não resolve biblioteca/ações Ignis juntas;
indexador global e dependência de desktop não se justificam neste incremento.

## Checklist de performance anterior

1. I/O de diretórios/metadados apenas em ações/startup; nada por tick/frame.
2. Página256/keeper257, histórico64, biblioteca32 projetos/16 favoritos e JSON limitado.
3. Caminhos absolutos/canonizados no navegador, IDs autorais preservados; biblioteca fora do jogo.
4. Mesmo dispatcher/kernels e confirmação de cena suja; nenhum segundo sistema de projetos.
5. Raízes/descoberta/paginação limitadas, truncamento explícito.
6. Nenhum shader/GPU/readback novo; ícones vetoriais e metadados cacheados.
7. Sem worker/lock/dependência; I/O síncrono por gesto é custo explícito.
8. Nova pasta exclusiva, sem overwrite; erro de load não apaga biblioteca/cena.
9. Testes de colisão/erros/symlinks/páginas/persistência, entrada/UI estreita, Rust/Clippy/MCP.
10. Sem promessa de Finder completo, busca de conteúdo, sync, FPS ou aceite físico por captura.

Referência: https://support.apple.com/guide/mac-help/organize-your-files-in-the-finder-mchle9f0a1b2/mac

## Integração EAF-128

A orientação de29/10 para consolidar frentes estáveis na main substitui a restrição inicial de não integrar branches. O delta isolado foi recuperado e revisado na consolidação. Rótulos de linhas/cartões são apresentação sem seleção de texto: o evento de clique/duplo clique pertence à linha, com identidade estável pelo caminho. A regressão de clique simples/duplo/voltar/avançar e criação de pasta passa; o cenário project_browser cobre os9comandos em processo MCP, persistência em segundo processo e falhas atômicas. Preferências reais do usuário não são usadas nos testes. Resultados definitivos do conjunto em main-consolidation.md/validation.md; sem aceite humano presumido.

Remoção/desafixação usa a identidade canônica quando o caminho ainda existe; caminhos ausentes continuam removíveis pela grafia armazenada. Regressões cobrem aliases macOS/symlinks, favoritos e projeto movido, sem apagar arquivos.

Validação na união main: buildd092a8fec0ed01f0,1139testesRust/114alvos,Clippy/fmt/build,113MCP/482grupos e11grupos da casa aprovados. Esta evidência corresponde ao conjunto integrado Scene86/replay90, não a um binário separado da origem file-browser. Aceite humano permanece separado.
