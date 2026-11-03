# Ressurreição do mesmo indivíduo — EAF-58

Recorte aprovado após [o estudo de ciclo de vida](lifecycle-study.md): voltar à vida no corpo existente, preservando energia e cooldowns. Implementação em cena/replay46, executor compartilhado entre jogador/NPC e autoria/acionamento/inspeção pelo MCP. Spawn/despawn, teleporte, novos indivíduos e triagem automática de cadáveres permanecem posteriores.

## Contrato autorado

Uma habilidade `selection: actor`, entrega `direct`, contém exatamente um efeito:

```json
{"kind":"resurrect","recipient":"target","amount":40}
```

`amount` é inteiro positivo até10^12, nas unidades de vitalidade do destinatário, limitado ao máximo. O alvo precisa ter vitalidade e estar morto no início e no commit; cura normal continua rejeitando mortos. Não há autorressurreição por um morto, pois ele não pode iniciar habilidades.

Pode compor efeitos `resource` no **source** e `noise`, dentro do orçamento existente. Outros efeitos no alvo, ressurreição duplicada, área, projétil, canal e agenda periódica são rejeitados antes de alterar a cena. Essa restrição evita ambiguidades de vida intermediária e restauração implícita de recursos; não limita a composição futura a um enum de classes/espécies.

Custo/cast/cooldown/cancelamento e movimento são os existentes. O curador paga apenas no commit bem-sucedido. Dois curadores obedecem à ordem de fases/IDs: uma ressurreição pode vencer e o outro plano falha sem cobrança. Fonte sem saldo, alvo vivo, falta de vitalidade, perda de evidência ou referência de vida obsoleta não pagam.

A energia, demais recursos, cooldowns e retry deadlines do **ressuscitado** são preservados. Casts cancelados pela morte não são retomados. Buffs/periódicos encerrados na morte não voltam. O documento autorado não muda e `reset` continua restaurando toda a cena, operação distinta de ressurreição.

## Identidade e transações

Catálogo/slots permanecem fixos por sessão. Cada ator recebe `life_index`, começando em0 e incrementado no retorno; overflow é rejeitado. Isso distingue vidas do mesmo indivíduo, sem introduzir reciclagem de slots ou identidade dinâmica de população.

Pedidos enfileirados e casts têm `life_stamp` do autor e destinatário. Intenções automáticas usam o snapshot sensorial anterior às transações da fase. Revalidação impede que um pedido preparado para a vida antiga passe a agir sobre a nova. Um cast manual antigo pode permanecer visível até seu prazo de revalidação, mas não conclui nem cobra sobre outra vida. Canalização revalida a cada passo. Cancelamento por ID continua sendo controle explícito de uma execução.

Transição letal mantém limpeza de buffs e cancelamentos imediatos. Periódicos vinculados a outra vida são descartados; áreas configuradas para terminar pela morte não sobrevivem ao retorno no mesmo tick. Máscaras de morte/retorno garantem que o físico consuma a transição mesmo quando a vida final do tick é positiva. Anéis separados e limitados a256 registram mortes e retornos com tick, autor, execução e vida, sem apagar o histórico ao ressuscitar.

## Corpo, movimento e conhecimento

Mesmo collider/rigid body, posição, gravidade/inércia e capacidades. Sem teste de ocupação para um teleporte que não acontece: o corpo já ocupa seu lugar. Um voador morto cai; voltar à vida rearma seu motor existente e permite replanejar, sem conceder voo a um caminhante.

Percepção/objetivos transitórios próprios reiniciam; navegação/recuperação perdem caminho antigo, motor espacial perde compromisso, patrulha reinicia seu cursor e reservas de passagem são liberadas. A morte seguinte volta a desativar o ator. Saldo/prazos de habilidades não passam por essa reconstrução. Memória social persistente ainda não existe e não deve ser confundida com esse estado transitório.

