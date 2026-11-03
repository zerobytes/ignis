# Execução contínua e gravação — EAF-132

## Contrato e decisão anterior ao código

O limite de36000ticks e4096eventos pertence ao laboratório de replay, não à
simulação de um jogo. Single player offline e multiplayer usam a mesma simulação
local; nenhum modo exige rede, conta ou processo de servidor separado.

Alternativas: ampliar logs/relógio apenas adia exaustão e acumula memória; descartar
um prefixo e chamá-lo replay inteiro gera um artefato impossível de reproduzir sem
checkpoint. Escolha: modo `recorded` padrão mantém a gravação limitada atual;
modo `continuous` descarta entradas consumidas, mantém apenas filas pendentes e
recibos limitados já existentes. Um anel de128ruídos manuais, com validade de90ticks,
preserva guias de diagnóstico sem alimentar novamente os sensores. Replay completo
só pode ser exportado no modo
gravado. Checkpoints persistentes/streaming de replay ficam para contrato próprio.

Seleção explícita por comando `set_execution_mode` e construtor nativo. Só mudar
política no tick0, pausado e antes de entradas/posse/eventos; erro não reseta o
mundo. `reset`, edição e load recompõem execução preservando a política escolhida,
mas continuam invalidando handles e históricos como antes. Inspeção declara modo,
filas pendentes/retidas e orçamento. Documento de autoria não recebe essa opção.

Sequências de comandos são64bits, monotônicas durante a execução e reiniciadas
somente sob reset com novo escopo. Limites de admissão por tick permanecem;
entrada rejeitada não consome sequência. Filas de população/habilidades/ruídos
são consumidas uma vez na fase existente e limpas em contínuo; controles aplicam
imediatamente e não acumulam histórico nesse modo. Orçamento de controle conta
eventos do tick mesmo sem retenção e reserva liberações dos vínculos presentes.

Relógio interno64bits; recusar avanço/admissão próximo da exaustão numérica, antes
de mutar e com margem para prazos configurados. Não prometer execução matemática
infinita. IDs de ruídos/counters também precisam domínio amplo para que limpeza
da fila não reutilize identidade e não cruze namespaces de passos/habilidades.
Scene87 permanece; replay92 distingue explicitamente política e sequências64bits.
Pools de entregas/origens, admissão de combate1000 e transporte são etapas próprias.

## Checklist estático prévio

1. Fase/tick existentes, sem worker, thread de rede ou relógio de parede novo.
2. Descartar com Vec::clear preservando capacidade; nunca remover prefixos com
   cópia de cada evento por tick. Modo gravado conserva as cópias sob exportação.
3. Política enum tipada; comandos/IDs/counters64bits e validação antes da mutação.
4. Contador de controles por tick evita buscar histórico para admitir comandos;
   sequência independente de `Vec::len`, sem reaplicar entrada consumida.
5. Filas pendentes limitadas por budgets; histórico gravado segue limite explícito.
   Recibos não viram log ilimitado e controles não reservam toda cota semanal.
6. Nenhum I/O/GPU por tick; arquivo de replay recusado antes de escrita em contínuo.
7. Mesmo dispatcher sequencial e ordem/fase autoritativa. Sem lock/dependência nova.
8. Reset/edição/load limpam filas e escopos, preservam somente política da sessão;
   falha de modo/comando conserva mundo, posse, fila e sequência.
9. Criar somente guard crítico de continuidade/retenção/IDs/replay; reutilizar
   transações/posse/lifecycle e gate offline existentes pelo impacto.
10. Mantidos: custo de simular atores reais, recibos/candidatos, serialização
    explícita; não alegar1000clientes, persistência de mundo, FPS ou throughput.

## Uso

No tick0, pausado e sem comandos admitidos:

```json
{"command":{"op":"set_execution_mode","mode":"continuous"}}
```

`inspect_execution` informa política, totais admitidos, filas retidas/pendentes e
anel de diagnóstico. Na API Rust, `Engine::new_with_mode(scene,
ExecutionMode::Continuous)` inicia com a mesma política. `recorded_replay` recusa
execução contínua; o exportador de arquivos usa essa guarda antes de escrever.
`replay()` mantém a API de snapshot diagnóstico com marca explícita de modo;
`verify_replay/replay_at` recusam essa marca em execução não gravada.

## Revisão após implementação

1. Mesmos ticks/fases da Simulation; a política pertence à sessão, fora de Scene.
2. Vec::clear/drain conservam buffers; nenhum log completo em contínuo. Anel128
   é reservado ao selecionar a política e expira pela frente em ordem de tick.
3. Sequências, pedidos, IDs e contadores de ruído64bits; namespaces intercalados
   após30bits, sem colidir passos/habilidades/manual. Limites de cena preservados.
4. Controles contam eventos por tick sem reler todo histórico; counters não usam
   comprimento de fila. Reconstrução de replay recompõe os totais/pending corretos.
5. Pendências16habilidades/64população/16ruídos por tick ainda são limites próprios;
   controles4096/tick reservam liberação. Pools/combate1000 seguem EAF-133/futuro.
6. Nenhum I/O/GPU novo no tick; SaveReplay falha antes de alterar arquivo. Guias
   de ruído leem o anel, não o usam como novo estímulo ou conhecimento de NPC.
7. Nenhum lock/worker/dependência/transporte novo; mesma autoridade offline.
8. Reset/edição/load/projeto preservam política, zeram filas/counters e renovam
   escopos. Troca tardia de política ou comando inválido é atômico.
9. Somente2guards Rust novos e3grupos MCP críticos: continuidade/retenção/IDs,
   falhas atômicas/relógio, exportação segura, offline e substituição de projeto.
   Cobertura existente selecionada pelo impacto; evidências em validation.md.
10. Mantidos com motivo: modo gravado ainda finito, serialização completa sob
    pedido, recibos limitados e física/candidatos reais. Sem checkpoint/savegame,
    transporte, FPS ou garantia de1000clientes inferidos.
