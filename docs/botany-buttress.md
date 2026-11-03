# Raízes em paredes com cristas — EAF-198

Continuação posterior: [correção do colo e da aba, Source/Recipe7](botany-collar.md).
O usuário aprovou a melhora das paredes e apontou dois artefatos na base; os
dados/builds abaixo registram a etapa6, anterior à correção.

Revisão após o usuário reprovar a aparência tubular de5e37102. As duas novas
fotos pedem corpo alto e fino, crista arredondada, transição que sobe pelo tronco,
trajetórias curvas e vales entre raízes. A [análise/lista BW01–10 anterior à
construção](references/botany/buttress-analysis.md) separa observações e hipóteses.
Não identifica espécie/escala/idade nem reconstrói contexto ou marcas d’água.

## Perfil de seção opcional

`root_systems[].settings.buttress` é opcional; ausência conserva exatamente a
geometria tubular anterior. Presença exige Source/Recipe6, leitor1..5 mantido.
O perfil está em `src/botany/root_sections.rs`, a geração em `roots.rs`, ambos
nativos Rust. O gerador Python apenas chama o mesmo `SetRoots` e salva curvas.
Não há foto, malha por imagem ou triangulação externa na geração.

| Campo | Faixa | Efeito |
| --- | --- | --- |
| `height` |0..8 | Altura adicional da parede, em raios atuais do tronco |
| `reach` |1..30 | Alcance da queda de altura, em raios atuais do tronco |
| `wall_width` |0,08..1 | Meia espessura do corpo relativa ao raio tubular local |
| `crest_width` |0,08..1,5 | Meia espessura/raio da crista relativa ao raio local |
| `meander` |0..0,35 | Flecha lateral seeded relativa ao comprimento do trecho |
| `fluting` |0..2 | Nervuras do tronco alinhadas às raízes, com queda por altura/distância |

Esses controles não substituem os de quantidade, raio, profundidade, ordens,
inclinação de solo e distribuição. O perfil é permitido no padrão radial;
os padrões pivotante/fibroso anteriores continuam disponíveis sem ele. Não é
uma taxonomia universal nem promessa de gerar toda raiz por cinco números.

Sejam `T` a tangente, `U` o eixo vertical projetado no plano da seção,
`V=U×T`, e `(x,y)` as componentes do círculo material unitário nessas direções.
`r(u)` mantém o afilamento do eixo; `R` é o raio atual do tronco, `d` a distância
horizontal da base e `S` o smoothstep limitado0..1. O perfil é:

```text
F(d) = exp(−(d/(R·reach))²)
H(d) = R · height · variação_seeded · F(d)
W(y) = wall_width + (crest_width−wall_width) · S(max(y,0))
X = r·x · (1+(W−1)·F)
Y = r·y · (1+(crest_width−1)·F) + H·S((y+0,15)/0,30)
Pseção = Peixo + V·X + U·Y
```

A faixa de transição vertical gera o corpo alto; a metade superior conserva
uma crista curva, cujo raio varia separadamente da espessura. É uma única seção
volumétrica, sem cilindro sobreposto ou plano sem espessura. Longe da base,
altura e achatamento tendem ao tubo subterrâneo original. Raízes com perfil
usam pelo menos12 amostras de seção, sem aumentar a resolução da copa inteira.
O cap terminal usa o centro amostrado da seção, evitando um leque a partir
da base de uma parede alta. Não há paredes vazadas/raízes ocas nesta gramática.

A forma segue a escala radial do perfil de idade; alcance também depende dessa
escala e da curva já desenvolvida. Mantêm-se idades autorais, sem calibração de
décadas ou inferência de idade das fotos. O plano do solo é autorado, não um
amostrador automático de terreno.

## Trajetória e encontros

O novo meandro é uma flecha lateral suave e seeded, com envelope `sin²(πq)`;
posição e derivada da flecha desaparecem nos extremos. `q` começa após55% do
trecho primário e20% dos filhos. O trecho proximal estável reserva espaço para
o colo, depois a raiz pode serpentear. A perturbação angular antiga permanece.
Como a parede fornece exposição própria, esse perfil não usa o arqueamento
exposto do centro tubular: o eixo acompanha solo/profundidade, a parede emerge.
Os filhos mantêm a origem comum e suas seções recebem altura por posição e seed;
o kernel preenche a transição entre seções de tamanhos diferentes.

Os colares circulares do kernel continuam definindo a conectividade inicial.
Somente nas junções afetadas pelo perfil, os colares são levados às seções novas
e os deslocamentos interiores estendidos por160 iterações de Laplace com
fronteiras fixas. Há32 passos de homotopia, recusando faces locais colapsadas
ou com inversão detectada entre passos. Fronteiras compartilham índices com
os tubos. Junções inviáveis continuam recusadas em candidata; não há fallback
de tubos atravessados. Isso não certifica ausência de autointerseção global,
boolean/CSG ou anastomose universal.
Continuidade aqui é de posição/índices (C0); tangência no colo permanece uma
aproximação, não um encontro C1 certificado.

