# Silhuetas atmosféricas: custo e direção visual — EAF-168

Registro histórico. Guilherme rejeitou o céu escuro desta paleta: o alvo é um
céu claro com montanhas escuras e pouco detalhe interno. Correção e medições
atuais em [EAF-169](atmosphere-silhouettes.md); os artefatos correntes e scripts
reproduzem a revisão nova. Os números abaixo pertencem ao build indicado.

Direção de Guilherme em31/10: priorizar redução de custo; montanhas distantes
quase sólidas, com menos variação de material e tons mais escuros. Cor do ar deve
ser autorável, inclusive pôr do sol/nascer do sol e mundos roxos. Remover limites
de distância é requisito futuro, explicitamente adiado nesta unidade.

## Aparência e configuração

A cena de `scripts/verify_atmosphere_landscape.py` mantém o relevo e os materiais
originais de EAF-167. A mudança visual é uma configuração autoral aplicada tanto
à referência completa quanto ao caminho selecionado. Não se reduz a qualidade
da referência para medir uma economia artificial do seletor.

| Perfil | Densidade por metro | Cor do ar, RGB linear | Cor solar | Influência |
| --- | ---: | --- | --- | ---: |
| Azul escuro |0,00035|0,075 /0,13 /0,21|0,55 /0,32 /0,16|0,15|
| Pôr do sol |0,00035|0,16 /0,075 /0,10|0,80 /0,24 /0,055|0,65|
| Roxo |0,00035|0,13 /0,055 /0,24|0,45 /0,17 /0,55|0,25|

Ar limpo0,000035 e névoa forte0,0007 continuam como controles. As cores não são
constantes universais da física; são escolhas do jogo. A direção/intensidade
solar também participa da cor resultante. Sem nuvens ou partículas adicionais.

Com densidade0,00035, a transmissão `exp(-densidade * distância)` conserva
70,5% do contraste a1km,35,0% a3km,12,2% a6km,3,02% a10km e0,184% a18km.
A20km seria0,0912%; a30km0,00275%. Estes dois últimos são valores da fórmula,
não capturas nem prova de mundo de30/40km. O recorte visual continua em18km.

UI: **Iluminação → Atmosfera da cena → Cor do ar / Cor solar / Influência solar**,
além de densidade. MCP: `set_light` com os mesmos campos `light.atmosphere`.
`reduce_details` só escolhe a representação; desligá-lo preserva a mesma
aparência autorada e permite comparar contra a geometria/material completos.
Não é preciso novo formato ou comando para céu roxo. Alterar a luz não recompila
os pacotes; a escolha é reavaliada em runtime.

## Mudança de custo

- A distância mínima passa a usar a caixa orientada da malha, com margem numérica,
  em vez da esfera. Uma montanha larga não fica artificialmente perto da câmera
  apenas pela sua largura. O bound de profundidade também usa essa caixa; mantém
  limites conservadores e a tolerância de0,5px físicos/0,002cor linear.
- O preenchimento barato reutiliza os índices da fonte ou do nível compilado.
  Vértices24B compartilhados +índicesu32, sem deduplicação/hash em runtime.
  Material original não é avaliado nesse caminho. Contagem de triângulos pode
  permanecer igual mesmo com menos dados/vértices processados.
- Buffers grandes passam a usar blocos de4096elementos, com margem de25%
  para encolher; buffers pequenos conservam potências de2 sem um mínimo de4096.
  Isso limita a sobra que antes retinha32MiB mais o buffer barato. Reentrada pode
  realocar: medir frames de transição além do custo aquecido.
- A detecção de uploads compara os bytes contíguos destinados à GPU, evitando
  igualdade campo a campo de milhões de componentes float. Continua sendo uma
  leitura linear, mas não avalia cada campo nem introduz hash/colisão de cache.
- Estatísticas distinguem vértices, índices, payload total e capacidade GPU total:
  `selected_geometry_bytes` e `allocated_geometry_bytes`. Não incluem targets,
  profundidade, uniformes ou driver. Fonte editável CPU e pacotes ATM2 permanecem.

## Validação

Build `e72dfbed06aa1d0d`, macOSarm64/Metal/M4Pro,1728×1117pixels físicos.
Sete pares nativos: ar limpo, azul escuro, névoa forte, câmera lateral/sol,
câmera elevada, pôr do sol e roxo. Erro máximo0 no ar limpo/lateral e1/255 nos
outros cinco. Maior erro médio0,003192/255 por canal. Referência e seleção usam
exatamente a mesma superfície/material/atmosfera; a referência mantém todos os
cálculos. Isso valida essas vistas, não ausência universal de cintilação.

| Caso | Instâncias baratas | Payload vértices+índices | GPU antes (seletor antigo) | GPU agora |
| --- | ---: | ---: | ---: | ---: |
| Ar limpo / câmera lateral |0|22,16MiB|32,00MiB|22,25MiB|
| Azul / elevada / pôr do sol / roxo |1|18,28MiB|32,00MiB|18,41MiB|
| Névoa forte |2|14,41MiB|35,00MiB|14,72MiB|

Comparação do allocator+seletor antigo com o atual, **com as mesmas novas cores e
densidades nos dois builds**. Economia de capacidade42,5% no azul e57,9% na
névoa forte. Dentro do renderer atual, referência completa aloca22,25MiB;
a seleção economiza17,3%/33,8% adicionais nesses dois casos. Não somar os
percentuais. O ganho do ar limpo é alocação mais justa, sem simplificar sua imagem.

