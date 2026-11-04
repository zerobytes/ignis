# Catálogo de habilidades de Land of Survivors

Anexo do [GDD](GDD.md). Preserva as 284 habilidades como design, com nomes em inglês e português do Brasil, efeitos, focos e valores de referência. Não contém programas, assets, efeitos visuais ou implementação.

Os números são balanceamento provisório. Distâncias e velocidades foram expressas em metros e metros por segundo; tempos em segundos. Os demais valores conservam a escala original do design. Campos como lentidão, mitigação e execução ficam na escala herdada, sem inventar uma porcentagem ou regra ausente da descrição. Parâmetros que não afetam determinada modalidade continuam apenas como referência de sua ficha.

O [runtime funcional no Ignis](habilidades-runtime.md) transforma estas entradas em conteúdo executável. As interpretações necessárias para regras incompletas estão documentadas separadamente como balanceamento provisório, sem reescrever o catálogo herdado.

## Bases e crescimento das classes

Ordem dos atributos: Força, Destreza, Vigor, Inteligência e Espírito. A base da espécie e os pontos livres se somam a esses valores de classe.

| Classe | Base | Crescimento por nível |
| --- | --- | --- |
| Mago / Mage | 0, 1, 1, 5, 3 | 1, 1, 2, 3, 2 |
| Guerreiro / Warrior | 5, 1, 4, 0, 0 | 3, 2, 3, 1, 1 |
| Arqueiro / Archer | 1, 5, 2, 1, 1 | 2, 3, 2, 1, 2 |
| Druida / Druid | 1, 1, 3, 1, 4 | 1, 2, 2, 2, 3 |

## Organização do catálogo

Seis especializações contêm 44 habilidades humanoides cada. Oito compartilhadas mais doze exclusivas de cada foco permitem vinte opções por foco. Urso e Felino têm dez habilidades próprias cada. Nomes e descrições em inglês permanecem a fonte textual de produto; as traduções brasileiras acompanham cada entrada.

## Mago e Fogo

### Fireball / Bola de Fogo

ID de conteúdo: `los.mage.fire.fireball`.

Modalidade: Projétil. Focos: DPS, Healer, Tank.

Conjure for 2 seconds, then launch a large fireball. On collision, deal 600 base damage to every enemy within 3 m and ignite the ground for 8 seconds, dealing 10 base damage each second. Allies take no damage. 20-second cooldown.

Conjura por 2 segundos e lança uma grande bola de fogo. Ao colidir, causa 600 de dano base a cada inimigo em 3 m e incendeia o chão por 8 segundos, causando 10 de dano base por segundo. Aliados não sofrem dano. Recarga de 20 segundos.

Custo: 15; Recarga (s): 20; Alcance (m): 30; Raio (m): 3; Dano base: 600; Conjuração (s): 2; Duração (s): 8; Intervalo (s): 1; Dano periódico: 10; Velocidade (m/s): 16; Diâmetro do projétil (m): 0.6.

### Circle of Fire / Círculo de Fogo

ID de conteúdo: `los.mage.fire.fire_circle`.

Modalidade: Zona persistente. Focos: DPS, Healer, Tank.

11 base damage; 14 base healing; effect duration 8s; values per pulse, every 1s; heals/protects allies and damages/controls enemies. Cast time 0.6s.

11 de dano base; 14 de cura base; efeito durante 8s; valores por pulso, a cada 1s; cura/protege aliados e causa dano/controle a inimigos. Conjuração de 0.6s.

Custo: 32; Recarga (s): 16; Alcance (m): 14; Raio (m): 3.4; Dano base: 11; Cura base: 14; Conjuração (s): 0.6; Duração (s): 8; Intervalo (s): 1; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Flame Step / Passo de Chamas

ID de conteúdo: `los.mage.fire.flame_step`.

Modalidade: Avanço. Focos: DPS, Healer, Tank.

24 base damage; swept forward movement; affects enemies.

24 de dano base; avanço com colisão; afeta inimigos.

Custo: 12; Recarga (s): 10; Alcance (m): 4.2; Raio (m): 1.5; Dano base: 24; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Ember Ward / Proteção de Brasas

ID de conteúdo: `los.mage.fire.ember_ward`.

Modalidade: Próprio personagem. Focos: DPS, Healer, Tank.

70 base shield; effect duration 8s; affects allies.

70 de escudo base; efeito durante 8s; afeta aliados.

Custo: 12; Recarga (s): 16; Alcance (m): 14; Raio (m): 0; Escudo: 70; Conjuração (s): 0; Duração (s): 8; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Searing Chains / Correntes Ardentes

ID de conteúdo: `los.mage.fire.searing_chains`.

Modalidade: Alvo. Focos: DPS, Healer, Tank.

20 base damage; roots enemies for 2s; affects enemies.

20 de dano base; imobiliza inimigos por 2s; afeta inimigos.

Custo: 12; Recarga (s): 13; Alcance (m): 14; Raio (m): 2.6; Dano base: 20; Imobilização: 2; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Heat Wave / Onda de Calor

ID de conteúdo: `los.mage.fire.heat_wave`.

Modalidade: Cone. Focos: DPS, Healer, Tank.

32 base damage; affects enemies.

32 de dano base; afeta inimigos.

Custo: 12; Recarga (s): 10; Alcance (m): 6.5; Raio (m): 5; Dano base: 32; Empurrão: 320; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Purifying Flame / Chama Purificadora

ID de conteúdo: `los.mage.fire.purifying_flame`.

Modalidade: Alvo. Focos: DPS, Healer, Tank.

25 base healing; removes movement impairments; affects allies.

25 de cura base; remove impedimentos de movimento; afeta aliados.

Custo: 12; Recarga (s): 14; Alcance (m): 14; Raio (m): 2.6; Cura base: 25; Limpeza: True; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Phoenix Breath / Sopro da Fênix

ID de conteúdo: `los.mage.fire.phoenix_breath`.

Modalidade: Cone. Focos: DPS, Healer, Tank.

25 base damage; 34 base healing; heals/protects allies and damages/controls enemies.

25 de dano base; 34 de cura base; cura/protege aliados e causa dano/controle a inimigos.

Custo: 22; Recarga (s): 12; Alcance (m): 6; Raio (m): 2.6; Dano base: 25; Cura base: 34; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

## Mago e Gelo

### Ice Bolt / Seta de Gelo

ID de conteúdo: `los.mage.frost.ice_bolt`.

Modalidade: Projétil. Focos: DPS, Healer, Tank.

42 base damage; effect duration 3s; slows enemies 30%; affects enemies. Cast time 0.85s.

42 de dano base; efeito durante 3s; reduz movimento inimigo em 30%; afeta inimigos. Conjuração de 0.85s.

Custo: 12; Recarga (s): 2; Alcance (m): 14; Raio (m): 0; Dano base: 42; Lentidão: 0.3; Conjuração (s): 0.85; Duração (s): 3; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Frozen Ground / Congelar no Chão

ID de conteúdo: `los.mage.frost.frozen_ground`.

Modalidade: Ponto no chão. Focos: DPS, Healer, Tank.

12 base damage; roots enemies for 3s; affects enemies. Cast time 0.6s.

12 de dano base; imobiliza inimigos por 3s; afeta inimigos. Conjuração de 0.6s.

Custo: 12; Recarga (s): 15; Alcance (m): 14; Raio (m): 3.2; Dano base: 12; Imobilização: 3; Conjuração (s): 0.6; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Frost Step / Passo Glacial

ID de conteúdo: `los.mage.frost.frost_step`.

Modalidade: Avanço. Focos: DPS, Healer, Tank.

effect duration 3s; slows enemies 40%; swept forward movement; affects enemies.

efeito durante 3s; reduz movimento inimigo em 40%; avanço com colisão; afeta inimigos.

Custo: 12; Recarga (s): 10; Alcance (m): 4; Raio (m): 2.6; Lentidão: 0.4; Conjuração (s): 0; Duração (s): 3; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Crystal Ward / Proteção de Cristal

ID de conteúdo: `los.mage.frost.crystal_ward`.

Modalidade: Alvo. Focos: DPS, Healer, Tank.

65 base shield; effect duration 8s; affects allies.

65 de escudo base; efeito durante 8s; afeta aliados.

Custo: 12; Recarga (s): 16; Alcance (m): 14; Raio (m): 2.6; Escudo: 65; Conjuração (s): 0; Duração (s): 8; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Restoring Snow / Neve Restauradora

ID de conteúdo: `los.mage.frost.restoring_snow`.

Modalidade: Zona persistente. Focos: DPS, Healer, Tank.

6 base damage; 12 base healing; effect duration 8s; values per pulse, every 1s; heals/protects allies and damages/controls enemies. Cast time 0.6s.

6 de dano base; 12 de cura base; efeito durante 8s; valores por pulso, a cada 1s; cura/protege aliados e causa dano/controle a inimigos. Conjuração de 0.6s.

Custo: 12; Recarga (s): 17; Alcance (m): 14; Raio (m): 2.6; Dano base: 6; Cura base: 12; Conjuração (s): 0.6; Duração (s): 8; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Cold Front / Frente Fria

ID de conteúdo: `los.mage.frost.cold_front`.

Modalidade: Cone. Focos: DPS, Healer, Tank.

25 base damage; effect duration 3s; values per pulse, every 1s; slows enemies 50%; affects enemies.

25 de dano base; efeito durante 3s; valores por pulso, a cada 1s; reduz movimento inimigo em 50%; afeta inimigos.

Custo: 12; Recarga (s): 10; Alcance (m): 6.5; Raio (m): 2.6; Dano base: 25; Lentidão: 0.5; Conjuração (s): 0; Duração (s): 3; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Thaw / Degelo

ID de conteúdo: `los.mage.frost.thaw`.

Modalidade: Alvo. Focos: DPS, Healer, Tank.

30 base healing; removes movement impairments; affects allies.

30 de cura base; remove impedimentos de movimento; afeta aliados.

Custo: 12; Recarga (s): 13; Alcance (m): 14; Raio (m): 2.6; Cura base: 30; Limpeza: True; Conjuração (s): 0; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Hail / Granizo

ID de conteúdo: `los.mage.frost.hail`.

Modalidade: Ponto no chão. Focos: DPS, Healer, Tank.

44 base damage; affects enemies. Cast time 0.6s.

44 de dano base; afeta inimigos. Conjuração de 0.6s.

Custo: 12; Recarga (s): 8; Alcance (m): 14; Raio (m): 3; Dano base: 44; Conjuração (s): 0.6; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

## Mago e Éter

### Aether Bolt / Seta de Éter

ID de conteúdo: `los.mage.aether.aether_bolt`.

Modalidade: Projétil. Focos: DPS, Healer, Tank.

48 base damage; affects enemies. Cast time 0.85s.

48 de dano base; afeta inimigos. Conjuração de 0.85s.

Custo: 12; Recarga (s): 2; Alcance (m): 14; Raio (m): 0; Dano base: 48; Conjuração (s): 0.85; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Blink / Lampejo

ID de conteúdo: `los.mage.aether.blink`.

Modalidade: Avanço. Focos: DPS, Healer, Tank.

swept forward movement.

avanço com colisão.

Custo: 12; Recarga (s): 10; Alcance (m): 5; Raio (m): 2.6; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Aether Field / Campo de Éter

ID de conteúdo: `los.mage.aether.aether_field`.

Modalidade: Zona persistente. Focos: DPS, Healer, Tank.

9 base damage; 13 base healing; effect duration 8s; values per pulse, every 1s; heals/protects allies and damages/controls enemies. Cast time 0.6s.

9 de dano base; 13 de cura base; efeito durante 8s; valores por pulso, a cada 1s; cura/protege aliados e causa dano/controle a inimigos. Conjuração de 0.6s.

Custo: 12; Recarga (s): 16; Alcance (m): 14; Raio (m): 2.6; Dano base: 9; Cura base: 13; Conjuração (s): 0.6; Duração (s): 8; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Phase Ward / Proteção de Fase

ID de conteúdo: `los.mage.aether.phase_ward`.

Modalidade: Alvo. Focos: DPS, Healer, Tank.

60 base shield; effect duration 8s; affects allies.

60 de escudo base; efeito durante 8s; afeta aliados.

Custo: 12; Recarga (s): 14; Alcance (m): 14; Raio (m): 2.6; Escudo: 60; Conjuração (s): 0; Duração (s): 8; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Spatial Snare / Laço Espacial

ID de conteúdo: `los.mage.aether.spatial_snare`.

Modalidade: Alvo. Focos: DPS, Healer, Tank.

18 base damage; roots enemies for 2.5s; affects enemies.

18 de dano base; imobiliza inimigos por 2.5s; afeta inimigos.

Custo: 12; Recarga (s): 13; Alcance (m): 14; Raio (m): 2.6; Dano base: 18; Imobilização: 2.5; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Unravel / Desatar

ID de conteúdo: `los.mage.aether.unravel`.

Modalidade: Alvo. Focos: DPS, Healer, Tank.

28 base healing; removes movement impairments; affects allies.

28 de cura base; remove impedimentos de movimento; afeta aliados.

Custo: 12; Recarga (s): 12; Alcance (m): 14; Raio (m): 2.6; Cura base: 28; Limpeza: True; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Aether Wave / Onda Etérea

ID de conteúdo: `los.mage.aether.aether_wave`.

Modalidade: Cone. Focos: DPS, Healer, Tank.

28 base damage; 25 base healing; heals/protects allies and damages/controls enemies.

28 de dano base; 25 de cura base; cura/protege aliados e causa dano/controle a inimigos.

Custo: 12; Recarga (s): 10; Alcance (m): 7; Raio (m): 2.6; Dano base: 28; Cura base: 25; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Meditation / Meditação

ID de conteúdo: `los.mage.aether.meditation`.

Modalidade: Próprio personagem. Focos: DPS, Healer, Tank.

restores 35 resource; affects allies.

restaura 35 de recurso; afeta aliados.

Custo: 0; Recarga (s): 25; Alcance (m): 14; Raio (m): 0; Mana restaurada: 35; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

## Guerreiro e Armas

### Cleave / Cutilada

ID de conteúdo: `los.warrior.arms.cleave`.

Modalidade: Cone. Focos: DPS, Healer, Tank.

38 base damage; affects enemies.

38 de dano base; afeta inimigos.

Custo: 8; Recarga (s): 2; Alcance (m): 2.4; Raio (m): 2.6; Dano base: 38; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Charge / Investida

ID de conteúdo: `los.warrior.arms.charge`.

Modalidade: Avanço. Focos: DPS, Healer, Tank.

22 base damage; swept forward movement; affects enemies.

22 de dano base; avanço com colisão; afeta inimigos.

Custo: 12; Recarga (s): 11; Alcance (m): 4.5; Raio (m): 2.6; Dano base: 22; Empurrão: 220; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Rallying Ground / Terreno de Reunião

ID de conteúdo: `los.warrior.arms.rallying_ground`.

Modalidade: Zona persistente. Focos: DPS, Healer, Tank.

7 base damage; 12 base healing; effect duration 8s; values per pulse, every 1s; heals/protects allies and damages/controls enemies.

7 de dano base; 12 de cura base; efeito durante 8s; valores por pulso, a cada 1s; cura/protege aliados e causa dano/controle a inimigos.

Custo: 12; Recarga (s): 18; Alcance (m): 14; Raio (m): 3.3; Dano base: 7; Cura base: 12; Conjuração (s): 0; Duração (s): 8; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Field Dressing / Curativo de Campo

ID de conteúdo: `los.warrior.arms.field_dressing`.

Modalidade: Alvo. Focos: DPS, Healer, Tank.

50 base healing; affects allies.

50 de cura base; afeta aliados.

Custo: 12; Recarga (s): 9; Alcance (m): 4.5; Raio (m): 2.6; Cura base: 50; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Brace / Firmar Guarda

ID de conteúdo: `los.warrior.arms.brace`.

Modalidade: Próprio personagem. Focos: DPS, Healer, Tank.

70 base shield; effect duration 7s; affects allies.

70 de escudo base; efeito durante 7s; afeta aliados.

Custo: 12; Recarga (s): 15; Alcance (m): 14; Raio (m): 0; Escudo: 70; Conjuração (s): 0; Duração (s): 7; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Hamstring / Cortar Tendões

ID de conteúdo: `los.warrior.arms.hamstring`.

Modalidade: Alvo. Focos: DPS, Healer, Tank.

20 base damage; effect duration 4s; slows enemies 50%; affects enemies.

20 de dano base; efeito durante 4s; reduz movimento inimigo em 50%; afeta inimigos.

Custo: 12; Recarga (s): 10; Alcance (m): 2.5; Raio (m): 2.6; Dano base: 20; Lentidão: 0.5; Conjuração (s): 0; Duração (s): 4; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Rally / Reagrupar

ID de conteúdo: `los.warrior.arms.rally`.

Modalidade: Cone. Focos: DPS, Healer, Tank.

25 base healing; removes movement impairments; affects allies.

25 de cura base; remove impedimentos de movimento; afeta aliados.

Custo: 12; Recarga (s): 17; Alcance (m): 5; Raio (m): 2.6; Cura base: 25; Limpeza: True; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Second Wind / Segundo Fôlego

ID de conteúdo: `los.warrior.arms.second_wind`.

Modalidade: Próprio personagem. Focos: DPS, Healer, Tank.

