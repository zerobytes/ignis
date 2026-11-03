# Rede lenhosa e ordens de brotação — EAF-197

Continuação posterior: [sistemas de raízes EAF-198](botany-roots.md) substitui
a aba basal da RS e adiciona Source/Recipe5. As contagens/builds desta página
registram a etapa anterior de ramificação; a copa autoral foi preservada.

Continuação de [figueira RS](botany-rs.md) e [arquitetura](botany-architecture.md).
O usuário identificou que os exemplares tinham poucos galhos. A nova foto de
uma figueira próxima mostra a rede interna e confirma essa falha visual;
[a análise e lista RS12–18](references/botany/figueira-house-analysis.md) foram
entregues antes da geração. A foto anterior orienta a silhueta externa; são
árvores/enquadramentos distintos, sem taxonomia/escala/idade calibradas.

## Matemática

A nuvem de destinos continua a ser repartida em subcopas por planos seeded.
A seção continua alocada por carga potencial:

`r_filho = r_pai · tip_ratio · (N_filho/N_pai)^(1/p)`.

A diferença é que o centro da subnuvem **influencia a direção**, em vez de ser
um ponto que todo galho precise alcançar. Na versão anterior, forçar um galho
longo a terminar junto de um destino já muito próximo produzia grandes retornos.
A primeira tentativa de apenas aumentar a quantidade agravou esses arcos.
Ela passou em topologia, mas falhou na revisão visual. Reduzir indiscriminadamente
o espaço de junção ou aproximar as direções dos irmãos também foi recusado
pelo kernel: não houve substituição por tubos atravessados.

Agora cada trecho estrutural avança na sua direção de saída. Sendo `d` a ordem,
a escala nominal de comprimento é:

`L_d = spread · structural_step · structural_decay^d`.

Há variação determinística de comprimento e um mínimo proporcional à seção
para acomodar as junções. Os irmãos saem em direções divergentes em um plano
local orientado pela partição3D. `target_bias` mistura essa direção com a direção
do centro da subcopa. `early_vertical`, `uplift` e `forward_min` regulam a
componente vertical e a mudança de orientação. `crown_floor` é uma altura
mínima autoral para os destinos estruturais; ajusta o crescimento, não corta
ou oculta a malha. Não representa simulação de colisão biológica com o terreno.
A curva Bézier/Hermite continua produzindo deformação entre os extremos, com
alavancas, arqueamento e perturbação seeded. Os endereços dos órgãos continuam
canônicos para a fonte gerada; regenerar parâmetros topológicos não preserva
automaticamente esculturas feitas na receita anterior.

Cada broto pode ter `shoot_orders` ordens adicionais antes de receber folhas.
Por exemplo, duas ordens geram `1+2+4=7` eixos e quatro terminações foliares por
broto inicial. `shoot_length_ratio` reduz o comprimento por ordem;
`shoot_fork_forward` define o ângulo local. Os raios dos filhos seguem a mesma
partilha por potência das seções, com o piso geométrico já existente. Um broto
intermediário termina em forquilha; só as terminações recebem as lâminas.
O arqueamento/ruído desses brotos também é limitado pelo próprio comprimento,
evita-se aplicar a mesma flecha absoluta a um ramo cada vez mais curto.

`reiteration_sites/orders/length` acrescentam ramos laterais em eixos interiores
de ordens1..6, com espaço métrico para uma junção. São48sítios na RS,32na
ornamental,16no eucalipto e12no plátano. A espessura deriva da seção hospedeira,
sem o teto fino dos brotos terminais; a direção favorece o centro/alto da copa,
respeitando a saída divergente. Esses ramos têm forquilhas e terminações próprias.
Assim, a rede não cresce apenas nas pontas da árvore. Colares e anatomia são
validados pelo mesmo compilador; falha de encaixe continua sendo recusa.

A figueira RS usa512destinos e duas ordens de brotação. A ornamental usa384e
uma ordem; o eucalipto192e uma; o plátano96e uma. A palmeira conserva os caules,
frondes/folíolos anteriores, sem receber ramificação lenhosa de árvore.
Menos folhas por hospedeiro liberam orçamento para mais galhos. A quantidade
de folhas não é usada como medida de ramificação, e os2880pendentes da RS
continuam excluídos das estatísticas lenhosas.

## Anéis quase coincidentes e fechamento

