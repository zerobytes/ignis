# Apresentação remota — EAF-143

## Decisão e checklist antes de código

Continuidade de EAF-117/141/142 e405dde7. O cliente nativo deve mostrar o mundo
recebido sem rodar uma segunda autoridade. Escolha: conteúdo visual público
separado da cena administrativa. Exigir cópia idêntica do projeto esconde desvio
entre clientes; serializar Scene revela políticas/templates/atributos privados.
Uma whitelist explícita conserva câmera/luz, meshes/rig/clips/materiais
procedurais e IDs de instâncias já consumidas por conexões de plantas. Esses
tipos não têm caminhos/execução de scripts/importação de arquivos externos.

Conteúdo é limitado8MiB, versionado e ligado à época do mundo; capturado uma vez,
serializado fora da mutação e compartilhado entre conexões. Uni stream separado
com kind+length; leitura/escrita por blocos com prazo de progresso e prazo total.
Full/delta usam kind distinto. Validar inteiro (câmera/luz/mesh/hierarquia/count/
consumo/época) antes de publicar ou tocar GPU. Sem deserializar um Scene remoto.

Primitivas visíveis atuais passam pelo mesmo catálogo de InstanceId lógico já
usado por TODAS entidades; não criar IDs de slots nem hashear rótulos. Projeção
inclui fixos/visuais/dinâmicos/Player/NPC com kind/geometry explícitos; vida/saúde
só em atores. Limites4096entidades e1024atores, todos1000Players do combate.
Formato de réplica e ALPN sobem para2; Scene88/replay97 inalterados.

Projéteis de apresentação têm namespace de execução separado, posições atuais e
extensão, nunca dano/recursos/efeitos privados. Até4096visuais, truncamento
explícito e seleção determinística; budget não remove projéteis da física nem
participantes do combate. Estado de projéteis é latest dentro do frame atômico,
independente de terem nascido e morrido entre amostras. Não prometer VFX completos
de todas entregas/pior caso de pools a partir desse recorte.

A mesma composição alimenta janela jogável e capture_multiplayer via MCP;
Renderer/Vertex/Globals/shader ABI preservados. Janela de autoria continua local.
Conectar cliente pausa simulação local e impede entrada local na janela remota;
até a unidade de controles, input/ações são os comandos remotos MCP/API já
existentes. Desconectar transporte congela vista com status; voltar ao offline
exige disconnect explícito, sem fallback de controle para outro personagem.
Captura offscreen não comprova foco/teclado nativo ou FPS.

1. Frequência: conteúdo uma vez por época; primitivas10Hz no snapshot existente.
2. Alocações: Arc de geometria/conteúdo, frames/buffers limitados; sem Scene clone
   por tick ou objeto GPU na simulação/decoder.
3. Dados: kind/geometria/identidade tipados, codec explícito v2; arquivos externos
   e informação de IA privados permanecem fora da whitelist.
4. Repetição: cache público/baselines/fragmentos comum; material/mesh frame reusado
   no cliente, skinning só se tick visual/documento muda.
5. Bounds:8MiBconteúdo,1MiBréplica,4096primitivas/1024atores/4096projéteis visuais,
   mesmos meshes16assets/32instâncias e limites de geometria existentes.
6. GPU/I/O: codec e rede fora do lock; validação antes de upload; captura sob demanda.
7. Uma autoridade de escrita; cliente é só apresentação, sem física divergente.
8. Época/conexão/vida cercam reset/reconexão; decoder não publica parcialmente.
9. Provas novas somente privacidade/bytes/epoch/codec e integração render material;
   reaproveitar QUIC1000/offline/render/controles existentes conforme impacto.
10. Custos mantidos: statics na projeção10Hz, JSON8MiB uma vez/conexão, owner/notify/
    timers do backend anterior; sem ganho de FPS alegado. Conteúdo grande/resume
    pode custar bytes reais; caches persistentes/predição/interpolação/hotkeys têm
    contratos próprios, sem inferência de conclusão do MMO.

## Revisão e evidências

Build `dde98de56108e5f0`,127inputs, protocolo/ALPN2, Scene88/replay97.63Rust
integração/janela (incluem3domínio+1novo crítico de conteúdo),2funções QUIC,
Clippy all-targets -Dwarnings e9MCP39grupos passaram. Offline com rede negada
permanece nos cenários continuous_runtime/local_transactions. Não houve teste novo
para cada campo/botão; o único novo Rust protege privacidade, formato e publicação
atômica de conteúdo/props/projéteis inválidos.

MCP entre2processos cobre conteúdo maior que um bloco32KiB (2assets,5769v de draw),
meshes/material, rampa, prop dinâmico, atores e projétil. Capture640×480 remota vs
host em mesmo tick:4pixels/307200diferentes, RGBMAE2,570125e-6,
SSIM0,99997459017, compatível com quantização milimétrica. Captura após transporte
fechar é idêntica à remota anterior; retorno ao mundo local ocorre apenas por
disconnect explícito. Cliente recusa Play/step local e tick permanece0 até sair
do modo remoto. Editor/cena de autoria e capture legado continuam locais.

Carga1000TLS/20milhits passou. Depois de um timeout isolado na abertura (apenas
"deadline has elapsed", etapa não registrada naquele momento), diagnóstico por
fase TLS/welcome e estado de falha foi acrescentado. Causa NÃO estabelecida,
sem aumento de prazos ou relaxamento de asserções. Três repetições consecutivas
passaram: conexão1083,396/1083,367/1100,003ms; passo p953,370/3,468/3,323ms;
cliente lento encerrou em3029/3031/3030ms. A terceira compôs/capturou por GPU o
cliente real:1000atores e1005primitivas960×640. Imagens foram inspecionadas.
Isso não declara corrigido o timeout anterior nem provaWAN, teclado físico ou FPS.

Implementação: conteúdo capturado uma vez, Arc de geometrias/bytes compartilhados;
codec/validação/JSON fora do mundo; boxes/projéteis pela identidade atual. Pacote
válido é publicado antes de render, sem Scene/template/política/recursos alheios.
Projeção de props10Hz e projectile latest têm custos proporcionais; snapshot muito
grande segue fallback full confiável1/s do backend, sem promessa de todos piores
casos de VFX a10Hz. Quota visual4 por slot de fonte primeiro e vagas restantes
é orçamento interno, nunca ID no wire nem culling de jogadores/colisão.

Renderer/Vertex/Globals/shader inalterados; frame/mesh caches reusados. Render de
mesh no cliente pode segurar brevemente o lock de sua réplica durante preparação
CPU, que não é o lock da autoridade; GPU/readback ocorrem depois de liberar a
réplica. Clones de câmera/luz e varredura de props mantidos; sem profiling/FPS
novo alegado. Stamp da janela inclui modo remoto para trocar também no estado
connecting quando tick/versão são0. Teclas locais ficam bloqueadas nessa vista;
input/ações remotos por MCP/API já funcionam, heartbeat/seleção/hotkeys e
interpolação/predição são unidades seguintes.

Artefatos: remote-{audit,report,stability,retention}.json e logs, network-thousand.png,
network-{host,remote,difference,frozen}.png e network-verification.json. Ajuste de
harness usou os campos escalares corretos de Grid; nenhuma asserção relaxada.
Release corrente24.526.960B e exemplos, sem abrir/instalar app ou tocar janela.
