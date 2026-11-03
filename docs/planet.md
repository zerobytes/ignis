# Mundo planetário — EAF-199

Continuação: [integração do runtime e residência](planet-runtime.md), em andamento.

Atualização03/11: [horizonte terrestre e coordenadas globais](planet-earth-horizon.md)
registra a câmera de referência da esfera inteira, vistas em várias altitudes e a
prova de horizonte com16tiles nativos. A apresentação de referência não herda os
limites locais10km/30km; integração de origem/física continua separada.

Direção de Guilherme em02/11/2026: modalidade finita e esférica, diâmetro
configurável, curvatura real e possibilidade de dar a volta. O mundo aberto
continua outra modalidade. Galáxias e zoom-out de escala arbitrária são contratos
posteriores.

## Primeiro incremento: fonte e geometria nativas

`src/planet.rs` e `src/planet/mesh.rs` implementam fonte planetária1 independente
do mundo residente, sem alterar o renderer. `diameter_meters` define o diâmetro
da **esfera de referência**, em metros, antes do relevo. Raio=D/2,
circunferência=πD e área de referência=πD². O relevo é deslocamento radial;
a superfície real pode ter diâmetros locais e distâncias maiores.

O arquivo de exemplo [planet.recipe.json](../examples/planet.recipe.json) tem
D=800m para evidenciar a curvatura. A mesma API aceita a fonteD=12.742.000m;
isso não significa publicar todo esse planeta de uma vez na cena física.
O intervalo numérico inicial é10..10¹²m, não uma afirmação de escala renderizável
ou simulação astronômica. `center` é Position1 canônica, com células inteiras e
offsetf64; centros globais nunca são convertidos integralmente parafloat.

Configuração explícita: versão, versão espacial, id, revisão, centro,
`diameter_meters`, `surface_gravity_mps2`, `base_height_meters` e operadores.
`surface_gravity_mps2` especifica a intensidade na esfera de referência,
independente do diâmetro; a consulta retorna o vetor radial com intensidade
`g(R/r)²`. É campo calculado, **não gravidade aplicada ao runtime**.

Operadores compartilhados por autoria fixa e procedural:

- `noise`: value noise3D seeded em coordenadas cartesianas relativas ao planeta,
  escala física, amplitude,1..8oitavas e persistência. Cada oitava é normalizada
  pela soma de pesos; não muda seed porface/tile.
- `raise`: elevação/depressão orientada por direção, raio angular e altura,
  com suporte compacto `(1-t²)²` até a borda do raio.

Ausência de operadores produz esfera uniforme; receitas manuais e geração usam
o mesmo compilador. Esta fonte não contém biomas, drenagem, erosão, cavernas ou
campos de altura arbitrários importados. A soma dos módulos das amplitudes e
altura base é limitada a25%R; é uma guarda numérica inicial do campo radial.

## Continuidade e publicação

A superfície é particionada em seis faces de cubo normalizadas para a esfera.
Uma grade inteira comum identifica arestas/cantos; a publicação `globe` usa um
único vértice por endereço compartilhado. ComN subdivisões há6N²+2vértices e
12N²triângulos, todos orientados para fora. Os polos são pontos ordinários,
sem longitude singular no armazenamento. Ruído e elevações dependem da direção
3D global, nunca de coordenadas independentes porface.

`tile` permite uma face, nível0..24 e endereço[x,y] dentro de2^nível. Subdivisões
são potências de dois1..128. Tiles vizinhos de mesma resolução/amostragem têm
posições de borda coincidentes; normais porasset e costura de resoluções distintas
**não estão certificadas**. Não há gerenciador de residência, troca automática
pai/filhos, descarregamento ou LOD novo.

Projeções explícitas:

- `centered`: centro do planeta na origem local, eixos planetários preservados.
- `surface`: origem numa pose radial; eixos locaisX=direita,Y=para fora,
  −Z=frente. Todos os assets da mesma cena devem usar a mesma projeção;
  isso é responsabilidade explícita do autor, não um rebase automático.

Perto da superfície a sagita é calculada por
`y=-R|n-up|²/2+h·dot(n,up)-altitude`, evitando subtrair dois raios grandes
para obter uma altura pequena. A transformação só vira f32 no final e recusa
componente fora de±10000m ou erro de arredondamento acima de1mm. Um planeta
terrestre inteiro não cabe nessa projeção; um trecho local cabe. A altura
retornada pela fonte é analítica, mas os triângulos aproximam essa superfície;
**não há certificado de erro entre amostras**, nem detecção automática de
frequências subamostradas.

