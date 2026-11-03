# Instâncias e população — proposta EAF-59

Continuidade EAF-67: retirada transacional de NPCs publicada em [population.md](population.md), cena46/replay50. Catálogo de templates e nascimento são o próximo incremento; comandos não reconstruirão sobreviventes.

Continuidade EAF-63: a fronteira de presença/configuração e a retenção compartilhada de origem foram implementadas conforme [runtime-world.md](runtime-world.md), com cena46/replay49. Spawn/despawn público, instalação dinâmica dos componentes e ampliação da população continuam pendentes. As propostas/limites abaixo devem ser lidos junto desse contrato posterior.


Estudo sobre o código073200e, checkout main apósddb0c47, em25/10/2026. EAF-58 recebeu aceite humano. Este documento detalha o próximo contrato, não implementa spawn/despawn nem aprova implicitamente as escolhas abaixo. Na data do estudo, cena/replay eram46. Direção: começar por identidade e ciclo de vida, mantendo NPCs como eixo e coordenando a frente gráfica.

## Direção aprovada e primeiro incremento

Após discussão de retenção/memória e tráfego de leitura/escrita nos cenários81/601 atores, Guilherme autorizou prosseguir. EAF-60 entrega a fundação de identidade e os comandos existentes de habilidades; [contrato implementado](instance-identity.md), cena46/replay47. As políticas de população abaixo orientam a continuação; spawn/despawn, contexto de origem e crescimento de capacidade ainda não estão implementados. A aprovação não transforma estimativas de carga em medição ou garantia de FPS.

Requisitos adicionais aprovados para evoluir escala:

1. Indexar contribuições por dono/atributo, evitando varrer o pool global de buffs a cada consulta. Preservar ordem determinística e snapshot semântico por transação.
2. Planos transacionais devem preparar apenas os campos tocados. Hoje `abilities::Plan` copia saldos e vitalidade globais; aumentar a população mantendo esse padrão multiplicaria tráfego por efeito.
3. Reservar a capacidade de retenção da origem ao admitir entregas que possam sobreviver ao autor. Falta de capacidade rejeita a nova entrega antes de cobrança/efeitos conforme sua fronteira transacional; despawn não deve descobrir essa falta e bloquear reposição de minions. Contexto compartilhado por origem, referências limitadas e expiração original; detalhes de pool/transferência serão validados na implementação.

## Conclusão da auditoria

O obstáculo não está apenas em inserir/remover um rigid body. O catálogo ordena atores pelos IDs autorados; vários sistemas guardam slots de ator, enquanto outros guardam índices de entidade ou índices locais de volume. Reordenar a cena ou reutilizar essas posições faria referências antigas apontarem para outro indivíduo. Trocar apenas `u8` por um handle nas habilidades deixaria o problema em buffs, ruídos, recibos e render.

Além disso, `Engine::replace` reconstrói a simulação e o renderer percorre entidades autoradas com fallback para a posição inicial. Reusar edição como spawn reiniciaria outros atores; remover só o corpo poderia deixar uma caixa fantasma no desenho e nas consultas geométricas. Precisamos de um conjunto explícito de instâncias presentes, separado do documento.

