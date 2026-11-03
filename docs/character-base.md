# Base humana feminina — EAF-187

Incremento atual: [papada, região orbital, nariz e encaixe ocular](character-face.md).

[Cadeia de pontos da mandíbula](character-jaw.md).

[Íris procedural e editor ocular](character-eyes.md).

[Base inferior da asa do nariz](character-nose.md).

[Vale central do arco do cupido](character-cupid.md).

[Malha suave, pontos e orelhas](character-smoothing.md).

Incremento anterior: [lábios, bases adultas e pivôs de cabeça/pescoço](character-refinement.md).
Fonte3 lê1/2; o contrato abaixo registra o marco anterior.

Ampliação posterior: [corpo, postura e manipuladores faciais3D](character-controls.md).
Fonte2 preserva leitura1 e o bake das configurações antigas;42pontos/155canais,
gordura/músculo até2 e cintura/postura próprias. O histórico abaixo descreve
o marco inicial; as capacidades/formatos atuais são os do contrato ampliado.

Direção de Guilherme: corpo feminino adulto realista, reutilizável e configurável,
incluindo ferramentas faltantes no Ignis. Primeiro marco: anatomia editável, formas,
repouso ajustado e rig articulável. Pele fotográfica, correções de pose e dinâmica
de tecidos têm critérios próprios; nenhum build aprova anatomia ou arte humana.

## Fonte e reprodução

`assets/characters/female-base` contém dados gráficos CC0 do MakeHuman, fixados no
commit `a8bc2d54ff0ac92e78ff71431b1023eda42bf482`, licença original e checksums em
`provenance.json`. Não há código MakeHuman/AGPL importado no motor ou no gerador.
A anatomia/topologia/alvos/rig são derivados dessa fonte, não escultura original
do Ignis. Esferas/capas oculares são geometria simples autorada neste projeto.
Dados e licenças upstream permanecem bit a bit, inclusive espaços e o sublinhado
`=======` do título da licença, que o `git diff --check` confunde com um marcador
de conflito. A checagem de estilo cobre os arquivos autorais; os dados originais
são conferidos pelos hashes, sem normalização que altere a procedência.

`python3 scripts/character/prepare_source.py` verifica os hashes locais e recompõe
`artifacts/character-base/female.ignis-character` sem rede. `--fetch` é uma operação
explícita de aquisição da revisão fixada; não acontece durante uso/runtime/build.
`default.json` é configuração legível, separada da fonte e da cena derivada.

Depois de editar a configuração, `python3 scripts/character/demo.py --base-only
--config assets/characters/female-base/default.json` publica e salva o bake atual,
com vistas nativas, usando o único `target/release/ignis`. Sem `--base-only`, o
roteiro também exercita variantes/poses e replay. A cena autocontida atual fica
em `artifacts/character-base/female.scene.json`; a fonte conserva os alvos e rig.

Fonte adulta fixa: três alvos regionais distribuídos igualmente para formar a
base desta demonstração, mais as combinações femininas adultas de músculo/gordura,
altura e volume/firmeza de forma dos seios. Não é taxonomia biológica, estimativa
de composição corporal em porcentagem ou suporte a idades infantis. Os controles
0..1 são coordenadas autorais; altura métrica é medida na geometria resultante.
Controles locais usam -1..1. Firmeza de forma não é rigidez dinâmica/bounce.

## Ferramentas nativas

`inspect_character_source {path}` lê uma fonte `.ignis-character` dentro da raiz
do projeto e retorna parâmetros, procedência e contagens, sem mudar o mundo.

