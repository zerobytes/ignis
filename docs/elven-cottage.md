# Casa élfica — autoria nativa

A [primeira revisão estrutural](cottage-structure.md) é o estado corrente após
este ensaio inicial: receita clientev2 e molduras nativas por sweep. Os números
e escolhas abaixo registram a primeira entrega ea8c6b8, preservada como baseline.



Estado corrente: [cornijas e coberturas](cottage-roof.md), receita clientev3, após

Estado corrente: [ornamentação em relevo](cottage-relief.md), receita clientev4, após

Estado corrente: [pedra e piso](cottage-stone.md), receita clientev5, após
a [ornamentação em relevo](cottage-relief.md),
a revisão de [cornijas e coberturas](cottage-roof.md) e
a [primeira revisão estrutural](cottage-structure.md). Os números e escolhas
abaixo registram a primeira entrega ea8c6b8, preservada como baseline.

A [revisão arquitetônica independente](elven-cottage-review.md), posterior ao
retorno de Guilherme, identifica diferenças estruturais além do acabamento
descrito neste primeiro registro. Cena e receita foram preservadas na revisão.

## Direção e análise antes da geometria

Referência: imagem original de1536×1024 fornecida por Guilherme. A forma observada
tem varanda vazada à esquerda (três vãos frontais maiores e mureta rendilhada),
entrada recuada com porta ogival, escada frontal, bay window poligonal à direita,
telhado principal curvo com cumeeira longitudinal, cobertura menor sobre a bay,
chaminé deslocada e ornamentação vegetal em pedra/madeira/metal. As curvas do
telhado, o recuo da porta e os espaços negativos são critérios principais;
vegetação não deve esconder defeitos da estrutura. Contagens e proporções são
interpretações desta perspectiva, não levantamento métrico.

Costas, planta interna, escala e conexões ocultas são inferidas. O estudo constrói
uma fachada tridimensional com coberturas e paredes laterais/posteriores; não
promete interior habitável ou reconstrução exata de superfícies invisíveis.

## Escolha de autoria

Usar o build corrente de codex/native-plants, sem modificar engine/limites.
Planta nativa por seções para a coluna reutilizada; meshes explícitos para
paredes, coberturas, telhas, arcos, motivos ramificados, porta e jardim.
`scripts/demo_elven_cottage.py` é o cliente autoral: calcula a geometria das
peças, publica pelo dispatcher MCP e salva o documento pela engine. Não é um
comando automático de interpretação de imagem nem um novo kernel CAD na engine.
A receita editável está em `examples/elven-cottage.recipe.json`.
Ela parametriza este estudo específico; parte da composição e das medidas das
paredes/props continua autorada no cliente. Não é um sistema CAD de restrições
que garante reconstrução válida para dimensões arbitrárias.

Alternativas: uma malha única reduziria slots mas perderia materiais e seleção
por família; uma instância por telha/folha excederia32instâncias. Agrupar por
família/material e reutilizar coluna mantém16assets/32instâncias e os tetos
geométricos existentes. As curvas são amostradas explicitamente. Telhas têm
espessura e sobreposição; ornamentos têm relevo geométrico. Materiais procedurais
existentes acrescentam variação/bump, não substituem a forma.

## Checklist estático anterior

1. Geração apenas na autoria, sem trabalho por tick novo.
2. Buffers Python por família, publicação em lotes limitados pelo transporte;
   geometria não é retransmitida por instância de coluna.
3. Índices inteiros e metros, seed e IDs determinísticos; f64 no cliente/f32 nativo.
4. Superfícies e motivos reutilizam funções; constantes de telhado fora dos loops.
5. Preflight por asset e total antes de publicar; sem aumentar caps.
6. Captura offscreen explícita; um processo próprio, sem tocar a demo.
7. Transporte sequencial e transações existentes, sem workers/locks novos.
8. Save/load/undo/replay e capturas repetidas verificam invalidação existente.
9. Fonte da árvore preservada; cena independente, sem NPC/colisão novos implícitos.
10. Mantidos: cálculo autoral de meshes no cliente, arrays/JSON e validação
    síncrona. Slots agrupados limitam seleção por peça; sem alegar FPS/PBR completo.

