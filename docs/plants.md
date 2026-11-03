# Plantas paramétricas — perfil e extrusão

## Conexões entre duas plantas

A receita nativa `plants.connections` liga duas Entry por instâncias e seções
nomeadas. `put_connection`/`remove_connection` no mesmo edit_plant/preview_plant,
mais inspect_plant_connections, permitem regenerar ao mudar o desenho sem bake.
O primeiro recorte exige pares disjuntos e filho rígido preso a osso explícito
do pai. Fontes separadas, retidos e descendentes preservados; detalhes/skin do
miolo novo são gerados, não transferidos implicitamente. Contrato, alternativas,
limites e reprodução em [plant-connections.md](plant-connections.md).

Direção de interface: [plant-workspace.md](plant-workspace.md) propõe coleção,
prancheta cotada e resultado 3D sobre o mesmo documento paramétrico. A interface
ainda não foi implementada; os comandos descritos abaixo continuam sendo a
capacidade entregue.

`inspect_plant_sampling {id}` consulta diretamente a amostragem da Definition
corrente: geometry, material, surface, faces e relatório de trajetória, junto da
revisão. É a base construtiva sem escultura/assembly; os índices pertencem a essa
amostragem/revisão. Evita reconstruir cortes por proximidade. Estudo de junção
entre duas plantas: [live-plant-junction-study.md](live-plant-junction-study.md).

Ensaio de ampliação da autoria: [tree-secondary-plant.md](tree-secondary-plant.md)
converte explicitamente um segundo ramo da mesma árvore em planta com trajetória,
seções e vínculos de ponta. Não converte toda a árvore nem resolve a junção entre
duas plantas vivas.

Extensão EAF-97: [plant-trajectories.md](plant-trajectories.md) adiciona
trajectory opcional com curva cúbica e cortes perpendiculares à tangente.
Sem trajectory, o contrato legado abaixo permanece exato. A trajetória do
primeiro recorte é um gráficoXYZ comYlinear crescente, não um sweep livre
com laços. O cilindro isolado precede qualquer aplicação à árvore.

Refinamento posterior: [mesh-topology.md](mesh-topology.md) define subdivisão linear
uniforme declarada em `subdivision_levels`, preservando a fonte e a proveniência de
todos os novos vértices. Esse contrato estende a amostragem descrita abaixo.

Recorte autorizado em26/10/2026 após discussão de alternativas de preservação. Um documento construtivo separado da malha avaliada guarda um perfil retangular cotado, extrusão positiva e amostragem explícita. O perfil é no plano XY local; profundidade no eixoZ, com volume centrado na origem. Dimensões em metros e coordenadas materiais normalizadas[-0.5,0.5]. Seis faces identificadas por eixo/sinal; amostragem conserva orientação externa e correspondências por face/célula.

Escolha: reavaliar uma fonte canônica quando mudam dimensões ou tesselação. Escultura e rig não são inferidos por proximidade; correspondências explícitas transferem detalhe e pesos. Repouso acompanha pontos materiais, orientações globais dos ossos são conservadas e clipes são adaptados pelo delta local relativo ao repouso. Isso não preserva automaticamente contato, trajetória mundial ou anatomia. Modificar uma dimensão não reinterpreta a escultura como uma nova referência; rebase ocorre apenas por edição direta explícita.

Alternativas: snapshot apenas impede perda mas não acompanha alterações; reprojeção pode atingir o lado errado de uma parede; cage acrescenta deformação volumétrica/correspondência própria; solver CAD completo/BRep não é necessário para provar este contrato. Formatos de plantas e malhas são distintos, com transações/revisões/inspeção compartilhadas. Planta e visual não alteram colisão ou percepção.

A implementação integra o kernel de preservação descrito em mesh-preservation.md. API descrita abaixo; evidências automatizadas em validation.md (EAF-77). Sem reconstrução multivista, solver geral, escultura com nova topologia, IK/retargeting, associação automática a NPC ou editor visual completo.

## Contrato de autoria

`edit_plant` e `preview_plant` recebem `edit.kind: put`, com `definition`, `detail_policy` e `animation_translation_policy`. Políticas obrigatórias: `physical` conserva vetores no XYZ local; `proportional` aplica os fatores dimensionais. Não é transporte de normais em uma superfície arbitrária. `preview_plant` valida uma cópia e devolve plantas/assets sem mudar revisão, execução ou histórico. `inspect_plants` lista definições; `inspect_plant {id}` devolve fonte, referência, políticas e relatório. `SceneEdit::Plant` permite lote com operações de malha e um único undo.