`PublishPlanet` compila nativamente e aplica asset+instância+colisão opcional numa
única transação existente, exigindo pausa. Falha conserva cena/revisão/física;
undo/redo usam os mesmos Arcs. `collision:true` publica a mesma malha completa
como superfície física estática; não ativa gravidade radial, orientação de atores
ou navegação planetária. A publicação substitui os IDs explicitamente fornecidos,
sem remover outros assets/instâncias. Fonte autoral fica no `.ignis-planet`;
a Scene guarda o bake, sem vínculo/regeneração automáticos neste incremento.

## Percurso e horizonte

`Pose` contém direção radial, frente tangente e altitude sobre a esfera de
referência. Frente e direita são transportadas geometricamente com a posição.
Não existe salto de longitude nos polos. Para distância `s` na referência,
`θ=s/R`, `n'=n cosθ+f sinθ`, `f'=f cosθ-n sinθ`; a operação aceita avanço e
retorno, inclusive voltas completas. Altitude é preservada, não amostrada do
relevo. A distância também não inclui o comprimento adicional das montanhas.

`QueryPlanet.travel` é uma consulta geométrica, não move um Player/NPC, não
consulta obstáculos e não mede caminho físico executado. O retorno declara
`geometric_only:true` e `collision_checked:false`.

`horizon` usa a esfera de referência sem relevo/refração:
linha tangente `sqrt(h(2R+h))`, ângulo de depressão `atan2(tangente,R)` e
arco de superfície `R·ângulo`. Não descarta objetos do renderer nem certifica
visibilidade em terreno real. A esfera publicada mostra curvatura geométrica
no renderer existente, sem efeito de tela para fingir arredondamento.

## MCP, formato e compatibilidade

Controles nativos disponíveis antes de nova interface visual:

- `query_planet {query:{kind:inspect|sample|travel|horizon|inspect_mesh,...}}`.
- `publish_planet {publication:{document,area,frame,asset,instance,color,material?,collision?}}`.
- `save_planet {path,document}` / `load_planet {path}`.

`inspect` informa dimensões e a fronteira `runtime_world_mode:false`.
`sample` aceita até4096direções por pedido e devolve altura, posição canônica e
vetor gravitacional calculado. Direções finitas com comprimento0,5..2 são
normalizadas; frente precisa ter componente tangente não nula.

Fonte `.ignis-planet`: `IGNISPLN`8B+versãoLEu16+tamanhoLEu64+SHA25632B+CBOR
tipado. Helpers binários compartilhados com terreno, limite8MiB/profundidade64,
EOF exato e escrita temporária+fsync+replace pelo Service. Position em binário
usa48bytesLE do codec já existente; no MCP preserva células como strings.
Load valida e retorna autoria, sem trocar o mundo. Caminhos permanecem confinados
ao projeto. Scene109/replay119/conteúdo7/protocolo5/checkpoint3/PSV1 preservados:
nenhuma nova primária de simulação foi introduzida. Os bakes são meshes existentes.

## Revisão estática de performance

1. **Há trabalho novo por tick/frame no mundo atual?** Não. Operações são
   consultas/publicações explícitas; física, sensores e renderização não ganharam
   varreduras ou branches planetários porframe.
2. **JSON participa da matemática?** Não. Document/Pose/Sample/Prepared e malha
   são tipados; serde só entra nas fronteiras MCP/persistência/relatórios.
3. **Invariantes são repetidas porvértice?** Coeficientes/normalizações/direções
   dos operadores são preparados uma vez emarray inline de32entradas. `acos`
   só é avaliado dentro do suporte de elevações. Não há clone de documento,
   arquivo, alocação ou validação completa poramostra.
4. **Memória escala com a área do planeta?** Não. Publicação é limitada ao patch
   pedido: até98306v/196608tri no globo,16641v/32768tri portile. BTreeMap de
   soldagem e grid temporário existem só no bake; buffers reservados previamente.
   Não converter isso em promessa de custo para toda a cena ou futura residência.
