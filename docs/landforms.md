# Formas de relevo 3D: estudo e ferramentas

Landform2 (EAF-221) adds independently scaled contour rays per section, common
plane/spherical placement, and altitude/slope queries. Source1 remains readable
and byte compatible. The version1 restrictions below describe the original
increment; [current requirements and API](relief-requirements.md) document the
new controls, validation evidence, performance review and open integration gates.

Atualização179: publicação166 e renderer otimizado atual já convivem na main;
vãos e superfícies foram verificados também com os atalhos gráficos ligados.
[Integração, referência de montanhas e limites](terrain-render-integration.md).
As exclusões da unidade166 abaixo são históricas; física/nav e streaming ainda
exigem suas próprias integrações.

Continuação de terreno164,31/10/2026. Perfis/seções complementam o campo de alturas; não substituir toda planície por voxels. Autoria manual e receitas usam o mesmo documento editável. Pesquisa Nanites, atmosfera/distant e física/nav não são integradas por esta unidade.

## Estudo de formas naturais

| Referência primária | Característica observada/documentada | Requisito para a ferramenta |
|---|---|---|
| [Australian Museum — Wave Rock](https://australian.museum/learn/australia-over-time/evolving-landscape/wave-rock/) | Face curva, base recuada e projeção superior arredondada | Perfil côncavo: uma vertical pode cruzar mais de uma superfície |
| [NPS — Canyonlands](https://www.nps.gov/articles/nps-geodiversity-atlas-canyonlands-national-park.htm) | Paredes íngremes, camadas resistentes, mesas e patamares | Faces realmente verticais, caps planos e degraus horizontais |
| [NPS — Bryce hoodoos](https://www.nps.gov/brca/learn/nature/hoodoos.htm/index.htm) | Colunas com saliências/reentrâncias associadas a resistência desigual das camadas | Escala variável por seção e avanço/recuo de beiradas |
| [USGS — Arches](https://www.usgs.gov/geology-and-ecology-of-national-parks/geology-arches-national-park) e [Utah Geological Survey](https://geology.utah.gov/map-pub/survey-notes/glad-you-asked/glad-you-asked-why-natural-arches-in-utah/) | Paredes estreitas com aberturas e pontes naturais | Vãos reais; tampas não podem preencher a cavidade por engano |
| [NPS — Arêtes e horns](https://www.nps.gov/articles/nunataksareteshorns.htm) | Cristas estreitas, faces definidas e picos pontiagudos | Perfil angular, deslocamento de eixo e fechamento em ápice |

São referências de morfologia. Os presets abaixo não afirmam reproduzir erosão, litologia, glaciações, equilíbrio estrutural ou reconstruir esses locais. Inclinar um pico artificialmente é uma capacidade geométrica; não identificar toda forma inclinada como Wave Rock. Dunas, terraços aluviais e relevo amplo ainda podem usar heightfields. Cavernas ramificadas, carste complexo e booleanas exigem ferramentas adicionais.

## Decisão de representação

Heightfield só possui uma altura porXZ e não representa parede exatamente vertical, teto ou solo sob outra superfície. Aumentar resolução não remove essa limitação. Malhas paramétricas locais preservam essas superfícies sem exigir uma grade volumétrica para o mundo inteiro. SDF/voxels/CSG são candidatos para escavação arbitrária futura, com custos próprios de extração/edição/residência; não são simulados por um flag neste incremento.

Núcleo: contorno2D simples, inclusive côncavo, orientadoCCW, extrudido por seções em planos paralelos de um referencial3D. Seção desloca centro e muda escala uniforme positiva; ápice terminal pode ter escala zero. A escala uniforme sem twist mantém homotetia e evita interpolação de perfis que se autoatravessam. Patamar entre seções na mesma altura exige centro igual e contorno estrelado em relação à origem; no máximo duas seções por plano. Cavity opcional usa contorno interno homotético e annuli de fechamento, conservando um túnel aberto nas bocas. Não aceita twist/perfis diferentes por seção ou união booleana entre peças.

Triangulação de caps por ear clipping com validação de contorno e orçamento explícito, sem fan que preencha o recorte côncavo. Sem dependência nova nem promessa de predicados geométricos exatos. Verificar área, faces degeneradas, incidência/orientação das arestas e volume após converter paraf32; erros recusam a publicação inteira.

## Checklist estático antes do código

1. Frequência: validar/triangular/compilar só em edição/consulta explícita, nada no tick/frame. Publicar malha nativa já pronta.
2. Memória: arrays densos, capacidades calculadas antes de gerar, limites de contorno/seções/peças/trabalho e orçamentos nativos preservados. Sem volume alocado proporcional à área do mundo.
3. Tipos: fontef64 local+Position global; malhaf32/u32. JSON só MCP/autoria, CBOR tipado versionado na persistência. Reutilizar codec posicional48B de terreno.
4. Reuso: caps e referencial preparados uma vez por peça; incidência/volume ao compilar. Compilado compartilhado porArc na publicação; sampler/rays Rust sem JSON e sem recompilar por raio.
5. Complexidade: validar interseções2D quadrático, ear clipping limitado e preflight de custo; não fazer auto-interseção global de malhas arbitrárias ou teste de toda a cena.
6. GPU/I/O: nenhum shader/novo pipeline/readback no caminho estável. Mesmo upload/cache de meshes e escrita atômica confinada ao projeto.
7. Concorrência: nenhum worker/lock novo, fonte candidata validada antes de mutar. Fonte digest/revisão, perfis e limites versionados; não usar hash como posse/autoridade.
8. Segurança: decoder bounded e checksum/EOF/profundidade, sem relaxar PositionJSON para células numéricas inseguras. Erro tardio não publica asset/instância parcialmente.
9. Provas: riscos de cap tampando arco/túnel, malha aberta/invertida, falha de codec e edição/publicação atômica. Não criar teste por parâmetro/preset; prova integrada e regressões por impacto.
10. Limites: composição de peças não é boolean union; queries de superfície não significam colisão do gameplay. Sem nav/streaming/LOD atmosférico/aceite artístico ou geologia universal inferidos.

## Ferramentas e contratos

`query_landform` aceita `preset`, `inspect`, `compile`, `edit` e `raycast`. Presets wave_cliff/canyon/bent_peak/mesa/hoodoo/arch/tunnel retornam um `Document` comum. Uma crista manual pode ser um perfil triangular extrudido, sem chamar preset. O documento contém Position global, ID/revisão e até32peças independentes. Cada peça tem anchor local, axis/profile_x, contorno2D e2..128seções; não há evaluator geológico privado. axis é normalizado e profile_x projetado ortogonalmente; profile_y=axis×profile_x. Offset e lean estão nesse referencial, não são coordenadas globais XYZ.

O contorno tem3..96vértices finitos, sem fechamento repetido, área positivaCCW e sem cruzamentos/toques não adjacentes. Caps usam os mesmos índices das paredes. Parede vertical e topo plano são coordenadas reais, não um declive de89,9graus. Seções interpolam linearmente deslocamento e escala uniforme; um caminho curvo é uma sequência de seções (o preset bent_peak amostra centros quadráticos). O último anel sólido pode virar um único ápice; cavity não colapsa num ponto. Degraus em um plano só no interior da peça, com anéis homotéticos aninhados de centro igual; proibir extremidades coplanares evita películas sem espessura. Até duas seções por plano; três podem sobrepor anéis e são recusadas.

`inner_scale` exige origem estritamente no kernel estrelado do contorno e parede não nula. O contorno interno é homotético; tampas são annuli que preservam as bocas. Um arco com abertura pelo chão usa contorno côncavo sem cavity. Túneis com seção arbitrariamente diferente, twist, ramificações/booleanas e cavernas fechadas complexas ainda precisam de ferramentas adicionais. Não aprovar topologia arbitrária por um teste devolume; a construção é restrita, e incidência/volume/área são verificações complementares. Predicadosf64 têm tolerâncias e podem recusar detalhes quase degenerados. Arredondamento finalf32 é reportado, sem promessa de certificação geométrica de malhas importadas.

`edit` admite append/replace/remove/set_profile/set_sections/set_inner_scale. Opera documento candidato, verifica revisão esperada, limita32operações e valida resultado final uma vez, antes de devolver uma nova revisão. Remover todas as peças ou deixar perfil aberto/invertido falha. Revisão da entrada não é CAS de arquivo entre processos. Fontes podem ser compostas manualmente ou por regras/seed do jogo; valores aleatórios dos presets são locais por parâmetro/índice, sem RNG global.

`compile` produz geometria nativa e relatório por peça (intervalos de vértices/triângulos, volume positivo, vazio e ápice), digestSHA256 da fonte e erro de conversãof32. Perfil e caps preparados uma vez por peça; malha resultante emArc. Peças continuam independentes, inclusive se se sobrepõem: `boolean_union:false`, volume é por peça. Não há extração de superfície externa de uma união.

`raycast` usa os triângulos compilados, ambos os lados, em coordenadas LOCAIS do documento. Compila uma vez por lote; a API Rust `Compiled::intersections` reutiliza a malha. Retorna cruzamentos ordenados e remove duplicações de arestas da mesma peça, sem inventar física/nav/apoio ou união entre partes. Até128raios e4milhões de testes raio×triângulo; também limitar saída:4096acertos brutos/raio,512distintos/raio e4096porlote. Excesso recusa sem resultado parcial, pedindo consulta mais estreita. Agrupar por peça antes do dedup evita interferência de outra peça sobreposta. Ordenar registros numéricos primeiro; clonar nomes/calcular pontos/normais só para os acertos finais. Isso verifica que uma vertical atravessa a parte superior e inferior de um overhang, que o arco está vazio e que o túnel não foi tampado.

`publish_landform {publication}` aplica asset+instância em uma única transação autoral pausada, respeitando revisão de cena/undo/replay/validação existentes. O frame_origin só projeta para a cena visual atual; não rebasa simulação. A fonte não fica embutida na cena; salvar também `.ignis-landform`. `save_landform`/`load_landform` guardam/devolvem autoria, sem trocar mundo ou cena. Envelope8MiB/profundidade64/SHA256/EOF/replace atômico comum ao terreno, magic própriaIGNISLFM/versão1; Position em48B LE e JSONcanonicalstrings. Extração mecânica dos helpers preserva bytes de terrain1, protegidos pelo roundtrip do arquivo164 já versionado. Não há nova versão Scene/replay/protocolo desta unidade.

Budgets de compilação:32peças,96pontos/contorno,128seções,4milhões de unidades de trabalho estimadas antes de triangular; respeitar MAX_VERTICES16384/MAX_TRIANGLES32768 do asset e limites globais nativos. Reduzir resolução ou dividir assets quando necessário. Nenhuma tabela por frame, worker ou nova dependência.

## Reprodução e demonstração

`cargo test --locked --release --test landforms --test terrain --test world_space` cobre os riscos críticos de caps/vãos/manifold/formato/edição, além da continuidade164. `python3 scripts/verify_landforms.py` usa apenasMCPstdIO eGPUoffscreen: galeria de oito formas, incluindo crista manual, fonte `examples/landforms.ignis-landform` e receita `examples/landforms.recipe.json`. Cena derivada em `artifacts/landforms.scene.json`, independente da versão de Scene reservada pela frente165.

Imagens técnicas: `artifacts/landforms-gallery.png`, `landforms-wave.png`, `landforms-canyon.png`, `landforms-peak.png`, `landforms-arch.png`. São formas idealizadas inspecionáveis, sem reconstrução fotográfica, simulação de erosão ou playtest físico implícitos.

## Checklist estático após implementação

1. Frequência: perfis/caps/referencial e volume são preparados em comandos explícitos; não adicionar geração ou triangulação ao tick/frame. API Rust de compilado permite reuso nas consultas semserde.
2. Memória: vetores reservados com contagens previstas, geometriaArc, caps uma vez por peça. Raycast conserva no máximo4096registros brutos numéricos e512acertos finais por raio; orçamento de operações sozinho não bastaria para limitar a resposta. Lote limita4096resultados antes de construirJSON.
3. Cópias: publicação clonaArc, não malha; IDs dos hits só clonados após ordenação/dedup/limites. Fonte candidata é propriedade do comando e não muta o arquivo ouScene se falhar.
4. Tipos: fontef64 local e Position canônico; geometria nativaf32/u32, rounding máximo reportado. Trigonometria/seed dos presets só na criação; fontes salvas são pontos/seções ordinários, sem dependência do agente.
5. Reuso/trabalho: caps por earclipping e perfil simples validados uma vez por preparo; preflight4milhões e limites nativos antes de gerar. Edit valida a fonte e o resultado final, custo eventual limitado (pode preparar duas vezes); não há cache global de documentos sem invalidação.
6. Algoritmos: polígonos/rays limitados, sem voxelglobal nem booleanas. Contagem de incidência porBTreeMap e volume por peça em compilação; raycast linear na geometria fornecida, até4milhões de testes/lote. Não substituir broadphase de gameplay por este diagnóstico.
7. GPU/I/O: usa malhas/cache/shaders existentes, sem upload adicional por frame. Fonte CBOR8MiB/versionada e compartilhamento mecânico do envelope de terreno; golden roundtrip prova bytes164 preservados.
8. Concorrência: sem threads/locks novos, fases nativas de edição/undo preservadas; mundo pausado para publicar. Coordenação dos hunks/target da frente165, sem incorporar alterações alheias no commit.
9. Correção: testes independentes de incidência porTopology e raios reais atravessando vãos/overhang; caso adversarial de loft zigzag protege pressão de memória/saída, mantendo erro atômico. Não fazer teste por preset/flag; dois cenários críticos agregam esses riscos e formatos.
10. Limites: peças fechadas não significam união entre elas, erosão simulada, hidrologia, colisão/nav ou cobertura até horizonte. Demos são técnicas e facetadas; sem medição ou promessaFPS, fidelidade artística ou suporte multiplataforma presumidos.

## Evidências finais

6testesRust em3alvos aprovados:2provas críticas landforms (geometria/vãos e autoria/formatos/limites),3terrain e1world_space. Perfil côncavo é conferido também pelo Topology nativo independente; raycasts testam paredes/topos/pisos, WaveRock idealizado, pico curvado, arco e túnel. O arquivo terrain164 já versionado decodifica/recodifica byte-exato após extrair helpers. Caso adversarial32lofts zigzag prova orçamento de resposta, e17raios completos excedem o teto de lote; o primeiro fixture de stress tinha alcance100m num sólido127m, foi corrigido para200m sem mudar os limites de produto.

Clippy release/all-targets passou no primeiro freeze; revisão final proporcional lib/bin/landforms/terrain/world_space-Dwarnings passou após as cotas. Fmt/Python/diff e build aprovados. `verify_landforms.py` e regressão `verify_terrain.py` passaram6gruposMCP no binário conjunto `f3075f4a95af348a`, sem janela/áudio. Arquivos/purequeries/erro tardio/undo/replay/recarga e igualdade de pixels verificados. Galeria8formas/1370vértices/2712triângulos, mais plano de apoio visual, com capturas técnicas nativas inspecionadas; não é aceite de arte ou gameplay físico. Percursos de wave/canyon foram modificados por SetSections no mesmo lote autoral; crista foi criada sem preset.

Relatórios `artifacts/landforms-verification.json` e `terrain-verification.json`, logs `landforms-tests.log`, `landforms-clippy.log`, `landforms-clippy-final.log`, `landforms-build.log`, `landforms-mcp.log`, `landforms-terrain-regression.log`. Usou apenas saída/cache release existente, um app corrente `target/release/ignis`, sem builds históricos ou cachedebug novo. Trabalho166 preservou a frente gráfica concorrente;165 publicou seu próprio0e79eb5 durante os gates. Esta unidade não altera renderer/model/meshes/Scene/replay/rede e não importa worktree Nanites. O build conjunto não transforma testes geométricos em uma nova validação da pesquisa gráfica.
