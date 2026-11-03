# Comparação nativa sem/com representações compiladas

Pedido de Guilherme após EAF-170: comparar o desenho completo com o máximo de
otimização disponível, sem suavização de bordas. Medição de31/10/2026, main após
`3a3493d`, renderer build`70038ee07b714dc4`, Apple M4Pro/Metal,1728×1117 pixels.

## O que está sendo comparado

- **Referência:** pacotes compilados removidos, `reduce_details=false`, geometria
  fonte e materiais completos. Rasterização/depth, caches básicos e renderer
  nativo continuam os mesmos; não é reconstrução de uma versão histórica da engine.
- **Otimizado:** ATM3 compilado em autoria, seleção automática dos níveis admitidos,
  preenchimento barato e vértices compartilhados/índices. Blur ausente. Os limites
  de qualidade anteriores ficam em0,5px de contorno e0,002 de cor linear: não
  aumentamos tolerâncias ou apagamos detalhes para fabricar ganho.
- Mesma superfície, câmera, luz, resolução e aparência atmosférica. Tirar também
  o esmaecimento da referência compararia imagens diferentes. Máximo significa
  todas as otimizações compatíveis com esta cena e seus limites de qualidade.

Este é o caminho integrado na engine. Experimentos de símbolos/campos e seleção
regional da worktree graphics-research não são recursos automaticamente somados
ao renderer principal; ver main-consolidation.md. Não declarar um Nanite completo
a partir deste ensaio. A cena tem seis instâncias: cinco cadeias sintéticas e chão,
centros horizontais a1/3/6/10/18km, câmera inicial a1200m. Sombras/IBL/rigs não
fazem parte deste conteúdo; não desligamos recursos presentes para favorecer um lado.

## Qualidade e memória

Sete vistas/paletas e12 posições com câmera/luz variando: máximo **0 RGB8** entre
referência sem pacotes e seleção. Blur zero, zero imagens/passes de suavização
em ambos. Isso cobre as amostras registradas, não todos os jogos/câmeras.

| Paisagem azul | Referência | Otimizado |
|---|---:|---:|
| Triângulos desenhados |121002|121002|
| Instâncias com material barato |0|3|
| Payload de geometria GPU |22,16MiB|10,54MiB|
| Capacidade alocada de geometria GPU |22,25MiB|10,78MiB|
| Pacotes compilados utilizados |0B|200980B|
| Imagens de blur |0B|0B|

Capacidade de geometria cai51,5%; payload cai52,4%. A fonte CPU continua residente;
os compilados e níveis decodificados têm custo CPU adicional. Isso não mede RAM
total nem garante que o driver devolveu imediatamente a memória física. Na medida
de tempo, a engine de ensaio retém a fonte e os pacotes para alternar caminhos;
o Frame de referência não apresenta pacotes ao renderer. Não usar RSS desse
processo como se fossem duas aplicações isoladas.

**Não houve redução de triângulos nesta paisagem.** O ganho vem do preenchimento,
da representação compacta e do menor trabalho de materiais. A seleção inteira por
instância ainda impede simplificar agressivamente as curvas sem alterar cristas.
O controle de ar menos influente conserva os detalhes e oferece pouco espaço para
ganho. O controle plano anterior reduz2048→1024 triângulos, mas não é extrapolado
às montanhas. Aumentar densidade/forçar cor sólida mudaria a imagem aprovada.

## Método e reprodução

`scripts/compare_atmosphere_paths.py` carrega a paisagem anterior, remove os
compilados da referência pelo MCP nativo e depois recarrega a cena compilada.
Captura sete pares estáticos e12 pares móveis, valida ausência de blur/pacotes
na referência e conserva somente o par móvel atual/pior, além das vistas estáticas.
Pré-requisito reproduzível: `scripts/verify_atmosphere_landscape.py` gera a fonte.

O exemplo existente aceita `--uncompiled-reference`, preservando os demais usos:

