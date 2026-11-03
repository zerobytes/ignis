# Preservação canônica de meshes

## Revisão estática antes do incremento

Kernel puro e síncrono, somente em regeneração de autoria; nenhum trabalho por tick,
GPU, arquivo, thread, lock ou dependência nova. Entradas emprestadas e imutáveis;
saída nova publicada pelo chamador apenas após sucesso. Fonte canônica nunca é
substituída por um bake intermediário.

Limites existentes de meshes permanecem. Validação antes de alocar saída; um passe
por vértice/triângulo e detalhe, até32 ossos e4096 chaves. Pesos usam scratch fixo
de32 influências por amostra, sem busca espacial. Matrizes globais do repouso são
compiladas uma vez por avaliação. Clones de metadados/rig/clipes e buffers de saída
ocorrem por edição, explicitamente aceitos. Sem clone de Scene/Simulation.

Correspondência explícita por triângulo e coordenadas baricêntricas; comparar posição
material reconstruída, não contagem de vértices. Pesos com mais de4 influências
positivas recusados, sem redução implícita. Geometria e rig/clipes de saída passam
pela validação pública existente. Testar escala não uniforme, retesselação, volta à
fonte, repouso/animacão, índices/bary/NaN/limites e falha tardia sem mutação.

Tradeoff: transferência linear por triângulo representa o detalhe amostrado, não a
intenção dos pincéis nem uma superfície contínua desconhecida. Regiões semânticas
são responsabilidade do produtor das correspondências. Não alegar preservação de
contatos, volume, ausência de interseção ou ganho de performance.

## Contrato e domínio

`mesh_preservation::evaluate(&Evaluation) -> Result<Evaluated>` é independente de
plantas, edição, persistência e runtime. O chamador passa `source: &meshes::Asset`,
`reference_base` e `target_base` como `&meshes::Geometry`, dimensões positivas de
referência e destino, um slice de `Correspondence` e duas políticas independentes
para detalhe e translação animada. A saída contém `asset` e `report`.

A base está centrada na origem. Sua coordenada material é `posição/dimensões`,
contida em[-0.5,0.5] nos três eixos. A fonte pode ter detalhe fora desse volume;
ossos também podem estar fora. Origem, eixos e unidades devem coincidir entre
as entradas; transformações de instância não pertencem ao kernel.

`source.geometry` contém a superfície detalhada canônica e precisa compartilhar
exatamente índices e conectividade com `reference_base`. A saída tem os índices da
nova `target_base`. Cada amostra de destino recebe `Correspondence { triangle,
barycentric }`: triângulo da base de referência e três pesos em[0,1], soma1 com
tolerância1e-5. A faixa também tolera1e-5 para arredondamento em arestas; os pesos
são limitados a[0,1] e normalizados antes de usar. O resíduo material é verificado
depois desse ajuste.

O kernel verifica a posição reconstruída em coordenadas materiais, com erro
euclidiano máximo1e-5; isso não verifica a identidade semântica da região. O gerador
deve fornecer mapas da mesma face/parte e invalidar dependências removidas. Dois
pontos coincidentes em faces distintas não são semanticamente intercambiáveis.
Não há heurística de proximidade ou inferência a partir da contagem de vértices.

## Detalhe e retesselação

O vetor canônico por vértice é `source.position - reference_base.position`. Cada
nova amostra interpola esse vetor no triângulo indicado e soma à base de destino.
`detail_policy` escolhe:

- `physical`: conserva o vetor interpolado em unidades XYZ locais.
- `proportional`: multiplica suas componentes por `target_dimensions/reference_dimensions`.

Essa política controla amplitude e direção do vetor amostrado; a largura da região
segue as coordenadas materiais em ambos os modos. Não equivale a manter toda uma
cicatriz em centímetros, nem a transportar uma normal pela inversa transposta.
Campos normais, cages, reprojeção e receitas de pincéis requerem contratos próprios.

Refinar a tesselação reproduz o campo linear canônico; reduzir amostras pode apagar
um detalhe entre elas. O relatório mede o deslocamento nas amostras e o resíduo do
mapa, não um limite global de erro de superfície. A tesselação é escolha explícita
do chamador. Triângulos degenerados ou invertidos em relação à base de destino são
recusados; auto-interseções globais não são verificadas.

## Rig, pesos, bind e clipes

Hierarquia, nomes e ordem dos ossos permanecem. Compilam-se posições e rotações
globais de repouso uma vez; posições são multiplicadas pela escala dimensional,
rotações globais são mantidas. Reconstruir translações locais na orientação do pai
preserva as novas origens globais sem introduzir escala/shear em ossos rígidos.
Isso não realinha automaticamente eixos do osso com a direção de seus filhos.

Pesos são interpolados das influências canônicas no mesmo mapa. A soma de cada
vértice de origem é normalizada e influências do mesmo osso são somadas. A saída é
ordenada por índice do osso, não por peso. Mais de4 influências positivas é erro;
não há poda implícita nem mudança da identidade dos ossos. O runtime recompõe o
bind a partir do novo rig. A pose na chave de repouso deve reproduzir a nova
geometria; isso é verificado com o avaliador LBS existente nos testes.

