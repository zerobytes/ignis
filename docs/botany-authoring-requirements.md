# Requisitos de autoria de árvores ancestrais

4 de novembro de 2026 · LOS-21 · Requisitos para evolução das ferramentas.

Decisão do usuário: desenvolver primeiro os geradores de plantas e texturas
para construir um tronco central muito largo, antigo e irregular. O primeiro
ensaio de árvore e casca foi reprovado. A próxima implementação deve partir
destes requisitos, sem retomar automaticamente a geração da árvore completa.
O fluxo de image-to-3D fica fora desta etapa de definição e desenvolvimento.

A árvore élfica de folhas brancas é o primeiro caso de uso. As ferramentas
devem servir também a outras árvores. A referência visual orienta a forma;
não determina uma espécie, idade biológica, estrutura traseira ou medidas
desconhecidas. Cerca de 5 m de diâmetro é a referência inicial do usuário para
o tronco; a largura visual será ajustada com ele. O tronco precisa dominar a
composição e conservar massa até a região dos grandes braços.

Este documento especifica comportamento exigido, não declara funcionalidades
implementadas. Recursos existentes podem atender parte dos requisitos, mas
precisam passar pelos critérios de aceitação correspondentes. A escolha do
algoritmo, representação e formato fica para o projeto de cada incremento.

## Contrato comum aos dois geradores

| ID | Requisito obrigatório | Aceitação |
| --- | --- | --- |
| AUT-01 | Manual e procedural devem executar as mesmas operações sobre a mesma fonte autoral. Cada operação precisa expor parâmetros numéricos e controles espaciais quando pertinentes. | Autorizar manualmente uma receita e reaplicá-la por comando produz a mesma avaliação, com a mesma semente e ambiente de execução. Nenhum controle necessário fica exclusivo de scripts ou de um preset. |
| AUT-02 | Comprimentos, profundidades e tamanhos de detalhes devem ter unidades físicas; posição longitudinal normalizada pode complementar medidas em metros. | A interface informa unidades. Alterar o tamanho da árvore permite escolher manter o tamanho físico da casca ou escalá-lo junto. |
| AUT-03 | Cada curva, seção, deformação e máscara deve ter identidade estável, seleção e escopo de influência explícitos. Operações devem poder ser combinadas, reordenadas, desativadas e restauradas. | Editar um vale ou uma raiz não altera silenciosamente os controles de outro. Ordem e dependências ficam visíveis e são salvas. |
| AUT-04 | Regeneração deve preservar ajustes manuais compatíveis e os materiais aplicados. Incompatibilidades exigem diagnóstico e resolução explícita. | Preview e aplicação concordam; cancelamento, undo/redo e save/load conservam a fonte. Uma alteração incompatível é recusada sem substituir o último resultado válido. |

## Volume do tronco e dos braços

| ID | Requisito obrigatório | Parâmetros mínimos e critério de aceitação |
| --- | --- | --- |
| GEO-01 | Controlar o tronco central independentemente da copa. | Diâmetro, altura até as bifurcações e perfil de afilamento. Informar larguras em seções selecionadas, separando tronco e expansão das raízes. Demonstrar um tronco muito largo que conserva massa na parte superior. |
| GEO-02 | Editar seções transversais irregulares em várias alturas. | Posição, contorno, orientação e transição entre seções. Permitir concavidades locais e assimetria. A forma deve permanecer volumétrica em frente, lateral, verso e vista superior. |
| GEO-03 | Criar engrossamentos e estreitamentos locais. | Centro, extensão longitudinal/angular, intensidade e suavidade. Uma protuberância deve poder ser localizada em um lado do tronco sem engrossar todo o perímetro. |
| GEO-04 | Curvar, inclinar e torcer o eixo com transições controladas. | Curva-guia, torção ao longo do eixo e influência local. Seções e detalhes vinculados acompanham a edição sem saltos de orientação. |
| GEO-05 | Construir grandes cordões sinuosos de madeira na superfície. | Curva-guia, largura, altura, perfil transversal, torção, início/fim e transição. Permitir trajetórias que saem das raízes, percorrem o tronco e continuam nos braços, com influência editável nas junções. |
| GEO-06 | Construir vales profundos entre volumes. | Percurso, largura, profundidade, perfil e término. Os vales devem existir na geometria, produzir paralaxe e sombra sob luz neutra e permitir profundidades diferentes em cada região. |
| GEO-07 | Sobrepor irregularidades em escalas diferentes. | Escala física, amplitude, orientação, semente e máscara por camada. Permitir variação suave do volume e deformações menores, sem impor faixas periódicas ou ruído uniforme em toda a árvore. |
| GEO-08 | Criar nós, cicatrizes e madeira exposta localizados. | Posição, dimensão, orientação, contorno e relevo; máscara semântica exportável ao material. O detalhe deve alterar o volume quando sua escala exigir e permanecer editável. |
| GEO-09 | Criar recessos abertos com profundidade e borda. | Contorno de abertura, profundidade, espessura mínima e desgaste da borda. Conferir a cavidade por mais de um ângulo, sem pintura escura simulando profundidade. Túneis atravessando o tronco e anastomose geral não são exigidos nesta primeira entrega. |
| GEO-10 | Controlar a estrutura dos grandes braços e suas junções. | Inserção, direção, raio/perfil, curvatura, continuidade e extensão da transição. Permitir assimetria forte, braços baixos e um braço ascendente dominante. As junções devem ter volume contínuo e não apresentar aberturas acidentais. |

