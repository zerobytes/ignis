# Mundo contínuo e saves portáteis — direção de produto

Direção de Guilherme em 30/10/2026 após NPC v1: a Ignis deve suportar mundos procedurais contínuos, como Land of Survivors, sem fronteiras de cena impostas ao jogador. Separação em fases/cenas visíveis é uma escolha explícita do jogo. Este documento registra requisitos e proposta arquitetural; não implementa streaming nem save portátil.

## Diferença entre roaming e alcance de navegação

Roaming é a política que escolhe destinos; navegação encontra e executa o caminho até eles. Um NPC já pode patrulhar continuamente em XYZ, mas o runtime atual não consegue transportar esse comportamento através de um mundo procedural ilimitado. O grafo global pequeno pode não representar o caminho necessário; não descobrir uma ligação não prova impossibilidade física.

A limitação atual inclui rede de 64 nós/128 ligações, Scene global com limites de entidades/atores e validadores com coordenadas locais em +/-10000. Não é apenas aumentar MAX_NODES. O fechamento EAF-149 descreve a integração local do NPC v1, não suporte implementado a mundos contínuos de extensão arbitrária.

## Requisito de mundo

- Mundo lógico separado do conjunto atualmente carregado. Regiões/chunks são unidades internas de armazenamento/trabalho, não fronteiras obrigatórias de gameplay.
- Geração determinística por seed/versão/região e registro das alterações persistentes; a definição procedural não exige materializar o mundo inteiro.
- Carregar e descarregar conteúdo, colisão, navegação e apresentação de forma coordenada, com estado/identidade preservados e nenhuma perda por um simples unload.
- Navegação hierárquica: rotas entre regiões e caminhos locais refinados conforme necessário; capacidade e conectividade reais continuam obrigatórias. Roaming nunca implica onisciência.
- NPCs fora das regiões ativas precisam de política explícita: atividade completa, simulação simplificada ou suspensão. Suspensão não pode se passar por viagem física simulada.
- Coordenadas globais de grande alcance e física local com precisão controlada. Alternativas incluem célula inteira + posição local e rebasing/origens por região; representação e modelo de múltiplas regiões ativas exigem decisão técnica própria.
- Budgets limitam memória/trabalho residente e tempo de consulta, não o tamanho lógico do mundo. Multijogador pode manter regiões distantes ativas ao mesmo tempo.
- Mesmo contrato funciona local/offline; servidor opcional. MCP consulta mundo lógico, conteúdo carregado e motivos de limitação sem confundir desconhecido, não carregado e inalcançável.

“Infinito” aqui é extensão procedural conforme a exploração, sem borda artificial de cena, dentro dos limites numéricos/armazenamento declarados; não memória infinita ou todos os NPCs simulados integralmente ao mesmo tempo. Ainda não implementado.

## Save próprio e portável

O checkpoint atual já é um formato da Ignis (envelope/versão/CBOR), mas seu payload incorpora um snapshot interno do Rapier. Mudar nome/extensão/codec não o torna portátil.

Direção proposta: definir estado canônico versionado da Ignis, independente de backend/plataforma. Armazenar identidade, recursos, buffs, casts, cooldowns, memória, rotinas, diálogo, relações, posição/orientação/velocidade e fase do movimento em tipos próprios. Para mundo procedural, incluir seed/versão do gerador, alterações de regiões e atores persistentes, sem gerar o mundo todo para salvar.

Cada plataforma lê esse estado e recompõe corpos, colisores e caches pelo adapter físico. Migrações explícitas versionam o contrato de jogo, geradores e dados; alterações de física também podem exigir reconciliação semântica. Um arquivo canônico pode manter formato binário eficiente e codec por campo; independência não exige JSON nem conversão de estruturas arbitrárias.

Tradeoff a decidir: estado/progresso portáteis não garantem contatos internos ou trajetória futura bit a bit idênticos entre backends/arquiteturas. Definir tolerâncias e retomada de contato/apoio/casts/voo sem teleporte arbitrário nem custo repetido. Snapshot nativo opcional pode ser um acelerador de retomada exata no ambiente compatível, nunca a única fonte do save portátil.

Nenhuma garantia de migração/plataforma é adicionada ao checkpoint 1 por este documento. A implementação atual continua recusando física/OS/arquitetura incompatíveis.
