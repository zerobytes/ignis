# Símbolos de superfície na pedra — EAF-88

## Protocolo antes da implementação

Pedido de27/10/2026: testar uma representação diretamente renderizável numa pedra com relevo real, câmera girando e luz mudando; comparar qualidade, memória e performance com triângulos completos. Reusar a pedra procedural já exportada do Blender (81.920 triângulos, deslocamentos macro/fino/micro nas posições), sem subdivisão redundante ou alteração da fonte. Fonte original preservada. Experimento isolado, sem contrato de runtime/MCP/editor ou dependência nova.

Hipótese: agrupar triângulos próximos em elementos orientados (surfels) com centro, raio de cobertura, normal, espessura e cor média ponderados por área. Cada símbolo48bytes responde ao material/luz corrente; nenhum atlas de cores por direção de luz. Grupos formados por célula espacial e octante da normal, sem genealogia conhecida da malha. Grids16/32/64 dão três compromissos. Material da pedra GGX/Schlick roughness0,88/metallic0, mesma função em todos os caminhos. Avaliar projeção de discos e versão com acumulação ponderada e rejeição por profundidade; os dois triângulos usados para cobrir um disco são suporte raster, não reconstrução dos triângulos originais.

Limites declarados: disco não é certificado de erro ou cobertura equivalente à superfície; média de normal/cor perde distribuição e sua correlação. A hipótese pode falhar especialmente em contornos e concavidades. Nenhum fallback oculto ou seleção por resultado do teste. Não relaxar gates depois de olhar resultados. Comparar amostragem1spp igual e registrar RMSE foreground≤0,005, p99RGB≤0,02, diferença de cobertura≤0,0005; reportar falhas. Controle integrado por supersampling em poses selecionadas separa aliasing da representação.

Câmera em órbita e luz móvel: movimentos separados e combinados, elevação variável no combinado, perto e longe; fechamento360° e repetição/retorno explícitos. Resoluções físicas3456×2234 e1728×1117; sem usar pixelsCSS na avaliação. Capturas são offline, não aceitação de vídeo/browser humano.

Timing separado das capturas/build/regressões. GPU timestamp e totalCPU/GPU sobrepostas incluem todas as passagens de cada caminho. Duas duplicatas, rodadas intercaladas, frames em lote; sem usar FPS de jogo ou somar GPU/CPU sobrepostas. Medir payload residente de cada representação e intermediários separados, preparo/upload fora do frame, sem atribuir todoRSS aos símbolos. Três níveis residentes apenas como controles do laboratório.

Checklist prévio:1) agregaçãooffline;2) buffersGPU persistentes, sem agrupamento/alocação de símbolos porframe;3) posição/normal/cor f32, índicesu32 e limites da fonte validados;4) shader de material compartilhado, dados preparados uma vez;5) grupos/poses finitos, registrar contagens e memória;6) múltiplos passes/readback somente instrumentação explícita, custo do render completo incluído;7) nenhuma thread/lock novo, coletaGPU serial;8) hashes da fonte e shader, rebuild explícito;9) testes de área/atributos/finitude/determinismo, capturas de giro/luz/retorno e regressões headless;10) manter custos de overdraw/cobertura/filtragem e falhas visuais explícitos. Usar build e diretório de artefatos correntes.

Após o smoke e antes da coleta completa: a forma de disco já apresenta perda visível perto. Incluir também distância96, além de4/24, para separar o caso ainda resolvível do regime com muitos triângulos por pixel. Essa extensão não altera o material, os símbolos ou os gates. Resultados negativos fazem parte do experimento.

## Implementação medida

`rock_symbol_build.rs` agrega cada triângulo uma vez, por centroide em célula espacial e octante da normal média. Centro, normal e cor são ponderados por área; raio cobre a projeção dos vértices do grupo sobre o plano médio e espessura registra o afastamento normal máximo. Vértices fonte não são reconstruídos no frame. Símbolo de três vec4/f32 (48 bytes), armazenado em buffer GPU e arquivo corrente por grade.

