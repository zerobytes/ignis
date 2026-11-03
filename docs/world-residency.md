# Manifesto e planejamento de residência

Continuação de EAF-162. Esta unidade decide quais regiões solicitar, manter ou propor retirar. A saída é um plano, não uma afirmação de que a física/carregamento foram executados. Nunca transformar um flag de presença em substituto de unload real.

## Decisão de arquitetura

Separar mundo lógico, planejamento de residência, preparação de conteúdo e aplicação no runtime. Um manifesto contém identidade do mundo, seed, gerador/versão, versão do espaço e dimensão de região. Não enumera todo o mundo. Regiões são endereços XYZ, independentes de `Scene`; interesses vêm de observadores, gameplay ou retenções necessárias. O planejador não escolhe destino nem concede percepção a NPCs: políticas de interesse futuras devem consumir apenas objetivos/evidências autorizados.

Um único Scene global residente limita extensão; um Engine independente por chunk quebra a interação física entre vizinhos e cria relógios/autoridades concorrentes. A direção de integração é um coordenador de mundo com conjuntos físicos locais: regiões adjacentes que interagem compartilham a simulação necessária; observadores distantes podem ter conjuntos locais distintos. Particionar regiões não pode excluir participantes de um combate que exige escala1000.

O carregador prepara dados fora da publicação e só confirma presença quando colisão, navegação, entidades e apresentação estão prontos. A retirada é transacional e revalida dependências. Manifesto/revisão e tokens do runtime precisam impedir publicação tardia de jobs de uma versão anterior; este planejador puro não emite tokens de autoridade.

## Contrato deste incremento

- Manifesto próprio versionado; id e gerador abstratos, seed u64 exata, `world_space` versão1. Região mede uma potência de dois entre1 e1048576 METROS por eixo; pode subdividir ou agregar células de endereço1024m. Key guarda origem global como celli64 + offset_metersu16 canônico/alinhado, preservando todo o domínio mesmo para regiões64/256m; um índice regionali64 único reduziria indevidamente o alcance ao subdividir. O valor não fixa granularidade de malha/LOD/colisor/nav.
- Interesses até1024, cada um com posição global, prioridade, raio de solicitação e raio de retenção por eixo. Retenção maior implementa histerese. Interesses equivalentes são agrupados antes das sondagens, incluindo o caso1000observadores na mesma região.
- Estados observados são solicitada/preparada/residente/retirando. São entradas declaradas pelo chamador, não leitura do Engine. Todos ocupam capacidade até a observação de retirada efetiva remover a linha.
- Retenções declaradas por controle, ator, entrega, origem, contato ou estado não salvo impedem intenção de retirada. O adapter futuro deve extrair/revalidar essas dependências do estado real; fornecer uma lista vazia a uma consulta não autoriza descartar atores. Pin significa dependência que exige residência: uma origem histórica já congelada pode sobreviver fora da região, sem manter terreno inteiro carregado por mera referência.
- Planejar com prioridade, distância entre regiões e desempates estáveis. Pedidos existentes não duplicam; região retirando não recebe um novo load concorrente. Unloads propostos não liberam vaga antecipadamente.
- Seed de conteúdo deriva de worldid/seed/gerador/revisão do gerador/dimensão/key; não consome RNG global nem muda por ordem de pedido ou ajuste de orçamento. Seed/hash não é identidade de região.
- Tetos:4096observações,1024interesses/novos pedidos,65536sondagens por consulta. Validar orçamento após agrupar interesses e antes de enumerar. Domínio numérico na borda gera diagnóstico sem wrap. Não há um número global máximo de regiões existentes.
- `plan_world_regions` no dispatcher/MCP é somente leitura. Não cria/destrói corpos, não simula atores fora de região e não altera o documento, relógio, posse ou replay. Sua saída são intenções para a próxima camada.

## Aplicação seguinte no runtime

1. Identidade lógica estável independente de slots residentes; unload distinto de morte/despawn, referências antigas não retargetam novo ocupante.
2. Preparação versionada de conteúdo, nav e física, com reserva de memória e commit em fase definida do tick. Região só passa a residente após commit, rollback de falha preserva mundo.
3. Dependências extraídas de controles/atores/casts/projéteis/efeitos/origens/contatos e alterações ainda não persistidas; definir expiração e transferência de leases.
4. Duas regiões adjacentes na mesma simulação física, travessia sem reset de indivíduo; depois unload/reload preservando recursos, vida, memória, timers e progressão.
5. Streaming de recursos/render/rede, navegação entre regiões e política explícita de ator distante. Retomar ator suspenso não pode ser apresentado como viagem física executada.

As frentes apresentação160/save161 ainda integram física/checkpoint. Este incremento evita tocar esses arquivos; o adapter físico será implementado após conciliar suas interfaces, sem fingir que o planejador já instalou conteúdo.

## Revisão estática antes do código

