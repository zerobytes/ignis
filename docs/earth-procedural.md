# Relevo terrestre com seed

**Passeio planetário atual:** `python3 scripts/open_planet_walk.py` usa esta
mesma receita global com origem transportada, gravidade radial e residência
progressiva. Ver [planet-runtime.md](planet-runtime.md). O restante deste
documento registra a montagem anterior de patch fixo, seus comandos/evidências;
`open_earth_walk.py` continua sendo essa fixture finita, sem circunavegação.

A demo usa **seed 731199**, diâmetro **12.742.000 m** e o compilador planetário
Rust existente. Não reduzimos o planeta para caber na cena: publicamos uma região
local da sua superfície curva. A receita editável está em
`examples/earth-procedural.recipe.json`.

## Fonte, publicação e reprodução

Três camadas de noise 3D em coordenadas planetárias compõem relevo amplo, colinas
e detalhe. O gerador acrescenta oito picos com ombros sobrepostos e quatro
depressões: 23 operações nativas ao todo. A seed controla tanto as camadas quanto
os parâmetros dessas elevações. O noise não reinicia por tile; os picos também
são operações sobre a esfera, não remendos por chunk. A área livre de 400 m no
spawn exclui os caps adicionados, mas conserva o noise de base.

`scripts/build_earth_procedural.py` só monta os parâmetros e orquestra MCP.
`QueryPlanet`, `PublishPlanet`, `SavePlanet` e `LoadPlanet` executam amostragem,
compilação e persistência nativas. O documento resultante continua editável sem
esse gerador. Salvar e carregar a fonte usa `.ignis-planet`; a cena publicada
contém o bake e pode rodar sem Python ou agente.

O launcher agora prepara relevo por padrão:

```sh
python3 scripts/open_earth_walk.py --seed 731199
```

Pré-requisito da montagem atual: `artifacts/earth-horizon/native-eye.scene.json`,
gerado por `python3 scripts/verify_planet_reference.py`, e binário corrente em
`target/release/ignis`. Para preparar/verificar sem janela, usar
`python3 scripts/open_earth_walk.py --prepare-only --seed 731199`. Para abrir a
cena já preparada, `python3 scripts/open_earth_walk.py --skip-prepare`.
`--flat` reproduz a montagem uniforme anterior.

Saídas correntes em `artifacts/earth-walk/`:

- `earth.ignis-planet`: fonte nativa; `source.recipe.json`: seed e documento.
- `walk.scene.json`: cena jogável com terreno, colisão, Player e quatro marcos.
- `procedural-eye.png` e `procedural-aerial.png`: capturas do renderer normal.
- `verification.json`: geometria, costuras, reprodução e deslocamentos físicos.
- `launch.json`: PID, contexto da câmera, hash da cena e identidade do binário.

O launcher reutiliza a janela se cena e binário forem os mesmos. Se a janela
registrada estiver executando outra versão, recusa duplicá-la; fechar essa janela
antes de abrir a substituta. A preparação pode ocorrer com a janela anterior
aberta, pois usa outro processo sem janela.

## Validação desta receita

16 tiles nível 12/N128: **266.256 vértices e 524.288 triângulos**. As 24 bordas
compartilhadas têm posições exatamente iguais. Repetir 81 amostras da mesma
fonte deu igualdade exata; a seed seguinte mudou o relevo relativo em até
569,648 m nesse conjunto. Salvar/carregar a fonte preservou o documento.

As cotas locais publicadas vão de −194,696 m a 850,321 m. São alturas no
referencial tangente, incluindo curvatura, após normalizar a altura do spawn;
não são altitudes geográficas em relação a um nível oceânico.

Quatro caminhadas diagonais terminaram apoiadas no terreno, com deslocamentos
de 39,49 a 39,94 m. Duas corridas de 7.200 ticks, em lotes admitidos de 3.600,
terminaram apoiadas com deslocamentos de **710,575 m / +13,520 m de altura** e
**718,743 m / −13,193 m**. Esses números são distância entre início e fim,
não integral do caminho. Um ensaio inicial frontal encontrou o marco sólido
azul aos 29,39 m; os percursos diagonais isolam a locomoção no terreno.