`rock_symbol.wgsl` contém a função GGX/Schlick comum à malha e aos símbolos. O caminho próximo rasteriza dois triângulos por disco e descarta fora do círculo e faces voltadas para longe. O ponderado faz depth dos discos, acumula a cor iluminada com peso radial e rejeita contribuições atrás do depth mais que `max(2×espessura,0,02×raio)`, depois normaliza a soma. Essa tolerância é aproximação explícita, não certificado de visibilidade. A normalização usa um triângulo fullscreen; seu custo fixo pode ser reduzido em outro incremento e não deve ser tratado como limite intrínseco de símbolos.

A referência calcula normal interpolada e direção de observação por fragmento; a representação usa normal média por grupo. Mesmo material/roughness não significa resposta luminosa equivalente após resumir as orientações. Não há atlas de cores por ângulos, sombras projetadas, GI, textura ou animação. A pedra tem deslocamentos geométricos reais do gerador existente, mas é procedural, não um scan importado novo.

## Memória e preparo

Apple M4 Pro / Metal; fonte com 40.962 vértices de 40 bytes e 81.920 triângulos indexados u32: 2.621.520 bytes (2,500 MiB).

| Grade | Símbolos | Payload | Redução versus malha | Preparo CPU |
|---|---:|---:|---:|---:|
| 16 | 1.985 | 95.280 bytes / 0,091 MiB | 96,37% | 3,65 ms |
| 32 | 6.428 | 308.544 bytes / 0,294 MiB | 88,23% | 5,15 ms |
| 64 | 19.143 | 918.864 bytes / 0,876 MiB | 64,95% | 7,18 ms |

As três escalas somam 1.322.688 bytes, residentes simultaneamente como controles do laboratório. Inicialização do renderer e upload de todas as representações: 22,14 ms nesta execução; inclui pipelines de infraestrutura reaproveitados, não somente upload ou custo de um asset. CPU mantém geometria/símbolos durante o harness. Não é medida de RSS.

Recursos por viewport: cor RGBA32F (16 bytes/pixel), depth32 (4 bytes/pixel), acumulação RGBA16F adicional do ponderado (8 bytes/pixel) e readback de diagnóstico (16 bytes/pixel). Em 3456×2234, a acumulação custa **58,90 MiB**; em 1728×1117, 14,73 MiB. Esse target é reutilizável entre objetos, não um custo por pedra. Para este objeto isolado, a economia de payload NÃO se traduz em economia total do caminho ponderado. O harness aloca todos os targets para permitir comparação; o caminho próximo não necessita do target adicional.

## Qualidade

162 poses = duas resoluções × três distâncias (4/24/96) × três movimentos (câmera/luz/ambos) × nove frames. No combinado a elevação também muda. Seis candidatos por pose: próximo e ponderado em cada grade. Referência 1spp e candidatos 1spp, mesmas posições de amostra. **0/972 comparações passaram simultaneamente nos três gates anteriores.** Isso é um resultado negativo da representação, não falha da validação do harness.

Na pose inicial, a pedra cobre 651.792/17.391/1.091 pixels nativos nas distâncias4/24/96: 0,126/4,710/75,087 triângulos fonte por pixel coberto. Na resolução menor: 162.953/4.359/269 pixels, ou 0,503/18,793/304,535 triângulos por pixel. A razão não conta vencedores por amostra. Assim, o ensaio inclui tanto detalhes bem resolvidos quanto forte concentração subpixel; não se limita ao caso próximo inadequado para agregação.

Medianas das 27 poses por distância, resolução nativa:

