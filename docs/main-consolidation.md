# Consolidação da Ignis na main — EAF-128

Direção de Guilherme em29/10/2026: integrar frentes estáveis e desenvolver no checkout principal/main. Modly e Nanites continuam isolados. Sem push, publicação ou nova promessa de suporte de plataforma.

## Origens e fronteiras

- main eee9983: controlador terrestre, rede e conectores locais (EAF-124/126/127).
- editor385bbdd sobre02b1c2d: união prévia das ferramentas nativas, editor e autoria, mais seleção por retângulo/visibilidade/ícones e recibo de revisão final. Fonte congelado importado por patch/suplemento, SHA verificados; documentos de validação da origem preservados como evidência isolada.
- native-meshes460a407: materiais, ambiente, normais, Sweep por curva e relevo corrigido.
- native-plantsc5f2419: perfis/plantas, chanfros, transformação de ilhas e lanterna.
- multiplayer-runtimec1e3b13: controles independentes, writes locais e contribuições indexadas. Não implementa rede ou capacidade1000.
- file-browser1304a3e: delta estável adicional sobre02b1c2d, revisado na integração central; biblioteca/pastas/navegação via MCP/UI, sem novo formato de cena.
- site já está na main; multiplataforma foi discussão, sem código pendente.

O editor herdou estudos graphics-research8a5e664. Para não trazer o experimento nem sua ancestralidade para main, integrar o snapshot por squash e excluir os244arquivos introduzidos exclusivamente naquela frente, listados em integration-excluded-research.txt. As refs e a worktree de pesquisa permanecem intactas. Não usar glob de probes: os probes do editor e estudos da casa são validação estável. Documentos compartilhados podem conter registros históricos de pesquisa; não significam recursos incorporados. Remover OFFLINE_CHECKS do runner junto dos scripts excluídos e manter igualdade estrita entre cenários registrados e scripts presentes.

## Contratos da união

Scene86/replay90 substituem85/89 da main e83/87 da união anterior do editor. Gates históricos de cada campo permanecem: controles77, controlador82, rede terrestre84, conectores85, ambiente76 e cunha75. O número corrente não flexibiliza validação de arquivos antigos. Usar constantes Rust para asserts de versão corrente; fixtures históricas permanecem explícitas.

Não sobrescrever cenas da casa por uma das origens: usar a composição já resolvida pelo editor, que reúne pedra/piso, lanterna, amostragem e relevos corrigidos. Preservar limites de slots/geometria, IDs, índices de contribuições, controle por ator e separação documento/simulação. Replay compara controles, controladores, mobilidade e poses.

## Revisão estática antes da integração

1. Frequência: resolver contratos/fixtures na autoria; sem nova etapa de frame/tick.
2. Alocações: preservar caches e buffers das frentes; não adicionar clones do mundo para conciliar APIs.
3. Dados: conservar estruturas e precisões existentes; nenhuma ABI/ECS/dependência nova além da UI já aprovada.
4. Repetição: não duplicar os incrementos já incorporados no editor nem aplicar o deslocamento da lanterna duas vezes.
5. Limites: conservar9atores e limites de malhas/seleção/caches; registrar orçamento da casa, sem elevar caps para esconder conflitos.
6. GPU/I/O: manter captura offscreen sob demanda; não abrir/fechar/focar demo durante regressão.
7. Sincronização: integrações centralizadas com frentes em checkpoint; dispatcher/simulação continuam sequenciais.
8. Invalidação: preservar reset/load/undo/revisões, referências de seleção e recompilação do cache terrestre.
9. Contrato: gates Rust/Clippy/fmt/build e todos os cenários MCP, inclusive render/replay/offline, no binário conjunto.
10. Custos: compilar todos os alvos estáveis e executar testes tem custo local conhecido; sem ganho de FPS/escala ou aceite artístico presumidos.

## Validação

Buildd092a8fec0ed01f0, fonte e SHA256 do executável conferidos.1139testesRust/114alvos release/all-targets, zero falhas/ignorados; Clippy-Dwarnings,fmt/build/diff aprovados.113cenáriosMCP/482grupos headless completos, incluindo render/replay e singleplayer com rede negada. Estudos lookdev3+normais4+união4=11grupos no mesmo binário. A retomada após reiniciar o Codex reaproveitou o cache e repetiu somente o gate Rust interrompido; os demais gates válidos foram preservados. Auditoria em artifacts/main-integration-audit.json.

A união também corrigiu o recibo de view_revision após limpar ator na seleção de região, disputa do duplo clique entre rótulos e linhas, e remoção/desafixação por aliases ou caminhos ausentes. Regressões cobrem invalidade atômica e não exclusão de arquivos.

Fonte canônica composta32596v/54728triângulos, SHA0eef576ec08e0b697ce8bc70df95c9071933afb2a3379e89eb6cb2da1c633b70. Demonstração com materiais/ambiente e transição contínua da porta32588v/54716triângulos,16assets/21instâncias e180v livres; limites não ampliados. Launcher instalado atualizado para o executável/root da main, sem abrir/fechar/focar janela. Implementação e automação não equivalem a aceite artístico, entrada física ou rede multiplayer.

Site já integrado; seus textos técnicos ainda retratam um inventário anterior e a atualização editorial é trabalho posterior. Frentes estáveis devem continuar na main, sem novos merges automáticos no editor; heartbeat antigo de integração está pausado.

## Reproduzir a casa integrada

Executar em sequência, com o binário corrente da main:

```sh
python3 scripts/study_cottage_lookdev.py
python3 scripts/study_cottage_normals.py
python3 scripts/editor_cottage_union_probe.py
```

O primeiro estudo aplica materiais e ambiente à fonte canônica composta; o segundo reautora a transição tangente da porta por Sweep compartilhado e salva a demonstração; o terceiro verifica os deltas independentes de relevo/pedra/lanterna. Resultado completo em artifacts/cottage-normals/examples/cottage-spring.scene.json e artifacts/cottage-normals/artifacts/joined-hero.png. A canônica permanece inalterada. Publicadores antigos de estudos v4/v5 são receitas históricas do respectivo commit e não devem ser usados para regenerar a fonte composta v6. Fidelidade artística, point light da lanterna e UI/entrada nativas não são concluídas pela integração.
