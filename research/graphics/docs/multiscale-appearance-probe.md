# Resumos multiescala de aparência — EAF-65

## Recorte autorizado e revisão prévia

Continuação do EAF-64: preservar diferenças entre pixels e só agrupar uma região quando sua projeção inteira estiver contida em um pixel da imagem final. Comparar novamente câmera afastando/aproximando, referência integrada, resumo fixo e seleção anterior. Somente laboratório; contratos de Scene/Command/RenderFrame, gameplay e MCP permanecem independentes. Relatório com controles humanos de comparação; integração futura deve expor operações equivalentes UI/MCP.

Decisão experimental: quadtree de regiões espaciais, da pirâmide individual até a superfície completa, com distribuição 32 por nó. Somar momentos por área de baixo para cima, sem média de médias. Seleção usa projeção dos oito cantos do volume inteiro, incluindo microaltura, e exige o mesmo pixel nos extremos da caixa projetada. Limiar de entrada 0,75px/histerese até 1px, subordinada à contenção: cruzar fronteira exige refinar imediatamente. Isso é mais restritivo e pode trocar com frequência; será medido. Não promete radiância/cobertura exata, nem reúne automaticamente regiões distantes que se projetem juntas.

Checklist anterior ao código: preparação O(faces + nós×32) uma vez; visita descendente limitada à árvore e memória pré-alocada por percurso. Dois index buffers reutilizados permitem até dois draws, com upload apenas dos índices selecionados por pose, sem reconstrução de malha. Esse tráfego/CPU é custo explícito do protótipo, não seleção GPU escalável pronta. Resumos e geometria continuam residentes. Matriz/focal calculados uma vez por atualização; planos anteriores retidos em buffers reutilizáveis para medir troca na mesma pose. Sem dependências, threads, locks ou formatos novos. Near-plane/fora da viewport conservadores, resolução final separada do supersampling. Testar conservação de momentos, cobertura sem duplicatas, contenção real por pixel, fronteira subpixel, resize, reversão, extremos GPU e regressões de render.

## Implementação e contrato do seletor

`hierarchy.rs` constrói uma quadtree sobre 128×128 microcélulas, com 21.845 nós. Uma folha representa a pirâmide original de quatro triângulos. Os pais representam blocos de 2×2, 4×4, 8×8, …, 128×128 células. Cada nó guarda limites XYZ e 32 grupos de orientação/material; cada proxy possui dois triângulos. Acumuladores f 64 são somados de baixo para cima antes de normalizar; momentos finais são f 32. A média não normalizada das normais conserva área projetada assinada, mas não resolve visibilidade nem toda a correlação material/normal dentro de um grupo.

Para cada pose, a projeção dos oito cantos de cada volume fornece um retângulo conservador em coordenadas dos pixels finais. Com toda a caixa à frente do plano próximo, a projeção perspectiva de uma caixa convexa tem seus extremos entre os vértices projetados. O retângulo recebe uma margem de arredondamento f 32 antes da decisão:

```
mesmo_pixel = floor(min_x) == floor(max_x)
           && floor(min_y) == floor(max_y)
           && retângulo dentro da viewport
extensão = max(max_x - min_x, max_y - min_y)

entrar no resumo: mesmo_pixel && extensão < 0,75
conservar resumo: mesmo_pixel && extensão < 1
caso contrário: visitar filhos; na folha, desenhar geometria original
```

Uma região de 0,2 pixel que atravessa uma fronteira é recusada. Histerese nunca autoriza atravessar pixels. As escolhas são próprias de cada trajetória; nós encobertos por um pai escolhido mantêm histórico, mas a condição atual é sempre reavaliada antes de voltar a desenhá-los. Near-plane ou posição fora da viewport impedem o resumo; não foi introduzido culling. Supersampling não altera a resolução de seleção.

A seleção produz uma partição da superfície: cada célula original pertence exatamente a uma folha geométrica ou a um resumo escolhido. Duas listas de índices reutilizáveis enviam somente as faces selecionadas, em até dois draws. Os buffers de geometria e resumos são estáticos durante o percurso. A atualização dos índices valida capacidades e faixas antes de escrever; o envio à fila não equivale a uma espera pela conclusão GPU.

Não há combinação automática de regiões distantes que se projetam no mesmo pixel. A hierarquia é espacial e específica desta superfície procedural. A prova de contenção é geométrica: o proxy plano ainda pode alterar área coberta, profundidade, auto-oclusão e brilho. Tampouco é um contrato de streaming, orçamento global por pixel ou seleção inteiramente na GPU.

## Comparação reproduzível

```
cargo run --locked --release --example appearance_probe -- --multiscale
python 3 scripts/report_multiscale_appearance.py
```

