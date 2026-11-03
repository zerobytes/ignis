# Ativação espacial de geradores — EAF-96

Cena63/replay67. Um gerador pode condicionar sua produção à ocupação de um trigger autorado, sem criar geometria de detecção nova nem fornecer evidências aos seus NPCs.

```json
{"activation":{"volume":"encounter_zone","mode":"while_occupied"}}
```

`activation` ausente/null preserva EAF-95. `volume` referencia uma entidade com trigger válido, habilitado ou não. `mode` default `once`: após a primeira amostra ocupada, a condição fica satisfeita até reset/edição. `while_occupied`: abre enquanto há pelo menos um alvo presente na ocupação observada; fecha ao esvaziar. Inicializar dentro do volume também satisfaz a condição, não é exigido cruzar sua borda a partir de fora.

A autoria usa `set_npc_generators`, inclusive `edit_scene`, e o trigger usa `upsert`/edição existentes. Dependência validada antes de mutar; não remover o volume ou seu componente trigger enquanto referenciado. Pode remover condição e volume juntos em uma edição atômica. Scene63 salva apenas a definição, não o latch/estado da execução. Reset/load/undo recompõem bindings, condição fechada e prazos originais.

## Duas permissões e um relógio suspenso

Produção exige permissão manual active **e** condição espacial aberta, além dos limites/cadência/quotas normais. `initially_active:true` permite que entrar na região inicie sozinho; false requer também `activate`/B. Activate não ignora a região; pause/O não é desfeito por entrar nela; cancel/K permanece terminal. A condição continua sendo observada durante a pausa manual; once pode ficar satisfeita nesse período, mas só produzirá após a permissão manual. NPCs já existentes seguem jogando; limpeza de referências e retirada opcional de cadáveres continuam em tempo de mundo.

A suspensão efetiva usa um único `paused_since`, agora significando produção suspensa por qualquer dessas razões. Começa no tick0 quando há condição, ou no tick em que a produção deixa de estar permitida. Quando ambas as permissões ficam abertas, desloca uma vez os prazos de próxima tentativa e atraso da onda. Não acumula duas pausas ao sair da região enquanto o gerador já está manualmente pausado. Pausar/ativar no mesmo tick conserva a semântica da fila externa: a permissão final é observada antes da produção.

Ondas não avançam com condição fechada. Se seus membros morrerem/saírem, a confirmação ocorre ao voltar a produzir; o atraso da próxima onda começa nessa confirmação. Once não volta a fechar quando os ocupantes saem. Nenhum modo reabre sequência concluída/orçamento esgotado nem remove sobreviventes para corrigir meta. A condição pode continuar sendo amostrada depois do término, sem reiniciar relógios/produção.

## Amostragem, geometria e identidade

Reusa a ocupação pós-física dos triggers, atualizada no fim do step. A fase de geradores do step seguinte a consome, depois dos comandos externos e da própria limpeza de membros. Exemplo: primeiro step observa um jogador inicialmente dentro; segundo step pode produzir. A inspeção logo após um cruzamento pode mostrar trigger ocupado e condição ainda fechada: são fases distintas, não atraso de render.

Trigger existente usa interseção estrita de AABBs XYZ do volume e corpo; contato exato na borda não conta, penetração positiva conta. É sobreposição discreta, não centro/ponto nem sweep: atravessar todo um volume entre duas amostras pode não ser detectado. Visibilidade do volume e suas marcações não muda ocupação. Sem requisito de linha de visão; isso é regra do mundo, nunca percepção do NPC.

Só os targets explícitos do trigger participam. Qualquer um presente basta. Corpos mortos também ocupam; ressurreição mantém a identidade. Filtrar referências contra o catálogo atual evita usar um ocupante já retirado na fila externa, mesmo antes de publicar o próximo snapshot de trigger. Reuso do slot/nascimento novo não adota o lugar do antigo target. Novos alvos por classe, consulta de facção, somente vivos, multiplicidade mínima e composição booleana são contratos futuros; não inferir seleção de todos os jogadores.

## Observabilidade e custo

`inspect_generators.controls`: status manual, terminal, production_enabled efetivo, condition_open e activation. Estado de activation contém índice compilado do volume, modo, occupied amostrado, open, sampled_tick (tick da avaliação do gerador), last_change_tick e changes. Definição contém o ID do volume. Inspect_triggers mostra ocupantes/eventos da última física. Comparar ticks para separar fonte e amostra.

`remaining_delay_ticks` e `remaining_cadence_ticks` usam o início da suspensão quando bloqueado. Deadlines brutos só são deslocados ao retomar; não são previsão enquanto suspenso. Status active com production_enabled:false é permissão manual habilitada esperando condição, distinto de pause manual. Título da demo mostra aguardando regiao; B/O/K continuam o mesmo dispatcher, sem controle exclusivo visual.

Índice compilado uma vez na mesma ordem lexicográfica dos triggers; amostra lê máscara limitada e valida referências presentes. Até8 geradores×32 targets, sem novo teste geométrico, Vec, strings, fila ou log por tick. Estado Copy com última amostra e contadores; replay regenera e compara, incluindo prefixos. Sem callbacks do mundo, motor genérico de regras, dependência nova, novos limites de NPCs ou FPS alegado.

## Demo

`examples/npc-generator-activation.scene.json`, criada/salva pelo MCP: P inicia; entre na região verde para liberar a sequência de ondas. Saia para suspender produção/avanço, mantendo os NPCs vivos. Retorne para continuar os prazos restantes; O força pausa manual mesmo dentro e B libera essa pausa, K cancela.1 causa100dano e2 ressuscita no alvo selecionado; R reinicia. Marcadores no piso representam a projeção do volume, mas o corpo pode sobrepor sua borda antes de seu centro entrar. O trigger é invisível/sem colisão, altura explícita.

`python3 scripts/run_verifications.py --only generator_activation` valida os dois modos, comandos manuais, geometria/fase, prazos, replay/capturas, autoria e demo por MCP. Testes Rust também cobrem morte/retirada/reuso/condição ligada a NPC, erro/undo/versão e término. Teclado/foco e aceite humano desta demo são separados da automação; evidências completas em validation.md.
