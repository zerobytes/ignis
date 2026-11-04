# Habilidades funcionais de Land of Survivors

4 de novembro de 2026 · LOS-19.

As 284 entradas do [catálogo de design](habilidades.md) possuem definições executáveis e comportamento nativo em Rust. São 264 habilidades humanoides nas seis especializações e vinte de Urso/Felino. O componente de gameplay usa a física e o controlador de personagem do Ignis em um laboratório técnico próprio. O trabalho de relevo está preservado e temporariamente fora desta etapa.

## Executar

Na raiz do Ignis:

```sh
python3 games/land-of-survivors/abilities.py --verify
python3 games/land-of-survivors/abilities.py
python3 games/land-of-survivors/abilities.py --mcp
```

O launcher compila quando necessário o único executável corrente do laboratório em `target/release/examples/land_of_survivors`. Python apenas apresenta comandos e transporta dados. O relógio, os efeitos, a seleção de destinatários e a física executam em Rust. O processo é headless: não há animação, VFX, áudio, personagem artístico ou HUD gráfico. A janela planetária não recebe esses comandos.

O terminal mostra barras, vida, mana, estados, posições e recibos. `step 2` avança dois segundos a 60 ticks/s; não existe relógio de parede automático nesse cliente. Exemplos:

```text
cast 1
step 2
step 1
new frost dps
cast los.mage.frost.frozen_ground
confirm
step 1
move enemy 1 0 1
new nature dps
cast los.druid.nature.bear_form
bar
save
load
```

`list` mostra as vinte opções do foco atual, ou as dez da forma animal. `build frost healer` troca especialização/foco da mesma classe respeitando as regras de combate; `new nature dps` cria explicitamente outro encontro de laboratório. `equip` recebe dez IDs distintos disponíveis. O uso por slot passa pela barra; a invocação por ID é a interface de autoria para testar qualquer habilidade desbloqueada do kit. `point SLOT X Z` prepara uma escolha de chão; `confirm` envia a conjuração e `cancel` descarta a escolha sem custo. Cancelar durante a conjuração também não debita mana nem abre recarga.

O laboratório possui aliados feridos e adversários representados por primitivas físicas novas. Os times de ensaio são explícitos e não definem hostilidade entre espécies. `ai enemy on` habilita uma reação técnica por ameaça/provocação, com aproximação física e ataque básico. Não substitui a futura autoria dos NPCs do jogo.

## API compartilhada e arquivos

O servidor MCP oferece `los_game`, com schema tipado para `new`, `inspect`, `catalog`, `cast`, `slot`, `cancel`, `step`, `move`, `build`, `equip`, `mount`, `ai`, `save` e `load`. A interface do terminal e o MCP chegam ao mesmo dispatcher. Também é possível enviar um comando JSON por linha diretamente ao executável:

```json
{"op":"cast","actor":"player","ability":"los.mage.fire.fireball","aim":{"direction":[0,0,-1]}}
{"op":"step","ticks":120}
{"op":"inspect"}
```

- `games/land-of-survivors/content/abilities.json`: conteúdo executável, com IDs/textos preservados e seleções/efeitos explícitos.
- `games/land-of-survivors/compile_abilities.py`: transforma o catálogo de design em conteúdo; não importa implementação anterior. Campos desconhecidos interrompem a compilação. O runtime não adivinha efeitos pelos nomes.
- `games/land-of-survivors/src/schema.rs`: contratos e validação do catálogo.
- `games/land-of-survivors/src/combat.rs`: componente de execução, recursos, barras, efeitos, entregas e persistência do jogo.
- `games/land-of-survivors/src/lab.rs` e `examples/land_of_survivors.rs`: encontro técnico, dispatcher e transporte MCP/JSON.
- `src/physics/gameplay.rs`: deslocamento explícito pelo controlador físico existente, sem editar a cena ou reiniciar a simulação.

O componente usa seu catálogo de jogo completo. Não aumenta silenciosamente os limites ou reinterpreta o catálogo legado de habilidades da engine. Os formatos de Scene/replay/PSV da engine continuam preservados. A execução deste componente não está integrada ao replay ou ao multiplayer legado.

## Funcionamento

Custos e recarga são confirmados ao terminar a conjuração, após revalidar disponibilidade, vida, recurso, destinatário, alcance e visibilidade. Trocas de build preservam recursos, recargas por ID e barras próprias. Exigem forma original, vida, estar desmontado, nenhuma conjuração/entrega ativa e oito segundos sem ação de combate ou dano. As formas substituem a barra por dez habilidades; retorno recupera a barra anterior. Transformação não repõe vida/mana, bloqueia montar e é removida na morte.

Projéteis têm velocidade, dimensão física, distância restante e trajetória reta congelada. O contato é varrido contra as primitivas físicas do laboratório, incluindo obstáculos; não acompanha automaticamente um alvo que se move. Múltiplos disparos produzem projéteis separados. Bola de Fogo conjura por dois segundos, viaja, explode com dano de 600 e raio de 3 m e instala oito pulsos de queimadura de 10. Aliados não recebem seu dano.

