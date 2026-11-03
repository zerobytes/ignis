# Chanfros de pedra e piso da casa

Recorte autorizado após1054577/receita v4; fonte inicial SHAa742696f…57e6.
Arestas perfeitas e repetição retangular tornam bloco/piso artificiais. A frente
Sweep/arcos está isolada; este caso usa construtor genérico de caixas chanfradas,
documentado em [chamfer-boxes.md](chamfer-boxes.md), sem editar seu kernel.

## Plano/checklist anterior

Aplicar chanfro a14blocos alternados da parede direita da entrada e26lajes do
apron. Caixa nativa24v substitui caixa cliente8v:40×16=640v adicionais, dos812
livres. Nenhum aumento de limite. Piso também recebe alternância meia-peça em
uma fileira e variação limitada de largura/profundidade; manter o patamar/escada.
PRNG de pedra separado da receita existente para preservar vegetação e coberturas.

1. Só autoria, sem tick novo.2. Pré-contar saídas, append por família material,
nenhum slot extra.3. Centro/dimensões/quaternion/chanfro finitos, f64→f32 validado
nativamente.4. Sem matemática de faces/índices do chanfro no cliente.
5. Manter caps e reservar640v, não chanfrar toda a casa indiscriminadamente.
6. Câmera/luz/material/renderer hero constantes; usar comparação clay.
7. MCP próprio/headless e build corrente.8. Preview/publicação/undo/replay da
receita; fontes de rig/planta não são regeneradas por este gerador.
9. Closes e comparação v4/v5, topologia dos assets, duas gerações repetíveis,
regressão Rust/Clippy/MCP do kernel novo.10. Chanfro plano não é desgaste físico
ou arredondamento geral; caixa/módulo continua sem colisão arquitetônica.

## Resultado e evidência

Receita clientev5:14blocos alternados da parede à direita da porta e26lajes do
apron usam `chamfer_boxes`, com orientaçãoY e larguras de chanfro limitadas.
As demais caixas permanecem como antes. A fileira central do apron desloca-se
meia peça; largura/profundidade de lajes selecionadas varia por PRNG exclusivo.
Não há desgaste aleatório de vértices nem mudança dos degraus/molduras/ornamentos.

32596v/54728t armazenados,55948t no draw;16assets/21instâncias,360Sweep e40caixas
chanfradas. Crescimento640v/1280t,172v livres e zero slots. Sem ampliação de teto.
Resultado demonstra uma ferramenta genérica de autoria; a geometria continua
simples e os chanfros planos não produzem pedra envelhecida por si sós.

`scripts/study_cottage_stone.py` carrega antes/depois pelo MCP e prova que somente
stone-blocks.geometry e paving.geometry mudaram; propriedades dos assets, demais
assets/plantas/instâncias/câmera/luz e restante da cena iguais. Topologia dos dois
assets finais é fechada/manifold, sem vértices isolados. Closes nativos de piso e
parede têm câmera/restauração registradas em artifacts/cottage-stone/artifacts/
stone-report.json. Herói/entrada/clay usam compare_cottage_structure.py:

```sh
python3 scripts/compare_cottage_structure.py --root artifacts/cottage-stone \
  --before artifacts/cottage-stone/examples/before.scene.json \
  --baseline-image artifacts/cottage-relief/artifacts/after-hero.png
python3 scripts/study_cottage_stone.py
```

Antes extraído de1054577:examples/elven-cottage.scene.json, SHAa742696f…57e6.
Baseline preservou pixels no renderer inalterado.15checks da receita passaram,
incluindo previews, topologia, contornos, relevos, erro atômico, editar coluna/
undo/redo, save/load/reset/replay/prefixo/repetição/viewport. Duas gerações da
casa deram cena e hero byte-idênticos. Build final945ebbe0ba1b6c41, conferido com
fontes finais;860Rust/Clippy e87cenários/358grupos MCP headless do núcleo passaram.
Python compile/fmt/diff check aprovados. Sem janela/demo/main ou outra worktree
alterada; referência e árvore anteriores preservadas.

Publicação compacta2.351.797bytes, SHA
`88b5a8410e62d5db58a2db0f5b00da27b624322a000514f73ee2056b98fb823b`, reaberta igual
ao export nativo e PNG idêntico. Artefatos correntes em artifacts/elven-cottage/
artifacts e artifacts/cottage-stone: authoring/comparison/study.log, report,
stone-report/publication/build-audit.json e before/after-paving/entry-stone.png.

## Dificuldades e próximos contratos

A limitação dominante agora é o orçamento do documento:172v livres restringem
até detalhes modestos. Mantivemos seleção explícita e caps nesta prova; não
usar a seleção de40peças como evidência de que a pedra da casa inteira foi
refinada. Composição/particionamento/LOD, orçamento observável e material por
região exigem decisão própria. Aumentar um literal do teto sem tratar custo e
organização não resolve esse fluxo de autoria.

Também segue pendente a consulta de apoio/folga na superfície visual: os relevos
do friso da v4 quase ficam enterrados nos capitéis. A frente paralela corrige sua
colocação com offset explícito e trabalha em spans por trecho do Sweep. Não foi
duplicado esse trabalho aqui, nem integrada sua variante de mola/normais/lookdev.

## Revisão estática final

1. Apenas autoria/consulta.2.40caixas geradas na engine por dois appends; vetores
pré-contados e mesma família material, sem slots novos.3. QuaternionY unitário,
chanfro≤menor dimensão/4 e geometriaf32 validada.4. PRNG novo só para pedra,
sequência antiga conservada para restante da casa e comprovada por igualdade.
5.640v usados de812livres, tetos mantidos; nenhum ganho de FPS medido.
6. Mesmo renderer/material/câmera/luz; closes e clay preservados.
7. Headless próprio, sem processos/janelas de outra frente.
8. Preview/publicação/undo/replay/loading testados; IDs de entidades preservados,
ranges de peças apenas no relatório do resultado.
9.15checks da casa e regressão completa do kernel novo; duas gerações e
publicação byte/pixel equivalentes.10. Mantidos: staging/JSON síncronos, peças
modulares, piso ainda retangular e metal/folhas/jardim simplificados. Sem bevel
arbitrário, arredondamento contínuo, física ou casa realista/final alegada.