## Base e raízes

| ID | Requisito obrigatório | Parâmetros mínimos e critério de aceitação |
| --- | --- | --- |
| RAI-01 | Alargar a base de modo irregular e contínuo com o tronco. | Alcance, altura e intensidade por direção; transição com as seções do tronco. A base precisa distribuir massa entre raízes sem produzir uma aba plana ao redor do tronco. |
| RAI-02 | Autorizar raízes individuais com seções arredondadas e irregulares. | Curva, seção, espessura, afilamento, ramificação e influência no colo por raiz. Demonstrar raízes grossas, dorsos curvos e vãos reais entre elas; paredes achatadas não atendem ao caso de uso. |
| RAI-03 | Controlar exposição e entrada no solo. | Trajeto vertical, profundidade, trecho exposto e transição por raiz; solo explícito quando usado. A raiz pode emergir, curvar e desaparecer no chão sem terminar em uma seção abruptamente cortada. |
| RAI-04 | Transportar formas e aparência entre raiz, colo, tronco e braço. | Vínculos editáveis entre curvas e campos. Demonstrar um cordão e sua orientação de casca atravessando essas regiões sem reiniciar o padrão em cada órgão. |

## Casca, coordenadas e materiais

| ID | Requisito obrigatório | Parâmetros mínimos e critério de aceitação |
| --- | --- | --- |
| MAT-01 | Gerar coordenadas e direções de superfície adequadas à madeira. | Direção longitudinal, direção transversal, escala física e ajustes locais. Fibras devem acompanhar curvas, contornar nós e se distribuir nas bifurcações. Conferir todo o perímetro e junções, incluindo colapso e distorção de coordenadas. |
| MAT-02 | Separar grandes volumes, relevo intermediário e detalhe fino. | Escala e intensidade independentes. Cordões e cavidades pertencem à geometria; placas e lascas exigem relevo ou geometria conforme sua dimensão; microfissuras e rugosidade podem ser material. Conferir closes oblíquos e silhueta. |
| MAT-03 | Gerar placas, fissuras e fibras irregulares. | Tamanho, espaçamento, direção, largura, profundidade, comprimento, ramificação, interrupção e variação por semente. Aceitar famílias alongadas e descontínuas; faixas senoidais regulares não atendem à casca ancestral. |
| MAT-04 | Correlacionar cor e resposta superficial com a estrutura. | Cor, rugosidade e relevo por máscara de fissura, placa, borda e madeira exposta. Clarear uma borda deve afetar as bordas identificadas, sem clarear arbitrariamente toda a casca. |
| MAT-05 | Usar máscaras da geometria e da autoria no gerador de materiais. | Disponibilizar posição, altura, orientação da superfície e identificadores/pesos de cordões, vales, nós, cicatrizes e raízes. Campos aproximados, como concavidade, devem declarar como foram calculados. Visualizar cada máscara isoladamente. |
| MAT-06 | Combinar distribuição procedural e pintura manual. | Adição, remoção, intensidade, transição e prioridade entre máscaras, na mesma fonte persistida. Demonstrar musgo localizado em raízes e recessos, editar uma mancha manualmente e regenerar conservando a edição compatível. |
| MAT-07 | Controlar desgaste da casca. | Intensidade, escala, arredondamento/lasca e regiões protegidas ou expostas. Desgaste que altera contorno precisa afetar geometria; mudança de cor isolada não satisfaz esse efeito. |
| MAT-08 | Avaliar e salvar materiais reutilizáveis em resoluções controladas. | Receita, mapas/canais, escala, resolução, filtragem e custo explícitos. Reabrir e reaplicar deve conservar o resultado dentro da precisão declarada; conferir repetição, aliasing e proximidade/distância. |

