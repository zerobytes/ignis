# Consulta coletiva da geometria — EAF-181

Experimento ICB1 **desligado**. Substitui o protótipo de vistas ICV1 no driver de
pesquisa, sem habilitá-lo no mundo aprovado. A API é diagnóstica do renderer;
não é uma configuração de produto entregue no MCP/UI. Scene, replay, conteúdo,
protocolo, save, física e controle do personagem permanecem iguais.

## Compilação e consumo

O compilador recebe um objeto composto explícito, sem regras por espécie ou nome.
Neste recorte, as peças precisam ter a mesma transformação, malhas estáticas,
geometria compartilhada e material constante/opaco suportado. Rigs, clips,
attachments, material incompatível, sombra/IBL, seleção e blur conservam a fonte.
Cores das peças e luz são inputs de runtime. Não depende da receita de subdivisão.

O pacote próprio `IGCB1\0\0\0` contém contagens, esfera, nomes dos assets, hash da
fonte, hierarquia espacial, triângulos e selo SHA256. Dados numéricos são little
endian. Nó de32B: mínimoXYZf32, escapeu32, máximoXYZf32 e descritor de folha.
Triângulo de80B: três posiçõesXYZf32, três normaisXYZf32, índice da peça e ordem da
face original. A GPU lê os mesmos buffers binários; não interpreta JSON, strings
ou árvores de objetos. A travessia é sem pilha e cada folha tem até8 faces.

O leitor verifica fonte/material, tamanho, permutação exata de posições/normais,
índices, profundidade da árvore, escapes e contenção de filhos/faces. Um pacote
inválido não substitui o anterior. O orçamento operacional de128MiB limita este
compilador experimental; não é um novo teto de objetos na Scene. O dispositivo
também pode recusar o pacote, conservando o caminho original.

Em runtime, dois triângulos cobrem o volume do objeto. Cada pixel procura a
interseção na hierarquia e usa a normal/material da face original atingida. Isso
reduz rasterização e trabalho sobre faces não atingidas, mas **não elimina** busca,
iluminação, acesso à memória, preenchimento ou composição. O pacote retém toda a
superfície: ainda não é um LOD geométrico simplificado nem um formato mais compacto
que toda malha indexada. FonteCPU e cache GPU nativo aquecido continuam residentes.

A seleção experimental usa densidade projetada em pixels físicos, com fallback
para regiões próximas/interseção do plano de chão e capacidades do dispositivo.
Densidade é previsão de custo, **não certificado de imagem**. Nenhum símbolo é
adotado automaticamente na configuração normal da engine.

## Composição com oclusão existente

A primeira integração reduzia o trabalho nativo e, junto dele, o orçamento de
oclusores. Isso removia a proteção oferecida pelo terreno e causava regressão no
solo. O cálculo do orçamento agora inclui o trabalho de fonte substituído. Uma
passagem de profundidade pode existir mesmo sem instâncias nativas remanescentes;
um buffer mínimo é alocado somente nesse caso experimental.

Antes da busca, o shader pode consultar a profundidade auxiliar do frame corrente
e rejeitar o raio quando a entrada do volume está comprovadamente atrás dela.
Preserva margens de transformação/profundidade e empates do Z convencional. Não
faz readback e usa a textura de oclusão anterior; não cria outro mapa de tela.
O teste A/B deste descarte precisa ser separado da comparação fonte contra busca.

## Limite de qualidade

Manter os triângulos exatos **não garante pixels iguais ao rasterizador nativo**.
Na primeira rodada, as18 poses isoladas foram selecionadas. Nos dois casos não
vazios com diâmetro de enquadramento2px, a imagem coincidiu; isso não certifica os
outros tamanhos. Nos16/4px persistiram diferenças internas e duas poses mudaram
um pixel de cobertura. Repetir o teste em escala menor e usando a mesma geometria
compartilhada reduziu parte das diferenças, sem eliminar todas.

