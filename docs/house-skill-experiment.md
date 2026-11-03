# Reconstrução da casa pela skill e pelo MCP — EAF-101

## Limite do experimento

Entrada de modelagem: skill ignis-image-to-3d, seu contrato de análise, imagem
original e schemas/resultados do MCP. Não carregar a análise ou o modelo
anteriores, fontes da engine, documentação de geometria ou scripts de autoria
preexistentes. O contexto já recebido não pode ser apagado: esta é uma execução
com fontes restritas, não um teste cego independente.

Bootstrap do transporte stdio, Git, checklist de performance e Kordo são tarefas
operacionais. Toda publicação/alteração/captura da cena passa por engine_command.
O cliente pode calcular vértices a partir das formas que descreve. Nenhuma API
privada da engine, outro modelador ou imagem 3D externa participa da construção.

## Nova leitura visual, antes da geometria

**Correção posterior do usuário:** a hipótese de organização abaixo resultou
num erro de planta. O corpo deve ser tratado como retangular, com porta numa
face longa e varanda à frente dessa face. A receita publicada em189ea1f ainda
põe a porta no plano da face curta/frontão, perpendicular à face longa da varanda.
O elogio ao avanço não foi aceite dessa organização nem da fidelidade final.
Preservar a leitura inicial como histórico do experimento. O usuário confirmou
que a bay window está correta no modelo e redirecionou o objetivo: melhorar a
ferramenta e o processo de reconstrução por imagem, sem corrigir esta cena.

Referência: imagem fornecida, 1536×1024. Coordenadas abaixo em pixels, origem
superior esquerda; regiões são localizadores aproximados, não segmentação.

| Conjunto | Evidência e interpretação | Forma/relação a construir |
|---|---|---|
| Implantação | Faixa inferior, x100–1450/y730–960: base clara elevada e pavimentação irregular. | Piso elevado contínuo, base da varanda e base saliente da janela; três degraus na entrada. Escala real desconhecida. |
| Organização | x120–1450/y400–865: varanda aberta longa à esquerda; entrada recuada no meio; volume envidraçado saliente à direita. | Hipótese: varanda na lateral de um corpo com frontão, entrada na face frontal e bay facetada na mesma face. Preservar profundidade entre arco de entrada e porta. |
| Apoios | x120–690/y440–860: suportes verticais com plintos e coroamentos. | Quatro linhas aparentes delimitam três vãos da face longa; extremos podem ser pilares, não colunas livres. Nomear cada um. |
| Vãos altos | x150–645/y435–690: topo apontado, curvas, espessura e ramificações. | Vão realmente aberto entre os apoios; moldura curva e rede interna separadas. |
| Muretas | x120–675/y685–820: travessa superior, rodapé e arcos vazados repetidos. | Repetição menor do motivo; altura alinhada entre apoios, vazios preservados. |
| Entrada | x675–990/y420–890: arco externo grande, porta menor atrás, escada à frente. | Hierarquia arco → espaço coberto → porta recuada; não coplanar. |
| Bay | x1010–1445/y465–850: três painéis principais distinguíveis; lateral estreita; faces mudam de orientação. | Planta poligonal arredondada, base maciça e montantes claros. Faces ocultas hipotéticas; não assumir total a partir do visível. |
| Cobertura principal | x80–1465/y80–455: cumeeira alta à direita, grande pano em curva suave sobre a varanda. | Seção assimétrica curva, cumeeira longitudinal, espessura/beiral contínuos; hipótese de seção a conferir no render. |
| Cobertura da bay | x995–1475/y270–475: telhado inferior saliente, fileiras de telhas, encontro encoberto pelo frontão. | Setor aproximadamente cônico/facetado, apex parcialmente enterrado na cobertura principal; encontro precisa de inspeção. |
| Chaminé | x530–650/y135–285: seção quadrangular, topo saliente e painel decorativo. | Peça que atravessa pano do telhado; coroamento e motivo no painel. |
| Talha | Vigas/apoios/montantes: ressaltos, sulcos, molduras e bases salientes. | Perfis por níveis, eixos/tamanhos compartilhados; ordem arquitetônica específica desconhecida. |
| Motivos | Porta x750–880/y470–765; janela x1200–1360/y490–725; varanda e muretas. | Eixo ascendente, dois pares principais de ramos curvos espelhados, sub-ramos e encontros na moldura. Número de pontas pequenas varia/não está resolvido em todo suporte. |
| Aparência | Pedra clara manchada, coberturas escuras em fileiras, ornamentos quentes, campos verdes no frontão. | Hipóteses de pedra envelhecida, telhas tipo ardósia, madeira/metal. Forma será avaliada antes da aparência final. |
| Vegetação | Bases, apoios e entorno da bay: folhas, flores claras/roxas e trepadeiras. | Conjuntos separados, oclusão localizada; manchas rasas distintas de folhas com volume. |
| Acessórios | x860–985/y480–795: vaso florido e luminária em suporte curvo ao lado da porta. | Suporte, corpo da luminária e vaso separados; não inventar acessórios simétricos ocultos. |

