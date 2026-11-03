# Escala explícita de NPCs — NPC v1

Preflight após apresentação95/105/protocolo3. Migração não pode presumir que
1.000Players provam1.000NPCs: percepção, reconhecimento, comunicação, planejamento
e geradores têm custos próprios. Primeiro teto128NPC e1024atores totais,4096entidades.
Capacidade autorada por cena com default8 mantém reserva e identidade antigas após
normalização; versões antigas recusam opt-in novo. Não ampliar só uma constante.

Dimensionar membros de gerador pela capacidade real, relatórios pela população,
perfis físicos por NPC+template. Preservar ordem InstanceId, lifecycle, quotas
por fonte, FIFO e limite de ação por fase; ordens de grupo continuam transações
limitadas, não dividir silenciosamente para caber. Perfil comum mantém esforço de
planejamento e decisões; não degradar cegamente sensores e fazer falso alvo perdido.

Visão/som usam broadphase do mesmo snapshot anterior à física e narrowphase exato.
Consulta visual tem operador any, áudio compõe transmissores em ordem autoral para
não mudar arredondamento/empates; candidatos da árvore não são autoridade de alvo.
Fallback linear em chamadas públicas sem índice; oráculo existente mantém contrato.
Sem navmesh universal ou planejamento pesado para128unidades simultâneas prometido.

## Checklist prévio

1. Inventário de arrays/máscaras/índices, não mexer limites8 de objetivos/efeitos.
2. Capacidade e scratch por mundo; reusar Vec e evitar cópia de todos membros por tick.
3. IDs u16/Reference/vidas; total<=1024, membros/budgets validados antes do nascimento.
4. BVH compartilhado e kernel exato; conhecimento somente após sensores autorizados.
5. Ruído/ações/rotina/origens/efeitos já têm tetos, manter diagnósticos de limitação.
6. Render separado; truncamento visual não altera simulação e não é prova de FPS.
7. Offline permanente, transporte opcional sem alterar protocolo por mera capacidade.
8. Cena/replay versionados, default8 para passado; falha não consome identidade.
9. Uma prova crítica de slots altos/reuso/gerador/replay e carga concreta, não teste por flag.
10. Registrar memória/custos/tempo de simulação do cenário medido e limites restantes.

## Implementação e revisão depois do código

Scene96/replay106/protocolo3; Scene.npc_capacity/SetNpcCapacity e operação de lote.
Default8 preserva cenas antigas; opt-in1..128 com total<=1024/reserva real e4096entidades.
Gerador target_alive pode alcançar capacidade; attempts_per_cycle continua1..8,
32pontos/8geradores, não criar rajada128nascimentos numa fase. Membros Vec dimensionados
uma vez, movidos para estado local e devolvidos sem clonar lista por tick; read limitado128.
Relatórios Vec reutilizado por mundo, identidade e ordem existentes preservadas.
Perfis terrestres limitados a128NPC+16templates, deduplicados como antes.

1. Arrays8 de membros/relatórios removidos; limites8 de efeitos/objetivos/rotinas
   mantidos por domínio. Passagem autorada ainda até8participantes por piso/gargalo.
2. Buffer de relatórios reservado uma vez; membros movidos com mem::take, sem heap novo no tick.
3. Capacidade u16/slots u16/referência+vida; rejeição de redução preserva mundo/identidades.
4. BVH compartilha snapshot e flags agregados de bloqueio visual/sonoro. Narrowphase
   intacto, candidatos de som ordenados por máscara4096bits/512bytes local sem alocação.
5. Custo de percepção ainda NPC×atores para classificação/visibilidade, mais pistas
   próximas. Não amostrar cegamente e inventar perda de alvo; inteligência já limita
   planejamento/cadência/objetivos, traits não são alterados pelo número de atores.
6. Renderer continua separado, sem prova de FPS; não dimensionar avatar pelo limite autoral32.
7. Nenhuma conta/servidor obrigatório; continuous mantém zero históricos de input.
8. Vetores de gerador admitem JSON antigo8, novo formato/versão em Scene/replay.
9. Uma prova crítica nova compara linear×espacial para100NPC/500Players, slots>=500,
   rádio>8, gerador24membros, débito/reuso sem herança, erros de capacidade e replay.
10. Custos mantidos: reconhecimento/conhecimento por NPC×slots, KCC de todos os corpos,
    planejamento síncrono com budgets e tempos finitos;128 é teto do recorte, não promessa
    de128planejamentos pesados simultâneos a60Hz ou de1000NPCs.

## Evidência e carga

Buildd39d26100daa70f7:59Rust/7alvos (58integração+1oráculo BVH),9MCP/35grupos,
checkalltargets/Clippy lib/bin/exemplo/prova-Dwarnings/fmt/Python/diff. Logs
npc-scale-*.log, report MCP e exemplo npc-population-scale.scene.json salvoMCP.
Receita compartilhada examples/support/npc_scene.rs e executável de prova npc_load:
100NPC/500Players/628slots,600KCC grounded,500jogadores com input real e ataques,
50NPCs com visão/ampla+rádio/ataque e50com visão1m/audição/pistas anônimas, perfis0/.5/1.
120ticks:71450dano/custos exatos,5181passos e50NPCs ouvindo, históricos todos0.

P95simulação31,559→18,576ms; total18,706ms na repetição com flags da BVH, mesmos
resultados. Medidas locais não isolam toda atividade externa e não garantem60Hz,
FPS/render/WAN/combate arbitrário. Sem renderer/rede/avatares nessa carga, sem
inteligência artificialmente desligada nem sensores substituídos por resultados prontos.
Persistência e demonstração final do programa149 permanecem.
