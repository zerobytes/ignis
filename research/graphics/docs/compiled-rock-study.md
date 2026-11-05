# Pedra compilada: respostas binárias consultáveis

Protocolo de27/10/2026, antes da implementação. Pedra Blender procedural existente,81.920 triângulos/relevo real/material GGX roughness0,88,metallic0. Fonte e contratos funcionais preservados. Investigar se um build mais caro pode trocar avaliação de material por consulta direta a respostas compactas.

## Representação experimental

Um arquivo binário versionado little-endian com cabeçalho validado, mapas ortográficos de código de superfície+profundidade (u16+u16), blocos8×8 vazios omitidos, diretório u32 e dicionário por vista. O build agrupa normal/cor por median-cut; somente as respostas RGB e cobertura quantizadas10:10:10:2 por direção de luz são distribuídas. Posição, normal, cor base, triângulos e árvore do compilador não são necessários ao consumidor. Cada campo distribuído tem consumidor; cabeçalho valida/layout/domínio, diretório localiza blocos, profundidade reprojecta, código consulta resposta.

Dois orçamentos:16vistas/16luzes/128²/256códigos e32vistas/32luzes/256²/512códigos. Elevação da câmera35° e luz55° fixas, rotação azimutal periódica. Material/intensidade fixos; sem sombras/GI/transparência/deformação. Uma camada por vista não resolve desoclusão geral. Refinamento angular/espacial e precisão são variáveis distintas; não alegar compressão sem perdas.

Runtime: câmera em perspectiva, seleção de duas vistas e duas luzes, reprojeção de profundidade com número fixo de consultas, interpolação espacial e angular, um passe sobre retângulo conservador. Nenhuma BRDF, normalização de normal de superfície, scan da malha, JSON ou geração do dicionário no consumidor. Ler/validar/upload uma vez, buffers residentes. Não elimina aritmética de projeção/interpolação nem promete mínimo global de custo.

Alternativas: atlas completo view×light cresce muito; surfel médio anterior perde relevo e ainda avalia BRDF; dicionário de respostas compartilha o cálculo entre pixels. O recorte é reversível e isolado, não formato público da engine. Um decoder binário separado da comparação deve produzir imagem idêntica carregando apenas o arquivo.

## Protocolo de avaliação

Duas resoluções físicas3456×2234 e1728×1117; distâncias4/24/96; câmera/luz/ambas, nove amostras incluindo retorno, ângulos fora da grade. Mesma função de material e fonte da referência,1spp; controles integrados selecionados e comparação temporal separada. Gates preservados RMSEforeground≤0,005,p99RGB≤0,02,cobertura divergente≤0,0005. Não ajustar limites após ver resultados. Relatório de imagens offline, recortes físicos; não é renderer de navegador nem aceite humano.

Medir build/carga/payload, memória dos targets e instrumentação separadamente. Benchmark intercalado com controles duplicados, CPU+GPU amortizado e timestampsGPU, sem bake/readbackPNG dentro do intervalo. Registrar todas as amostras e ruído, sem tabela de poses vencedoras. Testar arquivo truncado/corrompido, limites e retorno. Sem promessa de FPS ou ganho antes da medição.

## Checklist prévio

1. Build separado, somente consulta porframe.2. Vetores/alocações limitados na preparação; renderer reutiliza recursos.3. Binário LE/u32 para GPU, profundidade/código16bits, resposta10:10:10:2; nenhum dado de autoria no payload.4. Material computado uma vez por código/luz/vista; acesso direto por bloco.5. Domínio e orçamentos finitos e validados antes do upload.6. Um passe, mesmos targets da referência; readback só verificação.7. Sem thread/lock/dependência nova.8. Versão/checksum/assinatura de fonte no build, domínio rejeitado quando incompatível.9. Testes do decoder, captura, medidas, regressões Rust/MCP.10. Custos mantidos: quantização/dicionário, reprojeção, silhueta, leituras indiretas e memória; torná-los visíveis nas medições.

## Contrato binário implementado

