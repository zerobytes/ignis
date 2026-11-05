# Regiões coordenadas e submissão agrupada — EAF-88

## Protocolo e checklist prévio

Hipótese: unir regiões adjacentes no build permite simplificar a fronteira interna sem criar frestas entre níveis independentes. O contorno externo da união fica fixo. Rejeitar uniões sem projeção local válida (orientação, ausência de sobreposição); não mover fronteiras entre grupos independentes sem coordenação. Isto é um recorte de coarsening, não uma hierarquia completa de transições.

Comparar submissão direta e multi-draw indireto da mesma seleção. A implementação Metal pode executar vários desenhos internamente; não chamar uma API de multi-draw de um único desenho físico sem evidência. Incluir montagem/upload de argumentos e seleção no custo total.

Repetir 1×/2×/10×/100× da mesma superfície Boulder01, com material/campo de normais interpolado sem adicionar detalhes. O compilador recebe somente a malha embaralhada. Remover redundância pelo compilador coplanar anterior antes de formar regiões coordenadas. Isso mede densidade redundante, não quatro complexidades visuais diferentes.

1. Build: agrupamento, adjacência, projeção, certificados e atlas offline.
2. Frame: seleção por erro projetado e proteção de silhueta; mesmos gates, sem vencedores por pose.
3. Dados: IGM1/IGC1/IGL1/INF1 binários existentes; argumentos indiretos cinco u32 por desenho, buffers limitados/residentes.
4. Reuso: compartilhar geometria e campo por variante; manter fonte para referência/fallback e contabilizá-la.
5. Escala: limites explícitos de união e comandos; medir triângulos/patches/draws versus entrada.
6. GPU: mesmos shaders/seleções na comparação direta/indireta; incluir uploads de comandos, não apenas timestamp interno.
7. Sincronização: GPU isolada da regressão MCP, lotes alternados/duplicados e CPU/GPU/parede separados.
8. Invalidação: codecs/checksums existentes; gerar do mesmo input sem usar genealogia no compilador.
9. Correção: testes de fronteiras mistas; capturas perto/longe/órbita/retorno nas duas resoluções; RMSE≤0,005, p99≤0,02, cobertura≤0,0005; não esconder candidato que falhe.
10. Custos mantidos: atlas, fonte residente e regiões conservadoras; sem alteração dos contratos funcionais e sem promessa de ganho.

## Implementação e domínio

O compilador regional tem uma opção de união offline: começa com folhas de até64 faces, tenta pares adjacentes em até oito rodadas, limita cada união a1.024 faces e exige projeção sem sobreposição e orientações compatíveis. Só depois simplifica a união. As antigas bordas internas ficam livres; a nova fronteira externa permanece fixa. O teste de malha32×32 mistura níveis em mais de uma região e verifica que nenhuma aresta interna fica aberta. Isto não implementa transições entre diferentes cortes de uma hierarquia: as uniões formam uma partição fixa.

O controle anterior usa folhas512. Como também mudou o tamanho inicial das folhas, há um terceiro controle offline de folhas64 sem união; ele separa a redução potencial atribuível às uniões. Seus mínimos ignoram os gates de câmera e não são economias de render aceitas. Ambos os candidatos têm campo de normais64 por região. A recuperação coplanar anterior roda primeiro, sem acesso à malha parental; seu erro global é somado aos limites dos níveis regionais e expande suas caixas. A elegibilidade da recuperação é verificada em todas as poses de captura. Não foi criada uma política de fallback para câmeras arbitrárias neste novo harness.

Corrigido um validador compartilhado: normais interpoladas podem ter comprimento menor que0,5 sem serem inválidas. O teste agora exige comprimento finito maior que1e-8 e preserva a magnitude original; normalizar cada vértice intermediário mudaria o campo interpolado. Zero continua rejeitado. O novo teste verifica aceitação e roundtrip de uma normal curta e rejeição de zero.

A fonte Boulder01 tem66.122 triângulos, com posições e normais importadas; não usa texturas/UVs ou materiais PBR originais. A subdivisão acrescenta pontos baricêntricos, não relevo intencional. As normais seguem interpolação afim antes da normalização no fragmento. Os multiplicadores1/2/10/100 são relativos à malha, **não triângulos visíveis por pixel**.

## Controle da própria subdivisão

