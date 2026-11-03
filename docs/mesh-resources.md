# Meshes dimensionados pelo conteúdo — EAF-172

O catálogo visual não tem mais os tetos de laboratório de 16 assets, 32 instâncias,
32.768 vértices/65.536 triângulos agregados nem 131.072 triângulos desenhados.
A geometria também deixa de exigir o antigo teto por asset de 16.384 vértices e
32.768 triângulos. A quantidade é definida pelo autor. Isso não torna o trabalho
infinito nem garante FPS: armazenamento, formato de índices e recursos do
hardware continuam finitos, e desenhar triângulos repetidos ainda tem custo.

## Representação e execução

- Hierarquia, posições, normais e scratch de escultura crescem com os dados.
  A travessia de pais é iterativa; ciclos e referências inválidas continuam
  recusados antes de publicar uma edição. Skeletons só são alocados para rigs.
- Assets estáticos repetidos compartilham um stream local na GPU. Cada instância
  acrescenta 80 bytes (matriz e cor), sem replicar os vértices no frame.
  Os buffers desse caminho são paginados conforme o limite do dispositivo.
  Instâncias consecutivas do mesmo asset/nível são desenhadas juntas.
- A ordem autoral é preservada entre geometria compartilhada, geometria expandida
  e representações atmosféricas; não reordenamos objetos por material.
- A tabela de materiais usa storage buffer dimensionado pelo catálogo, em vez
  de um uniform com 17 entradas. O limite real do binding é consultado no device;
  excedê-lo produz diagnóstico. Paginação de tabelas de materiais é trabalho futuro.
- O seletor atmosférico escolhe representação por instância; a geometria escolhida
  é compartilhada por asset/nível. Sombras, rigs e attachments mantêm os critérios
  conservadores existentes. Outline usa a geometria transformada do selecionado.
- Geometria de uma única instância, rigs e attachments mantêm o caminho expandido.
  Esse caminho verifica tamanho de buffer/draw contra o device. Não há descarte
  silencioso de avatares por ultrapassar 131.072 triângulos.

`inspect_meshes.limits` expõe `null` nos antigos limites de quantidade; limites
representacionais u32 e a base de recursos ficam explícitos. Capturas expõem
`mesh_resident_vertex_count` além da quantidade lógica submetida; `adapter.instancing`
expõe instâncias, bytes de geometria/transformações, draws e uploads cumulativos.
Os bytes não representam toda a RAM/VRAM: documento, caches, materiais, imagens,
editor e driver são recursos adicionais.

Não há uma opção exclusivamente visual: edição, carga, inspeção e captura passam
pelos mesmos documentos/dispatcher usados pelo MCP e pelo editor. Nenhum novo
slider de quantidade é necessário para autorizar objetos além de 32.

## Compatibilidade e limites separados

Scene103/replay113/conteúdo5/protocolo5. O índice de asset da pose replicada passa
u8→u32 (varint no fio), evitando truncamento após asset255. Protocolo antigo é
recusado no handshake; leitores não devem interpretar bytes de uma versão como
outra. Scene102 e anteriores não podem declarar os novos tamanhos; cenas antigas
válidas continuam migrando. Checkpoint2 e PSV1 mantêm suas próprias validações.

Continuam separados: envelopes de arquivo/replay/rede/save, quotas de operações
procedurais/modelagem, ossos/clipes/chaves, capacidade de entidades físicas/NPCs,
limites de coordenadas e distância. Esta entrega remove os tetos do catálogo e
render de meshes; não declara removidos esses outros contratos. O compilador ATM
usa índices u16; fontes que não cabem conservam a geometria completa, sem truncar
índices para forçar uma simplificação. Consultas de visibilidade de componentes
podem expandir geometria na CPU e ainda precisam de uma estrutura por instância
para cenas muito grandes; não participam do desenho normal de cada frame.

A próxima medição de floresta deve usar a árvore real, variantes e distâncias,
com câmera em movimento e métricas de frame. Instancing reduz cópias/preparação;
a redução do trabalho geométrico visível continua dependendo da seleção de
representações. Não foi implementado culling novo nesta entrega.

## Validação reproduzível

