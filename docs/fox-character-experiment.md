# Personagem canídeo — ensaio de reconstrução por imagem

Fonte: PNG1024×1536 fornecido pelo usuário. Objetivo: testar as capacidades gerais
da Ignis com anatomia orgânica e roupa em camadas, não preparar um asset final
por outra ferramenta. Usar skill ignis-image-to-3d e MCP público do build corrente.
Sem Blender, modelo externo ou projeção da foto sobre uma fachada.

## Decisão e revisão estática antes da autoria

A alternativa de outra receita Python com fórmulas exclusivas para o personagem
não demonstra o produto. Usar definições de loft/seções/escultura já existentes
no MCP. As medidas, poses, camadas e motivos específicos ficam em dados.
Um cliente reutilizável pode publicar lotes e agrupar cópias transformadas de
malhas retornadas pela engine; isso é composição no cliente, não capacidade
nativa nova. Não reimplementar no cliente amostragem PCHIP ou triangulação dos
lofts. Corpos sobrepostos de roupa/anatomia não implicam união topológica.

1. Autoria/captura sob demanda; nenhum tick novo.
2. Entradas e partes limitadas, limites consultados antes de publicar; cache
   de fontes equivalentes evita gerar cada botão/folha de novo.
3. Índices numéricos para cópias e correspondências; IDs para rastreabilidade.
4. Agrupar por material após obter geometria nativa; preflight de somas antes
   da edição. Sem orçamento ampliado nem exclusão silenciosa de tarefas.
5. Manter16assets/32instâncias/16384v e limites reais do build; detalhes que
   excederem capacidade permanecem explicitamente divergentes/ausentes.
6. Mesmo renderer, seis capturas1024×1536 e pares locais; artefatos estáveis.
7. Processo MCP próprio, sem janelas ou intervenção na demo; sem downloads.
8. Fonte declarativa separada do exportado, histórico/undo/replay existentes.
9. Testar formas/transformações genéricas do cliente e cena real, geometria
   por parte, load/undo/replay estático e captures; registrar limites da prova.
10. Custos mantidos: I/O/JSON e staging, instâncias materializadas por grupo,
    observação humana/IA, source graph do cliente sem solver semântico nativo.

## Fronteiras

Imagem única não informa traseira, interior, anatomia sob roupa ou dimensões
métricas. Essas escolhas são hipóteses. Pelo não é simulado; detalhes ilegíveis
não devem virar informação fabricada. Peças e roupa precisam ter profundidade
e sobreposição coerentes nas vistas auxiliares. Um bom render frontal, SSIM
ou corpo fechado não aprova semelhança do rosto, anatomia ou acabamento.

## Execução e resultado

A lista de146tarefas/17conjuntos foi entregue antes da geometria, depois de
18recortes abertos e7revisitações. Recortes menores de olhos/focinho/orelhas/
chapéu e broche estão ligados aos contextos. A fonte versionada
[fox-character.png](references/image-to-3d/fox-character.png) preserva os pixels
1024×1536 e SHA2560339c215…2f14. A conversa anterior continua no contexto;
não apresentar este ensaio como teste cego de outro agente.

A [receita declarativa](references/image-to-3d/fox-character-recipe.json) usa
lofts elípticos/PCHIP e dois pincéis orbitais nativos. O
[cliente reutilizável](reconstruction-recipes.md) compõe fontes, cascas e
adornos; essas operações do cliente não foram incorporadas ao núcleo/MCP.
Não há Blender, modelo neural externo ou foto aplicada sobre uma fachada.
O arquivo da casa e sua receita não participam da construção deste personagem.

Saída:12assets/12instâncias,9726v/18096t,329partes contando chão/fundo;
118previews distintos, com reutilização de fontes equivalentes. Tetos mantidos:
16384v/32768t no documento. A cena nativa está em
[fox-character.scene.json](../examples/fox-character.scene.json).
Fontes individuais e relações ficam no JSON cliente, não como329entidades
paramétricas na cena. Não há rig, animação ou simulação de tecido/pelo.

A primeira leitura do render expôs rosto cartunesco, olhos salientes, orelhas
altas, dedos rígidos, casaco esquemático e adornos flutuantes. Revisões alteraram
proporções/camadas em dados, abriram punhos e golas com cascas reais, assentaram
adornos sobre hosts nomeados e separaram cenário de apresentação das vistas
auxiliares. A projeção parcial gerava espinhos: a regra geral passou a rejeitar
projeção incompleta da peça inteira. Ancorar folhas pela base e manter a linha
de referência do ramo evitou quebrar o grafo do padrão. Quatro adornos continuam
sem cobertura total (16de2528v), registrados como pendentes, não aprovados.

