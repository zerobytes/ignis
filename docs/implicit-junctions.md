# Ensaio de junção por volume implícito — EAF-85

## Decisão antes do código

Repro congelado de junction-research, sem importar Scene/trajectory/solvers da
frente de plantas. Ajustes de energia livre minimizam um escalar sem garantir
embedding; as tentativas anteriores foram reprovadas geometricamente.
Alternativas: outro regularizador precisa provar que evita as mesmas dobras;
lofts por pares preservam contornos mas já falharam no encontro agudo; união
implícita dá uma forma alvo explícita e exige extração/trim/conexão próprios.
Testar esta última em domínio limitado antes de estabilizar API. Não é boolean
arbitrário nem reconstrução de qualquer malha. Não reutilizar APIs fracassadas.

Alvo da hipótese: três portas reais orientadas, hub e blend explícitos, três
guias tubulares em repouso; união suave e extração limitada. Conectar a região
nova às portas sem deslocar pontos/canais antigos ou inventar identidade para
miolo. Rig inicial restrito a uma raiz explícita nas bordas; demais pesos fora
do recorte e clipes preservados. Rejeitar mistura não atendida, dobras>=90°,
degeneração/topologia fora do domínio ou excesso de orçamento. Não prometer
prova global de self-intersection por inspeção local.

## Checklist estático anterior

1. Autoria/query pura, nenhum trabalho por tick.2. Grade/segmentos/extrator com
orçamento explícito antes das alocações; falha não publica.3. f64 temporário,
f32 final, índices/Asset existentes; nenhuma dependência/unsafe/ABI novo.
4. Avaliação finita de segmentos e células, caches por aresta na autoria;
sem nearest-source ou identidade inferida.5. Manter4096v/8192t por asset e65536t
em desenho nesta branch. Repro denso pode ser verificado isoladamente, sem
alegar integração na árvore inteira.6. Renderer/layout/cache inalterados;
mais geometria usa preparação/upload existentes.7. Sem worker/lock novo.
8. Query nova só se a prova nativa passar; geometria existente persiste sem
novo Scene/replay.9. Validar topologia/orientação/bordas/raiz/canais/limites,
pureza, render nativo, erros e regressão antes de commit.10. Custos mantidos:
extração volumétrica e aproximação finita; recorte/conexão são gates separados.
Nenhum ganho de FPS presumido.

## Contrato estabilizado da base

`mesh_junction_union::fit(&Asset,&[Vec<u32>;3],Option<&str>,&Options)` e query
pura `preview_mesh_junction_union {source,ports,root,options}`. Input é Asset
já recortado, com exatamente três ciclos reais no sentido dos triângulos da
fonte,4..32pontos cada. A query não edita cena/simulação/histórico. Resultado
`{asset,report}` exige publicação explícita por put_asset/dispatcher, onde o
orçamento global de documento/desenho continua sendo validado.

Options físicos explícitos: hub/hub_radius, blend, cell_size, guard_length,
collar_length, relaxation_steps e material_offset. Hub deve ficar no lado
interno de cada porta orientada, sem exigir que planos de cada porta suportem
TODAS as outras. Portas aproximadamente circulares/planares: desvio radial
até25% e de plano até20% do raio, além da admissão geométrica final. Não é
boolean arbitrário ou suporte a qualquer seção/ângulo.

Guias cúbicas do hub para centros/normais das portas,25segmentos por braço,
raio Hermite e extensão terminal. Campo é mínimo dos segmentos por braço e
união polinomial suave entre braços. Grade regular com teto65536pontos;
extração por6tetraedros consistentes/célula, cache de arestas, teto intermediário
16384v32768t. Classificação do tetra usa arrays na stack, não Vec por amostra.
Redistribuição limitada e projeção ao MESMO campo, quatro passos por iteração,
0..8iterações. Orientação contra gradiente/erro conferida nos pontos quantizados
f32. Trim local por três esferas exteriores abre as portas; exigir componente
único com3bordas eχ=-1. Domínio que falhar é recusado.

Uma faixa fonte tangente usa bissetriz de arestas unitárias e normal média de
área; seus índices novos são adicionados depois do miolo extraído. Conexão
zipper por programação dinâmica preserva contagens desiguais, sem subdividir
faces antigas ou mover/soldar por epsilon. Oito fases geometricamente próximas
são o máximo; custo de condicionamento/normal e costura conservadora60°.
Resultado inteiro deve ter orientação coerente,0bordas/0não-manifold/nenhum
isolado e ângulos estritamente abaixo90° em toda adjacência envolvendo faces
novas. Não equivale a acabamento artístico aprovado nem prova global contra
self-intersection com outras regiões.

Prefixo inteiro de posições/triângulos/coordenadas/pesos exato. Ossos/clipes/
material/shading preservados. Miolo/guardas são novos, domínio explícito
posição_local+material_offset. Não alegar transporte de fibras/detalhes para
miolo sem correspondência. Rig opcional: root nominal deve ser uma raiz, todas
influências positivas dos vértices de porta devem apontar para ela. Outros
pesos fora das bordas permanecem. Novos pontos recebem apenas essa raiz;
multirig nas portas é recusado, sem truncar influências.

## Evidência da base e próximos gates

