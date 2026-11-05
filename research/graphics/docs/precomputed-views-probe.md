# Vistas e respostas pré-calculadas

Experimento autorizado em26/10/2026, worktree `codex/graphics-research`. Etapas independentes: bake de um asset imutável e avaliação de vistas/luzes novas com os dados persistidos. Não altera runtime, Scene, RenderFrame ou MCP.

## Decisão antes de implementar

Comparar tabela angular de resposta RGB linear (material misto difuso/brilhante) com geometria completa. Duas densidades angulares,12×12 e24×24 vistas/luzes, órbita azimutal completa com elevação de câmera55° e luz35° fixas. Texturas HDR RGBA16F,128² por vista/resposta, pré-filtradas com16 amostras espaciais. Guardar altura visível e cobertura por vista para reprojeção; controle sem reprojeção mede a mistura sobre plano. Fonte: microgeometria colorida, relevo e ponte elevada para expor sobreposição/desoclusão.

Alternativas: coeficientes compactos poderiam poupar memória, mas adicionariam erro de ajuste e escolha de base. Este recorte usa uma tabela explícita para isolar interpolação angular e custo/memória. Uma esfera de vistas, várias elevações de luz, animações e múltiplas camadas aumentariam dimensões e custo; não são implicitamente suportadas. A primeira camada pode falhar em partes recém-expostas. Medir o erro antes de propor integração/expansão.

O bake contém respostas para parâmetros fixos do material e intensidade unitária; runtime interpola direções, sem loop sobre triângulos ou recompilação da resposta. Sombreamento direto compartilhado, sem sombras projetadas/GI. Câmera usa perspectiva no teste e vistas ortográficas na tabela; reprojectar por altura é aproximação de um campo de altura, não solução geral de visibilidade.

## Checklist prévio de performance

1. Preparação por asset no bake; seleção de vizinhos e uniforms por frame.
2. Atlas e buffers residentes; nenhum download/recálculo do atlas na órbita. Readback só na instrumentação offline.
3. RGBA16F filtrável no atlas, saída/erros f32 lineares. Contabilizar quantização e memória; sem nova dependência.
4. Quatro respostas vizinhas (duas vistas ×duas luzes); coordenadas periódicas, salto360° testado. Matrizes de vista preparadas e usadas no mesmo contrato.
5. Limites explícitos12/24 direções e128²; memória quadrática nas duas dimensões angulares. Nenhuma promessa para bilhões de triângulos.
6. Bake supersampled offline; runtime dois triângulos cobrindo uma região conservadora projetada e poucas leituras por pixel. Geometria completa usa o mesmo material/luz/target.
7. Timestamps intercalados e parede CPU/GPU bloqueante separada; não somar medianas como FPS. Sem threads/locks novos.
8. Artefato versionado, assinatura de geometria/shader/configuração e checksum; arquivos truncados/obsoletos rejeitados antes de upload. Rebuild explícito em caminhos estáveis.
9. Validação em ângulos fora da grade, órbita completa, luz/câmera isoladas e conjuntas, controle sem reprojeção; erro de cor/cobertura/temporal. Interface reproduz dados, CLI executa experimento; não é controle novo do editor.
10. Custos mantidos: memória, banda, preparo offline, desoclusão e reflexos estreitos. Medir e reportar falhas em vez de esconder com fallback não contabilizado.


## Implementação e contrato do artefato

`examples/appearance_probe/views.rs` e `views.wgsl` implementam o bake e a consulta. O mesmo shader de resposta direta (Blinn–Phong, dois perfis fixos por cor) ilumina a geometria e produz o atlas.12 ou24 vistas azimutais ×12 ou24 direções de luz; elevações fixas declaradas. Textura2D-array com uma camada por vista, respostas de luz em tiles horizontais e um tile adicional de altura/cobertura. RGB linear HDR e cobertura premultiplicada são pré-filtrados juntos; altura é armazenada ponderada pela cobertura. Cada texel usa8bytes RGBA16F. A interpolação usa duas vistas ×duas luzes; o modo com altura faz três iterações de interseção aproximada, com mais leituras da mesma tabela.

