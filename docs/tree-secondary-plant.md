# Segunda planta na mesma árvore — ensaio

## Avaliação anterior ao código

Continuidade de 27/10/2026: a UI segue em outra frente. Este incremento exercita
trajetória e seções no ramo `secondary-06`, atualmente uma malha estática de119
vértices ligada ao osso homônimo de `branch-editable-1`. A copa `crown-06` e suas
rametas são filhas dessa instância. Não alterar a primeira planta/assembly,
tronco, demais ramos, materiais existentes ou animações da copa.

Alternativas: converter qualquer malha automaticamente exigiria um contrato de
correspondências/inversa novo; fazer bake encerraria a edição por plantas;
compor outro tronco duplicaria o contexto. Escolha: reautorar explicitamente
apenas esse ramo com uma segunda Definition nativa, usando os cinco anéis
canônicos conhecidos para propor o eixo e as seções. O núcleo gera a malha.
O objeto antigo não tem rig ou clipes próprios. Sua geometria/irregularidade
não será apresentada como escultura transferida: é uma mudança de representação,
reversível pela transação e preservada na entrada do ensaio.

Orientar Y local pela corda entre raiz e ponta permite usar o domínio monotônico
existente mesmo num ramo bastante lateral. Ajustar controles XZ ao centro dos
anéis, com extremidades fixas. A posição da ponta usa o vínculo real da copa.
Transformações dos filhos serão expressas no novo quadro, com teste das poses
mundiais nos ticks0/60/240. Um osso de ponta explícito permite que copa/rametas
acompanhem uma edição posterior sem recalcular seus vértices.

Adicionar um pequeno relevo de escultura e rig ao novo documento como autoria
explícita, antes dos ciclos de regeneração. Conferir a fonte decorada, o repouso,
os pesos e os vínculos; não atribuir estes dados novos à malha anterior. A
união geométrica da raiz com o galho pai continua fora deste ensaio: transformar
uma malha em planta não resolve automaticamente uma junção.

## Gates e revisão estática

- Preview puro e publicação igual à previsão; undo/redo do lote de conversão.
- Duas plantas na cena; primeira entrada e assets não tocados exatos.
- Fonte do novo ramo estável em A-B-A, alteração real da curva e relevo não nulo.
- Copa/rametas e demais poses preservadas na conversão dentro da quantização
  explícita de transforms f32; ao mover a ponta, somente seus descendentes seguem.
- Rig/clipes existentes intactos; save/load, reset, replay/prefixo e pixels
  repetidos. Capturas nativas com câmera/luz/tick comparáveis, incluindo neutro.
- Medir contagens antes de publicar, sem elevar tetos ou reservar nova versão.

Checklist anterior, proporcional ao script de autoria:1.só consultas/edições,
fora do tick;2.snapshots finitos para igualdade, sem geometria calculada pelo
helper;3.IDs e anéis conhecidos explícitos, sem nearest;4.ajuste de dois controles
por eixo uma vez;5.mesmos budgets;6.capturas offscreen em paths estáveis;
7.sem threads/locks/dependências novos;8.undo/reset/load exercitam invalidação
existente;9.gates acima, sem usar contagens como aceite artístico;10.custo de
autoria síncrona e aproximação de forma mantidos. Sem alteração Rust, shader,
UI, formatos ou promessa de FPS.

## Resultado e reprodução

`python3 scripts/demo_tree_secondary_plant.py` usa a árvore mantida em
`examples/native-tree-study.scene.json` (SHA25689fe1fa2…9400). A mesma prova foi
executada sobre o ensaio mais recente de casca, com `--input
artifacts/tree-realism/examples/after.scene.json` (SHA2562a5d38b2…bd66). A saída
corrente, em artifacts/tree-secondary-plant, contém a cena before/input,
after.scene.json com duas plantas, curve-edited.scene.json e a Definition
secondary.plant.json. As capturas são da própria Ignis, sem janela.

O ramo passa de119v234t para898v1792t, com seções circulares e trajetória cúbica,
PCHIP no raio, segmentos[8,6,8] e tolerância superficial amostrada1mm. A primeira
tentativa[6,6,6] foi recusada pelo núcleo: erro1,01249mm maior que1mm; aumentou-se
a resolução, sem afrouxar a tolerância. As três posições de seção internas
mudam8,28..13,15mm em relação aos centros da representação anterior; isto é
reautoria, não conservação exata da silhueta ou do relevo antigo.

O novo rig tem ossos base/tip. Folhas e rametas existentes passam a seguir tip
com transformação local explícita; seus assets, clipes e a primeira planta
ficam exatos. Erro mundial máximo da conversão nos ticks0/60/240:0,983µm.
Mover o controle interno40mm altera o corpo mantendo as inserções extremas;
mover o controle final60mm leva os filhos com erro máximo0,731µm em relação
ao deslocamento do osso avaliado. Isso não realinha automaticamente os eixos
do rig com a tangente. As rotações globais seguem o contrato existente.

Relevo novo de1mm foi autorado explicitamente antes dos ciclos; fonte e
referência ficam exatas em A-B-A e no refino[10,8,10]→[8,6,8]. Pesos/rig se
mantêm no ensaio de curva; a primeira planta/assembly e demais assets não são
reautorados. Preview/lote/undo/redo, erro sem mutação, save/load/reset e pixels
repeat/replay/prefixo passaram. O ramo isolado é fechado e conectado, mas sua
raiz ainda sobrepõe o pai: não afirmar que a árvore toda foi soldada.

Documento avaliado:22888v39690t,88234t desenhados, sem ampliar tetos. Material
do novo ramo usa a mesma receita, com canal reautorado; o domínio antigo não
foi transferido implicitamente. A casca ainda apresenta distorção e o encaixe
continua com aparência de peça inserida. Próximo problema: junção regenerável
entre duas plantas vivas, sem duplicar o pai nem encerrar sua edição. As fontes
de contexto estáticas atuais não resolvem essa dependência automaticamente.

Validação no build05454b395586158e, macOS Apple Silicon:847testes Rust/84grupos
release/all-targets, Clippy-Dwarnings;7cenários MCP/37grupos (lab,render,
replay_inspection,plants,mesh_attachments,plant_trajectories,plant_assemblies).
Prova própria de autoria nos dois inputs acima; logs secondary-plant-* em
artifacts. Python compila e diff/links verificados. Sem alteração Rust, UI,
renderer, formato, binário, main, demo ou push. O exemplo de Definition acompanha
o script; não contém a escultura/rig ou a cena completa.

Checklist posterior:1.adequado, apenas autoria;2.mantidos snapshots finitos para
comparar fontes/poses, nenhum payload novo no tick;3.adequado, IDs e anéis
explícitos;4.ajuste pequeno uma vez;5.contagens acima admitidas pelos limites
existentes;6.capturas em paths correntes sem GPU nova;7.nenhum lock/thread novo;
8.undo/reset/load/replay exatos;9.gates exercitados sem confundir tolerância de
transform f32 com igualdade de fonte;10.mantidos o ajuste aproximado e a
interseção raiz-pai aberta. Sem medição/alegação de FPS ou aceite de realismo.
