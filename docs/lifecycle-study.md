# Ciclo de vida: identidade, ressurreição e respawn

O recorte de ressurreição do mesmo indivíduo foi aprovado por Guilherme em25/10/2026 e implementado em EAF-58; contrato vigente em [resurrection.md](resurrection.md). As alternativas de spawn/despawn abaixo continuam propostas. O restante deste documento preserva o estudo anterior à implementação.

EAF-57, estudo sobre o runtime de `d590930`, em 25/10/2026. Documento de proposta para discussão técnica; nenhuma capacidade de ressurreição, spawn ou savegame foi acrescentada. Cena/replay continuam na versão 45. A implementação deverá passar pelo checklist de performance e pelos testes abaixo quando o contrato for alinhado.

## Problema concreto

Hoje morte é terminal até o reset da cena. Dar vida positiva ao cadáver não basta: percepção, motores, reservas e ações também têm estado. Além disso, um índice de ator identifica o mesmo objeto por toda a sessão. Reutilizá-lo para outro indivíduo faria referências antigas apontarem para o novo ocupante.

A recomendação é separar três operações:

- **Ressurreição:** o mesmo indivíduo volta à vida; identidade, autoria e atribuição de ações permanecem.
- **Respawn de população:** um ponto ou gerador cria uma nova instância. O NPC anterior e seus efeitos podem ainda existir; compartilhar template não significa compartilhar identidade.
- **Reposição do jogador:** regra do jogo que pode combinar ressurreição, deslocamento e restauração de recursos. Não deve ser deduzida da palavra respawn nem imposta a todos os NPCs.

`reset` continua sendo uma operação de laboratório que reconstrói a simulação inteira a partir do documento. Não serve como primitiva de nenhum desses processos.

## Inventário verificado

| Área | Código atual | Consequência para o contrato |
|---|---|---|
| Identidade compilada | [targeting.rs](../src/targeting.rs), `Catalog::new`, `slot`, `MAX_ACTORS` | Até nove atores, ordenados pelo ID autorado. Slots `u8` válidos durante a simulação; inserir/reordenar atores não é uma operação local segura. |
| Vitalidade | [vitality.rs](../src/vitality.rs), `Runtime::dead`, `commit`, `State` | Vida zero significa morto. Anel de 256 mortes; não há contador de vida nem evento de retorno. |
| Física | [physics.rs](../src/physics.rs), `Simulation::step_with_commands`, `deactivated_actors` | Desativação ocorre uma vez por slot; corpo permanece com gravidade. Retorno exige rearmar essa transição e motores, não reconstruir o mundo. |
| Conhecimento | [perception.rs](../src/perception.rs), `NpcState::die`, `recognize_death`, `observe_support`, `select_observation` | `known_dead` é uma máscara de slots. Candidatos conhecidos mortos são excluídos antes da observação. Sem mudança, um ressuscitado pode nunca ser redescoberto. |
| Rádio | [communication.rs](../src/communication.rs), `Report`, `collect` | Observação congelada inclui índice de ator e tick. Relato antigo precisa continuar sendo evidência antiga, sem acesso à vida atual. |
| Execuções | [abilities.rs](../src/abilities.rs), `Cast`, `State`, `apply_effects` | Alvo opcional por slot, recursos/cooldowns por ator, morte cancela casts. Não há distinção de vida no pedido pendente. |
| Entregas persistentes | [projectiles.rs](../src/projectiles.rs), `Shot`; [periodic.rs](../src/periodic.rs), `Instance`; [persistent_area.rs](../src/persistent_area.rs), `Instance::end` | Origem/cast sobrevivem por vários ticks. Projétil lançado não é recolhido pela morte; área pode sobreviver à origem conforme configuração. |
| Limpeza | [abilities.rs](../src/abilities.rs), `remove_dead_periodic`, `clean_buffs`, `clean_areas` | Filtros consultam morte atual. Morrer e ressuscitar entre limpezas exige registrar a transição, para não deixar efeitos que deveriam ter terminado. |
| Edição e replay | [engine.rs](../src/engine.rs), `replace`, `reset`, `replay`, `replay_at` | Edição recompila; replay registra entradas e estados esperados e exige mesmo build. Não existe inserção dinâmica nem persistência de campanha. |

Este inventário é análise de código. Não demonstra uma ressurreição em execução.

## Alternativas de identidade

| Alternativa | Benefício | Custo ou falha |
|---|---|---|
| Reutilizar o slot e restaurar vida | Menor mudança imediata | Casts, relatos, autoria de dano e memória confundem indivíduos/vidas; risco de referência antiga válida por coincidência. Não recomendada. |
| Novo ID textual em cada retorno | Referências externas distinguíveis | Confunde ressurreição com novo indivíduo; recompilar catálogo reorganiza slots e reinicia estado. Não resolve runtime por si só. |
| Handle de instância + contador de vida | Separa objeto, reutilização de armazenamento e retorno à vida | Mais metadados em referências e replay; cada subsistema precisa declarar a qual identidade se vincula. Recomendada como direção. |

