# Deformação dos galhos e referências dos exemplares

Continuação atual: [EAF-193, idade em anos e exemplares adultos](botany-age.md).
As medidas abaixo registram EAF-192; as fontes correntes usam receita/fonte3
e perfis cronológicos, sem invalidar a evidência histórica desta revisão.

EAF-192, continuação da integração EAF-191 a pedido de Guilherme. O objetivo é
melhorar a ferramenta e a semelhança visual das quatro receitas; não declarar
uma réplica botânica exata a partir de uma foto.

## Revisão prévia e critério visual

A V3 usava poucos galhos igualmente espaçados num tronco quase reto. A copa
resultante era esparsa e os exemplares compartilhavam demais a arquitetura.
A avaliação nova compara silhueta inteira, estrutura sem folhas, proporções,
afilamento, direção das pontas e agrupamentos de folhas. Quantidade de polígonos
ou aprovação dos testes não mede semelhança.

Referências consultadas:

- Palmeirinha areca (*Dypsis lutescens*): [JCU](https://www.jcu.edu.au/discover-nature-at-jcu/plants/plants-by-scientific-name2/dypsis-lutescens)
  e [foto de touceira ornamental](https://www.windyridgegardencentre.ie/products/dypsis-lutescens-areca-160-cm).
  Touceira com caules finos, frondes ascendentes que arqueiam, folíolos estreitos
  e comprimentos variáveis ao longo da ráquis. Referência também na foto do usuário.
- Plátano (*Platanus × hispanica*): [Stanford, verão/inverno e folhas](https://trees.stanford.edu/ENCYC/PLATace.htm)
  e [Van den Berk](https://www.vdberk.nl/bomen/platanus-hispanica/).
  Tronco manchado, forquilhas irregulares, copa ampla e folhas palmadas. O exemplar
  podado de viveiro não representa sozinho a silhueta de uma árvore livre adulta.
- Eucalipto (*Eucalyptus globulus* como direção): [registro municipal de Glen Eira](https://www.gleneira.vic.gov.au/our-city/classified-trees/classified-tree-register/357ctr2022-hawthorn-road)
  e [Universidade de Évora](https://www.mitra-nature.uevora.pt/Especies-e-habitats/Plantas/Lenhosas/Arvores/Myrtaceae/Eucalyptus-globulus).
  Ramos ascendentes irregulares, madeira aparente, folhagem estreita e pendente
  em grupos nas ramificações finais; copa mais aberta que a figueira.
- Figueira ornamental (*Ficus benjamina*): [NParks](https://www.nparks.gov.sg/FloraFaunaWeb/Flora/2/9/2902)
  e [foto de copa adulta](https://www.gardenershq.com/Ficus-benjamina.php).
  Copa larga e densa, ramúsculos pendentes, folhas ovadas/alongadas e raízes aéreas.
  A escolha ornamental de copa ampla/raízes aparentes veio do usuário.

Fotos externas permanecem no cache ignorado de análise; os arquivos distribuídos
são fontes procedurais próprias. Os links conservam autoria e contexto originais.

## Contrato de implementação

O campo de deformação modifica a **curva de repouso** antes de posicionar os
filhos. É distinto do deslocamento dinâmico por peso/vento. Base/ponta controlam
curvaturas nos dois eixos materiais em rad/m; somam-se torção, atração para +Y,
pendência e irregularidade 3D determinística por seed/ID. O vetor de rotação é
`ω = T × dT/ds + twist T`; integração exponencial no ponto médio conserva o
comprimento de cada segmento. A passagem suave base→ponta é cúbica. O campo
vertical é projetado no plano normal à tangente. Não representa gravidade
física estática calibrada: peso e rigidez pertencem ao ensaio XPBD existente.

Sem amplitudes ativas, o caminho antigo é preservado, inclusive ao ajustar só a
frequência do ruído. Ativar o campo não reamostra a spline original: isso
alteraria discretamente seu comprimento e a massa. A subdivisão material conserva
o comprimento de galhos que já eram curvos. Campo ativo subdivide a polilinha
material existente até pelo menos128intervalos de integração (múltiplo da
discretização anterior), fora do frame estável; a tesselação final conserva
endereços materiais. Os perfis base→ponta/frequência usam a coordenada material
normalizada do eixo; incrementos de rotação usam a distância física em metros.
Incremento angular >0,35rad por intervalo ou posição inválida recusa a receita.
Filhos seguem o frame alterado do pai e o solver recebe as mesmas poses de
repouso usadas pelo bake. Editar uma curva invalida a sessão física anterior.

Fonte/receita2 acrescentam campos opcionais com defaults de identidade; leitura
aceita1/2 e publicação grava receita2. Scene109/replay119 correntes são preservados,
bem como conteúdo/protocolo/save portátil. Cenas JSON/CBOR conservam os campos
por nome. Binários antigos não passam a suportar fontes2 por essa compatibilidade.
A fonte também admite afilamento por potência e conexão `sheath`: pecíolos de
frondes são superfícies fechadas independentes, vinculadas mecanicamente ao
caule, sem fingir forquilhas lenhosas. Galhos `graft` continuam exigindo junção
contínua validada; nunca atravessamentos silenciosos como fallback.

As junções preservam as primeiras tentativas da V3 quando válidas. Caso falhem,
a busca converte distâncias em coordenadas pela soma dos segmentos da curva,
em vez de supor parâmetro uniforme. Isso permite curvas de velocidade variável
sem relaxar a separação dos planos de colar. Os pecíolos `sheath` continuam com
superfícies separadas; a validação não os apresenta como uma pele lenhosa unida.

Checklist: compilar só após edição, quotas antes de malhas, buffers físicos
existentes, sem dependência externa nova ou JSON por frame. Ajustes manuais
continuam por identidade/material; perda de endereço exige recusa ou descarte
explícito. Teste novo protege essa fronteira de formato/identidade/repouso físico
e atomicidade, não os valores individuais dos sliders. Janela e mundo humanos
permanecem intocados; um build corrente e capturas offscreen em caminhos estáveis.

## Evidências e avaliação

A revisão visual teve mais de uma rodada nativa. A primeira ainda produzia
copas em andares e uma areca baixa/larga; a seguinte concentrou os destinos dos
ramos no volume da copa, distribuiu secundários ao redor de todo o eixo e variou
os comprimentos dos ramúsculos. A figueira ganhou mais ramúsculos, copa menor
para a mesma área foliar e oito raízes nascidas dos ramos. A areca inverteu a
altura de inserção: frondes novas/ascendentes perto do ápice e frondes mais
pendentes abaixo. O eucalipto recebeu mais folhas estreitas com orientação
predominantemente descendente e variação angular.

O teto do ensaio físico subiu de65536 para131072pontos, checado antes da alocação,
para não omitir folhas da figueira mais densa. Os limites de1Mrestrições,
100kpares/1Mentradas de contato e1Mvertices/2Mtriângulos continuam explícitos;
mais pontos não certificam contato completo entre todas as folhas de uma copa.
O modelo mecânico e o protocolo de persistência não mudaram.

A avaliação melhora a leitura dos quatro arquétipos; permanece uma aproximação
procedural, sem reprodução métrica de uma árvore fotografada, envelhecimento
botânico calibrado, anastomose entre raízes/troncos, casca laminada ou tecido
foliar vascular completo. As árvores de referência são adultas; escala e
proporção das folhas das receitas são escolhas autorais, não medidas extraídas
da fotografia. A iluminação é a do laboratório Ignis, sem pós-produção.

As capturas antes/depois e os quatro campos da mesma curva estão no relatório
local `artifacts/botany-native/review.html`. A cópia anterior serve à comparação
ativa; iterações intermediárias substituem os mesmos caminhos. Fonte e cena da
janela humana não são carregadas por esses roteiros.

Reprodução:

```sh
python3 scripts/botany_exemplars.py
cargo test --offline --test botany
cargo build --offline --release --bin ignis
python3 scripts/demo_botany_native.py
python3 scripts/demo_botany_deformation.py
python3 scripts/verify_botany_native.py
python3 scripts/review_botany_native.py
```

`--only` nos geradores de fontes/capturas permite iterar um exemplar. A publicação
agora expõe bounds do bake para enquadrar a nova copa sem cortes. Folhas usam
cinco controles longitudinais em vez de dezessete; a tesselação fechada permanece
a mesma. Não há download de referência ou execução de Python no runtime/editor.

Validação:10testes Rust (um teste crítico novo, com fonte inicialmente curva e
campo de frequência dormente), Clippy lib+alvo botany com `-D warnings`, rustfmt
nos arquivos tocados, Python e diff sem erro. Cenários existentes continuam
cobrindo escultura/undo/formatos, crescimento/momento, grandes dobramentos e
falha de contato atômica. A política de testes por risco foi conservada.

| Exemplar | Órgãos | Junções lenhosas | Vértices | Triângulos |
|---|---:|---:|---:|---:|
| palmeirinha | 2065 | 0 | 237370 | 466480 |
| platano | 2196 | 215 | 806438 | 1604952 |
| eucalipto | 3816 | 215 | 452574 | 890744 |
| figueira | 4210 | 368 | 509532 | 1003696 |

A galeria local tem aproximadamente156MiB de JSON de fonte+bake, sem envelope
novo de rede/replay. Publicação isolada dos exemplares ficou em torno de0,15–0,52s
neste Mac; isso não é FPS nem garantia universal de performance. A figueira
completa usa104920pontos/482492restrições no ensaio. O roteiro cresce de1,8 a3,4,
resolve128passos, poda uma subárvore e verifica identidade, vínculos e balanços
de massa/momento. Tempo e resíduos exatos estão no JSON da verificação.

Build corrente macOS arm64: `c9ad9ad6c02f24ec`, em
`target/release/ignis`. MCP final aprovado para preview/cancelamento, fonte,
replay e save portátil com o campo de curvatura; crescimento/poda da figueira
completa, grande dobramento/chão e contato entre galhos. Galeria salva/reaberta
com PNG bit a bit igual. O crescimento completo levou41.74s
nesta máquina; resíduo final de vínculo9.27e-09m.
São104920pontos participantes, sem reduzir a planta a um galho representativo.

Relatórios correntes: `report.json`, `deformation-report.json`, `verification.json`,
`ref-tests.log`, `clippy.log` e `build.log` sob `artifacts/botany-native/`.
Capturas completas/estrutura e os quatro campos foram inspecionados no renderer
nativo; `review.html` reúne os comparativos. Aceite artístico humano permanece
separado da conclusão técnica desta revisão. Sem push, alteração do mundo humano,
lançamento de janela, cena de personagem ou promessa de réplica fotográfica.

## Uso dos controles

No painel **Vegetação**, escolha a planta e filtre o identificador do órgão
(por exemplo `tronco0/ramo0`). Abra **Curva de crescimento do galho**: base/ponta
controlam N/B; os demais controles somam torção, pendência, orientação vertical
e irregularidade. **Prévia da curva** usa a candidata sem alterar o documento;
**Aplicar ajuste ao órgão** publica a receita e o bake juntos. Use a seção de
ensaio físico depois de aplicar a curva de repouso.

O mesmo ajuste pela API, no pequeno exemplar produzido por
`demo_botany_deformation.py`:

```json
{"op":"botany","operation":{"kind":"edit_organ","prefix":"curve","organ":"branch","edit":{"deformation":{"base":[0.95,0],"tip":[-1.4,0.15],"twist":0.2}}}}
```

Forças altas, junções sem espaço ou perda de um endereço manual ajustado
continuam causando uma recusa explícita da candidata. A ferramenta conserva
o documento anterior nesses casos; não afrouxa a integridade para aceitar uma
curva arbitrária.
