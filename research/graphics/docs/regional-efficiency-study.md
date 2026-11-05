# EAF-75 — custo total e contribuição visível

Hipótese: manter a decisão visual e reduzir a preparação/submissão, antes de alterar a representação. Comparar a geometria completa também agrupada evita atribuir ao resumo um ganho que pertence à submissão.

Experimento autorizado em 26/10/2026, isolado no worktree gráfico. Mesma cena de EAF-74 e mesmas resoluções/câmeras. Nenhum contrato de runtime, Scene ou MCP muda.

## Revisão antes de implementar

1. Frequência: projeção por região/frame; fatorar transformação linear em centro e extensão e reutilizar por asset, pois as instâncias têm translação e escala uniforme positiva.
2. Alocações: planos e scratch pré-alocados, capacidade reutilizada. Capturas e estatísticas ficam fora do caminho medido.
3. Dados: metadados numéricos, identidade de objeto preservada ao agrupar instâncias; nenhum texto por região.
4. Repetição: guardar transformações comuns, evitar projeção de materiais que sempre retêm geometria, agrupar faixas de índices idênticas.
5. Limites: cena fixa de 96 instâncias; não extrapolar para mundo aberto. Agrupamento e testes de visibilidade também custam.
6. GPU/I/O: mesmo shader, cache e precisão. Medir upload, submissão e espera; não usar readback de visibilidade como se fosse gratuito.
7. Sincronização: cronômetros GPU e ciclo serial separado, ordem alternada e controles repetidos; sem threads ou dependências novas.
8. Validade: seleção física/histerese continua por instância, regiões próximas de limiares usam cálculo original. Não reciclar máscara de visibilidade entre câmeras como algoritmo correto.
9. Correção: comparar pixels, IDs, decisões e transições com a referência anterior; testar agrupamento e equivalência do bound analítico.
10. Custos mantidos: cache de uma camada/seis vistas, guarda de materiais brilhantes, sem sombras/GI/animação. "Invisível" neste experimento significa sem contribuição direta ao frame; não cobre reflexão/sombra.

## Limite ideal de oclusão

Renderizar IDs de região permite descobrir exatamente quem ganhou ao menos uma amostra de profundidade. Remover regiões sem amostras mede um teto ideal para esta câmera e este padrão de amostragem. A obtenção da máscara exige desenhar a geometria e fazer leitura de volta: medir esse custo separado e nunca chamar o resultado de culling de produção. Uma máscara antiga pode omitir desoclusões; comparação com câmera seguinte deve mostrar esse risco.

