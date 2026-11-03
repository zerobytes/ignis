# Mundo em execução e contexto de origem — EAF-63

**Continuidade EAF-70:** [spawn-templates.md](spawn-templates.md) publica nascimento/despawn e reuso seguro, cena47/replay51. Templates e catálogo permanecem imutáveis na execução; RuntimeWorld instala uma nova definição no slot livre por nascimento. `inspect_world` informa população dinâmica com spawn/despawn suportados. A descrição EAF-63 abaixo preserva o contexto histórico e seu contrato de origem.

Continuidade EAF-67: [population.md](population.md) publica retirada via MCP e replay50; nascimento e instalação de templates continuam pendentes. As referências abaixo à seam cfg(test) descrevem o incremento EAF-63.

## Fronteira implementada

`Engine.scene` continua sendo o documento editável, com revisão e undo/redo. `Simulation` possui `RuntimeWorld`: definições imutáveis da execução e placements vinculados à identidade original. A cópia de Scene na construção/reset já existia; foi encapsulada, não multiplicada por efeito. Não há acesso mutável às definições pelo mundo.

Uma entidade física só tem posição se sua instância está presente **e** seu corpo existe. Entidades sem corpo têm placement inicial próprio, também vinculado à instância. Ausência nunca recupera a posição do documento. Render, picking, observação, oclusão, colisão de projétil e seleção de área consultam presença. O snapshot de posições agora inclui também entidades sem corpo.

`inspect_world` no dispatcher/MCP mostra posições presentes, tick, quantidade autorada e `dynamic_population:false`. `inspect_origins` mostra capacidade, reservas, identidade original e se o contexto está destacado. Ambos são somente leitura. `inspect_abilities` também inclui as reservas.

Cena continua no formato46; replay passa a49 porque Cast conserva definição/reserva, o estado de habilidades contém contextos e o conjunto de posições inclui placements sem corpo. Reset/load/edição válida reconstruem o mundo; erro de entrada preserva execução e autoria.

## Origem de uma entrega

Cada Cast conserva o índice imutável da definição, separado do slot do grant. Projéteis, periódicos e áreas persistentes reservam um contexto compartilhado pela identidade da origem antes de pagar/admitir a entrega. Habilidades diretas sem entrega persistente não reservam. O pool tem32 slots, distintos dos9 atores atuais e dos pools64/64/16 de entregas.

Enquanto a origem está presente, fatores continuam dinâmicos: um buff recebido depois do lançamento pode alterar o impacto. O contexto reservado ainda não congela atributos. O hook interno `detach_origin` captura bases, contribuições ativas com seus prazos, identidade, facção, vida e estado de morte, usando buffers previamente reservados. Catálogos de regras/definições permanecem compartilhados na execução.

Depois da remoção, a mesma rotina de composição avalia as contribuições capturadas; `expires_at` continua absoluto. Não se congela um multiplicador final. O índice de facção original consulta a matriz numérica compilada; nenhuma entrega antiga consulta grants, atributos, vida ou facção de um substituto. Identidade do responsável continua nos recibos de dano, dissipação, interrupção e morte.

| Situação | Resultado |
|---|---|
| Projétil lançado ou periódico em outro alvo | Mantém origem e entrega, sujeita às demais regras de contato/alvo/vida. |
| Área fixa | Pode continuar; `end_on_source_death` trata morte, não ausência. |
| Aura ancorada | Encerra por `source_removed`; não vira zona fixa. |
| Cast/canal ainda ativo | Cancelado na retirada, sem novo custo/pulso. Pagamentos anteriores permanecem. |
| Alvo removido/substituído | Vínculo antigo não afeta o substituto. Novo contato físico pode selecioná-lo posteriormente. |
| Transação precisa modificar origem ausente ou emitir ruído na posição dela | `source_unavailable`, rollback integral daquele destinatário, sem reembolso do lançamento. |
| Última entrega dependente termina | Reserva liberada. Recibos e memórias não prolongam retenção. |

As combinações de source effects em projéteis/periódicos continuam proibidas na autoria atual. O guard adicional do executor é testado com fixture interna para que uma futura ampliação de composição preserve atomicidade; este incremento não amplia silenciosamente o formato de habilidades.

A coleta percorre os três pools limitados e monta uma máscara de32 bits. Não há Arc, locks, refcount por recibo ou tabela crescente de origens antigas. Na admissão sem reserva disponível, `origin_budget` rejeita antes de custo/cooldown/efeitos. Captura não pode falhar por falta tardia de espaço.

## Custos e limites

Em aarch64 deste build: Slot96bytes, contribuição88bytes e base24bytes. A reserva antecipada de32 × (Slot +64bases +64contribuições) corresponde a232448bytes (~227KiB), excluindo overhead do alocador. Esse é payload reservado por runtime, não tráfego por tick. Um contexto guarda apenas contribuições do autor, no máximo64; cada entrega acrescenta definição/reserva compactas ao Cast. Não há cópia de cena por projétil.

O pool32 é um limite conservador deste laboratório: não prova suficiência para uma população grande com reposição rápida. É separado dos slots de atores, permite histórico de origens pendentes e fornece backpressure explícito. Ainda faltam atributos indexados por dono/chave, planos transacionais locais e limites derivados dos cenários de escala. Nenhuma afirmação de FPS ou banda de memória medida.

## O que ainda falta para spawn/despawn

A remoção completa usada nos testes é uma seam `cfg(test)`: retira identidade, corpo Rapier e componentes de atividade, preservando autoria. Não existe comando público para remoção/nascimento. A próxima etapa precisa implementar a transação de população, fila/replay de comandos, instalação de placements/componentes/grants, índices, invalidação de cache e ordenação estável de novos indivíduos.

Geometria de apoio, rede e rotas continuam definições estáticas da execução. A seam permite verificar a ausência de obstáculos, mas não representa suporte a retirar dinamicamente pisos/redes/rotas. Não há template/ECS novo, população601 ou edição arbitrária de terreno vivo. A extração de mundo é a fronteira de presença e configuração para essa integração; não a conclusão do sistema de população.

Verificação: `cargo test --locked --release --all-targets`, clippy e `scripts/run_verifications.py` (inclui `world_origins`). Casos de remoção/reuso são Rust; MCP valida interfaces públicas, reservas, origem viva dinâmica, reset, replay/prefixo e pixels offscreen. Sem foreground e sem alegação de aceite de teclado/janela.
