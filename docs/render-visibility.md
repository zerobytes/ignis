# Visibilidade genérica antes do desenho — EAF-174

O renderer decide visibilidade por volumes geométricos, sem conhecer espécies,
nomes ou papéis dos assets. A floresta de EAF-173 é uma carga de validação deste
caminho, não uma exceção. Este incremento implementa rejeição fora do frustum e
agrupamento conservador; não implementa oclusão por terreno ou novo LOD regional.

## Contrato

- A câmera e a luz direcional têm máscaras independentes. Uma malha fora da
  câmera continua disponível para projetar sombras quando intersecta o volume
  da luz. Nada é removido da simulação, seleção, persistência ou consultas físicas.
- Cada mesh estática tem bounds cobrindo a fonte e todos os proxies compilados
  válidos. Meshes deformadas usam a pose atual. Caixas/cunhas também passam pelo
  descarte; a cauda de avatares derivados é conservada/rejeitada como um conjunto.
- Bounds são reconstruídos quando muda a geração da geometria/pose ou o pacote
  compilado. Câmera, viewport e luz classificam esses volumes a cada desenho, sem
  varrer os triângulos estáticos a cada frame. Streams externos sem geração
  reconstruem bounds para não assumir um cache válido.
- O teste usa os seis planos homogêneos da projeção real do renderer, incluindo
  near/far. Objetos que cruzam planos são mantidos. Margens incluem a magnitude
  das operações antes do cancelamento de translações; matemática inválida mantém
  geometria, nunca autoriza descarte.
- Instâncias rejeitadas em ambas as vistas não entram nos buffers de instâncias
  nem exigem upload de sua geometria. Desenhos expandidos rejeitados não entram
  no stream filtrado. Caches previamente usados podem permanecer residentes:
  descarte de desenho não é uma política de eviction/streaming.
- Materiais de instâncias descartadas ou substituídas por cor não precisam da
  preparação detalhada. Índices da tabela são preservados, preenchendo slots
  inativos com material padrão; não há renumeração de identidade.

## Agrupamento e ordem

Depois do descarte, instâncias compartilhadas compatíveis podem atravessar até
32 vizinhas para formar um grupo. Isso só ocorre quando os retângulos projetados
conservadores comprovam ausência de cobertura raster comum, tanto na câmera como
na luz. Os retângulos incluem margem de dois pixels e erro de arredondamento;
cruzamento do near plane impede essa prova. Caminhos legados são barreiras.

Sobreposição possível conserva a ordem autoral, incluindo desempates de
profundidade e materiais diferentes. O limite32 é orçamento da busca por
agrupamentos, não limite de objetos. O sistema continua processando todos os
objetos definidos pelo autor dentro dos recursos do dispositivo.

## Observabilidade e controle de referência

Capturas MCP e `inspect_viewport_performance` expõem `adapter.visibility` ou
`sample.renderer.visibility`: instâncias/caixas vistas pela câmera, pela luz,
rejeitadas em ambas e triângulos fonte rejeitados. `instancing` inclui desenhos,
instâncias enviadas e `reordered_instances`. Triângulos fonte rejeitados diferem
nos casos de LOD dos triângulos efetivamente economizados no desenho.

A otimização de visibilidade é automática, inclusive sem atmosfera.
`Renderer::set_visibility_culling(false)` é um controle de diagnóstico Rust que
mantém os demais caminhos iguais e desliga descarte e reagrupamento. O checkbox
atual **Otimização** controla `atmosphere.reduce_details`; ele não é um interruptor
universal do renderer. Não confundir esse checkbox com o oráculo deste ensaio.
Nenhum formato Scene/replay/rede foi alterado.

## Validação reproduzível

`cargo run --release --example visibility_cost` lê a mesma cena de floresta e
compara referência/otimizado a2048×1536 (dentro do limite da captura nativa), em
vistas próxima, panorâmica e inversa. Compara RGB8 exatamente antes de cronometrar;
três rodadas alternadas, oito aquecimentos e16 amostras por variante. Oito poses
adicionais mudam posição e proporção da viewport, novamente com referência exata.
Relatórios e PNGs atuais: `artifacts/visibility/`.

O benchmark separa tempo CPU de submissão e tempo draw+espera de conclusão da
GPU. O primeiro inclui driver; o segundo não é timestamp GPU isolado nem FPS de
editor/gameplay. PNG/readback, apresentação, simulação e compilação ficam fora do
trecho cronometrado. Não converter diretamente esse número em FPS de gameplay.

