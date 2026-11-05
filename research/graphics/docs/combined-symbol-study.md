# Comparação incremental de símbolos e seleção anterior

## Protocolo antes da implementação — 27/10/2026

Base B congelada: variante135 do estudo near-plateau, que combina seleção global/regional, modelo automático forward/depth e posições compactadas em ordem espacial. Não chamar geometria completa de baseline otimizada. A é full0, D é B com resposta de material compilada forçada, C é B com essa resposta opcional. Mesmo asset, normal, cobertura, resolução, câmera e luz. Não somar percentuais de experimentos diferentes.

O atlas geométrico IRK1 anterior não passa os gates e não tem profundidade de composição. Ele não será promovido. O candidato incremental conserva toda a forma selecionada por B e compila somente a função de resposta do material existente, em coordenadas dot(N,L) e dot(N,H). É um símbolo de material, não o atlas de geometria da pedra e não um novo teste de triângulos por pixel. O material específico é o Blinn-Phong do heightfield anterior, não GGX arbitrário. Uma tabela RGB10A2 filtrável128² guarda RGB e cobertura opaca, com64KiB de payload compartilhado. Normal, direções e coordenadas continuam sendo calculadas; não alegar ausência de cálculo.

Calibração: quatro densidades1×/2×/10×/100×, duas resoluções físicas3456×2234 e1728×1117, distâncias16/48/192 e deslocamentos angulares independentes. Congelar política antes de medir teste14/28/224, incluindo câmera, luz e ambos variando; percurso de aproximação/afastamento. Esses fatores são os assets anteriores, que mudam frequência do relevo; não são o experimento de subdivisão com superfície fixa. Não extrapolar platô.

Política barata em oito classes pré-definidas (forward/depth e área projetada<0,01/<0,1/<0,5/restante da tela). Ativar somente classes com pelo menos três coortes e benefício mínimo5% em todas as coortes, maior que dispersão de controles. Sem tabela de vencedores por pose/asset, sem leitura de pixels no frame. Domínio/política inválidos rejeitam símbolo. Registro de escolha, ambos os passes e seleção dentro do tempo total. Tempos GPU separados do throughput. Ordens alternadas, duplicatas, rodada de repetição e câmera/luz realmente atualizadas dentro dos blocos.

Gates inalterados: RMSE foreground≤0,005, p99RGB≤0,02, divergência de cobertura≤0,0005 contra A; também medir diferença C/D contra B e resíduo temporal. Medir todos os caminhos, inclusive quando fallback100%. Não mudar critérios após resultados. O limite analítico da tabela é quantização0,5/1023 + interpolação0,08×24×23/(8×127²), aproximadamente0,000831 por canal, além do arredondamento do filtroGPU. Erro geométrico existente em B continua sendo validado contra A.

## Revisão estática prévia

1. Frequência: compilaçãoCPU e upload uma vez; seleção por desenho, lookup no fragmento. Sem reconstrução de mesh.
2. Alocações: tabela limitada128²u32; frame usa arrays/recursos persistentes. Capturas e relatórios fora da medição.
3. Layout: binárioLE versionado, checksum e identidade do shader; texelRGB10A2 consumido diretamente pelaGPU, cobertura lida. JSON apenas instrumentação, não payload/política no frame.
4. Repetição: resposta compartilhada por todo o material, uma amostra filtrada por fragmento; não repetir tabelas por triângulo/objeto.
5. Escala: oito classes fixas, sem histórico crescente; payload64KiB, sem alvos adicionais.
6. GPU/I/O: mesmos draws/depth/targets; pipelines opt-in mantêm baseline original; leitura de arquivo só no preparo. Bind extra e lookup incluídos nos tempos.
7. Sincronização: nenhuma thread/lock/dependência nova; calibração, teste e regressãoGPU sequenciais.
8. Invalidação: validar versão/checksum/assinatura do material, adaptador e política; mudanças no shader invalidam dados.
9. Correção: gates A/B/C/D, identidade no fallback, corrupção de binário, limites da tabela, movimento/retorno; testes/clippy/fmt e regressões headless.
10. Custos mantidos: normal/direções, textura/material específicos, memória dos comparadores e readbacks científicos. Resultado pode ser zero ganho; não integrar runtime/Scene/Command/MCP nem alegar aceite humano.

