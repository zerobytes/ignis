# Árvore élfica ancestral — análise e lista de construção

4 de novembro de 2026 · LOS-21 · Análise anterior à geometria.

Referência aprovada pelo usuário: `games/land-of-survivors/references/ancient-white-elven-tree.png`, 1536 × 1024, paisagem, SHA256 `f3bd81b0b93ff8d0901e0a9707accaa789792e81cf8b15ac4e096fbf33e14bdf`. A imagem é um conceito visual, não uma medição botânica ou um modelo de múltiplas vistas. A exigência métrica vem do usuário: tronco de aproximadamente 5 m de diâmetro. Não foi solicitada uma árvore Ent com rosto.

## Leitura global e hipótese espacial

Uma única árvore domina quase toda a imagem. Tronco muito grosso e irregular; grandes braços oblíquos/horizontais abrem uma copa ampla, branca, lobulada e assimétrica. A câmera está baixa, olhando ligeiramente para cima. As raízes expostas abrem em várias direções, com bifurcações e vãos entre elas. O tronco tem costelas sinuosas, recessos escuros e áreas de madeira exposta; a casca é cinza/bege/marrom, com manchas verdes. Folhas brancas são formas individuais pontiagudas, sobrepostas em conjuntos; não são neve, pétalas ou uma superfície branca contínua. Alguns extremos da copa tocam ou ultrapassam o quadro.

Coordenadas locais propostas: Y para cima, base em (0,0,0), frente aproximada vista da imagem voltada a +Z. Profundidade, costas e radiação completa de raízes são hipóteses. Diâmetro nominal do eixo principal 5 m, copa pretendida de cerca de 40–45 m de largura e altura de cerca de 27–30 m são escolhas de autoria; somente a primeira dimensão corresponde ao pedido. A forma será tridimensional, sem reduzir os galhos a curvas em um plano de câmera.

Contagens: pelo menos sete grandes trajetórias lenhosas são distinguíveis na projeção, mas sobreposição não permite contar todas as suas conexões. Pelo menos nove trechos de raiz grossa aparecem na frente/laterais; não equivalem ao total de raízes. Folhas e ramos finos formam famílias distribuídas, sem contagem exata recuperável. A proposta construtiva usará bifurcações sucessivas e oito grupos principais de copa, não uma contagem atribuída à referência.

## Recortes e revisitação

Origem de pixels no canto superior esquerdo, X à direita/Y abaixo. Recortes sem resize, abertos e inspecionados, em `games/land-of-survivors/artifacts/elven-tree/reference/`; manifesto `crops.json`. As caixas abaixo são (x,y,largura,altura) na referência original.

| Ficha | Caixa | Leitura aprofundada e efeito na lista |
| --- | --- | --- |
| F1 primary-forks | 307,225,983,502 | O braço ascendente da direita é mais grosso e atravessa a leitura do centro; braços inferiores tendem à horizontal. Junções largas, não tubos apenas encostados. Confirma T03–T06. A hierarquia interna permanece hipótese onde há oclusão. |
| F2 trunk-roots | 261,399,1029,553 | A base não é cilíndrica: costelas abrem para raízes, com depressões e vãos. Os recessos escuros do tronco não provam um túnel atravessando a árvore. Divide T02/T07/T08/T09. |
| F3 bark-detail, filho de F2 | 614,553,246,246 | Sulcos longitudinais se curvam com os volumes; bordas são gastas e desiguais, interior é mais marrom. Relevo grande e casca fina são tarefas separadas, T07/T14/T15. Não há pixels para identificar espécie da casca. |
| F4 root-detail, filho de F2 | 476,707,553,235 | Raízes arredondadas se bifurcam, entram/saem do chão e deixam espaços negativos; há pequenos prolongamentos e manchas verdes. Acrescenta T10/T16; não simular vãos por pintura preta. |
| F5 left-crown | 0,20,722,635 | Várias camadas escalonadas, ramos secundários expostos sob a folhagem e pontas mais leves. Copa não é uma esfera nem um feixe só de folhas nas extremidades. Confirma T05/T11/T12. |
| F6 right-crown | 799,10,737,656 | Lobos altos acima do braço diagonal e uma faixa baixa pendente; distribuição assimétrica em relação à esquerda. Confirma T04/T11/T12. |
| F7 leaf-detail, filho de F6 | 1044,154,262,215 | Lâminas alongadas/pontiagudas, pequenos pecíolos, orientações variadas e branco com sombras cinza. Nervuras finas não são resolvidas com segurança nesse recorte. Confirma T12/T13/T17. |

