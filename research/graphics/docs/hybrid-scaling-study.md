# Seleção global/regional por custo

Protocolo prévio: manter fontes1×/2×/10×/100×, resoluções3456×2234 e1728×1117, câmera14→224→14, material/luz/campo de normais e gatesRMSE≤0,005,p99≤0,02,cobertura≤0,05% de EAF-86. Controle global preciso antigo permanece explícito, inclusive sua falha de contorno.

## Revisão antes de implementar

1. Preparar níveis globais com a mesma polilinha de contorno fonte usando o construtor de fans/intervalos regionais. Reusar vértices fonte; metadados extras fora do frame. Não considerar o global anterior elegível no híbrido porque já falhou cobertura.
2. Estimativa prévia `triângulos + 1024×draws`. Peso heurístico transparente, não medição de tempo nem promessa de mínimo real. Ambos usam o mesmo shader de normais fonte. Comparar com mínimos medidos após a execução, sem retroalimentar seletor com os vencedores do benchmark.
3. Global é um draw. Limite inferior regional preparado=sum(mínimos de triângulos das regiões). Se global já custa≤esse limite+um draw, retornar sem visitar regiões. Durante a visita, custo parcial+limite inferior restante permite interromper. Empate favorece global.
4. Scratch64 na stack, sem alocação/scan de vértices/I/O no frame; câmera/resolução reavaliadas sempre. Incluir seleção/codificação na medição. Sem cache por pose, dependência/thread/contrato novo.
5. Cinco controles full/global antigo/regional/global com contorno/híbrido.40slots duplicados,12rodadas,128frames/bloco, quatro densidades residentes alternadas; queries960 dentro da capacidade1024.
6. Comparar qualidade/híbrido contra fonte, igualdade com candidato escolhido, reverso e posições intermediárias que exercitem mudança de estratégia. Não confundir pixel subpixel com igualdade de imagem.
7. Medir GPU, vazão e CPU; publicar diferenças contra regional e contra o melhor caminho elegível, incluindo regressões e controles. Nenhum percentual pequeno é garantia de diferença estatística.
8. Manter dados anteriores; novo diretório de experimento, sem nova worktree/build. Fonte/normais e todos controles residentes crescem com densidade; targets/CPU separados. Protótipo de heightfield vertical, sem arquitetura de engine implícita.


## Decisão e implementação

O global anterior não pode entrar diretamente no híbrido: já havia falhado no critério de cobertura. O novo global usa uma única região com contorno fonte, reutilizando o construtor de fans e os intervalos de erro assinados de EAF-86. Os vértices da fonte são compartilhados com as regiões; os índices dos níveis globais preservados são adicionais. Global anterior e regional anterior continuam controles independentes.

O seletor calcula primeiro o nível global admissível pela projeção física. Seu custo estimado é `triângulos + 1024 × draws`. Cada região possui um mínimo de triângulos preparado no bake. Durante a avaliação regional, os triângulos já selecionados mais os mínimos das regiões restantes são um limite inferior; a quantidade de draws coalescidos não pode diminuir ao acrescentar regiões. Se esse limite já empata ou supera o global, a busca termina. A decisão é comparada com a busca exaustiva nos testes e nas144poses. Não há leitura de pixels, tempos medidos ou tabela de vencedores no seletor.

O peso1024 é uma heurística explícita, sem ajuste aos vencedores destas poses. O menor valor do modelo não garante o menor tempo real: localidade de índices, textura, rasterização e custos CPU/GPU não são resumidos perfeitamente por duas contagens. O relatório compara o híbrido com ambos os caminhos elegíveis medidos separadamente, inclusive quando o híbrido perde.

Quando o global preservado escolhe o nível fonte completo, a normal do vértice já contém a informação necessária: usa o caminho nativo e evita buscar/interpolar novamente a mesma normal na textura. Os controles anteriores mantêm seu shader original. O ganho desse caso inclui essa eliminação de trabalho, além da seleção híbrida; não deve ser atribuído somente ao modelo de custo.

## Reprodução

```sh
cargo run --locked --release --example appearance_probe -- --bake-hybrid-scaling
cargo run --locked --release --example appearance_probe -- --hybrid-scaling
cargo run --locked --release --example appearance_probe -- --hybrid-transitions
cargo run --locked --release --example appearance_probe -- --confirm-hybrid-scaling
python3 scripts/check_hybrid_scaling.py
python3 scripts/report_hybrid_scaling.py
node scripts/verify_triangle_scaling.mjs artifacts/appearance-probe/hybrid-scaling
```

Resultados correntes em `artifacts/appearance-probe/hybrid-scaling/`: report.json guarda capturas/métricas; confirmation.json guarda20coortes com40slots duplicados,12rodadas e128frames por bloco; summary.json e regret.json derivam desses dados. O HTML permite comparar os cinco caminhos, duas resoluções, quatro densidades e câmera14→224→14; recorte512² ajusta CSS pelo DPR. Trata-se de reprodução de capturas, não render ao vivo no navegador.