O teste crítico `visibility_preserves_pixels_across_views_shadows_resize_and_motion`
em `tests/mesh_resources.rs` protege contra geometria/sombras apagadas indevidamente,
cache inválido e mudanças de desempate. Executar os testes GPU explicitamente com
`cargo test --release --test mesh_resources -- --ignored --nocapture`.
Regressões existentes de atmosfera, attachments e apresentação de NPCs complementam
os contratos de pose, material, sombra e persistência.

## Próximos contratos do programa

Oclusão de objetos cobertos por outros objetos é diferente de rejeição fora do
frustum e continua pendente. A granularidade de uma instância visível ainda pode
ser grande. O compilador ATM atual certifica deslocamento geométrico e erro de
substituição atmosférica: seus níveis não são autorização para simplificar
materiais detalhados arbitrários. A seleção regional independente da atmosfera
precisa certificar também aparência, cobertura de folhas/furos e contornos.
Este incremento não usa distância sozinha como autorização para apagar detalhes.

## Resultado local

Apple M4 Pro/Metal, mesma floresta de10.000 árvores, atmosfera e câmera nos dois
caminhos. Mediana das três medianas por vista, a2048×1536:

| Vista | Anterior (ms) | Visibilidade+agrupamento (ms) | Redução de tempo | Triângulos antes → depois |
|---|---:|---:|---:|---:|
| Próxima | 135,212 | 54,591 | 59,6% | 162.545.936 → 51.480.713 |
| Panorâmica | 117,793 | 70,140 | 40,5% | 151.214.298 → 86.796.840 |
| Inversa | 136,928 | 4,239 | 96,9% | 203.041.002 → 24.202 |

Tempo CPU de submissão: 2,392→1,586ms, 2,978→2,289ms e1,916→0,445ms,
respectivamente. Chamadas do caminho instanciado: 3.712→176, 5.149→288 e1.723→1;
essa contagem não inclui o desenho legado do chão, guias ou UI. Na vista próxima,
11.142 componentes são rejeitados em ambas as vistas e8.864 são mantidos na câmera.
Os componentes casca/folhas são contados separadamente, não como árvores inteiras.

RGB8 exatamente igual nas três vistas e nas oito poses móveis com resize.
A referência já usa instancing e o seletor atmosférico anterior: esta tabela mede
somente o ganho adicional, não compara com uma engine sem nenhuma otimização.
A vista próxima ainda exige51,5milhões de triângulos; esse ganho não resolve o
platô da geometria visível nem significa atingir60FPS. O lado inverso barato
confirma o descarte fora da câmera; não prova descarte de objetos oclusos.

Geometria previamente usada permanece no cache (~16,54MB neste ensaio); o novo
caminho reduz submissão/instâncias ativas, não promete liberar toda a memória dos
objetos invisíveis. Dados atuais em cost.json, summary.json e motion.json.

Build validado: `52861a06e61d864c`. Doze testes Rust existentes nos quatro alvos
mesh_resources/atmosphere/mesh_attachments/npc_presentation passaram no código
final, além dos dois testes GPU explícitos (cinco pares de representação e dez
pares de visibilidade). Clippy all-targets com -D warnings, fmt/diff check, build
release e seis grupos MCP de atmosfera passaram. Os128 testes de lib também
passaram durante o desenvolvimento; os gates de pixels finais foram executados
após as margens de precisão e o agrupamento.

O editor anterior já estava fechado. A cena foi reaberta no build corrente com
seu layout/câmera salvos, luz da cena e FPS ativo. A amostra inicial da janela
2085×1560 marcou21,2FPS; a inspeção visual posterior, já com câmera em outra
posição, marcou17,1FPS. São amostras interativas em vistas distintas, não o A/B
controlado da tabela. Evidência de build e desempenho em `artifacts/visibility/live/`.
Aceite humano continua separado; a janela permanece livre para navegação.

## Revisão estática de custo

Bounds, máscaras, retângulos e ordem mantêm vetores reutilizáveis. A montagem de
bounds pode alocar por mudança de geração/pacote, nunca por câmera estática.
A classificação é linear em instâncias; a busca de agrupamento é limitada a32
vizinhas, sem ordenação global por material ou readback de GPU. Não há I/O,
JSON, locks ou geração de árvores no loop de desenho. Restam custos lineares
por instância e a cauda agregada de avatares; hierarquia espacial, oclusão e
seleção regional são próximos contratos, não benefícios implícitos deste patch.
