# Residência de malhas estáticas únicas — EAF-181

O renderer anterior mantinha em buffers residentes compactos somente assets
repetidos. Uma malha estática grande usada uma única vez continuava no fluxo
expandido de 64 B por canto, com comparação do payload a cada desenho. Na paisagem
ampliada, quatro terrenos únicos somam 2.264.192 triângulos; esse caso motivou o teste.

Agora assets estáticos com pelo menos 1.024 triângulos também podem usar o caminho
residente, mesmo com uma única instância. Rig, clipes, playback e attachments
mantêm o fallback existente. Malhas únicas menores conservam o caminho anterior.
O limiar é uma heurística de preparação, não limite de conteúdo nem certificado
universal de qual alternativa é mais rápida. A regra é genérica, sem nomes de
assets, classes de vegetação ou mudança de população.

Não há simplificação nesta etapa. Posições/normais continuam f32, mesmos triângulos,
mesmos materiais e coordenadas de material quando consumidas. O layout existente
usa 48 B por canto com material procedural ou 24 B sem canais procedurais; transform,
tint e material ficam por instância em 80 B. Só são omitidos atributos que o shader
não consome. LODs/atmosfera/visibilidade/oclusão anteriores continuam compondo com
a representação escolhida. Fonte autoral e colisão não são alteradas; sem mudança
de Scene/replay/protocolo ou de formato de asset.

## Uma falha de fidelidade encontrada e contida

O primeiro candidato também permitia indexação de malhas únicas simples.
Um teste com superfície curva de 1.152 triângulos cruzando a grade diagnóstica do
piso apresentou máximo 10 RGB8 contra o fluxo anterior. Desligar só a indexação
zerou a diferença nessa vista; desativar adicionalmente compactação/layout mínimo
não trouxe correção adicional. Isso isola a mudança de caminho indexado como
condição da falha observada, sem afirmar a causa interna do driver/interpolador.

Por isso a promoção de assets únicos conserva seu fluxo de cantos, sem indexação
da geometria detalhada. Assets repetidos preservam a política anterior; preenchimentos
atmosféricos constantes ainda podem usar índices. Não relaxamos o limite de 2 RGB8 do
comparativo para aprovar o candidato. O teste crítico mantém material, transformação,
edição da fonte, sombra e resize para proteger invalidação e integração do cache.

`Frame::set_unique_static_residency(false)` é um oráculo técnico para comparar o
caminho anterior, sem alterar o documento. Não é configuração gráfica de produto;
a residência aprovada funciona automaticamente e não requer novo controle UI/MCP.

## Reprodução e fronteiras

```sh
cargo test --offline --test mesh_resources --test render_occlusion --test mesh_shading --test mesh_attachments -- --include-ignored --nocapture
cargo build --offline --release --bin ignis --example static_residency_cost
target/release/examples/static_residency_cost artifacts/expanded-world/world.scene.json artifacts/static-residency/expanded
target/release/examples/static_residency_cost artifacts/landscape-forest-dense/forest.scene.json artifacts/static-residency/original --authored-only
```

O exemplo mantém conteúdo, atmosfera, câmera e luz iguais entre alternativas,
com os atores ocultos em ambas. Compara 3 vistas e 2 resoluções físicas na ampliada;
a opção authored-only usa só a câmera autoral da cena anterior. Cada caso faz
3 repetições alternadas,8 desenhos de aquecimento+20 amostras, parado e em movimento.
A medição inclui draw nativo+espera de conclusão GPU, não é timestampGPU nem FPS
de apresentação. Captura/PNG/readback/simulação/preparação fria ficam fora das
amostras. Preparação de Frame e primeiro draw da rodada são registrados separados;
aquele draw já pode encontrar caches quentes e não representa custo de carga inicial.

Capturas estáticas repetidas precisam ser estáveis; o JSON registra máximoRGB8 e
pixels alterados. Esses recortes não são prova de equivalência universal em todo
material/dispositivo. `selected_geometry_bytes`/`allocated_geometry_bytes` medem
buffers geométricos, não RAM total/VRAM total: fontesCPU, caches, imagens de depth,
instâncias e demais recursos continuam existindo. Os símbolos ICB1 permanecem OFF.

A rodada de medição interrompida durante investigação da falha indexada não é
evidência de performance. Os resultados finais devem vir da execução posterior
sem Cargo, outra captura ou janela contínua disputandoGPU.

## Resultado final em 01/11/2026

Build `f6e2cb9f82fa102d`, macOS arm64/Metal/M4 Pro, sem Cargo, janela contínua
ou outra captura concorrente. Mediana das três medianas de cada alternativa;
mesma cena ampliada de 20 × 20 km, 30 mil objetos/60 mil peças de vegetação,
mesma atmosfera com influência 0,65. Nenhum triângulo foi removido nesta mudança.

