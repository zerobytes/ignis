# Navegação derivada da geometria — programa EAF-149

Antes de implementar: Scene91/replay101 após90/100, protocolo2 preservado. Receita autoral opcional define entidade/template de referência, região XYZ, cell_size, connection_radius, seed, include_air e orçamento de passos de admissão. Escolha de grade/superfícies amostradas reutiliza a rede64nós/128links em vez de introduzir navmesh/dependência/representação paralela. Descobre ligações a partir da colisão física Box/Wedge, não da aparência de meshes.

## Semântica

Gerar pontos de piso por coluna XZ sobre superfícies fixas na região; acomodar a caixa na inclinação e validar contato/volume com o mesmo KCC. Posições são pés nominais, skin aplicado só na consulta. Voo opcional acrescenta posições livres XYZ e ligações permitidas pelas capacidades. Aproximar vizinhos em raio explícito, validar corpo/trajetória/apoio por modo e publicar limites/truncamento. Não assumir conectividade nem confundir ausência de ligação com prova de impossibilidade.

A receita é persistida; normalizar após edição/reset/load regenera quando colisores, perfil físico ou parâmetros mudam. Cache de assinatura derivado não é aceito de arquivos nem participa do protocolo. Edição manual set_spatial_network desliga a receita explicitamente. Geometria imóvel conhecida constitui mapa autorado, nunca observação de alvo; perfis mentais individuais continuam limitando consulta/planejamento/preferência, sem modificar o kernel físico.

Autoria/inspeção/preview pelo dispatcher/MCP; erros preservam cena e histórico, confirmação gera um undo. Ator/template de referência precisa controlador+walk+local_connections; ausência de apoio produz relatório vazio válido, não corpo teletransportado. Impossibilidade dinâmica e descoberta parcial do mundo seguem distintas do mapa estático conhecido.

## Revisão estática prévia

1. Geração só na autoria/normalização com fonte alterada; nenhuma geração por tick/ator.
2. Um World fixo de queries, buffers de nós64/links128, até4032pares transitórios em uma Vec de autoria, sem Scene clone por sonda. Cache ignora alterações visuais/posição de atores.
3. Nós/índices/tipos existentes, assinatura FNV de dados físicos/receita, sem formato binário/protocolo/dependência novos.
4. Reutilizar validador KCC e motores aéreos; recompilar Runtime após edição como antes. Sem terceiro solver.
5. Até64colunas,64nós,128links; teto de pares/sondas e passos total declarado, report limited. Nunca retry infinito nem aumentar caps para caber.
6. GPU/renderer inalterados; preview/JSON sob comando. Uploads/render só refletem diagnóstico existente.
7. Mesmo dispatcher/PRNG local derivado da seed, sem locks/workers ou conhecimento oculto.
8. Receita validada por versão/referência, cache derivado fora de arquivos; undo/reset/load restauram fonte e regeneram determinístico.
9. Prova nova somente atomicidade/invalidação/replay/ausência de alvo oculto, cobertura existente por impacto.
10. Região/cell_size/capacidade limitam cobertura, grafo de amostras não é navmesh global ou garantia de circulação de multidões. Budgets e ausência de FPS/escala explícitos.

Cache usa assinatura da geometria/perfil/receita e do grafo materializado; não é autenticador nem aceita cache serializado. Até8alturas distintas por coluna,64colunas/512amostras aéreas/64nós/128links,64..1024sondagens (padrão512) e600..76800passosKCC (padrão24000). Pares ordenados por distância/índices; escolher primeira modalidade fisicamente admitida walk/jump/glide/fly. Custos e amostras auxiliares dos motores aéreos permanecem limitados pelos seus validadores, não são passosKCC. Perfil mental atua na consulta/seleção durante gameplay; geometria física não é alterada por inteligência.
