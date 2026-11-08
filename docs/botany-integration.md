# Plantas completas: autoria e dinâmica nativas

## English walkthrough: native procedural botany

Shortest tested path (rev EAF-194, macOS):

- Build: `cargo build --release`
- Open/inspect: load `gallery.scene.json` in the native renderer, open the Vegetação panel.
- Contract/schema: see [botany-architecture.md](botany-architecture.md).
- Included: sources/recipes + `gallery.scene.json`. Generated locally: `artifacts/botany-native/*` (report.json, verification.json, PNGs).
- Boundaries: native authoring is not universal botany, calibrated growth, or production gameplay physics.
- Example: the isolated plátano leaf captured in the gallery.

EAF-194 acrescenta [forquilhas terminais e copas volumétricas](botany-architecture.md), fonte/receita4 com leitor1..4, preservando os contratos nativos descritos abaixo.

EAF-193 acrescenta [idade cronológica por perfil](botany-age.md), fonte/receita3
com leitura1/2 e exemplares de décadas. As evidências abaixo registram as entregas
anteriores; fontes correntes e comparativos de idade estão no documento novo.

A continuação EAF-192 amplia a curva de repouso e revisa os quatro exemplares
por fotografias: [deformação e referências](botany-reference-refinement.md).
Fonte/receita2 leem1; Scene109/replay119 correntes são preservados.
Os números e capturas abaixo registram a primeira entrega EAF-191.

EAF-191. Guilherme autorizou consolidar integração, grandes deformações/contato
e autoria no Ignis, com palmeirinha, plátano, eucalipto e figueira ornamental
de copa ampla e raízes aparentes. A matemática anterior permanece em
[procedural-botany-v2.md](procedural-botany-v2.md).

## Contratos e revisão anterior ao código

- Fonte construtiva tipada: grafo de órgãos, curvas orientadas, seções e contornos,
  não uma coleção de triângulos. As quatro fontes são preparadas a partir das
  receitas matemáticas; o compilador e os controles integrados são Rust e não
  precisam executar Python. A fonte é incorporada à receita da cena para permitir
  regeneração sem depender do arquivo externo.
- Receita/bake entram na mesma transação. Prévia é estado de sessão, com revisão
  de origem; cancelar não muda Scene/undo/simulação. Seleção e ajustes por órgão
  usam as mesmas operações MCP/editor. Exportação/importação tem versão própria.
- Preservação usa endereço material estável (órgão e coordenadas paramétricas).
  Deltas de escultura são transportados pelo frame material. Alteração de topologia
  manual, rig/clipes desconhecidos ou perda de um ponto ajustado recusam a
  regeneração preservada; descarte exige política explícita. Não inferir
  correspondência por proximidade. Fonte autoral e bake continuam separados.
- Grandes deformações usam uma rede material tridimensional com restrições XPBD,
  sem limite angular do modelo reduzido V2. Seções, vínculos e velocidades são
  acompanhados por identidade. Crescimento atualiza comprimentos/massa e conserva
  momento dos pontos sobreviventes sob a convenção declarada de massa nova.
  Poda contabiliza massa e momento removidos. Trabalho de crescimento e reações
  externas não são apresentados como sistema mecanicamente fechado.
- Contato usa proxies de segmentos/seções e chão/obstáculos
  explícitos, separados de colisão exata da superfície renderizada e do gameplay
  Rapier. Não manter um colisor autoral estático como se acompanhasse deformação.
- Novos contratos autorais exigem Scene/replay coerentes. Conteúdo público e
  persistências separadas só mudam se seus layouts efetivamente mudarem. Trabalho
  sujo de personagens será preservado; novos módulos de botânica e pontos mínimos
  de conexão, sem reformatar arquivos de outra frente.

Checklist de performance prévio: fonte em Arc e controles pequenos; compilação
somente em alteração; contagens/quota antes de alocar; frames e correspondências
preparados uma vez, sem JSON/arquivo por frame; broadphase de contato determinista
com limite de trabalho e sem descarte silencioso; buffers/caches limitados;
nenhuma dependência/worker nova; operações compartilhadas pelo Service; logs e
erros conservados; saídas substituem caminhos estáveis e somente um build corrente.
Testes novos focados em integridade, identidade, atomicidade, persistência e
integração física material, com capturas nativas inspecionadas separadamente.

## Modelo executável

