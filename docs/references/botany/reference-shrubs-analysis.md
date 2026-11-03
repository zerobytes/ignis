# Sete referências de arbustos — análise antes de construir

Pedido: acrescentar os arbustos das sete imagens fornecidas. Não reconstruir
dunas, jardim, montanhas, céu, marcas-d'água ou textos. Identificação de espécie
não é comprovada pelas imagens; os perfis terão nomes descritivos. As fotos
guiam regras e proporções, sem fornecer vértices ou texturas.

Estado desta entrega: **nenhuma nova curva/fonte construída; comparação dos
candidatos não executada**. Todas as tarefas abaixo estão não construídas.

## Conjunto, fonte e recortes reais

Ordem R1..R7 é a ordem do pedido. Originais e SHA256 ficam no manifesto de estudo
`artifacts/reference-shrubs/references/manifest.json`. Arquivos originais são os
anexos `codex-clipboard-*`, preservados nesse diretório ignorado. Imagens de
estoque mantêm suas marcas; não serão incorporadas aos assets ou ao Git.

Coordenadas de crop `(x,y,w,h)`, origem superior esquerda, sem resize/rotação:

| Fonte/resolução | Organização global e evidência | Crops abertos |
|---|---|---|
| R1,600×420 | Plano de areia com arbustos separados; moita esquerda alta cinza/castanha com ramos expostos; primeiro plano direito baixo verde-acinzentado e irregular. Muitas plantas menores ao fundo, sem detalhe foliar resolvido | (60,165,210,160) e (210,210,185,130) |
| R2,1600×1109 | Moitas em almofadas sobrepostas/lobadas no primeiro plano, uma mais alta atrás. Núcleo castanho de ramagem e pontas verde-amareladas salpicadas de pequenas cabeças claras; ramos nus secos à frente | (390,490,490,440), (820,300,480,300), (540,570,240,240) |
| R3,1749×980 | Arbusto verde em cúpula oval, base multi-haste de madeira exposta, folhas elípticas estreitas/pontudas e brotos ascendentes. Duna esconde parte das inserções inferiores | (630,490,490,180) e (850,220,270,280) |
| R4,1749×980 | Touceira compacta de ramos verdes ascendentes, folhas longas muito estreitas, faces claras e escuras, ponteiros livres. Base parcialmente enterrada/oculta pela areia | (870,180,360,320) e (790,450,300,160) |
| R5,1000×667 | Esfera/cúpula podada muito densa à direita; repetição de esferas atrás em perspectiva. Folhas pequenas ovais de verde escuro e terminais amarelo-verdes. Ramos quase invisíveis | (660,170,240,240), conjunto inteiro |
| R6,1715×980 | Arbusto isolado arredondado irregular de folhas maiores ovais/pontudas, camadas sobrepostas e brotos claros que sobressaem. Fundo preto não demonstra profundidade/traseira | (680,220,380,380), conjunto inteiro |
| R7,1080×1080 | Vários arbustos de viveiro; indivíduo central com massa oval alta, muitos ramos quase verticais, folhas pequenas repetidas e pontas claras. Base/madeira ocultas | (360,270,360,370), conjunto inteiro |

Os conjuntos inteiros foram fornecidos no pedido e revisitados depois dos crops.
As contagens totais de folhas/galhos, raízes e faces ocultas são desconhecidas.
Perspectiva, oclusão e ausência de escala impedem alturas/espessuras métricas.
Unidade autoral será metro/Y para cima; escala e profundidade serão hipóteses.

## Segunda leitura e efeitos na construção

- R1: o crop separa dois hábitos que um único perfil não cobre: ramagem seca alta
  e moita baixa folhosa. Criar dois perfis; folhas finas não são identificáveis
  individualmente nessa resolução. Não transformar textura da marca em galhos.
