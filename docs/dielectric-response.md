# Resposta dielétrica à luz direta

## Contrato e revisão anterior ao código — EAF-94

Folhas/casca continuam o ensaio geral de autoria nativa, sem UI ou tipo de
árvore. A frente de continuidade geométrica fica separada e os formatos60/64
foram coordenados com ela. Baseline db3b3de, Scene59/replay63.

Escolha: Material.reflectance opcional com roughness perceptual0.2..1 e
f0 escalar0..0.16. Ausência ou f0=0 desabilita explicitamente o lobo e mantém
os pixels legados, inclusive no rasante. Essa convenção de zero não representa
o Fresnel físico de um meio de índice1. Não há metais, texturas PBR, reflexos
do ambiente, IBL, espessura/SSS, HDR ou exposição novos. Introduzir todos esses
contratos agora ampliaria recursos, samplers, espaços de cor e energia além
da comparação necessária. Material por asset permanece uma limitação explícita.

Modelo isotrópico GGX, visibilidade Smith correlacionada e Fresnel Schlick,
conforme [Filament](https://google.github.io/filament/Filament.md.html).
alpha=roughness²; D=alpha²/[π(NoH²(alpha²−1)+1)²];
V=0.5/[NoL sqrt(NoV²(1−alpha²)+alpha²)+NoV sqrt(NoL²(1−alpha²)+alpha²)].
F=f0+(1−f0)(1−VoH)^5. Com NoL/NoV positivos, subtrair baseColor*F*NoL
do difuso frontal e somar branco*πDVFNoL, ambos sob a mesma luz/sombra direta.
O π converte o BRDF para a unidade difusa legada, que não usa1/π. Ambiente e
transmissão traseira aproximados mantidos; não alegar conservação global de
energia ou renderer PBR completo. Normais já perturbadas por bump são usadas.
Câmera/half-vector degenerados desabilitam o lobo; denominadores protegidos.
Rugosidade mínima limita singularidades, mas não constitui antialiasing
especular. O alvo UNORM atual pode saturar picos; não inventar tonemapping.

Scene60/replay64: validar campo em asset e fonte de planta antes da migração;
set_material/preview/inspect e transação existentes. Reflectance Copy pequena
na igualdade/cache de Material. Reutilizar response.w para f0 e low.w para
roughness, após empacotar pattern. Tabela1136B×17=19312B e Vertex52B mantidos.
Globals160→176B pela posição da câmera, CPU/WGSL concordantes. Sombra/guias
sem lobo novo e nenhuma alocação GPU/pipeline adicional.

1. Frequência: validação/autoria; lobo só no fragmento habilitado. Sem tick NPC.
2. Alocações: dois f32 opcionais; tabela/Arc/Frame existentes, nenhuma por pixel.
3. Tipos: valores finitos limitados, WGSLf32, layouts alinhados. Sem dependências.
4. Repetição: cálculo constante por fragmento; reaproveitar visibilidade já
   calculada e cache de material. Sem scan geométrico/texture fetch extra.
5. Limites: mesmos assets/V/T/desenho/instâncias. Erros antes de publicar.
6. GPU/I/O: uniforme global+16B; Vertex/tabela/recursos de sombra mantidos;
   capturas e persistência explícitas, build/artefatos em caminhos correntes.
7. Sincronização: dispatcher/staging síncronos existentes; sem thread/lock novo.
8. Invalidação: igualdade inclui reflectance; câmera atualiza Globals por frame;
   undo/load/reset/slots/replay devem manter a mesma semântica.
9. Correção: validação/versões/preservação em Rust e GPU MCP com neutralidade,
   rugosidade/F0/câmera/luz/bump/sombra/rig/replay/viewport. Comparar a mesma
   folha real da copa e casca em câmera/luz/tick fixos, sem tocar geometria.
10. Custos mantidos: BRDF frontal aproximado em iluminação legada, validação e
    snapshots limitados, skinning CPU e UNORM. Sem promessa de FPS/realismo.

## Revisão posterior

1. Frequência confirmada: validação no comando; shader somente com f0 positivo
   e frente iluminada/visível. Sem trabalho adicional de simulação/skinning.
2. Nenhuma alocação nova por vértice/fragmento. Option<Reflectance> copia dois
   parâmetros no metadado existente; Detail/Geometry/Rig/Clip continuam Arc.
3. Layouts verificados: Globals176B/offset câmera160B, Vertex52B e tabela19312B.
   Parâmetros finitos limitados; divisores e vetores degenerados protegidos.
4. Visibilidade avaliada uma vez; cinco multiplicações no Schlick, sem pow ou
   textura/busca nova. Cache reconverte receitas só ao mudar o documento.
5. Sem ampliar budgets. Árvore conserva11172v/16250t armazenados,64794t de
   desenho e742t livres. Aplicar metadados conserva geometrias/canais/poses.
6. Apenas+16B no uniforme global, atualizado pelo caminho existente. Sombra e
   guias intactos. Ausência conserva inclusive a ordem da aritmética legada;
   captura neutra anterior ao incremento é idêntica byte a byte no mesmo cenário.
7. Sem locks/threads/filas novas. Staging/preview/undo síncronos existentes.
8. Igualdade inclui reflectance; testes de slots/undo/viewport/load/reset e
   câmera/luz confirmam invalidação. Nenhum material usa câmera autorada antiga.
9. Rust cobre finito/limites/rollback/histórico/plant-source/extração/subdivisão/
   retesselação/formato/replay. MCP testa shader real: neutro/rugosidade/F0,
   ângulos/bump/sombra/rig compensado, persistência/replay/prefixo/viewport.
   Capturas da folha real da copa e casca têm controles fixos e foram inspecionadas.
10. Mantidos: reflexo isotrópico uniforme por asset, unidades legadas e ausência
    de IBL/HDR/tonemapping. Controle roughness0.3 expõe realce de aliasing das
    nervuras/bump e aspecto plástico; não é receita final. Folha final0.58 e
    casca0.92/F0=0.035, sem alterar desenho da superfície para esconder o limite.
    Antialiasing especular e rugosidade por domínio/máscara são próximos estudos,
    não recursos entregues. Evidências quantitativas em validation.md.

## Fronteira de integração de formatos

60/64 são números locais de codex/native-meshes, base db3b3de, coordenados com
native-plants. O checkpoint Kordo revelou que a frente funcional usou60/64 em
EAF-92,61/65 em EAF-93 e reservou62/66 em EAF-95. Numeração igual não implica
contratos interoperáveis entre essas branches. Integrar schemas/guardas/testes
de ambas as frentes e escolher versão unificada antes de publicar integração;
esta unidade não faz merge no main nem declara compatibilidade com esses novos
campos funcionais. Replay também exige o build exato. Nenhum payload foi
silenciosamente carregado/reescrito entre as frentes nesta validação.