Extensão `.irk`, magic `IRK1`, versão1, little-endian, cabeçalho68bytes. Os15 primeiros u32 definem magic/versão, lado do mapa, número de vistas/luzes/códigos, extent/centro/elevações f32 e tamanhos das três seções; os dois últimos guardam FNV64 do cabeçalho útil+payload (integridade acidental, não autenticação). Decoder limita arquivo a64MiB, confere dimensões/domínio, tamanhos, offsets, ausência de tiles duplicados/inacessíveis, IDs de resposta e cobertura antes do upload. JSON não é parte do asset.

| Seção | Layout | Consumidor |
|---|---|---|
| Diretório | u32 por bloco8×8/vista;0xffffffff vazio | Endereçamento direto de bloco |
| Mapa | u32 por texel: código16bits + profundidade16bits | Busca de resposta e reprojeção |
| Respostas | u32 RGB10:10:10 + cobertura2bits; RGB linear em[0,2] | Interpolação de luz/cor e cobertura |

O build agrupa normal+cor por median-cut independente por vista. Guarda respostas de255/511 códigos de superfície; código0 vazio existe somente nos mapas, calculando respostas para16/32 luzes; o verificador exige que todos os códigos de superfície sejam referenciados. Normais, cor base, árvore de partição e malha não constam do binário. Código0 marca vazio nos mapas; sua entrada de resposta reservada foi removida porque o shader a ignorava. Não há registro de resposta vazio distribuído. Não é dicionário universal entre assets, não é codec sem perdas, e os dois presets mudam simultaneamente resolução, direções e capacidade do dicionário: não isolam causalidade de cada variável.

A GPU recebe três buffersu32 residentes. VetoresCPU dos payloads são liberados após o upload. Uniform por desenho256bytes, incluindo alinhamento necessário, duas bases de vista e matrizes. A consulta faz duas aproximações de profundidade por vista e filtro bilinear final, interpolando duas vistas×duas luzes. No caminho ocupado são até12 consultas de mapa,12 de diretório e16 de resposta por fragmento (40 palavras/160bytes lógicos); cache, reuse e transações reais não foram medidos por contador de banda. Não equivale a160bytes saindo daDRAM por pixel. Há projeção, divisões e interpolação, mas nenhuma BRDF, normal de superfície ou triângulo original consultado pelo shader compilado.

O retângulo conservador reduz fragmentos fora do objeto, mas inclui vazios. Um passe, sem acumulador de viewport. O comparador usa colorRGBA32F+depth32 para a referência; o consumidor isolado precisa apenas de colorRGBA32F para este teste de objeto único, com recursosdummy mínimos herdados da infraestrutura. ReadbackRGBA32F é diagnóstico fora do intervaloGPU/throughput. O consumidor não produz profundidade de cena para compor múltiplos objetos; essa capacidade e sua medição permanecem futuras, não estão implicitamente resolvidas pelo mapa de profundidade interno.

A reprojeção inicia no plano frontal conservador do volume, não no plano central que rejeitava raios de borda cedo demais. Duas iterações são aproximação fixa, não interseção exata nem garantia de cobertura. Material usa direção de vista central do bake ortográfico; câmera de avaliação é perspectiva. A divergência local de direção, a quantização, os grupos de normal/cor e a camada única são fontes de erro explicitadas. Não ajustar os gates para ocultá-las.

## Reprodução

```
CARGO_TARGET_DIR=/Volumes/Zero/build-cache/ignis-graphics python3 scripts/run_compiled_rock.py
```

Etapas CLI independentes: `--compile-rock`, `--compiled-rock-load`, `--compiled-rock`, `--compiled-rock-smoke`, `--compiled-rock-bench`. `load` não lê malha/JSON e é executado pelo runner em diretório temporário contendo somente links para os dois binários; seu resultado deve coincidir exatamente com o comparador. JSON escrito por essa etapa contém apenas evidência de carga/captura, não é uma entrada. O runner remove automaticamente a área temporária e mantém caminhos correntes. Comparador exige hashes atuais da fonte/shaders e checksums do asset, enquanto o decoder independente valida somente o formato/domínio do arquivo.

Relatório `artifacts/appearance-probe/compiled-rock/index.html`: controle de resolução, distância, movimento, representação e posição; playback de capturas e recortes512² compensados porDPR. Tempos exibidos para pose combinada inicial, explicitamente independentes da pose visual selecionada. Integração64spp cobre apenas seis poses iniciais, não o percurso inteiro.

## Controle de custo acrescentado após a primeira medição