- R2: as cabeças claras repetem nas terminações e não são folhas largas. Criar
  pequenos órgãos arredondados separados por cor, sustentados por raminhos.
  Identidade flor/fruto e número de pétalas permanecem desconhecidos. O núcleo
  inferior tem ramagem nua distinta do manto superior. Vários lobos se sobrepõem;
  não usar uma esfera lisa única. Subcrop de240×240 confirma cabeças e ramagem,
  mas não resolve pétalas; parar nessa fronteira de resolução.
- R3: o crop basal mostra várias hastes, arqueando para fora antes da folhagem;
  sustentar a copa com um único tronco seria divergente. O crop da folha confirma
  lâminas maiores que R5/R7 e brotações no eixo ascendente.
- R4: a textura fina vem de lâminas compridas estreitas e de ramos visíveis,
  não de folhas ovais em escala reduzida. Construir lâminas orientadas para cima
  e alternar faces/azimutes. A areia oculta a base: profundidade enterrada é hipótese.
- R5: o crop confirma pequena folhagem sobreposta e pouco espaço entre terminais.
  A esfera tem superfície feita de folhas, pequenas irregularidades e fundo
  escuro entre órgãos; uma bola opaca verde não satisfaz o item.
- R6: em contraste com R5, lâminas maiores e grupos terminais claros rompem o
  contorno. Construir tamanho e contorno próprios, não só outra cor de R5.
- R7: os ramos terminais ascendentes alongam a massa e diferenciam a silhueta
  da topiaria redonda. O crop não permite identificar cada nó nem a espécie.

Revisita do contexto: moitas pequenas/fundos em R1/R2, árvores de folhas largas
ao fundo de R2, dunas/sombras R3/R4, esferas repetidas/jardim em R5 e vizinhos em
R7 são contexto. Repetições de arbustos serão exemplificadas pelo plantio; o
pedido é ampliar a biblioteca, não copiar posição/cenário de cada fotografia.
R6 permite olhar detalhe foliar, mas não prova raízes/interior/verso.

## Lista construtiva consolidada

Agrupadores A..H são perfis. Cada linha executável tem estado **não construído /
validação não executada**. Repetições indicam famílias de órgãos, não contagens
exatas extraídas dos pixels. Relações de suporte serão geradas por curvas reais;
ramificação interior invisível é uma hipótese autoral.

