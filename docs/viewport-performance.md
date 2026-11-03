# FPS da paisagem navegável — EAF-171

Pedido: abrir a paisagem sem blur para navegação humana e comparar FPS reais
da janela. A medida anterior era draw CPU + espera pela GPU; 3,60ms contra3,43ms
é regressão de tempo total, não medida isolada de CPU nem FPS de apresentação.

## Contrato e revisão prévia

O editor existente redesenha sob demanda quando pausado e usa AutoVsync. Inverter
uma mediana offscreen ou contar redraws ociosos não mede capacidade interativa.
O modo opt-in **Exibir → Medir FPS continuamente** solicita redesenho mesmo
sem simulação, usando Poll somente durante a medição e com janela não oclusa.
Ao desligar retorna ao agendamento normal. **VSync na medição** escolhe Fifo;
desativado prefere Immediate, depois Mailbox, depois Fifo suportado. O modo
efetivo aparece na barra, portanto uma plataforma com fallback não é anunciada
como desbloqueada. Capacidades da superfície são consultadas uma vez na criação.

Dispatcher/MCP: `set_viewport_performance` recebe
`{"settings":{"enabled":true,"vsync":false}}`;
`inspect_viewport_performance` lê o mesmo diagnóstico da UI. Configuração e
amostras são de sessão no Service, fora do documento/undo/replay/save/rede.
Scene102/replay112/conteúdo4 permanecem. Capturas offscreen não incrementam FPS.

O contador usa os intervalos entre chamadas nativas `present` concluídas, até1s
e512 intervalos, publicando a cada0,5s. Não é o instante físico de scanout nem
timestamp GPU: Mailbox pode descartar frames na apresentação. A janela contabiliza
viewport, UI, aquisição/submissão e escalonamento. `redraw_wall_ms` é tempo de
parede da função de redraw, incluindo esperas; não rotular como CPU pura.
Redimensionar, mudar modo/aplicar configuração ou alternar otimização reinicia
as amostras; oclusão/retomada limpa a amostra. `age_ms` permite reconhecer valores
desatualizados se o host não continuar desenhando. Após fechar a janela o processo
da demo termina, portanto não há fonte de FPS ativa.

Ring512 pré-alocado, sem crescimento indefinido. JSON/estatísticas do renderer
são materializados somente ao publicar ou inspecionar, não por vértice; sem
readback/GPUwait por frame adicionado. A telemetria e render contínuo têm custo
próprio, portanto comparar ambos caminhos com o mesmo modo/resolução. Não
alegar FPS de gameplay ou ganho universal a partir de uma cena pausada.

## Uso

A barra inferior mostra FPS, intervalo médio, resolução física da viewport,
modo de apresentação e checkbox **Otimização**, pelo mesmo `SetLight` existente.
O checkbox muda `reduce_details`; não altera densidade, paleta, geometria fonte
nem blur. A fonte sem compilação foi verificada no ensaio anterior; neste A/B
interativo os pacotes podem permanecer em cache ao desativar a seleção.

- Botão direito + arraste: orbitar.
- Botão do meio, ou Shift + direito + arraste: deslocar câmera/alvo.
- Roda do mouse: aproximar/afastar.
- Exibir → Restaurar câmera da cena: voltar à vista inicial.

São os controles existentes do editor, não um novo controlador WASD/voo.
A demo usa luz da cena (`work_light=null`), área da viewport maximizada,
simulação pausada e blur ausente. A janela NPC anterior é preservada. Resolução
física é a realmente desenhada, limitada pelo alvo atual4096px por dimensão,
não uma resolução presumida da tela. Distância/far-plane não foram ampliados.

## Verificação nativa — 31/10/2026

Build`49bc6414ce1dc4ee`, main, Apple M4Pro/Metal. Janela própria PID71626 aberta
pelo launcher já instalado em `~/Applications/Ignis Editor.app`, que aponta para
`target/release/ignis`; nenhuma cópia histórica adicional. PID46932 da demo NPC
foi preservado. Fonte `artifacts/render-comparison/landscape.scene.json`, cópia de
uso `artifacts/landscape-live/landscape.scene.json`; sem blur, luz da cena, viewport
maximizada. A câmera do editor mudou enquanto a janela estava disponível, sem
comando de câmera do agente, e foi conservada durante as medições.

Interface inspecionada nativamente: paisagem visível, contador e checkbox na
barra inferior. Clique no checkbox pela UI mudou reduce_details e o diagnóstico
confirmou zero instâncias substituídas. APIs de medição não alteraram Scene;
desligar zerou amostras, retomar recomeçou a janela temporal. VSync confirmado
como Fifo a119,69FPS; retomada Immediate a248,89FPS. Sem alegação de scanout a249Hz.

Viewport física2784×1560, câmera `[-1501.84,2789.89,9773.66]`, alvo `[0,1300,0]`,
FOV55°. A/B/B/A sem mover câmera pelo agente;1,5s de aquecimento e5 leituras a
cada0,55s por fase. Janela móvel1s sobrepõe leituras; isso não é amostra independente
nem intervalo estatístico de confiança. Câmera inicial/final idêntica em cada
fase; frame sample confirmou modo, resolução e estado de otimização.

| Caminho | Mediana FPS fase1 | Mediana FPS fase2 |
|---|---:|---:|
| Completo |230,73|234,03|
| Otimizado |255,20|250,60|

Aproximadamente8,8% mais FPS nesta vista (médias das duas medianas), sem blur.
É outra câmera/resolução e outro protocolo, portanto não contradiz nem substitui
as regressões do ensaio offscreen anterior. O contador variou fora das janelas
medidas; capturar um único FPS alto não serve de comparação. O ganho não é garantia
para toda vista, gameplay ou quantidade de assets.

Evidências atuais: `artifacts/landscape-live/setup.json`, `fps-comparison.json`,
`controls-verification.json` e log MCP local. UI/câmera continuam disponíveis
após o trabalho, com otimização/medição ligadas e pedido de VSync desligado.
Build, Clippy all-targets, fmt/diff e8 testes existentes de editor passaram.
Prova de integração nativa cobriu configuração/inspeção, estado autoral intacto,
retorno e modo efetivo; não foram criados testes unitários para cada contador.

Revisão final: oclusão suspende somente a medição; simulação em execução conserva
o agendamento anterior mesmo com janela oculta. Build final`1a5d928e5347642e`,
PID72080, substitui a janela da demo após salvar e restaurar cena/layout/câmera
exatos (simulação pausada). Build/fmt/diff aprovados; inspeção nativa confirmou
Immediate,2784×1560, otimização ativa e zero passes de blur. A câmera voltou a
mudar durante a navegação humana; não tratar o FPS dessa vista como repetição
do A/B acima. Evidência de preservação em `artifacts/landscape-live/final-verification.json`.
