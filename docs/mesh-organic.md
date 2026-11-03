# Junções orgânicas conectadas — contrato inicial

## Continuidade entre guarda e campo — incremento EAF-85

Pedido de Guilherme: corrigir as quinas reais dos galhos no close da casca.
O eixo Bézier já é uma função, mas o pescoço usa apenas quatro intervalos e
o kernel anterior fixa um cilindro antes de projetar o miolo em outro campo.
Essa passagem não impõe tangência. Normais smooth não corrigem a geometria.

Alternativas: aumentar somente a amostragem conserva uma quina matemática;
relaxamento genérico move a forma sem uma condição de contorno explícita;
um grafo de sweeps com frames móveis/transferência de domínio é mais amplo e
caro de substituir. O recorte escolhido é uma condição de contorno no campo
existente, mantendo colares, topologia, limites e o caminho legado exatos.

`Fusion.transition_lengths:[f32;3]` é uma opção da consulta pura, em metros,
na ordem dos colares. Zero nos três mantém o algoritmo antigo. Três valores
positivos usam a combinação do campo com o cilindro local em uma faixa após
a guarda. O peso é `h(u)=u³(10-15u+6u²)`, saturado em0..1. Tanto h como
sua primeira derivada são zero na guarda; em1 o campo original é recuperado
com derivada do peso zero. O campo implícito encontra o cilindro com normal
radial onde o gradiente é regular. A malha continua uma aproximação finita:
não declarar que triângulos individuais formam uma superfície C1 exata.

Faixas de outros colares devem estar inativas sobre todo o cilindro de guarda
de cada abertura; validar a distância mínima analítica dos círculos aos planos,
incluindo seus extremos axiais. Recusar faixas que escapem do domínio; não
encurtar valores silenciosamente. O modo novo continua limitado ao domínio
radial admitido e não prova ausência global de autointerseções. Publicação
continua explícita, sem inferir transferência de escultura/rig para miolos novos.
Sem Scene/replay novos: a opção só persiste como geometria publicada existente.

Ensaio no mesmo arquivo da árvore após db3b3de: bandas explícitas
`[0.20,0.10,0.03]`, colares32/refinamento2. A região nova é só o primeiro fork.
O pescoço antigo tinha produto raio×curvatura acima de1 em amostragem, podendo
dobrar o offset apesar de um eixo suave. Alças Bézier0.575/0.375m distribuem
o giro mantendo início/fim e direções tangentes. Taper Hermite impõe inclinação
zero na abertura cilíndrica e `dr/ds` igual ao trecho retido no splice. Frame
usa derivadas analíticas do eixo; relevo geométrico entra com o mesmo peso de
derivada nula. Oito intervalos no pescoço substituem quatro. Isso é uma receita
autoral usando o kernel geral, não um novo tipo de árvore nem um sweepXYZ nativo.

Medição na borda do ramo: máximo diédrico46.1047°→10.6236°; na junção interna
31.6454°→22.7334°. Portanto há melhoria real mas a aproximação triangular ainda
tem ângulos. Produto raio×curvatura máximo amostrado0.734084 em1001pontos;
todas as faces laterais do pescoço orientadas para fora em relação ao eixo.
O primeiro experimento só aumentava amostragem e foi rejeitado por dobra155°:
essa regressão motivou o teste de orientação, além de testar tangentes.

Outros pontos de inserção, inclusive encaixe sobreposto do lado oposto visível
no close, continuam abertos. A derivada do campo não torna esses outros
componentes uma superfície conectada, nem garante segunda derivada nos ramos.
Demais assets/plantas/rig/clipes/instâncias e poses0/60 permanecem exatos.
O canal material é preservado no patch/raiz/tronco e madeira restante; o novo
pescoço recebe domínio de repouso autorado explícito, sem transferência
implícita de escultura. Dados em artifacts/tree-continuity/artifacts/report.json.

