# Pincéis de forma e acabamento

Incremento da escultura autorizada em EAF-110. Além de Volume/Mover/Suavizar,
implementa seis operações geométricas reais, cada uma com força própria:
Achatar, Pinçar, Vinco, Raspar, Preencher e Relaxar. Não são presets de Volume.
Máscaras, simetria, remesh e pressão de tablet permanecem incrementos separados.

Atualização30/10: o editor usa `Edit::SculptStroke` para Volume/Mover. Força de
Volume é percentual, altura máxima por gesto=raio×força×0,5, sem multiplicar
relevo pela sobreposição de amostras. Detalhe automático usa o orçamento disponível
e continua pintando ao saturar; Mover limita deslocamento antes da primeira dobra.
O painel informa limites e raio efetivo sem descartar o gesto. Layouts antigos
migram m/amostra uma vez, preservando altura de um carimbo no raio antigo.
Sculpt/SculptDetailed estritos continuam disponíveis para autoria explícita.

Atualização anterior: Volume pode refinar localmente a topologia antes de pintar,
com **Detalhar sob o pincel**, ligado por padrão e persistido nos layouts.
`Edit::SculptDetailed` é o contrato compartilhado, com referência escopada,
`stroke` e `max_edge`; detalhe e traço têm um desfazer. Os demais pincéis e
`Edit::Sculpt` conservam a topologia. Limites e provas das diagonais estão em
[editor-sculpt.md](editor-sculpt.md).

## Contrato deste incremento

O Stroke existente continua limitado a128 centros, com raio em metros locais.
Draw mantém a unidade em metros por amostra. A correção posterior em
[editor-sculpt.md](editor-sculpt.md) usa normal média da área por carimbo e
recusa dobramento de faces em Draw/Mover. As operações novas usam força adimensional:

- Achatar aproxima ambos os lados do plano médio da superfície sob o pincel.
- Pinçar aproxima tangencialmente do centro; força negativa espalha.
- Vinco combina concentração tangencial e depressão normal; negativo cria crista.
- Raspar remove somente acima do plano; negativo preenche abaixo.
- Preencher eleva somente abaixo do plano; negativo raspa acima.
- Relaxar usa a média dos vizinhos projetada no plano tangente, preservando o relevo normal melhor que a suavização comum; sem inversão.

Normais, vizinhança e planos vêm da base congelada do gesto. Influência acumulada
com saturação evita perseguir a própria deformação, depende da distância entre
amostras em vez da cadência de frames e não muda ao inverter a ordem do traço.
Preview e confirmação reavaliam o mesmo Stroke sobre a mesma base. Erros
preservam o documento, a topologia, materiais, rigs, fontes e histórico.

Preferências adicionais têm defaults de desserialização: layouts3 anteriores
continuam abrindo, sem alterar Scene86/replay90/projeto1 ou dependências. A
expansão de BrushKind existe simultaneamente no schema MCP e na UI. Não inferir
compatibilidade de arquivos novos com executáveis antigos.

## Checklist antes de implementar

1. Kernel somente na autoria e somente quando chegam amostras; sem trabalho no tick.
2. Normais/vizinhos temporários limitados por vértices, planos limitados128; sem cópia por par vértice/amostra.
3. Enums e vetores tipados, f64 apenas para soma estável, saída f32 existente.
4. Reutilizar base congelada; UI reavalia Stroke completo para coincidir com commit, sem algoritmos aproximados distintos.
5. Mesmos tetos de amostras/vértices/triângulos; sem remesh implícito ou crescimento de topologia.
6. Sem nova ABI GPU/readback/textura/I/O; renderer recebe geometria da prévia existente.
7. Mesmo dispatcher sequencial e undo único; sem locks/threads/dependências.
8. Cancelamento/revisão/picking congelados existentes preservados; preferências por pincel explícitas.
9. Testar diferenças entre kernels, inversão, força/raio, regiões não afetadas, caminho invertido, preview/commit/undo/save/load/replay e UI real.
10. O(amostras×vértices+triângulos) por avaliação completa é custo consciente e limitado; sem promessa de FPS, geometria adaptativa ou tablet.

## Implementação e uso

Na Escultura, a faixa lateral apresenta os nove pincéis com desenho e nome;
atalhos1..9 seguem a ordem Volume, Mover, Suavizar, Achatar, Pinçar, Vinco,
Raspar, Preencher, Relaxar. O inspetor mostra a operação, raio em metros e força
própria. Volume conserva metros/amostra; os demais mostram porcentagem. Ctrl
inverte apenas operações que têm inverso; Shift inicia Suavizar temporário.
Campos continuam com arraste horizontal, ajuste fino e digitação.

Os seis novos kernels estão em src/mesh_sculpt_shapes.rs e são chamados pelo
mesmo Edit::Sculpt usado pela UI, `preview_mesh_edit` e `edit_mesh` MCP. Somam
influências em f64 antes de publicar f32; alpha=1−exp(−|força|×peso) limitado0.95.
A verificação de orientação das faces recusa colapso/inversão antes de publicar
as posições; não tenta reparar a topologia ou desvincular fontes automaticamente.
Para Relaxar, normais e média de vizinhos vêm do início do gesto, e a componente
normal do deslocamento é removida. Normais ponderadas por área e plano médio
por amostra são escolhas explícitas; não prometem equivalência numérica aos
pincéis do Blender ou suporte de tablet.

## Revisão estática do código

1. Kernel só na autoria; redraw sem evento não carimba amostras. 2. Dois buffers
normais/posições limitados por16384v; vizinhos/contagens alocados somente para
Relaxar, planos128 no stack. Sem clone adicional de triângulos; rollback troca
somente o vetor de posições. 3. BrushKind tipado, preferências f32, somas f64,
sem JSON/reflexão no kernel. 4. Cada avaliação congela geometria inteira, UI
reavalia todos os centros para coincidência com a confirmação; nenhum passo
persegue a deformação anterior. 5. Mesmos limites128/16384/triângulos e sem
crescimento geométrico. 6. Renderer/ABI de vértices mantidos, sem I/O/GPU novo
no kernel. 7. Nenhuma thread/lock novo. 8. Cancelamento/revisões/picking/undo
existentes preservados e campos novos com defaults de desserialização.
9. Cinco testes de integração novos exercitam seis kernels distintos, influência
local, sinal, força/raio, inversão do percurso, orientação de faces, preview/
commit/undo e preferências antigas/independentes; testes de ponteiro existentes
agora percorrem os nove pincéis. 10. Planos ponderados custam O(amostras×vértices)
e cópia de posições por avaliação; custo de autoria limitado e registrado,
sem alegação de FPS ou necessidade de benchmark como condição da entrega.

Validação final, capturas GPU e limites de aceite são registrados em
artifacts/editor-authoring-validation/ e docs/validation.md. Nove pincéis não
concluem toda a modelagem/escultura: máscaras, simetria, remesh, pressão,
extrusão/inset/bevel genéricos e edição avançada de rig continuam frentes próprias.
