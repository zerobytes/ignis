# Land of Survivors GDD

Versão 1.3 · 4 de novembro de 2026 · Desenvolvimento no Ignis.

Land of Survivors é um RPG 3D de fantasia clássica, com exploração de um planeta esférico finito do tamanho da Terra, evolução do personagem, combate PvE/PvP e histórias com consequências permanentes. Preserva a lore, a história e as decisões não técnicas do projeto anterior. Toda a construção do jogo no Ignis será nova.

## Escopo preservado

Permanecem os povos, personagens, criaturas, histórias, classes, habilidades, regras de jogo, personalização, progressão, equipamentos, comércio, exploração, viagens e aventuras por IA. Os catálogos completos de [lore e criaturas](lore.md) e [habilidades](habilidades.md) fazem parte deste GDD.

Nenhum asset anterior será reaproveitado: modelos, meshes, construções, terrenos, cenas, materiais, texturas, imagens, interface pronta, animações, rigs, áudio ou efeitos. Código, geradores, receitas de construção, ferramentas e soluções técnicas anteriores também ficam fora. Uma vila, um portal ou um Guarda de Pedra permanece como conceito do universo, sem importar sua construção anterior.

As regras descritas como definidas orientam o produto. Os conceitos identificados como propostas e os valores de balanceamento provisórios permanecem disponíveis para desenvolvimento, sem se tornarem decisões finais. O GDD especifica o jogo desejado; não declara esses sistemas implementados no Ignis.

## Experiência e pilares

O jogador cria uma identidade, explora lugares desconhecidos, encontra povos e criaturas, participa de conflitos, melhora suas capacidades e deixa acontecimentos que o mundo conserva. O ciclo de experiência combina explorar, conhecer personagens, assumir aventuras, enfrentar desafios, receber recompensas e retornar a um mundo transformado.

- Exploração contínua de um planeta finito, sem fronteiras de cena impostas ao jogador.
- Personagens reconhecíveis por identidade, classe, aparência e equipamento.
- Combate legível, com preparação, execução, impacto e recuperação compreensíveis.
- Diversidade de estilos de jogo, incluindo dano, cura e proteção em todas as especializações.
- Continuidade das pessoas, dos lugares, das escolhas e das consequências.

World of Warcraft permanece como referência para câmera, alvo, barra de habilidades, interação e quests. Genshin Impact orienta o estudo de geografia, relevo e composição regional. A nova decisão de mundo esférico substitui a expansão infinita inspirada em Minecraft; a continuidade da exploração e a permanência das histórias são preservadas. Referências não determinam automaticamente escala MMO, mineração ou construção livre.

## Universo e história

O mundo reúne civilizações de Altos Elfos e Orcs, natureza extraordinária e vestígios de épocas anteriores. A magia faz parte dessa fantasia clássica. A cosmologia, os nomes definitivos das facções e uma campanha principal ainda não estão definidos.

Os Ents são os primeiros seres pensantes do mundo. Vivem há milhares e milhares de anos e testemunharam guerras, destruição, vida e criação. São respeitados, imponentes e seguros; seus rostos pertencem à madeira viva. As seis formas de referência são carvalho, salgueiro, sequoia, baobá, oliveira e araucária.

Os Criadores deixaram Guardas de Pedra anteriores às civilizações atuais, feitos para guardar o mundo. Têm espada, escudo, olhos luminosos e presença defensiva. A identidade dos Criadores e sua relação com os Ents continuam abertas. Quatro conceitos de guardião permanecem: Baluarte dos Limiares, Vigília das Nascentes, Égide dos Caminhos e Colosso dos Selos.

A fauna élfica combina anatomia animal e flora viva. A natureza deve despertar encanto por sua presença e comportamento, além da cor. O Pégaso do Arco-Íris constitui uma exceção explícita: não tem chifre nem flora integrada. O catálogo narrativo detalha dezoito entradas de fauna e as funções propostas dos quatro guardiões.

Esta continuidade corresponde a Land of Survivors. A lore de Land of Chaos que já havia sido descartada continua histórica; arena com bola, gols e placar não integra o produto. Não se acrescenta uma nova cronologia ou conflito central para preencher lacunas da história.

## Personagens jogáveis

