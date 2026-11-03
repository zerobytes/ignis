# Duas escalas procedurais de superfície — EAF-112

Integração do editor: Scene78/replay82 reúne os contratos; gates históricos
permanecem73/74. As provas materiais usam o asset de calibração estável
`examples/cottage-material-baseline.scene.json` (SHA7286e74c…f1e2c), separado da
casa estrutural corrente `examples/elven-cottage.scene.json`. Receitas mantêm
hashes de origem/pixels; `base_recipe_sha256` acompanha somente a atualização do
caminho de entrada, sem relaxar comparação. Números abaixo são da frente original.


Contrato/checklist antes do código. A casa ea8c6b8/EAF-101 tem32172v/15assets:
596v e1slot livres. Materiais atuais têm um Pattern; suas oitavas compartilham
paleta, frequência base e relevo, dificultando combinar grão fino e manchas
amplas independentes. Isso é fricção de ferramenta, distinta das discrepâncias
estruturais da casa. A referência não é uma reconstrução geométrica certificada.

Alternativas: mais slots/meshes consomem os budgets já próximos; cores por
vértice exigem propagar outro canal por toda modelagem/regeneração; texturas,
UVs e um grafo de materiais exigem recursos/contratos mais amplos. Escolha
limitada: Material.secondary_pattern opcional do mesmo tipo Pattern, no máximo
dois campos. Nenhuma casa/gerador específico ou dependência de imagem gerada.

Ambos usam o mesmo M de repouso, com frequências, sementes, tipo, paletas, warp,
oitavas e bump independentes. tint=tintPrimário*tintSecundário; alturas somam e
o gradiente mundial é reconstruído uma vez junto do Detail existente. Não é
mascaramento de regiões, mudança de UV, metal físico, transparência, texturas
ou PBR completo. Pode funcionar sem pattern primário. None omitido, caminho
anterior preservado. Mesmos bounds de Pattern, soma dos bumps dos dois≤0.05m
(avaliada em f64 contra o limite f32 existente), Detail continua seu limite.
Tinta multiplicada pode alcançar4 por canal (dois multiplicadores até2), custo
finito/explicitado; autoria deve evitar saturação no framebuffer atual.

GPU secundário é bloco5vec4=80B: frequency/low/high/control/response. Mesmo
schema sem ID persistente novo. Material1152→1232B;17slots20944B (+1360B).
Vertex64/Globals176 e todos tetos geométricos mantidos. Footprints de cada campo
calculados antes dos branches; segunda avaliação somente quando habilitada,
até4oitavas adicionais (cellular consulta27vizinhos por oitava, como antes).
No máximo8oitavas entre dois campos; nenhum ganho FPS alegado.

Scene73/replay77 coordenados. Declaração nova em≤72 recusada inclusive fontes
ocultas antes de avaliar receitas. Validação compartilhada de Pattern, igualdade
semântica e cache de Material incluem a opção. Dispatcher/preview/undo/lote/
save/load/replay existentes publicam primeiro no MCP. Nenhuma janela da demo.

Checklist prévio:1.só autoria/fragmento habilitado, zero tick novo;
2.Option<Pattern> pequeno, Detail continuaArc, não criar dados por vértice;
3.tipo fechado/finito, validação de soma e layout explícito;
4.ruído comum reutilizado, footprints uma vez, nada de gerar textura/ler deGPU;
5.dois campos, bounds existentes/4oitavas cada, tetos de meshes/slots intactos;
6.+80B/slot, sem textura/novo draw/vertex;7.sem thread/lock/deps;
8.cache invalida por igualdade, testar troca/remoção/reset/load/undo/slots;
9.Rust/Clippy/MCP e legado em pixels, canais/escalas independentes, erro atômico,
replay/prefixo/animação/viewport, mesma casa e geometria/paleta de instância fixas;
10.manter scans/render/serialização existentes e custo extra por pixel explícito.

Prova autoral: cópia da MESMA casa, quatro escalas de aparência nos grupos de
pedra/ardósia/madeira, antes/depois e ablações com câmera/luz fixas. Preservar
geometria/plantas paramétricas/rig/clipes/instâncias e arquivo apresentado72.
A camada não corrige pórtico, bay, empena, seções das colunas ou vãos/motivos.

Verificação GPU distingue preservação byte-exata do legado/neutro de troca de
ordem dos dois campos: esta última aceita até1LSB de quantização por diferenças
de arredondamento na composição da cor. Não relaxar comparações do legado.

Resultado:854testes Rust,86cenários MCP/349grupos,Clippy/fmt/build aprovados.
Cottage-materials.md registra a prova, geometria preservada e avaliação visual.
O controle material-neutro e hero/detail ec96 são byte-exatos; camada secundária
muda as vistas e sua ablação remove esse efeito. Build e0a95a4b90e7fb39; critérios
de qualidade arquitetônica/artística permanecem abertos em EAF-101.
