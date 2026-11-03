# Cliente declarativo de reconstrução

`scripts/reconstruction_recipe.py` é um cliente MCP reutilizável. Não é uma
nova operação Rust nem um modelo de inferência. `sources` guarda definições
nativas de plantas/lofts; `parts` instancia essas fontes em frames rígidos;
`materials` agrupa a saída. Todos os parâmetros de um objeto ficam em JSON.

- Superfícies são geradas por `preview_scene_edits`/`preview_plant`; pincéis
  opcionais usam `sculpt` nativo. Não existe tesselação PCHIP no cliente.
- `hollow` gera lofts externo/interno nativos com mesma topologia, remove tampas
  e une anéis no cliente. Essa composição de cascas NÃO é uma ferramenta
  nativa nova. O teste independente confere volume material e centro aberto.
- `chains` distribui segmentos e folhas por dados; `anchor: base` identifica
  a origem da folha no ramo, em vez do centro da peça.
- `project_to` escolhe um host explícito. A projeção é ao longo do eixo+Z do
  MUNDO, não pela câmera. `fit_host_band` ajusta um padrão monotônico emY à
  seção horizontal do host, preservando sua linha de referência e espessura.
  Não é uma transferência de rig/escultura com o contrato nativo de plantas.
- Projeção parcial pode causar espinhos; quando algum vértice não tem suporte,
  a peça inteira permanece original e é marcada pendente no relatório. Não
  completar silenciosamente o contato nem chamar a peça de aprovada.

Até1500partes/fontes, arquivo8MiB e cache de previews até duas vezes o teto de
vértices; publicação respeita os budgets reais consultados. Falha de preflight
antecede a mutação final. O lote usa staging/revisão/undo nativos. O cliente
conserva faixas parte→índices na cena agrupada; individualidade e fonte de cada
parte estão no JSON do cliente, não em novas entidades paramétricas nativas.
Regenerar substitui o conjunto; não conserva edições independentes posteriores
sobre a malha publicada, escultura ou rig por inferência.

O comando `build` captura, salva, confere carga/undo/replay ESTÁTICO, compara
regiões e gera orbit.html. É execução de dados autorados, não interpretação
automática da imagem. Câmera manual e projeções auxiliares não provam fidelidade.

```sh
python3 scripts/reconstruction_recipe.py probe --root artifacts/fox-character
python3 scripts/reconstruction_recipe.py build --root artifacts/fox-character --recipe docs/references/image-to-3d/fox-character-recipe.json
python3 -m unittest discover -s scripts -p test_reconstruction_recipe.py -v
```

`--binary` permite selecionar outro executável compatível. Esta unidade foi
verificada apenas no macOS arm64/Metal do build397d7865d7051b9b. A referência
versionada é copiada se o root estiver vazio e seu hash é verificado. A galeria
mínima é criada em um novo root; a análise/diagnóstico completo do ensaio fica
nos documentos e artefatos associados. O viewer tem iluminação simples própria;
compare fidelidade apenas nas capturas nativas.

Cinco testes independentes cobrem transformações destrógiras, frames/escala,
orientação por extremidades inclusive antiparalela, volume e abertura da casca,
projeção de superfície e ancoragem/repetição. Testes não aprovam um objeto.
