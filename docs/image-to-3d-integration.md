# Base de autoria para imagem em 3D

Continuidade EAF-99: [comparador visual](image-similarity.md) disponível no
MCP; primeiro estudo de volumes da casa em examples/house-blockout.scene.json.
A integração histórica abaixo permanece como contexto, sem afirmar geração
automática ou fidelidade final da casa.

Ferramentas posteriores no cliente, sem alterar esses contratos históricos:
[ajuste de câmera, ledger de revisão e ensaio img2threejs — EAF-189](reconstruction-client-tools.md).

Integração local EAF-98, branch `codex/image-to-3d`, partindo da main local
`f6b707e`. Pontas capturadas: `codex/native-meshes` em `7daee87` e
`codex/native-plants` em `2ba75eb`. As branches/worktrees de origem permanecem
independentes; mudanças posteriores nelas não entram implicitamente aqui.

## Contratos integrados

Cena67 reúne meshes, plantas e geradores de população. As guardas históricas
por capacidade de ambas as frentes são mantidas: o número maior não substitui
a validação dos campos. As cenas autorais antigas continuam passando pelo
mesmo carregamento/validação/migração. Replay71 mantém simultaneamente
`expected_meshes` e `expected_generators`; `matched` exige os dois, além dos
resultados anteriores. Replays de outro build/formato continuam recusados.

São preservados os limites de armazenamento e desenho já ampliados pela
branch de plantas, suas fontes canônicas e composições, os kernels de união
e suavização da branch de malhas e os controles/ondas/ativação espacial da
main. A consulta de estado conserva `mesh_selection` e `generators`.
O runner retém os82cenários MCP, sem duplicatas.

O teste `tests/authoring_integration.rs` combina uma planta curva real com
produção por trigger, testa serialização/prefixo/replay e adulteração
independente dos resultados das duas frentes. Os testes de recusa de versões
antigas por capacidade permanecem separados das expectativas de versão
corrente atualizadas para67/71.

## Referência para o debate

[Casa élfica — imagem original](references/image-to-3d/elven-house.png), cópia
sem alteração da imagem fornecida por Guilherme. Será o objeto de teste da
funcionalidade de autoria a partir de imagem. Nenhuma reconstrução, decisão
de representação da casa ou ferramenta nova de interpretação de imagem foi
implementada nesta integração.

## Validação e build corrente

Validação sem janela no macOS; logs estáveis em `artifacts/integration/` e
relatório MCP em `artifacts/mcp-validation/report.json`. Build exclusivo desta
worktree em `/Volumes/Zero/dev/games/.ignis-build/image-to-3d`, usando
`CARGO_TARGET_DIR` para evitar ocupar o disco interno com compilação adicional.
Nenhum build, artefato ou sessão das outras worktrees é substituído.

Resultados finais registrados em `docs/validation.md` após a execução.
Checklist proporcional em `docs/performance.md`. Sem push/publicação.