35 base healing; restores 20 resource; affects allies.

35 de cura base; restaura 20 de recurso; afeta aliados.

Custo: 0; Recarga (s): 24; Alcance (m): 14; Raio (m): 0; Cura base: 35; Mana restaurada: 20; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

## Arqueiro e Pontaria

### Aimed Shot / Tiro Preciso

ID de conteúdo: `los.archer.marksmanship.aimed_shot`.

Modalidade: Projétil. Focos: DPS, Healer, Tank.

49 base damage; affects enemies. Cast time 0.85s.

49 de dano base; afeta inimigos. Conjuração de 0.85s.

Custo: 10; Recarga (s): 2; Alcance (m): 18; Raio (m): 0; Dano base: 49; Conjuração (s): 0.85; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Evasive Step / Passo Evasivo

ID de conteúdo: `los.archer.marksmanship.evasive_step`.

Modalidade: Avanço. Focos: DPS, Healer, Tank.

swept forward movement.

avanço com colisão.

Custo: 12; Recarga (s): 9; Alcance (m): 4.2; Raio (m): 2.6; Conjuração (s): 0; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Briar Trap / Armadilha de Espinhos

ID de conteúdo: `los.archer.marksmanship.briar_trap`.

Modalidade: Zona persistente. Focos: DPS, Healer, Tank.

8 base damage; effect duration 7s; values per pulse, every 1s; slows enemies 50%; affects enemies. Cast time 0.6s.

8 de dano base; efeito durante 7s; valores por pulso, a cada 1s; reduz movimento inimigo em 50%; afeta inimigos. Conjuração de 0.6s.

Custo: 12; Recarga (s): 16; Alcance (m): 14; Raio (m): 2.6; Dano base: 8; Lentidão: 0.5; Conjuração (s): 0.6; Duração (s): 7; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Herbal Arrow / Flecha de Ervas

ID de conteúdo: `los.archer.marksmanship.herbal_arrow`.

Modalidade: Alvo. Focos: DPS, Healer, Tank.

44 base healing; affects allies. Cast time 0.45s.

44 de cura base; afeta aliados. Conjuração de 0.45s.

Custo: 12; Recarga (s): 7; Alcance (m): 16; Raio (m): 2.6; Cura base: 44; Conjuração (s): 0.45; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Smoke Ward / Proteção de Fumaça

ID de conteúdo: `los.archer.marksmanship.smoke_ward`.

Modalidade: Próprio personagem. Focos: DPS, Healer, Tank.

60 base shield; effect duration 7s; affects allies.

60 de escudo base; efeito durante 7s; afeta aliados.

Custo: 12; Recarga (s): 14; Alcance (m): 14; Raio (m): 0; Escudo: 60; Conjuração (s): 0; Duração (s): 7; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Binding Shot / Disparo Aprisionador

ID de conteúdo: `los.archer.marksmanship.binding_shot`.

Modalidade: Projétil. Focos: DPS, Healer, Tank.

18 base damage; roots enemies for 2.5s; affects enemies. Cast time 0.85s.

18 de dano base; imobiliza inimigos por 2.5s; afeta inimigos. Conjuração de 0.85s.

Custo: 12; Recarga (s): 14; Alcance (m): 14; Raio (m): 0; Dano base: 18; Imobilização: 2.5; Conjuração (s): 0.85; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Healing Thicket / Moita Restauradora

ID de conteúdo: `los.archer.marksmanship.healing_thicket`.

Modalidade: Zona persistente. Focos: DPS, Healer, Tank.

6 base damage; 12 base healing; effect duration 8s; values per pulse, every 1s; heals/protects allies and damages/controls enemies. Cast time 0.6s.

6 de dano base; 12 de cura base; efeito durante 8s; valores por pulso, a cada 1s; cura/protege aliados e causa dano/controle a inimigos. Conjuração de 0.6s.

Custo: 12; Recarga (s): 18; Alcance (m): 14; Raio (m): 2.6; Dano base: 6; Cura base: 12; Conjuração (s): 0.6; Duração (s): 8; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Antidote / Antídoto

ID de conteúdo: `los.archer.marksmanship.antidote`.

Modalidade: Alvo. Focos: DPS, Healer, Tank.

25 base healing; removes movement impairments; affects allies.

25 de cura base; remove impedimentos de movimento; afeta aliados.

Custo: 12; Recarga (s): 13; Alcance (m): 14; Raio (m): 2.6; Cura base: 25; Limpeza: True; Conjuração (s): 0; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

## Druida e Natureza

### Thorn Bolt / Espinho Perfurante

ID de conteúdo: `los.druid.nature.thorn_bolt`.

Modalidade: Projétil. Focos: DPS, Healer, Tank.

43 base damage; affects enemies. Cast time 0.85s.

43 de dano base; afeta inimigos. Conjuração de 0.85s.

Custo: 10; Recarga (s): 2; Alcance (m): 14; Raio (m): 0; Dano base: 43; Conjuração (s): 0.85; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Living Grove / Bosque Vivo

ID de conteúdo: `los.druid.nature.living_grove`.

Modalidade: Zona persistente. Focos: DPS, Healer, Tank.

8 base damage; 14 base healing; effect duration 8s; values per pulse, every 1s; heals/protects allies and damages/controls enemies. Cast time 0.6s.

8 de dano base; 14 de cura base; efeito durante 8s; valores por pulso, a cada 1s; cura/protege aliados e causa dano/controle a inimigos. Conjuração de 0.6s.

Custo: 12; Recarga (s): 17; Alcance (m): 14; Raio (m): 2.6; Dano base: 8; Cura base: 14; Conjuração (s): 0.6; Duração (s): 8; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Entangling Roots / Raízes Aprisionadoras

ID de conteúdo: `los.druid.nature.entangling_roots`.

Modalidade: Ponto no chão. Focos: DPS, Healer, Tank.

12 base damage; roots enemies for 3s; affects enemies. Cast time 0.6s.

12 de dano base; imobiliza inimigos por 3s; afeta inimigos. Conjuração de 0.6s.

Custo: 12; Recarga (s): 14; Alcance (m): 14; Raio (m): 2.6; Dano base: 12; Imobilização: 3; Conjuração (s): 0.6; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Barkskin / Pele de Casca

ID de conteúdo: `los.druid.nature.barkskin`.

Modalidade: Próprio personagem. Focos: DPS, Healer, Tank.

70 base shield; effect duration 8s; affects allies.

70 de escudo base; efeito durante 8s; afeta aliados.

Custo: 12; Recarga (s): 16; Alcance (m): 14; Raio (m): 0; Escudo: 70; Conjuração (s): 0; Duração (s): 8; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Rejuvenation / Rejuvenescimento

ID de conteúdo: `los.druid.nature.rejuvenation`.

Modalidade: Alvo. Focos: DPS, Healer, Tank.

13 base healing; effect duration 6s; values per pulse, every 1s; affects allies.

13 de cura base; efeito durante 6s; valores por pulso, a cada 1s; afeta aliados.

Custo: 12; Recarga (s): 8; Alcance (m): 14; Raio (m): 2.6; Cura base: 13; Conjuração (s): 0; Duração (s): 6; Intervalo (s): 1; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Renewal / Renovação

ID de conteúdo: `los.druid.nature.renewal`.

Modalidade: Alvo. Focos: DPS, Healer, Tank.

28 base healing; removes movement impairments; affects allies.

28 de cura base; remove impedimentos de movimento; afeta aliados.

Custo: 12; Recarga (s): 13; Alcance (m): 14; Raio (m): 2.6; Cura base: 28; Limpeza: True; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Bear Form / Forma de Urso

ID de conteúdo: `los.druid.nature.bear_form`.

Modalidade: Transformar em Urso. Focos: DPS, Healer, Tank.

transforms and replaces the action bar; no free healing.

transforma e troca a barra; sem cura gratuita.

Custo: 0; Recarga (s): 3; Alcance (m): 14; Raio (m): 2.6; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Feline Form / Forma Felina

ID de conteúdo: `los.druid.nature.cat_form`.

Modalidade: Transformar em Felino. Focos: DPS, Healer, Tank.

transforms and replaces the action bar; no free healing.

transforma e troca a barra; sem cura gratuita.

Custo: 0; Recarga (s): 3; Alcance (m): 14; Raio (m): 2.6; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

## Mago e Fogo

### Pyroblast / Piroexplosão

ID de conteúdo: `los.mage.fire.dps_01`.

Modalidade: Projétil. Focos: DPS.

95 base damage; affects enemies. Cast time 0.85s.

95 de dano base; afeta inimigos. Conjuração de 0.85s.

Custo: 28; Recarga (s): 10; Alcance (m): 14; Raio (m): 0; Dano base: 95; Conjuração (s): 0.85; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Combustion / Combustão

ID de conteúdo: `los.mage.fire.dps_02`.

Modalidade: Alvo. Focos: DPS.

12 base damage; effect duration 7s; values per pulse, every 1s; affects enemies.

12 de dano base; efeito durante 7s; valores por pulso, a cada 1s; afeta inimigos.

Custo: 12; Recarga (s): 13; Alcance (m): 14; Raio (m): 2.6; Dano base: 12; Conjuração (s): 0; Duração (s): 7; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Meteor / Meteoro

ID de conteúdo: `los.mage.fire.dps_03`.

Modalidade: Ponto no chão. Focos: DPS.

85 base damage; affects enemies. Cast time 0.6s.

85 de dano base; afeta inimigos. Conjuração de 0.6s.

Custo: 30; Recarga (s): 16; Alcance (m): 14; Raio (m): 3.4; Dano base: 85; Conjuração (s): 0.6; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Ember Volley / Salva de Brasas

ID de conteúdo: `los.mage.fire.dps_04`.

Modalidade: Projétil. Focos: DPS.

24 base damage; 3 projectiles; affects enemies. Cast time 0.85s.

24 de dano base; 3 projéteis; afeta inimigos. Conjuração de 0.85s.

Custo: 12; Recarga (s): 9; Alcance (m): 14; Raio (m): 0; Dano base: 24; Disparos: 3; Conjuração (s): 0.85; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Inferno / Inferno

ID de conteúdo: `los.mage.fire.dps_05`.

Modalidade: Zona persistente. Focos: DPS.

16 base damage; effect duration 8s; values per pulse, every 1s; affects enemies. Cast time 0.6s.

16 de dano base; efeito durante 8s; valores por pulso, a cada 1s; afeta inimigos. Conjuração de 0.6s.

Custo: 32; Recarga (s): 18; Alcance (m): 14; Raio (m): 3; Dano base: 16; Conjuração (s): 0.6; Duração (s): 8; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Final Spark / Última Faísca

ID de conteúdo: `los.mage.fire.dps_06`.

Modalidade: Alvo. Focos: DPS.

68 base damage; 50% stronger below 30% target health; affects enemies.

68 de dano base; 50% mais forte abaixo de 30% de vida do alvo; afeta inimigos.

Custo: 12; Recarga (s): 12; Alcance (m): 14; Raio (m): 2.6; Dano base: 68; Execução: True; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Scorching Blast / Explosão Escaldante

ID de conteúdo: `los.mage.fire.dps_07`.

Modalidade: Cone. Focos: DPS.

55 base damage; affects enemies.

55 de dano base; afeta inimigos.

Custo: 12; Recarga (s): 11; Alcance (m): 6.5; Raio (m): 2.6; Dano base: 55; Empurrão: 380; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Leaping Flame / Chama Saltitante

ID de conteúdo: `los.mage.fire.dps_08`.

Modalidade: Alvo. Focos: DPS.

36 base damage; jumps to 3 additional targets; affects enemies.

36 de dano base; salta para mais 3 alvos; afeta inimigos.

Custo: 12; Recarga (s): 14; Alcance (m): 14; Raio (m): 2.6; Dano base: 36; Encadeamentos: 3; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Burning Seal / Selo Ardente

ID de conteúdo: `los.mage.fire.dps_09`.

Modalidade: Ponto no chão. Focos: DPS.

35 base damage; roots enemies for 2s; affects enemies. Cast time 0.6s.

35 de dano base; imobiliza inimigos por 2s; afeta inimigos. Conjuração de 0.6s.

Custo: 12; Recarga (s): 14; Alcance (m): 14; Raio (m): 2.6; Dano base: 35; Imobilização: 2; Conjuração (s): 0.6; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Flamethrower / Lança-Chamas

ID de conteúdo: `los.mage.fire.dps_10`.

Modalidade: Cone. Focos: DPS.

17 base damage; effect duration 4s; values per pulse, every 0.8s; affects enemies.

17 de dano base; efeito durante 4s; valores por pulso, a cada 0.8s; afeta inimigos.

Custo: 26; Recarga (s): 15; Alcance (m): 7; Raio (m): 2.6; Dano base: 17; Conjuração (s): 0; Duração (s): 4; Intervalo (s): 0.8; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Molten Bolt / Seta de Magma

ID de conteúdo: `los.mage.fire.dps_11`.

Modalidade: Projétil. Focos: DPS.

40 base damage; effect duration 3s; slows enemies 60%; affects enemies. Cast time 0.85s.

40 de dano base; efeito durante 3s; reduz movimento inimigo em 60%; afeta inimigos. Conjuração de 0.85s.

Custo: 12; Recarga (s): 12; Alcance (m): 14; Raio (m): 0; Dano base: 40; Lentidão: 0.6; Conjuração (s): 0.85; Duração (s): 3; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Supernova / Supernova

ID de conteúdo: `los.mage.fire.dps_12`.

Modalidade: Próprio personagem. Focos: DPS.

115 base damage; affects enemies.

115 de dano base; afeta inimigos.

Custo: 38; Recarga (s): 28; Alcance (m): 14; Raio (m): 4.2; Dano base: 115; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Cauterize Wounds / Cauterizar Feridas

ID de conteúdo: `los.mage.fire.healer_01`.

Modalidade: Alvo. Focos: Healer.

90 base healing; affects allies. Cast time 0.45s.

90 de cura base; afeta aliados. Conjuração de 0.45s.

Custo: 24; Recarga (s): 5; Alcance (m): 14; Raio (m): 2.6; Cura base: 90; Conjuração (s): 0.45; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Kindling Life / Acender a Vida

ID de conteúdo: `los.mage.fire.healer_02`.

Modalidade: Alvo. Focos: Healer.

18 base healing; effect duration 7s; values per pulse, every 1s; affects allies.

18 de cura base; efeito durante 7s; valores por pulso, a cada 1s; afeta aliados.

Custo: 24; Recarga (s): 12; Alcance (m): 14; Raio (m): 2.6; Cura base: 18; Conjuração (s): 0; Duração (s): 7; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Warm Sanctuary / Santuário Acolhedor

ID de conteúdo: `los.mage.fire.healer_03`.

Modalidade: Ponto no chão. Focos: Healer.

70 base healing; affects allies. Cast time 0.6s.

70 de cura base; afeta aliados. Conjuração de 0.6s.

Custo: 30; Recarga (s): 14; Alcance (m): 14; Raio (m): 3.5; Cura base: 70; Conjuração (s): 0.6; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Phoenix Feather / Pena da Fênix

ID de conteúdo: `los.mage.fire.healer_04`.

Modalidade: Alvo. Focos: Healer.

30 base healing; 60 base shield; effect duration 8s; affects allies.

30 de cura base; 60 de escudo base; efeito durante 8s; afeta aliados.

Custo: 12; Recarga (s): 15; Alcance (m): 14; Raio (m): 2.6; Cura base: 30; Escudo: 60; Conjuração (s): 0; Duração (s): 8; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Hearthfire / Fogo do Lar

ID de conteúdo: `los.mage.fire.healer_05`.

Modalidade: Zona persistente. Focos: Healer.

21 base healing; effect duration 9s; values per pulse, every 1s; affects allies. Cast time 0.6s.

21 de cura base; efeito durante 9s; valores por pulso, a cada 1s; afeta aliados. Conjuração de 0.6s.

Custo: 36; Recarga (s): 22; Alcance (m): 14; Raio (m): 2.6; Cura base: 21; Conjuração (s): 0.6; Duração (s): 9; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Rekindle / Reacender

ID de conteúdo: `los.mage.fire.healer_06`.

Modalidade: Alvo. Focos: Healer.

140 base healing; affects allies. Cast time 0.45s.

140 de cura base; afeta aliados. Conjuração de 0.45s.

Custo: 40; Recarga (s): 22; Alcance (m): 14; Raio (m): 2.6; Cura base: 140; Conjuração (s): 0.45; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Purifying Breath / Sopro Purificador

ID de conteúdo: `los.mage.fire.healer_07`.

Modalidade: Cone. Focos: Healer.

55 base healing; removes movement impairments; affects allies. Cast time 0.45s.

55 de cura base; remove impedimentos de movimento; afeta aliados. Conjuração de 0.45s.

Custo: 12; Recarga (s): 18; Alcance (m): 6.5; Raio (m): 2.6; Cura base: 55; Limpeza: True; Conjuração (s): 0.45; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Kindred Flames / Chamas Irmãs

ID de conteúdo: `los.mage.fire.healer_08`.

Modalidade: Alvo. Focos: Healer.

42 base healing; jumps to 3 additional targets; affects allies. Cast time 0.45s.

42 de cura base; salta para mais 3 alvos; afeta aliados. Conjuração de 0.45s.