As transições são procuradas por metadados da seleção em14…224, passo0,5, no relevo100×. Capturamos os dois lados de toda troca global/regional encontrada,20poses no total; não selecionamos transições por seu erro visual ou por seu tempo. Isso cobre trocas discretas desta trajetória, sem provar ausência de popping contínuo ou estabilidade em todos os ângulos.


## Resultados medidos

Apple M4 Pro/Metal,20coortes com quatro densidades residentes e alternadas,40slots duplicados,12rodadas,128frames por bloco. Compilações observadas e regressões desta tarefa terminaram antes da medição; outras atividades do desktop não são controladas. Câmera, seleção, codificação, submissão e trabalhoGPU entram na vazão; capturas/bake ficam fora. Estes são custos do experimento, não FPS de jogo ou confiança estatística.

Custo híbrido CPU+GPU sobrepostas, ms/frame, família com relevo:

| Resolução | Distância | 1× | 2× | 10× | 100× | 100/1 | Maior/menor | 100/10 | Economia vs full100× |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 3456×2234 | 14 | 0.2355 | 0.2670 | 0.4440 | 0.9003 | 3.824× | 3.824× | 2.028× | 56.6% |
| 3456×2234 | 28 | 0.1924 | 0.2137 | 0.3768 | 0.3764 | 1.956× | 1.958× | 0.999× | 73.9% |
| 3456×2234 | 56 | 0.1845 | 0.2011 | 0.2119 | 0.2592 | 1.405× | 1.405× | 1.223× | 75.0% |
| 3456×2234 | 112 | 0.1767 | 0.1808 | 0.1753 | 0.1946 | 1.102× | 1.110× | 1.110× | 76.9% |
| 3456×2234 | 224 | 0.1755 | 0.1778 | 0.1779 | 0.1745 | 0.995× | 1.019× | 0.981× | 77.8% |
| 1728×1117 | 14 | 0.0841 | 0.1077 | 0.2795 | 0.2729 | 3.246× | 3.325× | 0.976× | 80.6% |
| 1728×1117 | 28 | 0.0812 | 0.0994 | 0.1141 | 0.1595 | 1.964× | 1.964× | 1.398× | 83.0% |
| 1728×1117 | 56 | 0.0706 | 0.0739 | 0.0775 | 0.0964 | 1.365× | 1.365× | 1.243× | 88.1% |
| 1728×1117 | 112 | 0.0634 | 0.0693 | 0.0648 | 0.0663 | 1.046× | 1.092× | 1.023× | 90.6% |
| 1728×1117 | 224 | 0.0591 | 0.0509 | 0.0546 | 0.0561 | 0.950× | 1.161× | 1.027× | 91.7% |

No caso próximo3456×2234/distância14/fonte100×, full2,073ms, regional0,890ms e híbrido0,900ms: conserva a economia de56,6% sobre full, porém não melhora o regional nesta posição (+1,2%). A seleção é regional e mantém880.384triângulos. Na menor resolução/perto, full1,410ms→híbrido0,273ms (−80,6%),255.744triângulos. A curva completa1×→100× ainda cresce3,82×/3,25× nessas duas posições; não alcançamos independência de densidade perto.

O platô10×→100× permanece na distância28 da resolução maior (0,3768→0,3764ms), mas não é contínuo: distância56 ainda cresce22,3%. Na menor resolução/distância28,10×→100× cresce39,8%. A combinação melhora principalmente os extremos distantes: no relevo1728×1117/distância112, regional0,0758→híbrido0,0663ms (−12,4%); distância224,0,0693→0,0561ms (−19,0%). Em3456×2234/distância224, regional0,1817→0,1745ms (−3,9%), com curva híbrida1×→100× praticamente estável nesta posição.

O plano redundante utiliza global nas72poses e seu custo deixa de crescer significativamente com a fonte no intervalo medido. O melhor ganho relativo ao regional puro entre os80pontos cronometrados foi20,2% (plano100×,1728×1117,distância112:0,0670→0,0535ms). Isso é o melhor caso, não ganho geral. Mediana das razões híbrido/regional:0,9833 (−1,7%).

### Regressões que permanecem

A mediana do excesso de custo sobre o menor tempo entre os dois candidatos elegíveis foi+0,97%; o pior caso+11,39%. Nesse caso, plano1×/1728×1117/distância14, regional0,07203ms, global preservado0,08015ms e híbrido0,08024ms. O modelo escolhe global por menos triângulos e mesmo número de draws, mas ele custa mais na prática. Ambos os controles duplicados confirmam a mesma ordem nessa rodada: regional0,07374/0,07033ms e híbrido0,08093/0,07955ms. Não há correção demonstrada para esse erro de previsão, nem atribuição causal isolada a cache/rasterização.

