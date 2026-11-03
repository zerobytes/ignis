## EAF-213 — spatial wind and free aerodynamic meshes

[Model, numerical contract, timings and validation evidence](wind.md#validation-evidence-2026-11-03).
Eight focused wind MCP groups plus laboratory/render regressions run headlessly;
replay/native continuation exact in the tested build, portable primaries exact
with the measured cold-contact continuation difference documented. CPU samples
are workload measurements, not FPS claims. Human visual/native-input acceptance
is pending; current executable and existing editor are kept separate.

## EAF-187 — base inferior das asas nasais e capacidade autoral

[Contrato](character-nose.md). Fonte6 human-nose, SHA
6c5c6a91c00609147c7ae22a0ced5a5bab8856b3fce6fc09e88f28447ca9582b,
73.340.401bytes/262canais/469alvos/67XYZ. Par inferior ancorado360R/7121L,
18vértices/campo;XYZzero preserva o bake anterior exatamente. Botões E/D,
altura pela seta verde e simetria opcional. Fonte6/Recipe2 até512canais,
legadas128/256 preservadas; Recipe2 exigeScene109/replay119, publicação sobe
versão atomicamente. Rede/conteúdo7/PSV1/checkpoint3/envelopes de bytes mantidos.

Build6f2a9f5c4f1ab8f5/macOSarm64/Metal;13Rust/3alvos (character_base11,gizmo1,
portable1), Clippyrelease/alltargets-Dwarnings/build/fmt/Python/diff aprovados.
Nova prova crítica cobre formatos/limites/atomicidade/undo/config/portátil/replay;
4MCP/6PNGs:256valores/bakezero exatos,Y±2mm nos dois lados e unilateral,
12extremosXYZ, preview/erro, Scene/config/portátil/reabertura. Sentinelas na asa
acima/lábios/V/cantos/ponta do nariz/queixo/orelha permaneceram invariantes.

EditorPID98241 pronto/pausado/FPSOFF, Rosto/baseinferiorD selecionada/zero,
mesmo build e câmera própria do editor próxima do nariz. Scene/config/layout
salvos; antes/verification.json/native-verification.json/editor.png e vistas
em artifacts/character-nose.256valores humanos e bake mantidos na atualização
real. CUA confirmou a janela; aceite artístico humano separado. Um binário
corrente por symlink.191 integrou os hunks globais109/119/SetCharacterBinding em
9a267e4 durante esta validação; demais alterações191/181 preservadas.

## EAF-187 — vale central do arco do cupido

[Contrato e evidências](character-cupid.md). Fonte5 separada human-cupid,
SHA b35eba1c9db2ec58bc5606d0c8797d3ac97e6b49a515b5821619cfd0a28ac7ba,
73.333.717bytes/256canais/463alvos/65XYZ (21labiais). Novo campo local de38vértices
no vale central, preservando alvos anteriores e bake bit-exato a zero; default0.
Atalho de seleção e dica de eixos no painel. Fontes5/legadas até80/64handles,
parâmetros256 e demais envelopes/formatos preservados. Sem dados externos novos.

11Rust/2alvos (character_base10+gesto nativo1), nova prova crítica de formato/
capacidade; Clippyrelease/alltargets-Dwarnings/build/fmt/Python/diff aprovados.
4gruposMCP/6PNGs no build3b5b69864dae4031:253valores/bakezero idênticos, isolamento
regional, limites/preview/simetria/assimetria/stale, undo, config/Scene/portátil e
reabertura. Erro de distância7,62e-8m; efeitos em picos≤0,108mm e centro do
vermelhão≤0,325mm nos exemplos, sentinelas inferiores/cantos/nariz/queixo/orelha
invariantes. Medição inicial comparava com frame inferido do rig, que inclui a
forma craniana superior;18micrômetros/0,2grau de direção não eram erro da distância
do campo. Oráculo corrigido para distância+sentido, sem alterar código/limites.

EditorPID96153 pronto/pausado/FPSOFF, build conjunto587d93bed378d69b, fonte5,
Lábios/V selecionado/zero; última criação preservada (253valores, inclusive6novos
ajustes durante o trabalho), bake exato a zero confirmado de novo. Cena/config/
layout salvos, recibos e provas em artifacts/character-cupid. CUA testou botão e
enquadramento na janela real, deixada próxima dos lábios. Arte humana permanece
aberta; replayJSON8MiB da base refinada continua limitado conforme incremento
anterior. Um target/release/ignis e app por symlink.181/191 e seus diffs preservados.

## EAF-187 — subdivisão suave, pontos anatômicos e orelhas

Fonte4 `human-smooth.ignis-character`, SHA
b9136c8fe2a9dcc570baa42c258c96f73aa1edc4545370a97dd35479d13d4450,
73.327.859bytes.253canais/460alvos/64pontos (34Rosto/20Lábios/10Orelhas),
7warps/163ossos. Catmull-Clark no cage poligonal: corpo53.514v/107.024tri;
55.922v/111.664tri com olhos, até14influências, sem poda.22alvosCC0 de orelhas
acrescentados;240hashes conferidos. Fontes1/2/3 preservadas. Sem novo formato
Scene/replay/conteúdo/rede neste incremento; working tree conjunto de botânica
já em Scene108/replay118, conteúdo7/protocolo5/PSV1/checkpoint3 preservados.

10Rust/2alvos: character_base9 e gesto nativo1. Uma prova nova crítica cobre
cage/canais, cache mutado, limites de influências, upgrade protegido, atomicidade,
undo/redo, JSON e replay sintético. Clippyrelease/alltargets-Dwarnings, build,
fmt autoral, Python e diff aprovados.5gruposMCP/11PNGs no build9abbbec15019d093:
223valores humanos preservados,12formas/24extremos,10XYZ de orelha com simetria,
anchors na superfície refinada, grupos visíveis, pivôs/olhos conservados,
undo/redo e save portátil. Rest pose erro máximo2,67e-7m; distâncias faciais
nos giros compostos6,07e-8m. Relevo artificial retirado; definição geométrica
permanece. Mais resolução suaviza contornos, sem inventar detalhe anatômico.

**Limite confirmado:**replay JSON da base completa ultrapassa8MiB; cena de prova
9.495.501bytes e portátil10.414.458bytes. Exportação de replay recusou sem mutar
ou escrever parcialmente. Envelope preservado; não declarar replay completo da
base refinada. JSON de cena e portátil são os caminhos duráveis verificados.

EditorPID89450 aberto/pronto/pausado/FPSOFF no build conjunto8d3f357564f8d817,
223valores anteriores mantidos (comparação numérica f32), câmera próxima do rosto,
Todos ativo/lábio selecionado,64pontos disponíveis. Cena corrente salva em
character-base; fonte anterior preservada em user-before-smoothing.scene.json e
config/layout/verification.json/native-verification.json/editor-all.png em
artifacts/character-smoothing. Captura da própria UI revisada. CUA não conseguiu
reconectar ao app após a troca; interação física não é declarada como aceite.
Um target/release/ignis/app por symlink, sem cópia de binário. Botânica dirty
preservada e excluída do commit autoral;181/186 preservadas. [Contrato](character-smoothing.md).

## EAF-187 — lábios, base masculina e pivôs rígidos

Build1b8af57259aedd38/macOS arm64/Metal.9Rust/2alvos (character_base8 e1lib de
gesto nativo), com2provas novas críticas: Surface/coords persistentes e atômicos,
e transporte completo de socket ocular em giros compostos, legado e máscaras
inválidas. Checkalltargets, Clippyrelease/alltargets-Dwarnings, fmt/Python/diff
aprovados. Fonte3 lê1/2; Scene107/replay117/conteúdo7/protocolo5/PSV1/checkpoint3
preservados. Fonte93c26f858de02ef66405b973e119d5a835bdc061cd5066407bd08d2f8305ae53,
72.868.880bytes,223canais/420alvos/58pontos/7warps/163ossos,218arquivosCC0
conferidos. Fonte3até96MiB e legadas64MiB; contratos de geometria mantidos.

7gruposMCP/28capturas nativas passaram: legado2→3 com bake exato; pivôs ±40° e
composição conservam crânio, olhos e cabeça já estilizada; formas antigas ativas,
inferior/altura nos extremos; preset com os valores humanos;14formas labiais/
28extremos e16pontos novos/32extremosXYZ; bases adultas e relevo; undo/portátil/
replay completos. Distâncias craniofaciais máximas variaram8,64e-8m. A prova extra
dos olhos encontrou0,43mm de erro no primeiro build pela orientação derivada de
Yglobal; transporte do twist anterior corrigiu a falha sem relaxar o limite.

Revisão visual rejeitou a primeira transição rígida que incluía influências
pequenas da cabeça nas clavículas. Máscara atual acompanha mandíbula/nuca e
mistura a junção cervical; altura cresce a partir da base inferior do crânio.
Fibras longas em coordenadas globais foram substituídas por um patch local
reutilizado em regiões musculares, com amplitude menor; ainda são aproximação
procedural e não acabamento muscular fotográfico. Arte, misturas extremas,
correctives/expressões/tecidos/bounce e Playeravatar continuam separados.

Cena/configuração humana salvas antes da troca;155valores anteriores preservados
exatamente, sem zerar as formas cranianas desejadas. Fontes1/2 mantidas para
undo; nova cena aponta human.ignis-character. EditorPID81777 aberto, pronto,
pausado/FPSOFF; grupos Rosto/Lábios, novos parâmetros e seletor masculino visíveis.
Recibos verification.json/native-verification.json/editor*.png e configuração
user-current.ignis-character-config em artifacts/character-refinement; fonte e
cena correntes em character-base. Um target/release/ignis com app por symlink;
rascunho de lançamento removido, sem push. [Contrato](character-refinement.md).

## EAF-187 — corpo ampliado, postura e rosto3D

Build conjunto8527ea70431041ea/macOS arm64/Metal; Scene107/replay117/conteúdo7/
protocolo5/PSV1/checkpoint3 preservados.29Rust únicos/9alvos: character_base6,
editor8,editor_authoring2,editor_mesh4,editor_transform4,mesh_objects2,
npc_checkpoint1,portable_save1 e1lib de gesto nativo. Três provas novas protegem
formato/rig, atomicidade/limites/simetria/referência antiga e cancelamento/HiDPI/
um undo ao soltar. Check/Clippyalltargets-Dwarnings do build conjunto, fmt,
Python e diff autoral aprovados; dadosCC0 mantidos bit a bit e160hashes verificados.

Fonte2 finalbaf9b861dd1df3ed9769567ae987ec7505bc283d74bb12e22dada9d3c65f91aa,
46.073.103bytes,155canais/306alvos/42pontos/4warps/163ossos, reproduzida offline.
Nove gruposMCP com a fonte real passaram, incluindo84extremosXYZ, preview sem
alterar revisão/binding e commit equivalente, olho/órbita juntos, cintura sem
mover costelas, postura sem mover pés, undo/redo e save portátil/replay com
valores novos não zero. Receita legada21 reconstruída em processo isolado:
conexão à fonte2 mantém o bake exato.15capturas nativas de extremos/postura/face
revisadas; gordura máxima tem3,359× o estimador de volume geométrico anterior,
sem equivalência a percentual médico. Hipertrofia/definição ainda pedem arte.

Revisão da UI identificou e corrigiu centros baixos de lábios/queixo e altos de
sobrancelhas. Configuração humana anterior e os ajustes faciais feitos durante a
atualização foram preservados; fontes e cenas antes das calibrações estão em
artifacts/character-controls/user-before-*.scene.json e female-before-*.ignis-character.
A troca conservou os valores, reaplicando campos corrigidos; não promete bake
igual para offsets faciais que já estavam ativos. EditorPID65799 deixado aberto,
pausado/FPSOFF, fonte final pronta, pontos ativos; save corrente na cena feminina
e export user-current.ignis-character-config. Recibos verification.json e
native-verification.json/editor.png no mesmo diretório. Drag via egui RawInput
foi validado; não alegar aceite humano nem certificação física CUA de arraste.
Um executável corrente target/release/ignis, app por symlink, sem push.
[Contrato, reprodução e limites](character-controls.md).

## EAF-187 — painel de parâmetros no editor

Scene106/replay116; conteúdo6/protocolo5/PSV1/checkpoint3 preservados. Build
`5e0c59f86cc89bfc`,24 Rust/7alvos (2novos críticos), Clippyrelease/alltargets
-Dwarnings, fmt/Python/diff/build aprovados.4gruposMCP com a fonte feminina real:
21controles, gesto/undo/redo, inválido/export/import, Scene/portátil/replay.
Receita/bake atômicos e cache sem releitura de fonte, ausência de fonte não
impede renderização, alteração manual e metadado adulterado bloqueiam sliders;
receitas não entram no conteúdo público da rede. Clique CUA0,5→0,735 mudou a
silhueta; undo restaurou a base. Drag físico não certificado pela automação;
gesto coberto emMCP/Rust. Editor final aberto pausado/FPSOFF, sem UI de jogo.
[Contrato e limites](character-base.md); recibos em artifacts/character-panel.
Um build corrente, symlink do app preservado, rascunhos próprios removidos.

## EAF-183 — fonte nativa e paisagem representativa

Terrain2 com leitura1 bit-exata; SourceRidge global e publicação regional atômica.
Build5c2e797b819983f2,9Rust/4alvos,4MCP11grupos isolados,Clippyalltargets/fmt/
Python/diff/build aprovados. Regiãogrande2km gerada/capturada nativamente e player
caminhou11,865m grounded;17MiB local buffered, replay/rede8MiB não ampliados.
[Contrato, revisão visual/performance e logs](representative-world.md). Primeiro
preview íngreme foi recusado; proporções/canal/câmera/névoa ajustados antes de
vegetação184. Base física/player preservados na calibração final density0,00006.
Uma versão corrente target/release/ignis, sem GUI pelo systems; imagem integrada
184 e aceite humano ainda precisam ser distinguidos de validação automática.

## EAF-182 — terreno físico e caminhada

Build b13af5530ec3621b; Scene104/replay114/checkpoint3.79Rust/9alvos selecionados,
Clippy release/all-targets -Dwarnings,fmt/Python/diff/build aprovados.9MCP31grupos
sem janela: novo terrain_collision3 e8 roteiros existentes28 em workspace isolado.
Contrato, comandos, logs, classificação do novo registro nativo no teste portátil
e limitações em [terrain-collision.md](terrain-collision.md). Provas de chão real,
subida/descida/junção/queda/arco/escala/rotação/corpo dinâmico, apoio sem tapar
buraco, erro atômico, edição/undo e save/load/replay; PNG exato. Não é suíte completa
(o agregador mantém5 scripts gráficos não registrados), teste humano ou streaming.
Uma versão corrente target/release/ignis; app humano preservado, nenhuma cópia
nova de binário/instalador. Demo examples/terrain-walk.scene.json aguarda aceite.

## EAF-175 — geometria compartilhada compacta

Build640c86a4cc5c60d0, macOSarm64/Metal.16Rust/lib+4alvos, incluindo3provasGPU;
seis pares de material/sombra/seleção/atmosfera shared×expanded comRGB8exato.
Nova prova de remapeamento por todos os atributos e nova provaGPU de66.049vértices
coplanares: reproduziu1.422pixels errados antes de corrigir ordem/paginação,
passou apósu16/u32adaptativo e ordem por instância. Clippyalltargets-Dwarnings,
fmt/buildrelease/Python/diffcheck aprovados. QuatroMCP/18grupos executados
isoladamente e diretamente: agregador bloqueado por quatro roteiros preexistentes
fora deSCENARIOS/interface, sem relaxar asserções ou alegar suíte completa.
Comparativo nativo3vistas+8poses e32repetições exatos; tempos/memória e limites
em [small-geometry-cost.md](small-geometry-cost.md). Cena103/replay113/protocolo5
preservados, janela/câmera humana intactas, executável corrente target/release/ignis;
nenhum FPS de gameplay/aceite físico ou outro backend inferido.

## EAF-172 — catálogo visual e compartilhamento estático

Scene103/replay113/conteúdo5/protocolo5, buildefde0093aaf3059b. 218Rust/14alvos+lib, quatro roteiros MCP/15grupos, Clippyalltargets -Dwarnings/fmt/release/Python aprovados. Novas provas críticas: 10mil instâncias com rollback/persistência; cinco pares GPU incluindo sombra/material/atmosfera/seleção/sobreposição com RGB8 exato; asset300 no codec sem truncamento. Native MCP save/load/capture10mil instâncias/1,28M triângulos, 384v residentes/800kB de instâncias. Referência comparada e variação temporal em [mesh-resources.md](mesh-resources.md); evidências atuais em artifacts/mesh-resources. Sem nova floresta/streaming/FPS de gameplay/aceite artístico inferidos; uma build corrente e nenhuma janela reiniciada.

## EAF-160 — apresentação nativa completa no recorte declarado

Scene99/replay109/protocolo4/conteúdo1/checkpoint2; build72ce80dca500d2c0,
macOSarm64/release/rustc1.97.1.49Rust/10alvos release/locked +1callback semdevice;
Clippyalltargets e --no-default-features lib/bin com -Dwarnings;17MCP76grupos
headless aprovados. Root1,5m/s incidiu contra intenção2m/s e obstáculo dinâmico;
sem deslocamento duplo, cast/life/reuso/checkpoint/continuação/replay exatos.
Retarget entre rigs alterou gait real; IK público validado por rig antes do commit
de réplica. QUIC compartilhou poses e render; prefixoPNG exato e PCM não nulo
com dispositivo desligado. Cena canônica salva pelo MCP em examples/npc-presentation-complete.scene.json.
Roteiro, contratos, perf e limites em npc-presentation-complete.md; logs/relatórios
em artifacts/npc-presentation-complete-* e PNG/WAV no workspace mcp-validation.
Um executável corrente target/release/ignis; nenhum pacote/app histórico criado,
scratch próprio removido após validação. Sem janela/foco/alto-falantes/aceiteartístico,
sem suporte extra/plataforma, criador universal/importação/streaming inferidos.

## EAF-159 — circulação aérea XYZ (30/10/2026)

Build `b5d67fa265b6db6b`, macOS arm64, Scene98/replay108/protocolo3/conteúdo1/checkpoint1.
27Rust/6alvos,14MCP/61grupos,check/Clippyreleasealltargets-Dwarnings,
fmt/Python/diff aprovados. Contrato, revisão estática e limites em
[npc-air-circulation.md](npc-air-circulation.md).

Prova crítica: oposiçãoXYZ reta/curvada/semhover, corpos dinâmicos anônimos,
limites físicos, checkpoint/baseline/replay, identidade/reuso e lançamento
adiado por conflito sem usar cache de chão ainda inexistente. Quatro casos MCP
(cruzamento reto, corredor curvado, vertical e convergência) chegam sem
interpenetração; continuação do checkpoint e PNGprefixo exatos. Cobertura existente
inclui voo/patrulha/híbrido/terrestre/percepção/render/rede e controles remotos.
Relatório corrente em `artifacts/mcp-validation/report.json`.

Cena `examples/npc-air-circulation.scene.json` salva via MCP e preservada na autoria.
Binário corrente `target/release/ignis`; nenhuma janela humana manipulada, cópia
histórica de binário, instalação ou push. Aceite visual humano separado;
sem garantia de arbitragem universal/FPS/WAN/streaming/save portátil.

## EAF-158 / EAF-149 — checkpoint e NPC v1 integrado

Scene 97/replay 107/protocolo 3/conteúdo 1/checkpoint 1, build `c316e5eeb5acde56`.
79 testes Rust distintos em 12 alvos selecionados por impacto; 26 cenários MCP/92 grupos,
check e Clippy release all-targets `-D warnings`, fmt/Python/diff aprovados. A repetição
final da prova crítica e do cenário integrado usa cura de 90 ticks: lease ativa no
save aos 60 ticks, continuação sem duplicar custo, voo/buff/DOT/rotina preservados.

Gates Rust agregados: actor_controls 14, character_controller 19, identity 4,
local_transactions 5, npc_checkpoint 1, npc_hybrid_mobility 1,
npc_population_scale 1, npc_presentation 1, npc_routines_interactions 1,
population 11, resurrection 16, replay_inspection 5. Um teste antigo de população
falhou ao retirar 100 NPCs num tick; passou após usar lotes no limite real de 64,
sem aumentar a admissão. Logs npc-checkpoint-regression/final-tests/critical-final.

O relatório amplo `artifacts/npc-checkpoint-suite-report.json` preserva os 26 cenários;
`artifacts/mcp-validation/report.json` registra a repetição final de npc_checkpoint.
Integrado guarda/civil/aliado/criatura salvo via MCP em examples/npc-v1.scene.json;
capturas pré-save/prefixo 0 byteexact. Checksum e catálogo forjado recusados sem
mutação, tokens externos expirados, sessão nova continua; reuso com origem retirada,
ruído após serial 2^30, continuous sem histórico e roundtrip 100 NPCs+500 Players.
Load recusado durante host ativo. Documentos: npc-checkpoints.md e npc-v1-demo.md.

Os nove eixos EAF-150..158 fecham tecnicamente a matriz EAF-149. Validação inteiramente
headless/offscreen; sem janela/foco/aceite humano, dispositivo de áudio, importação
Blender, criador de corpos ou 60Hz/FPS/WAN inferidos. Compatibilidade nativa do save,
nav gerada 64/128, circulação terrestre, cura individual e capacidade 128 NPCs são
limites explícitos do v1. Não conclui automaticamente M09 ou os demais sistemas.

## EAF-145/148 — carga conjunta QUIC, 30/10/2026

## EAF-157 — capacidade real de NPCs

Scene96/replay106/protocolo3, buildd39d26100daa70f7.59Rust/7alvos e9MCP/35grupos,
checkalltargets/Clippy lib/bin/exemplo/prova-Dwarnings/fmt/Python/diff. Uma prova crítica
nova compara sensores lineares/espaciais, slots NPC>=500/rádio>8, gen24membros e reuso
sem herdar recurso pago, capacidade inválida atômica e replay. Carga local120ticks:
100NPC+500Players/628slots/600grounded,71450dano e custos exatos,5181passos/50ouvintes,
históricos0. P95sim31,559→18,576ms/total18,706ms após flags de oclusão na BVH, mesmas
saídas; atividade externa não isolada, sem60Hz/FPS/WAN/render/avatares inferidos.
Logs npc-scale-* e reportMCP; receita npc_load e exemplo npc-population-scale.scene.json.
Programa149 ainda aguarda checkpoint e cenário conjunto.


## EAF-156 — avatar, animação e sinais por indivíduo

Scene95/replay105/protocolo3/conteúdo1, build33966d1daf4e1e66.29Rust/6alvos
(meshes20, conteúdo1, sessões3, avatar1, rotinas1, transporte3),6MCP/23grupos,
checkalltargets/Clippy lib/bin/prova-Dwarnings/fmt/Python/diff. Variante por traits,
walk/air/cast/dead, mesma pessoa revive/nova pessoa limpa, parser público atômico,
WAV24kHz não nulo e prefixo20ticks/PNG byteexact. Logs npc-presentation-*.log e
artifacts/mcp-validation/report.json. Exemplo npc-presentation.scene.json salvo
pelo MCP. Sem janela/foco/dispositivo de áudio/Blender/IK/root motion/aceite humano;
checkpoint/escala/programa149 continuam abertos.


## EAF-155 — rotinas, diálogo e propriedades físicas

Scene94/replay104/protocolo2, build aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|5ceec3aaec6d143a. 52Rust/6alvos,
11MCP/37grupos, checkalltargets, Clippy lib/bin/prova-Dwarnings, fmt/diff/Python.
Prova crítica de recursos por indivíduo/traits, agenda/ciclo, revisão/vida de sessão,
rollback tardio, colisor aberto/fechamento ocupado, fonte do bake retirada/reusada,
voo curvado e réplica por bytes. Prefixo300ticks/pixels iguais no replay. Corrigida
reinserção de collider desativado no BVH manual KCC; filtro também ignora proxy antigo.
Teste histórico de NoiseBudget atualizado da antiga quota global para esgotamento
serial, sem perder proteção de rollback de ressurreição. Logs npc-routines-*.log,
artifacts/mcp-validation/report.json; exemplo npc-routines.scene.json. Sem janela,
aceite humano, dispositivo de áudio ou escala NPC alegados. Programa149 aberto.


## EAF-154 — tática e cooperação sobre evidências

Scene93/replay103/build2651a50e94e99cf1/protocolo2, fonte/binário conferidos. Objetivo tactical com alcance/cobertura/flanco e cadência/esforço/perfil comum; ordens/role/formação com posição/indivíduo/vida congelados, admissão completa ou sem mutação. Leases de cura opt-in por grupo/alvo/vida/execução,1por fonte e prazo, eleição por concern/sociability/intelligence/discipline; comandos humanos explícitos preservados.

27Rust/3alvos (ability_policy15,population11,uma nova prova crítica integrada ampliada). Protege privacidade/posição conhecida, formação física, lote insuficiente sem parcial, rejeição de player, ordem obsoleta após despawn/spawn, limpeza morte/revive mesmo indivíduo, eleição/custos/cast/projétil, dono duplicado em serde, override humano e replay.8MCP25grupos: tática/cooperação,circulação,objetivos,política de habilidades,ressurreição,população,mind,híbrido; captura PNG100 vivo/replay byteexata e inspecionada. Check releasealltargets,Clippy lib/bin/prova-Dwarnings,fmt/Python/diff aprovados. Primeira fixture excedia limite existente8m de conector; corrigida a receita, sem ampliar limite para fazer passar.

Evidências artifacts/npc-tactics-{check,tests,critical,clippy,mcp}.log/npc-tactics-validation.json e mcp-validation/report.json/PNGs. Demo de apoio examples/npc-tactics-cooperation.scene.json viaMCP/cópia exata. Sem janela/foco/aceite humano; limites de candidato/rota/fila64 e lease/prazo documentados. Matriz149 ainda mantém rotinas/apresentação/checkpoint/escala abertos.

## EAF-153 — circulação terrestre por proximidade anônima

Scene92/replay102/buildaa595aee7c5cfef3/protocolo2, fonte/binário conferidos. Mesmo executor/KCC com velocidade alternativa limitada e apoio, sem teleporte. Inteligência controla horizonte/esforço; traits comuns controlam espaço/risco/contorno. Trechos jump/glide/fly comprometidos explicitam committed_air e preservam trajetória; não são multidões aéreas novas.

13Rust/3alvos finais (npc_circulation1crítico,passages11,npc_hybrid1); prova protege cruzamento real, velocidade/distância, sensor sem memória/alvo, parede oclusiva, identidade/reuso/reset/erro atômico e replay/tamper. Falha inicial revelou leitura da velocidade anterior em KCC, corrigido usando requested_velocity.6MCP19grupos headless: circulação/passagens/voo/controller/mind/híbrido; PNGprefixo240 vivo/replay byteexato e imagem inspecionada. Check releasealltargets,Clippy lib/bin/prova-Dwarnings,fmt/Python/diff aprovados. Demo salva pelo MCP e cópia byteexata em examples/npc-circulation.scene.json.

Evidências artifacts/npc-circulation-{check,tests,clippy,mcp}.log/npc-circulation-validation.json e artifacts/mcp-validation/report.json/PNGs. Sem janela/foco/aceite humano/FPS/escala universal. Matriz149 conserva tática/cooperação/rotinas/apresentação/checkpoint/escala abertos.

## EAF-152 — rede gerada de colisores

Scene91/replay101, build4a580516052d8dc0, protocolo2; hash do fonte/executável conferido. Receita por entidade/template/região/célula/raio/seed/budget e amostragem aérea opt-in gera rede física64/128. Preview read-only, fonte física invalida/regenera em autoria, actor movement/visual não; cache derivado não vai para arquivo. Máscara de custos/preferências/limites do perfil mental comum permanece no planejador.

6Rust/2alvos no fonte final (nova prova crítica de autoria/erro atômico/invalidação/privacidade/replay e scene_edits existente); rodada inicial também17groundnavigation. Check releasealltargets,Clippy lib/bin/prova-Dwarnings/fmt/sintaxePython/diff aprovados.7MCP24grupos headless (projects/replay/sceneedits/ground/mind/hybrid/generation), incluindo desvio automático ao redor da parede, subida de rampa por nós derivados e navegação aéreaXYZ física. PNGprefixo vivo/replay byteexato; demo examples/npc-generated-navigation.scene.json salva/load peloMCP e cópia byteexata. Uma nova prova de risco, sem testes por flag. Sem janela/foco/aceite manual.

Evidências artifacts/npc-generation-{check,tests,clippy,mcp}.log/npc-generation-validation.json e artifacts/mcp-validation/report.json/npc-navigation-generation-verification.json/PNGs. Primeiro gate revelou assinatura dependente da ordem de entidades; canonicalização corrigida e gates repetidos no fonte final. Região/discretização/budgets limitam cobertura; report limited não é prova de impossibilidade e runtime não inventa posição de alvo. Cache/preparação KCC pode ser custoso em autoria; sem alegação de FPS/navmesh global/multidões. Matriz EAF-149 mantém circulação/tática/cooperação/rotinas/avatar/persistência/escala abertos.


## EAF-151 — controlador terrestre e motores aéreos integrados

Scene90/replay100, build9b59736f831becaf, protocolo2, fonte/binário conferidos. Mesmo corpo troca KCC/dinâmico conforme motor; Walk/Fly/Jump/Glide expostos em mobility.motor_mode e controller.airborne, gravidade/intenção/colisão preservados. Custo mental por cautela/medo/coragem/inteligência é separado de Motion físico.

Fonte final:30Rust/3alvos selecionados (mobility,groundlocal,npc_hybrid); prova crítica ampliada protege sequênciaswalk→fly→walk/walk→jump→walk/glide→walk, localflysemciclodeâncora, capacidade ausente, morte/retorno no ar com identidade, reset/replay/prefixo. Primeira rodada50Rust/4alvos também cobriucontroller/groundnavigation; após correção da âncora foram repetidos gates impactados. Check releasealltargets eClippy lib/bin-Dwarnings aprovados,fmt/sintaxePython/diff-check;7MCP26grupos headless no binário final (mobilidade,voo,controller,ground,groundlocal,mind,hybrid). Nova verificação somente de integração crítica, não teste por flag. PNGprefixo vivo/replay byteexato, capturatick1501000×700 inspecionada. Demo examples/npc-hybrid-mobility.scene.json viaMCP, sem janela/foco/aceite manual.

Evidência artifacts/npc-hybrid-{check,tests,critical,clippy,mcp}.log/npc-hybrid-validation.json e artifacts/mcp-validation/report.json/npc-hybrid-verification.json/PNGs. Planeio não decola sozinho; jump/glide mantêm apoio plano completo nos endpoints, rampas são walkKCC; não declarar navmesh/descoberta/cooperação/multidões concluídos. Matriz EAF-149 permanece aberta para os demais eixos.


## EAF-150 — cognição inicial, programa NPCv1 aberto

Main f1751ec, Scene89/replay99, builde49756e740f75e05, protocolo2. Perfil opt-in de inteligência/personality/seed/variation quantizado por indivíduo; objetivos elegíveis/prioridade/histerese/cadência, limite de pontos de investigação e esforço KCC integrados. inspect_npc_mind read-only e configuração MCP-first.

34Rust/4alvos (npc_mind/objectives/perception/ground_local_connections), zero falhas; só1nova prova crítica protege atomicidade/privacidade/identidade/replay. Check releasealltargets e Clippy lib/bin-Dwarnings aprovados, formatter aplicado, sintaxePython e diff-check.6MCP27grupos headless passaram: personalidade, objetivos, percepção, identidade, spawn e groundlocal. PNGprefixo vivo/replay byteexato, tamper detectado, template/seed/disabled/config inválida e limites de esforço cobertos. Não executada suíte inteira sem risco adicional concreto. Aviso block0.1.6 preexistente separado.

Evidência artifacts/npc-mind-{check,tests,clippy,mcp}.log/npc-mind-validation.json e artifacts/mcp-validation/report.json/npc-mind-verification.json/PNGs. Demo examples/npc-mind.scene.json salva/carregada pelo MCP, cópia byteexata; captura1000×700 tick30 inspecionada, azul aproxima/rosa recua. Sem janela/foco/aceite humano nesta unidade. Fonte/hash do build conferidos; sem push. Programa EAF-149 permanece aberto: matriz docs/npc-v1-closure.md, próximos híbrido/descoberta/circulação/tática/cooperação/rotinas/avatar/persistência/escala.


Buildf75f4a7ea9154d40/130inputs (exemplos/helpers por SHA256 separado), Scene88/
replay98/protocolo2.1000TLS reais×100ações em cada receita mixed e chefe central,
1000grounded/1000fontes de passos, saúde/entregas/filas/história corretas.1000réplicas
no mesmo tick,1atrás apósdrain; gate≤12. Chefe recebeu100000commits/1000buffs;
players intactos. P95total autoridade7,449/17,778ms; lag durante combate estimado
p95250/100ms, mixed máximo1033ms; não FPS/WAN/60Hz universal. Gerador4workers/
32endpoints e backend2workers, M4Pro14CPU/48GiB. Duas conexões do chefe recuperadas
na2ªtentativaTLS, sem repetir ação/sessão. Counter ACK regressivo0 na carga.

28Rust e7MCP27grupos/Clippyalltargets do EAF-148 passaram; física anteriormente
77Rust/11MCP44 e TLS27Rust/7MCP27 (conjuntos sobrepostos). Só provas novas críticas
para chão/handshake/ACK, sem teste por flags do probe. Local1000sem rede repetido:
mixed/chefe saúde exata, p95total6,644/14,906ms e zero histórico. Offline com rede
negada e replay/PNG existentes preservados. Contrato/receita em joint-network-load.md;
auditorias/relatórios/logs current em joint-network-*, network-load-* e runtime-load-*.
Sem app/janela humana/instalação/push. Hipótese de fallback geral1Hz foi corrigida:
codec já usa deltas por campo, maioria dos frames em10Hz; lag/NeedBaseline ficam
explícitos, sem mudança de protocolo por hipótese.

## EAF-146 — estabilidade de contato KCC, 30/10/2026

Build09c4088aba9675cc/130inputs, Scene88/replay98/protocolo2.77Rust selecionados,
Clippy all-targets e11MCP44grupos aprovados. Uma nova prova crítica6combinações
repouso240+movimento600/pés/grounded/passos/replay; solo plano não afunda na
diagonal. GroundNavigation/runtime usam SAT especializado só no contato inicial
de caixas, demais consultas/solver delegados. Offline com rede negada preservado.
Carga1000TLS×100ações mixed/boss tem todos1000grounded/1000fontes de passos e
saúde exata; p95total7,773/18,023ms, sem FPS/WAN/60Hz universal. Contrato/checklist
em kcc-contact-stability.md; relatórios kcc-contact-* e network-load-{mixed,boss}.
Sem instalação/janela humana/push; timeoutTLS anterior continua registrado.

## EAF-137 — seleção e histórico de área, 29/10/2026

Main após a58b6e3, Scene87/replay95/JSON mantidos; build8d0d1805caf7d57e auditado
fonte/binário/example.102Rust existentes+2oráculos (104total),Clippy all-targets,
16MCP/66grupos aprovados. Novo oráculo crítico versus caminho linear anterior:
seleção/ordem/oclusão/cunha/bordas/ausência/empate,12configurações com fallback/
índice, status/falha, primeiro read após mundo mudar, clone/serde/erro. Oráculo
espacial anterior também passou. MCP:area,persistent_area,aura,projectiles,
free_aim,dispel,attributes,damage,outgoing,local_transactions,identity_bindings,
world_origins,population,actor_controls,continuous_runtime,geometry. Offline/replay/
prefixo/capturas/read-only preservados, sem instalação/janela/aceite humano novo.

Mesma receita1000auras/pico2000volumes,60warmup+600medidos,110000lançamentos,
110impactos/vida999890 por jogador e zero histórico. Simulação p95=7,683ms
(antes78,607),p99=7,908,máximo8,178; totalp95=8,355/máximo8,873ms.2ticks de drain
separados confirmam último pulso/expiração. Sem buffs/ruído/NPC/render/rede na
carga. Todos rows/ordem/JSON mantidos; leitura completa paga ordenação sob demanda.
Artefatos:area-runtime-{integration,oracle,spatial,clippy,build,mcp}.log,
area-runtime-report.json e runtime-load-area.json. Contrato/revisão10itens em
area-selection-runtime.md; buffs64/bases512/ruído ainda são próximos limites.

## EAF-136 — agendas e dissipação locais, 29/10/2026

Main após cf55cb7, Scene87/replay95, buildbfdfc6445209aaae auditado fonte/binário/
example.131Rust selecionados (114integrações,16origins,1novo crítico),Clippy
all-targets e14MCP/57grupos aprovados. Novo crítico:1000agendas/1000áreas, quotas
64/16, refresh/IDs>63 fora da ordem numérica, dissipação exata/rollback tardio
por pool ausente, serde/alvo inválido, morte e ordem de recibos/replay. MCP:
periodic,persistent_area,aura,area,dispel,dispel_triage,local_transactions,
channel,projectiles,world_origins,identity_bindings,population,actor_controls,
continuous_runtime. Offline com rede negada preservado. Primeiro lote MCP começou
antes de fechar build; repetido com fonte/binário final antes das cargas isoladas.

60warmup+600medidos,1000Players movendo,110000aplicações e110pulsos/vida999890 por
jogador, zero histórico. Periódicos: pico1000, p95simulação2,955/p99=3,087ms,
totalp95=3,710ms, drain1tick. Áreas: pico2000, p95simulação78,607/p99=79,128ms,
totalp95=79,315ms, drain2ticks. Drain tem custo separado nos JSONs. Áreas corretas
mas acima16,7ms: classificação/ordenação de1000candidatos por pulso é gargalo aberto.
Sem NPC/buffs/ruído/render/rede nas cargas. Contrato/checklist/custos e próximo
recorte em scheduled-source-pools.md. Artefatos correntes:scheduled-{critical,
integration,origins,clippy,build,mcp}.log, scheduled-report.json,
runtime-load-{periodic,area}.json. Sem push/instalação/janela/aceite humano novo.

## EAF-135 — projéteis e origens escaláveis, 29/10/2026

Main local após dc1e713, Scene87/replay94, build28cbb8961e9a9fdb auditado contra
fonte/binário/example.136Rust selecionados:16world_origins,9identity_bindings,
110integrações(projectiles/free_aim/npc_aim/periodic/persistent_area/population/
continuous_runtime/scalable_actors),1oráculo. Um teste novo crítico1000fontes,
quota/rollback/lease1000/detach/serde/coleta/reuso/replay e oráculo existente
ampliado para sweep/TOI/arestas/9000m. Clippy all-targets aprovado; após derivar
Default equivalente, prova crítica e oráculo repetidos no fonte final.
14MCP/60grupos:projectiles,free_aim,npc_aim,periodic,dispel,channel,persistent_area,
aura,identity_bindings,world_origins,population,geometry,actor_controls,
continuous_runtime. Offline com rede negada/replay/prefixo/capturas preservados.

runtime_load --delivery projectile:1000corpos movendo/colisão/oclusão,1000projéteis
a cada6ticks,60warmup+600medidos,110000lançamentos/661000controles, todos110impactos/
vida999890; zero histórico/drain. Simulação p95=2,838/p99=2,891/máximo2,946ms;
total p95=3,670/p99=3,754/máximo3,846ms. Direto repetido:p95simulação2,519/total
3,338ms e mesmo resultado. Sem NPC/buff/periódico/área/ruído/render/rede na carga;
64mil projéteis não foram medidos. Nenhuma instalação/janela/aceite humano novo.
Logs/relatórios:artifacts/source-pools-{origins,bindings,integration,critical,
oracle,clippy,build,mcp}.log, source-pools-report.json e runtime-load{,-projectile}.json.
Contrato/checklist/custos mantidos em source-delivery-pools.md. Próximo recorte:
agendas periódicas/áreas e seleções locais de dissipação, ainda não escalados.

## EAF-134 — broadphase conservador de visada, 29/10/2026

Build fc15e84187cbfd78 auditado fonte/binário/example, main após e193390.
Scene87/replay93 inalterados. Um oráculo crítico novo passou (160raios/4096entidades,
caixas/cunhas/flags/bordas/degenerados/mover/remover/repor);111Rust existentes/8alvos,
Clippy all-targets e11MCP/42grupos passaram. MCP:abilities,ability_policy,
cast_movement,channel,identity_bindings,world_origins,population,spawn,geometry,
actor_controls,continuous_runtime. Replay/lifecycle e offline com rede negada
preservados. Sem instalação/janela/controle do editor humano.

Mesma carga runtime_load de EAF-133 isolada:1000Players reais,60warmup+600ticks,
1000ataques diretos a cada6ticks, todos110impactos/vida999890,110000ações/661000
controles e zero histórico. Simulação p95=2,515ms (antes108,271),p99=2,588,
máximo2,760ms; total com dispatch p95=3,252/máximo3,528ms. Receita coube16,7ms
nestas amostras, sem promessa de latência/FPS/rede ou outros tipos de combate.
Scans de outros consumidores e pools pequenos permanecem. Relatórios correntes:
artifacts/spatial-{integration,clippy,build,mcp}.log, spatial-report.json,
runtime-load.json/stderr.log. Decisão/revisão de10itens:combat-broadphase.md.

## EAF-133 — admissão por ator e índice FIFO, 29/10/2026

Main local após eb8e604, Scene87/replay93; build `f5157f99137abb7e` auditado
contra src/Cargo.lock/build.rs, binário e workload. Nenhuma instalação/janela nova.
121Rust em10alvos, Clippy release/all-targets e do example passaram. Um teste
crítico novo protege justiça entre1000atores, input coalescido, identidade,
rejeição atômica e replay/prefixo.13MCP/54grupos passaram (editor_mesh,
sculpt_brushes,abilities,cast_movement,channel,identity,identity_bindings,
world_origins,population,spawn,actor_controls,local_transactions,continuous_runtime).
Inclui modo offline com bind/connect negados; escultura integrada preservada.

Workload runtime_load:1000corpos reais movendo,110000ações diretas,661000controles,
110impactos por jogador/vida999890, zero histórico contínuo.60warmup+600ticks sem
pacing, sem suíte concorrente: simulação p95=108,271ms/p99=110,891ms, dispatch
p95=0,765ms. Correto mas acima de16,7ms nos ticks de ação devido a visada linear;
EAF-134 trata broadphase. Sem projéteis/buffs/NPCs/ruído/render/clientes de rede.
Primeira receita com pares entre linhas teve oclusão legítima; só pares foram
corrigidos, sem desativar corpos/oclusão. Não é aceite MMO/60Hz/FPS.
Logs/relatórios correntes:artifacts/admission-{check,clippy,rust,build,mcp}.log,
admission-report.json, runtime-load.json/stderr.log. Contrato e checklist completo
em frame-admission.md; Scene permanece87, versão nova somente do replay93.

## EAF-132 — execução contínua e exportação íntegra, 29/10/2026

Main local sobre5dbde55 + reparo de escultura6fbaac3, Scene87/replay92. Build
`aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|7138146e3d188320`
auditado contra src/Cargo.lock/build.rs e binário. Sem instalar ou controlar o
editor humano aberto.

- Clippy release/all-targets -D warnings e65Rust selecionados em8alvos passaram:
  continuous_runtime,actor_controls,local_transactions,hearing,footsteps,
  population,spawn,scalable_actors. Somente2guards Rust novos nesta etapa.
- Teste crítico de continuidade:37400ticks,5000habilidades/ruídos e mais5000inputs
  sem acumular histórico; ring de diagnóstico limitado, identidade de ruídos nas
  fronteiras30/62bits, erro atômico de modo/input/relógio, reset/escopo e replay
  gravado preservados. Verificação de1000Players/área/vida/reuso/replay também
  passou no mesmo fonte conjunto.
- 21cenários MCP/97grupos:projects,editor_mesh,sculpt_brushes,lab,hearing,
  hearing_guides,footsteps,scenarios,replay_inspection,sound_attention,abilities,
  projectiles,periodic,channel,identity_bindings,world_origins,population,spawn,
  actor_controls,local_transactions,continuous_runtime. Inclui os consumidores
  de escultura do reparo paralelo e as rotas de I/O/projeto afetadas.
- O novo cenário crítico MCP chegou a39600ticks sob sandbox macOS com bind/connect
  comprovadamente negados. Continua admitindo controle, não retém entradas
  consumidas, recusa sobrescrever bytes existentes com replay incompleto, mantém
  política em reset/load/projeto e volta a exportar replay gravado verificável.
  Três grupos novos, nenhum gate de janela física/cliente de rede inferido.

Logs correntes:artifacts/continuous-{check,clippy,rust,build,mcp}.log,
continuous-inspect.json e mcp-validation/report.json; contrato/checklist em
continuous-runtime.md/performance.md. Nenhuma dependência/worker/transporte novo.
A admissão de16habilidades globais/tick e pools de entregas/origens permanecem
limitados neste checkpoint; EAF-133 trata admissão/indexação por ator. Capacidade
de armazenamento/execução contínua não equivale a combate1000 sustentado nem rede.

## EAF-131 — armazenamento e consumidores de atores, 29/10/2026

Main local, Scene87/replay91. Build auditado
`aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|0e4f391547a15b2e`;
hash dos fontes/Cargo.lock/build.rs coincide com o executável usado.

- Todos os alvos compilados e Clippy release/all-targets sem warnings do projeto.
- 234testes Rust existentes/novos selecionados:123unitários e111integrações de
  controles, transações, identidade, população, retorno, percepção, área,
  reconhecimento, rádio, cast e armazenamento denso. A fixture manual de rádio
  foi adaptada para inicializar o armazenamento antes de forjar conhecimento;
  seu rerun7/7 passou. Asserções antigas de limite513/default-array foram
  migradas para o contrato novo, sem remover invariantes de identidade/evidência.
- 23cenários MCP/96grupos aprovados no mesmo build: lab, replay_inspection,
  targeting, recognition, communication, abilities, vitality, cast_movement,
  projectiles, periodic, area, persistent_area, resurrection, identity,
  identity_bindings, world_origins, population, spawn, actor_controls,
  local_transactions, character_controller, ground_navigation,
  ground_local_connections. Inclui replay/prefixo/PNG e gate offline que nega
  bind/connect no processo macOS. Nenhuma janela/demo humana foi manipulada.
- Somente2testes novos, ambos críticos: fronteiras de ActorSet e integração com
 1000Players controlados, área que aplica dano em todos, vida64/256/999, observação
  do alvo999 por NPC acima1000, remoção/reuso de identidade e replay. O ensaio
  curto levou51,5ms incluindo setup/replay, dos quais9,0ms de preparação/controles;
  não é benchmark sustentado nem tempo de tick. Não alegar FPS por esse valor.

Logs correntes: artifacts/scalable-{check,clippy,rust,critical,build,mcp}.log,
scalable-vitality-recheck.log e mcp-validation/report.json. Scripts existentes
centralizam versões correntes no fonte via verify_lab, preservando fixtures
históricas de rejeição. Controles4096por tick ainda têm log4096total; prazos
36000ticks, efeitos/origens e rede continuam gates próprios. Próxima etapa:
execução contínua separada da gravação (EAF-132). Sem push, instalação ou aceite
de1000clientes/combate sustentado inferido.

# Validação

## 29/10 — proteção da escultura e direção coerente de Volume (EAF-110)

Reincidência humana: espinhos já no primeiro traço. Leitura da janela mostrou
Cube24/3458v/6912t, raio0.241m e Mover ativo; o pincel do print original não foi
confirmado. Reparo cobre Draw e Grab sem atribuir a causa ao uso de um pincel.
Teste crítico falhou antes da mudança: Grab deslocando0.35m na tangente de cubo
novo era aceito com faces dobradas. Agora há guarda de orientação e rollback;
soltar um candidato inválido preserva cena/histórico, recuar recupera a prévia
válida e permite uma confirmação/undo. UI não anuncia carimbos contínuos/controle
de inversão para Mover; explica que Volume desenha relevo.

Draw usa normal média da área por carimbo sobre a base congelada, em vez da
normal individual de cada vértice. Mesmo Stroke/unidades/raio, sem reduzir força
silenciosamente ou remesh. Preservados IDs/triângulos/canais/rig/fontes e contratos
MCP. A guarda é conservadora em relação às faces anteriores, sem detector global
de auto-interseções ou reparo de deformações já gravadas.

Validação proporcional ao risco, sem suíte ampla automática: **10 testes de
kernel/contrato** (dois novos críticos) e **17 testes UI existentes** passaram;
Clippy focado-Dwarnings/fmt/diff e **3MCP/23grupos** de editor_mesh/sculpt_brushes/
editor aprovados. Polimento final apenas removeu instrução de Ctrl no Grab;
build/Clippy/probe/MCP foram repetidos, sem repetir testes de semântica intacta.
Probe compositor/GPU existente ampliado: primeiro Draw em cubo novo nos dois
sentidos sem faces dobradas, diferença máxima0.0114301m devido à fase de amostras
(menor que um carimbo de0.04m); Grab±direção inválido recusado, mouse-up sem edição,
recuo/recuperação/um undo. Esfera preserva95v/max0.282722m e preview/commit/undo/
redo/comando serializado idênticos. Seis outros pincéis mantêm seus gates. PNGs
cube-first-forward/reverse e grab-rejected inspecionados.

Build **1221b4e671cd3aec**, fonte **115 arquivos**/binário/app samefile auditados.
Release corrente **21150240 bytes**, SHA256 `b198495820dbc5cdf81ca294c35ee6f8b979a059de484e20d20b973659ec9d78`.
Scene87/replay91/layout3/projeto1 herdados de EAF-131/5dbde55, sem novo formato/deps
neste reparo. Evidências em artifacts/sculpt-direction/{before,kernel,ui,clippy,
build,probe,mcp}.log e validation/source-manifest/retention.json; capturas em
artifacts/editor-sculpt-quality/. App /Users/zero/Applications/Ignis Editor.app.
Sessão humanaPID79953 preservada, sem comandos de entrada/fechamento/reinício;
processo aberto continua no código anterior e precisa salvar/reabrir para testar.
Retenção com lock/stat/lsof removeu111outputs first-party supersedidos/inativos
(1000088688B), sem remover processos/arquivos em uso ou logs de prova.
Aceite humano/nativo separado; EAF-110 permanece In Progress. Docs e alterações
paralelas de execução contínua preservados, sem push.

## 29/10 — modelagem por componentes (EAF-130)

Main desde0bea5d6, build **51d87e3799b55576**, Scene86/replay90/layout3/
projeto1/dependências preservados. **1173 testes Rust/120 alvos release/all-targets**,
zero falhas/ignorados; Clippy all-targets-Dwarnings, fmt/diff e build aprovados.
**117 cenários MCP/504 grupos completos headless**; 97 resumos identificados
coincidem com a fonte/binário corrente, demais resumos legados omitem ID.

Extrusão de região conectada mantém furos/fechamento sem tampa interna velha;
inset plano convexo conserva subdivisões/largura constante; bevel plano de cadeias
em sólido convexo partilha interseções por índice. Dez testes de geometria/contrato
cobrem volume/orientação/subdivisão/rig/clipes/canais/budgets/isolados/refusais e
três testes UI cobrem botões/idle/arraste/número/cancelamento/undo/seleção.
MCP novo cobre preview=commit, operações encadeadas, refs obsoletas/erro atômico/
batch/plantas/detach/save/load/reset/replay/prefixo/PNG/editor.

Probe GPU novo: botões Editar→operação, drag puro até soltar, seleção resultante,
um undo e cancelamento por foco/resize/HiDPI com release consumido. Capturas
1440/1000/900/800 inspecionadas; nomes de componentes não quebram dentro da palavra
e contagens usam caracteres presentes na fonte. Probes anteriores de gizmos,
escultura9pincéis, seleção/raioX/47vXYZ/NPC/rig/catálogo e layout também passaram.
Entrada é sintética/offscreen: nenhum aceite físico/humano/acessibilidade inferido.

Mesmos planejadores/validadores no MCP e UI. Preview limitado e cacheado fora do
Engine, confirmação única revalida scope/revision; Esc/foco/câmera/layout/seleção/
HiDPI cancelam. Rig/clipes/canais/plantas protegidos sem poda/detach/weld implícitos.
Bevel é plano/convexo e inset é plano/convexo; não há perfil curvo, inset arbitrário,
dissolve/remesh ou detecção global de auto-interseção de extrusão.

Evidências: artifacts/editor-modeling-validation/{validation,source-manifest,
retention}.json/logs, artifacts/editor-modeling/, mcp-validation/report.json.
Contrato/checklists em editor-modeling.md/performance.md; fluxos e roadmap
atualizados. Um release corrente **21215920 bytes**, app em
/Users/zero/Applications/Ignis Editor.app aponta ao mesmo arquivo. SHA256
`dd695e8f1afcf1b392157f71b407f6d37aca787de1a8cebb32acc94ddd9d4944`. Sem instaladores/cópias históricas/push ou controle
da sessão visual do usuário. Aceite humano e demais itens do editor continuam
abertos; EAF-130/EAF-104 mantidos In Progress.

## 29/10 — integração de transações, controlador e ilhas no editor

Build960a1c4f79326bc1, Scene83/replay87/layout3/manifesto1: **1220Rust/115alvos**,
zero falhas/ignorados, Clippy-Dwarnings/fmt/build/check/diff aprovados;
**108MCP/461grupos completos headless**,88resumos com ID exato/20legados sem ID.
UI quatro larguras/arraste horizontal/Shift e rail preservados, escultura95v/
0.35097972m+paridade MCPf32/undo/redo, quatro recortes517500pixels exatos.
Duas intenções cinemáticas independentes+Player dinâmico, liberação/lifecycle e
replay com motores+controles+meshes verificados. Oito estudos/provas de conteúdo
passaram; receita6 reproduz a casa inteira exceto versão, montagem altera140v
selecionados com canais/restante preservados. Detalhes/pontas/hash/evidências
em editor-integration.md e artifacts/editor-integration/validation.json.
Fonte/binário/app samefile auditados, release único18298240B/cache2,2GiB.
Capturas e entrada sintéticas offscreen; aceite humano/nativo e UI dos novos
recursos permanecem abertos. Nenhuma janela/main/push ou cena pessoal alterada.

## Resultado29/10/2026 — organização do editor e edição numérica

Nova organização sobre167d8de, após Guilherme confirmar a correção do traçado e
retomar o pedido de design. Removida a barra global de ferramentas; modo/luz no
cabeçalho da viewport, pincéis em faixa lateral e inspetor Pincel/Objeto. Tokens
neutros, miniaturas vetoriais, estados ativos contidos e divisores finos com área
de interação preservada. Campos horizontais compartilhados em vetores/escala,
raio/forças e bias, com Shift fino, digitação/Enter, foco e teclado do egui.
Movimento vertical não publica alterações; cache rebasa após edição externa.
Sem dependência/fonte/shader/contrato/kernel novos ou pincéis fictícios.

Fonte/executável4518b6ecfbe8bc94 conferidos:1189testes Rust/113alvos release/
all-targets,0falhas/ignorados;Clippy all-targets-Dwarnings/fmt/diff/build aprovados.
105cenários MCP/449grupos completos completed/passed/native_window:false.
Três regressões novas do widget cobrem X/Y/diagonal/Shift, alvo inteiro,
precisão/texto/Enter e rebase externo. Qualidade de escultura e demais controles
de entrada anteriores continuam aprovados.

Probe Graphics/GPU real passou em1440/1000/900/800: contextos Pincel/Objeto,
ausência de edição pendente em repouso, campos/toolbar/ferramentas sem overflow,
arraste real de raio0.4→0.6→0.62 com Shift, Y sem revisão nova, três pincéis sem
esculpir através dos controles, popup de iluminação e formulário de projeto.
Mundo autoral inalterado. Primeiro probe encontrou altura excessiva no formulário;
layout foi compactado sem retirar sua asserção. Uma rodada completa foi interrompida
apenas no grupo de processos desta tarefa para restaurar a explicação de Escultura
desabilitada; a rodada final acima completou após essa correção.

Probe de escultura na nova viewport alterou95v/máximo0.35097972m, confirmou
undo/redo e comando serializado. Esse mesmo comando foi enviado ao MCP público:
preview/commit/save/load coincidem bit a bit f32, com índices/metadados exatos.
Scene81/replay85/layout3/manifesto1 preservados. Decisão/checklist/relatório em
docs/editor-ui.md e .ui-craft; logs/consolidado em artifacts/editor-studio.
Capturas atuais artifacts/editor-design-{wide,800,1000,light,project}.png,
object/initial em editor-studio e provas de escultura em editor-sculpt-quality.

Um release corrente18145392B/cache~2.1GiB, app instalado por symlink ao mesmo
executável. Sem main/push/cópias históricas ou sessão/cena humana manipulada.
Os testes/capturas são sintéticos/offscreen; foco/teclado físicos, leitor de tela
e avaliação humana do design permanecem gates separados. Catálogo maior de
pincéis continua como incremento de núcleo+MCP+UI, além destes três existentes.

## Resultado29/10/2026 — estabilidade do traço de escultura

Relato humano reproduzido em processo isolado: Volume na esfera padrão1106v,
raio0.4/força0.08m por amostra, arraste160px, deslocava até1.11m/arestas8.65×.
Picking seguia o relevo da própria prévia; redistribuição por evento também
alterava a densidade. Corrigido com referência congelada, resto de distância
transportado entre eventos e conservação de movimentos coalescidos. Ausência
de superfície quebra a continuidade;128aplicações/gesto continuam limitadas.
Kernel/força por amostra/topologia/Scene81/replay85/layout3/manifesto1 intactos.

Build7d2198ecb59e038a conferido contra fonte/executável:1186testes Rust/113alvos
release/all-targets,0falhas/ignorados;Clippy-Dwarnings/fmt/diff/build aprovados.
Regressão completa105cenários MCP/449grupos completed/passed/headless.
Novas regressões egui conferem esfera padrão, faces sem inversão, deslocamento
limitado, eventos1/8px (desvio máximo0.00031318716m), caminho curvo coalescido,
falha de superfície, teto de amostras e preview=commit/undo/redo. No traço reto,
raio máximo1.3066127m/arestas2.1938198×, sem crescimento descontrolado.

Novo `cargo run --locked --release --example editor_sculpt_probe` usa entrada
real do compositor do editor e GPU offscreen, sem janela. Gesto curvo alterou
102v, máximo0.3513956m; confirmação única, undo/redo exatos e reexecução do
comando serializado aprovados. Antes/depois, cena e comando em
artifacts/editor-sculpt-quality/pointer-*. Prova externa enviou esse mesmo
comando ao MCP público: preview/commit/persistência coincidem bit a bit em f32,
com índices/metadados exatos. A primeira comparação Python de números JSON
exigiu reconciliar serialização direta f32 versus promoção para Value/f64;
conferência final compara os bits f32, sem tolerância geométrica.

Evidências/logs/validation.json/build-audit.json/mcp-pointer-parity.json em
artifacts/editor-sculpt-quality. Um build corrente18095616B, app instalado
aponta para o mesmo executável; sem cópias históricas, main/push ou reinício da
sessão humana. Captura/entrada sintética não comprovam aceite humano nativo.
Redesign geral, novos pincéis, remesh/simetria continuam pendentes; este recorte
corrige o traço existente e não conclui EAF-104/EAF-110 por aceite automático.

## Resultado29/10/2026 — controles, amostragem e chanfros

Basea9e3d3c/checklist83305a2; merges caf6d46(e473e5a),81aa264(460a407),
b3f0587(f608818). Scene81/replay85 unificam controles independentes por Player,
Sweep por curva e caixas chanfradas com editor/meshes/plantas/ambiente/cunha.
Gates históricos77/76/75/66/73/74 e anteriores, layout3/manifesto1 preservados.
Snapshot conserva mesh_selection+controls; replay exige controles E meshes.
Sem rede/autenticação remota/escala1000 ou promoção das pesquisas opcionais.

Fonte/executável d67af2172b203466 conferidos:1184testes Rust/112alvos release/
all-targets,zero falhas/ignorados;Clippy-Dwarnings/fmt/diff/py_compile/build e
probe UI1440/1000/900/800 aprovados.105cenários MCP/449grupos completos headless
passaram. Novo caso combinado exercita dois Player com handles, Sweep em lista,
chanfro/preview, ambiente, seleção/captura/editor, controls+meshes replay/PNG
e load invalidando handle. Quatro recortes900×575pixels legados RGB exatos.

Sete estudos/receitas passaram: materiais/metal/ambiente3+3+3,lookdev3,normais4,
união3 e autoria15checks. A casa une bronze corrigido460a407 e stone-blocks/paving
f608818, preservando todos os outros dados; receita v5 incorpora parâmetros
de relevo e o novo hash8f8e87ee…f20af.32596v/54728t/55948draw,172v livres,
16slots cheios. Autoria reproduz fonte inteira exceto versão. Prova de união
usa ambas origens como entradas independentes, topologia e capturas repetidas;
a prova histórica de frisos conserva seus hashes e não foi relaxada para a casa
chanfrada. Variante da mola[1,9] valida normal jump0 sem alterar fonte canônica.

Sem falhas de validação nesta rodada. Conflitos reconciliam constantes/runner,
snapshot/replay, enums/admissão e deltas de conteúdo independentes. Nenhum
validador/asserção removido. Evidências em artifacts/editor-integration, novo
editor_cottage_union_probe.py e artifacts/editor-cottage-union; relatório MCP
corrente completo. Um release18095552B/cache~2,1GiB, instalação/wrappers por
symlink. Processo39326/cena pessoal preservados, sem janela/entrada controlada,
sem main/push/instalador/cópia de binário ou mudança das worktrees de origem.

A avaliação humana posterior considera o design ainda insuficiente e reportou
espinhos ao arrastar Volume. Isso é pendência funcional EAF-110, não uma falha
resolvida por esta integração. Próximo trabalho neste chat é diagnosticar/corrigir
o traço antes da proposta de redesign; novos controles visuais continuam backlog.


## Resultado da rodada — 28/10/2026, pontas capturadas às20h03 BRT

Preflight7662d3b; merges51e849e(main6bfbc5c),2731d89(malhasc5a6a13) e
b1f38a2(plantas1054577). Scene80/replay84 reúnem cunhas, ambiente e normais
avaliadas com todas as capacidades anteriores; gates históricos75/76/66/73/74
e demais preservados, layout3/manifesto1. EAF-104/115/116/119/113 consultadas.
RenderBox.geometry convive com canais materiais neutros nos vértices da cunha;
Light::validate é compartilhado com work_light e migra defaults omitidos.
Fixture combinada cobre volume sólido/vazio, normais puras, ambiente/override,
mesmo viewport em pixels, persistência de luz/layout, geração e replay.

Build862798f32d7d2aca conferido contra fonte/executável:1163testes Rust/111alvos
release/all-targets, zero falhas/ignorados; Clippy all-targets-Dwarnings,fmt,
diff,py_compile e probe UI1440/1000/900/800 aprovados. MCP completo103cenários/
434grupos completed/passed/native_window:false. Quatro recortes legados
900×575pixels cada (x0,y0,x1,y1=0,125,900,700) têm zero pixels RGB alterados.
As dimensões aqui são explícitas para não interpretar o último par como largura/altura.
Probe UI é entrada sintética/GPU offscreen, não teclado/foco/aceite humano.

Provas de materiais/metal/ambiente3+3+3mantêm hashes de pixels antigos;
lookdev3, diagnóstico de normais4, comparação estrutural e autoria14checks
aprovados. A receita lookdev agora vincula explicitamente a fonte1054577
(SHAa742696f…57e6,31956v/53448t/54668t desenhados); baseline7286…f1e2c
preservada. Carved-stone permanece exato àv3; diagnóstico repetido mede salto
52.6037°→0apenas na variante de contato, +72v/+148t, sem mudar cena canônica.
Autoria reproduz fonte inteira exceto normalização da versão; sem aceite artístico.

Falhas iniciais: módulo repetido e canais Vertex novos ausentes no trecho da
cunha, corrigidos com campos neutros legados; probe usava Pillow indisponível,
substituído pelo decoder PNG existente; comparação de luz do arquivo precisava
normalização f32 exata antes da igualdade com inspect. Nenhum validador/checagem
de pixels foi removido ou relaxado. Logs/first-failure/build-audit/validation.json
em artifacts/editor-integration; relatório MCP corrente completo.

Release único17.943.104B/cache~2,1GiB; instalação/wrappers são symlinks do mesmo
arquivo. Processo81744 e cenas pessoais preservados, sem janela controlada ou
reiniciada. Fetch final exit0 e inventário sem pontas pendentes na captura final.
Origens/dirties/main preservados, sem push/cópia binária/instaladores. UI dos
novos campos/comandos continua backlog: merge do núcleo não conclui controles
de cunhas/ambiente/normais, demais ferramentas ou aceite da nova apresentação.


## EAF-104 — integração de apoio, Sweep e materiais,28/10/2026

Pontas congeladas efcc538/b33a76c/14636ba integradas em af84e63/ff16220/4d18e4a,
checklist7da68a0. Contratos reunidos Scene78/replay82 e gates históricos mantidos,
layout3/manifesto1/UI preservados. MMOMeta1000 é estudo, sem capacidade entregue.
Provas antigas de materiais passam a usar cottage-material-baseline.scene.json,
SHA7286…f1e2c; composição atual d031a8…7c05 permanece separada e reproduzível.

Build5b466fd1c0faa1bd auditado contra fonte/binário:1.142testes Rust/109alvos
release/all-targets, zero falhas/ignorados; Clippy all-targets -Dwarnings,
fmt/diff/py_compile/build aprovados. Novo caso combinado no cenário editor:
consulta de apoio pura, plano/preview Sweep, materiais secundário+metálico,
picking de superfície, undo/redo, layout3, save/load/geradores/replay82 e PNGs
vivo/replay exatos. MCP completo100cenários/420grupos completed/passed, sem janela.
Probe Graphics/Service final em1440/1000/900/800 aprovado. Legado de775e344
mantém4regiões da viewport de630000pixels cada byte-exatas. Provas materiais3+3
e autoria da casa11checks aprovadas; geometria/cena inteira reproduzida igual,
exceto versão normalizada. Comparação estrutural reabre ambas com câmera/luz
fixas e confirma hash de hero original; sem avaliação artística positiva inferida.

Primeira suíte encontrou teste de apoio com SCENE_VERSION−1, incorreto para gate
histórico após união. Refeito com65/rejeição específica e66/aceitação, sem mudar
runtime. Removido import sem uso apontado por Clippy;19testes de apoio e Clippy
repetidos passaram. Primeiro erro preservado e resultado final consolidado em
artifacts/editor-integration/validation.json, logs tests/clippy/ui-probe/mcp-full/
cottage-*.log, first-failure.txt e legacy-pixels.json. Aviso futuro block0.1.6
preexistente, separado dos gates.

Executável corrente17.807.856bytes/cache~2,1GiB, app e wrappers usam symlinks.
Sem janela/reinício/processo encerrado ou arquivos pessoais alterados, sem
main/push/instaladores/cópias históricas. UI de novas capacidades e aceite
nativo/humano seguem pendentes. Pontas posteriores main6bfbc5c, malhas6632348 e
plantasf0f9664 foram inventariadas para próxima rodada, fora desta evidência.


## EAF-104 — refinamento geral da interface, 28/10/2026

`codex/editor` desde 570781f; build `adea604e9a6ffdd4`, fonte/hash conferidos.
Tokens e helpers compartilhados, barra contextual agrupada, projeto compacto,
abas neutras com foco/hover, seleção de linha inteira, propriedades em seções e
campos adaptativos. Iluminação em popover de formulário com trilhos visíveis.
Cor só atualiza o draft por interação, evitando falsa edição por conversão do
picker. Sem mudança de pincéis, autoria, dispatcher, renderização da cena ou
Scene 75/Replay 79/layout 3/manifesto 1. Auditoria, decisões, checklist e relatório
de revisão em [editor-ui.md](editor-ui.md).

1.109 testes Rust/106 alvos release/all-targets, zero falhas/ignorados;
Clippy all-targets -D warnings, fmt/diff e build aprovados. Regressão MCP completa
96 cenários/397 grupos, completed/passed/native_window:false. Teste de cor em
repouso novo; `editor_ui_probe` usa Graphics/Service reais e entrada sintética,
confere toolbar/XYZ em 1440/1000/800, projeto em 900×600, popover persistente e
invariância da cena. Capturas GPU finais editor-design-wide/light/1000/800/project
em artifacts, logs editor-design-*.log e consolidado editor-design-validation.json.
Não é prova de teclado/foco físico, leitor de tela ou aceite humano da nova UI.
Guilherme confirmou funcionamento da escultura anterior e pediu este refinamento.

Release único 17.705.216 bytes/cache ~2 GiB, instalação e wrappers ativos apontam
ao mesmo executável. Nenhuma janela aberta/controlada ou processo interrompido
durante regressões; cenas e sessões do usuário preservadas. A nova apresentação
requer nova abertura do app; janela atual conserva a versão em execução. Sem
main/push, cópias binárias ou instaladores. Aviso futuro block 0.1.6 preexistente.


## EAF-110 — correção de força e iluminação de trabalho,28/10/2026

Build9c80622307cd2659; codex/editor desde5798717. Layout3 inclui pincel/raio/forças
e override de luz de trabalho; migração de layouts1/2. Scene75/replay79/manifesto1
preservados. Volume expõe metros reais por amostra, Mover proporção do arraste,
Suavizar blend; configuração compartilhada UI/MCP. Luz de trabalho lateral com
ambiente0.08 e sombra1024/bias0.002, controles e restauração da luz autoral.

Inspeção GPU encontrou acne com bias0.0004; comparações até0.003 determinaram
padrão0.002. Conferência nativa final verificou menu persistente, sombras off/on
e fechamento sobre a superfície sem alterar geometria/luz autoral (arquivo
exato). Raio0.6/força0.12 digitados em campos reais e um arrasto alterou87vértices,
deslocamento máximo0.386m; um undo/redo e Save restauraram documentos exatos.
Seleção de Mover/Suavizar mostra percentuais próprios; teste extra de salvar
layout pela UI interrompido quando usuário retomou a janela. Persistência e
migração verificadas separadamente no dispatcher/MCP. Evidências em
artifacts/editor-light-native.jpg, editor-light-native-audit.json e editor-light-*;
contrato/checklist em editor-sculpt.md/performance.md. Aceite humano permanece
aberto após a correção; outros pincéis/simetria/máscaras/remesh ainda pendentes.


Gates finais do mesmo fonte/build:1108testes Rust/105alvos release/all-targets,
zero falhas/ignorados; Clippy all-targets -Dwarnings, fmt/diff e sintaxePython
aprovados. MCP completo96cenários/397grupos,completed/passed/native_window:false,
incluindo editor/render/replay/seleção/projetos e todas as frentes integradas.
Quatro regressões Rust novas: força/raio reais no gesto, consumo de clique de
popup, persistência/invariância e migração1/2 sem os campos novos. Dois novos
grupos MCP cobrem configuração/atomicidade e pixels sob luz/sombras distintas.
Arquivo é salvo antes das capturas para não comparar o diálogo de boas-vindas.

Um release corrente17.655.504B/cache~2,0GiB, app instalado ligado ao mesmo
executável. Wrappers de conferência usam symlinks, sem cópia binária. Sessões
anteriores e wrappers ativos preservados quando o usuário retomou a janela;
remover wrappers artifacts/Ignis Sculpt Check.app e Ignis Sculpt Review.app
somente depois de inativos. Cena de revisão aberta na versão final; trabalho
original do usuário preservado. Sem main/push. Artefato consolidado:
artifacts/editor-light-validation.json. Aviso futuro block0.1.6 preexistente.

## EAF-110 — malhas editáveis e pincéis na superfície,28/10/2026

Build5fcf3f9b1b05dc9c, fonte/Cargo.lock/build.rs conferidos. codex/editor desde
626cf84; Scene75/replay79/manifesto1/layout2 mantidos, sem alteração de main/push.
Criação de esfera/cubo/cilindro/plano, picking triangular e seleção de sessão
pelo dispatcher/MCP; propriedades/material/subdivisão/duplicação/exclusão na UI.
Volume/Mover/Suavizar têm raio/intensidade, cursor projetado na superfície,
prévia local, confirmação única e cancelamento por Esc/foco/revisão. Fontes,
rig e animação não são convertidos implicitamente. Contrato, alternativas,
checklist antes/depois e custos mantidos em editor-sculpt.md/performance.md.

Gates: cargo test --locked --release --all-targets1104testes/105alvos, zero
falhas/ignorados; cargo clippy --locked --release --all-targets -- -D warnings,
fmt/diff/sintaxePython aprovados. Aviso futuro block0.1.6 preexistente separado.
MCP completo96cenários/395grupos,completed/passed/native_window:false. Novo
editor_mesh cobre primitivas/erro atômico/picking, equivalência preview/commit,
undo/redo, captura repetida, save/load e replay. Cinco regressões de entrada UI
cobrem três pincéis, redraw sem novos carimbos, gesto rápido, cancelamento e
mouse-up com salvar no mesmo frame; o arquivo deve conter o traço concluído.

Conferência nativa isolada criou esfera pelo menu e selecionou por hierarquia/
superfície. Repetição dos três pincéis no build final alterou28/34/45vértices;
⌘S persistiu e ⌘Z/⌘ShiftZ restauraram documentos exatos. Evidências em
artifacts/editor-sculpt-native.jpg, editor-sculpt-native-verification.json,
editor-sculpt-validation.json, arquivos de cena e logs editor-sculpt-*.
A captura repetida inicial diferia pelo diálogo de boas-vindas; cenário salva
a cena antes de capturar. Gesto rápido e ordenação soltar/salvar receberam
regressões específicas. Capturas/headless não substituem aceite humano.

Um executável release corrente17.554.944B, instalado por symlink/samefile;
cache exclusivo~2,0GiB. Janela/wrapper temporários encerrados/removidos sem
copiar binário. Sessão do usuário preservada. Remesh/dyntopo/simetria/máscaras,
tablet e pincéis adicionais, rig/animação visual e colisão por malha continuam
recortes próprios. EAF-110 permanece In Progress aguardando avaliação humana.


## EAF-109 — projetos, cenas e objetos no editor,28/10/2026

Build5b74972fb0197436, fonte/Cargo.lock/build.rs conferidos; Scene75/replay79
inalterados, manifesto de projeto1 e layout2 com migração de1. Worktree editor,
branch codex/editor desde497d36c; sem main/push. Contrato/checklist antes e
revisão posterior em projects.md/performance.md/architecture.md.

Validação: cargo test --locked --release --all-targets1092testes/104alvos, zero
falhas/ignorados; cargo clippy --locked --release --all-targets -- -D warnings,
fmt/diff/sintaxePython aprovados. Aviso futuro block0.1.6 preexistente separado.
MCP completo95cenários/392grupos,completed/passed/native_window:false. Novo
projects cobre projeto real/manifesto, piso+jogador, três tipos de objeto e
duplicação/undo, dirty e erros atômicos, múltiplas cenas/entrada, captura/replay
e reabertura em outro processo. Cinco testes Rust de projeto, incluindo
symlink/rollback/collision e layout1→2. Logs estáveis artifacts/editor-project-*.

UI nativa do mesmo executável exercitada: formulário com nome/pasta, criação
de projeto/bloco, Principal→Floresta com Salvar e continuar, painel de cenas e
retorno a Principal com bloco salvo. Campo de pasta corrigido para conservar
o texto entre frames. Executar, foco de viewport, W400ms, Esc e pausa observados:
Zruntime−1,65/tick98, Zautoral0. Fechar após nova criação abriu guarda de dirty;
Salvar e continuar persistiu e encerrou. Arquivos reais conferidos na pasta de
verificação, dois documentos e objetos visual/fixo. PNGs editor-project-native.png
e editor-project-close.png e metadados editor-project-native.json.

App instalado atualizado com entrada de projetos; executável único17.371.216B,
cache exclusivo corrente~2,0GiB, samefile confirmado. Ferramenta de UI selecionava
a instância antiga, então wrapper temporário do mesmo executável isolou o teste;
wrapper e janela de verificação removidos/encerrados depois, sem cópia de binário.
Sessão original12548 preservada com a caixa do usuário; instância atualizada17351
aberta. Uma tentativa CLI auxiliar também encerrada, sem autoria do usuário.
Nenhuma outra worktree/app/processo alterado. Aceite humano e ferramentas restantes
de NPC/modelagem/exportação continuam pendentes; EAF-109 pronto para avaliação.


## EAF-104 — atualização horária,28/10/2026 00h35 BRT

Somente ponta8a5e664 de pesquisa gráfica nova, integrada por dc0ae96 desde
2d13274 inicialmente limpo. Cwd/branch e ausência de operação/build conferidos;
fetch/inventário completo, demais pontas já ancestrais. Checklist anterior
5ce8ae4, EAF-104/88 lidas e104revisitada no checkpoint. Documentação reconciliada
preservando histórico; IGP1/projeção/handles somente em harness opt-in.

Gates novos:1087testes Rust/103alvos release/all-targets, zero falhas/ignorados;
Clippy-Dwarnings (aviso futuro block0.1.6 preexistente), fmt/diff, sintaxePython
completa e node --check do novo JS aprovados. MCP headless proporcional:
editor,lab,render,replay_inspection,selection;5cenários/30grupos completed/passed,
native_window:false. Editor preserva1878pixels de malha/45.486de painel,
captura pura/undo RGB exato e replay junto de população ponderada.

Justificativa do recorte: diff src/Cargo.lock/build.rs vazio, nenhum runtime,
renderer funcional, schema/Command ou novo cenário MCP. Mesmo executável/build
c2d5bb39a0526e01,Scene75/replay79. A suíte completa94/388 aprovada em2d13274
permanece evidência anterior do mesmo fonte funcional, sem execução nova
alegada. Benchmarks científicos não reexecutados; falhas/custos documentados
na frente de origem, sem promoção ou alegação de FPS/aceite humano.

Logs estáveis artifacts/editor-integration/rust-tests.log,clippy.log,
mcp-targeted.log,build-audit.json/coverage.json/inputs.json. Report.json corrente
em artifacts/mcp-validation é o recorte de5cenários; mcp-full.log conserva a
rodada anterior. Todas as pontas capturadas ancestrais. Release17.268.544bytes,
app instalado samefile, único target externo corrente (~1,9GiB); sem cópias ou
instaladores novos, nenhum app/processo do usuário reaberto/interrompido.
Origens/alterações em andamento/main preservados, sem push. Ferramentas visuais
continuam pendentes; EAF-104 In Progress. Atualização de laboratório não conclui
UI nem retoma EAF-107 cancelada. Próximas pontas ficam para outra rodada.


## EAF-104 — atualização horária do editor,27/10/2026 23h32 BRT

Base ec3ccc6 limpa, cwd/branch codex/editor conferidos, sem operação/build ativo;
fetch/inventário de todos os refs concluídos. Pontas recebidas: main6b1ab5e
(EAF-108)→ea0cbc1; imagem→3Dd0da3b0→d3ed109; pesquisa gráfica7696d8d
(EAF-88)→75ba58a. Malhas31944f7, plantas6333222 e remotos já ancestrais.
Requisitos/comentários EAF-104/108/88/107 lidos,104revisitada no checkpoint.
Checklist anterior62fbed8/revisão posterior em performance.md.

Cena75/replay79 reúnem a nova autoria/estado de seleção ponderada e as frentes
anteriores. Gate65 de template_selection,64regional,72conexões e demais guardas
históricas mantidos; versões correntes dos testes ajustadas, negativos preservados.
Constants de versão e expected_meshes/expected_generators do replay conservados.
Auditoria das cinco pontas em coverage.json:99Command/12SceneEdit/20MeshEdit/
6PlantEdit e94cenários preservados, todos os commits ancestrais.

Seleção mantém pending/binding nas falhas, stream próprio e template em membros/
recibos; tabelas compiladas até16candidatos, pool8 mantido. Editor testa primeira
onda ponderada junto de plantas/malha e painel real, confirma draws/nascimentos,
replay dos dois domínios e undo exato. IGT1/primitive_index permanecem no harness
opt-in, sem requisitos/buffers/shader experimentais no renderer funcional.
EAF-107 cancelada por Guilherme: receitas, documentos/cena/histórico preservados;
nenhuma nova tentativa ou melhoria de fidelidade do personagem/E02 executada.

Gates do mesmo buildc2d5bb39a0526e01 (macOS arm64):

- cargo test --locked --release --all-targets:1084testes/103alvos, zero falhas
  ou ignorados. Inclui seleção ponderada e codec/projeção de tangentes do harness.
- cargo clippy --locked --release --all-targets -- -D warnings:aprovado;
  aviso futuro preexistente block0.1.6 separado de warnings do projeto.
- fmt/diff, sintaxe de todos os scripts Python e node --check do novo JS:
  aprovados. scripts/test_reconstruction_recipe.py:5testes numéricos aprovados.
- MCP completo:94cenários/388grupos,completed/passed/native_window:false;
  template_selection4grupos,editor6,render/replay/seleção/conexões/painéis/população.
- Editor conserva1878pixels da malha e45.486do painel na viewport, captura pura
  e undo RGB exato. Nascimentos e template_draws positivos; replay combinado
  confere meshes/geradores. Captura editor-combined-live.png aberta/inspecionada.

Logs/auditorias estáveis em artifacts/editor-integration; relatório completo em
artifacts/mcp-validation/report.json e JSON/PNGs na sua pasta artifacts.
Build ID conferido contra fontes/Cargo.lock/build.rs antes da regressão.
Executável17.268.544bytes, único target corrente externo (~1,9GiB), app instalado
samefile; sem cópias/instaladores/histórico novos. Worktrees de origem/mudanças
em andamento/main e janelas/processos do usuário preservados; sem push.
Benchmarks científicos não repetidos nem resultados negativos apagados. UI de
autoria de templates ponderados/demais ferramentas e entrada física/aceite humano
permanecem pendentes; EAF-104 In Progress. Próximas pontas em outra rodada.


## EAF-104 — atualização horária do editor,27/10/2026 22h32 BRT

Base45671ce inicialmente limpa, sem merge/build em andamento; cwd e branch
codex/editor conferidos. Fetch e inventário de todos os refs. Painéis sólidos
EAF-106 imagem→3D87273b8→merge37563c2; footprint analítico EAF-88
d756fb1→merge2252c21. Main b18c837, malhas31944f7, plantas6333222 e remotos
já ancestrais. Requisitos/comentários EAF-104/106/88 consultados; checklist
anterior em9a0827f, revisão posterior em performance.md.

Cena74/replay78 preservados: Panel persiste Geometry existente, sem novo campo
autoral ou agenda de simulação. Documentação e módulos reconciliados sem remover
histórico. Runner conserva a união; auditoria coverage.json das cinco pontas
confirma ancestralidade e99Command/12SceneEdit/20MeshEdit/6PlantEdit preservados.
Fontes vinculadas recusam Panel; conexões/máscara/material e geradores mantidos.
Pesquisa permanece opt-in, com resultados negativos documentados; nenhum shader
experimental promovido para o renderer funcional.

Gates finais do build3a33e1ba4931d5bc (macOS arm64):

- cargo test --locked --release --all-targets:1068testes/102alvos, zero falhas
  ou ignorados. Inclui quatro testes novos de painel e fórmula analítica do harness.
- cargo clippy --locked --release --all-targets -- -D warnings:aprovado;
  aviso futuro preexistente do pacote block0.1.6 permanece separado.
- cargo fmt --all -- --check, git diff --check, sintaxe de todos os scripts
  Python e node --check do novo script JS:aprovados.
- Regressão completa MCP:93cenários/384grupos,completed/passed/native_window:false;
  inclui mesh_panels(8grupos),editor(6),render/replay/seleção/conexões/população.
- Editor:1878pixels da malha anterior e45.486pixels do painel, somente ROI da
  viewport; captura pura, repetição e undo RGB exatos. Gerador nasce e replay
  combinado confere meshes+geradores. Captura editor-panel-visible.png aberta e
  inspecionada:espessura/vão visíveis ao lado da planta na cena funcional.

Logs/auditorias correntes em artifacts/editor-integration; relatório completo
artifacts/mcp-validation/report.json, evidências PNG/JSON em sua pasta artifacts.
Fonte/Cargo.lock/build.rs conferidos com build_id. Um executável corrente
17.150.784bytes, target externo exclusivo (~1,9GiB), app instalado samefile;
nenhuma cópia/instalador/histórico criado, nenhum app reiniciado/controlado.
Origens/alterações em andamento/main preservados; sem push. Benchmarks científicos
não reexecutados: suas falhas/gates permanecem no estudo de origem. Entrada
nativa/aceite humano e ferramentas UI de autoria de painéis continuam pendentes;
EAF-104 permanece In Progress. Próximos commits entram na próxima atualização.


## EAF-104 — atualização horária do editor, 27/10/2026 21h32 BRT

Destino codex/editor encontrado limpo em f883d5c, sem merge/build/regressão em andamento. Fetch e inventário: main b18c837, native-meshes31944f7 e remotos já ancestrais. Novas pontas capturadas image-to-3dc75c474, native-plants6333222 e graphics-research6bcd31b incorporadas por11bf178/029e3e4/ad597f0. Inclui conexões regeneráveis, refinamento/segundo enxerto, núcleo estrutural/E02 e diagnóstico de repetibilidade de textura. Scene74/replay78 preservam os gates históricos72 de conexões e64 regional; custos/contrato em editor-integration.md/performance.md. Experimentos continuam explícitos.

- `cargo test --locked --release --all-targets`: **1063testes/101alvos aprovados**, zero falhas/ignorados. Inclui2testes de conexões com pureza/atomicidade/dependências/cache/CG/persistência/máscara e invariância de Frame. Clippy release/all-targets com-Dwarnings, fmt/diff e sintaxe de todos os scriptsPython aprovados. Aviso futuro preexistente de block0.1.6 separado.
- Regressão MCP completa: **92cenários/375grupos**, completed/passed/native_window:false. Novo plant_connections cobre composição, consumo seletivo/descendentes, previews/erros/undo/load/reset/replay/prefixos/pixels; editor/render/seleção e demais cenários mantidos. Compositor do editor conserva1878pixels da instância e retorno RGB exato no recorte da viewport, sem confundir texto/revisão com geometria.
- Auditoria preserva99Command/12SceneEdit e todos os cenários das cinco pontas capturadas; todas ancestrais. Artefatos estáveis em `artifacts/editor-integration/{inputs,coverage,build-audit}.json`, rust-tests.log/clippy.log/mcp-full.log; relatório completo artifacts/mcp-validation/report.json. Editor e conexões em artifacts/mcp-validation/artifacts/*verification.json.

Build `aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|36ef7a1e39e30544` confere com o hash das fontes/Cargo.lock/build.rs. Um executável corrente17.166.752bytes e cache release~1,9GiB no target externo exclusivo do editor; app instalado mantém a referência ao mesmo binário, sem cópia/instalador/histórico novo. Worktrees/alterações de origem preservadas; nenhuma janela/processo do usuário controlados, push ou alteração de main. Benchmarks científicos, entrada nativa e aceite humano não inferidos. UI específica das capacidades recebidas segue pendente; EAF-104 permanece In Progress. Novos commits após as pontas desta rodada ficam para a próxima checagem horária.

## EAF-104 — integração das branches no editor, 27/10/2026

Rodada fechada em `codex/editor`, incluindo main b18c837, image-to-3d0e7253a, native-meshes31944f7, native-plants c305ded e graphics-research003f251. Merges584b5b1/dab6f00/4925755/6e679f4 e75dd17b; reconciliação b37dcf7. Contratos e política recorrente em [editor-integration.md](editor-integration.md). Scene73/replay77 conservam geradores/ondas/ativação/regiões, malhas/plantas/materiais e suas guardas históricas; estado de editor segue separado. A pesquisa científica permanece explícita nos seus executáveis.

- `cargo test --locked --release --all-targets`: **1061 testes/100alvos aprovados**, zero falhas/ignorados, incluindo testes dos executáveis de pesquisa e integração planta/malha+geração/replay. `cargo clippy --locked --release --all-targets -- -D warnings`, fmt/diff e sintaxe de todos os scriptsPython aprovados. Aviso futuro de block0.1.6 preexistente, fora dos warnings do projeto.
- MCP completo: **91cenários/370grupos**, completed/passed/native_window:false. Novo gerador regional e todos os cenários de cada origem preservados; editor inclui malha/planta real, gerador ativo, pureza da captura e replay dos dois domínios.
- Revisão visual do cenário combinado motivou uma fixture maior/contrastante. Reexecutado somente editor sobre o mesmo binário:5grupos aprovados,1878pixels alterados no recorte exclusivo da viewport ao retirar a instância, igualdade RGB exata após restaurar. Evita confundir diferenças de texto/revisão com malha apresentada. Capturas editor-combined/without-mesh/restored/live em artifacts/mcp-validation/artifacts; resultado em editor-verification.json e log editor-integration/editor-visible.log. Nenhuma mudança Rust após o build final.
- Auditoria de cobertura:98Command/12SceneEdit e todos os cenários de origem preservados; cada ponta capturada é ancestral. `artifacts/editor-integration/{inputs,coverage,build-audit}.json`. Logs rust-tests.log/clippy.log/mcp-full.log no mesmo diretório; relatório completo artifacts/mcp-validation/report.json.

Build `aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|96a2c2cefbba9a91` conferido contra o hash das fontes/Cargo.lock/build.rs. Executável corrente16.965.104bytes, cache release~1,9GiB no destino exclusivo `/Volumes/Zero/dev/build-cache/ignis-editor/target`; app instalado continua apontando para esse executável, sem cópia/instalador adicional. Não houve controle de janela, push ou alteração das worktrees de origem. Benchmarks científicos não foram reexecutados; não há ganho de FPS ou aceite nativo/humano inferido.

Automação de thread **Manter Ignis Editor atualizado**, ACTIVE, horária, confirmada pelo app e configuração local. Novos commits surgidos após o fechamento desta rodada — detectados image-to-3d2c04781 e native-plants f0dad0c — ficam para a próxima execução; não fazem parte da evidência acima. UI das capacidades recebidas ainda exige implementação própria; EAF-104 permanece In Progress.

## E02 — nova construção volumétrica, sem reaproveitar a anterior

Fonte original+skill atual+schema/MCP público; contexto não é teste cego.
Receita nova `scripts/e02_volume.py`, cena `examples/house-e02-volume.scene.json`,
relatório/checklist [e02-volume.md](e02-volume.md).188tarefas entregues antes
da geometria,22recortes abertos e9revisitações; passagem interna inferida foi
acrescentada como189ºitem na revisão espacial, com histórico explícito.

Build9860a09115365ad3/Scene67/macOS/Metal offscreen, processo próprio encerrado.
15assets/18instâncias/16295v/26260t armazenados/26512t desenhados,89v livres;
budgets inalterados. Coluna nativa96v/84t por11estações×4instâncias. Corpos
ocorrem em coordenadas do objeto com paredes internas/externas, pisos e jambas;
nenhum plano definido pela câmera foi usado para reconstituir partes.

13checks aprovados em `e02-volume-verification.json`: três corpos e coluna
fechados/uma componente/orientação/volume, coberturas fechadas,17vãos atravessam
paredes, passagem atravessa os dois corpos, captura repetida/carga nova/undo
exatos, replay estático tick0 matched/pixels exatos e fonte preservada.
Render7522862c471de38be489ea64019383651e3ed7d61ef12705d00c1501819081b4;
cena0657baeb9d92ac498fd8f7b183409c6cf96139999fb900dee22eec2087b9fd22,
exportada byte a byte do save nativo. Nenhuma prova de colisão física,
ausência global de interseção ou aceite arquitetônico/artístico inferida.

Seis ângulos nativos e22pares de janelas fixas realmente abertos. Comparação
nativa em1402×1122 sem alinhamento:MAE0.275375/RMSE0.351458/SSIM0.275501;
não são percentual de fidelidade nem ranking contra outra câmera. Diagnósticos,
vínculos/189pendências e hashes versionados; nenhum item artístico aceito.
Fonte Unreal e cena/render antigos preservados, estes com hashes89c6aa59…94ff1
e eabff512…328bc2 respectivamente.

Corte horizontal `plan.png` extraído dos triângulos salvos depois da construção;
não é blueprint nativo prévio. Corrigida ordem de desenho2D para piso não
encobrir paredes no corte. Visualizador genérico usa a geometria nova; JS syntax,
payload, HTML/recursos, AST/JSON/links/IDs/estados/hash e diff aprovados, sem
alegar compilação WebGL/interação humana. Ensaios anteriores e demo intactos.
Nenhum kernel/build/schema/dependência instalado ou alterado; regressão Rust
não repetida para autoria cliente/conteúdo. Sem push/publicação externa.

## EAF-103 — skill e lista consolidada E02, sem construção

Skill/contratos/metadados revisados para imagem inteira→inventário→recortes→
revisitações→lista específica→construção/validação. Novo contrato
.agents/skills/ignis-image-to-3d/references/construction-list.md. Respeita a
fronteira pedida: entregar lista antes da autoria, sem aprovação genérica
adicional em execuções integrais já autorizadas.

Nova aplicação somente à imagem E02 original e skill revisada:40recortes
realmente abertos (12cobertura/16contexto/12detalhe), conjunto reaberto e
contextos C03/C05/C06/C11/C14 revisitados.10registros de revisitação vinculam
achados aos itens;248tarefas em21conjuntos, todas não construídas/validação
não executada. Lista completa em docs/e02-construction-list.md e fonte JSON
em docs/references/image-to-3d/e02-construction-list.json. Extração/publicação
reproduzíveis por scripts/e02_construction_inventory.py; artefatos estáveis
artifacts/house-e02-analysis/. Nenhuma receita/cena anterior importada como
fonte da análise; histórico da conversa permanece, sem alegar teste cego.

quick_validate.py aprovou a skill usando PyYAML já disponível em
artifacts/skill-validation/python. AST,157links,248IDs únicos,40arquivos/caixas,
sub-recortes contidos nos pais, vínculos das revisitações e estados aprovados.
Varredura cobre cada pixel do original; isso não mede completude semântica.
Hash da referência4153a456…882ad e render E02 anterior70d57370…11b8712
preservados. diff check aprovado; nenhum Rust/schema/build/processo engine,
geometria/fonte paramétrica/cena/render novo. Regressões Rust não repetidas
para documentação, dados e extração de pixels. open_in_codex da lista retornou
queued, sem alegar que o painel ficou visível ou interação UI foi testada.

## EAF-102 — fontes nativas e pranchas E02

Nova autoria em scripts/e02_native_blueprints.py, sem importar cena/receita
anteriores, com imagem+skill+interface pública. Schema atual confirmado:
sem comando blueprint/exportador2D; uso real de preview_plant/edit_plant,
inspect_plant/inspect_mesh.16fontes/31instâncias/2278v/2620t, metadados
versionados em docs/references/image-to-3d/e02-native-blueprints.json.
Cena, PNG nativo, fontes e desenhos em artifacts/house-e02-blueprints/.

Capturas repetidas e após carga com pixels exatos;16fontes confirmadas e
malhas avaliadas iguais às fontes nesta configuração. Desenhos ortográficos
derivados das mesmas malhas inspecionadas, com transformação por instância.
Pranchas fontes1920×1920 e conjunto1800×1400 abertas para conferência visual.
Quatro pares de recortes nativos comparados; SSIM global0.198308 nesta condição
não aprova fidelidade e não é A/B com o ensaio anterior. Motivos/vãos/encontros
pendentes estão discriminados no relatório. Não entregar como casa concluída.

Recusa inicial da trajetória mantida como evidência; fonte final usa13estações
paralelas deslocadas/pchip, sem relaxar kernel/limites ou enviar malha direta.
AST/índices/limites/quaternions/hash da fonte/diff conferidos. Sem código Rust,
regressão completa nova, replay/undo novo ou GUI. Cena E02 e Unreal preservadas;
bootstrap/probe descartado somente no diretório criado por esta unidade.

## EAF-102 — investigação da fidelidade E02

Auditoria em codex/image-to-3d, macOS, sem mudar cena, skill, Rust, renderer,
schema ou build. scripts/audit_e02_similarity.py executado com Pillow12.3.0
e build9860a09115365ad3, em processo MCP próprio headless. Três comparações:
identidade exata; detalhe interno da porta apagado com SSIM global0.993235 e
local0.183477; medalhão apagado com global0.987101 e local0.115433. Ambas as
perdas passam por três cortes globais ingênuos0.98. Fixtures são fotos alteradas
para testar o avaliador, não renders/candidatos3D. Relatório versionado em
docs/references/image-to-3d/e02-fidelity-audit.json; conclusões/arquitetura
proposta em docs/e02-fidelity-investigation.md.

Cinco checks do relatório aprovados: hash da fonte, MAE conferido de forma
independente, controle identidade, ROIs apagadas constantes e contraprovas dos
cortes. SSIM usa somente implementação nativa. Recontagem da receita:6104v
diretos em mass e16138v em detail, mais80v da coluna nativa. Revisão confirma
17fichas divergentes+1inconclusiva e18ações pendentes. AST do script,10links do
documento e diff check aprovados. Hashes da referência/render anteriores
preservados e export nativo idêntico a examples/house-e02.scene.json.

Artefatos estáveis em artifacts/e02-fidelity-audit/. Sem render/GPU/janela,
regressão Rust ou UI nova; nenhum solver, gate automático, limiar geométrico
calibrado ou ganho de fidelidade implementado nesta investigação. EAF-102
continua sem aceite visual da reconstrução.


## EAF-99 — comparação determinística de imagens e primeiro estudo da casa

Implementação na branch `codex/image-to-3d`, a partir de `fa02650`, macOS
Apple Silicon/Metal headless. Novo Command::CompareImages atendido pelo
Service/MCP comum; núcleo CPU independente de render/I/O, PNG e buffers
limitados, RGB/SSIM/ROI/máscara e mapa de diferença. Cena67/replay71 mantidos.

**867testes Rust em82grupos, zero falhas/ignorados**, incluindo8novos testes
focados; cargo test --locked --release --all-targets, Clippy release/all-targets
-Dwarnings, fmt, sintaxe Python e diff aprovados. SSIM comparado contra
referência direta11×11 com média/covariância por desvios, independente das
somas deslizantes. Constantes analíticas, padrão deslocado com SSIM negativo,
ROI, máscara, suporte ausente, alfa/profundidade/paleta/cinza, referências e
saídas/aliases/revisão verificadas. Aviso futuro preexistente block0.1.6 é
distinto dos warnings do projeto.

**83cenários MCP/339verificações** completos aprovados, native_window:false.
Novo cenário image_similarity compara capturas GPU repetidas e replay, detecta
mudança de câmera, explicita erro local escondido pela média global e recusa
casos inválidos sem alterar cena/estado/histórico/entradas. Build
`9860a09115365ad3`;65summaries informam esseID,18legados omitem o campo.
Auditoria em artifacts/image-similarity/build-audit.json; logs tests/rust-full/
clippy/mcp-focused/mcp-full/house-demo em artifacts/image-similarity/.
Relatório completo do runner em artifacts/mcp-validation/report.json.

A demonstração usa a referência original em1536×1024 e um estudo de volumes
criado pelo MCP nativo. Fonte corrente examples/house-blockout.scene.json;
reprodução scripts/demo_house_similarity.py. Resultado em
artifacts/house-similarity/index.html, report.json e artifacts/{reference,
blockout,camera-shift,difference}.png. Câmera, vidro/pedra, cobertura/escada/
varanda/colunas e chaminé são hipóteses manuais; sem detalhes, segmentação,
calibração/ajuste automático, interior/verso ou fechamento físico.

Na versão demonstrada, MAE global base0.191676/SSIM0.388609; outra câmera
mantendo geometria dá MAE0.193302/SSIM0.390464. O ranking discorda: não escolher
câmera correta ou declarar38.9%de fidelidade3D por essas medidas. O próximo
recorte exige correspondências/contornos e critérios por etapa; métricas
atuais oferecem observação e localização de erro, não reconstrução automática.

Build corrente único reaproveitado em
/Volumes/Zero/dev/games/.ignis-build/image-to-3d/release/ignis; nenhum pacote,
instalador, cópia histórica de binário, janela ou output de outra worktree.
Logs/mapas têm nomes estáveis por cenário/ensaio; a tentativa inicial do estudo
foi substituída no mesmo destino. Sem push/publicação ou aceite artístico
inferido. Contrato/alternativas/checklist em image-similarity.md/performance.md.


## EAF-98 — integração de autoria e população para imagem em 3D

Branch `codex/image-to-3d`, worktree própria, base main local `f6b707e`,
meshes `7daee87` e plants `2ba75eb`. Primeira união em `c890661`; segunda
união reúne plantas e contratos funcionais em cena67/replay71. Não modifica
main, worktrees de origem ou pesquisa gráfica; nenhuma janela manipulada.
Referência original da casa preservada em `docs/references/image-to-3d/elven-house.png`,
SHA256 `209cd5cb91c216100413d6b175b5ce29a70cab751a0eda710e4baf95895c6553`.

Validação final em macOS Apple Silicon/Metal, build `2bdb4835a853cc6e`:
**859 testes Rust em81grupos, zero falhas/ignorados**, com
`cargo test --locked --release --all-targets`; Clippy release/all-targets
com `-D warnings`, fmt, sintaxe Python e diff aprovados. Aviso futuro da
dependência preexistente block0.1.6 separado dos warnings do projeto.

Regressão completa pelo runner: **82cenários MCP/335verificações**,
`completed/passed`, `native_window:false`. Todos os cenários de ambas as
frentes preservados, inclusive generadores/ondas/controle/ativação espacial,
meshes/armazenamento/plantas/trajetórias/adoção/composição, render e replay.
Os64summaries que informam build_id confirmam o mesmo build;18legados omitem
o campo. Auditoria em `artifacts/integration/build-audit.json`, relatório em
`artifacts/mcp-validation/report.json`. Logs Rust/MCP/Clippy em
`artifacts/integration/{rust-tests,mcp-full,clippy}.log`.

Novo teste conjunto demonstra planta curva e gerador por trigger no mesmo
mundo, nascimento real, roundtrip de cena/replay, prefixo e detecção separada
de adulteração de poses e produção. `matched` depende dos dois resultados;
formato anterior de replay recusa. Guardas históricas por capacidade e testes
de versões antigas mantidos; expectativas de versão corrente atualizadas.
Nenhuma implementação de reconstrução de imagem ou aceite visual da casa
é alegada.

Build corrente exclusivo em
`/Volumes/Zero/dev/games/.ignis-build/image-to-3d/release/ignis`, sem cópia de
app/instalador. `CARGO_TARGET_DIR` estável por worktree; nenhum output das
outras frentes alterado ou removido. Esta unidade não criou builds históricos.
Fontes/contratos e fronteiras: `docs/image-to-3d-integration.md`.

## Lanterna da casa editada como conjunto de peças

Receita6, mesma Scene72: helper cottage_composition usa inspect_mesh_islands e
transform_islands para9ilhas bronze/1flowers no mesmo lote. Estudo9grupos e
autoria completa16checks aprovados; só140posições alteradas, contagens e demais
assets/instâncias/plantas/câmera/luz/rig/clips/material preservados. Reconstrução
desde receita coincide com a edição da cena anterior e publicação compacta em
semântica/pixels. Provas/limites/checklist em [cottage-composition.md](cottage-composition.md),
artefatos estáveis artifacts/cottage-composition/artifacts/report.json e build-audit.json.

## Transformação local de ilhas — EAF-125

Build macOS Apple Silicon `c9ffc6d3852bd23d`, Scene72/replay76 mantidos.
`cargo test --locked --release --all-targets`:863testes/85targets, incluindo
3novos em mesh_islands (10no target). Clippy release/all-targets `-D warnings`,
fmt e build aprovados. Logs estáveis `artifacts/island-transform-{tests,clippy,build}.log`.
Regressão completa `python3 scripts/run_verifications.py`:88cenários/363grupos
headless;5grupos novos no cenário mesh_island_transform. Relatório
`artifacts/mcp-validation/report.json` e log `artifacts/island-transform-full-mcp.log`.

Analítico pivô/rotação/escala, seleção de ilhas coincidentes por índices distintos,
isolados, material implícito/explícito, rig/pose/Arcs, erros finitos e colapsof32.
MCP cobre schema/preview/commit, stale/seleção/lote atômicos, proteção de planta,
undo/redo, GPU, animação, load/reset/replay/prefixo/viewport. Nenhum teste de
controle nativo/UI ou aceite artístico inferido. Contrato e checklist em
[mesh-island-transform.md](mesh-island-transform.md).

## Segundo enxerto estático na mesma árvore

Receita native-tree-secondary02.graft.json e demo_tree_second_graft.py usam o
ContextGraft existente:8opções/5recusas, nova união fechada/conectada, sem mudança
Rust/formato/teto. Mesh15653v31302t,95416t desenhados,32instâncias. Contexto/source/
ref/def/rig/clipes e montagem anterior intactos;13669pontos retidos anteriores
exatos,160substituídos no recorte explícito e91retidos da fonte estática, com
erro f32≤0,119µm no mundo.3004pontos/triângulos/canais do encaixe aprovado e PNG
clay exatos. Demais poses/folhas/rametas exatas0/60/240; oito poses de faces novas
cosmín1. Sem prova global de embedding ou C1 na fonte de7lados.

Previews/lote, restauração da ilha, undo/redo, A-B-A de ambas plantas e do novo
enxerto, save/load/repeat/replay/prefixo aprovados. Atual4MCP/19grupos: render,
plant_assemblies,plant_material_frames,plant_connections. Python/fmt/diff e
auditoria de fonte/build ec96a3441fef43e2 conferem;850Rust/85MCP completos são
anteriores no mesmo binário, não reexecutados para esta autoria. Capturas frontal,
lateral, inteira e comparação do encaixe aprovado em artifacts/tree-second-graft.
Detalhes/custos/fricção de orçamento em [plant-connections.md](plant-connections.md).
Mais uma junção tratada, outras permanecem; nenhum aceite global de realismo/UI.

## Refinamento autoral do encaixe regenerável

Mesmo binário ec96a3441fef43e2 de f0dad0c; sem mudança Rust, formato ou build.
Controle e19variações reproduzíveis em native-tree-connection-shape.input.json e
study_plant_connection_shape.py; limites recusam3das12variações iniciais sem
mutação. Escolhida down080 mantém os cortes/retidos e reduz dobra máxima
75,49→40,60°, p99ponderado24,67→18,65°, área nova−2,87% e volume−0,000975m³.
Custo: mais522v/1044t, campo48160/65536pontos e108iterações CG; costura extraída
11,72→16,14° é tradeoff registrado. Costura com fonte0,0594° permanece exata.

Demo nativa com fonte original e receita refinada confirma fontes/rig/clipes/
instâncias/retidos, A-B-A das duas plantas, unlink, undo e pixels repeat/replay/
prefixo/load; poses retidas≤0,292µm, demais/descendentes exatas. Oito poses de
faces novas cosmín0,999511, sem inversão observada. Capturas clay frontal/lateral
com controle conectado; material persistido preservado. Mesh13829v27654t,
92002t desenhados. Árvore canônica e fonte de duas plantas preservadas.

Regressão headless atual: render,plant_trajectories,plant_assemblies e
plant_connections,4cenários/20grupos aprovados. Python/fmt/diff conferidos.
Os850Rust/85MCP completos da seção anterior correspondem ao mesmo binário,
não a uma nova execução nesta unidade de autoria. Fonte/build e resultados em
artifacts/connection-shape-audit.json, capturas/cena em artifacts/plant-connections.
Limites e revisão: [plant-connections.md](plant-connections.md). Dobra inferior
menos marcada, ainda com concavidade; não conclui todas as junções/realismo da
árvore, C1, embedding global ou UI.

## Conexões regeneráveis de plantas72

Build ec96a3441fef43e2, macOS Apple Silicon. 850testes Rust/85grupos release/all-targets aprovados.
Clippy-Dwarnings/fmt/diff/Python aprovados. Regressão completa headless:
85cenários MCP/345grupos; auditoria67summaries com mesmoID e18legados semcampo,
hash de src/Cargo.lock/build.rs confere. Logs connections-* e relatório agregado
artifacts/mcp-validation/report.json. Scene72/replay76 locais desta frente.

Dois testes Rust e cenário MCP cobrem fonte/caches/dependências, curvas de ambas,
refino longitudinal, transformação da instância, previews puros, erros atômicos,
component reference após unlink, consumo seletivo de desenho, undo/redo/load/
reset/replay/prefixo pixel-idênticos. Escultura/rig/clip rebase conectado recusa;
parâmetros e aparências continuam editáveis. Contrato e revisão estática em
[plant-connections.md](plant-connections.md), sem dependência ou teto ampliado.

Ensaio reproduzível demo_tree_plant_connections.py na mesma árvore: duas plantas
mantidas, receita644B, união13307v26610t fechada/conectada,27144amostras de campo,
CG96/resíduo8,65e-7; poses retidas≤0,292µm em0/60/240 e demais/descendentes exatas.
A-B-A das duas trajetórias exato, unlink restaura o pai; pixels undo/repeat/
replay/prefixo/load exatos. Capturas textured e clay e cena corrente em
artifacts/plant-connections. Colar volumoso/dobra inferior continuam visíveis:
não declarar C1, acabamento realista, árvore inteira conectada ou aceite de UI.

## Amostragem nativa e estudo de conexão entre plantas

Build24285b4b58acb464, macOS Apple Silicon:848testes Rust/84grupos release/all-targets,
Clippy-Dwarnings, fmt, diff e Python aprovados. Regressão completa headless:
84cenários MCP/340grupos. Auditoria de fonte/build:66summaries com o mesmo ID,
18legados sem ID; hash de src/Cargo.lock/build.rs coincide. Logs/auditoria
live-junction-* em artifacts. Nenhum formato Scene/replay novo.

inspect_plant_sampling verifica pureza/revisão/domínio e base distinta de
escultura; ausência de ID recusa sem mutação. Proveniência pública de união
verifica prefixos, faixa iso e cobertura única das guardas. Matemática de
geometria/render não mudou; apenas dados antes internos ficam observáveis.

Estudo reproduzível e revisão em [live-plant-junction-study.md](live-plant-junction-study.md):
junção nativa13307v26610t, fechada/conectada; atribuição explícita de pesos no
lado autor preserva partes retidas≤0,343µm e descendentes≤0,119µm nas poses0/60/240.
Normais novas positivas em8poses (cos mínimo0,999271); pixels repeat/replay/prefixo/
reset exatos. A cena shape-study é bake isolado, não montagem viva entregue;
fonte com duas plantas/main/demo preservados. Sem aceite de realismo, FPS ou UI.

## Domínios de detalhes71 — integração na frente de plantas

Unidade31944f7 importada sobrea29d654 em native-plants. Scene71/replay75,
profile band/rounded e repeat2D opcionais; mesmo código da unidade candidata,
validação própria. Build05454b395586158e:847Rust/84grupos release/all-targets,
Clippy-Dwarnings/fmt/build e84MCP/339grupos completos passaram.
Auditoria66summaries mesmoID/18legados semcampo e fontehash correspondente.

37PNGs próprios do caminho legado70 permaneceram byte-idênticos. Medição da
translação periódica:9/640000pixels, máximo1LSB, média.000003515625por canal;
tolerância restrita a esse ensaio, não a replay/undo/load/repeat. A mesma árvore
foi regenerada/inspecionada/renderizada pelo binário71 em4vistas, com geometria,
fontes geométricas, rig/clipes/instâncias e4PNGs neutros exatos. Outputs em
artifacts/tree-realism, tree-material-recipe e tree-projection; cenárioMCP
detail_domains e scripts reproduzíveis mantidos. Logs detail-domain-*.log,
auditorias detail-domain-{integration,planar,period}-audit.json.

Revisão/limites e checklist próprio em detail-domain-integration.md. Ainda há
padrão repetido/angular na base, orientação comprimida nas forquilhas, copa
procedural e junções por concluir. A referência canônica69 não foi substituída;
melhorar realismo exige próximas provas e não aprovação estética como critério
de implementação. Nenhuma alegação de ganho de FPS/teclado nativo ou suporte
validado além de macOS.


## Seção/repetição de traços e estudo de realismo —71/75

Base de integração a29d654 incorporada por merge6acf3a1, src/tests/scripts iguais
à unidade validada do integrador antes deste incremento. Scene71/replay75:
Detail.profile band/default ou rounded e repeat2D opcional, MCP-first e gates
antes da avaliação de fontes ocultas. Build `05454b395586158e`, macOS/Metal.

-847testes Rust/84targets em release/all-targets passaram;3novos testes públicos
  de schema/compatibilidade, cápsulas variáveis/atomicidade e formato/replay.
  TabelaGPU/offsets/igualdade incluem o novo vec4; Vertex64/Globals176 mantidos.
- Clippy release/all-targets com-Dwarnings e build aprovados; fmt/diff/py_compile.
-84cenários MCP/339grupos completos headless passaram. Novo detail_domains:
  perfil/cor/bump separados, preview/histórico, negativos/periodicidade, triplanar,
  erros, gate71 em fontes visíveis/ocultas, subpixel, load/viewport/replay75.
- Translação de período tolera≤1nível8-bit por arredondamento f32; caso medido
  teve9/640000pixels alterados, máximo1. Frames repetidos/undo/load/replay exatos.
-3baselines antigas d314 (árvore/folha/detalhe,tick60/800²) e3vistas completas
  do build integrado70ed0b (neck/fork/whole) permaneceram PNG byte-idênticos
  para band/None. artifacts/triplanar-study/profile-legacy-comparison.json e
  artifacts/tree-realism/legacy-whole-comparison.json, sem sobrescrever baselines.
- Study_tree_bark.py passou via MCP persistente: muda só aparência na mesma
  árvore integrada; posições/triângulos/fontes geométricas/rig/clipes/instâncias
  exatos,4neutros pixel-idênticos, preview/undo/load/repeat/replay aprovados.
  Cenas/capturas em artifacts/tree-realism, relatório/inputSHA explícitos.
  A canônica69 permanece SHA89fe1fa2112cbb8a2d3ae266a53cf0d6308c8b371d356a8f5b1b05a208789400.

Inspeção independente rejeitou a primeira receita de cortes finos em faixas e
experimentou densidade/contraste/celular/ruído/seção/escala em ablações. Receita
corrente é ensaio, não certificação de casca/árvore realista. Repetição, direção
de fibra nas forquilhas, materiais regionais e copa ainda limitam realismo.
Contrato/checklist e critérios em detail-domains.md/performance.md; main/demo,
janela e push preservados, sem afirmação FPS ou aceite artístico por testes.

## Referenciais, acabamento e projeção materiais — integração70 validada

Fonte de integração codex/native-plants sobre7d84460; kernel puro e estudo da
mesma junção integrados, sem promover a cena canônica69. Scene70/replay74
adicionam opções materiais explícitas e projeção de detalhes opt-in pelo MCP.
Build `aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|ed0b5d78243910d8`.

- `cargo test --locked --release --all-targets`:844testes/83grupos passaram, incluindo os18novos
  testes de finisher/frames/projeção integrados sobre7d84460.
- Quatro testes novos de receita passaram: acabamento final nas fontes preservadas;
  afim geral independente da geometria/skin; preview/versão/regeneração/undo/replay;
  coeficientes/resultados inválidos, envelope e fonte singular atômicos.
-83cenários MCP/336grupos headless passaram no mesmo build; novo cenário
  plant_material_frames cobre ambas junções, domínio3D regular, pureza/schema,
  erros/versão, retorno de trajetória, persistência e capturas replay/repeat.
- Auditoria material-triplanar-build-audit.json:65summaries informam o mesmo ID e
  os18cenários legados omitem o campo. Hash de todos src+Cargo.lock+build.rs coincide
  com o binário do estudo e da regressão. Clippy release/all-targets com-Dwarnings,
  build, fmt/diff e py_compile aprovados. O Clippy intermediário apontou crescimento
  dos enums com novas opções inline; opções agora compartilham Arc, sem mudança
  de JSON ou supressão do lint.

Prova `scripts/demo_tree_material_frames.py`, artifacts/tree-material-recipe:
mesma geometria final, fonte baseline7d84460 congelada por commit. A cópia avaliada
recebe offset explícito da filha e da planta principal; fontes canônicas continuam
exatas. São mudanças de fase material intencionais, não conservação da aparência.
O novo acabamento usa planos emitidos pelo fit e suprime o solver material antigo.

| Região | Filha: máximo anisotropia3D | Junção principal: máximo3D |
|---|---:|---:|
| Guarda |1.35513|1.74187|
| Zipper |6.68589|2.00468|
| Iso |35.45969|1.75358|

Ambas sem faces singulares3D. Filha55passos/resíduo.000442893/alteração máxima
.606863unidadesM; principal19passos/.000453154/.0628244. O envelope menor da filha
relativamente à prova externa03fbc3f decorre de retirar a difusão legada; o campo
final equivale dentro da quantização, não de afrouxar o solver.
Posições/triângulos/rig/clipes/fontes/instâncias/outrosassets exatos. PNGs neutros
em neck/profile/fork/whole bit-idênticos. Voltas de trajetória e loft3→4→3, remoção
da camada/undo/redo/load/replay/captura repetida passaram. As linhas authored
persistem: a projeçãoXY pode comprimir um campo3D regular. Capturas nativas não
certificam realismo, teclado ou aceite artístico; EAF-80/EAF-85 seguem abertos.

Projeção integrada: três testes públicos de tipo/layout/publicação70/cache e três
testes internos de pesos/cache, mais teste de tabelaGpuMaterial. A publicação
real70 pelo MCP confere Some({})→sharpness2, preview sem mutação, limites/schema,
undo/redo/save/load, toggle no mesmo instante, replay/prefixo e pixels. A fixture
usa baselineXY em gridXZ; o defaultXZ coincide com a projeção dominante nessa
face e por isso não seria prova de mudança visível. Arquivos69 são recusados
inclusive nas4fontes visíveis/ocultas antes da avaliação da receita.

Probe GPU transitório repetido pelo binário integrado no fixture ESTÁTICO da
mesma junção:8grupos, ablações independentes cor/bump, sharpness1/2/4/8,4vistas,
resize, identity skin e movimento mantendoM/pesos, rank0/1 e rotação do domínio.
Uma execução preliminar na árvore inteira não serve à fixture de ablação
independente: o detalhe é pequeno na vista total e a substituição artificial do
rig não conserva os attachments; não é evidência de falha do material.

Estudo persistente da árvore INTEIRA em scripts/study_tree_projection.py,
artifacts/tree-projection: preview/editScene/undo/redo/load/replay70/74 aprovados.
Posições/triângulos/rig/clipes/instâncias exatos;4vistas neutras bit-idênticas;
cor material muda. A faixa inferior do close neck é atenuada pelo triplanar,
mas faixas e orientação artificial permanecem nas outras regiões. O modo fica
opcional; nenhuma ativação ou promoção automática na cena canônica69.

Auditoria material-planar-pixels-audit.json:19PNGs planares de antes/depois da
integração do shader/Layout64B são byte-idênticos (inclui ambas receitas, vistas
autoradas/neutras e replay/repeat). Evidência própria deste ramo, distinta das
3baselines independentes do ramo original. Mesmo builded0b5d78243910d8 no estudo
de frames, estudo inteiro e65summaries MCP;18cenários legados não informamID.
SHA256 da cena canônica preservado:89fe1fa2112cbb8a2d3ae266a53cf0d6308c8b371d356a8f5b1b05a208789400.

Logs material-triplanar-{all-tests,clippy,build,gpu,full-mcp,tree}.log e
material-frames-study.log; relatórios do runner/estudos/auditorias. A próxima
fricção artística é direção/continuidade da fibra da casca ao longo de ramos e
forquilhas. Mistura de projeções, suavidade dos pesos ou domínio3D regular
isoladamente não a definem. Nenhum ganho de FPS, teclado ou aceite humano alegado.


## Diagnóstico puro do domínio material — EAF-80/EAF-85

Base local b1058b5 em codex/native-meshes; Scene68/replay72 mantidos. Novas
consultas inspect_mesh_material_domain e preview_mesh_material_domain no
dispatcher/MCP; sem alteração de estado, renderer, material ou formatos.
Build `aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|25e12694e32424a3`.

- Sete testes analíticos em tests/mesh_material_domain.rs passaram: unidades,
  isometria/escala/shear, projeções, postos0/1, singularidade numérica, seleção,
  topK e média por área, winding/subdivisão afim, erros/pureza e admissão relativa
  à forma (triângulos pequenos isotrópicos admitidos; mal condicionados recusados).
- `cargo test --locked --release --all-targets`:821testes/79grupos passaram.
- Clippy release/all-targets com-Dwarnings, build, fmt/diff e py_compile passaram.
- Sete cenários MCP/33grupos headless passaram: mesh_material_domain,lab,
  mesh_surface_tools,materials,surface_details,mesh_junction_union,plant_assemblies.
  O novo cenário confere schema, query por asset/Geometry equivalentes, shear
  conhecido, fallback/projeção/colapso, erros/revisão obsoleta e histórico undo/redo.
- Logs estáveis artifacts/material-domain-{focused,clippy,build,mcp,tests}.log;
  relatório do runner artifacts/mcp-validation/report.json. Summaries com build_id
  conferem com o estudo; o cenário legado lab não informa esse campo.

Estudo nativo da MESMA árvore em artifacts/tree-material-domain/report.json e
scripts/study_tree_material_domain.py, resultados por origem em mesh-material-domain.md.
Snapshots69 são fontes EXTERNAS de Geometry explícita para a consulta pura,
não prova de load do formato69 pelo binário68. Atual SHA89fe1fa2…89400,
partição da união local6407source/4865iso/142guard/220zipperfaces. Todas142guardas
têm posto1 em3D antes/depois do collar; XY distingue2posto0+140posto1. Origem da
degeneração confirmada no código: valores de material copiados à guarda. Na
árvore composta334faces3Dsingulares,522XY; sem atribuir outros encontros/projeções
ao collar nem apresentar razão numérica instável como anisotropia regular.

Fontes originais/árvore/instâncias/capturas do integrador/main/demo preservados.
Nenhuma correção material ou nova aparência entregue nesta unidade. Próxima prova
é transportar derivadas à guarda com sourcecoords fixas, opt-in/legado exato e
controle do iso/zipper. EAF-80/EAF-85 continuam em andamento; não é aceite artístico.

## EAF-80/EAF-85 — junções integradas e segunda árvore (26/10/2026)

Build75a671e2f5f40d99, macOS Apple Silicon/Metal, cena55/replay59.732testes
Rust/65grupos release/all-targets, Clippy-Dwarnings/fmt/build e66cenários/
271grupos MCP headless aprovados. O runner inclui materiais, sombras e a
consulta pura preview_mesh_junction no mesmo binário final. Todos os relatórios
JSON com build_id no root isolado referem-se a esse build.

Handoff8641b61/7c7ebd0/d92db39 integrado como bc755c7/820efb5/dfa5473,
resolvendo módulo materials+mesh_organic, construtores do canal opcional e
ordem da regressão. Nenhuma versão adicional de Scene/replay para a query.
Logs artifacts/organic-integrated-{all-tests,clippy,build,full-mcp}.log,
relatório artifacts/mcp-validation/report.json.

Mesma árvore, primeira junção32amostras/refino2/fusão0.06/raio0.29:687v/1280t,
96bordas/192fixos, resíduo5.61e-8m, movimento máximo0.15587m, volume-11.02%.
Montagem de tronco/primeiro ramo/líder central por índices compartilhados:
1839v/3674t,1componente/χ2, zero bordas abertas ou conflitos de orientação.
Não é prova de interseção global nem conexão de todos os ramos. Sonda antiga
22.8815° continua recusada atomicamente no build atual.

Fonte examples/native-tree-study.scene.json,5.192.880bytes, cópia idêntica ao
save_scene do root isolado.1782folhas26v/32t,7assets/29instâncias,10810v/
15526t armazenados,64070t desenhados;19ossos/162chaves por variante. Folhagem
animada, madeira estática. Capturas before/after/back/root/fork/bark/leaf e
estrutura inspecionadas;48PNGs840×770 distintos codificados em GIF4s/2.482.138B.
Undo/redo, rejeição atômica por desenho, query sem mutação, seis pontos de
inserção preservados matematicamente, save/load/loop e PNG vivo/replay exatos.
Detalhes/custos/fricções em tree-authoring-lab.md. Arte ainda reprovada para
realismo: materiais aproximados, encaixes restantes, folhagem sem nervuras e
hierarquia de movimento ausente. Sem janela/main/push ou aceite humano novo.


## EAF-80 — orçamento de desenho explícito (26/10/2026)

Buildca2050fb74b44fa2, macOS Apple Silicon/Metal, cena55/replay59.722 testes
Rust/61grupos, Clippy-Dwarnings/fmt/build e65cenários/267grupos MCP headless
aprovados. Captura nativa de65536tri/196608v e replay byte a byte idênticos;
65537 e cena54 que esconda desenho acima32768 recusados atomicamente. Undo e
limites separados de armazenamento cobertos. Os testes de preflight que usavam
o teto antigo foram atualizados para67048t por subdivisão e71442t por instâncias;
a primeira rodada recusou essas expectativas antigas, sem falha de runtime.

Logs artifacts/draw-budget-{all-tests,clippy,build,full-mcp}.log e relatório
artifacts/mcp-validation/report.json. Contrato/custos/checklist antes/depois em
mesh-draw-budget.md. Mesmo limite4096v por asset/16384v e32768t armazenados,
32instâncias/16assets. Até2× geometria expandida, upload/desenho, sem alegar FPS,
LOD, instancing GPU ou realismo. Nenhuma janela/main/push.


## EAF-80 — materiais procedurais e domínio de superfície (26/10/2026)

Build5d3c11530f9e044f, macOS Apple Silicon/Metal, cena54/replay58.721 testes
Rust/61 grupos release/all-targets (7 novos), Clippy-Dwarnings/fmt/build e65
cenários/266 grupos MCP headless aprovados. O cenário materials verifica os3
padrões, pixels/undo/redo/repetição, remoção de slot, limites e canais inválidos,
legado, transmissão, skinning/replay/prefixo e fonte de plantas sem rebase por
metadados. Coordenadas são conservadas na escultura e interpoladas no midpoint
e na regeneração por correspondências explícitas. Todos os relatórios JSON que
possuem build_id no root isolado referem-se ao mesmo build final.

Logs artifacts/materials-{all-tests,clippy,build,full-mcp}.log e relatório
artifacts/mcp-validation/report.json. Vertex52bytes; tabela17×80bytes; mesmo
buffer/layout para captura nativa. Uma execução anterior de Clippy recusou if
aninhado redundante; colapsado e toda a cadeia repetida no build final acima.
Casca celular inspecionada em diferentes receitas: linhas fortes parecem rede,
ruído liso parece plástico; ajuste de receita permanece tarefa de autoria.
Contrato e custos em procedural-materials.md/performance.md. Sem PBR completo,
janela, integração na main ou alegação de realismo concluído.


## EAF-80 — sombras direcionais opt-in (26/10/2026)

Build7a0fdb376936bace, macOS Apple Silicon/Metal, cena53/replay57.714 testes
Rust/60 grupos release/all-targets (4 novos), Clippy-Dwarnings/fmt/build e64
cenários/262 grupos MCP headless aprovados. Cenário directional_shadows cobre
caixa e mesh opacos, pixel distinto ativo/desativado, repeat/undo/redo, limites
atômicos/legado, remoção/restauração, mapa512/1024/2048, viewport/luz/load/reset,
replay completo/prefixo. Inspeção visual das sombras da árvore e da base feita;
bias0,0008 na árvore removeu acne evidente da primeira configuração0,00025.

Logs `artifacts/shadows-{all-tests,clippy,build,full-mcp}.log`, relatório
`artifacts/mcp-validation/report.json`, PNGs `artifacts/mcp-validation/artifacts/shadow-*.png`.
Contrato/checklist em directional-shadows.md/performance.md. Vertex sem mudança;
folhas ainda opacas para sombra, ambiente escalar sem GI. Sem janela/main/push;
aceite artístico da árvore continua aberto.

## EAF-80 — sombreamento nativo (26/10/2026)

Build6278f10a82cab5b1, macOS Apple Silicon/Metal, cena52/replay56. Integração de
seções3a43a3b herdada como cdacdf7.710 testes Rust/59 grupos release/all-targets
(8 novos), Clippy-Dwarnings/fmt/build aprovados.63 cenários/258 grupos MCP
headless completed/passed; novo mesh_shading cobre troca no mesmo tick sem mudar
poses/payloads, erro atômico, migração, pixels distintos flat/smooth, undo/redo,
animação/reset/load, replay/prefixo e planta sem rebase da fonte. Logs correntes
`artifacts/mesh-shading-{all-tests,clippy,build,full-mcp}.log`, relatório
`artifacts/mcp-validation/report.json`. Contrato/checklist em mesh-shading.md e
performance.md. PNGs em `artifacts/mcp-validation/artifacts/shading-*.png`.

O recurso controla normais; não altera topologia, silhueta, física ou percepção.
Entrega de implementação/automação distinta de árvore realista ou aceite humano.
Sem janela/merge na main/push. Próximas capacidades de aparência mantêm gates próprios.

## EAF-79 — componentes e subdivisão linear (26/10/2026)

Worktree `/Users/zero/.codex/worktrees/native-meshes/custom-engine`, branch
`codex/native-meshes`. Integra plantas c94c6e4 como ebbc90c; native-plants e main
preservadas. macOS Apple Silicon/Metal, cena50/replay54, build
`aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|3077e97f88c54625`.
Contrato/decisões em [mesh-topology.md](mesh-topology.md), roteiro em
[mesh-roadmap.md](mesh-roadmap.md) e revisão antes/depois em performance.md.

- `cargo clippy --locked --release --all-targets -- -D warnings`, build release,
  formatação e diff aprovados. Logs `artifacts/mesh-topology-clippy.log` e
  `artifacts/mesh-topology-build.log`. Aviso futuro preexistente de block0.1.6
  separado dos warnings do projeto.
- `IGNIS_MCP_STARTUP_TIMEOUT=600 python3 scripts/run_verifications.py`:
  **61 cenários/250 grupos aprovados**, completed/passed/native_window:false,
  exit0. Inclui todos os cenários de NPC/população/física, render/seleção/replay,
  meshes e plants. Relatório `artifacts/mcp-validation/report.json`, log
  `artifacts/mesh-topology-mcp-full.log`.
- `mesh_topology`:4 grupos. Seleção de vértices/arestas/faces é apenas sessão,
  read-only para gameplay/histórico; refs obsoletas recusadas. Subdivisão parcial
  da grade4v/2t resulta7v/6t com vizinho conformante e undo/redo exatos; uniforme
  chega25v/32t e habilita escultura apenas no novo centro. Limites e erros não
  publicam estado parcial. Escultura altera a captura; cache recebe a nova geometria.
- Planta uniforme chega150v/192t sem substituir fonte; subdivisão localizada
  vinculada é recusada. Relevo criado somente num midpoint faz a redução de níveis
  falhar pela tolerância amostrada. Mudança simultânea de segmentos/níveis/dimensões
  preserva fonte e volta ao estado canônico. Edição explícita de rig/clipes após
  refinamento estabelece referência reproduzível e admite novo nível uniforme.
- Save/load/reset, Scene50 e Replay54, poses animadas sem interferência da seleção
  e viewport aprovados. PNGs repetidos, vivo/replay final e tick0/prefixo coincidem
  byte a byte. Capturas/logs `artifacts/mcp-validation/artifacts/mesh-topology-*`.
  Inspeção visual da captura: relevo alaranjado na grade refinada e extrusão azul;
  ensaio geométrico, sem alegar qualidade artística ou aceite humano.

`cargo test --locked --release --all-targets`: **697 testes/57 grupos aprovados**,
exit0. Log `artifacts/mesh-topology-tests.log`;18 testes novos sobre EAF-77. Os testes
específicos cobrem padrões de splits, área/orientação,
incidência, seams/não manifold/isolados, referências/seleção, transações e budgets,
rig/clipes/poses, plantas/refino/proveniência e perda de relevo em midpoint. A API
pura também recusa plano reutilizado com asset rigado incompatível antes de indexar
seus pesos.

Demo MCP autocontida em `examples/native-mesh-topology.scene.json`, idêntica ao
arquivo gerado pela verificação. Build corrente único `target/release/ignis`,
11.769.328bytes; sem cópias históricas de binário/installer. Não houve janela,
merge na main ou push. Aceite visual/integração permanecem separados. Não há editor
manual de componentes, Loop/Catmull-Clark, extrude/bevel, weld/remesh ou modifier
stack genérica entregues por inferência. Interpolação dos pesos novos não garante a
mesma superfície em todas as poses; contato/IK/retargeting continuam fora.

## EAF-77 — plantas e preservação canônica (26/10/2026)

Implementação em `/Users/zero/.codex/worktrees/native-plants/custom-engine`, branch `codex/native-plants`, desde a base de meshes482f0f9. Kernel de preservação00b18e0 integrado como665b349. Main funcional c1ada9c e pesquisa gráfica preservadas; nenhuma janela aberta, focada, fechada ou controlada. macOS Apple Silicon/Metal, cena49/replay53, build `aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|b3c93b958fd8d522`.

- `cargo test --locked --release --all-targets`: **679 testes aprovados**,56 grupos, exit0.22 adicionais sobre EAF-76:15 do kernel,3 de amostragem e4 de integração. Log `artifacts/rust-tests.log`. Expectativas de migração atualizadas para cena49/replay53, preservando os casos legados.
- `cargo build --locked --release`, `cargo clippy --locked --release --all-targets -- -D warnings`, `cargo fmt --all -- --check` e `git diff --check`: aprovados. Logs build/clippy em artifacts. Aviso futuro preexistente de block0.1.6 separado dos warnings do projeto.
- `IGNIS_MCP_STARTUP_TIMEOUT=600 python3 scripts/run_verifications.py`: **60 cenários/246 grupos aprovados**, completed/passed/native_window:false, exit0. Relatório `artifacts/mcp-validation/report.json`, log `artifacts/mcp-full.log`. Inclui meshes, render, replay, todos os cenários de NPC/população/física e o novo plants.
- Plants:4 grupos. Perfil2×4m/extrusão0,5m recebe escultura,3ossos/pesos e clipe; regeneração3×6×1m adapta repouso/chaves, conserva vetor físico e fonte, retorna sem drift. Retesselação de[4,12,2] para[8,24,4] conserva fonte; redução[1,1,1] é recusada por erro amostrado. Troca de perfil, budgets, stale revision e remoção vinculada preservam estado/histórico. Preview/consulta não mutam. Undo/redo, save/load/reset, detach/lote, viewport e animação aprovados.
- PNGs repetidos coincidem; execução viva/replay final e tick0/prefixo coincidem byte a byte. Relatório e imagens `artifacts/mcp-validation/artifacts/plants-*`. Captura final inspecionada: peça extrudada alaranjada com relevo e flexão do rig; imagem de ensaio geométrico, não prova de qualidade artística ou aceite humano.

Cena reproduzível `examples/native-plants.scene.json` foi autorada/salva pelo MCP, com fonte canônica e resultado avaliado. Build corrente único desta frente: `target/release/ignis` (11.567.760bytes). Reusar os caminhos de build/artifacts; nenhuma cópia histórica de app/installer criada. Sem push/publicação ou merge na main. EAF-77 mantém gate de aceite visual e integração coordenada; capacidades manuais de desenho/pincel/timeline não foram implementadas por inferência.

Contratos e alternativas em [plants.md](plants.md)/[mesh-preservation.md](mesh-preservation.md); checklist antes/depois em performance.md. Domínio explícito: retângulo/extrusão e deformação dimensional affine positiva; detalhe físico conserva vetor local, não todas as medidas de uma cicatriz. Erro de amostragem não é Hausdorff contínuo; fold local não comprova ausência global de interseções. Clipes preservam deltas relativos ao repouso, sem contatos/IK/retargeting. Sem solver CAD/multivista, cage, remesh livre, nova colisão/percepção, ganho de FPS ou outras plataformas alegados.

## EAF-76 — autoria nativa de malhas (26/10/2026)

Implementação isolada em `/Users/zero/.codex/worktrees/native-meshes/custom-engine`, branch `codex/native-meshes`, desde main local c1ada9c. macOS Apple Silicon/Metal, cena48/replay52, build `aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|f7a981411e343439`. Contrato/alternativas em [mesh-authoring.md](mesh-authoring.md); revisão antes/depois em [performance.md](performance.md).

- `cargo test --locked --release --all-targets`: **657 testes aprovados**, exit0,19 novos. Log `artifacts/rust-tests.log`; testes específicos em `artifacts/meshes-tests.log`.
- `cargo clippy --locked --release --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo build --locked --release` e `git diff --check`: aprovados. Logs `artifacts/clippy.log`/`build.log`. Aviso preexistente de futura incompatibilidade de block0.1.6 não é warning do projeto.
- `IGNIS_MCP_STARTUP_TIMEOUT=600 python3 scripts/run_verifications.py`: **59 cenários/242 grupos aprovados**, completed/passed/native_window:false, exit0. Log `artifacts/mcp-full.log`, relatório `artifacts/mcp-validation/report.json`. Inclui todos os cenários de NPC, população, física, render, seleção e replay.
- Cenário MCP meshes:5 grupos; schema compartilhado, criação de grade289v/512t, escultura draw com undo/redo, rig3 ossos/4 influências, clipe e duas instâncias em fases distintas. Inspeção de bind/pose/resumos e tempo explícito sem mutação; captura informa3072 vértices expandidos/1024 triângulos. Erros de índices/hierarquia/referências/budget/traço/revisão preservam estado e histórico.
- Capturas GPU repetidas são byte a byte idênticas; captura final e replay final coincidem, prefixo0 coincide com execução0. Remoção/growth/undo/redo/load/reset e viewport também aprovados. `meshes-verification.json` e PNGs em `artifacts/mcp-validation/artifacts/`. Captura inspecionada: duas folhas esculpidas/deformadas em fases distintas, sem erro de geometria/cache observado; isto não substitui aceite humano.

Os19 testes novos cobrem bind/transformação, quatro influências, pesos mistos, herança entre ossos, quaternions antipodais/interpolação, loop/offset/fim, pincéis/Arc/histórico, referências/ciclos/atomicidade/revisão, limites globais, grade pequena, legado e replay adulterado. Também cobrem persistência de uma cena no limite16384 vértices e replay de32 instâncias/131072 posições: JSON formatado ultrapassa8MiB, fallback compacto reabre e preserva os dados. A gravação recusa documentos cujo compacto também ultrapasse8MiB. Fixture de cena compara o payload de malhas, respeitando a ordenação preexistente de entidades pelo load.

Demo autorada inteiramente pelo MCP e salva em `examples/native-meshes.scene.json`; arquivo fonte coincide com o produzido na verificação. GIF `artifacts/meshes-demo.gif`:60 capturas nativas640×480,3 segundos de simulação, gerado novamente no mesmo build; `meshes-demo.json` registra proveniência. GIF é preview quantizado; provas de igualdade usam os PNGs originais. Nenhuma janela/demo do usuário foi aberta, focada, fechada ou controlada.

Build corrente único deste trabalho: `target/release/ignis` nesta worktree. Cache dev de check gerado pela tarefa (~224MiB) removido após lsof sem arquivos abertos; release/caches de outras frentes preservados. `main` funcional permanece em c1ada9c e não recebeu este branch; pesquisa gráfica separada. Commit local de entrega, sem push/publicação. EAF-76 mantém gate de aceite visual e integração coordenada das versões/contratos. MCP-first não conclui ferramentas manuais de pincel/pesos/timeline. CPU skinning/normais flat, sem remesh/IK/retarget/root motion/associação a NPC ou troca de animação ao vivo, sem FPS/escala de produção/outras plataformas alegados.

## EAF-104 — app local instalado e aberto, 27/10/2026

`python3 scripts/package_macos.py --editor --install` instalou `/Users/zero/Applications/Ignis Editor.app` diretamente, sem staging ou cópia de binário. Identificador próprio `dev.ignisengine.editor`, launch executável e plist aprovados; `sh -n`, `plutil -lint`, execução `--help` a partir de `/` e identidade de arquivo com o release corrente aprovados. Link simbólico ao release no volume Zero, pois o destino está em outro volume; apenas launcher/plist locais.

LaunchServices abriu o bundle com sucesso. Processo77024 confirmado com `--root` da worktree editor, `--editor` e `--scene examples/stealth-lab.scene.json`. Inspeção nativa confirmou app **Ignis Editor**, janela **Ignis — Editor**, menus/workspaces/painéis e cena renderizada, pausada no tick0. App deixado aberto para avaliação. Não foram exercitados teclado, arrastes, edição nem gameplay nesta verificação; aceite humano continua pendente, EAF-104 In Progress. Isso avança o gate de apresentação nativa registrado como pendente no relatório headless anterior abaixo.

Somente script de empacotamento/documentação alterados, nenhum kernel/build novo; as697 verificações Rust e63cenários/258grupos do mesmo binário permanecem a evidência anterior, sem repetição neste empacotamento. Um app instalado apontando para o build corrente, sem histórico adicional, outputs de outras frentes preservados. Projeto e volume devem permanecer disponíveis.

## EAF-104 — primeira estrutura do editor nativo, 27/10/2026

Worktree `editor`, branch `codex/editor`, desde main local f6b707e; macOS Apple Silicon/Metal. Build `aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|f482d8bd8a50b01f`. Cena63/replay67 preservados. Contrato, alternativas e checklist antes/depois em [editor.md](editor.md); menus/workspaces/áreas e câmera de inspeção fora da autoria/simulação.

- `cargo test --locked --release --all-targets`: **697 testes aprovados**, incluindo6 integrações do editor e1 teste de eventos UI. Cobrem transações/invariância de cena/runtime/replay, persistência confinada, collapse/move/viewport preservada, referências inválidas mesmo em no-op, limites/profundidade/revisão/overflow, projeção física/HiDPI e câmera separada. Eventos egui verificam clique na viewport e texto em painel sem seleção/gameplay indevidos; não são entrada física do SO.
- `cargo clippy --locked --release --all-targets -- -D warnings`, build release, fmt e diff sem erros. Aviso futuro preexistente de `block0.1.6` separado dos checks. Logs correntes: `artifacts/rust-tests.log`, `clippy.log`, `build.log`, `mcp-full.log`.
- Regressão MCP completa: **63 cenários/258 grupos**, relatório `artifacts/mcp-validation/report.json`, `completed/passed/native_window:false`. Inclui editor e todo o fluxo de NPCs/população, render/seleção/replay.
- Editor MCP:4grupos cobrem layout/workspace/persistência sem alterar mundo/replay, erros atômicos/revisão/caminhos, compositor GPU real com capturas repetidas e retorno de workspace/câmera idênticos, viewport compacto e edição/undo/redo/step/reset pelo dispatcher existente. Capturas estáveis em `artifacts/mcp-validation/artifacts/editor-*`. Revisão adicional: cinco divisões sucessivas aceitas renderizadas em900×600 sem janela ou falha, `artifacts/editor-dense.png`.
- Captura de entrega limpa: `artifacts/editor-current.png`,1440×900, cena stealth-lab/hunter selecionado, somente workspaces padrão; metadados/build em `artifacts/editor-current.json`. Mesmo compositor do host nativo, estado de widgets isolado; não representa screenshot de janela nem menus/foco/rascunhos vivos.

Nenhuma janela foi aberta/focada/fechada. **Integração de entrada/apresentação nativas e aceite visual humano permanecem pendentes**; manter EAF-104 In Progress. Primeiro incremento: uma cena/viewport por sessão, layout salvo/carregado explicitamente, sem editores de materiais/modelagem/animação das outras frentes ou chat fictício. Nenhuma alegação de FPS, escala nova ou outra plataforma validada.

Build corrente único desta frente: `target/release/ignis` via symlink local para `/Volumes/Zero/dev/build-cache/ignis-editor/target`,14.570.432bytes; cache release corrente~1,1GiB. Sem build debug, cópias de app/instaladores ou saídas por revisão. Frontes, árvores e outputs externos preservados; sem push/merge na main.

## EAF-123 — transações locais, buffs indexados e single player offline

29/10/2026, branch multiplayer-runtime desde e473e5a, macOS Apple Silicon:
fonte/binário75693edd70fe6348 conferidos pelo mesmo hash de build.rs.
769 testes Rust/63alvos release/all-targets, zero falhas/ignorados; Clippy
`-D warnings`, fmt, build, Python e diff aprovados. Regressão MCP completa
68cenários/280grupos completed/passed/native_window:false; todos os resumos
identificam o build final. Cena77/replay81 e JSON de buffs em lista preservados.

Novo cenário local_transactions3grupos roda o executável por stdio sob sandbox
macOS deny network*, confirmando EPERM em bind/connect. Movimento legado,
custo/self/vida/buffs, save/replay, dois jogadores, refresh/compactação/dispel,
reconstrução do índice por serde e undo/reset/load/rollback aprovados com rede
negada ao processo. O Mac e suas outras aplicações mantêm a conexão normal.
Novas4unidades+5integrações cobrem writes esparsos independentes da população,
16campos distintos, self, rollback tardio, morte dupla/ordem/cast de terceiro,
comparação com oráculo linear após256ciclos, referência obsoleta e buffers.

Evidências: artifacts/transactions-{rust-tests,clippy,fmt,build,mcp}.log,
transactions-build-audit.json, artifacts/mcp-validation/report.json e
artifacts/mcp-validation/artifacts/local-transactions-*. Contrato, tradeoffs,
checklist final e mapa de continuidade em [local-transactions.md](local-transactions.md).
Um release corrente11.435.552B no target externo exclusivo multiplayer-runtime,
cache1,0GiB/artifacts45MiB; sem app/instalador/cópia histórica ou janela.

Limites transitórios9atores/u8/máscarasu16, pools/capacidade de entidade e
relógio/gravação continuam pendentes. Single player offline é requisito permanente;
nenhum servidor separado/autenticação/internet exigido pelo núcleo local.
Não declarar capacidade1000, transporte de rede, cliente gráfico ou aceite humano.


## EAF-122 — controles independentes por personagem

Branch codex/multiplayer-runtime desde main6bfbc5c, macOS Apple Silicon/Metal,
cena77/replay81; build `aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|9f0587fa3f842121`.
Runtime Rust com vínculos exclusivos escopados, input/perfil por Player e eventos
de controle no replay; SetInput legado preservado quando não ambíguo. Contrato,
tradeoffs, checklist e reprodução em [actor-controls.md](actor-controls.md).

- `cargo test --locked --release --all-targets`:760 aprovados/62alvos, zero falhas
  ou ignorados;13testes novos de controles e1 de posse nativa sem janela.
- `cargo clippy --locked --release --all-targets -- -D warnings`, fmt e build
  release aprovados; warning transitivo antigo block0.1.6 separado.
- `python3 scripts/run_verifications.py`:67cenários/277grupos, completed/passed,
  native_window:false. Actor_controls5grupos/zero pedidos lentos, vida/input/cast,
  handles/reset/undo/load e prefixo/tickfinal; PNG vivo/replay byte-exato e imagem
  real inspecionada. Rust cobre também colisões, capacidade/NPCs/templates,
  morte-retorno no mesmo tick, serial/geração, logs saturados e liberação reservada.

Falha inicial: snapshot novo assumia referência de Player retirado pelo seam
cfg(test), detectada por world_origins. Consulta passou a exigir presença antes
de formar linha; regressão preservada e gates finais repetidos no fonte final.
Fonte/binário conferidos por FNV do mesmo conjunto src/Cargo.lock/build.rs.
Exemplo salvo via MCP e CLI final confirma2jogadores/zero vínculos persistidos.
Evidências em artifacts/controls-{rust-tests,clippy,fmt,build,mcp}.log e
artifacts/mcp-validation/report.json e arquivos actor-controls-* em
artifacts/mcp-validation/artifacts/.
Um build corrente11.401.728B em target→cache externo exclusivo962MiB; artifacts44MiB
no checkpoint. Sem cópia de app/instalador/push ou alteração da demo/frentes alheias.
Integração ao editor, teclado/foco/aceite humano, rede e capacidade1000 permanecem
separados; orçamento9atores e migração de índices/transações ainda transitórios.

## EAF-127 — conectores terrestres limitados

Main local desde790b046, macOS Apple Silicon, cena85/replay89, buildac325bd919355593 conferido com src/Cargo.lock/build.rs e CLI da demo. Reserva registrada em EAF-104 após editor02b1c2d83/87; sem importar capacidades isoladas ou modificar outras worktrees. Contrato/performance antes-depois em ground-local-connections.md/performance.md.

- `cargo test --locked --release --all-targets`: **796testes/64alvos**, zero falhas/ignorados;15integrações novas. Entrada/rede/saída com desvio físico, rampa direta sem rede,20/40graus/override,degrau20/50cm,teto/altura/gap,probe/candidatos/passos/retry estável e alvo móvel,conhecimento oculto/perseguição com ponto congelado,budget de investigação anônimo,aproximação parcial sem falsa chegada,cast lock por150ticks,templates/identidade/retirada/reuso,erro/undo/reset/replay/prefixo/adulteração. Teste antigo do controlador atualizado para recusar local na84 e aceitar na85, mantendo recusa de motor aéreo nos templates; gate completo repetido aprovado.
- Build/fmt/Clippy all-targets-Dwarnings/sintaxePython/diff-check aprovados. Aviso preexistente block0.1.6 separado; nenhuma dependência nova, profiling/FPS/escala não alegados.
- `python3 scripts/run_verifications.py`: **69cenários/285grupos**, completed/passed/native_window:false.51resumos com build_id correspondem aac325bd919355593;18legados não emitem ID. Novo ground_local_connections5grupos: desvio com entrada/saída sem loop zero,orçamento/retry,replay do budget,rampa/teto/erros/undo,perseguição visual móvel,demo MCP/replay/prefixo/tamper/reset/load e PNGs prefixo/final byteexatos. Legados terrestre/planar/aéreo/percepção/habilidades/população passaram.

Evidências: artifacts/ground-local-{full-tests,tests,clippy,fmt,build,mcp}.log, ground-local-build-audit.json, ground-local-gate-summary.json, ground-local-demo-inspection.json; artifacts/mcp-validation/report.json e artifacts/ground-local-connections-verification.json/PNGs/replays dentro do workspace isolado. Demo examples/npc-ground-local.scene.json criada/salva/carregada pelo MCP, byteexata à isolada. CLI confirma controladores guard/player no build auditado. Captura1000×700 tick100 inspecionada: roxo subindo rampa verde atrás do jogador amarelo na plataforma azul, rotas e raios diagnósticos. P inicia,WASD move jogador,R reset,J rotas; teclado/foco/aceite humano separados, nenhuma janela aberta/manipulada.

Reutiliza Dijkstra/local_connections/KCC, até3entradas e4saídas próximas,direto,cache8pares por plano,2400passos virtuais totais/600porprobe+2contatos, lançamento revalida até600extras. GroundPlanning opcional no State/replay expõe probes/steps/limited; ground_local_budget distingue busca limitada de falha geométrica completa. Seleção truncada não garante completude/ótimo global; retry60 após falha permanece mesmo se pista muda. Ponto da travessia corrente permanece comprometido, percepção não ganha acesso oculto. Nó com contato confirmado pelo probe evita conexão de comprimento zero; cast lock preserva connector/started_tick, gravidade permanece física.

Somente geometria fixa nas queries, sem reserva/previsão de dinâmicos/navmesh/transição aérea. Spawn continua apoio integral; caminhada/voo legados por omissão preservados. Cache de preparação24×128×600 herdado e alocações internas Rapier conhecidos, sem promessa MMO. Renderer/RenderFrame/Light/Vertex/Globals e janelas/worktrees alheias preservados. Um release corrente target/release/ignis/cache~1GiB/artifacts~75MiB, sem cópias de app/instalador ou push. Próximo contrato a discutir: transições entre motores terrestres/aéreos por trechos explícitos; NPCv1/M09 continuam abertos.


## EAF-126 — rede terrestre com controlador

Main local sobre d049fdf, macOS Apple Silicon, cena84/replay88. Buildbbab749348af7594 conferido com src/Cargo.lock/build.rs; nenhuma dependência nova. Reserva84/88 registrada em EAF-104/EAF-124 após confirmar união editor83/87/ccce577, sem importar capacidades das worktrees isoladas. Contrato e revisão antes/depois em ground-navigation.md/performance.md.

- `cargo test --locked --release --all-targets`: **781testes/63alvos**, zero falhas/ignorados;17integrações novas. Rampa20graus/descida,40graus recusada e permitida com45graus, degrau20/50cm/teto/tamanho, desvio por rota compatível, gap sem queda implícita, probe somente leitura/limites/apoio/orçamento, cache Arc compartilhado nos templates/nascimentos/retirada/reuso, legado sem herança de capacidade, casts preservando trecho, vida/retorno do mesmo indivíduo, patrulha repetida, alvo oculto/budgetXYZ, bloqueador dinâmico liberado por input sem reset, edição/undo/cache/replay/prefixo/adulteração. Probes reais dirigem admissão; corpo confirma chegada sem teleporte.
- Build/fmt/Clippy all-targets-Dwarnings/sintaxePython/diff-check aprovados. Aviso preexistente block0.1.6 de futura incompatibilidade separado. Sem profiling/FPS/escala alegados.
- `python3 scripts/run_verifications.py`: **68cenários/280grupos**, completed/passed/native_window:false.50resumos com build_id correspondem ao executávelbbab749348af7594;18legados não informam ID. Ground_navigation4grupos: query/erros/compatibilidade; físico20/40graus/override e degraus/teto; templates/pendência/reuso; escolha de rotas por capacidade, cache/undo/redo/reset/load, replay final/prefixo/tamper e PNGs byte-exatos. Legado espacial/aéreo/planar e sistemas anteriores passaram no mesmo gate.

Evidências estáveis: artifacts/ground-nav-full-tests.log, ground-nav-tests.log, ground-nav-clippy.log, ground-nav-fmt.log, ground-nav-build.log, ground-nav-mcp.log, ground-nav-build-audit.json, ground-nav-gate-summary.json, ground-nav-demo-inspection.json; artifacts/mcp-validation/report.json e artifacts/mcp-validation/artifacts/ground-navigation-verification.json/PNGs/replays. Demo examples/npc-ground-navigation.scene.json criada/salva/carregada por MCP e byteexata à cópia isolada; CLI confirma três atores/controllers e admissões. Captura1000×700 no tick90 inspecionada: roxo rumo à verde por desvio, azul na aproximação da vermelha curta, cinza junto ao degrau alto, jogador amarelo separado. Plataformas encostadas sem faces coplanares sobrepostas; captura/replay prefixo e final byte-exatos. Teclado/foco/aceite humano permanecem separados, sem janela aberta nesta rodada.

P inicia, WASD jogador, R reinicia, J rotas. Destinos fixos para comparar capacidades: roxo30graus usa rota1/2/3 verde(~20graus), azul45graus usa4/0/5 vermelha(~40graus), cinza recusa50cm com autostep30cm. Ambos os primeiros chegam à plataforma; terceiro permanece sem rota compatível. Controlador usa gravidade/colisão comuns. Probes usam somente geometria fixa, velocidade vertical inicial0 e até600passos virtuais+2checagens de contato; compilação usa nós+offset, lançamento e query usam pés explícitos. Tolerância de3ticks só após autostep com contato anterior, sem renovação enquanto no ar e contato recuperado para chegada. Cache por perfil preparado para templates preserva estado individual; custo de preparação máximo24×128×600 é explícito e não foi validado como escala MMO1000.

Sem navmesh/descoberta automática/conectores livres para este motor, alternância ground-air, reserva ou previsão de dinâmicos. Spawn/projeção conservam apoio integral; outros motores não migram. Renderer/RenderFrame/Light/Vertex/Globals, janelas e worktrees paralelas preservados. Próximo recorte: conexões locais limitadas entre posição/alvo e rede terrestre. NPCv1/M09 não concluídos. Um release corrente target/release/ignis/cache~1GiB e artifacts~74MiB, sem cópias de app/instalador nem push.


## EAF-124 — controlador terrestre opt-in

Main local sobre6bfbc5c, macOS Apple Silicon, cena82/replay86. Build7f73e8ed8358fbdf conferido contra src/Cargo.lock/build.rs; nenhuma dependência nova. Colisão inicial de reserva77/81 resolvida após consultar EAF-122/123 e editor81/85; coordenação registrada em EAF-122/EAF-104 e descrição EAF-124 corrigida. O primeiro gate Rust no formato anterior foi interrompido e os gates definitivos abaixo usam o novo executável. Nenhuma capacidade das worktrees isoladas é importada pela numeração.

- `cargo test --locked --release --all-targets`: **764 testes/62 alvos**, zero falhas/ignorados;18integrações novas de controlador. Incluem primeiro tick sem passo extra, rampas20/40graus e override, descida, autostep20/50cm/teto/clearance frontal, parede/deslizamento lateral, queda e gravidade/velocidade limite, push opt-in, contatos de objetos dinâmicos, cast lock/interrupção, NPC planar, erros/versões/histórico, morte/ressurreição/cadáver inicial, despawn/reuso por template, passos por offset/superfície, replay/prefixo/adulteração. Novo caso de passos acrescentado na revisão foi incluído no gate final completo.
- Build, fmt, Clippy all-targets -Dwarnings, sintaxePython e diff-check aprovados. Aviso preexistente block0.1.6 de futura incompatibilidade separado; sem profiling/FPS alegados.
- MCP completo: **67cenários/276grupos**, completed/passed/native_window:false.49resumos com build_id correspondem ao executável7f73e8ed8358fbdf;18legados não o informam. Character_controller4grupos: inspeção read-only/validação/legado/undo/redo/default pixels; física de rampa/degrau/teto/queda e comparação de motor; NPC/nascimento/retirada/reuso; demo/MCP/save/load/reset/prefixo/tamper e PNG vivo/replay byte-exato.

Evidências estáveis: artifacts/controller-full-tests.log, controller-clippy.log, controller-fmt.log, controller-build.log, controller-mcp.log, controller-build-audit.json, controller-gate-summary.json e controller-demo-inspection.json; artifacts/mcp-validation/report.json e artifacts/mcp-validation/artifacts/character-controller-verification.json/PNGs/replays. Demo examples/npc-character-controller.scene.json criada/salva/carregada pelo MCP, byteexata à cópia isolada; CLI final confirma tick0 e controladores guard/player com limite30graus. Captura final800×600 inspecionada: jogador sobre plataforma verde após rampa20graus, guarda no chão; vermelha40graus e quatro degraus azuis20cm. P inicia, WASD jogador, R reset; inspect_character_controllers expõe configs/estado. Capturas não comprovam teclado/foco/aceite humano, que permanecem separados e não foram solicitados nesta rodada.

Motor dinâmico padrão preservado. Contato parcial/grounded Rapier não certifica apoio integral ou inclinação caminhável; spawn/planejador continuam conservadores, motores aéreos incompatíveis explicitamente. Morte volta a corpo dinâmico, revive reinicia estado, reuso não herda dados. Push usa referência unitária de densidade; não implementa knockback recebido/multidões/plataformas móveis. Próximo contrato: consultas e planejamento compatíveis com rampa/degrau. NPCv1/M09 e meta MMO1000 não concluídos. Checklist antes/depois em performance.md, contrato/tradeoffs em character-controller.md. Um release corrente em target/release/ignis (cache~1GiB), artifacts~73MiB; sem cópias de app/instalador, janelas ou worktrees alheias controladas, nem push.


## EAF-115 — caixas e cunhas convexas compartilhadas

Main local sobre e16ae24, macOS Apple Silicon/Metal, cena75/replay79. Build `aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|003fd5138e831944`; hash do binário conferido contra arquivos de fonte/lock/build.rs antes da regressão. Reserva de formato coordenada com EAF-113 e EAF-116 (ambiente isolado76/80). Contrato/alternativas em [geometry.md](geometry.md), arquitetura e checklist antes/depois em performance.md.

- `cargo test --locked --release --all-targets`: **746 testes aprovados**,12novos (2unit de render/cache e10integrações). Grids de pontos/raios e overlap AABB expandido comparados ao Rapier nas4orientações. Inclui políticas de fronteira, altura/projeção, queda física no volume real, normais externas/8triângulos/padding degenerado, shape/cache/contorno, erro de ator/trigger/surface/versão, consulta sem mutação, visão/som, dano direto/projétil/área, voo/spawn/apoio inclinado recusado, rota terrestre contornando o sólido, undo/reset/replay e adulteração.
- `cargo fmt --all -- --check`, `cargo clippy --locked --release --all-targets -- -D warnings`, `cargo build --locked --release` e diff-check aprovados. Aviso preexistente de futura incompatibilidade de block0.1.6 separado; nenhuma dependência atualizada, nenhum profiling/FPS alegado.
- `python3 scripts/run_verifications.py`: **66cenários/272grupos aprovados**, completed/passed/native_window:false, exit0. Todos os48resumos desta execução que emitem build_id usam003fd5138e831944;18legados não emitem. Relatórios antigos fora desta seleção não são evidência do gate.
- Geometry5grupos: query read-only/erro/volume/vazio/fronteira; render com pixels idênticos Box explícito/omitido, cunha/undo e captura repetida; picking do sólido ignorando vazio, contorno/viewport; visão/som/LOS/projétil livre/área concordam; voo/spawn/apoio, persistência e replay final/prefixo. Captura final viva e replay byteexatas. Demais cenários cobrem render/cache/growth/removal, navegadores, populações e entregas antigas no novo formato.

Evidências correntes: artifacts/geometry-full-tests.log, geometry-clippy.log, geometry-build.log, geometry-mcp.log; artifacts/mcp-validation/report.json e artifacts/mcp-validation/artifacts/geometry-verification.json, PNGs/replays geometry. Demo examples/npc-geometry.scene.json criada/salva/carregada pelo MCP, byteexata à cópia isolada; CLI no build final confirma schema75/tick0. Captura inicial1000×700 inspecionada:4orientações de rampas e jogador amarelo; NPC roxo de0.5×1×0.5 em[-6,1.3,-3] voa até[-6,1.3,3], confirmado após240ticks. P mundo,R reset,WASD jogador,clique seleção;1 dano alvo,2 projétil livre,3 área com oclusão pelos controles de mira existentes. Aceite de teclado/foco e visual humano separado das capturas.

Volume estático/visual tipado, sem migração de motor ou subida configurada. AABB permanece limite amplo, mas vazio não vira parede nas consultas; apoio plano retorna sloped_surface no topo real da cunha. Personagens/triggers/surfaces de NPC/passos/passagens permanecem caixas; locomoção planar usa projeção da faixa vertical do corpo. QueryGeometry não reserva mundo nem concede conhecimento ao NPC. RenderBox recebe Geometry Copy, Vertex/Globals/shader/pipelines/bindings inalterados nesta frente; cache36v conserva12v degenerados por cunha e seleção9arestas/108v. Sem navmesh/heightfield/terreno irregular/controlador/inclinação/autostep/escala novos ou conclusão NPCv1/M09.

Um build corrente target/release/ignis; target/release991MiB/artifacts73MiB, sem cópias por revisão ou push. Janelas do usuário e worktrees gráficas preservadas. Nenhuma capacidade de material/mesh das frentes isoladas é incorporada só por usar número de cena maior. Próximo contrato: controlador opt-in jogador/NPC, política explícita de inclinação/degrau/snap/gravidade/empurrões e planejamento compatível.

## EAF-111 — apoio para nascimento e consulta de colocação

Main local sobre6b1ab5e, macOS Apple Silicon/Metal, cena66/replay70. Build `aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|c66f33c72803e60f`. Contrato/tradeoffs em [spawn-support.md](spawn-support.md), arquitetura e checklist antes/depois em performance.md. query_spawn_support somente leitura e support_projection opcional por gerador; primeiro topo fixo por XZ/max_drop, pegada modular e corpos correntes, sem fallback entre andares. Recibo conserva origem sorteada e relatório/centro; superfície legível na consulta/mapa de inspeção. Sem alteração de PRNG, pool, geometria, motores ou renderer.

- `cargo test --locked --release --all-targets`: **734 testes aprovados**,19novos. Cobrem consulta sem mutação, topo/empate independente da ordem e drop inclusivo, junção/fresta5mm, teto/tamanhos/ocupação física atual, template ponderado/alturas, limite64/fast path, falhas limitadas/identidade/pending/seed, legado exato/voo/mobilidade, cadáver/reuso, condição espacial+manual+ondas, alvo obsoleto, erro/versão/reset/undo/replay adulterado/prefixo.
- `cargo fmt --all -- --check`, `cargo clippy --locked --release --all-targets -- -D warnings`, `cargo build --locked --release` e diff-check aprovados. Aviso preexistente de futura incompatibilidade de block0.1.6 mantido separado. Nenhum profiling/FPS alegado.
- `python3 scripts/run_verifications.py`: **65cenários/267grupos aprovados**, completed/passed/native_window:false, exit0. Build ID único nos47resumos desta execução que o informam;18cenários antigos não emitem build_id no resumo. Arquivos de verificações antigos fora da seleção corrente não são evidência deste gate.
- spawn_support5grupos: consulta read-only/limites/erros; primeiro piso/junções/fresta/teto; alturas por template, pending/reuso/identidade/replay; falha sem quota/identidade/pausa/undo; demo gerada e persistida por MCP em plataformas distintas. PNGs first/reset/prefix idênticos e replay final/pendente/prefixo aprovados. Captura da demo final inspecionada offscreen; sem comprovar teclado/foco ou aceite visual humano.

Evidências correntes: artifacts/spawn-support-full-tests.log, spawn-support-clippy.log, spawn-support-build.log, spawn-support-mcp.log; artifacts/mcp-validation/report.json e artifacts/mcp-validation/artifacts/spawn-support-verification.json, PNGs e replays spawn-support no mesmo diretório. Demo examples/npc-spawn-support.scene.json criada/salva/carregada pelo MCP, byteexata à cópia isolada e carregada pelo CLI no build final:tick0/births0/schema66. Captura após241ticks confirma6nascimentos com3templates e pelo menos2alturas de piso; recusas no vão registradas. Plataformas autoradas0/1.5/3m, apoio estreito rejeitado e personagens1.6/3.2/0.8m. Gerador meta6/cadência60/tentativas8/budget18/retirada120; P mundo,B/O/K produtor,R reset,1/2 dano/ressurreição sujeitos à seleção/alcance/oclusão. NPCs parados intencionalmente para observar colocação; não é demonstração de navegação entre plataformas.

Um build corrente target/release/ignis; target/release981MiB e artifacts72MiB ao concluir (sem cópias de binários/instaladores por revisão). Janela do usuário/frentes gráficas preservadas, sem push. Consulta geometry_only não reserva capacidade/alvo/mobilidade ou prova caminho; piso fixo abaixo apenas, sem rampas/terreno/navmesh/controlador ou ampliação de escala. NPCv1/M09 continuam parciais; integração de geometria/controlador é decisão estrutural posterior.

## EAF-108 — composição ponderada de templates

Main local desdeb18c837, macOS Apple Silicon/Metal, cena65/replay69. Build `aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|61b2fd563f625e5f`. Contrato/tradeoffs em [weighted-population.md](weighted-population.md), arquitetura e checklist antes/depois em performance.md. Template único ou lista por gerador/onda; fonte herdada sem clonar, escolha estável até sucesso, stream próprio de template sem alterar o de posição. Membros/recibos atribuem template e inspeção mostra IDs/pending/draws.

- `cargo test --locked --release --all-targets`: **715 testes aprovados**,13 novos (1unit de limites do bucket e12integrações). Cobrem pesos/seed conhecida, fontes/vida próprias, streams independentes, pending/falha/pausa/capacidade, ausência de fallback, fontes por onda, alvo original retirado, dependências/erro/legado/reset/undo/replay e atribuição de retirada.
- `cargo fmt --all -- --check`, `cargo clippy --locked --release --all-targets -- -D warnings`, `cargo build --locked --release` e diff-check aprovados. Aviso preexistente de futura incompatibilidade de block0.1.6 mantido separado; sem profiling/FPS alegado.
- `IGNIS_MCP_STARTUP_TIMEOUT=600 python3 scripts/run_verifications.py`: **64 cenários/262 grupos aprovados**, completed/passed/native_window:false, exit0. Template_selection4grupos: escolhas conhecidas/vida própria/reset, igualdade do stream de posições com singleton, falha/pending/pausa/identidade preservada, erros/dependências/undo/load, herança/override por onda e chefe fixo. Build ID único nos resultados que o informam.
- Replay completo/bloqueado/prefixo aprovados. PNGs first/reset/prefix idênticos. Demo inspecionada: jogador no verde e primeira onda de3roxos+1laranja na área ciano; não é garantia geral da proporção3:1. O MCP removeu explicitamente membros para validar as3ondas/8nascimentos e7draws ponderados; não simula aceite humano de combate/teclado/foco.

Evidências correntes: artifacts/template-selection-full-tests.log, template-selection-clippy.log, template-selection-build.log, template-selection-mcp.log, artifacts/mcp-validation/report.json e artifacts/mcp-validation/artifacts/template-selection-verification.json. PNGs/replays template-selection no mesmo diretório. Cena examples/npc-weighted-population.scene.json criada/salva/carregada pelo MCP e byteexata à cópia isolada: P inicia,D/A entra/sai do verde,B/O/K permissão manual,1/2 dano/ressurreição,R reset. Primeira onda4 com guard3:elite1; segunda3 com guard1:elite3; último1boss200vida fixo. Seed42 produz3roxos+1laranja, depois2laranjas+1roxo, e chefe, em reprodução dos mesmos comandos/ticks.

Limites: até16candidatos únicos/pesos1..1000000, pool8NPCs/8geradores/16ondas e recibos256 mantidos. Escolha inviável ou com alvo original retirado pode bloquear produção; sem fallback/reroll ou cota exata por tipo implícita. Cancelamento impede executar pending preservado como diagnóstico; reset/edição o reconstrói. Um build corrente target/release/ignis, sem novas cópias históricas/instaladores ou push. Nenhuma janela do usuário controlada; RenderFrame/renderer/frentes paralelas preservados. Sem ganho de FPS, escala601 ou encerramento de NPCv1/M09 alegados.

## EAF-105 — posições regionais de nascimento

Main local desdef6b707e, macOS Apple Silicon/Metal, cena64/replay68. Build `aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|e1d61c1b482d64dd`. Contrato/tradeoffs em [spawn-regions.md](spawn-regions.md), checklist antes/depois em performance.md e arquitetura. Fonte regional exclusiva de points, centros XYZ/altura explícita, sem projeção em terreno. Posição tentada em recibos e transação spawn existente.

- `cargo test --locked --release --all-targets`: **702 testes aprovados**,12 novos de regiões. Incluem amostra conhecida/seed0/mesma/outra, centros limitados/plano/XYZ, voo/apoio/ocupação/capacidade, identidade/quota, cadência/pausa/ondas/ativação, fontes/versão/erros atômicos, legado/undo/reset e replay adulterado.
- `cargo fmt --all -- --check`, `cargo clippy --locked --release --all-targets -- -D warnings`, `cargo build --locked --release` e diff-check aprovados. Aviso preexistente de futura incompatibilidade de block0.1.6 mantido separado; sem profiling/FPS alegado.
- `IGNIS_MCP_STARTUP_TIMEOUT=600 python3 scripts/run_verifications.py`: **63 cenários/258 grupos aprovados**, completed/passed/native_window:false, exit0. Novo spawn_regions4grupos: mesma seed/limites/reset, recusas occupied/unsupported sem identidade/quota, fontes/bounds inválidos atômicos, despawn/reuso, replay pendente/prefixo/final, undo/load e demo completa de6nascimentos em3ondas. ID único nos resultados que informam build.
- Capturas first/reset/prefix idênticas; live/replay idênticas. Demo inspecionada: jogador amarelo no verde,3NPCs roxos em posições da região ciano e bloco baixo central. PNG/replay não comprovam teclado/foco/apresentação nativa.

Fixtures específicas foram corrigidas para os contratos existentes: conexões locais declaradas para mobilidade aérea, prazo exclusivo contado antes do step elegível, bounds f32 normalizados usados na comparação MCP e marcação com dimensão mínima0.05m. Obstáculo da demo ficou baixo0.4m para isolar recusa de posição sem obstruir cast; a recusa occluded com bloco alto era correta e não foi relaxada. Fontes finais passaram nos testes registrados acima.

Evidências correntes: artifacts/spawn-regions-full-tests.log, spawn-regions-clippy.log, spawn-regions-build.log, spawn-regions-mcp.log, artifacts/mcp-validation/report.json e artifacts/mcp-validation/artifacts/spawn-regions-verification.json. PNGs/replays spawn-regions no mesmo diretório. Demo examples/npc-spawn-regions.scene.json criada/salva/carregada por MCP e byteexata à cópia isolada: P inicia, D/A entra/sai do verde, B/O/K controlam permissão manual,1/2 dano/ressurreição e R reset. Região de centros min[-5,0.8,-1.5]/max[5,0.8,2.5], seed42, até8tentativas/ciclo30ticks,3roxos→2laranjas→1azul200vida com120ticks entre ondas.

Um build corrente target/release/ignis, sem cópias históricas novas/instaladores ou push. Sem janela do usuário controlada, worktrees gráficas e RenderFrame/renderer preservados. Pool8NPCs e8geradores, recibos256; corpos podem ultrapassar limites de centros, candidatos podem repetir e falhar, sem garantir rota/chão/encaixe/distribuição uniforme ideal. Nada deste recorte conclui NPCv1/M09, escala601 ou ganho de FPS; aceite humano da demo permanece separado.

## EAF-96 — ativação espacial de geradores

Main local desde751b4a4, macOS Apple Silicon/Metal, cena63/replay67. Build `aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|708a5bfd9223c609`. Contrato em [generator-activation.md](generator-activation.md), arquitetura e revisão estática antes/depois em performance.md. EAF-95 recebeu aceite humano; formatos coordenados no Kordo com frente gráfica isolada, sem RenderFrame/Globals/Vertex/renderer novos.

- `cargo test --locked --release --all-targets`: **690 testes aprovados**,12 novos de ativação. Cobrem início dentro/primeira física, atraso/cadência/ondas, once/while_occupied, manual+espacial sem dupla contagem, cancel/terminal, alvo morto/ressuscitado/retirado/substituído, ordem de volumes/XYZ, ausência de novo alvo NPC, erros/legado/undo/reset e replay/prefixo/adulteração.
- `cargo fmt --all -- --check`, `cargo clippy --locked --release --all-targets -- -D warnings`, build release e diff-check aprovados. Mantido aviso preexistente de futura incompatibilidade de block0.1.6, separado do resultado. Nenhum profiling ou ganho de FPS alegado.
- `IGNIS_MCP_STARTUP_TIMEOUT=600 python3 scripts/run_verifications.py`: **62 cenários/254 grupos aprovados**, completed/passed/native_window:false, exit0. Generator_activation com4 grupos: entrada/saída/once, pausa combinada/deadlines, cancelamento/reentrada, fase pós-física/XYZ/inicial dentro, erro/dependência/undo/reset/load, replay/prefixo/capturas e demo autorada por MCP. ID único nos resultados que informam build.
- Capturas entered/prefix e live/replay idênticas; condição, relógio e fase reproduzidos. Demo-inside inspecionada: jogador amarelo sobre marcação verde e2NPCs roxos presentes. Capturas GPU offscreen não comprovam teclado/foco/apresentação nativa.

Evidências correntes: artifacts/generator-activation-full-tests.log, generator-activation-clippy.log, generator-activation-build.log, generator-activation-mcp.log, artifacts/mcp-validation/report.json e artifacts/mcp-validation/artifacts/generator-activation-verification.json. PNGs/replays generator-activation no mesmo diretório. Demo examples/npc-generator-activation.scene.json salva/carregada pelo MCP e byteexata à cópia isolada: P executa, D entra na região à direita, A sai, B/O/K permissão manual,1/2 dano/ressurreição e R reset. While_occupied com ondas2roxos/2laranjas/1azul200vida e intervalo120ticks.

Uma versão corrente em target/release/ignis, sem cópias históricas de binários/instaladores ou push. Nenhuma janela da demo do usuário foi controlada; worktrees gráficas preservadas. Ocupação discreta AABB XYZ de targets explícitos, incluindo corpos mortos; Reference removida não passa autorização ao novo ocupante. Permissão de mundo não fornece conhecimento ao NPC. Mantidos pool8/geradores8/ondas16 e limites de triggers; sem composição genérica de regras/seleção automática de todos os jogadores, regiões de nascimento, escala601, FPS alegado ou fechamento de NPCv1/M09.

## EAF-95 — controle de geradores em execução

Main local desde685505b, macOS Apple Silicon/Metal, cena62/replay66. Build `aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|fe25fdc119c54abf`. Contrato/tradeoffs em [generator-control.md](generator-control.md), checklist antes/depois em performance.md. EAF-93 recebeu aceite humano. Reserva de versão coordenada com EAF-94; RenderFrame/Globals/Vertex e worktrees gráficas preservados.

- `cargo test --locked --release --all-targets`: **678 testes aprovados**,13 novos. Incluem pausa no deadline, atraso/cadência restantes, ressurreição durante pausa, terminal por quota/ondas/cancelamento, isolamento entre geradores, NPC locomovendo/jogador com input, reuso externo, fila64/log4096/anel256, erros/legado/undo/reset e replay adulterado.
- `cargo fmt --all -- --check`, `cargo clippy --locked --release --all-targets -- -D warnings`, `cargo build --locked --release` e diff-check aprovados. Aviso preexistente de futura incompatibilidade de block0.1.6 mantido separado. Sem profiling/FPS alegado.
- `IGNIS_MCP_STARTUP_TIMEOUT=600 python3 scripts/run_verifications.py`: **61 cenários/250 grupos aprovados**, completed/passed/native_window:false, exit0. Novo generator_control com4 grupos: ativação pendente/atraso combinado, pausa/cadência/interonda, dano/retirada durante suspensão, cancelamento/sobreviventes/terminal, erros, save/load/undo/redo/reset, mesma fila e demo autorada por MCP. Build ID único nos resultados que o informam.
- Replay66 pendente, final e prefixo aprovados. Capturas generator-control-paused/prefix idênticas; cancelled/replay idênticas. PNG cancelled inspecionado com dois NPCs laranjas vivos após cancelamento; demo inspecionada com dois roxos após ativação e jogador amarelo. Capturas não comprovam atalhos/foco/apresentação nativa.

Evidências correntes: `artifacts/generator-control-full-tests.log`, `generator-control-clippy.log`, `generator-control-build.log`, `generator-control-mcp.log`, `artifacts/mcp-validation/report.json` e `artifacts/mcp-validation/artifacts/generator-control-verification.json`; PNGs/replays generator-control no mesmo diretório. Uma fixture inicial de replay adulterado apontava para um campo JSON inexistente; corrigida para expected_generators e cenário específico aprovado. Não foi falha do runtime.

Demo `examples/npc-generator-control.scene.json` salva/carregada pelo MCP e copiada byte a byte da validação isolada: inicialmente pausada, P executa mundo, B ativa/retoma, O pausa produção, K cancela, R restaura autoria. Mesma sequência2roxos/2laranjas/1azul200vida;1 causa100dano,2 ressuscita cadáver presente. Atalhos só quando há um gerador, sem escolher outro implicitamente. Teste humano desta demo separado da automação.

Build corrente em target/release/ignis, sem binários/instaladores históricos novos. Nenhuma janela de demo controlada; sem push. Mantidos pool8NPCs,8geradores,16ondas, fila64/log4096/recibos256 compartilhados. Cancel/pause não removem vivos; retirada de cadáveres continua no tempo do mundo. Não há reinício parcial, condições automáticas de ativação, escala601 ou encerramento de NPCv1/M09 neste recorte.

## EAF-93 — ondas finitas de população

Main local desde8d4f972, macOS Apple Silicon/Metal. Cena61/replay65 coordenados com a frente gráfica EAF-91; RenderFrame preservado. Build `aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|e51089df9bba9c91`. Contrato em [population-waves.md](population-waves.md), checklist antes/depois em performance.md.

- `cargo test --locked --release --all-targets`: **665 testes aprovados**,13 novos de ondas. Cobrem quotas maiores que a concorrência, atrasos/cadência, troca de template, candidatos bloqueados, retirada versus morte, ressurreição antes/depois da conclusão, identidade reutilizada, alvo original de onda futura, limpeza após término, cadáveres ocupando o pool, erros/undo/reset, replay adulterado e16×8 ondas com eventos limitados.
- `cargo fmt --all -- --check`, `cargo clippy --locked --release --all-targets -- -D warnings`, `cargo build --locked --release` e diff-check aprovados. Aviso preexistente de futura incompatibilidade de block0.1.6 mantido separado; nenhum profiling ou ganho de FPS alegado.
- `IGNIS_MCP_STARTUP_TIMEOUT=600 python3 scripts/run_verifications.py`: **60 cenários/246 grupos aprovados**, completed/passed/native_window:false, exit0. Cenário waves com5 grupos; geradores legados também aprovados. Replay65 reproduz quotas/fases/eventos, pedidos pendentes e prefixos. Capturas repetidas/finais e prefixos correspondentes idênticos.
- Demo `examples/npc-waves.scene.json` criada/salva/carregada pelo MCP e idêntica ao arquivo produzido na regressão isolada. Sequência completa de cinco nascimentos:2 roxos,2 laranjas,1 azul com200vida, dois segundos entre ondas; último NPC validado em200→100→0. PNGs waves-second e waves-demo inspecionados: jogador amarelo, template laranja na segunda onda e dois roxos na primeira. Captura offscreen não comprova teclado/foco/apresentação nativa.

Evidências correntes: `artifacts/waves-full-tests.log`, `waves-clippy.log`, `waves-build.log`, `waves-mcp.log`, `artifacts/mcp-validation/report.json` e `artifacts/mcp-validation/artifacts/waves-verification.json`; replays e PNGs waves no mesmo diretório. Build corrente único em `target/release/ignis`, sem cópias por revisão ou instaladores. Nenhuma janela de demo do usuário foi aberta, fechada ou controlada; worktrees gráficas preservadas.

Implementação e automação concluídas; aceite humano da demo permanece separado. Pool8 NPCs e16 ondas por gerador mantidos. Conclusão não desfaz ressuscitados anteriores; cadáveres preservados podem impedir vagas. Ativação/pausa/cancelamento em execução, composição de templates por onda, regiões contínuas e escala permanecem futuros; NPCv1/M09 não encerrados por este incremento.

## EAF-92 — geradores e reposição de população

Main local desde c1ada9c. Cena60/replay64 reservados após consulta a EAF-91 (59/63 na frente gráfica isolada). Build `aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|d8205314e3395ffd`. Configuração e inspeção MCP, seed/candidatos limitados, meta viva/cadência/budget, retirada opcional de cadáver e associação por Reference/life_index. RenderFrame inalterado; sem integração implícita das capacidades gráficas em outras worktrees.

- `cargo test --locked --release --all-targets`:652 testes aprovados,14 novos (13 de integração e1 de fronteira de vitalidade). Inclui cadáver preservado, excesso por ressurreição, prazo/mesmo tick, ressurreição+nova morte entre amostras por commits reais, prioridade externa, outro ocupante, alvo original, seed0/outra seed, competição/pool, budget/tentativas, identidade, replay/prefixo e erros/reset/undo.
- `cargo clippy --locked --release --all-targets -- -D warnings`, `cargo fmt --all -- --check` e build release aprovados. Mantido aviso preexistente de futura incompatibilidade de block0.1.6; sem profiling ou FPS alegado.
- `python3 scripts/run_verifications.py`:59 cenários/241 grupos, completed/passed, native_window:false, exit0. Novo cenário generators4 grupos: autoria, reposição/revive, retirada/ref antiga, replay pendente/prefixo/final, captura idêntica, erros/limites/undo/redo/reset. Cenário novo repetido isoladamente após aproximar a câmera da demo; aprovado no mesmo build.
- Capturas GPU `generators-live.png`, `generators-repeat.png` e `generators-replay.png` idênticas; `generators-initial.png` idêntica ao prefixo. Demo offscreen inspecionada: jogador amarelo e dois NPCs roxos, câmera próxima. Não comprova teclado, foco ou apresentação nativa.

Evidências correntes: `artifacts/generators-full-tests.log`, `generators-clippy.log`, `generators-build.log`, `generators-mcp.log`, `artifacts/mcp-validation/report.json`, `artifacts/mcp-validation/artifacts/generators-verification.json` e PNGs no mesmo diretório. Logs intermediários supersedidos substituídos/removidos; mantidos target/release/ignis e artefatos correntes, sem cópias de binários por revisão. Nenhuma janela/processo de demo do usuário foi controlado ou encerrado; worktrees gráficas preservadas.

Cena `examples/npc-generators.scene.json` autorada/salva por MCP e copiada da verificação isolada:2 vivos,3 pontos, seed42, intervalo60ticks, retirada após180ticks observados e orçamento20nascimentos. P inicia; clique/1 mata; clique/2 ressuscita enquanto o cadáver existe; R reset. Contrato e custos em population-generators.md/performance.md. Gates de implementação e automação concluídos; teste humano separado. Ondas, regiões contínuas e aumento da capacidade permanecem próximos incrementos, sem conclusão implícita de NPCv1/M09.

## EAF-70 — nascimento por templates (26/10/2026)

macOS Apple Silicon/Metal, cena47/replay51, build `aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|1dbf032605b518c6`. [Contrato](spawn-templates.md), decisões em architecture.md e checklist antes/depois em performance.md.

- `cargo test --locked --release --all-targets`: **638 testes aprovados**,18 novos; exit0. Evidência `artifacts/spawn-full-tests.log`.
- `cargo fmt --all -- --check`, `cargo clippy --locked --release --all-targets -- -D warnings`, `cargo build --locked --release` e diff-check aprovados. Logs spawn-clippy.log/spawn-build.log. Aviso preexistente de futura incompatibilidade de block0.1.6 permanece separado.
- `IGNIS_MCP_STARTUP_TIMEOUT=600 python3 scripts/run_verifications.py`: **58 cenários/237 grupos aprovados**, completed/passed/native_window:false, exit0. `artifacts/spawn-mcp.log` e `artifacts/mcp-validation/report.json`.
- Cenário spawn:4 grupos; templates criados/inspecionados/salvos pelo MCP, nascimento pendente, irmãos independentes, troca de template, rejeição de token antigo, sobrevivente preservado, todas as inspeções, ocupação/apoio/erro atômico/reset/undo/redo. Replay51 e captura offscreen repetida/prefixo produzem PNGs idênticos; resize também exercitado. `artifacts/mcp-validation/artifacts/spawn-verification.json`, spawn-live.png/spawn-repeat.png/spawn-prefix.png. Imagem live inspecionada: jogador amarelo, irmão roxo e substituto verde selecionado, guias próprios.

Rust cobre mesma/diferente configuração no slot reutilizado, recursos/cooldowns/cast/buffs/bases, vida0 após morte/ressurreição, memória própria, fonte destacada com dano original, alvo periódico removido, cast sobrevivente, cancel obsoleto com recibo único, ordenação lógica após reciclar slot inferior, capacidade e ordem da fila, apoio/volume/voo/passagem, patrulha/passos,300 gerações com mapas/anéis limitados e templates inválidos sem reset. Replay reconstitui as entradas válidas para a população de cada tick; comandos para indivíduos ainda inexistentes são recusados.

A validação separou colisão nova de transferência indevida: um projétil que ainda atravessa o volume de origem pode atingir quem nascer ali depois; a fixture de origem destacada passou a nascer fora dessa trajetória para testar apenas retenção da autoria. DoT dirigido ao alvo retirado encerra sem afetar o substituto. Outras correções de verificação: esperar tempo físico suficiente para impacto, declarar conectores para o voo e filtrar recibos Completed ao verificar ordem (há também recibo Started). Revisão corrigiu dupla rejeição de cancel obsoleto e casts desnecessários; fontes finais passaram por toda a regressão acima.

Cena de templates `examples/npc-spawn.scene.json` salva pelo MCP isolado; inicialmente contém somente a população autorada, sem nascimento automático. Build corrente `target/release/ignis`; demo existente PID67453 preservada, sem foreground nem troca de processo. Sem push/publicação ou aceite de teclado nativo alegado. Limite8 NPCs+jogador,16 templates; geradores/reposição e aumento de escala continuam posteriores, sem profiling/FPS declarado.


## EAF-67 — retirada de NPCs por fila MCP (25/10/2026)

macOS Apple Silicon/Metal, cena46/replay50, build `aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|1d6335d9fa0027e7`. Contrato em [population.md](population.md); checklist antes/depois em [performance.md](performance.md).

- `cargo test --locked --release --all-targets`: **620 testes aprovados**, exit0,12 novos. `artifacts/population-full-tests.log`.
- `cargo fmt --all -- --check`, `cargo clippy --locked --release --all-targets -- -D warnings` e `cargo build --locked --release`: aprovados. Logs population-clippy.log/population-build.log. Aviso preexistente de futura incompatibilidade de block0.1.6 separado da validação.
- `IGNIS_MCP_STARTUP_TIMEOUT=600 python3 scripts/run_verifications.py`: **57 cenários/233 grupos aprovados**, completed/passed/native_window:false, exit0. `artifacts/population-mcp.log` e `artifacts/mcp-validation/report.json`.
- Cenário MCP population:4 grupos, fila pausada/duplicado/identidade/atomicidade, retirada com trigger/seleção, origem de projétil/periódico sobrevivente, liberação de contexto, replay50 completo e prefixo com pedidos pendentes, reset/token antigo. `artifacts/mcp-validation/artifacts/population-verification.json`; population-live.png e population-prefix.png idênticos. Captura GPU inspecionada, sem corpo ausente reaparecendo; projétil permanece em voo.

Rust cobre cancelamento do cast retirado sem reiniciar cast/saldo/cooldown do sobrevivente, pedido de habilidade obsoleto, expiração de buff na origem removida, morte versus retirada de cadáver, memória congelada, trigger com identidade original, liberação de passagem para sobrevivente, limites e replay adulterado. Teste integrado retira NPCs nas43 cenas de exemplo com NPCs e verifica step/snapshot/frame/replay; não confundir isso com teste de cada situação possível de gameplay. Anel limitado testado com1000 recibos; nenhum teste de reuso público/spawn alegado.

A regressão encontrou participante de passagem que continuava na tabela após liberar reserva, causando acesso à posição ausente; corrigido removendo também a entrada do participante. Fixtures novas foram ajustadas ao contrato sensorial existente: NPC precisa estar habilitado e ter evidência do alvo mesmo em comandos externos; fixture de entrega usa percepção onisciente explícita. Mantidos saldos/cooldowns inativos nos arrays limitados, sem reiniciar/pagar/reembolsar na retirada e sem exposição como ator presente; instalação futura deve inicializá-los. Fontes estabilizadas passaram por toda a suíte acima.

Um build corrente em `target/release/ignis`; sem cópias históricas novas, push ou controle de janela. Sessão visual PID67453 permaneceu ativa e intocada; não foi atualizada para este build. Capturas/relatórios substituem caminhos estáveis. Nascimento/templates, ordenação de novos indivíduos, geradores e ampliação de população continuam pendentes. Limite9 preservado, sem medição de FPS ou garantia de escala.


## EAF-63 — mundo presente e contexto de origem (25/10/2026)

macOS Apple Silicon/Metal, cena46/replay49, build `aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|748cf535d9ff16de`. [Contrato](runtime-world.md) e checklist antes/depois em [performance.md](performance.md).

- `cargo test --locked --release --all-targets`: **608 testes aprovados**, exit0;16 novos de mundo/origem. Evidência corrente: `artifacts/world-full-tests.log`.
- `cargo fmt --all -- --check`, `cargo clippy --locked --release --all-targets -- -D warnings`, build release e diff-check aprovados. Logs world-clippy.log/world-build.log. Aviso preexistente de futura incompatibilidade de block0.1.6 permanece separado.
- `IGNIS_MCP_STARTUP_TIMEOUT=600 python3 scripts/run_verifications.py`: **56 cenários/229 grupos aprovados**, completed/passed/native_window:false, exit0. Logs world-mcp.log e `artifacts/mcp-validation/report.json`.
- Cenário novo world_origins:4 grupos; posições presentes separadas da autoria, reservas compartilhadas entre projétil/periódico, buff aplicado depois do lançamento ainda afeta impacto, liberação sem retenção por recibos, erros atômicos/reset, replay49 completo/prefixo e PNGs world-live/world-prefix idênticos. PNG live inspecionado; sem foreground.

Testes Rust retiram corpos/componentes por seam cfg(test): fonte ausente ainda causa dano e recebe atribuição correta na morte, contribuições expiram pelo prazo original, dissipação respeita proteção até expirar, slot substituído não fornece atributos/vida, áreas fixas sobrevivem e auras encerram, ausência não deixa colisão/render/guias/radio fantasmas, cast não pago cancela, pool cheio rejeita antes de custo/cooldown, última entrega libera reserva, buffers preservam ponteiros na captura. Source effects proibidos na autoria atual foram injetados somente na fixture interna do executor para verificar rollback defensivo, sem ampliar o contrato público.

A regressão revelou pressupostos de presença na coleta de rádio e ordenação de áreas; corrigidos. Fixtures de seleção passaram a reconstruir a execução após alteração direta do documento (incluindo dimensão mínima válida de0.05m); o teste também comprova que alterar autoria isoladamente não muda visibilidade viva. Comparação de física em triggers agora separa o placement do volume sem corpo, incluído deliberadamente nas posições do mundo. Fixture de dissipação habilita o NPC receptor, conforme regra preexistente do executor. A suíte final completa acima foi executada depois desses ajustes.

Build corrente: `target/release/ignis`; sem cópia histórica/instalador novo. Sessão visual existente PID67453 preservada. Capturas e relatórios substituem caminhos estáveis sob artifacts. Remoção/reuso internos não são prova de comandos públicos spawn/despawn, ainda pendentes, nem de replay de comandos de população inexistentes. Apoio/rede/rotas seguem estáticos e limite9 atores mantido; nenhuma medição de FPS/escala601 alegada.


## EAF-61 — vínculos persistentes de identidade (25/10/2026)

macOS Apple Silicon/Metal, cena46/replay48, build `aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|dfdcc7ff9a424809`. Buffs/refresh/dissipação, observações/reconhecimento/rádio, ruídos e históricos conservam indivíduo; aliases não são reatribuídos pelo slot. Contrato em [identity-bindings.md](identity-bindings.md), checklist antes/depois em [performance.md](performance.md).

- `cargo test --locked --release --all-targets`: **592 testes aprovados**, exit0;9 novos de reciclagem no registro real por interface cfg(test). `artifacts/identity-migration-tests.log`.
- `cargo fmt --all -- --check`, `cargo clippy --locked --release --all-targets -- -D warnings` e build release passaram. Logs identity-migration-clippy.log e identity-migration-build.log. Aviso preexistente de futura incompatibilidade de block0.1.6 permanece separado.
- `IGNIS_MCP_STARTUP_TIMEOUT=600 python3 scripts/run_verifications.py`: **55 cenários/225 grupos aprovados**, completed/passed/native_window:false, exit0. `artifacts/identity-migration-mcp.log` e `artifacts/mcp-validation/report.json`.
- Novo cenário identity_bindings:4 grupos, buffs/contribuições/expiração, proveniência visual/rádio, autoria e destinatário de pulsos, morte/ressurreição do mesmo indivíduo. Replay48 completo e prefixo conferem NPCs/habilidades/vitalidade/triggers. Capturas bindings-radio-live/prefix idênticas, PNG live inspecionado. Nenhuma janela usada.

Rust cobre novo ocupante sem buff/refresh/dissipação/reconhecimento herdados, limpeza de dono removido, pista oculta preservada sem alvo corrente, remetente/sujeito obsoleto rejeitado, som anônimo sem falsa autoexclusão, histórico de dano/morte sem renomear participantes, periódico sem refresh entre indivíduos e saída de trigger sem consultar posição removida. Reuso é do registro isolado; substituição real dos componentes do mundo continua inacessível no runtime.

Ajustes de validação: fixture antiga de autoexclusão acústica passou a fornecer referência do emissor; expectativa de aim_observation inclui instance_id. O primeiro teste novo de prefixo vital capturava antes de enfileirar uma ação no mesmo tick; corrigido para comparar o estado com a fila pendente que o replay preserva. Uma rodada intermediária de compilação viu a assinatura antiga da biblioteca e chamadas de teste já atualizadas durante a revisão de trigger; descartada, seguida de Clippy e suíte final completa sobre fontes estabilizadas. Não atribuir essas ocorrências a falhas de gameplay. Revisão removeu catálogo redundante nos triggers e buscas adicionais por ID na montagem dos corpos.

Binário corrente em target/release/ignis, inode8857120/10.521.568bytes. `lsof` confirmou a demo anterior em uso por PID67453, inode8848784/build9e734a8d82ffc860 em artifacts/Ignis.app/Contents/MacOS/ignis: app/processo preservados, sem reempacotar ou tocar janela. Nenhuma cópia histórica nova de build criada. Sem nova interação visual, gate manual obrigatório ou push neste incremento de infraestrutura; não inferir suporte a spawn/despawn ou escala601. Visão do mundo presente, bindings/componentes, contexto de origem/reserva e otimizações de escala continuam posteriores.

## EAF-88 — compactação da tangente (27/10/2026)

722testesRust release/all-targets,Clippy-Dwarnings,fmt/diff e13gruposMCPheadless (render,replay_inspection,selection) aprovados.3testes novos cobrem codec/quantização/corrupção/fonte e equivalência da projeção.960timings de128frames;138vistas×5candidatos passam,640/640repetições dos candidatos e60/60retornos totais exatos. Original falha64/128repetições estáticas; runner encerra1 e conserva dados, sem repetir até passar.552PNGs decodificados,40paresPNG iguais,1.656estadosDOM/DPRsimulados. Smoke legado prior/field16/16,8falhasconhecidas de reduced;24PNGs decodificados e removidos após lsof, JSON/log mantidos. Script offline usa prefixo check_, evitando descoberta indevida como cenário MCP; nenhum ajuste no runner funcional. Dados/limites/reprodução/checklist em [rich-relief-study.md](rich-relief-study.md#compactação-e-custo-da-tangente--resultados). Sem aceite humano, navegador nativo ou integração do candidato.

## EAF-88 — hierarquia de certificados locais (27/10/2026)

696testesRust release/all-targets, Clippy-Dwarnings, fmt e13gruposMCPheadless passaram.4.096casos de auditoria,144/144poses aprovadas contra importada1× e142/144contra fonte densa (mesmas duas falhas, nenhuma nova).576PNGs,64retornos idênticos,140pares global/local idênticos quando ambos escolhem proxy,2.048estadosDOM/DPRsimulados.100×perto3456×2234:1,166495→0,204714ms total,8/8rodadas favoráveis,proxy48→128frames/lote. Quatro fatores próximos0,205–0,226ms;40.920B adicionaisCPU. Pequenas regressões leves e memória da fonte permanecem. Protocolo/checklist/formatos/limites em [certificate-hierarchy-study.md](certificate-hierarchy-study.md). Sem aceite humano/browser nativo nem entrega no renderer funcional.

## EAF-88 — segunda malha externa (27/10/2026)

692testesRust release/all-targets, Clippy-Dwarnings, fmt e13gruposMCPheadless aprovados; recorte final do integrador também validado. Banco:144/144poses contra base,142/144contra fonte densa. Platô presente1728/perto, ausente3456/perto100× porfallback80/128frames; limites mantidos. Referência1/4/16/64spp em dez poses, ROI sem alterar viewport e convergência explícita.380PNGs,64pares de retorno e1.024estadosDOM/DPRsimulados. Relatório e limitações em [external-recovery-study.md](external-recovery-study.md); sem aceite humano ou entrega no renderer funcional.

## EAF-88 — bordas assimétricas (27/10/2026)

690 testesRust release/all-targets, Clippy-Dwarnings, fmt e13gruposMCPheadless aprovados. Ensaio1/2/10/100× em duas resoluções, perto/longe e luz/câmera variáveis:144/144 poses passam contra a base; quatro divergências anteriores contra fonte densa permanecem.2× perto nativo reduz tempo total9,9%, com mesma superfície e23,6% menos bytes de proxyGPU. Controles byte-idênticos nos demais fatores; sem ganho distante consistente ou aceite humano novo. Protocolo, custos, testes geométricos em duas fixtures e limitações em [border-recovery-study.md](border-recovery-study.md).

## EAF-88 — campo de normais por região (27/10/2026)

366 comparações entre três grades, duas resoluções, câmera/luz/retorno. Campo64 e128: zero falhas em122 casos cada; campo32: uma; sem campo:30 falhas por variante. Cobertura idêntica, diferenças de cor não nulas. 670 testesRust,Clippy-Dwarnings,fmt e13 gruposMCPheadless passaram;768 estadosDOM/DPRsimulados. Sem ganho líquido, sem aceite humano/browser nativo. Evidências, memória e reprodução em [normal-field-study.md](normal-field-study.md).

## EAF-88 — malha externa irregular (27/10/2026)

Boulder01/glTF66.122T;158comparaçõesRGBA32F idênticas em2resoluções com câmera/luz/retorno, mas só2T removidos no melhor caso. Controle geométrico sem limite de normais falhou11/60; redução forçada também falhou.664testesRust,Clippy-Dwarnings,fmt e13gruposMCPheadless passaram;384estadosDOM/DPRsimulados. Sem ganho robusto, sem aceite humano/browser nativo. Protocolo,memória,timings e limitações em [irregular-lod-study.md](irregular-lod-study.md).

## EAF-88 — compilador sem genealogia (27/10/2026)

`geometry_compiler_probe` recebe apenas malha embaralhada e emite IGC1/regiões/certificados offline. Pedra49.152→4.915.200triângulos de entrada mantém49.152na saída; controle de dobras100×819.200→9.145.288comparações/864gates passaram nas duas resoluções e distâncias com câmera/luz;32retornosRGBA32F idênticos.660testesRust,Clippy-Dwarnings,fmt/diff e13gruposMCP headless passaram;320estadosDOMsimulados/DPR, sem navegador nativo/aceite humano. Dados, protocolos, limitações de certificado/memória e timings confirmados em [geometry-compiler-study.md](geometry-compiler-study.md). Artefatos correntes em `artifacts/appearance-probe/geometry-compiler/`; sem mudanças funcionais ou de contrato.


## EAF-60 — identidade de instâncias (25/10/2026)

macOS Apple Silicon/Metal, cena46/replay47, build `aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|bc0d08b211893a6d`. Registro slot/geração/serial, escopo externo de execução e referências em entradas/casts de habilidades; mesma instância na ressurreição. Catálogo vivo ainda fixo. Contrato em [instance-identity.md](instance-identity.md), checklist antes/depois em [performance.md](performance.md).

- `cargo test --locked --release --all-targets`: **583 testes aprovados**, exit0;10 novos (4 registro,4 integração de identidade,2 ressurreição/casts). `artifacts/identity-tests.log`.
- `cargo fmt --all -- --check`, `cargo clippy --locked --release --all-targets -- -D warnings` e build release passaram. Logs `artifacts/identity-clippy.log` e `identity-build.log`. Aviso preexistente de futura incompatibilidade de block0.1.6 não é erro Clippy.
- `IGNIS_MCP_STARTUP_TIMEOUT=600 python3 scripts/run_verifications.py`: **54 cenários/221 grupos aprovados**, relatório completed/passed/native_window:false. `artifacts/identity-mcp.log` e `artifacts/mcp-validation/report.json`.
- Cenário identity:3 grupos via MCP real, tokens em activate/aim/aim_at/cancel, rejeição atômica de referências inválidas/de não atores, reset/edição/undo/redo/load, ressurreição, compatibilidade de aliases, replay pendente/completo/prefixo e capturas offscreen idênticas. PNG identity-live inspecionado. Primeira tentativa do cenário omitiu seed obrigatório em generate e foi corretamente rejeitada; fixture corrigida, recorte e regressão completa posteriores aprovados. Nenhuma correção de runtime atribuída a essa falha de teste.

Rust cobre reuso10000 vezes sem crescimento, capacidade, duplicata, overflow de geração/serial sem wrap, referências forjadas, escopo entre Engines, ordem de autoria, replay adulterado, entrada/cancelamento obsoletos e cast inválido sem custo/efeito. Isso prova o registro isolado e as fronteiras atuais de habilidades; não prova remoção/reuso real de atores, ainda não expostos.

Binário corrente atualizado em target/release/ignis. `lsof` encontrou a demo anterior ativa (PID67453, inode8848784) usando artifacts/Ignis.app/Contents/MacOS/ignis; app/processo preservados, sem reempacotar, encerrar ou controlar janela. O launcher .app ainda aponta ao build anterior9e734a8d82ffc860 até renovação após deixar de estar em uso. Nenhuma cópia histórica nova criada. Sem nova interação visual ou teste manual exigido neste incremento de infraestrutura; não inferir aceite físico de outra feature. Sem push/publicação, spawn/despawn ou escala601 validada.

## EAF-58 — ressurreição do mesmo indivíduo (25/10/2026)

macOS Apple Silicon/Metal, cena/replay46, build `aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|9e734a8d82ffc860`. Efeito direto no alvo morto, preserva corpo/posição, recursos e deadlines; nova vida invalida intenções antigas, limpa transições mesmo no mesmo tick e exige nova evidência de observadores. Contrato em [resurrection.md](resurrection.md), revisão estática antes/depois em [performance.md](performance.md).

- `cargo test --locked --release --all-targets`: **573 testes aprovados**, incluindo14 em resurrection e7 em vitality_perception (2 novos), com exit0. Log corrente: artifacts/resurrection-tests.log.
- `cargo fmt --all -- --check`, `cargo clippy --locked --release --all-targets -- -D warnings` e build release aprovados. Aviso preexistente de futura incompatibilidade de block0.1.6 permanece separado. Logs: artifacts/resurrection-clippy.log e resurrection-build.log.
- `IGNIS_MCP_STARTUP_TIMEOUT=600 python3 scripts/run_verifications.py`: **53 cenários/218 grupos aprovados**, relatório completed/passed/native_window:false. artifacts/mcp-validation/report.json e artifacts/resurrection-mcp.log. Uma expectativa anterior de aim_observation exigia o objeto sem life_index; atualizada para o contrato46, seguida da regressão completa aprovada.
- Cenário novo3 grupos: autoria/schema MCP, vida/saldos/cooldowns/recibos, rejeições atômicas, cura comum rejeitando morto, reset/undo/redo/load, NPC explícito e cadáver oculto, replay completo e prefixo durante cast, comparação de capturas GPU. Demo examples/npc-resurrection.scene.json corresponde ao arquivo salvo via MCP; captura resurrection-demo.png inspecionada.

Rust cobre ainda corrida entre curadores sem pagamento duplo, cancelamento no prazo, falta de saldo, falha tardia no orçamento de ruído com rollback, overflow do contador, fila/cast de vida antiga, alvo sem vitalidade, história limitada256, morte/retorno no mesmo tick, descarte de periódico e área encerrada pela morte, projétil que colide após retorno, motor de voo reativado e morte seguinte. Observação oculta preserva crença/budgets; relato de vida anterior não substitui identidade recém-confirmada. Não há acesso de teste aos handles privados: conservação do corpo é evidenciada pelo código sem recriação e pelo comportamento físico/posição.

Build/app correntes atualizados após lsof sem consumidores: target/release/ignis e artifacts/Ignis.app/Contents/MacOS/ignis compartilham inode8848784,10.420.160bytes. Sem cópia histórica nova. Nenhuma janela aberta/focada/fechada durante a validação; teclado nativo e aceite humano continuam separados. EAF-58 permanece In Progress aguardando esse aceite; não implica aceite retroativo de EAF-56 nem fechamento de NPCv1/M08/M09.

## EAF-56 — triagem por efeitos dissipáveis

macOS Apple Silicon/Metal, cena/replay45, build `aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|6b42189a1230c68f`. `require_dispel_match` consulta apenas candidato com evidência aceita; seletor compartilhado com plano transacional, máscaras periódicos/buffs, categoria/proteção efetiva, filtro antes do ranking e revalidação antes do commit. Contrato em dispel-triage.md; checklist antes/depois em performance.md.

- 14 testes focados novos passaram na versão final; 62 regressões focadas de atributos/dissipação/triagem de vida também passaram na extração inicial do seletor.
- Clippy release/all-targets `-D warnings`, build release e fmt passaram. Aviso preexistente de futura incompatibilidade de block0.1.6, sem erro Clippy.
- Primeira rodada MCP do recorte passou4 grupos e captura inspecionada. Revisão posterior identificou buff expirado ainda no pool antes da fase de limpeza; filtro explícito de prazo e teste de fronteira acrescentados. Rodada Rust geral anterior foi interrompida enquanto compilava; rodada Rust final concluída com exit0: **557 testes release/all-targets aprovados**, incluindo o exemplo de locomoção; evidência em artifacts/dispel-triage-tests.log. Clippy/build/fmt repetidos após a correção passaram.

Regressão MCP final: **52 cenários/215 grupos aprovados**, report completed/passed/native_window:false. `IGNIS_MCP_STARTUP_TIMEOUT=600 python3 scripts/run_verifications.py`; artifacts/dispel-triage-mcp.log e artifacts/mcp-validation/report.json. Novo cenário4 grupos: autoria/schema, protegido versus vida cheia afetada, independência de memória, buff removível/cancel sem custo, erro atômico/ocultação/undo/reset/load, replay completo/prefixo e pixels idênticos. Captura final dispel-triage-demo.png inspecionada. Demo autorada/salva pelo MCP e copiada para examples/npc-dispel-triage.scene.json.

Build/app correntes renovados após lsof sem consumidores: target/release/ignis e artifacts/Ignis.app compartilham inode8842024,10.369.568bytes; nenhuma cópia histórica nova. Ainda não representa aceite de teclado nativo/demo humana.

Cobre vida cheia afetada, protegido/categoria incompatível, buff sem vitalidade, múltiplos operadores sem duplicação, máscara mista, visão oculta/perdida, autoapoio e alvo explícito, expiração/proteção dinâmica/cancelamento sem custo, remoção no mesmo tick antes da cobrança, projétil já lançado, legado e erro atômico. Nenhuma janela do usuário aberta/focada/fechada. Aceite humano de EAF-55 recebido; EAF-56 terá gate próprio.


## EAF-55 — triagem de aliados observados

Plataforma: macOS Apple Silicon/Metal; cena/replay44; build `aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|dcd7d8faa6b81222`.

- `cargo fmt --all -- --check`, `cargo clippy --locked --release --all-targets -- -D warnings` e `cargo build --locked --release`: passaram. Aviso preexistente de futura incompatibilidade de block0.1.6, sem erro Clippy.
- Tests/triage:12 testes passaram após corrigir uma expectativa temporal: conferia posição após o NPC já ter curado e escolhido outro aliado; agora confere movimento durante a aproximação e cura depois. Código de comportamento não precisou mudar por essa falha.
- `cargo test --locked --release --all-targets`: 543 testes aprovados, incluindo o teste do exemplo de locomoção; processo concluído com exit0. Evidência: artifacts/triage-tests.log.
- `IGNIS_MCP_STARTUP_TIMEOUT=600 python3 scripts/run_verifications.py`:51 cenários/211 grupos, report completed/passed/native_window:false em artifacts/mcp-validation/report.json. Nenhuma janela aberta/focada/fechada.
- Cenário triage:4 grupos, autoria real da demo pelo MCP, menor fração vital, alvo sensorial independente, compromisso/cancelamento sem custo, aliado oculto, erro atômico, undo/redo/reset/load, replay completo/prefixo e PNG idêntico. Captura final triage-demo.png inspecionada; câmera mostra verde, aliados e a parede.

Rust cobre também confirmação visual, som anônimo, FOV/perda com memória ainda válida, XYZ, alcance antes do ranking, desempate, inclusão própria, aproximação física, território/morte, revalidação no mesmo tick antes de custo, versão/combinações e edição atômica. Contrato e teste humano em [ally-triage.md](ally-triage.md).

Incidente de infraestrutura: a primeira regressão MCP parou em initialize do cenário attributes após120s, antes de comandos de gameplay; encerramento também falhou, deixando processo próprio órfão. Amostras de executáveis próprios mostraram `_dyld_start` antes do Rust. Client agora encerra/recolhe seu próprio processo após falha de transporte e valida orçamento opt-in1..600s antes de spawn (default120); os budgets de comandos continuam iguais. Um servidor de teste deliberadamente sem resposta comprovou cleanup/fechamento dos streams; prazo inválido não lança processo. Regressão completa foi repetida com600s e passou. Isso não altera segurança do sistema nem comprova melhoria de inicialização/FPS.

Comparação com cópia byte a byte no disco interno também aguardou inicialização e passou12/12; não se atribuiu causa ao volume externo. Cópia de4.359.856bytes removida depois de confirmar ausência de consumidores. Mantido somente build corrente target/release/ignis e artifacts/Ignis.app com o mesmo inode8838022,10.369.616bytes; pacote atualizado após lsof sem consumidores. Demonstração/teclado nativo e aceite humano continuam separados; EAF-54 aceito, EAF-55 aguarda aceite próprio.


## EAF-54 — auras pulsantes ancoradas no autor (25/10/2026)

Build `10cd1992cb16f079`, macOS Apple Silicon/Metal, cena/replay43. `area.persistence.anchor` fixed default/source opt-in; source exige direct+self_actor, acompanha centro físico XYZ e preserva relógio/custo/identidade. Pulso usa snapshot pré-física e guarda centro no recibo; ativo/guia sincronizam após Rapier. end_on_source_death false acompanha corpo morto, true encerra. Contrato/tradeoffs em auras.md/architecture.md; EAF-53 recebeu aceite humano e foi concluído no Kordo.

**531 testes Rust release/all-targets aprovados** (11 novos), fmt, Clippy release/all-targets `-D warnings` e build release aprovados. Cobertura nova: andar para dentro/fora do raio, fixed/source, cadência/refresh/independent/custo, recibo pré-física versus guia pós-física, NPC em voo XYZ sem conhecimento secundário, morte/queda do corpo, oclusão mudando ao contornar parede, cura de aliados, buff persistindo após saída, combinações/versão/edição inválida atômica, legado, auto-inclusão e replay/prefixo. Aviso preexistente de futura incompatibilidade de block0.1.6 permanece separado. Houve espera de inicialização do executável vitality no dyld, antes do harness; após liberação os15 testes passaram. Diagnóstico de inicialização não mede performance/FPS da engine.

Regressão MCP completa **50 cenários /207 grupos aprovada**, todos headless, incluindo render/replay e versões anteriores. Relatório artifacts/mcp-validation/report.json confirmado completed/passed/native_window:false. Novo cenário aura tem4 grupos: autoria, movimento de portador para dentro/fora de hostis com100/70/40, custo/prazo/identidade preservados, comparação com zona fixa, cura/refresh, NPC que ativa aura própria e retoma patrulha aérea, combinações inválidas sem mutação, undo/redo/load/reset, alternância de guia e captura. Entrada pendente/full/prefix replay e PNGs repetidos/prefixo idênticos. Ajuste de fixture: raio demonstrativo3m inclui os três alvos ao fim do deslocamento de60ticks; raio2,5m legitimamente incluía só o central naquele pulso.

Demo examples/npc-aura.scene.json autorada/salva pelo MCP no workspace isolado e copiada para exemplos. Captura aura-demo.png inspecionada: aura azul-claro no jogador laranja junto dos alvos, NPC verde em altitude com aura própria, parede e cinza preservados.1 dano3m em hostis;2 cura60/4m em aliados e portador;3 zona fixa2m que também machuca jogador;4 buff;5 cura instantânea;6 dissipa resistências. Auras10s/pulsos1s, zona fixa6s. P/WASD/J/X/R; nenhuma janela aberta/focada/fechada. Aceite físico próprio pendente, EAF-54 In Progress. Não entrega aura em terceiros, enter/exit contínuos, revogação ao sair ou triagem tática.

Checklist estático antes/depois em performance.md: mesmo pool/rings, enum compilado, até16 consultas de corpo após física e posição do snapshot no pulso; sem alocação de coleção/mapa/clone de cena por atualização. Lookups de handles existentes e invalidação do cache de linhas mantidos conscientemente. Sem novo scheduler/lock/dependência ou alegação de FPS.

Build corrente target/release/ignis e artifacts/Ignis.app compartilham inode8778372, **10,318,496 bytes**, launcher renovado após lsof sem consumidores. Sem cópia histórica adicional, push ou publicação; processo visual do usuário preservado. Logs estáveis artifacts/rust-tests.log, clippy.log, fmt.log, build.log, verification-run.log; cenário artifacts/mcp-validation/artifacts/aura-verification.json e logs/aura.*.

## EAF-53 — áreas persistentes 3D (25/10/2026)

Build `a6c43b9164a64f71`, macOS Apple Silicon/Metal, cena/replay42. `area.persistence` cria volume fixo na entrega direta/impacto, com duração, intervalo, pulso inicial, refresh/independent e término por morte do autor. Seleção física renovada por pulso e transação individual preservam atributos atuais/privacidade NPC. Entrada/saída só afeta amostragem seguinte, sem callbacks contínuos; custo único. Capacidade16 revalidada antes do custo direto ou no impacto pago. Contrato/tradeoffs em persistent-area.md/architecture.md.

**520 testes Rust release/all-targets aprovados** (21 novos); fmt, Clippy release/all-targets `-D warnings` e build release aprovados. Aviso preexistente de futura incompatibilidade de block0.1.6 permanece separado. Testes novos cobrem prazo exclusivo/pulso inicial, refresh/reposicionamento/identidade, sobreposição, origem morta/inicial/postumamente, capacidade cheia/refresh/liberação no prazo/preenchimento durante cast/impacto sem reserva, movimento físico de entrada/saída e travessia entre pulsos, XYZ/oclusão/falha individual, buff expirando entre pulsos/cura, cancelamento, duração iniciada no impacto, versões/combinações, NPC com ponto congelado sem conhecimento secundário, rings/guias/prefixo/replay.

Regressão MCP completa **49 cenários /203 grupos aprovada**, headless. Cenário persistent_area com4 grupos: autoria/schema/entrada pendente, seis pulsos100/70/40 com parede e custo único, expiração exata, refresh/reposicionamento e IDs, zona própria independente com fuga física, projétil antes do volume, buff adquirido após instalação alterando próximo pulso, erros atômicos, edit/undo/redo/load/reset/cursor/guia. Replay completo/prefixo/PNG reproduzidos; capturas repetidas idênticas. Relatório `artifacts/mcp-validation/report.json` confirmado completed/passed/native_window:false. Comparação de coordenada MCP usa tolerância f32 explícita; sem exigir igualdade decimal exata de0.8 serializado.

Demo `examples/npc-persistent-area.scene.json` autorada/salva pelo MCP no workspace isolado e copiada para exemplos. Captura offscreen persistent-area-demo.png inspecionada: esfera verde-lima durante vida, jogador laranja, alvos vermelho/azul/roxo e cinza protegido por parede, barras visíveis.1 projétil cria zona independente;2 cursor renova/relocaliza zona;3 zona própria independente inclui jogador (sair evita próximos pulsos);4 poder;5 cura;6 dissipa resistências pessoais. P/WASD/J/X/R/clique. Duração6s, pulso inicial e depois a cada1s; guia nominal não é VFX nem prova de alcance efetivo. Nenhuma janela aberta/focada/fechada. Teclado/aceite humano pendentes; EAF-53 permanece In Progress.

Revisão estática antes/depois registrada em performance.md: pool16/ring128, array9 e plano anterior por alvo, deadlines/índices compilados, sem coleção por pulso, cache/linhas reutilizados. Scans limitados e cópias transacionais mantidos conscientemente; sem profiling/FPS alegados. Sem aura móvel, callbacks enter/exit, dispel de volumes ou tática nova.

Build corrente `target/release/ignis` e `artifacts/Ignis.app` compartilham inode8773937, **10,335,472 bytes**, launcher renovado após lsof sem consumidores. Uma versão corrente, sem cópia histórica adicional ou publicação. Logs estáveis em artifacts/rust-tests.log, clippy.log, fmt.log, build.log, verification-run.log e artifacts/mcp-validation/logs/persistent_area.*; resumo específico artifacts/mcp-validation/artifacts/persistent-area-verification.json.

## EAF-52 — área instantânea esférica 3D (25/10/2026)

Build `9f6cbc70e1cfa4b1`, macOS Apple Silicon/Metal, cena/replay41. Configuração `area` no catálogo e mesmo dispatcher para autoria, execução e inspeção. Centro no emissor/alvo-âncora, Point explícito ou contato do projétil; raio XYZ de centros, relação/auto-inclusão/cap/oclusão configuráveis. Seleção congelada por distância²/ID; custo único e atomicidade por destinatário. Efeitos anteriores podem alterar o contexto da próxima transação. Contrato/tradeoffs em instant-area.md/architecture.md.

**499 testes Rust release/all-targets aprovados** (21 novos), além de fmt, Clippy release/all-targets -D warnings e build release. Cobertura: XYZ/fronteira, cap/empate/autoria invertida, relações/origem, chão/parede invisível/interior de sólido, Point/range, vazio, cancel, falha com rollback individual, cadáveres, buff pool64, dano/cura/buff/dispel/interrupção, origem morta afetando contexto posterior, projétil/parede/expiração, NPC com ponto congelado/esquiva/conhecimento, schema/erros, ring64/guia/replay. Duas fixtures foram ajustadas: personagens intermediários bloqueavam visão antes do cast, portanto alvos foram reposicionados para isolar interrupção e esquiva sem remover as regras de percepção.

Regressão MCP completa aprovada: **48 cenários / 199 grupos**, todos sem janela. Novo cenário area cobre dano100/70/40 com cobrança única, cinza protegido pela parede, buffs ofensivos110/80/50, projétil/cura coletiva/dispel, falha individual sem perda de dano nos vizinhos, erros atômicos, undo/redo/removal, reset/save/load, Point e cursor/plano pelo MCP. Pending/preparo/full/prefix replay reproduzidos; PNG repetido/prefixo idênticos; captura/resize/alternância de guia preservam simulação. Relatório completed/passed em artifacts/mcp-validation/report.json; roteiro e detalhes em scripts/verify_area.py e artifacts/mcp-validation/artifacts/area-verification.json.

Demo examples/npc-area.scene.json autorada/salva via MCP e copiada do workspace isolado. Captura final area-demo.png inspecionada: esfera laranja, jogador laranja, alvos vermelho/azul/roxo e cinza atrás da parede verde, barras visíveis.1 projétil explosivo selecionado;2 área no cursor;3 área própria;4 poder próprio;5 cura coletiva;6 dissipação na área do cursor. P/WASD/J/X/R/clique. Atalhos físicos e aceite humano seguem pendentes; nenhuma janela manipulada. Não entrega VFX, área persistente ou triagem/tática nova de NPC.

Checklist estático antes/depois em performance.md: seleção sem heap em array9, rings64/256, plano existente por destinatário, distâncias², círculo unitário e buffers de render reutilizados. Oclusão apenas no interior de corpos fixed/dynamic; atores não são escudos, flags visuais não alteram resultado. Custos de scans/cópias por transação mantidos com motivo; sem profiling ou alegação de FPS.


Build corrente target/release/ignis e artifacts/Ignis.app compartilham inode, **10,284,464 bytes**, launcher renovado após lsof sem consumidores. Sem cópias históricas adicionais ou publicação. EAF-52 permanece In Progress para aceite manual. Logs: artifacts/rust-tests.log, clippy.log, fmt.log, build.log e verification-run.log.

## EAF-51 — modificadores de dano causado (25/10/2026)

Build `8a76c623f85309d4`, macOS Apple Silicon/Metal, cena/replay40. Quarto atributo nativo combat.damage_dealt.multiplier, no autor original, com tags do componente/ID da habilidade. Dano original × fator causado × fator recebido em i128, divisão única por SCALE² por parcela; soma antes de vida. Defaults neutros, máximo100× por fator, soma bruta por efeito<=10¹⁶ e teto vital10¹². Alias multiplier/damage_multiplier continua recebido; MCP mostra dealt_multiplier/received_multiplier separados. Contrato/tradeoffs em outgoing-damage.md e architecture.md.

Validação Rust aprovada: fmt, **478 testes release/all-targets** (15 novos), Clippy all-targets -D warnings e build release. Aviso preexistente de futura incompatibilidade de block0.1.6 não é falha do Clippy. Os novos testes cobrem fatores e proprietários, arredondamento único, filtros/tema/ID, extremos/teto/overkill, snapshot/rollback, projétil com buff ganho/expirado, pulsos periódicos/canal, morte do autor com base preservada, dissipação, source-recipient/NPC, dano zero com cura e migração/capacidade61 customizadas. Registro tem65 chaves totais,4 nativas; colisão legada com nova chave reservada exige renomeação explícita antes da carga.

Cenários MCP focados damage/outgoing aprovados. Outgoing tem quatro grupos: matriz100/70/40→110/80/50, inspeção/contexto/dissipação/replay/prefixo/PNG; erros atômicos de chave/base e undo/redo; poder ganho durante voo, pulsos70/80/70 com ganho/remoção e expiração exata; demo autorada/salva/carregada/resetada via MCP. Durante montagem do roteiro, corrigidas expectativas temporais: uma entrada enviada no mesmo tick do prefixo aparece como pendente no replay; separada por um step. Buff instalado no tick0 expira no600, então último tick válido é599. Nenhuma mudança do runtime foi necessária para essas correções do teste.

Regressão completa final aprovada: **47 cenários / 195 grupos MCP**, todos headless, incluindo render, replay e sistemas anteriores. Relatório completed/passed confirmado em artifacts/mcp-validation/report.json.

Demo examples/npc-outgoing.scene.json, gerada no workspace isolado pelo MCP:1 direto,2 projétil,3 três pulsos,4 poder próprio +25%fire por600ticks/10s,5 dissipar power do selecionado,6 cura500. P/WASD/J/X/R/clique. NPCs vermelho/azul/roxo preservam proteções e ficam parados; sem tática elemental ou VFX novos. Captura offscreen outgoing-active.png inspecionada, com jogador laranja e três alvos/barras; prefixo reproduziu os mesmos pixels. Entrada física/aceite humano separados; nenhuma janela aberta/focada/fechada.

Checklist estático antes/depois em performance.md. Uma consulta extra por componente, índices/máscaras compilados e um i64 extra por Breakdown fixo; sem heap no cálculo, cache/thread/dependência/GPU novos. Buffs vigentes por transação/impacto/pulso; não congela poder no lançamento. Fonte morta perde seus buffs, mas entregas existentes usam bases remanescentes. Sem profiling/ganho de FPS alegado. Evidências em artifacts/rust-tests.log, clippy.log, build.log, verification-run.log e artifacts/mcp-validation/report.json/logs/outgoing.*/artifacts/outgoing-verification.json.

Build/app corrente: target/release/ignis e artifacts/Ignis.app compartilham inode, **10,166,240 bytes**; launcher renovado após lsof sem consumidores, sem cópias históricas adicionais. Commit local, sem push/publicação. EAF-51 permanece In Progress para aceite físico; não encerra M08/M09, área, respawn, triagem ou callbacks on-hit.

## EAF-50 — dano composto e tags do jogo (25/10/2026)

Build `cb94531306f9951a`, macOS Apple Silicon/Metal, cena/replay 39. Efeito damage_components com quantidades/tags independentes, registro explícito de tags e referências validadas. Cada componente consulta resistência atual com ID da habilidade e tags próprias; soma precede uma única alteração de vida por efeito. 40 de fire +60 de ice com redução de50% do gelo resulta70. Tags temáticas da habilidade não vazam ao dano. Registro e semântica antigos são migrados: dano escalar classificado vira um componente explícito, preservando IDs/comportamento. Contrato/tradeoffs em damage-components.md/architecture.md.

Validação final aprovada: fmt, **463 testes Rust release/all-targets** (22 novos), Clippy all-targets `-D warnings`, build release e **46 cenários/191 grupos MCP**, todos headless. Aviso preexistente de futura incompatibilidade de block0.1.6 permanece separado do Clippy. Durante montagem dos testes, corrigidas duas fixtures: alvo sem pool também perdeu grants que exigiam esse pool; alvo oculto passou a usar parede, em vez de desabilitar o próprio NPC e receber ActorUnavailable. No MCP, comparação de undo/redo passou a usar catálogo normalizado com defaults, em vez do input parcial. Testes focados e rodada completa final passaram.

Os22 testes novos cobrem40+30=70 com uma aplicação/custo, ordem e morte única, classificação múltipla versus parcelas, regra magical compartilhada, isolamento de tema/escalares, contexto de inspect_attribute com tags explícitas e filtro de ID, referências desconhecidas/duplicadas, quantidades/versões/orçamento, arredondamento por componente, teto numérico separado de overkill, rollback integral, snapshot de buffs, resistência no impacto/pulso/canal, cancel antes do prazo, migração38 e roundtrip, tags arbitrárias, oito componentes entre efeitos, NPC oculto/observado e destinatário source. Recibos e replay validam a decomposição. Os24 testes anteriores de atributos também exercitam a migração do contrato38.

Quatro grupos MCP verificam autoria/schema, recibo e valores exatos, erros atômicos, permutação/undo/redo, resistência instalada durante voo, três pulsos de70, demo comparativa, dispel/restauração, save/load/reset, replay final/prefixo e PNGs repetidos/reproduzidos idênticos. Cenário completo inclui render e todos os sistemas anteriores. Captura offscreen `artifacts/mcp-validation/artifacts/damage-demo.png` inspecionada: jogador laranja; vermelho sem proteção, azul com redução parcial e roxo com imunidade de gelo; barras de vida e buffs visíveis. MCP confirmou danos100/70/40 e, após dissipação do azul,100 novamente.

Demo `examples/npc-damage.scene.json` autorada/salva pelo MCP e copiada do workspace isolado. P inicia; clique seleciona;1 direto40fire+60ice,2 projétil,3 três pulsos,4 resistência,5 dissipa,6 cura500; WASD/J/X/R. NPCs aplicam proteção própria no início, com prazo216000ticks/cooldown3600; ficam parados. Não há VFX/áudio elemental. Entrada física/aceite humano ainda separados; nenhuma janela foi aberta/focada/fechada durante a validação.

Revisão estática antes/depois em performance.md. Até8 componentes por habilidade; máscara/índices compilados, produto i128/soma i64, sem heap no cálculo. Decomposição em array fixo por plano/recibo compartilhado entre efeitos, em vez de oito arrays em cada Applied; removido total duplicado do pacote e tirados beneficiário/ID da habilidade do loop de componentes. Crescimento fixo do anel, preflights e scan limitado do avaliador mantidos explicitamente. Sem profiling ou ganho de FPS alegado.

Evidências: artifacts/damage-tests.log, damage-mcp.log, rust-tests.log, clippy.log, build.log, verification-run.log; artifacts/mcp-validation/report.json, logs/damage.*, artifacts/damage-verification.json e capturas/replays no workspace isolado. Build corrente target/release/ignis e artifacts/Ignis.app compartilham inode, **10.166.192 bytes**, launcher renovado após lsof sem consumidores. Sem cópias históricas adicionais, push ou publicação. EAF-50 In Progress para aceite manual; não encerra M08/M09, dano causado, tática elemental/NPC, callbacks on-hit, área, equipamentos ou respawn.


## EAF-49 — atributos extensíveis e buffs temporários (24/10/2026)

Build `4a52f365f8da4408`, macOS Apple Silicon/Metal, cena/replay 38. Registro por chave com valores bool/número fixo, bases por entidade, operações set/add/multiply/min/max, prioridade e filtros por tags/ID de habilidade. Nomes compilados em índices/máscaras; executor genérico sem casos por atributo customizado. Consumidores iniciais: interrupção do cast, dissipação dos DoTs/HoTs pelo autor e multiplicador de dano recebido. Buffs aplicados pelo mesmo executor direto/projétil, com prazo, refresh/independent, proveniência, capacidade e commit atômico. Contrato e tradeoffs em attributes.md/architecture.md; revisão estática antes/depois em performance.md.

Validação final aprovada: fmt, **441 testes Rust release/all-targets** (24 novos), Clippy all-targets `-D warnings`, build release e **45 cenários/187 grupos MCP**, todos headless. Aviso preexistente de futura incompatibilidade de block 0.1.6 permanece separado do Clippy. Durante integração foram corrigidos um literal i64 na fixture, o tipo do responsável nos recibos de buff e uma chamada antiga de inspeção em tests/dispel.rs que precisava do tick atual. Nenhuma dessas falhas permanece na rodada final.

Os 24 testes novos cobrem composição/tipos/limites/base customizada, filtros/prioridades, expiração exata/refresh/sobreposição, fonte versus beneficiário, proteção de efeitos existentes, morte, rollback de buffs/vida/custos, pool cheio/reserva pendente/refresh/reuso após dispel, impacto espacial pago sem reembolso, resistência consultada a cada pulso/impacto, NPC sem posição oculta, objetivo automático e anel de recibos limitado. Quatro grupos MCP verificam schema/inspeção/base/efetivo/proveniência, imunidade a gelo e vulnerabilidade a fogo, imunidade geral/restauração por dispel, erros atômicos/undo/redo/reset/load, replay final/prefixo e igualdade integral de capturas repetidas e reproduzidas. Suíte completa inclui render e demais sistemas.

Demo `examples/npc-attributes.scene.json` autorada/salva pelo MCP e copiada do workspace isolado. Captura offscreen `artifacts/mcp-validation/artifacts/attributes-demo.png` inspecionada: jogador laranja com barra de buff verde-lima e canal ciano; roxo interrompe, vermelho/azul lançam projéteis, verde usa antídoto. Teclas 1 proteção de cast/DoT por 10s; 2 gelo imune/fogo ×1,5; 3 imunidade geral; 4 cura canalizada; 5 veneno no selecionado; 6 remove buffs próprios. P/WASD/J/X/R/clique. No MCP, 4 sozinho foi interrompido, 1+4 permaneceu canalizando e o veneno protegido persistiu até a expiração do buff do autor, sendo então dissipado. Entrada física e aceite humano continuam pendentes; nenhuma janela foi aberta/focada/fechada.

Evidências estáveis: artifacts/rust-tests.log, clippy.log, build.log, verification-run.log; artifacts/mcp-validation/report.json, logs/attributes.*, artifacts/attributes-verification.json e capturas/replays dentro do workspace isolado. Build corrente target/release/ignis e artifacts/Ignis.app compartilham inode, **10.099.248 bytes**, launcher renovado depois de lsof sem consumidores. Sem cópias históricas adicionais, push ou publicação.

EAF-49 permanece In Progress para aceite manual. Registro genérico não significa todos os consumidores integrados: física/render/custos/vida máxima e outros tipos ainda exigem adaptadores e invalidação próprios. Entrega de habilidade continua limitada a atores; teste da API genérica também cobre entidade não ator. Propriedade dispellable do próprio buff é estática; proteção dinâmica integra agendas periódicas. Sem ABI, tática de buff, área, equipamentos, respawn ou fechamento de M08/M09. Scans limitados de regras × instâncias e aumento fixo dos planos/recibos foram mantidos explicitamente; sem profiling ou ganho de FPS alegado.


## EAF-48 — interrupção externa e proteção (24/10/2026)

Build06d6eaa6300aaa98, macOS Apple Silicon/Metal, cena/replay37. Efeito interrupt target-only direto/projétil com require_active; interruptible defaulttrue na definição. Seleção por ID da execução atual, revalidada no commit/impacto; custo/efeitos/interrupção atômicos, múltiplos operadores sem duplicação. Recibos correlacionam vítima/responsável. Interrupção preserva pulsos/custos/cooldown anteriores e não recolhe projéteis/agendas. Barreira de um tick impede reinício imediato; retry automático respeitado. Proteção não impede cancel próprio, movimento, morte ou perda das condições do canal.

Validação final aprovada: fmt, **417 testes Rust release/all-targets** (18 novos), Clippy all-targets `-D warnings`, build release e **44 cenários/183 grupos MCP**, todos headless. Aviso preexistente de futura incompatibilidade de block0.1.6 permanece separado do resultado do Clippy. Na primeira execução do cenário novo, a fixture de projétil usou `mode` em vez de `kind`; schema rejeitou antes de mutar. Corrigida a fixture, cenário focal e duas rodadas completas passaram. A última rodada foi necessária após simplificar o plano transacional; números/hash acima são do código final.

Testes novos: preparação interrompida sem custo da vítima, ausência/proteção required/opcional, duplicação e rollback por operador posterior, dano letal com atribuição única, canal on_start/per_pulse preservando efeitos passados, cancel/movimento/morte de protegido, substituição da execução durante preparo/voo, impacto espacial protegido/interceptor físico, privacidade atrás de parede, barreira mesmo tick/retry NPC, ordem entre pulso/commit preparado/nova ordem, legado/composições inválidas, falta de saldo, próprio ator inelegível e persistência de disparos/agendas anteriores.4 grupos MCP verificam schema/recibos/proteção, erro atômico/undo/redo/reset/load, custo de projétil/falha sem reembolso, interação NPC/jogador, replay final/prefixo e igualdade de PNGs repetidos/replay. Suíte completa inclui render e demais sistemas.

Demo examples/npc-interrupt.scene.json autorada e salva pelo MCP, copiada do workspace isolado. Captura offscreen interrupt-demo.png inspecionada: roxo com preparação amarela, azul com canal ciano protegido, vermelho parado à direita do jogador laranja.1 interrupção direta;2 projétil;3 cura própria canalizada;4 mesma cura protegida. Vermelho interrompe o jogador sob visão/alcance4m; não persegue nem causa dano. P/WASD/J/X/R/clique; parâmetros e sequência de teste em interrupts.md. Teclado/foco nativo e aceite manual continuam separados.

Checklist estático antes/depois em performance.md. Revisão final substituiu array de9 IDs e loop de commit por único Option<u64> do destinatário; consulta candidata por referência. Sem nova alocação/coleção por interrupção, sem dependência/thread/I/O/GPU novo. Aumento fixo de recibos para proveniência e planos anteriores de vida/saldo mantidos com motivo. Sem profiling/FPS alegado.

Evidências finais: artifacts/rust-tests.log, clippy.log, build.log, verification-run.log; artifacts/mcp-validation/report.json e logs/interrupt.*, artifacts/interrupt-verification.json, interrupt*.png/replays no workspace isolado. Build corrente target/release/ignis e artifacts/Ignis.app compartilham inode,9.761.120bytes; launcher substituído após lsof sem consumidores. Sem versões históricas adicionais, janela/foco/push/publicação. EAF-48 In Progress para aceite humano; EAF-47 mantém seu aceite próprio pendente. Não conclui silêncio por duração, modificadores, área, triagem, respawn, NPCv1 ou M08/M09.


## EAF-47 — canalização sustentada (24/10/2026)

Build597761534ff6ae35, macOS Apple Silicon/Metal, cena/replay36. Definição channel opcional com intervalo/pulsos/cost_mode explícitos; preparação seguida de canal direto que ocupa o ator. Custo on_start ou per_pulse, cooldown na entrada, primeiro pulso atrasado; cancel/falha preservam efeitos/pagamentos anteriores. Mesmo executor, revalidação contínua de fonte/alvo/visão/alcance/condição, política/retry/movimento existentes e inspeção MCP. Barra ciano decrescente durante canal, amarela crescente durante preparação. Sem scheduler/ABI/dependência nova.

Validação final aprovada: fmt, **399 testes Rust release/all-targets** (16 novos), Clippy all-targets `-D warnings`, build release e **43 cenários/179 grupos MCP**, todos headless. Aviso preexistente de futura incompatibilidade de block0.1.6 separado do resultado do Clippy. Nenhuma falha nos testes deste incremento; ajuste de script de autoria para o nome `below` do helper de política antes de executar.

Testes novos: prazos exatos/atraso/recibos/cobrança única, per_pulse/esgotamento, cancel no prazo antes da entrada e entre pulsos, free/interrupt/lock, perda de alcance e visão real com parede, morte de fonte/alvo e autoletal, NPC pré-pago com saldo zero/política, custo não financiado pelo pulso, falha posterior de dispel sem dano/débito parcial, ocupação mesmo com cooldown expirado, cancel de ID antigo, limites/legado/composição, orçamento de ruído e interrupção por limiar vital.4 grupos MCP cobrem schema/phase/progresso, transações/cancel, erro atômico/undo/redo/reset, demo salva/reaberta, NPC/executor, replay final/prefixo e PNGs idênticos. Suíte inclui render e demais sistemas anteriores.

Captura offscreen artifacts/mcp-validation/artifacts/channel-demo.png inspecionada: jogador laranja e azul com barra ciano, roxo passivo, vermelho e cobertura. Não há feixe gráfico nesta etapa. Demo examples/npc-channel.scene.json autorada/salva por MCP e copiada do workspace isolado:1 dano livre por pulso;2 cura própria paga na entrada/mover cancela;3 cura de alvo por pulso/bloqueia movimento;4 veneno independente. Azul cura própria desde50hp; vermelho aproxima e canaliza dano sob visão. P/WASD/J/X/R/clique; detalhes em channeling.md. Teclado/foco nativo e aceite manual continuam gates separados.

Checklist estático antes/depois em performance.md: estado Copy limitado, consulta contínua sem construir plano entre pulsos, custo compilado/mesma transação, anéis limitados, guia com prazos numéricos sem lookup de nomes, sem nova alocação por pulso. Tamanho fixo maior do Cast e consulta de oclusão por tick mantidos explicitamente; sem profiling/FPS alegado. Logs estáveis artifacts/channel-tests.log, rust-tests.log, clippy.log, build.log, verification-run.log; relatório artifacts/mcp-validation/report.json, logs/channel.*, artifacts/channel-verification.json e channel*.png/replays no workspace isolado.

Build corrente target/release/ignis e artifacts/Ignis.app compartilham inode,9.726.800bytes, substituído após lsof sem consumidores. Sem cópias históricas/janela/foco/push/publicação. EAF-47 In Progress para aceite humano; continuidade após EAF-46 não comprova novo teste manual. Não conclui modificadores, área, triagem, respawn ou M08/M09.


## EAF-46 — proteção explícita de efeitos (24/10/2026)

Build b11eec4bef7353b1, macOS Apple Silicon/Metal, cena/replay35. `dispellable:false` independente de categoria, padrão true preservando cenas antigas. Protegidos são excluídos antes de limite/require_match; ausência removível falha sem custo direto, projétil continua pago após lançamento. Pulsos/refresh/expiração/morte preservados. Autoria/schema MCP, inspeção de grants/agendas e persistência/replay usam o contrato único.

Validação aprovada: fmt, **383 testes Rust release** (26 de dissipação, seis novos), Clippy all-targets `-D warnings`, build release e **42 cenários/175 grupos MCP**, todos headless. Aviso preexistente de futura incompatibilidade de block0.1.6 continua separado. Novos testes cobrem mistura protegidos/removíveis sem consumir limite, require_match/opcional/rollback, expiração do removível durante cast/voo, NPC self-antídoto, refresh/pulsos/expiração/morte, default/roundtrip/versão/schema. Grupo MCP cobre inspeção, erro atômico, prefixo/final de replay, save/load/undo/redo; suíte completa inclui render e replay.

Correção de fixture: o novo teste capturava prefixo e emitia a próxima ordem no mesmo tick, incluindo legitimamente essa ordem pendente no replay. Separado por um tick, mantendo igualdade integral do snapshot; runtime não alterado para acomodar a expectativa. Cenário focal e rodada final completa aprovados.

Demo examples/npc-dispel.scene.json salva pelo MCP e copiada do workspace isolado: veneno do vermelho protegido; veneno do jogador permanece removível pelo antídoto do azul/jogador. Tecla2 não remove o veneno protegido nem cobra quando só houver esse efeito. Interface visual não foi aberta/focada/fechada; teste físico/aceite manual pendentes.

Checklist de performance antes/depois em performance.md: um bool por definição compilada, sem estado/coleção extra por instância ou scan novo. Evidências nos logs estáveis artifacts/rust-tests.log, dispel-tests.log, clippy.log, build.log, verification-run.log e artifacts/mcp-validation/report.json (175 grupos), artifacts/dispel-verification.json/replays dentro do workspace isolado. Sem profiling/FPS alegado.

Build corrente target/release/ignis e artifacts/Ignis.app compartilham inode,9.709.648bytes; atualizado após lsof sem consumidores. Sem cópias históricas/push/publicação. EAF-46 permanece In Progress para aceite humano da demo. Esta extensão substitui a exclusão anterior de proteção booleana, sem implementar resistência probabilística, override por instância, inventário ou imunidade à aplicação.


## EAF-46 — dissipação transacional (24/10/2026)

Build15247d9b0f4d0f72, macOS Apple Silicon/Metal, cena/replay34. Classificação opcional por definição periódica (até64 categorias compiladas em u8), operador dispel com destinatário/limite/require_match. Máscara u64 no plano transacional impede duplicação entre operadores; compactação apenas no commit com custos/vida/recursos. Cast revalida agenda ao concluir; projétil usa vítima física/agenda atual no impacto sem nova cobrança ou reembolso. Pulsos devidos precedem dissipação, antídoto não desfaz dano anterior. Inspeção registra categoria/contagem e execução/ator responsável por cada remoção. Contrato/alternativas em dispel.md e architecture.md.

Validação final aprovada: `cargo fmt --all -- --check`, `cargo test --locked --release --all-targets` (**377 testes**), `cargo clippy --locked --release --all-targets -- -D warnings` e build release. Aviso preexistente de futura incompatibilidade de block0.1.6 permanece separado do resultado do Clippy.

42 cenários /174 grupos MCP passaram com native_window:false, incluindo4 grupos novos.20 testes Rust novos cobrem categoria/limite/ordem/refresh, ausência requerida/opcional, transação parcialmente inválida, máscara64, operadores repetidos, cast/expiração/cancelamento, concorrência, pulso devido, entrega espacial/interceptor/fonte morta, privacidade, NPC self-antídoto e validação de autoria. MCP cobre schema/nomes/recibos, erro atômico/undo/redo/reset, transação falha, projétil/custos, demo autorada/salva/reaberta; prefixos/finais e PNGs repetidos/replay idênticos. Captura dispel-demo.png inspecionada: azul preparando antídoto com faixa lilás, alvo roxo, atirador vermelho, jogador laranja, projétil e cobertura.

Checklist estático antes/depois em performance.md: nomes resolvidos só na compilação, busca limitada64 por operador/até8 operadores no plano, dados Copy/máscara e compactação in-place; nenhum Vec/string/JSON por remoção, callback/thread/dependência/pipeline novos. Planos de vida/saldos e retries/recibos limitados mantidos com motivo. Sem profiling ou ganho de FPS declarado. Evidências artifacts/rust-tests.log, dispel-tests.log, clippy.log, build.log, verification-run.log; artifacts/mcp-validation/report.json, logs/dispel.*, artifacts/dispel-verification.json, dispel*.png/replays.

Demo examples/npc-dispel.scene.json criada/salva pelo MCP e copiada do workspace isolado. Jogador1 veneno5/s×6(independent),2 antídoto próprio,3 antídoto no selecionado,4 regeneração5/s×6; cast0,5s/cooldown0,5s/custo5. Antídoto remove até2 poison, exige correspondência e preserva regeneration. Roxo passivo; azul self-antídoto cast1,5s/cooldown3s/retry0,5s; vermelho projétil de veneno4/s×4,cast1s/cooldown8s/6m/s. P/WASD/J/X/R e clique. Atalho4 adicionado sobre slot já suportado pelo MCP; teste de teclado/foco nativo e aceite manual separados da automação.

Build corrente target/release/ignis e artifacts/Ignis.app no mesmo inode,9.709.600bytes; substituído após lsof sem consumidores. Nenhuma janela aberta/fechada/focada durante regressão, nenhuma cópia histórica/push/publicação; trabalho paralelo no site preservado. EAF-46 In Progress aguardando aceite humano próprio; entusiasmo/continuidade após EAF-45 não é confirmação de teste manual. M08/M09 parciais: múltiplas tags, imunidades, triagem de aliados, modificadores, canalização, área, respawn e integração permanecem posteriores.


## EAF-45 — efeitos periódicos (24/10/2026)

Build7672c69e8420c753, macOS Apple Silicon/Metal, cena/replay33. Agenda opcional por definição, alvo direto ou impacto físico, pulsos após intervalo e custo único. Refresh por origem/definição/destinatário ou instâncias independentes; limite64, recibos256, ordem estável. Fonte morta/fora de visão não apaga efeitos instalados; alvo morto encerra sem ressurreição. Contrato/alternativas em periodic-effects.md e architecture.md. Nenhuma janela aberta, fechada ou focada durante implementação/verificação.

Validação final: `cargo fmt --all -- --check`, `cargo test --locked --release --all-targets` (**357 testes**), `cargo clippy --locked --release --all-targets -- -D warnings` e build aprovados. `python3 scripts/run_verifications.py`: **41 cenários / 170 grupos MCP**, todos headless. Aviso preexistente de futura incompatibilidade de block0.1.6 permanece separado do resultado de Clippy.

20 testes Rust novos: pulso inicial/final/custo/replay/prefixo, refresh/independent/origens, mesma data de pulso/reaplicação, morte fonte/destino/cast vencido, dano+cura transacionais, recursos/saturação, componente ausente/cancel/energia insuficiente, pool64/refresh em cheio, impacto/interceptor/pool sem reembolso, validação/formato, anéis/privacidade/guias/reset.5 grupos MCP: agenda/inspeção/custo, refresh/stack, projétil/parede, autoria inválida/edição/remoção/undo/redo, demo NPC/jogador; prefixo/final e PNG repetido/prefixo idênticos. Captura periodic-demo.png inspecionada: três atores, cobertura, barras e faixa lilás de agenda.

Correções de fixtures: interceptor sólido também bloqueava visão pré-lançamento; teste tornou-o transparente à visão para isolar contato físico. Demo verificava dano antes de cast+viagem+primeiro intervalo; passou a aguardar tick242. Regressão inicial parou em npc_autonomy por expectativa legada32; atualizadas também as expectativas de versão em mobility/targeting e repetida suíte completa. Nenhuma regra de runtime relaxada para acomodar essas expectativas.

Checklist de performance antes/depois em performance.md: instâncias Copy/Vec64, anéis256, sem timer/thread/JSON/heap por pulso, mesmos planos tipados de vida/saldos, máscara visual por ator. Mantidas cópias transacionais, varreduras limitadas e busca de refresh; nenhum profiling/ganho de FPS alegado. Evidências: artifacts/rust-tests.log, periodic-tests.log, clippy.log, build.log, verification-run.log; artifacts/mcp-validation/report.json, logs/periodic.*, artifacts/periodic-verification.json e periodic*.png/replays no workspace isolado.

Demo examples/npc-periodic.scene.json criada e salva via MCP, copiada do workspace isolado; exemplos antigos preservados. Jogador1 veneno direto (5/s×6,refresh),2 regeneração própria (5/s×6,independent),3 projétil de veneno; preparação0,5s/cooldown0,5s, custo5, projétil6m/s. Roxo passivo100hp; jogador60/100; vermelho veneno4/s×4,cast1s/cooldown8s, observação confirmada. P/WASD/J/X/R e clique. Primeiro pulso após1s da entrega; faixa lilás indica agenda, não tipo de efeito.

Build corrente target/release/ignis e artifacts/Ignis.app compartilham inode,9.658.736bytes; launcher atualizado após lsof sem consumidores. Nenhuma cópia histórica criada; alterações paralelas em website preservadas, sem push/publicação. EAF-45 In Progress para aceite humano próprio; continuidade após EAF-44 não equivale a confirmação de teste manual. M08/M09 seguem parciais: modificadores, dissipação, canalização, área, respawn e integração de NPCs continuam abertos.


## EAF-44 — mira espacial automática por observação (24/10/2026)

Build7cb31408a393c20f, macOS Apple Silicon/Metal, cena/replay32. Política opt-in use_ability.spatial_aim observed_point/range0,1..100 congela Aim::Point a partir de observação confirmada no início do cast; configuração está em action.policy.spatial_aim. Proveniência ator/tick/fonte acompanha cast/voo sem virar alvo rastreado. Perda de visão/troca/alcance/condição cancela; executor revalida visão/identidade/vida/relação/limiar antes de pagar. Onisciência explicitamente autorada permanece identificada; som/memória antigos não autorizam mira automática. Aproximação usa alcance efetivo XYZ da política. Ordens espaciais externas permanecem distintas. Contrato/tradeoffs em npc-aim.md e architecture.md.

337 testes Rust release aprovados (322 anteriores +15 novos), fmt --check, clippy --all-targets -D warnings e build release aprovados no fonte final. Novos casos: ponto/proveniência congelados, esquiva durante cast, alvo parado atingido depois do lançamento/custo único, perda de visão/alcance/troca de candidato, aproximação por alcance próprio, altura/downward shot, som anônimo/posição oculta, fonte onisciente rotulada e parede física, condições vitais/relação/cura durante cast, morte/cancel, ordem externa sem proveniência, versões/configuração atômica, guia sem efeito de gameplay e reset. Aviso preexistente block0.1.6 permanece.

40 roteiros MCP/165 grupos passaram no build final, todos headless. Quatro grupos novos cobrem autoria/schema, congelamento/proveniência, custos/efeitos, cancel por cobertura, aproximação, erro atômico/undo/redo/reset, guia opt-in e demo comparativa. Replay final e prefixo reproduzem estado; capturas repetidas/prefixo idênticas e toggle de guia altera apenas apresentação. Captura GPU inspecionada: ponto amarelo fica separado do jogador após movimento. Nenhuma janela aberta/focada/fechada; teclado/foco/aceite humano específicos pendentes.

Durante construção, dois asserts de verificação foram corrigidos: comparação decimal exata substituída por tolerância1e-6 na coordenada f32 serializada; out_of_range observado no tick do cancelamento, antes do retry que corretamente passa para unavailable. Os testes focados seguintes e a regressão final passaram; não houve alteração do runtime para mascarar essas expectativas.

Revisão estática antes/depois em performance.md: observação existente, dados Copy fixos, alcance ao quadrado, sem scan/raycast/alocação de entidade por intenção; até3 caixas por cast no frame/cache atual. Plano transacional, mapas e consulta de política autorada permanecem, sem profiling/FPS alegado. Evidências: artifacts/rust-tests.log, clippy.log, build.log, verification-run.log e artifacts/mcp-validation/report.json, logs/npc_aim.*, artifacts/npc-aim-verification.json, npc-aim*.png e replays dentro do workspace isolado.

Demo examples/npc-aim.scene.json autorada/salva via MCP; único exemplo novo copiado, anteriores preservados. Vermelho fixa ponto ao iniciar; azul mira no ator até lançar; ambos cast1,5s/dano10/6m/s/alcance18m/custo5/cooldown3s. P inicia/WASD move/J marcador e barras/R reset; jogador1 tiro livre,2 cura25,3 tiro por alvo/X cancela. Parede verde bloqueia visão/disparo. App corrente artifacts/Ignis.app e target/release/ignis no mesmo inode,9.573.824bytes, substituído após lsof sem consumidores. Sem builds históricos/push/publicação.

EAF-44 In Progress para aceite humano próprio. EAF-43 recebeu autorização de continuidade, sem inferir confirmação de teste manual. M08/M09 parciais; antecipação, mira por pistas anônimas, esquiva/cobertura táticas, triagem, canalização, efeitos persistentes e respawn permanecem posteriores.

## EAF-43 — mira livre por direção/ponto (24/10/2026)

Build038162bde325e24e, macOS Apple Silicon/Metal, cena/replay31. Selection spatial opt-in com projectile/range0; `aim_ability` recebe direção/ponto XYZ e `aim_ability_at` resolve cursor em plano explícito no dispatcher. Log guarda resultado mundial; UI usa o mesmo comando. Origem atual ao lançar, mira congelada no pedido; ponto não encerra voo. Custos/condições da fonte no launch, transação de efeitos na primeira vítima física; alvo inexistente não impede disparo. NPC pode receber ordem espacial sem consultar ator oculto; objetivo automático espacial rejeitado até contrato de evidência. EAF-42 recebeu aceite humano e foi encerrado. Contrato/tradeoffs em free-aim.md e architecture.md.

322 testes Rust release aprovados (305 anteriores +17 novos), fmt --check, clippy --all-targets -D warnings e build release aprovados. Novos casos: disparo sem seleção, direção normalizada, ponto ultrapassado, mira vertical/expiração/fonte sem vida, movimento da origem, ponto coincidente no início e no deadline sem custo/cooldown, interrupt/cancel/reset, componentes ausentes no impacto, parede invisível, NPC sem pista/posição oculta alterada, NPC desabilitado/fundos insuficientes, versões/modos/objetivos incompatíveis, viewport/raio paralelo/plano atrás, erros atômicos e replay pendente/prefixo/final. Aviso preexistente block0.1.6 permanece. Nenhuma falha nos testes focados ou na regressão final.

39 roteiros MCP/161 grupos aprovados no build final, todos headless. Os cinco grupos novos cobrem autoria/schema/entrada inválida, efeitos/custos, tiro vertical/vazio, ponto como referência, primeiro bloqueio, undo/redo/reset, mira do cursor e demo; capturas repetidas e de replay de prefixo idênticas. Inspeção visual offscreen confirma projétil em voo e barras da vítima após impacto. Nenhuma janela aberta, focada, fechada ou controlada; teclado/foco/aceite humano da nova demo continuam separados.

Revisão estática antes/depois em performance.md: payload numérico Copy, sem corpo/entidade/heap por mira, projeção só em comando, alvo opcional real, buffers e sweep existentes. Mantidos normalização limitada ao início/commit/lançamento, cópia do plano fixo e snapshot de corpos; sem medição/FPS alegado. Testes nativos de regressão de picking/render preservados após extração do raio compartilhado.

Evidências correntes: artifacts/rust-tests.log, clippy.log, build.log, verification-run.log; artifacts/mcp-validation/report.json, logs/free_aim.*, artifacts/free-aim-verification.json, free-aim*.replay.json e free-aim*.png dentro do workspace isolado. Apenas examples/free-aim.scene.json foi copiada da autoria MCP; exemplos anteriores preservados. App corrente artifacts/Ignis.app e target/release/ignis compartilham inode,9.556.912bytes, substituição após lsof sem consumidores. Sem builds históricos, push/publicação ou modificação da demo em execução.

Demo: P inicia, WASD move,1 mira no cursor sem seleção (plano no centro do jogador, cast0,5s/dano20/6m/s/18m/custo5/cooldown0,5s),2 cura própria bloqueando locomoção,3 projétil por alvo selecionado. Roxo passivo60/100, vermelho atira por percepção, parede verde bloqueia. Cursor/posição mundial congelados, sem retículo contínuo/mira FPS/tática automática nova. EAF-43 permanece In Progress para aceite humano próprio; M08/M09 parciais.

## EAF-42 — lançamento e impacto por projétil (24/10/2026)

Build91ecdb5543ac4098, macOS Apple Silicon/Metal, cena/replay30. Delivery opt-in direct/projectile; direção XYZ congelada, sweep AABB contra snapshot dos corpos, primeiro contato e desempate por ID. Lançamento valida capacidade/componentes, cobra custo/cooldown; impacto aplica plano transacional atual sem cobrar novamente. Emissor morto/cancelamento tardio não recolhem disparos; aliado/cadáver pode bloquear. Efeitos de alvo vida/recurso suportados, sem impulso/homing/gravidade/área/CCD relativo. Contrato/tradeoffs em projectiles.md. EAF-41 recebeu aceite humano e foi encerrado.

305 testes Rust release aprovados (285 anteriores +20 novos), fmt --check, clippy --all-targets -D warnings e build release aprovados no fonte final. Cobertura nova: atraso/custo único/prefixo, legado direto, cancel antes/depois, parede5cm invisível/velocidade100m/s, oclusão visual, NPC com alvo oculto/desvio, direção XYZ/hover, alcance/expiração, morte do emissor/vítima e cancel de cast, interceptação/elegibilidade, cadáver sem ressurreição, cura com vida atual, efeitos atômicos com componente ausente, capacidade64 e disputa pelo último slot, empate por ID/ordem da cena, configuração inválida/aim coincidente/reset. Aviso preexistente block0.1.6 permanece.

38 roteiros MCP/156 grupos passaram no build final, todos headless, sem falha/retry na suíte completa. Roteiro projectiles: autoria/schema/inspeção, lançamento pago antes do dano, impacto sem cobrança, bloqueio/desvio pelo jogador, erro atômico, undo/redo/direct/reset, fila pendente e replay final/prefixo com capturas repetidas/prefixo idênticas. Demo confirma dois disparos ativos simultâneos (jogador/NPC), dano real depois da viagem, gravação/reabertura da cena. Captura GPU offscreen inspecionada; nenhuma janela aberta/focada/fechada. Teclado/aceite humano próprio permanecem separados.

Revisão estática antes/depois em performance.md: buffers limitados/pré-alocados, compactação estável em uma passagem, custos por referência, sem corpo físico ou heap por disparo. Mantidos conscientemente varredura linear64×512, snapshot dinâmico por tick e plano transacional de validação também no lançamento; sem benchmark/FPS alegado. Ajustes durante construção: borrow da origem resolvido antes de mutar Shot; fixtures corrigidas para espessura mínima5cm e conectores aéreos explícitos, sem relaxar contratos.

Evidências correntes: artifacts/rust-tests.log, clippy.log, build.log, verification-run.log; artifacts/mcp-validation/report.json, logs/projectiles.* e artifacts/projectiles-verification.json, projectile*.replay.json e projectile*.png dentro do workspace isolado. Apenas examples/npc-projectiles.scene.json copiada da autoria MCP; exemplos anteriores preservados.

Binário corrente target/release/ignis9.472.208bytes; artifacts/Ignis.app atualizado por hardlink após lsof sem consumidores, mesmo inode. Sem histórico de builds/instaladores, push ou publicação. EAF-42 permanece In Progress para aceite humano da demo; M08/M09 parciais. Esquiva/tática autônoma de NPC, triagem, canalização, efeitos persistentes, área, balística/homing e respawn continuam posteriores.


## EAF-41 — movimento durante preparação de habilidades (24/10/2026)

Build243cfafaf4f71747, macOS Apple Silicon/Metal, cena/replay29. Três políticas por definição autorizadas: free, interrupt com tolerância física XYZ, lock de locomoção voluntária. Mesmo executor jogador/NPC, controles MCP existentes, amostras/recibos/bloqueio inspecionáveis. Cancelamento por entrada precede commit; motor e solver verificam movimento nas fases seguintes. Custos/efeitos/cooldown somente no commit; lock derivado libera sem apagar input. Patrulha suspende, reservas são liberadas, caminhada retoma; travessia aérea comprometida recusa lock sem congelar. Contrato/tradeoffs em cast-movement.md. EAF-40 recebeu aceite humano e foi encerrado.

285 testes Rust release aprovados (270 anteriores +15 novos), fmt --check, clippy --all-targets -D warnings e build release aprovados no fonte final. Casos novos: legado free; rejeição simultânea input/cast, cancelamento no prazo, jitter, custo/efeito/cooldown, input segurado/retomada, falha/cancel/morte/reset, queda/empurrão, NPC automático, patrulha terrestre/espacial suspensa além do prazo, passagem e voo incompatível, versão/tolerância/campos desconhecidos, replay/prefixo. Aviso preexistente block0.1.6 de incompatibilidade futura permanece.

37 roteiros MCP/152 grupos passaram no build final, todos sem janela, sem falha/retry na suíte completa. Novo cast_movement cria/salva a demo pelo dispatcher, valida os três modos, deadline, custos/vida, schema, erro atômico, undo/redo/load/reset, pendências, replay final/prefixo e capturas GPU repetidas/prefixo idênticas. Captura offscreen inspecionada: jogador laranja com barras de vida/energia/cast, alvo roxo selecionado. Sem interação com a janela do usuário; teclado/foreground e aceite humano da demo permanecem gates separados.

Ajustes durante validação: serde aceitava campo extra em variante unit; variantes vazias de struct agora rejeitam esse erro. Fixture inicial de empurrão usava NPC, que desviava corretamente do jogador; substituída por queda de corpo dinâmico fora do centro, provocando deslocamento real pelo solver. Lint identificou unwrap redundante após is_some; corrigido com if let e repetida validação no fonte final. Revisão de performance antes/depois em performance.md; sem profiling/FPS alegado.

Evidências correntes: artifacts/rust-tests.log, clippy.log, build.log, verification-run.log; artifacts/mcp-validation/report.json, logs/cast_movement.* e artifacts/cast-movement-verification.json, cast-*.replay.json e cast-*.png dentro do workspace isolado. Apenas examples/cast-movement.scene.json copiada da autoria MCP; demos anteriores preservadas.

Binário corrente target/release/ignis9.403.952bytes; artifacts/Ignis.app atualizado por hardlink após lsof confirmar ausência de consumidores, mesmo inode. Sem cópias históricas/instaladores, janelas manipuladas, push ou publicação. EAF-41 permanece In Progress aguardando aceite humano da demo; M08/M09 parciais. Canalização, efeitos persistentes, entrega física/área, animação e respawn continuam fora deste incremento.


## EAF-40 — decisão vital e aproximação de habilidades (24/10/2026)

Build7ed818f6ab1f86b1, macOS Apple Silicon/Metal, cena/replay28. Política opt-in por objetivo com limiar vital em pontos-base, custo/relação/visão/alcance, aproximação com margem e diagnóstico por candidato. Vida de alvo só após confirmação visual corrente; memória/som/rádio não autorizam consulta vital. Executor transacional e motores existentes preservados. Contrato/tradeoffs em npc-ability-policy.md; EAF-39 recebeu aceite humano e foi encerrado.

270 testes Rust release aprovados (255 anteriores +15 de política), fmt --check, clippy --all-targets -D warnings e build release. Cobertura nova: limiar exato e máximo10^12, histerese/target_changed, vida própria sem alvo, ausência de componente/custo, dois objetivos do mesmo grant, cast mantido/cancel por cura ou perda visual, legado/retry, obstáculo com desvio físico, equivalência sob vida/posição oculta, território/relação, alvo elevado inalcançável por terrestre e rota aérea válida, versão/validação e replay. Aviso preexistente de incompatibilidade futura em block0.1.6 não é falha de lint.

36 roteiros MCP/149 grupos passaram no build final, todos headless. Novo roteiro ability_policy: autoria pelo dispatcher/schema, aproximação sem recibos rejeitados, cura única para70%, espera no limiar, custo exato, erro atômico, undo/redo/reset/load, replay final/prefixo e capturas GPU repetidas/prefixo iguais. Demo confirmou atacante móvel, curador condicionado e autocura somente após segundo dano. Captura final inspecionada visualmente; câmera aproximada para tornar atores/barras legíveis.

Ajustes durante validação: teste próprio precisava aguardar retry antes de inspecionar novo motivo; fixture aérea corrigida para limite8m de conexões locais. A plataforma bloqueava o segmento direto, corretamente diagnosticado como no_local_route; o teste positivo recebeu rede autorada por cima e passou, sem inventar desvio aéreo automático. Teste negativo terrestre permaneceu. Revisão estática removeu cópia de256bytes de saldos por oferta e segunda varredura de custos ao consultar alcance; máscara por objetivo preserva condições diferentes do mesmo grant. Nenhum profiling ou ganho de FPS declarado.

Evidências correntes: artifacts/rust-tests.log, clippy.log, build.log, verification-run.log; artifacts/mcp-validation/report.json e logs/ability_policy.*; artifacts/ability-policy-verification.json, policy.replay.json, policy-demo.replay.json e PNGs policy-* dentro do workspace isolado. Apenas examples/npc-ability-policy.scene.json copiada da autoria MCP; exemplos anteriores preservados.

Binário corrente target/release/ignis9.352.736bytes; artifacts/Ignis.app atualizado por hardlink após lsof sem consumidores, mesmo inode. Sem cópias históricas/instaladores novos. Nenhuma janela aberta, fechada ou focada. EAF-40 permanece In Progress aguardando aceite humano da nova demo/atalhos físicos; M08/M09 continuam parciais. Sem push/publicação. Triagem, posicionamento tático, projéteis/área, efeitos persistentes e respawn seguem contratos posteriores.


## EAF-39 — vitalidade, dano, cura e morte (24/10/2026)

Build6fe89b0573805fbf, macOS Apple Silicon/Metal, cena/replay27. Contrato discutido e aprovado antes da implementação: componente vital independente, unidades inteiras/escala, Damage/Heal tipados, atomicidade com custos e morte ao fim da execução. Ordem sequencial por fase/ID preservada; interrupção de ações/motores/reservas e corpo físico mantido. Nenhuma dependência nova.

255 testes Rust release aprovados (234 anteriores +21:15 vitalidade,5 percepção e1 passagem), fmt --check e clippy --all-targets -D warnings. Cobertura: limites/escala, vida cheia/ausente/morta, recibos/excesso, dano letal seguido de cura, causa final, rollback após efeito inválido, revalidação durante cast, ordem entre fases/atores, cancelamento por morte, ausência de novas intenções/ruído de passos, entrada do jogador morto, queda/inércia aérea, liberação de reserva e cadáver obstruindo a passagem. Percepção cobre morte oculta, confirmação visual/omnisciente, alternativa hostil viva, rádio antigo e origem do dano sem conhecimento gratuito. Reset, erro atômico, prefixo/pendências/final e tampering de expected_vitality também verificados.

35 roteiros MCP/146 grupos aprovados no build final, sem janela. Novo vitality autorou catálogo/entidades, aplicou dano/cura/morte, verificou recibos e rejeição sem débito, histórico/reset/undo/redo, replay e capturas repetidas/prefixo idênticas. Demo NPC/jogador confirmou vida100→80 por dano20 e80→92 por cura12. Capturas vitality-demo.png e vitality-dead.png inspecionadas visualmente: barras/seleção, alvo passivo sem grants e cadáver cinza com barra vazia. Nenhum ganho de FPS alegado; checklist antes/depois em performance.md.

Ajustes de fixtures durante validação: SetInput exige struct input; voo em segmentos locais usa phase Traversing e não o campo committed reservado a outros compromissos; política omnisciente não permite rádio; occupied da passagem exclui o próprio dono vivo, mas inclui seu corpo após perder a reserva. As verificações foram corrigidas para medir esses contratos reais; não se atribuiu falha a foco/teclado nem se alterou o runtime para acomodar as fixtures.

Evidências correntes em artifacts/rust-tests.log, clippy.log, build.log, verification-run.log e artifacts/mcp-validation/report.json. Detalhes em artifacts/mcp-validation/artifacts/vitality-verification.json, replays/PNGs vitality-* e logs/vitality.*. Apenas examples/npc-vitality.scene.json copiada da autoria MCP; exemplos anteriores preservados. Binário target/release/ignis9.251.824bytes; artifacts/Ignis.app renovado por hardlink após lsof sem consumidores, mesmo inode. Sem instaladores ou versões históricas novas. Nenhuma janela aberta, fechada ou focada.

EAF-38 recebeu aceite humano e foi encerrado. EAF-39 recebeu aceite humano explícito após a entrega (“Funcionou perfeitamente”) e foi encerrado; esta confirmação não amplia o escopo para ressurreição/respawn. Contrato/guia em docs/vitality.md. M08/M09 continuam parciais; ressurreição/respawn, política de cura/aproximação, entregas físicas e persistência seguem abertos. Sem push/publicação.

## EAF-38 — habilidades diretas compartilhadas (24/10/2026)

Build0e549c0be6000916, macOS Apple Silicon/Metal, cena/replay26. Recorte A autorizado: catálogo compilado, recursos inteiros/escala, cast único por ator, custos/cooldown no commit, cancelamento por execução, efeitos diretos de recurso/ruído e objetivo use_ability compartilhado com jogador. Percepção confirmada limita alvos de NPC; memória/rádio não fornecem localização oculta para a habilidade. Nenhuma dependência nova.

234 testes Rust release aprovados (215 anteriores +19 de habilidades), cargo fmt --check e clippy --all-targets -D warnings. Cobertura inclui custos múltiplos/agregados, insuficiência/limites, clamp/recibo real, mudança de saldo durante cast, cancel no prazo, perda de alcance/visão, pool ausente/oclusão, retry/interrupção/retomada, rádio/privacidade, continuidade de voo, ordem lexical, saturação de ruído, cast futuro sem reservar o orçamento atual, entradas pendentes/prefixo, histórico e replay. Aviso preexistente de incompatibilidade futura em block0.1.6 permanece separado do lint.

34 roteiros MCP/143 grupos aprovados no build final, todos sem janela. O novo roteiro abilities cobre autoria em lote, player/conversor/apoio com executor comum, recursos/casts, cancel no vencimento, sinal percebido no passo seguinte, erro atômico, undo/redo/load/reset, condição própria, replay pendente/final/prefixo e pixels. Capturas repetidas e prefixo são idênticos; imagem após commit difere. Captura de cast inspecionada visualmente, com barras e atores legíveis; câmera aproximada e sensores ocultos para reduzir sobreposição, disponíveis por V/H.

Falhas intermediárias: fixture usava ally em vez do enum allied; corrigida. Teste de voo inicialmente amostrava a espera no ponto inicial; passou a exigir Traversing antes de iniciar a habilidade e confirmou movimento durante cast. Revisão encontrou orçamento de ruído do tick atual recusando cast futuro; corrigido para conferir capacidade no commit, com teste dedicado. Cast instantâneo reutiliza plano validado em vez de repetir consultas. Revisão completa em performance.md; sem profiling, FPS ou escala além dos limites declarados.

Evidências correntes: artifacts/rust-tests.log, clippy.log, verification-run.log; artifacts/mcp-validation/report.json, logs/abilities.*, artifacts/abilities-verification.json, abilities.replay.json, abilities-pending.replay.json, abilities-signal.replay.json e PNGs de cast/commit/sinal/prefixo dentro do workspace isolado. Somente examples/npc-abilities.scene.json foi copiada da autoria MCP; cenas antigas preservadas. Binário corrente target/release/ignis9.133.120bytes, bundle artifacts/Ignis.app renovado por hardlink após lsof confirmar ausência de uso. Mesmo inode; nenhum instalador/histórico de binários novo, push ou publicação.

EAF-38 permanece In Progress aguardando aceite humano da demo/atalhos físicos. Nenhuma janela aberta, fechada ou focada; MCP/offscreen não comprova integração de teclado. M08/M09 continuam parciais. Próximo contrato: vida/morte e dano/cura; entregas físicas, persistência, scripts e ABI permanecem posteriores.

## EAF-37 — estudo de habilidades (24/10/2026)

Unidade documental sobre a base Ignis a65f68a; nenhum fonte Rust, binário, formato cena/replay25, exemplo jogável ou launcher foi alterado. Sem nova suíte Rust/MCP ou execução Unreal, pois não houve alteração de comportamento. Os 215 testes/140 grupos anteriores continuam evidência de EAF-36, não validação de habilidades implementadas.

Referência LandOfChaos7b978932c0428b9b8f3691f5edb73226e49fac89:16 arquivos de programa, execução, entrada, seleção, entrega, ações, recibos, atributos, adaptação, exemplos e documentação identificados por hash em ability-reference-inventory.json. Conteúdo consultado correspondia ao commit; mudanças paralelas de arte ficaram intactas. Revisão percorreu Start/Execute/Finish, reserva/cobrança, cancelamento/filhos, seleção/contato e retorno de magnitude aplicada. Relatório histórico de294 programas declara auditoria de autoria, sem gameplay; não foi reproduzido nem tratado como teste de runtime.

Checagens desta unidade: inventário contra objetos Git da revisão fixada, hashes/caminhos, números/escopo do relatório de referência, links locais do estudo e diff sem whitespace inválido. Checklist proporcional registrado em performance.md. Resultado em ability-runtime-study.md com alternativas, proposta numérica/atomicidade/concorrência, ordem de ticks, privacidade, fronteira MCP/replay e matriz de aceitação. São propostas para revisão técnica, sem ABI ou contrato de combate silenciosamente adotado.

EAF-36 recebeu aceite humano explícito e foi encerrado no Kordo. EAF-37 conclui estudo/proposta, não implementação/aprovação das escolhas. M08/M09 permanecem parciais; próximo incremento recomendado é execução direta de habilidades não combativas, depois de alinhados os contratos indicados.

## EAF-36 — comunicação de pistas por rádio (24/10/2026)

Build d95fab188b7c35ef, macOS Apple Silicon/Metal, cena/replay25. Rádio opcional compartilha visão confirmada anterior, com identidade/posição/instante originais; canal/alcanceXYZ/cooldown/idade/send/receive e alianças mútuas. Sem retransmissão, onisciência, áudio ou leitura da posição oculta. Memória report distinta de visão/som; prioridade direta, território/identidade/hostilidade, reconhecimento e objetivos preservados.

215 testes Rust release aprovados (207 existentes +8 novos), fmt --check, clippy --all-targets -D warnings e build. Acréscimo de assert explícito de ausência de reconhecimento também passou no teste dedicado. Novos testes cobrem movimento oculto, ordem/empate lexical, canais/raioXYZ/relações, TTL original, ausência de cascata, prioridade de visão/som/memória direta, reconhecimento, território/desativação/hostilidade, atomicidade/legado/reset e replay. Aviso preexistente de incompatibilidade futura em block0.1.6 permanece.

33 roteiros MCP /140 grupos aprovados, todos sem janela, no build final e sem retry na suíte completa. Novo roteiro communication autorou demo, validou confirmação visual antes do envio, investigação física atrás da parede e canal distinto, alianças direcionais, erro atômico, disable/undo/redo/remove/load e replay completo/prefixo/PNG idêntico. Captura offscreen inspecionada. Erros durante construção das fixtures Rust foram corrigidos: envelope/tipos do comando, campo noise_id e dois emissores inicialmente sobrepostos que bloqueavam a visão um do outro. Nenhuma falha de runtime foi mascarada por mudar expectativa.

Revisão estática antes/depois em performance.md: arrays fixos, valores Copy, catálogo existente, distâncias ao quadrado, nenhum heap/JSON por mensagem; custo O(N²) limitado a8 NPCs. Sem profiling/FPS ou escala além do limite atual alegados. Documentação descreve um tick de latência, política conservadora de memória direta, recibo retido só para diagnóstico e rádio através de paredes.

Evidências correntes: artifacts/rust-tests.log, radio-rust-tests.log, clippy.log; artifacts/mcp-validation/report.json, logs/communication.*, artifacts/radio-verification.json, radio.replay.json, radio.png e radio-prefix.png dentro do workspace de validação. Demo examples/npc-radio.scene.json copiada da autoria MCP isolada; demais exemplos preservados. Binário corrente target/release/ignis (8.778.608 bytes); launcher artifacts/Ignis.app renovado por hardlink após lsof confirmar ausência de uso do binário anterior. Nenhuma janela aberta, fechada, focada ou controlada; nenhum instalador/cópia histórica novo, push ou publicação.

EAF-36 permanece In Progress para aceite humano próprio. M09 e aceites anteriores continuam abertos. Integração de rampas/controlador fica para retomada posterior; próximo contrato proposto é ações/habilidades compartilhadas, precedido de estudo do runtime do LandOfChaos e discussão técnica.

## EAF-35 — experimento de controlador (24/10/2026)

Estudo Rust separado em examples/locomotion_probe.rs, Rapier0.30.1/macOS Apple Silicon. Sem alteração de src, Cargo.lock, formato cena/replay24 ou build da engine bf1cde1c22341efe.11cenários×3motores, repetidos (66execuções de240ticks+setup), com assertions de resultados físicos, penetração limitada e igualdade de trajetória/observações no mesmo build. Configuração, hash próprio do fonte e métricas em artifacts/locomotion-probe/report.json. Matriz e proposta em docs/locomotion-controller-study.md.

207 testes Rust release aprovados (206existentes+1experimento), fmt e clippy --all-targets -D warnings aprovados. Aviso preexistente block0.1.6 permanece.14grupos MCP headless em lab/walk_support passaram; esse é um recorte, não repetição da suíte completa32roteiros. O experimento compara a política física isolada e não usa o planejamento/sensores/replay da engine. Nenhum controle de gameplay foi acrescentado fora do MCP.

Resultados: ambos sobem20°; dinâmico sobe40° enquanto cinemático configurado em30° recusa. Dinâmico para em20cm, cinemático supera e recusa50cm/teto baixo. Parede bloqueia; borda1m causa queda/aterrissagem. Cinemático simples não empurra caixa; resolver impulsos explicitamente a desloca3,087m, contra5,541m no dinâmico. Impulso externo no ator é ignorado pelo cinemático e dura um passo no motor dinâmico atual, que sobrescreve XZ. Sem claim de controlador dinâmico completo, zero alocação, benchmark/FPS, navmesh ou rampas integradas.

Checklist estático antes/depois concluído, incluindo Vec de manifolds no grounded/impulsos da biblioteca e custo de autostep. Relatório/logs em artifacts/locomotion-probe, relatório MCP corrente em artifacts/mcp-validation/report.json. Uma localização estável para o executável de estudo; runtime/bundle existente preservados, nenhuma janela controlada. Decisão de produção permanece proposta; aceites humanos anteriores/M09 não encerrados pelo experimento.

## EAF-34 — apoio contínuo entre pisos modulares (23/10/2026)

Buildbf1cde1c22341efe, macOS Apple Silicon/Metal. Sem mudança do formato cena/replay24. Links/conectores walk e apoio pontual usam walk_support: fast path de caixa única e cobertura composta da área varrida por até64 superfícies fixas. Partição analítica em faixasX/uniãoZ evita amostras que atravessem buracos; largura/depth do corpo, tolerâncias existentes e clearance físico separados. Retângulos não são expandidos para unir gaps artificialmente. Diagnósticos por link: walk_support_gap, walk_height_mismatch, walk_support_limit.

206 testes Rust release passaram (6novos), fmt --check, clippy --all-targets -D warnings e build aprovados. Primeira compilação do teste falhou por token '<-' em comparação negativa; corrigido para '< -', sem alteração de runtime. Testes cobrem ordem/eixos, apoio sobre emenda, diagonal sem exigir AABB inteira, buraco1cm fora das linhas centro/cantos, patch, largura/altura/gap/superfície visual/dinâmica, limite65 e fast path, movimento físico em links e conectores, replay/prefixo e remoção/undo. Warning preexistente block0.1.6 permanece. Revisão estática antes/depois em performance.md; nenhum ganho de FPS medido.

32 roteiros MCP/137 grupos passaram, todos sem janela e no build final. Novo roteiro walk_support: demo parametrizada de três passagens, patrulha de ida/volta, rejeições explicáveis, remover ponte/undo/redo/load, links explícitos e conectores locais, som/interrupção/retomada e replay/prefixo/PNG idêntico. Primeiro teste MCP tinha expectativa incorreta: uma rota bloqueada [destino,spawn] pode contar legitimamente a visita ao spawn. Corrigida fixture dos NPCs bloqueados para um único destino remoto e assert de remoção da ponte para posição/skipped; sem relaxar expectativa sobre atravessar vãos. Suíte completa final sem falha/retry.

Evidências: artifacts/rust-tests.log, clippy.log, build.log; artifacts/mcp-validation/report.json e logs. Dentro desse workspace, artifacts/walk-surfaces-verification.json, walk-surfaces.replay.json, walk-surfaces-interruption.replay.json, walk-surfaces.png e walk-surfaces-prefix.png. Captura inspecionada: azul no centro da passagem contínua, roxo antes dos vãos, cinza antes da ponte estreita, laranja no piso inferior. Demo examples/npc-walk-surfaces.scene.json criada/salva pelo MCP e copiada ao projeto sem regravar fixtures antigas. P inicia, R reinicia, J guias; N não modifica estes NPCs na demo manual, que isola apoio. O cenário automatizado habilita audição separadamente para verificar integração.

Executável corrente target/release/ignis8.727.504bytes. Bundle antigo artifacts/Ignis.app em uso pela sessão anterior preservado (inode8396016,8.659.280bytes); nenhuma janela aberta/focada/reiniciada. Não substituir binário ativo: próximo empacotamento após encerrar a sessão atualiza hardlink pelo script existente. Sem instalação, push ou publicação. EAF-34 permanece aguardando aceite humano próprio; relatos da demo antiga não validam esta cena.

Rampas/degraus/navmesh e superfícies móveis continuam fora do recorte. architecture.md registra alternativas para geometria inclinada e controlador, com impacto em render/colisão/picking/oclusão/navegação; nenhuma foi escolhida implicitamente. M09 continua parcial.


## EAF-33 — patrulhas espaciais e retomada (23/10/2026)

Build4e41b5bda662d797, macOS Apple Silicon/Metal, cena/replay24. Rotas points_3d/ring_3d com até16 centrosXYZ, normal/raio/semente, geração somente em edição/carga. Mesma FSM de patrulha e motores físicos; nenhum controle exclusivamente visual. Loop/ping-pong, espera, prazos/cooldown, percepção e seletor de objetivos preservam índice/compromisso aéreo. Aproximação parcial permanece falha, sem visita falsa; voador sem hover espera orbitando.

200 testes Rust release passaram (7novos), fmt --check, clippy --all-targets -D warnings e build aprovados. A primeira execução dos novos testes encontrou a antiga proibição de patrol+mobility e uma fixture de salto12m/s acima do limite10; a combinação foi habilitada explicitamente na versão24 e a fixture corrigida sem relaxar limites. A execução seguinte revelou falso unreachable ao aterrissar: contato ainda penetrado encerrava o link cedo. Confirmação agora exige apoio, baixa velocidade vertical e clearance até endpoint, consultado somente perto dele. Sem teleporte/tolerância ampliada. Regressão completa passou após correção e novamente após limitar essa consulta/estender teste de objetivos. Warning preexistente block0.1.6 permanece.

31 roteiros MCP/134 grupos passaram sem janela, todos no build final, sem retries/falhas nessa suíte. Novo roteiro spatial_patrol valida quatro capacidades, pontos em alturas distintas, proceduralidade/seed, atomicidade/histórico/persistência, ruído anônimo/interrupção/retomada, reset, replay final/prefixo e pixels GPU idênticos. Testes Rust também verificam ping-pong, espera em órbita, salto repetido/descida sem subida, velocidade sem teleporte, feedback antigo/altura incorreta, timeout e tolerância de conversão centro/pés. Checklist de performance antes/depois em performance.md; nenhum ganho de FPS medido.

Evidências: artifacts/rust-tests.log, clippy.log, build.log e artifacts/mcp-validation/report.json (31cenários, native_window:false). Dentro do workspace isolado: artifacts/spatial-patrol-verification.json, spatial-patrol.replay.json, spatial-patrol-interruption.replay.json, spatial-patrol.png e spatial-patrol-prefix.png; logs por cenário. Captura inspecionada: azul na rota aérea inclinada, roxo no arco de salto, verde descendo, cinza parado e jogador laranja. Demo examples/npc-spatial-patrol.scene.json autorada/salva pelo MCP, copiada ao projeto sem alterar fixtures antigas. Guia lilás de autoria difere de rota física planejada; destino impossível permanece diagnóstico visível.

Executável corrente target/release/ignis8.709.776bytes. Bundle artifacts/Ignis.app continua na sessão ativa PID20015, build anterior cbb2c441cafb27eb, inode8396016/8.659.280bytes; preservado sem reabrir/ativar/reiniciar/substituir. Essa cópia em uso é exceção temporária ao build único, não histórico novo. Próximo empacotamento após encerramento deve atualizar hardlink com scripts/package_macos.py. Sem instalação, publicação ou push. Demo nova disponível com o executável corrente; aceite humano EAF-33 ainda pendente.

Próximo recorte: superfícies caminháveis/rampas/degraus, com discussão de geometria/colisão/controlador antes de implementação. Território/passagens/recuperação espaciais, habilidades/ciclo de vida, comunicação/interação/população e apresentação continuam abertos. Esta entrega não conclui NPCv1/M09.


## EAF-32 — orientação sensorial e visão volumétrica (23/10/2026)

Build4f42a4c6ad10bb58, macOS Apple Silicon/Metal, cena/replay23. Opt-in npc.spatial_vision: pitch inicial[-90,90], orientação acompanhando velocidade comandada ou independente. FOV circularXYZ reutiliza abertura/alcance existentes, mantém oclusão e conhecimento limitado. Cenas antigas preservam caminho horizontal. inspect_npcs/inspect expõem pitch, acompanhamento e direçãoXYZ; guias V/set_debug_view usam volume esférico nominal e seta na origem do sensor. Sem rotação de corpo/mesh ou varredura vertical automática.

193 testes Rust release passaram (7novos:6 de integração sensorial e1 de geometria/cache), fmt --check, clippy -D warnings e build passaram. A primeira compilação dos novos testes tentou comparar NavGoal sem PartialEq; corrigida somente a comparação dos seis campos do objetivo no teste, sem alterar o runtime. Warning antigo block0.1.6 permanece. Checklist estático antes/depois em performance.md. Validação cobre cone acima/abaixo, borda angular, alcance, teto/blocks_vision, onisciência explícita, sensor legado, alvo oculto em altitudes diferentes, acompanhamentoXYZ/parada/independência, voo físico/replay/prefixo, validação atômica e histórico/reset.

30 roteiros MCP/130 grupos passaram sem janela, sem repetição automática/falha nesta regressão. Captura do estado no tick1 igual ao replay desse prefixo; save/load, pitch inválido, undo/redo, oclusor visual independente e toggle de guias aprovados. Evidências em artifacts/mcp-validation/report.json, logs por cenário, artifacts/spatial-vision-verification.json, spatial-vision.replay.json, spatial-vision.png e spatial-vision-prefix.png dentro desse workspace. A execução bem-sucedida não demonstra correção de intermitências históricas.

Demo examples/npc-spatial-vision.scene.json criada pelo MCP no workspace isolado e adicionada ao projeto, sem regravar exemplos antigos. Captura GPU inspecionada: player laranja sobre plataforma, azul com cone horizontal sem detecção, roxo selecionado com cone inclinado verde detectando player. Ambos usam wait e ficam parados intencionalmente. Alcance9m, FOV45°, pitch0°/35°, yaw voltado à plataforma, memória180ticks; busca com giro0 na demo. P executa, V guias, R reset. Não é demo de perseguição ou combate.

Binário corrente target/release/ignis8.676.320bytes. Bundle artifacts/Ignis.app continua em uso pela sessão antiga PID20015, build anterior cbb2c441cafb27eb; preservado sem substituir/fechar/reabrir/ativar. Essa cópia ativa é mantida enquanto em uso, não constitui novo histórico de builds. Próximo empacotamento após encerramento dessa sessão deve renovar o hardlink com scripts/package_macos.py. Não houve instalação/publicação/push. Para a nova demo, usar explicitamente target/release/ignis quando desejado. Aceite humano EAF-32 pendente.

Roadmap atualizado com situação por frente e critérios integrados de NPCv1. Próximo bloco: superfícies/navegação e rotinas espaciais, antes de integrar ações/habilidades/ciclo de vida, comunicação/interações e apresentação. Não declarar M09 completo ou percentual de conclusão sem detalhar essas dependências. EAF-30 recebeu aceite explícito do usuário; isso não substitui o aceite próprio desta entrega.


## EAF-31 — validação MCP sem janela (23/10/2026)

Regressão completa com `python3 scripts/run_verifications.py --binary target/release/ignis`: 29 roteiros, 126 grupos aprovados, todos com native_window=false, sem retries. Executável corrente cbb2c441cafb27eb reutilizado (mesmo build de EAF-30); nenhuma mudança Rust, build/app novo ou execução de cargo necessária nesta unidade de tooling/documentação. Nenhuma janela aberta, fechada, focada ou controlada para validar esta mudança.

Executor serial, workspace estável em artifacts/mcp-validation, cópia dos exemplos e logs por cenário. Relatório incremental report.json distingue seleção, resultados, checks, status e falhas; uma falha para a suíte e retorna código não zero. Opt-in --window exige --only. AGENTS, README/config MCP e instruções correntes usam headless; registros históricos com janela permanecem como evidência do que de fato foi executado antes. Scripts individuais continuam podendo autorar demos explicitamente no projeto; a suíte usa IGNIS_VERIFY_ROOT para isolar gravações.

Validação adicional pelo MCP sem janela: play avançou até tick5, pause congelou o snapshot durante80ms, step60 chegou exatamente a tick65 com movimento aplicado e replay correspondente. Evidência em artifacts/mcp-validation/artifacts/headless-clock-verification.json e headless-clock.replay.json. Captura flight-curves.png inspecionada diretamente do arquivo: três voadores, trajetórias curvadas e órbita visíveis; comparações GPU/replay constam dos roteiros. Não equivale a testar apresentação nativa/foco/teclado físico.

Contrato do executor: --window sem recorte rejeitado antes de iniciar; segunda suíte rejeitada sob flock; binário propositalmente inválido /usr/bin/false produziu relatório failed/código1, apenas primeiro roteiro registrado e nenhum retry/cenário seguinte. Esse teste negativo usou diretório temporário removido ao terminar e não substituiu o relatório da regressão aprovada. Registro em artifacts/mcp-validation/runner-verification.json. git diff --check passou; exemplos versionados intactos. Checklist estático antes/depois respondido. Sem alegar correção das intermitências históricas apenas por esta execução bem-sucedida.

Mantidos target/release/ignis e artifacts/Ignis.app existentes, sem novos outputs de build/instalação ou alteração da demo aberta. Operação headless validada no macOS atual; não declara suporte novo a plataformas, consumo zero ou editor conectável a uma sessão existente. Testes de integração desktop devem ser pedidos explicitamente quando necessários.


## EAF-30 — voo curvado configurável (23/10/2026)

Build `cbb2c441cafb27eb`, macOS Apple Silicon/Metal (Apple M4 Pro), cena/replay22. Perfil opcional straight/curved pelo MCP. Curved limita velocidade, aceleração/frenagem vetorial e giro, filtra direção por suavidade, mantém direção na chegada e diferencia parada/pairar de passagem com órbita. A velocidade mínima é de cruzeiro; colisão/desativação podem impor parada de segurança. Previsão física limitada do trecho e da órbita usa o mesmo controlador de execução; o grafo conserva estimativas nominais. Sem dependência nova; checklist antes/depois respondido em performance.md.

186 testes Rust release (9 novos), fmt --check, clippy -D warnings e build passaram. Testes novos: curva XYZ com limites por tick/parada/replay; cruzeiro mínimo/órbita; curva bloqueada com reta livre; efeitos independentes de suavidade e giro/validação atômica; entrada/rede/saída e ordem/prefixo; memória expirada/nova pista/budget da curva; colisão dinâmica/legado straight; histórico/reset e destino inalcançável durante holding; espaço da órbita e limites de trabalho. O teste dinâmico revelou que erro por tick isolado não detectava bloqueio persistente: agora seis desvios consecutivos acima5mm ou erro instantâneo acima15cm interrompem o voo. Comparações do orçamento usam o raio efetivo da intenção sonora, que pode diferir do objetivo fixo. Warning preexistente de compatibilidade futura de block0.1.6 permanece.

29 roteiros MCP,126 grupos com execuções finais aprovadas; todos exceto laboratório com janela nativa. Quatro grupos novos cobrem autoria/schema, curva versus reta em XYZ, parada e duas voltas completas sem pairar, mudanças/erro atômico/undo/redo/load/reset, preview e direção visual sem efeito na simulação, replay/prefixo/PNG idênticos. Novo roteiro sem slow_requests. Capturas flight-curves.png e flight-holding.png inspecionadas. Evidências em artifacts/flight-verification.json, flight-*.replay.json/PNG, mcp-regression.json e logs Rust/clippy/build.

A primeira demo tinha a órbita verde cruzando o corpo roxo parado. Extensão para duas voltas revelou deslocamento; diagnóstico pelo MCP confirmou contact_started no tick874 e contact_ended no884, preservados em artifacts/flight-contact-diagnostic.json. O posicionamento inicial/destino do verde mudou deZ4 paraZ7, separando os percursos; a demo corrigida passou pelas duas voltas. Não foi desativada colisão nem implementado desvio de atores móveis. Uma execução legada de verify_hearing_guides falhou no limite16 de guias de ruído sem valores diagnósticos; acrescentado registro de captura/estado/ruídos e duas repetições passaram no mesmo build. Causa não estabelecida, sem correção funcional alegada; ocorrência e repetições preservadas em mcp-regression.json, junto aos diagnósticos antigos.

Demo examples/npc-flight.scene.json autorada/salva integralmente pelo MCP: azul reto, roxo curvado e pairador, verde curvado e orbitador. Destinos fixos elevados, não perseguição. Máximo3m/s; curvos usam aceleração/frenagem3m/s², giro45°/s, suavidade0,4s e direção inicial[0,0,-1]; verde mínimo1m/s. P inicia/pausa, R reset, J guias violetas/direção amarela; jogador laranja em WASD. As caixas não inclinam; heading XYZ é diagnóstico do motor, distinto da orientação sensorial horizontal. Continuidade em todas as junções, aerodinâmica, navegação global por dinâmica, inclinação visual e desvio de móveis permanecem fora deste recorte.

Dezoito exemplos legados restaurados após comparação estrutural confirmar somente versão22/rede vazia. App corrente por hardlink do release (8.659.280bytes, mesmo inode), aberto com npc-flight; MCP confirmou tick0/playing=false, curved selecionado e rotas ativas. Instância anterior encerrada por MCP exit; um build release corrente, sem diretório debug/worktree ou cópia histórica adicional. EAF-29 recebeu sinalização de aceite do usuário e foi encerrado; EAF-30 tem implementação/validação local concluídas e aguarda aceite próprio. M09 parcial; sem push/publicação.

## EAF-29 — conexões locais de caminhada e voo (23/10/2026)

Build `8bf4392904bd00eb`, macOS Apple Silicon/Metal (Apple M4 Pro), cena/replay21. Política local opcional por NPC conecta posição real, rede e destino conhecido por walk/fly com alcance limitado. Dijkstra multi-origem compara custo completo com conexão direta; a política parcial inclui custo de entrada e conserva desvios. Colisão, apoio, capacidade, compromisso após perda/troca de pista, timeout e budget permanecem ativos. Sem dependências novas. Revisão estática antes/depois em performance.md; segmentos lineares agora usam uma varredura inteira, amostragem fica no salto. Sem medição ou promessa de FPS.

177 testes Rust release (10 novos), fmt --check, clippy -D warnings e build passaram. Novos casos cobrem direto/XYZ, entrada-rede-saída, salto autorado, alcance/parede/lacuna, parcial/custo/ordem, memória/budget/privacidade, colisão dinâmica, seleção de modo, atomicidade/versão/histórico/reset, alinhamento perto do limite e parada sem contato com o jogador. Warning preexistente de compatibilidade futura de block0.1.6 permanece.

Na primeira integração MCP, o walker terminou dois trechos e recusou a saída: a posição física ainda estava antes do nó, fazendo a distância real exceder o limite apesar da saída nominal válida. Regressões adicionadas e chegada aos nós mais precisa no modo local. Um teste no limite exato mostrou resíduo de aproximadamente0,6mm sob contato com o piso; exigir1e-6m produziu timeout, não precisão útil. A solução final compartilha tolerância física explícita de2mm entre chegada ao nó e alcance local, sem relaxar a geometria de colisão. Caso3mm além do alcance permanece rejeitado. Conexões finais respeitam o raio de parada congelado, sem tentar ocupar o centro do jogador. Erros iniciais de fixture (parede abaixo do tamanho mínimo e consulta a entidade errada) foram corrigidos no teste; classificação direct/exit foi ajustada para ancoragem na rede.

28 roteiros MCP,122 grupos passaram no build final; todos exceto laboratório com janela nativa. O novo roteiro confirma chegada de walker/flyer com3 trechos cada, perseguição de jogador móvel, autoria/histórico/erro atômico, replay completo/prefixo, PNG vivo igual ao replay e guias sem efeito físico. Falha de integração anterior preservada em connectors-first-attempt.json e earlier_build_failure de mcp-regression.json; diagnósticos intermitentes antigos continuam em prior_build_history. Os roteiros legados de cenário e guias passaram nesta execução, sem alegar correção das causas antigas. Novo roteiro sem slow_requests.

Demo examples/npc-connectors.scene.json construída e salva exclusivamente pelo MCP. Roxo perceptivo: visão360°/16m, memória3s, busca1s, audição16m, caminhada3m/s, conexões locais4,5m e distância de parada0,8m. Laranja é o jogador; WASD move e N emite ruído, sem passos automáticos nesta cena. Azul: voo3m/s/conexões6m, objetivo fixo no ar do outro lado da parede, sem perseguir o jogador. P inicia/pausa, R reinicia, J mostra rede/caminho/conexão ciana, V visão. Percepção360° não vê através de paredes; trecho comprometido pode continuar após perder a pista.

Capturas connectors-demo.png e connectors-traversing.png inspecionadas: roxo no percurso terrestre, azul em ascensão, rede acima da parede e jogador laranja. Evidências em artifacts/local-connections-verification.json, connectors-*.replay.json/PNG, rust-tests.log, clippy.log, build.log e mcp-regression.json. Dezessete exemplos antigos restaurados após comparação estrutural confirmar apenas versão21/rede vazia; somente a nova demo foi adicionada.

App corrente recomposto por hardlink do release (8.574.800bytes, mesmo inode), aberto com a nova demo e confirmado pelo MCP emtick0/playing=false, walker selecionado e guias de rota/visão ativos. Um output release corrente, sem novo diretório debug/worktree ou cópia histórica. Implementação e validação local concluídas; aceite humano de EAF-29 permanece pendente, assim como os aceites próprios de EAF-27/EAF-28. M09 parcial; sem push/publicação. Limites: rede para desvios maiores, apoio em caixa plana comum, sem descoberta global/navmesh/rampas/steering contínuo/desvio dinâmico; orientação sensorial volumétrica e integração das rotinas3D continuam abertas.

## EAF-28 — aproximação parcial configurável (23/10/2026)

Build `1855532bfe648464`, macOS Apple Silicon/Metal, cena/replay20. Política independente de capacidades: stop padrão, approach_reachable opcional com progresso mínimo e esforço nominal limitado. Caminho completo tem prioridade. Fallback reaproveita Dijkstra e conserva rota inteira através de desvio inicialmente contrário; chegada parcial mantém objetivo original e status unreachable. Sem replanejar parado; mudança/perda de pista termina somente trecho comprometido antes de invalidar continuação. Apoio, colisões, capacidades e budgets permanecem ativos.

167 testes Rust release (5 novos), fmt --check, clippy -D warnings e build passaram. Novos testes cobrem padrão/rota completa preferida, desvio/backtracking, progresso/custo/empates/ordem, expiração/troca de pista, replay/prefixo, validação atômica/versão/reset. O teste de nova pista foi corrigido para observar chegada transitória: depois da investigação com search_ticks1 o estado correto é Idle; nenhuma mudança de runtime para reter uma pista expirada. Checklist de performance antes/depois respondido em performance.md. Warning de compatibilidade futura de block0.1.6 permanece, sem erro novo.

27 roteiros MCP,118 grupos com execuções finais aprovadas; todos exceto laboratório com janela nativa. O novo roteiro inicialmente exigia igualdade exata do destino Y=1; inspeção retornou0.9999999403953552 após subtração f32 de alturas. Comparação corrigida para tolerância1e-6; destino original e estado não mudaram. Aproximação termina com completed1/plans1 e permanece partial_reached durante mais600ticks, sem replanejar. Replay completo/prefixo e PNG coincidem; limites/erros/histórico/guias passaram. Câmera foi ajustada para mostrar cinza/base e o roteiro novo foi repetido com sucesso após essa alteração final de autoria.

Ocorrência aberta: verify_scenarios falhou uma vez comparando snapshot vivo após execução isolada (antiga linha43). A asserção não emitia diferenças. Acrescentado diagnóstico por campo a todas as comparações de isolamento; duas repetições nativas no mesmo build passaram. Causa não estabelecida e não há correção funcional alegada. Falha original e repetições permanecem em artifacts/mcp-regression.json. Ocorrência visual do build anterior também preservada como prior_build_history; verify_vision_guides passou nesta execução. Não atribuir falhas a foco/teclado ou ao novo motor sem evidência. Próxima reprodução deve registrar campos diferentes para localizar a origem.

Demo `examples/npc-mobility.scene.json` atualizada exclusivamente via MCP: cinza walker caminha de[-5,0,-7] até o nó de base[-2.5,0,-7] (pés), conserva destino[-1,1,-7] sobre plataforma e não salta. Configuração: min_progress0,25m, max_path_ticks300 (5s estimados), caminhada4m/s, percurso nominal2,5m/0,625s. P inicia, R reinicia, J alterna guias. X laranja indica nó parcial; X vermelho sobre plataforma indica destino ainda inalcançável. Roxo/azul/verde conservam capacidades e objetivos fixos; não é perseguição ao jogador.

Captura `artifacts/partial-route.png` inspecionada: cinza selecionado junto à face da plataforma, destino vermelho acima dela, roxo em salto, azul em voo e verde em planeio. `partial-route-replay.png` é idêntica; relatórios partial-route-verification.json, mcp-regression.json, rust-tests.log/clippy.log/build.log. Exemplos legados regravados pelos roteiros foram restaurados após comparação estrutural restringindo diferenças a versão/rede vazia; somente a demo espacial foi alterada intencionalmente.

App corrente `artifacts/Ignis.app` atualizado por hardlink (8.523.984bytes, mesmo inode do release), sem cópia histórica. Instância anterior encerrada via MCP exit. Nova demo aberta, confirmada pelo MCP emtick0/pausada/build1855532bfe648464, walker selecionado e guias de rotas ligados. Um único output release corrente, sem novo worktree ou diretório debug criado. Implementação/validação local concluídas; aceite humano de EAF-28 e EAF-27 permanece pendente, M09 parcial. Sem push/publicação. Limites: rede explícita, sem conectores automáticos/navmesh/decisão tática, custo nominal não é timeout físico.


## EAF-26 — reserva cooperativa de passagens (23/10/2026)

Cena/replay18, build final977e0f377f1ed26a, macOS Apple Silicon/Metal. Passagens explícitas retas, vagas de espera por lado, NPC opt-in/priority, escolha por prioridade+idade/ID, posse não preemptiva e liberação física/cancelamento/timeout. Gate bloqueia comando sem reserva inclusive em desvio incidental; colisão/push permanecem físicos. Objetivo original e percepção preservados; distância investigativa não cobrada na espera, relógios continuam. Controle set_passages/edit_scene/upsert, inspect_passages, guias J e replay pelo dispatcher comum.

153 testes Rust release (10 novos), fmt --check, clippy -D warnings/build aprovados. Testes: travessia oposta/retomada/replay; prioridade/empate/ordem/não preempção; ocupação/atomicidade; timeout; legado/reset; aging/cancelamento; memória/budget; gate/retirada;3 agentes/vagas distintas/eixoZ; validação de geometria/superfície/sobreposição. Fixture de audição inicialmente tinha search_ticks0, rejeitado corretamente; corrigido para1 sem alterar contrato. Revisão estática eliminou normalize/sqrt por tick em trechos alinhados. Simplificação posterior de três asserções idênticas para uma no teste de segmento degenerado, sem alterar cobertura/implementação.

112 grupos MCP passaram no build final (25 scripts; todos exceto laboratório com janela nativa).3 grupos novos de passagens: exclusão/retomada, prazo/ocupação/liberação física, autoria/histórico/erro atômico/guias. Captura e replay PNG idênticos, prefixo isolado, estado de reservas comparado. Durante regressão de build anterior, verify_patrol.py não observou retomada em900passos uma vez; repetição no mesmo runtime passou emtick922 sem correção funcional, assim como suíte final. Causa não estabelecida. Verificador agora imprime estado/input/nav em eventual falha, e a ocorrência foi preservada como earlier_build_failure no relatório; não alegar correção de causa nem atribuir a foco/teclado sem evidência.

Evidência: artifacts/passages-verification.json, passages[-waiting].replay.json, passages-waiting[-replay].png, passages-guides.png, mcp-regression.json, rust-tests.log. PNG inspecionada: roxo do lado esquerdo em aproximação, azul deslocando para espera lateral, laranja fora da passagem, faixa verde e vagas amarelas. Exemplos antigos regravados pelos roteiros restaurados após verificar diff somente de versão.

Demo examples/npc-passages.scene.json autorada integralmente pelo MCP: roxo prioridade10, azul0, velocidade2m/s; objetivo guard de trocar lados, player laranja. Corredor a[-1,0]/b[1,0], largura1,4m, duas vagas/lado; lease600ticks/10s, retry60ticks/1s, aging60ticks/1 ponto por segundo. P inicia, R reinicia, J guias. Passagem exige configuração explícita; não descobre gargalos nem encadeia links automaticamente. Bloqueios físicos/cancelamento sem evacuação continuam limites, sem garantia geral para multidões/deadlock/FPS. Implementação/validação prontas, aceite humano EAF-26 pendente; EAF-25 aceito/encerrado, M09 parcial. Sem push/publicação.

## EAF-25 — recuperação de navegação (23/10/2026)

Cena/replay17. Política opcional de espera exponencial limitada, detecção de ausência de deslocamento após movimento solicitado, retry_wait e diagnósticos no inspect_navigation. Espera evita reconstrução do mapa/A*. Sem configuração, comportamento anterior preservado. Progresso real/chegada limpa sequência; idle/disabled/troca de objetivo/território/reset limpa estado. Sensores/budgets continuam; conhecimento oculto não é consultado. Marcador vermelho reutiliza guia existente.

143 testes Rust release, fmt --check, clippy -D warnings e build aprovados. Seis testes novos: relógio/teto/fronteira/progresso; passagem liberada fisicamente sem reset e replay; corpo congelado apesar do comando, idle/desativação; validação atômica/versão/legado/reset; som interrompe espera e expiração retoma guarda; retorno territorial comprometido com recuperação. O primeiro roteiro movia o jogador para o próprio destino do guarda, mantendo-o bloqueado; corrigido o movimento para diagonal que libera passagem e destino. Um Box ausente no teste foi corrigido antes da suíte final; nenhum workaround de runtime para esses erros de teste.

109 grupos MCP aprovados:106 regressões+3 novos de verify_nav_recovery.py, todos exceto laboratório com janela nativa. Cadência exata (3 tentativas em100ticks bloqueados), liberação com input real, chegada, replay/prefixo isolado, pixels iguais, autoria/persistência/undo/redo/reset, erro atômico e guias sem efeito na simulação. Exemplos antigos regravados pelos scripts restaurados às versões autorais.

Build fc853419d95e3eab, macOS Apple Silicon/Metal. artifacts/recovery-verification.json, recovery[-blocked].replay.json, recovery-blocked[-replay].png, recovery-arrived.png, mcp-regression.json e rust-tests.log. Captura bloqueada inspecionada: parede com vão, jogador laranja bloqueando, guarda roxo à esquerda e X vermelho à direita. examples/npc-recovery.scene.json criada/salva pelo MCP. P inicia, D+S por cerca de1s libera o vão e o destino, R reinicia, J rotas. Guarda2m/s, espera0,25→0,5→1s, janela0,5s/progresso0,1m; objetivo é posto fixo, sem perseguir jogador.

Revisão estática antes/depois concluída. Implementação/validação automatizada concluídas; EAF-25 recebeu aceite humano (“isso ficou MUITO bom”) e foi encerrado. Aceite EAF-24 não foi inferido de “faz sentido”. Espera tem latência, deslocamento não prova avanço topológico; negociação de passagem/deadlock/prioridades entre NPCs é próximo recorte. M09 parcial; sem profiling/promessa de FPS ou push/publicação.

## EAF-24 — território e retorno comprometido (23/10/2026)

Cena/replay16. Território circular opcional, filtros após sensor/antes do ranking, retorno por deslocamento próprio/destino conhecido externo/saída observada do alvo ativo sem alternativa interna. Posição oculta não informa saída. Retorno limpa memória/suspeita/recibo, suspende reação/objetivos/patrulha, termina por chegada física e espera configurada. Origem bloqueada expõe falha sem teleporte. Guias laranja/vermelho e origem junto de npc_routes/J; inspect_territory no MCP.

137 testes Rust release, fmt --check, clippy -D warnings e build passaram. Oito novos cobrem fronteiras/cooldown/deslocamento/zero espera; saída observada/compromisso/rearme; aquisição externa e ocultação; alternativa sonora/visual interna antes do ranking; retorno físico/patrulha/origem bloqueada; fuga externa/desativação; validação atômica/versão/legado/histórico/replay/prefixo; guias/cores sem modificar estado. Exemplos antigos regravados pelos scripts foram restaurados às versões autorais.

106 grupos MCP passaram:102 regressões+4 novos de verify_territory.py, todos exceto laboratório com janela nativa. Fluxo real do jogador saindo, guarda retornando sem nova aquisição, retomada de patrulha, filtro sonoro, origem bloqueada, validação/persistência/undo/redo, guia ligada/desligada, replay/prefixo e PNG idêntico à captura viva. Primeiro roteiro usou output_path em capture_replay; rejeição correta do schema, ajustado para replay_path/path. Não houve mudança de runtime por esse erro do verificador. Enquadramento final da demo melhorado via set_camera em lote e roteiro novo repetido com sucesso.

Build final51a286d08b43fdcb, macOS Apple Silicon/Metal. Relatório territory-verification.json, replays territory/territory-returning/territory-demo, PNGs e mcp-regression.json em artifacts. territory-demo.png inspecionada após enquadramento final: roxo/laranja, parede, círculo e origem visíveis. examples/npc-territory.scene.json autorada pelo MCP: raio5m, chegada0.4m, espera2s, guarda1.5m/s/visão360°12m/audição10m/memória3s/busca1.5s. Player sem passos automáticos; P/WASD/N/R/J. FOV360° intencional para isolar território.

Revisão estática antes/depois concluída. Implementação/validação automatizada concluídas; aceite humano deste incremento pendente. Limite comportamental permite desvio/empurrão além do círculo; retorno bloqueado permanece explícito. Sem navegação restrita à área, recuperação tática, combate ou promessa de FPS/plataformas adicionais. M09 parcial. EAF-23 encerrado após aceite humano; acompanhamento azul continua configuração de resposta, sem módulo específico de cooperação. Sem push/publicação.


## EAF-23 — composição de objetivos e interrupção de rotinas (23/10/2026)

Cena/replay15. objectives opcional por NPC,1..8 entradas de prioridade/condição/ação: respond, flee, patrol, guard e wait. Seleção sobre evidência aceita, empate pela primeira entrada, um dono de movimento e diagnóstico MCP. Investigação suspende execução e cobrança de distância quando outra ação vence; percepção permanece ativa e memória/busca expiram por relógio. Patrulha conserva ponto/direção, reinicia etapa. Troca invalida caminho/feedback anterior. Fuga usa ponto radial e conserva posição quando já está distante; não conhece coordenadas ocultas.

129 testes Rust release, fmt --check, clippy -D warnings e build passaram. Oito testes novos: prioridade/empates/condições/requisitos; patrulha→som→retomada; guarda com audição/budget/expiração; fuga/congelamento/coincidência/já distante/movimento; posição oculta e desativação; feedback antigo e espera; validação atômica/versão/replay/prefixo/undo/redo; expiração de busca suspensa e legado. Exemplos regravados pelos scripts foram restaurados às versões antigas para preservar migração.

102 grupos MCP passaram:98 regressões e quatro novos de verify_objectives.py, todos exceto laboratório com janela. Exercitados interrupção/retomada real, capturas, replay/prefixo isolado, configuração inválida atômica, guarda que mantém posto, expiração, save/load/histórico, fuga física/espera, destino inalcançável e demonstração simultânea de três papéis. A primeira asserção de persistência comparava0.3 Python com o retorno f32 0.30000001192092896; corrigido o verificador para comparar a política normalizada lida antes de salvar com a reaberta. Não foi alteração do runtime. Roteiro e regressão posteriores passaram.

Build finale17c40bfbb7d88d9, macOS Apple Silicon/Metal. artifacts/objectives-verification.json, objectives.replay.json, objectives-flee.replay.json, objectives-demo.replay.json, mcp-regression.json e rust-tests.log. Captura objectives-demo.png inspecionada: roxo selecionado/guarda, ciano civil, azul aliado, laranja player, parede e guias; estados confirmados no mesmo passo por inspect_objectives. Demo examples/npc-objectives.scene.json criada/salva integralmente pelo MCP. P inicia, WASD move, N ruído manual; passos automáticos desligados. Guarda patrulha inicialmente atrás da parede, ruído interrompe; civil foge do jogador visto, aliado acompanha. Nenhum ataque/dano. Parâmetros detalhados no README.

Revisão estática antes/depois registrada. Implementação/validação automatizada concluídas; aceite humano explícito recebido em23/10/2026: “ok tudo funcional”. Fuga radial pode ficar bloqueada, sem busca alternativa; prioridades estáticas/preempção imediata, uma memória ativa, mapa plano conhecido. Sem promessa de framework geral, escala/FPS ou outras plataformas validadas; M09 parcial. Próximo recorte a detalhar: território/retorno; recuperação tática depende de navegação ampliada. Sem push/publicação.


## EAF-22 — prioridade e estabilidade da investigação sonora (23/10/2026)

Cena/replay14. sound_attention opcional: compromisso mínimo, margem aditiva de intensidade, supressão espacial de repetição e proteção temporária da memória visual. Visão confirmada atual continua prioritária. O filtro precede o ranking; intensidade fica congelada na recepção, sem consultar emissor oculto. Rejeitar não renova budgets nem enfileira a pista. Expiração da memória permite nova aquisição. Ausência da política preserva escolha legada. inspect_hearing expõe contagens, motivo, candidato mais forte e pista efetivamente selecionada pelo MCP.

121 testes Rust release, fmt --check, clippy -D warnings e build passaram. Oito testes novos cobrem compromisso/repetição/margem e budgets; alternativa elegível e recibo congelado; visão atual/proteção de memória; expiração por memória/busca/desistência/desativação; eventos automáticos/manuais, empate e legado; fronteira aditiva e ausência de fila; ordem/física/replay/prefixo/undo/redo; validação e erros atômicos. Exemplos antigos regenerados pelos scripts foram restaurados às versões autorais para manter cobertura de migração.

98 grupos MCP passaram:94 regressões e quatro novos em verify_sound_attention.py, todos exceto laboratório com janela. Novo roteiro cobre retenção/troca, save/load/undo/redo/erro atômico/expiração, proteção visual e autoria integral da demonstração pelo MCP. Captura sound-attention-demo.png inspecionada: roxo selecionado, jogador laranja, barreira e guias acústicas visíveis. Replay completo e prefixo reproduzem decisões. Build final06953c7c650f3ede, macOS Apple Silicon/Metal; artifacts/sound-attention-verification.json, sound-attention.replay.json, sound-attention-visual.replay.json, sound-attention-demo.replay.json e mcp-regression.json.

Demo examples/npc-sound-attention.scene.json isola audição: guarda sem alvo visual, jogador sem passos automáticos, N emite impulso manual e P inicia/pausa. Compromisso90ticks/1.5s, margem aditiva0.15, raio de repetição1.25m, audição10m, memória600ticks/10s, busca180ticks/3s, velocidade1.5m/s. visual_priority_ticks60 fica inativo nessa cena sem alvo; foi exercitado no cenário automatizado separado. Caminhar perto do guarda sem N não emite pista nesta demonstração.

Revisão estática antes/depois concluída. Implementação e validação automatizada concluídas; aceite humano desta cena pendente. Sem medição de FPS, suporte validado em outras plataformas, fila de tarefas, ameaça de combate ou conclusão de M09. Próximo incremento: composição de objetivos, com prioridades, interrupção e retomada; contrato a discutir antes de adotar framework. Sem push/publicação.



## EAF-21 — confirmação visual gradual por indivíduo (23/10/2026)

Cena/replay13. recognition opcional(acquire_ticks/decay_ticks), exposição independente por ator, confirmação latched até zero, última posição somente observada, seleção apenas de confirmados visíveis. Memória ativa de perseguição mantém seus budgets; sons continuam anônimos e podem iniciar investigação antes do limiar visual. inspect_knowledge e inspeções gerais expõem estado; guia amarelo para suspeita sem memória ativa, verde para confirmação observada.

113 testes Rust release, fmt --check, clippy -D warnings e build passaram. Oito testes novos cobrem: limiar/independência, perda e posição oculta congelada, decay/recuperação/limites1 e3600, explícito/imediato/onisciente, parede/ruído/desativação, patrulha antes/depois, ordem/inspeção/replay/prefixo/reset/undo e validação/migração. O teste legado de alvo opcional agora fixa versão10: os verificadores MCP podem salvar o exemplo em versão atual, o que anteriormente tornava essa asserção dependente da ordem das suítes. Exemplos antigos foram restaurados após os roteiros.

94 grupos MCP passaram no build final:90 anteriores e quatro novos em verify_recognition.py, todos exceto laboratório com janela nativa. A confirmação ocorre exatamente no tick configurado; prefixo de replay e PNG do estado suspeito são idênticos à execução viva. Configuração salva/recarregada, undo/redo e erros atômicos preservam contratos. Sons não confirmam identidade, onisciência explícita não espera, demo é criada pelo MCP.

Durante o desenvolvimento houve uma falha única na comparação do snapshot após edição inválida, sem diff capturado naquela execução. A asserção ganhou diagnóstico por campo; a repetição imediata, três execuções subsequentes e a regressão final passaram. Causa não estabelecida; não tratamos repetição bem-sucedida como correção comprovada. Se reaparecer, registrar o diff de estado antes de atribuir à edição, captura ou eventos nativos.

Build final2d438f74d24d2b14, macOS Apple Silicon/Metal. artifacts/recognition-verification.json, recognition.replay.json e recognition-demo.replay.json; imagens recognition-demo-pending.png/recognition-demo-confirmed.png inspecionadas visualmente. Dois atores e parede: roxo guarda, laranja jogador. Guias amarelo e verde distinguíveis, sem mudança de geometria. Demo examples/npc-recognition.scene.json: aquisição120ticks/2s, decay240ticks/4s, memória180ticks/3s, busca90ticks/1.5s, visão120°/12m, audição10m, velocidade1.5m/s. Player tem passos físicos; parado inicialmente isola a visão.

Revisão estática antes/depois registrada. Sem ganho de FPS alegado. Implementação/validação automatizada concluídas; aceite humano da nova cena pendente. Não há combate, identificação facial, fila de pistas acústicas ou memória espacial multiobjetivo. M09 permanece parcial; sem push/publicação.


## EAF-20 — relações e seleção entre hostis percebidos (23/10/2026)

Cena/replay12. Facção opcional, regras direcionais, matriz compilada, hostile_nearest opt-in com margem relativa; target explícito e ausência continuam compatíveis. Memória visual identifica ator por índice interno/ID público, som permanece anônimo. Controles/inspeções e autoria da demonstração via MCP compartilhado.

105 testes Rust release passaram (oito novos), cargo fmt --check, clippy --release --all-targets -D warnings e build. Novos testes cobrem padrões/direção/override, empate e margem, perda/expiração, parede/FOV/alcance, posições ocultas sem efeito na memória, onisciência opt-in, ruído anônimo, disabled, capacidade9 atores, autoria invertida, validação atômica/migração/undo/redo/replay. Apenas aviso transitivo já existente de futura incompatibilidade block0.1.6.

90 grupos MCP passaram:85 regressões e cinco novos em verify_targeting.py, todos exceto laboratório com janela. Roteiro novo: relação direcional e aliado próximo excluído; empate de hostis e snapshot pré-física; retenção e troca real movendo player até superar margem; save/replay/prefixo/captura isolada; erro atômico e invalidação/undo/redo; som sem identidade; cena demonstrativa persistida/recarregada com patrulheiro autônomo. Para isolar a margem, esse teste usa guarda0.1m/s e FOV360°; a demonstração usa guarda1.5m/s e FOV120°. Imagem targeting-demo.png inspecionada: quatro atores, contorno do guarda, guias e patrulha visíveis.

Build 4a2c8428aebe47e8, macOS Apple Silicon/Metal. Relatório targeting-verification.json, replays targeting/targeting-sound/targeting-demo e capturas em artifacts. examples/npc-relations.scene.json contém jogador laranja com passos físicos, guarda roxo, aliado ciano e patrulheiro vermelho; guarda considera player e raider hostis. Visão12m/120°, audição10m, memória180ticks/3s, busca90ticks/1.5s, margem20%; patrulheiro1.2m/s. Sem dano/ataque.

Implementação e validação automatizada concluídas; aceite humano desta demonstração separado/pendente. M09 permanece parcial. Uma memória, mapa global/plano, até8NPCs; não há reputação, ameaça de combate, memória multiator ou medição de FPS. Sem push/publicação.

## EAF-19 — autonomia e alvos NPC (23/10/2026)

Continuidade priorizada em NPCs documentada em npc-roadmap.md. Primeiro incremento: target opcional/null ou NPC/player distinto, sem fallback implícito para jogador. Cenas/replays11; cenas1..10 continuam validadas com alvo player obrigatório antes de normalizar. Rotinas e audição independentes de alvo; ciclos de referência entre agentes são observados no snapshot comum pré-física. Pistas anônimas usam altura do ouvinte, sem consultar dimensões do alvo não identificado.

97 testes Rust release, clippy -D warnings e build passaram. Seis novos testes verificam patrulha/ruído/retomada/replay sem jogador, ausência de aquisição implícita até em onisciência, perseguição mútua/ordem autoral, movimentos ocultos de NPC sem alteração de memória/rota, migração/erros atômicos/remoção em lote/undo e independência de dimensões do alvo na pista sonora.

85 grupos MCP aprovados, todos exceto laboratório com janela. Três novos: cena sem player e target com patrulha/pista/retomada; NPC persegue outro em movimento usando observação pré-passo; referências inválidas e remoção protegidas, undo/redo/save/load e replay/prefixo. Exemplo npc-autonomy.scene.json construído/salvo pelo MCP: civil azul sem alvo, patrulha ao redor da barreira, hunter roxo com alvo civilian, jogador laranja fora da seleção visual. No tick241 hunter observou civilian no tick240 em[-4,0.79893,-1.453897]; civil estava no trecho1 da patrulha. Captura npc-autonomy-moving.png inspecionada. Hashes antigos de replay permanecem iguais.

Build efb729067ee17096, macOS/Metal. Relatório npc-autonomy-verification.json e replays npc-autonomy/npc-targetless em artifacts. Aceite humano desta nova semântica pendente. Nenhuma conclusão de M09: alvo ainda único/explicitamente configurado, sem relações sociais ou seleção de ameaças; mapa plano conhecido, até8NPC/128corpos. Audição em outra altura pode produzir pista inacessível ao navegador atual. Sem push/publicação.


## EAF-18 — seleção e inspeção de entidades (23/10/2026)

select_entity/select_at/inspect_selection no dispatcher/MCP e clique esquerdo mapeado ao mesmo SelectAt. Seleção de sessão com índice cacheado, contorno dourado, ID no título e metadados de captura. Picking usa projeção compartilhada e posições físicas atuais, visíveis, near/far e desempate por ID. Nenhum formato de cena/replay mudou.

91 testes Rust release, clippy -D warnings e build passaram. Sete novos testes (seis integração Rust/um cache) cobrem superfície/distância/closest/miss, empate independente de ordem, exclusão de invisíveis com seleção explícita permitida, posição física atual em três viewports, clipping/segmento paralelo, valores inválidos atômicos, reindex/reset/remove/undo/redo e componentes. Contorno tem144 vértices e nenhum triângulo degenerado, inclusive nas arestas verticais; memória reutilizada ao mover/limpar.

82 grupos MCP passaram: quatro novos de seleção e78 anteriores, todos exceto laboratório com janela nativa. Capturas repetidas são iguais; limpar seleção restaura PNG original. Replay herda ID existente, gera os pixels exatos da partida viva e não muda seu estado; ID ausente é omitido. Remoção/rename e undo/redo não inventam seleção. Hashes das quatro capturas do roteiro antigo de replay permanecem iguais. Relatório selection-verification.json, imagens selection-npc.png/selection-region.png e selection.replay.json em caminhos correntes. Captura do NPC inspecionada: caixa roxa contornada no laboratório combinado.

Build validado: 9d0902f5425252f7, macOS Apple Silicon/Metal. Clique físico não exercitado no Mac bloqueado: native_window=true prova janela/MCP/captura, native_click_tested=false explicita o limite. Aceite humano permanece pendente; sem publicação, sem picking de meshes/gizmos/editor completo e sem medição de FPS.


## EAF-17 — edição atômica de cenas (23/10/2026)

edit_scene de1..512 edições tipadas, staging/validação final/um reset/revisão/undo, paridade dos comandos individuais. Payload e cena anterior movidos; histórico cheio descarta antes de push. Sem novo formato. Checklist de performance antes/depois e contratos de referências futuras/ordem/limites registrados.

84 testes Rust release, clippy -D warnings e build passaram, incluindo cinco novos de lote e a correção de foco independente EAF-11. Testes cobrem renomeação com referências temporariamente ausentes, erros locais/globais/obsoletos preservando partida em play e redo, schema sem runtime/recursão,512 operações, staging768→final512 e paridade do replay com comandos individuais.

78 grupos MCP aprovados, todos exceto laboratório com janela. Três novos grupos constroem examples/stealth-lab.scene.json: player renomeado hero, alvo do hunter atualizado, região e marcador criados em lote com referência futura. Sucesso gera uma revisão/undo e zera runtime/pistas; um undo/redo restaura cenas e PNGs exatos. Save/load e replay completo/prefixo passaram; hero se move e emite passos pelo mesmo controle. Falhas preservam sessão e bytes de replay. Captura batch-combined.png inspecionada; marcador verde separado da região invisível, NPC/patrulha e jogador presentes.

A execução inicial da regressão de volumes encontrou input zerado por perda de foco, corrigido separadamente em EAF-11; não era falha da atomicidade ou cenário isolado. Dados atuais em scene-edits-verification.json e batch.replay.json, substituídos no mesmo caminho. macOS/Metal somente, sem publicação. Aceite humano da cena combinada permanece separado.

## EAF-11 — correção da perda de foco durante controle MCP (23/10/2026)

Regressão encontrada ao validar EAF-17: verify_triggers.py --window falhou duas vezes depois de run_scenario isolado, sem alteração de cena/tick; diff mostrou somente input de walk/x1/z0 para walk/x0/z0. WindowEvent::Focused(false) enviava SetInputDefault incondicionalmente, mesmo sem teclado ter controlado a sessão. A operação isolada estava correta; o evento nativo concorrente mudava a entrada.

Engine agora conta revisões internas de entrada válida/reset, e KeyboardControl registra a revisão sob o mesmo lock. Foco só libera a revisão que ainda pertence ao teclado; MCP posterior prevalece mesmo com valores idênticos. Repetições sem mudança e releases tardios após limpar keys não reenviam estado. Nenhum formato/replay/snapshot ou hot loop mudou. Checklist antes/depois documentado.

Teste de handler cobre ausência de posse, liberação nativa, MCP igual/diferente, reset e erro de validação. Clippy/build e84 testes Rust release passaram no checkout que também contém a edição em lote EAF-17. A regressão com janela voltou a passar; suíte MCP completa78 grupos aprovada nesse mesmo checkout. O commit desta correção é separado do lote; teste físico de teclado e aceite humano continuam pendentes no Mac bloqueado.

## EAF-16 — volumes de gameplay e visibilidade (23/10/2026)

Componente trigger/visible em cena/replay10 e predicados de cenário2. Volumes fixos sem corpo, alvos por ID, enter/exit pós-física, máscara e ring numéricos limitados; inspect_triggers e snapshot/replay completos. Eventos não controlam NPC/quests. ADR0010 e checklist antes/depois registram AABB discreto, precisão f32, seleção explícita e futuro sweep.

78 testes Rust release, clippy -D warnings e build aprovados. Sete testes novos cobrem passagem com trajetória idêntica à ausência de sensor, fronteiras/altura/estado inicial/stay, seleção/desativação, reset/undo, limites/referências/migração, visibilidade, replay/prefixo/adulteração e cenário2. Teste máximo:32 volumes×32 alvos×20 transições=20.480eventos;512retidos,19.968descartados, capacidade do anel constante e32 consultas físicas/tick. Ordenação do documento e seleção invertidas produziram estados/eventos iguais. Teste de fronteira foi ajustado para dimensões exatamente representáveis em f32; expectativa de versão inválida do cenário atualizada para3 após suporte a2. Nenhuma lógica alterada para forçar esses testes.

75 grupos MCP passaram (71 anteriores + quatro de volumes), todos exceto laboratório com janela. Exemplo triggers.scene.json criado por upsert/save MCP: região invisível, marcador verde e corpo não selecionado dentro. Nenhum evento para esse corpo; jogador entrou no tick55 e saiu96, sem stay repetido. Cenário2 isolado verificou passagem em66ticks e preservou sessão viva. Replay e prefixo confirmaram ocupação no primeiro enter. Visibilidade/undo/load restauraram PNGs exatos; parede oculta continuou bloqueando movimento. Captura triggers-entered.png inspecionada, coerente com região/marcador e jogador. Artefatos em triggers-verification.json/triggers-report.json/triggers.replay.json. Cenas legadas mantêm suas versões originais para migração.

Implementação/validação local separadas do aceite humano, ainda pendente. macOS/Metal somente; sem publicação ou profiling. A região lógica não é regra de vitória nem quest pronta.

## EAF-15 — observação de replay por tick (23/10/2026)

inspect_replay/capture_replay compartilham reconstrução com run_replay; sem mudar cena/replay9. Estado isolado, ruídos do tick selecionado pendentes, final_verification somente no final, câmera gravada/guias da sessão. Renderer/cache compartilhado entre captura viva/replay, caminhos protegidos contra sobreposição da origem. Checklist antes/depois e contrato de estado de controle não gravado documentados.

71 testes Rust release, clippy -D warnings e build passaram. Cinco testes novos cobrem prefixo/zero/final, eventos pendentes e passos, validação de sufixo/limites/build, reexportação válida, preservação de sessão em play, final adulterado, aliases/viewport e replay vazio. Um import incorreto do enum de movimento no teste foi corrigido; nenhum comportamento de produção alterado para acomodar expectativa.

71 grupos MCP passaram (67 anteriores + quatro de replay), todos exceto laboratório com janela. PNGs reconstruídos nos ticks0/30/31/50 idênticos byte a byte aos originais, inclusive ruídos pendentes1/1/0/1. Partida viva em outra cena/tick90 e seu replay preservados. Alternância entre captura viva/replay, resize e guias restaura pixels exatos. Final adulterado informa matched=false mas permite capturar o estado real; prefixo informa null. Captura replay-selected.png inspecionada: visão amarela de memória, pistas laranja, alcance roxo, caminho branco e patrulha lilás coerentes. Relatório artifacts/replay-inspection-verification.json guarda hashes e build. Sem publicação ou aceite humano implícito; macOS/Metal validado.

## EAF-14 — cenários de verificação pelo MCP (23/10/2026)

Documento v1 limitado a 256 ações/3.600 ticks, predicados tipados, save/inspect/run pelo MCP e execução isolada desde autoria. Primeiro erro/timeout encerra com índice/tick/esperado/real e snapshot final; relatório e replay opcionais. Cena/replay continuam v9. ADR 0009 e revisão estática antes/depois documentam sincronismo/lock/custos e limites.

66 testes Rust release, clippy -D warnings e build aprovados. Sete testes novos cobrem repetição, condição inicial sem tick, último tick inclusivo, sequência de patrulha, falha/timeout/limite de ruídos, orçamento/referências/NaN, preservação de partida viva e arquivo, exportação/replay e aliases de paths. A suíte revelou diferença de representação f32 entre relatório salvo/devolvido; corrigida convertendo o relatório uma única vez para Value, compartilhado por ambos. Expectativa de tick da retomada foi corrigida de 921 (decisão pré-física) para 922 (snapshot observado), coerente com a evidência MCP anterior.

67 grupos MCP aprovados, todos exceto laboratório com janela. `verify_scenarios.py` criou os três documentos versionados pelo MCP: quiet.scenario.json passou em 50 ticks/quatro checks; expected-failure.scenario.json falhou intencionalmente na ação3/tick50, source esperado sound e real null; timeout separado parou no tick3. patrol.scenario.json verificou interrupção/retomada/chegada em 1.045 ticks/seis checks, retomada no snapshot922. Runs repetidos geraram relatórios idênticos; arquivo salvo e resposta iguais; replays exportados passaram. Estado/replay da partida viva em tick90 permaneceu idêntico. Outputs inválidos não alteraram arquivo de entrada nem sessão.

Relatórios correntes em artifacts/scenarios-verification.json, quiet-report.json, expected-failure-report.json e patrol-report.json. Teste falho é resultado da ferramenta (isError=false, report.passed=false), não erro de transporte. Runner não inicializa GPU nem executa scripts de gameplay; macOS é único ambiente validado, sem publicação.

## EAF-13 — guias de caminhos e patrulha (23/10/2026)

Patch de sessão npc_routes/tecla J; caminho calculado branco, destino conhecido branco/vermelho e rota autorada lilás tracejada. Limite de 256 segmentos de path por NPC com contador de truncamento em capture; cache/buffer/draw compartilhados. Sem alteração de cena/replay 9 ou comportamento. Checklist antes/depois documentado.

59 testes Rust release, clippy -D warnings e build passaram. Dois testes novos cobrem máximo de oito NPCs/16 pontos/256 segmentos, contador de corte, ausência de influência do alvo oculto, invariância de replay, capacidade retida e mudança de estilo/cor/quantidade do cache. 63 grupos MCP aprovados: rotas 4 + os 59 grupos anteriores. Um teste acústico que assumia que todas as opções existentes eram verdadeiras foi ajustado para verificar explicitamente suas três guias e que npc_routes permanece false; nenhum comportamento de runtime mudou para acomodar esse teste.

Capturas routes-investigation.png e routes-unreachable.png inspecionadas: caminho/destino da pista separados do circuito autorado, e cruz vermelha sobre a região bloqueada. MCP verificou patch/idempotência pausada, bytes do replay idênticos, pixels idênticos no toggle/undo/reset, remoção/desativação, resize, load e composição com visão/audição. Metadados reportam quantidade de linhas e truncamento; overlay nunca interpreta a posição real oculta como objetivo. Teste físico de J e aceite humano pendentes com Mac bloqueado.

## EAF-12 — rotinas de patrulha (23/10/2026)

Componente opcional `npc.patrol` com rota explícita ou círculo procedural, até 16 pontos, loop/ping-pong, espera, prazo por tentativa e cooldown de rota bloqueada. Módulo separado de percepção; somente sem memória fornece objetivo ao navegador. Interrupção por visão/som e retomada ao índice interrompido, sem consulta à posição oculta. Cena/replay 9, inspect_patrols e snapshot/replay com estado próprio. ADR 0008 e revisão estática antes/depois registrados.

57 testes Rust release, clippy -D warnings e build aprovados. Cinco testes novos cobrem ordem/espera, retorno ping-pong, interrupção/retomada, movimentos ocultos independentes, visão prioritária, passos próprios ignorados, bloqueio/prazo/cooldown, seed, migração/erros atômicos, reset/undo, replay e detecção de adulteração. Bitset de pontos falhos impede cooldown prematuro quando ping-pong repete um ponto antes de testar todos.

59 grupos MCP aprovados: patrulha 3, locomoção 3, passos 4, audição 6, guias acústicos 4, guias visuais 4, percepção 7, navegação 8, busca 4, render 5, laboratório 11. Todos exceto laboratório com janela. `verify_patrol.py` cria `examples/patrol.scene.json` pelo MCP: quatro pontos ao redor da barreira, 60 ticks de espera, 1.800 de prazo, 180 de retry, chegada em 0,25 m. Ruído interrompeu no tick 30 e rotina retomou ponto 0 no tick 922, sem memória. Rota circular seed 27 visitou 11 pontos em 600 ticks, sem skips; configuração com dois pontos fora do piso descartou ambos e manteve cooldown sem objetivo de navegação. Replays coincidiram. Captura patrol-resumed.png inspecionada: NPC em retorno com visão azul sem pista. Evidências em artifacts/patrol-*.

Implementação/testes independem do aceite humano; interação física no Mac bloqueado continua pendente. Sem publicação, alteração de navegação, ECS, navmesh ou promessa de escala. Fixtures legadas preservam versões originais.

## EAF-11 — modos de locomoção e furtividade (23/10/2026)

Cena/replay 8, entrada walk/sneak/run e perfis opcionais do jogador. Defaults 4/1,5/6 m/s e multiplicadores de ruído 1/0,2/1,8. Teclas Shift/Control pelo mesmo dispatcher, título com modo e prioridade furtiva. Zero silencia sem acumular passos; raio positivo limitado a [0,1;100] m. Nenhuma mudança de postura, altura ou percepção. ADR 0007 e checklist antes/depois registrados.

52 testes Rust release, clippy -D warnings e build passaram. Novos testes cobrem velocidade física/diagonal, audição oculta por modo, silêncio, limites, perfis/inputs inválidos atômicos, migração, reset/undo, replay e combinação/release de modificadores. 56 grupos MCP passaram: locomoção 3, passos 4, audição 6, guias acústicos 4, guias visuais 4, percepção 7, navegação 8, busca 4, render 5, laboratório 11. Todos exceto laboratório com janela; nenhuma alegação de teste físico de teclado. Mac bloqueado impediu esse teste, que fica pendente com o aceite humano.

`verify_locomotion.py` cria `examples/stealth.scene.json` pelo MCP. Em 20 ticks, caminhada percorreu 1,3214 m com quatro impulsos/raio 8 m e foi ouvida; furtivo percorreu 0,4881 m com um impulso/raio 1,6 m e não foi ouvido; corrida percorreu 1,9881 m com seis impulsos/raio 14,4 m e foi ouvida. Nenhum dos três casos teve visão do jogador. Replays coincidiram, inclusive silêncio explícito. Capturas `locomotion-sneak.png` e `locomotion-run.png` inspecionadas: azul sem pista versus amarelo com memória sonora; círculos nominais diferenciados. Comparações Python usam tolerância para a representação f32 (a primeira igualdade decimal exata foi corrigida no teste). Cenas legadas mantidas nas versões originais como fixtures de migração.

## EAF-10 — passos físicos opt-in (23/10/2026)

Componente de passos implementado em jogador/NPC, fila automática e histórico limitado, própria emissão ignorada pelo ouvinte, inspeção/MCP e replay regenerado. Cena/replay 7. Demo examples/footsteps.scene.json criada e salva pelo MCP: stride0,6m, raio 8 m, min_speed0,1m/s, piso floor. Cenários anteriores sem componente continuam silenciosos.

47 testes Rust release, clippy -D warnings e build aprovados. Seis novos testes cobrem parede, ar, silêncio legado, anel saturado, replay, configuração inválida, reset/undo, próxima amostra de audição, memória congelada e dois NPCs. 53 grupos MCP aprovados: passos 4, audição 6, guias acústicos 4, guias visão 4, percepção 7, navegação 8, busca 4, render 5, laboratório 11; todos exceto laboratório com janela. Verificação de migração foi atualizada para versão7; no restante, comportamentos antigos passaram. Relatórios atuais em artifacts/, sem slow_requests. Captura footsteps-heard.png inspecionada: jogador oculto, memória sonora amarela, pista laranja na emissão e círculo nominal roxo. Testes de scripts versionados legados permanecem disponíveis.

Sem som no dispositivo, animação de pés ou mudança implícita em cenas antigas. ADR 0006 e checklist final documentam memória limitada, replay regenerado, autoexclusão e compromisso do piso fixo. Integração por fast-forward no checkout principal concluída. Build corrente2f5355b132cdae16, app/target por hardlink (inode8240739, 7.031.920bytes), aberto em examples/footsteps.scene.json, pausado/tick0 com as três guias ligadas por MCP. O teste físico de H foi impedido pelo Mac bloqueado; teste de comando/captura passou e aceite humano permanece pendente. Processo anterior encerrou pelo MCP; nenhuma cópia histórica ou publicação.

## EAF-9 — alcance auditivo e ruídos (23/10/2026)

Implementado em checkout isolado de main30528ae para preservar trabalho de identidade/site. Patch de visualização com npc_hearing/noise_events, teclaH no dispatcher, círculo nominal roxo por ouvinte e laranja por impulso congelado. Até16 impulsos com idade<90ticks, sem alterações em simulação/formatos. Emissão pausada invalida a janela; preferências persistem em reset/load.

41 testes Rust release, clippy -D warnings e build passaram. 37 grupos MCP: audição visual4, render 5, visão visual4, audição 6, percepção 7, laboratório 11; todos exceto laboratório com janela, slow_requests vazios. Captura acoustic-emission.png inspecionada: raio nominal do NPC e raio menor da fonte distinguíveis. Testes cobrem limites, expiração, buffer, toggle/patch, snapshot/replay inalterados, pausado, resize, estados surdo/desativado/onisciente, reset/load/remove/undo. TeclaH ligada ao mesmo comando; teste físico e aceite humano ainda pendentes. Build/artefatos atuais no worktree hearing-and-footsteps, sem publicação. Integração da demonstração corrente junto ao próximo incremento de passos.

## EAF-6 — correção de investigação junto à parede (23/10/2026)

Relato humano: N torna o guia amarelo por pouco tempo, depois azul, sem movimento. Reproduzido na sessão original via MCP: hunter (3.3489742, 0.79893064, 0.43535045), yaw 81.55771; player (2.2461648, 0.7989311, 0.6489972). Ruído recebido em tick35369; seis passos depois, `search_points_exhausted`, quatro pontos descartados e distância zero. Audição funcionava; origem do NPC estava dentro da margem expandida da parede, invalidando todo caminho. Após corrigir essa saída, apareceu também falso destino inalcançável por ausência de célula dentro da região de chegada de 40 cm junto à parede.

Correção: saída local de contato por velocidade física, direções limitadas e segmentos seguros, sem teleporte; estado `recovering/leaving_contact_margin`. Grade conecta o nó final à região contínua por segmento varrido de até uma célula. Nenhuma mudança em sentidos, memória, perfis, cenas/replay ou controles. Checklist de performance antes/depois registrado.

39 testes Rust release, clippy -D warnings e build release aprovados. 36 grupos MCP: audição 6, navegação 8, percepção 7, busca 4 e laboratório 11; todos exceto laboratório com janela; slow_requests vazios. Nova regressão usa as coordenadas do relato, confirma audição fora do FOV, recuperação, ausência de esquecimento prematuro/skip, colisão preservada e aquisição visual em até120ticks. Estado final tick122: hunter (3.463513,0.7989314,0.6189532), visão real, navegação arrived dentro do stop_distance1,2m. Replay exato passou. Teste geométrico cobre quatro faces, canto com dois contatos, parede fina que não pode ser atravessada, sobreposição profunda e limite do piso.

Evidências: `artifacts/hearing-verification.json`, `hearing-contact.png`, `hearing-contact-acquired.png` (inspecionada), `hearing-contact.replay.json`. Cena do caso salva pelo MCP em `artifacts/hearing-contact.scene.json`. Build corrente c8414a00db0ce7ef; app anterior encerrado normalmente via MCP, launcher/target compartilham inode8214190 e6.914.032bytes; sem cópia histórica/publicação. App corrigido aberto pausado na cena do caso, guias ligados, para repetir P/N; R restaura as posições do caso. Aceite humano permanece pendente em EAF-6.

## EAF-7 — guias visuais de visão (23/10/2026)

Relato: durante a demonstração de audição, não era possível saber para onde o NPC olhava e distinguir falta de visão de problema lógico. A caixa não girava visualmente. Implementado overlay opt-in com seta de orientação atual, contorno tracejado nominal e cor por detecção/memória, sem modificar percepção ou concluir que o caso relatado era bug. Guias controladas por `set_debug_view` e tecla V; estado separado de cena/replay.

37 testes Rust release aprovados (2 cache/geometria, 11 contratos, 8 navegação, 10 percepção, 6 audição), clippy -D warnings e build release aprovados. Os dois novos cobrem sinal dos ângulos, geometria/cache, limite de vértices, remoção/reativação, estados visuais e igualdade do replay com on/off. 32 grupos MCP nesta alteração: 4 guias, 5 render, 11 laboratório, 7 percepção e 5 audição. Todos exceto laboratório com janela; slow_requests vazios. Guias: capturas repetidas/restauradas idênticas, toggle idempotente sem mutar tick/revisão/documento/partida/replay, yaw ±90, FOV 1/360, remoção/undo, viewport, memória sonora e aquisição visual, reset/load, desativação e onisciência.

Capturas `vision-before.png` e `vision-seeing.png` inspecionadas: seta e campo azul com jogador oculto atrás da parede; verde após aquisição verdadeira na ponta. `vision-memory.png` e `vision-verification.json` completam evidência. Uma verificação nativa adicional pressionou V duas vezes com a janela pausada: guias desapareceram e voltaram, título atualizou e tick continuou zero. Janela deixada com guias ligadas na cena de audição. O contorno atravessa paredes por ser diagnóstico nominal; não promete área efetivamente visível. A seta mostra orientação atual; cor a amostra pré-física, potencialmente um tick anterior.

Build corrente a3ba4b9660322f58, macOS/Metal. Processo anterior encerrado via MCP; launcher e target/release/ignis compartilham inode 8211154, 6.913.888 bytes. Um único build corrente, sem cópias históricas/publicação. Checklist de performance respondido; sem profiling. Aceite humano das guias e diagnóstico do comportamento original ainda separados da implementação e dos testes; EAF-6/EAF-7 permanecem em avaliação.

## EAF-6 — audição por pistas pontuais (23/10/2026)

EAF-5 recebeu aceite humano nesta conversa (“Ficou muito bom ... Vamos pra frente!”) e foi concluída no Kordo. EAF-6 acrescenta audição opt-in, `sound_transmission` independente de física/visão, `emit_noise`/`inspect_noises`, memória de fonte `sound` e prioridade da visão. N usa o mesmo dispatcher; ainda não há reprodução de áudio nem passos automáticos. Cena/replay v6; defaults anteriores preservados. Checklist antes/depois em `performance.md` e tradeoffs na ADR 0005.

35 testes Rust release passaram: 10 contratos, 8 navegação, 10 percepção, 6 audição e 1 cache visual. Clippy com -D warnings e build release passaram. Os testes novos cobrem silêncio/opt-in, limites de alcance do estímulo/listener, transmissão por caixas físicas e visuais, composição de atenuação, prioridade visual, força relativa/desempate, ausência de influência de movimentos ocultos, origem congelada, replays com pendentes/consumidos e adulteração, configuração inválida atômica, migração v5, expiração e aquisição visual. Após ampliar o teste do limite para preencher realmente 4096 eventos na sessão, os seis testes de audição e clippy passaram novamente. Sem mudança de runtime após a validação completa.

40 grupos MCP passaram: laboratório 11, navegação 8, percepção 7, render 5, busca espacial 4 e audição 5. Todos exceto laboratório com janela nativa. Relatórios correntes registram `slow_requests: []`. `verify_hearing.py` constrói/salva `examples/hearing.scene.json` exclusivamente via MCP, valida roundtrip, erro atômico, histórico/reset, bloqueio acústico total, isca anônima, limite de 16 por tick, capturas e replay.

Demonstração: 30 ticks sem conhecimento do jogador atrás da parede; ruído 0 emitido no tick 30 em (0,0.79893064,-2). No tick 46, jogador já havia se movido para X≈0,991, mas a memória sonora permaneceu na origem e `observed_this_tick` continuou false. No tick 208, a visão do tick 207 substituiu a pista por uma observação real e ativou perseguição. Os replays coincidiram. Captura `hearing-acquired.png` inspecionada, além de `hearing-before.png`/`hearing-investigating.png` e relatório `hearing-verification.json`.

Dois ajustes de testes: uma caixa convertida em visual passou a permitir visão por default e precisou de `blocks_vision: true` para isolar atenuação acústica; a comparação Python da isca passou a usar a coordenada f32 canônica devolvida pelo comando. Nenhuma mudança no sensor para forçar esses resultados.

Build corrente `e74e4c30f5e6f6c1`, macOS Apple Silicon/Metal. Processo anterior encerrado normalmente via MCP; `target/release/ignis` e `artifacts/Ignis.app/Contents/MacOS/ignis` compartilham inode 8203904, 6.930.176 bytes, sem cópia histórica. Aplicativo atualizado aberto em `examples/hearing.scene.json`; inspeção MCP confirmou tick zero, pausa, memória vazia e nenhuma pista pendente. Atalho N ligado ao comando compartilhado por inspeção de código; entrada física dessa tecla e aceite humano da nova funcionalidade ainda não registrados. Aviso transitivo de incompatibilidade futura de `block 0.1.6` permanece.

Limites: segmento/raio com atenuação simples e localização exata, sem difração ou áudio reproduzido; um único alvo visual, mapa conhecido e piso plano. Pista sonora pode ser isca; memória não identifica emissor. Nenhum profiling/FPS medido, teste de escala máxima, publicação ou validação em outra plataforma.

## EAF-5 — investigação espacial (23/10/2026)

Implementado perfil opcional `area_search` com raio, até oito pontos, varredura por ponto e semente; pontos gerados da memória, sem acesso à posição oculta. Fase `exploring`, ponto hipotético e contadores separados da observação verdadeira. Descarte de destinos inalcançáveis, esgotamento da lista, interrupção por visão nova e budgets globais. Cena/replay v5; cenas v1–v4 legíveis e perfil anterior preservado. Checklist estático antes/depois respondido em `performance.md`; sem dependências novas ou profiling.

29 testes Rust release passaram (10 contratos, 8 navegação, 10 percepção e 1 cache visual), clippy -D warnings e build release passaram. Os três testes novos abrangem visitas/skip, replay e adulteração, configuração inválida atômica, migração v4, seeds, memória congelada sob movimentos ocultos distintos, redetecção e limites durante exploração. O teste de privacidade foi ampliado para simular região inicial inalcançável e passou novamente, junto ao clippy, após a ampliação.

35 grupos MCP passaram: 11 laboratório, 8 navegação, 7 percepção, 5 render e 4 busca espacial; todos exceto laboratório executaram com janela. `verify_search.py` constrói/salva o exemplo exclusivamente por MCP, testa save/load, invalid config, undo/redo, reset e replay. Com raio 2 m, os quatro pontos foram visitados e houve desistência registrada no tick 850, após 12,4545 m de investigação; raio 20 m gerou quatro pontos inalcançáveis, descartados, com desistência registrada no tick 250. Relatórios correntes em `artifacts/*-verification.json`, sem slow_requests.

Cena da parede: jogador inicia em (-5, 0.85, 4), NPC em (-2, 0.85, 4). Roteiro: um tick inicial, W por 100 ticks e D por 52 ticks, então parar. Última observação antes da fuga: tick 67, posição (-5, 0.79893, -0.35791). No snapshot do tick 219, `exploring` tinha destino hipotético (-5, 0.79893, -3.85791), mantendo intacta a memória do tick 67. No snapshot 231, a observação visual do tick 230 permitiu redetecção e reiniciou os contadores. Replays durante exploração e após redetecção coincidiram. Captura `search-reacquired.png` inspecionada; `search-before.png`, `search-exploring.png` e `search-verification.json` complementam a evidência.

O roteiro inicial de demonstração batia na parede e não perdia visão; foi corrigido para contornar a ponta. Variantes de posição/varredura também mostraram busca que falha ou redetecta antes de explorar, comportamento coerente com alcance/FOV; o código não foi alterado para forçar sucesso. O roteiro final é um caso reproduzível, não garantia de reencontro em qualquer fuga.

Build corrente `1d6caa34c03ad778`, macOS/Metal; launcher e `target/release/ignis` compartilham inode, 6.861.968 bytes. App anterior encerrado via MCP e substituído pelo build atual, sem cópia histórica. Nova sessão aberta com `examples/search.scene.json`, inspect confirmou pausada no tick zero, memória/ponto vazios e perfil configurado. Velocidade 1,5 m/s, visão 4 m/FOV120, memória 1800 ticks, busca1200, distância40 m, raio3,5 m, quatro pontos, scan120 ticks e seed0. Aceite humano desta nova demonstração permanece pendente, separado da validação automatizada. Nenhuma publicação.

## EAF-4 — revisão estática de performance (23/10/2026)

Checklist obrigatório criado em `docs/performance.md`, vinculado por `AGENTS.md` e README: aplicar antes de codificar cada incremento e responder sobre o diff final. Profiling/benchmark não são requisitos nesta fase, conforme decisão de Guilherme. A seção final do checklist registra cada correção e custo conscientemente mantido; ADR 0004 foi reconciliada com essa decisão.

Correções: snapshot físico e mapas de objetivos/velocidades reutilizados, capacidade de strings de estado preservada, scratch de A* reutilizado, seleção mínima sem alocação/ordenação de candidatos, contatos numéricos internos em buffers reutilizados, frame visual compacto, vértices/buffer GPU persistentes com invalidação por conteúdo e upload da faixa alterada. Janela pausada inalterada não solicita frames; mantém consulta MCP a cada 16 ms e atende redraw do SO/resize. Título só muda quando seu estado muda, fora do lock. Protocolo/schema e montagem textual/base64 da resposta MCP ficam fora do lock do serviço. Sem dependências, formatos de arquivo ou algoritmos de gameplay novos.

26 testes Rust release passaram (25 regressões existentes + invalidação/reutilização do cache de geometria); clippy release com `-D warnings` e build release passaram. MCP real: 31 grupos passaram (11 laboratório, 8 navegação, 7 percepção, 5 renderização). Os três últimos executaram com janela. `verify_render.py` compara PNGs exatos: repetição, cor/tamanho/posição com undo, crescimento do buffer, remoção sem geometria residual, movimento/reset, câmera e troca de viewport. Teste CPU cobre também reordenação e cena visual vazia. Replays, limites de investigação, colisões e ausência de vazamento de conhecimento continuaram passando.

As quatro capturas correntes `perception-before.png`, `npc-before.png`, `lab.png` e `collision.png` tiveram SHA-256 idêntico ao registrado antes desta alteração. Captura de percepção inspecionada visualmente. Relatórios em `artifacts/{verification,npc-verification,perception-verification,render-verification}.json`, todos com `slow_requests: []`. Isso é evidência funcional/visual, não profiling ou promessa de ganho de FPS. Aviso transitivo de incompatibilidade futura de `block 0.1.6` permanece.

Aplicativo anterior encerrado normalmente via MCP. Build corrente `85c58788f280c66f`, macOS/Metal, com `target/release/ignis` e `artifacts/Ignis.app/Contents/MacOS/ignis` compartilhando inode (6.844.816 bytes). Nova sessão com janela recebeu play/pause/reset pelo MCP e ficou pausada no tick zero; sem cópias históricas do app. Não houve publicação. Custos remanescentes documentados: mapas por strings/scans limitados, coleta de diagnóstico e operações síncronas de captura/persistência; nenhuma alegação de otimização completa da engine.

## EAF-3 — limites opcionais de tempo e distância

Extensão validada localmente em macOS Apple Silicon/Metal. `memory_ticks` e `search_ticks` podem ser desativados; `max_investigation_distance` permite distância isolada ou combinada com tempo. Primeiro limite ativo encerra; pelo menos um é obrigatório. Distância acumula deslocamento XZ após física durante investigação/busca, com tolerância de 1 mm no encerramento, sem arredondar o diagnóstico. Cena e replay v4; cenas v1–v3 preservadas por migração. Configuração/inspeção/save/load pelo MCP, sem novas dependências.

25 testes Rust release passaram (10 contratos, 8 navegação, 7 percepção), assim como clippy release com `-D warnings` e build release. Os novos testes cobrem distância percorrida versus deslocamento líquido, busca parada, limites opcionais inválidos sem mutação, migração v3, ordem de esgotamento, redetecção/reset, replay e adulteração do contador. Um teste detectou que o movimento limitado ao pequeno saldo final podia ser anulado pelo atrito; a tolerância de 1 mm evita esse estado sem consumir distância fictícia. Os testes anteriores de privacidade do conhecimento, colisões e histórico continuam passando.

MCP real: 26 grupos passaram (11 laboratório, 8 navegação, 7 percepção); navegação e percepção com janela nativa. Nos dois cenários de limite de 1 m (distância isolada e memória de 600 ticks), `travel_budget_exhausted` ocorreu com 0,99940395 m contabilizados no tick 84. Com memória de 5 ticks e distância de 100 m, venceu `memory_expired`, com 0,06428146 m contabilizados até o tick 25. Configurações passaram por save/load antes da execução; replay verificou o estado final e reset zerou o contador. Relatórios correntes em `artifacts/verification.json`, `npc-verification.json`, `perception-verification.json`; todos registraram `slow_requests: []`. Arquivos de orçamento em `artifacts/perception-budget.scene.json` e `perception-budget.replay.json` são substituídos entre cenários; o relatório retém os três resultados.

Captura corrente `artifacts/perception-before.png` inspecionada. Exemplo interativo salvo pelo MCP em v4, mantendo configuração anterior: velocidade 1 m/s, visão 4 m/FOV 120°, orientação 90°, memória 600 ticks, busca 180 ticks e distância desativada. Aplicativo anterior encerrado por MCP, launcher atualizado e nova sessão aberta/pausada; `inspect` da mesma sessão confirmou tick zero, contagem zero, memória vazia e build `825daa4d5c2a0506`. `artifacts/Ignis.app/Contents/MacOS/ignis` e `target/release/ignis` compartilham inode, 6.911.568 bytes, sem cópia histórica do app. Aceite humano dos novos limites permanece separado.

ADR 0004 registra diagnóstico e proposta sobre autoria, pacotes binários e layout de runtime. Leitura de código identificou alocações/clones e reconstrução de buffers; não houve profiling, benchmark comparativo de codecs, cooker binário nem implementação da proposta de otimização. Nenhuma promessa de ganho de FPS foi validada. O aviso transitivo de incompatibilidade futura de `block 0.1.6` permanece.

Escopo: EAF-1, primeiro laboratório 3D. macOS Apple Silicon, Apple M4 Pro, backend Metal, Rust 1.97.1. Build local otimizado (`release`).

## Atualização de identidade

Nome escolhido: Ignis Engine. Pacote, biblioteca, executável, identidade MCP, janela, empacotamento e documentação atualizados. Testes e clippy executados no perfil release para reutilizar o build corrente. As evidências do teclado abaixo são da implementação inicial; a atualização preserva esses controles.

## Evidências

- `cargo test --locked --release --all-targets`: 10 testes passaram. Cobrem geração por semente/IDs, validação atômica, revisão obsoleta, colisão, contato, isolamento edição/partida, remoção/restauração de parede, replay serializado, persistência, caminhos e handshake MCP.
- `cargo clippy --locked --release --all-targets -- -D warnings`: passou para o código do projeto. Cargo também informa um aviso de compatibilidade futura da dependência transitiva `block 0.1.6`; não impediu o build com o compilador validado.
- Build `release`: concluído.
- `scripts/verify_lab.py --binary target/release/ignis --window`: 11 verificações de integração passaram, conversando pelo protocolo MCP com o processo real e sua janela nativa.
- Capturas offscreen PNG geradas pela GPU Metal e devolvidas como conteúdo de imagem MCP; bytes comparados com o arquivo salvo. Imagem inspecionada: sala, paredes, grade, obstáculos e personagem legíveis.
- Aplicativo local aberto pela interface do macOS. P iniciou a simulação; W movimentou o personagem; P pausou no passo 70; R retornou ao passo zero. Cena deixada aberta e pausada para avaliação humana.

O roteiro de falha executa 300 passos com entrada constante: sem a parede norte, o personagem ultrapassa o limite; após undo, as mesmas entradas param na parede e produzem evento de contato. O replay verifica posições finais idênticas em duas execuções isoladas do mesmo build. Os valores exatos e metadados de captura estão no relatório gerado.

## Onde consultar/reproduzir

- `artifacts/verification.json`: relatório corrente de integração.
- `artifacts/lab.png`: visão geral corrente.
- `artifacts/collision.png`: captura do cenário de colisão corrigido.
- `artifacts/lab.replay.json`: replay do cenário de colisão.
- `artifacts/lab.scene.json`: cena de apresentação gerada ao final do roteiro.
- `artifacts/command-schema.json`: contrato de comandos descoberto via MCP.
- `artifacts/runtime.log`: diagnóstico do processo do verificador.
- `examples/lab.scene.json`: cena de referência versionada.

Artefatos gerados são substituídos no mesmo caminho. O build corrente é `target/release/ignis`; o launcher `artifacts/Ignis.app` usa um hardlink para esse mesmo binário. O build de desenvolvimento usado nos testes foi removido após a validação da substituição. O estado incremental necessário ao build release permanece em `target/release`.

## Verificação da renomeação

Os dez testes e clippy passaram no perfil release; o roteiro MCP com janela passou nas onze verificações. `Ignis.app` abriu no macOS com título Ignis Engine, cena visível e simulação pausada. O pacote antigo e os outputs próprios do nome anterior foram removidos após a substituição; o novo app e `target/release/ignis` compartilham o mesmo inode.

A primeira tentativa de integração excedeu o timeout de inicialização MCP (45 s); uma resposta chegou atrasada durante o fechamento e o processo terminou posteriormente. A nova execução passou, seguida de abertura normal do aplicativo. A causa da demora não foi confirmada; não atribuir a correção à renomeação nem considerar o episódio resolvido de forma geral.

## Limite da conclusão

Implementação e verificações locais concluídas para este recorte. Aceite humano pendente. Não foi feita publicação, upload ou validação em Windows/Linux/mobile/web. O laboratório não é um editor completo nem conclui os milestones abrangentes do roadmap.


## EAF-2 — NPC perseguidor (22/10/2026)

Implementado no mesmo ambiente macOS Apple Silicon / Apple M4 Pro / Metal. Configuração por `upsert`, inspeção por `inspect_navigation` e `inspect`, execução por `step`/`play`. Toda a construção e verificação automatizada da demonstração usou o protocolo MCP real.

- `cargo test --locked --release --all-targets`: **16 testes passaram** (10 contratos anteriores e 6 de navegação). Desvio com volume do corpo, bloqueio total/undo, passagem estreita, alvo móvel, desativação, referências inválidas atômicas, migração de cena v1, independência de ordem, alvo fora do piso e replay serializado.
- `cargo clippy --locked --release --all-targets -- -D warnings`: passou. Permanece o aviso transitivo de compatibilidade futura de `block 0.1.6`.
- `cargo build --locked --release`: passou.
- `scripts/verify_lab.py --binary target/release/ignis`: **11 verificações MCP passaram**, incluindo captura offscreen e replay no formato atualizado.
- `scripts/verify_npc.py --binary target/release/ignis --window`: **7 grupos de verificações MCP passaram** com janela nativa: descoberta; erros atômicos; desvio/folga/chegada; alvo móvel; replay exato duas vezes; bloqueio/undo/redo; persistência/desativação/remoção.
- O NPC contornou uma barreira de 6 m, com desvio lateral máximo observado de aproximadamente **3,554 m**. Ao passo 540 estava em `arrived`, sem modificar a posição de edição. A comparação de replay inclui também estado e caminho de navegação.
- Capturas antes, durante e após a perseguição foram retornadas pelo MCP; a imagem foi inspecionada. Na janela nativa, P iniciou, D deslocou o jogador, P pausou e R retornou ao início. A demonstração foi deixada aberta e pausada no passo zero.

Artefatos correntes: `artifacts/npc-verification.json`, `npc-before.png`, `npc-detour.png`, `npc-arrived.png`, `npc.replay.json`, `npc.scene.json`. Cena versionada reproduzível: `examples/npc.scene.json`, obtida do save pelo MCP. README contém os comandos de execução. Os relatórios dos dois roteiros são separados; `artifacts/runtime.log` contém a execução mais recente.

Build corrente: `target/release/ignis`; `artifacts/Ignis.app` atualizado para o mesmo inode. Nenhum build de desenvolvimento ou cópia histórica foi criado. O processo anterior foi fechado normalmente antes de abrir a demonstração atual.

Limite do aceite: validação automatizada e inspeção local concluídas; aceite humano desta funcionalidade permanece pendente. Navegação limitada a um piso plano, caixas e grade conservadora; não há teste de carga no limite de 8 NPCs/128 corpos nem garantia de multidões/rampas/múltiplos andares. Sem publicação ou validação de outras plataformas.


## Correção de EAF-2 — jogador encostado na parede (22/10/2026)

Relato do usuário reproduzido pelo MCP no build anterior: jogador pressionado contra a barreira em Z≈0,6499, NPC parado em Z=5, status `unreachable/no_clear_path`, embora ambos estivessem do mesmo lado. Evidência anterior em `artifacts/npc-wall-before.json`. A regressão adicionada em Rust falhou antes da correção.

Causa: o destino era o centro do jogador; o teste de caminho expandia os obstáculos pelo corpo do NPC + folga e rejeitava esse centro próximo à parede. A mesma checagem era usada indevidamente para visibilidade e chegada.

Correção: procurar uma posição livre na região de parada ao redor do alvo, com visibilidade pelas caixas físicas sem expansão. O caminho até essa posição continua exigindo folga para o corpo inteiro. A* aceita múltiplas células de chegada; manter a parede entre os dois impede chegada falsa. API MCP e formatos persistidos permanecem iguais; o replay continua exigindo o mesmo build.

Dezoito testes release passaram, incluindo jogador junto à parede central, lateral e canto, dois tamanhos de NPC e alvo próximo do outro lado da parede. Barreira contornável exige desvio; barreira fechando a sala continua inacessível. Clippy release e build release passaram.


Validação final da correção: 11 verificações MCP do laboratório e 8 grupos MCP de NPCs passaram (este último com janela). O novo cenário mantém entrada em direção à parede, confirma contato físico jogador/barreira e verifica chegada do NPC do mesmo lado e após contornar pelo lado oposto; replay correspondente também passou. Evidências correntes em `artifacts/npc-verification.json` (`wall_contact`), `npc-wall.replay.json` e `npc-wall-contact.png`.

Houve timeout de 45 s na primeira inicialização e, na tentativa seguinte, durante uma captura. Na segunda ocorrência, a amostragem do processo mostrou espera em `fcntl` e um arquivo temporário aberto em artifacts/ durante a gravação; a resposta de captura chegou posteriormente. O verificador agora dá 120 s para inicialização e operações de arquivo/captura, mantendo 45 s nas demais, e registra respostas acima de 45 s em `slow_requests`. Não foi reduzida a garantia de persistência da engine. A execução final passou; isso não estabelece uma correção geral de latência de disco ou inicialização.

Aplicativo recomposto por hardlink do executável corrente e reaberto com `examples/npc.scene.json`. Inspeção nativa: P iniciou, W por 1,4 s levou o jogador à parede; o NPC contornou e aproximou-se; P pausou no passo 671. A janela ficou aberta nesse estado. Um build release corrente, sem cópias históricas; aceite do usuário da correção ainda separado.

## EAF-3 — percepção visual e memória (22/10/2026)

Implementado o primeiro recorte da ADR 0003: sensor visual, observação memorizada, investigação da última posição, varredura limitada, desistência e redetecção, com onisciência explícita. Sem dependências novas. Configuração pelo MCP em `upsert`; diagnóstico em `inspect_npcs`/`inspect`; navegação recebe `NavGoal`. Cena v3 migra NPCs v2 para onisciência e resolve NPCs novos sem política para percepção limitada. Replay v3 compara conhecimento além de posição/navegação.

Validação final local: 22 testes Rust release passaram (10 contratos, 8 navegação, 4 percepção); clippy release com warnings como erros e build release passaram. Os testes novos cobrem alcance/FOV/oclusão 3D, caixa baixa, oclusão independente de colisão, movimentos ocultos distintos com memória/decisão/rotas idênticas, memória expirada, busca/desistência/redetecção, migração/defaults, edição inválida atômica, histórico, reset, serialização e adulteração do estado esperado de conhecimento no replay.

MCP real: 11 grupos do laboratório, 8 de navegação e 6 de percepção passaram. Navegação e percepção executadas com janela nativa; capturas GPU Metal inspecionadas. No cenário de fuga por entrada de jogador, a última observação foi no tick 20 (X≈-0,7474, Z=4). No tick 136 o NPC investigava essa posição enquanto o jogador já estava distante; a amostragem registrou busca no tick 244 e desistência no 304, por orçamento de busca. A memória permaneceu igual à observação do tick 20 até ser descartada. Retorno do jogador gerou nova aquisição. Replays durante busca, após desistência e após redetecção passaram.

Evidências correntes: `artifacts/perception-verification.json`, `perception-before.png`, `perception-searching.png`, `perception-lost.png`, `perception.scene.json` e replays `perception-search.replay.json`, `perception.replay.json`, `perception-reacquired.replay.json`. Os três relatórios de integração registraram `slow_requests: []`. Cena interativa `examples/perception.scene.json` construída/salva exclusivamente via MCP; mantém barreira, FOV de 120° e busca de 180 ticks. A janela final foi inspecionada, e a mesma sessão respondeu por MCP: fuga levou a `lost/search_budget_exhausted` no tick 556; reset confirmou memória vazia, orientação inicial de 90° e tick zero. Deixada aberta e pausada para o usuário.

Build corrente `target/release/ignis` (~6,9 MB); `artifacts/Ignis.app` recomposto por hardlink, sem cópia histórica. O visualizador anterior foi encerrado normalmente. Houve espera prolongada durante clippy: amostragem do processo auxiliar encontrou somente `_dyld_start`, antes da execução do programa; concluiu depois sem intervenção. Isso não é evidência de correção do carregador ou do volume externo. O aviso de incompatibilidade futura da dependência transitiva `block 0.1.6` permanece.

Limites: sensor para o centro do alvo, FOV horizontal, caixas sem rotação, orientação sensorial sem rotação visual, mapa conhecido, busca local por giro sem explorar rotas alternativas, um alvo/jogador e ausência de audição/patrulha. Sem teste de escala máxima, aceite humano da nova funcionalidade ou validação fora de macOS/Metal. M09 permanece parcial. Nenhuma publicação/push.


## EAF-27 — locomoção espacial por capacidades

Build `8d38f2c26df1576a`, macOS Apple Silicon/Metal (Apple M4 Pro), cena/replay19.162 testes Rust release passaram (9novos), cargo fmt/clippy -D warnings/build passaram.26 scripts MCP,115 grupos com resultados aprovados; todos exceto laboratório com janela. Uma primeira execução do último script verify_vision_guides falhou na igualdade de PNG após undo (linha100 anterior), duas repetições no mesmo build passaram; causa não determinada. A falha foi preservada em `earlier_attempt` de artifacts/mcp-regression.json; assert ganhou baseline/current para diagnóstico futuro, sem alterar comparação nem alegar correção. Warning antigo de compatibilidade futura block0.1.6 permanece.

Novos testes Rust: quatro motores com chegada XYZ física e limites de deslocamento por tick/replay; ausência de capacidade, obstrução estática/planeio ascendente; rota por altura intermediária/ordem de entidades; atomicidade/reset/undo/versão; salto termina após expiração de pista sonora; corpo dinâmico mantém colisão/timeout; orçamento XYZ e recusa de link caro; políticas planares incompatíveis/destino fora da rede; voo pousa antes de salto. Asserções de privacidade gerais continuam cobertas pelas regressões de percepção/alvos; o motor espacial não recebe posições de alvos ocultos.

MCP criou e salvou `examples/npc-mobility.scene.json`: roxo jumper salta duas plataformas (impulso7m/s, velocidade horizontal limitada4m/s); azul flyer sobe5m, cruza sobre parede e pousa (3m/s); verde glider começa no ar junto à plataforma alta (6m) e desce1m/s até o chão com velocidade horizontal máxima3m/s; cinza walker sem capacidade de salto fica parado com no_compatible_route. Player laranja move-se em WASD, sem emissão de passos neste cenário. Todos os NPCs usam guard_3d fixo; não é teste de perseguição. P inicia, R reset, J guias. A cena completa termina em cerca de7s de simulação; reset permite repetir.

Captura real inspecionada em `artifacts/mobility-airborne.png`, aos40ticks: salto/arco, voo em subida, planeio e guia de rede com alturas distintas. PNG do replay idêntico. Camera ajustada para leitura dos atores e testes MCP repetidos com a cena final. Relatórios: mobility-verification.json, mobility.replay.json, mobility-airborne.replay.json, mobility-airborne-replay.png, mobility-no-guides.png, rust-tests.log, clippy.log, build.log e mcp-regression.json. Exemplos legados restaurados após confirmar que diferenças geradas eram somente versão19 e rede vazia; preservada cobertura de migração.

EAF-26 recebeu aceite humano explícito; EAF-27 implementação e verificações locais prontas, aceite humano próprio permanece pendente. Limites de rede, combinações planares, orientação e planeio terminal documentados na arquitetura. M09 permanece parcial.

Pacote corrente `artifacts/Ignis.app` substituído por hardlink do binário release (8.472.384bytes, mesmo inode). Instância anterior encerrada por MCP exit; nova instância aberta com npc-mobility, confirmada pelo MCP emtick0/pausada/build8d38f2c26df1576a, jumper selecionado e guias ativos. Mantido somente output release corrente; diretório debug parcial de72MiB criado por uma invocação de check sem release foi interrompido, verificado ignorado/sem arquivos abertos e removido. Sem novo worktree, cópia histórica, push ou publicação.

## EAF-81 — espada por perfis e seções (26/10/2026)

Fonte: worktree native-plants, branch codex/native-plants, após integrar a
subdivisão estável98c5d09 como5a5738e. Main e árvore paralela preservadas.
macOS Apple Silicon/Apple M4 Pro, Metal, cena51/replay55,
build `aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|633e2bf7fe7d72c0`.

- `cargo test --locked --release --all-targets`:702 testes/58 grupos, zero falhas
  (`artifacts/rust-tests.log`). Inclui4 testes de seções e1 de domínio explícito.
- `cargo clippy --locked --release --all-targets -- -D warnings`, `cargo fmt --all --check`,
  `cargo build --locked --release` e `git diff --check`: aprovados.
- `python3 scripts/run_verifications.py`:62 cenários/254 grupos headless aprovados,
  incluindo render/replay e novo plant_sections. Relatório corrente
  `artifacts/mcp-validation/report.json`; todos os197 JSONs com build_id do
  workspace isolado referem-se ao mesmo build final. Uma execução intermediária
  que cruzou a troca de binário durante compilação foi substituída por esta rodada.
- Espada nativa com4 plantas: lâmina/guarda/cabo/pomo; escultura vetorial,3 ossos,
  clipe flex de120ticks. Lâmina original430v, relevo máximo0.009558059m,
  erro material máximo2.6491e-8, erro amostrado de reconstrução0. Alteração do
  comprimento3.2→4m, largura0.58→0.82m, profundidade0.14→0.18m e estações movidas.
  Fonte canônica/pesos/rotações preservados; repouso e translações de keys adaptados.
  Volta A-B-A exata, preview sem mutação, erro de domínio/colapso/loss/budgets/revisão
  sem alteração parcial. Subdivisão/retesselação/persistência cobertas.
- PNGs sword-original/redrawn/flex inspecionados; repetição, replay e prefixo
  byte a byte iguais às respectivas capturas vivas. Cenas autocontidas em
  examples/native-sword.scene.json e examples/native-sword-redrawn.scene.json.
- Build corrente: target/release/ignis,11.786.560bytes. Nenhum build histórico novo,
  push, merge na main ou alteração da demo existente. Capturas foram offscreen;
  teclado físico, aceite humano e editor gráfico2D não são comprovados pela suíte.

O recorte é loft reto centrado com cortes rectangle/diamond e tampas positivas;
identidade de seções deve permanecer. Não conclui solver CAD, sweep curvo,
remesh livre, frames/twist automáticos, IK ou autointerseção global de escultura.

## EAF-83 — machado composto (26/10/2026)

Apenas conteúdo, script e relatório na codex/native-plants, sobre3a43a3b;
nenhuma mudança de Rust, schema ou renderer. Build corrente633e2bf7fe7d72c0,
cena51/replay55, macOS Apple M4 Pro/Metal. Fonte e restrições em
[native-axe.md](native-axe.md).

`create_native_axe.py` constrói10 plantas/26instâncias via MCP:3052v/4416t
armazenados,7552t desenhados. Curvatura/sulcos feitos por Grab nativo, rig3ossos
e clipe120ticks compartilhados pelas lâminas. Alteração de proporções/estações,
fonte/pesos/rotações preservados, bind/TR adaptados, retorno A-B-A exato,
preview, undo/redo, save/load das duas variantes e replay/prefixo/pixels passaram
nos3grupos próprios. Capturas de frente/detalhe/original/alterada/flexão inspecionadas.

702 testes Rust/58 grupos release/all-targets, Clippy-Dwarnings e fmt aprovados;
regressão proporcional lab/render/replay_inspection/meshes/plants/mesh_topology/
plant_sections:7cenários/37grupos, todos aprovados. Logs correntes
artifacts/axe-authoring.log, axe-regression.log, rust-tests.log e clippy.log;
relatório artifacts/mcp-validation/artifacts/axe-verification.json.

Cenas originais/autocontidas examples/native-axe.scene.json e
examples/native-axe-redrawn.scene.json, aproximadamente2.04MB cada. Janela própria
de teste pode carregar a cena após regressão; isso não comprova teclado físico ou
aceite humano. Main/árvore/demo anterior preservadas, sem push. Normais flat,
uniões sobrepostas e vínculos entre peças explícitos continuam limitações;
curvatura canônica esculpida não é implementação de sweep curvo.

## EAF-85 — primitiva Y conectada: primeiro checkpoint (26/10/2026)

Na codex/native-plants, base d8d2360 equivalente ef90cb7 (shading52/56), kernel
puro em mesh_organic.rs sem Scene/Command/renderer/versões novos. Build21abfc8834e0642b,
macOS Apple M4 Pro/Metal.716 testes Rust/62grupos release/all-targets, Clippy-Dwarnings,
fmt/build/diff aprovados;63cenários/258grupos MCP completos aprovados. Seis testes
próprios cobrem superfície orientada,3fronteiras exatas/Euler-1, refino/volume,
frames/raios/casos recusados, rotação e máximo96 amostras/3refinos.

Prova nativa em examples/organic_junction_probe.rs + scripts/demo_organic_junction.py:
controle3componentes sobrepostos, montagem por mesmos índices1componente fechado,
fronteira0/Euler2. Patch231v/428t,36arestas de bordo antes da montagem;
volume2.328642661→2.312618492 (perda0.688%), deslocamento máximo0.0912m. Três grupos
MCP próprios aprovados (topologia/componentes, undo/redo, pixels vivos/replay iguais).
Capturas before/after inspecionadas: **forma reprovada**, ponte convexa larga e
achatada entre os filhos. Não foi integrada à árvore central nem chamada realista.

organic_tree_case.rs deriva sonda local do primeiro fork da fonte17fb9ed,
attachment_t0.34, divergência22.8815°. Não é recorte exato; configuração próxima
recusa suporte de trunk_above, sem mover ramos ou inventar correspondência.
Integração real/forma/material/rig no interior continuam gates separados.
Logs correntes organic-{all-tests,clippy,build,full-mcp,mcp}.log, prova/relatório
em artifacts/organic-junction e artifacts/organic-tree-case.json. Sem push/main,
sem controle da cena central ou demos em uso. Próxima hipótese de forma documentada
em mesh-organic.md mantém fronteiras e molda interior por campo explícito.

## EAF-85 — forma em campo e consulta nativa (26/10/2026)

Na codex/native-plants sobre8641b61/d8d2360, cena52/replay56 mantidos. Build final
08323cf326d8910f, macOS Apple M4 Pro/Metal; executável corrente11.871.808bytes.
Não foram alterados Scene/renderer/material/formatos nem a cena central da árvore.

`fused_junction` mantém contornos e duas faixas de anéis fixas e encontra o zero
radial de um campo explícito de três tubos afunilados. Envelopes de resíduo,
deslocamento, rotação e volume são validados; nova superfície não promete mapa
de escultura/rig/material. A prova comparativa do mesmo fork passou de tubos
sobrepostos e ponte convexa larga para vale entre os galhos, mantendo índices
compartilhados. Patch267v/500t,72v fixos, perda28.90%, deslocamento0.418m,
rotação73.19°, resíduo2.38e-8. Controle3componentes→montagem1componente fechado,
fronteira0/Euler2. Exporter confere orientação radial e ângulo sólido4π
(grau1.0000000000000007) no caso de referência. Before/topology-only/after/back
foram inspecionados; isso não é aceite de realismo da árvore inteira.

A consulta `preview_mesh_junction` existe no Command/dispatcher/MCP, com input
Box<[Collar;3]>, options/fusion tipados e serde/JsonSchema estritos. Ela não
muta cena/runtime/histórico, respeita expected_revision, publica erro sem
resultado parcial e devolve Geometry/boundaries/diagnósticos. put_asset continua
a publicação explícita. A consulta não mudou formatos nem criou UI exclusiva.

720 testes Rust/63grupos release/all-targets aprovados, incluindo10 testes
orgânicos/dispatcher/schema; Clippy-Dwarnings, fmt/build/diff aprovados.
Regressão MCP completa64cenários/262grupos aprovada, incluindo4grupos novos
mesh_organic (pureza em play, campo/contornos, erros atômicos, histórico/publicação/
persistência/PNG/replay). Prova isolada3grupos também aprovada com pixels
vivos/replay iguais. Logs finais organic-final-tests.log, organic-final-clippy.log,
organic-field-build.log, organic-full-mcp.log e organic-mcp.log.

Proposta da PRIMEIRA bifurcação da mesma árvore: scripts/propose_tree_junction.py
chama a consulta MCP e compara exatamente com biblioteca. Altera explicitamente
só o pescoço inferior; tronco recortado t0.24..0.42, antes do fork0.44. Curva
original t≥0.30 mantida por splitBézier: seis anchors de copa protegidos, erro
matemático máximo9.42e-16m. JSON inclui controles originais/novo pescoço/curva
retida, decisões, request e Junction em artifacts/tree-junction-proposal/proposal.json.
Default32amostras/refinement1:291v/488t,96bordas/192vfixos, volume-11.53%,
maxmove0.16186m, rotação64.14°, resíduo5.50e-8; não altera a cena. Variante12/ref2
267v/500t continua reproduzível. Caso original agudo22.88° continua recusado,
sem mover geometria implicitamente. Os pontos novos da madeira não retêm IDs,
material/rig ou escultura por alegação automática.

A integração na árvore central cabe à frente compartilhada de EAF-80 após o
handoff estável, com índices reais, transições autorais e material explicitado.
A proposta e o kernel não ampliaram budgets. Normais/sombras/materiais/folhas
continuam coordenados no outro chat; alvo de realismo e aceite humano pendentes.
Sem push/main, nenhuma janela nova/controle das demos existentes nesta etapa.

### EAF-85 — resolução local da primeira bifurcação

Após7c7ebd0, apenas helper/documentação: comparação por preview_mesh_junction,
sem alterar kernel, colares, anchors, cena ou budgets.32/ref2 agrega396v sobre
32/ref1:687v/1280t. Blend0.04/raio0.29 passa; blend0.08/raio0.29 recusa suporte;
blend0.06/raio0.29 passa (volume-11.02%, maxmove0.15587m, rotação61.86°);
blend0.08/raio0.27 passa com raio explicitamente modificado (volume-14.45%).
Todos os casos admitidos têm resíduo<5.7e-8,96 bordas e192v fixos. A variante0.06
foi repetida pelo helper e comparada exatamente à biblioteca/MCP, anchors com
erro≤9.42e-16, sem Scene mutada. Dados correntes em sampling-comparison.json e
comparison-proposal.json sob artifacts/tree-junction-proposal. Nenhuma nova
compilação/feature Rust; gates720Rust/64MCP do núcleo08323cf326d8910f mantidos.
Avaliação na árvore central e escolha visual da receita continuam com a frente
compartilhada. Não declarar ganho de FPS ou forma final apenas por métricas.
## EAF-87 — plantas curvas com domínio material explícito

macOS Apple Silicon/Metal, fonte native-plants, cena56/replay60,
build`e3d1731ed06f70ab`. Base55/59 de sombras/materiais/orçamento incorporada,
com `Geometry.material_coordinates:None` nos construtores de junção anteriores.
Nenhuma janela da demo foi aberta, fechada, focada ou controlada.

- `cargo test --locked --release --all-targets`:736 testes/66 grupos aprovados,
  incluindo4provas novas de domínio/topologia/preservação/rejeição.
- `cargo clippy --locked --release --all-targets -- -D warnings`,fmt/diff e
  build release aprovados. Logs estáveis em artifacts/curved-plants-*.log.
- `python3 scripts/run_verifications.py`:67cenários/276grupos aprovados;
  relatório completo em artifacts/mcp-validation/report.json.
- Novo cenário curved_plants tem5grupos: fonte da mesma seed20261026, elipse
  nativa fechada1090v/2176t, escultura/material/rig/clipe; curva/taper alterados
  com preview puro e A-B-A/undo; erros/budgets/revisão/retesselação; perda de
  detalhe fino recusada; animação/persistência/replay/GPU/formato antigo.

Capturas nativas em artifacts/mcp-validation/artifacts/curved-trunk-before.png,
curved-trunk-after.png, curved-trunk-animated.png e curved-trunk-replay.png.
Pixels repeat/live/replay idênticos. O close foi inspecionado: forma curva
funciona, mas casca continua procedural e o tronco isolado é diagnóstico,
não nova árvore realista aprovada. A deformação exagerada expõe preservação.

Receita versionada em examples/native-tree-trunk.plant.json; descreve11estações
do eixo/radius da árvore central. Python fornece estações, não triangula.
Planos de seção paralelos e interpolação linear são explícitos. Retesselação
8→10 amostras perdeu1.37mm de relevo e foi corretamente recusada à tolerância
1mm; o teste usa refino angular aninhado8→16,2434v/4864t, dentro do limite.
Uma escultura só nessa amostra fina impede redução acima da tolerância.

Limitação de integração mantida: colares circulares do kernel de junção usam
outra amostragem/frame.32v não os tornam compatíveis com estes contornos.
Preservação de canais/rig não se transfere implicitamente ao miolo novo ou a
uma malha agregada. A árvore central/primeiro fork, cenas e renderer do outro
chat permaneceram intactos; aplicar plantas vinculadas a componentes da mesma
árvore e avaliar forma/casca/folhas continua com EAF-80/EAF-85. Sem aceite
artístico, entrada física, nova plataforma ou ganho de FPS declarados.

### Integração na árvore central — EAF-80/EAF-87

Repetida no worktree native-meshes após9527034:736Rust/66grupos,Clippy release
com-Dwarnings,build,fmt/diff e67cenários MCP/276grupos aprovados, build
e3d1731ed06f70ab/cena56/replay60. Logs artifacts/curved-tree-*.log e relatório
artifacts/mcp-validation/report.json. Nenhuma janela controlada.

Ensaio real `demo_tree_authoring.py --compare-branch --frames 48`: segundo
ramo primário publicado por edit_plant, preservando vínculo/fonte canônica;
258v512t,8estações. Árvore8assets/30instâncias10948v15802t armazenados64346t
desenho, sem ampliar budgets. Preview sem mutação; mudança de controle com
6centros de inserção≤1.174e-7m, fonte/material/canal e pose foliar preservados;
A-B-A repetido/undo/redo exatos. Componente conectado1839v3674t e demais
assets/instâncias iguais ao baseline pela mesma API de inspeção. Save/load
compara meshes completo, PNGs de execução/replay iguais e48frames distintos
formam ciclo4s exato. Fonte e cópia salva MCP byte a byte iguais.

Capturas após/estrutura/raiz/fork/casca/folha e editable-branch-A/B inspecionadas.
Edição B expõe quinas entre seções; não há attachments automáticos nem fusão
implícita desse ramo. Fonte5.358.019bytes/GIF2.481.820bytes correntes em
examples/native-tree-study.scene.json e artifacts/tree-study/artifacts/tree.gif.
Detalhes/checklist em docs/tree-plant-integration.md. Realismo e aceite artístico
continuam abertos; nenhuma nova alegação de performance ou plataforma.

## EAF-87 — interpolação suave de seções

Fonte native-plants sobre2bb65b7, cena57/replay61,
build`6758029880829ea2`, macOS Apple Silicon/Metal. Sem janela ou controle da
demo, novo renderer, dependência ou alteração dos limites geométricos.

- `cargo test --locked --release --all-targets`:742testes/66grupos aprovados.
  Seis testes novos:3matemáticos e3de integração de plantas.
- Clippy release/all-targets-Dwarnings, fmt, diff e build release aprovados.
  Logs estáveis em artifacts/smooth-plants-{rust,clippy,build}.log.
- `python3 scripts/run_verifications.py`:68cenários/281grupos aprovados;
  relatório completo em artifacts/mcp-validation/report.json.

PCHIP opt-in calcula tangentes limitadas e coeficientesf64 fora do tick. Provas
matemáticas cobrem valores racionais independentes para dados não uniformes,
C1 por Yfísico, extremos/platôs/duas estações, offsets extremos e dimensões
positivas. Provas de volume cobrem inversa das3formas, posições exatas das
estações e contagens iguais ao Linear; Linear sem campo/explicitamente Linear
gera exatamente a mesma geometria. Preservação exercita escultura, canal de
material, pesos, pai rotacionado/ossos fora de estação, TRrest-relative,
Linear↔PCHIP A-B-A, retesselação, undo/persistência/replay e guarda de formato
no target E na referência. Fonte canônica não é rebaseada por mudar o modo.

Novo cenário smooth_plants tem5grupos e usa a definição REAL de branch-editable-1
extraída da cena48aa4d4: examples/native-tree-branch.plant.json. Python descreve
o mesmo desenho e os comandos; nenhuma triangulação/cúbica é calculada nele.
Quatro casos Linear-A/PCHIP-A/Linear-B/PCHIP-B usam258v512t e câmera/luz/tick
iguais;6centros de inserção verificados em2µm. Três casos densos usam706v1408t,
segmentos[4,6,4], dentro do orçamento por asset. Rig realmente deforma a malha;
repeat/live/replay PNGs idênticos, save/load da fonte e asset exatos. Modos
desconhecidos, excesso de budget, revisão antiga e cena56 com PCHIP recusados
sem mutação. Artefatos smooth-*.png e smooth-plants-verification.json em
artifacts/mcp-validation/artifacts.

Inspeção das capturas: Linear-B denso conserva o cotovelo angular; PCHIP-B
denso arredonda a passagem, ainda mantendo a inflexão exagerada autorada.
PCHIP esparso segue facetado. A comparação distingue função de tesselação,
sem atribuir realismo aos testes. Casca procedural e encaixe sobreposto seguem
abertos. Outra frente confirmou a leitura visual e fará a aplicação no MESMO
ramo da árvore após o commit estável, com A como forma final e B diagnóstico.
Custo previsto para[4,6,4] é+896t:64346→65242, ainda sujeito ao preflight real.
Não declarar esses valores previstos como captura integrada antes do ensaio.

### Suavização aplicada à árvore salva — EAF-80/EAF-87

Integração do núcleo8aa1a11 como761c77b no worktree native-meshes. Mesma build
6758029880829ea2, cena57/replay61. Validação própria:742testes Rust/66grupos,
Clippy release-Dwarnings,build,fmt/diff e68cenários MCP/281grupos aprovados.
Logs artifacts/smooth-tree-*.log; relatório artifacts/mcp-validation/report.json.
Nenhuma janela/main/demo/push alterados.

`demo_tree_smoothing.py --frames 48 --segments-y 4 --input
artifacts/tree-study/examples/smoothing-input.scene.json` começou no arquivo56
de48aa4d4, byte a byte igual, carregado/migrado no57. Não houve reconstrução
de assets nem substituição da fonte.12casos de Linear/PCHIP×A/B×Y2/4/6, mesmas
estações/câmera/luz/tick; preview puro e resultado igual ao publicado. Source E
reference originais exatos em cada etapa, ciclos e persistência. Wood inteiro,
outros assets/instâncias/pose foliar exatos;6centros≤6.59e-8m. Mesmo número de
triângulos/canal material entre os quatro casos de cada resolução.

Topologia fechada/sem não-manifold ou isolados, loops/undo/redo, recusa de cópia
foliar especificamente por draw com inspect inalterado, save/load meshes+plants
e pixels vivo/replay exatos. GIF48frames distintos/4000ms/2.481.539B. Fonte
5.443.474B, igual byte a byte ao saveMCP. As únicas chaves superiores alteradas
contra a fonte anterior são version,meshes,plants; outros campos da cena,
source/reference serializados, demais assets e instâncias também são iguais.

Escolha final PCHIP-A[4,4,4]:482v960t no ramo,8assets/30instâncias,
11172v16250t armazenados64794t desenho,742t livres.6Y foi capturado65242t,
294t livres; melhora B diagnóstico, mas não justificou default em A neste close.
Capturas12casos,after/structure/back/root/fork/bark/leaf inspecionadas. Casca,
folhas, outros encaixes, attachments e realismo continuam abertos. CurvaC1 não
é tesselação/frame automático; nenhuma alegação de FPS/plataforma nova. Custos
e checklist em tree-plant-integration.md; estados explícitos em mesh-roadmap.md.


## EAF-89 — vínculos de instâncias e ossos na árvore

macOS Apple Silicon/Metal, native-meshes, cena58/replay62, build
`b0aba8aa96ad42cb`. Contrato antes do código em mesh-attachments.md, mesma
fonte488df28 carregada pelo MCP. Nenhuma janela/main/push/demo alterados.

- 748Rust/67grupos aprovados:6novos testes de hierarquia analíticaTRS, osso
 nomeado, armazenamento livre/chain32, clocks distintos, ciclos/ausências/
 remoções/escala, staging/undo/Frame e formatos.
- Clippy release/all-targets-Dwarnings,fmt/diff e build aprovados.
- 69cenários/285grupos MCP aprovados, incluindo4novos mesh_attachments:
 composição/clock, rejeições atômicas, detach/undo/versão e GPU/replay.
- 223build_id dos JSONs de relatório do root de regressão concordam com o
 mesmo build. Logs artifacts/attachments-*.log; mcp-validation/report.json.

Ensaio `demo_tree_attachments.py --frames48 --input
artifacts/tree-study/examples/attachment-input.scene.json`: replica recipe
só para conferir todos os índices/arrays de wood, extrai71v114t por índices
para secundário e rametas, primeira montagem1839v3674t exata. Restpose da
extração≤2.98e-8m e copa sem diferença; outros assets/instâncias permanecem
iguais, exceto assets wood/branch-editable-1 e instâncias
tree-branch-editable-1/crown-06, além do novo asset/instância secundário.
Não houve geração de geometria extra.

Socket no rig do galho e parent da copa são declarativos; editar estação0.30
em[0.15,-0.10] move ramo+copa[0.150000036,0,-0.099999994]m sem mudar seus
assets. Rejeição ao renomear osso referenciado/remover pai preserva inspect.
Animação do osso1.5graus e playback próprio da folhagem comparados com rotação
analítica da pose antiga no tick60, erro<5e-6m. Madeira principal fica estática.
Rig/clipe autorados na referência original; canal material sofreu roundoff
1.3411e-7m no rebase explícito e foi restaurado pelo setter. Geometria/material/
referência originais exatos +rig/clipe novos; fonte enriquecida fica exata
nas edições seguintes. Compensação autoral, não correção nativa de deriva.

Mais12casos de modo/resolução em câmera/luz/tick iguais sobre a árvore agora
vinculada, undo/redo, preview puro, save/load meshes+plants e pixels de execução/
replay aprovados. Final9assets/31instâncias11172v16250t armazenados64794t
desenhados,742t livres. Fonte5.663.332B/GIF2.481.054B48frames distintos/4s,
mesmos paths correntes e fonte salva pelo MCP igual byte a byte. Capturas
attachment-rest/edited/animated,after/structure e detalhes inspecionadas.

Vínculo genérico implementado, mas somente um secundário/copa convertido nesta
árvore;1slot de instância restante. Seleção/extração nativa, organização de
componentes, distribuição/máscaras, materiais avançados e LOD continuam abertos.
Sem solda/frame de curva ou vento físico implícitos, nem aceite de realismo.
Checklist antes/depois em mesh-attachments.md/performance.md.

## EAF-90 — inspeção e extração nativa de ilhas

Base f810466, codex/native-meshes, macOS Apple Silicon/Metal, build
`5f8a49530de4298c`. Contrato/tradeoffs e checklist anterior/posterior em
mesh-islands.md/performance.md. Sem mudança de formato: Scene58/replay62,
comandos/consultas produzem assets já representáveis. Render/layout e budgets
permanecem. Nenhuma janela/main/demo existente/push alterado.

- `cargo test --locked --release --all-targets`:755testes/68grupos,7novos testes
 de ilhas. Contato por1v, posições coincidentes com índices distintos, isolados,
 canais/pesos/ossos/clipes/poses exatos, Arc de clipes, previews genéricos,
 referências/lotes/seleções inválidas, source/dest, undo/replay. Copy de planta
 mantém fonte; Move exige detach explícito (inclusive detach+Move no mesmo lote).
 Preflight de assets/V/T/chaves; Copy e Move recusados ao duplicar clipes acima
 do teto. Erros preservam execução/histórico.
- Clippy release/all-targets `-D warnings`, build, fmt e diff aprovados.
- Regressão completa:70cenários/288grupos MCP headless, incluindo3novos
 mesh_islands. Schema, queries/bounds determinísticos, seleção/pure preview,
 Move/Copy rigados, buffers/cache/viewport, undo/redo, save/load/reset e PNGs de
 execução/replay/prefixo.225build_id nos JSONs do root isolado concordam com o
 mesmo build. Logs artifacts/mesh-islands-*.log; mcp-validation/report.json.

`demo_tree_islands.py --frames48` carrega a árvore de f810466 e usa summaries
nativos para selecionar seis rametas. Move→secondary-twigs e instância explícita
filha do secundário. Ramo35v66t, rametas36v48t; poses e PNGs antes/depois exatos
em repouso/tick60. Canais e faces da união dos subsets idênticos; outros assets,
instâncias, planta/source/reference e primeirofork preservados. Copy completo
rigado de folhagem em preview exato, sem instância/rebase. Referência antiga
recusada e undo/redo integral aprovados. Capturas islands-before/after-rest/
motion e árvore completa inspecionadas; a reorganização conserva a aparência.

Bateria adicional12casos Linear/PCHIP×A/B×Y2/4/6 sobre a mesma árvore reorganizada,
preview/publicação, undo/redo, fonte/reference, save/load meshes+plants e PNG
vivo/replay exatos; GIF48frames distintos/4s/2.481.054B. Final10assets/32instâncias,
11172v16250t armazenados64794t desenho/742t livres. Fonte corrente5.664.534B igual
byte a byte ao save_scene MCP. Probe de draw substitui uma instância por madeira
mais pesada ao atingir32slots; erro por65536t explicitamente conferido sem
publicação. Paths estáveis artifacts/tree-study/artifacts/islands-report.json,
report.json,tree.gif e examples/native-tree-study.scene.json.

Capacidade técnica geral entregue. Não há ferramenta exclusiva de árvores,
seleção por proximidade, solda de costuras, cortes/booleanos arbitrários, scatter,
LOD ou rig compartilhado. Todos32slots estão ocupados; isso é dado do benchmark,
não decisão automática de arquitetura. Casca e folhas ainda insuficientes no
close; objetivo artístico EAF-80 continua aberto, sem novo aceite humano ou
alegação de FPS/escala/multiplataforma.

## EAF-91 — ferramentas de superfície, folhas e casca

Base1a043f1, codex/native-meshes, macOS Apple Silicon/Metal; build
`27f760fca90ee90d`, Scene59/replay63. Material.detail genérico com até32segmentos
2D afunilados no domínio material de repouso, tinta/bump independentes, fade
subpixel. fit_material_coordinates e copy_material_coordinates pelo mesmo
MCP/dispatcher/preview/undo. Ausência neutra, guardas de formato antigo; nenhum
shader/tipo botânico. Contrato/alternativas e revisão antes/depois em
surface-details.md/performance.md. UI explicitamente adiada.

- 763testes Rust/69grupos release/all-targets,8novos:7de superfície/correspondência
 e1de layout. Validação de limites/degeneração/finito, fit por asset/ilha/fallback/
 eixo constante/isolados, preview/undo, rig/clipes/extração/subdivisão/replay,
 plantas/source e legado58. Copy com topologia ordenada correspondente e
 geometria deformada, casos incompatíveis/isolados/canal ausente sem mutação.
- Clippy release/all-targets `-D warnings`, build/fmt/diff aprovados. Logs
 artifacts/surface-rust-final.log,surface-clippy-final.log,surface-build-final.log.
- 71cenários/291grupos MCP headless,3grupos novos de superfície. Pixels para
 traços/tinta/bump isolados, repetição/undo/slots; fit/cópia por template,
 erros/pure preview, rig com transformação compensada preservando pixels de
 domínio de repouso, subpixel distante idêntico ao neutro, persistência/reset/
 viewport e replay/prefixo GPU exatos.227build_id nos JSONs do root isolado
 concordam com o mesmo build. artifacts/surface-mcp-final.log e relatório
 artifacts/mcp-validation/report.json. Render/plantas/funcionalidade completos.

A demo carrega a MESMA árvore salva, sem reautorá-la por receita. Folha-fonte
recebe fit,4variantes66ilhas cada recebem domínio por correspondência exata,
inspecionada nativamente; aplicação em todas as264folhas armazenadas conferida,
repetidas nas1782desenhadas. Primeira tentativa de fit diretamente nas variantes
rotacionadas não orientava nervuras: corrigida por transferência explícita
validada de topologia, sem nearest/PCA. Captura de folha extraída da copa real
com rig/clipe comprova orientação, além da folha-fonte. Montagens de inspeção
são transações temporárias desfeitas e comparadas com a cena anterior.

31segmentos de nervuras curvas por polylines e ramificações,32de fissuras na
casca com campo ridged anisotrópico. Primeiro desenho reto foi refinado após
inspeção; não basta gerar um diagrama de nervuras. Antes/depois com mesma
câmera/luz/tick60 e ablação sem detail na folha e casca; capturas de folha-fonte,
folha da copa,casca,forquilha e árvore inteira inspecionadas. Todos os assets
conservam posições/triângulos/rig/clipes e instâncias/poses0/60 exatos; somente
canais das folhas e materiais mudam. Planta mantém geometry/rig/clips da fonte,
reference/definition; primeirofork geometricamente exato. Materiais novos são
autoria explícita, não preservação da aparência antiga.

12casos Linear/PCHIP×A/B×Y2/4/6, fonte/reference, undo/redo/save/load meshes+plants,
PNG vivo/replay e loop48frames distintos/4s aprovados no conteúdo final.
10assets/32instâncias11172v16250t armazenados64794t desenho/742t livres, mesmos
budgets. Fonte corrente6.449.801B igual byte a byte ao saveMCP; GIF2.491.445B.
Relatórios artifacts/tree-study/artifacts/surface-report.json e report.json,
script demo_tree_surfaces.py --frames48 --input
artifacts/tree-study/examples/surface-input.scene.json reproduz desde o baseline.

Revisão corrigiu conversão da tabela maior a cada frame: agora só quando a
receita muda, comparação por Arc/conteúdo e upload cacheado. Tabela19312B,
Vertex52B e sombra mantidos. Fit sem Vec por isolado. Sem profiling/FPS,
multiplataforma, UI ou nova escala presumidos; nenhuma janela/demo/main/push.
Limites artísticos abertos: zero espessura e contorno poligonal das folhas,
casca ainda procedural/fibra sem frame global nas bifurcações, difuso sem
roughness/reflexão/PBR completo.32segmentos já limitam detalhes mais densos;
Copy é snapshot por topologia, não vínculo vivo. EAF-80/realismo permanece aberto.
### Consolidação na árvore salva —488df28

A frente central importou8aa1a11 como761c77b e consolidou a aplicação em
488df28, mesmo build6758029880829ea2/cena57/replay61. Validação integrada
742Rust/66grupos,Clippy/fmt/build/diff e68MCP/281grupos;221build_ids conferidos
sem divergência. Relatório/capturas finais foram inspecionados na worktree
native-meshes. Não houve nova modificação do núcleo.

`demo_tree_smoothing.py` carregou a cena56 anterior de48aa4d4 byte exata;
12casos Linear/PCHIP A/B em2/4/6Y preservam source E reference históricos,
wood inteiro/primeirofork, outros assets/instâncias e pose foliar no tick.
Seis centros têm erro≤6.59e-8m. Ciclos/undo/save/load/replay/PNGs exatos.
Final pchip-A-medium:[4,4,4],482v960t,11172v16250t armazenados,64794t
desenhados e742livres. A densidade6 foi capturada:65242t/294livres, mantida
como comparação. Fonte5.443.474B; GIF48frames distintos/4s/2.481.539B.

Artefatos na outra worktree:artifacts/tree-study/artifacts/after.png,
tree-smooth-linear-B-medium.png,tree-smooth-pchip-B-medium.png,tree.gif e
report.json. PCHIP arredonda a passagem; B mantém inflexão exagerada autorada.
A imagem completa ainda reprova realismo, casca e encaixes. Restrição do
próximo kernel registrada em curved-plants.md: se tronco E ramo fossem
loftsXZ/Identity, suas saídas paralelas+Y não satisfariam dois suportes
estritos, independentemente de contar3ou4colares. A montagem atual NÃO tem
dois desses planos: trunk_above segue tangent(0.42), levemente inclinado.
Sua admissão precisa ser medida com os contornos reais; o exemplo paralelo
não prova sua recusa. Frames fixos/cortes oblíquos/sweepXYZ são alternativas,
não implementação. A correção é documental e não altera o núcleo validado.

## EAF-94 — rugosidade e reflexo dielétrico da luz direta

Base db3b3de, codex/native-meshes, macOS Apple Silicon/Metal; build
`087b85a4ab8c3859`, Scene60/replay64 LOCAIS à frente gráfica. Duas variáveis
opcionais no Material existente, GGX/Smith/Schlick frontal com câmera/luz atuais,
difuso atenuado e brilho sob a mesma sombra. None/F0zero mantém legado exato.
Contrato/alternativas/revisão antes-depois em dielectric-response.md/performance.md.
Versionamento funcional concorrente EAF-92/93/95 exige integração/unificação
explícitas; nenhum merge no main ou interoperabilidade presumida nesta unidade.

- 769testes Rust/70grupos release/all-targets aprovados (6novos:4de contrato,
  1de empacotamento/cache e1de layout global). Limites/NaN/infinito/rollback sem
  histórico, pure preview, fonte/reference/regeneração/extração/subdivisão e
  Scene59/replay63 forjados recusados. Logs artifacts/rust-validation.log.
- Clippy release/all-targets -Dwarnings, fmt/build/diff aprovados. Log
  artifacts/clippy-validation.log; aviso preexistente de incompatibilidade
  futura da dependência block0.1.6 não é warning novo do código.
- 72cenários/296grupos MCP completos headless,5novos. Shader real: None/F0zero,
  rugosidade/F0, câmera/luz/bump, backlight/all-ambient neutros, rasante/plano
  coincidente, sombra cobrindo folha sem especular residual, rig compensado
  com pixels exatos, slots/undo/redo/save/load/reset/viewport/replay/prefixo.
  artifacts/reflectance-mcp-final.log e artifacts/mcp-validation/report.json.
  229build_id dos JSONs do root isolado concordam com o mesmo build. Captura
  surface-neutral.png também byte-exata contra reflectance-legacy-neutral.png
  preservada do build anterior, além dos controles neutros no build novo.

Mesmo examples/native-tree-study.scene.json carregado, nunca refeito por receita;
só metadados de material mudam. Folhas roughness0.58/casca0.92,F0=0.035. Posições,
triângulos,canais,rigs,clipes,shading,instâncias e poses0/60 exatos; fonte da planta
só recebe metadado, geometry/rig/clips/reference/definition preservados sem rebase.
10assets/32instâncias11172v16250t armazenados64794t desenho/742t livres. Source
6450759B igual byte a byte ao saveMCP corrente; GIF2487677B,48frames distintos/4s.
12casos Linear/PCHIP×A/B×Y2/4/6, undo/redo/persistência/loop e PNG vivo/replay
continuam passando com o novo material; todos no mesmo build. Relatórios
artifacts/tree-study/artifacts/reflectance-report.json e report.json.

Antes/depois inteiro/casca/forquilha em câmera/luz/tick60 iguais; folha extraída
da copa REAL com rig/clipe e montagem temporária reversível em tick0, direção
câmera/luz explícita para observar reflexo. Controles roughness0.3/0.9/F0zero na
mesma montagem; zero reproduz antes byte a byte. Imagens inspecionadas sem retoque.
Delta médio de canal em0..255 nos pixels de primeiro plano: folha21.8932, casca
0.8422; controle de rugosidade foliar72.9013. Medidas provam diferença, não qualidade.

Script demo_tree_reflectance.py --frames48 --input
artifacts/tree-study/examples/reflectance-input.scene.json reproduz; Pillow/NumPy
apenas medem PNGs nativos/empacotam GIF, não geram geometria nem retocam imagens.
Reexecução com material já aplicado faz ablação neutra explícita. Capturas
reflectance-before/after-crown-leaf,bark,fork,whole e controles em root corrente.
Sem janela/demo/main/push, UI adiada. Vertex52B/tabela19312B mantidos e Globals
160→176B; cálculo constante por fragmento habilitado, sem textura/lock/dependência
nova. Não alegar FPS. Alto brilho expõe aliasing nas nervuras e aspecto plástico;
UNORM sem HDR, ausência de IBL e rugosidade uniforme por asset são limites
mantidos. Folha ainda poligonal/sem espessura, casca demasiado estriada e encaixes
sob frente geométrica separada. EAF-80 e aceite artístico continuam abertos.


### Continuidade da primeira bifurcação — EAF-85/EAF-80

Baseline db3b3de, worktree native-plants. Novo campo opcional
Fusion.transition_lengths na consulta pura preview_mesh_junction. Scene59/replay63
mantidos nesta frente; nenhum contrato de material/render/main alterado. Números
locais não comprovam compatibilidade com a linha funcional paralela.

A opção zero conserva toda a resposta antiga exatamente. Positivos preservam os
mesmos colares/guardas/triângulos e impõem normal radial no limite da guarda;
validação analítica recusa faixas que atinjam outras guardas. Testes matemáticos
e de topologia/legado/envelopes em src/mesh_organic.rs e tests/mesh_organic.rs.
Novo grupo MCP cobre opção/relatório/erros atômicos, inclusive em play.

Ensaio scripts/demo_tree_continuity.py carrega a MESMA árvore salva, reconhece
bit a bit o prefixo da primeira montagem e muda apenas essa região de wood.
Bordas/índices são compartilhados, não soldados por distância. Novo pescoço
Bézier usa alças0.575/0.375m, derivadas analíticas, taper Hermite compatível por
comprimento de arco e8intervalos. O primeiro teste, com mais amostras e a curva
antiga apertada, produziu dobra155° e foi rejeitado; faces laterais agora têm
orientação externa e o produto raio×curvatura máximo amostrado é0.734084/1001p.
Isso não certifica ausência global de autointerseção.

Comparação na mesma câmera/luz: borda do ramo46.1047°→10.6236° máximo entre
faces; guarda interna31.6454°→22.7334°. Tronco superior continua28.7064° no pior
triângulo; tesselação e demais encaixes permanecem abertos. Capturas reais
before/after-whole,fork,bark e animated-bark em artifacts/tree-continuity/artifacts.
Contorno do cotovelo melhorou; o encaixe sobreposto do lado oposto continua
visível. Nenhuma alegação de árvore realista concluída.

Demais assets/plantas/instâncias/materiais/rig/clipes e poses0/60 são exatos;
madeira restante é preservada por offset explícito dos índices. Canal de
material conservado no patch/raiz/tronco e restante de wood; o pescoço novo
recebe domínio autorado de repouso, sem alegar transferência de escultura.
Undo/redo, preview puro/publicação, save/load e PNG vivo/replay passaram.
Árvore10assets/32instâncias,11300v/16506t armazenados,65050t desenhados
(+128v/+256t),486t livres; limites não aumentados. Fonte candidata é reproduzível
no caminho estável artifacts/tree-continuity/examples/candidate.scene.json;
a cena central da outra frente não foi substituída.

765testes Rust/69grupos passaram em cargo test --locked --release --all-targets;
Clippy release/all-targets com-Dwarnings,build,fmt/diff e py_compile passaram.
Regressão MCP completa:71cenários/292grupos passaram sem janela; relatório
artifacts/mcp-validation/report.json. Build e7d9b7c1c8624d49, macOS Apple Silicon.
Checklist anterior/posterior em performance.md e mesh-organic.md. Sem push,
janela/demo/main ou aceite artístico presumido.


### Trajetória + seções: cilindro isolado — EAF-97

Contrato/checklist anterior ao código em plant-trajectories.md. Scene64/replay68
LOCAIS, reserva registrada após63/67funcionais; não incluem o schema funcional
nem o lookdev60/64 de outra frente. Build73ee6f5e3b49ef68,macOS Apple Silicon.
Fonte limpa examples/native-curved-cylinder.plant.json e .scene.json, salva pelo
MCP; raio2cm/altura30cm/desloc10cm, duas estações circulares e curva cúbica em S.
Não houve retorno à árvore nem alteração de main/demo/janelas.

771testes Rust/70grupos (6novos em plant_trajectories.rs), release/all-targets;
Clippy-Dwarnings,build,fmt/diff e py_compile aprovados. Regressão completa
72cenários MCP/296grupos headless, relatório artifacts/mcp-validation/report.json.
Cenário novo cobre preview puro/malha fechada, parâmetros inválidos/resolução/
identidade/undo/ciclos canônicos e origem bind/clipes, refinamento válido,
source/reference/material/escultura/pesos, tick60 REAL depois de set_camera,
PNG vivo/replay/repetido, save/load e recusa de cena anterior64. Retornos para
resolução anterior preservam a fonte, sem rebase por regeneração.

Resultado:3922v/7840t;64cortes×56v de borda=3584v laterais verificados por
fórmulas independentes de C(t),C'(t). Raio relativo ao escalarf32 armazenado:
erro máximo4.4318e-9m; plano normal7.3863e-9m. Essa precisão de vértice NÃO é
precisão da face inteira. Erro lateral AMOSTRADO entre a malha e o mapa analítico
7.16133e-5m=0.071614mm/28224probes, abaixo da tolerância0.2mm. Não é limite
Hausdorff global. Certificado de inversa: margem0.01302876m²; F' estritamente
negativo em todo domínio admitido, não uma mera checagem amostrada de dobra.

Capturas nativas neutras cylinder-front.png,cylinder-oblique.png e
cylinder-front-flat.png mostram geometria/silhueta sem textura/bump; a vista flat
expõe facetas finitas. Inspecionadas visualmente. Fixture separada de escultura/
rig/clipe é removida antes de salvar a cena limpa. Artefatos/report próprio em
artifacts/mcp-validation/artifacts/plant-trajectories-verification.json.

Limites: curva é gráficoXZ deYlinear crescente; não faz loops/retornos. Frame
por referência fixa, sem transporte paralelo geral. Critério suficiente pode
recusar formas válidas; políticasXYZ e rotação global do rig anteriores são
mantidas, sem transporte tangente de escultura implícito. Controle e relatórios
viaedit_plant/preview_plant/inspect_plant, sem UI/renderer novo. Implementação e
validação automatizada concluídas; aceite visual do cilindro separado. A árvore
continua fora deste incremento por pedido direto do usuário.
## EAF-80/EAF-85 — continuidade geométrica com materiais EAF-94

Direção direta de Guilherme: integrar e mostrar a árvore, mantendo-a como ensaio
para ferramentas gerais. Importado c1fbf6a como ae8fb78 sobre435742d, com conflito
apenas documental resolvido preservando ambos os históricos. Scene60/replay64
LOCAIS, renderer/material/Globals176/Vertex52/tabela19312 intactos. Nenhum merge
com formatos funcionais concorrentes. Checklist antes/depois e reprodução em
[tree-geometry-material-integration.md](tree-geometry-material-integration.md).

Buildfda9972793711701:771testes Rust/70grupos release/all-targets (2importados),
Clippy release/all-targets-Dwarnings,build/fmt/diff/py_compile aprovados.
72cenários/297grupos MCP headless completos,229build_id em JSONs do root isolado
coerentes. Logs artifacts/continuity-{rust,clippy,build,mcp}-final.log e
continuity-integration-final.log. Relatório mcp-validation/report.json.

A MESMA cena435742d foi carregada, reconhecido prefixo exato e corrigido só
primeiro componente de wood. Todos os9materiais com reflectance/detail exatos;
assets não wood inteiros, plantas/source/reference, instâncias/attachments e
poses0/60 também exatos. Resto de wood reindexado explicitamente e conservado;
domínio comum mantido, pescoço novo tem domínio autorado. Mesmo prefixo de
raízes/tronco, nenhum bake da candidata59 por cima do lookdev. Topologia/colares,
orientação das faces, preview puro/undo/redo/save/load e pixels replay validados.

Máximo diédrico do encontro do ramo46.1047°→10.6236° e guarda interna
31.6454°→22.7334° no build integrado. As capturas antes (whole/bark/fork) são
byte-exatas às capturas após o incremento de material435742d, comprovando que a
comparação parte da mesma aparência. Depois, câmera/luz/tick60 idênticos e
imagens nativas inspecionadas: cotovelo arredondado, ainda com facetas na parte
superior e encaixe sobreposto à direita visível. Guarda superior28.7064° mantida;
r×κ amostrado0.734084/1001p não certifica autointerseção global ou realismo.

Correção de evidência: set_camera sempre passa por replace/reset, inclusive
valor igual. O capture do ensaio recebido fazia isso DEPOIS de step60; seu
animated-bark era efetivamente tick0. Script agora edita câmera ANTES de step,
confere inspect.tick60 e testa prefixo0 separado contra cena carregada emrepouso.
Sem alteração da engine para acomodar o teste. Finalizador deixou de presumir
64794t e confere custos dos assets preservados+fixture960/1408t do ramo; limite
65536 e teste de rejeição por desenho mantidos.12casos Linear/PCHIP×A/B×Y2/4/6
passam conservando a nova madeira; pixels vivo/prefixo/reset/viewport exatos.

10assets/32instâncias11300v16506t armazenados,65050t desenho/486livres
(+128v/+256t). Madeira3808v<4096. Fonte6500223B byte-exata ao saveMCP corrente e
cópia de validação integrada; GIF2489726B/48frames distintos/4s. Relatórios
artifacts/tree-integrated/artifacts/report.json,tree-continuity/artifacts/report.json
e tree-study/artifacts/report.json no mesmo build. Script demo_tree_integrated.py
--frames48 --input artifacts/tree-integrated/examples/input.scene.json reproduz.
Pillow só empacota GIF, sem geometria/pixels gerados substituindo o render.

Nenhum ganho FPS/portabilidade/aceite artístico/UI/árvore totalmente soldada
alegado; restante das junções e detalhamento permanecem abertos. Main/demo/
janelas e worktree de plantas preservados, sem push. EAF-80/EAF-85 In Progress.

## EAF-85/EAF-80 — pontes reais e facetas da transição central

Base d39c73a em codex/native-meshes, build3f194031f2cdda49. Scene60/replay64
LOCAIS, renderer/layout/limites mantidos; sem importar trajetória/seções da
frente paralela.776testes Rust/71grupos release/all-targets, Clippy-Dwarnings,
build/fmt/diff/py_compile e73cenários MCP/300grupos headless passaram.231ocorrências
de build_id no root da regressão concordam; relatório bridge-build-audit.json.
Logs bridge-all-tests.log, bridge-clippy-final.log, bridge-build.log,
bridge-full-mcp.log. Nenhuma janela/demo/entrada física controlada.

Novos contratos nativos: inspect_mesh_boundaries, remove_faces e
bridge_boundary_loops por dispatcher/referência/preview/staging. Cenário
mesh_bridges verifica ciclos/orientação, erros/stale/lote sem mutação,
preflight/rig4 em Rust, preservação de canais/rig/clipes, novo skin, undo/redo,
save/load/reset, replay/live/prefixo por pixels e viewport. Fixture com8spans e
handles2 demonstra dobra que acompanha o Jacobiano; a guarda de faces
adjacentes>=90° a recusa. Limite conservador, não prova global de self-intersection.

Árvore: ensaio de grupos radiais e depois duas pontes superiores reprovados;
melhor fase superior ainda126.126° em faces internas. Componentes originais
preservados,178ilhas antes/depois. Nenhuma declaração de todos os encaixes
seamless. A candidata entregue troca só320faces/128pontos internos do pescoço
por ponte32×10 e suavização com pincel nativo, seis strokes em duas passagens.
Todos os pontos/canais retidos exatos, demais assets/plantas/source/reference/
instâncias/attachments/poses0/60 e materiais exatos. Região completa:
max24.2474°→22.8988°, média6.11391°→5.26886°. Deslocamento só em novos pontos,
max22.2593mm pela escultura; volume total wood varia0.065284%, sem afirmar
conservação local. Ainda há facetas, encaixe paramétrico oposto e secundários.

Wood3968v7224t;10assets/32instâncias,11460v16826t armazenados,65370t desenhados,
166livres. Incremento160v320t; expansão adicional960v×52B=49920B. Nenhum teto
ampliado. Densidade6 exigiria65818t:4casos agora verificam rejeição atômica,
8casos Linear/PCHIP A/B nas densidades2/4 continuam aceitos e repetíveis.
Loop48frames distintos/4s/GIF2489015B; replay/prefixo/save/load/reset exatos.
Fonte6561867B SHA256
48519674cb41d31eab394c44c74d1c6a4fbd9ea475bf96dfc9edbfdda22d7386,
byte-exata ao save MCP; integração compara materiais e todo o restante da cena.

Reproduzir sobre o snapshot de entrada pré-ponte:
`python3 scripts/demo_tree_bridges.py --input artifacts/tree-bridges/examples/input.scene.json`,
depois `python3 scripts/demo_tree_smoothing.py --input artifacts/tree-bridges/examples/candidate.scene.json --frames 48`
(usando Python com Pillow). Relatórios tree-bridges/
artifacts/{report,integration}.json e tree-study/artifacts/report.json; comparações
nativas before/after-{whole,upper,back,bark}.png com câmera/luz/tick60 iguais.
Reprovação de superiores pode ser exercitada com --include-upper em root
separado artifacts/tree-bridge-upper-probe, sem promover fonte. Arte aceitável apenas como próximo ensaio; aceite humano/realismo aberto.


## EAF-80/EAF-85 — adoção de trajetória na árvore, checkpoint

Integração local815da1d+5ee628d, Scene65/replay69. Build ee56daed754aa95a,
macOS Apple Silicon, captura Metal offscreen; main/demo/janela preservados.
783testes Rust em72grupos,fmt,Clippy-Dwarnings/build,py_compile/diff e75cenários
MCP/307grupos aprovados. Logs tree-trajectory-{all-tests,clippy,build,full-mcp}.log.
O novo plant_adoption cobre fonte decorada com escultura/material físico/rig
2bones e clipe real, preview puro, identidade, densidades/ciclos, rollback,
undo/redo, gate65, save/load e pixels animados60/replay/prefixo. Meshes prova
131072t na GPU e131073 recusados atomicamente; gate64/54 testado em Rust.

Na MESMA árvore, somente branch-editable-1 recebe trajetória nativa[6,6,6];
source258v e referência Linear[4,2,4] permanecem exatos. Alvo1082v2160t,
8064probes, erro lateral amostrado3.101402mm<4mm, margem inversa0.63517678.
Material, rig, clipes e domínios canônicos preservados; fonte desse ramo tem
max_detail_displacement0 (preservação de escultura não nula é provada no teste
separado, não inventada nesta cena).9keyframes adaptados; erro de mapeamento
4.26e−8m. Todos os outros assets/instâncias exatos e poses0/60 com erro0m.
Preview/undo/redo/ciclos A/B/save-load/replay/prefixo passaram. Capturas em
artifacts/tree-trajectories/artifacts/{before,after}-{whole,branch,fork}.png;
relatório report.json e fonte nativa salva candidate.scene.json promovida ao
exemplo mantido.10assets/32instâncias,12060v18026t armazenados,66570t desenhados.

ENCAIXE AINDA NÃO CONCLUÍDO: capturas deste checkpoint conservam a conexão
anterior. Outros galhos estáticos continuam fora do vínculo paramétrico. Lofts
e ajustes de energia reprovados não estão no produto; repro congelado em
artifacts/junction-research. Kernel tubular novo está na frente de malhas;
composição regenerável proposta em plant-assemblies.md. EAF-80/85 continuam
In Progress; capturas/testes não equivalem a realismo ou aceite humano.

## União volumétrica limitada — EAF-85, base nativa

Build24c643076c7740e5 em codex/native-meshes, Scene60/replay64 LOCAIS mantidos.
780testes Rust/72grupos release/all-targets, Clippy-Dwarnings, build/fmt/diff e
74cenários MCP/304grupos headless passaram. Logs junction-union/{rust,clippy,
build,mcp-full}.log. Novo cenário mesh_junction_union cobre query pura repetível,
oito classes de erro sem mutação, prefixo exato, publicação/undo/redo, poses0/60,
filho animado preso ao socket, save/load/reset/cache/replay/prefixo e viewport.
O corpo fonte é rígido à raiz; movimento é provado no filho, não atribuído ao
corpo. Testes Rust incluem três tubos sintéticos sem rig/domínio de árvore.

Fixture versionado examples/mesh-junction-union.input.json, source2800v5506t,
portas32/24/32; grade9044, extraído1107v2099t, final3995v7986t. Fonte/canais/pesos
retidos exatos, ossos/clipes/material preservados.0bordas/0não-manifold/nenhum
isolado/1componente/Euler2. Ângulos: máximo59.7362°, média5.01344°, costura fonte
1.15059°, conexão interna59.7362°, interior53.5986°. Erro de campo quantizado f32
5.83302e-8m. Métricas não equivalem a aceite artístico ou prova global de
autointerseção. Colar ainda perceptível no neutro; transporte do domínio nos
pontos novos também continua aberto.

Capturas nativas correntes mcp-validation/artifacts/union-{authored,neutral}-
{front,profile,back}.png, report mesh-junction-union-verification.json e audit
union-build-audit.json. Capturas neutras sem textura procedural/sombras com
câmera/luz/tick60 controlados. Relatório distingue standalone_proof=true de
whole_tree_integration=false. Nenhuma janela/demo/entrada física controlada;
fonte native-tree-study permanece SHA256
48519674cb41d31eab394c44c74d1c6a4fbd9ea475bf96dfc9edbfdda22d7386.
Contrato/checklist em implicit-junctions.md/performance.md. Sem novo formato,
dependência, renderer/layout/teto nesta unidade; EAF-80/EAF-85 permanecem abertos.

## Refinamento finito da junção — EAF-85

Build5a5ce80254e1f74d, mesma branch/Scene60/replay64/tetos.783testes Rust release
all-targets/72grupos, Clippy-Dwarnings,fmt/build/diff/py_compile e74MCP/305grupos
headless passaram;233build_ids concordam. Logs junction-union/refine-{rust,
clippy,build,mcp-full}.log. Os7testes do kernel cobrem opções default0, fonte/
guarda/canais/rig exatos,1/2/4passos, erro8por piora angular, parâmetros inválidos
e domínio com RHSzero/condições fixas não nulas num fixture genérico sem rig.

Saída mantém3995v7986t:352v iso selecionados, fonte e guarda88 exatas. Pico
59.7362°→43.4687°, conexão interna41.7063°, interior43.4687°, média4.96777°,
costura fonte1.15059°. Envelope25mm, deslocamento medido24.999994mm, volume
assinado1.748973275→1.748597314m³ (−0.021496% do total fechado, não prova local).
Campo antes da faixa5.833e-8m, após6.9267mm: a melhoria sai da iso-superfície
dentro do envelope, ambos os erros são reportados. Guardas90 e contagens não
foram relaxadas, sem garantia global de autointerseção.

Material: diferença fonte/guarda3.593691→0,64passos de correção no domínio,
resíduo recalculado pós-f326.0509e-5unidades. A malha já era C0 por índices
compartilhados; o defeito era variação forte no comprimento8mm da faixa.
Comparação authored evidencia menos linhas abruptas, ainda veios distorcidos
e derivadas artificiais. Não chamar de fibras naturais/transferência resolvida.
Vistas atuais union-{authored,neutral}-{front,profile,back}.png e controles
union-baseline-{authored,neutral}-{front,profile,back}.png em mcp-validation/
artifacts; câmera/luz/tick60 iguais, marcador omitido das vistas geométricas.

Cenário ampliado preserva dados/poses0/60/filho animado no socket, publicação/
undo/redo/save/load/reset/cache/erros/replay/prefixo e viewport. Material é
independente da geometria e não altera ossos/pesos. Fonte da árvore inteira
permanece SHA48519674cb41d31eab394c44c74d1c6a4fbd9ea475bf96dfc9edbfdda22d7386;
whole_tree_integration=false. Sem janela/demo/controlar outra branch. A
composição e o ensaio de densidade da frente de plantas são gates próprios;
EAF-80/EAF-85 continuam abertos e o colar artístico permanece visível.

## EAF-85 — kernel puro de suavização por região

Novo módulo mesh_surface, API/tipos públicos e testes; sem enum MeshEdit,
dispatcher/plantas ou migração.794testes Rust/73grupos release/all-targets,
Clippy-Dwarnings,fmt/diff passaram em macOS Apple Silicon.11testes focados
cobrem máscara/âncoras/bordas/envelope,ilhas próximas não soldadas,faces
incidentes com vizinho fora da seleção,winding/arestas/estrelas não-manifold,
degeneração/inversão/piora de qualidade,volume fechado explicitamente reduzido,
rig/canais e poses fixas/socket0/60 exatos. Fonte imutável; fixos preservam bits
inclusive-0; região totalmente fixa retorna Arc compartilhado. Contrato/
checklist/custos em mesh-surface-filter.md/performance.md, logs estáveis
artifacts/mesh-surface/{tests,all-tests,clippy}.log.

Nova operação ainda sem exposição MCP/undo/persistência ou prova visual da
árvore nesta unidade. Integração pertence à frente de loft/contexto, com
referências/guardas de rebase e seleção por proveniência. Tetos/Scene/replay,
árvore salva/main/demo inalterados; não converter testes de kernel em alegação
de encaixes unidos ou árvore orgânica concluída. EAF-85/EAF-80 seguem abertos.


## EAF-80/EAF-85 — composição viva da trajetória, local66

Integração de8f23990/979ff26 como b2adff8/6fba57a sobre1d36a65. Scene66/replay70
LOCAIS, macOS Apple Silicon/Metal headless, build729c105081544a5d. Main, demo,
janela e branch de malhas preservados.794testes Rust/75grupos,fmt/Clippy-Dwarnings/
build/py_compile/diff e78MCP/318grupos passaram. Logs plant-assembly-{all-tests,
clippy,build,full-mcp}.log; report em artifacts/mcp-validation/report.json.
Audit plant-assembly-build-audit.json:60summaries fornecem build_id e todos são
729c105081544a5d;18summaries legados omitem o campo. Runner inteiro usou o mesmo
executável estável. Não afirmar IDs ausentes como lidos.

Composição regenerável de três portas sem bake, fontes e referência históricas
mantidas, corte pela identidade030 (socket retido). Preview_scene_edits compartilha
staging da edição: prevê extração/remoção temporária/instalação sem tocar execução,
histórico ou seleção; publicação e previsão iguais. Novos testes cobrem fonte
esculpida1mm, domínio/pesos/bones/clipes/rig, prefixo retido, fonte oculta maior,
versões, recusa de rebase, erro atômico, referência obsoleta e undo. Pipeline
nativeTree cobre A/B/volta exata, outros assets/instâncias/poses0/60 e contexto
mundo com erro1,2645e−7m. save/load/replay/prefixo/repeat PNGs exatos.

Somente teto por asset4096v8192t→8192v16384t. Totais16384v32768t e draw131072
mantidos; hipótese de duplicar globais foi abandonada após medir16208v26326t.
Remover temporário antes de compor evita pico intermediário indevido. Torus
fechado8192v16384t prova max index8191, armazenamento total com dois assets,
GPU131072t/393216v, overflow por asset/documento, gate65→66 e persistência/replay.
Grid/curvas mantêm limites paramétricos próprios; não prometer todos os formatos
no limite de transporte com todos os canais.

Estudo mantido da MESMA árvore:10assets32instâncias,16208v26326t armazenados,
74870t desenhados. Conjunto7357v14710t (2031contexto+1361corpo+3965novos),ramo
[8,8,8],cell37,5mm/52920pontos. Colares32/32/32→99/56/63; contorno extraído128
limita DP33×129. Pico59,7362°(prova grosseira)→38,8196°,zipper31,0997°,média
2,78647°,source seam1,15059°.684pontos iso/envelope12,5mm movem até11,0724mm;
volume fechado total1,749515176→1,749468411 (não garantia local). Resíduo do campo
nos pontos iso antes5,76e−8m/pós4,2077mm, sem inferir Hausdorff. Domínio: guarda
copia fonte, salto3,59375→0,CG finito64/resíduo pós-f320,00534755unidades;
sem afirmar convergência exata ou gradientes físicos. Veios ainda esticam.

Fonte nativa examples/native-tree-study.scene.json byte-exata ao save MCP,
2669718B/SHA2567c667772d32864ace1c8b51335f60aa91f666b392baea7bfea51895407ec5b3c.
Capturas: artifacts/tree-assembly/artifacts/{before,after}-{branch,profile,back,
whole}.png,structure/neutral e report.json. Animação tree.gif2486063B,48frames
nativos distintos/4s; PNGs de todos os frames e poses de TODOS os32objetos após
240ticks iguais ao início. Gerada com Python/Pillow do runtime disponível,
nenhuma imagem gerada substitui a geometria. Binário corrente target/release/ignis.

Este é estudo corrente, NÃO árvore realista concluída: apenas o ramo editável e
sua bifurcação recebem vínculo regenerável. Outros galhos, escultura autoral no
miolo novo e gradientes da casca continuam abertos. Fonte completa do contexto
retém dados cortados; não transferimos implicitamente detalhe dessa superfície
para a região nova. EAF-80/EAF-85 permanecem In Progress. Capturas offscreen não
comprovam UI/teclado nativo nem aceite artístico. Contrato/custos: plant-assemblies.md.
# Guia de reconstrução e análise da casa — EAF-100

Unidade documental na branch codex/image-to-3d, posterior ao comparador EAF-99.
Skill no repositório, contrato de análise e exemplo com 30 entradas, hierarquia,
relações e motivo ramificado. AGENTS/architecture apontam para o guia. O ensaio
anterior foi identificado como estruturalmente insuficiente, inclusive sem
textura/cor; rótulo corrigido no gerador e nos outputs atuais HTML/report.

Validação: quick_validate.py aprovou frontmatter/nome/descrição; oito links
locais resolvem, YAML de interface válido, nenhuma marca de template restante.
Imagem original 1536×1024 e SHA256 intactos; 13 regiões dentro dos limites;
30 IDs únicos no inventário e referências A resolvidas. Regiões/relações foram
revisadas visualmente sobre a referência; classificação de apoios, material,
telhas e pontas dos motivos permanece explícita como dúvida/hipótese onde cabe.
Sintaxe AST do gerador e git diff --check aprovados. Relatório estável em
artifacts/skill-validation/report.json. O Python padrão/bundled não tinha PyYAML;
PyYAML6.0.3 foi instalado somente em artifacts/skill-validation/python para o
validador, sem nova dependência do produto ou ambiente global.

Sem código Rust, algoritmo, renderização ou geometria novos: nenhuma suíte
Rust/MCP completa repetida, build corrente e PNGs preservados. As verificações
de integridade documental não validam interpretação semântica ou fidelidade 3D.
Ainda não há solver de câmera, medidas de silhueta/landmarks ou distribuição da
skill por recurso MCP; entrega atual pelo repositório. Não houve aceite da casa.

## Reconstrução usando skill, imagem e MCP — EAF-101

Receita/scripts, análise e limites em house-skill-experiment.md. Cena autoral
examples/house-skill.scene.json salva pelo MCP. Nenhuma fonte da engine ou
modelo/análise anteriores foi carregado para construir; contexto prévio não
pode ser apagado, portanto não alegar teste cego independente.

Build existente9860a09115365ad3, Metal Apple M4 Pro. 16assets,32instâncias,
16090v/21818t armazenados,32272t desenhados. Nenhum budget alterado;17ºasset
recusado e cena/revisão preservadas. Captura repetida, undo de materiais/cores
neutros, save/load e captura do replay estático0ticks: igualdade exata de pixels.
run_replay matched:true inclui meshes/geradores; todos os IDs de correspondência
resolvem. Dois processos novos produziram cena e PNG idênticos. Resultados no
destino estável artifacts/house-skill/report.json; referência original por hash,
schema MCP, câmera, regiões e contagens registrados. node --check e diff check
passaram; nenhum Rust/renderer mudou e nenhuma regressão geral/build foi repetido.

MAE global0.166730/SSIM0.399808 incluem fundo. SSIM local varanda0.056368,
entrada0.072197,bay0.100926; resultado visual ainda distante da referência.
Métricas não são aceite geométrico. Perfis/encaixes/motivos/materiais/vegetação
simplificados, câmera sem calibração, interior/verso/colisão/manifold abertos.
Página local com lado a lado/sobreposição, forma neutra e vista elevada; sem
janela da demo ou controle de sessão de outro usuário. EAF-101 mantém gate de
avaliação visual; unidade técnica não conclui a reconstrução fiel da casa.

### EAF-101 — skill com análise aprofundada por recortes

Direcionamento posterior do usuário: evoluir a ferramenta/processo; não corrigir
o modelo desta casa. Bay window confirmada correta no modelo. Implementação
deste incremento restrita à skill, contrato e documentação. Nova sequência:
análise global → fichas por característica → inspeção de recortes reais e
sub-recortes motivados → atualização do inventário/plano → comparação local
com métricas e diagnóstico visual → revisão de contexto e conjunto.

O procedimento detail-review.md especifica proveniência/coordenadas, cobertura,
perguntas de aprofundamento, limite de resolução/oclusão, relação entre ficha e
objeto, diagnósticos acionáveis e critérios de encerramento. Recorte fixo para
posição/proporção e par normalizado para desenho interno são diagnósticos
distintos. ROI de compare_images não é apresentada como ferramenta de recorte;
capacidade ausente e SSIM sem suporte devem permanecer explícitos.

Validação documental: quick_validate.py aprovado; YAML de interface válido,
descrição com comprimento permitido e invocação da skill no default_prompt;
12links locais resolvidos; git diff --check aprovado. PNG original preservado,
SHA256209cd5cb91c216100413d6b175b5ce29a70cab751a0eda710e4baf95895c6553.
Revisão do procedimento cobriu relações que exigem contexto, detalhe sem pixels
suficientes, peça fora da ROI e divergência entre score e observação visual;
essa revisão é documental, não uma execução de reconstrução.

Receita, cena, capturas existentes, código Rust/renderer/MCP e build preservados.
Nenhum recorte ou análise aprofundada da casa foi executado neste incremento;
não houve render novo, regressão Rust/MCP, ensaio independente ou demonstração
de melhora da fidelidade. Próxima avaliação deve exercitar o procedimento com
imagem e ferramentas permitidas, distinguindo aplicação da skill de aceitação
visual. Checklist proporcional em performance.md; commit local, sem push.

## E02 com análise aprofundada por recortes — EAF-102

Fonte E02.png1402×1122 do projeto Unreal, original preservado; cópia versionada
docs/references/image-to-3d/e02.png, SHA2564153a456f9fdf2e9e6f43328171692156db7563c2062654199e66c0c540882ad.
Skill/contratos atualizados aplicados sem carregar receita/cena/análise da casa
anterior ou fontes Rust para modelar. Contexto prévio existe: não é teste cego.
Análise, hipóteses revistas, reprodução e custos em docs/e02-reconstruction.md;
18fichas,5sub-recortes efetivamente abertos e diagnóstico por característica em
e02-review.json. Recortes via Pillow12.3.0 do runtime local, sem resize/geração.
Publicação/captura/material/undo/persistência/replay somente por engine_command.

Cliente scripts/reconstruct_e02.py, extração/relatório scripts/e02_evidence.py e
template scripts/e02_report.html. Cena examples/house-e02.scene.json é cópia exata
do save_scene MCP. Build9860a09115365ad3 reutilizado, Metal Apple M4 Pro/macOS,
Scene67/replay71. Planta de coluna por9estações,80v/68t e5instâncias; galeria
compartilhada6vezes; demais geometria/perfis/motivos no cliente. Final16assets,
25instâncias,16218v/18512t armazenados,166v restantes. Nenhum limite ampliado.

Hipótese inicial avançava sobre o pátio, ocultava entrada/arcos e colocava a ala
paralela à fachada alta. Recortes orientaram recuo, transformação oblíqua da
ala, recesso da porta, cobertura longitudinal, abertura/altura da balaustrada
e medalhão. A/B recapturado com câmera/viewport/luz comparáveis;18regiões nativas
em lotes16+2, nenhum alinhamento/máscara/resize ou score composto. MAE global
0.179949→0.179525, SSIM global0.391281→0.376478. Algumas regiões melhoraram,
outras pioraram; **não houve evidência de ganho global de fidelidade**. Forma,
junções, perfil dos telhados, motivos, vidro/interior e plantas continuam muito
simplificados. Casa não recebeu aceite artístico; F17 ainda tem correspondência
local deslocada e diagnóstico inconclusivo para o motivo isolado.

Validação executada:

- Repetição de captura, undo após alterar aparência, save/load, replay estático
  **0ticks**, retorno da câmera auxiliar e carga em processo novo: pixels exatos.
  PNG final SHA25670d573701fd3d889ff698dedc826174f67f1b02078eafac4654ad8bb811b8712.
- Replay matched:true; referências de assets das18fichas resolvem. Arrays da
  cena exportada com posições finitas, índices válidos e limites por asset/global.
- AST dos dois scripts Python, node --check no JS da página gerada,18IDs/fichas
  únicos,5vínculos de sub-recortes,18regiões métricas e8recursos HTML locais válidos.
  Fonte por hash, exportação exata e git diff --check aprovados.
- A tentativa de inspeção da página file:// foi bloqueada pela política do
  navegador. Nenhum desvio foi tentado; somente revisão estática de sintaxe,
  dados/recursos e inspeção direta dos PNGs. Não alegar teste de interação da
  página. Solicitação de abertura pelo painel retornou queued.

Artefatos correntes em artifacts/house-e02/ (index.html, report.json, evidence.json,
pares, mapa de erro, forma neutra, vistas e verificação). Processos próprios
headless encerrados, demo e ensaio anterior preservados. Sem código da engine,
build, instalação ou regressão Rust geral; validação não cobre gameplay, teclado,
colisão, manifold, interior ou verso. Um bug de acompanhamento de revisão no
cliente foi corrigido: capture_replay retorna revisão histórica, que não pode
substituir a viva. O servidor recusou o comando obsoleto antes da mutação.
EAF-102 mantém avaliação visual pendente; commit local sem push.

## EAF-102 — E02 a partir dos 248 itens consolidados

Nova tentativa isolada em `artifacts/house-e02-rebuild/`, sem importar geometria
ou receitas dos ensaios anteriores. Fontes: referência E02, skill/lista763ba8d,
recortes e schema público MCP. Receita e cena atual versionadas; análise base
preservada. [Relatório](e02-rebuild.md) inclui revisão estática/iterações e
[registro por item](references/image-to-3d/e02-rebuild-progress.json).

MCP offscreen próprio, build9860a09115365ad3, Scene67:16assets/30instâncias,
16255v/18450t armazenados. Fonte nativa de coluna104v/92t×4; demais peças em
meshes indexados, maciço de folhas instanciado12vezes. Blueprint = projeções no
cliente de geometria MCP, não novo exportador da engine.

Captura repetida, save/load em processo novo, undo de câmera e replay estático0
reproduzem exatamente o PNG final SHA256 eabff512c0e70c8cc61cb33fd7144bcd97537bb1211de3ecbcc9da46df328bc2. Não são testes de física/manifold/jogo.
Comparação nativa em28regiões fixas: MAE0.165061, RMSE0.229390, SSIM0.396895,
sem alinhamento/ajuste de cor. Pares locais e vista auxiliar abertos; não existe
aceite de fidelidade.231itens com representação parcial/17sem construção,
0artisticamente validados; vidro/luz local/acabamento/densidade/fechamentos
pendentes. A lista evita omissões de famílias, mas não prova que os subdetalhes
foram reproduzidos. Sem Rust/build novo, mudança de tetos, push ou controle da
demo. EAF-102 permanece In Progress.

## EAF-102 — diferenças e inspeção orbital

[Revisão específica](e02-differences.md) e18achados estruturados, com IDs/evidência,
separam análise ambígua/insuficiente de requisitos já descritos e mal executados.
Capturas MCP próprias no verso/lateral/superior, removendo apenas o painel de
estúdio da cópia efêmera: superfícies desconectadas e fechamentos incompletos.
A vista auxiliar anterior não provava coerência volumétrica e foi insuficiente.

Cena/receita/PNG comparativo preservados. Visualizador WebGL de inspeção com
órbita/zoom/pan/arestas/home/vistas, sem dependências externas; não é o renderer
nativo nem um novo controle da janela Ignis. Node verifica matemática/limites,
17953v/19188t expandidos, índices,18seleções e recursos; PythonAST/JSsyntax/hashes
passam. WebGL e interação real não testados; não houve automação alternativa de
browser. Kernel/build/skill sem alteração, sem push. EAF-102 segue In Progress.


## EAF-85 — kernel puro de suavização por região

Novo módulo mesh_surface, API/tipos públicos e testes; sem enum MeshEdit,
dispatcher/plantas ou migração.794testes Rust/73grupos release/all-targets,
Clippy-Dwarnings,fmt/diff passaram em macOS Apple Silicon.11testes focados
cobrem máscara/âncoras/bordas/envelope,ilhas próximas não soldadas,faces
incidentes com vizinho fora da seleção,winding/arestas/estrelas não-manifold,
degeneração/inversão/piora de qualidade,volume fechado explicitamente reduzido,
rig/canais e poses fixas/socket0/60 exatos. Fonte imutável; fixos preservam bits
inclusive-0; região totalmente fixa retorna Arc compartilhado. Contrato/
checklist/custos em mesh-surface-filter.md/performance.md, logs estáveis
artifacts/mesh-surface/{tests,all-tests,clippy}.log.

Nova operação ainda sem exposição MCP/undo/persistência ou prova visual da
árvore nesta unidade. Integração pertence à frente de loft/contexto, com
referências/guardas de rebase e seleção por proveniência. Tetos/Scene/replay,
árvore salva/main/demo inalterados; não converter testes de kernel em alegação
de encaixes unidos ou árvore orgânica concluída. EAF-85/EAF-80 seguem abertos.
## Curvas de anéis e acabamento canônico — Scene67/replay71

Validação macOS aarch64 release, build `f0fdc4fc8798fadf`:

- `cargo test --locked --release --all-targets`:809 testes/77 grupos, aprovados.
- `cargo clippy --locked --release --all-targets -- -D warnings`,fmt,diff e
  py_compile dos scripts alterados:aprovados.
- Regressão MCP completa headless:79cenários/321grupos de checks aprovados.
 61resumos expõem esse mesmo build_id;18legados omitem o campo. Nenhuma janela
 ou controle da demo. Audit em artifacts/organic-build-audit.json, logs em
 artifacts/organic-{all-tests,clippy,build,full-mcp}.log e mcp-validation/report.json.

Novos kernels: loft cúbico/linear de anéis com bound relativo ao loft linear,
fonte/domínio/rig retidos e guardas de célula; filtro de superfície com seleção,
pinos, faces incidentes, envelope e relatórios de ângulos/volume. A consulta
MCP e a edição revisionada têm preview puro, publicação igual à previsão,
undo/redo/cache, skinning, save/load/replay/prefixos de pixels nativos. A prova
do pico selecionado está em mesh-surface-tools-verification.json. O fixture
Rust de composição comprova máscara canônica propagada pelo loft, relevo
não zero, acabamento de contexto/iso e regeneração sem trocar fontes/rig.

Armazenamento: torus fechado16384v32768t, dois assets total32768v65536t,
GPU131072t. Fronteiras/overflow e gate66→67 aprovados, também para fontes
canônicas privadas. Fixtures de orçamento foram movidos ao novo limite real:
grade63 refina para16129v e recusa o próximo nível; cópia após8grades4096v
e refino com quatro assets7921v excedem o total. Guardas de geometria não foram
relaxadas para aprovar essas provas.

MESMA árvore, `examples/native-tree-study.scene.json`,3109356bytes,
SHA256`e999a2090ab650850793d2605ddd7f11782f430fc5affd46c5df7891361af75a`:
19816v33542t armazenados,82086t desenhados,10assets32instâncias. Refinados
21tubos de wood e secondary-06, mais o pescoço canônico da composição; fontes
da planta/referência/contexto originais intactas, clips/bones preservados e
poses dos demais assets/descendentes0/60exatas. Ciclos de curva, preview de
lote, undo, save/load e replay nativos aprovados. Reproduzir com
`python3 scripts/demo_tree_ring_lofts.py`; capturas neutras e com material,
mesmas câmeras/luz/tick60, em artifacts/tree-surface/artifacts.

Contexto pós-loft: seleção1952/pinos160/móveis1792,12×.3/envelope.03m;
região56.213948→27.851475°,média8.583096→7.287621°,movimento30.00004mm
(tolerância f32). Volume algébrico do componente fechado1.691058845→1.672979076,
aproximadamente−1.07%;não conservação local. A tampa distal excluída mantém
quinas de94° e não participa desse máximo regional. Acabamento iso4×.1/.01m:
38.819615→37.247076°,média2.757778→2.741331°,2.1285mm,fonte/guardasfixas.
Relatório da união refere-se à etapa pré-filtro e não foi substituído pelo
relatório final de acabamento.

**Pendente:** o ressalto de peça atravessada e outras junções não foram unidos
por este filtro. Ramos estáticos refinados não são um grafo inteiro de plantas;
casca/gradientes, ponta e qualidade artística ainda abertos. EAF-80/EAF-85
continuam In Progress. A prova acima não é aceite humano/UI, nem instalação
na demo ou integração de formatos na main funcional.

## Enxerto estático regenerável — Scene68/replay72 locais

Validação em codex/native-plants, macOS Apple Silicon/Metal, sem main/push ou
controle da demo. Build `aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|a70f0a1fe9dcecae`.

- `cargo test --locked --release --all-targets`: **814testes/78grupos**, sem falhas.
- `cargo clippy --locked --release --all-targets -- -D warnings`, fmt, py_compile e diff check passaram.
- `scripts/run_verifications.py`: **79cenários/322grupos MCP** headless passaram.
  Audit61summaries com mesmo build_id;18legados omitem esse campo. Cenário
  plant_assemblies reexecutado após ampliar a prova de mudança do loft pai:4grupos passaram.
- Logs estáveis `artifacts/graft-{all-tests,clippy,full-mcp,parent-mcp}.log`,
  auditoria `artifacts/graft-build-audit.json` e report `artifacts/mcp-validation/report.json`.

Cinco testes novos em `tests/context_grafts.rs`: fonte filha com detalhe autoral
nãozero1mm em ponto retido, translação f32/domínio material bit-exatos, bones/clips
originais e componente fechado; preview/publicação/retorno paramétrico/undo/redo;
mudança da discretização do loft pai3→4→3 sem índices efêmeros;12classes de
recusa em preview/execução com estado/tick preservados; aparência sem rebase;
fontes ocultas integrais inválidas, IDs repetidos e orçamento total recusados.
A fonte com rig na recusa é um Asset válido antes da guarda de fonte estática.
O cenário MCP inclui10classes adicionais de recusa de enxerto e mudança do loft
pai, além dos ciclos/save/load/capturas/replay/prefixo da assembly anterior.

Fonte mantida `examples/native-tree-study.scene.json`: **3488170bytes**, SHA256
`2d371b81ec51bb022eb9c3d8777a8d5a1daecc58c1f254935a4a70961be57b97`,
22109v/38132t armazenados,86676t draw,10assets/32instâncias. Repro nativo
`scripts/demo_tree_graft.py` parte do snapshot fixo7ee07c6, extrai119v/234t por
preview Move nativo e publica wood sem esse filho junto à receita em um lote.
Wood177→176ilhas; composição com filho é1componente fechado. Fontes anteriores,
contexto/ref originais, bones/clips e demais assets/instâncias/poses0/60/240
permanecem exatos. Fonte filha integral119v guardada;91v retidos com translação
explícita, nenhum detalhe de faces cortadas transferido implicitamente.

Portas32/32/7 e grade39527; guarda8mm/colar30mm, seam4×/8mm, domínio64passos.
União PRÉ-acabamento adiciona2577v/5227t,iso2506v/4865t;costura fonte2.8630°,
pico51.8635°,média4.54149°. Residual material8.37486e−4 é finito, não prova de
veios físicos. Filtro iso16×.3/envelope20mm: REGIÃO48.35424→41.22771°,
média4.498495→4.358142°,máximo movimento9.98206mm;volume algébrico fechado
1.690421300→1.688771023(−0.098%,não conservação física/local). O pico51.86°
pré-filtro usa outra região e NÃO deve ser apresentado como reduzido a41.23°.
Fonte e primeira guarda permanecem fixas; tampa distal antiga94° continua fora
desse recorte. Guardas/grade/tetos não foram relaxados.

Capturas nativas `artifacts/tree-graft/artifacts/{before,after}-{neck,profile,fork,whole}.png`
e variantes `neutral-`, mesmas câmeras/luz/tick60. A tampa atravessada foi removida.
Ainda há transição de curvatura/iluminação em anel e domínio de casca distorcido;
outros galhos se atravessam. Lista endereça somente contexto canônico+derivados
do loft, sem grafo recursivo ou transferência multibone do secundário06. Realismo,
continuidade analítica perfeita/UI/aceite humano seguem abertos. EAF-80/EAF-85
permanecem In Progress.

## Continuidade do colar — Scene69/replay73 locais

Validação completa no build
`aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|31d020edf874736d`,
macOS Apple Silicon/Metal, codex/native-plants. Sem main/push ou controle da demo.

- **819testes/78grupos Rust** release/all-targets, Clippy-Dwarnings, fmt,
  build, py_compile e diff check passaram.
- **79cenários/323grupos MCP** headless passaram. Audit61summaries com mesmo
  build_id e18legados sem campo. Relatório/audit e logs estáveis em
  artifacts/collar-{all-tests,clippy,build,full-mcp,build-audit}.* e
  artifacts/mcp-validation/report.json.
- Cinco testes novos: região BFS exata/âncoras/multi-source/ordem independente;
  ilhas coincidentes por posição não se unem, parâmetros inválidos/componente
  sem camadas externas recusados; pós-filtro intencional com fontes/canais/rig
  exatos e legado68 sem campo; oito recusas de colar em preview/execução; fixture
  nativo privado que põe uma porta NÃO escolhida dentro da região móvel e prova
  sua proteção com as primeiras guardas enquanto outros pontos se movem.
- MCP plant_assemblies acrescenta diagnóstico por origem e seis classes de
  recusa do colar, além de preservar mudança do loft pai3→4→3, trajetória,
  previews, undo/redo, save/load/replay/prefixo/PNG.

Mesma fonte nativa3488335bytes, SHA256
`89fe1fa2112cbb8a2d3ae266a53cf0d6308c8b371d356a8f5b1b05a208789400`;
22109v/38132t armazenados,86676t draw,10assets32instâncias. Tetos/índices/renderer
sem alteração. Repro scripts/demo_tree_collar.py parte do snapshot fixob1058b5.
Capturas antes/depois, neutras/com casca, mesmas câmeras/luz/tick60 em
artifacts/tree-collar. Native scene salva/recarregada exata, imagens neutras de
pescoço/perfil mudaram, live/repeat/replay coincidem em bytes.

Fontes da planta,contexto e filha/ref originais,triângulos/domínios/pesos/bones/
clipes/instâncias e demais assets ficaram exatos. Poses dos outros objetos e
joints/world_transform/clip_tick do alvo são iguais em0/60/240. Os765pontos
remodelados mudam intencionalmente; não declarar preservação visual exata de
escultura/pose dentro da região. Dados originais permanecem disponíveis para
regeneração/undo.

Colar: ports[0,1],rings6,fixed_rings2,4×.3/envelope10mm. Região1154v,
389pins,765móveis:320retidos do pai+445gerados(381iso+64guardas),filha0.
Pico REGIONAL41.227714→23.993567°,média6.010326→5.801218°,0dobras na região;
maxmove10.0000326mm(dentro tol.f321e−6m),RMS4.107423mm. Volume algébrico do
componente fechado1.688771023→1.688032324(~−0.044%,não volume local conservado).
Todas as posições alteradas do asset final correspondem aos765pontos dessa
camada. Fonte e âncoras permanecem exatas; relatório da união e do filtro iso
são anteriores ao colar. Não comparar o pico51.86° da união crua com23.99°
como se fossem a mesma região. A tampa antiga94° continua fora do recorte.

A revisão independente somente leitura não identificou bloqueador no contrato/
seleção/mapeamento/gates; não substitui os testes acima nem aceite da árvore toda.
O anel atenuou nas vistas frontal/lateral; casca distorcida, outras interseções,
folhas e skin de junções animadas ainda são gates abertos. EAF-80/EAF-85 seguem
In Progress. A frente complementar de casca assumiu o diagnóstico de domínio
material; este incremento não muda esse domínio nem prometeC1/C2/realismo final.

## Integração69 das consultas de domínio material

Integrado por patch o commit41be11d sobre e1fef8f, sem incorporar formatos antigos
ou substituir a árvore. Scene69/replay73 mantidos; nenhuma alteração geométrica,
material,renderer ou fonte. Build local
`aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|3b60bd71429184f0`.

826testes Rust/79grupos release/all-targets,Clippy-Dwarnings/fmt/build/py_compile/
diff passaram. Oito dezenas de cenários MCP(**80cenários/326grupos**) passaram
headless, incluindo o novo mesh_material_domain e a assembly69 com colar. Logs
estáveis artifacts/material-query-{focused,all-tests,clippy,build,full-mcp,study}.log,
report do runner e audit de build. O estudo nativo foi repetido nesta integração,
incluindo `--load-after`: load69 aprovado no processo isolado e relatório por
asset igual ao preview da Geometry, com cena/revisão/tick estáveis durante as
consultas. Fontes mantidas com SHA89fe1fa2…9400,sem alterar as capturas anteriores.

A consulta não corrige a casca. Foram confirmados142/142colapsos na primeira
guarda local e quadros materiais diferentes entre pai/filha. Contrato da
próxima prova em material-continuity-study.md, ainda sem persistência/finisher
integrado. O candidato harmônico isolado da outra frente melhora guarda/iso,
mas piora zipper; não foi promovido à árvore mantida. Não confundir esse
candidato com os80cenários aprovados desta unidade de diagnóstico.

## EAF-88 — investigação do platô próximo

Laboratório isolado em codex/graphics-research; evidência/reprodução em `docs/near-plateau-study.md`. Geometria/depth e material isolados, timestamps sobrepostos documentados; preparo de posições compactas e ordem espacial conserva pixelsRGBA32F.64pontos de tempo+32repetidos,112poses/560gates/336igualdades exatas,672PNGs/240controles anteriores/retorno idênticos. 633testes Rust release/all-targets, Clippy/fmt, render/replay_inspection/selection headless e560casosJS/DPR/playback aprovados. Candidato100× próximo ganha6,93%/5,90% total, mas ainda cresce~3× entre1× e100× e usa muita memória adicional no comparador. Continua experimental; perdas preservadas. Diagnóstico64coortes expressa área em pixels físicos:100×é relativo a32.768triângulos, não100triângulos/pixel; relevo também acrescenta frequências. Meta de platô próximo/aceite humano permanece aberta; runtime/contratos/demo não alterados.

## EAF-88 — densidade física com superfície detalhada fixa

Incremento isolado na worktree `codex/graphics-research`; protocolo, limitações e comandos em `docs/pixel-density-study.md`. Aumentar apenas subdivisão baricêntrica mantendo parental f1_d2/material/normais. Metas 1/2/10/100 triângulos fonte por pixel coberto a distância 14: 2.263.908 a 226.390.700 triângulos reais no nativo. Render 3456×2234 e 1728×1117; mesmas cinco poses 12→14→16→14→12, duas rodadas de timing a14 com duplicatas/8 blocos.

636 testes Rust release/all-targets, Clippy-Dwarnings, fmt/diff-check e render/replay_inspection/selection headless (13 grupos) aprovados. 80 gates de qualidade passaram; seleção certificada igual à referência RGBA32F nas 40 poses. 120 PNGs íntegros, mesma referência entre densidades e retorno exato. Fallback com certificado inválido produz imagem/contagem densa. Hashes da fonte/normais/material/depth preservados; caixa expandida inteira no frustum da pose medida, sem prova de oclusão individual.

Curva nativa total certificada 0,42587/0,42466/0,42901/0,42593 ms contra completa 1,24878/2,00564/6,54413/46,10117 ms; seleção envia 327.680 triângulos em todos os pontos. Certificado vem da construção, não de simplificador genérico; platô não generaliza a conteúdo novo. Maior fonte exige 3,94 GiB e 2,45 s de preparo/upload, mantidos fora do frame e explícitos. Duas rodadas no mesmo processo; controles e intervalos no relatório, sem promessa de regressão zero.

80 casos JS em DOM simulado validam relatório/DPR/crops/reprodução/carregamento obsoleto; recortes 512² inspecionados. Não comprova browser nativo, movimento contínuo ou aceite visual humano. Artefato corrente `artifacts/appearance-probe/pixel-density/index.html`, dados raw/verificações no mesmo diretório. Densas só em memória durante execução; build corrente externo reutilizado. Sem alteração de runtime/NPC/Scene/Command/RenderFrame/MCP/demo.

## EAF-88 — símbolo diretamente renderizável na pedra

Experimento autorizado27/10/2026, isolado em `codex/graphics-research`; documentação/reprodução em `docs/rock-symbol-study.md`. Pedra Blender procedural existente com relevo nas posições,81.920triângulos/40.962vértices. Agrupamento espacial/orientação em1.985/6.428/19.143surfels de48bytes; iluminaçãoGGX compartilhada dinâmica, suporte de discos sem reconstrução da malha fonte. Caminhos próximo e ponderado3passes por escala, sem fallback oculto.

162poses (duas resoluções físicas, distâncias4/24/96, movimentoscâmera/luz/combinado, noveframes),972comparações: **nenhum candidato passou em todos os gates de qualidade**. Grade64ponderado reduzpayload2,50→0,876MiB, mas perde microrelevo/contorno e acrescenta target58,90MiB no nativo. Tempo total nativo/perto na posecombinada0:malha0,2548ms versus0,5895ms candidato; regressão em todos os controles pareados da variante maisfiel. Algumas variantesmaisgrossas reduzemtempo com errovisual grande. Referência64spp emseiscontroles mostra possível redução de aliasing de cor muito longe (nativo96:RMSE0,02880malha1spp versus0,02021candidato), sem aprovação de cobertura/próximo.

639testesRust,Clippy/fmt/diff-check,13gruposMCPheadless render/replay_inspection/selection e972casosJS/DPR/playback/staleload passaram.1.140PNGs íntegros, hashes/payloads conferidos,18coortes+6repetições comduplicatas/seterodadas/32frames. Falha de timestamp intermediário foi corrigida antes da coleta final; imagens/shader/payload inalterados na retomada. Duplicatas variam0,699–1,510, logo diferençaspequenas são inconclusivas. Wrap/luz/retorno controlado exato;34fechamentosPNG combinados diferem pela elevaçãof32 também nareferência, explicitados. Recortes512² mostramperda de detalhe; sem browsernativo/vídeocontínuo/aceitehumano.

Relatório corrente `artifacts/appearance-probe/rock-symbol/index.html`, dadosraw/verificadores no mesmo diretório. Buildcorrente externo reutilizado; fonteBlender, contratos runtime/NPC/Scene/Command/RenderFrame/MCP e demo preservados. Resultado encerra o teste deste símbolo simples, não a pesquisa de representação nem a hipótese geral de renderizar dados comprimidos.

## Pedra compilada — EAF-88,27/10/2026

Pesquisa isolada em `codex/graphics-research`: compilador binário `.irk`, mapas de profundidade/códigos em blocos e respostas de material/luz pré-calculadas. CLIbuild separado do loader; consumidor carregou apenas binários em diretório temporário sem fonte/JSON, produzindo as quatro imagens idênticas ao comparador. Arquivos0,753/5,669MiB para fonte2,500MiB; representação direta/filtrada dos mesmos dados, um passe. Critérios de qualidade mantidos:0/648aprovações em162poses. Isso é conclusão experimental negativa de fidelidade, não bug ocultado como sucesso nem renderer pronto.

642testes Rust `--locked --release --all-targets`, Clippy `-D warnings`, fmt/diff-check;13grupos headless render/replay_inspection/selection.820PNG com dimensões/CRC/descompressão,90retornos exatos, hashes/source, referências16/64spp em seis poses e24coortes de tempo. UI648combinações comDOMsimulado/DPR/crops/playback/loadsobsoletos; nenhum browser/aceite humano implícito. SetePNG do símbolo anterior bit-idênticos em smoke isolado, relatório antigo preservado. FramebufferRGBA32F e memória de captura são instrumentação explícita. Sem mudanças emruntime/NPC/Scene/Command/RenderFrame/MCP ou fonteBlender, sem demo/janela/publicação. Build externo corrente reutilizado.

Reprodução: `CARGO_TARGET_DIR=/Volumes/Zero/build-cache/ignis-graphics python3 scripts/run_compiled_rock.py`. Estudo e limitações em `docs/compiled-rock-study.md`; comparador corrente `artifacts/appearance-probe/compiled-rock/index.html`. Não demonstrado ganho preservando qualidade: filtrado perto~0,586ms vsfonte~0,238ms; direto~0,227ms com perda visual eGPU sem ganho próximo. Manter pesquisa/aceite geral abertos.

## EAF-88 — símbolos opcionais sobre o caminho anterior (27/10/2026)

Estudo isolado `docs/combined-symbol-study.md`; relatório `artifacts/appearance-probe/combined-symbol/index.html`.645testes Rust release/all-targets aprovados (três novos), Clippy `-- -D warnings`, fmt e13gruposMCP headless render/replay_inspection/selection.252poses/756comparações passaram gates anteriores; fallbackRGBA32F exato, resposta forçada com cobertura idêntica.764PNGs íntegros,84fechamentos e8baselines anteriores byte-idênticos;504estadosUI e recortes físicos emDPR1/1,5/2/3 viaDOMsimulado. Política congelada em0classes: sem ganho incremental robusto (forçado tempo total mediano+0,04%/+1,51%). Nenhum novo aceite humano/nativebrowser, integração de atlas geométrico, FPS ou economia deRSS alegados. Runtime/NPC/MCP/demo preservados, build corrente externo reutilizado.

## EAF-88 — campo geométrico binário opcional, 27/10/2026

`python3 scripts/run_geometric_field.py`; resultados correntes em `artifacts/appearance-probe/geometric-field/`, protocolo e limites em `docs/geometric-field-study.md`.648testesRust release/all-targets,Clippy-Dwarnings/fmt/diff;13gruposMCP headless (render/replay_inspection/selection) aprovados.864/864gates em288poses físicas, símbolo ativo196vezes noforçado; opcional conservaB135 com pixels idênticos.872PNGsCRC/dimensões/decompressão,96retornos,8baselineslegados iguais,18casosdepth (16ativos),24calibração+24validação+16repetição e4auditoriaGPU; política/inputs verificados. UI576estados eDPR1/1,5/2/3 porDOMsimulado, sem aceite humano/browsernativo.

Resultado negativo de performance:100×médio nativo .3484→.5202ms forçado; CPU_submit eGPU/janela medidos separadamente. Auditoria identificou sobreposição de intervalosMetal, corrigindo soma de latências para throughputGPU. MáximoRMSEforçado .0028473/cobertura .00013716, dentro dos gates; não pixels idênticos.100×próximo permaneceB por certificado conservador. Campoheightfield não resolve pedra fechada/multicamada nem oclusão dinâmica geral. Nenhum contrato/runtime/NPC alterado, build corrente reutilizado e demo preservada.

### EAF-88 — hierarquia geométrica por regiões/escalas (27/10/2026)

Experimento isolado em `docs/hierarchical-field-study.md`; reprodução `python3 scripts/run_hierarchical_field.py`. 651 testes Rust (`cargo test --locked --release --all-targets`), Clippy `-- -D warnings`, fmt e diff-check aprovados. Treze grupos MCP headless: render, replay_inspection, selection. Sem interação com demo/contratos funcionais.

288 poses em duas resoluções físicas, 864 gates aprovados, 872 PNGs íntegros, 96 retornos idênticos e oito baselines legados byte-idênticos. Smoke do caminho geométrico antigo: três PNGs byte-idênticos. Hierarquia ativa em 268 poses; 18 depth checks e 24 comparações de trabalho/qualidade E/D aprovados. Calibração 40 coortes, validação 24 e repetição 16, duplicatas e auditoria de timestamps. Política congelada máscara zero: opcional = B em RGBA32F; avanço de custo contra símbolo plano não é ganho contra B. Checksum Python corrigido (offset FNV digitado incorretamente), revalidado sem alterar dados medidos.

UI em DOM simulado: 576 estados e DPR 1/1,5/2/3; inspeção complementar de capturas, sem aceitação de navegador nativo/humana. Evidências correntes em `artifacts/appearance-probe/hierarchical-field/{verification,ui-verification,analysis,work,depth,legacy-smoke}.json` e logs; detalhes/limites e valores no estudo.

### EAF-88 — blocos de pixels e controle de conteúdo fixo (27/10/2026)

`docs/tiled-field-study.md`, runner `scripts/run_tiled_field.py`. 653 testes Rust release/all-targets, Clippy-Dwarnings/fmt/diff e13gruposMCP headless aprovados. Sem alteração funcional/demo. 288poses/864gates principais,48poses/240gates de superfície fixa;18checksdepth;872+16PNGs íntegros,96retornos,8baselines e6PNGs smoke legados idênticos. DOM576estados/DPR1/1,5/2/3; sem browser nativo/aceite humano.

Calibração40coortes, validação24, repetição16,4auditorias e16coortesfixas. Política congelada máscara0: C=B exato no conjunto principal. Corrigida a exigência herdada de que timestamps necessariamente se sobreponham; nesta auditoria inclusive controles não sobrepuseram. Intervalos e parede validados, dados brutos preservados. Harness fixo foi congelado/recalibrado antes da coleta final para usar as mesmas quatro poses por caminho. Fontes densas reais até254.467.400triângulos, uma por vez, liberadas ao terminar. Ganho incremental de blocos não demonstrado; platô de redundância conhecida deB confirmado. Evidências correntes em `artifacts/appearance-probe/tiled-field/`.

## EAF-88 — regiões coordenadas, 27/10/2026

Incremento isolado em codex/graphics-research; estudo `docs/coordinated-regions-study.md`, painel/dados em `artifacts/appearance-probe/coordinated-regions/`.683testesRust release/all-targets, Clippy-Dwarnings, fmt incluindo gpu.rs, diff-check e13gruposMCPheadless (render/replay_inspection/selection) aprovados. Novos testes de fronteiras mistas após uniões e normal interpolada curta/codec/zero; compartilhamento de módulos repete testes em exemplos.

144capturas em1/2/10/100× da Boulder01,1728×1117/3456×2234, perto/longe, órbita/luz e retorno;1.280amostras temporais alternadas/duplicadas,128frames/lote.409PNGs íntegros,80paresPNG de retorno exatos,144igualdadesRGBA32F direto/indireto. Controle de subdivisão versus1× falha4/144, todaslonge1728; recuperação/campo antigo/coordenado falham nas mesmas4. Outros140 passam; não declarar preservação visual universal, certificado fotométrico ou teste contínuo de cintilação.1.024estadosDOM/DPRsimulados e recortePNG inspecionado, sem navegador nativo/aceite humano.

Resultado negativo do incremento no pesado:100× próximo nativo total0,32944ms anterior→0,37518 coordenado→0,38711 indireto. GanhoGPU não pagaCPU/submissão; anterior continua mostrando grande ganho contra fonte densa6,86343ms. Não confundir essa economia anterior com ganho incremental. Fonte100× recupera265.105T em vez de66.122, próximo gargalo explícito. Nenhum runtime/contrato/demo alterado.21outputs de smoke supersedidos removidos após lsof,12pacotes de variantes atuais mantidos e hashes registrados; target/release corrente, sem push/publicação.

## EAF-88 — refit offline da recuperação, 27/10/2026

Estudo `docs/recovery-stability-study.md`, comparador em `artifacts/appearance-probe/recovery-stability/`, isolados emcodex/graphics-research.687testesRust release/all-targets,Clippy-Dwarnings,fmt/diff e13gruposMCPheadless(render/replay_inspection/selection) aprovados. Testes de fixture arredondado98→1T, relevo/normal não afim preservados, materiais, fechamento e ordem; quatro compilações legadas byte-idênticas. Nenhum contrato funcional/demo alterado.

Refit opt-in reduz265.105→67.139T no100×; build conserva anterior1/2/10×, pois não há melhoria estrita nesses casos.144poses físicas perto/longe/câmera/luz/retorno: escolhido passa contra pedra-base em144/144, mas mantém quatro falhas contra fonte subdividida longe1728. Raiof64 em quatro pixels encontra geometria/cor quase iguais entrebase/densa ou ausência emambas; não é referência integrada nem prova do mecanismo exato doMetal.100× RMSEmax contra base2,5202e-6, cobertura idêntica.320PNG íntegros,64paresPNG retorno idênticos,1.024estadosDOM/DPRsimulados,PNG próximo inspecionado; sem navegador nativo/aceite humano.

Coleta final após regressões:1.024amostras,128frames/lote, oito rodadas alternadas/duplicadas, bounds incluidos porframe.100× próximo3456 total0,33107→0,27138ms (−18%); sete/oito rodadas favorecem candidato.1×→100×0,26873→0,27138ms;2×0,28889ms residual. Longe nativo não demonstrou ganho. Fonte densa7,00728ms, mas o ganho incremental é18%, não toda a diferença contra fonte. Proxy7,107→2,201MiB, fonte163,484MiB ainda mantida. Dois outputs supersedidos removidos apóslsof; quatro pacotes correntes com hashes, target/release reutilizado. Sem push/publicação.


## EAF-88 — relevo e cores em múltiplas escalas, 27/10/2026

Experimento isolado `rich_relief_probe`, estudo `docs/rich-relief-study.md`, painel em `artifacts/appearance-probe/rich-relief/`. 712testesRust release/all-targets,Clippy-Dwarnings,fmt/diff e13gruposMCPheadless aprovados. Smoke normal_field_probe verifica caminho antigo com UV (16/16 campo/conservador; alternativa forçada sem campo mantém oito falhas explícitas). Mesma superfície entre pacotes verificada por remapeamento exato, seis pacotes byte-idênticos após reduzir transporte GPU a12B/vértice.

138poses em1728×1117/3456×2234, câmera/luz/aproximação/retorno;800amostras temporais,128frames/lote,8rodadas duplicadas. 690PNGs íntegros e2.208estadosDOM/DPRsimulados; captura próxima inspecionada, sem navegador nativo/aceite humano. Regional0,05passa138métricas e10retornos, mas não apresenta ganho consistente; reduz só0,98% no distante1728 e nada em3456. 0,15 e conservador passam métricas138/138, porém cada um falha um retorno (4pixelsPNG/max1 e2pixels/max6, respectivamente). Anterior falha1728/aproximação/frame14 RMSE0,00516107. Fonte10/10retornos exatos;48/50todos os caminhos. Não relaxar gates nem confundir esses resultados com o ganho anterior de redundância.

Normal/UV ausentes do buffer GPU porque o shader usa mapas/XY; mapas compartilhados26,667MiB. Memória fonte+mapas46,934MiB, fonte+níveis+mapas53,447MiB, semtargets/driver/CPU/controles. 50PNGs smoke supersedidos removidos após lsof; dados diagnósticos mantidos. Target/release reutilizado; engine funcional/NPC/contratos/demo preservados. Próximo foco: estabilidade dos controles e regiões que isolem fissuras sem sacrificar detalhe de material.


## EAF-88 — refinamento local e diagnóstico de repetibilidade, 27/10/2026

Continuação isolada de `rich_relief_probe`; estudo `docs/rich-relief-study.md`, painel `artifacts/appearance-probe/rich-relief/refined.html`.713testesRust release/all-targets,Clippy-Dwarnings,fmt/diff e13gruposMCPheadless aprovados. Novo teste exige preservar pico isolado/bordas/arestas e codec, reduzindo somente fora do detalhe e compartilhando intervalos idênticos.690novosPNGs íntegros,2.208estadosDOM/DPRincluindo falha da referência;2.208estados do painel antigo rechecados. Fonte do pacote refinado igual à original, materiais/shader preservados. Captura próxima inspecionada, sem aceite humano/navegador nativo.

Benchmark diagnóstico:138poses/800amostras,2resoluções/câmera/luz/retorno. Local0,05 e0,15passam138métricas e10retornos cada;anterior+hierarquia mantém uma falha fotométrica.48/50retornos totais:uniforme3456/perto falha(3pixelsPNG,max7/255),fonte3456/aproximação falha(4pixels,max2/255). O runner terminou com erro e preservou dados, painel avisa reprovação global e por trajetória. Não confundir testes de implementação aprovados com aprovação geral do experimento. Nas vistas estáticas afastadas com retornos aprovados há ganho:3456/distante−43,65%tempo total e−61,92%triângulos;perto não há ganho consistente. Memória70,261MiB versus53,447MiB uniformes,semtargets/driver/CPU/controles.

Diagnóstico separado:512capturas iniciais e768com ordem invertida. Implícito varia em algumas capturas;derivadas finas também falham,logo não foram adotadas. Mipfixo/UVestáveis nas amostras não estabelecem causa raiz.50PNGs smoke refinados supersedidos removidos após lsof;JSONdiagnóstico econtroles atuais preservados. Target/release reutilizado,engine/NPC/contratos/demo intactos,sem push.

## EAF-88 — custo automático no relevo rico e diagnóstico de filtragem

`docs/rich-relief-study.md`, `artifacts/appearance-probe/rich-relief/auto.html`. Modelo monotônico calibrado em seis vistas separadas por resolução, margem unilateral de superestimação+ruído e rejeição otimista dos certificados antes de144regiões. Custo dentro dos lotes; mesmos materiais/mapas/fonte.715testesRust release/all-targets,Clippy-Dwarnings,fmt/diff e13gruposMCPheadless(render/replay_inspection/selection) passaram.690PNGs íntegros,2.208estadosDOM/DPRautomático e2.208em cada painel anterior. Sem aceite humano/navegador nativo.

138comparações/800amostras/50retornos:automático138/138qualidade mas9/10retornos;fonte10/10,local0,05forçado10/10. Automático1728/perto varia1pixelPNG em1/255 usando anterior;exploratório1728/aproximação também1pixel/1nível. Anterior mantém1falha numérica. `--audit-auto` reprova com código1 sobre resultados preservados; runner automático passa pela mesma auditoria. Painel não transforma tempo menor em aprovação visual.

Ganho parcial distante3456 anterior0,353173→auto0,196798ms(−44,28%,8/8rodadas);1728−55,58%. Intermediário3456 auto0,382659 versus anterior0,374183 e localforçado0,271594:estimador conservador perde oportunidade. Perto usa mesma representação anterior; diferenças de timing não atribuídas ao algoritmo. Memória90,528MiB incluindo fallback versus70,261local;calibração inicial7,720/10,820s separada. Primeiro modelo/margem bilateral preservado em auto-conservative-diagnostic.json.

Diagnóstico dedicado768retornos:original1/256mudou,LOD explícito33/256mudaram;derivadasUV eUV128/128cada estáveis nas amostras. Não prova causa raiz nem correção; shader original mantido. Nenhuma alteração funcional/NPC/Scene/MCP/replay,janela da demo,push ou integração emprodução. EAF-88 permanece investigação.

# Preflight estrutural de reconstrução — cliente

Branch `codex/image-to-3d`, macOS, sem mudança de kernel/build/schema/cena.
Contrato/checklist: [reconstruction-structure.md](reconstruction-structure.md).
Auditor `scripts/reconstruction_structure.py` (stdlib) e testes
`scripts/test_reconstruction_structure.py`:18 aprovados com Python preexistente
contendo NumPy para importar/executar a receita. O Python padrão sem NumPy
executou a auditoria, mas não a integração com a receita; nenhum pacote instalado.

Contraprovas: cubo externo/volume1, seams exatos, gap fino sem correção,
face ausente/invertida, orientação interior, não manifold, duas ilhas fechadas,
superfície dupla plana/volume0, degeneração/isolados, translação/escala,
malformados, pureza/determinismo, CLI/exit0/1/2, assets reais da E02 e adorno
aberto sem rejeitar corpo fechado. Publicação E02: corpo ausente/inválido,
detalhe invalidando e weld invalidando recusados antes de Engine; execução
estrutural real em root temporário não gera PNG/cena/processo MCP.

Diagnóstico versionado em `docs/references/image-to-3d/e02-structure-audit.json`,
entrada SHA25689c6aa59af4b3142a6a4df18fc3feb9659aafda944d96a0e378aa96e21e94ff1.
Pedra2274bordas/141componentes; cobertura906/211; coluna56duplicados exatos,
0bordas/1componente/passou. Agrupamento por material não é identidade semântica.
Modelo/render preservados; PNG SHA256eabff512c0e70c8cc61cb33fd7144bcd97537bb1211de3ecbcc9da46df328bc2.

Skill encaminha procedimento antes de detalhes e distingue corpo, espaços
abertos e hipóteses do verso. Ainda não existe gate universal do dispatcher/MCP:
este bloqueio atua na receita cliente e o auditor não prova casa/fidelidade,
apoios, espessura ou ausência de autointerseção. Rust/regressão MCP não
reexecutados para cliente/skill; sem render, interação visual ou aceite novos.

## EAF-88 — texels explícitos e derivadas no mesmo desenho (27/10/2026)

Roadmap em `docs/graphics-roadmap.md`: consolidar seleção/estabilidade, compor cena e só depois experimentar amostragem guiada pela atmosfera. Proposta preserva geometria detalhada e reduz consultas por amostra; nuvens fora do primeiro recorte, mesma atmosfera nos controles, UI/MCP na integração. Nenhuma implementação atmosférica nesta unidade.

Diagnóstico em `docs/rich-relief-study.md`, comandos --texel-repeat/--inputs-repeat e auditoria `scripts/report_rich_relief_repeat.py`.1.536retornos fixos em duas resoluções e perto/longe. Sem MRT:original0/256falhas,LOD explícito1/256,manual3/256(até21pixels). Com MRT:original4/256,LOD explícito33/256,manual0/256; zero mudança de cor com derivadas finas iguais nos pixels afetados. A instrumentação altera execução: manual estável comMRT não corrige a reprovação semMRT. Causa raiz não concluída, shader/material/seleção principais intactos, sem medição nova de performance. Gates numéricos24/24porcoleta aprovados não anulam reprovação de retorno; ambos runners terminam1 após salvar diagnósticos, não repetir até obter verde. Relatório confere cardinalidade, coortes e limites; hashes dos resultados em texel-summary.json. Painel anterior mantém seus resultados e reprovação.

715testesRust (`cargo test --locked --release --all-targets`), Clippy release/all-targets-Dwarnings, fmt e diff passaram.13gruposMCPheadless render/replay_inspection/selection aprovados, nenhuma janela. Smoke do campo de normais legado testa pipeline normal/UV e troca de bindings:16/16capturas prior efield aprovadas; controle reduced sem campo conserva8falhas explícitas. Formato/cena/NPC/MCP da engine não mudaram. ShaderWGSL manual e attachment adicional compilados/exercitados naGPU; target extra só no diagnóstico. Sem aceite visual humano, navegador ou plataforma adicional. Build corrente reutilizado, sem publicação.


## EAF-106 — painéis com espessura e vãos nativos

Build397d7865d7051b9b, codex/image-to-3d, macOS arm64/Metal. `cargo test
--locked --release --all-targets`:871 aprovados (quatro novos); Clippy
`--all-targets -- -D warnings`, fmt e diff aprovados. Regressão completa
headless:84cenários/347checks. O cenário mesh_panels confere oito grupos:
preview sem mutação, volume/topologia, A→B→A/undo, erros/stale/lote,
segundo painel rotacionado/escalado, proteção de fonte procedural, save/load e
replay estático com pixels exatos. Raios independentes no teste Rust distinguem
abertura real de superfície fechada. Capturas frente/verso abertas.

Contrato, alternativas, checklist e limites em [mesh-panels.md](mesh-panels.md);
[relatório](references/image-to-3d/mesh-panels-verification.json),
[cena](../examples/native-panels.scene.json), kernel src/mesh_panel.rs,
script scripts/verify_mesh_panels.py. Sem renderer/formato/dependência nova,
sem inferência de imagem, CSG geral, colisão ou certificado de casa. E02 e
ensaios anteriores preservados; capacidades gerais de montagem continuam abertas.

## EAF-88 — amostragem analítica independente de derivadas

`rich_relief_probe --analytic-repeat/--analytic-run`, relatório `scripts/report_rich_relief_analytic.py`, painel `artifacts/appearance-probe/rich-relief/analytic.html`. Fonte/shader original, anterior/regional originais e mesmos caminhos com shader analítico; mesmas texturas residentes compartilhadas.716testesRust release/all-targets,Clippy-Dwarnings,fmt/diff e13gruposMCPheadless render/replay_inspection/selection passaram. Fórmula contra768casos independentes de raio/plano. Smoke legado normal_field:prior/field16/16cada,controle reduced8falhas conhecidas;24PNGs íntegros removidos após lsof,smoke.json/log retidos. Nenhuma janela ou contrato funcional alterado.

Coleta dedicada768retornos:original252/256iguais,analíticoLOD256/256,manual256/256. Coleta ampliada138poses/800timings/50retornos em1728×1117/3456×2234:analíticoanterior97/138imagens,analíticoregional97/138;ambos10/10retornos. Originalregional138/138imagens/10retornos,anterior137/138/10retornos. Fonte8/10retornos:PNG1728/perto muda12.404pixels,max54/255;3456/perto37.616pixels,max81/255. São falhas maiores que as poucas diferenças observadas em algumas coletas precedentes; não minimizar como ruído de um nível. Causa não estabelecida. Ambos runners terminam1 e preservam dados; aproximação por normal de shading não promovida. Não confundir retorno exato com estabilidade durante movimento ou gates agregados com pixels idênticos.

690PNGs decodificados/resoluções conferidas,50pares de retorno auditados (`analytic-verification.json`),2.208estadosDOM/DPRsimulados com avisos de falha; vistas próximas e piorRMSE inspecionados. Sem navegador nativo/aceite humano. Tempos mostram ausência de ganho robusto e regressão de8,07%no regional3456/distância6; tabelaCPU/GPU/total e dispersão preservada. Nenhuma nova suíte1×–100× nesta unidade,nenhum ganhoFPS/integração/produção alegado. Target/release corrente reutilizado; artefatos anteriores de comparação preservados.


## EAF-107 — personagem orgânico por imagem

Tentativa registrada em [fox-character-experiment.md](fox-character-experiment.md);
[relatório](references/image-to-3d/fox-character-verification.json).
Build397d7865d7051b9b inalterado; somente cliente/dados/docs/cena.5testes
numéricos novos, pipeline MCP real em processo próprio/headless, fontes nativas
PCHIP+escultura e composições cliente declaradas.12assets/12instâncias,9726v/
18096t,329partes incluindo cenário; limites mantidos.329faixas extraídas do
save nativo com fechamento/orientação válidos. Preview puro, carga/undo/replay
estático com pixels idênticos. Fonte0339c215…2f14 preservada;18pares fixos/seis
vistas inspecionados.16v de4adornos ainda sem suporte integral, registrados.

Fidelidade visual REPROVADA: cabeça/anatomia, pose, roupas, motivos e materiais
diferem.146tarefas conservadas;87vínculos de geometria parcial, zero aceites.
SSIM0.704219 inclui fundo e não é percentual. Sem nova regressão Rust/Clippy,
sem renderer/rig/colisão/tecido físico ou validação de interação WebGL.

## EAF-88 — orientação geométrica versus normal de material

Ensaio `rich_relief_probe --tangent-build/--tangent-run`, IGT1/versionamento/hashes/quantizaçãof32 e três testes de plano/corrupção/fonte/cancelamento. Fórmula de projeção independente já coberta pelo teste raio/plano.719testesRust release/all-targets,Clippy-Dwarnings,fmt/diff e13gruposMCPheadless render/replay_inspection/selection. Feature primitive_index testada noAppleM4Pro/Metal apenas nesteharness; backend principal/Scene/NPC/MCP intactos.

512repetiçõesestáticas,138poses,640timings e40retornos de trajetória. Geometric-lod egeometric-grad138/138imagem,128/128repetições e10/10retornos cada;shading-lod98/138imagem. Cobertura igual,RMSEmax geom.grad0,004259/p990,017008,canalmax0,376984,sem identidade depixels. Fonteoriginal64/128repetiçõesestáticas,até160.851pixelsfloat alterados,mas10/10retornos posteriores. Runnertermina1,sem repetir atépassar; não converter estabilidade decandidato emaprovaçãogeral. Dados/report/painel tangent-*preservamreprovação, inclinaçõescompiladas+9MiB e regressãopróxima~12%.

552PNGs decodificados nasresoluções corretas,40paresPNGretorno idênticos;1.656estadosDOM/DPRsimulados com falhas/exibiçãofísica/estatística dareferência visíveis. Captura próxima inspecionada;sem navegador nativo ouaceitehumano. Dadosgerados emcaminhosestáveis,mesmobuildcorrente,sem nova suíte1×–100×,sem push/integração.

Smoke legado do campo de normais:16/16prior/field aprovados,8falhasconhecidas docontrole reduced;24PNGsdecodificados/removidosapóslsof,smoke.json/logmantidos. Roteiro completo de comandos e checklist posterior no estudo.

# Casa élfica — cena nativa por MCP

Receita/script/cena em examples/elven-cottage.recipe.json,
scripts/demo_elven_cottage.py e examples/elven-cottage.scene.json.
Escopo, checklist, revisão visual e limitações em [elven-cottage.md](elven-cottage.md).
Onze grupos próprios passaram com pixels exatos em repeat/undo/load/reset/replay/
prefixo e duas execuções novas idênticas. Artefatos correntes em
artifacts/elven-cottage/artifacts (hero/front/west/rear/detail,report,determinism).
Seis cenários MCP/27grupos headless passaram: render,meshes,plants,plant_sections,
directional_shadows,materials. Build ec96a3441fef43e2 reutilizado, sem Rust/shader
novo ou suites Rust completas repetidas. Python compile/fmt/diff check aprovados.
15assets/19instâncias/32172v/53376t armazenados/61312t desenhados; nenhum teto
ampliado. Árvore e demo preservadas. Resultado técnico não é aceite artístico,
interior habitável, transparência física de vidro ou interpretação automática.

# Caixas chanfradas nativas — EAF-121

Aplicação da casa v5:14blocos/26lajes gerados nativamente, com juntas do apron
desencontradas/variação dimensional limitada por PRNG próprio.32596v/54728t
armazenados,55948t draw;16assets/21instâncias,172v livres.15checks da receita
passaram e duas gerações deram cena/hero byte-idênticos. Cena compacta2.351.797B,
SHA`88b5a8410e62d5db58a2db0f5b00da27b624322a000514f73ee2056b98fb823b`, reaberta
via MCP igual ao export e captura idêntica. Estudo MCP prova que só geometria de
stone-blocks/paving mudou, demais assets/canais/aparências/plantas/instâncias/
câmera/luz/cena iguais. Before1054577/after com baseline pixels exatos e clay;
closes paving/entry-stone, câmeras/restauração e fontes em artifacts/cottage-stone.
Mesmos shader/layout/versões; referência/árvore/main/demo/outros checkouts intactos.
Receita/estudo/checklist/limites em [cottage-stone.md](cottage-stone.md). Realismo,
friso oculto e integração da outra frente seguem separados desta prova.

Branch codex/native-plants, macOS Apple Silicon/Metal, Scene72/replay76 mantidos.
Build `aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|945ebbe0ba1b6c41`.
Fingerprint conferido por componentes de caminho, como build.rs.860testes Rust
release/all-targets em85alvos passaram (3novos); Clippy all-targets-Dwarnings,
fmt e build release passaram. Regressão completa87cenários/358grupos MCP headless
passou;69summaries com build_id idêntico,18legados sem esse campo. Sem janela.

Kernel: volume analítico, bounds, contagens8v12t/24v44t, normal para fora,
manifold, transformação quaternion, material coords, append e precisão/limites.
Seis grupos MCP próprios: schema/preview puro/publicação, inválidos/stale/precisão
f32 atômicos, append conservando canais/aparência/rig recusado, referências em
lote/planta/draw budget, invalidação/undo/redo/remove e persistência/replay/prefixo/
viewport/pixels. Teste visual isola chanfro com material fixo. Volume removido
é2c²(x+y+z)−16c³/3; chanfro é plano, sem raio contínuo ou bevel de região.

Logs artifacts/chamfer-final-tests.log, chamfer-clippy.log, chamfer-build.log,
chamfer-full-mcp.log e mcp-validation/report.json;3testes iniciais também em
chamfer-tests.log. A verificação completa Rust foi repetida após reforçar recusa
de inversão/colapso pós-f32. Sem dependência, GPU/Vertex/formato persistido novo,
nem ganho de FPS ou suporte multiplataforma inferido. [Contrato](chamfer-boxes.md).

# Casa — ornamentação em relevo

Receita clientev4 sobre f0f9664, mesmo build53d31b5b0cead1f3/Scene72/replay76.
19folhas de seção losangular e9estações positivas usando Sweep existente;
exemplo genérico examples/foliate-relief.edit.json.31956v/53448t armazenados,
54668t desenhados,16assets/21instâncias/360Sweep,812v livres. Sem kernel ou shader
novo. Somente bronze.geometry mudou: demais assets/plantas/instâncias/câmera/
luz/aparências e restante da cena iguais ao baseline, preservando frente dos arcos.

14checks próprios da casa e2cenários/12grupos MCP headless (render5/sweep7)
aprovados. Folhas com áreas transversais positivas, início<meio e ponta<início;
bronze fechado sem bordas abertas/não-manifold/isolados. Exemplo isolado36v/68t
validou plano/preview/publicação e manifold. Duas gerações da casa deram cena e
hero byte-idênticos. Python compile/fmt/diff check aprovados; sem repetir Rust
completo porque não houve alteração Rust. Evidência histórica do núcleo mantida.

Cena compacta2.189.575bytes SHA
`a742696faca960b08684a8a269975477c6544568de50b4b38b578536649a57e6`, reaberta pelo
MCP igual ao export, pixels publicados idênticos. Comparação before f0f9664/
after, mesma câmera/luz, baseline pixel-exato e clay em artifacts/cottage-relief;
logs e preservation/publication/leaf-report.json. Relatório da casa identifica
índices/ranges das lâminas, sem inventar IDs persistentes. Árvore/reference/main/
demo/worktree paralela preservados. [Limites e checklist](cottage-relief.md):
densidade de seções ainda explícita, planner só limita linha central; peças
independentes sobrepostas, sem weld/escultura transferida ou realismo alegado.

# Casa — cornijas e cobertura contínuas

Continuidade após b33a76c: receita clientev3, mesmo build53d31b5b0cead1f3,
Scene72/replay76. Corrigido beiral com inversão local e aplicado Sweep às cornijas
do telhado, usando curvas compartilhadas pela superfície. Backing com3ilhas
fechadas/orientadas, volumes positivos, sem bordas abertas/não-manifold/isolados.
31720v/52988t armazenados,54208t draw;16assets/21instâncias/357Sweep,1048v livres.

13grupos próprios passaram, incluindo contornos/folga, topologia/orientação,
preview/erros/undo/redo/save/load/reset/replay/prefixo/pixels/viewport. Duas
gerações byte-idênticas de cena e hero. Regressão recortada headless3cenários/
16grupos (render5,sweep7,plants4), Python compile/fmt/diff check aprovados;
sem Rust/shader novo, sem repetir os gates857Rust/86MCP do núcleo anterior.

Cena compacta2.170.713bytes, SHA
`a69488c31be211e7f9d1d5209b38629a7f59d877cea1d79d33c8cb8092da98ed`;
reaberta via MCP igual ao export nativo, captura publicada idêntica. Comparação
b33a76c/nova com mesma câmera/luz, baseline pixel-exato e vistas hero/entrada/clay
em artifacts/cottage-roof/artifacts; report/publication/backing-volumes.json e
logs render/sweep/plants/authoring preservam evidência. Ferramenta de comparação
aceita root/PNG explícitos, evitando vincular toda revisão à v1 implicitamente.
Referência e árvore SHA89fe1fa…9400 conservadas; main/demo/worktree paralela
intactos. [Decisões/checklist/limites](cottage-roof.md). Aceite da melhora anterior
não é aceite artístico desta revisão, nem prova de casa inteira por plantas.

# Varredura de perfil3D — EAF-113

Aplicação da casa: receita clientev2,16assets/21instâncias,32390v/54068t armazenados
e55288t desenhados. Dois ensaios integrais geraram cena e hero byte-idênticos;
11checks próprios passaram (preview/planta/erro atômico/undo/redo/repetição/tick60/
replay/prefixo/load/reset/viewport). Comparação before/after com câmera/luz iguais,
views de entrada e clay; baseline anterior preservou pixels. Capturas reais em
artifacts/elven-cottage/artifacts e cottage-structure/artifacts, sem abrir janela.
Publicação compacta2.213.678bytes reaberta pela engine e comparada ao export nativo:
SHA256`d031a89195d76cc447da9601ca98d44f47652dbecad186e54be13ba864267c05`.
Árvore anterior mantida SHA`89fe1fa2112cbb8a2d3ae266a53cf0d6308c8b371d356a8f5b1b05a208789400`.
Referência original preservada; casa continua hipótese de autoria com encontros,
materiais/jardim/interior ainda simplificados. [Revisão e fricções](cottage-structure.md).

macOS Apple Silicon/Metal, branch codex/native-plants, Scene72/replay76, build
`aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|53d31b5b0cead1f3`.
`cargo test --locked --release --all-targets`:857testes em85alvos, incluindo7novos.
Clippy all-targets `-D warnings`, fmt e build release passaram. Regressão completa
`scripts/run_verifications.py`:86cenários/352grupos MCP headless, todos passaram;
7grupos específicos de mesh_sweep. Logs em artifacts/sweep-rust-tests.log,
sweep-clippy.log e mcp-validation/report.json. Um único binário corrente em target.

Cobertura: seção/volume analíticos, perfil assimétrico/3D/taper, caps/manifold,
junções/frame/material coords, regularidade, curva fechada recusada, tolerância
contra1001amostras independentes e limites. MCP exercita planner puro, preview,
publicação igual à previsão, append preservando canais/aparência, erros atômicos,
rig/planta protegidos, invalidação de fonte em lote, undo/redo/remoção/reset,
save/load/replay/prefixo/viewport. `surface_validated:false` é testado: resolução
certificada não equivale a superfície admitida. Contrato e revisão estática em
[mesh-sweep.md](mesh-sweep.md). Sem alteração de renderer ou formato persistido;
sem prova global de interseções, aceite artístico ou interação física de UI.



# Revisão arquitetônica independente da casa

Retorno de Guilherme motivou diagnóstico estrutural em
[elven-cottage-review.md](elven-cottage-review.md): dez recortes reais conferidos,
manifesto de fontes/regiões/hash e três vistas nativas adicionais1536×1024
(frente,empena,câmera baixa) em artifacts/elven-cottage-review. Câmera restaurada,
estado e arquivo de cena originais preservados; sem modificação de modelo,
receita,Rust,shader ou novo build. Unidade documental/de inspeção, sem alegar
nova melhoria de fidelidade ou repetição das suites completas. Diagnóstico
separa erros de composição/seção de câmera/material e de hipóteses de planta.



## EAF-112 — segundo padrão procedural e prova na mesma casa

Build e0a95a4b90e7fb39, macOS Apple Silicon/Metal, Scene73/replay77.
854testes Rust/86executáveis de teste em release/all-targets passaram (quatro
testes novos: encoding/layout e três contratos de camadas). Clippy release/
all-targets com-Dwarnings, fmt/build, py_compile e diff check aprovados.
Regressão MCP completa:86cenários/349grupos headless, inclusive novo
layered_patterns. Logs correntes artifacts/layered-{all-tests,clippy,build,
full-mcp}.log; runner artifacts/mcp-validation/report.json; fingerprint de fonte/
binário e hashes das cenas em artifacts/layered-build-audit.json. Não há evidência
de teclado/foco/janela nativa, outra plataforma ou ganho FPS nesta unidade.

Novo cenário cobre schema, camada neutra exata, primary/secondary-alone exatos,
cor/bump/frequência/seed/kind independentes, fade subpixel exato, troca de ordem
com máximo observado1LSB, limites/soma bump/erro atômico/preview puro/publicação,
geometria/undo/redo/reindexação de slots, gates scene72 na raiz e3fontes ocultas
(source/context/graft), rig compensado e animação, replay/prefixo/load/reset/
viewport. Legado continua byte-exato, sem aplicar a tolerância da troca de ordem.

Estudo específico `scripts/study_cottage_materials.py` repetido no mesmo build:
três grupos com vistas hero/detail/front1536×1024, nove materiais, lote puro/
publicado, invariantes de toda a cena exceto materiais/version/câmera, undo/redo,
repeat, ablação da segunda camada, controle material-neutro, save/load/reset,
60ticks estáticos/replay/prefixo e viewport. Hero/detail antigos ec96a3441fef43e2
preservam SHA/bytes exatos no novo shader. Ablação muda a imagem; controle sem
padrões igual nas duas versões. Resultado corrente em artifacts/cottage-materials/
artifacts/ (before/after-hero.png, before/after-detail.png e report.json);
receita mantida examples/cottage-materials.recipe.json, saída73 separada da72.
15assets/19instâncias/32172v/53376t/61312draw inalterados; árvore SHA89fe1fa2…9400
preservada. Custo uniforme1152→1232B/slot/19584→20944B por17slots; Vertex64 e
Globals176 mantidos. Sem mais geometria, texturas, instâncias ou tetos.

A revisão material reduz aspecto de relevo amassado e separa grão/envelhecimento;
deltas de imagem não são nota de qualidade. EAF-101 permanece em progresso:
pórtico/bay/empena/coberturas/seções/vazios ainda diferem da referência, vidro
opaco/metal dielétrico/lanterna sem luz pontual permanecem. Não declarar casa
realista/fiel, UI concluída, CAD novo ou árvore encerrada com aceite artístico.

## EAF-114 — metalness e ensaio nos ornamentos da casa

Build993076c35e8162d4, macOS Apple Silicon/Metal, Scene74/replay78.
857testes Rust/87executáveis release/all-targets, Clippy-Dwarnings,fmt/build,
py_compile e diff check passaram. Três testes novos: encoding/layout, contrato
serde/bounds/zero/igualdade, publicação/preview/history/tick/replay/gate.
Regressão MCP completa:87cenários/354grupos headless. Logs correntes em
artifacts/metallic-{all-tests,clippy,build,full-mcp}.log; relatório do runner em
artifacts/mcp-validation/report.json; audit de fonte/binário/cenas em
artifacts/metallic-build-audit.json. Todos os relatórios MCP com build_id
concordam com o fingerprint da fonte. Sem janela/teclado, outra plataforma ou FPS.

Cenário metallic/5grupos: zero e ausência byte-exatos; incidência quase normal
GGX com pixel central[126,81,45] igual ao analítico (tolerância1LSB); mistura
linear de BRDFs com erro máximo observado1LSB (limite2LSB para quantização dos
controles sRGB); F0 dielétrico independente no metal puro, cor/rugosidade,
bounds/transmission/erros atômicos/pureza e igualdade de preview/undo/redo/slots,
gates root+3fontes ocultas em cena73, luz/vista/bump primário+secundário, sombra
completa/ambiente/backlight, degenerados/rasante seguros, rig compensado/animado,
replay78/prefixo/load/reset/viewport. Tolerância analítica não relaxa igualdade
exata de legado/history/replay.

scripts/study_cottage_metallic.py:3grupos próprios, hero/detail/front/ornament
1536×1024; mesma casa e recipe material anterior. Baselines e0a95a4b90e7fb39
hero/detail/front retornam SHA/bytes exatos no novo shader. Só bronze.material
muda no objeto da cena inteiro: geometria/canais/fontes/plantas/instâncias/cores/
rig/clipes/luz/câmera preservados. 15assets/19instâncias/32172v/53376t/61312draw,
nenhum teto aumentado. Roughness0.3/0.6/0.9 dão controles distintos; somente
ambiente mantém o mesmo PNG para bronze anterior/metal, expondo ausência de IBL.
Preview puro, undo/redo/repeat, save/load/reset,60ticks/replay/prefixo/resize
passaram. Artefatos correntes artifacts/cottage-metallic/artifacts/ e saída74
separada examples/cottage-metallic.scene.json dentro desse root de estudo;
receita mantida examples/cottage-metallic.recipe.json. Fonte72 SHA7286e74c…f1e2c
e árvore SHA89fe1fa2…9400 preservadas.

Revisão visual: realces agora têm cor metálica; faces fora do realce podem
parecer planas sob ambiente constante. Não é resultado final realista/fiel.
Reflexão de ambiente, parâmetros físicos por região/oxidação, vidro, HDR e
aliasing continuam próximos casos em authoring-gaps.md. A arquitetura/silhueta
não foi alterada nesta prova; composição/Sweep é frente independente. high.w
reutilizado/default0, layouts1232/20944/64/176B mantidos, zero texturas/passes/
geometry draws novos. EAF-101 segue aberto; commit local sem push/main/demo.


## EAF-116 — ambiente distante procedural

Buildb2f55019d92fe070, macOS Apple Silicon/Metal, Scene76/replay80. Reserva75/79
permanece na frente funcional EAF-115; Wedge não está integrada nesta unidade.
862testes Rust/88executáveis release-all-targets,Clippy-Dwarnings,fmt/build,
py_compile e diff check passaram. Cinco testes novos: três de parâmetros/cache/
pré-integração/pack/analítico e dois contratos Light/preview/rollback/replay.
88cenários MCP/358grupos headless passaram. Logs artifacts/environment-{all-tests,
clippy,build,full-mcp}.log; relatório do runner e audit em
artifacts/environment-build-audit.json. Todos build_id reportados conferem com
fonte/binário. Sem janela, teclado nativo, outra plataforma ou medição FPS.

Cenário environment/4grupos: None/I0/direct1 byte-exato e sem preparo, diffuse
constante/cor/intensidade, metal branco roughness1 com pixel[151,151,151] versus
analítico150 (limite2LSB), normalização de eixo e chave sem intensity; eixo/vista/
roughness em esfera fechada, transparência luminosa aproximada traseira distinta,
preview puro/publicação/undo/redo/erros/gate75, slots, None/A/B/A/cache/rebind de
sombra, rig compensado/animação,76load/reset/80replay/prefixo/viewport. Contadores
terminam7probes/1LUT pelas trocas autoradas de conteúdo; câmera/material/pose/
intensidade não aumentam contadores. Reexecução específica após incluir o
controle de transmissão também passou, sem ampliar tolerâncias do legado.

scripts/study_cottage_environment.py:3grupos, mesma casa ea8c6b8 com materiais
anteriores. Hero/detail/front/ornament993076c35e8162d4 byte-exatos no novo shader.
Só Light muda no documento; geometrias/fontes/plantas/instâncias/cores/rig/clips
iguais. Direção/sombra e peso efetivo do sol preservados; constante ambient0.32
substituída explicitamente na receita por ambiente procedural e ambient0/direct0.68.
Ablações sun-only/environment-only/nonmetal/intensity/axis distinguem os efeitos;
repeat/history/60ticks/save/load/reset/replay/prefixo/resize exatos.3bakesProbe/1LUT
no final, zero bakes na reprodução antiga, intensidade/câmera/viewport sem bake.
Fonte72 SHA7286e74c…f1e2c e árvore SHA89fe1fa2…9400 preservadas.
15assets/19instâncias/32172v/53376t/61312draw, caps intactos. Artefatos correntes
artifacts/cottage-environment/artifacts/ e cópia76 no root do estudo; receita
mantida examples/cottage-environment.recipe.json. Novo material não corrige
arquitetura; a casa estrutural da outra frente será integração separada.

148984B lógicos/3texturas/sampler por Renderer, Globals192B/offsetenv176,
Vertex64/Material1232/tabela20944. Split-sum/v=n, amostragem finita/UNORM8 dos
cubos e DFG16 empacotado são limites explícitos; nenhuma imagemHDR importada,
reflexão dos objetos,AO/GI,tonemapping ou specAA. A luz nova expõe terminações e
transições de brilho abruptas nos ornamentos antigos; normal/sidedness é hipótese
para diagnóstico posterior, não reparo geométrico declarado.


Integração Sweep+materiais+ambiente: build10aa09af0bf647b9,869testes Rust,
Clippy e89cenários/365grupos MCP aprovados. Três estudos antigos reaprovados
na fixture cottage-material-baseline.scene.json; composição b33a76c com
três grupos próprios, quatro vistas repetidas e replay/viewport idênticos.
Evidência e limites em [cottage-lookdev-integration.md](cottage-lookdev-integration.md).


## Normais avaliadas e contato da casa — EAF-119

Build55b307c7b20159ec:873testes Rust/88executáveis, Clippy/fmt/build e90cenários/
369grupos MCP headless aprovados. Query nova4grupos; casa4grupos próprios e
quatro vistas legadas byte-exatas após extrair helper do renderer.
`python3 scripts/study_cottage_lookdev.py` seguido de
`python3 scripts/study_cottage_normals.py` reproduz diagnóstico e correção local
via Sweep. Mola da porta52.6037°→0° no índice examinado; variantes/ablações e
undo/load/replay exatos. +72v/+148t, demais spans/channels/peças preservados;
fonte canônica intacta, sem aceite de fidelidade.
Contrato/custos/evidência em [mesh-normal-inspection.md](mesh-normal-inspection.md).


## Amostragem por curva — EAF-120

Build eeef1b9ea254661a:877Rust/88executáveis, Clippy/fmt/build e90cenários/
372grupos MCP headless aprovados. Quatro testes Rust e três grupos MCP novos
cobrem array/escalar, refinamento independente, tempo das seções, limites e
persistência. Estudo da mola4grupos: spans[1,9] em88v/172t vs uniforme168v/332t,
salto52.6°→0, posições originais conservadas sem anel duplicado; demais spans/
canais/peças exatos. Quatro vistas legadas byte-exatas.
Contrato/checklists e limites em [sweep-curve-sampling.md](sweep-curve-sampling.md).


Integração de relevos/colocação/aparência no mesmo build eeef1b9ea254661a:
3grupos lookdev+4normais+3integração aprovados, publicação MCP com regeneração
legada exata e8varreduras/272v reposicionados, resto da fonte preservado. Fonte atual
1b4a7909…a3795; cena final31948v/53436t/54656draw,4vistas repetidas e
save/load/reset/replay/viewport exatos. Src/binário mantidos: gates877Rust/
90MCP372 pertencem ao núcleo já validado, sem repetição no merge de conteúdo.
Evidência e reprodução em cottage-relief-integration.md.

## Autoria imediata e descoberta de ferramentas — EAF-104/110 (29/10)

Worktree editor/codex/editor desdeccce577. Build **fd9217681899e227** auditado
contra101fontes/Cargo.lock/build.rs e o executável corrente20154352B; instalação
por symlink ao mesmo arquivo, target exclusivo/cache~2.2GiB, sem cópias de app,
instaladores ou binários históricos. Scene83/replay87/layout3/projeto1 mantidos.

**1230 testes Rust/118 alvos release/all-targets**, zero falhas/ignorados;
Clippy all-targets-Dwarnings, fmt/diff/build e sintaxePython aprovados. **109
cenários MCP/465 grupos completos headless** passaram;89summaries com build ID
exato,20legados sem ID próprio. Report e logs em artifacts/mcp-validation e
artifacts/editor-studio/{validation,build-audit}.json. Nenhuma janela nativa
aberta/focada/fechada ou cena pessoal modificada pelos testes.

Novos casos: validação em cada update live, undo único e interrupção por edição
externa; rascunhos inválidos não são repetidos em frames idle; criação NPC
selecionada/alvo/percepção; referências obsoletas, material/topologia preservados
em TransformVertices; rigs/pesos/clipes com save/load/replay e overlay de sessão
sem mutação. Entrada egui verifica seleção tipada de vértice/aresta/face e
subdivisão de apenas1aresta/1face, além da troca malha→personagem recuperar picking.
Schemas compartilhados preservam nullables e não acrescentam componentes idle.

Probes GPU/compositor:1440/1000/900/800, campos horizontais/Shift/Y sem falsa
mudança, projeto/luz/pincéis e idle; novo editor_workflows_probe cria NPC pela UI,
configura alvo live, cria raiz/filho, seleciona/move vértice com um undo e abre
catálogo tipado. Capturas em artifacts/editor-workflows/{npc,rig,components,
vertices,tools}.png. O probe encontrou o cabeçalho de rig herdando estado
recolhido do modo Objeto; modo Rig agora abre seus controles explicitamente.
Validação final completa foi posterior à correção.

Draw lê posições/normais congeladas e soma influência do gesto inteiro. Plano/
esfera24, percursos retos/diagonais e força±0.08 nos dois sentidos mantêm faces
orientadas e diferença<2e-6m; entrada coalescida/cadência também cobertas. Padrão
novo0.04m/amostra mantém a régua de relevo anterior0.1..0.5m: probe95v/
máximo0.28602996m. Preview/commit/undo/redo exatos; comando real da UI enviado ao
MCP público coincide bit a bit em f32, inclusive save/load. Valores de força já
salvos não foram migrados. Não há reparo automático de malhas já deformadas.

As rodadas preliminares detectaram validação ausente no caminho coalescido,
suposição incorreta da contagem de amostras por pixels e seletor de label do
probe confundindo menu Editar com modo Editar; corrigidos sem ampliar os limites
de qualidade. Compilações intermediárias foram interrompidas somente no grupo
de processos de validação desta tarefa para revisão; a rodada acima é completa.

Barra macOS integrada ao conteúdo e comandos de janela compilados; captura
offscreen não comprova botões nativos/foco/teclado/acessibilidade. Aceite humano,
interfaces especializadas para todo o catálogo, novos pincéis, remesh/extrusão
topológica, pintura de pesos e timeline/IK/retargeting permanecem pendentes.
Contratos/custos: editor-workflows.md, editor-sculpt.md, performance.md.


## EAF-104 — superfície visível, retângulo e ícones de área (29/10)

Recorte sobre02b1c2d em codex/editor: flag `mesh_view.xray` independente,
false por omissão, vista e seleção respeitam superfície real/oclusores.
`select_editor_mesh_region` compartilha referência, câmera, viewport física e
união aditiva com a UI. Sem Shift substitui; vazio limpa, Shift preserva/adiciona.
Domínios de arestas/faces usam centro; vértices usam posição. Cancelamentos de
Esc/foco/câmera/resize/revisão preservam seleção anterior. A limpeza externa
não é restaurada pelo rascunho do inspetor.

Ícones vetoriais substituem Área no cabeçalho direito: maximizar/restaurar,
dividir lado a lado/acima/abaixo, abrir/fechar abas, com tooltips/nomes acessíveis.
Menus de escolha conservam texto; viewport não permite fechar sua última aba.
Cabeçalho estreito agrupa modos para conservar Vista/Iluminação/ações. Nenhum
formato Scene/Replay/layout nem dependência foi alterado por este incremento.

A primeira rodada completa passou1235Rust/119alvos,Clippy,110MCP/469grupos,
probes1440/1000/900/800,workflows e escultura, build74cfec1ffe1ee253. Revisão final
detectou recibo de região com view_revision anterior à limpeza do ator; corrigido
para devolver revisão final. Nova regressão começa com player selecionado,
verifica invalidade atômica e seleção válida sem mudar cena/histórico;4/4casos
no alvo editor_mesh_selection passaram. Gates finais abaixo substituem esse
build preliminar.


Gates definitivos no build **ff6d6e045cb4645c**: **1236testesRust/119alvos
release/all-targets**, zero falhas/ignorados; Clippy all-targets-Dwarnings,
fmt/diff/build aprovados. **110cenáriosMCP/469grupos completos headless**;
90summaries com buildID exato e20legados semID próprio. Fonte/binário auditados
(FNV de103arquivos), instalação samefile por symlink: release corrente20272160B,
cache exclusivo~2.3GiB; sem cópias/instaladores/histórico de binários.

Probes egui/GPU1440/1000/900/800 e workflows passaram: ícones por clique,
retângulo/Shift/vazio, checkboxraioX real, NPC/rig/live drag com undo e catálogo.
Na esfera24 do fixture:492vértices visíveis versus1106com raioX. Oclusão por
malha/caixa/cunha e mudanças de câmera/cache cobertas; overlay de ossos continua
guia independente. Escultura preservada:95v/max0.28602996m, preview/commit/undo/
redo exatos e comando da UI bit-exato f32 no MCP público/save/load. Evidências
em artifacts/editor-studio/{validation,build-audit}.json/logs,
artifacts/editor-workflows/selection-{surface,xray}.png e
artifacts/mcp-validation/report.json. Nenhuma janela humana/cena pessoal
aberta, fechada, focada ou modificada pela validação offscreen.

Handoff para consolidação na main autorizado no chat meshes:22arquivos congelados
em frozen-delta.json/patch; recibo corrigido entregue separadamente em
receipt-supplement.patch (SHA2564059ddb9b3a14598b367785f379c8f2dd078ae51bc7e3541027ed6582a1c15bf).
Patch completo f6d0262090122a5f2293cc2209ee8115b9f58ae9b5b033fe3436763b2d3c7e76,
base02b1c2d, sem divergências após gates; apenas este registro adicional.
A frente central integra/valida a união86/90 separadamente e preserva exclusão
de Nanites/Modly; nenhum merge/main/push feito neste incremento. Rotina antiga
manter-ignis-editor-atualizado pausada para evitar corrida com a consolidação.
Worktree/output em uso preservados. EAF-104 permanece In Progress: aceite
humano/foco/teclado/acessibilidade nativos e UX especializada do catálogo,
novos pincéis/remesh/extrusão topológica/pintura de pesos seguem gates próprios.

## EAF-128 — união estável na main

Buildd092a8fec0ed01f0/SHA256bin91c0a943062458052158759a3b7855f0473cb807e9d7d7f432a67b360ad5531d; fonte conferido por FNV com a mesma ordenação de componentes de Path do build.rs.1139testesRust/114alvos release/all-targets, zero falhas/ignorados,Clippy-Dwarnings/fmt/build/diff aprovados.113cenáriosMCP/482grupos completed/passed/native_window:false, inclusive render/cache/replay e singleplayer offline isolado da rede. Estudos da casa3+4+4grupos no mesmo build; captures repetidas e persistência/replay exatos. Evidência consolidada artifacts/main-integration-audit.json e logs main-integration-*.

UniãoScene86/replay90 preserva gates históricos e compara controles/controladores/mobilidade/meshes; novo teste combina2Players/KCC/conectores e meshes e detecta alteração independente do controlador no replay. Fonte385bbdd mais1304a3e e frentes nativas/runtime identificadas em main-consolidation.md, com squash filtrado para excluir244arquivos exclusivos Nanites e sua ancestralidade. Modly/research refs preservados. Erros encontrados e corrigidos: recibo de seleção com revisão anterior, duplo clique capturado por seleção textual, aliases macOS na remoção de metadados.

App local retargetado à main sem iniciar janela; aceite humano/teclado/nativo e fidelidade artística permanecem separados. Sem push/publicação/rede/escala1000 inferidos. Aviso futuro block0.1.6 é preexistente e distinto de Clippy.


## EAF-104 — campos numéricos da seleção de componentes (29/10)

Bug humano reproduzido em main066f897 com47vértices: TransformVertices recusada
fazia os campos de centro sumirem por1frame. A revisão local era avançada antes
da aceitação do comando; recusa conservava revisão real e disparava invalidação
local. A confirmação agora acompanha só edição aceita; seleção canônica atualiza
sua revisão na reidratação, inclusive em undo seguido de nova seleção. O segundo
caso foi detectado no probeXYZ: o campo de osso podia ocupar o lugar temporário
do campo da seleção. Corrigido antes dos gates finais, sem enfraquecer validação.

Host encaminha recusa ao contexto de geometria; aviso em português junto dos
campos conserva motivo original no tooltip. Valores/geometria/histórico válidos
são preservados, sem retry em idle ou perda de seleção. Passo local de vértices
0.01→0.001m/pixel, Shift fino existente0.1×; deslocamentos explícitos do MCP e
kernel de colapso/inversão mantidos. Sem formato/dependência novos.

Regressão egui red→green47v: rejeição/estado/campos/idle/recuperação, múltiplos
updates e um undo. Probe GPU usa caminho real de resultado/erro do host e testa
X/Y/Z com47v, delta0.01m por arraste10px, vértices externos intactos e undo
exato. Captura artifacts/editor-workflows/component-edit-rejected.png. Probes
1440/1000/900/800, ícones, seleção/raioX, NPC/rig e catálogo passaram; escultura
preservada95v/max0.28602996m. Probe de layout inicialmente não encontrou pasta
de capturas no checkout principal; agora cria artifacts/editor-studio antes de
gravar. Clippy e probes repetidos após esse ajuste de preparação, com fonte de
produto congelado; sem repetição desnecessária da suíte Rust já aprovada.

Gates no build **e50ec1e2495e32ff**: **1140Rust/114alvos release/all-targets**,
zero falhas/ignorados; Clippy-Dwarnings/fmt/diff/build e **113MCP/482grupos
completos headless**,93summariesIDexato/20legados. Fonte106arquivos/FNV/binário
conferidos e instalação samefile por symlink: release corrente20645312B,
target exclusivo~2.3GiB, sem cópias/instaladores/histórico de builds.
Scene86/replay90/layout3/projeto1 preservados. Auditoria/logs/reprodução em
artifacts/editor-component-numeric/{validation,source-manifest}.json,
run_validation.py e *.log; relatório geral artifacts/mcp-validation/report.json.

Janela humana PID65196, cena e mudanças em memória preservadas; nenhuma janela
aberta/fechada/focada pela regressão. A janela existente continua no executável
carregado anteriormente; salvar/reabrir é necessário para avaliação da correção
na malha humana. Entrada nativa física e aceite humano permanecem separados.
Gizmos, novos pincéis/modelagem, timeline/pintura de pesos e UX especializada
não foram acrescentados por este bugfix. Revisão/custos em editor-workflows.md e
performance.md; sem push/publicação ou reativação do heartbeat antigo.

## EAF-129 — projeção nativa de landmarks e estudo estrutural

Main, build **f914158cfdfd5f3a**, fonte/FNV conferido contra o executável.
`cargo test --locked --release --all-targets`: **1143 testes/115 alvos**;
Clippy release/all-targets com -D warnings, fmt/check, diff/check e build
aprovados. Regressão completa headless: **114 cenários/486 grupos**, incluindo
render/editor/replay e4grupos MCP novos de projeção. Logs atuais em
artifacts/landmark-projection/; relatório em artifacts/mcp-validation/report.json.

Consulta pública testa clipping, perspectiva, reciprocidade com picking,
resíduos de pontos fora do enquadramento, exclusões explícitas, limites/erros,
guarda de revisão, imutabilidade de pixels/estado e undo/redo/save/load/reset/
replay. Nenhum campo Scene86/replay90/layout3/projeto1 ou dependência novo.

`scripts/study_cottage_projection.py` aprovado no mesmo build:4câmeras,
10âncoras autorais e5vistas da estrutura existente em material neutro. Mesmos
meshes entre tentativas de câmera; consulta sem mutação; geometria retida no
isolamento idêntica à fonte,6assets/11instâncias. Fonte spring SHA256
65bfe4ecf2b453ff55a3c63a857eed25b184867316a78a22883b12f02af0c712 preservada;
referência SHA256209cd5cb91c216100413d6b175b5ce29a70cab751a0eda710e4baf95895c6553.
Capturas/report/cena isolada em artifacts/cottage-camera-study/.

Não houve nova hipótese geométrica aceita, calibração da referência ou correção
de H01..H07. A projeção mediu uma inversão do sinal da inclinação2D da mesma
cumeeira nivelada, confirmando o confundidor de câmera. Identificada fricção
de isolamento: assets agrupados por material contêm partes construtivas e
ornamentos; seleção/ilhas existem, agrupadores da receita ainda não persistem
no documento. Decisões, limites e próximos testes em landmark-projection.md e
cottage-structural-review.md. Sem janela, reinício da sessão65196, push ou
publicação; um executável corrente, nenhuma cópia de build criada pelo estudo.

## Editor: gizmos e nove pincéis —29/10/2026

Main desde1efe71f, gizmos em f4f394b, correções HiDPI b186597 e foco84d595a.
Build **ba35aaa3c259011c**, macOS Apple Silicon/release; Scene86/replay90/layout3/
projeto1 e dependências preservados. Fonte110arquivos/FNV conferidos com o binário.

- **1160 testes Rust/118alvos**, release/all-targets, zero falhas/ignorados.
- Clippy release/all-targets -Dwarnings, fmt e diff-check passaram.
- **116cenários MCP/499grupos completos headless** passaram;96summaries com o
  ID exato e20verificações legadas sem ID próprio, todas usando o mesmo binário.
- Probes do compositor: gizmos3ferramentas×4larguras (1440/1000/900/800), UI geral,
  fluxos de NPC/rig/catálogo/seleção e caso47vértices XYZ/recusa/recuperação.
  Prévia/commit/undo dos seis pincéis novos coincidiu usando comandos reais de
  mouse. Volume legado manteve95vértices e máximo0.28602996m.
- Projeção/rays de gizmos cobertos em1.5×/2× com limite físico4096, e foco do
  viewport aceita atalhos sem capturar números/texto de campos; gameplay separado.
- Seis kernels distintos, influência local, força/raio, inversão de caminho/sinal,
  orientação de faces, preferências antigas/independentes, save/load/replay e
  erros atômicos cobertos. Prévia não altera Engine/histórico; um undo por gesto.

Evidências: artifacts/editor-authoring-validation/{validation,source-manifest}.json,
pre-{lib,clippy,gizmos,sculpt,workflows,ui,build,mcp}.log, tests.log/mcp.log e
artifacts/mcp-validation/report.json. Capturas em artifacts/editor-gizmos/ e
artifacts/editor-sculpt-quality/brush-*.png, com shaping-report.json identificado.
A comparação Python de preferências usa o valor f32 canônico retornado pelo
serviço antes de exigir igualdade exata no save/load.

Release corrente **20897824B**, SHA256
`e54210654a24f094ef50d9cf876222f3befe4c7d65505a5b5150e23662030670`.
/Users/zero/Applications/Ignis Editor.app aponta ao mesmo binário da main.
Retenção conferiu131artefatos,111com ID corrente; nenhum artefato com ID
supersedido restante para exclusão. Target corrente~2.1GiB inclui os alvos de
validação, sem cópias de app/instaladores/históricos. Pesquisa separada intacta.

Nenhuma janela/sessão do usuário aberta, fechada, focada ou controlada pelos
testes. App humano PID16465 permaneceu aberto; salvar/reabrir é necessário para
carregar a substituição do executável em disco. Automação de merge antiga segue
pausada. Sem push/publicação. Aceite nativo/humano/acessibilidade e demais frentes
do editor continuam pendentes; não inferir Blender completo, escala não uniforme,
rotação livre de caixas, máscaras/simetria/remesh/tablet, timeline/IK/pintura de
pesos, editores especializados completos ou documentos/viewports independentes.
Contratos e checklists estáticos em editor-gizmos.md e sculpt-brushes.md.


## EAF-110 — diagonais opostas de Volume

Build bab348c00e11cd59 sobre5cf41aa:14testes de kernel/contrato,17UI, Clippy
focado-Dwarnings, fmt/diff e4MCP/28grupos aprovados. Novo SculptDetailed usa
subdivisão local conformante antes do Draw, mantendo orçamento, escopo, canais,
fontes e undo único; preferência nativa visível ligada por padrão. Regressão
Cube24/raio0.241 mede crista geométrica:15.4097%→0.9677% de oscilação na diagonal
adversa,0.0720% na outra, forças0.04/0.12.4testes críticos novos, sem relaxar
critério antigo; testes UI de topologia fixa mantêm seu modo explicitamente.
Probe Graphics/GPU existente ampliado para ambas as inclinações, preview=commit,
um undo, PNG sem dentes. Sem janela/aceite humano ou reparo de dentes já salvos.
Logs/manifesto/checksums/retention em artifacts/sculpt-diagonals/ e PNGs em
artifacts/editor-sculpt-quality/. Scene87/replay92/layout3/projeto1 preservados.
Release atual21217216B, SHA25647993c8f95bf9d9ab28ddd20a20174aa8fd64a2dbe967f584d777a3b319d8504;
app aponta para o mesmo arquivo. Build corrente único; outputs antigos confirmados
inativos removidos com lock/stat/lsof, sessão humana preservada. Ver detalhes e
limites em editor-sculpt.md.

## EAF-161 — restauração portátil

`cargo test --locked --release --test portable_save` passou (1 prova crítica),
`python3 scripts/run_verifications.py --only portable_save` passou (1 cenário/2 grupos),
build3f31de512312c2ee/macOSarm64. Primárias físicas e gameplay exatas no load,
CCD/KCC/contatos frios/voo/efeitos pagos e origem retirada/reuso, tokens novos/inputneutro,
checksum/forja/versão recusados sem mutação, migração/destino preservado,100NPC+500Players,
rootclock/IK prefixo0 e continuação local/replay exatos. Original180ticks manteve
custos/saúde/cooldowns/execuções; maxerroXYZ4,2945147e-5m. PNGprefixo idêntico.
Relatórios portable-save-test.log, portable-save-suite-report.json e
portable-save-verification.json em artifacts. Contrato/checklist em portable-saves.md.
PSV1 não contém arenas/handles/manifolds Rapier nem requisito OS/arquitetura/build;
checkpoint nativo2 mantém guards originais. Validação Windows/Linux não executada,
nenhuma janela/áudio/streaming/aceitehumano ou continuação universal bitexact inferidos.

Fechamento após base95e72e0/cena canônica final160:4provas críticas/4alvos Rust
(portable_save,npc_checkpoint,npc_presentation_motion,npc_air_circulation),
Clippy releasealltargets-Dwarnings, fmt global, Python e diff aprovados.
Logs correntes portable-save-final-rust.log/final-clippy.log/fmt.log em artifacts.


## EAF-165 — representações atmosféricas compiladas

Build visual nativo `61b0a72c829c639d` (macOSarm64/Metal). 18 testes Rust
selecionados (atmosphere2,environment2,network_content1,scene_edits5,editor8),
Clippy proporcional com warnings negados, fmt/Python/diff aprovados. Duas provas
novas protegem formato/atomicidade/replay/save/rede e fonte/certificado adulterado,
inclusive mutação de Arc após a prova memorizada. Layout Globals272B verificado.
MCP atmosférico6grupos e regressões render/environment2cenários/9grupos passaram.
Desligado igual ao PNG do binário anterior; zero idêntico; load/replay/undo/redo
exatos. Referência e otimização compartilham atmosfera/fonte/material; perto100m,
longe1000/9000m e retorno,1728×1117; transições com câmera e sol variando tiveram
máximo1RGB8. Contagens e métricas completas em artifacts/atmosphere/verification.json,
timings separados em cost.json. Contrato/resultados/limites em atmosphere.md.

Compilação explícita na UI/MCP, ATM2porasset; sem avaliação procedural no fill e
seleção antes do upload. Scene100/replay110/conteúdo2,protocolo4/checkpoint2/PSV1
preservados. Fonte CPU ainda residente; sombras/IBL/specular/rigs/attachments usam
fallback. Sem streaming mundial/texturas externas/Nanite universal/FPS/aceite
artístico ou Windows/Linux inferidos. Build corrente em target/release/ignis;
cache debug temporária desta tarefa119MiB removida, sem remover outputs paralelos
ou tocar a janela humana existente.

Benchmark final: caso visível4,317→1,974ms de draw+GPUwait; payload22,16→7,06MiB,
15%menos triângulos e max1RGB8. Movimento mediana4,440→1,900ms, masp95
6,428→6,900ms/max8,101→12,469ms: picos de transição continuam pendentes, sem
alegação de ausência de regressão. Medidas exploratórias com compilação concorrente
preservadas separadamente; não usar o retorno perto como ganho estrutural.


## EAF-176 — floresta30mil e persistência de cena local

Build1a000db026bedafe, macOSarm64/Metal:20Rust em contracts/mesh_resources/projects,
Clippyrelease/all-targets-Dwarnings/fmt/Python/release/diff aprovados. Uma nova
prova protege save/load>8MiB, caminhos de projeto, rejeição sem mutação, escrita
atômica e envelope independente de replay. Renderer não mudou;3provas GPU
existentes não executadas nessa seleção. MCP gerou/compilou/salvou/reabriu30mil
árvores; captura1728×1117 inspecionada, fonte anterior f32/asset/ATM/luz exatos.
Cena de14.426.349bytes aberta no editor para navegação, sem blur, FPSativo.
Amostras móveis2784×1560:7,59–10,04FPS; não A/B parado nem ganho de desempenho.
Contrato/evidências em docs/landscape-forest.md e artifacts/landscape-forest-dense.

## EAF-177 — oclusão conservadora corrente

Metal/M4Pro, floresta30mil em2048×1536, referência anterior com mesma qualidade:
três poses repetíveis e oito poses de câmera/resize com zero canaisRGB8 alterados.
Três rodadas alternadas8warmup+16amostras:91,961→71,080ms perto e117,781→86,456ms
panorama; inverso leve5,104→5,062ms/fallback. Diagnóstico de indiretos fora do tempo
confirma57.140.870/99.135.786triângulos evitados; prepass254682/121002 incluído.
Não é FPS de gameplay, GPU isolada, descarte de memória ou garantia universal.

Teste crítico novo protege falsa oclusão e ordem coplanar:11situações com abertura,
revelação no frame atual, objeto/parede/câmera móvel, near plane, seleção, sombra,
resize, translação8000m, mesh único e compartilhado. Registros em
`artifacts/occlusion/gates-final.log`; provas existentes de mesh_resources preservadas.
Naga validou os shaders; fmt/diff/check/Clippy e build final são gates separados.
Resultados e limitações completos em `docs/render-occlusion.md`.

Gate final:6testes Rust passaram (5mesh_resources, incluindo as3provasGPU, e1
render_occlusion com11situações), Clippy release/all-targets sem warnings do
projeto, fmt/diff/check e build release aprovados. Aviso externo histórico
future-incompat do crate block0.1.6 preservado. Build corrente
d0e474bef699d037 em `target/release/ignis`; app usa symlink para esse único binário.
Janela da floresta30mil aberta e inspecionada:2784×1560, oclusão corrente ativa,
sem blur,26mil candidatos aproximados; snapshot de cadência11,89FPS em
`artifacts/occlusion/live.json`. É uma amostra da janela, sem A/B de FPS na mesma
trajetória; não substitui o benchmark pareado nem constitui aceite humano novo.
Ferramenta temporária própria de validaçãoWGSL removida após uso; sem cópia
histórica de app/build gerada. Mudanças locais, sem publicação.

## EAF-178 — canais consumidos /31-10-2026

Build44dae857d2128e92:7Rust (mesh_resources6/render_occlusion1), Clippy release
all-targets-Dwarnings,fmt/diff e buildrelease aprovados. A/B sobre177 na mesma
floresta30mil/2048×1536:70,700→44,700ms perto e85,962→62,805ms panorama;
RGB8exato3poses+8movimentos/luz/resize, referência anterior ao shader também
exata por hash. Buffers geométricos -798.912B, triângulos iguais. Fonte/critério/
limites em docs/minimal-vertex-stream.md; artifacts/minimal-vertices preserva
cost.json,motion.json,baseline-hashes.json,PNGs e logs correntes. Janela carregada
pelo MCP,layout24B confirmado e cadência19,68FPS amostral; não A/B de FPS ou
aceite humano. Único executável corrente em target/release/ignis, bundle aponta
para esse caminho; nenhum instalador/snapshot de binário criado.
## EAF-179 — topografia e renderer na main /31-10-2026

Commits164/166/175/177/178 confirmados ancestrais da main.6Rust emterrain,
landforms,world_space e6gruposMCP (publicação/fontes/undo/replay/vãos) aprovados
no app44dae857d2128e92. Novo exemploRust exige repetibilidade e igualdade exata
em15pares com atalhos gráficos desligados/ligados; segunda fase exercita pacotes
atmosféricos em vistas distantes.5casos: cadeia aprovada, arredondada, aguda,
terreno nativo e formas3D. Hashs dos dois assets aprovados idênticos antes/depois
de extrair scripts/mountain_recipe.py. Clippy do exemplo,fmt/Python/diff
aprovados; artefatos correntes em artifacts/terrain-render-integration e contrato
em docs/terrain-render-integration.md. Sem alteração de runtime/formato, medição
FPS, colisão/streaming ou aceite artístico novo. Fonte/cena humana preservadas.


## EAF-185 — passeio físico no terreno ampliado

Build6dda7b1233983522/macOS/Metal;155Rust/7alvos (lib129,actor_controls14,
continuous_runtime2,player_view2,terrain3,terrain_collision2,terrain_region3),
repetições finais dos alvos alterados, Clippyalltargets/fmt/Python/diff/check.
Dois testes novos de câmera protegem posição física/autoria/reset e recusa
atômica de configuração/controle alheio. Correções preliminares de fixture e
corrida de compilação documentadas em player-walk.md; critérios preservados.

Base20km systems cf63fe7:4walks +fresh39,504mgrounded; gates de bordas reais
4×533v com errosposição/material0 e raízes<=0,675mm na floresta30mil.
GUI jogávelPID6348, modo continuous/visão1,65m confirmados pelo MCP no mesmo
processo. Teclado nativo deslocou4,400mgrounded; mouse mudouyaw/pitch e Esc
soltou captura. Fotos nativas e GUI inspecionadas. Relatórios em
artifacts/expanded-world, contrato docs/player-walk.md. Janela deixada para
teste humano; não declarar FPS, arte/streaming/normal de borda/horizonte aceitos.


## EAF-191 — botânica nativa e quatro exemplares

Contrato, equações e limites em [botany-integration.md](botany-integration.md).
Build587d93bed378d69b/macOS arm64/Metal; Scene108/replay118 e fonte1/receita1.
Conteúdo7/protocolo5/checkpoint3/PSV1 preservados.9Rust novos críticos de autoria/
identidade/momento/integração material, quotas e persistência;16existentes de
editor/mesh_objects/checkpoint/portátil/identity selecionados por impacto.
Check/Clippy dos alvos relacionados, rustfmt autoral, Python e diff aprovados;
não se trata de execução da suíte inteira.

MCP real: fontes dos quatro exemplares, preview sem alteração de Scene/revisão,
replay/portátil de fixture pequena, galeria save/load com pixels exatos, painel
nativo offscreen, figueira completa39152pontos com crescimento/poda e balanço de
massa/momento. Vínculos fechados a9,164e-9m no roteiro; haste dobra1,733rad e
apoia seus pontos no chão; proxies cruzados28,633mm→0. Noveprovas detalhadas,
recibos e caminhos correntes em artifacts/botany-native. Balanço numérico não
prova calibração de tecido/veias, CCD, contato triangular exato ou gameplay Rapier.

Fonte/malha autorais, camada manual e undo juntos; dinâmica de sessão fora da
persistência/rede. Proteção nova do portátil permite a reserva vazia de identidade
para cenas só visuais e continua recusando ocupantes inválidos. Laboratório V1/V2
regredido sem alterações de contrato. Quatro arquétipos sintéticos, não espécies
calibradas ou certificação de toda a flora. Janela/cena humanas preservadas, um
executável corrente compartilhado target/release/ignis; sem push/produção/aceite
humano inferidos.
