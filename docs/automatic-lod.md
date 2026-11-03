# LOD automático compilado — EAF-181

Escopo: medir a cena atual e comparar com ela mesma mais representações opcionais.
Renderer/custo pertencem a esta frente; fonte/colisão/residência continuam systems.
Não substituir conteúdo invisível por buraco, nem apagar cobertura coletiva.

## Decisão e revisão anterior ao compilador

ATM já compila posições/índices, mas sua seleção exige substituir também o material
por preenchimento atmosférico. LOD de geometria mantendo a aparência original
precisa de um caminho independente. O primeiro recorte é asset estático com
material uniforme difuso; materiais procedurais/PBR, rigs/clipes e sombras têm
fallback. Sem regra por árvore/nome/procedural. Impostor universal de2triângulos
não está prometido. Gerador de candidatos: meshoptimizer0.6.2 MIT, pinned via
wrapper Rust; sua métrica heurística não é certificado de erro da imagem.

Pacote sidecar binário, fora do Scene/replay/save/rede. Fonte editável/física
preservada. Referências a cantos fonte conservam posição/normal consumidas pelo
shader nativo; índices compactos e bounds serão verificados antes de publicar.
Geometria compartilhada por asset/nível, cálculo caro em compile/load. Em runtime,
seleção antes de upload/desenho por resolução física e resposta residual. Câmera
próxima, luz incompatível, fonte editada ou candidato sem ganho seguem completos.

Erro geométrico e erro da imagem são diferentes. Bounds de correspondência de
superfície/normal não provam oclusão ou iluminação arbitrárias; comparar pixels,
silhueta e estabilidade com luz/câmera/resize reais antes de promover um caminho.
Não tratar redução de triângulos como FPS. Pacotes inúteis não serão emitidos.

## Medição

NativeWindow mostra FPS/ms no início do título, F3 liga/desliga pelo mesmo
SetViewportPerformance/InspectViewportPerformance do editor. Janela de até1s,
publicação2Hz, resolução física e VSync registrados. Cadência de chamadas present
inclui simulação/preparação/acquisition/submission/scheduling. Não é tempo GPU
isolado nem scanout. Nenhum pipeline/textura de HUD extra; estatísticas detalhadas
somente na publicação. Amostras são reiniciadas em ocultação/resize/configuração.

Build e arquivos compartilhados coordenados com a terceira pessoa de systems.
A janela185 não será recarregada enquanto ele/usuário a utilizam sem combinação.

## IGL1 — candidato experimental

Cabeçalho `IGLOD1\0\0`, contagem u32 LE; por asset: tamanho u32/nome UTF-8,
SHA256 da fonte/material/shading, contagem de níveis u32. Por nível: erro de
superfície f32, erro de normal f32, contagens u32, referências a cantos fonte u32
(equivalentes à expansão de triângulos do renderer) e índices de triângulos u16.
SHA256 dos bytes anteriores encerra o arquivo. Nenhum JSON/base64 no sidecar.
Posições/normais não são duplicadas nele: vêm da fonte imutável ao preparar o
buffer GPU. O buffer compartilhado é preparado uma vez por asset/nível.

O verificador recompõe os limites em duas direções e exige cobertura de cada
componente conectado. Checksum não substitui essa verificação. Arquivo truncado,
índice inválido, fonte diferente ou limite subestimado são recusados antes da
publicação. O arquivo inteiro precisa ser válido; não se publica metade.

