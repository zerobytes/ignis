# Painéis com espessura e aberturas

## Contrato e alternativas antes do código

Um agente informa dimensões e retângulos vazios; o kernel constrói a superfície
material completa. Triangulação por imagem mantém o erro no cliente. CSG geral
permitiria cortes arbitrários, mas acrescentaria tolerâncias, interseções e
preservação de atributos muito maiores. Este recorte usa decomposição cartesiana
limitada, determinística, com índices compartilhados e jambas reais.

`edit_mesh` e `preview_mesh_edit` recebem `edit.kind: panel`, `id`,
`dimensions: [largura, altura, espessura]` e `openings: [{min:[x,y],max:[x,y]}]`.
XY local, centro na origem, espessura simétrica em Z. Instâncias existentes
orientam o painel no mundo. Dimensões0.01..100; até16 aberturas; medidas locais
finitas. Vãos têm largura/altura e margens mínimas0.001. Apenas a margem inferior
pode ser zero, para uma porta que começa na base. Retângulos sobrepostos ou
separados por menos de0.001 nos dois eixos são recusados. Não há ajuste silencioso.

A malha contém frente, verso, contorno e faces de espessura nos vãos. Interior
vazio não recebe triângulos. Apenas limites entre células cheias/vazias emitem
faces laterais; nenhuma parede interna redundante entre células cheias.
Sem weld por proximidade: vértices compartilham a mesma chave de grade.

Como `grid`, a operação substitui o asset completo; não conserva rig, clipes,
escultura ou material anteriores. Fonte vinculada a planta exige detach explícito.
A edição em lote e preview usam o staging/validação existentes. A cena persiste
a malha resultante, não uma nova fonte paramétrica; guardar parâmetros no documento
do cliente permite regenerar. Nenhum schema de cena/replay ou dependência muda.

Não é um gerador de casas, solver de câmera, inferência de imagem, parede curva,
vão em arco, associação de faces entre assets ou colisão de gameplay. A topologia
demonstra fechamento do MATERIAL, não preenchimento do vazio da porta. Compor
paredes isoladas não prova que o edifício tenha encontros corretos.

## Revisão estática anterior

1. Execução só na autoria/preview, sem tick/frame.
2. Até34 coordenadas por eixo,2312 slots de vértices; arrays limitados.
3. Chaves numéricas de grade, sem nomes/string por triângulo.
4. Classificação limitada a33×33×16; sem busca espacial irrestrita.
5. Até16 vãos, limites existentes de asset/documento conservados.
6. GPU/renderer/cache inalterados; captura explícita para evidência.
7. Mesmo dispatcher/staging, sem thread, lock ou dependência nova.
8. Substituição total explícita; planta vinculada protegida, undo existente.
9. Testar volume analítico, incidência/orientação/raios, variações e erros,
   preview/undo/save/load/replay e captura MCP; regressão headless.
10. Custos mantidos: grade pode subdividir regiões além do necessário, cópias
    de staging/JSON e construção síncrona limitada. Sem alegar ganho de FPS.

## Exemplo pelo MCP

```json
{"command":{"op":"edit_mesh","edit":{"kind":"panel","id":"wall","dimensions":[6,4,0.3],"openings":[{"min":[-2,-2],"max":[-1,0.5]},{"min":[0,-0.5],"max":[1.5,1]}]}}}
```

Nesse exemplo a porta começa emY=-2, a base do painel. O segundo retângulo é
uma janela. Para conferir sem publicar, trocar `op` por `preview_mesh_edit`.
A instância posicionada emY=2 apoia a base emY=0. Inspecionar com
`inspect_mesh`, `inspect_mesh_topology` e capturar frente/verso/ângulos laterais.
Uma janela preserva uma passagem vazia; vidro/folha de porta são assets separados.

O schema MCP expõe parâmetros/tipos; os limites relacionais acima são conferidos
no kernel e retornam erro. Alterar dimensões não reposiciona automaticamente os
vãos: todos continuam em coordenadas locais explícitas, podendo tornar o pedido
inválido. O emissor deve recalcular parâmetros a partir da hipótese/medidas.

## Reprodução

```sh
python3 scripts/run_verifications.py --only mesh_panels --binary /caminho/ignis
```

Cenário constrói dois painéis por parâmetros, sem transmitir triângulos;
resultados correntes em `artifacts/mcp-validation/artifacts/panels-*` e cena
`artifacts/mcp-validation/examples/native-panels.scene.json`. Testes Rust em
`tests/mesh_panel.rs` incluem raios independentes e volume analítico. Auditoria
independente Python confere a malha retornada pelo MCP; não é um novo certificado
semântico no runtime. Reprodução de replay é estática; sem prova de gameplay.

## Resultado verificado — EAF-106

Build `397d7865d7051b9b`, macOS arm64/Metal.871 testes Rust/all-targets,
Clippy com warnings negados, fmt e84cenários MCP/347checks aprovados
sem janela. Oito checks específicos MCP e quatro testes Rust novos.
Pedido com porta+janela produz60v/120t, uma componente, volume5.77500023
(esperado5.775), zero arestas abertas/não manifold/inconsistentes, sem isolados.
Raios independentes atravessam os dois vãos e encontram material fora deles.
Segundo conjunto de dimensões/vãos, rotação/escala, A→B→A, prévia, recusa
atômica, lote falho, vínculo de fonte, carga/undo/replay estático cobertos.
Capturas reais de frente/verso foram abertas e conferidas; replay/carga restauram
pixels exatos. Sem teste de interação humana ou nova colisão de gameplay.

[Cena nativa de demonstração](../examples/native-panels.scene.json) e
[relatório resumido](references/image-to-3d/mesh-panels-verification.json).
O caso extremo16vãos também confere volume/orientação/arestas no teste Rust.
Não houve alteração da E02 nem validação de inferência visual nesta unidade.

Revisão final: execução exclusiva na autoria, máximo2312slots; grades numéricas
compartilham índices, sem faces internas entre células. Sem dependência nova,
formato de cena, trabalho por tick/frame, renderer ou cache novo. Staging/Arc e
proteção de plantas existentes reutilizados. Custos conscientemente mantidos:
malha mais subdividida que um triangulador ótimo, cópias de preview/JSON,
compilação síncrona e substituição integral sem preservar detalhe. Sem ganho de
FPS ou fechamento de um edifício alegado. A análise da imagem e as relações entre
partes continuam responsabilidades distintas ainda sem um solver semântico.
