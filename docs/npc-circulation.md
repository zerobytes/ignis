# Circulação local de NPCs

Extensão aérea posterior: [npc-air-circulation.md](npc-air-circulation.md),
EAF-159/Scene98/replay108. O recorte terrestre histórico abaixo continua válido;
voo opt-in passa a selecionar orientação dentro do motor antes do próximo passo.
Jump/glide comprometidos continuam sem correção posterior.

EAF-153. Scene92/replay102, protocolo remoto2 preservado. Autoria por
`npc.circulation` em upsert/edit_scene/templates; inspeção por
`inspect_npc_circulation`, no dispatcher compartilhado.

Decisão antes do código: acrescentar amostragem local de velocidades ao executor
existente, em vez de substituir navegação por ORCA global ou aplicar forças ao
corpo. O plano continua responsável pelo destino; a circulação só escolhe uma
velocidade física próxima da intenção. Filas/reservas de passagens continuam
autoritativas. Configuração opt-in; omissão preserva o caminho anterior.

Primeiro contrato: circulação terrestre (incluindo walk em motor híbrido).
Trechos jump/glide/fly comprometidos preservam a trajetória física; diagnóstico
explicita `committed_air` em vez de deformar voo curvado depois do controlador.
Não é promessa de multidões arbitrárias, formação ou arbitragem automática de
todos os gargalos. Esses limites continuam separados na matriz de NPC v1.

O sensor é proximidade anônima de corpos, oclusa por geometria fixa. Não produz
observação, identidade de alvo, memória visual ou audição. Vizinhos usam posições
e velocidades do mesmo snapshot anterior à física; nunca a velocidade já
escolhida pelo NPC anterior. Até128 folhas candidatas e16 vizinhos retidos,
ordenados por distância/índice estável; truncamento explícito. Até9 velocidades.
Não se mover se o planejador/fila/cast pediu velocidade zero.

Inteligência ajusta horizonte/quantidade de alternativas; cautela/medo ajustam
distância pessoal, coragem/agressividade a cedência e disciplina a preferência
consistente de contorno. São preferências limitadas pela configuração, sem
aumentar velocidade, atravessar sólidos, habilitar poderes ou descobrir alvo.
Desvios terrestres com KCC são admitidos em um passo no mesmo mundo estático,
incluindo apoio; legado planar conserva superfície/obstáculos por volume.

## Revisão estática antes da implementação

1. Reutilizar BVH de combate; sem reconstrução/varredura do mundo por NPC.
2. Array local16 vizinhos e9 alternativas; snapshot de velocidades reutilizado.
3. Autoria finita/limitada; vetores f32 no solver já existente, sem JSON no tick.
4. Sensor/score não acessam alvo, memória nem facções; estado no próprio NpcState.
5.128 folhas/16 vizinhos/9 alternativas e consulta estática de um passo; expor limite.
6. Sem nova renderização ou alocação de diagnóstico durante movimento.
7. Snapshot único evita dependência da ordem de atualização; sem workers/locks.
8. Nascimento/reset zeram diagnóstico; morte limpa, replay compara estado.
9. Uma prova crítica de integração física/identidade/privacidade/replay;
   verificações existentes de passagem/controller/híbrido por impacto.
10. Custo O(candidatos×vizinhos), sem profiling/FPS/escala1000 inferidos.

## Implementação e revisão

Snapshot de velocidades em Vec reutilizado, somente quando uma definição ou
template possui circulação. Array16 de vizinhos e9 alternativas no stack;
perfil neutro constante, sem reamostrar PRNG por frame. BVH existente consulta
até128 folhas; oclusão usa kernel Box/Wedge. Inteligência altera horizonte e
5,7ou9 alternativas, traits alteram espaço/peso/contorno. Desempate local de
mão direita; colisão/KCC continuam autoridade física.

Admissão KCC de cada alternativa limitada a2passos virtuais, com apoio inicial
e final; alternativas sem apoio descartadas. Motor legado usa superfície e
sweep de volume. Intenção original sem risco continua sem sondagem. Parada
sempre é alternativa; sem velocidade extra ou deslocamento direto. Distância
de investigação e passos continuam medindo movimento físico final.

A prova crítica encontrou uso incorreto da velocidade física anterior de corpos
KinematicPositionBased: Rapier ignora set_linvel nesse modo. Corrigido lendo a
intenção requested_velocity atual do KCC. Velocidade dos vizinhos continua
congelada antes das decisões para todos no mesmo tick.

13Rust/3alvos e6MCP/19grupos no fonte final, check releasealltargets,
Clippy lib/bin/prova-Dwarnings/fmt/Python/diff passaram. Uma prova crítica nova
cobre cruzamento físico/velocidade/privacidade/identidade/reuso/atomicidade/replay;
passagens e motor híbrido existentes por impacto. Prefixo PNG vivo/replay exato.
Buildaa595aee7c5cfef3. Demo examples/npc-circulation.scene.json salva pelo MCP.
Sem janela, foco, aceite humano ou alegação de FPS/multidões universais.
