# Hierarquia de certificados locais — EAF-88

## Protocolo antes de implementar

Usar os mesmos pacotes `external-recovery/stable-{1,2,10,100}.igc`: fonte, proxy, materiais e limites inalterados. Comparar certificado global anterior contra árvore de certificados locais, fonte subdividida e malha importada1×. Duas resoluções, perto/longe, órbita/luz, duplicatas e custo completo de seleção. A árvore não conhece poses nem resultados de captura.

Alternativas: varrer todas as regiões tem custo linear por frame; novos draws regionais exigiriam outro formato de geometria. Este incremento primeiro agrupa certificados numa árvore binária espacial offline. Aprovar um nó aprova seus descendentes; nó reprovado exige filhos; folha reprovada ou orçamento esgotado mantém a fonte inteira. Não altera shader ou malha. O custo é metadado adicional e consulta CPU limitada; não promete resolver memória da fonte residente.

Checklist prévio: (1) construir/ordenar offline, validar no load; (2) percurso sem pilha/alocação por frame, no máximo128 nós; (3) bloco ICH1 versionado little-endian f32/u32, JSON só relatório; (4) geometria e ranges compartilhados, root fast path; (5) folhas até32 regiões, limite de tamanho/profundidade no codec; (6) nenhum upload/readback adicional de seleção; (7) coletaGPU isolada das regressões; (8) hash vincula pacote completo, validação de árvore/permutação/bounds antes do uso; (9) testar corrupção, fallback, conservadorismo e órbita/retorno; (10) manter certificados originais e fonte fallback, medir bytes e custo em vez de alegar ausência de trabalho. Texturas/PBR e novo relevo continuam fora deste teste.

## Formato e seleção

ICH1 é um sidecar experimental; IGC1 e contratos funcionais não mudam. Cabeçalho32B: magic4, versãou32, hashFNV64 do pacoteIGC inteiro, hashFNV64 do payload, quantidade de nósu32 e regiõesu32. Payload: nós40B (`low.xyz`, erro espacial, `high.xyz`, erro normal em8f32LE, escapeu32, contagem de folha u32), seguidos da permutação de IDs de região u32. Esses hashes detectam mistura/corrupção; não são assinatura criptográfica.

Construção por mediana espacial da maior extensão, desempate por ID, folhas até32 regiões. Árvore preorder com índice de escape: quando um bound passa, salta diretamente ao primeiro nó fora da subárvore. Se falha, desce; falha de folha ou limite128 mantém fonte. Nenhuma decisão usa fator de subdivisão, pose pré-gravada ou métricas de imagem. Projeção reutiliza a fórmula existente, com profundidade/near-plane conservadores. Normais mantêm limite0,000101.

Load verifica versão/tamanho/checksum/vínculo, permutação completa sem repetição, escape/profundidade/estrutura binária, cobertura e igualdade dos bounds com agregações dos certificados originais. Só então descarta IDs e mantém nós. Validação éO(regiões+nós), não recompila/ordena a hierarquia. Runtime não aloca, não descomprime geometria e não lê JSON. Coordenadas/erros permanecemf32 como IGC; sem quantização nova que reduza qualidade. Os nós ficam naCPU; GPU recebe os mesmos buffers/draws.

## Reprodução

```sh
cargo build --locked --release --example recovery_probe
target/release/examples/recovery_probe --certificates artifacts/appearance-probe/external-recovery artifacts/appearance-probe/certificate-hierarchy
RECOVERY_CERTIFICATES=1 RECOVERY_BORDERS=1 RECOVERY_FALLBACK=1 target/release/examples/recovery_probe --run artifacts/appearance-probe/external-recovery artifacts/appearance-probe/certificate-hierarchy
python3 scripts/report_certificate_hierarchy.py
node scripts/check_recovery_stability_ui.mjs --certificates
```

A referência anterior usa `external-recovery/stable-*`, já incluindo todas as correções anteriores. A mesma geometria é lida para os dois seletores; não há cópia dos pacotes32MiB no novo diretório. Fonte subdividida e malha importada1× são controles distintos no painel. Fontes não são adicionadas à árvore nem removidas daGPU. Tempos medidos incluem seleçãoCPU, codificação, submissão e espera do lote128frames; não são latência individual nemFPS de um jogo.

A auditoria offline percorre4.096 configurações (quatro fatores, oito distâncias, duas alturas,64ângulos), checando cada decisão favorável contra todas as regiões individuais. Inclui câmera dentro/próxima do objeto e casos em que o fallback deve permanecer. É uma verificação de conservadorismo do agrupamento sob os certificados existentes, não uma prova nova de erro do compilador original.

## Resultados medidos

