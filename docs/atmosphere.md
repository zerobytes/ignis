# Representações atmosféricas compiladas — EAF-165

Contrato inicial abaixo. A evolução atual separa céu e montanhas, acrescenta
ATM3 e contorno de seleção: [Camadas e silhuetas — EAF-169](atmosphere-silhouettes.md).
Scene101/replay111/conteúdo3 substituem os números históricos deste documento.

A entrega substitui malhas estáticas distantes ANTES do upload e do desenho.
O interior usa um shader de preenchimento que não consulta normais, ruído,
material procedural, mapa de sombras ou ambiente especular. A referência com
geometria completa usa a mesma regra de cor atmosférica; fog aplicado depois do
trabalho completo não é contabilizado como otimização. Sem partículas no ar.

## Uso nativo e contrato

Na UI, **Iluminação → Atmosfera da cena**: ativar, configurar densidade/cores,
**Compilar malhas distantes** e **Usar representações compiladas**. A compilação
é explícita, de autoria/build time; não acontece ao mover câmera/luz. No MCP,
os mesmos controles usam `set_light` e `edit_scene` com
`{"op":"compile_distant_meshes"}` / `{"op":"clear_distant_meshes"}`.
`inspect.scene.distant_meshes` contém os pacotes e `capture.adapter.distant_selection`
expõe triângulos, bytes selecionados/alocados, instâncias e limites utilizados.
Editar geometria ou material descarta o pacote derivado na normalização;
não bloqueia a edição nem aceita uma representação desatualizada. Undo restaura
fonte/pacote anteriores juntos. Remover compilados libera a cache gráfica na próxima atualização.

`Light.atmosphere` opcional, com `density`0..1/m, `color`/`sun_color`RGBlinear0..1,
`sun_strength`0..1; `reduce_details` false por omissão no formato,
`silhouette_error_px`0..2(default0,5), `color_error`0..0,05(default0,002 linear).
A UI cria um perfil com redução habilitada; sem compilados usa a fonte.
Ausência/densidade0 preservam o desenho anterior. Densidade é aparência autoral;
tolerâncias são qualidade. Desativar redução mantém a atmosfera da referência.
Guia diagnóstica não é conteúdo do mundo e permanece legível.

## Compilador e dados

ATM2: magic4, SHA25632 da geometria + material binário, limite de tintRGB3×f32,
quantidadeu32; até5 níveis com errof32, eixou32, contagensu32, posições3×f32 e
triângulos3×u16, little endian. Máximo4MiB/pacote,16pacotes. Sem níveis dominados
ou vértices não utilizados nos proxies. RGBlimite vira preenchimento/critério de
seleção, não uma média fotográfica ou radiância pré-calculada para toda luz.

Campos de altura retangulares são reconhecidos pela topologia e coordenadas,
sem histórico de subdivisão. O certificado mede a diferença vertical máxima
nos vértices da sobreposição exata das duas triangulações, incluindo cruzamentos
de diagonais. A projeção distingue erro lateral e profundidade segundo o eixo
vertical transformado. Outras malhas usam agrupamento espacial e limite
conservador de deslocamento. A validação de um pacote correspondente recompõe
sua derivação; um hash sozinho não autoriza um certificado forjado.
Essa prova é memorizada por identidade dos dados imutáveis (pacote/geometria)
e valor do material. `Arc::make_mut` produz outra alocação porque a prova retém
as referências; uma edição não pode herdar uma prova de bytes diferentes.
Load valida novamente; câmera/luz reutilizam a prova, sem recompilar a fonte.

Dados são binários no pacote e arrays tipados no runtime/GPU. JSON/MCP/cena
editável codificam o blob em base64 apenas na borda de autoria; saves CBOR usam
bytes. Não há JSON, interpretação de comandos ou compilação no draw.
Scene100/replay110/conteúdo visual2; protocolo4,checkpoint2 e PSV1 mantidos.
Nenhuma frente de terreno/NPC é importada ou substituída por este mecanismo.

## Seleção e aparência