| Dependência atual | Evidência verificada | Migração necessária |
|---|---|---|
| Atores e relações | [targeting.rs](../src/targeting.rs), `Catalog::new/entity/id`, MAX_ACTORS9 | Catálogo não pode reordenar slots existentes a cada nascimento; relações consultam instâncias atuais. |
| Recursos/casts/intenção | [abilities.rs](../src/abilities.rs), `State`, `Entry`, `Cast`, `LifeStamp` | Resolver identidade + geração antes de acessar arrays; vida continua separada da geração. |
| Buffs/atributos | [attributes.rs](../src/attributes.rs), `Instance.owner/source`, `Runtime::base/evaluate`, `State::inspect` | Hoje usam índices u16 da cena, inclusive na inspeção histórica. Migrar dono e autoria; valores base devem vir da instância ou contexto de origem. |
| Memória e rádio | [perception.rs](../src/perception.rs), `Observation`, `known_lives/known_dead`; [communication.rs](../src/communication.rs), `Report` | Identificar indivíduo e vida sem confundir o novo ocupante com a pista antiga; memória congelada pode sobreviver à ausência física. |
| Projéteis | [projectiles.rs](../src/projectiles.rs), `Shot`, `advance`, `Receipt` | Origem não pode excluir da colisão o novo ocupante do mesmo slot; contatos/recibos deixam de depender do índice da cena. |
| Zonas/periódicos | [persistent_area.rs](../src/persistent_area.rs), `Instance`; [periodic.rs](../src/periodic.rs), `Instance` | Separar autoria, vínculo ao destinatário e âncora física. Desaparecimento não equivale a morte. |
| Ruídos/passos | [noise.rs](../src/noise.rs), `Noise.emitter_index`; [footsteps.rs](../src/footsteps.rs) | Autoexclusão compara a instância emissora original, nunca um índice reciclado. Ruído já emitido mantém posição/tempo. |
| Física/contatos | [physics.rs](../src/physics.rs), `Simulation::new`, `handles/names/contact_names`, `step_with_commands` | Inserção/remoção incremental coerente em corpos, colisores, caches e contatos; eventos históricos retêm identidade. |
| Triggers/seleção | [trigger.rs](../src/trigger.rs), `Runtime::new`, `Volume.targets`, `Event`; [engine.rs](../src/engine.rs), seleção | Listas compiladas por ID não viram filtros de template automaticamente. Seleção removida é invalidada, sem selecionar o substituto. |
| Render e consultas | [engine.rs](../src/engine.rs), `update_render_frame`; navegação/percepção e `projectiles::advance` percorrem Scene | Iterar uma visão coerente do mundo presente: estáticos + instâncias. Não depender do fallback para transform autorado de atores ausentes. |
| Edição/replay | [engine.rs](../src/engine.rs), `replace`, `Replay`, `replay_at` | Autoria continua separada de partida. Replay precisa registrar comandos de população, ordem, resultados, alocador e contextos retidos. |

## 1. Identidade e armazenamento

Recomendação: separar quatro conceitos.

- **TemplateId:** definição reutilizável de um NPC; dois guardas do mesmo template são indivíduos distintos.
- **InstanceId:** identidade imutável do indivíduo dentro de uma execução, gerada por sequência determinística sem wrap. Reset/load inicia outra execução. A API deve exigir escopo de execução para rejeitar comandos de uma partida anterior.
- **EntityHandle:** endereço interno validado por slot + geração. A geração muda ao reciclar armazenamento; não muda ao ressuscitar. Não expor o layout de memória como ABI pública.
- **LifeIndex:** contador já introduzido em EAF-58. Muda na ressurreição do mesmo indivíduo. Um spawn novo começa sua própria vida0 e não herda estado do cadáver anterior.

O ID público pode ser um token opaco contendo escopo/serial; evitar JSON number para inteiros maiores que2^53. O hot path usa handles numéricos, sem busca textual. Recibos guardam InstanceId e proveniência imutável suficiente para inspeção, sem exigir que a instância continue no pool. Não é necessário armazenar simultaneamente todos os identificadores em toda estrutura: referência de acesso usa handle/vida; registro histórico usa identidade/proveniência.

| Alternativa | Benefício | Custo/risco |
|---|---|---|
| Acrescentar/remover Entity e chamar replace | Reusa a edição | Reinicia o mundo, invalida ordem/slots; rejeitada como operação de runtime. |
| HashMap por ID para todo estado | Integração inicial direta | Hash/indireção e ordem explícita em cada iteração; não resolve sozinho referências históricas. |
| Pool com slots estáveis, geração e lista ativa | Acesso indexado, invalidação local, custo limitado | Migrar consumidores e validar geração; precisa separar ordem de processamento de localização física. Recomendada. |
| ECS genérico como pré-requisito | Infraestrutura ampla para composição | Mudança maior do que este contrato exige; não recomendada como condição para spawn. |

Primeira fase mantém o limite existente de9 atores simultâneos enquanto remove o acoplamento a índices autorados. Isso é uma etapa de compatibilidade, não limite arquitetural pretendido nem promessa de multidões. Tornar capacidade configurável exige revisar arrays, máscaras, pares de relações, reconhecimento e scratch transacional; aumentar MAX_ACTORS isoladamente não é suficiente.

Ordem proposta: IDs iniciais atribuídos pela ordem autorada já determinística; nascimentos recebem serial na ordem dos comandos aceitos. Processamento por InstanceId, não pela ordem de slots livres. Lista ativa limitada mantém essa ordem; liberação não deve trocar arbitrariamente a ordem dos sobreviventes. Geração/serial esgotados produzem erro, nunca wrap. A escolha de larguras/layout e capacidade definitiva será validada na implementação, sem alegação de performance por suposição.

## 2. Definições versus mundo presente

