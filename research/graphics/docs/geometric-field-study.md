# Símbolo geométrico de superfície — protocolo

Recorte EAF-88: comparação incremental com B=135 (seleção global/regional e shading automático espacial), sem trocar o material ou a referência completa. O símbolo guarda alturas da superfície original, com a mesma diagonal triangular em cada célula. Não é atlas de cores nem malha simplificada. Um raio por pixel encontra a superfície e escreve cor e profundidade. Domínio: heightfields estáticos; não representa cavidades, superfícies sobrepostas ou uma pedra inteira sem múltiplas parametrizações.

## Antes de implementar

1. Frequência: compilação extrai o grid e limites uma vez; runtime decide elegibilidade/custo por objeto, interseção somente nos pixels do retângulo projetado.
2. Alocações: payload e bind groups no carregamento; uniformes fixos no frame. Sem construir malhas ou listas por pixel na CPU.
3. Layout: binário versionado com alturas f32, limites e checksum; textura R32Float sem descompressão. f32 preserva relevo; quantização não é parte deste incremento.
4. Repetição: funções de material/normais existentes compartilhadas. Alturas substituem índices e processamento dos vértices; normais continuam compartilhadas com B.
5. Escala: custo em pixels e busca limitada; memória ainda cresce com amostras únicas da superfície. Densidades históricas acrescentam detalhe real e não são subdivisões redundantes.
6. GPU/I/O: upload único; sem JSON de geometria ou readback no frame. JSON apenas configuração histórica e relatório de laboratório.
7. Sincronização: nenhuma consulta GPU para escolher no frame. Calibração offline separada da validação. Tempos incluem submissão e espera amortizada; CPU de submissão informada separadamente.
8. Invalidação: versão/checksum/dimensões/limites validados no load; shader/política fixados para a rodada.
9. Correção: limites conservadores de derivadas certificam interseção única. Vistas não certificadas usam B; iteração limitada com solução analítica do triângulo local. Gates antigos RMSE<=.005, p99<=.02, cobertura<=.0005, retorno e transições. Não confundir amostras aprovadas com prova universal de equivalência rasterizador/ray tracing.
10. Custos mantidos: leitura e interseção por pixel, textura adicional residente, ambos caminhos disponíveis; sombra/GI/animação/deformação e superfície multicamada fora deste recorte. Não declarar ganho antes da medição.

A referência completa, B, símbolo elegível forçado e B+símbolo opcional usam câmera/luz e resolução física idênticas. O caminho forçado também respeita o domínio geométrico; o relatório registra quando é fallback. Câmera perto/longe, rotação, retorno e luz em movimento. Falhas visuais impedem habilitar o símbolo; a calibração de custo não relaxa os gates.

## Representação e domínio

`IGF1` armazena as alturas f32 da malha fonte, dimensões, mínimo/máximo e limites absolutos das derivadas em X/Z. O compilador verifica posições do grid e todos os índices/diagonais; rejeita topologia diferente. Checksum do header e payload detecta corrupção. Normais e material são os mesmos de B, compartilhados; não há atlas de iluminação nem dependência de vistas discretas. O payload é enviado diretamente para R32Float. Os quatro arquivos têm 66.628 / 132.676 / 659.012 / 6.563.908 bytes, incluindo header64. O maior payload acrescenta **6,260 MiB** à comparação; não reduz a memória residente de B, que permanece disponível como fallback. A fonte completa100× ocupa78.704.664bytes em vértices/índices; as normais compartilhadas ocupam26.255.376bytes. Os279.928.680bytes de B incluem níveis/contornos/regiões e normais, antes dos buffers compactados adicionais. Não confundir essas bases nem alegar redução deRSS/VRAM física.

A CPU calcula um limite conservador para todos os raios que atingem o volume:

`rho = (Sx*(abs(eye.x)+6) + Sz*(abs(eye.z)+6)) / (eye.y-hmax)`.

Se `eye.y>hmax+.1` e `rho<.95`, a diferença entre altura do raio e altura da superfície decresce estritamente: há no máximo uma interseção. Cada pixel intersecta a caixa, verifica os extremos e faz busca com bracket, usando a solução analítica do plano triangular quando esta permanece no mesmo plano da superfície. Máximo32iterações, sem alocação ou caminhada por todos os triângulos. Posição do grid e diagonal permanecem as originais; interpolação bilinear de altura não é utilizada. Raios são construídos a partir da rotação da câmera para evitar subtrair coordenadas grandes em ponto flutuante. A projeção deste laboratório usa FOV60°. O símbolo escreve profundidade reversa emDepth32Float.

