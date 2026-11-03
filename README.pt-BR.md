<div align="center">
  <img src="website/dist/assets/lockup.webp" alt="Ignis Engine" width="460">
  <h3>Uma engine 3D para humanos e agentes de IA construírem juntos.</h3>
  <p>Construir → executar → inspecionar → capturar → refinar. Uma API. Um mundo.</p>
  <p><a href="README.md">English</a> · <a href="docs/features.md">Recursos</a> · <a href="https://ignis.web.app/">Site</a> · <a href="docs/quickstart.md">Começar</a> · <a href="docs/status.md">Estado atual</a> · <a href="https://github.com/zerobytes/ignis/discussions">Comunidade</a></p>
</div>

![Ignis Horizon: vale com florestas cercado por cadeias de montanhas](docs/media/horizon.webp)
***Ignis Horizon.** Vales, florestas e montanhas renderizados pela engine nativa. Captura selecionada pelo criador para o site.*

Ignis é uma **engine 3D com código disponível em Rust, com autoria via Model Context
Protocol (MCP) e editor visual nativo**. Humanos e agentes usam a mesma API de
comandos para editar, executar, inspecionar e pedir capturas reais da cena.

O núcleo funciona **offline**, sem conta, chave de API ou modelo de IA.
Você conecta seu próprio cliente MCP quando quiser trabalhar com um agente.

## O que você pode construir

Crie paisagens e bibliotecas de plantas editáveis, modele personagens e objetos,
prototipe gameplay com NPCs ou automatize autoria e verificação por um agente.
O código público reúne os sistemas abaixo.

