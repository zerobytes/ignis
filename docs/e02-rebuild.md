# E02 — construção a partir da lista consolidada

Escopo autorizado: executar a lista de248tarefas entregue em763ba8d. Preservar
esse inventário como baseline; registrar execução e evidência separadamente.
Modelagem baseada na imagem, recortes/lista e interface pública MCP; não carregar
geometria/receitas dos ensaios anteriores. Contexto anterior permanece.

## Estratégia anterior à autoria

Usar contornos/âncoras observados e planos hospedeiros explícitos. Interseção
raio–plano no cliente transforma pontos2D em coordenadas3D sob uma câmera
declarada. Paredes compartilham planos, coberturas têm profundidade e superfícies
próprias, vazios são vazios. Isso é uma hipótese de reconstrução de vista única,
não prova de profundidades reais ou autorização para usar a foto como textura.
Não carregar a imagem como aparência da geometria.

Colunas/fontes repetidas podem usar plantas nativas; peças compostas precisam
de geometria indexada e correspondência semântica por intervalos de vértices/
triângulos no cliente. Agrupar material não equivale a validar a peça. API e
limites consultados antes de publicar. Não mudar engine/tetos para acomodar
o ensaio. Prioridade organiza a execução sem remover tarefas.

## Revisão estática anterior

1.Autoria/captura sob demanda; nenhum tick/frame novo. 2–3.Vetores/JSON no
cliente, buffers limitados pelo preflight, fonte original1402×1122. 4.Fontes,
curvas e regras compartilhadas só quando a referência sustenta repetição.
5.Um root estável, budgets públicos e nenhum histórico de builds. 6.Capturas
offscreen explícitas e I/O de PNG/JSON, nenhum modelo externo ou textura de foto.
7.Processo próprio sequencial com revisão esperada. 8.Undo/carga/replay e
checagens antes de mutar; preservar ensaios e demo. 9.Verificar geometria,
reprojeção, capturas e registro por item, além da integridade de persistência.
10.Câmera e profundidade hipotéticas, custo de geometria e projeção no cliente;
nenhum ganho de FPS, porcentagem semântica ou fechamento físico presumido.

## Resultado desta tentativa

Cena editável [house-e02-rebuild.scene.json](../examples/house-e02-rebuild.scene.json),
receita [e02_rebuild.py](../scripts/e02_rebuild.py) e publicação de evidências
[e02_rebuild_report.py](../scripts/e02_rebuild_report.py). Root corrente:
`artifacts/house-e02-rebuild/`; `index.html` reúne referência/render, 28 pares
fixos, vista auxiliar, projeções da geometria salva, métricas e lista de execução.
O original e os ensaios anteriores foram preservados.

Há três coberturas curvas, 480 telhas quadrangulares em fiadas desencontradas,
cinco remates, água-furtada, galeria, quatro colunas com fonte paramétrica nativa,
arcos da varanda, duas portas, três janelas baixas, grande janela com duas redes
ornamentais distintas, duas janelas laterais, dois retornos vazados da cerca,
dois pilares de entrada com vasos, duas escadas independentes, piso paginado,
faixas planas/medalhão, banco, cadeira, objetos internos, lanternas e vegetação.
Presença não equivale a reprodução fiel de cada forma.

O [registro dos 248 itens](references/image-to-3d/e02-rebuild-progress.json)
conserva IDs, descrições e critérios da baseline. 231 itens têm alguma
representação parcial vinculada; 17 ainda não têm construção. Nenhum item é
considerado validado artisticamente. Relações têm vínculos com os dois lados;
faixas de vértices do cliente são convertidas em índices após weld, sem alegar
IDs topológicos estáveis no servidor. O fonte compartilhado de folhagem tem
12 instâncias e plantas localizadas; não contar isso como 12 espécies distintas.

Apenas as colunas usam `preview_plant`/`edit_plant`/`inspect_plant`: fonte de
12 estações, 104 vértices e 92 triângulos, quatro instâncias. Os outros elementos
usam geometria indexada pelo MCP. `blueprints.png` contém projeções ortográficas
no cliente da geometria realmente salva; `column-source.png` projeta a malha
retornada por `inspect_mesh`. Não há exportador de blueprint nativo neste build.

## Iterações e diagnóstico visual

1. Captura estrutural: contornos/vãos posicionados, parede posterior ainda
ocultava o quarto apoio. Recuar seu plano preservando os pontos da imagem
separou parede, porta e apoios dianteiros. A profundidade continua hipótese.
2. Compilação detalhada inicial: 24.245 vértices, recusada pelo preflight antes
   de publicar. Simplificação das curvas até 0,9 pixel de erro no desenho,
   seções triangulares, menor tesselação e fonte instanciada de maciço baixo
   mantiveram as famílias, com acabamento explícito mais grosseiro. Não ampliar
   limites ou chamar esse resultado de fidelidade suficiente.