Cada referência subdividida é comparada também com a entrada1×, nos mesmos pixels físicos. O primeiro ensaio abortou nesse controle; a execução diagnóstica completa conserva as reprovações no relatório, sem relaxar limiares nem aprovar essas coortes como superfície visualmente constante.

Quatro das144 poses reprovaram a densidade, todas em1728×1117, distância64: fator2/180°, fator10/45°, fator100/45° e90°. Na primeira, um único pixel do PNG muda mais que3/255 por canal, deRGB(58,52,44) para(128,117,102); o RMSE linear é0,005359, p99 0,000760 e cobertura idêntica, sobre708 pixels cobertos. Em100×/90° há também um pixel de cobertura divergente, razão0,001312. Isto é um controle falho, não uma melhora do candidato. Interpolação/rasterização de microtriângulos e as junções da subdivisão são hipóteses; a causa numérica completa não foi isolada.

Os caminhos recuperado, campo anterior e campo coordenado reprovam precisamente nessas mesmas quatro poses contra suas respectivas fontes densas. Nos outros140 casos passam. Todas as72 poses próximas passam, assim como todas as72 poses da resolução3456×2234 (conjuntos parcialmente sobrepostos). Direto e indireto têm RGBA32F byte-idêntico nas144 poses; os32 retornos de referência/candidato repetem seus próprios hashes. Estes testes não estabelecem identidade entre fonte e candidato nem ausência de cintilação em movimento contínuo.

## Protocolo de custo e dados

Cinco modos: fonte densa; recuperação coplanar anterior; regiões anteriores+campo; regiões coordenadas+campo diretas; exatamente as mesmas regiões via indireto. Duas resoluções, duas distâncias, quatro densidades;32 poses de câmera/luz por lote de128 frames. Oito rodadas em ordem alternada/palindrômica, duas duplicatas por modo/rodada, com aquecimento antes de cada amostra. Capturas são separadas dos timings. CPU inclui seleção/codificação; parede inclui upload/submissão/espera/readback do lote; timestamp GPU cobre a janela de desenhos, não o upload que a precede. Nenhum valor é FPS de gameplay.

Os dois atlas permanecem residentes e somente o bind group troca; não reconstruir pipelines/texturas dentro do benchmark. Todos os buffers comparadores ficam residentes simultaneamente. O ring de argumentos indiretos reserva128×4.096×20bytes =10MiB no harness; cada comando ativo são cinco u32. O frame monta e envia apenas os argumentos ativos, mas a reserva total existe. A API agrupa a codificação; o backend Metal de wgpu-hal27.0.4 (`src/metal/command.rs`, `draw_indexed_indirect`) itera sobre `draw_count` e chama `draw_indexed_primitives_indirect` para cada comando. Logo não é um único desenho físico.

Os pacotesIGC1/INF1 são binários compilados offline. JSON é exclusivamente relatório, não entrada do render. Os tamanhos de geometria/UV/atlas abaixo não incluem fontes separadas de comparação/fallback, cópiasCPU, metadados, framebufferRGBA32F/depth, staging/readback ou driver. RSS não foi medido. Os pacotes por densidade/variante são controles simultaneamente necessários a este experimento, não cópias históricas de builds.

## Reprodução

Na worktree gráfica, usar a entrada importada conforme `irregular-lod-study.md`:

```sh
cargo build --locked --release --example coordinated_probe
for factor in 1 2 10 100; do
  target/release/examples/coordinated_probe --build artifacts/appearance-probe/irregular-lod/boulder.igm artifacts/appearance-probe/coordinated-regions "$factor"
done
target/release/examples/coordinated_probe --run artifacts/appearance-probe/coordinated-regions
python3 scripts/report_coordinated_regions.py
node scripts/check_coordinated_regions_ui.mjs
```

`--smoke` usa apenas1× em864×558. `COORDINATED_QUALITY_ONLY=1` com `--run` executa as144 capturas sem benchmark e grava `quality-only.json`. Falhas de fidelidade são resultados explícitos no relatório; não são sucesso de qualidade só porque o processo terminou. Falhas de layout/capacidade, elegibilidade da recuperação, igualdade direto/indireto ou retorno interrompem a execução. O relatório completo requer todas as144 capturas e1.280 amostras de timing; coortes com controle de densidade falho ficam marcadas como inválidas para a alegação de conteúdo visual constante.

