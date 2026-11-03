# Transformação local de ilhas — EAF-125

## Decisão anterior ao código

A casa ocupa16assets; extrair um acessório apenas para reposicioná-lo exige um
slot ausente. Aumentar os tetos só adia isso. Componentes nomeados persistentes
seriam úteis para regeneração, mas exigem identidade/remapeamento de faces e um
novo contrato de documento. Neste recorte reutilizamos as ilhas por índices
compartilhados e a referência scope/revision já existentes.

`MeshEdit.transform_islands {asset,reference,islands,transform}` atua no asset,
afetando todas as suas instâncias. `transform` exige `pivot`, `translation`,
`rotation` XYZW unitária e `scale` positiva uniforme0.001..100:
`p' = pivot + translation + rotation * ((p-pivot)*scale)`.
Pivô/translação/saída são finitos dentro de±10000. Não há mirror/shear nem
reparenting. IDs de ilha continuam mínimos índices de faces, válidos somente
na revisão inspecionada; proximidade espacial nunca une peças.

Índices, contagens, outras ilhas e vértices isolados ficam intactos. Coordenadas
de material existentes permanecem exatas; quando implícitas, as posições antigas
são materializadas para o padrão acompanhar a peça. Rig/pesos/clipes são
preservados: é edição da geometria de repouso, sem transformar ossos ou garantir
que a pose animada seja um movimento rígido equivalente. Plantas vinculadas
exigem detach explícito, evitando rebase implícito da fonte paramétrica.

Preview/edit/lote/undo usam o dispatcher existente. Seleções/parâmetros e a
orientação final arredondada para f32 são validados antes de substituir Geometry.
Sem novo formato salvo, renderer, dependência ou aumento de limites.

## Checklist estático prévio

1. Só autoria por pedido, nenhum tick/frame novo.
2. Partição limitada existente; copiar posições uma vez após validar seleção.
3. f32 público/f64 na transformação; máscara por índice, quaternion normalizado.
4. O(V+T) mais ordenação da seleção; sem busca espacial/solver.
5. Tetos atuais mantidos; verificar limites e colapso/inversão no resultado f32.
6. Sem GPU/I/O novo; canal implícito passa a consumir12V bytes quando congelado.
7. Mesmo dispatcher, sem locks/workers/dependências.
8. changed_assets e referências de componente invalidam lote/cache existentes.
9. Analítico/rig/material/erro e MCP preview/history/load/replay/capturas.
10. Cópia dos buffers de Geometry permanece custo limitado de autoria; não mede FPS.


## Uso

Consultar `inspect_mesh_islands` e enviar a referência retornada junto dos IDs
selecionados. O mesmo edit serve para `preview_mesh_edit` e `edit_mesh`:

```json
{
  "kind": "transform_islands",
  "asset": "accessories",
  "reference": {"scope": "valor-retornado", "revision": 12},
  "islands": [24, 68],
  "transform": {
    "pivot": [0, 1, 0],
    "translation": [0.25, 0, 0],
    "rotation": [0, 0, 0, 1],
    "scale": 1
  }
}
```

Lote permite transformar peças de assets diferentes sob uma única revisão e um
único undo. Uma segunda edição por referência do mesmo asset após alterá-lo no
lote é recusada; combinar suas ilhas no mesmo edit ou inspecionar novamente.
Vértices isolados não são ilhas selecionáveis. A opção não encontra contatos,
resolve colisões ou preserva uma relação arquitetônica automaticamente.

## Revisão estática final

1. Adequado: `Document::transform_islands` só executa por edição/preview.
2. Adequado: seleção e parâmetros validados antes de clonar posições; Geometry
   nova só depois da verificação f32. Rig/clips permanecem nos mesmos Arcs.
3. Adequado: DQuat/DVec3 temporários, quaternion normalizado; índices u32 originais.
4. Mantido com motivo: `Partition::new` valida e cria listas de faces/vértices
   limitadas, inclusive faces que a transformação não usa; evita duplicar o
   conceito de ilha. O(V+T), seleção O(S log S), sem operação por frame.
5. Adequado: nenhum crescimento de contagens; finito±10000 e área/orientação
   contra normal de referência rotacionada, antes da publicação. Geometry.validate
   também verifica o resultado final segundo o mesmo contrato do renderer.
6. Mantido com motivo: coordenadas implícitas congeladas custam12V bytes no
   documento (até192KiB/asset), sem crescer Vertex, GPU layout ou draw count.
7. Adequado: sem dependências, threads, locks ou I/O de rotina novos.
8. Adequado: variante incluída em component_reference/changed_assets e proteção
   de plantas; mesma transação/clonagem de cena, invalidação e histórico existentes.
9. Analítico com pivô/rotação/escala, ilhas coincidentes distintas/isolados,
   material implícito/explícito, rig/pose/Arcs e erros de seleção/precisão; MCP
   cobre schema/preview/lote/stale/undo/redo/plant/load/reset/replay/prefixo/GPU.
10. Mantido com motivo: copiar buffers de Geometry e validar a geometria inteira
    continua custo síncrono limitado de autoria. IDs semânticos persistentes,
    transformações por instância de subpeças e redução de orçamento ficam abertos.
