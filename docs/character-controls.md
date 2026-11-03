# Corpo ampliado, postura e edição facial — EAF-187

Incremento atual: [lábios, bases adultas e pivôs de cabeça/pescoço](character-refinement.md).
Fonte3 lê1/2; o contrato abaixo registra o marco anterior.

Incremento solicitado após teste humano: extremos maiores de gordura e músculos,
cintura independente da caixa torácica, postura ajustável e pontos faciais3D.
As imagens são referências artísticas de alinhamento; suas afirmações médicas
não fazem parte do contrato do editor.

## Fonte e reprodução

`scripts/character/extend_source.py` deriva `female-extended.ignis-character`
da fonte1 existente. A anatomia original, o rig e os alvos continuam identificados
como dados gráficos CC0 MakeHuman na revisão fixada em character-base.md.
Treze alvos regionais adicionais (membros, dorsais, peitorais e tônus abdominal)
foram incorporados com hashes no mesmo provenance.json. Nenhum código MakeHuman
foi importado. Campos faciais, hipertrofia radial, tecido abdominal/flancos,
cintura, máscaras posturais e montagem são código/dados autorais deste projeto.

Reprodução offline, depois de gerar a fonte1:

```sh
python3 scripts/character/prepare_source.py
python3 scripts/character/extend_source.py
```

`extend_source.py --fetch` adquire explicitamente os13arquivos gráficos da
revisão fixada e registra/verifica hashes; não existe aquisição no runtime.
Fonte1 permanece em `female.ignis-character` para cenas/undo legados.
Fonte2 usa arquivo próprio e155parâmetros:29gerais e126canais de42pontosXYZ.
São306alvos e4warps de repouso. Todos os novos valores default são zero, exceto
os21valores herdados; ligar a nova fonte a uma configuração antiga conserva
bit a bit o bake, comprovado por fingerprint antes de atualizar o vínculo.

## Corpo e postura

- Gordura e musculatura aceitam0..2. O domínio0..1 preserva a forma anterior;
  além de1 entram novos alvos, sem apenas aumentar o número do slider.
- Gordura acrescenta abdômen, flancos, costas e pescoço à extensão condicionada
  por músculo dos alvos originais. O coeficiente não é percentual médico.
- Musculatura acrescenta massa regional e hipertrofia suave por eixo de membro,
  com atenuação em tendões/juntas. Definição muscular é um controle separado,
  com relevo geométrico abdominal e alvos regionais, atenuado pela gordura.
- Cintura altera largura/profundidade do abdômen numa faixa própria, com apoio
  nulo acima da caixa torácica inferior. Não reutiliza largura do tronco.
- Postura separa pelve, lombar, tórax, inclinação da cabeça, recuo da cabeça e
  ombros. Os presets Base/Curvada/Natural/Ereta/Heroica configuram esses mesmos
  valores; a inclinação da cabeça compensa o tórax nos presets eretos.

Warps rotacionam posições com máscaras graduais e pivôs de âncoras já deformadas;
braços acompanham a região superior e os pés permanecem fora das máscaras.
Depois de todos os alvos/warps o compilador ajusta novamente o rig/binds. Isso
edita o repouso autoral, não é animação de postura ou IK de apoio dinâmico.

## Rosto3D

42pontos abrangem testa superior/central/lateral, glabela, têmporas,
sobrancelhas interna/arco/cauda, órbitas, pálpebras/cantos dos olhos,
ponte/ponta/asas do nariz, maçãs, bochechas, ângulos da mandíbula,
queixo, lábios/cantos da boca, parte superior/lóbulos das orelhas.

No painel **Personagem**, **Editar rosto em3D** mostra pontos sobre a superfície
visível; **Enquadrar rosto** aproxima a câmera. Arraste o ponto no plano da tela
ou uma setaX/Y/Z para restringir o eixo. O eixo de profundidade continua acessível
quando alinhado à câmera por uma haste rotulada. Botão direito orbita a câmera.
O seletor de pontos e camposXYZ em centímetros dão acesso numérico equivalente.

Cada ponto tem limites próprios em metros locais, campo suave e âncora em
vértice real. Simetria refleteX e conservaY/Z no par; pontos da linha média
mantêmX=0 com simetria ativa. Desligá-la permite deslocamento lateral e
assimetria. Órbitas movem também as âncoras dos olhos: esferas/íris/pupilas
seguem o rig ajustado. Pontos e gizmos usam a geometria/câmera correntes e a
mesma projeção/consulta de profundidade do editor; seleção permanece destacada.

Durante o arraste há apenas uma Scene candidata, sem alterar documento/histórico.
Soltar publica uma transação; Esc/foco/câmera/layout/HiDPI/seleção/referência antiga
cancelam a prévia e consomem sua liberação. Guardas incluem scope+revision,
revisões de sessão/editor/visão, câmera e retângulo físico da viewport.

## API, persistência e limites

`inspect_character_editor` expõe pontos, posições/bases mundiais, offsets/limites
locais e `reference`. `set_character_handle_view {view:{enabled,symmetry,selected}}`
controla apenas a sessão. `set_character_handle {prefix,handle,offset,symmetry,
reference,gesture?}` usa metros locais e o mesmo kernel da viewport. A prévia
MCP é `preview_character_handle` com os mesmos campos semgesture, retornando
binding/pontos candidatos e `world_changed:false`. Entradas fora dos limites,
referências antigas e alterações manuais não publicam nada.

Offsets ficam nos canais escalares da configuração, junto de cintura/postura/
extremos. Save/undo/replay/receita exportada usam o vínculo existente. Scene107,
replay117, conteúdo7/protocolo5/PSV1/checkpoint3 permanecem; fonte2 é o novo
contrato de autoria, com leitura1 preservada. Requer engine com fonte2 para editar.
O jogo continua usando o bake, sem precisar da fonte ou da UI do editor.

Fonte2 permite até256parâmetros,64pontos e16warps. Mantém64MiB,100mil vértices,
512alvos,4milhões de deltas+pesos de warp,32partes e256ossos/16influências.
Campos opcionais/curvas assinadas são recusados sob fonte1. A receita aceita
até256canais e mantém o envelope64KiB. Objetos107 continuam externos ao
compilador: nenhuma associação de partes é apagada/redefinida por um gizmo.

Limites são por ponto, não uma certificação de ausência de auto-interseção em
todas as combinações. Extremos caricatos e misturas exigem revisão artística.
Pele/olhos/texturas, correções de pose, expressões coordenadas, dentes/língua,
tecidos/bounce e Playeravatar continuam os marcos separados já registrados.

## Custo e validação

Fonte validada e geometria de referência ficam em cache; sem parse/I/O por frame.
Posições/bases dos pontos são derivadas do bake por revisão, fora do frame estável.
Aceleração de profundidade é reconstruída por revisão/tick e compartilhada nas
consultas dos pontos, não por câmera. Prévia recompila apenas quando o ponteiro
muda; guarda uma candidata e descarta a anterior. Bakes/rigs usam os Arcs e o
undo32 já existentes. Compilação/validação/hash são custos síncronos de edição;
não há deformação GPU nova nem promessa de FPS numa cena mundial pesada.

`scripts/character/controls_demo.py --capture --all-handles` verifica a fonte
real e atualiza evidências em artifacts/character-controls, sem salvar sobre a
cena humana. Testes críticos cobrem formato1/2, rig ajustado, limites/simetria,
referência antiga, preview sem commit, cancelamento/HiDPI e um undo por gesto.
Comparações usam volumes geométricos aproximados, não composição corporal.
