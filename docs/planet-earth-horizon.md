# Horizonte terrestre e coordenadas globais — EAF-199

Direção03/11/2026: o mundo não deve ter teto arbitrário de10km; demonstrar a
câmera e o horizonte de um planeta em escala terrestre. O teste usará diâmetro
12.742.000m (duas vezes o raio médio volumétrico6371km da
[NASA](https://nssdc.gsfc.nasa.gov/planetary/factsheet/earthfact.html)). A superfície
é uma esfera ideal sem relevo/refração; não é reconstrução da geografia terrestre.

## Diagnóstico antes de editar

`world_space::Position` já usa3células i64 de1024m e offsetsf64 canônicos: a ordem
de alcance por eixo é9,44×10²¹m, muito além da escala terrestre. O centro global
não passa integralmente porfloat. Os10000m são validações legadas em Camera,
Mesh/Entity e colisão da cena residente, não limites desse endereço.
O renderer de meshes também usa far-plane fixo30km. Afrouxar validações e passar
coordenadas astronômicas diretamente aof32 não resolve precisão nem residência.

O teste anterior mostrou um patch terrestre isolado; a borda do patch não é o
horizonte. Esta rodada exige cobertura do horizonte, e duas provas separadas:
malha nativa em torno da câmera ao nível do solo e referência geométrica global
por interseção raio/esfera no próprio wgpu da Ignis, em várias altitudes.

## Checklist estático antes da implementação

- Fonte/pose/globalcamera f64+Position1; matemática render-relative, sem alocar
  uma malha proporcional ao diâmetro ou converter centro global paraf32.
- Resolver a esfera implicitamente porpixel tem custo proporcional aos pixels,
  não à área mundial. Pipeline/uniform reutilizados no device de captura já
  existente; nenhuma thread/socket/dependência ou nova varredura no frame normal.
- Preparar orientação/câmera/coeficientes uma vez porcaptura; não repetir parse,
  clones, validação ou trigonometria porpixel naCPU.
- Usar coeficientes estáveis para câmera próxima da esfera: `η=h/R`,
  `c=η(2+η)`, `b=(1+η)dot(up,ray)` e raiz próxima `t=c/(-b+sqrt(b²-c))`.
  Isso evita obter h pela subtração de dois raios arredondados emf32. A unidade
  interna do cálculo éR; diâmetro/altitude/distância autorais continuam em metros.
- Captura de referência admite apenas esfera uniforme. Fonte com relevo deve
  ser recusada explicitamente; não ignorar silenciosamente operadores.
- Validar fonte, pose, frustum, dimensões/caminho antes deGPU/escrita. Readback
  é exclusivo de captura explícita; memória temporária limitada pela imagem.
- Provas críticas: campos globais grandes/offset, silhouetteGPU contraoráculoCPU
  f64, ausência de borda artificial, validação sem mutar cena ou sobrescrever
  resultado em falha. Não criar testes porcor/detalhe visual.

A nova operação não remove as guardas legadas de gameplay nem implementa
mudança de origem da física. Sua imagem representa toda a esfera de referência,
sem far-plane de30km ou teto10km; integrar esse espaço ao gameplay continua um
contrato próprio. Não apresentar este teste como planeta jogável completo.

## Implementação e contrato

`CapturePlanetReference {document,view,path,width,height}` é MCP-first e somente
observação/escrita da captura. `view` contém pose radial/altitude, pitch,FOV
vertical e máscara diagnóstica opcional. A câmera global aparece como Position1
nos metadados. Há o mesmo limite de imagem64..2048px dos captures normais;
não há far-plane nem distância máxima fixa em metros nesta operação. Posição
continua sujeita ao domínio numérico de Position1 e à precisão da posef64.

Fonte com operadores ou altura base diferente de zero é recusada: essa imagem
é explicitamente uma **esfera de referência**, não um renderer que silenciosamente
ignora relevo. O PNG é renderizado pelo próprio wgpu da Ignis. Não há imagem
externa, edição de PNG ou deformação aplicada depois de renderizar.
`src/planet/view.rs` prepara a câmera; `reference.wgsl` intersecta cada raio
com a esfera. Uniform de96bytes e pipeline são reutilizados no mesmo device do
renderer de captura; não é criado outro device. O fundo tem cor de referência,
sem atmosfera/refração, e a superfície tem cor e iluminação simples.

Perto do solo a forma estável comη=h/R evita cancelamento. Longe (h≥R), a
classificação usa distância angular pelo produto vetorial e o normal deriva
do ponto de maior aproximação do raio. Isso evita tentar recuperarR² por
subtração de duas distâncias astronômicas ao quadrado. A formulação implícita
é geometria de esfera; distinção entre superfície implícita e paramétrica e
interseção porraio é descrita no [PBRT](https://www.pbr-book.org/4ed/Shapes/Spheres).
A implementação não incorpora código ou certificação numérica do PBRT.

A fonte conserva seu diâmetro em metros em todas as vistas. Usarη e unidadesR
na álgebra do shader é mudança de unidade do cálculo, com a razão física h/R
conservada. Não é gerar uma Terra de800m ou alterar a altura da câmera para
caber na cena antiga. A operação não usa `Scene.camera` nem a projeção local
f32 de um planeta inteiro e, portanto, não herda seus limites10km/30km.

## Prova independente com a malha de terreno

A fonte terrestre uniforme também foi publicada pelo `PublishPlanet` normal,
com16tilesN128/nível12 em torno da câmera, face+Y. Os campos da cena continuam
em metros locais:266256vértices e524288triângulos; a malha curva é exatamente a
que o renderer normal recebe. Câmera nativa[0,2,0], alvo[0,2,-100],FOVvertical60°.
O centro da fonte usa células±9007199254740993 e offsets[0,125;0,5;0,25]m.
O renderer não recebe esses inteiros enormes comofloat.

A borda externa mais próxima fica a6221,677710m de arco. O horizonte da
câmera2m fica a5048,167324m de arco: todo o disco de horizonte está coberto,
com margem. Não é a borda de um patch isolado. A malha fica aproximadamente
12,44×12,44km no referencial tangente; isso não é um limite mundial novo.
As linhas de grade visíveis na captura nativa pertencem à apresentação existente
do renderer, sem um piso plano adicional na cena.

A máscara do céu/solo da captura nativa foi comparada com um oráculof64 porraio
em metros físicos, independente dos coeficientesGPU. A esfera analítica passa
pelo mesmo oráculo em cinco altitudes, incluindo órbita. Comparação permite
incerteza só na vizinhança de1pixel da silhueta; a primeira rodada teve **zero
pixels diferentes**, inclusive na malha nativa1280×800 e nas máscarasGPU400×256.

As imagens originadas de centro0 e do centro global enorme devem ser byteexatas.
Isso prova a invariância portranslação deste caminho de câmera/fonte, não um
runtime de todos os objetos da engine em coordenadas globais. Os offsets
submétricos preservados são conferidos emCPU e nos metadadosMCP.

## Dimensões do horizonte

Sem refração, comR=6371000m:

| Altitude da câmera | Distância em linha reta ao horizonte | Depressão abaixo da horizontal local |
|---|---|---|
| 2m | 5,048168km | 0,045399° |
| 100m | 35,696078km | 0,321019° |
| 10km | 357,099426km | 3,208115° |
| 400km | 2292,771248km | 19,792597° |

Em2m a curvatura lateral é discreta demais para exigir um arco visível nesseFOV
ou nessa resolução. Não exagerar curvatura para parecer um planeta. Em10km e
400km ela fica progressivamente perceptível; o pitch de400km é−25°, explicitado
na galeria. A vista do disco inteiro usa altura10000km e pitch−90°, mesmo raio.

## Validação, reprodução e fronteiras

`python3 scripts/verify_planet_reference.py` gera fontes, capturas, máscaras,
relatório e `artifacts/earth-horizon/review.html` semjanela, sem dependênciasPython
novas e sem interferir no mundo aprovado. Usa o decoderPNG limitado já existente.
O replay/Scene/PSV1/checkpoint/protocolo não mudaram; a nova câmera é configuração
de uma captura explícita, não primária de simulação. Fonte/cena viva permanecem
iguais nas consultas; erros preservam um output preexistente. O cenário está
registrado no agregador, sem afirmar execução da suíte global.

Dois testesRust agregados protegem endereço/offset/orientação/horizonte e a
validação que impede alocação inválida ou omissão silenciosa de relevo; incluída
preparação de câmera a10¹⁸m sem teto arbitrário de altitude. Gates selecionados:
planet_reference2 +planet3 +world_space1 +terrain_collision2. Testes anteriores
não foram removidos nem afrouxados. Checklist posterior: nenhuma nova alocação
ou análise de documento porpixelCPU, nenhum trabalho no frame normal; textura e
readback limitados porresolução, pipeline/uniform residentes e temporários
liberados a cada captura. Não foram medidosFPS/custo de gameplay.

**Continua aberto:** as validações±10km da cena/física legadas não foram removidas
em massa, e não houve migração de atores para coordenadas globais nem mudança
de origem física. Devem deixar de ser uma fronteira de mundo quando o runtime
regional for integrado. Esta rodada entrega o teste visual em escala real e um
caminho planetário de apresentação que já não depende dessas caixas; não é
streaming automático, terreno procedural completo à distância ou planeta jogável.

Rodada final: buildmacOSarm64release `a1253ad95fa6238f` (rustc1.97.1), fingerprint
conferido contra os fontes.8Rust distintos/4alvos, repetição dos2novos após revisão
numérica perto/longe; Clippyalltargets -Dwarnings, check, build, fmt/Python/diff.
MCP4grupos passaram no binário final;5máscarasGPU400×256 e1máscara de malha
1280×800 com0pixels diferentes do oráculof64. Cinco pares origem0/coordenadas
globais enormes com PNG byteexato. Erros mantêm estado vivo e saída anterior.
Maior erro de projeção da malha0,244135767mm. Capturas finais de solo, voo,
órbita e disco inspecionadas. Aceite humano e física planetária não inferidos.

## Passeio local aberto por solicitação humana

**Atualização:** o launcher agora prepara relevo seeded por padrão; receita,
verificações e janela corrente estão em [earth-procedural.md](earth-procedural.md).
Os números abaixo registram o passeio uniforme inicial, reproduzível com `--flat`.

`scripts/open_earth_walk.py` prepara porMCP uma cópia derivada da cena terrestre,
ativa colisão nos16tiles e adiciona Player/KCC e quatro marcos sólidos. Não
altera fonte/galeria/mundo aprovado. A preparação aplica cor/colisão/atores
numa transação fria; geometria lida localmente uma vez, sem milhões de vértices
porJSON/MCP e sem script porframe. Reutiliza binário corrente e launcherIgnisPlay
com symlink, sem novo build/cópia. Modocontinuous offline, primeira pessoa e
Play pelo mesmoMCP. A janela sobrevive ao término do cliente.

Verificação direta:15,969256m caminhados/240ticks, grounded no final; captura
`artifacts/earth-walk/verified.png` e relatório verification.json. Cena corrente
`artifacts/earth-walk/walk.scene.json`; launch.json contémPID/buildcontexto.
Janela PLAY e superfície confirmadas porCUA; não inferir aceite humano.

**É passeio local:** gravidade continua−Y e referencial dos controles continua
o atual, sem gravidade radial/origem móvel/navegação mundial. A fonte é terrestre
e a malha é curva, mas esta demo não permite circunavegação física completa.
O terreno cobre cerca12,44km porlado no frame fixo; afastar-se do miolo perde a
garantia anterior de cobertura de todo horizonte. Não há paredes invisíveis nem
streaming implementado. Clique+mouse/WASD,Shift corre,Tab alterna1ª/3ª,Esc libera
mouse,P pausa. Validação proporcional à montagem de cena/launcher, sem testes
novos ou rebuildRust. Nenhuma mediçãoFPS ou melhoria do renderer inferida.
