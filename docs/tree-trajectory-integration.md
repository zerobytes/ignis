# Trajetórias na árvore existente — checkpoint65

Este é o checkpoint1d36a65; composição corrente66 em
[plant-assemblies.md](plant-assemblies.md). Integração gráfica local de815da1d e5ee628d. Scene65/replay69 são versões
locais desta frente; numeração funcional de main exige integração separada.

## Decisão anterior ao código

Detach/bake perde a regeneração. Inserir um eixo na referência altera a fonte
histórica. A escolha é `adopt_trajectory`, explícito e permitido somente num
vínculo cuja definição atual e referência ainda não tenham trajetória. Conserva
os Arcs da fonte e referência; a inversa continua usando a referência legada,
enquanto a ida usa as seções normais ao eixo novo. Perfil/seções/estações mantêm
as identidades. `put` compara também a definição atual, impedindo adotar, remover
ou trocar o ID do eixo implicitamente. Para remover/trocar, detach explícito.

A adoção muda a base geométrica. Escultura e translações de animação seguem as
políticas XYZ existentes; rotações de rig não são inventadas. Preservar os nomes
não basta: posições dos sockets e poses dos descendentes devem ser conferidas.
Scene64 rejeita referência sem eixo/alvo com eixo;65 admite esse estado sem
reinterpretar documentos antigos. Neste checkpoint não havia campo assembly no produto.

## Orçamento de desenho

A árvore de referência usa65370 triângulos desenhados, deixando166 no teto
anterior. Aumentar resolução sem reduzir folhas/escultura requer ampliar somente
o desenho:65536→131072. Permanecem4096v/8192t por asset,16384v/32768t armazenados,
16assets/32instâncias. `inspect_meshes.limits` expõe todos esses tetos. Scene<65
rejeita mais65536t, e a guarda histórica55 para mais32768t continua.

Vertex52B e buffers dinâmicos permanecem. Capacidade por potência de2 pode subir
de262144→524288 vértices expandidos,13→26MiB por Vec/buffer; ativo máximo19,5MiB.
Até87381t, a capacidade anterior ainda serve. Expansão, skinning e upload mantêm
custo proporcional aos triângulos; nenhum ganho de FPS é presumido.

## Aplicação à mesma árvore

`demo_tree_trajectories.py` parte do snapshot da árvore mantida, altera somente o
ramo vinculado `branch-editable-1`, e preserva os demais assets/instâncias.
Ajusta o eixo cúbico com endpoints fixos e a estação `branch1-030` como restrição:
é o socket real da cadeia secundária. Mantém alturas/IDs, zera offsets das
seções alvo e usa resolução[6,6,6]. Fonte Linear[4,2,4] e referência continuam
intactas. A tolerância lateral é4mm, erro amostrado medido pelo kernel.

Verifica preview puro, undo/redo, ciclos A/B, domínio material, fonte/referência,
poses0/60 de todos os outros objetos com erro abaixo1µm, save/load e PNGs reais
vivo/replay/prefixo. A árvore existente não tem toda sua geometria ligada às
plantas; converter esse ramo não significa converter os demais galhos.

## Encaixes: pesquisa separada

Uma trajetória curva não resolve a interseção entre tronco e galho. Tentativas
de loft direto, correspondência polar, guias e fairing não atingiram qualidade
aceitável. Soluções de energia também convergiram com dobras: convergência não
prova embedding. Esses protótipos não estão compilados nem persistidos como API.
Repro e fontes rejeitadas ficam em `artifacts/junction-research`, ignorado pelo
Git, para continuidade da frente de malhas. Nenhuma guarda90 foi relaxada.

Recortar até `branch1-044` removeria a região do socket030, mesmo mantendo o
bone. Portanto o caso reproduzível conserva o corpo desde030 e usa três portas:
tronco inferior, ramo, tronco superior. A solução deve preservar fonte e canais
fora do recorte; superfície nova não recebe falsa identidade de escultura antiga.
A frente de malhas está avaliando uma superfície de união de volumes tubulares.
Integração viva da junção com regeneração continua pendente dessa solução.

A guarda de orientação do bridge existente foi corrigida para avaliar a derivada
no centroide real: pesos1/3 e2/3, em vez de0,5 nos dois triângulos. Controles,
pontos e guardas de ângulo permanecem.

## Revisão de performance

Checklist anterior: somente autoria; Arcs e buffers limitados; variante tipada;
identidades antes da mutação; preflight de contagens; nenhum shader/I/O por tick;
mesmo dispatcher/undo; invalidação existente; testes de preservação e replay;
custo síncrono e certificado conservador mantidos.

Respostas finais proporcionais:1. Nenhum trabalho novo por tick.2. Adoção conserva
Arcs, sem copiar a fonte; saída usa staging existente.3. Enum tipado/f64 temporário.
4. Inversa/amostragem existentes, sem nearest.5. Somente draw ampliado, com gates
por versão e preflight; contagens de armazenamento mantidas.6. Layout GPU mantido,
custo máximo de memória explicitado acima.7. Nenhuma dependência/lock/thread novo.
8. Invalidação/undo existentes.9. Rust e MCP cobrem fonte decorada, densidade,
identidade, rollback, rig/clipes, versões e pixels.10. Mantidos autoria síncrona,
erro superficial amostrado e ausência de certificado global de autointerseção.
Evidências e limitações finais em validation.md; capturas não são aceite humano.
