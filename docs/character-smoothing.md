# Malha suave, pontos anatômicos e orelhas — EAF-187

Incremento após a avaliação humana da fonte3: mandíbula/lóbulo fora do lugar,
lábios ocultos pelo filtro de região, fibras artificiais e contornos grosseiros.

## Geometria e rig

Fonte4 acrescenta `refinements`, uma lista opcional de cages poligonais por
parte. A triangulação em leque do cage deve coincidir exatamente com a parte
original; índices, orientação, incidência de arestas e conectividade dos leques
são validados. Uma etapa Catmull-Clark calcula vértices/arestas/centros com
regras de fronteira. Não é apenas subdivisão linear dos triângulos existentes.

A avaliação acontece depois dos alvos e warps, antes da publicação do bake.
Os slots originais permanecem primeiro, agora em suas posições suavizadas;
handles leem esses mesmos slots da geometria efetivamente desenhada. Campos XYZ
são normalizados pela resposta no anchor subdividido. As regras e pesos de rig
ficam em cache; as posições e coordenadas materiais usam os mesmos stencils.
O cache confere cage/triângulos/pesos/contagem antes do reuso, inclusive em clones
modificados. Nenhuma influência positiva é descartada: mais de16 recusa a fonte.

Corpo:13.380→53.514v,26.756→107.024tri. Com os olhos:55.922v/111.664tri,
163ossos; até14influências no bake refinado desta base. Limite autoral agregado
100mil vértices refinados,256canais/64handles/512alvos/16warps/4M deltas+pesos;
96MiB fonte3/4 e64MiB fonte1/2 preservados. Uma etapa é deliberada: mais detalhes
anatômicos exigem autoria, e não surgem apenas multiplicando polígonos.

## Pontos e orelhas

64pontos:34Rosto,20Lábios,10Orelhas. **Todos** aparece como opção visível, junto
às regiões; a sessão começa sem filtro. Mandíbula foi ancorada à borda mandibular,
fora do pescoço; lóbulo na extremidade inferior real da orelha. Borda superior,
ponta, borda externa, concha e lóbulo têm XYZ e simetria por lado.

12formas escalares de orelha: tamanho, tamanho do lóbulo, altura, profundidade,
abertura e ponta, por lado. Cinco pares−1..1 e ponta0..2, com22alvos gráficos
CC0 da mesma revisão MakeHuman; controles3D acrescentam18canais. Fonte4 tem
253canais/460alvos/7warps. Orelhas pontudas são formas autorais de fantasia;
limites individuais não garantem ausência de interseções em toda mistura extrema.

O relevo procedural de fibras da fonte3 foi removido da nova base: os prints
mostraram um resultado artificial. `muscle_striation` permanece na configuração
para não perder valores antigos, mas não produz linhas na fonte4 e fica oculto
no painel dessa fonte. A definição geométrica ampla permanece. Fonte3 continua
inalterada para receitas antigas e desfazer; seu material conserva o comportamento
histórico. Refinamento suave também não equivale a pele/poros fotográficos.

## Atualizar sem perder a criação

`UpgradeCharacterSource {prefix,path,reference}` é compartilhado por MCP/editor.
Só uma base sincronizada/editável e pausada pode atualizar. Exige referência
atual, partes com as mesmas identidades e configuração anterior válida na nova
fonte. Preserva prefixo/placement/valores e acrescenta defaults novos; compila e
valida o candidato antes de substituir receita e bake numa única transação.
Falha não publica. Edição manual bloqueia, como nos sliders; undo/redo restaura
fonte e geometria correspondentes. **Configuração e fonte → Atualizar fonte da
base** permite escolher o arquivo. `ConnectCharacterSource` conserva seu contrato
de reconhecer somente um bake idêntico e não faz essa substituição.

Nova fonte: `artifacts/character-base/human-smooth.ignis-character`.
Fontes1/2/3 mantidas. Reprodução offline, após gerar as anteriores:

```sh
python3 scripts/character/smooth_source.py
python3 scripts/character/smoothing_demo.py
```

`--fetch` no primeiro script adquire apenas dados gráficos pinned, com manifesto
verificado; nenhum código MakeHuman importado. Sem I/O por frame. A subdivisão
amplia memória, hash, skinning e custo de edição/render; não implica FPS garantido,
LOD, novos correctives, expressões, bounce ou Playeravatar.

O cenário real MCP salva relatórios/vistas em `artifacts/character-smoothing`,
sem substituir a cena humana. Protege migração/configuração, rig, anchors,
formas/XYZ das orelhas, pivôs, undo e save portátil. Replay da fonte4 é
protegido pela prova sintética crítica; a base real refinada já tem9.495.501bytes
no JSON da cena de prova e excede o envelope de8MiB do replay JSON. A exportação
recusa sem mutação/escrita parcial; o limite foi preservado. Scene/save portátil
são os caminhos de persistência verificados para a base completa. Aceite artístico
humano permanece separado das provas automatizadas.