Áreas e cones selecionam destinatários por posição, visada e relação. Círculo de Fogo aplica dano aos inimigos e cura aos aliados em cada pulso. Entrar/sair de uma área muda sua influência; a morte do ator usado para escolher o ponto não encerra a zona. Cones têm direção; efeitos sustentados podem ser cancelados. Periódicos em alvo acompanham seu destinatário, inclusive a cura própria de Urso/Felino. Encadeamentos não repetem um destinatário; as descrições com três alvos adicionais permitem quatro destinatários ao todo.

Dano passa por armadura, mitigação e escudos antes de reduzir a vida. Cura não ressuscita. Lentidão e imobilização alteram deslocamento físico; limpeza remove esses impedimentos. Provocação prevalece na escolha de alvo do adversário técnico. Avanços e empurrões usam o controlador com colisão. Escudos, mitigação, restauração de mana e execução têm efeitos concretos, sem dependência de apresentação.

## Balanceamento provisório explícito

Os valores que já estavam no catálogo foram mantidos. Para executar regras incompletas, esta unidade adota os seguintes parâmetros ajustáveis, que não são decisões finais herdadas:

| Regra incompleta | Parametrização inicial |
| --- | --- |
| Pulso sem intervalo numérico | 1 s, conforme descrições do catálogo; pulso inicial na entrega, restantes durante a duração. Queimadura de Bola de Fogo começa 1 s após impacto. |
| Cone | Abertura total de 90°; cones descritos com pulsos funcionam como canalizações canceláveis. |
| Encadeamento | Até 6 m entre destinatários sucessivos, com visibilidade. |
| Múltiplos disparos | Separação angular de 0,045 rad entre disparos. |
| Execução | Dano multiplicado por dois quando a vida está em até 25%. |
| Armadura | Multiplicador de dano `100 / (100 + armadura)`; Urso multiplica armadura por 1,8. |
| Felino | Velocidade multiplicada por 1,3. |
| Mitigação e lentidão concorrentes | Prevalece a maior intensidade; os efeitos mantêm suas durações. |
| Escudo reaplicado da mesma fonte/habilidade | Acumula até a vida máxima e renova duração; fontes diferentes mantêm camadas próprias. |
| Provocação sem duração | 3 s; modificadores com duração de habilidade são renovados a cada aplicação/pulso. |
| Empurrão herdado sem unidade explícita | Valor dividido por 100 como velocidade em m/s, aplicado por 0,25 s com colisão. Essa interpretação deve ser rebalanceada. |
| Laboratório | Vida máxima 1.000, mana máxima 600, início ferido com vida 500/mana 300, armadura zero; não é progressão final. |

## Persistência, limites e validação

O save de jogo versão 1 reúne catálogo, recursos, barras, recargas, formas, modificadores, conjurações, projéteis, periódicos e o save portátil físico do Ignis. Carregar valida o candidato antes de substituir a sessão; os relógios precisam concordar. O cliente mantém um único `artifacts/abilities/current.ignis-game`. O formato do jogo não altera PSV1 e não promete compatibilidade com executáveis anteriores.

Limites atuais: 128 combatentes, 1.024 definições, 4.096 projéteis e 4.096 agendas ativas, 256 modificadores por combatente, anel diagnóstico de 512 eventos, save de até 32 MiB, passos explícitos de até 3.600 ticks. Limites não equivalem a carga ou FPS comprovados. A seleção de combate deste laboratório cobre caixas/cunhas físicas e chão dessas primitivas; integração com a superfície planetária, colisores de meshes gerais e atores em escala global permanece outra etapa.

A cobertura comportamental executa cada uma das 284 entradas sobre atores físicos do Ignis. Casos críticos protegem consumo/cancelamento/revalidação, dano e destinatários, colisão de projéteis e avanços, movimento impedido/limpo, barras e transformações sem ganho de recursos, persistência de combate em andamento e rejeição de estado inválido. Regressões selecionadas preservam habilidades/vitalidade/controlador legados. O relatório reproduzível do executável fica em `artifacts/abilities/verification.json`.

Validação local: 19 testes do jogo e 54 regressões selecionadas da engine passaram em release, totalizando 73. O executável também confirmou 284/284 habilidades; inicialização/schema MCP, ativação por slot, voo/impacto, morte, save/load, rejeição sem alteração de estado e uso do terminal foram exercitados. Clippy release dos alvos relevantes passou com warnings tratados como erro. Não se executou a suíte global ou um ensaio gráfico/humano de combate.

Isso entrega o funcionamento do catálogo em um laboratório nativo acessível por comandos. A conexão com o futuro HUD gráfico, modelos/animações, balanceamento humano, multiplayer e combate sobre o planeta ainda não está implementada nesta unidade.
