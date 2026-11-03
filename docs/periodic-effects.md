# Efeitos periódicos (EAF-45)

Uma habilidade pode instalar uma agenda de efeitos no destinatário. A preparação termina normalmente; os efeitos continuam sem manter o emissor ocupado ou imóvel. Jogador e NPC usam o mesmo executor, entrega direta ou projétil e controles MCP.

## Contrato e escolhas

Escolhemos uma agenda opcional por definição, reutilizando sua lista transacional de efeitos, em vez de criar um scheduler de callbacks ou um catálogo independente de buffs nesta etapa. Isso reduz contratos e permite veneno, regeneração e alterações periódicas de recursos. Em troca, uma habilidade usa a mesma lista em todos os pulsos: não mistura automaticamente dano inicial e periódico, não altera atributos e não oferece canalização. Essas extensões exigem contratos próprios. A extensão EAF-46 acrescenta [dissipação por categoria](dispel.md) para agendas opt-in.

No catálogo de habilidades, por `set_ability_catalog` ou `edit_scene`:

```json
"periodic": {
  "interval_ticks": 60,
  "pulses": 5,
  "reapply": "refresh"
}
```

Ausência mantém execução imediata anterior. Formato de cena/replay33; cenas antigas sem a agenda são preservadas, replay exige o mesmo build. Intervalo e quantidade são inteiros1..3600, produto máximo216000ticks (1h a60Hz). Os valores são de simulação: pause congela a agenda e step/replay avançam deterministicamente. A primeira aplicação acontece **um intervalo após a entrega**, a última após `interval_ticks × pulses`. Não há pulso imediato nem residual parcial.

A lista permite damage/heal/resource com recipient target. Self_actor é o próprio destinatário; ruídos e efeitos source são rejeitados para evitar operações ambíguas após morte do emissor. Cada pulso valida e aplica toda a lista atomicamente; mantém clamps, inteiros/escala e morte somente ao fim da transação. Falha interrompe a instância e registra motivo, sem aplicação parcial ou reembolso. Cura não ressuscita.

Custo e cooldown ocorrem uma vez: ao instalar diretamente ou ao lançar projétil. Aplicação direta valida todos os componentes e capacidade antes de debitar; projétil não reserva vaga e instala no primeiro impacto elegível, na vítima física real. Se não houver vaga ou componente no impacto, o disparo termina com effect_failed e motivo específico no executor, sem reembolso. Um impacto bem-sucedido instala a agenda, não antecipa o primeiro pulso.

## Reaplicação e ciclo de vida

- `refresh`: mesma origem + definição da habilidade + destinatário substituem a execução anterior, reiniciando intervalo e quantidade. Mantém a posição na ordem de processamento. Outra origem ou outra definição não é combinada. Renovar antes do primeiro pulso pode adiá-lo continuamente; é comportamento configurado, não dano imediato oculto.
- `independent`: cada aplicação cria uma instância independente com seus próprios prazos. Não soma magnitudes numa instância nem modifica a agenda anterior.
- Máximo64 instâncias globais; refresh pode substituir mesmo no limite. Demais aplicações falham explicitamente. Até256 recibos de ciclo de vida e256 recibos de habilidade em anéis distintos; descarte contabilizado. A inspeção não é histórico ilimitado.
- Depois da instalação, distância, visão, condição da habilidade, relação e vida/ativação do emissor não desfazem o efeito. Relação é verificada na entrega. Pulsos leem somente componentes do destinatário, não sua localização para orientar NPCs. A inspeção de desenvolvedor não é conhecimento do agente de gameplay.
- Morte do destinatário encerra todas as instâncias, sem pulsos futuros. Morte da origem preserva agendas já aplicadas e atribuição de dano. Cancelar cast não dissipa um efeito já entregue. Reset, edição, remoção, load e undo/redo reconstroem a simulação e limpam agendas como os demais estados de gameplay; save_scene salva autoria, não partida em andamento.

## Ordem e observabilidade

Percepção/intenções usam snapshot prévio. Após cancelamentos explícitos, processam-se agendas já instaladas na ordem estável de aplicação; cada uma executa no máximo um pulso no tick. Seguem projéteis, casts vencidos e ordens novas. Assim, pulso letal pode cancelar um cast que venceria no mesmo tick. Refresh no tick exato do vencimento ocorre depois do pulso já devido. Uma cura posterior não reverte morte confirmada por outra execução. Agendas de destinatários mortos são removidas no processamento e na limpeza final do tick; vagas de morte ocorrida depois da fase de pulsos ficam disponíveis após essa limpeza.

`inspect_abilities.periodic` expõe instâncias, origem/destino, habilidade, execução, aplicação, próximo/último tick, pulsos restantes, intervalo e modo; recibos applied/refreshed/finished/target_dead/effect_failed e replaced_by. Recibos do executor `periodic_pulse` contêm efeitos concretos e nenhum novo custo. Completed/impacted com effects vazio significam que a entrega instalou uma agenda; consultar a seção periodic para os resultados posteriores. Morte atribui origem e execução do pulso.

`J` / `set_debug_view(npc_routes=true)` exibe uma pequena faixa lilás acima do corpo enquanto há alguma agenda ativa, além das barras de vida/recursos existentes. É indicador de duração, não ícone de veneno/cura; não participa da física ou da decisão. Até uma faixa por ator mesmo com várias instâncias. Prefixos de replay reconstroem agenda e pixels; não reproduzem recibos como inputs novos.

## Demonstração

```sh
./target/release/ignis --scene examples/npc-periodic.scene.json --window
```

P inicia/pausa, WASD move, J mostra guias/barras, X cancela preparação e R reinicia. Clique seleciona o alvo.

- Roxo: alvo passivo com100 de vida. Jogador **1** aplica veneno direto: cast0,5s, dano5 por segundo,6 pulsos, refresh. Reaplique para renovar e observe a vida cair em etapas.
- Jogador **2**: regeneração própria, cast0,5s, cura5 por segundo,6 pulsos, independent. Repetir acumula agendas; cura limita na vida máxima. Começa com60/100 para comparar.
- Jogador **3**: projétil de veneno, cast0,5s, velocidade6m/s,5 de dano por segundo,6 pulsos. Instala somente quando o cubo atinge um ator; parede bloqueia.
- Vermelho: atira veneno no jogador observado, cast1s, intervalo entre lançamentos8s, projétil6m/s,4 de dano por segundo,4 pulsos. Esconder-se antes do lançamento cancela; esconder-se após ser atingido não cura o veneno.
- Todas as habilidades custam5 de energia por execução, nunca por pulso. Jogador e atirador começam com1000. Habilidades do jogador têm cooldown0,5s; vermelho tem8s.

Validação: `python3 scripts/run_verifications.py --only periodic`, `tests/periodic.rs`, testes Rust release e regressão completa sem janela. Captura offscreen/MCP não prova teclado/foco ou aceite humano. Sem profiling/FPS ou afirmação de sistema completo de buffs.

Desde cena35, `dispellable:false` protege todas as instâncias da definição contra dissipação sem alterar pulsos, refresh, expiração ou limpeza por morte. Categoria independente; padrão true preserva legado. Ver [dispel.md](dispel.md).
