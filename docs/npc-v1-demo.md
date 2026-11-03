# Demonstração integrada do NPC v1

Autoria em `examples/npc-v1.scene.json`, com quatro papéis sobre o mesmo runtime e perfil mental. Os modelos são pirâmides técnicas com rig/clipes nativos; cor, orientação e deslocamento permitem acompanhar a simulação. Não são avatares humanos de produção.

| Papel | Perfil e comportamento |
|---|---|
| Guarda roxo | Inteligência 0,8, cautela/coragem/agressão/disciplina; percepção, perseguição tática e projétil. Só usa pistas aceitas. |
| Civil laranja | Inteligência 0,6, medo 0,9, coragem 0,1; recarrega necessidade no poço, cumpre agenda no santuário e prefere a opção segura do diálogo. |
| Aliado azul | Inteligência 0,9, preocupação 1 e sociabilidade 0,9; escolhe aliado ferido, reserva cura e completa cast de 90 ticks. Pode conversar com o jogador. |
| Criatura verde | Inteligência 0,7, curiosidade/coragem; alterna capacidades terrestres/aéreas sobre rede gerada e voa curvado até destino XYZ. |

O jogador amarelo começa ferido. Há uma porta fixa mutável entre ele e a cena. O cenário autoral começa pausado e com a porta fechada. A prova integrada abre a porta, conversa com o aliado e emite ruído **pelo MCP** antes de avançar. Apenas iniciar a janela não executa esse preparo automaticamente.

## Repetir a prova sem janela

```sh
python3 scripts/run_verifications.py --only npc_checkpoint
```

O script `scripts/verify_npc_checkpoint.py` usa o mesmo dispatcher público:

1. `load_scene`, `save_scene`, `bind_control` do jogador e `talk_npc` com `ally`.
2. `interact {"actor":"player","point":"door","action":"open"}` e emissão de ruído posicional. Salvar com pedidos pendentes é recusado.
3. `step` de 60 ticks: cura/reserva e voo em andamento. `save_checkpoint` e captura offscreen.
4. Avançar mais, `load_checkpoint`: mesmas posições/NPCs/rotinas/props/controladores/poses. Tokens antigos recusados; `inspect_dialogue` fornece sessão nova para continuar.
5. `step` de 240 ticks: civil com `charge=100`, `safe=1`, `bold=0`, jogador curado. Novo replay reproduz estado; prefixo 0 e captura do checkpoint têm pixels idênticos.
6. Checksum corrompido preserva mundo; continuous salva/restaura com histórico zero; load com host ativo é recusado.

O arquivo salvo usa extensão `.ignis-save`. Ver [contrato do checkpoint](npc-checkpoints.md) antes de tratar esse formato nativo como save portátil. Perfis e demais parâmetros são autoria normal, editáveis via `upsert_entity`/`edit_scene`, sem controle exclusivamente visual.

## Acompanhar manualmente

Quando desejar abrir a demonstração:

```sh
./target/release/ignis --scene examples/npc-v1.scene.json --window
```

`P` inicia/pausa, `WASD` move o jogador e `R` reinicia a autoria. As operações de interação/diálogo/checkpoint acima já existem no MCP; não foi adicionada uma tecla visual exclusiva. Para reproduzir exatamente o preparo, use o MCP no mesmo processo da janela. Processos separados não compartilham sessão.

A verificação automática usa GPU offscreen e não comprova foco/entrada física, experiência artística ou aceite humano. Ela também não mede o custo de renderizar centenas de avatares. Carga independente de 100 NPCs + 500 jogadores, com limites e tempos, em [npc-population-scale.md](npc-population-scale.md).