As espécies iniciais são Altos Elfos e Orcs. Cada uma possui uma raça inicial, sem seletor de raça, e duas bases corporais, masculina e feminina: quatro bases ao todo. Espécie, base corporal, aparência, classe e facção são escolhas distintas.

O jogador escolhe o lado dos Elfos ou dos Orcs. Restrições por espécie, alianças, reputação e regras de hostilidade/PvP continuam abertas; escolher um lado não torna automaticamente o outro inimigo.

As bases têm proporções estáveis, com opções discretas de rosto, pele, olhos, cabelo, pelos faciais e acessórios adequadas a cada povo. Não há o antigo criador livre de peso, musculatura e altura. A força élfica não depende de musculatura ostensiva; a direção feminina preserva feições suaves e proporções naturais.

Cada personagem conserva nome, identidade, espécie, base, aparência, classe, facção, progressão, builds, inventário e posição de retorno. Nome, base, classe e facção pertencem à criação. A edição cosmética em jogo altera somente a aparência.

O fluxo de entrada apresenta menu, lista de personagens e criação. Confirmar cria a identidade antes da entrada; cancelar não cria um personagem. Editar aparência permite confirmar ou restaurar o estado anterior. Trocar de personagem preserva o progresso individual e os acontecimentos do mundo compartilhado.

## Classes e estilos de jogo

| Classe | Especializações iniciais | Identidade |
| --- | --- | --- |
| Mago | Fogo, Gelo e Éter | Magia ofensiva, controle, restauração e proteção conforme o foco. |
| Guerreiro | Armas | Combate marcial, proteção, medicina de campo e apoio por liderança. |
| Arqueiro | Pontaria | Combate à distância, ferramentas de exploração, apoio e proteção. |
| Druida | Natureza | Magia natural e transformação em Urso ou Felino. |

Armas, Pontaria e Natureza são especializações iniciais, sem impedir expansão futura. Todas as especializações oferecem os focos DPS, Healer e Tank. A cura do Guerreiro não depende de causar dano: medicina de campo e apoio incluem cura direta, periódica, limpeza, proteção e cura em área.

Classe é uma escolha de identidade. Especialização, foco e barra podem mudar fora de combate. Cada combinação especialização/foco oferece pelo menos vinte habilidades ativas, com dez equipadas. Habilidades compartilhadas são a mesma habilidade disponível em vários focos.

O catálogo herdado de design contém 44 habilidades humanoides por especialização: oito compartilhadas, doze DPS, doze Healer e doze Tank. Cada foco acessa vinte opções. São 264 habilidades humanoides e vinte de formas animais, totalizando 284 entradas. Nomes, descrições e efeitos estão preservados no anexo; números são balanceamento inicial.

Cada combinação lembra sua própria barra. Trocas não restauram vida/mana nem apagam recargas. A regra herdada exige estar vivo, desmontado, na forma original, sem habilidade ativa e há pelo menos oito segundos sem ação de combate ou dano recebido; esse intervalo é um valor provisório de balanceamento.

## Atributos e progressão

Os atributos principais são Força, Destreza, Vigor, Inteligência e Espírito. O total combina base da espécie, base da classe, crescimento por nível e pontos distribuídos pelo jogador. Esses atributos influenciam poder de ataque, cura, armadura, recursos máximos e regeneração.

O nível, a experiência acumulada, os pontos livres, as escolhas de atributos e as barras pertencem ao personagem. A ficha explica a origem dos valores. A parametrização anterior começa com cinco pontos livres no nível um, concede três por nível e usa sessenta como teto provisório. Esses números, as bases e o crescimento por classe são referências de ajuste, não limites finais do produto.

Recompensas precisam respeitar a progressão e as escolhas do personagem. Ritmo de experiência, obtenção de equipamento, raridades e curva econômica ainda requerem definição.

## Combate e habilidades

O combate combina seleção explícita de alvo, mira direcional e seleção de ponto no chão. A barra possui dez posições, associadas inicialmente às teclas 1–9 e 0. A ficha usa C; menus e seleções pendentes aceitam Escape.

Habilidades direcionais usam a mira central. Habilidades de chão mostram uma área antes da confirmação; cancelar essa escolha não consome recurso nem inicia recarga. Confirmar precisa respeitar alcance, superfície, visibilidade, recursos e disponibilidade. Um projétil continua viajando até o contato; uma mira não o transforma automaticamente em magia guiada.

