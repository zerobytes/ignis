# Botânica matemática v2 — sete eixos

EAF-190, continuação autorizada em 01/11/2026. Laboratório independente, sem
integração ao Ignis. V1 e perfis de folíolos permanecem em
[procedural-botany-math.md](procedural-botany-math.md).

## Decisões e revisão anterior à implementação

1. Folhas palmadas usam um domínio polar estrelado com pontas/vales explícitos.
   Número de lobos, distribuição angular, comprimentos, recortes e arredondamento
   são independentes. O domínio admite furos elípticos por exclusão; não promete
   representar toda folha, nem sólidos foliares estanques. A V1 longitudinal fica
   preservada. A quantidade de pontas principais não conta dentes secundários.
2. Nervuras são um grafo geométrico próprio, com ordens e raios; redes podem ter
   ciclos. Relevo e reforço material consultam esse grafo, sem confundir os dois.
3. Junções usam união implícita suave de volumes locais e extração por tetraedros.
   Escolha evita colar anéis por proximidade; resolução e aproximação do campo
   ficam expostas. É construção de uma superfície nova, não preservação automática
   de escultura/rig da engine. Campos de elipsoides não são distâncias exatas.
4. Copa/ambiente controlam orientação e admissão de candidatos, com rejeições
   observáveis. Dimensionamento de raios é uma passagem posterior, sem realimentar
   a topologia por espessura e introduzir um ponto fixo oculto.
5. Física usa modos reduzidos de flexão/torção e massa acoplada por Jacobianos.
   Euler implícito, amortecimento Rayleigh e estado por identidade. É uma
   aproximação para deformações pequenas/moderadas; não é o solver DER completo,
   colisão ou fratura volumétrica. Preferimos um modelo com energia e oráculos a
   uma oscilação de vento decorativa. Há limites explícitos de DOFs/partículas.
6. Crescimento revela eixos por comprimento material e escala lâminas; nascimento
   depende do instante em que o pai alcança a inserção. Poda/novos brotos são
   transações sobre IDs estáveis. Influências de luz/obstáculos pertencem à geração;
   isto não implementa metabolismo ou uma simulação botânica causal universal.
7. Flores, frutos/sementes, raízes, tecidos volumosos e trepadeira helicoidal usam
   definições e receitas, sem criar um algoritmo por espécie.

Revisão estática prévia: somente CLI/chamadas; biblioteca padrão; floats64 e
definições compartilhadas; quotas antes de expansão; caches derivados locais;
quadratura e preparo de grafos fora dos passos físicos; fatoração da matriz
implícita reutilizada quando constante; nenhum worker/lock/rede/GPU/arquivo da
engine. Saídas substituem caminhos atuais. Checks novos protegem topologia,
identidade, estado físico, atomicidade e integrações materiais; forma visual usa
inspeção e oráculos diretos. Custos Python/dense O(D³), campo×grade e diferenças
finitas são aceitos no laboratório, sem promessa de FPS ou adoção na engine.

## Fundamentação

