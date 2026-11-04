#!/usr/bin/env python3
"""Terminal/MCP entry point. Combat and physics execute exclusively in the native Rust game."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shlex
import subprocess
import sys
import tempfile
ROOT=Path(__file__).resolve().parent
ENGINE=ROOT.parents[1]
BINARY=ENGINE/'target/release/examples/land_of_survivors'
OUT=ROOT/'artifacts/abilities'
SPECS={'fire':'mage','frost':'mage','aether':'mage','arms':'warrior','marksmanship':'archer','nature':'druid'}

def save_session(path, data):
    temporary=None
    try:
        with tempfile.NamedTemporaryFile(dir=path.parent,prefix='.game-save-',suffix='.tmp',delete=False) as f:
            temporary=Path(f.name);f.write(data);f.flush();os.fsync(f.fileno())
        temporary.replace(path)
    finally:
        if temporary is not None:temporary.unlink(missing_ok=True)

def build():
    sources=[Path(__file__),ENGINE/'examples/land_of_survivors.rs',ROOT/'content/abilities.json',* (ROOT/'src').glob('*.rs'),ENGINE/'src/physics/gameplay.rs']
    if not BINARY.exists() or any(p.stat().st_mtime>BINARY.stat().st_mtime for p in sources):
        subprocess.run(['cargo','build','--locked','--release','--example','land_of_survivors'],cwd=ENGINE,check=True)

class Client:
    def __init__(self):
        self.p=subprocess.Popen([str(BINARY)],stdin=subprocess.PIPE,stdout=subprocess.PIPE,text=True)
    def call(self,**command):
        self.p.stdin.write(json.dumps(command)+'\n');self.p.stdin.flush()
        line=self.p.stdout.readline()
        if not line: raise RuntimeError('Native game process stopped')
        answer=json.loads(line)
        if 'error' in answer: raise ValueError(answer['error'])
        return answer['result']
    def close(self):
        self.p.stdin.close();self.p.wait()

def brief(state,catalog):
    print(f"Tick {state['state']['tick']} · projéteis {len(state['state']['shots'])} · efeitos periódicos {len(state['state']['pulses'])}")
    for name,f in state['state']['fighters'].items():
        print(f"{name}: HP {f['health']:.1f}/{f['max_health']:.0f} · mana {f['mana']:.1f} · {f['form']} · posição {state['positions'][name]}")
    for slot,ident in enumerate(state['bar'],1):
        a=catalog[ident];print(f"{slot:2}: {a['name_pt']} ({ident})")

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--verify',action='store_true',help='Exercise all284 abilities against native physical actors')
    p.add_argument('--mcp',action='store_true',help='Run the native MCP/JSON-lines server directly')
    p.add_argument('--build',action='store_true',help='Build the current game executable and exit')
    a=p.parse_args();build()
    if a.build:return
    if a.mcp:
        raise SystemExit(subprocess.call([str(BINARY)]))
    c=Client()
    try:
        if a.verify:
            data=c.call(op='verify');OUT.mkdir(parents=True,exist_ok=True)
            data['binary_sha256']=hashlib.sha256(BINARY.read_bytes()).hexdigest()
            (OUT/'verification.json').write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n')
            print(f"{data['verified']}/284 habilidades executadas no Ignis; relatório: {OUT/'verification.json'}")
            return
        catalog={x['id']:x for x in c.call(op='catalog')['abilities']}
        state=c.call(op='inspect');brief(state,catalog);pending=None
        print('Comandos: list, bar, cast SLOT|ID [ALVO], point SLOT X Z, confirm, cancel, step SEGUNDOS, move ATOR X Z SEGUNDOS, build ESPECIALIZAÇÃO FOCO, equip ID... (10), new ESPECIALIZAÇÃO FOCO, ai ATOR on|off, save, load, quit.')
        while True:
            try: words=shlex.split(input('LOS> '))
            except EOFError:break
            if not words:continue
            cmd,*args=words
            try:
                if cmd in ('quit','exit'):break
                if cmd=='list':
                    for ident in state['available']: print(ident,'—',catalog[ident]['name_pt'])
                    continue
                if cmd=='bar':brief(state,catalog);continue
                if cmd in ('cast','point'):
                    ident=state['bar'][int(args[0])-1] if args[0].isdigit() else args[0]
                    ability=catalog[ident]
                    if cmd=='point':pending=(ident,{'point':[float(args[1]),0,float(args[2])]});print('Seleção pendente: confirm aplica, cancel descarta sem custo.');continue
                    target=args[1] if len(args)>1 else 'enemy' if any(e['relation']=='enemy' for e in ability['effects']) else 'ally'
                    aim={'target':target,'direction':[0,0,-1]}
                    if ability['delivery'] in ('ground','zone'):
                        pos=state['positions'][target];pending=(ident,{'point':[pos[0],0,pos[2]]});print('Seleção pendente: confirm aplica, cancel descarta sem custo.');continue
                    state=c.call(op='cast',actor='player',ability=ident,aim=aim)
                elif cmd=='confirm':
                    if pending is None:raise ValueError('Nenhuma seleção de chão pendente')
                    state=c.call(op='cast',actor='player',ability=pending[0],aim=pending[1]);pending=None
                elif cmd=='cancel':
                    if pending is not None:pending=None;print('Seleção descartada.');continue
                    state=c.call(op='cancel',actor='player')
                elif cmd=='step':state=c.call(op='step',ticks=max(1,round(float(args[0])*60)))
                elif cmd=='move':
                    state=c.call(op='move',actor=args[0],input=[float(args[1]),float(args[2])])
                    state=c.call(op='step',ticks=max(1,round(float(args[3])*60)))
                    state=c.call(op='move',actor=args[0],input=[0,0])
                elif cmd in ('new','build'):
                    spec,focus=args
                    if cmd=='new':state=c.call(op='new',class_id=SPECS[spec],specialization=spec,focus=focus)
                    else:state=c.call(op='build',actor='player',specialization=spec,focus=focus)
                    pending=None
                elif cmd=='equip':state=c.call(op='equip',actor='player',slots=args)
                elif cmd=='ai':state=c.call(op='ai',actor=args[0],enabled=args[1]=='on')
                elif cmd=='save':
                    data=c.call(op='save');OUT.mkdir(parents=True,exist_ok=True);save_session(OUT/'current.ignis-game',bytes(data['save']));print('Sessão salva.');continue
                elif cmd=='load':
                    state=c.call(op='load',save=list((OUT/'current.ignis-game').read_bytes()));pending=None
                else:raise ValueError('Comando desconhecido')
                brief(state,catalog)
                for event in state['state']['events'][-8:]:print(event)
            except (ValueError,KeyError,IndexError,OSError) as e: print(f'Não aplicado: {e}')
    finally:c.close()
if __name__=='__main__':main()
