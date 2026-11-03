# Extensão tangencial do domínio material — prova pura

Decisão anterior ao código, EAF-85/EAF-80: operar na geometria FINAL, após iso e
collar. O dado recebido é um Asset explícito com canal material. Patch informa
prefixos retidos, intervalo iso e pares[source,guard]. Nunca descobrir
correspondências por proximidade nem inferir uma origem comum para pai/filho.

Estimar o Jacobiano tangencial de cada face retida incidente às fontes dos
pares, na geometria final. A previsão da variação de material até cada guarda
é ponderada por área. Guard_target=sourcecoord+Javg*(guardpos-sourcepos).
As coordenadas da fonte são imutáveis. Dispersão entre previsões é observável;
esta média discreta não certifica derivada contínua/C1 nem orientação de fibra.

Resolver no interior o Laplaciano uniforme da CORREÇÃO a uma referência AFIM
explícita (3colunas+offset), com diferenças nas fontes/guardas prescritas. Esse
referencial é um prior declarado, não inferência dos quadros pai/filho. Método
CG com budget1..64, envelope de mudança de coordenadas e tolerância do resíduo
normalizado por grau, medida novamente após f32. Sem clamp silencioso. Toda
componente desconhecida deve tocar um vértice fixo. Não é unwrap/ARAP/solver de
fibra nem garantia global de ausência de sobreposição.

Plano: retained_vertices/retained_faces (prefixos), iso_vertices (intervalo
seguinte) e pares[source,guard] cobrindo todos os últimos vértices. Faces
retidas não podem conter pontos móveis. Fontes dos pares devem estar na borda
real do submesh retido, arestas source→guard precisam existir. O patch admite
partes abertas; todo o prefixo retido, inclusive suas bordas não emparelhadas,
permanece fixo. Coordenadas fora de
guardas+interior, posições, triângulos, rig/clipes/aparência permanecem exatos.
Fonte já singular na face incidente à porta é recusada comID; não inventar
direção para passar no gate de guarda regular. É um limite desta prova, não
uma proibição de materiais constantes/posto1 no renderer ou no diagnóstico.

Métricas3D antes/depois por faixa fonte→guarda e restante afetado; XY não vira
gate3D. Recusar singularidade nova/remanescente nas faces afetadas ou anisotropia
acima do teto explícito. O teto não implica melhora média: reportar a mudança
de região, máximo/média e área sem misturar com faces originais. Controle de
rejeição usa o valor final quantizado, não o resultado temporário f64.

Alternativas: nova política no union exigiria reexecutar depois do collar e
poderia difundir duas vezes; remapear tudo por posição apagaria referenciais
autorados; clamp esconderia extrapolação inválida. A operação pura separada
permite comparar candidatos antes de desenhar a persistência. Rawunion e
CopyGuard existentes ficam intactos; nenhuma migração de Scene/replay aqui.

Checklist anterior de performance:1.só autoria/preview;2.CSR uma vez, mapas de
papéis/pares e buffersCG limitados por Geometry, sem alocações por iteração;
3.f64 temporário/f32 validado, IDs/finito/opções verificados antes do solve;
4.um passe das faces retidas+grafo e CG O(iteracoes×arestas incidentes), sem
buscas globais por par;5.mesmos tetos de geometria/MCP, budgetCG explícito;
6.sem shader/layout/GPU novo;7.sem lock/thread/dependência;8.saída compartilha
rig/clips e só substitui Geometry, nenhum cache/persistência novo;9.fixture
afim, guardas colapsadas, gradiente em geometria final, fonte/rig/fora exatos,
recusas/âncoras/CG/f32/MCP e ensaio da árvore;10.custo de inspeção/relatórios e
clone da Geometry mantido na autoria, sem promessa de FPS/realismo.

## API e limites