Grafo do motivo a testar: tronco central contínuo; primeiro par de ramos mais
abertos, segundo par mais alto; cada ramo recebe terminações menores; conexões
à moldura dependem do vão. Reusar a regra em coordenadas locais com variação
de proporção. Simetria local não significa simetria na imagem em perspectiva.

Âncoras aproximadas para conferência visual: base do canto da varanda (667,854),
base do apoio distante (130,811), porta/piso (804,772), pé da escada (811,885),
cumeeira frontal (1187,85), cumeeira distante (618,213), beiral perto da entrada
(660,410), beiral distante (90,419), beiral direito (1467,454). Sem precisão
fotogramétrica; correspondência precisa ser estabelecida antes de medir erro.

## Plano e capacidades descobertas

Build MCP 9860a09115365ad3, cena67. inspect_meshes informa 16 assets, 32 instâncias,
8192v/16384t por asset, 16384v/32768t armazenados e 131072t desenhados. Assets,
instâncias, plantas, materiais, câmera, luz/sombra, capture, comparação e replay
estão no schema. Não foram descobertos solver de câmera, segmentação ou CSG.

1. Volumes/fontes: base e escada, corpo recuado, bay facetada, piso da varanda.
2. Curvas: coberturas, apoios com perfis e vãos altos/baixos realmente abertos.
3. Conferir câmera/silhueta/oclusões; corrigir causa geométrica antes de detalhar.
4. Motivo reutilizável, molduras, fileiras de telhas e acessórios.
5. Aparência/vegetação; revisão por peça e capturas auxiliares.

Receita cliente com parâmetros e tabelas de correspondência, meshes publicados
pelo MCP. Priorizar plantas quando o schema representar a peça; usar geometria
direta para contornos e coberturas fora da família paramétrica disponível.

## Checklist estático anterior

1.Autoria eventual; sem tick novo. 2–3.Vetores limitados pelos budgets expostos,
índices compartilhados nas superfícies. 4.Reusar funções/perfis/motivos no cliente.
5.Contar vértices/triângulos antes de publicar; não elevar limites. 6.Captura só
por etapa, arquivos correntes estáveis. 7.Transporte sequencial; nenhuma thread
ou lock no produto. 8.Revisão esperada e save/load/undo da interface. 9.Verificar
validação MCP, persistência/replay e pixels repetidos. 10.Custos mantidos: autoria
CPU e malhas visuais sem colisão, câmera ajustada manualmente, hipótese de verso.

## Resultado executado

Receita [reconstruct_house_skill.mjs](../scripts/reconstruct_house_skill.mjs),
cena autoral [house-skill.scene.json](../examples/house-skill.scene.json).
O cliente usa somente a biblioteca padrão do Node e engine_command por MCP.
Coordenadas, perfis, amostragem e regras dos motivos são autoria do cliente;
não são novos kernels da engine. Coluna: planta nativa por seções, quatro
instâncias; demais curvas/superfícies: malhas diretas. Motivos de varanda,
mureta e bay compartilham três assets canônicos com instâncias próprias.