Clipes existentes guardam transformações locais absolutas. Para cada chave:

```
D = inverse(R_old) * P_old
P_new = R_new * D
```

`R` é o repouso local rígido e `P` a chave local absoluta, com convenção de vetores
coluna. Rotações relativas são conservadas. `animation_translation_policy` physical
mantém a translação de `D`; proportional a transporta para os eixos XYZ do objeto,
aplica a escala dimensional e retorna ao referencial de repouso do osso. Tempos,
duração, IDs, tracks e política de interpolação permanecem. O offset de repouso
constante e a adaptação linear também preservam essa regra entre chaves.

Não se garantem contatos, trajetória mundial, volume, alcance de IK ou ausência
de interpenetração. Translações derivadas acima de10000, pesos inválidos, clipes
inválidos ou outras violações do validador público de meshes recusam a operação.

## Publicação, fonte e integração

Entradas são emprestadas e nunca mutadas. Não existe atualização parcial nem
fallback silencioso: o chamador conserva a última revisão publicada ao receber
erro. Regenerações sucessivas recebem a mesma fonte canônica, e não o bake anterior.
Uma edição manual explícita pode estabelecer outra fonte, por decisão da camada
de autoria; este módulo não decide rebasing, freeze, detach, revisão ou migração.

O `Report` serializável contém `source_vertices`, `target_vertices`,
`max_mapping_error`, `max_detail_displacement` e `adapted_keyframes`. Não declara
sucesso semântico universal. Chamadores devem preservar referência de região e
políticas junto da fonte e verificar os limites agregados do documento completo.

Neste commit o módulo é compilado pelo teste de integração via `#[path]`, sem
editar `lib.rs`. A frente de plantas exportará o módulo e integrará dispatcher,
persistência e replay. Os testes deste kernel não constituem validação desse fluxo.

## Revisão estática do código

| Pergunta | Resposta e evidência |
|---|---|
| Frequência | Adequado: `evaluate` é explícito, sem integração ao tick/frame. |
| Alocações/cópias | Mantido com motivo: fonte validada por Document temporário com Asset/Arc compartilhados; posições/triângulos/pesos/clipes de saída próprios. Sem copiar payload fonte para apenas validá-lo. |
| Tipos/acesso | Adequado: mapa u32 e bary f32 serializáveis; DVec3/DQuat locais na transferência para reduzir cancelamento e erro na reconstrução hierárquica, resultado f32 compatível com runtime. Sem mudança global de precisão/unsafe. |
| Repetição | Mantido com motivo: um passe valida mapas antes de alocar saída; outro reavalia bary sem buffers intermediários. Repouso global calculado uma vez; scratch de pesos fixo32. |
| Escala | Adequado: limites de Geometry/Document existentes antes de saída; até4096 amostras×32 influências e4096 chaves, sem busca espacial, ordenação ou crescimento histórico. Chamador valida budgets agregados. |
| GPU/I/O | Não se aplica: nenhuma chamada de GPU, captura, arquivo ou rede no módulo. |
| Sincronização | Não se aplica: empréstimos imutáveis e retorno próprio; sem threads/locks novos. |
| Invalidação | Adequado: sem cache; fonte é parâmetro explícito. Apenas chamador publica resultado completo. Volta à referência recompõe sempre da fonte. |
| Contrato | Validado no kernel: sucesso não uniforme e retesselação, rig rotacionado/repouso LBS/clipes relativos, pesos e erros atômicos. Integração MCP/replay/persistência pertence à frente de plantas. |
| Custos mantidos | Transferência e validação síncronas, clones por regeneração e comparações exatas de topologia. Um mapa por amostra ocupa autoria; não pressupor escala ilimitada ou ganho de FPS. |

## Validação do incremento

15 testes específicos em `tests/mesh_preservation.rs` aprovados com
`cargo test --locked --release --test mesh_preservation`; Clippy de todos os targets
release com `-D warnings` aprovado. Cobrem relevo vetorial localizado, duas políticas,
mais amostras, retorno canônico sem drift, hierarquia rotacionada, pesos interpolados,
repouso via runtime existente, movimento animado, chaves relativas, influência
excedente sem poda, índices ausentes, mapa espacial errado, topologia divergente,
NaN/faixas/limites, dobra local e falha tardia preservando a fonte. Capturas e aceite
visual não fazem parte desses testes puros. A regressão Rust completa usa
`cargo test --locked --release --all-targets`, com log corrente em
`artifacts/mesh-preservation-tests.log`; sua conclusão e a validação integrada
serão registradas nas respectivas tarefas de entrega.

## Domínio explícito para lofts de seções — EAF-81

`evaluate_material` recebe o mesmo Evaluation e um `MaterialDomain` com
coordenadas fonte/destino por vértice e origens globais de bind já mapeadas pelo
gerador. Confere cardinalidade, finitude, limites e reconstrução baricêntrica
antes da transferência. `evaluate` conserva a regra dimensional anterior.
O gerador valida a identidade das superfícies/estações; o kernel não deduz
semântica por distância. Origens mapeadas alteram translações locais, mantendo
orientações globais e a adaptação TR dos clipes. Os mesmos gates de dobra local,
influências e validação do asset continuam ativos.