Modelo conceitual proposto, ainda sem escolher layout/ABI:

- ID autorado identifica a entidade/template no documento.
- Handle de instância identifica um indivíduo no runtime. Na futura população dinâmica, um slot reutilizado terá outra geração; geração nunca dá wrap silencioso.
- Contador de vida muda no retorno do mesmo indivíduo. Intenções sobre a vida anterior não se tornam válidas automaticamente.
- ID de execução permanece único na sessão e associa recibos/efeitos à autoria original.

Não é necessário introduzir um ECS nem substituir todo armazenamento para provar ressurreição. Primeiro recorte pode manter os nove atores existentes, adicionar contador de vida e **proibir reutilização de slot**. O handle com geração deve preceder spawn/despawn dinâmicos; essa etapa não pode apenas ampliar arrays e reutilizar índices.

## Primeiro recorte recomendado: ressuscitar no próprio corpo

Proposta de efeito explícito de habilidade, direto e dirigido a um ator morto observado, com quantidade positiva de vida nas unidades já existentes. Cura normal continua incapaz de ressuscitar. Preparação, custo, cooldown, cancelamento e inspeção usam o executor compartilhado; autoria e acionamento chegam pelo MCP antes de um atalho visual.

O primeiro incremento não inclui área, canal ou projétil de ressurreição. A seleção de mortos precisa de contrato próprio: não afrouxar globalmente os filtros de vivos para fazer essa habilidade passar. Começar com alvo explícito e evidência visual confirmada; triagem automática de cadáveres é uma extensão posterior. O curador morto não pode executar a própria ressurreição nessa modalidade; reposição externa do jogador será outra operação.

Recomendações que precisam de alinhamento antes do schema:

| Estado | Comportamento inicial proposto | Tradeoff |
|---|---|---|
| Identidade/posição/corpo | Mesmo indivíduo, no corpo atual, contador de vida incrementado | Evita decidir teleporte, ponto de spawn e busca de espaço no mesmo incremento. Um corpo caindo não é teleportado ao chão. |
| Vida | Quantidade explícita, positiva, limitada ao máximo | Reutiliza unidades inteiras; porcentagem/escala por definição pode vir depois. |
| Recursos e cooldowns | Preservar saldo e prazos correntes | Ressurreição não vira restauração de energia/cooldowns. Um jogo poderá compor políticas específicas depois. |
| Casts/canais cancelados | Continuam cancelados | Impede execução atrasada da vida anterior; novo pedido inicia outra execução. |
| Buffs/periódicos removidos na morte | Continuam removidos | Retornar não reinstala efeitos antigos. A transição letal precisa ser consumida mesmo com retorno no mesmo tick. |
| Locomoção/objetivos | Rearmar módulos; começar sem caminho/reserva/compromisso antigos | Replanejamento explícito, sem fingir chegada nem recuperar posse de passagem. Retomada detalhada de rotina fica configurável em etapa posterior. |
| Memória própria | Reiniciar estado transitório de decisão/percepção | Recorte simples; amnésia seletiva/memória social persistente não pode ficar acoplada a esse reset transitório. |
| Conhecimento de terceiros | Só nova evidência muda a crença de morto para vivo | Nenhum broadcast global de ressurreição ou localização oculta. |

Transação proposta: validar alvo morto/evidência/vida/custos e capacidade de registrar eventos; preparar plano; revalidar no commit; aplicar vida e efeitos internos de reativação de forma indivisível antes da física. Dois curadores no mesmo tick seguem a ordem existente: o primeiro pode ressuscitar; o segundo falha ao revalidar sem pagar. Morte e retorno precisam constar em eventos distintos, mesmo que o estado final do tick seja vivo.

Se o recorte passar a reposicionar, haverá uma decisão adicional obrigatória: validar volume livre e apoio/capacidade no destino **antes** de pagar. Destino ocupado deve falhar ou aguardar por política explícita, nunca empurrar corpos ou procurar coordenadas ocultamente sem contrato. Isso pertence ao deslocamento/spawn posterior, não à ressurreição in-place.

## Memória e fronteira entre vidas

Não basta limpar `known_dead` de todos os NPCs. Isso revelaria uma mudança que pode ter ocorrido atrás de uma parede. A recomendação é permitir a sondagem visual de corpos conhecidos mortos e atualizar a crença somente após confirmação sensorial; o conhecimento de morte deve identificar a vida observada, não só o slot.

Uma referência interna pode ser rejeitada como obsoleta sem atualizar memória/posição do observador. Para evitar onisciência, rejeição interna não deve antecipar a desistência de uma investigação oculta: o NPC mantém a pista congelada e seu orçamento até obter evidência ou expirar. Novo contador de vida não é um novo sentido.

