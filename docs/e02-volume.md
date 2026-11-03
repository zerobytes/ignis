# E02 — nova construção a partir da imagem

Pedido: análise e construção novas, sem reaproveitar geometria/receita/inventário
anteriores. Fonte original, skill atual e ferramentas/schema públicos MCP.
Contexto da conversa permanece; não é avaliação cega. Bootstrap, transporte,
recortes determinísticos e acompanhamento não são fonte de geometria anterior.

## Análise antes da construção

[Lista nova](e02-volume-list.md),188tarefas após22recortes realmente abertos e
9revisitações. [Dados estruturados](references/image-to-3d/e02-volume-analysis.json)
preservam fonte/hash, janelas exatas dos recortes, hipóteses e estados anteriores
à autoria. Lista entregue antes de publicar geometria. Nenhum item é aprovado
por estar descrito. Root corrente próprio: `artifacts/house-e02-volume`.

Hipótese inicial: corpo alto à direita, ala baixa à esquerda, varanda aberta
diante da ala, galeria sobre ela e pátio descoberto. Eixos: X ao longo da fachada,
Y para cima, +Z para o pátio. Profundidades/traseira/interior são hipóteses.
Portas ficam nas paredes posteriores à linha de arcos. Coberturas e paredes
derivam de coordenadas do objeto, sem planos definidos por contornos da câmera.

## Revisão estática antes da geometria

1.Autoria/inspeção/captura explícitas, sem tick/frame novo.2.Buffers Python e
JSON limitados pelo schema antes de publicar; compartilhar materiais/famílias.
3.Índices inteiros, float64 na geração e f32 no MCP; conferir geometria retornada.
4.Fontes paramétricas para apoios e regras geométricas reutilizáveis para vãos,
volumes e coberturas; sem busca de profundidade por pixel.5.Tetos consultados:
16assets/32instâncias/16384v totais/32768t/8192v por asset/131072t desenhados.
Não cortar inventário ou aumentar limites para caber.6.Pillow/NumPy já presentes,
recortes e renders somente sob demanda; um root estável.7.Processo MCP próprio,
headless e serial; não tocar demo ou outro checkout.8.Revisão esperada, undo,
save/load/replay; fontes e ensaios anteriores preservados.9.Inspecionar sólidos
reais após f32, vãos/jambas/interior, encontros, apoio e vistas distribuídas,
antes de detalhe; integridade não certifica fidelidade.10.Custos de autoria,
triangulação/JSON/captura e verso inferido mantidos, sem FPS ou fidelidade98%.

## Construção e revisão espacial

Receita independente: `scripts/e02_volume.py`; cena nativa
[house-e02-volume.scene.json](../examples/house-e02-volume.scene.json).
Não importa receitas, cenas ou listas anteriores. Reusa somente o auditor
geométrico da ferramenta e o template genérico de visualizador, sem geometria
embutida. Não houve mudança no kernel, schema ou budgets para acomodar a casa.

Dois corpos ocos têm superfícies externas e internas, piso, teto e jambas;
as paredes não são painéis colocados pela câmera. Portas e janelas atravessam
essas paredes. Varanda aberta, pátio, apoios e coberturas são conjuntos distintos.
Uma passagem na parede comum conecta os ambientes: hipótese não observável
acrescentada como VOL-COR-13 durante a revisão espacial. A lista foi de188para189
itens, sem alegar que esse requisito constava da primeira entrega.

As quatro colunas usam `preview_plant/edit_plant/inspect_mesh`; perfis por11estações,
fonte96v/84t compartilhada e escala por apoio. As demais peças são meshes
indexados publicadas pelo MCP. As coberturas são superfícies com espessura,
panos posteriores e bordas. Remates, água-furtada, dois grupos de balaustradas,
portas, galeria, janelas, motivos, degraus, vasos, móveis, lanternas e vegetação
têm representações ainda simplificadas. Folhas e telhas podem ser superfícies;
não são usadas para substituir o corpo da casa.

Iterações corrigiram: encontro de segmentos no interior das paredes, vãos
atravessando teto, apoios abaixo do forro, galeria excessivamente oculta,
arcos angulosos, posição inicial do acesso, triangulação de degraus curvos,
amostragem da cumeeira deixando partes da parede aparecerem sobre o telhado
e ambientes sem ligação interna. O topo da câmera precisou desvio de vertical
para respeitar a guarda pública contra direção paralela ao world-up.

