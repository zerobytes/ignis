# Matemática de plantas procedurais — núcleo independente v1

Continuação implementada: [V2 nos sete eixos](procedural-botany-v2.md), incluindo
folhas palmadas, redes vasculares, junções, copa, dinâmica reduzida, crescimento
e órgãos adicionais. Este documento preserva o recorte e as evidências da V1;
os limites que foram ampliados estão discriminados na V2.

## Complemento: perfil de tamanho de uma folha composta

Pedido de 01/11/2026, após aprovação visual da V1: folíolos pequenos perto da
inserção, maiores logo adiante e progressivamente menores na ponta, formando
a silhueta de uma folha grande. A fotografia fornecida orienta esse padrão;
não extraímos medidas nem identificamos uma espécie a partir dela.

O campo de tamanho da V1 já consulta a posição de nascimento no pai. O
complemento expõe essa possibilidade numa receita de folha pinada, com
folíolos diretamente sobre a raque. A folhagem composta anterior tem dois níveis
(raque → pinas → folíolos) e permanece disponível.

Para `u=s/Lraque`, da base 0 à ponta 1:

    comprimento_folíolo(u) = comprimento_referência · perfil_comprimento(u)
    largura_folíolo(u) = largura_referência · perfil_largura(u)

Os dois perfis são curvas adimensionais editáveis, avaliadas na inserção do
folíolo. Exemplo: comprimento relativo 0,16 na base, 1 em 18% do eixo, 0,94 em 42%,
0,50 em 75% e 0,06 na ponta. A interpolação suave da V1 liga os pontos. Largura
tem curva própria; não exigir escala uniforme nem alterar espaçamento para
conseguir a silhueta. Variação aleatória opcional do `Field` pode ser composta
com esse perfil; o exemplo deixa-a zerada para mostrar só a curva.

Esta coordenada é a do **pai imediato**. Numa folha duplamente composta, um
perfil em `pinna.length` controla o tamanho das pinas ao longo da raque; um perfil
em `leaflet.length` controla os folíolos ao longo de cada pina. Não atribuir ao
segundo a posição da raque ancestral nem usar o `u` interno da superfície da
lâmina como se fosse sua posição de nascimento. O exemplo pinado evita essa
ambiguidade e demonstra diretamente o padrão solicitado.

Revisão estática anterior ao complemento: reutilizar `Field/Curve` e o avaliador,
sem novo estado de órgão/identidade, herança implícita de contexto ou formato.
Preparar as curvas uma vez na receita, avaliar comprimento/largura somente no
nascimento; mesmas filas e quotas. Diagrama comparativo usa seed, eixos,
estações e escala de projeção iguais. Validação direta de medidas/independência,
regressão existente de identidade e matemática; nenhum teste novo de slider.
Sem engine, build, dependências, solver ou trabalho no tick.

Configuração explícita da nova receita, usando a API existente:

```python
from botany_math import Curve, compound_leaf_recipe, generate

recipe = compound_leaf_recipe(
    length_profile=Curve(((0., .16), (.18, 1.), (.42, .94), (.75, .50), (1., .06))),
    width_profile=Curve(((0., .35), (.18, 1.), (.6, .8), (1., .10))),
)
plant = generate(recipe)
```

`Curve()` é um perfil constante de multiplicador 1; fornecer esse valor em
qualquer um dos argumentos mantém aquela dimensão uniforme. As curvas são
editáveis por pontos, inclusive posição do máximo e rapidez de redução.
Multiplicadores precisam produzir dimensões positivas nos nascimentos; zero
não elimina órgãos silenciosamente. Ausência de folíolos se configura na regra
de nascimento/densidade.

Reprodução: `python3 scripts/botany_math.py --example folha_pinada` e
`python3 scripts/plot_botany_math.py`. A CLI passa a oferecer cinco receitas;
o diagrama `compound-profile.svg` compara dois perfis na **mesma escala**, com
a mesma raque, estações, IDs, seed e orientação inicial. `overview.svg` inclui
também a nova receita, mantendo os quatro exemplos iniciais.

