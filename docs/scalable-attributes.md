# Buffs e bases para o runtime 1000 — EAF-138

## Decisão anterior ao código

O mundo 64 buffs/512 bases ainda não sustenta 1000 personagens. Quota 64 por destinatário
permitiria 1000 buffs próprios, porém impediria 1000 fontes de contribuir no mesmo
chefe; quota 64 por fonte impediria um buff de grupo em 1000 destinatários. Escolha:
quota por fonte e por destinatário = max(64,slots de ator), teto global = slots×
quota. Com 1024 slots, teto absoluto 1048576 contribuições; é limite de armazenamento,
não promessa de CPU a 60 Hz nesse extremo. Uma fonte não monopoliza o orçamento do
mundo. Cobrar Entity física, inclusive gerações antigas, impede contornar quota
por reuso; referências completas continuam separando indivíduos.

Contributions mantém ordem densa/JSON em lista e slice somente leitura; índices
por dono/definição, dono/ordem, fonte/ordem e ID. Push insere só novo índice;
refresh remove/apende na ordem antiga, ajustando índices em vez de ordenar todo
mundo por mutação. Remoção/compactação/serde reconstroem derivados antes da próxima
leitura. Custo O(M) de deslocamento denso/índices em refresh permanece declarado;
alternativa de slots estáveis/tombstones alteraria mais contratos e não será
adotada sem medição que a justifique. Capacidade/refresh consultam fonte/dono,
planos contam apenas pending 8 e remoções locais 512, sem clone do mundo.

Buff dispel também passa a IDs locais (até 512 por plano); não ampliar máscara u64.
Seleção por dono conserva ordem densa. Composição conserva prioridade/definição/
regra/instância e snapshot inicial da transação; nenhuma soma/agregação que mude
arredondamento ou regras de tags. IDs únicos e contador são validados em serde e
exaustão é recusada antes de cobrar.

Origem viva reserva espaço para a quantidade projetada de buffs do dono antes de
instalar/retirar recursos, em vez de reservar 1024 para todos upfront. Reserva nova
considera buffs já existentes; decode de State recompõe reserva a partir dos
buffs/refs. Snapshot destacado fica congelado e não cresce mais; detach continua
copiando em buffers pré-reservados. Buffs locais e seleção removida não podem
realocar dados de mundo para simular transação.

Scene 88 autoriza bases até 65×(entidades+templates), conservando rejeição>512 em
versões anteriores antes da normalização. Mapas de IDs na validação/compilação
evitarão busca de todos donos para cada base. 65 atributos/64 definições de buff e
64 tags são domínios diferentes e não mudam. Replay 96 explicita quotas/estado;
MCP/UI já usam os mesmos comandos, sem nova dependência, rede ou processo offline.

## Checklist estático prévio

1. Capacidade/refresh por índices, não scan de todas contribuições; compile de
   bases indexado uma vez. Composição ainda percorre contribuições relevantes.
2. Vetores/índices densos reutilizados; plano local até 8 instalações/512 remoções,
   snapshots crescem por necessidade antes do custo. Sem prealloc A² por ator.
3. Actor slot, Entity, atributo/buff e ID de contribuição mantêm domínios próprios;
   não truncar máscaras nem inferir Entity a partir de Reference.slot.
4. Índices compartilhados por orçamento/refresh/dissipação/captura, reconstituídos
   por todos mutadores/serde, ordem de contribuição não muda.
5. Quotas fonte/destinatário e teto real; refresh/pending/remoções e contador
   calculados antes de mutar; falha tardia não altera custo/vida/buffs.
6. Sem I/O/GPU novo no tick; inspector/serde mantêm lista completa e custos sob
   demanda. Memória máxima declarada, sem alegar carga máxima medida.
7. Mesma autoridade/fases e composição por transação, sem locks/workers/PRNG/ECS.
8. Expiração/morte/retirada/reuso e snapshot destacado mantêm geração/prazos;
   reset/load/edit/serde reconstroem índices, reserva e contratos de versões.
9. Só prova crítica 1000 buffs/mesmo alvo/área, quota dupla, IDs>63, rollback,
   composição/captura/serde/replay/legado; cobertura existente por impacto e carga.
10. Mantidos custos densos, composição real, payload de snapshots, JSON/Rapier;
    medir antes de acrescentar cache/estrutura mais complexa. Ruídos e servidor
    permanecem etapas separadas; single player offline não depende delas.


