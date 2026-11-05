# Segunda malha externa e referência integrada — EAF-88

## Protocolo e revisão antes de implementar

Importar Wooden Stool02 de Kuutti Siitonen, Poly Haven, CC0: https://polyhaven.com/a/wooden_stool_02. Modelo com vãos, pernas e dois primitives, um material. Importar integralmente posições/normais/índices; normalizar escala/eixos como na pedra. Texturas/UV/PBR originais não fazem parte deste recorte. Fixar hashes dos arquivos glTF/bin e registrar contagens, sem selecionar apenas um primitive.

Gerar1/2/10/100× por subdivisão afim da mesma superfície, embaralhar antes de compilar; compilador recebe somente malha, sem base/genealogia. Comparar melhor compilação anterior (legada/refit) com bordas opcionais, fonte completa e base1×. Seleção offline por menos faces sem mais bytes, gates anteriores de qualidade/projeção. Reutilizar protocolo perto/longe, duas resoluções, luz/câmera variáveis, duplicatas. Não extrapolar aumento de subdivisão redundante para novo detalhe visível.

Referência integrada diagnóstica: jitter estratificado4/16/64 amostras por pixel, acumulaçãoRGBA linear sem normalizar por cobertura. Manter viewport completo e ler apenas ROI central distante; não recortar matriz de projeção. Comparar fontes/base/compilado na mesma grade e a convergência16→64. Não tratar64 amostras como integral exata nem tempo dessas capturas como custo do renderer proposto. Aplicar também às quatro poses divergentes da pedra. Não relaxar gates anteriores de captura1spp.

Checklist: (1) compilação offline, jitter apenas diagnóstico; (2) acumulação de ROI e reaproveitamento de buffersGPU, sem atlas; (3) IGM1/IGC1 bináriosf32/u32, JSON só import/relatórios; (4) reutilizar compilador/benchmark e pacotes, sem genealogia no algoritmo; (5) medir contagens/memória/build e manter fallback; (6) readbackROI fora das medições, shader inalterado; (7) regressões separadas da coletaGPU; (8) hashes fixos e invariância de projeção sem jitter; (9) validar importer, codec, retorno, integrador e regressões; (10) PBR, cenas complexas e outros hardwares permanecem fora deste ensaio.

## Implementação e reprodução

`scripts/import_external_stool.py` fixa glTF MD5 `44db244a27f82a7a8536c689e2995e79` e BIN `ea27eb52e71c40319d9ca108843b19a0`; registra SHA256 no import.json. Ambos os primitives são importados:6.299 vérticesglTF,5.571 após união exata posição+normal,11.146T. Índicesu16 convertidos parau32 noIGM1; material0 preservado. Sem texturas baixadas. O banco possui geometria explícita de pernas/vãos e componentes separados, contrastando com o volume irregular da pedra, mas não constitui ensaio de PBR diferente.

`external_recovery_probe --build` compara legado/refit, conserva o menor e depois testa bordas. A referência1× é a fonte importada inteira, não o proxy1×. O gerador conhece a base para fazer o controle; o compilador só recebe as faces subdivididas e embaralhadas. Os formatos binários existentes não mudam.

```sh
python3 scripts/import_external_stool.py
cargo build --locked --release --example external_recovery_probe --example recovery_probe
target/release/examples/external_recovery_probe --build artifacts/appearance-probe/external-recovery
target/release/examples/external_recovery_probe --integrate artifacts/appearance-probe/external-recovery
RECOVERY_FALLBACK=1 target/release/examples/recovery_probe --run artifacts/appearance-probe/external-recovery artifacts/appearance-probe/external-recovery
python3 scripts/report_recovery_stability.py --external
node scripts/check_recovery_stability_ui.mjs --external
```

`--integrate` reutiliza também os pacotes de `border-recovery` da pedra. O viewport permanece1728×1117 comROI=[832,526,64,64], ou3456×2234 comROI=[1664,1053,128,128] na segunda pose problemática do banco. Jitter uniforme estratificado em[-0,5;0,5]px, média linear deRGBA, incluindo zeros de fundo. A projeção sem jitter segue exatamente a expressão anterior; não aplicar matriz de identidade extra nesse caminho. Para cada uma das dez poses, a ROI sem jitter deve ser idêntica à região da captura completa. Nenhum pixel coberto pode tocar a borda da ROI em nenhuma amostra. Os dois invariantes interrompem o experimento se falharem.