A primeira rodada filtrada mediu aproximadamente0,56ms perto contra0,23ms da fonte na pose combinada inicial nativa. Para testar se o gargalo era somente a consulta/reconstrução e não geração da resposta, foi acrescentado um decoder direto dos MESMOS binários: escolhe vista/luz mais próximas, lê profundidade e corrige a projeção uma vez, sem filtro bilinear/angular. Até cinco leiturasu32 lógicas por fragmento ocupado (dois diretórios, dois mapas, uma resposta), contra40 da versão filtrada. Não há payload extra. Código direto ainda projeta raios e resolve divisões; não significa lookup grátis ou custo mínimo matemático.

A comparação final repete todos os controles: fonte, compacto filtrado, detalhado filtrado, compacto direto, detalhado direto. O filtro foi retirado somente no controle identificado, sem alegar preservação de qualidade. A imagem mais rápida pode exibir aliasing e saltos angulares; critérios e referência permanecem iguais. Resultados correntes substituem a primeira rodada, sem coleções de binários/builds históricos.

## Resultado final — Apple M4 Pro / Metal

### Preparação, formato e memória

| Preset | Arquivo | PayloadGPU | Contra fonte2,500MiB | Build |
|---|---:|---:|---:|---:|
|16vistas/128²/16luzes/255respostas por vista/luz|789.828B (0,75324MiB)|789.760B (0,75317MiB)|−69,87%|84,89ms|
|32vistas/256²/32luzes/511respostas por vista/luz|5.944.644B (5,66925MiB)|5.944.576B (5,66919MiB)|+126,76%|271,81ms|

Cabeçalho68B fica no loader. Compacto: diretório16.384B + mapas512.256B + respostas261.120B. Detalhado:131.072B +3.720.448B +2.093.056B. Não armazenar a resposta reservada0 poupou1.024/4.096B; código0 continua somente no mapa para representar vazio. Todos os códigos de superfície e todos os blocos armazenados são alcançáveis. Não foi medido nem alegado formato globalmente ótimo.

Loader isolado dos dois arquivos:8,80ms leitura+validação;84,35ms criação de recursos/pipeline de consulta+upload+espera. Esta segunda duração NÃO é upload puro, nem custo por frame. InstânciaGPU/pipelines comuns criada antes desse intervalo. Build inclui gbufferGPU/capturasCPU, median-cut, avaliação das respostas, quantização, serialização/validação e gravação em cache doSO; exclui inicialização compartilhada do dispositivo/pipelines, semfsync e sem benchmark controlado de tempo de build. Não comparar esses custos com a preparação do símbolo antigo como se fossem o mesmo trabalho.

A consulta usa um passe, sem alvo de acumulação de viewport (o anterior consumia58,90MiB nativos). O comparador mantém ambas as variantes mais a fonte, colorRGBA32F117,81MiB e depth32 29,45MiB na resolução nativa, além de readback117,81MiB para evidência. Pequenos recursosdummy/pipelines/allocator/driver são adicionais; nenhumRSS/VRAM de processo medido. No decoder isolado a profundidade de referência é1×1. UsarRGBA32F mantém a mesma referência numérica do laboratório; não é recomendação de formato final de framebuffer. Ambos os binários ficam residentes para comparação, sem streaming.

### Custo medido

24coortes:18 principais (2resoluções×3distâncias×3poses) e6 repetições da pose inicial; cinco métodos×dois controles,7rodadas×32frames. Ordem rotacionada/reversa, sem bake/capturaPNG dentro dos intervalos. Médias das quatro medianas de controles/rodadas da pose inicial combinada nativa:

| Distância | Fonte total/GPU ms | Compacto filtrado total/GPU | Detalhado filtrado total/GPU | Compacto direto total/GPU | Detalhado direto total/GPU |
|---|---:|---:|---:|---:|---:|
|4|0,2383 /0,1955|0,5862 /0,5419|0,5812 /0,5580|0,2268 /0,2015|0,2367 /0,2063|
|24|0,2397 /0,2102|0,2101 /0,1748|0,2266 /0,1610|0,2173 /0,1640|0,2061 /0,1755|
|96|0,1927 /0,1482|0,1609 /0,1301|0,1603 /0,1300|0,1579 /0,1313|0,1759 /0,1333|

