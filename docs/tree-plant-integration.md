# Ensaio de uma planta curva na árvore compartilhada

## Avaliação anterior ao incremento

A árvore atual mantém o primeiro fork/root/líder conectado, enquanto outros
ramos são componentes sobrepostos. O próximo ensaio retira somente o sweep do
segundo ramo primário (limb1) e publica uma planta curva vinculada separada.
Usar preview_plant apenas para conferir geometria/custos; publicar por edit_plant,
preservando definition/reference/source. Embutir somente o resultado no asset
wood descartaria a editabilidade que este teste pretende avaliar.

Os planos do loft são XZ paralelos; seu perímetro derivado do quadrado material
não coincide por ordem/posição com círculos uniformes do kernel de junção.
Não soldar por proximidade. O encaixe desse ramo continua sobreposto e deve ser
julgado explicitamente como tal. Primeira junção, base e líder permanecem iguais.
Madeira estática evita presumir attachments ainda ausentes.

Fonte: os mesmos controles Bézier do ramo da seed20261026. Cada estação usa
at=(Y-Yinicial)/altura, não o parâmetro t da Bézier. Incluir os seis pontos de
inserção dos ramos secundários como estações; mover apenas uma estação de
controle entre inserções permite um teste de reautoria com posições protegidas.
As seções elípticas não reproduzem exatamente os antigos cortes perpendiculares
à tangente: é mudança explícita de representação, não preservação dessa malha.

Custos previstos: uma instância e um asset extras, dos29/7 atuais. Remover120v/
236t do sweep antigo; escolher segmentos/estações pelo preview antes de publicar,
pois1090v/2176t acrescentados ultrapassariam65536t de desenho. Manter4096v por
asset,16384v/32768t armazenados e32instâncias. Não subir limites para o teste.

Checklist anterior: geração/contabilidade apenas na autoria; fonte Arc e
validação existentes; nenhum trabalho de tick/renderer/threads novo. Inspeções
e capturas finitas em paths correntes. Instância estática, mesmos clipes de
folhagem. Custos de source+avaliado, validação e expansão/upload existentes
permanecem. Sem alegar FPS ou realismo a partir das contagens.

Gates: commit estável56/60 e regressão integrada; preview sem mutação; fonte e
material preservados em A-B-A; inserções protegidas; primeiro componente exato;
outros assets/instâncias intactos; undo/redo/save/load/replay/pixels; captura
próxima do ramo antes/depois com câmera/luz fixas. Não substituir o teste
artístico por sucesso desses checks.

## Resultado na árvore

Núcleo EAF-87 incorporado em9527034, build e3d1731ed06f70ab, cena56/replay60.
`demo_tree_authoring.py --compare-branch --frames 48` gera primeiro o baseline
com o sweep anterior, depois a árvore com uma planta vinculada. A receita fica
em Scene.plants, com definition/reference/source; o helper não triangula esse
ramo e não embute seu preview no asset wood. A autoria explícita do canal de
material estabelece a fonte uma vez, antes do ensaio de preservação.

Ramo258v/512t,8estações, segmentos[4,2,4]. Árvore8assets/30instâncias,
10948v/15802t armazenados,64346t desenhados; custo líquido138v/276t e uma
instância/asset. Cabe nos limites existentes. Fonte5.358.019bytes. O componente
inicial1839v/3674t (base, primeira bifurcação e líder) permaneceu exatamente
igual no mesmo canal de inspeção; os demais assets e instâncias também.

Mover só o controle branch1-015 em[0.18,-0.10]m altera a malha e mantém os
seis centros de inserção, com erro máximo1.174e-7m. Essa é preservação geométrica
de estações congeladas, não um sistema de attachments. Preview puro, fonte,
shading/material/coordenadas, pose da folhagem no tick de edição, A-B-A repetido
e undo/redo passam. Save/load compara meshes inteiro; execução/replay produzem
PNGs idênticos. O snapshot de comparação usa inspect em ambos os lados:
save_scene serializa f32 compacto e structuredContent os amplia para f64;
comparar textos numéricos desses dois canais dava falso negativo. Nenhuma
tolerância foi introduzida no teste de igualdade do componente.

Capturas correntes: tree-study/artifacts/branch-before.png (árvore com sweep),
after.png (planta vinculada), editable-branch-A.png e editable-branch-B.png
(mesma câmera/luz/tempo). GIF48frames distintos/4000ms/2.481.820bytes.
736testes Rust/66grupos,Clippy/build/fmt e67cenários MCP/276grupos aprovados
na integração. Logs artifacts/curved-tree-*.log e relatório mcp-validation.

