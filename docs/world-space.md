# Mundo contínuo — primeira fundação espacial (EAF-162)

Esta unidade inicia a infraestrutura escolhida por Guilherme. Entrega representação e consultas de posição global; não carrega/descarrega terreno, não move atores e não substitui a física existente. As frentes EAF-159/160 integram circulação/apresentação; EAF-161 implementa save portátil.

## Inventário e arquitetura

| Área atual | Acoplamento observado | Migração necessária |
|---|---|---|
| `model::Scene`, `templates`, `targeting::Catalog` | Lista global limitada de entidades/atores e reserva por template | Mundo lógico com identidade persistente independente da presença; limites por conjunto residente |
| `identity::Registry`, `runtime_world::placements` | Referência slot/geração/InstanceId dentro de execução; placements ligados ao catálogo | Separar identidade lógica durável e handle residente; descarregar não equivale a morrer/despawnar |
| `Transform`, `physics::handles`, Rapier, KCC | Posição `[f32;3]`, um conjunto físico; validadores locais até +/-10000 | Posição global canônica e origens locais, incluindo múltiplas regiões físicas distantes simultâneas |
| `mobility::Network`, `npc_navigation_generation`, ground plans | Grafo global 64/128, perfil congelado e caches do snapshot | Grafos locais por região + conexões/planejamento hierárquico, regiões ausentes distintas de inacessíveis |
| `combat_spatial`, percepção/áreas/projéteis/ruídos | Um snapshot e catálogo residente, origem histórica por referência | Consultas/entregas entre regiões adjacentes sem conceder conhecimento oculto; retenção de regiões/origens ativas |
| `Engine`, inputs, replay, controles | Relógio/admissão de um mundo residente e reset autoral | Aplicação de mudanças de residência em fase determinística, posse/relógios e replay dos eventos relevantes |
| `network::Hub`, réplica/conteúdo | Uma projeção pública de atores/conteúdo | Endereços globais, múltiplos conjuntos relevantes, versão e transição atômica por conexão; manter caso denso1000 |
| `meshes`, plantas, materiais, render/frame/editor | Recursos e transforms preparados do documento | Recursos compartilhados/referenciados por região; render relativo à câmera/origem sem clones de todo o mundo |
| `checkpoint`, físico nativo, save futuro | Snapshot físico de execução; contato/estado por plataforma | Save canônico de mundo lógico e regiões, seed/versão gerador/deltas; caches físicos opcionais |

`Scene` continua útil como documento de composição, template ou fase proposital. Não deve ser sinônimo obrigatório de todo o mundo lógico. Mundo aberto também precisa funcionar offline; regiões não servem para remover participantes de um mesmo combate que exige escala densa.

## Decisão de representação

Usar célula XYZ inteira `i64` e deslocamento local `f64` canônico em `[0,1024)` metros. A célula de endereço tem **1024m**, potência de dois estável; não é tamanho obrigatório de chunk, grafo, bioma, área carregada ou ilha de física. Esses domínios podem agregar/subdividir células conforme seus próprios contratos.

Alternativas consideradas:

- Float absoluto: simples, mas precisão absoluta depende da distância à origem; migrar tudo para f64 ainda deixa o custo/contrato da física/render e coordenadas gigantes misturados.
- Uma origem global móvel: atende um observador, mas não resolve por si só jogadores simultaneamente em regiões muito distantes.
- Endereço inteiro + fração local: aritmética/carry explícitos e mais bytes por posição, com precisão local independente da distância; permite origens físicas/render distintas. Escolhida para esta fundação.

A posição tem 48 bytes de dados (3 i64 + 3 f64), não possui alocação. Inteiros são finitos e overflow gera erro; não se promete infinito matemático. Não se converte `cell*1024 + offset` para float global. Normalização usa intermediários i128; deslocamento é decomposto antes de somar a fração, e diferenças de células são calculadas em inteiro antes da projeção.

Versão própria `world_space::VERSION=1`. JSON/MCP usa strings decimais canônicas para células (`"9007199254740993"`), evitando arredondamento em clientes JavaScript. CBOR serializa i64, sem strings no binário. Offset é número f64. Deserialização de `Position` recusa offset não canônico; normalizar entrada bruta é operação explícita. A precisão da entrada f64 e arredondamento na fração local são limites reais, incluindo deslocamentos menores que o ULP local.

## Consulta pública

`query_world_space` recebe `query` com uma das operações:

- `normalize`: `point {cell:["0","0","0"], offset:[1024.25,-0.5,0]}` resolve carry positivo/negativo.
- `translate`: `point` canônico e `delta:[f64;3]`, sem perder a fração por somar deslocamento grande a um float global.
- `project`: `point`, `origin`, `radius_meters` e `max_error_meters`; devolve deslocamento local f32 e origem explícita.

Projeção admite raio `(0,1048576]m` e erro absoluto **por componente** `[0,1]m`; raio/tolerância são escolhas do consumidor. Recusa posição fora da janela ou quantização f32 acima do erro pedido. Esse teto é da consulta/projeção local, não do mundo. Projeção não muda a origem do Rapier e não rebaseia/renderiza/teletransporta nada. Reconstrução inversa de uma projeção segue `origin.translated(local)` com a perda f32 já explicitada.

Nenhum estado novo é guardado no Engine/Scene por essas consultas. Scene/replay/protocolo/checkpoint não sobem de versão nesta unidade. O adapter do save portátil deve incluir a versão do espaço no seu próprio envelope/manifest e usar `Position` como dado próprio, sem reduzir para float absoluto.

## Sequência de integração seguinte

