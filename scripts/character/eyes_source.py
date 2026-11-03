#!/usr/bin/env python3
"""Immutable source7: Ignis-authored ocular generators, no new vendor assets."""
import hashlib,json,re
from extend_source import ROOT,OUT

def main():
 s=json.loads((OUT/'human-nose.ignis-character').read_text());assert s['version']==6
 s.update(version=7,id='human_adult_v7')
 s['provenance']+=' Source7: Ignis-authored seeded ocular pigment, recessed iris and corneal surface; body and prior configuration unchanged.'
 # Read the single authoritative bounded control declaration used by the UI/compiler.
 text=(ROOT/'src/meshes/character/eyes.rs').read_text().split('pub const CONTROLS:')[1].split('];',1)[0]
 controls=re.findall(r'\(\s*"([a-z_]+)",\s*"[^"]+",\s*([\d.]+),\s*([\d.]+),\s*([\d.]+),?\s*\)',text)
 assert len(controls)==21,len(controls)
 s['eyes']=[]
 for side in ['L','R']:
  part=next(p for p in s['parts'] if p['name']==f'eye_{side}_sclera');joint=s['joints'][part['socket']]
  s['eyes'].append(dict(sclera=part['name'],iris=f'eye_{side}_iris',cornea=f'eye_{side}_pupil',center=joint['head'],prefix=f'eye_{side}',radius=.012))
  for name,lo,hi,default in controls:
   s['parameters'].append(dict(name=f'eye_{side}_{name}',minimum=float(lo),maximum=float(hi),default=83 if side=='R' and name=='seed' else float(default)))
 s['parameters'].sort(key=lambda p:p['name']);assert len(s['parameters'])==304
 data=json.dumps(s,separators=(',',':')).encode();path=OUT/'human-eyes.ignis-character';path.write_bytes(data)
 out=ROOT/'artifacts/character-eyes';out.mkdir(exist_ok=True)
 report=dict(path=str(path),bytes=len(data),sha256=hashlib.sha256(data).hexdigest(),parameters=len(s['parameters']),eyes=s['eyes'],controls=controls)
 (out/'source.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items() if k not in ['controls','eyes']}))
if __name__=='__main__':main()
