# Varredura de perfil em trajetória 3D — EAF-113

## Fricção, alternativas e decisão antes do código

Na casa ea8c6b8, Mesh.tube/curve calculam frames, anéis, faces e coordenadas de
material no cliente. A planta Y permite seções deslocadas e BézierXZ, mas não
uma ogiva inteira que sobe e desce. O loft de anéis precisa dos anéis já
calculados. Replicar essa matemática para cada ferramenta consumidora mantém a
fricção; mudar toda a parametrização das plantas/rigs seria um contrato maior.

Incremento escolhido: gerador genérico `MeshEdit.sweep{id,sweeps}`. Cada peça
recebe perfil2D convexo, cadeia Bézier cúbicaXYZ aberta, orientação inicial do
perfil e escalas por estação. Gera Asset comum; edit_mesh/preview_mesh_edit/lotes
usam o dispatcher, undo e budgets existentes. Não persiste uma receita de sweep
nem a trata como planta. Alterar parâmetros gera novamente geometria; substituir
asset elimina rig/clips/metadados como o gerador grid, sem promessa de transferir
escultura. Planta vinculada exige detach explícito. Scene72/replay76 mantidos:
é operação de autoria, não campo novo persistido. Materiais73/77 seguem outra
frente. Sem renderer, dependência, ABI, Vertex ou limite novo de documento.

## Contrato

1..256 peças; cada perfil3..32 pontos distintos, estritamente convexo, CCW,
origem estritamente interna e coordenadas finitas±100m. Cadeia1..16 cúbicas;
controles finitos±10000m, endpoints compartilhados exatamente e tangentes de
mesma direção nas junções. Sem fechado/cantos automáticos: uma quina intencional
é autoria de peças separadas. Regularidade da derivada é admitida por limites
conservadores em subdivisões da curva quadrática derivada, profundidade máxima12;
cúspides e casos não certificados são recusados.

1..128 intervalos por cúbica. `centerline_tolerance`0.000001..10m limita o desvio
da curva em relação aos segmentos amostrados pelo casco convexo dos controles
de cada subintervalo; não é limite do erro de toda a superfície ou do perfil
poligonal. `profile_x` finito, não nulo e não paralelo à tangente inicial;
projeção perpendicular define primeiro frame. Transporte por rotação mínima
entre tangentes amostradas; reortogonalizar, recusar reversão. Sem prova de
torção/auto-interseção global entre amostras ou peças.

Escalas lineares por estações0..1 estritamente ordenadas,2..32, endpoints0 e1,
valores positivos0.001..100. Tempo da cadeia é (índice da curva+t)/quantidade;
não é parametrização por comprimento de arco. Coordenada de material é
[perfilX×escalaX, comprimento poligonal acumulado, perfilY×escalaY]. Caps opcionais
compartilham os anéis extremos. Triângulos laterais precisam orientar-se para
fora segundo normais do perfil nos frames adjacentes; geometria final publicada
em f32 passa Geometry.validate. Recusa local não prova ausência de interseções
distantes. Peças permanecem ilhas independentes, sem boolean/weld implícito.

Contagens exatas: anéis=(curvas×spans+1), vértices=anéis×lados,
faces laterais=2×(anéis−1)×lados, caps=2×(lados−2). Preflight antes de expandir,
por asset e documento; alocar buffers uma vez. Rejeição não altera autoria.

Na aplicação da casa surgiu outra fricção: molduras e blocos precisam compartilhar
o asset/material, pois há16slots. `append:true` acrescenta as peças a asset
existente sem rig/clips, preserva posições/faces/metadados antigos e usa posição
local antiga como canal material explícito quando ele era ausente. Plan valida
uma vez; preflight soma a geometria antiga antes de cloná-la/reservar saída.
Asset inexistente ou rigado recusa append. Não é weld nem skin automático.