Medianas totais emms/frame de lote, seleção incluída. Cada célula usa16amostras, oito rodadas alternadas, duplicatas,32poses e128frames/lote.1.024amostras no total. GPUApple/Metal do laboratório, sem workload de jogo completo.

| Fator | Resolução | Câmera | Fonte subdividida | Global anterior | Hierarquia | Importada1× | Proxy global/local de128 |
|---|---|---|---:|---:|---:|---:|---:|
| 1× | 1728 | perto | 0.078685 | 0.080673 | 0.080747 | 0.077031 | 128/128 |
| 1× | 1728 | longe | 0.082104 | 0.086088 | 0.081680 | 0.084609 | 128/128 |
| 1× | 3456 | perto | 0.201972 | 0.215460 | 0.206616 | 0.212818 | 128/128 |
| 1× | 3456 | longe | 0.186885 | 0.190558 | 0.193720 | 0.192497 | 128/128 |
| 2× | 1728 | perto | 0.112775 | 0.105396 | 0.106923 | 0.105011 | 128/128 |
| 2× | 1728 | longe | 0.086727 | 0.081116 | 0.085194 | 0.083400 | 128/128 |
| 2× | 3456 | perto | 0.228348 | 0.222749 | 0.225867 | 0.207933 | 128/128 |
| 2× | 3456 | longe | 0.197366 | 0.195318 | 0.194108 | 0.184057 | 128/128 |
| 10× | 1728 | perto | 0.155007 | 0.077354 | 0.078377 | 0.077361 | 128/128 |
| 10× | 1728 | longe | 0.086719 | 0.078350 | 0.079013 | 0.082749 | 128/128 |
| 10× | 3456 | perto | 0.282356 | 0.228901 | 0.215619 | 0.212636 | 128/128 |
| 10× | 3456 | longe | 0.199917 | 0.192391 | 0.189316 | 0.185318 | 128/128 |
| 100× | 1728 | perto | 1.228761 | 0.081376 | 0.083690 | 0.081100 | 128/128 |
| 100× | 1728 | longe | 0.431394 | 0.077775 | 0.081083 | 0.077319 | 128/128 |
| 100× | 3456 | perto | 1.649225 | 1.166495 | 0.204714 | 0.215863 | 48/128 |
| 100× | 3456 | longe | 0.427428 | 0.181637 | 0.181356 | 0.176558 | 128/128 |

O caso crítico100×/3456×2234/perto passa de1,166495ms para0,204714ms:−82,45% no total,5,70× mais rápido que o seletor global anterior. Fonte subdividida1,649225ms:−87,59% versus desenho denso,8,06×. GPU1,120337→0,167893ms (−85,01%). CPU de codificação/submissão, incluindo decisão,0,027881→0,027954ms; não é medição isolada do seletor. Importada1×0,215863ms: resultado próximo; não atribuir superioridade de5% à hierarquia frente a uma geometria que ela não simplifica além da base. GPU/CPU se sobrepõem; não somá-los para obter total.

O ganho crítico aparece nas8/8rodadas; variabilidade mediana das duplicatas3,76% no candidato,6,08% no anterior. Proxy passa de48/128para128/128frames. No mesmo enquadramento,1/2/10/100× com hierarquia custa0,206616/0,225867/0,215619/0,204714ms. O crescimento100× de entrada não aumenta o custo nesse teste de subdivisão redundante; os intervalos entre coortes não devem ser lidos como curva temporal exata.

Não houve melhoria universal nos casos leves: várias medianas pioram0,8–5,0%, incluindo2×/1728/longe0,081116→0,085194ms (+5,03%,3/8rodadas favoráveis, duplicatas2,82%). Outros casos melhoram com geometria/decisão iguais, sinal de variação de medição. Não declarar regressões leves eliminadas nem ganho confiável em fatores cujo root já passava. A melhoria robusta deste incremento é o caso100×perto nativo; nela se concentra a decisão de continuar.

## Fidelidade e limites

144/144poses passam contra a superfície importada1×, cobertura idêntica e maiorRMSE1,00894e-6.142/144contra fonte subdividida: mesmas duas falhas100×distantes,1728/frame5 e3456/frame7, também presentes em fonte versus importada. Nenhuma falha nova e nenhum limite relaxado. Quatro poses próximas de captura antes emfallback passam a usar o proxy; as métricas agora removem a diferença introduzida pela própria fonte densa nessas poses. Isso não equivale a todos os floats idênticos, nem a solucionar aliasing da referência densa.

