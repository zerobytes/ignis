# Geometria pequena na tela — EAF-175

Continuação de EAF-174: o tamanho projetado e o contraste devem governar o
trabalho, sem corte fixo em quilômetros e sem regras específicas para árvores.
Uma contribuição subpixel isolada pode afetar cobertura; muitas contribuições
juntas podem formar uma copa ou um contorno visível. Apagar cada uma por sua
altura individual não é uma prova de equivalência da floresta.

## Decisão e revisão de custo

Primeiro incremento escolhido: eliminar a repetição exata de vértices nos
uploads/desenhos estáticos, mantendo a mesma ordem de triângulos e atributos.
Este passo reduz trabalho redundante e memória sem depender de uma aproximação
visual nova. Não resolve sozinho o custo proporcional aos triângulos visíveis.

- A transformação passa a poder reutilizar o cache de vértices indexados da GPU.
  Posições, normais, coordenadas de material e todos os demais atributos entram
  na identidade binária; não soldar apenas posições em seams/arestas duras.
- Construção da tabela somente no upload de um asset/nível imutável, com cache
  invalidado pelos mesmos contratos de geração e pacote. Não há hashing por frame.
- Índices u16 quando cabem, u32 para malhas maiores; páginas só conforme os
  limites de buffer. O limite de uma página não é limite do asset ou de instâncias. Página sem economia
  de memória mantém o stream original; não reordenar faces para melhorar cache.
- Mesmas máscaras de câmera/luz, materiais, simulação, seleção e formatos.
  A variante compacta de vertex shader lê as constantes da instância. Sem campo autoral, dependência, thread, lock, I/O ou JSON no desenho.
- CPU de preparação e buffers de índices são custos explícitos. Comparar tempo
  total aquecido e bytes realmente alocados; não inferir FPS pelo número de
  vértices nem interpretar vértices únicos como contagem exata de invocações GPU.
- Reutilizar gates existentes de material, sombras, seleção, invalidação e
  visibilidade. Proteger adicionalmente fronteiras de páginas/índices: truncar
  índices altera geometria arbitrária e é uma falha de integração material.

`Renderer::set_geometry_indexing(false)` fornece o oráculo sem indexação.
`cargo run --release --example visibility_cost -- --indexing` compara ambos
com o mesmo seletor atmosférico e descarte de frustum, incluindo movimento e
resize. Artefatos em `artifacts/indexing/`. Não é um controle de qualidade novo:
o uso normal é automático, e o controle Rust existe só para diagnóstico.

## Limites em investigação

Segundo incremento: apertar o limite de projeção usando o alcance transversal
da caixa orientada na câmera. Para um ponto `(x,y,z)` e deslocamento limitado,
a mudança de perspectiva é limitada por
`f / (z_min - delta_z) * (delta_xy + delta_z * max_xy / z_min)`.
O cálculo antigo usava distância total mais raio no lugar de `max_xy`, incluindo
a profundidade nesse termo transversal. O novo intervalo continua cobrindo toda
a fonte; preserva tolerância autorada, margem20%, tratamento do near plane e
fallback de material/luz. Não trata subpixel como ausência de contribuição.

`--projection` no mesmo exemplo compara limites antigo/novo; o controle de
diagnóstico é `set_precise_lod_projection`. Esta comparação pode mudar pixels
porque seleciona outros níveis dentro do mesmo orçamento; precisa publicar
erro visual e contornos, além do custo. A indexação é medida separadamente.

Inspeções expõem `instancing.camera_footprint`: seis faixas do maior lado da
caixa projetada (`<=1, <=2, <=4, <=16, <=64, >64px/indeterminado`), com
instâncias, triângulos fonte e efetivamente desenhados. Contagem por componente,
não por árvore lógica. Margem numérica conservadora mantida; exclui sombras e
draws legados. Nenhum triângulo é rejeitado por esse diagnóstico.

Os resumos ATM atuais não autorizam simplificar material geral nem garantem
preservação de cobertura integrada de folhas desconectadas. Geometria pequena
exige comparar cobertura/contorno em movimento, incluindo conjuntos densos.
O próximo corte deve ser escolhido pela distribuição de custo por tamanho
projetado; não basta contar instâncias distantes.

## Atributos constantes e fronteira da indexação

