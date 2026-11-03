# E02 — por que a reconstrução não reproduziu a referência

Investigação de EAF-102, 27/10/2026, branch `codex/image-to-3d`.
Objeto: a ferramenta e seu procedimento, sem corrigir manualmente a casa.
Cena, captura anterior e imagem Unreal preservadas. Esta unidade acrescenta
diagnóstico e contraprovas do avaliador; não implementa solver ou aprovação.

## Conclusão

A skill melhorou o inventário, mas esse inventário não foi convertido em um
problema de ajuste geométrico verificável. A receita construiu uma interpretação
aproximada com números escolhidos pela IA; o comparador mediu discrepâncias sem
controlar a continuação ou a conclusão. A execução parou com todas as fichas
divergentes/inconclusivas. Isso é uma falha do processo de reconstrução, mesmo
com persistência, reprodução e transparência técnica funcionando.

O próximo avanço deve transformar observações em restrições e ligar medições
ao estado de aprovação. Mais texto na skill, mais polígonos ou um corte global
de SSIM0.98, isoladamente, não resolvem essas falhas.

## Evidências e causas

| Constatação verificável | Efeito | Correção necessária |
|---|---|---|
| `reconstruct_e02.py` recebe `SOURCE` como caminho/metadado; `build_house()` gera números e geometrias sem ler pixels ou consumir um contrato de observações. | Não existe ainda um pipeline automático imagem → restrições → modelo. A ligação depende da interpretação e da programação feitas pela IA nesta execução. | Inventário geométrico legível por máquina, com coordenadas, relações e parâmetros vinculados. |
| `tree()` usa três níveis fixos `.32/.57/.76`, com largura e poucos segmentos; `opening()`, balaustrada e água-furtada reutilizam a função. | Representar a categoria “árvore” substituiu reproduzir o grafo e os contornos específicos. O texto das fichas era mais preciso que a geometria. | Nós/arestas/curvas observados por motivo; compartilhamento apenas onde a equivalência foi conferida. |
| `main_roof()`, `upper_roof()` e `porch_roof()` codificam curvas com constantes e variantes manuais. Câmera também é fixa manualmente. | Pontos extremos podem coincidir enquanto curvatura, orientação, profundidade e interseções internas divergem. | Ajuste conjunto controlado de câmera e forma contra âncoras distribuídas, contornos e oclusões. |
| `render()` usa `--stage detail` por padrão; compara, salva e imprime métricas sem verificar uma condição de fidelidade. | A construção pode avançar aos detalhes com estrutura errada. Comparar é obrigatório na execução, corrigir até atingir um requisito não é. | Estados de etapa e requisitos de passagem no orquestrador, sem impedir salvar prévias honestamente incompletas. |
| Revisão contém17fichas divergentes e1inconclusiva;18ações seguintes permanecem. | As divergências foram documentadas, mas não resolvidas. O encerramento técnico foi permitido apesar disso. | Pendências críticas devem impedir `accepted`; ausência de progresso deve gerar resultado incompleto com causa e próxima ação. |
| `verify()` verifica pixels repetíveis, undo, carga/replay e resolução de IDs. O vínculo das fichas aponta principalmente para assets agrupados por material. | Esses checks não provam que o detalhe existe ou está correto. Um ID de “pedra” pode resolver sem identificar a coluna da ficha. | Correspondência por peça e subparte, com evidência projetada e revisão invalidada após mudanças relevantes. |
| O comparador mede RGB/SSIM em janelas fixas, sem segmentar ou distinguir geometria, luz e material. | Mesma janela pode mostrar peças diferentes; a média não explica a causa e permite compensações entre regiões. | Silhuetas, landmarks, contornos e testes locais próprios, mantendo a leitura visual dos pares. |

Fontes: [receita](../scripts/reconstruct_e02.py),
[extração de evidências](../scripts/e02_evidence.py),
[revisão das18fichas](references/image-to-3d/e02-review.json),
[skill](../.agents/skills/ignis-image-to-3d/SKILL.md),
[procedimento de detalhes](../.agents/skills/ignis-image-to-3d/references/detail-review.md),
[contrato do comparador](image-similarity.md).

Não faltou somente instrução: a skill já pedia grafo, contornos, estrutura antes
de detalhes e revalidação. A receita não materializou essas exigências em dados
e condições de execução. Também não há nela um loop que escolha parâmetros a
partir do erro e procure uma solução até um critério definido. Houve revisão
manual de variantes, não convergência automática demonstrada.

## Limites da engine: contribuição real e o que não foi provado

O build público usado expõe16assets,32instâncias e16384vértices totais.
A cena detalhada ocupa16assets/25instâncias/16218vértices, deixando166livres.
Isso restringe a quantidade e a independência das peças e pressionou a receita
a simplificar curvas, vegetação e ornamentos. O agrupamento por material
dificulta revisar uma peça isolada. A primeira expansão de35874v foi descartada
no cliente antes de publicar; não houve teste de fidelidade dessa alternativa.

Recontagem da própria receita, sem render ou mutação:

