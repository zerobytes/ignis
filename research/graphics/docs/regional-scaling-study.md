# Regiões e projeção na curva de densidade

EAF-86. Continuação isolada de EAF-84. Protocolo prévio: mesma fonte1×/2×/10×/100×, duas famílias, câmera14→224→14 e3456×2234/1728×1117. Gates mantidos: RMSE foreground≤0,005, p99RGB≤0,02, cobertura divergente≤0,05%. Não usar contagem total de triângulos como garantia de erro subpixel.

## Revisão antes de implementar

Preparar64regiões (8×8) por superfície. Fonte e níveis globais antigos continuam como controles. Cada região mantém a polilinha fonte completa na fronteira; triângulos grosseiros adjacentes à fronteira viram fans com centro no plano grosseiro. Correção dos vértices de fronteira≤erro original, interpolação convexa da correção≤erro; portanto erro do fan≤2×erro original. O bake final aperta esse bound por triângulo: se fonte−plano está em[a,b] e fan−plano em[c,d], fonte−fan está em[a−d,b−c]. Extremos da primeira diferença estão nos vértices fonte da triangulação aninhada, os da segunda nos vértices do fan. Centro fica no plano, correção zero. Interior conserva triangulação aninhada. Memória e custo da borda crescem com densidade: compromisso experimental explícito, não solução final de streaming.

Para p em view space e deslocamento vertical δa, diferença projetada = fδ(a.xy*p.z-p.xy*a.z)/(p.z*(p.z+δa.z)). Numerador afim em p: máximo da norma na caixa ocorre nos cantos. Ambos endpoints dentro da caixa de posições fonte; denominador≥profundidade_mínima². Caixa cruzando near-plane conserva fonte. Usar0,15px inalterado. Seleção consulta somente metadados;64draws no máximo, scratch fixo na stack, sem scan de fonte/alocação/I/O no frame.

Quatro controles: full, global com esfera anterior + normais fonte, global com caixa/projeção precisa + normais fonte, regional com caixas precisas + normais fonte. Mesmo shader/material/luz; preservar campo normal via textura não comprova iluminação arbitrária ou referência integrada. CPU de seleção/codificação e todos os draws entram na vazão. Sem cache de escolha dependente da câmera, dependência/thread/contrato novo.

Medir quatro densidades simultaneamente residentes,32slots duplicados,12rodadas e128submissões por bloco. Capturas/readback fora da medida. Comparar100/1,100/10 e maior/menor de toda curva; publicar falhas e overhead regional. Comparação de referência própria por densidade, não contra1×. Capturas offscreen e métricas não equivalem a aceite humano.


Índices são agrupados por nível na preparação, permitindo coalescer regiões consecutivas que escolhem o mesmo nível sem upload/reconstrução dos índices por frame. O fator projetivo é calculado uma vez por região e multiplicado pelos erros dos níveis. Este protótipo é heightfield de deslocamento vertical, não simplificador de malha arbitrária.

Amostras da curva em potências de2 podem ocultar a seleção mista. Uma verificação adicional procura posições intermediárias14…56 em incrementos0,5, selecionando até três poses por resolução com mais de um trecho desenhado e separação≥5unidades. Essa escolha usa exclusivamente a seleção geométrica, sem consultar qualidade/tempo. Comparar RGB/cobertura contra full; não incluir essas capturas como medições cronometradas nem alegar estabilidade temporal contínua.

## Reprodução

```sh
cargo run --locked --release --example appearance_probe -- --bake-regional-scaling
cargo run --locked --release --example appearance_probe -- --regional-scaling
cargo run --locked --release --example appearance_probe -- --regional-transitions
cargo run --locked --release --example appearance_probe -- --confirm-regional-scaling
python3 scripts/check_regional_scaling.py
python3 scripts/report_regional_scaling.py
node scripts/verify_triangle_scaling.mjs artifacts/appearance-probe/regional-scaling
```

