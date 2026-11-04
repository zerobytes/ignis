# Protótipo de relevo de Land of Survivors

3 de novembro de 2026 · LOS-18 · Aceite artístico pendente.

Foi construída uma região original no planeta de 12.742 km de diâmetro para verificar o que as ferramentas atuais conseguem expressar. A estrutura já combina cristas, picos, vale, passagem e patamares, mas o resultado ainda é um estudo de formas: as encostas permanecem regulares e a aproximação no chão encontra limites de detalhe. Não representa a topografia final nem uma réplica de Genshin.

## Examinar o resultado

Abrir `games/land-of-survivors/artifacts/terrain/review.html` depois de executar o estudo. A galeria coloca o controle suave e a versão com patamares lado a lado, em cinco câmeras idênticas: vale, oeste, leste, oblíqua e aérea. Todas são capturas do renderer nativo, sem retoque ou geração de imagem. Há também uma imagem diagnóstica do primeiro ensaio serrilhado.

```sh
python3 games/land-of-survivors/terrain.py
python3 games/land-of-survivors/world.py --open
```

A abertura padrão mostra a versão publicada com patamares, em vista regional. `--view ground` e `--view globe` permitem inspecionar as outras escalas. A sessão começa pausada; P inicia/pausa, roda aproxima/afasta, F volta à superfície e G alterna globo/superfície.

## Composição própria

- Crista oeste sinuosa, encostas assimétricas e dois picos de alturas distintas.
- Cadeia leste com patamares, contrastando com a crista mais suave do oeste.
- Sela ao fundo, vale principal descendente e passagem lateral.
- Uma plataforma com núcleo plano e transição independente, composta como campo.
- Pequena ondulação de solo, sem substituir as formas principais por ruído.

O quadro de amostragem cobre aproximadamente 5 × 5 km, em torno de uma composição localizada. Coordenadas, caminhos e parâmetros foram autorados para este projeto. As formas principais têm suporte local e não atribuem os mesmos patamares ao planeta inteiro. A união suave das cadeias deixa um fundo constante de 7,03125 m fora da região, observado nas consultas dos eixos externos; portanto o campo completo ainda não retorna ao datum zero. Esse detalhe de composição deve ser normalizado antes de definir oceanos. O observador começa no vale, a cerca de 1.700 m ao norte da âncora de autoria.

A fonte publicada usa 26 nós Surface1 em um documento Planet2. As ferramentas nativas Curve, Raise, Remap, Mix, SmoothMaximum e Channel avaliam a mesma superfície usada pelo runtime e pelas consultas. A receita não lê imagens, modelos, mapas ou fontes de mundos anteriores.

## O que funcionou

As ferramentas existentes são suficientes para um primeiro bloco estrutural. Curve define a direção das cadeias e permite assimetria entre as duas encostas. Raise acrescenta picos e máscaras. Remap cria patamares regionais. Channel abre o vale e a passagem. Um grafo Raise → Remap → Mix consegue separar núcleo plano e borda sem trocar o sampler por geometria externa.

As duas versões preservam as mesmas posições e câmeras. A comparação mostra que os patamares podem ser restritos a uma encosta, mantendo outras formas suaves. Aplicá-los a todas as montanhas produzia repetição excessiva; essa era uma escolha de composição, não falta de capacidade matemática.

A fonte passou pelo save/load nativo sem alteração, e o projeto foi reaberto em outro processo. O observador percorreu 59,997 m no vale e terminou grounded, em velocidade normal de corrida. Isso valida somente esse percurso curto; não certifica todas as passagens, escarpas ou navegação futura de NPCs.

## Limites observados

### Perfil e resolução

No primeiro ensaio, faixas de altura estreitas aplicadas a todas as cristas produziram degraus serrilhados. A versão atual usa transições mais largas, patamares somente no leste e grade canônica mais fina. Essas mudanças melhoraram a leitura, mas foram feitas em conjunto; a comparação não atribui todo o efeito à resolução.