A consulta direta reduz bastante o custo contra o filtro, mas perto NÃO demonstra ganhoGPU contra a fonte; os totais ficam próximos. Nas24coortes, mediana de razões pareadas total/fonte0,810 compacto direto e0,789 detalhado direto, com regressões máximas1,166×/1,162×. Filtro tem medianas0,955/0,954, mas máximos2,666×/2,572×. Essa mistura de perto/longe não representa economia universal nemFPS. Duplicatas do mesmo método variaram0,607–1,580× (mediana0,999): diferenças pequenas não sustentam conclusão firme. A fonte neste teste é a malha original f32/u32, não uma malha também compactada/otimizada em novo formato.

CPU envio por frame na pose próxima: aproximadamente0,026–0,029ms. GPU e total foram medidos em blocos separados, permitindo sobreposição; não somar medianas. Sem contadores de cache/banda, reduzir40 para5 leituras lógicas juntamente com menos projeção/filtros isola o custo do decoder como conjunto, NÃO prova gargalo exclusivamente de memória ouALU.

### Fidelidade e validade do teste

162poses,648comparações, nenhuma aprovada simultaneamente nos gates anteriores. RMSEmediano próximo/nativo: compacto filtrado0,02912; detalhado filtrado0,02743; compacto direto0,04937; detalhado direto0,04818. Na pose inicial nativa detalhado filtradoRMSE0,02832/cobertura divergente12,46%; direto0,05073/20,70%. Inspeção das capturas confirma alteração de contorno, perda/aliasing de relevo e escurecimento local no direto. Não houve aceite visual humano ou vídeo contínuo. Nove amostras de órbita não provam ausência de popping entre posições; o direto troca vista/luz em degraus por construção.

Controle integrado64spp, seis poses: nativo/perto RMSEfonte1spp0,00317, detalhado filtrado0,02795, detalhado direto0,05053 (convergência16→64=0,00053). Na distância24:0,00972/0,00990/0,02041. Na distância96: fonte0,02918, compacto filtrado0,01869, detalhado filtrado0,02325, compacto direto0,03098, detalhado direto0,03002; convergência0,00360. O filtro pode melhorar a média de cor muito longe, mas não preserva a imagem em geral;64spp não é integral analítica exata.

Na pose inicial nativa a referência cobre651.792/17.391/1.091pixels nas distâncias4/24/96. Dividir TODOS os81.920triângulos por essa cobertura dá0,126/4,710/75,087: são limites superiores, não contagens de triângulos visíveis por pixel. Perto, a malha já tem menos triângulos que pixels cobertos; mapas128²/256² por vista ampliados não podem preservar arbitrariamente detalhes resolvidos. Este experimento não repete226milhões de triângulos nem comprova o platô1×–100×. Uma próxima representação deve preservar a resolução requerida e lidar com múltiplas profundidades/visibilidade antes de promover qualquer economia a ganho mantendo qualidade.

### Verificação e conclusão

642testes Rust release/all-targets, Clippy-Dwarnings, fmt/diff-check aprovados. Render/replay_inspection/selection:13grupos headless.820PNGs íntegros,648comparações,90pares de fechamento idênticos, seis referências integradas e24coortes completas. Decoder em diretório sem fonte/JSON produziu exatamente as quatro imagens do comparador; alteração de luz/câmera e retorno/repetição/+360° aprovados. Remoção da resposta vazia preservou os quatro hashes de imagem da checagem isolada. Testes binários cobrem truncamento, corrupção, checksum, offsets, códigos e tiles duplicados/inacessíveis. UI648combinações, recorte512²/DPR1/1,5/2/3, playback e descarte de carga obsoleta emDOMsimulado. SetePNG do símbolo anterior permaneceram idênticos em smoke isolado, sem alterar o relatório anterior. Nada disso é aceite nativo/humano.

Resultado: build e consulta binária independentes foram demonstrados; payload compacto diminuiu e o controle direto tornou a reconstrução mais barata. NÃO foi demonstrado ganho preservando qualidade. O filtro consome novamente boa parte da economia perto; o direto elimina trabalho sacrificando detalhes/contorno. Resolver isso exige representar melhor o que cada pixel enxerga e controlar erro/resolução, não apenas compactar mais os mesmos dados. Formato/decoder permanecem experimento local, sem contrato público ou integração à engine funcional.
