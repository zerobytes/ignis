# Traços de superfície e ajuste de domínio material

## Contrato anterior ao código

Direção autorizada: UI depois; ferramentas gerais de folhas/casca na árvore
salva1a043f1. Não criar tipo botânico. Problema observado: um campo de ruído
acopla cor/relevo e não permite detalhes desenhados como nervuras ou costuras.
Alternativas: mais geometria consome desenho e dificulta retesselação; texturas
UV/PBR exigem outro contrato de recursos/samplers/tangentes. Escolha limitada:
camada opcional de segmentos analíticos no domínio material existente, somada
ao material atual, e ajuste nativo de coordenadas por asset/ilha.

Material.detail é opcional, Arc compartilhado, máximo32segmentos. PlanoXY/XZ/YZ
explícito das coordenadas3D de repouso; cada segmento tem start/end2D e larguras
(radiais) nas duas pontas. Tinta multiplicadoraRGB/opacity e bump assinado em
metros da camada, independentes do campo de ruído. Campo é união por máximo,
sem escurecer interseções; relevo máximo5cm. Traços só alteram cor/normal,
sem espessura/silhueta/sombra/colisão. Não é PBR, tinta raster, SSS ou nervura
botânica embutida. Polylines são séries de segmentos, não spline nova.

Por fragmento: distância ao segmento no plano escolhido, largura interpolada,
transição por footprint; traços mais finos que um pixel atenuados explicitamente.
Derivadas em fluxo uniforme e bump pelo kernel de gradiente já existente.
Até32testes de segmento por fragmento com detail ligado, nenhum nearest de
triângulos nem textura externa. Uniforme por slot80+32+32×32=1136bytes,17slots
19312bytes (18,86KiB), contra1360bytes anteriores. Vertex52bytes mantido.
Material deixa Copy para Clone por Arc; clonar histórico/frame não copia todos
os traços. Sem thread/lock/dependência nova. Custo GPU não é promessa de FPS.

fit_material_coordinates recebe asset, per_island e bounds alvo3D. Entrada é
canal existente, fallback posição local. Bounds finitos ordenados até±10000,
eixos de saída podem colapsar. Ajuste affine de cada AABB; eixo de entrada
constante mapeia ao centro da saída. Por ilha usa índices compartilhados e
isolados mapeiam ao centro. Não infere tangentes/orientação por nomes nem weld;
seam topológico continua explícito. Não modifica posições/triângulos/pesos/rig/
clipes. Em planta vinculada é edição de canal como set_material_coordinates:
estabelece nova fonte pelo contrato de rebase existente, não regeneração.

set_material/preview_mesh_edit/inspect existentes expõem camada; fit também no
mesmo dispatcher/undo/staging e marca o asset no lote. Ausência de detail retém
renderer/material legados. Campo novo exige Scene59/replay63; cenas até58 com
camada devem falhar antes de migração. Nenhum formato novo de geometria.

## Checklist estático anterior

1. Fit só na autoria; detalhe só no fragmento habilitado. Nenhuma fase NPC nova.
2. Camada Arc, uniforme fixo por slot; Geometry copy-on-write só no fit.
3. Tipos f32/finito e índices existentes; gradiente/normal no kernel compartilhado.
4. Fit usa partição nativa uma vez, não busca por proximidade; layer soma32
   distâncias limitadas por pixel, sem scan de vértices/faces nesse caminho.
5. Validar contagens/valores antes de payload; budgets V/T/instâncias mantidos.
6. Mesmo Vertex/sombra/cache; tabela maior atualiza somente quando conteúdo muda.
7. Dispatcher/staging síncronos, nenhum lock/worker/IO no tick.
8. Edição invalida documento/Frame/seleção existentes; preview é puro.
9. Testar limites, legado, rebase e preservação, GPU/undo/slots/replay/distância;
   mesma árvore, vistas/luz/ticks fixos e inspeção dos closes.
10. Manter gradiente de bump aproximado, custo de shader/validação/serialização;
    sem FPS, PBR completo, espessura física ou realismo concluído presumidos.

## Revisão durante a implementação

A tabela fixa maior era reconstruída antes da comparação GPU em cada captura/
frame, herdando o caminho pequeno anterior. Corrigido: renderer mantém cópia das
receitas compartilhadas e só converte a tabela se mudarem (ou no primeiro uso).
Animação sozinha não converte19312bytes; comparação de Material usa identidade
Arc primeiro e igualdade de conteúdo quando necessário. Upload segue condicionado
à tabela final. Não é medição de ganho. Fit inicial criava um Vec por vértice
isolado: substituído por inicialização do resultado no centro, sem essas
alocações individuais. Ambos mantêm contratos/resultados.

## Checklist estático posterior

1. Frequência: fit/validação só na autoria/query; avaliação do campo no fragmento
   com detail habilitado. Nenhuma fase NPC/física/frame de skinning acrescentada.
2. Alocações: Detail Arc, sem copiar segmentos por vértice/histórico; pesos/rig
   intactos. Fit aloca coordenadas de saída e grupos limitados; isolados não
   geram Vecs individuais. Frame só clona receitas se documento mudar.
3. Tipos: coordenadas/contagens finitas limitadas, raio mínimo1e-6 e segmento
   não degenerado. Fit usa f64 intermediário contra overflow/cancelamento;
   saída f32. GPU1136B/slot validado; sem unsafe/dependência nova.
4. Repetição: AABB por grupo após partição nativa, sem nearest geométrico. Até32
   distâncias por fragmento habilitado, inclusive no caso distante; fade limita
   o sinal, não é promessa de remover todo custo. União por máximo nos encontros.
5. Limites:32traços/asset, tint0..2/opacity0..1/bump±5cm/radius1e-6..10 e domínio
   ±10000; mesmos budgets V/T/assets/instâncias/ossos/chaves. Erros atômicos.