Propor catálogo de templates de NPCs separado das colocações iniciais. Template não é um corpo presente e não tem estado de vida/energia/cooldown em execução. Cena inicial continua criando os atores existentes com equivalência de comportamento; novo formato deve migrar explicitamente sem transformar cada corpo estático em um ator dinâmico.

Configuração compilada é compartilhada por referência imutável durante a execução. Cada instância tem transform/estado próprio. Floor, redes, rotas e facções são referências de domínio; alvo ator explícito é binding para uma instância específica. Um alvo removido não passa a significar “o próximo NPC desse template”. Reaquisição automática exige política configurada e percepção nova.

Primeiro spawn de NPC: template válido + posição explícita + bindings permitidos, sem overrides arbitrários de toda a estrutura. Inicializa vida/recursos/grants/rotina a partir do template, sem transferir saldo/cooldown/memória de outra instância. Jogador permanece único e fora do primeiro despawn público; reposição do jogador terá contrato próprio.

Consultas geométricas precisam consumir uma visão comum do mundo atual. A autoria não pode ser usada como fallback para atores removidos. O renderer recebe transform/aparência e identidade de render quando necessária; não recebe handles Rapier ou decisões de NPC. Novos materiais/malhas não devem usar slot de gameplay como identidade persistente de cache GPU. Coordenar essa extração com [a frente gráfica](graphics-handoff.md).

## 3. O que fazer com efeitos de uma origem removida

Exemplo que orienta a decisão: A lança um projétil; A é removido; B nasce no slot liberado. O projétil continua pertencendo a A e nunca consulta energia, buffs, facção ou grants de B. O mesmo vale para recibos e atribuição de dano.

| Política | Benefício | Tradeoff |
|---|---|---|
| Encerrar todas as entregas no despawn | Simples e libera tudo imediatamente | Muda gameplay: apagar autor apaga flechas/venenos já pagos. Pode ser opção futura explícita, não default implícito. |
| Manter o slot completo até toda referência sumir | Preserva acesso ao estado | Efeitos prolongados bloqueiam vagas; recibos/memórias poderiam reter população indefinidamente. Não recomendada. |
| Separar contexto de origem do ator presente | Libera corpo/slot, preserva autoria e entregas | Novo pool e regras para avaliações após remoção. Recomendada, com orçamento próprio. |

Proposta de semântica para o contexto separado:

1. Enquanto o autor existe, fatores continuam dinâmicos como hoje, inclusive após morte/ressurreição conforme EAF-51/58.
2. No despawn, capturar apenas dados necessários às entregas sobreviventes: identidade, facção/relações pertinentes, definição/grants pertinentes, bases e contribuições de atributos ainda relevantes com seus prazos originais. Referenciar catálogos imutáveis; não clonar Scene por projétil.
3. Depois da remoção, não aceitar mutações de gameplay no autor ausente. Contribuições capturadas continuam expirando pelo tick original. Portanto, não congelar apenas um multiplicador final: tags/componentes e expiração ainda afetam a avaliação. Se o autor já morreu, buffs removidos pela morte não entram no contexto.
4. Uma transação sobrevivente que precise **alterar** a origem ausente (resource/heal/buff no source) falha integralmente para aquele destinatário, sem reembolso de um lançamento já pago. Não aplicar metade da composição. Noise que dependa da posição atual do autor também falha; não inventar emissão na última posição. Ruídos já emitidos continuam válidos e anônimos.
5. Recibos, memória e rastros históricos guardam identidade/proveniência própria, sem manter o contexto de avaliação vivo. Liberar contexto quando nenhuma entrega/buff sobrevivente precisar dele. Se for usado refcount, contador limitado e determinístico no runtime, sem Arc/locks por conveniência.

Regra de gameplay da direção agora aprovada: após despawn, buffs existentes podem expirar, mas o autor ausente não recebe buffs novos. Uma alternativa é extinguir contribuições ao despawn e manter só bases; é mais simples, porém faz uma remoção de população alterar imediatamente o dano de efeitos em voo. Não confundir essa decisão com ressurreição, que já mantém o autor presente.