Custo: 12; Recarga (s): 13; Alcance (m): 14; Raio (m): 2.6; Cura base: 42; Encadeamentos: 3; Conjuração (s): 0.45; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Warm Embrace / Abraço Caloroso

ID de conteúdo: `los.mage.fire.healer_09`.

Modalidade: Alvo. Focos: Healer.

35 base healing; effect duration 6s; reduces incoming damage 25%; affects allies.

35 de cura base; efeito durante 6s; reduz dano recebido em 25%; afeta aliados.

Custo: 12; Recarga (s): 20; Alcance (m): 14; Raio (m): 2.6; Cura base: 35; Mitigação: 0.25; Conjuração (s): 0; Duração (s): 6; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Phoenix Heart / Coração da Fênix

ID de conteúdo: `los.mage.fire.healer_10`.

Modalidade: Próprio personagem. Focos: Healer.

95 base healing; affects allies. Cast time 0.45s.

95 de cura base; afeta aliados. Conjuração de 0.45s.

Custo: 34; Recarga (s): 25; Alcance (m): 14; Raio (m): 4.5; Cura base: 95; Conjuração (s): 0.45; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Renewing Ember / Brasa Renovadora

ID de conteúdo: `los.mage.fire.healer_11`.

Modalidade: Alvo. Focos: Healer.

25 base healing; restores 30 resource; affects allies.

25 de cura base; restaura 30 de recurso; afeta aliados.

Custo: 10; Recarga (s): 24; Alcance (m): 14; Raio (m): 2.6; Cura base: 25; Mana restaurada: 30; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Flame of Life / Chama da Vida

ID de conteúdo: `los.mage.fire.healer_12`.

Modalidade: Zona persistente. Focos: Healer.

8 base damage; 16 base healing; effect duration 8s; values per pulse, every 1s; heals/protects allies and damages/controls enemies. Cast time 0.6s.

8 de dano base; 16 de cura base; efeito durante 8s; valores por pulso, a cada 1s; cura/protege aliados e causa dano/controle a inimigos. Conjuração de 0.6s.

Custo: 32; Recarga (s): 18; Alcance (m): 14; Raio (m): 3.2; Dano base: 8; Cura base: 16; Conjuração (s): 0.6; Duração (s): 8; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Molten Armor / Armadura de Magma

ID de conteúdo: `los.mage.fire.tank_01`.

Modalidade: Próprio personagem. Focos: Tank.

150 base shield; effect duration 10s; affects allies.

150 de escudo base; efeito durante 10s; afeta aliados.

Custo: 26; Recarga (s): 20; Alcance (m): 14; Raio (m): 0; Escudo: 150; Conjuração (s): 0; Duração (s): 10; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Challenging Roar / Rugido Desafiador

ID de conteúdo: `los.mage.fire.tank_02`.

Modalidade: Próprio personagem. Focos: Tank.

20 base damage; forces hostile attention; affects enemies.

20 de dano base; provoca inimigos; afeta inimigos.

Custo: 12; Recarga (s): 12; Alcance (m): 14; Raio (m): 4.5; Dano base: 20; Provocação: True; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Searing Challenge / Desafio Escaldante

ID de conteúdo: `los.mage.fire.tank_03`.

Modalidade: Alvo. Focos: Tank.

25 base damage; effect duration 4s; slows enemies 40%; forces hostile attention; affects enemies.

25 de dano base; efeito durante 4s; reduz movimento inimigo em 40%; provoca inimigos; afeta inimigos.

Custo: 12; Recarga (s): 9; Alcance (m): 14; Raio (m): 2.6; Dano base: 25; Lentidão: 0.4; Provocação: True; Conjuração (s): 0; Duração (s): 4; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Heart of the Furnace / Coração da Fornalha

ID de conteúdo: `los.mage.fire.tank_04`.

Modalidade: Próprio personagem. Focos: Tank.

effect duration 6s; reduces incoming damage 45%; affects allies.

efeito durante 6s; reduz dano recebido em 45%; afeta aliados.

Custo: 24; Recarga (s): 24; Alcance (m): 14; Raio (m): 0; Mitigação: 0.45; Conjuração (s): 0; Duração (s): 6; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Backdraft / Refluxo Ardente

ID de conteúdo: `los.mage.fire.tank_05`.

Modalidade: Cone. Focos: Tank.

35 base damage; affects enemies.

35 de dano base; afeta inimigos.

Custo: 12; Recarga (s): 13; Alcance (m): 4.5; Raio (m): 2.6; Dano base: 35; Empurrão: 520; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Ember Bastion / Bastião de Brasas

ID de conteúdo: `los.mage.fire.tank_06`.

Modalidade: Zona persistente. Focos: Tank.

18 base shield; effect duration 8s; values per pulse, every 1s; affects allies. Cast time 0.6s.

18 de escudo base; efeito durante 8s; valores por pulso, a cada 1s; afeta aliados. Conjuração de 0.6s.

Custo: 30; Recarga (s): 20; Alcance (m): 14; Raio (m): 3.3; Escudo: 18; Conjuração (s): 0.6; Duração (s): 8; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Magma Grasp / Garras de Magma

ID de conteúdo: `los.mage.fire.tank_07`.

Modalidade: Ponto no chão. Focos: Tank.

16 base damage; roots enemies for 3s; affects enemies. Cast time 0.6s.

16 de dano base; imobiliza inimigos por 3s; afeta inimigos. Conjuração de 0.6s.

Custo: 12; Recarga (s): 16; Alcance (m): 14; Raio (m): 3.5; Dano base: 16; Imobilização: 3; Conjuração (s): 0.6; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Flameguard / Guarda de Chamas

ID de conteúdo: `los.mage.fire.tank_08`.

Modalidade: Alvo. Focos: Tank.

110 base shield; effect duration 7s; reduces incoming damage 20%; affects allies.

110 de escudo base; efeito durante 7s; reduz dano recebido em 20%; afeta aliados.

Custo: 12; Recarga (s): 18; Alcance (m): 14; Raio (m): 2.6; Escudo: 110; Mitigação: 0.2; Conjuração (s): 0; Duração (s): 7; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Ash Renewal / Renovação das Cinzas

ID de conteúdo: `los.mage.fire.tank_09`.

Modalidade: Próprio personagem. Focos: Tank.

80 base healing; removes movement impairments; affects allies. Cast time 0.45s.

80 de cura base; remove impedimentos de movimento; afeta aliados. Conjuração de 0.45s.

Custo: 25; Recarga (s): 24; Alcance (m): 14; Raio (m): 0; Cura base: 80; Limpeza: True; Conjuração (s): 0.45; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Scorching Front / Frente Escaldante

ID de conteúdo: `los.mage.fire.tank_10`.

Modalidade: Cone. Focos: Tank.

38 base damage; effect duration 5s; values per pulse, every 1s; slows enemies 65%; affects enemies.

38 de dano base; efeito durante 5s; valores por pulso, a cada 1s; reduz movimento inimigo em 65%; afeta inimigos.

Custo: 12; Recarga (s): 12; Alcance (m): 6.5; Raio (m): 2.6; Dano base: 38; Lentidão: 0.65; Conjuração (s): 0; Duração (s): 5; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Furnace Circle / Círculo da Fornalha

ID de conteúdo: `los.mage.fire.tank_11`.

Modalidade: Zona persistente. Focos: Tank.

12 base damage; effect duration 7s; values per pulse, every 1s; forces hostile attention; affects enemies. Cast time 0.6s.

12 de dano base; efeito durante 7s; valores por pulso, a cada 1s; provoca inimigos; afeta inimigos. Conjuração de 0.6s.

Custo: 28; Recarga (s): 18; Alcance (m): 14; Raio (m): 3.5; Dano base: 12; Provocação: True; Conjuração (s): 0.6; Duração (s): 7; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Blazing Aegis / Égide Flamejante

ID de conteúdo: `los.mage.fire.tank_12`.

Modalidade: Próprio personagem. Focos: Tank.

50 base damage; 70 base shield; effect duration 6s; heals/protects allies and damages/controls enemies.

50 de dano base; 70 de escudo base; efeito durante 6s; cura/protege aliados e causa dano/controle a inimigos.

Custo: 28; Recarga (s): 16; Alcance (m): 14; Raio (m): 3; Dano base: 50; Escudo: 70; Conjuração (s): 0; Duração (s): 6; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

## Mago e Gelo

### Glacial Spear / Lança Glacial

ID de conteúdo: `los.mage.frost.dps_01`.

Modalidade: Projétil. Focos: DPS.

95 base damage; affects enemies. Cast time 0.85s.

95 de dano base; afeta inimigos. Conjuração de 0.85s.

Custo: 28; Recarga (s): 10; Alcance (m): 14; Raio (m): 0; Dano base: 95; Conjuração (s): 0.85; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Frostbite / Queimadura de Gelo

ID de conteúdo: `los.mage.frost.dps_02`.

Modalidade: Alvo. Focos: DPS.

12 base damage; effect duration 7s; values per pulse, every 1s; affects enemies.

12 de dano base; efeito durante 7s; valores por pulso, a cada 1s; afeta inimigos.

Custo: 12; Recarga (s): 13; Alcance (m): 14; Raio (m): 2.6; Dano base: 12; Conjuração (s): 0; Duração (s): 7; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Icefall / Queda de Gelo

ID de conteúdo: `los.mage.frost.dps_03`.

Modalidade: Ponto no chão. Focos: DPS.

85 base damage; affects enemies. Cast time 0.6s.

85 de dano base; afeta inimigos. Conjuração de 0.6s.

Custo: 30; Recarga (s): 16; Alcance (m): 14; Raio (m): 3.4; Dano base: 85; Conjuração (s): 0.6; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Shard Volley / Salva de Estilhaços

ID de conteúdo: `los.mage.frost.dps_04`.

Modalidade: Projétil. Focos: DPS.

24 base damage; 3 projectiles; affects enemies. Cast time 0.85s.

24 de dano base; 3 projéteis; afeta inimigos. Conjuração de 0.85s.

Custo: 12; Recarga (s): 9; Alcance (m): 14; Raio (m): 0; Dano base: 24; Disparos: 3; Conjuração (s): 0.85; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Blizzard / Nevasca

ID de conteúdo: `los.mage.frost.dps_05`.

Modalidade: Zona persistente. Focos: DPS.

16 base damage; effect duration 8s; values per pulse, every 1s; affects enemies. Cast time 0.6s.

16 de dano base; efeito durante 8s; valores por pulso, a cada 1s; afeta inimigos. Conjuração de 0.6s.

Custo: 32; Recarga (s): 18; Alcance (m): 14; Raio (m): 3; Dano base: 16; Conjuração (s): 0.6; Duração (s): 8; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Shatter / Estilhaçar

ID de conteúdo: `los.mage.frost.dps_06`.

Modalidade: Alvo. Focos: DPS.

68 base damage; 50% stronger below 30% target health; affects enemies.

68 de dano base; 50% mais forte abaixo de 30% de vida do alvo; afeta inimigos.

Custo: 12; Recarga (s): 12; Alcance (m): 14; Raio (m): 2.6; Dano base: 68; Execução: True; Conjuração (s): 0; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Avalanche Breath / Sopro de Avalanche

ID de conteúdo: `los.mage.frost.dps_07`.

Modalidade: Cone. Focos: DPS.

55 base damage; affects enemies.

55 de dano base; afeta inimigos.

Custo: 12; Recarga (s): 11; Alcance (m): 6.5; Raio (m): 2.6; Dano base: 55; Empurrão: 380; Conjuração (s): 0; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Leaping Crystal / Cristal Saltitante

ID de conteúdo: `los.mage.frost.dps_08`.

Modalidade: Alvo. Focos: DPS.

36 base damage; jumps to 3 additional targets; affects enemies.

36 de dano base; salta para mais 3 alvos; afeta inimigos.

Custo: 12; Recarga (s): 14; Alcance (m): 14; Raio (m): 2.6; Dano base: 36; Encadeamentos: 3; Conjuração (s): 0; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Frozen Seal / Selo Congelado

ID de conteúdo: `los.mage.frost.dps_09`.

Modalidade: Ponto no chão. Focos: DPS.

35 base damage; roots enemies for 2s; affects enemies. Cast time 0.6s.

35 de dano base; imobiliza inimigos por 2s; afeta inimigos. Conjuração de 0.6s.

Custo: 12; Recarga (s): 14; Alcance (m): 14; Raio (m): 2.6; Dano base: 35; Imobilização: 2; Conjuração (s): 0.6; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Arctic Torrent / Torrente Ártica

ID de conteúdo: `los.mage.frost.dps_10`.

Modalidade: Cone. Focos: DPS.

17 base damage; effect duration 4s; values per pulse, every 0.8s; affects enemies.

17 de dano base; efeito durante 4s; valores por pulso, a cada 0.8s; afeta inimigos.

Custo: 26; Recarga (s): 15; Alcance (m): 7; Raio (m): 2.6; Dano base: 17; Conjuração (s): 0; Duração (s): 4; Intervalo (s): 0.8; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Deep Freeze Bolt / Seta de Frio Profundo

ID de conteúdo: `los.mage.frost.dps_11`.

Modalidade: Projétil. Focos: DPS.

40 base damage; effect duration 3s; slows enemies 60%; affects enemies. Cast time 0.85s.

40 de dano base; efeito durante 3s; reduz movimento inimigo em 60%; afeta inimigos. Conjuração de 0.85s.

Custo: 12; Recarga (s): 12; Alcance (m): 14; Raio (m): 0; Dano base: 40; Lentidão: 0.6; Conjuração (s): 0.85; Duração (s): 3; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Absolute Zero / Zero Absoluto

ID de conteúdo: `los.mage.frost.dps_12`.

Modalidade: Próprio personagem. Focos: DPS.

115 base damage; affects enemies.

115 de dano base; afeta inimigos.

Custo: 38; Recarga (s): 28; Alcance (m): 14; Raio (m): 4.2; Dano base: 115; Conjuração (s): 0; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Cryotherapy / Crioterapia

ID de conteúdo: `los.mage.frost.healer_01`.

Modalidade: Alvo. Focos: Healer.

90 base healing; affects allies. Cast time 0.45s.

90 de cura base; afeta aliados. Conjuração de 0.45s.

Custo: 24; Recarga (s): 5; Alcance (m): 14; Raio (m): 2.6; Cura base: 90; Conjuração (s): 0.45; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Restoring Frost / Geada Restauradora

ID de conteúdo: `los.mage.frost.healer_02`.

Modalidade: Alvo. Focos: Healer.

18 base healing; effect duration 7s; values per pulse, every 1s; affects allies.

18 de cura base; efeito durante 7s; valores por pulso, a cada 1s; afeta aliados.

Custo: 24; Recarga (s): 12; Alcance (m): 14; Raio (m): 2.6; Cura base: 18; Conjuração (s): 0; Duração (s): 7; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Snow Sanctuary / Santuário de Neve

ID de conteúdo: `los.mage.frost.healer_03`.

Modalidade: Ponto no chão. Focos: Healer.

70 base healing; affects allies. Cast time 0.6s.

70 de cura base; afeta aliados. Conjuração de 0.6s.

Custo: 30; Recarga (s): 14; Alcance (m): 14; Raio (m): 3.5; Cura base: 70; Conjuração (s): 0.6; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Crystal Cocoon / Casulo de Cristal

ID de conteúdo: `los.mage.frost.healer_04`.

Modalidade: Alvo. Focos: Healer.

30 base healing; 60 base shield; effect duration 8s; affects allies.

30 de cura base; 60 de escudo base; efeito durante 8s; afeta aliados.

Custo: 12; Recarga (s): 15; Alcance (m): 14; Raio (m): 2.6; Cura base: 30; Escudo: 60; Conjuração (s): 0; Duração (s): 8; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Spring Snow / Neve da Primavera

ID de conteúdo: `los.mage.frost.healer_05`.

Modalidade: Zona persistente. Focos: Healer.

21 base healing; effect duration 9s; values per pulse, every 1s; affects allies. Cast time 0.6s.

21 de cura base; efeito durante 9s; valores por pulso, a cada 1s; afeta aliados. Conjuração de 0.6s.

Custo: 36; Recarga (s): 22; Alcance (m): 14; Raio (m): 2.6; Cura base: 21; Conjuração (s): 0.6; Duração (s): 9; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Winter Renewal / Renovação Invernal

ID de conteúdo: `los.mage.frost.healer_06`.

Modalidade: Alvo. Focos: Healer.

140 base healing; affects allies. Cast time 0.45s.

140 de cura base; afeta aliados. Conjuração de 0.45s.

Custo: 40; Recarga (s): 22; Alcance (m): 14; Raio (m): 2.6; Cura base: 140; Conjuração (s): 0.45; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Cleansing Breeze / Brisa Purificadora

ID de conteúdo: `los.mage.frost.healer_07`.

Modalidade: Cone. Focos: Healer.

55 base healing; removes movement impairments; affects allies. Cast time 0.45s.

55 de cura base; remove impedimentos de movimento; afeta aliados. Conjuração de 0.45s.

Custo: 12; Recarga (s): 18; Alcance (m): 6.5; Raio (m): 2.6; Cura base: 55; Limpeza: True; Conjuração (s): 0.45; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Crystal Current / Corrente de Cristal

ID de conteúdo: `los.mage.frost.healer_08`.

Modalidade: Alvo. Focos: Healer.

42 base healing; jumps to 3 additional targets; affects allies. Cast time 0.45s.