```json
{"command":{"op":"edit_plant","edit":{"kind":"put","definition":{"id":"limb","profile":{"id":"outline","width":2,"height":4},"depth":0.5,"segments":[4,12,2],"sampling_tolerance":0.0009765625},"detail_policy":"physical","animation_translation_policy":"physical"}}}
```

Dimensões finitas0,01..100m; segmentos1..63 por eixo, contados antes da alocação e limitados a4096 vértices/8192 triângulos. Até16 plantas; fontes canônicas e malhas avaliadas respeitam separadamente os budgets globais de meshes. `sampling_tolerance` aceita0,000001..1m, padrão1/1024m. Perfis e assets têm IDs ASCII alfanuméricos,_,- de1..64 caracteres. A identidade do perfil permanece estável; trocar o ID exige desvinculação explícita.

Cada face tem vértices próprios; não há weld nas quinas. A correspondência usa a face/célula/triângulo/bari, sem ponto mais próximo. A reconstrução compara os vértices da fonte canônica contra a superfície reamostrada; erro acima da tolerância impede a edição. `sampled_reconstruction_error` é uma métrica amostrada, não Hausdorff contínuo, garantia de ausência global de interseções ou avaliação artística.

`sculpt`, `set_rig`, `put_clip` e `remove_clip` em malha vinculada estabelecem uma nova fonte canônica na configuração corrente. Essa é uma edição explícita: futuras regenerações partem dela. Alterações apenas de dimensões/tesselação/políticas não rebasam. `put_asset`, `grid` e remoção de asset vinculado são recusados; `edit_plant {edit:{kind:"detach",id:"limb"}}` conserva o asset e encerra o vínculo. Em lote, detach seguido de remoção da instância/asset é permitido.

## Persistência e isolamento

Cena49 guarda `plants.entries`: definição corrente, definição de referência, fonte canônica compartilhada por Arc e políticas. O asset avaliado permanece em `meshes`, alimentando o renderer existente. Ao validar/load, reconstruir e comparar com o avaliado impede cache adulterado; cenas1..48 migram, mas plantas não podem ser escondidas num schema antigo. Replay53 preserva o documento canônico e as poses; reprodução permanece restrita ao mesmo build. Edições seguem reset de execução preexistente.

Não há algoritmo novo no tick, solver dentro da física ou mudança de colisão/percepção. Editar, validar e inspecionar ainda usam o serviço síncrono. A validação reconstrói resultados de plantas limitadas e tem custo deliberado na autoria; não é cooking assíncrono. Render/cache/buffers atuais continuam responsáveis por apresentação.

## Verificação e limites

`python3 scripts/run_verifications.py --only plants` constrói perfil/extrusão, esculpe, instala três ossos/pesos e clipe pelo MCP. Testa mudanças não uniformes, retesselação, redução insuficiente, troca de identidade, stale revision, undo/redo, fonte sem drift, persistência/reset, detach/lote e pixels repetidos/replay/prefixo/viewport. A cena fonte é `examples/native-plants.scene.json`; os PNGs e relatório ficam em `artifacts/mcp-validation/artifacts/plants-*`.

Este incremento prova um domínio de construção e preservação; não é um sistema geral de desenhos coordenados, CAD completo ou escultura profissional. Falhas mantêm a versão anterior, sem transferir pesos/detalhes para outra região silenciosamente. Microdetalhe é o vetor entre a fonte esculpida e sua base, interpolado na superfície canônica. Orientações globais dos ossos são mantidas; cisalhamento e orientação anatômica não são inferidos. Clipes relativos ao repouso não garantem contatos/IK/retargeting ou trajetória mundial. Ferramentas de desenho/pincel/timeline na interface humana e aceite visual são gates posteriores.

## Perfis e seções — contrato do incremento da espada

A alternativa de reconstruir por proximidade perde identidade em lâminas finas;
um solver CAD geral e transporte de frames acrescentariam contratos grandes.
Este incremento usa um volume material cúbico explícito, mapeado para um loft
reto centrado em Y. `sections` opcional contém um ID de domínio, uma seção
transversal `rectangle` ou `diamond` e 2..16 estações `{id, at, width, depth}`.
`at` vai de0 a1 em ordem estrita; width/depth são frações positivas até1 das
dimensões globais. A silhueta frontal resulta das larguras e posições das
estações; as espessuras e a seção transversal definem os cortes. O losango tem
quatro lados: arestas laterais e nervuras frontal/traseira. As tampas fecham as
extremidades com área positiva; ponta de área zero é recusada neste recorte.