## Revisão do código e validação

1. `Registry::new` e validação indexam IDs uma vez. Orçamento/refresh consultam
   buckets por fonte/dono e IDs removidos; composição percorre apenas dono/buff.
2. Quatro índices derivados acompanham o vetor denso. Push insere novo índice,
   refresh ajusta índices sem ordenar tudo; compactação reindexa uma vez. Planos
   mantêm até oito instalações e 512 IDs removidos, sem vetor do tamanho do mundo.
3. Slots de ator, Entity, atributo/buff e referências permanecem domínios distintos.
   Scene 88 e replay 96; JSON de contribuições continua uma lista somente leitura.
4. `owner_indices`, `source_indices`, `get_id` e `for_buff` compartilham os mesmos
   mutadores. Serde recompõe índices e rejeita IDs duplicados/contador incoerente.
5. Quotas incluem pending, refresh e remoções. Contador esgotado é recusado antes
   de custo; falha tardia em recurso conserva vida, saldo e contribuições.
6. Nenhum renderer/I/O novo no tick. Na plataforma medida, Instance ocupa 88 B e
   quatro índices custam 32 B por entrada: 120 MiB de dados/índices no teto lógico
   de 1.048.576, além da folga de capacidade dos Vecs. Snapshots retidos têm custo
   adicional proporcional aos buffs capturados e às origens admitidas (32 por
   slot); 1.024 contribuições equivalem a 88 KiB de dados por origem. Não reservar
   esse extremo para todos os slots nem inferir um orçamento universal de memória.
7. Mesmas fases e autoridade sequencial; composição, arredondamento, seleção de
   dissipação e aplicação de custos não mudam de ordem. Sem workers/dependências.
8. Snapshots vivos crescem antes de cobrar, inclusive origem criada com buffs já
   presentes; decode recompõe reserva. Retirada captura sem realocar e congela
   referências, bases, contribuições e expiração originais. Lifecycle/reuso ainda
   limpa estados; bases antigas acima de 512 são rejeitadas antes da normalização.
9. Uma prova crítica nova cobre 65.000 bases, 1.000 fontes/mesmo chefe, área de
   1.001 destinatários, quotas duplas, refresh/cobertura existente, IDs altos,
   rollback, captura/serde/composição/replay e exaustão de contador. Total único:
   137 testes de integração + 2 oráculos de índice = 139 Rust; Clippy all-targets,
   lint da prova final e 14 cenários MCP/57 grupos passaram. A cobertura offline
   inclui execução com rede negada no processo macOS.
10. Custos mantidos: deslocamento O(M) em refresh, composição real, cópias dos
    snapshots e serialização sob pedido. Nenhuma prova de máximo de contribuições
    sustentado, FPS gráfico, rede, clientes conectados ou MMO completo.

Build auditado `ab3fddd4a6dc2d68`, fonte/binário/example iguais, macOS Apple Silicon.
Receita: `target/release/examples/runtime_load --delivery buff`, 1.000 Players
em movimento com piso/colisão/visada reais, dano + buff neutro refresh em pares a
cada seis ticks; 60 de aquecimento + 600 medidos. Pico 1.000 buffs, 110.000 ações,
661.000 controles, 110 impactos por jogador/vida 999890, nenhuma entrada histórica
retida. Simulação p50 1,935 ms / p95 6,357 ms / p99 6,644 ms / máximo 7,146 ms;
total p95 7,104 ms / máximo 7,932 ms. Sem ruídos, NPCs, render ou transporte nesta
carga. O deslocamento denso medido não justifica trocar agora o contrato por
slots estáveis/tombstones. O teste separado de 1.000 buffs no mesmo chefe prova
contratos e integridade, sem alegar desempenho sustentado de todas as combinações.

Logs: artifacts/attributes-scale-{critical,index,integration,clippy,critical-clippy,
build,mcp}.log; auditoria/report em attributes-scale-{audit,report}.json e carga
runtime-load-buff.json. Os primeiros ensaios corrigiram superfície inválida e uma
suposição errada de slot no fixture; nenhum critério funcional foi relaxado.
Próxima etapa: orçamento dos ruídos de habilidade ainda global de 16 por tick e
revisão dos consumidores de inspeção para slots altos. App humano preservado.