O primeiro compilador tem orçamento de 32.768 faces por asset e índices locais
u16. Assets maiores continuam no caminho anterior; isso é limite desta operação,
não limite de quantidade ou tamanho da cena. Regiões para assets maiores são
continuidade, e não suporte já entregue. `LockBorder` conserva fronteiras; não
usar poda automática de ilhas. Referência do gerador de candidatos:
[meshoptimizer](https://github.com/zeux/meshoptimizer).

Seleção experimental desligada por padrão. Erros de correspondência não provam
igualdade de pixels: mudanças de oclusão, normais por raio e rastreamento temporal
precisam de comparação de imagem. Sem promoção automática ou economia alegada
antes de medir a mesma cena, câmera, luz e resolução.

## Limite de correspondência usado pelo compilador

Para cada triângulo fonte, escolhemos um único triângulo alvo e três pontos nele,
`q0/q1/q2`, correspondentes aos cantos fonte `p0/p1/p2`. Para qualquer combinação
baricêntrica `λ`, `q(λ) = Σ λi qi` permanece dentro desse alvo convexo e:

`|p(λ) − q(λ)| ≤ max_i |pi − qi|`.

O mesmo vale para a diferença dos campos de normal **antes da normalização**,
pois ambos são afins nessa correspondência. Um limite inferior positivo para o
comprimento das normais fornece o limite após normalizar; sem esse limite,
conservamos o pior caso de 2. Subdividir durante a verificação aperta os limites,
mas esses triângulos de verificação não vão para o pacote nem para o runtime.
Verificamos os dois sentidos e cada componente conectado.

Isso substitui a primeira estimativa pela soma de gradientes, que era muito
frouxa. Não exige que a correspondência no interior seja a projeção mais próxima.
Também não prova que os dois raios da câmera encontrarão pontos correspondentes:
a comparação de imagem/oclusão continua sendo um gate independente.

## Resultado do primeiro incremento

Build conjunto: `aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|81727a97aaf6250a`. Os formatos novos105/115/6 pertencem à frente187;181 não alteraScene/replay/rede.

O caso de integração plano (2instâncias, uma fora da visão) passou de2.048 para512triângulos, com PNGs1280×800 idênticos. Limite projetado0,1465px; resposta2,72e-6. GPU selecionada10.776B; cache aquecido reteve também a referência, total49.200B (referência38.424B). Sidecar25.495B. Esses números não são uma redução de memória residente do jogo.

Na floresta30mil, o primeiro candidato sem filtro reduzia o bark11.140→8.590/7.778/7.516, porém os limites globais de normal1,153/2/2 impedem uso seguro no orçamento visual atual. Após reter componentes ruins completos nos três níveis (.02/.06/.2), não sobrou redução de pelo menos10%: pacote vazio, nenhum arquivo útil emitido. Folhas permanecem originais; o compilador geométrico não apaga suas ilhas para fabricar economia. Portanto **nenhum ganho novo de FPS da floresta foi obtido por esseLOD**, que permaneceOFF.

Comparação do caminho atual contra ele mesmo com a opção ligada (fallback),16amostras por variante, blocos alternados aquecidos, draw+esperaGPU, semPNG/UI/simulação. Não são timestampsGPU nemFPS de gameplay. A primeira pessoa usaSetPlayerView para ocultar o próprio corpo; a fixture inicial com câmera dentro do cubo foi descartada.

| Vista | Pixels físicos | Atual ms | +LOD ms | ErroRGB8máx |
|---|---:|---:|---:|---:|
| Primeira pessoa | 1280×800 | 91.623 | 91.335 | 0 |
| Primeira pessoa | 1728×1117 | 95.319 | 94.821 | 0 |
| Panorama3× | 1280×800 | 439.746 | 440.112 | 0 |
| Panorama3× | 1728×1117 | 487.187 | 487.786 | 0 |
| Panorama5× | 1280×800 | 27.333 | 27.289 | 0 |
| Panorama5× | 1728×1117 | 27.257 | 27.212 | 0 |

A variação pequena não demonstra ganho; não há LOD novo selecionado. Evidências locais:artifacts/automatic-lod/comparison.json,compiler.json,mcp.json,plane-render.json e PNGs. Codec/publicação9+1gates com atmosfera/player e clippyalltargets passaram; MCPplano verificou consumo real de índicesu16 sem expandir e deduplicar novamente.

Próxima investigação: os grandes assets do terreno (566.048tri por peça) excedem o orçamento de um oclusor completo. Na vista distante do teste o seed de oclusão teve somente2tri do oceano, deixando até608milhões de triângulos de instâncias candidatos. Avaliar regiões com subconjuntos EXATOS da geometria fonte para oclusão conservadora, além de representações coletivas/impostors para copas. Não atribuir esses gargalos a RAM ou prometer60FPS antes de medir.

Continuidade: [orçamento adaptativo de oclusores](adaptive-occluders.md) implementa
uma correção anterior à partição: permite o próprio terreno exato no pré-passe
quando seu custo é pequeno diante do trabalho de instâncias. As vistas aéreas
corrigidas ficam dentro do alcance efetivamente renderizado;3×/5× acima não são
gates válidos de detalhe distante por causa do far plane atual. O ganho novo
medido pertence à visibilidade, não ao protótipoIGL1 da floresta.

Continuidade EAF-181: [compilador por regiões](regional-lod.md) refina a busca e os
limites, conservando o melhor candidato global; a floresta ainda usa fallback.
[Oclusão em duas escalas](fine-occlusion.md) mede o ganho adicional sem mudar a
geometria visível. Para abrir o passeio independentemente do cliente MCP/terminal:
`python3 scripts/open_world_walk.py`. No macOS, Ignis Play usa um atalho separado
que aponta para o mesmo `target/release/ignis`, sem copiar outro build.

Objetos com várias peças agora podem declarar uma identidade visual explícita em
[meshes.objects](mesh-objects.md), sem usar attachments ou perder instancing. O
primeiro consumidor é a seleção de visibilidade por conjunto; isso ainda não
constitui o LOD coletivo da copa. Os formatos passam para107/117/7 nesse incremento.
