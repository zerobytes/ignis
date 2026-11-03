# Vínculos de meshes por instância e osso

## Contrato anterior ao código

Motivo: reautoramos um galho da árvore, mas seus secundários e folhas ainda
dependem de posições soltas. Alternativas: vínculo novo por coordenada/frame
material da planta, ou composição de instâncias/ossos do rig genérico existente.
Escolha inicial: segunda alternativa, sem dependência nova de meshes em plantas.
A planta já transporta origens de ossos no seu domínio explícito; rotação do
osso continua autorada, sem inferir tangente de curva ou frame transportado.

Instance recebe attachment opcional `{parent: ID, bone: nome|null}`. Ausente,
position/rotation/scale são mundiais como antes. Presente, são locais à instância
pai ou ao osso posado dela. Matriz mundial=worldPai*ossoGlobal*transformLocal.
Bone nulo usa identidade. Cor/material/playback do filho são independentes.
Nome do osso é identidade validada, não índice que possa mudar silenciosamente.

Até32instâncias, ordem de armazenamento livre e grafo acíclico validado inteiro
no commit. Pai/asset/rig/osso ausente ou ciclo rejeita, incluindo remoção/renomeação
referenciada; editar em lote permite remover/desvincular dependentes na mesma
transação. Desvincular é put_instance com attachment ausente e TRS explícito:
sem keep-world implícito. Escala uniforme efetiva de cada cadeia fica0.001..100,
além dos limites locais existentes. Sem aumentar budgets. Campo omitido no
legado; arquivo com attachment exige58, replay62 guarda pose com world_transform.
Matrizes de ossos da inspeção continuam globais no espaço do asset, como antes.

Resolver topologia/IDs/ossos uma vez quando documento muda no Frame; ordem e
índices em arrays fixos. Amostrar skeleton uma vez por instância/tick, inclusive
pais sem geometria dependente animada; reutilizar em world/skin/inspeção. Pose,
captura, sombra e replay compartilham o mesmo caminho. Assets/arcs/canais de
repouso existentes; nenhuma fase de gameplay/física/percepção nova.

## Ensaio da árvore

Começar na cena salva488df28. Voltar só o galho para sua referência linear
original antes de AUTORAR rig/clip por setters existentes: rebase de metadados
é explícito, geometria/material/referência originais devem permanecer iguais.
Reaplicar alvoPCHIP médio e conferir preservação dessa fonte enriquecida.
Root de rig no colo, socket nomeado na inserção de um secundário; pesos da
madeira principal no root. Socket anima somente o secundário de teste, com
amplitude pequena, sem fingir vento de toda a árvore.

Extrair do asset wood um secundário completo e suas rametas, por intervalos de
índices reproduzidos pela receita atual (conferir exatamente antes de remover).
Novo asset/instância mais uma instância foliar já existente: previsão9assets/
31instâncias, mesmas contagens de geometria/desenho. Primeiro componente e
demais superfícies/canais intactos. Secundário anexado ao socket; coroa anexada
ao secundário, com playback próprio. Restante da árvore mantém autoria atual;
isso não anexa todos os outros componentes ou solda bifurcações.

Testar curva alterada no ponto dessa inserção, centros e poses do ramo/coroa
acompanham; sem exigir que outros secundários acompanhem neste recorte. Medir
erro em repouso da extração em coordenadas mundiais. Guardar before/after e
close do movimento, undo/redo/A-B-A, fonte/canais, rejeições e ciclo/replay.

## Revisão estática anterior

1. Hierarquia compilada na edição/invalidação, skeleton em frame/consulta,
   sem trabalho extra de gameplay/tick. Frame estático conserva short-circuit.
2. Arrays limitados32instâncias×32ossos, sem Vec/hashmap por osso/frame; fonte
   Geometry/Rig/Clip segue Arc. Snapshots JSON somente nas consultas/ensaio.
3. IDs autorais resolvidos em índices limitados; matrizesf32 existentes. Escala
   composta limitada para impedir cadeias exponenciais; sem unsafe/deps.
4. Preparar skeleton uma vez por instância e reutilizar skin/world/pose; topologia
   não recompila por frame inalterado. poses completo não reavalia pais por filho.
5. Limites16assets/32instâncias/32ossos e budgets mantidos, ciclos recusados antes
   da publicação. Profundidade máxima já limitada pelo total de instâncias.
6. Mesmo Vertex/shader/material/sombra, upload existente; capturas explícitas.
7. Dispatcher síncrono, sem lock/thread/fila nova.
8. Mudança de rig/instância/asset invalida mesmo Frame; erro preserva documento,
   runtime/histórico. Undo/load/reset/replay exercitados.
9. Testes numéricos independentes de composição, nomes/ordem/ciclos/limites,
   animações próprias/pai e coerência pixels/replay/formatos; árvore nativa.
10. Manter custo existente de skinning/upload e arrays por preparação; não
    alegar ganho de FPS/escala floresta, física ou alta qualidade/realismo.

## Resultado na árvore

Cena original488df28 foi carregada, não reconstruída. O helper reproduz a
receita somente para conferir índices: toda posição/canal quantizadof32 e
triângulo de wood coincidem exatamente antes da extração. A seleção do
secundário1/estação0.30 e das rametas de crown-06 remove71v/114t de wood e
publica secondary-06 com os mesmos canais, em coordenadas locais ao socket.
Não existe weld/nearest ou alteração da primeira junção. Seu prefixo1839v/
3674t permanece exato. Extração em repouso: erro máximo2.98e-8m; pose da copa
no tick0 exatamente igual à anterior.

