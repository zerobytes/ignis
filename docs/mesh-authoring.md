# Autoria nativa de malhas — primeiro recorte

Construção reutilizável de superfícies com espessura e vãos: [painéis](mesh-panels.md).

Continuidade: [topologia e subdivisão linear](mesh-topology.md) acrescenta seleção
de componentes e subdivide MCP-first, incluindo amostragem uniforme de plantas.
As limitações históricas deste primeiro recorte não substituem o contrato novo.

Direção de Guilherme em 26/10/2026: criação, escultura, rigging e animação nativos na Ignis, MCP-first. Base auditada: main local c1ada9c, cena47/replay51; renderer de caixas, sem malhas/rigs/clipes. Pesquisa gráfica EAF-75 permanece isolada. Este documento descreve o recorte em implementação; evidências de conclusão serão registradas separadamente.

## Alternativas e escolhas do recorte

- Autoria externa no Blender reduziria o trabalho de ferramentas, mas não atende à escolha explícita de autoria nativa. Intercâmbio continua possível em incremento futuro; não é dependência deste pipeline.
- Malhas trianguladas indexadas, coordenadas locais em metros, Y para cima; instâncias têm translação, quaternion XYZW unitário e escala positiva uniforme. Suportar escala não uniforme exigiria contrato adicional de normais e deformação. Aparência não substitui colisão, apoio ou oclusão sensorial.
- Assets visuais e instâncias autorais separados de entidades/InstanceId de gameplay. Primeiro recorte independente: não anexar implicitamente ao NPC, nem usar slots de população como handles de assets. Anexação a atores exige contrato próprio de identidade/retirada.
- Escultura desloca vértices com pincéis radiais explícitos (draw, grab, smooth), falloff e traços em coordenadas locais; preserva índices/topologia. Remesh/dyntopo exigiriam remapear pesos/UVs/clipes e ficam para outro incremento. Escultura de uma base rigada conserva os pesos, sem prometer correção anatômica.
- Rig genérico, sem pressupor humano: hierarquia em ordem de pais, bind pose local, até quatro influências por vértice, índices válidos e soma de pesos unitária. Bind global inversa calculada na avaliação; linear blend skinning na CPU. GPU skinning reduz uploads de geometria animada, mas acrescenta buffers/pipeline e sincronização; a CPU permite verificar numericamente este recorte limitado. Sem IK, auto-rig ou retargeting implícitos.
- Animação: tracks por osso com translação e quaternion locais, keyframes em ticks inteiros; interpolação linear/slerp, escala de osso fixa em1. Clipes e playback são autoria; play/pause/step/reset existentes controlam o relógio fixo da simulação. Mudança de autoria reinicia a execução, conforme contrato atual; troca de clipe ao vivo e blend tree exigirão comandos/logs próprios. Root motion não move corpo físico.
- Geometria/rig/clipes imutáveis compartilhados entre histórico/cenas por Arc; edição copia apenas payload tocado. Não embutir malha pesada em cada Entity/RenderFrame nem clonar todos os vértices por tick. RenderFrame reutiliza vértices de saída; cache compara conteúdo para reset/undo/replay, sem confiar apenas em revisão/tick.

## Ordem e gates

1. Criar asset por vértices/triângulos ou receita de grade, instanciar e capturar com iluminação atual. Validar índices, degeneração, valores finitos e budgets antes de publicar.
2. Aplicar escultura determinística e reversível. Smooth lê o estado anterior de cada amostra, sem dependência da ordem dos vértices. Uma operação/traço é um passo de undo.
3. Autorar rig/pesos, inspecionar bind/pose; validar hierarquia/ciclos, nomes, pesos e singularidades. Rig incompatível com clipes existentes rejeita atomicamente.
4. Autorar clipe, configurar playback, avaliar pose em tick explícito ou corrente, executar/capturar/reproduzir replay. Um mesmo asset admite instâncias com playback distinto.