A diferença para o melhor candidato também inclui custo da decisão e ruído: por exemplo, plano1×/1728×1117/distância112, global preservado0,05248ms e híbrido0,05799ms (+10,5%), apesar de escolherem os mesmos comandos de desenho. Não tratar todo excesso como erro de escolha, nem descartar todos os casos como ruído. O arquivo regret.json publica os80pontos, incluindo valores negativos e perdas. O global antigo permanece visível como controle: pode ser mais rápido, mas não entra no híbrido por sua falha de cobertura comprovada.

Os timestampsGPU isolados ficaram instáveis: entre todos os controles duplicados, razão0,220…7,294; a vazão em blocos variou0,791…1,196, mediana0,998. O pior excessoGPU bruto do híbrido sobre o menor candidato foi140%, em relevo1×/1728×1117/distância56, com pares do próprio híbrido0,0453/0,1783ms. Isso impede alegar um limite confiável de regressãoGPU ou inferir causas a partir desse valor. Os dados de GPU, serial e CPU continuam publicados; priorizamos a vazão e mostramos os controles, sem equipará-la a FPS ou comprovação estatística. Mediana de excessoCPU sobre o menor candidato+4,2%; pior+45,7%, também com dispersão. O seletor tem custo e não é um oráculo de tempo.

### Qualidade, memória e evidência

Híbrido164/164poses aprovadas:144da curva mais20nas trocas de estratégia. Zero cobertura divergente; máximos RMSEforeground0,00358144, p99RGB0,01664484 e maxRGB0,02676493. Gates permanecem0,005/0,02/0,0005. Global preservado passa144/144; regional anterior também. O global preciso anterior conserva sua falha em plano10×/1728×1117/distância224, um pixel de cobertura. Não alteramos o limiar para eliminar a falha.

Híbrido exatamente igual ao candidato escolhido nos buffersRGBA32F de todas as164poses. Não é exatamente igual à referência full: as métricas de erro não são zero. Ida/volta reproduz PNGs idênticos;432imagens dos três controles anteriores são byte a byte iguais a EAF-86. Poda versus exaustivo mantém decisão/draws em todas144poses;118escolhas globais e26regionais,79sem visitar regiões,52com64visitas,13com poda parcial. No plano, todas72escolhas globais encerram sem visita regional.

Inspeção de recortes512² de perto, dos dois lados da troca43→43,5 e do caso de contorno anteriormente falho não mostrou frestas/linhas novas; referência, híbrido e diferença8× estão em inspection.png. A referência preserva também seu aliasing de amostra única. Capturas estáticas e métricas não comprovam vídeo contínuo, referência integrada, materiais arbitrários, cena multiasse ou apresentação nativa/aceite humano.

Payload por fonte, incluindo todos os controles e normais:3,00/5,82/27,62/266,96MiB em1×/2×/10×/100×. A inclusão dos níveis globais preservados acrescenta50,58MiB ao100× comparado aos216,38MiB de EAF-86 (+23,4%), mesmo reutilizando vértices fonte. Isso não é memória mínima de um renderer de produção. Quatro fontes residentes por família somam303,40MiB; targets, readback, buffersCPU e driver são adicionais. Preparação100× mediu565ms/plano e604ms/relevo, com escrita em cache doSO, semfsync e sem comparação controlada de build-time. Fonte/fronteira/normal ainda crescem; nenhum streaming ou compressão novo.

Validação final:629testes Rust release/all-targets, Clippy com-Dwarnings e fmt; render/replay_inspection/selection headless aprovados.760PNGs com dimensões/CRC/descompressão/retorno,576comparações de qualidade da curva,20transições e432controles anteriores idênticos. JavaScript real do relatório executado comDOMsimulado:576combinações novas e432legadas, DPR/recorte/playback/carga obsoleta; resumos anteriores reproduzidos exatamente pelo código compartilhado. Sem browser nativo ou janela da demo manipulados. Mantido o build corrente no cache externo; não há novo contrato de runtime/MCP ou publicação.

Próximo limite: estimador precisa representar melhor o custo real, com validação independente das poses usadas para calibrar, e instrumentaçãoGPU em regime sustentado para reduzir a instabilidade dos passes isolados. Antes de prometer zero regressão, comparar ganhos previstos com controles repetidos e contabilizar o próprio seletor. A combinação está implementada, mas a meta de escolher sempre o menor tempo e a de custo constante para qualquer câmera continuam abertas.


Continuação: [previsão calibrada e validação em poses separadas](calibrated-cost-study.md). Mantém os resultados desta rodada como controle; não substitui estes números por medições de outro protocolo.
