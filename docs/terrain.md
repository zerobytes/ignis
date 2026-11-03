# Topografia nativa — EAF-164

Atualização179: as ferramentas164/166 e o renderer otimizado165..178 já estão
na main e foram verificados juntos. A publicação nativa usa o mesmo caminho de
meshes/otimizações. [Integração e paisagem de referência](terrain-render-integration.md).
As notas de isolamento abaixo descrevem a fronteira histórica da entrega164;
colisão/nav, residência real e costura continuam pendentes.

Direção31/10: ferramentas de terreno independentes do gerador procedural. Mapas fixos usam amostras autoradas; mapas procedurais usam os mesmos operadores e podem ser congelados em amostras editáveis. Não importar a pesquisa gráfica nesta unidade.

## Fronteira com a pesquisa gráfica

Chat `nanites` (`01a17526-e37e-70e0-833a-60ee0e5c23eb`) consultado, incluindo as respostas31/10 e os documentos `graphics-roadmap.md`/`certificate-hierarchy-study.md` na worktree `codex/graphics-research`. Há compilador/hierarquia experimental e fallback da fonte. Na leitura inicial atmosfera e amostragem atmosférica ainda eram propostas; tangentes compactas não foram promovidas. A frente165 iniciou depois um shader atmosférico na main e foi interrompida pelo usuário para esclarecer a substituição da representação distante. Seus rascunhos são preservados e NÃO pertencem a esta unidade. Nenhum commit de pesquisa é importado aqui.

Terreno visível precisa de representação disponível, mesmo distante. Não confundir residência de apresentação com simulação: colisão/nav/NPCs não precisam ter o mesmo raio do desenho. A futura integração deve usar a MESMA atmosfera da imagem para determinar contribuição residual; ar limpo não tem corte finito implícito. Enquanto uma silhueta contribui, manter malha ou representação mais simples. Trocar pai por quatro filhos só depois que todos estiverem prontos; budget/latência não autorizam um buraco. Sem fallback pronto, o plano declara cobertura incompleta e não pode ser publicado como imagem completa. Reduzir névoa, voar, teleportar ou mudar câmera exige preparar cobertura antes de tornar a nova vista visível.

## Decisões e limites

Base inicial: campo de alturas, eixoY vertical, superfície triangular XZ. Autoria por amostras, elevação/depressão, nivelamento/rampas, ruído e canais são operações tipadas. Não é voxel/caverna/overhang: complementar com malhas/volumes futuramente, sem redefinir essas ferramentas. Água geográfica (nível/curso) é separada do renderer/fluido; escavar um canal não é calcular uma bacia hidrográfica ou simular água.

Documento próprio versionado persiste fonte e operações, separado da malha compilada. Alturas são relativas ao datumY global; XZ usa `world_space::Position`, sem converter posição global parafloat. Grade fixa tem domínio explícito; fora dela retorna ausência, nunca chão inventado. Campo constante+operações pode ser consultado em qualquer posição representável. Ruído usa coordenadas de lattice inteiras globais e seed do campo, nunca uma seed diferente por chunk. Operações locais só influenciam sua região.

Compilação é eventual, não porframe/tick. A malha nativa publicada usa as transações/undo/persistência/replay existentes; não implica colisão, navegação ou streaming. O arquivo `.ignis-terrain` conserva a autoria editável; a cena contém o resultado compilado. Não substituir o save de gameplay ou anexar manifesto fictício ao Engine.

## Checklist estático antes do código

1. Tipos e fronteiras: documento/edição/patch/compilação tipados; serde/CBOR apenas I/O e MCP, nenhum JSON no caminho de amostragem.
2. Memória: limites de amostras/operações/sondas antes de alocar; arrays densosf32, coordenadasf64 locais e células inteiras; nada proporcional à área total do mundo.
3. Reuso: validar documento uma vez por consulta/compilação; amostrar núcleo preparado; não clonar documento por vértice nem regenerar malha a cada frame.
4. Trabalho: budgets de pontos×operações/ruído/canais; edições sequenciais numa cópia candidata, uma revisão por lote; fonte antiga permanece válida se falhar.
5. Geometria: triangulação única para amostra, bake e malha; calcular erro contra fonte declarada, não confundir amostras com limite contínuo de um procedural ainda não amostrado.
6. Cobertura: prontidão declarada é entrada de consulta, não comprovação GPU; orçamento nunca remove fallback nem libera memória antes da confirmação. Sem culling por distância na engine por esta unidade.
7. GPU/I/O: compilação/upload só explícitos; zero novo shader/worker/lock/dependência, gravação atômica e decodificação limitada; material e câmera existentes.
8. Invalidação: versão/espaço/id/revisão/hash de payload; checksums não concedem autoridade. Compilado deriva da versão exata da fonte; nenhuma reutilização silenciosa de dados antigos.
9. Risco: prova crítica de domínio/bordas/formato/edição atômica/continuidade e publicação nativa; MCP sem janela. Verificação visual técnica separada de aceite artístico.
10. Integração posterior: física/nav de terreno, residência/transações e cobertura real até horizonte, backend atmosférico e Nanites são contratos ainda não implementados aqui.

