# Ferramentas de relevo: inventário e próximos contratos

Continuação: [Surface1](surface-program.md) implementa o programa comum em
Terrain3/Planet2. O inventário Terrain2/Planet1 abaixo registra o diagnóstico
anterior; os leitores legados permanecem, e a fonte nova compartilha o catálogo
completo de campos/perfis. Hidrologia, erosão e composição volumétrica continuam
contratos próprios.

Revisão de03/11/2026, após o teste humano de `fff9c97`/build8195d0e3b6ac04bf.
Guilherme reprovou a aparência das montanhas e reportou FPS entre2 e50. Pedido:
analisar as ferramentas necessárias, preservando esta cena, e coordenar a
instabilidade com o Horizon. Este documento é uma proposta de engenharia;
nenhuma das capacidades futuras abaixo é declarada implementada.

## Diretriz arquitetural confirmada

Guilherme reforçou nesta revisão: plano e esfera devem oferecer as mesmas
ferramentas, receitas, edição, parâmetros e geração; só a geometria do domínio muda.
A separação atual entre Terrain e Planet é estado legado a migrar, não a arquitetura
alvo. Não criar um catálogo planetário menor ou portar cada ferramenta duas vezes.

Uma fonte/programa comum recebe um domínio de superfície, responsável por posição,
vertical/normal local, distância e direção métricas, vizinhança, área e condições
de fronteira. A esfera fecha a superfície e muda esses cálculos; o plano fornece
seu referencial plano. Relevo, máscaras, curvas, erosão e hidrologia usam o mesmo
contrato. Distâncias e alturas são em metros. Mesma ferramenta não promete uma
realização global numericamente idêntica em domínios geometricamente distintos.

Receitas criativas externas serão convertidas para essa fonte comum. O estudo
[terrain-generators-study.md](terrain-generators-study.md) registra candidatos,
licenças/versionamento, formas de reuso e limites verificados. Nenhuma dependência
ou receita externa foi integrada nesta etapa.

## Diagnóstico e evidência atual

No print há predominância de encostas compridas e arredondadas, silhuetas de
colinas parecidas e pouca articulação visível entre cristas, vales e patamares.
Isso é uma observação dessa vista; não permite diagnosticar sozinho todo o mundo
ou atribuir toda a aparência à geometria. Cor, iluminação e atmosfera também
influenciam a leitura, mas não serão alteradas para maquiar a lacuna de ferramentas.

A diferença entre os kernels é concreta:

| Camada | Capacidade verificada no código | Lacuna |
|---|---|---|
| `src/terrain.rs`, Terrain2 | `Ridge`, `Raise`, `Level`, `Noise` smooth/ridged, `Channel`, `Clamp`; fonte fixa por amostras ou operações ordenadas | Sem composição por grafo/máscaras, cristas ramificadas por curvas, hidrologia ou erosão do terreno |
| `src/planet.rs`, Planet1 | `Noise`3D e `Raise` radial com suporte angular | Não consome as ferramentas Terrain2; variar seed/amplitude mantém a mesma família limitada de formas |
| `src/landforms.rs` | Perfis/seções3D, ápices, patamares, paredes, cavidades; presets pico curvo/cânion/mesa/arco/túnel | Peças locais independentes; sem união entre peças e sem integração com uma superfície planetária composta |
| `src/terrain/region.rs` | Compilação/publicação regional do campo comum | Região estática não estabelece planejamento geográfico contínuo nem dependências de bacias entre regiões |
| `docs/representative-world.md` | Ridge nativo e uma composição regional com colisão | A entrega183 atualiza a limitação histórica de179: Ridge já foi integrado ao Terrain, ainda não ao Planet |

Portanto há duas necessidades distintas: tornar ferramentas existentes utilizáveis
nos dois modos de mundo e ampliar o vocabulário e a composição do relevo. Não
precisamos recriar do zero toda parede, montanha ou canal que já existe.

## Referências estudadas