O disco contém `atlas12.bin/json` e `atlas24.bin/json`. Metadados incluem versão, dimensões, assinatura de vértices/shaders/uniformes de bake, tamanho exato e checksum FNV64 (detecção de alterações, não autenticação criptográfica). O teste exige artefato presente e compatível; não faz bake silencioso. Alterar câmera/luz dentro do domínio não escreve nem recalcula a tabela. Os parâmetros do material, geometria e domínio de elevação são fixos: modificá-los exige rebuild. A geometria só fica residente no laboratório para produzir a referência.

O proxy foi limitado a uma caixa projetada conservadora, sem cortar pixels nas639 comparações do percurso com o desenho de tela inteira. Raios derivados da posição absoluta do fragmento preservam precisão quando a triangulação da tela muda. Normalização angular anterior a seno/cosseno e à seleção torna ângulos equivalentes exatamente iguais; a primeira versão revelou sensibilidade numérica na volta360°, corrigida e coberta por teste.

## Reprodução e protocolo

```
cargo run --locked --release --example appearance_probe -- --bake-views
cargo run --locked --release --example appearance_probe -- --views
python3 scripts/report_precomputed_views.py
node scripts/verify_views_report.mjs
```

São processos separados. Atlas persistidos, referência e resultados em `artifacts/appearance-probe/views/`, com caminhos estáveis. A interface tem percursos câmera/luz/ambos/próximo/passagem360°, orçamento12/24, reprodução, avanço/retorno e ampliação comum. A interface reproduz capturas reais da execução nativa; não é um renderizador 3D em JavaScript.

213 poses:49 por percurso principal e17 na fronteira angular. As quatro órbitas principais usam ângulos fora de ambas as grades; a passagem360° inclui nós da grade. Os quatro controles adicionais0/90/180/270° mostram que até nós exatos têm erro espacial/de reconstrução. Referência256spp; candidatos e controle completo16spp. Erros medidos em RGBA linear antes de tone mapping. Dois controles256→1.024spp mediram RMSE0,00260 e0,00294, portanto a referência também é uma aproximação com resíduo explícito.

Performance:256²/1spp em todos os métodos,63 rodadas intercaladas após aquecimento, com rotação de ordem. Mediana/p10/p90 por pose. CPU de construção/envio de parâmetros é separada. Nove rodadas intercaladas de frame serial incluem atualização, codificação, submissão e espera, sem leitura de imagem. Esse ciclo bloqueante mede o laboratório; não é pipeline de gameplay nem FPS. A geometria completa não constrói matrizes/limites de atlas inutilizados.

## Resultado — Apple M4 Pro / Metal

Asset:65.572 triângulos (microgeometria, relevo e ponte),7.868.640bytes de vértices, aproximadamente7,50MiB. Proxy:2triângulos; nenhum loop de geometria/32lobos nem compute de resposta durante o desenho. Os custos do atlas são:

| Tabela | Payload disco/GPU | Vezes a geometria | Bake medido | Leitura/validação/upload |
| --- | ---: | ---: | ---: | ---: |
|12×12|19,5MiB|2,60×|0,267s|39,82ms|
|24×24|75MiB|9,99×|0,987s|113,76ms|

Bake inclui recursos por tabela, render, filtro, readbacks e checksum; exclui geração inicial da malha, inicialização do dispositivo/pipelines comuns e gravação final dos arquivos. Memória é payload, sem overhead do driver. O laboratório retém as duas tabelas, fonte e targets/readbacks de captura; isso não é memória exclusiva de um proxy no runtime. Mais direções custam quadraticamente neste recorte. Sem compressão ou seleção hierárquica integrada.

Tabela24×24 com altura, **mediana das razões pareadas por pose**, referência de performance é geometria completa:

| Percurso | Razão GPU | Economia GPU | Razão frame serial | RMSE RGB mediano / máximo |
| --- | ---: | ---: | ---: | ---: |
|Só câmera|0,384|61,6%|0,993|0,02067 /0,03920|
|Só luz|0,404|59,6%|0,996|0,01930 /0,03719|
|Câmera+luz|0,483|51,7%|0,993|0,02168 /0,04743|
|Próximo|0,693|30,7%|0,997|0,05801 /0,12467|
|Fronteira360°|0,488|51,2%|1,001|0,04942 /0,05122|

No percurso conjunto, medianas de tempos GPU0,11025ms completo e0,04671ms tabela; a razão dessas duas medianas não é a razão pareada usada acima. Frame serial aproximadamente1,35ms nos dois caminhos: não há melhoria relevante de frame completo estabelecida. As distribuições variam com carga/frequência/scheduling; diferenças pequenas não são conclusivas. Os tempos não incluem bake/upload inicial, reportados separadamente.

