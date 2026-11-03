# Materiais e transparência compartilhados — EAF-203

## Direção e contrato antes de código

Em 03/11/2026 Guilherme autorizou construir o alicerce do sistema VFX estudado
em [EAF-201](particle-vfx-requirements.md), incluindo vidro e materiais para
outros sistemas. A fundação pertence ao domínio geral de materiais/renderer:
meshes, arquitetura, personagens e futuros emissores usam as mesmas regras.

Primeiro incremento: superfície opt-in com modo opaco, recorte, translucidez e
aditivo; opacidade/cobertura procedural independentes da cor, emissivo e vidro
de lâmina fina com IOR/Fresnel e absorção Beer–Lambert. `transmission` legado
continua sendo transmissão difusa de folhas e não muda de significado.

Opacos e recortes escrevem profundidade. Translúcidos/aditivos testam a
profundidade opaca, sem escrevê-la; não servem como oclusores nem recebem
representações atmosféricas/LOD que não certifiquem sua nova composição.
Sombras de recorte respeitam a mesma cobertura; translúcidos não lançam sombra
opaca nesta primeira implementação. Seleção continua sendo ferramenta explícita
do editor, com seu próprio custo e política de cobertura.

Alternativas: ordenar objetos é simples, mas falha em superfícies que se
intersectam e pode estourar custo CPU em efeitos densos. Ordenar triângulos ou
depth peeling custa trabalho/memória proporcional à geometria/camadas. A opção
deste incremento é composição ponderada independente da ordem (WBOIT), com
acumulação de radiância e produto de transmitância RGB; aproximação de mistura
em sobreposições, não solução exata de transporte óptico. Aditivo tem acumulação
própria; a posição relativa entre emissão e várias lâminas permanece aproximada.

Vidro fino transmite o fundo, absorve por cor/distância/espessura e reflete
conforme o ângulo e IOR. Usa a iluminação direcional e o probe IBL existentes;
sem probe não inventa reflexos do ambiente. Cada superfície autorada representa
uma lâmina: malha fechada com duas paredes aplica duas lâminas. Refração espacial,
desfoque da transmissão, caústicas, múltiplos rebotes, sombras coloridas e volumes
ópticos ficam para incrementos próprios. A fonte atual não é rebatizada como
vidro sólido fisicamente completo. HDR/exposição/bloom gerais também continuam
contratos separados; intermediários float não mudam por si só a saída SDR.

