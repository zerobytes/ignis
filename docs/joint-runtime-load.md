# Cargas conjuntas do runtime — EAF-140

## Decisão e checklist antes de implementar

Aprovar cargas separadas não prova coexistência. Estender runtime_load, mantendo
receitas anteriores, sem novo framework de testes, dependência ou campo de Scene.

- Mixed: 1.000 jogadores em pares, quatro grupos de 250 por entrega. Direto usa
  dano/buff/ruído, projétil dano/buff, periódico dano, área dano/buff. São os tipos
  permitidos hoje; não habilitar Noise em entregas que o contrato recusa. Pares
  do mesmo grupo preservam dano esperado uniforme, todos no mesmo mundo/fase.
- Boss: 1.000 jogadores num anel de raio90m atacam um chefe central, dano1/buff
  neutro refresh/ruído. Colisores0,35×1,6×0,35m dão espaço real no anel; range100m,
  visão/colisão ativas, alvo com1.000contribuições. Arena3×3pisos de64m, chefe usa
  piso central; remover paredes do recinto63m. Declarar geometria, não extrapolar
  a cena para corpos maiores, obstáculos ou multidão arbitrária.
- Opções de controlador terrestre/passos e até8ouvintes perceptivos em espera
  (boss ocupa1NPC, então até7ouvintes). Ouvintes não precisam observar players;
  som continua anônimo. Passos no boss usam grounding físico KCC, porque o anel
  cruza pisos; não declarar navegação multi-superfície nova.

1. Receita/compile só na preparação, medição separa dispatch, step e drain. Não
   percorrer todos dados no tempo de step só para coletar métricas.
2. Buff/pools/counters reais reutilizados; parâmetros CLI não reservam limites
   máximos de engine nem criam cópias históricas de binários.
3. Fontes/atores/Entity/ID continuam pelo catálogo. Métricas de vida distinguem
   players, chefe e ouvintes sem vitalidade; jamais inferir slot de Entity.
4. Uma função de definição por tipo evita gerar mundos repetidos. Mesmo executor
   existente; nenhuma simulação alternativa ou bypass de visão/colisão.
5. Dano total, saúde de todos, emissões/passos, grounding e ouvintes observáveis
   verificam resultados. Drain termina entregas sem entrar na amostra de600ticks.
6. Scene salva e captura são opt-in de evidência; GPU/JSON completo fora do tempo
   de CPU. Captura offscreen não é FPS nem aceite humano/teclado.
7. Mesma autoridade offline, sem locks/threads/rede/servidor obrigatório.
8. Flags de cena recompõem Engine uma vez. Salvar só documento autorado atual,
   não chamar isso checkpoint persistente do runtime.
9. Sem teste novo por flag. Executar cargas críticas e lint/build do exemplo;
   se revelar defeito de core, registrar e cobrir somente risco crítico afetado.
10. Limites:1.000Players/8NPCs, corpos e topologia declarados, efeitos definidos,
    contagem real de transações; não prometer todas combinações/1.000NPCs/rede/
    GPU/frame60Hz de um cliente final. Registrar capacidade validada e gaps.


## Primeiro diagnóstico medido

Mixed com KCC/passos/8ouvintes deu vida correta e grounding1000/passos18000,
mas p95sim95,386ms. Boss com7ouvintes entre anel e chefe teve3105ataques
legitimamente ocluídos por esses corpos; mover ouvintes para fora do anel, sobre
piso64próprio, preservando obstáculos/visada reais. Isso corrige a receita.

O hear percorre entidades para cada estímulo e chama positions.contains_key
antes de verificar transmission<1. A maioria dos atores não atenua som. Primeiro
incremento mínimo: avaliar transmission<1 e autoexclusão antes do lookup de
presença, mantendo mesma ordem/aritmética/segmento. Equivalência booleana, sem
cache/estado novo; medir isolado sem ouvintes e novamente com eles antes de
adotar índice. Regredir audição/atenção/passos/replay pelo impacto; sem novo teste
que só espelhe a ordem do if. Custos de atenuadores reais continuam explícitos.


A receita boss ainda teve15recusas: rastreamento temporário identificou exatamente
Occluded, de p0298 atrás de p0299 após colisões do grupo nos ticks570..654. BVH e
scan linear concordaram. Não é perda de transação nem erro a ocultar reduzindo
corpos/desligando visão. Removido o rastreamento antes do build final.

Decisão de observabilidade: anel detalhado256 não conserva todos resultados de
1.000fontes. Agregar contadores desde criação/reset de Runtime, por variante de
Outcome, em Vec pre-reservado64 (crescimento só por tipos compilados, não eventos).
Getter readonly e inspeção MCP; saturação explícita u64, sem log ilimitado. Não
adicionar contadores diagnósticos a State/Scene/persistência nem mudar replay por
isso: reconstrução gera os mesmos eventos, WorldState permanece separado. Custo
linear no número pequeno de variantes vistas (normalmente2..4), sem alocação por
evento. Alternativas de reter todos recibos/novo callback não ajudam este contrato.