| Vínculo | Proposta ao remover |
|---|---|
| Cast/canal e intenção do autor | Cancelar sem novos pulsos/custos; pagamentos anteriores mantidos. |
| Pedido/cast direcionado ao removido | Falhar por instância indisponível na revalidação; nunca apontar ao substituto. |
| Periódico no destinatário removido; buff cujo owner foi removido | Encerrar com motivo target/owner_removed, distinto de morte. |
| Projétil lançado, periódico em outro destinatário, buff em outro owner | Preservar origem pelo contexto/identidade; aplicar regras de transação acima. Inclui dispellable por origem de periódico. |
| Área fixa | Pode sobreviver pelo contexto. `end_on_source_death` continua tratando morte; definir flag/política separada de remoção, sem interpretar despawn como morte silenciosamente. |
| Aura ancorada ao autor | Encerrar quando o corpo deixa de existir; não migrar para novo ocupante nem fixar no último ponto implicitamente. |
| Colisão nova de projétil/zona com B | Pode afetar B por contato/ocupação novos, com identidade/vida atuais. |
| Memória/relato de A | Manter pista congelada até regras sensoriais/budget expirarem; não renomear para B. |

Um contexto por origem removida compartilhado por entregas, com pool e payload limitados; capacidade distinta de atores presentes. Refinamento aprovado após o estudo: reservar capacidade quando a entrega é admitida, conforme os requisitos acima, evitando bloquear despawn/reposição por falta tardia de contexto. A alternativa original de falhar no despawn foi substituída. Não expulsar um contexto ainda usado. Limites numéricos serão escolhidos a partir dos pools atuais (64 projéteis,64 periódicos,64 buffs,16 zonas), evitando crescimento proporcional ao histórico de nascimentos. Referências só históricas não entram nessa contagem de retenção.

## 4. Comandos MCP e transação proposta

Nomes ilustrativos, ainda não publicados: `spawn_actor`, `despawn_actor`, `inspect_instances`, `inspect_population`. Toda configuração/execução/inspeção segue Command/Service comuns. `upsert/remove` continuam edição do documento; não serão aliases de spawn/despawn.

Propor comandos enfileirados, ordenados por sequência, consumidos no início do próximo tick antes do snapshot sensorial/físico. Pausado, o pedido fica pendente até `step`/`play`. Resposta imediata informa request_id, não um nascimento que ainda pode falhar. Recibo final informa outcome, tick, InstanceId e motivo. `inspect` deve distinguir fila aceita, operação aplicada e rejeição.

Nesta primeira API, spawn/despawn não são efeitos de habilidades; não introduzir transações aninhadas no executor. Pedidos no mesmo tick são sequenciais: despawn bem-sucedido pode liberar vaga para spawn seguinte; ordem inversa pode falhar por capacidade. Nova instância participa do snapshot posterior à aplicação da fila. Comando dirigido a um spawn depende do recibo de nascimento; não adivinhar o próximo ID.

Antes de mutar: validar template/bindings, orçamento de fila/histórico/população, capacidade dos subsistemas/contexto, transform finito, volume livre e condições físicas declaradas. Spawn verifica colisão e apoio conforme capacidade/política: caminhar exige apoio válido; posição aérea exige capacidade/configuração explícita. Não procurar uma posição alternativa nem conceder voo silenciosamente. Aproximação/pathfinding não é comprovada por caber no ponto.

Reservar/preparar dados por subsistema antes do commit. Nenhum evento, ID consumido, débito, corpo ou mapa parcial quando houver falha recuperável. Atomicidade prometida para erros de validação/capacidade; não prometer recuperação de OOM/process abort. Revalidar ocupação na ordem aplicada: dois spawns para o mesmo ponto não podem passar usando ambos uma consulta anterior à fila.

Despawn bem-sucedido remove corpo/collider e participação em decisões/sensores/áreas/contatos; libera navegação/passagem e seleção. Relações/buff owners/referências futuras usam geração validada. Triggers explicitamente vinculados ao removido emitem saída com motivo de remoção e não herdam o novo ator. Queries por classe/template de spawn ficam para um contrato de seleção posterior; não ampliá-las implicitamente.

Despawn oculto não deve zerar a memória dos observadores nem mudar seus budgets de investigação. Validade interna para aplicar uma ação e evidência disponível para decidir são consultas diferentes. Pistas antigas de A não confirmam B mesmo que apareça na mesma posição com a mesma aparência.

## 5. Replay, autoria e limites

Scene salva templates/colocações/regras autoradas. Snapshot inspeciona instâncias correntes. Replay registra fila com tick/sequence, inclusive pendentes e operações falhas, e compara população, alocação, contextos de origem, estados e recibos além do mundo físico. Savegame continua separado.

Escopo externo de execução muda no reset para rejeitar tokens antigos; reprodução interna usa identidades lógicas determinísticas e não compara um UUID de transporte aleatório. Migração de cena e versão de replay terão incremento coordenado com a frente gráfica. Replays atuais exigem mesmo build; não prometer replay cross-version.

