# Transações locais e buffs indexados — EAF-123

## Contrato e decisão antes do código

Single player offline e multiplayer são requisitos permanentes. A simulação local não
exige internet, autenticação remota nem processo separado de servidor. As mesmas
regras Rust de física, habilidades, vida e população servem aos dois modos.
Preparação para 1.000 jogadores no mesmo combate não pode regredir esse contrato.

Este incremento parte de e473e5a em multiplayer-runtime. Não amplia o teto de 9 de
atores nem implementa rede. Migra armazenamento de contribuições e scratch de
transações antes de ampliar índices/máscaras/componentes do catálogo.

Alternativas: copiar o mundo facilita rollback mas custa O(atores×recursos) por
entrega; clonar mapas por operação mantém esse custo/alocação; um ECS novo altera
muitos contratos sem resolver atomicidade. Escolha: write set limitado pelos
campos tocados, sem alocação por transação. Até 8 custos + 8 operadores de recurso e
até 2 destinatários de vida (origem/alvo). Origem=alvo reutiliza o mesmo registro.
Uma área continua sendo uma sequência de transações por destinatário, sem novo
rollback coletivo; cada transação observa os commits anteriores. Atributos são
avaliados no estado anterior ao commit, inclusive buff/dispel na mesma operação.
Commit de vida ordena somente as mudanças por InstanceId e cancela casts dos
atores mortos tocados, preservando recibos/life_index e sem scan de todos os atores.

Buffs: manter vetor denso em ordem de aplicação e acrescentar índice secundário
ordenado por (dono, definição, posição). Consultas de atributo usam busca binária
por dono/definição, com filtros de geração, validade e contexto mantidos. Um mapa
por ator/atributo traria mais alocações e duplicaria referências por regra; listas
encadeadas complicariam compactação/ordem. O índice de posições usa buffers
reutilizados: reconstrução somente quando houver mudança na coleção, custo
O(B log B); leitura O(regras×log B + contribuições pertinentes). Não promete
resolver todo custo de instalação/remoção/expiração. Mutação encapsulada impede
índice obsoleto; desserialização reconstrói índice, que não é dado autoral nem
campo adicional do replay. Ordem por regra/prioridade e aplicação continua igual.

Cena 77/replay 81 e comandos existentes preservados: wire format de buffs continua
lista e scratch não é serializado. Replay já exige build id idêntico. Sem mudança
em RenderFrame/renderer, dependências, threads, locks, física ou conhecimento NPC.

## Checklist estático antes de implementar

1. Frequência: planejamento/preflight/commit/impacto/pulso existentes; índice só
   muda em instalação/refresh/remoção. Nenhum polling novo.
2. Cópias: remover arrays mundiais do Plan; write sets stack limitados pelos
   efeitos/custos; índice reserva capacidade, sem heap na consulta.
3. Tipos: slots internos existentes, referências geracionais continuam validadas;
   posições do índice usize e ponteiros emprestados, sem unsafe.
4. Repetição: reutilizar write set para custo+efeitos e self; consultas restringem
   contribuições antes da composição. Ordenar somente mudanças/índice alterado.
5. Escala: scratch independente de MAX_ACTORS; buffs ainda pool de 64, instalação e
   limpeza globais explícitas. Não aumentar constantes para alegar escala.
6. GPU/I/O: nada novo; inspeção, captura e persistência pelos comandos existentes.
7. Sincronização: mesma fase serial e fronteira de commit; sem paralelismo novo.
8. Invalidação: refresh/compactação/clear/deserialização recompõem índice; reset,
   load, undo/redo recompõem runtime. Geração/vida filtradas como antes.
9. Contrato: testes de rollback tardio, self, custos+efeitos repetidos, lifecycle,
   ordem, índices após mutações/serde, contribuições alheias, legado e replay.
   Cargo release/all-targets, Clippy e regressão MCP completa headless.
10. Custos mantidos: arrays/máscaras globais, pools, varreduras de sensores/limpeza,
    relógio/log do laboratório. Próximos incrementos; sem FPS/capacidade de 1.000 alegados.

## Validação e revisão final

Gates finais aprovados em29/10/2026, macOS Apple Silicon, fonte/binário
75693edd70fe6348: **769 testes Rust/63alvos**, zero falhas/ignorados;
Clippy release/all-targets `-D warnings`, fmt, build e diff aprovados.
**68cenários MCP/280grupos**, completed/passed/native_window:false, todos os
resumos com o build final. Cenário local_transactions:3grupos e
network_denied:true. Rust novo:4unidades e5integrações. Nenhuma janela aberta.

Logs correntes em artifacts/transactions-{rust-tests,clippy,fmt,build,mcp}.log;
auditoria artifacts/transactions-build-audit.json, relatório completo
artifacts/mcp-validation/report.json e artefatos local-transactions-* do cenário.
Um release corrente11.435.552bytes em
/Volumes/Zero/dev/games/.ignis-build/multiplayer-runtime, target symlink ignorado;
cache1,0GiB/artifacts45MiB no checkpoint, sem instalação/cópias históricas.
Cena77/replay81 e dependências preservados. Aviso futuro de block0.1.6 já existente;
Clippy passa sem warnings próprios. Não há rede multiplayer/escala1000 ou
aceite nativo/humano inferidos.

 Os testes incluem write sets isolados com ledgers de
1, 9, 1.000 e 4.096 entradas, lendo/escrevendo somente o campo tocado; isso valida
o mecanismo local, não uma simulação com essa população. Oráculo linear compara
o índice após 256 ciclos de instalação/refresh/compactação/serde, incluindo
referência obsoleta, expiração, capacidade e ponteiros dos buffers reutilizados.
Integrações cobrem16 writes (8 custos esparsos + 8 efeitos), self/repetição/rollback,
morte dupla ordenada, cast de terceiro intacto, replay/prefixo e paridade entre
input legado local e controle explícito.

