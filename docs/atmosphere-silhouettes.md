# Camadas de cor e contorno real — EAF-169

A aparência nítida segue como padrão. [EAF-170](atmosphere-edge-softness.md)
acrescenta suavização opcional e versiona Scene102/replay112/conteúdo4; as
medições históricas abaixo são do caminho sem esse acabamento.

Comparação atualizada no build70038ee07b714dc4, com pacotes removidos da
referência e blur ausente: [sem/com otimização](render-optimization-comparison.md).
Memória/pixels repetiram; os tempos novos incluem regressões e não sustentam
um ganho universal. Não substituir o comparativo atual pelos ganhos históricos abaixo.

As referências de Guilherme mostram montanhas escuras contra um céu claro. O
detalhe interno perde contraste antes do contorno. A paleta de EAF-168 escurecia
também o fundo e não atendia a essa direção visual, mesmo com erro baixo entre
referência e otimização. Este recorte corrige a aparência de ambos os caminhos.

## Aparência autorável

`light.atmosphere.layers` é opcional, com cores RGB lineares `sky_color` e
`mountain_color`, distâncias em metros `detail_start`/`detail_end` e
`influence` de 0 a 1. A UI expõe os mesmos campos em **Iluminação → Atmosfera da
cena → Separar céu e silhuetas distantes**; o MCP usa `set_light` pelo mesmo
dispatcher. Não há partículas, nuvens ou nova geometria atmosférica.

Para distância `d`, o peso dos detalhes é
`R = 1 - influence * smoothstep(detail_start, detail_end, d)`.
O preenchimento usa `R * material + (1-R) * mountain_color`; depois recebe a
mistura com a cor do ar pela transmitância `T = exp(-density * d)` existente.
O céu usa sua própria cor, com a influência direcional solar. Assim, as montanhas
podem perder textura mantendo contraste contra o céu e clarear mais adiante.

São controles artísticos, não uma simulação física de umidade. Um sistema de
clima pode dirigir densidade, influência, intervalo e cores. A fotografia sozinha
não fornece uma distância ou umidade calibrada. Os valores da demonstração são
uma escolha autoral, não uma regra universal de que toda montanha perde detalhes
a uma distância fixa. O perfil afeta as superfícies da cena; ainda não há
exclusão por objeto ou volume climático local.

Ausência de `layers` preserva o modelo anterior. Densidade zero conserva o
legado quando `layers` está ausente; com o novo perfil, apenas desliga o
clareamento pelo ar. Desativar `reduce_details` conserva exatamente a aparência
autoral e fornece a referência com geometria e materiais completos.

## Economia e certificados

O seletor multiplica o limite do erro de material por `R` e `T`, usando a menor
distância conservadora da caixa orientada. Quando o erro cabe no orçamento, o
shader barato evita normais, material procedural e consultas de iluminação
complexa. Vértices compartilhados de 24 B e índices u32 substituem a expansão
de 64 B por vértice de triângulo. O preenchimento ainda calcula distância e cor;
não é custo zero nem reprodução de pixels pré-calculados.

ATM3 acrescenta um nível compilado por regiões a campos de altura reconhecidos:
regiões de até 4×4 células **exatamente horizontais** são retrianguladas mantendo
todos os segmentos do perímetro. As demais células permanecem completas. Esse
nível tem deslocamento geométrico zero; não pode inventar uma silhueta nova,
fechar buracos ou romper uma borda compartilhada. Os demais níveis continuam
submetidos ao limite de projeção em pixels físicos e ao limite de cor.

O compilador só inclui esse nível adicional se seu tamanho binário for menor
que os bytes de geometria GPU economizados em uma instância. Isso evita
armazenar quase outra cópia de uma montanha para cortar poucos triângulos.
O critério não promete economia de RAM total ou de tempo.

Isso é um primeiro caso conservador, não um simplificador universal de interiores
curvos. A seleção continua por instância; uma região curva com erro alto ainda
pode impedir um nível mais agressivo. Não há garantia de igualdade de rasterização
em toda câmera só por haver equivalência matemática da superfície.

ATM2 continua legível e sua derivação original continua disponível para validar
pacotes antigos. ATM3 mantém posições f32/índices u16 e até cinco níveis. Pacotes
são binários; base64/JSON ficam na borda de autoria. Câmera/luz não recompilam.
Scene101/replay111/conteúdo visual3; protocolo4/checkpoint2/PSV1 preservados.
Formatos antigos não podem carregar os novos campos ou um certificado ATM3.