Fontes primárias: [composição WebGPU](https://www.w3.org/TR/webgpu/#blending),
[WBOIT, McGuire/Bavoil](https://jcgt.org/published/0002/02/09/paper.pdf),
[modelo de materiais Filament](https://google.github.io/filament/Materials.html)
e [transmissão/volume Khronos](https://github.khronos.org/glTF-Tutorials/AddingMaterialExtensions/AddingMaterialExtensions_003_TransmissionAndVolume.html).

## Revisão estática de performance prévia

1. Dados por material em storage buffer; sem receita duplicada por vértice,
   sem alteração de geometria física e sem listas de partículas antecipadas.
2. Recursos/transparência alocados sob demanda e reutilizados por resolução;
   cenas opacas mantêm o caminho anterior, sem targets/passes de OIT ativos.
3. Instancing e caches existentes preservados; classificação por material,
   nunca por nome/espécie. Não expandir instâncias estáticas na CPU para ordenar.
4. Render targets são custo explícito por pixel; sem teto autoral novo para
   objetos, mas respeitando limites do device e memória de cada superfície.
5. Nenhum readback por frame, nova thread/lock ou I/O no desenho. Capturas e
   diagnóstico são operações explícitas; não declarar ganho de FPS.
6. Layout mínimo só quando todos os canais procedurais estão ausentes;
   alterações de material reavaliam elegibilidade e invalidam caches relevantes.
7. Quedas de precisão/overdraw e mistura aproximada exigem cenas de controle;
   limites numéricos evitam NaN/Inf/overflow nos acumuladores.
8. Formato autoral, replay e conteúdo público devem versionar a superfície;
   fontes ocultas de regeneração também são verificadas. Defaults omitem o novo
   domínio, preservando serialização/pixels e fingerprints de materiais antigos.

## Validação planejada

Provas críticas: contrato/finito/rollback/gates e round-trip; transparente à
frente/atrás de opaco; sobreposição/interseção com ordem de instâncias invertida;
absorção RGB/espessura e Fresnel por ângulo; cobertura de recorte e sombra;
aditivo/emissivo; shared/legacy/skinning, edição só de material e cache quente/
frio; atmosfera/soft edges/seleção; caminho totalmente opaco preservado.

Galeria nativa editável em saída estável `artifacts/material-transparency`, com
receita/cena reproduzíveis e capturas offscreen. Não substituir a cena/janela
humana em uso. Build corrente único; instalação e aceite artístico separados.

## Fonte e integração autoral

O domínio opcional `Material.surface` tem versão própria 1. Cena com esse domínio
exige Scene110; replay120 e conteúdo público8 carregam a nova receita. As versões
de protocolo/checkpoint/save portátil e das fontes botânicas/personagens não são
alteradas por este domínio embutido. Leitores antigos recusam o campo desconhecido;
ausência do domínio mantém os formatos e fingerprints de material anteriores.

Exemplo de material de lâmina verde, aplicado a qualquer asset de mesh:

```json
{
  "surface": {
    "version": 1,
    "mode": "blend",
    "opacity": 1,
    "glass": {
      "ior": 1.5,
      "roughness": 0.2,
      "thickness_m": 0.02,
      "attenuation_color": [0.05, 0.6, 0.18],
      "attenuation_distance_m": 0.04
    }
  }
}
```

`edit_mesh/set_material` e `preview_mesh_edit` usam o mesmo tipo validado do
painel Aparência. Inspeção, undo/redo, save/load e réplica conservam a receita.
O painel expõe modo, opacidade/cutoff, cobertura por ruído/cristas/células,
emissivo RGB, IOR, rugosidade, espessura e cor/distância de absorção.

| Campo | Contrato |
| --- | --- |
| `mode` | `opaque`, `mask`, `blend`, `additive` |
| `opacity` / `cutoff` | 0..1; opaque exige cobertura integral |
| `coverage` | Campo escalar no repouso, independente da cor; frequência XYZ0,01..2048, low/high0..1, seed16bits e kind |
| `emissive` | Radiância RGB0..64; saída atual SDR, sem bloom/exposição novos ou criação automática de luz/GI |
| `glass.ior` | 1..2,5; F0 derivado do índice e Fresnel angular |
| `glass.roughness` | 0,2..1, compatível com GGX/IBL existentes; afeta reflexão |
| `glass.thickness_m` | 0..10m por superfície; caminho interno considera o ângulo de transmissão |
| `glass.attenuation_color` | RGB0..1 transmitido na distância de referência |
| `glass.attenuation_distance_m` | 0,0001..10000m; Beer–Lambert |

Vidro exige `blend`, sem `reflectance` ou `transmission` legados concorrentes.
Tintas/cor base modulam a cor de absorção na distância de referência. O recorte
descarta quando cobertura < cutoff, incluindo a sombra. A seleção do editor
contorna a superfície geométrica contra profundidade opaca: não gera bordas para
cada furo procedural nem torna um objeto oculto por opaco visível por raio X.

OIT acumula em três RGBA16Float e um fundo no formato da saída: 28bytes/pixel
adicionais para RGBA8, fora da profundidade/IBL/seleção/edge finish já existentes.
Esse número é payload lógico dos targets, não VRAM total/alinhamento do device.
Desativa/libera os targets quando não há submissões transparentes na câmera;
reutiliza resolução corrente. Emissão aditiva é composta atrás do produto de
transmitância das lâminas; sua ordenação mista é uma aproximação explícita.
Sob overdraw extremo, fp16 pode saturar: composição limita valores antes de
normalizar para evitar NaN, sem certificar fidelidade para camadas ilimitadas.

Geometria24/48/64B e instancing permanecem disponíveis; cobertura procedural
requer coordenada de repouso e impede o layout24B. A tabela de material cresce
de1232 para1376bytes por asset, sem duplicação por vértice. Fontes com domínio
Surface não usam atalhos atmosféricos/LOD geométrico sem novo certificado;
materiais masked/translúcidos não são oclusores opacos. Soft edges do ambiente
acabam o fundo opaco antes da composição; translúcidos mantêm sua silhueta própria.

## Estado

Fundação implementada localmente. A validação dirigida passou no Mac/Metal:

- 33 testes Rust: 4 novos de superfície (3 com GPU), 6 de materiais legados,
  7 de domínio material, 7 de acabamento, 1 de conteúdo público, 5 de ABI/encoding,
  1 de versionamento de personagem e 2 de renderer na GPU.
- Os dois testes gráficos existentes cobrem 13 pares de oclusão ON/OFF (incluindo
  parede translúcida e recorte que não podem semear profundidade opaca) e 22 pares
  de residência quente/fria. Pixels exatos nas comparações desses roteiros.
- Novos oráculos verificam alpha contra cor linear conhecida, opaco à frente,
  Fresnel frontal/oblíquo, IOR1, absorção RGB, canal zero e espessura zero sem NaN,
  emissivo opaco/aditivo, sobreposição com ordem invertida e liberação dos targets.
  Também exercem cobertura/sombra, caminhos shared/legacy e cauda de apresentação
  após compactação atmosférica, com tolerâncias RGB8 explícitas onde apropriado.
- Clippy all-targets com `-D warnings`, formatação dos arquivos próprios,
  compilação Python, links documentais e diff sem erros de whitespace.
- Galeria MCP nativa: schema, preview sem mutação, undo/redo, rejeição atômica,
  save/load, replay/capture de baseline tick0 e save portátil com PNG exato.
  Isso prova persistência desta cena estática, não toda trajetória física ou VFX
  temporal futuros. Cena opaca também coincidiu byte a byte com a captura do
  executável anterior (Scene109), cujo SHA está em `baseline.json`.
- Cinco vistas de materiais e captura do editor inspecionadas; os parâmetros de
  vidro estão visíveis no painel Aparência. Sem teste humano de mouse/teclado ou
  aceite artístico inferido a partir dessas capturas.

Fonte reproduzível: [demo_material_transparency.py](../scripts/demo_material_transparency.py),
cena [material-transparency.scene.json](../examples/material-transparency.scene.json).
Galeria corrente `artifacts/material-transparency/review.html`, evidências em
`verification.json` e `native-checks.log` nessa pasta. O roteiro usa workspace
isolado de dados e o executável corrente `target/release/ignis`; não abre janela.

Os checks foram executados na união do checkout com a frente orbital199 em
andamento. O commit de203 separa os hunks de materiais dos campos/projeção/
atmosfera planetários. Compatibilidade artística dos novos materiais em órbita,
novas plataformas e FPS não são certificados. Build corrente único compartilhado,
sem cópia histórica, sem push e sem modificar a cena/janela humana nesta frente.
EAF-203 permanece em revisão humana; o recorte técnico não conclui o sistema VFX
nem as extensões ópticas/volumétricas descritas acima.