Ainda é preciso trabalhar variedade de encostas, ombros, cortes e junções. Um corredor de montanhas com o mesmo perfil ao longo de toda a curva continua artificial mesmo se não houver defeito de rasterização. Dois picos locais precisaram de campos adicionais porque Curve usa uma largura e uma altura para o percurso inteiro.

### Seleção de detalhe no chão

No build `3234fb288729f4ae`, a vista solicitou 2 px de bound geométrico. A inspeção da versão publicada retornou:

| Vista | Tiles | Vértices selecionados | Bound máximo informado | Qualidade limitada |
| --- | ---: | ---: | ---: | --- |
| Vale | 1.248 | 1.371.510 | 66,40 px | Sim |
| Oeste | 1.248 | 1.371.271 | 67,02 px | Sim |
| Leste | 1.248 | 1.371.271 | 67,02 px | Sim |
| Oblíqua | 498 | 553.208 | 1,59 px | Não |
| Aérea | 321 | 356.925 | Menor que 0,01 px | Não |

Esses números são diagnósticos de seleção, não FPS nem erro medido da imagem. O grafo tem um intervalo conservador de aproximadamente −7..596 m, enquanto a grade de probes encontrou −2..415,30 m. O intervalo global de Surface é usado pelo caminho orbital; ele não se estreita automaticamente ao valor do fundo constante nas partes do planeta fora da região. Isso ajuda a explicar a pressão sobre o LOD, sem medir um erro real de 66 px na imagem.

O bound também toma a grade canônica como referência. Uma vista aérea cumprir esse contrato não prova que qualquer parede mais fina que a grade foi reconstruída com fidelidade contínua. Precisamos de critérios locais para detalhe e transições, além da comparação de imagens.

### Controles de autoria

Os parâmetros estão em dados editáveis e passam pelo MCP. A sessão planetária atual, porém, não está integrada ao editor autoral completo: o fluxo deste estudo é editar a receita, preparar e recarregar a fonte. Não há uma experiência pronta de esculpir o perfil de uma crista em contexto nesse mundo planetário.

## Ferramentas a melhorar ou criar

| Prioridade | Ferramenta | Problema concreto | Critério para o próximo incremento |
| --- | --- | --- | --- |
| 1 | Bounds locais de Surface e refinamento regional | O chão esgota o detalhe solicitado; o intervalo global não distingue a região de relevo do restante vazio. | Mesmas fontes/poses, sem esconder o problema por névoa: bound local conservador, preservação de costuras e melhor distribuição de amostras. |
| 1 | Perfis variáveis ao longo de Curve | Altura e largura constantes exigem campos extras para picos, ombros e afunilamentos. | Altura, largura e perfil por marcos editáveis, com transições e junções explícitas; preservar fontes antigas. |
| 2 | Patamares e nivelamento com núcleo/borda separados | Um patamar útil exige montar máscaras; Remap global gera repetição e estreita faixas sem considerar o espaço disponível. | Controlar altura, extensão, dureza/transição e interrupções locais, sem replicar o mesmo degrau em todo o terreno. |
| 2 | Prévia de cristas e cortes no planeta | A autoria atual depende de recarregar e capturar para avaliar cada perfil. | Mostrar caminhos, larguras e cortes sobre a superfície e permitir ajustar pelos mesmos comandos MCP da autoria. |

Essas são conclusões para a continuidade, não ferramentas declaradas implementadas. O mecanismo radial atual fornece uma altura por direção; saliências, arcos e cavernas volumétricas exigiriam outra representação se entrassem no escopo. Não são necessários para este primeiro ensaio e não foram simulados com meshes escondidas.

## Custos e validação

Python apenas monta o grafo e organiza consultas/capturas. Não calcula alturas ou malhas, não roda no frame e não altera o renderer. O programa possui 26 nós, abaixo do limite nativo de 128; o estimador atual soma 904 unidades por ponto, ou 6.891.192 para 33 × 33 vértices com sete amostras, abaixo do envelope de 16 milhões. Essa estimativa não mede tempo de CPU/GPU.