O painel HTML compara capturas; não é renderer no navegador. Visão geral reduz imagens para caber; desmarcar usa escala física nominal porDPR, com rolagem. Os resultados são de render offscreen em1728×1117 e3456×2234; `system_profiler` confirmou tela física3456×2234 neste Mac M4 Pro. Isso não demonstra apresentação do painel no monitor nem aceite humano.

## Geometria e memória medidas

| Entrada | Fonte T | Recuperada T | Mínimo offline sem união64 | Mínimo offline coordenado | Geometria+UV+atlas antigo / coordenado |
|---|---:|---:|---:|---:|---:|
| 1× | 66,122 | 66,122 | 51,908 | 49,732 | 10.866 / 21.447 MiB |
| 2× | 132,244 | 85,705 | 84,537 | 84,351 | 7.610 / 10.494 MiB |
| 10× | 661,220 | 66,151 | 51,821 | 49,767 | 11.232 / 21.450 MiB |
| 100× | 6,612,200 | 265,105 | 150,297 | 137,677 | 32.285 / 52.688 MiB |

Os mínimos offline não são a seleção elegível. Na fonte1×, a seleção coordenada chega a63.052T (−4,64%), contra mínimo65.092T do controle anterior desta mesma rodada. Perto, mantém66.114–66.122T em1728 e66.116–66.122T em3456. O campo e as uniões não autorizam apagar detalhes ainda visíveis. Até348 ranges em1× e959 em100×: baixar triângulos pode aumentar trabalho de seleção/submissão.

A recuperação em100× ainda deixa265.105T, aproximadamente4,01× o conteúdo1×. A seleção coordenada chega a222.311T longe, mas permanece226.653–236.853T perto em1728. Portanto a curva geométrica ainda não é plana. Não atribuir este excesso a detalhe novo nem presumir que tolerâncias mais frouxas seriam seguras. É necessário auditar reconstrução de regiões, junções e estabilidade numérica antes de ampliar o erro aceito.

O aumento de memória torna esta variante pouco atraente: para1×, o pacote coordenado exige21,447MiB de geometria/UV/atlas; só a fonte original ocupa2,176MiB. O100× exige52,688MiB no coordenado, além de163,484MiB da fonte densa quando mantida. Build/regeneração/recuperação/escrita da fonte100×:6,623s; build do candidato coordenado+campo+roundtrip+escrita:13,401s. Tempos de preparo observados, não parte do frame.

## Performance medida e decisão

Apple M4 Pro/Metal, resolução3456×2234. Tempos totais em ms/frame de lote; os cinco caminhos são pareados nesta execução.

| Entrada / câmera | Fonte densa | Recuperação anterior | Regiões anteriores+campo | Coordenadas diretas | Coordenadas indiretas |
|---|---:|---:|---:|---:|---:|
| 1× / perto | 0.27304 | 0.27700 | 0.25828 | 0.26101 | 0.26880 |
| 1× / longe | 0.19346 | 0.19996 | 0.20221 | 0.22224 | 0.22501 |
| 2× / perto | 0.36649 | 0.30123 | 0.26934 | 0.25950 | 0.27701 |
| 2× / longe | 0.19943 | 0.19453 | 0.20309 | 0.20277 | 0.21850 |
| 10× / perto | 0.99755 | 0.26575 | 0.25809 | 0.26366 | 0.26412 |
| 10× / longe | 0.29279 | 0.18939 | 0.21096 | 0.21749 | 0.23568 |
| 100× / perto | 6.86343 | 0.32944 | 0.34605 | 0.37518 | 0.38711 |
| 100× / longe | 2.58719 | 0.19639 | 0.25780 | 0.30044 | 0.31607 |

**A nova representação não demonstrou ganho líquido no caso pesado.** Em100× perto, recuperação anterior→coordenado direto:0,32944→0,37518ms (+13,9%); indireto0,38711ms (+17,5%). No mesmo par, GPU0,28438→0,23004ms melhora19,1%, mas CPU0,02782→0,13389ms cresce4,81×. As parcelas são intervalos parcialmente sobrepostos; não somá-las para reconstruir parede. O conjunto de medições indica que seleção/submissão anulou a economiaGPU.

