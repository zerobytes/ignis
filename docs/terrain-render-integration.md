# Topografia e renderer atual — EAF-179

Continuação: [divisão entre systems e nanites, contratos físicos/gráficos e
reservas de trabalho](terrain-graphics-coordination.md), EAF-180. O experimento
gráfico de LOD distante está registrado em EAF-181.

Atualização posterior182: colisão estática real e demo local caminhável foram
implementadas na main86b6561 (Scene104/replay114/checkpoint3). Ver
[terrain-collision.md](terrain-collision.md) e a liberação no contrato180.
Aceite humano e residência contínua continuam pendentes. As fronteiras de colisão
descritas na auditoria179 abaixo registram o estado anterior à entrega182.

Verificação31/10/2026, main local91f2503. O chat systems foi consultado pelas
entregas finais de164/166; a direção atual do usuário é desenvolver geração e
otimização no mesmo checkout estável. Os commits2b561a2 (terrain),de05154
(landforms),3b01500 (geometria compartilhada),d5ca9fc (oclusão) e91f2503
(canais consumidos) já são ancestrais da main. Nenhum merge pendente foi necessário.
Pesquisa antiga codex/graphics-research continua isolada; esta prova usa o renderer
nativo já integrado, sem importar o experimento de tangentes que falhou.

## O que funciona junto

| Camada | Implementado e verificado | Fronteira atual |
|---|---|---|
| Terreno164 | Campo fixo editável ou composição procedural; elevação, plano/rampa, ruído smooth/ridged, canal e clamp; publicação como mesh nativo | Campo de alturas possui uma altura porXZ; não representa teto/overhang |
| Formas166 | Perfis/seções3D, picos curvos, mesas, penhascos, cânions, arcos e túneis; fontes editáveis comuns a presets e autoria manual | Peças independentes; sem união booleana, erosão ou cavernas arbitrárias |
| Renderer165..178 | Frustum, instancing, índices/streams compactos, shader por canais consumidos, oclusão e representações atmosféricas | Máscaras/sombras/seleção e fallback dependem dos contratos existentes; não há LOD coletivo universal |
| Cobertura164 | Planeja substituição atômica pai→filhos, conserva fallback quando falta recurso/costura | Ainda é consulta; não controla residência real/upload/streaming ou enumera todo o horizonte |
| Gameplay | Controlador terrestre e colisão de primitivas já existem | Terreno/landforms publicados são visuais: ainda não têm colisão/nav integrada |

A publicação porMCP produz os mesmos Asset/Instance consumidos pelo renderer.
Não há condição por espécie ou origem procedural. Gerar uma malha não desliga as
otimizações; materiais/canais e invariantes determinam os caminhos elegíveis.
As duas APIs já conservam fonte separada em arquivos binários próprios e publicam
em uma transação autoral com undo/replay. Não acrescentar um segundo renderer.

## Receita aprovada de cadeias

Fonte original: `scripts/verify_atmosphere_landscape.py::ridge`, extraída sem
mudar resultados para `scripts/mountain_recipe.py`. Os defaults são largura4000m,
profundidade1600m, altura-base850m, desvio lateral da crista0,13, expoente de
encosta1,15 e dobras48m. Em coordenadas normalizadasu/v:

- Crista sinuosa:0,13sin(6u+fase).
- Envelope transversal: max(0,1-|v-crista|)^1,15.
- Perfil longitudinal:0,76+0,18sin(5u+fase)+0,10sin(13u-fase)+0,05sin(31u+fase).
- Dobras menores:48sin(47u+19v+fase)sin(17v-11u).
- Altura: max(0,(850perfil+dobras)envelope).

Triangulação, f32, normais suaves e material originais preservados. Duas malhas
de12221v/24200triângulos são reutilizadas nas cinco cadeias com escala/rotação e
posição distintas. A composição em profundidade e a atmosfera também participam
do resultado visual aprovado; não atribuir tudo à fórmula de altura.