| Modo | Vértices diretos | Fonte nativa da coluna | Total armazenado esperado |
|---|---:|---:|---:|
| mass | 6104 | 80 | 6184 |
| detail | 16138 | 80 | 16218 |

O modo de massas não esgota o orçamento. Logo, falta de vértices não explica
por si só a ausência de um ajuste prévio de proporções, contornos e encaixes.
Esta contagem também não prova que o orçamento atual baste para todo o detalhe
da referência. É necessário testar custo por erro projetado antes de decidir
ampliar tetos ou adotar outra representação. Não foi executada uma comparação
de qualidade entre motores, nem demonstrada a máxima qualidade do renderer.

O schema público salvo no ensaio tem90operações, incluindo `set_camera`,
`capture`, `compare_images`, autoria e inspeção de meshes/plantas. Não expõe
operações dedicadas para solver de câmera, projeção de landmarks, passes de
ID/profundidade/silhueta ou avaliação geométrica por referência. Coordenadas e
relações continuam trabalho do cliente/IA. Confirmar novamente o schema do
build antes de implementar qualquer extensão; não inventar comandos na skill.

## Contraprova reproduzível: um corte global de0.98 aprova defeitos

[audit_e02_similarity.py](../scripts/audit_e02_similarity.py) usa uma cópia da
própria foto em processo MCP independente e preenche uma ROI inteira com sua
cor média. São fixtures adversariais do avaliador, **não modelos ou renders**.
O resto da foto fica inalterado. Nenhuma entrada do ensaio E02 é substituída.

| Fixture | SSIM global | Pixels exatamente iguais | SSIM na região apagada |
|---|---:|---:|---:|
| Identidade, controle | 1.000000 | 100% | 1.000000 |
| Apagar recorte do detalhe interno da porta,85×155 | 0.993235 | 99.1625% | 0.183477 |
| Apagar recorte do medalhão,225×100 | 0.987101 | 98.5697% | 0.115433 |

As duas corrupções passam por três regras globais ingênuas: SSIM≥0.98,
pixels idênticos≥98% e `1−MAE≥0.98`. Esta última transformação foi usada apenas
como contraprova; não é uma métrica de fidelidade adotada pelo produto.
O defeito na porta afeta menos de1% da imagem; a média o dilui mesmo sem
qualquer fundo artificial adicional. A ROI local detecta a perda que o total
esconde. O controle identidade também mostra que `compare_images` aceita uma
foto como candidato: sozinho, não autentica a existência de geometria 3D.

Resultados completos, hashes, build e checks em
[e02-fidelity-audit.json](references/image-to-3d/e02-fidelity-audit.json).
MAE foi conferido independentemente por diferenças de bytes com Pillow;
SSIM é o resultado nativo, sem alegar uma segunda implementação independente.
O teste prova uma vulnerabilidade de interpretação/agregação da nota, não
um defeito na implementação das fórmulas do comparador.

As capturas anteriores do modelo têm MAE0.179949→0.179525 e
SSIM0.391281→0.376478. Não demonstram melhora global; MAE e SSIM também
misturam diferenças de forma e aparência. Essas medidas não significam uma
porcentagem de casa reconstruída.

