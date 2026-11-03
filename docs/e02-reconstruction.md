# E02 — reconstrução pelo procedimento de recortes

## Fonte, escopo e leitura global anterior à construção

Imagem E02.png fornecida pelo usuário, 1402×1122, SHA256
4153a456f9fdf2e9e6f43328171692156db7563c2062654199e66c0c540882ad.
Fonte original preservada no projeto Unreal. Modelagem a partir desta imagem,
skill/contratos e schema MCP; sem consultar receita/cena/análise da casa anterior
ou implementação Rust. Contexto anterior existe; não é teste cego independente.
Recortes por Pillow do runtime local são extração de pixels, sem geração ou
super-resolução; publicação, captura e persistência da cena somente pelo MCP.

Casa de linguagem gótica/orgânica com corpo alto à direita, ala longa mais baixa
à esquerda, galeria de janelas altas sobre varanda e pátio aberto elevado.
O corpo alto tem cobertura muito assimétrica e uma grande janela frontal;
na lateral direita aparecem janelas menores e uma água-furtada no telhado.
O pátio tem balaustrada curva vazada, abertura central e degraus curvos.
Pedra clara envelhecida, telhas esverdeadas, madeira/ornamentos quentes e plantas
são observados; espécie de pedra, dimensões reais e organização interna são
desconhecidos. A casa anterior não determina a planta desta referência.

## Inventário e fichas aprofundadas

Coordenadas em pixels, origem superior esquerda. Recortes em
artifacts/house-e02/reference/, caixas e hash em evidence.json. As regiões
foram abertas diretamente; a prancha reduzida é apenas apoio de navegação.

| ID / recorte | Afirmações globais e leitura local | Consequência construtiva / limite |
|---|---|---|
| F01 mass | Corpo direito mais alto; ala esquerda e pátio aberto. A imagem não mostra interior/traseira. | Hipótese de volumes em L com pátio dianteiro; explicitar eixos/faces e preservar separação entre pátio e casa. |
| F02 main-roof | Pano principal amplo inclinado à direita, cumeeira recuando na imagem, pontas erguidas e beiral recurvado. O pico próximo está deslocado à esquerda da janela principal. | Cobertura assimétrica, curvas e espessura, inclinação longitudinal; não usar frontão simétrico genérico. Profundidade/verso hipotéticos. |
| F03 left-roofs | Dois níveis de cobertura: pano superior atrás e cobertura baixa da varanda à frente. O pano baixo curva no extremo esquerdo; uma faixa de janelas fica entre os níveis. | Duas superfícies distintas com beirais/arestas independentes; conferir que a galeria não seja escondida pelo telhado baixo. |
| F04 porch | Vãos abertos com arcos apontados e apoios perfilados. Sob eles há bancos, vasos e aberturas na parede posterior. | Espaço aberto entre linha de colunas e parede; não substituir arcos por triângulos preenchidos. Quantidade exata de apoios ocultos é hipótese. |
| F05 entry | Porta dupla central-direita sob cobertura baixa; arco da porta é diferente dos arcos entre apoios. Há cerca de três pequenos degraus semicirculares próprios, além da escada do pátio. | Hipótese inicial: mesma face da ala esquerda. Revisada na construção para recesso próprio voltado ao pátio; face exata continua hipótese. Escadinha própria e lanternas laterais. |
| F06 great-window | Janela grande em arco, moldura de pedra múltipla, montante central grosso e ramificação clara; vidro revela vasos/prateleiras. | Vão real com vidro recuado, múltiplas molduras e árvore hierárquica. Reproduzir relações, sem presumir transmissão física correta. |
| F07 side-windows | Pelo menos duas janelas estreitas visíveis na face lateral, separadas por pilares e peitoris, sob beiral baixo. | Janelas na face +X do corpo direito com normais distintas da grande janela frontal. Total oculto desconhecido. |
| F08 upper-gallery | Cinco vãos pequenos distinguíveis na faixa principal, outro parcial no extremo esquerdo; cada um tem arco e padrão interno. | Repetição paramétrica com montantes; manter faixa visível entre telhados, sem tratar como andar habitável comprovado. |
| F09 courtyard | Piso amplo sem teto, lajes irregulares, faixa verde sinuosa e medalhão radial. Balaustrada curva deixa entrada central. | Piso elevado em setor oval, acesso desobstruído e inlay sobre o piso; atrás do pátio há circulação para as portas. |
| F10 rail-left | Base e travessa contínuas, arcos vazados menores, árvore clara em cada painel; pilares maiores recebem campos verdes e vasos. | Capas/base curvas e painéis em planos tangentes; vazios de verdade. Não copiar proporcionalmente o motivo da janela grande. |
| F11 rail-right | Retorno curvo curto com dois/três painéis aparentes; pilar de entrada com base escalonada e vaso. | Entrada entre dois pilares destacados, retorno com orientação variável; contagem atrás de folhagem permanece incerta. |
| F12 dormer | Pequeno volume sai do pano principal; janela apontada frontal com árvore simples e telhadinho próprio, não uma janela plana pintada no pano. | Água-furtada com profundidade e cobertura própria; encaixe oculto é hipótese. |
| F13 plants | Arbustos altos estreitos ao lado da casa, flores brancas/roxas e trepadeiras concentradas em bases/apoios; não cobertura uniforme. | Grupos distintos com volume e distribuição localizada; espécies botânicas desconhecidas. |
| F14 tree-detail (filha F06) | Tronco central largo, primeiro par aberto curvo e pares superiores menores; junções ligam à moldura. Redes finas ao fundo são separadas do tronco claro. | Dois níveis de espessura e ramos curvos; profundidade por camadas. Pontas minúsculas não são contáveis com segurança. |
| F15 column-detail (filha F04) | Bloco/plinto grande embaixo, degraus de moldura, fuste fino facetado, capitel e arcos bifurcando acima; cintas quentes. | Planta nativa por seções para perfil repetido, complementada por arcos separados. Ordem clássica específica desconhecida. |
| F16 entry-tree (filha F05) | Porta dupla tem linha central, tronco/pares ramificados gravados ou aplicados, tábuas verticais e terminação apontada. | Motivo de madeira mais fino que o da janela; desenho bilateral não exige copiar todas as ramificações. |
| F17 rail-tree (filha F10) | Um tronco com pares curvos domina um painel aberto baixo; ramos terminam no arco, com pequenos triângulos verdes acima. | Motivo baixo dedicado, arco de moldura e folha de fundo como peças distintas. Recorte de 65×105 limita terminações finas. |
| F18 floor-medallion (filha F09) | Medalhão oval na projeção, campo verde com aro duplo e raios/folhas em torno do centro; faixa verde continua sinuosa. | Círculo no plano do piso como hipótese, raios e faixa separados; projeção elíptica não determina elipse real. |