## Fricções que o ensaio acrescentou

1. at físico não é t da curva. O autor ainda converte coordenadas e escolhe
   estações/resolução. Consulta de domínio/frame/custo composto deveria tornar
   essa correspondência observável antes de publicar.
2. A edição B exagerada mostra quinas: interpolação linear e planos paralelos
   são limites explícitos desta ferramenta, não defeito oculto de normais.
   Curva suave/frame transportado precisam preservar domínio e fonte, antes
   de espalhar sua implementação em vários helpers.
3. Os centros congelados não acompanham mudanças arbitrárias do pai. Um vínculo
   por parâmetro/frame deve atualizar secundários e folhagem na autoria e na
   animação; hoje mover esses pontos os descolaria. Madeira continua estática.
4. A planta curva e o primeiro patch convivem no mesmo documento/asset comum,
   mas seus contornos são distintos. Precisa haver correspondência de fronteira
   explícita para editar conexões; fechar componentes separados não os funde.
5. A inspeção visual ainda reprova realismo: superfície da casca parece rede,
   há encaixes sobrepostos, folhas planas e copa em conjuntos reconhecíveis.
   Preservar um material não demonstra que ele é artisticamente adequado.

Próximo teste proposto: alterar a curva de um ramo com secundários vinculados,
conferir inserções/frame em repouso e no vento, e julgar a transição no close.
Depois, distribuir folhas por domínio com seed/densidade/máscaras e visualizar
contagens/vazios/colisões. São capacidades genéricas sobre o contrato comum;
não um gerador exclusivo de árvores. Essas propostas continuam abertas.

## Checklist posterior

Adequado: frequência só de autoria, previews finitos e mesma validação nativa;
dados/IDs de planta existentes, contagens obtidas antes de publicar instâncias.
Alocações/cópias mantidas com motivo: snapshots JSON completos no ensaio e
baseline finito permitem igualdade exata; não entram no tick. Repetição limitada
aos ciclos A-B e baseline explícito. GPU/I/O somente em capturas solicitadas,
mesmo renderer e paths correntes. Sem lock/thread/dependência nova. Caches,
undo/reset/load/replay exercitados. Núcleo imutável durante a regressão.
Custos de avaliação/preservação por fonte, skinning/upload e sombra continuam;
nenhum ganho de FPS ou suporte em outra plataforma alegado.

## Suavização do mesmo ramo — avaliação antes do incremento

Integrar8aa1a11,57/61, sem alterar renderer/contrato de asset. Comparar os quatro
casos Linear-A/PCHIP-A/Linear-B/PCHIP-B com as mesmas estações, câmera, luz,
tick e segmentos[4,2,4]. Depois repetir a comparação em[4,6,4], inclusive A;
curvaC1 e amostragem da malha são variáveis distintas. A é forma final e B
continua diagnóstico exagerado. PCHIP altera tangentes dos intervalos vizinhos:
centros congelados não significam orientação preservada ou controle local.

Fonte canônica deve continuar a do ramo linear inicial, com canal material
autorado uma vez; troca de interpolação/resolução não rebaseia. Manter preview
puro, fonte/material, A-B-A, undo/redo,6centros, demais assets/instâncias e pose
foliar exatos, primeiro componente conectado idêntico, save/load/replay/pixels.
Publicar a receita PCHIP-A densa pela API nativa vinculada; nada de bake.

Preflight:706v/1408t no ramo denso, +448v/896t sobre258v/512t; previsão de
11396v/16698t armazenados e65242t desenho, sob65536. Captura deve confirmar
custo real antes de declarar. Mesmos4096v/asset,16assets/32instâncias e fonte
Arc. Checklist anterior: cálculo/coeficientes só em autoria, snapshots JSON
finitos para comparação, casos/resoluções em laços limitados; sem tick/thread/
lock/cache/dependência/GPU nova. Capturas em paths correntes. Regressão completa
do núcleo integrado e inspeção artística separadas. Mais tesselação conserva
custos existentes de geração/validação/upload; sem alegar FPS ou realismo.

O ensaio dedicado `demo_tree_smoothing.py` carregará a cena já salva, em vez
de reconstruir os outros assets. Assim a fonte anterior do ramo é a evidência
de preservação: comparar source/reference exatos antes/depois de trocar modo
e resolução. Todos os outros assets, inclusive wood inteiro, serão idênticos.
O script de construção original continua disponível como autoria inicial;
este ensaio é uma edição incremental da mesma cena.

## Resultado da suavização incremental

