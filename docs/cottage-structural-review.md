# Revisão estrutural da casa — EAF-101

## Escopo e método

Pedido de29/10/2026: identificar autonomamente as diferenças estruturais em relação à imagem, sem pistas do usuário e sem confundir a questão com qualidade da escultura. Esta rodada entrega diagnóstico e lista de correções; nenhuma geometria, câmera, material ou ferramenta foi alterada.

Apliquei ignis-image-to-3d e seus contratos de análise/recortes/lista/estrutura. Abri primeiro as duas imagens completas e oito recortes reais, sem consultar as análises antigas da casa. Só depois consultei a receita e as funções atuais para distinguir uma observação visual de uma escolha autoral. O contexto anterior da conversa continua conhecido; isto não é um novo ensaio cego nem uma reconstrução sem conhecimento da implementação.

Base candidata: captura nativa do conjunto integrado066f897, buildd092a8fec0ed01f0. A main contém alterações paralelas de UI nos arquivos editor_mesh_ui/editor_ui/editor_window/performance/workflows; todas preservadas. Esta revisão não recompila nem controla a janela.

## Fontes e evidência

- Referência1536×1024: Imagem do ChatGPT23deoutde2026,20_54_15.png, SHA256209cd5cb91c216100413d6b175b5ce29a70cab751a0eda710e4baf95895c6553.
- Candidato1536×1024: artifacts/cottage-normals/artifacts/joined-hero.png, SHAcc68683839efa6aaa6f7bafbe8ac38b8d48f2423ad3e9e958ed7703a30e44349.
- Câmera candidata: position[11,8.7,16],target[0,2.65,0.5],fov34. Câmera da referência desconhecida.
- Manifesto/crops: artifacts/cottage-structure-review/manifest.json e reference/candidate-{roof,entrance,bay,porch}.png.

Origem dos pixels no canto superior esquerdo, X à direita e Y para baixo. As caixas abaixo são(x,y,largura,altura), iguais nas duas imagens, sem resize/rotação/translação. As câmeras não estão calibradas: a revisão é qualitativa, sem erro de reprojeção, escala física, SSIM ou percentual de fidelidade. Diferença de enquadramento não aprova nem reprova sozinha uma dimensão3D.

| Ficha | Caixa | Pergunta | Segunda leitura após abrir ambos os recortes |
|---|---|---|---|
| R1 |50,0,1450,510|Organização das coberturas|Confirma que a cobertura do anexo tem peso e perfil diferentes; a direção projetada da cumeeira também diverge, mas não permite concluir rotação de90graus nem altura real sem câmera/planta.|
| R2 |610,330,440,600|Entrada, apoios, reentrâncias e ligação ao anexo|A referência articula acesso e anexo por uma transição compacta; o candidato mostra um pano cego largo e uma esquina muito exposta entre eles. A posição/profundidade do anexo e do conjunto da entrada precisa ser revista, não só o arco.|
| R3 |970,270,550,630|Corpo envidraçado, faces e cobertura|Na referência o anexo lê como bay window largo sob cobertura baixa apoiada no frontão; o candidato lê como pequeno volume vertical com cobertura pontuda. Não concluir número faltante de janelas: a receita tem4faces e a câmera esconde parte delas.|
| R4 |70,390,660,530|Ritmo da varanda, vãos e profundidade|As3arcadas principais estão presentes, mas o candidato tem vãos projetados mais estreitos/verticais e os apoios pesam mais no conjunto. Recessos e espaço coberto ficam menos legíveis. A câmera participa desse efeito; a proporção física ainda precisa de conferência controlada.|

## Planta existente, não uma planta inferida da fotografia

Unidades autorais, sem converter pixels em metros. EixoY vertical;X percorre a fileira de arcadas eZpositivo aponta para o acesso externo. Os dados seguintes vêm de examples/elven-cottage.recipe.json e scripts/demo_elven_cottage.py, não são medidas da referência.

