# Paisagem representativa — EAF-183

Correção humana: terrain-walk182 valida física, não atende à paisagem natural.
Esta unidade pertence ao chat systems; vegetação/cena final e aceite visual são
coordenados com nanites. Base terrain.scene.json e placement.json em
artifacts/representative-world; final world.scene.json pertence a nanites.

## Decisões e revisão prévia

Campo global editável com cristas compactas na direção artística179 (mesmo perfil
sinuoso/dobras; extremidades com suporte suave para não cortar o mundo). Terrain2
introduz Ridge; leitura/re-encode1 bit-exatos e guardas por versão preservados.
Uma malha regional contínua2048m/4m compartilha normais/vértices e evita emendas
visíveis do recorte estático. Não é streaming: a futura partição precisa conservar
esse sampler e certificar normais/costuras. PublishTerrainRegion pelo dispatcher
usa Geometry nativo, sem enviar mesh por JSON. Scene/replay/render não mudam.

- Sampler preparado uma vez, seno/cosseno de orientação frios; sem JSON/IO/heap
  por ponto. Descartar contribuição fora do suporte antes de avaliar dobras.
- Região com até1048576v/64milhões de unidades de trabalho, preflight antes de
  alocar; quotas do Patch1 e formatos antigos não ampliadas silenciosamente.
- Fonte f64, bake f32 e índicesu32; Arc nativo, um shape/BVH físico por asset/escala.
  Buffers grandes são custo de geração/upload/residência, não de serialização/tick.
- Sem renderer/worker/lock novo. Água é representação visual; leito e margens
  vêm do campo físico. Nada de CSG, fluxo simulado ou mundo contínuo alegado.
- Gates novos somente para formatos/perda de autoria, continuidade global,
  atomicidade e integração física material. Imagem nativa e aceite humano separados.

## Contrato do kernel

`Operation::Ridge` é dado comum de autoria/procedural: centro Position global,
azimute, largura/profundidade/altura, fase, sinuosidade, expoente de encosta,
dobras e faixa de término suave. O miolo segue o perfil179; apenas as pontas
longitudinais recebem smoothstep de quinta ordem para retornar à base sem
terminação abrupta. Não é erosão/geologia física e não representa overhangs;
formas166 continuam complementares. Orientações são preparadas, diferenças
locais de células usam i128/f64, sem converter coordenada absoluta grande.

Ridge exige Terrain2; o novo leitor também aceitaTerrain1. Header e versão do
Document precisam concordar; versão1 com Ridge é recusada. Escrita conserva a
versão do documento, inclusive metadado de SaveTerrain e PublishTerrain. Arquivo
antigo é re-encodificado bit-exato; layout de posição48B e Landform1 preservados.
Scene104/replay114/checkpoint3/conteúdo5/protocolo5/PSV1 não mudam.

`QueryTerrain.inspect_region` retorna resumo compilado, sem JSON de todos os
vértices. `PublishTerrainRegion` cria asset/instância e vínculo físico numa única
transação, pausada, pelo dispatcher comum. Região: origem/células/espaçamento;
até1048576v,64M unidades estimadas e10000m por eixo. Contagem tem checked_mul e
preflight antes dos buffers grandes. Isso é orçamento dessa operação, não um
novo teto de scene/engine; Patch1 mantém256células/66049samples/16Mwork. O asset
regional usa a mesma Geometry/Arc e pipeline nativos que os demais meshes.

Não existe agora um novo chunk renderer: é uma região estática completa com
índices compartilhados, daí suas normais não têm emendas internas. Publicação
reconstrói a física pelo contrato autoral existente; não é deformação/streaming
em runtime sem reset. A câmera de inspeção do editor continua independente do
Scene, enquanto mudanças autorais de cena/luz refazem a simulação como antes.

## Composição e revisão visual

A primeira composição600/430/360m com profundidades estreitas foi rejeitada na
coordenação por parecer cercas de agulhas/costelas. Receita final tem três
cadeias380/260/220m, profundidades1000/800/800m, dobras20/16/12m e secundárias
mais arredondadas; canal inicia no vale, não corta a crista principal. A clareira
do player nivela à altura natural amostrada, sem depressão arbitrária. Toda raiz
na184 usa os triângulos já publicados, não o sampler analítico pré-bake.

Solo integral:2048×2048m,4m entre amostras,263169v/524288tri em uma malha física.
Origem do frame residente zero; Position global não equivale a ligar manifesto de
residência ou save a regiões. O controlador limita inclinação (40° na demo), não
há promessa de subir qualquer encosta. Source supports ground; água é plano
visualY0,05 estendido±27km, sem colisor/fluxo/natação; o mesmo plano aparece onde
o canal foi escavado a-2→-3m. É composição de ilha/canal/fiorde local, não uma
simulação de rios ou diversidade geológica mundial completa. Costa submersa evita
expor o quadrado da malha. Horizonte além do renderer30km não foi certificado.