`publish_character {path,configuration:{parameters:{...}},placement:{prefix,
position,rotation,scale}}` gera todos os assets/instâncias e publica pelo mesmo
`EditScene` transacional de MCP/editor. Um undo desfaz a publicação inteira;
parâmetro desconhecido/inválido ou malha/rig inválido não publica parcialmente.
Exige pausa. Omitir controles usa defaults; publicar sempre deriva da fonte, sem
acumular deformações sobre o resultado anterior. Mesma fonte/prefixo substitui
os mesmos IDs; trocar prefixo cria outra personagem. Nomes prefixo+parte precisam
caber no contrato de IDs existente. A física/gameplay não é criada implicitamente.
Edições manuais tornam o vínculo desatualizado e bloqueiam novos parâmetros.
`regenerate_character` substitui explicitamente o bake inteiro desses IDs,
inclusive clipes e edições manuais; undo recupera o documento anterior.
Para manter alterações autorais, preservar a fonte/configuração correspondente
ou outro prefixo. Transferir clipes
entre repousos diferentes usa o retarget explícito existente, sem reaplicar tracks
absolutas automaticamente em uma nova anatomia.

Fonte1 é JSON editável com posições canônicas, alvos esparsos ordenados, curvas
lineares por parâmetro, produto de fatores para combinações, partes e âncoras de
juntas. Helpers anatômicos participam do ajuste, sem serem desenhados. O compilador
recompõe eixos/posições globais e binds locais após a forma; não estica ossos de
animação para esconder um repouso incompatível. Inserts rígidos seguem sockets.

Limites da operação: 64MiB, 100mil vértices fonte/partes, 128 parâmetros, 512
alvos, 4milhões de deltas, 8fatores/alvo, 16pontos/curva e 32partes. Não são
limites globais do mundo nem certificação de combinações anatômicas arbitrárias.
O compilador valida finitude/índices/triângulos/eixos/rig; não detecta todas as
auto-interseções ou garante que todos os extremos combinados pareçam humanos.

## Painel de personagem no editor

O botão **Personagem**, na barra de espaços de trabalho, abre os controles em
Propriedades. A interface pertence ao editor: cada jogo implementa sua própria
tela de criação, consumindo a mesma fonte/compilador se desejar. A base feminina
oferece21 controles em Corpo, Tronco e quadril, Seios, Membros e Rosto. Sliders
aplicam a forma e ajustam o repouso do rig; cada arraste gera um único undo.
Os números são coeficientes da fonte, não medidas médicas ou percentuais reais
de gordura. Firmeza altera a forma estática; não implementa bounce.

Scene106 guarda `characters`: caminho/hash da fonte, configuração completa,
placement, IDs das partes e fingerprint do bake. Fonte/alvos permanecem em
arquivo separado; uma cena salva abre e renderiza sem esse arquivo. Carregar a
fonte é necessário apenas para editar parâmetros. Um snapshot validado fica em
cache de sessão, sem leitura/parse por frame ou por slider. A cache mantém uma
fonte por vez; alternar para outra base carrega sua fonte. Uma fonte substituída
no disco não muda silenciosamente a cache; recarregar exige o SHA salvo.

`publish_character` grava bake+receita numa só transação. Para cenas antigas,
`connect_character_source {path,configuration,placement}` anexa a receita apenas
se o bake existente corresponder exatamente: não altera geometria. Um prefixo
ocupado sem vínculo não pode ser sobrescrito por publicação. Novas proporções
não acumulam deslocamentos sobre o resultado anterior. Edição manual de malha,
material, clipe ou transformação é preservada e desabilita sliders; o painel
oferece regeneração explícita com explicação e undo. Undo/redo restaura receita
e malha juntas. Metadados adulterados não autorizam editar sem verificar o bake
canônico da fonte. A validação de Scene não exige I/O nem apaga vínculos quando
uma parte é removida; esse caso aparece como edição manual a resolver.

Paridade MCP/UI: `open_character_panel {open,prefix?}`,
`inspect_character_editor`, `load_character_editor_source {prefix}`,
`set_character_parameters {prefix,configuration,gesture?}` e
`regenerate_character {prefix}`. `configuration.parameters` parcial altera só
os controles informados. O token de gesto agrupa mudanças consecutivas em um
undo pelo mecanismo `EditSceneLive`, isolado pelo prefixo da personagem.

