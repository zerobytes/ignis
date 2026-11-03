# Continuidade material nas junções — proposta para a prova

Escopo coordenado após o colar69: medir e corrigir a primeira faixa material sem
alterar geometria, shader ou fontes autoradas. A consulta pura está sendo integrada
separadamente; este documento define os invariantes da próxima prova de kernel.
A persistência e o formato da receita só serão definidos após os resultados.

## Evidência e restrição adicional

A medição nativa confirma142/142faces da primeira guarda com posto1 em3D. Os
triângulos têm materiais [ma,mb,ma]/[mb,mb,ma]. A área geométrica aumenta com o
colar, enquanto a área no domínio permanece zero. Valor igual na guarda não é
continuidade de derivada. A suavização geométrica não criou esse colapso.

Os dados não usam um referencial material global comum: na entrada da mesma
junção há um ponto do pai em posição local[-.67774,-.93205,.27386] com material
[-.54462,2.63862,.43860], enquanto a filha tem posição[-.98776,-.63789,-.07219]
e material[.065,.40922,0]. Esses quadros são autorados e permanecem fixos nesta
prova. Não alinhar/deslocar coordenadas da fonte implicitamente. Isso limita o
quanto um miolo pode ter derivadas pequenas/isotrópicas: fronteiras incompatíveis
podem exigir distorção. Não prometer casca perfeita só removendo posto1 na guarda.

## Estágio e propriedade

O finisher recebe a GEOMETRIA FINAL, depois de todos os filtros iso/colar, e
modifica somente coordenadas materiais geradas. Posições, triângulos, materiais
retidos, rig/clipes e fontes originais ficam exatos. O colar moveu64guardas e
320pontos retidos; derivadas estimadas antes desse movimento não descrevem a
superfície final. A integração/persistência pertence à frente de plantas;
a biblioteca pura e os fixtures pertencem à frente de meshes.

Aplicação por estágio de enxerto e depois da junção principal. Um enxerto
posterior usa como fonte o resultado anterior; portanto a ordem de composição
continua autoral. Não é resolução global simultânea do domínio de toda a árvore.

## Entrada explícita proposta

- Geometry final com coordenadas materiais explícitas. None é recusado nesta
  operação; a ausência tem outro significado válido no renderer.
- Número de vértices e faces retidos (prefixos), intervalo iso e pares exatos
  (vértice de fonte,vértice de guarda). Esta proveniência vem da união atual,
  não de busca por proximidade nem de IDs persistidos de uma avaliação antiga.
- Referência afim explícita para os pontos gerados:3colunas de matriz e offset,
  com convenção declarada. É um prior de reconstrução, não campo físico inferido.
  O caso atual pode declarar identidade+material_offset da união. Uma referência
  afim permite provar identidade/rotação/escala/shear sem depender da valência
  irregular da triangulação. Campos constantes ou de posto menor são legítimos.
- Budget finito de até64iterações para correção interna. Nenhuma convergência
  implícita; reportar iterações realizadas e resíduo recalculado após f32.

Validar índices, duplicatas, cobertura e pertença das fontes/guardas/iso antes de
alocar o solver. Faces de fonte referenciam somente o prefixo retido. Pares têm
origem nesse prefixo e destino no domínio gerado da guarda; não misturar outros
enxertos por coincidência de índices. Vértices iso precisam de caminho para
Dirichlet fixo; recusar componente livre/sistema singular. Manter tetos existentes.

A receita futura guarda só opções. Os pares e intervalos atuais serão metadados
internos transitórios da avaliação da união. Uma consulta pura de Geometry+plano
explícito pode servir à prova, sem publicar cena ou prometer vínculo persistente.

## Derivada da fonte e extensão à guarda

Para uma face retida, e1=p1−p0,e2=p2−p0,n=e1×e2. Bases recíprocas:
b1=(e2×n)/|n|² e b2=(n×e1)/|n|². A derivada tangencial de material é
J=(m1−m0)⊗b1+(m2−m0)⊗b2. Sua derivada normal mínima é zero: o campo sobre a
superfície não determina um volume material fora dela.

