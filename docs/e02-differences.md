# E02 — revisão por diferenças e inspeção orbital

Revisão da cena115cd80, preservando geometria, captura de referência e inventário
baseline763ba8d. Dados específicos e ações: [18 diferenças](references/image-to-3d/e02-differences.json).
Artefatos correntes: `artifacts/house-e02-rebuild/differences.html` e `orbit.html`.

## Câmera existente e visualizador

A Ignis possui uma câmera em perspectiva por documento, com posição/alvo/FOV,
comando MCP `set_camera`, captura offscreen e persistência. `src/window.rs`
processa cursor e clique esquerdo para seleção; não oferece MouseWheel ou
arraste para órbita. README também descreve câmera fixa. Não há um catálogo de
atores-câmera independentes inferido por isso. Nenhuma câmera nova foi adicionada
ao kernel nesta unidade.

O visualizador de inspeção WebGL carrega diretamente os16assets/30instâncias
salvos. Arraste orbita, roda/pinça aproxima, botão direito ou Shift+arraste
desloca; há ângulo de referência, frente, verso, superior, arestas e giro
contínuo. É uma visualização de geometria com iluminação própria: materiais,
sombras e aparência não reproduzem o renderer nativo. A câmera equivalente por
posição/alvo/FOV já tem operação MCP; o visualizador não edita a cena.

Publicação: `python3 scripts/e02_orbit_viewer.py` e
`python3 scripts/e02_difference_report.py`. Template reutilizável em
`scripts/viewers/scene_orbit.html`, sem dependência de CDN, servidor ou biblioteca.
Referência e links de comparação ficam junto do HTML no root corrente.

## Achado prioritário: coerência espacial

As novas capturas nativas laterais, superior e verso expõem uma montagem de
fachadas com profundidade incompleta. Galeria/coberturas se afastam e faltam
paredes/fechamentos. A receita posiciona pontos projetados em planos separados,
mas não impõe encontros compartilhados em3D. Isso permite bom alinhamento na
câmera da foto sem garantir uma construção coerente ao girar.

A09 já exigia fechamento coerente. A avaliação anterior da vista auxiliar
próxima foi insuficiente e precisa ser corrigida; não tratá-la como aprovação
volumétrica. A imagem não determina o verso real, mas a hipótese de verso
precisa respeitar continuidade, espessuras e apoios. O orçamento detalhado não
justifica pular essa etapa de massa/planta antes dos ornamentos.

As capturas `review-left.png`, `review-top.png` e `review-back.png` vieram de
processo MCP próprio/headless. Só na cópia efêmera de inspeção o painel
`studio-background` foi removido para não bloquear a câmera do verso. Nenhum
save foi feito; fonte, cena exportada e captura comparativa continuam idênticas.

## Erros/insuficiências de análise versus execução

| Diferença | Referência versus modelo | Diagnóstico |
|---|---|---|
| Balaustrada esquerda | Cinco coroas principais completas legíveis em C09; sete módulos na receita | Análise dizia apenas vários e não localizava ocorrências. Contagem visual a confirmar ao traçar; ocultos não incluídos. |
| Árvore clara da janela | Montante contínuo e espaços foliformes laterais; modelo com grande olho central | L08 ambíguo: reabrir limites dos vazios e coerência com L06. Grafo do modelo também diverge. |
| Água-furtada | Um pano amplamente visível e borda oposta parcialmente oclusa; duas placas rasas no modelo | N03 classificou configuração completa como observada; segundo pano completo precisa de hipótese. Volume já exigido em N01 foi mal executado. |
| Telhas | Peças menores/arredondadas e fiadas densas; quadriláteros grandes | Faltou escala amostrada na análise; borda arredondada já prevista foi omitida na execução. |
| Espigão da cobertura baixa | Faixa curva interna divide campos de telhas; não executada adequadamente | D04 já identificava a peça: falha de construção. |
| Forro da ponta alta | Forro/escoras escuros; grande região lisa de pedra no modelo | B09 explicitamente proibia preencher com pedra: falha de construção. |
| Medalhão | Rede foliforme e integração com a faixa; oito raios genéricos | RV08/Q09 já pediam folhas e diziam que não bastavam raios: falha de construção. |
| Colunas e arcos | Sulcos, capitéis e nervuras conectadas; perfis lisos/arcos simples | H04/H06/H09 já descreviam diferenças: presença de planta nativa não certifica o perfil. |
| Vidro e luz | Reflexos, objetos em camadas, luz localizada; fundos opacos escuros | Execução/capacidade de aparência pendente, não erro da classificação vidro. |
| Vegetação | Massas densas e heterogêneas; folhas e flores esparsas | Famílias identificadas corretamente; densidade, envelope e ramificação insuficientes. |

O restante da revisão detalha galeria, lajes/degraus, motivos das portas,
pilares/vasos, espessuras da cerca e remates. Não inventar erros de análise
para justificar requisitos que já existiam e não foram executados.

## Consequência para o procedimento

A próxima construção precisa começar por planta/seções, faces hospedeiras e
curvas de junção compartilhadas, conferidas em vistas distribuídas, antes de
buscar detalhe na mesma foto. Para cada motivo, guardar desenho/grafo e espaços
negativos; para repetições, localizar ocorrências; para famílias densas, guardar
amostras de escala/espaçamento. Uma frase como “tem árvore” ou “vários arcos”
ainda permite escolhas genéricas demais. A lista e a skill já pedem boa parte
disso; aumentar instruções sem conferir sua execução não resolve a lacuna.

Não há solver/segmentação/gate volumétrico novo nesta unidade. Testes de
conectividade, junções/apoios e reprojeção são próximos contratos de ferramenta
a discutir e implementar; não estão certificados por texto ou por SSIM.

## Verificação e custos

PNG nativo permanece SHA256
eabff512c0e70c8cc61cb33fd7144bcd97537bb1211de3ecbcc9da46df328bc2;
cena exportada e cena no root permanecem iguais.18achados referenciam IDs válidos
e evidências existentes. AST Python, sintaxe JS, geometria/índices finitos,
lookAt centralizando alvo, limites de pitch/zoom, pan e seleção dos18achados
verificados por `node scripts/verify_e02_viewer.cjs`. View contém17953v/19188t
após expandir instâncias no cliente; fontes originais preservadas.

WebGL compilação/render e interação física no navegador não foram testados.
Os testes JS de controles usam DOM simulado e não são prova de apresentação.
Capturas auxiliares são nativas; não validam a aparência do WebGL. Nenhum browser
foi automatizado por rota alternativa. Kernel/Rust/build/skill/receita/cena não
mudaram; nenhum processo da demo foi manipulado, sem push/publicação.

Revisão proporcional de performance:1.navegação/eventos;2–3.JSON parse e buffers
Float32/Uint16 uma vez, índices32só se necessário;4.instâncias expandidas uma vez,
render apenas dirty/resize/giro;5.zoom/pitch/tamanhoDPR limitados;6.buffersGL uma
vez e arquivos estáveis;7.sem rede/worker/lock;8.home restaura câmera, gestos só
em sessão;9.testes matemáticos/estrutura separados de apresentação real;
10.custos mantidos: cópia CPU da geometria, expansão das instâncias e callback
rAF leve mesmo parado. Nenhuma promessa de FPS, paridade de luz ou fechamento.
