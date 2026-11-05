# Land of Survivors

Projeto nativo Ignis, iniciado em 3 de novembro de 2026. A entrada usa um palco de 200 × 200 m com menu e criação/seleção nativa de personagens. Veja [Entrada e personagens](../../docs/games/land-of-survivors/entrada-e-personagens.md) para persistência, prévia e limites do multiplayer. O mundo planetário e as 284 habilidades em laboratório separado estão preservados. [GDD](../../docs/games/land-of-survivors/GDD.md) · [Habilidades funcionais](../../docs/games/land-of-survivors/habilidades-runtime.md) · [Geografia](../../docs/games/land-of-survivors/geografia.md) · [Protótipo e ferramentas](../../docs/games/land-of-survivors/prototipo-relevo.md).

## Entrada do app

`scenes/Inicio.scene.json` é a cena inicial registrada no manifesto. Contém somente um piso físico plano de 200 × 200 m, de −100 a +100 m nos eixos X/Z, com topo em Y=0. Câmera e iluminação são configurações da cena. Não há jogador, NPC, personagem visual, decoração ou combate. Essa cena é a base do palco. O launcher do jogo compõe menu, lista e criação por cima dela, com personagens persistidos separadamente.

```sh
python3 games/land-of-survivors/start.py
python3 games/land-of-survivors/app.py --build
python3 games/land-of-survivors/app.py --open
python3 games/land-of-survivors/app.py --play
python3 games/land-of-survivors/start.py --editor
```

Sem opção, verifica a entrada, as dimensões, save/load e preservação da câmera por MCP, capturando `artifacts/front-end/start.png` e `overview.png`. `start.py --open` abre o jogo com seu menu; `--editor` abre o projeto para autoria. Usa o executável corrente do Ignis, sem compilar ou copiar um app. `app.py --play` abre o personagem selecionado diretamente no palco offline, com seu modelo nativo animado e câmera de terceira pessoa. WASD move, Shift corre, arraste esquerdo orbita, direito orienta o personagem e roda aproxima/afasta. As capturas são evidência automatizada; aceite humano de interface/arte permanece separado.

`scenes/Principal.scene.json` continua sendo a cena do observador planetário. Os launchers de mundo/relevo selecionam essa cena explicitamente, sem depender da nova entrada. A geografia e o laboratório de habilidades continuam independentes do palco de interface.

## Autoria de animação

Clipes, passada e fases de apoio ficam em `assets/animations/locomotion.ignis-motion`,
com edição/save/load pelo MCP e recarga visual durante a sessão.
`app.py --animation-preview` abre um palco sem save de personagem.
Veja [o fluxo de autoria nativo](../../docs/animation-authoring.md) para exportar
uma cena, editar o rig/clipes e gravar apoios no clipe. Os ciclos iniciais ainda
são rascunhos; a caminhada natural continua pendente de trabalho e revisão visual.

## Habilidades funcionais

```sh
python3 games/land-of-survivors/abilities.py --verify
python3 games/land-of-survivors/abilities.py
python3 games/land-of-survivors/abilities.py --mcp
```

264 habilidades humanoides e vinte de Urso/Felino, com custos, conjuração, recargas, projéteis, áreas, efeitos periódicos, controles e barras de dez posições. O componente nativo em `src/combat.rs` usa Engine/Rapier e um encontro técnico de primitivas novas. O terminal e o MCP compartilham o dispatcher. Não há HUD gráfico, animações ou integração com o runtime planetário nesta etapa. Comandos, balanceamento provisório, persistência e limites estão no [relatório funcional](../../docs/games/land-of-survivors/habilidades-runtime.md).

## Mundo atual

