# Mira espacial automática — EAF-44

Cena/replay32. Objetivos `use_ability` podem usar habilidades `selection: spatial` com uma política explícita de origem da mira:

```json
{
  "kind": "use_ability",
  "slot": 1,
  "retry_ticks": 30,
  "policy": {
    "spatial_aim": { "mode": "observed_point", "range": 18 },
    "approach": null,
    "vitality_below": null
  }
}
```

A habilidade continua spatial/projectile/range0. `policy.spatial_aim.range` limita a distância XYZ entre emissor e candidato observado para decidir/iniciar/concluir; `delivery.max_distance` continua limitando a viagem física do projétil. São limites independentes. Alcance de decisão deve ser finito em0,1..100m; `spatial_aim` em habilidades actor/self é inválido. Habilidade spatial sem política explícita continua inválida num objetivo automático.

## Evidência, cast e lançamento

O seletor usa a observação confirmada do tick atual, inclusive reconhecimento gradual configurado. Som, rádio e memória visual antiga não fornecem identidade/localização atual para iniciar. A política onisciente autorada continua válida: sua proveniência é `omniscient`, distinguida de `vision` na inspeção.

No início do cast, o executor copia a posição observada para `Aim::Point` e registra ator/tick/fonte em `aim_observation`. A identidade nessa proveniência permite verificar se o candidato mudou; ela não é um destino de rastreamento. `target`/`intended_target` continuam null. Cada passo de percepção pode produzir uma posição nova, mas não altera o ponto já escolhido.

Enquanto prepara, o NPC precisa continuar observando o mesmo candidato, dentro do alcance e satisfazendo relação/condições. Perda de visão, troca, saída do alcance ou mudança de condição torna o objetivo inelegível e cancela. Retentativa usa o prazo existente; durante esse intervalo a decisão aparece como `unavailable`. Ao vencer o cast, o executor revalida visão atual/identidade, distância, vida/relação/limiar vital, fonte, recursos e capacidade antes de cobrar. Nenhuma consulta de posição escondida é feita a partir da identidade original.

O projétil nasce na posição atual do emissor e aponta para o ponto congelado. O personagem observado pode ter se deslocado enquanto ainda visível: o disparo vai para a posição anterior. Depois do lançamento não exige visão, não persegue a identidade e pode atingir outro corpo. Fonte morta ou cancelamento tardio não recolhem o disparo. Custos, cooldown, movimentos de cast, colisão e efeitos são os mesmos dos [projéteis](projectiles.md).

## Aproximação e separação de responsabilidades

`approach: {margin: 0.4}` permite aproximar usando o alcance da política espacial. A histerese e os motores existentes são reutilizados; distância contempla altura em XYZ. Sem approach, espera fora de alcance. Durante um cast que perde alcance, cancela antes de voltar a aproximar. Aproximação não garante caminho e não concede voo/salto nem supera limites territoriais.

A política é do objetivo, não da definição da habilidade: a mesma habilidade pode ser usada pelo jogador, por ordens MCP e por NPCs com políticas distintas. `aim_ability` externo continua uma ordem explícita, sem fabricar proveniência visual; o objetivo automático não cancela uma ordem externa por perder a visão.

Alternativas: recalcular posição até lançar equivaleria ao modo actor existente e reduziria a oportunidade de esquiva durante preparação; usar memória/som automaticamente exigiria semântica de idade/confiança/compromisso ainda não definida. Antecipação por velocidade exigiria histórico perceptivo e um modelo temporal próprio. Este recorte congela uma evidência confirmada e mantém os modos distinguíveis.

## Observabilidade e controles

Autoria por `upsert`/`edit_scene`, descoberta do schema no MCP, `inspect_objectives`/`inspect_npcs` para fase/candidato/política e `inspect_abilities` para ponto/proveniência/cast/voo. Estado e evidência original entram no replay. Edição/reset/undo/redo reconstroem a simulação como antes.

**J** e `set_debug_view {npc_routes:true}` mostram um pequeno marcador amarelo XYZ no ponto escolhido durante o cast automático. Ao mover o personagem observado, o marcador fica para trás; desaparece quando o cast termina/cancela. Usa três caixas de diagnóstico, sem corpo, e nunca participa de percepção/colisão. Pode ficar encoberto pelo personagem enquanto os dois ocupam o mesmo ponto.

## Demo comparativa

```sh
./target/release/ignis --scene examples/npc-aim.scene.json --window
```

- **P** inicia; **WASD** movimenta; **J** barras/marcador; **R** reinicia.
- **Vermelho** congela a posição vista ao começar cada cast. **Azul** usa o modo actor anterior: atualiza a mira até o lançamento. Ambos ficam no posto, com cast1,5s, dano10, velocidade6m/s, alcance físico18m, custo5 e cooldown3s.
- Assim que as barras de cast começarem, mova lateralmente e pare. Vermelho atira no ponto anterior; azul aponta para a posição atual ao lançar. Ambos podem errar se você desviar depois do lançamento. Ficar parado permite que ambos acertem.
- A parede verde é cobertura. Perder contato visual durante o preparo cancela; paredes/corpos bloqueiam projéteis já lançados. Um NPC pode enxergar enquanto o outro está bloqueado, dependendo da posição.
- Jogador: **1** tiro livre pelo cursor (centro do corpo/plano na altura do jogador), **2** cura própria25, **3** projétil por alvo selecionado por clique; **X** cancela.

O marcador é diagnóstico, não um sistema completo de aviso visual de ataques. Não há antecipação, precisão probabilística, mira por som/memória, fogo de supressão, escolha de cobertura, esquiva automática ou triagem nova. A API continua XYZ; teclado/foco e aceite humano são gates separados da validação MCP/offscreen.

Verificação: `python3 scripts/run_verifications.py --only npc_aim`, `tests/npc_aim.rs` e regressão headless completa, incluindo render/replay. Revisão de performance em [performance.md](performance.md).