42 de cura base; salta para mais 3 alvos; afeta aliados. Conjuração de 0.45s.

Custo: 12; Recarga (s): 13; Alcance (m): 14; Raio (m): 2.6; Cura base: 42; Encadeamentos: 3; Conjuração (s): 0.45; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Gentle Winter / Inverno Gentil

ID de conteúdo: `los.mage.frost.healer_09`.

Modalidade: Alvo. Focos: Healer.

35 base healing; effect duration 6s; reduces incoming damage 25%; affects allies.

35 de cura base; efeito durante 6s; reduz dano recebido em 25%; afeta aliados.

Custo: 12; Recarga (s): 20; Alcance (m): 14; Raio (m): 2.6; Cura base: 35; Mitigação: 0.25; Conjuração (s): 0; Duração (s): 6; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Heart of Snow / Coração de Neve

ID de conteúdo: `los.mage.frost.healer_10`.

Modalidade: Próprio personagem. Focos: Healer.

95 base healing; affects allies. Cast time 0.45s.

95 de cura base; afeta aliados. Conjuração de 0.45s.

Custo: 34; Recarga (s): 25; Alcance (m): 14; Raio (m): 4.5; Cura base: 95; Conjuração (s): 0.45; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Clearwater / Água Cristalina

ID de conteúdo: `los.mage.frost.healer_11`.

Modalidade: Alvo. Focos: Healer.

25 base healing; restores 30 resource; affects allies.

25 de cura base; restaura 30 de recurso; afeta aliados.

Custo: 10; Recarga (s): 24; Alcance (m): 14; Raio (m): 2.6; Cura base: 25; Mana restaurada: 30; Conjuração (s): 0; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Healing Sleet / Aguaneve Curativa

ID de conteúdo: `los.mage.frost.healer_12`.

Modalidade: Zona persistente. Focos: Healer.

8 base damage; 16 base healing; effect duration 8s; values per pulse, every 1s; heals/protects allies and damages/controls enemies. Cast time 0.6s.

8 de dano base; 16 de cura base; efeito durante 8s; valores por pulso, a cada 1s; cura/protege aliados e causa dano/controle a inimigos. Conjuração de 0.6s.

Custo: 32; Recarga (s): 18; Alcance (m): 14; Raio (m): 3.2; Dano base: 8; Cura base: 16; Conjuração (s): 0.6; Duração (s): 8; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Glacial Armor / Armadura Glacial

ID de conteúdo: `los.mage.frost.tank_01`.

Modalidade: Próprio personagem. Focos: Tank.

150 base shield; effect duration 10s; affects allies.

150 de escudo base; efeito durante 10s; afeta aliados.

Custo: 26; Recarga (s): 20; Alcance (m): 14; Raio (m): 0; Escudo: 150; Conjuração (s): 0; Duração (s): 10; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Winter Challenge / Desafio Invernal

ID de conteúdo: `los.mage.frost.tank_02`.

Modalidade: Próprio personagem. Focos: Tank.

20 base damage; forces hostile attention; affects enemies.

20 de dano base; provoca inimigos; afeta inimigos.

Custo: 12; Recarga (s): 12; Alcance (m): 14; Raio (m): 4.5; Dano base: 20; Provocação: True; Conjuração (s): 0; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Frost Challenge / Provocação Glacial

ID de conteúdo: `los.mage.frost.tank_03`.

Modalidade: Alvo. Focos: Tank.

25 base damage; effect duration 4s; slows enemies 40%; forces hostile attention; affects enemies.

25 de dano base; efeito durante 4s; reduz movimento inimigo em 40%; provoca inimigos; afeta inimigos.

Custo: 12; Recarga (s): 9; Alcance (m): 14; Raio (m): 2.6; Dano base: 25; Lentidão: 0.4; Provocação: True; Conjuração (s): 0; Duração (s): 4; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Frozen Heart / Coração Congelado

ID de conteúdo: `los.mage.frost.tank_04`.

Modalidade: Próprio personagem. Focos: Tank.

effect duration 6s; reduces incoming damage 45%; affects allies.

efeito durante 6s; reduz dano recebido em 45%; afeta aliados.

Custo: 24; Recarga (s): 24; Alcance (m): 14; Raio (m): 0; Mitigação: 0.45; Conjuração (s): 0; Duração (s): 6; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Icebreaker / Quebra-Gelo

ID de conteúdo: `los.mage.frost.tank_05`.

Modalidade: Cone. Focos: Tank.

35 base damage; affects enemies.

35 de dano base; afeta inimigos.

Custo: 12; Recarga (s): 13; Alcance (m): 4.5; Raio (m): 2.6; Dano base: 35; Empurrão: 520; Conjuração (s): 0; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Frozen Bastion / Bastião Congelado

ID de conteúdo: `los.mage.frost.tank_06`.

Modalidade: Zona persistente. Focos: Tank.

18 base shield; effect duration 8s; values per pulse, every 1s; affects allies. Cast time 0.6s.

18 de escudo base; efeito durante 8s; valores por pulso, a cada 1s; afeta aliados. Conjuração de 0.6s.

Custo: 30; Recarga (s): 20; Alcance (m): 14; Raio (m): 3.3; Escudo: 18; Conjuração (s): 0.6; Duração (s): 8; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Ice Prison / Prisão de Gelo

ID de conteúdo: `los.mage.frost.tank_07`.

Modalidade: Ponto no chão. Focos: Tank.

16 base damage; roots enemies for 3s; affects enemies. Cast time 0.6s.

16 de dano base; imobiliza inimigos por 3s; afeta inimigos. Conjuração de 0.6s.

Custo: 12; Recarga (s): 16; Alcance (m): 14; Raio (m): 3.5; Dano base: 16; Imobilização: 3; Conjuração (s): 0.6; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Crystal Aegis / Égide de Cristal

ID de conteúdo: `los.mage.frost.tank_08`.

Modalidade: Alvo. Focos: Tank.

110 base shield; effect duration 7s; reduces incoming damage 20%; affects allies.

110 de escudo base; efeito durante 7s; reduz dano recebido em 20%; afeta aliados.

Custo: 12; Recarga (s): 18; Alcance (m): 14; Raio (m): 2.6; Escudo: 110; Mitigação: 0.2; Conjuração (s): 0; Duração (s): 7; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Thawing Heart / Coração em Degelo

ID de conteúdo: `los.mage.frost.tank_09`.

Modalidade: Próprio personagem. Focos: Tank.

80 base healing; removes movement impairments; affects allies. Cast time 0.45s.

80 de cura base; remove impedimentos de movimento; afeta aliados. Conjuração de 0.45s.

Custo: 25; Recarga (s): 24; Alcance (m): 14; Raio (m): 0; Cura base: 80; Limpeza: True; Conjuração (s): 0.45; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Polar Front / Frente Polar

ID de conteúdo: `los.mage.frost.tank_10`.

Modalidade: Cone. Focos: Tank.

38 base damage; effect duration 5s; values per pulse, every 1s; slows enemies 65%; affects enemies.

38 de dano base; efeito durante 5s; valores por pulso, a cada 1s; reduz movimento inimigo em 65%; afeta inimigos.

Custo: 12; Recarga (s): 12; Alcance (m): 6.5; Raio (m): 2.6; Dano base: 38; Lentidão: 0.65; Conjuração (s): 0; Duração (s): 5; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Winter Keep / Fortaleza Invernal

ID de conteúdo: `los.mage.frost.tank_11`.

Modalidade: Zona persistente. Focos: Tank.

12 base damage; effect duration 7s; values per pulse, every 1s; forces hostile attention; affects enemies. Cast time 0.6s.

12 de dano base; efeito durante 7s; valores por pulso, a cada 1s; provoca inimigos; afeta inimigos. Conjuração de 0.6s.

Custo: 28; Recarga (s): 18; Alcance (m): 14; Raio (m): 3.5; Dano base: 12; Provocação: True; Conjuração (s): 0.6; Duração (s): 7; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

### Arctic Shell / Carapaça Ártica

ID de conteúdo: `los.mage.frost.tank_12`.

Modalidade: Próprio personagem. Focos: Tank.

50 base damage; 70 base shield; effect duration 6s; heals/protects allies and damages/controls enemies.

50 de dano base; 70 de escudo base; efeito durante 6s; cura/protege aliados e causa dano/controle a inimigos.

Custo: 28; Recarga (s): 16; Alcance (m): 14; Raio (m): 3; Dano base: 50; Escudo: 70; Conjuração (s): 0; Duração (s): 6; Velocidade (m/s): 19; Diâmetro do projétil (m): 0.24.

## Mago e Éter

### Aether Lance / Lança Etérea

ID de conteúdo: `los.mage.aether.dps_01`.

Modalidade: Projétil. Focos: DPS.

95 base damage; affects enemies. Cast time 0.85s.

95 de dano base; afeta inimigos. Conjuração de 0.85s.

Custo: 28; Recarga (s): 10; Alcance (m): 14; Raio (m): 0; Dano base: 95; Conjuração (s): 0.85; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Unmaking / Desfazimento

ID de conteúdo: `los.mage.aether.dps_02`.

Modalidade: Alvo. Focos: DPS.

12 base damage; effect duration 7s; values per pulse, every 1s; affects enemies.

12 de dano base; efeito durante 7s; valores por pulso, a cada 1s; afeta inimigos.

Custo: 12; Recarga (s): 13; Alcance (m): 14; Raio (m): 2.6; Dano base: 12; Conjuração (s): 0; Duração (s): 7; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Spatial Rupture / Ruptura Espacial

ID de conteúdo: `los.mage.aether.dps_03`.

Modalidade: Ponto no chão. Focos: DPS.

85 base damage; affects enemies. Cast time 0.6s.

85 de dano base; afeta inimigos. Conjuração de 0.6s.

Custo: 30; Recarga (s): 16; Alcance (m): 14; Raio (m): 3.4; Dano base: 85; Conjuração (s): 0.6; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Arcane Volley / Salva Arcana

ID de conteúdo: `los.mage.aether.dps_04`.

Modalidade: Projétil. Focos: DPS.

24 base damage; 3 projectiles; affects enemies. Cast time 0.85s.

24 de dano base; 3 projéteis; afeta inimigos. Conjuração de 0.85s.

Custo: 12; Recarga (s): 9; Alcance (m): 14; Raio (m): 0; Dano base: 24; Disparos: 3; Conjuração (s): 0.85; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Unstable Rift / Fenda Instável

ID de conteúdo: `los.mage.aether.dps_05`.

Modalidade: Zona persistente. Focos: DPS.

16 base damage; effect duration 8s; values per pulse, every 1s; affects enemies. Cast time 0.6s.

16 de dano base; efeito durante 8s; valores por pulso, a cada 1s; afeta inimigos. Conjuração de 0.6s.

Custo: 32; Recarga (s): 18; Alcance (m): 14; Raio (m): 3; Dano base: 16; Conjuração (s): 0.6; Duração (s): 8; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Severance / Ruptura Final

ID de conteúdo: `los.mage.aether.dps_06`.

Modalidade: Alvo. Focos: DPS.

68 base damage; 50% stronger below 30% target health; affects enemies.

68 de dano base; 50% mais forte abaixo de 30% de vida do alvo; afeta inimigos.

Custo: 12; Recarga (s): 12; Alcance (m): 14; Raio (m): 2.6; Dano base: 68; Execução: True; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Repulsion / Repulsão

ID de conteúdo: `los.mage.aether.dps_07`.

Modalidade: Cone. Focos: DPS.

55 base damage; affects enemies.

55 de dano base; afeta inimigos.

Custo: 12; Recarga (s): 11; Alcance (m): 6.5; Raio (m): 2.6; Dano base: 55; Empurrão: 380; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Arcane Conduction / Condução Arcana

ID de conteúdo: `los.mage.aether.dps_08`.

Modalidade: Alvo. Focos: DPS.

36 base damage; jumps to 3 additional targets; affects enemies.

36 de dano base; salta para mais 3 alvos; afeta inimigos.

Custo: 12; Recarga (s): 14; Alcance (m): 14; Raio (m): 2.6; Dano base: 36; Encadeamentos: 3; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Spatial Seal / Selo Espacial

ID de conteúdo: `los.mage.aether.dps_09`.

Modalidade: Ponto no chão. Focos: DPS.

35 base damage; roots enemies for 2s; affects enemies. Cast time 0.6s.

35 de dano base; imobiliza inimigos por 2s; afeta inimigos. Conjuração de 0.6s.

Custo: 12; Recarga (s): 14; Alcance (m): 14; Raio (m): 2.6; Dano base: 35; Imobilização: 2; Conjuração (s): 0.6; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Aether Torrent / Torrente de Éter

ID de conteúdo: `los.mage.aether.dps_10`.

Modalidade: Cone. Focos: DPS.

17 base damage; effect duration 4s; values per pulse, every 0.8s; affects enemies.

17 de dano base; efeito durante 4s; valores por pulso, a cada 0.8s; afeta inimigos.

Custo: 26; Recarga (s): 15; Alcance (m): 7; Raio (m): 2.6; Dano base: 17; Conjuração (s): 0; Duração (s): 4; Intervalo (s): 0.8; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Distorting Bolt / Seta Distorcedora

ID de conteúdo: `los.mage.aether.dps_11`.

Modalidade: Projétil. Focos: DPS.

40 base damage; effect duration 3s; slows enemies 60%; affects enemies. Cast time 0.85s.

40 de dano base; efeito durante 3s; reduz movimento inimigo em 60%; afeta inimigos. Conjuração de 0.85s.

Custo: 12; Recarga (s): 12; Alcance (m): 14; Raio (m): 0; Dano base: 40; Lentidão: 0.6; Conjuração (s): 0.85; Duração (s): 3; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Singularity / Singularidade

ID de conteúdo: `los.mage.aether.dps_12`.

Modalidade: Próprio personagem. Focos: DPS.

115 base damage; affects enemies.

115 de dano base; afeta inimigos.

Custo: 38; Recarga (s): 28; Alcance (m): 14; Raio (m): 4.2; Dano base: 115; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Recomposition / Recomposição

ID de conteúdo: `los.mage.aether.healer_01`.

Modalidade: Alvo. Focos: Healer.

90 base healing; affects allies. Cast time 0.45s.

90 de cura base; afeta aliados. Conjuração de 0.45s.

Custo: 24; Recarga (s): 5; Alcance (m): 14; Raio (m): 2.6; Cura base: 90; Conjuração (s): 0.45; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Mending Thread / Fio Restaurador

ID de conteúdo: `los.mage.aether.healer_02`.

Modalidade: Alvo. Focos: Healer.

18 base healing; effect duration 7s; values per pulse, every 1s; affects allies.

18 de cura base; efeito durante 7s; valores por pulso, a cada 1s; afeta aliados.

Custo: 24; Recarga (s): 12; Alcance (m): 14; Raio (m): 2.6; Cura base: 18; Conjuração (s): 0; Duração (s): 7; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Restoring Rift / Fenda Restauradora

ID de conteúdo: `los.mage.aether.healer_03`.

Modalidade: Ponto no chão. Focos: Healer.

70 base healing; affects allies. Cast time 0.6s.

70 de cura base; afeta aliados. Conjuração de 0.6s.

Custo: 30; Recarga (s): 14; Alcance (m): 14; Raio (m): 3.5; Cura base: 70; Conjuração (s): 0.6; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Soul Envelope / Invólucro da Alma

ID de conteúdo: `los.mage.aether.healer_04`.

Modalidade: Alvo. Focos: Healer.

30 base healing; 60 base shield; effect duration 8s; affects allies.

30 de cura base; 60 de escudo base; efeito durante 8s; afeta aliados.

Custo: 12; Recarga (s): 15; Alcance (m): 14; Raio (m): 2.6; Cura base: 30; Escudo: 60; Conjuração (s): 0; Duração (s): 8; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Resonant Sanctuary / Santuário Ressonante

ID de conteúdo: `los.mage.aether.healer_05`.

Modalidade: Zona persistente. Focos: Healer.

21 base healing; effect duration 9s; values per pulse, every 1s; affects allies. Cast time 0.6s.

21 de cura base; efeito durante 9s; valores por pulso, a cada 1s; afeta aliados. Conjuração de 0.6s.

Custo: 36; Recarga (s): 22; Alcance (m): 14; Raio (m): 2.6; Cura base: 21; Conjuração (s): 0.6; Duração (s): 9; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Perfect Restoration / Restauração Plena

ID de conteúdo: `los.mage.aether.healer_06`.

Modalidade: Alvo. Focos: Healer.

140 base healing; affects allies. Cast time 0.45s.

140 de cura base; afeta aliados. Conjuração de 0.45s.

Custo: 40; Recarga (s): 22; Alcance (m): 14; Raio (m): 2.6; Cura base: 140; Conjuração (s): 0.45; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Cleansing Pulse / Pulso Purificador

ID de conteúdo: `los.mage.aether.healer_07`.

Modalidade: Cone. Focos: Healer.

55 base healing; removes movement impairments; affects allies. Cast time 0.45s.

55 de cura base; remove impedimentos de movimento; afeta aliados. Conjuração de 0.45s.

Custo: 12; Recarga (s): 18; Alcance (m): 6.5; Raio (m): 2.6; Cura base: 55; Limpeza: True; Conjuração (s): 0.45; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Vital Conduction / Condução Vital