Atmosphere.color alinhada a layers.sky_color, mantendo mountain_color separado.
Após avaliar árvores, density foi calibrada a0,00006 para2km;0,00025 ocultava
detalhe demais. Sem blur/shader novo. A184 pode escolher câmera inicial derivada
[100,220,800]→[0,130,-150],FOV60 e preserva jogador/colisão. A cena e vista finais
precisam de aceite humano; imagem técnica/caminhada automática não o substituem.

## Revisão de performance após implementação

- Sampler e trigonometria da orientação frios; contribuições fora do suporte
  retornam antes das dobras/pow. Nenhuma serde/I/O/alocação por ponto/tick.
- Region prepara uma vez e aloca buffers dimensionados;f64→f32/u32 no bake,
  Arc após validar; publicação não devolve centenas de milhares de vértices noRPC.
  Só posições/índices desta malha ocupam9449484B (~9,01MiB), excluindo capacidade,
  BVH, estados, GPU e demais assets. Não é medida de RAM total.
- Tudo residente: fonte visual CPU, BVH físico e buffers de render existem.
  Malha524288tri não entra como oclusor inteiro no recorte177 (até262144tri).
  Futuros LOD/partição precisam conservar normais/costuras e cobertura; não diminuir
  qualidade autoral para satisfazer aquele seletor. Frustum/shaders/pacotes/instancing
  continuam genéricos e renderer inalterado. Nenhum ganhoFPS alegado.
- Sem rede, worker, lock ou dependência nova. JSON só em configuração/inspeção;
  Shape/BVH e geometria não são recriados durante a caminhada. Recriação autoral
  em publicação/luz/câmera de Scene é custo frio existente, distinto de órbita
  de câmera do editor. Inspeção completa de Scene grande continua cara.
- Quotas e atomicidade recusam antes do commit; o incremento só acrescenta dois
  testes críticos (fonte/continuidade/formato e publicação grande), com regressão
  existente selecionada. Arte depende de captura nativa e avaliação humana.

## Evidências e reprodução

Build5c2e797b819983f2, release/aarch64/macOS.9Rust/4alvos: terrain_region2,
terrain3,landforms2,terrain_collision2. Clippy release/all-targets-Dwarnings,
fmt/Python/build/diff aprovados.4roteirosMCP/11grupos: terrain_region2,
terrain3,landforms3,terrain_collision3; regressões existentes isoladas. A composição
nativa acrescenta a prova de caminhada11,865m grounded, save/load da cena grande
e imagens play/panorama. Não é suíte completa; o agregador ainda tem os roteiros
gráficos históricos não registrados, conforme182.

Fontes: examples/representative-world.recipe.json e .ignis-terrain. Gerador:

```sh
python3 scripts/build_representative_terrain.py
```

Base/manifesto/PNGs em artifacts/representative-world. terrain.scene.json é meu
artefato; world.scene.json, vegetação e câmeras derivadas são184. Não regenerar
base enquanto184 implanta: sinalizar revisão/hash e renovar a derivação.
Base congelada após a calibração de densidade, geometria/player/placement iguais.
Logs: tests-final.log,clippy.log,build.log,generation.log,mcp-region.log,
terrain-verification.json e regression/report.json nessa pasta.

A cena17MiB usa o caminho local buffered. ReplayJSON e conteúdo de rede mantêm
envelope8MiB; replay foi provado numa região menor, não nesta paisagem completa.
Para passeio prolongado, selecionar execution_mode continuous viaMCP no tick0 pausado;
não confundir com opt-in de redraw contínuo da viewport. Portable/checkpoint e
multiplayer desta paisagem não foram certificados neste recorte. Nenhuma janela
humana foi aberta/fechada/focada pelo chat systems; abertura final pertence à184.

## Entrega integrada184 e estado da janela

Floresta derivada em bff7c4f:12mil árvores/24mil componentes e terreno/oceano,
4assets; raízes interpoladas nos triângulos publicados, erro após save/load
<0,00005m. Capturas scenic/walk/panorama e forest-verification.json no mesmo
diretório; propriedades e limiares em docs/representative-forest.md. O padrão
de bosques/encostas ainda é regular, há um modelo de árvore e água plana;
vegetação não possui colisores. Não declarar realismo/geodiversidade final.

O chat nanites abriu world.scene.json na GUI preservadaPID1262/buildb13af5530ec3621b
(182 anterior, compatível comScene104). Essa janela permite avaliar a imagem e a
mesma colisão de mesh, mas NÃO demonstra o novo kernel/API Ridge2 da183.
Geração/capturas/gates ocorreram em processoMCP próprio5c2e797b819983f2. Nenhuma
cópia de executável nem reinício da janela humana foi solicitado por este chat.
A GUI ficou pausada/em edição; não háMCP conectado nem seletor continuous no
menu atual. Não afirmar modo contínuo ativo ou nova instalação nessa janela.
O núcleo atual e seu executável estão disponíveis na main; atualização da janela
e integração humana de continuous são ações distintas. Aceite humano183/184
continua pendente.
