# Raízes basais: análise e lista anterior à construção — EAF-198

Pedido: substituir a aba basal da figueira RS por raízes muito grossas e
visíveis, e criar o sistema procedural que falta. Fontes recebidas: imagem do
artefato412×356, referência1600×1200 e detalhe442×334. São imagens fornecidas,
sem identificação científica/escala confirmadas. O detalhe e a foto maior
mostram o mesmo motivo; não presumir que são a mesma árvore das fotos anteriores.
Fonte/hash/resolução/crops ficam em `artifacts/botany-roots/reference-manifest.json`.

A árvore da referência tem tronco/colo robusto e lobulado, cujas saliências se
prolongam em raízes grossas sobre o terreno. Há vales escuros entre os volumes.
As raízes diminuem de seção e desaparecem sob o solo/folhas; bancos ocluem parte
da frente/direita. Não se vê o sistema subterrâneo, nem sua profundidade, número
total de raízes ou suas uniões internas. Tronco e raízes parecem uma massa
contínua, sem a placa plana e inclinada presente no render rejeitado.

Recortes extraídos sem resize/rotação e abertos, após a imagem inteira:

| Ficha | Fonte/caixa `(x,y,w,h)` | Leitura local e consequência |
| --- | --- | --- |
| R01 | foto maior820,540,760,580 | O colo se abre em volumes grossos; um prolongamento vai à esquerda e outros à frente/direita. As partes atrás do banco estão ocultas. Criar raízes distintas ligadas por pele contínua, preservando os intervalos de solo. |
| R02 | detalhe90,20,330,300 | Cristas verticais na base descem e seguem pelo solo, com sulcos escuros entre elas. A borda não é uma circunferência alargada única. Criar colo com vários portos/raízes e vales; proporções em unidades autorais. |
| R03 | imagem do artefato inteira | A aba termina numa borda quase plana e elevada do lado esquerdo. É falha do alargamento circular inclinado. Desativar esse flare na presença da nova base, sem alterar os defaults legados. |

Revisitando o conjunto: a copa/galhos/epífitas são contexto já tratado e devem
ser preservados. O chão desce para a direita; raízes não ficam todas na mesma
altura absoluta. Musgo, manchas de casca e folhas no solo não são prova de
geometria oculta. Pessoa, bancos, trilha, cerca, relevo distante e demais plantas
são contexto, fora da construção deste sistema. A pessoa não serve de régua
métrica sem calibração/perspectiva conhecidas.

## Lista consolidada antes da geometria

Todos os itens executáveis: **não construído**; verificação do modelo:
**não executada**. A análise não aprova uma base futura.

| ID/pai | Construir/ocorrências, evidência | Relações/hipóteses | Conferência |
| --- | --- | --- | --- |
| RT01/planta | Colo robusto contínuo, R01/R02 | Une tronco e várias raízes, verso inferido | Pele única fechada/orientada e vistas distribuídas; sem aba inclinada |
| RT02/RT01 | Raiz lateral esquerda grossa e afilada, R01 | Sai da base, acompanha o terreno e some sob a superfície | Volume alongado elevado visível, transição suave e ponta enterrada |
| RT03/RT01 | Famílias de raízes frontais/direitas, R01/R02 | Pelo menos3prolongamentos aparentes no conjunto, número total desconhecido; banco oculta trechos | Variação de direção/comprimento/seção; não copiar a mesma raiz em um círculo perfeito |
| RT04/RT01-03 | Cristas de contraforte e vales entre raízes, R02 | Região não cilíndrica; seção interna/fusão real desconhecidas | Cristas descem do colo e viram raízes; sulcos/vãos legíveis, sem preencher tudo com disco |
| RT05/RT02-03 | Afilamento e subdivisões finas, R01/família parcialmente oculta | Ramificação procedural proposta, não contagem extraída da foto | Filhos contínuos e mais finos, terminações entram no solo |
| RT06/RT02-05 | Trechos subterrâneos, desconhecidos | Hipótese autoral necessária; extensão/profundidade configuráveis | Vista de diagnóstico sem solo mostra continuidade e fechamento; não alegar raiz medida |
| RT07/RT01-06 | Relação com terreno inclinado, conjunto | Plano local de solo configurável nesta etapa; sem adaptação automática a terreno arbitrário | Raízes variam altura com o plano e não são apenas escondidas por clipping |
| RT08/RT01-06 | Distribuição/material/crescimento, requisito do sistema | Seed, espessura, comprimento, exposição, profundidade, ordens e idade; casca contínua | Mudanças reproduzíveis e dados autorais nativos; manchas da foto não viram textura |
| RT09/planta | Preservar copa aprovada, requisito | Extensão virtual do colo pode elevar a copa; não redesenhar seus galhos/folhas | Mesmos órgãos da copa/fontes anteriores, comparação da árvore inteira |
| RT10/ferramenta | Raízes editáveis via MCP e painel, requisito | Fonte/receita versionadas, preview/publicação/undo/poda, escultura e massa | Mesma operação nos dois caminhos; falha recusa sem perda de ajustes |

## Gramática e limites a implementar

Hoje há raízes aéreas como eixos, mas o kernel de junções aceita três colares.
Para uma coroa de raízes, generalizar com limites uma junção de vários portos:
colar do tronco acima, colares de raízes ao redor/abaixo e fechamento inferior.
Cada raiz é um eixo de curva/seção, com filhos e terminações subterrâneas.
Um colo virtual vertical substitui o flare sem inclinar seu plano inferior;
a copa mantém os órgãos e sua forma. Parâmetros de solo são autorais, não
escavação de terreno nem resposta biológica a obstáculos.

A ampliação exige conservar exatamente o caminho de três portos existente,
limitar número de raízes/ordens/refinamento, compartilhar índices nos colares e
recusar junções inviáveis. Não unir por tubos sobrepostos. Raízes poderão ter
âncoras distais explícitas no ensaio; isso não representa mecânica constitutiva
do solo, anastomose entre plantas, hidro/nutrição ou colisão de gameplay.

Revisão estática anterior: preparo por alteração autoral, nenhuma geração por
frame. Operação trabalha em candidato antes de publicar, com guardas de fonte,
órgãos e geometria existentes; IDs seeded e domínio da junção preservam ou
recusam escultura. Junção multicolare tem custo maior (envoltória/refinamento),
portanto quotas separadas e contagens reais serão registradas. Nenhum passeGPU,
rede ou nova cópia de build. Resultados/pendências posteriores ficam em
`docs/botany-roots.md`, sem reescrever estes estados iniciais.
