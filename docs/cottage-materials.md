# Casa — estudo de duas escalas materiais

Integração do editor: Scene78/replay82 reúne os contratos; gates históricos
permanecem73/74. As provas materiais usam o asset de calibração estável
`examples/cottage-material-baseline.scene.json` (SHA7286e74c…f1e2c), separado da
casa estrutural corrente `examples/elven-cottage.scene.json`. Receitas mantêm
hashes de origem/pixels; `base_recipe_sha256` acompanha somente a atualização do
caminho de entrada, sem relaxar comparação. Números abaixo são da frente original.


A casa ea8c6b8 é o novo modelo de teste autorizado. A árvore permanece preservada.
Esta unidade EAF-112 trata uma fricção genérica: um único ruído compartilhava
paleta, escala e relevo; aumentar a rugosidade da pedra produzia manchas grandes
com aspecto amassado. Um Pattern fino agora fornece grão/relevo, enquanto o
secondary_pattern fornece manchas maiores com amplitude independente.
Contrato, alternativas e checklist anteriores em [layered-patterns.md](layered-patterns.md).

`python3 scripts/study_cottage_materials.py` usa o binário release corrente e o
MCP real em processo próprio, sem janela. Carrega a casa72 mantida, publica um
lote de nove materiais e salva uma cópia73 em
`artifacts/cottage-materials/examples/cottage-materials.scene.json`. Receita
editável em `examples/cottage-materials.recipe.json`; hashes da cena fonte e dos
hero/detail antigos ec96a3441fef43e2 impedem comparação com outra casa ou alteração
silenciosa do legado. A cena72 apresentada não é sobrescrita.

Comparação fixa hero/detail/front, luz direcional/sombra/câmera idênticas.
Pedra recebe grão fino55ciclos/unidade e manchas amplas1.1; ardósia combina
camada ridged anisotrópica34/6/34 com manchas2.5/1/2.5; carvalho usa65/1.2/65
mais variação1.4/.7/1.4. São frequências no domínioM, não garantias de centímetros
físicos se o domínio da peça for deformado/escalado. Tintas e relevo pequenos
são escolhas autorais reproduzíveis, não presets universais de materiais reais.

O estudo compara toda a cena excluindo somente materiais/version/câmera:
posições, índices, domínios, rig/clips, definições/fontes,19instâncias e suas
cores/transforms intactos. Cinco colunas mantêm vínculo de planta por seções.
15assets/32172v/53376t armazenados/61312t desenhados; budgets sem alteração.
Acrescenta80B/slot no uniforme, sem vértices/draws/texturas novos.

Saídas estáveis em `artifacts/cottage-materials/artifacts/`:
`before-{hero,detail,front}.png`, `after-{hero,detail,front}.png`,
`primary-only.png`, controles `neutral-{before,after}.png`, replay e report.json.
Ablação remove somente o segundo campo e muda a imagem; controle sem materiais
é byte-exato. Preview puro equivale ao lote publicado; undo/redo, repeat,
save/load/reset, replay/prefixo e resize restauram os pixels esperados.

## Revisão visual e próximos problemas

O primeiro efeito é reduzir manchas de relevo da pedra e controlar separadamente
granularidade/envelhecimento; não há critério quantitativo de realismo cumprido.
Os deltas de imagem no report medem efeito, não qualidade nem fidelidade.
A distribuição procedural não entende escorrimento, juntas, musgo ou desgaste de
quinas: isso exigiria máscaras/domínios e evidência próprios. Toda peça agrupada
compartilha o domínio; direção da fibra por peça continua uma limitação de autoria.

A geometria permanece divergente da referência: pórtico comprimido, bay/empena/
coberturas mal relacionados, pilares e nervuras simplificados. O diagnóstico
[elven-cottage-review.md](elven-cottage-review.md) é independente deste shader.
Vidro continua opaco, bronze usa lobo dielétrico e a lanterna não emite luz
pontual. Aumentar contraste, trocar câmera ou subdividir tubos não resolve esses
itens. Próxima unidade deve validar relações espaciais/vazios antes de acrescentar
mais motivos, sem confundir decisão autoral com ferramenta ausente.