ID de conteúdo: `los.mage.aether.healer_08`.

Modalidade: Alvo. Focos: Healer.

42 base healing; jumps to 3 additional targets; affects allies. Cast time 0.45s.

42 de cura base; salta para mais 3 alvos; afeta aliados. Conjuração de 0.45s.

Custo: 12; Recarga (s): 13; Alcance (m): 14; Raio (m): 2.6; Cura base: 42; Encadeamentos: 3; Conjuração (s): 0.45; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Protective Distortion / Distorção Protetora

ID de conteúdo: `los.mage.aether.healer_09`.

Modalidade: Alvo. Focos: Healer.

35 base healing; effect duration 6s; reduces incoming damage 25%; affects allies.

35 de cura base; efeito durante 6s; reduz dano recebido em 25%; afeta aliados.

Custo: 12; Recarga (s): 20; Alcance (m): 14; Raio (m): 2.6; Cura base: 35; Mitigação: 0.25; Conjuração (s): 0; Duração (s): 6; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Harmonic Pulse / Pulso Harmônico

ID de conteúdo: `los.mage.aether.healer_10`.

Modalidade: Próprio personagem. Focos: Healer.

95 base healing; affects allies. Cast time 0.45s.

95 de cura base; afeta aliados. Conjuração de 0.45s.

Custo: 34; Recarga (s): 25; Alcance (m): 14; Raio (m): 4.5; Cura base: 95; Conjuração (s): 0.45; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Essence Return / Retorno da Essência

ID de conteúdo: `los.mage.aether.healer_11`.

Modalidade: Alvo. Focos: Healer.

25 base healing; restores 30 resource; affects allies.

25 de cura base; restaura 30 de recurso; afeta aliados.

Custo: 10; Recarga (s): 24; Alcance (m): 14; Raio (m): 2.6; Cura base: 25; Mana restaurada: 30; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Balance Field / Campo de Equilíbrio

ID de conteúdo: `los.mage.aether.healer_12`.

Modalidade: Zona persistente. Focos: Healer.

8 base damage; 16 base healing; effect duration 8s; values per pulse, every 1s; heals/protects allies and damages/controls enemies. Cast time 0.6s.

8 de dano base; 16 de cura base; efeito durante 8s; valores por pulso, a cada 1s; cura/protege aliados e causa dano/controle a inimigos. Conjuração de 0.6s.

Custo: 32; Recarga (s): 18; Alcance (m): 14; Raio (m): 3.2; Dano base: 8; Cura base: 16; Conjuração (s): 0.6; Duração (s): 8; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Aether Aegis / Égide Etérea

ID de conteúdo: `los.mage.aether.tank_01`.

Modalidade: Próprio personagem. Focos: Tank.

150 base shield; effect duration 10s; affects allies.

150 de escudo base; efeito durante 10s; afeta aliados.

Custo: 26; Recarga (s): 20; Alcance (m): 14; Raio (m): 0; Escudo: 150; Conjuração (s): 0; Duração (s): 10; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Gravitational Challenge / Desafio Gravitacional

ID de conteúdo: `los.mage.aether.tank_02`.

Modalidade: Próprio personagem. Focos: Tank.

20 base damage; forces hostile attention; affects enemies.

20 de dano base; provoca inimigos; afeta inimigos.

Custo: 12; Recarga (s): 12; Alcance (m): 14; Raio (m): 4.5; Dano base: 20; Provocação: True; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Attention Thread / Fio de Atenção

ID de conteúdo: `los.mage.aether.tank_03`.

Modalidade: Alvo. Focos: Tank.

25 base damage; effect duration 4s; slows enemies 40%; forces hostile attention; affects enemies.

25 de dano base; efeito durante 4s; reduz movimento inimigo em 40%; provoca inimigos; afeta inimigos.

Custo: 12; Recarga (s): 9; Alcance (m): 14; Raio (m): 2.6; Dano base: 25; Lentidão: 0.4; Provocação: True; Conjuração (s): 0; Duração (s): 4; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Phase Anchor / Âncora de Fase

ID de conteúdo: `los.mage.aether.tank_04`.

Modalidade: Próprio personagem. Focos: Tank.

effect duration 6s; reduces incoming damage 45%; affects allies.

efeito durante 6s; reduz dano recebido em 45%; afeta aliados.

Custo: 24; Recarga (s): 24; Alcance (m): 14; Raio (m): 0; Mitigação: 0.45; Conjuração (s): 0; Duração (s): 6; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Spatial Rejection / Rejeição Espacial

ID de conteúdo: `los.mage.aether.tank_05`.

Modalidade: Cone. Focos: Tank.

35 base damage; affects enemies.

35 de dano base; afeta inimigos.

Custo: 12; Recarga (s): 13; Alcance (m): 4.5; Raio (m): 2.6; Dano base: 35; Empurrão: 520; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Resonant Bastion / Bastião Ressonante

ID de conteúdo: `los.mage.aether.tank_06`.

Modalidade: Zona persistente. Focos: Tank.

18 base shield; effect duration 8s; values per pulse, every 1s; affects allies. Cast time 0.6s.

18 de escudo base; efeito durante 8s; valores por pulso, a cada 1s; afeta aliados. Conjuração de 0.6s.

Custo: 30; Recarga (s): 20; Alcance (m): 14; Raio (m): 3.3; Escudo: 18; Conjuração (s): 0.6; Duração (s): 8; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Gravity Well / Poço Gravitacional

ID de conteúdo: `los.mage.aether.tank_07`.

Modalidade: Ponto no chão. Focos: Tank.

16 base damage; roots enemies for 3s; affects enemies. Cast time 0.6s.

16 de dano base; imobiliza inimigos por 3s; afeta inimigos. Conjuração de 0.6s.

Custo: 12; Recarga (s): 16; Alcance (m): 14; Raio (m): 3.5; Dano base: 16; Imobilização: 3; Conjuração (s): 0.6; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Spatial Shelter / Abrigo Espacial

ID de conteúdo: `los.mage.aether.tank_08`.

Modalidade: Alvo. Focos: Tank.

110 base shield; effect duration 7s; reduces incoming damage 20%; affects allies.

110 de escudo base; efeito durante 7s; reduz dano recebido em 20%; afeta aliados.

Custo: 12; Recarga (s): 18; Alcance (m): 14; Raio (m): 2.6; Escudo: 110; Mitigação: 0.2; Conjuração (s): 0; Duração (s): 7; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Reconstitute / Reconstituir

ID de conteúdo: `los.mage.aether.tank_09`.

Modalidade: Próprio personagem. Focos: Tank.

80 base healing; removes movement impairments; affects allies. Cast time 0.45s.

80 de cura base; remove impedimentos de movimento; afeta aliados. Conjuração de 0.45s.

Custo: 25; Recarga (s): 24; Alcance (m): 14; Raio (m): 0; Cura base: 80; Limpeza: True; Conjuração (s): 0.45; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Slowing Wave / Onda de Lentidão

ID de conteúdo: `los.mage.aether.tank_10`.

Modalidade: Cone. Focos: Tank.

38 base damage; effect duration 5s; values per pulse, every 1s; slows enemies 65%; affects enemies.

38 de dano base; efeito durante 5s; valores por pulso, a cada 1s; reduz movimento inimigo em 65%; afeta inimigos.

Custo: 12; Recarga (s): 12; Alcance (m): 6.5; Raio (m): 2.6; Dano base: 38; Lentidão: 0.65; Conjuração (s): 0; Duração (s): 5; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Challenge Rift / Fenda Desafiadora

ID de conteúdo: `los.mage.aether.tank_11`.

Modalidade: Zona persistente. Focos: Tank.

12 base damage; effect duration 7s; values per pulse, every 1s; forces hostile attention; affects enemies. Cast time 0.6s.

12 de dano base; efeito durante 7s; valores por pulso, a cada 1s; provoca inimigos; afeta inimigos. Conjuração de 0.6s.

Custo: 28; Recarga (s): 18; Alcance (m): 14; Raio (m): 3.5; Dano base: 12; Provocação: True; Conjuração (s): 0.6; Duração (s): 7; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Protective Envelope / Invólucro Protetor

ID de conteúdo: `los.mage.aether.tank_12`.

Modalidade: Próprio personagem. Focos: Tank.

50 base damage; 70 base shield; effect duration 6s; heals/protects allies and damages/controls enemies.

50 de dano base; 70 de escudo base; efeito durante 6s; cura/protege aliados e causa dano/controle a inimigos.

Custo: 28; Recarga (s): 16; Alcance (m): 14; Raio (m): 3; Dano base: 50; Escudo: 70; Conjuração (s): 0; Duração (s): 6; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

## Guerreiro e Armas

### Javelin / Azagaia

ID de conteúdo: `los.warrior.arms.dps_01`.

Modalidade: Projétil. Focos: DPS.

95 base damage; affects enemies.

95 de dano base; afeta inimigos.

Custo: 28; Recarga (s): 10; Alcance (m): 14; Raio (m): 0; Dano base: 95; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Rend / Dilacerar

ID de conteúdo: `los.warrior.arms.dps_02`.

Modalidade: Alvo. Focos: DPS.

12 base damage; effect duration 7s; values per pulse, every 1s; affects enemies.

12 de dano base; efeito durante 7s; valores por pulso, a cada 1s; afeta inimigos.

Custo: 12; Recarga (s): 13; Alcance (m): 2.6; Raio (m): 2.6; Dano base: 12; Conjuração (s): 0; Duração (s): 7; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Shattering Impact / Impacto Devastador

ID de conteúdo: `los.warrior.arms.dps_03`.

Modalidade: Ponto no chão. Focos: DPS.

85 base damage; affects enemies.

85 de dano base; afeta inimigos.

Custo: 30; Recarga (s): 16; Alcance (m): 14; Raio (m): 3.4; Dano base: 85; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Throwing Blades / Lâminas Arremessadas

ID de conteúdo: `los.warrior.arms.dps_04`.

Modalidade: Projétil. Focos: DPS.

24 base damage; 3 projectiles; affects enemies.

24 de dano base; 3 projéteis; afeta inimigos.

Custo: 12; Recarga (s): 9; Alcance (m): 14; Raio (m): 0; Dano base: 24; Disparos: 3; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Whirling Blades / Lâminas Giratórias

ID de conteúdo: `los.warrior.arms.dps_05`.

Modalidade: Zona persistente. Focos: DPS.

16 base damage; effect duration 8s; values per pulse, every 1s; affects enemies.

16 de dano base; efeito durante 8s; valores por pulso, a cada 1s; afeta inimigos.

Custo: 32; Recarga (s): 18; Alcance (m): 14; Raio (m): 3; Dano base: 16; Conjuração (s): 0; Duração (s): 8; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Execution / Execução

ID de conteúdo: `los.warrior.arms.dps_06`.

Modalidade: Alvo. Focos: DPS.

68 base damage; 50% stronger below 30% target health; affects enemies.

68 de dano base; 50% mais forte abaixo de 30% de vida do alvo; afeta inimigos.

Custo: 12; Recarga (s): 12; Alcance (m): 2.6; Raio (m): 2.6; Dano base: 68; Execução: True; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Sweeping Strike / Golpe Varredor

ID de conteúdo: `los.warrior.arms.dps_07`.

Modalidade: Cone. Focos: DPS.

55 base damage; affects enemies.

55 de dano base; afeta inimigos.

Custo: 12; Recarga (s): 11; Alcance (m): 6.5; Raio (m): 2.6; Dano base: 55; Empurrão: 380; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Ricocheting Axe / Machado Ricocheteante

ID de conteúdo: `los.warrior.arms.dps_08`.

Modalidade: Alvo. Focos: DPS.

36 base damage; jumps to 3 additional targets; affects enemies.

36 de dano base; salta para mais 3 alvos; afeta inimigos.

Custo: 12; Recarga (s): 14; Alcance (m): 2.6; Raio (m): 2.6; Dano base: 36; Encadeamentos: 3; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Pinning Strike / Golpe Aprisionador

ID de conteúdo: `los.warrior.arms.dps_09`.

Modalidade: Ponto no chão. Focos: DPS.

35 base damage; roots enemies for 2s; affects enemies.

35 de dano base; imobiliza inimigos por 2s; afeta inimigos.

Custo: 12; Recarga (s): 14; Alcance (m): 14; Raio (m): 2.6; Dano base: 35; Imobilização: 2; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Blade Flurry / Rajada de Lâminas

ID de conteúdo: `los.warrior.arms.dps_10`.

Modalidade: Cone. Focos: DPS.

17 base damage; effect duration 4s; values per pulse, every 0.8s; affects enemies.

17 de dano base; efeito durante 4s; valores por pulso, a cada 0.8s; afeta inimigos.

Custo: 26; Recarga (s): 15; Alcance (m): 7; Raio (m): 2.6; Dano base: 17; Conjuração (s): 0; Duração (s): 4; Intervalo (s): 0.8; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Crippling Throw / Arremesso Incapacitante

ID de conteúdo: `los.warrior.arms.dps_11`.

Modalidade: Projétil. Focos: DPS.

40 base damage; effect duration 3s; slows enemies 60%; affects enemies.

40 de dano base; efeito durante 3s; reduz movimento inimigo em 60%; afeta inimigos.

Custo: 12; Recarga (s): 12; Alcance (m): 14; Raio (m): 0; Dano base: 40; Lentidão: 0.6; Conjuração (s): 0; Duração (s): 3; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Earthshaker / Abalo da Terra

ID de conteúdo: `los.warrior.arms.dps_12`.

Modalidade: Próprio personagem. Focos: DPS.

115 base damage; affects enemies.

115 de dano base; afeta inimigos.

Custo: 38; Recarga (s): 28; Alcance (m): 14; Raio (m): 4.2; Dano base: 115; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Combat First Aid / Primeiros Socorros

ID de conteúdo: `los.warrior.arms.healer_01`.

Modalidade: Alvo. Focos: Healer.

90 base healing; affects allies.

90 de cura base; afeta aliados.

Custo: 24; Recarga (s): 5; Alcance (m): 14; Raio (m): 2.6; Cura base: 90; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Steady Recovery / Recuperação Constante

ID de conteúdo: `los.warrior.arms.healer_02`.

Modalidade: Alvo. Focos: Healer.

18 base healing; effect duration 7s; values per pulse, every 1s; affects allies.

18 de cura base; efeito durante 7s; valores por pulso, a cada 1s; afeta aliados.

Custo: 24; Recarga (s): 12; Alcance (m): 14; Raio (m): 2.6; Cura base: 18; Conjuração (s): 0; Duração (s): 7; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Medic Supply Drop / Suprimentos Médicos

ID de conteúdo: `los.warrior.arms.healer_03`.

Modalidade: Ponto no chão. Focos: Healer.

70 base healing; affects allies.

70 de cura base; afeta aliados.

Custo: 30; Recarga (s): 14; Alcance (m): 14; Raio (m): 3.5; Cura base: 70; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Protective Dressing / Curativo Protetor

ID de conteúdo: `los.warrior.arms.healer_04`.

Modalidade: Alvo. Focos: Healer.

30 base healing; 60 base shield; effect duration 8s; affects allies.

30 de cura base; 60 de escudo base; efeito durante 8s; afeta aliados.

Custo: 12; Recarga (s): 15; Alcance (m): 14; Raio (m): 2.6; Cura base: 30; Escudo: 60; Conjuração (s): 0; Duração (s): 8; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Recovery Standard / Estandarte de Recuperação

ID de conteúdo: `los.warrior.arms.healer_05`.

Modalidade: Zona persistente. Focos: Healer.

21 base healing; effect duration 9s; values per pulse, every 1s; affects allies.

21 de cura base; efeito durante 9s; valores por pulso, a cada 1s; afeta aliados.

Custo: 36; Recarga (s): 22; Alcance (m): 14; Raio (m): 2.6; Cura base: 21; Conjuração (s): 0; Duração (s): 9; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Emergency Treatment / Tratamento de Emergência

ID de conteúdo: `los.warrior.arms.healer_06`.

Modalidade: Alvo. Focos: Healer.

140 base healing; affects allies.

140 de cura base; afeta aliados.

Custo: 40; Recarga (s): 22; Alcance (m): 14; Raio (m): 2.6; Cura base: 140; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Rallying Breath / Fôlego de Batalha

ID de conteúdo: `los.warrior.arms.healer_07`.

Modalidade: Cone. Focos: Healer.

55 base healing; removes movement impairments; affects allies.

55 de cura base; remove impedimentos de movimento; afeta aliados.

Custo: 12; Recarga (s): 18; Alcance (m): 6.5; Raio (m): 2.6; Cura base: 55; Limpeza: True; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Field Medic Chain / Socorro Coordenado

ID de conteúdo: `los.warrior.arms.healer_08`.

Modalidade: Alvo. Focos: Healer.

42 base healing; jumps to 3 additional targets; affects allies.

42 de cura base; salta para mais 3 alvos; afeta aliados.

Custo: 12; Recarga (s): 13; Alcance (m): 14; Raio (m): 2.6; Cura base: 42; Encadeamentos: 3; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Hold Together / Resistir Juntos

ID de conteúdo: `los.warrior.arms.healer_09`.

Modalidade: Alvo. Focos: Healer.

35 base healing; effect duration 6s; reduces incoming damage 25%; affects allies.

35 de cura base; efeito durante 6s; reduz dano recebido em 25%; afeta aliados.

