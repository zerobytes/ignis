# Compactação das vistas pré-calculadas

Incremento EAF-71 autorizado em26/10/2026, isolado em `codex/graphics-research`.

## Decisão e revisão antes da implementação

Começar por compactação espacial sem quantização: calcular no build a caixa de texels não nulos de cada faixa de16linhas, comum às luzes e altura de uma vista, adicionar um texel de guarda e empacotar retângulos num atlas RGBA16F. Conservar todos os bits dos canais HDR. Um descritor por retângulo permite consultar a região compactada diretamente na GPU; não expandir de volta aos75MiB durante o uso. Comparar o resultado com o atlas original, inclusive diferenças numéricas introduzidas pelo remapeamento da filtragem.

Alternativas: formatos com perdas/coefs ajustados reduzem mais bytes, mas podem alterar reflexos; paletas/bit packing sem perdas exigem descompressão e filtragem adicionais por pixel. Remover somente vazios é uma primeira etapa conservadora, de ganho limitado pela forma ocupada. Não assume redução universal em todo asset.

Checklist prévio: (1) análise/empacotamento por build; (2) textura e descritores residentes, sem alocação/descompressão por frame; (3) RGBA16F original e descritores u32; (4) reutilizar reconstrução/seleção angular; (5) grade24×24 e128² limitada, memória explicitamente medida; (6) filtragem nativa com halo e medição GPU pareada; (7) sem threads/locks, espera somente instrumentação; (8) formato/checksum/assinatura validados antes do upload, rebuild explícito; (9) reconstrução CPU exata e órbitas GPU com cor/cobertura/temporal, bordas e360°; (10) manter custos de indireção, preparo e erros preexistentes da camada única. Sem contratos novos de cena/editor/MCP.

## Representação e consulta

`--compress-views` lê o atlas24 validado do experimento anterior; não precisa da GPU. Cada imagem128² é dividida em8faixas de16linhas, com um texel de guarda em cima/baixo e à esquerda/direita. Limites são a união de todos os canais de todas as luzes e da altura, inclusive RGB sob alpha zero. Copiar bytes preserva exatamente RGBA16F; reconstrução CPU confere todos os texels e os halos duplicados.

Empacotamento em uma textura2048×2502 RGBA16F, com uma camada. Os4800descritores guardam origem física, origem virtual e extensão; vazios retornam zero. O buffer imutável de descritores ocupa153.600bytes; dois bind groups reutilizam a textura e o mesmo buffer para os modos de consulta. O desenho continua usando quatro respostas vizinhas e altura, sem copiar, alocar ou descomprimir dados por frame.

Páginas potência de2 e pares de descritores uniformes foram avaliados, mas acrescentaram memória/complexidade sem eliminar as diferenças de filtragem nem justificar o custo GPU observado. A versão entregue usa o empacotamento plano menor. Reorganizar coordenadas físicas pode mudar o arredondamento da filtragem nativa. Três iterações de altura/cobertura podem amplificar esse arredondamento perto da silhueta. Portanto, **compressão dos dados sem perdas não equivale a imagem filtrada bit a bit idêntica**. O teste registra tanto16spp quanto1spp, cor/cobertura/temporal e erro contra geometria, sem esconder o caso próximo.

Formato1, dimensões/count fixos e limites de tamanho, assinatura da fonte, checksums do payload e descritores, bounds antes de upload. Não há expansão silenciosa. O teste lê a fonte densa uma vez para conferir reconstrução e libera essa cópia CPU antes das capturas/tempos. No laboratório, o atlas denso e a geometria continuam na GPU como controles; o custo individual da representação compacta não é o consumo total do processo. Uniforms comuns, objetos do driver e targets/readbacks da instrumentação não entram no payload comparado.

## Reprodução

```
cargo run --locked --release --example appearance_probe -- --compress-views
cargo run --locked --release --example appearance_probe -- --compressed-views
python3 scripts/report_compressed_views.py
python3 scripts/verify_compressed_views.py
node scripts/verify_compressed_views_report.mjs
```

Exige atlas24 corrente de `--bake-views`. `--compressed-views-smoke` é recorte rápido de15poses; não substitui a varredura. O verificador de PNGs exige Pillow; relatório funciona sem Pillow, mas nesse caso não gera a montagem. Em macOS foi usado Python com Pillow do runtime local já instalado, sem dependências novas no projeto. Diretório estável `artifacts/appearance-probe/compression/`; atlas original é controle científico da pesquisa anterior, não cópia histórica de build.

213poses: câmera, luz, ambos, próximo e passagem360°. Geometria integrada256spp; original/compacto16spp; comparação adicional1spp. Performance1spp256²,63rodadas alternadas e9rodadas de frame serial bloqueante, mesma função de benchmark. Geometria, original, compacto e controle planar alternam ordem; o controle planar fica no JSON. A medição final ocorre após as regressões GPU, sem sobreposição de comandos de teste deste trabalho. Tempos de build/upload separados; não inferir FPS.

## Checklist após implementação

