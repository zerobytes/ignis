# Oclusão GPU corrente — EAF-177

Incremento nativo após176; pesquisa82 permanece isolada. Objetivo: evitar
geometria completamente oculta sem modificar pixels. Não alterar LOD/material,
resolução, cena, simulação ou formatos. Usar oclusores opacos do frame atual,
inclusive sua representação selecionada, nunca caixas externas como sólidos.

Plano e revisão prévia: prepass separado de profundidade reversa, nas mesmas
coordenadas XY da câmera, evita a baixa precisão distante do depth convencional.
Redução conservadora por tiles16×16 (mínimo de reverse-Z, incluindo fundo zero);
bounds ampliados já calculados pelo frustum fornecem retângulo e profundidade
mais próxima. Manter objetos em dúvida, near plane e grandes footprints. A GPU
compacta instâncias de cada lote mantendo ordem e escreve argumentos indiretos;
sem readback para decidir, espera GPU→CPU ou retardo de um frame. Sombras seguem
caminho original. Prepass não altera depth final nem prioridades coplanares.

Oclusores escolhidos por área projetada/custo, com orçamento de trabalho, sem
semântica de assets. Ativação apenas para carga relevante; sem oclusores úteis ou
com buffers/páginas incompatíveis, fallback original. Nenhum teto autoral novo.
Dados: instância80B existente, bound32B, job16B, argumentos32B, buffers tipados e
texturasDepth32/R32. Reusar recursos entre frames; nenhum JSON, novo worker, lock
ou dependência no loop. Preparação, prepass, redução, compactação e indiretos
entram no custo total. Medir perdas em cenário aberto e preservação de ordem.

Gates: pixels exatos perto/longe, movimento, revelação, near plane, resize,
material/seleção/sombra; teste crítico de falsa oclusão e compactação estável.
Floresta30mil e vista aberta/inversa, referência igual sem oclusão. Promover
somente após comparar custos; o restante do programa (cobertura coletiva e LOD
independente da atmosfera) não é implicitamente entregue neste incremento.

## Contrato implementado

- Oclusores são triângulos opacos reais: meshes únicos usam os buffers expandidos
  já preparados; assets compartilhados usam sua transformação e nível atuais.
  Um AABB fornece somente a consulta conservadora, nunca uma superfície opaca.
- O vertex shader mantém as coordenadas de clipping originais. Reverse-Z é
  escrito no fragmento, portanto não introduz cobertura além do near/far plane.
  Uma guarda adicional conserva empates do depth convencional do passe principal.
- A compactação GPU mantém a ordem relativa de cada lote, inclusive quando há
  superfícies coplanares de cores diferentes. A referência não exige transparência
  de ordem independente nem relaxa as diferenças de RGB8.
- O descarte nesta etapa atua nas instâncias estáticas compartilhadas. Meshes
  únicos/deformados e primitivas continuam no desenho original; meshes únicos
  podem servir de oclusores. Sombras, seleção e simulação permanecem independentes
  da máscara da câmera. Não é descarte de assets residentes nem streaming.
- Buffers e texturas são reutilizados, mas têm custo e permanecem em cache quando
  a cena retorna ao fallback. `occlusion.image_bytes/buffer_bytes` contabilizam
  esses recursos; `instancing` continua descrevendo a seleção anterior à oclusão.
  `inspect_occlusion_gpu` é readback bloqueante explícito de benchmark, fora do
  loop e das medições; não é chamado pela janela nem pelas decisões do renderer.
- O seletor limita apenas o trabalho extra: área relevante, no máximo262144
  triângulos/128 oclusores e footprints de até9×9 tiles por consulta. Qualquer
  limite excedido mantém o desenho original. Esses orçamentos não limitam autoria.

Reprodução: `cargo test --release --test render_occlusion -- --ignored --nocapture`
e `cargo run --release --example visibility_cost -- --occlusion artifacts/landscape-forest-dense/forest.scene.json`.
O exemplo exige imagens repetíveis e equivalência exata antes de medir três
rodadas alternadas com aquecimento. Draw+espera inclui GPU e CPU; não equivale a
um timestamp GPU isolado nem ao FPS de gameplay.

## Resultado nativo — 31/10/2026

M4 Pro/Metal, 30 mil árvores, 2048×1536, atmosfera e níveis anteriores mantidos,
blur desligado. Mediana das três medianas de 16 amostras após8 de aquecimento:

| Câmera | Anterior | Com oclusão | Redução do tempo | Triângulos evitados/quadro |
|---|---:|---:|---:|---:|
| Próxima | 91,961ms | 71,080ms | 22,7% | 57.140.870 |
| Panorâmica | 117,781ms | 86,456ms | 26,6% | 99.135.786 |
| Inversa/leve | 5,104ms | 5,062ms | empate prático/fallback | 0 |

Os números incluem preparação, passe extra, consultas, compactação e desenho.
CPU de submissão mudou5,603→5,684ms e7,294→7,622ms; portanto não houve ganho de CPU.
A economia está no trabalho total de renderização. Não converter esta tabela
em FPS garantido da janela, nem em tempo GPU isolado.

Todos os canais RGB8 ficaram exatos nas três câmeras e nas oito posições de
movimento/resize. Teste crítico de11situações protege vãos, ordem coplanar,
objeto/câmera/oclusor em movimento, near plane, sombra, seleção, mundo transladado
8000m e os caminhos de mesh único/compartilhado. Três provasGPU anteriores de
mesh_resources também passaram. Captura otimizada inspecionada visualmente.
Evidências: `artifacts/occlusion/cost.json`, `motion.json`, `gates.log` e PNGs.

A oclusão usa mais memória:15,57MiB perto e19,09MiB no pico panorâmico para
texturas/buffers próprios. Fonte de instâncias80B permanece compartilhada;
`source_buffer_bytes` informa a referência retida e não deve ser somado duas
vezes ao buffer do caminho principal. `allocated_instance_bytes` distingue
capacidade alocada de bytes úteis. Recursos em cache continuam contabilizados
no fallback. Não há liberação da geometria autoral ou redução de memóriaCPU.

O padrão do renderer passa a habilitar o seletor automático; o setter Rust é
apenas oráculo de diagnóstico, como os de indexação/frustum. Os contadores
aparecem na inspeção existente de renderer/MCP. Nenhum novo controle autoral,
formato de cena, protocolo, dependência, thread, lock ou worker foi introduzido.
Os outros modos do benchmark desativam oclusão explicitamente para manter seus
comparativos isolados.

Ainda sobram93,8/153,0milhões de triângulos compartilhados desenhados nas duas
vistas pesadas. Oclusão não elimina o custo das copas que realmente aparecem.
O próximo gargalo é a representação coletiva do detalhe pequeno visível,
preservando cobertura/contornos; este incremento não entrega essa simplificação.
O critério de ativação é conservador e heurístico, não uma promessa de ganho em
toda cena/hardware. Cenas pesadas abertas sem ocultação podem pagar overhead;
novas medições devem ampliar essa matriz antes de afirmar desempenho universal.

Build final `d0e474bef699d037`,6testes Rust e Clippy all-targets aprovados.
Janela nativa2784×1560 reaberta na floresta30mil e inspecionada com oclusão ativa;
amostra de cadência11,89FPS, sem alegar comparação de FPS porque a câmera não
foi percorrida em uma trajetória A/B controlada. Aceite humano continua separado.
