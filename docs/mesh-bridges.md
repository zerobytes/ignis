# Transições por bordas reais — EAF-85/EAF-80

## Decisão e revisão antes do código

O ensaio de grupos3..6/campo radial foi testado e rejeitado artisticamente na
árvore d39c73a: mesmo com topologia válida, suportes convexos/centro comum
obrigavam afastar saídas e criar curvas artificiais. Não promover o kernel ou a
receita por contagem/testes. Capturas e código do experimento em
artifacts/tree-groups e artifacts/group-probe/rejected; cena central preservada.

Escolha seguinte: remove_faces por IDs/referência de sessão; inspeção dos ciclos
de borda reais; bridge_boundary_loops no MESMO asset, compartilhando os índices
das duas bordas. Não assumir círculos, centro radial ou planos de suporte. Isso
serve a peças, personagens e estruturas, sem tipo de árvore. Não é boolean geral,
weld por proximidade ou transferência silenciosa entre fontes paramétricas.

remove_faces recebe faces não vazias/únicas e mantém ao menos uma face; compacta
vértices utilizados em ordem, preserva coordenadas/pesos e rig/clipes/materiais.
Referências expiram pela revisão já existente. inspect_mesh_boundaries retorna
ciclos orientados pelos triângulos atuais, centros e referência; junção de
bordas ambígua/não manifold falha. Seleção de faces/loops é explícita.

bridge_boundary_loops recebe loops[2], spans2..32 e handles[2]0.0001..2m. Mesma
contagem3..64, ciclos disjuntos; primeiro reverso da borda existente, segundo no
sentido da borda existente. Ordem/fase fornecida pelo autor. Normais médias de
área do asset e tangente de cada loop determinam derivadas de saída/entrada
no plano tangente. Cada par vira curva cúbica; amostras t=(1−cos(πi/spans))/2 concentram resolução
nas duas bordas sem aumentar o número de vértices; intermediários e duas
faces por quadrilátero usam índices compartilhados. Checar Jacobiano amostrado
e orientação de cada triângulo; também recusar ângulo >=90° entre faces
adjacentes quando ao menos uma é nova, incluindo as duas bordas. O Jacobiano
isolado podia acompanhar uma dobra da própria superfície. Essa restrição
conservadora pode recusar superfícies válidas muito curvas; não relaxá-la
silenciosamente para acomodar o exemplo. Sem alegar prova global
de autointerseção. Guias e shader não mudam para esconder problemas.

Só novos pontos interpolam coordenadas e pesos entre bordas por t. Quatro
influências positivas máximas: reutilizar transfer_weight, sem truncar. Rig/clipes/
materiais e pontos antigos intactos. Continuidade tangencial é condição da curva
em repouso, não garantia de suavidade de toda malha finita ou após qualquer rig.
Local topology em planta vinculada recusa, exigindo detach explícito fora deste
ensaio; não perder fonte/rig para tapar o encaixe oposto paramétrico.

## Checklist anterior

1. Autoria/query somente, sem nova fase por tick.2. Remoção/ponte alocam somente
payload limitado do asset no staging; preflight V/T/desenho antes da ponte.
3. f64 em curvas/normais/derivadas, f32 saída e índices existentes; nenhuma
nova dependência/unsafe/ABI.4. Normais em um passe por face; vizinhos/loops em
mapas finitos, sem nearest de superfície ou Vec por ponto no tick.
5.4096v8192t por asset e budgets globais mantidos.6.Mesmo Vertex52/tabela19312/
Globals176/sombra; mais geometria custa preparação/upload/desenho existentes.
7.Dispatcher/preview/undo/staging existentes, sem worker/lock novo.
8.Referência de sessão/revisão protege seleção, lotes recusam fonte alterada
antes de operação referenciada; sem novo ID persistente.
9.Testar ciclos/orientação/curvas/limites/rig4/rollback/plantas/undo/save/replay e
GPU na árvore, câmera/luz/ticks fixos; preservar folhas/materiais/source/poses.
10.Mantidos: amostragem finita, transferência de canais por interpolação explícita,
sem solver de todas as junções, auto seleção, fibras transportadas, UI ou FPS.

Alvo: dois encaixes superiores estáticos por recortes locais e pontes tangentes;
facetas por resolução/derivadas nas transições, conservando pontos fora dos
recortes. O ramo paramétrico continuará protegido até haver composição compatível
com sua fonte, sem bake/detach implícitos. Evidência de limitação deve ser registrada.

## Recorte de entrega

As pontes dos dois ramos superiores foram reprovadas: a melhor fase da busca
ainda chegou a126.126° em faces internas, apesar de bordas aceitáveis. O teste
revelou a insuficiência do Jacobiano por triângulo e motivou a guarda adjacente.
A versão salva mantém esses componentes originais. Só o trecho central entre
loops reais de32pontos é promovido, com10intervalos e comparação de toda a
região, além das bordas. Encaixes restantes continuam abertos, sem declarar
todos seamless. Não substituir falha de forma por contagem/topologia válida.


A região central sozinha ainda tinha facetas internas. O ensaio aplica depois
seis strokes do pincel Smooth existente: duas passagens, força0.3, centros em
pontos novos a>5mm de todos os pontos retidos e raio2mm. Cada chunk tem no
máximo128centros; o preview nativo é igual à publicação e todos os pontos antigos
são conferidos exatos. Isso é autoria explícita com ferramentas genéricas,
sem relaxamento oculto no contrato da ponte. Máximo deslocamento22.2593mm,
variação do volume total de wood0.065284%; esse número não é conservação do
volume local do pescoço. Máximo diédrico da região24.2474°→22.8988°, média
6.11391°→5.26886°. Ainda existem facetas; não alegar continuidade C1 da malha.

Fricções restantes: o pincel precisa de seleção/máscara explícita de vértices,
para dispensar centros pequenos e proteção espacial externa; o preview genérico
serializa o documento inteiro por tentativa; fase/alças/recorte continuam
manuais; medir erro/ângulos no preview nativo ajudaria a distribuir a tesselação.
Essas melhorias são genéricas, não um gerador de árvores.
