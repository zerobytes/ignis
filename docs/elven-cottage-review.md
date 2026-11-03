# Revisão arquitetônica independente da casa

Pedido: avaliar por conta própria o que diferencia a casa da referência da casa
autorada, especialmente sua constituição espacial. Esta unidade registra o
diagnóstico antes de alterar o modelo. A cena/script/receita de ea8c6b8 permanecem
intactos; não é uma nova reconstrução nem aceite de fidelidade.

## Evidência e método

Referência1536×1024: `/Users/zero/Downloads/Imagem do ChatGPT 23 de out. de 2026, 20_54_15.png`,
SHA256209cd5cb91c216100413d6b175b5ce29a70cab751a0eda710e4baf95895c6553.
Render apresentado: `artifacts/elven-cottage/artifacts/hero.png`, SHA256
af437ce49ba3256d4bbe747b53b25de2709c0cb7398aa503e0b2df626108e488.

Foram inspecionadas as duas imagens inteiras, dez recortes reais e três novas
capturas nativas: frente, extremidade da empena e o mesmo azimute com câmera mais
baixa. Evidências em [elven-cottage-review](../artifacts/elven-cottage-review/).
`manifest.json` registra origens/hash e caixas[x0,y0,x1,y1]; não houve alinhamento
de perspectiva, retificação ou cálculo de porcentagem de fidelidade.
`native-views.json` registra câmeras, build ec96a3441fef43e2 e preservação da cena.

| Ficha | Recorte de referência | Recorte do render | Pergunta observada |
|---|---|---|---|
| Entrada |625,300,1035,870|625,450,865,850|Existe um pórtico amplo distinto da porta?|
| Coberturas |600,40,1480,520|650,175,1230,600|Como empena, varanda, entrada e cobertura da bay se encontram?|
| Varanda |70,380,705,860|270,430,755,835|Quais seções sustentam os arcos e quais são seus vazios?|
| Bay |990,250,1485,870|780,470,1205,925|Como o volume se relaciona com a extremidade da casa?|
| Rendilhado |145,410,640,710|350,470,670,775|São nervuras esculpidas, caixilhos ou traços de espessura uniforme?|

Os recortes têm escalas/enquadramentos independentes. Servem à leitura de
subpartes; não permitem medir proporção global comparando seus tamanhos exibidos.
Apenas uma vista do objeto de referência existe: orientação absoluta em planta,
profundidades ocultas e número total de janelas continuam hipóteses.

## Diagnóstico principal

A autoria reconheceu um inventário — varanda, arco, porta, bay, telhas, motivo de
árvore — mas não verificou suficientemente as relações que compõem a arquitetura.
O resultado tem uma fachada longa com uma bay frontal e uma empena lateral.
Na referência, entrada, elevação ornamental e volume arredondado formam uma
composição diferente. Acrescentar manchas ou subdividir tubos não resolve essa
diferença. A revisão anterior descreveu o restante sobretudo como acabamento;
essa classificação foi insuficiente e fica corrigida por este diagnóstico.

### A. Entrada e pórtico — diferença estrutural confirmada

Na referência, um arco exterior amplo assenta em suportes de pedra e enquadra um
trecho de parede com a porta ogival menor, múltiplas molduras reentrantes, lanterna
e vaso. Há área legível dos dois lados da porta e acima dela, um patamar coberto
e transição clara para a escada. O arco exterior e o contorno da porta têm funções
e proporções distintas.

No render, a entrada aparece como uma fresta junto à bay: porta alta parcialmente
encoberta, pórtico quase desaparecido sob o beiral e pequena faixa de circulação.
O modelo contém um arco exterior; sua simples presença no inventário não prova
que a composição funciona. A vista frontal e a câmera mais baixa mantêm o
estreitamento. Portanto a ausência visual não se explica apenas pelo primeiro
enquadramento.

Confirmação autoral: `walls()` coloca a porta emz1.265; a frente dos pilares está
emz3.15 e o arco exterior emz2.12. Esse recuo existe. Porém o arco exterior tem
centrox.85/largura2.02: seu lado direito emx1.86 avança na faixa da bay, cujo
extremo esquerdo em planta está emx1.23. Isso é sobreposição de intervalos no
eixoX, não prova de colisão física ou medida de clearance. O limite visual do
conjunto não foi validado contra o volume vizinho.

Critério antes do detalhe: reconhecer separadamente arco exterior, parede
recuada, porta menor e patamar em uma captura neutra; a bay não deve engolir a
entrada. Não mudar apenas a largura da folha de porta.

