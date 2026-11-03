# Lábios, bases adultas e pivôs cranianos — EAF-187

Atualização posterior: [malha suave, pontos e orelhas](character-smoothing.md) retira as fibras artificiais da fonte4. Este documento registra o comportamento da fonte3.

Continuação de character-controls.md após os prints humanos. A instrução mais
recente mantém as deformações antigas da parte superior da cabeça como formas
úteis para alienígenas, em vez de removê-las. As posturas de coluna e ombros foram
preservadas; o preset Heroica usa os quatro valores mostrados pelo usuário:
pelve−0,53, lombar0,69, tórax−0,15 e ombros0,44. Presets não acionam formas cranianas.

## Movimentos e formas

- `neck_pitch`: pescoço e cabeça em torno da base do pescoço na coluna (`neck01`).
- `head_pitch`: cabeça inteira em torno da junção crânio/pescoço (`head`).
- Ambos usam−1..1 para±40graus. Todos os vértices craniofaciais/âncoras têm peso1;
  apenas a junção com pescoço/tronco mistura pesos. Olhos seguem os binds ajustados.
- `posture_head_pitch` e `posture_head_retraction` conservam exatamente os campos
  antigos, agora rotulados como inclinação/recuo **da parte superior** da cabeça.
- `head_lower_pitch`: forma intencional da parte inferior, até±35graus com transição
  próxima à ponte nasal; não é um movimento rígido.
- `head_height`: forma vertical ampla do crânio. O delta autoral varia de−55% a
  +120% da distância de referência ao limite inferior do crânio; combinações com outros alvos
  não são uma promessa de altura métrica exata.

Movimentos continuam sendo edições do repouso com rig novamente ajustado. Não
são uma nova camada de animação ou IK. Formas cranianas podem deliberadamente
alterar anatomia/proporções; os dois novos pivôs conservam a forma recebida.

## Lábios

14controles de forma independentes: espessura, volume e largura superiores/
inferiores; curvatura e largura do arco do cupido; filtro; centro superior/
inferior; cantos e laterais superiores/inferiores.28alvos CC0 pinned dão os dois
extremos de cada parâmetro; valores default0 conservam a boca anterior.

16novos pontosXYZ abrangem picos do cupido, volumes laterais, bordas externas e
internas superiores/inferiores, tubérculo central e contorno inferior central.
58pontos totais/223canais na fonte atual. Os campos novos respeitam a separação
entre as superfícies labiais superior/inferior; simetria e limites são os mesmos
do kernel existente. Isso permite formas assimétricas, sem garantir que toda
combinação extrema feche os lábios ou evite auto-interseções.

`Handle.group` opcional vem da fonte; `HandleView.group` opcional seleciona a
região da sessão, com paridade MCP/editor. **Lábios** tem20pontos incluindo os
quatro anteriores; **Rosto** tem38. Enquadrar região aproxima somente a seleção
desse grupo. Grupos não fazem parte da Scene/configuração e não filtram o bake.

## Masculino e definição

`body_masculinity`0..1 mistura as bases anatômicas adultas feminina/masculina.
Botões de base escolhem0 ou1, preservando os demais valores; o slider permite
intermediários.30alvos masculinos (bases, músculo/gordura e altura) usam a mesma
revisão CC0 e topologia/rig. Alvos específicos de seios são atenuados ao trocar
para a base masculina. Extremos de massa também são condicionados à base.

Definição combina separações geométricas mais amplas e relevo fino de estrias.
`muscle_striation` controla o relevo fino junto de `muscle_definition`; gordura
atenua ambos. O relevo é uma aproximação procedural que ainda exige acabamento artístico.
Estrias usam o `Material.Detail` nativo existente (31fibras reutilizadas por regiões, limite32segmentos,
perfil arredondado, bump e opacidade de cor0), com coordenadas de repouso
ancoradas à malha. A superfície acompanha postura/animação sem recalcular uma
receita por frame; não são listras de cor nem uma nova textura externa. A
silhueta continua determinada pela geometria, não pelo bump do material.

## Fonte3, arquivos e custo

Reprodução offline:

```sh
python3 scripts/character/prepare_source.py
python3 scripts/character/extend_source.py
python3 scripts/character/refine_source.py
```

`refine_source.py --fetch` adquire explicitamente os58alvos gráficos adicionais;
218arquivos do manifesto têm hashes conferidos. Sem aquisição no runtime e sem
código MakeHuman importado. Fonte1 e fonte2 permanecem disponíveis para receitas
antigas. A atual fica em artifacts/character-base/human.ignis-character.

Fonte3 acrescenta `surfaces` opcionais, grupos de handles e `Warp.rigid_sockets`.
O grupo final de warps rígidos conserva o twist ocular anterior, com os mesmos
sockets e peso1 em suas âncoras; não deriva orientação novamente só pelo eixo
Y global. Isso evita rotação ocular residual mesmo quando o crânio está rígido.
Surface associa uma
parte skinned, curvas escalares0..1, um Detail validado e coordenadas finitas
com contagem igual à geometria. Ganho0 mantém o bake anterior sem canais extras;
ganho ativo instala material e coordenadas na mesma transação. Fonte1/2 recusam
esses campos; seu envelope64MiB é preservado, fonte3 admite96MiB para as duas
bases. Limites256canais/64pontos/16warps/512alvos/4M deltas+pesos/100mil vértices
permanecem. Scene107/replay117/conteúdo7/protocolo5/PSV1/checkpoint3 preservados.

Fonte/descritores/coords em cache. Curvas, geração de mesh/rig/hash e cópia das
coordenadas ativas são custos síncronos de edição. Sem I/O por frame, novo shader,
thread, solver ou promessa de FPS. Mais detalhes custam o caminho de material
existente; renderização do bake dispensa arquivos de autoria.

`scripts/character/refinement_demo.py --capture` reproduz o legado, os novos
pivôs, lábios, masculino/feminino, superfície e persistência pelo executável real,
sem salvar sobre a cena humana. Relatórios/PNGs em artifacts/character-refinement.
