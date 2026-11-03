# Atributos utilizados pelo material — EAF-178

Continuação de177, com a oclusão preservada nos dois lados do comparativo.
Revisão prévia: meshes compartilhados elegíveis já transportam cor/material por
instância; o stream48B ainda envia coordenadas de material e pesos de projeção
mesmo quando nenhum pattern, secondary_pattern ou detail os consome.

Hipótese: usar24B (posição e normal float32, sem quantização) nesses casos reduz
tráfego/custo sem aproximar cobertura, silhueta, cor ou luz. Seleção por propriedades
do material e invariantes dos vértices, nunca por nome, espécie ou receita.
Materiais procedurais, tintas/índices de material variáveis ou outros canais
necessários mantêm48/64B. Alterações de material invalidam o layout derivado;
fonte autoral mantém todos os dados. Sombras/oclusão precisam do stride correto.

Trabalho somente na preparação/upload imutável e no cache existente. Nenhum
novo I/O, worker, lock, formato, dependência ou alocação por triângulo em frame
estável. Índices continuam lossless por atributos consumidos, com mesma ordem.
Medir com referência177 e exigir RGB8 exato/repetibilidade antes de promover.
Aproximação coletiva de cobertura continua outra etapa; este corte não a entrega.

## Especialização do shader e revisão estática

A elegibilidade também escolhe uma constante de compilação do fragment shader.
Somente a preparação dos canais ausentes (coordenadas, ruído, detalhe e suas
derivadas) é eliminada. A mesma função conserva aritmética de normal/iluminação,
transmissão, PBR, sombra, ambiente, grade e cor atmosférica. Não duplicar um
segundo modelo de iluminação; materiais procedurais mantêm a variante original.

1. Frequência: seleção por instância no preparo existente; layout e índices
   construídos somente no upload de conteúdo imutável/invalidação.
2. Alocações: buffer temporário24B por vértice só no upload; fonte autoral
   permanece completa. Nenhum clone de cena ou geometria por frame estável.
3. Tipos: posição/normalf32, instância80B e índicesu16/u32 existentes. Sem
   quantização; representação GPU não altera formatos de autoria/save/replay.
4. Repetição: mesmo kernel de iluminação, constante de pipeline remove somente
   trabalho sem consumidor. Não introduzir shader específico para um asset.
5. Limites: páginas obedecem ao device; nenhum teto autoral novo. Materiais e
   vértices incompatíveis mantêm os caminhos48/64B; atmosfera já barata fica12B.
6. GPU/I/O: variantes24B para principal, sombra e oclusor. Sem textura, passe,
   readback, rede ou I/O adicional no frame. Contadores de layout na inspeção
   existente, acessível pelo mesmo dispatcher MCP/UI.
7. Sincronização: mesma ordem de desenhos/triângulos e compactação177; sem novos
   workers, locks ou dependências. Sombras independentes da visibilidade da câmera.
8. Invalidação: material pode mudar sem alterar vértices; comparar elegibilidade
   separadamente e reconstruir cache derivado. Toggle diagnóstico também limpa.
9. Contrato: prova crítica de troca de layout/material evita ler atributos pelo
   stride errado ou usar shader neutro em material procedural. Reutilizar gates
   existentes de compartilhamento, sombras, seleção, oclusão e ordem coplanar.
10. Custos mantidos: todos os triângulos escolhidos continuam desenhados. Não
    entrega resumo coletivo/subpixel, liberação da fonteCPU ou streaming.

## Primeiro ensaio: somente o stream

O corte48→24B sem especialização reduziu os buffers geométricos compartilhados
9.948.316→9.149.404B (-8,0% no conjunto, não metade da memória total da GPU).
Draw+espera perto71,120→70,899ms; panorama86,860→85,653ms. Variação pequena,
sem evidência de ganho relevante de FPS. RGB8 exato nas três poses/oito movimentos,
prova de materiais/luz/layout aprovada. Isso motivou eliminar também o trabalho
procedural ausente no shader, mantendo o mesmo critério de qualidade.

## Resultado com stream e especialização — 31/10/2026

M4 Pro/Metal, mesma floresta30mil,2048×1536, sem blur. Oclusão177 ativa nos
**dois** lados. Três rodadas alternadas,8warmup+16amostras por rodada; abaixo,
mediana das três medianas. Desenho+espera inclui submissãoCPU e conclusãoGPU,
não é timestampGPU isolado nem FPS de gameplay.

| Câmera | Caminho177 | + atributos/shader elegíveis | Redução do tempo |
|---|---:|---:|---:|
| Próxima | 70,700ms | 44,700ms | 36,8% |
| Panorâmica | 85,962ms | 62,805ms | 26,9% |
| Inversa/leve | 6,237ms | 6,031ms | empate prático; nenhum asset elegível |

CPU de submissão5,644→5,399ms e7,353→7,314ms: não é o ganho principal.
Buffers geométricos9.948.316→9.149.404B (-798.912B/~0,762MiB). As outras
alocações e fonteCPU permanecem; não anunciar8%de redução da memória total.
Triângulos após oclusão iguais:93.824.980 perto e152.980.609 na panorâmica.
A economia vem do custo de executar o desenho, não de apagar folhas/detalhes.

RGB8 exato/repetível nas três câmeras e oito poses com câmera/luz/resize.
Hashes dos PNGs de referência também iguais aos capturados antes da mudança
no shader, evitando comparar só dois caminhos já alterados. Prova crítica
exercita nove estados de material/layout, sombra e seleção com malhas de grade,
não árvores. Evidências em `artifacts/minimal-vertices/{cost,motion}.json`,
`baseline-hashes.json`, PNGs e logs. Captura próxima inspecionada visualmente.

Escolha automática habilitada por padrão. SetterRust é oráculo diagnóstico;
sem flag autoral nova, controles exclusivos deUI ou identificação por espécie.
`instancing.minimal_instances/minimal_triangles` descreve os componentes elegíveis
antes da oclusão, não os visíveis finais. É uma otimização de meshes estáticos
compartilhados; meshes únicos/deformados conservam o renderer anterior.

Próximo gargalo: reduzir a representação do detalhe pequeno coletivo que ainda
contribui para a imagem. Medir cobertura/contorno do conjunto em movimento antes
de substituir geometria; eliminar objetos individualmente por3px ou por uma
distância fixa não é equivalente. A contagem de triângulos restante continua
alta e não há promessa de platô universal nesta entrega.

Validação final:7testes Rust em mesh_resources/render_occlusion aprovados,
Clippy release all-targets com-Dwarnings,fmt/diff e buildrelease aprovados.
Build `44dae857d2128e92`; executável corrente reaproveitado, sem cópia histórica.
Floresta30mil carregada na janela nativa via mesmo MCP, blur desligado/otimização
ativa; inspeção confirmou11.789componentes no layout24B. Uma amostra de cadência
foi19,68FPS. Sem trajetória A/B de janela, não comparar diretamente essa amostra
com os~15FPS relatados pelo usuário nem com o ensaio2048×1536. Aceite humano da
nova versão permanece separado; nenhum novo ganho universal alegado.
