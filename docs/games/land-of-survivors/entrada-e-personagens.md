# Entrada e criação de personagens no Ignis

LOS-22 · 5 de novembro de 2026.

A entrada usa o palco existente de200×200m e uma aplicação nativa Rust/egui,
separada do editor. O jogo usa o compositor e renderizador do Ignis; seus estados,
identidades e telas ficam em `games/land-of-survivors/src/front_*.rs` e
`characters.rs`. O laboratório das284 habilidades permanece separado.

```sh
python3 games/land-of-survivors/app.py --build
python3 games/land-of-survivors/app.py --open
python3 games/land-of-survivors/app.py --play
python3 games/land-of-survivors/app.py --verify
```

`start.py --open` também abre o jogo. `start.py --editor` continua abrindo o palco
para autoria; sem opções, verifica sua cena-base. A cena planetária permanece
separada. Nenhum mundo planetário é ativado pela tela de personagens.

## Fluxo e identidade

Menu → personagens locais → criar → confirmar → selecionar. Cancelar não cria
identidade. O cadastro contém ID aleatório estável, nome, Alto Elfo/Orc, base
feminina/masculina, classe, facção e escolhas discretas de aparência. As quatro
combinações de povo/base são identidades; não se declara quatro modelos finais
aprovados. A prévia usa a fonte nativa existente: corpo, orelhas e rosto; pele e
olhos recebem as opções do jogo. A representação Orc é provisória. Cabelos,
roupas/equipamentos finais e animações não foram importados ou construídos aqui.
A prévia é enquadrada como retrato.

Classes iniciais: Mago, Guerreiro, Arqueiro, Druida. Facção é uma escolha separada
entre Elfos/Orcs. Não foi criada uma regra de hostilidade automática entre elas.
Especializações, focos, barras e execução do catálogo ainda pertencem ao
laboratório; esta tela escolhe a identidade de classe e não conecta o combate
completo ao multiplayer. Aparência pode ser editada depois sem alterar nome,
ID, povo, corpo, classe ou facção. Não há sliders livres de porte/musculatura.

Inglês é padrão, com seletor EN/PT-BR persistido. Erros provenientes do núcleo
podem permanecer em inglês. O logo original é reutilizado sem edição por pedido
explícito do usuário: `assets/ui/provenance.json` registra origem e SHA256.
Essa autorização é específica para o logo, não libera outros assets/código Unreal.

## Persistência

No macOS o cadastro é salvo em
`~/Library/Application Support/Ignis/LandOfSurvivors/characters.json`.
`LOS_PROFILE_PATH` permite um arquivo explícito para outra instalação/ensaio.
O arquivo é versionado, validado, limitado a1MiB/64personagens e substituído
atomicamente. Nome é único neste cadastro, sem promessa de unicidade global.
Arquivo inválido não é recriado por cima; a tela apresenta o erro e desabilita
a criação. Mudança externa é detectada antes de salvar; a janela deve reabrir
o cadastro. Um lock exclusivo durante a gravação evita escritores simultâneos;
se um processo for encerrado abruptamente e deixar `.json.lock`, recuperar o
lock apenas após confirmar que não há escritor ativo.

Cada personagem offline tem um save portátil separado em `saves/<ID>.ignis-portable`
no diretório do cadastro. Sair da sessão neutraliza o input e salva o estado
antes de voltar à lista. Um lease por ID impede dois escritores offline do mesmo
personagem; ele é liberado ao sair. Um `.lease` remanescente após encerramento
forçado só deve ser recuperado depois de confirmar a ausência de escritor ativo.
Erro de gravação mantém a sessão aberta com diagnóstico.
Isso não é sincronização de conta/nuvem nem um checkpoint do planeta.

A fonte de prévia corrente é
`artifacts/character-base/human-nose.ignis-character`, produzida pelo pipeline
nativo já existente em `scripts/character/`. O launcher não baixa dados nem
reconstrói essa fonte por frame. `--build` prepara a cadeia local de fontes
somente se a fonte final estiver ausente; os dados nativos de entrada precisam
estar presentes no checkout. A fonte é carregada/validada uma vez e a
compilação ocorre somente após uma escolha de aparência. Se estiver ausente,
a aplicação informa a indisponibilidade e não confirma uma nova criação sem
prévia. O SHA da fonte usada fica no registro; migração artística entre fontes
futuras ainda precisa ser definida.

## Sessões offline e multiplayer