O runtime usa nível 15, 32 subdivisões e raio físico de 128 m. A densidade da atmosfera física passou a 0,0016 para manter sua residência no orçamento; a vista orbital usa profundidade óptica explícita de 0,18 e cobre a esfera. Isso muda o custo em relação à esfera lisa: não é um benchmark antes/depois. Não se reduziu o terreno para uma esfera menor nem se ocultou a sinalização de qualidade limitada.

Os verificadores cobrem documento e dimensões, fonte nativa idêntica, observador grounded, percurso curto, preservação da simulação ao trocar câmera, cobertura das seis faces e costuras conformes. A abertura independente confirmou os arquivos publicados. Não houve novo build, cópia de executável, suíte global, alteração de engine ou importação de assets.

Capturas, fontes de comparação, relatório e galeria usam caminhos estáveis em `artifacts/terrain/`. A variante diagnóstica permanece somente por sua utilidade na investigação. A fonte atual do jogo é `world/sphere.planet.json` com seu binário nativo correspondente; os dois candidatos não são builds de aplicativo diferentes.

O próximo passo recomendado é trabalhar a riqueza das formas e a distribuição de detalhe no chão antes de ampliar o relevo para o planeta inteiro. A estrutura é examinável agora; a aproximação artística ao estilo desejado ainda precisa da avaliação do usuário. A revisão abaixo detalha por que corrigir apenas o LOD não resolve a qualidade.

## Revisão de qualidade: referência World Creator e imagem do usuário

Em seguida o usuário considerou o protótipo melhor, mas cobrou qualidade de terreno, apontou World Creator e enviou uma imagem de ambiente costeiro. Isso não constitui aceite da topografia final. O diagnóstico anterior focou excessivamente no funcionamento do planeta e no LOD; a qualidade das formas deve ser um critério próprio de continuidade.

### Por que o estudo ficou artificial

1. **Autoria simplificada.** A receita usa três curvas principais, perfis constantes ao longo de cada curva, poucos picos adicionais e remapeamento de altitude para os patamares. O ruído tem período de 1.024 m, duas oitavas e amplitude de 7 m. Não há uma composição trabalhada de cristas secundárias, ravinas, ombros, recortes e fragmentação das bordas em escalas intermediárias. O número de nós, por si só, não mede qualidade.
2. **Ferramentas existentes subutilizadas.** `src/terrain/surface.rs` já inclui `Ridge`, com `crest_wander`, `folds_meters` e perfil de encosta, mas `terrain.py` não o usa. Portanto não é correto atribuir toda a simplicidade a ferramentas ausentes. Antes de ampliar a API, precisamos explorar e avaliar esse operador e a composição de ramificações/máscaras com autoria própria.
3. **Formação do relevo incompleta.** O enum Surface1 atual não oferece erosão hidráulica/térmica, transporte de sedimentos ou acumulação de fluxo. Channel corta um leito ao longo de uma linha autorada; não calcula uma rede de drenagem. Ruído e um canal não substituem esses processos. Eles são candidatos a ferramentas novas, com intensidade e máscaras controláveis para a direção estilizada de Land of Survivors.
4. **Amostragem limitada.** O orçamento e os bounds globais dificultam representar detalhe localizado no chão. Corrigir isso melhora a representação da fonte; não inventa as formas que faltam nela.
5. **Apresentação parcial.** O estudo usa cor/material uniformes e não construiu água, rocha/solo em camadas ou vegetação. Esses componentes contribuem para o resultado final da referência. Ainda assim, o relevo precisa ser convincente sob material neutro.

Essas conclusões são sobre o caminho usado pelo protótipo. Não são uma auditoria de todos os recursos da engine nem prova de que nenhum outro subsistema tenha ferramentas semelhantes.

### O que a imagem acrescenta à meta