## API e autoria

`query_terrain` recebe `query.kind`:

- `inspect`: valida e devolve documento/limites.
- `sample`: alturas e profundidade geográfica sob o nível de água, ou `null` fora de uma fonte fixa.
- `edit`: lote append/replace/remove/set_base/set_water/patch_samples; revisão esperada da entrada, validação a cada operação e uma revisão nova para o lote. A revisão é da entrada fornecida, não um lock/CAS entre processos ou uma sessão autoritativa de arquivo.
- `bake`: congela um patch em amostrasf32 ordinárias e limpa operadores, incrementando revisão. A fonte original do chamador continua intacta; isso não estima todo detalhe analítico entre amostras.
- `compile`: grade triangular, bounds da fonte amostrada, stride, erro contra a grade mais fina e SHA256 da fonte canônica. A assinatura é calculada em streamingCBOR, sem uma segunda cópia inteira do documento.

`save_terrain {path,document}` e `load_terrain {path}` usam `.ignis-terrain`; load só devolve autoria, não troca cena/mundo. Envelope `IGNISTER`8B + versãoLEu16 + tamanhoLEu64 + SHA25632B + CBOR tipado, máximo8MiB/profundidade64/EOF exato. Posição em CBOR é bloco48B LE (3i64+3f64 canônicos); no JSON continua mapa Position com células decimais em strings. O bloco evita ambiguidade do buffer de enums tagged do serde, que perde is_human_readable em mapas internos; nunca relaxar JSON para aceitar números inseguros. Vetores são limitados antes de aceitar alocações grandes; escrita pelo Service é temporária+fsync+replace, com resolução de caminhos confinada ao projeto. Não substitui `.ignis-portable` de gameplay.

`publish_terrain {publication}` compila e aplica PutAsset+PutInstance numa única transação autoral. Requer pausa e respeita revisão de cena, orçamento nativo/validação/undo/replay existentes. `frame_origin` é projeção visual explícita, não um rebase físico. `visual_only:true` e `source_embedded_in_scene:false`: guardar também o `.ignis-terrain` para continuar editando a fonte. Geometria compartilhada porArc, sem cópia profunda adicional na publicação. Não há nova versão Scene/replay desta unidade.

Base `constant` admite campo de extensão arbitrária representável; `samples` define mapa fixo finito. Alturas são relativas ao datumY. Operadores locais têm posições globais; raise aceita sinal negativo, level inclui inclinação de plano, noise aceita smooth/ridged, channel usa polilinha explícita com leito descendente e só escava, clamp limita altura. No canal, width_meters é o alcance lateral a partir do eixo (largura total2×). A própria receita é composição dessas ferramentas; não há um caminho separado menos editável para o procedural. Ainda não há erosão, extração automática de bacias, rios globais, clima ou biomas.

Limites por consulta:64operações,64pontos por canal,66049amostras e16milhões de unidades de trabalho estimadas. Patch tem1..256células por eixo e espaçamento0,125..4096m; o bake pode exceder um asset nativo, enquanto compile/publicação respeitam16384vértices/32768triângulos e coordenadas±10000m. Dividir representação não limita a extensão do campo lógico. Sem aumento silencioso dos limites globais existentes.

Stride é potência de dois e divide o patch. A triangulação regular aninhada usa a mesma diagonal da interpolação. O erro reportado se refere à superfície amostrada: não é garantia de fidelidade contra ruído contínuo entre amostras ou o compilador Nanites. LODs com strides diferentes exigem costura explícita; o compilador marca isso e não realiza troca automática. A demonstração usa stride1 em ambos os lados e alturas idênticas na borda. Normais suaves ainda são derivadas por asset; transição de normais entre patches não é certificada aqui.

## Cobertura preparada para integração

`plan_terrain_coverage` valida uma floresta de até4096nós/profundidade16. Cada pai tem quatro quadrantes XZ exatos, origem global e dimensão potência de dois; duplicatas, ciclos, filho ausente, footprint incorreto e bounds não contidos recusam. Prontidão de recursos e de costura vêm declaradas do produtor e deverão ser extraídas/revalidadas pelo adapter real. Troca por filhos é atômica; orçamento insuficiente mantém pai. Se raízes já exigem mais que o budget, mantém cobertura disponível e informa excedente; não apaga fallback para fingir cumprimento do budget.

Atmosfera homogênea opcional neste PLANO: `T=exp(-extinction_per_meter*d)`. Horizonte negligível é `ln(maximum_contrast/negligible_contrast)/extinction`; usar distância mínima conservadora da esfera, não seu centro. Densidade zero ou disabled não tem horizonte finito. Densidade e limites de contraste/qualidade são parâmetros distintos; o contraste máximo deve majorar a contribuição real do material/iluminação no backend. Uma exponencial não chega exatamente a zero: o limiar é uma tolerância explícita, não invisibilidade matemática. Não omitir uma montanha ainda visível só porque seus detalhes já sumiram.

