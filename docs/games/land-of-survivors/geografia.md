# Geografia e relevo de Land of Survivors

Estudo inicial · 3 de novembro de 2026.

O mundo será uma esfera finita do tamanho da Terra. Genshin Impact é a referência para a linguagem das paisagens e a relação entre relevo e exploração. A esfera-base já foi criada; esta etapa estuda formas e critérios antes de desenhar sua topografia.

## Decisões do mundo

| Propriedade | Decisão |
| --- | --- |
| Forma | Esfera finita, contornável e sem borda de mapa. |
| Raio de referência | 6.371.000 m. |
| Diâmetro de referência | 12.742.000 m. |
| Circunferência de referência | Aproximadamente 40.030 km, calculada por 2πR. |
| Área de referência | Aproximadamente 510,064 milhões de km², calculada por 4πR². |
| Fonte atual | Esfera uniforme, altura radial zero e nenhuma operação de relevo. |
| Topografia | Próxima etapa, após o estudo e a composição geográfica. |

O raio adotado corresponde à aproximação de 6.371 km apresentada pela [NASA](https://science.nasa.gov/resource/solar-system-sizes/). O jogo usa uma esfera, sem reproduzir o achatamento terrestre. Ter o tamanho da Terra não define porcentagem de oceanos, posição de continentes, clima, estações ou distribuição cultural.

## O que a referência mostra

As descrições oficiais e as paisagens publicadas permitem identificar famílias distintas de relevo. A tabela separa o conteúdo descrito pelas fontes da interpretação para nosso projeto. Não é um levantamento altimétrico nem uma reconstrução do mapa de Genshin.

| Referência | Observação apoiada pela fonte | Interpretação para Land of Survivors |
| --- | --- | --- |
| Mondstadt | Montanhas, planícies abertas e um lago com ilha central. [Guia oficial](https://genshin.hoyoverse.com/en/map?region=1) | Alternar espaços amplos e legíveis com limites de relevo; deixar a água organizar a paisagem. |
| Liyue | Montanhas, floresta de pedra, planícies e rios; Dihua é uma zona úmida e Jueyun concentra picos. [Guia oficial](https://genshin.hoyoverse.com/en/map?region=2) | Combinar cristas, paredes e fundos de vale; montanha e planície precisam de relações reconhecíveis. |
| Dragonspine | Maciço montanhoso congelado ao sul de Mondstadt. [Desenvolvedores, 2020](https://blog.playstation.com/2020/12/11/genshin-impact-v1-2-get-ready-to-chill-in-dragonspine/) | Uma grande massa pode organizar a orientação regional, com aproximação, encostas e passagens próprias. |
| Inazuma | Arquipélago de seis ilhas principais e ilhotas, com paisagens distintas. [Desenvolvedores, 2021](https://blog.playstation.com/2021/07/09/genshin-impact-version-2-0-all-about-inazuma/) | Desenhar cada ilha como unidade de paisagem e circulação, articulada pelo mar. |
| Sumeru | Contraste entre floresta úmida e deserto; este inclui dunas, cânions e oásis. [Desenvolvedores, 2022](https://blog.playstation.com/2022/09/16/genshin-impact-version-3-1-a-journey-through-the-desert/) | A identidade regional deve estar nas formas do terreno, além da cobertura vegetal ou cor. |
| Fontaine | Chegada por uma grande cachoeira, canais e áreas de exploração subaquática. [Desenvolvedores, 2023](https://blog.playstation.com/2023/08/04/genshin-impact-version-4-0-launches-august-16-first-details/) | Relacionar patamares, desníveis e bacias; a água precisa de posição vertical coerente. |
| Natlan | Cânions, escarpas, águas correntes, fontes termais e vulcões, associados a travessias verticais. [Desenvolvedores, 2024](https://blog.playstation.com/2024/08/16/genshin-impact-welcomes-the-sixth-major-nation-of-teyvat-with-version-5-0-going-live-august-28/) | Avaliar a circulação junto das paredes e dos desníveis, evitando uma região visualmente rica mas impraticável a pé. |

O guia oficial foi consultado também em sua página renderizada, incluindo os detalhes de Liyue. As imagens de apresentação de Mondstadt e Liyue mostram planos de terreno em níveis e recortes de encosta. Esta leitura visual é nossa análise de composição, não uma declaração sobre os algoritmos usados pelos desenvolvedores. A [apresentação de Haoyu Cai no GDC 2021](https://www.gdcvault.com/play/1027539) fornece contexto sobre um mundo estilizado, sem estabelecer dimensões ou regras de geração para o nosso planeta.

## Princípios de composição a desenvolver

Os critérios abaixo são propostas derivadas do estudo para Land of Survivors. Não são especificações atribuídas ao motor de Genshin.

### Grandes formas antes do detalhe

Uma região precisa funcionar em silhueta e em uma visualização simples de alturas. O relevo principal estabelece a bacia, o divisor de águas, a serra, o planalto ou a ilha. Formas secundárias definem ombros, esporões, vales laterais e passagens. Ondulação e detalhe entram depois.

Um campo de ruído uniforme não define, por si só, uma geografia regional. Picos repetidos ou calotas isoladas também não substituem uma cadeia com crista, encostas, depressões e relação com os vales.

### Patamares e rupturas de declive

Planícies e áreas de descanso precisam coexistir com escarpas, paredes e desníveis. A transição pode incluir rampas, vales ou selas, em vez de depender exclusivamente de escalada. Uma paisagem inteira com a mesma inclinação perde hierarquia; uma paisagem inteira plana perde estrutura.

As paredes podem comunicar uma direção regional, enquanto os patamares oferecem áreas legíveis para exploração. A quantidade e a frequência desses elementos ainda precisam de estudo em protótipos próprios.

### Circulação acompanha o relevo

Um vale conduz; uma crista separa; uma sela permite atravessar; uma garganta concentra a passagem. Mesmo sem estradas ou construções, o terreno precisa sugerir continuidades e escolhas.

O personagem não terá automaticamente os meios de travessia de Genshin. Nesta etapa, nenhuma habilidade de escalada, gancho ou transformação de outra franquia é incorporada. As rotas futuras precisam considerar as capacidades que Land of Survivors realmente oferecerá, incluindo a distinção entre acesso a pé e por montaria voadora.

### Silhueta e revelação

Um maciço dominante pode orientar de longe. Vales laterais e ombros podem ocultar e revelar a próxima paisagem durante o percurso. A composição deve oferecer vistas de fundo de vale, meia encosta e crista, e não depender de um único enquadramento aéreo.

A identidade de uma região precisa continuar reconhecível com material neutro. Cor, vegetação, iluminação e arquitetura poderão reforçar essa identidade depois, sem substituir o desenho do terreno.

### Água e terreno como uma composição

Rios, lagos, costas e cachoeiras devem ser planejados junto dos desníveis e das bacias. Definir uma linha azul em um mapa não estabelece leito, margem, nascente, saída ou declividade.

O planeta ainda não tem nível de mar, rede de drenagem nem proporção oceânica definidos. Hidrografia contínua, regiões sem saída e exceções de fantasia precisarão de regras explícitas. Não presumir erosão ou hidrologia simulada a partir da aparência final.

## Como adaptar para uma esfera terrestre

A escala do planeta e a escala de uma paisagem são decisões distintas. Um globo com dezenas de milhares de quilômetros de circunferência pode conter regiões densas e legíveis; ampliar uniformemente um pequeno mapa também ampliaria seus intervalos, encostas e tempos de deslocamento.

Propõe-se trabalhar em três escalas:

1. **Planetária:** continentes, oceanos e grandes bacias. Ainda faltam quantidade, posição e proporção de terra/água.
2. **Regional:** redes de serras, planaltos, vales, zonas úmidas e arquipélagos, com identidade e conexões próprias.
3. **Local:** paredes, patamares, ombros, gargantas, margens e passagens avaliados na escala de caminhada.

As mesmas regras locais podem variar ao longo de uma região extensa, sem transformar o planeta inteiro em cópias da mesma montanha. A continuidade global precisa atravessar polos e limites internos de representação sem emendas autorais.

Esse caminho não exige copiar a forma dos continentes da Terra nem reproduzir as nações, lugares, lore ou mapa de Genshin. Também não estabelece que Teyvat tenha a mesma geometria planetária escolhida para Land of Survivors.

## Próxima elaboração de topografia

Antes de acrescentar relevo definitivo à esfera, preparar:

1. Um esboço global de massas de terra e água, com hemisférios e polos identificados.
2. Uma região de estudo original, com contraste entre área aberta e cadeia montanhosa, sem cidades ou vegetação.
3. Um desenho em planta de cristas, divisores, vales principais, passagens e bacias; acompanhar com cortes de altura.
4. Variantes que comparem formas arredondadas, encostas estruturadas e patamares, em material neutro e com as mesmas câmeras.
5. Percursos de inspeção no vale, na encosta e no alto, registrando largura das passagens, inclinações e visibilidade.
6. Avaliação artística antes de expandir essa composição para outras regiões do planeta.

Larguras, alturas, distâncias e densidade de marcos serão parâmetros de projeto próprios. Nenhum valor em metros deve ser apresentado como medida de Genshin sem uma metodologia de medição. O estudo atual é suficiente para organizar a primeira experiência de relevo, mas não escolhe silenciosamente a distribuição de continentes ou a topografia final.

## Limites desta etapa

Apenas a esfera-base nativa foi construída. Não há relevo inspirado em Genshin aplicado, oceanos visuais, rios, biomas, vegetação, cidades, construções, fauna ou personagem artístico. Existe somente o observador técnico requerido pelo adapter planetário do Ignis.

O material cinza da esfera é uma superfície de inspeção. Não representa água, rocha final ou direção de clima. A captura nativa comprova abertura e renderização da base, separadamente de desempenho, qualidade artística e aceitação do futuro terreno.