O colo busca altura compatível com a parede. O seu prolongamento virtual pode
elevar a copa; os dados de galhos/folhas anteriores são preservados. A resolução
e conectividade nova têm domínio material próprio, incluindo hash da topologia
nas junções. Escultura que perder correspondência recusa; descarte é explícito.
As superfícies de parede/fork/colo são recompiladas e fechadas na poda autoral.

As nervuras usam um campo espacial comum sobre a pele já costurada, em vez de
deformar somente um colar. A tentativa de colar não circular deformado por
Laplace causou inversão local e foi recusada no teste; esse caminho foi retirado.
Para cada direção de raiz `θᵢ`, há um lobo angular suave; `A` reúne o máximo
desses lobos, `fluting` e a queda quadrática por altura. Em coordenadas cilíndricas
da base, a deformação é:

```text
r' = r · (1 + A/(1+(r/R)²))
φ' = φ
y' = y
```

Em cada raio, `dr'/dr >= 1−A/8 >= 0,75` nos limites configurados. Isso evita
trocar a ordem radial no campo contínuo; não é certificado de autointerseção
de qualquer malha discretizada. Tronco, colo, raízes e demais superfícies da
mesma árvore recebem o mesmo campo local, preservando encontros. Acima da faixa
de influência, os pontos são conservados exatamente. Não muda a conectividade
nem aumenta a resolução da copa. A árvore de cada pose é indexada uma vez;
campos de troncos distintos não são aplicados indiscriminadamente entre si.
Offsets de escultura são aplicados em metros após o campo. A pose perto da base
pode mudar além da elevação do colo; não se afirma igualdade dos pixels da copa.

## MCP, painel e compatibilidade

Mesma operação `Botany.SetRoots`, com preview/publicação/cancelamento existentes:

```json
{"op":"botany","operation":{"kind":"set_roots","prefix":"figueira_rs",
 "stem":"tronco0","settings":{"kind":"radial","count":7,"orders":2,
 "spread":9,"depth":1.2,"radius_ratio":0.55,"neck_height":2.5,
 "irregularity":0.30,"buttress":{"height":2.6,"reach":5.5,"wall_width":0.28,
 "crest_width":0.65,"meander":0.18,"fluting":1.2}},"preview":true,"policy":"preserve"}}
```

Vegetação → Sistema de raízes → **Paredes / contrafortes com crista** expõe os
mesmos campos. A quantidade considera o total real de amostras do tronco e
raízes: no máximo160 por colo/13 colares, além de12 primárias e16 sistemas por
fonte. Demais quotas combinadas de fonte/malha/ensaio/contato permanecem.
Scene109/replay119/conteúdo7/protocolo5/checkpoint3/PSV1 não mudam. Desativar o
perfil retorna à família tubular; mudar a configuração regenera seu subgrafo
com as mesmas políticas de edições/escultura e recusa de perda.

## Física e performance

O ensaio continua uma lattice reduzida de eixos, com seções transportadas;
não vira simulação de tecido da parede. Massa e rigidez por área usam uma
quadratura projetada de24 pontos da seção nova, incluindo o campo de nervuras.
Proxies usam raio amostrado da seção; não certificam envelope contínuo nem
colisão exata da parede ou da junção. Âncoras distais
continuam explícitas e raízes não são projetadas ao plano de chão. Sem solo
constitutivo, nutrição, erosão, escavação ou física de gameplay implícita.

O teste da parede alta revelou distorção de repouso: a interpolação do raio da
lattice era dividida pelo raio analítico, em vez de pela própria seção de repouso
interpolada. A normalização agora compara spans atuais/de repouso e aplica a
fração de expansão. Ativar o ensaio conserva os pontos no repouso testado; não
se afirma identidade de trajetórias XPBD antigas após corrigir essa distorção.

Revisão estática: sem geração no frame estável, I/O/rede ou novo passe/shaderGPU.
Perfil e campo por Pose são opcionais em Arc, evitando embutir seus dados em
cada folha. O campo é compartilhado por árvore, e ignora pontos acima da faixa
ativa. Buffers da deformação limitados à junção; quadratura de área usa array
na pilha e ocorre na preparação, sem alocação por ponto. A expansão do grafo e
as cópias autorais/bake completo mantêm custo real. Novas seções não multiplicam
a resolução da copa. Não há garantia de FPS. Preservar mundo/janela humanos
e um executável corrente; sem cópias históricas de builds.

## Conferência

A [galeria nativa](../artifacts/botany-buttress/review.html) e o relatório local
registram as capturas, configuração, comparação e pendências. Teste crítico
`buttress_walls_have_closed_skin_distinct_crest_and_preserved_rest_state` cobre
seção mais alta que espessa, pele fechada/orientada, determinismo/JSON, poda de
um filho, recusa de versões/escultura e repouso/identidade/massa no ensaio.
O corpus anterior continua responsável pelos contratos legados.

