# Runtime planetário — integração EAF-199 em andamento

O runtime opt-in já percorre fisicamente a esfera e transporta o referencial do
jogador enquanto instala/retira terreno nativo. A demo antiga `earth-walk` é um
bake fixo; o passeio planetário usa `scripts/open_planet_walk.py` e
`artifacts/planet-runtime/`. Não confundir os dois modos.

## Executar e controlar

```sh
cargo build --locked --release --bin ignis
python3 scripts/open_planet_walk.py --seed 731199
```

`--prepare-only` prepara e verifica pelo MCP sem janela. `--skip-prepare` abre
os arquivos já preparados. Uma janela existente com o mesmo contexto é
reutilizada; contexto diferente recusa duplicação. Um executável corrente em
`target/release/ignis`, acessado pelo symlink de Ignis Play, sem cópias históricas.

A montagem tem diâmetro terrestre real **12.742.000 m**, agora com a receita de
[cadeias e vale](planet-mountain-chains.md) de31operações (a montagem inicial
abaixo usava23), WASD/mouse, Shift600 m/s, Tab primeira/terceira pessoa, P pausa
e R reinicia **preservando o modo planetário**. O personagem pode perder apoio
ao sair de uma crista em alta velocidade; gravidade/colisões continuam físicas.
A curvatura não é exagerada para ficar visível à altura dos olhos.

MCP e CLI usam a mesma implementação:

- `LoadPlanetRuntime {path}` lê configuração relativa ao projeto.
- `SetPlanetRuntime {config}` instala candidato validado; `config:null` desativa
  explicitamente e reinicia a cena autoral.
- `InspectPlanetRuntime` informa pose/global Position1, terreno residente,
  nascimentos/retiradas, distância física acumulada e estado do controlador.
- `--continuous --planet-runtime CONFIG --first-person player` abre sem agente.

Ativação requer paused/tick0/continuous/offline e um Player box/KCC pequeno,
sem outros atores/componentes de gameplay. É uma restrição explícita deste
adapter inicial, **não limite de diâmetro ou de área mundial**. Configuração
inclui documento planetário, ator, spawn radial do centro do corpo, nível,
subdivisões, raio físico, cor/material e compilação atmosférica.

## Física, origem e continuidade

O KCC existente continua sendo a autoridade do movimento. Após cada passo,
seu deslocamento real é convertido em pose radial f64. Referencial tangente,
velocidade, observações do controlador, orientação da câmera e luz são
transportados; o corpo volta à pequena origem residente sem renascer nem
trocar identidade. Gravidade segue o raio, com magnitude g*(R/(R+h))².
O centro global Position1 conserva células inteiras enormes e nunca vira f32.

Cada tile conserva geometria e anchor canônicos imutáveis. Instâncias/corpos
mudam de transform; samplers não reiniciam na borda. Normais vêm do gradiente
do mesmo campo esférico, inclusive entre faces; seu Arc identifica um derivado
separado no cache GPU. Coordenadas de material e direção global da luz também
conservam seus domínios. A grade diagnóstica XZ é suprimida apenas neste modo.

Colisão usa a triangulação completa da fonte, sem ATM/LOD gráfico. Contatos
genéricos f32 travaram o primeiro ensaio terrestre aos22,6 km: o motor
planetário passou a usar SAT contínuo f64 triângulo/cuboide com13eixos e BVH
existente. A margem projetada é conservadora, não uma pele euclidiana arredondada
exata. Testes incluem piso grande, afastamento, parede fina e extensão finita;
esse dispatcher é opt-in planetário e não substitui os contatos do mundo legado.

## Residência e cobertura real

Recursos candidatos (malha, derivação atmosférica e SharedShape quando físico)
ficam prontos antes da troca dos conjuntos. Física e visual têm raios distintos.
Refresh ocorre após64 m de deslocamento3D; recursos sobreviventes conservam
geometria GPU. Saídos são retirados também do catálogo, não só das instâncias.
Há orçamentos de4096tiles e2M vértices fonte por residência; pedido excessivo
falha inteiro, preserva cobertura anterior e pausa a execução. Não enumera nem
materializa a superfície completa do planeta.

ATM simplifica aparência, mas não autoriza sozinho apagar terreno. Este primeiro
adapter exige atmosfera positiva, fundo igual ao ar, terreno difuso opaco e
ausência de layers/edge_softness ativo/shadows/IBL/F0/metallic/transmission.
Combina bound real de material×iluminação com erro de omissão ≤2*C*exp(−density*d),
C arredondado para cima e10% do orçamento color_error. A receita usa density0,0008
e color_error0,002: raio visual aproximadamente11,5 km, mais512 m de prefetch
para deslocamento, passo seguinte e câmera relativa de até125 m.
PlayerView é obrigatório; câmera livre/editor não recebem cobertura presumida.