O stream detalhado compartilhado passa de64 para48bytes por vértice: posição,
normal, coordenada de material e pesos continuam f32, sem quantização. Cor branca
e índice de material constantes do asset saem do vértice e ficam na instância,
que continua ocupando80bytes. A elegibilidade é conferida no upload; dados que
não obedecerem ao contrato usam o formato64bytes. O caminho atmosférico conserva
12bytes por posição. Índices usam u16/u32 conforme o alcance e só existem quando economizam bytes.
Contadores separam bytes de índices e vértices, além do total.

A indexação irrestrita revelou uma diferença entre o primeiro e o segundo desenho
na floresta parada: máximo11RGB8 em3797canais da vista próxima. Os dados/contadores
CPU eram iguais. Oito capturas de cada combinação separaram o efeito: sem índices,
ambos os layouts eram estáveis; com índices, ambos divergiam após o primeiro frame.
Derivadas Fine e anotação invariant não resolveram; não foram mantidas. A causa
interna do backend não está estabelecida. A divergência aparece no terreno com
material procedural; não é correto anunciar esse ensaio como equivalência exata.

Decisão conservadora: material detalhado com pattern/secondary_pattern/detail
mantém stream não indexado. Se a seleção atmosférica já substituiu o material,
a geometria de posição pode ser indexada. Materiais neutros também são candidatos.
O seletor de páginas ainda recusa indexação sem economia. Isso é genérico e não
identifica árvore ou montanha por nome/espécie. `--stability` reproduz32capturas
(4combinações ×8) sem mover câmera ou alterar dados. Não aquecer e ignorar a
primeira imagem para esconder a diferença.

O limite de projeção mais preciso fica **desligado por padrão**. No ensaio ele
reduziu~10–13%dos triângulos mas somente~3–5%do tempo e alterou pixels de alto
contraste (máximo137RGB8 em movimento). Só um limite de deslocamento de posição
não certifica cobertura/cor de folhas desconectadas. Essa aproximação não entra
na otimização promovida.

## Revisão estática posterior

1. Trabalho novo no upload de conteúdo, sem fase de simulação/tick adicional.
2. Páginas obedecem aos limites do device; u16/u32 adaptativo não limita objetos.
3. Deduplicação temporária por bits, com todos os atributos; não roda por frame.
4. Buffer de posição/material mantém valores f32 e ordem de triângulos. Índices
   e remoção de constantes reduzem transporte; não há descompressão por pixel.
5. Cache usa geração/conteúdo/pacote e política de material; controles diagnósticos
   invalidam somente recursos derivados. Fonte e formatos persistidos preservados.
6. Sem nova dependência, lock, worker, rede ou alocação por triângulo em runtime
   aquecido. Contagem por escala adiciona trabalho por instância, medido no submit.
7. Preserva máscaras independentes de câmera/sombra, barreiras e ordem em empates
   de profundidade; não representa objetos ocultos como vazios no mundo.
8. Fallback para materiais procedurais detalhados e páginas sem economia; projeção
   experimental desativada. Não esconder regressão numérica por um limite mais frouxo.
9. Prova crítica remonta todos os atributos e triângulos através de páginas;
   demais gates reutilizam regressões nativas de render/material/sombra/seleção.
10. Mantidos conscientemente: memória CPU de autoria, preparação no primeiro upload,
    processamento de instâncias visíveis e custo de triângulos restantes. Não é ainda
    descarte por oclusão nem resumo certificado da cobertura coletiva subpixel.

## Ordem entre instâncias e páginas

A prova GPU com duas superfícies coplanares sobrepostas e66.049vértices encontrou
1.422pixels com prioridade errada quando páginas u16 de todas as instâncias eram
desenhadas antes da próxima página. Correção: malhas grandes usam u32, evitando
fragmentar desenhos só pela largura do índice; quando o limite real do device
exigir páginas, terminar todas as páginas de uma instância antes da próxima.
Isso preserva empates de profundidade. O custo de draws adicionais nesse caso
extremo é explícito; a cena de floresta usa páginas únicas por recurso.

## Próximo corte orientado pelos dados

Na vista próxima, os bounds de componentes até4px respondem por7,4%dos
51,48milhões de triângulos selecionados. A faixa4–16px responde por69,3%;
4–64px, por84,3%. São limites conservadores na resolução física2048×1536,
contados por componente casca/folhas, não por árvore lógica.

O próximo experimento deve compilar **cobertura e silhueta por região/escala**
para essa faixa dominante. Um componente3×1px ainda pode alterar três amostras
contra o céu; muitos juntos formam um contorno. O objetivo é substituir esse
conjunto por uma representação barata da contribuição visível, e dispensar
materiais/folhas individuais quando o erro final permitir. Comparar com a mesma
referência, em movimento, zoom/resolução e luz, sem linhas, buracos ou cintilação.
Descarte de oclusão também segue aberto. Não foi implementado um corte por3px
ou10km, nem um novo formato de cobertura, nesta unidade.

