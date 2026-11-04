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

O próximo passo recomendado é corrigir a distribuição de detalhe no chão e melhorar os perfis das cristas antes de ampliar o relevo para o planeta inteiro. A estrutura é examinável agora; a aproximação artística ao estilo desejado ainda precisa da avaliação do usuário.