Geradores por onda/tempo/região virão depois de spawn explícito seguro. Deverão usar sementes/versionamento, candidatos/retentativas limitadas, mesma transação e eventos explicáveis. Primeiro recorte não entrega loot, respawn automático, persistência de campanha, streaming ou milhares de NPCs.

## 6. Sequência incremental recomendada

1. **Referências seguras sem população dinâmica:** tipos de identidade/handle/vida, resolver central, migração dos vínculos e recibos acima, teste de stale handle/reutilização em pool isolado; cenário legado equivalente e inspeção MCP. Nenhum novo comando de spawn ainda.
2. **Mundo presente separado da autoria:** extração das consultas/RenderFrame e adapters de instâncias, mantendo atores iniciais e resultados. Coordenar interface com a frente gráfica; não duplicar uma Scene mutável como se fosse runtime autoritativo.
3. **Spawn/despawn explícitos:** catálogo/templates, fila transacional, física incremental, origem retida e limpeza, novo cenário MCP com reutilização e replay. Completar captura/seleção/undo/reset sem resetar sobreviventes.
4. **Gerador mínimo procedural:** somente após validar o ciclo explícito; semente e ponto/candidatos limitados com ocupação, cooldown e contagem, separado da reposição de jogador.

Cada fase deve ser um incremento revisável, com checklist, testes e commit próprio. A primeira migração já toca muitos módulos: não é uma otimização pequena de variáveis nem uma troca trivial de tipo. Não há dependência nova/ECS aprovada nesta proposta.

## Matriz de validação para implementação

| Caso | Resultado obrigatório |
|---|---|
| A removido, B ocupa mesmo slot | Handle antigo falha; recibos/memória/ruído/projétil seguem identificando A. |
| A ressuscita | Mesmo InstanceId/geração, LifeIndex incrementa; preserva energia/cooldown conforme EAF-58. |
| Slot e contador esgotados | Erro explícito, sem wrap, mutação parcial ou identidade duplicada. |
| Spawn entre atores em combate | Vida, cooldowns, cast, posição e RNG dos sobreviventes preservados. |
| Dois spawns concorrentes no mesmo ponto ou última vaga | Ordem definida, um sucesso quando só um cabe; nenhum corpo parcial. |
| Despawn com contexto cheio | Usa reserva obtida na admissão da entrega; não bloqueia retirada por falta tardia de contexto. Novas entregas sem reserva falham antes da cobrança. |
| Origem com fire/ice/buff que expira após despawn | Parcelas usam origem A e prazo original; bases de B não afetam o dano. |
| Efeito target + source removido ou noise dependente do source | Falha transacional, sem efeito parcial nem reembolso do lançamento pago. |
| Aura source versus área fixa; morte versus despawn | Encerramento conforme vínculo/política explícita, eventos distintos. |
| Despawn atrás da parede e relato velho | Crença/budget não recebem informação global; novo NPC exige observação própria. |
| Buff/periódico de A em C enquanto B recicla slot | Refresh/dispel/proteção/autoria não confundem A e B. |
| NPC voador/removido em passagem/trigger/contato | Sem reserva, motor, collider ou membro fantasma; evento de saída com identidade antiga. |
| Seleção/captura após remoção, spawn e crescimento de buffers | Nenhum desenho fantasma/fallback autorado; cache invalida e demais objetos permanecem. |
| Fila pendente, falha, reset/load e replay completo/prefixo | Mesmas decisões/IDs lógicos/recibos/estado e pixels dentro do mesmo build; token da execução anterior rejeitado. |
| Repetidas gerações acima da capacidade simultânea | Memória limitada por pools ativos e anéis, sem acumular contextos ou builds. |

## Revisão estática deste estudo

Não houve mudança de runtime. Inventário lido diretamente no código atual; links e símbolos revisados. Sem nova compilação, profiling ou alegação de FPS. O checklist de implementação deverá responder: frequência de resolução, alocações esparsas de spawn, tamanho real dos handles/records, retenção por origem, custo dos scans, invalidação de índices/cache, ordem/locks, limites de históricos e equivalência/replay. Resultados anteriores573 Rust/218 grupos MCP descrevem EAF-58, não validam esta proposta.

Decisões para alinhamento: pool com geração sem introduzir ECS; contexto separado com contribuições expirando e rejeição de efeitos no autor ausente; fase de população antes dos sensores; primeiros spawns apenas NPCs com capacidade atual preservada. Após alinhamento, começar pela fase1, não implementar silenciosamente todas as quatro fases.
