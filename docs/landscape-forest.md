# Floresta na paisagem nativa — EAF-173

A demonstração usa a paisagem aprovada de cinco cadeias de montanhas, a atmosfera
sem blur e a caixa humana original. Acrescenta exatamente 10.000 árvores visuais
sobre a superfície triangulada do terreno. O teste anterior de quadrados não era
uma demonstração de floresta; seus resultados não representam esta carga.

## Reprodução

Com o build release corrente e a paisagem aprovada disponível:

```sh
python3 scripts/build_landscape_forest.py --base artifacts/landscape-live/landscape.scene.json
target/release/ignis --root . --editor --scene artifacts/landscape-forest/forest.scene.json
```

A receita gera a população deterministicamente (seed 1737231), compila os resumos
distantes pelo dispatcher nativo e salva uma captura de 1728×1117. Os resultados
ficam em `artifacts/landscape-forest/`: `forest.scene.json`, `forest.png`,
`population.json` e `verification.json`. O arquivo base é preservado. A cena
original de 10 mil cabia no antigo limite de documento de 8 MiB. A continuação
com 30 mil abaixo remove essa barreira para cenas locais de autoria.

No editor, desabilitar **Usar luz de trabalho** para usar a iluminação da cena.
Uma luz diferente pode invalidar o certificado de erro dos resumos compilados.
Pelo MCP: `edit_editor`, action `lighting`, light `null`. A câmera inicial está
em [445,920,8280], olhando para [390,880,7780], FOV 60°. Botão direito orbita,
meio ou Shift+direito desloca e roda aproxima. O medidor da viewport mostra FPS;
`inspect_viewport_performance` expõe a mesma informação.

## Conteúdo e memória

- Uma espécie do experimento procedural anterior, com tronco, galhos e folhas
  geométricas; 20.292 triângulos por árvore. Variação de altura (14–27 m), rotação
  e cor; não são espécies ou copas geometricamente diferentes.
- Duas malhas compartilhadas, casca e folhas: 20.000 componentes para 10.000
  árvores. Pares têm posição, rotação e escala idênticas.
- 202.920.000 triângulos lógicos nas árvores; 203.041.002 incluindo a paisagem.
  Instancing evita duplicar a geometria residente, mas não elimina seu desenho.
- Raízes calculadas por interpolação baricêntrica dos triângulos autorais, usando
  a superfície mais alta onde cadeias se sobrepõem, com penetração de 0,55 m.
  Distância mínima entre troncos de 7 m e área livre ao redor da caixa humana.
- População distribuída em bosques, mais densa perto da câmera. Dez mil árvores
  não formam cobertura contínua de toda a extensão quilométrica do cenário.
- Árvores são visuais, sem vento ou colisores novos. Não há novo streaming,
  sistema de espécies ou mudança do renderer nesta entrega.

`examples/assets/forest-tree.bin` é o insumo compacto de build IGFT1: magic de
8 bytes, contagens u32, posições float32 XYZ e faces com três índices u32 e
material u32, tudo little-endian. Tem 531.952 bytes e 17.272 posições após soldar
duplicatas exatas; os 20.292 triângulos originais foram preservados. Proveniência
e hashes estão em `forest-tree.json`. Canais não consumidos pelo renderer nativo
não foram armazenados. O documento autoral gerado usa o formato de cena existente;
IGFT1 não é um novo codec de streaming em runtime.

## Validação e resultado em 31/10/2026

Build existente `efde0093aaf3059b`, Apple M4 Pro/Metal. Geração, compilação,
save/load nativo, captura e inspeção visual aprovados. Contagem dos 10.000 pares
casca/folhas verificada; terreno, entidades e luz preservados por asserções.
Não foi necessário reconstruir a engine nem criar testes de contratos novos.

A cena foi carregada na janela existente do Ignis. Antes da troca, cena e editor
do usuário foram salvos em `artifacts/mesh-resources/live/user.scene.json` e
`user.editor.json`. Estado de câmera/luz entregue em
`artifacts/landscape-forest/forest.editor.json`.

