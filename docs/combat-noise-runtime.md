# Ruídos de combate e inspeção — EAF-139

## Decisão antes do código

16 ruídos globais de habilidades impedem 1.000 fontes no mesmo combate. Usar
16 por slot físico de ator e teto slots × 16, incluindo gerações antigas quando
compartilharem o slot. Contadores reutilizados por fonte ficam no Runtime, fora de FrameScratch (que é
movido temporariamente durante step). A leitura completa confirmou que a prévia
NPC só consulta Offer/prontidão/estado e não executa plan_effects. A fase existente
já consome ruídos anteriores nos sensores e limpa a fila antes dos planos: zerar
contadores nesse mesmo ponto. Nenhum relógio/época adicional é necessário; entrada
sonora do tick anterior não consome orçamento de saída deste tick.

Fila de saída reserva slots × 16 antes dos commits; decode/restauração de State
não pode retirar essa reserva do próximo passo. Contratos atuais admitem uma ação efetiva por ator/tick e até oito efeitos;
ruído só é permitido em ações diretas/canalizações. Projéteis, áreas e agendas
periódicas rejeitam Noise na validação. Assim, a quota 16 por fonte é uma guarda
defensiva; spam já encontra Busy antes de ultrapassá-la. Nenhum efeito/custo é aplicado
antes de validar quota, teto e serial. Emissão incrementa contador da fonte no
mesmo ponto em que insere Noise; ordem/IDs/namespace/referências e consumo no
próximo tick permanecem. Máximo lógico de 16.384 eventos, não promessa de custo
constante de sensores: todo ouvinte ainda avalia os estímulos relevantes.

Anel recente 128 é só diagnóstico, fila completa alimenta percepção.
InspectNoises exporá também a fila de habilidades, anel e quotas para diagnóstico
completo por MCP/native; JSON completo é custo explícito sob pedido. Ruído
manual autoral continua 16/tick e anônimo, sem nova regra de posse. Passos já
reservam fila proporcional aos personagens e não são descartados pelo anel
visual. Nenhuma dessas estruturas equivale a áudio reproduzido ou transporte.

Inspeção periódica ainda usa máscara u16, inválida para atores altos. Substituir
pelo ActorSet de 1.024 bits existente, sem ampliar máscaras de habilidades/tags
que pertencem a outros domínios. Scene 88 não recebe novos campos; replay 97
registra a política de execução. Mesmo núcleo local e offline, sem dependência.

## Checklist estático prévio

1. Contagem O(1) por efeito/fonte, zero busca do histórico por admissão. Sensores
   mantêm custo real do conjunto completo, sem remover participantes.
2. Contadores e fila reservados pelo domínio real; clear/fill reutilizam buffers
   antes do commit. Anéis de diagnóstico continuam limitados.
3. Fonte é slot de ator físico; Entity e Reference continuam separados. Contador
   u8 até 16, serial u64 e ActorSet 1024, sem shifts de ator em máscara de 16 bits.
4. Mesmo orçamento atende cast direto e pulsos de canal. Fase existente
   separa sensor e novas emissões; mutação só no executor.
5. Quota/teto/serial antes de custos/dano/recibos; nenhuma fonte consome toda fila.
6. Nenhum GPU/I/O novo no tick; inspector explica quotas. Guia usa conjunto fixo
   existente; anel visual não determina recepção de som.
7. Mesmas fases/ordem/autoridade, sem lock/thread/rede/PRNG. Som segue anônimo.
8. Reset/load/edição recompõem Runtime; State restaurado recebe reserva antes do
   passo. Replay/prefixo conservam fila pendente e contador, sem reemissão.
9. Uma prova crítica nova de 1.000 fontes/spam/rollback/reset de tick/IDs/replay;
   ampliar a prova existente de periódicos para inspeção de 1.000 marcadores e
   selecionar cobertura de audição/casts/offline/persistência por impacto.
