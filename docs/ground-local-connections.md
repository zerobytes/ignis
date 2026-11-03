# Conectores terrestres locais

Continuidade EAF-126, cena85/replay89. Entity.character_controller + npc.mobility walk-only pode usar local_connections.walk; fly continua recusado. Reutiliza Dijkstra e conectores Entry/Direct/Exit, com o mesmo KCC da execução. Nós são pés nominais mais offset, origem é o corpo físico sem teleporte e destino vem exclusivamente do NavGoal conhecido.

## Custo e fronteiras

Busca local considera os3 nós de entrada mais próximos da origem e os4 de saída mais próximos do destino dentro de max_distance (XYZ), desempate por índice. Até8 sondagens únicas (direto+3+4),2400passos virtuais no total,600por sondagem, mais2consultas de contato por sondagem. Candidatos são ordenados em arrays64 no stack. Cache transitório8 evita sondar o mesmo par novamente no plano; nenhum clone de Scene. Preparação de links estáticos por perfil permanece como EAF-126. Lançamento revalida apenas o trecho escolhido, até600passos extras. Retry60ticks sem sondagens durante a espera após falha, inclusive se a pista mudar; uma chegada já dentro do raio ainda é reconhecida.

O resultado é a rota de menor custo entre conectores efetivamente avaliados. Limitar candidatos pode excluir uma rota válida; não alegar ótimo global ou impossibilidade geométrica quando limitado. ground_planning no estado expõe probes/steps/limited. Se nenhum caminho foi encontrado e houve truncamento ou falta de passos, motivo ground_local_budget, distinto de no_local_route. Aproximação parcial permanece explícita e não prova destino alcançável. Alvos móveis atualizam o destino conhecido, preservam segmento comprometido e respeitam cooldown da busca falha; não há prazo de CPU/profiling ou garantia MMO.

Sondagens usam apenas fixos e velocidade vertical inicial zero, não reservam corpos dinâmicos. Curvas, navmesh e transição ground/air continuam fora. Entrada/saída/direto usam tamanho/inclinação/degrau/snap do indivíduo. Cast lock preserva conector e congela prazo; percepção/vida continuam. Distância de investigação usa trajetória XYZ medida. Spawn preserva apoio integral separado; retirada/reuso removem estado e novo indivíduo começa sem diagnóstico/trecho herdado.

MCP usa autoria local_connections e inspect_mobility existentes, query_ground_traversal para diagnóstico. Replay inclui ground_planning; erro/undo/reset/load mantêm semântica. Sem controle exclusivamente visual, renderer/janelas preservados; validação headless e aceite físico separados.
