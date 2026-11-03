# Segunda referência de figueira: rede interna de galhos — EAF-197

Complementa, sem substituir, [a lista RS01–11](figueira-rs-analysis.md).
Fonte própria do usuário: `IMG_7671.HEIC`, convertida com `sips` para
`artifacts/botany-rs/house-reference.png`,5712×4284, orientação recebida.
A primeira foto mostra uma árvore inteira no campo; esta é outra árvore,
observada de perto por baixo da copa. O usuário as associa à mesma espécie;
a identificação científica não foi estabelecida. Não transferir uma contagem
ou câmera de uma foto para a outra. Não há escala métrica confirmada.

## Conjunto e recortes inspecionados

Vários eixos grossos sobem obliquamente a partir de uma região basal parcialmente
oculta; subdividem-se em braços laterais médios e uma rede muito densa de ramos
finos. Há galhos sem folhas ao longo de trechos e pequenas folhas concentradas
em terminações. O interior tem lenho visível, não apenas uma casca de folhas
na superfície externa. Sobreposição em projeção não demonstra fusão dos galhos.
Há vazios de céu irregulares e estruturas vegetais de trás/frente sobrepostas.
O limite superior/lateral da árvore e o chão junto à base estão fora do quadro
ou ocultos: não se medem altura, largura, número total de galhos ou raízes.

Coordenadas de inspeção: origem superior esquerda, X à direita/Y abaixo;
recortes reais sem resize/rotação, abertos após a imagem inteira:

| Ficha | `(x,y,w,h)` | Leitura local e efeito na construção |
| --- | --- | --- |
| H01/base |1550,1850,2100,1800| Diversos eixos grossos em leque, com divisões em alturas diferentes; a área inferior é ocultada por vegetação/coluna. Aumentar subdivisões estruturais; não contar as sobreposições como uma união única. |
| H02/laterais |2850,750,2200,1750| Braços quase horizontais têm ramos intermediários ascendentes e laterais, e estes voltam a se subdividir. Folhas não começam imediatamente em todos os braços. Introduzir ordens de brotação lenhosa antes das terminações foliares. |
| H03/rede fina |850,100,1900,1500| Rede espacial ramificada com trechos nus e muitas terminações curtas; não são poucos eixos com dezenas de folhas enfileiradas. Distribuir o orçamento foliar entre mais terminações. |

Revisitação do conjunto: a densidade aparece no centro e nas laterais, com
orientações diferentes e espaços de céu. Duplicar ramos coincidentes ou colocar
ramos em um plano não atende à referência. A folhagem tem tons verde/oliva e
reflexos claros, e a casca tem manchas claras/escuras. Os pendentes não são o
principal critério desta foto; preservá-los como capacidade já existente.
O detalhe disponível sustenta a hierarquia, mas não uma contagem total exata.
Não é necessário ampliar pixels para inventar as partes oclusas.

## Lista adicional consolidada anterior à construção

Todos os itens executáveis abaixo: **não construído**; verificação do modelo:
**não executada**. A revisão da análise não aprova o futuro exemplar.

| ID/pai | Construir/ocorrências e evidência | Relações e hipótese | Critério de conferência |
| --- | --- | --- | --- |
| RS12/RS03 | Mais subdivisões estruturais, família visível em H01/H02; contagem total oculta | Uma fonte volumétrica com mais destinos/forquilhas, mantendo espessuras por carga de subcopa; não traçar posições da foto | Mais eixos estruturais efetivos que a versão099f5d6, no centro e laterais da estrutura despida |
| RS13/RS04 | Ordens intermediárias lenhosas entre braços e brotos finais, H02/H03 | Cada ramo pode produzir filhos que voltam a se ramificar, por profundidade/ângulo/raio/comprimento configuráveis | Maior profundidade e quantidade de eixos reais; ramos conectados e afinando, sem tubos sobrepostos simulando junções |
| RS14/RS07 | Folhas distribuídas entre mais terminações, H02/H03 | Ramos estruturais/intermediários podem ficar nus; a folhagem usa o pai imediato, mantendo identidade e vínculos | Mais hospedeiros foliares com menos folhas por hospedeiro; volume, tamanho e cobertura revistos com folhas |
| RS15/RS06 | Rede interna e vazios, no conjunto e H02/H03 | Regras em vários azimutes, abertura variável; verso continua hipotético | Vistas de frente/lado/verso/superior e close interno; não fechar todos os vãos nem fazer um leque plano |
| RS16/RS01-04 | Variação de orientação, espessura e comprimento, H01–03 | Forquilhas assimétricas, afilamento e seed; tronco circular e fusão basal continuam limitações | Separar galhos estruturais, brotos e pendentes nas estatísticas; não inflar contagem com folhas/filamentos |
| RS17/perfis | Aplicar aumento menor a plátano/eucalipto e ornamental, requisito do usuário | Mesma gramática com parâmetros próprios; não reproduzir a arquitetura da figueira em todas as árvores | Comparativos antes/depois despidos e com folhas; palmeira conserva a arquitetura de touceira/frondes |
| RS18/apresentação | Ponto de vista interno, além da árvore inteira, requerido para conferir H01–03 | Câmera manual não calibrada; não confundir a nova foto com o enquadramento da antiga | Comparação estrutural localizada, sem alegar réplica, alinhamento ou porcentagem de fidelidade |

Contexto observado mas fora da construção da planta: muro/reboco, telhado
parcialmente ocluso, grades/portão, colunas, vegetação de primeiro plano,
terreno seco, céu e estruturas ao fundo. Esses elementos não alimentam geometria,
textura ou identificação da planta; nenhuma localização/endereço entra no asset.
Manchas de casca, epífitas e folhas de outras plantas sobrepostas não são usadas
como medição de sulco, fusão ou taxonomia da figueira.

## Gramática proposta e revisão estática anterior

Aumentar destinos estruturais e acrescentar ordens binárias aos brotos, com
comprimento reduzido por ordem e saída3D em direções divergentes. Até a última
ordem, o broto é um eixo lenhoso com filhos; só a última recebe folhas. Os raios
são repartidos pela potência das seções; a montagem usa `graft` lateral e
`fork` terminal existentes. Parâmetros explícitos: destinos, ordens dos brotos,
ângulo de forquilha, razão de comprimento, folhas por terminação e número de
brotos. O compilador não reconhecerá nomes de espécies.

Revisão de custo: mais eixos/junções custam vértices/restrições e compilação;
reduzir folhas por hospedeiro em vez de elevar automaticamente as quotas.
A expansão por ordens é exponencial: manter os perfis limitados e medir órgãos,
vértices/triângulos/fonte e física nos budgets atuais. Não preparar geometria
por frame, adicionar passeGPU, trocar locks ou multiplicar builds. O gerador
Python é um preparo autoral offline, não um novo solver de gameplay. Evitar
varreduras repetidas da lista de órgãos para buscar raios/identidade, usando
índice por ID sem duplicar objetos. Confirmar fonte legada/palmeira e validações
críticas existentes. Estados e resultados posteriores ficarão no contrato da
ramificação, sem reescrever a lista anterior como se já estivesse construída.