Saída corrente: `artifacts/appearance-probe/multiscale/index.html`, `report.json`, `results.csv`, `comparison.png` opcional e imagens. O Python usa stdlib; Pillow apenas exporta a montagem. Links para o novo recorte aparecem nos relatórios anteriores quando regenerados.

São 65 poses por cena, 195 frames e 1.365 PNGs. O trecho inicial preserva as 25 posições de distância 2,5 a 16 do experimento anterior; acrescenta 20,24,28,32,40,48,56,64 e retorna pelas mesmas poses. Há cenas difusa 35°, brilhante 35° e rasante 75°, com luz fixa e o mesmo material por percurso.

Os quatro métodos são geometria completa, resumo fixo, seleção anterior e multiescala. **Todos os candidatos agora usam 128 amostras/pixel**, oito grades 4×4 deslocadas pela mesma sequência Halton. A referência mantém 32 grades 8×8,2.048 amostras/pixel. Na primeira execução, o filtro anterior de 16 amostras deixava aliasing relevante na geometria fina retida; ampliar igualmente a integração dos candidatos reduz esse fator de confusão. Os números deste relatório não devem ser comparados diretamente com erros dos relatórios antigos de 16 amostras. O seletor anterior foi recalculado aqui com o filtro novo.

A comparação de troca usa o plano anterior e o atual na mesma câmera, com o mesmo filtro 128; a referência não participa dessa diferença. O resíduo temporal usa diferenças de erro em coordenadas de tela e inclui movimento de contorno. Não é métrica perceptual com fluxo óptico. Referências/controles são capturas separadas: seu custo não está escondido no tempo do desenho multiescala.

O relatório apresenta aparência em quatro painéis. O mapa azul/laranja fica em uma seção de diagnóstico separada. Ampliação usa exatamente o mesmo recorte em todos os painéis, com opção de mostrar a grade dos pixels existentes. Ela não aumenta a resolução do render nem muda a seleção. O último painel pode mostrar a geometria completa filtrada para revelar o erro de amostragem dos candidatos.

## Resultado observado

Na comparação brilhante de distância 10,375, a multiescala preserva a variação que o método anterior substituía por colunas uniformes. O RMSE cai de 0,046934 para 0,012859. Na distância 16, passa de 0,044911 para 0,015968. Esses resultados usam o mesmo filtro 128 em ambos os métodos.

A média aritmética do RMSE de superfície por frame, dando o mesmo peso a cada uma das 65 poses de cada cena, foi:

| Cena | Seleção anterior | Multiescala | Redução relativa dessa média |
|---|---:|---:|---:|
| Difusa 35° | 0,003765 | 0,003394 | 9,9% |
| Brilhante 35° | 0,033453 | 0,013266 | 60,3% |
| Rasante 75° | 0,037043 | 0,026025 | 29,7% |

Essas porcentagens não são percentual de fidelidade, pixels corretos ou ganho perceptual certificado. Há poses em que o resumo fixo/anterior é melhor: na cena difusa a 16, por exemplo, RMSE0,003933 contra 0,004187 da multiescala. No rasante a 10,375, o controle completo filtrado tem 0,024494 contra 0,031054 da multiescala; mesmo com contenção, o proxy não reproduz toda a visibilidade/cobertura.

A referência também tem erro residual. O maior indicador 16/32 passes foi 0,002562 na difusa,0,007576 na brilhante e 0,012008 na rasante. Diferenças pequenas próximas desse piso exigem referência mais refinada; o indicador correlacionado de convergência não é limite garantido. A média do erro do controle completo 128 foi 0,004038/0,015612/0,023521 respectivamente: preservar geometria não elimina aliasing por si só.

| Distância / cena 35° | Triângulos multiescala | Triângulos seleção anterior |
|---|---:|---:|
| 2,5, início e retorno | 65.536 | 65.536 |
| 10,375, afastando | 56.870 | 24.656 |
| 16, afastando | 47.850 | 128 |
| 64, afastando | 16.300 | 128 |

Na distância 64, o caso 35° escolheu 2.217 resumos de 1×1 célula,951 de 2×2,344 de 4×4 e 40 de 8×8;2.299 células permaneceram em geometria. A regra rígida mantém geometria nas fronteiras entre pixels mesmo muito longe. Os níveis maiores existem na árvore, mas não foram selecionados neste percurso. Na cena rasante o mínimo foi 14.496 triângulos. Todos os percursos recuperaram 65.536 faces no retorno e os extremos totalmente geométricos produziram exatamente os pixels do controle completo 128.

O maior salto de troca na mesma pose foi 0,003521/0,019947/0,036910, respectivamente difusa/brilhante/rasante. São métricas com 128 amostras; não comparar diretamente com os saltos de 16 amostras do EAF-64. A contenção por pixel pode forçar mudanças independentemente da histerese; ausência de grandes colunas uniformes não prova ausência de oscilação ou popping.

