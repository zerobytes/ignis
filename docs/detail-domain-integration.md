# Integração dos domínios de detalhes —71

Revisão anterior à importação31944f7 sobre a29d654 em native-plants.
Contrato e alternativas lidos em detail-domains.md da unidade candidata.

Escolha: integrar profile/repeat como ferramentas genéricas opcionais, manter
a receita de casca em cópia comparável. A observação direta dos closes e da
árvore inteira mostra fissuras mais curtas, mas padrão repetido/compressão nas
forquilhas; não promover aparência só por contagem de testes ou aprovação estética.
Fontes/rig/instâncias e a árvore canônica69 permanecem como referência.

Checklist prévio proporcional ao patch:
1.Só edição e fragmentos detalhados; sem tick/física novos.
2.Detail continuaArc; um vec4 por slot, sem dados por vértice/célula.
3.Profile fechado, períodos e cápsulas validados antes de publicar.
4.Um fract por plano e o mesmo loop32; sem varrer vizinhos nem nova alocação no shader.
5.Tetos existentes; margem2raios e fadeAA não equivalem a filtragem analítica.
6.Tabela19312→19584B (+272B), Vertex64B/Globals176B mantidos, sem textura/readback novo.
7.Mesmo dispatcher/cache, sem threads/locks/dependências.
8.Igualdade de material invalida tabela; verificar toggle/undo/load/reset/resize.
9.Rust/Clippy/MCP próprios, versão71/replay75 e fontes ocultas; pixels do legado
comparados aos mesmos19PNGs anteriores; mesma árvore em4vistas com neutro exato.
10.Mantidos C1 somente radial, max entre traços e taper de raio podem ter quebra
de derivada; periódico f32 tem erro de fase, sobretudo para coordenada/período
muito alto. Não prometer periodicidade bit-exata, ausência global de alias ou FPS.

Campo band/None deve permanecer sem alteração perceptível ou de pixels nas
fixtures antigas. A tolerância≤1LSB na translação periódica pertence somente ao
ensaio numérico explícito, não substitui repetição/replay byte-exatos nem gate
visual. Preservar o relato da receita rejeitada para não reintroduzir cortes.

## Resultado próprio da integração

Build05454b395586158e em native-plants:847testes/84grupos Rust release/all-targets,
Clippy-Dwarnings/fmt/build e84cenários/339grupos MCP headless completos passaram.
Mesmo hash de código da unidade31944f7, execução própria neste checkout.
Auditoria detail-domain-integration-audit.json confere66summaries comID e18legados
semcampo, estudos de frames/projeção/casca e hash de todos src+Cargo.lock+build.rs.

37PNGs do caminho legado (planar e triplanar, neutro/autorado/replay) permaneceram
byte-idênticos ao build70 deste ramo; detail-domain-planar-audit.json. A translação
periódica do ensaio800² mudou9pixels, máximo1nível8-bit e média.000003515625por
canal, conforme tolerância explícita; detail-domain-period-audit.json. Repetição,
undo/load/replay continuam exatos e não usam essa tolerância.

Rerodados study_tree_bark.py, demo_tree_material_frames.py e study_tree_projection.py
com o binário integrado. A mesma árvore em whole/base/neck/fork mantém posições,
triângulos, fontes geométricas, rig/clipes/instâncias e4neutros exatos. Os4PNGs
autorados novos coincidem também com o estudo da unidade de origem. Fonte
canônica69 continua SHA89fe1fa2112cbb8a2d3ae266a53cf0d6308c8b371d356a8f5b1b05a208789400.
Aparência nova está em artifacts/tree-realism/examples/after.scene.json71,
reproduzível pelo script, sem substituir a referência automaticamente.

Inspeção própria: fissuras menores deixam o close menos parecido com fitas, mas
a base ainda mostra padrão angular/repetido e as forquilhas comprimem/viram
a direção dos traços. Copa e junções restantes continuam procedurais. Não é
realismo concluído; separar forma, iluminação, direção do domínio e escala dos
traços na próxima decisão, sem recorrer a mais ruído indiscriminado.

Revisão posterior, dez respostas:1.só shader/autoria;2.DetailArc e nenhum buffer
por célula;3.materials::Detail::validate limita períodos/cápsulas;4.surface_detail
mantém32testes/plano;5.margemAA e tetos preservados;6.tabela+272B/Vertex64B igual;
7.dispatcher sem nova sincronização;8.material equality/cache invalida por
conteúdo;9.gates e regressão própria acima;10.C1 radial, fasef32, taper/max e
filtragem aproximada conscientemente mantidos, sem FPS alegado. Nenhum ajuste
extra de Rust foi necessário após a revisão da unidade candidata.
