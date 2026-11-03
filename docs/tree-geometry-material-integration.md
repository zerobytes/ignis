# Integração de continuidade e materiais — EAF-80/EAF-85

## Revisão anterior à integração

Pedido direto: incorporar a continuidade geométrica de c1fbf6a aos materiais de
435742d e mostrar a mesma árvore. Checkout codex/native-meshes limpo em435742d.
Escolha: importar o kernel/query, testes e receita explícita, depois executar
sobre a cena central60/64 com reflectance. Não copiar a candidata59/63 da outra
frente por cima dos materiais. Sem integração de main/geradores funcionais ou
novo número de formato; limites de compatibilidade de dielectric-response.md
continuam válidos. UI adiada, captura GPU sem janela para mostrar o resultado.

1. Frequência: campo de transição somente na consulta/autoria. Tick/skinning,
   materiais e renderer existentes mantidos.
2. Alocações: scratch limitado do kernel/importação existente; script monta
   apenas a primeira porção de wood. Outros payloads não são reconstruídos.
3. Tipos: f64 no campo e guardas, saída f32, três comprimentos validados antes
   da projeção. Sem dependência/unsafe/ABI/pipeline novo.
4. Trabalho repetido: preservar caminho zero/legado exato; até três combinações
   extras por avaliação habilitada do campo. Nenhum scan novo por frame.
5. Limites: +128v/+256t, desenho previsto65050/65536,486t livres; manter
   assets/instâncias/ossos/chaves e preflight/erro atômico existentes.
6. GPU/I/O: mesmos Vertex52/tabela19312/Globals176. Capturas/arquivos explícitos,
   caminhos correntes e apenas um build local ativo nesta worktree.
7. Sincronização: query pura, publicação pelo dispatcher/staging/undo existente;
   nenhum worker/lock novo, nenhuma janela/demo compartilhada controlada.
8. Invalidação/preservação: só geometria local de wood muda. Conferir todos os
   materiais (inclusive reflectance/detail), source/reference/plantas, demais
   assets, instâncias/attachments e poses0/60. Preservar domínio material nas
   regiões comuns; pescoço novo tem domínio explícito, sem transferência inventada.
9. Correção: Rust/Clippy/fmt/build e regressão MCP completa no build integrado;
   cenas antes/depois com câmera/luz/tick iguais, pure preview, orientação das
   faces, topologia, undo/redo/save/load/replay/prefixo e loop48frames/4s.
10. Custos/limites: correção local com domínio restrito, resolução finita e
    prova amostrada de curvatura; não árvore inteira soldada/sem autointerseções,
    não realismo ou FPS concluídos. Registrar medidas/inspeção posteriores.

## Revisão de integração por leitura

Importado c1fbf6a como ae8fb78. Conflito somente no registro append-only de
validation.md; mantidas evidências locais de EAF-94 e seção nova de continuidade,
sem trazer candidata59, renderer ou esquema de outra frente. O kernel opt-in
mantém zero/legado, deriva restrições de suporte antes da projeção e limita
crescimento pelos budgets existentes. Funções de receita usam derivadas e taper
explícitos; isso não entrega um sweepXYZ geral com preservação autoral.

O ensaio recebido reaplicava câmera após avançar60ticks: replace reseta execução
mesmo para o mesmo valor. Corrigido capture para editar câmera ANTES de step,
registrar comparação emtick60 e comparar prefixo0 separadamente. A igualdade do
replay antigo no tick0 era válida, mas não provava a captura animada anunciada.
Não houve mudança na semântica de câmera/engine para acomodar o teste.

O finalizador de retesselação tinha contagem fixa64794 presa ao fork anterior.
Agora usa orçamento dos componentes conservados e fixture independente do ramo
(960t emY4/1408t emY6), além do limite65536. Nenhum orçamento foi aumentado ou
assertion de pixels removido. Acrescentado prefixo0 versus captura de repouso.
Orquestrador demo_tree_integrated.py separa processos/roots dos ensaios para
não herdar acidentalmente o ROOT de verify_lab entre módulos importados.

