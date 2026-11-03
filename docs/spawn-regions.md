# Regiões de nascimento procedural — EAF-105

Continuação EAF-111: [support_projection](spawn-support.md) opcional transforma os candidatos em origens da busca dos pés e calcula o centro pelo piso/template. Sem o opt-in, o contrato XYZ explícito abaixo permanece. O modo projetado limita origens, não a altura final.

Um gerador escolhe exatamente uma fonte de posição: `points` (lista atual) ou `region` (limites dos centros dos corpos). Scene64/replay68 acrescentam region; ausência preserva o sorteio de pontos anterior.

```json
{"id":"encounter","template":"guard","region":{"min":[-5,0.8,-1.5],"max":[5,0.8,2.5]},"seed":42,"target_alive":3,"interval_ticks":30,"attempts_per_cycle":8,"max_births":6}
```

`points` pode ser omitido ou vazio quando há region. Configurar os dois, ou nenhum, falha antes de mutar. Min/max precisam ser finitos e dentro de±10000, min≤max em cada eixo e pelo menos um eixo com extensão positiva. Eixo fixo permite plano horizontal ou linha; caixa toda degenerada deve usar points. A região é autoral e alinhada aos eixos do mundo; não referencia trigger e não possui corpo físico.

## Posição, corpo e altura

Os limites restringem o **centro** do NPC. Seu corpo pode se estender para fora da região; tamanho, espaço livre, apoio e capacidade continuam validados pelo spawn. Isso permite manter a mesma região com templates de tamanhos diferentes sem reinterpretar limites. Quem precisa de corpo inteiro dentro deve reduzir os limites de centros explicitamente.

Para terrestres, usar altura coerente com chão e metade da altura do template, por exemplo Y0.8 para corpo1.6. Não projetar no terreno ou ajustar altura silenciosamente. Uma caixa alta pode produzir recusas unsupported_position para um terrestre; um voador com hover permitido pode nascer em altura. Planar/saltar não concede hover. O recorte não descobre chão, navmesh, caminho, região navegável ou distribuição de terreno procedural.

O spawn existente aplica capacidade global, ocupação dos corpos presentes, apoio/hover, limites de passagem, alvo e identidade. A seleção espacial não concede capacidade ou conhecimento novo ao NPC. Geometria/tamanho/capacidade do template da onda atual prevalecem. Candidato recusado não vira nascimento nem consome quota/InstanceId.

## Semente, orçamento e ordem

LCG u32 existente: estado=estado×1664525+1013904223, wrapping. Em região, cada tentativa consome3draws em ordem X,Y,Z, inclusive eixos de extensão0. Fração=(estado>>8)/16777216 em f32; coordenada=min+(max−min)×fração. Coordenadas resultantes podem arredondar para max em f32. Garantir limites inclusivos; não alegar aleatoriedade criptográfica ou amostragem uniforme ideal em números reais.

Cada ciclo elegível tenta até attempts_per_cycle (1..8), interrompendo ao atingir meta viva, quota da onda, budget total ou capacidade de membros. Candidatos regionais podem se repetir; ocupação recusada consome PRNG/tentativa. Sem lista preparada de candidatos, retry até sucesso, distância mínima garantida ou ponto alternativo fora da região. Pausa/condição fechada/meta satisfeita não consomem draws regionais. Capacidade global cheia pode produzir recibos capacity; não consome nascimento, mas houve tentativa de posição.

Points preserva sua regra anterior:1draw por ciclo para escolher início e candidatos distintos circularmente até min(tentativas,points.len). Uma região não altera o significado/seed dos geradores de points. Mesma autoria, build, comandos e ticks reproduzem os mesmos candidatos/resultados. Seed diferente pode continuar produzindo resultados parecidos; geometria e limites podem dominar o resultado.

Ordem permanece: pedidos externos de população, geradores em ordem autoral, sensores/habilidades/física. Spawn/despawn externo pode liberar/ocupar espaço antes do ciclo. Não há reserva/equidade entre geradores. Ondas, permissões manual/espacial, retirada de cadáver e ressurreição preservam seus contratos; reposição sempre cria novo indivíduo.

## MCP, observação e persistência

`set_npc_generators` e a operação correspondente em `edit_scene` autoram region; `inspect_generators` retorna definição, PRNG/contadores e recibos. Cada tentativa tem position, tick, generator, wave e outcome, com point null para region e índice para points. Retirada de cadáver tem position null. Assim uma recusa occupied/unsupported_position/capacity não se confunde com ausência de tentativa. Anel256 e contagem de descartes permanecem.

Edição valida antes de reconstruir runtime e participa de undo/redo. Reset/load retomam semente e contadores autorados. Cena salva região/configuração, não os NPCs gerados. Replay68 regenera candidatos, guarda e compara estado/recibos inclusive posição; prefixos não antecipam sorteios futuros. Comandos pendentes permanecem na fila existente; nenhum comando externo novo por candidato.

## Custos e limites

Até8geradores×8tentativas/tick, pool8NPCs+jogador. Sorteio escalar/stack, sem Vec/String/Scene clone por candidato. Spawn/Rapier mantêm custos esparsos existentes; recibo ganha coordenadas limitadas a256entradas e perde Eq derivado devido a f32, mantendo comparação PartialEq determinística no replay. Renderer/RenderFrame e dependências inalterados. JSON permanece na fronteira; nenhuma promessa de FPS ou escala601.

Demo `examples/npc-spawn-regions.scene.json` criada/salva por MCP: região de nascimento marcada em ciano, obstáculo no centro e região verde de ativação à direita. P inicia; D entra no verde e libera ondas, A sai e suspende produção. B/O/K controlam a permissão manual;1 dano100,2 ressuscita cadáver presente, R reinicia a mesma seed. Repor ou avançar a onda sorteia novas posições; reset reproduz as anteriores. Marcadores não participam de colisão/apoio/ativação.

Validação: tests/spawn_regions.rs e cenário MCP spawn_regions, mais regressão completa. Testes de teclado/foco e aceite humano da demo são gates separados. Resultados em validation.md e checklist antes/depois em performance.md.