## Resultado

**Não houve ganho incremental robusto.** A tabela de material preservou a qualidade neste domínio, mas a calibração rejeitou todas as classes. C executou B em252/252poses e produziu pixelsRGBA32F exatamente iguais. Isso valida o fallback, não demonstra uma cena com uso misto útil de símbolos. Não promover o candidato nem interpretar flutuações de C contra B como economia.

Apple M4 Pro/Metal;384 medições pareadas de calibração em24coortes,24coortes de validação e16de repetição. Oito rodadas, dois controles por caminho, throughput de64frames eGPU32frames, câmera/luz recalculadas e submetidas a cada frame. Sem readback de decisão, criação de recursos ou compilação no timing. Comparação de médias das duas medianas de controles por coorte; percentuais agregados são medianas de razões por coorte. Não são FPS de jogo.

| Comparação | Validação | Repetição perto/longe |
|---|---:|---:|
| D forçado/B, razão mediana de tempo total | 1,0004 (+0,04%) | 1,0151 (+1,51%) |
| C opcional/B, razão mediana de tempo total | 0,9897 | 0,9951 |
| Classes habilitadas | 0/8 | política congelada |

C e B executam o mesmo trabalhoGPU: diferenças de tempo são controles da dispersão e do pequeno desvioCPU, não ganho dos símbolos. Razão entre duplicatas totais no conjunto0,8248–1,7591; isso impede promessas de regressão zero ou significância de pequenas diferenças. O critério conservador de5% + dispersão + mínimo3coortes por classe foi mantido. Classes atravessadas por um mesmo percurso não servem para habilitar uma classe isolada na calibração.

Exemplo100×,3456×2234, perto14, material/luz/câmera em movimento:

| Rodada | B anterior | D forçado | C opcional |
|---|---:|---:|---:|
| Validação | 0,68329ms | 0,69576ms | 0,68027ms |
| Repetição | 0,70176ms | 0,70583ms | 0,69525ms |

GPU do mesmo exemplo: B0,67948/D0,67876/C0,69454ms na validação e B0,66050/D0,67958/C0,67207ms na repetição. GPU e throughput são intervalos distintos, não parcelas a somar.

Memória incremental medida por layout: arquivo65.568bytes (header32 + payload65.536), texturaGPU64KiB e política40bytes em disco, máscarau8 na execução. Pipeline/bind/sampler são objetos de driver sem tamanho físico deVRAM medido. Nenhum alvo adicional e nenhuma economia geométrica: triângulos, normais, buffers e visibilidade são os deB. O comparador mantém a tabela para D mesmo com C desabilitado; não alegar redução deRSS. RGB10A2 fornece filtragem hardware e lêRGB+coverage; não é prova de formato ótimo. A função tem estrutura separável: esse experimento testa uma consulta completa, não esgota representações menores nem materiais complexos.

## Qualidade e movimento

252poses físicas em duas resoluções, quatro assets de densidade, três distâncias, nove amostras por percurso; no100× próximo há controles separados de câmera e luz. Arco da câmera aproximadamente±18,3°, variação de distância±12%, elevação oscilante; luz percorre360°. Não é órbita completa da câmera, deslocamento contínuo de14até224, nem teste de animação, obstáculos ou múltiplos objetos. Benchmarks usam64poses de movimento por bloco, distintos das nove capturas. Não alegar ausência de popping entre todas as amostras.

- B252/252: RMSEmax0,00356056, p99max0,01676044, cobertura divergente0.
- D252/252: RMSEmax0,00359341, p99max0,01678604, cobertura divergente0.
- C252/252: pixels exatamente B; gates anteriores sem relaxamento.
- D contra B: RMSEmax0,00025204, maior diferença individualRGB0,00085312. A aproximação analítica da tabela é0,00083081 antes da precisão do filtroGPU; não confundir esse limite ideal com bound completo de raster/material.
- Resíduo temporalRMSEmax B/C0,00476501; D0,00478550. Sem gate temporal novo ou certificado contínuo.
- 84pares de fechamentoPNG idênticos e8capturas do baseline anterior byte-idênticas a near-plateau. Caminho opcional com máscara255 foi exercitado e produz D exato antes de restaurar a política congelada; não foi usado nos timings nem nas escolhas finais.