O primeiro ensaio recusou13curvas por amostragem insuficiente (erro registra peça
e limite). Para evitar ciclos de tentativa/erro no cliente, `plan_mesh_sweep`
valida os parâmetros e dobra spans até certificar a tolerância, máximo128. Retorna
resolução por peça, contagens e limite da linha central, sem expandir malha.
`surface_validated:false`: preview/edit ainda precisam admitir frames, faces e
conversão f32. Não promete menor resolução possível; no máximo8tentativas por
peça, soma de amostras limitada pela progressão até128. Query puro, sem cache,
arquivo ou estado novo. O cliente da casa usa o plano para publicar resolução
explícita; não recalcula frames ou tesselação das molduras.

## Checklist estático anterior

1. Só autoria, nada por tick.2. Contagens antes dos buffers; frame/escala e
derivadas por anel, não por vértice.3. f64 temporário/u32 índices/f32 saída;
sem unsafe ou dependência.4. Validar perfil uma vez; estação avança em ordem;
amostragem limitada, nenhuma busca global por ponto.5. Limites de peças/perfil/
curvas/spans e caps de mesh herdados; checar soma antes de expandir.6. Mesmo
renderer/material/cache, capturas somente explícitas.7. Mesmo staging sequencial,
sem lock/worker novo.8. changed_assets/proteção de planta participam da nova
operação, exercitar preview/undo/reset/load/replay.9. Casos analíticos, perfil
assimétrico/3D, caps/manifold, erros/limites e uso na casa.10. Mantidos: avaliação
síncrona e frame discretizado; sem inversa de planta, preservação automática de
escultura/rig, C1 de malha finita, câmera/solver ou FPS medido.

## Revisão estática final

1. Frequência: `Plan::new`, `plan` e `append` executam só em consultas/edições;
nenhum novo trabalho de simulação ou frame.
2. Alocação: `counts`/`check_replacement_counts` antecedem expansão e clone do
asset antigo. Buffers reservam contagens exatas; material ausente antigo vira
canal explícito por cópia, custo deliberado para conservar aparência no append.
3. Representação: cálculo temporário f64, publicação f32/u32; Geometry.validate
rejeita perda degenerada após conversão. Vertex e dependências inalterados.
4. Repetição: regularidade/perfil validados uma vez por Plan; escala e frame por
anel, estações avançam em ordem. Planner não expande superfícies.
5. Limites: até256peças,16curvas,32lados,128spans; planner dobra até o teto e
recusa orçamento. Regularidade usa árvore de profundidade12, sem retry livre.
6. Render: malha comum e cache existente; comparação anterior preservou pixels.
7. Concorrência: dispatcher/staging sequenciais existentes; nenhum lock novo.
8. Invalidação: Sweep participa de changed_assets e da guarda de planta; preview,
lote, append, undo/redo, remoção e reset cobertos pelo cenário MCP.
9. Evidência:7testes Rust novos,7grupos MCP próprios; suíte final857testes Rust e
86cenários/352grupos MCP, Clippy/fmt aprovados. Tolerância conferida contra1001
amostras independentes; teste distingue plano admitido de superfície recusada.
10. Custos mantidos: autoria síncrona com staging/validação existentes, frame
discretizado, contagem conservadora e ilhas sem weld. Nenhum ganho de FPS medido;
auto-interseção global, preservação de rig e edição paramétrica persistente não
foram entregues por esta operação.

## Resolução por curva — EAF-120

spans agora aceita escalar1..128 ou lista exatamente do tamanho de curves com
cada valor1..128. Escalar preserva saída/refinamento antigos; lista permite que
plan_mesh_sweep refine só os trechos necessários. Requested_spans/spans voltam
na mesma forma; curve_centerline_bounds expõe o limite central de cada curva.
Contagens por1+soma(spans), com um anel compartilhado nas junções. Tempo de seção
é o da cadeia, não a densidade dos anéis. Não há garantia de amostrar cada
estação ou de erro da superfície. Contrato/checklist e caso da mola em
[sweep-curve-sampling.md](sweep-curve-sampling.md).
