# Primeira cena no Ignis

## Criar e salvar

1. Use **Arquivo → Novo projeto…** ou **Projeto…**. Escolha a pasta pelo navegador
   do formulário e dê um nome ao projeto.
2. Crie/abra uma cena pelo painel **Projeto**. Um projeto organiza arquivos;
   uma cena contém objetos, malhas, personagens e suas configurações.
3. Use **Arquivo → Salvar cena** ou o atalho de salvar. Alterar uma propriedade
   atualiza a cena imediatamente, mas não salva o arquivo automaticamente.
4. **Executar** inicia a simulação. **Reiniciar** restaura o estado autorado;
   configuração de objetos fica disponível enquanto a simulação está pausada.

## Um personagem que persegue o jogador

1. Tenha um piso estático na cena. Use **Adicionar → NPC / personagem**.
2. O novo NPC fica selecionado. Em **Propriedades → Componentes e comportamento
   → Comportamento do NPC**, confira **Ativo**, velocidade, distância de parada
   e piso de navegação.
3. **+ Alvo** permite escolher o jogador ou outro personagem existente.
4. Abra **Percepção e memória** para configurar alcance, campo de visão, memória
   e busca. O modo perceptivo depende de evidência visual; onisciente é uma
   opção de diagnóstico. Criar um NPC não escolhe alvo/combate automaticamente.
5. **Executar** permite observar o resultado. Para mais comportamentos, expanda
   os componentes de patrulha, objetivos, território, habilidades ou comunicação.
   As configurações são validadas em conjunto; uma combinação incompleta mostra
   erro e conserva a cena anterior até ser corrigida.

## Modelar e esculpir

1. Em **Adicionar → Malha editável**, escolha uma primitiva e sua resolução.
2. Selecione a malha na viewport ou na hierarquia. **Objeto** edita posição,
   rotação, escala e aparência. Arraste campos numéricos horizontalmente;
   Shift oferece ajuste fino. Também é possível digitar o valor.
3. **Escultura** oferece Volume, Mover e Suavizar. Em áreas estreitas, os modos
   ficam agrupados no menu do cabeçalho. Raio/força ficam no inspetor;
   arraste sobre a superfície. Ctrl inverte Volume, Shift suaviza e Esc cancela
   o traço. Cada traço confirmado é um passo de desfazer.
4. **Editar** seleciona vértices, arestas ou faces. Arraste com o botão esquerdo
   para formar um retângulo. Shift acrescenta à seleção; clique no vazio sem
   Shift desseleciona os componentes.
   O centro local XYZ move seus vértices imediatamente. **Subdividir seleção**
   acrescenta resolução nos componentes escolhidos; **Suavizar seleção** usa
   seus vértices.
5. **Vista** mostra arestas, vértices e ossos. **Raio X da malha** desligado
   mostra/seleciona só a superfície visível; ligado inclui a estrutura atrás.
   Ossos são guias independentes. As faces atuais são triângulos.
6. A edição geométrica altera o asset: suas instâncias compartilham o resultado.
   Desfazer/Refazer recupera as alterações. O bug de direção dos traços foi
   corrigido para novos gestos; uma malha já salva não é reparada automaticamente.

## Começar um rig

1. Selecione a malha e abra **Rig → Criar esqueleto**. Escolha a raiz e adicione
   ossos filhos. Edite posição/rotação de repouso relativa ao pai.
2. Em **Editar**, selecione os vértices que acompanharão um osso. Volte a **Rig**,
   escolha o osso e seu peso, depois use **Atribuir peso à seleção**. O restante
   do peso vai à raiz; não há atribuição por proximidade ou pintura automática.
3. **Criar clipe** inicia uma animação. Escolha o tick, translação/rotação local
   e **Inserir / substituir chave do osso**. A interface inicial edita as chaves
   do primeiro clipe; **Ferramentas** dá acesso ao contrato completo de clipes.
4. **Usar clipe** o atribui à instância. Execute a simulação para reproduzir.
   Timeline gráfica, pintura de pesos, IK e retargeting continuam pendentes.

## Encontrar uma operação avançada

**Ferramentas…** oferece busca por tarefa e categorias. Escolha a operação,
preencha o formulário e execute; o resultado aparece abaixo. Selecione uma malha
antes de abrir suas operações para preencher o contexto de asset/revisão.
O catálogo acompanha os comandos compartilhados com MCP; vários sistemas ainda
precisam de uma interface especializada além desses formulários avançados.

Os ícones no canto direito de cada cabeçalho maximizam/restauram, dividem a área
e abrem/fecham abas. Passe o mouse para ver o nome; ao dividir/abrir, escolha o
painel pelo popup.

Contratos, validação e limites de implementação: [editor-workflows.md](editor-workflows.md).