Capacidade: duas expansões foram recusadas no preflight antes de mutação
por ultrapassar16384v. Remover centros desnecessários em triângulos/quads e
polígonos estritamente convexos planos, fechar loops de tubos periodicamente
e eliminar grades interiores redundantes de pisos planos preservou as formas
e os pontos de fronteira. Não aumentar tetos ou excluir famílias do inventário.
Gaps da primeira tentativa impediram a publicação até serem resolvidos.

## Evidência e limites

Build público9860a09115365ad3, Scene67, macOS/Metal offscreen em processo próprio.
15assets/18instâncias,16295v/26260t armazenados e26512t desenhados.89v globais
livres. [Verificação rastreável](references/image-to-3d/e02-volume-verification.json):
13checks aprovados; os três corpos e a fonte de coluna têm uma componente,
nenhuma borda aberta, orientação consistente e volume positivo. Coberturas
também têm contornos fechados. Esses testes são geométricos, não classificação
automática de casa, prova global de interseções ou colisão física.

17testes de segmento contra triângulos reais dos corpos verificam15vãos externos
e dois lados da passagem; segmento adicional atravessa as duas paredes comuns.
Eles não substituem inspeção de todo o contorno nem tornam a casa navegável no
runtime: meshes visuais não criam colisores de gameplay implicitamente.

Captura repetida, load em processo novo, undo de câmera e replay estático tick0
reproduzem os mesmos pixels. Render SHA256
7522862c471de38be489ea64019383651e3ed7d61ef12705d00c1501819081b4.
Cena SHA2560657baeb9d92ac498fd8f7b183409c6cf96139999fb900dee22eec2087b9fd22.
Imagem original e cena/render do ensaio anterior preservados por hash.

Seis ângulos nativos e22pares fixos realmente abertos mostram avanço em
coerência3D e grande divergência visual. MAE0.275375/RMSE0.351458/SSIM0.275501,
sem alinhamento, em1402×1122. Fundo/luz/câmera são diferentes: não são percentual
de fidelidade nem ranking silencioso contra outra tentativa. Recortes de tela
eventualmente reduzidos pela ferramenta de exibição não mudam os arquivos ou
as janelas usadas para medir.

[Revisão por recorte](references/image-to-3d/e02-volume-review.json) e
[pendências por item](references/image-to-3d/e02-volume-progress.json) mantêm189
entradas; vínculos a famílias de objetos não significam tarefa executada/aceita.
Faltam reprodução dos grafos/curvas e perfis esculpidos, configuração exata dos
espigões, telhas pequenas arredondadas, densidade/variedade de vegetação,
paginação convincente, desgaste/musgo, vidro/reflexos e iluminação quente local.
Painéis verdes são aproximações opacas; cores quentes das lanternas não são
luzes locais. A água-furtada é volumétrica, mas seu encontro não recorta o pano
hospedeiro. Traseira, interior e profundidades continuam hipóteses.

`artifacts/house-e02-volume/index.html` reúne capturas/pares/diagnósticos e
`orbit.html` usa o template genérico com a geometria nova. Sintaxe JS e payload
verificados; não se alegou compilação WebGL ou interação humana no navegador.
`plan.png` é corte horizontal emY1.63 dos triângulos exportados, desenhado no
cliente depois da construção, não um blueprint entregue antes dela. Distingue
parede, vãos e passagem interna e deixa explícita a hipótese de planta.
Não há novo comando de planta/blueprint nativo nem integração do gate ao MCP.
Esta é autoria experimental pelo fluxo atual, não um gerador automático novo.

## Reprodução e revisão estática final

Usar o Python já disponível com NumPy/Pillow e o binário indicado na receita:

```sh
python3 scripts/e02_volume.py --analyze --probe
python3 scripts/e02_volume.py --stage detail --verify --report
```

1.Só autoria/consulta.2.Centro de polígonos apenas quando necessário, pisos sem
grade interior redundante, fonte de coluna compartilhada.3.Cooking explícito1e-6
seguido de f32; inspeção usa os dados efetivamente salvos.4.Uma receita gera
estrutura/detalhe e relatórios; nenhuma interseção de raios da câmera para
posicionar partes.5.Tetos consultados e recusas anteriores à publicação;
orçamento apertado mantido explícito.6.Capturas/I/O sob pedido e um root atual.
7.MCP serial em processo próprio, encerrado em finally.8.Revisão esperada,
revalidação depois de detalhes/load, undo/replay.9.13checks,22pares, seis vistas,
AST/JSON/links/JS/diff e referência preservada.10.Mantidos custo de autoria
Python/JSON/GPU explícita, aproximações de forma e ausência de provas físicas
ou semânticas. Nenhuma promessa de FPS/tempo de inferência/fidelidade98%.
