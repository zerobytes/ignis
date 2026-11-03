# Folhas e ornamentação em relevo

Continuação autorizada sobre f0f9664. A frente de normais/terminações dos arcos
segue no outro checkout; esta unidade preserva paths/molduras/shading e trata
ornamentos. Referência: mesma imagem original, sem interpretação das costas como
medidas observadas. A ornamentação apresenta folhas com largura e nervura central;
o ensaio anterior usa sobretudo fios finos com uma folha apenas desenhada no ar.

## Recorte e alternativas antes da alteração

Adotar Sweep existente com perfil losangular convexo: espessura no eixo normal à
fachada, largura no plano e estações de escala formando lâmina afilada com nervura
central. Preserva matemática de frames/faces no kernel. Não chamar isso de nova
ferramenta nativa, escultura transferida, interpretação automática ou planta
persistente. A receita fornece controles, perfil e escalas, publicados pelo MCP.

Alternativas: folhas trianguladas pelo cliente (menos nativo), ampliar o contrato
para seção escalar suave/normal customizada (ainda não exigido), novo componente
ornamental persistente (mais identidade, porém contrato estrutural distinto).
Aplicação restrita a finial, chaminé, campo da entrada e friso; não trocar todo o
rendilhado nem encostar no diagnóstico concorrente dos arcos.

## Checklist estático anterior

1. Somente autoria, sem tick/frame novo.2. Pré-contagem e plano nativo antes da
expansão; reutilizar família bronze, sem gastar slots novos.3. Perfil convexo
finito/CCW, escalas positivas e cap finito na ponta; não esconder degenerescência.
4. Math de tesselação/frames fica em Sweep, controles da composição no cliente.
5. Teto32768v/16assets mantido; só1048v livres no início.6. Mesma câmera/luz/
renderer/material na comparação.7. Cliente MCP headless próprio, sem app aberto.
8. Preview/undo/save/load/replay da receita; partes novas identificadas no relato.
9. Inspecionar folhas sem bordas abertas/não-manifold e repetir captura/geração;
closes reais e comparação clay, sem prometer fidelidade.10. Seções lineares,
caps e peças independentes são limitações; nada de weld/rig/LOD novo.

## Resultado

Receita clientev4.19lâminas em relevo distribuídas no finial, chaminé, campo da
entrada e dois pontos do friso da varanda. Perfil losangular4lados, trajetória
Bézier ligeiramente abaulada para fora da fachada,8intervalos e9estações de escala
positiva produzem36v/68t por lâmina. Ao todo684v/1292t pertencem a essas folhas;
substituir três motivos antigos mantém o custo líquido em+236v/+460t.

A primeira composição ficou densa no close. As folhas foram afinadas e suas
direções mais abertas, com raiz no eixo do caule; largura e profundidade foram
ajustadas separadamente. A lâmina continua afilada com cap finito, não ponta de
área zero. Exemplo genérico em `examples/foliate-relief.edit.json`: edit_mesh e
preview_mesh_edit, sem comando de casa/folha novo no kernel.

Cena:31956v/53448t armazenados,54668t desenhados;16assets/21instâncias e360peças
Sweep,13refinadas pelo planner.812v e zero slots livres. Não houve aumento de
limites, mudança de renderer/material/normal ou novo build. Somente geometria do
asset bronze difere de f0f9664: carved-stone, demais assets, plantas, instâncias,
câmera/luz/aparências e restante da cena são iguais. Isso preserva os arcos para
o diagnóstico paralelo, sem supor que seus resultados já estejam integrados.

## Verificação e evidência

14grupos próprios passaram: os13anteriores mais topologia/área/taper dos relevos.
Consulta MCP do bronze confirma zero bordas abertas/não-manifold/isolados. Cada
lâmina tem área transversal inicial positiva, meio maior e ponta menor positiva,
checados nos vértices efetivamente publicados em f32. `native_reliefs` no relatório
guarda índice da varredura e intervalos de vértices/faces para inspeção. São
referências deste resultado, não IDs persistentes de componentes.

