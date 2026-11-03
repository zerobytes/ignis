# Reuso de geradores e receitas de geografia

Pesquisa de03/11/2026. Pedido de Guilherme: mesmas ferramentas e geração para
plano/esfera, com diferença somente no domínio geométrico; avaliar criatividade
reutilizável de Tectonic e geradores gratuitos. Esta unidade entrega decisões e
inventário, sem modificar código, cena, materiais, licença da Ignis ou build.

## Um programa de terreno, dois domínios geométricos

Ferramentas, receitas, seed, edição e MCP pertencem ao programa comum. O domínio
fornece as operações geométricas necessárias: posição precisa, vertical local,
distâncias/direções, vizinhos, área e fronteiras. A implementação plana ou esférica
desses cálculos não deve retirar ferramentas do usuário nem criar dois catálogos.

Uma largura1000m significa1000m nos dois casos. Curvas/rios atravessam fronteiras
de partição sem ganhar outra identidade. Faces do cubo não são regiões geológicas.
Erosão/fluxo precisam da métrica e da vizinhança corretas, não de lógica autoral
separada. Um planeta fechado e um plano ilimitado têm fronteiras diferentes;
portanto não prometer mapas globais idênticos para a mesma seed. Exigir paridade
de capacidades, unidades e regras, continuidade e coerência local.

Camadas da proposta:

1. Receita editável comum, manual ou produzida por um gerador/importador.
2. Compilação/validação em programa Rust tipado e estágios regionais preparados.
3. Domínio plano/esférico fornecendo geometria e vizinhança.
4. Campos de altura, densidade3D, máscaras e redes geográficas conforme a receita.
5. Geometria física/visual e representações Horizon derivadas dessa fonte.

Documentos Terrain2/Planet1 existentes precisam continuar legíveis. A migração,
formatos/save/replay e contratos de erro serão uma unidade de implementação própria.

## Candidatos conferidos