Checklist anterior ao código: (1) consulta/autoria, nunca tick; (2) mesmas
alocações limitadas do kernel, sem histórico novo; (3)3comprimentos fixos/f64
temporário; (4) no máximo3misturas extras por avaliação, bisseção40passos
mantida; (5) mesmos4096v/8192t e refinamento; (6) sem GPU/I/O novo; (7) sem
locks/dependências; (8) sem cache novo; (9) testar campo/tangência, legado
exato, topologia/bordas, envelopes e pureza MCP; (10) custo de autoria e
aproximação por triângulos mantidos conscientemente, sem alegar FPS.

A árvore realista de EAF-80 é o alvo compartilhado. A inspeção de after,
bark-detail e structure do build6278f10a mostra que o sombreado melhora facetas,
mas conserva linhas de inserção: tronco e ramos ainda são tubos sobrepostos.
Assimetria e quantidade de folhas não resolvem essa falha de superfície.

## Alternativas e decisão deste experimento

- Campo implícito/marching permite fusão geral, mas demanda resolução/adaptação
  de malha e correspondência; rametas finas podem desaparecer ou exceder budgets.
- Conexão explícita de três contornos oferece uma primeira superfície Y com
  fronteiras identificadas e custo limitado. Restringe posições/ângulos dos
  contornos e não resolve árvores inteiras automaticamente.
- Manter tubos sobrepostos e ocultar tampas não cria continuidade topológica.

O primeiro kernel puro escolhe três colares circulares com ID, centro, normal
exterior unitária, radial unitário perpendicular, raio positivo e3..32 amostras.
As amostras são CCW em torno da normal. Cada plano de colar deve ser um plano
estrito de suporte do conjunto: todas as amostras dos outros colares ficam
atrás dele. Isso recusa tangência, sobreposição, ângulo agudo sem espaço e
configurações quase coplanares. O autor pode reposicionar os colares; o kernel
não projeta nem prende pontos silenciosamente.

Um casco convexo liga as amostras; suas três tampas são removidas. Facetas
coplanares são agrupadas e trianguladas deterministicamente. Cada aresta de anel
aparece exatamente uma vez na fronteira, orientada ao contrário da tampa;
arestas internas aparecem duas vezes com sentidos opostos. A superfície deve
ser conexa, sem vértices isolados, ter exatamente essas três fronteiras e
característica de Euler-1. Não basta coincidir posições de malhas separadas.

Refinamento divide apenas arestas internas com o Plan conformante existente;
nunca subdivide as fronteiras. Relaxamento opcional move somente vértices
internos por média de vizinhos, em passos limitados. Manter orientação local,
volume fechado com tampas auxiliares positivo e perda relativa configurada
limita encolhimento; reportar volume antes/depois e deslocamento máximo.
Não constitui prova geral de ausência de autointerseção após relaxamento.

Saída: Geometry, três tabelas ID→índices do colar e diagnósticos. Vértices de
colar ficam no prefixo original e suas posições são preservadas exatamente;
novos vértices internos não têm correspondência de superfície anterior.
O sweep adjacente precisa reutilizar os mesmos índices, não duplicar posições,
para haver continuidade e compartilhamento de normais.

## Fronteiras do recorte

O primeiro kernel foi entregue sem Scene/Command/versão/renderer novos. A
consulta MCP pura foi acrescentada no segundo incremento, conforme seção
abaixo, sem alterar formatos. O kernel não implementa sweep completo,
árvore procedural, weld arbitrário, boolean CAD, preservação de rig/escultura
sobre o miolo novo ou realismo por si só. A integração posterior deve separar
atributos autorais das faces novas, validar custos agregados e publicar via
MCP/dispatcher. Não tocar o contrato de plantas enquanto correspondência não
estiver explicitamente definida.

Mantidos os limites4096v/8192t por asset,16assets/32instâncias e32768t desenhados.
Na árvore atual,2100 folhas custam25200t e solo48t: sobram7520t de madeira,
com o limite4096v ainda aplicável. O kernel deve recusar extrapolação;
refinamento não autoriza ampliar os limites nem esconder rametas perdidas.

