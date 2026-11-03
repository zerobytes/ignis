# Circulação aérea local — EAF-159

## Decisão anterior ao código

Selecionar comandos dentro do executor de voo, antes de avançar o estado físico.
Cada alternativa curva usa o mesmo integrador de aceleração, frenagem, suavização
e taxa de curva. Um horizonte local finito compara trajetórias contra corpos
anônimos do snapshot anterior às decisões; o motor só compromete o primeiro
passo escolhido. A cada tick o destino continua sendo o endpoint do trecho.
Isso troca a previsão integral imutável por controle de horizonte recuado opt-in.
Sem circulação, a trajetória anterior permanece. ORCA global ou forças depois do
controlador introduziriam outra autoridade e não respeitariam esses contratos.

Voo reto admite mudanças XYZ instantâneas de direção como seu motor já admite;
voo curvado respeita o perfil autorado. Hover pode ceder; sem hover conserva
velocidade mínima e órbita, sem inventar parada. Salto e planeio só podem adiar
lançamento diante de conflito previsto; no ar mantêm balística/descida e colisão.
Nenhum modo ganha capacidade, aceleração ou informação sobre alvos.

## Revisão estática prévia

1. Reusar BVH e snapshot físico único; nenhum mapa/Scene clone por NPC.
2. Até128 folhas/16 vizinhos, alternativas e previews em arrays/valores no stack.
3. Perfil mental comum quantizado; autoria finita e nenhuma consulta JSON no tick.
4. Selecionar o próximo estado do mesmo integrador, sem aplicar desvio posterior.
5. Horizonte até120 passos por alternativa; inteligência limita esforço e horizonte.
6. Narrowphase Box/Wedge por volume; GPU/I/O/render não participam da decisão.
7. Sem worker/lock/PRNG novo; vizinho nunca lê decisão do ator anterior.
8. Estado no indivíduo existente; diagnóstico e voo já entram no checkpoint/replay.
9. Prova nova somente para risco de integração física/identidade/persistência;
   reusar gates de voo, patrulha XYZ, híbrido, circulação e checkpoint.
10. Custo preview×vizinhos e consultas estáticas é explícito; sem FPS ou arbitragem
    universal. Não adicionar índices globais eternos nem implementar streaming aqui.

## Contrato implementado

Scene98/replay108, coordenados com EAF-160; protocolo3/conteúdo1/checkpoint1.
Autoria continua em `npc.circulation` pelo mesmo upsert/edit_scene/templates.
`inspect_npc_circulation` acrescenta `state.air`: fase `flight`, `holding` ou
`launch`, quantidade de passos previstos, rejeições estáticas e `fallback`.
Ausência de configuração preserva a execução anterior. O checkpoint continua
exigindo a versão corrente de Scene: checkpoint1 não significa migração de97→98.

O sensor aéreo inclui NPCs, jogadores e objetos dinâmicos. Retém até16 corpos
entre128 folhas candidatas da BVH, com oclusão fixa,
posições/velocidades congeladas antes das decisões e desempate estável. Não lê
alvos, facções, memória nem a decisão do vizinho. Posição/velocidade do sensor são
locais/transitórias; nenhum índice de entidade é persistido como identidade nova.

Há5/7/9 direções conforme inteligência, mais meia velocidade e cedência pelo
próprio motor. As alternativas incluem subida/descida e contorno lateral; a mão
de contorno é consistente no referencial de movimento, incluindo oposição
vertical. O horizonte mental é35%..100% do autorado, até120 passos físicos.
Cautela/medo aumentam espaço e penalidade; coragem/agressividade reduzem cedência;
disciplina favorece consistência. Nenhum traço aumenta capacidade/velocidade.

Cada previsão integra um comando e faz sweeps Box/Wedge contra geometria fixa,
mais interseção contínua de caixas em movimento relativo contra os vizinhos.
O score pondera progresso real e risco; só o primeiro estado do motor escolhido
é publicado. A alternativa original segura encerra a busca cedo. As previsões
assumem velocidade constante dos vizinhos; cooperação é local/recíproca, sem
comunicação de identidade ou arbitragem global. Guias longas de voo continuam
mostrando o plano de lançamento; `requested/chosen` e corpo físico descrevem o
comando local atual, não uma rota futura garantida.

Voo curvado conserva aceleração, frenagem, suavização, taxa de curva, chegada e
capacidade de hover. Sem hover, a opção de cedência respeita a velocidade mínima
do motor e a chegada inicia órbita; a circulação também pode orientar a órbita
antes de seu próximo passo, com os mesmos limites. Voo reto continua o motor
com mudança instantânea de direção autorada, com velocidade limitada.

Jump/glide em andamento publicam `committed_air` e não recebem correção lateral
posterior. Se ainda apoiados, podem adiar lançamento por conflito no horizonte
(`launch_deferred`/`air_launch_deferred`), conservando a cadência existente de
retentativa do planejador (60ticks). A admissão consulta apoio físico mesmo antes
de existir contato em cache no KCC. Planeio já no ar mantém descida, sem
transformar espera em sustentação. Movimento zero exigido por fila/cast/ausência
de destino não inicia manobra de fuga. Investigação mede a distância física final.