Precisão de profundidade, interpolação e cobertura subpixel são hipóteses a
investigar, não uma explicação comprovada para todo erro. A referência de GPU
continua sendo o alvo; não relaxamos o critério por ter preservado a fonte.
[Precisão de profundidade](https://developer.nvidia.com/blog/visualizing-depth-precision/)
e [precisão de rasterização no WebGPU](https://gpuweb.github.io/gpuweb/#rasterization)
explicam por que equivalência geométrica e equivalência de imagem são contratos
separados. Ainda falta um critério conservador de cobertura/aparência e uma
previsão de custo que impeça regressões antes de habilitar a seleção em produto.

## Rodada final — 01/11/2026

Apple M4 Pro/Metal, build `66b29166526d5004`. Mundo aprovado20×20km,30mil objetos
/60mil peças, mesmas posições/materiais/atmosfera. “Anterior” abaixo já contém
as otimizações nativas existentes e visibilidade coletiva; não é renderer bruto.
O pacote de um objeto20.292faces/2peças levou40,34ms para compilar e2,42ms para
ler/verificar. Possui8.191nós,1.885.604B no arquivo (1,80MiB) e1.885.472B de
nós/triângulos GPU, mais uniformes, poses e cores. Na vista ampla, as capacidades
de poses/paleta chegaram a3.251.696B: só esses recursos coletivos somam cerca de
4,90MiB. Isso é **memória adicional**, não redução de RAM total. O mapa auxiliar
existente continua alocado; quando antes não existia oclusão, seu custo também
entra no conjunto. Os contadores de `occlusion` separam imagens e buffers.

| Vista | Pixels físicos | Anterior → anterior+ICB1 ms | RGB8máx |
|---|---:|---:|---:|
| Solo | 1280×800 | 46,965 → 52,618 | 139 |
| Solo | 1728×1117 | 51,256 → 61,913 | 48 |
| Aérea média | 1280×800 | 155,644 → 46,272 | 25 |
| Aérea média | 1728×1117 | 158,713 → 64,239 | 33 |
| Aérea ampla | 1280×800 | 218,404 → 34,309 | 16 |
| Aérea ampla | 1728×1117 | 221,054 → 49,248 | 20 |

São32amostras alternadas aquecidas/variante, draw+esperaGPU, semPNG/readback no
intervalo medido, sem compilação concorrente. Não são FPS nem timestampsGPU.
As vistas aéreas custaram2,47–6,37×menos tempo, mas **todas falham** no limite de
2RGB8 por canal. No solo houve regressão de12,0–20,8%. A média de erro sobre o
quadro inteiro é baixa, mas esconde diferenças em objetos pequenos; não é aceite.

Separadamente, `--seed-only` compara ICB1 sem/com a consulta ao depth corrente.
As seis vistas estáticas foram RGB8exatas. Oito frames adicionais, movendo câmera,
luz e64objetos compostos e alternando640×400/800×500, também foram RGB8exatos e
verificaram que o descarte estava ativo. O driver falha se não ativar ou mudar
um pixel. Os tempos da primeira rodada seed-only não são usados como resultado
de performance porque houve compilação concorrente; o quadro acima foi repetido
isoladamente. Regressão de oclusão nativa existente passou em11poses, inclusive
vãos, ordem, movimento, near-plane, resize e sombras.

Nas18poses isoladas finais, as16/4px ainda tiveram erro máximo até109RGB8 e duas
poses alteraram um pixel de cobertura. As2px tiveram imagem exata; só duas tinham
contribuição real de1pixel e quatro estavam vazias. Nem mesmo a exatidão de um
caso vazio é contabilizada como preservação de detalhes. Teste adicional com
plano subdividido realmente selecionado passou no limite de2RGB8; não certifica
a floresta ou silhuetas complexas.

Os testes críticos verificam pacote truncado/corrompido/reassinado com referência
inválida, ciclo de escape, volume que perde fonte, fonte/material divergentes,
instalação atômica e fallback após edição. Dois testes próprios + oráculo de
oclusão passaram; build release, check/Clippyalltargets-Dwarnings e formatação
própria passaram. O build foi coordenado com187:7testes character_base e1gizmo
passaram naquele snapshot; alterações posteriores daquela frente são outra
validação. Janela humana preservada, sem promessa de novo FPS jogável.

## Reprodução e continuação

Executável de diagnóstico: `target/release/examples/collective_lod_cost`.
Fonte: `artifacts/expanded-world/world.scene.json` (não foi editada).
Argumentos comuns: `--packet artifacts/collective-lod/object.igcb`.

- `--compile --compiler-only`: gerar pacote e relatório de compilação/verificação.
- `--output artifacts/collective-lod/composed`: A/B anterior contra anterior+ICB1.
- `--seed-only --output artifacts/collective-lod/seed`: próprioICB1 sem/com depth.
- `--seed-only --motion --output artifacts/collective-lod/motion`: oráculo de movimento.
- `--isolated --output artifacts/collective-lod/final-isolated`:18poses e máscaras.
- `--isolated --scale-factor 0.01 --output artifacts/collective-lod/shared-near`:
  diagnóstico de profundidade com o mesmo tamanho projetado, sem mudar a cena fonte.

Relatórios JSON, logs e PNGs estão nessas pastas locais ignoradas pelo Git. O
compilador e os gates ficam versionados. Próxima etapa: explicar e certificar
cobertura/aparência nos objetos pequenos, mantendo o descarte que já passou, e
só depois calibrar seleção por custo. Ganhos aéreos não justificam habilitar um
caminho que perde pixels relevantes ou regride perto. EAF-181 permanece em andamento.


Continuação: [seleção perto/longe e cobertura subpixel](collective-selection.md)
registra o fallback de custo perto e a correção das máscaras, mantendo a cor aberta.
