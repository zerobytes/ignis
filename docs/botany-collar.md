# Colo sem massa bulbosa e sem aba reta — EAF-198

O usuário aprovou a melhora das paredes de2d49df8, mas apontou uma “bola” no
colo e uma borda reta que parecia se projetar entre raízes. A [análise C01/C02
e lista CC01–04](references/botany/collar-correction-analysis.md) precedem a
correção. A [comparação nativa](../artifacts/botany-collar/review.html) mantém
câmera, luz, parâmetros, curvas e órgãos, com ampliação da região indicada.

## Causa e solução

A junção antiga partia de uma envoltória convexa circular. Transformar os
colares em paredes e estender seus deslocamentos ao interior conservava parte
da massa convexa inicial. A leitura arredondada e a aba pertenciam à própria
pele costurada; não havia uma peça independente para apagar. Recortar faces
abriria a malha. Trocar material ou câmera também não corrigiria o volume.

Source/Recipe7 introduzem um bake específico do colo em paredes. Leitura e
bake1..6 permanecem, inclusive a geometria antiga quando o documento continua
em6. Não muda o perfil tubular, os forks/grafts comuns, o renderer ou os
formatos globais Scene109/replay119/conteúdo7/protocolo5/checkpoint3/PSV1.

`src/botany/root_crown.rs` resolve **posições** do interior, com guia de vales
orientada pelas raízes; não conserva simplesmente o deslocamento de uma bola.
Os colares continuam exatos e compartilham índices com os tubos. O guia tem um
eixo interpolado entre base e colar do tronco, raio que segue a seção do tronco
e lóbulos derivados dos colares reais das raízes:

```text
ρguia(φ,y) = rtronco(y) + maxᵢ [ alcanceᵢ · exp(−(Δφᵢ/σᵢ)²) · quedaᵢ(y) ]
```

`σᵢ` depende da largura angular do colar e da separação entre raízes; a queda
é suave entre crista e colar do tronco. Entre contrafortes o guia volta ao raio
do tronco, em vez de preencher todo o setor com uma cúpula. A altura de corte
também precisa deixar espaço para as cristas; combinações inviáveis recusam.

A primeira faixa interior recebe uma restrição suave orientada pela derivada
da seção varrida. A profundidade da faixa é limitada a20% do raio local. Um
termo de segunda ordem conserva a separação angular/radial do chart: impor
apenas uma coordenada de sweep podia colapsar pontos distintos. Essa direção
reduz a quebra aparente na entrada da crista; não certifica tangência C1 exata.

São192 iterações por candidato, com três pesos limitados de guia (0,8/0,4/0,15)
e viés0,5 para a faixa de encontro. A mudança é conferida em64 passos de
homotopia, recusando colapso/inversão local. Priorizam-se soluções completas;
75%/50% da mudança podem ser usados quando conservam a orientação. A busca de
colares prioriza ajuste completo, com alternativas de altura limitadas. Não há
fallback silencioso para uma malha atravessada, nem remoção de faces para fazer
o teste passar. Fechamento/orientação são conferidos separadamente da aparência;
não se prova ausência de qualquer autointerseção global ou união CSG universal.

O campo de nervuras posterior continua comum à pele da mesma árvore. Não foram
modificados os36.400 órgãos, suas curvas, parâmetros de raízes ou idade autoral.
Não se redesenhou a copa, nem se aumentou a sua resolução para corrigir o colo.

## Autoria, versões e escultura

`Botany.SetRoots` e **Vegetação → Aplicar raízes** produzem versão7 quando há
perfil em parede. Publicar/carregar uma fonte6 preserva o bake6; regenerar pelo
controle de raízes faz a atualização explícita. Nenhum controle exclusivo de
MCP ou da UI foi acrescentado. O painel usa o mesmo dispatcher; não foi aberto
para teste manual nesta rodada, preservando a janela humana.

O domínio dos pontos interiores novos recebe `valley-v1`, além dos hashes de
colares/conectividade existentes. Uma escultura antiga sem correspondência na
nova junção provoca recusa atômica; não é descartada durante a atualização.
Existe a política explícita de descarte já documentada, sem descarte automático.
Receita6 com fonte7 em paredes é recusada; documentos1..6 continuam legíveis.