## Resultado integrado e revisão posterior

Build própriofda9972793711701, macOS/Metal,60/64 locais mantidos.771Rust/70grupos,
Clippy-Dwarnings/build/fmt/diff e72MCP/297grupos headless aprovados;229build_id dos
JSONs do root mcp-validation conferidos sem divergência. Os três relatórios do
ensaio integrado, continuidade e finalizador também concordam nesse build.

1. Frequência confirmada: campo só na query/autoria, nenhuma fase nova por tick.
   Preparação/desenho existentes processam a geometria maior descrita abaixo;
   material e deformação existentes mantidos.
2. Apenas primeira montagem de wood substituída (1967v3930t). Geometria total
   +128v/+256t; arrays brutos de posições+coordenadas+índices somam+6144B, além
   de capacidades/metadados. Cópias JSON/poses são explícitas e limitadas.
3. Campo f64/saída f32; controles finitos/guardas e normal radial validados nos
   testes importados. Layouts Vertex52/tabela19312/Globals176 não mudaram.
4. Três blends extras só no campo habilitado, zero mantém legado. Kernel não
   percorre a árvore nem materiais. Os ensaios recompõem apenas região admitida.
5. Confirmados10assets/32instâncias,11300v16506t armazenados,65050t desenho,
   486t livres. Madeira3808v<4096. Nenhum teto alterado.
6. Expansão de desenho cresce768v×52B=39936B ativos por representação expandida;
   mesma faixa de capacidade por potência de2. Sombra/cache/captura existentes.
   Fonte6500223B, igual ao saveMCP; sem cópia de build/versionamento de binário.
7. Sem threads/locks/fases novas. Scripts usam processos/roots próprios e não
   controlam sessão visual compartilhada.
8. Todos os9materiais com reflectance/detail, demais assets, plantas/fonte/
   reference, instâncias/attachments e poses0/60 exatos. Demais campos da cena
   exatos. Prefixo comum de domínio e restante de wood conservados; região
   nova autorada explicitamente. Preview/undo/redo/save/load/replay/reset testados.
9. Mesma câmera/luz/tick60: contorno do ramo46.1047°→10.6236°, guarda interna
   31.6454°→22.7334°. Geometria comum, colares e orientação das faces verificadas.
   Até as capturas ANTES no build integrado são byte-exatas às capturas após
   material435742d (whole/bark/fork).12casos de planta/retesselação preservam o
   fork corrigido; pixels vivo/prefixo/rest/reset/viewport exatos. Loop48frames
   distintos/4s, GIF2489726B. Closes/árvore inteira inspecionados sem retoque.
10. Mantidos: pior guarda superior28.7064°, facetas no pescoço superior,
    inserções sobrepostas restantes e casca de domínio não transportado. Maior
    densidade não certifica ausência global de autointerseção; r×κ máximo
    0.734084 em1001amostras é diagnóstico. Não alegar realismo ou FPS. A árvore
    continua benchmark de ferramentas gerais, sem subsistema botânico no núcleo.

Reprodução (usar a entrada anterior à correção, pois o prefixo é verificado):

```sh
/Users/zero/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3 scripts/demo_tree_integrated.py --frames 48 --input artifacts/tree-integrated/examples/input.scene.json
```

Orquestra continuidade→finalizador→verificação do conteúdo final. Relatório
artifacts/tree-integrated/artifacts/report.json, comparação
artifacts/tree-continuity/artifacts/before/after-{whole,bark,fork}.png e GIF
artifacts/tree-study/artifacts/tree.gif. Source central examples/native-tree-study.scene.json
é o saveMCP integrado, não a candidata antiga59 da outra worktree. Sem merge
main/instalação/janela/push; EAF-80/EAF-85 e aceite artístico continuam abertos.
