# Agendas por fonte e dissipação local — EAF-136

## Decisão anterior ao código

Estender Pool<T> de EAF-135 a periódicos (64por slot/teto real slots×64) e áreas
persistentes (16por slot/slots×16), com índice por fonte e, para periódico, por
alvo. JSON de active continua lista e mutação encapsulada mantém ordem densa/
derivados. Refresh consulta só bucket da fonte; dissipação consulta bucket do
alvo, sem varrer agendas de outros alvos. Fontes/gerações antigas seguem cobradas
no slot. Não ampliar buffs64, bases512 ou ruídos16 nesta etapa.

Máscara periódica u64 não pode representar agenda ampliada. Escolha: conjunto
local ordenado de IDs de execução (até8efeitos×64remoções=512), vazio sem heap e
alocação somente quando seleciona remoção. Busca binária e remoção em ordem densa;
ID não muda com compactação/refresh. Buffs ainda64mantêm sua máscara válida até
migração própria. Rejeição tardia descarta seleção e todos writes/custos; quotas
revalidam no commit/impacto. Nenhum plano/cópia com tamanho do mundo.

Não repetir clean_areas do mundo para cada lançamento: marcar necessidade uma
vez no início do tick e quando morte altera lifecycle; limpar nas mesmas fases/
pontos anteriores, mas somente se houver nova mudança. Isso preserva recibos
SourceDead antes de Created após morte e evita1000varreduras no tick de admissão.
Capacidade de área lê só entradas vivas da fonte (até16); soma das quotas implica
teto real global, com limpeza pendente aplicada antes de instalar.

Scene87/replay95, sem novo campo autorado/dependência/worker/rede. Autoridade
sequencial, pulso antes de dispel/novos casts e conhecimento NPC permanecem.
Carga adiciona receitas de periódico/área ao workload existente, sem retirar
candidatos de diagnósticos/efeitos ou declarar custo de área antes de medir.

## Checklist estático prévio

1. Quota/refresh por fonte e seleção por alvo, índice recomposto só em mudança
   estrutural. Limpeza global por tick/lifecycle, não por entrega sem mudanças.
2. Pool/buckets reutilizados, plano local512IDs com Vec vazio por omissão;
   ordenar até512 IDs aceito para manter lookup/removal independente de índice.
3. actor/targetu16, execução64bits; quotas64/16 por slot, sem confundir instância,
   posição densa, definição/grant ou máscara de buffs64.
4. Índice-alvo preserva ordem densa; pares/refresh sem scan de todas agendas.
5. Limites reais derivados dos slots; fonte cheia não bloqueia outra; refresh
   não cobra vaga nova. Sem custo/cooldown/HP/remoção após falha de plano.
6. Sem I/O/GPU novo no tick, inspector somente sob demanda, receitas headless.
7. Mesmas fases/TOI/vida/ordem InstanceId; seletor por IDs não ordena autoridade
   nem corta vítimas, agenda continua amostrando alvos na hora do pulso.
8. push/replace/compactação/clear/serde reparam source/target; flag de limpeza
   reinicia por tick e acompanha mortes entre transações, reset/edit/load recriam.
9. Somente prova crítica1000agendas/quota/refresh/IDs>63/rollback/morte/serde/replay;
   cobertura existente afetada e carga reproduzível, sem testes por detalhe.
10. Custos mantidos: amostrar todos candidatos/recibos de área, efeitos reais,
    alocação local na dissipação, BTree de origens e JSON/Rapier. Sem inferir
    combate arbitrário/render/rede; medir receitas e registrar gargalos restantes.

## Revisão final e evidência

1. Pool por fonte limita refresh/capacidade a64periódicos/16áreas; bucket de alvo
   limita busca de dissipação ao alvo. Compactação global continua necessária ao
   remover entradas de uma ordem densa, somente uma vez por operação/fase.
2. Buckets/Vec reutilizados; IdSet<512> vazio não aloca, insere ordenado apenas IDs
   selecionados e é movido entre efeitos do plano, sem clone de estado global.
3. Cast.id64 identifica remoção; target/actoru16. Buffs continuam64/máscara u64
   válida. Pool valida fonte/alvo/quota antes de mutar índices; serde repara ambos
   e recusa periódico sem alvo. ActorSlot e Entity/lease não são intercambiáveis.
4. matching usa source_positions; select_dispel usa target_positions e mantém
   seleção na ordem densa. ID de refresh novo não muda posição/ordem da agenda.
5. Quotas por slot impedem monopolização; outra fonte não preenche vaga do cast
   pendente. Fixture anterior de área global foi adaptada a essa nova regra;
   impacto pago com pool da própria fonte cheio continua falhando sem reembolso.
6. Sem GPU/I/O novo; recipe e inspector usam dispatcher compartilhado existente.
7. cleanup flag marca início do tick/morte, limpa nos mesmos pontos; mantém
   SourceDead antes de Created após morte. Pulso antes de dissipação e commits
   sequenciais preservados, sem efeito/custo após falha de plano.
8. replace só aceita mesma fonte/alvo; compactação/clear/serde recompõem derivados.
   Reset/edição/load recriam flag/índices, prefixo/replay reconstruídos exatamente.
9. Um novo cenário crítico1000periódicos+1000áreas, quotas64/16, refresh alto,
   dissipação64IDs fora de ordem numérica/posição>63, erro tardio MissingResource
   depois de selecionar remoções/dano, serde/alvo inválido, morte/recibos/replay.
   131Rust selecionados (114integrações+16origins+1novo),Clippy all-targets e
   14MCP/57grupos aprovados. Nenhum teste novo por detalhe trivial.
10. Mantidos: classificação/ordenação completa dos diagnósticos de área, BTree
    de origens, heap local no dispel, recibos/JSON/Rapier e custo real de vítimas.
    A medição identifica gargalo; não extrapolar capacidade a todos combates/rede.

Main após cf55cb7, Scene87/replay95; build `bfdfc6445209aaae` auditado contra
fonte/binário/example. O primeiro lote MCP começou antes de fechar o build;
foi repetido com fonte/binário final congelados, antes das medições isoladas.

runtime_load --delivery periodic:1000fontes/1000agendas de pico,110000aplicações,
110pulsos/vida999890 por jogador, zero histórico.60warmup+600medidos, p95simulação
2,955ms/p99=3,087/máximo3,262ms; total p95=3,710/máximo4,058ms.1tick de drain
fora da amostra, custo reportado separadamente, confirma último pulso.

--delivery area:1000fontes,2000volumes de pico (sobreposição de uma janela),
mesmo110000lançamentos/110pulsos corretos por jogador, zero histórico;
p95simulação78,607ms/p99=79,128/máximo89,137ms; total p95=79,315ms.2ticks de
drain reportados separadamente. Esta receita não cabe em16,7ms:1000pulsos ainda
classificam/ordenam1000candidatos cada, mesmo selecionando apenas o próprio autor.
Próximo recorte otimiza consulta/diagnóstico sem remover participantes/vítimas
nem omitir rows/ordenar autoridade de outra forma. Buffs/bases/ruído seguem
pequenos e não foram incluídos. Sem renderFPS/cliente de rede/latência prometidos.
Logs/relatórios:artifacts/scheduled-*,runtime-load-{periodic,area}.json.
