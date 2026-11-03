# Geometria residente incremental — EAF-199/181

Integração orientada pelo handoff read-only do chat nanites em03/11. Reutiliza o
renderer existente, sem mudar projeção ou qualidade dos assets. O adapter
planetário posterior acrescenta a supressão opt-in da grade diagnóstica e o
derivado de normais contínuas; não altera a equação atmosférica.
É pré-requisito de residência planetária; não entrega streaming/circunavegação
por si só.

## Transform não é identidade de geometria

`meshes::Frame::update_filtered` possui caminho para conjunto estático inteiramente
compartilhado/residente: assets idênticos por Arc de geometria, shading/material,
ordem/máscara/identidade/membership iguais, sem rig/clip/attachment/playback.
Atualiza transforms, cores e cópia autoritativa das instâncias. Hierarquia por
índices continua válida; não reavalia vértices, normais ou pesos de projeção.
Demais documentos preservam a preparação completa anterior.

`geometry_generation` identifica o payload compartilhado, independente de
`generation` dos draws/poses. `static_generation` **continua mudando** quando
placements mudam: bounds de objetos compostos são mundiais. Visibility reutiliza
bounds locais somente nesse conjunto inteiramente compartilhado, com a mesma
geração de geometria e os mesmos pacotes; recompõe bounds mundiais de objetos.
Caudas de avatares, legados, mudanças de conteúdo/pacotes seguem o fallback.

## Entrada e saída de assets

O cache GPU deixa de usar slot denso como identidade. Sua chave identifica Arc
da geometria, shading, normais derivadas opcionais, representação/pacote, nível, layout, indexação e, no full64,
o slot de material embutido. Entries mantêm os Arcs usados como identidade;
reutilização de endereço pelo allocator não confunde conteúdo velho e novo.
Layouts compact48/minimal24 remapeiam material por instância; full64 invalida a
entry correspondente quando seu slot muda. Batches continuam separados por
asset, preservando regras de opacidade/oclusão e a ordem anterior.

Mudar a lista de pacotes ou o stream concatenado não limpa todo o cache. Uma
mudança de catálogo conserva só fontes/representações ainda referenciadas e
slots full64 válidos. Saída libera recursos não referenciados; o wgpu conserva
a propriedade necessária dos recursos em submissões em voo. Variantes de
representação/layout de assets ainda ativos podem ficar residentes. Não é um
LRU com orçamento global de VRAM nem upload assíncrono.

Não promover únicos detalhados para indexação automaticamente: permanece a
política anterior que conservou pixels nas superfícies curvas. Elegibilidade
24/48/64B e edições só de material continuam sendo reavaliadas.

## Provas e custo

`tests/render_residency.rs`, GPU explícita: rebase/rotação, entrada de um asset,
saída, reordenação, troca de shading, geometria, cor/material, GeometryArc comum
a materiais distintos e reciclagem de nome/slot. São22comparações RGB8 exatas
entre cache quente e upload frio, alternando resize e layouts compactos/full64.
Cada sujeito relevante precisa contribuir mais de100pixels em seu enquadramento.
Materiais são realmente diferentes e a edição muda valores. A revisão nanites
mostrou que a fixture inicial de16pares usava malhas pequenas demais e copiava
Default sobre Default; esses resultados anteriores não provavam todos os casos
visuais e foram substituídos por este gate.

No compacto: rebase/saída/reordenação/material não enviam geometria nova;
entrada/shading/geometria afetam um recurso cada. O oráculo força limpeza só
no renderer de referência, sem mudar sua política final. Normais derivadas
planetárias têm identidade e lifetime Arc próprios; sua edição visível troca
apenas o payload afetado, com RGB8 igual ao upload frio.

Regressões executadas:7mesh_resources (incluindo GPU/objetos/sombras/seleção/
materiais/paginação) +1render_occlusion +1render_residency, todos aprovados.
Near0,05/far30000 e IGL/ICB permanecem como antes. Sem mediçãoFPS.

Checklist estático: transforms são trabalho linear em instâncias, sem varrer
vértices/alocar novos streams no caso compatível. Entrada/saída ainda reconstrói
o stream concatenado **CPU**, mas não reenvia geometria GPU dos sobreviventes.
Mapas de identidade/representação são recompostos em mudança de catálogo,
independentes da mera mudança de origem. Memória dos recursos acompanha o
catálogo vivo e variantes retidas; não cresce com o histórico de tiles saídos.
