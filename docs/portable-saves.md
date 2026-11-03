# Save portátil — EAF-161

## Decisão antes do código

Checkpoint nativo continua sendo otimização de retomada no mesmo Rapier/OS/arquitetura.
O save portátil tem envelope e versão próprios, primárias físicas em tipos da Ignis e
estado de gameplay completo; não serializa handles, ilhas, broadphase, manifolds,
solver, joints ou arenas Rapier. O adapter reconstrói corpos/colisores a partir das
formas e materiais das definições validadas. Não executa ticks/história nem cobra
custos para restaurar. Construção e validação acontecem em candidato isolado.

Alternativas: tornar snapshot Rapier opcional no mesmo payload misturaria contratos;
exportar só transforms perderia identidade/efeitos/rotas. Escolhido um arquivo canônico
separado e migração explícita do checkpoint nativo compatível. Gameplay usa tipos
próprios existentes e IDs lógicos InstanceId/geração/vida preservados; slots são apenas
o layout residente validado desta versão. Identidade entre universos/streaming ainda
exige contrato futuro, sem transformar slot em identidade global.

Frame residente tem origem `world_space::Position` e versão espacial1 (EAF-162),
sem duplicar representação global. Neste runtime ainda finito a origem é zero;
primárias e alvos/rotas permanecem relativos a ela. Não há rebasing/streaming.
Extensões de regiões/seed/versão gerador/deltas serão versionadas: geradores residentes
já conservam sua receita e estado; nenhum manifest de regiões fictícias é emitido.

## Retomada física

Guardar pose, próxima pose cinemática, velocidades, força/torque persistentes,
gravidade, modo, CCD, enabled/sleeping e estado do controlador/mobilidade/voo. Construir
formas exatas sem snap/teleporte/avançar tempo. Apoio/KCC e contato são consultados
novamente na física seguinte; histórico de contatos/triggers/passos é estado da Ignis
preservado para não reemitir entrada/custo. Manifolds/warmstart são descartados.
Continuação física entre backends não é bitexact; validar erro de posição ao carregar
zero no backend local e retomada nas tolerâncias normais do controlador (alinhamento
de nó2mm). Essa tolerância não autoriza alterar pose no load nem prometer bound de
divergência temporal entre plataformas. Provas em Mac não equivalem a Windows/Linux.

## Checklist estático prévio

1. Captura/encode/decode/rebuild só sob comando pausado, fora do tick.
2. Cópia limitada das primárias e candidato; sem replay desde zero nem log adicional.
3. Tipos próprios e CBOR tipado, identidade/relógios exatos; sem ponteiros/ABI física.
4. Reusar validadores/catálogos próprios; caches reconstruídos com perfil congelado.
5.128MiB/profundidade128, integridade e EOF exato; custo de pico no load explícito.
6. Sem GPU/dispositivo/servidor/rede/I/O por tick; gravação temporária e fsync+replace.
7. Mesmo dispatcher; nenhuma thread/lock no núcleo de persistência.
8. Candidate antes de publicar; tokens renovados e input externo neutralizado.
9. Provas novas por perda/corrupção/identidade/duplicação/formato/replay, gates por impacto.
10. Cold start do solver é tradeoff explícito; sem FPS/load sem pausa/aceite universal.

## Formato e comandos

`IGNISPSV` + versão u16 little endian + comprimento u64 + SHA256 + CBOR tipado,
cabeçalho50bytes, teto128MiB e profundidade128. EOF exato. Hash detecta corrupção,
sem autenticar autoria; não oferecer importação irrestrita de arquivos hostis.
O payload inclui versão da cena/gameplay, versão espacial, frame residente, modo,
watermarks das admissões, ruídos recentes e primárias próprias. O sistema numérico
usa inteiros CBOR (não números JSON para IDs). Contatos sem slot usam Option<u16>,
nunca usize::MAX. Capacidade própria de origens é limitada1024 e índices derivados
não são persistidos. Definições de shapes/materiais são validadas antes do adapter.

- `save_portable_save {path}`: pausado e sem pedidos pendentes; extensão `.ignis-portable`.
- `load_portable_save {path,mode?}`: restaura pausado em candidato e publica de uma vez.
- `inspect_portable_save {path?}`: capacidades atuais ou validação completa de um arquivo sem publicar.
- `migrate_portable_save {source_path,path}`: checkpoint nativo compatível → portátil1,
  ou portátil1 validado → portátil1. Valida tudo antes de escrever e não muda o mundo.

Versões futuras/desconhecidas são recusadas explicitamente. Nesta primeira versão,
cena/gameplay antigos sem migrador e checkpoint nativo de outra física/OS/arquitetura
são recusados, nunca reinterpretados silenciosamente. Migração não remove guards do
checkpoint legado. `save_checkpoint/load_checkpoint` preservam contrato nativo.

O adapter atual usa Rapier como executor na plataforma de destino; ele não lê seus
internals do arquivo portátil. Recompõe colisores caixa/cunha, modos dinâmico/fixo/KCC,
corpo por referência/vida e políticas físicas já suportadas. Joints não possuem
contrato autoral atual e são recusados, evitando perdas silenciosas em extensões.
Não é um importador universal de qualquer mundo de qualquer backend.

Baseline pós-load em modo recorded é um checkpoint nativo **gerado no destino**, nunca um requisito
embutido no save portátil. Assim o replay seguinte é exato no backend local compatível
sem prometer uma mesma trajetória binária entre backends. Limite8MiB do replay é
preservado: um save128MiB não implica replay exportável. Em recorded a baseline
local também respeita128MiB; se exceder, o load recusa com contexto e pode ser feito
em continuous. `continuous` restaura diretamente sem serializar caches nativos e
não ganha log/baseline binária.