| Candidato | RMSE perto4 | RMSE longe24 | RMSE muito longe96 | Cobertura divergente mediana perto4 |
|---|---:|---:|---:|---:|
| Grade16 próximo | 0,05620 | 0,05703 | 0,05538 | 5,303% |
| Grade16 ponderado | 0,04475 | 0,04424 | 0,04407 | 5,303% |
| Grade32 próximo | 0,04855 | 0,04827 | 0,04863 | 1,840% |
| Grade32 ponderado | 0,03762 | 0,03678 | 0,03661 | 1,840% |
| Grade64 próximo | 0,03756 | 0,03659 | 0,03590 | 0,775% |
| Grade64 ponderado | 0,02638 | 0,02595 | 0,02494 | 0,775% |

Limites mantidos: RMSE≤0,005, p99RGB≤0,02 e cobertura≤0,05%. Na variante mais fiel (grade64 ponderado), pior RMSE0,030073/p99 0,128513/cobertura1,992% entre todas as poses. A ponderação melhora a cor, mas não a união de cobertura, que continua a dos discos. Recortes nativos512² confirmam perda de microrelevo e descontinuidades de aparência, mesmo sem depender das médias numéricas.

### Controle integrado e interpretação de aliasing

Seis controles adicionais (duas resoluções × três distâncias) na pose inicial da órbita de câmera, referências16/64spp. Esses controles não cobrem a órbita toda. Nativo, RMSE contra referência64:

| Distância | Malha1spp | Grade64 ponderado1spp | Convergência16→64 |
|---|---:|---:|---:|
| 4 | 0,00317 | 0,02699 | 0,00054 |
| 24 | 0,00967 | 0,02333 | 0,00160 |
| 96 | 0,02880 | 0,02021 | 0,00349 |

Muito longe, a média pode reduzir aliasing de cor e aproximar a imagem integrada melhor que a geometria1spp. Portanto, falhar contra a referência1spp não significa que toda diferença seja perda perceptual. Porém, esse resultado não recupera os detalhes próximos nem prova cobertura correta: a96, alphaRMSE da malha é0,0927 e do candidato0,0930; perto,0,0201 versus0,0836. A convergência também não é zero, logo64spp não é integral analítica. Não apresentar menor RMSE de cor isolado como aprovação visual geral.

Repetição, wrap angular+360°, mudança da luz e retorno exato testados para os sete métodos numa pose na resolução menor. Nas sequências salvas,34 de126 comparações de fechamentoPNG diferem, todas no movimento combinado: `sin(2π)` emf32 altera ligeiramente a elevação; isso ocorre também na referência. As órbitas somente câmera/luz fecham exatamente. Relatório lista cada divergência e não a chama de instabilidade do símbolo nem de retorno exato universal. Resíduo temporal registrado é diferença de erros em coordenadas de tela, sem reprojeção. Não há validação de vídeo contínuo/browser nativo/aceite humano.

## Performance medida

18 coortes (duas resoluções × três distâncias × três poses combinadas0/3/6), mais seis coortes repetindo a pose0. Cada coorte tem sete métodos × dois controles, sete rodadas alternadas/rotacionadas e32frames/bloco. Referência/mais próximo têm um passe; ponderado tem três, todos incluídos no total. Timestamp GPU em blocos separados de32frames; média dos intervalos de cada frame pode ser maior que a vazão amortizada por sobreposição no pipeline. Não somar intervalosGPU eCPU nem extrapolar FPS de jogo.

Pose combinada0, nativo3456×2234: câmera23,75°/25°, luz67,25°/55°. Valores são médias de quatro medianas (dois controles × duas rodadas), em ms/frame de CPU/GPU sobrepostas:

| Distância | Malha | G16 próximo | G16 ponderado | G32 próximo | G32 ponderado | G64 próximo | G64 ponderado |
|---|---:|---:|---:|---:|---:|---:|---:|
| 4 | 0,2548 | 0,3049 | 0,6061 | 0,3369 | 0,5750 | 0,3650 | 0,5895 |
| 24 | 0,2308 | 0,1899 | 0,3144 | 0,1984 | 0,3427 | 0,2669 | 0,4632 |
| 96 | 0,2089 | 0,1905 | 0,3009 | 0,1911 | 0,3664 | 0,2393 | 0,3979 |