## Reprodução

`python3 scripts/demo_elven_cottage.py` usa o binário release corrente. Saídas
correntes em `artifacts/elven-cottage/`; cena salva, relatório de contagens,
vistas da casa e replay. A execução não abre janela.

Cena publicada em `examples/elven-cottage.scene.json`, com o mesmo valor JSON
da saída nativa, compactado para armazenamento. Pode ser aberta por `load_scene`
no mesmo build/formato local72. As cinco instâncias da coluna usam a mesma
planta, com seus seis IDs de seção. Demais famílias são meshes explícitos,
editáveis pelos comandos existentes. `report.json` mapeia oito grupos visuais
para intervalos de vértices/triângulos; esses intervalos descrevem esta geração,
não são IDs persistentes após uma edição de topologia.

## Resultado e limites visuais

15assets,19instâncias,32172vértices/53376triângulos armazenados e61312triângulos
desenhados. Restam596vértices no total do documento. Capacidade individual e
draw não foram ampliados. Coluna nativa994v/1984t; curvas pequenas usam4lados
e até8amostras por Bézier, molduras maiores6lados e14amostras; o shader smooth
interpola normais, não muda a silhueta. Blocos compartilham8vértices com shading
flat. As seções multiplicam o número de intervalos da coluna:12segmentos por
intervalo bastaram para este tamanho de tela;24 consumiam orçamento em detalhe
sem vantagem visível neste enquadramento.

A revisão em cinco vistas corrigiu o encontro visual dos beirais, as curvas dos
painéis/porta e a visibilidade da chaminé. A varanda é vazada, com parede de
fundo; a porta está recuada e os degraus alcançam o patamar. Telhas sobrepostas
e colares de cornija são peças modulares, não uma superfície soldada única.

O modelo é reconhecível como interpretação da referência, mas seu acabamento é
simplificado: motivos mais esparsos, folhas/jardim menores, pedra menos variada,
vidros opacos com resposta especular e câmara diferente da original. Costas
inferidas são simples e não há interior habitável, colisão arquitetônica ou
iluminação pontual da lanterna. Não declarar reconstrução fiel, PBR completo,
geração automática por imagem ou aceite artístico.

## Verificação e revisão final

Onze grupos locais: preview de planta puro, preview do lote igual à publicação,
mesh malformado rejeitado atomicamente, edição da coluna com undo/redo, repetição,
imagem estática em60ticks, replay, prefixo, save/load, reset e viewport800×600.
Todos passaram; capturas equivalentes1536×1024 retornam bytes exatos. Duas
execuções novas produziram a mesma cena e o mesmo PNG (determinism.json).
O arquivo compactado de examples também foi carregado pelo MCP: estado igual
ao exportado e captura idêntica ao hero (published-check.json).
Seis cenários MCP/27grupos passaram: render,meshes,plants,plant_sections,
directional_shadows,materials. Binário ec96a3441fef43e2 existente, sem novo build
ou mudança Rust/shader; suites Rust completas não foram repetidas nesta unidade
de conteúdo. Python compile, fmt e diff check aprovados. Árvore canônica SHA256
89fe1fa2112cbb8a2d3ae266a53cf0d6308c8b371d356a8f5b1b05a208789400 preservada.

Checklist final:1.adequado,build() apenas na autoria;2.corrigido,box usa8pontos e
curvas finitas/coluna instanciada;3.adequado,índices/seed/IDs e publicação f32;
4.adequado,arch_top usa30bisseções limitadas por amostra e moldura compartilhada;
5.adequado,preflight e validação nativa antes da mutação;6.adequado,capturas
explícitas/mesmo renderer e nenhum upload novo no produto;7.não se aplica,nenhum
worker/lock novo;8.validado,undo/reset/load/replay/repeat/viewport;
9.validado,arquivos da árvore e demais fontes preservados;10.mantido com motivo,
JSON/cópias limitadas e sobreposição modular na autoria. Sem medição/alegação de FPS.
