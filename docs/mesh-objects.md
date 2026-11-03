# Objetos visuais compostos

As peças de um objeto podem conservar malhas, materiais e transformações próprias
sem perder a identidade visual do conjunto. `meshes.objects` associa um ID a uma
lista explícita de instâncias. Isso vale para árvores, edifícios, máquinas e
qualquer composição autoral; o renderer não interpreta nomes, espécie ou forma.

Um objeto não é um parent de transformações, ator, colisor ou entidade física.
Agrupar não move, combina, duplica ou solda malhas e não remove folhas nem vãos.
Geometria repetida continua compartilhada por asset. Attachments continuam tendo
sua semântica anterior, inclusive animação; não foram usados como agrupamento.

## Autoria e persistência

O campo opcional `meshes.objects` contém `{id, members}`. IDs usam o mesmo domínio
ASCII de identificadores autorais (1..64 letras/dígitos/underscore/hífen).
Cada membro deve existir e pertencer a no máximo um objeto; grupos vazios,
referências ausentes e IDs duplicados são recusados na transação candidata.
Não há teto arbitrário de objetos/peças além dos limites estruturais e de memória
já presentes no documento. A representação autoral mantém os IDs; o caminho quente
usa somente índices numéricos e volumes compilados na preparação do frame.

`EditMesh` e `SceneEdit::Mesh` recebem:

- `put_object {object}`: publica ou substitui o grupo completo.
- `assign_object {instance, object}`: transfere uma peça; `null` desvincula.
- `remove_object {id}`: remove apenas o agrupamento.

`inspect_mesh_objects {object?}` retorna totais e, opcionalmente, um objeto por ID.
O painel de propriedades da malha tem **Objeto composto**, com vínculo, transferência,
desvínculo e remoção pelo mesmo dispatcher. Como outras edições autorais, essas
operações pausam/reconstroem a execução e renovam seus escopos. Não preservam tokens
de controle de uma execução anterior. Um `RemoveInstance` limpa o vínculo e remove
um grupo que ficou vazio; remover apenas o grupo preserva todas as peças.

Scene107, replay117 e conteúdo público7 registram a nova identidade. Protocolo5,
checkpoint3 e envelope PSV1 permanecem; suas guardas de Scene também verificam107.
Cenas antigas sem o campo continuam legíveis. Um campo vazio não é serializado,
preservando o fingerprint dos subconjuntos usados por personagens existentes.
Vínculos não são inferidos nem aplicados automaticamente ao corpo humano.

As receitas `build_landscape_forest.py` e `build_representative_forest.py` agora
publicam as duas peças como um objeto. `link_forest_objects.py` migra a cena
aprovada usando seu manifesto de população, conferindo poses sem regenerá-las.
Essa convenção de IDs pertence à receita de demonstração, nunca ao renderer.

## Primeiro consumidor: visibilidade conservadora

Na preparação de fonte estática, o renderer resolve membros para índices de draw e
constrói o AABB mundial que contém todas as peças e representações distantes.
O cache é separado da geração animada dos avatares, evitando reconstruí-lo só
porque um personagem mudou de pose. Mudanças na fonte, na máscara de instâncias
consumidas ou nos pacotes distantes invalidam esse cache.

Antes da passagem inteira, uma amostra distribuída pela ordem autoral estima
quantos testes por peça seriam evitados. Câmera e sombra decidem separadamente;
se os grupos cruzam demais os planos do frustum e não compensam a consulta
adicional, o renderer conserva o caminho anterior. Essa amostra limita apenas o
trabalho da previsão de custo, nunca a população de objetos, e não decide
visibilidade nem qualidade. É uma previsão conservadora, não um medidor universal
de tempo em qualquer hardware.

Quando ativo, o volume composto tem três resultados: fora, dentro ou parcial.
Fora/dentro eliminam os testes individuais de frustum daquele conjunto; parcial
mantém os testes anteriores por peça. Margens de erro de transformação e projeção
preservam o fallback conservador. Grupos com rig, clip, attachment ou membro ausente
no frame mantêm integralmente o caminho anterior. Projeção individual, ordem de
draw, oclusão na GPU, materiais, sombras, contornos e colisores ficam inalterados.

`adapter.visibility` expõe `object_groups`, `object_camera_tests_saved`,
`object_shadow_tests_saved`, `object_camera_active`, `object_shadow_active` e
`object_cache_bytes` (capacidades alocadas, não apenas
comprimento útil). `object_groups` conta os grupos elegíveis em cache; os campos
`active` informam se foram usados naquela vista. O cache usa RAM de CPU; não acrescenta textura ou buffer na GPU.
`Renderer::set_object_culling(false)` é o oráculo de diagnóstico do caminho anterior.

## Fronteira com LOD coletivo

