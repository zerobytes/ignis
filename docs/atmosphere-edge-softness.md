# Suavização opcional das bordas — EAF-170

Guilherme aprovou a paisagem de EAF-169 e pediu um acabamento configurável que
misture as bordas distantes com o fundo, semelhante a um smudge. O efeito é uma
suavização em tela guiada pela profundidade. Não é simulação atmosférica física,
pintura direcional, reconstrução temporal nem uma nova redução de geometria.

## Controles compartilhados

UI: **Iluminação → Atmosfera da cena → Suavizar bordas distantes**.
MCP: `set_light`, campo opcional `light.atmosphere.edge_softness`:

```json
{"strength":0.5,"radius_px":6,"start":2000,"end":10000}
```

- `strength`: 0 a 1. Zero desliga completamente, mantendo os demais valores.
- `radius_px`: inteiro de 0 a 12 pixels físicos por eixo. Zero também desliga.
- `start`/`end`: metros, `0 <= start < end <= 100000`. A força cresce suavemente
  nesse intervalo. Objetos anteriores ao início conservam os pixels originais.
- Ausência/null mantém a aparência nítida anterior. O recurso vem desligado.

É possível suavizar mesmo sem clareamento do ar (`density=0`). Cores do céu,
montanhas e influência solar continuam independentes. O raio é físico: não se
confunde com pixels CSS/lógicos nem cresce automaticamente com o DPR.

Scene102/replay112/conteúdo visual4. Scene101 e anteriores não aceitam o campo,
inclusive se estiver zerado. ATM3, protocolo4, checkpoint2 e PSV1 permanecem.
O mesmo Light validado participa de undo/redo, save portátil, replay e conteúdo
visual da rede; não há configuração exclusivamente visual.

## Render e custo estrutural

Quando ativo, a cena é desenhada numa imagem intermediária. Dois passes
separáveis misturam cores em luz linear com pesos gaussianos. O raio é limitado;
os pesos são calculados na CPU somente ao mudar configuração, FOV ou resolução.
Não há textura de ruído, histórico temporal, readback ou recompilação de meshes.

Os passes primeiro procuram uma descontinuidade de profundidade ao longo do
kernel. Sem borda, copiam o pixel. O limiar é relativo de 5%, com mínimo de 2 m,
e há tratamento explícito da transição superfície/céu. Isso evita confundir
pequenas diferenças da profundidade distante com textura interna. É uma
heurística de aparência: declives abruptos podem ser classificados como borda,
e superfícies de profundidades muito parecidas podem conservar uma borda nítida.

A influência usa a distância reconstruída pela projeção atual, com os mesmos
near/far da câmera. A mistura de cada par usa a menor influência dos dois lados:
o primeiro plano não espalha sua cor para o fundo. UI, guias diagnósticas e
outline são desenhados depois, conservando sua nitidez. A profundidade continua
representando a superfície original, não a borda suavizada; picking/física não
mudam. Objetos transparentes e desfoque cinematográfico não são implementados.

O filtro depende da profundidade. Por isso, com ele ativo, a seleção distante
conserva a geometria original e ainda pode substituir o material pelo
preenchimento barato. Um proxy deslocado poderia mudar a classificação de
bordas mesmo respeitando seu limite de projeção. Não se utiliza o desfoque como
justificativa para relaxar certificados. A seleção autoral do editor permanece
intacta. Ao desligar, os níveis geométricos compilados voltam a ser elegíveis.

Há duas imagens de cor de resolução completa e profundidade legível. Em RGBA8,
as imagens acrescentam `largura × altura × 8` bytes. O depth já existia; ativo
passa a ser armazenado/lido. Buffers de geometria são contabilizados à parte.
Não interpretar menor força visual como redução proporcional de custo: a força
positiva mantém os dois passes; raio e área distante afetam o trabalho.

Ausência, raio zero ou força zero usam o caminho direto anterior, sem imagens
intermediárias nem passes de suavização. Desligar libera suas referências e
recria o depth sem uso de leitura. Pipelines e um uniforme de 96 B podem ficar
em cache após a primeira ativação; a liberação física depende do driver.
`capture.adapter.atmospheric_edges` informa estado, passes e bytes lógicos.

## Verificação e reprodução

