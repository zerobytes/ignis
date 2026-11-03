# Passeio em primeira pessoa — EAF-185

Pedido: personagem físico, visão humana e caminhada na paisagem ampliada.
Não confundir câmera panorâmica/órbita de editor com deslocamento no terreno.

## Decisão e custo

Reutilizar câmera autoral por SetCamera em cada movimento reconstruiria a
simulação. A alternativa implementada é visão local de sessão sobre RenderFrame:
consulta a posição do ator físico, soma altura dos olhos a partir dos pés e
orienta pelo yaw/pitch. Sem geração, serialização de cena, mesh ou IO por olhar.
Um vínculo Reference+vida e serial protege o ator; reset/edição/load descartam a
visão. Morte/reuso/vida diferente tornam a visão inativa. Scene/replay/checkpoint
não recebem campos novos. Somente os inputs de movimento em coordenadas de mundo
seguem o sistema de controles/replay existente. A câmera não é salva como gameplay.

O corpo próprio (primitivo/avatar) é omitido da apresentação em primeira pessoa;
colisão/corpo físico e outros atores permanecem. Não é noclip, voo ou teleporte.
Não há animação nova do boneco nem colisão nova de troncos/água.

## Interface comum

- SetPlayerView{view:{actor,yaw_degrees,pitch_degrees,eye_height,fov_degrees}}
  seleciona a visão de um Player vivo. Olhos ficam dentro da altura do corpo;
  pitch[-85,85], FOV[10,120], dados finitos. view:null restaura câmera da cena.
- InspectPlayerView retorna ativo/config/câmera/tick, sem dump da cena.
- SetPlayerViewInput{input,control?} gira movimento local XZ pelo yaw, ignora
  pitch para caminhar no chão. Com control explícito, exige mesmo Reference;
  controles de outro Player são recusados antes da mutação. Sem control usa
  legado SetInput, que já exige um único Player sem vínculo explícito.
- NativeWindow: F alterna visão, clique captura mouse, movimento do mouse olha,
  WASD/setas caminham relativos ao olhar, Shift corre, Ctrl furtivo. Esc solta
  mouse primeiro; Esc novamente fecha. P inicia/pausa; R reseta (F liga visão de
  novo). Perda de foco libera mouse/teclas e neutraliza só input ainda do teclado;
  troca de contexto limpa teclas e exige um novo pressionamento, sem OS repeat.
- C alterna recorded/continuous somente nas condições já exigidas pelo comando
  SetExecutionMode (tick0, pausado, sem controles/pedidos). Falhas são explícitas.
- CLI: --continuous e --first-person PLAYER usam os mesmos comandos antes de
  abrir a janela. Não há socket no modo offline. --editor continua com sua câmera
  autoral; a demonstração usa --window, não um viewport de autoria disfarçado.

## Região ampliada

Geração/receita/eixos e publicação pertencem a systems (docs/expanded-world.md).
A representação estática usa malha fina no miolo e espaçada fora; não é LOD de
runtime nem streaming. Vegetação deriva dos triângulos publicados com busca
binária nos eixos não uniformes. Zonas de implantação vêm do manifesto (12mil no
miolo,18mil fora no candidato20km); densidade não uniforme é explícita.
Não alegar máximo universal do hardware nem derivar FPS de triângulos poupados.

## Validação

Duas provas críticas novas em tests/player_view.rs protegem movimento físico e
câmera, autoria intacta/reset, configuração inválida atômica e recusa de controle
alheio. Regressões de controles, execução contínua, terreno/regiões/colisão e lib
selecionadas por impacto. A captura nativa e o passeio humano são gates distintos.
Build inicial9bb5cfc0aae7b2b2 passou155Rust/7alvos e Clippyalltargets;
verificação MCP em representative-world deslocou11,944m em180ticks, grounded,
câmera ativa1,65m a partir dos pés e histórico retido zero em continuous.
Configuração/captura em artifacts/representative-world/player-view-verification.json
 e first-person.png. Isso prova o mecanismo na região anterior, não a área20km.