Custo: 12; Recarga (s): 20; Alcance (m): 14; Raio (m): 2.6; Cura base: 35; Mitigação: 0.25; Conjuração (s): 0; Duração (s): 6; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Commanding Renewal / Renovação do Comandante

ID de conteúdo: `los.warrior.arms.healer_10`.

Modalidade: Próprio personagem. Focos: Healer.

95 base healing; affects allies.

95 de cura base; afeta aliados.

Custo: 34; Recarga (s): 25; Alcance (m): 14; Raio (m): 4.5; Cura base: 95; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Fresh Supplies / Suprimentos Frescos

ID de conteúdo: `los.warrior.arms.healer_11`.

Modalidade: Alvo. Focos: Healer.

25 base healing; restores 30 resource; affects allies.

25 de cura base; restaura 30 de recurso; afeta aliados.

Custo: 10; Recarga (s): 24; Alcance (m): 14; Raio (m): 2.6; Cura base: 25; Mana restaurada: 30; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Medic Rally Point / Ponto de Socorro

ID de conteúdo: `los.warrior.arms.healer_12`.

Modalidade: Zona persistente. Focos: Healer.

8 base damage; 16 base healing; effect duration 8s; values per pulse, every 1s; heals/protects allies and damages/controls enemies.

8 de dano base; 16 de cura base; efeito durante 8s; valores por pulso, a cada 1s; cura/protege aliados e causa dano/controle a inimigos.

Custo: 32; Recarga (s): 18; Alcance (m): 14; Raio (m): 3.2; Dano base: 8; Cura base: 16; Conjuração (s): 0; Duração (s): 8; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Shield Wall / Muralha de Escudos

ID de conteúdo: `los.warrior.arms.tank_01`.

Modalidade: Próprio personagem. Focos: Tank.

150 base shield; effect duration 10s; affects allies.

150 de escudo base; efeito durante 10s; afeta aliados.

Custo: 26; Recarga (s): 20; Alcance (m): 14; Raio (m): 0; Escudo: 150; Conjuração (s): 0; Duração (s): 10; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Challenging Shout / Grito Desafiador

ID de conteúdo: `los.warrior.arms.tank_02`.

Modalidade: Próprio personagem. Focos: Tank.

20 base damage; forces hostile attention; affects enemies.

20 de dano base; provoca inimigos; afeta inimigos.

Custo: 12; Recarga (s): 12; Alcance (m): 14; Raio (m): 4.5; Dano base: 20; Provocação: True; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Duel Challenge / Desafio de Duelo

ID de conteúdo: `los.warrior.arms.tank_03`.

Modalidade: Alvo. Focos: Tank.

25 base damage; effect duration 4s; slows enemies 40%; forces hostile attention; affects enemies.

25 de dano base; efeito durante 4s; reduz movimento inimigo em 40%; provoca inimigos; afeta inimigos.

Custo: 12; Recarga (s): 9; Alcance (m): 14; Raio (m): 2.6; Dano base: 25; Lentidão: 0.4; Provocação: True; Conjuração (s): 0; Duração (s): 4; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Unbreakable / Inabalável

ID de conteúdo: `los.warrior.arms.tank_04`.

Modalidade: Próprio personagem. Focos: Tank.

effect duration 6s; reduces incoming damage 45%; affects allies.

efeito durante 6s; reduz dano recebido em 45%; afeta aliados.

Custo: 24; Recarga (s): 24; Alcance (m): 14; Raio (m): 0; Mitigação: 0.45; Conjuração (s): 0; Duração (s): 6; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Shield Sweep / Varredura de Escudo

ID de conteúdo: `los.warrior.arms.tank_05`.

Modalidade: Cone. Focos: Tank.

35 base damage; affects enemies.

35 de dano base; afeta inimigos.

Custo: 12; Recarga (s): 13; Alcance (m): 4.5; Raio (m): 2.6; Dano base: 35; Empurrão: 520; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Defensive Standard / Estandarte Defensivo

ID de conteúdo: `los.warrior.arms.tank_06`.

Modalidade: Zona persistente. Focos: Tank.

18 base shield; effect duration 8s; values per pulse, every 1s; affects allies.

18 de escudo base; efeito durante 8s; valores por pulso, a cada 1s; afeta aliados.

Custo: 30; Recarga (s): 20; Alcance (m): 14; Raio (m): 3.3; Escudo: 18; Conjuração (s): 0; Duração (s): 8; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Pinning Ground / Terreno Aprisionador

ID de conteúdo: `los.warrior.arms.tank_07`.

Modalidade: Ponto no chão. Focos: Tank.

16 base damage; roots enemies for 3s; affects enemies.

16 de dano base; imobiliza inimigos por 3s; afeta inimigos.

Custo: 12; Recarga (s): 16; Alcance (m): 14; Raio (m): 3.5; Dano base: 16; Imobilização: 3; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Protect Ally / Proteger Aliado

ID de conteúdo: `los.warrior.arms.tank_08`.

Modalidade: Alvo. Focos: Tank.

110 base shield; effect duration 7s; reduces incoming damage 20%; affects allies.

110 de escudo base; efeito durante 7s; reduz dano recebido em 20%; afeta aliados.

Custo: 12; Recarga (s): 18; Alcance (m): 14; Raio (m): 2.6; Escudo: 110; Mitigação: 0.2; Conjuração (s): 0; Duração (s): 7; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Battle Recovery / Recuperação de Batalha

ID de conteúdo: `los.warrior.arms.tank_09`.

Modalidade: Próprio personagem. Focos: Tank.

80 base healing; removes movement impairments; affects allies.

80 de cura base; remove impedimentos de movimento; afeta aliados.

Custo: 25; Recarga (s): 24; Alcance (m): 14; Raio (m): 0; Cura base: 80; Limpeza: True; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Crippling Sweep / Varredura Incapacitante

ID de conteúdo: `los.warrior.arms.tank_10`.

Modalidade: Cone. Focos: Tank.

38 base damage; effect duration 5s; values per pulse, every 1s; slows enemies 65%; affects enemies.

38 de dano base; efeito durante 5s; valores por pulso, a cada 1s; reduz movimento inimigo em 65%; afeta inimigos.

Custo: 12; Recarga (s): 12; Alcance (m): 6.5; Raio (m): 2.6; Dano base: 38; Lentidão: 0.65; Conjuração (s): 0; Duração (s): 5; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Hold the Line / Manter a Linha

ID de conteúdo: `los.warrior.arms.tank_11`.

Modalidade: Zona persistente. Focos: Tank.

12 base damage; effect duration 7s; values per pulse, every 1s; forces hostile attention; affects enemies.

12 de dano base; efeito durante 7s; valores por pulso, a cada 1s; provoca inimigos; afeta inimigos.

Custo: 28; Recarga (s): 18; Alcance (m): 14; Raio (m): 3.5; Dano base: 12; Provocação: True; Conjuração (s): 0; Duração (s): 7; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Iron Retaliation / Retaliação de Ferro

ID de conteúdo: `los.warrior.arms.tank_12`.

Modalidade: Próprio personagem. Focos: Tank.

50 base damage; 70 base shield; effect duration 6s; heals/protects allies and damages/controls enemies.

50 de dano base; 70 de escudo base; efeito durante 6s; cura/protege aliados e causa dano/controle a inimigos.

Custo: 28; Recarga (s): 16; Alcance (m): 14; Raio (m): 3; Dano base: 50; Escudo: 70; Conjuração (s): 0; Duração (s): 6; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

## Arqueiro e Pontaria

### Piercing Arrow / Flecha Perfurante

ID de conteúdo: `los.archer.marksmanship.dps_01`.

Modalidade: Projétil. Focos: DPS.

95 base damage; affects enemies. Cast time 0.85s.

95 de dano base; afeta inimigos. Conjuração de 0.85s.

Custo: 28; Recarga (s): 10; Alcance (m): 14; Raio (m): 0; Dano base: 95; Conjuração (s): 0.85; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Bleeding Shot / Disparo Sangrento

ID de conteúdo: `los.archer.marksmanship.dps_02`.

Modalidade: Alvo. Focos: DPS.

12 base damage; effect duration 7s; values per pulse, every 1s; affects enemies.

12 de dano base; efeito durante 7s; valores por pulso, a cada 1s; afeta inimigos.

Custo: 12; Recarga (s): 13; Alcance (m): 14; Raio (m): 2.6; Dano base: 12; Conjuração (s): 0; Duração (s): 7; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Explosive Arrow / Flecha Explosiva

ID de conteúdo: `los.archer.marksmanship.dps_03`.

Modalidade: Ponto no chão. Focos: DPS.

85 base damage; affects enemies. Cast time 0.6s.

85 de dano base; afeta inimigos. Conjuração de 0.6s.

Custo: 30; Recarga (s): 16; Alcance (m): 14; Raio (m): 3.4; Dano base: 85; Conjuração (s): 0.6; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Triple Shot / Disparo Triplo

ID de conteúdo: `los.archer.marksmanship.dps_04`.

Modalidade: Projétil. Focos: DPS.

24 base damage; 3 projectiles; affects enemies. Cast time 0.85s.

24 de dano base; 3 projéteis; afeta inimigos. Conjuração de 0.85s.

Custo: 12; Recarga (s): 9; Alcance (m): 14; Raio (m): 0; Dano base: 24; Disparos: 3; Conjuração (s): 0.85; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Arrow Rain / Chuva de Flechas

ID de conteúdo: `los.archer.marksmanship.dps_05`.

Modalidade: Zona persistente. Focos: DPS.

16 base damage; effect duration 8s; values per pulse, every 1s; affects enemies. Cast time 0.6s.

16 de dano base; efeito durante 8s; valores por pulso, a cada 1s; afeta inimigos. Conjuração de 0.6s.

Custo: 32; Recarga (s): 18; Alcance (m): 14; Raio (m): 3; Dano base: 16; Conjuração (s): 0.6; Duração (s): 8; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Finishing Shot / Disparo Final

ID de conteúdo: `los.archer.marksmanship.dps_06`.

Modalidade: Alvo. Focos: DPS.

68 base damage; 50% stronger below 30% target health; affects enemies.

68 de dano base; 50% mais forte abaixo de 30% de vida do alvo; afeta inimigos.

Custo: 12; Recarga (s): 12; Alcance (m): 14; Raio (m): 2.6; Dano base: 68; Execução: True; Conjuração (s): 0; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Scattershot / Disparo Dispersor

ID de conteúdo: `los.archer.marksmanship.dps_07`.

Modalidade: Cone. Focos: DPS.

55 base damage; affects enemies.

55 de dano base; afeta inimigos.

Custo: 12; Recarga (s): 11; Alcance (m): 6.5; Raio (m): 2.6; Dano base: 55; Empurrão: 380; Conjuração (s): 0; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Ricochet / Ricochete

ID de conteúdo: `los.archer.marksmanship.dps_08`.

Modalidade: Alvo. Focos: DPS.

36 base damage; jumps to 3 additional targets; affects enemies.

36 de dano base; salta para mais 3 alvos; afeta inimigos.

Custo: 12; Recarga (s): 14; Alcance (m): 14; Raio (m): 2.6; Dano base: 36; Encadeamentos: 3; Conjuração (s): 0; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Pinning Trap / Armadilha Aprisionadora

ID de conteúdo: `los.archer.marksmanship.dps_09`.

Modalidade: Ponto no chão. Focos: DPS.

35 base damage; roots enemies for 2s; affects enemies. Cast time 0.6s.

35 de dano base; imobiliza inimigos por 2s; afeta inimigos. Conjuração de 0.6s.

Custo: 12; Recarga (s): 14; Alcance (m): 14; Raio (m): 2.6; Dano base: 35; Imobilização: 2; Conjuração (s): 0.6; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Rapid Volley / Salva Rápida

ID de conteúdo: `los.archer.marksmanship.dps_10`.

Modalidade: Cone. Focos: DPS.

17 base damage; effect duration 4s; values per pulse, every 0.8s; affects enemies.

17 de dano base; efeito durante 4s; valores por pulso, a cada 0.8s; afeta inimigos.

Custo: 26; Recarga (s): 15; Alcance (m): 7; Raio (m): 2.6; Dano base: 17; Conjuração (s): 0; Duração (s): 4; Intervalo (s): 0.8; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Crippling Arrow / Flecha Incapacitante

ID de conteúdo: `los.archer.marksmanship.dps_11`.

Modalidade: Projétil. Focos: DPS.

40 base damage; effect duration 3s; slows enemies 60%; affects enemies. Cast time 0.85s.

40 de dano base; efeito durante 3s; reduz movimento inimigo em 60%; afeta inimigos. Conjuração de 0.85s.

Custo: 12; Recarga (s): 12; Alcance (m): 14; Raio (m): 0; Dano base: 40; Lentidão: 0.6; Conjuração (s): 0.85; Duração (s): 3; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Point-Blank Barrage / Barragem Próxima

ID de conteúdo: `los.archer.marksmanship.dps_12`.

Modalidade: Próprio personagem. Focos: DPS.

115 base damage; affects enemies.

115 de dano base; afeta inimigos.

Custo: 38; Recarga (s): 28; Alcance (m): 14; Raio (m): 4.2; Dano base: 115; Conjuração (s): 0; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Restoring Arrow / Flecha Restauradora

ID de conteúdo: `los.archer.marksmanship.healer_01`.

Modalidade: Alvo. Focos: Healer.

90 base healing; affects allies. Cast time 0.45s.

90 de cura base; afeta aliados. Conjuração de 0.45s.

Custo: 24; Recarga (s): 5; Alcance (m): 14; Raio (m): 2.6; Cura base: 90; Conjuração (s): 0.45; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Herbal Infusion / Infusão de Ervas

ID de conteúdo: `los.archer.marksmanship.healer_02`.

Modalidade: Alvo. Focos: Healer.

18 base healing; effect duration 7s; values per pulse, every 1s; affects allies.

18 de cura base; efeito durante 7s; valores por pulso, a cada 1s; afeta aliados.

Custo: 24; Recarga (s): 12; Alcance (m): 14; Raio (m): 2.6; Cura base: 18; Conjuração (s): 0; Duração (s): 7; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Medic Burst Arrow / Flecha de Socorro

ID de conteúdo: `los.archer.marksmanship.healer_03`.

Modalidade: Ponto no chão. Focos: Healer.

70 base healing; affects allies. Cast time 0.6s.

70 de cura base; afeta aliados. Conjuração de 0.6s.

Custo: 30; Recarga (s): 14; Alcance (m): 14; Raio (m): 3.5; Cura base: 70; Conjuração (s): 0.6; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Protective Poultice / Cataplasma Protetor

ID de conteúdo: `los.archer.marksmanship.healer_04`.

Modalidade: Alvo. Focos: Healer.

30 base healing; 60 base shield; effect duration 8s; affects allies.

30 de cura base; 60 de escudo base; efeito durante 8s; afeta aliados.

Custo: 12; Recarga (s): 15; Alcance (m): 14; Raio (m): 2.6; Cura base: 30; Escudo: 60; Conjuração (s): 0; Duração (s): 8; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Healing Camp / Acampamento Curativo

ID de conteúdo: `los.archer.marksmanship.healer_05`.

Modalidade: Zona persistente. Focos: Healer.

21 base healing; effect duration 9s; values per pulse, every 1s; affects allies. Cast time 0.6s.

21 de cura base; efeito durante 9s; valores por pulso, a cada 1s; afeta aliados. Conjuração de 0.6s.

Custo: 36; Recarga (s): 22; Alcance (m): 14; Raio (m): 2.6; Cura base: 21; Conjuração (s): 0.6; Duração (s): 9; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Emergency Remedy / Remédio de Emergência

ID de conteúdo: `los.archer.marksmanship.healer_06`.

Modalidade: Alvo. Focos: Healer.

140 base healing; affects allies. Cast time 0.45s.

140 de cura base; afeta aliados. Conjuração de 0.45s.

Custo: 40; Recarga (s): 22; Alcance (m): 14; Raio (m): 2.6; Cura base: 140; Conjuração (s): 0.45; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Purifying Salve Spray / Névoa Purificadora

ID de conteúdo: `los.archer.marksmanship.healer_07`.

Modalidade: Cone. Focos: Healer.

55 base healing; removes movement impairments; affects allies. Cast time 0.45s.

55 de cura base; remove impedimentos de movimento; afeta aliados. Conjuração de 0.45s.

Custo: 12; Recarga (s): 18; Alcance (m): 6.5; Raio (m): 2.6; Cura base: 55; Limpeza: True; Conjuração (s): 0.45; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Remedy Relay / Revezamento de Remédios

ID de conteúdo: `los.archer.marksmanship.healer_08`.

Modalidade: Alvo. Focos: Healer.

42 base healing; jumps to 3 additional targets; affects allies. Cast time 0.45s.

42 de cura base; salta para mais 3 alvos; afeta aliados. Conjuração de 0.45s.

Custo: 12; Recarga (s): 13; Alcance (m): 14; Raio (m): 2.6; Cura base: 42; Encadeamentos: 3; Conjuração (s): 0.45; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Sheltering Remedy / Remédio Protetor

ID de conteúdo: `los.archer.marksmanship.healer_09`.

Modalidade: Alvo. Focos: Healer.

35 base healing; effect duration 6s; reduces incoming damage 25%; affects allies.

