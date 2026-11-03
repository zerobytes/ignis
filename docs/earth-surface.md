# Surface no mundo terrestre jogável

A fixture plana de `surface-program` demonstra operadores; não é o mundo pedido
pelo usuário. Sua crista alta/estreita, com85% de perfil linear e altura uniforme,
produziu cortinas serrilhadas e foi reprovada explicitamente. Publicar uma malha e
passar testes de fonte não validava a paisagem nem a integração ao mundo.

`examples/earth-surface.recipe.json` e `scripts/surface_geography.py` compõem as
ferramentas nativas de Surface1. A função `draft_for(domain='plane'|'sphere')`
muda somente a conversão dos locais autorais; operadores, perfis e parâmetros
são os mesmos. Python não avalia alturas, normais, colisão ou malhas.

A receita contém campos continentais/regionais, duas cadeias meandrantes e uma
ramificação, envelopes de terminação, encostas assimétricas arredondadas, alturas
moduladas por ruído suave e cristas secundárias, além de um vale largo. A seed
altera campos e pontos da receita. Os alinhamentos principais são autorais;
isso ainda não é um gerador global de redes hidrográficas ou erosão geológica.

## Executar

Na raiz do projeto:

```sh
python3 scripts/prepare_earth_surface.py
python3 scripts/open_earth_surface.py
```

O primeiro comando usa MCP real sem janela, compila o grafo, normaliza a altura
do nascimento dentro do próprio programa, salva/reabre a fonte e carrega o
runtime. Gera `artifacts/earth-surface/player.scene.json`, `world.json`, fonte
`.ignis-planet`, capturas de chão/terceira pessoa/altitude/globo e um relatório.
O segundo abre o mundo nativo com um controlador MCP local destacado do terminal,
mantendo o transporte aberto até o usuário fechar a janela. Inicia a simulação
pelo MCP; P pausa/continua, WASD anda, Shift corre a600m/s só nesta demo, clique+mouse olha,
Tab alterna primeira/terceira pessoa, roda muda distância, G alterna globo e
superfície. Esc solta o mouse. Não existe uma borda de patch para o personagem.

O planeta tem12.742.000m de diâmetro. O runtime transportado continua usando
residência, colisão e câmera orbitais existentes; não é uma esfera reduzida nem
uma malha estática plana disfarçada. Arquivos de `planet-runtime` usados pela
frente Horizon permanecem intactos.

## Revisão estática de desempenho

- **Há trabalho de autoria no frame?** Não; composição Python e preparação de
  coeficientes ficam fora da execução.33nós tipados são avaliados pelo sampler
  Rust preparado já existente, compartilhado por Arc com o Atlas.
- **Há busca/alocação por amostra?** Slots fixos e curvas preparadas;23pontos
  de cadeias e6do vale. Busca linear local limitada; não ampliar a receita para
  representar uma rede planetária inteira sem índice regional.
- **O custo foi limitado?** Estimativa nativa1368unidades por ponto;
  uma grade33×33 com7amostras por vértice soma10.428.264unidades, abaixo do
  envelope16M. Não é uma medição de milissegundos.
- **Foi reduzida resolução para esconder custo?** Não. Nível12,64subdivisões
  da residência física/visual e raio físico256m do mundo anterior mantidos.
  A câmera global mantém seu LOD/limites próprios. Perfil arredondado evita o
  cusp do fixture; detalhes de crista têm período mínimo256m.
- **A atmosfera mudou?** Nesta receita a densidade passou de0,00065 a0,0004
  para permitir avaliar o relevo. A residência inicial cresceu de88para208tiles
  (878.800vértices da fonte com65×65/tile). Isso AUMENTA o custo e não é ganho
  de FPS. O controle e a otimização atmosféricos nativos continuam ativos.
- **Foi criado outro build/cache?** Não. Reutilizado o executável corrente e o
  mesmo `Ignis Play.app` por symlink, uma saída estável por receita. Renderer,
  materiais e implementações orbitais reservadas por outras frentes não mudam.

## Evidência e limites

MCP: fonte repetível, save/load do programa exato, nascimento grounded, vistas
sem mover o jogador, seis faces cobertas e bordas conformes no globo, percurso
real de aproximadamente1.781m e grounded após parar. O relatório identifica o
build exato; não confundir um executável recompilado em paralelo com esse teste.
Capturas foram inspecionadas antes de abrir a janela humana.

As formas já diferem das cortinas reprovadas; aparência ainda depende do aceite
do usuário. Há relevo e colisão, mas materiais ainda são simples, a atmosfera
homogênea pode encobrir o globo e não há rios com água ou erosão neste recorte.
Não se afirma estabilidade de FPS, qualidade geológica completa ou correção dos
hitches do Horizon; EAF-199 continua aberto por esses limites.

Abertura desta rodada: controlador MCP confirmou `play`, runtime e câmera no
build b11b867816ecade1. O controle visual do macOS recusou a reconexão à Ignis
com `errAETimeout/-1712`; portanto a janela em primeiro plano e a aceitação
interativa não foram confirmadas. A fixture antiga foi fechada sem alterações.