`save_character_configuration {prefix,path}` e
`load_character_configuration {path}` exportam/importam receita1 JSON em
`.ignis-character-config` até64KiB, com versão/SHA/placement explícitos.
Importação exige o mesmo vínculo/fonte/placement e base sincronizada; falhas
não alteram a cena nem o histórico. Salvar cena já guarda todos os parâmetros;
exportação é opcional. Caminhos são relativos à raiz do projeto, sem travessia.
Metadados autorais não entram no pacote visual público da rede. Replay116
acompanha Scene106; conteúdo6/protocolo5/PSV1/checkpoint3 permanecem iguais.

`python3 scripts/character/editor_panel.py` verifica o fluxo com a fonte feminina
real e atualiza a única cena demo corrente. `--capture` acrescenta captura do
editor e precisa ser coordenado com outras medições GPU. Testes críticos
protegem corrupção por descompasso receita/bake, undo de arraste, save sem fonte,
importação inválida e preservação de alterações manuais; não há teste por slider.

Validação deste incremento:24 testes Rust em7 alvos (`character_base`, `editor`,
`editor_mesh`, `editor_authoring`, `editor_transform`, `portable_save`,
`npc_checkpoint`), Clippy release/alltargets com-Dwarnings, build release,
fmt/Python/diff aprovados. Quatro grupos MCP com fonte feminina real passaram
no build `5e0c59f86cc89bfc`:21 descritores, gesto em um undo/redo, rejeição atômica/
exportação/importação, Scene/save portátil/replay preservando receita e bake.
Logs e recibos em `artifacts/character-panel`. CUA confirmou clique nativo no
slider: gordura0,5→0,735 mudou a silhueta e o fingerprint; um undo restaurou a
base. A automação de drag não entregou movimento de ponteiro; não é prova de
arraste físico nem aceite humano. Coalescência do gesto foi validada por MCP e
Rust. Captura do compositor e janela final com painel foram inspecionadas; editor
final aberto pausado com medidor desligado, sem acrescentar UI ao gameplay.
Fonte/base e layout preservados na troca do executável; rascunhos descartados,
único executável corrente em `target/release/ignis` (o app usa symlink).

## Extensão de rig e formatos

Até256 ossos, índicesu8 preservados; nenhuma truncagem acima de255. Quatro canais
continuam inline em `Weight`, com `extra` opcional para até16 influências totais.
Pesos adicionais entram no skinning, transferência/subdivisão/modelagem e guardas
de rigidez; exceder16 é erro explícito, sem poda. Fixtures de rejeição acima de4
passam a conferir preservação; o limite16 tem prova de rejeição própria.

Scene105 admite esse domínio ampliado e recusa escondê-lo em Scene104 ou anterior.
O domínio ampliado de rig continua disponível desde Scene105; autoria paramétrica
usa Scene106/replay116. Conteúdo visual6/protocolo5/PSV1/checkpoint3 não mudam.
Conteúdo antigo/incompatível é recusado pela versão. Bakes em Scene são autocontidos;
remover o arquivo fonte não remove a personagem, mas impede regenerar a morfologia.
Config/fonte não são parâmetros animáveis ou estado privado de um NPC.

## Revisão estática de desempenho

Decisões prévias: formas compiladas fora do tick, dados compartilhados por Arc,
curvas/deltas esparsos e binds ajustados em uma passagem por junta. Não adicionar
consulta anatômica, hash da fonte, parsing, I/O ou geração por vértice no frame.
Publicação faz validação fria antes de substituir a cena; mesma política de undo
existente, sem histórico novo de cópias de fonte ou executáveis.

Pesos overflow alocam somente onde necessários (`Option<Box<[Influence]>>`);
quatro canais legados não alocam overflow. Cada transferência agrega até256 somas
na pilha e recusa excesso. A fonte/partes são clonadas na compilação/publicação,
não continuamente. Decodificação é limitada em bytes; validação e geração ainda
podem alocar proporcionalmente ao conteúdo, até os budgets documentados.