35 de cura base; efeito durante 6s; reduz dano recebido em 25%; afeta aliados.

Custo: 12; Recarga (s): 20; Alcance (m): 14; Raio (m): 2.6; Cura base: 35; Mitigação: 0.25; Conjuração (s): 0; Duração (s): 6; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Field Apothecary / Boticário de Campo

ID de conteúdo: `los.archer.marksmanship.healer_10`.

Modalidade: Próprio personagem. Focos: Healer.

95 base healing; affects allies. Cast time 0.45s.

95 de cura base; afeta aliados. Conjuração de 0.45s.

Custo: 34; Recarga (s): 25; Alcance (m): 14; Raio (m): 4.5; Cura base: 95; Conjuração (s): 0.45; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Invigorating Tonic / Tônico Revigorante

ID de conteúdo: `los.archer.marksmanship.healer_11`.

Modalidade: Alvo. Focos: Healer.

25 base healing; restores 30 resource; affects allies.

25 de cura base; restaura 30 de recurso; afeta aliados.

Custo: 10; Recarga (s): 24; Alcance (m): 14; Raio (m): 2.6; Cura base: 25; Mana restaurada: 30; Conjuração (s): 0; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Renewing Thicket / Moita Renovadora

ID de conteúdo: `los.archer.marksmanship.healer_12`.

Modalidade: Zona persistente. Focos: Healer.

8 base damage; 16 base healing; effect duration 8s; values per pulse, every 1s; heals/protects allies and damages/controls enemies. Cast time 0.6s.

8 de dano base; 16 de cura base; efeito durante 8s; valores por pulso, a cada 1s; cura/protege aliados e causa dano/controle a inimigos. Conjuração de 0.6s.

Custo: 32; Recarga (s): 18; Alcance (m): 14; Raio (m): 3.2; Dano base: 8; Cura base: 16; Conjuração (s): 0.6; Duração (s): 8; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Ranger Aegis / Égide do Patrulheiro

ID de conteúdo: `los.archer.marksmanship.tank_01`.

Modalidade: Próprio personagem. Focos: Tank.

150 base shield; effect duration 10s; affects allies.

150 de escudo base; efeito durante 10s; afeta aliados.

Custo: 26; Recarga (s): 20; Alcance (m): 14; Raio (m): 0; Escudo: 150; Conjuração (s): 0; Duração (s): 10; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Challenging Signal / Sinal Desafiador

ID de conteúdo: `los.archer.marksmanship.tank_02`.

Modalidade: Próprio personagem. Focos: Tank.

20 base damage; forces hostile attention; affects enemies.

20 de dano base; provoca inimigos; afeta inimigos.

Custo: 12; Recarga (s): 12; Alcance (m): 14; Raio (m): 4.5; Dano base: 20; Provocação: True; Conjuração (s): 0; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Provoking Shot / Disparo Provocador

ID de conteúdo: `los.archer.marksmanship.tank_03`.

Modalidade: Alvo. Focos: Tank.

25 base damage; effect duration 4s; slows enemies 40%; forces hostile attention; affects enemies.

25 de dano base; efeito durante 4s; reduz movimento inimigo em 40%; provoca inimigos; afeta inimigos.

Custo: 12; Recarga (s): 9; Alcance (m): 14; Raio (m): 2.6; Dano base: 25; Lentidão: 0.4; Provocação: True; Conjuração (s): 0; Duração (s): 4; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Unyielding Stance / Postura Resoluta

ID de conteúdo: `los.archer.marksmanship.tank_04`.

Modalidade: Próprio personagem. Focos: Tank.

effect duration 6s; reduces incoming damage 45%; affects allies.

efeito durante 6s; reduz dano recebido em 45%; afeta aliados.

Custo: 24; Recarga (s): 24; Alcance (m): 14; Raio (m): 0; Mitigação: 0.45; Conjuração (s): 0; Duração (s): 6; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Repelling Shot / Disparo Repulsor

ID de conteúdo: `los.archer.marksmanship.tank_05`.

Modalidade: Cone. Focos: Tank.

35 base damage; affects enemies.

35 de dano base; afeta inimigos.

Custo: 12; Recarga (s): 13; Alcance (m): 4.5; Raio (m): 2.6; Dano base: 35; Empurrão: 520; Conjuração (s): 0; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Sheltering Camp / Acampamento Protetor

ID de conteúdo: `los.archer.marksmanship.tank_06`.

Modalidade: Zona persistente. Focos: Tank.

18 base shield; effect duration 8s; values per pulse, every 1s; affects allies. Cast time 0.6s.

18 de escudo base; efeito durante 8s; valores por pulso, a cada 1s; afeta aliados. Conjuração de 0.6s.

Custo: 30; Recarga (s): 20; Alcance (m): 14; Raio (m): 3.3; Escudo: 18; Conjuração (s): 0.6; Duração (s): 8; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Binding Field / Campo Aprisionador

ID de conteúdo: `los.archer.marksmanship.tank_07`.

Modalidade: Ponto no chão. Focos: Tank.

16 base damage; roots enemies for 3s; affects enemies. Cast time 0.6s.

16 de dano base; imobiliza inimigos por 3s; afeta inimigos. Conjuração de 0.6s.

Custo: 12; Recarga (s): 16; Alcance (m): 14; Raio (m): 3.5; Dano base: 16; Imobilização: 3; Conjuração (s): 0.6; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Guarding Arrow / Flecha Protetora

ID de conteúdo: `los.archer.marksmanship.tank_08`.

Modalidade: Alvo. Focos: Tank.

110 base shield; effect duration 7s; reduces incoming damage 20%; affects allies.

110 de escudo base; efeito durante 7s; reduz dano recebido em 20%; afeta aliados.

Custo: 12; Recarga (s): 18; Alcance (m): 14; Raio (m): 2.6; Escudo: 110; Mitigação: 0.2; Conjuração (s): 0; Duração (s): 7; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Survival Remedy / Remédio de Sobrevivência

ID de conteúdo: `los.archer.marksmanship.tank_09`.

Modalidade: Próprio personagem. Focos: Tank.

80 base healing; removes movement impairments; affects allies. Cast time 0.45s.

80 de cura base; remove impedimentos de movimento; afeta aliados. Conjuração de 0.45s.

Custo: 25; Recarga (s): 24; Alcance (m): 14; Raio (m): 0; Cura base: 80; Limpeza: True; Conjuração (s): 0.45; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Crippling Barrage / Barragem Incapacitante

ID de conteúdo: `los.archer.marksmanship.tank_10`.

Modalidade: Cone. Focos: Tank.

38 base damage; effect duration 5s; values per pulse, every 1s; slows enemies 65%; affects enemies.

38 de dano base; efeito durante 5s; valores por pulso, a cada 1s; reduz movimento inimigo em 65%; afeta inimigos.

Custo: 12; Recarga (s): 12; Alcance (m): 6.5; Raio (m): 2.6; Dano base: 38; Lentidão: 0.65; Conjuração (s): 0; Duração (s): 5; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Diversion Field / Campo de Distração

ID de conteúdo: `los.archer.marksmanship.tank_11`.

Modalidade: Zona persistente. Focos: Tank.

12 base damage; effect duration 7s; values per pulse, every 1s; forces hostile attention; affects enemies. Cast time 0.6s.

12 de dano base; efeito durante 7s; valores por pulso, a cada 1s; provoca inimigos; afeta inimigos. Conjuração de 0.6s.

Custo: 28; Recarga (s): 18; Alcance (m): 14; Raio (m): 3.5; Dano base: 12; Provocação: True; Conjuração (s): 0.6; Duração (s): 7; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

### Thorned Guard / Guarda Espinhosa

ID de conteúdo: `los.archer.marksmanship.tank_12`.

Modalidade: Próprio personagem. Focos: Tank.

50 base damage; 70 base shield; effect duration 6s; heals/protects allies and damages/controls enemies.

50 de dano base; 70 de escudo base; efeito durante 6s; cura/protege aliados e causa dano/controle a inimigos.

Custo: 28; Recarga (s): 16; Alcance (m): 14; Raio (m): 3; Dano base: 50; Escudo: 70; Conjuração (s): 0; Duração (s): 6; Velocidade (m/s): 32; Diâmetro do projétil (m): 0.12.

## Druida e Natureza

### Thorn Lance / Lança de Espinhos

ID de conteúdo: `los.druid.nature.dps_01`.

Modalidade: Projétil. Focos: DPS.

95 base damage; affects enemies. Cast time 0.85s.

95 de dano base; afeta inimigos. Conjuração de 0.85s.

Custo: 28; Recarga (s): 10; Alcance (m): 14; Raio (m): 0; Dano base: 95; Conjuração (s): 0.85; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Creeping Vines / Vinhas Rastejantes

ID de conteúdo: `los.druid.nature.dps_02`.

Modalidade: Alvo. Focos: DPS.

12 base damage; effect duration 7s; values per pulse, every 1s; affects enemies.

12 de dano base; efeito durante 7s; valores por pulso, a cada 1s; afeta inimigos.

Custo: 12; Recarga (s): 13; Alcance (m): 14; Raio (m): 2.6; Dano base: 12; Conjuração (s): 0; Duração (s): 7; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Falling Boulder / Rocha Cadente

ID de conteúdo: `los.druid.nature.dps_03`.

Modalidade: Ponto no chão. Focos: DPS.

85 base damage; affects enemies. Cast time 0.6s.

85 de dano base; afeta inimigos. Conjuração de 0.6s.

Custo: 30; Recarga (s): 16; Alcance (m): 14; Raio (m): 3.4; Dano base: 85; Conjuração (s): 0.6; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Thorn Volley / Salva de Espinhos

ID de conteúdo: `los.druid.nature.dps_04`.

Modalidade: Projétil. Focos: DPS.

24 base damage; 3 projectiles; affects enemies. Cast time 0.85s.

24 de dano base; 3 projéteis; afeta inimigos. Conjuração de 0.85s.

Custo: 12; Recarga (s): 9; Alcance (m): 14; Raio (m): 0; Dano base: 24; Disparos: 3; Conjuração (s): 0.85; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Briar Storm / Tempestade de Espinhos

ID de conteúdo: `los.druid.nature.dps_05`.

Modalidade: Zona persistente. Focos: DPS.

16 base damage; effect duration 8s; values per pulse, every 1s; affects enemies. Cast time 0.6s.

16 de dano base; efeito durante 8s; valores por pulso, a cada 1s; afeta inimigos. Conjuração de 0.6s.

Custo: 32; Recarga (s): 18; Alcance (m): 14; Raio (m): 3; Dano base: 16; Conjuração (s): 0.6; Duração (s): 8; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Predatory End / Fim Predatório

ID de conteúdo: `los.druid.nature.dps_06`.

Modalidade: Alvo. Focos: DPS.

68 base damage; 50% stronger below 30% target health; affects enemies.

68 de dano base; 50% mais forte abaixo de 30% de vida do alvo; afeta inimigos.

Custo: 12; Recarga (s): 12; Alcance (m): 14; Raio (m): 2.6; Dano base: 68; Execução: True; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Gale Breath / Sopro de Vendaval

ID de conteúdo: `los.druid.nature.dps_07`.

Modalidade: Cone. Focos: DPS.

55 base damage; affects enemies.

55 de dano base; afeta inimigos.

Custo: 12; Recarga (s): 11; Alcance (m): 6.5; Raio (m): 2.6; Dano base: 55; Empurrão: 380; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Leaping Thorn / Espinho Saltitante

ID de conteúdo: `los.druid.nature.dps_08`.

Modalidade: Alvo. Focos: DPS.

36 base damage; jumps to 3 additional targets; affects enemies.

36 de dano base; salta para mais 3 alvos; afeta inimigos.

Custo: 12; Recarga (s): 14; Alcance (m): 14; Raio (m): 2.6; Dano base: 36; Encadeamentos: 3; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Root Seal / Selo de Raízes

ID de conteúdo: `los.druid.nature.dps_09`.

Modalidade: Ponto no chão. Focos: DPS.

35 base damage; roots enemies for 2s; affects enemies. Cast time 0.6s.

35 de dano base; imobiliza inimigos por 2s; afeta inimigos. Conjuração de 0.6s.

Custo: 12; Recarga (s): 14; Alcance (m): 14; Raio (m): 2.6; Dano base: 35; Imobilização: 2; Conjuração (s): 0.6; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Wild Torrent / Torrente Selvagem

ID de conteúdo: `los.druid.nature.dps_10`.

Modalidade: Cone. Focos: DPS.

17 base damage; effect duration 4s; values per pulse, every 0.8s; affects enemies.

17 de dano base; efeito durante 4s; valores por pulso, a cada 0.8s; afeta inimigos.

Custo: 26; Recarga (s): 15; Alcance (m): 7; Raio (m): 2.6; Dano base: 17; Conjuração (s): 0; Duração (s): 4; Intervalo (s): 0.8; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Chilling Spore / Esporo Entorpecente

ID de conteúdo: `los.druid.nature.dps_11`.

Modalidade: Projétil. Focos: DPS.

40 base damage; effect duration 3s; slows enemies 60%; affects enemies. Cast time 0.85s.

40 de dano base; efeito durante 3s; reduz movimento inimigo em 60%; afeta inimigos. Conjuração de 0.85s.

Custo: 12; Recarga (s): 12; Alcance (m): 14; Raio (m): 0; Dano base: 40; Lentidão: 0.6; Conjuração (s): 0.85; Duração (s): 3; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Wrath of the Grove / Ira do Bosque

ID de conteúdo: `los.druid.nature.dps_12`.

Modalidade: Próprio personagem. Focos: DPS.

115 base damage; affects enemies.

115 de dano base; afeta inimigos.

Custo: 38; Recarga (s): 28; Alcance (m): 14; Raio (m): 4.2; Dano base: 115; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Regrowth / Recrescimento

ID de conteúdo: `los.druid.nature.healer_01`.

Modalidade: Alvo. Focos: Healer.

90 base healing; affects allies. Cast time 0.45s.

90 de cura base; afeta aliados. Conjuração de 0.45s.

Custo: 24; Recarga (s): 5; Alcance (m): 14; Raio (m): 2.6; Cura base: 90; Conjuração (s): 0.45; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Lifebloom / Flor da Vida

ID de conteúdo: `los.druid.nature.healer_02`.

Modalidade: Alvo. Focos: Healer.

18 base healing; effect duration 7s; values per pulse, every 1s; affects allies.

18 de cura base; efeito durante 7s; valores por pulso, a cada 1s; afeta aliados.

Custo: 24; Recarga (s): 12; Alcance (m): 14; Raio (m): 2.6; Cura base: 18; Conjuração (s): 0; Duração (s): 7; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Blooming Ground / Solo Florido

ID de conteúdo: `los.druid.nature.healer_03`.

Modalidade: Ponto no chão. Focos: Healer.

70 base healing; affects allies. Cast time 0.6s.

70 de cura base; afeta aliados. Conjuração de 0.6s.

Custo: 30; Recarga (s): 14; Alcance (m): 14; Raio (m): 3.5; Cura base: 70; Conjuração (s): 0.6; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Living Bark / Casca Viva

ID de conteúdo: `los.druid.nature.healer_04`.

Modalidade: Alvo. Focos: Healer.

30 base healing; 60 base shield; effect duration 8s; affects allies.

30 de cura base; 60 de escudo base; efeito durante 8s; afeta aliados.

Custo: 12; Recarga (s): 15; Alcance (m): 14; Raio (m): 2.6; Cura base: 30; Escudo: 60; Conjuração (s): 0; Duração (s): 8; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Sanctuary Grove / Bosque Santuário

ID de conteúdo: `los.druid.nature.healer_05`.

Modalidade: Zona persistente. Focos: Healer.

21 base healing; effect duration 9s; values per pulse, every 1s; affects allies. Cast time 0.6s.

21 de cura base; efeito durante 9s; valores por pulso, a cada 1s; afeta aliados. Conjuração de 0.6s.

Custo: 36; Recarga (s): 22; Alcance (m): 14; Raio (m): 2.6; Cura base: 21; Conjuração (s): 0.6; Duração (s): 9; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Ancient Restoration / Restauração Ancestral

ID de conteúdo: `los.druid.nature.healer_06`.

Modalidade: Alvo. Focos: Healer.

140 base healing; affects allies. Cast time 0.45s.

140 de cura base; afeta aliados. Conjuração de 0.45s.

Custo: 40; Recarga (s): 22; Alcance (m): 14; Raio (m): 2.6; Cura base: 140; Conjuração (s): 0.45; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Purifying Wind / Vento Purificador

ID de conteúdo: `los.druid.nature.healer_07`.

Modalidade: Cone. Focos: Healer.

55 base healing; removes movement impairments; affects allies. Cast time 0.45s.

55 de cura base; remove impedimentos de movimento; afeta aliados. Conjuração de 0.45s.

Custo: 12; Recarga (s): 18; Alcance (m): 6.5; Raio (m): 2.6; Cura base: 55; Limpeza: True; Conjuração (s): 0.45; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Life Chain / Corrente de Vida

ID de conteúdo: `los.druid.nature.healer_08`.

Modalidade: Alvo. Focos: Healer.

42 base healing; jumps to 3 additional targets; affects allies. Cast time 0.45s.

42 de cura base; salta para mais 3 alvos; afeta aliados. Conjuração de 0.45s.

