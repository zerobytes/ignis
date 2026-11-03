# Figueira campestre — referência fornecida pelo usuário

## Fonte, conjunto e interpretação

Arquivo `codex-clipboard-eec551c4-fcf2-402c-94f4-0fc8944fee11.png`,1000×1000,
orientação recebida; cópia/crops/hash em `artifacts/botany-rs/reference-manifest.json`.
Coordenadas2D em pixels, origem superior esquerda, X→direita/Y→baixo.
É requisito do usuário acrescentar outro exemplar às plantas procedurais,
com a aparência de figueira que conhece no RS. Não substituir a ornamental.
Espécie exata, dimensões, idade, profundidade e verso são desconhecidos.
O perfil autoral poderá usar100anos; isso não data a árvore fotografada.

A foto mostra uma árvore isolada, vista aproximadamente à altura de uma pessoa,
com tronco escuro curto, inclinado para a direita, base larga e galhos que se
espalham lateralmente sob massas verdes em patamares. A copa ultrapassa ambas
as bordas laterais da imagem; sua largura completa não é mensurável aqui.
Os espaços de céu entre os patamares fazem parte da arquitetura, não são
buracos a preencher com mais folhas. A luz destaca superfícies superiores
verde-oliva; sob elas há sombra escura e famílias de pendentes acastanhados.

## Fichas de inspeção e revisitação

Todos os recortes abaixo foram extraídos sem resize/rotação e abertos.

| Ficha | Caixa `(x,y,w,h)` | Segunda leitura e consequência |
| --- | --- | --- |
| F01/base |280,480,370,310| A base não é uma fileira de colunas verticais. Há alargamentos em direção ao solo e faixas grossas oblíquas que podem ser relevos/fustes unidos. Não é possível resolver a seção no escuro. Tronco baixo/inclinado e flare; relevo geométrico/fusão ficam como lacuna. |
| F02/camadas |340,120,620,490| Superfícies de copa quase horizontais sobrepostas, galhos laterais escuros entre elas e aberturas de céu em diferentes alturas. Usar volumes foliares achatados separados por faixas, não uma única elipsoide densa. |
| F03/pendentes |630,390,300,230| Há grupos pardos/bege, bordas franjadas e prolongamentos finos sob o verde. Hipótese de epífitas; a foto não confirma espécie/composição. Precisam de geometria pendente independente das folhas e dos ramos de sustentação. |
| F04/topo |380,50,400,230| O topo tem lobos achatados assimétricos em alturas diferentes. Não é uma tampa plana única nem simetria radial exata. Repetir grupos com variação por seed e alturas. |

Reaberta a imagem inteira: as copas da esquerda e direita também mostram
patamares e franjas. O tronco é majoritariamente oculto na zona superior;
não se pode contar todos os braços nem provar suas uniões internas. Há grama,
agaves/folhas grandes e postes no primeiro plano, campo/horizonte e céu como
contexto. O pedido é o exemplar da árvore: esses objetos não serão reconstruídos;
um chão e iluminação servirão à apresentação. Não confundir a sombra com
relevo medido, nem os pendentes com raízes-pilares do exemplar anterior.

## Lista consolidada anterior à construção

Estado desta entrega: **todos os itens não construídos; validação do modelo
não executada**. Análise visual não aprova uma geometria futura.

