# Composição de peças da casa — EAF-125

## Recorte e checklist anterior ao cliente

A lanterna na parede direita da entrada estava parcialmente encoberta pela
coluna do pórtico na câmera hero. A família bronze também contém porta, frisos
e janelas, enquanto flowers contém o miolo da lanterna e flores do jardim.
Reconstruir ambas para deslocar um acessório é desnecessário. Extrair gastaria
slots indisponíveis. Aplicamos a transformação nativa de ilhas por seleção
inspecionada, com dois assets num único lote/undo.

1. Só autoria/captura explícita, nenhum tick novo.
2. Consultar bounds/IDs nativos; não enviar arrays substitutos de vértices.
3. Deslocamento local explícito−0.27m emX; rotação identidade/escala1.
4. Duas consultas e um lote, sem mover cada vértice em chamadas individuais.
5. Esperar9ilhas bronze/1flowers; abortar se a composição fonte mudar.
6. Sem novos assets/instâncias/triângulos; domínio implícito pode virar explícito.
7. Mesma engine headless, sem janelas ou processos externos alterados.
8. Preview puro e igualdade com commit; camera restaurada depois dos recortes.
9. Validar só vértices selecionados alterados, outros dados exatos, undo/load/replay.
10. Seleção semântica ainda pertence à receita; operação geométrica é nativa/genérica.

## Reprodução

Receita6 conserva toda construção da5 e acrescenta
`entrance.lantern_offset:[-0.27,0,0]`. `demo_elven_cottage.py` constrói as mesmas
famílias e aplica `cottage_composition.adjust_lantern` depois de instanciá-las.
`cottage_composition.py` escolhe peças completas através de
`inspect_mesh_islands`, exigindo as contagens previstas e usando a referência
retornada em `preview_scene_edits`/`edit_scene`.

`study_cottage_composition.py` parte da cena publicada anterior, salva em
`artifacts/cottage-composition/examples/before.scene.json`, aplica o mesmo lote
nativo e prova a alteração exata dos membros da seleção. A travessia de faces no
cliente só verifica o resultado; não calcula/publica geometria substituta.
Caminhos estáveis de PNGs antes/depois: `artifacts/cottage-composition/artifacts/`
com vistas hero e lantern. `report.json` registra seleção/build/contagens/checks.

Na primeira execução o estudo preserva a cena canônica anterior; execuções
seguintes reutilizam essa origem ou recebem `--before <cena>`. A evidência de
origem é a cena do commitf608818. `--after <cena>` também verifica igualdade
semântica e pixels da publicação regenerada. Fontes de árvore e referência visual
permanecem intactas. Não há novo material/luz, colisão ou interior; fidelidade
artística da casa é avaliada separadamente.

## Resultado e revisão final

Origemf608818 SHA256`88b5a8410e62d5db58a2db0f5b00da27b624322a000514f73ee2056b98fb823b`.
Publicação atual SHA256`d189be9c54fd0e514003f992b8f27b6a51a414551e9c69f85b7349f73074bd0b`,
2369486bytes compactos, mesma Scene72. Build`c9ffc6d3852bd23d`.

1. Adequado: novo helper só após construção/autoria; nenhum tick/frame novo.
2. Adequado: nove ilhas bronze132v e uma flowers8v movidas pelo comando nativo,
   sem reconstrução/envio dos5484v bronze ou608v flowers no cliente.
3. Adequado: bounds da inspeção e referência nativa; escala1 e deslocamento−.27X.
4. Adequado: preview/lote únicos, duas famílias editadas sob uma referência.
5. Adequado:16assets/21instâncias/32596v/54728t armazenados/55948t desenhados,
   exatamente os mesmos valores da origem;172v livres, nenhum slot livre.
6. Mantido com motivo: flowers materializa608coordenadas (7296bytes de payload
   f32); bronze já tinha domínio explícito. Valores amostrados acompanham peças.
7. Adequado: processos headless isolados; nenhuma janela/main/demo tocada.
8. Adequado: outras14famílias exatas; nos dois assets só140posições mudam,
   índices/contagens/rig/clips/material/shading preservados e flores não movidas
   mantêm posições/domínio. Cena/restantes instâncias/plantas/câmera/luz iguais.
9. Aprovado:9grupos do estudo incluindo publicação,16checks da autoria completa,
   preview/commit, undo/redo único, save/load/reset/tick60/replay/prefixo/pixels.
   Reconstrução desde receita6 coincide semanticamente e em pixels com a edição
   da origem pelo comando nativo. Auditoria de fingerprint/report/build aprovada;
  70summaries MCP identificam o build atual,18legados omitem esse campo.
10. Mantido com motivo: apenas reposicionamento, sem redução do orçamento nem
    nomes persistentes de subpeças. A lanterna ainda tem miolo opaco simplificado,
    sem luz pontual/vidro. A captura melhora sua visibilidade, não atesta fidelidade
    artística geral ou suporte físico de montagem.

Evidência: `artifacts/cottage-composition/artifacts/{report,build-audit}.json`,
`artifacts/cottage-composition-publication.log`, `artifacts/cottage-composition-authoring.log`
e `artifacts/elven-cottage/artifacts/report.json`. Capturas reais antes/depois em
`before-lantern.png`/`after-lantern.png` e `before-hero.png`/`after-hero.png`.
Árvore canônica SHA256`89fe1fa2112cbb8a2d3ae266a53cf0d6308c8b371d356a8f5b1b05a208789400`
preservada. Fonte da referência preservada; nenhum conteúdo novo via imagem gerada.
