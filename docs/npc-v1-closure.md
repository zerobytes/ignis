# Fechamento integrado de NPC v1 — EAF-149

Direção de Guilherme30/10/2026: executar os nove pontos levantados, incluindo inteligência e personalidade em todos os algoritmos de decisão pertinentes. O programa termina com guarda/civil/aliado/criatura integrados e fronteiras explícitas. Não encerra após adicionar campos/configurações. Main compartilhada, MCP-first, modo offline permanente, revisão estática e testes conforme risco.

| Eixo | Contrato de fechamento | Estado |
|---|---|---|
| Cognição transversal | inteligência0..1, traits independentes graduais, variação seeded por indivíduo, escolha/cadência/planejamento explicáveis sem onisciência | Integrado EAF-150..157: perfil comum em decisão, investigação/planejamento, circulação, tática, apoio, rotinas/diálogo e variantes de apresentação |
| Mobilidade híbrida | walk KCC + trechos jump/fly/glide, entrada/saída física com capacidades/apoio/gravidade e mesmo executor | Implementado EAF-151; restrições de decolagem/apoio em npc-hybrid-mobility.md |
| Descoberta de caminhos | gerar rede a partir de geometria/região/budgets declarados, revalidar ao editar, admitir por corpo/capacidade | Implementado EAF-152, grade/superfícies limitada64/128; não navmesh global |
| Circulação | evitar outros atores, recuperação/filas e gargalos com progresso, prioridades/perfil sem teleporte | Integração terrestre EAF-153; passagens preservadas, trechos aéreos comprometidos mantêm trajetória; limites em npc-circulation.md |
| Tática | posicionamento e escolhas sobre evidência conhecida, risco/alcance/recursos, traços influenciam preferências sem revelar inimigos | EAF-154 integrado: alcance/cobertura/flanco sobre pistas, sugestão sujeita à rota/capacidade; não sensor global de perigos |
| Cooperação | papéis/ordens/formação e reservas de apoio escopadas por identidade/vida | EAF-154 integrado: ordens/formação congeladas e leases de cura por alvo individual; limites em npc-tactics-cooperation.md |
| Rotinas/interação | agendas/necessidades/pontos de interesse, operações sobre objetos e diálogo estruturado configurável | EAF-155 integrado: recursos/agendas, objetos transacionais e diálogos com revisão/vida; limites em npc-routines-interactions.md |
| Corpo/apresentação | vínculo de avatar/rig/clipes por indivíduo, orientação/ação/transição/eventos sonoros coerentes | EAF-156 integrado: avatar/rig/variantes/poses, réplica pública e mix/captura PCM sem dispositivo; limites em npc-presentation.md |
| Persistência | checkpoint durável local, restauração validada de indivíduos/memórias/rotinas/timers e estados correlatos | EAF-158 integrado: checkpoint nativo local e baseline de replay; compatibilidade física/plataforma explícita em npc-checkpoints.md |
| Escala NPC | dimensionamento real de armazenamento/sensores/busca/decisões e budgets; não inferir de1000Players | EAF-157 integrado: opt-in128NPC, sensores espaciais e carga100NPC/500Players; limites e p95 em npc-population-scale.md |

Os nove eixos têm implementação, commits locais e gates registrados. Este é o fechamento técnico do NPC v1; aceite humano do cenário conjunto permanece separado. Orçamentos finitos/truncamentos são parte do resultado público.

## Cognição: decisão estrutural inicial

Usar o seletor atual de objetivos como executor e acrescentar utilidade baseada em contexto, em vez de trocar tudo por behavior tree ou inserir LLM obrigatório. Prioridade do objetivo continua um nível explícito; coeficientes ponderam candidatos elegíveis dentro desse nível. Invalidação de alvo/vida/condição nunca aguarda intervalo de pensamento.

Inteligência geral0..1 controla esforço/cadência de deliberação e cobertura da busca dentro dos tetos existentes, nunca visão através de parede, precisão física artificial ou habilidade ausente. Personalidade independente: anxiety, caution, concern, fear, courage, curiosity, aggression, sociability, discipline. Traços coexistem: coragem aceita risco, cautela prefere preparação; não são complementos obrigatórios. Defaults neutros, perfil inteiro opt-in e omissão conserva decisões antigas.

Seed/variation distribuem perfis por InstanceId reproduzivelmente, sem sorteio por frame. Diagnóstico conserva perfil efetivo/scores/opção/motivo/próxima avaliação, comparado no replay. Autoria f32 normalizada, representação quantizada no runtime. Os demais algoritmos de decisão consomem esse perfil comum, com integrações documentadas na matriz. Física, validação de identidade e persistência preservam as regras e o perfil; inteligência não enfraquece essas invariantes nem altera FPS visual.


## Entrega integrada e fronteira de encerramento

Scene 97 / replay 107 / protocolo 3 / conteúdo 1 / checkpoint 1. O programa EAF-149 fecha tecnicamente com as unidades EAF-150..158. As notas históricas anteriores que diziam “aberto” não descrevem este estado atual. Isso não conclui automaticamente todos os futuros NPCs, o marco geral M09 ou o roadmap da engine.

Cenário [npc-v1.scene.json](../examples/npc-v1.scene.json), salvo pelo MCP, combina guarda, civil, aliado e criatura. Roteiro, comandos e diferenças entre autoria/manual e prova automática em [npc-v1-demo.md](npc-v1-demo.md). Captura/validação inteiramente offscreen, sem foco ou janela. Gate final: 79 testes Rust distintos/12 alvos, 26 cenários MCP/92 grupos, check e Clippy all-targets, fmt/Python/diff. Evidência e compatibilidade em [npc-checkpoints.md](npc-checkpoints.md).

Inteligência controla esforço/cadência/cobertura da busca; traits independentes ponderam preferências. Não são uma garantia de que aumentar um escalar vence em todo contexto. Exposição/percepção/recursos/capacidade e prioridade autoral continuam obrigatórios. Onisciência exige configuração explícita.

Limites aceitos no v1: rede gerada até 64 nós/128 links; circulação local terrestre e passagem autorada até 8 participantes; voo mantém trajetórias físicas comprometidas; apoio reserva cura por alvo individual; diálogos estruturados e agendas finitas. Avatar nativo tem rig/clipes/variantes e PCM capturável, sem criador de corpos, importação Blender, IK/root motion ou reprodução automática em dispositivo. Capacidade opt-in até 128 NPCs/1024 atores totais; a carga medida tem 100 NPCs + 500 jogadores, não 1000 NPCs. Checkpoint inicial depende de Rapier/plataforma compatíveis. Essas extensões são novos contratos, não pendências ocultas deste recorte.


## Direção posterior — mundo contínuo e extensões autorizadas

Em30/10 Guilherme reforçou mundo procedural sem fronteira artificial de cena e solicitou chats paralelos para circulação aérea e apresentação completa. O fechamento acima é do recorte local validado; não implementa navegação/streaming global nem save portável. Requisitos e proposta em [continuous-world-and-portable-saves.md](continuous-world-and-portable-saves.md); coordenação em [npc-parallel-followups.md](npc-parallel-followups.md). Os limites globais atuais de cena/coordenadas são dívida arquitetural frente a esse requisito, não limites permanentes do produto.
