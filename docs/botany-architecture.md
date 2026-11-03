# Arquitetura de copa e bifurcações terminais — EAF-194

Continuação corrente: [rede lenhosa e ordens de brotação](botany-branching.md).
As contagens e evidências desta página são da rodada `5770b60`.

Exemplar adicional posterior: [figueira campestre do RS — EAF-197](botany-rs.md), preservando as quatro fontes desta rodada.

Continuação de [idade em anos](botany-age.md). O usuário rejeitou a semelhança
visual da revisão anterior: envelhecer o mesmo eixo com braços laterais não
produzia a arquitetura das referências. Aprovação de testes não prova semelhança.

## Matemática e fonte autoral

A fonte/receita4 acrescenta `connection: "fork"`: exatamente dois eixos filhos
se inserem em `u=1` de um pai lenhoso que termina. `graft` continua sendo um ramo
lateral; `sheath` continua sendo uma superfície independente. Nenhuma espécie é
reconhecida pelo compilador ou renderer. Fonte1..3 e receitas antigas continuam
legíveis, com os mesmos defaults. Scene109/replay119 e os envelopes de
conteúdo7/protocolo5/PSV1/checkpoint3 não mudam.

Uma forquilha reúne três colares: fim recortado do pai com normal `-T` e inícios
dos dois filhos com normal `+T`. Reutiliza o kernel nativo de junção orgânica,
com planos separados, refinamento e relaxação. Janelas são buscadas em distância
métrica sobre as curvas atuais, respeitando outros colares. O tubo do pai não
continua escondido além da bifurcação. Não há fallback de tubos atravessados.
Se um filho for podado/ausente cronologicamente, dois anéis são unidos pela
correspondência circular de menor comprimento; se ambos faltam, o pai termina
fechado. Isso preserva uma pele fechada, mas não simula uma cicatriz botânica.
A poda dinâmica continua tendo seu contrato anterior de máscara/massa, distinto
da recompilação da fonte. Endereços de pele removidos ainda exigem recusa ou
política explícita de descarte; publicar uma fonte inválida não altera a cena.

O gerador de exemplares fornece pontos de atração dentro de uma envoltória
elipsoidal perturbada por seed. Cada nuvem é repartida por um plano espacial;
a fração varia por seed para produzir hierarquias assimétricas. O destino de um
eixo avança em direção ao centro da subnuvem. O crescimento inicial dos irmãos
abre para lados opostos do plano de divisão, com componente na tangente do pai.
Destinos atrás da ponta são projetados para seu semiespaço de avanço; a curva
pode então se deformar e terminar em outra direção, inclusive para baixo.

A seção é alocada pela carga potencial de cada subcopa:

`r_filho = r_pai · tip_ratio · (N_filho/N_pai)^(1/p)`.

`p=2,3` é uma regra autoral configurável, não uma lei universal nem uma medição
da foto. Até os pisos de espessura, vale a soma das potências das seções dos
filhos igual à seção terminal do pai elevada a `p`. O afilamento segue o campo
existente. O perfil cronológico destes exemplares aplica a mesma razão radial
aos eixos estruturais para conservar essa distribuição; os brotos permanecem
finos pela seção autorada. A idade continua uma aproximação de desenvolvimento.

Altura, largura, altura da primeira forquilha, espessura, altura da envoltória,
quantidade de brotos/folhas, sítios interiores, atração ascendente, formato/tamanho da folha, pendência e cores são dados
em `CROWNS`, em `scripts/botany_exemplars.py`. `--seed` e `--age` são independentes.
As nuvens usam64destinos no plátano,128no eucalipto e256na figueira; curvas de controle e seções são exportadas, nunca
triângulos obtidos de fotografias. Os três exemplares lenhosos usam o mesmo
algoritmo; a palmeira mantém o gerador de touceira/frondes/folíolos.
A resolução longitudinal das folhas de contorno simples agora é independente
(`settings.blade_samples`,4..32, default12 preservado). Folhas palmadas conservam
sua discretização de contorno. A figueira usa4, passando de114 para34vértices
por folha; o eucalipto usa6/54vértices. Conservam a tesselação transversal,
espessura, dobra e curva autoral, com uma aproximação longitudinal menos densa.
A figueira final tem50.688folhas geométricas, distribuídas em256destinos de
crescimento, sem exceder3M vértices/6M triângulos. A tentativa intermediária com
61.632folhas em apenas64destinos engrossava feixes sem preencher a copa; por
isso a versão final distribui o orçamento em mais ramificações finas. Não usa transparência, textura de copa, fotografia ou imagem gerada.
A física conserva sua discretização/material e calcula cada folha.