3. Normais de superfícies derivadas da imagem estavam invertidas. Corrigir a
   orientação mudou leitura de piso/fachadas; não era um problema de material.
4. Raios de tubos por diferença de interseções no plano explodiam em planos
   rasantes da cerca. Usar profundidade de câmera/focal corrigiu as expansões
   espúrias. Superfícies curvas são geometria; foto não é usada como textura.
5. Revisão dos 28 pares: forma e densidade das telhas, relevos, folhagem, desenho
   dos ramos, perfis e encontros continuam bastante diferentes. A grande rede
   clara recebeu grafo específico, independente da regra das janelas pequenas.
   Faixas de piso viraram inlays planos em vez de tubos elevados.

Os pares foram abertos em sete pranchas com miniaturas apenas para exibição;
os recortes individuais conservam os pixels originais, sem deslocar ou ajustar
cada peça para melhorar a comparação. A projeção auxiliar mostra volume de
coberturas e planos separados, mas também espessuras/fechamentos incompletos.
Não há prova de manifold, colisão, cômodos, verso ou reconstrução volumétrica
correta. A área lisa no frontão, galeria, água-furtada e retorno direito da cerca
continuam divergentes. Vidro transparente/reflexos, desgaste esculpido, musgo
nas juntas e iluminação quente local estão ausentes ou incompletos; transmissão
traseira da engine não foi apresentada como transparência de vidro.

## Validação técnica e reprodução

**Atualização estrutural:** a inspeção orbital posterior reprovou esta montagem.
O preflight de [corpo 3D](reconstruction-structure.md) agora interrompe a receita
histórica antes de publicar, pois ela não fornece o corpo principal fechado.
Os comandos de geração abaixo registram a reprodução histórica e atualmente
falham com esse diagnóstico; não são uma receita aprovada. `--verify` continua
um teste de integridade do artefato salvo, sem autorizar sua publicação.

Build MCP `9860a09115365ad3`, macOS Apple Silicon/Metal, offscreen em processo
próprio. Cena67, 16 assets/30 instâncias, 16.255 vértices/18.450 triângulos
armazenados; 129 vértices livres. Material procedural por asset, juntas simples
em coordenadas XY na pedra e reflectance aproximada em telhas/ornamentos.
Não há materiais fotográficos, IA externa, alterações de renderer ou tetos.

Captura repetida, carga em processo novo, undo da câmera e replay **estático,
tick0** reproduzem pixels exatamente. Geometria/índices finitos, vínculos e
budgets conferidos; fonte original preservada. Comparador nativo: RGB MAE
0,165061; RMSE0,229390; SSIM de luminância0,396895, com mesma grade1402×1122 e
28 regiões fixas em dois lotes. Não são percentuais de fidelidade, nem ranking
com ensaios de outra câmera. `verification.json` separa integridade aprovada de
`fidelity_accepted:false`. Página apenas verificada estaticamente, sem alegar
teste de interação de browser. Nenhuma janela da demo foi manipulada.

Reproduzir com Python contendo NumPy/Pillow e o binário informado na receita:

```sh
python3 scripts/e02_rebuild.py --stage detail
python3 scripts/e02_rebuild.py --verify
python3 scripts/e02_rebuild_report.py
```

## Revisão estática posterior

1. Autoria/compilação/captura explícitas, sem tick novo. 2–3. Buffers Python/JSON
locais; valores finitos e índices/limites checados; nenhuma dependência no Rust.
4. Curvas simplificadas/weld no cliente e um maciço instanciado12vezes evitam
repetição de buffers; lista de tarefas não foi cortada. 5. Limites admitidos
reais preservados, estado de execução parcial explícito. 6. I/O PNG/JSON e
projeções apenas neste root estável; cena exportada atual sem histórico de builds.
7. MCP sequencial/revisão esperada, processos próprios encerrados, sem locks
novos. 8. IDs de assets/instâncias/fontes preservados na persistência/undo/replay;
intervalos semânticos remapeados após weld. 9. Verificações nativas e pares
visuais distinguem técnica/fidelidade. 10. Mantidos: tesselação grosseira, normais
suaves por grupo, materiais compartilhados, hipótese de profundidade e custo de
subprocesso/JSON por edição. Sem ganho de FPS alegado; Rust não mudou e a suíte
completa do kernel não foi reexecutada para esta autoria de conteúdo.