### Aprendizado espacial e continuidade do processo

Requisito confirmado pelo usuário: corpo retangular, porta numa das faces longas,
varanda diante dessa parede e bay como projeção anexa. A enumeração de objetos
da análise foi insuficiente: faltou identificar que parede hospeda a porta e
qual direção ela enfrenta.

Na receita existente, mapFront fixa Z para porta/arco de entrada; mapSide fixa X
para a varanda. Consequentemente, porta e fachada longa da varanda estão em
planos perpendiculares. Não é uma diferença solucionável girando a câmera.

A skill passa a exigir face hospedeira e orientação para aberturas/anexos,
incluindo relações entre espaço coberto, acesso externo e porta. O erro serve
para avaliar o procedimento reutilizável; não autoriza reconstruir esta casa.

Incremento aprovado pelo usuário: análise global seguida de fichas por
característica, com recortes realmente abertos, segunda leitura mais profunda
e sub-recortes motivados por dúvidas. Os achados voltam ao inventário e ao plano.
Durante uma construção autorizada, cada ficha orienta comparação local por
métricas e interpretação visual de referência/render, seguida de revisão global.
Procedimento em
[detail-review.md](../.agents/skills/ignis-image-to-3d/references/detail-review.md).

Este incremento entrega instruções e contrato documental. Nenhuma receita,
geometria, câmera, PNG do ensaio, schema ou operação MCP foi alterado; não foram
produzidas fichas/recortes desta casa nem executada uma nova reconstrução.
Recorte precisa de ferramenta de imagem disponível no ambiente consumidor;
uma ROI do comparador não equivale a um recorte inspecionado. Qualidade da
nova análise e efeito sobre fidelidade exigem próximo ensaio de uso.

A primeira câmera estava alta, e a primeira receita de degraus sobrepunha suas
frentes. Capturas levaram à correção dos degraus, posição da entrada, comprimento
da varanda, câmera, curvatura longitudinal e posição da chaminé. O perfil da
cobertura varia ao longo da profundidade; alturas dos apoios acompanham essa
hipótese. Isso é ajuste manual da hipótese, não recuperação certificada da forma.
Telhas têm faces planas, sobreposição e borda inferior com espessura; aparência
e conexões continuam simplificadas. O padrão de pedra foi separado dos relevos.

Primeira expansão ingênua dos detalhes: 24577 vértices diretos, excedendo os
16384 globais e também o limite de um asset de ornamentos. Instâncias dos motivos,
amostragem da coluna por trechos lineares e tesselação reduzida permitiram a cena
final: **16 assets / 32 instâncias / 16090 vértices / 21818 triângulos armazenados**,
32272 triângulos desenhados. Restam 294 vértices globais; contagem de assets e
instâncias está saturada. Limites do build não foram modificados.

Capturas/relatório/página no destino corrente `artifacts/house-skill/`. A página
oferece lado a lado, sobreposição sem alinhamento, forma neutra, vista elevada e
diferença RGB. Métricas nativas finais, seleção global sem máscara:

| Região fixa | MAE RGB | SSIM luminância |
|---|---:|---:|
| total, inclui fundo | 0.166730 | 0.399808 |
| cobertura | 0.164237 | 0.421990 |
| varanda | 0.238878 | 0.056368 |
| entrada | 0.195687 | 0.072197 |
| bay | 0.205196 | 0.100926 |

**A reconstrução ainda está distante da referência.** Valores locais baixos,
especialmente na varanda/entrada, não são anulados pelo total. Câmera manual,
materiais/luz e geometria contribuem juntos; não há percentual de fidelidade.

## Revisão por parte

