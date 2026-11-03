# Diagnóstico das duas demonstrações de floresta — EAF-181

Em01/11/2026, a apresentação abriu a paisagem procedural ampliada de20×20km,
`artifacts/expanded-world/world.scene.json`. Ela não é a mesma configuração de
`artifacts/landscape-forest-dense/forest.scene.json`, usada antes dos estudos
procedurais. O pedido de demonstração não deixou essa distinção explícita.
Nenhum ajuste de renderer foi feito nesta investigação.

## O quadrado vem da população

`artifacts/expanded-world/placement.json` define duas cotas rígidas em
`forest_zones`, consumidas por `scripts/build_representative_forest.py`:

| Região | Área bruta | Árvores | Densidade por km² bruto |
|---|---:|---:|---:|
| X/Z entre−1000e1000m | 4km² | 12.000 | 3.000 |
| Restante dos20×20km | 396km² | 18.000 | 45,45 |

Conferência das posições realmente salvas confirmou12.000dentro/18.000fora.
Portanto40%da população ocupa1%da área e a densidade bruta muda66× no limite.
Água/declividade/clareiras afetam a área plantável; o fator66 usa área bruta.
A borda quadrada é da receita de distribuição. Não demonstra falha de culling.
A quota preservava o miolo do teste anterior na expansão; não é uma distribuição
natural concluída para apresentar o mundo inteiro. Corrigir a distribuição pede
transições espaciais sem quota retangular, mantendo o total declarado e verificando
raízes, água, declividade e colisão; não alegar ganho do renderer por deslocar carga.

## O que mudou no custo

As duas cenas têm30mil árvores e geometrias de tronco/folhas exatamente iguais:
11.140+9.152=20.292triângulos por árvore. São608.760.000triângulos nominais de
instâncias, não geometria duplicada residente nem quantidade efetivamente desenhada.

O terreno/água da demonstração anterior soma121.002triângulos por instância;
o novo soma2.264.194, cerca18,7×mais. Não confundir com o total de cena: incluindo
todas as árvores, o total nominal aumenta só de608.881.002para611.024.194.
O terreno novo também é composto por quatro assets únicos; a GPU mantém o fluxo
expandido deles, enquanto assets estáticos repetidos usam o caminho residente
compacto/indexado. É uma oportunidade adicional de investigação, não causa de
FPS quantificada por um ensaio isolado nesta rodada.

A diferença atmosférica é verificável:

| Parâmetro de camadas | Demonstração anterior | Ampliada20km |
|---|---:|---:|
| detail_start | 800m | 500m |
| detail_end | 3.200m | 1.800m |
| influence | 1,0 | 0,65 |

Ambas mantêm density0,00006, reduce_details=true e color_error0,002. As cores
atmosféricas também diferem. O multiplicador de material em `Layers::detail`
tende a `1−influence`: na ampliada restam35%antes da atenuação exponencial.
A mera presença de névoa visual ou do checkbox Otimização não autoriza eliminar
essa contribuição. O compilador conserva a fonte quando o limite de cor não passa.

Contadores nativos confirmaram17.906peças substituídas e17.347com nível geométrico
na câmera da cena anterior, contra zero/zero nas três vistas ampliadas. A oclusão
GPU e a visibilidade estavam ativas em todas; os símbolos ICB1 estavam OFF em todas.
O contador de uploads dos assets compartilhados permaneceu2 nas três vistas
ampliadas; não houve sinal de reenvio contínuo das malhas de árvore. Esse contador
não mede uploads ou comparações de memória do fluxo legado do terreno.

## Medição nativa e limites

No mesmo executável corrente, Metal/M4Pro, editor pausado, iluminação da cena,
viewport maximizada2784×1560, Immediate e medição contínua:

| Cena/vista | FPS | ms/frame |
|---|---:|---:|
| Ampliada, panorama | 6,57 | 152,12 |
| Ampliada, aérea | 10,88 | 91,91 |
| Ampliada, próxima | 11,73 | 85,23 |
| Anterior, câmera original | 16,94 | 59,04 |

São amostras de cadência de apresentação de até1s, não GPUtimestamps ou uma
regressão controlada de código. A janela ampliada foi minimizada ao medir a
anterior. Confirmação visual posterior da anterior mostrou19,9FPS/50,24ms na mesma
resolução. Diferenças de câmera/carga permanecem: não foi isolada a parcela de cada
causa. A resolução atual possui2,25×os pixels dos ensaios1728×1117; a comparação
nativa acima mantém a resolução igual entre cenas e não atribui tudo a esse fator.

Um diagnóstico separado conservou a câmera/objetos da ampliada e variou somente
a luz: com influência0,65 houve zero substituições; com1,0/mesma redução ligada,
houve13.523substituições/12.932níveis geométricos em1280×800. Isso confirma o bloqueio
por configuração. Nenhum FPS foi medido nesse ensaio de captura.

Restaurar a influência altera a aparência. Além disso, comparar geometria completa
contra redução, ambas com influência1,0, revelou3.516pixels alterados e máximo134RGB8
nessa câmera. A média0,00735RGB8 sobre o quadro não prova fidelidade. Não foi usada
essa troca para declarar um ganho com imagem preservada ou alterar a cena fonte.
Essa falha de imagem do ATM nesta vista também precisa ser tratada, separadamente
dos símbolos ICB1. Aceites anteriores em outras vistas não são garantia universal.

## Evidências e apresentação

- `artifacts/expanded-world/presentation-verification.json`: três vistas ampliadas.
- `artifacts/expanded-world/presentation-original-verification.json`: cena anterior.
- `artifacts/presentation-diagnosis/atmosphere.json`: três capturas controladas.
- `artifacts/presentation-diagnosis/{current,full,reduced}.png`: referência e reduções.
- `artifacts/expanded-world/presentation-original-editor.png`: composição do editor.

A apresentação anterior foi deixada visível, sem personagem e com orbitar/pan/zoom,
usando o bundle existente Ignis Play em modo editor para separar sua identidade
das janelas de personagens. Fonte e geometria das duas cenas originais preservadas;
cópias de apresentação são arquivos locais ignorados. Não houve build, pipeline
novo, redução da população ou publicação nesta investigação. EAF-181 permanece
em andamento; geração20km/demonstração e comparação de render precisam ser mantidas
como fixtures distintas para não misturar ganhos, configuração e conteúdo.

## Continuação

A oportunidade de residência dos terrenos únicos foi implementada e medida em
[Residência de malhas estáticas únicas](static-residency.md). Os números acima
continuam sendo o diagnóstico histórico anterior a essa melhoria; não representam
uma nova medição de FPS após a alteração.
