# Figueira campestre · referência RS — EAF-197

Continuação corrente: [mais galhos e rede interna](botany-branching.md), após a
segunda foto do usuário. As contagens, testes e capturas abaixo documentam a
primeira versão `099f5d6`; não descrevem os assets regenerados na continuação.

Quinto exemplar, adicional à figueira ornamental de EAF-194. A imagem enviada
pelo usuário orienta este perfil; não há identificação taxonômica nem escala
real confirmadas. [Análise, crops e lista anterior à construção](references/botany/figueira-rs-analysis.md).
O perfil de100anos é autoral, não uma estimativa da idade da árvore fotografada.

## Construção e matemática

`assets/botany/figueira_rs.ignis-botany` usa a mesma fonte/receita4 e o mesmo
compilador nativo dos demais exemplares. O painel Vegetação oferece
**Figueira campestre · RS** separadamente de **Figueira ornamental**.
Não há imagem, textura fotográfica, billboard de copa ou reconhecimento de
espécie no renderer. O verso/profundidade são hipóteses3D.

O gerador recebe192destinos repartidos em5faixas de altura, com espessura,
variação azimutal e raio próprios. As partições espaciais existentes produzem
a hierarquia; a alocação de seção continua por carga potencial de subcopa.
Os campos novos de `Crown` têm defaults que preservam os quatro anteriores:

- `tiers/tier_thickness`: distribuição em faixas em vez de uma única elipsoide.
- `early_vertical`: componente vertical do avanço inicial, menor nos braços baixos.
- `root_fork_forward/fork_forward`: aberturas distintas da primeira forquilha e das seguintes.
- `curve_handle`: alavanca tangencial das curvas estruturais iniciais; ramos finos mantêm a distância de saída necessária aos colares.
- `trunk_lean/min_segment_ratio`: inclinação e espaço entre junções, ainda validadas pelo kernel.
- `forward_min`: permite que terminações retornem em direção aos destinos, evitando elevar artificialmente todas as faixas.
- `shoot_min/shoot_span/shoot_vertical_scale`: brotos menores e mais laterais nos grupos foliares.
- `beard_groups/beard_strands/beard_length/beard_radius/beard_radius_span`: pendentes em grupos, com comprimentos, diâmetros e pequenas ramificações diferentes por seed.

Na variante RS, altura e copa usam o mesmo tempo de crescimento do perfil,
preservando melhor a disposição das camadas ao envelhecer. O engrossamento
e o flare basal têm curvas próprias. As folhas continuam de tamanho limitado.
A base é seção circular alargada, não contrafortes sulcados nem raízes fundidas.

Os pendentes são2880eixos finos (`epifita`/`fio`) em120grupos de8filamentos com
duas terminações secundárias cada. São superfícies fechadas independentes
`sheath`, ligadas mecanicamente ao hospedeiro. `life.role=frond` conserva uma
escala de comprimento limitada e sua densidade/módulos são menores que os dos
ramos lenhosos. Não se pretende uma simulação botânica de epífitas identificadas.
Compartilham a família de material de casca: falta um material específico para
a aparência parda/felpuda da foto. Não há raízes-pilares nesta variante.

## Revisão estrutural e visual

A lista inicial foi entregue antes das curvas. Crops originais foram abertos;
depois foram abertas vistas nativas frontal, inteira, lateral, traseira e
superior da estrutura antes da revisão da folhagem. A hipótese é volumétrica,
com braços em vários azimutes, não camadas alinhadas a uma única câmera.

A primeira estrutura subia demais e formava arcos exagerados. A redução das
alavancas foi limitada aos níveis estruturais: encurtá-las indiscriminadamente
nos ramos finais não acomodava os colares, e o kernel recusou a receita sem
publicar tubos atravessados. A seguir, separar a abertura da primeira forquilha,
permitir retorno aos destinos e limitar a subida dos brotos reduziu a altura
artificial e espalhou mais a folhagem lateralmente. Mais filamentos foram
agrupados em ancoragens próximas; aumentar só seu comprimento não reproduzia
as franjas.

A condição final usa câmera manual/1000×1000, sem solver de alinhamento ou
calibração. A vista próxima pode cortar as bordas laterais como na referência;
a vista inteira e as auxiliares mostram a hipótese completa. As alterações de
geometria e enquadramento foram condições de revisão distintas, sem ranquear
iterações por um score RGB.