SSIM foi proposto para qualidade de imagens, incluindo avaliação de compressão,
e não certificação arquitetônica/3D
([artigo original](https://eceweb.uwaterloo.ca/~z70wang/publications/ssim.html)).
Uma métrica perceptual aprendida poderia complementar o diagnóstico; os autores
de [LPIPS](https://richzhang.github.io/PerceptualSimilarity/) a avaliaram contra
julgamentos perceptuais humanos. Ela não está instalada nesta auditoria e
adotá-la não eliminaria os requisitos geométricos e de procedência.

## Como tornar atalhos detectáveis

Proposta de arquitetura, ainda não implementada nem aprovada como contrato:

1. **Especificação de referência.** IA propõe peças, faces hospedeiras,
   contagens observáveis, máscaras, landmarks, polilinhas e grafos locais.
   Cada observação tem ID, pixels, contexto, incerteza e prioridade; oclusão
   permanece explícita. Essas anotações precisam de revisão: determinismo não
   torna correta uma interpretação errada da imagem.
2. **Ferramentas geométricas determinísticas.** Projetar pontos com a mesma
   câmera do render; capturar máscara/IDs/profundidade em condições declaradas;
   medir distância entre contornos e landmarks. Relacionar cada peça editável
   à sua evidência. Preferir operações compartilhadas entre UI e MCP.
3. **Ajuste de câmera e massas.** IA escolhe hipótese e parametrização; um
   otimizador limitado ajusta números contra observações. Começar com uma
   câmera/modelo simples, verificar ambiguidades e testar causas separadas.
   Uma implementação de mínimos quadrados ou busca numérica é uma escolha
   futura; não introduzir agora um framework/diferenciador caro por presunção.
4. **Passagem de etapa.** Estrutura, depois vãos/perfis, detalhes e aparência.
   Sem satisfazer critérios críticos da etapa, nenhum estado de aprovação.
   Prévia e exportação diagnóstica continuam possíveis com `accepted:false`.
   Mudança de câmera ou dependência invalida os checks afetados.
5. **Verificador separado do autor.** O orquestrador recolhe capturas nativas
   e calcula medidas; a IA revisa pares reais e interpreta diferenças. A IA
   construtora não pode declarar a própria peça aprovada só escrevendo texto,
   trocar a referência/ROI para melhorar a nota ou alterar a política de corte.
   Avaliação visual por IA permanece julgamento sujeito a erro; não é prova
   determinística. Independência aqui é de dados/permissões, não exige contratar
   um segundo modelo ou adicionar um agente nesta investigação.
6. **Resultados honestos e limitados.** `accepted`, `not_converged`,
   `insufficient_evidence` ou `capability_blocked`, com razões. Limitar tentativas
   e detectar ausência de progresso; atingir limite de tempo/custo não aprova a
   cena. A ferramenta controla sua própria certificação, não pode impedir toda
   declaração falsa de uma IA fora do seu fluxo.

Atalhos a reprovar no conjunto de avaliação: trocar grande arco por mancha,
omitir peça pequena crítica, repetir motivo incompatível, deslocar a peça e
recentralizar o crop para disfarçar, compensar erro estrutural com luz/cor,
usar a referência como candidato ou uma imagem plana como suposto volume.
Verificar procedência por captura nativa vinculada à revisão/câmera/hash;
testar geometria e vistas auxiliares para os atalhos espaciais. Vistas novas
verificam coerência 3D, sem afirmar fidelidade de faces não observadas.

## O que pode significar98%

Aceitar exigência alta de fidelidade exige definir a medida. Não há calibração
que permita declarar que SSIM0.98 equivale a98% da casa correta. Também não
há evidência de que o renderer atual consiga esse SSIM contra esta ilustração.
Manter a meta de0.98 para uma medida nomeada é possível; se não atingir, o
resultado deve continuar reprovado, sem baixar o corte silenciosamente.

Uma política candidata deve exigir condições simultâneas, sem média que
compense falha crítica:

- todos os requisitos críticos observáveis conferidos; desconhecido não conta
  como aprovado e simples presença de ID não conta como execução;
- ajuste de estrutura e câmera por landmarks/contornos em pixels, respeitando
  a incerteza da anotação;
- cobertura da silhueta do objeto, por exemplo alvo inicial IoU≥0.98, **proposto
  e não calibrado**; máscara de seleção atual não calcula isso;
- medidas locais e interpretação de cada peça crítica, com limiares definidos
  por tipo/escala e sem esconder divergências através da pontuação global;
- aparência avaliada depois, com condições de captura e limites explícitos;
- procedência, edição nativa e coerência espacial verificadas separadamente.

Calibrar em referências variadas e defeitos deliberados, incluindo os casos
acima e erros de câmera/contagem/conectividade. Comparar decisões do avaliador
com julgamento humano, medir falsos aceites/rejeições e só então fixar cortes.
Uma silhueta98% sobreposta ainda pode esconder uma porta errada; todos os
requisitos continuam necessários. Não prometer independência da capacidade de
interpretação de cada IA, nem fidelidade das partes ocultas.

## Próxima unidade recomendada

Implementar primeiro **observações geométricas versionadas + medição de
projeção + gate de estrutura**, com caso pequeno e depois E02. Critério de êxito:
um modelo estruturalmente errado recebe razões objetivas de reprovação e não
avança como aceito; ajustes de parâmetros reduzem erro medido sob a mesma
política. Não usar uma terceira casa completa como teste inicial dessa camada.
Explicar e alinhar os tradeoffs do contrato antes de alterar Scene/MCP/render.

## Revisão estática e validação desta unidade

Antes:1.execução de auditoria sob comando, fora de tick/frame;2–3.cópias Pillow
limitadas a três fixtures1402×1122, JSON pequeno;4.reusar cliente MCP existente,
sem duplicar comparador;5.três casos e caminhos estáveis, sem histórico de
builds;6.somente CPU/I/O de PNG, nenhum render/readback/download de modelo;
7.processo próprio sequencial;8.sem mutação de cena/replay/cache;9.checks de
hash, fixture e MAE independente, AST e diff;10.SSIM nativo não valida semântica.

Depois: execução real no build9860a09115365ad3 por MCP; três comparações nativas,
checks do relatório aprovados, fonte intacta, cena/export e render anteriores
preservados. Sintaxe Python/diff conferidos. Nenhum Rust, schema, renderer,
build ou skill alterado; regressões Rust não repetidas. Custos de CPU/I/O e
dependência Pillow no cliente permanecem explícitos. Sem nova prova de UI,
fidelidade, fechamento físico ou desempenho. Artefatos estáveis em
`artifacts/e02-fidelity-audit/`; relatório resumido versionado acima.

Reprodução, com Python que já contenha Pillow:

```sh
python3 scripts/audit_e02_similarity.py --binary /Volumes/Zero/dev/games/.ignis-build/image-to-3d/release/ignis
```
