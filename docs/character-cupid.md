# Vale central do arco do cupido — EAF-187

O ponto `upper_lip` existente controla uma área ampla do vermelhão superior;
`upper_tubercle` fica junto à borda interna. Nenhum deles oferecia a edição
local independente do V na borda externa pedida nos prints humanos.

`cupid_valley`, **Vale central do arco do cupido (V)**, é um novo pontoXYZ no
vértice455 do cage, entre os picos456/7215, acima do centro467 e tubérculo509.
Pertence a Lábios (agora21pontos); total65pontos/256canais/463alvos na fonte5.
O atalho **Selecionar V central do lábio** ativa o grupo/ponto/edição3D.
Seta azul recua/projeta a região central; a verde muda a altura do V. Os picos
laterais continuam independentes. X permite assimetria quando Simetria está
desligada; com simetria ligada, o ponto central permanece no plano mediano.

Limites locais: X±4mm, Y±4mm, Z±10mm. Campo elipsoidal de raio6,5mm na largura,
3,8mm abaixo/10mm acima do centro e20mm na profundidade, limitado à superfície
frontal do lábio superior/filtro.38vértices do cage, sem alvo no lábio inferior,
cantos, nariz ou corpo. A resposta no anchor suavizado é normalizada pela mesma
regra Catmull-Clark da fonte4. A influência indireta da subdivisão nos vizinhos
é medida na prova nativa, não presumida como zero em todos os pontos próximos.

A fonte **human-cupid.ignis-character** é separada da fonte4. Os alvos, campos,
warps, rig e partes antigos não mudam. NovosXYZ default0 preservam a geometria
da configuração anterior; não aplica automaticamente uma nova forma ao usuário.
A referência fotográfica orienta a capacidade de criar um V definido, sem alegar
reprodução do rosto completo da pessoa. Misturas extremas podem exigir ajustes
adicionais; limites individuais não certificam ausência de auto-interseções.

Fonte5 lê1..4 e permite até80handles; fontes1..4 conservam o teto64. O envelope
256parâmetros permanece, portanto não há novo formato de receita/Scene/replay/
conteúdo/rede/portátil neste incremento. Fonte3..5 continua até96MiB,1/2 até64MiB.
O limite de replayJSON8MiB da base refinada permanece descrito em
[character-smoothing.md](character-smoothing.md); Scene e portátil são seus
caminhos duráveis verificados.

Reprodução, após as fontes anteriores:

```sh
python3 scripts/character/cupid_source.py
python3 scripts/character/cupid_demo.py
```

Nenhuma aquisição ou código externo novo. Os scripts produzem a fonte, prova
MCP, capturas frente/perfil e preset de demonstração em artifacts/character-cupid.
Atualização usa UpgradeCharacterSource MCP/UI existente, com referência atual,
manual-edit guard e receita/bake juntos no undo/save; fontes antigas permanecem
para receitas e desfazer. A janela humana recebe a fonte nova com o ponto a zero.

## Evidência deste incremento

Fonte73.333.717bytes, SHA
b35eba1c9db2ec58bc5606d0c8797d3ac97e6b49a515b5821619cfd0a28ac7ba.
11Rust/2alvos: character_base10 e gesto nativo1. Uma prova nova protege a
fronteira de versão/capacidade: fonte5 até80handles, legadas64, parâmetros256.
Clippyrelease/alltargets-Dwarnings, build, fmt autoral, Python e diff aprovados.
4gruposMCP/6PNGs, build3b5b69864dae4031: bake anterior bit-exato a zero,
253valores mantidos, isolamento local, limites/simetria/assimetria/preview,
referência inválida, undo e configuração/Scene/portátil/reabertura.

Exemplo moderado: Y−1,2mm/Z+1,8mm. Foram conferidos frente/perfil e projeção
no sentido oposto. Nos dois exemplos, picos moveram no máximo0,108mm e centro
do vermelhão0,325mm; lábio inferior/cantos/nariz/queixo/orelha usados como
sentinelas não moveram. Erro métrico máximo de deslocamento7,62e-8m. A base
visual de eixos derivada do rig difere até0,2grau do campo local quando a forma
craniana superior existente está ativa: a primeira medição exigia identidade
vetorial com esse frame e acusou18micrômetros. O oráculo foi corrigido para
medir distância em metros e sentido (<1grau), mantendo tolerância3e-7m na
distância; não houve mudança de código/limites para mascarar a divergência.

Janela humanaPID96153, build conjunto587d93bed378d69b, pronta/pausada/FPSOFF,
fonte5 e ponto zero. Os253valores da última gravação foram mantidos, inclusive
6ajustes feitos enquanto as provas rodavam. Bake idêntico na atualização real.
Cena corrente salva em character-base; antes/config/layout/recibos e capturas
em artifacts/character-cupid. CUA conferiu o botão de seleção e Enquadrar região
na janela real; os lábios ficaram aproximados e o novo ponto selecionado.
Aceite artístico humano permanece aberto. Um binário corrente por symlink;
fontes antigas e frentes181/191 preservadas, sem push.