A extensão de captura valida retângulo com checked_add e jitter finito dentro de meio pixel. Nenhuma mudança no shader, resolução, filtro ou seleção do benchmark. A imagemPNG recebe conversão sRGB somente depois da média; as métricas usam floats lineares e cobertura fracionária.

## Geometria e memória de build

| Fator | Fonte T | Melhor anterior T | Escolhido T | Bytes proxyGPU | Bytes fonteGPU | Arquivo escolhido B | Compilar três alternativas |
|---|---:|---:|---:|---:|---:|---:|---:|
| 1× | 11.146 | 11.138 | 11.138 | 267.264 | 267.456 | 1.157.576 | 20,25ms |
| 2× | 22.292 | 13.910 | 11.140 | 267.312 | 568.200 | 1.503.104 | 51,39ms |
| 10× | 111.460 | 11.226 | 11.226 | 269.376 | 2.674.992 | 3.968.832 | 203,81ms |
| 100× | 1.114.600 | 11.280 | 11.280 | 270.672 | 26.750.352 | 32.058.360 | 2.536,35ms |

Bordas opcionais são escolhidas somente em2×; os outros candidatos preservam os melhores anteriores. De1× a100×, o proxy cresce1,3% emfaces/bytesGPU, enquanto a fonte continua residente parafallback. Portanto não há platô da memória total do pacote:100× mantém25,51MiB da fonteGPU e arquivo30,57MiB. Proxy100×≈0,258MiB. RSS não medido; soma de source/proxy não inclui targets, staging, buffers de comparação ou custoCPU.

## Diagnóstico de integração

As quatro poses distantes que falhavam na pedra foram mantidas (fator/frame:2/4,10/1,100/1,100/2). Fonte subdividida versus base na mesma grade:

| Pose | RMSE1spp | RMSE64spp | Redução | Convergência fonte16→64 RMSE |
|---|---:|---:|---:|---:|
| Pedra2×/4 | 0,00535895 | 0,00045349 | 11,8× | 0,00331256 |
| Pedra10×/1 | 0,00770341 | 0,00022143 | 34,8× | 0,00401468 |
| Pedra100×/1 | 0,00769396 | 0,00030992 | 24,8× | 0,00402153 |
| Pedra100×/2 | 0,00721568 | 0,00041835 | 17,2× | 0,00392278 |

Compilado versus base64spp: máximoRMSE9,54e-7 nessas poses. Isso reforça que a discrepância1spp da fonte densa está dominada por amostragem/rasterização, não por perda macroscópica de relevo no compilado. Não isola a regra de rasterização ou quantização exata daGPU;64spp ainda tem incerteza maior (convergência16→64≈0,003–0,004) do que a diferença entre representações. Não chamar64spp de verdade física nem converter este diagnóstico em fechamento das quatro falhas anteriores.

Quatro poses distantes do banco: fonte versus base64spp RMSE9,34e-5..3,57e-4, compilado versus base≤2,04e-7. A discrepância da fonte não diminui monotonicamente em relação a1spp; amostrar mais pode revelar diferenças que o centro do pixel não capturava. Convergência16→64≈0,00255..0,00332. Cobertura fracionária conservada, sem normalizar a cor apenas pelos samples que acertam a superfície.

Os números integrados cobrem dez poses distantes,40linhas de métricas e60PNGs deROI. Não substituem a avaliação completa perto/longe1spp nem demonstram estabilidade temporal contínua. Custo de integração/readback excluído do benchmark e mantido explicitamente como diagnóstico offline.


## Ajuste do protocolo após o gate

A primeira execução foi interrompida pelo certificado em100×/3456/perto antes de concluir os resultados. Não houve relaxamento de erro nem aprovação visual inferida. O laboratório anterior só aceitava medir trajetórias integralmente dentro do domínio; agora `RECOVERY_FALLBACK=1` implementa o comportamento requerido: a cada pose, `Bound::choose` usa erro projetado≤0,15px, erro de normais≤0,000101 e redução de faces; fora disso desenha a fonte completa.