Resultado numérico: uma raque e 84 folíolos. O primeiro nasceu em `u=0,01680`,
com comprimento 5,42 cm; o máximo ocorreu em `u=0,18346`, com 30,00 cm; o último
em `u=0,99299`, com 1,83 cm. A inspeção direta confirmou a igualdade entre cada
comprimento/largura e sua curva no ponto de inserção, os mesmos IDs e nascimentos
do comparativo uniforme e a independência dos dois controles. A regressão
existente de estrutura, identidade, orçamento/atomicidade e oráculos numéricos
passou nos cinco exemplos; nenhum teste novo por parâmetro foi acrescentado.
Compilação Python, exemplos documentados e verificação de diff passaram. Figuras
inspecionadas; é uma aproximação autoral do padrão da fotografia, sem ajuste
botânico quantitativo ou integração ao Ignis.

Revisão final do complemento: nenhum campo, formato ou algoritmo novo no
avaliador; o driver `u` de `Field` reutilizado em duas dimensões no nascimento. A receita
compartilha definições, usa as mesmas quotas, contabilizou 11305 unidades de
trabalho e mantém 85 órgãos. As curvas não alteram a distribuição nem geram
novas identidades. O diagrama prepara geometria uma vez por variante e usa um
único envelope de projeção; a amostragem e o I/O explícito de SVG/JSON são os
custos aceitos. Sem dependências, threads/locks/cache global, runtime ou build
novo. Saídas atuais substituídas nos caminhos estáveis; sem histórico de builds.

Direção de Guilherme, 01/11/2026: construir primeiro a matemática; decidir a
aplicação às ferramentas do Ignis em uma etapa posterior. Este documento e o
laboratório `scripts/botany_math.py` não alteram a engine. Aqui, **vegetal** é o
organismo; `src/plants.rs` continua sendo o documento construtivo já existente.

## Escopo e decisões anteriores ao código

Resultado pretendido: uma receita descreve órgãos, regras locais e influências
mundiais; sua avaliação produz uma hierarquia identificada, curvas orientadas,
superfícies consultáveis e grandezas mecânicas. Grama, árvore, arbusto florido e
folhagem composta exercitam a mesma matemática. São arquétipos sintéticos, sem
calibração de espécie nem alegação de representar qualquer planta.

Escolhemos um grafo de órgãos e regras condicionais explícitas. Uma gramática
textual tipo L-system é uma possível interface futura; não é necessária para
definir o modelo. Colonização do espaço pode fornecer um campo de orientação
futuro, mas não substitui as regras de nascimento. Curvas orientadas e lâminas
paramétricas fornecem representação independente de tesselação ou renderer.

Python com biblioteca padrão é uma **referência numérica descartável/portável**,
não um novo núcleo de runtime do Ignis; eventual implementação da engine continua
em Rust. Não congelamos formato de asset, ABI, UI, MCP, rig ou estratégia de LOD.
O JSON exportado é um relatório do laboratório identificado como tal.

Revisão estática prévia proporcional:

1. Avaliar somente por chamada/CLI, sem trabalho no tick ou frame da engine.
2. Usar floats de dupla precisão, vetores pequenos e definições imutáveis;
   compartilhar receitas, sem copiar a árvore para cada órgão.
3. Randomicidade por chave e canal, sem gerador global ou ordem oculta.
4. Contabilizar órgãos, amostras e trabalho antes de expandir; falhar sem saída
   parcial, sem poda silenciosa para fazer uma receita caber.
5. Construir somente curvas e órgãos; amostrar lâminas/nervuras sob demanda.
6. Nenhuma GPU, rede, dependência, worker, lock ou arquivo da engine.
7. Cache só de amostras da avaliação atual, sem cache persistente implícito.
8. I/O explícito para substituir relatórios no diretório corrente.
9. Verificação numérica direta; testes persistentes apenas para identidade,
   atomicidade/recusa e integridade da estrutura/relatório.
10. Registrar aproximações e custo, sem inferir FPS ou simulação física pronta.

## 1. Objetos, unidades e entradas

Receita `R`, semente inteira `z`, raízes explícitas e contexto ambiental `E`
determinam `G = Evaluate(R,z,E)`. `G` é uma floresta enraizada de órgãos. Cada
órgão tem identidade, tipo, pai, coordenada de inserção, frame e definição.
O parentesco é acíclico por construção: filhos referenciam apenas pais existentes.
Regras podem repetir uma definição até o limite de profundidade da avaliação.

