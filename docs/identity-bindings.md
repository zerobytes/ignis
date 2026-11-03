# Vínculos persistentes de identidade — EAF-61

Continuidade EAF-63: a fronteira de presença/configuração e a retenção compartilhada de origem foram implementadas conforme [runtime-world.md](runtime-world.md), com cena46/replay49. Spawn/despawn público, instalação dinâmica dos componentes e ampliação da população continuam pendentes. As propostas/limites abaixo devem ser lidos junto desse contrato posterior.


Continuação de [EAF-60](instance-identity.md), preparando população dinâmica. Cena46 permanece; replay48 registra os novos stamps em percepção, buffs e históricos. O catálogo público ainda é fixo: reciclagem real do registro é usada somente por testes isolados, sem comando de spawn/despawn.

## Vínculo e acesso

`Reference` contém slot, geração e serial do indivíduo; índices de ator/entidade continuam aceleradores, conferidos antes de consumir um vínculo persistente. `life_index` continua distinguindo ressurreições do mesmo indivíduo. Não há parsing de token ou JSON por pulso. A inspeção converte o serial para hexadecimal de16 caracteres, compatível com `inspect_instances`.

Buffs conservam referências de dono e origem. Refresh exige mesma definição, dono e origem completos; reutilizar o slot de qualquer um não renova o buff antigo. Consulta e dissipação só consideram dono correspondente. Limpeza encerra vínculo de dono ausente com `owner_removed`, sem consultar a vida do substituto. Um buff cujo autor desapareça pode permanecer no destinatário original até seu prazo; sua contribuição é independente do slot atual do autor. O responsável por refresh/dispel também é congelado no recibo.

Periódicos com vínculo inválido encerram com `instance_changed`; áreas com origem inválida encerram com `source_removed`. Estes são guardas internos da migração. **Não são a política final de despawn:** retenção de contexto de origem e sua reserva antecipada ainda precisam integrar o mundo vivo para entregas sobreviventes. Projétil não ignora a colisão com novo ocupante do slot do autor; efeito continua sujeito à validação da origem antes do commit.

## Conhecimento do NPC

Observações visuais guardam indivíduo e vida. Reconhecimento reinicia ao mudar qualquer um, mesmo se slot e vida numérica coincidirem. Memória antiga conserva posição e data, mas não fornece um ator vivo ou exclusão de colisão do novo ocupante. Ausência oculta não dispara esquecimento global nem revela uma posição nova.

Conhecimento de morte/vida só se aplica ao indivíduo observado. Rádio congela identidade do remetente e do sujeito; referências inválidas são rejeitadas antes de relações, vida e ranking. Um recibo anterior conserva a autoria histórica. Som permanece anônimo para a decisão do NPC; a referência do emissor serve para autoexclusão, evitando que outro indivíduo no mesmo slot ignore um ruído que não produziu.

## Inspeção e históricos

Os comandos MCP existentes expõem os campos adicionais, sem controle exclusivamente visual:

| Registro | Identidade preservada |
|---|---|
| Buff / contribuição / remoção | dono, autor e responsável |
| Memória / reconhecimento / rádio | sujeito, vida e remetente |
| Habilidade / efeito / interrupção | autor, destinatário e interruptor |
| Projétil / área / periódico | origem, contato/ocupantes/destinatário e remoção |
| Morte / ressurreição | indivíduo, origem e vida |
| Ruído / contato / trigger | emissor ou participantes |

Nos históricos de combate e percepção, aliases como `actor`, `source` e `target` são resolvidos somente se a referência original ainda existir; caso contrário retornam null, mantendo `actor_instance`, `source_instance`, `target_instance` ou o stamp correspondente. Contatos já congelavam nomes no evento e agora também conservam referências. Triggers resolvem nomes pelo binding compilado original, podendo mantê-los após a saída; nunca pelo novo ocupante. Conservam esse binding: a remoção do alvo provoca saída do indivíduo original, sem adotar automaticamente quem reutilize o slot. Rebinding de seletores no spawn é integração posterior.

Recibos de habilidade congelam o índice da definição no momento do registro, evitando reinterpretar o histórico pelo grant corrente. Definições de cena continuam imutáveis durante a execução; editar recompõe a simulação. Os anéis existentes e seus contadores de descarte permanecem limitados. Não foi criado um arquivo histórico ilimitado nem uma cópia de Scene por evento.

Inspeção de periódico cujo autor não resolve retorna `dispellable:null`, pois não há contexto de origem retido para avaliar sua proteção. Esse caso só é alcançável nos testes internos de reciclagem nesta etapa. Identidade serializada internamente conserva campos numéricos para replay; IDs diagnósticos em hexadecimal não são novos tokens de comando.

## Fronteiras restantes

A visão do mundo presente, instalação/remoção de componentes físicos, catálogo de definições/grants por instância, bindings autorados e apresentação ainda precisam compor a operação transacional de população. Estados transitórios por slot são recalculados na simulação atual; não se deve liberar reciclagem apenas porque os vínculos persistentes deste recorte estão protegidos.

O limite segue9 atores. Consultas de atributos por dono/chave, planos por campos tocados e reserva de contexto de origem na admissão das entregas seguem requisitos de evolução de escala. Nenhum aumento de população, throughput de memória ou ganho de FPS foi demonstrado neste incremento.

## Validação

`identity_bindings_tests` recicla slots do registro real em uma interface exclusiva de testes, cobrindo buff/dissipação/refresh/limpeza, reconhecimento/pista, rádio, anonimato, periódico, histórico vital e trigger. A substituição de componentes do mundo não é simulada como se fosse spawn público.

`python3 scripts/run_verifications.py --only identity_bindings` usa o dispatcher MCP para buffs, rádio, pulsos, morte/ressurreição, replay completo/prefixo e igualdade de captura GPU. A regressão completa permanece headless. Resultados em [validation.md](validation.md); revisão antes/depois em [performance.md](performance.md). Nenhuma nova demo ou interação de teclado é necessária para esse contrato de infraestrutura.
