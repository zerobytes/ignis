# Sistemas de raízes procedurais — EAF-198

Continuação posterior: [paredes com cristas, Source/Recipe6](botany-buttress.md).
O usuário reprovou a aparência tubular desta primeira etapa; valores/builds
abaixo registram5e37102 e não constituem aceite visual.

A figueira RS passa a ter raízes grossas, ramificadas e parcialmente expostas,
unidas ao tronco por uma pele contínua. Havia raízes aéreas autoradas na figueira
ornamental; faltavam geração basal, colo próprio e parâmetros de solo. A nova
operação é genérica e nativa. As fotos orientaram a leitura visual, sem entrar
na geometria ou no material. Espécie, escala e idade reais não foram confirmadas.

A [análise com recortes e lista RT01–10](references/botany/root-system-analysis.md)
precede a construção. A [galeria nativa](../artifacts/botany-roots/review.html)
tem antes/depois, corpo cinza, folhas, frente/lateral/verso/superior e rede sem
solo. O verso e toda a rede subterrânea são hipóteses autorais.

## Matemática e dados

`source.root_systems` contém até16 sistemas, cada um associado a um eixo de
primeiro nível por `stem`. `src/botany/roots.rs` gera eixos explícitos com nove
controles de curva, seção, material mecânico e parentesco. Não exporta triângulos
nem depende de Python no renderer. Os eixos continuam editáveis com as operações
de órgãos e escultura existentes; mudar a configuração regenera o seu subgrafo.

Sejam `R` o raio autoral do tronco, `n` a quantidade de raízes principais, `d` as
ordens adicionais e `ξ` números determinísticos em `[−1,1]`, derivados da seed
da fonte XOR seed do sistema e do ID. Para a raiz principal `i`:

```text
θᵢ = 2π · (i + irregularity · ξθ) / n
Lᵢ = 0.55 · R · spread · (1 + 0.8 · irregularity · ξL)
rᵢ = R · radius_ratio · (1 + 1.5 · irregularity · ξr)
Lfilho = Lpai · length_ratio
rfilho = rpai · 0.8 · 2^(−1/2.3)
θfilhos = θpai ± branch_angle
eixos = n · (2^(d+1) − 1)
```

A perturbação angular varia progressivamente ao longo da curva. A raiz primária
usa `connection: basal`, inserção0; cada bifurcação usa dois filhos `fork` na
extremidade do pai. Raios têm piso geométrico0,0012 em unidades da fonte; seção
terminal afila até8%, seção antes da bifurcação até80%, potência de afilamento1,2.

O plano de solo local é `h(x,z)=ybase+soil_height+sx·(x−xbase)+sz·(z−zbase)`.
Sendo `q` a progressão hierárquica até a terminação, a altura alvo radial é:

`y(q) = h(x,z) + exposure·r·(1−exp(−8q)) − R·depth·q²`.

Cada trecho compensa linearmente o desvio da sua origem para garantir posição
contínua. As bifurcações são unidas pelo kernel existente, não apenas encostadas.
O modo `taproot` exige uma raiz principal, quase vertical, usa profundidade
linear em `q` e depois bifurca; `fibrous` remove o arco exposto. São padrões
autorais, sem garantia de reproduzir todas as anatomias pivotantes/fasciculadas.
O painel fornece valores iniciais distintos para esses modos; a API permite
configurar os mesmos campos explicitamente.

| Campo | Faixa / significado |
| --- | --- |
| `kind`, `count`, `seed` | radial/taproot/fibrous;1..12 primárias (pivotante1); seedu64 |
| `spread`, `depth` |2..40 e0,1..40, múltiplos de `R`; spread controla comprimento inicial, não raio total exato |
| `radius_ratio`, `neck_height` |0,03..0,9 e0,5..5, relativos a `R` |
| `exposure`, `irregularity` |0..1 e0..0,35 |
| `orders`, `length_ratio`, `branch_angle` |0..3;0,3..0,85;0,2..1,2rad |
| `soil_height`, `soil_slope` | altura em unidades da fonte; inclinações X/Z em[−1,1] |
| `anchored_fraction` |0..1, fração distal dos eixos terminais fixada no ensaio |

O perfil de idade existente escala extensão/profundidade das raízes pela copa,
seção pelo crescimento radial e expansão pela vida do órgão. A idade é uma
avaliação autoral do perfil, sem calibração empírica ou simulação de décadas.
Há vida `role: root` quando a fonte tem cronologia. Sem perfil, mantém-se escala
unitária. Filhos recebem nascimento por ordem; não há outro relógio biológico.