Todas as operações passam por Command/SceneEdit e engine_command MCP; nenhuma operação visual pode ser adicionada sem a correspondente operação compartilhada. A ferramenta MCP também recebe expected_revision. Escultura/rig/clipes inválidos preservam autoria, execução, revisão e histórico. Consultas não alteram relógio nem partida. Ferramentas manuais de pincel/gizmos/timeline serão gates separados; MCP-first não equivale a editor visual completo.

## Revisão estática antes de implementar

1. Preparação e validação na edição; avaliação de pose somente ao preparar frame/consulta/replay, sem nova fase de gameplay.
2. Arc nos payloads pesados, scratch de skinning limitado, saída de frame reutilizada. Cópia de geometria somente na escultura/substituição; histórico existente de32 estados, sem coleções históricas de builds.
3. Índices numéricos após validação; IDs textuais no documento/controle. Nenhuma ABI/plugin nova.
4. Rig avaliado em ordem dos pais; normals por triângulo após deformação, sem cálculo de normais por pixel na CPU. Compartilhamento de pose entre instâncias é melhoria futura, não promessa do recorte.
5. Limites por asset e por documento para vértices, triângulos, ossos, influências, clipes/keyframes, instâncias e traços. Rejeitar excedentes antes da publicação; não alegar escala de produção.
6. Buffer GPU reutilizado, upload apenas quando vértices mudam. Captura/readback e JSON de pose apenas sob demanda; nenhum arquivo externo mutável necessário para replay.
7. Mesmo dispatcher e locks existentes; nenhuma thread/dependência de Blender. Payloads inline limitados: bulk assets/cooker assíncrono continuam futuros.
8. Testar mudança/remoção/growth/undo/reset/load/resize, instâncias compartilhadas e poses distintas no mesmo tick.
9. Testar geometria conhecida, pesos/bind/skin, interpolação/deadlines, rollback, save/load, replay/prefixo/pixels, regressão NPC completa headless.
10. CPU skinning, normals flat, uploads de malha animada e reset por edição são custos explícitos deste recorte. Sem FPS, qualidade de escultura profissional, suporte multiplataforma ou aceite humano presumidos.

## Contrato de controle

`engine_command` recebe `command.op: edit_mesh` com `edit.kind`:

| kind | Payload | Efeito |
|---|---|---|
| grid | id, segments, size | Cria/substitui asset de grade XZ, normal +Y |
| put_asset | asset | Cria/substitui geometria indexada, rig e clipes explícitos |
| sculpt | asset, stroke | Modifica a base; não modifica índices nem pesos |
| set_rig | asset, rig (ou null) | Substitui rig/pesos; clipes existentes precisam continuar válidos |
| put_clip / remove_clip | asset, clip | Autoriza/substitui clipe completo / remove pelo ID |
| put_instance | instance | Cria/substitui posição, rotação, escala, cor e playback |
| remove_instance / remove_asset | id | Remove; referências remanescentes fazem a transação falhar |

`inspect_meshes` retorna resumos e instâncias. `inspect_mesh {asset}` retorna payload autoral completo. `inspect_mesh_pose {instance,tick?}` retorna vértices no mundo e matrizes globais dos ossos no espaço do asset; tick ausente consulta o presente, tick explícito consulta0..36000 sem alterar execução. Normais são flat, recalculadas por triângulo depois da deformação. Não há UVs/PBR neste recorte.

Todas as edições aceitam `expected_revision` no Request. Para alterações interdependentes, usar `edit_scene` com operações `{op:"mesh",edit:...}`; referências são verificadas no documento final, com um único undo. Remover asset referenciado isoladamente falha; remover suas instâncias e o asset no mesmo lote funciona. `put_asset` e `grid` substituem o asset completo; não preservam rig/clipes omitidos implicitamente. Uma instância com referência inválida faz a substituição falhar.

Exemplo de criação pelo MCP:

```json
{"command":{"op":"edit_mesh","edit":{"kind":"grid","id":"surface","segments":16,"size":4}}}
```

Exemplo de escultura (coordenadas locais em metros):

