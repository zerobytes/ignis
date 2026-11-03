# Caixas chanfradas nativas

Rastreado em EAF-121. Cena72/replay76 mantidos nesta branch isolada.

## Fricção, alternativas e decisão antes do código

Blocos e lajes da casa usam caixas perfeitas calculadas no cliente. Aplicar
chanfro manual replica faces/índices; Sweep de perfil fixo não troca uma seção
octagonal por tampa quadrangular em três eixos. Bevel genérico de regiões exige
contrato maior de contornos/valência/rig, ainda aberto. Um construtor limitado
de caixas chanfradas resolve o caso com custo e geometria verificáveis.

Adotado `MeshEdit.chamfer_boxes{id,boxes,append:false}`. Cada caixa recebe center,
size, rotation quaternion unitário e bevel. Até512caixas. Gera Geometry comum,
sem persistir receita nova; substituição cria asset sem rig/clips, material
padrão/shading flat. Append conserva geometria/canais/aparência existentes e
recusa rig/clips. Planta vinculada exige detach. Sem kernel Sweep, renderer,
dependência, Vertex/Globals, versão de cena/replay ou física novos.

## Contrato

center finito±10000m, size por eixo0.0001..1000m. rotation default identidade,
norma² com desvio≤0.0001, normalizada para avaliação. bevel zero ou finito
0.00001..min(size)/4. Zero gera8v/12t; positivo24v/44t:6faces quadrangulares,
12faixas de aresta quadrangulares e8faces triangulares de canto. Chanfro plano,
uma faixa, sem arredondamento/subdivisão ou irregularidade procedural implícita.

Vértices/conectividade derivados em coordenadas locais f64. Faces por26planos
conhecidos, sem hull geral. Ordenação angular por face no referencial da normal;
indices u32/saída f32 passam Geometry.validate. Material coordinates são posições
no espaço local do asset após transformação, como a aparência prévia das caixas.
Peças distintas não são soldadas. Precisão f32 final pode recusar peças pequenas
distantes da origem; contagem aprovada não garante admissão dessa conversão.

Preflight por asset/documento/draw antes de expandir/clonar a fonte. Mesmos
preview_mesh_edit/edit_mesh/lotes, changed_assets e histórico. Não é bevel de
uma região de mesh existente ou preservação de escultura/skin após regenerar.

## Checklist estático anterior

1. Só autoria.2. Contagens8/24e12/44 antes dos buffers/clone; reserve uma vez.
3. f64 temporário/f32publicação/u32indices, sem unsafe/dependência.
4. Planos/vertices constantes por caixa, nenhuma busca na malha antiga por face.
5.512caixas e caps herdados, rejeitar excesso antes de expansão.
6. Renderer/cache iguais, captura explícita.7. Dispatcher/staging existentes,
sem thread/lock novo.8. changed_assets/guarda de planta/undo/append/erros no MCP.
9. Volume analítico, manifold/orientação/bounds, transformação/precisão/budgets,
persistência/replay e caso da casa.10. Mantidos: JSON e staging síncronos, sem
bevel geral, normais por região, materiais por face ou desempenho medido.

## Revisão estática do diff

1. `Plan::new/append_to` chamados somente pela autoria; nenhum tick/frame novo.
2. `counts` e `check_replacement_counts` antecedem clone e expansão. Três buffers
de Geometry reservam saída prevista; pontos/planos/índices de face usam vetores
temporários pequenos por caixa, até26faces. Esse heap limitado de autoria foi
mantido em favor de clareza; não é alegação de zero alocações.
3. Avaliação f64 e quaternion normalizado; antes de inserir cada face, normal
publicada em f32 precisa apontar para o semiespaço esperado. Geometry.validate
também rejeita área zero/índices inválidos. Sem unsafe/dependência nova.
4. Ordenação até4v por face em26planos/24pontos conhecidos, sem hull genérico,
busca na fonte ou solver iterativo. Atan2 repetido no sort é custo fixo mantido.
5.1..512caixas, parâmetros limitados, somas pequenas e caps herdados. Seis
instâncias de512caixas chanfradas excedem draw e recusam antes da expansão.
6. Asset/renderer/cache comuns, material coordinates iguais às posições
publicadas; append de canal ausente copia posições antigas para conservar look.
7. Mesmo staging sequencial, sem lock/worker novo.
8. ChamferBoxes participa de changed_assets e guardas de planta; novo braço de
publicação é deliberadamente independente do braço Sweep da frente paralela.
9. Testes de volume/bounds/orientação/manifold, transformação/precisão e MCP
exercitam o contrato; resultados finais ficam em validation.md.
10. Gerador descarta rig/clips na substituição, append os recusa. Sem bevel de
componentes/região, preservação de escultura, arredondamento, física ou FPS novo.
