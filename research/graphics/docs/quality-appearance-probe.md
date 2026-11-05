# Qualidade, custo e transição de resumos

Incremento experimental na worktree gráfica. A solicitação autoriza um controle de fidelidade/custo e investigação das faixas verticais. Não altera contratos de gameplay, cena ou MCP. Uma futura configuração da engine terá operação compartilhada entre UI e MCP; o controle atual seleciona capturas reproduzíveis do laboratório.

## Decisão e revisão antes do código

A seleção antiga aplica um resumo a uma região que ainda ocupa muitos pixels, criando fronteiras largas de aparência. A hierarquia espacial elimina essa unidade fixa, mas uma decisão binária na borda de cada pixel ainda pode trocar bruscamente. Testaremos convergência contínua de posição e radiância antes da redução de triângulos, comparada à troca binária.

Para um nó cuja caixa inteira cabe no mesmo pixel: peso = smoothstep da extensão projetada × smoothstep da folga às bordas desse pixel. O peso zera ao tocar a borda, evitando que a condição de contenção introduza um salto. Cada descendente mistura sua radiância com a do ancestral e reduz sua altura pelo mesmo peso. Apenas quando o peso chega a1 é permitido substituir o conjunto pelo plano do nó. Na passagem de topologia, todos os descendentes já têm a posição plana e a mesma radiância. A reconstrução continua aproximando cobertura/visibilidade; continuidade não é prova de fidelidade.

Alternativas: troca binária é mais barata mas pode saltar; desenhar duas representações e fazer crossfade duplica trabalho e exige resolver cobertura; dithering exige estudar ruído/integração temporal. A convergência hierárquica mantém uma única partição e até dois draws, ao custo de consultas adicionais no shader. É uma hipótese local do experimento, sem decisão de arquitetura permanente.

Checklist prévio (docs/performance.md):
1. Seleção por pose, preparação da árvore uma vez; nenhuma mudança no tick da engine.
2. Reutilizar índices e estado por nó; não reconstruir geometria por frame.
3. IDs u32, pesos f32, estado POD de16 bytes; hierarquia limitada a8 níveis.
4. Matriz/focal uma vez por seleção; avaliação de lobos repetida por vértice mantida e medida.
5. Limite128² células/21.845 nós; não extrapolar para bilhões de triângulos.
6. Upload do estado/índices limitado, recursos residentes; capturas/readback somente offline.
7. Esperas GPU pertencem ao laboratório/medição, não ao gameplay.
8. Recalcular peso de cada nó visitado; ancestrais precedem filhos. Sem histórico que sobreviva à troca de perfil.
9. Testar contenção, partição, endpoints, retorno da câmera e paridade dos controles existentes; comparar erro e custo na mesma pose.
10. Geometria + resumos residentes, plano aproximado, shader mais caro e filtro comum128spp mantidos. Ganho de FPS não presumido.

## Regra implementada

Para a projeção conservadora do volume completo de um nó, `s` é a maior extensão em pixels, `c` a menor folga às quatro bordas do pixel e `b` o orçamento do perfil (1,0 /0,75 /0,5px). Se não há contenção em um pixel, `w=0`. Dentro dele:

```
t = clamp((b-s)/(0.35*b), 0, 1)
u = clamp(c/0.06, 0, 1)
S(x) = x*x*(3-2*x)
w = S(t)*S(u)
```

O predicado exato de remoção é `s <= 0.65*b && c >= 0.06`. Fora dele, o peso é limitado abaixo de1: `smoothstep` em f32 pode arredondar para1 prematuramente. A seleção binária usa o mesmo predicado, garantindo exatamente a mesma partição/contagem nos dois métodos e isolando a contribuição da transição.

O shader começa na folha ou no resumo selecionado e caminha até oito ancestrais. Em cada nível, `R = mix(R, R_resumo, w)` e `y *= 1-w`. Isso interpola radiância e altura, não normais/material de uma BRDF física de geometria deformada. Os resumos são avaliados com a mesma resposta de iluminação do laboratório. `x,z` permanecem fixos; a base das pirâmides compartilha bordas. Não há segunda representação desenhada, dithering, crossfade de imagens ou dependência de histórico.

