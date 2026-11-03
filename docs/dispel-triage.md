# Triagem por efeitos dissipáveis

EAF-56, cena/replay45. `use_ability.policy.require_dispel_match: true` é um filtro opt-in de necessidade: pelo menos uma instância no destinatário deve ser removível pela habilidade escolhida. Não deduz efeitos negativos, severidade, doenças ou classe; categorias são dados do jogo. O padrão false preserva comportamento legado, inclusive dissipação opcional vazia paga.

```json
{
  "kind": "use_ability", "slot": 1, "retry_ticks": 15,
  "policy": {
    "allies": { "ranking": "nearest", "include_self": true },
    "require_dispel_match": true,
    "approach": { "margin": 0.4 }
  }
}
```

Pode combinar com o limiar de vida (AND), ranking por fração vital ou proximidade. Sem limiar, vida cheia não impede apoio. `nearest` permite destinatário sem vitalidade; `lowest_health_ratio` continua exigindo vitalidade. Também serve a objetivo de alvo explícito ou self_actor sem triagem de aliados. Exige pelo menos um operador dispel no destinatário (source também resolve ao próprio corpo em self_actor). Entrega direta ou projétil por ator; área, agenda periódica, canalização e mira spatial são rejeitadas nessa política para evitar ambiguidade sobre qual destinatário precisa de tratamento.

## Conhecimento e decisão

Só consulta as instâncias depois da evidência visual confirmada corrente ou conhecimento próprio explícito. Preserva reconhecimento, FOV/XYZ/oclusão/território, alvo de combate, som anônimo e rádio. Como o contrato anterior de dissipação, visão autoriza consultar efeitos; não existe um sistema de doenças ocultas, diagnóstico por sintomas ou descoberta por som. O registro privilegiado de efeitos não entra na memória sensorial.

A consulta acontece depois dos filtros básicos de elegibilidade e antes do ranking. Compartilha o seletor transacional: periódicos primeiro, depois buffs, ordem estável dos pools, limite por operador e máscaras contra seleção duplicada. Proteção periódica usa o valor efetivo de `effect.dispellable` no autor original; buffs usam sua proteção própria. Categoria incompatível/protegida não conta. Buff cujo expires_at já chegou é excluído mesmo antes da limpeza física do pool. Todos os operadores com require_match precisam ter correspondência distinta; um operador obrigatório sobre source pode impedir a habilidade, mas só remoções no destinatário satisfazem a política.

A decisão não simula cura/dano/capacidade de outros operadores nem reserva instâncias. Esses requisitos continuam responsabilidade do plano atômico. Reutilizar o seletor evita duplicar categorias e regras de proteção; deliberadamente mantém scans limitados dos pools em vez de cache com invalidação adicional.

## Cast, execução e fases

Destinatário comprometido durante preparo válido; outro aliado passar a precisar mais não redireciona o cast. Perda de necessidade/visão cancela na próxima decisão com retry, sem cobrança/cooldown. O executor revalida a condição antes do commit, mesmo quando o operador autorado tem require_match false. Isso cobre dissipação concorrente, expiração e proteção instalada após a decisão. Pulsos já devidos e outras transações anteriores mantêm a ordem do runtime; não há rollback de efeitos de outro ator.

Projétil paga ao lançar e reavalia seus operadores na vítima física do impacto. A política não recolhe projétil, não reembolsa e não altera require_match dos operadores: dissipação opcional pode concluir vazia no impacto. Não há reserva coletiva entre curadores nem escolha automática da categoria/habilidade.

A inspeção por objetivo expõe o alvo escolhido e fases existentes; alvo explícito/self sem correspondência usa `no_matching_effect`, triagem sem candidato usa `no_eligible_ally`. Não há lista nova de status sensoriais ou contagem usada para priorizar. Autoria, schema, edição atômica, undo/reset/load e replay usam o dispatcher/MCP compartilhado.

## Demo

```sh
./target/release/ignis --scene examples/npc-dispel-triage.scene.json --window
```

P inicia, WASD move, clique seleciona, J mostra guias/barras, X cancela cast próprio, R reinicia.

- Verde: apoio em aliados observados, menor percentual de vida entre os que têm correspondência, inclui próprio corpo. Alcance4m, aproximação0,4m, preparo1s, cooldown2s, custo5, remove até uma instância ailment.
- Azul: vida50/100 com efeito protegido. Não deve motivar dissipação.
- Roxo: vida cheia com efeito removível; o verde o atende primeiro.
- Cinza: efeito removível atrás da parede; continua afetado enquanto oculto.
- Vermelho: parado, alvo sensorial principal independente do apoio.
- 1 aplica efeito periódico removível no selecionado;2 aplica protegido;3 dissipa uma instância removível selecionada;4 aplica buff removível da mesma categoria;5 aplica efeito removível em você;6 dissipa em você.

Periódicos causam2 por segundo durante10 pulsos; buff dura10s. Os pacientes reaplicam seus efeitos próprios após o cooldown de60s. Nenhuma dessas remoções cura dano já recebido. Para testar cancelamento, aplique no roxo e use3 enquanto o verde prepara. Para receber apoio, use5; sair da visão durante o preparo cancela. Protegido e removível podem coexistir: o removível continua elegível e o protegido permanece.

Validação: tests/dispel_triage.rs e `python3 scripts/run_verifications.py --only dispel_triage`, além da regressão completa. Capturas/replay sem janela são distintos de teclado nativo e aceite humano.
