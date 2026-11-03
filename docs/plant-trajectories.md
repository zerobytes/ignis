# Trajetórias e seções normais — EAF-97

Pedido: provar um cilindro curvado simples antes de retornar à árvore. Planta
contém trajetória e seções associadas; a trajetória determina centro/direção,
seções determinam forma/dimensões no plano perpendicular à tangente.

## Contrato anterior ao código

Alternativas: arco circular tem inversa exata mas não descreve o deslocamento
em S com pontas verticais. Bézier XYZ livre/frames por transporte oferecem mais
liberdade, mas exigem tratar inversas múltiplas/retornos e torção. Escolha do
primeiro recorte: Bézier cúbica nos controles XZ, Y linear estritamente crescente
pela altura do perfil. Permite curvas3D e S, sem loops ou tangente horizontal.
Não substituir o loft legado nem inferir que offsetXZ antigo virou trajetória.

`Definition.trajectory` opcional:

```
{"kind":"cubic_bezier","id":"axis","control_points_xz":
 [[0,0],[0,0],[-0.1,0],[-0.1,0]],"surface_tolerance":0.0002}
```

`C(t)=(BezierX(t),height*(t-0.5),BezierZ(t))`,0<=t<=1. Estações `at`
referem-se a t, não comprimento de arco. Controles são metros locais; ID da
trajetória permanece estável entre edições do alvo. Um vínculo legado pode
receber eixo via adopt_trajectory explícito, preservando fonte/referência.
Remover/trocar a identidade exige detach; reta usa controles constantes/lineares.
Contrato da adoção e versões em [tree-trajectory-integration.md](tree-trajectory-integration.md).
Seções podem continuar usando largura/profundidade/offset e PCHIP existentes,
agora em seu plano local normal. Altura continua extensãoY, não comprimento.

Frame explícito: rotação de menor ângulo de +Y para tangente unitária T.
Com q=1+Ty, N=(1-Tx²/q,-Tx,-Tx*Tz/q), B=(-Tx*Tz/q,-Tz,1-Tz²/q).
Ty>0 garante q>1; reta e inflexão não são singulares. O frame não é Frenet nem
transporte paralelo geral; o roll de curvas3D segue esta regra fixa documentada.
`P=C+N*x+B*z`, aplicando o mapa de seção existente em(x,z).

## Unicidade do volume: limite suficiente conservador

R envolve todo ponto de seção, incluindo offset/retângulos/elipses e tolerância
material. Δx/Δz são amplitudes dos controles; Ax/Az máximos absolutos da segunda
derivada (linear, extremos dados pelos controles). Exigir:

`mu=height²-(Δx+R)*Ax-(Δz+R)*Az > height²*1e-6`.

Para qualquer ponto P do volume, F(t)=(P-C(t))·C'(t). Pela envoltória convexa
dos controles e pelo limite R, F'=-|C'|²+(P-C)·C'' <= -mu<0 em todo[0,1].
Logo cada ponto admite no máximo uma seção normal. Como a construção fornece
essa seção, ela é única; escala positiva/seção bijetiva conservam o mapa.
Este é um critério próprio suficiente, pode recusar tubos válidos; não é teste
amostrado de curvatura nem autorização para curvas fora do domínio certificado.
Inversa usa48bisseções limitadas de F e projeção no frame, sem nearest-surface.
Pontos além dos extremos ou fora da seção são recusados; tolerância de fronteira
material existente só cobre quantização. Origem de osso também usa essa inversa.

## Aproximação da malha e preservação

Superfície analítica normal/circular; triângulos continuam aproximações finitas.
`surface_tolerance` controla erro AMOSTRADO entre interpolação da malha e mapa
analítico em três midpoints/centroide de cada triângulo lateral; tampas são
planos exatos, cuja borda já é coberta pelas laterais. Não é prova Hausdorff
global. Relatório separa esse erro do erro de reconstrução da fonte já existente.
Se falhar, aumentar resolução ou assumir explicitamente uma tolerância maior;
nenhuma alteração silenciosa de segmentos/budgets.

Domínio material/correspondência continua canônico. Vetores de escultura e
translações de animação seguem políticas físicas/proporcionaisXYZ existentes;
não há transporte implícito de vetor pela tangente ou rotação automática do rig.
Posições bind mapeiam pelo volume; pesos/material/clipes preservam os contratos.
Sem trajetória, geometria, respostas e serialização antigas ficam exatas.

Reserva local Scene64/replay68 registrada em EAF-96 após conflito63/67.
Guardas incluem definição E referência. Não inclui schemas funcionais de main;
integração futura deve unificar campos/guardas e versões, sem inferir
interoperabilidade pelo número. Renderer/Vertex/materiais inalterados.

## Checklist antes do código

1. Apenas autoria/validação, nunca tick.2. Coeficientes/frame na stack,
compilados uma vez por sample; frames avaliados por ponto/probe, mesmos vetores limitados.3. f64 temporário/f32
persistido e IDs existentes, sem dependência.4. Inversa48passos por osso,
até4probes/triângulo somente com trajetória; sem busca por vizinhos.5. Mesmos
4096v/8192t por asset e budgets compostos.6. Nenhum recurso GPU/I/O novo.
7. Dispatcher/staging existente, sem lock/thread novo.8. Sem cache permanente;
invalidação de documento existente.9. Testar frame/inversa/raio/erro/legado,
recusa atômica, preservação, versões, undo/save-load/replay/capturas.10. Custos
de autoria e critério conservador mantidos; nenhum ganho de FPS alegado.

## Uso e resultado do cilindro

Definição reproduzível em examples/native-curved-cylinder.plant.json; usar o
mesmo edit.kind put em edit_plant ou preview_plant. inspect_plant.report.trajectory
inclui margem certificada da inversa, erro lateral amostrado, tolerância e
contagem de probes. Fonte/referência/avaliação continuam visíveis na inspeção.
surface_tolerance só controla o volume base, não escultura adicionada depois.

No tubo raio2cm/altura30cm/deslocamento10cm:3922v/7840t,64cortes×56v de borda.
Erro lateral amostrado0.071614mm/28224probes, abaixo da tolerância0.2mm.
Testes de raio e normal verificam os3584v laterais, mais testes f64 da função
independente nas três formas e em curvas3D. Malha finita continua aproximada;
não interpretar precisão dos vértices como precisão de toda face.
Capturas cylinder-front/oblique/front-flat são da própria Ignis, neutras,
sem textura/bump. A vista flat expõe a tesselação em vez de depender de normais
smooth para provar forma. Aprovação visual é separada das verificações.
