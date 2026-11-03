# Topologia e subdivisão linear

## Decisões e revisão antes do código

EAF-79 continua a autoria nativa sobre a base de plantas EAF-77. Triângulos
indexados continuam sendo as faces deste recorte. Uma estrutura half-edge ou uma
representação autoral de polígonos facilitaria operações futuras, mas exigiria
migração de referências e tesselação. A incidência de arestas é derivada sob demanda;
referências de componentes têm escopo de sessão/revisão, sem prometer IDs permanentes.

Subdivisão linear cria pontos médios compartilhados por índice de aresta e mantém
a superfície de repouso. Arestas entre vértices coincidentes de seams continuam
distintas. Seleção de faces marca suas arestas; seleção de vértices marca as arestas
incidentes. Vizinhos recebem divisões conformantes, evitando junções em T.
Loop/Catmull-Clark, vincos, extrude/bevel e uma stack genérica são incrementos
posteriores com contratos próprios. Undo/redo existente reverte esta operação.

Pesos novos interpolam o campo canônico sem poda; até4 influências positivas.
Ossos e clipes são compartilhados/conservados. Vértices antigos e suas poses são
mantidos em meshes livres; novos vértices seguem o campo de pesos interpolado.
LBS de posições e pesos interpolados não equivale, em geral, a interpolar as
posições já animadas. Não prometer a mesma superfície em todas as poses.

Plantas vinculadas registram subdivisão uniforme em `subdivision_levels` da
amostragem, conservando fonte canônica durante regeneração. Referência e destino
são reproduzidos pelo mesmo kernel. Subdivisão localizada vinculada é recusada
até existir uma representação autoral correspondente. Não há detach/freeze ou
rebase implícito. Adaptação de amostras percorre todos os vértices, inclusive novos
midpoints, e usa face/coordenadas materiais e descendência baricêntrica explícita.

Checklist prévio: preparação apenas em edição/consulta; BTreeMap determinístico
de incidência O(T log E), sem busca espacial ou scan de faces por vértice. Contagens
de saída verificadas antes de posições/pesos novos; limites existentes por asset,
documento e desenho. Crescimento geométrico por nível e limite finito antes de
alocação. Scratch de pesos32 reutiliza o kernel existente. Arc conserva payloads
não modificados; saída e planos temporários limitados alocam por edição, nunca por
tick. Sem dependência/thread/lock, GPU/readback ou I/O novos. Seleção é sessão e não
altera autoria, relógio ou conhecimento NPC. Erros preservam estado/histórico.
Testar referências obsoletas, limites/atomicidade, incidência, orientação, seams,
pesos/clipes, plantas subdivididas/rebase/perda em midpoints, undo/load/replay/render.
Sem profiling/FPS ou aceite visual inferidos.

## API e referências

`inspect_mesh_topology {asset}` retorna `asset`, `reference:{scope,revision}` e
`topology:{vertices,edges,faces,boundary_edges,non_manifold_edges,isolated_vertices}`.
Vértices são índices das posições de `inspect_mesh`; faces são índices em `faces`.
Arestas usam pares de índices de vértice em ordem crescente, com faces incidentes.
A consulta inclui elementos não manifold e isolados; não declara retopologia limpa.

`select_mesh_components {selection}` recebe null para limpar, ou:

```json
{
  "asset": "surface",
  "reference": {"scope": "copiar-da-consulta", "revision": 1},
  "components": {"kind": "faces", "indices": [0]}
}
```

Componentes admitem `faces`/`vertices` com `indices`, ou `edges` com
`vertices:[[a,b],...]`. A seleção deve ser não vazia, sem duplicados e com índices
existentes. Arestas invertidas são canonicalizadas. Índices são ordenados para
comparação/idempotência; erro preserva a seleção anterior. `inspect_mesh_selection`
retorna `selection` e `view_revision`. Selecionar não altera revisão de autoria,
histórico, relógio, playback, física ou poses. Não há picking/highlight/editor manual
de componentes entregue neste recorte.

Referências expiram quando o escopo de execução muda (reset/load/edição/undo/redo),
quando muda a revisão de autoria ou ao reiniciar a engine. Tokens antigos geram
`topology_conflict`; o cliente deve consultar novamente. Seleção é limpa nesses
eventos e não é persistida na Scene ou replay. Input/play/pause/step não a alteram.
O escopo é conservador para o documento inteiro, não uma identidade eterna de vértice.

## Operação e transações