`preview_mesh_material_finish {source:Asset,plan,options}` é uma consulta pura.
Plan contém retained_vertices,retained_faces,iso_vertices e pares[source,guard].
As guardas ocupam o último intervalo; pares são ordenados por guarda para que
a ordem recebida não altere o resultado. A fronteira da fonte deve ter grau2
nos vértices emparelhados, e winding/arestas manifold são validados.

Options exige referência afim (columns[3][3]+offset, colunas), iterations1..64,
tolerance1e-9..1, max_change1e-6..10000, max_anisotropy1..1e8. Tolerância mede
resíduo do campo CORREÇÃO normalizado por grau, após quantização f32. Não é
tolerância de coordenadas entre pai/filho nem erro de reconstrução geométrica.
Envelope mede unidades materiais e admite1e-6absoluto para quantização, não
metros físicos. Referência pode ser singular; ela é prior, não uma inversa.

CG pode parar antes do budget quando o resíduo temporário chega a um quarto
da tolerância; o resíduo pós-f32 é a condição efetiva. ZeroRHS usa0passos e
reproduz referência afim no interior. Falha de convergência/cap/envelope tem
mensagem própria. Todo resultado aceito contém métricas ANTES/DEPOIS por fonte,
guarda,zipper,iso; cap finito não é promessa de melhora em todas essas regiões.
Publicação explícita de um asset livre usa edit_mesh; receita regenerável,
preservação de vínculo e integração regenerável não fazem parte desta prova.

## Prova na junção final da mesma árvore

Repro `scripts/study_tree_material_finish.py` recebe o Asset filtrado da prova
collar-study, a entrada local-graft e a cena isolada de captura. O estudo
somente consulta o kernel e renderiza snapshots locais; não altera a árvore
do integrador, fontes canônicas ou sua receita69. Material/sulcos são os da
árvore atual, copiados igualmente nas duas variantes de diagnóstico.
Report/inputs/saída e imagens em `artifacts/tree-material-finish/`.

Plan3242v/6407faces retidas,2506iso,71pares; referência identidade+offset
[.19227563,3.531,.14530726],64passos,tol.002,max_change1,cap10000.
Resíduo final .00084577597, máximo de alteração .032512886unidades materiais;
71guardas e2506iso mudaram. DispersãoRMS máxima da previsão por faces .002664079,
erro de quantização máximo1.2222e-7. Posições/triângulos/rig/clips/aparência e
materiais retidos exatos; PNGs neutros antes/depois idênticos.

| Região | Singular3D antes→depois | Anisotropia média antes→depois | Máxima antes→depois |
|---|---|---|---|
| Fonte6407faces | 0→0 | 1.04244→1.04244 | 21.7686→21.7686 |
| Guarda142faces | 142→0 | null→1.21496 | null→1.35513 |
| Iso4865faces | 0→0 | 6.79458→6.41214 | 1626.695→207.799 |
| Zipper220faces | 0→0 | 5.80355→6.73360 | 53.9745→110.824 |

**O zipper piorou**, embora a guarda tenha recuperado posto2 e o iso tenha
melhorado. XY precisa ser lido à parte: iso média154.54→27.44,máximo593223→11017;
zipper singular1→0, mas média35.69→2789.20,máximo1497.8→193230. Retirar um
colapso exato não garante projeção bem condicionada. As imagens com casca
mudam, mas ainda há banda/sulco e veios pouco naturais. Não promover esse
resultado como casca resolvida nem transferir médias entre regiões.

Os quadros materiais de pai/filho continuam diferentes por autoria. Corrigir
esse conflito ou trocar a projeção dos sulcos exige decisão explícita de domínio,
não remapeamento silencioso das fontes nem mais iterações indiscriminadas.
Esta unidade entrega a ferramenta/prova com limites, não uma árvore realista.

## Alternativa avaliada sem mudar o kernel

