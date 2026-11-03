# Cornijas contínuas e cobertura da entrada

Registro da receita v3/f0f9664. Continuidade corrente na
[ornamentação em relevo](cottage-relief.md), receita v4; contornos/estrutura desta
revisão foram preservados.

Continuidade de b33a76c após retorno positivo e autorização de prosseguir.
Close nativo `artifacts/elven-cottage/artifacts/detail.png` mostra beiral com
elevação local, descida e nova subida. `front()` somava Gaussian centrado na
entrada à subida lateral; a inversão de curvatura foi autorada, não imposta pelo
Sweep. Cornijas ainda usavam anéis/faces calculados no cliente como polilinhas.

## Alternativas e recorte antes da alteração

1. Corrigir curva e aplicar Sweep existente: compartilha contorno explícito com
superfície, admite perfil chanfrado e torna custo/tolerância observáveis.
2. Criar patches paramétricos nativos: útil para superfície inteira, mas ainda
não necessário para corrigir o contorno; dependência maior de amostragem/domínio.
3. Assemblies/componentes/materiais por região: fricção real, porém contrato
distinto de curvas; manter discussão aberta em cottage-structure.md.

Adotado1 nesta unidade. Elevação frontal monotônica por Bézier, perfil transversal
cúbico e controles compartilhados nas molduras. Não equivale a nova ferramenta
nativa nem a plantas persistentes para telhas. Bay conserva a implantação anterior;
revisar seu ápice sob a empena sem tratar a imagem única como planta métrica.

## Checklist estático anterior

1. Só receita/autoria, sem tick novo.2. Contar antes da publicação e manter
limites; substituir backing de prismas repetidos por casca indexada com bordas
compartilhadas, se necessário.3. Controles finitos/publicação f32 existente.
4. Superfície e perfis usam a mesma curva, sem copiar amostras para obter frames.
5. Mesmos caps de mesh/documento/assets, planejar spans pelo dispatcher.
6. Renderer/luz/câmera hero constantes na comparação.7. MCP headless próprio,
nenhum controle da demo.8. Preview/undo/persistência/replay da receita conservados.
9. Rodar verificação própria e recortes nativos antes/depois; não repetir Rust
completo sem alteração Rust.10. Ainda sem solver CAD/interior/colisão/material
por face/IDs de componentes ou certificado global de junções.

## Resultado e dificuldades reais

Receita clientev3: `roof.front_eave` são quatro controlesXY monotônicos, em vez
do Gaussian anterior. Há um trecho reto antes dessa cúbica com tangente comum.
O perfil transversal é cúbico, com controles de altura monotônicos em cada água;
a elevação frontal usa pesos[0,.55,1,1]. Isso evita que o bordo alto crie uma
descida e subida na extremidade. Surface e cornijas derivam dos mesmos controles;
checagem em65pontos por curva/nível compara parametrizações e a folga do pórtico.
Não é reconstrução métrica da referência, nem certificação contínua da superfície.

Cornijas do telhado principal e borda semicircular da bay agora usam Sweep:
perfis chanfrados, frames e tesselação nativos. Semicírculo aproximado por duas
Béziers de quarto de círculo, erro geométrico circular não equivale à tolerância
do planner. Câmera/luz/renderer continuam iguais a b33a76c. A cobertura da bay usa
ápice5.25m e perfil cúbico finito no beiral; preserva o volume/lado da janela.

Backing passou de140prismas isolados para3cascas indexadas fechadas, uma por água
principal e uma da bay:796v/1580t. Consultas MCP confirmam zero bordas abertas,
zero não-manifold/isolados; cada aresta tem orientações opostas nas duas faces.
Volumes assinados positivos foram conferidos separadamente. Não significa que
as três cascas ou o edifício inteiro estejam soldados.

A primeira captura revelou interferência entre triângulos da casca e do
revestimento inferior: diagonais diferentes em quads não coplanares e folga6mm.
Revestimento passou a seguir as mesmas diagonais e folga10mm. Não corrigir isso
com luz, material ou bias. Outra exposição era um painel vertical inacabado à
direita da entrada; completado campo verde e corpo de pedra que nasce do pilar.
O corpo usa uma face contínua; prismas em tiras causavam faixas de normais. Sua
extremidade ultrapassa a face da empena4cm para evitar caps coplanares. O relevo
e encontro construtivo ainda são simplificados; não declarar união topológica.

Arcos frontais receberam uma segunda moldura, e o campo sobre a entrada um
motivo inclinado. Nenhum kernel novo foi necessário. A casa tem357peças Sweep
(antes323),13refinadas pelo planner, limite máximo da linha central≈11.8135mm.
31720v/52988t armazenados,54208t no draw;16assets/21instâncias,1048v livres.
Em relação a b33a76c:−670v/−1080t armazenados mesmo com as molduras adicionais.
Esse balanço de geometria não é medição de FPS. Zero slots de assets permanece
limitação; componentes/relações e material por região continuam em aberto.

## Evidência e reprodução

`python3 scripts/demo_elven_cottage.py` gera13grupos de checagem: os11anteriores
mais contorno/folga e topologia/orientação do backing. Save/load, undo/redo,
replay/prefixo/tick60, repetição e viewport continuam incluídos. Duas gerações
integrais produzem cena/hero byte-idênticos. Artefatos correntes em
artifacts/elven-cottage/artifacts, sem novo binário ou janela.

Antes vem de `b33a76c:examples/elven-cottage.scene.json`, materializado em
artifacts/cottage-roof/examples/before.scene.json. Comparação reproduzida por:

```sh
python3 scripts/compare_cottage_structure.py --root artifacts/cottage-roof \
  --before artifacts/cottage-roof/examples/before.scene.json \
  --baseline-image artifacts/cottage-structure/artifacts/after-hero.png
```

Ferramenta agora aceita diretório e PNG de baseline explícitos: reutilizar uma
comparação intermediária não pode compará-la silenciosamente ao PNG da v1.
Relatório registra fonte/hash, câmera/luz iguais e baseline pixel-idêntico;
views hero/entrada/clay em artifacts/cottage-roof/artifacts.

Regressão MCP recortada render/sweep/plants:3cenários/16grupos passaram no mesmo
build53d31b5b0cead1f3, além dos13grupos da casa. Sem Rust/shader/Cargo novo; gates
857Rust e86cenários/352grupos completos do núcleo permanecem a evidência anterior,
não uma suíte repetida nesta unidade. Python compile/fmt/diff check aprovados.

## Revisão estática final

1. Só autoria, sem nova frequência no runtime.2. Removidas paredes internas dos
prismas de backing;796v ficam compartilhados em três ilhas, buffers temporários
limitados pelas linhas/colunas da receita.3. Curvas avaliadas no cliente com
float64 e publicadas/validadas em f32; Sweep continua nativo.
4. BisseçãoX fixa40iterações por ponto da receita; não há solver infinito,
resolução de superfície adaptativa ou custo por frame novo.
5. Caps e slots mantidos;1048v disponíveis, nenhum limite elevado.
6. Mesmo renderer/material/luz/câmera; comparação clay separa aparência.
7. Processos MCP próprios/headless; main/demo e outra worktree intactos.
8. Preview/undo/persistência/replay do documento exercitados; sem ID novo salvo.
9.13checks da casa e16da regressão recortada, repetibilidade e revisão visual
real. Garantias numéricas declaradas acima não implicam fidelidade ao PNG.
10. Mantidos: avaliação síncrona/JSON/staging, frames discretos e superfícies
calculadas no cliente, material único por asset e encontros modulares. Ainda há
diferenças na inclinação/encaixe das coberturas e densidade dos relevos/jardim.