| ID/pai | Construir, evidência e ocorrência | Relações/estado da observação | Conferir no candidato |
|---|---|---|---|
| A1/A | Moita cinzenta alta aberta, R1 crop0 | Diversas hastes curvas basais e divisões finas; observado | Madeira aparece no miolo e na base; volume em3 vistas |
| A2/A | Muitos raminhos finos nus e terminais esparsos, R1 crop0 | Ligados às hastes, secos expostos; lâmina individual desconhecida | Rede de ramos legível, sem preencher com folhas grandes |
| A3/A | Cinza-verde discreto + madeira palha/castanha, R1 | Aparência observada, pigmentação/luz não separáveis | Contraste natural de partes secas/folhosas |
| B1/B | Moita baixa larga assimétrica, R1 crop1 | Múltiplos centros/lobos parcialmente sobrepostos; observado | Perfil achatado irregular e3D real |
| B2/B | Ramagem fina com folhagem pequena cinza-verde, R1 crop1 | Repetida nos raminhos; folhas exatas ilegíveis | Textura mais cheia e baixa que A, sem folha larga inventada |
| C1/C | Almofada lobada, R2 crops0/1 | Ramos nascem em mais de um centro próximo; observado | Lobos e vãos diferenciam de esfera podada |
| C2/C | Rede lenhosa nua na base e ramagem fina superior, R2 crops0/2 | Suporte das cabeças; observado | Núcleo castanho exposto sob ponteiros verdes |
| C3/C | Pequenas cabeças claras redondas nos terminais, R2 crop2 | Várias por raminho; flor/fruto desconhecido | Pontos claros como órgãos3D discretos; sem pétalas fictícias |
| D1/D | Cúpula verde oval com base multi-haste, R3 crop0 | Hastes arqueadas sustentam grupos foliares; observado | Base aberta e copa mais larga que alta |
| D2/D | Folhas elípticas pontudas + grupos terminais claros, R3 crop1 | Ao longo de ramos ascendentes; observado | Lâmina ampla distinta de R4/R5, broto menor |
| E1/E | Cúpula compacta ascendente, R4 | Base parcialmente oculta; volume traseiro é hipótese | Planta tem profundidade e ponteiros livres |
| E2/E | Folhas longas muito estreitas, R4 crop0 | Repetidas próximas dos ramos e apontando para cima; observado | Comprimento/largura e orientação separam E de D |
| E3/E | Verde cinza/escuro e pontas mais claras, R4 | Reflexo e pigmento desconhecidos separadamente | Variação de direção/tonalidade sem pintar textura da foto |
| F1/F | Esfera/cúpula podada, R5 conjunto | Centros internos/ramificação são hipóteses | Contorno cheio, redondo e baixas irregularidades |
| F2/F | Folhas ovais muito pequenas densamente sobrepostas, R5 crop | Ramos quase ocultos; observado | Densidade efetiva de folhas e ramos, sem proxy opaco |
| F3/F | Brotos claros menores nas terminações, R5 crop | Sobre manto verde escuro; observado | Contraste localizado nos terminais |
| G1/G | Esfera folhosa irregular, R6 conjunto | Ramos suportam camadas em várias profundidades; hipótese3D | Contorno irregular e volume coerente em vistas auxiliares |
| G2/G | Folhas ovais/pontudas maiores + brotos em rosetas claras, R6 crop | Grupos repetidos e sobrepostos; observado | Folha maior e agrupamento distintos de F |
| G3/G | Discreta dobra central/nervura aparente, R6 crop | Nervuras finas/serrilha não resolvidas | Dobra aproximada; detalhe vascular fino será lacuna explícita |
| H1/H | Copa oval alta, ponteiros ascendentes, R7 conjunto | Ramificação oculta inferida; terminais observados | Mais alta e vertical que F/G |
| H2/H | Folhas pequenas repetidas em pares/grupos, R7 crop | Nós/filotaxia exata não confirmados | Grupos foliares ao longo de eixos reais, verde e brotação clara |

Detalhes microscópicos, raízes não vistas, redes vasculares soldadas, contagem
de pétalas e identificação científica não são comprovados. Não serão alegados.
Não eliminar C3 por custo: construir cabeças discretas; sua anatomia fica limitada
pela referência. A aparência das imagens não basta para identificar uma espécie.

## Gramática, capacidades e sequência

Oito perfis canônicos em `assets/botany/reference-shrubs`, três variações seeded.
Ramos basais → ramos principais → brotos → raminhos terminais, com envelope
elipsoidal, lobos/assimetria e orientações por perfil. Folhas/cabeças nascem em
pontos dessas curvas; manto denso é feito de órgãos reais. Topiaria usa envelope
autoral de crescimento, não clipping pela câmera nem bola substituta.

Reusar Source/Recipe7 e canais madeira/folha, mais um componente para brotação
ou cabeças claras. Altura/alcance/espessura são autorais. Consultar schema vivo
antes de publicar. Não alterar Rust, formatos, quotas, física ou renderer.
As quatro coleções existentes devem regenerar iguais e manter seus PNGs locais.

Gate estrutural: primeiro exemplares neutros frente/costas/topo e fechamento
por parte; depois oito cartões coloridos e recortes locais de cada candidato,
comparados qualitativamente às referências. Câmeras/luz independentes, sem score
de fidelidade RGB. Save/load nativo compara a mesma câmera/resolução e registra
qualquer diferença segundo a guarda existente. Fonte→captura→diagnóstico por ID
será registrada no documento final. Aceite artístico continua com o usuário.
