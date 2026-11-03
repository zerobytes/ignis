# Detalhes triplanares — prova opt-in

Decisão anterior ao código, EAF-85/EAF-80. Pesos por face derivados no fragmento
podem quebrar C0 nas arestas. Escolha coordenada: atributo próprio
material_projection_weights, não normal assinada nem dados escondidos em IDs.
Por face no espaço material, normalN, pesos=N²/dot(N,N), acumulados por área
material nos seus vértices. Normalizar soma por vértice; área material0 e
vértices isolados usam[1/3,1/3,1/3]. Não cancelar normais opostas nem inventar
um domínio quando coordenadas materiais têm posto menor. Ausência de canalM
usa posições de repouso, igual ao renderer existente.

Pesos são não negativos e soma1: interpolação não pode cancelar como uma normal
assinada, e preserva C0 em arestas compartilhadas. Sharpnessp transforma pesos
interpolados por potência p/2 e renormaliza, p1..8, p2 padrão ao ativar. Isso é
aproximação de pesos suavizados, não normal material exata, garantia de
anisotropia/ausência de overlap ou fibras físicas. Pode misturar fases/traços
nas transições; precisa de avaliação visual e por ablação.

Cache pertence ao Frame, por Arc<Geometry> imutável, compartilhado entre
instâncias e poses. Calcular mesmo em planar para não deixar cache obsoleto
quando só o material muda. Preparar antes do short-circuit visual para podar
identidades trocadas com conteúdo igual em load/reset. Manter só geometrias
atuais, inclusive assets não instanciados; Arc retido força COW em make_mut.
Nenhum OnceLock na Geometry clonável. Não repetir O(T) por instância ou tick.

Vertex52→64B, atributo vec3 extra, +23.1% e+4.5MiB de payload ativo no teto
131072triângulos expandidos; buffers/capacidade e cópias podem custar mais.
Tabela1136B×17 e Globals176B mantidos. Não alegar FPS.

Detail.triplanar: Option<Triplanar{sharpness default2}>, skipNone,
deny_unknown_fields. None é caminho planar legado; Some({}) ativa p2.
detail_control.w livre carrega0ou p1..8. Projeções XY=(Mx,My), ZY=(Mz,My)
nos lados e XZ nas tampas, Y longitudinal do material. Plane continua usado
somente no modo planar. Ruído3D existente não é triplicado.

Tint usa a soma dos campos ponderados. Bump mistura covetores SCREEN dos
três campos com os pesos congelados, reconstruindo o gradiente mundial uma
vez. É aproximação, não derivada exata do tint/altura misturada (omite dw).
Derivadas de campo/M/world ficam em controle uniforme fora dos if/loops de
material/projeção. Footprint próprio por projeção. Não somar normalmaps em
frames distintos. Até96testes de segmento por fragmento detalhado, contra32.

Integração de formato: Scene70/replay74 tornam a opção autorável pelo dispatcher,
preview e edição nativos. Cenas declaradas<70 recusam a opção inclusive nas
fontes ocultas de plantas, antes de reavaliar a receita. O ramo original provou
GPU em RenderFrame transitório mantendo69/73; esses resultados são distintos
da publicação persistente70 desta integração. O modo permanece desativado na
árvore canônica: misturar projeções não conclui orientação física da casca.

O cache vive em meshes::Frame. A janela conserva esse Frame e reutiliza os pesos
entre poses. Engine::render_frame cria um Frame novo por snapshot/captura MCP:
nesse caminho há reconstrução por captura, sem reutilização entre snapshots.
Esse custo é mantido explicitamente; não se alterou o serviço para cache global.

Checklist prévio:1.autoria de pesos apenas por identidade nova, shader só
fragmento;2.cache limitado aos assets atuais, 12B/v no cache e12B por Vertex,
sem reconstruir pesos por pose no Frame persistente;3.f64 CPU/f32 pesos e limites finitos;4.um passeV+T por
Geometry nova e até3campos de32segmentos por fragmento habilitado;5.tetos V/T/
slots/traços mantidos, sharpness1..8;6.layout explícito64B e uploads maiores,
sem textura/readback extra;7.sem thread/lock/dependência;8.cache podado, toggle/
COW/load/remoção/reset testados;9.legado em pixels contra build prévio, GPU
cor/bump/combo, frameM divergente da geometria, rank0/1/fallback, animação,
replay/cache/viewport e Rust/Clippy/MCP;10.suavização de pesos/gradiente congelado
e mistura de padrões mantidos como aproximações, nenhum aceite artístico inferido.
