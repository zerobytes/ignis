#!/usr/bin/env python3
"""Source6: paired local alar-base XYZ landmarks; zero keeps source5 geometry."""
import hashlib,json,math
from collections import defaultdict
from extend_source import ROOT,OUT,neutral

def main():
 s=json.loads((OUT/'human-cupid.ignis-character').read_text());assert s['version']==5
 s.update(version=6,id='human_adult_v6');s['provenance']+=' Source6: Ignis-authored paired inferior nasal ala controls; original shapes preserved at zero.'
 points=neutral(s);skin=s['parts'][0];ids=skin['vertices'];local={v:i for i,v in enumerate(ids)};faces=s['refinements'][0]['faces'];reports=[]
 for side,sign,label in [('L',-1,'E'),('R',1,'D')]:
  anchor=min(ids,key=lambda i:math.dist(points[i],[sign*.01251,.6242,-.13661]));center=points[anchor];name='nose_alar_base_'+side;limits=[.004,.005,.006];channels=['face_'+name+'_'+a for a in 'xyz'];weights={}
  for i in ids:
   p=points[i]
   if p[2]>-.119 or sign*p[0]<.003:continue
   radii=[.007,.0045 if p[1]>center[1] else .0055,.014]
   d=sum(((p[a]-center[a])/radii[a])**2 for a in range(3))
   if d<1:weights[i]=(1-d)**2
  incident=[f for f in faces if local[anchor] in f];neighbors=set()
  for f in incident:
   at=f.index(local[anchor]);neighbors.update([f[at-1],f[(at+1)%len(f)]])
  n=len(neighbors);row=defaultdict(float);row[local[anchor]]+=(n-3)/n
  for f in incident:
   for v in f:row[v]+=1/(n*n*len(f))
  for v in neighbors:row[local[anchor]]+=1/(n*n);row[v]+=1/(n*n)
  response=sum(w*weights.get(ids[v],0) for v,w in row.items());assert .25<response<=1
  for axis,(channel,limit) in enumerate(zip(channels,limits)):
   s['parameters'].append(dict(name=channel,minimum=-limit,maximum=limit,default=0))
   s['targets'].append(dict(name='handle_'+name+'_'+str(axis),gain=1,factors=[dict(parameter=channel,curve=[[-limit,-1],[0,0],[limit,1]])],deltas=[dict(vertex=i,offset=[round(limit*w/response,9) if k==axis else 0 for k in range(3)]) for i,w in sorted(weights.items())]))
  s['handles'].append(dict(name=name,label='Base inferior da asa do nariz · '+label,group='Rosto',part='body',anchor=dict(vertices=[anchor]),parameters=channels,limits=limits,mirror='nose_alar_base_'+('R' if side=='L' else 'L'),midline=False,frame_joint=next(i for i,j in enumerate(s['joints']) if j['name']=='head')))
  reports.append(dict(side=side,anchor=anchor,affected=len(weights),response=response))
 s['parameters'].sort(key=lambda p:p['name']);assert len(s['parameters'])==262 and len(s['handles'])==67
 data=json.dumps(s,separators=(',',':')).encode();path=OUT/'human-nose.ignis-character';path.write_bytes(data)
 report=dict(bytes=len(data),sha256=hashlib.sha256(data).hexdigest(),parameters=len(s['parameters']),targets=len(s['targets']),handles=len(s['handles']),fields=reports)
 out=ROOT/'artifacts/character-nose';out.mkdir(exist_ok=True);(out/'source.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
if __name__=='__main__':main()