Para reproduzir: `cargo run --locked --release --example visibility_cost -- --combined`
e `--stability`. O comparativo inclui seleção atmosférica/frustum em ambos os
lados: mede o incremento sobre EAF-174, não sobre renderização sem otimização.
Capturas/readback não entram nos tempos. Três rodadas alternadas com8aquecimentos
+16amostras; CPU inclui submissão/driver, draw+espera inclui conclusão da GPU.
Nenhum dos dois mede FPS de gameplay ou GPU isolada. A janela humana é preservada.

## Resultado final — 31/10/2026

Build `640c86a4cc5c60d0`, macOSarm64/release, Apple M4 Pro/Metal,2048×1536.
Mesma floresta10mil árvores/20mil componentes, materiais/luz e blur desligado.
Referência é o caminho EAF-174 com frustum/agrupamento/seleção atmosférica ativos;
a única diferença é indexação elegível + atributos constantes por instância.

| Vista | Antes (ms) | Depois (ms) | Redução | CPU antes/depois (ms) |
|---|---:|---:|---:|---:|
| Próxima |54,765|38,315|30,0%|1,665 /1,659|
| Panorama |70,607|46,370|34,3%|2,279 /2,198|
| Olhando para trás |4,230|4,251|−0,5%|0,436 /0,459|

Tempos são medianas das três medianas por configuração, draw+espera, sem
captura/readback. A diferença de0,021ms na vista leve não demonstra ganho.
A CPU oscilou entre execuções; um ensaio anterior apresentou+0,14–0,26ms de
submissão nas vistas pesadas. Não afirmar aceleração de CPU: o ganho robusto
medido é no tempo total. Não converter esses números em FPS do editor/gameplay.

Buffers geométricos compartilhados nas vistas pesadas:16.541.940→9.948.316bytes,
**15,78→9,49MiB (−39,86%)**, incluindo índices e padding. Na vista reversa,
4.646.400→3.484.800bytes. Não são RAM total, memória unificada total, texturas,
instâncias nem memória da autoria CPU. Fonte e quantidade de triângulos escolhidos
permaneceram iguais:51.480.711 perto,86.796.838 panorama,24.200 reversa,
nesses contadores de meshes compartilhados; chão legado e caixa ficam à parte.

Qualidade: zero pixels/canais RGB8 diferentes nas três vistas principais e oito
poses de câmera com alternância retrato/paisagem.32capturas estáticas nas quatro
combinações de índice/layout também são idênticas desde o primeiro desenho.
As três capturas EAF-174 guardadas antes desta unidade também coincidem
exatamente com a saída final (`prior-visibility-pixels.json`). O gate compara
a referência sem alinhamento, reescala ou tolerância relaxada.
Inspeção visual nativa conserva floresta, terreno e contornos; isso não é prova
universal para todo material/GPU nem aceite humano de FPS.

Validação:16testes Rust em lib+4alvos, incluindo três provas GPU e as duas novas
proteções críticas de índices/paginação/ordem; seis pares shared/expanded com
materiais, sombras, seleção e atmosfera tiveram erro0. Clippyalltargets
`-D warnings`, fmt, buildrelease, Python e diffcheck aprovados. Quatro roteiros
MCP/18grupos (render,materials,directional_shadows,atmosphere) passaram em workspace
isolado, com capturas repetidas/invalidação/undo/reset/save/load/replay conforme
cada roteiro. A rotina agregadora foi bloqueada antes de executar: quatro scripts
preexistentes (`atmosphere_edges`, `atmosphere_landscape`, `mesh_resources`,
`silhouettes`) estão fora deSCENARIOS e alguns não têm a interface/fixtures da
suíte. Executamos diretamente os quatro roteiros selecionados, sem alterar suas
asserções. A integração desses scripts ao agregador permanece pendente; não foi
alegada regressão completa.

Reprodução/evidências: `artifacts/geometry-cost/{cost,summary,motion,mcp-report}.json`,
`artifacts/geometry-stability/stability.json`, logs e PNGs nos mesmos diretórios.
O relatório antes da proteção procedural e o teste de ordem antes da correção
ficam preservados para investigação. Executável corrente `target/release/ignis`;
nenhuma janela humana reiniciada, câmera/cena alterada, binário histórico,
instalação adicional, push ou mudança de Scene103/replay113/protocolo5.