Inspeção de imagens confirma o mesmo relevo aparente nos exemplos vistos; padrões direcionais existentes na referência não são removidos. Métricas usam pixelsfloat na resolução declarada, e não inspeção visual reduzida como critério. Aceite visual humano/navegador nativo permanece separado.

## Interpretação e continuidade

A hipótese restrita “substituir esta resposta barata de material por uma tabela melhora o renderer anterior” não foi sustentada. A combinação segura funciona, mas não economiza trabalho suficiente. Não extrapolar esse resultado para o atlas geométricoIRK1: esse não foi integrado e continua inelegível pela falha de forma/visibilidade. O experimento não responde ainda se uma representação geométrica mais fiel reduziria trabalho residual deB. Esse passo requer resolver profundidade, desoclusão e detalhe, mantendo comparação contraB; não otimizar novamente apenas contrafull.

Os assets1×/2×/10×/100× aqui têm frequências de relevo diferentes, como near-plateau. Não confundir com o estudo pixel-density de226milhões de triângulos e superfície constante. A rodada comprova a metodologia incremental e um resultado negativo específico; não um novo platô próximo, rendererAAA ou integração funcional.

## Reprodução e validação

`python3 scripts/run_combined_symbol.py` compila o binário e a tabela, calibra somente em16/48/192, congela política/assinaturas, captura e mede14/28/224 e repete perto/longe. Saídas correntes em `artifacts/appearance-probe/combined-symbol`; relatórioHTML com referência/B/CouD, controles de densidade/resolução/distância/movimento, playback de capturas e recortes1:1 físicos porDPR. Fonte/modelo/payload registrados porhash; estatísticas brutas e perdas publicadas.

645testesRust release/all-targets, Clippy-Dwarnings e fmt aprovados;13gruposMCP headless render/replay_inspection/selection. Verificador científico:756comparações,764PNGsCRC/dimensões/decompressão,84retornos e8baselines legados idênticos,24+24+16coortes completas e política congelada. UI504estados eDPR1/1,5/2/3 emDOMsimulado; não comprova navegador nativo. Nenhuma janela da demo foi tocada. Build externo corrente reutilizado, runtime/NPC/Scene/Command/RenderFrame/MCP preservados.

## Revisão estática após implementação

1. Frequência adequada: `material_symbol::compile` só emCLIbuild; `Pipelines::allows` apenas por desenho, sem scan de triângulos.
2. Alocações adequadas:64KiB+header no preparo; frame sem nova coleção de seleção. Capturas alocam imagens/estatísticas fora do tempo. Modelo existente e encoder mantidos.
3. Layout adequado ao teste: RGB10A2 filtrável direto, header/checksum/material validados. Não há JSON no decoder da tabela/política; manifests de geometria e relatório continuam instrumentos offline anteriores.
4. Repetição mantida com motivo: normal/direções calculadas por fragmento para preservar detalhes; uma consulta substitui somente a resposta do material. Sem ganho demonstrado.
5. Escala limitada: oito classes, payload compartilhado, nenhuma representação nova por objeto. Não afirmar aplicabilidade geral da função simples.
6. GPU/I/O adequado: usa osmesmos passes/draws/targets; umbind e uma consulta quando forçado. Máscara0 evita projeção extra e qualquer bind/lookup de símbolo emC.
7. Sincronização adequada: mediçãoGPU serializada, waits de instrumentação fora de seleção; teste/clippy/MCP não concorreram com timings finais.
8. Invalidação adequada: assinatura do shader, payload, modelo anterior e adaptador checados na carga. Política congelada antes do teste; fora das classes numéricas válidas retornafalse. Não há generalização de custo garantida a outro viewport/cena.
9. Correção verificada: validade/corrupção/limiteideal em três testes novos; baseline antigo e fallback exatos; erro e movimento medidos. UI e aceite humano separados.
10. Custos conscientemente mantidos: memória de todos os comparadores, tabela64KiB no testeforçado e material/normais restantes. Não promover resultado negativo nem trocar contratos funcionais.