`src/botany.rs` define fonte1/receita1, em SI e +Y para cima. O grafo é
ordenado por ascendência; cada órgão tem identidade, curva de controle, seção,
inserção no pai, normal material, nascimento/duração e material efetivo. A fonte
contém curvas/seções, nunca triângulos. Os padrões de multiplicação e campos V2
continuam no laboratório; `scripts/botany_exemplars.py` prepara as quatro fontes.
A geração da malha, edição e dinâmica integradas não executam Python. Esta
primeira ponte não porta toda a gramática ambiental/vascular V2 para widgets.

A curva usa interpolação cúbica Hermite/Catmull com derivadas dos vizinhos. A
normal é transportada pela rotação mínima entre tangentes, mais a torção por
comprimento. Alterar o pai transporta a curva descendente por
`R = [T N B]editado [T N B]originalᵀ`. Comprimento, largura, raio, deslocamento e
rotação por órgão ficam na receita; poda elimina também os descendentes. A
máscara de densidade usa hash de seed/identidade, sem renumerar sobreviventes.

Para lâminas estreitas, a meia largura é proporcional a
`u^a(1-u)^c / max(u^a(1-u)^c)`. Para palmadas, raios nos ângulos de pontas e vales
são interpolados por uma transição assimétrica de potências; quantidade,
distribuição, comprimentos, recortes e arredondamentos pertencem à fonte. O
normalizador de largura é preparado uma vez por lâmina. Curvatura, dobra e
concavidade usam o frame da curva. Duas faces separadas pela espessura são
fechadas nas bordas, com centro/pontas/costura compartilhados. A fonte do plátano
usa cinco pontas; furos e grafos de nervuras completos permanecem na referência
V2, sem fingir que esta malha nativa já os compila.

Junções lenhosas cortam janelas no eixo pai e na base do filho. Três colares
circulares reutilizam os índices de suas bordas no kernel `mesh_organic`; a
superfície final tem duas incidências opostas por aresta. Tentativas crescentes de
comprimento de transição exigem planos separados e janelas sem sobreposição.
Se não couber, a receita é recusada com a indicação do órgão; não há fallback
silencioso para tubos atravessados. Seções elípticas são aceitas em eixos isolados;
esta junção exige seções circulares e raio resolvido pelo kernel (mínimo1mm).
A malha madura é fechada; isso não certifica ausência global de interseções entre
órgãos que a fonte posicionar no mesmo espaço. Raízes aéreas da figueira são
filhos verdadeiros dos ramos, com o mesmo vínculo e solver.

## Crescimento, força e contato

A sessão nativa usa quatro pontos materiais por seção,12 intervalos de eixo por
padrão e quatro por lâmina. Ligações longitudinais, transversais e diagonais
respondem a alongamento/cisalhamento/flexão/torção sem ângulo máximo imposto.
É uma redução volumétrica efetiva, não um elemento finito calibrado de madeira
anisotrópica ou tecido vascular. Young/shear são materiais homogenizados; a área
lenhosa é elíptica e a área foliar é uma quadratura aproximada de envelope.
Contorno palmado, nervuras e escultura não redefinem uma massa volumétrica exata.

A fração de crescimento é `g=smoothstep(clamp((idade-nascimento)/duração))`;
comprimentos/seções de repouso crescem com `g` e a massa com `g³`. É uma política
homotética por órgão, distinta da revelação longitudinal do laboratório V2.
Identidades e velocidades existentes sobrevivem à alteração de repouso; correções
de inserção podem deslocar posições, e o solver resolve o trabalho de crescimento. Para massa nova entrando com momento zero,
`v_novo = m_antigo v_antigo / m_novo`. Pontos recém-nascidos começam no repouso.
Poda retira a subárvore e acumula a massa e o momento retirados, conservando os
endereços dos outros pontos. Não é destacamento físico de um galho cortado.

Para uma ligação, `C=|xa-xb|-l0`, `α=compliance/dt²` e
`Δλ=-(C+αλ)/(wa+wb+α)`; correções são ponderadas por massas inversas. Rigidez
efetiva deriva de `EA/L` ou `GA/L`. Uma barreira de volume assinado das células
combate espelhamento/inversão durante dobramentos; convergência continua finita,
não é uma prova global de injetividade. Os quatro pontos da inserção do filho
se vinculam a combinações materiais das seções vizinhas do pai, com reação nas
mesmas partículas do pai. A base usa a inversa completa do frame material
interpolado, inclusive quando não é ortogonal em uma curva; uma projeção final
resolve `(J M⁻¹ Jᵀ)λ=Jx` por gradientes conjugados pré-condicionados, sem matriz
densa (até512iterações), e fecha os vínculos até tolerância1e-4m ou recusa a candidata. Frames de repouso e atuais transportam a malha sem
salto inicial ao iniciar o ensaio. Manual offsets usam o mesmo transporte.