`proposals_only:true`, `renderer_binding_required:true`: cobre apenas raízes fornecidas, não prova que todo horizonte foi enumerado nem que GPU/streaming estão prontos. Sem fallback, `uncovered_roots` impede afirmar cobertura completa. O planner não muda render/física/nav e não descarrega recursos; omissão só poderá ser aplicada quando a política corresponder à imagem efetiva, incluindo efeitos em sombra/reflexo/disoclusão. Não existe corte de terreno por distância adicionado à engine nesta unidade.

## Reprodução

`cargo test --locked --release --test terrain --test world_space --test world_regions` verifica riscos críticos. `python3 scripts/verify_terrain.py` usa o binário atual porMCP, sem janela, guarda fonte editável em `examples/terrain-tools.ignis-terrain` e gera cena atual em `artifacts/terrain-tools.scene.json` a partir de `examples/terrain-tools.recipe.json`. A cena derivada fica no artefato para não atrelar a autoria de terreno1 a reservas de Scene da frente165. Captura `artifacts/terrain-tools.png`; relevo e plano estático no nível d'água, sem fluido/colisão/playtest inferidos.

## Checklist estático após implementação

1. Frequência: `Sampler::new` valida/prepara uma vez, `Sampler::sample` é API Rust tipada sem JSON. Nada novo em tick/frame. Compilar/publicar são operações explícitas.
2. Alocações: arrays de alturas densos, capacidade conhecida reservada antes dos loops; posição binária48B na stack. Malha compartilhada porArc, sem clone profundo ao publicar. Documento candidato pertence à consulta; erro não altera fonte no cliente/arquivo/cena.
3. Cálculos: lattice inteiroi128 deriva de célula+offset, hashwrapping64 por ponto de lattice; sem floatglobal. Distâncias locais descartam centros remotos antes de converter. Segmentos/prefixos de canais calculados uma vez porSampler; não por vértice.
4. Repetição: cálculo de alturas só no bake; compile reutiliza grade para geometria/bounds/erro. Hash da fonte por streamingCBOR, sem vetor extra de payload. Há custoO(amostras×operações/octaves/segmentos), contabilizado no preflight; não prometemos consulta constante com64canais complexos.
5. Limites: budgets de amostras/operações/segmentos e trabalho antes de gerar. Planner limitado4096nós/profundidade16, ordem de IDs estável; varre árvore fornecida, nunca mundo inteiro. `Sampler` público pode ser usado por hosts com seu próprio orçamento, enquanto a bordaMCP aplica o preflight.
6. Layout e precisão: camposf64/célulasi64 próprios, altura finalf32 e triângulosu32 compatíveis com a malha nativa. Datum e canto final conferidos; overflow recusa. Reprodutibilidade/igualdade de borda demonstradas no Mac desta unidade, sem promessa bitexact entre compiladores/plataformas.
7. GPU/I/O: nenhum shader/upload/readback novo no caminho estável; publicação usa cache/renderer existente. CBOR/arquivo/JSON só na fronteira explícita, leitura8MiB, hash/EOF/profundidade/arrays validados antes de usar.
8. Concorrência: sem worker/lock novo; arquivo substituído atomicamente peloService. Revisão de documento não vira CAS de arquivos entre processos. Nenhum job de streaming ou reserva de memória é inventado pela consulta.
9. Correção: persistência de posições dentro de variantes falhou no primeiroMCP e foi corrigida no codec dedicado; a mesma prova crítica foi ampliada para essas variantes e grid fixo. Flags de prontidão não são prova física; partição exata é validada e integração real continua explícita.
10. Custos/limites preservados: fonte e compilado podem coexistir durante edição, PNG/replay só nas evidências, limites nativos de mesh mantidos. Sem ganhosFPS, física de terreno, rios por bacias, sistema de água ou streaming/ausência de buracos na imagem real alegados.

## Evidência desta unidade

5testesRust em3alvos passaram (3críticos de terreno, mais as provas existentes de espaço global e residência). Clippy release/all-targets-Dwarnings, build do binário, fmt, Python e diff aprovados. `verify_terrain.py` passou3gruposMCP no build `438a821b7fdbdb8f`, sem janela/áudio, capturando porGPU offscreen. Fontes/arquivo/editável/erro tardio/bordas/coverage/undo/load/replay conferidos; 65amostras da borda de dois patches idênticas e cada malha4225vértices/8192triângulos. PNGs do estado inicial, replaytick2 e recarga exatos, imagem técnica inspecionada; não é aceite artístico, gameplay caminhável ou física de água.

Relatório `artifacts/terrain-verification.json`, logs `terrain-tests.log`, `terrain-clippy.log`, `terrain-build.log` e `terrain-mcp.log`. O primeiroMCP detectou e recusou o codec posicional incorreto; só os resultados da repetição após correção sustentam esta conclusão. Não houve relaxamento de JSON ou testes. O executável foi compilado do checkout conjunto com drafts165 desligados; estes não entram no commit164 e não são validados como entrega atmosférica por esta prova. Único app corrente `target/release/ignis`; cachedebug criado por engano nesta tarefa foi cancelado e removido após comprovar ownership/ignored/ausência de uso (122019089B), relatório `terrain-cleanup.json`. Fonte do relevo em formato1 independente das versões reservadas pela frente gráfica.
