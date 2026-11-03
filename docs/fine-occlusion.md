# Oclusão em duas escalas — EAF-181

Continuidade de [adaptive-occluders.md](adaptive-occluders.md). A representação da superfície e o depth do frame corrente são os mesmos; a malha fonte e a física não são alteradas. Esta é uma otimização de visibilidade, separada do LOD geométrico experimental.

## Critério e conservação

O pré-passe produz a profundidade corrente completa. Uma redução conserva o mínimo da profundidade reversa em blocos de4×4 pixels físicos. Uma segunda redução de 4×4 desses blocos recupera exatamente o mínimo do bloco original de16×16 pixels. O mínimo é associativo e não há interpolação de profundidade, média ou cobertura inventada.

Um candidato pequeno usa os blocos de 4 pixels quando sua caixa projetada, incluindo a margem existente, ocupa no máximo 9×9 blocos. Candidatos maiores usam o nível original de 16 pixels; os demais casos conservam o fallback. Cobertura parcial, céu ou pixel sem oclusor ainda produzem zero e impedem descarte. Permanecem os guardas de projeção/plano próximo e de empates do depth convencional.

A escala fina só fica ativa quando o trabalho de triângulos dos candidatos supera 32 triângulos por pixel do framebuffer. É uma heurística de rentabilidade, não um limite de objetos nem prova de ganho universal. O controle `Renderer::set_fine_occlusion(false)` é diagnóstico A/B; não é configuração autoral. Estado e bytes são observáveis no mesmo `renderer.info`/`InspectViewportPerformance`.

## Revisão de custo

A imagem adicional é R32Float, com um valor para cada bloco de 4 pixels. Os buffers de instâncias, jobs, bounds, indiretos e geometria são os mesmos. O cache retém a imagem entre frames e também durante o A/B desligado. Não há readback nas decisões do frame, hash por instância, serialização ou novo formato persistido.

A redução fina lê 16 profundidades por bloco em vez de 256; o número total de leituras da imagem completa é semelhante. Acrescenta escrita/leitura do nível fino e um passe para o nível grosso. A consulta de candidatos continua limitada em tamanho e a compactação preserva a ordem original. CPU e espera de GPU precisam ser medidos em conjunto, sem converter contagem de triângulos em FPS.

## Evidências

O exemplo `automatic_lod_cost --world-player-view --fine-occlusion` gera evidências em `artifacts/fine-occlusion`: seis vistas/resoluções estáticas, oito poses com câmera/luz/objetos móveis e comparação com geometria completa. A comparação de tempo usa blocos alternados aquecidos, sem PNG ou readback cronometrados. FPS da janela nativa é uma medição separada.

## Resultado medido

Apple M4 Pro, macOS/Metal, build `aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|06d221e5584e0984`. 16 amostras por variante, em blocos alternados aquecidos; o restante do renderer é idêntico, com orçamento adaptativo de oclusores ativo nas duas variantes. Nenhum LOD geométrico novo foi selecionado nesta floresta.

| Vista | Pixels físicos | 16 px ms | 4+16 px ms | Redução de tempo | Tri compartilhados mantidos antes→depois | RGB8 máx |
|---|---:|---:|---:|---:|---:|---:|
| Primeira pessoa | 1280×800 | 48.591 | 44.751 | 7.9% | 74,305,776→56,899,216 | 0 |
| Primeira pessoa | 1728×1117 | 49.768 | 47.259 | 5.0% | 64,488,836→54,976,652 | 0 |
| Aérea média | 1280×800 | 153.928 | 148.623 | 3.4% | 386,578,128→365,924,060 | 0 |
| Aérea média | 1728×1117 | 155.328 | 152.637 | 1.7% | 374,581,580→360,320,280 | 0 |
| Aérea ampla | 1280×800 | 214.727 | 210.815 | 1.8% | 575,729,800→558,563,968 | 0 |
| Aérea ampla | 1728×1117 | 217.054 | 212.053 | 2.3% | 570,385,052→551,828,224 | 0 |

As seis estáticas são idênticas. Nas oito poses móveis, sete foram exatas e uma pose sob a superfície variou 1 unidade em 1 pixel, dentro da variação observada nas seis repetições da própria referência sem oclusão. Nenhum pixel otimizado ficou fora desse intervalo. Isso não é certificado universal de igualdade visual; o arquivo `motion.json` registra exatidão e repetibilidade separadamente.

CPU submit ficou aproximadamente 9,3 ms perto e 13,3–14,5 ms nas vistas aéreas, praticamente igual entre variantes. Draw+espera inclui o trabalho adicional dos dois níveis e exclui PNG/readback/UI/simulação; não é GPU timestamp nem FPS de gameplay. O contador de triângulos acima exclui geometria legada e pré-passe, mas seus custos entram no tempo total.

A textura adicional consome 256.000 B em 1280×800 e 483.840 B em 1728×1117 (aproximadamente 0,24/0,46 MiB). Ela fica no cache também quando o A/B volta para 16 px; por isso as duas variantes do JSON mostram o mesmo total residente aquecido. Isso não significa aumento de toda a memória do jogo por essa proporção. Em 2200×1520 a imagem fina requer 836.000 B, aproximadamente 0,80 MiB.

O ganho adicional de tempo foi de 1,7–7,9% nestes casos, sobre o renderer que já tinha a melhoria anterior. O próximo gargalo continua sendo a geometria mantida por bounds inteiros e a aparência das pequenas superfícies. O LOD geométrico experimental permanece desligado por padrão.
