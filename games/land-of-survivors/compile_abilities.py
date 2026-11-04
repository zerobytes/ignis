#!/usr/bin/env python3
"""Compile the preserved design into explicit native gameplay data, never runtime name heuristics."""
import json
import re
from pathlib import Path
ROOT = Path(__file__).resolve().parent
DESIGN = ROOT.parents[1] / 'docs/games/land-of-survivors/habilidades.md'
KEYS = {'Custo':'cost','Recarga (s)':'cooldown','Alcance (m)':'range','Raio (m)':'radius',
'Dano base':'damage','Cura base':'heal','Escudo':'shield','Conjuração (s)':'cast',
'Duração (s)':'duration','Intervalo (s)':'interval','Dano periódico':'burn',
'Velocidade (m/s)':'speed','Diâmetro do projétil (m)':'diameter','Imobilização':'root',
'Empurrão':'push','Limpeza':'cleanse','Lentidão':'slow','Mana restaurada':'mana',
'Disparos':'shots','Execução':'execute','Encadeamentos':'chain','Mitigação':'mitigation','Provocação':'taunt'}
MODES = {'Projétil':'projectile','Zona persistente':'zone','Avanço':'dash','Próprio personagem':'self_actor',
'Alvo':'target','Cone':'cone','Ponto no chão':'ground','Retornar à forma original':'humanoid',
'Transformar em Urso':'bear','Transformar em Felino':'feline'}
def compile_catalog():
    entries=[]
    for block in re.split(r'(?m)^### ', DESIGN.read_text())[1:]:
        paragraphs=block.split('\n\n')
        ident=re.search(r'ID de conteúdo: `([^`]+)`',block)[1]
        mode,focus=re.search(r'Modalidade: ([^.]+)\. Focos: ([^.]+)',block).groups()
        numbers={}
        for k,v in re.findall(r'([^;:]+): ([^;]+)',next(l for l in block.splitlines() if l.startswith('Custo:'))):
            key=KEYS[k.strip()]; value=v.strip().rstrip('.')
            numbers[key]=value=='True' if value in ('True','False') else float(value)
        en,pt=paragraphs[3:5]
        names=paragraphs[0].split(' / ',1)
        parts=ident.split('.')
        form='bear' if re.search(r'\.bear_\d+',ident) else 'feline' if re.search(r'\.cat_\d+',ident) else 'humanoid'
        periodic='values per pulse' in en
        effects=[]
        duration=numbers.get('duration',0)
        for field,kind,relation in [('damage','damage','enemy'),('heal','heal','ally'),('shield','shield','ally'),
                    ('mitigation','mitigation','ally'),('slow','slow','enemy'),('root','root','enemy'),
                    ('cleanse','cleanse','ally'),('mana','mana','ally'),('taunt','taunt','enemy'),('push','push','enemy')]:
            if numbers.get(field):
                amount=float(numbers[field])
                seconds=duration
                if field=='root': seconds=amount; amount=1
                if field=='taunt': seconds=duration or 3; amount=1
                if field=='push': amount/=100; seconds=.25
                if field in ('shield','mitigation','slow'): seconds=duration or 6
                effects.append(dict(kind=kind,relation=relation,amount=amount,seconds=seconds))
        delivery=MODES[mode]
        if delivery in ('bear','feline','humanoid'):
            effects=[dict(kind='form',relation='self_actor',amount=0,seconds=0,form=delivery)]
            delivery='self_actor'
        elif delivery=='dash':
            effects.append(dict(kind='move',relation='self_actor',amount=numbers['range'],seconds=0))
        entry=dict(id=ident,name_en=names[0],name_pt=names[1],description_en=en,description_pt=pt,
            class_id=parts[1],specialization=parts[2],form=form,
            focuses=[f.lower() for f in focus.split(', ')],delivery=delivery,
            cost=numbers['cost'],cooldown=numbers['cooldown'],cast=numbers['cast'],range=numbers['range'],
            radius=numbers['radius'],speed=numbers['speed'],diameter=numbers['diameter'],
            duration=duration,interval=numbers.get('interval',1),periodic=periodic,
            shots=int(numbers.get('shots',1)),chain=int(numbers['chain'])+1 if 'chain' in numbers else 1,
            execute=numbers.get('execute',False),burn=numbers.get('burn',0),effects=effects)
        assert effects,ident
        entries.append(entry)
    assert len(entries)==284 and len({e['id'] for e in entries})==284
    return dict(version=1,abilities=entries)
if __name__=='__main__':
    path=ROOT/'content/abilities.json'; path.write_text(json.dumps(compile_catalog(),ensure_ascii=False,indent=2)+'\n')
    print(f'{path}: 284 executable definitions')
