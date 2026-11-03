# Colo bulboso e borda reta — análise anterior à correção EAF-198

O usuário aprovou a melhora das paredes em2d49df8 e identificou duas falhas no
render: volume semelhante a uma bola no colo e saliência quase reta entre
raízes. São imagens do modelo, não novas fotos botânicas. Fontes recebidas:
`codex-clipboard-213e86c7-e06d-4aa7-b161-53f1fce4ccd8.png`1392×900 e
`codex-clipboard-f6e5aa41-070c-4dd4-9f13-b28e02ec67fe.png`690×362.
Inteiras e recortes reais foram abertos antes da alteração geométrica.

| Ficha | Recorte na fonte `(x,y,w,h)` | Leitura e consequência |
| --- | --- | --- |
| C01 | imagem1,170,0,980,660 | Tronco estreito encontra uma massa larga/arredondada; essa massa preenche parte dos vales entre paredes. A saliência permanece na geometria cinza, não é só textura. Corrigir a distribuição do volume da junção, mantendo raízes/colo conectados. |
| C02 | imagem2,70,45,550,280 | Uma borda quase horizontal aparece entre a crista frontal esquerda e a parede direita. Não parece seguir a curva de crescimento de nenhuma delas. A imagem não comprova peça solta: a causa pode ser inclinação/triangulação da pele contínua. Inspecionar a conectividade e a direção de encontro antes de remover faces. |

Revisitação: o corpo de parede/crista já está presente e deve ser preservado.
Os defeitos atravessam a mesma região da junção central; apagar um triângulo ou
esconder com solo/material não resolve o volume. Galhos finos, cor e sombra são
contexto. As fotos botânicas e seus limites continuam em `buttress-analysis.md`.

## Lista antes da geometria

Todos executáveis **não construído**, verificação nova **não executada**.

| ID/pai | Construir/corrigir | Evidência/estado | Critério |
| --- | --- | --- | --- |
| CC01/colo | Retirar a distribuição bulbosa contínua entre raízes; formar vales separados por contrafortes | C01 observado; a solução3D é hipótese a conferir | Frente/lado/verso mostram vales e continuidade com o tronco, sem bola/saia circular dominante |
| CC02/CC01 | Remover a leitura de borda reta/saliência no vale | C02 observado; peça solta não confirmada | Mesma câmera/região do defeito e vistas auxiliares; sem apagar a pele ou criar buracos |
| CC03/CC01 | Faixa de encontro orientada pela derivada da seção de cada raiz/tronco | Requisito de correção, hipótese matemática | Menor quebra de direção no colar, volume fechado/orientado; sem afirmar C1 exato por render |
| CC04/planta | Preservar paredes/cristas, grafo, copa e fontes antigas | Requisito do usuário/contexto anterior | Mesmos órgãos/curvas e parâmetros da fonte, nova versão de bake explícita; escultura incompatível recusa, legados exatos |

## Diagnóstico de código e plano limitado

O colo usa envoltória convexa circular (`junction_many`), dois refinamentos e
duas relaxações. O perfil em parede muda os colares, mas o interior recebe uma
extensão harmônica de **deslocamentos**. Portanto parte da massa convexa inicial
permanece. A conectividade compartilhada já impede que seja um cilindro solto;
a aparência da borda deve ser conferida na pele, não tratada como objeto extra.

Proposta: nova versão de bake para raízes em parede, conservando leitura/bake
anteriores. Resolver posições interiores com guia de vales em torno do tronco e
lóbulos orientados pelas raízes. Preservar colares exatos e orientar a primeira
faixa interior pela derivada da seção. Guardas de fechamento/orientação/perda de
escultura permanecem; sem boolean/união universal ou remendo de câmera.

Revisão estática anterior: alteração apenas na preparação do colo, buffers
limitados à junção/quantidade de raízes; sem trabalho de geração no frame,
shader/passeGPU, rede ou novo cache global. Mesmas quotas combinadas e transação
em candidata. Não acrescentar resolução à copa. Ensaio e massa continuam modelo
reduzido, não volume constitutivo exato da nova junção. Resultados posteriores
ficam em `docs/botany-collar.md`, com análise visual e limites separados dos testes.