Duas correções de validação precederam a entrega: compilação inicial misturou
lib anterior/teste regional posterior durante a última adição de campo; foi
repetida no fonte congelado. A nova prova de controle teve asserção inválida de
serialização de RenderBox corrigida para contagem e outro ator foi colocado
fora da trajetória, para testar input alheio sem misturar contato físico.
Nenhum contrato de produção foi relaxado para passar. A publicação20km ainda
exigiu corrigir a projeção esférica do adaptador para respeitar a caixa autoral
por componente; caso diagonal e recusa de10001m pertencem ao gate regional.
Resultados finais/área/residência/abertura da janela serão registrados abaixo.


## Entrega20km e janela humana

Terreno systems em cf63fe7: docs/expanded-world.md contém reprodução e medições.
Base real20×20km (400km²,100× área da demonstração2km), quatro malhas físicas,
1136356v/2264192tri.4m no miolo fixo±1km e até32m no exterior, sem refino móvel.
Este é o maior recorte ensaiado nesta unidade dentro do frame atual±10km;
não é máximo do hardware nem extensão ilimitada da engine. Os testes físicos
cruzaramX0/Z0 e caminharam a8km do centro, todos grounded.

Floresta final: artifacts/expanded-world/world.scene.json,30mil árvores,60005
instâncias/7assets;12mil no miolo e18mil fora (densidade não uniforme). Fonte-base
SHA8915ada3fc01c6abfe373f3e84bcacacacc5473fa6fd364582e39466c81b68be
preservada. Quatro bordas reais com533v cada têm erro posição/material zero;
normal ainda é derivada por asset, não certificada entre quadrantes. Erro máximo
de raiz após save/load0,000675m; compilados399784B. Árvores/água visuais; física
usa as quatro malhas completas. Moiré e horizonte plano no panorama distante
permanecem limites; hipótese de precisão de profundidade ainda não comprovada.

Build de execução6dda7b1233983522, macOS/Metal.155Rust em7alvos passaram;
alvo regional repetido após correção diagonal,2player_view repetidos no final,
Clippyalltargets/fmt/Python/diff e cargo check do fonte final aprovados. A última
mudança de fonte após build foi apenas ordenar declaração do módulo em lib.rs,
sem alteração de comportamento. Sem medição de FPS. A base semGPU/vegetação
carregou em2,068s e~617MB RSS após640ticks; não usar como memória do jogo completo.
O pico de geração/captura4,24GB também não é residência típica do launcher.

Janela jogável realPID6348 iniciada via executável do bundle existente (symlink
para target/release/ignis), com --window/--mcp/--continuous/--first-person player.
O primeiro processo direto6258 não era selecionável pelo CUA e foi encerrado
limpo antes de abrir este; não ficaram duas demos. Nenhum executável foi copiado.
A GUI foi conferida na altura humana; tecladoP/W deslocou4,400m grounded,
posição650,229,251,400 →649,9998,229,2512,395,6000. Mouse capturado alterou
ângulos pelo caminho nativo e Esc soltou cursor sem fechar a janela. Arte/sensação
física/FPS ainda dependem do teste humano. Relatório player-window-verification.json;
captura first-person-live.png2048×1415. O endpoint de captura mantém limite2048;
a janela observada renderiza em2200×1520 físicos, sem redução de resolução oculta.
A imagem CUA da própria janela foi inspecionada separadamente.

Reabrir manualmente (P para começar; clique para olhar):

```sh
target/release/ignis --root . --scene artifacts/expanded-world/world.scene.json --window --continuous --first-person player
```

O launcher usa inicialmente o heading da câmera autoral. A sessão demonstrada
selecionou placement.player_view (yaw0/olhos1,65m/FOV75) pelo mesmo dispatcher.
A cena salva mantém panorama como câmera de autoria, não o estado efêmero do passeio.
185 permanece InProgress aguardando aceite humano; não inferir streaming, replay
ou rede desta cena grande nem FPS universal.