| Peça/face | Localização/orientação autorada | Relação a conferir |
|---|---|---|
| Corpo principal |X[-4.6,3.6],fundoZ=-2.6; fachada da galeria emZ=1.1 e trecho da entrada emZ=2.1|A quebra entre esses planos precisa ter retorno/fechamento coerente; a simples presença dos painéis não valida o corpo.|
| Varanda coberta |X[-4.6,-.05],entre paredeZ=1.1 e alinhamento de pilaresZ=3.15; normal externa+Z|Profundidade do espaço aberto, proporção dos vãos e relação com a parede hospedeira.|
| Acesso externo |ArcoX[-.05,2.95],planoZ=3.15,normal+Z; escada se aproxima por+Z|Não confundir arco de acesso com porta da casa.|
| Porta |CentroX=1.3; parede hospedeiraZ=2.1,normal+Z; folha emZ≈2.265|Reentrância, espessuras e percurso sob cobertura, sem supor posição exata da face na referência.|
| Bay window |Face hospedeiraX=3.6,normal+X; centroXZ=[3.6,-.65],raio1.65,4faces em semicírculo|Seu limite frontal éZ=1.0,atrás da parede de entradaZ=2.1 e dos pilaresZ=3.15. Conferir se esse recuo reproduz a transição observada.|
| Cobertura principal |Cumeeira paralela aX,altura nominal5.95,Z=-.65; beiral dianteiro recebe elevação local3.72→4.85|A receita assume cumeeira nivelada e levanta parte do beiral. Essa regra não foi demonstrada como equivalente ao telhado da referência.|
| Cobertura do anexo |Perfil radial; ápice nominal5.25,beiral3.70,raio externo1.90|A hipótese quase cônica está construída, mas sua correspondência ao perfil mais baixo da referência está divergente.|

A fotografia sustenta a adjacência varanda/acesso/anexo e a hierarquia de coberturas; não fornece sozinha todos os comprimentos, planos ocultos, traseira ou planta fechada. Não declarar que a porta ou o bay estão na face errada apenas porque o desenho projetado diverge. A revisão deve testar essa hipótese com evidências; os recortes atuais sustentam a divergência da articulação, não uma planta alternativa única.

## Lista de correções estruturais

Agrupadores: H-COB(coberturas),H-ESP(espaços/acesso),H-VAR(varanda),H-BASE(apoio). A lista abaixo é específica desta revisão estrutural; escultura, tracery fina, materiais e vegetação continuam fora do pedido atual, não foram apagados do escopo artístico geral. Estado de execução significa correção nova, não inexistência da peça já modelada.

| ID/pai | Correção e evidência | Estado epistemológico | Critério de conferência | Execução/validação |
|---|---|---|---|---|
| H01/H-COB |Rever corpo e cobertura do bay;R1/R3. A referência mostra conjunto mais largo/baixo, o candidato termina em cobertura alta/pontuda.|Divergência visual observada; dimensões3D corretas ainda hipótese.|Conferir perfil completo, altura relativa de cobertura/janelas e ligação ao frontão em câmera comparável e massa neutra.|Não corrigido; estrutura existente revisada em lateral/hero neutras no EAF-129, sem aceite.|
| H02/H-COB |Reconstruir a relação entre pano principal, frontão e cobertura do anexo;R1/R2/R3. Não tratar os três como formas independentes que apenas se sobrepõem.|Divergência visual observada; solução de superfícies pendente.|Silhueta e encontros de bordas/cumeeira/beirais coerentes no conjunto e em vistas distribuídas.|Não corrigido; vistas distribuídas da hipótese existente no EAF-129, sem novo encontro geométrico.|
| H03/H-ESP |Rever posição e recuo do anexo relativamente à entrada;R2/R3. O candidato expõe grande pano cego/esquina entre porta e janelas.|Pano visível observado; profundidade/orientação corretas da referência são hipótese.|Transição compacta e percurso entre acesso/porta/anexo coerentes em planta e na vista da referência.|Não corrigido; planta/frente existentes revisadas no EAF-129, sem hipótese alternativa validada.|
| H04/H-ESP |Conferir arco externo, porta e seus planos hospedeiros separadamente;R2 e planta existente.|Existência dos planos atuais é fato autoral; deslocamento/rotação corretivos não determinados.|Planta e seção explicitam alinhamento, reentrância, apoios e vazio da passagem; não corrigir por câmera isolada.|Não corrigido; planos existentes expostos no EAF-129, sem calibração da referência.|
| H05/H-VAR |Rever largura/altura dos3vãos principais, espessura relativa dos apoios e profundidade da galeria;R4.|Divergência projetada observada; câmera é confundidor.|Razões entre elementos da mesma face e espaço aberto comparados sob enquadramento registrado; nenhuma coluna extra/faltante inventada.|Não corrigido/não executada.|
| H06/H-BASE |Rever continuidade de embasamento/deck/escada/apoio do bay;R2/R3/R4.|Junções e grandes patamares visíveis no candidato; extensão da base oculta na referência é desconhecida.|Vistas superior/laterais demonstram cobertura/apoio e contato entre volumes, preservando vazios legítimos.|Não corrigido/não executada.|
| H07/H-COB |Conferir hipótese da cumeeira e da elevação do beiral principal;R1 e regra atual da receita.|Inclinação2D observada; não prova cumeeira fisicamente inclinada nem eixo girado.|Âncoras de planos baseadas em geometria independente, câmera explícita e perfil longitudinal/transversal; separar tentativa de câmera de geometria.|Não corrigido; projeção dos mesmos extremos medida em4câmeras no EAF-129, sem câmera calibrada.|

