# Plantas curvas por seções elípticas

Extensão posterior EAF-97 em [plant-trajectories.md](plant-trajectories.md):
trajectory explícita é independente dos offsets de seção e orienta os cortes
pela tangente da curva. Este documento descreve o loft de planosXZ legado
quando trajectory está ausente. PCHIP continua disponível para dimensões e
offsets no plano normal quando a nova trajetória está presente.

## Incremento C1 — decisão anterior ao código

O close A/B de branch-editable-1 confirmou quinas entre estações. Há duas
alternativas: Hermite com tangentes autoradas permite direção explícita, mas
também overshoot e raios negativos sem envelopes; PCHIP calcula tangentes
limitadas, conserva extremos escalares e oferece menos controle autoral.
Escolha deste incremento: `sections.interpolation: pchip`, opt-in. `linear`
continua default, omitido ao serializar e numericamente idêntico ao legado.

PCHIP aplica-se independentemente a width/depth/offsetX/offsetZ em função do
Y físico normalizado (`Station.at`), inclusive com espaçamentos não uniformes.
Y e a correspondência por IDs/intervalo permanecem lineares e ordenados. A
inversa avalia as mesmas quatro funções no Y conhecido, sem solver/proximidade.
Nos extremos/trechos constantes as tangentes são limitadas; os valores de cada
intervalo permanecem entre seus endpoints, preservando escalas positivas. C1
refere-se à forma contínua por Y; não exige continuidade da segunda derivada,
não rotaciona planos nem garante que uma malha pouco amostrada pareça lisa.