O repertório preserva dano, cura, escudos, mitigação, controle, lentidão, imobilização, limpeza de efeitos, restauração de recurso, provocação, deslocamentos, áreas persistentes, cones, encadeamentos e múltiplos disparos. Tempo de conjuração, custo, recarga, duração e seleção de aliados/inimigos fazem parte do design de cada habilidade.

O [componente funcional das 284 habilidades](habilidades-runtime.md) executa esse catálogo no Ignis em um laboratório de combate por terminal/MCP, sem animações ou HUD gráfico. Valores não definidos no design têm parametrização provisória documentada. A integração com personagens finais, a interface gráfica e o mundo planetário permanece pendente; ter o catálogo executável não declara o RPG completo.

O dano considera armadura, mitigação temporária e escudos antes da vida. Cura, efeitos periódicos e proteção mantêm destinatários próprios. Efeitos de aliados e inimigos podem coexistir na mesma área sem aplicar o mesmo resultado a todos.

### Magias obrigatórias

- Bola de Fogo: conjuração seguida de projétil, explosão e chão incendiado; aliados não recebem seu dano.
- Seta de Gelo: projétil pontiagudo de gelo.
- Chão Congelado: imobilização dos pés dos inimigos no solo.
- Círculo de Fogo: causa dano a inimigos e cura aliados simultaneamente, disponível nos três focos de Fogo.

### Formas do Druida

Urso e Felino possuem dez habilidades próprias cada, incluindo retorno à forma original. Urso favorece armadura; Felino favorece velocidade. Transformar não oferece cura ou reposição gratuita de mana. Montar transformado é bloqueado; morrer restaura a forma original antes do retorno após morte.

### Morte e retorno

Morte e respawn fazem parte do jogo. O retorno precisa levar o personagem a um local seguro. O último ponto de solo seguro e a chegada inicial são referências existentes; regras finais de cemitérios, penalidades, perda de itens e relação com facções permanecem abertas. Não se herdam automaticamente as regras de equipes da arena antiga.

## Equipamento e inventário

Roupas, armaduras, armas e acessórios alteram a aparência do personagem. A identidade do item permanece a mesma entre bases corporais; variações de encaixe não criam itens de gameplay distintos. Bônus devem ser aplicados uma única vez, independentemente de como a aparência é construída.

O corpo é preservado sob as peças. Há continuidade entre a aparência escolhida, os itens equipados e o que aparece durante movimento e combate. A aquisição de um bem no comércio não equivale automaticamente a equipá-lo ou consumi-lo.

Slots finais, restrições por classe, requisitos de nível, raridades, durabilidade, crafting e efeitos completos dos consumíveis continuam abertos. O comércio de referência já distingue provisões, matérias-primas, armas, armaduras e remédios; seu conteúdo está no anexo de lore e itens.

## Exploração e mundo

O mundo é uma esfera finita com raio de referência de 6.371 km e diâmetro de 12.742 km, equivalentes ao tamanho médio terrestre. Sua circunferência de referência é aproximadamente 40.030 km. É possível contornar o planeta e retornar ao ponto de partida; não há geração de terra infinita além do globo. Isso não exige reproduzir os continentes da Terra.

Terreno, biomas, árvores, rios, lagos, mar, ilhas e montanhas formarão uma geografia conectada. A semente continua sendo uma ferramenta de autoria e identidade do mundo, sem mudar seu tamanho finito. Lugares visitados e mudanças do jogador continuam existindo quando ele se afasta e retorna.

A topografia terá Genshin Impact como referência de linguagem geográfica. Primeiro se estuda como suas regiões combinam planícies, cristas, vales, patamares, costas e bacias; depois se compõe uma geografia própria para Land of Survivors. Não se copia o mapa de Teyvat nem se amplia sua escala de maneira uniforme para preencher o planeta. O estudo e as decisões pendentes estão em [Geografia e relevo](geografia.md).