6. GPU/I/O: Vertex52B e sombra mantidos; tabela19312B só reconvertida quando
   receitas mudam, inicialização garantida no primeiro uso e upload cacheado.
   Fonte/arquivo/capturas somente explícitos. Sem FPS alegado.
7. Sincronização: dispatcher/preview/staging/undo existentes, nenhum worker,
   lock, material cooking assíncrono ou sistema de recursos externo novo.
8. Invalidação: fit marca asset alterado nos lotes; material/shading/canais
   invalidam Frame pelo documento. Cache por valor com atalho de Arc preserva
   igualdade semântica mesmo após load gerar outras alocações. Slots reindexados.
9. Correção: Rust de validação/fit/subdivisão/extração/plantas/legado/layout;
   MCP de pixels/cor e bump isolados, subpixel, rig compensado, seleção/preview,
   undo/slots/replay/load/reset/viewport. Mesma árvore com ablações folha/casca.
10. Custos mantidos: aproximação de distância com raio interpolado, fade pelo
    maior footprint e gradiente de bump por derivadas; não integração analítica
    ou spline/extrusão. JSON e validação integral limitados. Fibra em curvas,
    espessura geométrica e PBR continuam pendentes. Evidência em validation.md.

## Correspondência de domínio — extensão antes do código

O teste da copa encontrou ausência de canal material local nas variantes antigas;
fit sobre suas posições já rotacionadas normaliza tamanho, não orientação.
A folha isolada correta não prova sua cópia na copa. Alternativas PCA/nearest
introduzem inferência ambígua de eixos/sinais e transferência inexata. Escolha:
copy_material_coordinates com asset destino, source explícito e per_island.
Só aceita mesma contagem de vértices e sequência de triângulos remapeada por
ordem local de vértices. Isso declara correspondência, não a descobre. Fonte
precisa canal explícito; modo por ilha exige fonte com uma ilha/sem isolados e
todas as ilhas destino compatíveis/sem isolados. Triângulos/ordem incompatíveis
recusam a transação, não geram aproximação. Whole-asset também exige topologia
idêntica. Não copia posições, rig, material ou clipes.

Pré-custo: partição uma vez, validar todas as correspondências antes do Vec de
saída, remap por array de índices sem nearest. Só na autoria, mesmos budgets;
fit folha-fonte→copy66ilhas é lote único. Em planta vinculada segue rebase de
canal existente; marca só destino alterado. API produz canal já existente,
sem consumir nova versão além59/63. Testar orientação via template explícito,
rejeição de triângulos/contagem/canal ausente, preview/undo/persistência e copa.

## Resultado na árvore e limites observados

A cena1a043f1 foi carregada, sem reconstruir outra árvore. fit folha-fonte e
copy por topologia aplicam o mesmo domínio a66ilhas de cada uma das4variantes;
inspeção nativa confirma264folhas armazenadas e correspondências exatas. As
instâncias repetem esse conteúdo nas1782folhas desenhadas. A comparação inclui
folha-fonte e uma folha extraída da copa real por Copy nativo, com rig/clipe
mantidos, câmera derivada da pose e undo completo da montagem de inspeção.
Isso evita usar apenas a folha isolada como prova de orientação na copa.

31segmentos descrevem nervura central/laterais dobradas/ramificações menores;
primeira receita reta/simétrica foi refinada após inspeção. Casca usa ruído
ridged anisotrópico e32segmentos em8polylines para fissuras, substituindo a rede
celular anterior. Ablações removem só detail e conservam o resto para mostrar
a contribuição da camada. Comparações antes/depois têm câmera/luz/tick iguais.

Posições/triângulos, rig/clipes, instâncias e poses nos ticks0/60 permanecem
exatos em todos os assets; canais mudam somente nas folhas. Planta mantém
source geometry/rig/clips e reference/definition; material é alteração autoral
explícita, não promessa de material antigo igual. Madeira inteira/primeirofork
mantêm geometria.10assets/32instâncias11172v16250t armazenados64794t desenho,
742t livres; nenhum teto ampliado.12casos de retesselação/edição da planta,
undo/redo/persistência/replay e48frames distintos continuam exercitados.

Limites importantes: fit normaliza, não alinha eixos. Copy exige correspondência
por índices/triângulos ordenados; não transfere para topologia diferente e não
cria vínculo vivo com o template. Cópia explícita precisa ser repetida após
reautoria. Material continua difuso, sem roughness/specular/UV-texturas PBR;
bump não cria espessura, silhueta ou sombras das nervuras. Folhas ainda têm
contorno poligonal e espessura zero. Casca estriada permanece procedural;
plano de traços não acompanha sozinho frames de todas as bifurcações.

31/32segmentos já atingem o recorte. Redes muito mais densas pedem avaliar
curvas/máscaras ou representação preparada, com contrato de recursos próprio,
em vez de elevar loops por fragmento indefinidamente. Fade é conservador e
perde contribuição média subpixel, não filtragem analítica sem aliasing.
Mais geometria por todas as folhas excederia742t livres: até1tri/folha exigiria
1782t. Não inferir espessura/LOD/escala de produção ou realismo concluídos.


## Extensão71: seção e domínio

`Detail.profile` e `Detail.repeat` estendem a mesma receita e seus comandos.
Band/None conservam o caminho anterior. Rounded e patch periódico com suporte
contido estão descritos em [detail-domains.md](detail-domains.md); tabela atual
1152 B/slot,19584 B no total, Vertex64 B após triplanar70. Os tamanhos52/1136
acima descrevem o incremento original59, não o layout corrente. Não altera
espessura, silhueta, física, sombras ou cria ferramenta especializada em árvores.
