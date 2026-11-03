# Árvore articulada — demonstração nativa

## Revisão antes da autoria

Aplicação proporcional do checklist de `performance.md`: somente conteúdo e
orquestração MCP, sem mudanças no núcleo, formato, renderer ou dependências da
engine. Geometria base explícita, subdivisão/escultura/rig/clipes pelo dispatcher.
Folhas agrupadas em sprays reutilizáveis para respeitar32 instâncias e32768
triângulos desenhados. Até18 sprays de9 folhas, um asset de madeira contendo os
pecíolos; contar armazenamento/desenho antes de publicar as instâncias.

Geração/consultas apenas na autoria, sem gerador Python no tick. Sem threads,
locks, caches ou ABI novos. Custo mantido: skinning CPU, normais planas e upload
existentes; a composição não demonstra escala de produção nem ganho de FPS.
Capturas explícitas offscreen, root isolado `artifacts/tree-demo`, caminhos
correntes substituídos. GIF codifica uma sequência finita de PNGs nativos com
Pillow, fora da engine. Verificar undo/redo, animação, save/load/replay e pixels;
inspecionar enquadramento e conexões entre tronco, galhos e folhas.

As irregularidades da folha anterior misturam relevo deliberado, traços
sobrepostos e normais planas. Esta autoria usa camber suave na base, pincéis
mais fracos e smooth; não introduz normais interpoladas ou materiais PBR.

## Reprodução e resultado

`python3 scripts/demo_mesh_tree.py --frames 0` cria a cena e capturas pelo build
corrente `target/release/ignis`. Com Pillow disponível, `--frames 48` também
codifica48 capturas nativas800×800 num GIF de4 segundos. O script usa somente
MCP stdio, em processo próprio sem janela; não controla a demo aberta.

Fonte em `examples/native-tree.scene.json`, salva pelo MCP e idêntica à cópia
isolada. `artifacts/tree-demo/artifacts/tree.png` é a captura1200×1200;
`tree.gif` é a prévia animada; `report.json` registra build, operações, contagens
e checks. Binário existente3077e97f88c54625, macOS Apple Silicon/Metal.

- Folha base31 vértices/38 triângulos; subdivisão nativa99v/152t, seguida de
  draw/grab/smooth com relevo menor que a demonstração anterior.
- Cada conjunto reúne9 folhas,891v/1368t, rig compartilhado de19 ossos e até2
  influências por vértice. Clipe breeze com171 chaves em240ticks,18 fases distintas.
- 18 conjuntos,162 folhas, mais uma instância de madeira:19 instâncias/3 assets.
  Madeira3504v/6040t; armazenamento total4494v/7560t; desenho30664t, abaixo32768.
  O asset de folha fonte é mantido como peça de autoria, sem instância visível.
- Raízes, tronco, galhos e pecíolos são tubos explícitos combinados no asset de
  madeira. Nas bifurcações os tubos se sobrepõem; esta demo não implementa solda,
  boolean/remesh ou um gerador de árvores no núcleo. Madeira permanece estática;
  o rig anima a folhagem, com pequenas rotações também na base do conjunto.

## Revisão depois da autoria

Checks MCP aprovados: undo/redo exato do smooth, posições animadas distintas,
save/load da pose inicial, PNG vivo/replay byte a byte idêntico e retorno exato à
pose inicial após240ticks. GIF com48 frames distintos, duração4000ms,
1690060bytes. Capturas de árvore inteira e fase intermediária inspecionadas:
silhueta reconhecível, ramificações e folhas conectadas visualmente, relevo
mais suave. Sobreposição de folhas e facetas continuam visíveis; o resultado é
um estudo estilizado, sem aceite artístico/de produção presumido.

Frequência/alocações/tipos: Python gera o conteúdo uma vez, com listas finitas;
payloads validados pelos budgets existentes. Não há novo trabalho no tick do
núcleo. Memória de prévia:48 imagens paletizadas800×800, cerca29,3MiB de pixels
mais overhead/conversões temporárias. I/O/GPU: capturas explícitas e sequência
finita substituída em paths estáveis; nenhum build adicional. Sincronização,
invalidação e caches: contratos existentes, sem mudança. Custo mantido: CPU
skinning e avaliações por instância, render flat, readback/encoding da prévia.
Sem benchmark ou alegação de FPS. Testes Rust do mesmo núcleo não repetidos;
execução real do novo script/cena e inspeção dos PNGs verificam este incremento.
