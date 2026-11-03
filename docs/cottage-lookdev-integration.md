# Integração da casa estrutural e aparência

## Composição conjunta no editor — 29/10/2026

A fonte corrente combina bronze.geometry corrigida460a407 e stone-blocks/paving
chanfradosf608818; todos os outros dados permanecem iguais. A receita v5 inclui
os parâmetros relief_front_offset/bottom/height da outra frente. Lookdev prende
o novo hash conjunto e32596v/54728t/55948draw,172v livres/16slots cheios; não
usa automaticamente o hash de qualquer origem isolada. Scene81/replay85 são a
união do editor, versões abaixo são históricas. Baseline material7286…f1e2c
permanece separada e suas comparações de pixels não são relaxadas.

editor_cottage_union_probe.py comprova cada delta contra os commits originais,
consulta topologia e compara imagens nativas repetidas. study_cottage_normals.py
valida a mesma pedra com spans[1,9]; carved-stone não muda na fonte canônica.
A prova histórica study_cottage_relief_integration.py conserva seu baseline v3
e não é aplicada implicitamente à cena que ganhou chanfros. A prova conjunta
usa as duas origens para evitar transformar um antes/depois diferente em prova
do caso antigo. Receita completa deve reproduzir a fonte unida exceto versão.
Sem aceite artístico, atualização de físicos ou conclusão de ferramentas UI.


## Integração atual no editor

A receita aponta explicitamente para1054577: fonte a742696f…57e6,
31956v/53448t/54668t desenhados,16assets/21instâncias. Mantém luz/materiais do
estudo, sem mudar a fonte. A fonte material de calibração7286e74c…f1e2c e seus
hashes de pixels continuam separados. Carved-stone é byte-exato à composição
v3 usada no diagnóstico119; este diagnóstico é repetido sobre a composição
nova sem publicar sua variante sobre a cena canônica. As medições antigas
abaixo ficam como histórico v3, não contagens da fonte corrente.
Scene80/replay84 são a união do editor;75cunha/76ambiente são gates históricos.
Ver [integração](editor-integration.md) para a validação conjunta.


Unidade posterior a cc4c6d3/EAF-116, integrando ec93bb2/b33a76c da frente nativa
sem tocar sua worktree. Objetivo: mesma casa atual com composição/Sweep e
materiais/ambiente compartilhados, mantendo as provas antigas reproduzíveis.
Não é integração no main/na Wedge75/79 funcional nem aceite de fidelidade.

Antes do merge/código: preservar ea8c6b8:elven-cottage.scene.json byte-exato como
examples/cottage-material-baseline.scene.json. É fixture de fonte ativa dos
probes, não build histórico. Recipes anteriores apontarão para ela com os mesmos
hashes de conteúdo/imagens; atualizar hashes de recipes dependentes porque o
nome muda. Casa atual permanece examples/elven-cottage.scene.json/b33a76c.

Checklist proporcional:1.Sweep só autoria, bake ambiente só conteúdo;
2.nenhuma cópia de geometria em runtime para aparência, fixture1.8MB de fonte
para comparativos;3.resolver enums/versões manualmente mantendo76/80 e fontes
ocultas;4.não recomputar curvas para trocar material/luz;5.caps16slots/tetos
mantidos;6.mesmos buffers/Vertex64/Globals192/Material1232, nenhum recurso novo
além do EAF-116;7.sem deps/locks/threads/janelas;8.runner reúne sweep+environment
e cache histórico da prova;9.Rust/Clippy/MCP completos e captura da composição
atual com ablações de luz, fonte/geometria preservada;10.source edits/scripts e
fidelidade artística separados, sem alegar FPS/normalização automática.


Resultado integrado b33a76c: build `10aa09af0bf647b9`.89cenários/365grupos
MCP headless aprovados, incluindo Sweep e ambiente. Os três estudos anteriores
(materiais, metal, ambiente) reaprovados com fixture7286e74c…f1e2c e pixels
legados exatos. A casa atual passa3grupos com16assets/21instâncias,32390v,
54068t armazenados/55288t draw. Materiais/luz só: geometria/fontes/canais/rig
preservados; preview/undo/load/reset/replay/prefixo/viewport e quatro câmeras
repetidas iguais. Um bake de probe e LUT,148984B lógicos.

Custos mantidos: fixture1.8MB para comparações, slots16 cheios,378v livres na
composição b33; nenhuma mudança nos layouts/tetos/deps. Inspeção visual ainda
mostra vidros opacos, pedra muito regular, jardim simplificado e transições de
ornamentos a investigar. Iluminação distante não resolve oclusão local.
Não há aceite artístico nem promessa de FPS. Prova em artifacts/cottage-lookdev.

Validação final do núcleo combinado:869testes Rust/88executáveis, Clippy
release/all-targets com-Dwarnings, fmt/build e py_compile aprovados.
Source fingerprint10aa09af0bf647b9 coincide com o binário e relatórios;
audit em artifacts/cottage-integration-build-audit.json. Logs estáveis
cottage-integration-{tests,clippy,build,mcp}.log.


## Revisão anterior à integração do telhado v3

Aplicar f0f9664 como conteúdo sobre1edebff.1.só autoria/estudo;2.sem dados
novos por frame;3.curvas finitas já admitidas pelo Sweep;4.mesmo binário;
5.tetos mantidos,31720v esperados;6.layouts/recursos intactos;7.sem deps/threads;
8.fonte atual e recipe hash vinculados;9.reexecutar estudo combinado com
ablações/save/load/replay/viewport e imagens;10.fidelidade e junções ainda abertas.
Contrato de Sweep e GPU não mudam, então não repetir suíte Rust/MCP completa
sem mudança nesses fontes. Registrar source fingerprint e escopo do merge.


## Resultado atual v3

Composição f0f9664 com materiais/ambiente:31720v/52988t armazenados/54208t draw,
16assets/21instâncias,1048v livres. Três grupos do estudo combinado passaram;
quatro vistas repetidas, ablações, preview/history/save/load/reset/replay/prefixo/
viewport preservados. Um probe/uma LUT, sem recurso adicional. Fonte compacta
SHA a69488c31be211e7f9d1d5209b38629a7f59d877cea1d79d33c8cb8092da98ed.
A recipe prende fonte/contagens/revisão e o estudo usa esses valores explícitos.

Este merge altera só conteúdo/docs/clientes Python: o source fingerprint
10aa09af0bf647b9 e binário permanecem os validados por869Rust/89MCP.
Nenhuma suíte completa repetida; gates afetados foram reexecutados no estudo
combinado. py_compile e diff check aprovados. Custos do checklist mantidos:
mesmos tetos/layouts/cache; menos670v/1080t armazenados, sem medir FPS.

Revisão visual independente: beiral mais contínuo e molduras mais legíveis;
close da entrada ainda revela uma linha entre metades do arco e terminações
expostas. Hipóteses de autoria/normal/encaixe precisam diagnóstico separado,
sem suavizar ou inverter faces automaticamente. Vidro opaco, pouca variação
na pedra e vegetação/chão simplificados ainda impedem o realismo pretendido.
Fricções de plataforma seguintes: seleção/identidade de peças agrupadas,
material por região e diagnóstico de junções/shading. Não há gerador de casa
novo, aceite artístico ou integração na frente funcional Wedge.

Continuidade: os dadosv3 acima são o baseline anterior à ornamentação. O resultado
corrente combina relevosv4/colocação corrigida e a mola com resolução por trecho;
reprodução, novo binding e evidência em
[cottage-relief-integration.md](cottage-relief-integration.md).