Artefatos correntes em `artifacts/appearance-probe/regional-scaling/`, controles anteriores preservados no diretóriotriangle-scaling. A preparação inicial com2×máximo regional foi substituída pelo bound assinado antes da medição final; não mistura resultados das duas implementações. Tempos incluem somente a implementação final. Nenhum contrato da engine funcional foi modificado.

## Resultado medido

Apple M4 Pro/Metal.20coortes com quatro densidades, controles duplicados e blocos128; sem compilação/regressão concorrente desta tarefa. A compilação observada da worktree paralela terminou antes de iniciar o benchmark; outras atividades do desktop não são controladas. Dados brutos conservam medianas/p90 e controles. Não são intervalos de confiança nem FPS de jogo.

Custo CPU+GPU sobrepostas, ms/frame, família com relevo, caminho regional:

| Resolução | Distância | 1× | 2× | 10× | 100× | 100/1 | Maior/menor | 100/10 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| 3456×2234 | 14 | 0.2389 | 0.2600 | 0.4599 | 0.9122 | 3.819× | 3.819× | 1.984× |
| 3456×2234 | 28 | 0.1898 | 0.2106 | 0.3988 | 0.3869 | 2.038× | 2.101× | 0.970× |
| 3456×2234 | 56 | 0.1856 | 0.1991 | 0.2154 | 0.2636 | 1.420× | 1.420× | 1.224× |
| 3456×2234 | 112 | 0.1673 | 0.1745 | 0.1786 | 0.1958 | 1.170× | 1.170× | 1.096× |
| 3456×2234 | 224 | 0.1663 | 0.1766 | 0.1781 | 0.1787 | 1.075× | 1.075× | 1.004× |
| 1728×1117 | 14 | 0.0965 | 0.1125 | 0.2935 | 0.2765 | 2.866× | 3.042× | 0.942× |
| 1728×1117 | 28 | 0.0783 | 0.0973 | 0.1085 | 0.1529 | 1.955× | 1.955× | 1.410× |
| 1728×1117 | 56 | 0.0675 | 0.0699 | 0.0789 | 0.0950 | 1.407× | 1.407× | 1.203× |
| 1728×1117 | 112 | 0.0681 | 0.0692 | 0.0673 | 0.0799 | 1.172× | 1.186× | 1.186× |
| 1728×1117 | 224 | 0.0724 | 0.0768 | 0.0930 | 0.0944 | 1.303× | 1.303× | 1.015× |

A câmera próxima melhorou substancialmente, mas a curva inteira ainda não é constante. Em3456×2234/distância14,100× custa0,912ms versus2,118ms full (−56,9%) e2,161ms do seletor anterior (−57,8%). A geometria cai3.276.800→880.384triângulos. Mesmo assim,1×→100× custa3,82× no total e4,33× no passeGPU. Na resolução menor/distância14,100× cai1,448→0,277ms (−80,9%),255.744triângulos, porém a curva inteira ainda cresce2,87×.

Há um platô local10×→100× na distância28 da resolução maior:0,399→0,387ms (−3,0%),GPU0,350→0,327ms. Isso é uma posição quatro vezes mais próxima que o platô observado em112 no experimento anterior. Na resolução menor,14 também fica próxima entre10× e100× (0,293→0,277ms). **Não há faixa contínua de estabilidade:** na resolução maior/distância56,10×→100× cresce22,4% no total; na menor/distância28 cresce41,0%. Nesses casos, as medianas dos controles duplicados devem ser consultadas antes de atribuir tudo a ruído. Não escolher apenas as extremidades ou a distância favorável para declarar independência da fonte.

O global preciso explica parte importante do ganho. Em3456×2234/distância56,100× custa0,513ms pelo global antigo,0,306ms pelo global preciso e0,264ms pelo regional. Na distância112, global preciso e regional ficam próximos (0,197/0,196ms). Na resolução menor/distância112, o regional custa0,080ms contra0,070ms do anterior: regressão de13,6%. Em objetos planos a região também impõe custo desnecessário frente a poucos triângulos globais. O seletor regional puro desta ablação não é automaticamente a melhor escolha em todas as escalas.

