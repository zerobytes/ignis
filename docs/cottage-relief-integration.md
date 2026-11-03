# Relevos, materiais e junção da casa — integração

## Decisão e checklist antes do conteúdo

Integrar1054577 sobre o núcleo próprio EAF-120 depois de validá-lo. Handoff
comparado com f0f9664: somente bronze.geometry difere; carved-stone/arch e demais
assets/fontes/instâncias/luz/câmera são iguais. Não integrar automaticamente o
trabalho posterior de caixas chanfradas ou Wedge/NPCs. Main e checkout de plantas
preservados. Reservas76/80/renderer existentes permanecem como estão.

Salvar a fonte de aparência v3 corrente como entrada de comparação no diretório
estável artifacts/cottage-relief-integration antes de substituí-la. Atualizar a
recipe de lookdev conscientemente para fontev4/a742696f…57e6 e contagens31956v/
53448t/54668draw. Rodar lookdev e estudo da mola no mesmo binário: novo bronze
com materiais/ambiente e pedra reautorada por spans[1,9]. Preservar o teste da
mola porque seu asset é byte-exato no handoff; não usar índices do bronze para
inferir partes da pedra. Cena integrada de trabalho no output cottage-spring.

Checklist proporcional:1.só autoria/capturas explícitas;2.sem buffer por frame;
3.mesmo f32/fontes hash-bound;4.não recomputar curvas ao mudar material/luz;
5.tetos16slots/32768v mantidos, conteúdo deve caber antes de publicar;
6.layouts/Vertex/Globals/GpuMaterial/probe-cache intactos;7.sem deps/threads;
8.old/new câmera/luz/aparência iguais, só bronze.geom antes da correção local;
9.estudos afetados/undo/save/load/replay/viewport e views repetidas, sem repetir
Rust/MCP completo se fingerprint de src/Cargo/build.rs permanecer o validado;
10.fidelidade artística, relevo por sobreposição, superfície por seção e miter
continuam abertos. Sem FPS alegado.

## Fricção encontrada na captura, antes da correção

O comparador inicial detectou hero/finial/chaminé diferentes, mas friso byte-igual.
Fonte confirma os sprays em porch.front+0.10; a face dos capitéis tem offset
0.45/2=0.225. Relevo/caules estão atrás dessa face. Não aceitar contagem de19folhas
como prova de19folhas visíveis. Corrigir apenas posição dos dois sprays no bronze,
com porch.relief_front_offset=0.23, relief_bottom=3.38 e relief_height=0.22
explícitos na recipev4, defaults0.10/3.43/0.25 para preservar
a reprodução anterior. Não ocupar versão5 que a outra frente reservou para
caixas chanfradas. Paramétrico de conteúdo, sem lógica de casa no runtime.

Publicar só bronze.geometry reavaliada por Sweep/MCP; manter restante da fonte
canonical72 byte-equivalente em conteúdo, triângulos/contagens/material/rig e
layouts idênticos. Preflight/preview antes de publicar. Novo hash e câmeras
fixas no estudo. Limite novo identificado: falta query genérica de superfície
visual/folga para apoiar peças decorativas; offset manual continua custo da autoria.

## Resultado integrado

Núcleo fc8fc2f/source eeef1b9ea254661a:877Rust/88executáveis, Clippy/fmt/build e
90cenários/372grupos MCP headless aprovados. Este merge é somente conteúdo/docs/
clientes: fingerprint e binário inalterados, sem repetir a suíte completa.
Reexecutados3grupos lookdev,4normais/mola e3integração, além da publicação MCP
localizada e py_compile/fmt/diff check.

Cena de trabalho com materiais/metal/ambiente/relevos e mola[1,9]:16assets,
21instâncias,31948v/53436t armazenados,54656t draw,820v livres/zero slots. Probe
mais LUT1bake cada, mesmos148984B lógicos. Relevo só muda bronze.geometry antes
da correção da mola; esta só muda carved-stone.geometry. Mesmos IDs/fontes/rig/
aparências/câmeras/luz. Quatro vistas repetidas por estágio e baselinev3 com hash
conhecido exato. Save/load/reset/60ticks/replay/viewport da cena final exatos.

Publicação corrigida da fonte canônicav4 mantém versão72 e todo o restante da
cena: SHA1b4a79095e3fa487499ace281e432850a0603d1ded52ad7d9fc28e7ec74a3795.
Das181varreduras do bronze, só87..94 foram reposicionados/redimensionados:272v,
triângulos e vértices/canais dos outros spans exatos. Geometria gerada via Sweep;
regeneração dos parâmetros legados é igual à fonte na inspeção nativa antes
da alteração.
Y medido3.376968..3.600833m cabe na face3.375..3.615; menorZ3.374845m,
face3.375m: incorporação rasa0.155mm, não o afastamento de centímetros do primeiro
ensaio. Nenhuma prova global de colisão ou solda implícita. Capturas agora
mostram os motivos completos abaixo do beiral, sem elevar limites.

Parâmetros de conteúdo explícitos relief_front_offset0.23/relief_bottom3.38/
relief_height0.22; omitidos preservam0.10/3.43/0.25 para a receita1054577.
Não é novo contrato de engine. Query genérica de apoio/folga de superfícies
visuais e encontros angulares continuam lacunas; seções ainda exigem densidade
escolhida pela autoria. Pedra/vidro/jardim e realismo completo permanecem abertos.

## Reprodução

Com o histórico das fontesfc8fc2f/1054577 disponível:

```sh
python3 scripts/study_cottage_relief_integration.py --publish-source
python3 scripts/study_cottage_lookdev.py
python3 scripts/study_cottage_normals.py
python3 scripts/study_cottage_relief_integration.py
```

A flag de publicação é explícita: atualiza só bronze.geometry e o binding da
recipe de lookdev. Sem flag, a ferramenta só compara/captura. Bootstrap do
before usa a fonte histórica e aparência corrente, após provar que as fontes
só diferem em bronze.geometry; SHA esperado do PNG anterior evita baseline
circular. É evidência macOS deste renderer, não validação multiplataforma.
Leitura de arquivos normaliza númerosf32 para comparar com a inspeção nativa;
hashes/bytes das fontes são verificados separadamente.

Saídas estáveis em artifacts/cottage-relief-integration: examples/integrated.scene.json,
artifacts/integrated-{hero,finial,chimney,frieze}.png, report.json,
placement-report.json e build-audit.json. Modelos/relatórios em cottage-lookdev
/cottage-normals também são correntes. Nenhum main/demo/check-out paralelo ou
árvore modificado, sem push/janela ou ganho de FPS declarado.
