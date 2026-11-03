# Horizon: laboratório de vegetação em 1 km²

EAF-215, continuação de EAF-210. **Em investigação; não é uma entrega de qualidade/FPS aprovada.** A cena original e todas as fontes/bakes botânicos continuam intactos em `artifacts/botanical-valley`.

## Primeiro recorte: campo instanciado

`examples/horizon_meadow.rs compile` cria a cena corrente em `artifacts/horizon-meadow/world.scene.json`, mantendo o terreno físico/visual original (1.000 × 1.000 m, amostras a cada 2 m), personagem, árvores e arbustos. O novo campo rasteiro cobre a região inteira com sítios a cada 25 cm e jitter determinístico. Caminho e água são exclusões explícitas; não há zonas de densidade feitas para a câmera.

O campo contém 16.000.000 sítios, dos quais 15.667.937 plantados: 344.694.614 lâminas lógicas do exemplar nativo de 22 folhas. Isso descreve a população fonte, **não a quantidade de lâminas desenhadas por frame**. A prova de ocupação por células de 50 m está em `compile.json`; não substitui a revisão visual próxima/aérea.

`IGFLD1` armazena alturas/máscaras em f32 little-endian (4 bytes por sítio), métrica da grade e seis malhas compiladas compartilhadas. Coordenadas/jitter são reconstruídos pelo mesmo hash inteiro do compilador. O pacote tem 64.185.944 bytes, aproximadamente 61,2 MiB. Não há instâncias JSON individuais, geração botânica ou upload das alturas por frame. A cena autoral ainda usa o formato existente de Scene.

O renderer percorre blocos espaciais de 4 m, testa o frustum, escolhe uma escala pela resolução física e envia até seis lotes. O shader lê alturas/templates residentes; raízes fora do campo são mascaradas. A física permanece na fonte completa. Sombras têm seleção própria no volume da luz. `Renderer::set_surface_field` é uma API diagnóstica, desligada por padrão. O opt-in de processo `IGNIS_HORIZON_FIELD` aplica o mesmo pacote à janela nativa e às capturas MCP. O pacote não é persistido por SaveScene nem se revincula automaticamente a outra cena.

Este primeiro compilador usa estações da malha botânica nativa para formar superfícies finas. O renderer recebe geometria, alturas e escalas; não testa nomes de espécies. O exemplar/receita específico pertence ao laboratório. Rigs, crescimento, vento, shaders gerais, formatos globais e IGL1/ICB não são habilitados por este trabalho.

## Medição inicial

Metal/M4 Pro, 1.100 × 760 pixels, câmera/luz congeladas, três aquecimentos e oito amostras. Mediana de `Renderer::draw` mais espera de conclusão. **Não é FPS de apresentação nem timestamp isolado da GPU.** A ablação sem campo remove apenas o campo novo e conserva árvores/arbustos/terreno; não é uma referência visual equivalente de vegetação completa.

| Vista | Nativos sem campo (ms) | Nativos + campo (ms) | Incremento (ms) |
|---|---:|---:|---:|
| Jogador | 123,643 | 131,553 | 7,911 |
| Vale | 83,606 | 88,073 | 4,467 |
| Panorama | 299,747 | 306,451 | 6,704 |

O campo envia 2,71–5,30 milhões de triângulos de câmera, mais 513.964 candidatos de sombra. A memória GPU própria é 66.186.000 bytes. A preparação CPU observada nas capturas ficou em aproximadamente 1–3,3 ms; não é uma distribuição estatística independente. Os nativos restantes ainda enviam 138–327 milhões de triângulos. Dados/capturas: `field-probe.json`, `meadow-*-field.png` e `meadow-*-native_only_ablation.png`.

O teste de agrupamento anterior está em `batch-probe.json`: reduziu 376.078 para 14.331 draw units na vista do jogador, mas a geometria continuou cara. A ordenação candidata altera alguns empates de profundidade e **não foi promovida a reordenação global exata**.

## Gates abertos

- A troca de densidade/escala do campo ainda produz uma faixa visual. A compensação de largura atual não conserva a aparência integrada; não declarar qualidade preservada.
- O custo dominante continua nas malhas completas de árvores/arbustos. Compilar representações menores e medir contra os mesmos objetos, com fallback para a fonte quando detalhes forem visíveis.
- Ainda faltam maior densidade de árvores, comparação A/B com população idêntica, resolução física da janela, trajetórias de câmera/luz, FPS nativo e aceite humano. Não anunciar 60 FPS nem equivalência a Ghost of Tsushima.
- O recurso é um laboratório de sessão. Uma integração persistente com ferramentas MCP/UI e vínculo seguro de fonte será uma unidade posterior; não confundir este opt-in com um formato de mundo pronto.

## Reprodução

```sh
cargo run --release --example horizon_meadow -- compile
cargo run --release --example horizon_meadow -- probe
python3 scripts/open_horizon_meadow.py
```

A janela usa o personagem/KCC e o terreno originais, WASD/mouse, Shift para correr e medidor de apresentação nativo. Não abrir para aceite como otimização concluída enquanto os gates visuais acima falharem. `scripts/link_vegetation_probe.py --probe field --library <rlib>` permite reutilizar uma biblioteca release identificada, registrando hashes e removendo a cópia temporária de link, sem outro build monolítico.

Validação desta unidade: checagem do exemplo, dois testes do leitor binário/frustum e ensaio nativo completo com sombras. A primeira tentativa revelou vínculo incorreto do shadow map; a captura repetida após separar layout/bindings de profundidade passou. Cena/fonte canônicas não foram regravadas; os novos arquivos pertencem somente ao laboratório.
