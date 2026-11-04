# Cor da luz direcional

`Light.direct_color` é RGB **linear**, três canais finitos de0..1. Branco
`[1,1,1]` é o default e fica omitido da serialização, preservando a autoria antiga.
`direct_intensity` continua independente, de0..4. A cor escala os canais da luz;
não é albedo, uma cor sRGB de interface, temperatura Kelvin ou outro ganho HDR.
Exemplo de sol quente: `direct_color:[1,0.8,0.5]`, `direct_intensity:1`.

## Contrato compartilhado

SetLight/SceneEdit no MCP e painel Iluminação usam o mesmo `Light`. O painel tem
picker linear e canais numéricos R/G/B, sem converter silenciosamente os valores
autorais ao abrir. Preview, commit, undo/redo e replay usam a transação existente;
uma cor inválida não pode alterar cena, histórico, física ou tick.

A cor modula a contribuição DIRETA difusa, dielétrica, metálica, vidro e
transmissão superficial. Ambiente uniforme, IBL, emissão e as cores da névoa
continuam independentes. `atmosphere.sun_color` é a névoa iluminada, não a cor
das superfícies. O campo diagnóstico de vegetação chama `shade_surface`, portanto
recebe a mesma luz sem um segundo modelo ou mudanças em `surface_field`.
Não é uma implementação de novos tipos de sombra, cascatas ou iluminação global.

Scene120 é exigida para cor não branca. SetLight promove a versão quando preciso;
Scene1..119 com luz branca continuam legíveis. Replay130 e conteúdo visual16
acompanham o novo estado público. Protocolo5, PSV1 e checkpoint3 não mudam de
envelope, mas o `scene_version` gravado passa a120. Os saves anteriores continuam
sujeitos às recusas estritas de versão já existentes; não foi criada uma migração
de saves/replays antigos nesta unidade.

## Revisão estática de desempenho

1. JSON/schema são fronteiras de autoria/MCP; o frame usa `Light` tipado.
2. `Globals` cresce448→464bytes, com um vec4 no offset448. Todos os offsets
   anteriores são preservados, inclusive a câmera planetária high/low.
3. RGB×intensidade é preparado com três multiplicações na CPU. O shader recebe
   o ganho final RGB uma única vez, sem multiplicar novamente por `environment.z`.
4. O quarto canal marca branco exato. O vidro branco mantém a ordem escalar
   anterior das operações, inclusive em intensidades diferentes de1. O shader
   opaco mantém a expressão antiga quando o ganho é1 e a mesma ordem de
   multiplicação por canal nas demais intensidades brancas.
5. Não há novos buffers por objeto/material, draws, texturas, passes, alocações
   por pixel ou variantes de pipeline por cor. Há um teste uniforme RGB no shader
   e tratamento uniforme do vidro. Os16bytes integram o upload global existente.
6. Canais limitados a0..1 não ampliam os limites de contribuição direta usados
   pelas seleções conservadoras de LOD/atmosfera. A cor é parte do `Light`
   comparado pelos caches existentes. Não afrouxar essas provas para cores HDR.

## Reprodução da validação

Antes de compilar o shader novo, `python3 scripts/verify_directional_light_color.py
--baseline` captura24referências no executável anterior: seis materiais e quatro
intensidades. Após o build corrente, rodar sem `--baseline` compara RGB8, valida
schema/rejeição atômica e salva/reabre Scene/replay/checkpoint/PSV pelo MCP.
Todos os arquivos ficam em `artifacts/directional-light-color`, sem janela.

`tests/directional_light_color.rs` verifica defaults/formatos/versões, publicação,
undo/replay e atomicidade. Os três testes GPU ignorados por padrão devem ser
executados explicitamente com `--include-ignored`: possuem oráculo numérico para
RGB×intensidade, separação dos lobos/ambiente/IBL/emissão e campo mínimo de grama.

Resultados e build exatos ficam no relatório MCP e nas evidências da EAF-215.
Esta unidade fornece a capacidade; a escolha artística do sol na floresta e sua
medição de FPS pertencem à integração Horizon. Não modificar a cena humana ou
reiniciar sua janela durante estas verificações.

## Resultado desta unidade

Quatro testes CPU e três GPU explícitos aprovados. O oráculo difuso com albedo
0,25, cor[1;0,5;0,25] e intensidade2 produz radiância linear[0,5;0,25;0,125]
antes da conversão sRGB. Casos dielétrico/metálico/vidro/transmissão comprovam
isolamento dos canais; ambiente/IBL/emissão permanecem iguais. O campo mínimo
de16×16sítios também recebeu a cor pelo shader comum.

MCP no build615775b423c36a09:24pares RGB8exatos contra o shader anterior
1863847ea6ee3b26, schema com o novo campo, rejeição atômica e Scene/replay matched/
checkpoint/PSV preservados. Antes de cada load de Scene/checkpoint/PSV, a luz foi
alterada para uma cor diferente para evitar um roundtrip trivial.
Build release corrente, check e Clippy dos alvos lib/bin/teste com `-D warnings`
passaram. O teste unitário do layout464B/offset448 também passou. Dois avisos
iniciais do Clippy nos hooks paralelos de impostor foram corrigidos pelo dono
dessa frente antes da repetição aprovada. Nenhuma janela humana foi aberta.
