# Ambiente distante procedural — EAF-116

Contrato e checklist anteriores ao código, base14636ba/Scene74/replay78.
Necessidade observada: o bronze tem realce colorido, mas fica plano fora da luz
direta. Um ambiente dá ao reflexo variação conforme normal, vista e rugosidade.
A composição/Sweep permanece em native-plants; esta frente não muda a geometria.

Alternativas: gradiente avaliado só na direção refletida é barato, mas ignora o
filtro de rugosidade/BRDF; integrar dezenas de amostras por pixel repete trabalho;
HDR importado requer contrato de recursos/decodificação/cores/paths. Escolha:
ambiente procedural limitado, pré-filtrado ao mudar parâmetros, com pipeline de
cubemap/LUT reutilizável. Não é importadorHDR, GI, reflexo local, parallax,
oclusão do ambiente ou céu de fundo. Pode iluminar interiores sem oclusão local.

Light.environment opcional: sky/horizon/ground RGB lineares0..1, axis finito
componentes±1/norma≥0.01 (normalizado), intensity0..4. Cores descrevem radiância
por direção: mix(horizon,sky,max(dot(d,axis),0)) e correspondente ground negativo.
Light.direct_intensity0..4 default1/omitido controla a luz direcional, porque o
ambiente não pode ser isolado com ambient legado (que também limita o direto).
None ou environment.intensity0 preservam pixels; direct_intensity1 conserva
aritmética legada. Recursos novos requerem Scene76/replay80, inclusive ambiente
configurado com intensidade0; direct_intensity!=1 em≤75 recusa. set_light/lote/
preview/inspect/undo/save/load/replay usam dispatcher existente, sem UI nova.

O ambiente é aditivo/independente da constante ambient legada; ela não é removida
implicitamente. Para ensaios só ambiente usar ambient0/direct_intensity0. Sombra
direcional não apaga o ambiente: não há AO/visibilidade para todas as direções.
Cores lineares em Rgba8Unorm (não sRGB), intensidade fora da textura; UNORM final
pode saturar, não há HDR/tonemapping novo.

Base primária: [Filament, Image based lights](https://google.github.io/filament/Filament.md.html).
Split-sum: cubemap especular64/7níveis pré-filtrada GGX/Smith com v=n,128amostras
Hammersley; irradiância dividida porπ em cube8,128amostras cosseno; LUTDFG64²,
512amostras por ponto, NoV0.001..1 e roughness0.2..1. No shader: diffuseCube*base
com redução1−f0 dielétrico; specCube(R,roughness)*(f0*A+B), metal f0=clamp(base),
mistura por metallic. Convenção f0=0 desliga especular dielétrico; transmissão
traseira usa outra consulta difusa. Sem compensação multiscattering/energia
global certificada. View-dependent Fresnel do difuso é aproximação declarada.

DFG A/B em dois UNORM16 empacotados emRGBA8, reconstruídos após filtragem linear;
precisão não exige dependência half nem recurso float32 filterable. Cube linear8
pode ter banding; limites/qualidade comparados com casos constantes/gradientes.
Cubemap v=n e amostragem finita são aproximações, não reflexo de path tracing.

Recursos fixos por Renderer: spec131064B +diffuse1536B +LUT16384B =148984B lógicos,
três texturas/um sampler; driver pode alocar mais. Alocar vazios no init, LUT só
no primeiro ambiente positivo, depois uma vez. Cubes só mudando cores/axis; eixo
normalizado e intensity fora da chave. Sem preparar em tick/frame estável. Uma
chave em cache, mesmas texturas reutilizadas; sem reter histórico de ambientes.
Temporários CPU de preparo limitados: cerca de149KB de saída e vetores amostrais.
Globals176→192B,Vertex64/Material1232/tabela20944 e caps mantidos; bindings4..7.
Por fragmento habilitado:2cube+1LUT (mais1cube se transmission), sem loops de
integração/sqrt novos no caminho estável; shader legado não lê essas texturas.

Checklist prévio:1.preparo eventual/fragmento opt-in, zero tick;2.buffers finitos,
sem bake por frame e sem novos canais por vértice;3.parâmetros/layouts/packs
explícitos;4.amostras GGX por nível/linha pré-calculadas, chave exclui intensity;
5.64/8/64/128/512 fixos, sem configurações de custo arbitrário;
6.148984B/3texturas+sampler/+16B Globals, uploads só quando muda;
7.sem thread/lock/deps/importador;8.testar None/A/B/A/intensity/undo/load/reset/
resize/sombra/slots, cache por conteúdo e texturas reutilizadas;
9.Rust/GPU/MCP com ambiente constante, eixo/vista/rugosidade, branco metálico
roughness1≈1−ln2 no normal, legado exato/replay/erro/rig/casa mesma geometria;
10.manter quantização/LUT finita/semAO/HDR/reflexo local e medir antes de FPS.

Coordenação: EAF-115 funcional reservou75/79 para Entity.geometry/Wedge. Esta
frente mudou a reserva para76/80 antes de persistir. Isso não integra a cunha:
validadores/RenderBox das duas frentes ainda precisam de merge explícito, não
interpretar número maior como capacidade funcional importada.

## Resultado da unidade

862Rust,88MCP/358grupos,Clippy/fmt/build aprovados no buildb2f55019d92fe070.
Metal branco áspero quase normal[151,151,151] contra analítico150; controles
constantes/eixos/cor/transmissão/rig e diagnóstico de cache passaram. Na mesma
casa, quatro vistas993076c35e8162d4 preservam os bytes antes de habilitar o probe.
Só Light muda; receita examples/cottage-environment.recipe.json e cliente
scripts/study_cottage_environment.py reproduzem controles e captura. Imagens/
report/saída76 atuais em artifacts/cottage-environment; todos os dados geométricos
permanecem os da prova anterior. Gates/custos em validation/performance.

Capturas incluem adapter.environment_preparation (bakesProbe/bakesLUT/bytes
lógicos); diagnóstico não participa da autoria/simulação/replay. A prova final
usa3/1bakes por trocas explícitas de eixo e cache único. Repetições, intensidade,
câmera, material, animação e viewport não geram novos prepares. Apresentação
nativa/aceite de fidelidade continuam separados.
