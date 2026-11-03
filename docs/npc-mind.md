# Cognição e personalidade dos NPCs

Contrato EAF-149/primeira unidade. Scene89/replay99 após88/98. NpcConfig.mind opcional, autoria pelo dispatcher/MCP/upsert/edit_scene/templates. Sem controle exclusivo visual, LLM/rede obrigatórios ou alteração do protocolo remoto2 por diagnóstico privado.

Perfil: intelligence0..1, personality com anxiety/caution/concern/fear/courage/curiosity/aggression/sociability/discipline0..1, seedu64 e variation0..1. Defaults0.5 e variation0. Variação determinística por indivíduo gerada uma vez, quantizada em pontos-base, clamp explícito0..10000; não modifica template nem consome RNG de outros sistemas.

Prioridade autoral precede utilidade; conhecimento/condições/recursos/relação/capacidade continuam filtros obrigatórios. Contexto só usa vida própria e evidências aceitas, nunca consulta posição oculta ou vida alheia. Scores adaptam fugir/responder/habilidade/rotina/esperar. Histerese/cadência evitam oscilação; invalidação é imediata. Coeficientes não tornam NPC mais rápido/forte, nem prometem melhor decisão em toda situação.

Planejamento local usa esforço derivado da inteligência dentro dos tetos existentes; amostragem menor aparece como truncamento. Perfil não altera geometria, clearance, custos pagos ou budgets de viagem declarados. Onisciência continua somente quando explicitamente autorada.

Matriz do programa em npc-v1-closure.md. Esta unidade não implementa implicitamente avatar/persistência/navmesh/escala.

## Integrações efetivas

Perfil comum cacheado em NpcState.mind_profile por nascimento, com identidade estável; morte/retorno conserva ou recompõe a mesma amostra, novo indivíduo recebe seed por InstanceId. Objetivos usam utilidade/histerese/cadência. Pontos de investigação reduzem a contagem autorada dentro dos seus limites, com mind_search_points visível; raio/prazo/conhecimento não são ampliados. Conectores KCC usam1..3entradas/1..4saídas/600..2400passos segundo inteligência; GroundPlanning registra limites efetivos. Custo/clearance e lançamento físico não são enfraquecidos.

`inspect_npc_mind` retorna configuration, trait_order, profile quantizado, decision/scores/next_think_tick/reason, search_points e planning, sem mudar execução. `inspect_objectives` inclui configuração e decisão; autoria usa mind via upsert/edit_scene/templates. Omissão conserva comportamento anterior. Sem objetivos autorados, perfil continua disponível para investigação/planejamento mas não inventa ações/habilidades.

Demo examples/npc-mind.scene.json: azul corajoso aproxima, rosa medroso recua diante do mesmo jogador amarelo visível. Coeficientes coexistem, sem enum de personalidade. Uma prova Rust crítica protege erro atômico, versão, perfil por indivíduo, conhecimento oculto e replay; restante usa cobertura existente por impacto, não testes por coeficiente.
