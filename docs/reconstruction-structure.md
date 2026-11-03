# Reconstrução: corpo 3D antes da projeção

## Causa e recorte desta correção

Na E02, `Camera.on_plane` e `Plane` posicionaram contornos da imagem em planos
independentes. `Mesh.poly2` gerou superfícies; as laterais de alguns contornos
não criaram traseiras, forros ou encontros compartilhados. `publish` verificava
budgets e `verify` verificava persistência/pixels, sem condição volumétrica.
A câmera auxiliar próxima não expôs o problema. A lista já exigia fechamento:
houve falha na execução e na condição de avanço, não falta desse item.

Primeiro incremento: auditor CPU de superfície, reutilizável e independente de
câmera, e bloqueio **no cliente de publicação E02**. A receita histórica não
fornece `building-core` e será recusada antes de abrir o processo MCP. Esse
asset deve ser a geometria efetiva do corpo principal, ainda sem ornamentos;
uma caixa auxiliar escondida não satisfaz a revisão de correspondência.
Nenhuma geometria existente foi corrigida ou substituída.

`scripts/reconstruction_structure.py` inspeciona JSON de cena ou geometria
obtida por `inspect_mesh`. A inspeção considera coincidência **exata** de
posições, sem editar/weldar a fonte, para não reprovar seams duplicados só
por índice. Não arredonda, aproxima nem fecha gaps. Reporta bordas abertas,
arestas não manifold, orientação, degenerações, componentes por aresta,
vértices isolados, bounds e volume assinado por componente. Em superfícies
abertas, esse valor é apenas diagnóstico e não representa volume encerrado.

O preflight do corpo principal exige uma componente, ausência desses defeitos
e volume orientado para fora acima de `diagonal³ × 1e-12` (tolerância numérica,
não qualidade). Usa origem local ao calcular volume, evitando cancelamento por
translação. Não soma volumes de ilhas para esconder folhas soltas ou cancelamentos.
O resultado chama-se `solid_preflight_passed`, nunca “casa validada”.

```sh
python3 scripts/reconstruction_structure.py examples/house-e02-rebuild.scene.json
python3 -m unittest discover -s scripts -p 'test_reconstruction_structure.py'
```

O auditor usa apenas a biblioteca padrão. Os testes que executam a receita
E02 precisam do Python com NumPy já exigido por essa receita.

A CLI apenas diagnostica; exige `--require-solid ASSET` para um teste obrigatório,
com exit 1 se não passar e exit 2 para dados inválidos. Em cena, mede geometrias
de asset em repouso, não instâncias transformadas/animadas nem entidades físicas.
Resultado não depende da câmera nem de nomes de materiais. JSON tem hash da
entrada exata e versão de algoritmo; não reutilizar após mudar a geometria.

## O que deve orientar a próxima construção

1. Inventário e recortes continuam necessários. Identificar corpo fechado,
   projeções, espaços abertos e suportes, com observação/hipótese separadas.
2. Definir planta e seções em coordenadas do objeto. Paredes e coberturas
   derivam de volumes e bordas compartilhadas. A câmera mede a projeção;
   não determina profundidades independentes para cada detalhe.
3. Construir o corpo principal simples. Porta aberta deve ter batentes,
   espessura e continuidade com o interior; janela deve ter vão e tratamento
   coerente do interior/vidro. O fechamento pode incluir superfícies internas,
   sem tampar aberturas visíveis para passar no teste. Varanda aberta, folhas,
   grades e painéis têm contratos próprios: não exigir tudo watertight.
4. Conferir geometria real, planta, seções, frente, lados, verso e superior.
   Marcar o verso inferido como hipótese. Resolver junções, apoio e separações
   antes de detalhes. Não exigir referência inexistente para o verso.
5. Só depois ajustar câmera, proporções e detalhes por recortes. Mudanças
   estruturais invalidam a inspeção anterior. Software e revisão visual devem
   concordar; ausência de evidência deixa a etapa pendente.

## Limites e próximo contrato de engine

Fechamento não prova ausência de autointerseção, manifold nos vértices,
espessura mínima, apoio, habitabilidade, conectividade entre assets ou semântica.
Um cubo passa e pode ser uma casa completamente errada. Inspeção visual continua
necessária. Não adicionar score global que compense falhas estruturais com detalhes.

Alternativas: só instrução é barata mas não impõe execução; rejeitar todo mesh
aberto quebra vegetação/aberturas; um contrato semântico persistido permite
vincular volumes, faces, vãos, apoios e junções a geometria efetiva. A terceira
opção é a direção proposta para o dispatcher/MCP, com custo de versionamento,
seleções e invalidação. Não foi implementada por este script. `edit_mesh` e
`save_scene` continuam genéricos; não existe bloqueio universal da engine.

## Revisão estática anterior ao código

1. Só autoria/consulta explícita, nenhum tick/frame. 2. Mapas de posições,
arestas e adjacências limitados à entrada; sem cópias por instância. 3. Inteiros
para índices, float64 para geometria e somas locais. 4. Incidência e componentes
em passes compartilhados, sem busca espacial/par por triângulo. 5. Preflight
de 8192 vértices/16384 faces por asset e 16 assets/32MiB por arquivo; O(V+T)
esperado com hash maps, saída por componente limitada pelas faces. 6. Sem GPU,
render ou modelo; JSON apenas sob pedido. 7. Sem engine/threads/locks no auditor.
8. Sem cache; hash da entrada no relatório. 9. Testar seams, face ausente,
orientação invertida, duas ilhas, degeneração, não manifold, volume nulo,
translação, dados inválidos e a cena E02 real; comprovar recusa antes de Engine.
10. Custos Python/JSON e critério conservador de coincidência exata mantidos;
integração nativa e semântica são unidades posteriores. Sem alegação de FPS.

## Resultado da verificação

18 testes aprovados, incluindo execução da receita estrutural real com root
temporário e recusa anterior a qualquer abertura de MCP/render/cena. Também
testados corpo inválido, invalidação pelo detalhe e por remapeamento de vértices;
nenhum deles chega ao processo de publicação. Python da receita com NumPy
preexistente; não houve instalação ou download.

[Diagnóstico rastreável da E02](references/image-to-3d/e02-structure-audit.json):
pedra tem2274 bordas abertas e141 componentes por aresta; cobertura906 e211.
São assets agrupados por material: os números não contam paredes ou objetos
semânticos nem são prova isolada de que todas essas ilhas são erros. A coluna
tem56 posições duplicadas e, após coincidência exata para a inspeção, nenhuma
borda aberta, uma componente e passa no preflight. Esse controle impede a
confusão entre seams de índice e ausência real de fechamento.

A cena e o PNG anteriores permanecem idênticos. Não há casa nova, fechamento
corrigido, benchmark, teste Rust/MCP novo ou certificado semântico. O relatório
completo de todos os assets está em `artifacts/house-e02-rebuild/structural-audit.json`.

Revisão posterior:1.só comando/autoria;2.mapas/filas por asset;3.valores/índices
checados e origem local;4.uma incidência e um percurso de componentes;
5.limites e saída por face mantidos;6.sem GPU/I/O no loop do produto;
7.sem engine/locks no auditor;8.sem cache/reuso entre geometrias;
9.18 contraprovas e checks de pureza/CLI/publicação;10.mantidos exatidão
conservadora, custo de JSON e ausência de análise de interseção/semântica.
