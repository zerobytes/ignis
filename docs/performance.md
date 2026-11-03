## EAF-200 — coleção adicional de mato alto

Contrato em [tallgrass-assets.md](tallgrass-assets.md).8tipos/24variações/39fontes
editáveis, sem escala global do mesmo tufo: hastes folhosas, ramos laterais,
espigas cilíndricas, plumas e flores. Biblioteca84154v/158294tri com chão;
650tufos/14×14m,1333instâncias/3197854tri antes do descarte. Fonte1878342bytes;
só folhagens825instâncias/1616822tri. Compartilhamento não elimina custo por
cópia, sem FPS/memória total/LOD novo inferidos. Coleção baixa aprovada preservada:
fontes e18PNGs exatos. Save/load alto inicialmente1pixel/1RGB8 diferente; valores
de cena iguais e segundo par PNG byte exato, discrepância registrada sem causa
inferida.60meshes vegetais fechados/orientados, seed/composição/identidade/hash
verificados pelo build corrente, sem novo executável/renderer/solver/formato.
Galeria artifacts/tallgrass/review.html; aceite artístico alto e terreno/vento/
gameplay permanecem separados. Mundo/janela humanos preservados.

## EAF-200 — assets de gramas e matos compartilhados

Contrato em [groundcover-assets.md](groundcover-assets.md).12 tipos/36variações,
57fontes curvas Source/Recipe7 compiladas pelo Rust corrente; sem geração por
frame ou alterações de renderer/solver/formatos. Biblioteca completa48634v/
91502tri incluindo chão;57fontes1106617bytes. Campo10×10m/2400tufos usa4069
instâncias/4643358tri somados antes do descarte; campo somente gramas2401/
3251330tri. Compartilhamento poupa duplicação de geometria, não trabalho por
cópia; biblioteca completa reside nas demos, composição importa apenas usados.
Não inferir FPS, memória total, LOD, streaming, vento ou física de gameplay.
Load/SaveScene nativos/fechamento dos78meshes/reprodução/identidade e recusa de
biblioteca desatualizada verificados; galeria em artifacts/groundcover/review.html.
Build corrente reutilizado, sem binários novos ou mudanças no mundo/janela humanos.

## EAF-198 — vales e faixa de encontro do colo

Contrato em [botany-collar.md](botany-collar.md). Source/Recipe7 conservam os
bakes1..6 e resolvem posições do interior do colo, com guia de vales/derivadas
de seção.192iterações e três pesos de guia por candidato, faixa suave e64passos
de guarda; busca limitada de colares prioriza ajuste completo. Adjacência e
buffers locais ao colo, sem resolução extra na copa, geração por frame, I/O,
rede, shader/passeGPU ou cache global novo. Não se enfraquece a guarda para
aceitar uma dobra; escultura incompatível recusa atomicamente. Caso RS mantém
2884806v/5644424tri e37101362bytes, curvas/órgãos e solver inalterados; ensaio
1175104nós/5511516restrições. Publicação adulta2,91s neste roteiro não é FPS
nem latência universal. Quotas combinadas preservadas; um release corrente,
mundo/janela humanos preservados. Massa por seções não é volume exato do colo.

## EAF-198 — paredes, cristas e nervuras

Contrato em [botany-buttress.md](botany-buttress.md). Perfil opcional nativo,
12amostras mínimas só nas raízes afetadas; sem aumentar a resolução da copa.
Colares exatos e extensão de deslocamentos limitada à junção (160iterações,
32passos de homotopia), com recusa de colapso/inversão local. Campo de nervuras
compartilhado por árvore atua uma vez sobre a pele costurada, sem novos passes
GPU; acima da faixa ativa conserva pontos e ignora o cálculo angular. Perfis
opcionais em Arc e índice de árvore por pose na preparação, sem dados volumosos
embutidos em cada folha. Quadratura24 na pilha prepara área/massa/proxies;
ensaio continua lattice reduzida, não tecido da parede/solo constitutivo.
Correção de normalização compara spans atuais/de repouso; não se afirma
identidade das trajetórias XPBD antigas após corrigir a distorção. RS adiciona
972v/1944tri ao tubular5e37102, total2884806v/5644424tri dentro das mesmas quotas.
Fonte37.101.362bytes, ensaio1175104nós/5511516restrições/672âncoras. Publicação
completa medida2,79s no roteiro não é FPS. Sem novo I/O/rede/lock no frame estável;
cópias autorais e bake completo mantêm custo real. Um release corrente, mundo/
janela humanos preservados. Nenhuma gramática compacta ou custo universal inferido.

## EAF-198 — sistemas de raízes e colo multicolare

Contrato em [botany-roots.md](botany-roots.md), análise anterior em
[root-system-analysis.md](references/botany/root-system-analysis.md). Geração
seeded nativa por autoria, até16 sistemas/12 primárias/3 ordens; expansão
exponencial limitada. Cópia da fonte Arc somente na alteração e bake completo
em candidata antes de publicar. Busca de colares limitada, até13 colares e160
amostras por colo; buffers de envoltória/refinamento/relaxação por junção.
Sem trabalho novo no frame estável, passe/shaderGPU, I/O/rede ou lock global.
Domínio material por conectividade/resolução/portos/cortes recusa perda de escultura.
Ensaio guarda um bool adicional por nó e prepara parentes/âncoras uma vez;
não regenera curvas ou consulta a fonte por passo. Solo é plano autoral, não
simulação constitutiva. RS:49 eixos novos, +10.308v/+20.616tri, total
2.883.834v/5.642.480tri;1.175.104 nós/5.511.516 restrições e672 âncoras.
Mesmas quotas combinadas de fonte/malha/ensaio/contato;160 amostras impõem
quantidade menor de primárias em resoluções maiores. Publicação completa
medida em2,72s neste roteiro não é FPS nem latência interativa garantida.
JSON sem espaços reduz tamanho serializado, sem gramática compacta nova.
Um release corrente; mundo/janela humanos preservados.

## EAF-197 — densidade de ramificação e colares

Contrato em [botany-branching.md](botany-branching.md). Geração autoral Python,
mais eixos/ordens e menos folhas por terminação dentro dos mesmos envelopes;
sem trabalho novo no frame estável/GPU/rede. `Builder.by_id` mantém referências,
sem copiar os órgãos, e elimina buscas lineares por seção. A correção de anéis
quase coincidentes faz um passe nos samples e um pequeno buffer de fronteiras
por tubo durante o bake; evita interpolar cada fronteira para cada sample.
Colares/endpoints e seus endereços são preservados, amostras redundantes próximas
são omitidas, perda de escultura é recusada. Não há novo cache global/lock/passe.
A expansão por ordens permanece exponencial (limite0..3; perfis1/2); o compilador
continua responsável pelas quotas combinadas e pelas recusas de encaixe. Custo
real de junções/malha/XPBD não é convertido em promessa de FPS/gameplay.

## EAF-187 — lábios, bases adultas e pivôs rígidos

Contrato em [character-refinement.md](character-refinement.md). Fonte3 com duas
bases (~73MB) em cache, até96MiB; leitura1/2 conserva64MiB. Sem fonte/parse por
frame. Novos pivôs transportam o twist ocular já ajustado, com um snapshot extra
de bind apenas quando o primeiro warp rígido está ativo; mapa pequeno por socket,
sem nova busca/solver de gameplay. Coordenadas de repouso ativas do corpo custam
160560bytes por bake, copiadas na edição e compartilhadas pelo histórico existente.
Estrias usam32segmentos do Detail existente: custo de material/pixels explícito,
sem shader/passes novos, geometria extra ou promessa de FPS. Ganho0 conserva o
material/stream anterior; grupos/descritores pequenos só pertencem à UI autoral.

## EAF-187 — corpo ampliado e pontos faciais

Revisão em [character-controls.md](character-controls.md). Fonte2 validada46,07MB
em cache;155canais e42pontos derivados apenas quando o bake/revisão muda. Sem
I/O ou compilação no frame estável. Profundidade dos pontos usa o mesmo índice
do editor por revisão/tick. Arraste prepara uma candidata apenas quando muda o
ponteiro; não acumula cenas no histórico durante a prévia. Soltar usa uma só
transação e os Arcs/undo32 existentes; cancelamento descarta a candidata.
Warps, ajuste do rig, validação e hash permanecem custos síncronos de autoria.
Sem novo deformadorGPU, dependência/runtime MakeHuman, thread ou custo de
simulação; não foi medido FPS/latência artística desta interação. Limites64pontos/
16warps/256canais e4M deltas+pesos preservam orçamento finito de preparação.

## EAF-187 — parâmetros no editor

Revisão em [character-base.md](character-base.md). Fonte validada em um Arc de
sessão, sem ler/parsear39MB a cada frame/slider. Configuração pequena e descritores
alimentam os controles; bake só quando um parâmetro muda. Fingerprint JSON em
writer SHA incremental evita buffer JSON temporário do bake. Receita/bake entram
no mesmo EditSceneLive, com32 snapshots de undo já existentes e geometria/rig
compartilhados por Arc. Revalidação de consistência ocorre por revisão/binding,
nunca por frame inalterado; aceite de um bake recém-gerado evita recompilá-lo.
Trocar binding/undo pode recompilar e hash do bake é custo de edição aceito.
Cache mantém uma fonte; inspeção/exportação pode materializar JSON pequeno.
Não é deformação GPU contínua, garantia de FPS ou escala de famílias animadas.

## EAF-183 — cristas e região representativa

Checklist antes/depois em [representative-world.md](representative-world.md).
Sampler/orientações preparados, saída regional tipada com preflight1Mv/64Mwork,
sem mesh JSON noRPC nem geração por tick. Um asset compartilha normais/índices;
buffers CPUgeom9,01MiB separados de BVH/GPU/RAMtotal. Fonte física completa,
renderer intacto; oclusor177 não seleciona o asset inteiro524ktri e a futura
partição precisa conservar costuras. Nenhum ganhoFPS ou streaming inferido.
9Rust/4alvos,4MCP11grupos+composição/caminhada,Clippyalltargets/fmt/Python/build,
no5c2e797b819983f2. Arte final184 e aceite humano separados.

## EAF-182 — superfícies físicas do terreno

Contrato e checklist antes/depois em [terrain-collision.md](terrain-collision.md).
Mesma fonte visual completa e transformação alimentam TriMesh; BVH compartilhado
por asset/escala no mundo, caminho vazio sem índices e nenhuma preparação/serde
por tick. Scratch de spawn limitado e f64 local para preservar buracos; erros
recusam antes da identidade. Custo físico/BVH frio e residente explícito, inclusive
preparação separada do planejador; nenhum renderer/passe/lock novo ou FPS alegado.
79Rust/9alvos,9MCP31grupos,Clippyalltargets/build/fmt/Python; demo técnica e PNG
replay exato, aceite humano distinto. Build corrente b13af5530ec3621b.

## EAF-179 — publicação de topografia no renderer atual

Inventário/contrato em [terrain-render-integration.md](terrain-render-integration.md).
Receita da cadeia aprovada extraída para build reutilizável, assets originais
iguais porSHA256. Nenhum runtime/ABI/passe/cache novo;15pares de câmera/luz/resize
em terreno,formas3D e variantes de montanhas preservam pixels com os atalhos
sem perda atuais. Resumos atmosféricos também exercitados, sem novoLOD ou
benchmarkFPS inferido.6Rust/6gruposMCP/Clippy do exemplo e fontes editáveis
verificados no app44dae857d2128e92, sem cópia de executável.

## EAF-178 — atributos e shader conforme canais consumidos

Contrato, revisão estática e evidências em [minimal-vertex-stream.md](minimal-vertex-stream.md).
Stream24B sem quantização e especialização de compilação para materiais elegíveis,
mesmo kernel de iluminação. Sem novo passe/I/O/readback/tick; cache invalida por
mudança de material mesmo sem mudança nos vértices. Sobre177, draw+espera
70,700→44,700ms perto e85,962→62,805ms panorama; RGB8exato, mesma geometria.
Buffers geométricos -798.912B, CPU quase igual; não converter em FPS garantido.

## EAF-175 — indexação e atributos compartilhados

Decisões, checklist e evidências em [small-geometry-cost.md](small-geometry-cost.md).
Vértices48bytes, constantes por instância80bytes, índicesu16/u32 quando economizam;
deduplicação só no upload, cache e fallback procedural após diferença temporal
observada no Metal. Ordem entre instâncias preservada mesmo com páginas. Na
floresta, draw+espera54,765→38,315ms e70,607→46,370ms; buffers geométricos
15,78→9,49MiB. RGB8 exato nos11pares e32capturas repetidas. CPU sem ganho
consistente; vista leve4,230→4,251ms. Fonte CPU e triângulos escolhidos preservados,
sem FPS/oclusão/corte por3px/platô universal inferidos. Projeção experimental off.

## EAF-172 — meshes conforme conteúdo e GPU instancing

Preflight, alocações/cópias, representação binária, caches, fallback e medições em [mesh-resources.md](mesh-resources.md). Hierarquia iterativa/dinâmica; geometria estática repetida armazenada uma vez, 80 bytes por instância, tabela de materiais conforme device. Ordem autoral preservada. 10 mil instâncias/1,28M triângulos: mediana draw+espera12,151→6,526ms no M4Pro/Metal, pixels exatos, dados geométricos+instâncias234,375→0,786MiB. São ensaios offscreen; não FPS de gameplay ou garantia universal. Variação/regressão exploratória leve documentada.

## EAF-160 — IK, root motion, retarget e dispositivo

Contrato/tradeoffs prévios e dez respostas finais em [npc-presentation-complete.md](npc-presentation-complete.md).
Retarget fora do tick, clips nativos/Arc; IK até8cadeias/40rays do snapshot físico,
arrays32/24 e skinning CPU. Root só opt-in/KCC após intenção e antes de circulação,
sem teleporte pós-solver; Motion Copy por identidade/vida e seleção por episódio.
Relógios versionados e validação de rig antes da mutação da réplica. Áudio CPAL
opcional/desligado, mailbox8ticks/400amostras/64vozes, callback sem mundo/I/O.
Busca limitada, matrizes/StateSet preexistentes e sincronização interna std mantidas;
sem promessa FPS/latência.49Rust/10alvos+1callback,17MCP76grupos, Clippyalltargets
+semfeatures aprovados; áudio físico/arte humana separados, sem janela/dispositivo.

## EAF-159 — circulação aérea XYZ

Decisão prévia e dez respostas finais em [npc-air-circulation.md](npc-air-circulation.md).
Sensor/BVH/snapshot reutilizados,16vizinhos/128folhas,até11comandos×120passos
pelo motor comum antes de comprometer o primeiro passo. Sem forças posteriores,
novos pools/workers/locks/GPU/dependências nesta frente. Risco residual/truncamento e
rejeição estática observáveis; salto/planeio só cedem antes de lançar se apoiados.
Custo local de previews e previsão linear de vizinhos mantidos, sem promessa de
FPS, arbitragem universal, streaming ou novo formato portátil de save.

## EAF-158 — checkpoint e fechamento NPC v1

Revisão prévia e dez respostas finais em [npc-checkpoints.md](npc-checkpoints.md).
CBOR/SHA/fsync só em comando pausado; captura limitada fora do tick, sem serialização
na decisão. Identidades/sequências/contatos preservados, caches recompostos em candidato
isolado; formato até 128 MiB e recursão 128. Cópias de snapshot/buffer/candidato e
recompilação no load são custos aceitos, não zero-copy nem load sem pausa. Baseline
binária evita JSON dos internos físicos, continuous segue sem log. 79 Rust/12 alvos,
26 MCP/92 grupos e check/Clippy all-targets aprovados. Sem FPS ou portabilidade de
saves entre Rapier/plataformas inferidos. Matriz NPC v1 tecnicamente fechada.

## EAF-157 — população NPC e sensores espaciais

Revisão em [npc-population-scale.md](npc-population-scale.md). Default8/opt-in128,
Vec de membros/relatórios por mundo e sem cópia de lista por tick. Flags agregados
na BVH e máscara4096bits preservam ordem do som sem heap.59Rust/9MCP35grupos;
100NPC+500Players/600KCC/71450hits/5181passos corretos, p95sim18,576ms/total18,706ms.
Classificação NPC×atores e planejamento síncrono finito mantidos; sem60Hz/FPS/WAN
ou128planos pesados simultâneos prometidos. Zero históricos em continuous.

## EAF-156 — projeção de avatar e PCM local

Revisão em [npc-presentation.md](npc-presentation.md). Arc/Frame reutilizados,
instâncias derivadas sem inflar hierarquia32, orçamento visual com fallback físico.
PCM i32→i16 por captura e vozes64. Busca nominal limitada16/40 e skinning CPU
mantidos explicitamente; stream completo de poses por snapshot, sem ganho de FPS
inferido.29Rust/6MCP23grupos/Clippy aprovados, sem dispositivo de áudio automático.

## EAF-155 — rotinas, diálogo e objetos transacionais

Checklist prévio/depois em [npc-routines-interactions.md](npc-routines-interactions.md).
IDs compilados, Writes8, quotas16ações/tick e cursor justo, sessão terminal independente
do ring. Snapshot movido sem clone; rebuild físico agrupado mantém endpoint aéreo.
52Rust/11MCP e Clippy; sem FPS/escala NPC/custo de rebuild universal prometidos.

## EAF-154 — revisão de tática e coordenação

Preflight/dez respostas e contrato em npc-tactics-cooperation.md. Posição/evidência Copy, até64amostras de mapa/anel com BVH e apoio; IQ limita esforço/cadência, traits ponderam distância/cobertura/flanco. Dois loops do catálogo para tática/ordens são custo explícito, sem varrer todos os candidatos por objetivo. Lotes de ordens alocam somente sob comando, validam todos antes de publicar e usam fila64existente. Apoio mantém1lease/fonte e diagnóstico limitado por atores, Vecs reutilizados/reservados antes de commit só se opt-in, sort_unstable e ActorSet de bloqueio; cleanup consulta buckets por fonte em vez de pools globais por lease. Máscara de prontidão automática não impede comando humano, mantém cast ativo; serde valida orçamento/dono único. Sem Scene clone/JSON/reflexão na decisão, sem worker/LLM/rede/GPU novos.27Rust/3alvos+8MCP25grupos,check alltargets/Clippy/fmt aprovados. Queries por candidato e buckets de entrega mantidos; sem FPS/escala universal ou ausência universal de overheal inferidos.

## EAF-153 — revisão da circulação local

Preflight/dez respostas em npc-circulation.md. Snapshot reutilizado só quando circulação existe em definições/templates; BVH existente,128folhas/16vizinhos/5..9alternativas, arrays no stack e perfil neutro constante. Scores não acessam alvo/vida oculta/facção; proximidade oclusa por sólidos. Até2passosKCC por alternativa com apoio, legado usa volume/superfície; sem sondagem sem risco. Intenção atual do KCC separada da velocidade física anterior, correção encontrada pela prova integrada. Cast/fila zero não geram deslocamento; trajetória aérea comprometida permanece. Diagnóstico Copy no NpcState/replay, JSON só em inspeção; sem worker/lock/GPU/ABI nova.13Rust/3alvos,6MCP19grupos,check alltargets/Clippy/fmt aprovados. Custo local×vizinhos e preparação World mantidos; sem profiling/FPS/escala universal inferidos.

## EAF-152 — revisão da navegação gerada

Preflight em npc-navigation-generation.md. Normalização só regenera se assinatura física/receita/grafo mudar; colisores ordenados por ID como no World, corrigindo invalidação da primeira edição só pela ordenação posterior do documento. Fonte de alturas usa lista fixa ordenada,8alturas no stack por coluna, World único/caixas no stack; Vecs de autoria64nós/128links e até4032pares são limitadas e fora do tick. Cache não é serializado/aceito de arquivos; perfil mental não altera física, mas continua limitando planejamento/preferência.64colunas/512amostras aéreas/64nós/128links, padrão512probes/24000passosKCC, auxiliares aéreos limitados pelos validadores. Preview read-only, autorias com1undo, manualnetwork limpa receita. Obstáculos dinâmicos não são previstos; mapa é layout conhecido explícito sem alvo oculto. Runtime conserva motivo generated_navigation_incomplete para falha de rota sob mapa truncado.6Rust/2alvos+7MCP24grupos e check releasealltargets/Clippy lib/bin/test-Dwarnings/fmt passaram; só1nova prova crítica, demais coberturas por impacto. Sem FPS/navmesh/multidões/escala alegados.

## EAF-151 — revisão de mobilidade híbrida

Preflight/alternativas em npc-hybrid-mobility.md. Troca de tipo apenas na mudança de modo do mesmo corpo, sem teleporte/recriação/Scene clone; estado motor Option<Mode>/airborne pequeno. World/KCC/cache permanecem, compilação só walk peloKCC e outros links usam validadores atuais; lançamento revalida só trecho escolhido. Custos mentais inteiros fora de Motion preservam duração/distância/velocidade físicas. Wrapper publica último motor com lookupBTree adicional limitado, BVH pré-KCC ainda publicada quando há controlador ativo mesmo se aéreo (custo conservador conhecido). Sem workers/locks/PRNG novos, sem GPU/renderer/protocolo2 novos. Morte/retorno/reuso/reset recompõem motor; cast aéreo comprometido continua recusado. Corrigida âncora híbrida: livre só precisa/capacidadefly/clearance, evitando ciclo descer/subir.30Rust/3alvos e7MCP26grupos selecionados no fonte final;1prova crítica ampliada comwalk/fly/jump/glide/posse física/identidade/replay e morte-retorno no ar. Primeiro gate50Rust e verificações atuais de controller/ground/render mantidos; não inventar total de testes executados na mesma revisão. Sem FPS/navmesh/escala alegados.

## EAF-150 — revisão do código implementado

1. Perfil quantizado/cacheado só na primeira decisão de cada indivíduo;9misturas seeded fora da pontuação por frame, sem consumir PRNG global.
2. Profile/Context/Decision Copy,9traits/8scores e arrays limitados; nenhum heap/Scene clone por score. Inspeção monta JSON apenas sob comando.
3. Inteiros em utilidade/margem/contagens, autoriaf32 finita0..1; produtos máximos<600milhões em i32, vida própria em i128 evita overflow.
4. Reutilizado actor_slot existente em vez de nova busca linear; perfil comum atende objetivo/investigação/planejamento. Array do perfil na decisão é cópia pequena explícita para evidência/replay.
5. Até8objetivos;1..3entradas/1..4saídas/600..2400passos, sem aumentar tetos globais. Baixo esforço publica truncamento, nunca impossibilidade fictícia.
6. Render e protocolo2 inalterados; PNG offscreen/prefixo byteexatos. Formatação do índice de módulos da rede apenas ordena declarações.
7. Mesmo dispatcher, nenhuma nova concorrência/consulta oculta/lock/entrega. Filtros de elegibilidade precedem scores e prioridade autoral preservada.
8. Configuração imutável por sessão, reset/replay reamostram por indivíduo, spawn recebe perfil independente; morte cancela seletor e novo nascimento não herda decisões.
9.34Rust em4alvos existentes/1prova crítica nova,6MCP27grupos por impacto e check releasealltargets/Clippy lib/bin; demais eixos continuam abertos.
10. Pontuação/contexto calculados até8opções para diagnóstico; cadência restringe troca, não promete zero CPU entre avaliações. Sem profiling/FPS/MMO alegados.

## Cognição NPC — avaliação antes do código

1. Compilar/variar perfil uma vez por indivíduo; utilidade no máximo8objetivos, cadência limitada com invalidação imediata.
2. Traits/scores Copy/arrays no estado existente; nenhum Scene clone, lista de mundo ou heap por pontuação.
3. F32 de autoria validado/quantizado; scores inteiros, sem JSON/reflexão/dependência/ABI nova no tick.
4. Perfil comum por consumidores; declarar apenas integrações efetivas, sem duplicar cérebro em cada sistema.
5. Tetos globais de busca preservados, esforço individual e truncamentos explícitos; escala NPC ainda aberta.
6. GPU/render inalterados, inspeção/JSON/replay sob demanda e evidência offscreen.
7. Mesmo dispatcher, sem workers/locks/RNG global ou consulta oculta nova.
8. Perfil por indivíduo, novo nascimento independente; edição/reset recompilam e replay conserva decisões.
9. Novas provas só para atomicidade/identidade/conhecimento/replay, gates existentes por impacto e comparação MCP.
10. Arrays/custo finito de utilidade e busca mantidos, sem profiling/FPS/escala alegados. Programa9eixos continua aberto.

## EAF-145 — receita conjunta local/rede

Preflight/revisão em joint-network-load.md. Receitas compartilham suporte sem
kernel duplicado; preparação separada de before/sim/after e collect. Gerador
finito100rounds/15sround/120stotal,1000TLS reais com4workers/32endpoints explícitos.
Gate crítico verifica resultados/HP/pés/passos/entregas/frescor/história, sem teste
para cada flag/counter. P95total autoridade mixed7,449ms/chefe17,778ms; lag p95250/
100ms e pico mixed1033ms mantidos como limites. Sem ganho de FPS, WAN,60Hz universal
ou causalidade de parâmetros do gerador inferidos. Local sem rede repetido no
mesmo build e dados auditados, nenhum renderer/socket no tick de simulação.

## EAF-148 — ordem de ACK e recursos do gerador

Checklist/revisão em replica-ack-order.md. ACK monotônico O(1)/mutex existente,
1counter agregado, sem mudança de formato/cache/encoder/heap do tick.28Rust e
7MCP27grupos por impacto; só1nova prova crítica Hub/codec/Replica/ACK fora de ordem.
Carga1000×100mixed/boss mantém todos participantes e termina1000réplicas no mesmo
tick. Counter regressivo0 no ensaio, sem causalidade de lag atribuída. Perfil do
gerador4workers/32endpoints é explícito;2parâmetros mudaram, não alegar ganho
isolado de throughput. P95total chefe17,778ms, sem60Hz/FPS/WAN universal.

## EAF-147 — recuperação limitada TLS

Preflight/revisão em initial-handshake-recovery.md. Loop até3×3s/backoff100/200ms
somente no handshake antes de Hello; sem workers/buffers/filas/dependências novos,
crypto/pin falha imediatamente. Native observa stop25ms durante connect.27Rust/
7MCP27grupos e cargas1000 aprovadas; só1nova prova crítica de blackhole/posse/
retry seguro/pin/cancel, sem testes por campo. Custo extra só sob expiração;
offline não cria I/O. Não alegar causa original corrigida ou disponibilidade WAN.

## EAF-146 — contato estável de caixas no KCC

Checklist prévio/decisão/revisão10itens em kcc-contact-stability.md. Adapter no
stack reutiliza contato SAT inicial do Parry; restantes consultas delegadas,
mesmo planner/runtime, sem varrer mundo/heap/solver novo.77Rust por impacto,
11MCP44grupos e1000TLS×100ações mixed/boss corretos, todos com passos/grounding.
P95total do chefe18,023ms; sem ganho de FPS/60Hz universal alegado. Replay98,
Scene88; nenhum teste por delegate/flag, apenas1nova prova crítica de chão real.

## Política de testes por risco — direção de Guilherme em 29/10/2026

- Criar testes novos somente para comportamento crítico: perda/corrupção de estado, identidade e posse, atomicidade, formatos/persistência/replay, segurança ou falhas de integração com consequência material. Explicar brevemente qual risco o teste protege.
- Não criar teste para cada mudança, detalhe visual, texto, refatoração trivial ou comportamento que só espelhe a implementação. Usar inspeção, compilação, análise estática ou verificação direta quando suficientes.
- Manter e executar cobertura crítica útil. Selecionar testes e cenários existentes conforme o impacto; não repetir automaticamente toda a suíte a cada incremento. Ampliar a regressão quando mudanças estruturais ou falhas deixarem riscos concretos sem cobertura.
- Não remover testes em massa nem relaxar critérios para fazer passar. Se um teste deixar de proteger um contrato relevante, revisar sua utilidade no contexto da alteração.
- Esta política prevalece sobre exigências históricas de testes/suíte completa listadas abaixo. Checklist estático, validação proporcional, registro de limites e distinção entre build, execução e aceite continuam obrigatórios.

## EAF-140 — carga conjunta

Checklist prévio/revisão e tradeoffs em `joint-runtime-load.md`. Filtro acústico
barato antes do BTreeMap removeu custo medido sem cache novo; resultado de
transações preservado e oclusões legítimas contabilizadas.67Rust existentes/
12MCP47grupos/probe1000replay+PNG; sem novos testes por cada flag/contador.
Custos mantidos e deadline não garantido constam na matriz de runtime.

## EAF-127 — revisão do código implementado

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | Adequado: plan_local só no replanejamento, launch revalida trecho escolhido; falha mantém retry60 mesmo se destino muda. Testes estável/móvel conferem plans sem spam. |
| 2. Alocações | Adequado: LocalQueries cache8 Copy e máscaras/ordem64 no stack; Dijkstra arrays existentes, sem Scene clone. Rapier ainda aloca manifolds internamente; route/diagnóstico String mantêm custos anteriores. |
| 3. Dados | Adequado: GroundPlanning Copy u8/u16/bool opcional no State; nenhum JSON/reflexão no motor. Sem dependência/ABI/ECS novos. |
| 4. Repetição | Corrigido: cache transitório por par exato reutiliza direto/entrada/saída repetidos no plano; âncora física confirmada não dispara conexão zero. Cache estático por perfil do EAF-126 preservado. |
| 5. Limites | Adequado:3entradas/4saídas próximas,direto,8probes/2400passos + no máximo600 no lançamento;2consultas de contato extras por probe. Seleção truncada é declarada limited; não garante ótimo global ou completude. |
| 6. GPU/I/O | Adequado: nenhum upload/renderer/captura novo no tick; inspeção/JSON/replay sob demanda. Capturas offscreen iguais em prefixo/final. |
| 7. Locks | Adequado: mesmo dispatcher/fase sequencial, mundo fixo compartilhado, sem novo lock/worker/PRNG/reserva de dinâmicos. |
| 8. Invalidação | Adequado: edição/undo/reset/load recompilam; retirada apaga estado, template nasce com identidade/planos próprios. stop_for limpa diagnóstico; cast lock mantém connector e congela started_tick. |
| 9. Contrato |15testes de integração novos e5grupos MCP cobrem movimento/pursuit conhecido,curvas de terreno/teto/corpo/gap/steps,budget/parcial/casts,erro/undo/reset/templates/reuso/replay/prefixo/tamper/PNG. Gates gerais em validation.md. |
| 10. Custos mantidos | Arrays/cache também no caminho local legado (limitados); sondagens fixas com velocidade vertical inicial0, contato parcial/skin e manifolds Rapier. Preparação24×128×600 herdada. Sem profiling/FPS/MMO/navmesh/ótimo global alegados. |

## Conectores terrestres — avaliação antes de implementar

1. Sondar só ao replanejar/lançar, retry60 para destino estável; sem rede KCC inteira por tick.
2. Arrays64/caches8 no stack, mundo fixo/cache de perfis reutilizados; Rapier mantém manifolds internos.
3. Estado opcional Copy com contadores tipados, sem JSON por movimento/novo ABI/dependência.
4. Cache por pares no plano evita consultas repetidas; geometria por sessão, nós com skin e pés físicos distintos.
5. 3entradas+4saídas+direto,8probes/2400passos por plano +600 no lançamento; truncamento explícito, sem ótimo global quando limitado.
6. Sem GPU/I/O novos no tick, serializar inspeção/replay sob demanda.
7. Dispatcher sequencial, queries fixas sem impulso/PRNG/reserva; dinâmicos resolvidos na execução.
8. Edit/reset/undo/load recompilam; morte/revive/retirada/nascimento limpam estado, lock preserva conector e prazo.
9. Testar direto/entrada/saída/detour/corpo/limites/budget/ocultação/cast/lifecycle/replay/PNG e legado completo.
10. Contato parcial e custos internos Rapier mantidos; rotas truncadas podem perder solução. Sem profiling/FPS/escala/navmesh.

## EAF-126 — revisão do código implementado

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | Adequado: World/custos por perfil na criação; templates aquecem cache antes de spawn. Dijkstra existente lê custos, lançamento sonda somente seu link; não há rede inteira sondada por tick. |
| 2. Alocações | Corrigido: Cuboid normal/inset no stack; mundo fixo compartilhado, Arc de resultados por perfil e mapas por nascimento. Não copiar Scene por probe. Rapier ainda aloca manifolds internos, custo conhecido. |
| 3. Dados | Adequado: Profile Copy e Status/Report tipados; mesmo f32/índices rede64/128; NPC.speed e caixa/config determinam cache, ID/cor/vida não. Sem ABI/ECS/dependência novos. |
| 4. Repetição | Corrigido: cache de perfis/templates compartilhado comprovado com Arc::ptr_eq; query para player sem mundo de rede cria mundo temporário somente sob comando. Apoio legado permanece lazy, evitando um scan novo por tick em voadores. |
| 5. Limites | Mantido:24perfis×128links×600passos virtuais na preparação, mais2contatos por probe. Custo máximo de autoria pode ser alto; não extrapolar para MMO1000. Lançamento limitado a1link por ator, nenhum probe durante retry wait. |
| 6. GPU/I/O | Adequado: renderer/RenderFrame/Vertex/Globals inalterados; JSON/relatórios/persistência/captura sob demanda. Sem I/O no probe/motor. |
| 7. Locks | Adequado: mesmo dispatcher/passo sequencial; mundo de queries fixo e imutável no controle, não escreve física/PRNG. Não prevê nem reserva corpos dinâmicos. |
| 8. Invalidação | Adequado: reset/edição/undo/load refazem mundo/perfis; NPC retirado perde bindings, cache de definições permanece; novo indivíduo recebe estado zero. Morte/retorno preservam apenas geometria compilada imutável. |
| 9. Contrato |17integrações cobrem rampas/degraus/teto/tamanhos/desvio/gap, limite/erros/probe read-only, cache/spawn/reuso, patrol/casts/life/memória/budgetXYZ/dinâmico liberado e replay; gates completos em validation.md. |
| 10. Custos mantidos | Contato parcial KCC e grace3ticks de autostep explícitos, chegada exige contato. Probes começam com velocidade vertical0 e só fixos; reserva/multidões/local connectors/ground-air não implementados. Sem FPS/profiling/escala alegados. |

## Navegação terrestre com controlador — avaliação antes de implementar

1. Compilar links estáticos fora do controle por tick, por perfil de caixa/velocidade/controlador; Dijkstra reutiliza custos existentes. Revalidar somente link lançado, não toda a rede.
2. Mundo de queries fixas compartilhado na sessão, sem Scene clone/PhysicsPipeline por probe. Resultados Arc por perfil, arrays fixos na busca; Rapier mantém manifolds internos alocados. Até600ticks por probe.
3. Motivos/relatórios tipados, floats/bounds existentes, índices da rede64/128. Sem ECS/ABI/dependência/precisão novos.
4. Cache por perfil imutável inclui templates antes do spawn; nascimento/reuso aproveita relatórios. Edição/reset/load reconstrói mundo e cache. Não usar identidade do dono como chave de geometria.
5. No máximo24perfis (8NPC+16templates),128links,600passos por link; custo de preparação explícito, potencialmente alto em autoria. Lançamento faz no máximo um probe por ator e não é executado enquanto espera; sem conectores locais sondados por tick neste recorte.
6. GPU/RenderFrame inalterados; inspeção/JSON/captura sob demanda. Runtime guarda só estado de navegação já existente.
7. Simulação/dispatcher sequenciais; queries estáticas não reservam nem modificam corpos/velocidades/PRNG. Dinâmicos permanecem solver/timeout, sem previsão coletiva.
8. Mundo/cache ligado à Simulation; estado removido no despawn e zero no nascimento. Cast lock pausa trecho terrestre sem perder âncora no meio da rampa; relógio de percepção continua.
9. Testar execução versus probe, rotas alternativas/capacidades/teto/gaps/orçamento, ausência de teleporte, lifecycle/erro/undo/reset/replay/PNG e legado completo headless.
10. Política de contato do controlador explícita e distinta de apoio integral do spawn/legado; recusar voo/salto/planeio e conectores locais neste opt-in. Sem navmesh/FPS/escala ou indexação especulativa.

## EAF-124 — revisão do código implementado

## EAF-104 — revisão final da união de transações/controladores/ilhas

Checklist prévio em editor-integration.md. (1) Mesmas fases, input por ator no
loop existente; (2) Writes16/2 e Contributions indexado da origem, buffers de
motor20 e controles reutilizados; (3) catálogo liga slot a entidade sem confundir
InstanceId; (4) intenção por Player/perfil/lock alimenta motor uma vez;
(5) pools/tetos mantidos, nenhum claim1000; (6) nenhum shader/readback/frame-I/O
novo, congelar domínio flowers custa608×12B na autoria; (7) sem worker/lock/rede;
(8) clear/refresh/serde reindexam buffs, lifecycle reinicia motor e neutraliza
controles, gates históricos e três estados de replay mantidos;
(9) 1220Rust/115alvos,Clippy,108MCP/461grupos e probes cruzados/UI/escultura/
conteúdo aprovados no build960a1c4f79326bc1; (10) Rapier casts/manifolds, scans
limitados e cópias de geometria por pedido permanecem custos explícitos. Não há
medição/alegação de FPS. Release único18298240B/cache2,2GiB, sem históricos.

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | Adequado: compile de ângulos/política em insert_body, movimento somente para controladores ativos por tick; omissão retorna antes de consultar BVH. |
| 2. Alocações | Adequado: maps/IDs por nascimento, scratch Vec20 reservado só ao primeiro controlador e reutilizado. Clone de SharedShape no push incrementa referência, não duplica malha. Rapier move_shape/impulsos mantêm Vec interno de manifolds; não declarar zero heap. |
| 3. Dados | Adequado: Config/State Copy tipados, entity_index compilado e Reference generacional; intenção XZ própria porque set_linvel ignora cinemático de posição. Sem JSON no movimento. |
| 4. Repetição | Mantido com motivo: AABBs de corpos correntes publicados antes das consultas para incluir nascimento/primeiro tick sem passo extra; física atualiza broad phase em sua fase. Dupla publicação opt-in limitada, sem índice paralelo ou cache de invalidação complexo. |
| 5. Crescimento | Mantido:9atores,128corpos com NPC,20iterações/callbacks Rapier por movimento; máximo20colisões no scratch. Autostep usa casts extras; não satisfaz por si o alvo MMO1000. |
| 6. GPU/I/O | Não se aplica ao tick: nenhum recurso/render/upload novo, persistência/captura/inspect sob demanda. Mapa configs na inspeção é transitório e limitado. |
| 7. Locks | Adequado: mesmo dispatcher e passo sequencial, sem lock/worker novos; consultas de atores usam posições pré-física. Push é ordenado pelo ID no mapa. |
| 8. Invalidação | Adequado: insert_body inicializa estado, remove_entity apaga; morte desativa/converte corpo, revive reinicia sem teleporte; reset/undo/edição/load recompilam. Sem histórico por frame novo além do replay já existente. |
| 9. Contrato | Testes cobrem primeira consulta sem passo, limites/erros, rampas/degraus/teto/queda/parede/push, NPC e casts/lifecycle/reuso/contatos dinâmicos; replay compara motor. Gates completos registrados em validation.md ao concluir. |
| 10. Custos mantidos | Contato parcial físico é explícito e não relaxa apoio integral do spawn/navegação. Sem rotas inclinadas/knockback entrante/plataformas/multidões; motores aéreos recusados neste opt-in. Sem profiling/FPS/escala alegados. |

## EAF-124 — avaliação antes de implementar

1. Controlador opt-in por ator/tick fixo; configurar ângulos uma vez ao instalar, sem JSON no tick.
2. Estado por ator e scratch de colisões reutilizado com limite20 (limite interno de iterações Rapier); sem clones de cena por movimento. Rapier mantém alocações internas de manifolds.
3. Configuração tipada Copy, estado generacional e enums; motor legado por omissão. Sem ECS/ABI/dependência novos.
4. Publicar AABBs correntes antes da consulta somente quando houver controlador; API set_aabb preserva detecção posterior do pipeline. Consultas usam snapshot anterior à física.
5. Até9atores/128corpos com NPC; até20colisões por movimento. Autostep acrescenta casts. Não extrapolar para1000jogadores; requisito MMO requer budgets/medição próprios.
6. Nenhuma alteração de GPU ou persistência por tick; inspeção/captura/replay sob demanda.
7. Mesmo dispatcher/lock; física sequencial, sem concorrência nova.
8. Estado instalado/removido junto ao corpo, morte troca para dinâmico e revive reinicia motor sem teleporte; edição/reset/load refazem estado.
9. Testar geometria/gravidade/teto/push, casts/ciclo de vida/reuso, erros/undo/replay/prefixo e regressão completa headless. Planejador e apoio de spawn permanecem conservadores.
10. Aceitar contatos parciais do controlador como política física explícita, distinta do apoio integral do planejador/spawn. Sem knockback entrante, plataformas móveis ou garantias de multidões. Custo interno Rapier mantido, sem alegar FPS.

# Revisão de performance por leitura de código

## EAF-129 — projeção de landmarks

Decisão/checklist anterior em [landmark-projection.md](landmark-projection.md).
1. `landmark_projection::inspect` executa apenas sob comando; tick/frame sem
nova consulta. 2. Uma Projection/matriz por pedido e Vec de saída com capacidade
exata, até128; sem clone de Scene/mesh. 3. Entradas tipadas e coordenadas/IDs
validados; JSON na fronteira Engine existente. 4. `project_clipped` compartilha
a mesma transformação e clipping com overlays, sem segunda matriz/convenção;
XY só é classificado no diagnóstico. 5. Viewport16384 e128pontos, IDs64bytes;
duplicatas verificadas por até8128comparações, custo quadrático limitado aceito
para evitar outro container/alocação em consulta pequena. 6. Sem GPU/upload/
readback/I/O na consulta; capturas somente no harness. 7. Sem worker/lock/rede/
dependência/PRNG novos. 8. Sem cache ou persistência novos, guarda de revisão
comum e histórico intacto. 9. Rust próprio cobre perspectiva/reciprocidade,
clipping/resíduos/exclusões/limites; MCP cobre atomicidade, pixels, histórico,
save/load/reset/replay, além da regressão completa. 10. f32 da projeção e
relatório/JSON síncronos mantidos; RMSE f64 só sobre pixels existentes. Sem
calibração/oclusão/aceite3D/FPS alegados. Ensaios não alteram a casa canônica,
não aumentam caps e usam o único binário corrente, sem janela.

## EAF-104 — superfície visível, retângulo e ações de área (29/10)

Checklist anterior/decisão em editor-workflows.md, seção Seleção e visibilidade.
1. BVH só é preparado quando overlays de malha sem raio X estão ativos ou há
   pedido de região; nada no tick da simulação.
2. Um BVH corrente na UI por revisão/tick; frame e buffers de construção são
   transitórios. Comando de retângulo prepara índice temporário próprio por
   pedido. Não há clone Scene nem retenção de versões históricas; duplicação
   transitória UI/consulta e soma de triângulos posed são custos mantidos.
3. Referência de malha, instância e pixels locais são domínios separados.
4. UI/MCP reutilizam Projection/Visibility/union e o TriMesh do Parry existente;
   caixas/cunhas vêm do gerador exato do renderer, near plane do picking.
5. Tetos de meshes/draw/atores/painéis preservados, viewport limitada16384;
   linhas têm até2048intervalos amostrados por aresta, runs visíveis coalescidos.
6. Sem novo shader/pass/bufferGPU/readback/dependência. Guias são egui; apenas
   capturas explícitas dos probes fazem leitura GPU.
7. Execução serial, sem locks/worker/estado de jogo ou persistência de seleção.
8. Erros de referência/retângulo não mutam; câmera/resize/foco/Esc/revisão cancelam
   gesto; seleção limpa não ressuscita no inspetor. Câmara não reconstrói BVH.
9. Frente/fundo, oclusores estrangeiros e cunha real, domínio, Shift/vazio/união,
   eventos de mouse/cancelamentos, ícones/layouts e MCP/render/replay exercitados.
10. Construção BVH síncrona, poses/projeções e raycasts por ponto/trecho visível
    permanecem custos limitados explícitos. Invalidação por material/luz autoral
    também reconstrói o índice; não há promessa de zero heap, FPS ou ganho medido.

Gates d67af2172b203466:1184Rust/112alvos,Clippy-Dwarnings,105MCP/449grupos,
probe UI4larguras e7receitas aprovados.4recortes517500pixels RGB exatos.
Release18095552B/cache~2,1GiB; efeitos e limites em editor-integration.md.

## EAF-104 — controles independentes, Sweep por curva e chanfros

Checklist anterior completo em editor-integration.md. Inputs em frame reutilizado,
mapa por handle sob comando e lifecycle por ator; sem clone de Scene no tick.
Preflight de mesh antes de expandir/clonar, listas de spans limitadas16, gerações
por comandos compartilhados. Mesmos tetos/draw/Vertex64/Globals192/Material1232;
nenhum shader, recursoGPU, dependência, lock ou worker novo nesta união.
Snapshot/replay mantêm todos os contratos (controles+meshes), gate77 e históricos
75/76/etc.; Scene81/replay85 fora de layout3. Consultas/handles expirados são
atômicos. Testes combinados usam processos isolados, nunca o app do usuário.
Custos de scans/log/duração/Plan globais continuam transitórios, sem claim1000
atores/rede/FPS. Autoria síncrona e loops limitados de chanfro mantidos.
Composição da casa respeita32596v/172livres/16slots; materiais/rigs/fontes do
restante preservados. Mola[1,9] é variante de estudo, não mutação silenciosa.
Evidências finais em validation.md/editor-integration.md.


Gates da união862798f32d7d2aca:1163Rust/111alvos,Clippy-Dwarnings,
103MCP/434grupos e probe UI4larguras aprovados.4recortes900×575pixels legados
exatos e7receitas/estudos passaram; release17.943.104B/cache~2,1GiB.
Falhas e correções sem relaxar guardas em editor-integration.md.

## EAF-104 — união de cunha, ambiente e normais

Checklist anterior em [editor-integration.md](editor-integration.md).
1.Consulta/collision/kernel comum; bake por conteúdo, query de normais explícita.
2.Sem clone de mundo; scratch limitado e cache único de probe, sem histórico.
3.Scene80/replay84 unificados; gates75/76 e anteriores, layout3/manifesto1.
4.Normais extraídas sem alterar aritmética; sem cálculo duplicado por consumidor.
5.Tetos/pools/validações mantidos; fixture material separada da composição atual.
6.Globals192/Vertex64/Material1232;148984B lógicos de ambiente. Cunha36v com
material neutro e contorno compartilhado; override Light só no RenderFrame.
7.Sem dependência/worker/lock novo; pesquisas gráficas seguem opt-in.
8.Light::validate inclui ambiente/intensidade também nas preferências do editor;
formas antigas mantêm defaults e erros preservam autoria/layout/simulação.
9.Rust/Clippy completos, MCP completo, probe UI e receitas nativas são os gates.
10.Bake/scratch/staging síncronos mantidos; sem otimização especulativa/FPS.
Custo da correção de contato só na variante do estudo:72v/148t, sem mudar fonte.
Resultados finais e falhas corrigidas em validation.md/editor-integration.md.


## EAF-104 — união de apoio, Sweep e materiais

Pré/pós-checklist e pontas em [editor-integration.md](editor-integration.md).
Nenhum algoritmo extra de integração: apoio/ocupação compartilhados, preflight
Sweep preservado, segunda camada+80B/slot e metal em high.w, igualdade/cache e
gates históricos66/73/74 mantidos. Scene78/replay82 fora de layout3. Estudo MMO
não muda runtime. Script de prova combina contratos sem recursos do app aberto.
A casa de calibração material fica separada da revisão estrutural, com hashes
originais preservados. Custo síncrono e fragmento opt-in explicitados nas frentes,
sem benchmark/FPS/escala alegados; nenhuma dependência/pipeline experimental novo.

1.142Rust/109alvos,Clippy-Dwarnings/fmt/diff/py_compile,100MCP/420grupos e probe UI
aprovados no build5b466fd1c0faa1bd. Falha inicial de teste baseada em versão
corrente foi corrigida para gate histórico, com recorte19repetido; runtime não
mudou. Legado da viewport exato em4regiões e provas materiais/hashs3+3exatos.
Release17.807.856B/cache~2,1GiB, saídas correntes e sessões preservadas.


## EAF-104 — refinamento visual de formulários e organização

Checklist anterior/posterior, auditoria e reprodução em
[editor-ui.md](editor-ui.md). Tokens/helpers egui compartilhados; agrupamento e
campos adaptativos somente na composição. Menu de projeto reutiliza comandos;
popover de iluminação usa formulário, preservando o consumo de entrada anterior.
O seletor de cor só publica no draft quando há interação, evitando mudança RGB
por inspeção. Sem novo contrato, clone de mundo/geometria, pass/buffer/shader,
thread, lock, I/O por frame, dependência ou fonte. Custos de layout/string/lista
linear limitada e preview síncrono anteriores mantidos; probe GPU/readbacks só
na verificação isolada. Não há claim de FPS ou aceite humano por captura.

Gates do build adea604e9a6ffdd4: 1.109 Rust/106 alvos, Clippy -D warnings,
fmt/diff, probe UI em quatro larguras e MCP completo 96/397 aprovados.
Release corrente 17.705.216 bytes, cache ~2 GiB, sem cópias históricas. Sessões e
cenas pessoais preservadas, nova avaliação humana da apresentação pendente.


## EAF-110 — correção de parâmetros e legibilidade

Checklist anterior/posterior e alternativas em [editor-sculpt.md](editor-sculpt.md).
Configuração tipada pequena por evento, força efetiva sem fator oculto e luz
substituída somente no RenderFrame do editor. Light::validate compartilhado,
layout3 migra1/2; autoria/replay/tick permanecem separados. Nenhuma geometria ou
pipeline novo por controle. Passe de sombras existente com mapa1024 é agora
padrão da viewport: custo GPU assumido para leitura do relevo, sem claim de FPS.
Bias0.002 escolhido após inspeção de acne nas capturas. Popups consomem entrada
antes de escultura/seleção/câmera, sem clone/tick/histórico adicional. Normais e
raycast continuam síncronos e limitados como no incremento anterior.

Gates:1108Rust/105alvos,Clippy-Dwarnings/fmt/diff/sintaxePython e96MCP/397grupos
completos aprovados no build9c80622307cd2659. Entrada nativa, alternância de
sombras, consumo de clique e undo/redo exato conferidos; aceite humano aberto.
Release corrente17.655.504B/cache~2GiB; wrappers por symlink ativos preservados.

## EAF-110 — pincéis de escultura

Contrato, alternativas e checklist de dez pontos antes do código em
[editor-sculpt.md](editor-sculpt.md). Prévia do gesto separada da autoria;
publicação única por traço, kernel compartilhado e limites existentes.
Revisão posterior no mesmo documento: raycast cacheado por cursor/revisão/tick,
poses uma vez por consulta, COW na geometria tocada e até128amostras por gesto.
Normais/validação/upload continuam síncronos; sem readback rotineiro, pipeline,
dependência ou promessa de FPS. RenderFrame recebe diretamente a prévia, sem
reconstrução duplicada do documento original. Confirmação em lote serial mantém
revisão inicial apresentada e acompanha apenas sucessos desse lote.


Gates do build5fcf3f9b1b05dc9c:1104Rust/105alvos,Clippy-Dwarnings/fmt/diff e
96MCP/395grupos completos; entrada nativa dos três pincéis e undo/redo exato
conferidos, aceite humano separado. Um release corrente17.554.944B/cache~2GiB.

## EAF-109 — seleção estruturada de pastas, revisão anterior

Pedido: escolher o local sem conhecer um caminho. Navegador integrado com Locais,
breadcrumbs, pasta anterior, busca, ocultos, paginação e destino explícito. Reuso
adotado em vez de nova dependência de diálogo nativo; nenhuma decisão de formato.
1.I/O apenas ao abrir/navegar/atualizar/buscar;2.cache de até257 nomes por página,
sem documento/mesh clonado;3.PathBuf/OsString preservam nomes;4.mesmo formulário e
Create/OpenProject, nenhum dispatcher paralelo;5.página256 e lista ordenada por
seleção limitada, sem ocultar silenciosamente diretórios além do teto;
6.sem alteração GPU/readback;7.sem thread/lock/dependência nova;
8.cache renovado por evento, erros desabilitam confirmar;9.testar pasta com mais
arquivos que o limite, páginas, busca, ocultos, erro e navegação UI;
10.enumeração/metadados síncronos ao navegar mantidos, não por frame, sem FPS.


Revisão posterior: directory_page seleciona os257menores nomes acima do cursor
com BTreeSet, incluindo pastas encontradas depois de centenas de arquivos; cache
visível256 e histórico só de cursores, renovados ao mudar de pasta/busca/ocultos.
Locais e breadcrumbs são preparados em eventos, read_dir fora de redraw. Nenhuma
cena, captura, comando, cache de geometria, dependência ou formato foi alterado.
Custos mantidos: scan completo síncrono da pasta por navegação/página/busca e
metadados para seguir links de diretórios; não é busca recursiva/indexada.
Conferência nativa detectou crescimento vertical de separadores em horizontais;
foram retirados, com regressão que exige destino/Criar/Cancelar dentro da tela.
Teste de cliques cobre Local→subpasta→Subir→breadcrumb sem digitar caminho.


Gates finais do build b540b542eab8bf06:1096testes Rust/104alvos release/all-targets,
Clippy-Dwarnings/fmt/diff e95cenários MCP/392grupos headless aprovados, sem falhas
ou testes ignorados. Mesmo fonte/hash conferido; manifesto1/layout2/Scene75/
replay79 mantidos. App instalado aponta ao único release corrente17.387.568bytes,
cache exclusivo~2,0GiB. Nova instância instalada aberta após o teste isolado;
sessões anteriores não controladas. Sem push, cópia de binário ou instalador.
Logs/auditoria editor-folder-* e relatório MCP corrente. Aceite humano separado.

## Projetos e cenas — revisão anterior

Checklist de dez pontos e alternativas antes do código em [projects.md](projects.md).
Pasta/manifesto separado, ações explícitas de I/O e troca de raiz; sem trabalho
por tick/frame, dependência/GPU nova ou alteração de Scene/Replay.

Depois: Service troca Engine somente após validar, revisions de autoria/sessão
separadas; manifesto até256cenas e pasta até256entradas, sem scan em redraw.
Layout2 migra1, proteção de dirty no dispatcher e no fechamento, campos de pasta
persistentes e lote UI interrompido se salvar falhar. Fontes/caches/renderers
existentes preservados; nenhum shader, buffer, dependência ou worker novo.
Custos síncronos/dirty conservador mantidos e documentados em projects.md.
1092testes/104alvos, Clippy/fmt e95MCP/392grupos; fluxo nativo exercitado.
Um executável corrente17.371.216bytes, cache release~2,0GiB; wrapper transitório
sem cópia de binário removido após teste, app instalado samefile. Sem FPS alegado.

## EAF-104 — integração do laboratório de tangentes compactas, revisão anterior

Ponta8a5e664, destino2d13274 limpo. Checklist proporcional:
1–2.codec/quantização offline e bindings persistentes somente no laboratório;
3.u32porface+escala com formato/hashes, sem expansão CPU porframe;
4.mapas/fonte comuns, cada handle conserva tabela bruta/compacta;
5.domínio heightfield/drawúnico e budgets funcionais preservados;
6.leitura/IGP1/primitive_index só opt-in, nenhum renderer funcional novo;
7.sem thread/lock/dependência nova;8.src/Cargo.lock/build.rs e Scene75/replay79
inalterados;9.Rust release/all-targets/Clippy e recorte MCP headless de
editor/lab/render/replay/seleção;10.custo/erro de quantização e falhas da
referência documentados, sem promoção ou benchmark repetido para alegar FPS.
A integração não acrescenta ferramentas UI nem retoma reconstrução cancelada.

Revisão posterior: diff de src/Cargo.lock/build.rs vazio; build funcional
c2d5bb39a0526e01/Scene75/replay79 preservado. Codec, quantização/projeção e
bindings das variantes residem apenas em examples/rich_relief_probe. Handles
conservam sua tabela, sem dependência ou feature nova do editor. Custo de
buffers bruto+compacto e erro de quantização mantidos no estudo; nenhum ganho
robusto ou correção da referência instável inferidos.1087testes/103alvos,
Clippy/fmt/diff/sintaxePython/JS e5cenários MCP/30grupos aprovados. Recorte
proporcional justificado por ausência de mudança funcional; suíte completa
94/388 anterior continua sendo do mesmo fonte/build funcional.

## EAF-104 — templates ponderados e tangentes, revisão anterior

Checklist em editor-integration.md reaplicado às pontas6b1ab5e/d0da3b0/7696d8d:
1.escolha só em novo nascimento pendente, sem reroll por retentativa;
2.bindings/tabelas compiladas e Single sem heap;3.índices tipados por membro,
streams template/posição independentes;4.reusar spawn/validação/replay;
5.16candidatos/pesos e budgets existentes, sem expansão do pool;
6.IGT1/primitive_index só no harness opt-in, nenhum novo recurso no renderer;
7.sem lock/thread/dependência nova;8.Scene75/replay79 unificados conservando
gate65 ponderado e demais guardas históricas;9.Rust release/all-targets,
Clippy, regressão MCP completa e integração editor/população ponderada;
10.escolha inviável pode bloquear produção, autoria/captura síncronas e custo
experimental explícitos. Sem promover pesquisa, inferir fidelidade do personagem
ou ganho de FPS. Receitas são dados/cliente; não alterar runtime para o ensaio.

Revisão posterior do código integrado: Selection::Single guarda Binding sem heap,
Weighted consulta no máximo16cumulativos por nascimento. Retentativas usam
pending; o stream de posição não recebe draws de template. Atribuição template
nos membros/recibos e referências de alvo preservadas. Engine/replay conservam
expected_meshes e expected_generators, com Scene75/replay79 e gate65 explícito.
Auditoria mantém99Command/12SceneEdit/20MeshEdit/6PlantEdit e94cenários. Custos
mantidos: compilação de fontes/inspeção síncronas, escolhido inviável bloqueia
produção com diagnóstico, sem fallback silencioso. IGT1/primitive_index apenas
no harness, sem novos buffers/feature no renderer funcional. Cinco testes do
cliente genérico de receitas passam; não aprovam objeto nem reabrem EAF-107.
Gates finais em validation.md; nenhum benchmark científico ou ganhoFPS alegado.

## EAF-104 — atualização horária com painéis sólidos, revisão anterior

Checklist anterior em editor-integration.md reaplicado às pontas87273b8/d756fb1:
1.execução de painel apenas em autoria/preview;2.grade limitada a2312slots,
Arc/staging existentes;3.chaves numéricas, IDs e vínculos distintos;
4.reutilizar dispatcher/PutAsset/validação, sem duplicação no frame;
5.16aberturas e budgets de geometria conservados;6.renderer funcional intacto,
footprint analítico só no harness opt-in;7.sem thread/lock/dependência nova;
8.substituição integral/proteção de fonte e formatos74/78 preservados;
9.Rust release/all-targets, Clippy e regressão MCP completa com mesh_panels;
10.custos síncronos/grade redundante e aproximação experimental explícitos.
Nenhum experimento promovido, ganho de FPS ou aceite visual inferido.

Revisão posterior: Panel encaminha o resultado para PutAsset; renderer,
frame e cache funcionais não receberam trabalho novo. A guarda de plantas
inclui Panel, sem sobrescrever fonte vinculada. Máscara de conexões e edições
de materiais continuam presentes. Auditoria das cinco pontas conserva
99Command/12SceneEdit/20MeshEdit/6PlantEdit,93cenários. Teste de integração
acrescenta captura explícita com painel, plantas e geradores;45.486pixels na
viewport mudam e undo restaura RGB exato, sem usar cabeçalhos como evidência.
Custos mantidos: montagem síncrona/preview, grade mais subdividida que solução
ótima e substituição integral sem rig/detalhes antigos. Jacobiano/amostra mip0
experimentais permanecem fora do renderer funcional. Gates finais e limites
em validation.md; nenhum benchmark científico reexecutado nesta integração.

## EAF-104 — atualização horária com conexões de plantas

Checklist anterior em [editor-integration.md](editor-integration.md); contrato recebido em plant-connections.md. União/CG/recomposição continuam limitadas à autoria/validação/consulta, sem solver no tick/frame. Sources Arc, canais/rig e limites preservados; Frame compara a máscara de até8 instâncias consumidas e só reconstrói seus IDs quando muda. O compositor do editor continua chamando Engine::update_render_frame, usando a mesma filtragem dos demais renders. Nenhuma thread, dependência, layout de GPU ou readback rotineiro novo. Custo síncrono/revalidação duplicada mantidos explicitamente. Scene74/replay78 unificam a publicação conservando gates históricos; testes de versão corrente ajustados sem remover negativos. Auditoria conserva99Command/12SceneEdit/92cenários. Gates finais em validation.md; estudos de textura permanecem instrumentos explícitos, sem alegação de ganho de FPS ou aceite visual humano.

## EAF-104 — integração das branches

Checklist anterior em [editor-integration.md](editor-integration.md). Revisão do conjunto:

1. Adequado: mesmos redraws/ticks; merges não acrescentam fase de simulação ou polling MCP por frame.
2. Adequado: GeometryArc/Frame de malhas e textura/contexto do editor preservados; `Graphics::build` reusa `Engine::update_render_frame`, sem clone de Scene por frame.
3. Adequado: comandos/SceneEdit continuam tipados e compartilhados; IDs de assets, layout e InstanceId não foram unificados. Região usa o mesmo spawn e seus limites.
4. Corrigido: declaração duplicada de mesh_surface removida; enums, dispatchers e runner conservam as capacidades de todas as pontas, auditadas em coverage.json.
5. Adequado: budgets existentes de geometria, população e áreas mantidos. Runner91cenários únicos; três verificadores científicos offline continuam fora do protocolo MCP.
6. Adequado: compositor usa o renderer integrado e seus materiais/sombras; readback somente na captura. Experimentos recebidos continuam explícitos, sem ativação no jogo.
7. Adequado: nenhum lock/thread novo; desenho da janela permanece fora do lock do Service. Consultas e persistência síncronas continuam custos de ferramentas.
8. Corrigido: Scene73/replay77 reúnem os contratos sem remover guardas históricas; revisões de editor e cena separadas. Testes de versão corrente ajustados, fixtures de rejeição preservadas.
9. Verificação: cenário combinado inclui editor+malha/planta real+geração+replay; testes conjuntos anteriores conservam adulteração independente de meshes e geradores. Gates finais e build auditado em validation.md.
10. Mantido com motivo: captura/consultas de autoria síncronas e custo das ferramentas geométricas existentes. Benchmarks científicos não são reexecutados para alegar desempenho; nenhum ganho de FPS, escala ou aceite humano inferido.

## EAF-104 — app local para Finder

Antes: empacotamento eventual, fora do runtime; reaproveitar o release existente, sem build adicional, cópia de binário, thread, lock ou mudança de cena/MCP. Instalar diretamente no destino corrente, distinguir editor/demo e recusar bundle de outro identificador. Raiz explícita e argumentos escapados para execução independente do cwd do Finder. Verificar metadados, launch e ligação ao binário entre volumes; sem repetir regressão de kernel inalterado.

Depois: `scripts/package_macos.py --editor --install` escreve somente launcher/plist e liga o executável corrente; hardlink no mesmo volume, symlink em EXDEV. Substituição atômica de cada arquivo, um app no destino, nenhum staging ou histórico retido. Custo mantido: app local depende do checkout e do volume do release; não é distribuição autônoma. Sem trabalho novo por frame/tick, GPU ou ganho de FPS alegado.

## EAF-104 — estrutura do editor nativo

Checklist antes/depois, alternativas e evidências de código em [editor.md](editor.md). UI de sessão tipada fora da simulação/Scene/Replay; contexto e textura GPU reutilizados. Captura/replay de cena preservados. Custos conscientemente mantidos: widgets imediatos, scans limitados, cópia do layout por ação para atomicidade e captura/persistência síncronas. Sem ganho de FPS alegado. Resultados automatizados e fronteira de aceite em validation.md.

## Personagem orgânico — ensaio EAF-107

Checklist anterior e respostas em [fox-character-experiment.md](fox-character-experiment.md).
Cliente reutilizável de fontes nativas+dados, sem Rust/tick/frame/schema novo.
Cache de118previews para329partes,9726v18096t/12assets; cascas e assentamento
de adornos explicitamente no cliente. Publicação com preflight/preview/revisão,
limites mantidos;5testes e auditoria de329faixas da cena nativa/carga/undo/replay
estático aprovados.18pares/seis vistas abertos; fidelidade REPROVADA. Custos
de JSON/cópias, consultas triangulares por vértice e avaliação visual mantidos.
Nenhum ganho de FPS ou geração automática de personagem alegado.

## Painéis nativos com vãos

Checklist prévio em [mesh-panels.md](mesh-panels.md): autoria apenas, grade
limitada34×34×2, índices numéricos e16aberturas; sem dependência/tick/GPU novo.
Resultados:871 testes Rust/all-targets, Clippy/fmt e84cenários MCP/347checks.
Painel com porta/janela60v120t, volume/raios/orientação conferidos;
carga/replay estático com pixels exatos. Grade mais subdividida que o ótimo,
cópias de staging/JSON e autoria síncrona mantidas conscientemente. Sem ganho
de FPS, novo formato, fonte paramétrica persistente ou fechamento de edifício.

## E02 — nova autoria volumétrica

Checklist anterior e respostas finais em [e02-volume.md](e02-volume.md).
Autoria Python/MCP apenas sob pedido, geometria em coordenadas do objeto,
cooking1e-6/f32 e verificação da malha nativa. Sólidos internos/externos com
jambas, pisos, cobertura e passagem; sem solver de câmera por peça. Centros
dispensáveis removidos de faces convexas planas e pisos sem grade redundante,
preservando bordas compartilhadas; colunas instanciam uma fonte nativa96v.
16295v/26260t, mesmos budgets, duas recusas pré-publicação. Nenhuma mudança
Rust/tick/frame/renderer/cache/schema ou dependência instalada.13checks,
22pares e seis vistas; JSON e capturas explícitos permanecem custos de autoria.
Não afirmar ganho de FPS, geração automática ou fidelidade aceita.

## Reconstrução — preflight de corpo 3D no cliente

Checklist anterior e respostas finais em
[reconstruction-structure.md](reconstruction-structure.md). Auditor Python
stdlib apenas sob comando, mapas de posições/arestas e componentes por asset,
limites finitos de entrada, origem local para volume. Sem GPU/tick/frame,
dependência nova, engine/processo ou alteração de Scene/replay/schema.
18 testes e diagnóstico da cena real; seams exatos não são gaps. O publicador
E02 recusa ausência de corpo antes de detalhes/MCP e revalida geometria final.
Custos mantidos: parse/arrays/JSON e coincidência conservadora, sem inferência
semântica, autointerseção ou relação entre assets. Nenhum ganho de FPS alegado.

## EAF-103 — lista de construção antes da autoria

Antes:1.execução apenas documental/extração explícita, sem tick/frame;
2–3.Pillow/JSON no cliente, uma fonte1402×1122 e40recortes finitos;
4.lista canônica JSON gera Markdown, sem duplicar descrições manualmente;
5.caminhos estáveis e conjunto finito, sem histórico de builds;
6.sem GPU/render/download/modelo novo;7.sem processo engine/thread/lock;
8.cenas/formatos/revisões intactos;9.validar skill/links/hash/coordenadas,
IDs/revisitações/estados e aplicar a análise real;10.completude semântica
continua julgamento visual, não resultado do validador de campos.

Depois:248tarefas/21conjuntos,40recortes inspecionados,10revisitações;
12recortes de varredura cobrem todos os pixels sem afirmar percepção perfeita.
Sub-recortes contidos nos pais; contexto C14 ampliado para incluir a trepadeira
inteira. Skill/AST/157links/IDs/estados/hash aprovados. Cópias PNG/JSON e
renderização de Markdown no cliente mantidas conscientemente; nenhum Rust,
build, engine, fonte paramétrica ou render novo. Evidência em
[lista consolidada](e02-construction-list.md) e validation.md. Sem ganho de
performance/fidelidade ou cobertura semântica automática alegado.

## EAF-102 — desenhos de fontes paramétricas E02

Revisão anterior e resultado em
[e02-native-blueprints.md](e02-native-blueprints.md). Autoria pública MCP,
16fontes/31instâncias/2278v/2620t; nenhuma malha direta ou mudança de limites.
Projeção2D no cliente a partir de inspect_mesh, fontes por inspect_plant,
cópias/PNG sob demanda e caminhos estáveis. Repetição/carga com pixels iguais;
checagens de fontes/índices/limites/AST/desenhos descritas em validation.md.
Trajetória inicial recusada, parametrização final por seções paralelas deslocadas
explicita o tradeoff; sem alterar kernel/solver para acomodar a forma. Pranchas
expõem estudo estrutural incompleto, sem ganho de fidelidade/performance alegado.

## EAF-102 — auditoria de fidelidade e threshold

Checklist anterior e respostas em
[e02-fidelity-investigation.md](e02-fidelity-investigation.md). Auditoria CPU
sob comando com três fixtures1402×1122 e comparação MCP em processo próprio;
nenhum tick/frame, render, kernel, formato ou limite alterado. Caminhos estáveis,
Pillow já disponível, hashes e MAE independente conferidos. Duas perdas locais
importantes passam por SSIM global0.98; isso valida a contraprova, não fidelidade
3D. Custo de cópias/PNG/JSON no cliente mantido conscientemente. Sem profiling,
ganho de performance ou regressão Rust nova; resultados em validation.md.

## EAF-102 — E02 com recortes e revisão local

Checklist anterior, escolhas e respostas finais em
[e02-reconstruction.md](e02-reconstruction.md). Somente cliente de autoria,
evidências e cena nova; nenhum Rust/tick/renderer/MCP ou build alterado.
Primeira expansão35874v descartada antes de publicar; coluna nativa80v/5instâncias,
galeria compartilhada6vezes, weld e tesselação limitada deixam16218v/18512t
armazenados,16assets/25instâncias e166v livres. Mesmos budgets. Recortes18 e
comparação16+2 sob comando, caminhos correntes. Arrays/cópias/JSON/I/O no cliente
continuam custos explícitos; engenharia de forma não é ganho de FPS. Captura,
undo, load, replay estático e processo novo preservam pixels; fidelidade segue
insuficiente, com SSIM global pior após a revisão. UI da página sem teste visual
por bloqueio file://; sintaxe/dados/recursos são checados estaticamente.

## EAF-101 — análise aprofundada por recortes, revisão anterior

Incremento documental da skill: análise global → fichas visuais → inspeção
aprofundada → comparação local e global. Sem alterar receita, cena, comparador,
renderer ou schema. 1–5: nenhum tick, buffer, tipo ou budget novo; fichas podem
compartilhar recortes e só aprofundam regiões com evidência útil. 6: orientar
recortes de pixels originais, caminhos correntes e captura explícita; não gerar
imagens ou executar renders nesta unidade. 7–8: nenhum lock/cache/estado novo.
9: validar skill, YAML, links, contrato e diff. 10: leitura visual continua a
cargo da IA; documentação não implementa recorte MCP nem avaliação semântica.

Após: somente Markdown/YAML; nenhuma dependência ou execução por tick/frame.
Referência original intacta por SHA256; skill, YAML,12links e diff validados.
Recortes podem ser compartilhados e o refinamento encerra por suficiência ou
limite de evidência. Custos mantidos no procedimento futuro: anotação, I/O de
recortes/capturas e leitura da IA sob demanda; nenhuma melhora de fidelidade ou
performance alegada nesta revisão documental. Sem novo build ou regressão Rust.

## EAF-101 — reconstrução pela interface pública

Checklist anterior e respostas posteriores em
[house-skill-experiment.md](house-skill-experiment.md). Cliente de autoria,
nenhuma alteração no kernel/tick/renderer. Motivos instanciados substituem
duplicação de vértices; coluna por seções lineares160v em vez de2896v. Cena final
16090v/21818t armazenados,16assets/32instâncias e32272t desenhados; limites
inalterados. 294v globais livres, assets/instâncias saturados; agrupamento de
subpartes permanece um custo de autoria explícito. Recusa atômica, undo/reload/
replay/capturas exatas por MCP. Sem profiling ou ganho de FPS alegado.

## EAF-100 — guia de reconstrução, avaliação anterior

Recorte documental: skill local ao projeto, contrato de análise e leitura da
casa; rótulo do ensaio existente será corrigido. 1.Frequência: só autoria/leitura.
2.Alocações e 3.tipos: nenhum estado Rust novo. 4.Repetição: uma fonte de análise,
referenciada pelo guia. 5.Limites: sem histórico ou capturas novas. 6.GPU/I/O:
mesma imagem original e mesmos outputs, sem novo render/build. 7.Sincronização:
sem threads/locks. 8.Invalidação: guia não altera Scene/replay/cache.
9.Validar frontmatter, links, regiões na imagem e texto gerado da demonstração;
não executar regressão Rust para documentação/rótulos. 10.Custo mantido: análise
semântica e revisão continuam humanas/da IA; skill não é validador geométrico.

### EAF-100 — respostas após a entrega

1–5.Adequado: apenas Markdown/YAML e rótulos constantes no gerador do ensaio;
sem cálculo, tipo, fila, cache ou fase novos. 6.Mantido: mesmos arquivos/PNGs,
sem GPU, download ou duplicação da imagem. 7–8.Adequado: runtime, formatos e
sincronização intactos. 9.Evidências em validation.md: skill/links, regiões,
hash, sintaxe e rótulos conferidos. 10.Mantido conscientemente: guia depende de
leitura da IA e revisão, distribuído no repositório; nenhuma garantia automática
de semântica/câmera/forma ou ganho de performance foi entregue.


## EAF-99 — revisão do comparador implementado

1. Adequado: image_similarity::compare apenas por comando explícito; tick/frame/renderer preservados.
2. Adequado: RGB8 limitados, máscara e mapa opcionais; SSIM usa ring11linhas, somas verticais e fila11pixels, O(largura) de scratch. Sem imagens integrais de momentos ou alocação por pixel.
3. Adequado: totais RGBu64, estatísticasf64, seleção binária, limites2048×2048/32MiB e retângulos/IDs validados.
4. Corrigido por desenho: momentos preparados uma vez por pixel, janela atualizada por diferenças e compartilhada entre até16regiões; um decode por entrada declarada.
5. Adequado: O(pixels×(1+regiões)) com16regiões e4096tiles; sort limitado aos tiles e apenas8retornados. Saídas/filas sem crescimento histórico.
6. Adequado: png existente, nenhum modelo/recurso GPU novo; capture explícito já fornece PNG/câmera. Difference somente solicitado, escrita atômica protege entradas e aliases.
7. Mantido com motivo: consulta síncrona sob lock do Service existente, custo de autoria fora do tick. Nenhuma nova thread/dependência/unsafe.
8. Adequado: sem cache persistente ou mudança de revisão; relatório1 documenta política, suporte e ausência de alinhamento. Falhas não criam diretórios/arquivos parciais.
9. Validado:8testes novos com referência SSIM direta independente, cobertura/identidade/posição/cor/fundo/regiões/máscaras/PNG/paths/estado;867Rust e83MCP/339grupos headless completos, Clippy/fmt aprovados.
10. Mantido com motivo: cor codificada sem gestãoICC/gamma, câmera/ROIs manuais, SSIM de luminância sem entendimento semântico. Estudo da casa não prova modelo final, verso/interior, apoio físico ou calibração. Sem ganho de FPS/profiling alegado; contrato image-similarity.md.

## Comparador visual — revisão antes do código

As dez respostas, alternativas e limites estão em [image-similarity.md](image-similarity.md#revisão-estática-anterior-à-implementação). Só comando de autoria, PNG/RGB limitados, somas deslizantes O(pixels), regiões explícitas e nenhum modelo/dependência/tick/render novo.
## EAF-123 — revisão do código implementado

[Respostas das10perguntas e evidências](local-transactions.md#revisão-final-de-performance).
Plan tem writes locais16/2, sem cópia global de saldos/vida; commit/lifecycle
ordenam só tocados. Buffs densos têm índice derivado dono/definição, sem scan
mundial por componente; mutação encapsulada/serde e buffers reutilizados testados.
Single player offline executado com rede negada ao processo macOS, probes EPERM.
769Rust/63alvos, Clippy-Dwarnings/fmt/build e68MCP/280grupos completos headless
aprovados no fonte/binário75693edd70fe6348. Mantidos custos de índice O(B log B)
em mutações, scans de limpeza/retirada/admissão e componentes/máscaras globais;
sem FPS/escala1000 alegados. Renderer e contratos persistidos preservados.

## EAF-123 — avaliação antes de implementar

Checklist e decisões em [local-transactions.md](local-transactions.md).
Write sets limitados por8custos+8efeitos e2atores de vida, sem cópia global;
buffs densos com índice derivado dono/definição, mutação encapsulada e serde
preservado. Single player offline obrigatório. Limites de atores/máscaras,
limpeza e pools permanecem explícitos; sem ganho/escala alegados.

## EAF-122 — revisão do código implementado

[Checklist respondido e evidências](actor-controls.md#revisão-final-de-performance).
Frame de inputs reservado/reutilizado com acesso por slot O(1); preparação e
lifecycle O(P). BTreeMap por handle evita scans no caminho de input, alocando
somente em bind. Log por mudança, reservas de release por tick/total e admissão
compartilhada vivo/replay, sem JSON intermediário. Personagem/profile/cast/passos
individuais, sem renderer/thread/dependência nova. População combinada validada
antes do catálogo; templates reservam somente capacidade de NPC disponível.
760Rust, Clippy/fmt/build e67MCP/277grupos completos headless aprovados; PNGs iguais.
Mantidos conscientemente: inspeção/exclusividade com scans pequenos sob comando,
log/duração/Plan globais transitórios. Sem FPS, escala1000 ou aceite físico alegados.

## EAF-122 — avaliação antes de implementar

Checklist aplicado antes do código em [actor-controls.md](actor-controls.md).
Vínculos/eventos sob comando; inputs por ator em buffer reutilizado e acesso por
slot, O(P) por tick sem cópia de mundo ou busca quadrática. População global e
reservas de templates validadas antes do catálogo; log4096/eventos64 por tick,
sem alteração de renderer/locks/dependências. Serialu64+escopo e referências
geracionais protegem release/reset/replay; neutralização de morte/vida testável.
Índices/máscaras/transações globais e duração do laboratório permanecem custos
transitórios explícitos, não capacidade1000. Revisão final após implementação.

## EAF-115 — revisão do código implementado

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | Adequado: branches de cunha nos consumidores existentes; Box preserva fast paths anteriores. Collider criado somente em insert_body; render tessela somente entrada dirty do cache. |
| 2. Alocações/cópias | Adequado: Geometry/Rise/Volume Copy, arrays6vertices/8triângulos/9arestas e clipping escalar. Sem heap/JSON no kernel de consulta; convex_hull usa custos Rapier na construção, inspeção formata JSON sob demanda. |
| 3. Tipos/acesso | Adequado: enum fechado, dimensões f32 já limitadas0.05..1000 e origem±10000. Cunha somente fixa/visual; query half_extent0..500. Personagens/volumes/surfaces continuam caixas, sem alteração de índices/identidade. |
| 4. Repetição | Adequado: volume/topo/projeção/vertices partilhados; razões de inclinação calculadas só após slabs espaciais passarem. Aritmética Box anterior conservada para preservar fronteiras e TOI legados. Sem sort/hull por tick. |
| 5. Limites | Adequado: até512 entidades, clipping6slabs+1plano,6vertices físicos. Cache36v/slot,24v úteis+12degenerados por cunha; seleção108v. Apoio composto64 e pools existentes preservados. |
| 6. GPU/I/O | Adequado: Vertex/Globals/shader/pipeline/bindings iguais; discriminante somente CPU e invalidação por igualdade. Caixas têm pixels iguais com forma explícita/omitida; alteração de shape invalida entrada, remoção reduz draw count. |
| 7. Sincronização | Adequado: mesmos sensores/fases/solver/dispatcher, sem thread/lock novo. Posições de corpos correntes no spawn e identidades intactas; geometria fixa não implica fonte sensorial onisciente. |
| 8. Invalidação | Adequado: validar tipo/versão/combinações antes de editar; reset/undo/load recompõem collider/shape. Cache/seleção incluem discriminante e dimensões, shift de vértices evitado por slots fixos. |
| 9. Contrato | Rapier×kernel para pontos/raios/overlap expandido nas4orientações, normais/triângulos/contorno/cache e interação física. Integrações cobrem visão/som/direct/projectile/area/fly/spawn, suporte recusado, erro/undo/reset/replay; MCP inclui pixels/viewport/save/load. Evidências em validation.md. |
| 10. Custos mantidos |12v degenerados por cunha e hull esparso trocam simplicidade/cache estável por pequeno desperdício limitado; considerar novo batching apenas com dados. Navegação planar mantém projeção do volume na faixa do corpo, sem subir a rampa. Sem profiling/FPS/escala ou controlador alegados. |

## EAF-115 — avaliação antes de implementar

1. Consultas nos consumidores existentes; cunha opt-in, fast paths das caixas preservados. Compilar colisor só em construção/edição.
2. Planos/vertices em arrays stack fixos; enum Copy e sem String/Vec/Scene clone no tick. Rapier convex hull pode alocar somente na criação.
3. Planos f32 derivados de dimensões já limitadas; validar versão/tipo/combinações antes de mutar. Não usar unsafe ou enum aberto de plugins.
4. Compartilhar volume analítico; manter matemática legada nas caixas para não mudar fronteiras. Apoio plano recusa inclinado explicitamente.
5. Até512 entidades,7planos/cunha,6vertices físicos; sem busca/retry infinito. Render36v/cache mantém índices com12v degenerados por cunha.
6. Vertex/Globals/pipelines/recursos GPU inalterados; cache inclui discriminante e invalida ao mudar forma/dimensão. Consultas MCP/JSON esparsas.
7. Mesmas fases/solver, sem threads/locks novos; coordenar RenderBox/versões com frente gráfica isolada.
8. Reset/undo/load recompõem colisor/cache; presença física atual e shape comuns. Recusa de ator/trigger/surface inclinada evita consumidores planos silenciosos.
9. Testar quatro orientações/volume/vazio/fronteiras/expansão, colisor/render/picking/sensores/área/projétil/clearance/spawn e legado/replay/prefixo/pixels/erros.
10. Padding de cache e convex hull na construção mantidos por simplicidade/coerência; não profiling/FPS/escala. Novo controlador exige decisão separada.

## EAF-111 — revisão do código implementado

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | Adequado: query_spawn_support somente sob comando ou tentativa elegível opt-in. Alvo obsoleto rejeita antes da busca; sem trabalho novo por frame. |
| 2. Alocações/cópias | Adequado: spawn_support::Report/Projection Copy; scan escolhe um par índice/topo no stack. Sem Vec/String/Scene clone/JSON no kernel. Apoio reutiliza arrays locais limitados existentes. |
| 3. Tipos/acesso | Adequado: índice u16/status enum, f32 e validação finita/bounds; constância do template por pending. IDs e mapa de superfícies formatados somente na inspeção. |
| 4. Repetição | Adequado: overlap extraído sem mudança de tolerância em physics::spawn_position_occupied e compartilhado; centro fora de bounds recusa antes de apoio/overlap. Spawn revalida apoio/overlap por manter admissão completa única. |
| 5. Limites | Adequado:8geradores×8tentativas, entidades limitadas, apoio composto64 e fast path de superfície única. Recibo256 inclui Option<Report>, sem novo log/histórico crescente ou busca do próximo piso. |
| 6. GPU/I/O | Não se aplica ao kernel: RenderFrame/renderer/Vertex/dependências inalterados; captura/JSON/persistência sob demanda. Demo usa caixas existentes. |
| 7. Sincronização | Adequado: ordem serial de população preservada, consulta não reserva nem muta. Ocupação usa rigid bodies correntes, incluindo nascimentos/retiradas anteriores na mesma fase. |
| 8. Invalidação | Adequado: opt-in/limites/versão validados antes de edição; sem cache novo para invalidar. Reset/load/undo reconstroem autoria/seed/pending; superfície fixa mantém índice autoral. |
| 9. Contrato | Testes específicos cobrem junção/gap5mm/teto/corpos/limite64, ordem/altura/seed/pending, slot/identidade/cadáver/voo/mobilidade, condição+manual+ondas, alvo ausente, erros/reset/undo/replay adulterado. MCP cobre consultas/PNG/prefixos/load/demo. Evidências em validation.md. |
| 10. Custos mantidos | Scans esparsos e dupla validação física antes da instalação mantidos para partilhar a transação spawn. Recibos maiores limitados256; sem profiling/FPS/escala alegados. Rampas/navmesh/controlador continuam fora deste contrato. |

## EAF-111 — avaliação antes de implementar

1. Consulta esparsa por MCP ou por tentativa elegível, até8geradores×8tentativas; ausência neutra, nada novo por frame.
2. Seleção de maior piso em scan escalar, relatório Copy sem String/Vec/JSON/clone de cena no tick; usar buffers limitados do apoio atual.
3. Índice de superfície u16, coordenadas f32 finitas±10000, distância positiva≤20000; bounds checados antes da mutação.
4. Compartilhar overlap com spawn e apoio modular existente; admissão revalida intencionalmente para manter transação única. Sem sort/cache espacial novo.
5. Entidades e64 superfícies compostas já limitadas, recibos256 recebem relatório compacto; sem retry até sucesso ou procura de piso inferior.
6. Nenhuma alteração GPU/render/I/O frequente; JSON/ID legível somente na inspeção, demo com caixas existentes.
7. Mesma fase serial antes dos sensores; consulta não reserva estado, sem thread/lock/dependência novos.
8. Opt-in validado por versão e limites antes de editar; reset/undo/load recompõem autoria/seed. Corpos ocupantes consultados no runtime atual.
9. Testar apoio modular/gaps/altura/teto/ocupação/corpos distintos, recusa sem identidade/quota, pending/PRNG/ondas/pausa/condição/reuso, erros e replay/prefixos/PNG/legado.
10. Scans e revalidação física esparsos conscientemente mantidos; sem profiling/FPS/escala alegados. Consulta não prova capacidade de spawn ou rota.

## EAF-108 — revisão do código implementado

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | Adequado: Selection::select somente para novo pending depois de guardas/meta/quota/slot; falhas e pausas não ressorteiam. |
| 2. Alocações/cópias | Adequado: enum Single sem heap, Weighted com tabela limitada na construção; ondas herdadas guardam None e consultam base. Sem Vec/String/JSON/clone de fonte no tick. Generator continua Copy, sem tabelas dentro dele. |
| 3. Tipos/acesso | Adequado: pesos/somas/PRNGs u32, multiply-high u64 limitado, template u8 e alvos Reference. Membros/recibos guardam tipo original, sem inferir do ocupante atual. |
| 4. Repetição | Adequado: prefixos e referências compilados uma vez; varredura até16 apenas por novo nascimento. Stream de posições independente e Single sem draws adicionais. |
| 5. Limites | Adequado:16 candidatos×8geradores×base/16ondas, sem clonar herança;8membros/gerador e recibos256 existentes. Draws pendentes limitados por quota+uma escolha, sem fila extra. |
| 6. GPU/I/O | Não se aplica ao kernel: RenderFrame/renderer preservados, JSON/IDs legíveis somente sob consulta. Demo usa autoria/cores existentes. |
| 7. Sincronização | Adequado: mesma fase serial/ordem autoral e comandos externos; sem thread, lock ou dependência nova. |
| 8. Invalidação | Adequado: validação de todos os candidatos/dependências antes de edição; fontes/pending reconstruídos em reset/load/undo. Alvo original não se vincula ao substituto e retirada atribui template do membro antigo. |
| 9. Contrato | Unit cobre buckets0/fronteira/MAX; integração cobre mistura conhecida, vida/tipo próprios, pending/bloqueio/pool/pausa, fonte por onda, alvo removido, erro/dependência/legado/reset/replay. MCP cobre posições independentes, prefixos/PNG e demo de8nascimentos. Evidências em validation.md. |
| 10. Custos mantidos | Tabelas de override limitadas e pequenas adições numéricas em membro/recibo/Generator. Escolha inviável pode bloquear produção por desenho; sem fallback enviesado, profiling, ganho de FPS ou escala601 alegados. |

## EAF-108 — avaliação antes de implementar

1. Sorteio só ao começar um nascimento pendente, atrás dos limites/meta/quota/slot. Retentativas de posição mantêm binding; não sortear por tick sem trabalho.
2. Fontes compiladas na construção: enum Single sem heap, Weighted com Vec limitado16; ondas herdadas referenciam base sem clonar listas. Estado quente Copy guarda índice/Reference/flag e RNG u32, sem strings/listas por tentativa.
3. Pesos u32 positivos≤1000000, soma≤16000000 sem overflow; bucket multiply-high de draw u32. Template e membro u8, referência de alvo congelada para cada candidato.
4. Stream de template seed XOR0x9e3779b9 usa LCG existente, separado de positions; legado único sem draw adicional. Preservar source por onda/quotas e compartilhar transação spawn.
5.8geradores×16ondas×16candidatos limitados; escolha no máximo varre16 entradas, apenas por novo nascimento. Recibos256 e membros8 ganham índice de template; sem histórico crescente.
6. Sem GPU/I/O novo no tick; inspeção converte nomes sob demanda. RenderFrame preservado, demo por autoria existente.
7. Mesma fase serial/ordem autoral/fila externa; sem concorrência, dependência ou locks novos.
8. Validar todos os candidatos, pesos e dependências antes de mutar; reset/load/undo recompõem fontes/RNG e invalidam pending. Não adotar alvo substituto nem alterar escolha por falta de espaço.
9. Testar mesma/outra seed e peso, fronteiras do bucket, fontes/overrides/herança, posição independente, falha/pending/pausa/condição, alvo obsoleto, corpo/vida próprios, quota/replay/prefixo/pixels/undo e legado.
10. Escolha estável pode bloquear produção se o selecionado nunca puder nascer; manter diagnóstico em vez de enviesar para os menores. Pesos são probabilidades relativas sem cota garantida. Sem profiling/FPS/escala601 alegados.

## EAF-105 — revisão do código implementado

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | Adequado: Region::sample só após guardas de produção/meta/quota/cadência e vaga de membros, uma vez por tentativa. |
| 2. Alocações/cópias | Adequado: Region Copy24bytes por ciclo e candidato [f32;3] no stack. Sem coleção temporária ou clone/String/JSON por tentativa; spawn mantém custos esparsos existentes. |
| 3. Tipos/acesso | Adequado: LCG u32 wrapping,24bits altos convertidos para fração f32; min/max finitos e ordenados±10000. Recibos finitos usam PartialEq, sem Eq artificial. |
| 4. Repetição | Adequado: reaproveitar mesma admissão física, sem duplicar overlap/apoio. Points mantém1draw/ciclo e visita circular; região3draws por candidato, inclusive eixo fixo. |
| 5. Limites | Adequado:8tentativas×8geradores, pool8 e recibos256. Falhas avançam tentativa/PRNG e param no limite; não consomem quota/identidade. |
| 6. GPU/I/O | Não se aplica ao kernel: nenhuma alteração de renderer/RenderFrame. Marcações da demo usam autoria existente; posição só serializada em inspeção/replay sob demanda. |
| 7. Sincronização | Adequado: mesma fase/ordem autoral e prioridade da fila externa, sem threads/locks/dependências ou fila adicional. |
| 8. Invalidação | Adequado: fonte exclusiva/versão validadas antes da edição. Reset/undo/load restauram seed; onda atual determina corpo/capacidades e novos indivíduos não herdam estados. |
| 9. Contrato | Testes específicos: amostra conhecida/seed0/mesma/outra, limites, apoio/ocupação/hover/pool, quota/ondas/pausa/ativação, erro/legado/undo/reset e replay adulterado/PNG. Resultados em validation.md. |
| 10. Custos mantidos | Recibo ganha Option<[f32;3]> limitado256, custos físicos de admissão preservados. Candidatos podem repetir; sem pré-amostra/mapa espacial especulativo, profiling/FPS ou escala601 alegados. |

## EAF-105 — avaliação antes de implementar

1. Sorteio somente em ciclos elegíveis, até8tentativas×8geradores/tick. Nenhum trabalho espacial extra quando ativo sem déficit, pausado ou sem quota.
2. Region Copy de min/max na autoria; construir candidato no stack, sem lista temporária/String/JSON por tentativa. Recibo limitado recebe posição numérica.
3. LCG u32 existente, coordenadas f32 finitas ±10000;3draws por tentativa regional, usando24bits altos por eixo. Eixos fixos também consomem draw para regra estável.
4. Preservar seleção de pontos legada (1draw/ciclo e visitas circulares). Reusar transação spawn e mesma validação, sem segundo teste de volume/apoio.
5. Tentativas1..8, pool8 e recibos256. Região não tem retry indefinido/fallback; falha não consome indivíduo/quota, mas consome PRNG/tentativa e deixa posição inspecionável.
6. Sem renderer/GPU/I/O novo em tick; marcadores da demo pela autoria existente. JSON na inspeção/replay sob demanda.
7. Mesma fase serial, prioridade externa e ordem autoral; evitar fila adicional e dependência/lock novo.
8. Configuração exclusiva points/region validada antes de mutar; reset/undo recompõem seed e estado. Respeitar tamanho/capacidades do template atual da onda.
9. Testar seed0/mesma/outra seed, caixa/plano/eixo fixo/limites, bloqueio/apoio/voo/capacidade, admissão sem identidade/quota, pausa/ondas/ativação, replay/prefixos/pixels/undo e legado.
10. Mantidos custos esparsos de spawn/Rapier e recibos com Option<[f32;3]> adicional; sem profiling/FPS/garantia de distribuição contínua ideal ou escala601.

## EAF-96 — revisão do código implementado

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | Adequado: uma sample por gerador/tick, só se activation existe; ocupado consulta máscara do trigger já calculada. |
| 2. Alocações/cópias | Adequado: Config na autoria e State Copy, sem Vec/String/JSON/novo anel por tick. Índice resolvido na construção sem lista temporária. |
| 3. Tipos/acesso | Adequado: volume u8, enum de modo, bool e ticks/counter limitados; máscaras existentes e Reference atual. |
| 4. Repetição | Corrigido por desenho: uma suspensão efetiva; sync_production_clock desloca deadlines uma vez por retomada, sem somar pausa manual e espacial duas vezes. Nenhum overlap repetido. |
| 5. Limites | Adequado: até8geradores×32targets, consulta para cedo ao achar ocupante válido. Última amostra/mudança/contador, sem coleção crescente. |
| 6. GPU/I/O | Não se aplica ao kernel: RenderFrame/renderer inalterados; marcações da demo usam caixas sem corpo existentes. JSON/capturas sob demanda. |
| 7. Sincronização | Adequado: fase serial após pedidos externos/própria limpeza, antes de sensores/física; ocupação anterior e presença atual explícitas. Sem threads/locks novos. |
| 8. Invalidação | Adequado: validar dependência do trigger antes de edição, recompilar em reset/load/undo. Removido/substituto não ocupa em nome de Reference antiga; once rearmado apenas por reconstrução. |
| 9. Contrato | Testes específicos cobrem modos/fases/XYZ, condição+manual, atraso/cadência/ondas, morto/revive/removido/substituto, terminal/erro/undo/reset, prefixos/pixels e legado EAF-95. Resultados completos em validation.md. |
| 10. Custos mantidos | Scans de targets ocupantes para validar identidade, limitados; estado compilado adicional por gerador. Sem profiling ou ganho de FPS/escala601 alegado. |

## EAF-96 — avaliação antes de implementar

1. Até8 consultas por tick ao estado de triggers existente; sem recomputar geometria ou consultar posição oculta para NPCs.
2. Config opcional e estado Copy limitado por gerador, sem filas/arrays novos. Índice de volume compilado uma vez na construção; JSON só na inspeção.
3. Enum de modo, índice u8 e bool/tick para amostra/abertura. Ocupação limitada32 alvos com validação de Reference presente; não adotar substituto de slot.
4. Reusar suspensão efetiva única: manual e condição se combinam antes de deslocar prazos, evitando dupla contagem na retomada. Não criar relógio separado por gerador.
5. Reusar limites8geradores/32volumes/32alvos. Nenhum log crescente de amostras; último estado/tick/counters limitados pela execução36000.
6. Sem GPU/I/O adicional no tick. Região da demo é entidade visual sem corpo usando ferramentas existentes; não alterar RenderFrame.
7. Ler último snapshot pós-física, depois de pedidos externos. Remoções presentes filtradas mesmo antes da atualização pós-física seguinte; nascimento novo não ocupa retroativamente.
8. Autoria valida volume referenciado; edições/reset/undo recompõem bindings e latch. Pausa manual não é sobrescrita; cancelamento/conclusão não reabre.
9. Testar entrada/saída/início dentro/XYZ/contato, modos, desabilitado, pausa combinada, prazos, morto/presente, retirada/reuso, seleção de alvos, replay/prefixos/pixels, erros/limites e legado.
10. Mantidos scans pequenos de alvos ocupantes para validar identidade; nenhum ganho de FPS/escala prometido. Propriedade do mundo não fornece conhecimento perceptivo ao NPC.

## EAF-95 — revisão do código implementado

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | Adequado: Generator::control somente por pedido; step_generators verifica enum depois da limpeza existente e pula produção/ondas suspensas. |
| 2. Alocações/cópias | Adequado: enum/índice/Option<u64> Copy, nenhuma fila/anel novo. Sem clone/JSON/alocação adicional no tick da política. Custos esparsos de spawn/Rapier preservados. |
| 3. Tipos/acesso | Adequado: ID textual resolvido uma vez para u8; status enum e timestamps u64. Prazos deslocados só na retomada, dentro dos limites da execução. |
| 4. Repetição | Adequado: mesma fase/ordem de população, sem ordenar ou reinicializar seed; pausas repetidas não renovam paused_since. |
| 5. Limites | Adequado:64 pedidos/tick e4096 totais compartilhados, recibos256; testes saturam ambos e conferem3840 descartes. Pool8/geradores8/ondas16 mantidos. |
| 6. GPU/I/O | Não se aplica ao kernel novo: renderer/RenderFrame inalterados. Título nativo usa invalidação existente; captura e replay sob demanda. |
| 7. Sincronização | Adequado: comandos consumidos sequencialmente antes de geradores, sensores/habilidades/física; sem thread, lock ou fase concorrente novos. |
| 8. Invalidação | Adequado: reset/edição/undo/load recompõem status autorado e descartam pedidos. Limpeza por Reference/life_index continua pausada/cancelada; não adota slot reciclado. |
| 9. Contrato | Validado nos testes específicos: pausa no deadline, restante inicial/interonda/cadência, comandos no mesmo tick, morte/revive/limpeza, cancelamento terminal, outros geradores, locomoção, erros/replay/prefixos. Resultados completos em validation.md. |
| 10. Custos mantidos | Inspeção cria Vec/JSON limitado por8 controles sob demanda; título formata status apenas ao atualizar texto. Limpeza continua até8 membros por gerador, necessária para o contrato. Sem profiling ou promessa de FPS/601atores. |

## EAF-95 — avaliação antes de implementar

1. Controle eventual; tick consulta estado enum antes de avançar ondas/nascer. Limpeza já existente continua nos8 membros.
2. Sem fila/anel novo: ControlGenerator ocupa a fila64/log4096/recibos256 de população. Comando Copy com índice u8; strings somente na admissão/inspeção.
3. Estado Active/Paused/Cancelled e Option<u64> para início da suspensão; deslocar prazos na retomada, sem atualizar todos os prazos por tick pausado.
4. Reusar a ordem e a validação de população, resolução do ID uma vez; sem sorting, clone de cena ou reinicialização de seed.
5. Custos limitados pelos8 geradores e orçamento compartilhado; repetição idempotente também consome pedido, evitando log sem limite.
6. Sem GPU/I/O novo no tick; captures/replay sob demanda e JSON na fronteira.
7. Mesma fase serial antes dos sensores, comandos na ordem da fila; pausa global não aplica pedidos até step/play.
8. Reset/edição/undo/load recompõem runtime e descartam pedidos; cena só armazena initially_active. Cadáveres e life_index continuam observados durante suspensão/cancelamento.
9. Testar pausa no prazo/atraso/cadência/ondas, start inativo, retomada repetida e múltiplos comandos/tick; cancel terminal sem remoção, replay pendente/prefixo/pixels, reuso/revive, erro e limites.
10. Mantidos scans limitados/limpeza automática: congelar NPCs ou cadáveres seria um controle de mundo distinto. Sem dependências, estruturas globais ou promessa de FPS/escala novas.

## EAF-93 — revisão do código implementado

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | `Progress::advance` é constante por gerador/tick e confirma no máximo um evento. Sem ondas, `wave=None` mantém reposição contínua. |
| 2. Cópias/alocações | Progresso Copy pequeno; bindings em `State.wave_bindings` compilados na construção, não copiados junto de Generator por tick. Anel256 de eventos pré-reservado só quando há ondas. Custos de spawn/Rapier mantidos. |
| 3. Tipos/acesso | Índices u8 para onda/template, Reference para alvo/membro e u32/u64 para quotas/prazos. `Member.wave` impede adotar outro ocupante ou confundir coortes. |
| 4. Repetição | Contagens da onda aproveitam a varredura de membros. Sem strings/sorteio adicional para transições; seed só muda nos ciclos de nascimento existentes. Inspeção também conta diretamente, removendo Vec intermediário de membros. |
| 5. Limites |16 ondas×8 geradores, membros8 e pool global8; soma de quotas validada contra max_births≤36000. Eventos até256 no recorte finito, anel defensivo256. Tentativas/cadência/recibos de nascimento preservados. |
| 6. GPU/I/O | Nenhum novo pipeline/readback/RenderFrame; imagens sob demanda. Eventos regenerados no replay, sem multiplicar log de inputs. |
| 7. Sincronização | Mesma fase serial depois de comandos externos e antes dos sensores; confirmação posterior não muda o passado. Sem threads/locks/dependências novos. |
| 8. Invalidação | Alvos inclusive de ondas futuras vinculados ao indivíduo original. Membros preservam geração/life_index; revival não retrocede sequência nem expulsa indivíduo. Reset/undo recompilam estado e invalidam tokens. |
| 9. Correção | Rust cobre atraso/quota/template, streaming por limite vivo, retenção/retirada/revive, reuso externo, alvo futuro, conclusão e16×8 ondas/eventos. MCP cobre autoria/load/undo, prefixos/capturas, demo3ondas completa e erros atômicos; evidências em validation.md. |
| 10. Custos mantidos | Vetores de bindings por gerador na construção, estado global limitado e consultas de presença/vitalidade existentes. Contagem de vida consultada para coorte depois da limpeza; sem otimização especulativa, profiling, ganho de FPS ou escala ampliada. |

## EAF-93 — avaliação antes de implementar

Sequência opcional de até16 ondas por gerador,8 geradores e pool global8 NPCs preservados. Compilar template/alvo de cada onda uma vez; não procurar string/clone de configuração por tick. Estado de onda Copy pequeno (índice, fase, contagem/prazo), referência+onda por membro. Reusar nascimento/retirada/seed/cadência existentes; um avanço de onda por tick, sem loop de transições. Eventos derivados em anel limitado, sem inflar log de inputs. Somar orçamento na validação, contador só por sucesso. Varredura atual dos8 membros reutilizada para condição de término; mortos preservados podem impedir vagas, recusa observável. Ressurreição não reabre onda já confirmada e nunca remove excedentes. JSON/captura sob demanda, sem RenderFrame/threads/dependências novas. Testar sequência/tempos, bloqueio/retentativa, alvo removido, cadáver/revive, saturação, identidade externa, legado/replay/prefixos e erros/undo/reset. Sem promessa de throughput/FPS/601 atores.

## EAF-92 — revisão do código implementado

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | `Simulation::step_generators` retorna naturalmente sem trabalho quando o vetor é vazio. Até8×8 membros conferidos por tick; candidatos apenas com déficit/cadência/budget. |
| 2. Alocações/cópias | `Generator` é Copy: cópia limitada de um estado por gerador, sem configuração/Scene/heap por tick da política. Membros são array8 e recibos VecDeque pré-reservado256 quando há gerador. Spawn/remove mantêm custos esparsos de Entity, mapas e Rapier. |
| 3. Tipos/acesso | Template u8, alvo e membros por Reference, contadores/PRNG inteiros. IDs textuais resolvidos na construção; pontos acessados diretamente da configuração imutável. Contagem usa `Catalog::matches`, nunca somente slot. |
| 4. Repetição | Um ciclo sorteia um início e visita candidatos distintos até min(tentativas,pontos); não tenta até encontrar um válido. Para cedo em meta/budget/capacidade de acompanhamento. Sem RNG quando meta satisfeita. |
| 5. Limites |8 geradores/32 pontos/8 membros, pool global8 NPCs,64 tentativas automáticas/tick e256 recibos. Orçamento de até36000 nascimentos por gerador; contadores seguros dentro do limite de36000 ticks da execução. Eventos derivados não crescem o log de inputs. |
| 6. GPU/I/O | Nenhum pipeline/readback por política; usa invalidação de tick existente. JSON/Vecs de inspeção só sob demanda; captura offscreen explícita. |
| 7. Sincronização | Fase serial após fila externa e antes dos sensores. Ordem autoral de geradores é prioridade explícita de capacidade; sem nova thread/lock/dependência. |
| 8. Invalidação | Alvo original congelado; membro ausente/reutilizado é removido do acompanhamento. Reset/edição recompilam geradores. Retirada usa o mesmo caminho que preserva origens e limpa componentes; ressurreição limpa timer por life_index, inclusive se voltar a morrer entre amostras, e não expulsa excedentes. |
| 9. Correção | Rust cobre seed/prefixo, cadência/budget, falhas sem identidade, morto/revive/retirada no prazo, prioridade/reuso externo, competição, limites e erros atômicos. MCP verifica o contrato, imagens e replay; resultados em validation.md. |
| 10. Custos mantidos | Estado por gerador copiado e slots limitados varridos, validação geométrica por tentativa, memória de corpos retidos, custos existentes do spawn. Não há índice espacial novo, fairness, expansão para601 atores ou ganho de FPS medido. |

## EAF-92 — avaliação antes de implementar

Até8 geradores,32 candidatos por gerador e8 membros por gerador, dentro do pool global8 NPCs. Configuração compilada uma vez (template u8 e alvo Reference); estado numérico Copy, sem clone de Scene/Simulation nem strings por tick. Varredura limitada de membros por tick para ausência/morte/ressurreição; sorteio somente quando há déficit e vence a cadência. Até8 tentativas por ciclo/gerador, sem busca até encontrar espaço. Usar spawn/remove existentes, preservando custos esparsos de configuração/Rapier por nascimento. Eventos derivados regenerados no replay, separados do log de comandos externos; anel256 de recibos. Seed e ordem autoral determinísticos; ocupação/apoio/capacidade revalidados antes de consumir identidade. Nenhuma alteração de RenderFrame, dependência, thread, população máxima ou promessa de FPS. Testar retenção/retirada de cadáver, ressurreição, saturação, prioridade externa, identidade, replay/prefixo/reset/undo.
## Composição da lanterna da casa

Checklist anterior/final em [cottage-composition.md](cottage-composition.md).
Lote nativo desloca140v de10ilhas entre bronze/flowers, sem buffers substitutos
no cliente; demais posições/índices e cena intactos. Mesmos32596v/54728t/55948draw;
flowers congela608coordenadas (7296B f32). Sem tetos elevados, tick ou FPS medido.

## Transformação local de ilhas — EAF-125

Checklist prévio e dez respostas finais em [mesh-island-transform.md](mesh-island-transform.md).
Seleção/parâmetros antes de copiar posições; cálculo f64 e orientação final f32
antes de substituir Geometry. Arcs de rig/clips preservados; domínio implícito
congelado custa12V bytes (até192KiB/asset). Mesmos tetos, renderer e tick.
863Rust/Clippy e88MCP/363grupos headless no buildc9ffc6d3852bd23d. Sem FPS medido.

## Caixas chanfradas nativas — EAF-121

Checklist anterior/revisão do diff em [chamfer-boxes.md](chamfer-boxes.md).
Contagens8/24v e12/44t antes dos buffers/clone; faces por26planos conhecidos,
f64→f32 com orientação final verificada. Vetores pequenos por caixa/atan2 por
face mantidos como custo limitado da autoria. Sem novo tick/renderer/dependência
ou limite elevado.860Rust/Clippy e87MCP/358grupos headless no build945ebbe0ba1b6c41.

## Pedra e piso da casa

Checklist anterior/final em [cottage-stone.md](cottage-stone.md).14blocos/26lajes
usam novo gerador; somente stone-blocks/paving mudaram. Custo+640v/+1280t,32596v/
54728t armazenados e55948t draw;172v livres, zero slots, tetos mantidos. PRNG
separado preserva demais famílias. Sem frequência nova no runtime ou FPS medido.

## Ornamentação em relevo da casa

Checklist anterior/final em [cottage-relief.md](cottage-relief.md). Somente bronze
ganha geometria:19lâminas36v/68t por Sweep nativo; substituição de motivos mantém
custo líquido+236v/+460t.31956v/53448t armazenados,54668t draw,812v livres e zero
slots. Sem Rust/render/material novo ou frequência por tick; escalas lineares e
resolução explícita conservadas. Sem medição de FPS.

## Cornijas contínuas e cobertura da casa

Checklist anterior/final em [cottage-roof.md](cottage-roof.md). Somente cliente,
receita e comparação: backing140prismas→3cascas fechadas indexadas796v/1580t;
total31720v/52988t armazenados e54208t draw (−670v/−1080t ante b33a76c),357peças
Sweep. BisseçãoX fixa40iterações só na autoria; mesmos caps/renderer/binário e
zero trabalho de simulação novo. Mudança de contagens não é ganho de FPS medido.

## Revisão estrutural da casa com perfis nativos

Checklist proporcional anterior e dez respostas finais em
[cottage-structure.md](cottage-structure.md). Apenas receita/publicação:16assets,
21instâncias,32390v/54068t armazenados,55288t desenhados;378v e zero slots livres.
Planejador admite323peças/refina13, append evita reenviar arrays antigos; staging
e preview/commit duplicados continuam custos explícitos da autoria. Renderer e
limites mantidos. Sem medição de FPS ou aceitação artística presumida.

## Varredura de perfil3D

Checklist anterior e revisão final em [mesh-sweep.md](mesh-sweep.md). Plan valida
parâmetros e contagens antes de reservar os três buffers de saída; frame e escala
por anel. Append clona somente após admissão do orçamento, preserva índices e
metadados, recusa rig. Planejador puro sem expansão da superfície; até8tentativas
de resolução por peça. Mesmos limites, renderer e fases de simulação.



## Revisão arquitetônica da casa

Inspeção documental sem mudança da engine ou cena: dez recortes e três novas
vistas MCP offscreen, com câmera restaurada e hashes preservados. Checklist
proporcional e diagnóstico em [elven-cottage-review.md](elven-cottage-review.md).
Nenhuma otimização, novo build ou ganho de desempenho alegado.

## Casa élfica — conteúdo autorado

Checklist anterior e dez respostas finais em [elven-cottage.md](elven-cottage.md).
Cliente Python sem dependências novas, meshes publicados via MCP e coluna nativa
instanciada; Rust/frame/tick/shaders/limites intactos.32172v/53376t no documento,
61312t no draw. Tetos existentes determinaram amostragem e agrupamento por material;
não foi adicionado trabalho por frame além da geometria/material autorados.
Capturas explícitas e validação de autoria são custos síncronos mantidos.


## Amostragem de plantas e proveniência de junção

Checklist anterior e dez respostas posteriores em
[live-plant-junction-study.md](live-plant-junction-study.md). Consulta nova em
Engine::execute chama Definition.sample só por pedido; Sampling/Face ganham
Serialize, sem novos campos persistidos ou trabalho no tick. Fitted serializa o
material_plan já calculado, sem recomputar correspondências. Mantidos vetores e
custo síncrono limitados da autoria, mais payload de consulta e serialização.
Estudos Python/CG/capturas são ferramentas de autoria/pesquisa, não runtime.
Sem cache, thread, recurso GPU, dependência ou ganho de FPS novo.

## Seção e repetição de traços — revisão final

Checklist prévio e tradeoffs em detail-domains.md.
1. Adequado: opções só na autoria e fragmentos detalhados, nenhum tick NPC novo.
2. Adequado: enum+Option pequenos no DetailArc, sem geometria/clones por vértice.
3. Adequado: períodos e cápsulas variáveis validados em f64 antes de publicar;
   enum fechado, materiais GPU reprC/Pod com offsets testados.
4. Mantido com motivo: até32distâncias por plano/96triplanar; wrap uma vez por
   plano, nenhuma vizinhança9células. Rounded acrescenta smoothstep à avaliação
   da banda existente; cálculo é limitado e preserva aritmética legada.
5. Adequado: mesmos tetos de faces/vértices/segmentos; repetição não expande dados.
6. Mantido com motivo: +16 B/slot, tabela19584 B (+272 B); Vertex64/Globals176
   mantidos. Sem textura/novos draws/readback rotineiro ou recurso adicional.
7. Não se aplica: sem worker, lock ou dependência nova.
8. Adequado: igualdade/Arc/material cache existentes incluem ambos campos;
   toggle/slots/remoção/reset/load/viewport/history cobertos na regressão.
9. Validar por Rust/Clippy e MCP completo; band/None pixel-idênticos em3baselines,
   periodicidade em domínio negativo, câmera distante, triplanar e replay.
10. Mantido com motivo: repetição do patch pode ficar visível; coordM não equivale
    a metros físicos em superfície deformada. Perfil não fornece fibra, UV ou
    múltiplas camadas PBR. Não alegar FPS/realismo pela aprovação dos testes.

## EAF-80/EAF-85 — diagnóstico material por triângulo

Checklist prévio e alternativas em mesh-material-domain.md. Revisão de código:
1.só consulta explícita, zero tick/frame;2.Geometry emprestada, sem clone de
vértices/rig/clipes; lista opcional de faces ordenada uma vez e topK com capacidade
reservada, remove antes de inserir quando cheio;3.f64 temporário, dados finitos,
seleção/escala/forma numérica validados;4.O(V+T) validação, O(SlogS) seleção e
O(32S) topK máximo, sem grafo/solver;5.tetos existentes, amostras0..32, transporte
2MiB mantido;6.sem shader/buffer/readback/GPU novo;7.sem lock/thread/dependência;
8.sem cache/formato persistido novo, mesma guard de revisão do dispatcher;
9.casos analíticos e pureza/schema/undo MCP, seguida de estudo da árvore;
10.mantidos por escopo validação completa por consulta e projeção afim por
triângulo. Não fornece direção de fibra, distorção entre faces ou FPS medido.

## EAF-85 — condição de tangência de colares, antes/depois

Alternativas e checklist anterior ao código em [mesh-organic.md](mesh-organic.md).
Recorte: consulta pura `Fusion.transition_lengths`, sem formato persistido novo.

1. Adequado: só consulta/autoria; nenhum tick/frame/render novo.
2. Adequado: três comprimentos na stack; mesmos vetores de geometria/refinamento
   limitados, sem cópia adicional do documento no kernel.
3. Adequado: f32 público/f64 temporário, contornos e índices preservados;
   comprimentos em ordem explícita dos colares, relatório confirma a opção.
4. Mantido com motivo: até3misturas/cilindros extras por avaliação do campo,
   normalização dos frames e40bisseções existentes. Custo limitado de autoria;
   a preparação do campo poderia ser compilada se latência medida justificar.
   Legado sai antes das novas misturas e conserva resultado bit a bit.
5. Adequado: faixas positivas limitadas, círculos inteiros/guardas verificados
   analiticamente antes da projeção; mesmos4096v/8192t/refinamento.
6. Não se aplica: nenhum buffer/shader/readback/upload novo. Comparação GPU é
   captura explícita. Na árvore, mais128v/256t no cotovelo,65050t desenhados,
  486t livres; não se aumentou teto.
7. Adequado: dispatcher puro existente, sem lock/thread/dependência nova.
8. Não se aplica: sem cache novo; publicação explícita usa invalidação/undo
   existentes. Outros assets/instâncias/plantas não são reconstruídos.
9. Adequado: teste de gradiente radial na guarda, legado exato, topologia,
   envelopes/erros/pureza MCP; ensaio árvore preserva demais poses0/60,
   materiais/rig/clipes/plantas e pixels no replay. Evidência em validation.md.
10. Mantido com motivo: malha é aproximação finita, campo radial limitado;
    sem prova global de autointerseção, sweepXYZ editável ou todos os encaixes
    resolvidos. Curvatura do pescoço é amostrada em1001pontos, não certificada
    como limite analítico global. Nenhuma alegação de ganho de FPS.


## EAF-80 — teto de desenho

Revisão estática anterior e10respostas posteriores em
[mesh-draw-budget.md](mesh-draw-budget.md). Aumento deliberado32768→65536t
somente no desenho; buffer/Vec por potência de2 podem reter13MiB cada na faixa
de meshes (9,75MiB ativos no máximo). Preflight mantém os demais limites;
scan legado adicional só para cenas anteriores55. Validação em validation.md,
sem profiling/FPS ou promessa de escala de produção.

## EAF-80 — materiais procedurais, antes/depois

Desenho prévio e alternativas em [procedural-materials.md](procedural-materials.md).

1. Frequência: receita somente em autoria; shader avalia padrão por fragmento
   quando habilitado. Frame mantém short-circuit por documento/relógio.
2. Alocações/cópias: canal opcional limitado a4096 triplas/asset, compartilhado
   pelo Arc de Geometry; setter da receita preserva os payloads. Vertex36→52bytes,
   +1,5MiB para32768tri expandido. Sem copiar a receita por vértice.
3. Tipos/acesso: enum de3 padrões, seedu16, tabela fixa17×80=1360bytes e slotu32
   derivado do frame. Lookup de asset/slot fora do loop de vértices.
4. Repetição: até4 oitavas, valor usa8 hashes e celular27 células por oitava;
   footprint atenua e evita oitavas sem cobertura. Warp é opt-in. Derivadas
   em fluxo uniforme; não há busca de vizinhos de mesh no fragmento.
5. Limites: budgets geométricos mantidos, receita/finito/contagem verificados
   antes da mutação; canal/planta fonte/avaliado seguem mesma validação.
6. GPU/I/O: buffer de materiais persistente, upload só quando tabela difere;
   layout/atributos maiores são custo também em assets neutros. Passe de sombra
   mantém posição; nenhum readback/textura/arquivo externo por frame.
7. Sincronização: mesmo dispatcher/undo, sem lock/thread/dependência nova.
8. Invalidação: mudança da receita/canal invalida Frame mesmo no mesmo tick;
   tabela compara conteúdo; slots removidos/reordenados são recompostos.
9. Correção: testes de neutralidade/atomicidade/Arc/canais/skin/subdivisão,
   correspondências de regeneração, migração e layout; MCP mede pixels,
   repeat/undo/load/replay/prefixo e remoção de slot. Evidência em validation.md.
10. Custos mantidos: CPU skinning/expansão/upload por instância; interpolação
    síncrona na autoria e custo celular por fragmento. Footprint não é garantia
    analítica de antialiasing. Nenhuma alegação de FPS ou escala de produção.

## EAF-80 — sombras direcionais, antes/depois

Avaliação anterior em [directional-shadows.md](directional-shadows.md).

1. Frequência: só no renderer, mapa redesenhado em cada apresentação ativa.
   Setter/validação de parâmetros fora do tick de gameplay.
2. Alocações/cópias: depth persistente, recriado só se resolução mudar;1×1 ao
   desativar. Uniform160bytes; sem copiar geometria ou poses para o shadow pass.
3. Tipos/acesso: matriz ortográfica finita, resolução512/1024/2048 limitada;
   comparison sampler Depth32Float. Vertex e IDs de assets permanecem iguais.
4. Repetição: até2 draws extras reusam vertex/mesh buffers. Sem extrair malhas
   novamente;9 comparações no fragmento dentro do volume, nenhuma fora/desativado.
5. Limites: extent0,25..256m, bias0..0,01, máximo16MiB de depth +driver; clipping
   explícito, sem ampliar cobertura ou resolução silenciosamente.
6. GPU/I/O: novo passe somente opt-in; sem readback/arquivo por frame. Bias e
   PCF têm custo/artefatos conhecidos; não equivalem a área light/penumbra física.
7. Sincronização: layouts separados de depth e sampling evitam uso simultâneo
   de uma textura como attachment/amostragem. Mesma queue, sem locks/threads.
8. Invalidação: shadow map recebe sempre conteúdo atual; remoção de oclusores,
   resize e mudança da luz não mantêm mapa antigo. Scene53/replay57 explícitos.
9. Correção:4 testes matemáticos/contratuais novos, cenário MCP de pixels,
   atomicidade, oclusores, resize, load/reset e replay/prefixo; regressão completa
   em validation.md. Captura da base inspecionada para acne/bias.
10. Custos mantidos: passe por apresentação mesmo estático, PCF fixo, ambiente
    sem oclusão, mapa finito sem cascatas. Sem FPS/ganho de performance alegados.

## EAF-80 — sombreamento de meshes, antes/depois

Decisão e avaliação anterior em [mesh-shading.md](mesh-shading.md).

1. Frequência: `Frame::update` conserva short-circuit por documento/tempo; apenas
   assets smooth acumulam normais na preparação de um frame alterado.
2. Alocações/cópias: scratch fixo adicional4096 Vec3/48KiB na stack, reutilizado
   entre instâncias. Setter preserva Arc de geometria/rig/clipes. Planta copia
   apenas metadados do Asset da fonte, sem rebase ou cópia de seus payloads.
3. Tipos/acesso: enum fechado flat/smooth; índices u32 existentes. Posições já
   avaliadas determinam área e orientação, incluindo skinning/rotação/escala.
4. Repetição: dois passes O(T)+normalização O(V) por instância smooth; sem scan
   de vizinhos por vértice. Normal por face mantida como fallback; sem busca de
   asset por vértice, descoberta e teste do modo fora do loop das faces.
5. Limites: budgets de assets/vértices/triângulos/instâncias mantidos. Fonte e
   avaliado precisam concordar no metadado; cena52/replay56 explícitos.
6. GPU/I/O: layout Vertex/shader/buffers existentes; apenas dados de normal
   diferentes no upload/cache já existentes. Nenhum pipeline/readback adicional.
7. Sincronização: mesmo dispatcher/staging, sem dependência/thread/lock nova.
8. Invalidação: igualdade do documento inclui shading, mudando o frame no mesmo
   tick. Planta atualiza fonte sem rebase; subdivisão/preservação copiam o modo.
9. Correção: testes específicos de área, seam, transformação, deformação,
   cancelamento/colapso, cache/undo/Arc/migração e fonte canônica; MCP verifica
   mudança visível e pixels repetidos/replay/load. Resultados em validation.md.
10. Custos mantidos: skinning/normais por instância em CPU e upload integral
    quando animado. Sem normal cache por pose ou garantia de superfície suave
    geometricamente; sem FPS/profiling ou suporte de plataforma novo alegados.

## EAF-79 — topologia e subdivisão, antes/depois

Avaliação anterior ao código e escolhas em [mesh-topology.md](mesh-topology.md).

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | Adequado: `Topology::inspect`, `Plan::new` e amostragem refinada apenas em consulta/autoria/validação. Nenhuma fase de gameplay/frame nova. |
| 2. Alocações/cópias | Mantido com motivo: mapa de incidência, marcação e plano temporários limitados; posições/pesos de saída próprios. Source/Geometry/Clip compartilham Arc no histórico; clipes mantêm Arc e rig copia apenas o payload do asset editado. |
| 3. Tipos/acesso | Adequado: índices u32, aresta por par ordenado; referências de componentes têm scope/revision. Média usa f64 temporário contra cancelamento/arredondamento; saída f32. Sem unsafe, ABI/dependência ou estrutura half-edge global. |
| 4. Repetição | Adequado: incidência por face O(T log E), consulta de midpoint por busca binária; sem scan de faces por vértice. Kernel único de pesos e de filhos baricêntricos. Sampling de planta carrega face/material em todo vértice, sem nearest-point. Comparação de topologia em asset/geometry é mantida para proteger os dois caminhos públicos antes dos índices; validação síncrona fora do tick. |
| 5. Limites | Adequado: contagens exatas antes das posições/pesos; checks de armazenamento/desenho e limites4096v/8192t. Plant counts verifica níveis antes de alocar base; crescimento4^n finito e rejeição antecipada. Pools/escala NPC não mudam. |
| 6. GPU/I/O | Não se aplica ao kernel: renderer/cache existentes recebem o asset alterado; nenhuma criação de pipeline/upload/readback por tick adicionada. Capturas de verificação explícitas. |
| 7. Sincronização | Adequado: mesmo dispatcher/staging, sem nova thread/lock. Edição síncrona limitada; não alegar cooking assíncrono. |
| 8. Invalidação | Adequado: seleção somente sessão; scope muda no reset/load/edição/undo/redo. Guardas verificam referências antes de staging e detectam alteração anterior do mesmo asset no lote. Nenhum cache de topologia permanente a invalidar. |
| 9. Contrato | Testes específicos cobrem sucesso/falha, pesos/clipes, seams/T-junctions, seleção sem gameplay/histórico, budgets e versões. MCP cobre criação→subdivisão→escultura→rig→clipe, plantas/refino/perda de detalhe, persistência/render/replay. Evidência final em validation.md. |
| 10. Custos mantidos | Mapas/Vec por edição, normalização/validação integral de saída e amostragens repetidas no validate de plantas. Limites pequenos e fonte reproduzível justificam este custo; cooking/cache por fonte exige desenho próprio. Sem FPS ou escala de produção alegados. |

## Plantas paramétricas — avaliação antes de implementar

1. Geração e preservação executam na autoria/validação, nunca no tick de gameplay.
2. Fonte canônica e payloads compartilham Arc no histórico; reamostragem aloca somente malha e mapas limitados do asset alterado.
3. IDs de perfil/asset são autorais; índices e coordenadas materiais são correspondências explícitas, nunca identidade persistente. Sem dependência/ABI nova.
4. Grades de seis faces geram correspondência direta por célula/bari sem busca espacial por vértice. Canonicalização não ocorre em cada regeneração.
5. Até16 plantas, budgets existentes de4096v/8192t por asset; validar a faixa1..63 antes das contagens e de reservar memória; fonte canônica também tem budget global.
6. Reusar renderer/cache existente; nenhuma captura/I/O por tick. Persistência contém fonte e avaliado, custo explícito dentro dos limites de arquivo.
7. Dispatcher/transação síncronos existentes; sem nova thread/lock. Custos da edição limitados, sem alegação de cooking assíncrono.
8. Regeneração publica somente resultado validado. Undo/reset/load/replay usam autoria e malha; erro conserva última versão válida e execução.
9. Testar orientação/contagens, mudança dimensional, tesselação, escultura/pesos/clipes, erros atômicos, save/load/undo/replay/prefixo/PNG e regressão funcional headless.
10. Recorte de perfil retangular/extrusão e deformação affine positiva. Solver geral, BRep, curvas/loft, transferência por proximidade/cage e editor visual permanecem fora; sem promessa de FPS.

## EAF-76 — autoria nativa de malhas, antes/depois

Avaliação prévia e alternativas em [mesh-authoring.md](mesh-authoring.md), registradas antes do código.

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | Preparação/validação apenas na edição. `meshes::Frame::update` evita reavaliar documento/tempo inalterados; assets sem playback ignoram avanço do relógio. Consultas/captura/replay são explícitos, sem fase nova de gameplay. |
| 2. Alocações/cópias | `Asset` compartilha Geometry/Rig/Clip por Arc; histórico e staging copiam metadados, não payloads. `Arc::make_mut` copia somente geometria esculpida. Frame e cache GPU reutilizam Vec/capacidade; scratch de posições/ossos/vizinhança tem tamanho fixo. JSON/poses e clonagem de metadados alocam sob demanda; não alegar zero alocações. |
| 3. Tipos/acesso | Triângulos u32, ossos u8, quatro pesos f32, matrizes/quaternions glam. Validação verifica índices/finito/hierarquia/soma de pesos. Assets/instâncias têm IDs autorais separados de identidade de gameplay. Sem ABI/ECS/dependência nova; serde habilita Arc. |
| 4. Repetição | Traço O(amostras×(vértices+triângulos)); smooth acumula vizinhos em um passe por face antes de editar, sem scan de todas as faces por vértice. Pose percorre pais em ordem e usa binary search nas chaves. `Scene::validate` verifica assets uma vez, fora dos probes por template NPC. Bind inversa e poses compartilhadas ainda são recalculadas por instância avaliada, conscientemente mantidos no recorte limitado. |
| 5. Limites |16 assets/32 instâncias;4096v/8192t por asset;16384v/32768t armazenados e32768t desenhados;32 ossos/4 influências;16 clipes/4096 chaves globais;128 amostras. Orçamentos e referências verificados no staging antes de publicação. Não representa escala de produção. |
| 6. GPU/I/O | Buffer adicional persistente e crescimento geométrico. Upload integral da malha preparada somente quando conteúdo muda; remoção muda contagem. CPU skinning/expansão/normais flat mantidos; não atribuir ganho de FPS. Captura/readback e arquivos não entram no tick. Persistência tenta JSON compacto quando o formatado excede8MiB; reutiliza o Vec e rejeita excesso compacto antes da escrita, preservando o limite de leitura. Custo extra ocorre somente no salvamento grande. |
| 7. Sincronização | Dispatcher/locks existentes, sem thread/scheduler novo. Edição/inspeção/captura continuam sob o serviço síncrono; grandes payloads são limitados e podem bloquear esse serviço. Não apresentar como cooking/streaming assíncrono. |
| 8. Invalidação | Comparar documento e tempo efetivo, não apenas revisão; frame reutilizado sobre reset/undo/load e cenas distintas no mesmo tick. Renderer compara vértices, respeita remoção/growth/cor/pose e mantém caixa/guias independentes. |
| 9. Correção | Testes novos cobrem rig/pesos/interpolação/loop, deformação previsível, traços/Arc/undo, rollback/budgets/legado e replay adulterado. Cenário MCP meshes cobre autoria completa, consultas sem mutação, save/load/reset e GPU/replay/prefixo/cache/viewport. Resultados finais em validation.md. |
| 10. Custos mantidos | CPU skinning, bind por instância, expansão de triângulos/upload total quando animados, buscas de IDs até16 assets e comparação de conteúdo. Edição continua reiniciando execução. GPU skinning, pose compartilhada e formato cooked requerem outro incremento com tradeoffs próprios. Sem profiling/FPS ou outra plataforma alegados. |

## EAF-70 — avaliação antes de implementar

Templates imutáveis por execução, slots limitados aos8 NPCs mais jogador existente. Preparar/copiar apenas configuração e componentes de um nascimento; nenhum clone de Scene/Simulation por spawn/tick. Catálogos de habilidades/regras compartilhados; bases/grants/rotas/motores por instância inicializados explicitamente. Validação geométrica consulta corpos presentes e apoio estático antes de consumir identidade; erro não deixa corpo parcial. IDs e fila numéricos, log/recibos limitados; contatos usam slots limitados com identidade histórica, sem acumular nomes por geração. Ordem lógica por InstanceId desacoplada do slot; preservar legado nos mundos sem templates. Inspeção/replay/captura sob demanda. Validar reuso entre templates/estados, origem antiga, capacidades/ocupação/apoio/voo, sobreviventes e replay. Sem ampliar população, ECS/dependências ou prometer FPS.

## EAF-70 — revisão do código implementado

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | `Simulation::apply_population` recebe slice vazio no tick comum; `spawn` prepara apenas nascimentos pedidos. Templates são validados na fronteira de autoria. |
| 2. Alocações/cópias | `templates::expand` reserva slots na construção; `RuntimeWorld::install` substitui uma Entity/placement. Clone de configuração, Strings/BTreeMap/corpo Rapier e compilação de links/rota ocorrem por nascimento; não há clone de Scene/Simulation por spawn/tick. Reservas e mapas não são promessa de zero alocações. |
| 3. Tipos/acesso | Operação Copy com template u8, posição f32 XYZ e Reference numérica; InstanceId/geração não são reinterpretados como endereço. Bases compiladas por template em `attributes::Registry`, copiadas para um dono com `clone_from`. Catálogo de habilidades/regras permanece compartilhado. |
| 4. Trabalho repetido | Volume percorre definições uma vez e consulta handles presentes; evitada busca linear da Entity para cada corpo. Apoio usa consulta existente. Ordem lógica ordena array fixo de até9 índices, sem heap; custo conscientemente mantido em fases/seletores. Facções usam matriz por facção em vez de recompilar relações entre todos os atores. |
| 5. Limites |8 NPCs,16 templates,64 pedidos/tick,4096 entradas,256 recibos. Mapas de navegação/patrulha/passos/mobilidade/participantes removem aposentados; nomes/referências de contato reutilizam slots limitados. Teste com300 gerações confirma mapas e anel limitados. |
| 6. GPU/I/O | Render/picking/inspeção passam a consultar definições de execução; tick invalida frame existente. Sem pipeline/readback periódico novo. JSON permanece somente na autoria/transporte/replay/diagnóstico. |
| 7. Sincronização | Fila serial antes de sensores/habilidades; não há thread/lock/dependência nova. Ordem lógica por indivíduo evita prioridade acidental de slot reciclado. Revalidação de alvo ocorre antes de consumir identidade. |
| 8. Invalidação | Instalação limpa explicitamente estado por ator; referências velhas não controlam o ocupante novo. Contexto de origem capturado antes de substituir bases/facção; contato encerrado conserva referência antes do reuso. Replay admite comandos no mundo de cada tick, não apenas na população inicial. |
| 9. Correção | Testes incluem mesma/diferente configuração, cooldown/recursos/buffs/vida, alvo periódico antigo, cast sobrevivente, fonte paga destacada,300 gerações, ordem lógica, ocupação/apoio/voo/passagem, patrulha/passos, reset/undo/erros. Evidências de Rust/MCP/replay/captura em validation.md. |
| 10. Custos mantidos | Construção clona configurações da cena; autoria reutiliza validação por template com probes temporários, fora do tick. Planos transacionais ainda copiam arrays globais e atributos ainda varrem pools. Ampliar população exige os trabalhos de índice/plano local já registrados; sem profiling, ganho de FPS ou promessa601 atores. |

## EAF-67 — revisão do código implementado

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | `Engine::queue_despawn` resolve token/alias na admissão; `Simulation::apply_population` recebe slice vazio nos ticks sem pedidos. Revalidação por referência antes de retirar. |
| 2. Alocações/cópias | Entry/Receipt Copy; anel pré-alocado256, log cresce até4096. Nenhum clone de Scene/Simulation por retirada. Captura de origem reutiliza reserva e buffers do EAF-63; remoção de BTreeMap/Rapier é esparsa, sem promessa de zero alocações internas. |
| 3. Tipos/acesso | Actor u8 + Reference numérica; request sequence u32 limitada pelo log. Alias textual não é resolvido novamente no consumo. Sobreviventes conservam slots e identidade. |
| 4. Trabalho repetido | Um passe pela fila no início do tick. Limpeza normal de buffs/periódicos/áreas continua nos pools limitados existentes; sem varrer históricos para manter origem. Seleção ausente é descartada após o step. |
| 5. Escala/limites |64 pedidos/tick,4096/log,256 recibos;9 atores e pools preservados. Anel testado com1000 registros; limite de fila/ordem/replay testado. Não há crescimento proporcional a um histórico ilimitado. |
| 6. GPU/I/O | Sem pipeline/readback novo por tick. Frame já filtra presença; captura é explícita. Inspeção/snapshot aloca listas/JSON sob demanda e passa a incluir população. |
| 7. Sincronização | Mesmo dispatcher/thread/fase; sem dependência, lock ou scheduler novo. Ordem da fila explícita, anterior às habilidades, mesmo quando estas foram enfileiradas antes. |
| 8. Invalidação | Registro invalidado antes dos sensores; corpo/mapas/passagem/seleção retirados. Memória não recebe evento global. Triggers distinguem removed; origem retida e histórico conservam identidade. Replay50 inclui fila e recibos. |
| 9. Correção | Casos de fila pausada/duplicada, cast pendente/sobrevivente, fonte paga com prazo de buff, cadáver, memória, trigger/passagem, ausência em todos os exemplos e replay adulterado. MCP cobre origem/replay/prefixo/captura/reset; evidência final em validation.md. |
| 10. Custos mantidos | Metadados de contato/rotas e arrays inativos limitados à cena permanecem; spawn futuro precisa instalar/inicializar e ordenar indivíduos por InstanceId. Atributos globais/Plan continuam com custos já registrados. Sem profiling ou ganho de FPS alegado. |

## EAF-67 — avaliação antes de implementar

Fila limitada64 pedidos/tick, log4096/execução e recibos256; payload numérico Copy e resolução de tokens apenas na fronteira. Aplicação antes dos sensores, sem clone de Scene/Simulation nem rebuild de sobreviventes. Retirada captura origem na reserva existente antes de liberar componentes; sem alocação de contexto tardia. Remover mapas/corpo é trabalho esparso; custos Rapier de remoção e mapas existentes são mantidos. Metadados de contatos permanecem limitados às entidades iniciais nesta etapa sem spawn. Replay/inspeção serializam sob demanda. Revisar ausência em consumidores/overlays, fila duplicada, pausada, limites, identidade, origem/prazos, histórico/trigger e prefixos. Não aumentar população/pools nem alegar ganho de FPS; templates/nascimento são próximo incremento.

## EAF-63 — revisão do código implementado

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | RuntimeWorld compila bindings uma vez por construção/reset. Presença é conferida nas consultas; captura só na retirada. Avaliação viva/destacada usa o mesmo solver de atributos. |
| 2. Alocações/cópias | Pool32 pré-reserva64 bases/64 contribuições por origem:232448bytes de payload no aarch64 observado (Slot96, base24, contribuição88). Captura reutiliza buffers; teste compara ponteiros antes/depois. Sem Scene por entrega. A cópia de Scene por Simulation já existente passa a RuntimeWorld. |
| 3. Tipos/acesso | Definição u8 e lease Option<u8> no Cast; lease validada também pela Reference original. Facções numéricas/matriz32² compiladas. Reuso de slot não altera atributos/vida/definição da origem antiga. |
| 4. Trabalho repetido | Reserva busca até32 slots; coleta percorre pools64/64/16 e máscara u32, duas fronteiras de fase para liberar antes de novas admissões e após commits. Sem varrer históricos. Mantidas comparações de identidade/revalidação por segurança transacional. |
| 5. Escala/limites | Pool32 independente de9 atores; saturação recusa antes de custo/cooldown. Não aumenta população. Capturas de origem limitadas64 contribuições; atributos vivos ainda percorrem pool global e Plan ainda copia arrays9×32, conscientemente pendentes de migração. |
| 6. GPU/I/O | Nenhum pipeline/readback novo por tick. Render/picking filtram presença; MCP/JSON/replay/capturas só sob demanda. Snapshot de posições inclui entidades sem corpo, custo limitado512; BTreeMap/String preexistentes continuam no caminho de snapshot. Placements adicionam mapa limitado por execução. |
| 7. Sincronização | Sem dependências, threads, Arc ou locks novos. Máscara de retenção calculada dos pools ativos no mesmo thread e com ordem de fases preservada. |
| 8. Invalidação | Mundo verifica binding original; ausência não volta à autoria. Contexto libera na última entrega, lease antiga rejeita novo ocupante. Morte, ausência, vida e expiração são distintas. Replay49, cena46. |
| 9. Correção | Testes retiram corpos/componentes por seam cfg(test), verificam reuso, expiração, áreas/auras, rollback e saturação. MCP verifica mundo/reservas/reset/replay/prefixo/pixels; resultados finais em validation.md. |
| 10. Custos mantidos | Reserva antecipada ~227KiB/runtime, mapas de posições e cópia das definições no reset; sem alegação de FPS/banda. Apoio/rede são estáticos e spawn/despawn público exige transação/instalação/invalidação futuras. Limite32 não é dimensionamento comprovado para601 atores. |


## EAF-63 — avaliação antes de implementar

Separar snapshot imutável de definições da presença/posição em execução; preservar a cópia por construção/reset existente, sem clonagem de Scene por tick/entrega. Consultas geométricas usam apenas posições presentes, sem ressuscitar coordenada autorada. Contextos de origem limitados e compartilhados por referência numérica, reserva antes de pagar; dados de avaliação capturados na retirada, contribuições com prazos originais, sem Arc/locks por pulso. Definição da habilidade deve acompanhar Cast para não reinterpretar grant de slot substituído. Inspeção/replay somente sob demanda. Preservar limites9 atores e pools atuais; não confundir fundação com spawn público ou ganho de FPS. Testar ausência em consultas/render/picking, contexto/reuso/expiração/mutação proibida/capacidade/limpeza/replay e regressão completa. Revisar cópias e scans do novo contexto antes de concluir; planos locais e indexação de contribuições não serão presumidos concluídos.

## EAF-61 — revisão do código implementado

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | `Observation::current_actor`, `Catalog::matches/entity_matches` conferem identidade nos consumidores; `Registry::resolve` é acesso por slot, sem busca textual. `Reference::id` só em inspeção. |
| 2. Alocações/cópias | Stamps Copy ampliam Instance/Observation/Receipt e seus anéis. Layout conserva slot u16 + geração/serial u64 com padding nativo; Option não usa sentinel inventado. Sem String/clone de Scene por buff, pulso ou identidade histórica; contatos mantêm strings preexistentes. |
| 3. Tipos/acesso | Índices u8/u16 continuam aceleradores, nunca prova de identidade persistente. Buff owner/source e responsáveis guardam referências; life_index separado. IDs externos hex só na fronteira; inspecionar aloca deliberadamente. |
| 4. Trabalho repetido | Avaliação de atributos confere dono dentro do scan já existente; regra não coincidente pula antes do scan. `actor_instances` é atualizado uma vez por step em array9 para recibos sem cast. Trigger reutiliza o catálogo comum; compilação de corpo/trigger reutiliza índice enumerado, sem nova busca por ID. Amostragem de posição ignora binding ausente. |
| 5. Escala/limites | Pools/anéis mantêm capacidades anteriores (buff64/ring256, periódico64/ring256, projétil64/ring256, área persistente16/ring128, trigger512). Stamps aumentam custo limitado; transações ainda copiam arrays globais e atributos ainda varrem pool. Indexação por dono/chave e planos locais seguem necessários para aumentar população. |
| 6. GPU/I/O | Nenhuma mudança de render/pipeline/readback por tick. JSON/replay48 e captura seguem operações explícitas; cena46 preservada. Inspeção/prefixo comprovam proveniência sem exigir janela. |
| 7. Sincronização | Sem locks/threads/dependências novas; ordem de fases e compactação estável preservadas. Identidade é comparada como campos escalares. |
| 8. Invalidação | Refresh confere dono/origem; limpeza não testa vida do substituto. Reconhecimento compara indivíduo e vida; memória oculta preserva pista. Rádio/ruído não transferem identidade; históricos resolvem nome só pelo stamp. Triggers saem do binding antigo sem adotar substituto. |
| 9. Correção | Testes reciclam o registro real por função cfg(test) e cobrem buffs, reconhecimento, rádio, som, histórico, periódico e trigger. MCP verifica replay/prefixo/PNG e vida separada. Resultados e ajustes de fixtures em validation.md. |
| 10. Custos mantidos | Mais bytes por registro e comparações extras são o tradeoff de validade temporal. Reuso público, visão do mundo, retenção/reserva de autor e escala permanecem posteriores. Sem profiling, throughput ou ganho de FPS alegado. |

## EAF-61 — avaliação antes de implementar

Vínculos persistentes ganham stamps Copy; índices continuam aceleradores conferidos antes do uso. Buffs e históricos mantêm pools/aneis limitados; sem strings/clones de Scene por evento. Inspeção resolve nome somente se a identidade ainda coincidir; ID lógico congelado permanece disponível. Percepção preserva pista congelada, mas reconhecimento/vida/radio/alvos vivos verificam indivíduo. Sem limpeza global de memória por ausência oculta. Custo: structs maiores e comparações escalares extras; sem promessa de zero alocação em inspeção, indexação por dono ou planos locais neste incremento. Registrar layout/limites e testar stale stamps, morte/retorno, anonimato, replay e MCP completo antes de concluir. Catálogo vivo não recicla até migrar mundo presente e contextos de origem.

## EAF-88 — tangente compacta e projeção de fragmento

Protocolo prévio, checklist e resultados em [rich-relief-study.md](rich-relief-study.md#compactação-e-custo-da-tangente--resultados). IGP1 guarda4B/face+4B de escala, sem expansão no upload;9→4,5MiB de tabela,55,934→51,434MiB de payload isolado. Candidatos138/138vistas,128/128repetições e10/10retornos. Mediana próxima3456 reduz2,33% contra tangente anterior, mas só5/8rodadas e dispersão2,92%; ainda11,23% acima do original. Sem promoção/ganhoFPS. Guardas com discard não isolam custo puro do índice; próximo controle precisa remover esse efeito.722testesRust/Clippy/fmt e13gruposMCPheadless passam; referência original mantém falha estática, dados e gate reprovado preservados.

## EAF-88 — hierarquia de certificados locais

Protocolo antes da implementação, formato e resultados em [certificate-hierarchy-study.md](certificate-hierarchy-study.md). Árvore espacial binária compilada em ICH1, validada contra o pacoteIGC1 e percorrida sem alocação por frame. Mantém geometria, shaders e limites; orçamento128 e falha de folha conservam a fonte. Mede o custo de seleção dentro do lote.

## EAF-88 — segunda malha e integração por pixel

Checklist e medições em [external-recovery-study.md](external-recovery-study.md). Import binário de dois primitives do banco; compilação recupera a superfície, mas bound global provoca fallback excessivo100× perto nativo. Consulta local é hipótese seguinte. Jitter/readbackROI apenas diagnóstico offline, shader intacto; fontes e seletor incluídos na medição.

## EAF-88 — bordas assimétricas offline

Protocolo prévio, checklist posterior e medições em [border-recovery-study.md](border-recovery-study.md). Reunião opt-in de caminhos colineares duplicados no grafo de bordas, com erro acumulado e restauração por regiões nativas; nenhuma operação nova no frame. Seleção conserva pacotes sem redução.

## EAF-88 — campo de normais offline

Checklist antes/depois e medições em [normal-field-study.md](normal-field-study.md). Atlas planar desacopla normais da triangulação; grades 64/128 passam nos gates do ensaio, mas redução geométrica de apenas 2,73%, memória e até 193 draws impedem ganho líquido. Manter experimental; nenhum contrato funcional alterado.

## EAF-88 — regiões em malha externa

Checklist antes/depois, custos medidos e decisão de não promover o caminho em [irregular-lod-study.md](irregular-lod-study.md). Build regional offline com limites de superfície/normais; seleção CPU e buffers residentes. Pixels preservados com redução insignificante, memória maior e nenhum ganho confiável; sem mudanças funcionais.

## EAF-88 — compilador de geometria sem genealogia

Protocolo prévio e respostas posteriores aos dez itens em [geometry-compiler-study.md](geometry-compiler-study.md). Adjacência, regiões e certificados são offline; no frame, projeção de bound e escolha entre buffers residentes. Medição distingue GPU, CPU, parede, arquivo, buffers e RSS. Fonte de fallback e validação no load são custos mantidos explicitamente; sem mudanças nos contratos funcionais.

## EAF-60 — revisão do código implementado

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | `Catalog::new` atribui referências na construção/reset. `Catalog::matches`/`Registry::resolve` conferem arrays/slot/geração/serial antes de ações e efeitos; não percorrem entidades por pulso. |
| 2. Alocações/cópias | Registro pré-aloca512 slots; tabela de referências fixa por catálogo. `Reference`/`LifeStamp` Copy aumentam entradas/casts e os pools que os conservam, deliberadamente. Nenhuma alocação/clone/string na resolução numérica. `snapshot` clona o registro somente ao gerar/comparar replay explícito. |
| 3. Tipos/acesso | Slot u16, geração/serial u64 com overflow verificado; aposentadoria impede wrap. Serial/generation externos em hex evitam perda de precisão JavaScript. Parsing/token só na entrada MCP/dispatcher; resolução externa linear limitada aos9 atores atuais. |
| 4. Trabalho repetido | Construção ordena entidades e inserção verifica duplicatas/slot livre: custo O(E²) limitado a512, fora do tick. Mantido neste incremento; não replicar esse padrão por nascimento ao escalar. Validação na entrada não substitui revalidação no consumo/commit. Comparação do registro em replay calculada uma vez. |
| 5. Escala/limites | Sem mapas históricos crescentes; reuso isolado testado10000 vezes. Pools e9 atores mantidos. Atributos ainda varrem buffs globais e Plan ainda copia estado global limitado: indexação por dono/chave e planos locais são requisitos seguintes, não ganhos já entregues. |
| 6. GPU/I/O | Render/cache/GPU não mudaram; inspeção e replay serializam sob demanda. Capturas offscreen só na verificação. Sem I/O de identidade por tick. |
| 7. Sincronização | Nonce atômico relaxed e OnceLock só ao criar escopo de execução; nenhuma trava/atomic no loop de efeitos. Sem dependência/thread nova. Ordem lógica inicial por ID autorado. |
| 8. Invalidação | Escopo troca em reset/edição/load bem-sucedidos, não em erro ou ressurreição. Vida e indivíduo separados. Contatos novos recebem referência corrente. Catálogo não recicla antes de migrar consumidores legados; não prometer segurança de despawn ainda inexistente. |
| 9. Correção | Testes de registro/reuso/overflow, tokens/atomicidade/cast/cancel/vida/replay; cenário identity usa dispatcher MCP e compara pixels. Resultados finais em validation.md. |
| 10. Custos mantidos | Stamps e catálogos maiores; não otimizar layout por suposição nem aumentar limites como prova de escala. Contexto de origem exigirá reserva na admissão das entregas. Nenhum profiling, ganho de FPS ou capacidade601 alegado. |

## EAF-60 — avaliação antes de implementar

Registro limitado e pré-alocado por execução; resolução numérica de geração/identidade no caminho de habilidades, tokens textuais só na fronteira MCP. Sem clone de Scene por referência ou mapa histórico crescente. Stamps Copy aumentam Cast/Entry deliberadamente; layouts/limites e regressões serão verificados. IDs lógicos determinísticos ficam separados do escopo externo, sem UUID de transporte no replay comparável. Nenhuma reciclagem no mundo antes de migrar índices de atributos, percepção, ruído, contatos e históricos; reuso é validado em registro isolado. Sem mudança de render/GPU, dependência ou threads. Validar tokens obsoletos/reset/edição/cancel, identidade versus vida, erros atômicos e replay. Crescimento de população exigirá transações por campos tocados, atributos indexados por dono e reserva de origem na admissão de entregas; não aumentar arrays/pools e chamar isso de escala validada.

## EAF-59 — estudo de instâncias/população, antes/depois

1. Frequência: só inventário/documentação; nenhuma resolução nova por tick executada. Proposta separa comandos esparsos de loops de simulação.
2. Alocações/cópias: sem mudança; proposta evita clone de Scene por spawn/efeito e prevê contexto de origem compartilhado/limitado. Payload real ainda precisa ser medido por layout na implementação.
3. Tipos/acesso: auditados slots u8, índices u16 de entidade e índices locais de trigger. Handle/InstanceId/LifeIndex são proposta, sem ABI ou layout aprovado.
4. Repetição: substituir índices não basta; documentada extração de visão comum do mundo para evitar reconstrução/global reset e consultas inconsistentes.
5. Escala: limite atual9 atores mantido na primeira migração proposta; crescimento exige revisar máscaras/pares/scratch. Contextos não podem crescer com histórico.
6. GPU/I/O: código/cache/captura inalterados; fronteira RenderFrame marcada para coordenação gráfica. Sem build ou cópia de artefato.
7. Sincronização: fase antes dos sensores proposta; sem thread/lock/ECS novo. Ordem de slots livres não deve governar gameplay.
8. Invalidação: matriz inclui autor removido, destinatário/vida, slots reciclados, ruído, buffs, histórico, seleção, contatos e reset da execução.
9. Correção: links e símbolos do inventário revisados, diff validado. Matriz futura cobre stale handles/atomicidade/ocupação/privacidade/replay; não declarar testes de spawn ausente.
10. Custos mantidos: migração transversal é mudança estrutural; contexto de origem exige política explícita de expiração e pool. Sem profiling/FPS alegados; alternativas registradas em population-runtime-study.md.

## Coordenação da frente gráfica — 25/10/2026

Antes/depois: incremento apenas documental (graphics-handoff, orientação AGENTS e roadmap). Sem código, formato, dependência, build ou caminho quente alterado. Alocações/cópias, tipos/layout, repetição, escala, GPU/I/O, sincronização e invalidação: sem mudança. Links locais e diff verificados; não repetir testes/build por esta edição. Custos futuros de malhas/materiais/luzes serão avaliados antes de implementar; os573 testes/218 grupos anteriores não validam alterações futuras.

## EAF-58 — revisão do código implementado

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | `Simulation::step` consome máscaras apenas nos atores que mudaram de vida, catálogo limitado a9. Percepção volta a sondar cadáveres conhecidos dentro dos limites sensoriais existentes; custo adicional explícito. |
| 2. Alocações/cópias | `LifeStamp` Copy, vidas/máscaras em arrays fixos, novo anel256 pré-alocado em vitality. Nenhum JSON/clone de cena no efeito. Retorno raro recria NpcState (incluindo strings); `mobility::State::revive` limpa Vecs preservando capacidade. Não alegar zero alocações em toda a transição. |
| 3. Tipos/acesso | Slots u8, máscaras u16, contador u64 com checked_add e validação anterior ao commit. Cast/Observation ficam maiores deliberadamente; todos os pools permanecem limitados. IDs textuais são resolvidos fora da aplicação do efeito. |
| 4. Trabalho repetido | Estado vital revalidado antes de pagar; referências de vida evitam executar intenção obsoleta. Não há rebuild da Simulation, catálogo, rigid body ou collider. Revalidação no commit é necessária mesmo após elegibilidade sensorial. |
| 5. Escala/limites | Até9 atores, pools existentes e256 recibos de retorno. Reavaliação visual não cria população dinâmica nem índice global; custo linear limitado aceito neste laboratório. |
| 6. GPU/I/O | Render reutiliza cor/vida/cast/guias existentes; sem pipeline ou readback adicional por tick. JSON, captura e replay em operações explícitas de diagnóstico. |
| 7. Sincronização | Nenhuma dependência/thread/lock nova. Intenções preservam vidas do snapshot sensorial; commits seguem ordem existente de fases/IDs. |
| 8. Invalidação | `life_stamp`, máscaras de morte/retorno e limpeza diferida dos pools distinguem referências antigas de novos contatos físicos. Formato46; reset/edição/replay recompõem autoria, ressurreição preserva saldos/deadlines. |
| 9. Correção | 573 testes Rust,53 cenários/218 grupos MCP, fmt/clippy/build aprovados. Inclui concorrência, rollback tardio, mortes repetidas/no mesmo tick, voo, ocultação, rádio antigo, limite do anel, replay/prefixo e pixels idênticos. Evidência em validation.md. |
| 10. Custos mantidos | Sem spawn, teleporte, triagem/aproximação a cadáveres ou novo controlador. Fatores de dano de entregas sobreviventes continuam dinâmicos pelo mesmo autor; congelá-los seria outro contrato. Nenhum ganho de FPS alegado. |

## EAF-58 — avaliação antes da implementação

1. Retorno é transição esparsa; manter catálogo/corpo, sem reconstruir Simulation. Sondagens visuais de mortos conhecidos usam os mesmos limites de nove atores.
2. Contadores de vida e máscaras em arrays fixos; recibos de retorno em anel limitado. Nenhum mapa de gerações crescente, JSON ou clone de cena por efeito.
3. Referências compiladas recebem epoch u64; incremento verificado sem wrap. Strings continuam na autoria/inspeção.
4. Vida revalidada no commit; efeitos persistentes distinguem vínculo antigo de colisão/ocupação nova. Limpeza sem mutação dos pools enquanto iterados.
5. Preservar pools/limites atuais; tamanho de Cast/Observation aumenta deliberadamente para validade temporal. Não prometer escala de população dinâmica.
6. Reusar barras/guias/cor de cadáver; sem pipeline/GPU/I/O novos por tick.
7. Preservar ordem determinística de fases/IDs, sem dependências, threads ou locks.
8. Morte/retorno no mesmo tick exigem máscaras de transição; invalidar ações antigas e rearmar morte física. Conhecimento de terceiros só por evidência.
9. Testar atomicidade, concorrência, preservação de saldo/prazos, mortes repetidas, entregas persistentes, ocultação, rádio, mobilidade e MCP/replay/undo/PNG.
10. Sem spawn, teleporte ou triagem de cadáveres; custo sensorial adicional e metadados são tradeoffs aceitos para o contrato aprovado. Sem profiling/FPS alegados.

## EAF-56 — revisão do código implementado

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | `matches_effects` só consulta quando require_dispel_match está ativo e `Decision::eligible` já passou; visão/relação/vida/alcance precedem o filtro. |
| 2. Alocações | `DispelSelection` contém duas máscaras u64 na stack; `DispelQuery`/iteradores sem coleção temporária. Nenhum clone de cena, sort ou JSON no caminho. |
| 3. Tipos | bool compilado na política, índices u8 e máscaras u64. Soma de correspondências u16 suporta até8×64; strings só na autoria/inspeção. |
| 4. Repetição | `select_dispel` compartilhado pelo plano e política. Categoria agora é testada antes da consulta de proteção efetiva, evitando avaliar atributos de categorias incompatíveis. Revalidação no commit é deliberada. |
| 5. Escala | Limite até8 objetivos×9 candidatos×8 operadores×128 instâncias, mais avaliação existente de atributos para periódicos compatíveis. Cast comprometido consulta só seu candidato. Scans limitados mantidos; não é promessa de multidões ou ganho de FPS. |
| 6. GPU/I/O | Nenhum pipeline/readback/recurso visual novo; demo usa guias existentes. I/O apenas nos comandos de autoria/inspeção/captura. |
| 7. Sincronização | Sem threads/locks/dependência. Intenções conservam snapshot; consulta antes de pagar observa transações anteriores da fase. |
| 8. Invalidação | Sem cache/reserva nova; consulta corrente, máscaras locais, reset/edição/replay existentes. Buff expirado é filtrado pelo tick antes da limpeza do pool. Formato45, false padrão preservado. |
| 9. Semântica | Cobertura em dispel_triage.rs/verify_dispel_triage.py, incluindo proteção dinâmica, máscaras mistas, status sem vida, cast/custo/impacto/ocultação e legado. 557 testes Rust e52 cenários/215 grupos MCP passaram; evidências finais em validation.md. |
| 10. Custos mantidos | Filtro não simula demais operadores/capacidades nem escolhe severidade; executor continua atômico. Duplicação de scan no commit evita stale state sem invalidadores extras. |

## EAF-56 — avaliação antes da implementação

1. Filtro opt-in após evidência e elegibilidade básica, antes do ranking; até9 candidatos por objetivo e8 operadores por habilidade.
2. Reutilizar seleção de dissipação com duas máscaras u64, sem Vec/sort/JSON/clone no tick.
3. Categorias e atores compilados, bool na política; proteção efetiva consulta o avaliador existente.
4. Mesmo seletor no planejamento e na decisão; revalidar no commit sem cache sensorial extra.
5. Scans limitados dos pools64 periódicos/64 buffs; custo multiplicativo por objetivo/candidato aceito e explícito, sem promessa de multidões.
6. Sem render/GPU/I/O novo; guias e inspeção existentes.
7. Snapshot/ordem de fases atuais, sem locks/threads/dependências.
8. Sem reservas persistentes; expiração/dissipação/proteção invalidam no próximo sensor e no commit. Versão45, defaults legados.
9. Cobrir proteção/categorias/buffs/múltiplos operadores, ocultação, cast/retry/custo, impacto, replay/undo/erros e regressão completa.
10. Não é detecção de doenças nem reserva entre curadores; consulta de status após visão é o contrato existente. Sem profiling/FPS alegados.

## EAF-55 — respostas sobre o código implementado

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | `observe_support` só percorre aliados quando há objetivo opt-in; uma passagem por NPC/tick, compartilhada entre objetivos. `choose_ally` compara até9 candidatos por objetivo. |
| 2. Alocações | Support usa arrays9 e Knowledge fixo; iteradores/Option e decisão Copy, sem Vec/sort/clone de cena. IDs/JSON são resolvidos apenas por inspect_state. |
| 3. Tipos/localidade | Enum Ranking, índices u8, posições f32, produtos i128 para frações até1e12. Sem string/reflexão no ranking. Novo estado fixo por NPC, inclusive sem triagem, aceito no laboratório. |
| 4. Repetição | Falta de recursos retorna antes do ranking. Distância² é calculada na elegibilidade existente e no ranking; custo pequeno limitado, mantido para preservar avaliador comum. Aliado também alvo explícito repete sensor, mas trilhas independentes não duplicam exposição. |
| 5. Escala | Até9 observações por NPC opt-in e8×9 avaliações; oclusão continua linear nas entidades. Não adiciona broadphase nem promete escala de multidões. |
| 6. GPU/I/O | Usa guias/barras de cast e rotas existentes; nenhum pipeline/readback/I/O por tick. Capturas só sob comando MCP. Verificador Python lê orçamento de initialize uma vez antes de spawn, default120/opt-in até600s; não afeta tick ou budget de comandos. |
| 7. Sincronização | Mesmo snapshot pré-física; intenções antes dos efeitos, limiar revalidado no commit. Sem threads/locks/dependências. |
| 8. Invalidação | Observações limpas a cada tick; conhecimento decai e limpa no retorno/disable/morte. Edição/reset/undo/load recompilam; versão44 serializa estado no replay. Client.close agora reaproveita o handle próprio para terminate/wait/kill limitado após falha, sem procurar/encerrar processos do usuário. |
| 9. Semântica | Testes em triage.rs e verify_triage.py cobrem ranking, conhecimento, compromisso/cancelamento, revalidação, aproximação, formato/edição e replay/captura. 543 testes Rust e 51 cenários/211 grupos MCP passaram; evidências finais em validation.md. |
| 10. Restante | Caminho não entra no ranking; aproximação reavalia, cast compromete; não há reserva coletiva de cura, observação de status ou lesão oculta. Nenhuma alegação de FPS ou profiling. |

## EAF-55 — avaliação antes da implementação

1. Observar aliados uma vez por NPC/tick somente quando há política opt-in; no máximo9 atores, reutilizando sensores XYZ/oclusão existentes.
2. Evidências e reconhecimento em arrays fixos, seleção por scan sem Vec/sort/strings no hot path; JSON só na inspeção.
3. Ranking de fração vital com produto i128 (valores até1e12), distância quadrática f32 e desempate por ID compilado.
4. Compartilhar observações de apoio entre até8 objetivos. Reconhecimento de apoio separado do combate evita dupla acumulação ou alteração de memória; aceita consultas sensoriais duplicadas quando alvo explícito também é aliado.
5. Custo limitado: até9 testes sensoriais por NPC mais8×9 avaliações simples; sem broadphase ou alegação de multidões.
6. Reutilizar guias de cast/aproximação existentes; nenhum pipeline, captura ou I/O automático novo.
7. Mesmo snapshot pré-física e executor; sem thread/lock/dependência.
8. Limpar evidência corrente em cada tick/disable/morte/retorno; reconhecimento tem decay. Versão44 e replay incluem estado; edits/undo/reset recompilam.
9. Testar informação oculta, reconhecimento, ranking/empate, cast estável/cancel, recursos/condição no commit, aproximação, morte/território, MCP/replay/erros.
10. Triagem não reserva curas entre NPCs e não conhece status ocultos; cast fixa destinatário, aproximação pode reavaliar ranking. Sem profiling/FPS alegados.

## EAF-54 — respostas sobre o código implementado

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | **Adequado:** `State::sync_centers` percorre até16 instâncias depois da física; só source consulta corpo. `advance_areas` resolve posição do snapshot apenas nos pulsos devidos; nenhuma seleção contínua de ocupantes. |
| 2. Alocações | **Adequado:** callback genérico temporário usa handles/corpos existentes; sem novo mapa, Vec, clone de Scene ou string por passo. Pool/rings anteriores reutilizados. JSON aloca somente em inspeção. |
| 3. Tipos/localidade | **Adequado:** enum Anchor compilado, índice de ator u8 e centro f32XYZ. Campo novo aumenta configuração/estruturas limitadas; sem referência a entidade instável ou parsing no hot path. |
| 4. Repetição | **Mantido com motivo:** resolve centro no snapshot do pulso e após física por fases distintas. Várias auras no mesmo portador repetem lookup de handle limitado a16; não adiciona cache/invalidação para evitar esse custo pequeno não medido. |
| 5. Escala | **Adequado ao recorte:** O(16) testes de modo + lookups de handles existentes; budgets de9 candidatos/8 operadores e oclusão anteriores. Sem broadphase/ECS/scheduler novo ou promessa de população massiva. |
| 6. GPU/I/O | **Adequado:** mesmo limite16×96 linhas de áreas ativas; cor distingue âncoras e deslocamento invalida cache existente. Nenhum pipeline/readback/textura/I/O automático adicional. |
| 7. Sincronização | **Adequado:** após Rapier, no mesmo passo/dispatcher; callback sem estado persistente/thread/lock. Recibo de pulso conserva snapshot aplicado e não é reescrito pela sincronização visual. |
| 8. Invalidação | **Adequado:** fixed default/migração preservados, source opt-in43; morte usa política existente, edição/reset/undo/load recompilam e replay inclui centros sincronizados. Sem novo pool ou ring. |
| 9. Semântica | **Validado:** movimento real, voadorXYZ, centro pré/pós-física, cadência/refresh/independent, cadáver, oclusão/aliados/buffs após saída, erros/legado e replay; MCP verifica demo, undo/load, prefixo e PNG idêntico. Evidências finais em validation.md. |
| 10. Restante | **Mantido com motivo:** lookup BTreeMap por ID emprestado ainda existe no adaptador físico; limita-se às âncoras e não cria strings. Auras pulsantes não revogam buffs ao sair; direct+self_actor evita semântica ambígua de alvo/impacto. Sem profiling/FPS alegados. |

## EAF-54 — avaliação antes da implementação

1. Movimento da âncora atualiza somente centro de até16 instâncias após física; seleção física continua somente no prazo do pulso.
2. Reusar pool/rings/array de candidatos existentes, sem clone de cena/mapa/coleção por atualização. Closure de posição usa handles existentes.
3. Enum compilado fixed/source e índice de ator u8; posição XYZ f32, sem string/reflexão no contrato runtime além dos lookups de handles atuais.
4. Resolver centro no pulso a partir do snapshot e sincronizar apresentação após física; recibo conserva centro aplicado, evitando consultar/renderizar posições de fases diferentes como se fossem iguais.
5. No máximo16 consultas de âncora por passo; seleção/oclusão/efeitos mantêm budgets anteriores. Não introduzir percepção/consulta contínua de ocupantes.
6. Reutilizar guias/cache de linhas; mudança física invalida geometria visual existente, sem pipeline/readback/I/O novo.
7. Mesmo dispatcher/clock; sem threads/locks/dependências. Source só em entrega direta self_actor evita ponto/impacto ignorados.
8. Defaults fixed para cenas antigas; versão43 para source. Reset/load/undo/replay e morte conforme política precisam manter âncora/centro coerentes.
9. Validar movimento/XYZ, pulso pré-física versus guia pós-física, morte/corpo, custo/cadência/refresh, relação/oclusão, NPC e replay.
10. Aura pulsante não é buff condicional contínuo: sair evita aplicações futuras, mas não revoga contribuição já instalada. Sem profiling/FPS alegados.

## EAF-53 — respostas sobre o código implementado

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | **Adequado:** `advance_areas` consulta prazos de até16 instâncias por tick; `pulse_area` só seleciona ocupantes no prazo/inicial. Nenhum scan de geometria entre pulsos. |
| 2. Alocações/cópias | **Adequado:** pool16 e ring128 reservados uma vez por runtime; instâncias Copy, array9 e plano fixo anteriores. Nenhuma coleção/string/JSON criada por pulso. Inspeção aloca sob demanda; runtime reserva pools mesmo em cenas sem esse recurso, como agendas anteriores. |
| 3. Tipos/localidade | **Adequado:** ator/definição/slot compilados u8, prazos u64, contador u32 e posições f32. Valores de dano preservam cálculo inteiro existente; configuração Copy anexada à área. |
| 4. Repetição | **Mantido com motivo:** seleção/ordenação única por pulso; capacidade direta revalidada antes do custo e instalação. Limpeza antes/depois dos pulsos e ao final do passo faz scans16 limitados para morte/expiração sem índices obsoletos. Pode ser simplificada com geração de mortes se uma medição justificar. |
| 5. Escala | **Mantido com motivo:** máximo16×9 transações por tick,8 operadores cada, além de oclusores/atributos. Sem broadphase/scheduler novo. Refresh e capacidade percorrem pool limitado; sem alegação de escala MMO. |
| 6. GPU/I/O | **Adequado:** até16×96 linhas de zonas vivas, mais limites existentes de previews/instantâneas; filtros evitam esfera extra por pulso. Circle LazyLock/cache/buffers anteriores, nenhum pipeline/I/O/readback no tick. |
| 7. Sincronização | **Adequado:** mesmo dispatcher/relógio/fase; sem lock/thread/dependência nova. Pulso não instala área recursivamente, e limpeza/compactação não ocorrem durante iteração de pulsos. |
| 8. Limites/invalidação | **Adequado:** pool16, ciclo de vida128, resultados64 e recibos individuais256 com descarte explícito. Refresh mantém ordem; expiração/morte liberam vaga; reset/edição/undo/replay recriam estado. Formato42. |
| 9. Contrato | **Validado:** testes focados de fronteiras, movimento real, sobreposição, capacidade no commit/impacto, origem morta, buffs vigentes, NPC e replay; cenário MCP cobre autoria/erros/undo/load/prefixo/PNG. Evidências completas em validation.md. |
| 10. Restante | **Mantido com motivo:** memória fixa adicional e cópia de instâncias/planos; samplings deliberados, centro fixo e transações sequenciais. Sem profiling ou promessa de FPS; enter/exit contínuos, aura móvel e dispel de volumes exigem outro contrato. |

## EAF-53 — avaliação antes da implementação

1. Scan limitado a16 prazos por tick; seleção física somente em pulsos devidos, sem callback contínuo de entrada.
2. Instâncias Copy, pool16 e ring128 pré-reservados; candidatos em array9 e planos transacionais existentes. JSON somente sob inspeção.
3. Índices compilados e deadlines u64; posições f32. Nenhum lookup textual no pulso.
4. Seleção/ordenação uma vez por pulso, consulta de geometria não repetida por operador; círculo unitário visual já compartilhado.
5. Pior caso16×9 destinatários×8 efeitos por tick mais oclusores; limite explícito do laboratório, sem promessa de multidões.
6. Guias ativos limitados a16×96 segmentos; reutilizar linhas/cache, sem pipeline, I/O ou captura automática.
7. Mesmo dispatcher e relógio determinístico, sem locks/threads/dependências novos.
8. Expiração, refresh, morte, edição/reset/undo e replay precisam invalidar instâncias. Capacidade direta antes do custo; projétil verifica no impacto.
9. Testar fronteiras de pulso/duração, ocupantes atuais, custo único, transações individuais e conhecimento limitado do NPC.
10. Amostragem por pulso e centro fixo são escolhas deliberadas; auras móveis/callbacks de entrada ficam fora. Sem profiling ou alegação de FPS.

## EAF-52 — respostas sobre o código implementado

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | **Adequado:** `area::select` somente no commit/contato. Preview visual é separado da consulta física; NPC não passa a consultar agrupamentos por tick. |
| 2. Alocações | **Adequado:** array9 `Option<Candidate>`, sort_unstable sem heap e ring64 pré-reservado. `Plan` existente reutilizado por alvo; nenhum clone de Scene, JSON ou strings no caminho de aplicação. Inspeção aloca sob demanda. |
| 3. Tipos/localidade | **Adequado:** atores u8, índices/máscaras compilados, f32 espacial, dano i128 preservado. `Area` Copy no catálogo; recibos numéricos e limitados. |
| 4. Repetição | **Corrigido:** seleção/ordenação uma vez por entrega, não por efeito; distâncias² sem sqrt. Círculo unitário LazyLock32 evita trigonometria por esfera/frame. |
| 5. Escala | **Mantido com motivo:** até9×corpos para oclusão e9×8 efeitos; planos copiam balances/vitalidade como antes por destinatário. Scans de atributos preservam snapshot por transação e ordem; índice espacial/plano coletivo não introduzidos sem necessidade demonstrada. |
| 6. GPU/I/O | **Adequado:** guia usa LineGuide/cache existentes; últimas8 áreas +até9 preparações,96 segmentos/esfera (máx1632 adicionais), duração60ticks. Sem pipeline/upload/readback novo; PNG somente sob comando. |
| 7. Locks | **Adequado:** execução no dispatcher existente, sem threads/locks adicionais. Captura/I/O continuam custo síncrono já documentado. |
| 8. Limites/invalidação | **Adequado:** ring64 extra por runtime, descarte explícito; buffers de guias reutilizados. Reset/edição/replay recriam estado e schema tem versão41. Recibos individuais continuam ring256 e podem expirar antes do resumo. |
| 9. Contrato | **Validado (499 testes Rust,48 cenários/199 grupos MCP; validation.md):** legado sem área, custos únicos, alvos congelados, transações individuais, oclusão de interiores, conhecimento NPC e guias sem efeitos de gameplay. |
| 10. Restante | **Mantido com motivo:** consulta física linear limitada ao laboratório e transações sequenciais; efeito/morte/dissipação em um alvo pode afetar contexto do seguinte. Nenhuma promessa de FPS/profiling, multidões, VFX ou área persistente. |

## EAF-52 — avaliação antes da implementação

1. Área seleciona atores somente na entrega/impacto, sem consulta contínua por frame de gameplay.
2. Candidatos em array fixo de nove atores; recibos em ring pré-reservado64. Sem strings/JSON/alocação por seleção. Reutilizar Plan existente por destinatário.
3. IDs compilados e distância quadrática XYZ; preservar f32 espacial e i128 no dano.
4. Congelar seleção e ordenar uma vez por distância/ID; não repetir query por efeito.
5. Até9 candidatos×corpos para oclusão, até9 transações de8 efeitos. Custo limitado pelo laboratório; sem alegação de escala de multidões.
6. Guias opcionais reutilizam linhas/cache; últimas8 áreas por60ticks e preparações diretas. Capturas/I/O sob demanda.
7. Mesmo dispatcher/lock; nenhuma thread/protocolo novo.
8. Estado/rings pertencem à simulação: reset/edição/undo/replay recriam corretamente. Cap e descarte explícitos.
9. Legado sem área permanece; custo único, falha por destinatário, oclusão, privacidade NPC, versão/replay e guia serão testados.
10. Transações sequenciais por destinatário evitam um plano coletivo muito maior. Efeitos anteriores podem alterar os atributos/capacidade usados pelos seguintes, em ordem documentada. Sem profiling/FPS prometidos.

## Dano causado — antes de codificar (EAF-51)

1. Uma consulta adicional de atributo por componente no commit/preflight/impacto/pulso; nenhuma varredura nova por tick.
2. Origem resolvida fora do loop; índices/máscaras compilados. Mais um i64 por Breakdown fixo (8 por plano/recibo), sem heap no cálculo.
3. Produto amount × dealt × received em i128, divisão única por SCALE². Ambos fatores limitados a100×; soma por efeito até10¹⁶ cabe em i64, teto vital preservado.
4. Reutilizar avaliador/dispatcher e início transacional, sem cache, thread, dependência, GPU ou I/O adicionais.
5. Quarto atributo nativo; preservar61 chaves customizadas (65 totais). Colisão legada com nova chave reservada exige renomeação explícita, sem ativar atributo antes inerte. Cena/replay40, defaults neutros e contexto inspecionável.
6. Cobrir fatores/rounding/limites, filtros, expiração, morte da origem, impacto/pulsos/canal, self-damage, atomicidade, legado, NPC e MCP/replay/captura. Registrar custos finais; sem profiling/FPS.

## Dano causado — respostas da revisão (EAF-51)

| Pergunta | Resposta/evidência |
|---|---|
|1. Frequência|`Runtime::plan_effects` consulta o fator causado por componente/preflight/commit/impacto/pulso. Nenhum novo trabalho periódico global; mantidos8 componentes totais por habilidade.|
|2. Alocações/cópias|`Breakdown` acrescenta um i64; arrays8 e anel256 existentes continuam fixos. Nenhuma alocação no cálculo; JSON com alias recebido só na inspeção.|
|3. Tipos/acesso|Índices u8 e máscaras u64; quarto nativo mais61 personalizados cabem em u8. Produto i128 até10²⁴ antes da divisão; total bruto por efeito<=10¹⁶ em i64.|
|4. Repetição|Origem resolvida uma vez antes dos efeitos, destinatário antes das parcelas. Reutiliza regras compiladas por atributo; nenhum lookup de nome/parse/ordenação em gameplay.|
|5. Escala|Até16 avaliações de dano (duas por parcela), cada uma com regras/instâncias limitadas. Aumento de custo é explícito; não introduz cache de contextos mutáveis antes de medir necessidade.|
|6. GPU/I/O|Sem alteração de render, buffers GPU ou I/O no tick. Mesmas barras; PNG/replay apenas por comandos MCP da regressão.|
|7. Sincronização|Fatores do estado inicial de buffs, uma divisão por parcela e uma alteração de vida por efeito. Sem locks/threads/dependências; instalação/dissipação pendentes não vazam ao cálculo.|
|8. Invalidação|Edição/reset/undo/load recompilam índices. Cena/replay40; colisão de chave personalizada falha antes de mutar. Expiração/morte/dissipação usam ciclo de vida existente, sem cache adicional.|
|9. Contrato|15 testes Rust novos: precisão/limites/aliases, filtros/proprietários, NPC, source, snapshot/rollback, impacto/pulsos/canal, morte, legado. MCP cobre matriz100/70/40→110/80/50, contexto, erros/undo, entrega dinâmica, prazo exato, save/load/replay/prefixo/captura. Resultados finais em validation.md.|
|10. Custos mantidos|Consulta adicional limitada e mais64 bytes de dados por array8 (layout sujeito a alinhamento) aceitos para fator inspecionável. Sem snapshot ofensivo por lançamento; sem profiling/alegação de FPS.|

## Dano composto — antes de codificar (EAF-50)

1. Componentes compilados uma vez em quantidades e máscaras u64; sem nomes/JSON/alocação na resolução. Registro explícito de tags em cena39; migração validada de38 preserva classificações antigas.
2. Até8 componentes de dano no total por habilidade, incluindo dano escalar. Pacotes em armazenamento compilado; plano/recibo compartilham array fixo de8 resultados por execução, evitando multiplicar oito componentes por cada um dos oito efeitos/recibos. Crescimento fixo do anel256 aceito para decomposição inspecionável.
3. Avaliar multiplicador recebido por componente contra buffs iniciais; somar antes da única alteração de vida do efeito. Inteiros/i128, arredondar por componente e limitar total a LIMIT; registrar excesso do teto separadamente de overkill. Custos/efeitos/vida continuam atômicos.
4. Sem novo evento por componente, dependência, thread, I/O ou GPU. Escalar continua válido sem tags; migração converte escalar classificado antigo para pacote explícito. Tags da habilidade não vazam ao dano novo.
5. Cobrir40+60→70, ordem, tags múltiplas versus parcelas, imunidade/vulnerabilidade, limites/arredondamento/rollback/morte, impacto/pulso/canal/NPC, schema/migração/undo/load/replay/prefixo/capturas. Regressores completos headless; sem profiling/FPS alegado.

## Dano composto — respostas da revisão (EAF-50)

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | `plan_effects` avalia no commit/impacto/pulso e nos preflights existentes. Até8 consultas de dano por habilidade, mantendo o mesmo teto de consultas que oito efeitos escalares anteriores. Sem atualização global adicional por tick. |
| 2. Alocações/cópias | Pacotes/Vec de componentes são compilados uma vez; nenhum Vec é criado ao calcular dano. Um array fixo de8 `Breakdown` em Plan/Receipt, compartilhado por todos os efeitos, evita array de8 em cada Applied. Anel256 pré-alocado; aumento fixo de memória e cópias dos recibos aceito para diagnóstico/replay. |
| 3. Tipos/acesso | Tags em máscara u64, packet u16, effect/component u8, quantidade/multiplicador i64 e produto i128. Registro explícito resolve nomes e rejeita referências na autoria. Beneficiário e índice da habilidade resolvidos fora do loop de parcelas. |
| 4. Repetição | Removido total duplicado no pacote compilado; reutiliza amount do efeito. Atribuição do beneficiário e índice de definição fora do loop. Avaliador continua agrupado por atributo com saída rápida sem buffs; JSON/nomes só na inspeção. |
| 5. Escala | Até64 habilidades×8 efeitos; até8 parcelas de dano por habilidade. Soma original de cada pacote <=10¹² e multiplicador <=100×; resultado antes do teto <=10¹⁴ cabe em i64. Orçamento evita multiplicar oito efeitos por oito parcelas no hot path. |
| 6. GPU/I/O | Render/barra existentes, sem pipeline/upload/textura novo. Capturas/PNG/replay explícitos pelo MCP; regressão headless. Componentes são gameplay, sem VFX/áudio implícito. |
| 7. Sincronização | Snapshot inicial de buffs; soma por pacote antes de alterar o plano de vida. Mesmo commit de custos/vida/efeitos, sem callbacks por componente, locks/threads/dependências novas. Falha posterior descarta decomposição e mutações planejadas. |
| 8. Invalidação | Normalização validada transforma escalares classificados pre39 em pacotes e materializa registro. Edição/undo/load/reset recompilam índices/limpam partida; replay39 compara recibos/componentes e exige build compatível. |
| 9. Contrato |22 testes novos cobrem40+30, ordem/classificação versus parcelas, contextos/tipos/limites/rounding/overkill, rollback/morte, impacto/pulso/canal, NPC, source e migração. Quatro grupos MCP cobrem schema/diagnóstico/undo, entrega/pulso, demo100/70/40, replay/prefixo/captura. Resultados finais em validation.md. |
| 10. Custos mantidos | Preflights e consultas por componente deliberados; scan regras×instâncias herdado. Anel maior e serialização da decomposição apenas sob demanda; clone de State no salvamento de replay, sem clone adicional no tick. Inteiros arredondam cada componente; nenhuma promessa de invariância a subdivisão arbitrária ou ganho de FPS. |


## Atributos e buffs — antes de codificar (EAF-49)

1. Registro aberto por chave, resolvido em índices na compilação. Bool e números fixos em unidades/10000, operações tipadas e tags em máscara u64; nenhum caminho JSON ou nome interpretado no tick.
2. Buffs em pool64 e recibos256; duração/IDs em ticks, refresh/independent. Plano guarda até8 instalações e máscara de remoção, sem clonar pool/registro por tentativa. Custos/vida/efeitos e buffs só mudam no commit.
3. Regras agrupadas por atributo e ordenadas uma vez por prioridade/definição/modificador. Consulta só no evento relevante, caminho rápido sem buffs; limite64 definições×8 regras. Scans de instâncias ativos permanecem limitados e explícitos; sem cache precoce de contexto condicional.
4. Consumidores iniciais: interruptible do executor, dispellable do autor de DoT e multiplicador de dano recebido. Base da definição permanece imutável; atributos customizados avaliáveis/inspecionáveis sem novos casos no executor. Adaptadores físicos/render e outros tipos permanecem trabalho próprio.
5. Expiração antes das ações do tick; remoção por morte do beneficiário, efeitos instalados sobrevivem morte da origem. Duas proteções sobrepostas não restauram base prematuramente; composição/empates/reaplicação documentados.
6. Sem nova dependência/thread/I/O/pipeline. MCP antes da demo, inspect base/efetivo/proveniência/contexto, replay/undo/legado. Testar imunidades, vulnerabilidade, autoria, custo/rollback/capacidade, filtros, expiração exata, estado morto/oculto e render. Sem profiling ou ganho de FPS alegado.


## Atributos e buffs — respostas da revisão (EAF-49)

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | `Registry::evaluate` é consultado no evento consumidor: interrupção, dissipação ou dano. Expiração percorre o pool uma vez por tick; pós-commit só repete a limpeza quando há causa potencial de morte. Inspeção/render são caminhos separados. |
| 2. Alocações/cópias | Pool de 64 e anel de 256 pré-alocados; `Plan` guarda oito instalações opcionais e máscara u64, sem clonar registro/pool por tentativa. Avaliação usa callback genérico vazio no gameplay; contribuições JSON só são construídas na inspeção. Campos adicionais aumentam planos/recibos por tamanho fixo, custo aceito. |
| 3. Tipos/acesso | Chaves, entidades, habilidades, categorias e tags são resolvidas na compilação; índices u8/u16 e máscaras u64 no runtime. Valores booleanos ou i64 em escala 10000; intermediários i128 limitados, sem parsing/reflexão/string no consumidor. |
| 4. Repetição | Regras agrupadas por atributo e ordenadas uma vez; saída rápida com pool vazio. Remoção pula compactação quando máscara zero; sobreviventes só são copiados quando realmente mudam de posição. Sem cache de contextos condicionais para invalidar. |
| 5. Escala | Até 64 atributos, 64 definições de buff × 8 modificadores, 64 instâncias e 256 recibos. Consulta custa O(regras do atributo × instâncias), com filtros antes do scan de instâncias. Até 32768 pares no pior caso por consulta; limite explícito do laboratório, não prova de escala MMO. Bases por entidade são busca linear limitada a 64 chaves. |
| 6. GPU/I/O | Barra reutiliza duas caixas e buffers do render, sob J/MCP; consulta o menor prazo entre até 64 instâncias por ator. Sem pipeline, textura, I/O ou captura automática no tick. PNG/replay sob demanda; validação offscreen. |
| 7. Sincronização | Mesmo executor e commit de custos/vida/efeitos. Reservas consideram instalações pendentes, refresh e vagas removidas. Consumidores leem contribuições existentes no início da transação; sem callback de mutação, lock, thread ou dependência nova. |
| 8. Invalidação | Base imutável mais contribuições evitam restauração obsoleta; expiração/refresh/dissipação/morte retiram apenas as instâncias correspondentes. Edição/reset/load/undo recompilam; cena/replay 38 incluem buffs e recibos, ordem estável de aplicação. |
| 9. Contrato | 24 testes Rust novos cobrem operações/contexto/autor/beneficiário, duração, capacidade/rollback, projétil/pulso/NPC, morte e anel. Quatro grupos MCP cobrem autoria/schema, inspeção, demo, undo/load, replay/prefixo e PNGs. Resultados finais registrados em validation.md. |
| 10. Custos mantidos | Scan de regras × instâncias, lookup de bases e planos anteriores de vida/saldo são deliberados neste recorte. Registro atualmente admite bool/número; novos consumidores precisam integrar consulta e invalidação próprias. Sem profiling ou ganho de FPS alegado. |


## Interrupção externa — antes de codificar (EAF-48)

1. Propriedade interruptible compilada uma vez por definição. Efeito consulta somente o cast do destinatário após validação perceptiva; O(1) por operador, sem scan de mundo/coleção por interrupção.
2. Plano guarda IDs opcionais por9 atores, sem copiar Cast/agenda; múltiplos operadores não selecionam a mesma execução duas vezes. Commit único com custos/vida/recursos; sem callback ou rollback parcial.
3. Recibo Copy de resultado e proveniência; anel256 existente. Timestamp por ator impede reinício no mesmo tick e retry existente trata NPC. Nenhum lock/thread/dependência/I/O/GPU novo.
4. Cast protegido ainda cancela por morte, movimento, perda de condições ou comando próprio; proteção não bloqueia a simulação. Efeito não recolhe projéteis/agendas anteriores.
5. Revalidar candidato no commit/impacto sem reservar identidade durante cast. Testar ausência/proteção/duplicação/atomicidade/ordem, canal/custo/NPC/conhecimento, morte, impacto/interceptor, replay/prefixo/undo/legado. Sem profiling/FPS alegado.


## Interrupção externa — respostas da revisão (EAF-48)

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | **Adequado:** proteção compilada em Runtime::new; consulta de cast em plan_effects por operador/tentativa/commit/impacto. Nenhum sensor ou varredura de mundo adicional. |
| 2. Alocações | **Adequado:** plano guarda um único Option<u64>, estado tem9 prazos, recibos Copy no anel256. Sem nova coleção/entidade/callback por interrupção. Campos opcionais aumentam o tamanho de todos os Applied/Receipt; custo fixo aceito para observabilidade comum. |
| 3. Tipos/acesso | **Adequado:** índice de ator/grant/definição já resolvido; bool/enum/u64, sem strings/JSON na decisão ou commit. Candidato emprestado por referência no planejamento, sem copiar Cast ou documento. |
| 4. Repetição | **Adequado:** seleção marca ID uma vez; segundo operador consulta marca e não duplica interrupção. Preflight e commit são repetidos intencionalmente para revalidar execução atual. |
| 5. Escala | **Corrigido:** cada execução tem um destinatário; removidos array9 e loop de commit da versão inicial. Um ID opcional, consulta O(1), até8 operadores. Anel256 pode receber recibo de ambos os atores. |
| 6. GPU/I/O | **Não se aplica ao hot path:** render continua derivando barra do cast existente; JSON, save/load e captura somente sob demanda. Nenhum upload/pipeline/readback adicional por interrupção. |
| 7. Sincronização | **Adequado:** mesmo passo sequencial/plano transacional. Seleção por ID permanece válida até commit imediato; nenhum lock/thread/dependência nova. Prazo bloqueia reinício dentro do mesmo tick e retry limita NPC. |
| 8. Invalidação | **Adequado:** Runtime novo em edição/reset/load/undo recompila bool e zera prazos/casts. Cena/replay37 mantém versão/build explícitos; anéis continuam limitados. |
| 9. Contrato | **Adequado:**18 testes Rust de atomicidade, custos/proteção, privacidade, fase/ordem, cast/canal/projétil/interceptor, retry/barreira, legado e replay.4 grupos MCP cobrem schema/recibos/undo/erros/persistência, replay final/prefixo e pixels repetidos, NPC/jogador na demo. Resultados finais em validation.md. |
| 10. Restante | **Mantido com motivo:** planos copiam saldos/vida como antes; recibos maiores consomem memória fixa, oclusão e mapas existentes permanecem. Não há alegação de FPS/profiling ou escala além dos limites do laboratório. |

## Canalização — antes de codificar (EAF-47)

1. Estado Copy de progresso dentro do cast já limitado a9 atores; sem scheduler, entidade, callback ou alocação por pulso. O Cast também integra pools64 de projéteis/agendas: aumento fixo de tamanho aceito, sem duplicar estado em registry paralelo.
2. Separar validação de alvo/condições do plano de efeitos. Canal ativo revalida sensores/alcance/oclusão por tick; plano fixo de saldos/vida só no início e pulso devido. Até9 consultas limitadas ao mundo atual; custo explícito para interrupção contínua.
3. Custos compilados; on_start uma vez, per_pulse na transação de cada pulso. Nunca financiar custo com o próprio efeito; falha não aplica parte do pulso. Sem strings/JSON na execução.
4. Reusar cast/cancelamento/movement locks/retry/recibos256, um pulso por ator/tick com intervalo positivo/duração finita. Nenhum novo lock/thread/dependência ou I/O por tick.
5. Guia usa a barra existente, agora decrescente/ciano durante canal; sem pipeline/recurso GPU novo. Autoria/inspeção/prefixo/replay/edição/reset compartilham contrato versionado.
6. Testar custos/intervalo/cancel deadline/morte/perda de visão/alcance/condição, movimento livre/interrompido/bloqueado, NPC pré-pago sem saldo, transações falhas, orçamento/erros/legado e render/replay. Sem profiling/FPS alegado.


## Canalização — respostas da revisão (EAF-47)

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | `advance_channel` revalida execução ativa uma vez/tick; `plan_effects` só roda no prazo de pulso. Preparação mantém fluxo anterior. |
| 2. Alocações | Progresso Copy opcional no Cast; sem Vec/timer/entidade por canal/pulso. Cast maior também integra pools de64 agendas/projéteis; custo fixo aceito para manter uma única identidade/representação da execução. |
| 3. Tipos/acesso | Ticks u64, pulsos u32 limitados, modo enum e índices compilados. Barra usa started/finishes_at numéricos; eliminada busca de definição por nome no render durante revisão. |
| 4. Repetição | Separação `validate_execution`/plano evita cópia de9×32 saldos/vida entre pulsos. Validação de linha de visão existente continua por tick ativo, custo deliberado para interrupção contínua. |
| 5. Escala | Até9 execuções e8 operadores por pulso, intervalo mínimo1 e duração máxima216000ticks; anéis256 e orçamento16 ruídos/tick preservados. Sem promessa de grandes populações. |
| 6. GPU/I/O | Reusa duas caixas da barra, cor ciano/razão decrescente, cache e J/MCP existentes. Nenhum pipeline/recurso GPU/I/O novo no tick; PNG/replay sob demanda. |
| 7. Sincronização | Mesmo commit de custos/vida/efeitos, cancelamentos antes de pulsos e ator ordenado; sem threads/locks/dependências. Canal pago não reavalia custo de uma segunda ativação na política. |
| 8. Invalidação | Reset/edição/load/undo recompilam e limpam casts; cancelar/morrer não ressuscita progresso após efeito. Replay36 inclui configuração e estado; prefixo/pixels cobertos. |
| 9. Contrato |16 testes novos de timing/custos/recibos, cancel deadline, lock/interrupt/free, visão/alcance, morte, NPC/política, transações/orçamento, versões/limites.4 grupos MCP de autoria/undo/persistência/phase/replay/capturas/demo. Resultados finais em validation.md. |
| 10. Restante | Planos transacionais fixos, consultas de oclusores e mapas de percepção existentes mantidos; custo de consulta por tick ativo e tamanho do Cast explícitos. Sem profiling/FPS alegado. |

## Proteção contra dissipação — antes de codificar (EAF-46)

1. Booleano na definição autorada e compilada, default true; nenhum campo adicional por instância ou recibo. Validar false somente com agenda e cena35.
2. Filtrar pelo booleano no scan já limitado a64 instâncias antes de marcar máscara/contar limite. Nenhum novo scan, coleção, string, lock, dependência, I/O ou trabalho GPU por tick.
3. Reutilizar transação/revalidação para cast, projétil e NPC; require_match conta apenas removíveis. Não alterar pulsos, expiração, refresh ou limpeza de morte.
4. Inspeção/schema e replay expõem/preservam propriedade; edição/undo/reset recompilam. Testar misturas, limite, ausência protegida/opcional, rollback, impacto, NPC, morte/expiração, legado e erros atômicos.
5. Custo mantido: leitura booleana por candidato, dentro do orçamento existente de8 operadores×64 agendas. Sem profiling ou promessa de FPS.

## Proteção contra dissipação — respostas da revisão (EAF-46)

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | `plan_effects` consulta proteção somente no scan de dispel existente; nenhum processamento novo por pulso/frame. |
| 2. Alocações | Booleano por `Definition`/`Compiled`; `periodic::Instance`, máscara e recibos não cresceram. Sem nova alocação por tentativa/remoção. |
| 3. Tipos/acesso | Booleano nativo resolvido por índice da definição; categoria u8 preservada, sem strings/JSON no filtro. |
| 4. Repetição | Mesmo loop limitado; protegidos rejeitados antes da máscara/contagem. Sem segundo filtro para NPCs/projéteis. |
| 5. Escala | Mesmos limites64 agendas/8 operadores; um teste booleano por candidato do destinatário. Sem índice/cache novo. |
| 6. GPU/I/O | Nenhuma mudança de render; campo extra de inspeção apenas sob demanda. Demo configurada pelo MCP. |
| 7. Sincronização | Reusa plano/commit/revalidação; sem locks, threads ou dependências novas. |
| 8. Invalidação | Recompilação já existente ao editar/reset/load/undo; replay35 inclui configuração. Não guardar cópia por instância evita divergência. |
| 9. Contrato | Seis testes Rust novos cobrem mistura/limite, ausência/opcional/rollback, commit/impacto, NPC, refresh/pulsos/expiração/morte e legado/validação. Grupo MCP cobre inspeção/schema/prefixo/erro atômico/persistência/undo. Resultados finais em validation.md. |
| 10. Restante | Scans e cópia do plano transacional mantidos; proteção por definição exige definições distintas para variantes. Sem profiling ou alegação de FPS. |

## Dissipação — antes de codificar

1. Classificação opcional compilada uma vez: no máximo64 nomes, índices u8 no tick; nomes/JSON somente na fronteira de autoria/inspeção.
2. Dispel varre até64 agendas apenas na preparação/commit/impacto, por até8 operadores. Máscara u64 local ao plano evita remover a mesma instância duas vezes; nenhuma coleção/cópia de agendas por execução.
3. Reusar plano transacional de vida/saldos e validar require_match antes de mutar. Compactar agendas uma vez no commit, conservando ordem/prazos; pulsos não podem conter dispel para evitar mutação reentrante da agenda.
4. Recibos existentes em anéis256 recebem índices/IDs numéricos de origem/execução. Sem callbacks, threads, locks, recursos GPU, I/O ou dependências novos.
5. Não inserir busca de estado alheio na percepção/seleção. NPC self-antídoto usa executor e retry já existentes; alvo externo continua exigindo visão atual antes de consultar seus efeitos.
6. Testar categoria/limite/ordem/ausência, atomicidade com custo/dano/cura, cast vencido/cancel/morte, múltiplos operadores, impacto/reembolso, privacidade, pool64/replay/undo/render. Custos de scans/planos limitados explícitos; profiling/FPS não alegados.


## Dissipação — respostas da revisão (EAF-46)

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | Adequado: busca só em `plan_effects` ao validar início/commit/impacto; nenhum scan novo por frame/pulso. NPC sem correspondência repete apenas no retry existente. |
| 2. Alocações | Adequado: máscara u64 no plano e DispelApplied/Removal Copy nos recibos; compactação estável in-place, Vec/anel reutilizados. Sem Vec de candidatos nem strings por remoção. |
| 3. Tipos/acesso | Adequado: até64 categorias internadas em u8 na compilação; nomes apenas no catálogo e inspeção. Lookup por definição da instância, índices/IDs numéricos no runtime. |
| 4. Repetição | Mantido com motivo: até8 scans de64 instâncias por plano; máscara exclui duplicações entre operadores. Validação de início/commit recompõe lista para respeitar expiração e disputas; sem reserva ou cache de candidatos. |
| 5. Escala | Adequado ao recorte:64 agendas/64 categorias/8 operadores/256 recibos, remoção compacta uma vez no commit; refresh/independent mantêm ordem. Não representa stack/tag engine geral. |
| 6. GPU/I/O | Adequado: usa desaparecimento da faixa já derivada do estado, nenhum recurso visual ou captura por tick novo. Tecla4 só resolve grant e chama dispatcher existente; MCP já suportava esse slot. |
| 7. Sincronização | Adequado: remoção no mesmo commit de custos/vida/saldos, sem lock/thread/dependência novos. Pulsos não podem conter dispel, preservando a iteração da agenda; máscara zero retorna imediatamente. |
| 8. Invalidação | Adequado: plano tem vida local e é recalculado antes de efetivar cast, projétil usa vítima atual. Edição/reset/load/undo reconstroem categorias/agenda, replay registra remoções pela execução das ordens. |
| 9. Contrato | Coberto por testes de categoria/ordem/máscara64, atomicidade, ausência/opcional, cast/cancel/concorrência, impacto/expiração/morte, privacidade/NPC próprio, replay/prefixo/render e autoria atômica; resultados finais em validation.md. |
| 10. Restante | Mantido com motivo: plano copia saldos/vida mesmo para dispel puro; varreduras e recibos de tentativas NPC limitados. Categorias compiladas por busca linear fria; não introduzir registry global. Sem profiling/FPS alegado. |

## Efeitos periódicos — antes de codificar

1. Agenda por tick, no máximo64 instâncias e um pulso/instância/tick; intervalos inteiros positivos dispensam relógio real/fila de tarefas.
2. Vec pré-alocado, instância Copy com origem/destino/definição/cast/prazos. Compactação estável; sem string/JSON/heap por pulso. Recibos em anel limitado.
3. Reutilizar plano transacional de saldos/vida para cada pulso; revalidar componente/destinatário vivo sem consultar posição ou percepção da origem. Custos somente na aplicação/lançamento.
4. Refresh busca linear por origem/definição/destinatário, só na aplicação; independent não busca. Ordem de instalação preservada, refresh mantém posição. Checar capacidade antes de debitar entrega direta.
5. Mesmos limites9 atores/32 recursos/8 efeitos; até64 planos por tick é custo explícito, não promessa de escala. Nenhuma nova dependência, thread, lock ou I/O no tick.
6. Guias derivadas de estado ativo usando até um marcador por ator via cache existente/J/MCP; evitar um marcador por instância sobreposta. Reset/edição/undo reconstruem estado, replay compara agenda e recibos.
7. Verificar atraso inicial/último pulso, refresh/independent/origens, custo/atomicidade/pool, projétil/colisão, morte fonte/destino, privacidade, erro/legado e replay/prefixo/render. Sem profiling ou ganho de FPS alegado.


## Efeitos periódicos — respostas da revisão (EAF-45)

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | Adequado: `advance_periodic` varre até64 agendas/tick e só chama plano quando vence; `remove_dead_periodic` faz limpeza final para mortes de casts/impactos posteriores. Nenhum timer do SO. |
| 2. Alocações | Adequado: `periodic::State` pré-aloca Vec64/VecDeque256, Instance/Receipt Copy, compactação estável e truncate. Nenhuma entidade/corpo/coleção temporária por pulso; estado fixo também reservado em cenas sem agendas. |
| 3. Tipos/acesso | Adequado: índices u8, ticks u64, intervalo/contagem u32 limitados e valores i64; definição compilada/grants existentes. JSON/IDs somente na autoria/inspeção. |
| 4. Repetição | Mantido com motivo: refresh busca no máximo64 entradas na checagem e instalação, só ao entregar. Plano transacional de saldos/vida é reutilizado para validar entrega e pulso; validação inicial descarta efeitos previstos, evitando segundo validador divergente. |
| 5. Escala | Mantido com motivo: até64 planos por tick, cada um copia9×32 saldos e9 vidas e executa até8 efeitos; cap global explícito. Anel de recibos evita crescimento. Não há scheduler/broad phase geral ou promessa de escala. |
| 6. GPU/I/O | Adequado: máscara u16 calculada uma vez para guias, até uma caixa por ator usando frame/cache existentes. Sem recurso/pipeline novo, I/O por tick, raio de percepção ou efeito visual na lógica. |
| 7. Sincronização | Adequado: instalação no mesmo dispatcher/commit, capacidade revalidada antes da entrega, pulses antes de projéteis/casts. Sem threads/locks/dependências novas; cobrança e efeitos não se entrelaçam com edição. |
| 8. Invalidação | Adequado: reset/edição/remoção/load/undo reconstroem agendas/recibos, morte remove instâncias e refresh substitui proveniência/prazos sem mudar ordem. Replay/prefixo compara estado completo, render deriva da lista corrente. |
| 9. Contrato | Adequado:20 testes novos incluem datas exatas, refresh/stack/origens, orçamento, impacto/interceptor, mortalidade/cancelamento/privacidade, saturação/transação e anéis.5 grupos MCP cobrem autoria/undo/remoção, replay/prefixo/pixels e demo. Validação completa em validation.md. |
| 10. Restante | Mantido com motivo: duas varreduras limitadas por tick tratam fases distintas de morte, cópias de planos e pesquisa de refresh; nenhum ganho de FPS/profiling alegado. Efeitos mistos imediatos/periódicos, modificadores e dissipação exigem contratos posteriores. |

## Mira automática de NPC — antes de codificar (EAF-44)

1. Decisão usa a observação já confirmada, sem novos scans de alvo/raycast; comparar distância XYZ ao quadrado. Política tipada por objetivo, validada na autoria.
2. Proveniência Copy fixa (ator/tick/fonte) no cast existente; ponto já está em Aim. Nenhuma coleção/entidade nova por NPC ou disparo. Limites atuais9 atores/8 objetivos/64 voos.
3. Congelar ponto somente ao iniciar, revalidar identidade/visão/alcance/vida/condição antes de cobrar. Não consultar posição oculta nem recalcular mira durante cast/voo.
4. Aproximação reusa motores/alcance efetivo da política; evitar duplicar decisão/executor. Comando espacial externo continua explícito, sem fabricar evidência visual.
5. Guia de diagnóstico usa até3 caixas por cast automático via cache existente/J/set_debug_view; não participa da física/conhecimento. Sem GPU/I/O/lock/dependência novos.
6. Testar aquisição/perda/troca de alvo, movimento/alcance/relação/vida/custo, XYZ/aproximação, fonte onisciente explícita, replay/prefixo/render/reset/undo e legado. Profiling/FPS não alegados.

## Mira automática de NPC — respostas da revisão (EAF-44)

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | Adequado: `ability_policy::evaluate` por candidato/objetivo existente; captura do ponto somente em `abilities::start`, revalidação no plan de início/commit. Sem atualização de mira em voo. |
| 2. Alocações | Adequado: AimObservation Copy guarda ator/tick/fonte; posição reutiliza Aim. Nenhum Vec/entidade/corpo novo por cast; structs maiores mantêm limites de atores/log/pool. |
| 3. Tipos/acesso | Adequado: índices u8/tick u64/f32 XYZ finito, política por objetivo e referências às observações atuais. IDs só na inspeção; nenhuma busca da posição oculta pela identidade original. |
| 4. Repetição | Adequado: observação/reconhecimento já produzidos por percepção, alcance ao quadrado e range efetivo compartilhado com aproximação. Nenhum scan/raycast novo; distância também calculada nos offers antigos durante cast, custo escalar limitado mantido para fluxo comum. |
| 5. Escala | Mantido com motivo: até8NPCs×8objetivos,9casts e64projéteis. Scans/sweep/navegação existentes preservados; sem promessa de multidões ou nova broad phase. |
| 6. GPU/I/O | Adequado: até3 caixas por cast automático visível, adicionadas ao frame/cache existente somente com guias; sem pipeline/textura/upload dedicado. Captura/JSON/replay sob demanda. |
| 7. Sincronização | Adequado: mesma sequência sensores/intenções/execuções/física, sem threads/locks adicionais. Plan revalida vida/limiar atual, pois commits anteriores do passo podem mudar elegibilidade. |
| 8. Invalidação | Adequado: reset/edição/undo/redo limpam cast/proveniência; marker deriva do cast corrente. Cancelamento/deadline removem marcador sem cache próprio; replay inclui proveniência exata. |
| 9. Contrato | Adequado:15 testes novos de ponto/candidato/visão/alcance/XYZ/condições/ordem explícita e guias;4 grupos MCP incluem replay/prefixo/PNG/undo/erro atômico. Evidência da regressão final em validation.md. |
| 10. Restante | Mantido com motivo: política autorada consultada em start/commit, BTreeMap de observações e plano transacional fixo. Novo alcance é de decisão; não garante trajeto. Sem previsão, busca tática, profiling ou ganho de FPS alegado. |

## Mira livre — antes de codificar (EAF-43)

1. Novo comando frio; mira numérica Copy no log/cast, sem entidades, strings ou heap por tick/disparo. Mesmos limites4096 entradas/16 por tick/64 projéteis.
2. Direção/ponto finitos e limitados; normalizar somente ao validar/iniciar e lançar, nunca por passo de voo. Ponto congelado não consulta ator. Origem de lançamento atual.
3. Sem vítima prévia: plano de lançamento valida só fonte/custos; efeitos transacionais no contato. Reusar executor/sweep, preservando seleção por ator e ordem.
4. Cursor convertido no dispatcher para ponto em plano horizontal explícito; matriz inversa apenas no comando. Replay guarda resultado XYZ, não depende da câmera/viewport futura.
5. Sem GPU/I/O/thread/lock novos; casts/voos existentes invalidam em reset/edição/undo. Testar entrada inválida, versões, política NPC, movimento, primeiro contato, replay e paridade MCP/UI.
6. Cópia do plano fixo e varredura linear limitada permanecem; sem profiling/FPS alegado. Objetivo automático espacial/tática e retículo contínuo ficam para incrementos próprios.

## Mira livre — respostas da revisão (EAF-43)

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | Adequado: validação/projeção em comando; plan no início e commit; avanço de projétil inalterado. Sem raycast acompanhando cursor por frame. |
| 2. Alocações | Adequado: Aim/Operation/Cast Copy, target Option numérico; log4096, fila16/tick, pool64. Nenhuma entidade/corpo/buffer novo por mira. Payload de cast/entrada maior, limitado pelos mesmos budgets. |
| 3. Tipos/acesso | Adequado: f32 XYZ finito/limitado, vetor não nulo; alvo opcional real, sem ator fictício. IDs resolvidos no comando, runtime em índices. PartialEq substitui Eq nos tipos que passaram a conter f32. |
| 4. Repetição | Mantido com motivo: direção calculada em validação de start/commit e lançamento (no máximo três vezes por cast), sem trabalho adicional no voo. Raio da câmera extraído/reutilizado entre picking e mira; inversão só no comando. |
| 5. Escala | Adequado ao recorte: nenhuma busca adicional de alvo/obstáculo por frame. Sweep64×512 permanece; definição/objetivos validados só na edição. |
| 6. GPU/I/O | Não se aplica ao tick novo: mesmas caixas e cache render, sem novo pipeline/upload/PNG por movimento do mouse. JSON e diagnóstico na fronteira MCP/inspect/replay. |
| 7. Locks | Adequado: dispatcher atômico existente; UI lê cursor/dimensões e chama o mesmo comando após leitura de grant. Sem thread/lock adicional nem dependência. |
| 8. Invalidação | Adequado: mira faz parte do cast/log existentes, reset/edição/undo reconstroem runtime. Replay registra ponto resolvido; nenhum cache de câmera ou mira de sessão a invalidar. |
| 9. Contrato | Adequado:322 testes Rust e39 cenários MCP/161 grupos aprovados, incluindo fim do cast/coincidência/movimento, ausência de vítima, falhas atômicas, custo/impacto, NPC sem identidade, cursor/replay/pixels; evidência em validation.md. |
| 10. Restante | Mantido com motivo: plano fixo copia saldos/vida mesmo no launch espacial para reutilizar débito transacional; sweep/mapas existentes. Retículo contínuo/tática/mira FPS ainda ausentes. Sem profiling, percentual de ganho ou FPS declarado. |

## Projéteis — antes de codificar

1. Entrega tipada compilada, direct legado; até64 disparos ativos/256 recibos de entrega, Vec pré-alocado e retenção em ordem de lançamento. Não instanciar entidade/corpo Rapier por disparo.
2. Direção XYZ congelada no commit, velocidade/half_extent/distância finitos. Sweep AABB de segmento, sem amostras por metro; distância máxima limita duração e lançamento coincidente falha antes de cobrar.
3. Consulta linear limitada64×512 entidades por tick, só corpos físicos, sem BVH nova especulativa. Empate por ID autorado. Posições dinâmicas congeladas no snapshot do tick; não alegar CCD relativo completo.
4. Lançamento revalida antes de pagar/reservar slot; efeito no impacto reusa plano transacional sem segunda cobrança nem reconsulta de visão/posição do alvo pretendido. Primeiro corpo bloqueia mesmo sem elegibilidade; emissor excluído.
5. Índices/Copy nos voos/recibos; JSON só em inspeção/replay; buffers visuais reutilizados. Sem callbacks/dependências/threads novos. Cancel/morte não apagam disparo lançado; morte por impacto deve interromper casts antes de commits posteriores.
6. Testar parede fina/velocidade/altura/desvio/interceptor/relação/morte/falha/pool, legado/input/replay/prefixo/render/undo e regressão MCP headless. Custos e limites explícitos; nenhum profiling/FPS alegado.

## Projéteis — respostas da revisão (EAF-42)

| Pergunta | Resposta/evidência |
|---|---|
| Frequência/representação | `projectiles::Delivery` compilado em `abilities::Compiled`; Shot Copy guarda slots/execução, posição/direção congeladas e parâmetros finitos. Sem parsing/JSON/resolução de recurso no sweep ou efeito. |
| Memória/alocação | Vec64 ativo e VecDeque256 de recibos pré-alocados; compactação estável retém voos em uma passagem e truncate reutiliza capacidade. Sem corpo/entidade/heap por disparo. Estado também reserva buffers em cenas sem projéteis, custo limitado mantido. |
| Trabalho repetido | Varredura linear até64×512 entidades físicas por tick; posições reutilizam snapshot existente. Slabs por eixo, sem microsteps por metro nem sqrt por colisão. Normalização única no lançamento. Broad phase própria adiada até evidência de necessidade. |
| Efeitos/tipos | Plano existente copia saldos9×32/vida9 para validar atomicidade; mesma implementação no impacto sem débito. Previsão no lançamento valida componentes, mas é descartada: mantém custo limitado de plano duplicado entre launch/impact para evitar segundo validador divergente. Custos iterados por referência, sem copiar array32 desnecessariamente. |
| Ordem/limites | Capacidade revalidada no commit, inclusive casts concorrentes; falha sem débito parcial. Compactação preserva ordem de lançamento; colisão escolhe menor TOI/ID. Corpo aliado/cadáver bloqueia, sem custo de busca de segundo alvo após contato. |
| Cache/render | Cubos usam frame.boxes/instancing existentes; crescimento inicial de buffers segue cache atual e limite64. Capturas repetidas/prefixo verificam invalidação por tick. Sem pipeline/material/textura nova, efeito visual não participa de colisão. |
| I/O/concorrência | Sem dependência/thread/lock/I/O por tick. Inspeção resolve IDs e constrói JSON só na fronteira fria; gravação/replay podem alocar cópias completas, como antes. |
| Correções | Compactação substituiu remove(i), evitando deslocamentos repetidos ao encerrar vários voos. Iteração dos custos usa referência. Fixtures respeitam caixa mínima5cm e opt-in de conectores aéreos; não relaxar contratos existentes para teste passar. |
| Custos/limites mantidos | AABB sem rotação/impulso/CCD relativo; corpos dinâmicos amostrados antes da física. BTreeMap de posições, cópia transacional e consulta linear permanecem. Sem profiling, ganho de FPS, escala além dos limites ou promessa de balística completa. |


## Movimento durante cast — antes de codificar

1. Política enum por definição compilada; padrão livre preserva cenas. Snapshot fixo9 atores com velocidade²/intenção/capacidade de bloqueio, sem mapa/fila/JSON adicionais no tick.
2. Interrupção verifica entrada atual antes de commit e movimento resolvido após física. Comparar velocidade² com tolerância², não usar relógio real nem acumular trajetórias. Recibos/retry usam anel e ordem existentes; cancelamento não debita recursos.
3. Bloqueio deriva de casts ativos, sem flags de teclado persistentes. Suprimir comandos voluntários, preservar gravidade/solver; suspender patrulha/liberar reservas, invalidar caminho de caminhada para replanejar após liberação, preservando buffers.
4. Não congelar travessias jump/glide/fly ou órbita comprometidas: rejeitar lock nesse estado; free preserva integração existente. Sem alteração de corpos para cinemáticos ou teleporte.
5. Inspeção resolve enums/velocidade/IDs na fronteira fria; replay captura amostras/decisões. Validar entrada+ativação no mesmo tick, cancelamento no prazo, falha/morte/liberação, empurrão/queda, NPC espacial/patrulha/passagem, prefixo/legado/edição atômica e MCP sem janela. Sem profiling/FPS.


## Movimento durante cast — respostas da revisão (EAF-41)

| Pergunta | Resposta/evidência |
|---|---|
| Frequência/representação | `CastMovement` compilado por definição; `MotionSample` Copy em array9. Guardas consultam slots/enums, sem parsing/JSON/IDs por habilidade no tick. Default free conserva cenas antigas. |
| Memória/alocação | Sem coleção nova por tick; máscara u16 derivada de casts, amostras fixas reutilizadas. `State::stop_for` limpa buffers de rota sem descartar capacidade. String diagnóstica usa clone_into e pode ampliar capacidade na primeira mensagem mais longa; custo limitado conscientemente mantido. |
| Tipos/matemática | Comparação velocidade²/tolerância² em f32; norma XYZ e deslocamento²/dt², sem sqrt na decisão. sqrt somente na inspeção. Prazos usam ticks existentes, sem relógio real/acumulador de distância novo. |
| Trabalho repetido | Três varreduras de até9 casts (fronteira de habilidade, comandos de motores, pós-física) protegem fases diferentes. Snapshot de posições anterior já existente reaproveitado; uma máscara na inspeção para todos os atores. Consultas a BTreeMaps/corpos permanecem limitadas ao catálogo atual. |
| Física/cache | Bloqueio só suprime motor voluntário; solver/gravidade preservados. Caminhada invalida rota ao suspender, sem executar busca durante lock; replaneja na retomada. Cast recusado não altera motor; travessias aéreas/órbitas incompatíveis preservadas. |
| Invalidação/ordem | Máscara não persiste além da execução; morte/cancel/falha/completion/reset não deixam trava. Input original gravado, sem estado de teclado duplicado. Prefixo/final de replay incluem amostras e estado NPC. Commit anterior ao solver não é desfeito por movimento posterior. |
| GPU/I/O/concorrência | Reutiliza barras/captura existentes. Nenhum recurso GPU, I/O por tick, lock, thread ou dependência acrescentado. MCP/JSON continuam nas fronteiras frias. |
| Correções da revisão | Variantes vazias de struct rejeitam campos desconhecidos (serde em variante unit ignorava extras); máscara de inspeção calculada uma vez. Teste de colisão usa corpo dinâmico, pois NPC navega desviando do ator. |
| Custos mantidos | Amostra/consulta também para atores sem habilidade e modos free, limite atual9; facilita diagnóstico/replay uniforme. Solver, mapas, planos transacionais/recibos existentes mantidos. Sem profiling, benchmark, ganho de FPS ou escala de multidões alegados. |


## Política de habilidades — antes de codificar

1. Extensão opt-in por objetivo, até8 por NPC/9 atores. Configuração Copy e decisões em array fixo; definições/grants já compilados. Nenhum JSON/heap no filtro por tick.
2. Limiar em pontos-base (10000 = 100%): comparar unidades×10000 e máximo×limiar em i64, máximo10^12 implica intermediário10^16 seguro; sem divisão/arredondamento de vida.
3. Ler saúde do alvo somente depois de observação visual confirmada no tick. Política recebe snapshot anterior aos commits; executor preserva revalidação/atomicidade. Vida oculta não altera decisão; som/rádio não revelam saúde.
4. Reusar objetivos/NavGoal/motores, distância XYZ ao quadrado e margem de entrada; sem segundo pathfinder/raycast no filtro. Custos agregados em32 slots; estado/diagnóstico limitado e reset/replay explícitos.
5. Aproximação não concede capacidade; cooldown libera seletor para fallback autorado. Cast em andamento não deve auto-cancelar por sua própria ocupação; perda de condição cancela pelo executor existente.
6. Testar limiares/ausência de componente, cancelamento, múltiplos objetivos do mesmo grant, custo, alcance/obstáculo/perda visual/verticalidade, legado, edição atômica e replay/captura MCP sem janela. Sem profiling ou alegação de FPS.


## Política de habilidades — respostas da revisão (EAF-40)

| Pergunta | Resposta/evidência |
|---|---|
| Frequência/representação | `ability_policy::Policy/Decision` tipados; até8 objetivos por NPC. Filtro após percepção, antes de commits; `policy_offer` usa grant/definição compilados e array32 custos. JSON somente na fronteira de autoria/inspeção/replay. |
| Memória/alocação | `objectives::State.ability_decisions` é array8, sem fila/histórico crescente. Evidence/Offer/Target são temporários de pilha, sem Vec/String/heap. O custo fixo das8 entradas também existe em NPCs sem política; mantido conscientemente pela simplicidade do estado/replay. |
| Tipos/overflow | Vida i64≤10^12; comparação cruzada por10000/limiar≤10000 produz≤10^16. Distância² em f32 para geometria existente; enum de fase e slots u8. |
| Repetição/consultas | Readiness existente reaproveitado; uma varredura de32 custos por objetivo configurado disponível. Aproximação lê só range compilado, sem repetir custos. Não duplica pathfinding/raycast; até8 objetivos com mesmo grant podem repetir custo, limite pequeno consciente. |
| Privacidade/ordem | Vida de alvo lida somente após observação visual confirmada corrente. Rádio/som/memória não fornecem alvo vital. Snapshot de intenções anterior aos efeitos; executor revalida e cobra no commit. Sem callbacks ou reserva de cura implícita. |
| Física/cache | NavGoal compartilhado, stop XYZ ou XZ corrigido por altura, margem4cm para tolerância existente. Perda de visão/liberação do objetivo usa invalidação existente; edição/reset/undo recompilam estado. Aproximação não garante rota nem altera capacidades. |
| GPU/I/O/concorrência | Sem renderer, dependência, thread ou lock novos. Barras/rotas/captura GPU existentes; nenhum I/O por tick. `target_id` resolve apenas nas consultas frias. |
| Correções da revisão | Máscara de elegibilidade por objetivo evita misturar limiares do mesmo grant; cast conserva disponibilidade, mas alvo/condição alterados cancelam. Getter de alcance elimina segunda varredura de custos; zip por referência evita copiar256bytes de saldos por oferta. Relação de self_actor respeita o executor existente. |
| Custos mantidos | BTreeMaps, inspeção JSON, buscas/solver e cópias transacionais existentes. Nenhum benchmark, profiling, ganho de FPS ou promessa de multidões. Validação comportamental registrada separadamente em validation.md. |


## Vitalidade — antes de codificar (EAF-39)

1. Estado vital independente dos grants, arrays limitados aos9 atores, inteiros i64/escala e enums compilados; nenhuma serialização/lookup de recurso por efeito.
2. Plano atômico copia saldos e9 estados vitais na stack; recibos de até8 efeitos e anel de256 mortes pré-alocado. Validar tudo antes do commit; morte somente ao final do plano.
3. Ordem existente preservada: cancelamentos, casts vencidos, solicitações novas por ID. Revalidar vida entre execuções, cancelar casts imediatamente; sem callbacks recursivos.
4. Máscara fixa de mortos no tick e conhecimento individual confirmado; não filtrar memória por estado oculto. Corpse sem motores, gravidade/inércia/colisão preservadas; reservas liberadas no mesmo tick.
5. Guias reutilizam buffers e controles MCP; replay inclui estado vital/recibos. Verificar atomicidade, morte oculta, física aérea, reservas e filas pendentes sem janela. Sem profiling ou alegação de FPS.


## Vitalidade — respostas da revisão (EAF-39)

| Pergunta | Resposta/evidência |
|---|---|
| Frequência/representação | `vitality::Runtime` mantém9 slots tipados; `abilities::CompiledEffect::Vital` usa enum/índice/i64. Catálogos na edição, sem JSON/string parsing no plano. Inspeção/arquivos continuam frios. |
| Memória/alocação | Plano copia9 `Option<Health>` e causas fixas, além de9×32 saldos existentes; até8 recibos fixos. Mortes usam VecDeque256 pré-alocado; nenhum heap por efeito. Estado vital existe mesmo sem grants, custo limitado consciente. |
| Tipos/overflow | Initial/máximo/amount até10^12; dano usa min antes da subtração e cura min(máximo−atual) antes da soma. Todos os intermediários permanecem válidos em i64. Escala não multiplica o valor durante o tick. |
| Repetição/consultas | Máscara u16 global derivada dos9 slots; conhecimento individual em u16. Varreduras limitadas aos9 atores no commit/tick. Morto físico desativado uma única vez por máscara derivada, evitando set_gravity_scale(wake=true) contínuo. Slot do jogador compilado no setup. |
| Atomicidade/ordem | Plano instantâneo reutilizado; custos/vida/ruído só após validação integral. Cancelamento de casts após commit; nova intenção revalida vida. Sensores/radio precedem efeitos, liberação de passagem precede física. Sem callbacks/recursão/threads novos. |
| Privacidade/cache | Morte consultada pelo sensor só após observação aceita; máscara de mortos conhecidos barra relatos antigos. Reset/load/undo reconstroem todos os estados, replay inclui estado vital, conhecimento e pendências. |
| Render/GPU | Até2 caixas extras de vida por ator comJ, reutilizando buffers/cache existentes. Corpse muda cor e elimina guias sensoriais, sem efeito sobre conhecimento. Capturas repetidas/prefixo verificadas separadamente do teclado nativo. |
| Custos mantidos | BTreeMaps e varreduras geométricas existentes, cópia do plano, anéis pré-alocados e JSON nas consultas. Corpo morto permanece no solver; não há pooling/removal/ragdoll novo nem promessa de escala maior. |
| Correções da revisão | Guarda de vida aplicada antes do motor do jogador; desativação de gravidade somente na transição; pistas ocultas preservadas e rádio antigo filtrado por conhecimento individual. Nenhum profiling ou ganho de FPS declarado. |

## Habilidades diretas — antes de codificar (EAF-38)

1. Catálogo/grants compilados na edição; IDs externos viram índices e operações tipadas. Recursos i64/escala, tempo em ticks; sem parsing/JSON no executor.
2. Até9 atores,32 recursos globais/16 por ator,16 grants/ator,64 habilidades,8 custos/efeitos. Uma execução/ator,16 entradas/tick,4096 entradas registradas e256 recibos em anel pré-alocado.
3. Custos agregados, plano de saldo/efeitos em arrays limitados antes do commit. Atomicidade inclui capacidade de ruído; sem efeitos/callbacks externos neste recorte.
4. Intenções NPC usam snapshot anterior aos commits; cancel antes do vencimento, casts existentes antes de solicitações novas; ordem estável por ID. Visão confirmada exigida para qualquer ator NPC, inclusive comando MCP.
5. Ruído próprio regenerado e consumido no próximo tick. Reset/edição/replay recompilam tudo; prefixo preserva entradas pendentes. Não copiar log de efeitos como input.
6. Revisar heaps, clones, buffers/limites, consultas e guias; testar atomicidade, interrupção/retomada, cooldown, privacidade, legado/replay e captura headless. Nenhum profiling/FPS alegado.


## Habilidades diretas — respostas da revisão (EAF-38)

| Pergunta | Resposta/evidência |
|---|---|
| Frequência e representação | `abilities::Runtime` compila IDs de recursos/grants em índices, custos agregados e enums. Tick usa i64/u64/f32; serialização somente na autoria, inspeção e replay. Nenhum parsing de JSON nos efeitos. |
| Alocação e memória | Saldos/cooldown/retry/casts em arrays; plano9×32 i64 em scratch de pilha (~2,3KiB), efeitos em array8. Filas de ruído16 e anéis128/256 pré-alocados/reutilizados; sem alocação por efeito. Log externo cresce apenas ao receber comandos, limitado4096; consultas frias alocam JSON. |
| Trabalho repetido | Cast instantâneo reutiliza o plano validado, evitando segunda consulta de alcance/oclusão e segunda cópia de saldos. Cast com duração revalida no vencimento por necessidade semântica. Catálogo vazio tem caminho curto; readiness percorre até9×16 slots. |
| Tipos/overflow | Cada custo até10^12, até8 termos; agregado validado até10^12. Saldo+delta fica entre−10^12 e2×10^12 antes de clamp, seguro em i64. Ticks/IDs ficam limitados pela sessão36.000 e entradas4096; ruídos têm namespace distinto de passos/manuais. |
| Consultas/escala | Distância XYZ ao quadrado, matriz de relações compilada; oclusão varre até512 caixas com short-circuit. BTreeMap de posições e comparação de IDs da geometria existente continuam, sem alegar ausência de lookup no mundo. Até9 atores; não promete escala de multidões. |
| Eventos/ordem | Intenções anteriores aos commits; cancel, casts existentes e novas solicitações em fases estáveis. Máximo16 sinais/tick; saturação rejeita plano inteiro. Nenhum callback recursivo ou cadeia no mesmo tick. |
| Render/GPU | Barras somente comJ, até34 caixas extras por ator; Vec/cache de vértices existente reaproveitado. Cast/saldo alterados invalidam as caixas correspondentes. Ruídos fundem três históricos limitados sem sort/coleta por frame. |
| Vida útil/concorrência | Sem threads/locks novos. Edição/reset recompilam tudo; replay inclui entradas pendentes e estado completo. Guias não alteram percepção. Nenhuma janela usada pelos testes. |
| Custos mantidos | Scratch copia saldos de todos os atores para atomicidade simples; memória inicial limitada mesmo sem grants, inspeção JSON e Rapier permanecem custos distintos. Não há profiling, comparação com outra engine ou ganho de FPS alegado. |

## Estudo de habilidades — revisão proporcional (EAF-37)

Unidade documental, sem alteração de runtime/build/formatos. Antes de propor implementação: ler pontos de custo reais da referência, não inferir desempenho do catálogo. Depois da leitura:

| Pergunta | Evidência e orientação proposta |
|---|---|
| Alocações | Referência cria Actor por execução/entrega, TMap para planejar custos e GameplayEffect transitório. Não transportar esse layout diretamente; Ignis pode compilar índices e pré-alocar slots. Não há benchmark comparativo. |
| Tipos/contrato | Proposta de unidades i64 com escala para recursos; f32 para geometria existente. Justificativa é semântica de saldo/overflow, não alegação de FPS. |
| Repetição | Compilar definições/grants na edição, agregar custos antes do débito e manter prazos por tick. Evitar parsing, lookup de strings e validação completa de catálogo por ativação. |
| Escala | Limites propostos para definições, recursos, efeitos, solicitações, casts e recibos. Nenhuma promessa de suportar esses novos limites já neste build. |
| Eventos/vida útil | Referência tem timers, recursão, filhos sobreviventes e callbacks. Primeiro recorte proposto é direto; eventos de som somente no tick seguinte. Projéteis terão lifetime separado. |
| GPU/I/O/locks | Estudo não adiciona nenhum. Inspeção/captura futura pelo MCP; serialização fora da aplicação de efeitos, sem I/O de arquivos por tick. |
| Validação desta unidade | Caminhos/símbolos e hashes de16 fontes consultados; relatório histórico separado de teste de gameplay; revisão de consistência do contrato, diff e docs. Sem recompilar engine ou executar Unreal para validar prose. |
| Riscos a resolver | Integração de intenções/objetivos, privacidade de alvo, cancelamento no prazo, replay de entradas pendentes e falta de atomicidade de efeitos persistentes. Matriz concreta no estudo. |


## Comunicação de pistas — antes de codificar

1. Rádio opt-in tipado/Copy, canal u16, alcanceXYZ/cooldown/idade; arrays fixos com até9slots de atores (8NPCs). Coletar snapshot das observações do tick anterior antes de atualizar qualquer receptor; nada de ler posição corrente do alvo.
2. O(N²) com N≤8, catálogo de relações existente, distâncias ao quadrado; nenhuma fila crescente, String/JSON/clone de cena ou alocação por mensagem. Relatos não são retransmitidos. Ausência de configuração preserva sensores existentes.
3. Origem/instante/alvo em índices estáveis do catálogo; converter IDs/JSON só na inspeção. Um recibo por NPC e contadores saturados/cooldown; mundo reconstruído em edição/reset/load/undo.
4. Observação direta prioritária; evidência visual/som ainda válida não é substituída por rádio. Idade original governa TTL e memória, evitando renovar pista antiga. Receber não equivale a ver/ouvir nem confirmar visualmente.
5. Testar determinismo/ordem, alianças/canal/raioXYZ, reconhecimento, expiração sem renovação/cascata, conhecimento oculto, território/objetivos, replay/captura via MCP headless. Sem GUI nova ou profiling/FPS alegado.

## Comunicação de pistas — respostas da revisão (EAF-36)

| Pergunta | Resposta/evidência |
|---|---|
| Frequência | collect/receive uma vez por tick, antes/durante percepção. Sender consulta somente a observação confirmada anterior; não repete raycast para comunicar. |
| Alocações/cópias | Array fixo de9 slots e valores Copy; um recibo por NPC. Sem heap/JSON/String por mensagem. A inspeção fria continua serializando estado. |
| Tipos/acesso | Canal u16, índices u8 do catálogo ordenado, ticks inteiros, distâncias XYZ ao quadrado e relações em matriz já compilada. |
| Trabalho repetido | Cooldown por emissor, filtros antes da seleção, sem fila/retransmissão. Mesmo relato/relato antigo não reinicia investigação. Default direto continua prioritário. |
| Escala | Até8 NPCs; O(N²) limitado. Nenhum broadcast de rede ou promessa de multidões. |
| GPU/I/O | Sem alteração de renderer ou I/O no tick; guias existentes refletem investigação. Capturas somente quando solicitadas via MCP. |
| Concorrência | Sem threads/locks novos. Snapshot de todos os emissores precede atualização dos receptores; empate lexical elimina dependência da ordem. |
| Cache/vida útil | Estado recompilado em edição/reset/load/undo; recibo expirado persiste apenas para diagnóstico, sem validade para decisão. Replay inclui o estado. |
| Contratos | Privacidade sob movimentos ocultos, fontes distintas, alianças/canal/altura, território, reconhecimento, prioridades, expiração, histórico e replay cobertos por Rust/MCP headless. |
| Custos mantidos | Scan dos poucos rádios mesmo sem entrega; percepção ainda usa BTreeMap/clone de policy existentes. Memória única e prioridade conservadora documentadas. Sem profiling ou ganho de FPS alegado. |

## Experimento de controlador — antes de codificar

1. Experimento Rust separado do runtime: reaproveitar Rapier 0.30.1 do lockfile, sem dependência, formato, controlador ou operação de gameplay nova. Contratos de produção continuam em discussão.
2. Mesma caixa de personagem, passo fixo e comandos para comparar velocidade em corpo dinâmico com deslocamento corrigido por shape casts. Construir geometrias e coleções uma vez por caso; relatório serializado apenas depois da simulação.
3. Casos e ticks finitos, amostras pré-alocadas. Contar callbacks/grounded/deslocamento como evidência funcional, sem tratar isso como profiling ou comparar FPS.
4. Inspecionar custos do controlador existente, especialmente autostep/consultas e aplicação opcional de impulsos. Não prometer ausência de alocações dentro da biblioteca.
5. Registrar colisão, limites de inclinação/degrau, teto, queda e interação dinâmica; repetir execução para comparar trajetória no mesmo build/plataforma. O experimento não comprova replay da engine nem comportamento integrado de NPC.
6. Um relatório em artifacts/locomotion-probe/report.json; executável de experimento em target/release/examples. Não abrir janela nem alterar a sessão visual do usuário. Validar o runtime existente por MCP headless e manter a decisão de integração explícita.

## Experimento de controlador — respostas da revisão (EAF-35)

| Pergunta | Resposta/evidência |
|---|---|
| Frequência/escopo | Somente executável de estudo:11casos×3motores×2execuções×240ticks, mais um passo inicial por mundo. src/Cargo.lock/formato da engine inalterados. |
| Alocações próprias | Geometrias, mundo, Vec de32colisões e8amostras por caso; clear reaproveita callbacks por tick. move_shape limita a20iterações na versão fixada. Não copiar/serializar cenário por passo. |
| Biblioteca | Revisão de rapier3d0.30.1 encontrou Vec de manifolds na detecção de chão e em impulsos, além das estruturas internas do solver. Autostep acrescenta casts. Não é controlador sem heap; explicitar e revisar scratch/budgets ao integrar. |
| Tipos/repetição | f32, enums, SharedShape criado uma vez por mundo; seis vértices de cunha na construção. Hash usa bits de posições por tick; normalização trigonométrica da rampa só no setup. |
| Diagnóstico | Contact query para cada colisor depois de cada passo mede penetração amostrada; custo O(ticks×colisores) do experimento, não proposta de telemetria de produção. Repetição tem propósito de validar resultados no mesmo build. |
| GPU/I/O/locks | Sem GPU/janela/threads novos. JSON somente ao concluir relatório; stdout redirecionado para arquivo estável. Regressão MCP continua offscreen. |
| Contratos | Mesmo corpo/atrito/comando horizontal; gravidade cinemática explícita.33resultados com asserts de movimento/bloqueio/contato/impulsos e repetição;207testes Rust, fmt/clippy,14grupos MCP. |
| Restante | Sem benchmark/FPS, navmesh, integração de controlador, replay da nova física ou conclusão da v1. Discutir geometria compartilhada, apoio parcial, impulsos e transições aéreas antes do próximo contrato. |

## Apoio entre superfícies — antes de codificar (EAF-34)

1. Reutilizar geometria de caixas fixas e motores; nenhuma dependência, formato de cena ou canal visual novo. Rede/conectores/entidades seguem autoria MCP existente. Não implementar rampas/degraus implicitamente.
2. Fast path de caixa única preservado. Slow path cobre exatamente a projeção varrida do corpo via faixasX nos limites dos retângulos e das extremidades do corpo; união de intervalosZ, sem amostragem por distância que ignore buracos.
3. Até64 superfícies relevantes por consulta composta, buffers fixos na stack e ordenação in-place sem heap. Acima disso, rejeitar conservadoramente e expor diagnóstico nos links. Máximo132 cortesX, ordenação de até64 retângulos uma vez e scan por faixa; manter custo conhecido/bounded. Fast path permite cena maior quando uma caixa basta.
4. Consulta de segmento durante compilação/planejamento/lançamento; consulta pontual para apoio/aterrissagem, fast path usual. Sem construir malha ou dados JSON por tick; sem memoização com invalidação incompleta.
5. Diminuir footprint pela margem de borda existente1cm, sem expandir cada piso e fechar gaps artificialmente. Tolerância vertical de contato6cm mantida; sweep de colisão continua obrigatório e separado.
6. Validar junções, diagonal, largura, gaps finos, limite/ordem, alturas, física/replay e remoção/undo de ponte. Capturas GPU/MCP headless e um binário corrente; preservar sessão visual ativa. Sem alegação de ganho de FPS.

## Apoio entre superfícies — respostas da revisão (EAF-34)

| Pergunta | Resposta/evidência |
|---|---|
| Frequência | Consulta de segmento na compilação/planejamento/lançamento; apoio pontual no motor existente. Fast path de caixa única preservado. |
| Alocações/cópias | Sem heap por consulta:64 Rects/132 cortes em arrays fixos, sort_unstable in-place. Aproximadamente1,6KiB de dados temporários no slow path; sem Vec/JSON/cache novo. |
| Tipos/acesso | f32/Vec3 e enum Support; cópias pequenas. Distâncias/formatos existentes preservados; motivo traduzido para string estática na compilação de links. |
| Repetição | Retângulos ordenados uma vez por consulta; scan da uniãoZ por faixaX. Ponto comum sobre uma caixa evita composição. Não há construção de malha. |
| Escala | Scans até512 entidades, composto até64 retângulos/132 cortes. O(S log S+S²) limitado; excesso recusa explicitamente nos links. Superfícies dentro da AABB mas fora do polígono podem consumir orçamento conservador. |
| GPU/I/O | Sem alteração de renderer, buffer GPU ou protocolo. Guias/captura existentes exibem as novas cenas. |
| Locks | Nenhum lock/thread/IO introduzido; mesma ordem de simulação. |
| Invalidação | Sem cache novo. Edição/load/undo recompilam rede; consulta usa a cena atual. Geometria dinâmica não é superfície de apoio. |
| Contratos |206 testes Rust,32 roteiros/137 grupos MCP; junções/diagonal/buraco1cm/limite/ordem/capacidade, replay/prefixo/PNG e remoção/undo. |
| Restante | Scan de entidades para apoio pontual permanece; endpoints de voo sobre emendas podem repetir composição. Fast path/limites controlam custo, sem profiling ou FPS alegado. Não inclui navmesh/rampas ou planejamento de desvio novo. |

## Patrulhas XYZ — antes de codificar

1. Reutilizar FSM/objetivos/motores existentes, sem scheduler/coleções novas no tick. Pontos3D e ring3D opt-in, até16 destinos, requerem mobilidade espacial.
2. Resolver normal/base/trigonometria e pontos procedurais apenas na edição/carga. Vec de rota resolvida já existente; sem gerar geometria/rotas no tick de patrulha.
3. Coordenadas são centros dos corpos, como NavGoal/guard_3d; rede permanece em pés. Comparação de feedback espacial usa tolerância física2mm para ida/volta centro-pés, sem ampliar colisões ou considerar parcial como sucesso.
4. Guides preservam XYZ de rotas espaciais, com mesmo limite de pontos/segmentos. Inspeção/save/replay pelo MCP existente, sem JSON na simulação.
5. Interrupção preserva índice; motores conservam compromisso aéreo. Espera é atraso para próximo ponto, não garantia de repouso para voadores sem hover; não introduzir parada impossível.
6. Validar capacidades/pontos inacessíveis, wait/loop/ping-pong, proceduralidade, pista/retomada/compromisso, parcial/timeout, replay/histórico/guias. Cena/replay24; regressões headless e um build corrente.

## Patrulhas XYZ — respostas da revisão

| Pergunta | Resposta/evidência |
|---|---|
| Frequência | Mesma FSM por tick, mesmos estados/timers; geração apenas em construção/edição da simulação. |
| Alocações/cópias | Sem coleção ou clone adicional no tick. Rota resolvida reutiliza Vec existente com até16 centrosXYZ; enum de autoria cresce somente com novos dados de edição. |
| Tipos/acesso | Arrays f32/enum tipado, índices u8 e máscara u16 preservados. Tolerância de feedback2mm por componente usa operações escalares, sem raiz ou codec no runtime. |
| Repetição | Normal/base/sin_cos somente na resolução do anel. Pontos não são recalculados durante espera ou a cada frame. |
| Escala | Limites existentes de NPCs/16pontos e planejamento da mobilidade conservados. Geração não busca conectividade implicitamente. |
| GPU/I/O | Guias reaproveitam buffers/limites existentes e mantêm altura. Capturas sob demanda via GPU offscreen; nenhum renderer ou canal novo. |
| Locks | Sem threads, locks ou I/O na rotina; dispatcher MCP compartilhado. |
| Invalidação | Load/reset/edição/histórico recompilam. Resultado antigo exige tick e destino compatíveis; parcial permanece falha. |
| Contratos | Loop/ping-pong/espera, interrupção/retomada, órbita, saltos/planeio/capacidades, validação/legado e replay cobertos; evidências finais em validation.md. |
| Restante | Verificação de contato acrescenta consulta geométrica de clearance somente perto da aterrissagem, mantendo tolerâncias do motor. Scans, grafo e previsão de voo preexistentes permanecem limitados e síncronos. Sem profiling ou ganho de FPS alegado. |

## Visão volumétrica — antes de codificar

1. Opt-in tipado por NPC, pitch inicial e acompanhamento de movimento; yaw/FOV/alcance reutilizados. Legado usa caminho horizontal existente. Sem dados JSON no tick.
2. Runtime escalar de pitch/configuração, sem novos lookups de alvo oculto ou coleções. Direção segue comando do motor, não rastreamento do alvo nem deslocamento de colisão.
3. Teste volumétrico por produto escalar, sem acos. Alcance XYZ e oclusão por caixas existentes; sem amostragem de corpo inteiro. Trigonometria limitada por candidato existente, custo explícito, sem reformular sensores antes de medição.
4. Guias limitadas a anéis/meridianos com resolução fixa e cache existente. Corrigir ribbon vertical degenerado sem mudar geometria horizontal.
5. Cena/replay23; configuração inválida atômica, reset/histórico recompilam orientação. Replay continua vinculado ao build; sem migração de replays de builds anteriores.
6. Validar verticalidade, limites, oclusão/teto, conhecimento oculto, movimento XYZ, legado, reset/histórico, replay e capturas via MCP headless. Um build release corrente; sem abrir/reiniciar demo do usuário.

## Visão volumétrica — respostas da revisão

| Pergunta | Resposta/evidência |
|---|---|
| Frequência | Filtro angular no estágio sensorial existente; orientação atualizada com comando de motor. Inspeção/serialização somente por comando. |
| Alocações/cópias | Config Copy; pitch opcional e bool no runtime. Nenhum Vec, String ou clone adicional no tick. Guias reutilizam cache/buffers existentes. |
| Tipos/acesso | f32 tipado/validado; yaw/pitch bastam para cone circular. Nenhum novo acesso ao alvo oculto fora de observe. |
| Repetição | Produto escalar/normalização, sem acos. FOV360 ignora cálculo angular. Trigonometria por candidato existente mantida; custo conhecido, sem alegação de gargalo medido. |
| Escala | Catálogo/candidatos e varredura de oclusores mantêm limites existentes. Até594 vértices por guia,306 em360°, sem geometria por número arbitrário de amostras. |
| GPU/I/O | Guias atualizam apenas ao mudar entrada do cache; imagens pelo caminho offscreen existente. Nenhuma textura nova além das capturas normais. |
| Locks | Nenhuma thread/lock/IO novo na simulação; mesmo dispatcher e sincronização. |
| Invalidação | Edição/reset/load/undo recompilam orientação inicial. Pitch/eye integram comparação de guias; teste de cache cobre mudança e verticalidade. |
| Contratos |193 testes Rust (7novos),30 roteiros/130 grupos MCP headless, replay/prefixo/PNG, erro atômico/histórico/legado e conhecimento oculto. |
| Restante | Sem rotação sensorial gradual, roll/corpo/animação, varredura vertical ou amostragem de membros. Comando motor difere de deslocamento após colisão. Sem profiling/FPS alegado. |

## Validação sem janela — antes de codificar

1. Executor apenas de desenvolvimento, fora do tick/frame. Reutilizar binário release corrente; nenhuma dependência ou mudança Rust.
2. Roteiros sequenciais: um processo de engine por cenário, sem renderer de janela; GPU offscreen somente quando os comandos existentes pedirem captura. Sem promessa de ganho medido.
3. Workspace gerado em caminho estável, com cópia dos exemplos pequenos e outputs substituídos. Evitar regravar fixtures/fontes ou controlar a sessão visual do usuário.
4. Logs em arquivo e resumo limitado por cenário; sem guardar imagens na memória do executor. Relatório incremental, sem retry que esconda falha.
5. Impedir duas suítes concorrentes sobre os mesmos outputs; preservar execução individual e opt-in de janela, com gate desktop separado.
6. Validar o executor com a regressão MCP completa, incluindo capturas e replay, e conferir exemplos intactos. Runtime Rust inalterado dispensa reconstruir binário/repetir testes Rust nesta unidade.

## Validação sem janela — respostas da revisão

| Pergunta | Resposta/evidência |
|---|---|
| Frequência | Executor Python apenas em validação; nenhuma alteração Rust, tick ou frame. |
| Alocações/cópias | Copia exemplos uma vez por execução para workspace gerado estável; stdout/stderr direto em arquivo, resumo JSON limitado a 1 MiB por cenário. Não carrega PNGs no executor. |
| Tipos/acesso | Paths resolvidos e subprocessos com lista de argumentos, sem shell; códigos de saída e contagem inteira de checks validados. |
| Trabalho repetido | Uma engine por roteiro para isolamento; binário corrente reutilizado, sem rebuild e sem retry automático. |
| Escala | Lista explícita de 29 roteiros em ordem de dependência, comparada aos scripts presentes para detectar omissão. Um roteiro ativo por vez, seleção opcional em ordem canônica. |
| GPU/I/O | Sem criação de janela/surface; captura offscreen sob demanda pelo renderer existente. Logs por cenário e relatório substituído atomicamente em caminho fixo. |
| Sincronização | flock não bloqueante impede duas suítes sobre os mesmos outputs; não adquire locks ou controla a sessão visual do usuário. Suporte do executor validado em macOS, não Windows. |
| Limites/invalidação | Exemplos versionados copiados antes do run; somente workspace gerado é regravado. Relatório lista seleção/resultados atuais; outputs de roteiros não executados podem ser antigos e não contam como evidência corrente. |
| Contratos/testes | 29 roteiros/126 grupos reais headless passaram; play/pause/step/replay verificados separadamente. Falha real de subprocesso, parada sem retry, trava e opt-in de janela validados sem abrir UI. Exemplos versionados intactos. |
| Custos mantidos | Inicialização de processos/GPU e capturas/readback permanecem; sem profiling/alegação de FPS. Janela física permanece gate explícito; sessão persistente com editor conectável não foi implementada. |

## Voo curvado — antes de codificar

1. Perfil opt-in, straight explícito/ausente preserva legado. Velocidade máxima reutiliza fly_speed; dados numéricos tipados, sem JSON no tick.
2. Controle de velocidade/direção em passo fixo: aceleração/desaceleração vetorial, giro e filtro de direção, estado escalar. Trajetória é previsão do mesmo controlador, não spline independente da execução.
3. Prever somente o trecho selecionado no lançamento, com limite1800ticks e órbita até3600ticks. Varredura do corpo por segmento; rejeitar bloqueio/limite sem fallback retilíneo silencioso. Grafo usa custo nominal como antes; não promete ótimo considerando curvas dependentes da direção.
4. Preview visual limitado128 pontos, capacidade reutilizada; sem gravação de histórico por frame. Holding usa rotação vetorial fixa por tick, sem replanejar a cada volta.
5. Controle usa apenas destino conhecido congelado. Verificação do próximo deslocamento e divergência física protege execução; colisão pode impor parada de segurança fora dos limites nominais. Órbita não inventa nova pista.
6. Testar limites físicos, variação de suavidade/giro, hover/min-speed/holding, curva bloqueada com reta livre, colisão, orçamento real, perda/troca de pista, replay/histórico/legado, guias MCP e regressões completas. Sem profiling/FPS alegados.


## Voo curvado — respostas da revisão

| Pergunta | Resposta/evidência |
|---|---|
| Frequência | Controle e próximo sweep por tick somente para perfil curvado ativo; previsão completa somente ao lançar o trecho escolhido e validar a órbita de chegada. Holding sem novo objetivo não replaneja. |
| Alocações/cópias | Flight State/Settings Copy, campos escalares. flight_path reutiliza Vec limitado128 pontos; nenhum histórico por tick ou conjunto de candidatos temporário. NPC legado não aloca preview. |
| Tipos/acesso | f32/Vec3, contadores u32/u8 saturantes, perfil tipado. Sem consulta de alvo oculto, serialização ou lookup adicional de ator no controle. |
| Cálculos repetidos | Direção inicial normalizada no lançamento; quaternion da órbita calculado uma vez e reutilizado. Dot substitui acos na escolha do orçamento de giro. Normalização/ângulos restantes fazem parte do controle limitado; nenhum ganho medido alegado. |
| Escala | Até1800 passos por preview de deslocamento e3600 por órbita, cada um varre caixas fixas da cena limitada. Custo alto conhecido no lançamento, sem busca sobre todos os estados de velocidade/orientação; grafo usa custo nominal anterior. Falha encerra previsão imediatamente. |
| GPU/I/O | Até127 linhas do preview mais direção no buffer existente; captura/JSON/I/O só por comando. Preview de colisão completo e guia reduzida são distintos. |
| Locks/concorrência | Dispatcher/locks existentes preservados; nenhuma thread, dependência ou acesso externo novo. Preview síncrono pode aumentar latência do passo que lança voo, tradeoff explícito. |
| Invalidação | Reset/edição/load/histórico recompilam e limpam controlador; direção final pode persistir na mesma partida. Mudança de pista termina trecho congelado. Orbitador mantém rota segura diante de destino novo indisponível. |
| Contratos/testes | Nove testes novos cobrem limites vetoriais/giro, curvas/parâmetros, hover/órbita, colisão estática/dinâmica, orçamento real, memória, erro atômico/legado/histórico/replay/ordem e limites de previsão. Quatro grupos MCP com duas voltas, captura/prefixo/PNG e guias; resultados finais em validation.md. |
| Restante | Parada de segurança pode exceder desaceleração nominal; movimento pós-colisão pertence ao solver. Não há aerodinâmica, inclinação de mesh, optimalidade global por dinâmica, desvio de móveis ou continuidade garantida em toda junção. Sem profiling/FPS/escala máxima declarados. |

## Conexões locais espaciais — antes de codificar

1. Opt-in por NPC: alcance máximo e modos walk/fly, validados contra capacidades. Sem dependência nova, sem geração de salto/planeio nem alteração da rede autorada.
2. Planejamento com fontes múltiplas: até64 entradas e64 saídas, dois modos lineares; distância² filtra antes de geometria. Arrays fixos e Vec de rota existente, custo inteiro e desempate estável. Direto compete com percurso pela rede; caminho completo precede fallback parcial.
3. Varredura linear deve testar segmento inteiro uma vez, evitando repetir até600 sweeps colineares. Amostragem balística permanece somente em salto. Apoio de caminhada continua exigindo superfície comum.
4. Conector ativo escalar com tipo/modo/endpoint, sem adicionar nós persistentes ou histórico crescente. Destino congelado durante trecho; perda/troca de pista respeita compromisso, colisão e budget.
5. Alvos móveis podem renovar intenção; processamento limitado a término do trecho/retry. Conectores não consultam ator oculto, somente NavGoal. Guias e replay expõem conector real; edição/reset limpa runtime.
6. Testar posição fora da rede, destino livre, rota bloqueada/desvio, capacidade/alcance/apoio, parcial com entrada, memória/budget/interrupção, replay/ordem/atomicidade/legado. Manter um build release corrente após reconstrução do cache.


## Conexões locais espaciais — respostas da revisão

| Pergunta | Resposta/evidência |
|---|---|
| Busca e trabalho repetido | Dijkstra multi-origem em arrays fixos64; até129 tentativas de conector (direto/entrada/saída), além de testes limitados de ancoragem/chegada. Distância² antes de varrer geometria; modos compartilham a mesma varredura. |
| Geometria | motion usa um sweep inteiro para segmentos lineares e até600 somente para salto. local_motion usa um sweep e uma consulta de superfície comum para caminhada. Sem amostrar chão que possa perder uma lacuna estreita. |
| Alocações/cópias | Dois arrays de64 Option<Connector>, custos/visitados/predecessores na pilha. Connector/Config/Motion Copy; nenhuma inserção na rede ou coleção de candidatos. Vec da rota pré-alocado e reutilizado; path cresce só ao atingir nova capacidade. |
| Tipos/custos | Endpoint f32, flags bool, enum de modo/tipo; custo u32 de duração nominal arredondada por trecho, limite10s e alcance8m. Menor custo total inclui entrada/saída; desempate direto/nó/modo determinístico. |
| Cadência e invalidação | Movimento ativo não replaneja a cada frame; endpoint congelado. Conclusão, mudança de objetivo conhecido e retry são fronteiras. Mudança contínua de destino inalcançável pode antecipar retry: custo conscientemente mantido para preservar responsividade atual, sem garantia de escala. Parcial estacionário não replaneja. |
| Conhecimento/física | Somente NavGoal; capacidades, apoio, colisão Rapier, compromisso, timeout e budget antes do trecho. Nó próximo não teleporta o corpo; alinhamento físico corrige saída nominal que falha desde a posição real. Chegada precisa no modo local usa distância² e a mesma tolerância física de2mm do alcance; consulta de apoio do endpoint somente em voo. Testes com limite exato e3mm além dele evitam ampliar essa margem silenciosamente. |
| GPU/JSON/I/O | Duas linhas de marcador ciano por NPC no buffer existente; nenhuma dependência/recurso/thread/lock novo. Configurações serializadas somente na borda, replay tipado inclui conector. |
| Evidência | Dez testes novos: parada sem contato na perseguição, alinhamento físico de âncora perto do limite, direto walk/fly/replay, desvio/ordem, salto autorado, alcance/parede/lacuna, parcial com custo de entrada, memória/troca/budget/privacidade, colisão/modos e atomicidade/histórico/legado. Verificador MCP cobre fluxo físico, perseguição, guias e pixels; resultados em validation.md. |
| Limites mantidos | Rede explícita, segmentos lineares, apoio em caixa comum, sem navmesh/desvio dinâmico/steering contínuo. Custo nominal não é duração observada. Sem profiling ou promessa de FPS. |


## Aproximação parcial — antes de codificar

1. Política opcional por NPC, independente de capacidades: stop padrão ou approach_reachable com min_progress e max_path_ticks. Validação fria, cena/replay versionados.
2. Reusar o mesmo Dijkstra e arrays de 64 entradas; selecionar candidato entre nós já alcançados, distância XYZ²/custo/índice. Rota completa tem preferência, sem novo scan de geometria por candidato parcial.
3. Exigir melhora mínima sobre a posição real inicial e teto de custo estimado do percurso. Preservar caminho parcial inteiro através de desvios que primeiro afastam; não replanejar de volta a cada nó.
4. Estado escalar adicional (nó parcial/raio planejado); Vec de rota existente, sem coleção crescente. Ao chegar, estado distinto de sucesso e sem novas buscas enquanto objetivo/posição permanecem estáveis.
5. Perda/troca de pista termina apenas trecho comprometido; invalida restante, sem consultar alvo oculto. Orçamento investigativo e validação física de lançamento continuam ativos.
6. Guias/capturas/inspeção pelo dispatcher MCP; reutilizar linhas e cores. Testar stop/padrão, aproximação, rota completa preferida, desvios, empate, progresso/custo, memória, replay e atomicidade. Sem profiling/FPS alegados.


## Aproximação parcial — respostas da revisão

| Pergunta | Resposta/evidência |
|---|---|
| Trabalho adicional no caminho quente | Fallback faz um scan de até64 nós já visitados pelo Dijkstra, somente quando não encontrou solução completa. Comparações por distância²; uma raiz para calcular limiar desde posição própria na tentativa. Nenhuma segunda busca/consulta geométrica por candidato. |
| Memória e alocações | Política Copy e dois campos escalares no State; mesma rota Vec prealocada e arrays de64 entradas. Remover primeiro edge desloca no máximo63 índices u16 apenas na chegada ao trecho; custo limitado aceito, sem deque/cópia de rota adicional. |
| Repetição e invalidação | Rota parcial sobrevive a desvios intermediários e partial_reached retorna sem Dijkstra. Objetivo/raio/deslocamento/reset invalidam; falhas de lançamento/budget/timeout limpam cache e preservam retry60ticks. |
| Tipos e limites | Nó u8, custo u32; até64 nós/128links, duração nominal por link limitada10s, max_path_ticks até3600. Desempate estável por distância/custo/índice; margem ARRIVAL0,12m no limiar de progresso. |
| Física e conhecimento | Mesmas capacidades, geometria/apoio, colisão e budget XYZ. Nenhum lookup adicional de ator/alvo; somente destino conhecido. Conexão comprometida termina antes de trocar intenção. |
| Serialização/GPU/I/O | JSON somente na borda MCP/replay; policies construído sob demanda no inspect. Marcador usa2 linhas no Vec/cache existente por NPC; nenhum recurso GPU, dependência, thread ou lock novo. |
| Verificação | Cinco testes Rust adicionais: desvio/backtracking/replay, padrão/rota completa, limites/empate/ordem, expiração/troca de pista, erros atômicos/versão/reset. Roteiro MCP cobre estado terminal, persistência e pixels. Resultados completos em validation.md. |
| Custos e limites mantidos | Dijkstra continua scan limitado; lançamento ainda revalida trajetória. Rede explícita, proximidade euclidiana e custo nominal não são planejamento tático nem duração física garantida. Sem profiling ou afirmação de FPS/escala. |


## Locomoção espacial — antes de codificar

1. Rede explícita limitada a64 nós/128 conexões; capacidades compiladas por NPC no reset. Dijkstra em arrays fixos, sem heap/fila crescente; custo por tempo estimado, empate por índice autorado.
2. Geometria estática e arcos balísticos verificados na compilação; trajetórias limitadas a10s e600 segmentos físicos. Execução mantém colisões Rapier, prazo e chegada física; bloqueio dinâmico pode falhar sem teleporte.
3. Estado limitado por NPC, Vec de rota reutilizado, sem JSON/I/O/GPU/threads no tick. Diagnóstico e serialização só na borda.
4. Objetivo conhecido apenas; ancoragem explícita próxima à rede. Conexão aérea comprometida termina em ponto seguro antes de trocar intenção, mas não revela posição oculta.
5. Preservar navegador plano legado; rejeitar combinações ainda incompatíveis com contratos XZ (patrulha/território/passagens/recuperação). Budget investigativo espacial mede XYZ e exige orçamento suficiente antes de uma conexão.
6. Cena/replay versionados, comandos MCP, guias XYZ, reset/undo/load, testes de capacidades, física/colisão, expiração/interrupção e regressão. Sem alegação de FPS ou navegação volumétrica livre.


## Locomoção espacial — respostas da revisão

| Pergunta | Resposta/evidência |
|---|---|
| Hot path | Até8NPCs, controle vetorial por tick; Dijkstra apenas ao iniciar/próximo trecho ou retry60ticks. Arrays de64 custos/visitados/predecessores; scan até128links, sem heap de prioridade. |
| Memória | State.route reserva64 entradas no reset; compiled até128 Motion por NPC. Mapas/caches criados no reset. NavState.path reutiliza capacidade após crescimento inicial; Strings reutilizam capacidade. Nenhum histórico crescente. |
| Geometria | Compilação estática no reset; trajetória é revalidada uma vez no lançamento a partir da posição física real. Até600 sweeps/trecho, cenas limitadas128corpos com NPCs. Custo de lançamento conscientemente mantido, não a cada frame. Walk exige caixa de apoio comum, sem amostragem que possa perder um vão estreito. |
| Tipos/custo | Índices u8/u16, contadores saturantes u32, ticks u64; custo inteiro ceil(segundos/DT). Sqrt vetorial e scans limitados de apoio permanecem no motor; sem conversão JSON ou lookup de alvo oculto. |
| Física/budgets | Colisão/CCD Rapier mantidos. Salto preserva vy após impulso; planeio nunca sobe; voo compensa gravidade. Apoio usa footprint completo, tolerância vertical0,06m e baixa vy. Budget XYZ real, limite pré-lançamento com margem0,05m; compromisso pode sobreviver à expiração. |
| Invalidação | Edição/reset/load/undo recompilam. Objetivo novo durante trecho não reescreve endpoint nem inicia continuação antiga. Fly pode pairar, sem capacidade de voo conserva gravidade. Falha dinâmica explicita timeout/ausência de ancoragem. |
| Render/I/O | Mesmos buffers/cache de linhas, até3 primitivas/link mais32segmentos de arco por salto ativo e rotas limitadas. Sem dependência, thread, lock ou I/O novo por tick. |
| Evidência |9 testes Rust novos e3grupos MCP: quatro modos físicos, alcance/capacidade/obstáculo, rede intermediária/ordem, atomicidade, pista expirada, bloqueio dinâmico, budget vertical, incompatibilidades e voo→pouso→salto. Replay/prefixo/PNG idêntico na demo. |
| Limites | Rede explícita não descobre caminhos livres; chegada exige nó compatível, sem navmesh/rampas/coordenação3D. Planeio terminal sem aerodinâmica. Sem profiling, escala/FPS alegados. Ocorrência intermitente de comparação visual legada registrada em validation.md, sem correção alegada. |


## Reserva de passagens — antes de codificar

1. Até8 passagens/8NPCs; políticas/slots pré-alocados no reset, estado escalar por participante. Sem fila crescente.
2. Scan limitado de candidatos com prioridade+idade, empate ID; sem ordenação/heap por tick. Uma reserva por NPC, concessão não preemptiva.
3. Usar apenas destino já conhecido, posição própria e ocupação física da passagem declarada. Coordenação não escreve percepção/memória.
4. Navegador mantém colisão/recuperação existentes. Mudança de waypoint invalida feedback/caminho; espera não cobra distância de investigação, relógios continuam.
5. Validação fria de geometria, referência de superfície, pontos/slots, sobreposição e limites. Geração usa mesmo documento e seed externo.
6. Replay inclui estado limitado; JSON/IDs textuais de inspeção apenas na borda. Guias usam Vec/cache existentes.
7. Testar exclusão, prioridade/aging, ocupação real, cancelamento/timeout, ordem independente, legado, físico/MCP/replay. Sem promessa geral de deadlock/starvation sob bloqueio físico persistente.

## Reserva de passagens — respostas da revisão

| Pergunta | Resposta/evidência |
|---|---|
| Caminho quente | Até8x8 candidatos/agentes/trechos. Seleção por scan e desempate ID, sem sort/heap. Direção/comprimento de trecho alinhado usam sinal/abs, sem normalize/sqrt por tick. |
| Memória | Runtime aloca BTreeMap de até8 participantes e Vec de até8 slots no reset; Agent/Slot escalares Copy. Vagas em máscara u16 temporária; nenhum histórico/fila crescente ou alocação por concessão. |
| Ocupação/gate | Ocupação varre cena limitada128 corpos com NPCs; gate usa slab sweep local de dois eixos, posição própria e velocidade. Reaproveita snapshot/maps. |
| Invalidação | Waypoint XZ muda → NavState idle; acomodação vertical não invalida. Cancelamento preserva waypoint anterior até comparar, evitando feedback antigo. IDs resolvidos por índice fixo no reset. |
| Conhecimento/budgets | Coordenador recebe destino conhecido e ocupação; NpcState é referência imutável. Espera/cooldown não cobra viagem investigativa; travessia preserva remaining_travel. Relógios/sensores seguem ativos. |
| GPU/I/O |4 linhas por faixa e2 por vaga (máximo36 por passagem) no Vec/cache existente. Nenhum recurso/draw/lock/thread/dependência novo; JSON apenas inspect/replay. |
| Limites |8 passagens/8NPCs; contadores u32 saturantes; tick até36000 no runtime; uma posse por NPC e slots autorados suficientes validados. |
| Evidência |10 testes Rust novos: exclusão/retomada/replay, desempate/ordem/não preempção, ocupação/atomicidade, timeout, legado/reset, aging/cancelamento, memória/budget, gate/retirada, fila3NPCs/eixoZ e geometria inválida.3 grupos MCP/capturas. |
| Custos mantidos | Navegador global/replanos já existentes e scan de ocupação por trecho. Layout exige autoria; gate pode parar um desvio sem pedido compatível. Sem garantia geral de deadlock ou medição/FPS alegado. |

## Recuperação de navegação — antes de codificar

1. Política opcional; ausência conserva navegação atual. Estado fixo de ticks, posições e contadores, sem fila/histórico crescente.
2. Espera por falha evita reconstrução do mapa/A*; cadência exponencial limitada. Movimento normal mantém replano existente.
3. Progresso compara distância XZ ao ponto de referência, sem acumular trajeto ou consultar alvo oculto; somente comandos efetivos de movimento contam.
4. Chegada/progresso significativo limpam sequência de falhas. Idle, disabled e mudança de objetivo invalidam estado; percepção e budgets não param.
5. Contadores saturantes; nenhuma alocação, thread, lock, GPU ou I/O novo por tick. Inspeção serializa apenas na borda MCP.
6. Configuração validada antes de mutar; cena/replay versionados. Testar recuperação com obstáculo móvel sem reset, stall, fronteiras, legado e replay.
7. Limites: espera introduz latência configurável após liberação; não garante passagem entre agentes nem fuga de mínimos locais; sem promessa de FPS.

## Recuperação de navegação — respostas da revisão

| Pergunta | Resposta/evidência |
|---|---|
| Caminho quente | Branch opcional, relógio e distância². retry_wait retorna antes de World.refresh/A*, sem percorrer obstáculos. |
| Memória/alocações | Config/State Copy, escalares e Option; nenhum Vec/fila nova por tick. Strings de status/motivo existentes reutilizam capacidade após crescimento inicial. |
| Trabalho repetido | Cadência15→30→60ticks na demo; teto3600 configurável. Progresso físico reinicia sequência; movimento normal conserva replano12ticks. |
| Invalidação | Idle/disabled/troca de objetivo/território/reset recriam recuperação. Sensores/budgets continuam. Destino da mesma ação atualizado não cancela espera. |
| Limites | Até8NPCs; contagens u32 saturantes, deadline u64 saturante, shift limitado12, posição XZ f32. |
| I/O/GPU/locks | Nenhuma dependência/thread/I/O novo. MCP serializa estado sob demanda; marcador existente muda de cor, sem primitivas/draws adicionais. |
| Evidência | Seis testes Rust novos, três grupos MCP; liberação física, stall, retorno territorial, interrupção/expiração, reset/legado, replay/pixels. Regressões completas aprovadas. |
| Custos aceitos | Latência até max_retry_ticks após liberação; detector mede deslocamento, não avanço topológico. Planejador global e scans em tentativas mantidos; sem arbitragem multiagente/profiling/ganho de FPS alegado. |

## Território e retorno — antes de codificar

1. Política opcional e estado fixo por NPC; testes XZ por distância², sem consultas ao alvo oculto.
2. Filtrar candidatos externos no scan sensorial existente antes do ranking; não deixar pista externa mascarar alternativa interna.
3. Retorno/cooldown comprometidos pulam sensores e investigação; limpam memória/recibo/suspeita, preservam apenas ponto da patrulha.
4. Invalidação de navegação nas transições, mesmo quando origem coincide com destino anterior; chegada por posição física com tolerância do navegador.
5. Sem coleções por tick, filas ou eventos crescentes; contadores saturantes e enums/Option numéricos.
6. Guias reaproveitam LineGuide/cache existente; círculo unitário calculado uma vez, até64 segmentos por NPC, sem trigonometria por frame.
7. Autoria valida limites e pontos de rotina no círculo; resolução de rota procedural apenas na validação fria/reset.
8. Mesmo dispatcher/ordem/física, nenhum lock/thread/dependência ou I/O novo no tick. JSON sob demanda.
9. Validar retorno exclusivo, aquisição externa, alternativa interna, ocultação, limites, bloqueio, cooldown deslocado, histórico/replay/render/MCP e legado.
10. Limite comportamental: geometria global pode exigir desvio externo, empurrões atravessam círculo, origem pode ser inalcançável; sem promessa de hard constraint/FPS.


## Território e retorno — respostas da revisão

| Pergunta | Resposta/evidência |
|---|---|
| 1. Caminho quente | begin/contains usam escalares/distância² por NPC. Filtro externo antes de reconhecimento/ranking visual e acústico, sem segundo scan de sensores. |
| 2. Memória/alocação | Config/State Copy com enums/Option/contadores; sem listas ou heap novo por candidato/tick. Rotas são resolvidas na validação fria, limitadas a16 pontos. |
| 3. Trabalho evitado | Retorno/cooldown pulam sensores/investigação; limpar Knowledge somente quando threshold!=0 evita rezerar buffer vazio. Nenhuma consulta ao alvo oculto. |
| 4. Ordem/feedback | Mesmo passo de invalidação existente incorpora changed_this_tick do território. Chegada por posição própria, não status de navegação anterior. |
| 5. Escala | Mesmos8NPCs/limites de eventos; contagens u16, episódios u32 saturante, ticks u64. Nenhum histórico crescente. |
| 6. GPU/guias |64 segmentos+2 linhas do X por NPC no Vec reutilizado. Sin/cos e inicialização sincronizada de LazyLock somente uma vez; transformações numéricas por frame, cache evita upload se guias iguais. |
| 7. I/O/locks | Sem I/O, thread ou lock de simulação novo; inicialização preguiçosa da geometria é única. JSON de inspect_territory só na borda. |
| 8. Invalidação | Reset/edição/load/undo/disabled limpam território. Iniciar retorno limpa memória/recibo/suspeita e suspende objetivos/patrulha; replay16 captura estado. |
| 9. Evidência | Oito testes Rust/quatro grupos MCP novos; regressão completa, pixels de replay iguais, fronteiras, privacidade, cooldown, bloqueio e guias sem efeito no gameplay. |
| 10. Custos mantidos | Navegação global/plana e seus scans/replanos existentes; origem inalcançável continua tentando, sem alternativa tática. Limite comportamental explicitado, sem clipping físico ou alegação de FPS. |

## Composição de objetivos — antes de codificar

1. Um scan de até8 objetivos por NPC, sem árvore, ordenação ou segunda consulta sensorial.
2. Configuração vetorial limitada na autoria; estado de execução escalar/enum/máscaras, sem alocação adicional por tick.
3. Condições recebem só observação aceita/memória válida; movimento recebe posição própria e ponto conhecido, nunca alvo oculto.
4. Um dono de navegação por passo; troca invalida feedback/caminho anterior. Ausência de objetivos mantém fluxo legado.
5. Percepção continua ativa durante interrupção; expiração por relógio permanece. Rotina mantém ponto, reinicia etapa; deslocamento não investigativo não consome budget de investigação.
6. Fuga gera destino a partir de pista/direção própria somente em nova evidência ou ativação; reutilizar navegador e cadência de replano.
7. Até8NPCs, mesmas capacidades físicas; sem jobs/threads/locks/dependências/GPU novos.
8. Reset/edição/undo/load limpam seletor; replay inclui estado tipado. JSON/descrições apenas na inspeção MCP.
9. Validar legado, prioridade/empate, fonte/expiração, interrupção/retomada, privacidade, falhas, colisões, MCP e replay/prefixo.
10. Limites explícitos: prioridades estáticas, fuga radial sobre mapa plano, uma pista ativa; sem planejamento tático, diálogo ou combate. Sem alegação de FPS.



## Composição de objetivos — respostas da revisão

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência/escala | Um scan de até8 entradas por NPC. Maior prioridade/primeiro empate em uma passagem, sem sort. Invalidação posterior percorre até8NPCs, custo fixo adicional aceito. |
| 2. Alocações | Configuração Vec limitada na autoria; seletor Copy com escalares/Option/enums/máscaras. Nenhum heap novo por tick/candidato. NavState vazio usado para invalidar feedback cria Vec vazio sem alocar. |
| 3. Tipos/acesso | Índices/prioridades/máscaras u8, contador de transições saturante u32, ticks u64 e pontos f32. Identidade pública só na inspeção. |
| 4. Repetição | Sensores executam uma vez; ações usam recibo. Fuga calcula direção/sqrt somente em nova evidência ou ativação; guarda reutiliza ponto autorado. Sem política, seletor retorna default e preserva fluxo legado. |
| 5. Orçamentos | Percepção expira por relógio mesmo suspensa; record_travel exclui fase suspended. Cursor de busca preservado, sem avanço durante suspensão. Patrulha reinicia apenas etapa ao retomar. |
| 6. GPU/I/O | Reutiliza guias/capturas e navegador; nenhum recurso GPU/draw/readback/arquivo adicional por tick. inspect_objectives monta JSON/config apenas sob demanda. |
| 7. Ordem/locks | Mesmo snapshot pré-física e dispatcher. Troca de índice limpa feedback antes de investigar e caminho antes da rotina/navegação; nenhum lock/thread/dependência novo. |
| 8. Invalidação | Edição/load/reset/histórico recriam State; disabled limpa seletor. Replay15 inclui estado nos NPCs existentes, sem coleção de histórico de decisões crescente. |
| 9. Evidência | Oito testes Rust novos/quatro grupos MCP, regressão completa, privacidade, fronteiras, física/fuga, expiração, feedback, replay/prefixo e persistência. Resultados em validation.md. |
| 10. Custos mantidos | Scans de cena/mapas e replano do navegador continuam limitados; guard/flee sem alternativa tática. Seletor não faz pooling/compilação de árvores especulativos. Sem profiling ou ganho de FPS alegado. |

## Arbitragem sonora — antes de codificar

1. Reaproveitar um único scan de impulsos/atenuação; filtrar candidatos antes de escolher o mais forte para que repetição descartada não masque alternativa válida.
2. Estado escalar limitado por NPC: IDs de ruído, intensidade na recepção, contagens e enum de motivo; sem lista de pistas/strings/heap por tick.
3. Intensidade nominal existente (1-distância/raio efetivo) é congelada ao aceitar, sem recalcular emissor oculto. Repetição espacial usa distância².
4. Políticas opcionais por NPC: compromisso mínimo, margem aditiva, supressão por raio e proteção de memória visual; ausência preserva seleção legada.
5. Visão confirmada atual continua prioritária e evita scan acústico; memória expirada não bloqueia novo som. Rejeição nunca renova tick/viagem/busca.
6. Mesmos limites16 impulsos manuais por passo, emissões automáticas limitadas e8 NPCs; sem fila de candidatos rejeitados.
7. Reset/undo/load/desativação/memória limpa invalidam intensidade/ID ativos; replay14 inclui diagnósticos/estado tipado.
8. Inspeção pelo MCP reconstrói JSON apenas na borda; render/threads/locks/I/O inalterados, guias/rotas existentes explicam demo.
9. Testar prioridades, margem/tempo/fronteiras, repetição, elegibilidade de alternativas, expiração, ruído automático/empates, anonimato, replay/persistência e MCP.
10. Limites: escore de gameplay, não pressão sonora; sem tags de ameaça, fila de pistas, comunicação ou nova arquitetura de decisões. Sem promessa de FPS.

## Arbitragem sonora — respostas da revisão

| Pergunta | Resposta/evidência |
|---|---|
| 1. Caminho quente | `hear` mantém um único scan acústico. `consider` filtra antes do ranking; um candidato repetido mais forte não esconde outra pista elegível. |
| 2. Alocações/cópias | `sound_attention::State` contém apenas escalares/Option/enums; Observation é Copy. Nenhum Vec/String/heap adicional por tick ou fila de pistas por NPC. |
| 3. Aritmética/tipos | Intensidade f32 congelada na recepção; repetição usa distância². Contadores u16 comportam os eventos limitados do passo; ticks usam diferença saturante u64. |
| 4. Trabalho repetido | Visão confirmada pula audição; passo sem impulsos retorna imediatamente. Margem aditiva compara recibos, sem recalcular distância ao emissor oculto. |
| 5. Escala | Mesmos limites de NPCs/eventos/oclusores. Trabalho adicional constante por candidato audível e estado constante por ouvinte; diagnósticos também calculados no modo legado. |
| 6. GPU/I/O | Nenhum recurso visual, upload, readback, áudio de dispositivo ou I/O novo por tick. JSON e busca de configuração por ID em inspect_hearing são sob demanda. |
| 7. Ordem/locks | Mesmo dispatcher/snapshot e ordem de desempate automático→manual; sem thread, lock ou dependência nova. |
| 8. Invalidação | Begin/synchronize/forget alinham recibo à memória; expiração, desativação, reset, edição, load e histórico não deixam intensidade ativa órfã. Rejeição não renova tempo/distância/busca. |
| 9. Contrato | Oito testes Rust novos e quatro grupos MCP; regressão completa, replay/prefixo, erro atômico, legado, expiração e demonstração. Contagens e build em validation.md. |
| 10. Limites | Uma memória ativa, sem fila, ameaça semântica ou identificação sonora. Raio agrupa posições, não identidades; intensidade é escore de gameplay, não acústica física. Sem profiling ou alegação de FPS. |

## Confirmação visual individual — antes de codificar

1. Mesmo sensor, sem segunda consulta por candidato: registrar evidência antes do filtro de confirmação/ranking.
2. Até9 slots fixos por NPC, índices u8 do catálogo; nenhuma lista crescente ou string no tick. Observações só numéricas.
3. Acúmulo/decay em u32: incremento decay_ticks, decremento acquire_ticks, limiar produto (máximo12.960.000); sem erro de integração f32/raiz.
4. Configuração opcional preserva reação imediata; onisciência ignora espera, audição permanece anônima. Confirmado só participa se observado neste tick.
5. Um scan fixo de slots para limpar flags/decair; nenhum scan adicional de geometria. Ausência não consulta posição real.
6. Persistência tipada/replay13; inspeções traduzem IDs/progresso apenas na fronteira MCP. Reset/undo/load limpam evidência.
7. Render reaproveita guias existentes, mudando cor por estado; sem novo draw, geometria ou upload por indivíduo. MCP primeiro.
8. Mesma ordem e snapshot pré-física; nenhum lock/thread/log/I/O por passo.
9. Testes determinísticos de limiar, decay, identidade, ocultação, ruído, modos legado/onisciente, memória/patrulha, replay e MCP; regressão existente.
10. Limites deliberados: não é probabilidade nem reconhecimento facial; uma pista ativa de movimento, sem fila de sons, memória espacial multiobjetivo ou comunicação. Sem promessa de FPS.


## Confirmação visual individual — respostas da revisão

| Pergunta | Resposta/evidência |
|---|---|
| 1. Caminho quente | Mesmo sensor; registro numérico antes da seleção. Observações não são recalculadas para acumular exposição. |
| 2. Memória/alocação | Nove Track fixos por NPC, sem Vec/String/heap adicional no tick. Observation é Copy de escalares; clones antigos nos testes removidos. Custo fixo de estado/replay aceito para o limite de8NPCs. |
| 3. Aritmética/tipos | Escore/limiar u32, índices do catálogo u8, adição saturada pelo limiar e subtração saturante. Divisão para progress ocorre somente na inspeção. |
| 4. Trabalho repetido | Sem recognition, scans de begin/finish e busca de suspeita no frame são pulados. Disabled limpa o buffer somente se houve reconhecimento. Configuração limita produto a3600². |
| 5. Escala | Dois scans fixos de9 slots por passo configurado; custo de sensor/candidatos permanece limitado pelo catálogo atual. Não há lista crescente de memórias. |
| 6. GPU/I/O | Reutiliza guias existentes; muda apenas cor. Nenhum novo draw, geometria, log ou arquivo por passo. |
| 7. Ordem/locks | Sem threads/locks novos; snapshot pré-física compartilhado e desempate estável. |
| 8. Invalidação | Reset/cena/undo/redo zeram registros; desativação apaga exposição e memória ativa. Última posição só muda ao observar. |
| 9. Evidência | Testes de limiar/decay/retomada, memória independente, candidatos ocultos, som, modos imediato/onisciente, patrol, ordem, replay/prefixo, MCP e guias. Contagens/build registrados em validation.md. |
| 10. Limites | Conhecimento individual restrito à exposição visual elegível; uma pista ativa de navegação e prioridade sonora anterior. JSON/IDs somente nas fronteiras; sem profiling ou alegação de FPS. |

## Relações e seleção hostil — antes de codificar

1. Filtrar elegibilidade por matriz de relações; somente sensores consultam posição dos candidatos. Política automática opt-in, alvo explícito legado preservado.
2. Compilar catálogo de até9 atores/índices de entidade e matriz9x9 no reset; consulta de atitude O(1) no tick, sem comparar rótulos/ordenar/alocar candidatos.
3. Até8 NPCs×8 candidatos; observar somente hostis, primeiro alcance/FOV depois oclusão. Manter um melhor candidato e incumbente na pilha, distância² e margem².
4. Identidade de memória/escolha em u8 e entidade de navegação em u16, estatísticas/reasons em enums/escalares. IDs reconstruídos só na fronteira MCP; replay numérico vinculado à cena ordenada/build.
5. Sem seleção baseada em distância não observada; margem só compara dois observados. Som continua sem identidade; um único registro de memória, sem lista crescente.
6. Catálogo/preferências refeitos no reset/edição/undo. Alterar relações é autoria e reinicia sessão; nenhum comando de rede ou runtime retarget ad hoc.
7. Navegação exclui o ator lembrado via índice de objetivo, preservando coordenada conhecida; mapa global/obstáculos físicos existentes continuam limite documentado, sem alegar conhecimento local do mapa.
8. JSON/logs/IDs/detalhes da matriz só em inspeções; mesmo lock/dispatcher, sem nova thread/draw/uploadGPU ou I/O por tick.
9. Testar relações/direção/defaults, visibilidade, empate, histerese, identidade/ruído, limites/invalidação/ordem/replay e MCP; regressão de modos explícito/sem alvo.
10. Limites:8NPC/1player/128corpos,32 facções/64 regras; sem dano/ameaça/reputação/memória multiator. Mais sensores custam O(NPC×atores×oclusores); limite explícito, sem promessa de escala/FPS.


## Relações e seleção hostil — respostas da revisão

| Pergunta | Resposta/evidência |
|---|---|
| 1. Caminho quente | Simulation usa catálogo persistente em update_with_context; elegibilidade precede sensor/ranking, sem alteração da ordem percepção→patrulha→navegação→física. |
| 2. Alocações/cópias | Seleção usa dois Option de observação numérica na pilha; sem Vec/String/JSON/ordenação adicional no tick. Clones de Observation copiam apenas escalares. APIs auxiliares de percepção recompilam catálogo por chamada, fora do caminho de Simulation. |
| 3. Tipos/acesso | Matriz9×9, slots u8 e índices de entidade u16; distâncias3D² evitam raiz no ranking. Cache ordenado por ID garante desempates sem strings durante a escolha. |
| 4. Trabalho repetido | Resolução de regras/rótulos só no reset; comparação O(1) por candidato. Scans de entidade e BTreeMap de posições permanecem no sensor existente. |
| 5. Escala/budgets | Até8 NPCs/9 atores/128 corpos;32 facções/64 regras. Custo sensorial pode crescer até8×8×oclusores; não foi ocultado nem substituído por otimização especulativa. |
| 6. GPU/I/O | Nenhum novo overlay, upload, arquivo ou log por tick; JSON só em autoria/inspeção/replay. |
| 7. Locks/ordem | Nenhum lock/thread novo. Todos observam o mesmo snapshot pré-física; autoria invertida preserva resultados. |
| 8. Invalidação | Edição final validada recompila catálogo no reset; undo/redo/load idem. Memória numérica nunca atravessa troca de cena. |
| 9. Evidências | 105 testes Rust release, clippy -D warnings,90 grupos MCP. Troca com margem validada em passos reais; ruído anônimo, replay/prefixo, privacidade de memória e máximo de atores cobertos. |
| 10. Limites mantidos | Uma memória ativa, mapa global/plano, sem ameaça de combate/reputação. Inspeção pode alocar/serializar toda a pequena matriz; não ocorre no frame. Sem alegação de ganho de FPS ou escala medida. |


## Autonomia de NPCs — antes de codificar

1. Nenhum loop de seleção multi-alvo: alvo continua um ID opcional resolvido na consulta sensorial; retirar acoplamento a player sem ampliar8NPC/128corpos.
2. Option<String> só em autoria/configuração; observação e NavGoal continuam numéricos, sem novo JSON/string no tick.
3. Sem alvo, pular consulta visual/oclusão; audição/rotina existentes continuam. Nenhuma busca implícita pelo jogador.
4. Snapshot de posições pré-física já compartilhado preserva atualização simultânea e evita dependência da ordem entre NPCs.
5. Pista anônima usa dimensões do próprio ouvinte para navegação; não precisa resolver alvo nem suas dimensões. Observação visual usa dimensões autorais do alvo conhecido.
6. Navegação exclui alvo somente quando Some; sem alvo todos os demais corpos continuam obstáculos. Não mudar física/caches/render sem necessidade.
7. Validação/versionamento na borda: versões antigas exigem player, novas aceitamNone/NPC; erro não toca runtime/histórico. Replay11 e testes de migração.
8. Sem nova alocação por frame/tick, lock/thread/GPU/I/O; custo assintótico igual, menos consulta quando não há objetivo visual.
9. Testes de sem jogador, patrulha/ruído/retomada, NPC-NPC visível/oculto, ordem/ciclos, referências inválidas, undo/persistência/replay e MCP real.
10. Mantidos mapas/scans existentes, grade plana e alvo único; facções/utility/behavior trees são decisões posteriores explícitas. Sem profiling como bloqueio.


## Autonomia de NPCs — respostas da revisão

| Pergunta | Resposta/evidência |
|---|---|
| 1. Caminho quente | **Mantido:** mesma sequência percepção→patrulha→navegação→física; sem seleção automática ou etapa extra. |
| 2. Alocação/cópias | **Sem acréscimo no tick:** Option de ID é configuração, consultada por referência; memória/objetivo mantêm campos existentes. |
| 3. Tipos/acesso | **Explícito:** None não usa string sentinela ou ID global; consulta visual retorna antes de buscar posições/oclusores. |
| 4. Trabalho repetido | **Reduzido no caso sem alvo:** elimina scan de alvo/oclusão; pista anônima usa o próprio NPC em vez de buscar metadados de outro ator. |
| 5. Escala | **Mantida:** oito NPCs/128 corpos, um objetivo explícito; ciclos de IDs não geram recursão. |
| 6. GPU/I/O | **Inalterado:** renderer/overlay/captura reaproveitados; persistência somente nos comandos existentes. |
| 7. Locks/ordem | **Inalterados:** snapshot pré-física compartilhado; referências mútuas observam posições anteriores no mesmo tick. |
| 8. Invalidação | **Existente:** autoria validada antes de reset/undo; referência removida em lote não deixa alvo antigo no runtime. |
| 9. Evidências | **Aprovadas:**97 testes Rust e85 grupos MCP, incluindo invariância de memória/rota para NPC oculto e ordem autoral invertida. |
| 10. Custos/limites | **Mantidos:** scans/mapas e grade plana; sem facções, candidatos, novos budgets ou medição de FPS. Cenas legadas exigem player antes da migração. |

## Seleção de entidades — antes de codificar

1. Picking/inspeção apenas por comando/clique; mover cursor só guarda coordenadas. Contorno atualizado somente quando há seleção, sem mudança de simulação.
2. Seleção guarda ID e índice u16 resolvidos no comando/reset de cena; frame lê entidade por índice, sem procurar ID nem clonar strings. Uma caixa numérica opcional no frame/cache.
3. Segmento near/far obtido da mesma matriz da renderização; interseção de caixas por slabs, mantendo menor parâmetro e desempate por ID. Sem ordenar/alocar lista de hits.
4. Até512 caixas por clique; omitir invisíveis. Seleção explícita por ID pode mostrar invisível para diagnóstico. Nenhum raycast usado automaticamente por NPC.
5. Contorno12 arestas/144 vértices (duas fitas cruzadas por aresta para não degenerar nas verticais) no buffer/pipeline existente; sem nova textura/draw. Cache compara presença/conteúdo e remove sem resíduos.
6. JSON e consulta de componentes só em inspect_selection; não montar snapshot completo/histórico de eventos para um item selecionado.
7. Mesmo lock/dispatcher. Clique e MCP compartilham validação; seleção incrementa somente view_revision quando muda, sem reset/entrada física ou revisão autoral.
8. Reset preserva ID e recompõe índice; remoção/renomeação limpa. Replay isolado herda guias/seleção válida, sem alterar live; nenhum formato novo.
9. Testar câmera/viewport, closest/tie/miss, invisível, índice após edições, contorno/cache/captura, replay e estado inalterado. Clique físico permanece separado no Mac bloqueado.
10. Limite atual: caixas alinhadas visíveis, não picking de triângulos/GUI/guia. Contrato de editor, sem ECS/assets/ABI novos e sem alegação de FPS.

## Seleção — respostas da revisão estática

| Pergunta | Resposta/evidência |
|---|---|
| 1. Caminho quente | **Limitado:** picking só em clique/comando. Cursor guarda dois f32; frame só trata uma caixa selecionada. Nenhum novo trabalho no tick físico. |
| 2. Alocações/cópias | **Controlado:** ID alocado no comando e índice u16 cacheado; frame/Hit/contorno são Copy. Inspeção aloca JSON/ocupantes somente quando solicitada. |
| 3. Tipos/acesso | **Mantido:** floats de projeção/física, slabs numéricos; índice direto autoral, uma consulta ao mapa físico para posição selecionada, sem scan de IDs por frame. |
| 4. Repetição | **Evitada:** comandos repetidos não mudam view_revision; GuideCache não recalcula/reenvia geometria igual. Título formata ID emprestado junto da string já existente. |
| 5. Escala | **Limitado:** até512 caixas por picking, melhor hit em espaço constante, sem vetor/sort. Uma seleção e144 vértices extras; cache mantém capacidade após limpar. |
| 6. GPU/I/O | **Reutilizado:** mesmo draw/buffer/pipeline dos guias. Duas fitas cruzadas por aresta evitam vertical degenerada; captura somente sob comando. |
| 7. Locks | **Mantido:** mesmo dispatcher, sem thread/lock novo; chamada nativa set_title após liberar serviço. Picking linear é frio e limitado. |
| 8. Invalidação | **Verificada:** reset/reordenação recalculam índice; remoção/rename limpa; undo não ressuscita seleção; replay copia somente ID existente. |
| 9. Contratos | **Verificados:** 91 testes Rust e82 grupos MCP; posição atual/projeção/resize, near/far, empate/visibilidade, estado/replay, cache/capturas e erros atômicos. |
| 10. Custos mantidos | **Explícitos:** O(entidades) por clique; todos os guias reconstruídos se um muda; título já atualiza com tick; largura mundial/overlay sem oclusão. Sem profiling/FPS alegado; clique físico pendente. |

## EAF-11 — posse da entrada na perda de foco, antes de corrigir

1. Problema reproduzido com janela: após run_scenario isolado, input MCP x1 virou0 por evento Focused(false); mundo/tick permaneceram iguais.
2. Usar contador u64 interno por comando de entrada válido/reset e uma Option<u64> na janela; sem origem textual, JSON por frame ou mudança de replay.
3. Teclado registra sua revisão sob o mesmo lock de SetInput. Perda de foco libera somente se ainda possuir essa revisão; comando MCP mais recente, mesmo com valor idêntico, prevalece.
4. Ignorar mudanças inexistentes de teclas para impedir releases tardios após limpar o conjunto; repetição não precisa reenviar o mesmo estado contínuo.
5. Sem alocações, GPU/I/O, snapshots ou loops novos; comparação O(1) apenas em evento nativo, estado constante por sessão.
6. Reset/edição invalidam posse antiga. Entrada inválida não altera revisão nem posse. O contador não é posição de replay nem revisão autoral.
7. Testar ausência de posse, liberação nativa válida, MCP diferente/idêntico, reset e input inválido; repetir regressão MCP com janela e preservar cobertura de modificadores.
8. Teste físico de teclado continua separado no Mac bloqueado; correção verificável com os mesmos handlers tipados e regressão de janela.

## EAF-11 — respostas sobre posse da entrada

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | **Adequado:** revisão atualizada só em SetInput válido/reset; conferência somente em evento de perda de foco. |
| 2. Alocações | **Adequado:** u64 na Engine e Option<u64> no teclado; nenhuma coleção, string ou JSON novo. |
| 3. Tipos | **Adequado:** revisão interna de sessão separada da revisão autoral/tick/replay. Igualdade de valor não determina posse. |
| 4. Repetição | **Corrigido:** teclas sem mudança de estado não reenviam SetInput; releases tardios após limpar keys não anulam MCP. |
| 5. Escala | **Constante:** comparação O(1), estado constante por sessão, sem trabalho por entidade. |
| 6. GPU/I/O | **Não se aplica:** nenhum recurso visual ou persistência novos. |
| 7. Locks | **Adequado:** aplicar/registrar posse e conferir/liberar ocorrem sob o mesmo lock do serviço, sem janela de corrida. |
| 8. Ciclo de vida | **Validado:** reset/edição invalidam revisão antiga; input inválido não muda revisão; foco consome posse uma vez. |
| 9. Contrato | **Validado:** sem posse, teclado ativo, MCP igual/diferente, reset e comando inválido; regressão real com janela preserva input. |
| 10. Restante | **Mantido:** última mudança explícita de tecla ainda pode assumir controle; não é arbitragem multiplayer. Aceite físico de atalhos segue pendente. |

## Edição atômica em lote — antes de codificar

1. Caminho frio de autoria/MCP;1..512 operações. Nenhuma chamada ao novo código por tick/frame.
2. Clonar cena uma vez como staging, mover payload das operações, validar resultado final uma vez. Mover cena anterior ao undo em vez de clonar novamente e descartá-la.
3. Enum de edição separado de Command para impedir runtime/I/O/recursão no lote. Entidade em Box mantém enum compacto; consumidor move valores sem clone.
4. Um reset de física/cache/replay por lote, um incremento de revisão e uma entrada no histórico. Comandos individuais reutilizam o mesmo caminho via iterador de um item, sem Vec extra.
5. Até512 operações, cena final≤512 entidades; staging pode ter até1.024 antes de remoções finais. Scans lineares frios O(operações×entidades) explícitos; sem índice/estrutura nova.
6. Sem GPU/I/O na aplicação; frame/captura existentes observam estado final. Save/load continuam comandos separados.
7. Mesmo lock/guard/revisão. Staging impede estados parciais observáveis, não cria concorrência ou transação distribuída.
8. Erro preserva mundo/inputs/playback/replay/histórico/revisões; sucesso mantém reset/pausa e invalida redo como qualquer edição. Empty batch é erro.
9. Testar referências futuras e renomeação, ordem/remove ausente/final inválido, revisão obsoleta, limite, undo/redo único, paridade com edição individual e captura/replay.
10. Cada upsert substitui entidade completa, não merge de campos; última escrita na sequência vence, referências são verificadas no final. Nenhuma promessa de FPS/latência, operação de autoria limitada.

## EAF-17 — respostas sobre edição atômica

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | **Adequado:** apenas autoria explícita, sem alteração de tick/frame. |
| 2. Cópias | **Corrigido:** staging clona a cena uma vez; entidades/câmera/luz movidas do payload; replace move cena anterior ao undo. Simulation mantém sua cópia durante o único reset final. |
| 3. Tipos | **Adequado:** SceneEdit restrito/sem recursão, Entity em Box; comandos individuais usam once, sem Vec temporário. |
| 4. Repetição | **Corrigido:** uma validação final, uma normalização/sort, uma reconstrução de runtime por lote. |
| 5. Escala | **Limitado:**1..512 operações, staging≤1.024 entidades/final≤512. Scans lineares frios mantidos; teste substitui256 entidades em cena512 com staging768. |
| 6. GPU/I/O | **Não se aplica à aplicação:** sem captura/save no lote; renderer observa somente resultado final e caches existentes. |
| 7. Locks | **Adequado:** guard da revisão antes de staging e commit sob lock existente; nenhum estado parcial visível. |
| 8. Histórico | **Corrigido:** um undo/redo por lote; descarte antes de push no limite32 evita crescimento para33. Erro preserva redo e partida em andamento. |
| 9. Contrato | **Validado:** referências futuras/renomeação, falha local/final/obsoleta, empty/limites, paridade individual e captura/reset/replay. |
| 10. Restante | **Mantido:** upsert integral e ordem explícita, sem transação de arquivos/runtime ou colaboração distribuída. Sem promessa de FPS/latência. |

## Volumes de gameplay — antes de codificar

1. Amostragem pós-física por tick, até32 volumes ×32 alvos explícitos; sem alterar colisão/navegação/percepção.
2. Resolver IDs, ordenar volumes/alvos, reservar arrays/anel ao criar simulação; posições de alvos únicos atualizadas uma vez. Máscara u32 de ocupação e eventos numéricos Copy, sem Strings/JSON por transição.
3. Caixas alinhadas nos três eixos; sobreposição estritamente positiva, sem sqrt. IDs externos decodificados só na inspeção. Até512 atores/1.024 relações, ring512 eventos.
4. Constantes (centro, semi-extensões, alvos/índices) preparadas uma vez; alvos compartilhados evitam busca física repetida para cada volume.
5. Zero trabalho se nenhum volume, nenhum índice espacial novo para esse limite. Colisões do jogo continuam no Rapier; sensor simples usa o mesmo contrato de caixas sem rotação.
6. Visibilidade de entidades filtra montagem de frame/cache; não muda corpo, visão ou som. Marcador visual separado pode mostrar zona invisível. Sem novo draw/pipeline.
7. Mesmo dispatcher/lock, sem I/O por tick; eventos são dados e não executam callbacks/scripts ou mudam objetivos do NPC.
8. Edição/load/undo/reset recriam estado/caches, sem eventos sintéticos de saída no reset. Primeiro tick observa ocupantes iniciais; replay regenera eventos e compara estado.
9. Testar enter/stay/exit, alvo não selecionado, passagem sem bloqueio, limite do anel, fronteiras/3D, ordem determinística, inválidos atômicos, visibilidade/cache, replay/prefixo e cenários.
10. Detecção discreta pode perder passagem que ocorra inteiramente entre ticks; sweep/formas rotacionadas/móveis e filtros por tags são extensões futuras. Sem ganho de FPS/profiling alegado.

## EAF-16 — respostas sobre volumes e visibilidade

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | **Adequado:** Runtime::update pós-física; retorno imediato sem volumes. Não interfere no planejamento/percepção. |
| 2. Alocações | **Adequado:** atores/volumes/alvos/estados e ring512 preparados na inicialização; eventos Copy, pop antes de push. Nenhuma cópia de ID/JSON por transição. |
| 3. Tipos | **Adequado:** índices de atores u16, volume/slot u8, máscara u32; contadores u32 limitados por relações×36.000ticks. IDs decodificados na consulta. |
| 4. Repetição | **Corrigido:** alvos únicos atualizados uma vez, referências/ordem resolvidas uma vez; teste de32 volumes×32 alvos exige32 consultas físicas por tick. |
| 5. Escala | **Limitado:** até1.024 testes AABB/tick,512 eventos retidos,32 volumes; sem busca espacial ou callbacks arbitrários. |
| 6. GPU/I/O | **Adequado:** visible filtra frame existente; cache por conteúdo limpa geometria oculta/restaurada. Nenhum draw/recurso/IO novo por tick. |
| 7. Locks | **Adequado:** mesmo dispatcher e contrato de edição; amostragem usa corpos atuais sem snapshots extras ou locks novos. |
| 8. Ciclo de vida | **Validado:** reset/edição/undo/load zeram estado; primeiro tick emite ocupação inicial; disabled ignora amostragem e seleções são explícitas. |
| 9. Contrato | **Validado:** trajetória idêntica sem sensor, fronteiras/altura, ordem independente, limites/referências atômicos, ring20.480 eventos/512retidos, replay/prefixo/cenário2 e captura. |
| 10. Restante | **Mantido:** amostragem discreta/f32, caixas fixas sem rotação; sweep e filtros de população futuros. Cópias existentes de cena em Engine/Simulation foram explicitadas na revisão de EAF14/15. |

## Inspeção/captura de replay — antes de codificar

1. Comandos sob demanda; reconstruir do início até o tick pedido, sem tocar no tick vivo. Mesmo limite de 36.000 entradas/4.096 ruídos/8MiB do replay.
2. Compartilhar validação e reconstrução com run_replay; cópias de cena para Engine/Simulation por consulta, sem snapshot/JSON por tick nem segunda execução para verificar o final.
3. Tick u64 validado antes de converter/indexar; entradas e impulsos tipados, cursor monotônico. Inputs/ruídos do prefixo copiados uma vez para inspeção e guias, não a cada passo.
4. Snapshot e comparação final apenas após reconstruir; prefixo não afirma correspondência do final. Sem checkpoints/cache de simulação neste recorte.
5. Eventos de ruído no próprio tick selecionado permanecem pendentes; passos regenerados pelo runtime. Input exibido é último aplicado (ou padrão em zero), não entrada pendente que o formato não grava.
6. Captura reutiliza Renderer/GPU/cache existentes, uma imagem de 64..2048 por dimensão; pixels/arquivo só sob demanda. Câmera/autoria do replay, guias da sessão.
7. Comandos síncronos sob lock existente; janela pode aguardar durante replay/readback. Sem nova thread/dependência/ABI. Limite de ticks não promete latência.
8. Paths/output validados antes de reconstruir; impedir sobrescrever arquivo de origem por nome/symlink/alias de diretório. Partida viva/replay/revisões intactos.
9. Validar zero/intermediário/final, ruído pendente/consumido, passos, build/inputs/limites inválidos inclusive sufixo, replay adulterado, imagem igual à captura viva, caches e preservação de estado.
10. Não é savegame/seek da partida/timeline, não altera formato9. Reexecutar O(ticks) por consulta é compromisso explícito; checkpoints/jobs exigirão decisão própria.

## EAF-15 — respostas sobre inspeção/captura de replay

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | **Adequado:** somente chamada explícita; Engine::replay_at não modifica o loop vivo. |
| 2. Alocações | **Mantido com motivo:** cópias existentes de cena em Engine/Simulation e cópia limitada do prefixo de inputs/ruídos por consulta, sem snapshots por tick; possibilita reutilizar Engine/diagnóstico/render. |
| 3. Tipos | **Adequado:** tick u64 limitado por len≤36.000 antes de conversão; entradas/ruídos tipados e cursor usize. |
| 4. Repetição | **Corrigido:** validação/reconstrução únicas para três comandos; cena validada uma vez em Engine::new; comparação final sem ressimular. |
| 5. Escala | **Limitado:** consulta O(tick) + validação do documento inteiro; 36.000 inputs, 4.096 ruídos, 8MiB. Sem cache de simulação/checkpoints. |
| 6. GPU/I/O | **Adequado:** capture_frame compartilha Renderer/cache; uma imagem até2048², readback/PNG/escrita só por comando. Inspect não cria GPU. |
| 7. Locks | **Mantido com motivo:** serviço síncrono, como run_replay/capture existentes; janela pode aguardar. Nenhuma promessa de latência/FPS. |
| 8. Invalidação | **Validado:** alternância cena viva/replay, viewport e guias produz pixels exatos; paths/destinos inválidos preservam origem e partida. |
| 9. Contrato | **Validado:** zero/prefixo/final, pista pendente/consumida, passos, sufixo inválido, build, adulteração, empty replay, aliases e estado vivo/replay preservados. |
| 10. Restante | **Mantido:** revisão/playback/entrada ainda não aplicada não estão gravados; snapshot reconstruído explicita defaults. Sem savegame/timeline/formato novo. |

## Cenários de verificação — antes de codificar

1. Ferramenta de teste sob demanda, isolada da partida; até 256 ações e orçamento conservador de 3.600 ticks por execução. Sem mudança no loop do jogo.
2. Validar documento inteiro, referências e soma dos máximos antes de criar simulação. Reusar Engine/inputs/ruídos/física; snapshot e serialização somente em checkpoints/fim.
3. Predicados tipados de posição/percepção/patrulha/passos; until testa dados Rust diretamente, sem JSONPath, strings convertidas ou árvore de JSON por tick.
4. Até 256 resultados de checks, interromper no primeiro erro/timeout; não armazenar snapshot por tick. Relatório determinístico sem relógio de parede.
5. Cena autorada copiada para Engine isolada e seu Simulation; custo de inicialização e lookup limitado aceito nesta ferramenta. Snapshot final/replay sob demanda são fronteiras frias.
6. Save/inspect/run pelo MCP; root/limites/symlinks/escrita atômica existentes. Relatório e replay opcionais em caminhos explícitos, sem versões históricas.
7. Execução síncrona sob o lock atual, como run_replay, com limite menor; pode bloquear janela durante a chamada. Jobs/cancelamento exigem contrato próprio futuro.
8. Partida ao vivo, entradas, replay, revisão e preferências não mudam; caso inválido não sobrescreve roteiro existente. Validação de paths antes da execução.
9. Testar acerto/falha/timeout/erro de execução, condição inicialmente satisfeita, budgets/IDs/valores/paths, cena inicial versus estado vivo, repetibilidade e replay exportado.
10. Runner não é linguagem de gameplay nem substitui testes Rust/aceite humano. Sem infraestrutura, dependências, threads, GPU ou profiling novos.

## EAF-14 — respostas sobre cenários de verificação

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | **Adequado:** ferramenta sob demanda; não entra no tick da partida viva. Condição until avaliada por tick somente na simulação isolada. |
| 2. Memória | **Adequado:** checkpoints com capacidade reservada, no máximo 256; um snapshot final, sem histórico de snapshots. Engine/replay usam limites existentes. |
| 3. Tipos | **Adequado:** enums/structs com valores/referências validados; predicados acessam estado Rust, sem JSONPath/JSON por tick. |
| 4. Repetição | **Mantido:** consultas por ID limitadas por condição; validação antes de criar Engine. Report convertido uma vez para Value e compartilhado entre arquivo/resposta. |
| 5. Escala | **Limitado:** soma conservadora de máximos ≤3.600 ticks, ≤256 ações, primeiro erro encerra; sem recursão/loops livres. |
| 6. GPU/I/O | **Adequado:** runner não cria GPU/áudio; leitura/escrita só nas fronteiras. Exportar replay é opcional e reutiliza o contrato existente. |
| 7. Locks | **Mantido com motivo:** execução síncrona sob lock do serviço como replay; janela pode aguardar. Jobs/cancelamento são contrato futuro, sem promessa de latência. |
| 8. Isolamento | **Validado:** autoria de origem preservada, partida viva/sua entrada/replay/revisão intactos; erros não sobrescrevem cenário; destinos distintos incluindo aliases. |
| 9. Contrato | **Validado:** acerto/falha/timeout/erro operacional, condição inicial/limite inclusivo, budgets/referências/paths, relatório idêntico salvo/devolvido, repetição e replay exportado. |
| 10. Restante | **Mantido:** cópias da cena e serialização final são custos frios; dois outputs não têm transação conjunta. Sem scripting geral, suite distribuída ou profiling. |

## Guias de rotas — antes de codificar

1. Preencher somente com npc_routes ativo, na montagem do frame/captura; sem alteração de sensores, rotina, navegação ou tick.
2. LineGuide numérico Copy em Vec reutilizado, cache por conteúdo e buffer de guias existente; sem strings/configuração no renderer.
3. Até oito NPCs, 16 pontos/arestas de patrulha cada e 256 segmentos de navegação por NPC. Capture informa caminhos truncados.
4. Linhas autoradas tracejadas com oito traços fixos por aresta; no máximo 2.448 LineGuides/20.064 vértices extras, independente de distância.
5. Usar somente rota resolvida, caminho/destino conhecidos e posição própria do NPC; não consultar alvo oculto.
6. Sem novo draw/pipeline/texture/thread/I/O. Guia partilha comparação/invalidação/cache/GPU existentes.
7. Patch MCP/tecla J, visualização de sessão e revisão visual; V/H preservam rotas. Não incrementar revisão de documento.
8. Limpar listas/truncamento por frame; reset/load/remoção/desativação e viewport cobertos; buffers retêm capacidade limitada.
9. Validar ausência de alterações na simulação/replay, posição de memória congelada, caminho/autoradas distintos, erro/limite, cache e capturas.
10. Overlay no plano do piso é diagnóstico e pode atravessar objetos; linhas autoradas não garantem caminho transitável. Sem FPS/profiling alegado.

## EAF-13 — respostas sobre guias de rotas

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | **Adequado:** montagem do frame somente com opção ligada; desligada apenas limpa listas/contador. |
| 2. Memória | **Adequado:** LineGuide Copy e Vec/cache reutilizados; sem clone de path/cena ou strings no renderer. |
| 3. Tipos | **Adequado:** coordenadas/cor f32, estilo bool e contador de truncamento u8. |
| 4. Repetição | **Adequado:** usa rota já resolvida e path/destino conhecidos; nenhuma busca/sensor/geração procedural repetida. |
| 5. Escala | **Limitado:** 256 segmentos/NPC e 16 pontos/arestas; até 2.448 linhas/20.064 vértices extras. Tracejado tem oito partes fixas por aresta. |
| 6. GPU/I/O | **Adequado:** cache/buffer/draw de guias existentes; upload só quando conteúdo muda. Captura permanece sob demanda. |
| 7. Locks | **Adequado:** mesma sessão/dispatcher; nenhum lock/thread/async novo. |
| 8. Invalidação | **Validado:** mudança de estilo/cor, crescimento/redução/vazio, toggle, reset, remoção/undo, desativação e resize. |
| 9. Contrato | **Validado:** replay/simulação inalterados, sem posição oculta, metadados de corte, composição com visão/audição e destino inacessível. |
| 10. Restante | **Mantido:** overlay sem oclusão e projeção plana; caminho completo só no MCP quando truncado. Sem FPS medido. |

## Patrulha — avaliação antes de codificar

1. Rotina opcional após percepção e antes da navegação; somente NPCs configurados (até oito). Sem pista, fornece um objetivo autorado; com pista, suspende.
2. Resolver até 16 pontos por NPC uma vez na criação/reset. Runtime fixo de fase/índice/direção/timers/contadores; nenhuma coleção por tick.
3. Pontos XZ numéricos, Y derivado do piso e altura do NPC; enum de rota e percurso. Receita circular com semente, trigonometria só ao resolver a rota.
4. Reutilizar NavGoal/A*/velocidade/física; sem ler coordenadas do jogador. Consumo de resultado de navegação exige destino e tick pertencentes à perna atual.
5. No máximo uma transição/novo destino por NPC/tick; prazo de viagem e cooldown quando todos os pontos falham. Não tentar todos os destinos no mesmo tick.
6. Sem renderizador, GPU, áudio, I/O, thread ou scheduler novo. Configuração/inspect pelo dispatcher/MCP.
7. Interrupção limpa timers transitórios e mantém índice; resume tentativa ao ponto interrompido. Reset/edição recriam tudo.
8. Estado separado da memória/percepção, incluído no replay e snapshots. Cenas anteriores sem rotina mantêm comportamento; nova versão explícita.
9. Verificar loop/ping-pong/espera, bloqueio/timeout/cooldown, aquisição/interrupção/retomada, alvo oculto, semente, erros/migração/histórico/replay.
10. Mapa conhecido e navegação plana permanecem; não confundir pontos autorados com localização inferida do jogador. Custo de A* permanece limitado e não medido.

## EAF-12 — respostas sobre patrulha

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | **Adequado:** estágio opt-in após percepção; sem rotina o mapa está vazio. |
| 2. Memória | **Adequado:** até 16 pontos por NPC resolvidos no reset, timers/contadores fixos, bitset u16; sem Vec/clone de rota por tick. |
| 3. Tipos | **Adequado:** fases/falhas/percurso tipados; índice u8, direção i8, timers u64, coordenadas f32. |
| 4. Repetição | **Adequado:** trigonometria só ao resolver receita; reaproveita NavGoal e resultado anterior associado à tentativa atual. |
| 5. Escala | **Limitado:** oito NPCs, uma transição por tick. Todos os índices falhos geram cooldown; repetições ping-pong não contam como índices distintos. |
| 6. GPU/I/O | **Não se aplica ao novo tick:** nenhum recurso visual/áudio/I/O; serialização de diagnóstico/replay só sob demanda. |
| 7. Locks | **Adequado:** mesmo dispatcher/fixed-step; nenhum lock/thread/scheduler novo. |
| 8. Invalidação | **Validado:** edição/reset/histórico recriam rotas/estado; interrupção limpa timers e mantém índice; cooldown limpa tentativas ao retomar. |
| 9. Contrato | **Validado:** espera, loop/ping-pong, seed, falhas/timeout/cooldown, visão/som prioritários, passos próprios ignorados, alvo oculto independente, replay e adulteração. |
| 10. Restante | **Mantido:** lookups/scans limitados e A* existente; pontos podem ser inacessíveis. Sem promessa de conectividade, navmesh, comportamento geral ou FPS. |

## Modos de locomoção — antes de codificar

1. Perfis Copy com velocidade/escala de ruído, resolvidos ao construir a simulação; seleção enum por tick sem strings/lookup extra.
2. Input ganha enum walk/sneak/run com default walk; replay armazena modo junto aos eixos. Nenhuma coleção ou alocação no tick.
3. Componente opcional no jogador; velocidade [0.1,10] e escala [0,4], zero produz silêncio. Raio efetivo positivo limitado a[0.1,100].
4. Mesma distância física/apoio/cadência dos passos; escala aplicada só na emissão. NPC mantém configuração própria de velocidade/ruído.
5. Sem aumentar limites de entidades/eventos/histórico; sem threads/dependências/áudio/GPU novos.
6. Shift=run, Control=sneak, prioridade sneak; teclado e MCP usam SetInput. Foco perdido restaura default e solta teclas.
7. Incluir modo no stamp visual para atualizar título pausado; não incrementar revisão de documento ou reiniciar gameplay.
8. Reset/edição reiniciam modo/estado; cena/replay8, entradas antigas sem modo continuam walk.
9. Verificar perfis/erros atômicos, velocidade, alcance/silêncio, combinação/release de teclas, replay e migração.
10. Furtividade não altera colisor/altura/animação; sem promessa de agachamento ou profiling.

## EAF-11 — respostas sobre modos de locomoção

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | **Adequado:** seleção de perfil uma vez por tick; resolver componente na construção da simulação. |
| 2. Memória | **Adequado:** structs Copy e enum no Input; nenhuma coleção/alocação por seleção ou passo. Replay mantém o limite de 36.000 entradas. |
| 3. Tipos | **Adequado:** enum tipado em Rust/schema; f32 preservado, valores finitos e limites validados antes da edição. |
| 4. Repetição | **Adequado:** usa o perfil já escolhido para velocidade e ruído; cálculo de distância/contato existente é reutilizado. |
| 5. Escala | **Adequado:** só o jogador recebe perfis; limites de emissores/filas/anel permanecem. |
| 6. GPU/I/O | **Adequado:** nenhuma geometria/upload novo; modo no stamp atualiza título pausado, sem frames extras quando inalterado. |
| 7. Locks | **Adequado:** teclado usa dispatcher, sem novo lock ou thread. |
| 8. Invalidação | **Validado:** reset/edição/histórico reiniciam entrada e caches físicos; foco perdido limpa teclado, pausa preserva entrada e congela tempo. |
| 9. Contrato | **Validado:** velocidade, diagonal, ruído furtivo, silêncio zero, clamp, migração, validação atômica, modos no replay e release/prioridade de teclas. |
| 10. Restante | **Mantido:** controlador do jogador distinto da navegação NPC, sem postura/animação/estamina. Limites e defaults são do laboratório; sem profiling/FPS alegado. |

## Passos físicos configuráveis — antes de codificar

1. Componente opt-in em jogador/NPC; no máximo9 emissores. Medir deslocamento pós-física com intenção e contato no piso configurado.
2. Estado numérico por emissor, resolvido na criação/reset; fila reutilizada de até 9 impulsos e anel de 256 eventos recentes. Não clonar IDs/config por tick.
3. Eventos Copy numéricos; índice u16 na cena normalizada somente para filtrar o próprio ouvinte. Memória continua sem identidade.
4. Distância XZ efetiva, limiar de velocidade e comprimento do passo. Sem callbacks/strings/JSON no tick; no máximo um impulso por emissor/tick.
5. Audição percorre slices de comandos e passos; sem fila duplicada por NPC. Comandos manuais mantêm limite4096/16; passos não o consomem.
6. Passos são regenerados no replay a partir de cena + entradas; comparar estado/impulsos finais. Histórico automático limitado não exige log crescente.
7. Inspeção e configuração pelo dispatcher; nenhuma thread/lock/I/O/GPU novo. Guias reutilizam os eventos e cache existentes.
8. Reset/edição/load/undo limpam acumuladores/filas. Versões cena/replay 7; cenas antigas sem componente continuam silenciosas.
9. Validar silêncio, parede, ar, cadência, autoexclusão, recepção por outro NPC, fila/anel, replay, referência e migração/atomicidade.
10. `Command::Upsert` passa a carregar Box<Entity>: uma alocação por edição, evitando inflar todos os comandos frequentes com o crescimento de componentes. JSON/schema inalterados para essa indireção.
11. Aproximação plana sem animação de pés nem som de dispositivo. Contato e intenção evitam sons por drift/empurrão passivo; no profiling obrigatório.

## EAF-10 — respostas sobre passos físicos

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | **Adequado:** somente emissores opt-in (até 9), após física; percepção consome fila anterior. Sem emissores, loop vazio. |
| 2. Memória | **Adequado:** estados criados no reset, pending com capacidade de emissores, VecDeque de 256 pré-alocado; Noise Copy sem strings. Sem fila por NPC nem log automático crescente. |
| 3. Tipos | **Adequado:** índices u16 de entidade/piso resolvidos uma vez, IDs automáticos u32 e tick u64; f32 para distância/cadência. |
| 4. Repetição | **Adequado:** usa snapshot anterior e corpos existentes; consulta contato por par de colisores sem varrer contatos. Cálculo XZ uma vez por emissor. Guias mesclam duas listas ordenadas sem sort/clones. |
| 5. Escala | **Limitado:** máximo9 estímulos automáticos + 16 manuais por tick; anel de 256 e contadores de descarte. Uma emissão por personagem/tick coalesce deslocamento excepcional. |
| 6. GPU/I/O | **Adequado:** guias existentes, sem I/O/áudio de dispositivo no tick. Estado/histórico serializados somente em inspeção/replay. |
| 7. Locks | **Adequado:** mesmo dispatcher/loop; Upsert usa Box para reduzir enum de comandos, uma alocação adicional somente ao editar. |
| 8. Invalidação | **Validado:** reset/edição/histórico recriam estados e referências; fila é consumida antes de limpar e gerar próxima. Cenas antigas sem emissor mantêm comportamento. |
| 9. Contrato | **Validado:** física real, silêncio no ar/contra parede, própria emissão ignorada, outro NPC audível, ponto congelado, anel saturado e replay regenerado com detecção de adulteração. |
| 10. Restante | **Mantido:** piso único fixo por emissor, locomoção plana e intenção como aproximação de animação; lookup de corpos por ID nos mapas existentes. Sem ganhos de FPS declarados. |

## Guias de audição e ruídos — antes de codificar

1. Visualização opcional por sessão, patch MCP sem reset de simulação; H alterna audição/ruídos pelo mesmo dispatcher.
2. Snapshot numérico: no máximo oito ouvintes e dezesseis impulsos recentes (90 ticks). Vec/cache/GPU reutilizados.
3. Ler alcance e eventos existentes, sem repetir sensor acústico nem consultar identidade do emissor.
4. Varredura reversa interrompida na idade máxima e limite de quantidade; nenhum scan do log inteiro por frame.
5. Círculos com 48 segmentos fixos, sem crescimento por raio/distância; 288 vértices por ouvinte e 156 por impulso.
6. Compartilhar draw/pipeline de guias; nenhuma nova textura, thread, I/O ou readback.
7. Comparação de conteúdo invalida geometria; emitir ruído pausado invalida revisão visual quando necessário.
8. Impulsos congelados na origem, expiração por tempo de simulação; pausa conserva evidência. Reset/load limpam eventos, preferência visual persiste.
9. Validar patch/idempotência, legenda, emissão pausada, limite/expiração, captura/replay e controles antigos.
10. Contorno nominal em plano horizontal não calcula transmissão/oclusão nem garante que um ruído seja ouvido; sem áudio de dispositivo ou FPS alegado.

## EAF-9 — respostas sobre guias acústicos

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | **Adequado:** snapshots só preenchem listas habilitadas; sem alteração de sensores/ticks. |
| 2. Memória | **Adequado:** RingGuide numérico Copy; três Vecs de guias e um de vértices retêm capacidade. Nenhum clone de log/cena por frame. |
| 3. Dados | **Adequado:** centro/radius existentes; não consulta identidade ou posição atual do emissor de uma pista antiga. |
| 4. Trabalho repetido | **Adequado:** cache compara conteúdo; reverse/take_while/take limita consulta a16 eventos mais um teste de expiração. Trigonometria apenas quando guia muda. |
| 5. Escala | **Limitado:** 8×288 vértices de ouvintes +16×156 de emissores; no máximo4800 extras, além dos2448 de visão. |
| 6. GPU/I/O | **Adequado:** mesmo buffer/draw/pipeline das guias; buffer cresce somente por necessidade. Nenhuma textura ou leitura GPU adicional. |
| 7. Sincronização | **Adequado:** patch tipado no dispatcher; H lê um bool sob lock curto e chama comando fora dele. |
| 8. Invalidação | **Validado:** ruído pausado aumenta view_revision; expiração por ticks, reset/undo/load/remoção/toggle/resize cobertos. |
| 9. Contrato | **Validado:** replay idêntico ao alternar visualização; controles antigos preservados, patch vazio idempotente, sensores não dependem das guias. |
| 10. Restante | **Mantido:** projeção horizontal nominal e draw sobre objetos; sem promessa de visibilidade/audibilidade. Geometria de todos os guias reconstruída quando qualquer guia muda, limitada a24 anéis e8 setas. Sem profiling. |

## EAF-6 — recuperação de contato: antes de codificar

1. Executar recuperação somente quando a origem já viola a margem do planejador.
2. Até oito direções fixas, sem Vec temporário, clone, JSON ou novo estado persistente.
3. Reutilizar caixas expandidas e posição física; permitir apenas saída da margem de 6 cm e até 5 mm de tolerância física.
4. Candidatos curtos (12 cm), destino livre e segmento validado contra todas as caixas; nunca atravessar um obstáculo novo ou aprofundar contato.
5. Custo limitado a oito scans dos obstáculos existentes; A* adiado até recuperar espaço livre.
6. Sem alterações GPU/I/O/locks, formatos ou novos controles; MCP e janela compartilham a correção.
7. Velocidade e distância restante continuam limitando movimento real; física resolve contatos, sem teleporte.
8. Invalidar plano durante recuperação; nenhuma posição do alvo oculto usada para escolher a saída.
9. Regressão com as coordenadas observadas, cantos, bloqueio real, distância/replay e regressões MCP de navegação/percepção/busca/audição.
10. Conexão final da grade ao destino: segmento de até uma célula, checagem varrida e heurística conservadora; sem ampliar a tolerância de chegada. Verificação extra de obstáculos apenas próxima ao destino; cálculo do candidato por nó expandido, sem novas coleções.
11. Recuperação local não substitui depenetração geral ou navegação 3D; nenhum ganho de FPS alegado.

## EAF-6 — respostas sobre recuperação de contato

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | **Adequado:** `World::recovery` somente com origem sem folga e objetivo conhecido; ticks normais seguem caminho anterior. |
| 2. Alocações/cópias | **Adequado:** oito vetores em array fixo e candidato local; nenhum Vec novo, clone de cena ou JSON. Path/scratch e strings retêm capacidade. |
| 3. Tipos/dados | **Adequado:** Vec2/f32, caixas e snapshot físico existentes; memória sonora e identidade continuam separadas. |
| 4. Repetição | **Mantido com motivo:** scans de segurança por candidato; conexão final calcula aproximação por nó do A*, mas consulta obstáculos apenas no alcance de uma célula. |
| 5. Escala | **Limitado:** oito candidatos de 12 cm; limites existentes de obstáculos/grade. Heurística mais conservadora pode expandir mais nós para alcançar a região contínua. |
| 6. GPU/I/O | **Não se aplica:** nenhuma alteração de render, I/O ou dependências. |
| 7. Locks | **Adequado:** mesma etapa da simulação e mesmo dispatcher MCP/janela. |
| 8. Invalidação | **Corrigido:** recuperação limpa caminho/instante de planejamento; plano recomposto ao sair da margem. Sem cache adicional. |
| 9. Contrato | **Validado:** velocidade/saldo de distância continuam limitando deslocamento; estado recuperável não é confundido com destino impossível. Regressão física nas coordenadas relatadas, cantos, obstáculos finos, sobreposição profunda e replay. |
| 10. Restante | **Mantido com motivo:** navegação plana/discretizada, direções locais fixas e tolerância física de 5 mm. Não é depenetração geral nem garantia de solução em qualquer geometria. Sem profiling/FPS alegado. |

## Guias visuais de percepção — antes de codificar

1. Overlay de diagnóstico opt-in por sessão; nenhuma alteração em decisão, navegação ou física.
2. Snapshot compacto por NPC (até oito); reutilizar Vec de guias/vértices e buffer GPU. Sem cópias de cena/JSON no frame.
3. Usar posição/orientação/estado já existentes; não repetir raycasts nem fazer inferência visual de detecção.
4. Gerar geometria apenas se posição, yaw, configuração ou cor mudar; ângulos calculados para geometria alterada, segmentos limitados.
5. Contorno nominal de alcance/FOV, sem tesselação proporcional a distância; no máximo 48 segmentos por arco e tracejado fixo nos raios.
6. Um draw adicional somente quando há guias; buffer persistente com crescimento limitado, sem I/O/GPU readback adicional.
7. Mesmo dispatcher para MCP/teclado. Estado visual separado da revisão do documento e simulação; invalidação da janela pausada por revisão visual.
8. Desligar/reset/load/remoção/undo/resize precisam remover ou atualizar geometrias sem resíduos; offscreen e janela usam mesmo frame.
9. Validar orientação/sinal yaw, cores/estados, captura on/off, independência do gameplay/replay, histórico e repetição de capturas.
10. Contorno ignora oclusão propositalmente e é identificado como nominal; seta mostra orientação atual, cor mostra última amostra. Sem profiling/FPS alegado.

## EAF-7 — respostas sobre guias visuais

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | **Adequado:** snapshot só preenche guias com opção ligada; não altera tick/sensores. Cache vazio inalterado retorna imediatamente. |
| 2. Alocações/cópias | **Adequado:** structs compactas sem strings; Vec de frame/cache/vértices retém capacidade. Nenhum clone de cena, lista de entidades ou JSON por frame. |
| 3. Dados | **Adequado:** posição/yaw/configuração/resultado existentes; nenhum acesso novo ao jogador oculto e nenhum raycast duplicado. |
| 4. Repetição | **Adequado:** igualdade dos guias evita geometria/upload repetidos; trigonometria só em mudança. Regerar todos os guias (máximo oito) é compromisso local explícito. |
| 5. Escala | **Limitado:** até 306 vértices por NPC perceptivo, 2448 para oito; arco fixo 48 segmentos alternados e raios com 12 traços cada. Crescimento de buffer por potência de dois. |
| 6. GPU/I/O | **Adequado:** buffer persistente e até um draw extra; zero draw de guias vazias. Pipeline e buffer mínimo de seis vértices inicializados mesmo desligado. Sem I/O/readback adicional além de capturas solicitadas. |
| 7. Locks | **Adequado:** dispatcher existente e snapshot sob lock; tecla V não mantém lock ao chamar comando. Título/GPU seguem fora do lock. |
| 8. Invalidação | **Validado:** revisão visual separada provoca redraw pausado; caches comparam conteúdo, reset/load/undo/remoção/off/resize sem resíduos em testes CPU/GPU. |
| 9. Contrato | **Validado:** replay e simulação iguais com overlay on/off; captura idêntica após restaurar estado. Cor deriva da amostra anterior, sem simular sensor no render. |
| 10. Restante | **Mantido com motivo:** contorno nominal sobreposto e projeção no piso; busca linear de superfície por até oito NPCs. Sem profiling ou ganho de FPS declarado. |

## Audição — avaliação antes de codificar

1. Eventos pontuais consumidos no próximo tick; nenhum som implícito por frame ou movimento.
2. Log único reutilizável de até 4096 ruídos por sessão, até 16 no mesmo tick; slices emprestadas para percepção/replay, sem fila por NPC.
3. Posição/radius/tick numéricos; origem resolvida uma vez no comando. Memória guarda ID numérico da pista, nunca acompanha emissor oculto.
4. Rejeitar por distância ao quadrado antes de consultar oclusores; comparar intensidade apenas entre candidatos audíveis. Visão tem prioridade.
5. Custo limitado a NPCs × eventos do tick × entidades; sem scan acústico nos ticks sem evento. Manter limites atuais de entidades/NPCs.
6. Sem áudio de dispositivo, dependências, recursos GPU ou I/O por tick. Captura e log completo são sob demanda.
7. Mesmo dispatcher e lock; teclado emite o mesmo comando MCP. Sem thread/ABI/bus genérico novo.
8. Reset/edição/histórico limpam eventos e cursores; replay inclui ordem, tick e posição congelada, inclusive eventos ainda pendentes.
9. Validar silêncio, alcance, transmissão, prioridade visual, desempate, memória congelada, limites/erros atômicos, replay e migração.
10. Propagação por segmento é aproximação explícita: sem difração/tempo de voo/acústica física. Novo ruído pode renovar investigação; evento não prova identidade do jogador.

## EAF-6 — respostas sobre audição

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | **Adequado:** seleção somente no tick do impulso; `hear` retorna imediatamente sem eventos ou audição habilitada. Visão válida dispensa acústica. |
| 2. Alocações/cópias | **Adequado:** um `Vec<Noise>` limitado por sessão e cursor de consumo, slices emprestadas; `Noise` é Copy numérico, sem strings/referência a emissor e sem coleção por NPC. Log cresce no comando, nunca por frame silencioso. |
| 3. Tipos/acesso | **Adequado:** id u32, tick u64, coordenadas/raio f32; origem resolvida só no dispatcher. Observação carrega `Option<u32>`, sem guardar entidade oculta. |
| 4. Repetição | **Adequado:** distância ao quadrado filtra antes de percorrer oclusores; sqrt apenas para pontuar candidato audível. O replay usa cursor monotônico, não varredura integral a cada passo. |
| 5. Escala | **Mantido com motivo:** até 8 NPCs × 16 eventos × 512 entidades, com limite físico anterior de 128 corpos; até 4096 eventos/sessão. Scans de IDs/oclusores conhecidos, sem promessa de escala de mundo aberto. |
| 6. GPU/I/O | **Adequado:** nenhum recurso/upload/readback novo, nenhum áudio de dispositivo ou I/O por tick. Log completo somente por `inspect_noises`/replay sob demanda; snapshot comum só inclui contagens. |
| 7. Locks | **Adequado:** mesma fronteira atômica do dispatcher; N resolve ID de jogador sob lock curto e usa comando compartilhado. Sem thread/lock/bus novo. |
| 8. Invalidação | **Adequado:** reset/edição/load/undo/redo zeram log/cursor; pause preserva pendentes. Rejeições não consomem IDs/eventos; Vec retém capacidade limitada para reutilização. |
| 9. Contrato | **Validado:** ausência de pista, alcance, atenuação composta, prioridade visual, desempate, memória congelada, aquisição visual, budgets, limites/erros, migração, histórico e replay; cena/replay v6. |
| 10. Restante | **Mantido com motivo:** escolha acústica por segmento e posição exata, mapa conhecido e navegação plana. Captura/persistência continuam síncronas; nenhuma medição/FPS alegada. |

## Investigação espacial — avaliação antes de codificar

1. Decisão por tick, geração de um destino apenas ao iniciar/avançar um ponto.
2. Estado de tamanho fixo (índice, ponto e instante), sem Vec de candidatos; reutilizar mapas e scratch existentes.
3. Configuração numérica opcional; manter f32/ticks, sem dependências/ABI novas.
4. Calcular ângulo/destino somente em transições; replanejar ao mudar um objetivo de memória, sem mudar a cadência da perseguição visual.
5. Máximo de oito pontos, uma tentativa de navegação por vez; mesmos limites de grade/NPCs.
6. Nenhuma mudança de render/GPU/I/O por tick; demonstração e diagnósticos pelo MCP existente.
7. Nenhuma nova thread ou lock; preservar dispatcher/atomicidade.
8. Limpar episódio em observação/reset/edição/desativação/desistência; descartar navegação de destino anterior.
9. Testar ausência de influência do alvo oculto, sementes, desistência, destino inalcançável, redetecção, budgets e replay.
10. Custo adicional de A* quando muda o ponto é intencional/limitado; não representa planejamento semântico de rotas de fuga nem requer profiling nesta fase.

## EAF-5 — respostas sobre investigação espacial

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | **Adequado:** decisão por tick; ponto calculado só ao iniciar/avançar (`perception::explore`). |
| 2. Alocações/cópias | **Adequado:** configuração/estado fixos e Option; sem lista de candidatos, strings usam capacidade existente; mantém mapas/scratch de EAF-4. |
| 3. Tipos/acesso | **Adequado:** contador u8 limitado a oito, semente u32, tempos u64 em runtime; f32 preservado. Sem referências ao alvo na geração do destino. |
| 4. Repetição | **Adequado:** seno/cosseno somente na geração do ponto; plano invalidado na mudança de destino lembrado, perseguição visual mantém cadência. |
| 5. Escala | **Mantido com motivo:** até oito destinos, um por vez, mesmas limitações de NPCs/grade; cada destino pode custar um A*. |
| 6. GPU/I/O | **Não se aplica ao tick novo:** nenhum recurso visual/I/O novo por frame; capturas e save/load continuam sob demanda. |
| 7. Locks | **Adequado:** mesmos locks e dispatcher, sem concorrência ou protocolo novos. |
| 8. Invalidação | **Adequado:** nova observação/reset/edição/desativação limpa episódio; objetivo anterior não é reaproveitado para novo ponto; desistência retém só contadores diagnósticos. |
| 9. Contrato | **Validado:** memória e destino hipotético separados, movimentos ocultos comparados, sementes, budgets, skip, redetecção, histórico/migração e replay exercitados; cena/replay v5. |
| 10. Restante | **Mantido com motivo:** círculo não prioriza portas nem direção de fuga, mapa é conhecido e bloqueio físico pode consumir somente tempo; usar prazo quando necessário. Nenhuma afirmação de FPS ou profiling. |

Obrigatória antes de implementar cada incremento e revisitada sobre o diff final. Não exige profiling nem benchmark nesta fase. Respostas curtas: **corrigido**, **adequado**, **mantido com motivo** ou **não se aplica**, com referência ao trecho pertinente. Corrigir desperdícios claros; registrar decisões estruturais sem transformar o checklist em um bloqueio burocrático. Validar comportamento continua obrigatório.

| Pergunta | O que conferir |
|---|---|
| 1. Com que frequência esse código executa? | Inicialização, edição, comando, frame, tick ou por entidade; evitar trabalho de preparação nos loops frequentes |
| 2. Há alocações, clones ou conversões dispensáveis? | Strings, coleções temporárias, JSON, buffers; reutilizar capacidade com limites e ciclo de vida claros |
| 3. Os dados têm tipos e acesso adequados? | Tamanhos, referências versus cópias, locality, IDs resolvidos; evitar unsafe ou mudança de precisão sem necessidade |
| 4. Repetimos cálculos, buscas ou ordenações? | Constantes/invariantes, resultados já disponíveis, trabalho duplicado; preservar ordem determinística e precisão |
| 5. O custo cresce de forma aceitável? | Loops aninhados, scans, limites explícitos; distinguir custo conhecido de gargalo medido |
| 6. GPU e I/O fazem só o necessário? | Recriação/upload/readback de recursos, chamadas ao SO, logs e persistência nos caminhos frequentes |
| 7. A sincronização protege apenas o necessário? | Duração dos locks, serialização, espera de GPU/disco, contenção; não quebrar consistência para reduzir lock |
| 8. Caches e buffers têm invalidação e limites corretos? | Edição, crescimento/redução, reset, undo/redo, resize, entidades removidas e múltiplos consumidores |
| 9. A otimização preserva o contrato? | Replay/ordem, física, conhecimento limitado, paridade MCP, erros e atomicidade; testes proporcionais |
| 10. O que fica e por quê? | Custo restante, tradeoff e gatilho de revisão; nenhuma promessa de FPS ou escala sem medição |

## EAF-4 — avaliação antes da implementação

1. Caminhos frequentes: `Simulation::step`, percepção/navegação, `App::redraw` e `Renderer::draw`. MCP é eventual.
2. Corrigir mapas de posições/objetivos/velocidades reconstruídos por tick, strings de estado recriadas, temporários de navegação e cópia integral da cena para renderizar.
3. Manter contratos e mapas por ID nesta rodada; usar dados visuais compactos sem strings/configuração de gameplay. Sem ECS/ABI novos.
4. Reutilizar geometria inalterada e buffers; conexão inicial da grade pode escolher o mínimo sem alocar/ordenar candidatos.
5. Scans de entidades e oclusores permanecem limitados pelo laboratório (8 NPCs, 128 corpos com NPCs, 512 entidades); índice espacial é uma decisão posterior.
6. Evitar buffer GPU novo por desenho; atualizar apenas faixa alterada. Título da janela só quando o estado exibido mudar.
7. Montagem textual/base64 da resposta MCP pode ocorrer após liberar o lock. Captura/persistência síncronas exigem uma divisão de serviço maior e ficam explícitas como custo restante.
8. Caches devem comparar conteúdo, não somente revisão/tick; reset e troca de cenas podem repetir valores. Testar remoção, crescimento, aparência/tamanho, movimento e estado parado.
9. Preservar cálculos/ordem dos algoritmos, validar replay, sensores, colisões e janela/captura pelo MCP.
10. Memória retida em buffers troca pequenas alocações repetidas por capacidade reutilizável; vida útil limitada à simulação/renderizador. Nenhum ganho de tempo foi medido.

## EAF-4 — respostas sobre o código implementado

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | **Corrigido:** buffers de simulação por sessão, GPU por renderizador. Janela pausada consulta estado a cada 16 ms, mas só solicita desenho quando há mudança; redraw do SO/resize preservados (`window.rs`). |
| 2. Alocações/cópias | **Corrigido:** `positions_before_step`, mapas de objetivos/velocidades e `NavigationScratch` reutilizados; strings de status usam `clone_into`; contatos numéricos copiam nomes somente em eventos/diagnósticos; `RenderFrame` substitui clone de `Scene`. Buffer de entrada MCP também reutilizado. Crescimento inicial ainda pode alocar. |
| 3. Tipos/acesso | **Adequado ao recorte:** dados visuais fixos de 36 bytes por caixa; índices de contato seguem IDs ordenados. Mantidos f32, structs/enums e mapas por ID; sem unsafe, alteração de precisão, codec ou ABI novos. |
| 4. Trabalho repetido | **Corrigido:** geometria somente para caixas alteradas, sem ordenar candidatos para escolher o mais próximo. **Mantido:** grade recalculada por replanejamento, para respeitar obstáculos móveis e conhecimento do NPC. |
| 5. Escala | **Mantido com motivo:** scans limitados, A* até 16.641 células, oito NPCs. Sem evidência de escala máxima; índice espacial/ECS requer decisão própria. Buffers evitam realocação recorrente, não eliminam complexidade algorítmica. |
| 6. GPU/I/O | **Corrigido:** buffer de vértices cresce apenas quando necessário e recebe a faixa alterada; pausa inalterada não agenda frames; título atualizado só com mudança. **Mantido:** uniform de câmera/luz por desenho, draw único das caixas e readback/PNG sob demanda. Sem instancing/culling novos. |
| 7. Locks | **Corrigido:** protocolo/schema e envelope textual/base64 MCP fora do lock, título fora do lock. **Mantido:** execução atômica do serviço, snapshots, captura e I/O síncronos; separá-los exige contratos de consistência/ordenação adicionais. |
| 8. Cache/limites | **Corrigido:** comparar posição/tamanho/cor, não só revisão; redução ajusta comprimento/draw count. Testes cobrem crescimento, remoção, reordenação, vazio, restauração, movimento, aparência, câmera e viewport. Capacidade retida limitada pelos tamanhos máximos suportados e liberada ao destruir simulação/renderizador. |
| 9. Contrato | **Preservado por construção, sujeito à validação:** mesmos cálculos e desempates, IDs/ordem externos, controle MCP e cena/replay v4. Buffers de saída removem chaves de NPCs ausentes. A evidência final dos testes fica em `validation.md`. |
| 10. Restante | **Mantido com motivo:** alocações de Rapier, registro de entradas/eventos, mapas string, scans, compact snapshot visual, duas cópias de dados visuais (frame/cache) e upload de intervalo que pode incluir caixas inalteradas entre duas alteradas. São compromissos locais explícitos; profiling futuro, sem alegação de porcentagem/FPS. |
## EAF-57 — revisão proporcional do estudo de ciclo de vida

Antes da elaboração: incremento documental, sem alteração de Rust, schema ou dependências. Auditar slots/arrays, consultas sensoriais, limpeza dos pools e reativação física; não substituir armazenamento nem introduzir caches com base em otimização especulativa.

Respostas após leitura do código:

1. Frequência: retorno à vida seria transição esparsa; não reconstruir `Simulation` nem catálogo em cada retorno. Sondar conhecidos mortos amplia consultas visuais e precisa de orçamento explícito.
2. Alocações: contador por ator e referências tipadas cabem em estado limitado; tamanho real de structs/alinhamento precisa ser revisado na implementação. Nenhuma alocação nova nesta entrega.
3. Tipos/localidade: `u8` atual identifica slot estável, não geração. Separar identidade de vida evita strings/lookup textual por efeito, mas aumenta payloads de observação/cast/replay.
4. Repetição: revalidar vida no commit é necessário; atualização de conhecimento só após evidência. Não duplicar reconstrução de NPC/rotas como atalho para ressuscitar.
5. Escala: nove atores atuais; não extrapolar para população dinâmica. Retenção de origens e filas de spawn precisarão de limites próprios; não criar tombstones ilimitados.
6. GPU/I/O: nenhum recurso/pipeline/readback ou arquivo de runtime novo; futura inspeção serializa somente sob comando.
7. Sincronização: proposta mantém ordem de fases/atores e transações; sem thread/lock/ECS novo. Retorno no mesmo tick exige consumir a transição de morte antes da reativação.
8. Invalidação: vida e instância são domínios distintos. `known_dead`, `deactivated_actors`, casts, periódicos, auras e rádio exigem política explícita, não limpeza global indiscriminada.
9. Semântica: matriz de aceitação em lifecycle-study.md. Revisão documental não substitui futuros testes Rust/MCP/replay; não houve modificação executável que justifique repetir toda a suíte agora.
10. Custos mantidos: autoria dinâmica no impacto/pulso, scans limitados e catálogo fixo preservados no recorte proposto. Sem ganho de FPS, benchmark ou capacidade de ressurreição alegados.

## Plantas paramétricas — revisão do código implementado

| Pergunta | Resposta/evidência |
|---|---|
| 1. Frequência | `plants::Definition::sample` e `Entry::evaluate` executam na autoria/validação/consulta; o runtime recebe somente Asset avaliado. Nenhuma fase nova de gameplay. |
| 2. Alocações/cópias | Scene.plants clona definições e Arc da fonte, compartilhando Geometry/Rig/Clip. Reamostragem aloca geometria e correspondências somente na avaliação. Validar compara um avaliado temporário por planta; custo limitado e deliberado. |
| 3. Tipos/acesso | Dimensões/posições f32; preservação e medição usam f64. IDs autorais separados de índices por face/célula, sem ponteiro/slot de NPC. Sem dependência/unsafe/ABI novos. |
| 4. Repetição | Correspondência O(1) por amostra pela célula do perfil; geração O(V+T), medição O(Vfonte), sem busca espacial. Validação pode reconstruir novamente após staging; preserva contrato no load e detecta cache adulterado. |
| 5. Limites | Contagens após validar segmentos1..63, máximo4096v/8192t por asset.16 plantas; fonte canônica e avaliado obedecem separadamente budgets globais. Falha antes de publicar; sem promessa de escala de produção. |
| 6. GPU/I/O | RenderFrame/cache/buffers EAF-76 reutilizados. Fonte e avaliado persistidos dentro dos limites existentes8MiB arquivo/2MiB envelope MCP. Fonte duplicada no documento é custo explícito de inspeção e reconstrução sem dependência externa. |
| 7. Sincronização | Edição/preview/validação síncronos no dispatcher existente; sem thread/lock novos. Não é cooking assíncrono. |
| 8. Invalidação | Edição direta explícita rebasa fonte; regeneração não. Perfil alterado, referência ausente, excesso de erro/pesos/budget ou revisão antiga recusam staging. Desvinculação explícita preserva asset independente. |
| 9. Contrato | Rust testa orientação, correspondência, não uniformidade, rig/clipe, fonte sem drift, retesselação/load/cache adulterado/preview/undo/lote. MCP cobre GPU/replay/prefixo/save/load/reset e redução que apaga relevo. Evidências finais em validation.md. |
| 10. Custos mantidos | CPU skinning e normais flat existentes; fonte triangulada limita detalhe; tolerância é erro amostrado, não Hausdorff contínuo. Não há solver CAD, transporte geral de normais, remesh livre, cage, IK ou ganho de FPS alegado. |

## Perfis e seções da espada — antes de implementar

1. Avaliação eventual na autoria/inspeção; nenhuma nova fase por tick.
2. Reusar fonte Arc e correspondências existentes; alocar domínio explícito e até32 origens de ossos somente na avaliação.
3. Estações tipadas, IDs autorais estáveis e domínio f32, cálculos de preservação f64; sem dependência/unsafe nova.
4. Localizar intervalo por índice material; inversão física percorre até16 estações por osso, não por triângulo. Sem nearest-surface search.
5. Preflight usa contagens exatas com segmentos longitudinais por intervalo e budgets existentes antes de gerar geometria.
6. Sem novo pipeline GPU, readback periódico ou I/O por frame; capturas somente no teste.
7. Dispatcher síncrono existente; sem locks/threads novos.
8. Identidade e ordenação das seções validadas contra referência; cache deve concordar com fonte e definição.
9. Gates: A-B-A, mapeamento não affine do repouso, deltas TR, pesos, fonte invariável, erros/versão, undo/persistência/replay/GPU e regressão headless.
10. Custos mantidos: reconstrução na validação, fonte+cache persistidos e skinning CPU limitado; sem solver CAD, IK, mapa de frames, autointerseção global ou promessa de FPS.

## Perfis e seções da espada — respostas sobre a implementação

1. Frequência: `Definition::sample`, `Entry::evaluate` e inspeção na autoria; nenhuma fase de simulação/render nova.
2. Alocações: posições/coordenadas e correspondências limitadas por4096v; até32 origens em Vec, scratch global de32 frames na pilha; fontes Arc preservadas.
3. Tipos: estações tipadas e IDs ASCII, domínio f32 e mapas de volume/repouso f64; sem dependências novas.
4. Repetição: correspondência por face/célula O(1), inversão percorre no máximo15 intervalos por osso; validação da fonte repetida antes de acessar hierarquia evita índices malformados no load.
5. Limites:2..16 estações e segmentos por intervalo, contagens exatas antes da geração, mesmas cotas por asset/documento e influência máxima4 sem poda.
6. GPU/I/O: renderer e skinning existentes reutilizados; cenas de demonstração cerca de866kB cada, abaixo8MiB; sem trabalho periódico de arquivo/readback.
7. Sincronização: staging e preview síncronos no dispatcher; nada novo em locks/threads.
8. Invalidação: incompatibilidade de IDs/tipo/ordem, ossos externos, folds, perda amostrada e versões antigas recusados; regeneração não rebasa, edição direta continua explícita.
9. Contrato: Rust cobre domínio material, rig fora do eixo, políticas/deltas TR e ausência de drift; MCP cobre ciclo completo/GPU e regressão headless. Evidência final em validation.md.
10. Custos mantidos: CPU skinning, reconstrução redundante limitada na validação e persistência de fonte+cache. Frames/twist conservados, sem solver CAD/IK/interseção global ou ganho de FPS alegado.

## EAF-83 — machado composto, antes da autoria

Conteúdo/script apenas; núcleo, formatos e dependências permanecem os de EAF-81.
1. Geração eventual pelo MCP, sem fase nova por tick; playback diagnóstico reutiliza skinning existente.
2. Reusar assets de faixas/lâminas em instâncias; evitar um mesh idêntico por ferragem.
3. Dimensões/seções e transformações tipadas existentes; dados Python apenas conduzem operações nativas.
4. Não recalcular loft fora da engine; curva e relevo aplicados pelos pincéis nativos, rigs/pesos/clipes pelo mesmo dispatcher.
5. Preflight durante autoria: até16assets/32instâncias/32768triângulos desenhados; reduzir amostragem onde não representa detalhe.
6. Capturas de frente/detalhe e replay somente sob demanda; nenhuma textura/pipeline ou cópia de build nova.
7. Cliente headless isolado; janela existente só recebe a cena final após verificações.
8. Reuso das fontes canônicas, A-B-A e erros atômicos; transformação de peças continua explícita, sem anexação implícita.
9. Validar preservação em duas instâncias da lâmina, persistência/undo/replay/prefixo/pixels e inspeção visual das capturas.
10. Custos/limites: normais flat atuais, uniões por sobreposição e ausência de constraints entre peças. O exercício deve registrar essas limitações, sem vender conteúdo como solver novo.

## EAF-83 — respostas após autoria

1. Nenhuma fase nova; criação e testes ocorrem por comandos esparsos.
2. Dez assets para26 peças, com lâminas/bandas/couro/gemas reutilizados por instância.
3. Script conduz dados tipados existentes; sem triangulação ou loft alternativos em Python.
4. Curvatura e sulcos produzidos por dois strokes nativos; avaliação das plantas permanece canônica.
5. 3052v/4416t armazenados e7552t desenhados; maior asset958v/1568t; cotas preservadas.
6. Cenas cerca2.04MB cada, capturas sob demanda e único binário existente reutilizado.
7. Cliente isolado sem janela, demo só após os gates.
8. Fonte/rig/clipe/A-B-A/undo/save-load e replay/prefixos verificados.
9. Três grupos próprios,702 testes Rust, Clippy/fmt e7cenários/37grupos MCP relevantes aprovados.
10. Custos herdados de skinning/normais flat, sobreposição e vínculos de montagem explícitos mantidos; limitações em docs/native-axe.md, sem alegação de FPS/realismo.

## EAF-85 — junção orgânica: revisão proporcional do primeiro kernel

Antes de codificar: checklist e alternativas em docs/mesh-organic.md, alinhados
com a frente central da árvore. Hull O(n^4) limitado a96 amostras; evitar usá-lo
para malha inteira. Sem fase de gameplay, dependência, thread ou I/O de kernel.

Após código: entrada no máximo96 vértices;3refinos internos limitados pelo Plan,
que faz preflight. Geometry f32, construção/hull/volume f64, dados da fonte por
empréstimo imutável. Frames/colares validados antes de índices; fronteiras
orientadas e Euler são invariantes independentes da captura. Relaxamento aloca
um snapshot por passo (máximo8), mantendo colares; nenhum cache novo. Identidade
só de colar, nunca declarar transferência no interior novo. Temporários de hull,
mapas topológicos e validação repetida são custos mantidos na autoria limitada.
Renderer/GPU atuais reutilizados só no experimento explícito. Sem FPS alegado.
Amostragem máxima gera2175v/4256t, ainda dentro do asset; não garante orçamento
agregado da árvore. Forma convexa foi reprovada mesmo com testes topológicos
aprovados; integrar na árvore exige mais trabalho geométrico.

## EAF-85 — campo e consulta: avaliação e respostas

Antes: manter a topologia limitada, adicionar campo de três tubos com centro
comum e40 bisseções por vértice livre; nenhuma amostragem volumétrica/marching,
nearest-surface ou material novo. Guardas/contornos e envelopes explícitos; a
consulta deve ser pura mesmo com runtime em play. Contrato em mesh-organic.md.

Após: até2271v/4448t com as faixas de guarda, dentro das cotas existentes.
Snapshot de posições e dados temporários por avaliação na autoria; sem cache,
thread, lock ou trabalho por tick novo. f64 no campo/volume, quantização f32
medida por resíduo. IDs só nos colares, interior sem transferência presumida.
O input usa Box<[Collar;3]> para não inflar Command; serde/JsonSchema estritos
com defaults explícitos. Dispatcher síncrono pode ficar ocupado pelo cálculo
limitado; saída compacta fica sob o envelope existente, sem I/O/preview GPU
implícitos. Publicação, GPU, replay/histórico continuam no caminho existente.
Proposta preserva âncoras altas por split de Bézier, não por busca/compensação
oculta. Native query e biblioteca comparadas exatamente; validação final em
validation.md. Sem alegação de FPS/escala/realismo completo.
## EAF-87 — plantas curvas por seções

Revisão anterior e alternativas em [curved-plants.md](curved-plants.md).

1. `Definition::sample/Entry::evaluate` continuam na autoria/validação, sem nova
   fase de runtime. Formato56/60 não altera gameplay/render.
2. Map temporário de chaves inteiras deduplica a superfície elíptica; grades de
   índices temporárias por face. Fonte Arc/materials/rig compartilhados mantidos.
3. Mapa e inversa f64, saída f32; serde/default estritos. Nenhuma dependência,
   unsafe, ABI, thread ou cache novo.
4. Geração O(V log V) limitada; inversa percorre no máximo15 intervalos por osso.
   Sem busca espacial por ponto. Correspondência por face/célula O(1) mantida.
5. Contagens exatas de superfície fechada e E=3T/2 antes do refino. Limites por
   asset/documento/desenho conservados; offsets finitos +/-100m.
6. GPU/renderer/canais de material existentes reutilizados. I/O só sob comando;
   receita descreve estações e nunca triangula em Python.
7. Dispatcher síncrono e validação antes de publicar preservados; sem bloqueios
   ou scheduler adicionais.
8. Topologia de formas antigas mantida. Tipo/IDs incompatíveis recusados; fonte
   não rebasa por regeneração. Aparência/canal da fonte acompanham a avaliação.
9. Inversa interior/borda, manifold/χ2/contagens, pai rotacionado, escultura,
   coordenadas/weights/TR, A-B-A, erros/persistência/replay: evidências finais em
   validation.md. Preview/capturas não comprovam interação física nem realismo.
10. Custos mantidos: validação/reconstrução duplicada limitada, fonte+cache e
    skinning CPU. Planos paralelos e interpolação linear são domínio declarado,
    sem transporte de normais ou frames, mapa geral de árvore ou ganho de FPS.

## EAF-87 — interpolaçãoC1, antes do código

Escolha PCHIP opt-in versus tangentes Hermite autorais e domínio em
[curved-plants.md](curved-plants.md). Compilar15×4×4coeficientesf64 uma vez
por sample, scratch fixo e sem alocações por vértice; consultas/bones apenas
na autoria. Y permanece invertível sem solver; Linear usa operações antigas
exatas. Sem dependência/renderer/cache/thread/budget novos; mesma transação,
fonte Arc, canais de material e testes de invalidação. Testes matemáticos e
visuais separados; malha finita não herda automaticamente continuidadeC1.

### EAF-87 — respostas após interpolaçãoC1

1. Frequência: compile/values_at na autoria/validação; nada por tick.
2. Alocações:1920B fixos de coeficientes por sample e scratch limitado para
   spans/secantes/tangentes; geometria/fonte Arc existentes. Sem Vec por ponto.
3. Tipos: f64 nos canais/cúbicas/inversa, f32 na geometria; enum Linear/Pchip
   serde/schemars estrito. Sem dependência, unsafe ou thread.
4. Repetição: coeficientes compilados uma vez em sample; consultas/bones
   recompilam no máximo15intervalos, custo pequeno e explícito na autoria.
5. Limites: mesmos2..16estações, espaçamento mínimo0.001 e budgets;
   interpolação não aumenta contagens automaticamente.
6. GPU/I/O: pipelines/canais existentes, capturas sob demanda; mais amostras
   só por segmentos explícitos,706v1408t na comparação densa do ramo.
7. Sincronização: dispatcher/staging existente e preview puro; sem locks.
8. Invalidação: alvo OU referência versionados; Linear omitido/exato;
   regeneração/modo não rebaseia fonte, undo/load/replay existentes.
9. Correção: racionais/C1/extremos/positividade, inversa/ID/rig/TR/material,
   quatro casos visuais com resolução controlada e replay/PNG; validation.md.
10. Custos mantidos: fonte+cache/reconstrução repetida, skinning CPU e
    amostragem linear de superfície; sem frames transportados, interpolação
    adaptativa, junção/attachment automáticos ou ganho de FPS.


## EAF-89 — vínculos de instâncias/ossos, antes e depois

Contrato/alternativas e checklist anterior ao código em
[mesh-attachments.md](mesh-attachments.md). Reusar rig existente evita novo
domínio de frame/curva nesta unidade;58/62 coordenados. Avaliação posterior:

1. Adequado: Hierarchy na validação/invalidação, Frame estático retorna cedo;
   não entra no tick de gameplay. Skeleton uma vez por instância preparada.
2. Mantido com motivo: arrays limitados32×32posed+skin, cerca132KiB de payload
   temporário por preparação mais scratch existente; sem Vec por osso/frame.
   Fonte Geometry/Rig/Clip continuaArc; JSON/Pose alocam só por consulta.
3. Adequado: IDs/nomes resolvidos em índices, escala composta verificadaf64 e
   matrizesf32 existentes; sem unsafe/dependências/threads.
4. Corrigido: poses compartilha Prepared em vez de reavaliar a cadeia para
   cada filho. Frame recompila hierarquia só ao mudar documento.
5. Adequado:32instâncias/32ossos por asset, ciclos e escala composta limitada;
   sem aumentar geometria/draw/assets/instâncias. Árvore9/31 e64794t.
6. Adequado: mesmo Vertex/shader/sombra e upload, só capturas explícitas e
   paths correntes. Sem novo pipeline/readback no tick.
7. Não se aplica: dispatcher síncrono existente, sem lock/fila/thread nova.
8. Validado: mudanças de pai/rig/asset/undo/reset/load invalidam Frame;
   dependência ausente/ciclo falha antes de publicar runtime/histórico.
9. Validado: composição analítica, osso nomeado, clock próprio/ancestral,
   ordem livre/chain/ciclos/formato e mesma árvore salva; pose/captura/replay
   compartilham world. Matriz de ossos mantém espaço do asset anterior.
10. Mantido com motivo: preparação finita/skin CPU/upload e snapshots do ensaio.
    Extração por71v114t verificou arrays completos, rig/clipe autorados na
    referência original e canal material restaurado explicitamente após
    roundoff1.34e-7m. Não alegar FPS ou floresta/vento físico.

## EAF-90 — ilhas e extração de meshes

Contrato e10respostas antes/depois em [mesh-islands.md](mesh-islands.md).
Partição por índices apenas em autoria/query; subsets/cópias de pesos depois
de preflight de assets/V/T/clips/chaves. Ossos/clipes inteiros mantidos, clipes
compartilham Arc mas contam no orçamento global. Preview genérico clona staging
e valida pelos mesmos caminhos; custo de serialização do documento completo é
mantido sob demanda. Sem trabalho novo por tick, dependência, renderer/layout,
thread ou teto ampliado. Árvore reorganizada conserva64794t e pixels;32slots
ocupados são limitação observada, não prova de necessidade de um novo sistema.
Sem alegação de FPS/LOD/realismo. Evidências em validation.md.

## EAF-91 — detalhes de superfície e ajuste de domínio

Revisão anterior, correções e10respostas finais em
[surface-details.md](surface-details.md). Detail Arc até32segmentos por material,
plano2D explícito, distância/afunilamento/footprint por fragmento habilitado;
sem scan de geometria no shader. Tabela1360→19312bytes e Vertex52bytes mantido.
Renderer só reconverte receitas quando mudam, upload também cacheado. Fit faz
partição/AABB/remapeamento somente na autoria; resultado inicializado no centro
sem Vec por isolado. Custos finitos de shader/gradiente, metadados/JSON e staging
continuam. Nenhum budget geométrico ou gameplay ampliado; sem FPS presumido.


## EAF-97 — trajetória de plantas, antes/depois

Checklist e escolhas anteriores em plant-trajectories.md.

1. Adequado: somente edição/validação/consulta; nenhum tick novo.
2. Adequado: dois polinômios cúbicos na stack, compilados uma vez por sample;
   frames por ponto/probe, sem Vec intermediária por avaliação. Source/rig/clips
   conservam Arc; nenhuma cópia de malha para cada frame/probe.
3. Adequado: controlesf32/temporáriosf64,3vetores por frame, identidade estável.
   Curva e seção separadas; legado não é reinterpretado.
4. Mantido com motivo: até4probes por triângulo lateral (28224neste tubo),
   normalização/frame em cada probe;48bisseções por origem bind. Coeficientes
   e envelopes preparados antes do loop. Sem nearest ou ordenação espacial.
5. Adequado: certificado analítico por envoltórias de4controles/até16estações;
   recusa do volume antes da amostragem. Mesmos4096v/8192t/63segmentos e budgets
   compostos. Cilindro3922v/7840t, sem ampliar teto.
6. Não se aplica: sem shader/Vertex/uniform/pipeline/readback por frame novos;
   capturas PNG só por comando explícito.
7. Adequado: dispatcher/staging/revisões existentes, sem thread/lock/dependência.
8. Adequado: sem cache persistente novo; edição invalida o documento como antes.
   OptionNone sai antes do novo mapeamento/probes, mantendo resposta antiga.
9. Adequado: testes independentes de círculo/plano/inversa3D, fechado/manifold,
   recusa conservadora/resolução/identidade/versões, fonte/escultura/material/
   rig rotacionado/pesos/clipes e ciclos. MCP cobre3584v/64cortes,
   tick60 explícito após câmera, PNG vivo/replay, undo/save-load e legado.
10. Mantido com motivo: custo síncrono de autoria, critério conservador que pode
    recusar curvas válidas, frame por referência fixa em vez de transporte geral,
    erro superficial amostrado sem limite Hausdorff global, sem loops. Nenhuma
    promessa de FPS ou desempenho em outra plataforma.
## EAF-94 — resposta dielétrica direta

Contrato/tradeoffs e10respostas antes/depois em
[dielectric-response.md](dielectric-response.md). Reflectance opcional pequeno
na igualdade/cache existente; GPU reutiliza dois escalares não usados na tabela
19312B, Vertex52B mantido. Globals160→176B pela câmera. GGX/Smith/Schlick frontal
por fragmento habilitado, sem alocação/textura nova; visibilidade compartilhada
com difuso. Nenhum tick/budget/lock/dependência novo. Ordem aritmética legada
conservada e comparada contra captura do build anterior. Mantidos UNORM, lobo
isotrópico uniforme e iluminação legada; aliasing especular exposto no controle
de alto brilho. Sem ganho de FPS presumido. Integração com versões funcionais
concorrentes requer schema/numeração unificados, conforme o contrato.

## EAF-80/EAF-85 — integração da junção com reflectance

Checklist anterior à importação e revisão proporcional em
[tree-geometry-material-integration.md](tree-geometry-material-integration.md).
Reuso do kernel puro de transição sem avaliar o campo por tick; preparação e
desenho existentes processam a geometria maior. Mesmos layouts e budgets. Publicação só da região explicitamente reautorada,
com material/rest-domain nas regiões comuns conservados. Finalizador contabiliza
componentes reais em vez de assumir o fork antigo; prefixo de replay separado
confere o tick, sem mudar a engine para acomodar o ensaio. Orquestração em roots
isolados e um build corrente. Resultados próprios e custos finais em validation.md.

## EAF-85/EAF-80 — cortes e pontes tangentes

Checklist anterior/tradeoffs em [mesh-bridges.md](mesh-bridges.md). Revisão final:
1. Operações de autoria/query, nenhum trabalho por tick.2. Staging mantém clones
limitados do asset; contagens por asset/documento/desenho antes de gerar ponte.
3. f64 temporário e f32 final, índices/revisões existentes; sem dependência/unsafe.
4. Normais num passe por triângulos; mapas ordenados para ciclos/arestas, sem busca
nearest por vértice.5. Mesmos limites4096v/8192t por asset e65536t desenhados.
6. Vertex52B/tabela19312B/Globals176B e shader/cache preservados; custo
geométrico adicional medido no relatório da árvore, sem FPS presumido.7. Sem lock/worker
novo.8. Arc de geometria/rig apenas no staging, clipes/material compartilhados;
referências obsoletas/lote validadas no dispatcher.9. Rust/MCP/árvore verificam
pureza, canais/rig, erros, animação, undo/persistência/replay/prefixo/GPU.
10. Mantidos por escopo: inspeção topológica/validação repetida por operação,
serialização do preview completo e amostragem finita. Não alegar suavidade
matemática dos triângulos, self-intersection global ou ganho de FPS.

Resultado geométrico e orçamento final em validation.md. O ensaio contabiliza
o custo real dos demais componentes e verifica recusa atômica das densidades
que ultrapassem65536, sem elevar tetos.


## Adoção de trajetória na árvore

Contrato, alternativas, limites e revisão estática em
[tree-trajectory-integration.md](tree-trajectory-integration.md). Adoção explícita
conserva fonte/referência históricas; Scene65/replay69 locais. Draw131072, demais
tetos mantidos e consultáveis via inspect_meshes.limits. Junção regenerável
permanece uma etapa distinta; protótipos reprovados não fazem parte da API.

## EAF-85 — base de junção por volume implícito

Checklist anterior e tradeoffs em [implicit-junctions.md](implicit-junctions.md).
Revisão final:1.Autoria/query pura, nenhum tick novo.2.Preflight65536pontos de
campo/16384v32768t intermediários antes de crescer buffers; preflight final
4096v8192t antes de copiar o asset e adicionar pesos.3.f64 temporário/f32 final,
sem dependência/unsafe/ABI.4.Arrays na stack para classificação por tetra;
cache por aresta e adjacência por vértice limitados à autoria, sem source-nearest.
5.Tetos globais mantidos; publicação separada revalida storage/draw existentes.
6.Layout/renderer/cache iguais;1195v2480t adicionais no fixture,7440v×52B de
expansão extra no caminho existente.7.Sem worker/lock novo.8.Arc de metadados/
clipes preservado; staging/cache usuais na publicação, query não grava replay.
9.Rust/MCP medem pureza/erros/orientação/canais/rig/poses/socket/undo/persistência/
replay/GPU.10.Custos mantidos: segmentos/células/projeção, mapas ordenados,
três passes de normais de borda e DP de fases com buffers finitos na autoria;
preview serializa asset completo. Sem alegação de FPS/LOD/injetividade global.

No repro,9044pontos da grade usam217056B de posiçõesf64 e72352B de escalares,
além da extração/cache/adjacências. Arrays por tetra corrigem alocação evitável
observada na revisão. Estes são custos geométricos, não medição de GPU/FPS.

## EAF-85 — refinamento local e domínio, antes/depois

Checklist anterior em implicit-junctions.md.1.Somente query/autoria, sem tick.
2.CSR de adjacência (2E índices/offsetsV+1) compartilhado entre geometria e
domínio; buffers de atualizações reusados nos passos. Nenhuma alocação por
vértice/iteração.3.f64 temporário/f32 validado, prefixo e guarda exatos.
4.Até8passos locais e64CG somente de material; normais/ângulos completos antes
e depois, sem reconstruir topologia por passo. Erro de campo final reusa as
guias extraídas.5.Mesmos tetos, mesma contagem geométrica; envelope finito,
orientação local/guarda90 e não piora máxima/média antes de retornar.
6.Layout/renderer/upload iguais; nenhum custo geométrico novo por frame.
7.Sem dependência/thread/lock.8.Opções default0 conservam saída anterior;
publicação/undo/cache existentes.9.Rust/MCP comparam0/1/2/4, recusam8que piora,
domínio/prefixo/guardas/rig/socket/erro/persistência/replay e neutro.
10.Custos mantidos: cópias temporárias limitadas de pontos/normais e até64
produtos esparsos no domínio, serialização integral do preview. Não é solução
global de autointerseção nem garantia de conservação de volume/FPS. CG não é
usado para embedding geométrico; resíduo real pós-f32 é observável.

## EAF-85 — kernel geral de filtro de superfície

Checklist anterior/alternativas e contrato em mesh-surface-filter.md.
1.Só autoria pura, sem tick/frame.2.CSR de vizinhos e faces, linksu32 por face,
buffers de médias/normais/cossenos/filas preparados uma vez e reusados; caso
totalmente fixo compartilha Arc sem cópia geométrica.3.Índices validados,
f64 temporário/f32 final e fonte imutável.4.Custo O(V+E+T+passos×região incidente),
mais ordenação da seleção; estrelas de vértices e componentes percorridos com
filas reutilizadas, não busca global por vértice.5.Tetos atuais herdados do
Asset, até32passos, envelope total/qualidade/orientação validados antes do
retorno.6.Sem GPU/layout/readback novo; função ainda sem dispatcher.
7.Sem lock/worker/dependência/unsafe.8.Clone da geometria somente após sucesso,
rig/clipes compartilhados; nenhum cache novo.9.Testes focados cobrem máscaras,
fixos/rig/poses/socket/canais, duas ilhas, qualidade, não-manifold, erros e
determinismo.10.Mantidos por escopo: Topology::inspect repete validação básica,
volumes percorrem componentes na autoria e Laplaciano pode encolher. Sem ganho
de FPS, volume preservado ou prova global de embedding. MCP/undo/persistência
da nova operação são gates de integração posteriores, não comprovados aqui.


## Composição paramétrica com contexto — local66

Receita de3portas reavaliada de fontes imutáveis, sem usar saída como origem.
IDs de seção/contornos reais, raiz rígida explícita e aparência compatível;
metadados atualizam fontes e geometria direta recusa rebase. Preview de lote
compartilha staging do editor. Apenas asset8192v16384t; totais e draw mantidos.
Escolhas/revisão10pontos, custos e limitações: [plant-assemblies.md](plant-assemblies.md).
Scene66/replay70 locais; dados canônicos cortados ficam na fonte, sem falsa
transferência de escultura para miolo novo. Artgate e demais galhos continuam.

## E02 por inventário consolidado — autoria de conteúdo

Revisão anterior/posterior dos10pontos em [e02-rebuild.md](e02-rebuild.md).
Nenhum caminho Rust, frame/tick, buffer GPU, limite ou dependência do kernel
alterado. Preflight no cliente, curvas até0.9px, weld com remapeamento semântico
e fonte compartilhada de maciço×12:16assets/30instâncias/16255v/18450t.
Redução de geometria preserva tarefas com estado parcial, não certifica detalhe.
I/O de autoria/captura explícitos e root corrente estável; sem claim de FPS.

## Revisão E02 e visualizador orbital de inspeção

Escopo, custos e10pontos em [e02-differences.md](e02-differences.md). Sem mudança
no renderer/kernel. JSON→buffersWebGL uma vez; instâncias expandidas no cliente
17953v/19188t, render dirty/resize/giro, DPR≤2 e controles limitados. Sem CDN,
rede/worker/edição da cena; rAF leve mantido parado. Testes matemáticos e dados
não certificam WebGL/presentação. Sem claim de FPS ou nova câmera na engine.

## EAF-85 — kernel geral de filtro de superfície

Checklist anterior/alternativas e contrato em mesh-surface-filter.md.
1.Só autoria pura, sem tick/frame.2.CSR de vizinhos e faces, linksu32 por face,
buffers de médias/normais/cossenos/filas preparados uma vez e reusados; caso
totalmente fixo compartilha Arc sem cópia geométrica.3.Índices validados,
f64 temporário/f32 final e fonte imutável.4.Custo O(V+E+T+passos×região incidente),
mais ordenação da seleção; estrelas de vértices e componentes percorridos com
filas reutilizadas, não busca global por vértice.5.Tetos atuais herdados do
Asset, até32passos, envelope total/qualidade/orientação validados antes do
retorno.6.Sem GPU/layout/readback novo; função ainda sem dispatcher.
7.Sem lock/worker/dependência/unsafe.8.Clone da geometria somente após sucesso,
rig/clipes compartilhados; nenhum cache novo.9.Testes focados cobrem máscaras,
fixos/rig/poses/socket/canais, duas ilhas, qualidade, não-manifold, erros e
determinismo.10.Mantidos por escopo: Topology::inspect repete validação básica,
volumes percorrem componentes na autoria e Laplaciano pode encolher. Sem ganho
de FPS, volume preservado ou prova global de embedding. MCP/undo/persistência
da nova operação são gates de integração posteriores, não comprovados aqui.
## Continuidade e acabamento por origem — revisão do diff

1. **Adequado:** loft/fairing e reconstrução de assembly somente na autoria e
   validação; nada novo por tick. Skinning/renderização continuam existentes.
2. **Corrigido:** loft prealoca posições/domínio/pesos finais; não clona a lista
   de triângulos para descartá-la. Interpolação linear não prepara derivadas
   cúbicas. Mantidas cópias de saída e fontes compartilhadas imutáveis.
3. **Adequado:** u32, cálculos f64/publicação f32; CSR no filtro, proveniência de
   dois endpoints por ponto novo, sem busca espacial ou unsafe/dependência nova.
4. **Corrigido:** faces indexadas uma vez, normais de células preparadas fora dos
   loops de subdivisão. **Mantido com motivo:** Scene.validate reavalia autoria
   para provar correspondência com suas fontes, inclusive os filtros finitos.
5. **Adequado:** até128 patches,128anéis,32lados,16spans; faces disjuntas e tetos
   de saída antes da expansão. Filtro até32passos, seleções limitadas pelo asset.
6. **Mantido com motivo:** armazenamento passa a16k/32k por asset e32k/64k no
   documento; draw131072/layout/grade da união mantidos. Arrays de pose/normais
   por asset crescem com esse teto, custo conhecido; sem ganho de FPS alegado.
7. **Adequado:** mesmos dispatcher, locks e transação. Sem workers novos.
8. **Adequado:** changed_assets/component_reference cobrem SmoothRegion; preview
   puro, batch obsoleto/undo/replay/capturas exercitam invalidação existente.
9. **Adequado:** fontes/canais e pesos retidos, fonte privada também versionada;
   filtros recusam dobras, inversões e piora de pico/média. Capturas neutras e
   com material não equivalem a aceite artístico/teclado/UI.
10. **Mantido com motivo:** Laplaciano pode encolher; volume e deslocamento
    reportados. Preparação/revalidação síncrona é limitada e custa autoria.
    Fontes e primeiras guardas são fixas no filtro iso; filtro não une ilhas.

## Enxertos locais no contexto — revisão do incremento68

Prévia e alternativas registradas em organic-continuity.md antes do código.

1. **Adequado:** avaliação de autoria em `plants/assembly.rs`/`graft.rs`; sem novo trabalho por tick/frame.
2. **Adequado:** capacidades de concatenação reservadas, fontes Arc imutáveis; cópias de geometria nos cortes/união mantidas por atomicidade. Mapas de índices compactos por remoção.
3. **Adequado:** índices u32, `Option<u32>` para remoção explícita; translação f32 declarada, campo existente f64. Sem unsafe ou skin inferido.
4. **Corrigido:** catálogo canônico completo valida fontes/IDs/totais antes da reavaliação em `Document::validate`; lookup de triângulos orientados uma vez após os enxertos. Validações topológicas repetidas por portas/cortes permanecem limitadas e defensivas.
5. **Adequado:** até8enxertos; tetos16assets/32768v/65536t canônicos,16384v/32768t avaliados porasset e grade65536 mantidos. Não aumentar para generalizar diferenças arbitrárias de raios.
6. **Não se aplica:** kernel sem GPU/I/O; capturas/persistência somente por comandos existentes.
7. **Adequado:** mesmo staging/dispatcher e transação, sem lock/thread novo.
8. **Adequado:** novo resultado passa pelos assets/revisões existentes; preview puro/undo/save/replay exercitam invalidação, sem cache novo.
9. **Adequado:** fontes integrais/rig/clipes preservados, formatos68/72 e gate explícito; máscaras, overlap, referências ausentes, aparência/frame/grid e budgets testados. Nenhum contrato das guardas alterado.
10. **Mantido com motivo:** grade uniforme, cópias limitadas e reconstrução inteira do contexto por edição. Até a mudança de trajetória de outro braço reavalia os enxertos estáticos; cache incremental exigirá dependências/versionamento próprios. Não houve medição de FPS/tempo nem promessa de escala.

## Colar por anéis de arestas — revisão do incremento69

Decisão/checklist prévios e recusas que delimitaram o recorte em
organic-continuity.md. Código em mesh_surface::ring_region e plants/graft.rs.

1. **Adequado:** seleção e acabamento somente na avaliação de autoria; sem novo caminho por tick/frame.
2. **Adequado:** CSR, fila com capacidade deV e distâncias preparados uma vez; sem fila/coleção nova por camada. Fontes Arc não são rebased. Seleção/âncoras são temporárias por edição.
3. **Adequado:** índicesu32,distânciasu8(limitadas16),Option para compactação; mesmas posiçõesf32/cálculosf64 do filtro.
4. **Adequado:** BFSmulti-source visita cada ponto no máximo uma vez,arestas únicas; não funde âncoras de BFSs separados. Pins de portas não selecionadas resolvidos pelos loops reais. **Corrigido:** a checagem final das interfaces usa o mapa diretamente, sem alocar um Vec por vértice.
5. **Adequado:** alcance1..16,3portas no máximo; custoO(V+E) de seleção e filtro já limitado. Tetos de fontes/saída/draw/grade mantidos.
6. **Não se aplica:** sem I/O/GPU novos; geometria publicada/captura sob comandos existentes.
7. **Adequado:** mesmos staging/dispatcher/rollback,sem threads/locks novos.
8. **Adequado:** máscara derivada de portas em cada avaliação,sem cache de IDs efêmeros. Ciclo do loft3→4→3/undo/load/replay verifica regeneração.
9. **Adequado:** guardas do filtro não mudaram; parâmetros excessivos continuam recusados. Report por etapa/origem,canônico imutável,mainports comparadas após todos filtros. Legado sem campo omitido e gate69/replay73.
10. **Mantido com motivo:** Topology/CSR adicional para selecionar a região antes de a filtragem validar a malha; reconstrução inteira em edição,shrink e dependência da tesselação. Sem volume conservado,C1/C2 ou ganho deFPS alegados.

## Integração das consultas de domínio na frente69 — prévia

Incremento somente de consultas, sobre e1fef8f.1.executa sob comando,sem tick;
2.Geometry emprestada,seleção/top32limitados;3.f64/índices/finito validados;
4.um passe e seleção ordenada uma vez;5.mesmos tetos de geometria/transporte;
6.nenhuma GPU/I/O novo;7.mesmo dispatcher/revisão,sem thread/lock novo;
8.sem cache/formato persistido;9.preserva Scene69/replay73,fonte/cena exatas e
pureza/erros verificados no MCP;10.validação integral e topK mantidos sem FPS.
A correção material é uma etapa posterior: esta integração somente mede.

## Referenciais e finisher materiais — revisão70

Decisão e checklist prévios em material-continuity-study.md.
1.Somente avaliação/query de autoria, nenhum tick/frame novo.
2.Afim reserva saída O(V) e usa Arc::make_mut somente na geometria alterada;
   rig/clipes/fontes/opções compartilham Arc. Plano consome os pares já produzidos pelo fit.
3.Cálculo afim/solver em f64 e publicação f32 finita dentro dos limites existentes.
4.Um passe afim e um finisher por estágio; material_relaxation_steps legado zerado
   somente no caminho novo, evitando duas difusões materiais.
5.Até8enxertos/tetos existentes, CG1..64, tolerância/envelope/cap explícitos.
6.Nenhum shader/upload/readback novo neste incremento; capturas são prova explícita.
7.Sem dependência/thread/lock novo, dispatcher e staging existentes.
8.Proveniência temporária exata no Fitted, omitida do JSON; campos opcionais entram
   na igualdade da receita e na regeneração, versões70/74 com rejeição legada.
9.Testes de fonte/rig/posições exatos, afim geral, erros atômicos, regeneração,
   versão/undo/replay e comparação neutra de4vistas; resultados em validation.md.
10.Mantidos: clone de Geometry e relatórios completos na autoria síncrona, CSR/CG
   por estágio e plano limitado mesmo na união sem finisher. Sem C1 físico/FPS
   alegados. ProjeçãoXY pode comprimir domínio3D regular: gate visual separado.

## Projeção triplanar integrada — revisão do diff70

Contrato/alternativas e checklist anteriores em triplanar-details.md.
1.Frame persistente calcula pesos só para novas identidades Geometry; shader
   avalia até3campos por fragmento opt-in, sem novo tick de gameplay.
2.Cache guarda12B por vértice fonte, no máximo384KiB no teto global32768v,
   além de Vec/Arc. History/source permanecem Arc; cache atual podado por ptr_eq.
3.Pesos unsigned f64→f32 finitos, Vertex64B explícito, IDs/cores sem reuso oculto.
4.Um passeV+T por nova Geometry, sem repetir por instância/pose; prepare percorre
   até16assets. Snapshot Engine::render_frame recria Frame, portanto captura MCP
   separada reconstrói pesos; custo mantido, não é cache global.
5.Tetos preservados e sharpness1..8. Fallback uniforme finito em material rank0/1
   não restaura informação perdida nem prova regularidade ou direção de fibra.
6.+4.5MiB de payload ativo no teto393216v desenhados; até+6MiB por buffer com
   capacidade524288v. Memória total depende das cópias/cache/driver. Três footprints
   novos são calculados fora de divergência também no caminho planar; até96testes
   de traço no modo habilitado. Tabela/Globals, shadow pass e readback mantidos.
7.Sem thread/lock/dependência nova.
8.Prepare antes de early-return poda geometrias novas com conteúdo igual; COW
   gera nova identidade, material toggle usa pesos já preparados.
9.CPU/cache/layout/schema, publicação70, arquivos69 recusados inclusive fontes
   ocultas, GPU/cor/bump/pose/rank0/1/toggle/resize, replay/prefixo e legado em
   pixels verificados; evidência em validation.md.
10.Mantidos gradiente de bump com pesos congelados, mistura de fases/direções e
   preparo por snapshot MCP. Sem FPS/realismo ou teclado/aceite humano alegados.

## EAF-72 — assets representativos no laboratório gráfico

Checklist prévio/final, alternativas e custos mantidos em [representative-assets-probe.md](representative-assets-probe.md). Modelos Blender exportados com índices/material explícito; bake/referência GGX compartilhados, limites e assinatura de índices/shader. Captura/I/O ficam fora do timing;9 blocos×32 desenhos e controle duplicado expõem variação.342 poses,598 testes Rust, Clippy/fmt e render/replay/selection (13 grupos MCP) aprovados. Descoberta do runner separa verificadores offline de PNGs dos cenários MCP. Nenhuma alteração do runtime/Scene/Command/RenderFrame. Atlas ainda custa12–106× a malha; ganho do passe próximo vem com perda visual e não demonstrou melhoria do frame completo. Inspeção nativa do relatório bloqueada pela política file://; harness e imagens não substituem aceite humano.

## EAF-73 — seleção na resolução física

Checklist antes/depois, derivação e evidências em [physical-adaptive-probe.md](physical-adaptive-probe.md). Bound de maior aresta/caixa preparado uma vez; oito cantos por seleção, limiares0,75/1px com histerese e resize. Recorte512² preserva projeção física3456×2234/1728×1117; defaults legados intactos. Diagnóstico de IDs visíveis amostrados fica fora do seletor/timing.252 poses e casos JS,602 testes Rust, fmt/clippy,15 controles legados e13 grupos MCP aprovados. Perto, adaptativo idêntico à geometria; baú evidencia que subpixel não garante fidelidade. Sem ganho estável de GPU/FPS demonstrado nem mudança de runtime/Scene/MCP. Memória e apresentação nativa/aceite humano mantidos como limites explícitos.

## EAF-74 — regiões em cena conjunta

Checklist prévio/final e evidências em [regional-scene-probe.md](regional-scene-probe.md).479 regiões por posição/material, cache local de cor/normal/profundidade em seis vistas e mips, compartilhado por96instâncias; representação explicitamente diferente do atlas de radiância anterior. Culling comum por objeto, seletor por região sem triângulos/JSON no caminho, faixas coalescidas e proxies instanciados. Depth invertido reduziu dependência de ordem distante; materiais brilhantes preservam geometria após detectar picos por média de normais.52frames completos físicos, mistura visível4–19objetos,611 testes Rust, Clippy/fmt e13grupos MCP headless aprovados. Redução37–74% de triângulos e mediana de custoGPU18–31% menor; ciclo serial pareado16–25% mais caro, com~0,33ms de seleçãoCPU. Não declarar ganho de FPS. Memória de cache29,94MiB compartilhada, targets/instrumentação à parte; apresentação nativa/aceite humano pendentes. Sem alteração de runtime/Scene/MCP ou da demo funcional.


## EAF-75 — custo total e limite de visibilidade

Checklist antes/depois em [regional-efficiency-study.md](regional-efficiency-study.md). Bound analítico conservador por asset/instância com fallback nos limiares, agrupamento por faixa de índices e baseline completa igualmente agrupada.52poses mantiveram RGB/IDs exatamente iguais;613 testes Rust, Clippy/fmt,13grupos MCP e176PNGs íntegros. CPU de seleção23–28% menor, draws329–819→76–201; vazão em blocos de8frames13–20% mais barata que geometria agrupada, sujeita à variação dos controles, sem alegar FPS. Diagnóstico ideal eliminou55–67% dos triângulos completos sem mudar pixels, mas a descoberta custou20/7,4ms: não é culling utilizável. Máscara antiga falha em desoclusões. Build release único movido após ENOSPC para `/Volumes/Zero/build-cache/ignis-graphics/release`, link em target/release; nenhum relatório anterior ou build de outra tarefa removido. Engine funcional e contratos intactos; apresentação nativa/aceite humano pendentes.


## EAF-82 — visibilidade GPU com geometria completa

Checklist prévio/final e números em [gpu-visibility-study.md](gpu-visibility-study.md). Hierarquia de479regiões+três raízes preparada antes do percurso; compactação GPU/indirect sem readback de decisão. Hi-Z do frame corrente comparado com prova convexa para caixas opacas conhecidas; seleção de objetos e regiões separadas.156poses+16extremas,860comparações de RGB float/IDs exatas,780PNGs e624casos JS;617testes Rust, fmt/clippy e13grupos MCP headless aprovados. Em cenas com paredes, prova por objeto reduziu custo de vazão em41–43%/49–55% nas duas resoluções, com79–82% menos triângulos; cena aberta não melhora e ciclo serial na menor resolução fica1–3% mais caro. Hi-Z não apresentou ganho uniforme. Não é FPS de jogo, oclusão genérica, suporte validado fora de macOS/Metal ou aceite visual nativo. Buffers estáticos6,78MiB compartilhados, pirâmide5,33/1,33MiB e targets/readback à parte; mantém um build corrente na localização externa existente. Engine funcional/contratos/demo preservados; commit local sem publicação.


## EAF-84 — curva de custo por densidade fonte

Protocolo e revisão antes/depois em [triangle-scaling-study.md](triangle-scaling-study.md). Duas famílias procedurais,32.768/65.536/327.680/3.276.800triângulos, oito níveis preparados e seleção por erro projetado; normal fonte em textura mantém aparência no recorte parametrizado.144poses/432comparações,576PNGs,622testes Rust, Clippy/fmt,13grupos MCP e432casos JS aprovados quanto à execução/integridade; critérios visuais falham em casos publicados. Confirmação com quatro fontes intercaladas e blocos de128frames corrigiu otimismo dos blocos curtos: relevo distante a112 mantém10×→100× na mesma faixa (+3,9%/−2,6%), mas1×→100× cresce40%/26% em vazão e54%/47% em GPU. Perto cresce9–15×; não declarar custo geral constante. Cache de normais passa141/144poses, três falhas de cobertura por um pixel na menor resolução/far extremo; memória1,27→125,15MiB com todos os níveis/normais. Sem integração funcional/contratos ou aceite visual nativo, mesmo build corrente externo e commit local sem publicação.

## EAF-86 — regiões e projeção, revisão antes/depois

1. Frequência: bake mede intervalos de erro por triângulo; frame consulta no máximo64regiões e seus níveis. `regional_scaling_render::selected` calcula o fator projetivo uma vez por caixa, sem percorrer vértices fonte.
2. Alocações/cópias: seleção em array fixo64 na stack; buffers/targets persistentes. Upload, normais, ordenação dos índices e fans preparados fora da medição. Alocações internas do backend não são declaradas ausentes.
3. Tipos/acesso: índicesu32 e posição/normalf32; intervalos assinados evitam duplicar indiscriminadamente o erro máximo da região. Frame não usa strings para selecionar.
4. Repetição: índices agrupados por nível permitem coalescer regiões contíguas; um fator de projeção por região. Mantidos três caminhos de controle para separar efeito da projeção e da partição.
5. Escala/limites:64regiões fixas, quatro fontes32768…3276800triângulos; fronteira fonte custa proporcionalmente à densidade. Não há custo assintótico constante ou solução de geometria arbitrária prometidos.
6. GPU/I/O: um passe, mesma textura de normal fonte e material. Até64draws coalescidos; total CPU+GPU inclui seleção/encodificação/submissão. Readback/PNG só na verificação, fora da medição.
7. Sincronização: benchmark instrumenta timestamps/espera deliberadamente; sem nova thread, trava ou dependência no sistema. Quatro densidades intercaladas, controles duplicados e128frames por bloco.
8. Invalidação: não há cache de seleção entre frames; resolução/câmera entram a cada consulta. Bake substitui arquivos correntes locais; exemplo requer rebake explícito ao alterar fonte/topologia, não constitui formato persistente público.
9. Correção: testes de partição/área/arestas, erro dos fans e projeção; mesmas metas RGB/cobertura e ida/volta. Capturas adicionais exercitam níveis mistos entre posições da curva. Evidências e limites em `regional-scaling-study.md`.
10. Custos mantidos: todos os níveis globais e regionais residentes para comparação, bordas fonte e texturaRGBA32F sem mip/compressão. Material/luz fixos e heightfield vertical isolado; sem alterar Scene/Command/RenderFrame/MCP ou a demo funcional.


## EAF-88 — seleção híbrida, revisão antes/depois

1. Frequência: `hybrid_choice` consulta metadados globais e até64regiões por frame; limites inferiores preparados podem encerrar antes. A decisão não consulta o relatório nem pixels/tempos de referência.
2. Alocações/cópias: scratch64 na stack; zero nova alocação explícita no seletor, sem percorrer/copiar malha. Preparação e uploads permanecem fora do timing; não alegar ausência de alocação no backend.
3. Tipos/localidade: contagensu32 e custou64 evitam overflow no domínio atual; geometria/normalf32 preservadas. Vértices fonte compartilhados entre global preservado/regional; índices extras ainda residentes.
4. Repetição: poda demonstrada equivalente à escolha exaustiva. Se nível global é a fonte, normal de vértice dispensa a textura redundante; controles legados preservados para separar os efeitos.
5. Escala:64regiões fixas,1×/2×/10×/100×; o modelo triângulos+1024×draws não representa perfeitamente todo custoGPU/CPU. Reportar regret medido, não prometer mínimo real nem custo constante.
6. GPU/I/O: preservação do contorno fonte evita herdar a falha de cobertura do global anterior. Até64draws, coalescidos; seleção/codificação/submissão entram no total. Captura/PNG/readback fora dos blocos medidos.
7. Sincronização:40slots duplicados,12rodadas,128frames/bloco, quatro densidades alternadas. Sem thread/lock/dependência nova; espera instrumentada explícita e compilação/regressão desta tarefa fora do benchmark.
8. Invalidação: câmera/resolução reavaliadas; sem cache temporal de vencedor. Novos metadados são de artefato experimental, com rebake explícito e validação de presença; não são formato público de cena.
9. Correção: contorno/área/erro com uma e64regiões e reuso de fonte; poda versus exaustivo; captura do híbrido idêntica ao candidato escolhido, reverso e controles legados comparados. Evidência completa em `hybrid-scaling-study.md`.
10. Custos mantidos: novos índices globais, contorno fonte e campo de normais crescem com densidade; todos os caminhos residentes no comparador. Material/luz fixos, heightfield vertical, referência de amostra única. Sem mudar runtime/Scene/Command/RenderFrame/MCP, sem controlar a demo, aceite visual nativo separado.


## EAF-88 — previsão calibrada, revisão antes/depois

1. Frequência: cálculo por frame em `cost_model::choose`, caixa projetada por8cantos e consulta de no máximo64regiões; sem malha/PNG/medição no seletor. Calibração executada explicitamente fora do frame.
2. Alocações/cópias: modelo6f64 e scratch64 na stack; nenhuma nova alocação explícita por seleção. Texturas/meshes reutilizados do bake anterior. Backend não declarado livre de alocação.
3. Tipos/localidade: característicasf64 de contagensu32/projeçãof32; coeficientes finitos e monotônicos validados na carga. Modelo ocupa48bytes; JSON de calibração é artefato de pesquisa, não schema da engine.
4. Trabalho repetido: poda usa limite inferior regional e contagens parciais; mesma decisão do exaustivo. Computar área e avaliar o modelo adiciona custoCPU, incluído no benchmark; não assumir ganho pelo modelo ser pequeno.
5. Escala: mesmos1×/2×/10×/100×,64regiões; modelo por hardware/contexto de render, sem prova de generalização para assets/materiais/oclusão diferentes. Proxy de perímetro não mede triângulos finos individualmente.
6. GPU/I/O: shader/geometria/gates preservados; GPU com32passes enfileirados e resolução após fence; vazão128frames sem instrumentaçãoGPU. Readback/PNG só fora do tempo medido e do caminho de decisão.
7. Sincronização:8rodadas, controles duplicados e quatro densidades alternadas. Compilação/capturas fora da cronometragem. Sem thread/trava/dependência nova no runtime; custo de espera de medição separado da seleção.
8. Invalidação: modelo lido uma vez antes do experimento, registrado por coorte e verificado contra capturas; calibração18/40/80/160 disjunta de14/28/56/112/224. Mudança de hardware/contexto exige nova calibração, não seleção por cache de poses.
9. Correção: monotonicidade/coeficientes inválidos, poda versus exaustivo em testes e capturas; imagens anteriores byte-idênticas e preservação do candidato escolhido. Evidências em `calibrated-cost-study.md`.
10. Custos mantidos: todos os controles continuam residentes; fonte e normal crescem com densidade, sem streaming. Capturas não provam vídeo contínuo/aceite humano. Scene/Command/RenderFrame/MCP e demo funcional preservados.

## EAF-88 — ablação do custo de material e profundidade

Checklist prévio/final e resultados em [scaling-ablation-study.md](scaling-ablation-study.md). Intervenções de shader/ordem mantêm geometria; interpolação sem bifurcação conserva pixels exatos. Intervalo fonte preparado no upload evita recuperação redundante da normal em regiões completas. PasseRGBA32F rejeitado por custo; alternativa reutiliza depth32 e normais do heightfield preparadas, sem alvo extra necessário à variante. Mesmo seletor, novos pipelines opt-in no laboratório; custo de ambos os passes/CPU incluído. Tradeoff: ganho total9,5–15% repetido em relevo100× próximo nativo, mas perdas estáveis longe/casos leves,97/112pontos>3% mais caros na rodada final; não substituir padrão nem declarar platô próximo/FPS. Fonte completa2,138ms versus anterior0,882/new0,750: ganho de geometria anterior não atribuído ao passe novo.228poses de profundidade e1684comparações totais aprovadas; cobertura zero,1288PNGs íntegros/retorno,controles antigos preservados.631testesRust,fmt/clippy,render/replay/selection headless,832+576casosJS; aceitehumano/browser nativo separados. Fonte/bake/payload e buffer de controle ainda residentes no comparador; sem claim de RSS/VRAM menor. Enginefuncional/Scene/Command/RenderFrame/MCP/demo preservados.

## EAF-88 — escolha automática entre forward e profundidade

Checklist prévio e protocolo em `automatic-shading-study.md`, definidos antes da coleta; ampliação geométrica do domínio de calibração registrada antes do teste completo.

| Pergunta | Revisão do incremento |
|---|---|
| 1. Frequência | **Adequado:** modo95 em `regional_scaling_render::encode` escolhe geometria uma vez e então estima apenas o caminho de sombreamento; fit/I/O fora do frame. |
| 2. Alocações/cópias | **Adequado:** features `[f64;6]`, oito cantos e produtos escalares na stack, sem Vec/clone/JSON novo por decisão. Modelo carregado uma vez. |
| 3. Tipos/acesso | **Adequado:** geometria/imagem mantêm precisão anterior; f64 somente no estimadorCPU. Não usa IDs/família/densidade como chave. |
| 4. Trabalho repetido | **Corrigido:** reutiliza contagem/draws/native_source da seleção existente, sem segunda seleção ou scan de malha. Projeção8cantos por decisão é custo explícito. |
| 5. Escala/limites | **Adequado ao laboratório:** O(1) adicional, modelo21valoresf64+nome do adaptador; fora do domínio ou abaixo da margem conserva anterior. Não generaliza a cenas/materiais arbitrários. |
| 6. GPU/I/O | **Corrigido:** construtor automático dispensa alvoRGBA32F intermediário; fallback só um passe, candidato usa depth32 existente e mais um passe. Readback somente em benchmark/captura, nunca para decidir. |
| 7. Sincronização | **Adequado:** nenhuma thread/lock/espera nova no seletor; ambas as passagens incluídas no timing. Benchmark separado das capturas/testes. |
| 8. Invalidação | **Adequado ao experimento:** sem estado persistente de escolha, recalcula em câmera/resize; adapter validado, modelo/hash congelados. Regeneração exigida após shader/material/GPU/driver; não é cache automático de produção. |
| 9. Correção | **Validado:**316poses automáticas dentro dos gates,624controles PNG anteriores exatos, retorno exato, imagemfloat igual ao caminho escolhido;632testes Rust e912casosJS. Sem contrato runtime/NPC/Scene/Command/RenderFrame/MCP novo. |
| 10. Custos mantidos | **Explícito:** criação inicial ainda prepara e descarta pipelines diagnósticos reutilizados; bake/targets/readback anteriores permanecem. Seleção mediana0,0155µs; ganho pesado repetido, mas ruído e casos intermediários sem benefício claro impedem promessa de regressãozero ou platô próximo. Dados completos no estudo. |

## EAF-88 — platô próximo, organização de geometria

Pré-revisão e protocolo em `docs/near-plateau-study.md`; escopo exclusivo do exemplo de pesquisa.

1. Frequência: seleção geométrica e modelo anteriores uma vez/frame; organização/reindexação somente no preparo explícito.
2. Alocações/cópias: frame acrescenta escolha de buffer/pipeline, sem novas coleções. Preparo usa vetores de posições/índices, dois arrays de remap/geração e scratch de triângulos por intervalo; libera após salvar. Captura/bench são operações explícitas fora do runtime.
3. Layout: posiçãof32×3/stride12 em vez de V24 no depth; índicesu32 e offsets/counts conservados. Sem quantização de coordenadas/normais. Duplicação por intervalo reduz o benefício de memória:100× posições78,32MiB contra88,68MiB do V original, índices153,24MiB inalterados.
4. Repetição: Morton calculado no sort de preparo; custo aceito fora do frame. Pipeline de posições preparado uma vez no construtor diagnóstico. Nenhum sort/scan de vértices no encode.
5. Escala: complexidade do preparo por intervalo O(TlogT), frame ainda depende dos triângulos elegíveis; mudança não garante platô. Os dois controles extras somam463,12MiB para100× e são mantidos residentes na comparação; não chamar de redução deRSS/VRAM.
6. GPU/I/O: mesmos dois passes, targets, material e normais; apenas posições/índices/bindings mudam. Profiler usa64queries existentes; resolve/readback fora do tempo total. Imagens/JSON/bake no diretório gerado estável.
7. Sincronização: nenhuma thread/lock/dependência nova. Esperas GPU só no laboratório de medição/captura; ablações têm dependências diferentes, logo seus intervalos não são parcelas aditivas.
8. Validade: preparar derivados novamente após mudar o bake fonte. Upload valida limites/alinhamento; teste de packing conserva winding/posição/intervalos, capturas comparam controles anteriores e pixelsfloat. Não é formato de cache de produção com invalidação automática.
9. Correção: seleção0,15px e modelo congelados; gates e igualdadeRGBA32F conferidos separadamente. Evidência final em `docs/near-plateau-study.md`; runtime/render/replay/seleção têm regressão headless própria.
10. Custos mantidos: fonte completa, níveis e variantes residentes; sem streaming, nova representação geométrica ou integração Scene/Command/MCP. Melhorar organização pode reduzir constante, mas não retirar o trabalho que ainda cresce de1× a100×.

Complemento da investigação: diagnóstico de footprints projeta vértices/triângulos fonte somente no comando offline `--footprint-near-plateau`, em vetores temporários por asset/câmera. Sem uso no seletor/frame. Separa clipping/semiespaços e não confunde área com visibilidade/erro.64coortes verificadas; novo teste de packing e regressões totalizam633testes Rust, mais560casosJS e validação headless. Resultados completos, perdas e memória em `docs/near-plateau-study.md`.

## EAF-88 — subdivisão com superfície fixa e densidade física

Protocolo e resultados em `docs/pixel-density-study.md`; revisão prévia registrada antes da implementação/coleta.

1. Frequência: geração baricêntrica e cálculo do erro no preparo offline; frame testa oito cantos e um limite projetado. Sem scan dos milhões de triângulos no seletor.
2. Alocações/cópias: geração usa vetores reutilizados por bloco; scratch principal até 224 MiB de capacidade mais dados fonte. Decisão no frame sem nova coleção, readback ou geração de malha. Câmera/encoder/submit existentes incluídos no timing.
3. Layout: posições f32×3, índices u32, contagens u64; casts limitados por validação e chunks. Coordenadas intermediárias f64 para medir arredondamento; sem quantização do material/normais ou novo formato público.
4. Repetição: referencia a parental existente em vez de percorrer/tesselar a fonte; bounds consultados por câmera/resolução. Construção O(T) mantida e cronometrada fora do frame.
5. Escala: buffers individuais ≤128 MiB, chunks ≤8 milhões de triângulos. As quatro fontes densas realmente residentes custam 4,517 GiB no nativo, não memória constante. Fonte 100 tem 29 draws e 226.390.700 triângulos. Seleção mantém 327.680; cenário redundante por construção.
6. GPU/I/O: mesmos depth/material, normais/luz e targets. Novas fontes densas indexadas só no laboratório; nenhum shader de geração ou estimativa de tempo. Sem GB de derivados gravados no disco; JSON/PNG em caminhos correntes ignorados.
7. Sincronização: esperas de preparo/captura/benchmark explícitas, fora da decisão. GPU timestamp separado do total; ambas as passagens incluídas. Coleta concluída antes de iniciar testes/regressões. Nenhuma thread/lock/dependência nova.
8. Validade: fonte fixa e hashes de inputs antes/depois; certificado limitado à genealogia de construção. Ausência/invalidez/near rejeitam o resumo; fallback GPU forçado e conferido. Não é certificação de malha importada ou invalidação automática de cache de produção.
9. Correção: bordas uniformes e refinamento interior evitam T-junctions; três testes novos, 636 testes Rust e 13 grupos headless aprovados. 40 poses/80 gates, candidato exato RGBA32F e 120 PNGs íntegros/retorno exato; 80 casos JS/DPR. Resíduo temporal contínuo e aceite humano não comprovados.
10. Custos mantidos: preprocessing O(T), memória dos comparadores, leitura fonte, diagnóstico de captura e pipeline inicial reutilizado. Margem numérica não prova igualdade raster universal. Platô nativo 0,4247–0,4290 ms é evidência deste experimento com conteúdo constante, não promessa para qualquer complexidade de cena. Runtime e contratos preservados.

## EAF-88 — símbolo de superfície na pedra

Protocolo e pré-revisão em `docs/rock-symbol-study.md`; resultado negativo de fidelidade registrado sem relaxar gates.

1. Frequência: agrupamento espacial/orientação e ponderação por área somente no preparo. Frame usa buffers imutáveis; não reconstrói os81.920triângulos fonte.
2. Alocações/cópias: BTreeMap de grupos/listas de IDs no preparo; número de entradas limitado aos triângulos validados. Capturas/harness mantêm imagens e cópias da referência para resíduo temporal, fora dos timings. Frame não aloca coleção de símbolos; encoder/uniform existentes incluídos.
3. Layout: três vec4/f32 por símbolo48bytes, sem quantização; origem indexada40bytes/vértice e u32/índice. Material pedra explícito, funçãoWGSL compartilhada. Layout/finitude/área e índices testados.
4. Repetição: shader lê o símbolo por vértice do suporte e porfragmento; esse trabalho/overdraw é mantido e medido. Ponderado rasteriza duas vezes, normaliza em terceiro passe; não inferir ganho de triângulos/bytes.
5. Escala: grades16/32/64 dão1.985/6.428/19.143símbolos; fonte2,50MiB e maiorcandidato0,876MiB. Sem streaming/hierarquia adaptativa. Normal/cor média perdem detalhe; não existe certificado que autorize substituição automática emprodução.
6. GPU/I/O: recursos persistentes, cor/depth/readback comuns e acumulaçãoRGBA16F adicional58,90MiB no nativo. Target é porviewport, não porasset. Resolvefullscreen é custo evitável conhecido; não implementado recorte neste experimento porque a falha de aparência já impede promoção. Preparo/PNGs/readbacktimestamp fora do total.
7. Sincronização: nenhuma thread/lock/dependência nova. Capturas e referências concluídas antes de timings; regressõesGPU depois. Timestamp intermediário sem índice corrigido antes de coletar dados; intervaloGPU e vazão total distintos, sem somas de parcelas sobrepostas.
8. Validade: hashes de geometria/shader e payloads de símbolos conferidos na retomada; rebuild explícito após fonte. FonteBlender preservada. Discos/espessura são aproximações, não prova de cobertura ou visibilidade.
9. Correção:639testesRust,13gruposheadless,972casosJS aprovados;162poses/1.140PNGs íntegros.972gatesvisuais falharam e são resultados explícitos. Wrap/retorno controlado aprovado;34fechamentosPNG do combinado diferem por elevaçãof32, também na referência. Sem aceitehumano/browser/vídeocontínuo.
10. Custos mantidos: memória das três escalas/targets e infraestruturaGPU do laboratório, overdraw,48bytes/símbolo, múltiplos passes e normalizaçãofullscreen. Filtro maisfiel regrediu1,6–4× nos controles de tempo total; pouco ruído não foi presumido (duplicatasaté1,51×). Economia de payload não é economia deframe/RSS. Próxima representação deve resolver forma/orientação/visibilidade antes de otimizar os custos fixos.

## EAF-88 — pedra compilada em respostas binárias (27/10/2026)

1. Frequência: `compiled_rock::compile` calcula agrupamentos/BRDF uma vez; `Draw::encode` só projeta, seleciona e consulta. Processos separados, load sem fonte/JSON comprovado em diretório isolado.
2. Alocações: payloadGPU persistente, vetoresCPU liberados no upload; uniform256B por desenho. `Asset::load` tem cópias temporárias para validação/checksum fora do frame; captura/report e benchmark alocam deliberadamente como instrumentação.
3. Tipos: arquivoLE com header68B, diretório u32, código16+profundidade16 por texel e RGB10:10:10/cobertura2 por resposta. A entrada de resposta0 sem consumidor foi removida; blocos vazios omitidos, todos os códigos/tiles alcançáveis verificados. Não alegar formato universalmente ótimo.
4. Repetição: resposta compartilhada entre texels, sem normal/material fonte no shader de consulta. Filtrado até40 leituras lógicas por fragmento; direto até5, com menos filtro/projeção. CPU calcula bases/matriz/retângulo a cada desenho, sem cache por pose vencedora.
5. Limites:16/32 vistas e luzes,128²/256²,255/511 respostas; decoder limita64MiB e valida dimensões/tamanhos/offsets/domínio. Uma camada de profundidade e elevações fixas são restrições, não suporte geral.
6. GPU/I/O: um passe em retângulo, sem acumulador de viewport; mesmo colorRGBA32F de referência. Frame não abre arquivo, JSON ou readback. Timestamps separados do throughput; buffersGPU e memória de instrumento separados do payload.
7. Sincronização: nenhuma thread/lock/dependência nova; compilação/captura/benchmark/regressões serializados naGPU. Espera no fim dos blocos para throughput, não por pixel.
8. Invalidação: magic/versão/hash/layout no loader; runner confere fonte/shaders e checksums antes de comparar. CLIbuild explícito; câmera/luz só variam no domínio declarado. Código/artefatos correntes sem históricos de binários.
9. Correção:642 testes/clippy/fmt,13grupos headless;820PNG/648comparações,90retornos idênticos, decoder isolado igual em quatro caminhos, UI648casos. Smoke anterior7PNG idênticos após refatorar harness; critérios visuais não relaxados e todos648 reprovados.
10. Custos mantidos:0,753/5,669MiB payload, alto custo do filtro perto, quantização/camada única/aliasing. Direto perto temGPU semelhante ou pior que fonte; pequenos ganhos totais não são conclusivos sob ruído. Não há composição com profundidade de cena, streaming, suporte a elevação arbitrária, plateaus1×–100× nem entrega de renderer de produção. Evidências em docs/compiled-rock-study.md.

## EAF-88 — comparação incremental de símbolos de material

Protocolo/checklist antes e depois em `docs/combined-symbol-study.md`. Baseline135 (global/regional+automático forward/depth+packing) preservado; candidato só substitui resposta de material por tabelaRGB10A2 de64KiB, sem mudar geometria/depth/draws. BinárioIMS1 e políticaISP1 com hashes/layout validados; frame sem I/O/readback/coleção, máscara0 evita projeção/bind/consulta extra. Calibração24coortes rejeitou todas as classes, mantida congelada nos40casos de validação/repetição. Forçado não mostrou ganho robusto (razão mediana1,0004/1,0151); fallback mantém pixels exatos, não alegar economia por flutuação.252poses/756gates aprovados,764PNGs,84fechamentos/8baselines legados idênticos,645testesRust/Clippy/fmt/13gruposMCP headless e504estadosUI emDOMsimulado. Atlas geométricoIRK1 não integrado, falha prévia de forma permanece; este recorte não comprova redução adicional de triângulos, platô universal ou aceite humano. Engine/contratos/demo preservados; custos/limites e reprodução no estudo.

## EAF-88 — símbolo geométrico de alturas

Checklist prévio e dez respostas pós-implementação em `docs/geometric-field-study.md`. Fonte triangular preservada emIGF1/R32Float, certificado de interseção única, busca porpixel com profundidade, B135 intacto e fallback.864gates/288poses;196ativas. Memória adicional100×6,260MiB. Sem ganho incremental robusto: custo100×médio .3484→.5202ms (+49,3%), máscara de custo0. Corrigida agregaçãoGPU para janela do lote: passes sobrepostos não podem ser somados como throughput. Campos CPU_submit/latência/janela/parede instrumentada separados.648testesRust,Clippy/fmt,13gruposMCP eUI576estados passaram. Domínioheightfield; pedra multicamada, platô próximo e invisibilidade geral permanecem abertos.

### EAF-88 — hierarquia geométrica compilada (27/10/2026)

Checklist prévio e dez respostas posteriores em `docs/hierarchical-field-study.md`. IHF1: limites min/max por folhas 8×8 e mipmaps, 64 certificados direcionais; mesmas alturas/triângulos, até quatro estreitamentos por pixel e busca fina anterior. Sem I/O/readback na seleção, payload CPU liberado após upload. 100× acrescenta 699.048 B de limites GPU e 1.024 B de certificado CPU; geometria anterior preservada para fallback.

Busca fina −68–70% nos hits 100×; tempo total do símbolo cerca de −20% contra controle plano pareado em resolução nativa. Ainda pior que B perto: 0,95527 vs 0,60877 ms. Política congelada rejeitou todas as classes; opcional permanece B, sem alegar ganho incremental/FPS. 864/864 gates visuais, 268/288 poses elegíveis, profundidade/legado validados. Domínio heightfield, sem solução genérica de pedra ou mistura regional de caminhos. Relatório em `artifacts/appearance-probe/hierarchical-field/`.

### EAF-88 — busca compartilhada por blocos (27/10/2026)

Checklist prévio/posterior e protocolo em `docs/tiled-field-study.md`. Compute por bloco8×8 consulta IHF1; scratch de967.680B e parâmetros128B, scissor conservador, draw resolve cada pixel original. Alturas/limites permanecem compilados; faixas da tela são refeitas por câmera, sem I/O/readback na escolha. Custo completo inclui compute, dependência de storage e render.

Resultado negativo: 100× próximo nativo B0,63641/H1,02791/T1,01833ms; repetição B0,63932/H1,02614/T1,02737. Blocos não promovidos; máscara0 conservaB. Controle de superfície fixa real1/2/10/100T/pixel: B0,45–0,48ms enquanto denso cresce1,57→53,65ms; T1,14–1,23ms não melhora esse platô. Não atribuir ao símbolo a economia que o certificado de subdivisão já fornece. Qualidade1104/1104gates e profundidade preservados dentro dos limites; sem conclusão sobre malhas arbitrárias/fechadas.

## EAF-88 — uniões regionais e submissão indireta

Pré/pós-checklist e reprodução em `docs/coordinated-regions-study.md`. Uniões offline de charts válidos, até8 rodadas/1.024 faces; novas bordas externas fixas, internas livres. Comparação1/2/10/100× da Boulder01 sem genealogia no compilador, campo64, duas resoluções/perto/longe. Erro da recuperação anterior somado ao regional; normal interpolada curta válida preservada sem normalização prematura. Frame mantém seleçãoCPU/vetores e argumentos20B/u32; ring de benchmark10MiB. Atlas/bind groups reutilizados, fonte/candidatos residentes explícitos. API multi-draw de Metal ainda codifica vários desenhos.

Sem ganho líquido do incremento pesado:100× próximo nativo, anterior0,32944ms→coordenado0,37518→indireto0,38711; GPU melhora0,28438→0,23004, CPU cresce0,02782→0,13389. Não somar intervalos sobrepostos. Dispersão duplicada exposta; diferenças pequenas inconclusivas.1× seleciona no mínimo63.052/66.122T, até348ranges;100× permanece acima de222milT após recuperação265mil, fonte6,61milhões. Memória coordenada1×21,447MiB,100×52,688MiB fora fontes/targets/CPU/driver. Quatro controles de densidade falham longe em1728; candidatos falham nas mesmas poses. Não promover nem relaxar gates.683testesRust/Clippy/fmt,13gruposMCPheadless e1.024estadosDOM/DPR passaram. Próximo foco é recuperação de redundância/estabilidade da referência e custo de decisões/desenhos, não novos atlas indiscriminados.

## EAF-88 — recuperação numericamente estável

Pré/pós-checklist em `docs/recovery-stability-study.md`. Refit opt-in do plano/campo afim em potências de dois, usando pontos afastados da própria região e revalidando todas as faces sob limites antigos. Nenhum novo cálculo porpixel, textura ou formato; runtime mede bounds porframe e desenha pacote pronto. Build compara ambos e mantém anterior se não diminuir triângulos/bytes. Fonte de fallback continua residente no comparador.

100× Boulder01:265.105→67.139T (−74,7%), proxy7,107→2,201MiB; pacote completo209,74→203,63MB pois guarda fonte. Compilar ambos~11,79s fora doframe. Perto3456:total0,33107→0,27138ms (−18%),GPU0,28757→0,22856; sete de oito rodadas melhores, duplicatas5,1%.1×→100× selecionado0,26873→0,27138ms;2× ainda0,28889. Longe nativo sem ganho demonstrado (+1,8% dentro de duplicatas8,8%). Coortes com mesmo pacote não ganham por diferenças de timing.144/144 contra pedra-base passam;4/144 contra subdividida falham, diagnósticof64 aponta divergência no caminho de projeção/rasterização, sem eliminar gates.687testes/Clippy/fmt,13gruposMCPheadless,1.024estadosDOM/DPR. Ganho de redundância desta superfície, sem promessa de platô universal ou FPS.


## EAF-88 — relevo real com material rico

Pré/pós-checklist, protocolo, binários e resultados em `docs/rich-relief-study.md`. Novo heightfield de 1.179.648 triângulos, 144 regiões, cinco escalas com bordas nativas e certificados verificados em XYf32. Mapas RGBA8/mips compilados em build, mesmos materiais/iluminação para todos os controles. GPU usa posição 12B e índice 4B, sem normal/UV redundantes; seleção CPU reaproveita projeção por região, mas ainda aloca escolhas/ranges. Fonte + níveis + mapas 53,447 MiB contra 46,934 MiB de fonte + mapas; sem economia total de memória.

Resultado negativo de performance incremental: regional 0,05px passa 138/138 métricas de cor/cobertura e 10/10 retornos, mas reduz no máximo 0,98% dos triângulos e não vence consistentemente o anterior. Perto 3456 total anterior0,74090/regional0,79890ms; longe1728 anterior0,33757/regional0,39783ms. Variância e controles geometricamente iguais impedem atribuir pequenas diferenças a otimização. 0,15px reduz até10,68%, porém falha um retorno; controle conservador também falha um retorno; anterior reprova uma captura. Não promover. 712testesRust/Clippy/fmt,13gruposMCPheadless,690PNGs,2.208estadosDOM/DPR. Detalhe geométrico novo não herda o platô de subdivisão redundante; instabilidade dos controles continua aberta.


## EAF-88 — refinamento dentro das regiões do relevo

Protocolo/checklist/medidas em `docs/rich-relief-study.md`. Árvore temporária64→2células compilada em quatro limites de erro; mantém144decisões/frame e todos os perímetros nativos das folhas. Interseções verificadas em XYf32, variantes nativas/idênticas compartilham ranges e evitam16,5MiB de índices duplicados. Fonte+níveis+mapas70,261MiB contra53,447MiB uniformes; build observado12,449s. Nenhuma árvore/shader novo porpixel no comparador final; material original preservado porque derivadas finas também falharam na repetibilidade.

Ganho parcial:3456/distante total anterior+hierarquia0,346289→local0,05px0,195118ms(−43,65%,8/8rodadas);afastada−31,24%.449.200T selecionados(−61,92%) e1draw. Muito perto3456:0,776606→0,791177ms(+1,88%,semganho consistente),até33draws. Gates de cor/cobertura138/138 do local passam; máximo de canal0,4667 impede alegar pixels idênticos. Fonte falha um retorno3456/aproximação e uniforme um retorno3456/perto:runner termina código1,sem aprovação geral,coortes estáticas afastadas explicitamente separadas. 713testesRust/Clippy/fmt e13gruposMCPheadless passam;690novosPNGs e2.208estadosDOM/DPR. Próximos custos/limites:instabilidade de microtriângulos/filtragem,muitosdraws perto,memória maior; não promover ou prometer platô universal.

## EAF-88 — seletor de custo no relevo rico

Protocolo/checklist em `docs/rich-relief-study.md`: ajuste inicial com vistas separadas, coeficientes monotônicos por triângulos/desenhos e margem por resíduos/duplicatas; custo do seletor incluído nos lotes. Fronteira otimista derivada dos certificados no load permite rejeitar sem visitar144regiões, preservando a decisão. Mantém mapas/shader/material e gates; geometria anterior residente deve entrar na memória do automático. Alocações de ranges e uniform slots existentes continuam; sem contrato funcional, thread ou dependência nova. Diagnóstico de LOD explícito ainda falha em repetibilidade e não altera o caminho principal.

Posterior:715testesRust/Clippy/fmt e13gruposMCP aprovados. Margem corrigida para somente superestimação+ruído; atalho conserva decisão nas138capturas. Automático mantém ganho distante3456 de44,28%(8/8rodadas), mas perde oportunidade intermediária; perto desenha o anterior, sem ganho geométrico atribuído à variação de timing. Fonte+níveis+fallback+mapas90,528MiB; calibração inicial7,720/10,820s. Automático138gates numéricos aprovados e9/10retornos:um pixelPNG varia1/255 perto1728; auditoria retorna1 e painel avisa reprovação. Não promover globalmente nem alegar estabilidade resolvida. Detalhes e primeira tentativa preservados no estudo.

## Conexões regeneráveis de plantas72

Checklist anterior e dez respostas posteriores em
[plant-connections.md](plant-connections.md). União/CG somente no staging,
validação e consultas. CSR e256iterações limitadas, bases Arc preservadas,
composição copia retidos; nenhuma simulação/GPU nova. Frame inclui máscara de
até8instâncias na invalidação sem alocar no caminho estável. Recomputação de
validação após autoria é custo síncrono deliberado; mantidos pares disjuntos,
filho rígido, material único do pai e teto de desenho conservador.


## Receita refinada da conexão de plantas

Checklist antes/depois e custos em [plant-connections.md](plant-connections.md).
Sem alteração Rust/runtime: somente receita e estudo de autoria. Campo27144→48160
na avaliação síncrona, +522v/+1044t no asset e92002t desenhados; tetos mantidos.
Arrays de normais/arestas/percentis do estudo são temporários e limitados; sem
solver/otimizador novo. Nenhum ganho de FPS alegado.


## Segundo ContextGraft estático da árvore

Checklist anterior/depois em [plant-connections.md](plant-connections.md).
Sem Rust novo: uma união limitada adicional no staging da autoria, fonte119v,
resultado+1824v/+3648t;95416t desenhados e32instâncias, nenhum aumento de teto.
Somente731v restantes no asset é fricção real para próximas junções; divisão de
saída/skin compartilhado requer contrato separado. Sem promessa de FPS ou escala.

## EAF-88 — diagnóstico conjunto de filtragem e entradas

Protocolo prévio, respostas posteriores e resultados em `docs/rich-relief-study.md`. Filtro trilinear manual opt-in dos mesmos mipmaps; até16textureLoad porfragmento, sem payload novo. MRT diagnóstico grava cor e derivadas UV no mesmo desenho; target extra16B/pixel (29,452/117,809MiB), readback fora de qualquer timing, descartado ao reinstalar material normal. Attachments em array fixo, sem alocação nova pordraw no caminho comum. Nenhuma mudança no shader principal, material, seleção, cena/NPC ou contrato.

1.536retornos em duas coletas fixas: manual falha3/256semMRT, apesar de passar256/256comMRT. Nos37retornos com cor diferente doMRT, todos os pixels de cor alterada também mudam derivadas. Evidência conjunta corrige a inferência anterior baseada em shaders separados; não prova causa raiz/driver nem resolve estabilidade. Gates de retorno mantidos, runners terminam1 e diagnósticos preservados. Sem nova medição de performance/ganho. Roadmap gráfico registra atmosfera por amostragem como etapa posterior, após consolidação e cena composta.

## EAF-88 — footprint analítico do material rico

Protocolo/checklist e dados em `docs/rich-relief-study.md`. Shader opt-in calcula Jacobiano de projeção usando câmera/resolução física e normal compilada como aproximação de tangente. Mesmos mapas/mipsRGBA8, sem payload novo; uma amostra mip0 adicional e matemática porfragmento. Pipelines compartilham bindings/mapas; troca fora dos lotes, seleção/ranges existentes dentro. Material/shader original e seletor automático preservados.

Resultado negativo para adoção: candidatos repetem256/256retornos dedicados e10/10na trajetória, mas passam só97/138gates de imagem cada; manter geometria idêntica também reprova a troca de shader. Fórmula validada contra768interseções independentes raio/plano não valida a aproximação pela normal de shading. Sem ganho consistente: regional3456/distância6 total0,501186→0,541652ms(+8,07%,apenas1/8rodadasmaisrápida). Memória90,528MiB incluindo mapas/fallback,0Bdeasset adicional, sem targets/driver/CPU. Fonte original8/10retornos e anterior137/138imagens: falhas explícitas impedem promoção geral. Não repetir até passar, relaxar gates ou atribuir novidade/FPS.716testesRust/Clippy/fmt,13gruposMCPheadless; evidência separada de aprovação visual e integração.

## EAF-88 — tangentes geométricas compiladas

Protocolo/checklist antes/depois em `docs/rich-relief-study.md`. IGT1 armazena duas inclinaçõesf32/face,8B, hashes/versionamento/validade de fonte. Derivaçãof64 offline,9MiBnovos consumidos via primitive_index nofragmento; recursoGPUapenasopcionalnoharness. Geometria/material originais preservados, umdrawfonte obrigatório. Gradientes da projeção reutilizados nosdoismapas; pipelines/bindings compartilhados, sem expansão de vértices.

Resultado: candidatosLOD/gradientes geométricos138/138gates,128/128repetições e10/10retornos cada; normaldeshading98/138. Originalfalha64/128repetiçõesestáticas,embora passe10retornos;runnerreprova. Custo próximo3456 total0,779658→0,875909ms(gradientes,+12,35%,0/8rodadasmaisrápida);GPU0,736732→0,830669ms. Payload46,934→55,934MiB;não é comparador comtodososníveis/fallbacks. Nãoadotar,relaxargatesouatribuirFPS; separar custodeconsulta/Jacobiano e investigar compactação/compartilhamento antesdeintegraçãoregional.719testesRust/Clippy/fmt,13gruposMCPheadless; dados/resultados reprovados preservados.

## EAF-112 — segundo Pattern independente

Checklist anterior e alternativas em [layered-patterns.md](layered-patterns.md).
1. **Adequado:** autoria eventual e fragmento, sem trabalho de simulação novo.
2. **Adequado:** Option<Pattern> por material, Detail segueArc, nenhum canal por
vértice/cópia de geometria para o segundo campo.
3. **Adequado:** mesmo tipo fechado/validação; soma bump em f64 contra limite
f32 existente; layout80B e offset1152 explícitos em teste.
4. **Adequado:** kernel comum pattern_field, footprints uniformes antes dos
branches e uma reconstrução de gradiente mundial para ambas as alturas.
5. **Mantido com motivo:** até4oitavas extras habilitadas, total8, cellular27
vizinhos/oitava; sem prometer ganho de tempo/FPS ou elevar tetos.
6. **Mantido com motivo:** +80B/slot/+1360B/tabela,1232B/20944B; Vertex64/Globals176,
draws/slots/geometry budgets intactos. Derivada secundária também nos neutros.
7. **Não se aplica:** sem locks, threads, dependências ou I/O novo no frame.
8. **Validado:** igualdade/cache inclui a opção; cenário testa troca/remoção,
reindexação de slot, crescimento/redução da aparência, undo/load/reset/viewport.
9. **Validado:** preview/lote/MCP/replay, fontes ocultas/erro atômico, cor/bump/
frequência separados, rig e legado em pixels; evidência final em validation.md.
10. **Mantido com motivo:** tintas podem multiplicar até4/canal e saturar a saída
atual; sem máscaras/texturas/grafo/PBR completo. Casa só muda materiais, não
corrige proporções/encaixes. Medir performance antes de alegar benefício de FPS.

## EAF-114 — metalness opt-in

Checklist/alternativas antes do código em [metallic-response.md](metallic-response.md).
1. **Adequado:** autoria/fragmento habilitado; nenhum tick/skinning novo.
2. **Adequado:** um f32Copy no Reflectance; Geometry/Detail/rig/instâncias seguem
os mesmos canais/Arc. Não há arrays/texture fetch por fragmento.
3. **Corrigido:** high.w antes livre tinha default1 nos neutros; agora0 explícito
impede metal implícito sem pattern/reflectance. Bounds finitos, incompatibilidade
com transmission e preservação do restante do encoding testados.
4. **Adequado:** kernel GGX compartilhado produz NoL/fator/Schlick; caminho novo
avalia uma vez e mistura BRDFs. Componentes/ordem do cálculo legado preservados.
Sombra avaliada uma vez. Sem normalização/scan de geometria duplicado.
5. **Adequado:** zero oitavas/loops novos, mesmos caps/draw/slots; poucos vetores
aritméticos adicionais quando metallic>0. Nenhum ganho de FPS alegado.
6. **Adequado:** GpuMaterial1232/tabela20944/Vertex64/Globals176 intactos; nenhum
buffer/pipeline/render pass novo. Readback apenas nas capturas explícitas.
7. **Não se aplica:** sem lock/thread/dependência nova.
8. **Validado no cenário específico:** toggle/default, reindexação de slots,
undo/load/reset/viewport e replay; igualdade inclui scalar em Reflectance.
9. **Validado no cenário específico:** preview/erro/gates em3fontes ocultas,
GGX analítico/quase normal, interpolação física em linear, cor/roughness/F0,
luz/sombra/rig, legado da casa em3vistas byte-exato. Gates finais em validation.md.
10. **Mantido com motivo:** ambiente constante/UNORM/falta de IBL e antialiasing
especular são limites visíveis; manter a prova separada de nova arquitetura de
recursos. Máscaras por região e transparência precisam de casos/contratos próprios.

## EAF-116 — ambiente procedural pré-filtrado

Contrato/alternativas/checklist anterior em procedural-environment.md.
1. **Adequado:** bake eventual por conteúdo; fragmento opt-in faz consultas
pré-filtradas, sem integrar amostras/tick. Cena inválida recusa antes do renderer.
2. **Adequado:** saídas CPU limitadas, chaveCopy normalizada/sem intensity,
3texturas reutilizadas e nenhuma retenção de histórico de ambientes. Normalizar
chave a cada draw é custo pequeno constante mantido, não um bake por frame.
3. **Adequado:** dados finitos/bounded, UNORM8linear e dois coeficientesUNORM16
empacotados emRGBA8; pack/layout/constante/limite analítico testados. Globals192B.
4. **Corrigido:** amostras GGX por nível/linha e pesos por nível pré-calculados,
LUT lazy uma vez, cache antes dos uploads. Counters GPU/MCP confirmam que trocar
intensidade/câmera/material/pose e repetir/resize não rebakeia.
5. **Adequado:** 64/8/64,128/512 amostras fixos; sem parâmetros de custo livre.
Mesmos tetos de meshes/instâncias/draw; sem promessa de FPS/escala.
6. **Mantido com motivo:** +148984B lógicos de texturas por Renderer/1sampler e
+16BGlobals. Driver pode reservar mais. None não prepara CPU; recursos vazios
alocados no init. Metal puro só usa cubeSpec+LUT; difuso sem lobo só cubeDiff;
pior caso também difuso traseiro. Nenhum loop de integração/texture asset novo.
7. **Não se aplica:** sem lock/thread/dependência nova. Preparo síncrono pode
custar tempo na primeira captura/edição; não alegar latência medida.
8. **Validado no cenário:** None/A/B/A/intensity, raiz normalizada, slots,
sombra/rebind, undo/load/reset/replay/viewport; cache por conteúdo, não revisão.
Counters são diagnósticos de recurso, não estado de jogo.
9. **Validado no cenário:** valores analíticos constantes, metal branco áspero
próximo de1−ln2, legado da casa em4vistas exato, luz/material ablações, rig e
formatos/erros. Gates completos finais em validation.md.
10. **Mantido com motivo:** split-sum/v=n, amostragem finita/quantização/semAO,
multiscattering/HDR/tonemapping/specAA e reflexão local ausentes. Não confundir
ambiente de3cores com fotografia HDR ou correção da arquitetura da casa.


Integração de composição com aparência: checklist anterior e custos finais em
[cottage-lookdev-integration.md](cottage-lookdev-integration.md). Nenhuma nova
fase/buffer/limite; fixture de fonte1.8MB mantém comparativos. Probe/LUT
continuam um bake por conteúdo/renderer,148984B lógicos.869Rust e89MCP/365grupos
aprovados; sem afirmação de FPS ou fidelidade artística.

## Inspeção de normais avaliadas — EAF-119

Checklist anterior e dez respostas finais em
[mesh-normal-inspection.md](mesh-normal-inspection.md). Rotinas de normais do
renderer extraídas sem mudar aritmética; query explícita com pose/scratch192KiB,
resposta topK≤32, sem novo tick/recursoGPU/deps. Quatro capturas anteriores da
casa byte-exatas. Correção localizada via cadeia Sweep existente +72v/+148t;
uniforme spans por cúbica é custo mantido identificado, sem alegação de FPS.

## Resolução por curva — EAF-120

Checklist anterior e dez respostas finais em
[sweep-curve-sampling.md](sweep-curve-sampling.md). Só autoria, contagens antes
da saída, cursor por curva e listas≤16. Mesmo runtime/layout/tetos. Plano uniforme
mantém pequeno vetor por tentativa≤8; lista não refina retas desnecessariamente.
Mola[1,9] economiza80v/160t ante variante uniforme, sem alegar FPS. Erro central
não é certificado de superfície/seção; mesmo contrato surface_validated:false.


Integração de relevos/aparência: checklist anterior e custos finais em
[cottage-relief-integration.md](cottage-relief-integration.md). Só conteúdo e
autoria síncrona,16slots e tetos mantidos;31948v/820livres. Nenhum recursoGPU
novo, mesmo binário validado. Três estudos afetados aprovados; fonte muda só
bronze antes de corrigir um contato da pedra. Folga/posição manual continuam
fricção de autoria, não há solver de apoio visual ou FPS medido.

## EAF-110 — estabilidade do traçado de escultura (29/10)

Decisão/checklist anterior e respostas do diff em [editor-sculpt.md](editor-sculpt.md#correção-do-traçado--decisão-antes-do-código).
Picking consulta o documento congelado do gesto; reamostragem transporta resto
por comprimento e conserva eventos coalescidos. Até128 aplicações/gesto, sem
alterar kernel, força por amostra, Scene81/replay85/layout3 ou topologia.
Raycasts por evento, normais/validação/upload síncronos e clones limitados da
prévia permanecem custos explícitos. Dois casos novos de entrada egui e probe
GPU isolado verificam qualidade, cadência, histórico e comando compartilhado.
Sem estimativa de FPS nem aceite humano inferido de testes/capturas.

## EAF-104 — reorganização do editor e campos horizontais (29/10)

Checklist/alternativas anteriores e dez respostas em [editor-ui.md](editor-ui.md#segunda-direção--reorganização-nativa-2910).
Faixa de ferramentas/contexto dentro da viewport e inspetor Pincel/Objeto;
retângulo físico exclui controles antes do picking. Scrub reutiliza edição/foco/
teclado do egui com draft local e cache f64/f32 que rebasa em alterações externas.
Sem mutar entrada global, publicar falsa mudança vertical, clonar cena ou introduzir
shader/dependência/thread/fonte. Miniaturas vetoriais têm custo pequeno e limitado
por três pincéis; layout imediato/strings/scans síncronos seguem explícitos, sem
alegação de FPS. Gates finais/capturas em validation.md e artifacts/editor-studio.

## EAF-104/110 — autoria imediata, componentes e traços inversos (29/10)

Decisões anteriores em [editor-ui.md](editor-ui.md) e [editor-sculpt.md](editor-sculpt.md);
dez respostas do incremento em [editor-workflows.md](editor-workflows.md#revisão-estática).
Edições são publicadas somente por controles alterados; Scene/Geometry COW e
histórico32 mantidos, um registro por gesto consecutivo, validação também em
updates coalescidos. Schemas/cache de formulários OnceLock; listas paginadas16,
profundidade limitada e tetos de domínio finais. TransformVertices reutiliza
o transformador de ilhas, conserva canais/rig/clipes e recusa inversão f32.
Seleção mantém tipos vértice/aresta/face e referência de revisão; subdivisão não
expande uma aresta para todas as arestas incidentes de seus vértices.

Draw acumula influência da superfície inicial, com normais calculadas uma vez
por avaliação; prévia completa O(T+V×amostras≤128), sem realimentar a deformação.
Reconstrução síncrona de runtime a cada edição válida, cópias de rascunhos de
formulário e poses/projeções dos overlays pedidos são custos mantidos. Arestas
deduplicadas cacheadas por geometria; nenhum shader/recursoGPU/worker/dependência
novo, nenhum acesso a estado oculto de NPC ou ampliação de pools. Fullsize
content/titlebar é configuração de janela, sem mudar contratos Scene83/replay87/
layout3/projeto1. Sem alegação de FPS ou equivalência de UX por contar comandos.

## EAF-128 — revisão proporcional da consolidação

Preflight e fronteiras em main-consolidation.md: trabalho de integração/autoria, sem novo algoritmo no tick, novo limite ou otimização especulativa. Conservar caches de render/mesh/controle e mundo terrestre, índices Contributions e write sets locais. Execução dos gates no binário conjunto é obrigatória; resultados isolados não substituem regressão. A remoção de244arquivos exclusivos Nanites exclui os experimentos dos alvos de build da main; não implica ganho de FPS. Respostas finais e evidências serão registradas após validação.

Navegador recuperado da frente file-browser: páginas256/keeper257, biblioteca32/favoritos16/JSON256KiB, descoberta8raízes×4páginas/64candidatos, somente por gesto/startup; nenhum scan no tick/frame. Scans de diretório completos para obter a página ordenada e I/O síncrono por gesto são custos mantidos, não um indexador global. Preferências separadas de Scene/Replay; criação exclusiva e persistência por arquivo temporário/rename. Recibo de seleção atualiza view_revision somente após limpar ator selecionado; regressão cobre erro atômico e sucesso.

### EAF-128 — respostas finais

1. Adequado: nenhuma nova fase no tick; navegador/metadados por gesto/startup, seleção por comando.
2. Adequado: caches e writes locais preservados; keeper257 e bibliotecas limitadas. Cópias pequenas de rascunhos/estilo UI e I/O síncrono por gesto mantidos.
3. Adequado: mesmos floats/IDs/layouts; biblioteca1 fora de Scene86/replay90. Nenhuma migração de capacidade/ECS.
4. Corrigido: integrar cada delta uma vez; lanterna não deslocada novamente; identidade de linha por caminho e aliases resolvidos para metadados.
5. Mantido com motivo:9atores, limites nativos e orçamento restante180v da demonstração; não ampliar caps para mascarar conflito.
6. Adequado: pipeline/cache gráfico preservados e11checks visuais no mesmo build; nenhuma captura de janela/readback no tick novo.
7. Adequado: frentes em checkpoint e merge central, sem novos locks/workers.
8. Adequado: revisão final da seleção após limpar ator; erros preservam estado, arquivo temporário/rename e remoção de registros ausentes cobertos.
9. Preservado:1139Rust/114alvos,113MCP/482grupos e11checks da casa; controles+KCC+ground+mesh no replay conjunto.
10. Mantido com motivo: scans completos de diretório para páginas ordenadas, I/O/queries síncronas e recompilação dos alvos após mudanças do núcleo. Nenhum ganho de FPS/escala alegado.


## EAF-104 — correção dos campos de componentes (29/10)

Checklist prévio em editor-workflows.md.1.apenas evento de campo/resultado e
sincronização de seleção; nenhum tick novo.2.um String de erro por sessão,
alocado só em recusa, sem cópias novas de geometria/histórico.3.revisão prevista
não substitui revisão aceita; centro permanece local ao asset.4.TransformVertices,
EditSceneLive, Scrub e resultado do host compartilhados, sem kernel alternativo.
5.limites/validadores inalterados.6.sem GPU/readback/I/O/dependência novos.
7.mesmo dispatcher sequencial; callback de erro atualiza somente UI.
8.recusa conserva seleção/campos; undo+nova seleção reidrata referência atual;
clear externo continua sem ressurreição.9.regressão47v red→green e probeXYZ com
recusa/idle/recuperação/undo; gates finais em validation.md.10.validação síncrona,
COW/reconstrução e projeção existentes mantidas; sem alegação de FPS ou aceite
físico por compositor offscreen. Menor passo numérico reduz overshoot da interação,
sem alterar deslocamentos explícitos enviados ao MCP.

## Autoria direta29/10 — gizmos e pincéis de forma

Checklist prévio, decisões/tradeoffs e respostas finais das dez perguntas em
[editor-gizmos.md](editor-gizmos.md) e [sculpt-brushes.md](sculpt-brushes.md).
Gizmos mantêm seleção/topologia em cache Arc e prévia Scene somente por movimento
efetivo, sem recriar Simulation; commit único ao soltar. Seis pincéis de forma
usam base congelada, planos128 e buffers limitados por16384v; vizinhos só para
Relaxar e nenhuma cópia adicional de triângulos. Kernel compartilhado com MCP,
orientação de faces protegida e sem trabalho de edição no tick. Renderer/ABI,
Scene86/replay90/layout3/projeto1, budgets e dependências mantidos. Gates finais
completos e build identificado estão em validation.md; não há claim de FPS,
multiplataforma, tablet ou aceite físico decorrente de testes offscreen.

## EAF-130 — modelagem por componentes

Decisão prévia e respostas pós-implementação das dez perguntas em
[editor-modeling.md](editor-modeling.md#revisão-estática-do-incremento).
Geometria/BVH cacheadas na prévia, kernel tipado e limitado, commit único sob
scope/revision e teto global de armazenamento/desenho. Custo de cópias/recortes
na autoria aceito; nenhuma nova Simulation, dependência ou trabalho no tick.
Bevel convexo plano e inset plano convexo têm domínio explícito, sem fallback
que modifique silenciosamente a geometria. Evidências finais em validation.md.

## EAF-131 — avaliação antes da migração de atores

Contrato/tradeoffs e dez respostas em [scalable-actor-runtime.md](scalable-actor-runtime.md).
Componentes densos por slot, ator16bits, conjuntos limitados por palavras e ordem
cacheada; preservar writes locais e índices de buffs. Até1.000Players/8NPCs com
budgets próprios, sem ampliar apenas arrays. Testes novos somente para riscos
críticos de overflow/identidade/posse/atomicidade/replay; reusar cenários existentes
conforme impacto. Fontes/output liberados por EAF-130 antes da migração. Sem ganho/capacidade
sustentada, rede ou cliente gráfico alegados.

## EAF-110 — reincidência de espinhos na escultura

Checklist anterior e revisão posterior em
[editor-sculpt.md](editor-sculpt.md#reincidência2910--direção-do-pincel-e-proteção-de-deformação-eaf-110).
Draw com normal média da área por dab, guarda/rollback de orientação em
Draw/Grab e confirmação de prévia válida; nenhum fator de força oculto, remesh
ou formato novo. Custo de dois passes limitados V×amostras, cópia V/guarda T
aceito para evitar publicar geometria dobrada. Validação focada no risco de
corrupção, usando probes existentes; sem suíte ampla automática/claim de FPS.


## EAF-132 — execução contínua separada da gravação

Contrato, alternativas, checklist prévio e dez respostas finais em
[continuous-runtime.md](continuous-runtime.md). Recorded mantém limites de replay;
continuous conserva somente pendências/recibos limitados e anel manual128, sem
realimentar sensores. Counters64bits independentes dos buffers, controle conta tick
em O(1), guardas numéricas e exportação antes de I/O. Reset/load/projeto conservam
política e renovam escopos. Nenhuma dependência/worker/rede/GPU nova. Custo mantido:
serialização explícita, buffers limitados, mesmos16pedidos globais de habilidade
até a migração de admissão. Evidências nativas/MCP no build7138146e3d188320 em
validation.md, sem alegação de FPS ou escala sustentada.

## EAF-133 — admissão por ator e carga direta1000

Decisão/checklist prévio e revisão de10itens em [frame-admission.md](frame-admission.md).
Índice FIFO elimina actor×entries, counters/lookup são locais/cacheados. Quotas16por
ator e global real, controles contínuos sem histórico crescente.121Rust/13MCP54grupos
e Clippy passaram. Carga real1000corpos/110000ações correta; simulação p95=108,271ms
expõe visada linear, sem alegar60Hz sustentado/FPS/rede. Próximo recorte EAF-134.

## EAF-134 — broadphase de visada no snapshot

Checklist prévio e revisão completa em [combat-broadphase.md](combat-broadphase.md).
BVH mediana/centroides, buffers reutilizados e stack32 por consulta; narrowphase
e regras intactos.1oráculo crítico novo/111Rust existentes/11MCP42grupos e Clippy
passaram. Mesma carga1000 correta: simulação p95=108,271→2,515ms; total3,252ms,
sem renderer/rede/pools ampliados. Rebuild por tick e pior caso linear mantidos.

## EAF-135 — projéteis e origens por fonte

Decisão e revisão de10itens em [source-delivery-pools.md](source-delivery-pools.md).
Pool denso/indexado/encapsulado; quotas64projéteis/32contextos por slot e teto
real, leaseu16 e snapshot65/64reservado antes do custo. BVH filtra sweep, narrowphase
TOI/ID intacto. BTree mantém alocações de nós, JSON/Rapier mantêm heap; não se
promete zero alocação.136Rust/14MCP60grupos/Clippy aprovados. Carga1000projéteis
correta p95simulação2,838ms/total3,670ms; sem outros pools/render/rede medidos.

## EAF-136 — agendas por fonte e seleção local de dissipação

Decisão/revisão10itens em [scheduled-source-pools.md](scheduled-source-pools.md).
Pool source/target, quotas64/16, IdSet512local sem custo quando vazio, cleanup
marcado por tick/morte em vez de scan por lançamento.131Rust/14MCP57grupos e
Clippy passaram. Carga1000periódicos p95simulação2,955ms;1000áreas/2000volumes de
pico correta, porém p95=78,607ms devido a classificação/ordenação completa de
candidatos de diagnóstico. Próximo recorte trata esse custo, sem culling semântico.

## EAF-137 — seleção pequena e diagnóstico completo sob leitura

Checklist/revisão em [area-selection-runtime.md](area-selection-runtime.md).
Posições por Entity no snapshot, oclusão por BVH/interior0, todos rows congelados,
ordenação só de elegíveis no tick e completa sob leitura.104Rust/16MCP66grupos/
Clippy aprovados. Mesma receita1000auras correta:p95simulação78,607→7,683ms;
total8,355ms. Custo linear/Vec de todos rows, cache extra se observado e custo
de dump completo mantidos; sem renderFPS/rede/combate arbitrário prometidos.

## EAF-141 — sessões e réplica

Checklist prévio e revisão em network-session-runtime.md: filas16/64 por sessão,
latest input, tap opt-in limitado por entradas/atores, codec1MiB/1024atores
com parse antes de mutação; nenhuma rede/GPU/Scene clone no domínio.
68Rust (3 novos críticos), Clippy all-targets e9MCP/33grupos no build
f8b11dd7370eac27. Socket/encoding compartilhado/cliente lento pertencem EAF-142;
sem alegação de ganho de FPS ou1.000conexões de testes em memória.

## EAF-142 — QUIC real

Checklist prévio/revisão/evidência em quic-transport.md. Mesmo executor local,
I/O/TLS/serde/codec em runtime separado; mailboxes16/64, owner latest, snapshot10Hz,
cache12frames/baseline e fragmentos compartilhados. Bound global/por conexão,
vida/época/geração/dedup e timeout neutralizam.79Rust afetados+2QUIC críticos,
Clippy all-targets,11MCP48grupos e CLI headless no builda57ddc00a2a686ce.
1000conexões reais/20milhits/roster completo: passo p953,634ms;~401MB UDP
recebidos pelos clientes. Corpo dinâmico/hits pareados, sem WAN/cliente gráfico.
Custos mantidos: JSON controle/owner, mapas/Vec, notify/timer por conexão; não
inferir60Hz universal ou FPS. Retenção480MB gerados inativos removidos,
release único/estado incremental corrente e app instalado preservados.

## EAF-143 — apresentação pública remota

Checklist/revisão em remote-presentation.md. Pacote público8MiB uma vez por época,
Arc/bytes comuns e codec fora do mundo; snapshot4096primitivas/1024atores,
projéteis visuais4096 com truncamento independente da física. Renderer/ABI intactos,
mesma composição janela/capture MCP e sem simulação local ao conectar.63Rust+
2QUIC, Clippy,9MCP39grupos no builddde98de56108e5f0;1novo Rust de risco crítico.
1000TLS repetidos3vezes, captura1000atores/1005boxes; SSIMhost×cliente0,99997459.
Timeout isolado anterior registrado sem causa/correção alegada. Custos de props10Hz,
JSON inicial, lock curto da réplica na preparação CPU e VFX budget mantidos.
Sem FPS/WAN/teclado físico inferidos; offline negado preservado.

## EAF-161 — save portátil canônico

Contrato prévio/dez respostas finais em [portable-saves.md](portable-saves.md).
Primárias próprias com CCD/velocidades por builder (setters ignoram KCC posicional),
AABBs sem tick para IK e candidato atômico; nenhum internalsRapier no arquivo.
Captura/CBOR/hash/fsync apenas pausado; continuous restaura sem serializar caches,
recorded gera baseline nativa local. Índices Option<u16>,128MiB/profundidade128,
sem novos workers/GPU/rede/áudio. Cópia validada/recompilação de caches mantidas.
Prova crítica física/gameplay/identidade/custos/replay/100NPC+500Players/rootIK passou;
MCP1cenário/2grupos no build3f31de512312c2ee, prefixoPNGexato. Erro local180ticks
4,2945147e-5m (<2mm), sem bounduniversal de trajetória/WindowsLinux/FPS inferidos.
Validação final sobre95e72e0:4provas críticas/4alvos Rust, Clippyalltargets-Dwarnings,
fmt/Python/diff aprovados; sem custo novo no tick nem medição de FPS inferida.


## EAF-165 — substituição atmosférica antes do desenho

Contrato, revisão estática e medidas em [atmosphere.md](atmosphere.md). ATM2 compilado
explicitamente por autoria, prova de derivação memorizada em dados imutáveis;
seleção por instância e até5níveis, erro em pixels físicos + contribuição residual.
Shader24B/v sem avaliação de material, buffers escolhidos antes do upload; fonte
inativa usa diretamente os buffers anteriores. Sem partículas/worker/crossfade;
fonte CPU editável preservada. Reentrada pode realocar/uploadar: registrar transições,
não só mediana estável. Sombras/IBL/specular/animação conservam fonte; uniform+80B.
As duas provas novas protegem atomicidade, formato, persistência/replay/rede e
certificados forjados, inclusive após a cache; inspeção e comparação nativa cobrem
as alterações visuais. UI/MCP compartilham comandos; versão100/110/conteúdo2.

### EAF-168 — silhuetas distantes, revisão de custo

Inspeção prévia: esfera de toda a montanha antecipava indevidamente sua distância
mínima; expansão por triângulo repetia posições, e capacidade em potências de2
com margem4× podia aumentar memória mesmo após reduzir payload. Solução usa
caixa orientada conservadora, índices existentes e blocos4096 com margem25%.
Sem formato persistido novo, worker, rede, locks ou I/O por frame; cache do plano
preservada. Comparação direta de bytes Pod substitui igualdade por campos float, sem hash ou
alteração do payload. Ainda há leitura linear no renderer e cópia/upload na troca
real de representação. Fonte CPU continua residente. Não extrapolar contagem
menor de dados para ganho de tempo sem benchmark. Probes e limites em
[atmosphere-cost.md](atmosphere-cost.md); cores configuráveis pelo mesmo MCP/UI.


### EAF-169 — camadas autorais e cobertura da seleção

O perfil opcional separa cor do céu, preenchimento e perda de detalhe; nenhum
volume/partícula ou material completo é criado para as instâncias substituídas.
O limite de contribuição usa transmitância e peso residual dos detalhes; limites
de geometria continuam em pixels físicos. ATM3 só guarda a retriangulação plana
adicional quando seus bytes compensam o payload GPU economizado em uma instância.
Fonte e material CPU continuam presentes; não inferir memória total/streaming.

A seleção do editor usa posições já posadas, buffer reutilizado e máscara R8
por viewport, sem leitura de volta no draw. Dois passes extras somente com
seleção, profundidade armazenada nesse caso, máscara liberada ao limpar seleção.
O contorno evita varrer a caixa em CPU e segue cobertura/oclusão reais, mas
acrescenta rasterização e busca de vizinhança: não é uma otimização de tempo do editor.
Perfis, formatos, custo completo e limites em [atmosphere-silhouettes.md](atmosphere-silhouettes.md).


### EAF-170 — acabamento de bordas opt-in

Inspeção prévia: não havia alvo legível de cor/depth no render direto. Suavizar
depois do shader exige leituras e não elimina geometria. A opção mantém o
caminho antigo quando ausente/zerada; cria targets somente quando ativa, guarda
pipelines/pesos e não lê pixels para CPU. Dois passes com raio máximo12 e
retorno rápido no primeiro plano/interior sem descontinuidade. Guia/outline são
compostos depois. Geometria fonte preservada durante o efeito, porque proxies
podem alterar sua classificação de profundidade; limites de material continuam.
Dados são uniformes/texels tipados, sem JSON por frame. Testes novos limitados
a formatos/atomicidade/persistência; provas visuais nativas e custo em
[atmosphere-edge-softness.md](atmosphere-edge-softness.md).


### EAF-176 — densidade e persistência local

30mil árvores usam os mesmos assets/render/qualidade, acrescentando instâncias na
mesma paisagem. Geração/compilação ocorrem fora do frame. A persistência de cenas
locais passa a BufReader/BufWriter, sem vetor de bytes do documento completo nem
dupla serialização pretty/compacta. Validação integral antes da troca e arquivo
temporário/flush/sync/rename antes de marcar salvo. Replay/rede/layouts preservam
seus envelopes; sem dependência, nova fase GPU, lock ou worker. Fonte autoral
continua residente; não é streaming mundial. Prova e resultado em
[landscape-forest.md](landscape-forest.md#densidade-triplicada--eaf-176).

### EAF-177 — oclusão corrente na GPU

Contrato e números em [render-occlusion.md](render-occlusion.md). Passe de depth
com geometria real, tiles conservadores16×16 e compactação estável indireta;
cache de buffers/texturas e capacidades, origem/limites do dispositivo, sem
readback no frame. Dados tipados32B/bound,16B/job,80B/instância,32B/indireto.
O CPU ainda percorre a seleção e envia bounds/jobs, cerca de0,08–0,33ms adicional
nas vistas pesadas; nenhuma alegação de redução da simulação/CPU. Prepass até
262144triângulos/128oclusores limita custo extra, sem limite autoral novo.

Em30mil árvores,2048×1536, o tempo draw+espera caiu22,7%/26,6% com RGB8exato;
15,57–19,09MiB extras de scratch/cache. Fonte e geometria permanecem residentes.
Instâncias sem oclusor útil/baixo trabalho/limites incompatíveis seguem original;
ordem, sombras, seleção e formatos preservados. Falhas preliminares de validação
WGSL e fixture foram corrigidas antes da medição, sem relaxar equivalência.


## EAF-185 — visão física e paisagem20km

Contrato/revisão em docs/player-walk.md e expanded-world.md. Câmera de sessão
sobre Frame consulta1ator, sem editar Scene/reconstruir física por frame. Input
relativo vira input mundo pelo dispatcher; controle alheio recusado. Bosques em
build usam índices espaciais e busca em eixos reais, máscara de água limitada ao
envelope, sem custo de geração por tick. Fonte e BVH/GPU residentes separados.
20km estático com1136356v/2264192tri;30mil árvores por instanciamento, não
geometria duplicada por exemplar.4bordas533v exatas; raízes<=0,675mm apósroundtrip.
Modo continuous preserva histórico0. Não foi medido FPS desta janela; medições de
617MB RSS semGPU/floresta e pico4,24GB de geração não representam RAMtotal do jogo.
LOD móvel/streaming181 e precisão visual de horizonte continuam separados.


## EAF-191 — plantas autorais e dinâmica material

Contrato/preflight em [botany-integration.md](botany-integration.md). Fonte em
Arc, curvas/endereços materiais preparados no bake, sem parse/I/O por frame.
Malha madura estática segue o renderer existente; overlay não duplica toda a
fonte no histórico. Preview e regeneração são custos síncronos de autoria; não
foram apresentados como sliders com latência garantida. Normalizador de folha
palmada preparado por lâmina remove repetição do mesmo cálculo por vértice.

Quatro fontes totalizam607856v/1200368tri (duas peças por planta), antes de
qualquer substituição/culling do renderer. Cena de galeria local ~49MiB em JSON,
separada dos envelopes de replay/rede. Isso não é uma floresta ou teste de FPS.
Fontes/arcos e malhas são preservados por Arc nos32undos existentes.

Ensaio físico é opt-in, local, com até65536pontos. Matriz de vínculos nunca é
densa: a projeção usa produto Jacobiano/massa por stencils, precondicionador
diagonal e até512iterações de CG; distância/células/contato têm quotas próprias.
A candidata compartilha compilação imutável e copia somente os vetores de estado
e restrições. Malha derivada é recalculada e reenviada quando o ensaio avança;
é um custo CPU/GPU real, sem promessa de deformaçãoGPU ou tempo de gameplay.
Contato usa grade limitada, não todos os pares globais; excesso retorna erro,
sem retirar órgãos para cumprir orçamento. O painel consulta massa diretamente,
sem serializar relatório JSON ou construir broadphase no desenho estável.

Não foram criados workers, dependências ou cache sem limite. Build corrente
compartilhado: target/release/ignis; fontes/capturas/recibos substituem caminhos
em artifacts/botany-native. Janela humana e cena aprovada preservadas.

Publicação MCP final (leitura+compilação+transação) observada: palmeirinha0,122s,
plátano0,109s, eucalipto0,035s, figueira0,140s. Não são garantias de latência;
o processo compartilha o Mac. Crescimento acelerado da figueira completa e
atualização das malhas/relatórios custaram13,51s para128subpassos: custo explícito
de um ensaio autoral, não evidência de tempo real. O renderer invalida seu cache
de geometria quando o stream compartilhado muda (`render_instances::Gpu::prepare`);
o overlay não acumula uma página GPU por instante da planta.
