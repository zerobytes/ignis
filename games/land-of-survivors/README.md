# Land of Survivors

Projeto nativo Ignis, iniciado em 3 de novembro de 2026. Primeira etapa: mundo esférico finito do tamanho médio terrestre, antes da elaboração de relevo. [GDD](../../docs/games/land-of-survivors/GDD.md) · [Estudo de geografia](../../docs/games/land-of-survivors/geografia.md).

## Mundo atual

- Raio de referência: 6.371 km; diâmetro: 12.742 km.
- Fonte própria Planet2, com altura radial zero e nenhuma operação de relevo.
- Cena inicial sem piso plano, NPCs, construções ou assets anteriores.
- Um observador técnico sem avatar, habilidades ou gameplay, exigido pelo runtime planetário atual.
- Nenhuma geografia de Genshin aplicada; o estudo precede a topografia.

`ignis.project.json` registra a cena inicial. `world/sphere.planet.json` é a fonte legível de autoria e `world/sphere.ignis-planet` sua persistência nativa. `runtime.json` configura a sessão planetária; `view.json` define a inspeção orbital. Divergências entre essas fontes são recusadas pelo verificador.

## Verificar e abrir

A partir da raiz do repositório Ignis:

```sh
python3 games/land-of-survivors/world.py
python3 games/land-of-survivors/world.py --open
```

O primeiro comando abre o projeto pelo MCP sem janela, carrega a fonte e o runtime e grava a captura e o relatório correntes em `artifacts/` deste projeto. O segundo abre a janela nativa. Roda aproxima/afasta; G alterna globo/superfície e F volta à superfície. O mundo inicia pausado para inspeção.

Usa o executável corrente `target/release/ignis`, sem compilar ou copiar um aplicativo. `--binary` permite indicar outro executável explicitamente. Uma falha não substitui o projeto nem suas fontes. Não abre rede ou dispositivos de áudio.

## Persistência e escopo

O manifesto e a cena são reabertos nativamente. A fonte esférica também passa pelo save/load nativo. O runtime planetário ainda é estado de sessão: abrir somente o manifesto no editor não ativa esse adapter. O launcher faz essa ativação explicitamente; a esfera não é apresentada como um estado planetário já integrado ao save de gameplay ou ao editor autoral.

A cena-base e as fontes permanecem estáticas durante inspeção. A captura orbital mantém a simulação sem avanço. O adapter atual admite um observador e possui limites próprios de física, residência e atmosfera; não representa um planeta com todos os atores do RPG integrados.

A verificação desta entrega foi headless, com o renderer nativo; não se abriu janela de jogo nem se declarou aceite interativo, FPS ou acabamento artístico. As capacidades genéricas da engine foram usadas sem reutilizar assets, cenas, modelos ou receitas de mundos anteriores.

## Custos e outputs locais

Python apenas abre o projeto e envia comandos de sessão; não amostra vértices nem executa trabalho por frame. Leitura das fontes, conferência e captura são explícitas. Física, amostragem, residência e renderização permanecem no Ignis, com cobertura orbital limitada a 384 tiles nesta configuração. Esse limite e a esfera simples não estabelecem o custo do futuro relevo.

Mantém-se um único executável corrente da engine, sem builds ou bundles adicionais para o jogo. A captura, o log e o relatório usam caminhos estáveis em `artifacts/` e são substituídos pela verificação seguinte. Esses outputs locais ficam fora do Git; manifesto, cena, fontes próprias e comandos de abertura são versionados.
