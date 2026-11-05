# Seleção adaptativa de microgeometria — EAF-64

Continuação: [resumos multiescala com contenção em um pixel](multiscale-appearance-probe.md).

## Recorte e decisão

Continuação autorizada do [experimento de aparência](appearance-probe.md): manter a geometria quando o detalhe ainda está resolvido e selecionar o resumo quando ele fica subpixel, observando a câmera nos dois sentidos. Worktree `codex/graphics-research`. O laboratório continua isolado dos contratos de Scene, Command, RenderFrame, formatos e runtime MCP; sua UI é um relatório de capturas, com seleção de cena, reprodução e controle de posições da câmera. Integração futura no editor precisa preservar operações compartilhadas UI/MCP.

Foi escolhida uma seleção discreta por região, com histerese, para medir o erro da troca antes de introduzir mistura. Alternativas: limiar único é mais simples, mas alterna perto da fronteira; crossfade pode suavizar a troca, mas desenha duas representações na faixa e precisa resolver cobertura/profundidade; dithering temporal exige reconstrução temporal ainda ausente. Nenhuma dessas alternativas foi incorporada silenciosamente ao resultado.

O tamanho projetado é um **limite geométrico**, não um certificado de fidelidade de aparência. A agregação de muitas células subpixel em uma região ainda pode perder variações espaciais, correlação de material/normal, visibilidade e brilho. [PBRT, teoria de amostragem](https://pbr-book.org/4ed/Sampling_and_Reconstruction/Sampling_Theory) explica por que sinais com descontinuidades não se tornam perfeitamente reconstruíveis apenas aumentando a frequência de amostragem. O filtro box e a referência numérica usados aqui também têm limites.

## Revisão estática antes de implementar

Aplicado o checklist de `performance.md`: seleção O(64) por frame, estado e metadados limitados; preparar limites e ordenar vértices por região uma vez, mantendo os buffers residentes. Evitar index buffer extra e reconstrução/upload de malha com movimento da câmera. Agrupar intervalos adjacentes de seleção. Uniform/readback/capturas continuam custos explícitos do laboratório. Near plane conserva geometria; dimensão final da imagem participa da seleção. Testar projeção, histerese, retorno, resize, cobertura dos intervalos e equivalência GPU nos extremos. Sem lock, dependência, thread ou formato novo. Preparação O(faces) e ambas as representações residentes são custos aceitos; nenhum ganho de FPS ou streaming presumido.

## Matemática implementada

Para uma câmera perspectiva quadrada com altura `H` e FOV vertical `θ`, a distância focal em pixels é:

```
f = H / (2 tan(θ/2))
π(x,y,z) = f (x/z, y/z), com z positivo à frente da câmera
||Jπ||₂ = (f/z) sqrt(1 + (x²+y²)/z²)
```

Cada região tem uma caixa que contém todas as suas pirâmides. Seus oito cantos transformados para câmera fornecem `z_min`, `X_max = max |x|` e `Y_max = max |y|`. O maior diâmetro da caixa de uma microcélula é `D = sqrt(2 cell² + max_height²)`. Ele inclui a altura; não é só o comprimento de uma aresta horizontal.

A região usa o limite conservador:

```
p_max = f D / z_min * sqrt(1 + (X_max² + Y_max²) / z_min²)
```

A norma do Jacobiano limita a variação da projeção ao longo de qualquer segmento dentro da caixa convexa. Consequentemente, esse valor limita a distância projetada entre pontos da mesma microcélula quando toda a caixa está à frente do plano próximo. O cálculo usa a menor profundidade da região e pode manter mais geometria que o necessário, especialmente fora do eixo. Não é simplificação por erro Hausdorff nem prova de erro fotométrico. A implementação é f 32; os testes comparam projeções diretas com tolerância numérica.

Se `z_min <= near`, manter geometria. Caso contrário:

- `p_max < 0,75`: usar resumo de 32 grupos de orientação/material.
- `p_max > 1`: usar geometria.
- Na faixa inclusiva `[0,75; 1]`: manter a escolha anterior.
- Inicialização: geometria. Recalcular o limite quando câmera/resolução muda; histórico pertence ao seletor desta trajetória, não à malha compartilhada.

A seleção sempre usa `H=256`, independentemente dos 16 ou 2.048 samples/pixel usados para integrar a imagem. Um resumo nunca permanece selecionado com limite acima de 1 pixel neste protocolo.

## Desenho efetivamente adaptativo

`geometry.rs` prepara 64 regiões contíguas de vértices completos. O resumo mantém dois triângulos por região; um bit do identificador de vértice distingue proxy de microgeometria dentro do shader do laboratório. Não há index buffer extra nem cópia por frame. Os modos legados mascaram esse bit para acessar a distribuição correta.

`selection.rs` calcula os limites a partir de uma única matriz/focal por atualização e conserva um array de 64 escolhas. Iteradores sem alocação agrupam regiões contíguas com a mesma representação. `gpu.rs` desenha somente essas faixas dos buffers completo/proxy; ambos usam profundidade, material e câmera do experimento. Assim, a contagem reportada é geometria submetida de fato, não estimativa de uma imagem completa renderizada escondida. As referências/controles completos são capturas separadas e estão fora do tempo GPU adaptativo.

Essa representação de dois níveis não é uma hierarquia geral. Não há streaming, frustum/occlusion culling adicional, reconstrução de bordas entre níveis, correção de auto-oclusão, sombras ou GI. O proxy na altura zero pode ter diferença de contorno/profundidade junto às regiões reais. A malha e os resumos continuam residentes em CPU/GPU.

## Protocolo reproduzível

```
cargo run --locked --release --example appearance_probe -- --adaptive
python 3 scripts/report_adaptive_appearance.py
```

Resultado corrente: `artifacts/appearance-probe/adaptive/index.html`, `report.json`, `results.csv` e capturas PNG. O gerador usa stdlib; Pillow é opcional para `comparison.png`. O relatório inicial ganha link para este incremento quando seus dados estão presentes. A linha de comando original e `--output PATH` continuam disponíveis; `--adaptive` usa o caminho corrente fixo acima.

- 128×128 pirâmides, 65.536 triângulos completos; 128 triângulos no resumo integral.
- Três cenas: difusa a 35°, brilhante a 35° e brilhante rasante a 75°. Luz fixa em cada percurso, expoente 96 no brilho.
- 49 posições: distância 2,5→16→2,5, mesmas poses no retorno. 147 frames; 882 PNGs para referência, completo filtrado, adaptativo, sempre resumo, seleção e erro.
- Referência: 32 grades 8×8 deslocadas por Halton, 2.048 samples/pixel. Diferença do prefixo 16 para 32 passes mede convergência, sem certificar verdade absoluta.
- Candidatos: grade 4×4, 16 samples/pixel. RGB linear nas métricas, Reinhard+sRGB só na apresentação; métricas de superfície/interior/contorno separadas.
- Troca: renderizar a escolha anterior e a nova **na mesma pose**, com o mesmo filtro 4×4; medir RMSE entre elas. Esse indicador isola a troca de seleção, mas inclui limitações da integração dos candidatos.
- Resíduo temporal: diferença dos erros em coordenadas de tela, sem compensação de movimento. Não é métrica perceptual de flicker.
- Timings: adaptativo e completo em resolução nativa,31 passes depois de warmup, com p 10/mediana/p 90; uma amostra por pixel. Não representam o custo das capturas filtradas. Ordem fixa adaptativo/completo neste percurso; carga compartilhada e ordem podem enviesar comparação. JSON inclui também erros das imagens nativas.
- Seleção CPU: uma atualização medida por frame, separada de GPU/readback/PNG; valor de diagnóstico ruidoso, não benchmark estatístico.
- Controle de oscilação:40 posições alternadas ±0,03 perto do limite 0,875 de uma região. Comparar seletor com histerese contra limiar único; contar mudanças após inicialização. Esse controle mede decisões, não renderiza uma animação de cintilação.

## Resultado observado

Em todas as cenas, o percurso começa e termina com 65.536 triângulos e chega a 128 no ponto distante. Os extremos adaptativos são exatamente iguais, em pixels lineares, aos respectivos controles completo/resumo com o mesmo filtro. A imagem completa 4×4 ainda difere da referência 2.048, particularmente no brilho; essa diferença permanece exposta no relatório.

No percurso 35°, as trocas para resumo ocorreram nas distâncias 9,8125,10,375 e 10,9375. Na volta, o retorno à geometria ocorreu em 8,125,7,5625 e 7. Na distância 10,375 da ida,24 regiões ainda mantêm geometria:24.656 triângulos. O retorno na mesma pose pode ter outra seleção dentro da faixa de histerese. As duas curvas aparecem no gráfico.

| Caso | Maior salto de troca (RMSE linear) | Maior erro adaptativo | Erro adaptativo em distância 16 | Maior residual da referência 16/32 |
|---|---:|---:|---:|---:|
| Difuso 35° | 0,013100 | 0,019346 | 0,011464 | 0,001080 |
| Brilhante 35° | 0,074155 | 0,072770 | 0,046775 | 0,003536 |
| Rasante 75° | 0,077005 | 0,083044 | 0,050817 | 0,005261 |

No controle de oscilação foram 0 trocas com histerese e 117 mudanças individuais de região com limiar único, após inicialização. Isso comprova estabilidade da decisão nesse controle, não ausência de popping: os saltos acima continuam presentes. A comparação visual ampliada mostra regiões com brilho ainda variável na referência, enquanto o resumo uniformiza sua aparência.

Conclusão do recorte: a seleção geométrica e sua reversão funcionam, a redução de desenho é real, mas o critério subpixel sozinho ainda aceita perda de aparência. O próximo problema de pesquisa é um critério adicional de erro de aparência/visibilidade e/ou resumos espacialmente mais ricos; suavizar a troca pode reduzir o salto, mas não corrige o viés do resumo. Nenhuma superioridade sobre outra engine ou ganho de FPS foi demonstrado.

## Validação e revisão estática final

- `cargo test --locked --release --all-targets`:589 testes aprovados;6 no experimento, incluindo 3 novos de projeção/resolução, histerese/near-plane/reversão e cobertura dos desenhos contíguos. Após a extração de matriz/focal invariantes, os 6 testes do exemplo foram repetidos.
- `cargo clippy --locked --release --all-targets -- -D warnings` e `cargo fmt --all -- --check`: aprovados. Aviso preexistente de compatibilidade futura de `block 0.1.6` permanece fora deste diff.
- Reexecução do ensaio inicial: 277 PNGs preservados byte a byte após reorganização por região e marcação do proxy; nenhum controle legado alterou seus pixels.
- GPU offscreen:147 frames, igualdade exata nos extremos para cada cena, retorno de geometria e controle de oscilação verificados por asserts. Relatório verifica poses inversas, contagens e nenhum resumo acima de 1px.
- `scripts/run_verifications.py --only render replay_inspection selection`:3 cenários/13 grupos aprovados, sem janela; runtime não modificado.
- Relatório: sintaxe JavaScript,882 referências de imagens e CSV/JSON verificados; montagem PNG inspecionada. Controles HTML não constituem teste de interação nativa nem aceite humano; este permanece separado.

| Checklist | Evidência e custo mantido |
|---|---|
| Frequência | Uma seleção por pose,64 regiões; geração/agregação/upload uma vez. Capturas de referência e controles são exclusivamente offline. |
| Alocações/cópias | Array fixo 64; intervalos por iterador sem Vec. PNG, imagens e JSON alocam no relatório offline; nenhuma cópia de malha na seleção. |
| Tipos/localidade | Bounds e matrizes f 32, escolhas bool, regiões/vértices contíguos; bit de proxy restrito ao exemplo. Nenhuma ABI/runtime alterada. |
| Repetição | Matriz/focal uma vez por atualização; diâmetro preparado uma vez por região; draws adjacentes agrupados. Oito cantos por região são conservadorismo mantido. |
| Escala | O(64) seleção; máximo 64 faixas de desenho para qualquer máscara. Preparação O(faces) e buffers completos residentes. Dois níveis fixos não provam bilhões de faces. |
| GPU/I/O | Buffers reutilizados; nenhuma malha enviada por troca. Readback/sincronização repetidos medem integração offline e não entram em tempo GPU; uploads, targets e PNG separados. |
| Sincronização | Sem thread/lock/dependência nova. Esperas limitadas existentes. Tempos de GPU compartilhada variam, não inferir FPS de contagem de faces. |
| Invalidação | Escolha atualizada com pose/resolução e inicializada por cena; dimensões finais independem do supersampling. Near-plane conserva geometria. Edição dinâmica de asset não implementada. |
| Correção | Limite de projeção, histerese, faixas, extremos GPU, ida/volta e regressão do renderer existente; erro de aparência medido e preservado como resultado. |
| Custos/tradeoffs | Troca discreta e proxy plano mantidos para revelar saltos; resumo 32 não preserva visibilidade/correlações completas. Sem crossfade/TAA/culling/streaming implícitos. |
