# Casa: primeira revisão estrutural e ferramentas de varredura

Registro histórico de b33a76c/receita v2. A continuidade corrente está em
[cornijas e coberturas](cottage-roof.md), com receita v3.

Continuação autorizada após [diagnóstico arquitetônico](elven-cottage-review.md).
O alvo permanece a referência original. A imagem única não determina uma planta
oculta única; esta é uma hipótese revisada de composição, não reconstrução exata.

## Alteração da casa

Bay vinculada à extremidade+X, com centro em[3.6,-.65] no planoXZ; a empena alta
pertence à mesma extremidade. Quatro painéis mais largos substituem os cinco
painéis estreitos da bay frontal. Quatro é a hipótese construída para este ritmo
projetado; não uma contagem certificada das janelas ocultas da referência.

O pórtico externo passa a ter largura3m e arco mais baixo/aberto, distinto da
ogiva da porta. A parede da entrada avança paraz2.1, enquanto a parede de fundo
da varanda continua emz1.1. A folha está emx1.3,z2.265, recuada0.885m em relação
à frentez3.15. Só alargar o pórtico conservava uma oclusão forte pela coluna
direita; a revisão explicitou o plano da porta para corrigir essa relação.
Patamar e escada ampliados; lanterna e vaso seguem o novo plano. Nenhuma dessas
distâncias é apresentada como medida inferida do PNG.

Pilares usam seção retangular com pedestais/capitéis, em seis instâncias da mesma
planta. Guarda-corpo elevado e nervuras com perfil chanfrado/escala variável.
O beiral sobe sobre a entrada, com campo verde, moldura e revestimento de
intradorso. Campos verdes separam-se dos vidros opacos em um asset próprio.
Geometria nova usa16assets/21instâncias,32390v/54068t armazenados e55288t no draw;
378v livres, último slot de asset ocupado. Caps mantidos. Jardim foi reduzido de
24 para20aglomerados para reservar geometria ao pórtico, sem ocultar a entrada.

Câmera/luz do hero foram conservadas em relação a ea8c6b8. Os mesmos campos de
material dos assets anteriores continuam; green-panels separa um novo domínio
de aparência que antes compartilhava window-infill. A comparação adicional em
clay elimina a diferença de aparência. Não foi integrado o trabalho paralelo de
materiais/metais/ambiente nem alterado shader/renderizador nesta unidade.

## O que se tornou nativo

[MeshEdit.sweep e plan_mesh_sweep](mesh-sweep.md): perfil convexo em cadeia
Bézier3D, escala, transporte de frames, faces e coordenadas de material gerados
na engine.323peças de molduras/nervuras neste estudo usam essas operações;
13exigiram resolução maior que a solicitação inicial. O planejador nativo
escolheu spans explícitos, com maior limite da linha central≈0.01181347m para
tolerância0.012m. Isso não mede erro da superfície completa nem fidelidade ao PNG.
Preview ainda admite a geometria final e eventuais dobras.

`append:true` permite misturar blocos e molduras no mesmo asset de material,
sem o cliente reconstituir sua malha ou consumir outro slot. Preserva os dados
existentes e recusa rig. Plantas vinculadas continuam protegidas. As curvas de
arco e as pernas retas são Béziers/perfis declarados; o cliente também especifica
qual eixo do perfil corresponde à profundidade de cada fachada.

Coluna continua planta por seções. Telhados, telhas, blocos, painéis e folhagem
ainda têm matemática no cliente; as cornijas que recebem um caminho poligonal
com muitos pontos também usam o helper anterior. Não declarar a casa inteira
derivada de plantas, motor de interpretação de imagem ou solver arquitetônico.

## Dificuldades reproduzidas e tratamento

