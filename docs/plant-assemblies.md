# Composição regenerável de uma planta com contexto

Implementação local Scene66/replay70. Adoção anterior em
[tree-trajectory-integration.md](tree-trajectory-integration.md). Integra os
kernels puros de8f23990/979ff26 com fontes e identidade da planta. Exemplo de
estudo, sem aceite artístico ou conversão de todos os galhos.

## Alternativas e escolha

Publicar a união como malha independente encerra a regeneração por plantas.
Guardar índices da última malha como fonte acumula deformações e perde o estado
canônico. Escolha: receita opcional no vínculo da planta, contendo um
Asset de contexto imutável, faces do contexto a remover, dois ciclos identificados
por índices canônicos e uma seção interior nomeada da planta. A receita retém a
geometria da planta depois dessa seção e recompõe as três portas a cada edição.
O kernel recebe só o asset já recortado, ciclos reais e opções físicas explícitas.

A fonte e referência históricas da planta permanecem intactas. O contexto não
é uma referência viva a outro asset editável. Isso evita dependências circulares,
mas exige reinstalar sua fonte explicitamente para mudar o contexto. A receita
só aceita mesmo material/shading, presença compatível do domínio material e
contexto sem skeleton. Se a planta tiver rig, uma raiz nominal explícita recebe
os pesos do contexto e da região nova. Pesos retidos da planta são copiados por
índices; o kernel recusa bordas com influências incompatíveis.

Não há promessa de conservar escultura de faces removidas dentro do miolo.
Pontos/canais/bones/clipes fora do corte mantêm origem verificável. O socket030
fica no corpo retido, antes da união. Escolher044 perderia seu suporte geométrico.

## Edição, formatos e validação

Operação `set_assembly`, também em preview e lote do dispatcher.
A remoção explícita retorna a planta isolada; undo recompõe o estado anterior.
Parâmetros da planta regeneram sempre a partir de suas duas fontes. Editar
diretamente a geometria composta é recusado para evitar rebase acidental.
Metadados de aparência atualizam as duas fontes mantendo seus domínios.
Alterar rig/clipes após compor requer contrato próprio ou retirar a receita.

Preflight de fonte e contagens após cortes precede concatenação; o kernel limita
adicionais antes de publicar. Saída, documento e fontes canônicas continuam com
budgets validados. IDs da seção e das portas devem resolver exatamente; fase,
coincidência aproximada ou nearest-surface não são fonte de identidade. Formato
persistido exige66;65 rejeita assembly ou assets acima dos limites anteriores.
Replay corrente70; versões são locais desta frente, sem unificar main.

Testar pureza/preview, rollback de todos os erros, parâmetros/densidade A/B,
remoção/undo, fontes/canais/rig/clipes exatos, fronteiras/ângulos reais,
save/load/replay/prefixo e capturas na mesma árvore. Comparar poses0/60 dos
sockets e descendentes; contagem fechada não é aceite artístico.

## Checklist anterior ao código

1. Só autoria/query, nada novo no tick.2. Arcs de fontes, buffers temporários
limitados e preflight; evitar cópias por probe.3. Receita tipada, índices
canônicos, matemáticaf64.4. Preparar cortes/loops uma vez por avaliação, sem
nearest.5. Tetos por asset/documento/fontes antes de publicar.6. Sem novo shader,
layout/readback por frame.7. Mesmo dispatcher/staging/undo, sem dependências,
locks ou threads.8. Invalidação existente.9. Matriz acima e regressões Rust/MCP
após integração.10. Custo síncrono da união e domínio tubular limitado aceitos;
sem boolean geral, prova global de autointerseção ou ganho de FPS presumidos.

## Ensaio de resolução antes de alterar tetos

A composição de referência passa com3995v7986t, mas deixa apenas101v por asset;
o colar continua perceptível. A amostragem de campo usa9044pontos a70mm. Um
ensaio a37,5mm estima menos65536pontos e ramo[8,8,8] mantém32v no contorno,
mas a saída pode precisar aproximadamente7,5kv. Dividir em vários assets exige
novos contratos de propriedade/IDs e cria bordas de desenho; elevar apenas
budget finito é a alternativa de menor custo estrutural para este ensaio.

Proposta local66, ainda em validação:8192v16384t/asset,32768v65536t armazenados;
draw131072 não muda. Cenas anteriores66 recusam excessos também na fonte
canônica. BuffersGPU/layout não mudam, pois o teto desenhado já é131072t.
Payloads máximos CPU de posições/domínio/índices/pesos dobram, aproximadamente
1,1→2,2MiB por conjunto autorado completo sem clipes; Arcs/staging existentes.
Normais/topologia por asset podem dobrar. Trabalho de tick/render continua
proporcional às instâncias/triângulos realmente usados. Guardas permanecem90°.

O contorno extraído pode precisar64→128v; DP fica limitado a33×129 estados,
8fases/3portas, só autoria. Grade65536 e raw16384v32768t permanecem. Não é
aumento silencioso nem promessa de qualidade: se a vista neutra não melhorar,
a candidata não deve ser promovida só por ser mais densa. Testar admissão/recusa,
versões antigas, limites compostos, save/load/replay e geometria efetiva antes
de consolidar esse teto. Sem aumento de limite de transporte ou FPS alegado.

## Orçamento medido e preview transacional

O ensaio efetivo a37,5mm produziu7357v14710t no conjunto e16208v26326t no
DOCUMENTO: cabe nos tetos totais antigos16384v32768t. Decisão final: elevar
somente asset4096v8192t→8192v16384t; totais e draw131072 permanecem. As propostas
maiores acima registram a hipótese anterior, não os limites finais. O custo
máximo global de posições/domínio/índices/pesos não dobra; só temporários por
asset podem dobrar. Cenas<66 recusam asset maior também em fonte canônica.