## Colo contínuo

Um colo virtual vertical é acrescentado no bake ao tronco, com altura
`R·neck_height·scale·age.height`. O polígono de controles é reamostrado por arco
antes dos frames para evitar overshoot na transição curta. Isso pode elevar a
copa; os dados originais dos seus órgãos não são reescritos. As raízes seguem a
translação da base, mantendo o referencial autoral do solo em vez da inclinação
do tronco. O flare circular legado é desativado somente nos troncos com sistema.

`mesh_organic::junction_many` aceita2..13 colares e no máximo160 amostras de
fronteira. O colo procura cortes métricos compatíveis com as outras junções,
prioriza a altura configurada, recusa sobreposição de janelas e colares não
circulares. Usa envoltória, dois refinamentos e duas relaxações limitadas, com
verificação de topologia/orientação/volume. A superfície tem Euler `2−N` antes
de costurar os tubos. Os índices das fronteiras são compartilhados; a parte
inferior fecha geometricamente, inclusive na vista sem solo. Encaixe inviável
recusa a candidata. Não é uma união booleana/SDF universal, anastomose ou prova
de ausência de qualquer autointerseção global.

A API antiga de três colares permanece. Recompilar a fonte4 de `ee71675` sobre a
cena antiga salva preservou os pixels PNG exatos no novo build; evidência em
`artifacts/botany-roots/legacy-rebake.json`. Remover um sistema sem ajustes também
restaura a malha anterior exatamente no teste reduzido.

## Autoria, transação e persistência

MCP e Vegetação → **Sistema de raízes** usam a mesma `Botany.SetRoots`:

```json
{"op":"botany","operation":{"kind":"set_roots","prefix":"figueira_rs",
 "stem":"tronco0","settings":{"kind":"radial","count":7,"spread":6,
 "depth":2,"radius_ratio":0.6,"neck_height":1.25,"exposure":0.55,
 "orders":2,"irregularity":0.25},"preview":true,"policy":"preserve"}}
```

Omitir campos de settings usa defaults documentados no schema nativo; o exemplo
acima usa os demais defaults. `preview:false` publica na transação existente;
`settings:null` remove o sistema. O painel pré-visualiza ao soltar o controle,
permite aplicar, cancelar e descartar ajustes explicitamente. A interface
compilou e compartilha o dispatcher testado; não se infere aceite de interação
manual com o painel, que não foi aberto nesta validação.

IDs reservados `stem/rootN/fN` têm dono basal validado; colisão ou ancestral
incompatível recusa. Fonte/receita5 são exigidas para os novos dados; leitura1..4
continua. Scene109/replay119/conteúdo7/protocolo5/checkpoint3/PSV1 permanecem.
Os detalhes do colo têm domínio material por hash de conectividade, resolução,
portos e cortes. Alteração que perder uma escultura recusa atomicamente; descarte
é explícito. Remover/regenerar raízes com edições de órgãos inexistentes também
recusa em preserve. Não se promete migração silenciosa de esculturas topológicas.

Poda autoral reavalia o colo e fecha a pele restante. A poda do ensaio mantém o
contrato anterior de máscara/massa, sem novo tecido cicatricial dinâmico. As
raízes saem da projeção simplificada do chão para poder existir abaixo dele;
somente as frações terminais configuradas ficam presas ao repouso. A base do
tronco conserva sua fixação antiga. Isso não demonstra suporte de uma árvore
livre pelo solo, rigidez constitutiva do terreno, contato de gameplay, escavação,
nutrição, erosão ou resposta a obstáculos. `root_anchor_nodes` expõe as âncoras.

## Exemplar e conferência

`figueira_rs.ignis-botany`:7 raízes primárias, duas ordens adicionais,49 eixos
novos; demais valores no exemplo acima, seed0, razão de comprimento0,65,
ângulo0,60rad, âncoras0,30 e solo plano. Os36.351 órgãos anteriores permanecem
idênticos em dados normalizados; o exemplar passa a36.400 órgãos,5.104 eixos
lenhosos sem epífitas,28.416 folhas e2.880 pendentes. Fonte nativa compactada
em JSON sem espaços:37.104.834bytes (~35,39MiB); a redução de bytes em relação
ao JSON indentado não é uma gramática compacta nova.

