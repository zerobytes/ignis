# Agendas, necessidades e interações — EAF-155

Contrato antes do código, base974db42/Scene93/replay103. Próximo campo94/replay104.
Continuar com objetivos, navegação física, recursos e fase de comandos existentes.
Não criar relógio de parede ou LLM obrigatório. Agendas usam tick/ciclo; necessidades
referenciam recursos registrados pelo jogo. Pontos de interesse são mapa autorado
conhecido; não são consulta à posição de NPC escondido.

Decisão estrutural: interações não são habilidades fictícias contra objetos sem
atributos de ator. Um contrato próprio valida alcance/condições e prepara um lote
de até8efeitos tipados: recursos próprios, chaves de estado do jogo, propriedades
explicitamente registradas de objetos e eventos. Usar write sets existentes para
aplicar tudo ou nada. Nada de reflexão sobre memória Rust ou mutação livre de Scene.

Objetos mutáveis preservam identidade/posição e documento autoral. Estado explícito
solid/visible/blocks_vision/sound_transmission mantém independência entre render e
percepção. Fechamento não pode materializar sólido sobre um corpo. Colisor real e
definição de execução são atualizados juntos; mundo estático/cache KCC/rede gerada
devem refletir a geometria nova. Sobreviventes conservam vida, memórias, buffs e
cooldowns; trechos aéreos comprometidos conservam endpoint físico congelado ao
trocar grafo, sem índice pendurado. Sem reset global da simulação para abrir porta.

Projeção remota já transmite presença/kind/aparência de objetos pelo protocolo2;
reusar reposição ALL existente e corrigir reaproveitamento de Appearance apenas
quando metadados ainda coincidem. Não inventar protocolo3 para campos já suportados.
Esta unidade não cria permissão remota implícita para comandos administrativos de
interação; controles locais/MCP e intenção autônoma usam o mesmo contrato.

Diálogo é grafo de nós/escolhas/condições/efeitos, com sessão por participante,
Reference+vida das duas pontas, serial e prazo. Escolha revalida condições antes de
mutar; novo indivíduo/vida/reset invalida sessão antiga. Texto e eventos do jogo não
exigem rede/LLM. IA pode escolher por pesos autorados de personalidade, dentro de
orçamento/cadência de inteligência; nenhuma escolha lê posição oculta por isso.

Agendas/necessidades entram como objetivo routine, interrompível por prioridades
autorais. Recursos próprios fornecem urgência; perfil comum limita candidatos,
reação e preferências de esforço/segurança/rotina. Aproximação usa corpo/capacidades
e executor existente; interação só acontece no alcance físico. Uma falha não concede
recursos nem completa agenda. Pontos inalcançáveis têm cooldown, não loop de buscas
sem limite. Morte/reuso não transferem agenda/sessão; mesma pessoa preserva recursos.

## Revisão estática antes do código

1. Compilar nomes/recursos/ações/condições fora do tick; pontuar até64pontos conhecidos.
2. Estado por ator dimensionado uma vez, arrays/write sets até8efeitos; sem Scene clone por score.
3. Recursos/chaves i64 limitados, índices pequenos e prazos u64 saturados; sem JSON/reflexão no tick.
4. Reusar population/objetivos/recursos/geometry/KCC; não duplicar simulação de habilidades.
5. Documento até64pontos/32ações/64valores; diálogo16grafos×32nós×8escolhas, recibos256.
6. Mudanças físicas agrupam reconstrução estática fora da pontuação; GPU só para prova offscreen.
7. Mesmo tick/FIFO/snapshot e executor, sem threads/locks/serviço obrigatório.
8. Referência+vida+serial nos comandos/sessões; erro não publica lote parcial.
9. Uma prova crítica integrada de atomicidade/identidade/objeto/rede/replay, gates por impacto.
10. Explicitar custo de rare geometry rebuild e armazenamento de sessões/cooldowns;
    sem prometer FPS, navmesh universal ou sistema completo de quests.

## Implementação e limites

Scene94/replay104/protocolo2. SetInteractions/Interact/TalkNpc/ChooseDialogue/
CloseDialogue/InspectInteractions/InspectDialogue/InspectNpcRoutines compartilham
comandos/MCP. Autoria em edit_scene preserva lote/undo único. Exemplo
examples/npc-routines.scene.json: dois indivíduos vão ao ponto de recarga e
escolhem ramos opostos por coragem/medo em agenda uma vez por ciclo.

Documento64valores/64eventos/64pontos/32ações/16diálogos; grafo32nós/8escolhas,
condições/efeitos8. IDs compilados para índices fora do tick. Objetos mutáveis
somente fixos não usados como piso; quatro propriedades são independentes.
Collider disabled deixa de entrar na publicação BVH manual do KCC e seu filtro
ignora proxies ainda antigos. Fechar sobre corpo rejeita todo o lote.
Appearance remota só reaproveita metadados iguais; prova por bytes não é WAN.

Sessão conserva um resultado terminal por participante, separado do ring256.
Serial/revisão/Reference/vida/prazo evitam débito duplicado e retarget após morte.
Opções automáticas ponderam traits e condições atuais, sem prever ocupação oculta.
Necessidades usam próprios pools e histerese; agendas tick/ciclo com fase seeded.
Conhecimento dos pontos estáticos é autorado, não posição secreta de atores.
Até8necessidades/8janelas,64candidatos e16interações automáticas/tick com cursor
justo. Caminho impossível tem cooldown; não supor que um ponto distante é acessível.
Same-individual revive conserva recursos e conclusão da agenda, reinicia viagem e
rate clocks; novo nascimento não herda. Diálogos não são um sistema de quests/LLM.

## Revisão estática depois do código

1. Chaves/ações/tags/recursos compilados; rotina usa índices e Profile quantizado.
2. Writes locais8 e estado pequeno por NPC; buffers de cooldown/sessão só se necessários.
3. Prazos u64 saturados e valores i64 limitados; serde/JSON apenas nas interfaces.
4. Objetivos/navegação/recursos reutilizados; nenhuma segunda física/relógio.
5. Ring256 é diagnóstico; resultado de sessão persistido separadamente para gameplay.
6. Geometria recompõe cache uma vez por fase suja; custo raro explícito, não por score.
7. Snapshot pré-física movido via mem::take e devolvido ao fim, sem clone de Scene/mapa.
8. Planejamento de lote completo antes de commit; fechamento ocupado não cobra recursos.
9. Uma nova prova crítica integrada e regressões proporcionais; fixture antiga de
   ruído ajustada à quota por fonte, preservando rollback por exaustão de serial.
10. Custos mantidos: rebuild estático/grafo, pontuação limitada e CPU skinning já existente;
    sem alegar FPS ou IA universal. Checkpoint durável e escala NPC ainda separados.

## Evidência

52 testes Rust em6alvos (actor_controls14, ground_local_connections15, híbrido1,
prova integrada1, ressurreição16, edições5); 11MCP/37grupos, todos headless.
Check release all-targets, Clippy lib/bin/prova -Dwarnings, fmt/Python/diff.
Captura300ticks igual byte a byte no replay; geometria dinâmica preserva voo
curvado mesmo após retirar/reusar a fonte do bake, e réplica reflete Fixed→Visual.
Nenhuma janela/foco/áudio físico. Aceite humano separado; programa149 continua.