Duas gerações integrais produziram cena e hero byte-idênticos. Cena compacta
publicada2.189.575bytes, SHA
`a742696faca960b08684a8a269975477c6544568de50b4b38b578536649a57e6`;
reaberta via MCP igual ao export nativo, PNG igual à captura do ensaio.
Exemplo isolado36v/68t admitiu plano/preview/publicação iguais e malha fechada;
captura real em artifacts/cottage-relief/artifacts/leaf.png. Render e Sweep:
2cenários/12grupos MCP headless aprovados no mesmo build53d31b5b0cead1f3.
Sem Rust/shader/Cargo novo, nenhuma repetição da suíte Rust inteira nesta unidade.

Antes: f0f9664, SHAa69488c31be211e7f9d1d5209b38629a7f59d877cea1d79d33c8cb8092da98ed.
Comparação com câmera/luz iguais, baseline pixel-exato, hero/entrada/clay:

```sh
python3 scripts/compare_cottage_structure.py --root artifacts/cottage-relief \
  --before artifacts/cottage-relief/examples/before.scene.json \
  --baseline-image artifacts/cottage-roof/artifacts/after-hero.png
```

Evidências em artifacts/cottage-relief: logs authoring/render/sweep/comparison;
artifacts/report.json, preservation.json, publication.json e leaf-report.json.
Vistas nativas adicionais finial/chaminé/friso e câmeras em ornament-views.json.
Capturas correntes da casa em artifacts/elven-cottage/artifacts. Referência e
árvore anteriores preservadas; sem tocar main, demo ou checkout paralelo.

## Dificuldades que permanecem

Revisão posterior do close frieze confirmou que as duas sprays quase desaparecem
atrás dos capitéis: relief emz=front+.10 contra face frontal do capitel+.225.
Sua presença geométrica não comprova contribuição visível. A frente paralela
trata colocação com parâmetro explícito relief_front_offset; essa correção ainda
não está integrada neste registro da v4. Finial/chaminé/entrada têm contribuição
visível nas capturas próprias. Não contar friso como acabamento visual aceito.

O planejador do Sweep só limita erro da linha central. Ele não decide quantos
anéis uma variação de largura/profundidade exige; aqui8intervalos alinham as
9estações explicitamente. A silhueta é poligonal e a escala linear por trecho;
não prometer C1 da seção ou erro de superfície certificado. Perfil/nervura mais
ricos e amostragem por erro de superfície são alternativas para um contrato futuro.

Caules/folhas são peças fechadas sobrepostas, não um único relevo soldado; ainda
há simplificação de desenho e junções. Normal/shading dos arcos pertence à outra
frente. Sem integração de seus materiais/ambiente nesta worktree, nem aceite
artístico/realismo demonstrado. Inspeção por componente e autoria nativa persistente
das relações ainda faltam; não usar a receita Python como substituto dessa feature.

## Revisão estática final

1. Apenas autoria.2. Plano nativo preflight antes da publicação; mesmo asset
bronze;19folhas/684v, sem slot adicional.3. Perfil estritamente convexo e escalas
finitas positivas, verificadas na geometria final.4. Frame/faces/material coords
gerados pela engine, sem replicá-los no cliente.5.812v livres e caps existentes.
6. Câmera/luz/material/renderer constantes; vistas clay para separar aparência.
7. MCP próprio/headless, sem threads/locks novos ou janela.
8. Undo/load/reset/replay/prefixo/capturas exercitados; metadados do relatório não
mudam formato da cena.9.14checks da casa,12MCP recortados, exemplo isolado e
preservação do restante da cena; Python compile/fmt/diff check aprovados.
10. Mantidos: JSON/staging síncronos, densidade explícita e peças sobrepostas;
sem ganho de FPS, preservação automática de rig, solver ou fidelidade alegados.

## Integração de aparência e colocação

O handoff1054577 foi integrado ao núcleo de materiais/ambiente/normais e
resolução por curva. Inspeção da captura detectou friso enterrado nos capitéis;
recipev4 ganhou parâmetros opcionais de posição/tamanho, mantendo os defaults
anteriores. Só8varreduras do bronze foram corrigidos via Sweep, resto da cena exato.
Fonte atual e prova visual/contagens/reprodução em
[cottage-relief-integration.md](cottage-relief-integration.md).