| Ficha | Conferência final |
| --- | --- |
| RT01 | Colo fechado/orientado, conectado; aba inclinada removida. Frente/lado/verso inspecionados. |
| RT02–03 | Raízes grossas distintas e afiladas, irregularidade seeded e partes enterradas; número/verso autorais. |
| RT04 | Transição lobulada e intervalos de solo presentes; ainda faltam contrafortes altos, sulcos e assimetria da referência. Sem equivalência artística afirmada. |
| RT05–06 | Forquilhas e rede subterrânea reais, visíveis sem solo; anatomia oculta é hipótese. |
| RT07 | Campo de inclinação implementado, teste de variantes em plano; galeria final usa solo horizontal. Sem adaptação automática ao terreno. |
| RT08 | Seed/configuração nativos, três padrões e idades8/20/80 verificadas em caso reduzido;100anos autorais na RS. |
| RT09 | Dados da copa preservados, comparação da árvore inteira; colo pode elevar sua pose. Outros quatro assets não foram alterados. |
| RT10 | Operação comum e controles implementados; preview/atomicidade/escultura/persistência testados; interação GUI e aceite artístico humanos pendentes. |

## Evidências e custo

Build macOS arm64/Metal `0ac2af39a68650b8`, fonte `main`, binário corrente
`target/release/ignis`.17 testes botany e11 mesh_organic passaram, Clippy com
`-D warnings` aprovado (aviso transitivo futuro de `block` não é novo). Provas
críticas novas cobrem radial/pivotante/fibroso, pele fechada/orientada,
determinismo/JSON, poda parcial, remoção, idade, versões, pureza de preview,
publicação atômica, preservação de massa/vínculo e recusa de perda de escultura.
O corpus botany foi repetido após atualizar a fonte canônica.

O ensaio nativo completo contém1.175.104 nós/5.511.516 restrições,672 âncoras;
quatro passos com vento, identidade estável e erro de vínculo9,23e−9m. Podar a
primeira raiz remove1.136,56kg do material autoral, com balanço de massa dentro
de1e−8kg. Não é massa medida da árvore real. Save/load de cena preserva PNG
exato. Regerar o exemplar pelo script preserva bytes/hash da fonte com raízes.
`report.json`, `regeneration.json`, logs e PNGs estão na galeria local.

Malha adulta:2.883.834 vértices/5.642.480 triângulos, acréscimo de10.308v/20.616tri
sobre a base anterior. Carregamento/compilação/publicação da candidata completa
medido neste roteiro em2,72s; não é benchmark de FPS/latência do painel. Mesmos
tetos combinados:65.536 órgãos/96MiB fonte/3M vértices/6M triângulos;
ensaio1.572.864 nós/6M restrições, contato com quotas próprias. A coroa adiciona
limites de16 sistemas/12 primárias e160 amostras por junção, portanto
`count <= min(12, floor(160/sides)−1)`.

Geração/bake são trabalho de autoria em candidata, não por frame. Clone de fonte
usa o Arc existente; modificar exige cópia dos órgãos e bake completo antes da
publicação. O kernel multicolare usa buffers limitados por junção; a expansão
por ordens é exponencial e limitada. O ensaio guarda um bool por nó e calcula
âncoras/parentes na preparação; não busca as receitas por passo. Sem novos
passes/shadersGPU, I/O ou locks no frame estável, rede ou cache global. Custo de
fonte/malha/simulação permanece real. Mundo/janela humanos preservados. Um único
release corrente; sem push/publicação ou promessa de FPS/física de gameplay.

Reprodução:

```sh
cargo test --release --test botany --test mesh_organic
cargo clippy --lib --bin ignis --test botany --test mesh_organic -- -D warnings
cargo build --release
python3 scripts/demo_botany_roots.py
# Depois de conferir a geometria cinza, aplicar aparência e verificar persistência:
python3 scripts/demo_botany_roots.py --appearance-only --publish-source
python3 scripts/botany_exemplars.py --only figueira_rs
```

`--without-roots` no gerador retorna explicitamente à geração anterior; normalmente
o gerador reaplica pelo Rust os sistemas salvos no asset de destino, sem apagá-los.
`scripts/apply_botany_roots.py` permite aplicar uma configuração JSON a outra
receita pelo mesmo MCP nativo e só grava o destino após sucesso.