O corte deve caber na projeção existente (far30 km); clima sem certificado ou
cobertura maior é recusado, **não silenciosamente truncado ou alterado**.
Ar limpo, iluminação mais geral e visão orbital precisam de uma representação
distante adicional. Depth32/empates e ATM impedem inferir um certificado RGB8
universal só pela fórmula linear; os oráculos nativos abaixo são evidência
amostral, não promessa para todo conteúdo, resolução ou GPU.

## Provas integradas

`tests/planet_runtime.rs`, release explícito incluindo o ensaio ignorado:

- Esfera2 km:6289,87 m pelo KCC/11583ticks; retorno0,64 m, quatro faces e
  dois polos, identidade preservada e reset continua planetário.
- **Terra12.742 km**:2.401.944 passos físicos, **40.030.192,91 m percorridos**,
  retorno **14,13 m** da origem (um passo tem16,67 m a1000m/s), quatro faces
  e ambos os polos, grounded e mesmo indivíduo. Centro com células
  ±9007199254740993, sem relógio acelerado, teleporte de rota ou GPU nesse ensaio.
- Ativação inválida atômica e persistências incompletas recusadas.

Não usar apenas soma de ângulos para declarar uma volta: o teste exige progresso
meridiano desembrulhado, retorno global, faces/polos e identidade, a partir dos
resultados reais do KCC. Consultas geométricas continuam sendo prova separada.

Os testes internos verificam normais compartilhadas entre todas as costuras
residentes e invalidação GPU por mudança visível de um derivado, com apenas um
payload enviado e igualdade RGB8 com upload frio. `render_residency` compara
catálogos mutáveis com oráculo frio; ver [incremental-residency.md](incremental-residency.md).

Oráculo de cobertura GPU: seis poses nativas (polo, borda de face e troca de
residência), FOV45/60/90, três formatos de viewport. Frustum/oclusão ON×OFF
preservam RGB8 exato; residência normal/ATM contra raio1,5× e fonte completa
difere no máximo1nível RGB8. Sem atmosfera as mesmas vistas expõem11402pixels
da faixa distante omitida, evitando um teste vazio. É evidência desses casos
difusos, não certificado universal de todos os materiais ou profundidades.

Preparação MCP inicial no build **e071400322a33c01** (histórico), seed731199: dois percursos
no relevo atingiram14.901,49 m e14.560,47 m da origem, grounded, com372/378tiles
criados e150/155retirados; conjuntos finais222/223visuais e2físicos. Capturas
primeira/terceira pessoa inspecionadas, sem borda quadrada aparente nesses casos.
Inicial:216tiles/1.769.472triângulos fonte; frustum reteve61tiles e499.712triângulos,
40instâncias receberam material atmosférico econômico (erro declarado0,001528).
Geometria selecionada27,43MiB, **não RAM/VRAM total**. Oclusão está habilitada,
mas a política existente dispensa o passe abaixo de1M triângulos candidatos;
não foi forçada só para mostrar um contador ativo. IGL/ICB permanecem OFF.

44testes release focados (40em7alvos de integração e4unitários contato/normais/
GPU), Clippy selecionado com-Dwarnings, fmt, Python AST e diff aprovados.
Relatórios/capturas correntes em `artifacts/planet-runtime/`; `verification.json`
inclui build, percursos e adapters gráficos de cada captura.

Demo inicial aberta em PLAY/primeira pessoa, **PID93648/PPID1**, binário via
symlink Ignis Play; AX e screenshot da janela conferidos, processo sobreviveu
à saída do cliente MCP. `artifacts/earth-walk/launch.json` guarda o contexto da
única janela corrente, incluindo a nova configuração planetária. Essa janela
foi substituída pela receita de cadeias/PID10923; contexto e evidência atuais
em [planet-mountain-chains.md](planet-mountain-chains.md). Preservar a
janela humana em builds seguintes até coordenar a substituição. Aceite humano
do passeio e perfil de desempenho durante streaming continuam separados.

## Contratos ainda não migrados

Runtime é de sessão, separado da autoria Scene. Cena109/replay119/conteúdo7,
protocolo5/checkpoint3/PSV1 permanecem com seus contratos atuais. Checkpoint/save
portátil, SaveScene e multiplayer são **recusados** enquanto o modo está ativo,
para não perder estado primário silenciosamente. Fonte `.ignis-planet` e
configuração são reproduzíveis, mas não são save de uma partida planetária.
NPCs, habilidades, mundo multiator, navegação global e persistência/undo deste
runtime ainda precisam migrar. EAF-199 permanece In Progress, com aceite humano
da nova demonstração separado das provas automatizadas.

## Checklist estático de performance do runtime

