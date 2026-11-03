# Sombreado de malhas — EAF-80

## Decisão e revisão anterior ao código

O teste da árvore mostrou facetas na base e nos galhos mesmo onde o contorno
era contínuo. Aumentar tesselação consome o budget sem controlar as normais.
Normais autorais persistidas exigiriam transferência/transformação na escultura,
subdivisão, regeneração e skinning, além de política para dados obsoletos.

Escolha deste recorte: `Asset.shading = flat | smooth`, padrão flat, com
`edit_mesh/set_shading` no dispatcher/MCP. Smooth deriva normais ponderadas por
área das posições avaliadas depois do skinning e do transform da instância.
Compartilhamento ocorre apenas por índice do mesmo asset; índices duplicados
mantêm seams. Não há weld por proximidade, normal map, ângulo de vinco, material
PBR ou solda de superfícies. A topologia autorada governa continuidade.

Cena52/replay56 após integrar o handoff de seções51/55. Modo preservado por
subdivisão/regeneração/save/load/undo/replay. Alterar shading de planta vinculada
atualiza apenas esse metadado na fonte e no avaliado; não rebasa fonte/detalhe/rig.

Checklist anterior: trabalho só quando o Frame precisa ser atualizado; flat
mantém caminho anterior. Smooth usa scratch fixo4096 Vec3 (48KiB), limpo só na
faixa de vértices do asset avaliado. Dois passes lineares por triângulos, sem
mapa, busca de vizinhos, heap, lock, thread ou I/O por instância. Sem ampliar
budgets. Custo conscientemente mantido: normais e skinning por instância em CPU;
poses compartilhadas/normal cache separado exigem outro desenho. Normais nulas
por cancelamento/faces colapsadas têm fallback finito por face. Não alegar FPS.

Verificar área ponderada, seams, rotação/escala, skinning, cache no mesmo tick,
cancelamento/degeneração, metadados e migração; cenário MCP com pixels antes/depois,
undo/redo/load/reset/replay/prefixo e planta regenerada. Regressão completa e
testes Rust/Clippy/fmt/build antes de entregar o recurso.

## Uso e contrato

```json
{"op":"edit_mesh","edit":{"kind":"set_shading","asset":"wood","shading":"smooth"}}
```

`inspect_mesh` informa `asset.shading`. Flat mantém uma normal por triângulo;
smooth compartilha apenas entre faces que referenciam o mesmo índice. Para uma
borda dura, a autoria precisa separar índices nessa borda. Superfícies
sobrepostas continuam separadas mesmo que ocupem o mesmo lugar. Não usar isso
como correção de T-junctions ou de interseções inválidas.

O campo omitido em cenas anteriores significa flat. Cena anterior a52 contendo
smooth é recusada antes de mutação/normalização. Replay56 exige o mesmo build;
pose continua registrando posições/ossos e o modo está na cena. Capturas verificam
a apresentação derivada, não um novo estado de gameplay.

Troca do modo é edição autoral: usa transação/histórico/reset de execução e
invalida seleção escopada como as outras edições. Geometria/rig/clipes mantêm os
mesmos Arc; planta vinculada muda metadado na fonte sem rebase de geometria/dimensões.
