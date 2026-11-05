# Experimento de aparência agregada

Continuação: [seleção adaptativa e câmera bidirecional — EAF-64](adaptive-appearance-probe.md).

## Escopo aprovado

Experimento isolado de pesquisa gráfica, autorizado em 25/10/2026. A Ignis é AI first, com operações humanas equivalentes na UI como direção de produto. O experimento não é um novo controle de jogo/editor, nem escolhe um formato de Scene/RenderFrame/asset. Integração futura deve usar o dispatcher compartilhado e MCP. A relação entre aparência e corpo físico permanece aberta para discussão.

Hipótese: substituir microgeometria subpixel por resumos de distribuição de normais e material pode preservar aparência melhor que uma superfície plana, com custo de desenho independente do número original de faces dentro de um recorte controlado. A preparação continua linear na geometria; não se presume ganho em atualização dinâmica, visibilidade, sombras, iluminação global ou cenas arbitrárias.

Worktree exclusiva `codex/graphics-research`, base local `d018aa6`. A frente funcional segue independente. Não modificar Scene, Command, RenderFrame ou versões neste experimento.

## Desenho experimental antes da implementação

- Superfície procedural de pirâmides triangulares, semente fixa, dimensões físicas constantes, microaltura proporcional à célula; mais células significam mais detalhe e não apenas tesselação redundante.
- Referência: geometria real rasterizada com profundidade, supersampling e média em espaço linear. Checar convergência contra resolução maior; referência numérica tem erro residual.
- Controles: plano com material médio e plano com média normal; candidato: distribuição direcional limitada por região. Resumos de material correlacionados com orientação.
- Câmera perspectiva, luz direcional e modelo de resposta difusa + brilho controlado compartilhado. Sem sombras lançadas nem GI. Visibilidade de câmera real na referência; agregado não resolve auto-oclusão, hipótese deliberadamente testada em ângulo rasante.
- Casos: contagem de faces, distância, inclinação, direção da luz, brilho e sequência de movimento. Relatar erro linear sobre superfície/contorno, erro temporal residual, custo GPU quando timestamps disponíveis, memória explícita dos buffers e preparação CPU.
- Custo do agregado inclui atualização de resposta à luz/câmera; capturas/readback/PNG ficam fora do tempo GPU medido. Mesma resolução e filtro entre métodos; referência supersampled mais cara não serve como baseline de velocidade em resolução nativa.
- Uma família de resumos fixos é prova inicial, não hierarquia adaptativa, culling, streaming, Nanite ou Lumen. Comparação de qualidade/custo dentro do experimento, sem alegação de superioridade comercial.

## Revisão estática prévia (checklist de performance.md)

1. Preparação por malha, resposta por configuração, raster por frame; nenhum loop da simulação alterado.
2. Malha detalhada e resumos construídos uma vez por densidade; reaproveitar buffers/targets por tamanho. Uma densidade residente por vez.
3. Dados GPU POD, buffers explícitos; somas de preparação em f64 onde pertinente; pixels lineares f32.
4. Resumos pré-agregam faces; medição separa preparação, upload e desenho.
5. Limites finitos de densidade, regiões, bins, resolução e frames; sem bilhões alocados para demonstrar princípio.
6. GPU offscreen, uploads fora do tempo de desenho; readback somente para avaliação/imagens. Timestamps opcionais e ausência explicitada.
7. Uma fila GPU e processamento sequencial; sem acesso ao processo/janela da demo.
8. Recursos reconstruídos ao trocar densidade/resolução, paths estáveis em artifacts/appearance-probe; um build target próprio.
9. Testar identidade difusa restrita, agregação/pesos e métricas; comparar referências convergentes, incluir casos de falha, não ajustar limiar para aprovar hipótese.
10. Captura e métricas offline custosas são intencionais. Sem novo backend/dependência/formato de runtime. Custos e limitações finais serão registrados após execução.

## Referências

