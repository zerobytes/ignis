#!/usr/bin/env python3
"""Immutable source9: spherical eyeballs and classified eyelid/socket contact."""
import hashlib,json
from collections import defaultdict,deque
from extend_source import ROOT,OUT

def build():
 parent=OUT/'human-face.ignis-character';raw=parent.read_bytes();s=json.loads(raw);assert s['version']==8
 s.update(version=9,id='human_adult_spherical_v9');s['provenance']+=' Source9: rigid spherical sclera; eyelids and socket walls wrap around the globe, never reshape its surface.'
 skin=next(p for p in s['parts'] if p['name']=='body');ids=skin['vertices'];faces=s['refinements'][0]['faces'];edges=defaultdict(list);ns=defaultdict(set);inc=defaultdict(list)
 for fi,f in enumerate(faces):
  for a,b in zip(f,f[1:]+f[:1]):edges[tuple(sorted((a,b)))].append(fi);ns[a].add(b);ns[b].add(a);inc[a].append(fi)
 rows=[]
 for i in range(len(ids)):rows.append({i}|ns[i]|{v for fi in inc[i] for v in faces[fi]})
 for (a,b),adj in sorted(edges.items()):rows.append({a,b}|{v for fi in adj for v in faces[fi]})
 rows.extend(set(f) for f in faces);report=[]
 for e in s['eyes']:
  fit=e['fit'];rim=set(fit['rim']);barrier={i for i in rim if i<len(ids)};assert len(barrier)==28
  seed=next(i for i,v in enumerate(ids) if s['positions'][v][1]<.3);outside={seed};pending=deque([seed])
  while pending:
   a=pending.popleft()
   for b in ns[a]:
    if b not in outside and b not in barrier:outside.add(b);pending.append(b)
  inside=set(range(len(ids)))-outside-barrier
  candidates={i for i,row in enumerate(rows) if i not in rim and row&inside and not row&outside}
  region={w['vertex']:w['weight'] for w in fit['lid']};assert all(region.get(i)==1 for i in candidates)
  fit['spherical']=dict(interior=sorted(candidates));report.append(dict(eye=e['prefix'],rim=len(rim),lid=len(region),interior=len(candidates)))
 data=json.dumps(s,separators=(',',':')).encode();path=OUT/'human-spherical.ignis-character';path.write_bytes(data);out=ROOT/'artifacts/character-sphere';out.mkdir(exist_ok=True)
 result=dict(path=str(path),version=9,bytes=len(data),sha256=hashlib.sha256(data).hexdigest(),parent_sha256=hashlib.sha256(raw).hexdigest(),parameters=len(s['parameters']),targets=len(s['targets']),handles=len(s['handles']),eyes=report)
 (out/'source.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
if __name__=='__main__':build()