Referências de abordagem: [oclusão conservadora na Unity](https://docs.unity.com/en-us/engine/6000.0/manual/analysis/graphics-performance-profiling/in-urp/gpu-culling) e [agrupamento de comandos no Vulkan](https://docs.vulkan.org/samples/latest/samples/performance/multi_draw_indirect/README.html). A implementação inicial usa instancing direto portátil no backend já selecionado, sem supor suporte a multi-draw indireto nativo.

## Implementação e comparação

CLI `cargo run --locked --release --example appearance_probe -- --regional-efficiency`. Arquivos em `regional_efficiency.rs` e `regional_efficiency_bench.rs`, renderer compartilhado com EAF-74. O modo anterior continua disponível e seus artefatos são preservados.

A transformação linear de uma caixa tem centro `R*c+t` e extensão `abs(R)*h`. Preparar a parte comum por região/asset e depois aplicar translação/escala por instância substitui as oito transformações de cantos. Uma margem proporcional à precisão f32 expande a extensão; perto do near plane, limiares 0,75/1px e mudanças de mip, o cálculo original é usado. Materiais sempre geométricos não precisam calcular tamanho projetado. Comparações com potências de dois substituem logaritmos para o mip, mantendo fallback perto da fronteira. As decisões de histerese são comparadas com o seletor original em todo o percurso.

Faixas de índices iguais são ordenadas e desenhadas com instancing. O buffer de instâncias conserva IDs lógicos e acrescenta registros para as faixas agrupadas; a faixa dos proxies é explícita e não inclui esses registros. Não se duplicam triângulos de malha em memória. O upload adicional dos registros e o custo do agrupamento permanecem medidos. A geometria completa passa de um draw por objeto para quatro (terreno e três assets), para que o controle receba a mesma otimização.

Seis variantes: geometria original; regional anterior; apenas seleção rápida; geometria agrupada; seleção rápida + agrupamento; geometria com máscara ideal offline. Todas usam os mesmos shaders, luz e profundidade, sem alterar os caches. A variante ideal conserva regiões inteiras com ao menos uma amostra vencedora: não classifica separadamente oclusão, projeção fora da tela e região subpixel que não atingiu uma amostra. Seu teto é restrito à granularidade regional e à amostragem 1spp usada.

Ao desenhar os IDs regionais, a cor deve continuar igual à geometria completa. Remontar a geometria com a máscara deve conservar cor e IDs de objetos. A máscara anterior é intencionalmente reaplicada à próxima posição **amostrada** (a cada três passos, não ao frame seguinte de um jogo); mede um exemplo de desoclusão incorreta, sem alegar magnitude típica em 60Hz. Nenhum oracle entra no caminho utilizável.

## Medição e interpretação

52 poses completas, duas resoluções físicas, ida e volta. Cronômetros em 20 poses. Cada modo tem dois controles, ordem rotacionada, 20 ciclos de aquecimento por modo, 24 amostras individuais por controle. Timestamp GPU mede o passe; serial inclui seleção, upload, codificação, submissão e espera. Upload e codificação também são registrados separadamente. Cada modo recebe seu próprio conteúdo de instâncias antes da submissão — não compartilhar uma última atualização entre modos agrupados diferentes.

Vazão é medida em 12 blocos por controle, oito frames da mesma pose enfileirados por bloco e espera no final. Isso permite sobreposição CPU/GPU, mas não mede latência de entrada nem FPS de um jogo com câmera/simulação variáveis. A ordem percorre todas as posições para reduzir viés de aquecimento. Medianas das razões **pareadas por pose**, nunca razão entre medianas independentes. Faixas entre controles duplicados ficam visíveis; variação grande impede chamar diferenças pequenas de ganho estável.

A primeira rodada teve oscilação grande entre controles idênticos. O aquecimento e as repetições foram ampliados por esse motivo, sem remover amostras desfavoráveis. O resultado anterior de EAF-74 não deve ser misturado numericamente com esta rodada: todos os controles são medidos novamente na mesma execução.

### Espaço de trabalho

A repetição ampliada encontrou ENOSPC no disco interno. `df` mostrou 222MiB disponíveis após a falha; `target` tinha 888MiB, sem arquivos abertos segundo `lsof +D target`. O único build release deste worktree foi movido para `/Volumes/Zero/build-cache/ignis-graphics/release`, com link em `target/release`. Não foram removidos relatórios anteriores, arquivos do usuário ou builds de outras tarefas; o destino externo tinha cerca de11GiB disponíveis. A execução parcial foi substituída no caminho estável ao repetir. Não há coleção de builds por tentativa.

Após a mudança, `df` observou 9,0GiB livres no interno e10GiB no externo. A diferença interna excede o tamanho movido e não deve ser atribuída integralmente a esta tarefa; pode haver liberação concorrente do sistema/outras tarefas.

## Resultado da rodada ampliada

Todas as52poses: RGB e IDs exatamente iguais entre regional anterior e otimizado; geometria agrupada também exatamente igual à original. Isso preserva a qualidade existente, sem resolver seus erros anteriores contra a referência integrada. A máscara ideal preservou RGB/IDs em todas as20poses instrumentadas. Nenhum critério foi relaxado para obter redução de custo.

| Resolução física | Desenhos regionais antes → depois | CPU seleção otimizada, mediana | Razão CPU versus anterior | GPU / geometria agrupada | Serial / geometria agrupada | Vazão / geometria agrupada |
|---|---|---|---|---|---|---|
| 3456×2234 | 329–453 → 76–120 | 164.75µs | 77.3% | 79.9% | 106.6% | 87.0% |
| 1728×1117 | 478–819 → 92–201 | 195.64µs | 71.7% | 67.9% | 87.1% | 80.4% |

Razões pareadas; menor que100% custa menos. Contra o regional anterior, a vazão caiu cerca de7% nas duas resoluções, com redução de23–28% no custo de seleção. Contra geometria completa igualmente agrupada, a vazão ficou13–20% mais barata; GPU20–32% mais barata. O serial ficou6,6% mais caro na resolução maior e12,9% mais barato na menor. Isso não demonstra um salto drástico nem ganho de FPS de gameplay.

A oscilação caiu após aquecimento/balanceamento, mas permanece: controles duplicados do regional otimizado variaram nas razões GPU0,966–1,037 e0,941–1,105; serial0,852–1,345 e0,845–1,471; vazão0,883–1,025 e0,938–1,045. O ganho incremental de~7% deve ser tratado como indicativo, sem intervalo de confiança ou promessa em outras máquinas. A contagem menor de chamadas e a identidade dos pixels são resultados determinísticos; os tempos são observações neste host.

| Limite ideal offline | 3456×2234 | 1728×1117 |
|---|---|---|
| Triângulos restantes, mediana | 45.15049% | 32.94367% |
| Descoberta da máscara, mediana | 20.07858ms | 7.40431ms |
| Erro máximo usando máscara da pose amostrada anterior | 0.16890 | 0.26793 |

Portanto cerca de55–67% dos triângulos completos podem ser eliminados nesta cena/câmera/amostragem sem mudar nenhum pixel, mantendo a granularidade de regiões. O controle ideal reduziu o custo GPU em31–48% e a vazão em24–36% versus geometria agrupada, **excluindo** os20/7,4ms de descoberta. Somar a descoberta torna essa técnica offline inadequada como culling de execução. O resultado não é uma otimização entregue: quantifica onde procurar. Reusar a máscara de câmera antiga falha; permitir isso para obter velocidade violaria a prioridade visual.

## Direção sustentada pelos resultados

O próximo experimento indicado é descobrir visibilidade conservadora dentro da GPU e compactar o trabalho lá, para evitar leitura de volta e milhares de decisões/submissões CPU. Uma hierarquia de profundidade e bounds regionais pode rejeitar regiões seguramente ocultas; casos ambíguos devem continuar a renderizar. A descoberta precisa custar menos que a economia medida. Como a máscara ideal também elimina regiões que não acertam nenhuma amostra, um teste conservador de oclusão pode alcançar apenas parte desse teto.

Hierarquia por região/objeto, reprojeção temporal com recuperação de desoclusões no mesmo frame e compactação GPU são alternativas futuras, não implementação existente. Ordem proposta: medir primeiro o custo de uma prova conservadora de visibilidade, depois combinar apenas os resumos cuja fidelidade esteja demonstrada. Reflexos, sombras e animação exigirão suas próprias noções de contribuição; não remover seus produtores apenas por estarem fora da câmera.

## Validação e reprodução

```sh
cargo run --locked --release --example appearance_probe -- --regional-efficiency
python3 scripts/check_regional_efficiency.py
python3 scripts/report_regional_efficiency.py
node scripts/verify_regional_efficiency.mjs
cargo fmt --all -- --check
cargo clippy --locked --release --all-targets -- -D warnings
cargo test --locked --release --all-targets
python3 scripts/run_verifications.py --only render replay_inspection selection
```

613 testes Rust (dois novos para bounds e agrupamento), fmt/Clippy e13grupos MCP headless aprovados.52poses,20instrumentadas,176PNGs completos com CRC/decompressão/dimensões validados, zero diferença de cor/ID na otimização, falha intencional da máscara antiga confirmada. Harness do JS entregue cobre52estados, DPR/recorte/clique/reprodução/carregamento obsoleto. Inspeção local do PNG completo não substitui aceite humano, recorte físico percebido ou apresentação nativa do HTML. Navegador local não automatizado por bloqueio prévio de file://, respeitado sem contorno.

Relatório corrente: `artifacts/appearance-probe/regional-efficiency/index.html`, com dados brutos em `report.json`, agregação em `summary.json` e checks em `verification.json`. Os resumos de medição substituem a execução parcial que falhou por espaço. Sem push, mudança da engine funcional, demo, Scene/Command/RenderFrame/MCP ou branch principal.

## Revisão final de performance

1. Frequência: suporte analítico por asset/frame; aplicação por instância. Continuidade de decisões comparada ao caminho original.
2. Alocação: capacidade de planos/scratch reutilizada; agrupamento in-place nos draws. Capturas, máscara ideal e serialização offline alocam deliberadamente.
3. Dados: IDs numéricos conservados por instância; IDs de região cabem exatamente em f32 neste teste. Sem formatos públicos novos.
4. Repetição: removidos oito cantos e logaritmos no caso comum; material sempre geométrico retorna cedo. Margens e fallback mantêm decisões e filtro.
5. Escala:96objetos/479regiões únicas, seleção plana ainda linear; sort de faixas limitado, sem alegar escala de mundo aberto.
6. GPU/I/O: upload adicional do agrupamento medido; cache permanece compartilhado. Readback de visibilidade claramente fora do caminho utilizável e contabilizado no diagnóstico.
7. Sincronização: serial e blocos de8frames separados; duplicação/rotação de controles, aquecimento ampliado após oscilação. Sem thread/dependência nova.
8. Invalidação: câmera/resolução/estado avaliados a cada frame. Máscara antiga somente controle de falha; nenhum reaproveitamento temporal inseguro.
9. Correção: testes de álgebra/agrupamento, igualdade de pixels/IDs, decisões, retorno da câmera, artefatos íntegros e regressões existentes. Sem janela funcional manipulada.
10. Custos mantidos: CPU ainda~0,16–0,20ms, dezenas/centenas de draws, cache~30MiB e aproximações anteriores. Descoberta ideal inviável no caminho atual; não convertida em promessa de desempenho.