O grade16 próximo pode economizar tempo longe, com grande perda de qualidade. O grade64 ponderado foi mais lento que a malha em todos os controles pareados:1,613–3,993× na rodada principal,1,635–4,049× na repetição. Medianas dessas razões:2,233×/2,173×. Isso demonstra regressão deste protótipo, não custo inevitável de toda representação por símbolos.

Duplicatas de tempo total variaram bastante: razão0,699–1,510 na coleta principal e0,720–1,173 na repetição. Há pouca confiança em diferenças pequenas. Tempos brutos/intervalos e resultado na resolução menor estão no relatório. Não afirmar precisão percentual estreita nem benefício sustentado dos candidatos com ganhos pequenos. A falha visual já impede promover qualquer candidato atual, mesmo onde o tempo caiu.

## Conclusão do recorte

O agrupamento sem genealogia reduziu dados e permitiu iluminação dinâmica direta. **Um disco com uma orientação e uma cor média não preservou a pedra.** Os candidatos não cumpriram o requisito de qualidade; a variante mais fiel ainda acrescentou custo relevante de render/intermediário.

Indicação para continuação: preservar variação local de profundidade e orientação/visibilidade no símbolo, em vez de ampliar discos e usar uma só média. Precisar de mais dados por símbolo é um tradeoff a medir, não uma derrota automática da ideia. Separar geometria que ainda ocupa pixels resolvíveis da agregação realmente subpixel; não forçar resumo próximo. Antes de otimizar o filtro ou escalar para AAA, uma nova representação precisa passar no teste de aparência. Há também custo evitável do resolvefullscreen e de targets não reutilizados de uma engine real, mas otimizar isso não recuperaria os detalhes perdidos.

## Reprodução e validação

Worktree `codex/graphics-research`, build corrente externo `/Volumes/Zero/build-cache/ignis-graphics/release`. Reusa a exportação existente em `artifacts/appearance-probe/assets/rock`; não executa Blender nem sobrescreve o arquivo de autoria.

```sh
python3 scripts/run_rock_symbol.py
cargo test --locked --release --all-targets
cargo clippy --locked --release --all-targets -- -D warnings
cargo fmt --all --check
python3 scripts/run_verifications.py --only render replay_inspection selection
```

`--rock-symbol-smoke`: uma pose na resolução menor. `--rock-symbol-bench`: retoma só os timings, exigindo os hashes originais de geometria/shader e payloads de símbolos bit-idênticos. Na coleta atual, capturas/controlereferência foram concluídos primeiro; o benchmark foi retomado após corrigir um descritor inválido de timestamp na passagem intermediária (nenhum índicebegin/end). O shader, payload e imagens permaneceram iguais. O caminho final completo contém a correção.

639 testes Rust (três novos), Clippy com warnings negados, fmt/diff-check e13 grupos MCP headless render/replay_inspection/selection passaram. O aviso futuro pré-existente de `block0.1.6` permanece. Verificador conferiu162poses/972comparações,1.140PNGs (CRC/dimensão/descompressão), payloads finitos/layout, hashes, controles integrados e24coortes de timing. Os972gates visuais reprovados são registrados, sem falsificar sucesso. UI:972combinações emDOM simulado, DPR1/1,5/2/3, recortes512², zoom1/2/4, reprodução e descarte de loads antigos; não é prova de browser nativo.

Evidências correntes em `artifacts/appearance-probe/rock-symbol`: `index.html`, `build.json`, `quality.json`, `integrated.json`, `timings.json`, `repeat.json`, `checks.json`, `summary.json`, `verification.json`, `ui-verification.json`, `input-manifest.json`, `grid*.symbols` ePNG. Logs no diretório pai. Outputs ignorados; fontes, scripts e documento versionados. Sem publicação/push, novo binário histórico, dependência, contratoMCP/Scene/RenderFrame ou alteração da frente funcional.