Todas as quatro vias temporais usam o mesmo novo coletor de draws por material, com fonte/base diretas e antigo/candidato adaptativos. Contagem de `proxy_frames` e custo da decisão estão dentro do lote medido. Capturas usam a mesma decisão, registrando bound candidato, bound selecionado, fallback e triângulos realmente desenhados. O código antigo de medição continua disponível por padrão; não comparar diferenças absolutas entre sessões como ganho desta mudança de instrumentação.

As capturas parciais são sobrescritas pela coleta final; os tempos interrompidos não entram no relatório. A necessidade de fallback é um resultado do ensaio, não um motivo para esconder a pose desfavorável.


## Auditoria do bound global

`scripts/audit_external_certificates.py` valida checksum/versão/tamanho doIGC1 e compara em f64 os certificados existentes. Banco100×:11.120regiões, maior erro espacial4,06009e-5. Em3456×2234/distância2,7,20das32poses ultrapassam0,15px sob o bound global, máximo0,173439px. Nenhuma região individual ultrapassa o limite; máximo local0,067353px.

O bound global mistura a maior incerteza de qualquer região com os extremos de todo o objeto. Esta é uma restrição excessiva demonstrável pelos próprios certificados; não indica necessariamente detalhe que precise retornar à malha densa. O auditor éoffline/f64, não um seletor novo medido. Percorrer11mil regiões por frame seria outro custo relevante. Próxima hipótese: compilar grupos/hierarquia de certificados para usar limites locais com poucas consultas, sem relaxar erro e sem trocar o objeto inteiro por fonte densa. Nenhum ganho dessa hipótese está incluído nos tempos atuais.

## Coleta temporal e qualidade final do banco

A coleta com fallback concluiu144poses e1.024amostras temporais.144/144passam contra a malha-base. Contra a fonte densa,142/144passam:100×/1728/longe/frame5 (cobertura0,00115607) e100×/3456/longe/frame7 (cobertura0,00060680). As duas também aparecem no controlebase versus fonte, e são mantidas explicitamente. Em1/2×, compilado e base têm floats idênticos em todas as36poses de cada fator; em10× máximoRMSE1,00894e-6; em100×0,00038511 incluindo fallback, cobertura máxima6,87e-6 contra base.

Resolução3456×2234, tempos totaisms/frame de lote:

| Fator / câmera | Fonte completa | Melhor anterior | Escolhido adaptativo | Base na coorte | Proxy frames /128 |
|---|---:|---:|---:|---:|---:|
| 1× / perto | 0.23719 | 0.24511 | 0.24795 | 0.24036 | 128 |
| 1× / longe | 0.18775 | 0.17692 | 0.18403 | 0.18260 | 128 |
| 2× / perto | 0.20626 | 0.21502 | 0.20157 | 0.20746 | 128 |
| 2× / longe | 0.17840 | 0.18466 | 0.17861 | 0.18672 | 128 |
| 10× / perto | 0.27843 | 0.21485 | 0.21516 | 0.21028 | 128 |
| 10× / longe | 0.18721 | 0.19137 | 0.18620 | 0.19645 | 128 |
| 100× / perto | 1.67054 | 1.19177 | 1.21077 | 0.22878 | 48 |
| 100× / longe | 0.47339 | 0.20792 | 0.19928 | 0.20490 | 128 |

O platô não é universal:3456/perto100× entra em fonte completa em80/128frames (20/32poses) e custa1,21077ms, contra0,22878ms da base (5,29×). Ambos os compilados têm o mesmo pacote e a mesma restrição; diferença1,19177→1,21077não é regressão da correção de bordas, e fica dentro da variação das duplicatas10,6%. Fonte completa custa1,67054ms; a seleção ainda poupa27,5%, mas falha em estabilizar o custo perto na resolução maior.

Em1728/perto o proxy é elegível em128/128frames, e o escolhido1/2/10/100× custa0,08677/0,08687/0,09009/0,08885ms. Fonte100×1,19845ms:13,49× o tempo do escolhido, ganho do compilador acumulado, não apenas da correção atual. Longe nativo o100× fica0,19928ms contra0,20490ms da base, dentro da variação. Esses resultados separam platô onde elegível de fallback excessivo no enquadramento crítico.

Incremento de bordas em2× perto nativo:0,21502→0,20157ms (−6,3%),GPU0,16945→0,16186ms (−4,5%),7/8rodadas favoráveis e duplicatas4,7%.1728 perto−4,7%, mas5/8rodadas e duplicatas9,2%: evidência fraca ali. Distante2× sem ganho consistente. Nenhum ganho em fatores com arquivo idêntico atribuído à correção.