Tipos mínimos: **eixo** e **lâmina**. Tronco, galho, caule, pecíolo, raque e raiz
usam eixos com receitas próprias; folha, folíolo, pétala e sépala usam lâminas.
Um conjunto de eixos/lâminas pode representar um órgão composto. Isto não inclui
automaticamente órgãos volumosos como frutos ou caules suculentos.

Convenções: metros, quilogramas, segundos, radianos e pascals; mundo cartesiano
destro, `+Y` para cima. Comprimento `L>0`; posição axial `s ∈ [0,L]` em metros;
posição normalizada `u=s/L`; nível inteiro `d`, começando em zero nas raízes.
Altura mundial `h=p·up` em metros; raio `r(s)` em metros. Transformação de mundo
é entrada, nunca inferida de uma câmera.

Estado de crescimento, configuração de repouso e configuração deformada são
conceitos diferentes. A v1 avalia uma forma de repouso madura. O contrato de
idade e as equações de deformação estão definidos abaixo, mas não constituem
um simulador temporal implementado.

## 2. Campos condicionais e variação

Um parâmetro não precisa ser um escalar fixo. Usamos campos separáveis:

    F(c,id,canal) = b · ∏j Cj(cj) · [1 + a(2U(z,id,canal) − 1)]

`b` carrega a unidade física; curvas `Cj` e amplitude `0≤a<1` são adimensionais.
As entradas `cj` podem ser `u`, `height`, `radius`, `level`, `parent_length`.
Nós das curvas usam a unidade da entrada: um limiar de raio é em metros,
um limiar de nível é inteiro. Não normalizar altura por um bounding box que
muda com a própria geração. Curvas de densidade precisam ser não negativas.

Entre nós `(xi,yi)`, `(xi+1,yi+1)`, com `t=(x−xi)/(xi+1−xi)`:

    C(x) = yi + (yi+1−yi) · (3t²−2t³)

Extensão constante fora dos nós. Isto é interpolação C1 com derivada zero em
cada nó, não uma spline PCHIP nem interpolação de dados científicos medida.
Intervalos degenerados, NaN/∞ e entradas não conhecidas são erros.

Produtos representam condições simultâneas. **Regras separadas e identificadas
representam alternativas aditivas**: uma regra por espessura e outra por altura
permitem nascimento em altura independentemente da primeira condição. Somar duas
regras pode criar dois órgãos no mesmo ponto; união/exclusão entre regras exige
política explícita futura. Não reinterpretar AND como OR silenciosamente.

`U` é obtido de hash BLAKE2b de uma lista JSON canônica de valores tipados:
versão do laboratório, semente, identidade e canal; os primeiros 52 bits viram
`U∈(0,1)`. IDs de filhos contêm ID do pai, **nome estável da regra**, ordinal da
estação e ordinal no grupo. Reordenar regras ou acrescentar pétalas não consome
os números aleatórios dos outros ramos. Identidades não são índices de malha.
Mudar a densidade pode mudar a correspondência espacial dos ordinais: não
prometemos preservar esculturas/attachments depois de mudar topologia.

Uma amostra por órgão gera variação coerente de comprimento/raio. Curvatura
irregular usa ondas suaves por órgão, com fase por hash, em vez de sortear cada
vértice. Assim a identidade aleatória não depende do passo de integração.

## 3. Eixo espacial e frame transportado

Um eixo é a curva `p(s)` com frame ortonormal destro `(T,N,B)`, onde `B=T×N`:

    p′(s) = T(s),                  |T|=1
    T′(s) = K(s),                  K·T=0
    N′(s) = −(K·N)T + τB
    B′(s) = −(K·B)T − τN

`K` é curvatura vetorial (m⁻¹), `τ` é torção material (rad/m). Frame de Bishop
transportado evita usar Frenet, que fica indefinido em trechos retos. Torção
material é um controle adicional; a curva por si só não determina a face da folha.