## Revisitação do conjunto e diagnóstico

Reabri o conjunto depois dos recortes. A conclusão principal permanece: o candidato reconhece os componentes, mas não reproduz as mesmas proporções e articulações. Telhas, relevos e materiais melhoraram partes de uma hipótese estrutural que ainda não estava aceita. Testes de topologia, persistência, pixels repetidos e replay não provavam a correspondência arquitetônica. Os11checks da integração não constituem aprovação estrutural ou artística.

Não usar câmera como explicação universal nem atribuir toda diferença à geometria. O anexo quase cônico e o pano cego/transição são prioridades; direção da cumeeira, quantidade de faces visíveis e proporções projetadas exigem controle de câmera antes de uma conclusão física. Não inventar largura real, traseira, portas ocultas ou número total de telhas.

Próxima sequência: planta esquemática da hipótese de referência, com ambiguidades; massa simples sem ornamentos; vistas superior/frontal/laterais e reenquadramento comparável; correção de relações/volumes; somente então retorno ao detalhe. As ferramentas atuais já permitem esse teste de massa. Nenhuma limitação de orçamento justifica aceitar a hipótese errada ou remover itens da lista. Nesta rodada não se executou a correção nem se aprovou a massa.

## Continuação: controle de câmera e isolamento estrutural — EAF-129

O pedido posterior de Guilherme retoma desenvolvimento de ferramentas a partir
das dificuldades do exercício. `project_landmarks` agora consulta pontos3D
explícitos em câmera candidata sem editar o documento. Contrato, alternativas,
custos e ensaio em [landmark-projection.md](landmark-projection.md).

Quatro câmeras do mesmo mesh,10âncoras,1536×1024: os extremos da cumeeira
passam de ΔY=+54.709pixels na câmera [11,8.7,16] para−48.855pixels em
[11,3.5,16], mantendo target[0,2.65,.5]/FOV34°. H07 ganhou um controle
mensurável: inclinação2D sozinha não determina inclinação3D. Não se resolveu
a câmera da referência nem se aprovou qualquer uma das tentativas.

Também foram abertas/revistas vistas hero, superior, frontal, lateral e
traseira da estrutura isolada, em artifacts/cottage-camera-study/artifacts/
mass-{hero,plan,front,side,rear}.png. Fonte preservada,6assets/11instâncias
retidos com a mesma geometria. Materiais neutralizados e detalhes de outros
assets removidos; ornamentos de pedra compartilhados com apoios ainda estão
presentes. Infills opacos e vista superior perspectiva são limitações
deliberadas desse diagnóstico, não uma massa nova aceita.

H01/H02: a lateral deixa clara a separação entre cobertura quase cônica do
anexo e frontão principal. H03/H04: planta e vista frontal explicitam o recuo
do anexo e os planos do acesso/porta. Isso documenta a hipótese existente;
não comprova que esses planos correspondem à referência. H05 permanece sem
proporção física calibrada. H06 permanece sem correção/aceite de continuidade.
**Nenhum H01..H07 está corrigido ou artisticamente aprovado por este ensaio.**

Gap concreto de operação: selecionar materiais para retirar detalhes também
retira partes estruturais. Seleção por componentes/ilhas já existe; preservar
os agrupadores construtivos entre materiais é outro problema. Avaliar conjuntos
nomeados e política de remapeamento antes de criar contrato persistido. A
correção de massa do anexo/entrada ainda é o próximo trabalho de conteúdo.

## Verificação e revisão estática proporcional

Documento e recortes de análise: nenhum código de runtime/MCP/renderer/Scene/replay/dependência novo; nenhum loop/tick/GPU/lock/cache alterado. Memória/I/O apenas na inspeção offline de8crops limitados aos1536×1024pixels de origem, em caminhos estáveis. Asserções de tamanho/hash e existência dos arquivos e leitura do manifesto conferidas. Sem métricas inexistentes nem nova comparaçãoSSIM. Sem testesRust novos para esta alteração documental; nenhuma janela ou build iniciado. Arquivos de UI em trabalho paralelo não foram editados ou stageados.
