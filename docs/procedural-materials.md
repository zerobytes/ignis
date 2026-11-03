# Materiais procedurais de superfície — incremento EAF-80

Reusable layered recipes, fixed-map assets and per-instance runtime variation are
documented in [material-recipes.md](material-recipes.md). This page retains the
original procedural-pattern contract and its historical limits.

## Motivação, alternativas e contrato antes do código

Após smooth e sombra, o close da árvore continua com uma superfície marrom lisa
e folhas uniformes. Esculpir cada sulco ou adicionar triângulos para cor não é
adequado ao budget. Texturas UV/PBR completas exigem recursos, samplers, UVs,
tangentes, persistência e transferência de canais que ainda não existem.

Recorte escolhido: material genérico opcional por asset, com padrão de ruído3D
autorado, variação de cor, bump de normal e transmissão difusa para superfícies
finas. Mesmo recurso precisa poder servir madeira/pedra/solo/folhas; não incluir
um ramo de shader condicionado a nome de árvore/asset. Não chamar de PBR completo,
normal map importado, espessura física ou displacement de geometria.

`Asset.material` padrão neutro; `set_material` e `inspect_mesh` pelo MCP.
Configuração fixa e limitada: frequência XYZ local, duas tintas multiplicadoras,
warp, octaves1..4, seed16bits, contraste, padrões noise/ridged/cellular e bump0..5cm;
transmission0..1. Material preservado por subdivisão e regeneração; alteração de
metadado em planta não rebasa a fonte canônica. Cenas anteriores ficam neutras.

O padrão usa coordenada de material3D opcional por vértice, antes do skinning;
ausência usa a posição local de repouso. Sem o canal, um padrão anisotrópico
vertical não seguiria galhos inclinados: isso foi identificado no desenho da
receita de casca. O sweep autoral fornece coordenadas radial/longitudinal locais
para acompanhar cada ramo. Assim animação não desliza o padrão pelo mundo. Escala da instância
escala o padrão junto; mudança dimensional por regeneração reavalia o campo nas
novas posições locais. Não há UV canônico ou promessa de preservar uma mancha
específica quando se usa fallback de posição em mudanças de domínio.
Canal explícito é conservado na escultura/skinning e interpolado no subdivide
e na regeneração por correspondências, com finito/contagem validados.
`set_material_coordinates` edita o canal no mesh corrente e estabelece fonte de
planta como os outros edits geométricos; `set_material` só altera metadados.
Source→A→B→A mantém receita/canal sem bake acumulado.

## Custos e limites implementados

Alternativa de copiar toda receita em cada vértice desperdiça upload. Preferir
tabela GPU até17 slots (16 assets+neutro), índice de material e posição local no
Vertex; slots são derivados do frame, nunca identidade persistente. Acréscimo
de16bytes/vértice expandido (36→52), até1,5MiB extras para32768triângulos.
Uniformes de receitas limitados a1360bytes, atualizados só quando mudarem.
Mesma geometria e mapa de sombras existentes.

Ruído de valor com hash inteiro, até4 oitavas; reduzir detalhes abaixo do pixel
por footprint da posição local. Bump deriva gradiente do campo no fragmento:
não altera silhueta, colisão, poses ou mapa de sombras. Transmissão é resposta
difusa de duas faces, sem refração/translucência volumétrica ou transparência.
Novos atributos e derivadas são custo mesmo sem padrão; ruído deve ser opt-in.
Sem novas threads/locks/arquivos externos/depêndencias da engine; não alegar FPS.

Validar limites/finito/persistência, neutralidade legada, preservação de fonte e
clipes, slot reinserido/removido sem material antigo, padrão ligado ao repouso
no skinning, pixels/undo/load/replay, densidade/aliasing e clamps do bump. Sombras
precisam manter buffers/layout coerentes. Discussão do contrato e reserva de
versões antecedem implementação.

## Semântica e limitações dos padrões

Noise usa ruído de valor trilinear com hash inteiro, ridged aplica módulo ao
campo, cellular usa as duas menores distâncias entre27 células vizinhas de3D
para separar placas. Até4 oitavas normalizadas, warp em baixa frequência e
contraste limitado. O footprint atenua as oitavas subpixel e evita avaliá-las
quando a cobertura chega a zero; não é integração analítica do celular nem
promessa de ausência de aliasing nas linhas finas. Cellular custa27 buscas por
oitava: escolha explícita da receita, não padrão de todos os assets.

`set_material_coordinates` recebe uma tripla por vértice corrente. Nenhuma
projeção ou orientação de fibra é inferida pelo nome do objeto. A escultura
conserva o canal, midpoint usa média, regeneração usa os mesmos triângulos e
baricêntricas verificados pelo kernel de preservação. Vértices interiores de
novas superfícies sem correspondência precisam de autoria própria do canal;
não há promessa de continuidade de grão em junções ainda não mapeadas.

Cena54/replay58. Arquivo antigo com receita não neutra ou canal explícito é
recusado antes de migrar; não esconder campos novos sob uma versão antiga.
Slots GPU são refeitos da ordem do documento: remover asset não pode aplicar
material antigo no sobrevivente. A tabela é enviada apenas quando muda; a
posição de repouso e o slot acompanham o upload existente do mesh animado.
Transmissão conserva a opacidade do mapa de sombras. Não é PBR, SSS, normal map,
textura importada, transparência, espessura física ou deslocamento geométrico.