Transmitância `T=exp(-density*distância)`;
cor linear `T*superfície+(1-T)*ar(direção)` antes da conversão sRGB.
A cor do ar interpola cor base e solar com lobo direcional suave. É uma
aproximação homogênea artística, sem espectro, altitude, nuvens, disco solar ou
sombras volumétricas. [Lei de transmitância no PBRT](https://www.pbr-book.org/4ed/Volume_Scattering/Transmittance).
O mesmo fundo faz o horizonte convergir, sem uma nuvem/volume geométrico adicional.

O limite do material compilado e a transmitância mais alta da esfera do objeto
controlam a substituição por preenchimento. Variação de profundidade entre
superfícies usa todo o intervalo de distância, inclusive em vistas rasantes.
A geometria só reduz quando o limite projetado cabe no erro de contorno em
pixels FÍSICOS. Reserva20% do orçamento para margem. A seleção mantém a ordem
autoral dos draws, inclusive entre caminhos distintos; isso preserva os empates
de profundidade da fonte. Transições são limitadas por erro, sem crossfade ou
reconstrução temporal. Limite geométrico não significa igualdade pixel a pixel
em bordas de rasterização; validar movimento/aliasing continua necessário.

## Fronteiras reais

- Primeiro recorte por instância estática inteira. Sub-regiões são evolução futura;
  uma montanha grande conserva mais detalhe se qualquer parte exigir.
- Rigs/animação/attachments, reflexos especulares de material, sombras direcionais
  ou IBL ativo conservam a fonte. Não omitir um shadow caster porque esmaeceu na câmera.
  A luz de trabalho do editor tem sombras por padrão; desligá-las ou usar luz da
  cena para experimentar a redução. A atmosfera da cena sobrevive ao override de luz.
- A fonte editável, materiais e vértices CPU completos ainda ficam na engine.
  A economia de memória medida aqui é nos buffers GPU selecionados, não RAM total,
  nem streaming de mundo. O renderer atual não possui texturas externas carregáveis;
  seu streaming/eviction não foi implementado nem alegado.
- Compilados/níveis decodificados ocupam memória CPU adicional. Buffers GPU menores
  são alocados quando o selecionado ultrapassa a capacidade ou cai abaixo de1/4;
  buffers antigos deixam de ser retidos pela aplicação. A liberação física fica
  com a fila/driver. A reaproximação pode gerar upload maior: medir a transição.
- Desenho continua finito e limitado pela câmera e budgets nativos. Não é Nanite
  universal, iluminação global, streaming planetário ou descarregamento de gameplay.

## Revisão estática antes/depois

1. Compilação/validação de derivação são eventuais, O((V+T)logV) por nível; zero no tick/draw.
2. Arc para pacotes/geometrias; planos/scratch/buffers reutilizados. Fonte CPU e cache
   de upload permanecem por autoria/retorno. Não afirmar memória total menor.
3. f32/u16 planos, sem nomes/JSON por vértice. Shader reduzido24B/v contra64B/v detalhado.
4. Bounds/cor do material preparados; seleção O(instâncias×5), sem varrer fonte.
   Reconstrução dos buffers só após mudança de plano, conteúdo ou cor usada.
5. Até16 assets/32instâncias/5níveis; certificados conservadores preferem fallback.
6. Recursos criados uma vez, uniform+80B, sem textura/readback novo no render normal.
   Upload e capacidade refletem seleção; tabela material substitui slots não usados
   por neutros, mantendo a alocação uniforme fixa preexistente.
7. Sem workers/dependências novos; `OnceLock` publica a prova uma vez, sem mutex
   no draw. Contador atômico local identifica gerações de Frame e impede cache
   de reutilizar buffers de snapshots distintos.
8. Edição/material/pacote/câmera/resolução/luz, near retorno, load/undo/replay e rede
   revalidam ou refazem decisão. Fonte alterada usa fallback, sem recompilação oculta.
9. Provas críticas de formato/atomicidade/persistência/conteúdo; imagens e custos
   verificados diretamente. Nenhuma promessa visual universal/FPS.
10. Fill e fundo ainda custam operações GPU. Objetivo é evitar mais trabalho do que
    se acrescenta; métricas incluem seleção/submissão/uploads e conclusão GPU.

## Reprodução

`python3 scripts/verify_atmosphere.py --binary target/release/ignis`
produz a cena compilada e evidências correntes em `artifacts/atmosphere/`.
`cargo run --release --example atmosphere_cost` mede desenho+espera de GPU nativa,
sem PNG, simulação ou apresentação da janela. Referência e candidato usam a mesma
atmosfera, fonte, materiais e resolução. Relatórios distinguem preparação,
transição e mediana estável; não são FPS de um jogo.


## Evidência nativa e interpretação

Cena sintética com cinco instâncias de uma malha de24.200triângulos e chão de2
triângulos:121.002 no total, material procedural com relevo, pacote compartilhado
ATM2 de100.516bytes (98,2KiB). A fonte não muda entre referência e candidato.
Com1728×1117pixels físicos e erro de contorno0,5px:

| Câmera / condição | Triângulos completos + baratos | Bytes de vértices selecionados | Diferença máxima RGB8 |
| --- | ---: | ---: | ---: |
|100m, densidade0,009|121.002 +0|23.232.384|0|
|1000m, densidade0,009|2 +121.000|8.712.384|0|
|9000m, densidade0,009|2 +62.068|4.469.280|0|
|Retorno100m|121.002 +0|23.232.384|0|
|9000m, densidade0,0004, FOV12°, tolerância de cor0,01|2 +102.850|7.405.584|1|

O primeiro caso distante ganha apenas em material/formato de vértice; não reduz
triângulos. No horizonte denso, a geometria cai48,7% e os bytes selecionados80,8%,
mas as montanhas já têm contraste quase nulo. No caso ainda visível, a queda de
triângulos é15,0% e a de bytes68,1%; é uma condição distinta de atmosfera/qualidade,
não um resultado que se pode atribuir ao perfil padrão. As três poses de transição
com câmera/luz variando tiveram diferença máxima1RGB8. Esses são ensaios discretos,
não uma prova universal de ausência de cintilação nem aceite artístico.

`selected_vertex_bytes` mede payload de vértices; `allocated_vertex_bytes` mede
as capacidades dos dois buffers GPU (potências de dois/histerese). Nenhum inclui
render targets, profundidade, uniformes, driver ou a fonte CPU. O pacote binário
é compartilhado pelas cinco instâncias, não multiplicado por cinco.


### Tempo de desenho observado

Apple M4 Pro/Metal,1728×1117; quatro rodadas alternando a ordem dos caminhos,
12frames de aquecimento +32amostras por rodada. Valores abaixo são a mediana das
quatro medianas. Tempo CPU de seleção/upload/submissão + espera pela conclusão
GPU; não é timestamp GPU, tempo de simulação, apresentação da janela ou FPS.

| Caso | Referência completa | Selecionado | Variação observada |
| --- | ---: | ---: | ---: |
|100m|5,114ms|5,134ms|+0,4%|
|1000m|4,233ms|2,316ms|−45,3%|
|9000m, contraste quase nulo|4,592ms|1,794ms|−60,9%|
|Retorno100m|5,551ms|4,702ms|−15,3%*|
|9000m, silhuetas ainda visíveis|4,317ms|1,974ms|−54,3%|

*O retorno usa a mesma geometria e shader detalhados. A diferença de15,3% não é
uma economia estrutural desse caso: demonstra a variabilidade da medição local.
Nas duas primeiras execuções exploratórias houve compilação paralela e oscilação
maior; estão preservadas em cost-repeatability.json. Na execução final não havia
cargo/rustc observado antes/depois, mas o desktop e a janela humana permaneceram
abertos. Não promover esses tempos a garantia de plataforma ou desempenho AAA.

No percurso100→9000→100m com câmera e direção solar variando (duas sequências por
caminho), a mediana das medianas caiu4,440→1,900ms(57,2%). Porém a mediana dosp95
subiu6,428→6,900ms e o máximo observado8,101→12,469ms: reconstrução/upload nas
mudanças de representação ainda produz picos. O mecanismo melhora o custo
sustentado distante; **não resolveu a suavidade de todos os frames**. Próximo
trabalho de performance é antecipar e limitar o trabalho dessas transições,
medindo o aumento de memória temporária, antes de alegar ausência de regressão.

No caso visível, os buffers alocados passaram de32MiB para12MiB (capacidade);
payload selecionado22,16→7,06MiB, mais pacote compartilhado98,2KiB em CPU. O caso
9000m mais denso selecionou4,26MiB e alocou6MiB. A fonte CPU não foi descarregada.

Continuação em escala de quilômetros: [paisagem EAF-167](atmosphere-landscape.md).
Nesse novo cenário os ganhos anteriores não se repetiram: preservar a distinção
entre as duas cargas ao apresentar resultados.

Continuação de custo e direção visual: [EAF-168](atmosphere-cost.md), com caixas
orientadas, vértices indexados, alocação mais justa e paletas autoráveis escuras,
pôr do sol e roxo. As métricas anteriores acima permanecem históricas.
