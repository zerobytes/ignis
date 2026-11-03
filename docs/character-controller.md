# Controlador terrestre opt-in — EAF-124

`Entity.character_controller` opcional escolhe motor cinemático de posição para jogador ou NPC terrestre; omissão preserva corpo dinâmico e aritmética anterior. Autoria usa `upsert`/`edit_scene`/templates pelo dispatcher/MCP. Cena82/replay86 reservados após ambiente76/80, controles77/81 e união do editor81/85 em worktrees isoladas. Colisão inicial de reserva77/81 corrigida antes do commit; esta frente não incorpora essas capacidades pela numeração.

## Correção de contato inicial — EAF-146

`kcc-contact-stability.md` registra o adapter SAT readonly comum a runtime e
probes. Evita afundamento de cuboid/cuboid em repouso diagonal no piso, incluindo
pose inicialmente tocando. Replay98 distingue a física; Scene88/protocolo2.
Não recupera corpos autorados profundamente dentro de sólidos. `sliding` continua
sendo diagnóstico bruto do Rapier e não certifica a inclinação do apoio.

## Contrato

Caixa alinhada ao mundo, +Y vertical. Configuração em metros/graus: `max_slope_degrees`30, `slide_slope_degrees`35 (máximo permitido85 e slide >= climb), `step` opcional `{max_height:0.3,min_width:0.4}` (omitido desabilita), `snap_distance`0.15 (zero desabilita), `offset`0.01, `gravity`9.81, `max_fall_speed`50, `push_dynamic_bodies`false. Todas as propriedades numéricas são finitas e limitadas; offset também limitado a1/4 da menor meia dimensão. Step não inclui corpos dinâmicos. min_width é a distância de clearance frontal sondada pelo Rapier ao subir, não uma garantia de apoio integral da pegada.

Mesmo caminho recebe intenção XZ do jogador ou planejador NPC. Acumula velocidade vertical com gravidade e teto de queda; contato de chão zera queda; não há comando de salto/velocidade ascendente neste motor. Cast lock zera intenção horizontal, não gravidade. Próxima posição cinemática vem da correção Rapier0.30.1. `inspect_character_controllers`/snapshot expõem identidade, ativo, grounded, sliding, velocidade vertical, deslocamento desejado/corrigido e contagem de colisões do último tick. Movimento real também permanece em runtime_positions; orientação do mesh não muda.

Morte converte o corpo para dinâmico, mantendo posição/velocidade física; controlador inativo não dirige cadáver. Ressurreição restaura cinemático e reinicia estado vertical. Spawn inicializa estado novo, despawn remove; não herdar estado por slot. Reset/edição/undo/load reconstroem. Reprodução compara estado do controlador além das posições.

O controlador aceita contato parcial de chão conforme Rapier. grounded indica contato, não certifica que a inclinação é caminhável sob max_slope_degrees. Isso é separado do apoio integral exigido no nascimento e na navegação: não muda `walk_support`/`query_spawn_support`. Não despenetra corpos autorados dentro de sólidos. AABBs publicados antes das consultas incluem corpos recém-criados; a física ainda processa seus contatos normalmente.

## Fronteiras e tradeoffs

Na entrega original82, NPC com `mobility` era recusado. [EAF-126](ground-navigation.md) permite controller+mobility exclusivamente walk pela rede em cena84; a extensão [EAF-127](ground-local-connections.md) admite conectores locais walk em cena85, com sondagens limitadas. Jump/glide/fly e transições continuam recusados e precisam contrato próprio. NPC planar usa controlador para executar suas intenções existentes; seu planejador ainda contorna rampas/degraus como obstáculos e rejeita alturas incompatíveis. O motor terrestre em rede de EAF-126 admite ligações por meio do KCC. Subir fisicamente não comprova planejar uma rota inclinada. Passos continuam vinculados à superfície autorada, sem inventar sons em rampas não configuradas.

`push_dynamic_bodies` aplica impulsos calculados pelo controlador com massa de referência igual ao volume da caixa a densidade1. Não habilita knockback recebido: cinemáticos não respondem a impulso externo como dinâmicos. Colisão do tick é consultada no snapshot corrente; dois personagens não recebem arbitragem conjunta/CCD relativo, reserva ou garantia de não sobreposição. Plataformas móveis, multidões e integração de navegação inclinada são próximos contratos distintos.

Alternativa de trocar todos os atores por cinemáticos quebraria gravidade/impulsos/motores aéreos existentes; opt-in preserva esses contratos. Integração Rapier evita recriar solver, ao custo de casts de autostep e alocações internas de manifolds. Sem promessa de escala ou FPS.

## Exemplo de autoria

```json
{"character_controller":{"max_slope_degrees":30,"slide_slope_degrees":35,"step":{"max_height":0.3,"min_width":0.4},"snap_distance":0.15,"offset":0.01,"gravity":9.81,"max_fall_speed":50,"push_dynamic_bodies":false}}
```

Step tem altura0.01..1m e largura0.01..4m; snap0..1m; offset0.001..0.1m, também limitado pela caixa; gravidade0.1..100m/s² e queda1..200m/s. Ausência de step desabilita autostep. A configuração não modifica a velocidade do jogador/NPC: locomotion/npc.speed continuam sendo a fonte da intenção. Deslocamento corrigido é o comando aprovado pelo motor, enquanto runtime_positions é o resultado publicado pela física.