| Dificuldade | Evidência | Tratamento nesta unidade |
|---|---|---|
| Planta de eixoY não descreve uma ogiva inteira que sobe/desce |Mesh.curve/tube antes calculavam a malha completa; loft exigia anéis|Sweep3D/perfil nativo; planta/rig não foram generalizados implicitamente|
| Adicionar molduras a famílias materiais gastaria slots ou exigiria remesclar arrays no cliente |Casa anterior já usava15assets|Append nativo sem rig, preflight antes do clone e canais antigos conservados|
| Número de intervalos escolhido por tentativa |13recusas concretas de tolerância, registradas em artifacts/sweep-probe/artifacts/failures.json|Planejador puro com limites/custos; erros identificam índice da peça|
| Mudar implantação não conservou legibilidade da porta |Primeiro pórtico largo ainda tinha porta encoberta pela coluna|Plano da entrada explicitado; corrigida autoria e conferida captura com câmera fixa|
| Material por asset mistura funções diferentes |Campo verde e vidro usavam window-infill|Separação usa o último slot; materiais por região continuam questão estrutural a discutir|
| IDs das peças não sobrevivem como entidades dentro de assets agrupados |Relatório usa intervalos de geometria/ordem das varreduras; layout muda no cliente|Limitação registrada; ainda sem documento nativo de componentes/relações|
| Jardim/props têm posições em mundo |Mudar a entrada exigiu reposicionar vaso, lanterna e trepadeira|Ancoragens corrigidas no cliente; não foi criada montagem automática de superfícies|

Os três primeiros são capacidades novas. Implantação, eixo do volume e seção
elíptica eram escolhas de autoria possíveis de corrigir com capacidades já
existentes. Os demais não justificam automaticamente aumentar budgets ou criar
um gerador específico desta casa. Próximas alternativas para discutir: componentes
com frames locais e relações versus melhorar agrupamento/material por região.

## Reprodução e evidência

Receita clientev2: examples/elven-cottage.recipe.json. Gerar com
`python3 scripts/demo_elven_cottage.py`; verificação própria e capturas reais em
artifacts/elven-cottage/artifacts. Cena corrente em examples/elven-cottage.scene.json.
Para reproduzir a v1, usar script/receita do commit ea8c6b8; não interpretar a
receita antiga com a semântica espacial nova. Scene72/replay76 não mudaram.

`python3 scripts/compare_cottage_structure.py` usa baseline preservado em
artifacts/elven-cottage-review/baseline.scene.json e cena corrente do ensaio.
Baseline também está no Git em acb886a:examples/elven-cottage.scene.json.
Saídas em artifacts/cottage-structure/artifacts: before/after-hero,entry,clay.png
e report.json, com câmera/luz iguais e hash das fontes. Baseline ec96 preserva
pixels no novo binário porque a renderização não mudou.

Ainda há divergências visíveis: relevo/rendilhado simplificados, junção entre
empena e coberturas menos integrada que a referência, pavimento regular, vidro
opaco, jardim esparso e costas/interior inferidos. Encontros são peças modulares,
não uma superfície global soldada. Versão para revisão artística, sem alegar
realismo completo, aprovação humana ou ganho de desempenho.

## Revisão estática da receita

Checklist anterior desta continuidade: manter caps/build corrente, publicar pelo
dispatcher, usar preview antes da geometria nova e repetir save/load/undo/replay;
reservar espaço da escada e molduras antes de adicionar ornamento. A revisão
matemática do kernel e seus custos próprios estão em mesh-sweep.md.

1. Somente autoria no cliente; sem lógica de casa por tick na engine.
2. Família material agrupa peças; append evita tráfego de malha antiga no pedido
de varredura, mas staging ainda valida/clona a transação.
3. Perfil e controles têm números finitos; geometria admite f32 no kernel.
4. Cliente usa plano nativo uma vez por família e publica spans explícitos;
preview/commit recomputam por escolha de atomicidade existente.
5.16assets/21instâncias/32390v: sem ampliação dos limites;378v restantes e zero
slots são limitação concreta, não justificativa para expansão automática.
6. Renderer/cache/material comuns; captura explícita e comparação com câmera/luz
iguais. Material verde separado é mudança da receita.
7. Um processo MCP próprio por ensaio; janela e outras frentes preservadas.
8. Grupos de partes continuam intervalos transitórios no relatório; não existem
IDs persistentes para cada moldura agrupada.
9. Dois ensaios novos produziram cena e hero byte-idênticos, mais11checks próprios
e comparação controlada. Cena compacta carregada pelo MCP; rastros em artifacts.
10. Mantidos: CPU/JSON da receita, jardim simples, módulos sem weld, ausência de
interior/colisão e edição de relações no cliente; sem ganho de FPS alegado.
