# Apoio para nascimento — EAF-111

Extensão EAF-115: cena75/replay79 acrescentam cunhas fixas/visuais. A busca encontra a altura real da face inclinada, mas retorna sloped_surface (sem nascimento nem procura do andar inferior). Apoio de nascimento continua plano; superfícies de navegação/passos também não podem referenciar cunha. [Geometria](geometry.md).

Contrato do incremento autorizado: cena66/replay70. `query_spawn_support` recebe template, `position` e `max_drop`; `support_projection:{max_drop}` é opcional no gerador. Ausência mantém XYZ exato. No modo projetado, points/region sorteiam **origens da busca dos pés**, não centros finais: X/Z ficam iguais, Y é o teto da busca vertical. A posição final pode sair dos limites Y da região por depender do piso e da altura do template. Nenhum draw adicional.

Escolher a maior face superior fixa sob o centro XZ, no intervalo inclusivo [Y−max_drop,Y]; empate por ID. Sem fallback para andar inferior se o primeiro piso não couber. Reusar apoio modular da pegada inteira e overlap com corpos presentes em suas posições físicas atuais. Sem caminho varrido entre origem e resultado: é consulta de colocação, não queda, teleporte em jogo ou navegação. Superfícies visuais/dinâmicas não sustentam esta política. Distância finita positiva até20000; origem e centro resolvido dentro de±10000.

Tolerâncias herdadas de walk_support:6cm entre alturas de apoio e1cm nas bordas externas da pegada; a composição não expande pisos para fechar frestas internas. Overlap conserva tolerância0.1mm da admissão spawn existente. Isso não é um novo controlador de contato nem garantia de estabilidade em geometrias arbitrárias.

Consulta somente leitura: retorna status, índice/ID da superfície, pés e centro quando encontrados. Não reserva espaço, não consome seed/tick/identidade/histórico e não garante capacidade de população, alvo ou admissão futura. Gerador usa o mesmo relatório imediatamente antes da transação spawn, que conserva todas as validações de capacidade/mobilidade/alvo/passagem/identidade. Falhas de apoio mapeiam unsupported_position; overlap mapeia occupied. Recibo mantém position sorteada e support com resultado/posição resolvida; XYZ legado, retirada e recusa prévia por alvo original ausente têm support null. `inspect_generators.support_surface_ids` mapeia índice numérico ao ID fixo autoral.

| report.status | Significado |
|---|---|
| supported | Pegada apoiada e volume livre neste instante; spawn ainda pode recusar por capacidade/mobilidade/alvo/passagem. |
| no_surface | Nenhum topo fixo dentro do intervalo sob o centro; surface/feet/position null. |
| insufficient_support | O primeiro topo existe, mas a união dos pisos não cobre a pegada. |
| support_limit | Mais de64 superfícies necessárias na consulta composta; sem sucesso aproximado. |
| occupied | Centro proposto sobre apoio sobrepõe um corpo físico presente, inclusive cadáver. |
| out_of_bounds | Piso encontrado, mas o centro calculado ultrapassa±10000. |

Nos últimos quatro estados, feet/position são propostas diagnósticas, não posições aceitas. Não passar uma proposta recusada ao spawn como se a consulta tivesse sido aprovada. A consulta deliberadamente não testa um volume varrido desde a origem: permite perguntar por colocação sob um teto usando origem abaixo dele; não simula um corpo caindo através de tetos.

```json
{"op":"query_spawn_support","template":"guard","position":[2,8,0],"max_drop":10}
```

No gerador já autorado: `support_projection:{"max_drop":10}` opt-in vale para points ou region e para todos os candidatos de templates/ondas. A altura do template selecionado é usada na consulta; não há override por onda, nova seed ou fallback para template menor. Nascer em uma plataforma não altera `npc.surface`, cria rede ou concede habilidade de caminhar até outro piso. A consulta é ferramenta de autoria/geração e não uma nova fonte de conhecimento sensorial do NPC.

Busca linear limitada pelas entidades existentes, sem lista de candidatos ou clone por consulta. Não introduz geometria, controlador, navmesh, rampas, terreno irregular, projeção lateral ou escala nova. Projeto futuro de rampas/degraus continua exigindo contrato de geometria compartilhada.

## Demo e validação

`examples/npc-spawn-support.scene.json`, criada/salva/carregada por MCP: três plataformas separadas em alturas0/1.5/3m, vãos sem apoio e um apoio estreito que não comporta os corpos. Gerador regional com seed42, três templates: roxo1.6m, laranja3.2m e azul0.8m. Meta6vivos, tentativas8/ciclo a cada60ticks, orçamento18nascimentos e retirada de cadáver após120ticks. P inicia/pausa mundo; B/O/K ativam/pausam/cancelam o único gerador, R reinicia. Clique seleciona;1 dano100/2 ressuscita no alvo selecionado pelas habilidades existentes, sujeitas a alcance/oclusão. NPCs desta demo ficam parados intencionalmente para observar colocação, sem demonstrar trânsito entre plataformas. Remover/derrotar membros permite reposição enquanto houver orçamento; nenhuma projeção teleporta um indivíduo existente.

`tests/spawn_support.rs` e cenário `python3 scripts/run_verifications.py --only spawn_support`, mais regressão completa headless. Replay70 guarda e compara relatórios; prefixos não antecipam novas tentativas e capturas/reset devem coincidir. Cena66 salva configuração, não indivíduos gerados. Teste de teclado/foco e aceite visual humano separados das capturas offscreen. Resultados em validation.md.
