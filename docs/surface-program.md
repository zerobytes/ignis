# Programa de relevo comum

Implementação do contrato aprovado em `terrain-generators-study.md`: um catálogo
de ferramentas, composição e edição para plano e esfera. O domínio fornece
geometria; o programa não conhece faces de cubo, chunks, câmera, LOD ou renderer.

## Fonte e compatibilidade

`terrain::surface::Program` é Surface1. Terrain3 e Planet2 aceitam a operação
`{"kind":"surface","program":...}`. Ela substitui a altura acumulada pela saída
do programa. O nó Input permite ler essa altura, somar novas formas, nivelar ou
escavar mapas existentes; fonte inteiramente comum pode usar base zero e uma
única operação Surface.
Todos os controles ficam em dados nativos, consultáveis/editáveis pelo MCP.

Terrain1/2 e Planet1 permanecem legíveis e conservam os kernels/ruídos antigos.
Não reinterpretar um ruído2D antigo como ruído3D novo: isso mudaria mundos salvos.
Um documento antigo que tente usar Surface deve ser recusado, incluindo envelopes
que mintam sobre a versão interna. Scene/replay/save existentes recebem meshes
publicadas pelas transações já existentes; não mudar suas versões por este formato
de fonte independente. A fonte editável continua no arquivo de terreno/planeta.

## Programa e ferramentas

Um programa contém `version`, `nodes`, `output` e `range_meters`. Os índices dos
inputs são slots anteriores: ciclos, referências futuras ou ausentes são erros.
`range_meters` é um certificado conservador, verificado por propagação de intervalos
antes da publicação. O renderer pode ler esse intervalo sem recompilar o programa
para cada tile. O construtor Rust `Program::new` calcula o intervalo automaticamente.

- Input lê a altura anterior da fonte com intervalo declarado. O documento prova
  que sua base/amostras/operações anteriores cabem nesse intervalo antes de aceitar
  o programa; o autor não pode forjar um limite para esconder geometria.
- Constant, Add, Subtract, Multiply, Minimum e Maximum compõem campos.
- SmoothMinimum/SmoothMaximum fundem campos com transição em metros, incluindo
  sua contribuição adicional no certificado de altura.
- Mix combina dois campos por máscara clampada em0..1; Clamp limita um campo.
- Noise oferece ruído3D smooth/ridged seeded com amplitude e período em metros.
  O período é potência de2 até2³⁰, com1..12oitavas e período mínimo1m na última
  oitava, mantendo o lattice inteiro preciso nas coordenadas globais.
- Remap transforma qualquer campo por Hermite cúbica com valor/derivada por nó.
  O certificado inclui overshoot pelos controles Bezier, não apenas pelos endpoints.
- Raise produz elevação/depressão compacta e também máscaras usando amplitude1.
- Level combina uma entrada com patamar/rampa local e transição suave.
- Ridge oferece a família de crista existente com orientação/dobras/perfil.
- Curve conduz uma cadeia/vale por polilinha editável, largura métrica, altura
  assinada, encostas assimétricas e arredondamento. Curvas adicionais são ramos
  normais da fonte, combinados por máximo/mínimo/soma/máscaras.
- Channel escava uma entrada ao longo de um leito explícito descendente. Não é
  cálculo de drenagem, bacia, fluxo, erosão ou simulação/renderização de água.

Não são operadores duplicados para o planeta. O mesmo enum/compilador/sampler
avalia todos esses nós nos dois domínios. Uma receita procedural constrói esses
mesmos dados que uma pessoa pode editar; não existe um caminho procedural secreto.

## Domínios e coordenadas

Âncoras e pontos de curvas são `world_space::Position` canônicos nos dois modos.
No plano pertencem ao datum vertical da fonte; na esfera pertencem à superfície
de referência, sem incorporar a altura do relevo. O domínio mede a distância e
converte direções no referencial tangente de cada ferramenta. As células globais
são subtraídas em i128 antes de converter diferenças locais em f64.

Na esfera, curvas usam arcos menores de grandes círculos. Dividir segmentos que
atinjam um quarto da circunferência; ferramentas locais devem caber num hemisfério
para evitar a ambiguidade do antípoda. Isso é uma propriedade geométrica do domínio,
não um catálogo planetário menor. Campos globais não usam essa carta local.
No plano, segmentos usam distâncias cartesianas. Os perfis são comuns.

O ruído comum é amostrado no espaço3D: posições precisas no plano e posições
planetocêntricas na esfera, independentemente de sua localização no universo.
Não há longitude, singularidade polar ou seed por face/tile. Mesma seed não promete
um mapa global numericamente idêntico em superfícies de topologias distintas.
O eixo tangente de uma ferramenta esférica é escolhido uma vez na sua âncora;
não se troca o referencial conforme a câmera cruza faces.

## MCP, preparação e publicação

`query_terrain`/`query_planet` recebem os documentos comuns nas consultas existentes.
Em ambos, `kind: prepare_surface` recebe `draft: {nodes, output}` e devolve o mesmo
programa nativo com `range_meters` calculado pelo compilador. O usuário não precisa
adivinhar um certificado para compor ferramentas ou editar uma spline.
`query_terrain.edit` acrescenta/substitui/remove a operação e faz validação atômica.
Para o planeta, editar a fonte e reenviá-la às operações existentes de inspeção,
save e publicação; os controles novos não exigem um editor de nós visual separado.
Save/LoadTerrain e Save/LoadPlanet conservam fonte/seed/curvas/programa; publicação
usa o mesmo sampler para geometria visual e colisão quando esta for solicitada.
Undo/redo e replay da publicação seguem as transações existentes.