## Contorno de seleção

Uma máscara R8 de cobertura é produzida pela geometria realmente selecionada,
usando a profundidade da cena. Um traço interno de dois pixels físicos acompanha
a borda externa e os vazios visíveis. Objetos na frente escondem o contorno;
não é raio X. Meshes usam os vértices já posados, e primitivas usam sua geometria
Box/Wedge. Não se afirma contorno de avatar articulado por seleção de ator.

`select_editor_mesh` e a seleção humana alimentam o mesmo estado; `capture_editor`
mostra o mesmo resultado. Gizmos e modos de componentes/arestas têm apresentação
própria. A malha selecionada conserva a fonte completa para evitar diferença entre
máscara e superfície. A máscara é ferramenta de editor e adiciona trabalho para
o objeto selecionado; não se contabiliza isso como economia da paisagem.

Só há passes de máscara/contorno com seleção ativa. A textura tem um byte por
pixel, é recriada ao redimensionar e liberada ao limpar a seleção. Posições 12 B
ficam em buffer reutilizado; sua capacidade pode permanecer após desselecionar.
`capture.adapter.selection_outline` expõe contagens e capacidade; não inclui
memória de driver. A profundidade é armazenada para este passe apenas quando
há seleção. Não há leitura de pixels para CPU no desenho interativo.

## Reprodução e limites

- `python3 scripts/verify_atmosphere_landscape.py`: pares com a mesma aparência,
  cores azul/dourada/roxa, menor influência, câmeras e luzes distintas.
- `target/release/examples/atmosphere_landscape_cost`: custo total de desenho e
  conclusão, além da parte CPU; não é timestamp GPU nem FPS de jogo.
- `target/release/examples/selection_outline`: malha perfurada, oclusão, câmera
  móvel e retirada da seleção; PNGs e invariantes de cobertura nativos.
- `python3 scripts/verify_atmosphere.py`: legado, formatos, aproximação e retorno.

O limite de 30 km do renderer e posições locais limitadas continuam adiados por
decisão do usuário. A fonte CPU permanece residente; não foi implementado
streaming de texturas ou descarregamento de gameplay. Sombras/IBL/reflexos e rigs
conservam os fallbacks anteriores. Validação técnica não substitui aceite visual.


## Resultado validado em 31/10/2026

Build `b94c889e498dd0d3`, macOS arm64 / Metal / M4 Pro. Sete vistas estáticas
(azul, maior/menor influência, lateral, elevada, pôr do sol e roxo) e 12 posições
da câmera com luz variável: **zero canais diferentes da referência completa**
em 1728×1117 pixels. A trajetória atravessa planos com duas ou três instâncias
substituídas. É evidência dessas amostras, não prova de toda câmera/malha.

Referência e otimização usam o mesmo relevo, material e perfil de aparência.
Céu claro e preenchimento escuro são uma nova direção autoral aplicada aos dois;
o erro zero não prova por si só fidelidade às fotos. PNGs foram inspecionados
nativamente; aceite artístico humano continua separado.

| Medida da paisagem azul | Referência completa | Seleção |
| --- | ---: | ---: |
| Triângulos | 121.002 | 121.002 |
| Buffers de geometria GPU alocados | 22,25 MiB | 10,78 MiB |
| Payload de geometria | 22,16 MiB | 10,54 MiB |
| Pacotes binários compartilhados | 200.980 B | 200.980 B |
| Instâncias com preenchimento barato | 0 | 3 |

Economia de capacidade de geometria **51,5%**. Os seis perfis com influência
forte têm esses mesmos valores; o controle de pouca influência conserva todos
os detalhes e 22,25 MiB. Não são medições da RAM total, memória física liberada
pelo driver ou FPS. Fonte CPU, profundidade e targets não entram nessa tabela.

O primeiro candidato ATM3 guardava 782.836 B e só eliminava 416 triângulos
(0,34%) na paisagem azul. Foi rejeitado pelo critério de bytes do compilador.
A carga final conserva 121.002 triângulos; reduzir curvas internas sem alterar
cristas continua sendo o gargalo geométrico. No controle plano independente,
2.048 → 1.024 triângulos, com erro geométrico certificado zero e diferença
máxima de 1/255 nos três renders. Esse ganho não é extrapolado às montanhas.