Para o campo de direção mundial unitária `D`, definimos a correção geodésica:

    θ = acos(clamp(T·D,−1,1))
    logT(D) = θ · normalize(D − (T·D)T)
    K = κN N + κB B + kD logT(D) + Knoise

`kD` tem unidade m⁻¹. Para `D=T`, correção zero; para `D=−T`, a direção de giro
é ambígua: escolher `N` do frame como desempate explícito. Esta escolha também
permite uma haste inicialmente apontada para baixo se voltar para cima. Misturar
alvos significa somar correções tangentes com pesos declarados; a referência v1
implementa um alvo mundial por definição. Gravidade de crescimento é uma direção
autoral aqui, sem equivaler a peso mecânico ou simular fotossíntese.

Integração: ponto médio explícito para posição e rotações exponenciais de
`ω=T×K`; transporte pela menor rotação entre tangentes e torção material a cada
passo. Recusar passos cuja rotação exceda 0,35 rad: pedir mais segmentos, sem
reduzir curvatura silenciosamente. Posições discretas têm segmentos de comprimento
`Δs`; frames interpolados descrevem uma aproximação da curva contínua. A tangente
interpolada não é exatamente a derivada da polilinha. Convergência é verificada
por refinamento, sem certificado de Hausdorff ou ausência de auto-interseções.
Curvatura e torção dependentes de altura/raio consultam o contexto local do eixo;
comprimento/largura são avaliados uma vez no contexto de nascimento do pai.

Na origem, projeta-se a normal de inserção no plano normal à tangente; somente
uma raiz sem normal usa um eixo mundial auxiliar determinístico. Amostrar em
`s` interpola posição, tangente e normal reortogonalizada; os filhos nascem dessa
mesma avaliação, evitando estimar o ponto por proximidade.

Seção elíptica e superfície lateral:

    r(u) = r0 [q + (1−q)(1−u)^p],     0<q≤1, p>0
    a(u)=r(u), b(u)=e·r(u),           e>0
    X(s,φ)=p(s)+a cosφ N+b sinφ B

Raio terminal nulo pode ser formulado como um ápice, mas não é admitido pela
referência v1. Bifurcação compartilha o ponto de inserção do eixo do pai; **a
superfície de união/colar é um problema posterior**. Dois tubos sobrepostos não
são apresentados como uma junção estanque.

## 4. Nascimento por intensidade e orientação tridimensional

Uma regra tem alvo, intervalo de níveis, intensidade `λ(c)≥0` em estações/m,
quantidade `n≥1` por estação, inclinação `α` e azimute `φ`. Primeiro resolve-se
o eixo do pai; depois sua distribuição de filhos. Assim altura e espessura são
consultadas em uma forma já conhecida, sem equação circular pai↔descendentes.

    H(u) = L ∫₀ᵘ λ(c(v))dv
    H(uj) = j + ξ,                   j=0,1,…; 0<ξ<1

`ξ` é fase aleatória por regra/pai. Esta é uma distribuição estratificada por
intensidade, **não um processo de Poisson**. A referência usa trapézios em uma
grade declarada e inverte cada intervalo com densidade linear (quadrática da
integral); testa refinamento dessa grade separadamente da curva do eixo.
Regiões de intensidade zero não produzem estações. Uma grade fixa pode perder
uma janela positiva mais estreita que seus bins; refinamento é necessário,
sem certificado de captura de toda janela arbitrária. Estações explícitas, inclusive
`u=0` e `u=1`, servem a tufos basais e flores terminais.

Para o membro `k` da estação `j`:

    φjk = φ0 + j·δ + 2πk/n + jitterjk
    Rjk = cosφjk N + sinφjk B
    Tchild = cosα T + sinα Rjk

`δ` determina progressão alternada/espiral; `n=2` dá pares; `n>2` dá verticilos.
Inclinação e azimute são campos independentes. O normal inicial do filho é a
projeção de uma direção herdada, com rotação de face adicional. A orientação da
lâmina também pode ser corrigida em direção a um alvo mundial antes da avaliação.

