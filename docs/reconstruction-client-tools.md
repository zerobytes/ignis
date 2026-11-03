# Ferramentas de reconstrução no cliente — EAF-189

Ferramentas de autoria/diagnóstico fora da engine. Não acrescentam comando MCP,
interpretação automática, mutação de Scene ou formato de persistência. Ignis
continua Rust; não foram alterados seus controles, renderer, build ou demo.

## Reaproveitamento e licença

Fonte pinada: `img2threejs/img2threejs@3e7099e1b7fe93e187e06db4bdbe1e1906c21a28`.
Apache-2.0 e copyright 2026 hoainho preservados em
[vendor/img2threejs](../scripts/vendor/img2threejs/NOTICE.md).

- Três módulos Python de projeção/validação/mínimos quadrados amortecidos.
  Imports tornados relativos e cabeçalhos de procedência acrescentados;
  algoritmo preservado.
- Helper TS de sweep com seções variáveis e frame carregado no eixo,
  extraído das strings do gerador. Import/export de módulo acrescentados;
  costura usa exatamente o primeiro ângulo na última amostra, evitando `sin(2π)`.
- Revisão por evidências e etapas inspirou um ledger próprio pequeno.

Não portar todos os kernels ao runtime: Ignis já possui seções, trajetórias,
sweeps, topologia e comparação de imagens. O helper TS serve o ensaio externo.

## Câmera por correspondências

```sh
python3 scripts/reconstruction_camera.py landmarks.json --out camera-fit.json
```

Entrada: `initial_camera` com `image_width/image_height/fov_degrees`,
`yaw_degrees/pitch_degrees/roll_degrees/position`; lista `correspondences` com
6..256 IDs `name` únicos, pontos `world:[x,y,z]` e `observed:[pixel_x,pixel_y]`.
Origem dos pixels superior esquerda. Não aceita pontos colineares, NaN ou bools.
O cliente fornece as hipóteses 3D e suas correspondências; não são descobertas.

Saída: câmera exata com `position/target/up/fov_degrees`, RMS em pixels,
resíduos e estado do solver. `stalled/max-iterations` continuam diagnósticos;
RMS não é porcentagem de fidelidade nem confiança estatística.

A convenção pitch→yaw→roll do upstream é convertida para câmera destrógira
com up explícito. Ignis Camera usa world-up fixo: uma solução incompatível
produz `ignis_camera=null`, `ignis_compatible=false`. Não descartar roll nem
enviar `up` ao schema nativo. Esta entrega não adiciona roll à engine.

## Ledger de revisão

```sh
python3 scripts/reconstruction_review.py review-ledger.json
```

Versão 1: `evidence` mapeia nomes para `path/sha256`; `required_items` registra
IDs planejados; `items` contém `id/criterion/diagnosis/evidence/verdict`.
Vereditos: `supported/divergent/unobserved/pending`. Caminhos relativos à pasta
do ledger. Hash recalculado por blocos de 1 MiB; falta/alteração de arquivo,
ID duplicado, item planejado ausente ou evidência desconhecida recusa validade.
Exit 0 só significa evidência atual: ler `remaining_items` e
`all_observable_items_supported`. O programa não aprova arte nem dá escores.

## Ensaio adaptado da bota

[Análise e lista antes da geometria](references/image-to-3d/boot-img2threejs-analysis.md).
Receita: `scripts/experiments/boot_model.ts`. Destino corrente:
`/Volumes/Zero/img2threejs/work/ignis-boot`.

```sh
python3 scripts/experiments/prepare_boot.py
python3 scripts/experiments/serve_boot.py /Volumes/Zero/img2threejs/work/ignis-boot
```

Dependências locais: Three.js 0.169.0 e esbuild 0.25.12. Nenhuma skill/MCP,
modelo neural ou dependência global instalado. Servidor somente localhost;
botões salvam PNG/JSON em `captures/`. Original/crops/hashes preservados.

É um teste **adaptado** do solver, sweep e construção/revisão por etapas, não
certificação da skill upstream inteira. Sem alegar passagem de oito etapas,
gates de materiais ou rigging. Sem GLB prévio ou textura da foto projetada.
O agente escreveu a geometria interpretando a imagem. Três estágios explícitos,
uma base anterior à correção e o modelo final são evidências de comparação;
intermediários Node duplicados foram removidos.

### Evidências e limites

- 40 meshes, 10.992 vértices e 20.384 triângulos no modelo final.
- Geometria exportada do navegador igual à construída pelo código em Node.
- Projeção Python versus Three.js: 12 pontos, erro máximo
  `2.3437142008433856e-13` px. Protege a convenção de câmera, não as anotações.
- Ajuste inicial RMS 101,48→30,65 px; terminou `stalled` e exige roll.
  Âncoras aproximadas/forma hipotética; câmera mantida nas comparações
  seguintes. O RMS não foi reinterpretado como medição do modelo final.
- Auditoria por peça: sem faces degeneradas, arestas não manifold ou orientação
  inconsistente. Cabedal com 64 arestas abertas no término do forro interno.
  Sem certificação de autointerseções, física, colisores ou vestibilidade.
- Gate upstream de turntable: RGB falhou por possíveis vazios em reflexos;
  silhuetas brancas reais do mesmo modelo passaram nas quatro direções,
  sem relaxar limiares ou permitir buracos. Ambos os relatórios preservados.
- Ledger: 13 itens, evidências atuais, **10 divergentes**. Arquivo alterado
  invalidou corretamente uma revisão na verificação direta.

**Fidelidade insuficiente:** cano muito regular, perfil do pé, dobras,
ramos/folhas e desgaste não reproduzem a referência. Forma neutra, quatro
direções e superior abertas; verso/interior da foto são desconhecidos.
Modelo 3D válido não equivale a asset pronto ou aceitação artística.

Arquivos: `review.png`, `turntable.png`, `review-ledger.json`,
`review-audit.json`, `geometry-audit.json`, `projection-check.json`,
`camera-fit.json`, `turntable-{rgb,mask}-gate.json` e `captures/`.

### Custos e validação proporcional

Solver no cliente: sete parâmetros, máximo 80 iterações × 12 tentativas de
amortecimento, até 256 pontos. Sem dependência numérica Python adicional,
worker, GPU ou trabalho no tick. Geometria criada uma vez por página,
reutilizada ao orbitar; hash em blocos, sem leitura de modelo por frame.
Exportação sob comando pode duplicar buffers JSON: custo de ferramenta aceito.
Destino com dependências/evidências: cerca de 47 MiB, sem novos pesos.

Compilação TS/esbuild, sintaxe Python, projeção cruzada, igualdade de geometria,
invalidação de evidência e auditorias diretas. Sem teste por campo, build ou
regressão Rust: engine não alterada. Sem medição de FPS ou ganho artístico.