| ID/pai | Construir e ocorrências | Evidência/estado | Relações e hipótese3D | Critério de conferência |
| --- | --- | --- | --- | --- |
| RS01/árvore | Tronco curto, muito grosso e oblíquo; uma base aparente, ramificação superior oclusa | F01/observado | Sustenta a copa; seção interna/verso desconhecidos | Relação altura do tronco/altura total e inclinação reconhecíveis |
| RS02/RS01 | Alargamento basal com projeções para os lados; relevos/fustes confluentes escuros | F01/observado; fusão é hipótese | Contato com o solo; não são colunas aéreas repetidas | Base larga e assentada; registrar aproximação circular/flare e ausência de sulcos volumétricos |
| RS03/RS01 | Forquilhas grossas e braços curvos, longos, predominantemente laterais | F01/F02/conjunto; observados parcialmente | Grafo binário proposto, não contagem extraída da foto; braços de trás inferidos | Estrutura despida tem extensão lateral real e junções contínuas, em vários ângulos |
| RS04/RS03 | Galhos finos/terminações ramificadas que sustentam cada massa foliar | F02/F04; família visível, contagem ilegível | Regra procedural hierárquica, sem repetir posições copiadas da foto | Cobertura espacial coerente, sem braços idênticos ou feixes isolados |
| RS05/RS04 | Patamares foliares: topo central, alto esquerdo/direito, médio direito e baixo esquerdo/direito; famílias se sobrepõem | F02/F04/conjunto; pelo menos6grupos aparentes, total oculto | Envoltórias locais achatadas em alturas variáveis, tridimensionais | Copa mais larga que alta, grupos em degraus e topo irregular |
| RS06/RS05 | Vãos entre grupos e galhos: centro alto, centro baixo e entre braços direitos | F02; observado | Fundo passa entre volumes; não escurecer um sólido para simular os vãos | Aberturas continuam legíveis na vista principal e vistas auxiliares |
| RS07/RS05 | Folhagem miúda verde-oliva com variação de brilho/densidade e bordas irregulares | F02/F04; folhas individuais ilegíveis | Lâminas ovadas são hipótese autoral; tamanhos não medidos em metros | Massas foliares laterais, sem repetir a pendência intensa da ornamental |
| RS08/RS03-05 | Grupos pendentes castanho-acinzentados, franjados, de comprimentos variados sob as massas | F03 e alto centro; observado; epífitas como hipótese | Corpos distintos aderidos aos suportes; não solda lenhosa nem identificação científica | Feixes finos pendentes visíveis, sem transformar tudo em raízes que alcançam o solo |
| RS09/RS01-03 | Casca escura/mate, manchas e rugosidade aparente | F01/F02; observado, sombra dificulta | Padrão procedural de material; sulcos volumétricos são capacidade ainda ausente | Madeira visualmente mais escura/irregular; não declarar relevo geométrico inexistente |
| RS10/árvore | Corpo com profundidade: patamares ao redor de vários azimutes | Hipótese necessária, verso desconhecido | Envoltórias3D e eixos fechados, não painéis alinhados à câmera | Frente, lateral/verso e superior coerentes; isso não valida o verso real |
| RS11/apresentação | Chão e sombra para apoio/contexto | Conjunto observado; contexto simplificado | Não reconstruir grama/agaves/postes/céu da foto como parte da árvore | Base apoiada, hipótese de luz explícita e árvore inteira enquadrada |

## Parâmetros, fonte e capacidades

Fonte canônica: quinto preset `figueira_rs` em `botany_exemplars.py`, exportado
como curvas/órgãos para o compilador nativo de botânica. Campos genéricos de
patamares, avanço vertical inicial, abertura das bifurcações, inclinação do
tronco, comprimento dos brotos e pendentes; defaults dos quatro anteriores
preservados. Colares `fork/graft` validam pele lenhosa. Pendentes são superfícies
separadas `sheath` vinculadas mecanicamente aos hospedeiros, sem afirmar fusão.

Não há capacidade existente de seção basal sulcada/fusão de raízes nem
identificação taxonômica. RS02 é aproximado por flare/curva/material; seu relevo
específico continuará divergente. A malha pode estar fechada e ainda ter forma
errada. Fonte/receita4 e os formatos globais serão preservados; inspecionar o
schema real do executável antes de publicar.

Conferência: capturas nativas inteiras, estrutura sem folhas, vários azimutes e
superior; crops locais com caixas iguais na referência/render1000×1000. A câmera
é manual, sem escala/calibração. RGB/SSIM, se medidos, incluem fundo e luz e não
são porcentagens de semelhança. Diagnósticos de proporção são distintos da
comparação RGB e não aprovam regiões ocultas. Execução/revisão ficam em
`docs/botany-rs.md`, vinculadas aos IDs acima.
