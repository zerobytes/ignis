# Cadeias e vale da demo planetária

**Aceite artístico reprovado por Guilherme em03/11.** O relevo ficou mais rico,
mas a família de calotas não reproduziu as cristas dos testes do nanites.
Diagnóstico, comparação nativa das otimizações e contrato de correção em
[planet-appearance-audit.md](planet-appearance-audit.md). Os testes funcionais
abaixo não significam que a aparência foi aprovada.

`examples/earth-mountain-chains.recipe.json` é a receita padrão de
`scripts/open_planet_walk.py`. Mantém Terra12.742.000m/seed731199 e o runtime
esférico existente. A receita anterior pode ser preparada explicitamente:

```sh
python3 scripts/open_planet_walk.py --recipe examples/earth-procedural.recipe.json
```

## Composição autoral e procedural

Duas linhas editáveis de pontos XZ, com cerca de28km de comprimento, orientam
as cadeias ocidental e oriental. Nove estações por cadeia distribuem ombros
sobrepostos; seis núcleos menores variam os cumes. Quatro depressões compõem
o fundo do vale, e três camadas de noise global acrescentam relevo de30km,
3,3km e650m. A seed controla alturas/larguras dos ombros/cumes e os noises.

`planet_mountain_recipe.py` apenas traduz essa composição fria para31operações
Planet1 existentes. Não introduz um segundo sampler, malha Python ou shader.
O documento compilado é editável/independente da receita e os mesmos operadores
servem a autoria fixa. `assets/planets/earth-mountains.ignis-planet` conserva a
fonte nativa canônica desta seed, usada também na regressão de locomoção.
É composição de relevo, não simulação de erosão, rios ou geologia universal.

## Resolução e residência

Nível12/N64 dá aproximadamente49m entre amostras no centro da face; a resolução
varia na esfera cúbica. Formas principais têm centenas/milhares de metros.
Física e visual continuam usando a mesma triangulação completa. Essa não é
uma promessa de microrelevo, solo de resolução métrica ou LOD regional adaptativo.

Atmosfera0,00065 aumenta a distância de cobertura calculada pelo bound nativo;
a janela de residência acompanha essa mudança, sem corte manual mais próximo.
O preparador consulta26direções (centros, bordas e cantos das faces) e rejeita
configurações que excedam2M vértices fonte nesses pontos. É uma verificação
amostral de capacidade; `resources_installed:false` não vira prova de caminhada.
O orçamento do runtime continua sendo revalidado a cada mudança de residência.

## Contato de cristas

A tentativa com triângulos de49m expôs um bloqueio aos3013,32m: o SAT retornava
normal lateral de aresta interna numa crista suave. O input ainda pedia600m/s,
mas a correção física era quasezero; amostras de terreno próximas mostraram
inclinação pequena. Não era uma parede nem motivo para remover colisão.

O dispatcher planetário usa agora a normal da superfície do triângulo,
conservando o SAT finito para tempo/extensão da colisão. Isso depende da
pré-condição explícita deste adapter: campo de altura planetário contínuo,
com cobertura física além de todo o sweep. Arestas internas e costuras de tiles
não são paredes. **Malhas finitas/desconectadas não recebem essa política**;
o caminho de contatos legado e suas normais de aresta permanecem intactos.
O teste físico com esta fonte atravessa a crista sem reposicionar o ator.

## Checklist estático de performance

- Receita tem31operações, abaixo do teto32; orçamento é verificado antes de
  alocar estações. Fonte canônica tem cerca de3,4KiB.
- Não há geração, Python/JSON, I/O ou worker novo porframe. O sampler Rust
  existente prepara operações; máscaras/caches/ATM continuam nativos.
- Mais operações por amostra aumentam o custo frio; nível12/N64 limita os
  vértices residentes ao ampliar o horizonte. Não inferir FPS dos contadores.
- A correção do contato acrescenta uma escolha de normal, sem alocação,
  consulta extra, alteração de passo, margem ou retirada de triângulos.
- Capturas e consultas são explícitas; uma única janela/executável corrente.

Verificação corrente em `artifacts/planet-runtime/verification.json`: perfis
transversais nativos, igualdade de amostras, mudança de seed, fonte roundtrip,
capacidade amostral, percursos reais e capturas de vale/cumes/terceira pessoa.
Aceite artístico humano permanece separado dos testes de funcionamento.

## Validação desta rodada

Build **e46d5879c672aec9**, seed731199. Cinco perfis transversais,55amostras
nativas reproduzíveis: alturas entre−531,20 e1765,89m em relação ao datum do
spawn (não nível do mar), com cristas entre841,59 e2067,58m acima do vale.
Trocar a seed muda o relevo relativo em até983,03m nesses pontos. Fonte nativa
salva/carregada preserva o documento.

Os26pedidos de residência tiveram máximo423tiles/1.787.175vértices fonte,
abaixo do orçamento2M. No spawn real:88tiles visuais/4físicos e raio visual
14.171,29m. Captura1280×800:720.896triângulos fonte,229.376selecionados,
16substituições atmosféricas e15.753.408bytes de geometria selecionada;
isso não mede memória total nem FPS.

Percursos reais ao norte/sul:14.862,90m e14.508,28m da origem (caminhos
acumulados15.103,74m e15.247,62m), grounded ao final. Entraram138/137tiles e
saíram48/45. A crista que travava aos3km foi atravessada. Capturas de vale,
duas faces laterais, percursos e terceira pessoa foram inspecionadas.

29testes release focados passaram:4planet_runtime incluindo regressão e volta
terrestre,20character_controller,4locomotion e1unitário de contato finito.
Clippy selecionado-Dwarnings, fmt, Python AST e diff aprovados. Não se repetiu
a suíte gráfica inteira: o renderer não foi alterado neste incremento, e a
nova composição passou por captura/inspeção nativa e seus adapters normais.

Demo anterior93648 fechada após validação; nova demo **PID10923/PPID1** aberta
em PLAY/primeira pessoa, AX e screenshot nativos conferidos. Preservar essa
janela até uma substituição coordenada. Executável corrente único em
target/release/ignis, symlink Ignis Play, sem cópia extra de build.