- [Modeling lobed leaves](https://algorithmicbotany.org/papers/leaves.cgi2003.pdf):
  separação entre silhueta e estrutura; nossa parametrização é polar, não a
  implementação de sticky splines do artigo.
- [Discrete Elastic Rods](https://www.cs.columbia.edu/cg/rods/index.html):
  referenciais materiais e energia de flexão/torção. O modelo reduzido abaixo tem
  graus de liberdade e garantias diferentes.
- [Space colonization](https://algorithmicbotany.org/papers/colonization.egwnp2007.html):
  referência para influência espacial; os controles de copa abaixo são autorais.

## 1. Domínio palmeado e pontas

`PalmateBlade` estende a definição de lâmina. Coordenadas materiais são agora
`r∈[0,1]`, `θ∈[−π,π]`; a API mantém `u=r`, `v=θ/π` para usar as mesmas consultas.
Essa coordenada radial não é a posição de nascimento no pai. O campo de tamanho
continua sendo avaliado uma vez na inserção, como na folha pinada.

Cada ponta principal tem ângulo `αi` e comprimento relativo `li`. A configuração
uniforme distribui `N` ângulos no intervalo `[-spread,+spread]`; ângulos e
comprimentos individuais podem substituir essa distribuição. Entre duas pontas
há um vale de raio `min(li,li+1)(1−cut_depth)`. Dois vales adicionais delimitam
o colo basal, que se liga ao pecíolo. O colo é uma região de inserção, não uma
ponta principal extra. O domínio é estrelado em relação ao centro de nervuras.

Em um intervalo entre vale e ponta, `t∈[0,1]`:

    w(t) = t^q / [t^q + (1−t)^p]
    R(t) = Rvale + (Rponta−Rvale)w(t)

`p=rounding` controla o arredondamento da ponta, admitindo cúspides para `p<1`;
`q=valley_rounding>1` controla o arredondamento entre pontas. Intervalos de descida
usam a mesma função invertida. Todas as amplitudes radiais são positivas, evitando
cruzamento do contorno polar. Isto não representa silhuetas não estreladas ou
lobos que se sobreponham no domínio plano.

Com `h=base_radius`, `xm=maxθ |sinθ R(θ)|` (amostrado em 2048 intervalos):

    x/W = r sinθ R(θ)/(2xm)
    y/L = [h + r cosθ R(θ)]/(1+h)

O pecíolo é `(0,0)` e a região central fica em `(0,h/(1+h))`. Lobos basais podem
descer abaixo da inserção. A superfície usa o eixo orientado da V1 em `y/L`, com
extensão tangente nas partes abaixo de0 ou acima de1, e aplica dobra/concavidade
e relevo em direção ao binormal. O máximo de largura é normalizado por uma busca
amostrada, sem garantia analítica de um máximo global exato.

Um furo elíptico exclui os pontos que satisfazem
`((x−xh)/rx)²+((y−yh)/ry)²<1` em coordenadas normalizadas `x/W,y/L`.
Furos são disjuntos e sua inclusão no contorno é verificada por 64 amostras de borda;
não é um certificado intervalar para detalhes arbitrariamente pequenos. Furos
que eliminam a origem vascular são recusados neste gerador. A área/massa usa
quadratura da superfície com indicador do domínio; detalhes menores que a grade
pedem refinamento. Não há malha sólida fechada da lâmina perfurada nesta etapa.

## 2. Grafo vascular e reforço

`VeinGraph` contém nós `(x/W,y/L)` e arestas com ordem e raio em metros. Conecta
pecíolo → centro → eixos dos lobos; gera até três ordens. Conexões próximas
opcionais formam ciclos reticulados. O grafo é validado quanto a índices,
conectividade, duplicatas, degeneração e finitude; sua rede não depende da
intensidade do relevo. Segmentos são cortados antes de sair do contorno
(admissão amostrada); interseção com furos usa raízes da quadrática da elipse.
Não simulamos diferenciação vascular ou transporte de seiva.

Para cada aresta `e` de raio `re`, comprimento `le` e direção plana unitária `de`:

    Vvascular = Σ πre² le
    mvascular = ρvascular Vvascular
    Bvascular = Σ [Evascular πre⁴ le/4] de⊗de

`Bvascular` tem unidade N·m³ e expressa reforço integrado direcional. É uma
homogeneização autoral; não um tensor constitutivo anatômico completo. Os
cilindros acrescentam massa/rigidez à lâmina base, sem descontar interseções ou
substituição de tecido. A física opcional de lâminas consome essa massa e um
reforço de flexão derivado de `Bvascular/L²`. `ρvascular=950 kg/m³` e
`Evascular=2×10⁸ Pa` são valores efetivos de demonstração, sem calibração.

## 3. Junções implícitas

`Tube` avalia distância ao segmento menos raio interpolado; `Ellipsoid` usa
`(norm(local/radii)−1)min(radii)`. O primeiro é uma aproximação para tubo com
raio variável, e o segundo não é SDF euclidano. A união usa:

    h = clamp(1/2 + (b−a)/(2k),0,1)
    smin(a,b,k) = (1−h)b + ha − kh(1−h)

O operador é C1 entre campos suaves, coincide com o menor campo fora da faixa
de mistura e admite controle de alargamento/colar por `k` em metros. A sequência
de união é explícita: o operador não é associativo. Uma junção galho/tronco usa
tubos; um pecíolo/volume foliar pode usar tubo e elipsoide.

Extração por seis tetraedros por célula, orientação pelo gradiente do campo e
identidade de vértice pela aresta global da grade. O orçamento padrão é100 mil
células/100 mil triângulos e até 16 volumes. A caixa precisa envolver toda a superfície;
recusar caixa cortante ou malha não fechada. A malha é triangular, portanto não
é uma superfície infinitamente suave.

Para evitar microtriângulos quando o isovalor coincide com nós, o nível é
`iso_offset_fraction·menor_passo`, padrão 0,001 voxel, exposto em `iso_level_m`.
O deslocamento converge a zero com refinamento. Coincidências restantes são
recusadas. Essa regularização é parte explícita da aproximação; não remover
triângulos degenerados silenciosamente nem apresentar a extração como união
exata de malhas preexistentes. Fonte, rigs e materiais da engine não participam.

## 4. Copa, ambiente e dimensionamento

`Environment` fornece alvos de luz, repulsão de volumes esféricos, atração para
dentro de uma copa elipsoidal, exclusões e competição entre pontas de ramos.
A correção é projetada no plano normal ao crescimento: `Kenv=F−(F·T)T`.
Perto de um obstáculo, a repulsão cresce com a penetração na faixa de influência;
o candidato final é recusado se as amostras ainda violarem o volume excluído.
Luz é um alvo autoral, sem cálculo de irradiância/sombreamento ou metabolismo.

Admissão verifica a curva e, para lâminas, sua borda; acrescenta margem física de
raio/espessura. É verificação amostrada, não colisão contínua certificada.
Raízes principais são implantadas explicitamente e não são recortadas pela copa.
Dominância suprime candidatos axiais perto do ápice por hash por identidade;
competição usa distância entre extremidades de ramos do mesmo nível. Não é uma
simulação de recursos, navmesh ou detecção universal de colisões da planta.
Cada recusa fica em `Plant.rejections`; budget esgotado recusa a avaliação inteira.

`dimension_radii` percorre descendentes primeiro, sem mudar IDs/topologia:

    demanda(u)^γ = rterminal_autoral^γ + Σfilhos_em_ou_acima_de_u rfilho^γ
    rfinal(u) = max(raio_autoral(u), perfil_de_demanda(u))

Folhas contribuem com raio efetivo `leaf_demand·sqrt(LW/π)`. O expoente é autoral,
padrão2; não uma lei biológica universal. O perfil absoluto fica em
`Organ.radius_profile`, com transições depois das inserções; `Organ.radius`
continua sendo o mínimo basal autoral. `radius_at` é a autoridade para geometria
e propriedades físicas. O envelope de demanda não força o taper original a
sustentar um ramo terminal e assim evita inflar desnecessariamente o tronco.
Reavaliação é idempotente; não realimenta regras de nascimento. Dimensionar pode
ampliar a ocupação física, exigindo nova análise de espaço antes de publicação.

## 5. Física reduzida executável

`ReducedModel` atribui três coordenadas por eixo flexível: duas de flexão e uma
de torção. Lâminas são flexíveis opcionalmente; volumes de frutos/sementes são
massas acopladas e não recebem um modo de haste esbelta. As bases de raízes são
fixas. Não há translação de corpo livre, colisão, extensibilidade ou fluido.

Modo angular de flexão `f(u)=2u−u²`, modo de torção `u`. Para uma haste reta de
seção constante sob força terminal, esse modo recupera a solução linear de
deflexão `FL³/(3EI)` em pequeno ângulo:

    Kflex = ∫ EI [2(1−u)/L]² ds       (constante:4EI/(3L))
    Ktwist = ∫ GJ/L² ds

Geometria deformada é reconstruída rotacionando cada segmento no ponto médio
do modo; comprimentos dos segmentos são preservados e filhos seguem o frame
de inserção do pai. Para integração, o Jacobiano `J=∂x/∂q` é calculado no repouso
por diferenças centrais e fica fixo:

    Mij = Σpartículas mp (∂xp/∂qi)·(∂xp/∂qj)
    Q = Σ Jpᵀ [mp g + Fdrag,p]
    C = αM + βK
    (M+Δt C+Δt² K) vnovo = M vantigo + Δt[Q−K(q−qplástico)]
    qnovo = q + Δt vnovo

A massa dos descendentes entra nas coordenadas dos ancestrais; termos fora da
diagonal preservam acoplamento entre ramos. Quadratura de eixos usa 8 estações
e 4 pontos transversais, recuperando momentos de seção elíptica; lâminas usam 4×4
pontos de área. Folhas/nervuras acrescentam massa aos eixos mesmo quando rígidas.
Normais e área projetada de vento usam aproximação no repouso:
`Fdrag=ρar Cd Aprojetada |vvento−vp|(vvento−vp)/2`.

Energia monitorada: `1/2 vᵀMv + 1/2(q−qplástico)ᵀK(q−qplástico)`.
Sem forças externas, Euler implícito e Rayleigh dissipam energia. O campo de
gravidade fornece força generalizada; a energia acima não inclui potencial
gravitacional quando a gravidade está ativa.

Deformação efetiva de ruptura/escoamento por órgão:
`εeff=(rmax/L)sqrt(4θN²+4θB²+θtwist²)`; para lâmina usa-se meia espessura.
Acima do limiar de escoamento, retorno radial limita a deformação elástica e
atualiza `qplástico`; acima do limiar de ruptura, retirar o órgão e sua subárvore
do sistema ativo. Não simular fragmentos soltos, ferida ou fratura volumétrica.
Parâmetros são efetivos, sem modelo de dano independente por camada da casca.

Domínio padrão: norma angular por órgão ≤ 0,65 rad; passo ≤ 0,05 s; até 96 DOFs,
256 órgãos,2048 partículas e 5 milhões de amostras de preparo do Jacobiano. Fora
disso, recusar explicitamente. Não limitar a deformação silenciosamente nem
interpretar o modelo linearizado como solver de grandes dobramentos de grama.
Matriz/fatoração corrente são reutilizadas; mudar passo ou ruptura substitui o
cache único. Exigir estado com fingerprint da receita, geometria, parâmetros e
DOFs; restore recusa origem divergente/NaN/IDs quebrados inconsistentes. Passos
constroem candidato imutável e só publicam após validação.

## 6. Crescimento, abertura, poda e brotos

`growth_at` fornece uma vista temporal; não altera o adulto canônico.
`g(a)=3a²−2a³`, `a=clamp((t−tnascimento)/Tm,0,1)`.
Eixos expõem a curva madura até o comprimento material `gL`; o filho em `u`
nasce quando `gparent=u`, mais atraso configurável. Isso evita nascimento de
galhos além da ponta ainda inexistente. Lâminas crescem em tamanho e rodam de
uma orientação fechada em direção à orientação madura. Raios crescem com `g^p`.
Na maturação completa, a vista usa o órgão canônico exato.

`GrowingOrgan.material_extent` distingue fração exposta do eixo e coordenadas
normalizadas da vista. Inserção adulta `u` corresponde a `u/material_extent`
na parte visível do pai. Perfis de raio temporários são derivados para a fração
exposta. A vista não é um novo formato Scene, nem um estado físico conservativo.

`PruningState` guarda fonte atual, IDs aposentados, serial e horários dos novos
brotos. `prune` retira a subárvore; `regrow` gera um candidato com prefixo de
identidade novo, reanexa ao pai vivo e só então publica. Referência inválida ou
orçamento insuficiente conserva a fonte. `growth_of` respeita o horário da
brotação; sobreviventes não rejuvenescem. `regrow(...,environment=...)` permite
que brotos respondam a luz/obstáculos. São operações explícitas; não há busca
autônoma de locais de poda ou simulação hormonal.

Física e crescimento têm relógios e hipóteses diferentes: uma vista temporal
pode iniciar um modelo físico novo; o fingerprint impede reaproveitar estado
dinâmico de uma geometria/massa diferente. Transferência conservativa de momento
entre estados de crescimento, fotossíntese, estações e senescência não foram
implementadas. São contratos posteriores, sem bloquear os sete exemplos atuais.

## 7. Órgãos e modos adicionais

`organ_recipes()` gera:

- Flor com pétalas, sépalas, estames, pólen, ovário, estilete e estigma (31 órgãos).
- Fruto e semente (3 órgãos com o pedúnculo). `SolidAxis` usa perfil
  `r(u)=rbase sqrt(1−(2u−1)²)` e seção elíptica; polo de raio zero é explícito.
  Volume analítico do elipsoide: `4πabc/3`, independente de um material de lâmina.
- Raízes com orientação para baixo e laterais (7 órgãos).
- Suculenta com órgãos volumosos e material efetivo próprio (11 órgãos).
- Trepadeira helicoidal com folhas (15 órgãos) em torno de um suporte cilíndrico
  declarado. Para raio `R` e avanço por volta `P`, `a=P/(2π)`, curvatura
  `κ=R/(R²+a²)` e torção material `τ=−a/(R²+a²)` na convenção XYZ usada.

O suporte helicoidal é conhecido na receita; não é detecção automática de apoios
arbitrários. Raízes não perfuram terreno físico do Ignis, frutos não têm
simulação de polinização/dispersão e a suculenta não simula hidráulica celular.
O resultado demonstra composição de órgãos e parâmetros, não anatomia universal.

## Reprodução e evidências

```sh
python3 scripts/verify_botany_math.py
python3 scripts/verify_botany_v2.py
python3 scripts/demo_botany_v2.py
```

Sem pacotes adicionais. Saídas atuais em `artifacts/botany-math/`:
`v2-verification.json`, `v2-demo.json`, `v2-junction.json` e SVGs
`v2-leaves`, `v2-junction`, `v2-crown`, `v2-physics`, `v2-growth`, `v2-organs`.
PNGs locais de apresentação são rasterizações dos vetores, sem edição artística.
Não são renders ou prova de integração ao Ignis. O diretório está ignorado no Git;
receitas/diagramas são reproduzíveis pelos scripts versionados.

Exemplo editável, com `PYTHONPATH=scripts` no ambiente Python:

```python
from botany_math import Field, properties
from botany_surfaces import PalmateBlade, leaf_plant

definition = PalmateBlade(
    length=Field(.26), width=Field(.28),
    tips=7, cut_depth=.93,
    rounding=.55, valley_rounding=2.,
    tip_lengths=(.65, .85, .96, 1., .96, .85, .65),
)
plant = leaf_plant(definition)
area_and_mass = properties(plant.organs[0], definition, resolution=64)
```

### Resultados no fonte final

| Eixo | Evidência executada |
| --- | --- |
| Contornos | Diagramas de 5/7/15 pontas e lâmina perfurada; consulta 3D finita. A área retirada pelos dois furos concordou com sua área analítica dentro de 1,75% em 160² amostras. |
| Nervuras | Grafos conectados de 77/107/137 nós, até 3 ordens; casos reticulados com 38 ciclos. Massa e reforço foram consumidos pelo modelo físico de lâmina. |
| Junções | Caso de 24 células/eixo:2828 v/5652 tri, zero bordas abertas/não manifold ou conflitos de orientação. Diagrama de 28: 3800 v/7596 tri. Esfera: erro de volume 3,06%→0,75% em 12→24. |
| Arquitetura | Árvore de 223→126 órgãos, 7 rejeições com motivo no diagrama; ensaio de arbusto 59→45. Alvo de luz e obstáculo curvaram crescimento; sizing preservou mínimos autorais e foi idempotente. |
| Física | Haste sob carga terminal: 4,47146 cm contra 4,47623 cm analíticos, erro 0,1065%. Gravidade/vento, flexão/torção, amortecimento, plasticidade e ruptura exercitados. |
| Tempo/poda | Maturação converge ao órgão canônico exato; inserções verificadas em múltiplas idades. Poda retirou 10 órgãos e brotação criou 10 novos sem reutilizar IDs, respeitando horário de nascimento e sobreviventes. |
| Órgãos | Flor 31, fruto/semente 3, raízes 7, suculenta 11, trepadeira 15 órgãos. Massa de elipsoide com erro relativo 0,00543%; desvio máximo da hélice 108,5→27,1 µm em 128→256 segmentos. |

No ensaio físico sem forças, a energia caiu monotonicamente de
`9,5560×10⁻⁴J` para`9,2535×10⁻⁶J` em 240 passos (2 s). Uma folha descendente
aumentou a inércia modal do pai de`0,00270860` para`0,00321930kg·m²`.
Um sistema de 6 DOFs manteve massa fora da diagonal (`1,94989×10⁻⁴kg·m²`),
conexões pai/filho e comprimentos dos segmentos; sua energia também dissipou.
Na lâmina reforçada, aumentar raio vascular de 0,10 para 0,35 mm levou a rigidez
reduzida de flexão fora do plano de`3,7932×10⁻⁵` para`4,7192×10⁻⁵N·m`.
O eixo fraco de flexão da lâmina fina foi verificado, evitando troca entre
flexão no plano e fora do plano.

Checks persistentes novos protegem conectividade/topologia, massa/elasticidade
como integração material, identidade de poda/brotação, origem e integridade do
estado físico, atomicidade de falha e limites de recursos. Não há teste por
slider. Guardas recusaram caixa de extração cortante, orçamento excessivo,
furo inválido, ambiente com budget insuficiente, referência de brotação inválida,
restore de outro modelo/NaN, forças inválidas e ângulos fora do domínio.
Figuras verificam contorno e apresentação diretamente.

Regressão V1, verificação V2, exemplos, compilação Python e diff-check passaram.
Os cinco exemplos V1 conservaram as contagens 36/223/59/303/85 e as identidades.
Não houve alteração de Rust/Scene/MCP/renderer/replay, build ou janela da engine;
por isso testes nativos não foram repetidos para este laboratório independente.

### Correções reveladas pelas provas

- Hélice revelou erro de primeira ordem quando a torção era aplicada depois da
  flexão na V1. O integrador agora usa o vetor de Darboux completo
  `Ω=T×K+τT` no ponto médio para atualizar tangente e normal conjuntamente.
  O arco circular anterior preservou convergência e o caso com torção passou
  a convergir em segunda ordem. Geometria com torção pode mudar ligeiramente;
  IDs/seed são preservados, sem promessa de pixels antigos em um integrador corrigido.
- Raiz do isocampo sobre nós da grade produzia microtriângulos. Regularização
  do nível é explícita e o extrator recusa degeneração; a cobertura não foi
  relaxada para aprovar a extração.
- Preservar um taper fixo até uma ramificação terminal inflava o tronco. O
  dimensionamento passou a transportar demanda por seção no perfil de raios,
  mantendo o mínimo autoral e sem feedback de topologia.
- Ensaio horizontal com material muito mole saiu do domínio angular do modelo
  reduzido. A recusa foi mantida; o oráculo usa rigidez compatível com esse domínio.

### Revisão estática final

1. Todos os cálculos são chamadas/CLI; nenhum tick/frame/callback da engine.
2. Receita e curvas permanecem compartilhadas; rede vascular derivada em cache
   por definição imutável. `asdict`/JSON materializam cópias sob comando.
3. Geometria/amostragem usa floats64, IDs estáveis e estruturas tipadas; sem
   `eval`, estado global de randomicidade ou IDs por índice de vértice.
4. Quotas de órgãos/queries/células/triângulos/DOFs/partículas/preparo verificadas
   antes da expansão relevante. Erro conserva a fonte e o estado publicado.
5. Campo da junção é calculado uma vez por nó de grade; vértices por aresta
   compartilhada. Grafo e Jacobianos preparados uma vez; fatoração corrente
   substituída somente quando passo/conjunto quebrado muda.
6. Sem renderer/rede/GPU no cálculo; rasterização externa opcional só apresenta
   vetores. Fonte da engine e build corrente preservados.
7. Nenhuma thread/lock/dependência nova. Caches são derivados locais e limitados;
   modelo físico mantém apenas uma fatoração corrente.
8. Relatórios de evidência usam temporário+fsync+replace. Caminhos atuais
   substituídos, sem coleção de builds ou pastas por tentativa.
9. Oráculos, checks críticos e figuras acima passaram. Sem FPS, WAN, aceite de
   arte ou física de dispositivo inferidos.
10. Custos aceitos: Python/tuplas/listas, matriz densa, quadratura/diferenças
    finitas, polígonos SVG e serialização integral em comandos. São escolhas
    para estudar a matemática; adoção em Rust/engine exige contrato próprio.

Esta unidade entrega implementações iniciais executáveis nos sete eixos.
Continuam fora das garantias: contornos não estrelados, rede vascular biológica,
booleans exatas de meshes autorais, contato físico/fragmentos livres, grandes
deformações não lineares, transferência conservativa crescimento↔dinâmica,
metabolismo/hidráulica e busca automática de apoio em superfícies arbitrárias.
