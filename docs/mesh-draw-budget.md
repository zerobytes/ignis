# Orçamento de desenho de meshes — avaliação antes do incremento

A árvore expôs uma escolha inadequada de autoria:12tri por folha mantém2100
folhas no teto32768t, mas o close exibe polígonos largos e pontas retas. O
armazenamento de variantes compartilhadas não é o gargalo; é a expansão por
instância na CPU. Aumentar apenas o número de folhas não corrigiria a forma.

Alternativas: trocar densidade por detalhe mantendo32Kt (piora os vazios já
observados); LOD/distância/instancingGPU (melhor caminho de escala, mas exige
novos contratos de erro/captura/seleção e não cabe num ajuste de conteúdo);
subir deliberadamente o teto de desenho sem mudar limites autorais.

Recorte escolhido para o próximo teste:65536tri desenhados, mesmos16assets/
32instâncias/4096v por asset/16384v e32768t armazenados. Sem remover preflight,
sem alocação ilimitada, sem prometer escala de floresta/FPS. Até196608 vértices
expandidos ×52bytes =9,75MiB CPU ativos no teto, contra4,875MiB anterior; Vec pode reter capacidade13MiB. O buffer
GPU arredondado a potência de2 pode chegar262144v=13MiB para a faixa de meshes,
contra131072v=6,5MiB. Capacidade total de render também inclui caixas/guias;
nenhuma multiplicação dos arrays de skinning/normais por instância.

CPU conserva skinning/normais/expansão por instância e upload em toda pose
alterada. Sombra redesenha a mesma superfície; celular pode custar mais por
fragmento que a geometria. Esta revisão estática aceita esse custo limitado
para o laboratório, não é uma medição de tempo nem valida um renderer de produção.

Cena55/replay59 devem reconhecer documentos acima do antigo teto; versões
anteriores não podem ocultar esse uso. Testar exato65536,65537 recusado, erro
atômico/undo e estado de replay; manter verificação de armazenamento separada.
Ensaio artístico:1650folhas32t com contorno8estações e curvatura sutil, manter
as vistas fixas e registrar se reduzir número de folhas piora a copa.

## Revisão posterior da implementação

1. Frequência: orçamento é verificado na autoria/validação, não em cada tick.
2. Alocação: mesmos buffers/Vec reaproveitados; capacidade maior somente ao
   receber mais geometria; nenhum array de skinning/normais cresce.
3. Tipos: índices u32 e contagens usize existentes; sem ABI nova.
4. Repetição: versões antigas fazem scan adicional de até32 instâncias para
   guardar o teto legado. Edição corrente não precisa desse scan.
5. Limites:65536 de desenho; armazenamento/assets/instâncias/rig/clipes mantidos.
6. GPU/I/O: mesmo passe/buffer/potência de2; captura explícita de196608v.
7. Sincronização: dispatcher/histórico existentes, sem thread/lock nova.
8. Invalidação: crescimento/remoção usam cache atual; versão55/59 explícita.
9. Correção: teste Rust de65536/65537, atomicidade/undo/legado e cenário meshes
   com captura GPU/replay byte a byte. Resultados em validation.md.
10. Custo aceito: até2× geometria expandida/upload/desenho e capacidade retida
    maior; não constitui instancing GPU/LOD nem medição de performance.