- Caminho quente tipado Rust: sem JSON, arquivo, agente, socket ou worker novo
  por frame; matriz pequena por tile residente, sem regenerar vértices a cada
  origem. Transporte e SAT usam valores na pilha.
- Sampler/gradientes/cook/ATM só na entrada fria de tiles; SharedShape e fontes
  imutáveis compartilhadas por Arc. Memória acompanha a janela residente.
- Cache GPU conserva sobreviventes e layouts nativos24/48/64B; frustum,
  oclusão e ATM existentes participam. IGL1/ICB1 experimentais continuam OFF.
- Mudança de catálogo ainda recompõe streams CPU. Preparação/uploads são
  síncronos e podem causar pausas; não se promete streaming assíncrono, ausência
  de hitch, orçamento global de VRAM ou FPS sem medição.
- Não se muda near/far isoladamente, não se elimina geometria física por LOD e
  não se acumula histórico de tiles/builds.

## Primeiro contrato integrado: residência e referenciais

`planet/residency.rs` fornece endereços estáveis `Tile { face, level, xy }` nas
seis faces da mesma esfera cúbica usada pelo compilador. O endereço independe
da origem residente. Cache deve combinar esse endereço com identidade/revisão
ou digest da fonte e parâmetros do bake; um nome de tile sozinho não identifica
seu conteúdo.

`QueryPlanet.plan_residence { document, request }` recebe pose, nível e raios
físico/visual em metros. Seleciona regiões nas seis faces, incluindo arestas e
cantos, sem singularidade de longitude/polo. São conjuntos separados, com o
físico contido no visual. A resposta declara `resources_installed:false`.
O raio visual é pedido do chamador, **não certificado atmosférico automático**.
Não retirar cobertura só porque um pedido passou nesse planejador.

A busca desce uma árvore de cones conservadores: cada retângulo normalizado da
face está contido no cone de seus quatro cantos. Pontos dentro de uma esfera
de raio `d` ao redor do observador radial `r` estão contidos no cone de abertura
`asin(d/r)` quando d<r; quando d>=r conservamos todas as direções. Isso independe
da altura do relevo. O orçamento de 4.096 tiles rejeita o pedido inteiro; não
entrega um subconjunto com buracos. É um orçamento de uma operação de residência,
não limite de extensão do planeta.

`Anchor` faz projeção/inversa e transporte rígido f64 entre referenciais
tangentes. Sagita e altura usam expressões estáveis; o centro global nunca entra
em f32. O tile conserva geometria local imutável e seu anchor canônico. Mudança
de origem deve alterar transforms de instâncias/corpos, sem reamostrar/rebakear
os vértices ou invalidar derivados gráficos pela mera mudança do observador.

Três testes agregados cobrem todas as faces, arestas/cantos/polos, subconjuntos
físicos/visuais, determinismo, rejeição de pedidos inválidos/excessivos e
transporte de ponto/velocidade/altura submétrica numa volta de 512 referenciais.
A última é **prova matemática de frames**, não caminhada física.

Checklist estático: busca hierárquica limitada, sem enumerar a área do planeta;
endereços inteiros e conjuntos ordenados determinísticos; cálculo tipado sem
JSON, GPU, I/O, worker ou lock; projeção/transporte sem alocação. Listas de
residência alocam somente em uma consulta explícita. Fonte/Scene/replay/PSV1 e
física corrente não foram alterados por esse primeiro contrato.

## Coordenação com nanites

Consulta autorizada em 03/11 ao chat `nanites`. A fonte já passa pelo caminho
nativo de Asset/Instance; frustum, buffers estáticos residentes, streams por
canais, oclusão e resumos atmosféricos são capacidades distintas de residência.
Handoff detalhado recebido; não importar pesquisa experimental como otimização
validada automaticamente. O primeiro adapter de caches está documentado em
[incremental-residency.md](incremental-residency.md).

Dois riscos identificados no início e tratados pela integração acima:

- `meshes::Frame` e `render_instances::Gpu` tinham invalidação por geração/conjunto
  global. Entrar/sair um tile pode limpar geometria GPU de tiles sobreviventes.
  Foi necessário desacoplar identidade de geometria, placement e origem, e limitar
  retenção/eviction. Manter Arcs imutáveis é necessário e não suficiente.
- Near 0,05 m/far 30 km são recompostos também pelo shader de oclusão. Mudar só
  FAR em `picking` pode produzir falsa oclusão. O primeiro recorte não deve
  alterar esses coeficientes isoladamente. View e consumidores também usam Y
  como up; referencial transportado precisa ser coerente em todos eles.

As reservas são systems para fonte/residência/física, e renderer/distant/shaders
sob coordenação com nanites. Mundo aprovado e cena botânica não são substituídos
pela montagem planetária. O primeiro contrato puro acima está no commit4bd2003;
a integração GPU incremental inicial está em74e3775.