Repro do tronco/ramo nativo, source preparado2800v5506t/portas32-24-32, guarda
socket030. Campo9044pontos; extraído1107v2099t; resultado3995v7986t. Prefixo
retido exato. Máximo total59.7362°, média5.01344°; costura na FONTE1.15059°,
conexão interna59.7362°, interior53.5986°. Erro máximo do campo após f32
5.83302e-8m. A linha de colar ainda é perceptível no neutro; não promover como
solução artística final ou árvore realista. Capturas nativas authored/neutral
sem sombras em front/profile/back e testes de animação/save/replay separados.

Candidato próximo: suavizar somente pontos novos próximos à conexão interna,
preservando prefixo fonte e primeira guarda tangente. Partir de superfície já
válida, medir ângulos e deslocamento/volume, manter guardas e limites. Isso não
é permissão para introduzir um solver reprovado ou esconder vinco com material.
Fonte da árvore inteira não foi alterada nesta unidade; composição regenerável
com fonte de contexto+planta pertence à frente de plantas. Nenhum Scene/Replay
novo ou aumento de drawbudget aqui.

## Próximo incremento: checklist antes do código

Ensaio finito: relaxamento local de pontos extraídos até três anéis do zipper,
fonte e primeira guarda geométrica fixas, passos0/1/2/4/8 comparados. Alternativa
de mais faixas custa vértices e não corrige sozinha a direção das faces; começar
sem adicionar geometria. Medir deslocamento máximo, volume assinado e ângulos
após f32; recusar inversão local/degeneração/guarda90 e não promover se piorar
o pico/média da base. Não é garantia global de embedding.

Material é gate separado: primeira guarda copia domínio correspondente da fonte;
nos demais pontos novos, relaxamento de deslocamento do domínio
em relação a posição_local+offset, com condições fixas explícitas nas guardas.
Não alterar prefixo nem alegar transporte físico de fibras. Desativado por
padrão para conservar chamadas existentes; parâmetros novos serde default e
relatório Clone compatíveis com a composição em andamento.

Checklist:1.autoria/query pura;2.adjacência e buffers limitados por4096/8192,
reusados entre passos;3.f64 temporário/f32 final e prefixo exato;4.no máximo8
passos geométricos/64de domínio, sem solve aberto;5.envelope local explícito e
validação antes de retornar;6.sem GPU/layout novo;7.sem thread/lock/dependência;
8.publicação/invalidação existentes;9.provar opção0 exata, fixos, erros,
domínio/rig/socket/replay e neutro;10.custos finitos de autoria conscientemente
mantidos, sem FPS nem conservação global de volume presumida.

## Contrato do refinamento opcional

Options acrescenta `seam_relaxation_steps` (0..8),
`seam_max_displacement` e `material_relaxation_steps` (0..64), todos default0.
Chamadas anteriores conservam a geometria e os canais. Com passos geométricos,
o envelope precisa ser positivo e não pode exceder cell_size/2 ou collar_length.
Somente os pontos extraídos no zipper e seus dois anéis vizinhos se movem;
passo Jacobi0.3 e fatores1/0.65/0.25. Fonte, primeira guarda e núcleo fora da
seleção são fixos. Cada passo quantiza f32, limita deslocamento total e recusa
inversão/degeneração contra as faces iniciais. Resultado mantém a guarda90 e
recusa piora do pico OU da média angular frente à base;8passos do fixture são
recusados, não automaticamente promovidos. Tolerância do envelope1e-6m cobre
somente quantização, sem soldar/mover fronteiras. Volume assinado antes/depois
e deslocamento real são reportados; não há promessa de conservação local.

Com passos de material, um canal explícito existente é obrigatório. Guardas
copiam coordenadas dos vértices fonte correspondentes por índice. A malha com
índices compartilhados já interpolava o canal continuamente; o defeito era sua
grande variação ao longo da faixa curta, não uma abertura C0. A cópia elimina
essa diferença na primeira faixa, sem garantir derivada ou fibras físicas. Nos pontos iso,
resolve-se somente a CORREÇÃO do domínio sobre posição+offset: Laplaciano
uniforme de adjacência, guardas como Dirichlet, CG determinístico limitado ao
número solicitado (máximo64). Nenhuma posição/normal/peso depende desse solve.
Condições fixas ficam fora das incógnitas, inclusive com RHSzero. O relatório
recalcula resíduo no domínio quantizado; não confundir quantidade de passos
solicitada com certificação de convergência. Sem ajuste de receita/semente ou
mudança de material para esconder a geometria.

`max_field_error` mantém o erro da extração ANTES do refinamento local;
`max_field_error_after_seam` mede os pontos iso finais no mesmo campo. O
refinamento sai deliberadamente da iso-superfície dentro do envelope, portanto
o primeiro erro não descreve a superfície final. `refinement` traz contagem
selecionada, deslocamento, volume, salto material nas guardas e resíduo final.
Esse salto é distância entre coordenadas de dois pontos distintos ligados pela
faixa, não descontinuidade de valores sobre uma aresta compartilhada.
Opção0 é a referência; vistas neutras e com material devem ser comparadas em
câmeras/luz/tick iguais. O colar artístico permanece um gate aberto.



Na integração local66 de plantas, o teto por asset passa8192v16384t e o contorno
extraído pode ter até128v (originais continuam4..32). DP33×129,8fases/3portas;
mesma grade65536 e raw16384v32768t. Totais16384v32768t/draw131072 mantidos na
frente integrada. O kernel isolado anterior e seus números acima são históricos;
medidas do estudo fino e limites em plant-assemblies.md.