A leitura local acrescentou segunda porta à esquerda, bancos, degraus próprios
da entrada, faixa de galeria, duas camadas de rede na janela e variações do
motivo conforme suporte. Recortes F14–F18 respondem dúvidas concretas dos pais;
encerrar a subdivisão onde as pontas deixam de ser legíveis, preservando dúvida.
Análise de F01/F08 sobre número de pisos continua hipótese, não contagem certa.

## Planta de trabalho e parâmetros

Unidade de trabalho arbitrada em metros, sem alegar escala recuperada. X cresce
para a direita da fachada; Y para cima; Z para o pátio/frente. Na hipótese
inicial, a ala esquerda se estende em X, com parede frontal +Z e varanda à
frente; as duas portas e janelas baixas pertenciam a essa parede. A galeria
aparecia na mesma orientação acima. Esta hipótese foi revista abaixo.
Corpo alto fica em +X; janela grande na face +Z e janelas menores na face +X.
Pátio ocupa Z positivo; balaustrada envolve sua borda com acesso frontal central.
Anexos/coberturas têm superfícies próprias. Profundidade dos corpos, verso,
interior e encontro invisível das coberturas são hipóteses de fechamento visual.

Parâmetros a avaliar: razões entre corpo alto e ala, deslocamento lateral da
cumeeira, curvatura/altura dos beirais, altura visível da galeria, linha de
apoios, posição das portas na parede, tamanho da janela grande e raio do pátio.
Câmera separada da geometria. Primeira etapa deve conferir silhueta, faces e
oclusões; depois vãos, motivos, materiais e vegetação.

## Revisão estática anterior