## Checklist estático anterior

1. Operação eventual de autoria, fora do tick e render.2. Entradas finitas,
no máximo96 amostras iniciais; saída/temporários limitados pelas cotas de malha.
3. f64 na construção/classificação, f32 na Geometry; sem dependência/unsafe.
4. Hull por enumeração de triplas e classificação de pontos (pior caso O(n^4)),
limitado explicitamente a96 amostras; não usar esse método para tronco inteiro.
5. Até3 refinos internos e8 passos de relaxamento; Plan faz preflight antes de
alocar saída.6. Nenhuma GPU/I/O no kernel; captura explícita no experimento.
7. Função pura sem threads/locks.8. IDs de contorno não são IDs de faces novas;
nenhum cache ou transferência por proximidade.9. Testar manifold orientado,
fronteiras exatas, refino/relaxamento, degeneração e erro sem mutação; regressões
Rust/Clippy/MCP apropriadas.10. Restrições de colar e custo superlinear são
tradeoffs conscientes deste primeiro experimento, sem alegação de ganho de FPS.

## Primeiro experimento: resultado e reprovação de forma

`organic_junction_probe` monta sweeps adjacentes reutilizando os MESMOS índices
dos colares. O controle tem3 componentes fechados sobrepostos; a montagem nova
tem1 componente fechado, sem arestas de fronteira e Euler2. O patch isolado
mantém3 fronteiras/36 arestas, Euler-1,231v/428t após2 refinos. Perda de volume
0.688%, deslocamento interno máximo0.0912m. `demo_organic_junction.py` publica
ambos via MCP e captura before/after com mesma câmera/luz; undo/redo e replay
com PNG idêntico foram aprovados.

**A forma foi reprovada para integração na árvore.** O casco convexo preenche
excessivamente a bifurcação e cria uma ponte larga/achatada entre os filhos.
O experimento comprova continuidade topológica; não comprova crotch natural,
realismo, fidelidade da madeira central ou preservação autoral no miolo.
Suavizar normais não remove essa forma excessivamente cheia.

Construção do hull usa coordenadas canônicas f64, ortonormalizando o frame dentro
da tolerância de admissão1e-5; a Geometry é quantizada somente ao sair. Isso
corrigiu agrupamentos ambíguos de facetas no extremo32 amostras. Há recusa
explícita quando raio/localização não têm resolução suficiente em f32.

`organic_tree_case` registra uma sonda tangente local do primeiro ramo primário
da fonte17fb9ed (attachment_t0.34), com direções/raios derivados dos mesmos
controles. Não é recorte exato de malha. A configuração próxima/aguda deve ser
recusada; a sonda não reposiciona tronco/galho para passar no kernel.

Reprodução, em paths correntes:

```sh
cargo run --locked --release --example organic_junction_probe > artifacts/organic-probe.json
python3 scripts/demo_organic_junction.py
cargo run --locked --release --example organic_tree_case > artifacts/organic-tree-case.json
```

O kernel continua sem ligação com Scene/Command, sem rig/material no miolo.
A próxima hipótese de forma preserva essa primitiva restrita e a topologia,
mas molda o interior por um campo de união de três tubos afunilados com centro
comum explícito. Projeção radial limitada por bisseção, colares/faixa cilíndrica
fixos, resíduos e envelopes de deslocamento/volume/ângulo são requisitos da
experiência; não há novo contrato aprovado de preservação, árvore ou materiais.

## Hipótese de forma: campo com origem comum (antes do segundo experimento)

A topologia do primeiro kernel é mantida. Uma variante de forma aceita centro,
raio nesse centro, largura de fusão, faixa de guarda, tolerância e envelopes
explícitos. Três campos de tubo afunilado partem do mesmo centro até os colares;
um mínimo suave polinomial combina-os. Isto é campo escalar em metros, não um
SDF exato nem marching/nearest-surface.

