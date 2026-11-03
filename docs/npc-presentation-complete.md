# Apresentação completa — EAF-160

Implementação técnica em main: Scene99/replay109/protocolo4/conteúdo1/checkpoint2.
IK, root motion planar, retarget autoral e backend opcional de dispositivo estão
implementados. Capturas/PCM/rede foram validados; aceite humano de arte e
reprodução física de áudio permanecem separados.

## Decisões anteriores ao código

Retarget foi escolhido como operação autoral que produz Clip nativo no asset destino,
por mapeamento explícito de nomes, orientação de base e escala de deslocamento.
Alternativa de resolver dois rigs em cada frame introduz dependências vivas,
mais custo e alterações no pacote remoto. O bake conserva reprodução, undo,
checkpoint e conteúdo público existentes. Rotações são relativas à pose de repouso;
hierarquia incompatível é recusada em vez de inventar correspondências anatômicas.
Não infere anatomia, constraints, contatos nem semântica por nomes humanos.

IK foi definido como correção visual de cadeias configuráveis, com contato consultado no mundo
físico e resultado público explícito; nunca mover collider para corrigir pose.
Root motion opcional entrega deslocamento ao controlador como única intenção
de movimento terrestre; não somará translação visual ao deslocamento físico.
A integração ocorreu após fdb5e58/EAF-159 liberar physics/mobility e preserva
control_with_circulation antes do passo físico.

Áudio usa o mixer PCM existente e saída opcional, desligada por padrão. Uma
dependência multiplataforma de dispositivo simplifica formatos/callbacks, mas
precisa de escolha/versão e isolamento do callback. Backend específico reduz
dependências, mas exige implementar lifecycle/formatos por plataforma. A escolha
CPAL está registrada abaixo; nenhum som foi reproduzido nesta tarefa.

## Revisão estática prévia

1. Retarget fora do tick, até32 ossos/16 clipes/4096 chaves existentes.
2. Bake compartilha geometria/rig Arc, aloca somente novo Clip autoral.
3. Validar nomes únicos, hierarquia, finitude, quaternion e escala antes de publicar.
4. Reutilizar MeshEdit/validação transacional e amostragem nativas, sem segundo player.
5. IK até8 cadeias por avatar, solver analítico limitado; sem solver iterativo global.
6. CPU skinning e orçamento visual existentes, sem promessa de FPS.
7. Callback de dispositivo sem acesso ao mundo/Service; fila limitada e silêncio em falta.
8. Identidade+vida continuam donos dos estados; reset/load invalidam áudio pendente.
9. Novas provas apenas para atomicidade/formato/estado/movimento e integração material.
10. Regressão headless e arte/alto-falantes são gates diferentes. Mundo contínuo e
save portátil permanecem contratos separados desta frente; nada indexado como identidade eterna de Scene.

## Unidade 1 — retarget autoral implementado

`edit_mesh` com `kind:retarget_clip`, `asset` destino, `source`, `clip`, `output`
e `mappings:[{source,target,basis,translation_scale}]`. Quaternion basis unitário;
translation_scale0 preserva comprimento de repouso do destino. Pais diretos precisam
corresponder e toda track animada precisa de destino. Branches adicionais no destino
ficam em repouso. Não há correspondência automática ou conversão de hierarquia.

Prova crítica `retarget_rest_relative_transaction_and_native_persistence` passou
em release/locked: mapa inválido atômico, rotação relativa com repousos diferentes,
escala de root, preservação de comprimento, undo, JSON, remoção da fonte e replay.

Revisão posterior: bake só no comando; índices resolvidos até32, chaves existentes
até4096; um novo Clip e Arc de rig/geometria preservados. Nenhum estado de tick,
worker, I/O, shader, formato ou versão novos nesta unidade. O schema MCP deriva
MeshEdit e a publicação reutiliza a transação/validação de plantas. Clones autorais
e validação completa são custo mantido. As demais unidades estão descritas abaixo.

## Inventário das interfaces de corpos e importação

O criador de corpos deverá produzir Geometry/Asset/Rig/Weight/Clip nativos e um
perfil de morfologia: cadeias de contato, pole, eixo normal da extremidade,
calibração de orientação e root opt-in. O corpo físico/controlador permanece
configuração de gameplay explícita; gerar uma silhueta não prova clearance ou apoio.
Tetos existentes:32 ossos/4 influências por vértice,16 assets/16 clipes no documento,
4096 chaves por clipe,16384 vértices por asset e orçamento visual separado. Não há
neste incremento criação universal de anatomia, auto-rig ou retopologia automática.
A receita da demo é conteúdo parametrizado específico, não aquele sistema inteiro.

