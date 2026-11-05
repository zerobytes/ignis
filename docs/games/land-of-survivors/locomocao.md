# Caminhada e corrida

Os clipes `los_walk` e `los_run` da biblioteca do jogo foram refeitos. O jogo lê
`assets/animations/locomotion.ignis-motion`; a implementação antiga de oscilação
continua apenas como fallback de criação de um arquivo ausente.

- Caminhada: ciclo de 54 ticks (0,9 s), passada completa de 1,44 m, apoio de cada
  pé em 60% do ciclo e retorno baixo. Entrada do jogo ajustada para 1,6 m/s.
- Corrida: ciclo de 40 ticks (2/3 s), passada completa de 4 m, apoio de cada pé em
  30% do ciclo, impulsão, fase aérea e recuperação do calcanhar. Shift mantém 6 m/s.
- Os pés recuam durante apoio na mesma velocidade do deslocamento; a rolagem
  usa a sola da malha fonte para apoiar calcanhar/planta/ponta sem girar no chão.
- O quadril acompanha o peso e o alcance das pernas. Tronco/cabeça compensam
  parte do giro; braços alternam com as pernas, cotovelos descontam a flexão do
  próprio rig, e as mãos ficam relaxadas na caminhada e parcialmente fechadas
  na corrida.

A cadência no jogo continua baseada na distância física percorrida. O ajuste da
caminhada usa ganho analógico de 0,4 sobre o perfil nativo de 4 m/s, normalizando
as diagonais. Assim, mundos portáteis existentes e slots de rede não precisam
ser regravados ou migrados. Corrida e Sneak mantêm o caminho nativo existente.

## Editar e gerar

`content/locomotion.recipe.json` guarda os parâmetros de autoria em metros do
personagem de 1,8 m, com durações, passada, apoio, altura do retorno, oscilação,
inclinação, braços e rolagem dos pés.

```sh
python3 games/land-of-survivors/author_locomotion.py
python3 games/land-of-survivors/app.py --animation-preview
```

O autor é uma ferramenta de criação de assets. Usa a API MCP existente para
avaliar poses e resolver as pernas com IK nativo; grava clipes nativos e salva a
biblioteca ao terminar. Não é um verificador ou teste. Uma execução interrompida
antes do save deixa o arquivo anterior. O idle não é redesenhado. O editor e o
MCP continuam podendo editar os clipes gerados normalmente.

A cena canônica de autoria é usada para medir altura e sola. Se a fonte do
personagem mudar, atualize-a antes de gerar:

```sh
python3 games/land-of-survivors/app.py --export-animation-scene \
  games/land-of-survivors/artifacts/app/animation.scene.json
```

As curvas são de locomoção para a frente em piso plano. A biblioteca não instala
foot locking durante giros, IK de terreno no jogador ou clipes direcionais para
andar lateralmente/de costas. Retarget mantém comprimentos nativos; não promete
contato idêntico em toda proporção de corpo. Esses limites não são resolvidos
por um ciclo reto bem apoiado.

A revisão atual usa render nativo de um ciclo completo, vistas lateral e em
três quartos, e a prévia do app. Capturas/GIFs correntes ficam em
`artifacts/app/locomotion/`. Aceite artístico humano permanece pendente. Nenhum
teste automatizado ou harness novo foi criado; nenhuma suíte foi executada.