Um ancestral com peso1 torna todos os seus descendentes planos e com a mesma radiância em aritmética ideal. A substituição por duas faces preserva esse limite matemático; isso não implica pixels bit a bit idênticos entre triangulações na GPU.

## Protocolo e fronteira de validade

178 poses:43 ida/volta por cena difusa, brilhante e rasante, distância2,5→64→2,5; mais49 poses entre10→11→10 para observar movimento fino. Três perfis ×duas transições. Todos os candidatos usam128 amostras/pixel iguais, referência2.048. Seleção em256², sem escolher pelo tamanho dos targets de supersampling. Controles completos, resumo fixo e seleção antiga recalculados na mesma pose. Tempos GPU:31 passes nativos, aquecimento e ordem alternada; seleção CPU e validação/envio separados. Não medem o custo das imagens integradas nem FPS.

RMSE em RGB linear sobre a união das superfícies, contorno e cobertura separados. Diagnóstico de faixas: média do resíduo RGB por coluna interior, maior diferença entre colunas vizinhas (mínimo4 pixels interiores). Esse número pode piorar sem produzir uma faixa percebida e não certifica ausência de artefatos. Resíduo temporal compara erro em coordenadas de tela e inclui movimento, sem fluxo óptico. A referência ainda tem erro de convergência.

`--quality-check` mantém a geometria fina, com os mesmos pesos da representação reduzida, e compara pixels antes/depois da remoção. No teste distante35°, a rasterização das duas triangulações difere em até1/128 de cobertura em um pixel, RMSE RGB~0,00062; a75°, o resultado foi idêntico. O teste inicialmente exigia identidade quase exata e **falhou**: a continuidade matemática não elimina esse resíduo de rasterização. O ensaio agora registra essa diferença explicitamente, com guarda experimental de no máximo2/128 de cobertura, e testa separadamente que estados cíclicos/NaN são rejeitados sem alterar os pixels. A guarda não é um limite universal ou aceite visual. Cobertura estável permanece uma questão aberta.

No near endpoint, todos os pesos zeram e o caminho gradual deve recuperar os mesmos pixels do controle original; ida/volta e troca de perfil são independentes de histórico. Nenhum ganho de memória foi implementado: malha e árvore seguem residentes.

## Uso e reprodução

```
cargo run --locked --release --example appearance_probe -- --quality
python3 scripts/report_quality_appearance.py
node scripts/verify_quality_report.mjs
```

Relatório corrente: `artifacts/appearance-probe/quality/index.html`. O slider navega por três configurações medidas; a câmera reproduz poses pré-calculadas, incluindo aproximação/afastamento. Os painéis gradual e binário são o experimento novo; as faixas largas permanecem intencionalmente no controle antigo. Grade opcional mostra pixels reais da captura; recorte comum evita confundir ampliação com nova seleção. JSON/CSV, mapas de erro e `endpoints.json` permitem auditar as afirmações. Os relatórios anteriores recebem link para esta comparação.

A verificação JavaScript executa o código real do relatório com DOM mínimo e caminhos reais das imagens: mudança de cena/perfil, extremos, ida/volta, reprodução, recorte, controles e resposta assíncrona obsoleta. Isso valida estado/dados, não layout, entrada física ou interação no navegador nativo.

## Revisão final de performance