### Tempo

Duas execuções completas; cada uma com quatro rodadas alternadas, 12 aquecimentos
e 40 amostras por caminho/vista. Sem cargo/rustc ou outros probes concorrentes
na medição; a janela humana e o desktop permaneceram ativos. Os valores são
medianas das quatro medianas, em ms, **completo → selecionado**. Tempo total
é CPU de draw + espera bloqueante pela conclusão; não é timestamp GPU.

| Perfil | Total execução 1 | Total execução 2 | CPU draw execução 1 | CPU draw execução 2 |
| --- | --- | --- | --- | --- |
| clear | 6,971 → 7,432 | 7,407 → 7,379 | 1,183 → 1,371 | 1,509 → 1,397 |
| haze | 4,545 → 3,027 | 7,437 → 3,727 | 1,234 → 0,840 | 1,340 → 0,683 |
| mist | 6,433 → 3,706 | 7,395 → 3,724 | 1,348 → 0,667 | 1,292 → 0,666 |
| move | 5,012 → 2,390 | 7,426 → 4,192 | 1,147 → 0,615 | 1,361 → 1,123 |
| high | 4,527 → 2,595 | 7,424 → 4,111 | 0,844 → 0,313 | 1,331 → 1,082 |
| sunset | 7,503 → 3,670 | 7,465 → 3,742 | 1,500 → 0,636 | 1,413 → 0,685 |
| purple | 6,481 → 3,588 | 7,238 → 3,652 | 1,308 → 0,546 | 1,291 → 0,676 |

No azul, o total caiu 33,4% / 49,9%. As durações absolutas variaram bastante;
o controle sem substituição regrediu 6,6% na primeira execução e ficou próximo
na segunda. Não atribuir toda variação a trabalho geométrico ou prometer ausência
de regressão em qualquer frame. Simulação, apresentação, leitura de PNG e preparo
inicial da fonte estão fora do cronômetro.

Movimento: duas sequências de 120 frames por caminho em cada execução. Mediana
das quatro medianas completas 5,894 ms → 3,964 ms (32,7% menor); mediana dos
p95 9,103 → 6,837 ms. Ainda há picos: na segunda execução, pior frame selecionado
11,532 ms contra 9,546 ms do completo. Capturas de qualidade e cronômetro são
ensaios separados da mesma trajetória; não se alega comparação visual de todos
os frames cronometrados.

### Integração e regressões

- 128 testes internos e 20 testes de sete alvos críticos passaram; após o filtro
  final do compilador, os três testes de atmosfera/certificados passaram novamente.
  Riscos novos cobertos: validação/atomicidade, persistência/replay, conteúdo de
  rede e rejeição de perfil/pacote em versão antiga.
- Clippy de todos os alvos passou, e lib/binário/dois exemplos passaram novamente
  na revisão final. Build, fmt, Python e diff limpos. Sem novos testes unitários
  que apenas espelhem controles visuais.
- Seis grupos MCP de atmosfera e sete de editor passaram. A cena Scene100/ATM2
  da entrega anterior carregou com pixels idênticos no renderer novo antes do
  filtro final; esse filtro altera somente a compilação ATM3.
- Probes nativos finais: três vistas de malha perfurada, oclusão e remoção da
  seleção, três vistas planas e captura real do editor. Todo pixel alterado pelo
  outline pertencia à superfície visível selecionada; nenhuma alteração no
  buraco, fundo ou oclusor. Máscara 800×600: 480.000 B; buffer de posições 6.144 B
  alocados, 128 triângulos adicionais. Não houve benchmark de tempo do outline.

Evidências correntes: `artifacts/atmosphere-landscape/verification.json`,
`cost.json`, `cost-current-repeatability.json`, `comparison.md` e PNGs;
`artifacts/selection-outline/{verification,integration}.json`, `editor.png` e
pares de cobertura; regressões em `artifacts/atmosphere/`. As imagens vêm do
renderer nativo. HTML auxiliar não foi executado nesta validação.

Próxima investigação geométrica: representação de interiores curvos com limite
local de contorno/oclusão, para cortar triângulos efetivamente. Não relaxar esse
limite nem guardar níveis de alto custo apenas para reduzir a contagem nominal.