Na amostra da viewport 2085×1560, otimização ligada, luz da cena, sem blur e
present Immediate: **7,36 FPS / 135,96 ms**. É cadência do editor, incluindo UI,
aquisição, submissão e agendamento, não tempo GPU isolado nem FPS de gameplay.
A imagem da janela confirmou árvores reais no primeiro plano e nas encostas.
Aceite humano de aparência e navegação permanece separado.

Essa amostra ainda desenhava 142.306.498 triângulos detalhados e 22.477.473
triângulos baratos, apesar de 5.597 componentes usarem LOD geométrico e 5.977
usarem substituição. A geometria alocada reportada era 16.543.452 bytes, mais
1.600.400 bytes de instâncias e 600.556 bytes compilados (não são memória total
do processo ou da GPU). Evidência completa em `live-verification.json`.

O próximo gargalo demonstrado é reduzir trabalho geométrico da floresta,
inclusive instâncias fora de vista e detalhes distantes, preservando contornos.
O bom resultado do grid de 128 triângulos por objeto não se generaliza para
esta árvore de 20.292 triângulos. Nenhuma meta de FPS foi alcançada por inferência.

## Continuação de custo — EAF-174/175

O descarte conservador de câmera/luz e agrupamento disjunto está em
[render-visibility.md](render-visibility.md). O incremento seguinte mantém os
triângulos selecionados e reduz bytes/trabalho repetido por indexação e atributos
por instância: [small-geometry-cost.md](small-geometry-cost.md). O comparativo
mais recente é offscreen2048×1536, com redução de30–34%nas vistas pesadas e
pixels idênticos. Não substitui a medição histórica de FPS da janela acima.
Cobertura coletiva dos componentes de4–16px e oclusão continuam pendentes;
não interpretar poucas linhas de pixels como permissão de apagar a floresta.


## Densidade triplicada — EAF-176

Após aceite humano do ganho de FPS de EAF-175, a mesma paisagem recebe **30.000
árvores**, em vez de10.000. A receita aceita `--trees` e `--output`:

```sh
python3 scripts/build_landscape_forest.py --trees 30000 --output artifacts/landscape-forest-dense
target/release/ignis --root . --editor --scene artifacts/landscape-forest-dense/forest.scene.json
```

A distribuição original se repete em lotes determinísticos: as10mil primeiras
árvores conservam posição, altura, orientação e cor. As20mil novas ocupam a mesma
paisagem, com o mesmo espaçamento mínimo de7m, altura14–27m e superfície real de
apoio. A densidade aumenta em bosques; não é cobertura uniforme de todo o terreno.
São60mil componentes casca/folhas,60.006instâncias de mesh ao todo e608.760.000
triângulos lógicos nas árvores. Não são608milhões de triângulos residentes nem
necessariamente desenhados; duas malhas de árvore permanecem compartilhadas.
Não há alteração de resolução, tolerância visual, material, luz ou renderer.
Blur permanece ausente. A versão10mil fica disponível para comparação controlada.

### Persistência local sem teto de8MiB para cenas

O catálogo maior expôs o limite antigo do documento JSON. `LoadScene`,
`OpenProjectScene` e `OpenProject` passam a ler a cena local por `BufReader`,
sem criar um vetor adicional com o arquivo inteiro. `SaveScene` escreve JSON
compacto por `BufWriter` em arquivo temporário, faz flush/sync e substituição
atômica; só depois marca a revisão como salva. Não há nova serialização por frame,
novo codec, transporte JSON para GPU, teto de objetos ou mudança de versão.
Os formatos e limites de replay, rede, layouts, manifesto e saves binários
continuam separados. Projetos novos ainda geram cenas iniciais pequenas.