`python3 scripts/verify_atmosphere_edges.py` usa o executável MCP e a paisagem
anterior, preservando seus PNGs como referência. Produz comparação nativa entre
desligado, suave e forte, com câmera/luz diferentes, retorno ao estado nítido,
persistência/replay e um oclusor próximo colorido para detectar contaminação.
`target/release/examples/selection_outline` verifica que a máscara e as cores do
contorno continuam idênticas sobre uma superfície suavizada. O controle plano em
`scripts/verify_silhouettes.py` verifica o fallback geométrico com material barato.

O cronômetro existente aceita um diretório de evidências:
`target/release/examples/atmosphere_landscape_cost artifacts/atmosphere-edges`.
Mede draw CPU e draw + espera por conclusão, sem PNG, simulação ou apresentação.
Não é timestamp GPU nem FPS de jogo. Preservar também medidas da primeira
ativação/transições; os tempos aquecidos não incluem toda preparação inicial.

Imagens e relatórios em `artifacts/atmosphere-edges/`. O efeito altera a aparência
por decisão autoral; a referência de qualidade é a geometria completa com o
mesmo efeito. Aceite artístico continua separado de igualdade numérica entre
caminhos. Distância máxima, precisão distante e streaming não foram ampliados.

## Resultado técnico — 31/10/2026

Build nativo `70038ee07b714dc4`, Apple M4 Pro/Metal, resolução1728×1117:

- Ausência, força zero, raio zero e retorno após ativação mantiveram os PNGs
  anteriores byte a byte em três vistas/iluminações.
- Seis pares suave/forte entre material completo e preenchimento selecionado:
  diferença máxima **0 RGB8** com o mesmo acabamento. Isso não significa que o
  efeito preserve a imagem nítida: sua finalidade é modificá-la.
- Primeiro plano: **213444 pixels intactos**;48632 pixels do fundo mudaram sem
  vazamento da cor vermelha do oclusor. Três vistas do outline mantiveram máscara
  e cor exatas depois do filtro. O controle plano confirmou geometria original
  enquanto ativo, material barato preservado e simplificação novamente elegível
  ao desligar.
- Save/load/replay nativos reproduziram os pixels; oito testes Rust em quatro
  alvos passaram (atmosphere, environment, network_content, portable_save), com
  os quatro de atmosphere repetidos após a proteção geométrica final. Clippy
  all-targets, fmt, Python e regressão nativa anterior de seis grupos passaram.
- Duas imagens de cor: **15441408 bytes (14,73MiB)** adicionais quando ativo;
  zero quando desligado. Há ainda o uniforme96B/pipelines em cache, armazenamento
  do depth e custos do driver não quantificados. Não confundir esses bytes com
  RAM total do processo. Esse acabamento pode custar mais memória que a economia
  de geometria obtida pela substituição de material nesta paisagem.

As capturas e controles foram verificados pelo renderer/MCP nativos. A janela
humana já aberta não foi reiniciada e permanece no build anterior; não se alega
aceite visual humano nem teste de interação manual dos sliders neste recorte.

### Tempo observado

Duas execuções, quatro rodadas cada, perfis previamente aquecidos e ordem dos
perfis/caminhos alternada. Mediana das quatro medianas de40 amostras por perfil,
mesma vista azul e preenchimento selecionado:

| Perfil | Execução1 | Execução2 |
|---|---:|---:|
| Desligado |1,980ms|2,533ms|
| Suave: força0,5/raio6 |3,360ms|3,394ms|
| Forte: força1/raio12 |3,389ms|3,773ms|

O acréscimo observado foi0,86–1,38ms no suave e1,24–1,41ms no forte. É tempo de
**draw CPU + espera pela GPU**, não GPU isolada nem frame completo de jogo. Há
ruído de sistema: as primeiras medições com perfis agrupados chegaram a inverter
a ordem dos tempos; foram substituídas pelo protocolo alternado e não sustentam
alegação de aceleração pelo filtro. A diferença suave/forte também não deve ser
tratada como curva confiável de custo.

`cost-repeatability.json` contém as duas execuções completas, inclusive ativação
fria, transições e movimento. Na segunda execução, a primeira ativação suave
custou8,47ms (caminho completo); esse custo não entra nas medianas aquecidas.
Os ensaios de movimento cobrem120 posições/luzes por rodada; não constituem
aceite humano de estabilidade temporal para qualquer conteúdo. As imagens de
qualidade cobrem as vistas explicitamente registradas em `verification.json`.