## Segurança de edição, qualidade e custos

| ID | Requisito obrigatório | Aceitação |
| --- | --- | --- |
| VAL-01 | Detectar configurações inválidas e preservar a edição válida. | Recusar ou sinalizar a região afetada por colares incompatíveis, inversões, autointerseções detectadas, espessura insuficiente e excesso de orçamento. Identificar controles envolvidos e ação possível. Não mascarar falha com geometria atravessada; declarar o alcance da detecção. |
| VAL-02 | Controlar resolução por região e escala do detalhe. | Detalhar uma cavidade não deve multiplicar obrigatoriamente toda a copa. Antes de aplicar, informar estimativas; depois, informar vértices, triângulos, memória e tempo de avaliação reais. Tetos existentes continuam explícitos até decisão fundamentada de revisão. |
| VAL-03 | Preservar versões anteriores e separar fonte de resultado compilado. | Novos recursos são opt-in/versionados; receitas antigas conservam comportamento. Salvar, reabrir, regenerar e editar deve funcionar sem tratar os triângulos compilados como única fonte autoral. |
| VAL-04 | Disponibilizar diagnóstico visual e técnico. | Vistas de superfície neutra, estrutura, direção das fibras, máscaras e materiais; capturas registram câmera/luz/resolução. Testes de integridade e reprodução não substituem avaliação artística nem comprovam FPS. |

## Ordem de implementação e aceitação

1. **Tronco e base em superfície neutra:** AUT-01–04, GEO-01–07, GEO-10,
   RAI-01–03 e VAL-01–04. Entregar controles suficientes para esculpir a massa
   central larga, as deformações maiores, braços e raízes arredondadas. Conferir
   frente, lateral, verso, topo e detalhes sob luz neutra, sem folhas ou textura.
   A forma precisa corresponder à direção artística antes de avançar a casca.
2. **Formas locais e continuidade:** GEO-08–09 e RAI-04, mantendo os contratos
   anteriores. Entregar nós, cicatrizes, recessos e cordões contínuos; conferir
   profundidade, espessura e transições em múltiplas vistas.
3. **Casca orientada pela superfície:** MAT-01–05 e MAT-08. Demonstrar primeiro
   direções/coordenadas e máscaras; depois fissuras, placas, relevo e cor na
   mesma forma já avaliada. Conferir frente, laterais, verso e bifurcações.
4. **Acabamento e autoria localizada:** MAT-06–07. Entregar pintura combinada
   ao procedural, desgaste e musgo, com preservação na regeneração.
5. **Retomada da árvore élfica:** aplicar as ferramentas ao asset após essas
   etapas; acrescentar a copa frondosa e folhas brancas com sombras legíveis.
   Aprovar a folhagem é uma avaliação posterior, sem encobrir falhas do tronco.

Os contratos AUT e VAL se aplicam a todos os incrementos, inclusive aos
materiais. A implementação deve trazer evidência técnica proporcional e
comparação visual que permita revisão humana. Aceite artístico pendente deve
permanecer declarado; este documento não concede aceite antecipado.

## Limites desta definição

Animação, simulação de crescimento biológico, física de destruição, copa nova,
menu, personagens e integração planetária não fazem parte do desenvolvimento
inicial destas ferramentas. Preservar os experimentos existentes sem instalá-los
como árvore final. Não foi escolhida uma solução de CSG, remalhamento ou UV;
essa escolha deve responder aos requisitos e à compatibilidade do Ignis.

Referências internas: [análise inicial da árvore](games/land-of-survivors/arvore-elfica-analise.md),
[botânica nativa](botany-integration.md), [raízes](botany-roots.md),
[colo](botany-collar.md), [receitas de materiais](material-recipes.md) e
[inspeção do domínio material](mesh-material-domain.md). A presente decisão
substitui a continuação automática do ensaio visual anterior; os documentos
históricos continuam como evidência dos respectivos trabalhos.
