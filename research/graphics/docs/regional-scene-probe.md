# Regiões numa cena conjunta — EAF-74

## Decisão experimental e revisão prévia

O atlas angular global de EAF-73 não fornece propriedade de região nem profundidade de composição entre objetos. Recortá-lo em imagens e sobrepô-las à geometria deixaria oclusão incorreta. Para este experimento isolado, preparar seis vistas locais por região com cor, normal, cobertura e profundidade. Reusar GGX em geometria/cache; a luz continua dinâmica, não há iluminação global baked. A representação muda explicitamente: não chamar isto de reutilização sem mudanças do atlas de radiância anterior. Alternativa de simplificação geométrica com fronteiras preservadas segue aberta; cache de múltiplas camadas pode resolver desoclusões, com mais memória e custo. Nenhuma escolha de produção, contrato de Scene/MCP ou dependência nova.

Referência conceitual: [GPU Gems3, True Impostors](https://developer.nvidia.com/gpugems/gpugems3/part-iv-image-effects/chapter-21-true-impostors), sobre representações com profundidade em proxies. Este recorte usa uma camada por vista e não implementa o método multicamada completo do capítulo.

1. Frequência: partição e caches offline persistidos por asset; seleção por região/instância, sem visitar triângulos por frame.
2. Alocações: pré-alocar planos de desenho; buffers/atlas compartilhados por instâncias. Readback/JSON/PNG apenas diagnóstico.
3. Tipos: dados numéricos compilados, índices u32, IDs de região estáveis; cor linear e normal/cobertura separadas.
4. Repetição: reusar fonte e cache entre todas as instâncias; não gerar atlas por pose ou por objeto colocado.
5. Escala: cena finita96 instâncias mais terreno; registrar número de regiões, chamadas e triângulos. Coalescer faixas de geometria adjacentes e desenhar proxies por instâncias.
6. GPU/I/O: depth test/write também nos proxies; cache manualmente filtrado dentro de cada região/mip, sem vazamento entre tiles. GPU da cena inteira, CPU/upload separados, controles equivalentes.
7. Sincronização: execução serial da GPU; sem novas threads/locks. Benchmarks sem readback de imagem no intervalo.
8. Invalidação: assinatura da fonte/versão na carga, câmera/resolução reavaliadas, histerese reiniciada por sequência; arquivos correntes substituídos no mesmo caminho.
9. Correção: propriedade única dos triângulos, bounds/câmera/resize, ida e volta, profundidade, casos mistos no mesmo objeto; medir foreground/por objeto para não diluir erros no terreno.
10. Custos mantidos: uma camada, seis vistas discretas, filtragem de normais não equivale a integrar BRDF, cobertura opaca aproximada. Não prometer ausência de artefatos ou FPS antes de comparar.

## Implementação e composição

`regional_build.rs` agrupa triângulos pelo centroide em células3D de0,3075unidades e pelo material. Cada face inteira tem um único dono; fronteiras não cortam triângulos. São157 regiões na pedra,168 no baú e154 na árvore. A malha de cada asset é indexada e reordenada por região, sem replicação por instância. Regiões pequenas demais para economizar primitivas permanecem como geometria.

Cada região prepara seis vistas axiais16², com quatro amostras por texel, profundidade frontal, RGB linear, normal orientada para a face e cobertura premultiplicados. Mips16/8/4/2/1 são próprios da região; o lookup manual não amostra o tile vizinho. Os payloads e manifestos são persistidos em `cache/<asset>/`, com assinatura da fonte/código de bake e checksum. Mudar a câmera não reconstrói os dados. A versão de build usa uma camada por vista, não um volume de superfícies ocultas.

Na execução, cada instância escolhe a vista axial mais próxima por região. O shader reconstrói posição por quatro iterações de profundidade, amostra os atributos, calcula GGX e escreve profundidade no mesmo attachment da geometria. O normal médio filtrado **não** é uma distribuição de microfacetas nem resposta BRDF pré-integrada. A diferença em relação ao atlas angular anterior é deliberada: caches locais de superfície permitem composição de profundidade com múltiplos objetos e reduzem a dimensão de iluminação do armazenamento, mantendo o custo de material por fragmento.

A seleção visita oito cantos da caixa de cada região usando a câmera/resolução física. Limita a maior aresta projetada e o suporte planar de filtragem. Entrada em resumo exige ambos abaixo de0,75px; qualquer limite >=1px ou plano próximo inválido retorna à geometria. A região em si pode ocupar vários pixels: o cache conserva variação espacial nesses pixels. O bound continua conservador e inclui triângulos ocultos. Não é estimativa exata de erro radiométrico ou visibilidade.

Guardas de material: regiões dos materiais2–4 (metais/vidro de menor rugosidade deste fixture) permanecem como geometria. Antes da guarda, a média de normais produziu picos especulares: no render reduzido, RMSE foreground chegou a0,15115 e diferença RGB de um pixel a56,55 unidades lineares. Não corrigimos isso por clipping de cor ou ocultação na métrica. A guarda preserva a fonte nessas regiões, com teste específico. Fatorar/integrar a resposta direcional é uma investigação seguinte, não capacidade já entregue.

A primeira versão de depth convencional deixou RMSE0,001126 ao inverter a ordem de desenho, concentrado no fundo. A projeção invertida infinita com near0,1, clear0 e comparaçãoGreater reduziu o erro a0,000004405 no render maior e zero no reduzido, sem mudança de IDs visíveis. O resíduo não zero é mantido no relatório; não afirmar igualdade absoluta de todos os fragmentos sob reordenação. Repetir exatamente o mesmo plano é idêntico bit a bit. A precisão distante tem teste matemático além da captura real.

A geometria é enviada em faixas contíguas coalescidas; os proxies selecionados usam uma chamada instanciada. Os dois caminhos compartilham vértices, índices, materiais, câmera, terreno e culling de frustum por objeto. Não há culling de oclusão ou de regiões no baseline, e os materiais são renderizados dos dois lados. Não são comparações com um renderer AAA otimizado. O baseline faz um desenho por instância visível; instancing agrupado da geometria é outro controle útil antes de qualquer extrapolação de produção.

## Cena, percurso e resolução

96 instâncias:32 de cada asset, terreno de32768triângulos e doze faixas de profundidade entre12 e1200unidades. Assets normalizados e escalados de0,75 a2; unidades não são metros de fontes reais. Posição vertical segue a altura procedural do terreno. Nenhuma malha foi artificialmente subdividida para aumentar a vantagem dos resumos.

Câmera com FOV vertical60°, altura7 e movimento longitudinal65→−25→65, com deslocamento lateral suave. São13 posições na ida e13 na volta, incluindo repetição do extremo:26 por resolução,52 capturas de cena. As duas resoluções físicas são3456×2234 e1728×1117, com mesmas poses. **São frames completos**, não recortes512². Luz direcional fixa; sem sombras projetadas, GI, vento, streaming, animação ou pós-processamento temporal.

Há mapas de seleção por pixel e contagens separadas de regiões escolhidas e mistura efetivamente visível. Muitas regiões escolhidas estão ocultas: não usar a primeira contagem como prova de mistura na imagem. O botão de inspeção escolhe um objeto com pixels dos dois caminhos e acompanha sua caixa projetada. Objetos com milhares de pixels permanecem majoritariamente geométricos; a economia aparece sobretudo no fundo. Esta rodada não comprova substituir uma grande parte de um objeto próximo e detalhado.

## Qualidade e medidas

Qualidade principal compara geometria1spp e regional1spp na mesma pose. Referência integrada4spp em vinte posições permite separar parcialmente o aliasing do desenho de uma amostra do erro de representação; não é integração convergida, especialmente nos brilhos da fonte. Erro RGB linear é medido no foreground, incluindo pixels extras e amostras da referência, e por objeto na máscara do controle. Terreno vazio não dilui essa métrica. IDs visíveis expõem diferenças de silhueta/oclusão, sem confundi-las automaticamente com falha de depth buffer.

GPU: nove amostras por método/pose, ordem alternada, controles duplicados G/R/G/R, após aquecimento. Vinte posições cronometradas. Medir o passe completo de terreno+objetos, sem readback/PNG/mapa de seleção. A preparação CPU é repetida nove vezes. O ciclo serial tem cinco amostras e inclui seleção, upload, codificação, submissão e espera; não inclui captura nem I/O. Esse ciclo síncrono não é FPS de uma engine com CPU/GPU sobrepostas. Usar a mediana das razões **pareadas por pose**, não dividir medianas independentes: neste conjunto as duas operações chegam a conclusões diferentes para o tempo serial.

Resultados de26/10/2026, Apple M4 Pro/Metal:

| Render físico | Redução de triângulos | GPU regional/geometria | Serial regional/geometria | Preparação regional CPU | RMSE foreground mediano / máximo | Objetos com mistura visível |
|---|---:|---:|---:|---:|---:|---:|
| 3456×2234 | 37.4–59.9% | 81.9% | 116.0% | 0.330ms | 0.00452 / 0.00915 | 4–9 |
| 1728×1117 | 47.4–74.2% | 69.0% | 125.2% | 0.318ms | 0.00940 / 0.01719 | 4–19 |

A redução de trabalho foi real, e a GPU ficou aproximadamente18%/31% mais barata nas medianas pareadas. **Isso não resultou em ganho do ciclo completo medido:** o custo serial pareado foi16%/25% maior. Preparação do baseline ficou perto de0,0008ms, contra0,32–0,33ms regional; chamadas regionais variaram329–453 no render maior e478–819 no reduzido. Não transformar economia de triângulos ou da GPU em promessa de FPS. Próximos candidatos concretos são seleção/compaction na GPU e redução de submissões, com baseline igualmente agrupado.

Nas posições com referência4spp, medianas foreground geometria/regional:0,093721/0,093733 no render maior e0,141399/0,141460 no reduzido. O aliasing dos brilhos da própria geometria domina parte dessa comparação; não é prova de referência convergida ou fidelidade perfeita. O maior erro adicional contra geometria1spp caiu para0,00915/0,01719 após preservar materiais brilhantes. A árvore e a silhueta de objetos distantes ainda mostram limitações de uma camada/cobertura, visíveis no controle de erro e na inspeção ampliada.

Payload GPU compartilhado dos caches:29,94MiB para os três assets, reutilizado nas96instâncias; aproximadamente9,80/10,49/9,62MiB por asset. Vértices+índices, incluindo terreno:5,32MiB. Metadados de região e buffer de instâncias são adicionais. Esses valores não incluem targetsRGBA32F/depth, driver, staging, readback ou memória CPU da instrumentação. O cache de cor/normal/profundidade não deve ser comparado como compressão lossless do atlas de radiância anterior: mudou a representação e o custo de shading.

## Reprodução, apresentação e validação

```sh
# No worktree gráfico; usa as malhas de EAF-72 já exportadas.
cargo run --locked --release --example appearance_probe -- --bake-regional-scene
cargo run --locked --release --example appearance_probe -- --regional-scene
python3 scripts/check_regional_scene.py
python3 scripts/report_regional_scene.py
node scripts/verify_regional_report.mjs
cargo test --locked --release --all-targets
cargo clippy --locked --release --all-targets -- -D warnings
cargo fmt --all -- --check
python3 scripts/run_verifications.py --only render replay_inspection selection
```

Relatório corrente: `artifacts/appearance-probe/regional-scene/index.html`; dados em `report.json`, `summary.json`, `verification.json` e caches assinados.52poses/228PNGs,611 testes Rust (nove novos), Clippy/fmt e três cenários MCP/13grupos headless aprovados. O verificador de artefatos confirma dimensões, percurso/resolução, mistura visível, limite subpixel, compartilhamento do cache e ausência dos grandes picos especulares observados antes da guarda. O harness executa o JS entregue em52estados,12combinações de DPR/ampliação, clique para recorte, reprodução e descarte de carregamentos antigos. A visualização principal é reduzida e assim rotulada; recortes512² são1:1 físico por DPR ou2×/4× explícitos. A referência4spp aparece nos recortes quando disponível.

Inspeção local dos frames completos e mapas confirmou objetos próximos geométricos e escolha regional no fundo. Não confundir inspeção de PNGs com aceitação da página nativa ou continuidade temporal percebida. A automação anterior de `file://` foi bloqueada pelo navegador e não foi contornada. A página tem validação de código/capturas, com apresentação nativa e aceite humano pendentes. Engine, formatosScene/Command/RenderFrame/MCP, demonstração funcional e branch principal não foram alterados.

## Revisão final de performance

1. **Adequado:** bake/cache uma vez, seleçãoO(instâncias×regiões) com oito cantos por região; custos medidos em CPU, sem varredura de triângulos por frame.
2. **Corrigido:** planos e estatísticas tipadas reaproveitam capacidade, sem JSON/string no seletor; caches por asset movidos ao buffer combinado sem conservar cópias duplicadas do payload. Capturas e métricas offline mantêm alocações deliberadas.
3. **Adequado:** índices/IDs u32, metadados POD, atributos lineares e premultiplicados; estado bool por região/instância e hash antes de upload. Depth invertido protege a precisão no fundo.
4. **Corrigido:** bounds de asset preparados, baseline pula a seleção regional, faixas contíguas coalescidas, um desenho instanciado para todos os proxies. Cálculos de projeção por região ainda se repetem e custam~0,33ms.
5. **Mantido com motivo:**479regiões,96instâncias e percurso finito. Seleção CPU plana e centenas de chamadas não comprovam escala de mundo aberto. Não ampliar objetos apenas para fabricar vantagem.
6. **Adequado:** cache e geometria GPU compartilhados, targets substituídos ao mudar resolução, captura fora dos cronômetros. Upload da lista selecionada é medido; recomposição por frame permanece custo conhecido.
7. **Adequado:** nenhuma thread/lock/dependência nova. GPU benchmark e regressões MCP executados separadamente; controles repetidos registram variação.
8. **Adequado:** assinatura/checksum da fonte/bake; câmera/resolução/histerese reavaliadas. Saídas em caminho único corrente, sem builds por revisão; experimentos anteriores preservados como controles em uso.
9. **Corrigido/validado:** partição única, mips premultiplicados, profundidade/ordem/repetição, projeção, mistura no mesmo objeto, guarda especular, métricas de foreground incluindo cobertura fracionária, CPU/GPU/capturas e regressões. Testes não substituem aceite visual.
10. **Mantido com motivo:** seis vistas discretas, uma camada, filtro de normais, materiais brilhantes preservados, sem sombras/GI/streaming e com overhead CPU explícito. Próximos passos exigem confrontar erro por região e custo total; nenhum ganho de FPS ou superioridade sobre engines existentes foi demonstrado.