O teste `valley_crown_versions_preserve_foliage_and_refuse_legacy_sculpt_loss`
confere geometria foliar exata entre6/7, mudança da pele lenhosa, pele fechada,
roundtrip6 e recusa de atualização com escultura incompatível sem mudar cena/
revisão. O corpus cobre também idade8/20/80, poda, vínculos, massa e persistência.

## Conferência por item

| Item | Evidência / resultado |
| --- | --- |
| CC01 | Frente/lado/verso da árvore completa: centro mais estreito e orientado pelos contrafortes; massa arredondada dominante removida na região indicada. Superior e rede sem solo também inspecionadas. |
| CC02 | Mesma câmera lateral, detalhe `(400,260,500,450)` em imagens1200×1000: a antiga aba horizontal não se projeta entre as paredes; encontro segue uma curva. Há pequenas irregularidades/variações de normal, sem promessa de acabamento botânico perfeito. |
| CC03 | Faixa pela derivada da seção, restrição suave e guardas de orientação; transição melhorada, C1 exato não afirmado. |
| CC04 | Todos os órgãos normalizados iguais, inclusive raízes; parâmetros e curvas preservados. Folhagem exata no caso reduzido; imagem Source6 anterior preservada e escultura protegida. |

O diagnóstico inicial de52 órgãos foi útil, mas não bastou: a copa reduzida
deixava espaços de colar diferentes. A árvore completa revelou uma dobra que
exigiu compatibilizar a faixa de encontro e o guia. A validação final usa a
árvore completa, sem tratar o diagnóstico local como aceite. Não se enfraqueceu
a guarda de orientação para aceitar a dobra.

`compare_images` foi aplicado à lateral completa com regiões fixas: colo
`(400,260,500,450)` e borda `(530,390,330,270)`. MAE RGB0..1:0,01373 e0,01437,
respectivamente, com225.000 e89.100 pixels. São medidas de mudança na mesma
grade, não fidelidade às fotos nem prova matemática de tangência. Diferença
em `change.png`; recortes exatos sem resize em `detail-before/after.png`.

## Validação e custo

Build macOS arm64/Metal `ff06abebb1988d66`, main, release atual
`target/release/ignis`.19 testes Rust botany release passaram após atualizar a
fonte canônica; Clippy `-D warnings`, fmt, Python, links e diff verificados.
O aviso transitivo futuro de `block` permanece o anterior. Nenhuma alteração
do kernel `mesh_organic` ou do solver físico foi necessária nesta revisão.

O PNG lateral Source6 recompilado no build7 é byte exato ao render da etapa
anterior. A fonte7 regenerada também é byte exata (`regeneration.json`).
Save/load adulto preserva PNG exato. São36.400 órgãos/28.416 folhas e
2.884.806v/5.644.424tri, mesmos totais da versão6; fonte37.101.362bytes. Os
parâmetros/órgãos permanecem, a diferença está na junção e versão de bake.
Outros quatro assets não alterados. Aceite artístico humano permanece separado.

Ensaio1.175.104 nós/5.511.516 restrições/672 âncoras: quatro passos com vento,
identidade estável e vínculo1,30e−9m; poda e balanço de massa dentro1e−8kg.
Massa é do modelo reduzido por seções, não volume constitutivo exato da junção
ou massa medida da árvore. Solo, tecido, anastomose e gameplay continuam fora
dessas garantias.

Revisão estática: adjacência, guias e buffers restritos ao colo/até12 raízes e
160 amostras de colares. Busca e iterações são custos de autoria, sem geração
por frame, shader/passeGPU, rede, I/O ou cache global novo. Cópias e bake completo
continuam reais; publicação adulta medida em2,91s neste roteiro, sem inferir FPS
ou latência universal de painel. Mesmas quotas combinadas de fonte/malha/ensaio/
contato. Mundo/janela humanos preservados e um release corrente, sem push.

Reprodução:

```sh
cargo build --release
python3 scripts/demo_botany_collar.py
# Conferir corpo e detalhes antes da aparência:
python3 scripts/demo_botany_collar.py --appearance-only --publish-source
python3 scripts/botany_exemplars.py --only figueira_rs
cargo test --release --test botany
```

`--local` gera o diagnóstico reduzido de52 órgãos em diretório próprio estável;
recusa publicação canônica. A galeria final usa todos os órgãos. As fontes e
curvas são procedurais; imagens servem à conferência, não entram na geometria.
