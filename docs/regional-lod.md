# Compilador de LOD por regiões — EAF-181

Continuidade de [automatic-lod.md](automatic-lod.md). A fonte editável e a geometria de colisão continuam intactas. O sidecar IGL1, controles MCP/UI e fallback original são os mesmos; este incremento altera geração de candidatos e estimativas de correspondência, sem alterar formatos da engine. Scene106/replay116 do build conjunto pertencem ao painel de personagens187.

## Mudanças

O compilador divide cada componente em regiões de até 128 faces, mantendo bordas durante a simplificação. O candidato por componente inteiro também continua disponível; entre os candidatos válidos de cada escala, escolhemos o de menor contagem de triângulos. Isso preserva o caminho anterior quando as bordas das regiões custam mais. Essa partição existe somente no build: não cria instâncias ou draw calls por região. Regiões sem candidato válido mantêm TODOS os cantos/normais originais; a tentativa de soldar posições para gerar um candidato nunca altera o fallback. Componentes pequenos também são considerados, sem apagar ilhas para fabricar redução de custo.

Para cada razão de redução, o gerador tenta pesos de normal 32/8/1 antes de desistir. Os limites de normal 0,02/0,06/0,2, retenção mínima de 10% de triângulos e tolerâncias de runtime continuam iguais. O orçamento offline foi ampliado para 8 milhões de nós por tentativa de particionamento; o leitor conserva seu orçamento de 8 milhões por nível. Assets elegíveis e fallback mantêm seus limites anteriores.

A prova escolhe, entre um triângulo próximo e até 32 vizinhos topológicos, uma correspondência afim para UM triângulo convexo. Qualquer um fornece um limite válido; escolher uma correspondência melhor pode apertar esse limite. Para normais, um único semiespaço positivo contém os seis vetores dos dois campos. Isso limita o módulo de cada vetor no segmento entre eles por `m>0`. Integrar o Jacobiano da normalização (`||J||≤1/m`) fornece `erro_normal≤erro_bruto/m`, limitado a2; se não houver semiespaço positivo, o resultado continua2. O leitor recompõe a prova completa por componente antes de publicar.

No seletor, a distância de dois pontos correspondentes difere por no máximo o erro geométrico `δ`. O erro de transmissão atmosférica usa `exp(-ρ d_min)×(1-exp(-ρδ))`; o blend de detalhe usa o limite de derivada 1,5 do smoothstep, multiplicado porδ e dividido pelo intervalo configurado. Não se cobra mais a profundidade inteira do asset como deslocamento do LOD. A estimativa de projeção e o orçamento de erro em pixels físicos permanecem iguais.

Isso é prova de correspondência de superfície/normal, não certificado de imagem, oclusão ou estabilidade temporal. A seleção continua experimental e desligada por padrão até que as imagens e o custo medido justifiquem uso.

## Validação e medição

As evidências correntes são geradas em `artifacts/regional-lod` pelo exemplo `automatic_lod_cost --world-player-view --regional-lod`. Compilação do binário, prova do codec/certificado, pixels renderizados e FPS nativo são gates separados. Não atribuir ganho de FPS a candidatos que não tenham sido selecionados.

## Resultado desta rodada

A floresta continua sem pacote IGL1 útil: nenhuma escala aprovada atingiu a redução mínima preservando os limites. O fallback mantém a fonte completa; não houve ganho de FPS atribuível ao novo LOD. O ensaio regional isolado aprovou cerca de 1% de redução no tronco e nenhuma nas folhas, insuficiente para publicação. Não aumentamos o erro permitido para fabricar um resultado.

Comparar os candidatos por componente inteiro e por região também protegeu o caso simples: o plano voltou a 2.048→512 triângulos, com PNG 1280×800 idêntico, pacote 25.495 B, GPU selecionada 10.776 B e cache aquecido 49.200 B incluindo a referência. Uma versão intermediária que só usava regiões mantinha 1.024 triângulos e foi corrigida antes da entrega.

Dois testes Rust críticos passaram: o certificado contém os campos de normal interpolados e falha conservadoramente quando há cancelamento; codec/publicação recusam corrupção, fonte divergente, ilhas perdidas e limites subestimados. O MCP real também confirmou load inválido atômico, fonte preservada e consumo do plano compilado. Clippy release de todos os alvos, formatação dos arquivos próprios e build conjunto aprovados no build `06d221e5584e0984`.

A próxima representação para essas pequenas superfícies precisa guardar cobertura e resposta à luz dos detalhes removidos em build time. O ganho de performance desta rodada está documentado separadamente em [fine-occlusion.md](fine-occlusion.md).
