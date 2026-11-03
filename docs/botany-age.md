# Idade cronológica e desenvolvimento de plantas

Continuação corrente: [arquitetura de copa e bifurcações terminais EAF-194](botany-architecture.md). As evidências abaixo registram EAF-193; as fontes lenhosas correntes usam fonte/receita4.

Continuação de EAF-192 a pedido de Guilherme: os exemplares aparentavam juventude;
criar versões de décadas e um controle de idade que coordene o desenvolvimento.

## Revisão prévia

O `dynamics.Settings.age` anterior é uma fase numérica de expansão dos órgãos
já existentes: após `birth + duration`, a fração suave chega a1. Não representa
anos nem gera uma arquitetura adulta ao receber50. Continua separado do relógio
cronológico, para preservar os ensaios e fontes anteriores.

Novo contrato: fonte3/receita3, leitura1/2 preservada, perfil de desenvolvimento
opcional e `settings.age_years` opcional. Ausência de perfil mantém o caminho
anterior. O perfil é arte/configuração explícita, não uma tabela empírica que
permita datar uma árvore real. O mesmo número de anos tem efeitos diferentes
em perfis diferentes e sob vigor diferente.

Altura, abertura da copa e engrossamento têm curvas próprias saturantes
`g(t,τ)=1-exp(-t/τ)`, normalizadas pela idade de referência da geometria. Os anos
fisiológicos são `t=idade_anos × vigor`. A altura não precisa acompanhar a taxa
de crescimento radial. O engrossamento cai com a ordem do ramo; ramúsculos e
folhas atuais não recebem automaticamente o diâmetro de tecidos antigos.
Palmeiras usam estabelecimento rápido de diâmetro e alongamento posterior,
sem aplicar a lei lenhosa de décadas ao caule inteiro.

Órgãos/sítios têm nascimento e expansão em anos, opcionais e determinísticos.
A fonte reserva candidatos com IDs estáveis; a idade seleciona os presentes,
com filhos dependentes de pais presentes. As folhas de uma árvore velha
representam folhagem corrente de tamanho limitado, não folhas com décadas.
Senescência começa num marco configurável e pode reduzir folhagem; décadas
não significam automaticamente planta morrendo.

A curva de repouso e a seção efetivas alimentam bake e ensaio físico. Alterar
idade recompila uma candidata autoral e invalida a sessão física anterior;
não transporta momento entre décadas nem simula retrospectivamente cada ano.
Deltas manuais continuam por endereço material; se um órgão ajustado deixaria
de existir, a publicação preservada recusa a perda, como a poda anterior.

Checklist de performance: calcular o estado do perfil uma vez por compilação;
marcos/IDs independentes da ordem de consulta; filtrar candidatos antes da
malha; não reexecutar anos anteriores ou criar trabalho por frame estável;
quotas antes de alocar; nenhum fallback de geometria inválida. Comparativos
nativos em caminhos estáveis, um build corrente, sem tocar a janela humana.
Novos testes apenas para integridade/identidade/formatos/atomicidade e
integração física material, sem teste individual de cada curva visual.

## Base conceitual