Construir primeiro os três colares internos deslocados para dentro pela faixa
de guarda, no mesmo raio/frame, e ligar aos colares externos por tiras com
índices compartilhados. Colares externos e internos permanecem fixos; refinar
somente o miolo antes de acrescentar as faixas evita T-junctions nas bordas.
A superfície inicial e a saída devem ser estreladas em torno do centro explícito.

Cada vértice livre conserva sua direção radial e encontra o zero do campo por
40 passos de bisseção limitada. Há um centro comum interior; cada tubo afunilado
é negativo antes de sua saída radial e positivo depois. Na região onde todos
os campos são positivos, o mínimo suave é monótono. Essa propriedade permite
buscar a saída radial sem trocar de lado nem escolher uma superfície próxima.
Não transforma faces novas em correspondências com um asset autoral anterior.

Conservar bordas/faixas e orientação radial/local, medir resíduo do campo após
quantização f32, rotação máxima de faces, deslocamento e volume. Recusar zero
fora do domínio dos colares, tolerância impossível, inversão ou envelope excedido.
O limite de perda de volume desta variante é explícito e distinto do relaxamento
conservador anterior, pois remover a ponte convexa é parte intencional da forma.

Custos previstos: até2271v/4448t no máximo de amostragem/refino, sem elevar cotas;
40 avaliações×3tubos por vértice livre, apenas na autoria. Sem dependência,
Scene/Command/versão, renderer, cache, thread ou I/O novos. Comparar o MESMO fork
com controle sobreposto e casco convexo; só considerar integração após aprovação
da forma. O caso real próximo/agudo pode continuar recusado sem reautoria.

## Forma ajustada e consulta MCP

`fused_junction` preserva o primeiro kernel `junction`. Aceita `Fusion` com
centro/raio de origem, blend_radius, guard_length, max_displacement, tolerance,
max_volume_loss e max_face_rotation_degrees explícitos. A faixa de guarda é
acrescentada APÓS refinar o miolo, com índices compartilhados e dois anéis fixos
por colar. Não há recorte/transferência de uma malha autoral anterior.

Na mesma prova, o campo removeu a ponte e produziu um vale entre os galhos:
patch267v/500t,72 vértices fixos, resíduo2.38e-8, deslocamento máximo0.418m,
volume2.11024→1.50031 (perda28.90%, limite explícito50%) e rotação máxima de face
73.19° (limite80°). Esses envelopes não são aproximação de fidelidade autoral;
a remoção de volume é uma mudança deliberada de forma. Montagem mantém um
componente, malha fechada/Euler2 e faces orientadas radialmente; o exporter
confere soma de ângulos sólidos4π/grau1 no fork de referência. Isso não é um
serviço genérico de colisão entre meshes arbitrárias.

As capturas before/topology-only/after/back com câmera/luz comparáveis são
produzidas por demo_organic_junction.py. A avaliação da forma isolada avançou;
ainda não comprova realismo ou integração da árvore inteira.

`preview_mesh_junction` é uma consulta pública do Command/dispatcher/MCP:

- `collars`: exatamente três Collars, com IDs/frame/raio/amostras explícitos;
- `options`: opcional, padrões do kernel de topologia (refinement_levels1);
- `fusion`: ausente/null escolhe apenas a primitiva topológica; objeto escolhe
  o campo, aceitando parâmetros parciais com defaults explícitos em metros.

O retorno é Junction (`geometry`, `boundaries`, `collar_vertices`, `report`).
Não insere asset, altera Scene, pausa/avança runtime nem cria histórico.
expected_revision continua recusando consulta obsoleta. Tipos e desconhecidos
são validados por serde/JsonSchema e pelo kernel; falha não publica resultado
parcial. `put_asset` existente publica a montagem feita conscientemente pelo
autor. A operação não muda formatos de Scene/replay, nem adiciona UI exclusiva.
A consulta pode bloquear o dispatcher enquanto faz o cálculo limitado; não é
cooking assíncrono e não tem cache oculto.

## Proposta explícita da primeira bifurcação da árvore