Boss aceita apenas completed+occluded=tentativas, started=completed e ruídos=
completed; saúde deve ser1.000.000-completed e players permanecem intactos. Essa
contabilidade verifica ações reais e oclusões físicas em vez de exigir tiro livre
permanente numa multidão. Somar agregados deve igualar recibos retidos+descartados.
É diagnóstico, não evento de gameplay nem checkpoint; novas provas só se houver
risco crítico além dos gates existentes e desta carga verificável.

## Revisão final

1. Definições compiladas uma vez; grupos por par e slots estáveis. Hear agora
   filtra transmissão antes do lookup, sem alterar ordem/aritmética dos bloqueios.
2. Buffs/agendas/origens reais coexistem; counters diagnósticos são limitados por
   variantes. Medição não inclui coleta de métricas/JSON/GPU; drain é separado.
3. Ator/Entity/Reference continuam distintos; chefe e ouvintes têm contabilidade
   própria. Scene88/replay97 e contratos de tipos de efeito preservados.
4. Nenhum bypass de visão/colisão:15oclusões reais explicadas, nenhuma transação
   perdida. All attempts=completed+occluded; ruídos=completed; vida coerente.
5. Quotas reais mantidas. Mixed alcançou250projéteis/250periódicos/500áreas/
   750buffs; boss1000buffs. Filas:16128ruídos reservados/1000passos, sem histórico.
6. GPU somente no probe offscreen separado. Fila completa e counters sob inspeção;
   anéis limitados não determinam gameplay. Nenhuma janela humana tocada.
7. Mesma autoridade sequencial/local; nenhuma thread/dependência/PRNG nova.
8. Reset/reconstrução zeram contadores Runtime; não são parte de State salvo.
   Replay/prefixo recomputam eventos; contadores não são checkpoint do mundo.
9. Sem novo teste por flags/contador/if.67Rust existentes,Clippy all-targets e do
   exemplo,12MCP/47grupos passaram. Probe real MCP1000ações/controle terrestre,
   replay e PNG idêntico no mesmo build. Legado offline/rede negada preservados.
10. Mantidos: avaliação real de contribuições no chefe e atenuadores de som,
    deslocamento denso de refresh, manifolds/casts Rapier, serialização/GPU sob
    pedido. Sem promessa de todas combinações, navmesh/multidão ou FPS gráfico.

Build final875c0d48a41a08a0 (macOS Apple Silicon/release), fonte/binário/example
auditoriados antes dos gates. Receita padrão60warmup+600ticks medidos, sem ritmo
de parede, sem testes concorrentes nossos; app humano preservado.

| Receita | p50 sim | p95 sim | p99 sim | máximo sim | p95 total | máximo total |
|---|---:|---:|---:|---:|---:|---:|
| mixed/KCC/passos/8ouvintes |3,799ms|7,356ms|7,736ms|8,719ms|8,009ms|9,376ms|
| boss/KCC/passos/7ouvintes |4,332ms|16,096ms|16,442ms|17,692ms|16,791ms|18,431ms|

Mixed:110000ações,110impactos/vida999890para cada player,27500ruídos,18000passos,
1000motores grounded e1000emissores, todos8ouvintes ouviram.750origens alocadas,
5394000B reservados em buffers de snapshot (não RSS). Antes do filtro acústico,
mesma carga física/participantes tinha p95sim95,386ms; vida/emissões preservadas.

Boss:110000tentativas,109985completed/started,15occluded; vida890015no chefe e
1000000em todos players.1000buffs simultâneos,109985ruídos,18182passos,1000motores
grounded e1000emissores,7ouvintes ouviram. Pico RSS21348352B (~20,36MiB), footprint
11338208B (~10,81MiB), medidos por time-l; não inclui GPU/rede nem limites máximos.
O p95total ultrapassa16,667ms: não afirmar deadline60Hz completo neste caso.

Probe MCP separado:1.000comandos reais contra o chefe emtick0,1.000commits,
chefe999000; render960×640 e replay com PNG exato SHA256
4ab3cac7d03f797d89bc62426d370257b41a9c82e358b86302231e528636b7d6.
Imagem de caixas/indicadores minúsculos num anel: prova de fluxo/captura, não arte,
legibilidade de1.000personagens, teclado humano ou qualidade de jogo.

Arquivos atuais: artifacts/joint-runtime-{audit,report,probe}.json, respectivos
{integration,clippy,example-clippy,build,mcp,memory}.log; runtime-load-{mixed,boss}.json,
runtime-load.scene.json, runtime-load.png/runtime-load-replay.png e
joint-runtime.replay.json. Amostras anteriores falhas/diagnóstico não são gates.
Transporte real foi autorizado por Guilherme nesta execução e será a próxima
camada. Não é parte destes números; multiplayer entregue ainda não está alegado.
