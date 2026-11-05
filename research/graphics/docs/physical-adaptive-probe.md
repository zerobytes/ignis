# Seleção subpixel na resolução física — EAF-73

## Antes de implementar

O experimento anterior força o atlas, com saída256², sem seletor geométrico. Este recorte mantém os assets e os bakes existentes e adiciona seleção por objeto. Não altera runtime, Scene ou MCP.

1. Frequência: bounds de malha calculados uma vez; oito cantos por objeto/pose, sem percorrer triângulos no seletor.
2. Alocações: seletor na stack; imagens, raster diagnóstico e JSON somente no laboratório offline.
3. Tipos: coordenadas f32, dimensões físicas explícitas; diagnóstico amostrado não é uma contagem exata de contribuições.
4. Repetição: referência supersampled e controles são trabalho deliberado de validação, fora do timing.
5. Escala: três assets, duas resoluções, trajetória finita; atlas inteiro exige fallback conservador para todo o objeto.
6. GPU/I/O: reusar atlas compactado; recorte central512² com projeção do viewport físico evita alocar uma referência fullscreen enorme. Timing é do recorte, não do jogo/fullscreen.
7. Sincronização: mesma fila/bench serial; sem thread/dependência nova. Readback só na captura/diagnóstico.
8. Invalidação: resolução e câmera entram a cada decisão; histerese nunca permite resumo com bound >=1px. Trocar asset reinicia estado.
9. Correção: testar projeção, bounds, resize, ida/volta, visibilidade e caminho efetivamente desenhado; comparar referência integrada e geometria no mesmo pixel físico.
10. Custos mantidos: duas representações residentes, limites conservadores e atlas de uma camada. Subpixel geométrico não garante fidelidade de material, silhueta ou oclusão.

## Método implementado

`physical_adaptive.rs` prepara uma caixa e a maior aresta da malha na inicialização, sem alterar ou subdividir os modelos. O seletor considera todos os triângulos, inclusive ocultos, para não ter de resolver visibilidade antes de decidir. Custa oito transformações de canto por objeto e pose; a escolha em si não aloca. Os bounds podem ser persistidos em um pipeline de build futuro; neste laboratório são calculados uma vez ao carregar.

Com altura física H, FOV vertical45°, profundidade mínima z e extremos absolutos x/y da caixa em coordenadas da câmera:

```
f = H / (2 tan(22,5°))
L = f/z * sqrt(1 + (x²+y²)/z²)
limite_aresta_px = L * maior_aresta_world
limite_filtro_px = L * (2 sqrt(2) * 3,2/128)
```

L limita a norma do Jacobiano da projeção sobre a caixa convexa. O segmento de cada aresta permanece nessa caixa; seu comprimento projetado fica limitado pelo comprimento3D vezes L. Plano próximo inválido ou não finito obriga geometria. O segundo limite representa o suporte bilinear de dois texels no plano de bake; **não** limita descontinuidades de altura, erro angular, BRDF ou silhueta.

Estado inicial geometria. Entra em resumo somente com ambos os limites <0,75px; volta à geometria se qualquer limite >=1px. Entre os dois limiares conserva a escolha anterior. Resolução maior reavalia imediatamente; histerese não conserva resumo fora do limite. Não há redução dinâmica de qualidade habilitada.

Tela principal verificada localmente por `system_profiler SPDisplaysDataType -json`:3456×2234 físicos,1728×1117 lógicos. Comparam-se **dois tamanhos de render físico**,3456×2234 e1728×1117. O segundo é resolução reduzida explícita, não uma equivalência de qualidade com o primeiro. Dimensões são fixas deste experimento, não descoberta automática da tela do leitor.

`Lab` mantém saída256² por padrão. Somente o novo modo configura recorte512² e viewport físico. A matriz do viewport completo recebe escala de recorte `(W/512,H/512,1)`; assim cada pixel do recorte coincide com o centro do viewport completo. Near1/far100000 neste modo permitem as distâncias longas, mantendo o padrão antigo. Testes comparam coordenadas físicas, não somente tamanhos de imagem. Os objetos inteiros cabem nos recortes verificados; isso não autoriza extrapolar seus tempos para fullscreen.

21 distâncias por objeto, incluindo amostras dos dois lados dos limiares em ambas as resoluções; ida e volta por objeto/resolução:84 posições. Câmera azimute37,5°/elevação55°, luz67,25°/35° fixas para isolar distância. Modelos de raio1,22 em unidades normalizadas; distâncias não são metros de assets originais.

Referência16spp, convergência64spp em12 posições. Geometria/resumo forçado/adaptativo1spp, todos na mesma projeção. O adaptativo realiza um novo desenho com a representação selecionada; não substitui a imagem depois do render. Sua saída float é comparada exatamente com o controle escolhido em todas as252 posições.

Diagnóstico CPU4×4 amostras por pixel com profundidade mantém o ID vencedor por amostra e conta IDs distintos por pixel. É um limite inferior amostrado, saturado em16 IDs/pixel; não é triângulos enviados divididos pela área, nem a contagem exata das contribuições. Inclui um teste de triângulo oculto. Faz varredura de triângulos e aloca buffers de diagnóstico somente offline; não participa da seleção nem do timing. Precisa de clipping mais geral para cenas fora desta trajetória, não é rasterizador de produção.

## Resultados — 26/10/2026

Apple M4 Pro/Metal. Primeira posição registrada de troca para resumo no afastamento:

| Objeto | Distância no render3456×2234 | Distância no render1728×1117 | Pixels ocupados amostrados, resolução maior | IDs visíveis amostrados/pixel ocupado |
|---|---:|---:|---:|---:|
| Pedra | 260,80 | 131,14 | 360 | 14,40 |
| Baú | 4663,04 | 2332,06 | 4 | 4,00 |
| Árvore | 562,22 | 281,83 | 48 | 10,13 |

