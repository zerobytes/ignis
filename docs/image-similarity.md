# Comparação visual de referência e render

Primeiro recorte da autoria por imagem. Alternativas: erro RGB é exato e fácil
de localizar, mas reage também à luz/cor; SSIM mede estatísticas locais, sem
compreender objetos; métricas aprendidas como LPIPS exigem modelos/pesos e
contrato de execução próprios. Escolha inicial: RGB + SSIM de luminância,
regiões explícitas e mapa de erro, sem dependência ou modelo novo. Não compor
um percentual de fidelidade 3D a partir dessas medidas.

## Contrato antes do código

`compare_images` recebe reference_path, candidate_path, mask_path opcional,
regions opcional e difference_path opcional, pelo Command/Service/MCP comum.
Consulta limitada a arquivos no projeto, sem modificar mundo/histórico/relógio.
`capture` existente gera o candidato com câmera/viewport/tick/revisão explícitos.
O comparador não afirma que imagens arbitrárias vieram dessa câmera.

PNGs estáticos, até32MiB codificados e2048×2048pixels, entradas com mesmas
dimensões. Expandir paleta/cinza paraRGB8, rejeitar16bits/animação/alfa não
opaco. Política: comparar valores RGB8 codificados armazenados, sem transformar
ICC/gamma nem ajustar exposição, câmera, crop, escala ou alinhamento. Logo,
correspondência de espaço de cor/iluminação continua responsabilidade declarada
do ensaio, e diferença de cor não prova diferença de geometria.

Máscara opcional é binária0/255 em cinza opaco e seleciona pixels nas duas
imagens. Não é segmentação automática nem comparação de duas silhuetas.
Regiões nomeadas (até16) são retângulos explícitos dentro da imagem; não
redimensionar nem escolher a melhor região silenciosamente. Cobertura informada
por métrica; seleção global vazia ou região sem pixels selecionados recusa.

Métricas por seleção global e região: MAE e RMSE RGB normalizados por255,
maior diferença de canal em bytes, pixels alterados e igualdade exata. SSIM
usa Y'=0.2126R'+0.7152G'+0.0722B', valores0..1, janela uniforme11×11,
variância populacional e constantes0.01²/0.03². Somente janelas inteiramente
dentro da região e da máscara; informar quantidade, retornar null quando não
houver suporte. SSIM[-1,1] não é porcentagem de verdade geométrica. Não misturar
com MAE num score composto. Relatório versionado1 inclui definição/política.

Mapa opcional: por pixel, média absoluta RGB em bytes, cinza0igual/255diferença
máxima; escala fixa, mascarados pretos. Até8tiles32×32 com maior MAE local,
empate porY/X; indicam onde olhar, sem diagnosticar causa ou semântica. Saída
nunca substitui uma entrada/máscara. Falha não publica arquivo parcial.

## Revisão estática anterior à implementação

1. Só comando explícito, sem frame/tick novo.
2. Dois RGB8 limitados, máscara opcional e PNG diagnóstico só quando pedido;
   SSIM com somas deslizantes/ring de11linhas e scratch O(largura), não cinco
   imagens integrais nem busca11×11 por pixel.
3. Totais RGB inteirosu64, SSIM f64; índices/retângulos validados, ordem fixa.
4. Reusar estatísticas de janela para regiões, sem convolução por região.
5. Até4.194.304pixels,16regiões e4096tiles; sem histórico de comparações.
6. Ler arquivos uma vez por consulta; nenhum readback novo além de capture
   solicitado. Usar png existente; não baixar modelos nem instalar runtime.
7. Serviço síncrono existente, sem thread/lock/dependência nova. Custo de comando
   continua sob o lock existente, fora do tick; worker assíncrono é outro recorte.
8. Sem cache persistente; relatório inclui entradas/política e cobertura.
9. Provar identidade, cor/deslocamento/forma, fundo dominante, máscara/ROI,
   referência independente de SSIM e limites/erros/paths/estado/MCP/replay.
10. Mantidos: anotação/ângulo e causalidade dependem do ensaio. Não inferir
    interior/verso, segmentar casa ou declarar percepção/solver por essas métricas.

Referências: [SSIM e parâmetros](https://scikit-image.org/docs/stable/api/skimage.metrics.html#skimage.metrics.structural_similarity),
[métrica aprendida LPIPS](https://richzhang.github.io/PerceptualSimilarity/).

## Uso e demonstração

Antes de reconstruir, aplicar a
[skill de análise](../.agents/skills/ignis-image-to-3d/SKILL.md) e consultar a
[leitura da casa](references/image-to-3d/elven-house-analysis.md). A demonstração
abaixo testa o avaliador com uma geometria **estruturalmente insuficiente**:
não é candidata fiel, mesmo deixando materiais e texturas de lado. Câmera manual
também não foi calibrada. Testes do comparador aprovam suas métricas/contrato,
não a forma dessa casa. O próximo incremento deve partir da análise, não detalhar
o estudo de volumes como se suas relações já estivessem corretas.

```json
{"command":{"op":"compare_images","reference_path":"artifacts/reference.png","candidate_path":"artifacts/render.png","regions":[{"id":"door","x":715,"y":415,"width":225,"height":410}],"difference_path":"artifacts/difference.png"}}
```

Capturar antes com `capture` no mesmo viewport da referência. O resultado
retorna `report` versionado, paths e build_id; não incorpora imagens pesadas
no JSON, nem copia essas entradas para a cena/replay. A imagem de diferença
opcional é um arquivo PNG, não uma publicação da cena. A seleção/guia visual
de sessão também aparece em capturas existentes: manter os mesmos toggles no
ensaio; o comparador não os remove silenciosamente.

`python3 scripts/run_verifications.py --only image_similarity --binary <executável>`
exercita captura nativa/replay, mudança de câmera, defeito local com fundo
dominante, máscara e erros atômicos. O núcleo também é comparado contra uma
implementação de referência direta, independente das somas deslizantes.

`python3 scripts/demo_house_similarity.py --binary <executável>` gera um
primeiro estudo de volumes usando apenas a autoria nativa, guarda a cena em
`examples/house-blockout.scene.json` e produz
`artifacts/house-similarity/index.html`, `report.json` e capturas. A referência
é a imagem original fornecida. Regiões e câmera são aproximadas manualmente;
metadados e hipóteses constam no relatório. Traseira/interior, ornamentação,
calibração automática, segmentação e fechamento físico não foram inferidos.

As duas câmeras da demonstração podem discordar no ranking entre MAE e SSIM:
isso não autoriza escolher uma câmera como correta. Próxima camada de ajuste
precisa de correspondências/contornos explícitos e critérios por etapa, com
regiões de avaliação estáveis. Um SSIM0.4 não significa40% da casa correta.
