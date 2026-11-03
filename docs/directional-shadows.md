# Sombras direcionais — EAF-80

## Correção opcional do receptor — EAF-208

`Light.shadow.receiver_plane_bias: true` corrige a profundidade de comparação
de cada amostra PCF pela inclinação geométrica do receptor. O mesmo `set_light`
do MCP e o painel Iluminação/Luz da cena expõem a opção. Ausência/false preserva
o caminho anterior e omite o campo ao serializar. Scene111 exige o novo campo
ativo; SetLight sobe a versão atomicamente, com undo. Replay121/conteúdo9;
protocolo5/checkpoint3/PSV1 e fontes botânicas/personagens não mudam.

Derivadas da posição projetada fornecem dz/du e dz/dv antes de sair do volume de
luz. O PCF mantém nove comparações e compensa também a área interna do filtro
de comparação. Jacobiano degenerado ou quase singular mantém o bias constante;
a inclinação é limitada a 1% da profundidade por texel nos casos rasantes. É uma
aproximação local plana: não elimina toda acne/descolamento em superfícies
curvas, descontinuidades ou cobertura grosseira. O bias autoral ainda existe.

Não adiciona targets, leituras de textura, vértices nem bytes no layout Globals:
reutiliza o quarto componente do uniforme de sombra. Aritmética/derivadas têm
custo próprio a medir. Não cria penumbra física, cascatas, oclusão ambiente ou GI.
Diagnóstico, revisão estática, fontes e laboratório em
[runtime-visual-quality.md](runtime-visual-quality.md).

## Contrato e avaliação anterior ao código

O teste da árvore precisa distinguir volumes que apenas recebem cores de
volumes que bloqueiam luz. Escolha inicial: mapa de profundidade ortográfico,
opt-in em `Light.shadow`, reutilizando os buffers opacos existentes. Alternativas
de cascatas, ray tracing e iluminação global exigiriam outra arquitetura; não
entram neste recorte. Sem desenho específico para árvores.

`set_light`/SceneEdit/MCP recebem `shadow: {center, extent, resolution, bias}`;
null/omissão desativa. Center é XYZ no mundo; extent é meia largura/altura do
volume ortográfico em metros (0,25..256), com profundidade4×extent; resolution
512/1024/2048; bias é deslocamento comparativo normalizado de profundidade
0..0,01. Autoria escolhe o volume explicitamente. Fora dele não há sombra; não
garantir cobertura infinita ou ajustar o volume silenciosamente à câmera.

Modelo53/replay57, ausência mantém cenas legadas. Nenhuma alteração no layout
Vertex ou no asset de mesh. O mapa recebe somente caixas/malhas opacas presentes;
guias/seleção não bloqueiam luz. Não altera colisão ou percepção. Filtragem PCF3×3,
sem transparência/transmissão, PBR, GI ou contato por screen space. Ambiente
escalar continua sem oclusão: sombra atenua a contribuição direcional.

Checklist anterior: mapa persistente Depth32Float, máximo16MiB a2048² mais
driver/targets. Recriar apenas ao mudar resolução; placeholder1×1 desativado.
Um passe de profundidade extra por render com sombra ativa (até2 draws para
caixas/malhas), sem uploads novos de geometria. Shader lit faz9 comparações por
fragmento dentro do volume. Matrizes e uniformes pequenos por frame; sem novos
threads/locks/readbacks. Custo mantido: mapa é redesenhado mesmo estático neste
primeiro recorte, reutilizando buffers; não alegar ganho de FPS. Capturas são
explícitas e regressões headless isoladas.

Verificar matriz finita/luz vertical/bordas do volume, limites/atomicidade e
migração. MCP compara sombra ativa/desativada/luz movida, sombras em meshes e
caixas, remoção/undo/resize/reset/save-load/replay/prefixo e invisibilidade de
guias no mapa. Acne, resolução, bias e clipping são limitações a inspecionar.

## Implementação e revisão posterior

O layout de vértice continua posição/normal/cor. Uniformes acrescentam matriz
de luz e parâmetros; passe de profundidade tem bind group só de uniformes para
evitar ligar o mesmo mapa simultaneamente como attachment e textura amostrada.
Shader de cena recebe o mapa/comparison sampler, faz PCF3×3 em lod0 e deixa
ambiente inalterado. Fora do volume retorna visibilidade1. Guias/seleção não são
submetidos ao passe de sombra. Desativação volta ao placeholder1×1.

Primeira inspeção na árvore revelou acne listrada no tronco com bias0,00025
(extent5,8m/res2048). Bias0,0008 removeu as listras evidentes na captura da base,
mantendo sombra da copa/contato visíveis. Esse ajuste é autoria explícita da
cena, não correção universal de acne. Bias alto ainda pode destacar sombras do
objeto; mapas finitos e filtragem não garantem todos os ângulos/distâncias.

Custos:4/16MiB de depth em1024/2048, até2 draws extras; não há readback per tick
ou upload separado de geometria. Mapa redesenhado em toda apresentação com
sombra habilitada. Sem cache de oclusores estáticos, cascatas ou transparência.
Validação matemática e cenário MCP próprio verificam sucesso/erros atômicos,
remoção/restauração de oclusores, resolução/viewport, replay e prefixo. Resultados
finais em validation.md; inspeção não equivale a aceite artístico da árvore.