### B. Bay, empena e orientação do conjunto — hipótese espacial forte

Na referência, a empena alta ornamentada aparece sobre a cobertura arredondada:
as curvas de pedra descem de seu pináculo e enquadram o telhado inferior em leque.
Esse conjunto ocupa a extremidade direita da composição. A entrada fica ao lado,
com uma transição alta de pedra que faz parte do desenho da cobertura.

No modelo a bay foi construída como projeção para+Z, na mesma frente da varanda.
A empena pertence ao planoX direito do bloco principal. A vista da extremidade
torna a dissociação explícita: abaixo da empena existe uma parede retangular
grande, e a bay aparece deslocada para um lado. Também no hero o pináculo não
organiza o volume arredondado como na referência.

Confirmação: `bay_windows()` usa x=cx+r·sin(a),z=cz+r·cos(a),a=-π/2..π/2,
portanto todas as faces avançam para o semiplano+Z a partir do centro.
`roofs()` fecha a empena emx=right constante. Esses eixos são decisões nossas;
a imagem não forneceu coordenadas métricas nem autorizou inferir tal planta.

Uma hipótese melhor é uma bay vinculada à extremidade da empena, possivelmente
avançando em direção diferente da adotada. A orientação exata precisa ser
confrontada com a projeção da referência; não é correto declarar uma rotação
obrigatória de90° ou uma planta oculta única a partir deste PNG.

Critério: avaliar juntas a posição relativa do pináculo, o campo ornamental,
a cobertura inferior e a projeção das janelas, além da faixa livre da entrada.

### C. Coberturas e sua transição sobre a entrada — diferença confirmada

A referência tem hierarquia legível: ampla água curva sobre a varanda; sua borda
sobe junto à entrada até a empena ornamentada; abaixo aparece outra cobertura
arredondada, com limite e encontros esculpidos. Pedra, campos verdes e nervuras
definem a transição. Não se trata apenas de uma inclinação lisa com acabamento
aplicado sobre ela.

A autoria usa uma lei de altura quase independente deX para o telhado principal,
um beiral que se aproxima do círculo da bay e um segundo telhado sobreposto.
Isso produz uma grande superfície contínua de telhas. O arco da entrada fica
subordinado a esse beiral; faltam a elevação e o desenvolvimento das molduras de
pedra que ligam o pórtico à empena. A haste dourada no plano da empena não
representa essa peça arquitetônica.

Uma câmera mais baixa reduz a área aparente do telhado e melhora a apresentação,
mas não cria os volumes ausentes. Não atribuir ao fov todos os desvios de forma,
nem copiar diretamente o ângulo aparente da cumeeira sem resolver perspectiva.

Critério: traçar os limites visíveis das coberturas e suas junções antes das
telhas; validar sua hierarquia em modelo de massas sem ornamento.

### D. Varanda, suportes e base — diferença de seção e proporção confirmada

Referência: pilares de pedra com seção predominantemente quadrada/chanfrada,
pedestais, capitéis, faixas e molduras. As nervuras ramificadas têm largura e
seção variáveis, subdividem os vãos e terminam nos contornos de forma construída.
O guarda-corpo inferior tem presença própria, com peças de pedra, vazios ogivais
e pequenas ramificações. A galeria é aberta e deixa ver profundidade/interior.

Modelo: coluna por seção elíptica, arredondada e esbelta; moldura por tubos e
uma árvore de traços. O guarda-corpo se tornou uma faixa de arcos pequenos,
visualmente mais baixa e delicada. Vários traços terminam sem um encontro
arquitetônico convincente. A mesma família é repetida quase sem variar sua
função. A referência também repete motivos; a repetição em si não é o defeito.

Há três vãos frontais grandes em ambas as leituras. Acertar essa contagem não
comprova seção, proporção ou desenho dos vazios. A parede de fundo no modelo
reduz a profundidade percebida, mas o PNG não permite reconstruir toda a galeria
oculta. Sua posição exata fica pendente.

Escolher ellipse na coluna foi uma decisão de autoria, não limitação demonstrada
da engine: seções rectangle/diamond já existem. A capacidade necessária para
novas nervuras deve ser avaliada com uma peça isolada, não presumida ausente.

### E. Janelas da bay e rendilhado — diferença de desenho confirmada

A referência mostra dois vãos grandes bem legíveis e pelo menos um trecho lateral
mais estreito/parcial. Seus caixilhos e nervuras têm tronco, bifurcações espessas,
formas de folha e variação de seção; há peitoris, ressaltos, campos entre arcos e
decoração na base. O vidro permite perceber elementos atrás dele.

