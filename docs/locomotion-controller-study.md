# Rampas e degraus: experimento de controlador (EAF-35)

Estado: experimento concluído; proposta de integração para discussão. Não é uma nova capacidade de cena, NPC, MCP ou demo. O runtime continua no build bf1cde1c22341efe/cena24. A implementação está em `examples/locomotion_probe.rs`, separada da biblioteca e do binário da engine.

## Pergunta e resultado

O comando atual de velocidade sobre um corpo dinâmico basta para um personagem caminhar por rampas/degraus com limites explícitos? Nos casos testados, não: sobe uma rampa de40° sem limite de inclinação, mas para num degrau de20cm. Isso não é um defeito do solver; o motor atual simplesmente não implementa essas políticas. Poderíamos adicioná-las mantendo o corpo dinâmico, com consultas e controle de contato próprios.

O `KinematicCharacterController` da versão Rapier0.30.1 já presente no lockfile resolve os casos básicos de inclinação/degrau/teto. Contudo, transfere responsabilidades ao motor do jogo: gravidade/velocidade vertical, empurrões, knockback e transições aéreas precisam de regras explícitas. A proposta é um motor de personagem opt-in, compartilhado por jogador e NPC, preservando corpos dinâmicos e cenas antigas. O experimento não autoriza substituir todos os corpos.

## Método reproduzível

```sh
mkdir -p artifacts/locomotion-probe
cargo run --locked --release --example locomotion_probe > artifacts/locomotion-probe/report.json
cargo test --locked --release --example locomotion_probe
```

Onze casos, três motores, cada combinação repetida desde um mundo novo:33 comparações/66 execuções.240 ticks de1/60s após um passo inicial para publicar geometrias na broad phase. Caixa0,6×1,6×0,6m, velocidade solicitada2m/s, gravidade9,81m/s². Mesma geometria e ordem de inserção por caso; ator não muda para cápsula. Piso/caixa empurrável com atrito0,7 e personagem0, como no runtime. A combinação de atrito padrão do Rapier continua valendo: velocidade solicitada não é garantia de deslocamento efetivo2m/s.

Motores: (1) corpo dinâmico, rotações travadas/CCD, sobrescreve XZ e preserva velocidadeY, reproduzindo a política física de caminhada; (2) corpo cinemático posicional com movimento corrigido por shape casts; (3) o mesmo cinemático aplicando também `solve_character_collision_impulses` com massa de referência0,576kg. Nenhum caso executa percepção, planejamento de caminho ou o motor aéreo da Ignis.

No cinemático: subida máxima30°, início de deslizamento35°, autostep até0,3m/largura mínima0,4m, sem subir corpos dinâmicos, snap até0,15m e offset nominal0,01m. Gravidade é acumulada pelo experimento; velocidade vertical é zerada quando grounded. O estado grounded vem do controlador e não é inferido para o corpo dinâmico. Na borda, uma queda de1m exige perder contato e cair; não se estende snap para atravessá-la instantaneamente.

Relatório guarda configuração, build da engine, hash do próprio fonte do experimento (o build da engine não inclui exemplos), amostras a cada30ticks, hash da trajetória de todos os ticks, deslocamento efetivo, faixaY, grounded/callbacks quando aplicáveis e penetração geométrica máxima amostrada após cada passo. Hash usa bits f32 e ordem fixa; repetição compara também os resultados completos. Isto estabelece repetibilidade neste build/macOS, não determinismo entre plataformas, replay integrado ou ausência de contatos intermediários entre ticks.

## Evidência obtida

Valores arredondados, centros dos corpos em metros. Todas as33 combinações repetiram os mesmos resultados. As condições funcionais abaixo também são asserts do executável e do teste, com margens geométricas; não são snapshots de números exatos.

| Caso | Dinâmico atual | Cinemático | Cinemático com impulsos |
|---|---|---|---|
| Piso plano | X final6,354 | X6,499 | X6,499 |
| Rampa20° ascendente | Chega ao patamar, X6,218 | Chega, X5,831 | Igual |
| Rampa40° ascendente | Sobe, X5,828 | Recusa, X−0,293 | Igual |
| Rampa20° descendente | Desce ao piso, X−2,424 | Desce, X−2,498; grounded240/240 | Igual |
| Degrau20cm | Para antes, X−0,299 | Sobe, X6,456 | Igual |
| Degrau50cm | Para | Recusa: acima do limite30cm | Igual |
| Degrau20cm sob teto baixo | Para | Recusa: sem espaço para o corpo | Igual |
| Parede3m | Para | Para | Igual |
| Borda com queda1m | Cai e aterrissa | Cai;25ticks sem grounded | Igual |
| Caixa dinâmica1kg | Empurra5,541m | Caixa praticamente imóvel | Empurra3,087m |
| Impulso externo2N·s no ator | Desloca5,7cm; comando seguinte cancela XZ | Sem deslocamento horizontal | Sem deslocamento horizontal |

Última linha: o impulso chega depois do comando, antes da física, no tick61. O corpo dinâmico reage naquele passo, mas o motor atual sobrescreve a velocidade horizontal no seguinte. Não é uma comparação com um controlador dinâmico completo que preserve knockback. O cinemático não recebe resposta automática a esse impulso; habilitar empurrões para objetos não implementa a reação inversa no personagem.