Passo fixo1/120s; animação acoplada ao relógio local executa dois subpassos por
tick1/60s do Ignis. Gravidade e arrasto de vento efetivo precedem XPBD;
amortecimento exponencial segue a projeção. O relatório separa impulso de
forças, impulso agregado do solver/apoios/contato/amortecimento e momento
retirado. Esse balanço detecta perda de movimento; não apresenta o conjunto
com chão, âncoras, dissipação e crescimento como sistema isolado.

Contato opcional usa cápsulas com raios de seção em repouso e distância entre segmentos,
com grade espacial determinista. Chão projeta pontos materiais; não é teste exato
dos triângulos da lâmina. Vizinhos locais do mesmo eixo e o domínio imediato de
inserção são excluídos. Lâminas podem participar por cápsulas largas, com falsos
contatos esperados; não há tecido auto-colidente de alta resolução ou CCD.
`max_proxy_overlap_m` informa o resíduo na configuração atual quando
`contact_probe_complete=true`; consulta desabilitada ou interrompida por quota
não certifica separação, e `contact_probe_error` expõe o erro. Não generalizar
os cenários demonstrados para ausência de atravessamento em qualquer velocidade.

A dinâmica é **ensaio autoral local**, fora de Scene/replay, save portátil e
replicação. `Reset` retorna ao bake; `Prune` da sessão não altera a receita.
Para persistir a poda, publique `Recipe.pruned` (ou use o botão de poda na fonte).
Um colisor Rapier estático habilitado nas peças impede o ensaio: não há collider
de gameplay acompanhando esta deformação. Host/cliente de rede recusam a sessão.

## Autoria compartilhada

MCP: `{"op":"botany","operation":{"kind":...}}`. Operações:
`inspect`, `open`, `load`, `save`, `preview`, `cancel_preview`, `publish`,
`select_organ`, `edit_organ`, `physics`. `physics.action.kind` aceita
`start`, `step`, `play`, `grow`, `prune`, `inspect`, `reset`.
O painel **Vegetação** chama exatamente essas operações. Permite carregar os
quatro exemplares, ajustar a planta, visualizar/aplicar/cancelar, escolher órgãos,
ajustar transformação/dimensões, podar, exportar e executar o ensaio.
Seleção nesta versão é pelo identificador no painel/API; não há gizmo botânico
ou seleção individual de órgão por clique no viewport.

`.ignis-botany` é JSON tipado versionado, com leitura limitada32MiB e escrita
atômica no escopo do projeto. Fonte incorporada na cena permite regeneração
sem o arquivo original. Scene108/replay118; conteúdo7/protocolo5/checkpoint3/PSV1
não tiveram layouts próprios alterados. Replay/portátil conservam a receita e o
bake autoral, não a sessão XPBD. Envelopes de replay/rede continuam independentes
do tamanho de uma cena local; a galeria inteira não foi declarada replayável
no envelope8MiB.

`Preserve` compila o bake anterior, captura deltas manuais em frames materiais e
reaplica por endereço `órgão/u/coordenada transversal`. Mudanças de discretização,
poda ou domínio de junção que perderiam um endereço ajustado são recusadas.
Topologia, material, rig/clipes e transformação manual da instância desconhecidos
também recusam transporte. `Reject` recusa escultura; `Discard` descarta de forma
explícita. Offset é em metros materiais, sem escalar automaticamente seu tamanho.
Mudanças de curva/frame transportam sua orientação. Fonte/bake/camada manual
compartilham um único undo. Prévia não altera Scene, física ou histórico.

A base de uma prévia/sessão acompanha as peças e a receita: câmera/luz e edição
alheia atualizam o documento derivado sem destruir o ensaio; editar sua geometria
invalida a sessão. Carregar outro documento ou iniciar rede limpa a sessão.
Não há JSON/arquivo/bake por frame estável; o overlay usa a malha atual da cena.

## Limites e reprodução

Por fonte:8192órgãos,2..33controles por curva; por bake:1M vértices/2Mtriângulos.
Até64receitas por cena. Dinâmica:65536pontos,1Mrestrições,100kpares e1Mentradas
na grade. Excesso/divergência recusa a candidata e conserva o estado anterior;
não há poda silenciosa para caber. Esses são orçamentos deste modelador, não
limites universais do mundo/render/simulação do Ignis.