Base matemática: [documentação primária do método PCHIP](https://docs.scipy.org/doc/scipy/reference/generated/scipy.interpolate.PchipInterpolator.html),
Fritsch–Butland, SIAM1984. Implementação Rust local, sem dependência SciPy.
Coeficientes Hermite em f64 limitados a15intervalos×4canais×4coeficientes,
compilados uma vez por amostragem. Não refazer tangentes por vértice. Inversa
e consultas pontuais compilam dados limitados na pilha; nada por tick.

Cena57/replay61 reservadas com a frente central. Target OU referência com
PCHIP exige formato57; tipo/IDs de seção continuam protegidos, mudança de
interpolação pode usar a fonte canônica existente. Sem ampliar budgets. Testar
C1, extremos/platôs, duas estações, espaçamento mínimo e valores racionais independentes, inversa,
geometria/preservação A-B-A, formato/undo/load/replay/MCP/GPU. Comparação visual
mantém mesmos pontos e resolução; uma segunda resolução distingue função de
tesselação. Aplicação final na MESMA árvore permanece com o outro chat.

Revisão estática anterior proporcional: avaliação eventual; até1920B de
coeficientes e scratch fixo limitado; f64 no cálculo/f32 na saída; geração
linear em estações+vértices fora do mapa topológico existente; nenhuma busca,
GPU/I/O periódico, thread, lock ou cache persistente novo; preservar fonte Arc,
materiais e invalidações/revisão; validar limites antes da compilação; manter
custos de fonte+cache, validação duplicada e skinning CPU sem alegação de FPS.

## Decisão anterior à implementação

O alvo é a mesma árvore de EAF-80. A autoria central ainda calcula sweeps em
Python. Este incremento traz à engine um loft editável com fonte canônica,
sem acrescentar um objeto especial de árvore ao runtime.

Um sweep XYZ com frames transportados permite retornos e loops, mas sua inversa
pode ser múltipla ou inexistente. Exigiria um domínio injetivo, diagnóstico de
interseções e uma política de orientação para detalhe e ossos. Não substituir
essa decisão por uma busca do ponto mais próximo. O recorte escolhido mantém
Y estritamente ordenado: cada seção fica num plano XZ, com centro deslocável,
largura e profundidade independentes. O eixo pode curvar nos dois planos; os
planos de seção continuam paralelos, não perpendiculares à tangente da curva.
Segmentos entre estações são lineares. Não é um sweep XYZ livre nem spline.

`cross_section: ellipse` usa o domínio quadrado material existente, mapeado no
disco por direção radial: multiplicar (x,z) por max(abs(x),abs(z))/length(x,z).
A inversa multiplica o disco por length/maxabs. No centro ambos são identidade.
Escalas positivas e Y ordenado tornam o mapa de volume único. Os offsets XZ
são metros locais (`offset: [x,z]`, default zero), interpolados por intervalo
de IDs de estação. Instâncias continuam orientando o objeto no mundo.

A superfície elíptica compartilha os índices nas seis grades do domínio;
fronteiras materiais não são seams geométricos. Correspondência usa a face e
célula material existentes, sem solda por proximidade. Topologia de seções
retangulares/diamante antigas permanece idêntica. Mudar o tipo de seção ou
IDs continua exigindo desvinculação explícita. Fontes, escultura vetorial em
XYZ local, pesos e clipes usam o contrato de preservação existente. Não se
promete transporte de vetores pela tangente ou rotação automática dos ossos.
Coordenadas de material autoradas na fonte devem continuar sendo transferidas.

Formato novo reservado em coordenação: cena56/replay60, base compartilhada55/59.
Formato antigo não pode ocultar elipse/offset em definition ou reference.
O kernel recusa dimensões/offsets não finitos, budgets e perda amostrada.
Limites herdados:2..16 estações,1..63 segmentos por eixo,4096v/8192t por asset.
Offsets limitados a +/-100m. Sem nova dependência, thread, renderer ou cache.

## Revisão estática anterior

1. Avaliação esparsa na autoria/validação; nada por tick.
2. Fonte Arc mantida; map de coordenadas inteiras e índices de grade temporários
   somente na geração da elipse, limitados pelas contagens antes da alocação.
3. Material/físico em f64 no mapa/inversa, saída f32; IDs/triângulos existentes.
4. Deduplicar por chave inteira do domínio, O(V log V), sem nearest search.
5. Preflight exato V=2(xy+xz+yz)+2, T=4(xy+xz+yz) para superfície fechada;
   refinamento usa E=3T/2 e orçamento por nível. Sem ampliar limites.
6. Renderer e material existentes; capturas só explícitas, sem I/O por frame.
7. Dispatcher síncrono existente; sem locks/trabalho assíncrono novo.
8. Fonte canônica/IDs/revisão e undo/load mantidos; preview puro.
9. Testar inversa/continuidade/manifold, ossos fora do eixo, escultura/pesos/
   deltas TR, A-B-A, tolerância de retesselação e formatos/replay/MCP/GPU.
10. Manter custo de validação repetida/fonte+cache/skinning CPU; nenhuma
    alegação de FPS ou realismo artístico a partir dos testes.


## Montagem com a árvore central

A receita `scripts/tree_plant_recipe.py` descreve o mesmo eixo/radius do tronco
seed20261026 em11estações e é triangulada somente pela engine. Continua sendo
um estudo editável do tronco: não substitui silenciosamente a madeira central.
Colares do kernel orgânico atual são círculos uniformes e planos próprios;
a elipse usa perímetro derivado do quadrado material e planos XZ paralelos.
Mesmo32amostras não estabelece correspondência por posição/ordem com esses
colares. Integrar exige contornos e frames explícitos de ambos os domínios;
repetir união por sobreposição perderia o objetivo da superfície contínua.


### Avaliação visual do modo suave

O cenário smooth_plants usa o MESMO branch-editable-1 da fonte48aa4d4,
sem gerar mesh em Python. Compara Linear-A/PCHIP-A/Linear-B/PCHIP-B em
258v512t, com mesmas estações/câmera/luz/tick, e compara Linear-B/PCHIP-B
em706v1408t([4,6,4]). O modo linear denso mantém o cotovelo; PCHIP denso
arredonda sua passagem. PCHIP com poucas amostras ainda fica facetado. Essa
comparação separa custo de tesselação de formato da função. B é uma edição
exagerada de diagnóstico; não é recomendação de forma final. Casca e encaixe
sobreposto continuam reprovados no alvo artístico. A integração final fica
na mesma árvore com A, preservando6inserções e primeiro fork.

### Integração comprovada e próxima restrição de superfície

A integração488df28 da frente central importa o núcleo8aa1a11 como761c77b,
sem mudar o código do kernel. Carrega a cena56 anterior de48aa4d4, byte exata,
e altera a planta existente: source E reference históricos permanecem exatos,
assim como wood inteiro, outros assets/instâncias e pose foliar no mesmo tick.
Doze casos usam Linear/PCHIP×A/B×2/4/6segmentosY. Seis centros de inserção têm
erro máximo6.59e-8m. A resolução4 foi escolhida para A final:482v960t no ramo,
64794t desenhados/742livres na árvore. A resolução6 foi capturada e passou,
mas não justifica seu custo como default nesta forma. Regressões e capturas em
validation.md; realismo/encaixes/casca/folhas permanecem com EAF-80.

Para uma montagem futura em que tronco E ramo usem loftsXZ/Identity, aumentar
3→4contornos não resolve duas saídas paralelas+Y: dois planos distintos com a
mesma normal não podem ser AMBOS suportes estritos do conjunto; isso exigiria
yA>yB e yB>yA. Essa é uma restrição sob hipótese explícita, não uma prova de
recusa da montagem atual. O tronco atual foi produzido por sweep e o colar
trunk_above usa tangent(0.42), aproximadamente[0.07034686,0.99711287,0.02858756];
somente o ramo editável tem seçãoXZ/Identity. Quase-paralelidade não equivale a
normais idênticas. A admissão da próxima junção exige os contornos/planos reais.
Os forks0.34/0.44 também são próximos da região já conectada até0.42. Não
publicar outra união sobreposta como superfície única.

Alternativas a apresentar antes de implementar esse próximo contrato:

- Frame autoral FIXO por loft: transformação rígida mantém a inversa analítica
  atual e permite saídas com normais diferentes. Exige política explícita para
  referência histórica, vetores de escultura, escala proporcional e rig ao
  mudar o frame; não confundir com transformação da instância ou preservação
  automática de uma união já existente.
- Corte oblíquo e contorno explícito: conserva o volume autorado, mas exige
  interseção/validação do contorno e um kernel que aceite sua amostragem real,
  incluindo material/rig na montagem. Não usar nearest-weld silencioso.
- Frames por estação/sweepXYZ: maior liberdade, mas o domínio pode deixar de
  ser injetivo; requer garantia de inversa, limites e política de torção.

Nenhuma dessas alternativas foi implementada neste incremento. A escolha de
grafo/ownership de uma malha conectada também deve explicitar como transportar
regiões existentes e inicializar miolos novos. C1 da curva não encerra esse
problema de superfície.