Nos casos testados, a penetração máxima observada no dinâmico ficou abaixo de9mm, compatível com tolerâncias de resolução deste experimento; nos cinemáticos o máximo registrado foi zero. Não interpretar o segundo resultado como prova de ausência universal de interpenetração. Todos os casos exigem penetração pós-passo menor que2cm.

## Proposta de arquitetura e tradeoffs

| Decisão | Recomendação | Alternativas e custo |
|---|---|---|
| Geometria | Contrato tipado compartilhado, começando por caixa e cunha convexa maciça | Caixa rotacionada exige rotação em todos os consumidores e tem base inclinada; malha/heightfield serve melhor a terrenos, mas traz pipeline/consultas maiores. O ensaio usa cunha somente no Rapier, ainda sem representação na cena. |
| Personagem | Motor cinemático opt-in, com política explícita de chão/degrau/impulsos | Manter dinâmico e implementar essas políticas preserva interação física automática, mas exige outro controlador. Não há evidência de que uma opção seja universalmente mais rápida. |
| Dono do movimento | Um motor por ator, recebendo intenção de jogador ou NPC | Evitar que navegação, solver e ações escrevam a pose independentemente. Não alternar tipo de corpo a cada degrau/salto sem contrato de transição. |
| Compatibilidade | Dinâmico legado continua padrão; geometrias/controladores novos exigem formato novo | Migração automática mudaria empurrões, contato, passos, distâncias e replays antigos. |
| Custos | Consultas limitadas e estado/scratch reaproveitáveis; impulsos opt-in | Rapier0.30.1 usa Vec de manifolds em grounded e na resolução de impulsos. Autostep é caro por desenho. Isso precisa entrar na revisão da integração, sem afirmar zero alocação. |

Um enum de motor e uma configuração tipada são suficientes para o primeiro recorte; não há motivo demonstrado para alocação/dispatch dinâmico de um plugin por personagem/tick. O corpo do personagem pode continuar alinhado e sem rotação física enquanto apresentação/orientação visual evoluem separadamente.

Ponto crítico: ter uma rampa no solver não basta. Renderização, picking, linha de visão, oclusão sonora, apoio e clearance precisam consultar a mesma geometria. O planejador também precisa testar a travessia de um degrau; o sweep linear atual contra caixa expandida não aceita a subida só porque o motor cinemático sabe executá-la. E o snap deve respeitar a política de apoio: o EAF-34 exige o footprint completo, enquanto o controlador pode aceitar contato parcial. Não relaxar essa diferença silenciosamente.

## Contrato MCP proposto e sequência

1. Geometria de entidade com discriminante explícito, caixa como default e cunha parametrizada por dimensões/eixo de subida. Mesma compilação alimenta render/colisor/raycast/sweep. Validar finitude, limites, versão e combinações antes de mutar, com autoria em `upsert_entity`/`edit_scene`, histórico e save/load existentes. A receita procedural deve produzir o mesmo documento editável.
2. Motor de personagem opt-in para jogador e NPC: inclinação, autostep, snap, offset, política de empurrões e estado vertical. Unidades públicas metros/graus/ticks; conversão na carga. Inspeção MCP deve expor grounded, apoio/normal, movimento solicitado/real e motivo de bloqueio. Configuração/inspeção pelo dispatcher antes de qualquer UI.
3. Planejamento usa consultas compatíveis com esse motor para links/conectores, sem prometer navmesh ou descoberta global. Caminhada valida apoio/headroom/transições; queda involuntária não vira salto autorizado. Revisar interpretação da velocidade ao projetar movimento na rampa e a política de apoio parcial.
4. Integrar salto/planeio/voo, impulso/knockback, plataformas móveis e convivência dinâmica por contratos próprios. Este experimento não os valida. Antes de produção, validar degraus em sequência, larguras, subida/descida/diagonal, spawn penetrado, mudanças de geometria, latência de consulta e coordenação entre personagens.
5. Demo autorada/testada por MCP headless: duas rampas, escada, teto, limite de capacidade, perseguição sem conhecimento oculto, alteração/undo, replay/prefixo/PNG. Gate visual humano separado. A janela atual permanece preservada.

O próximo incremento de produção recomendado é o contrato compartilhado de geometria caixa/cunha, antes de conectar o controlador aos NPCs. A escolha estrutural fica apresentada aqui para revisão conjunta; não foi incorporada ao runtime neste estudo.

## Validação e fontes

207 testes Rust release passaram (206 existentes e1teste do experimento cobrindo33combinações/repetições), fmt/clippy aprovados. Regressão MCP headless recortada `lab`+`walk_support`:14grupos aprovados, runtime inalterado. Não se afirma que a suíte completa32roteiros foi repetida nesta unidade. Logs/relatório em `artifacts/locomotion-probe/` e `artifacts/mcp-validation/report.json`. Sem instalação, nova janela, push ou publicação.

Fonte autoritativa de API/custos: código da dependência fixada `rapier3d-0.30.1/src/control/character_controller.rs`, especialmente `move_shape`, detecção de chão, `handle_stairs` e `solve_character_collision_impulses`. A [documentação oficial de controladores](https://rapier.rs/docs/user_guides/rust/character_controller/) explica o modelo, mas a página consultada já documenta0.36: as chamadas do ensaio foram conferidas na0.30.1 local. Nenhuma atualização de dependência foi feita.