Os limites projetados dobram exatamente ao dobrar a resolução linear na mesma pose; as distâncias de troca aproximadamente dobram. São posições discretas do percurso, não raízes analíticas. Perto, geometria é mantida e todos os detalhes do controle estão presentes pixel a pixel. A seleção por objeto inteiro retarda muito o baú, cuja maior aresta mede1,271unidades, contra0,05984 na pedra. Não subdividimos o baú artificialmente para fabricar vantagem.

Na primeira troca em resolução maior, RMSE RGB linear (geometria → adaptativo): pedra0,03765→0,01810; árvore0,03019→0,01592; **baú0,01798→0,03388**. Portanto, o critério espacial funciona, mas não prova fidelidade universal do atlas. O baú chega a quatro pixels ocupados e ainda pode piorar. O erro também inclui aliasing do render1spp: não atribuir toda diferença à representação. Convergência máxima16→64spp nos pontos selecionados: pedra0,00375; baú0,01240; árvore0,00615. Referência finita não é integração exata.

Os tempos medem9 blocos×32 desenhos, quatro métodos em ordem alternada, com geometria duplicada como controle. No recorte minúsculo de troca as diferenças ficaram sensíveis ao ruído do controle: por exemplo, pedra0,0675ms geometria/0,0613ms adaptativo/0,0763ms geometria-controle; árvore0,0299/0,0308/0,0360ms. **Esta rodada não estabelece um ganho estável de performance**. Não mede FPS, fullscreen, streaming, custo do seletor ou do diagnóstico. Payloads permanecem2,50+35,58MiB pedra,0,31+33,05MiB baú,1,50+18,43MiB árvore (malha+atlas), excluindo targets/driver/staging. Ambos ficam residentes para permitir aproximação.

Inspeção de PNGs perto e na primeira troca: detalhes próximos preservados; pedra resumida comparada à referência no tamanho físico registrado. Não há divisão em faixas no novo seletor. Não declarar ausência universal de artefatos do atlas ou de flicker com base em imagens estáticas: apresentação nativa do relatório e aceite humano continuam pendentes. Acesso automatizado anterior a `file://` foi bloqueado pela política do navegador; não foi contornado. O novo relatório foi validado com imagens locais e harness de DOM, não com uma navegação nativa.

## Reprodução e validação

Usar o worktree gráfico, com os assets/bakes de EAF-72 presentes:

```sh
cargo run --locked --release --example appearance_probe -- --adaptive-assets
# Python com Pillow disponível no runtime local; sem dependência nova no projeto.
python3 scripts/check_physical_adaptive.py
python3 scripts/report_physical_adaptive.py
node scripts/verify_physical_report.mjs
cargo test --locked --release --all-targets
cargo clippy --locked --release --all-targets -- -D warnings
cargo fmt --all -- --check
cargo run --locked --release --example appearance_probe -- --compressed-views-smoke
python3 scripts/run_verifications.py --only render replay_inspection selection
```

Artefato estável: `artifacts/appearance-probe/adaptive-assets/index.html`, JSON e capturas por asset, `verification.json`. Exibição1:1 define largura/altura CSS =512/DPR; ampliações2×/4× são explícitas. Resize/DPR só alteram apresentação, não falsificam um novo render. Slider reproduz decisões da trajetória registrada, inclusive histerese, e troca de resolução conserva a mesma distância. Não é renderer em tempo real no browser nem controle novo do runtime.

Validação concluída:602 testes Rust (quatro novos de projeção/bound/histerese/visibilidade), Clippy/fmt,252 posições com igualdade do desenho adaptativo,252 casos do JS entregue e12 combinações DPR/ampliação,15 poses de smoke da compactação legada, três cenários MCP/13 grupos headless. Sem interação com a demo do usuário. Capturas mantidas em um único caminho corrente; bakes anteriores são controles científicos necessários, não cópias históricas.

## Checklist final

1. **Adequado:** bounds calculados uma vez; `Bound::pixels` visita oito cantos, `Selector::update` duas medidas. Sem malha na decisão.
2. **Adequado:** estado/arrays na stack; buffers/imagens e ordenação pertencem ao relatório finito. Raster diagnóstico usa memória proporcional ao recorte4× e é explicitamente excluído do runtime medido.
3. **Adequado:** dimensões u32/f32 explícitas e seleção não finita conservadora; IDs de triângulos não confundidos com vértices/pontos.
4. **Mantido com motivo:** render de referência e controles repetidos para demonstrar escolha real. Nenhuma reconstrução de atlas por pose.
5. **Mantido com motivo:** bound inteiro conservador e finito; regiões/hierarquia necessárias para trocar partes de um objeto antes da maior face ficar subpixel, mas não implementadas implicitamente.
6. **Corrigido:** projeção agora usa pixels físicos. Atlas persistido reutilizado; default de256² preservado. Tempo de captura/diagnóstico separado.
7. **Adequado:** GPU serial, sem thread/lock/dependência novos; benchmarks e regressões executados separadamente.
8. **Corrigido:** resolução entra na seleção e no desenho; mudança força reavaliação e histerese testada. JS descarta carregamentos antigos e considera DPR no resize.
9. **Validado no recorte:** testes matemáticos, comparação float/PNG, referência integrada/convergência, controles com/sem resumo, regressão do renderer legado e MCP. Aceite humano separado.
10. **Mantido com motivo:** memória de ambas as representações, aliasing1spp, camada única, ângulos fixos e bound conservador. Sem afirmação de fidelidade perfeita ou ganho estável de GPU. Próxima investigação deve separar clusters locais e medir erro da representação, especialmente no baú, antes de otimizar seleção.