- [Nanite, SIGGRAPH 2021](https://advances.realtimerendering.com/s2021/Karis_Nanite_SIGGRAPH_Advances_2021_final.pdf): hierarquia e dificuldade de erro geométrico versus aparência.
- [LEAN Mapping](https://userpages.cs.umbc.edu/olano/papers/lean/): filtragem de distribuição de normais.
- [SGGX](https://research.nvidia.com/sites/default/files/pubs/2015-08_The-SGGX-microflake/sggx.pdf): representações agregadas de microestrutura.
- [PBRT: reconstrução](https://www.pbr-book.org/4ed/Sampling_and_Reconstruction/Image_Reconstruction): filtragem e referência por integração de amostras.

## Modelo e implementação
+
+Executar `cargo run --locked --release --example appearance_probe`, depois `python3 scripts/report_appearance_probe.py`. Outputs correntes em `artifacts/appearance-probe/`: JSON, CSV, HTML interativo offline, capturas e mapas de erro. Pillow opcional acrescenta `comparison.png`; HTML/CSV usam apenas a biblioteca padrão Python. `--output PATH` permite trocar o destino do executável; passar o mesmo caminho ao gerador de relatório. Não cria janela.
+
+A malha tem 32²,128² ou512² pirâmides, quatro triângulos por pirâmide: 4.096,65.536 e1.048.576 faces. Extensão[-1,1]², microalturas aleatórias determinísticas de0,15..1,4 vezes a meia célula. Mais densidade adiciona microestrutura e reduz seu tamanho; não é comparação do mesmo objeto apenas retesselado. Cores têm um padrão macro8×8 e correlação com inclinação. Isso favorece a representação por regiões e deve ser lembrado ao generalizar.
+
+Os proxies são64 regiões de dois triângulos cada, no plano base. `geometry.rs` acumula, por região e bin direcional, área, primeiro momento da normal e cor ponderada por área. Azimute8×elevação4 ou16, com32/128 slots; as faces de pirâmides ocupam somente parte dos bins. Soma em f64, GPU em f32. Não se normaliza o primeiro momento armazenado; seu comprimento conserva informação. A normal usada para avaliar brilho é a direção normalizada do momento, uma aproximação dentro de cada bin. Material médio é conjunto por bin, mas não conserva toda correlação espacial/direcional.
+
+Para um pixel e superfícies sem transparência, uma expressão útil é:
+
+`C = sum_i w_i(v,p) * response(n_i, material_i, light, v)`.
+
+O peso verdadeiro inclui cobertura projetada e máscara de visibilidade. O candidato substitui-o por `area_bin * max(dot(mean_normal_bin, view),0)` e normaliza a soma. Esta área projetada ignora ocultação entre microfaces e variação da vista dentro do pixel/região. Avaliar o produto de médias também não preserva automaticamente a média do produto. A identidade difusa de primeiro momento só é exata com pesos fixos conhecidos, material uniforme e produto normal-luz não negativo; os testes incluem seu contraexemplo ao normalizar o momento ou recortar luz negativa.
+
+`shader.wgsl` usa a mesma resposta para malha e proxies: `color*(0.03 + max(n·l,0)) + specular*max(n·h,0)^exponent*max(n·l,0)`. É uma resposta de laboratório, não BRDF fisicamente normalizada. Luz e direção de observação para shading são uniformes; a câmera de rasterização é perspectiva45°. Isso torna o shading constante em cada triângulo, permitindo avaliação no vertex shader, inclusive das distribuições. O custo do candidato inclui essa avaliação32/128 vezes para cada vértice do proxy. O shader não reaproveita uma cor previamente calculada para outra configuração.
+
+A referência rasteriza a microgeometria com depth test/backface culling; o proxy não tem sua microaltura. Sombras de luz, GI, transparência, deformação e múltiplos objetos não participam. Um resultado em rasante mistura perda de altura, cobertura, visibilidade e aproximação angular; não atribuir causalidade a um único fator.
+
+## Referência, qualidade e medição
+
+A grade regular4×4 versus8×8 apresentou aliasing forte em microestrutura repetitiva. A referência final acumula32 grades8×8 com deslocamento Halton2/3 dentro do subpixel:2.048 amostras por pixel. O shader aplica o deslocamento em clip space. O prefixo16passes é comparado com32 como diagnóstico de convergência; são estimativas correlacionadas e isso **não é um limite matemático de erro nem prova de convergência absoluta**. Usar seeds/padrões independentes e mais passes antes de reivindicar pequenos ganhos. A referência de movimento usa o mesmo procedimento.
+
+Candidatos filtrados e controle geométrico4× usam16 amostras por pixel. Cores RGBA32Float são reduzidas em espaço linear, sem clamp de radiância; display usa Reinhard+sRGB. RMSE cobre a união da cobertura de referência/candidato, separa interior com alpha>0,999 de contorno e registra erro de alpha e máximo absoluto. O fundo vazio não dilui a métrica. Os mapas visuais mostram erro absoluto RGB×8 antes do mesmo display transform.
+
+Benchmarks nativos usam256²/uma amostra em todos os métodos, aquecimento concluído e31passes GPU. Incluem shader, raster, depth e clear/store; excluem compilação, construção/upload, leitura e PNG. Captura final não entra na região cronometrada. Metal devolveu end timestamp0 ao resolver no mesmo command buffer; resolver em submissão posterior à conclusão produziu pares válidos e monotônicos. Leituras inválidas abortam; ausência do recurso devolve null. O relatório registra min/p10/mediana/p90/max, período e adaptador.
+
+Ordem de métodos rotacionada entre casos,31repetições por método, sistema compartilhado e frequência variável são limitações estatísticas. Tempos representam o passe deste laboratório, não FPS de jogo nem comparação entre engines. A qualidade da mesma execução nativa está no JSON/CSV. Não combinar tempo nativo com erro supersampled como se fossem um único ponto da curva custo/qualidade.
+
+Sequência:12frames com câmera/luz deslocadas, mantendo geometria. Métrica temporal é RMSE de `(candidato-referência)_t - (candidato-referência)_{t-1}` em coordenadas de tela, incluindo bordas; não há optical flow nem avaliação perceptual de flicker.
+
+## Revisão estática final
+
+1. **Adequado:** geração/agregação por densidade; matrizes/uniforms por configuração; shader por vértice. Não modifica caminho quente da engine.
+2. **Mantido com motivo:** uma malha por vez, três tamanhos de targets/readback reutilizados. Imagens temporárias por avaliação permitem comparação offline. Vetores da malha e buffers GPU coexistem na preparação; não declarar memória total igual ao buffer visual.
+3. **Adequado:** Vertex40bytes, Lobe32bytes POD; acumulaçãoCPU f64, execuçãoGPU/integração de imagens f32. Bins contíguos e tile numérico.
+4. **Corrigido:** distribuição independente de vista/luz preparada uma vez; shader reavalia resposta quando parâmetros mudam. Trabalho de bins repetido nos seis vértices por região é mantido pela simplicidade do ensaio; futura atualização por região poderia reduzir isso.
+5. **Limitado:** até512²células,64regiões,128bins, saída256², alvo2048² e32passes. Geração/agregação/upload continuam O(faces); desenho do proxy é constante para essa resolução/regiões fixas, não promessa universal.
+6. **Adequado:** timestamps só no passe, readback fora; PNG/JSON sob execução offline. Referência deliberadamente cara. GPU Metal validada localmente.
+7. **Adequado:** execução serial/uma fila. Esperas de conclusão garantem leitura consistente; sem locks entre simulação e experimento.
+8. **Adequado:** uploads novos por densidade, drop ao final; targets por resolução, paths estáveis sobrescritos. Uma versão corrente no target da worktree, sem compartilhar artifacts/binário com main.
+9. **Validado conforme evidência abaixo:** testes da identidade/restrições, conservação de área/material/projeção, reprodutibilidade e métricas; capturas nativas repetidas idênticas por densidade; referência com diagnóstico de convergência. Sem mudanças nos contratos da engine.
+10. **Mantido com motivo:** tiles fixos, erro de visibilidade/relief, shader simples e falta de seletor adaptativo são limites explícitos. Esses dados orientam a próxima hipótese, sem declarar método novo ou superioridade ao estado da arte.

## Resultados medidos — EAF-62
+
+Apple M4 Pro/Metal,30condições estáticas (três densidades×dez configurações),12frames de movimento. As configurações incluem uma comparação controlada de expoentes8/32/96 à mesma distância/luz/vista, além de casos próximos, distantes, inclinados e rasantes. Referência final:2.048 amostras/pixel; candidatos filtrados16; benchmark nativo1. A rodada final levou aproximadamente19s após compilação, sem janela.
+
+| Caso | RMSE plano | RMSE distribuição32 | RMSE distribuição128 | Diferença ref.16/32passes |
+|---|---:|---:|---:|---:|
+| 4.096faces, perto, difuso | 0,089330 | 0,056865 | 0,056851 | 0,000274 |
+| 1.048.576faces, longe, brilho96 | 0,123314 | 0,016092 | 0,015534 | 0,002976 |
+| 1.048.576faces, longe, brilho8 | 0,987183 | 0,045643 | 0,040990 | 0,007007 |
+| 1.048.576faces, vista75° | 0,083459 | 0,022167 | 0,023871 | 0,001397 |
+
+No caso distante/brilho96, distribuição32 reduz o RMSE em aproximadamente87% contra o plano; isso não é87% de melhoria perceptual nem igualdade com a referência. Os resultados visuais se repetiram numericamente entre rodadas. A superfície próxima conserva microdetalhe na referência que ambos os proxies apagam; a hipótese de substituição universal é rejeitada. A hipótese restrita de preservar melhor a aparência subpixel que o plano é sustentada nesta família. Mais bins não substituem informações espaciais/visibilidade: o caso rasante128 foi ligeiramente pior que32.
+
+**Tempo medido, com escopo limitado:** no caso distante/brilho96, a rodada final de31passes deu0,697167ms de mediana para malha completa (p10–p90:0,605209–1,000625ms),0,017917ms para distribuição32 (0,017709–0,018542ms) e0,045459ms para128 (0,044125–0,120958ms). São passes nativos256² no mesmo modelo simples, com RMSE nativo respectivamente0,216992 /0,047807 /0,046942 contra a referência integrada. A malha nativa sofre aliasing; não foi comparada uma implementação de produção com antialiasing avançado. Rodadas iniciais de5passes tiveram valores e outliers de vários milissegundos inclusive no plano; a amostragem foi ampliada e a ordem rotacionada. As faixas e a variabilidade entre execuções impedem tratar uma razão específica como ganho estável de FPS. Nenhum processo paralelo ou demo do usuário foi interrompido para isolar carga.
+
+**Memória de representação:** malha detalhada não indexada120MiB; proxy15KiB e distribuição32 de64KiB (79KiB juntos), ou128 de256KiB. Referência não indexada repete vértices: não é baseline ótimo de compressão/indexação. O processo do experimento mantém malha de referência e proxies juntos; esses números não significam VRAM total reduzida para79KiB. Geração15,80ms, agregação de ambos os resumos15,90ms e upload+espera de todos os buffers21,46ms na densidade máxima. Preparação/armazenamento originais continuam proporcionais à geometria.
+
+**Próxima hipótese sugerida:** seleção adaptativa por tamanho projetado e variação de resposta, com geometria de fallback nas regiões ainda resolvidas. Depois testar representação de visibilidade/altura e famílias sem padrão de regiões coincidente, especialmente silhuetas finas e camadas. A pesquisa não decidiu ainda estrutura de assets, hierarquia de runtime, API de materiais ou pipeline de GI.
+
+## Validação e entrega
+
+- `cargo test --locked --release --all-targets`:586 testes aprovados, incluindo3novos do experimento; a rodada posterior de refinamento da coleta repetiu os3testes do exemplo.
+- `cargo clippy --locked --release --all-targets -- -D warnings` e `cargo fmt --all -- --check`: aprovados. O aviso de incompatibilidade futura de `block0.1.6` vem de dependência existente, não é aviso de Clippy introduzido.
+- `python3 scripts/run_verifications.py --only render replay_inspection selection`:3cenários/13grupos aprovados, headless, nesta worktree. O runtime permanece na base funcionald018aa6; a regressão funcional completa não foi repetida porque nenhum módulo/contrato da engine mudou.
+- Capturas nativas repetidas idênticas em cada densidade; pixels finitos e referência não vazia; sequência e métricas salvas. HTML/CSV gerados, sintaxe JS e recursos locais conferidos; contact sheet inspecionada visualmente. A abertura automática de `file://` no navegador foi bloqueada por política de URL; interação do HTML não foi validada em navegador.
+- Outputs correntes exclusivos em `target/` e `artifacts/` desta worktree. Arquivos de tentativas são substituídos nos mesmos caminhos. Nenhuma instalação, alteração da demo, push ou publicação. EAF-62 permanece In Progress para revisão humana dos resultados.