O batch remove o asset temporário extraído antes de instalar a receita, cuja
fonte é um valor imutável próprio. Assim evita duplicação intermediária acima
do orçamento sem apagar fonte nem crescer teto total. Uma consulta nova
`preview_scene_edits` compartilha EXATAMENTE o staging/referências do edit_scene,
valida/normaliza/ordena a cena prevista e não toca revisão/tick/histórico/seleção.
Evita mutar/desfazer só para prever o recorte+composição. Limite512operações
existente; uma cópia de Scene com Arcs e buffers limitados, sem simulação nova,
lock/dependência/shader. Testar pureza, comparação com publicação, erro/budget e
referências obsoletas; manter a avaliação de autenticidade da fonte na validação.


## Resultado integrado e revisão final

A árvore mantida recebe ramo[8,8,8],seção030 retida,cell37,5mm,684pontos iso
suavizados em4passos/envelope12,5mm e domínio material em64passos.7357v14710t
no conjunto,16208v26326t armazenados,74870t desenhados,10assets32instâncias.
52920pontos de campo; contornos32/32/32→99/56/63. Fontes/reference/rig/clipes
mantidos; outros assets exceto recorte wood e poses0/60 exatos. Contexto muda de
frame local, com erro mundial medido1,2645e−7m. Escultura não nula de1mm e domínio
material são verificados no fixture Rust independente, incluindo pontos/pesos
retidos; o ramo desta árvore tem displacement0, e o contexto mantém sua forma.

Pico da região nova59,7362°(prova inicial)→38,8196°,costura iso31,0997°,média
2,78647°,costura fonte1,15059°. Não é C1 analítico entre triângulos. Deslocamento
novo máximo11,0724mm; volume FECHADO total1,749515176→1,749468411,sem garantia
local. Resíduo do campo nos pontos iso antes5,76e−8m e após suavização4,2077mm;
isso não é distância Hausdorff nem limite global dos triângulos. Domínio da guarda
copia a fonte (diferença3,59375→0),CG finito64 com resíduo pós-f320,00534755;
não afirmar convergência exata nem veios físicos sem distorção.

Revisão final:1.Só autoria/query, união não é avaliada por tick.2.Arcs mantêm
fontes; compaction da planta usa contagem já gerada, removida uma lista/varredura
redundante.3.Índices canônicos, enums e f64 temporário.4.Staging e Scene.validate
reevaluam a receita para conferir autenticidade; custo duplicado de autoria é
mantido, limitado, sem cache especulativa.5.Somente teto por asset ampliado;
globais mantidos e preflight antes da concatenação/adicionais.6.Layout/shaders
inalterados. Mais triângulos e contexto agora ligado ao rig usam CPU skinning/
expansão existentes; clock do clipe ainda invalida o asset inteiro, mesmo quando
só socket não ponderado anima. Custo mantido/visível, sem FPS presumido.7.Mesmo
dispatcher, nenhum lock/dependência/worker.8.Invalidação por edição e escopo de
componentes existentes.9.Rust/MCP cobrem fonte decorada, prefixo/pesos, erros,
versões, preview de lote, undo/ciclos/save/load/replay/prefixo/captura e budgets
máximos com toro fechado8192v16384t.10.Mantidos domínio tubular/raiz rígida,
amostragem finita, material distorcido e ausência de prova global de embedding.

Nova escultura diretamente no miolo composto ainda requer camada autoral com
âncoras estáveis; o rebase de uma malha avaliada é recusado. A escultura canônica
prévia continua nos mapeamentos e nas regiões retidas. A fonte completa do
contexto conserva também os dados cortados, mas esses detalhes não são
transferidos implicitamente ao miolo novo. Outros galhos estáticos, continuidade
de gradientes da casca e acabamento do colar permanecem trabalho aberto.

Reprodução: `IGNIS_TREE_HIGH_RES=1 python3 scripts/demo_tree_assembly.py` usa o
snapshot Git1d36a65 da MESMA árvore e gera um candidate nativo/capturas/relatório
em artifacts/tree-assembly. `--frames 48` requer Pillow
e produz GIF48frames/4s com48PNGs distintos e poses de todos os objetos iguais
após240ticks. Testes MCP de integração usam o exemplo mantido, sem depender do
histórico Git. Resultados automatizados finais em validation.md; não são aceite
humano nem prova de realismo.

## Enxertos laterais estáticos (Scene68)

O contexto agora aceita `context_grafts` ordenados antes do corte principal.
Fontes filhas imutáveis e quadros explícitos, máscaras canônicas expandidas por
origem e mapas parciais após cada corte permitem regeneração sem bake. Ver
[contrato e limites](context-grafts.md). `source_translation` altera somente
geometria avaliada, não o domínio material nem a fonte armazenada. Child rigado
é recusado; não inferir skin. Fontes filhas entram nos budgets canônicos e nas
mudanças de aparência. Gate68 e replay72 locais; contratos antigos preservados.

Acabamento69: `context_grafts[].collar_surface` deriva região por distância de
arestas às portas do enxerto, depois do filtro iso. Pode mover posições retidas
selecionadas e guardas; o fonte original permanece. As outras portas/guardas são
pins se alcançadas; interfaces da assembly principal não podem mudar. Defaults
sem campo continuam exatos. Report do kernel cru não é report do colar acabado.