```json
{"command":{"op":"edit_mesh","edit":{"kind":"sculpt","asset":"surface","stroke":{"kind":"draw","centers":[[0,0,0]],"radius":1.4,"strength":0.45,"direction":[0,0,0]}}}}
```

`direction` é usada somente por grab, normalizada antes do deslocamento. Draw usa normal média ponderada por área; smooth usa média dos vizinhos por incidência de aresta nos triângulos (arestas compartilhadas contam por face). Falloff radial `(1-(distância/raio)²)²`, sem efeito fora do raio. Cada amostra usa seu predecessor congelado; um traço completo é uma edição atômica. Isolados não se deslocam em draw/smooth. Triângulos degenerados na base final fazem a edição falhar.

Clipes usam poses locais absolutas, não deltas de bind; ossos sem track conservam sua bind pose. Antes da primeira/depois da última chave, conservar a chave extrema. No modo não looping, saturar em duration_ticks; no looping, intervalo[0,duração), inclusive quando offset+tick alcança a duração. Quaternions são normalizados após validação e slerp; pesos com tolerância de soma1e-4 são renormalizados na avaliação. Bind pose não incorpora animação. Escala positiva uniforme pertence à instância, não aos ossos.

Limites:16 assets,32 instâncias,4096 vértices/8192 triângulos por asset;16384 vértices/32768 triângulos armazenados e65536 triângulos desenhados no documento;32 ossos por rig,4 influências/vértice,16 clipes/4096 chaves no documento;128 amostras/traço; grade1..63 segmentos e tamanho0,01..100m. Raios0,001..100m, força[-1,1]m por amostra (smooth[0,1]), coordenadas autorais finitas±10000m. IDs ASCII1..64 caracteres. A geometria mínima tem3 vértices e1 triângulo com área positiva. Não inferir suporte a malhas de produção acima desses budgets.

Cena48 adiciona `meshes` opcional, autocontido, omitido em cenas vazias. Cenas1..47 continuam migrando, mas não aceitam payload novo escondido em versão antiga. Replay52 inclui poses esperadas, compara vértices/ossos/tempo e reconstrói prefixos; replay continua restrito ao mesmo build, sem promessa de migração entre builds. Sem dependência/ABI nova; serde habilita suporte a Arc apenas para compartilhar payloads em memória.

## Demo e reprodução

A verificação `python3 scripts/run_verifications.py --only meshes` constrói pelo MCP uma grade esculpida, três ossos, pesos graduais e um clipe com duas instâncias em fases distintas. Salva a cena em `artifacts/mcp-validation/examples/native-meshes.scene.json`, capturas em `artifacts/mcp-validation/artifacts/meshes-*.png` e replay no mesmo diretório. O exemplo revisado fica em `examples/native-meshes.scene.json`. Não depende de Blender ou arquivos externos de geometria.

Para abrir voluntariamente a demo nativa deste checkout: `target/release/ignis --root "$PWD" --scene examples/native-meshes.scene.json --window --mcp`. P inicia/pausa e R reinicia; as operações de autoria permanecem no MCP. Esta unidade não implementa pincel de mouse, seleção de triângulos, gizmos, editor de pesos/timeline, remesh, IK, retargeting, root motion, anexação a NPC ou troca de animação ao vivo. Os controles existentes não passam a editar malhas por inferência.

Persistência mantém o limite de leitura de8MiB por documento. Salvamentos JSON usam formatação legível enquanto couber; acima disso tentam JSON compacto, sem mudar o schema. Se o compacto também exceder8MiB, rejeitam antes de substituir o destino. Malhas/poses grandes válidas são cobertas por roundtrip; não ampliar o limite de arquivo silenciosamente. Capturas informam `mesh_vertex_count` e `mesh_triangle_count` além dos contadores de caixas/guias.

O envelope MCP permanece limitado a2MiB por mensagem; dados de asset, rig e clipe podem ser autorados em operações próprias. Esse limite de transporte é distinto do limite de arquivo e dos budgets de geometria. Nenhum canal binário/cooker novo está implícito.

Revisão do teto de desenho em cena55/replay59: [custos e limites](mesh-draw-budget.md).