| Vista | Pixels físicos | Câmera | Anterior (ms) | Residente (ms) | Menos tempo |
|---|---|---|---:|---:|---:|
| Panorama | 2784×1560 | parada | 225.917 | 217.939 | 3.53% |
| Panorama | 2784×1560 | movendo | 238.276 | 218.371 | 8.35% |
| Panorama | 1280×800 | parada | 212.175 | 203.541 | 4.07% |
| Panorama | 1280×800 | movendo | 223.513 | 204.181 | 8.65% |
| Próxima | 2784×1560 | parada | 100.003 | 92.572 | 7.43% |
| Próxima | 2784×1560 | movendo | 112.825 | 92.627 | 17.90% |
| Próxima | 1280×800 | parada | 80.913 | 72.326 | 10.61% |
| Próxima | 1280×800 | movendo | 93.538 | 72.425 | 22.57% |
| Oblíqua | 2784×1560 | parada | 159.688 | 151.094 | 5.38% |
| Oblíqua | 2784×1560 | movendo | 172.891 | 151.803 | 12.20% |
| Oblíqua | 1280×800 | parada | 142.272 | 133.406 | 6.23% |
| Oblíqua | 1280×800 | movendo | 153.678 | 132.957 | 13.48% |

Na resolução da janela, o ganho variou de 3,53% a 17,90%; em 1280×800,
de 4,07% a 22,57%. São tempos de draw + espera GPU, não FPS de apresentação.
Isso não mede separadamente tempo de CPU e tempo de execução GPU.

As seis comparações de imagem foram estáveis. Panorama nas duas resoluções e
vista próxima em 1280×800 ficaram RGB8 exatos. Próxima em 2784×1560 mudou três
pixels com máximo de 1 RGB8; oblíqua mudou um pixel nessa resolução e dois em
1280×800, também com máximo de 1 RGB8. São capturas estáticas nas poses testadas,
não certificado universal de todos os frames em movimento ou de outros devices.
O teste crítico separado varia transformação, material, geometria, luz e sombra.

| Buffer geométrico selecionado | Anterior | Residente |
|---|---:|---:|
| Bytes | 435.591.000 | 326.909.784 |
| MiB | 415,41 | 311,77 |

Redução de 103,65 MiB / 24,95% nesses buffers. Alocação geométrica reportada:
435.762.676 → 326.909.940 B. Não é redução medida da RAM total do processo.
Contador cumulativo de uploads de assets compartilhados ficou em 2 no anterior
contra 6 no residente, estável em todas as vistas/resoluções/rodadas: os quatro
terrenos agora são uploads individuais residentes, sem reenvio a cada câmera.
O contador não inclui o buffer legado, imagens ou upload de instâncias.

Preparação fria de Frame teve uma amostra de 78,89 → 91,81 ms. Não é um benchmark
robusto de carga inicial: há custo inicial de conversão/organização e ele não deve
ser confundido com o ganho aquecido. Tempo do primeiro upload GPU completo não
foi isolado. Os fontes CPU continuam residentes; o compilador de pacotes pode
reduzir preparação futura, mas isso não foi implementado nesta unidade.

A cena anterior usa os mesmos caminhos geométricos em ambas as alternativas:
6 vértices legados, 9.149.788 B selecionados e 17 uploads cumulativos. Pixels
RGB8 exatos nas duas resoluções. Os tempos aquecidos foram:

| Pixels | Câmera | Anterior (ms) | Residente (ms) |
|---|---|---:|---:|
| 2784×1560 | parada | 54.556 | 54.538 |
| 2784×1560 | movendo | 54.633 | 54.765 |
| 1280×800 | parada | 41.613 | 41.706 |
| 1280×800 | movendo | 41.910 | 41.765 |

Variações entre −0,24% e +0,35% de ganho na cena anterior: caminhos e dados iguais,
sem evidência de uma mudança relevante de desempenho nesse controle.

Gates: 25 testes em `mesh_resources`, `mesh_shading`, `mesh_attachments` e
`render_occlusion` passaram, incluindo os testes GPU ignorados por padrão.
Clippy em todos os alvos, Clippy final do utilitário, formatação e release passaram.
Um único executável corrente em `target/release/ignis`; bundles existentes continuam
apontando para ele. Fontes/cenas originais e trabalho paralelo de botany preservados.
O PNG/readback e o decoder locais do exemplo aceitam a resolução nativa do teste;
os envelopes públicos de captura/comparação de imagens permanecem inalterados.

Evidências locais: `artifacts/static-residency/{expanded,original}/cost.json`,
PNGs e logs correspondentes. A cópia temporária com instâncias artificiais usada
na sondagem foi removida; a implementação final não adiciona instâncias ocultas.
EAF-181 continua aberta para os LODs coletivos/símbolos com fidelidade e custo
certificados. Esta unidade melhora o fluxo nativo, sem aprovar os símbolos que
falharam imagem nem resolver todo o custo da floresta.
