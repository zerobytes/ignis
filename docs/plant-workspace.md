# Espaço de trabalho de plantas — proposta

Direção de 27/10/2026: plantas devem ser uma feature visível da Ignis. O usuário
precisa encontrar a coleção, abrir o desenho e entender sua relação com o objeto
3D, independentemente do agente que o criou. Esta proposta define o produto;
o espaço de trabalho ainda não foi implementado.

## Estado verificado

`Scene.plants.entries` guarda definição corrente, referência e fonte canônica.
`inspect_plants` lista definições; `inspect_plant` inspeciona uma entrada;
`preview_plant` e `edit_plant` validam e publicam pelo dispatcher compartilhado.
`src/window.rs` apresenta a cena com winit/wgpu e controles de entrada; não há
biblioteca gráfica de plantas nem dependência de toolkit de interface.

Os desenhos em artifacts/plant-drawings foram derivados dos dados para a
conversa, fora da interface nativa. Não são o documento de origem nem uma
capacidade entregue do editor. A árvore de ensaio contém uma planta paramétrica
de galho; os anéis de contexto e as demais malhas não podem aparecer como
plantas independentes que nunca existiram.

## Experiência proposta

- Coleção: lista de plantas com miniatura, identificação, origem e objetos que
  as usam. Abrir um objeto permite encontrar sua planta; abrir a planta permite
  destacar as instâncias correspondentes. Um objeto sem planta informa isso.
- Prancheta: vistas frontal, lateral e superior em escala consistente, eixos,
  unidades, cotas, trajetória e seções identificadas. Cortes normais à trajetória
  têm sua própria vista; não devem ser confundidos com cortes horizontais.
- Resultado 3D ao lado: seleção coordenada entre seção, trajetória e objeto.
  Distinguir a base construtiva da malha com escultura e contexto agregado.
- Propriedades: dimensões e parâmetros legíveis, com valores e unidades;
  controles da curva e IDs das seções correspondem aos do documento real.
- Edição posterior: alterar cota, mover controle ou seção produz uma prévia;
  confirmar publica uma transação reversível. Diagnósticos de preservação ou
  domínio inválido permanecem visíveis, conservando o último resultado válido.

Começar pela coleção da cena aberta e sua visualização nativa. Uma coleção de
projeto entre arquivos/cenas exige localizar e identificar documentos fora da
cena, sem confundir entradas homônimas nem instanciar todos os exemplos. Essa
segunda capacidade precisa de contrato próprio de catálogo/persistência; a lista
da cena não será apresentada como biblioteca completa do projeto.

Não transformar o painel em ferramenta específica de árvore: cilindro, espada
e galho exercitam o mesmo documento. A interface deve representar também perfis
retos e plantas sem trajetória opcional, conforme o formato efetivamente aberto.

## Documento e comandos

O desenho é uma apresentação calculada da mesma Definition usada pelo kernel.
Não manter um SVG editável independente que possa divergir do JSON. As fórmulas
de trajetória, interpolação e frames precisam ser compartilhadas com o núcleo;
traçar uma curva parecida na interface não prova que o objeto corresponde a ela.

Seleção, zoom e layout são estado de editor, separados da simulação. Operações
novas do editor terão paridade MCP no dispatcher. Publicação usa a revisão que
originou a prévia, recusando atualização concorrente obsoleta. Histórico,
escultura, pesos, repouso e animação mantêm as políticas e limites existentes;
o editor não introduz uma garantia nova de preservação.

Alternativas de apresentação: exportação SVG é útil para compartilhar, mas não
substitui a ferramenta nativa; um visualizador web separado acrescenta outra
superfície de interação; desenho próprio em wgpu exige texto, widgets e
acessibilidade; integrar um toolkit reduz parte desse trabalho e acrescenta
dependência, consumo de eventos e composição GPU. A direção recomendada é o
espaço nativo, com uma camada de apresentação pequena sobre os contratos atuais.
A escolha de toolkit depende de um protótipo de integração e comparação destes
custos; nenhuma biblioteca ou nova arquitetura gráfica foi escolhida aqui.

## Marcos verificáveis

1. Coleção da cena e prancheta somente de leitura, com as plantas reais do
   cilindro, espada e galho. Validar projeções e cortes contra o kernel, seleção,
   abertura/ausência de plantas, resize e escala de pixels. A coleção deve
   atualizar após edição MCP, undo/redo e load, sem alterar o documento ao abrir.
2. Edição visual dos parâmetros já admitidos: prévia, confirmação, cancelamento,
   revisão obsoleta, erro sem mutação e undo único. Demonstrar cota/curva alterando
   a geometria com escultura e rig segundo as políticas explícitas existentes.
3. Coleção entre cenas/arquivos, nomes de apresentação, organização e reutilização,
   após definir identidade e propriedade de documentos. Não duplicar fontes
   canônicas ou transformar miniaturas em origem de autoria.

Capturas offscreen validam desenho e regressão. Leitura das cotas, navegação,
seleção e edição na janela exigem demonstração própria; nenhum destes marcos
está concluído por esta proposta documental.

## Revisão estática prévia à implementação

Aplicação dos dez pontos de performance.md: (1) trabalho de autoria, fora do
tick; (2) não copiar fonte/rig/clipes para desenhar miniaturas; (3) usar IDs e
unidades explícitos, sem serializar JSON por frame; (4) reutilizar a avaliação
matemática do núcleo; (5) respeitar budgets existentes, limitando amostras e
miniaturas; (6) sem readback GPU rotineiro, revisar custo de texto e composição;
(7) não segurar o lock do serviço durante desenho ou interação; (8) invalidar
derivados por revisão/documento, projeção e viewport, distinguindo câmera de
mudança geométrica; (9) verificar pureza, atualização e igualdade com os dados,
além da interação humana; (10) registrar custos do toolkit e limitações reais,
sem promessa de FPS. A implementação deverá responder o checklist com evidência.

Esta unidade altera apenas documentação: sem src, dependências, formatos,
binário, cena ou janela novos. Validação aplicável: links locais e diff.
