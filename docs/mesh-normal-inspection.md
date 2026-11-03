# Inspeção de normais avaliadas

## Decisão antes do código

A casa v3 mostra linhas e terminações nos arcos. InspectMeshPose retorna posições,
mas não revela as normais efetivamente enviadas ao GPU. Inferi-las no cliente
repete o algoritmo e pode mascarar diferenças após skinning ou transformação.
Persistir normais autorais, adicionar crease angles ou soldar peças implicaria
novos contratos de edição e transferência antes de comprovar a causa.

Escolha: query pura `inspect_mesh_normals` por instância, tick opcional, faces
opcionais e sample_limit0..32 (default8). Avaliar a mesma pose e extrair a rotina
de normais já usada pelo renderer sem mudar sua aritmética. Sem formato/layout
novo, overlay ou correção automática. Relatório em mundo: área geométrica,
normal de face e normais por canto efetivas, ângulos face/canto, fallback,
contagem de cantos opostos e amostras de maior desvio (empate por face).

Faces são índices do asset da instância, não IDs persistentes de peças.
Filtro não vazio/único/no orçamento, normalização de ordem; null seleciona todas.
Suavização considera todas as faces incidentes, mesmo fora do filtro. Tick
explícito0..36000, omitido usa simulação atual sem avançá-la. Referência de sessão
permite detectar índices obsoletos. Normais são pré-material: bump/procedurais,
orientação para câmera e iluminação do shader não fazem parte do diagnóstico.
Desvio não é erro: quinas e caps podem ser intencionais. Posição coincidente com
índice distinto não cria vizinhança; não detectar automaticamente weld/gap.

## Checklist anterior

1.Só consulta explícita; rotina do frame mantém frequência e passes originais.
2.Pose aloca posições como query existente; scratch normals fixo pelo teto do
asset, respostas limitadas32faces; não clonar geometria/rig/material.
3.Mesma f32/aritmética do render; ângulos/área de relatório em f64 finitos.
4.Calcular normais uma vez/instância; filtro não restringe incidências, topK32.
5.Mesmos tetos; filtro limitado às faces antes da cópia/ordenação.
6.Sem buffer/shader/readback/textura ou captura implícita.
7.Sem thread/lock/dependência; dispatcher existente.
8.Sem estado/cache persistido; pureza e referências/history/load verificáveis.
9.Analíticos, cancelamento/seams/pose/rig e igualdade exata com Vertex.normals;
regressão completa por tocar rotina compartilhada do renderer, casa com baseline.
10.Mantidos: pose e normais derivadas por pedido, CPU síncrona, ausência de
interseção/oclusão/normal map/solver; sem FPS ou realismo inferidos.

## Ensaio de correção, antes da autoria

A consulta na mola esquerda da porta encontrou13pares de índices coincidentes,
com máximo52.6037° entre normais; sem fallback/normal oposta. O perfil da reta e
curva termina/começa no mesmo plano, mas cada peça está fechada com seu próprio
cap e as incidências são separadas. Testar somente os sweeps0/1 da pedra talhada
como cadeia Bézier regular única, usando planner e tolerância existentes. Não
soldar por distância nem alterar normais no cliente. Compor linha+curva pode
refinar ambas pelo mesmo spans; custo deliberado do contrato atual, preflight
antes de publicar. Fonte canônica v3 preservada; estudo salva variante própria.
Antes da mudança, regeneração do asset original deve ser exatamente igual.
Depois, demais assets/instâncias/fontes invariantes e resto dos spans exatos;
medir vértice antes duplicado/normais compartilhadas e comparar câmera/luz/material
fixos, undo/redo/load/replay. Crown não é tangente: não juntar por esta regra.

## Resultado do ensaio na casa

Build55b307c7b20159ec. `study_cottage_normals.py` carrega o resultado reproduzível
`study_cottage_lookdev.py`, confirma captura da entrada byte-exata no novo build,
consulta três regiões e salva baseline/flat/material-neutro sem alterar a fonte.
Mola da porta:76faces selecionadas,13pares de índices coincidentes, maior salto
52.6037316°. Crowns da porta/pórtico não têm pares coincidentes nessa tolerância;
portanto este diagnóstico não autoriza soldá-los nem atribuir sua linha a cap
coplanar. Nenhuma região tem fallback geométrico/smooth ou cantos opostos.