Custo: 12; Recarga (s): 13; Alcance (m): 14; Raio (m): 2.6; Cura base: 42; Encadeamentos: 3; Conjuração (s): 0.45; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Protective Growth / Crescimento Protetor

ID de conteúdo: `los.druid.nature.healer_09`.

Modalidade: Alvo. Focos: Healer.

35 base healing; effect duration 6s; reduces incoming damage 25%; affects allies.

35 de cura base; efeito durante 6s; reduz dano recebido em 25%; afeta aliados.

Custo: 12; Recarga (s): 20; Alcance (m): 14; Raio (m): 2.6; Cura base: 35; Mitigação: 0.25; Conjuração (s): 0; Duração (s): 6; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Verdant Renewal / Renovação Verdejante

ID de conteúdo: `los.druid.nature.healer_10`.

Modalidade: Próprio personagem. Focos: Healer.

95 base healing; affects allies. Cast time 0.45s.

95 de cura base; afeta aliados. Conjuração de 0.45s.

Custo: 34; Recarga (s): 25; Alcance (m): 14; Raio (m): 4.5; Cura base: 95; Conjuração (s): 0.45; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Essence Spring / Fonte de Essência

ID de conteúdo: `los.druid.nature.healer_11`.

Modalidade: Alvo. Focos: Healer.

25 base healing; restores 30 resource; affects allies.

25 de cura base; restaura 30 de recurso; afeta aliados.

Custo: 10; Recarga (s): 24; Alcance (m): 14; Raio (m): 2.6; Cura base: 25; Mana restaurada: 30; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Balanced Grove / Bosque do Equilíbrio

ID de conteúdo: `los.druid.nature.healer_12`.

Modalidade: Zona persistente. Focos: Healer.

8 base damage; 16 base healing; effect duration 8s; values per pulse, every 1s; heals/protects allies and damages/controls enemies. Cast time 0.6s.

8 de dano base; 16 de cura base; efeito durante 8s; valores por pulso, a cada 1s; cura/protege aliados e causa dano/controle a inimigos. Conjuração de 0.6s.

Custo: 32; Recarga (s): 18; Alcance (m): 14; Raio (m): 3.2; Dano base: 8; Cura base: 16; Conjuração (s): 0.6; Duração (s): 8; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Ancient Bark / Casca Ancestral

ID de conteúdo: `los.druid.nature.tank_01`.

Modalidade: Próprio personagem. Focos: Tank.

150 base shield; effect duration 10s; affects allies.

150 de escudo base; efeito durante 10s; afeta aliados.

Custo: 26; Recarga (s): 20; Alcance (m): 14; Raio (m): 0; Escudo: 150; Conjuração (s): 0; Duração (s): 10; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Guardian Roar / Rugido Guardião

ID de conteúdo: `los.druid.nature.tank_02`.

Modalidade: Próprio personagem. Focos: Tank.

20 base damage; forces hostile attention; affects enemies.

20 de dano base; provoca inimigos; afeta inimigos.

Custo: 12; Recarga (s): 12; Alcance (m): 14; Raio (m): 4.5; Dano base: 20; Provocação: True; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Wild Challenge / Desafio Selvagem

ID de conteúdo: `los.druid.nature.tank_03`.

Modalidade: Alvo. Focos: Tank.

25 base damage; effect duration 4s; slows enemies 40%; forces hostile attention; affects enemies.

25 de dano base; efeito durante 4s; reduz movimento inimigo em 40%; provoca inimigos; afeta inimigos.

Custo: 12; Recarga (s): 9; Alcance (m): 14; Raio (m): 2.6; Dano base: 25; Lentidão: 0.4; Provocação: True; Conjuração (s): 0; Duração (s): 4; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Heartwood / Cerne Ancestral

ID de conteúdo: `los.druid.nature.tank_04`.

Modalidade: Próprio personagem. Focos: Tank.

effect duration 6s; reduces incoming damage 45%; affects allies.

efeito durante 6s; reduz dano recebido em 45%; afeta aliados.

Custo: 24; Recarga (s): 24; Alcance (m): 14; Raio (m): 0; Mitigação: 0.45; Conjuração (s): 0; Duração (s): 6; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Repelling Gale / Vendaval Repulsor

ID de conteúdo: `los.druid.nature.tank_05`.

Modalidade: Cone. Focos: Tank.

35 base damage; affects enemies.

35 de dano base; afeta inimigos.

Custo: 12; Recarga (s): 13; Alcance (m): 4.5; Raio (m): 2.6; Dano base: 35; Empurrão: 520; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Guardian Grove / Bosque Guardião

ID de conteúdo: `los.druid.nature.tank_06`.

Modalidade: Zona persistente. Focos: Tank.

18 base shield; effect duration 8s; values per pulse, every 1s; affects allies. Cast time 0.6s.

18 de escudo base; efeito durante 8s; valores por pulso, a cada 1s; afeta aliados. Conjuração de 0.6s.

Custo: 30; Recarga (s): 20; Alcance (m): 14; Raio (m): 3.3; Escudo: 18; Conjuração (s): 0.6; Duração (s): 8; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Grasping Roots / Raízes Agarradoras

ID de conteúdo: `los.druid.nature.tank_07`.

Modalidade: Ponto no chão. Focos: Tank.

16 base damage; roots enemies for 3s; affects enemies. Cast time 0.6s.

16 de dano base; imobiliza inimigos por 3s; afeta inimigos. Conjuração de 0.6s.

Custo: 12; Recarga (s): 16; Alcance (m): 14; Raio (m): 3.5; Dano base: 16; Imobilização: 3; Conjuração (s): 0.6; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Sheltering Branch / Ramo Protetor

ID de conteúdo: `los.druid.nature.tank_08`.

Modalidade: Alvo. Focos: Tank.

110 base shield; effect duration 7s; reduces incoming damage 20%; affects allies.

110 de escudo base; efeito durante 7s; reduz dano recebido em 20%; afeta aliados.

Custo: 12; Recarga (s): 18; Alcance (m): 14; Raio (m): 2.6; Escudo: 110; Mitigação: 0.2; Conjuração (s): 0; Duração (s): 7; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Wild Recovery / Recuperação Selvagem

ID de conteúdo: `los.druid.nature.tank_09`.

Modalidade: Próprio personagem. Focos: Tank.

80 base healing; removes movement impairments; affects allies. Cast time 0.45s.

80 de cura base; remove impedimentos de movimento; afeta aliados. Conjuração de 0.45s.

Custo: 25; Recarga (s): 24; Alcance (m): 14; Raio (m): 0; Cura base: 80; Limpeza: True; Conjuração (s): 0.45; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Slowing Spores / Esporos Entorpecentes

ID de conteúdo: `los.druid.nature.tank_10`.

Modalidade: Cone. Focos: Tank.

38 base damage; effect duration 5s; values per pulse, every 1s; slows enemies 65%; affects enemies.

38 de dano base; efeito durante 5s; valores por pulso, a cada 1s; reduz movimento inimigo em 65%; afeta inimigos.

Custo: 12; Recarga (s): 12; Alcance (m): 6.5; Raio (m): 2.6; Dano base: 38; Lentidão: 0.65; Conjuração (s): 0; Duração (s): 5; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Territorial Grove / Bosque Territorial

ID de conteúdo: `los.druid.nature.tank_11`.

Modalidade: Zona persistente. Focos: Tank.

12 base damage; effect duration 7s; values per pulse, every 1s; forces hostile attention; affects enemies. Cast time 0.6s.

12 de dano base; efeito durante 7s; valores por pulso, a cada 1s; provoca inimigos; afeta inimigos. Conjuração de 0.6s.

Custo: 28; Recarga (s): 18; Alcance (m): 14; Raio (m): 3.5; Dano base: 12; Provocação: True; Conjuração (s): 0.6; Duração (s): 7; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Thorn Aegis / Égide de Espinhos

ID de conteúdo: `los.druid.nature.tank_12`.

Modalidade: Próprio personagem. Focos: Tank.

50 base damage; 70 base shield; effect duration 6s; heals/protects allies and damages/controls enemies.

50 de dano base; 70 de escudo base; efeito durante 6s; cura/protege aliados e causa dano/controle a inimigos.

Custo: 28; Recarga (s): 16; Alcance (m): 14; Raio (m): 3; Dano base: 50; Escudo: 70; Conjuração (s): 0; Duração (s): 6; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

## Druida e Natureza — Urso

### Bear: Maul / Urso: Dilacerar

ID de conteúdo: `los.druid.nature.bear_01`.

Modalidade: Alvo. Focos: DPS, Healer, Tank.

55 base damage; affects enemies.

55 de dano base; afeta inimigos.

Custo: 12; Recarga (s): 2; Alcance (m): 2.6; Raio (m): 2.6; Dano base: 55; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Bear: Raking Sweep / Urso: Varredura de Garras

ID de conteúdo: `los.druid.nature.bear_02`.

Modalidade: Cone. Focos: DPS, Healer, Tank.

34 base damage; affects enemies.

34 de dano base; afeta inimigos.

Custo: 12; Recarga (s): 6; Alcance (m): 3.3; Raio (m): 2.6; Dano base: 34; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Bear: Restoring Roar / Urso: Rugido Restaurador

ID de conteúdo: `los.druid.nature.bear_03`.

Modalidade: Próprio personagem. Focos: DPS, Healer, Tank.

55 base healing; affects allies.

55 de cura base; afeta aliados.

Custo: 12; Recarga (s): 14; Alcance (m): 14; Raio (m): 4; Cura base: 55; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Bear: Wild Guard / Urso: Guarda Selvagem

ID de conteúdo: `los.druid.nature.bear_04`.

Modalidade: Próprio personagem. Focos: DPS, Healer, Tank.

95 base shield; effect duration 8s; affects allies.

95 de escudo base; efeito durante 8s; afeta aliados.

Custo: 12; Recarga (s): 18; Alcance (m): 14; Raio (m): 0; Escudo: 95; Conjuração (s): 0; Duração (s): 8; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Bear: Pounce / Urso: Bote

ID de conteúdo: `los.druid.nature.bear_05`.

Modalidade: Avanço. Focos: DPS, Healer, Tank.

30 base damage; swept forward movement; affects enemies.

30 de dano base; avanço com colisão; afeta inimigos.

Custo: 12; Recarga (s): 10; Alcance (m): 4.2; Raio (m): 2.6; Dano base: 30; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Bear: Binding Claws / Urso: Garras Aprisionadoras

ID de conteúdo: `los.druid.nature.bear_06`.

Modalidade: Alvo. Focos: DPS, Healer, Tank.

18 base damage; roots enemies for 2s; affects enemies.

18 de dano base; imobiliza inimigos por 2s; afeta inimigos.

Custo: 12; Recarga (s): 12; Alcance (m): 2.6; Raio (m): 2.6; Dano base: 18; Imobilização: 2; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Bear: Life Scent / Urso: Aroma Vital

ID de conteúdo: `los.druid.nature.bear_07`.

Modalidade: Zona persistente. Focos: DPS, Healer, Tank.

8 base damage; 12 base healing; effect duration 6s; values per pulse, every 1s; heals/protects allies and damages/controls enemies.

8 de dano base; 12 de cura base; efeito durante 6s; valores por pulso, a cada 1s; cura/protege aliados e causa dano/controle a inimigos.

Custo: 12; Recarga (s): 17; Alcance (m): 14; Raio (m): 2.6; Dano base: 8; Cura base: 12; Conjuração (s): 0; Duração (s): 6; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Bear: Territorial Roar / Urso: Rugido Territorial

ID de conteúdo: `los.druid.nature.bear_08`.

Modalidade: Próprio personagem. Focos: DPS, Healer, Tank.

15 base damage; forces hostile attention; affects enemies.

15 de dano base; provoca inimigos; afeta inimigos.

Custo: 12; Recarga (s): 12; Alcance (m): 14; Raio (m): 4.5; Dano base: 15; Provocação: True; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Bear: Wild Mending / Urso: Cura Selvagem

ID de conteúdo: `los.druid.nature.bear_09`.

Modalidade: Próprio personagem. Focos: DPS, Healer, Tank.

14 base healing; effect duration 5s; values per pulse, every 1s; affects allies.

14 de cura base; efeito durante 5s; valores por pulso, a cada 1s; afeta aliados.

Custo: 12; Recarga (s): 15; Alcance (m): 14; Raio (m): 0; Cura base: 14; Conjuração (s): 0; Duração (s): 5; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Bear: Return to Caster / Urso: Retornar à Forma Original

ID de conteúdo: `los.druid.nature.bear_10`.

Modalidade: Retornar à forma original. Focos: DPS, Healer, Tank.

transforms and replaces the action bar; no free healing.

transforma e troca a barra; sem cura gratuita.

Custo: 0; Recarga (s): 0; Alcance (m): 14; Raio (m): 2.6; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

## Druida e Natureza — Felino

### Feline: Maul / Felino: Dilacerar

ID de conteúdo: `los.druid.nature.cat_01`.

Modalidade: Alvo. Focos: DPS, Healer, Tank.

55 base damage; affects enemies.

55 de dano base; afeta inimigos.

Custo: 12; Recarga (s): 2; Alcance (m): 2.6; Raio (m): 2.6; Dano base: 55; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Feline: Raking Sweep / Felino: Varredura de Garras

ID de conteúdo: `los.druid.nature.cat_02`.

Modalidade: Cone. Focos: DPS, Healer, Tank.

34 base damage; affects enemies.

34 de dano base; afeta inimigos.

Custo: 12; Recarga (s): 6; Alcance (m): 3.3; Raio (m): 2.6; Dano base: 34; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Feline: Restoring Roar / Felino: Rugido Restaurador

ID de conteúdo: `los.druid.nature.cat_03`.

Modalidade: Próprio personagem. Focos: DPS, Healer, Tank.

55 base healing; affects allies.

55 de cura base; afeta aliados.

Custo: 12; Recarga (s): 14; Alcance (m): 14; Raio (m): 4; Cura base: 55; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Feline: Wild Guard / Felino: Guarda Selvagem

ID de conteúdo: `los.druid.nature.cat_04`.

Modalidade: Próprio personagem. Focos: DPS, Healer, Tank.

95 base shield; effect duration 8s; affects allies.

95 de escudo base; efeito durante 8s; afeta aliados.

Custo: 12; Recarga (s): 18; Alcance (m): 14; Raio (m): 0; Escudo: 95; Conjuração (s): 0; Duração (s): 8; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Feline: Pounce / Felino: Bote

ID de conteúdo: `los.druid.nature.cat_05`.

Modalidade: Avanço. Focos: DPS, Healer, Tank.

30 base damage; swept forward movement; affects enemies.

30 de dano base; avanço com colisão; afeta inimigos.

Custo: 12; Recarga (s): 10; Alcance (m): 4.2; Raio (m): 2.6; Dano base: 30; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Feline: Binding Claws / Felino: Garras Aprisionadoras

ID de conteúdo: `los.druid.nature.cat_06`.

Modalidade: Alvo. Focos: DPS, Healer, Tank.

18 base damage; roots enemies for 2s; affects enemies.

18 de dano base; imobiliza inimigos por 2s; afeta inimigos.

Custo: 12; Recarga (s): 12; Alcance (m): 2.6; Raio (m): 2.6; Dano base: 18; Imobilização: 2; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Feline: Life Scent / Felino: Aroma Vital

ID de conteúdo: `los.druid.nature.cat_07`.

Modalidade: Zona persistente. Focos: DPS, Healer, Tank.

8 base damage; 12 base healing; effect duration 6s; values per pulse, every 1s; heals/protects allies and damages/controls enemies.

8 de dano base; 12 de cura base; efeito durante 6s; valores por pulso, a cada 1s; cura/protege aliados e causa dano/controle a inimigos.

Custo: 12; Recarga (s): 17; Alcance (m): 14; Raio (m): 2.6; Dano base: 8; Cura base: 12; Conjuração (s): 0; Duração (s): 6; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Feline: Territorial Roar / Felino: Rugido Territorial

ID de conteúdo: `los.druid.nature.cat_08`.

Modalidade: Próprio personagem. Focos: DPS, Healer, Tank.

15 base damage; forces hostile attention; affects enemies.

15 de dano base; provoca inimigos; afeta inimigos.

Custo: 12; Recarga (s): 12; Alcance (m): 14; Raio (m): 4.5; Dano base: 15; Provocação: True; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Feline: Wild Mending / Felino: Cura Selvagem

ID de conteúdo: `los.druid.nature.cat_09`.

Modalidade: Próprio personagem. Focos: DPS, Healer, Tank.

14 base healing; effect duration 5s; values per pulse, every 1s; affects allies.

14 de cura base; efeito durante 5s; valores por pulso, a cada 1s; afeta aliados.

Custo: 12; Recarga (s): 15; Alcance (m): 14; Raio (m): 0; Cura base: 14; Conjuração (s): 0; Duração (s): 5; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.

### Feline: Return to Caster / Felino: Retornar à Forma Original

ID de conteúdo: `los.druid.nature.cat_10`.

Modalidade: Retornar à forma original. Focos: DPS, Healer, Tank.

transforms and replaces the action bar; no free healing.

transforma e troca a barra; sem cura gratuita.

Custo: 0; Recarga (s): 0; Alcance (m): 14; Raio (m): 2.6; Conjuração (s): 0; Velocidade (m/s): 17.5; Diâmetro do projétil (m): 0.2.
