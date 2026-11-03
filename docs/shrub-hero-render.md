# Render de apresentação da topiaria

Pedido: escolher um arbusto da biblioteca e produzir um render com a maior
qualidade possível. Escolha: `esfera_miuda_0`, de `assets/botany/reference-shrubs`.
Este é um render offline em Cycles, Blender5.0.1/Metal no AppleM4Pro; sua aparência
não é uma demonstração de recursos ou FPS do renderer Ignis.

## Geometria e aparência

`prepare_shrub_hero.py` lê as duas fontes canônicas e recompila as mesmas curvas
no Ignis, sem reescrever a biblioteca. Conserva13.171 folhas e1224 eixos, sementes,
inserções, espessuras e parâmetros da forma. Somente tesselação de apresentação:
26 amostras nas lâminas,8 nos eixos e12 lados. A seleção26 é feita pelo orçamento
nativo de3Mvertices/6Mtri por fonte, sem ampliar quotas ou mudar Rust.

Bake nativo no build `8784ca62e3513867`:3 meshes reais com3.480.074v/6.902.568tri.
O triângulo de categoria vazia emitido pelo compilador não entra no render.
Conversão Y-up→Z-up é uma rotação com determinante+1. Importação e reabertura
do `.blend` conferem hashes dos buffers float32 e índices, sem remodelagem.

Cada folha recebe UV pelo grid strip exato do compilador. A contagem por ilha e
as raízes de todas as folhas foram verificadas contra a fonte: desvio máximo
1,2e-7m na leitura float32. Não se distribuiu folhagem nova nem se usou foto como
textura. Todos os shaders são procedurais, com variação por folha e brotação,
nervuras discretas em bump, rugosidade, cutícula e transmissão difusa de lâmina.
Esses materiais e detalhes de superfície são próprios desta apresentação em
Cycles; não foram portados para os materiais nativos do Ignis.

O chão escuro e os pequenos seixos são cenário procedural de apresentação.
Iluminação com área principal quente, preenchimento de céu frio e recorte suave.
O preenchimento usa light linking sobre a planta, como um rebatedor controlado.
Câmera85mm/f11, foco na copa e AgX Medium High Contrast com exposição+0,2EV.
Fundo sem horizonte cortando o enquadramento. Fontes/cenário/janela humanos não
foram abertos ou alterados; Blender executa em background/factory startup.

## Reprodução e arquivos

```sh
python3 scripts/prepare_shrub_hero.py
/Applications/Blender.app/Contents/MacOS/Blender --background --factory-startup \
  --python-exit-code 1 --python scripts/render_shrub_hero.py -- --stage build
/Applications/Blender.app/Contents/MacOS/Blender --background --factory-startup \
  --python-exit-code 1 --python scripts/render_shrub_hero.py -- --stage review
/Applications/Blender.app/Contents/MacOS/Blender --background --factory-startup \
  --python-exit-code 1 --python scripts/render_shrub_hero.py -- --stage final
```

Saídas correntes em `artifacts/shrub-hero`, sempre nos mesmos caminhos:

- `hero.png`:4096×4096, PNG16bits com transformação de exibição AgX.
- `hero.exr`:master linear32bits, multilayer ZIP, Combined/índice/noisy image.
- `hero.blend`:uma única cena atual, geometria e materiais autocontidos.
- `preview.png`/`preview.exr`:ensaio1024×1024 para revisão de luz/aparência.
- `native.scene.json`/`native.png`:bake/captura nativos de apresentação.
- `native-report.json`, `bridge-report.json`, `render-report.json`:proveniência,
  hashes, contagens, câmera, luzes, dispositivo e parâmetros de amostragem.
- `review-{front,side,back}.png`, `review-sheet.png`, `hero-values.png`,
  `detail-crop.png`:revisão espacial, valores e detalhe sem resize.

Não há cópias históricas de `.blend`/instaladores. `save_version=0` evita `.blend1`.
O QA opcional `--stage qa --image-stem hero --toolkit CAMINHO` usa `bx_light.py`
da skill local de iluminação; não é uma dependência do motor nem do `.blend`.

## Revisão e limites

Primeiro preview revelou horizonte e reflexos muito fortes; câmera foi elevada,
cutícula reduzida e solo refinado. A revisão desaturada também mostrou o chão
competindo com a planta: material do chão e preenchimento foram ajustados antes
da saída4K. Frente/lado/costas, beauty, estudo de valores e detalhe final foram
abertos para inspeção. Métricas são de leitura do render, não de fidelidade às
fotografias ou de desempenho em tempo real.

Uma primeira escrita do EXR falhou pelo enum condicionado ao `media_type` no
Blender5.0.1. Corrigida usando `MULTI_LAYER_IMAGE` antes de `OPEN_EXR_MULTILAYER`;
PNG e master são gravados pelo renderer, sem geração/edição por IA.

As curvas canônicas continuam iguais. Interseções entre folhas/ramos e padrões
de ramificação do asset não foram redesenhados por este pedido. Não se infere
suporte nativo aos shaders Cycles, qualidade de gameplay, física, animação ou
aceite artístico do usuário a partir deste render.

## Evidência final

Render final concluído em370.63s nesta execução,4096×4096, até1024 amostras adaptativas, limiar0,005. Tempo de render offline, não FPS. PNG de16bits e master EXR de32bits verificados. Nenhum pixel não finito nos dados analisados. Checagem visual de beauty, três ângulos, valores e crop900×900 original concluída. Veredito do toolkit, auditoria de cor e flags de iluminação: listas vazias. Clipping de luminância na imagem:0.0%; preto puro/quase preto:6.88%; máximo de luminância linear:3.512. Pico de contraste sobre a folhagem, contraste mediano com o entorno:+0.125. Essas heurísticas não são aprovação artística humana.

A rota NumPy/Accelerate de matmul de luminância em4K emitiu avisos numéricos; o QA foi refeito com soma explícita float64 e teste de finitude, sem alterar a imagem. Relatório definitivo `hero-quality.json`.

`hero-display.png` é somente um proxy2048²/8bits do PNG final para apresentação no chat; o original4K/16bits permanece intacto. No macOS, reproduzir o proxy com:

```sh
sips -Z 2048 artifacts/shrub-hero/hero.png --out artifacts/shrub-hero/hero-display.png
```

Os hashes das fontes canônicas permanecem iguais. Hash dos buffers importados/reabertos no Blender também exato, UVs por ilha verificadas e nenhum nó de textura de imagem. Python e diff próprios conferidos; sem alterações Rust/build/instalação/push ou interação na janela humana. Commit apenas dos scripts de render e deste guia.
