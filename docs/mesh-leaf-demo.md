# Folha articulada — demonstração nativa

## Revisão antes da autoria

Demo usa somente operações existentes pelo MCP: put_asset para base triangulada,
subdivide, sculpt draw/grab, set_rig, put_clip, put_instance e capture. Python
descreve a base e orquestra comandos; subdivisão, escultura, skinning, animação e
renderização acontecem na Ignis. Não modifica o núcleo, shaders ou formatos.

Checklist proporcional: geração e inspeção somente sob demanda; até657 vértices,
1216 triângulos,3 ossos/2 influências,1 clipe e1 instância. Buffers e payloads têm os
limites já validados da engine. Capturas são explícitas e um conjunto finito de
frames é substituído no caminho corrente de artifacts; sem histórico de builds.
Nenhum trabalho novo por tick, I/O periódico, thread/lock/dependência da engine ou
cache externo. Mesma validação, histórico e renderer existentes. Custo mantido:
CPU skinning e capturas sequenciais para a prévia. GIF apenas codifica os PNGs
nativos; não é prova de fidelidade ou ganho de FPS. Verificar poses distintas,
save/load, replay e inspecionar captura antes de entregar. Operação sem janela.

## Reprodução e resultado

`python3 scripts/demo_mesh_leaf.py --frames 0` gera cena e capturas sem dependências
Python adicionais. Com Pillow disponível, `--frames 48` também codifica48 capturas
nativas num GIF de4 segundos. A engine continua sem dependência nova. O script
usa o build corrente `target/release/ignis` e um root isolado em
`artifacts/leaf-demo`, preservando a sessão visual aberta e a regressão existente.

A base explícita tem51 vértices/76 triângulos. Duas operações nativas de subdivide
produzem177v/304t e657v/1216t. Pincéis draw/grab criam volume, nervura, nervuras
laterais e curvatura de bordas/ponta. Rig de3 ossos, no máximo2 influências por
vértice, anima um ciclo de vento de240ticks. Cor única verde e normais flat são
recursos atuais do renderer; não há textura ou material PBR novo nesta demo.

Fonte reutilizável: `examples/native-leaf.scene.json`, salva pelo MCP. Captura
principal em `artifacts/leaf-demo/artifacts/leaf.png`, prévia em `leaf.gif`, base
e escultura em `leaf-base.png`/`leaf-sculpt.png`. `report.json` registra build,
operações, contagens, checks e caminhos. Capturas originais e frames são preservados
para conferir a procedência da prévia quantizada.

## Revisão depois da autoria

Frequência/tipos/acesso: somente script de demonstração, sem código da engine
alterado. Alocações: lista finita de48 frames paletizados, aproximadamente21MiB de
pixels mais overhead e conversão temporária; não entra no tick. Capturas usam os
buffers existentes e paths estáveis, sem versões históricas do binário. GPU/I/O:
capturas explícitas, não otimização de render. Threads/locks/caches/ABI não mudam.
Trabalho repetido e custo são conscientemente mantidos para uma prévia reproduzível.

Checks pelo transporte MCP: undo/redo de escultura restaura exatamente o asset;
poses animadas diferem do repouso; save/load restaura a pose inicial; pixels da
captura viva coincidem com replay. Inspeção da imagem confirma folha com nervura
central e silhueta curvada. O build usado é o validado em EAF-79; testes Rust não
foram repetidos porque não há mudança de núcleo. A execução da demo valida a nova
cena/script. Aceite artístico/humano permanece distinto da autoria automatizada.