O programa é preparado antes das amostras: coeficientes de curvas, segmentos,
orientações, normalização do ruído e slots ativos são reutilizados. Amostragem usa
scratch fixo na pilha, sem alocação, busca por nomes ou interpretação JSON.
O executor percorre uma IR tipada com dispatch por enum e slots; não é um JIT
que gera código de máquina específico para cada receita. Esse é o tradeoff
inicial entre extensibilidade, verificabilidade e custo de dispatch. Operações
desconectadas da saída são eliminadas da ordem de execução preparada.
O World guarda o sampler; o Atlas clona seu estado preparado compartilhando os
programas por Arc. Meshes residentes e tiles do Atlas recebem esse sampler em vez
de recompilar grafos. As curvas descartam suportes remotos antes dos cálculos
trigonométricos. A busca ainda é linear, limitada por64pontos;
uma rede geográfica extensa exigirá índice espacial regional, não ampliar a lista
indefinidamente. Não há job de erosão/geração global incluído neste recorte.

## Revisão estática anterior à implementação

1. JSON é somente autoria/MCP; operação quente é enum tipado/slots u16 preparados.
2. Até128nós/programa,64pontos/curva,32knots/remap; arrays têm decoders limitados,
   inclusive CBOR sem size hint. Mantidos envelopes8MiB e limites de trabalho.
3. Sem heap/locks por amostra. Fonte/coeficientes preparados são compartilháveis;
   scratch fica na pilha e não torna o sampler dependente de ordem/threads.
4. Coordenadas precisas não se tornam f32 globais; geometria continua seguindo
   a projeção existente. Ruído tem lattice inteiro, não seed derivada de LOD.
5. Intervalos incluem soma, multiplicação, máscaras e overshoot de remapeamento;
   rejeitar limites falsos/não finitos antes de publicar. Culling não pode esconder
   uma montanha nova. Erro geométrico novo deve usar fallback conservador.
6. Preservar resultados antigos é um gate independente; não reduzir densidade,
   física ou grade para apresentar ganho. Nenhuma promessa de FPS nesta mudança.
7. A autoria é limitada por programa, não pela área total do mundo. Hidrologia,
   erosão regional, importadores e densidade volumétrica continuam etapas próprias.

Tradeoff de LOD: o programa não declara uma derivada espacial que ainda não foi
provada para toda composição. O planetário usa o intervalo certificado completo
como erro conservador do campo entre amostras. Isso pode exigir mais detalhe que
os bounds analíticos dos kernels legados. É uma limitação de eficiência explícita,
não licença para ignorar um pico entre vértices; envelopes locais/derivadas
certificadas por operador podem estreitar esse bound posteriormente. Nenhuma
melhoria universal de FPS é inferida do cache ou da nova expressividade.

## Proveniência

O vocabulário foi orientado pelo estudo de Tectonic/Mapgen4. Esta primeira unidade
usa implementação matemática nativa, sem copiar fontes/receitas de terceiros ou
adicionar dependências. Não declarar importação de mods, equivalência de ruído ou
suporte a cavernas de um grafo de densidade3D. A adoção futura de arquivos externos
precisa das revisões/licenças/avisos descritos no estudo.

## Validação desta unidade

32 testes Rust distintos aprovados:9 de Surface1,3 Terrain,3 Region,3 Planet e
14 planet_orbit, incluindo os2 testes GPU executados explicitamente com
`--include-ignored`. Mais9 Surface repetidos após os ajustes de Clippy. Cobrem
precisão submétrica em esfera de raio5e11m, campos globais acima de2^53células,
polos/faces, curvas/perfis/escavação/patamares, input de mapa fixo/planeta anterior,
certificados falsos/overshoot, versões/codecs antigos e rejeição atômica. O jogador
ficou grounded na colisão da fonte nova e a ativação da visão orbital preservou
sua pose. Os13 contratos anteriores do Horizon continuam passando.

Clippy release dos alvos lib/bin e dos quatro testes de integração selecionados
passou com `-D warnings`; build release de ignis/Surface, fmt, Python compile e
diff check passaram. Não é declaração de suíte inteira nem validação das frentes
paralelas. Pasta temporária de compilação isolou a fila compartilhada, reutilizando
bibliotecas CoW sem copiar executáveis de apps; remoção registrada na evidência.

MCP real:4grupos passaram, incluindo preparação automática, assimetria/largura,
input/limites, fontes binárias, publicação/collision/undo/redo/replay e as posições
exatamente iguais das4bordas entre tiles esféricos. `artifacts/surface-program/`
contém fontes, cenas,2PNGs, `verification.json`, `validation.json` e `review.html`.
O binário das capturas é79233fc6be72e0dd; o executável corrente compartilhado
f98e580c20fd29a4 também passou por probe MCP de preparação/input/nivelamento,
sem sobrescrever a entrega de outra frente. Nenhuma janela foi aberta/ativada.

A fixture esférica estática usa4assets e evidencia linhas de iluminação entre
peças, apesar das posições compartilhadas exatas; isso não é aceite de normais
estáticas ou artístico. O runtime usa as normais contínuas existentes do sampler.
A cena humana reprovada permaneceu preservada. Estabilidade de FPS, hidrologia,
erosão, importadores de pacotes e composição volumétrica continuam separados.
