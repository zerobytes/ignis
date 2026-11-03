# Primeira e terceira pessoa — EAF-186

Continuação de player-walk185. A aparência do personagem pertence ao avatar,
não à câmera. Este incremento usa o corpo/avatar já configurado; Guilherme
produzirá o personagem autoral depois. O pipeline de avatar atual valida/atualiza
NPCs; integrar a escultura como avatar do Player será trabalho próprio quando
o asset chegar. Esta unidade usa o Player e seu corpo primitivo, preservando
os avatares dos outros atores. Não inclui escultura, importador, animação de
braços própria deFPS ou transformação de humano em outra espécie.

## Contrato escolhido

Visão local de sessão, no mesmo SetPlayerView/InspectPlayerView/SetPlayerViewInput.
LookPlayerView acrescenta deltas yaw/pitch finitos, normaliza yaw e limita pitch;
retorna recibo leve sem resolver colisão/serializar inspeção completa. O mouse
nativo usa esse mesmo comando, evitando consultas físicas por evento bruto.
Modo first_person é padrão quando omitido, preservando clientes185. Modo
third_person orbita o pivô dos olhos pelo yaw/pitch, com afastamento, ombro e
altura configuráveis. MovimentoXZ acompanha yaw em ambos; pitch não faz o
controlador terrestre voar. Câmera não gira/teleporta corpo nem altera Scene.
As animações/orientação de avatar seguem o sistema de apresentação existente.

O pivô é posição física menos meia altura mais eye_height. Em terceira pessoa
somar height_offset, recuar distance_meters pelo olhar e deslocar shoulder_offset
para a direita. O alvo permanece paralelo à direção do olhar, permitindo câmera
central ou de ombro. Nenhum spring/lag/interpolação temporal foi acrescentado:
o recolhimento é imediato e determinístico no snapshot; evitar transição que
atravesse sólidos tem prioridade sobre acabamento cinematográfico neste recorte.

```json
{"op":"set_player_view","view":{"actor":"player","yaw_degrees":0,"pitch_degrees":-10,"eye_height":1.65,"fov_degrees":75,"mode":"third_person","third_person":{"distance_meters":4,"shoulder_offset":0.4,"height_offset":0,"collision_radius":0.15,"collision_padding":0.03,"hide_body_below_meters":0.65}}}
```

third_person pode ser omitido (defaults acima) ou conter apenas ajustes.
Distância0,25..100m, ombro/altura−10..10m, raio0,05..2m, margem0..0,5m e
distância de ocultação0..10m. Dados finitos, pitch−85..85, FOV10..120 e olhos
dentro da altura física do Player. Configuração inválida não altera sessão.
view:null restaura câmera autoral. Estado não é persistido em Scene/replay/save;
reset/edição/load e troca de vida invalidam vínculo Reference+vida existente.
Controles explícitos continuam exigindo o mesmo ator; offline sem sockets.

## Obstrução física

A câmera varre uma esfera do pivô até o destino, usando o BroadPhaseBvh atual
e os colisores Rapier reais, inclusive TriMeshTerrain/Landform. Exclui corpo
próprio, sensores e colisores desativados. Não usa bounds visuais, LODs, nomes
de espécie ou caixas invisíveis para representar terreno. Obstáculos dinâmicos
sólidos também participam; árvores/água apenas visuais continuam sem colisão.

Uma consulta de sobreposição inicial e uma varredura linear obtêm fração do
afastamento possível. O raio usado inclui collision_padding. Sobreposição no
pivô recolhe até ele e é exposta; não se promete sair de um ator nascido dentro
de sólido. A esfera é uma proteção ao redor da câmera, não certificação do
frustum inteiro para qualquerFOV/aspect. Cenários de câmera muito larga devem
configurar raio adequado. Não há teleporte do jogador para salvar a câmera.

InspectPlayerView mantém camera e acrescenta resolved: pivot, desired_position,
fraction, obstructed, initial_overlap e hide_body. Terceira mostra o corpo;
se recolher abaixo do limiar configurado, oculta só a representação própria para
evitar câmera dentro dela. Zero desliga essa ocultação de proximidade. Primeira
sempre omite representação própria conforme185. Física continua ativa.

Primitivos registram AABBs no BVH ao nascer, como malhas já faziam, permitindo
consulta correta antes do primeiro tick. Não adiantar simulação para preparar
câmera. Regredir controles/KCC/checkpoint/save por esse risco de integração.

## Controles nativos

F liga/desliga visão do Player (câmera autoral quando desligada). Tab alterna
primeira/terceira com visão ativa. Roda aproxima/afasta somente na terceira;
clique captura mouse, mouse olha, Esc solta, WASD/setas relativos ao olhar,
Shift corre e Ctrl furtivo. Os atalhos usam SetPlayerView/SetPlayerViewInput;
V continua reservado a guias de visãoNPC. F3/HUD pertence à frente gráfica181.
Troca de modo preserva identidade/heading e input; perda de foco/contexto
conserva neutralização e exigência de novo pressionamento do185.