Observadores não recebem um aviso global. Corpos conhecidos mortos voltam a poder ser sondados visualmente; reconhecimento acumulado recomeça ao encontrar uma nova vida. Só confirmação visual/omnisciência explícita atualiza a crença. Sem visão nova, memória/budgets permanecem iguais sob as mesmas evidências, independentemente de um retorno oculto.

NPC pode ressuscitar o **alvo explicitamente configurado**, mediante observação de cadáver confirmada no tick. Essa evidência aparece em `corpse_visual`, separada da memória de perseguição. Use objetivo `use_ability` sem `policy`, com `when: always` e retry; políticas de triagem/aproximação atuais são de vivos e são rejeitadas para essa habilidade. Sem aproximação automática a cadáveres neste recorte. Controle externo via MCP também respeita a evidência do NPC.

Relatos carregam a vida observada, conservam TTL e não autorizam habilidades por si mesmos. Um relato de vida anterior à já confirmada é rejeitado; o conhecimento de morte não é limpo por um relato velho. Som permanece anônimo. A inspeção global é diagnóstico e não fornece conhecimento adicional ao NPC.

## Entregas que já estão no mundo

Projéteis pagos continuam voando. Uma **nova colisão física** pode atingir um ressuscitado; o alvo da colisão recebe a vida corrente. Isso difere de revalidar um cast direcionado antigo. Zonas ainda existentes consultam ocupantes atuais a cada pulso e também podem afetá-lo.

Auras com `end_on_source_death:false` permanecem ligadas ao mesmo corpo, inclusive durante a morte e após o retorno. Com true, a transição letal encerra a área mesmo havendo retorno no mesmo tick. Autoria original é preservada; fatores de dano continuam dinâmicos no impacto/pulso, portanto novos buffs do mesmo indivíduo podem afetar entregas antigas ainda ativas. Congelar esses fatores no lançamento seria outro contrato.

## MCP, replay e demo

Operações existentes `set_ability_catalog`, `edit_scene`, `activate_ability`, `cancel_ability`, `inspect_vitality`, `inspect_abilities` e `inspect_npcs` expõem todo o fluxo. Esquema lista `resurrect`; recibos distinguem `target_alive`, `life_changed`, `life_budget` e falhas existentes. Vitalidade mostra `life_index` e `revivals`; casts mostram stamps, percepção mostra vida observada e cadáver confirmado.

Cena `examples/npc-resurrection.scene.json`, produzida por `scripts/verify_resurrection.py` via MCP:

```sh
./target/release/ignis --scene examples/npc-resurrection.scene.json --window
```

- **P** inicia; **WASD** move; clique seleciona; **J** barras; **X** cancela seu cast; **R** reinicia toda a cena.
- **1** causa100 de dano ao selecionado; **2** ressuscita com40 de vida; **3** cura25 em vivo; **4** emite um sinal próprio e gasta17 de energia.
- Roxo é o paciente; clique nele para selecionar. Emite sinal ao iniciar, gasta17 de energia e entra em cooldown de600ticks/10s. Verde tem o roxo como alvo explícito e ressuscita com preparação120ticks/2s, alcance8m, custo5 e cooldown180ticks/3s. Retry de30ticks pode acrescentar até0,5s antes de iniciar a preparação. Matar o roxo durante esse cooldown pode fazê-lo aguardar.
- Azul observa o roxo, sem atacá-lo. Cinza começa morto atrás da parede. Verde não troca automaticamente seu alvo por cinza; o jogador pode selecionar cinza, aproximar-se com linha livre e usar2.
- O sinal do paciente não reproduz áudio; apenas exercita recurso/cooldown. A habilidade não cura dano ocorrido depois da ressurreição.

O cenário automatizado roda sem janela, valida estado/cobrança/edição/replay e compara capturas GPU com prefixos reconstruídos. Teclado nativo e aceite humano são gates separados. Evidências finais em [validation.md](validation.md).