Relatos e intenções carregam a vida observada. Rádio velho não autoriza cast na vida nova; confirmação visual nova pode superar a crença antiga. Sons permanecem anônimos. Inspeção MCP pode mostrar o estado global, separada do conhecimento disponível à decisão.

## Efeitos antigos: vínculos diferentes

Não usar uma única regra de invalidação para todos os efeitos:

- **Cast direcionado/pulso já vinculado:** não deve reaparecer na vida nova por coincidência de slot. Capturar a vida do destinatário e revalidar. Ressurreição captura a vida morta que pretende encerrar.
- **Projétil físico já lançado:** continua no mundo; pode atingir um ressuscitado por uma colisão nova, como qualquer corpo elegível. A colisão não é rastreamento por referência antiga. Projétil que já terminou não retorna.
- **Zona persistente:** seleciona ocupantes atuais em cada pulso. Pode atingir o ressuscitado se ainda existir; política de encerramento pela morte da origem precisa observar a transição, não apenas vida final.
- **Aura sobrevivente ancorada à origem:** se o indivíduo é o mesmo, pode acompanhar seu corpo conforme a configuração existente. Novo spawn jamais herda essa âncora automaticamente.
- **Autoria de dano:** preservar autor e execução originais. Hoje fatores são dinâmicos no impacto/pulso. Para ressurreição do mesmo indivíduo, a recomendação é preservar essa regra: buffs novos podem afetar seus disparos antigos ainda em voo. Congelar fatores no lançamento seria outro contrato, com impacto em EAF-51.

Antes de remover uma instância de população com efeitos em voo, será preciso escolher entre manter um registro limitado da origem até os efeitos terminarem, capturar um contexto de origem ou encerrar entregas por política. Não avaliar atributos do novo ocupante do slot. Essa escolha permanece aberta para spawn/despawn; o primeiro recorte mantém a origem existente e evita essa migração.

## Sequência depois da decisão

1. Alinhar identidade e regras da tabela; implementar ressurreição explícita no executor, com contador de vida, reativação e novas evidências. Cena de um curador, cadáver visível e observador oculto; mesma operação pelo MCP e jogador.
2. Introduzir instâncias estáveis e autoria de spawn, com template separado de indivíduo, geração e limite de população. Definir retenção de cadáver/origem antes de reciclar slots.
3. Geradores com semente, posições candidatas, capacidade física, prazo/retentativa limitados e eventos de nascimento/despawn. Gerar posições não prova que o corpo cabe nem que existe caminho.
4. Reposição do jogador, persistência de população e integrações de quests/loot em contratos próprios. Replay continua diferente de savegame.

Essa sequência mantém NPCs como eixo e não antecipa integração de rampas/controlador nem promete fechar NPC v1.

## Matriz de aceitação para a futura implementação

| Caso | Evidência exigida |
|---|---|
| Morto visível, vivo, morto oculto, ausência de vitalidade | Sucesso somente no caso elegível; sem cobrança indevida nem informação oculta no comportamento. |
| Dois curadores, custo insuficiente, cancelamento no prazo | Uma ressurreição; demais planos falham atomicamente; custos/cooldowns corretos. |
| Morte e retorno no mesmo tick, em ordens distintas | Eventos sequenciais, limpeza de efeitos e motores consistente, sem cast antigo ressurgir. |
| Morte repetida após retorno | Nova desativação, novo evento, gravidade/passageiros/reservas e cancelamentos corretos. |
| Observador conhecia morto; retorno oculto | Mesmas decisões sob mesmas evidências; redescoberta apenas por nova visão confirmada. |
| Relato antigo, cast antigo e nova observação | Referências de vida distintas, TTL original; sem atualização de crença por leitura global. |
| Projétil/zona/aura sobreviventes | Colisão/ocupação atuais; autoria original e política de morte preservadas, inclusive no mesmo tick. |
| Recursos, buffs, cooldowns e memória | Política da tabela explícita e inspecionável, sem restauração implícita. |
| Físico XYZ, cadáver caindo e rota bloqueada | Retorno sem capacidade nova, chegada falsa ou teleporte; replanejamento pelos motores existentes. |
| MCP, autoria inválida, undo/reset, replay/prefixo e render | Dispatcher comum, erros atômicos, formato versionado e snapshots/pixels equivalentes headless. |
| Futuro slot reciclado | Toda referência antiga rejeitada; autoria nunca migra para nova instância. Gate obrigatório antes de spawn dinâmico. |

Testes futuros: Rust release/all-targets, Clippy, cenário MCP próprio e regressão completa incluindo vitalidade, triagem, dissipação, atributos, dano, áreas, projéteis, percepção, rádio, mobilidade e replay/render. Nesta entrega documental, a validação é revisão das referências e do diff; os 557 testes/52 cenários anteriores não comprovam o contrato proposto.
