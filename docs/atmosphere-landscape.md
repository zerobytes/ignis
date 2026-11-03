# Paisagem atmosférica em quilômetros — EAF-167

> Registro histórico de EAF-167. O gerador e benchmark mantidos evoluíram em
> [EAF-168](atmosphere-cost.md), com sete perfis/vistas e cores mais escuras.
> Os resultados abaixo descrevem o commit ed636ac, não o renderer corrente.

Continuação do teste EAF-165, autorizada após explicitar que a imagem anterior
tinha cadeias a41..408m. Esta receita usa metros nativos, sem mudar a escala das
unidades da engine, o shader, os budgets, o seletor ou os formatos.

## Escala e desenho

Observador inicial `[0,1200,9000]`, alvo `[0,1300,0]`, FOVvertical55°.
Centros das instâncias emX0/Y0 eZ8000/6000/3000/−1000/−9000.

| Cadeia | Profundidade horizontal | Câmera até base central | Largura nominal | Altura máxima |
| --- | ---: | ---: | ---: | ---: |
|1|1km|1,562km|4km|868m|
|2|3km|3,231km|5,6km|1.205m|
|3|6km|6,119km|8,8km|1.909m|
|4|10km|10,072km|13,2km|2.840m|
|5|18km|18,040km|20km|4.339m|

Larguras são antes da pequena rotação autoral. Distância a uma encosta/cume
difere da distância ao centro da base; a tabela não rotula todos os pixels de
uma cadeia com uma única distância. Alturas são relativas ao plano de base,
não dados geográficos de montanhas reais. Câmera livre, sem personagem físico.

Dois assets distintos, cada um com12.321vértices/24.200triângulos, reutilizados
em cinco instâncias +chão2triângulos:121.002triângulos. Formas e materiais
procedurais determinísticos. O relevo tem apenas110×110células por asset: a
distância entre amostras cresce com a escala, e vários triângulos continuam
ocupando muitos pixels. Não é uma carga de terreno AAA com micropolígonos.

Scene100/replay110/conteúdo2 permanecem. As posições de câmera/instância
obedecem ao limite existente±10.000m; o plano distante do renderer é30.000m.
O intervalo local de20km permite a vista de18km usando origemZ9000.40/100km não
foram implementados nem simulados com unidades fictícias. A infraestrutura
global/world_space existente ainda não move/rebaseia esse renderer. Modelo
plano, sem curvatura terrestre, refração, nuvens, densidade por altitude,
streaming ou física do terreno inferidos.

## Comparação justa e resultados

Referência e candidato usam a mesma geometria fonte, materiais, luz, atmosfera
e1728×1117pixels físicos. O único toggle é `reduce_details`. Mantidos os limites
0,5px de contorno e0,002 de cor linear; não relaxados para obter economia.

| Caso | Densidade/m | Metade do contraste | Instâncias com fill | Redução de triângulos | Erro máximoRGB8 |
| --- | ---: | ---: | ---: | ---: | ---: |
|Ar mais limpo|0,000035|19,80km|0|0|0|
|Névoa moderada|0,00015|4,62km|0|0|0|
|Névoa forte|0,0007|0,99km|1|0|0|
|Câmera lateral +outra luz, moderada|0,00015|4,62km|0|0|0|
|Vista elevada a2.000m, moderada|0,00015|4,62km|0|0|0|

São perfis artísticos do mesmo modelo homogêneo, não meteorologia calibrada.
As capturas de cada par tiveram pixels idênticos. Essa igualdade nas quatro
vistas sem troca confirma principalmente o fallback; não prova uma nova
aproximação eficaz. A névoa forte troca só o material de24.200triângulos e mantém
96.802detalhados. PayloadGPU22,16→19,39MiB(−12,5%), mas a capacidade dos buffers
**cresce32→35MiB**: o buffer completo ainda precisa32MiB e o fill acrescenta3MiB.
É uma limitação real da alocação/histerese atual. PacotesATM2 somam200.980bytes
(196,3KiB) emCPU, compartilhados; a fonteCPU também continua residente.

Medição emAppleM4Pro/Metal, quatro rodadas alternadas por caso,12aquecimentos+
40amostras, drawCPU+espera daGPU sem PNG/readback/simulação/apresentação. Foram
três execuções; a segunda coincidiu inicialmente com3,23s deClippy e não serve
de referência de performance. Desktop/janela humana permaneceram ativos.

Nas execuções1e3, medianas das rodadas variaram assim, emms:

| Caso | Completo, execuções1/3 | Selecionado, execuções1/3 |
| --- | ---: | ---: |
|Ar mais limpo|5,430 /4,209|5,933 /4,199|
|Moderada|6,006 /4,360|5,408 /5,231|
|Forte|4,013 /4,483|3,893 /4,203|
|Lateral/luz|3,893 /4,727|3,945 /4,818|
|Elevada|3,903 /6,931|5,685 /4,786|

Há variação grande até nos casos que desenham os mesmos buffers/shaders.
O estudo **não confirma um ganho agregado confiável**, nem permite atribuir
essas oscilações exclusivamente ao seletor. Não repetir a alegação de54% do
cenário anterior para esta paisagem. Tempos brutos preservados com condições
em `artifacts/atmosphere-landscape/cost-repeatability.json`.

## O que o ensaio revelou

O seletor considera a esfera da cadeia inteira. A cadeia distante tem20km de
largura e4,3km de altura: seu ponto potencialmente mais próximo continua com
contraste suficiente para conservar o material, e seu erro geométrico projetado
continua acima do orçamento. Distância do centro sozinha não certifica perda de
detalhe. A captura ampliada também conserva detalhe visível porque o número de
triângulos não aumentou na mesma proporção que o tamanho físico.

Próximo experimento sugerido: subdividir a fonte em regiões com cobertura e
costuras conservadas, certificar contribuição/contorno por região e medir a
ocupação real dos buffers. Incluir malha mais densa em regiões localizadas para
separar “geometria que ainda aparece” de overhead do seletor. Esta entrega não
implementa esse novo compilador nem aumenta budgets de produção.

## Reprodução e evidências

1. `python3 scripts/verify_atmosphere_landscape.py` gera a cena, cinco pares de
   capturas, `verification.json`, `index.html` e `comparison.md` em
   `artifacts/atmosphere-landscape/`.
2. `cargo run --locked --release --example atmosphere_landscape_cost` mede as
   exatas câmeras/luzes do manifesto; escreve `cost.json` sem sobrescrever165.
3. `--draft` no script faz apenas a vista moderada para inspeção rápida, sem
   atualizar o comparador. Usar a execução completa antes do benchmark final.

Binário e exemplo usaram buildf3075f4a95af348a/macOSarm64. Gate direto de cinco
comparações, validação nativa de todos os assets/cena/pacotes, build/Clippy do
exemplo, Python/sintaxeJS/referências de imagens e diff aprovados. Nenhum teste
unitário novo para o gerador visual; nenhum fonte de runtime foi modificado.
ComparadorHTML não foi executado no navegador: a política de URLs locais
bloqueou sua abertura automática. CapturasPNG foram inspecionadas diretamente;
versão estática emMarkdown permite ler a comparação sem executarJavaScript.
