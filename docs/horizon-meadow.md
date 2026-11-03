# Horizon: laboratório de vegetação em 1 km²

EAF-215, continuação de EAF-210. **Em investigação; não é uma entrega de qualidade/FPS aprovada.** A cena original e todas as fontes/bakes botânicos continuam intactos em `artifacts/botanical-valley`.

## Primeiro recorte: campo instanciado

`examples/horizon_meadow.rs compile` cria a cena corrente em `artifacts/horizon-meadow/world.scene.json`, mantendo o terreno físico/visual original (1.000 × 1.000 m, amostras a cada 2 m), personagem, árvores e arbustos. O novo campo rasteiro cobre a região inteira com sítios a cada 25 cm e jitter determinístico. Caminho e água são exclusões explícitas; não há zonas de densidade feitas para a câmera.

O campo contém 16.000.000 sítios, dos quais 15.667.937 plantados: 344.694.614 lâminas lógicas do exemplar nativo de 22 folhas. Isso descreve a população fonte, **não a quantidade de lâminas desenhadas por frame**. A prova de ocupação por células de 50 m está em `compile.json`; não substitui a revisão visual próxima/aérea.

`IGFLD1` armazena alturas/máscaras em f32 little-endian (4 bytes por sítio), métrica da grade e seis malhas compiladas compartilhadas. Coordenadas/jitter são reconstruídos pelo mesmo hash inteiro do compilador. O pacote tem 64.185.944 bytes, aproximadamente 61,2 MiB. Não há instâncias JSON individuais, geração botânica ou upload das alturas por frame. A cena autoral ainda usa o formato existente de Scene.

O renderer percorre blocos espaciais de 4 m, testa o frustum, escolhe uma escala pela resolução física e envia até seis lotes. O shader lê alturas/templates residentes; raízes fora do campo são mascaradas. A física permanece na fonte completa. Sombras têm seleção própria no volume da luz. `Renderer::set_surface_field` é uma API diagnóstica, desligada por padrão. O opt-in de processo `IGNIS_HORIZON_FIELD` aplica o mesmo pacote à janela nativa e às capturas MCP. O pacote não é persistido por SaveScene nem se revincula automaticamente a outra cena.

Este primeiro compilador usa estações da malha botânica nativa para formar superfícies finas. O renderer recebe geometria, alturas e escalas; não testa nomes de espécies. O exemplar/receita específico pertence ao laboratório. Rigs, crescimento, vento, shaders gerais, formatos globais e IGL1/ICB não são habilitados por este trabalho.

## Medição inicial

Metal/M4 Pro, 1.100 × 760 pixels, câmera/luz congeladas, três aquecimentos e oito amostras. Mediana de `Renderer::draw` mais espera de conclusão. **Não é FPS de apresentação nem timestamp isolado da GPU.** A ablação sem campo remove apenas o campo novo e conserva árvores/arbustos/terreno; não é uma referência visual equivalente de vegetação completa.

| Vista | Nativos sem campo (ms) | Nativos + campo (ms) | Incremento (ms) |
|---|---:|---:|---:|
| Jogador | 123,643 | 131,553 | 7,911 |
| Vale | 83,606 | 88,073 | 4,467 |
| Panorama | 299,747 | 306,451 | 6,704 |

O campo envia 2,71–5,30 milhões de triângulos de câmera, mais 513.964 candidatos de sombra. A memória GPU própria é 66.186.000 bytes. A preparação CPU observada nas capturas ficou em aproximadamente 1–3,3 ms; não é uma distribuição estatística independente. Os nativos restantes ainda enviam 138–327 milhões de triângulos. Dados/capturas: `field-probe.json`, `meadow-*-field.png` e `meadow-*-native_only_ablation.png`.

O teste de agrupamento anterior está em `batch-probe.json`: reduziu 376.078 para 14.331 draw units na vista do jogador, mas a geometria continuou cara. A ordenação candidata altera alguns empates de profundidade e **não foi promovida a reordenação global exata**.

## Gates abertos

- A troca de densidade/escala do campo ainda produz uma faixa visual. A compensação de largura atual não conserva a aparência integrada; não declarar qualidade preservada.
- O custo dominante continua nas malhas completas de árvores/arbustos. Compilar representações menores e medir contra os mesmos objetos, com fallback para a fonte quando detalhes forem visíveis.
- Ainda faltam maior densidade de árvores, comparação A/B com população idêntica, resolução física da janela, trajetórias de câmera/luz, FPS nativo e aceite humano. Não anunciar 60 FPS nem equivalência a Ghost of Tsushima.
- O recurso é um laboratório de sessão. Uma integração persistente com ferramentas MCP/UI e vínculo seguro de fonte será uma unidade posterior; não confundir este opt-in com um formato de mundo pronto.

## Reprodução

```sh
cargo run --release --example horizon_meadow -- compile
cargo run --release --example horizon_meadow -- probe
python3 scripts/open_horizon_meadow.py
```