O vínculo permite certificar cobertura e aparência do objeto inteiro no compilador,
mas ainda não cria essa representação. Este incremento reduz consultas de
visibilidade preservando os mesmos triângulos. Não promete uma árvore de dois
triângulos, economia de shading nem um novo FPS de gameplay. O próximo contrato
é compilar uma representação coletiva de silhueta/cobertura/materiais e selecioná-la
somente quando o erro em pixels físicos e o custo total compensarem.


## Evidência da rodada — 01/11/2026

Apple M4 Pro/Metal, build `08ca961bf0ceb9ca`. A/B do renderer anterior contra ele
mesmo com seleção de grupos; 30 mil objetos / 60 mil peças, as mesmas malhas,
poses, materiais, atmosfera, oclusão GPU e iluminação. IGL1 permanece desligado.
32 amostras por variante, blocos alternados aquecidos; PNG/readback fora da medição.
CPU/submissão é tempo de parede de `draw`, incluindo driver; total acrescenta espera
pela GPU. Não são timestamps GPU nem FPS de gameplay.

| Vista | Pixels físicos | CPU/submissão anterior → automática ms | Total anterior → automático ms | Grupos usados |
|---|---:|---:|---:|---|
| Solo | 1280×800 | 9,819 → 9,555 | 46,421 → 46,119 | Sim |
| Solo | 1728×1117 | 9,938 → 9,469 | 50,347 → 49,916 | Sim |
| Aérea média | 1280×800 | 14,040 → 13,956 | 153,274 → 154,029 | Não |
| Aérea média | 1728×1117 | 14,012 → 13,980 | 158,523 → 158,082 | Não |
| Aérea ampla | 1280×800 | 15,265 → 15,193 | 217,383 → 217,105 | Não |
| Aérea ampla | 1728×1117 | 15,159 → 15,226 | 220,446 → 220,999 | Não |

Perto, cerca de 50,6 mil testes individuais deixam de ser feitos, em troca de 30 mil
consultas mais baratas de grupo. O ganho de submissão foi 2,7–4,7%, com redução total
de apenas 0,65–0,86%. Nos panoramas, as variações totais de aproximadamente ±0,5%
com o mesmo caminho não demonstram ganho. Não há nova redução de triângulos.
A versão sem escolha por custo acrescentava cerca de 0,5 ms de CPU no panorama;
a política final evita essa passagem inútil. Não se infere ausência universal de
regressões em qualquer cena ou dispositivo a partir desses dados.

Todas as seis vistas estáticas e os oito frames com câmera/luz/oclusor/peças móveis
ficaram RGB8 exatos. O oráculo GPU menor também cobre sombras, resize, rotação,
recorte próximo, geometria única/compartilhada e fallback de attachments. Os dois
testes críticos novos cobrem identidade de membros, recusa atômica, transferência,
remoção/undo, cena/replay/save portátil/conteúdo público e guardas de versão.
16 testes Rust em sete alvos, roteiro MCP real, Clippy de todos os alvos, formatação
dos arquivos próprios e build release passaram. UI compilada e com dispatcher
compartilhado; clique humano no painel e novo FPS nativo não foram verificados.

O cache adicional medido é 1.348.596 B (1,29 MiB) de CPU. Há também os IDs autorais
na Scene; não se trata de uma redução da RAM total. A cena JSON aumentou 1.949.998 B
somente pelo vínculo. GPU não recebe esse catálogo de nomes nem um novo buffer;
a consulta quente usa índices/volumes numéricos preparados uma vez por fonte.

Reprodução:

```sh
python3 scripts/link_forest_objects.py
cargo run --locked --release --example automatic_lod_cost -- artifacts/object-culling/grouped.scene.json --object-culling --world-player-view
python3 scripts/verify_mesh_objects.py
```

A cena aprovada em `artifacts/expanded-world/world.scene.json` recebeu apenas
`objects` e a versão107, após conferir igualdade de todos os demais campos e o
hash da fonte anterior. Não houve regeneração de topografia, população ou colisão.
`target/release/ignis` é o único executável corrente. A janela de personagens em
uso humano foi preservada; nenhum aceite nativo foi presumido dessa atualização.
Dados locais: `artifacts/object-culling/comparison.json`, `motion.json`, `source.json`,
`mcp.json`, `unguarded-summary.json` e PNGs. A tarefa181 continua In Progress para
representações coletivas úteis, distintas deste primeiro consumidor de identidade.

Continuidade experimental: [resumo por vistas](collective-view-probe.md) mostrou
que diminuir o trabalho geométrico pode trazer ganho grande, mas o primeiro
pacote não preservou silhuetas e permanece desligado. Seus resultados negativos
orientam a próxima hierarquia; não substituem a validação do culling acima.

A continuação diagnóstica usa [hierarquia de geometria](collective-geometry-query.md),
ainda desligada enquanto cobertura/imagem e custo não passarem juntos.