Na referência, comprimento do filho = campo de comprimento da sua definição;
raio inicial = raio local do pai × razão da regra. Comprimento dependente do pai
é possível fornecendo uma curva de campo que use `parent_length`; as configurações de
exemplo escolhem comprimentos em metros. A v1 não redistribui espessuras a partir
da massa dos descendentes: o usuário controla a seção e vê a massa resultante.

Uma futura política de dimensionamento pode impor, com expoente autoral `γ>0`:

    rparent^γ ≥ rcontinuation^γ + Σ rchild^γ

Essa é uma restrição de dimensionamento opcional, não lei universal, não
conservação automática de massa e não está ativada na referência. Pode ser
resolvida de baixo para cima após a topologia; não realimentar nascimento por
espessura sem um algoritmo de ponto fixo/convergência separado.

## 5. Lâmina, contorno, lobos e espessura

Uma lâmina usa coordenadas materiais `u∈[0,1]`, `v∈[−1,1]`, comprimento `L`,
largura de referência `W` e espessura física `t(u,v)>0`. Centro `C(u)` e frame
`(T,N,B)` são uma curva orientada como no eixo. Forma transversal:

    envelope(u) = u^a (1−u)^b / maxₓ[x^a(1−x)^b],  a,b>0
    lobes(u) = 1 − A · [(1−cos(2πmu+ψ))/2]^q
    halfwidth(u) = W/2 · envelope(u) · lobes(u)
    x(u,v) = v · halfwidth(u) · (1 + asymmetry·sign(v))
    z(u,v) = fold·|x| + cup·x²/W + edgewave(u,v) + relief(u,v)
    S(u,v) = C(u) + x N(u) + z B(u)

`a,b` movem a região de maior largura; `m` controla pares de reentrâncias ao
longo de uma folha pinada; `A∈[0,1)` é profundidade dos lobos; `q>0` controla a
concentração/arredondamento das reentrâncias. `m` não é um contador universal
de pontas para folhas palmadas. `W` é largura de referência antes de assimetria,
lobos e curvatura; **não é largura média nem largura máxima garantida**. Estas
grandezas são derivadas por amostragem; uma interface futura pode inverter a
normalização para permitir editar a média. Serrilha usa outra escala de frequência
e amplitude; a referência mantém uma só camada de lobos para permanecer legível.

Curvatura central, dobra, concavidade e torção são controles independentes.
Uma folha composta é uma raque com regras de inserção de folíolos; uma flor é
um receptáculo/eixo com verticilos de pétalas e sépalas. Contornos palmados gerais,
furos, bases sobrepostas e volume de frutos requerem outros domínios; não cabem
todos no único gráfico transversal acima. A matemática admite acrescentar outros
avaliadores de órgão sem alterar o contrato da hierarquia.

Superfícies superior e inferior:

    n = normalize(∂uS × ∂vS)
    S± = S ± t(u,v)n/2

Nas pontas `halfwidth=0`, a parametrização é singular; usar o limite interior para
normal e quadratura. A referência consulta a superfície média e integra volume
por lâmina fina; não gera sólido estanque, costura lateral ou evita colisões.

## 6. Nervuras como campo separado

Nervuras são curvas `Vi(w)` no domínio material `(u,v)`, com largura `σi`,
amplitude de relevo `Ai` em metros e categoria. A v1 fornece rede pinada e rede
paralela com nervura central; o mesmo contrato aceita redes palmadas/reticuladas
em outro gerador. A rede independe da intensidade do relevo.

    relief(u,v) = Σ Ai exp[−di(u,v)²/(2σi²)]

`di` usa a distância a segmentos na métrica de referência `(Lu,Wv/2)`;
`σi` é em metros. Isto aproxima distância física: a deformação da lâmina pode
esticar o traço. Não alegar distância geodésica sobre a folha curva. A amplitude
é somada, inclusive em cruzamentos; trocar por máximo suave seria outra política.
Relevo é atenuado perto do contorno e das pontas. Rede lisa (`Ai=0`) conserva sua
identidade e desenho, permitindo variar liso→marcado sem mudar topologia.

Relevo desloca a superfície média. Espessura constante aplica-se em torno dela;
uma nervura volumosa que também aumenta espessura exige campo `t(u,v)` próprio.
A referência usa espessura uniforme e não atribui reforço anisotrópico às nervuras.