Host/cliente ativos recusam load; cliente também recusa save porque não possui estado
autoritativo. Tokens de controle/diálogo/world anteriores expiram e input fica neutro;
conversa/cast/efeitos internos continuam no mesmo tick. GPU/UI/sockets/threads/áudio
no dispositivo são externos; vozes/relógios de apresentação pertencem ao gameplay.
Erro de load preserva mundo/revisões/seleção; erro de migração preserva destino anterior.
Caminhos continuam no projeto e a escrita usa temporário no mesmo diretório,
fsync do arquivo e replace atômico do Service. Não alegar durabilidade de diretório
após perda de energia em todos os sistemas operacionais.

## Revisão posterior e prova crítica

1. Adequado: captura/encode/hash e restore só no comando pausado; zero trabalho novo por tick.
2. Corrigido: load direto das primárias, sem encode/decode nativo intermediário; continuous
   não cria baseline binária. Inspector evita parse duplicado. Cópia de primárias,
   validação em memória e caches/baseline de recorded continuam custos explícitos.
3. Corrigido: índices de contato Option<u16>/paresu16 substituem sentinela usize; IDs,
   gerações, vidas, recursos e ticks mantêm seus domínios inteiros.
4. Adequado: tipos/validadores próprios reutilizados, kernel do adapter para caixa/cunha;
   AABBs publicados sem tick para queries/IK no prefixo0 e navegação recomposta.
5. Adequado:128MiB/profundidade128/EOF/hash; construção isolada antes da publicação.
6. Corrigido: CCD é primária explícita, não inferida de Dynamic; híbridos criados como
   KCC mantêm a política original. Construtor recebe velocidades de todos os motores:
   setters Rapier ignoram KinematicPositionBased e perderiam sua velocidade medida.
   Sleeping é aplicado após anexar colisor. Nenhum snap nem tempo avançado.
7. Adequado: sem worker/lock/rede/GPU/áudio novos; scopes externos renovados.
8. Validado: erro de formato/identidade/checksum não publica; custo/recurso/cast/efeito/
   diálogo/objeto/memória/gerador/origem permanecem no mesmo instante.
9. Uma prova Rust nova protege corrupção/perda/duplicação/identidade/replay. Comparação
   EXATA de cada primária física antes de avançar detectou os bugs CCD/KCC acima;
   fence de campos de gameplay detecta omissões futuras do schema. Gate180ticks com
   solver frio mantém custos/saúde/cooldowns/execuções nesta receita e limita erroXYZ
   a2mm; observado0,000042945147m (~43µm). Esse resultado local não promete bound
   universal de divergência futura entre backends. Replay/continuações locais exatos.
10. Mantido com motivo: caches do solver são recompostos; plataformas/backends podem
    divergir fisicamente. Validação atual macOS arm64; Windows/Linux não executados.

Prova `cargo test --locked --release --test portable_save`: passou em30/10/2026.
Inclui cast/cura/DOT/buff/voo ativos, sessão/porta, renovação de tokens, migração nativa,
checksums/forja de primárias, rollback, origem retirada/reuso, continuous sem log,
100NPC+500Players e root motion/IK no instante carregado e na continuação20ticks.
A receita de NPCv1 deriva do gate nativo existente; novos checks visam a fronteira
canônica, não cobertura por flag. Log corrente: `artifacts/portable-save-test.log`.

Gate MCP `python3 scripts/run_verifications.py --only portable_save`: **1 cenário /
2 grupos aprovados**, build `3f31de512312c2ee`, macOS arm64. Snapshot inicial e PNG
prefixo0 exatos, replay posterior, migração com checksum/rollback/destino preservado,
guarda de host e continuous sem log. Relatórios correntes:
`artifacts/portable-save-suite-report.json` e `artifacts/portable-save-verification.json`.
Sem janela, áudio no dispositivo ou execução Windows/Linux. Check/Clippy do fonte
conjunto foram realizados pela integração; o gate final é registrado no fechamento.

EAF-163 fornece manifesto/planejador puro de residência (`docs/world-residency.md`),
sem estado ligado ao Engine. Ele não entra artificialmente no save. A versão espacial
continua sendo a fonte de endereços globais; anexar regiões ativas/deltas/identidade
lógica ao runtime e à persistência exige uma extensão versionada, não índices de
residência tratados como IDs eternos. As receitas/seed/estado dos geradores que já
existem neste runtime continuam incluídos nas primárias próprias.

## Fechamento sobre a base integrada

Dependência de apresentação consolidada em `95e72e0`: Scene99/replay109/protocolo4/
conteúdo1/checkpoint2. O formato portátil continua **PSV1**, independente do checkpoint.
Após a cena final dessa base, passaram **4 provas críticas / 4 alvos Rust**:
`portable_save`, `npc_checkpoint`, `npc_presentation_motion`, `npc_air_circulation`.
Clippy release/locked **all-targets -D warnings**, fmt global, py_compile dos scripts e
`git diff --check` passaram. Logs: `artifacts/portable-save-final-rust.log`,
`portable-save-final-clippy.log`, `portable-save-fmt.log`. O gate MCP próprio1/2 e
seu build3f31de512312c2ee permanecem registrados acima; não se infere runtime
Windows/Linux ou bitexact entre backends desses gates locais. Uma única saída
corrente em target/release/ignis, sem versões de build/instaladores paralelos criados.