Root/base do rig no colo; socket secondary-06 na inserção. Autoramos somente
rig/clipe na referência original linear258v, depois reaplicamos alvoPCHIP482v.
Fricção descoberta: setters de rig/clipe rebaseiam a fonte por contrato; canal
material reamostrado mudou1.3411e-7m. O helper restaurou explicitamente o canal
original com set_material_coordinates, ainda na referência original. Geometria
e coordenadas da fonte, material/shading e referência ficaram exatamente iguais
aos originais, acrescidos somente de rig/clipe. Não substituiu asset vinculado
por put_asset/bake: essa operação foi corretamente recusada. A fonte enriquecida
permanece exata nas edições subsequentes. Esse workaround mostra a oportunidade
de autoria de metadados de rig sem reamostrar canais; não foi implementada aqui.

Hipótese para investigar separadamente: coordenada materialf32 volta à célula
de referência com pesos quase one-hot em vez de identidade exata por índice.
Ainda precisa reprodução nativa focada. Não corrigir por snap/tolerância ou
nearest; uma correção numérica pode mudar o asset derivado que cenas antigas
validam por igualdade e exige política de compatibilidade. Não mudou nesta unidade.

tree-secondary-06 segue o osso; crown-06 segue a instância secundária e mantém
seu clipe/offset próprio. Alterar a estação0.30 em[0.15,-0.10] deslocou ambos
[0.150000036,0,-0.099999994]m. Assets do secundário e folhagem não mudam nessa
edição. Undo/redo retorna poses exatas. Movimento do socket1.5graus emZ foi
conferido contra rotação analítica da pose foliar antiga notick60, incluindo
o playback próprio; erro limitado5e-6m. Superfície da madeira principal fica
estática (pesos no base); só este secundário ganha movimento rígido autorado.

9assets/31instâncias,11172v/16250t armazenados e64794t desenhados,742t livres:
nenhuma geometria extra ou novo teto. Fonte5.663.332B em58/62, buildb0aba8aa96ad42cb.
O ensaio reaproveita as12comparações de modo/resolução, agora com a hierarquia
ativa; finalPCHIP[4,4,4]. Fonte/canais, persistência meshes+plants, execução/
replay/pixels e ciclo48frames/4s foram conferidos. GIF2.481.054B corrente.
Capturas attachment-rest/edited/animated e árvore inteira/detalhes inspecionadas.
Report.json e attachment-report.json ficam em artifacts/tree-study/artifacts.

Para repetir a autoria, usar
`python3 scripts/demo_tree_attachments.py --frames 48 --input artifacts/tree-study/examples/attachment-input.scene.json`.
O input é a árvore anterior sem vínculos. Em checkout novo, obter a fonte
488df28 do Git para esse arquivo (após criar os diretórios); não passar a cena
já vinculada como se fosse a fonte anterior. O script falha antes de substituí-la.
`demo_tree_smoothing.py` continua editando diretamente a árvore já vinculada.

## Limites e próximos testes

O modo de vínculo genérico está entregue para qualquer mesh, mas o ensaio só
converteu UM secundário/coroa. Outros ramos ainda são componentes de wood e
não devem acompanhar alterações arbitrárias do pai neste recorte.31instâncias
deixam1slot, portanto converter todos individualmente pede organização/extração,
compartilhamento de rig/agrupamento e orçamento antes de aumentar contagens.
Extração ainda depende do helper/índices; seleção/extração nativa por componente
é outra fricção confirmada. Nenhum gen especial de árvore foi posto no runtime.

Casca em rede, folhas sem nervuras/espessura, repetição da copa e encaixes
sobrepostos continuam reprovados artisticamente. Vínculo não solda nem produz
distribuição natural; movimento rígido pequeno não é física do vento. Próximo
incremento deve abordar organização/distribuição observável ou superfície,
mantendo essa mesma árvore como teste, sem confundir solução parcial e completa.

## Checklist posterior

1. Adequado: Hierarchy na validação ou mudança de Frame; static short-circuit
   mantido. Skeleton amostrado uma vez por instância e preparado compartilhado
   entre pose completa/vertices; nada no tick de gameplay.
2. Mantido com motivo: Prepared tem arrays de32×32matrizes posadas+skin, cerca
   de132KiB de payload temporário, além de scratch existente de posições/normais;
   custo limitado para evitar reavaliar ancestrais. Sem Vec por osso/frame;
   consultas JSON/Pose alocam explicitamente, fontes/histórico seguem Arc.
3. Adequado: IDs e nomes resolvidos em índices limitados, escala efetiva validada
   emf64; matrizesf32/legado mantidos, sem unsafe/deps.
4. Corrigido: poses inteiro compartilha preparação; não chama pose independente
   para cada filho. Cache de topologia só refeito quando documento muda.
5. Adequado: mesmas contagens e budgets; profundidade≤32 pelo grafo finito,
   ciclos e escala composta fora0.001..100 recusados antes da publicação.
6. Não se aplica pipeline novo: mesma geometria de Vertex, shader e sombra;
   capturas explícitas, paths estáveis e um build corrente.
7. Não se aplica sincronização nova: mesmo dispatcher, sem locks/threads.
8. Validado: pai/rig/instância/undo/load/reset invalidam Frame existente; erros
   de dependência/ciclo preservam documento, runtime e histórico.
9. Validado: testes analíticos de TRS/osso/chain/clock/formatos e ensaios reais
   da árvore; pose/shadow/replay seguem o mesmo caminho de world transform.
10. Mantido com motivo: skinning/expansão/upload CPU, snapshots de teste, hierarquia
    limitada compilada em consultas e amostragem de rig. Sem alegação de FPS.

Validação completa:748Rust/67grupos,Clippy release-Dwarnings/fmt/build/diff,
69MCP/285grupos e223build_id de relatórios coerentes no buildb0aba8aa96ad42cb.
Evidências em validation.md e artifacts/attachments-*.log.