Ensaio numérico temporário de energia bi-Laplaciana da correção, sugerido na
revisão coordenada: E=||Lδ||², L combinatorial total incluindo linhas fixas,
A=L_UᵀL_U e RHS=−L_UᵀL_Fδ_F. Precondicionador diagonal d²+d; mesmas guardas,
fontes e referência da prova. Protótipo numérico externo ao produto; campos
quantizadosf32 medidos pela consulta NATIVA de domínio. Não é uma implementação
MCP/Rust desse solver e não é prova de aceitação de uma nova API.

Resultados em artifacts/material-biharmonic-study/report.json (inclui64):
64passos não convergem, resíduo normalizado .00283833>.002. Budget256parou225,
resíduo pós-f32 .00048837. Resíduo aqui é L²δ dividido por d²+d, não o mesmo
resíduo do solver harmônico: não comparar seus números como ganho equivalente.
O zipper3D ficou média2.10539/máximo59.7615, mas iso média11.3830/máximo308.937.
Assim o zipper melhora muito frente ao harmônico, enquanto a distorção média
do iso cresce (antes6.79458). XY continua limitado: zipper média372.87/máximo
25505.6,iso98.81/90769.5. Sem promover candidato por convergência isolada.

Não ampliei o budget64do kernel nem mudei fontes/projeção para fazer o estudo
passar. A próxima decisão deve considerar o conflito de quadros, as medidas
por região e a aparência; mais suavização/mais passos não resolvem tudo.

## Prova de autoria explícita do referencial da filha

O solver acima permanece igual. Uma segunda prova declara uma camada de
autoria ANTES do finisher, somente numa cópia: translação dos91valores materiais
retidos da filha[3151..3242], sem rotação. Offset declarado
[-.4706738831428572,2.841677142857143,.32152964428571423], obtido da média
das posições dos primeiros7v da fonte canônica original menos a média material
desse anel. Não há busca nearest nem alinhamento automático. A fonte original
119v/234t em disco não muda; a camada altera INTENCIONALMENTE o campo avaliado.
Translação preserva derivadas salvo f32 e muda a fase do ruído/sulcos, portanto
não alegar que a aparência/material original da filha permanece igual.

Repro no mesmo driver: adicionar `--material-range 3151 91 --material-offset
-0.4706738831428572 2.841677142857143 0.32152964428571423 --max-change 5`.
Input autorado é canonicalizado por put_asset/inspect na instância isolada,
antes de chamar preview_mesh_material_finish. Não escreve nas fontes lidas.
O driver verifica hashes dos inputs antes/depois e distingue o campo original,
a camada de autoria e o finisher. Saídas estáveis `artifacts/tree-material-frame/`;
a comparação sem offset em tree-material-finish continua disponível.

Mesmos budget64/tol.002/cap10000, envelope5explicitamente declarado: termina55,
resíduo f32 .00044289329, máximo de mudança no finisher2.89029458unidadesM.
Finisher preserva o prefixo M da entrada JÁ ALINHADA; os91M autorados diferem
do inputoriginal. O inputoriginal tem142guardas singulares; após a autoria e
antes do finisher128; depois0. Posições/triângulos/rig e pixels neutros exatos.

| Região3D | Original sem autoria | Depois de autoria+finisher |
|---|---|---|
| Guarda anisotropia média/máxima | null/null (142singulares) | 1.21496/1.35513 (0singulares) |
| Iso média/máxima | 6.79458/1626.695 | 2.26536/35.4598 |
| Zipper média/máxima | 5.80355/53.9745 | 1.86259/6.68583 |

Assim a troca de referencial explicitamente autorada melhora as duas regiões,
sem trocar solver/regularidade para fazer a prova passar. Não conclui aparência:
a captura ainda tem uma linha na casca. XY de guarda mantém1singular, zipper
tem média728.89/máximo49993, iso22.26/37721. Projeção dos detalhes e continuidade
discreta de gradiente continuam abertas; valores3D não certificam o shader.
Não existe ainda transform material persistido na receita ou alinhamento automático.
