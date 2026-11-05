# Regiões multiescala em malha externa — EAF-88

## Protocolo e checklist anterior à implementação

Entrada externa: [Boulder 01](https://polyhaven.com/a/boulder_01), Rico Cilliers/Poly Haven, CC0. Powered by Poly Haven. Importar somente posições/normais/índices da geometria glTF; texturas/UV/shader originais ficam fora deste experimento. Fonte e candidatos usam o mesmo material de diagnóstico e iluminação variável. O compilador recebe IGM1, sem LODs autorados nem genealogia. Registrar hashes e transformação de normalização.

1. Offline: particionar a malha por adjacência e compilar reduções por região, fixando seus vértices de fronteira.
2. Alocação: mapas/adjacências/interseções somente no build. Frame usa arrays/ranges residentes e argumentos de desenho compactos.
3. Dados: posições/normais f32 e índices u32; binário LE versionado e checksum. glTF/JSON apenas importação e relatórios.
4. Trabalho repetido: níveis compartilham vértices; redução por colapso para vértice existente, sem reconstruir buffers por frame.
5. Escala: regiões e níveis limitados; medir ganho de triângulos separado do overhead de seleção/desenhos.
6. GPU: upload estático, seleção CPU de ranges; medir submissão regional incluindo todos os draws, sem omitir seu custo.
7. Sincronização: benchmark isolado, lotes alternados/duplicados, janelaGPU, CPU e parede separados.
8. Invalidação: compilação vinculada ao input; formato/layout/checksum e cobertura/ranges validados no load.
9. Correção: fronteiras fixas, link condition e orientação; medir desvio de superfície e normal em interseções de triângulos projetados. Sem domínio válido, manter fonte. Projeção usa pixels físicos; comparar RGBA32F e cobertura durante órbita e aproximação/afastamento.
10. Custos mantidos: normal/material convencional, fonte residente, limites conservadores e overhead regional. Não prometer que a malha tenha densidade suficiente perto nem que certificado geométrico sozinho garanta igualdade visual. Gates RMSE≤0,005, p99≤0,02, cobertura≤0,0005; falhas impedem promoção do caminho e ficam no relatório.

Hipótese matemática: em uma região que seja um gráfico sobre um plano local, fonte e simplificação são funções afins por triângulo. Sobre cada interseção de seus triângulos projetados, a diferença de altura é afim; o máximo absoluto ocorre nos vértices da interseção. O mesmo vale para limitar a norma da diferença do campo afim de normais. A comparação cobre ambos os domínios e mantém a borda. Predicados f64 e margens são numéricos, não aritmética intervalar/prova formal universal. O experimento precisa validar a hipótese do domínio antes de usar esse limite.


## Resultado — 27/10/2026

**Este incremento não demonstrou ganho útil de performance.** O compilador coplanar anterior não eliminou nenhum triângulo desta fonte. A redução regional conservadora manteve a imagem, mas retirou no máximo dois triângulos entre 66.122. Não promover este caminho ao runtime da engine.

O glTF baixado contém 66.122 triângulos e 67.042 vértices; solda exata posição+normal deixa 61.991 vértices. A página do asset anunciava aproximadamente124K: o número medido no payload é o usado aqui. Hashes SHA256, URLs, autoria, escala e rotação ficam em `import.json`, reproduzidos por `scripts/import_irregular_boulder.py`. O importador é deliberadamente específico dessa fixture fixada por hash, não um importador glTF geral.

Compilação: 450 regiões após unir regiões sem redução, até cinco níveis contando a fonte. Colapsos preservam os vértices de fronteira, link condition e orientação. Cada redução é comparada diretamente com a fonte pelas interseções completas, não só por amostras de vértices. O runtime conserva regiões potencialmente de silhueta pelo cone de normais geométricas e variação de direção da câmera na AABB. Nas outras regiões exige erro projetado≤0,15pixel e diferença do campo de normais≤0,01. Níveis diagnósticos relaxam este último requisito durante a construção, mas não no seletor aceito.

A projeção local sem sobreposição e a cobertura de ambos os domínios são condições do certificado. A diferença de normais é do campo interpolado antes da normalização; não constitui um limite universal de erro fotométrico para qualquer shader. O cone não resolve todas as oclusões internas. Aproximações f64/margens e limites conservadores não equivalem a prova formal nem autorização para remover detalhe em qualquer asset.

### Qualidade e controles

Apple M4 Pro/Metal;1728×1117 e3456×2234 pixels físicos;distâncias2,7/4/8/16/32/64. Nove poses por distância, sendo a última retorno à primeira, câmera e luz variáveis. Mais25poses por resolução numa trajetória de afastamento/órbita/retorno.

- **158/158 comparações conservadoras tiveram RGBA32F idêntico**, incluindo dois retornos completos. Menor seleção66.120T; redução máxima0,0030%. Essa igualdade não prova que as poucas regiões reduzidas estavam visíveis em todas as poses; portanto não é prova forte da qualidade de uma redução significativa.
- Controle que conserva o limite geométrico≤0,15pixel e a proteção de silhueta, mas permite erro grande de normais: **11/60 comparações falharam**. Os60 incluem12repetições de retorno;11falhas incluem3repetições. Exemplo1728×1117,d32,órbita90°:65.086T,erro projetado0,12069px,RMSE0,009088,p990,037006,erro máximo0,196381,cobertura divergentezero. Logo, limite geométrico subpixel sozinho não preserva a imagem iluminada deste método. Não isolamos toda contribuição fotométrica exclusivamente nas normais: o controle também altera a superfície dentro da tolerância.
- Controle forçado no nível mínimo:56.018T(−15,28%), ignorando os gates. Falha nas duas resoluções:RMSE≈0,0118,p99≈0,0578; não é um resultado aceito nem uma economia publicável preservando qualidade.
- Os gates medidos continuamRMSE≤0,005,p99≤0,02,cobertura≤0,0005. Erro máximo também fica exposto, pois pequenas manchas podem passar em métricas agregadas. Capturas são rasterização do shader diagnóstico, sem referência supersampled integrada, texturas originais, animação ou materiais gerais.

Perto, o objeto cobre516.227pixels na resolução menor e2.064.843na maior. Dividir TODOS os66.122triângulos por esses pixels dá0,128 e0,032T/pixel, incluindo faces ocultas: perto esta fonte não testa saturação de microtriângulos! Longe há densidade maior, mas a preservação da aparência e o particionamento continuam limitando a redução. Isso não refuta o platô anterior em subdivisão redundante, nem o generaliza a scans.

### Memória e custo

Fonte:2.1756MiB de buffers de geometria. Fonte+níveis compactados:2.8763MiB, **+32,21%**; arquivoIGL1:3.2655MiB. Build+validação+escrita≈1,352s, import/download excluídos. Removidos vértices não referenciados antes da serialização. Níveis diagnósticos usados nos controles estão incluídos nesses bytes; não confundir com pacote de produção otimizado.

Esses bytes não são RAM total do processo nem memória total da GPU: excluem targetsRGBA32F/depth, staging/readback, uniforms e caches do driver. CPU também mantém o arquivo decodificado e metadados; seu picoRSS não foi medido neste incremento. Memória unificada do Mac não elimina essas alocações distintas. Arquivo guarda face com materialu32(16B); índice da GPU usa12B/triângulo; posições+normais usam24B/vértice. Metadados por região44B+16B/nível em arquivo. Quantização/u16 locais/streaming não implementados; binário ainda não é o formato mínimo possível.

Benchmark:128frames/lote,32poses,8rodadas alternadas,duas duplicatas por caminho/rodada,16lotes por caminho/caso. Seleção CPU, alocações de ranges, uniforms, codificação e todos os draws incluídos. Ambos os caminhos compartilham o mesmo buffer combinado residente; o fonte desenha apenas sua faixa. Não há upload de geometria por frame. A janelaGPU vai do primeiro ao último timestamp; tempo de parede inclui espera/readback do lote, não FPS de gameplay. `proxy_frames` do tipo compartilhado permanece0 e não mede ativação neste harness; usar as seleções explícitas nas linhas de qualidade.

| Resolução / distância | GPU fonte→regional (ms) | CPU fonte→regional (ms) | Parede fonte→regional (ms) |
|---|---:|---:|---:|
| 1728 / 2.7 | 0.06819→0.07383 | 0.02727→0.03013 | 0.10907→0.11216 |
| 1728 / 4.0 | 0.07013→0.07009 | 0.02792→0.03064 | 0.10940→0.11243 |
| 1728 / 8.0 | 0.06110→0.06176 | 0.02711→0.03009 | 0.09893→0.10650 |
| 1728 / 16.0 | 0.05047→0.05046 | 0.02726→0.02935 | 0.08943→0.09147 |
| 1728 / 32.0 | 0.04943→0.04868 | 0.02838→0.03011 | 0.09003→0.09132 |
| 1728 / 64.0 | 0.04498→0.04023 | 0.02705→0.02913 | 0.08012→0.08145 |
| 3456 / 2.7 | 0.19576→0.19817 | 0.02822→0.03089 | 0.23533→0.23836 |
| 3456 / 4.0 | 0.18013→0.17637 | 0.02815→0.03079 | 0.21651→0.21870 |
| 3456 / 8.0 | 0.16986→0.17539 | 0.02823→0.03148 | 0.20854→0.21626 |
| 3456 / 16.0 | 0.16262→0.15330 | 0.02809→0.03034 | 0.20135→0.19864 |
| 3456 / 32.0 | 0.14793→0.14793 | 0.02761→0.03014 | 0.18649→0.18918 |
| 3456 / 64.0 | 0.13202→0.14245 | 0.02828→0.03015 | 0.17417→0.18216 |

Não há ganho robusto: geometria praticamente igual e CPU acrescenta mediana de2,58µs entre coortes. Os controles duplicados variam aproximadamente1,1–27,7% na GPU; pequenas diferenças de sinais opostos não demonstram aceleração ou ausência de regressão. Timings separados do controle destrutivo não foram usados para alegar ganho.

### Decisão e próxima hipótese

Manter o compilador coplanar anterior como resultado válido para redundância, e este experimento como diagnóstico de generalização insuficiente. O particionamento, as fronteiras fixas e os limites de normais/silhueta podem ser excessivamente conservadores; não concluímos impossibilidade matemática.

Próxima hipótese: compilar um campo de aparência/normais de várias escalas, independente da triangulação usada para forma e contorno. Comparar a mesma malha reduzida com e sem esse campo, mantendo iluminação dinâmica e contando leituras, memória e custo de filtragem. Não pré-bakear uma cor de iluminação fixa. Antes de tentar novo ganho100×, repetir os controles de silhueta/manchas e um teste de densidade que realmente ultrapasse um triângulo visível por pixel. Essa investigação permanece proposta, não implementação deste incremento.

### Checklist posterior e validação

1. Frequência: adjacência,colapsos,interseções,certificados em `compile.rs` só no build; `select.rs` executa escolha por frame.
2. Alocação: doisVecs transitórios de escolhas/ranges no frame, medidos. Não é caminho sem alocação; não vale otimizar isso enquanto a redução útil for quase zero.
3. Dados: IGL1/IGM1little-endian,versão,length,checksum,arraysf32/u32. glTF/JSON somente no import,relatórios e painel; render não lê JSON por frame.
4. Repetição: níveis compartilham vértices compactados, buffers persistentes; ranges contíguos fundidos. Fonte completa vira um draw.
5. Escala: teto512faces/grupo,até5níveis; interseção quadrática por grupo em build. Mantém custo aproximado linear em regiões no frame; não promete independência do conteúdo.
6. GPU/I/O: `render.rs` recebe seleção de ranges; caminho anterior sem seleção mantém sua API/desenhos. Readback apenas no harness de evidência.
7. Sincronização: benchmarks encerrados antes da regressãoGPU/MCP; nenhuma janela controlada, thread ou dependência nova.
8. Invalidação: hash de download/import e checksum/versionamento do binário; faixas,cobertura deL0,comprimento e campos finitos validados. Formato experimental para saída confiável do compilador, não parser público adversarial.
9. Correção:664testesRust release/all-targets passaram, incluindo quatro testes novos de certificados/relevo/normais,codec/corrupção,fronteiras mistas e pixels físicos. Clippy-Dwarnings,fmt e13gruposMCPheadless(render,replay_inspection,selection) passaram.384estadosDOM/DPRsimulados do painel, arquivos de imagens presentes. Inspeção de PNGs não estabelece aceite humano nem validação de browser nativo.
10. Custos mantidos: geometria fonte residente,níveis diagnósticos,shader convencional,CPU de seleção e constantes conservadoras. Nenhum contratoNPC/Scene/Command/MCP alterado e nenhum ganhoFPS alegado.

### Reprodução

Na worktree `codex/graphics-research`:

```sh
python3 scripts/import_irregular_boulder.py
cargo build --locked --release --example irregular_lod_probe
target/release/examples/irregular_lod_probe --build artifacts/appearance-probe/irregular-lod/boulder.igm artifacts/appearance-probe/irregular-lod/boulder.igl
target/release/examples/irregular_lod_probe --run artifacts/appearance-probe/irregular-lod/boulder.igl artifacts/appearance-probe/irregular-lod
python3 scripts/report_irregular_lod.py
```

O painel `artifacts/appearance-probe/irregular-lod/index.html` compara capturas pré-renderizadas e mostra os timings. Controles de distância,órbita/luz,resolução e gate; escala nominal1pixel renderizado/pixel físico viaDPR, com opção de visão geral explicitamente reduzida. Não é renderer interativo no browser nem prova da resolução atual da tela do usuário. Resultados e fontes gerados são ignorados peloGit, em localização estável; comandos acima os regeneram.
