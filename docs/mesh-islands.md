# Ilhas e extração nativa de componentes

Para reposicionar peças sem criar outro asset, consultar
[transformação local de ilhas](mesh-island-transform.md) (EAF-125).

## Contrato e alternativas antes do código

Evolução de autoria geral: separar peças soltas de qualquer mesh sem repetir
arrays/índices em scripts. A árvore é benchmark, não tipo especial da engine.
Conectividade por vértice compartilhado, não por posição/epsilon e não apenas
aresta: duas faces que tocam o mesmo índice pertencem à mesma ilha. Costuras
com vértices duplicados continuam separadas. Vértices isolados são relatados e
permanecem na origem em Move; não há weld/nearest ou limpeza implícita.

inspect_mesh_islands retorna summaries determinísticos (ID=min face, contagens
e bounds locais), isolados e referência scope/revision existente. Esses IDs
só valem nessa revisão. extract_islands em edit_mesh recebe asset,id novo,
islands e reference, mode:move(default)|copy. Lista não vazia, sem repetição,
IDs válidos. Destino novo, sem overwrite. Move afeta TODAS as instâncias da
origem; rejeita esvaziar suas faces. Copy mantém origem e faz snapshot avaliado
independente, inclusive de planta vinculada, sem rebase dela. Move de planta
vinculada exige detach explícito prévio, podendo ser no mesmo lote.

Assets gerados mantêm espaço local original, shading/material, coordenadas3D
de repouso e pesos exatos por índice. Ossos e índices/nomes e clips são mantidos
inteiros em ambos, sem poda/recentralização/novas chaves. Clips Arc compartilhados,
pesos subset e Geometry novos. Bone attachments na origem continuam válidos.
Nenhuma instância criada ou vínculo alterado automaticamente: controle autoral
do TRS/playback/attachment com as operações existentes. Sem máscara de corte
arbitrário, cap/boolean, skin externo ou fusão de rigs nesta unidade.

preview_mesh_edit usa mesmo edit_mesh/validação numa cena clonada, sem publicar;
operação genérica para todos os edits existentes. Referências de subdivisão e
extração validadas por scope/revision. Lote recusa referência de asset alterado
antes no mesmo lote; destino também entra nos assets alterados. Um commit/undo
para toda extração, erros preservam runtime/histórico/seleção. Persistência
permanece58/replay62: somente comandos/consultas novos; resultado é Document
de assets já expressável no formato atual. Build ID continua exigido no replay.
59/63 não são consumidas por uma mudança de API sem novo formato persistido.

Preflight antes de copiar Geometry/pesos:16assets,4096v/8192t porasset,
16384v/32768t totais e budget global de clips/chaves. Copy soma payload/custos;
Move conserva vertices/triângulos armazenados, mas duplica rig/clips do asset
(mesmo Arc conta no teto global atual). Desenho muda só nas instâncias existentes
da origem; nenhuma cópia invisível altera o draw. Staging/final valida referências.

## Experimentos

Numéricos genéricos: duas faces com1v comum, faces coincidentes por posição
mas índices distintos, isolados, várias ilhas; Move/Copy com rig/clip e canais
por índice, posições posadas preservadas sem recentrar; nome de osso preservado.
Erros de seleção/revisão/lote/destino/budget/empty/planta, preview/undo/redo e
save/load/replay/GPU. Comparar resultados por mapas de índices explícitos.

Na árvore salva f810466, secondary-06 tem o ramo e6ilhas de rametas. Escolher
via summaries retornados (todas menores que a maior ilha), sem índices da
receita. Move rametas para novo asset; instância explícita, identidade local,
filha do secundário existente. Previsão10assets/32instâncias, mesmos11172v/
16250t armazenados e64794t desenhados. Preservar planta/source/reference/rig,
primeirofork e poses de todos os outros componentes; comparar pixels antes/
depois e animação no tick60. Teste não converte restantes ramos em instâncias.

## Checklist estático anterior

1. Partition/plan/subsets só na autoria/consulta; nenhum trabalho novo por tick.
2. Union/find e labels limitados4096v/8192t; summaries/bounds O(V+T), sem
   nearest espacial. Preflight antes dos novos payloads, Arc de clips mantidos.
3. Índicesu32 e referência existente; cópia exataf32, sem transformação geométrica
   ou alteração de precisão, unsafe/deps/threads.
4. Um partition por operação, não BFS por semente; seleção canônica e ordem
   de vértices/faces original preservadas; subset sem clone de pesos descartados.
5. Limites atuais, cópia explícita e nenhuma instância automática; ossos/clips
   não podados, custos globais validados mesmo que payload seja compartilhado.