1. Manifest do mundo: identificador/seed/versão de geração e regiões com estados ausente/solicitada/preparada/residente/retirando; publicar residência só quando física/nav/atores estiverem prontos.
2. Identidade lógica independente de handles: unload conserva indivíduo/vida/estado; novo nascimento e morte continuam operações distintas. Lease de região enquanto houver controle, projétil, canal, origem/efeito, contato ou dependência que precise simulação real.
3. Carregar duas regiões adjacentes e atravessar sua fronteira no mesmo runtime; migrar origem/referências/caches sem reset nem duplicar spawn/recursos. Depois descarregar/recarregar e provar estado/identidade.
4. Navegação por regiões/portais + caminhos locais, geração incremental sob orçamento e retomada quando destino ainda não carregado. Roaming escolhe destino, não revela alvo ou mapa desconhecido.
5. Política explícita de NPC fora de região: suspenso ou simulação simplificada com semântica de relógios/efeitos/viagem; atividade integral quando necessária. Não dizer que um ator viajou fisicamente se só avançou agenda.
6. Várias origens locais/observadores e regiões distantes em multiplayer; relevância não reduz artificialmente o caso1000no mesmo combate. Recursos, transporte, replay e save versionados conjuntamente.

Essa sequência é plano de migração, não capacidades já implementadas. EAF-162 fecha só a fundação; mundo contínuo só será alegado após travessia real e unload/reload com estado preservado.

## Checklist estático antes do código

1. Endereços/queries Copy e arrays de tamanho3; nenhuma varredura, cache ou alocação na aritmética.
2. i128 apenas para carry/diferenças; evitar overflow i64 e float absoluto. Offset f64, projeção f32 explícita.
3. JSON/string só na borda de autoria/MCP; CBOR usa inteiros. Não alterar ABI do solver/tick existente.
4. Consulta stateless O(3), sem locks/workers/GPU/I/O/rede obrigatórios.
5. Limites de domínio, finitude, raio e erro verificados antes de resultado; deserialização valida canonicidade.
6. Sem formato persistido de cena novo ou migração silenciosa de posições existentes.
7. Uma prova crítica para perda/overflow/codificação e engine preservado; sem testes por cada campo.
8. Preservar arquivos/formatos/build/artifacts reservados às frentes159/160; não formatar arquivos inteiros concorrentes.
9. Check/Clippy focados/all-targets conforme risco da integração, cenário MCP próprio sem janela e sem interferir na demo.
10. Custos mantidos: posição48bytes e conversões f64/f32; nenhum ganho de FPS alegado.


## Revisão depois do código

- `Position` mantém invariantes em construtores/deserialização; `Coordinates` é entrada bruta explícita. Campo privado evita mutar um endereço canônico sem validar.
- Aritmética Copy/O(3), arrays fixos, sem heap/scene clone por operação. i128 absorve diferença entre células extremas; nenhuma conversão de endereço absoluto para f32/f64.
- Deslocamento grande decomposto antes de somar offset, preservando a fração local. Overflow/NaN/inf e projeção fora de janela/erro são recusados antes de publicar resultado.
- JSON decimal canônico recusa números inteiros inseguros/aliases; CBOR guarda i64. Nenhuma string na aritmética; alocação só na borda JSON.
- Consultas são stateless; não alteram autoria, controles, replay, relógio ou simulação. O Clippy conjunto encontrou um `Ok(...?)` redundante no braço de despacho, simplificado sem mudar semântica.
- Custos mantidos: endereço48bytes, f64 local e strings na interface humana. Não há ganho de FPS alegado, nova dependência, worker, lock, GPU ou solver.
- Uma prova crítica Rust passou com oráculo inteiro independente em células extremas/negativas, CBOR/JSON, deslocamento grande preservando fração, underflow negativo, raio/erro de projeção e consulta de engine sem mutação. Evidência `artifacts/world-space-tests.log`. O cenário MCP próprio também passou sem janela/GPU, sem escrever no workspace compartilhado de regressão.


## Evidência final da unidade

- Uma prova crítica em `cargo test --locked --release --test world_space` aprovada; log `artifacts/world-space-tests.log`.
- `python3 scripts/verify_world_space.py`: 2 grupos aprovados, build `e2bfaa370d56373b`, sem janela; relatório `artifacts/world-space-verification.json` e log `artifacts/world-space-mcp.log`.
- Kernel compilado isoladamente com metadata; Clippy conjunto aprovado após simplificar o braço da consulta (`artifacts/sculpt-x/clippy.log`), fmt dos arquivos próprios, Python e diff aprovados. Não se repete a suíte geral por uma consulta sem estado.
- O exemplo MCP usa células acima de 2^53, cruza fronteira positiva/negativa, projeta relativamente e confirma snapshot do jogo inalterado após consultas válidas e erros. A prova Rust cobre também i64 MIN/MAX e CBOR inteiro.
- Nenhuma migração de Scene/replay/solver por esta unidade; versões observadas no checkout podem avançar pelas outras frentes. Nenhum rebase, streaming, região residente ou NPC em viagem global é alegado.
- Próxima integração é manifest/residência + identidade lógica e travessia de duas regiões, antes de prometer unload/reload de gameplay. Save portátil consome este endereço canônico em seu contrato próprio.


## Continuidade EAF-163

Manifesto e planejamento puro de residência/interesses/histerese/retenções implementados em [world-residency.md](world-residency.md), com regiões que subdividem ou agregam células de endereço. Aplicação física, identidade lógica fora dos slots e travessia com unload/reload permanecem a integração seguinte; não confundir propostas com conteúdo já instalado.