As provas críticas em `tests/mesh_resources.rs` cobrem identidade/persistência/
rollback acima dos limites antigos e comparação nativa de pixels com o caminho
expandido (materiais, sombras, atmosfera, seleção e sobreposição coplanar).
O caso CPU mantém 10.000 instâncias de uma malha com 33.800 triângulos: 338 milhões
lógicos, mas apenas 101.400 vértices residentes no frame. Esse teste **não** mede a
GPU desenhando 338 milhões de triângulos.

`cargo run --release --example mesh_instance_cost` compara 1.000 e 10.000 instâncias,
128 triângulos cada, a 1728×1117. Faz 12 aquecimentos e 32 amostras de draw+espera da
GPU. PNG, apresentação, simulação e compilação não entram no tempo; portanto não
é FPS de gameplay nem timestamp isolado de GPU. Relatórios/capturas atuais ficam
em `artifacts/mesh-resources/`, incluindo `cost.json` e `comparison.json`.

## Revisão estática de custo

A cópia de geometria por repetição foi eliminada no caminho estático compartilhado.
Os streams são binários tipados; JSON fica na autoria/MCP, não no upload GPU por
frame. Matriz/cor usam uma estrutura de 80 bytes, e os vértices locais preservam o
layout/material existente. Hierarquia e scratch são preparados conforme o conteúdo;
frames estáticos preservam o cache. Instâncias usam dois vetores reutilizáveis e
só voltam à GPU quando mudam; batches/commands conservam capacidade. O conjunto de
assets usados ainda é construído por frame. A tabela de materiais só é convertida
quando muda; níveis compilados usados anteriormente permanecem no cache e entram
na contabilização de bytes alocados. Não foram adicionados locks, I/O ou geração
de assets ao desenho. Câmera/luz não recompilam pacotes.

O desenho completo continua processando triângulos por instância, com transformação
no vertex shader. Assim, uma carga pequena pode ficar mais lenta mesmo economizando
memória; o resultado pesado não é uma garantia universal. Um futuro seletor de custo
precisa medir essa troca sem reintroduzir um teto autoral de objetos.

## Resultado local em 31/10/2026

Apple M4 Pro/Metal, macOS arm64, build `efde0093aaf3059b`. Mesma geometria, cores e tabela de materiais nos dois caminhos; a referência concatena os frames legados de uma instância, sem criar um catálogo artificialmente maior. Três rodadas com ordem alternada; mediana das medianas:

| Instâncias | Expandido | Compartilhado | Redução do tempo | Dados GPU de geometria + instâncias |
|---:|---:|---:|---:|---:|
| 1,000 | 1.845 ms | 1.322 ms | 28.3% | 23.438 → 0.100 MiB |
| 10,000 | 12.151 ms | 6.526 ms | 46.3% | 234.375 → 0.786 MiB |

São 128 triângulos por instância, até 1,28 milhão submetido. Todos os pares comparados tiveram diferença RGB8 zero. Nas rodadas finais, a carga de 10 mil variou 12,141–12,263 ms no expandido e 6,518–6,637 ms no compartilhado. A carga de mil apresentou mais variação (1,812–3,283 contra 1,320–2,612 ms); uma rodada exploratória anterior chegou a 1,764 contra 2,604 ms, uma regressão que não se repetiu nas três rodadas finais. Não atribuímos causalidade sem isolamento, nem prometemos ganho em toda carga. A memória da tabela exclui documento/caches CPU, materiais, targets e driver.

218 testes Rust em 14 alvos de integração + lib aprovados, incluindo cinco comparações GPU; Clippy all-targets com -D warnings, fmt, build release e Python aprovados. Quatro roteiros MCP/15 grupos cobrem meshes, armazenamento, attachments e admissão/save/load/render de 10 mil instâncias. Os testes antigos que exigiam rejeitar quantidades agora permitidas foram atualizados; validadores de índice/estado e gates de versões antigas continuam cobertos.

O build corrente fica em `target/release/ignis`, utilizado pelo launcher instalado. Nenhuma janela foi reiniciada ou cena do usuário substituída. A cena de população do ensaio fica somente em `artifacts/mesh-resources/population.scene.json`. Isso conclui a infraestrutura de quantidade/compartilhamento, não a avaliação de uma floresta real ou de um mundo com streaming.
