# Área instantânea 3D — EAF-52

## Contrato

`Definition.area` opcional, cena/replay41. Ausência conserva o executor anterior. Campos obrigatórios: `radius`0.1..100m, `max_targets`1..9, `include_source`bool, `occlusion`bool. Não há enum de elementos. Efeitos existentes de dano composto, cura, recurso, buff, dissipação e interrupção são executados por destinatário. Todos devem usar `recipient:target`; efeitos na origem/ruído e combinações com periodic/channel são recusados para evitar multiplicação implícita. Áreas persistentes são uma extensão opcional versionada em [persistent-area.md](persistent-area.md).

- Direta `self_actor`: centro atual do emissor no commit.
- Direta `actor`: centro atual do alvo-âncora no commit, mantendo alcance, relação, vida e evidência visual da seleção existente.
- Direta `spatial`: `aim_ability` com Point ou `aim_ability_at` (cursor/plano explícito); ponto congelado no pedido, `range`0.1..100 revalidado no commit. Direction é inválida; Point pode coincidir com o emissor. Oclusão é radial a partir do ponto, não uma regra de colocação emissor→ponto: coordenadas explícitas dentro do alcance podem estar atrás de paredes. Política automática NPC continua exigindo observação; uma restrição adicional de colocação não foi introduzida.
- Projétil `actor`/`spatial`: centro físico do projétil no primeiro contato do sweep, inclusive parede, aliado ou cadáver. O contato dispara a área; a relação filtra destinatários, não superfícies de detonação. Expirar por distância sem contato não detona. Sem reembolso. O centro fica na superfície expandida pelo half_extent, não é deslocado artificialmente através da parede.

## Seleção e transação

Na entrega, consultar os centros físicos dos atores em XYZ (esfera, não cilindro nem interseção com o volume inteiro do corpo). Fronteira inclusiva. Excluir origem quando `include_source:false`, mortos, relação incompatível, centros fora do raio e ocluídos; depois ordenar por distância quadrática e ID, selecionar os primeiros `max_targets`. Origem incluída ignora o filtro de relação, que se aplica aos outros atores. `relation:null` aceita qualquer relação. IDs ordenados independem da ordem de autoria.

Oclusão usa corpos físicos fixed/dynamic axis-aligned, inclusive corpos invisíveis, no segmento centro→centro. Atores não funcionam como escudos radiais; `blocks_vision` não interfere. Somente cruzar o interior do corpo bloqueia: tocar/tangenciar sua superfície não basta, permitindo centros na superfície do chão; um centro dentro de sólido fica bloqueado. Não modela propagação de pressão, raio contornando paredes nem transmissão parcial. A opção desativada permite atravessar geometria.

Congelar a lista antes de aplicar. Custo e cooldown pagos uma vez no commit direto/lançamento, mesmo sem destinatários ou com todos falhando. Cada destinatário tem uma transação atômica dos efeitos na ordem da definição. Falha (ex.: recurso ausente, nada dissipável, pool cheio) descarta todos os efeitos daquele destinatário e mantém os demais; não tenta substituir sua vaga. Não há rollback coletivo.

As transações são sequenciais na ordem selecionada. Cada uma avalia atributos no seu início, assim como o executor anterior. Buffs/dissipações/vida/capacidade modificados numa transação são visíveis às seguintes; modificações dentro da mesma transação só afetam transações posteriores. Isso inclui a origem: se incluída e morta/dissipada antes de outro alvo, seus buffs já não contribuem para esse outro alvo. Sem congelamento ofensivo implícito de toda a explosão. A entrega já comprometida continua mesmo se matar/interromper a origem.

## Observabilidade e conhecimento

Autoria via `set_ability_catalog`/`edit_scene`; execução usa os mesmos comandos compartilhados com teclado/cursor. `inspect_abilities.areas` contém ring64 com descarte: execução, autor, slot, tick, centro/configuração e candidatos com distância², status/exclusão e falha específica. `receipts` mantém os efeitos individuais `area_applied`, com mesmo execution_id e sem nova cobrança; `completed`/`launched` contabiliza o custo único. Até256 recibos individuais; rings independentes podem descartar detalhe antes do resumo.

NPCs compartilham executor. Política automática espacial continua baseada em ponto observado congelado, com revalidação de evidência/alcance; não passa a escolher agrupamentos nem consultar o resultado físico para saber posições ocultas. Um alvo oculto pode sofrer uma explosão sem se tornar conhecido pelo NPC. Inspeção MCP é diagnóstico privilegiado, não sensor de gameplay.

`set_debug_view(npc_routes:true)` / J mostra esfera nominal em três círculos: ciano durante preparo direto; laranja após entrega, por60ticks, últimas8 áreas. Não é VFX ou prévia de destinatários garantidos. Projéteis mostram a esfera depois do impacto; sem predição de colisão. A configuração visual existe no dispatcher antes da interface; guia não modifica a simulação.

## Limites e tradeoffs

Array fixo9 candidatos, seleção/ordenação sem heap, até9 transações de8 efeitos e8 componentes cada. Scan de corpos por candidato quando solicitado. Plano existente reutilizado, índices compilados e rings limitados. Optou-se por atomicidade individual em vez de reservar todos os buffs/interrupções/recursos num plano coletivo; uma falha incidental não anula a habilidade inteira. Esses limites são do laboratório atual, não evidência de escalabilidade para multidões. Sem física radial, knockback, volume arbitrário, área persistente ou nova ABI.
