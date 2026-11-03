# Resumo coletivo por vistas — primeiro experimento EAF-181

Este é um experimento **desligado**. A cena aprovada não foi alterada. Não é uma
opção gráfica entregue no MCP/UI; o driver usa somente a API diagnóstica do renderer.
`None` é o padrão e não cria recursos GPU adicionais. Formatos da Scene, replay,
conteúdo, protocolo, checkpoint e save não mudam por esta frente.

ICV1 compila um objeto explícito de peças estáticas com transformação comum e
materiais constantes/opacos. Um texel R32Uint contém normal oct10×2, profundidade7,
canal de material4 e presença1. Os32bits são consumidos no shader. São162 vistas;
cores por instância e luz permanecem inputs de runtime. Uma representação com dois
triângulos consulta o pacote e preserva vazios amostrados e profundidade. Fonte,
colisão e câmera física não mudam; animação, material não suportado, sombra/IBL,
blur e seleção de edição conservam geometria. O recorte de16 canais pertence ao
formato experimental, não limita a autoria ou o renderer nativo.

O seletor usa uma estimativa de deslocamento/projeção e erro de normal, **não um
certificado de cobertura ou imagem**. O teste demonstrou por que essa distinção
é necessária: superfícies que mudam de visibilidade podem alterar a cor apesar
de um deslocamento geométrico pequeno.

## Resultados que impedem adoção

Apple M4 Pro/Metal, build `8527ea70431041ea`, com a frente187 congelada/coordenada.
Objeto20.292tri, duas peças. Compilação180ms; pacote2.654.336B (2,53MiB), leitura
verificada2,11ms. O payload GPU é2.654.208B, mais bases, poses e paleta; a fonteCPU
continua residente e o cache aquecido conserva também a geometria GPU anterior.
Não há redução demonstrada de RAM total. O shader consulta dados binários,
sem JSON ou reconstrução de triângulos a cada pixel.

Em18 poses isoladas512×384, com câmera/luz variáveis e diâmetro de enquadramento
16/4/2px, apenas quatro escolheram resumo. Duas dessas não tinham pixel de fonte
visível: não contam como sucesso de qualidade. Nas duas com contribuição real,
os erros máximos foram9 e96RGB8. Na segunda, a máscara mudou2pixels numa árvore
que ocupava1pixel. A média sobre a imagem grande esconderia essa perda; o driver
registra recorte focado e máscara branca de cobertura, separadamente.

| Vista | Pixels físicos | Anterior → resumo ms | Objetos resumidos | RGB8máx |
|---|---:|---:|---:|---:|
| Solo | 1280×800 | 54,601 → 71,823 | 4.572 | 19 |
| Solo | 1728×1117 | 61,067 → 58,701 | 4.100 | 0 |
| Aérea média | 1280×800 | 171,799 → 45,301 | 20.593 | 42 |
| Aérea média | 1728×1117 | 178,621 → 86,662 | 15.821 | 42 |
| Aérea ampla | 1280×800 | 241,193 → 35,856 | 28.691 | 20 |
| Aérea ampla | 1728×1117 | 246,538 → 47,185 | 28.107 | 21 |

São32 amostras alternadas aquecidas por variante, draw+espera GPU, semPNG/readback
na medição. Não são FPS de gameplay nem timestamps GPU. A regressão do solo e os
erros de silhueta/cor impedem habilitar o método, apesar do ganho aéreo de até6,7×.

Dois testes críticos passaram: codec/derivação de fonte/recusa de corrupção e
instalação inválida atômica/fallback por fonte divergente na GPU. Check e Clippy
alltargets, formatação própria e build conjunto passaram. A prova não declara
um certificado de imagem nem transforma testes com fallback em sucesso do resumo.

A próxima tentativa é compilar uma hierarquia espacial da superfície original e
consultá-la por pixel, evitando depender de uma vista aproximada. Isso busca
preservar a contribuição que o atlas perdeu, antes de voltar a comprimir aparência.
Dados completos locais: `artifacts/collective-atlas/{compiler,isolated,world}.json`,
logs e PNGs. O driver desta etapa está no commit `da08278`, em `examples/collective_lod_cost.rs`; compilar com
`--compile --compiler-only`, carregar o pacote com `--packet` e comparar com `--isolated`.
Não modificar a cena aprovada para promover este resultado.

A continuação com hierarquia de geometria exata está em
[consulta coletiva da geometria](collective-geometry-query.md).