`fallback=true` significa risco residual nas alternativas admitidas ou nenhuma
previsão estática completa admissível. Rapier e as interrupções físicas existentes
permanecem autoridade final: sensor truncado, mudança de intenção de terceiro ou
encurralamento não têm garantia de separação. Voos desviados têm prazo limitado
de1800ticks; não renovam prazo indefinidamente a cada desvio. Sem formação,
arbitragem universal de gargalos, streaming ou FPS garantidos.

## Demonstração e validação

`python3 scripts/run_verifications.py --only npc_air_circulation` executa cruzamento
reto, oposição XYZ, cruzamento curvado entre paredes e convergência. Salva a cena
`examples/npc-air-circulation.scene.json` pelo MCP. `P` executa a demonstração
finita, `R` reinicia; os corpos mantêm seus destinos após se cruzarem. Capturas
e relatório ficam em `artifacts/mcp-validation/artifacts/npc-air-*`.

A prova crítica Rust protege integração física, velocidade/aceleração/curva,
privacidade, identidade e checkpoint/replay com baseline. Os cenários existentes
de voo, híbrido, patrulha XYZ, circulação terrestre e checkpoint são os gates
de regressão por impacto. Captura offscreen e pixels exatos não são aceite visual
humano, janela, áudio físico, rede WAN ou prova de escala sustentada.

## Revisão estática após implementação

1. `Air::advance` integra candidatos antes de escolher; a fase terrestre em
   `physics.rs` conserva o resultado aéreo em vez de aplicar outro desvio.
2. Vizinhos16/comandos11/previsão120 usam arrays e cópias de estados pequenos;
   nenhum Vec novo, clone de mundo ou JSON na decisão.
3. A mesma matemática `advance_guided` serve ao comando original e aos desvios;
   autoria finita e limites do perfil continuam no motor, não no score.
4. Sensor `sense` compartilhado com o terrestre e snapshot reutilizado;
   alternativa original segura encerra busca. Sem PRNG por tick.
5. Tetos de folhas/vizinhos/horizonte/opções permanecem explícitos na inspeção.
   Não se ampliam pools/atores/grafo para mascarar congestionamento.
6. BVH e kernel Box/Wedge existentes fazem narrowphase; nenhuma GPU, rede ou
   escrita de arquivo no tick. Capturas são comandos de validação separados.
7. Sem thread/lock/dependência nova nesta frente; a ordem do loop não fornece
   velocidade futura do vizinho. Módulos de áudio pertencem à EAF-160.
8. Estado de motor/diagnóstico vive nos mapas existentes do indivíduo e usa o
   checkpoint nativo atual; nenhuma identidade nova por índice global de Scene.
9. Uma prova crítica integrada nova e cobertura existente por impacto; persistência
   compara continuação física/voo/NPC e replay pós-baseline, além de pixels MCP.
10. Custos mantidos: preview por candidato×passo×vizinhos e consulta estática por
    passo, plano integral de lançamento anterior, cópias off-tick no checkpoint.
    Não se alega zero alocação no Rapier, FPS, previsão universal ou mundo infinito.


## Evidência final

Build `b5d67fa265b6db6b`, macOS arm64. **27 testes Rust em6alvos**, **14 cenários MCP /
61grupos**, check e Clippy release all-targets `-D warnings`, fmt/Python/diff
aprovados. Apenas uma prova Rust nova, ampliada pelos riscos encontrados.
Ela inclui oposição horizontal/vertical, perfis reto/curvado/semhover, caixa
física dinâmica anônima, limites de movimento, identidade/retirada/reuso,
checkpoint/continuação/baseline/replay e espera de salto apoiado antes de haver
contato em cache. A espera conserva os60ticks existentes de retry, sem replanejar
por frame; retirada do bloqueador permite o lançamento seguinte.

No MCP os quatro casos chegaram aos destinos. Caixas de0,6m mantiveram menores
separações por eixo de0,856/0,843/0,829/0,938m (reto/corredor/vertical/convergência).
São evidências desses cenários, não distâncias garantidas em qualquer mundo.
Checkpoint e prefixo0 produziram PNGs idênticos. Cobertura existente adicional:
lab/NPC/percepção/render/busca/mobilidade/voo/patrulhaXYZ/terrestre/híbrido/
checkpoint/rede/controles remotos. Relatório corrente:
`artifacts/mcp-validation/report.json`; captura do cruzamento:
`artifacts/mcp-validation/artifacts/npc-air-crossing.png`.

Executável corrente único `target/release/ignis`; nenhum instalador ou cópia
histórica adicional. Fontes/hunks de áudio, coordenadas globais, save portátil e
apresentação paralelos foram preservados. Não houve janela/foco, instalação,
aceite humano, publicação ou alegação de FPS/WAN/streaming/save portátil.