5. **Trabalho é limitado antes das alocações grandes?** Sim: operações≤32,
   resolução/nível/endereço e pontos×custo≤16M. O limite de trabalho conta oito
   vértices do lattice poroitava. Endereços de fonte validam overflow antes do
   bake; projeção também pode rejeitar a candidata durante a geração.
6. **Há perdas silenciosas de precisão?** Globalf64+Position1, projeçãof32 com
   guarda1mm e±10km; não ampliar envelopes existentes nem escalar o planeta
   secretamente para caber. Se a projeção não serve, falha explícita.
7. **Mudou dependência/thread/lock/rede?** Não. Modo offline preservado.
   Nenhum socket, dispositivo de áudio ou janela é aberto pelos verificadores.
8. **O que protegem os testes novos?** Três provas agregadas: corrupção/identidade
   da fonte e transporte fechado; costuras/manifold/orientação/precisão terrestre;
   publicação atômica/undo/bake. Gates existentes selecionados porimpacto.

## Próximos contratos para caminhar ao redor

A modalidade está iniciada na fonte/geometria, **não está ligada ao runtime
planetário de gameplay**. Integração seguinte precisa publicar configuração
primária de mundo e remover pressupostosY/XZ em conjunto: gravidade porcorpo,
KCC/up/rotação/apoio, entrada e câmera1ª/3ªpessoa com referencial transportado,
apresentação física e spawn. Depois: NPC/sensores/navegação, efeitos/projéteis,
persistência/replicação desse estado e residência com mudança de origem.

Evitar integrar somente câmera curva mantendo colisão/gravidade planas, ou
chamar uma consulta de posição de caminhada aceita. A migração exige revisão
própria de Scene/replay/checkpoint/rede e aceitação física explícita. O Rapier
oferece direção `up` no controlador e exige que o chamador componha gravidade no
movimento ([documentação oficial](https://rapier.rs/docs/user_guides/templates/character_controller/));
isso não adapta automaticamente os planejadores da Ignis. A separação de
coordenadas globais e projeção local evita imporfloat planetário à simulação;
tradeoff semelhante é descrito na [documentação de mundos grandes do Godot](https://docs.godotengine.org/en/stable/tutorials/physics/large_world_coordinates.html).

## Reprodução

```sh
cargo test --offline --test planet --test world_space --test terrain --test terrain_region --test terrain_collision
cargo build --offline --release --bin ignis
python3 scripts/verify_planet.py
```

O verificador usa MCPstdio sem janela, fonte800m, órbita, horizonte em2/60m,
128passos de uma volta geométrica e tile de planeta terrestre. Artefatos correntes
em `artifacts/planet/`; não modifica o mundo aprovado em expanded-world nem
abre/apaga janelas humanas. Validação automática e aceite visual humano são
registrados separadamente.

## Evidência deste incremento

12testes Rust distintos/5alvos aprovados (planet3, terrain3, terrain_region3,
terrain_collision2, world_space1), com repetição dos três novos após a revisão do
sampler. Clippy all-targets `-D warnings`, compilação e verificações de formato/
Python/diff aprovadas. Não foi executada a suíte global de cenários.

MCP4grupos: fonte/consultas sem mutação;128passos geométricos por2.513,274122872m
fecham com erro4,44e-14m; publicação/falha/undo/replay/saveportátil; projeção local
terrestre e capturas. GloboN64=24578v/49152tri; a superfície de água de referência
é outra esfera, só visual. Patch terrestreN64/nível12 tem4225v/8192tri e maior
arredondamento0,000122061320326m. Restore da órbita produz PNG byte exato.

Capturas inspecionadas: órbita, horizonte2m e trecho terrestre. O trecho isolado
mostra sua borda externa por não possuir vizinhos residentes; não representa
horizonte global nem streaming. A composição água+terra pode apresentar serrilhado/
instabilidade de profundidade onde as superfícies quase coincidem; não houve
mudança de renderer para ocultar isso. A captura de terreno sem água separa a
geometria de relevo dessa apresentação. Nenhum aceite artístico humano, caminhada
planetária, tempo de frame/FPS ou mundo completo em escala terrestre é inferido.

Build final macOSarm64 release: `97f2ef67d592a83a` (rustc1.97.1); fingerprint
conferido contra os fontes usados na última rodada MCP. `verification.json`
guarda o identificador completo. `orbit-terrain.png` mostra o relevo sem água;
`horizon-eye.png` mostra a curvatura com observador2m acima da referência.
