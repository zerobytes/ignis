# Triagem de aliados observados

EAF-55, cena/replay44. `use_ability.policy.allies` escolhe um destinatário de apoio sem substituir o alvo/memória de combate. É opt-in por objetivo, apenas para habilidades `selection: actor`; a mesma definição continua disponível ao jogador e ao executor comum. Não é um sistema de classes ou uma heurística que deduz qual habilidade usar.

```json
{
  "kind": "use_ability", "slot": 1, "retry_ticks": 15,
  "policy": {
    "allies": { "ranking": "lowest_health_ratio", "include_self": true },
    "vitality_below": { "subject": "target", "basis_points": 7000 },
    "approach": { "margin": 0.4 }
  }
}
```

`allies` ausente preserva o alvo sensorial anterior. `include_self` padrão false e só permitido com entrega direta (projétil não pode atingir o próprio emissor); quando true, vida/posição próprias são conhecidas sem visão. `nearest` classifica por distância XYZ; `lowest_health_ratio` exige vitalidade e compara frações exatas de vida/máximo por produtos i128, depois distância² e ordem lexicográfica dos IDs no catálogo. 60/200 é mais urgente que40/100. Escalas distintas não alteram percentuais. Sem limiar, um aliado cheio pode ser escolhido: a decisão é autorada, não deduzida dos operadores de cura/buff/recurso.

## Evidência e isolamento

Aliados são definidos pela relação direcional do observador (`allied`); isso não exige aliança mútua. Todos os objetivos de apoio compartilham uma passagem sensorial por NPC/tick. Alcance/FOV/pitch/oclusão usam os sensores existentes. Reconhecimento acumula uma vez por candidato/tick em trilhas de apoio independentes, usando os mesmos parâmetros de confirmação/decay; é intencional não transferir exposição entre combate e apoio. Um alvo explicitamente aliado pode causar duas consultas geométricas limitadas, sem duplicar confirmação na mesma trilha.

Som, rádio e memória antiga não fornecem posição/vida atual para apoio. Saúde só é consultada depois da confirmação corrente, conforme o contrato anterior de habilidades. Onisciência continua opt-in explicitamente rotulada. Território filtra posições observadas; retorno/disable/morte limpam evidência de apoio. Cadáver observado não é candidato; morte oculta não limpa a memória de combate. A inspeção privilegiada não participa das decisões.

A memória de perseguição, rádio e audição continuam independentes. Condições gerais do objetivo (`visible_target`, `visual_memory` etc.) ainda se referem ao alvo sensorial principal: usar `when: always` para apoio independente. A confirmação de morte de um alvo explícito continua invalidando sua memória mesmo quando recebida na passagem visual de apoio.

## Seleção, preparo e execução

Antes do ranking, aplicam-se relação, vida/limiar, disponibilidade/custos e alcance. Sem aproximação, um candidato inalcançável por distância não bloqueia outro ao alcance. Com aproximação, o motor existente recebe o destino observado em XYZ; não há garantia de caminho viável nem escolha por custo de rota. Aproximação pode reavaliar o melhor candidato a cada tick; seu alvo nunca é uma posição oculta.

Ao iniciar o cast, o destinatário fica comprometido. Um aliado que se torna mais urgente não reinicia o preparo. Perda de visão/condição, morte ou preempção cancela e aplica retry; a próxima decisão pode escolher outro. Alcance/oclusão são revalidados no executor antes de pagar, como no contrato de cast anterior. O limiar vital de triagem é revalidado também no commit, inclusive se outra execução curou primeiro no mesmo tick. Canalizações usam a mesma condição por pulso. Projétil lançado conserva seu contrato: não é recolhido se a necessidade mudar depois do lançamento.

Não há reserva de cura entre vários NPCs, previsão de dano, escolha tática de buff, triagem por status/dissipação, cura atrás de paredes ou otimização de grupos para área. Uma habilidade de área pode mirar um aliado por essa regra, mas os destinatários secundários continuam física de entrega e não entram no conhecimento do NPC.

`inspect_npcs`/`inspect` expõem `support.observed_allies` com IDs, posição, tick e fonte; `support.knowledge` contém trilhas limitadas; decisões por objetivo têm fase/target_id. `no_eligible_ally` significa que nenhum candidato passou pelos filtros, não ausência de aliados no mundo. `inspect_abilities` conserva custos/recibos e destinatário do cast. Comando externo direcionado a NPC também pode usar essa evidência corrente de apoio, preservando as mesmas barreiras do executor. Autoria/edição procedural, schema, undo/reset/load e replay passam pelo dispatcher/MCP existentes.

## Demo e validação

```sh
./target/release/ignis --scene examples/npc-triage.scene.json --window
```

P inicia/pausa, WASD move, J mostra guias/barras, clique seleciona, X cancela o próprio cast, R reinicia.

- Verde é o curandeiro: menor percentual, abaixo70%, inclui a si próprio, alcance4m/aproximação0,4m, preparo45ticks (0,75s), cura30, cooldown120ticks (2s), custo5. Visão360°/20m.
- Roxo começa35/100 e azul50/100. Verde trata primeiro roxo e reavalia após cooldown. Seu alvo sensorial principal é o vermelho, que fica parado para isolar o teste.
- Cinza começa10/100 atrás da parede e não recebe cura enquanto continuar oculto.
- Jogador é aliado:1 causa20 no selecionado (inclusive aliados, para provocar necessidade);2 cura30 no selecionado;3 cura30 em si. Recursos1000 sem regeneração.
- Fique abaixo70% para receber apoio; fuja atrás da parede durante o preparo para testar cancelamento. Curar um destinatário antes do verde concluir também pode tornar o cast desnecessário. Outro aliado ficar mais ferido não troca o destinatário durante o cast.

`python3 scripts/run_verifications.py --only triage`; testes Rust em tests/triage.rs e regressão completa headless. Capturas offscreen/replay não comprovam teclado ou aceite humano; nenhuma janela do usuário é manipulada nos testes.

Diagnóstico operacional desta validação: executáveis ficaram em `_dyld_start` antes do Rust. O verificador aceita `IGNIS_MCP_STARTUP_TIMEOUT=600` somente para initialize (default120, intervalo1..600s); outros budgets e checks de gameplay não mudam. Falha de transporte encerra apenas o processo criado pelo Client e fecha streams, evitando órfãos. Isso não corrige nem comprova desempenho de inicialização do macOS.

EAF-56 acrescenta o filtro opt-in [require_dispel_match](dispel-triage.md), combinável com ranking/limiar. Os limites anteriores continuam valendo quando o filtro não está configurado.