Em64ângulos de auditoria3456/perto100×, global aprova24; hierarquia64, com1–5consultas. O limite máximo dos grupos aprovados é0,149283px, abaixo de0,15; não usar o máximo global rejeitado como erro do caminho escolhido. Para distâncias2,0e1,5, a auditoria aprova64/64 e2/64 respectivamente; são resultados de certificadosCPU, não novas medições visuais/GPU. Na distância1,5, regiões individuais permitiriam35/64: folhas agrupadas ainda são conservadoras. Dentro/próximo do objeto a fonte continua necessária. Nenhuma consulta das4.096auditadas excedeu43nós; orçamento permanece128.

1.023nós,40.920BresidentesCPU (39,96KiB) em todos os fatores. ArquivoICH85.424–85.436B (≈83,43KiB), incluindo permutação usada no load e então descartada.100×:build+autovalidação0,710ms,load+validação0,119ms, medições únicas indicativas, não benchmark de compilação repetido. Bytes deGPU permanecem os anteriores:proxy270.672B e fonte26.750.352B em100×. IGC32.058.360B continua; somarICH, não substituirIGC. Nenhum platô de armazenamento/memória total foi demonstrado.

A técnica ainda recupera subdivisão afim da mesma superfície. Não demonstra custo constante para100×novas saliências, contornos, normais ou materiais independentes. O próximo teste científico deve acrescentar relevo real em frequências/alturas controladas, preservá-lo no close e descartá-lo somente quando o certificado projetado permitir. Cenas com muitos assets, materiais/PBR e outros hardwares permanecem pendentes.

## Checklist posterior e validação

1. Frequência: particionamento, ordenação e serialização apenas no build; load faz validação linear. Frame consulta árvore pronta, com fast path de um nó.
2. Alocações/cópias: `Hierarchy::decide` não aloca nem cria pilha/Vec; nó40B e decisão na stack. EncoderGPU e uniformes mantêm custos anteriores. A permutação é descartada depois do load.
3. Tipos/layout: f32/u32LE, bounds sem nova quantização, escape em vez de ponteiros; ICH1 separado e vinculado a IGC1. JSON apenas diagnóstico/painel, sem tráfego no renderer medido.
4. Reuso: os dois caminhos usam exatamente os mesmos pacotes `stable-*`; geometria/shaders/materiais inalterados. Sem descobrir conectividade ou reconstruir resumos no frame.
5. Escala: no máximo128consultas, fallback ao exceder;contagem efetiva1.023nós, folhas até32regiões, limite2milhões de regiões no formato experimental. Pior percurso observado43nós no stressCPU, cinco no enquadramento crítico. Fonte permanece residente.
6. GPU/I/O: nenhuma nova textura/bufferGPU ou readback de seleção. Timing genérico reutiliza draw por material; readbacks ePNG somente capturas fora dos tempos.
7. Sincronização: sem novas threads/locks. Aquecimento/alternância/duplicatas; coletaGPU isolada de testes e regressões. Tempos de lote incluem decisão/submissão/espera; não sãoFPS.
8. Invalidação: checksum, hash do pacote completo, versão, limites e validação de topologia/permutação/bounds. Pacote de outra geometria e bounds adulterados são rejeitados, mesmo com checksum recalculado.
9. Correção:696testesRust release/all-targets, Clippy-Dwarnings, fmt e13gruposMCPheadless (`render`, `replay_inspection`, `selection`) passaram. Quatro testes novos cobrem conservadorismo, orçamento esgotado no meio do percurso, normais inseguras/ausência de redução e codec corrompido.4.096casos do asset com auditoria exaustiva.144poses,1.024amostras,576PNGs íntegros,64pares de retorno idênticos,140paresPNG global/local idênticos quando a decisão coincide, quatro capturas alteradas quando a decisão muda. Todas as capturas repetidas ao ampliar painel para oito ângulos reproduziram exatamente as métricasfloat; tempos originais preservados.2.048estadosDOM/DPRsimulados passaram. Capturas próximas2× e100× inspecionadas; preview da ferramenta redimensionado não é inspeção humana em resolução nativa. Sem aceite humano/browser nativo alegado.
10. Custos mantidos: memóriaCPU dos nós, validação no load, cálculos de projeção por nó, fonte/proxy/certificadosoriginais e buffers de referência do laboratório. Pequenas regressões leves, fonte1sppdivergente, novo microdetalhe, PBR, cenas complexas eGPUdiferentes permanecem questões explícitas.

Artefato corrente:`artifacts/appearance-probe/certificate-hierarchy/`, com quatroICH, relatório/auditoria/resultados, capturas e logs de validação. ReutilizaIGCs de `external-recovery` como controles necessários, sem duplicar malhas grandes. `target/release` corrente reutilizado, sem versões históricas novas. Sem mudanças emNPC/Scene/Command/RenderFrame/MCP ou demo; sempush/publicação. EAF-88 segue In Progress.