Capturas de solo e aérea inspecionadas. A janela uniforme anterior foi fechada
para abrir a nova demo em PLAY/primeira pessoa, PID 86191/PPID 1 nesta rodada.
Reabrir a mesma cena retornou o mesmo PID. O processo sobrevive ao encerramento
do cliente MCP. Aceite humano e FPS permanecem separados.

## Checklist estático de performance

- **Caminho quente:** nenhuma lógica Python, parsing JSON ou geração por frame.
  Reutilizamos sampler preparado Rust e malhas estáticas do caminho existente.
- **Alocações/cópias:** receita pequena; não enviamos vértices pelo MCP. O arquivo
  de cena é lido localmente uma vez para conferir costuras/cotas, com reduções
  por iteradores sem outra lista de alturas.
- **Trabalho repetido:** uma publicação por tile; colisões, jogador e marcos
  entram numa transação final, evitando reconstruir a física a cada tile.
- **I/O/GPU:** fonte e bake são saídas de preparação; capturas explícitas. Nenhum
  device, socket, thread de simulação ou caminho novo de renderização.
- **Build/cache:** sem alteração Rust ou rebuild. Binário corrente
  `a1253ad95fa6238f`, launcher com symlink; sem cópias de executáveis históricos.

Python compilado e diff verificado; validação direta MCP de geometria/colisão e
capturas, sem repetir a suíte Rust para esta montagem de receita/scripts.

## Fronteiras preservadas

### Corrida acelerada somente nesta demo

Por solicitação humana, o Player desta montagem usa `locomotion.run.speed=600`
m/s, 100 vezes a corrida anterior de 6 m/s, acionada pelo Shift existente.
Caminhada continua 4 m/s e movimento furtivo 1,5 m/s. A preparação plana e a
procedural aplicam esse perfil só à demo; os padrões da engine continuam
4/1,5/6 m/s. Não há alteração do tempo de simulação, teleporte ou bypass de
colisões. A validação dos perfis aceita agora 0,1..1.000 m/s (antes 0,1..10),
com finitude e ruído ainda validados. NPC.speed não foi alterado.

Revisão estática: a única mudança Rust é na validação fria do perfil, sem novas
alocações, cópias, consultas ou ramos por frame. O KCC continua usando o sweep
existente; um teste cobre barreira de 10 cm a 600 m/s, liberação da corrida e
replay. Isso não garante aderência ao solo em todo relevo: nessa velocidade,
o motor físico pode lançar o personagem ao sair de uma crista.

Rodada do boost: 24 testes Rust em `locomotion`/`character_controller` passaram,
incluindo o novo sweep, soltura de corrida e replay; Clippy desses alvos/binário
com `-D warnings`, build release, Python compile e diff aprovados. MCP no build
`8f0aeb31b296e5cf`: 60 ticks deram 3,999 m caminhando e 576,956 m correndo no
relevo; o marco frontal bloqueou a corrida em Z=−29,390 m. Ao fim da corrida
diagonal o KCC não estava grounded, preservando a queda física da saída de
crista. Relatório `artifacts/earth-walk/sprint-verification.json`.
A cena foi salva pelo MCP com o perfil acelerado e reaberta em PLAY/primeira
pessoa, PID 86761/PPID 1. O PID 86191 da rodada anterior foi encerrado.

É uma região local de aproximadamente 12,44 km por lado, com frame fixo e
gravidade −Y. A fonte procedural define relevo esférico global, mas residência,
streaming, origem móvel e locomoção radial ainda não estão integrados. Não é
possível circunavegar fisicamente nesta demo.

A garantia anterior de todo o horizonte coberto foi demonstrada para a esfera
uniforme e câmera central; não se transfere automaticamente ao relevo, a outras
posições ou altitudes. Sem esconder bordas com atmosfera. Não implementamos
erosão, hidrologia, biomas ou física planetária neste incremento.
