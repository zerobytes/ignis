#!/usr/bin/env python3
"""Add a bounded mandibular landmark chain to source7; no schema or vendor change."""
import hashlib,json,math
from collections import defaultdict
from extend_source import ROOT,OUT,neutral

def main():
 s=json.loads((OUT/'human-eyes.ignis-character').read_text());assert s['version']==7
 s['id']='human_adult_jaw_v1';s['provenance']+=' Ignis mandibular chain1: independent chin base, paired mental/body/ramus XYZ fields; existing shape preserved at zero.'
 points=neutral(s);skin=next(p for p in s['parts'] if p['name']=='body');ids=skin['vertices'];local={v:i for i,v in enumerate(ids)};faces=s['refinements'][0]['faces'];head=next(i for i,j in enumerate(s['joints']) if j['name']=='head')
 incident=defaultdict(list);neighbors=defaultdict(set);edges=defaultdict(list)
 for fi,f in enumerate(faces):
  for a,b in zip(f,f[1:]+f[:1]):incident[a].append(fi);neighbors[a].add(b);neighbors[b].add(a);edges[tuple(sorted((a,b)))].append(fi)
 def stencil(v):
  row=defaultdict(float);boundary=[b for b in neighbors[v] if len(edges[tuple(sorted((v,b)))])==1]
  if boundary:
   assert len(boundary)==2;row[v]=.75
   for b in boundary:row[b]+=.125
  else:
   n=len(neighbors[v]);row[v]+=(n-3)/n
   for fi in incident[v]:
    for b in faces[fi]:row[b]+=1/(n*n*len(faces[fi]))
   for b in neighbors[v]:row[v]+=1/(n*n);row[b]+=1/(n*n)
  return row
 # Protect drawn lip/ear anchors including their Catmull stencils. Their old
 # fields remain editable; the new jaw fields cannot move those visible pins.
 protected=set()
 for h in s['handles']:
  if h.get('group') in ['Lábios','Orelhas']:
   for a in h['anchor']['vertices']:
    protected.update(ids[v] for v in stencil(local[a]))
 ear_support={d['vertex'] for t in s['targets'] if t['name'].startswith('ear_size_') for d in t['deltas']}
 protected.update(i for i in ear_support if abs(points[i][0])>.061 and .619<points[i][1]<.683 and -.062<points[i][2]<-.018)
 candidates=[i for i in ids if i not in protected and points[i][2]<-.04 and .545<points[i][1]<.65]
 specs=[('chin_base','Base central do queixo',[0,.5673,-.120],[.022,.014,.022],[.012,.014,.016],True),
        ('chin_side','Lateral do queixo',[.013,.582,-.132],[.016,.010,.018],[.010,.010,.014],False),
        ('chin_corner','Canto inferior do queixo',[.025,.577,-.120],[.018,.014,.020],[.012,.012,.014],False),
        ('jaw_front','Mandíbula anterior',[.034,.578,-.108],[.020,.016,.023],[.014,.014,.016],False),
        ('jaw_body','Corpo da mandíbula',[.045,.588,-.096],[.020,.018,.024],[.015,.015,.018],False),
        ('jaw_back','Mandíbula posterior',[.054,.600,-.078],[.018,.018,.022],[.014,.016,.018],False),
        ('jaw_ramus','Ramo da mandíbula',[.057,.631,-.073],[.014,.018,.020],[.012,.014,.014],False)]
 used={h['anchor']['vertices'][0] for h in s['handles']};report=[]
 for stem,label,goal,radii,limits,midline in specs:
  for side,sign,text in [('',1,'')] if midline else [('L',-1,'E'),('R',1,'D')]:
   name=stem+('_'+side if side else '');target=[goal[0]*sign,goal[1],goal[2]]
   allowed=[i for i in candidates if i not in used and (abs(points[i][0])<1e-6 if midline else sign*points[i][0]>.004)]
   anchor=min(allowed,key=lambda i:math.dist(points[i],target));used.add(anchor);center=points[anchor];weights={}
   for i in candidates:
    d=sum(((points[i][a]-center[a])/radii[a])**2 for a in range(3))
    if d<1:weights[i]=(1-d)**2
   response=sum(w*weights.get(ids[v],0) for v,w in stencil(local[anchor]).items());assert .25<response<=1,(name,response)
   channels=['face_'+name+'_'+a for a in 'xyz']
   for axis,(channel,limit) in enumerate(zip(channels,limits)):
    s['parameters'].append(dict(name=channel,minimum=-limit,maximum=limit,default=0))
    s['targets'].append(dict(name='handle_'+name+'_'+str(axis),gain=1,factors=[dict(parameter=channel,curve=[[-limit,-1],[0,0],[limit,1]])],deltas=[dict(vertex=i,offset=[round(limit*w/response,9) if k==axis else 0 for k in range(3)]) for i,w in sorted(weights.items())]))
   s['handles'].append(dict(name=name,label=label+(' · '+text if side else ''),group='Mandíbula',part='body',anchor=dict(vertices=[anchor]),parameters=channels,limits=limits,mirror=stem+'_'+('R' if side=='L' else 'L') if side else None,midline=midline,frame_joint=head))
   report.append(dict(name=name,anchor=anchor,center=center,radii=radii,limits=limits,affected=len(weights),response=response))
 for h in s['handles']:
  if h['name'] in ['chin','jaw_angle_L','jaw_angle_R']:h['group']='Mandíbula'
 s['parameters'].sort(key=lambda p:p['name']);assert len(s['handles'])==80 and len(s['parameters'])==343 and len(s['targets'])==508
 assert len({h['name'] for h in s['handles']})==80 and len({t['name'] for t in s['targets']})==508
 data=json.dumps(s,separators=(',',':')).encode();path=OUT/'human-jaw.ignis-character';path.write_bytes(data)
 out=ROOT/'artifacts/character-jaw';out.mkdir(exist_ok=True);result=dict(path=str(path),version=s['version'],bytes=len(data),sha256=hashlib.sha256(data).hexdigest(),parameters=len(s['parameters']),targets=len(s['targets']),handles=len(s['handles']),protected_vertices=len(protected),fields=report)
 (out/'source.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
if __name__=='__main__':main()