## Custo e limites

A hierarquia usa aproximadamente 21,33MiB de distribuições,5,00MiB de proxies e 2,00MiB de metadados CPU, além dos 7,50MiB de geometria e 1,25MiB de capacidade de índices GPU. A execução mantém também os buffers dos controles para comparação, targets/readbacks e memória interna do driver. Os recursos CPU e GPU não são uma única cópia; a tabela não representa RSS total. Não há economia de memória demonstrada.

Construção medida nesta execução:7,29ms. A mediana das atualizações CPU individuais por percurso foi aproximadamente 0,570/0,567/0,622ms. Seleção, validação/envio de índices e bytes enviados aparecem separadamente no relatório. Medidas individuais variam e não constituem benchmark estatístico estável.

Tempos GPU nativos têm 31 passes após aquecimento, uma amostra/pixel, ordem alternada dos quatro métodos e p 10/mediana/p 90. Não correspondem ao custo das imagens 128/2.048. Houve grande variação entre poses e métodos, inclusive controles equivalentes; não há ganho geral de FPS comprovado. A multiescala pode desenhar menos triângulos e ainda custar mais: avalia distribuição 32 em cada vértice de proxy e adiciona indexação/seleção/upload. Esses custos estão visíveis, não foram retirados do método para melhorar a comparação.

O resultado é um avanço de fidelidade espacial em um laboratório controlado. Próximas questões são cobertura/oclusão, fronteiras de pixels e amortização do custo dos resumos/seleção. O recorte não entrega Nanite, Lumen, geometria arbitrária ou bilhões de triângulos em tempo real.

## Validação e revisão estática final

- 592 testes Rust release/all-targets, incluindo 9 do exemplo;3 novos testes de conservação da árvore, fronteira de pixel/near-plane e partição/retorno/resize. Clippy com `-D warnings` e fmt aprovados. Aviso de compatibilidade futura da dependência `block 0.1.6` preexistente.
- 195 poses GPU offscreen: partição exaustiva, contenção de cada resumo e equivalência exata dos extremos com o controle completo 128 verificadas por asserts. Todos os 1.365 PNGs têm 256×256, contagens de células/faces são consistentes e poses inversas são idênticas.
- Reexecução dos controles iniciais:277 PNGs preservados byte a byte. Referência/completo/fixo também idênticos em 288 pares de poses inversas neste percurso.
- Regressões headless de render/replay_inspection/selection:3 cenários/13 grupos aprovados, sem interagir com a demo funcional.
- Sintaxe Python/JavaScript e links de imagens verificados; montagem PNG inspecionada. Controles no navegador não foram exercitados por automação; aceite visual humano permanece separado.

| Pergunta de performance | Evidência/custo mantido |
|---|---|
| Frequência | Construção/upload estáticos; seleção por pose visita até 21.845 nós. Planos e índices são reutilizados. |
| Alocações | Sem crescimento das listas na seleção dentro do limite; capturas, JSON e readbacks alocam offline. Escrita de fila wgpu pode usar staging interno. Relatório limita cache a 64 imagens. |
| Tipos/localidade | Nós e distribuições contíguos; f 64 apenas na construção dos momentos, f 32 para projeção/GPU, índices u 32. Sem unsafe ou ABI nova. |
| Repetição | Matriz/focal uma vez por seleção; momentos somados de baixo para cima. Validação de índices e partição é explícita; a partição exaustiva de diagnóstico fica fora do tempo da seleção. |
| Escala | Árvore limitada a 128×128 folhas. Enviar índices ainda custa O(faces escolhidas); não é algoritmo de submissão independente da geometria. |
| GPU/I/O | Até dois draws, buffers reutilizados; upload de índices medido. Distribuição avaliada por vértice conserva o shader de comparação, com repetição deliberada ainda a amortizar. |
| Sincronização | Nenhuma thread/lock novo; esperas GPU limitadas herdadas do laboratório. Capturas e referência são offline. |
| Invalidação | Pose/resolução refeitas a cada seleção; near-plane/fronteira de pixel exigem refinar. Cache de história é por percurso; não há edição dinâmica de assets. |
| Correção | Momentos por área, partição sem lacunas/duplicatas, fronteiras/resize e extremos GPU; erro fotométrico/cobertura continua medido e não certificado pelo seletor. |
| Custos aceitos | Mais memória e trabalho que o resumo fixo para preservar variação espacial; proxy plano, histerese discreta e CPU selection permanecem hipóteses explícitas. Sem streaming/culling/TAA/GI implícitos. |

## Continuidade

O estudo de [qualidade e transição](quality-appearance-probe.md) acrescenta três perfis subpixel, convergência de altura/radiância e comparação com troca binária. Registra explicitamente custo adicional do shader e resíduo de cobertura na troca de topologia.