Qualidade:24×24 reduz RMSE mediano do percurso conjunto de0,02776 (12×12) para0,02168, ao custo de3,85× a memória. O controle24×24 sem altura tem RMSE0,02564 e erro de cobertura0,04773; com altura, cobertura0,02331. Mais amostras angulares não eliminam a perda de detalhe na câmera próxima: atlas128² filtrado e reconstrução de camada única ainda limitam o resultado. Os próprios nós de bake têm RMSE aproximadamente0,045–0,050 em controles brilhantes: erro não pode ser atribuído somente à interpolação angular.

A montagem evidencia ponte/relevo aproximados, suavização de microdetalhe e discrepâncias de brilho. Não é resultado equivalente pixel a pixel à geometria completa, nem prova de ausência universal de faixas/popping. A passagem periódica é determinística, porém a qualidade de desoclusão continua aberta. Próxima decisão: representação de camadas/visibilidade e seleção por erro/distância, além de compressão da resposta; não aumentar indiscriminadamente a tabela.

## Checklist após implementação

1. **Corrigido:** resposta cara migrou para bake separado; consulta por pixel, seleção de quatro vizinhos por frame.
2. **Adequado:** atlas/uniforms/grupos e mesh residentes; buffers de parâmetros reutilizados. Alocações/readbacks/JSON por captura pertencem à instrumentação.
3. **Adequado:** atlas RGBA16F filtrável e métricas f32; clamps de meio texel impedem vazamento entre tiles de luz. Limites12/24 e128² explícitos.
4. **Corrigido:** entradas periódicas normalizadas; método completo dispensa matrizes do atlas. Sem reavaliação de lobos/triângulos no proxy.
5. **Mantido com motivo:** memória Nvisão×Nluz×resolução²; dataset pequeno e domínio restrito. Não prova escalabilidade de biblioteca de assets ou bilhões de triângulos.
6. **Corrigido:** retângulo conservador limita áreas vazias;639 comparações exatas com fullscreen. Nenhum readback/download na lógica de consulta; bake/captura têm custos separados.
7. **Adequado:** timestamps resolvidos após submissão completa, ordem alternada; waits são instrumentação explícita, sem locks/threads novos.
8. **Adequado:** versão/signatura/checksum e rejeição antes do upload; runtime não modifica o atlas. Mudanças de geometria/material/domínio exigem novo artefato.
9. **Validado:** testes de parâmetros periódicos e arquivos, GPU de repetição/retorno/luz/wrap/controles/convergência; regressões headless e interface JS. Nenhum contrato de cena/MCP/editor alterado.
10. **Mantido com motivo:** altura única, resposta sem sombras/GI, banda/memória, aliases/reflexos e aproximação espacial. Não houve ganho relevante no frame serial nem aceite visual automático.


## Validação da entrega

596 testes Rust release/all-targets passaram (13 no exemplo, incluindo seleção periódica/erro de artefato); fmt e clippy `-D warnings` passaram. O aviso de incompatibilidade futura da dependência `block0.1.6` permanece. Cenários MCP `render`, `replay_inspection` e `selection` passaram sem janela.

639 comparações de qualidade em213 poses;639 comparações nativas bounded/fullscreen exatamente iguais; repetição, retorno após mudança de luz e equivalência+360° exatamente iguais.1.704 PNGs de256², metadados/arquivos de atlas conferidos, controles de referência e logs registrados. Harness do JavaScript real:432checks,1.742carregamentos e1.760recortes; relatório anterior46checks. Montagem PNG inspecionada. Não houve teste de interação nativa do navegador nem aceite visual do usuário.

Evidências correntes em `artifacts/appearance-probe/views/validation.json`, `ui-validation.json`, `report.json`, `summary.json`, `results.csv`, `comparison.png` e logs no mesmo diretório. EAF-69 acompanha o aceite e a decisão sobre a próxima representação. Outputs gerados/ignorados; fontes e protocolo versionados. Sem push ou mudança da demo do usuário.

## Continuação: compactação

O experimento de compactação da tabela24×24, com comparações diretas de dados, imagens e GPU, está em [compressed-views-probe.md](compressed-views-probe.md). O atlas original permanece como controle científico.