A imagem enviada mostra massas de terra irregulares, bordas rochosas fragmentadas, bancos rasos, canais e depressões submersas. Vegetação, areia, rocha exposta, sombras e água com diferenças visuais de profundidade reforçam a leitura. A escala física, a autoria, o software e os algoritmos usados nessa imagem não foram fornecidos; não podem ser deduzidos da captura.

A referência passa a orientar variedade de formas e transições entre terreno e água. Não define um novo mapa a copiar, não troca automaticamente o vale atual por uma ilha e não altera o escopo de construir somente o mundo. Para avaliar a contribuição da geometria, o próximo estudo precisa mostrar tanto uma vista neutra quanto uma apresentação de superfície, nas mesmas câmeras.

### Comparação com World Creator

Na [lista oficial de recursos](https://www.bitethebytes.com/products/world-creator/features), o fabricante descreve erosão e sedimentos, rios, máscaras por altura/inclinação/curvatura, escultura, materiais em camadas e distribuição de objetos. Esse conjunto reúne formação, controle artístico e apresentação. A [página do produto](https://www.bitethebytes.com/products/world-creator) também anuncia geração/iteração acelerada por GPU. São capacidades anunciadas, não benchmarks reproduzidos nesta máquina nem uma identificação das técnicas da imagem do usuário.

O contraste útil é o processo completo de autoria. Mais polígonos ou executar a mesma receita na GPU não acrescentaria por si só a organização que falta às formas atuais. O tamanho terrestre da esfera também não impede um trecho local detalhado; exige residência e detalhe regionais. Não precisamos simular toda a superfície terrestre com resolução de chão para trabalhar uma área de referência.

### Próximo incremento e critérios de qualidade

| Frente | Trabalho concreto | Evidência necessária |
| --- | --- | --- |
| Formas e direção artística | Refinar uma área pequena no planeta existente: composição principal assimétrica, ramificações secundárias, ombros, cortes e transições locais. Explorar Ridge e composição existentes antes de declarar novas APIs necessárias. | Mesmas vistas aérea, oblíqua e de chão; material neutro; silhuetas e cortes legíveis, sem corredores de perfil uniforme nem degraus regulares em toda a encosta. |
| Ferramentas de formação | Projetar um estágio regional de erosão/relaxamento e drenagem, preservando áreas de gameplay por máscaras; permitir variantes com e sem processamento. | Comparação da mesma fonte antes/depois, parâmetros explícitos, controles de intensidade, ausência de costuras e nenhuma alegação de erosão baseada apenas em ruído. |
| Representação no runtime | Bounds locais e refinamento conforme o detalhe físico da fonte, sem ampliar orçamento às cegas. | Mesma geometria de referência e mesmas poses, orçamento/tempo/memória registrados, transições/costuras e chão verificados; flag de qualidade visível. |
| Superfície e leitura | Integrar máscaras coerentes de rocha/solo/sedimento; avaliar água rasa quando uma região costeira entrar na autoria. | Comparação neutro/superfície na mesma câmera, sem usar vegetação ou névoa para encobrir falhas de relevo. |

Um estágio de erosão exige vizinhança e estado. A arquitetura recomendada é gerar/compilar dados regionais durante a autoria, com bordas de trabalho, cache por conteúdo e leitura pelo runtime; não recalcular uma simulação completa a cada consulta de altura ou frame. Esta é uma proposta para investigação, ainda não uma ferramenta implementada. A escolha de algoritmo, resolução, custo e integração nativa precisa ser medida antes de prometer execução em tempo real.

O marco seguinte deve demonstrar qualidade em uma área pequena antes de expandir continentes. Validação de colisão e persistência continua necessária, mas aprovação visual das formas é um requisito separado. Nenhuma ferramenta externa foi comprada/instalada, nenhum asset importado e o mundo publicado continua o mesmo protótipo durante esta revisão.

## Procedural e escultura compartilham as ferramentas

**Requisito explícito do usuário, 4 de novembro de 2026:** tudo que o procedural usar para construir o relevo deve ser utilizável manualmente. O terreno precisa ser gerável e esculpível, com os dois modos combináveis. Isso orienta a implementação seguinte; esta revisão não declara a escultura planetária integrada como pronta.

O catálogo de operações e seus kernels serão comuns. A receita procedural escolhe parâmetros, regiões, máscaras, sementes e ordem de composição. O uso manual fornece esses mesmos dados por pincéis, curvas, alças e controles locais. MCP e interface enviam os mesmos comandos nativos validados. O gerador também deve poder aplicar traços de pincel, em vez de existir uma família de deformações exclusiva do mouse.

| Operação compartilhada | Uso procedural | Uso manual |
| --- | --- | --- |
| Elevar, cavar, suavizar e nivelar | Aplicar operações/traços em regiões e percursos escolhidos pela receita. | Pintar traços ou ajustar uma região com raio, intensidade e transição. |
| Cristas, vales e patamares | Compor caminhos, perfis e máscaras segundo regras regionais. | Desenhar/editar caminhos e perfis, mover marcos e controlar bordas. |
| Erosão e deposição propostas | Processar regiões e máscaras definidas pelo gerador. | Pintar/selecionar a área, ajustar intensidade e executar a mesma etapa com prévia. Não precisa ser uma simulação instantânea em cada movimento do mouse. |
| Máscaras | Derivar campos de regras e combiná-los. | Pintar, proteger, apagar e combinar influência local. |

A camada de autoria deve conservar as operações, seus parâmetros, máscaras e traços. Ajustes manuais precisam de identidade e ordem explícitas, independentes da triangulação e do LOD visível. Não basta alterar vértices de um tile temporário: ele será substituído pela residência/refinamento. Operações com vizinhança podem ter dados regionais compilados e cache, mas a receita que os produz continua editável.

O fluxo obrigatório é: gerar uma base → acrescentar ajustes manuais → alterar/regenerar a base → reavaliar os ajustes conservados. Reaplicar um traço não garante uma aparência idêntica quando sua base muda. Áreas protegidas, parâmetros absolutos/relativos e a posição das camadas devem tornar essa escolha explícita; incompatibilidades exigem diagnóstico antes de substituir o resultado, sem descartar ajustes silenciosamente. A ordem importa: erosão antes de um patamar não equivale a erosão depois dele.

Estado verificado: [Surface1](../../surface-program.md) já compartilha operações/dados entre autoria e receita, em plano e esfera; editar por MCP não equivale a ter todos esses controles manipuláveis no viewport. A [escultura de meshes](../../editor-sculpt.md) tem seu próprio fluxo de traços e prévia. Ainda é preciso integrar a autoria de relevo planetário a gestos, máscaras e persistência dessas edições. O protótipo permanece baseado em receita/recarregamento. A integração deve preservar os pincéis de mesh existentes, sem prometer que todo deslocamento volumétrico se torna automaticamente uma altura radial.

Critérios do próximo incremento:

1. Mesma operação normalizada, base, ordem, resolução autoral e semente produz o mesmo resultado via UI, MCP ou receita; o adaptador de entrada não muda o kernel.
2. Prévia e aplicação coincidem; cancelar preserva a fonte; um gesto é uma unidade de desfazer/refazer.
3. Salvar/reabrir conserva parâmetros, traços, máscaras, ordem e edições manuais; regenerar a base mantém essas edições e sinaliza incompatibilidades.
4. Alternar LOD, atravessar limites de tiles/faces ou revisitar uma região não apaga escultura nem muda seu raio físico; fonte visual e colisão continuam coerentes.
5. Custo e resolução ficam explícitos, inclusive em etapas demoradas como erosão. Avaliar formas em material neutro e manter o aceite artístico separado.

Esse contrato é de construção do mundo no editor. Não acrescenta automaticamente terreno destrutível, escavação de gameplay, cavernas ou edição multiplayer ao jogo.