6. Mesmo renderer/cache/Vertex/shadow; capturas explícitas e paths correntes.
7. Dispatcher/staging síncronos; sem lock/thread novo.
8. Edits invalidam revisão/Frame/seleção existentes; preview não muta; mudanças
   anteriores no mesmo asset invalidam referência dentro do lote.
9. Preservação por índice e poses/pixels, erros atômicos/replay/persistência e
   árvore real; sem alegar que extração melhora aparência por si só.
10. Manter custo de validação/snapshot/skin/upload e contagem conservadora de
    clips compartilhados; sem FPS/LOD/grafo CAD ou realismo presumidos.

## Resultado na árvore salva

`demo_tree_islands.py --frames48` carrega o JSON existente, consulta o asset
secondary-06 e escolhe as seis ilhas menores que a maior. Nenhum array da
receita nem intervalo de índices é usado para construir geometria. O comando
nativo Move produz ramo35v/66t e conjunto de rametas36v/48t; a nova instância
identidade é filha de tree-secondary-06. Fonte/reference da planta, madeira
inteira (incluindo primeiro fork), demais assets/instâncias e poses nos ticks0
/e60 são exatos. Posições/canais/faces da união dos subsets coincidem com a
origem por multiset; não basta comparar contagens.

Capturas GPU antes/depois em repouso e tick60 são byte a byte iguais. Preview
coincide com assets publicados; undo/redo e rejeição da referência anterior
passam. Copy em preview de uma variante foliar rigada mantém geometria, pesos,
ossos, clipes e material completos sem tocar a origem. A bateria existente de
12casos de edição Linear/PCHIP×A/B×resolução também passa na árvore reorganizada,
incluindo save/load de meshes+plants, pixels de execução/replay e loop48frames.

Final10assets/32instâncias11172v16250t armazenados64794t desenhados; nenhum teto
foi ampliado. Fonte corrente5.664.534B, GIF2.481.054B/48frames distintos/4s.
Relatórios `artifacts/tree-study/artifacts/islands-report.json` e `report.json`.
A saturação das32instâncias é explícita; o probe de draw substitui uma instância
por madeira mais pesada para isolar esse limite, sem publicar o resultado.

A imagem foi preservada, não melhorada artisticamente por essa operação.
Materiais da casca ainda parecem rede, folhas sem nervuras/espessura, copa com
padrões repetidos; realismo continua aberto. Esta separação só organiza um
secundário. Não implementa seleção espacial, weld de seams, corte arbitrário,
scatter, LOD, rig compartilhado ou extração de regiões conectadas por corte.

## Checklist estático posterior

1. Frequência: `Document::islands/extract` e preview apenas em consultas/autoria;
   nenhuma nova fase, cálculo por tick ou caminho de gameplay.
2. Alocações: partição/labels/marks e subsets limitados pelo asset. Geometry e
   pesos próprios somente após preflight; clipes compartilham Arc e bones são
   copiados sem poda. Preview clona metadados de Scene e serializa resultado.
3. Tipos: índicesu32/referência existente, cópia exataf32 e espaço local mantido;
   sem unsafe, dependência ou ABI nova.
4. Repetição: união/find com compressão, um passe de faces e outro de vértices
   para grupos/bounds. Seleção canônica e remapeamento por array, sem busca
   espacial ou scan de todas as faces por semente. Validação prévia é mantida.
5. Limites:16assets e budgets globais V/T/clips/keys antes de copiar payloads;
   subsets não excedem asset de origem validado. Instâncias não são geradas;
   validação final inclui draw e dependências. Testes de limites/rollback.
6. GPU/I/O: mesmo renderer/Vertex/cache/sombra; nenhum upload/readback novo por
   tick. GPU usada explicitamente para provar igualdade e invalidação.
7. Sincronização: dispatcher/staging/undo existentes, nenhuma thread/lock novo.
   Partição, validação e serialização são síncronas e podem ocupar o serviço.
8. Invalidação: assets origem/destino marcados no lote, Copy não invalida fonte
   inalterada. Guardas scope/revisão antes de staging; preview não publica.
   Seleção/revisão/Frame seguem invalidação existente em edição/undo/load.
9. Correção: testes Rust de contato por1v, posição duplicada, isolados,
   preservação rigada e plantas; MCP de preview/Move/Copy, seleção, atomicidade,
   persistência/replay/GPU e árvore salva. Resultados finais em validation.md.
10. Custos mantidos: validação integral e JSON de preview, rig/clipes inteiros e
    contagem conservadora de clipes compartilhados. CPU skinning/expansão/upload
    já existentes. Sem FPS, escala de produção ou realismo presumidos.