Matrizes persistentes de pose/skin são dimensionadas pelo rig, evitando reservar
256 ossos para cada rig antigo. As rotinas existentes de IK/retarget/transferência
ainda usam scratch limitado a256 na pilha. Skinning segue na CPU, custo proporcional
a vértices e influências, sem ganho de FPS/GPU ou capacidade de multidões alegados.
Malha estática/pausada continua reutilizando o frame existente.

## Aceitação e continuação

Conferir frente/lado/costas/rosto/mãos/pés e silhuetas distintas, depois flexões
de ombro/cotovelo/quadril/joelho, punho/mão e face. Registrar distorções em vez de
aprovar por contagem de ossos. O rosto e os membros existem na fonte; expressões
convincentes exigem controles e correções próprias. O rig oferece articulação;
não prova contato físico, anatomia clínica ou deformação correta em toda pose.

Movimento secundário de seios/nádegas/barriga, correctives por pose/forma,
material de pele avançado, roupas adaptáveis e integração como avatar de Player
continuam capacidades próprias a implementar. Câmera186 não oferece essa integração
automaticamente. A base não encerra o sistema completo solicitado.

O orçamento existente de16clipes/4096chaves agregados no documento permanece.
O roteiro reutiliza um clipe corrente por parte, em vez de acumular clipes de
cada pose de inspeção. Isso não demonstra centenas de famílias corporais distintas
animadas na mesma cena. Inserts rígidos ainda carregam o mesmo rig para seguir
as poses; usar attachments e compartilhar/organizar clipes é uma melhoria futura
antes de escalar esse conteúdo. Não inferir escalabilidade do criador a partir
da capacidade de criar muitas configurações offline.

## Validação e revisão visual nativa

Build conjunto `f36849ada878e54e`, capturas1200×1200 em
`artifacts/character-base`,7partes/15.788vértices/31.396triângulos (pele13.380v/
26.756tri),163ossos,21controles/142alvos. Configuração inválida não mudou o
documento;30ticks de animação retornaram replay matched e PNG idêntico. Save
portátil também restaurou a pose, os pesos estendidos e a mesma imagem. Evidência
em `verification.json`, `female.replay.json` e `female.ignis-portable` nesse diretório.
112testes Rust únicos/15alvos, Clippyalltargets com-Dwarnings e buildrelease
passaram. As fontes autorais formatadas passaram rustfmt/check e os dados CC0
mantiveram seus hashes. Conferidas frente/lado/costas,
três-quartos, rosto, pés, variações esbelta/atlética/maior volume e flexões.
As capturas da mão foram enquadradas pelo centro real dos vértices influenciados
por punho/metacarpos/dedos; flexão das falanges foi inspecionada. Isso não aprova
oposição do polegar, fechamento completo da mão ou contato de preensão.

A anatomia básica está presente e as variações mudam a silhueta. Ainda há facetas
perceptíveis na cabeça/orelhas/pés e ombros/axilas precisam de correctives ao elevar
os braços. A flexão de quadril/joelhos é uma pose suspensa de inspeção, sem IK de
apoio ou agachamento físico. O primeiro ensaio de mandíbula usou +18° em X global,
fechando além do repouso, e produziu deformação inadequada nos cantos da boca.
Com -18° a abertura básica funciona; a deformação do ensaio invertido não era
prova de falha do rig. Expressões coordenadas, cavidade
oral/dentes/língua finais e correctives continuam critérios próprios;163ossos não
aprovam automaticamente um rosto animado. Olhos têm volumes/capas geométricos
simples, sem córnea/material ocular avançado. Material de pele atual é uniforme.

Arte e aceite humano continuam separados desses gates. O único executável
corrente é `target/release/ignis`; nenhuma janela do usuário foi aberta, fechada
ou substituída por esta frente. A fonte/cena atual está em
`artifacts/character-base`, com configuração e recibo em `publication.json`.
Cópias temporárias da pesquisa foram removidas (3.589.861bytes); os147arquivos
CC0 canônicos (~15MB), hashes e licenças permanecem versionados. Não foram
produzidos bundles/instaladores ou cópias históricas de executáveis.