A revisitação do conjunto confirmou que os vazios sob a copa e a diferença entre braços altos e baixos precisam sobreviver à folhagem. Musgo localizado, pequenas raízes, recessos e assimetria foram incluídos, em vez de resumidos como “detalhes depois”. A floresta, o céu, a relva, pedras e folhas caídas no chão pertencem ao contexto da imagem; não serão importados nem construídos nesta unidade, cujo objeto é a árvore.

## Lista consolidada antes da construção

Agrupadores: A corpo lenhoso; B copa; C superfícies; D integração/conferência. Todos os itens executáveis abaixo estão **não construídos**, com validação do modelo **não executada** nesta entrega inicial. Observado/requisito/hipótese indicam a origem da afirmação; nenhum dado oculto é apresentado como observado.

| ID / pai | Construir e ocorrência | Evidência / estado | Relações e hipótese | Critério de conferência |
| --- | --- | --- | --- | --- |
| T01/A | Eixo principal muito grosso, curvo e levemente inclinado, afilando sem ficar fino no colo. | F1/F2; observado + requisito de ~5m. | Base sustenta toda a árvore; seção não circular é aproximação. | Medida nominal da fonte, silhueta grossa e volume lateral/verso. |
| T02/A | Base alargada contínua, com costelas que se abrem para o chão. | F2/F4; observado. | Liga T01 a T09; superfície contínua, sem tubo sobreposto escondido. | Vistas de base, junções fechadas e transição sem aba reta. |
| T03/A | Bifurcações principais robustas e assimétricas. | F1; pelo menos7trajetórias projetadas, total desconhecido. | Grafo de bifurcações como hipótese3D; pai termina nos encontros nativos. | Estrutura sem folhas e laterais, sem extremidades atravessadas. |
| T04/A | Braço dominante à direita, oblíquo e sinuoso, sustentando lobos mais altos. | F1/F6; observado. | Uma família ascendente e suas subdivisões. | Relação de espessura/direção com tronco e copa no enquadramento de referência. |
| T05/A | Braços baixos laterais longos, arqueados e mais horizontais, com pontas ascendentes/pendentes variadas. | F1/F5/F6; observado. | Carga da copa; não são cópias rotacionadas idênticas. | Vãos sob copa e diversidade de alturas nas vistas frontal e lateral. |
| T06/A | Ramificação secundária e terciária, afilamento e inserções volumétricas. | F1/F5/F6; famílias, total oculto. | Continua o grafo principal e sustenta brotos folhosos. | Galhos visíveis entre massas de folhas; não haver leques coplanares. |
| T07/A | Costelas e vales maiores sinuosos no tronco/colo. | F2/F3; observado. | Relevo geométrico nativo; distingue-se da casca fina. | Luz neutra e silhueta de perto, não depender apenas da cor. |
| T08/A | Recessos entre costelas e junções, sem inventar um túnel interno. | F2/F3; observado, profundidade desconhecida. | Hipótese de cavidades abertas/vales; CSG/anastomose não disponíveis. | Declarar aproximação se o gerador não reproduzir a profundidade visível. |
| T09/A | Raízes principais grossas expostas, curvas e radialmente distribuídas. | F2/F4; observado + costas hipotéticas. | Sistema nativo basal; raízes emergem do colo. | Frente/laterais/verso, chegada ao solo e espaço entre raízes. |
| T10/A | Bifurcações de raízes menores e pontas que entram no chão. | F4; família observada. | Filhos reais de T09; profundidade traseira hipotética. | Vãos reais e redução de seção; nada apoiado por pintura preta. |
| T11/B | Copa larga com lobos de alturas diferentes, camadas e assimetria. | F5/F6 + conjunto; observado. | Oito grupos autorais propostos, conectados a T03–T06. | Silhueta global, largura/altura e ausência de esfera/coluna única. |
| T12/B | Conjuntos frondosos distribuídos também no interior, com lacunas pequenas entre massas. | F5/F6; observado. | Ramos/brotos reais, famílias repetidas com variação seeded. | Ocupação da copa sem esconder inteiramente os braços principais. |
| T13/B | Folhas individuais alongadas e pontiagudas, dobra leve, pecíolos e direções variadas. | F7/F5; observado, quantidade desconhecida. | Lâminas ligadas aos ramos, sem cartões de copa/foto. | Recorte nativo da folha e folhas na borda da copa; espessura/geometria. |
| T14/C | Casca cinza/bege antiga, rugosa, com fissuras alongadas em escalas diferentes. | F3; observado. | Receita de material sobre T01–T10; domínio2D disponível é uma aproximação. | Detalhe de tronco iluminado e canais de material não uniformes. |
| T15/C | Contraste de madeira marrom nas depressões e bordas gastas claras. | F2/F3; observado. | Máscaras/altura correlacionadas; não recortar a referência como textura. | Material nativo de perto, com rugosidade/relevo distintos. |
| T16/C | Manchas verdes localizadas, pequenas e desiguais, de aparência musgosa. | F2/F4; observado, espécie desconhecida. | Máscaras procedurais; distribuição por posição/inclinação ainda limitada. | Verde secundário à casca, sem pintar tronco/copa inteira de verde. |
| T17/C | Folhas brancas/ivório com variação discreta e transmissão fina, sem emissão. | F5–F7 + requisito do usuário. | Material próprio reutilizável; sombra mantém volumes legíveis. | Copa branca em geral e detalhe, sem parecer neve ou flores. |
| T18/D | Fonte botânica, fonte dos materiais e vínculo de publicação editáveis. | Requisito atual. | Aplicar geradores nativos; preservar ajustes/materiais na regeneração. | Save/load, reprodução e prévia coerente com aplicar. |
| T19/D | Árvore colocada no palco200x200m, com câmera de apresentação e luz que permitam avaliar a forma. | Contexto da tarefa. | Preservar piso, planeta e habilidades; sem cenário florestal adicional. | Abertura do projeto, enquadramento completo e plano do palco mantido. |
| T20/D | Conferir estrutura sem folhas, frente/verso/laterais e detalhes de casca/folha/raízes. | Contrato de qualidade. | Cámeras e condições registradas; traseira é hipótese, não fidelidade observada. | Capturas reais, diagnóstico por grupo e discrepâncias explícitas. |