## 7. Mecânica derivada e contrato de deformação

Para cada material: densidade `ρ` (kg/m³), módulo de Young `E` e cisalhamento `G`
(Pa), amortecimento e limites de deformação/ruptura são propriedades distintas.
Casca tem fração radial `f∈[0,1)` e material próprio; núcleo tem seção homotética
`ac=(1−f)a`, `bc=(1−f)b`. São materiais efetivos, sem calibração botânica.

    A = πab;             I_N = πab³/4;       I_B = πa³b/4
    Jellipse = πa³b³/(a²+b²)
    μ = ρcore Ac + ρbark(A−Ac)
    EA = Ecore Ac + Ebark(A−Ac)
    EI_N = Ecore I_N,core + Ebark(I_N−I_N,core)
    EI_B = Ecore I_B,core + Ebark(I_B−I_B,core)
    mass_axis = ∫ μ ds

`GJ` composto é uma aproximação por soma de contribuições homotéticas; para seção
circular concêntrica é a expressão usual. Para elipse heterogênea não resolve o
problema geral de empenamento/tensão de cisalhamento. A referência identifica isso
como aproximação, sem chamar o resultado de solver de elasticidade.

Para a lâmina fina:

    area = ∫∫ |∂uS × ∂vS| du dv
    volume ≈ ∫∫ t |∂uS × ∂vS| du dv
    mass = ρ · volume
    Dplate = E t³ / [12(1−ν²)]

`Dplate` é rigidez de flexão de placa isotrópica, em N·m. `−1<ν<0,5`; a
referência usa ν=0,3. Folhas anisotrópicas precisam de tensor constitutivo. Na
quadratura, derivadas são diferenças centrais, área é ponto médio, e os erros
numéricos são medidos por refinamento. Aproximação de lâmina fina não é volume
exato para espessura grande/curvatura extrema. Massa da hierarquia é a soma de
órgãos: sobreposições de junções não são descontadas por união volumétrica.

Contrato futuro para eixo deformável (não integrado temporalmente nesta v1):

    U = 1/2 ∫ [EA ε² + EI_N(κN−κN,0)²
                         + EI_B(κB−κB,0)² + GJ(τ−τ0)²] ds
    M q̈ + C q̇ + ∂U/∂q = fgravity + fwind + fcontact

Vínculos pai/filho, massa descendente e forças em folhas entram na montagem do
sistema; amortecimento exige coeficientes/unidades ou razões modais explícitas.
Arrasto aproximado depende de área projetada, orientação e velocidade relativa:
`F≈ρair Cd Aprojected |vrel|vrel/2`. Rigidez não determina sozinha amplitude ou
frequência do vento. Dureza de contato, resistência a fratura e rigidez elástica
não são sinônimos. Aqui fornecemos grandezas para essa futura etapa, sem solver,
colisão, vento animado, quebra ou física de runtime entregue.

## 8. Idade e crescimento temporal — contrato reservado

Idade absoluta `t`, nascimento `tb`, duração de maturação `Tm>0` e curva monotônica
`g(clamp((t−tb)/Tm,0,1))` podem modular comprimento/raio/tamanho. Cada estação
precisa de tempo de ativação, e identidade deve existir independentemente da
amostragem visual. O esqueleto maduro atual pode servir de alvo, mas interpolar
seu tamanho não constitui simulação causal de crescimento. Competição por luz,
poda, estações, dano e crescimento reativo permanecem modelos futuros explícitos.

## 9. Avaliação, recusa e continuidade

1. Validar definições, referências, campos, regras e orçamento.
2. Avaliar raízes e curvas; consultar estações no pai resolvido.
3. Criar filhos em fila, por identidade estável, até completar a estrutura.
4. Ordenar resultados por identidade para inspeção reprodutível.
5. Consultar superfícies/nervuras e integrar propriedades sob demanda.
6. Gerar relatório completo em memória; substituir arquivo de destino por
   temporário + flush/fsync + rename. Erro anterior preserva relatório existente.