```
python3 scripts/botany_exemplars.py
cargo test --offline --test botany
cargo build --offline --release --bin ignis
python3 scripts/demo_botany_native.py
python3 scripts/verify_botany_native.py
```

Fontes em `assets/botany/`; saídas substituídas em `artifacts/botany-native/`.
Galeria salva separadamente do mundo aprovado e do personagem corrente. Capturas
são do renderer/editor nativo offscreen, não imagens geradas por IA. Não abrir ou
substituir a cena da janela humana para reproduzir estes roteiros.

A verificação de save portátil com cena exclusivamente visual encontrou uma
guarda histórica de identidade que exigia uma entidade mesmo quando o construtor
reserva um único slot vazio. A leitura agora aceita exatamente essa reserva para
zero entidades; qualquer ocupante continua inválido. A prova de integração cobre
fonte/bake botânicos e a ausência de atores após restaurar.

## Evidências desta entrega

macOS arm64/Metal, build `587d93bed378d69b`. Nove provas novas em
`tests/botany.rs`: fonte/bake/undo/persistência e rejeição atômica de perda manual;
fontes reproduzíveis e fechamento orientado de madeira/lâminas; massa/escala/
espessura; identidade/momento em crescimento/poda; preview/câmera/invalidação;
portátil sem atores; grandes dobramentos/chão; separação de ramos; quota de
contato com erro explícito e estado preservado. Mais16provas existentes em editor,
mesh_objects, npc_checkpoint, portable_save e identity passaram conforme impacto.
Check e Clippy dos alvos relacionados, rustfmt dos módulos novos, Python e diff
aprovados; a suíte completa não é inferida desses recortes.

A figueira completa foi exercitada com39152pontos/179956restrições. Crescimento
1,8→3,4, com128subpassos e24iterações XPBD, conservou identidade e atingiu
resíduo de vínculo9,164e-9m. Massa17,6456→65,8452kg sob a aproximação declarada;
poda de ramo0 retirou6,95047kg e o momento correspondente. São massas do modelo,
sem calibração botânica. O tempo observado do roteiro crescimento/atualização
foi13,51s neste Mac compartilhado, não um benchmark isolado ou FPS.

Haste flexível:240subpassos, rotação máxima1,733rad (>limite0,65 da V2), contato
com chão e menorY dos pontos igual0. Dois ramos inicialmente cruzados: sobreposição
de proxies28,633mm→0 após120subpassos; consulta concluída e correções finitas.
São cenários de recuperação/contato discreto, não prova de CCD ou de separação
exata de todas as superfícies.

A primeira projeção local deixava folga de16,4cm em crescimento acelerado da
figueira. A projeção global por CG e a inversa completa do frame material
corrigiram esse risco; agora ausência de convergência recusa a candidata.
Verificador exige tolerância nas etapas, além de conferir geometria resultante.

Quatro fontes autorais/galeria e folha de plátano isolada foram capturadas e
inspecionadas no renderer nativo, junto ao painel Vegetação. Save/load da galeria
conservou os bytes dos PNGs. Fixture pequena passou pelo replay e save portátil
reais via MCP; cenas só visuais restauram sem inventar um ator. Recibos:
`artifacts/botany-native/report.json`, `verification.json`, `botany-tests.log`,
`identity-tests.log`, `clippy.log`, `check.log`, `gallery.scene.json` e PNGs.
O app humano permaneceu aberto na outra frente; não houve demonstração física
de mouse/teclado nesta janela, aceite humano, push ou produção.

## Independent generated materials (LOS-21)

Botanical regeneration under `Preserve` now transports validated procedural
materials by the existing wood/leaves asset identities. Preview, publication
and the physical overlay retain the same recipe/maps; legacy manual material,
rig, topology, instance and sculpt-domain guards remain in force. `Reject`
refuses a material override and explicit `Discard` restores the botanical
material. Botanical files still contain geometry primaries only: save material
libraries separately and use the scene/composition to retain their association.
No source, Scene, replay or GPU layout change and no new work per frame.

Validation: all20 botanical release tests passed, including fixed/runtime
materials through preview/cancel/reject/publish/save, physical step/reset and
Scene roundtrip; focused release Clippy and the current native build passed.
This verifies preservation, not an artistic result or performance guarantee.