A certificação é conservadora e específica de heightfields vistos de cima. A câmera100× próxima foi rejeitada em todo o percurso; ali D mantém B. Não houve avanço do platô próximo100× neste incremento. Uma pedra fechada, cavidades, sobreposições, vistas rasantes e silhuetas orgânicas ainda exigem outra representação ou múltiplas parametrizações. O contorno deste teste é a borda do terreno fonte. Este estudo não revalida o atlas IRK1 nem resolve o caso genérico da pedra.

## Comparação e calibração

A=fonte completa0; B=anterior135; D=campo geométrico325 quando geometricamente elegível, caso contrário B; C=335 com certificado geométrico e máscara de custo congelada. A escolha é por objeto inteiro neste recorte. D envia um triângulo de cobertura da tela, limitado pelo retângulo projetado; ele não é um triângulo da superfície. Nos casos ativos não são enviados vértices/índices da superfície. C mantém todos os recursos de B para fallback. Nenhuma mudança em Scene/Command/RenderFrame/MCP ou runtime funcional.

Quatro classes fixas de trabalho: triângulos selecionados por B divididos pela área projetada, limites .05/.25/1. Calibração separada em16/48/192, mínimo3coortes integralmente elegíveis e na mesma classe, benefício>5% e maior que a dispersão de duplicatas. Percursos que cruzam classes não habilitam uma classe isolada. A falta de suporte de calibração também rejeita classes: máscara0 não prova que nenhuma pose poderia beneficiar-se. Nenhuma tabela de vencedores por asset/pose e nenhuma consulta de imagem no frame.

Validação14/28/224 em3456×2234 e1728×1117, quatro fontes1/2/10/100× históricas. Esses fatores mudam as frequências de relevo; não são triângulos por pixel nem subdivisão redundante da mesma superfície. Câmera percorre360°, distância oscila±12% em cada faixa e elevação oscila em torno de.95rad. Luz percorre360°; controles independentes no100× a14e28. São nove capturas por percurso e64poses nos blocos de tempo, não prova de continuidade entre todos os frames nem um deslocamento contínuo14→224. Não há obstáculos, sombras, GI, animação ou múltiplos objetos neste teste.

## Resultado de custo

Apple M4 Pro/Metal. A calibração rejeitou todas as quatro classes; C manteve B em288/288poses e produziuRGBA32F idêntico. Os quatro grupos tiveram somente1/2/2/1coortes integralmente elegíveis e na mesma classe, abaixo do mínimo3; também houve regressões. O teste demonstra qualidade de uma representação geométrica ativa emD, mas ainda não demonstra benefício incremental robusto ou uso misto útil emC.

| 100×,3456×2234 | B total | D total | C total | B GPU/lote | D GPU/lote |
|---|---:|---:|---:|---:|---:|
| Perto14 | .5533ms | .5543ms | .5578ms | .5429ms | .5328ms |
| Médio28 | .3484ms | .5202ms | .3458ms | .3250ms | .4935ms |
| Longe224 | .1679ms | .1931ms | .1675ms | .1455ms | .1789ms |

Em14, D é fallback em64/64frames: pequenas diferenças não são ganho do símbolo. Em28, D usa símbolo em58/64frames e custa49,3%mais no total. Em224, símbolo ativo64/64 custa15,0%mais; repetição .1625→.1923ms (+18,3%). A busca por pixel e leituras da superfície detalhada continuam custando mais que o trabalho residual deB nesses casos, apesar de eliminar os triângulos enviados nos frames ativos.

Razão mediana D/B nas24coortes:1,1664; repetição16perto/longe:1,0671. Essas medianas incluem fallback, portanto não são custo puro do símbolo. C/B: .99976/.99615, mesma imagem e trabalhoGPU, sem ganho atribuível. Duplicatas totais variaram de.7748a1.1805; não prometer regressão zero. CPU_submit no100×médio: B.03158ms/D.03000ms; os intervalos deCPU/GPU se sobrepõem e não devem ser somados.

Auditoria:100×médio nativo teve latência média de passeD3,0789ms, mas janelaGPU/lote .4923ms e parede instrumentada .5398ms, com até28sobreposições em31pares. Corrigir essa agregação foi necessário para evitar uma explicação falsa do custo. O throughput sem queries .5077ms confirma que a marcação não produziu um custo físico de3ms porframe. As medições finais já usam janelaGPU, preservando a latência bruta como diagnóstico.

O próximo experimento fundamentado é uma hierarquia geométrica compilada que limite o intervalo de busca por região/escala e deixe detalhes visíveis na representação apropriada. O campo atual elimina rasterização de microtriângulos, mas ainda consulta a superfície fina para encontrar cada pixel. Não promover este candidato, nem somar a redução de triângulos ao ganho anterior como se fossem percentuais de tempo.

## Evidências visuais