- Raio de referência: 6.371 km; diâmetro: 12.742 km.
- Fonte própria Planet2 com programa Surface1 de 26 nós: cristas, vales, picos e patamares em uma região de estudo.
- Cena inicial sem piso plano, NPCs, construções ou assets anteriores.
- Um observador técnico sem avatar, habilidades ou gameplay, exigido pelo runtime planetário atual.
- Geografia original. Genshin orienta a linguagem das formas; nenhuma localização ou mapa foi replicado.
- O restante do planeta continua sem composição continental; não é um mundo geográfico concluído.

`ignis.project.json` registra a cena inicial. `world/sphere.planet.json` é a fonte legível de autoria e `world/sphere.ignis-planet` sua persistência nativa. `runtime.json` configura a sessão planetária; `view.json` define a inspeção orbital. Divergências entre essas fontes são recusadas pelo verificador.

## Verificar e abrir

A partir da raiz do repositório Ignis:

```sh
python3 games/land-of-survivors/world.py
python3 games/land-of-survivors/world.py --open
python3 games/land-of-survivors/world.py --open --view ground
python3 games/land-of-survivors/world.py --open --view globe
```

O primeiro comando abre o projeto pelo MCP sem janela, carrega a fonte e o runtime e grava `artifacts/world.png` e `artifacts/verification.json`. O segundo abre a janela nativa na vista regional. As opções seguintes começam no chão ou no globo. Roda aproxima/afasta; G alterna globo/superfície, F volta à superfície e P pausa/continua. O mundo inicia pausado para inspeção.

Usa o executável corrente `target/release/ignis`, sem compilar ou copiar um aplicativo. `--binary` permite indicar outro executável explicitamente. Uma falha não substitui o projeto nem suas fontes. Não abre rede ou dispositivos de áudio.

## Persistência e escopo

O manifesto e a cena são reabertos nativamente. A fonte esférica também passa pelo save/load nativo. O runtime planetário ainda é estado de sessão: abrir somente o manifesto no editor não ativa esse adapter. O launcher faz essa ativação explicitamente; a esfera não é apresentada como um estado planetário já integrado ao save de gameplay ou ao editor autoral.

A cena-base e as fontes permanecem estáticas durante inspeção. A captura orbital mantém a simulação sem avanço. O adapter atual admite um observador e possui limites próprios de física, residência e atmosfera; não representa um planeta com todos os atores do RPG integrados.

As fontes e o renderer foram verificados headless; o launcher também abriu uma sessão nativa para revisão. O teste local percorreu aproximadamente 60 m no vale e terminou grounded. Isso não estabelece aceite artístico, FPS ou validação de todas as encostas. As capacidades genéricas da engine foram usadas sem reutilizar assets, cenas, modelos ou receitas de mundos anteriores.

## Reproduzir o estudo de relevo

```sh
python3 games/land-of-survivors/terrain.py
python3 games/land-of-survivors/terrain.py --publish
```

O primeiro comando compara controle suave e patamares regionais, sem substituir as fontes publicadas. O segundo instala a versão com patamares depois de passar pelos checks nativos de fonte, câmera e percurso. A receita editável é `world/terrain.recipe.json`. A galeria atual fica em `artifacts/terrain/review.html`, com dez capturas nas mesmas câmeras e um relatório de probes/LOD. Nenhuma imagem de referência participa da geração.

## Custos e outputs locais

Python compõe dados e envia comandos de sessão; não calcula alturas, malhas ou trabalho por frame. Leitura das fontes, queries e capturas são explícitas. Física, amostragem, residência e renderização permanecem no Ignis. A vista aceita até 1.536 tiles, sujeita também ao limite nativo de vértices. No chão, o bound solicitado ainda não é atingido; ver o diagnóstico antes de aumentar escala ou detalhe.

Mantém-se um executável corrente da engine e um executável corrente do laboratório de habilidades em `target/release/examples/land_of_survivors`, sem bundles ou cópias históricas. A captura, o log, o save de jogo e os relatórios usam caminhos estáveis em `artifacts/` e são substituídos pela verificação seguinte. Esses outputs locais ficam fora do Git; manifesto, cena, fontes próprias e comandos de abertura são versionados.