Na autoria há cinco painéis iguais em um semicírculo, sem evidência suficiente
para esse total específico. Eles ficam muito esbeltos e o campo verde opaco
domina a leitura. O perfil geométrico dos motivos foi reduzido a traços de tubo;
um ruído de material ou normal smooth não recupera a seção nem seus vazios.

Não concluir que a referência tem exatamente três janelas no objeto completo:
isso confundiria faces visíveis com contagem total. Primeiro ajustar orientação,
ritmo e largura projetada; depois avaliar vidro/material.

### F. Detalhes secundários e apresentação

A porta, chaminé, cornijas e pináculo da referência têm múltiplos relevos e
espessuras; no estudo vários viraram um contorno simples. O acabamento da pedra
tem bordas, juntas, patina e variação de escala, e o metal aparece sobretudo como
caixilharia e acento. Nosso modelo tem pouca diferenciação entre essas funções.
A lanterna foi autorada, mas não é legível como na referência.

Vegetação e pavimentação também divergem: vasos com massas florais, trepadeiras
aderidas à arquitetura e pavimento irregular versus plantas dispersas e uma
faixa de lajes muito regular. São diferenças reais, com prioridade posterior à
composição. A contagem aproximada de degraus e a presença do patamar não foram
identificadas como erro principal: a escada deve ser reavaliada ao definir a
entrada, sem inventar um defeito de contagem.

A câmera atual é mais elevada e mostra mais cobertura e menos profundidade de
entrada. Luz, vidro opaco e ausência de sombras de contato ricas também afetam
a impressão. São fatores de apresentação separados das decisões geométricas
comprovadas acima.

## Aprendizado para a ferramenta e para o processo

O procedimento precisa registrar relações verificáveis além da lista de peças:
qual volume se projeta de qual face; em que plano está cada arco; quem enquadra
quem; onde os beirais se encontram; quais vazios precisam continuar legíveis;
quais contagens foram vistas e quais foram inferidas. Um campo "porta presente"
não detecta uma porta quase inteiramente ocluída por uma peça vizinha.

Antes de outro acabamento, produzir uma hipótese simples de implantação e
volumes, com incertezas explícitas, e uma vista de massas comparável à referência.
Critérios por relação: entrada ampla/legível; empena e bay formando conjunto;
hierarquia das coberturas; pilares/guarda-corpo/vazios com proporção coerente.
Depois desenvolver um módulo de arco e um de caixilho com seções corretas e só
então repetir. A próxima revisão deve poder reprovar uma relação mesmo que
todos os nomes dos objetos existam e o replay seja determinístico.

Isso é uma recomendação de método e de possíveis contratos genéricos, não uma
implementação aprovada de solver arquitetônico, grafo semântico ou CAD por imagem.
A divisão entre erro de autoria e capacidade ausente exige um caso mínimo real.
Os596vértices restantes limitam detalhe adicional nesta cena, mas não explicam
as escolhas de implantação ou o uso de seção circular. Corrigir massas pode
envolver redistribuir geometria, sem simplesmente elevar limites.

## Validação deste incremento

Unidade documental e de inspeção: build corrente, sem código Rust, shader,
novo build ou mudança de geometria. Dez recortes conferidos visualmente;
três capturas1536×1024 via MCP em processo próprio headless. Após restaurar a
câmera, o estado da cena é igual ao inicial; arquivo de autoria SHA256
7286e74c023f7319331589a5828a372a389d44619e203356a3449367802f1e2c preservado.
O hash do render apresentado e da referência também foram registrados. Não há
nova alegação de testes completos, melhoria de fidelidade ou aceite humano.

Checklist proporcional antes/final:1.adequado,inspeção sob demanda;
2.adequado,dez recortes/um manifesto/três capturas finitos;
3.adequado,coordenadas de pixel e fontes com hash, hipótese separada de dado;
4.adequado,reuso das imagens e do build, sem nova geração da casa;
5.adequado,1536×1024 e regiões explicitadas;6.mantido com motivo,readback somente
nas três capturas pedidas pela inspeção;7.não se aplica,sem lock/worker novo;
8.validado,câmera restaurada e autoria igual;9.adequado,cena e ferramenta
preservadas;10.mantido com motivo,perspectiva/planta ocultas indeterminadas por
imagem única. Revisão artística não se torna prova métrica de geometria.