Importação Blender/glTF precisa de contrato próprio para eixos/unidades, rest pose,
inverse binds, hierarquia, materiais, pesos, amostragem e validação sem poda silenciosa.
O bake de retarget recebe assets já válidos; não importa .blend/glTF, não descobre
semântica anatômica e recusa hierarquia diferente sem conversão explícita anterior.
Unidades locais dos assets e nomes mapeados não dependem de um índice eterno da Scene.
Streaming futuro deve carregar recursos por identidade estável e restaurar os
playbacks por Reference+vida, sem substituir os controladores por animações.

## Backend de áudio escolhido

[CPAL0.16.0](https://docs.rs/cpal/0.16.0/cpal/) opcional, feature `device-audio`
incluída na compilação padrão. `--no-default-features` mantém PCM/WAV headless.
Nenhum dispositivo é consultado ou aberto até `set_npc_audio_output` com enabled=true.
O stream pertence a um worker e o callback só recebe PCM por mailbox limitada8ticks;
f32/i16/u16,1..16 canais,8..192kHz, adaptação mono para canais e reamostragem por
retenção de amostra. Não é resampling de alta fidelidade/HRTF/oclusão. Saturação
é a do mixer comum, não uma interpretação nova de ruídos perceptivos.

`set_npc_audio_output {enabled,listener,radius}` cria/substitui/encerra o stream;
`inspect_npc_audio_output` publica formato/voz/fila, drops, underruns e erros.
Falta de dispositivo é erro explícito; a saída anterior só é substituída após a nova
inicializar. Cada tick do Service produz400 amostras pelos mesmos bancos/vozes/ganho
espacial da captura WAV. Executar muitos ticks mais rápido que tempo real pode
descartar blocos, contabilizados, sem atrasar a simulação. O callback não acessa
Engine/Service nem abre arquivo/rede. A mailbox std usa sincronização interna;
não se promete callback hard real-time sem locks do sistema.

Mudança de runtime_scope e pause invalidam a epoch de blocos pendentes; encerrar
saída destrói worker/stream. A fila e o dispositivo não entram em save/replay.
Cliente remoto não recebe bancos PCM públicos, portanto não habilita este backend
com uma falsa simulação local. Nenhum teste deve ligar o dispositivo sem pedido
explícito para ouvir. Compilação/callback em memória não comprovam alto-falantes.

## IK e movimento integrados

Calibração de avatar é um quaternion que converte eixos do asset para frente -Z e
cima +Y antes da orientação do executor. Cadeias são três ossos com pais diretos,
sem compartilhar articulações entre cadeias, até8/ator. Pole e eixo normal da
extremidade são dados da morfologia. A consulta usa colisão física atual, incluindo apoios móveis,
exclui o próprio ator, sensores e colisores desabilitados e recusa teto, ausência de apoio e inclinação
fora do perfil. support_radius>0 exige quatro suportes adicionais dentro de support_tolerance (padrão3cm);
zero significa contato pontual. Cinco amostras também não provam cobertura contínua de toda a sola. Distâncias de contato são metros locais do mundo; a autoria deve adaptá-las à escala da morfologia.

Solver analítico preserva comprimentos, limita alcance e alinha o eixo da ponta
com a normal do contato. IK é visual: não empurra o corpo nem participa de sensores,
navegação ou dano. Não corrige penetração profunda nem cria pé onde o rig não tem
cadeia. Limite de alcance fica explícito em ik_limited; não se declara contato
perfeito para um alvo fora do alcance. Não há foot locking temporal ou solução
global de pelve/espinha. Até40 rays/ator com footprints completos e custo de pose
CPU são limites de trabalho, sem garantia de FPS.

Root motion desta integração é translação planar de locomoção guiada pela intenção
já admitida, com clipe monotônico completo/loop e eixo de avanço configurado.
O root não recebe deslocamento de animação uma segunda vez no render: XZ é removido
da pose corrente e anterior antes do blend. KCC continua dono de gravidade, apoio,
degraus, colisão e correção física. Circulação decide sobre esse comando antes do
passo, não deforma uma posição após o solver. Velocidade nunca supera a intenção
admitida, incluindo a redução de chegada. Um clipe mais lento pode reduzir progresso
e atingir timeout da navegação; configurar velocidade/stride coerentes é autoria,
não um teleporte para cumprir prazo.

Casts com lock, morte e entrada aérea retiram a intenção root; casts livres podem
continuar a camada de locomoção física, enquanto a fase visual usa o clipe de cast.
Voo/salto/planeio conservam o executor de mobilidade; root não altera suas trajetórias.
Não há extração de root yaw, lunge automático de cast ou controle aéreo por clipe.
Esses limites não são escondidos sob flags de capacidades ausentes.

Motion mantém Reference+vida+clipe+relógio e diagnóstico do comando anterior ao
KCC, não posição real alegada. Novos nascimentos não herdam esse relógio. Estado
primário entra no checkpoint/replay; IK deriva do snapshot e publica somente até24
transforms locais de pose. Protocolo4 rejeita payload inválido antes de aplicar
réplica, incluindo índices de juntas incompatíveis com o pacote visual recebido.
Conteúdo1 continua com assets/rigs/clipes nativos; nenhuma informação de decisão
secreta ou banco sonoro é acrescentada ao pacote público.

## Unidade 2 — saída de áudio implementada

Prova `audio_device::backend::tests::callback_consumes_pcm_and_invalidates_world_without_device`
aprovada em release/locked: consumo do PCM, adaptação24→48kHz/mono→stereo e
invalidação de blocos antigos durante troca de epoch, sem dispositivo físico.
Backend/Service/Command/Cargo compilaram e passaram análise estática no conjunto
com as frentes159/162. Nenhum som reproduzido; alto-falantes continuam gate separado.

Revisão posterior: banco/vozes/mix existentes,400 amostras por tick somente com
saída ativa, mailbox8 e counters atômicos; driver/thread só ao habilitar. Callback
não consulta mundo/serde/I/O, usa sincronização std e reamostragem simples. Falhas
de inicialização não substituem saída válida; scope/pause/drop invalidam pendências.
Sem formato de Scene/replay/protocolo novo nesta unidade, sem promessa de latência
ou de suporte testado fora do Mac.


## Configuração e controles compartilhados

`Entity.avatar` e templates usam o mesmo schema/validador no documento e no
`edit_scene`; não existe caminho de configuração exclusivo da janela. Exemplo de
perfil para um membro de três ossos (índices locais do rig):

```json
{
  "orientation": [0, 1, 0, 0],
  "root_motion": {"bone": 0, "distance_scale": 1, "forward_axis": [0, 1]},
  "ik": [{
    "bones": [1, 2, 3], "pole": [0, 0, 1], "normal_axis": [0, 1, 0],
    "probe_up": 0.4, "probe_down": 0.8, "max_correction": 0.6,
    "support_radius": 0.05, "support_tolerance": 0.03,
    "sole_offset": 0.03, "max_slope_degrees": 45
  }]
}
```

Esse trecho complementa asset/variants existentes; os índices precisam corresponder
à hierarquia real, sem articulações compartilhadas entre cadeias. O pole é expresso
no espaço do avatar antes da orientação mundial. `inspect_npc_presentation` expõe
relógio/intenção/comando root e poses com joint_count/ik_limited. Cliente remoto
retorna somente poses públicas e disponibilidade de áudio, sem simulação privada.
`order_npc`, cast, morte/revive, spawn/despawn e checkpoint conservam seus comandos.
`set_npc_audio_output` e `inspect_npc_audio_output` estão no dispatcher/Service;
enabled=false encerra a saída. Nenhuma configuração habilita áudio automaticamente.

## Demonstração e roteiro

Cena canônica `examples/npc-presentation-complete.scene.json`, salva pelo MCP após
retarget real entre canid e grazer. São três rigs nativos (15/15/21 ossos), dois
quadrúpedes com proporções diferentes e uma criatura voadora de seis membros;
13.536 triângulos,16 clipes no documento. Variações por traits, idle, walk, cast,
air e dead usam clips compartilhados e playback por indivíduo/vida. O retarget
altera de fato o gait de amplitude reduzida do grazer, mantendo seus comprimentos.
Geometria articulada e morte deitada são conteúdo autoral; não um criador universal.

Roteiro opcional para o usuário (não executado com janela nesta entrega):

```sh
./target/release/ignis --scene examples/npc-presentation-complete.scene.json --window
```

P inicia: os quadrúpedes caminham ao destino com root/KCC e a criatura circula em
voo com seis membros animados até o destino, onde paira. R reinicia. Pelo MCP, order_npc permite novo destino,
activate_ability actor=a/slot=1 demonstra cast com lock; player/slot=1/target=a mata,
player/slot=2/target=a revive. Inspeções/capturas usam o mesmo runtime, sem foco.
Para ouvir, o usuário pode habilitar explicitamente set_npc_audio_output com
listener XYZ e radius; essa reprodução física ainda precisa de aceite próprio.

A receita `scripts/npc_presentation_demo.py` gera o conteúdo anterior ao bake;
`verify_npc_presentation_complete.py` refaz o retarget pelo MCP e salva a cena
canônica no workspace isolado da regressão. Não substituir a cena canônica apenas
pela saída pré-bake da receita. Verificação reproduzível sem janela/dispositivo:

```sh
python3 scripts/run_verifications.py --only npc_presentation_complete
```

Capturas em artifacts/mcp-validation/artifacts/npc-presentation-{morphologies,contact,
cast,dead,host,remote}.png; PCM em npc-presentation-complete.wav. A inspeção visual
local confirmou três silhuetas articuladas e fases distintas; não substitui aceite
artístico do usuário. Não houve foco, entrada física, áudio ou alteração da demo aberta.

## Validação e revisão posterior

Build macOS arm64 72ce80dca500d2c0, release/rustc1.97.1.49 testes Rust/10 alvos
passaram em release/locked: npc_presentation_motion, npc_presentation_complete,
npc_presentation, npc_checkpoint, meshes, character_controller, network_content,
network_transport, npc_hybrid_mobility e npc_air_circulation. A prova de callback
sem dispositivo passou separadamente (1 teste de biblioteca). Clippy all-targets
e Clippy --no-default-features lib/bin passaram com -D warnings.
17 cenários MCP/76 grupos passaram (incluindo a nova prova/6 grupos), render,
replay, checkpoint, editor de meshes, KCC, circulação aérea/terrestre, percepção,
seleção, controles remotos e QUIC. Relatórios/logs em artifacts/npc-presentation-complete-*.

A prova crítica nova protege incidência física (intenção2m/s vs root1,5m/s),
colisão com obstáculo dinâmico, retirada de um único deslocamento no skinning,
cast lock/morte/revive/reuso, relógio ativo em save/load e replay. Payload com
junta válida globalmente mas ausente no rig é recusado antes de limpar/aplicar a
réplica. No MCP: retarget real, PCM não nulo com dispositivo desligado, prefixo0
com PNG exato após checkpoint e igualdade de poses host/cliente com conteúdo real.
Checkpoint2 continua nativo e condicionado a Rapier/OS/arquitetura; EAF-161 é o
contrato separado de save portátil. Nenhuma garantia de streaming é inferida.

1. Opt-in derivado de definições/templates evita trabalho root quando ausente.
2. Assets/rigs/clips Arc compartilhados; Motion Copy por Reference+vida; nomes só
   alocados no início da locomoção, traits/variante resolvidos por episódio.
3. IK usa arrays32/24 e até8 cadeias/40rays, sem laço iterativo convergente.
4. Consultas do mesmo snapshot físico; excluem corpo/sensores/disabled, aceitam
   apoio móvel e exigem normal/alcance/footprint configurados.
5. Root entra depois da intenção e antes de circulação/interrupt/KCC. Uma primeira
   colocação antes da intenção foi detectada pelo teste e corrigida.
6. Skinning CPU, busca limitada de asset/track, recomposição de matrizes e clone
   preexistente de StateSet permanecem custos explícitos, sem ganho de FPS alegado.
7. Correção é derivada por inspeção/snapshot; só transforms públicos vão na rede;
   validação contextual precede qualquer publicação da réplica.
8. Motion e animation_started são primários versionados; retomada de save não
   reinicia indevidamente o gait. Cache root opt-in é recomposto no construtor.
9. Áudio continua limitado8ticks/64vozes/400amostras, worker só quando habilitado;
   callback não adquire lock do mundo nem acessa filesystem/rede.
10. Sem IK de pelve/footlocking, root yaw/lunge/air, retarget de hierarquia arbitrária,
    auto-rig/importador universal ou dispositivo/plataformas extras validados.


## Correção do roteiro — deslocamento do voador

Relato humano: roxo animava sem sair da origem. Reprodução isolada confirmou
posição[-3,3,-4] constante até480ticks e motivo not_anchored_to_network.
A cena tinha guard_3d e velocidade aérea, mas não tinha grafo nem conector local
opt-in para o voador. O clip Air indicava modo aéreo, não progresso de navegação;
a prova anterior verificava pose/voo animado, sem exigir deslocamento desse ator.

Revisão prévia desta correção: apenas dados de demo/receita e prova MCP existente;
reusar local_connections.fly do executor validado, max_distance8, sem alterar
física/formatos/algoritmos/dependências nem adicionar worker/cache/clock. Após:
conector direto real3m/s, chegada com tolerância0,2m e hover no destino; custo de
planejamento local já limitado pelo contrato de mobilidade. Sem benchmark/FPS novo.
A verificação agora exige avanço físico >2m e fase traversing no primeiro segundo,
além da pose Air. Não foi criado teste Rust para espelhar configuração trivial.

Verificação corrigida passou: npc_presentation_complete,1cenário/6grupos,
release72ce80dca500d2c0 sem janela/áudio; checkpoint/replay/pixels e QUIC continuam
exatos agora com o voador em deslocamento real. Probe físico: x=-3 no início,
x≈0 em60ticks e x≈2,8 em180ticks (arrived, alvo3/tolerância0,2m).
Cena corrigida novamente salva pelo MCP; relatório artifacts/npc-presentation-flyer-report.json.
Sem novo build nem manipulação da janela aberta. A instância já aberta conserva
os dados antigos; R reinicia essa mesma cena em memória, portanto reabrir o arquivo
é necessário para carregar a correção.