Direção de autoria definida em 4 de novembro de 2026: o relevo deverá permitir geração procedural e escultura manual combinadas. As ferramentas usadas pelo gerador também serão utilizáveis manualmente, incluindo controle de região, máscaras e intensidade. Será possível gerar uma base, esculpir ajustes e voltar à geração preservando o trabalho manual; esse requisito de autoria não define escavação ou deformação pelo jogador durante o gameplay. O contrato e o estado da implementação estão no [diagnóstico do protótipo](prototipo-relevo.md#procedural-e-escultura-compartilham-as-ferramentas).

O [projeto nativo](../../../games/land-of-survivors/README.md) contém uma região original de [protótipo de relevo](prototipo-relevo.md), comparando cristas suaves e patamares na mesma composição. Ela serve para avaliar a linguagem geográfica e as ferramentas, sem replicar localizações de Genshin. Continentes, oceanos e hidrografia global continuam pendentes; o restante da esfera ainda não tem composição geográfica. Em 4 de novembro, o trabalho do mundo foi deixado temporariamente de lado para implementar as habilidades em um laboratório separado. Personagens finais, vilas e vegetação não foram construídos nessa etapa.

Os biomas iniciais são floresta temperada, pradaria, floresta úmida, deserto, tundra, terras alpinas, costa e oceano. A cultura de um povo não é determinada automaticamente pelo bioma. A construção anterior desses ambientes não será reutilizada.

O jogador caminha, salta, nada e pode explorar por voo com montaria. Natação segue a orientação da câmera e permite subir na água. Correnteza física, barcos, fôlego, afogamento, escavação, blocos destrutíveis e cavernas volumétricas continuam abertos.

Câmera em terceira pessoa com zoom e órbita: arrastar com o botão esquerdo observa sem obrigar mudança de direção; segurar o direito orienta o personagem pela câmera. Cliques de seleção e confirmação precisam ser distintos do gesto de órbita.

O ciclo de dia/noite altera a hora e a iluminação do mundo. A duração do ciclo é ajustável; duração final e efeitos sobre criaturas, comércio e quests ainda precisam de definição.

## Povoados e comércio

Altos Elfos e Orcs possuem povoados culturalmente distintos, com nomes, famílias, moradores, histórias e serviços persistentes. Orcs são também artesãos estabelecidos, com trabalho de pedra, madeira e metal; não se resumem a acampamentos primitivos.

Pequenos povoados, vilas e cidades variam em porte. A chegada inicial favorece uma comunidade menor. Ruas, acessos e entradas precisam servir à exploração e às aventuras. Não se herdam plantas, construções prontas, geometrias ou receitas de montagem.

Estalagem, armazém, ferreiro e boticário formam o conjunto inicial de serviços, com praça/poço como centro comunitário. Famílias têm membros e relações com casas; comerciantes têm identidade e vínculo com uma loja. Profissões de referência incluem pedreiro, carpinteiro, metalúrgico, tecelão, jardineiro e carregador.

O estado das pessoas e dos comércios pode registrar presença, morte, propriedade e acontecimentos. A ausência ou morte de um comerciante não deve criar silenciosamente outro dono. Rotinas, trabalho, conversas e reação a eventos ainda exigem design e implementação próprios.

Compra e venda respeitam saldo, quantidade e categorias da loja. Comerciantes negociam categorias que conhecem; os preços não devem permitir lucro ilimitado por um ciclo de compra e revenda. Estoque renovável, saldo inicial e preços herdados são referências provisórias, sem estabelecer uma economia final.

## Montarias e portais

O Corcel Alado dos Bosques é a primeira montaria prevista. Permite invocar, montar, desmontar, decolar, voar, planar, acelerar, descer e pousar. Desmontar exige uma posição segura; um pedido no ar deve levar ao pouso antes da saída. Obtenção, desbloqueio, custo, tempo de invocação e progressão da montaria continuam abertos.

Portais são tecnologia élfica artesanal, conectada à exploração e próxima de povoados. Aproximar-se a pé e interagir abre um atlas 2D do mesmo mundo, com terreno, água, bosques, vilas, caminhos e portais. Pan e zoom permitem escolher um destino. Todos os portais da área consultada aparecem, inclusive os ainda não visitados; não há exigência de descoberta prévia.

Viajar exige desmontar e chegar a terreno seguro antes de retomar o controle. A viagem preserva personagem, aparência, inventário e progresso. O atlas representa o mundo real, sem inventar uma geografia separada. Fechar o atlas devolve os controles. Custos, recargas e restrições adicionais de viagem ainda não foram definidos.

## Quests e aventuras por IA

O jogador poderá escolher Começar uma nova aventura e pedir uma questline em linguagem natural, indicando tema, região e tom: sombrio, adorável, engraçado, misterioso, épico ou combinações compatíveis com o universo.

A aventura reúne premissa, quests, condições de início e conclusão, objetivos realizáveis, encontros, recompensas e consequências. Pode incluir personagens com nome, personalidade, relações e aparência, diálogos coerentes e locais como pontos de interesse, interiores, construções, povoados e cidades. Duração e dificuldade como controles do pedido continuam propostas.

Quests escritas e geradas respeitam as mesmas regras do jogo. Os personagens conhecem o que faz sentido para sua história e experiência. O conteúdo criado precisa ser acessível, coerente com o mundo e possível de concluir; falhas não devem deixar aventuras impossíveis ou pessoas e locais sem propósito.

### Permanência das histórias

Salvar preserva a aventura efetivamente criada, as pessoas, os lugares, as escolhas, os objetivos e os acontecimentos. Reabrir o jogo recupera a mesma história, sem gerar outra. Ao terminar uma questline, seus locais e personagens continuam existindo com as consequências do desfecho.

Novas aventuras consultam esse passado para reutilizar elementos e evitar contradições. Afastar-se de uma região não reinicia famílias, lojas, criaturas únicas ou quests. Uma mudança na apresentação de uma vila não apaga moradores nem acontecimentos anteriores.

O alcance dessa permanência, por jogador, grupo ou servidor, ainda depende da definição de multiplayer. Síntese de voz, modelo de acesso/cobrança das aventuras e limites de expansão também permanecem abertos.

## Interface e apresentação

Mundo e personagem são os protagonistas. Menu, seleção e criação mantêm continuidade espacial e atmosfera de fantasia; a interface precisa oferecer hierarquia, leitura e espaço para o avatar. A direção conceitual anterior orienta a intenção, sem reaproveitar imagens, molduras, logo pronto ou outros assets.

O HUD apresenta recursos, alvo, dez habilidades, minimapa, direção, coordenadas e hora do mundo. A ficha reúne atributos, progressão e edição da build. Menus, criação, comércio e atlas precisam impedir ações acidentais no mundo durante interação.

Inglês é o idioma padrão, prioritário e fonte dos textos de produto; português do Brasil integra o escopo inicial. Nomes e descrições comportam tradução. As plataformas são macOS primeiro e Windows; navegador fica fora do escopo atual.

## Decisões ainda abertas

- Cosmologia completa, identidade dos Criadores, cronologia, conflitos e campanha principal.
- Nomes definitivos das facções, alianças, reputação, restrições por espécie e regras de PvP.
- Alcance do mundo compartilhado, multiplayer, tamanho de grupos e quantidade de jogadores.
- Progressão final, fontes de experiência, recompensas, balanceamento, economia e obtenção de montarias.
- Slots/restrições de equipamento, crafting, raridades, durabilidade e efeitos dos consumíveis.
- Penalidades de morte, cemitérios, ressurreição de produto e regras de retorno.
- Comportamentos finais da fauna/guardiões, rotinas dos povoados e integração com quests.
- Efeitos do dia/noite, mecânicas aquáticas e possíveis atividades de sobrevivência.
- Relação de Chaos/Overdrive com o novo jogo: os antigos nomes técnicos não estabelecem recurso ou lore.
- Regras comerciais, duração, dificuldade, voz e limites das aventuras por IA.

O nome Survivors não acrescenta automaticamente fome, sede, crafting ou outras mecânicas de sobrevivência. A primeira experiência curta sugerida no chat permanece uma proposta futura; a prioridade atual é criar o mundo esférico e estudar sua geografia antes de elaborar a topografia.

## Origem das decisões

Base de design: GDD vigente de Land of Survivors no projeto histórico LandOfChaos; documentos de classes, mira, entrada, fauna, Guardas de Pedra, montarias, portais e povoados culturais; catálogo CombatCatalogue e definições de comércio consultados como referência semântica. Os documentos antigos explicitamente superados não substituem decisões posteriores.

A instrução de 3 de novembro de 2026 estabelece o novo desenvolvimento no Ignis e o reaproveitamento exclusivo de lore, história e decisões não técnicas. Este GDD e seus anexos são o ponto de entrada para a continuidade; toda implementação e construção anterior permanece fora dessa transferência.
