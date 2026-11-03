# Raízes em paredes e cristas — revisão anterior à construção EAF-198

O usuário reprovou a aparência de5e37102: o sistema ainda trata o corpo das
raízes como tubos. Requisito direto: suportar variedades de raízes, inclusive
“murinhos com cilindros no topo”. Duas novas fotos fornecidas nesta rodada
orientam a forma; não identificam espécie, escala métrica ou idade.

## Conjunto e recortes inspecionados

Fontes: `codex-clipboard-943a1bb5-9ed8-411e-b913-0fc537d273ba.png`,628×630,
e `codex-clipboard-cc3d4338-b509-44cc-8ac2-bde5106a08e9.png`,1300×866.
Orientação original, pixels sem resize/rotação; manifestos/crops em
`artifacts/botany-buttress`. Coordenadas `(x,y,w,h)` da origem superior esquerda.
Imagem inteira e os três recortes abaixo foram abertos antes de gerar geometria.

| Ficha | Fonte/caixa | Segunda leitura e consequência |
| --- | --- | --- |
| B01 | foto1,45,130,530,400 | Uma raiz cruza o centro como parede larga na projeção, com altura muito maior que a espessura da borda. Crista arredondada e ondulante; descida do topo próxima do tronco. Na frente há curva em U, solo/grama no lado interno. Não é apenas elipse alongada nem tubo horizontal. Seção com parede e crista controladas separadamente; meandro da trajetória. |
| B02 | foto2,370,160,800,640 | Parede central começa alta no tronco, curva à frente/direita e baixa gradualmente. Faces laterais têm sulcos e manchas, crista mais clara/convexa. Vales mostram folhas/solo entre raízes; sobreposições ocultam trechos. Altura deve decair com distância e a união com o tronco preservar os vales. |
| B03 | foto2,40,240,470,370 | Família lateral esquerda tem cristas principais e secundárias, com alturas distintas e encontros que parecem bifurcações. Profundidade/ligação exata parcialmente oculta; não inferir fusão universal nem rede fechada a partir da projeção. Reutilizar grafo hierárquico com junções contínuas e variedade seeded. |

Revisitação: ambas as fotos mostram a passagem tronco→crista→parede→solo.
A árvore secundária à direita da foto2 repete o motivo, com outra distribuição.
Os espaços entre raízes são vales abertos por cima, não orifícios perfurados na
parede. A marca d’água e suas linhas na foto2 são sobreposição da imagem, não
geometria/rachaduras da árvore. Não serão recriadas. Casca, manchas, pequenos
sulcos e grama afetam aparência, mas não justificam substituir volume por textura.

Contexto excluído da construção: edifício/varanda, portas, faixas de parede,
objetos verticais e suportes ao fundo da foto1; solo, grama, folhas caídas,
árvores/copa de fundo, tronco secundário, cerca e objeto claro na junção da
foto2. Sua identidade exata não é necessária ao sistema de raízes. Partes
subterrâneas, verso e contatos ocluídos permanecem hipóteses. Não reconstruir
a cena inteira nem copiar o contorno de uma foto como malha.

## Lista consolidada anterior à geometria

Todos os itens executáveis: **não construído**. Verificação do modelo novo:
**não executada**. Esta análise registra a falha anterior e não aprova o futuro.

| ID/pai | Construir / ocorrências e evidência | Relações e incerteza | Conferência |
| --- | --- | --- | --- |
| BW01/planta | Seção de parede alta e fina, família central/laterais B01–03 | Atravessa o solo; seção oculta inferida, não se mede espessura real | Frente e lateral mostram parede com corpo, não cilindro achatado ou plano sem espessura |
| BW02/BW01 | Crista arredondada, família B01/B02 | Pertence à mesma pele da parede, não cilindro sobreposto | Topo convexo e espessura/raio configuráveis independentemente da altura |
| BW03/tronco | Contrafortes sobem pelo colo e descem sobre raízes, B02 | Distribuição varia; verso inferido | Junções fechadas/contínuas em mais de uma vista; sem colar/disco alto escondendo os vales |
| BW04/BW01 | Altura decrescente e variável ao longo de raízes | Região próxima do tronco mais alta, terminação/solo ocluídos | Perfil de altura suave e independente do raio tubular, compatível entre pai/filhos |
| BW05/BW01 | Cristas sinuosas, curva frontal e lateral B01/B02 | Não há simetria radial perfeita | Curva3D seeded suave, sem quinas/cusps; não reproduzir exatamente U fechado por hipótese |
| BW06/BW01 | Ramificação de paredes/cristas, família lateral B03 | Encontros exatos parcialmente ocultos | Filhos pertencem ao mesmo sólido, menores/variáveis, poda recompila pele fechada |
| BW07/planta | Vales/vãos com solo entre paredes B01–03 | Não preencher com uma saia circular; profundidade desconhecida | Vistas superior/lateral e sem chão mostram intervalos e volumes distintos |
| BW08/BW01 | Casca contínua, pequenos sulcos e manchas B01/B02 | Não usar foto na geometria/material; detalhes finos não são topologia medida | Material procedural existente aplicado depois do corpo; sulcos que alterem forma dependem da capacidade real |
| BW09/sistema | Família tubular anterior, pivotante/fibrosa e nova família em paredes | Usuário pede variedades; não substituir toda raiz por parede | Perfil opcional, defaults legados exatos, controles MCP/UI comuns e sem perda de escultura |
| BW10/planta | Copa/folhas da figueira preservadas, requisito prévio | Colo pode alterar sua elevação; não redesenhar ramificação | Dados anteriores dos órgãos idênticos; comparação no conjunto com folhas |

## Gramática proposta e revisão estática

Estender a seção de eixo de raiz com altura de parede, largura do corpo, largura
da crista e alcance/queda da altura. O modo tubular fica sem o perfil novo.
Trajetórias recebem meandro seeded opcional e suave, com origem comum nos forks.
Colares usam a mesma seção que os tubos; junções precisam transportar a mudança
até o interior da superfície, sem paredes/cilindros soltos ou sobrepostos.
O domínio de escultura deve distinguir a nova amostragem da seção.

Lacuna existente: kernel aceita colares circulares. Nesta etapa, avaliar uma
deformação limitada da pele de junção, com fronteiras fixadas nas seções novas,
mantendo a conectividade fechada e recusando colapso/inversão local. A transição
do colo será validada visualmente; não presumir que topologia resolve a saia.
Não prometer boolean/união universal, anastomose, calibração ou solo constitutivo.

Preparação por edição, buffers limitados por colo e amostras de seção; nenhum
gerador no frame estável. Preservar curvas, fonte versionada, quotas combinadas,
transações/undo e política de escultura. O ensaio mantém lattice por eixo;
área/massa/proxy devem refletir a seção ou declarar explicitamente a aproximação,
sem converter a parede em promessa de tecido físico exato. Validação crítica:
pele orientada/fechada, poda, variantes/idade, repouso/massa, fonte antiga exata,
atomicidade/escultura e persistência. Corpo cinza em várias vistas precede
acabamento. Resultados posteriores ficam em `docs/botany-buttress.md`.
