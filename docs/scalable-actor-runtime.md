# Runtime escalável de atores — EAF-131

## Contrato antes da implementação

Meta permanente: 1.000 jogadores na mesma área e combate. Single player offline
usa o mesmo núcleo e não exige rede, autenticação ou processo de servidor separado.
Este incremento remove limites de representação antes de implementar transporte.
Não confundir admissão de 1.000 atores com taxa de tick, cliente gráfico ou rede
validados. UI/modelagem EAF-130 liberou fontes comuns após seus commits6e19bae/49b369f.

## Alternativas e decisão

Apenas ampliar MAX_ACTORS manteria arrays/cópias mundiais, índices truncados e
shifts inválidos. Adotar um ECS agora acrescentaria migração e dependência sem
resolver ordenação, conhecimento e atomicidade. Escolha: componentes densos
alocados pelo número de slots ativos/reservados, índice de ator de 16 bits e
referências de instância/geração já existentes. Índice não é identidade nem vida.

Capacidade inicial explícita: até1.000 Players e até8NPCs autorais/reservados,
com teto interno de1.024 slots; entidades gerais até4.096. Pools de habilidades,
origens e budgets de comandos têm contratos separados e não crescem por engano.
Autoria legada conserva seus limites por versão; Scene87/replay91 coordenados
após86/90. Formato maior não remove guardas históricas de outros campos.

Conjuntos de atores usam bitset limitado a1.024 posições (16 palavras de64bits),
com operação por slot e sem shifts além da palavra. A alternativa de Vec por
máscara alocaria/copiaria no passo; listas esparsas precisariam de buscas para
consultas frequentes. Bitset Copy de128bytes preserva passagens baratas de
snapshots, enquanto os componentes de cada ator deixam de reservar o mundo todo.
Esse teto é deliberado e validado, não nova identidade pública/ABI de plugins.

Ordenação de atores por InstanceId deve ser cacheada no catálogo e atualizada
somente em mudança de presença/identidade. Controles continuam por handle escopado;
callbacks de morte/retorno, memórias e entregas não podem adotar ocupante novo.
Prontidão de grants por ator permanece máscara16bits: não é conjunto de atores.
NPCs mantêm conhecimento limitado; Players não ganham sensores de NPC implícitos.

Plano local de EAF-123 e índice dono/definição de buffs são preservados. Área é
sequência determinística de transações por destinatário, sem rollback coletivo.
Representação de candidatos/recibos dimensionada pelos atores existentes, sem
arrays mundiais no stack. Limites autorais de alvos permanecem regra explícita,
não atalho para remover jogadores relevantes do combate.

## Checklist estático prévio

1. Frequência: componentes/índices na construção/edição; buffers reutilizados por
   tick e ordem recomposta só em lifecycle. Nenhum mapa/string novo por ator/tick.
2. Cópias: Vecs densos por slot; slices para leitura; bitset limitado Copy. Evitar
   reconstruir buffers/snapshots globais por transação ou consulta de componente.
3. Tipos: slot16bits, entidade16bits com limite4.096; geração/vida/InstanceId64bits
   preservados. Separar slots de atores, recursos, grants, volumes e rede de navegação.
4. Repetição: catálogo inverso direto, ordem cacheada e scratch de movimento/readiness
   reutilizados. Não classificar mil atores em cada commit local.
5. Escala: buffers pelo número real/reservado; NPCs e efeitos com budgets próprios.
   Áreas densas continuam trabalho real por destinatário; sem esconder participantes.
6. GPU/I/O: sem mudança estrutural no renderer; inspeção grande e JSON são comandos
   explícitos. Render de1.000 é gate próprio, não inferência de teste headless.
7. Sincronização: mesma autoridade/fase sequencial, nenhum paralelismo/lock novo.
   Fontes/builds compartilhados somente após liberação da frente UI.
8. Invalidação: reset/undo/load recompõem; reuso de slot limpa componentes, mantém
   referências históricas; índices/caches não sobrevivem indevidamente à revisão.
9. Validação: reutilizar regressões afetadas. Criar apenas teste crítico de fronteiras
   de bitset e cenário denso de identidade/posse/combate/replay/offline: overflow,
   corrupção, herança de slot e atomicidade são riscos materiais desta mudança.
10. Custos mantidos: física/oclusão e alvos densos, callbacks/limpeza limitados,
    serialização de inspeção/replay. Medir workload antes de declarar capacidade
    sustentada; não executar toda a suíte por rotina ou testar detalhes triviais.

## Continuidade do objetivo

Após storage/handles/conjuntos: execução contínua separada de gravação, admissão e
orçamentos de sessão, visão replicada/transporte, desconexão/reconexão e cliente.
Antes de chamar o sistema pronto, verificar workload com1.000 clientes e combate
compartilhado, latência/perdas/ordenação, tempo de tick/memória e single player
sem rede. Decisões de transporte e workload serão registradas na própria etapa.

## Evidência do incremento

Storage implementado com componentes densos, buffers reutilizados, ordem cacheada e
conjuntos de atores em palavras. Controles admitem até4096eventos por tick, com
reserva de liberações; a gravação ainda tem4096eventos totais neste incremento.
O teste crítico único de integração exercita1.000Players e8slotsNPC reservados,
controle individual, área com1.000destinatários, morte/retorno64/256/999,
reuso generacional acima1000 e replay. Evidência final de execução/versão em
validation.md; não equivale a1.000conexões ou combate sustentado.

### Revisão após implementação

1. Componentes criados na inicialização; `Catalog::rebuild_order` só em lifecycle.
2. `FrameScratch` e prontidão da física são reutilizados; `Writes16/Changes2`
   continuam locais. Recibos de área possuem Vec dimensionado pelos atores.
3. Ator u16, entidade u16 e identidades/vidas u64; grants/objectives/buffs/nós
   mantêm seus domínios próprios. Scene87/replay91, dependências inalteradas.
4. Corrigidos shifts/truncamentos e cópias globais de vida/movimento por passo;
   NPCs mantêm sensores e decisões só sobre evidências existentes.
5. Até1000Players/8NPCs,1024slots e4096entidades; pools de efeitos/origens e
   gravação continuam limitados e não recebem alegação de escala implicitamente.
6. Renderer, buffersGPU e I/O por frame preservados; JSON sob comando.
7. Autoridade sequencial e mesmos pontos de commit; nenhum lock/worker/rede novo.
8. Reset/load/edição recompõem componentes; remoção/nascimento/reforço de vida
   limpam estados por slot sem adotar identidades anteriores.
9. Dois testes novos protegem corrupção por fronteira de bits/slot, posse,
   lifecycle, área e replay; demais alterações de fixtures apenas adaptam a
   cobertura existente aos tipos e contratos novos.
10. Custos mantidos: física/oclusão, varredura real de candidatos e recibos
    densos; inspeção de relações ainda expande pares sob pedido explícito.
    Sem medição de FPS nem capacidade sustentada inferida deste incremento.
