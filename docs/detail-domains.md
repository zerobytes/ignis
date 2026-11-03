# Domínios repetidos e seção dos traços

Decisão anterior ao código — EAF-80/EAF-85. A mesma árvore integrada70 mostrou
três problemas separados nas ablações: ruído anisotrópico25:1 parece madeira
polida; traços longos uniformes parecem fitas; celular forte parece escamas.
Não escolher um deles apenas pela quantidade de detalhe. Referência de casca
rugosa/fissurada com idade: Forestry England, English oak
(https://www.forestryengland.uk/tree-species/english-oak). A crista no encontro
de ramos é uma estrutura real, não obrigação de apagar toda marca: NC State,
General Pruning Techniques (https://content.ces.ncsu.edu/general-pruning-techniques).
A árvore atual é estudo de folhosa, não reconstrução certificada de Quercus.

Fricção genérica observada: Detail usa uma banda plana; bump muda só nos bordos.
32segmentos longos cobrem grande domínio mas não descrevem muitos detalhes
curtos. Alternativas: aumentar o teto multiplica custo por pixel; texturas e UVs
impõem recursos/samplers/gestão novos; novo solver de fibra não resolve a seção
do traço. Escolha incremental: seção rounded opt-in e domínio periódico 2D
explícito, conservando representação, limites e os mesmos32testes por plano.
São úteis para costuras, nervuras, ranhuras e padrões de superfície em geral.

Detail.profile = band (default omitido) ou rounded. Rounded usa
1-smoothstep(0,radius,distance), com derivada radial zero no centro e no limite.
Bump assinado continua em metros e independente da tinta; não desloca geometria.
A filtragem mantém o fade por footprint; não é integração analítica de pixels.

Detail.repeat: Option<[period_x,period_y]>, origem0 no plano de cada projeção.
Períodos finitos .001..10000. Suporte de cada ponta ±2raios deve ficar contido
na célula[0,period]; descartar patches que cruzam a fronteira, não cortar traços
silenciosamente. A restrição conserva faixa vazia nas bordas até para banda AA.
Não resolve curvas que atravessam células, unwrap cilíndrico ou continuidade de
fibra de forquilhas. É repetição explícita; repetição visível é risco artístico.
Footprint/derivadas vêm de q NÃO embrulhado, evitando linha no wrap. Coordenadas
negativas usam fract(q/period)*period. Mesma regra nos três planos triplanares.

GPU ganha um vec4 detail_domain (períodosXY, profileZ, reservadoW):1136→1152 B
por slot, tabela17slots19312→19584 B (+272 B). Vertex64 B/Globals176 B mantidos.
None+band executa aritmética anterior. Identidade/igualdade/material cache já
cobrem os campos novos. Novo formato Scene71/replay75; campos habilitados em
cena≤70 recusados inclusive fontes ocultas antes de avaliar receitas. Contratos
anteriores triplanar70/frames70 permanecem. MCP/preview/undo/load/replay primeiro.

Checklist prévio proporcional:1.só shader detalhado/autoria, zero tick novo;
2.opções pequenas inline, Detail continuaArc, nenhum clone por vértice;
3.enums/dimensões/controles finitos com limites;4.um wrap por plano e mesma
união32segmentos, não9células vizinhas;5.tetos mantidos e patches sem cruzamento;
6.+16 B/slot, nenhum novo recurso/textura/vertex/readback;7.sem worker/lock/deps;
8.invalidar por Material existente, validar toggle/load/reset/undo;
9.legado pixel-idêntico, perfil/periodicidade/negativos/bordas/subpixel e casos
inválidos em CPU+GPU/MCP;10.repetição e aproximações AA/bump mantidas explicitamente,
sem promessa de FPS ou realismo concluído. Testar tudo na mesma árvore e preservar
fontes geométricas, rig/clipes/instâncias; mudar receitas de aparência é intencional.


## Prova final e dificuldades restantes

Implementado em Scene71/replay75. O cenário MCP detail_domains cobre publicação,
previews puros, canais independentes, toggle, histórico, periodicidade positiva/
negativa, triplanar, distância, resize, persistência e replay; gate também nas
fontes ocultas. Layout1152 B/slot/19584 B e offsets112/128 testados. Rounded
fornece seção radial C1; não promete C1 entre segmentos com raios/tangentes
diferentes nem no máximo entre traços sobrepostos.

Translação de coordenadas por períodos binários não é garantia de PNG idêntico:
interpolação f32 pode arredondar. Prova mediu9pixels/640000 diferentes, máximo
1nível8-bit, média absoluta.000003515625por canal. Teste usa tolerância≤1 para
essa propriedade; repetição do mesmo frame, undo, load e replay são byte-exatos.
PNG é decodificado sem dependência extra por scripts/png_pixels.py só para medida.

A primeira receita tinha fissuras finas demais, taper abrupto e fases similares:
parecia cortes em faixas. Foi rejeitada. A versão corrente usa comprimentos e
inícios variados, seção côncava com menor amplitude e celular discreto no padrão
3D; oito trajetórias e ramos curtos somam os mesmos32segmentos. São parâmetros
autorais do ensaio, não valores botânicos certificados ou novo gerador de árvore.
Mudanças de aparência não equivalem a preservação visual da receita anterior.

Reprodução: gerar frames pela demo_tree_material_frames.py, opcionalmente a prova
study_tree_projection.py, depois
`python3 scripts/study_tree_bark.py --input artifacts/tree-projection/examples/triplanar.scene.json`.
Saída estável artifacts/tree-realism; input explícito70 é cópia da árvore integrada,
com fontes/câmeras/tick/luz preservados. Resultado71 inclui antes/depois, árvore
inteira, base/encaixe/forquilha e ablações cor/bump. O script preserva posições,
topologia, fontes geométricas, rig/clipes/instâncias e4PNGs neutros exatos.

Realismo permanece aberto. Repetição de patch é perceptível em alguns locais;
Métricas materiais e escala física diferem nas deformações/taper, e a combinação
de charts não define fibra natural de forquilhas. Materiais por região/mascaramento,
escala física controlada dos detalhes e camadas independentes de aparência são
próximas capacidades genéricas a avaliar. Folhas, distribuição de galhos e
junções geométricas restantes precisam de testes próprios. Não esconder esses
limites com câmera favorável, mais ruído ou contagens de testes.