GPU e vazão não são a mesma medição: o primeiro usa passes isolados/timestamps; a segunda inclui preparação e submissões sobrepostas em blocos. Carga e clock podem diferir. A curvaGPU também não é universalmente plana: no caso1728×1117/distância112, o regional cresce56,5% emGPU de1× a100×, apesar do crescimento total de17,3%. Esses dados não identificam isoladamente o gargalo de memória/textura; normal source, fronteiras e seleção permanecem candidatos para estudo controlado, não causas provadas.

## Qualidade e transições

Regional passou144/144poses da curva e6/6poses intermediárias. Zero pixels de cobertura divergente nas150comparações; RMSE máximo0,003527, p99máximo0,016645, máximopixelRGB0,026765, mantendo gates0,005/0,02/0,0005. Valores não são zero: não declarar imagem numericamente idêntica. A referência ainda é rasterização de amostra única, com aliasing herdado; não é referência integrada/supersampled.

As posições em potências de2 escolhem o mesmo nível nas64regiões e coalescem em um draw. Isso não demonstra seleção mista sozinho. As seis posições adicionais exigem3…28trechos; exemplo3456×2234/distância45 usa26trechos/151.584triângulos e RMSE0,000371, cobertura zero. Perto, distância15, há3trechos/860.864triângulos, RMSE0,0000347. Inspeção de recortes físicos da referência/regional/diferença amplificada não mostrou frestas nas bordas examinadas; isso não substitui aceite visual humano ou teste de vídeo contínuo.

Os controles mantêm falhas explícitas: global antigo falha3/144 (plano10× e relevo10×/100× em1728×1117/distância224), global preciso falha1/144 (plano10× no mesmo extremo). São pixels únicos de cobertura (~0,056%, acima do gate0,05%); não relaxamos o critério. Não atribuir a causa exata exclusivamente a arredondamento sem isolá-la.

Ida e volta reproduzem os mesmos PNGs por representação. O diagnóstico temporal usa residual em coordenadas da tela, sem reprojeção; não prova ausência de popping em toda câmera ou estabilidade entre cada par contínuo de distâncias. Transições não foram cronometradas separadamente; a curva publicada mede as cinco distâncias fixas.

## Memória, validação e próximo limite

O comparador100× mantém216,38MiB de payload de assets naGPU, incluindo as duas famílias de níveis (globais e regionais) e uma textura de normais; não são216MiB de nova aparência apenas. Geometria global de controle ocupa100,12MiB; o subconjunto regional+normal soma116,26MiB. Essa separação é contabilidade do bake, não uma segunda medição de um runtime enxuto: todos os controles permaneceram residentes durante o benchmark. Textura normal source sozinha25,04MiB. Targets/readback/CPU/driver são adicionais. O build do relevo100× mediu643ms nesta rodada, incluindo escrita no cache doSO, semfsync, sob carga externa durante preparação. Não comparar essa duração como regressão controlada com a rodada anterior.

Validação:628testes Rust release/all-targets, Clippy/fmt, regressões render/replay_inspection/selection headless,588PNGs com CRC/descompressão/dimensões e reverso idêntico,432combinações do JavaScript com curvas/DPR/recortes/playback. HarnessDOM e capturasGPU não estabelecem apresentação nativa ou aceite humano. Nenhuma janela da demo funcional foi manipulada.

Próximo problema concreto: escolher entre resumo global e regiões quando o global domina em triângulos/draws, e reduzir o custo das fronteiras/campo de normais preservando seus erros. Esta rodada mede a ablação regional pura; não integra essa política híbrida nem streaming, mip/compressão, seleçãoGPU, superfícies deformáveis, materiais arbitrários, sombras ouGI. O objetivo de custo quase independente de densidade para qualquer câmera permanece aberto.


Continuação EAF-88: [seleção híbrida global/regional](hybrid-scaling-study.md) prepara um global com contorno preservado e compara custos, mantendo estes resultados como controle.