Em cada vértice de fonte emparelhado, usar somente faces RETIDAS incidentes.
Estimar a derivada por média ponderada pela área; extrapolar até a posição final
da guarda: m_guard=m_source+J_médio·(p_guard−p_source). Preparar reciprócas uma
vez por face e acumular somente nas fontes necessárias, sem solver por vértice.
Recusar geometria numericamente mal condicionada com face/vertexID. Não inventar
uma segunda direção quando o material autorado já tem posto0/1.

A fronteira original mantém os mesmos valores, portanto C0 nela é preservado.
Faces curvas/derivadas divergentes recebem uma aproximação local explícita;
não prometer continuidade analítica C1/C2 de um campo inteiro nem fibra física.
A área/rank depois da quantização f32 precisa ser medida, não inferida do f64.

## Interior e alternativas

Depois de estabelecer os valores na guarda, resolver uma correção em relação à
referência afim declarada, com fontes/guardas fixas e desconhecidos somente iso.
Um CG Dirichlet finito é candidato reutilizável. Calcular o resíduo final e a
mudança das coordenadas, sem chamar execução de um budget de passos de solução convergida.
Zero RHS deve manter a referência exatamente dentro da quantização declarada.

Usar como prior o campo antigo inteiro e simplesmente difundir o delta pode
poluir um interior já afim: o campo antigo tem a guarda colapsada mas o interior
pode já coincidir com a referência. Por isso o prior precisa ser explícito e
coerente, não inferido a partir dessa descontinuidade de derivadas.

Resolver m diretamente por média de vizinhos é outra alternativa; numa malha
irregular, harmônico combinatorial não reproduz todo campo afim da geometria.
Uma discretização cotangente seria outra escolha, com condições/custos próprios.
Extrapolar apenas na união crua é mais simples, mas o colar posterior mudaria a
geometria; não adotar essa ordem. Alinhar os quadros das fontes pode ser uma
etapa futura de autoria, distinta de preservar esses campos nesta prova.

O legado CopyGuard continua exato e a nova correção será opt-in. Na integração,
um finisher novo deve substituir a correção material legada naquele estágio,
em vez de executar duas difusões e dar um relatório ambíguo. A geometria/seam
legadas continuam iguais. Não decidir esse encaixe persistido antes do kernel.

## Provas e limites de aceitação

1. Campo afim conhecido numa faixa planar, inclusive escala/shear/rotação e
   offset, com guarda inicialmente colapsada; medir taxas com a consulta nativa.
2. Fonte constante/posto1 não ganha uma direção inventada; zero RHS, ausência de
   canal, geometria mal condicionada, IDs duplicados/fora/cobertura falham ou se
   comportam conforme contrato, sem mutar a entrada.
3. Guardas medidas depois de uma deformação geométrica, fontes/posições/canais
   não materiais exatos; quadros diferentes não são corrigidos implicitamente.
4. Fixture real da união local final69: guardar antes/depois para fontes,
   guarda,zipper,iso,em3D eXY separados,com frequência explícita em outro relatório.
   Verificar se a correção só deslocou a distorção para o próximo anel.
5. Reportar posto/áreas/taxas/anisotropia,resíduo e máximos de alteração. Reduzir
  142colapsos não certifica todo o domínio sem dobras/sobreposição ou casca realista.
6. Legado exato e futura receita/MCP/undo/regeneração/replay são gates da
   integração; a biblioteca pura sozinha não conclui a entrega visual.

Checklist prévio proporcional:1.só autoria;2.buffers limitados/CSR uma vez;
3.f64 temporário e publicaçãof32,índices explícitos;4.gradientes por face uma vez;
5.até64iterações/tetos atuais;6.semGPU/I/O;7.semthreads/locks;8.proveniência atual,
semcachepersistido;9.pureza/fonteexata/recusas/legado;10.correção global por estágio,
quadros incompatíveis e resíduo finito mantidos como limites,semFPS.

Decisão da prova alinhada: prefixos/intervalo iso contíguos e pares completos;
referência afim explícita; budget1..64. O reparo regular recusa com faceID
quando o suporte material incidente da fonte já tem posto0/1. Isso é domínio
da operação opt-in, não invalidação desses campos no renderer/diagnóstico.