| Área | Capacidades disponíveis |
| --- | --- |
| **[Renderização Ignis Horizon](docs/features.md#ignis-horizon-rendering)** | Iluminação e sombras nativas, geometria compartilhada, visibilidade/oclusão GPU, residência e representações distantes orientadas pela atmosfera. |
| **[Materiais e detalhes de superfície](docs/features.md#materials-and-surface-detail)** | Cor, rugosidade, bump e detalhe triplanar procedurais; dielétricos/metais; recorte, translucidez, aditivo e vidro de lâmina fina. |
| **[Mundos, terrenos e planetas](docs/features.md#worlds-terrain-and-planets)** | Terrenos e cristas editáveis, formas e cavidades, colisão, coordenadas globais e runtime planetário experimental com caminhada e LOD da superfície ao globo. |
| **[Botânica procedural](docs/features.md#procedural-botany)** | Árvores, palmeiras, gramas e arbustos por órgãos e curvas; ramificação, idade/vigor, raízes, contrafortes e colo. Laboratório separado de deformação/crescimento/poda. |
| **[Personagens paramétricos](docs/features.md#parametric-characters)** | Proporções do corpo, postura, controles locais de rosto/orelhas/lábios/nariz, manipuladores XYZ com simetria e regeneração preservando a fonte. |
| **[Modelagem e escultura](docs/features.md#modeling-and-sculpting)** | Edição de vértices/arestas/faces, extrusão/inset/bevel, pincéis e refinamento local, sweeps, junções orgânicas e objetos compostos. |
| **[Física e controle do jogador](docs/features.md#physics-and-player-control)** | Simulação fixa ou contínua, colisão, controladores, rampas/degraus, gatilhos e exploração em primeira/terceira pessoa. |
| **[Percepção e personalidade dos NPCs](docs/features.md#npc-perception-and-personality)** | Visão, audição, passos, memória, busca, relações, comunicação, inteligência, traços independentes e decisões inspecionáveis. |
| **[Navegação, tática e interação](docs/features.md#navigation-tactics-and-interaction)** | Rotas geradas, circulação terrestre/aérea, caminhada/salto/planagem/voo, cobertura/flanco, ordens/formações, agendas, necessidades, objetos e diálogo estruturado. |
| **[Habilidades e combate](docs/features.md#abilities-and-combat)** | Recursos/cooldowns, casts/canalização/interrupções, projéteis, dano/cura, buffs, dissipação, áreas, auras, morte e ressurreição. |
| **[Populações e nascimento](docs/features.md#populations-and-spawning)** | Templates, identidade individual persistente, regiões seeded/ponderadas, geradores de população e ondas finitas. |
| **[Animação e áudio](docs/features.md#animation-and-audio)** | Rigs/clipes, retarget, IK visual e root motion opcional; mixagem PCM espacial, eventos e saída opcional em dispositivo. |
| **[Multiplayer e runtime de atores](docs/features.md#multiplayer-and-actor-runtime)** | Sessões QUIC/TLS autoritativas opcionais, controles escopados, réplica/reconexão e ensaios documentados de 1.000 jogadores locais/loopback. |
| **[Editor, MCP e persistência](docs/features.md#editor-mcp-and-persistence)** | Projetos, áreas de trabalho, navegador de arquivos, comandos tipados, undo/redo, capturas reais, comparação de imagens, replay, checkpoints e saves portáteis. |

O **[guia completo de recursos, em inglês](docs/features.md)** detalha as
capacidades e seus contratos. [Estado atual](docs/status.md) diferencia
integrações, experimentos, laboratórios e planos futuros.

**Desenvolvimento inicial:** a engine funciona, mas ainda tem mudanças
incompatíveis e trabalho substancial em aberto. A validação desktop está
concentrada em macOS Apple Silicon. Não há promessa de engine pronta para
produção, FPS universal ou suporte multiplataforma já aceito.
[Recortes e limites atuais](docs/status.md).

## Experimente

Instale Rust 1.88 ou mais recente e as ferramentas nativas de desenvolvimento.
Janelas e capturas exigem GPU compatível; inspeção e simulação podem rodar sem janela.

```sh
git clone https://github.com/zerobytes/ignis.git
cd ignis
cargo build --locked --release
./target/release/ignis --root "$PWD" --editor --scene examples/stealth-lab.scene.json
```

Troque `--editor` por `--window` para a demo jogável. **P** inicia/pausa,
**WASD** move e **R** reinicia. [Configuração e diagnóstico](docs/quickstart.md).

## Conecte um agente

Configure o cliente MCP stdio com caminhos absolutos:

```json
{
  "mcpServers": {
    "ignis": {
      "command": "/caminho/absoluto/ignis/target/release/ignis",
      "args": ["--root", "/caminho/absoluto/ignis", "--mcp"]
    }
  }
}
```

Acrescente `--editor` aos argumentos para compartilhar a mesma sessão com o
editor visual. A ferramenta `engine_command` expõe o schema real dos comandos.
Peça ao agente para inspecionar a cena, construir um exemplo pequeno, executar
passos e capturar o resultado antes de propor a próxima alteração.

Os [exemplos e links por área estão no README principal](README.md#explore-the-engine).
Há também um [catálogo de demos em português](docs/demo-catalog.pt-BR.md).

## Capturas da engine

<table>
  <tr>
    <td><img src="docs/media/sunset.webp" alt="Silhuetas de montanhas e camadas atmosféricas ao pôr do sol na Ignis" width="600" height="388"></td>
    <td><img src="docs/media/cottage.webp" alt="Estudo de arquitetura nativa: casa detalhada renderizada na Ignis" width="600" height="400"></td>
  </tr>
  <tr><td>Silhuetas de montanhas e camadas atmosféricas. Captura nativa.</td><td>Estudo de arquitetura. Captura nativa da Ignis.</td></tr>
</table>

## Participe e acompanhe

Experimente o início rápido e relate o primeiro ponto que falhou, com ambiente
e comando exato. Melhorias de build, exemplos, documentação e integração MCP
são bem-vindas. Leia [CONTRIBUTING](CONTRIBUTING.md), abra uma
[issue](https://github.com/zerobytes/ignis/issues/new/choose) ou compartilhe sua
experiência em [Discussions](https://github.com/zerobytes/ignis/discussions).
Uma estrela ajuda quem quer acompanhar o projeto.

## Licença e autoria

Criado por **Guilherme Oliveira ([zerobytes](https://github.com/zerobytes))**.
Esta distribuição usa a [Ignis Engine Commercial Source License 1.0](LICENSE):
código disponível sob licença comercial, **não open source nos termos da OSI**.

**Grátis para começar. Depois de US$ 1 milhão de faturamento acumulado por jogo,
a participação é de 1% do lucro elegível, após custos elegíveis e prejuízos não
recuperados.** O primeiro milhão é isento, sem cobrança retroativa sobre o lucro
isento. Sem lucro elegível, não há royalty. Jogos proprietários são permitidos.

Preserve [NOTICE](NOTICE), a licença e os avisos de autoria aplicáveis.
Leia a [política e os exemplos de cálculo](docs/licensing.md) e mantenha
[relatórios financeiros privados](docs/royalty-reporting.md). O texto inglês
controla. Terceiros mantêm [suas licenças](THIRD_PARTY_NOTICES.md). A substituição
do repositório não revoga direitos Apache-2.0 de cópias publicadas anteriormente.
