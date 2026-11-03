# Auras pulsantes — EAF-54

## Âncora explícita

`area.persistence.anchor` aceita `fixed` (padrão, inclusive documentos antigos) ou `source`. Source exige cena/replay43, seleção `self_actor` e entrega `direct`. Os outros parâmetros de duração, intervalo, pulso inicial, reaplicação, término por morte, raio/oclusão/relação/limite continuam os de [áreas persistentes](persistent-area.md).

A âncora source é o centro físico atual do autor, em XYZ. Segue andar, salto, planeio, voo, colisões e gravidade; não usa a transformação autorada nem altera o movimento. O efeito usa os operadores genéricos existentes: cura/dano/recursos/buffs/dissipação/interrupção. Não há elemento/classe específica na engine.

A combinação direct+self_actor é uma restrição explícita deste recorte: evita aceitar ponto de mira ou projétil e silenciosamente ignorar sua posição. Aplicar aura em terceiros, prender em parte de um esqueleto ou gerar uma aura no ator atingido exigirá outro contrato de âncora/identidade. Não inferir essas capacidades de `source`.

## Relógio, efeitos e morte

Mover não muda execução, criação, expiração, próximo pulso, cooldown ou custo. Recast com refresh renova a área da mesma definição/autor e gera nova execução; independent permite múltiplas auras no mesmo autor com relógios separados. Custos continuam únicos. Raio é esférico, usa centros atuais dos destinatários e oclusores físicos existentes. Relação filtra os outros atores; `include_source` governa participação do próprio portador.

Ocupação é avaliada por pulso. Entrar não causa aplicação extra; sair evita aplicações futuras. Dano/cura anteriores permanecem. Buff concedido conserva sua duração e regras de dissipação; sair não o revoga automaticamente. Aura pulsante não é um modificador condicionado continuamente à presença dentro da esfera.

`end_on_source_death:true` encerra pela política já existente. Com false, continua ancorada no corpo morto, acompanhando queda/gravidade até expirar. Não congela no local da morte nem migra para outro ator. O pulso comprometido termina antes da remoção por morte, como a área fixa. Desabilitar IA não mata nem remove o corpo; não cancela área já entregue. Remoção/edição recompila a simulação e elimina as instâncias, como antes.

## Fases, observabilidade e replay

Antes de aplicar um pulso, o centro source é resolvido a partir do snapshot de posições usado pela fase de habilidades, anterior à física. `areas.receipts[].center` registra exatamente esse ponto. Após o passo físico, centros das instâncias source são sincronizados com os corpos, sem nova consulta de ocupantes: `persistent_areas.active[].center` e o guia representam a posição atual. Portanto recibo histórico e centro vivo podem diferir legitimamente por um passo de deslocamento.

`inspect_abilities.persistent_areas.active[].anchor_actor` identifica o autor quando source, ou null quando fixed. A configuração mostra o modo; execução e ciclo de vida mantêm identidade, prazos e pulsos. O guia J / `set_debug_view(npc_routes:true)` usa azul-claro para source e verde-lima para fixed; movimento invalida a geometria pelo cache de linhas existente. Não indica ocupantes garantidos ou acrescenta percepção ao NPC.

Autoria usa `set_ability_catalog`/`edit_scene`; execução `activate_ability`. Mesmo dispatcher para teclado/MCP. NPC usa o executor existente e pode ativar aura própria sem alvo, depois retomar patrulha durante cooldown. Alvos secundários atingidos/curados continuam sem alimentar memória/decisão. Registros de replay incluem posição de âncora sincronizada e centros dos pulsos, mantendo replay final/prefixo/captura.

## Custos e alternativas

Enum compilado, pool16 e históricos anteriores. Atualização pós-física consulta somente instâncias source (até16), por handles existentes; não cria mapa, vetor, entidade ou callback persistente. No pulso, consulta uma posição do snapshot já disponível. Seleção/oclusão/efeitos permanecem limitados como antes. Instâncias independentes no mesmo ator repetem lookup limitado; não há cache adicional para invalidar.

A alternativa de mover a esfera só nos pulsos foi rejeitada porque o guia/inspeção ficariam atrasados entre aplicações. Atualizar ocupantes todo frame também foi rejeitado: implicaria custo e semântica de enter/exit sem benefício para a amostragem escolhida. Sincronizar somente o centro após física preserva as duas fases sem alocar novo snapshot. Sem profiling, promessa de FPS, novos locks, dependências, broadphase ou scheduler.

## Demonstração

`examples/npc-aura.scene.json`: P inicia/pausa, WASD move, J alterna guias, R reinicia e X cancela preparo.

1. Aura de dano no jogador: raio3m, duração10s, pulso imediato e a cada1s. Caminhe até vermelho/azul/roxo para atingir os hostis; afaste-se para parar de atingi-los. Recast renova.
2. Aura de cura: raio4m, mesma duração/ritmo; cura60 no jogador e aliados próximos. Recast renova. Não cura inimigos.
3. Zona fixa anterior: raio2m, duração6s; causa dano inclusive ao jogador. Permite comparar a esfera que fica para trás com a aura que acompanha.
4. Buff próprio de dano;5 cura instantânea;6 dissipa resistências pessoais.

NPC verde voa numa patrulha XYZ e ativa sua própria aura de cura pelo executor comum, com duração/cooldown10s. A aura azul acompanha o corpo durante mudanças de altura. NPCs vermelho/azul/roxo conservam resistências; cinza fica atrás da parede. Guias são diagnósticos, não VFX. Configurações e demo são autoradas/salvas pelo MCP; `python3 scripts/run_verifications.py --only aura` valida sem janela. Aceite de teclado/experiência é separado.