| Parte | Presente no modelo | Divergência restante |
|---|---|---|
| implantação/acesso | base, pavimento e três degraus com alturas distintas | espessuras, limites e distribuição das lajes ainda genéricos |
| varanda | apoios com perfil, três vãos abertos e mureta vazada | perfis, proporções, profundidade e encontros com a cobertura diferem |
| entrada | arco externo, porta recuada e vaso/luminária | molduras simplificadas; suporte/luminária pequenos e sem acabamento |
| bay | quatro faces envidraçadas e cobertura própria | projeção, montantes, seção da cobertura e encaixe ainda aproximados |
| cobertura principal | assimétrica, curva e com telhas individuais | topologia/contorno e talha do frontão não reproduzem os detalhes visíveis |
| motivos | eixo central, dois pares, sub-ramos e repetição | espessuras, junções/terminações e variações por suporte ainda insuficientes |
| aparência/plantas | pedra com variação/juntas, folhas e flores separadas | pedra sem riqueza da referência, plantas esparsas e facetadas, luz simples |

Nenhuma fachada oculta, interior, colisão da casa ou fechamento manifold foi
comprovado. Vista elevada confirma uma hipótese volumétrica, não esses contratos.

## Verificação realizada

Dois processos MCP novos reproduziram cena JSON e PNG exatamente. Cada execução
verificou captura repetida, undo de materiais/cores neutros, save/load e replay
**estático de zero ticks**, com igualdade exata de pixels. Replay matched:true
inclui meshes e geradores. Erro ao tentar o 17º asset recusou sem mudar cena ou
revisão. Todos os IDs da tabela de correspondência resolvem na cena publicada.
`node --check` e `git diff --check` aprovados. Sem Rust ou renderer alterados;
não se repetiram suites Rust/MCP gerais nem se produziu build novo.

Reproduzir a partir da raiz do projeto (paths absolutos do ambiente de teste):

```sh
/Users/zero/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin/node --input-type=module -e 'const m=await import("./scripts/reconstruct_house_skill.mjs"); await m.runStudy({binary:"/Volumes/Zero/dev/games/.ignis-build/image-to-3d/release/ignis",root:"/Users/zero/.codex/worktrees/image-to-3d/custom-engine/artifacts/house-skill",reference:"/Users/zero/Downloads/Imagem do ChatGPT 23 de out. de 2026, 20_54_15.png"});'
```

O script abre e encerra seu próprio processo sem janela. Outputs substituem os
caminhos correntes. A cena versionada é cópia da autoria salva pelo MCP, não uma
cena prévia carregada para iniciar a reconstrução. O ambiente de teste e o build
foram mantidos; os caminhos acima não são configuração portátil do produto.

## O que o experimento ensina sobre skill e interface

1. O guia produz inventário e sequência melhores, mas critérios escritos não
   verificam a interpretação. Ainda faltam ferramentas de correspondências,
   contornos/máscaras e diagnóstico por peça, com evidência mensurável.
2. O MCP publica e inspeciona malhas, mas a descrição de arco/cobertura/motivo
   ainda exigiu bastante matemática no cliente. Uma biblioteca de construções
   parametrizadas reduziria a dependência da capacidade de cada IA.
3. Budgets globais obrigaram agrupar subpartes por material e compartilhar a
   malha de vidro com painéis do frontão. A correspondência registra essa perda
   de granularidade; não se deve confundir asset com peça semântica.
4. A skill contém links/exemplos específicos desta casa. Eles não foram abertos
   nesta execução, mas o contexto anterior continua conhecido; uma avaliação
   independente precisa de outra sessão e de imagem que não seja o exemplo.
5. A entrega é uma primeira cena técnica revisável. Fidelidade e aceite artístico
   permanecem abertos; funcionamento do pipeline não os substitui.

## Checklist após a execução

1.Autoria fora do tick. 2–4.Fontes/motivos compartilhados, coluna160v em vez de
2896v após remover amostragem redundante de segmentos lineares; nenhum ganho de
FPS alegado. 5.Limites expostos preservados e rejeição atômica comprovada.
6.Capturas offscreen só sob comando, caminhos estáveis e sem novo build.
7.Cliente serial, processo próprio encerrado; sem janela/demo de outra sessão.
8.Undo, reload e replay conferem invalidação. 9.Resultados acima e report.json.
10.Custos mantidos: tesselação/agrupamento limitados, matemática no cliente,
comparação fotométrica sem calibração/validação semântica automática.