A janela usa o personagem/KCC e o terreno originais, WASD/mouse, Shift para correr e medidor de apresentação nativo. Não abrir para aceite como otimização concluída enquanto os gates visuais acima falharem. `scripts/link_vegetation_probe.py --probe field --library <rlib>` permite reutilizar uma biblioteca release identificada, registrando hashes e removendo a cópia temporária de link, sem outro build monolítico.

Validação desta unidade: checagem do exemplo, dois testes do leitor binário/frustum e ensaio nativo completo com sombras. A primeira tentativa revelou vínculo incorreto do shadow map; a captura repetida após separar layout/bindings de profundidade passou. Cena/fonte canônicas não foram regravadas; os novos arquivos pertencem somente ao laboratório.

## Segundo recorte: floresta densa e proxies de malha

`horizon_meadow densify` acrescenta 5.383 árvores canônicas, mantendo as108 originais: **5.491 árvores**. Os cinco arquétipos existentes compartilham os mesmos assets/bakes. A grade seeded com jitter cobre todas as400 células de50m; há6–16 árvores novas por célula, com exclusões de caminho/água. Não foram reescritos modelos botânicos para fabricar o ganho. As3058 instâncias nativas iniciais continuam presentes; a nova população é explicitada em `forest.json`. A física conserva o terreno fonte completo; não há nova colisão de troncos/grama.

`src/mesh_proxy.rs` é um laboratório opt-in separado de IGL1/ICB, com `IGPRX1` little-endian: fingerprint SHA256 da fonte/material, índicesu32 de vértices canônicos, triângulos reduzidos e estimativa métrica de erro do meshoptimizer. O pacote é selado por checksum e validado contra índices/tamanhos/finito/truncamento. Não usa JSON no renderer. Fonte e materiais permanecem intactos; normais dos vértices mantidos são as da fonte. Componentes desconectados não são unidos ou apagados; componentes perdidos são restaurados. Faces de área zero são recusadas/retiradas antes de publicar. O pacote corrente contém68 assets e83.976.959 bytes (~80,1MiB), além da fonte CPU ainda residente.

A estimativa de simplificação **não é certificado de Hausdorff, cobertura, silhueta ou pixels**. A seleção usa resolução física, transformação e distância conservadora, com limite estimado0,35px e critério próprio no volume da sombra. Fonte próxima/near-plane, seleção, edição incompatível, rigs/animação, vento, partículas, recorte e planeta conservam o caminho fonte. O catálogo de representações e normais é preparado uma vez e a troca deLOD altera instâncias, sem reconstruir o stream geométrico. `set_mesh_proxy`/`IGNIS_HORIZON_PROXIES` são interfaces diagnósticas de sessão, OFF por padrão, não novos comandos persistentes MCP/UI.

O primeiro A/B com apenas as108 árvores originais e os mesmos materiais/câmeras/luz mediu216,284→69,509ms(jogador),150,901→56,521ms(vale),533,350→178,119ms(panorama). Preparação extra~0,4msCPU. Comparado com a fonte completa, houve diferenças de pixels(RMS RGB8 2,399/2,144/0,734, máximos186/161/76). Esse ensaio usava o pacote anterior de54 assets; não confundir com a população densa ou prometer qualidade exata.

## Stream24B e indexação explícita

Para materiais estáticos cujo domínio3D é a posição original, o caminho experimental `set_reconstructed_coordinates` envia apenas posição+normalf32(24B), reconstrói o domínio e mantém o shader completo de materiais/iluminação. Não substitui o material procedural por uma cor uniforme. Domínios autorais, detalhes projetados, cobertura/transparência, programa procedural, planeta, rigs, animações e vento usam fallback; posições com bits de zero negativo também são excluídas. A elegibilidade é cacheada por fonte, não revarre milhões de vértices por draw. Indexação dessa representação é um opt-in adicional, ainda OFF globalmente.

A/B **da mesma floresta densa** no mesmo executável,1100×760,3warm+8draw+completion, com todos os modelos/proxies/câmeras/luz iguais:

| Vista | Proxies48B(ms) | Proxies24B(ms) | Proxies24B indexados(ms) | Buffers geométricos48B→indexado(MiB) |
|---|---:|---:|---:|---:|
| Jogador |839,430|734,129|300,322|1297,32→314,86|
| Vale |752,774|609,578|328,473|1404,34→441,80|
| Panorama |1451,727|1142,462|456,002|195,35→102,25|

Nas três vistas, ambas as variantes24B produziram RGB8 exatamente igual ao caminho com os mesmos proxies48B. Isso não prova igualdade desses proxies com os modelos completos nem estabilidade universal. Os buffers acima incluem índices e representam a geometria GPU cacheada, **não memória total do processo**. Fonte CPU/catálogo expandido, campo, packet de proxies, targets e outras caches são separados. A memória do campo permanece66.186.000 bytes. A preparação dos proxies na floresta densa ficou aproximadamente1,7–2msCPU por captura, sem regeneração botânica por frame.

