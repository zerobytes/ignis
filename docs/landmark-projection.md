# Projeção de landmarks

## Decisão anterior à implementação

O estudo da casa mudou apenas a câmera do mesmo mesh em três capturas. A
cumeeira mudou de leitura; o anexo pontudo e a transição larga continuaram.
Falta consultar a projeção de pontos conhecidos sem editar a câmera da cena.

Escolha: consulta explícita `project_landmarks`, com câmera candidata opcional,
pela matemática compartilhada com seleção/render. Alternativas: só capturar
e medir manualmente repete trabalho e mistura câmera com documento; resolver
câmera automaticamente agora esconderia correspondências e hipóteses geométricas.
Não acrescentar solver nem semântica arquitetônica neste incremento.

Checklist estático anterior:

1. Somente comando solicitado; nenhum trabalho adicional por tick/frame.
2. Até 128 entradas/saídas; uma matriz por consulta, sem clonar Scene/mesh.
3. Entradas tipadas e coordenadas finitas; JSON só na fronteira existente.
4. Reutilizar Projection; não criar uma segunda convenção de clipping/pixels.
5. Viewport 1..16384, IDs únicos limitados, mundo dentro dos limites autorais.
6. Nenhum upload, readback, shader ou I/O dentro da consulta.
7. Dispatcher sequencial; nenhum worker, lock, dependência ou rede novos.
8. Consulta sem cache nem escrita; revisão/histórico/poses/PRNG preservados.
9. Testar clipping, perspectiva, reciprocidade do picking, erros atômicos,
   consulta MCP, pixels/histórico/save/load/replay e regressão completa.
10. Custos mantidos: alocação limitada de relatório/JSON por solicitação,
    precisão f32 compartilhada com renderer. Resíduo 2D não certifica forma 3D,
    correspondência, visibilidade ou calibração. Nenhum ganho de FPS alegado.

## Contrato público

```json
{"op":"project_landmarks","width":800,"height":600,
 "camera":{"position":[0,0,5],"target":[0,0,0],"fov_degrees":90},
 "landmarks":[{"id":"origin","world":[0,0,0],"reference_pixel":[397,296]}]}
```

`camera` omitida usa a câmera autoral atual. A câmera candidata não substitui
a da cena. Cada `id` tem 1..64 caracteres ASCII alfanuméricos ou `-_.`, único
na consulta; é um rótulo, não um identificador de objeto. `world` é um ponto
explícito em metros ±10000, já transformado ao mundo pelo chamador. Para meshes
animados, obter a pose no tick desejado antes de fornecer os pontos. A consulta
não descobre features nem acompanha vértices automaticamente.

Pixels contínuos têm origem no canto superior esquerdo, X para a direita/Y para baixo,
sem arredondamento/clamp. `reference_pixel` opcional deve estar dentro da imagem.
O resultado retorna câmera/viewport, samples na ordem de entrada e contagens:
requested, projectable, in_viewport, correspondences, compared, excluded.
As matrizes e clipping são os mesmos de seleção/render: near 0.05 m/far 30000 m,
precisão f32. Estados: in_viewport/outside_viewport para projeções possíveis;
before_near/after_far/behind_camera/non_finite sem pixel. Non_finite é guarda
defensiva da projeção compartilhada; entradas não finitas são recusadas.
Estar no viewport **não informa oclusão**.

Resíduo é predicted−reference. Distância euclidiana, RMSE e máximo usam f64
sobre os pixels projetados f32. Pontos fora deXY continuam comparados. Só
correspondências sem projeção são excluídas; compared/excluded ficam explícitos.
RMSE/máximo são null quando nada pôde ser comparado. Sem score percentual,
pesos automáticos, descarte de outliers, alinhamento ou solver implícitos.

Não muda revisão, tick, histórico, PRNG, câmera, seleção ou persistência.
`expected_revision` conserva a guarda comum. Scene 86/replay 90 permanecem iguais.
CLI, MCP e uma futura UI consomem o mesmo Command, sem endpoint paralelo.

## Ensaio reproduzível

`python3 scripts/study_cottage_projection.py` usa a casa integrada de
artifacts/cottage-normals/examples/cottage-spring.scene.json, em processo
MCP próprio e sem janela. Usa um único executável corrente de main/target,
consulta 10 âncoras autorais em 4 câmeras e guarda capturas/report.json em
artifacts/cottage-camera-study. Verifica que a consulta não muda o estado e
que todas as câmeras usam os mesmos meshes. Os pontos não são correspondências
inferidas da referência: não há RMSE artístico/calibração neste ensaio.

Depois isola a estrutura existente por edições públicas em outro documento:
6 assets/11 instâncias, material neutro e vistas hero/planta/frente/lateral/trás.
Não é uma nova hipótese de reconstrução. Geometria dos assets retidos é
conferida contra a fonte; ornamentos no asset de pedra continuam presentes.
A neutralização também torna os infills opacos, portanto essas vistas não
servem para julgar transparência nem existência de vãos. A vista superior é
perspectiva quase vertical, não ortográfica. Fonte canônica e sessão aberta
permanecem preservadas. Pré-requisito: estudo de integração/normais da casa
já executado, conforme docs/main-consolidation.md.

## Resultado do ensaio

Build f914158cfdfd5f3a, 1536×1024. Mesmos extremos nominais da cumeeira,
de oeste para leste, eixoY dos pixels para baixo:

| Câmera/posição | ΔX pixels | ΔY pixels |
|---|---:|---:|
| baseline [11,8.7,16] |568.761|54.709|
| lower [11,5.2,16] |574.299|-15.181|
| low [11,3.5,16] |570.031|-48.855|
| lower-front [8,5.2,18] |622.353|-10.548|

Target [0,2.65,0.5], FOV34° em todas. O sinal muda sem mover a cumeeira:
a inclinação observada na imagem não prova uma inclinação física. Não houve
seleção de uma câmera calibrada ou correção da geometria. O anexo cônico e a
articulação com o frontão continuam divergentes na inspeção visual.

Fricção adicional observada: tentar remover detalhes por asset retirou apoios
e molduras junto, porque material e parte construtiva são domínios distintos.
O ensaio final conserva o asset carved-stone inteiro. Seleções e ilhas já
permitem editar subconjuntos, mas o documento não guarda os agrupadores
construtivos da receita. Candidato a próximo incremento: conjuntos nomeados
reutilizáveis entre materiais. Antes de persistir esse contrato, definir como
subdivisão, append, exclusão e regeneração remapeiam ou invalidam a associação;
não ampliar o limite de assets para contornar esse problema.

## Verificação concluída

1143 testes Rust/115 alvos release/all-targets, Clippy -D warnings, fmt,
build e114cenários/486grupos MCP aprovados no build acima. Os4grupos novos
MCP conferem schema, erros atômicos, câmera candidata/default, revisão,
pixels, histórico e persistência/replay. Logs em artifacts/landmark-projection/.
Revisão estática final em performance.md; gates e limites em validation.md.