RidgeShape permite variar dimensões, altura, sinuosidade da crista, inclinação
das encostas e intensidade das dobras. O verificador produz configuração
original, arredondada e aguda, sem trocar o renderer. HashSHA256 canônico dos
dois assets completos (geometria+material) coincide com os hashes medidos antes
da extração, fixados no verificador. A cena humana/floresta original não é editada.

Esta receita roda em build time e gera meshes comuns editáveis. Ainda não é um
operadorRust do Sampler de terreno nem um gerador de mundo contínuo. Suas bordas
longitudinais não são certificadas como tileáveis. O próximo operador nativo
deve avaliar cristas em coordenadas globais com suporte/transições definidos,
compartilhar a mesma amostragem entre patches e preservar o controle por escalas.
Copiar a malha repetidamente em quadrados não estabelece continuidade natural.

Formas com paredes verticais, reentrâncias e vãos continuam sendo construídas
pelas ferramentas166; não forçar essas geometrias para dentro do heightfield.
As variações novas são propostas visuais e não receberam aceite artístico humano.

## Prova de integração

Reprodução no checkout com a paisagem aprovada já disponível em
`artifacts/landscape-live/landscape.scene.json`:

```sh
python3 scripts/verify_terrain.py
python3 scripts/verify_landforms.py
python3 scripts/verify_terrain_render_integration.py
cargo run --locked --release --example terrain_render_integration
```

O script confirma ancestralidade na main e igualdade dos assets aprovados,
publica/compila cinco casos (cadeia aprovada, arredondada, aguda, terreno164 e
formas166) e salva resumos pelo dispatcher nativo. Nos dois casos nativos,
cópias fora de vista exercitam compartilhamento/descarte genérico.

O exemploRust compara todos os atalhos sem perda desligados/ligados, mantendo
a mesma aparência e toda a geometria selecionada. São15pares: cinco casos,
três posições/distâncias da câmera, luz variável e resize retrato/paisagem.
Referência e otimizado são capturados duas vezes para exigir repetibilidade.
Todos os PNGs ficaram exatos. Depois habilita separadamente reduce_details para
exercitar pacotes atmosféricos; substituição ativa nas vistas distantes, máximo
e médiaRGB8 iguais a zero nestes casos. Isso não prova erro zero universal.

As inspeções confirmam canais24B ativos nos assets nativos elegíveis (3/9
componentes nas vistas iniciais de terreno/landforms) e substituição atmosférica
em ambos quando distantes. Nenhum nível geométrico mais grosseiro foi escolhido
nesses15casos; não usar esta prova como entrega do novoLOD ou de redução de
triângulos. Não foram medidos FPS nem ganho de performance nesta unidade.

6Rust emterrain/landforms/world_space,6gruposMCP existentes,15pares nativos de
render,Clippy do novo exemplo,fmt/Python/diff aprovados. Build do app permanece
44dae857d2128e92 porque o runtime não foi modificado. Artefatos correntes em
`artifacts/terrain-render-integration/`: manifest.json,render.json, logs ePNGs;
fontes binárias nativas continuam nos exemplos164/166. Sem novo app/instalador.

## Revisão estática e próximos contratos

Trabalho novo só nos scripts de build/verificação e num exemploRust. Não há
novas alocações, hashes, I/O, tipos, formatos, fases de tick/frame, workers, locks,
dependências ou controles exclusivos daUI no runtime. A receita produz posições
f32/índicesu32 dentro dos budgets existentes; valida dimensões/resolução antes
de alocar. Seus parâmetros e hashes ficam no manifesto derivado, sem inventar
um segundo formato de persistência da engine. Verificações nativas existentes
protegem codec, publicação atômica, vãos, undo/replay e fonte editável.

Custos mantidos: geração/upload eventual, fonteCPU residente, materiais e
triângulos ainda visíveis, planner de cobertura sem adapter real. Para um mundo
explorável, ainda conectar colisão de superfície ao mesmo dado do terreno,
residência física/visual, costuras de posições/normais e propagação de edição.
O experimento autorizado deLOD distante continua próximo passo do renderer;
deve usar essas malhas geradas e a floresta como matriz, preservando silhueta,
cobertura coletiva, resposta à luz e orçamento de erro em pixels físicos.
