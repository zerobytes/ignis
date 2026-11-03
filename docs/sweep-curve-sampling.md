# Amostragem independente por curva no Sweep

## Decisão anterior ao código

EAF-119 uniu reta+curva na mola da porta: incidências/cap deixaram um salto52.6°
entre normais. O planner uniforme refinou ambos os trechos e custou+72v/+148t.
A reta não precisava do refinamento. Outra frente usa seções para relevos;
a garantia central atual não certifica variação de seção/superfície.

Alternativas: campo adicional curve_spans com precedência sobre spans criaria
dois controles para o mesmo dado; remover spans quebraria clientes; amostragem
por comprimento de arco mudaria seções/material/frames e precisaria outro
contrato. Escolha: spans aceita inteiro1..128 ou lista de um inteiro por cúbica.
JSON escalar permanece igual. Rust usa enum não etiquetado Uniform/PerCurve.
Não é tipo persistido: resultado continua Geometry comum, sem versão nova.

Lista é explícita, não pedido automático de mínimo. Planner dobra apenas o
valor de cada curva que excede centerline_tolerance, até128. Escalar mantém
refinamento uniforme anterior. Plano devolve requested_spans/spans na mesma
forma, contagens exatas e limites de erro central por curva; surface_validated
continua false. Geração não refina implicitamente: exige plano admitido.

Anéis=1+soma dos intervalos, junções compartilham um só anel. Na lista não
uniforme, seção usa u=(índiceDaCurva+t)/nCurvas. Não usa índiceDoAnel/totalAnéis,
que deslocaria estações ao mudar resolução. Para escalar/lista uniforme manter
as operações antigas exatamente, inclusive arredondamento do parâmetro; arrays
uniformes devem ser Geometry/canais iguais ao escalar. Coordenada longitudinal
continua comprimento poligonal acumulado; reduzir tesselação pode alterá-la.

## Checklist anterior

1.Só autoria/planner, sem tick/frame novo.2.Preflight pela soma antes dos buffers;
lista e limites têm no máximo16entradas, nenhum grafo.3.f64 temporário/f32 saída,
u16 contagens validadas1..128; mesmo shader/Vertex.4.Validar cadeia/perfil uma
vez; cursor de curva avança entre anéis, sem busca de vizinhança.5.Mesmos tetos
por asset/documento, até8tentativas por trecho, sem relaxar orçamento.6.Nenhum
recursoGPU/cache/layout novo.7.Sem thread/dep/lock; staging sequencial.8.Sweep
mantém changed_assets/guard de plantas, append recusa rig como antes.9.Testes
analíticos de contagens/tempo/seções e equivalência uniforme, erro central e
limites; Rust/Clippy/MCP completos e casa/replay/pixels.10.Custos mantidos:
planner não escolhe anéis por seção nem certifica superfície/auto-interseção;
menos vértices não equivale a FPS ou fidelidade artística.

## Limites de compatibilidade e resultado na mola

A representação Rust de Sweep.spans muda para Spans; consumidores Rust usam
Uniform(n)/PerCurve(vec) ou n.into(). Compatibilidade prometida aqui é JSON e
Geometry/canais para entrada escalar, não ABI/compilação de código cliente antigo.

Seções conservam seu parâmetro; uma estação que não coincida com nenhum anel
não ganha anel automaticamente. A lista não é refinamento por seção/largura,
curvatura do perfil ou erro de toda a superfície. É responsabilidade da autoria
escolher densidade adequada além da certificação da linha central.

Casa v3: cadeia da mola usa[1,9],88v/172t; variante uniforme10por curva usa168v/
332t.80v/160t a menos; em relação às duas peças separadas,−8v/−12t. Posições do
span unido iguais às originais sem o anel duplicado; demais spans/coords exatos.
Salto de normal52.6037°→0no contato examinado, mesmos materiais/luz/câmera.
Material longitudinal do span unido passa a distância poligonal contínua;
não há promessa de preservar o antigo reinício da coordenada da curva.
Quatro vistas da fonte antiga continuam byte-exatas no build novo.

## Revisão estática final

1.plan/generate só em autoria; runtime/GPU sem fase nova.
2.Preflight1+soma antes de reservar saída; cursores/frames por anel, listas≤16.
Plano retorna mais uma lista de limites≤16por peça. Uniforme aloca um vetor de
limites por tentativa (≤8), custo síncrono pequeno mantido por escopo, sem cache.
3.Spans enumu16/Vec<u16>, f64 temporário e f32final preservados; Rust API precisa
adaptação explícita, JSON escalar exato.
4.Cadeia/perfil validados uma vez por plano; cursor amortizadoO(curvas+anéis).
Refinamento explícito é independente; uniforme antigo continua dobrando todos.
5.Curvas≤16, spans1..128, peças≤256 e mesmos budgets de saída; rejeição prévia.
6.Sem layout/Vertex/GpuMaterial/textura/shader/readback novo, cache comum.
7.Staging sequencial existente, sem thread/lock/deps novos.
8.Mesma operação MeshEdit::Sweep e guards/append; saída Geometry comum mantida.
9.Quatro casos Rust novos para equivalência/tempo/erro denso/limites e três
novos grupos MCP para preview/atomicidade/persistência; regressão completa e
estudo da mola. Contagens finais e logs registrados após os gates.
10.Custos mantidos: validação/JSON síncronos e reavaliação por preview/commit;
sem seletor de anéis por seção/garantia de superfície/canto angular. Economia
numérica de vértices não implica ganho de FPS nem aceite artístico.


Gates do núcleo: build/source fingerprinteeef1b9ea254661a,877testes Rust/
88executáveis release/all-targets, Clippy-Dwarnings/fmt/build/py_compile/diff
check aprovados.90cenários/372grupos MCP headless,11testes de Sweep(4novos),
4grupos do estudo da mola e4vistas legadas byte-exatas. Audit em
artifacts/curve-sampling-build-audit.json, logs curve-sampling-{tests,clippy,
build,full-mcp}.log. Sem janela/main/push, uma saída de build corrente.