Seis vistas e18pares locais fixos foram abertos. Fonte e candidato permanecem
1024×1536, sem alinhamento/redimensionamento por peça. A [revisão por recorte](references/image-to-3d/fox-character-review.json)
registra divergências. **O resultado foi reprovado em fidelidade visual.**
Rosto, anatomia, pose, corte/dobras, ornamentação, pelo e materiais não reproduzem
a referência. Costas e interior são hipóteses; existem sobreposições e relações
não resolvidas. Uma superfície fechada por parte não certifica continuidade
anatômica, ausência de interseções, costura de roupa ou correspondência visual.

Métricas globais finais: RGBMAE0.091480, RMSE0.165666, SSIM-luma0.704219.
O fundo cinza semelhante participa fortemente desses números. SSIM não é
percentual de fidelidade e nenhum limiar aprovou o modelo. A métrica RGB piorou
em relação ao primeiro candidato; isso foi mantido explícito, sem selecionar
uma captura anterior apenas pelo score.

O [progresso](references/image-to-3d/fox-character-progress.json) conserva as146
tarefas:87têm vínculo explícito a geometria parcial;59não têm vínculo granular
ou permanecem ausentes. Zero tarefas receberam aceite de fidelidade. Número de
partes/fontes e campos de análise não são cobertura semântica.

## O que aprendemos para a ferramenta

- Fechamento topológico e geração de muitas peças não resolvem o formato do
  crânio, órbitas, focinho e boca. A representação escolhida deixou a superfície
  como massas sobrepostas, com transições inadequadas.
- Os pincéis existentes não foram explorados exaustivamente: dois traços não
  provam uma incapacidade matemática do núcleo. Falta um procedimento eficaz
  de superfície contínua, landmarks e revisão local que impeça substituí-la por
  olhos/bochechas avulsos inadequados.
- A lista precedeu a autoria, mas o publicador ainda gerou ornamentos com a
  forma insuficientemente refinada. O fluxo não impôs uma fronteira efetiva
  de avanço por fidelidade. A qualidade da análise textual não corrigiu essa
  falha de execução.
- Roupa precisa de painéis, espessura, dobras, costuras e vínculos verificáveis;
  uma sequência de lofts finos não determina um corte plausível.
- Assentamento explícito e diagnóstico de adornos sem host são ferramentas
  reutilizáveis, mas o modo atual é frontal no eixo mundialZ; generalizar
  exige contrato de superfície e domínio, sem transferir detalhe por proximidade
  silenciosamente ou fingir que essas operações já são nativas.

O orçamento deixou6658v livres. Este resultado não identifica o teto como causa
principal nem demonstra que aumentar polígonos faria a interpretação funcionar.
O próximo recorte deve melhorar e medir uma forma orgânica contínua antes de
expandir ornamentos; não continuar fazendo ajustes específicos neste figurino
como substituto de evolução do produto.

## Verificação e reprodução

[Relatório](references/image-to-3d/fox-character-verification.json):5testes
numéricos independentes do cliente aprovados. Pipeline público MCP/build
397d7865d7051b9b no macOS arm64/Metal; preview sem mutação, somas/limites,
329superfícies por parte fechadas/orientadas, carga exata e pixels de carga,
undo e replay ESTÁTICO tick0 idênticos. As329faixas foram extraídas novamente
da cena salva nativamente e auditadas; não só da receita prevista. Isso não
comprova colisão, malha única, animação ou aceite.

Nenhum Rust/renderer/schema/build foi alterado; testes Rust/Clippy e regressão
completa da unidade EAF-106 não foram repetidos como se fossem novos resultados.
Capturas nativas e viewer são distintos; WebGL/entrada física não têm aceite
humano. Native scene copiava bytes exatos do save, fonte preservada por hash.

```sh
python3 scripts/reconstruction_recipe.py build --root artifacts/fox-character --recipe docs/references/image-to-3d/fox-character-recipe.json
python3 -m unittest discover -s scripts -p test_reconstruction_recipe.py -v
```

[Comparação completa](../artifacts/fox-character/index.html) ·
[Modelo para girar](../artifacts/fox-character/orbit.html) ·
[Lista anterior à autoria](fox-character-list.md).

## Revisão estática final

Execução apenas por solicitação; nenhum tick/frame/renderer novo. Cache de fontes
nativas equivalentes, mapas numéricos por peça/material, limites de previews e
publicação antes de mutar. Mesmo MCP/revisões/staging/undo; sem biblioteca nova,
processo gráfico ou download. Leitura/exportação e artefatos em caminhos estáveis.
Custos mantidos conscientemente: JSON/cópias, validação síncrona por preview,
consulta de triângulos por vértice dos adornos, materialização de instâncias,
auditoria CPU por parte e avaliação visual pela IA. Sem alegar FPS, inferência
automática, tempo de geração imagem→3D ou qualidade pronta para produção.