Decisão: segunda geometria confirma a recuperação offline e preservação visual dentro dos gates, mas expõe que o certificado global pode destruir o platô. O próximo incremento deve melhorar a consulta de certificados usando agrupamento/hierarquiaoffline, com fallback por região se necessário. Esse seletor ainda não foi implementado nem medido nesta unidade. PBR/texturas, subdivisão com novo detalhe e cenas com muitos assets continuam necessários antes de generalização.


As duas novas falhas de cobertura do banco foram acrescentadas ao diagnóstico nas resoluções/poses exatas:100×/1728/frame5 e100×/3456/frame7. Fonte versus base passa deRMSE0,00116110/0,00364530 em1spp para0,00032061/0,00020252 em64spp (3,6×/18× menores). Cobertura fracionária média divergente64spp0,00025432/0,00014168; compilado versus base64sppRMSE≤1,48e-7 e cobertura idêntica. Convergência fonte16→64RMSE0,00186826/0,00162820: a amostragem continua sendo uma aproximação. As duas reprovações1spp ficam no relatório; não são apagadas pelo diagnóstico64spp.

## Checklist posterior, validação e continuidade

1. Frequência: descoberta/compilação e agrupamento proposto sãooffline. O seletor medido avalia um bound global por pose; fallback não depende de tabela de poses/qualidade observada.
2. Alocações: buffers residentes no benchmark; novo coletor adaptativo usa ranges já enviados por material, sem Vec de seleção por frame. ReadbackROI aloca buffer por sample e médiasCPU apenas no diagnóstico; esse custo não é ocultado dentro do tempo de render.
3. Tipos: f32/u32IGM1/IGC1, formato intacto. Jitter/floats tipados; JSON apenas resultados/proveniência. Auditorf64 separado, checksum/versão verificados.
4. Reuso: fonte/proxy compartilhados entre poses, builds legados/refit/bordas sequenciais. O render não descobre topologia nem integra64amostras durante o benchmark.
5. Escala: fonte e certificados continuam proporcionais à entrada; proxy praticamente constante para subdivisão redundante. Custo100× próximo nativo cresce pelo fallback80/128frames, explicitamente contabilizado.
6. GPU/I/O: shader igual, capturaROI preserva viewport inteiro e só reduz tráfego de readback. Nenhum PBR/textura ou filtro novo no renderer medido. Timestamps e total de lote mantidos separados.
7. Sincronização: Rust/Clippy/MCP fora das coletasGPU; aquecimento, alternância, duplicatas e base por coorte. Relatório mantém variabilidade e caminhos idênticos como controles, sem promessa deFPS.
8. Invalidação: hashes do import, codec de todos os pacotes, fonte idêntica entre vias. Captura sem jitter versusROI verificada bit a bit em dez poses; retorno determinístico em64paresPNG. Retângulos e offsets são validados antes de criar bufferGPU.
9. Correção:692testesRust release/all-targets, Clippy-Dwarnings, fmt e13gruposMCPheadless passaram; recorte final do integrador recompilado/testado novamente após incluir as duas poses.380PNGs íntegros (320benchmark+60ROI),144poses,1.024amostras,40linhas integradas e1.024estadosDOM/DPRsimulados. Contadores conferidos:4poses de captura emfallback e48/128frames comproxy no lote100×perto nativo. Captura próxima inspecionada; nenhum aceite visual humano/navegador nativo alegado.
10. Custos mantidos: fonte parafallback, compilação tripla, certificados, targets e memória de controle. RSSnão medido.64spp não é integral exata; os dois assets ainda não cobrem UV/PBR, transparência, animação ou novo detalhe microscópico não afim.

Artefato corrente:`artifacts/appearance-probe/external-recovery/`, incluindo fonte pequena fixada,IGM1,quatro pacotes anteriores/quatro escolhidos, painéis/relatórios/verificação. A coleta interrompida foi substituída no mesmo caminho; não há binário histórico ou saída parcial necessária pendente. `target/release` reutilizado, sem alteração deNPC/Scene/Command/RenderFrame/MCP/demo, sempush/publicação. EAF-88 continua em investigação; próximo alvo é a hierarquia de certificados locais com custo explícito de seleção.