288poses,864comparações aprovadas nos gates originais. D usou o símbolo em196poses;92foram fallback geométrico. B e C tiveram cobertura exata contra a referência nas amostras; D teve divergências pontuais de cobertura, máximo0,013716% (gate0,05%). Não alegar identidade pixel a pixel ou ausência universal de artefatos.

| Caminho | RMSE máximo | p99RGB máximo | Resíduo temporal máximo |
|---|---:|---:|---:|
| B anterior | .00319861 | .01487812 | .00361047 |
| D símbolo elegível | .00284727 | .00166962 | .00265885 |
| C opcional | igual aB | igual aB | igual aB |

18casos de profundidade,16com símbolo ativo, verificam cobertura de depth igual ao alpha e erro máximo de profundidade normalizada<9,69e-7 nos pixels de cobertura comum. Isso valida escrita/profundidade neste domínio; não substitui um teste de composição com objetos móveis.96comparações de retorno de imagens fecharam exatamente;8capturas da baseline antiga permaneceram idênticas. A UI mostra imagem ajustada e recortes com1pixel físico por pixel de origem, com DPR explícito. Inspeção visual automatizada e gates numéricos não são aceite humano.

## Instrumentação de performance

Oito rodadas, dois controles por caminho, ordem alternada. Tempo total é throughput CPU+GPU com64submissões e espera final; CPU_submit é tempo de parede de preparação/submissão antes da espera (inclui possível backpressure do driver), não tempo deCPU puro nem parcela somável àGPU. Compilação, upload, capturas e leitura de queries ficam fora da medição.

A auditoria distinguiu latência por passe de throughputGPU: somar intervalos de passes que se sobrepõem pode produzir números maiores que o tempo real do lote. O relatório conserva `pass_latency` e `gpu_overlap_count` e calcula `gpu` pela janela do primeiro início ao último fim de32frames, dividida por32. `instrumented_total` mede o tempo de parede do mesmo lote. Não somar latências sobrepostas nem interpretar a antiga discrepância como custo físico adicional ou ganho. Os números finais de calibração/validação/repetição são gerados com essa correção.

## Reprodução

`python3 scripts/run_geometric_field.py` compila dados/binário, calibra, congela política e hashes, captura288poses, verifica profundidade, mede e repete perto/longe, audita timestamps e valida relatórios/UI. Usa os inputs existentes dehybrid-scaling, near-plateau eautomatic-shading; nenhuma recomputação de geometria em runtime. Saídas correntes em `artifacts/appearance-probe/geometric-field/`. Os scripts verificam formato/checksum, hashes, qualidade, seleção/fallback, imagens, retorno, controles e matriz de medições.

## Revisão estática após implementação

1. Build extrai fonte e certificado uma vez; frame decide por objeto, sem percorrer mesh.
2. Textura/bind/uniform persistentes; alturasCPU liberadas após upload. Encoders existentes mantidos; captura/readback alocam somente na instrumentação.
3. f32 preserva amostras; binárioLE validado e uploadR32Float direto. Normais anteriores compartilhadas; header/política não trafegam por fragmento.
4. Shading reutiliza shader anterior; cada pixel ainda busca a superfície e interpola normais. Esse trabalho adicional é medido, não declarado gratuito.
5.32iterações limitadas, payload proporcional ao grid, quatro classes de custo. Sem histórico crescente. Fonte100× próxima ainda usa geometria.
6. Scissor conservador e um passe com profundidade; nenhuma leitura de pixels/arquivo na decisão. Instrumentação de depth é opt-in no laboratório.
7. Sem thread/lock/dependência; GPU medida separadamente de testes/MCP. Intervalos sobrepostos não são somados como throughput.
8. Header/payload/política com versão/checksum; política inclui dados, shader, modelo de custo anterior e adaptador. Manifest independente fixa fontes do experimento. Não é loader de assets hostis de produção.
9. Binário válido/corrompido/truncado e domínio de raios testados; fonte/B/símbolo/fallback, resolução física, luz, câmera, profundidade e retorno comparados. Gates não relaxados.
10. Mantidos com propósito: normais e buffers deB, payload adicional, busca por pixel e certificado conservador. Sem promessa de formato ótimo, zero custo de invisíveis ou renderer geral pronto.

## Validação final

648testesRust release/all-targets, Clippy com-Dwarnings, fmt e diff aprovados.13gruposMCP headless de render/replay_inspection/selection; nenhuma janela da demo tocada.864gates,872PNGs íntegros,96fechamentos,8baselines legados,18casos de profundidade,24+24+16coortes e4coortes de auditoria de timestamps. UI576estados emDOMsimulado, recortes emDPR1/1,5/2/3. Sem aceite humano/navegador nativo ou composição de objetos comprovados. Build corrente externo reutilizado, sem cópias porrevisão.
