# Auditoria de aparência planetária — EAF-199

Em03/11/2026 Guilherme reconheceu mais variedade, mas reprovou a aparência:
elevações arredondadas não reproduzem a topologia dos testes do nanites. A
validação funcional anterior não equivale a aceite artístico. A correção visual
continua pendente; esta unidade investiga forma e configuração sem mudar a demo.

## Handoff verificado com nanites

Consulta explicitamente autorizada; revisão read-only no mesmo checkout, sem
execução concorrente de Cargo/GPU ou alteração da janela PID10923.

| Aspecto | Referências aprovadas | Demo planetária atual |
|---|---|---|
| Forma | `mountain_recipe.py`: crista anisotrópica sinuosa, perfil longitudinal irregular, encostas por potência e dobras oblíquas | Planet1 `Noise`/`Raise`: calotas isotrópicas suaves somadas |
| Composição | Duas fontes com fases diferentes em cinco instâncias com escala/rotação/distância variadas; depois Terrain2 com várias cristas | 18ombros/6cumes/4depressões/3noises |
| Material | Ridged,4octaves,bump0,04,warp0,45 | Noise,2octaves,bump0,warp0,3 |
| Atmosfera | Density0,00006, sky/mountain colors distintos por layers | Density0,00065, sem layers |
| Resolução | Receita n110 em4km×1,6km; expanded-world4m perto/32m fora | Nível12/N64,~49m no centro da face |

Fontes: `scripts/mountain_recipe.py`, `scripts/verify_atmosphere_landscape.py`,
`examples/expanded-world.recipe.json`, `docs/terrain-render-integration.md` e
`docs/expanded-world.md`. Capturas anteriores indicadas pelo nanites:
`artifacts/atmosphere-landscape/haze-full.png` e `artifacts/expanded-world/scenic.png`.
A descrição antiga de12221v possui um typo: n110 gera12321v/24200triângulos.

A família aprovada já tem um análogo Rust em `terrain::Operation::Ridge`, Terrain2:
suporte longitudinal/end_fade, orientação, crista, perfil e dobras. O sampler
planetário ainda não possui esse operador. A variedade aprovada também vinha da
composição; não se tratava de campos aleatórios independentes a cada tile.

A névoa atual é10,83vezes mais densa. A transmitância exp(−density*d) a5km é
3,88%, contra74,08% na referência. Isso compara somente esse fator: composição,
luz, material, câmera e layers também diferem. A aparência lavada não deve ser
tratada apenas como um problema da fórmula de relevo.

## Comparação nativa da configuração atual

Executar, com os arquivos correntes da demo preparados:

```sh
cargo run --locked --release --example planet_appearance_audit
```

O exemplo lê Scene/Config e cria duas simulações isoladas/offline. Congela três
vistas (vale, oeste e leste) em1280×800, no mesmo tick e com a mesma câmera/luz:

1. Fonte completa, ATM.reduce_details=false; frustum/objetos/oclusão/indexação/
   streams compactos e mínimos desligados como referência.
2. Esses mesmos atalhos ligados, sem aproximação ATM.
3. Configuração combinada com ATM ligado.
4. Controle visual separado, geometria/fonte iguais e material neutro, ATMoff.

Não desliga a residência estática/caches nem reivindica um oráculo de todas as
otimizações imagináveis. Não muda a densidade entre referência e resultado, nem
as primárias da sessão humana. Igualdade RGB8 testa esses atalhos; não prova
beleza, cobertura universal, custo/FPS ou validade de outras configurações.

Resultado no build **e46d5879c672aec9**: **RGB8 exato nas três vistas** para
referência×atalhos. ATM também teve máximo0/canais alterados0 nas três vistas.
O material neutro mantém a forma arredondada; retirar o material não cria as
cristas ausentes. Imagens/report em `artifacts/planet-appearance/`, com hash da
configuração e id da fonte. Fonte atual: `earth_mountain_chains_731199`.
O controle neutro ativou streams mínimos em28/28/29instâncias nas três vistas;
o material procedural original conserva0minimal_instances por elegibilidade,
não porque esse otimizador tenha sido esquecido/desligado.

## O que os contadores realmente dizem

Na captura inicial da demo85c02a9:

- Fonte:88tiles/720896triângulos.
- Frustum rejeita60tiles/491520triângulos;28tiles/229376triângulos permanecem.
- ATM barateia o material de16tiles/131072triângulos; outros98304seguem detalhados.
- `geometric_lod_instances=0`, `max_projected_error_px=0`. A queda720896→229376
  é inteiramente descarte fora da câmera, não redução geométrica porLOD.
- IGL sem pacote/enabled=false; collective_lod=null/ICBoff. Oclusão está
  habilitada no renderer, mas sua política dispensa o passe abaixo de1Mtriângulos
  candidatos. Minimal24B não é elegível para o material com pattern ativo.
- Geometria selecionada15753408bytes: não é memória total nem mediçãoFPS.

O handoff confirmou que IGL1/ICB1 continuam experimentais/OFF; nenhum novo ganho
planetário foi entregue silenciosamente pelo outro chat. Os resultados históricos
da floresta não são medições desta demo e não devem ser somados como ganhos.

## Contrato necessário para a correção

**Forma:** compartilhar o kernel Ridge existente e adaptá-lo à esfera, com
coordenada longitudinal em metros e distância transversal em referencial local
transportado, suporte/junções definidos. Variar curso, largura, ápices, encosta,
assimetria e dobras por identidade global/seed, independentes de tile/câmera/ordem.
Versionar a fonte preservando Planet1 e comparar posições/normais nas costuras,
faces e polos. Física usa a fonte completa. Evitar nova coleção de calotas ou
repetição de meshes em quadrados como substituto dessa integração.

**Cobertura:** a configuração estética antiga não cabe no adapter atual. Density
0,00006 exige raio de omissão muito além do FAR30km; layers com céu claro e
montanha escura deixam contribuição residual mesmo a distância infinita.
Portanto, copiar essa luz/névoa e remover o guard seria incorreto. O World atual
usa uma janela uniforme com orçamento2M; a névoa foi escolhida para caber nesse
recorte. Uma paisagem com atmosfera mais clara precisa de representação distante
real e residência com resolução espacial variável/costuras/prontidão, coerentes
com o renderer e seu contrato de projeção. É uma integração estrutural pendente,
não apenas habilitar um checkbox de otimização.

**Aceitação:** primeiro três vistas de forma neutra da família correta; depois
material/luz e atmosfera em controles separados, com validação artística humana.
Amostras de altura e percursos físicos continuam necessários, mas não substituem
essa aceitação. Não expandir o mundo usando a aparência reprovada como referência.

## Checklist estático desta auditoria

Somente exemplo diagnóstico/documentação: nenhum ramo novo no runtime, sampler,
shader, tick ou caminho de frame do app. Duas simulações e buffers de comparação
limitados a três capturas1280×800; leitura/escrita local explícitas, sem sockets,
workers ou foreground. Configuração hashada uma vez; sem profiling ou promessa
FPS. Não recompila/substitui o executável da demo nem edita mundo/botânica.