O cenário MCP local_transactions verifica o executável real por stdio. No macOS,
usa sandbox-exec com `(deny network*)`, confirmando EPERM em bind e connect antes
de executar movimento/combate/buffs/save/replay; a rede do Mac não é alterada.
Esse gate valida o núcleo headless offline, sem inferir teclado/apresentação nativa
ou aceite humano. Em outras plataformas o cenário registra network_denied:false
quando não houver esse mecanismo; não declara isolamento validado.

Revisão final fechada sobre o mesmo fonte/binário dos gates. Custos
mantidos: write set faz busca linear em no máximo 16 registros; bases e regras
compiladas continuam pequenas/limitadas. Índice acrescenta até 64 posições usize
por estado e é reconstruído O(B log B) nas mutações; instalação/refresh, limpeza,
captura de origem destacada e consultas de dispel ainda têm scans limitados.
Buffers fixos de recursos/vida/casts por MAX_ACTORS, máscaras u16, slots u8,
capacidade de entidades/efeitos, admissão e relógio/gravação ainda precisam migrar.
Não há alegação de FPS, capacidade de 1.000, rede ou cliente gráfico pronto.

## Revisão final de performance

| Pergunta | Resposta e evidência |
|---|---|
| 1. Frequência | Corrigido: plan_effects mantém as fases existentes; vitality::commit ordena e visita somente Changes. Índice é reparado em mutações, sem polling novo. |
| 2. Alocações/cópias | Corrigido: transaction::Writes usa arrays locais 16/2 e leitura lazy por chave; Plan não copia saldos/vida globais. Contributions reserva dois Vecs na construção; teste confirma ponteiros estáveis durante256 ciclos. |
| 3. Tipos/acesso | Adequado: chave de recurso (ator usize, recurso usize), Changes por ator e índice secundário usize. Sem unsafe/precisão nova; referências/vida continuam nos validadores existentes. |
| 4. Repetição | Corrigido: get_or_insert reutiliza valor planejado para custo/self/operadores repetidos; resolve usa for_buff por dono/definição e preserva fast path sem buffs. |
| 5. Escala | Corrigido para scratch: independente de MAX_ACTORS, limitado por 8 custos + 8 efeitos/2 atores. Mantido com motivo: índice O(B log B) em mutações e regras/consultas locais; pools e estado global ainda têm limites transitórios. |
| 6. GPU/I/O | Não se aplica ao tick alterado: renderer/RenderFrame/shaders e persistência sob demanda iguais. Cenário offline bloqueia somente rede do processo testado. |
| 7. Sincronização | Adequado: mesmo commit serial/fases, sem thread/lock/dependência nova; área mantém atomicidade por destinatário e visibilidade de commits anteriores. |
| 8. Invalidação | Corrigido: Contributions expõe apenas leitura; refresh/compactação/clear/serde reconstroem índice. JSON continua lista; reset/undo/load recompõem o runtime e não persistem índice. |
| 9. Contrato | Gates finais e evidências registrados abaixo; oráculo linear, rollback, self, morte dupla/cancelamento, origens, buffs e replay/prefixo/offline cobertos. Não inferir entrada nativa/aceite humano. |
| 10. Custos mantidos | Busca no write set até 16 registros, índice adicional até 64 usize, scans de admissão/limpeza/retirada e componentes globais permanecem explícitos. Próximo passo é storage/handles/máscaras; sem FPS/escala de 1.000 alegados. |

## Continuidade

Próxima unidade: storage de componentes por ator com capacidade explícita e
buffers reutilizados; substituir slots u8/máscaras u16 nos consumidores preservando
InstanceId/geração/vida e a ordem atual. Dimensionar jogadores/NPCs/efeitos
separadamente e validar escalas crescentes no núcleo local antes de transporte.
Servidor multiplayer e jogo single player embarcado usam esse mesmo núcleo;
nenhuma exigência de conexão pode entrar no caminho offline.

Mapa para a próxima migração (símbolos da branch deste incremento):

| Fronteira | Trabalho pendente |
|---|---|
| targeting::Catalog | actors Vec<u16> já é denso, mas slots inversos u8, factions[MAX_ACTORS], references[MAX_ENTITIES] e order() fixo ainda limitam o catálogo. Separar índice local de identidade e capacidade autoral/runtime. |
| vitality::State | actors/lives fixos e died/revived/dead masks u16. Write sets locais já removem a cópia/commit global; trocar storage e consumidores das máscaras em conjunto. |
| abilities::State/Runtime | saldos, cooldowns, casts, motion, grants/capacity/actor_instances ainda por MAX_ACTORS; máscaras e destinatários atravessam várias fases. Não mudar somente o limite. |
| perception e physics | observações/conhecimento e MotionSample/ready/dead passam por arrays/máscaras; manter conhecimento limitado e ordem dos sensores/física ao migrar. |
| trigger::Runtime | targets usa índices de entidade u16, enquanto occupancy u32 e Event.target u8 são posições na lista explícita de até 32 targets do volume. Não confundir esse orçamento autoral com slots globais de atores. |
| controls::Runtime | mapa por handle e frame reutilizado já existem; adaptar frame à capacidade de atores sem trocar referências externas nem reinstalar posse obsoleta. |
| templates/model | reserva atual respeita o orçamento combinado de 9; autoria de 512, NPCs/templates/população e capacidade de jogadores precisam de limites separados. |

Tipos pequenos de resource/grant/buff/category/modifier continuam adequados aos
catálogos limitados: não transformar todo u8/u16 do repositório em handle de ator.