`propose_tree_junction.py` deriva a fonte17fb9ed/seed20261026. Substitui apenas o
tronco entre parâmetros0.24 e0.42, antes do próximo fork0.44; o encaixe baixo do
primeiro ramo é movido para fora/mais baixo. Uma nova curva de pescoço liga o
colar lateral ao trecho ORIGINAL a partir do parâmetro0.30, com direção tangente
coerente. Split de Bézier conserva o restante da curva exatamente na matemática.
Todos os seis anchors de copa em parâmetros0.30..1 são conferidos contra a
fonte (erro máximo9.42e-16m antes de quantizar); não se promete preservar IDs de
vértices/faces da madeira ou mapear escultura/rig inexistentes naquele asset.

A consulta MCP calcula a proposta sem modificar cena e o resultado é comparado
exatamente à chamada direta da biblioteca. Saída em
artifacts/tree-junction-proposal/proposal.json: source_curve, new_neck,
retained_curve, parâmetros dos colares, anchors protegidos, decisões e Junction.
Proposta passou com267v/500t,36 bordas/72 fixos, perda de volume10.42%,
deslocamento máximo0.161m, rotação61.98° e resíduo5.22e-8. A configuração original
aguda22.88° continua recusada; não é reinterpretada silenciosamente.

```sh
cargo build --locked --release --example organic_fused_request
python3 scripts/propose_tree_junction.py
python3 scripts/run_verifications.py --only mesh_organic
```

A montagem central deve usar as boundaries da saída como os índices reais dos
sweeps e declarar novo material no miolo. O tronco atual36 amostras exige reautoria
explícita compatível com máximo32 por colar; transições aos anéis adjacentes e
os atributos de madeira não são solda por proximidade. Aprovação/integração na
árvore, preservação de atributos no interior e aceite humano continuam separados.

Amostragem para integração:32 por colar/refinement1 passou na MESMA proposta,
com291v/488t,96arestas de fronteira e192v fixos, perda de volume11.5275%,
deslocamento0.16186m, rotação64.14° e resíduo5.50e-8. É o default atual do
helper; `--samples 12 --refinement 2` reproduz a prova anterior267v/500t.
Ambas conservam os anchors altos. O resultado atual do helper vem da consulta
MCP e é comparado à biblioteca; não há falso ganho de densidade ou ampliação
implícita de4096v/asset. Colares32 exigem reautoria declarada do tronco36 ou uma
faixa de transição por parâmetro, mantendo índices reais da saída.

### Comparação adicional para a montagem da árvore

Mantidos os mesmos colares/anchors,32 amostras e2 refinos:

| Receita (blend/raio central, m) | Resultado | Custo e envelope |
|---|---|---|
|0.04/0.29|Admitida|687v/1280t, perda12.22%, rotação66.87°|
|0.08/0.29|Recusada|Centro+fusão+guarda não cabem no domínio dos colares|
|0.06/0.29|Admitida|687v/1280t, perda11.02%, rotação61.86°|
|0.08/0.27|Admitida com mudança EXPLÍCITA de raio central|687v/1280t, perda14.45%, rotação63.03°|

Todos os admitidos mantêm96 bordas/192 vértices fixos e resíduo<5.7e-8. O custo
adicional de2 refinos é396v, a ser realocado explicitamente dentro do limite
4096v/asset; não houve mudança de constantes. Menor rotação é diagnóstico,
não aceite visual automático. Comparação em sampling-comparison.json; a frente
central deve comparar material/luz e decidir a receita antes de publicar.

O helper aceita --blend-radius, --center-radius e --output. Para comparar sem
substituir a proposta base que outra frente está usando:

```sh
python3 scripts/propose_tree_junction.py --refinement 2 --blend-radius 0.06 --output artifacts/tree-junction-proposal/comparison-proposal.json
```

Essa receita também foi comparada exatamente entre biblioteca e MCP, com todos
os anchors protegidos. A faixa32→12/36 adjacente deve ser costurada por parâmetro
angular explicitado ou reautorada; coincidência geométrica não é solda por índice.