## Ferramentas, lacunas e decisão de integração

Build consultado pelo MCP: `4718d286feb268c4`, Source/Recipe botânicos7, materiais Recipe1. Fonte de plantas guarda curvas, seções, conexões, raízes e edições; Rust compila os triângulos. Serão usados eixos/forks/raízes/lâminas do gerador existente, com receita própria, não recoloração de uma fonte anterior. Texturas serão receitas nativas com canais de cor, rugosidade, altura e transmissão, exportadas em `.ignis-material`.

Lacunas reais: o gerador não oferece anastomose/CSG universal para um tronco oco; o material Recipe1 usa projeção planar, sem UV por órgão ou máscaras automáticas por inclinação. Essas limitações não eliminam T08/T14/T16: a aproximação precisa ser conferida e registrada.

Outra lacuna encontrada no serviço: aplicar um material procedural à árvore é classificado como alteração manual e bloqueia regeneração/preview. A opção de descartar tudo perderia ajustes; embutir novos campos na receita exigiria alterar formatos. A integração proposta preserva, no serviço existente, materiais procedurais já validados pelo gerador ao reavaliar geometria sob política Preserve, tanto na prévia quanto na publicação. Topologia/rig/domínio/material legado desconhecido continuam com as guardas anteriores; Discard permanece explícito. Materiais independentes continuam salvos e vinculados pela receita de composição do projeto. Um teste crítico deve proteger esse transporte e a recusa/cancelamento atômicos. Não é autorização para substituir silenciosamente toda edição manual de material.

Revisão estática anterior: somente autoria explícita e bake; nenhum novo trabalho por frame, I/O por tick, thread, dependência ou layoutGPU. Recusar excessos nos limites existentes de65.536órgãos/3Mvertices/6Mtri; fontes/material em arquivos atômicos com limites existentes. Materiais/fonte imutáveis compartilhados onde o sistema já o faz. Save/load, prévia/aplicar, preservação e custos reais serão revistos depois; não há promessa de FPS ou de fidelidade baseada somente em testes.
