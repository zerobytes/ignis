# Roadmap da pesquisa gráfica

Direção confirmada em 27/10/2026. Pesquisa isolada em `codex/graphics-research`, acompanhada em EAF-88; preserva continuidade funcional de NPCs. Entrada: [graphics-handoff.md](graphics-handoff.md).

## 1. Consolidar seleção e estabilidade — atual

Isolar a instabilidade de repetição no material rico sem relaxar igualdade de retornos ou trocar silenciosamente o material de referência. Corrigir a previsão de custo que ainda perde oportunidades em distância intermediária; separar ruído de caminhos equivalentes de ganho real. Preservar detalhes/contornos próximos, contabilizar memória de fallback e medir tempo total incluindo seleção. Evidências: [rich-relief-study.md](rich-relief-study.md).

Diagnóstico atual: tangentes compactas passam138/138comparações,128/128repetições e10/10retornos; tabela cai9→4,5MiB. Perto3456, mediana0,875541→0,855181ms contra geométrico anterior (−2,33%,5/8rodadas, dispersão~2,9%), ainda11,23% acima do original. Não há ganho robusto/global ou adoção. Projeção mais curta sozinha não resolve o custo. Controles com índice/guarda também ficam estáveis, porém muito mais caros; `discard` impede atribuir o resultado só ao acesso porface. Próximo isolamento: controles com índice/leitura observáveis em canal diagnóstico sem descarte, custos declarados, e repetição sem alternância de pipeline. Original ainda tem dois hashes posteriores à primeira captura nas coortes próximas; não tratar aquecimento como solução. Integrar à seleção regional somente após gates conjuntos.

## 2. Compor uma cena com vários objetos

Combinar hierarquia compilada, seleção regional e visibilidade, com objetos em várias distâncias, obstáculos e câmera em movimento. Medir qualidade espacial/temporal, memória residente, CPU/GPU/tempo total e controles duplicados. Repetir densidades mantendo superfície/material fixos; distinguir redundância de novo detalhe real. Streaming e integração na engine continuam marcos próprios, não capacidades já entregues.

## 3. Amostragem de geometria guiada pela atmosfera — adiada

Proposta do usuário: manter a geometria detalhada disponível, mas consultar sua contribuição em uma fração das amostras da imagem que diminui com a transmissão atmosférica. As demais recebem contribuição da atmosfera sem calcular a superfície detalhada. Não reduzir essa proposta a HLOD ou névoa aplicada depois de renderizar toda a geometria.

Referência visual: três fotografias de cadeias de montanhas discutidas em 27/10; primeiro plano rico, cadeias progressivamente esmaecidas, contornos contínuos e profundidade legível. Nuvens ficam fora do primeiro recorte. Fotos orientam aparência, não provam eficiência nem o método de amostragem.

Hipótese inicial em atmosfera uniforme, cor linear: escolher superfície com probabilidade T e atmosfera com 1−T pode reproduzir a mistura T·superfície+(1−T)·atmosfera em média. Média correta não garante imagem limpa por frame, conservação de detalhes finos ou eficiência. Investigar decisão anterior ao trabalho geométrico, obtenção de distância/visibilidade, oclusão correta entre cadeias, distribuição/reconstrução de amostras e custo adicional. Não descartar fragmentos no fim e chamar isso de economia equivalente de triângulos.

Build time: preparar hierarquias, limites e dados compactos efetivamente consumidos; runtime: câmera/atmosfera escolhem amostras sem reconstruir assets. Formatos binários versionados para payload; JSON de relatórios não é formato de render. Ainda não escolher arquitetura definitiva nem prometer ineditismo ou ganho percentual.

Configurações propostas: habilitação, densidade/distância de visibilidade, aparência atmosférica e tolerância/amostragem de qualidade separadas. Ao integrar, operações compartilhadas em UI e MCP (AI first, não AI only).

Gates: mesma geometria/material e mesma atmosfera na referência integral e no candidato; várias cadeias/distâncias, câmera/luz em movimento, ar limpo, entrada/saída de névoa e elevação da câmera. Medir consultas geométricas evitadas, reconstrução, tempo total e memória; observar contornos, ruído, cintilação, disoclusões e retorno. Não inferir 90% de ganho de 90% de amostras omitidas. Sombras/reflexos precisam de critérios próprios; continuidade visual não significa carregar/simular o mundo inteiro. Manter representação disponível até a substituta estar pronta ou a contribuição visual ficar desprezível.

Antecedentes conceituais: [transparência estocástica](https://research.nvidia.com/publication/2011-08_stochastic-transparency), [omissão probabilística de cálculos](https://www.pbr-book.org/3ed-2018/Monte_Carlo_Integration/Russian_Roulette_and_Splitting). Não estabelecem equivalência exata com a proposta nem validam sua integração/performance.