- [Distant Horizons, página dos autores](https://www.curseforge.com/minecraft/mc-mods/distant-horizons): representação de terreno distante por níveis de detalhe. Pode antecipar/solicitar geração conforme configuração; a forma dos terrenos depende do gerador do mundo. Não atribuir toda paisagem de um modpack ao algoritmo de LOD.
- [Tectonic, descrição do projeto](https://modrinth.com/datapack/tectonic): exemplo de vocabulário geográfico com continentes, ilhas, cadeias, cânions, patamares e vales. Referência funcional; nenhuma receita ou implementação foi importada.
- [Gaea, Terrain](https://docs.gaea.app/reference/nodes/terrain/): separação entre formas estruturais e refinamento posterior.
- [Gaea, MountainRange](https://docs.gaea.app/reference/nodes/terrain/mountainrange.html): controle de massa, escala e perfis de cadeias.
- [Gaea, Erosion](https://docs.gaea.app/reference/nodes/simulate/erosion): processos de desgaste/transporte/depósito e saídas de dados; documentação explicita o tradeoff de determinismo em processamento paralelo.
- [Gaea, Rivers](https://docs.gaea.app/reference/nodes/simulate/rivers): rede hidrográfica com cabeceiras e interferência no terreno, separada de simplesmente desenhar água.

Essas referências orientam a decomposição das ferramentas. Não constituem
comparação medida de desempenho, adoção de dependências ou promessa de réplica.

## Ferramentas e prioridades propostas

### P0 — Composição comum e integração entre modalidades

Uma fonte de relevo reutilizável por autoria fixa, procedural e planetária, com
kernels preparados comuns e adapters de coordenadas. Começar reaproveitando
Ridge/Level/Channel; não copiar um segundo catálogo dentro de Planet.

Composição por dados: somar, substituir, mínimo/máximo, mistura suave, curvas de
remapeamento e máscaras. Uma máscara controla onde e com qual intensidade uma
operação atua. Suas entradas podem ser distância de uma curva/região, altura,
inclinação ou outro campo; parâmetros podem variar espacialmente. A sequência
atual de operações é um ponto de partida, não um grafo geral já implementado.

Grafo deve ser DAG validado e compilado em programa tipado com slots preparados;
JSON e busca por nomes ficam na fronteira de autoria/MCP. Não interpretar
objetos JSON nem alocar um nó por amostra. O recorte inicial pode ser pequeno;
um editor visual de nós sofisticado não é requisito para validar esse contrato.

Coordenação espacial faz parte da ferramenta: distâncias/larguras em metros,
datum explícito, campos contínuos e seed estável no espaço do mundo. Projetar
cada face do cubo independentemente criaria emendas geológicas. Compartilhar a
matemática do perfil com coordenadas apropriadas em cada adapter, preservando
polos e bordas. Preservar leitores/fontes antigos mediante contrato de versão.

### P1 — Cristas, vales e perfis controláveis por curvas

Curvas editáveis para conduzir cadeias e vales; largura, altura, inclinação de cada
lado e seção variam ao longo do caminho. Suportar cristas secundárias, encontro
entre cadeias, passes/saddles e vales emV/U como perfis, com controle de término
e transição. A crista senoidal isolada atual não fornece toda essa estrutura.

Perfis assimétricos permitem uma escarpa rochosa de um lado e uma vertente longa
do outro. Relações entre cristas e vales produzem formas legíveis em várias
escalas. Campos de deslocamento/domain warp quebram regularidade quando úteis,
sem substituir essa estrutura por ruído indiscriminado.

### P1 — Planejamento geográfico e transições entre regiões

Campos para massas continentais, costa/batimetria, altitude de base e distribuição
de planícies, serras, depressões e bacias. Perfis geográficos combináveis pelo
jogo, com transições controláveis, evitam que todo lugar use a mesma montanha.
Não exigir simulação de placas tectônicas para obter essas capacidades.

A autoria manual define esses campos/contornos; uma receita procedural produz
os mesmos dados. Regras regionais precisam de endereços/IDs/seed independentes
dos chunks de renderização. Não enumerar todas as montanhas do planeta numa
lista global nem varrê-las a cada amostra; indexação espacial e derivação
hierárquica precisam fazer parte do contrato. Isso permite extensão grande sem
transformar a quota atual de operações numa quantidade máxima de acidentes do mundo.

### P2 — Drenagem, rios, lagos e bacias

Ferramentas para nascentes/saídas, direção e acumulação de escoamento, encontros
de afluentes, leitos, margens, planícies aluviais e depressões com lagos. Permitir
orientação manual de cursos e geração condicionada pelo terreno. Escavar um
`Channel` explícito não descobre uma bacia ou constrói uma rede conectada.

É um problema regional e com dependências: uma região pode receber água de outra.
Definir saídas/fluxos de fronteira e persistência dos resultados; não tratar cada
tile como uma ilha hidráulica. Carvar relevo e produzir dados hidrológicos é
independente de renderizar/simular a superfície líquida.

### P2 — Erosão e deposição

Erosão térmica para redistribuição em encostas/talus e hidráulica para sulcos,
ravinas, desgaste e depósitos. Intensidade, precipitação e resistência do material
são campos/configuração do jogo; não obrigar toda montanha a ser erodida.

Esses processos têm custo e dependência de vizinhança diferentes de um ruído
pontual. Executar por regiões/estágios com halo e condições de fronteira,
produzindo dados congelados consultáveis. Não rodar uma simulação erosiva em cada
consulta de altura, frame ou mudança de LOD. Políticas de determinismo, iterações,
memória e tratamento de bordas precisam ser explícitas, sem alegar equivalência
entre um filtro barato de aparência e um modelo físico validado.

### P3 — Estratos, falhas, patamares e integração volumétrica

Operadores de terraços, camadas/inclinação/dobramento e deslocamento de falhas,
com resistência por camada quando usada por erosão. Permitem distinguir
escarpas, mesas, cânions estratificados, dunas e outras famílias de terreno.
Perfis artísticos são permitidos; não prometer geologia universal calibrada.

Paredes exatas, reentrâncias, arcos e cavernas exigem mais que uma altura por
posição na superfície. Reutilizar Landform1 e especificar como recortar/costurar
uma peça à superfície, compor colisões e consultar múltiplas superfícies. CSG/SDF
local ou outras técnicas têm custo/robustez próprios. Recomendação: conservar
heightfields eficientes para a maior parte do solo e acrescentar recursos3D
locais onde necessários, sem exigir um volume denso para o planeta inteiro.

### Transversal — Dados derivados e autoria verificável

Inclinação, curvatura, distância de crista/curso/costa, acumulação de fluxo,
desgaste e sedimento como campos reutilizáveis. Servem a seleção de formas,
materiais e vegetação posteriores; as ferramentas de relevo não precisam decidir
que textura ou espécie será usada.

Mesmas operações por MCP antes/junto da interface visual: curvas/regiões editáveis,
preview com custo/escopo explícitos, salvar fonte, desfazer, congelar em amostras
e regenerar por seed. Máscaras/contornos e perfis intermediários inspecionáveis
permitem verificar por que uma forma foi produzida. Nenhum terreno depende do
agente que o criou.

## Recorte de implementação recomendado

Primeiro entregar P0 + o núcleo de P1: composição mascarada e perfis de cristas/
vales por curvas, compartilhando kernels entre plano e esfera. Validar autoria
manual e receita seeded produzindo a mesma fonte editável, em fixtures próprias.
Isso ataca a limitação mais imediata antes de financiar uma simulação geológica
completa. Hidrologia/erosão entram em seguida como processamento regional.

O gate será capacidade das ferramentas: controlar assimetria, ramificações,
transições e vales; continuidade entre regiões/faces/LODs; preservar autoria,
colisão e identidade da fonte. Inclui avaliação visual do relevo sob condições
controladas, próxima e distante. Não será simplesmente trocar seed/cores ou
escolher uma câmera bonita nesta demonstração humana reprovada.

## Revisão estática de performance da proposta

1. Separar operações pontuais preparadas de processos regionais de vizinhança.
2. Compilar o grafo uma vez; reutilizar scratch/slots e descartar suportes remotos cedo.
3. Dimensionalidade explícita: campos escalaresf32, coordenadas globais precisas,
   vetores/gradientes quando necessários; memória por região e não pela área total.
4. Cache por fonte/revisão/parâmetros/resolução/condições de fronteira; edição
   invalida dependências afetadas e seus vizinhos necessários, não todo o mundo.
5. Física e render derivam da mesma fonte/contrato; LOD não gera nova geografia,
   novo curso de rio ou novas seeds. Bounds/erro de cada representação são explícitos.
6. Processos não locais exigem preparação/bakes com limites e fallback disponível.
   Novos nós não devem recolocar geração pesada síncrona no frame principal.
7. Formatos/documentos, save/replay e codecs existentes só mudam em unidade própria,
   com leitura antiga e rejeições atômicas conforme o contrato aprovado.
8. Nenhum JSON no kernel; não transmitir malha expandida pelo MCP para autoria comum.
9. Medir tempo por estágio, p95/p99/pior frame e movimento/transições, além do custo
   aquecido de desenho. Cobertura disponível permanece até substituta pronta.
10. Nenhuma mudança de código/cena/build foi feita para este inventário. Não há nova
    validação executável, ganhoFPS ou aceite artístico inferido deste documento.

## FPS: frente separada, aceite aberto

Horizon/nanites assumiu diagnóstico e correção por solicitação explícita do usuário.
Reservas e evidências estão em EAF-199 e no contrato próprio de frame pacing.
Geração/replanejamento síncronos e recomposição de costuras são investigados por
percurso/pan contínuos em processo isolado; picos CPU já foram reproduzidos, mas
isso não encerra a explicação de todos os frames de2FPS nem a correção completa.
Preservar a janela/cena/config humana e coordenar build/GPU; não compensar custo
reduzindo densidade, grade ou qualidade de relevo.
