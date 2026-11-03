# Composição ponderada de população — EAF-108

Cena65/replay69. Um gerador escolhe `template:"guard"` ou `templates:[{template:"guard",weight:3},{template:"elite",weight:1}]`, exatamente uma fonte. Template único permanece compatível; no modo ponderado pode omiti-lo ou usar string vazia. Lista vazia equivale a ausência, não a população proibida.

Até16 candidatos únicos conhecidos, peso inteiro1..1000000, soma no máximo16000000. Zero, duplicado, referência desconhecida, fonte ausente/dupla ou formato antigo com lista ponderada falha antes de mutar. Não há tipos/classes de NPC embutidos: cada item referencia um template autoral existente.

## Herança e significado dos pesos

Ondas podem omitir template/templates para herdar a fonte base inteira, escolher um template único ou fornecer sua própria lista ponderada. Sobrescrever não mistura listas nem soma pesos da base. A base continua obrigatória, inclusive quando todas as ondas possuem override. Referências de todos os candidatos são validadas e compiladas na construção, inclusive ondas futuras; remover template em uso falha. É possível atualizar dependências conjuntamente em edit_scene.

Pesos são proporções relativas por novo nascimento.3:1 permite qualquer composição em quatro nascimentos, inclusive quatro do mesmo tipo; não é quota, ordem de formação nem garantia estatística de uma amostra pequena. A quota de onda e max_births continuam contagens de indivíduos bem-sucedidos independentemente do tipo. Composição exata por tipo é contrato futuro.

## Uma escolha por nascimento pendente

Ao começar uma tentativa elegível sem nascimento pendente, selecionar template e seu alvo compilado; guardar no estado do gerador. Todos os candidatos de posição e ciclos de retentativa usam essa escolha até um nascimento bem-sucedido. Falhas occupied/unsupported_position/capacity/instance_unavailable não ressorteiam template nem consomem quota/identidade. Sucesso libera a próxima escolha, inclusive dentro do mesmo ciclo se ainda houver orçamento de tentativas/meta/onda.

Pausa manual, condição espacial fechada ou cadência não fazem draws. Falta de slot de acompanhamento impede começar uma nova escolha. Slot global cheio pode recusar uma escolha já feita, que fica pendente. Cancelamento impede executar qualquer nova tentativa; reset/edição/load/undo recompõem fontes, stream e pending. Não há skip/reroll manual implícito nem fallback para um candidato menor/sem alvo.

Escolha estável evita descartar sistematicamente corpos grandes quando um local não cabe. O custo é possível bloqueio: se o selecionado nunca puder nascer, a produção aguarda, com template e motivo visíveis. A solução autoral pode ser região/pontos/capacidade adequados ou mudança explícita da definição; não alterar pesos silenciosamente conforme a geometria.

Cada nascimento instala vida, atributos, recursos, capacidades e comportamento do template escolhido pelo mesmo spawn. Cada membro guarda índice de template junto da Reference/onda/life_index. Ressuscitar não sorteia novo indivíduo nem troca template. Alvo explícito do candidato continua vinculado ao indivíduo original da autoria; um substituto no mesmo slot não o assume. Se essa origem foi retirada, a escolha pode ficar bloqueada mesmo que outros candidatos fossem possíveis.

## Dois streams reproduzíveis

PRNG de posição existente permanece em rng, sem draws adicionais; pontos e region conservam suas regras. Template usa template_rng inicial=seed XOR0x9e3779b9, mesmo LCG u32 wrapping (1664525,1013904223). Uma escolha ponderada consome1draw. Bucket=(draw×soma_pesos)>>32 em u64; selecionar primeira soma cumulativa estritamente maior que bucket, na ordem autorada. Limites positivos garantem cobertura até u32::MAX sem overflow. Não há aleatoriedade criptográfica ou amostragem uniforme ideal garantida.

Template único não consome draws de template. Lista ponderada com um item consome1 por nascimento selecionado, mantendo contrato do modo explícito. template_draws conta draws, não sucessos. A seed do template não interfere nos valores gerados pelo stream de posição; templates com geometrias/capacidades diferentes podem alterar quais posições são aceitas e quantas tentativas posteriores serão necessárias. Mesma autoria, build, comandos e ticks reproduzem ambos.

## MCP, estado e replay

Autorar pelo set_npc_generators/SceneEdit existente; inspecionar por inspect_generators. Definitions mostra listas e pesos, controls mostra pending_template legível e template_draws; state guarda PRNG, flag, binding atual e fontes compiladas. template_ids mapeia os índices dos membros/recibos a IDs autorais. Antes de existir escolha pendente, template/target brutos são apenas o primeiro binding configurado/último binding usado; flag/pending_template dizem se existe reserva real.

Recibos de tentativa incluem template, posição, ponto/onda, resultado e InstanceId quando nasceu. Retirada automática atribui o template original do membro, mesmo que a onda atual já use outro. Anel256 e contador de descartes mantidos; escolhas não criam eventos externos no log de população. Replay69 regenera e compara fontes/PRNG/pending/membros/recibos; prefixos não antecipam draws. Cena65 salva apenas autoria; save de cena não serializa a sequência já sorteada como novos templates.

Fontes Single guardam Binding sem heap; Weighted usa tabela cumulativa limitada. Ondas herdadas referenciam a base sem clonar listas. Nada de Strings, Vec ou compilação no caminho de escolha/tentativa; consulta limitada a16 entradas por novo nascimento. Recibos/membros ganham índices pequenos, estado quente continua Copy. Sem renderer/RenderFrame/dependências novos, limites de NPCs ampliados ou ganho de FPS alegado.

## Demo

`examples/npc-weighted-population.scene.json`, autorada via MCP: P inicia, D entra na região verde, nascimentos em ciano. Primeira onda4 indivíduos com mistura ponderada roxo/laranja; segunda3 com pesos próprios; último1 azul de200vida fixo. Não há contagem fixa por cor nas duas primeiras.1 dano100/2 ressuscita no alvo selecionado, B/O/K permissão manual e R reset. Mesmos comandos e seed reproduzem escolhas e posições.

Validação específica tests/template_selection.rs e scripts/verify_template_selection.py, mais regressão completa. Regras físicas/sensoriais de habilidade continuam: erro occluded não comprova defeito do gerador. Testes MCP usam remoção explícita quando querem apenas avançar a população. Aceite de teclado/foco/visual nativo separado de headless/capturas. Evidências em validation.md e checklist em performance.md.