1. Cliente de autoria/captura explícita, nenhum tick novo. 2–3. Geometria e
índices limitados pelos budgets do schema/inspect; buffers no cliente, sem
dependência Rust. 4. Perfis, curvas e motivos compartilham funções/fontes.
5. Preflight agregado antes de publicar, sem ampliar limites. 6. Recortes e
capturas em caminhos correntes, leitura por etapa; sem downloads/modelos.
7. Transporte sequencial em processo próprio headless, sem janela/lock novo.
8. Esperar revisão, validar lote antes de mutar e usar undo/save/load existentes.
9. Sintaxe, autoria, comparação, repetição, persistência e replay conforme uso.
10. Custos mantidos: matemática no cliente, câmera manual e interpretação da IA;
nenhuma reivindicação de FPS ou reconstrução certificada das faces ocultas.

## Revisão da hipótese durante a construção

A primeira passagem mostrou a entrada escondida pelo avanço do corpo alto e
arcos encobertos pelo beiral. Uma leitura posterior dos apoios mostrou outra
contradição: a sequência projetada da varanda não tinha a mesma orientação da
face da grande janela. Ajustar só a câmera não resolveria essas relações.

As posições aproximadas dos pés dos apoios (171,725), (293,732), (430,716),
(574,678) foram examinadas com raios de uma câmera pinhole assumida, intersectados
com piso Y=0.6. Isso orientou uma transformação da ala no cliente: eixo local X
aproximadamente (0.728,0,-0.779), normal do pórtico (0.731,0,0.683). Não é solver
de câmera ou recuperação métrica; câmera e altura de piso são hipóteses.

Na variante2, a ala/varanda ficam oblíquas ao corpo alto; a porta central tem
recesso próprio. Corpo alto foi recuado; abertura da balaustrada, raio do piso,
altura dos painéis e medalhão foram revistos pelos recortes. A faixa de galeria
foi reposicionada. Cumeeira e beirais ganharam variação longitudinal; as paredes
do frontão passaram a seguir o pano curvo para evitar atravessar o telhado.
Encaixes e profundidades continuam aproximados, especialmente na aba esquerda.

## Fontes publicadas e orçamento

Receita independente [reconstruct_e02.py](../scripts/reconstruct_e02.py), cliente
stdio com biblioteca padrão do Python; nenhuma fonte Rust ou receita anterior
foi consultada para modelar. Toda autoria/publicação passa por engine_command.
Coluna é uma planta nativa por9estações, fonte80v/68t, cinco instâncias. Galeria
compartilha uma fonte em seis instâncias. Demais formas são meshes diretos
calculados no cliente; padrões de aparência e juntas usam materiais nativos.
Os ornamentos incluem tiras planas com vazios reais: não há prova de volume
esculpido/manifold ou união física nas junções. IDs das fichas mapeiam assets;
várias peças podem pertencer ao mesmo asset, sem IDs individuais de vértices.

Cena [house-e02.scene.json](../examples/house-e02.scene.json), exportada pelo MCP,
Scene67/replay71 no build9860a09115365ad3. Resultado:16assets,25instâncias,
**16218v/18512t armazenados**,166v globais livres. Tetos originais preservados.
Uma primeira expansão chegaria a35874v/49755t; foi descartada pela contagem no
cliente antes da publicação. Compartilhamento da galeria, coluna por seções, deduplicação de posições
coincidentes e tesselação por escala viabilizaram o recorte. Não houve aumento
de limite, mudança de kernel ou ganho de FPS alegado.

## Comparação e diagnóstico visual

18pares foram recortados/abertos nas mesmas coordenadas dos pixels originais;
5sub-recortes F14–F18 aprofundam características de seus pais. Métricas nativas
em lotes16+2, sem máscara, resize, registro automático ou normalização por peça.
Diagnósticos por região em
[e02-review.json](references/image-to-3d/e02-review.json); artefatos correntes em
artifacts/house-e02/index.html, report.json e evidence.json. A página oferece
pares, sobreposição, mapa RGB, forma neutra, duas vistas auxiliares e a hipótese
anterior. As vistas adicionais verificam o objeto construído, não a referência
ausente de outros ângulos. Algumas ROIs ainda pegam parte diferente do objeto:
isso é registrado como erro de correspondência, sem mover o recorte para ocultá-lo.

