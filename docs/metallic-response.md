# Resposta metálica direta — EAF-114

Integração do editor: Scene78/replay82 reúne os contratos; gates históricos
permanecem73/74. As provas materiais usam o asset de calibração estável
`examples/cottage-material-baseline.scene.json` (SHA7286e74c…f1e2c), separado da
casa estrutural corrente `examples/elven-cottage.scene.json`. Receitas mantêm
hashes de origem/pixels; `base_recipe_sha256` acompanha somente a atualização do
caminho de entrada, sem relaxar comparação. Números abaixo são da frente original.


Contrato/checklist anterior ao código, base931896f/Scene73/replay77.
Pergunta autoral: o que permitiria melhorar os ornamentos da casa? O bronze
atual só oferece brilho branco dielétrico/f0≤.16. Preciso escolher metal e
rugosidade, com reflexo colorido e sem difuso no limite metálico. A composição
geométrica/Sweep fica na frente native-plants; esta prova só troca material.

Alternativas: aumentarf0 preserva brilho branco; tingir o brilho de todos os
materiais quebra o legado; importar texturas/IBL/grafo amplia recursos e espaço
de cor. Escolha bounded: Reflectance.metallic:f32 default0/omitido, finite0..1.
Roughness0.2..1 e f00..0.16 mantidos. Metallic>0 incompatível com transmission>0,
recusa atômica. Default0, inclusive explícito, mantém aritmética/pixels legados.
Não são complex IOR, espectro medido, clearcoat, texturas, vidro ou renderer PBR.

Referência primária: [Khronos glTF2, Appendix B](https://github.com/KhronosGroup/glTF/blob/main/specification/2.0/Specification.adoc#appendix-b-brdf-implementation).
Misturar BRDF dielétrica/metálica por metalness; Fresnel Schlick e GGX/Smith
correlacionado existentes conforme [Filament](https://google.github.io/filament/Filament.md.html).
Na unidade Lambert legada: s=πDVNoL, w=(1−VoH)^5;
Fd=f0+(1−f0)w; Fm=C+(1−C)w com C=clamp(baseColor*tints,0,1).
Ld=base*(1−Fd)*NoL+Fd*s; Lm=Fm*s; direto=mix(Ld,Lm,metallic).
NoL/NoV/câmera/half-vector inválidos retornam contribuição direta0.
Convenção legada f0=0 desliga a parcela Fresnel dielétrica (Fd=0), mas não
suprime metal. Não mudar convenção legada por implicação física nova.

Reutilizar kernel dielectric para obter NoL/s/w no caminho metálico com f0=1;
primeiros dois componentes e ordem aritmética do caminho legado ficam intactos.
Um GGX/fragmento e mesma consulta de sombra; pesos multiplicativos só no opt-in.
Ambiente constante base*ambient continua como aproximação explícita. Metal sob
sombra/sem luz pode parecer fosco: não há reflexão de ambiente/detalhes de sala,
IBL, HDR/exposição ou antialiasing especular. UNORM pode saturar realces.
Sem conservação global de energia alegada. Cores float autoradas são as lineares
já consumidas pelo shader; não acrescentar conversão sRGB implícita.

high.w livre recebe metallic depois de empacotar pattern;1f32 na receita,
GpuMaterial1232/tabela20944,Vertex64/Globals176 inalterados. Igualdade/cache de
Reflectance já é semântica, incluir o novo scalar. Scene74/replay78 coordenados;
metallic>0 recusado em≤73, inclusive fontes canônicas ocultas antes de receitas.
MCP compartilhado set_material/put_asset/preview/lote/inspect/undo/save/load/replay.

Checklist prévio:1.autoria eventual e fragmento opt-in, zero tick novo;
2.scalarCopy, Arc/canais/recursos preservados;3.finite/bounds/compatibilidade e
layout testados;4.GGX e sombra uma vez, sem texturas/scans novos;5.mesmos tetos;
6.high.w livre, sem GPUbuffer/pipeline/readback novo;7.sem locks/threads/deps;
8.testar troca/slots/growth/removal/undo/load/reset/viewport;
9.Rust/Clippy/MCP com legado exato, controles de cor/difuso/roughness/f0/luz/
sombra/ambiente/rig/replay/prefixos/erros/gates;10.manter ambiente aproximado,
limiteUNORM, risco de aliasing e sem FPS alegado. Mesma casa, baseline e novo
material com luz/câmera/paleta/geometria/instâncias/fontes constantes.

## Resultado, reprodução e dificuldades observadas

`python3 scripts/study_cottage_metallic.py` usa o release corrente por MCP em
processo headless próprio. Receita examples/cottage-metallic.recipe.json fixa
hashes da casa72, da receita material anterior e das capturas e0a95. Publica o
controle anterior, altera só bronze.material, verifica a cena inteira e salva
uma cópia74 em artifacts/cottage-metallic/examples/cottage-metallic.scene.json.
Câmeras/luz/cores/instâncias/geometria/fontes/colunas nativas permanecem iguais.
Saídas before/after-{hero,detail,front,ornament}.png e report.json no mesmo root.

857Rust,Clippy/fmt/build e87MCP/354grupos passaram no build993076c35e8162d4.
GPU confirma[126,81,45] contra o cálculo GGX quase normal e mistura das BRDFs
com máximo1LSB de quantização. Legacy e0a95 hero/detail/front é byte-exato,
inclusive sem reflectance/pattern (high.w neutralizado explicitamente0).
Dados e custos finais em validation.md/performance.md.

A prova mostra realce colorido, mas não dá formas refletidas ao restante da
superfície: ambiente constante ainda pode deixar o bronze plano. A mistura
física não deve ser usada como um controle para esconder esse limite com mais
difuso. Próximos casos: ambiente/IBL, máscaras de roughness/metallic para oxidação,
alcance dinâmico/filtragem de realces. Vidro e composição geométrica são problemas
separados. authoring-gaps.md registra capacidade, estado e caso mínimo, sem
transformar o documento numa declaração de features entregues.