A variante reproduz asset original exatamente antes da alteração; reúne apenas
sweeps0/1 numa cadeia regular. Preview nativo/commit iguais; demais assets/fontes/
instâncias e todos os spans/canais posteriores exatos, salvo remapeamento de
índices. No contato examinado, dois índices viram um e todos os cantos incidentes
usam a mesma normal: salto52.6°→0°. A captura melhora a passagem reta/curva; topo
pontudo e outras junções continuam. Undo/redo/save/load/replay/capturas exatos.

Custo identificado: planner sobe5→10intervalos por cúbica para respeitar o erro
central≤0.004867m. A reta recebe a mesma resolução da curva: +72v/+148t nesta
correção, total31792v/53136t,976v livres; slots16/instâncias21 mantidos. Isto
expõe uma próxima fricção genérica: amostragem por trecho evitaria tesselação
redundante das retas. Não foi expandido o contrato nesta unidade.

Reprodução: executar `python3 scripts/study_cottage_lookdev.py` e
`python3 scripts/study_cottage_normals.py`. Variante em
artifacts/cottage-normals/examples/cottage-spring.scene.json; capturas
baseline-entry/joined-entry e relatório no mesmo diretório artifacts/.
A fonte canônica da casa v3 e a árvore não mudam; a variante é evidência de
correção localizada, não atualização silenciosa de toda a casa.

## Revisão estática final

1.Query só em Engine::execute; nenhuma simulação adicional ou captura implícita.
2.Document::pose mantém seu buffer/Vec de posições; scratch de normais192KiB,
tamanho fixo16384Vec3, e topK com capacidade≤32; geometria/rig emprestados.
3.accumulate/face/corner mantêm as operaçõesf32 e fallback exatos do render;
ângulos normalizados e área em f64 apenas no relatório, sem alterar vértices.
4.Um passe de soma, um de normalização e seleçãoO(F×32), filtro ordenado uma
vez; vizinhança completa mesmo para uma face pedida. Sem busca global entre
peças no runtime; pares do estudo são análise restrita do cliente.
5.Tetos de asset/documento mantidos, faces filtradas não vazias/únicas/atuais;
query tick≤36000, respostas≤32faces, sample_limit0 calcula só estatísticas.
6.Nenhum novo recurso/layout/shader/readback; helper extraído não muda soma,
ordem ou publicação de Vertex. Quatro vistas da casa permaneceram byte-exatas.
7.Dispatcher sequencial, sem lock/thread/dependência nova.
8.Nenhum estado de diagnóstico persistido/cache; queries mantêm scene/revision/
history/runtime/pixels. Mesmos formatos76/80 e limitações de replays por build.
9.Quatro testes Rust novos com igualdade exata contra Vertex;4grupos MCP novos
mais4grupos no estudo da casa. Pureza/erros/ordem/tick/filtros/seams/cancelamento/
rig/colapso e history/load/reset/replay cobertos. Gates completos registrados
abaixo ao finalizar.
10.Custos mantidos: pose/scratch/sort por consulta, staging na autoria e uniforme
spans por curva; correção de um contato custa72v. Normais pré-material não
provam orientação global, qualidade de encaixe, interseção, realismo ou FPS.


Validação final:873testes Rust/88executáveis release/all-targets aprovados,
Clippy-Dwarnings/fmt/build/py_compile/diff check aprovados.90cenários/369grupos
MCP headless passaram (4grupos novos),4grupos próprios da casa e4vistas anteriores
byte-exatas. Build/source fingerprint55b307c7b20159ec e binário auditados em
artifacts/normal-inspection-build-audit.json. Logs normal-inspection-{tests,
clippy,build,full-mcp}.log; comparativo anterior em normal-inspection-legacy-pixels.json.
Sem janela/main/push, fonte canônica a69488…98ed e árvore preservadas.

## Continuidade EAF-120

Os números+72v/+148t acima descrevem o ensaio original de c5a6a13. O reprodutor
corrente agora compara essa variante uniforme com spans[1,9]:80v/160t a menos,
posições do span iguais à fonte sem o anel duplicado, salto0 no mesmo contato.
Dados atuais e limites em [sweep-curve-sampling.md](sweep-curve-sampling.md).
Hero e cena de trabalho correntes são salvos após integrar materiais/ambiente;
a integração de ornamentos mantém carved-stone byte-exato antes da reautoria.