10. Custos mantidos: fila total, avaliação de ouvintes, serialização completa,
    limite autoral manual e anel visual. Sem capacidade de rede/FPS alegada.


## Revisão final e evidência

1. `Runtime.noise_counts` faz admissão/incremento O(1) por fonte. Não recontar
   Vec global a cada plano. Guardas de teto/serial continuam antes dos efeitos.
2. Fila reserva 16 × slots na criação e após clear, antes de qualquer commit,
   inclusive State restaurado; counters u8 usam um byte por slot. Capacidade de
   eventos é proporcional a sizeof(Noise), além de folga de Vec e anel128.
3. Índices de ator e Entity são separados; Reference/IDs/namespaces preservados.
   Scene88 inalterada e replay97. Sem nova dependência ou rede obrigatória.
4. A leitura completa corrigiu hipóteses do preflight: nenhum relógio extra,
   porque sensores/limpeza/planos já têm fases; Noise é recusado em projéteis,
   áreas e periódicos. Não ampliar esses contratos para fabricar uma prova.
5. Quota16por fonte é defensiva: ação única e até8efeitos já limitam o produtor.
   Spam Busy não cobra nem produz ruído/dano adicional; outros1.000produtores
   conservam capacidade. Serial esgotado recusa toda transação, inclusive canal.
6. InspectNoises entrega fila completa/anel/quotas;7.000eventos inspecionados
   versus128recentes. Guias mantêm anel limitado, sem descartar percepção. O
   conjunto periódico ActorSet preservou1.000marcadores no RenderFrame; nenhuma
   alteração de shader/renderer ou interferência em janela humana.
7. Mesmas fases e ordem/identidades: nenhum estímulo duplicado, som segue anônimo
   e eventos são consumidos no próximo tick. Sem workers/locks/PRNG novos.
8. Clear/reserva/counters reiniciam juntos; reset/load/edição recompõem Runtime.
   State serde, replay e prefixo conservaram fila/IDs, duas janelas7000cada,
   sem misturar recepção anterior com emissão atual.
9. Uma prova crítica nova em scalable_actors protege1.000fontes, spam/refund,
   fila/IDs/referências/inspeção/serde/prefixo/replay/contador. Prova existente de
   agendas agora conta1.000marcadores. Dois testes existentes foram adaptados ao
   contrato aprovado:3fontes independentes agora emitem24eventos (ou21em canal),
   e falha de canal continua comprovada por injeção de serial esgotado, sem
   custo/dano parcial.93Rust únicos,Clippy all-targets e13MCP/53grupos passaram,
   incluindo render/replay/audição/passos/identidade/origens e legado offline.
10. Mantidos custos reais dos sensores e JSON completo sob pedido, anel128 e
    ruído manual16/tick. A carga medida não tem ouvintes/NPC, áudio ou rede;
    sua aprovação não prova custo de1.000ouvintes nem transporte/MMO completo.

Build `47b47e518c38931c`, macOS Apple Silicon/release; fonte/binário/example
conferidos antes dos cenários. Receita `runtime_load --delivery noise`:1.000Players
movendo, dano+um ruído em pares/6ticks,60aquecimento+600medidos,110.000ações e
110.000ruídos de habilidade,110impactos/vida999890para todos, zero histórico
retido. Simulação p50=1,811ms/p95=2,434ms/p99=2,469ms/máximo2,503ms;
totalp95=3,124ms/máximo3,208ms. Sem render, ouvintes/NPC, buffs, passos ou transporte.

Logs/report/auditoria em artifacts/combat-noise-{integration,clippy,build,mcp}.log,
combat-noise-{audit,report}.json e runtime-load-noise.json. Só um build corrente;
app humano e frentes paralelas preservados. Próximo: integrar cargas de entregas,
buffs, controle terrestre e passos no mesmo cenário antes de alegar fechamento
da base de escala. Rede/replicação/persistência de mundo continuam etapas próprias.
