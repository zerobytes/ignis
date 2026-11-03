# Base inferior da asa do nariz — EAF-187

Dois novos pontosXYZ, `nose_alar_base_L/R`, permitem subir/descer a base inferior
logo abaixo do controle da asa, com simetria opcional. Não substituem a asa nem
zeram os ajustes atuais. No painel: **Base da asa do nariz: E / D** seleciona o
lado; a seta verde altera a altura. Os pontos pertencem a Rosto.

Anchors7121L/360R na superfície frontal, abaixo dos7085L/323R da asa. Cada
campo usa18vértices do cage, raio7mm lateral,4,5mm para cima/5,5mm para baixo e
14mm na profundidade; máscara evita atravessar a linha central/interior do nariz.
Resposta normalizada no anchor Catmull-Clark. LimitesXYZ±4/5/6mm, defaults0.
Como nos demais pontos, limites individuais não garantem todas as combinações
sem interseção ou deformação artística indesejada.

Fonte6 separada **human-nose.ignis-character**,262canais/469alvos/67pontos
(36Rosto/21Lábios/10Orelhas),7warps/163ossos. Somente6canais/6alvos/2handles
são novos; campos existentes e fontes1..5 permanecem intactos. Zero deve
reproduzir o bake anterior bit-exato; nenhuma forma é aplicada automaticamente.
Nenhum dado externo adicional ou código MakeHuman novo.

## Capacidade e compatibilidade

A fonte5 já ocupava os256canais. Fonte6 permite512parâmetros; fonte1 conserva128
e fontes2..5 conservam256. Até80handles desde fonte5,64nas anteriores. O orçamento
96MiB de fontes3..6/64MiB de1/2 e os limites de geometria/rig permanecem.

Recipe2 admite512valores; Recipe1 continua legível e limitada a256. Cenas com
Recipe2 exigem Scene109; SetCharacterBinding sobe a versão junto da transação,
sem publicar receita incompatível. Replay119 acompanha a mudança. Scene1..108
com receitas antigas válidas continuam legíveis; escritores antigos recusam
os novos contratos. Conteúdo7/protocolo5/PSV1/checkpoint3 mantidos. Nenhuma
receita privada de personagens é enviada no pacote público de conteúdo.

Config/Scene/portátil preservam os novos valores e o hash exato da fonte;
undo/redo inclui versão, receita e bake. O limite de replayJSON8MiB da base
refinada permanece; replay sintético pequeno protege o contrato expandido.
Hunks botânicos de Scene108/replay118 foram preservados; novas constantes não
devem ser rebaixadas por integração paralela.

```sh
python3 scripts/character/nose_source.py
python3 scripts/character/nose_demo.py
```

Saídas/provas em artifacts/character-nose; fonte em character-base. Atualização
usa UpgradeCharacterSource existente, guardada por referência/identidade e
bloqueio de edição manual. Janela humana recebe os dois pontos em zero.

## Validação e entrega

Fonte73.340.401bytes, SHA
6c5c6a91c00609147c7ae22a0ced5a5bab8856b3fce6fc09e88f28447ca9582b.
13Rust/3alvos: character_base11, gesto nativo1 e portable_save1. Nova prova
crítica de capacidade/Recipe1→2/Scene109/atomicidade/undo/config/portátil/replay;
limites de fontes antigas mantidos. Clippyrelease/alltargets-Dwarnings, build,
fmt autoral e Python aprovados.4gruposMCP/6PNGs no build6f2a9f5c4f1ab8f5:
256valores antigos e bakebit-exato a zero; exemplosY±2mm, simétricos e unilateral;
12extremosXYZ, preview/erro atômicos, Scene/config/portátil/reabertura. Asa acima,
ponta do nariz/lábios/V/cantos/queixo/orelha sentinelas não moveram nos exemplos.
Revisão de frente/perfil confirmou a região inferior pretendida.

JanelaPID98241, mesmo build, pronta/pausada/FPSOFF com a base inferior direita
selecionada, simetria ligada, novosXYZzero e câmera próxima do nariz. Os256valores
humanos mantidos; geometria idêntica na atualização real. Scene109,configRecipe2
e layout com câmera do editor salvos em caminhos correntes; antes/recibos/PNGs
em artifacts/character-nose. CUA confirmou a nova janela. Arte/aceite humano
permanecem abertos; um target/release/ignis/app por symlink, sem binário extra.
Constantes109/119 e promoção SetCharacterBinding foram integradas no commit
compartilhado de botânica9a267e4 durante a validação; seu conteúdo foi preservado.
O commit de nariz contém os demais arquivos autorais, sem reintroduzir hunks191.
