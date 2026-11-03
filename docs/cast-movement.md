# Movimento durante preparação de habilidade (EAF-41)

Configuração por definição, compartilhada entre jogador e NPC, no executor existente. Guilherme autorizou os dois comportamentos restritivos; a ausência de configuração preserva movimento livre. O campo é `ability_catalog.abilities[].movement`, publicado pelo schema MCP e editável com `set_ability_catalog`/`edit_scene`, com undo/redo e validação antes de mutar.

| Configuração | Início e preparação |
|---|---|
| `{"mode":"free"}` (padrão) | Locomoção e cast independentes. |
| `{"mode":"interrupt","speed_tolerance":0.2}` | Entrada de movimento ou velocidade física acima da tolerância rejeita início (`moving`) ou interrompe execução (`interrupted_by_movement`). |
| `{"mode":"lock"}` | Uma execução aceita suspende locomoção voluntária durante a preparação. Concluir, cancelar, morrer ou falhar libera o bloqueio. |

`speed_tolerance` é obrigatória no modo interrupt, finita, de0,01 a2 m/s. Não é orçamento de distância: filtra pequenas correções de contato do solver. Entrada voluntária não usa essa tolerância; tentar andar contra uma parede também interrompe. Velocidade física considera XYZ, o maior valor entre velocidade do corpo e deslocamento resolvido dividido pelo passo fixo. Queda e empurrão podem interromper mesmo sem teclado. O campo ausente equivale a free e não é gravado; interrupt/lock exigem cena29. Replay29 exige o mesmo build/runtime, como anteriormente.

## Ordem e física

Entrada atual e velocidade anterior são verificadas antes dos commits. Cancelamento explícito ainda vem primeiro. Portanto, entrada de movimento no tick do vencimento cancela antes de cobrar ou aplicar efeito. Após emitir comandos dos motores, intenção de movimento de NPC também pode interromper uma preparação ativa; após o solver, movimento efetivo pode interrompê-la novamente. Cada execução gera somente um recibo de interrupção.

Os efeitos continuam precedendo os motores/física. Uma força ou decisão de motor surgida depois de um commit não desfaz aquele efeito retroativamente. Habilidade instantânea não tem intervalo de preparação para bloquear; ainda valida a política no início. Uma rejeição/interrupção não consome energia, não aplica efeitos e não inicia cooldown. Retry de objetivo automático continua separado de cooldown.

O bloqueio deriva dos casts ativos, não altera o teclado armazenado e não transforma o corpo em estático/cinemático. Segurar uma direção durante o cast retoma o movimento ao liberar. Gravidade e resolução de colisão continuam; lock não é imunidade a deslocamento. O controlador dinâmico atual continua sobrescrevendo velocidade horizontal comandada a cada tick; isto não implementa um sistema novo de knockback.

Patrulha fica suspensa sem gastar o prazo de viagem, mantendo o índice do ponto. Caminho de caminhada é invalidado, buffers são reutilizados, reservas de passagem são liberadas; sensores e decisões continuam ativos. Ao liberar, o objetivo vigente volta ao planejador. Um destino pode continuar inalcançável.

Trecho espacial jump/glide/fly em execução ou órbita em movimento recusa lock com `movement_lock_unavailable`: interromper esses motores arbitrariamente congelaria uma trajetória comprometida. Caminhada e hover estacionário podem bloquear; o hover segue a política existente do motor de voo. Não há aterrissagem ou frenagem aérea implícita. Use free para cast em voo; interrupt requer estar parado. Cast automático também pode precisar aguardar a velocidade residual baixar e o retry vencer antes de começar.

## Observabilidade e teste

`inspect_abilities` inclui a política nos grants, `movement_locked` e `motion` (`speed`, `requested`, `lock_supported`) por ator equipado, além de recibos, custos e prazos. `requested` mostra a entrada do jogador ou comando do motor de NPC amostrado; não representa o conhecimento do NPC. NPC também publica `locomotion_locked`. Reset/load/edição reconstroem o estado; prefixos de replay preservam entrada pressionada, bloqueio e fila pendente.

Demo autorada pelo MCP: `examples/cast-movement.scene.json`.

```sh
./target/release/ignis --scene examples/cast-movement.scene.json
```

Pressione P para rodar, J para barras e clique no roxo antes de usar1. Jogador laranja começa com40/100 de vida e1000 de energia; roxo é um alvo passivo com1000 de vida. Todos os casts duram90ticks (1,5s), custam5 de energia e têm60ticks (1s) de cooldown após sucesso.

1. **1 — ataque livre:** ande enquanto prepara; aplica15 de dano ao roxo se ele continuar no alcance12m e sem oclusão.
2. **2 — cura interrompível:** parado, cura15; andar antes do fim cancela. Tentar iniciar andando é rejeitado. Espere o corpo estabilizar após soltar a tecla.
3. **3 — cura com bloqueio:** pode iniciar andando; para a locomoção por1,5s, cura15 e volta a andar se a tecla continuar pressionada.

X cancela a execução atual; R reinicia vida/energia. Uma barra de cast pode sumir imediatamente quando interrompida; o resultado preciso fica no recibo MCP. Cura satura em100, então use R para repetir a comparação. O teclado/foreground é um gate humano separado: a regressão usa apenas MCP, física headless e captura GPU offscreen.

Verificação: `python3 scripts/run_verifications.py --only cast_movement`. Testes Rust cobrem também contato, queda, empurrão, deadline, morte, NPC/patrulha espacial/passagem e incompatibilidade aérea. Este recorte não adiciona canalização, projéteis, efeitos persistentes ou regras de animação.
