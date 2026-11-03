# Horizon: preparo de quadros durante movimento planetário

Continuação de EAF-199. O teste parado de desenho e espera da GPU não validava
os picos da janela em movimento. A investigação separa controle da câmera,
passo físico/residência e preparo do `RenderFrame`; apresentação/FPS continuam
um gate próprio. Não altera a fonte procedural, a densidade atmosférica, a
resolução física, o orçamento de erro visual nem a velocidade do jogador.

## Diagnóstico e alteração

O giro replanejava a cobertura global e uma assinatura de costura diferente
provocava novo bake de todo o tile. Agora, o bake reutiliza amostras do mesmo
tile imutável quando a chave de superfície **e o nível de filtragem da normal**
coincidem. Copia posição, normal, domínio de material e coordenada high/low
originais. A busca usa hash, mas os índices continuam atribuídos pela travessia
determinística; a ordem de iteração do hash não determina nenhum byte da malha.
O limite de erro de conversão reaproveitado permanece conservador. Só se retém
o conjunto atual de recursos; não há histórico ilimitado de regiões retiradas.

Outra despesa era expandir novamente todos os cantos dos triângulos da CPU
quando mudava o catálogo ou um material. O quadro planetário canônico mantém
os arrays indexados existentes. Seus ranges representam contagens lógicas;
o renderer lê diretamente posição, normal, domínio e high/low por índice.
A pequena malha do corpo continua no fluxo expandido. Layouts que precisam
do fluxo por canto o geram sob demanda na falta da cache GPU, preservando o
caminho detalhado. Sair desse modo força a reconstrução do quadro legado.
Receitas de material com variação única por instância usam o quadro legado,
preservando a derivação de seus slots; mapas fixos e receitas comuns continuam
elegíveis ao fluxo canônico.

Isso elimina trabalho repetido; **não acrescenta a redução atmosférica da
geometria ao atlas planetário**. O material distante barato já existe nesse
atlas. A seleção geométrica atmosférica do renderer anterior continua uma
integração separada, com limites de contorno e contribuição visual próprios.

## Reprodução

Usa os arquivos correntes de `artifacts/planet-runtime`, sem janela humana:

```sh
cargo build --release --bin ignis --example planet_motion_cost
target/release/examples/planet_motion_cost --reference
target/release/examples/planet_motion_cost
```

Mesma cena, câmera a 4 m, viewport 2248×1580, 120 ticks iniciais de acomodação;
12 ticks parados, 240 ticks correndo e 24 giros de 2 graus, um tick por amostra.
`--reference` desativa apenas o reuso de amostras e o quadro canônico.
`cpu-reference.json` e `cpu-current.json` separam controle, avanço e quadro;
incluem poses, deslocamento real, nascimento de tiles e contagens do atlas.
São tempos CPU sem GPU, IPC ou apresentação; não converter seu inverso em FPS.
Rodar sem compilação e sem outra carga de validação concorrente.

A validação deste incremento usa também um checkout temporário de `2923cee`
com apenas o patch Horizon: fontes de olhos, partículas e vento ainda em edição
no checkout compartilhado bloquearam a compilação dos testes. Reutiliza a cache
`target` do projeto, mas compila somente os testes e o exemplo de custo; não
substitui o executável da janela por uma versão sem as outras frentes. Gates
desse snapshot não significam que as alterações alheias já estão integradas.

`scripts/verify_planet_pacing.py` cobre também os comandos MCP e seus contadores.
O RTT inclui IPC/JSON e não inclui preparo do quadro ou desenho. A inspeção fica
fora do intervalo medido. `--cache-off` mantém o bake anterior como oráculo.
Os campos `last_update_ms` e `last_prepare_ms` são diagnósticos da última chamada,
inclusive chamadas quentes, não acumuladores do frame completo.

## Validação e fronteiras

O teste de costura compara payload frio/reutilizado; o teste CPU compara o
fluxo expandido, a contagem de triângulos, a mudança de material, a cache quente
e o retorno ao legado. O oráculo GPU de materiais/layouts também compara o
quadro canônico, incluindo o fallback não indexado e material detalhado.

Os 13 testes `planet_orbit` passaram no snapshot isolado, inclusive os dois
oráculos GPU ignorados por padrão (Metal nativo): contornos/partições, frustum,
reverse-Z, materiais, fallback por canto, câmera a 4 m e mudança de resolução.
O fluxo CPU expandido é byte exato, inclusive os sinais de zero produzidos pela
transformação identidade do avaliador legado. Mudanças de normais/coordenadas
mantêm os placements; o retorno também cobre documento vazio e slots únicos.
Isso não substitui validação de pacing/FPS da janela.
`cargo clippy --release --lib --example planet_motion_cost -- -D warnings`,
formatação dos fontes alterados e sintaxe Python passaram. Build do snapshot:
`4699b65e3ccc0a84` (aarch64-apple-darwin/release/Rust 1.97.1), não uma instalação
ou validação do executável com todas as frentes ainda abertas na main.

O diagnóstico CPU exploratório abaixo ocorreu **com compilação concorrente**.
São 240 ticks correndo (2402,94 m reais, mesmas poses/triângulos), 24 giros de
2 graus e 12 ticks parados, usando o mesmo executável para referência e atual:

| Segmento, total CPU sem GPU | Referência | Canônico |
| --- | ---: | ---: |
| Corrida p95 | 77,984 ms | 24,176 ms |
| Corrida p99 | 90,482 ms | 32,510 ms |
| Corrida máximo | 204,664 ms | 112,489 ms |
| Giro p95 | 135,946 ms | 68,352 ms |
| Giro p99 | 146,881 ms | 71,785 ms |
| Giro máximo | 273,590 ms | 180,894 ms |

Nas trocas, a fase de preparo do quadro ficou na faixa de 49–55 ms no expandido
e 0,3–1,3 ms no canônico. O payload de cantos CPU caiu de aproximadamente
4,77 milhões para 5184 (corpo), mantendo aproximadamente 1,59 milhão de
triângulos. A fonte indexada e os buffers GPU continuam residentes; não é
uma medição de RAM total. O cache de amostras acrescenta 24 bytes de chave e
1 byte de nível por vértice dos tiles atuais, além dos cabeçalhos dos vetores.

Esses tempos são provisórios, não um benchmark controlado ou ganho de FPS.
A tentativa de duas rodadas em ordem invertida foi abortada pela guarda de
processos: não houve uma janela livre de Cargo/rustc em 13 minutos. Houve também
um bloqueio transitório de leitura/carregamento Mach-O em várias frentes;
desbloqueou sem ação no sistema. O checkout temporário foi movido ao volume
externo, liberando seus 388 MiB no disco interno. Nenhum cache alheio foi apagado.

Revisão estática: não há JSON/I/O/locks novos no desenho, nem passes/targets
GPU adicionais. Arcs retêm a identidade da fonte; cópias por canto só ocorrem
no corpo/fallback. Memória de amostras é limitada à residência atual e se
retira com o recurso. Campos de tempo são diagnósticos de sessão, sem persistência.

Continuam síncronos o planejamento, a geração de tiles novos, a montagem dos
grupos e os uploads que realmente mudam. O runtime físico também mantém sua
residência local; a física usa a fonte completa. Não há alegação de geração
assíncrona, ausência de travadas, 60 FPS ou aceite humano da nova execução.
