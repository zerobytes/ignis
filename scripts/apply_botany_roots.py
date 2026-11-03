#!/usr/bin/env python3
"""Reapply root recipes with the native authoring API; replace output only on success."""
import argparse,json
from pathlib import Path
from verify_lab import Client,ROOT
import botany_math as bm

def generate(recipe,output,systems,binary=None):
    directory=ROOT/'artifacts/botany-roots';directory.mkdir(parents=True,exist_ok=True)
    candidate=directory/f"{recipe['prefix']}-generation.ignis-botany"
    output=Path(output).resolve();relative=output.relative_to(ROOT).as_posix()
    bm.atomic_json(candidate,recipe)
    client=Client(binary or ROOT/'target/release/ignis',log_path=directory/'generation-runtime.log')
    try:
        client.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'root-generator','version':'1'}});client.request('notifications/initialized',notification=True)
        assert 'set_roots' in json.dumps(client.request('tools/list')['result']),'Build does not expose root generation'
        client.call('generate',recipe={'width':20,'depth':20,'seed':198,'obstacles':0})
        result=client.call('botany',operation={'kind':'load','path':str(candidate.relative_to(ROOT)),'publish':True})
        for system in systems:
            result=client.call('botany',operation={'kind':'set_roots','prefix':recipe['prefix'],'stem':system['stem'],'settings':system['settings']})
        client.call('botany',operation={'kind':'save','prefix':recipe['prefix'],'path':relative})
        return result['compiled']['organs']
    finally:client.close();candidate.unlink(missing_ok=True)

def main():
    p=argparse.ArgumentParser();p.add_argument('source',type=Path);p.add_argument('output',type=Path);p.add_argument('--stem',default='tronco0');p.add_argument('--settings',type=Path);p.add_argument('--binary',type=Path);a=p.parse_args()
    recipe=json.loads(a.source.read_bytes());settings=json.loads(a.settings.read_bytes()) if a.settings else {}
    print(generate(recipe,a.output,[dict(stem=a.stem,settings=settings)],a.binary))
if __name__=='__main__':main()