`dense-probe-1100x760.json` registra hashes da cena/biblioteca, população, contadores e imagens de todas as variantes. Os tempos são warm draw+conclusão, não FPS de apresentação. Há outros apps/janelas ativos neste Mac; não extrapolar capacidade de outra GPU. A correção da instrumentação removeu o uso de `Engine::replay()` apenas para obter build_id: isso materializava poses da população inteira e esgotou memória depois da primeira imagem. O relatório agora usa a proveniência da biblioteca de link, sem gerar replay.

## Gargalo confirmado e direção

Na vista do jogador, o seletor reduz4,827bilhões de triângulos fonte elegíveis para354,517milhões nas representações; o contador nativo total é373,713milhões incluindo outros draws. São candidatos de câmera/sombra antes da compactação GPU, não uma contagem lida de triângulos efetivamente rasterizados. Há279draw units e oclusãoGPU ativa. Portanto, reduzir draw calls e bytes foi útil, mas preservar cada componente de folha impõe um piso geométrico alto. Ainda não é uma floresta utilizável.

O próximo recorte necessário é **representação coletiva de cobertura/aparência do objeto ou região**, preparada antes do gameplay, preservando a copa/contorno e os vazios relevantes sem manter cada folha como unidade distante. Deve usar os vínculos de objeto composto e métricas de projeção, sem teste de espécie. Exige comparação de cobertura, detalhes, memória e tempo com fallback para geometria próxima; simplesmente apagar folhas/árvores pequenas ou trocar tudo por uma massa opaca não satisfaz o contrato. A faixa/aliasing do campo de grama continua reprovada; não escondê-la com as árvores ou declarar aparência integrada preservada.

```sh
cargo run --release --example horizon_meadow -- compile-proxies
cargo run --release --example horizon_meadow -- densify
cargo run --release --example horizon_meadow -- probe 1100 760 dense
# Câmera/luz variando, comparando o mesmo proxy48B com24B indexado:
cargo run --release --example horizon_meadow -- probe 1100 760 verify
# Janela diagnóstica; fonte/proxies/campo em sessão, não uma entrega pronta:
python3 scripts/open_horizon_meadow.py --mode indexed
```

`probe ... sparse` filtra explicitamente apenas as novas colocações `horizon_tree_` em memória para repetir o cenário de108 árvores, sem regravar a floresta densa. `--measure-only` abre/mede/fecha somente a própria janela; o launcher registra resolução física, modo de apresentação, amostras frescas e `sample_complete`, sem inventar FPS quando a janela ainda não apresentou frames.

## Apresentação na janela real

`open_horizon_meadow.py --mode indexed --measure-only` foi executado numa janela própria, ativada pelo PID, e encerrou apenas essa janela. Após a inicialização sem amostras, oito amostras frescas registraram viewport físico **2200×1520**, apresentação `AutoNoVsync(requested)` e cadência mediana3,574FPS(intervalo2,928–7,659; as amostras iniciais incluem a transição de aquecimento). As seis últimas ficaram3,10–4,17FPS, mediana3,574. É cadência de chamadas de apresentação incluindo editor/simulação/aquisição/submissão, não timestampGPU nem scanout, e não um ensaio isolado das outras janelas do Mac. O campo/proxies/indexação estavam ativos por opt-in; `sample.optimized=false` desse medidor é o indicador legado da otimização atmosférica da cena, não um resumo desses novos caminhos. `window.json` e `presentation-summary.log` preservam amostras/posição/versão binária, e `sample_complete=true`.

**Resultado reprovado para uso jogável.** Não converter a redução de geometria/bytes em promessa de60FPS. O experimento confirma que a representação por componente, mesmo indexada, não é suficiente para essa densidade de modelos canônicos. A cena de1km² existe e é reproduzível, mas cobertura coletiva, artefatos do campo, estabilidade/qualidade e aceite humano permanecem abertos na EAF-215.

Verificação adicional do stream indexado: oito pares de câmera deslocada e direção de luz variável(`verify-probe-1100x760.json`, população explicitamente filtrada para as108 árvores originais), comparando o mesmo proxy48B com24B indexado, deram diferença RGB8 zero nos oito pares. Isso exercita mudanças de escolha de nível e sombras, mas não prova estabilidade contínua universal ou a fidelidade dos proxies contra a fonte. O caminho segue experimental/OFF globalmente.

Validação deste recorte: Clippy do lib+exemplo atual com `-D warnings` passou; três verificações isoladas passaram(roundtrip/corrupção/truncamento do pacote, preservação de dois componentes e reparo de face degenerada com restauração de componente). O reparo foi compilado diretamente do módulo atual, ligado à biblioteca identificada do ensaio; os quatro testes internos atuais de mesh_proxy também passaram diretamente no binário de testes release corrente(166testes existentes, filtro executou4): catálogo/cache em trocasLOD, fonte editada/fallback, pacote selado/índices inválidos, componentes e reparo degenerado. Os ensaios GPU completos e oito pares câmera/luz passaram sem erro de validação. A biblioteca temporária de link foi mantida somente durante o experimento e removida ao concluir a unidade. Nenhuma cena-fonte210 foi regravada; SHA256 permanece `da0805a03945de538bbcdf65bf1769a8dc66fcb21826e646e91454918d9c22c9`.