A autoria continua residente em memória e candidata inteira é validada antes
de trocar o mundo. Isso não implementa streaming de mundo nem memória ilimitada:
leitura de uma cena local grande consome os recursos reais correspondentes.
Checagem de caminhos/symlinks, profundidade JSON e validação semântica permanecem.
A prova crítica cobre documento acima8MiB, salvamento/reabertura pelos três
caminhos, rejeição de sufixo inválido sem mutação, erro de escrita sem substituir
estado/arquivo anterior e manutenção do envelope8MiB de replay.

### O que acontece com as árvores distantes

Distância por si só não elimina árvores. Bounds fora do frustum são rejeitados
antes dos desenhos; dentro dele, o seletor certificado pode reduzir a geometria
e substituir material por preenchimento atmosférico. As posições/triângulos da
representação escolhida ainda precisam ser processados. Oclusão raster por
profundidade existe, mas descarte antecipado de objetos inteiros escondidos por
montanhas ainda não está implementado. Não confundir objetos fora da vista com
objetos oclusos dentro da vista. Cobertura coletiva de copas/silhuetas continua
sendo a próxima oportunidade; não há corte automático por3px ou10km.

### Revisão de custo da unidade

Geração e compilação são de autoria/build time; o loop de frame não muda. O
parser/writer tem buffers fixos pequenos, sem cópia do arquivo completo nem
serialização pretty seguida de compacta para a cena. O candidato autoral continua
alocando conforme conteúdo e a troca conserva validação transacional. Sem nova
dependência, worker, lock, cache ou alteração de formatos. Instâncias aumentam
três vezes; o crescimento efetivo de CPU/GPU precisa ser medido na mesma vista.


### Resultado da entrega de30mil

Build `1a000db026bedafe`, macOSarm64/release/Metal. Cena compilada, salva e
reaberta pelo MCP;14.426.349bytes de autoria. Conferência após o save nativo:
10mil árvores anteriores conservadas bit a bit em f32, mesmos assets e pacotes
ATM, terreno, entidades e luz iguais.30mil pares casca/folhas confirmados.
Captura1728×1117 inspecionada: copas mais densas, silhuetas e atmosfera preservadas.
Não é uma comparação de pixels iguais, pois20mil árvores foram acrescentadas.

Nessa captura,31.818componentes foram rejeitados por frustum;28.188estavam na
câmera.109.116.810triângulos detalhados +40.711.854baratos continuaram selecionados,
contra608.881.002triângulos lógicos totais.17.051componentes usaram LOD e17.421
substituição de material (categorias sobrepostas). Geometria GPU alocada reportada
9.949.248bytes, mais2.254.960bytes de instâncias e600.556bytes compilados; não é
memória total. A floresta distante ainda tem custo significativo.

Janela aberta com30mil, luz da cena, sem blur, otimização ligada, câmera livre
e FPS/Immediate ativo. Três amostras durante navegação a2784×1560:10,04/9,91/7,59FPS.
A câmera mudou entre amostras (contagens de geometria diferentes), portanto não
são benchmark parado nem A/B contra os48FPS informados na cena10mil. O teste
mais denso pressiona fortemente o custo restante; não foi introduzido corte de
qualidade para compensar. A janela continua disponível para aceite humano.

Validação:20testes Rust em contracts/mesh_resources/projects, incluindo uma nova
prova crítica de persistência;3provas GPU existentes ignoradas nessa seleção,
pois o renderer não mudou. Clippyrelease/all-targets-Dwarnings, fmt, Python,
buildrelease e diffcheck aprovados. A fixture nova inicialmente comparou a ordem
não canônica de entidades do gerador com a ordem canônica de LoadScene; corrigida
na preparação da fixture, sem relaxar comparação de dados nem alterar a engine.
Evidências em `artifacts/landscape-forest-dense/`: population, verification,
continuity, live-verification, logs, PNG e cena. Binário corrente reutilizado,
sem cópias de build, push ou alteração de Scene103/replay113/protocolo5.