Orçamentos do laboratório são entradas de execução, não propriedades botânicas:
órgãos, profundidade, segmentos por eixo, bins de intensidade e trabalho da geração.
Quadratura de propriedades tem resolução própria limitada a512×512, além das
até129 nervuras por lâmina; esse custo não entra no contador da geração.
Sem truncamento silencioso. Curvas do eixo e grade de distribuição são resoluções
distintas; mudar segmentos não deve alterar IDs nem o stream aleatório. Órgãos
podem mudar de posição numericamente com a curva refinada. Determinismo é para
a mesma versão/configuração/ambiente Python; não promessa bit-exata entre libm,
arquiteturas ou futuras implementações Rust.

Limites de validade: sem união de malhas, teste global de interseção, contato,
modelo de recursos, hidrologia, anatomia universal, reprodução botânica, streaming,
LOD ou integração ao Ignis. Critério deste recorte é matemática inspecionável e
verificável, com exemplos diversos. Aceite visual e aplicação são etapas próprias.

## Referências consultadas

- Prusinkiewicz, [A look at the visual modeling of plants using L-systems](https://algorithmicbotany.org/papers/a-look-at-the-visual-modeling-of-plants-using-l-systems.html): regras e arquitetura vegetal.
- Runions, Lane e Prusinkiewicz, [Modeling Trees with a Space Colonization Algorithm](https://algorithmicbotany.org/papers/colonization.egwnp2007.html): ocupação espacial, extensão futura.
- Bergou et al., [Discrete Elastic Rods](https://www.cs.columbia.edu/cg/rods/): curvas orientadas, transporte e elasticidade; não afirmamos implementar o solver do artigo.
- Runions, [Computational Modeling of Leaf Development and Form](https://algorithmicbotany.org/papers/runionsa.th2014.html): diversidade de contornos e limites de um modelo único.

Equações e escolhas da referência acima são a formulação deste laboratório;
referências fundamentam conceitos, sem estabelecer equivalência de implementação.

## Reprodução e evidências — 01/11/2026

No checkout principal, sem dependências adicionais:

```sh
python3 scripts/botany_math.py
python3 scripts/verify_botany_math.py
python3 scripts/plot_botany_math.py
```

Saídas correntes em `artifacts/botany-math/`: cinco relatórios JSON, resultado
numérico `verification.json`, diagramas `overview.svg` e `leaves.svg`. PNGs de
apresentação foram rasterizados localmente a partir dos SVGs e inspecionados;
são diagramas matemáticos, não imagens do renderer do Ignis. O vetor é a fonte
reproduzível dos diagramas. Sem gerar um binário ou mudar o build corrente.

Configuração é a API tipada do módulo (`Recipe`, `Axis`, `Blade`, `Rule`, `Field`,
`Curve`, `Root`, `Limits`). `examples()` fornece quatro receitas editáveis no
código. A CLI exporta essas receitas; não lê um arquivo JSON arbitrário nem
apresenta o relatório como formato de autoria aprovado. Exemplo de condição
combinada, com intensidade de12estações/m quando ambos os fatores permitem:

```python
Field(12., drivers=(
    ("height", Curve(((0., 0.), (.65, 0.), (.7, 1.), (1., 1.)))),
    ("radius", Curve(((0., 0.), (.025, 0.), (.03, 1.), (.04, 1.)))),
))
```

Para altura **ou** espessura, criar duas regras com nomes próprios, uma para cada
campo. A experiência numérica independente usou um eixo reto de1m, raio basal
0,04m e terminal0,004m: nasceram quatro órgãos na zona espessa e quatro na zona
alta/fina. A regra de altura continuou ativa onde a regra de espessura era zero.

| Receita sintética | Estrutura avaliada | Unidades de trabalho da geração |
| --- | --- | --- |
| Grama | 12eixos basais +24lâminas | 4728 |
| Árvore | 1tronco +31ramos +191folhas | 33562 |
| Arbusto florido | 6eixos +5receptáculos +25pétalas +23folhas | 8669 |
| Folhagem composta | 1raque +22pinas +280folíolos | 42810 |

São contagens de órgãos lógicos, não malhas, triângulos ou corpos físicos.
Cada órgão possui65amostras na configuração padrão. Contador de trabalho é
instrumentação da referência, não número de FLOPs nem medida de desempenho.

Verificação executada no Python local/macOS:

- Identidades e resultados iguais após reordenar regras/dicionário e após
  acrescentar uma regra independente; fonte não muda durante avaliação.
- Erro máximo de ortonormalidade4,44×10⁻¹⁶; erro de posição na inserção dos
  filhos zero nos quatro exemplos. Não é garantia de união de superfícies.
- Guardas de órgãos, profundidade, trabalho, referências ausentes, identidade de
  regra duplicada, valores não finitos e resolução inadequada recusaram entradas.
  Falhas de geração/serialização conservaram o relatório anterior; temporários
  foram removidos. Estes são os checks persistentes de risco crítico.
- Curva circular de1m/curvatura1m⁻¹: erro de extremidade1,561×10⁻⁴ →
  3,902×10⁻⁵ →9,754×10⁻⁶ →2,438×10⁻⁶m em16/32/64/128segmentos,
  compatível com convergência de segunda ordem neste caso.
- Correção de direção inicialmente antiparalela terminou com produto escalar
  com o céu0,9999695; regra de desempate foi exercitada, sem posição especial.
- Refinamento de intensidade64→128→256bins reduziu a diferença máxima de
  estação4,373×10⁻⁶ →3,707×10⁻⁷ em `u`; intensidade constante recuperou
  posições analíticas e a zona de intensidade zero permaneceu vazia.
- Área da lâmina plana parabólica comparada com `2LW/3`: erro relativo
  0,3472% →0,08681% →0,02170% em12²/24²/48²pontos. Cilindro circular
  homogêneo recuperou massa e rigidezes analíticas dentro de10⁻¹² relativo.
- Lâmina curva/dobrada/com nervuras: diferença de área48²→96² de0,01516%.
  O último resultado foi0,01085255m² e2,3062g com os materiais sintéticos
  escolhidos. Não estender essa tolerância a folhas arbitrárias ou nervuras
  menores que a resolução utilizada.

As experiências numéricas usam oráculos analíticos e refinamento, sem uma suíte
por slider. Compilação Python e `git diff --check` passaram. Não executar a suíte
Rust/MCP para este incremento independente: nenhum código/contrato da engine
foi modificado. Validação nativa pertence à etapa futura de aplicação.

Revisão estática final (mesmos dez pontos da prévia):

1. Somente chamadas de laboratório/CLI; zero código novo no tick/frame.
2. Receitas compartilhadas durante geração; `asdict`/JSON materializam cópias
   somente na validação/exportação explícita, custo aceito do laboratório.
3. Floats binários64, vetores3D e IDs tipados por nomes; sem `eval`, reflexão no
   crescimento, PRNG global ou uso de índice de vértice como identidade.
4. Filas e expansões contabilizadas; passagem antecipada por referência inválida
   e erro de orçamento integral, sem truncamento como se fosse resultado completo.
5. Alvos normalizados e fases de ruído preparados uma vez por órgão; campos sem
   variação não calculam hash; rede de nervuras preparada uma vez por quadratura.
6. Nenhuma operação GPU/rede/renderer do Ignis; SVGs e rasterização opcional são
   apenas apresentação externa dos resultados matemáticos.
7. Sem worker/lock/cache global; somente amostras residentes da avaliação atual.
8. Relatórios substituídos atomicamente no mesmo caminho; saídas atuais sem
   histórico/timestamps, builds/copias da engine ausentes.
9. Checks críticos e oráculos acima passaram; figuras inspecionadas, limites
   geométricos e constitutivos registrados. Sem alegação de aceite artístico.
10. Custos aceitos: Python/tuplas/dicionários, quadratura numérica, JSON completo
    em memória, exportação sob comando. Nenhum benchmark/FPS/escala de runtime
    ou portabilidade bit-exata entre máquinas é inferido.

Continuidade matemática: outros domínios de contorno (especialmente palmados),
rede de nervuras reticulada, união local nas bifurcações e montagem/resolução
da elasticidade com massa descendente são extensões identificadas. A referência
atual já permite experimentar regras/arquétipos antes de escolher como publicá-los
pelas ferramentas do Ignis; nenhuma dessas escolhas futuras foi feita implicitamente.