Hipótese paralela (variante1) e hipótese revisada (variante2) foram recapturadas
com mesma câmera [13.4,15,20.6] → [0,3.25,0], FOV29°, viewport1402×1122, tick0,
mesma luz e políticas de materiais. As mudanças foram geométricas e de disposição;
o ruído seeded segue ordem de geração e não é textura calibrada da referência.

| Condição | MAE RGB global | SSIM luminância global |
|---|---:|---:|
| hipótese paralela | 0.179949 | 0.391281 |
| hipótese revisada | 0.179525 | 0.376478 |

**O resultado ainda está distante da referência.** O MAE global mudou pouco e
o SSIM global piorou. Grande janela, janelas laterais e água-furtada têm melhor
localização em alguns recortes; outras regiões pioram ou discordam entre
métricas. Não há evidência de melhora global, porcentagem de fidelidade ou
aceite artístico. Telhas, perfis, árvores ornamentais, juntas, transparência,
interior, plantas e encontros de coberturas permanecem simplificados. F17 é
inconclusiva como teste isolado do motivo por deslocamento do painel na ROI.

## Verificação técnica e limites

Captura repetida, alteração de aparência seguida de undo, save/load, replay
**estático de zero ticks**, retorno da câmera auxiliar e carga em processo novo
produziram PNGs com igualdade exata. Replay matched:true; IDs de assets das18fichas
resolvem. Não é teste de gameplay, colisão, teclado, fechamento ou reconstrução
do verso. Processos próprios headless encerrados; demo do usuário preservada.
Sem novo build, instalação, download de modelo ou regressão Rust geral, pois
nenhum código da engine foi alterado. Verificações do cliente/artefatos em
docs/validation.md e artifacts/house-e02/verification.json.

Uma falha de integração foi corrigida no cliente: a revisão histórica retornada
por capture_replay não pode substituir a revisão viva usada no próximo comando.
O conflito de revisão impediu a mutação; o cliente passou a distinguir esses
retornos. Nenhuma alteração no servidor foi necessária.

A inspeção automatizada da página file:// foi bloqueada pela política do
navegador. Sintaxe, dados e recursos locais são validados estaticamente; não
declarar teste de interação/render da página. As imagens nativas foram abertas
diretamente e as comparações locais revisadas. Pedido de abertura pelo painel
retornou queued; não comprova que a aba já ficou visível para o usuário.

## Reprodução

Primeiro extrair as evidências com Python que tenha Pillow (neste Mac, o runtime
local possui12.3.0). A referência versionada permite reproduzir sem o projeto
Unreal. O script usa a fonte original quando disponível e verifica o mesmo hash.

```sh
python3 scripts/e02_evidence.py
python3 scripts/reconstruct_e02.py --variant 1
cp artifacts/house-e02/render.png artifacts/house-e02/baseline.png
cp artifacts/house-e02/comparison.json artifacts/house-e02/baseline-comparison.json
cp artifacts/house-e02/capture.json artifacts/house-e02/baseline-capture.json
python3 scripts/reconstruct_e02.py --variant 2 --check
python3 scripts/e02_evidence.py --report
```

Usar --binary /caminho/ignis se o binário não estiver no destino corrente desta
worktree. O processo consulta/salva o schema público do build em uso. Nenhuma
imagem anterior/modelador externo entra na construção. O ensaio depende de
interpretação e matemática do cliente: a skill organizou evidências e erros,
mas não entregou um reconstrutor geométrico automático independente da IA.

## Respostas posteriores ao checklist

1.Sem tick/frame novo.2–3.Arrays/índices limitados no cliente; posições finitas
e serialização estrita.4.Fontes compartilhadas para coluna/galeria, funções para
perfis/motivos e weld por asset; sem estabilidade de índices presumida.
5.Preflight e contagens nativas acima, sem ampliar budgets.6.Recortes/PNGs só
sob comando em caminhos correntes; um build existente, sem cópias históricas.
7.Transporte sequencial/processos próprios, sem lock/thread na engine.
8.Revisão viva distinta da histórica; undo/load/replay preservam pixels.
9.MCP e integridade dos artefatos validados; limites de fidelidade/browser acima.
10.Mantidos: alocações/cópias/serialização no cliente, tesselação finita, fragmentos
planos nos motivos, câmera manual, superfícies ocultas hipotéticas e teto de
assets saturado. Sem promessa de performance ou geometria de produção.