Os envelopes passam a65536órgãos/96MiB por fonte e1572864pontos/6M restrições
por ensaio, guardados explicitamente. O limite geométrico permanece3M/6M.
As quotas de contato permanecem separadas e inalteradas. Aumentar uma capacidade
não garante que uma combinação caiba nas outras; reduzir a resolução não promete pixels
idênticos, e a perda de um ponto esculpido é recusada atomicamente. Esse ensaio
é autoral/offline, sem promessa de interação em tempo real com essa densidade.
As fontes expandidas permanecem grandes; uma gramática compacta nativa de
folhagem é uma otimização futura, distinta da geração procedural já existente.

## Reprodução e comparação

```
python3 scripts/botany_exemplars.py
cargo test --test botany
cargo build --release
python3 scripts/demo_botany_age.py --output artifacts/botany-architecture
python3 scripts/verify_botany_native.py
```

`artifacts/botany-architecture/review.html` reúne fotos, render nativo, estrutura
sem folhas, segundo ângulo e idades8/20/50 (palmeira30) com câmera comum por planta.
As fotos ficam no cache de análise, com atribuição nos links da página; não são
parte da geometria distribuída. A avaliação deve comparar silhueta, forquilhas,
espessura e ocupação da copa, sem inferir idade exata dos exemplares fotografados.

Janela e cena humanas não participam do ensaio: um processo MCP headless isolado
usa caminhos estáveis. O build corrente permanece `target/release/ignis`.

## Validação e limites

Build corrente macOS arm64/release/rustc1.97.1: `ae63d9e9655b1f8b`.
Treze testes de botânica aprovados, incluindo os dois riscos novos: continuidade
com poda parcial/total/formato4/repouso mecânico e perda de escultura ao mudar
a amostragem. O teste de forquilhas também exige recusa da escultura interna
quando muda o colar do segundo irmão. Junções laterais mantêm seus endereços.

Após os últimos ajustes de fontes, o cenário existente dos quatro exemplares
foi repetido:120,15s, determinismo das malhas, cada aresta com duas faces opostas
e crescimento/repouso físico da figueira inteira. Clippy lib+botany com
`-D warnings`, rustfmt dos arquivos tocados, Python e diff aprovados.

| Exemplar | Órgãos | Folhas | Vértices | Triângulos | Pontos físicos | Restrições |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Palmeirinha | 4128 | 4032 | 481728 | 946944 | 87936 | 398352 |
| Plátano | 7320 | 6908 | 2778450 | 5529264 | 169472 | 774156 |
| Eucalipto | 20095 | 19200 | 1285726 | 2494648 | 452020 | 2059146 |
| Figueira | 52555 | 50688 | 2242742 | 4282728 | 1155652 | 5252706 |

MCP no build corrente: publicação, idades8/20/adulta, repouso físico integral,
capturas com/sem folhas em dois ângulos e galeria salva/reaberta com PNG
bit-exato. Um ensaio em20anos detectou colares sem espaço nos ramos finais mais
curtos; o mínimo de comprimento relativo à seção foi ajustado no gerador para
acomodar os colares também durante a expansão dos órgãos. Não se relaxou o
kernel nem se inseriram tubos atravessados.

O ensaio MCP extenso da variante intermediária de61.632folhas usou o build
`ede2c73e23d941d5`:1.271.628pontos/5.736.686restrições,32incrementos de expansão
1,8→3,4 e128passos com vento, seguido de poda, livros de massa/momento,
preview/save/replay/portátil, grande flexão/chão e contato de ramos. A expansão
levou525,17s e o erro final de vínculo foi8,84e-9m; pico observado pelo `time -l`
~5GiB RSS, sem swaps. Essa medição é da variante mais carregada de folhas,
não uma medição temporal da fonte final com mais ramificações. O último build
acrescenta a invalidação dos endereços internos pelo segundo colar; o teste
crítico específico e a galeria final cobrem essa mudança.

Evidências: `artifacts/botany-architecture/report.json`, `rust-tests.log`,
`corpus-final.log`, `fork-address-test.log`, `clippy.log`, `build.log`,
`stress-verification.json`, `native-verification.log` e `review.html`.
Os arquivos são substituídos em caminhos estáveis. Não houve push, instalação
nova, mudança no mundo/janela humanos ou coleção de builds.

A estrutura e a ocupação da copa melhoraram, especialmente na figueira, mas
isso não encerra a avaliação artística. As fotos têm arquitetura e relevo de
tronco/raízes que ainda não estão reproduzidos: raízes fundidas/anastomose,
contrafortes/flutuações de seção e uma distribuição menos regular das massas
locais de folhas permanecem abertos. O plátano de referência está sem folhas;
compará-lo à versão despida evita atribuir semelhança à densidade. A planta
de8anos ainda depende da cronologia dos sítios predefinidos e não representa
uma reconstrução botânica calibrada. Sem promessa de FPS, simulação de décadas,
interação em tempo real na densidade máxima ou física de gameplay.
