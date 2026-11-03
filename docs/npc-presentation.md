# Avatar e sinais sonoros — NPC v1

Extensão atual: [EAF-160](npc-presentation-complete.md) implementa IK configurável,
root motion planar opcional, retarget autoral e saída local opcional de áudio.
As restrições sem root/dispositivo abaixo descrevem o recorte histórico EAF-156.

Preflight sobre8907c54/Scene94/replay104/protocolo2. Usar assets/rig/clipes nativos,
sem novo formato de importação nem suposição humanoide. Configuração de avatar
no ator/template referencia asset e variantes por fase. Estado de playback pertence
à Reference+vida, nunca ao asset compartilhado. Física/visão permanecem no corpo;
nenhuma animação move root/collider ou concede habilidade.

Projeção física: idle/walk/air/cast/dead; variante ponderada por traits do perfil
comum e desempate seeded por indivíduo/vida. Inteligência não reduz a qualidade
visual: ela já determina decisões; apresentação espelha o resultado, sem outro cérebro.
Transição reinicia clipe; velocidades/posição/orientação vêm do executor. Configurar
blend entre poses anteriores/novas, com teto; sem IK/root motion/retarget universal.

Renderer mantém skinning CPU existente e compartilha Arc de geometria/rig/clipe.
Instâncias de avatar derivadas não consomem32instâncias autorais; orçamento visual
separado e truncamento explícito, sem cortar simulação. Não construir um Scene ou
Document com centenas de clones a cada frame. Rede precisa campos públicos novos:
versionar protocolo e conteúdo; rejeitar versão incompatível, sem esconder pose em label.

Áudio de apresentação é separado de ruído sensorial: PCM mono24kHz registrado pelo
jogo, sinais de fase e passos com cadência, vozes limitadas. Mix/captura WAV são locais
sem saída automática no dispositivo. Não usar ring de diagnóstico de habilidades
como fonte autoritativa. Sem streaming/voz procedural/alto-falante comprovados.

## Checklist estático prévio

1. Resolver assets/clipes/sons fora do tick; variantes até8/fase, índices pequenos.
2. Estado por indivíduo/vida, sem mutar asset compartilhado; buffers visuais reutilizados.
3. Ticks u64/elapsed limitado; pesos de personalidade inteiros, PCM i16/soma i32.
4. Mesmo executor, pose/voz derivadas depois do passo físico, sem alterar percepção.
5. Orçamento de triângulos/vozes/captura finito e inspecionável; truncamento só visual.
6. CPU skinning existente, custo por vértice/influência assumido; sem FPS prometido.
7. Rede transporta somente pose/aparência pública, sem estado secreto/Scene.
8. Vida/geração invalidam playback antigo; parsing completo antes de publicar réplica.
9. Uma prova crítica integrada de reuso/morte/formatos/replay/réplica; demais gates por impacto.
10. Render/PCM offscreen, aceite humano/dispositivo separado; offline permanente.

## Implementação e revisão estática

Scene95/replay105/protocolo3 (conteúdo visual1 preservado, sem novos campos estáticos).
Entity.avatar aponta asset e até8variantes/fase; SetAvatarAudio/InspectNpcPresentation/
CaptureNpcAudio via dispatcher/MCP. Gameplay conhece idle/walk/air/cast/dead genéricos,
sem anatomia/classe humana. Troca de execução de cast reinicia animação mesmo sem
intervalo idle; traits usam perfil comum ou a mesma resolução seeded antes do primeiro tick.

1. Assets16/clipes16/variantes40 são buscas limitadas de nomes já validados; custo
   conscientemente mantido nesta versão, sem JSON/Scene clone por tick. Índices de
   pose/som são compactos. Cache de resolução adicional não foi exigido para avançar.
2. Estado próprio Reference+vida+execução de cast. Geometria/rig/clipes Arc compartilhados;
   Frame reusa buffers/base_vertices e não acrescenta instâncias autorais fictícias.
3. PCM mono24kHz/i16; soma de até64vozes em i32 antes de clamp único. Captura<=1segundo
   por chamada, banco16sons/20segundos total e4segundos por som. Sem I/O no tick.
4. Projeção posterior ao passo físico, sem root motion ou feedback sobre visão/audição.
5. Triângulos131072 de autoria+avatares; se exceder, corpo primitivo reaparece e
   omitted_avatars fica explícito no frame. Nunca apagar personagem por culling visual.
6. CPU skinning/cálculo de normais por avatar, blend local entre clipes; sem IK,
   retarget automático, GPU skinning ou garantia de FPS. Stream público de poses
   completo por snapshot é custo conhecido; deltas de pose ainda não implementados.
7. Protocolo3 publica só asset/clip/tick/blend/orientação/transform/indivíduo/vida.
   Conteúdo1 já transporta assets/rig/clipes; banco PCM e decisões privadas não viajam.
8. Pose inválida/versão2/owner-life ausente rejeitam antes de aplicar réplica. Editor
   valida dependência do asset; erro de autoria não altera mundo. WAV usa escrita
   atômica/resolução de caminho do Service, sem I/O arbitrário no domínio.
9. Uma prova crítica nova de identidade/vida/codec/replay/áudio; demais gates por impacto.
10. Captura WAV mistura apenas vozes já ativas no instante consultado; não simula sons
    futuros. Áudio espacial tem atenuação por raio, sem HRTF/oclusão física/dispositivo.
    Recursos abstratos/dados do jogo podem mapear sinais; isso não é criador de corpos.

## Evidência

Build33966d1daf4e1e66,29Rust/6alvos incluindo3transporte QUIC/TLS,6MCP/23grupos,
checkalltargets/Clippy lib/bin/prova-Dwarnings/fmt/Python/diff. Dois perfis opostos,
voador/cast/dead/revive/reuso e rig compartilhado; prefixo20ticks/PNG byteexact no
replay. Exemplo examples/npc-presentation.scene.json e WAV em artifacts/mcp-validation.
Sem janela/foco/alto-falantes/aceite humano; população ampliada/checkpoint/programa149 abertos.
