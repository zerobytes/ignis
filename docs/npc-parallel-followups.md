# Frentes paralelas após NPC v1 — 30/10/2026

Guilherme autorizou dois chats novos para implementar circulação aérea e apresentação completa. Ambos começam do main local b4c0cdd (Scene97/replay107/protocolo3/conteúdo1/checkpoint1), mais este contrato documental. Trabalham no checkout principal compartilhado; não criar worktree sem direção explícita. Não trocar de branch no checkout compartilhado, não sobrescrever mudanças alheias e não executar git add -A.

## Circulação aérea

Antecipação e desvio local em XYZ entre voadores sobre o executor físico, incluindo curvas, hover e transições. Modos sem manobra sustentada (pulo/planeio) exigem contrato de autoridade explícito, não deformação de trajetória após o controlador. Preservar rondas XYZ, percepção limitada, inteligência/personalidade e limites de aceleração/corpo. Esta frente tem preferência inicial de integração de movimento em physics/mobility; primeiro publicar reserva de arquivos/formatos no Kordo.

## Apresentação

Fechar IK, root motion opcional, retarget configurável entre rigs e áudio no dispositivo, com demonstração mais representativa, todos os controles MCP-first e modo headless preservado. Inventariar extensão do contrato de avatar/rig/clipes/morfologia e os requisitos futuros de corpos não humanoides/Blender sem fingir que já estão implementados. Pode avançar nos módulos de retarget/áudio/apresentação independentes enquanto a circulação integra movimento. Root motion/controller/physics só após conciliar a reserva de arquivos da circulação no Kordo.

## Coordenação obrigatória

- Ler AGENTS, docs/performance.md e tarefa própria antes de implementar. Registrar no Kordo a reserva concreta de arquivos e campos de formato; consultar a outra frente antes de editar arquivos compartilhados.
- Scene/Command/Engine/Physics/State/Replay/Checkpoint/RenderFrame/protocolo são contratos compartilhados. Não reservar o mesmo número de versão ou tratar número maior como incorporação automática da outra frente.
- Se um arquivo compartilhado estiver reservado, implementar os módulos independentes primeiro. Integrar um diff por vez, após commit e liberação documentada da reserva. Não reverter diferenças externas nem resetar estado Git.
- Não adicionar novo controle só visual. Preservar identidade/vida, custos atômicos, offline e tokens/replay/checkpoint; novos estados precisam ser persistidos/migrados/validados de forma coordenada.
- Um build corrente target/release/ignis e um workspace de verificação compartilhados. Não executar run_verifications concorrentemente: reservar uso do diretório artifacts/mcp-validation no Kordo e liberá-lo após execução. Não limpar outputs/processos em uso de outra frente. Testes sem janela/foco por padrão; não abrir demo/dispositivo físico apenas por conveniência.
- Revisão estática antes/depois; testes novos só por risco material e cobertura existente proporcional. Commits atômicos locais próprios, sem push. Verificação visual/dispositivo real separada de testes headless.

## Direção transversal

Ler [continuous-world-and-portable-saves.md](continuous-world-and-portable-saves.md). Mundo lógico contínuo é requisito; o limite da cena residente atual é transitório. Não codificar novos estados como índices globais eternos de uma única Scene. Save portátil próprio é direção a projetar; checkpoint nativo atual continua restrito. Nenhuma dessas duas capacidades passa a existir só com a abertura dos chats.


## Mundo contínuo e save portátil — frentes adicionadas

Guilherme escolheu começar mundo contínuo neste chat (EAF-162) e autorizou novo chat de save portátil (EAF-161). EAF-162 reserva src/world_space.rs, tests/world_space.rs, docs/world-space.md, script próprio e apenas os hunks QueryWorldSpace/lib/runner; não modifica física/cena/replay/renderer/checkpoint. EAF-161 cuida estado canônico/adapter/portable save e coordena com159/160 antes de alterar física/formatos. Contrato espacial em docs/world-space.md é compartilhado; não copiar DTO concorrente. A consulta inicial é stateless e não implementa streaming/rebasing. Todas as quatro frentes preservam reservas de arquivos e um build corrente.