Reprodução após build release:

```sh
python3 scripts/demo_botany_buttress.py
# Conferir corpo cinza antes de aparência:
python3 scripts/demo_botany_buttress.py --appearance-only --publish-source
python3 scripts/botany_exemplars.py --only figueira_rs
cargo test --release --test botany
```

Na iteração de autoria, `--reuse-before` reaproveita os PNGs da base5e37102;
a execução final sem essa flag refaz a base na mesma câmera/luz. Comparação RGB
antes/depois mede mudança, não fidelidade às fotos de câmeras diferentes.

## Resultado desta revisão

Build final macOS arm64/Metal `fbcd469e2818ebe6`, fonte main, release corrente
`target/release/ignis`. Corpo cinza inspecionado em frente, lateral, verso,
superior e rede sem solo; só depois foi restaurada a aparência. A galeria inclui
o tubo anterior, a parede alta e outra configuração com altura1,3/largura0,4/
crista0,65/meandro0,12/nervuras0,5. A câmera/luz de cada par são iguais; comparação
às fotos é visual, sem escala/alinhamento calibrados.

| Ficha | Fonte/objeto e conferência |
| --- | --- |
| BW01–02 | `tronco0/root*`, seção nova: paredes com espessura e crista convexa observadas em frente/lateral. A crista ainda tem leitura mais regular que a foto. |
| BW03 | Colo e campo `fluting`: encontro contínuo e nervuras prolongadas no tronco; base ainda mais cônica/circular e menos sulcada que a referência. Parcial/divergente em aparência, não aprovado como réplica. |
| BW04 | Campo de altura/alcance: parede baixa rumo ao solo, primárias e filhos com alturas diversas. Variante baixa confirma variação de seção. |
| BW05–06 | Curvas seeded e `fork`: meandros/bifurcações reais, fechamento após poda e vista sem solo. Curvas ainda têm repetição e ombros mais regulares que as referências. |
| BW07 | Vales abertos e solo entre paredes, confirmados também na superior; a coroa central ainda ocupa área mais uniforme que na foto. |
| BW08 | Material procedural anterior aplicado depois da revisão de forma. Sulcos/rachaduras/desgaste fino da foto não foram reproduzidos; marcas d’água excluídas. |
| BW09 | Perfil opcional, defaults antigos preservados. Source5 de5e37102 reaberta/recompilada com PNG exato (`legacy-rebake.json`). Painel compilado/API comum, interação GUI manual pendente. |
| BW10 | Mesmos36.351 órgãos anteriores da copa em dados normalizados;49 raízes regeneradas. Pose próxima da base/altura muda pelo colo/campo. Outros quatro assets não alterados. |

`report.json`:36.400 órgãos,28.416 folhas,2.884.806v/5.644.424tri; apenas
972v/1.944tri adicionais sobre o exemplar tubular5e37102, sem aumentar a copa.
Fonte37.101.362bytes (~35,38MiB) de curvas/órgãos explícitos; não é uma gramática
compacta nova. Publicação completa medida neste roteiro em2,79s, sem inferir FPS.
Mesmos tetos65.536 órgãos/96MiB fonte/3M vértices/6M triângulos; ensaio
1.572.864 nós/6M restrições e quotas próprias de contato.

18 testes Rust botany release passaram, incluindo caso de parede/idades8/20/80,
forquilha no tronco/poda, repouso e domínio de escultura. Corpus repetido após
salvar a fonte6 canônica; Clippy `-D warnings`, fmt, Python/links/diff verificados.
Não foi necessária alteração do kernel `mesh_organic` nem novo teste gráficoGUI.
Aviso transitivo futuro de `block` permanece o anterior.

Ensaio nativo:1.175.104 nós/5.511.516 restrições/672 âncoras, quatro passos com
vento e identidade conservada, vínculo1,30e−9m; poda remove2.034,95kg do material
autoral e conserva o balanço dentro1e−8kg. Massa total34.250,53kg é de parâmetros
e integração reduzida, não uma medição da árvore real. Save/load da cena adulta
preserva PNG exato; regeneração pelo script preserva bytes/hash canônicos
(`regeneration.json`). Estado de ensaio continua fora da persistência/gameplay.

Tentativas relevantes: meandro proximal inviabilizou colares e foi recusado;
saída primária estável e altura pela seção resolveram o encaixe. Deformar apenas
o colar de nervuras também foi recusado por inversão; campo espacial compartilhado
resolveu o teste sem remover a guarda. Testes aprovados não substituem aceite
artístico: colo/sulcos/assimetria fina e interaçãoGUI ainda pendentes. Mundo e
janela humanos preservados; um release atual, executável debug próprio descartado
após confirmar sem uso. Sem push/publicação.