Modelos de crescimento tratam dimensões e copa como variáveis diferentes:
[US Forest Service, GR02](https://research.fs.usda.gov/treesearch/21726).
A anatomia de palmeiras exige distinguir crescimento em altura e estabelecimento
do diâmetro: [UF/IFAS, Palm Morphology and Anatomy](https://ask.ifas.ufl.edu/publication/EP473/pdf).
As curvas desta implementação são nossas aproximações configuráveis; não são
coeficientes dessas publicações nem calibração das quatro espécies.

## Implementação e evidências

Implementação nativa concluída. Controle de idade e vigor no painel Vegetação,
`Botany.set_age` compacto e fonte3 editável. Os onze testes Rust passaram,
incluindo o novo contrato crítico de cronologia/preview/formato/massa/repouso e
perda manual recusada; o teste existente dos quatro exemplares cobre fechamento,
orientação e reprodução determinística das superfícies. Clippy lib+botany com
`-D warnings`, fmt dos arquivos tocados, Python e diff aprovados.

Build macOS arm64 corrente `b152a6f5fd1dcf88`. Capturas nativas
8/20/adulta realizadas com a mesma câmera/seed/luz/escala por planta e inspecionadas.
Galéria adulta salva/reaberta com PNG bit-exato. Todos os quatro exemplares
iniciaram o ensaio de repouso completo, sem reduzir a planta a um galho.

| Exemplar | Anos | Altura do bake | Órgãos | Vértices | Triângulos | Pontos físicos |
|---|---:|---:|---:|---:|---:|---:|
| palmeirinha | 30 | 4.53m | 3540 | 413304 | 812448 | 75504 |
| platano | 50 | 20.04m | 7416 | 2841886 | 5654968 | 160416 |
| eucalipto | 50 | 21.03m | 8136 | 965662 | 1899640 | 174816 |
| figueira | 50 | 13.27m | 13874 | 1680432 | 3307608 | 308952 |

As alturas são resultados das receitas autorais, não medições de espécimes reais
com essas idades. O comparativo em `artifacts/botany-age/review.html` expõe as
medidas e as estruturas sem folhas. A copa da figueira e o diâmetro/porte dos
troncos diferenciam o adulto do jovem; material e folhagem seguem o perfil.
Não se atribui aceite artístico humano ou réplica fotográfica a esses números.

MCP final no mesmo build: preview/cancelamento e `set_age` compacto,
fonte3/replay/save portátil, repouso das quatro plantas completas, crescimento
mecânico/poda da figueira adulta inteira e grandes dobramentos/chão/contato.
O roteiro de crescimento mecânico (não anos históricos) percorreu128passos e
32incrementos, conservando identidade/balanços de massa e momento, com erro
final de vínculo8.75e-09m em
308952pontos. Foram109.64s neste Mac;
isso evidencia custo autoral, sem promessa de tempo real ou FPS.

Evidências correntes: `artifacts/botany-age/report.json`, `verification.json`,
`tests.log`, `chronology-tests.log`, `build.log`, `clippy.log`, `fmt.log` e
`review.html`. Os PNGs nativos incluem8/20/adulta e estrutura sem folhas; a
cena adulta é própria, sem tocar mundo, personagem ou janela humanos.
Não houve push ou alteração nos layouts de Scene109/replay119/PSV1/conteúdo.



A revisão usa Source/Recipe3 por nome de campo, com `development` na fonte,
`life.born_years/expansion_years/role` nos órgãos e `age_years/vigor` na receita.
As folhas são sítios potenciais de folhagem corrente; datas não alegam que uma
folha individual permaneça viva desde aquele ano. A emergência de uma coorte
lenhosa tem fração visível mínima configurável, pois não é um solver de gemas
microscópicas. A ordem de avaliação não define identidade.

O perfil permite engrossamento diferente por ordem de ramo, flare basal,
acabamento de casca e rigidez efetiva. A folhagem cresce até tamanho limitado;
folhagem senescente é filtrada por hash estável. Intervenções/podas/competição,
anastomose de raízes, anéis anuais e tecidos/casca em camadas não são inferidos.

## Como parametrizar

O controle central é `settings.age_years`. `vigor` dilata o tempo fisiológico;
as taxas pertencem a `source.development`, e não ao renderer ou ao nome da
espécie. Exemplo de receita adulta (demais dados da fonte omitidos):

```json
{
  "version": 3,
  "settings": {"age_years": 50, "vigor": 1},
  "source": {
    "version": 3,
    "development": {
      "reference_age_years": 8,
      "height_time_years": 25,
      "crown_time_years": 14,
      "radial_time_years": 14,
      "branch_thickening_decay": 0.8,
      "branch_length_decay": 0.85,
      "basal_flare": 0.45,
      "senescence_start_years": 140,
      "senescence_time_years": 45
    }
  }
}
```

`τ` é uma constante de tempo: na curva não normalizada, `t=τ` atinge cerca
de63%do limite. Não é o aniversário exato em que a espécie se torna adulta.
O perfil e os marcos dos sítios são configuráveis; as quatro receitas são
arquétipos artísticos, sem coeficientes de campo atribuídos às espécies.

Para alterar a idade pela API, carregue a receita com outro `age_years`, ou
publique sua cópia modificada. Para alterações compactas use `Botany.set_age`,
sem reenviar a fonte. Prévia e confirmação usam o mesmo Service da UI.
O painel mostra **Idade da planta (anos)** e **Vigor de desenvolvimento** quando
há perfil na fonte; fontes anteriores continuam válidas e informam a ausência
de perfil cronológico. A prévia da idade só é enviada ao terminar o arrasto.
A **Fase de expansão do ensaio** continua sendo o controle mecânico legado.

Reprodução dos exemplares: `python3 scripts/botany_exemplars.py`, ou
`--only platano --age 50` para um arquivo. A avaliação da idade é Rust nativo;
Python somente prepara fontes e dirige as capturas. O comparativo
`demo_botany_age.py` reutiliza câmera, seed e escala nas idades8,20e adulta,
com cenas próprias sem substituir o mundo humano.

## Equações do perfil

Para `t=age_years×vigor`, usamos `q_x(t)=g(t,τ_x)/g(t_ref,τ_x)`.
Os multiplicadores H/C/R são os de altura principal, abertura e raio. A altura
final medida inclui a hierarquia orientada e vem dos bounds do bake, não de uma
estimativa isolada H. Coordenadas dos filhos seguem os frames efetivos dos pais.

Em madeira, o raio do eixo de ordem `d` usa `q_R(t)^(δ^d)` após a referência;
na fase anterior usa o fator radial comum, evitando filhos desproporcionalmente
grossos em pais jovens. Para palmeiras, o fator radial é o smoothstep do período
de estabelecimento, normalizado na referência, e chega a um patamar fixo.
O flare basal multiplica a seção perto da base por `1+f·m(t)·(1-u)^6`.

Um sítio presente desde `b` expande por
`e(t)=ε+(1-ε) smoothstep((t-b)/Δt)`; antes de `b` é ausente. Os filhos só existem
se o pai também existe. ε é uma convenção visível de coortes, não uma alegação
sobre o tamanho de uma gema real. A árvore potencial é finita e explícita.

O tamanho foliar global é limitado por
`l(t)=[0,55+0,45g(t,τ_f)]/[0,55+0,45g(t_ref,τ_f)]`.
Senescência usa `s(t)=g(max(t-t_s,0),τ_s)` e sobrevivência foliar
`1-loss·s(t)`, amostrada pelo mesmo hash de identidade. Datas e folhas atuais
são representações diferentes: a fonte não acompanha a vida individual de cada
folha ao longo das estações.

A massa é novamente derivada das dimensões efetivas e da densidade; o material
efetivo também recebe um ganho limitado de rigidez por maturação. A quadratura
e o solver continuam homogenizados, conforme o contrato de dinâmica anterior.
Não se mantém uma massa juvenil com um tronco visual adulto.


O alongamento também decai por ordem: os multiplicadores H/C recebem a potência
`λ^d` antes de formar o eixo. Isso evita que ramúsculos correntes se alonguem
como o tronco antigo. λ e δ são independentes. A expansão do sítio também afeta
comprimento/largura foliares e frondes; raízes aéreas se aproximam do nível do
solo durante sua expansão, chegando a ele ao concluir o estágio.

API compacta, sem reenviar milhares de órgãos:

```json
{"op":"botany","operation":{"kind":"set_age","prefix":"platano","age_years":50,"vigor":1,"preview":true}}
```

`preview:false` publica na mesma transação com as políticas de preservação
anteriores. A fonte precisa ter perfil; a API recusa atribuir anos a uma fonte
legada sem esse contrato. Perfil/cronologia novos exigem fonte/receita3;
valores legados continuam por seus defaults, sem uma conversão silenciosa.


Orçamentos locais deste modelador, ampliados explicitamente para as copas
adultas:16384órgãos potenciais,32MiBpor arquivo de fonte,3Mvertices/6Mtriângulos
por bake,524288pontos/2Mrestrições no ensaio físico. Contato mantém100kpares e
1Mentradas de grade. Excesso recusa a candidata; não poda silenciosamente para
caber. Não são limites universais do renderer/mundo nem promessa de FPS.
Todos os pontos de um exemplar aceito permanecem no modelo físico, sem reduzir
uma copa adulta a um galho representativo. O custo pode ser alto: o ensaio é
autoral, e a deformação de gameplay Rapier segue fora deste contrato.
