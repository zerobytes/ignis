# Filtro de superfície por região — proposta e kernel

## Decisão e checklist antes do código

Falta observada no exercício da árvore: o pincel Smooth é radial e não possui
seleção/âncoras. SetShading suaviza normais; subdivisão linear não arredonda a
forma; nenhum une componentes desconectados. A correção de curvas de construção
e união de peças continua separada do acabamento local da superfície.

Primeiro recorte: função pura genérica de Laplaciano com vizinhos únicos,
seleção explícita de vértices, âncoras e envelope total. Preservar o Smooth
legado, que conta incidências de triângulos. Taubin/bilateral/preservação de
volume acrescentam políticas e podem vir após medir a necessidade; Laplaciano
limitado permite observar o encolhimento, sem prometer conservá-lo. Não ampliar
tetos nem introduzir remesh, solda, nearest-source ou novas dependências.

1. Somente autoria/query, nenhum trabalho por tick/frame.
2. CSR de arestas e incidência preparado uma vez, buffers reutilizados entre
passos; vetores limitados pelos tetos de geometria existentes.
3. Índices validados/canônicos, f64 temporário/f32 final; dados fixos exatos.
4. Passos finitos1..32, seleção reduz o trabalho repetido; validar faces e
arestas incidentes, inclusive faces com vizinhos fora da seleção.
5. Envelopes/opções finitos antes das cópias; recusar não-manifold local,
inversão/degeneração/dobra nova e piora máxima/média; não alterar guardas de
bridge/união. Cantos existentes intencionais podem permanecer fixos.
6. Nenhum shader/layout/readback/upload novo; publicação futura usa o existente.
7. Nenhum worker/lock/dependência/unsafe.
8. Asset de saída compartilha rig/clipes; só geometria é copiada. Sem cache novo.
9. Testar máscara/âncoras/bordas/duas ilhas/rig/canais/opções/erros/determinismo,
poses fixas e regiões incidentes; dispatcher/MCP/undo são integração posterior.
10. Custos mantidos: inspeção topológica e validação síncrona da autoria,
Laplaciano pode encolher. Volume só em componente fechado orientado. Sem prova
global de autointerseção, anatomia/veios físicos, FPS ou acabamento automático.

O kernel não terá referência de sessão nem editará planta/Scene. A integração
deve usar Reference(scope/revision), staging/preview/undo compartilhados e as
guardas de rebase já existentes. Não contornar assembly persistida por bake ou
detach: índices avaliados não são âncoras canônicas para regeneração posterior.

## API e domínio admitido

`mesh_surface::smooth_region(&Asset, vertices:&[u32], anchors:&[u32], &Options)`
retorna `Filtered {asset,report}` sem mutar a fonte. Options público tem
`iterations:u8`1..32, `strength:f32`(0,1] e `max_displacement:f32`1e-6..100m
locais. Tipos de entrada serde/JsonSchema estritos; IDs não podem repetir nem
exceder a malha, seleção não pode ser vazia, âncoras devem pertencer à seleção.
Ordenação da entrada não muda a saída. Nenhuma nova configuração global/teto.

Todo vértice fora da seleção é fixo. Dentro dela, âncoras e vértices de borda
real também ficam fixos; contagens de âncoras explícitas e de borda podem se
sobrepor. Se nada puder mover, retorno compartilha o Arc original sem fazer
iterações. Iterations no relatório é a configuração, não declaração de trabalho
executado nesse caso. Isolados selecionados móveis, estrelas desconectadas,
arestas não-manifold e winding inconsistente incidentes são recusados; o
operador não conserta topologia ou une duas ilhas próximas.

As médias usam arestas únicas e posição anterior congelada de cada passo.
Envelope é TOTAL em relação ao input, não por passo, com conferência depois de
quantizar f32 e tolerância absoluta1e-6m de quantização. Essa tolerância também
é custo do contrato em deslocamentos muito pequenos; o relatório mostra o
máximo efetivo. Faces incidentes não podem degenerar ou inverter em relação à
normal inicial. Aresta antes abaixo90° não pode alcançar90°; arestas já em90°
ou mais não podem piorar. Comparar máximo e média de TODAS as adjacências
incidentes no final, recusando piora acima1e-6grau, inclusive na fronteira com
faces sem vértices editados. Erro informa antes/depois da qualidade recusada.
Algumas seleções redistribuem curvatura e pioram a média; serão recusadas mesmo
quando parte da região pareça melhor. Isso é guarda conservadora, não garantia
de que qualquer seleção tenha um resultado admitido.

Report público descreve seleção/âncoras/móveis/movidos, faces/arestas afetadas,
máximo/RMS do deslocamento e bends_before/after. RMS inclui todos os móveis,
inclusive os que permaneceram na mesma posição. `fold_edges` conta ângulos>=90°,
sem provar interseção global ou que um canto autorado seja defeituoso. Volume
assinado antes/depois vem somente para componentes tocados com arestas fechadas
e winding coerente; aberto ou incoerente retorna null. Origem local por
componente reduz cancelamento numérico. Nenhum volume é garantido conservado.

Somente posições móveis são escritas no clone final. Triângulos, coordenadas
materiais, pesos, ossos, clipes, aparência e dados fixos são preservados. Manter
o rig não mantém as posições dos vértices deliberadamente editados nem promete
fitting/anatomia ou orientação física do material. Poses fixas e descendente
preso a osso são verificados em0/60ticks.

## Fronteira da integração

Esta entrega exporta apenas o kernel e tipos. Não adiciona MeshEdit/Command,
MCP/UI/Scene/plantas nem nova persistência. A frente que integra loft/contexto
instalará preview e publicação com as guardas existentes. Para composição,
ordem e seleção por proveniência precisam ser explícitas: IDs canônicos não
selecionam automaticamente pontos novos do loft, e IDs de sessão não devem
virar receita persistida. O kernel não permite ignorar a guarda de rebase da
assembly. A correção efetiva da árvore e o aceite visual ainda não foram feitos
por esta unidade.

## Evidência do kernel

794testes Rust/73grupos release/all-targets, Clippy-Dwarnings e fmt/diff passaram
em macOS Apple Silicon.11testes novos em tests/mesh_surface.rs: pico conhecido,
bordas/âncoras/no-op compartilhado, ordem de seleção, envelope em32passos,
ilhas próximas desconectadas, adjacências com endpoints não selecionados,
volume fechado encolhendo,12classes de seleção/opções inválidas, winding/
não-manifold de aresta e vértice, colapso/inversão/piora de pico recusados,
rig/canais/fixos bit-exatos (inclusive-0) e poses0/60 do filho preso a osso.
O volume é algébrico assinado, não volume de união espacial de peças sobrepostas.

Logs estáveis artifacts/mesh-surface/{tests,all-tests,clippy}.log. Nenhuma prova
MCP/undo/persistência/render nativo da nova operação foi executada aqui: a
função ainda não participa do dispatcher. Não houve alteração da árvore salva,
Scene/replay, tetos, outras branches, main ou demo. Integração coordenada deve
completar esses gates antes de tratar o filtro como ferramenta MCP entregue.