Cada intervalo entre IDs ocupa uma faixa igual do domínio longitudinal;
`segments[1]` subdivide CADA intervalo. X/Z subdividem a seção como antes.
Superfícies e coordenadas materiais independem da posição física. Faces continuam
separadas nas costuras; triangulação e subdivisão linear são determinísticas.
A geração é um loft triangulado, sem promessa de superfície CAD lisa.
Adicionar/remover/reordenar estações, trocar o ID do domínio ou o tipo transversal
exige desvinculação explícita; editar posições/dimensões conserva identidade.

Detalhe é a diferença para a base canônica: `physical` conserva o vetor XYZ em
metros, `proportional` aplica apenas a razão das dimensões globais, sem alinhá-lo
à normal. A medição na retesselação compara os vetores de detalhe reconstruídos
em TODOS os vértices canônicos, separando a base alterada intencionalmente do
relevo preservado. Não mede Hausdorff contínuo nem interseção global.

Origens globais dos ossos são invertidas no volume de referência e mapeadas no
novo volume, inclusive fora do eixo central. Ossos fora do volume são recusados
para lofts; não são presos silenciosamente à borda. Orientações globais são
mantidas: não há escolha implícita de twist/frame pela superfície. Reconstruir
translações locais contra os pais e readaptar chaves absolutas pelos deltas TR
relativos ao repouso preserva a autoria do movimento, sem garantir contatos/IK.
Pesos seguem a correspondência baricêntrica, com recusa acima de4 influências.
Regeneração não altera a fonte; esculpir/editar rig/clipe explicitamente rebasa.

O formato passa a cena51/replay55; retângulos anteriores permanecem compatíveis.
A espada de teste terá lâmina, guarda, cabo e pomo separados, uma gravação
esculpida e um rig de flexão diagnóstica. Não representa simulação de aço nem
um asset final de produção. A→B→A, erro atômico, retesselação, persistência,
undo/replay e capturas offscreen são os gates automatizados. Aceite humano é separado.

### Testar a espada

Cenas autocontidas: `examples/native-sword.scene.json` (desenho original) e
`examples/native-sword-redrawn.scene.json` (lâmina maior/larga e seções movidas).
Cada uma contém os quatro componentes, a fonte canônica, gravação, rig de3 ossos
e clipe `flex`. Ao abrir, P inicia a flexão diagnóstica; R reinicia. Os comandos
continuam disponíveis pelo MCP. Não há editor gráfico de plantas2D neste recorte.

```sh
./target/release/ignis --root . --scene examples/native-sword.scene.json
```

O exercício inteiro é reproduzível sem janela com:

```sh
python3 scripts/run_verifications.py --only plant_sections
```

`edit_plant` ou `preview_plant`, edit.kind=`put`, recebem a Definition completa,
`detail_policy: physical` e `animation_translation_policy: physical`. Por exemplo,
a alteração usa profile.width=0.82, profile.height=4, depth=0.18, shoulder.at=0.24
e point_start.at=0.72. IDs e demais campos permanecem os da Definition inspecionada.
Mover a instância da lâmina para Y=3.28 mantém seu calcanhar junto à guarda;
transform da instância é separado da construção. `inspect_plant` retorna definição,
referência, fonte e erro amostrado. `undo`/`redo` revertem as edições; `edit_scene`
permite alterar planta e posição da instância numa única transação.

O script verifica preview sem mutação, alteração e retorno exato da malha,
rig/clip/pesos, recusa de domínio/colapso/perda de detalhe, retesselação, undo,
save/load, animação e replay/prefixo com pixels iguais. Capturas correntes em
`artifacts/mcp-validation/artifacts/sword-{original,redrawn,flex}.png`.

## Adoção de trajetória na árvore

Contrato, alternativas, limites e revisão estática em
[tree-trajectory-integration.md](tree-trajectory-integration.md). Adoção explícita
conserva fonte/referência históricas; Scene65/replay69 locais. Draw131072, demais
tetos mantidos e consultáveis via inspect_meshes.limits. Junção regenerável
permanece uma etapa distinta; protótipos reprovados não fazem parte da API.


## Composição paramétrica com contexto — local66

Receita de3portas reavaliada de fontes imutáveis, sem usar saída como origem.
IDs de seção/contornos reais, raiz rígida explícita e aparência compatível;
metadados atualizam fontes e geometria direta recusa rebase. Preview de lote
compartilha staging do editor. Apenas asset8192v16384t; totais e draw mantidos.
Escolhas/revisão10pontos, custos e limitações: [plant-assemblies.md](plant-assemblies.md).
Scene66/replay70 locais; dados canônicos cortados ficam na fonte, sem falsa
transferência de escultura para miolo novo. Artgate e demais galhos continuam.