O palco permite testar movimento com WASD/Shift usando o modelo nativo do
personagem selecionado, na pose de repouso e ainda com arte provisória.
A câmera acompanha esse personagem. `--play` abre diretamente o personagem
selecionado offline; sem cadastro selecionado, abre a criação. Offline usa o
mesmo núcleo Ignis e save portátil por personagem. Hospedar inicia um
host QUIC/TLS real com8slots e conecta o jogador por um cliente separado dentro
da aplicação. Outros apps podem entrar com o convite copiado, que contém
endereço e certificado público fixado. O convite não contém chave privada nem
segredo de retomada. Use um IP local específico para LAN;127.0.0.1 é loopback.
Firewall, acesso pela internet, infraestrutura e qualidade em WAN não são
estabelecidos pela verificação local.

O protocolo atual atribui um Player livre já presente no roster do host. A
identidade local escolhida não constitui autenticação, posse durável de conta
ou transmissão completa de sua aparência ao servidor. Os participantes de
rede são temporários; o progresso da sessão de rede não é salvo como progresso
permanente do personagem. Isso é informado na tela antes de conectar. Não há
API administrativa exposta para o cliente alterar livremente o roster do host.
Entrada e saída de rede usam as APIs de sessão existentes, com pin TLS e
controle escopado; fechar o host encerra os participantes da sessão.

## Verificação e limites de entrega

O verificador usa um cadastro temporário, nunca os personagens do usuário.
Exercita cliques e texto pelo mesmo egui da aplicação, além de criação/cancelamento,
reabertura, movimento e save offline, host e dois clientes QUIC. Capturas nativas
e recibo ficam em `games/land-of-survivors/artifacts/app/`, substituídos nos
mesmos caminhos. Os testes de cadastro protegem identidade, edição cosmética e
recusa de corrupção. Evidência final e limitações devem acompanhar a entrega;
compilação ou testes não constituem aceite humano de arte/interface nem prova
em duas máquinas físicas.

O build corrente é `target/release/examples/land_of_survivors_app`. Não há
bundles/copias por revisão, instaladores, push ou publicação nesta unidade.

Verificação desta entrega: build release e Clippy do exemplo com `-D warnings`
aprovados; teste crítico do cadastro aprovado. O verificador nativo completou
criação/cancelamento por cliques e texto egui, reabertura da identidade, recusa
de segundo escritor offline, movimento e save/load portátil, host com dois
clientes QUIC/TLS e movimento remoto aplicado pela autoridade. Capturas de
menu, lista, criador e sessão foram inspecionadas. Aceite humano e uso em duas
máquinas físicas permanecem separados.


## Personagem dentro da cena — continuação de LOS-22

A criação e a sessão compartilham `Front::character_meshes`: povo, base, rosto,
pele, olhos e orelhas usam a mesma configuração nativa. `presentation.rs` prepara
os vértices/materiais da pose de repouso uma vez ao entrar; cada frame aplica a
posição física, escala para a altura do colisor e orientação pelo deslocamento.
Pés coincidem com a base física. Não foram criadas animações, roupas ou arte nova.
A câmera segue o ator selecionado e preserva os controles WASD/Shift existentes.

O snapshot do Ignis agora expõe `player_primitives` com identidade/vida/índice
exatos das caixas de Player, tanto offline quanto na réplica recebida. A camada
do jogo substitui somente essas representações após preparar o frame nativo,
sem alterar colisores, Scene, PSV ou estado autoritativo. Piso, outros objetos e
projéteis permanecem visíveis. Os modelos são derivados locais, não ficam
embutidos no save. Este recorte exige o palco sem meshes autorais da entrada;
não constitui um sistema genérico de attachments ou integração planetária.

Na rede a própria aparência acompanha o Player concedido pelo host; outros
Players usam uma base nativa de viajante provisória. Isso não transmite a
aparência escolhida para os demais clientes nem cria posse durável. O vínculo
usa a identidade pública do snapshot, não posição, cor ou ordem presumida das
caixas. Os oito Players pré-autorizados do palco continuam presentes; não são
oito conexões nem NPCs com IA.

A validação deste incremento acrescenta captura de personagem na cena/ao mover,
alinhamento dos pés, ausência da caixa visual, câmera seguindo, contagem de
vértices estável, save sem meshes derivados e vínculo correto ao ator recebido
para dois clientes com configurações diferentes. A aparência remota completa,
animações e aceite humano permanecem etapas separadas.
