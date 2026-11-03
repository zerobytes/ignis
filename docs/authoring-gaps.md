# Fricções de autoria — estado do ensaio da casa

Pergunta de trabalho de Guilherme: **o que me permitiria melhorar o resultado se
estivesse disponível?** Cada resposta precisa de um caso mínimo, uma capacidade
genérica e uma nova prova na casa. Não converter escolhas de composição erradas
em desculpas de ferramenta ausente, nem substituir revisão visual por suite verde.

| Observação real | Capacidade necessária / diagnóstico | Situação e próxima prova |
|---|---|---|
| A entrada desaparece entre varanda e bay; empena não organiza o volume arredondado | Validar relações espaciais/vazios, corrigir hipótese de implantação. Não é falta de material ou subdivisão | Revisão elven-cottage-review.md; composição em native-plants. Primeiro imagem neutra com porta/pórtico/patamar legíveis |
| Pedra parece amassada quando o ruído de relevo fica grande | Separar grão/relevo e manchas amplas | Duas camadas EAF-112 concluídas; mesma casa, geometria/cores/instâncias fixas e ablação reproduzível |
| Bronze tinha realce branco e difuso semelhante a tinta | Resposta metálica com reflexo colorido e parcela difusa suprimida | EAF-114 implementada; caso de incidência quase normal concorda com GGX e casa altera somente um material. Não corrige os perfis dos ornamentos |
| Fora do realce, metal parece plano | Reflexão do ambiente: ambiente constante atual não dá formas ao reflexo | EAF-116 implementa probe procedural pré-filtrada/cubemap+DFG. Caso curvo/constante/eixo/vista/rugosidade e casa com ablações. Ainda sem HDR importado/reflexos locais/AO; não tratar isso como IBL geral concluída |
| Oxidação/rugosidade não podem variar localmente no bronze | Máscara/domínio para parâmetros físicos; roughness/metallic hoje uniformes por asset | Gap aberto. Não pintar manchas escuras e declarar oxidação. Caso mínimo separa mudança de cor, rugosidade e metalness no mesmo objeto |
| Painéis verdes continuam opacos | Transparência de superfície, ordenação/profundidade/sombra e interior visível | Gap aberto. Testar duas lâminas sobrepostas com objeto atrás, undo/replay e diferentes vistas. transmission existente é luz traseira aproximada, não vidro transparente |
| Nervuras/pilares são tubos simples e silhuetas facetadas | Seção explícita e varredura com trajetória/frame controlados | Sweep genérico em desenvolvimento em native-plants; provar seção e vazios antes de repetir em toda a casa. Sem duplicar implementação nesta frente |
| Peças agrupadas têm orientação de grão compartilhada | Domínio material de repouso por peça, continuação por trajetória e diagnóstico de distorção | Ferramentas de frame/consulta existem; cliente ainda precisa manter orientação por parte. Caso mínimo: três peças rotacionadas com grão longitudinal, inclusive após regeneração |
| 15assets/32172v deixam apenas1slot/596v livres nesta casa | Autoria precisa de orçamento observável; atribuição material por parte não deveria exigir duplicar malha | Caps mantidos. Split/submeshes/materiais por região exigem contrato de proveniência e invalidação; não elevar teto para esconder consumo. Caso mínimo isolado antes de mudança estrutural |
| A nova luz expõe terminações e transições de brilho abruptas nas peças finas | Diagnóstico de normais/sidedness e separação de geometria versus shading | Hipótese aberta, não causa certificada. Caso mínimo com orientação de faces conhecida e visão de normais; não acrescentar ambiente constante para esconder |
| Realces finos podem saturar/piscar e revelar facetas | Separar geometria insuficiente, aliasing especular e alcance dinâmico | UNORM/roughness mínima atuais não garantem filtragem. Caso mínimo com movimento de câmera e sequência de pixels; controles roughness/flat/smooth, sem confundir suavizar normais com corrigir silhueta |

## Evidências correntes

- Fonte original da casa ea8c6b8, Scene72: SHA7286e74c…f1e2c. Não sobrescrever
  a versão apresentada para evitar perder o comparativo.
- [Estudo das duas camadas](cottage-materials.md): script/receita e capturas
  before/after, controle sem padrões, ablação e invariantes de toda a cena.
- [Resposta metálica](metallic-response.md): contrato anterior ao código e
  estudo scripts/study_cottage_metallic.py; hero/detail/front/ornament com mesma
  luz/câmera e cor. Controle só ambiente permanece igual, expondo o limite de IBL.
- [Ambiente procedural](procedural-environment.md): fontes pré-filtradas, controles
  de sol/ambiente/metal e contadores de preparo; mesma casa, sem mudar forma.
- [Revisão de arquitetura](elven-cottage-review.md): relações observáveis versus
  planta/partes ocultas inferidas. Materiais e testes não certificam fidelidade.

Esta é uma fila de casos e decisões, não uma declaração de capacidades entregues.
Nenhuma mudança em main/demo, UI, colisão arquitetônica ou importação de Blender
é necessária para executar os casos já implementados pelo MCP.