1. **Adequado:** construção e upload uma vez; `Cut::update` por pose, sem tocar a simulação.
2. **Adequado:** capacidades de `Plan` e vetor de estados reutilizados; nenhuma reconstrução de vértices por pose. Clones de imagens somente na captura offline/ensaios.
3. **Adequado:** estados POD16bytes, pai u32 antes do filho, no máximo8 níveis; folha reusa os vértices existentes. Geometria de controle mantém IDs antigos.
4. **Mantido com motivo:** lobos32 avaliados por vértice e por ancestral com peso ativo; esse custo demonstrativo precisa ser amortizado antes da integração. Matriz/focal fora da recursão. Limpeza de21.845 pesos por pose evita estado dormente no ensaio expandido.
5. **Adequado ao recorte:**128² células; índice, profundidade e estados limitados. Validação exaustiva offline excluída do tempo de seleção.
6. **Mantido com motivo:**349.520 bytes de estado adicional GPU (341,3KiB), mais índices enviados a cada pose. Até dois draws; buffers residentes. Layout comum exige um buffer pequeno também nos controles, sem leituras nos modos anteriores.
7. **Adequado:** nenhuma thread/lock novo; esperas de readback/timestamp permanecem no laboratório. Medidas não incluem PNG/I/O.
8. **Corrigido:** pesos sem histórico; predicado de topologia compartilhado evita arredondamento antecipado para1. Estados inválidos validados antes de upload; pais ordenados impedem ciclo. Cache do relatório limitado a48 imagens.
9. **Adequado:** testes de contenção/partição/reversão/cortes iguais, equivalência near e resíduo GPU explícito; UI de relatório e CLI não introduzem comando exclusivo no jogo. Integração futura pela operação compartilhada MCP/UI.
10. **Mantido com motivo:** memória residente, seleção CPU, uploads, proxy plano e custo de shader. Menos triângulos não é sinônimo de menor custo. Visibilidade/oclusão e continuidade de cobertura ainda exigem estudo; sem ganho de FPS alegado.

## Resultado medido e decisão de continuidade

Apple M4 Pro/Metal. Médias de RMSE por pose no perfil **Mais resumos** (menor é melhor; cada cena tem sua própria escala de resposta):

| Cena | Regiões fixas antigas | Troca binária espacial | Transição gradual |
|---|---:|---:|---:|
| Difusa |0,003790|0,003557|0,003505|
| Brilhante |0,033779|0,013334|0,012945|
| Rasante |0,037464|0,024918|0,025696|
| Movimento fino brilhante |0,055168|0,013087|0,012705|

A melhora grande sobre as faixas largas é da **divisão espacial**, já iniciada em EAF-65. A transição gradual oferece melhora adicional pequena na cena brilhante (~2,9% no RMSE), mas piora a rasante (~3,1%). No movimento fino brilhante, resíduo temporal médio0,017831→0,017172 (~3,7% menor) e degrau médio entre colunas0,023729→0,022580 (~4,8% menor). Esses critérios não provam suavidade perfeita, e a montagem ainda permite ver diferenças de textura/contorno.

Custo: mediana dos tempos GPU por pose, perfil Mais resumos, binário→gradual =0,1017→0,2720ms difuso;0,1064→0,2819ms brilhante;0,0944→0,2179ms rasante;0,1063→0,2285ms no movimento fino. A transição custa cerca de2–3vezes o passe binário nesse experimento. Tempos são ruidosos e excluem seleção/upload; não são FPS. A redução de triângulos não compensou a avaliação de distribuições/ancestrais no shader. O perfil com mais resumos também não é necessariamente mais rápido que o perfil com mais geometria.

Na distância64 a35°, todos os perfis convergem para28.090 triângulos: nesse caso a folga das bordas do pixel domina o corte, portanto mover o slider pode não mudar a contagem. Na aproximação todos recuperam65.536. O controle é válido para explorar preservação de detalhe; ainda não está validado como configuração de performance do gameplay. O passo necessário antes de integração é amortizar a avaliação dos resumos e resolver melhor cobertura/visibilidade, com um orçamento que considere erro de aparência além do tamanho projetado.

Validação final:594 testes Rust release/all-targets (11 no exemplo), fmt e clippy sem warnings do projeto;3 cenários MCP headless render/replay_inspection/selection aprovados.178 poses/1.068 configurações,2.848 PNGs,1.305 pares de ida/volta idênticos (controle antigo com histerese excluído), partições iguais entre binário/gradual e contagens monotônicas por perfil. Seis ensaios GPU de troca/erros atômicos;277 PNGs do estudo inicial permaneceram byte a byte idênticos após mudança do shader.46 verificações do JavaScript real com DOM mínimo, sintaxe Python/JS e montagem PNG inspecionadas. Interação nativa do navegador e aceite visual humano permanecem separados. Logs e `validation.json` no diretório corrente; sem push, mudança de runtime ou interação com a demo funcional.

## Redução do custo

O estudo de [avaliação e reutilização dos resumos](summary-cost-probe.md) preserva corte/mistura/altura, separa custo de atualização do custo reutilizado e testa invalidação. Os tempos antigos acima permanecem como resultado do EAF-66.