Ainda são121.002triângulos: nesta malha ampliada, nenhum nível geométrico
satisfaz o orçamento de contorno. O preenchimento corta material e permite
vértices compartilhados: cada cadeia selecionada passa de72.600vértices completos
(4.646.400B) para12.321vértices24B +72.600índicesu32 (586.104B).
O pacote compartilhado continua200.980B; a fonte CPU não foi descarregada.
Não confundir redução de bytes/vértices repetidos com corte de triângulos.

Build, Clippy de lib/binário/exemplo, quatro testes críticos existentes
(atmosfera+ambiente), seis grupos MCP do probe atmosférico anterior e os sete
pares novos passaram. Zero/ausência preservam pixels anteriores; undo/replay,
retorno de câmera e simplificação do cenário anterior continuam cobertos. Sem
novos testes unitários para detalhes visuais. Capturas PNG inspecionadas diretamente;
comparação estática em `artifacts/atmosphere-landscape/comparison.md`.
HTML gerado continua auxiliar; sua execução em navegador não foi validada.
Arte escolhida ainda depende de aceite humano.

## Continuação e limites adiados

A caixa é por instância; ainda não é seleção regional. O relevo desta carga tem
apenas110×110células por asset, ampliadas para quilômetros; simplificar seu
interior sem deslocar a silhueta requer regiões/certificados mais específicos.
Não relaxar o erro de contorno silenciosamente só porque a textura esmaeceu.

Remover `FAR=30_000` e as restrições locais de posição±10.000m é trabalho futuro
exigido pelo usuário. Coordenar precisão/origem relativa, profundidade/picking e
cobertura contínua/streaming. Não basta aumentar uma constante e declarar mundo
arbitrariamente grande. Sombras, reflexos, física e simulação têm contratos
próprios; esta otimização é da representação vista pela câmera.


## Tempo medido e conclusão

Quatro execuções do build final, cada uma com quatro rodadas alternadas por
caminho/caso,12aquecimentos+40amostras. As duas últimas também separam a duração
CPU de `draw` da espera por conclusão. Não havia cargo/rustc/probes de captura
concorrentes; desktop e janela humana preservados. A espera inclui agendamento,
GPU e driver: **não é timestampGPU nem FPS de jogo**. Compilação/preparação da
fonte, simulação, apresentação e readback não entram neste cronômetro.

Medianas das quatro medianas por execução, emms. Pares abaixo são
referência completa → seleção, nas duas execuções instrumentadas finais:

| Caso | CPU execução3 | CPU execução4 | Total execução3 | Total execução4 |
| --- | --- | --- | --- | --- |
|clear|1.093 → 1.211|1.206 → 1.150|4.256 → 5.132|4.271 → 4.683|
|haze|1.176 → 1.035|1.244 → 1.014|4.359 → 4.138|5.016 → 5.250|
|mist|1.089 → 0.979|1.136 → 0.835|4.151 → 4.475|4.221 → 3.912|
|move|1.137 → 1.227|0.805 → 0.651|4.263 → 6.430|4.094 → 3.739|
|high|1.266 → 1.014|1.034 → 1.194|5.528 → 4.765|4.188 → 4.887|
|sunset|1.176 → 0.884|1.117 → 1.047|4.157 → 3.611|4.255 → 4.655|
|purple|1.146 → 0.963|1.189 → 1.056|4.250 → 4.103|4.286 → 4.159|

CPU do azul caiu12,0%/18,4% nessas duas execuções; névoa forte10,1%/26,5%.
Há variabilidade inclusive nos controles sem substituição. O tempo total do
azul melhorou numa execução e regrediu na outra; névoa forte também alternou.
Não há ganho estável de FPS comprovado. Comparação com o build anterior foi
salva sob a mesma nova aparência (`baseline-cost.json`, buildf3075f4a95af348a),
mas não sustenta atribuir diferenças totais entre execuções exclusivamente à
mudança do renderer. Benefício confirmado: memória e remoção de trabalho/dados
redundantes; duração CPU pareada sugere benefício, sujeito à variabilidade.

Movimento: câmeraZ9000→5000→9000, lateral±400m, altura1200±150m, direção solar
variando. Duas sequências por caminho em cada execução,120frames medidos com
cruzamento de limiares, incluindo reconstrução/upload. Resultados globais:
- Referência: mediana das medianas 4.654ms; mediana dos p95 7.834ms; máximo observado 10.691ms.
- Seleção: mediana das medianas 4.275ms; mediana dos p95 7.744ms; máximo observado 16.365ms.

Transições ainda têm picos: não declarar ausência de regressões nem estabilidade
visual temporal universal a partir de capturas estáticas. Próximo passo de
medição: timestampsGPU e perfil por fase, mantendo tempo total como gate.
Próximo passo geométrico: regiões menores com contorno/costuras certificados,
para que o erro de uma crista não obrigue todo o interior a conservar geometria.
A hierarquia regional não foi implementada nesta unidade.

Reproduzir: `python3 scripts/verify_atmosphere_landscape.py`, depois executar
`target/release/examples/atmosphere_landscape_cost` com build concluído e sem
outros probes. Evidências atuais: `verification.json`, `cost.json`,
`cost-current-repeatability.json` (quatro execuções; primeiras duas sem faseCPU),
`baseline-verification.json`, `baseline-cost.json`, logs e PNGs em
`artifacts/atmosphere-landscape/`. Nenhum novo binário histórico foi guardado.