1. Dados tipados/células inteiras; JSON só na borda de consulta, CBOR conserva i64/u64.
2. Agrupar interesses equivalentes antes de estimar trabalho; rejeitar excedente antes de enumerar.
3. Candidatos Copy e labels por referência durante seleção; clonar nomes só nos pedidos emitidos.
4. Memória proporcional a dados residentes e vizinhanças solicitadas, nunca ao tamanho do mundo lógico.
5. Diferenças/origens i128 com conversão verificada; nada de float global ou saturação silenciosa.
6. Plano ordenado/determinístico; RNG local por região, sem ordem da árvore como regra implícita.
7. Capacidade inclui jobs pendentes/retirando, e proposta de unload não equivale a memória já liberada.
8. O(known × grupos) para retenção e sort de candidatos são custos explícitos; consulta não está no tick hoje. Cache/índice futuro só quando integração exigir.
9. Sem worker/lock/GPU/rede/dependência nova nem alteração Scene/replay por uma consulta pura.
10. Uma prova crítica cobre formato, ordenação, domínio, retenção e admissão; MCP sem janela e validação proporcional, preservando frentes paralelas.


## Implementação e revisão depois do código

- `world_regions::plan` é puro e recebe uma `Query` por valor. `PlanWorldRegions` usa Box na fronteira do enum Command, sem aumentar todo comando pelo tamanho de listas/manifesto.
- Manifesto/grid validados antes da matemática; regiões observadas devem ter origem canônica/alinhada. Vizinhos pequenos usam carry inteiro com i128 na soma de células, sem float absoluto ou índices globais multiplicados que percam alcance.
- Endereços e seeds em JSON usam decimais canônicos; CBOR usa inteiros. FNV1a com domínio/versionamento deriva seed, não concede identidade ou autoridade. Prefixo do manifesto calculado uma vez, região acrescenta apenas suas coordenadas inteiras.
- Interesses por referência e candidatos Copy; labels clonados somente nos pedidos emitidos. Pins conferidos com máscara6bits, sem alocar um set por região. Dedupe antes da estimativa evita1000observadores equivalentes gerarem1000vezes a mesma vizinhança.
- Domínio numérico dos vizinhos é conferido por inteiros, sem construir Position ou erros alocados a cada sonda. Posição de origem só é materializada para pedidos emitidos.
- Trabalho limitado antes de enumerar; Vec de saída reservado até orçamento de pedidos. BTreeMaps para grupos/candidatos e sort final são custos explícitos de consulta; retenção ainda O(observed × grupos). Sem cache especulativo ou execução por frame adicionados.
- Todas as fases ocupam capacidade até retirada observada; unload/cancel são candidatos, nunca memória já liberada. Histerese e pins mantêm regiões; retiring não cria segundo pedido. O runtime futuro deve abortar/revalidar retirada conforme novos interesses antes do commit.
- Sem estado persistido no Engine, novo clock, GPU, worker, rede, I/O ou formato de Scene/replay por esta unidade. IDs do manifesto são metadados opacos, não caminhos de arquivo.
- A primeira implementação só agregava células; a revisão corrigiu isso antes de estabilizar o contrato e ampliou a mesma prova crítica para regiões64m e fronteiras de célula.
- Uma prova crítica Rust isolada e Clippy driver `-Dwarnings` aprovados no código efetivo, sem recompilar o motor enquanto as outras frentes usam o target. `tests/world_regions.rs` importa os módulos reais por path; também é alvo normal de Cargo para as regressões futuras. A mutação nula do Engine é verificada no cenário MCP real.


## Evidência final

Uma prova crítica Rust isolada do código real passou (`rustc --test -O tests/world_regions.rs`, com as dependências já compiladas do projeto). Ela cobre ordenação/prioridade, capacidade ainda ocupada, cancel/unload propostos, pins, histerese, regiões64m, fronteira de célula, i64MIN, alinhamento/duplicatas,1000interesses→9sondagens, seed golden independente em Python e roundtripJSON/CBOR. Logs `artifacts/world-regions-test-build.log` e `artifacts/world-regions-test.log`; Clippy driver da mesma prova/módulos `-Dwarnings` aprovado em `artifacts/world-regions-clippy.log`. Compilação e lint isolados evitaram recompilar o motor durante os gates paralelos; o arquivo também é alvo Cargo para futuras regressões.

`cargo build --locked --release --bin ignis` aprovado no fonte conjunto, sem nova mudança funcional do motor por este incremento. Cenário MCP `scripts/verify_world_regions.py` passou2grupos no build `9191318b7f2d66f8`, sem janela/GPU/áudio. Usa observações e interesses sintéticos; não cria1000jogadores nem medeFPS. Confirma plano determinístico, endereços acima2^53, retenção/capacidade e snapshot do Engine inalterado após consultas e erro de budget. Relatório `artifacts/world-regions-verification.json`, log `artifacts/world-regions-mcp.log`. Fmt/Python/diff dos arquivos próprios aprovados.

Para repetir isoladamente: `python3 scripts/verify_world_regions.py`. Cenário registrado no runner para execução futura com `python3 scripts/run_verifications.py --only world_regions`. O manifesto não está anexado ao Engine/save, pois ainda não existe residência física ligada ao mundo lógico. Não há novas versões de Scene/replay por esta consulta, embora frentes paralelas tenham evoluído suas próprias versões.