Contra o próprio coordenado direto, indireto apresenta medianas de parede0,18–18,50% maiores nas16 coortes. A diferença pequena não é estatisticamente conclusiva: discrepâncias medianas das duplicatas coordenadas chegam a16,3%, e cada coorte conserva sua dispersão no resumo. CPU indireta cresce em todas as coortes; não houve benefício total demonstrado pela API agrupada neste backend.

A recuperação anterior já reduz o custo da fonte100× próxima de6,86343 para0,32944ms (20,83× nesta comparação). Isso é ganho da técnica anterior, não da nova união ou submissão. Entre1× e100×, o anterior próximo cresce0,27700→0,32944ms (+18,9%); o coordenado cresce0,26101→0,37518ms (+43,7%). Perto em1728 também não há platô perfeito. Não atribuir toda diferença aos triângulos: ordem de vértices/índices e regiões muda entre os buffers, e perto na fonte1× quase nenhum triângulo é removido.

Decisão: manter ambos os novos caminhos experimentais, sem ativação automática nem mudança do runtime. A próxima investigação útil é a estabilidade do compilador que elimina redundância e do próprio controle de subdivisão: por que a fonte100× mantém265mil triângulos recuperados e por que quatro vistas distantes mudam? Depois disso, reduzir o número de decisões/desenhos por frame com hierarquia e grupos maiores selecionáveis sob limites válidos. Acrescentar mais atlas ou apenas trocar a API de submissão não resolveu o gargalo observado.

Qualidade não é igualdade: mesmo em1×, aprovado em36/36 poses, o máximo erro de canal coordenado chega a0,26305; RMSE máximo0,004335 e p99 máximo0,006688. Os gates toleram erros locais raros. Sem certificado fotométrico, vídeo contínuo ou aceite visual humano, não declarar preservação universal de detalhes.

## Checklist posterior e validação

1. Frequência: uniões, simplificação, certificados e atlas somente no build; frame visita metadados regionais, sem percorrer fonte densa.
2. Alocações: BTree/Hash/vetores offline; seletor e argumentos indiretos ainda criam vetores por frame, medidos no custoCPU. Não declarar zero alocações.
3. Layout: formatosIGC1/INF1 existentes, f32/u32, atlasRGBA8Snorm; argumentos20B/u32. Sem JSON no render nem compactação ótima prometida.
4. Reuso: buffers e dois atlas residentes durante cada coorte; bind groups reutilizados. Fonte/copias dos comparadores explicitamente adicionais.
5. Limites: oito rodadas de união,1.024 faces/união,4.096 comandos/frame,128 frames/ring. Modelo ainda escala com regiões, não somente pixels visíveis.
6. GPU/I/O: fonte/candidato compartilham shader de iluminação; indireto usa exatamente índices/ranges do direto, com upload adicional. Sem streaming ou seleçãoGPU novos.
7. Sincronização: coleta final sem regressões concorrentes; testesMCP depois. CPU e GPU não somados; dispersão e controles negativos preservados.
8. Invalidação: codecs/versionamento/checksum existentes, roundtrip no build e hashes dos12 pacotes registrados. Erro de recuperação composto com erro regional; sem certificado formal do atlas.
9. Correção:683 testes Rust release/all-targets, Clippy-Dwarnings, fmt/diff e13 gruposMCP headless aprovados. Testes novos cobrem normais curtas e fronteiras coordenadas com níveis mistos.144 poses,1.280 amostras temporais,409 PNGs íntegros,80 paresPNG de retorno idênticos,1.024 estadosDOM/DPR simulados; recorte próximo inspecionado. Quatro falhas visuais continuam explícitas; passar testes de código não as elimina.
10. Custos mantidos: fontes, atlas, ring10MiB e temporários do harness, seleçãoCPU e múltiplos desenhos Metal. Nenhuma mudança funcional/NPC/Scene/Command/RenderFrame/MCP, nenhum navegador/demo controlado. Sem aceite humano nem ganho universal/FPS alegado.

Artefatos correntes: `artifacts/appearance-probe/coordinated-regions/`, com `results.json`, `summary.json`, `verification.json`, logs e pacotes. Removidos21 outputs supersedidos de smoke após checar ausência de uso com lsof. Build corrente `target/release`, sem versões históricas novas; relatórios anteriores mantidos como referências de pesquisa em uso.