CLI: --first-person PLAYER ou --third-person PLAYER, mutuamente exclusivos;
--continuous continua opcional e usa o comando comum. Visão local, sem alterar
o editor de autoria. A demo grande/instância humana são compartilhadas com181;
coordenar build/abertura, não usar testes headless como aceite de controlesUI.

## Checklist estático de performance

1. Qual a frequência? Resolver uma vez por RenderFrame; primeira não faz queries,
   terceira faz até2consultasBVH. Olhar não reconstrói Scene, física ou mesh.
2. Há alocação/serde/I/O no caminho? Esfera/vetores/opções próprios na pilha, BVH e
   colisores reutilizados. JSON só em comando/inspeção. Config clona oID no
   input como185; nenhuma coleção persistente nova porframe. TravessiasParry
   usam SmallVec com armazenamento inline e podem crescer em árvores profundas;
   não alegar ausência universal de alocação dentro da biblioteca. Pose/mesh dos
   outros avatares continuam no caminho existente.
3. Qual o algoritmo? Broadphase mais BVH de malha para esfera; não percorrer
   todos os triângulos ou corpos para cada câmera. Primeiro contato apenas;
   sobreposição inicial precisa apenas de booleano. Sem ordenação/histórico.
4. Que memória é retida? Configuração e identidade escalares da sessão, sem
   BVH/shape persistente extra ou cópia de física por câmera. AABBcold no
   insert_body, sem novo trabalho porframe/tick normal.
5. Há cálculo duplicado? Frame reutiliza resolved para câmera/visibilidade;
   NativeWindow verifica contexto ativo, sem varrer câmera só para decidir
   se tecla/clique pertence à visão. Inspeção explícita resolve por demanda.
   LookPlayerView não clonaID/config nem faz queries; SetPlayerView completo é
   usado para configuração/troca/zoom e continua retornando inspeção compatível.
6. O que muda em concorrência/formatos? Mesmo lock/dispatcher, nenhum worker,
   socket/dependência ou novo formato próprio. Reservas181 preservadas.
7. Como provar riscos? Dois testes críticos agregados adicionados: colisão
   tick0/esfera/mesh/piso/overlap/atomicidade e avatares alheios/legado/reset. Reusar
   controles, KCC, checkpoints/save portátil e apresentação pelo impacto.
8. O que não foi medido? FPS/GPU/profiling e sensação cinematográfica não
   inferidos de compilação/queries. Aceite humano da imagem/movimento separado.

## Evidências e entrega

Build inicial0cd79d500f2a5a73:41Rust/7alvos selecionados passaram (player_view4,
actor_controls14, character_controller19, npc_checkpoint1, portable_save1,
npc_presentation1, npc_presentation_complete1). Check release/alltargets e
Clippyalltargets-Dwarnings aprovados. A única falha inicial era uma fixture que
convertiaNPCcomavatar emPlayer, contrariando a validação atual; corrigida para
Player real e preservação dos avatares alheios, sem alterar a regra de produção.

Após o comando leve LookPlayerView, os4testes player_view foram repetidos no
fonte combinado com a frente181; check/Clippyalltargets/build compartilhados
aprovados. O experimentoLOD de181 permanece desligado por padrão e fora deste
commit; sua validação/qualidade não são inferidas pelas provas de câmera.
Fmt dos arquivos próprios/compartilhados congelados, Python e diff aprovados.

Executável final e1f9fbcd43d20101, release/aarch64/macOS. MCP passou4grupos pelo
script verify_player_perspectives.py --landscape: FP/TPS/parede/erro atômico/Look;
movimento/Scene intacta; load/reset/vínculo inválido; composição20km.
Câmera recuou4→2,119999m diante de parede, fração0,529999,
sem mover o jogador. Caminhada na sala8,00013m e no terreno7,59967m, ambas
grounded. As capturas first-person/third-person/third-blocked1280×800 e
landscape-third-person1728×1117 são nativas e foram inspecionadas. O corpo é
o bloco provisório; não um modelo autoral ou figura importada.

Relatórios verification.json, logs check/clippy/build/tests/tests-final e
mcp-initial/mcp-final em artifacts/player-perspectives. Não é a suíte inteira
nem aceite humano. Os5roteiros gráficos históricos sem registro no agregador
continuam uma pendência separada; este roteiro está registrado e roda isolado.
O índice Git de engine/window separa explicitamente hunks186 de HUD/LOD181.
Nenhum push ou nova cópia de executável, nenhum socket no teste offline.
Processos MCP próprios encerraram limpos; não abriram/focaram/fecharam a GUI.

Abrir a demo (atualização da janela existente coordenada com nanites):

```sh
target/release/ignis --root . --scene artifacts/expanded-world/world.scene.json --window --continuous --third-person player
```

P inicia/pausa; Tab alterna com visão ativa; roda ajusta distância na terceira;
clique olha eEsc solta. F sai/entra na visão do Player; R reseta e invalida visão,
F religa depois. A primeira pessoa permanece acessível por --first-person player.
O launcher toma heading da câmera autoral; SetPlayerView permite orientar o
passeio pelo heading desejado sem alterar Scene. Aceite humano de controles e
sensação de câmera continua separado da implementação/automação.