A densidade nova revelou uma falha real no bake: uma amostra longitudinal
regular ficava muito próxima de um colar. As duas seções podiam arredondar para
os mesmos pontosf32; o descarte de triângulos colapsados deixava bordas abertas.
Um caso reduzido de junção lateral reproduziu a falha, além do exemplar completo.
O compilador conserva os parâmetros/endereço exatos dos colares e omite apenas
amostras regulares dentro da resolução representável perto dessas fronteiras.
A margem depende da magnitude das coordenadas e do epsilonf32; não solda
geometrias distintas ou muda a topologia da junção. A ausência de uma amostra
com escultura continua causando recusa explícita, sem apagar esse ajuste.

## Fonte, custo e compatibilidade

Mudam a geração dos exemplares, os assets autorais e a guarda de amostragem
descrita acima. Renderer, interface Vegetação/MCP e formatos
Source/Recipe4/Scene109/replay119 permanecem os mesmos. Os parâmetros são dados do gerador Python, exportados como curvas,
órgãos e relações nativos; não se afirma uma gramática compacta no Rust ou
novos sliders de topologia. Os cinco presets continuam editáveis/publicáveis
pelo painel e pelo MCP existentes. Fontes antigas podem ser carregadas sem
regeneração; não são convertidas silenciosamente para o novo grafo.

Revisão estática: índice `Builder.by_id` referencia os mesmos objetos da lista,
evita varrer todos os órgãos para consultar seção de cada ramo. Expansão por
ordens é limitada a0..3, e os presets usam1/2. Custos de fonte expandida, curvas,
malha, junções e física continuam reais; não foi adicionado trabalho por frame,
shader, passeGPU, rede, lock ou novo cache de runtime. Mesmas quotas de órgãos,
fonte, vértices/triângulos, nós/restrições e contato. A troca de fonte é autoria,
não migração automática dos detalhes esculpidos nem simulação de décadas.
Um único binário corrente; mundo/janela humanos não participam do ensaio.

## Conferência e reprodução

A lista RS12–18 será conferida com fontes e renders reais. O roteiro usa
`099f5d6` como base imutável e a fonte corrente como resultado; o par usa a
mesma câmera/luz, enquadrando os dois conjuntos. A vista inteira preserva as
bordas; a próxima pode cortar a copa, e a interna olha por baixo dos galhos.
As vistas lateral, verso e superior verificam coerência da hipótese3D, sem
validar o verso das árvores reais. A nova foto não é registrada contra o render
nem convertida em um score de fidelidade RGB. Contagens da fonte medem o grafo
gerado, não contam galhos oclusos da foto.

```
python3 scripts/botany_exemplars.py
cargo test --release --test botany
python3 scripts/demo_botany_branching.py --phase structure
# Abrir/comparar as estruturas antes da aparência.
python3 scripts/demo_botany_branching.py --phase appearance
```

O roteiro headless consulta o schema real e guarda fonte/hash, contagens,
proporções de seção, curvas com retorno na projeção da corda, bakes e câmeras em
`artifacts/botany-branching/report.json`. O diagnóstico de retorno inspeciona os
pontos da curva autoral estrutural: não é prova de curvatura mínima, ausência
de autointerseção, comportamento físico ou naturalidade universal.
`review.html` reúne pares de estrutura/copa, inclusive o interior, para os
quatro exemplares lenhosos. A foto própria fica apenas no cache local de análise,
sem integrar os assets distribuídos.

## Resultado da rodada

Build corrente `8fdd3a788c6273df`, macOS arm64/release/rustc1.97.1, em
`target/release/ignis`. Antes/depois usam esse mesmo compilador e renderer;
o antes carrega as fontes de `099f5d6`. Não mistura pixels de dois builds.
Os quatro assets lenhosos foram regenerados; a fonte da palmeira ficou
idêntica. O cenário e a janela humanos foram preservados.

| Exemplar | Eixos lenhosos antes → depois | Estruturais antes → depois | Hospedeiros foliares antes → depois | Folhas antes → depois |
| --- | --- | --- | --- | --- |
| Figueira RS | 1391 → 5055 | 383 → 1023 | 1008 → 2304 | 48576 → 28416 |
| Figueira ornamental | 1867 → 3371 | 511 → 767 | 1344 → 1728 | 50688 → 37120 |
| Plátano | 412 → 1139 | 127 → 191 | 285 → 632 | 6908 → 5312 |
| Eucalipto | 895 → 2159 | 255 → 383 | 640 → 1184 | 19200 → 17760 |