| Fonte | Aproveitamento recomendado | Linguagem/formato e licença consultada | Trabalho necessário |
|---|---|---|---|
| [Tectonic v3](https://github.com/Apollounknowndev/tectonic/tree/0761b561062de01b701d2571401f625c31059a9a) | Composição criativa de continentes, cristas, vales e patamares; portar receitas e controles para operadores comuns | Java + grafos JSON específicos de Minecraft; [LICENSE CC0-1.0 nessa revisão](https://github.com/Apollounknowndev/tectonic/blob/0761b561062de01b701d2571401f625c31059a9a/LICENSE) | Traduzir semântica dos campos, nós e coordenadas; referências externas e recursos de terceiros auditados individualmente |
| [Red Blob Games Mapgen4](https://www.redblobgames.com/maps/mapgen4/) | Terreno dirigido por pintura, elevação, redes de rios e relações de chuva/vento; boa base para autoria + geração | TypeScript/JavaScript; [código Apache-2.0](https://github.com/redblobgames/mapgen4/blob/main/LICENSE), dependências identificadas no README | Adaptar algoritmos/dados para regiões e domínio comum; o próprio README lista esfera como evolução, não suporte pronto |
| [Azgaar Fantasy Map Generator](https://azgaar.github.io/Fantasy-Map-Generator/) | Planejamento de continentes, costas, rios e regiões; importar dados editáveis como estrutura macro | Aplicação web JS/TS; [MIT com disposição explícita sobre saídas](https://github.com/Azgaar/Fantasy-Map-Generator/blob/master/LICENSE) | Importador de dados e escala/projeção; sua cartografia não é por si uma superfície3D caminhável |
| [WorldEngine](https://github.com/Mindwerks/worldengine) | Modelos de placas, erosão, chuva e biomas; geração offline de dados de mundo | Python; [núcleo MIT](https://github.com/Mindwerks/worldengine/blob/master/LICENSE.txt); dependências próprias | Começar por dados/algoritmos; dependências como PyPlatec precisam de auditoria separada antes de incorporar código |
| [FastNoiseLite](https://github.com/Auburn/FastNoiseLite) | Biblioteca matemática para ruído2D/3D, fractais, cellular e domain warp | [Port Rust](https://github.com/Auburn/FastNoiseLite/tree/master/Rust), [MIT](https://github.com/Auburn/FastNoiseLite/blob/master/LICENSE), entradaf64 opcional | Avaliar integração/precisão/desempenho; fornece blocos matemáticos, não receitas geográficas completas |

### Tectonic: fonte concreta de criatividade

Consulta fixada no commit `0761b561062de01b701d2571401f625c31059a9a`, branchv3.
A branch padrão consultada era `rewrite-squared`, com outra árvore; não presumir
licença/estrutura iguais entre versões. O LICENSE efetivamente lido nav3 éCC0,
independentemente de metadados de outras distribuições.

Foram lidos estes arquivos sob
`src/main/resources/resourcepacks/tectonic/data/tectonic/worldgen/density_function/`:

- `continent_selector.json`: composição aritmética de seletor de ilhas, com cache.
- `mountain_ridges/ridges.json`: base deslocada e detalhe ponderado, por referências.
- `noise/continent/erosion.json`: remapeamento por spline de um campo de ruído.
- `region/club/plateau_spline.json`: splines aninhadas e controles regionais.

A oportunidade está nas relações entre campos e perfis. Um nó chamado `erosion`
nessa amostra não executa simulação hidráulica: o nome do controle não comprova
um processo físico. As receitas também referenciam namespaces de Minecraft e,
em alguns recursos, outros packs. Não copiar esses terceiros por associação.

Uma conversão inicial deve suportar explicitamente referências, aritmética,
clamp/min/max, seleção, ruído/deslocamento e curvas de remapeamento, recusando
nós não suportados. Equivalência do resultado depende da semântica do ruído,
seed, unidades e avaliação; trocar a função de ruído pode alterar o desenho.
Não declarar reprodução pixel-exata do Tectonic ou compatibilidade geral com mods.

Campos de densidade3D/cavidades precisam de saída volumétrica ou ferramenta
local apropriada. Reduzir tudo à primeira altura eliminaria cavernas/reentrâncias;
um importador deve informar essa limitação ou recusar a parte não representável.
Referências aY e limites verticais de Minecraft exigem conversão para altura/datum
local do domínio, e não para um eixoY global obrigatório em toda a esfera.

### Geradores web e dados de mundo

[Azgaar oferece exportação JSON/GeoJSON](https://azgaar.github.io/Fantasy-Map-Generator/?source=pwa),
incluindo rios/células. Isso permite importar informação geográfica em vez de
reconstruir o mundo a partir de uma imagem. Precisamos converter unidades,
projeção, datum, conectividade e fronteiras, preservando a proveniência da fonte.
Mapas finitos exportados não são automaticamente geradores de extensão infinita.

[Mapgen4 separa dados de geração da apresentação](https://github.com/redblobgames/mapgen4):
`map.ts` contém algoritmos, `painting.ts` a autoria, `render.ts` a apresentação e
`worker.ts` o cálculo. O algoritmo lido pré-calcula ruídos e usa arrays tipados de
altura/chuva/fluxo e um grafo de vizinhança. Isso é útil para um estágio regional;
o renderer de cartografia não precisa ser levado para a Ignis.

[WorldEngine exporta dados/heightmaps e suporta formatos de mundo](https://github.com/Mindwerks/worldengine/blob/master/README.md).
É candidato a produzir bases macro offline, sem exigir Python no runtime de um
jogo. O projeto não demonstra por si streaming infinito ou geração planetária
seamless no nosso contrato. Seu `pyproject.toml` inclui PyPlatec/NumPy/Protobuf;
a licença do núcleo não substitui a análise dos componentes realmente importados.

FastNoiseLite já tem portRust e featuref64, conforme Cargo.toml/lib.rs. Isso
facilita avaliação semFFI, mas f64 na entrada não comprova precisão para todas as
coordenadas da Ignis, limites do lattice, determinismo entre backends ou maior
velocidade que o sampler atual. Exigir prova específica antes de substituir código.

## Forma de adoção recomendada

Primeiro preparar a fonte comum e os operadores necessários às receitas. Em
seguida selecionar uma família de terreno Tectonic e um caso de crista/vale/rios
Mapgen4, portar a semântica para fontes editáveis nativas e avaliar seus controles
nos dois domínios. Integração parcial deve ser identificada, não apresentada como
suporte integral ao pacote. Azgaar pode entrar por importação de planejamento;
WorldEngine fica como candidato de geração offline; FastNoiseLite como candidato
de kernels matemáticos. Não há motivo para integrar os cinco de uma vez.

Três mecanismos de reuso são possíveis: incorporar código compatível/adaptado,
converter receitas em operadores nativos ou importar dados gerados. Um site ser
gratuito não define a licença do código, dos assets ou de suas saídas; verificar
a revisão escolhida. A versão da Ignis continua regida pelo LICENSE atual e os
componentes externos conservam seus próprios termos. Na incorporação efetiva,
registrar origem/commit/arquivos/modificações/licenças e os avisos exigidos em
THIRD_PARTY_NOTICES; esta pesquisa não declara dependências já adotadas.

## Performance e verificação antes da integração

- Parsers/conversão fora do frame; programa Rust preparado, sem JSON por amostra.
- Campos pontuais separados de redes/erosão regionais, com dados cacheados.
- Versão/seed/hash fixos; ordem de consulta, chunk e LOD não mudam a geografia.
- Fonte antiga continua legível; atualizações de biblioteca não mudam saves existentes
  silenciosamente. O pacote externo não define o formato canônico da Ignis.
- Métrica, área, fronteiras e vizinhança fornecidas pelo domínio: testar polos,
  faces e regiões, não só um preview plano esticado sobre uma esfera.
- Sem material de terceiros não auditado, cópia de dependência transitiva implícita
  ou adoção de um runtime de outra engine. Jogos exportados continuam locais.
- Validar controles das receitas e geografia resultante em fixtures próprias,
  depois custo de geração/memória e picos em movimento. Não modificar a demo humana
  para apresentar uma melhoria artística que não foi solicitada.

Pesquisa documental com leitura de fontes/licenças/receitas. Nenhum software foi
instalado, algoritmo integrado, cena alterada ou benchmark novo executado.