```sh
python3 scripts/compare_atmosphere_paths.py
cargo build --release --example atmosphere_landscape_cost
target/release/examples/atmosphere_landscape_cost artifacts/render-comparison --uncompiled-reference
```

Duas execuções, quatro rodadas com ordem de perfis/caminhos alternada; todos os
perfis aquecidos antes da medição, mais12 frames de aquecimento por alternância
e40 amostras. Cada execução também mede duas trajetórias de120 frames por caminho,
com câmera/luz móveis e transições. Referência sem pacotes força reconstrução da
cache ao voltar à seleção; o custo da alternância é registrado fora das medianas
aquecidas. Isso não é o comportamento de um jogador alternando a cada frame.

Tempo total é **CPU draw + espera bloqueante pela GPU**. CPU draw inclui seleção,
upload e submissão; não é timestamp GPU. Sem simulação, apresentação, leitura de
PNG ou compilação de assets cronometradas. Desktop/janela humana permaneceram
ativos; sem build/probe concorrente durante os tempos. Não converter em FPS de jogo.

Evidências em `artifacts/render-comparison/verification.json`,
`cost-repeatability.json`, `comparison.md` e PNGs nativos. Somente instrumentos e
documentação mudaram neste recorte, sem alterar o algoritmo de render ou formatos.
Validação proporcional: comparação nativa, build/Clippy do exemplo, fmt,
Python e diff; não foi criada nova suíte unitária para o relatório.

## Tempos observados

Mediana das quatro medianas, em ms, **referência → otimizado**. Sinal positivo
na economia significa tempo menor; negativo significa regressão.

| Perfil | Execução1 | Economia1 | Execução2 | Economia2 |
|---|---:|---:|---:|---:|
| Pouca influência |4.524 → 4.363|+3.6%|4.609 → 5.068|-10.0%|
| Azul |4.718 → 4.109|+12.9%|3.425 → 3.600|-5.1%|
| Maior influência |3.994 → 5.024|-25.8%|3.624 → 3.995|-10.2%|
| Lateral |3.562 → 3.894|-9.3%|4.151 → 4.402|-6.0%|
| Elevada |3.246 → 2.528|+22.1%|3.780 → 2.856|+24.4%|
| Pôr do sol |4.111 → 2.922|+28.9%|3.912 → 3.120|+20.2%|
| Roxo |2.843 → 3.225|-13.4%|4.699 → 4.275|+9.0%|

Movimento, mediana das duas medianas por caminho em cada execução:

| Execução | Referência | Otimizado | Economia | Pior frame referência/otimizado |
|---|---:|---:|---:|---:|
|1|3.857ms|3.166ms|+17.9%|8.315/12.566ms|
|2|3.567ms|4.652ms|-30.4%|8.708/10.595ms|

A memória e a igualdade de pixels repetiram. O tempo não sustenta ganho universal:
o pôr do sol e a vista elevada melhoraram nas duas execuções, mas maior influência
e lateral regrediram em ambas; azul/roxo/movimento inverteram a direção. Não
atribuir as regressões somente ao ruído ou somente ao algoritmo sem isolar o
gargalo. O controle de pouca influência, que conserva a geometria e materiais,
também oscilou de3,6% menor a10% maior. A máquina não estava isolada.

No azul, CPU draw caiu1,185→0,940ms e0,861→0,589ms, apesar de a segunda
execução do tempo total ter regredido. Isso distingue trabalho CPU e espera,
mas não identifica sozinho a causa. Não substituir estes resultados por um
ganho histórico mais favorável. Próxima investigação de tempo exige separar
GPU com timestamps e custo CPU/transferências, mantendo referência e qualidade.

Conclusão deste comparativo: economia comprovada de buffers e preservação visual;
aceleração total ainda dependente do cenário e sem repetibilidade suficiente
para prometer um fator único. A redução dos triângulos curvos permanece pendente.