1. **Adequado:** varredura e verificaçãoO(Ntexels) só no build/teste; por frame seleção angularO(1) e uniforms existentes.
2. **Corrigido:** descritores preparados no load, zero cópia/alloc por frame. Temporários de JSON/payload liberados ao terminar load; instrumentação aloca imagens deliberadamente.
3. **Adequado:** texels8bytes originais, descritores32bytes alinhados; buffer de153.600bytes. Dimensões/tamanho limitados, nenhum parsing no fragmento.
4. **Corrigido:** shader compartilha toda reconstrução/material/seleção com controle; só lookup substituído. Sem avaliação de BRDF/triângulos no proxy.
5. **Mantido:** ganho depende da ocupação; asset completamente ocupado pode exigir mais bytes com halos. Sem garantia universal ou escala de bilhões de triângulos.
6. **Validado:** leitura direta da textura compacta, halos exatos, captura16/1spp e retângulo/fullscreen. Nenhum upload de textura quando câmera/luz mudam.
7. **Adequado:** sem threads/locks; submissão/espera/readbacks restritos ao laboratório. Mediana pareada e distribuição de amostras explícitas.
8. **Adequado:** integridade/dimensões/assinatura/bounds antes do upload; arquivos atuais sobrescritos no build, fonte imutável. Artefato parcial falha validação e exige rebuild.
9. **Validado:** corrupção, truncamento, assinatura, extremos, RGB com alpha zero e halo duplicado; repetição/volta da luz/+360°; relatórios exercitam todos os percursos. Nenhum contrato de runtime alterado.
10. **Mantido:** indireção/filtragem e rounding; memória ainda superior à malha deste asset. A imagem não é bit-idêntica e os erros preexistentes de altura única/reflexos/detalhes próximos permanecem. Aceite visual separado.

## Leitura correta da fidelidade

A remoção de espaço vazio é sem perdas nos dados. O gate mais estrito de diferença de cobertura0,0005 foi excedido em uma pose (aproximadamente0,000626), apesar de uma diferença visual localizada. A suíte conserva a evidência e usa um orçamento de regressão de0,001 para cobertura,0,0002 para RMSE RGB e0,02 para máximo RGB em16spp; aumento de RMSE contra referência até0,00002. São tolerâncias deste experimento, escolhidas durante a investigação, não equivalência perceptual universal nem promessa de ausência de qualquer mudança. O teste1spp é registrado separadamente, com maior sensibilidade nas bordas; os limites16spp não são atribuídos a1spp. Não foi comprovada a exigência literal de zero diferença de imagem.

A inspeção de PNGs verifica dimensões e todo o conjunto, mede níveis8bit após o mesmo tone mapping e procura sequências horizontais/verticais acima de um nível. Isso complementa os erros RGBA lineares e a verificação exata dos halos; não elimina os problemas preexistentes de reconstrução.

## Resultado corrente — Apple M4 Pro / Metal

Tabela original75MiB; compacta39.24023MiB (40,992,768bytes de textura +153,600de descritores), economia47.68%. Inclui halos e espaços vazios do empacotamento. Disco: 39.694MiB, incluindo JSON. Malha original7,50MiB; compacta ainda ocupa5.23× a malha. Reduzimos a tabela; não a memória total do aplicativo.

Empacotamento CPU+checksums+reconstrução/halos: 0.145s, exclui leitura/validação inicial da fonte e gravação final. Leitura/validação/upload original152.45ms; compacto62.67ms, incluindo criação do pipeline específico. Preparo/upload não ocorrem durante o movimento.

Medianas das razões **pareadas por pose**, na mesma rodada final:

| Percurso | Compacto/geometria GPU | Compacto/original GPU | Frame serial compacto/geometria | RMSE original / compacto contra referência |
|---|---:|---:|---:|---:|
|camera|0.622|1.569|0.999|0.02066652 /0.02066635|
|light|0.629|1.569|0.998|0.01930158 /0.01930160|
|both|0.538|1.406|0.998|0.02168259 /0.02168259|
|near|0.956|1.041|0.999|0.05800667 /0.05800648|
|seam|0.546|1.418|0.997|0.04942131 /0.04942150|

Câmera+luz: aproximadamente46%menos tempo GPU que geometria, mas41%mais que a tabela original nesta rodada. A redução de memória não é gratuita. Próximo: diferença de aproximadamente4%contra geometria, pequena diante da variabilidade observada; não há ganho robusto estabelecido nesse recorte. Frame serial permanece praticamente igual, sem ganho de FPS demonstrado. Não comparar diretamente razões de rodadas antigas: o controle original também varia. Timestamps e p10–p90 por pose ficam no JSON/interface.

Todos os texels/halos foram preservados byte a byte. Maior RMSE introduzido16spp:0.00012979; aumento máximo do erro contra geometria:0.00000764. Em8bit após tone mapping: até4níveis de diferença por canal, no máximo2pixels por imagem diferiram mais de um nível, maior sequência horizontal/vertical desses pixels=1. Nenhuma faixa nova detectada por esse critério nas213poses. Montagem dos piores RMSE de câmera/ambos/próximo inspecionada; diferenças pontuais, sem linha visível.

Em1spp, mais sensível nas bordas: RMSE máximo0.00043208, máximo RGB0.04304123, cobertura RMSE0.00157289. Isso impede chamar a imagem filtrada de lossless. Erros anteriores de detalhes/ponte/reflexos permanecem. A meta literal de zero alteração de qualidade não foi demonstrada; memória reduzida e fidelidade praticamente equivalente são os resultados medidos.

597testes Rust release/all-targets,fmt e clippy `-D warnings` passaram; aviso futuro de `block0.1.6` permanece. `render`, `replay_inspection`, `selection` headless passaram.213comparações bounded/fullscreen exatas, repetição/retorno de luz/+360° exatos; luz alterada muda resposta.852PNGs conferidos. Harness do JavaScript real:219checks/904carregamentos e recortes; relatórios anteriores432e46checks. Sem teste de browser nativo ou aceite visual humano.

Evidências: `artifacts/appearance-probe/compression/{index.html,report.json,summary.json,results.csv,validation.json,ui-validation.json,comparison.png}` e logs correntes no mesmo diretório. Formato e dados não foram integrados ao renderer/editor/MCP da engine. Próximo problema técnico: reduzir a indireção sem perder a economia de memória, e critério adaptativo para a câmera próxima. EAF-71 permanece com aceite visual pendente.