Na RS, o aumento de lenho é3,63×. Não conta folhas nem os2880filamentos
pendentes. Eixos estruturais são os trechos entre forquilhas do esqueleto;
brotos e suas subdivisões também são eixos lenhosos. Essa unidade não equivale
à contagem de braços botânicos inteiros na fotografia. Para conferir que o
incremento não vem apenas de ramos mínimos: os eixos com raio autoral pelo
menos10%/3%/1% do raio inicial do tronco passam de33/178/509 para60/523/1375.
É uma classificação da fonte, não medição de espessura da árvore real.
O diagnóstico de retorno da curva estrutural passa de137 para0na RS,53→0na
ornamental,1→0no plátano e4→0no eucalipto; não certifica autointerseções.

| Exemplar | Fonte MiB | Vértices | Triângulos |
| --- | --- | --- | --- |
| Figueira RS | 63.24 | 2873526 | 5621864 |
| Figueira ornamental | 63.09 | 2120570 | 4092656 |
| Plátano | 11.80 | 2335750 | 4650248 |
| Eucalipto | 31.41 | 1512222 | 2953400 |

Continuam os tetos65536órgãos/96MiB fonte/3Mvértices/6Mtriângulos. A RS tem
36351órgãos e fica próxima do teto geométrico; não é geração sem custo.

15testes Rust release aprovados na fonte final (18,31s), incluindo o corpus
integral dos cinco exemplares, determinismo nativo, pele fechada/orientada,
crescimento/repouso, poda, persistência e preservação/rejeição de escultura.
O caso reduzido novo falhou antes da correção e passou depois; protege a
integridade da malha perto de colares e a recusa de perda de um ponto ajustado.
Build, Clippy lib/bin/botany com `-D warnings`, Python, rustfmt edição2024 e
diff verificados. Os logs ficam em `artifacts/botany-branching/`.

MCP nativo: cinco vistas de estrutura mais a vista próxima, pares com folhas
em quatro vistas; as estruturas foram abertas antes da etapa de aparência.
A RS foi ensaiada com1171380nós/5493386restrições,4passos de vento/4iterações,
erro final de vínculo2,8429e-8m; poda de uma terminação lenhosa manteve identidade
e livro de massa. Reset, salvar/carregar e recapturar produziram PNG bit-exato.
Quotas de contato continuam separadas; não é teste de FPS ou física de gameplay.
Schema, hashes, estatísticas, câmeras, cenas e report são da execução real.

| Lista | Construção e conferência final |
| --- | --- |
| RS12 | Mais destinos e trechos estruturais efetivos nos quatro perfis; fonte e vistas despidas comparadas. |
| RS13 | Duas ordens na RS e uma nos outros; forquilhas terminais e grafts laterais com pele fechada na validação crítica. |
| RS14 | Mais hospedeiros e menos folhas por hospedeiro; fontes e aparência comparadas, com mudança de tamanho/distribuição foliar explícita. |
| RS15 | Rede em vários azimutes e48brotações interiores na RS; vistas auxiliares coerentes. O esqueleto ainda abre em dois braços principais dominantes, diferente da multiplicidade de fustes da foto próxima. |
| RS16 | Mais seções médias/grandes na RS, curvas sem o retorno grande anterior; persistem regularidade binária e simplificações do tronco. |
| RS17 | Ornamental/plátano/eucalipto têm mais eixos, mantendo perfis próprios e raízes aéreas da ornamental; palmeira idêntica. |
| RS18 | Pares com câmera interna e árvore inteira abertos; ponto de vista manual, sem registro ou calibração contra a nova fotografia. |

A revisão visual mostra a rede mais subdividida, braços intermediários
ascendentes e folhagem menos concentrada em poucos brotos longos. A copa RS
continua diferente das referências: patamares menos definidos, algumas franjas
muito baixas e casca/fuste simplificados. O close ainda mostra uma grande
forquilha dominante, enquanto a foto tem muitos eixos grossos sobrepostos.
Faltam a base lobulada/fundida, sulcos volumétricos e tufos epífitos felpudos.
Não se afirma espécie, idade empírica, réplica exata, anastomose ou aceite humano.
A implementação e a validação técnica desta rodada estão concluídas; EAF-197
permanece em revisão artística.