| Itens da análise | Fonte/objeto | Construção e comparação final |
| --- | --- | --- |
| RS01 | `tronco0`, asset `figueira_rs_wood` | Tronco curto/inclinado e grosso construído; gesto geral comparado, seção ainda simplificada. |
| RS02 | perfil de desenvolvimento/flare | Base alargada construída; divergente: falta seção lobulada, sulcos profundos e fusão dos fustes/raízes. |
| RS03-04 | eixos `fork/graft`,1199junções | Braços laterais e ramificação construída, pele fechada verificada; algumas curvas permanecem mais regulares/arqueadas que na foto. |
| RS05-06 |192destinos/5faixas e brotos laterais | Patamares/vãos construídos e comparados; ainda menos compactos e com disposição diferente dos grupos da referência. Não validados como réplica. |
| RS07 |48576lâminas, `figueira_rs_leaves` | Folhagem geométrica construída; escala individual e textura das massas não reproduzem exatamente a foto. |
| RS08 |2880eixos `epifita/fio` | Pendentes construídos e comparados com/sem eles; fios visíveis, mas faltam os tufos felpudos densos da referência. |
| RS09 | padrões/materiais nativos | Madeira escurecida e variação procedural; não é relevo basal nem material fotográfico. |
| RS10 | vistas lateral/verso/superior | Coerência volumétrica conferida; verso real permanece desconhecido e autointerseção global não foi certificada. |
| RS11 | chão/luz/câmeras do roteiro | Apoio visual e sombra; não reconstruídos os agaves, postes, grama e nuvens da foto. |

Os pares de crops referência/render foram abertos nas mesmas caixas:
base `(280,480,370,310)`, camadas `(340,120,620,490)`, pendentes
`(630,390,300,230)` e topo `(380,50,400,230)`, sem recentralizar cada peça.
O diagnóstico local confirma base demasiadamente lisa/estreita, camadas menos
compactas, pendentes finos em vez de tufos e topo mais pontudo/irregular.
Isso permanece divergência; testes de software não a aprovam.

RGB8 sem máscara/registro: MAE global0,29094 e SSIM luminância0,19751.
Por região, MAE/SSIM: base0,39854/0,09118; camadas0,32341/0,06766;
pendentes0,34282/0,03722; topo0,28458/0,09820. Incluem fundo, iluminação,
perspectiva e objetos não reconstruídos. SSIM não é porcentagem de fidelidade;
não houve medida de IoU/segmentação ou ajuste automático da câmera.

## Evidências e limites

Build corrente `e75b3a02d68e85d0` macOS arm64/release/rustc1.97.1, em
`target/release/ignis`. Apenas botão/preset/roteiros/testes/documentação foram
alterados: kernel geométrico, Source/Recipe4, Scene109/replay119 e os demais
formatos permanecem iguais. Um build corrente, sem tocar a cena/janela humana.

Fonte52.847órgãos:48.576folhas,1391eixos lenhosos e2880pendentes;
2.700.094vértices/5.194.360triângulos. Fonte expandida~81,38MiB, dentro de96MiB;
budgets65536órgãos/3Mv/6Mtri preservados. O custo de fonte expandida continua
alto; isso não é uma gramática compacta nativa nem um preset gratuito para jogos.

14testes Rust release aprovados (17,85s). O novo caso cobre integração do
exemplar completo com pendentes, determinismo, fechamento/orientação da pele,
repouso/crescimento físico e versão despida sem pendentes. Partes vazias têm o
placeholder microscópico de ownership já existente, sem bindings materiais;
este não é contado como pele botânica a exigir fechamento. O lenho real da
inspeção CPU tem zero arestas de borda e zero arestas com incidência/orientação
incorreta. Demais casos críticos foram conservados. Clippy lib/bin/botany,
Python, formatação de testes e diff também verificados.
Uma regeneração dos quatro exemplares anteriores manteve o JSON canônico
idêntico às fontes existentes; nenhum dos quatro assets foi alterado.

MCP: schema real consultado, publicação/capturas, repouso de1.296.116pontos e
5.952.106restrições,4passos com vento e erro final de vínculo2,53e-8m;
poda de um filamento conserva identidade e livro de massa. Cena salva/reaberta
reproduz PNG bit-exato. Estes são ensaios autorais locais, não FPS nem física de
gameplay; as quotas de contato e demais orçamentos continuam independentes.

Arquivos correntes em `artifacts/botany-rs/`: `review.html`, `reference-manifest.json`,
`native-schema.json`, `structure-report.json`, `structure-topology.json`,
`report.json`, `difference.png`, crops, cinco vistas, cenas e logs de validação.
A referência recebida fica no cache de análise e não integra os assets distribuídos.

```
python3 scripts/botany_exemplars.py --only figueira_rs
cargo build --release
cargo test --release --test botany
python3 scripts/demo_botany_rs.py --structure-only
# Abrir estrutura/auxiliares antes da etapa de aparência.
python3 scripts/demo_botany_rs.py
```

O comparativo completo exige `artifacts/botany-rs/reference.png`, a foto recebida
com o hash do manifesto; gerar/carregar a planta não depende dessa imagem.
Preservar a figueira ornamental anterior. A tarefa continua em revisão artística,
com as divergências acima abertas; não afirmar identificação científica ou
reprodução exata do exemplar do RS.
