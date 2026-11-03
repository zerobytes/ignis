# Cobertura vegetal procedural

12 arquétipos autorais, três variações por tipo,57 componentes de cor em fontes
nativas `.ignis-botany` Source/Recipe7. Os arquivos contêm curvas e seções,
não triângulos nem fotos usadas na geometria. `library.json` relaciona as peças
de cada tufo; flores e pendões possuem componentes próprios.

Gerador canônico: `scripts/groundcover_assets.py`. Rodá-lo explicitamente
substitui as fontes geradas nesta pasta. Salvar versões esculpidas/customizadas
em outro destino antes de regenerar. A mesma seed/regra reproduz as fontes no
ambiente Python corrente; mudar seed muda as curvas. Não é contrato bitexato
entre versões futuras do gerador ou ambientes arbitrários.

`scripts/demo_groundcover.py` compila com o executável nativo, prepara a biblioteca
compartilhada e as cenas de autoria/campo/gramas, e verifica integridade e save/load.
`scripts/plant_groundcover.py` acrescenta um trecho plano a uma cópia de uma cena.

Uso, controles, contagens e limites: [docs/groundcover-assets.md](../../../docs/groundcover-assets.md).