`edit_mesh` recebe `edit.kind:"subdivide"`, `asset`, `reference` e `components`.
`components:null` divide todas as arestas. Uma seleção retornada pelo MCP tem os
mesmos três campos necessários, permitindo reenviá-la com `kind:"subdivide"`.
Não existe consumo implícito de seleção ou número arbitrário de níveis por comando.

Uma operação faz um nível. As posições antigas são prefixo da saída; pontos médios
são anexados pela ordem lexicográfica dos pares de vértices. Faces de saída seguem
a ordem dos pais; `Plan.parents()` informa a origem de cada face. Todos os incidentes
de uma aresta marcada são divididos. Triângulos com0/1/2/3 arestas marcadas produzem
1/2/3/4 filhos com orientação conservada. Degeneração numérica de um midpoint recusa
a operação. Não há weld por posição, detecção global de auto-interseção ou correção
de malha não manifold.

`edit_scene` aceita a mesma edição em `{op:"mesh",edit:...}`. Tokens referem-se à
revisão inicial da transação. Um subdivide não pode seguir outra alteração de fonte
do mesmo asset no lote; ordenar subdivide primeiro ou consultar após publicar a
alteração precedente. Assets diferentes podem ser subdivididos no mesmo lote.
Detach explícito não muda geometria e pode preceder subdivide local no mesmo lote.
O lote completo é um único undo e nenhuma falha publica resultado intermediário.

Contagens por asset e orçamentos globais de geometria/desenho são verificados antes
de alocar posições/pesos novos. Ossos/clipes não são descartados; um vértice novo com
mais de4 influências positivas faz a edição falhar. Redução de influências exigirá
política explícita própria. Histórico mantém payloads antigos por Arc; não há um
modifier stack não destrutivo para meshes livres neste incremento.

## Amostragem de plantas

`Definition.subdivision_levels` é opcional, padrão0 e omitido no JSON quando zero.
Aplica refinamento uniforme após os segmentos XYZ de cada face. O limite sintático
é0..6, com preflight de todos os níveis pelos limites reais de4096v/8192t, portanto
nem todo nível é admissível para uma base. Para cada nível `V'=V+E`, `T'=4T`,
`E'=2E+3T`, válidos para as grades independentes produzidas pelo gerador. IDs de
faces e coordenadas materiais são carregados para todos os novos vértices.

Uma edição subdivide em planta vinculada incrementa esse campo e regenera da fonte;
não altera `Entry.source`/`reference`. Escultura, rig e clips explícitos continuam
estabelecendo uma nova fonte na definição corrente, que pode já ser refinada.
Correspondências usam a face/célula original e descendem os filhos definidos pelo
kernel `uniform_child`. A tolerância de reconstrução percorre TODOS os vértices da
referência, incluindo midpoints esculpidos. Ela é erro amostrado em metros, não uma
prova contínua de Hausdorff nem uma métrica de erro de animação/pesos.

Subdivisão localizada vinculada é recusada com diagnóstico; ela não cabe na receita
uniforme atual. Para fazê-la em geometria livre, destacar a planta explicitamente.
Mudanças de segmentos, dimensões e níveis continuam sujeitas à correspondência,
tolerância de detalhe e budgets. A fonte canônica não é o bake anterior.

Cena50 permite nível não zero na definição E na referência; versões anteriores
recusam esse campo ativo mesmo quando somente a fonte de referência está refinada.
Cenas anteriores válidas com nível0 continuam migrando. Replay54 preserva a cena
autocontida refinada e compara poses; seleção de componentes permanece fora do
contrato de gameplay/replay. Não há migração de replay entre builds.

## Validação e limites de entrega

18 testes em `tests/mesh_topology.rs` cobrem padrões de splits, área/orientação,
incidência, seams, vizinhos, componentes/escopos obsoletos, transações, undo/roundtrip,
limites globais/desenho, pesos/clipes e poses antigas. Reuso de plano com asset rigado
incompatível é recusado antes de indexar os pesos. Plantas cobrem níveis0/1/2,
segmentos distintos, todas as faces, proveniência completa, fonte sem rebase implícito
e recusa de perda de detalhe criado apenas num midpoint novo.

`python3 scripts/run_verifications.py --only mesh_topology` exercita o pipeline
pelo transporte MCP real e salva demo em `artifacts/mcp-validation/examples/`.
Capturas/registros ficam em `artifacts/mcp-validation/artifacts/mesh-topology-*`.
Resultados completos, build e aceite serão registrados em validation.md e EAF-79.