Núcleo8aa1a11 incorporado como761c77b,57/61, build6758029880829ea2. O ensaio
`python3 scripts/demo_tree_smoothing.py --frames 48 --segments-y 4` carrega a
árvore atual e edita só branch-editable-1. A prova desta unidade começou no
arquivo56 de48aa4d4, SHA256c16d4e18f1bb96a125214adee04b64799dbb822b870ed3e7d8f9b856adf9ce69,
byte a byte igual à fonte daquele commit, não numa reconstrução com modo novo.
`--input artifacts/tree-study/examples/smoothing-input.scene.json` repete a
mesma entrada preservada no root isolado; um checkout novo pode usar a cena
corrente, cuja referência/fonte continuam lineares originais.

Foram12casos com câmera/luz/tick/estações fixos: Linear/PCHIP × A/B × Y2/4/6.
Cada preview é puro, e sua saída coincide com a edição. Source E reference
permanecem exatos; o teste conserva inclusive wood inteiro, outros assets,
instâncias e pose foliar no tick de edição. Modo/forma repetidos em dois ciclos
e undo/redo retornam assets exatos. Coordenadas de material/triângulos são
iguais nos quatro casos de cada resolução. Os6centros têm erro máximo6.59e-8m;
isso não afirma tangente/orientação ou attachments preservados.

| Resolução do ramo | Vértices/triângulos | Desenho da árvore | Margem até65536 |
|---|---|---|---|
| [4,2,4] |258/512|64346|1190|
| [4,4,4] |482/960|64794|742|
| [4,6,4] |706/1408|65242|294|

Escolha visual desta unidade: PCHIP-A com[4,4,4], porque o close de A já mostra
contorno suave nesta escala; o ganho de6 não justifica gastar mais448t aqui.
É julgamento artístico localizado, não métrica universal de resolução. B forte
com6 deixa o arredondamento mais claro, mas é diagnóstico. Linear denso mantém
o cotovelo; PCHIP com2 ainda revela segmentos. Nenhuma mudança de teto ou LOD
automático. Fonte final mantém referenceLinear[4,2,4] e targetPCHIP[4,4,4].

Árvore final8assets/30instâncias,11172v/16250t armazenados,64794t desenhados;
fonte5.443.474bytes, igual ao saveMCP. Save/load compara meshes E plants
completos; execução/replay gera PNGs idênticos. Uma instância foliar adicional
é recusada especificamente pelo orçamento de desenho, com inspect inalterado.
GIF48frames distintos/4s/2.481.539bytes; animação continua só na folhagem.
Capturas comparativas em tree-study/artifacts/tree-smooth-{modo}-{forma}-{resolução}.png,
além das vistas correntes after/back/structure/root/fork/bark/leaf e report.json.

Julgamento: a suavização reduz a quina autorada; casca em rede, folhas planas,
copa repetitiva e encaixe sobreposto continuam visíveis. A árvore inteira não
foi soldada nem recebeu rig de madeira. Vínculos, frames transportados, scatter,
detalhe de folhas e LOD seguem pendentes. Esta entrega resolve o modo suave de
seções dentro do domínio declarado, não todas as dificuldades listadas.

Checklist posterior: adequado, coeficientes compilados só na autoria pelo núcleo;
o script acrescenta apenas consultas/edições/capturas finitas. Mantido com motivo,
snapshots completos e12casos para provar preservação histórica; nada no tick.
IDs/tipos/fonte/canais existentes, nenhum cálculo de mesh do ramo em Python.
Custos previstos e capturados iguais, resolução escolhida explicitamente; sem
cache/GPU/lock/thread/dependência nova. Undo/load/replay e recusa atômica exercitam
invalidação/contratos. Mais amostras conservam custo de geração/validação/upload
existente. Sem ganho de FPS nem nova plataforma alegados.

Validação integrada concluída:742Rust/66grupos,Clippy-Dwarnings/build/fmt/diff
e68cenários MCP/281grupos aprovados no mesmo build6758029880829ea2. Logs
artifacts/smooth-tree-*.log e relatório mcp-validation/report.json. Fonte/GIF
atuais substituem a versão anterior nos mesmos paths. Nenhuma janela controlada.

Antes de outra conexão estrutural: a primeira junção ocupa tronco até0.42 e o
segundo ramo nasce em0.44. Uma nova junção independente pode interagir com a
região anterior. Comparar fronteiras compartilhadas/múltiplos contornos e grafo
de autoria antes de escolher contrato; não somar fusões por sobreposição nem
inferir compatibilidade por proximidade. Esse desenho continua proposto.