## Primeira prova do kernel — resultado ainda não promovido

A frente de meshes produziu um candidato nativo sobre a geometria local final69,
sem editar a árvore mantida: referênciaI+offset,64passos,tolerância.002,
envelope1unidade material,cap de anisotropia10000. Fonte/posições/topologia exatas;
render neutro idêntico. A primeira guarda passa142singulares3D→0,anisotropia
média1.215/máxima1.355;iso melhora média6.7946→6.4121 e máximo1626.695→207.799.
Porém zipper piora média5.8036→6.7336 e máximo53.9745→110.824. EmXY há outro
tradeoff: a guarda ainda tem1singular e o zipper regular fica muito anisotrópico.
Não promover à cena com base apenas na redução de colapsos da primeira faixa.
Artefatos da prova em native-meshes/artifacts/tree-material-finish; são candidatos,
não o estado da árvore mantida ou uma certificação de realismo.

O solver harmônico da correção impõe valores nas guardas, mas não controla a
mudança de derivada seguinte. Próxima alternativa a avaliar: minimizar ||Lδ||²
sobre todas as linhas do Laplaciano (incluindo pontos fixos), com incógnitas só
iso e fontes/guardas fixas. A matriz é L_U^T L_U e o RHS é −L_U^T L_F δ_F.
Esse objetivo pode penalizar a quebra no zipper sem inserir outro anel fixo;
condicionamento pior/overshoot exigem resíduo,envelope e qualidade explícitos.
Uma diagonal de precondicionamento é candidata, sem relaxar gates para caber.
Estender J a mais anéis por conectividade do zipper é outra opção, mas pode só
mover a banda para adiante. Ainda não há decisão de implementação/promovibilidade
para essa alternativa, nem novo formato persistido ou mudança de shader.

## Decisão de integração70 — antes do código

Adotar uma camada afim opcional e explícita sobre o domínio material da fonte
na avaliação: `Assembly.plant_material_frame` e
`ContextGraft.source_material_frame`. A transformação geométrica permanece
independente. A fonte canônica conserva seus valores originais; a aparência
avaliada muda intencionalmente e a remoção da opção desfaz essa autoria.
Não inferir alinhamento, rotação ou fibra por proximidade.

`material_finish` opcional em cada junção executa após os filtros geométricos
iso/colar. A união fornece proveniência transitória exata: prefixos retidos,
intervalo iso e pares fonte→guarda, sem persistir índices avaliados. Quando
habilitado, suprime a difusão material antiga nesse estágio; sem opção, mantém
o resultado legado exato. A referência afim e budgets são autoria explícita.
Versões70/replay74 incluem estas opções e a projeção triplanar coordenada;
nenhuma cena70 será promovida antes da integração completa. A projeção pode
reduzir compressão2D, mas não corrige um domínio3D degenerado.

Alternativas: reescrever os valores canônicos perde reversibilidade; alinhar por
nearest cria uma correspondência oculta; executar ambos os solvers duplica custo
e torna os relatórios ambíguos; alterar todo o material da árvore agora amplia
o escopo sem prova. A integração começa pela mesma junção e mantém o legado.

Checklist prévio:1.somente autoria;2.clone apenas da geometria transformada,
plano temporário limitado;3.afim f64→f32, canal explícito e finito;4.passe O(V)
para frame e um finisher por estágio, não por tick;5.mesmos tetos e CG<=64;
6.sem GPU/I/O neste recorte;7.dispatcher e staging existentes;8.undo/load e
regeneração incluem opções, sem cache novo;9.legado, fonte/rig exatos, afim
invertível ou singular explícito, erros atômicos, regeneração e replay;
10.validação síncrona e solves por estágio mantidos, sem promessa de C1 ou FPS.

A revisão encontrou crescimento do enum de edição ao guardar opções afins/solver
inline. Novos campos são `Option<Arc<...>>`: JSON permanece transparente, histórico
compartilha opções imutáveis e o enum volta a tamanho limitado. Evita Box clonado
a cada snapshot; custo é uma alocação/contagem Arc somente ao criar cada opção.
