#!/usr/bin/env python3
"""Source8: isolated facial landmarks and authored eyelid contact regions."""
import hashlib,json,math
from collections import defaultdict,deque
from extend_source import ROOT,OUT,neutral

def build():
 s=json.loads((OUT/'human-jaw.ignis-character').read_text());assert s['version']==7
 s.update(version=8,id='human_adult_face_v8')
 s['provenance']+=' Source8: Ignis-authored submental, lateral orbital/malar, three nasal levels and nostril roof fields; bounded eye/lid contact. Previous coefficients retained.'
 p=neutral(s);skin=next(x for x in s['parts'] if x['name']=='body');ids=skin['vertices'];local={v:i for i,v in enumerate(ids)};faces=s['refinements'][0]['faces'];ns=defaultdict(set);inc=defaultdict(list);edges=defaultdict(list)
 for fi,f in enumerate(faces):
  for a,b in zip(f,f[1:]+f[:1]):ns[a].add(b);ns[b].add(a);inc[a].append(fi);edges[tuple(sorted((a,b)))].append(fi)
 def stencil(v):
  n=len(ns[v]);row=defaultdict(float);row[v]=(n-3)/n
  for fi in inc[v]:
   for a in faces[fi]:row[a]+=1/(n*n*len(faces[fi]))
  for a in ns[v]:row[v]+=1/(n*n);row[a]+=1/(n*n)
  return row
 rows=[stencil(i) for i in range(len(ids))];edge_index={e:len(ids)+i for i,e in enumerate(sorted(edges))}
 for (a,b),adj in sorted(edges.items()):
  row=defaultdict(float);row[a]=row[b]=.25
  for fi in adj:
   for v in faces[fi]:row[v]+=.25/len(faces[fi])
  assert len(adj)==2;rows.append(row)
 for f in faces:rows.append({v:1/len(f) for v in f})
 def loop(a,b):
  result=[local[a],local[b]]
  for _ in range(128):
   a,b=result[-2:];together={x for fi in inc[b] if a in faces[fi] for x in faces[fi]};options=ns[b]-together;assert len(options)==1
   v=options.pop()
   if v==result[0]:return result
   result.append(v)
  raise ValueError('Unclosed eyelid loop')
 right=loop(41,36);assert len(right)==28
 mirror={i:min(range(len(ids)),key=lambda v:math.dist(p[ids[v]],[-p[ids[i]][0],p[ids[i]][1],p[ids[i]][2]])) for i in right}
 contact=set()
 for eye in s['eyes']:
  ring=right if eye['prefix']=='eye_R' else [mirror[i] for i in right]
  # Cut the rim topologically. The complete inner socket follows depth rigidly;
  # only the external facial side fades out, avoiding stationary inner walls
  # crossing a recessed eye. Rest contact never projects skin onto a sphere.
  barrier=set(ring);seed=next(i for i,v in enumerate(ids) if p[v][1]<.3);outside={seed};pending=deque([seed])
  while pending:
   a=pending.popleft()
   for b in ns[a]:
    if b not in barrier and b not in outside:outside.add(b);pending.append(b)
  inside=set(range(len(ids)))-outside-barrier;assert len(inside)>28
  dist={i:0 for i in ring};queue=deque(ring)
  while queue:
   a=queue.popleft()
   if dist[a]>=8:continue
   for b in ns[a]:
    if b not in dist and b not in inside:dist[b]=dist[a]+1;queue.append(b)
  sign=1 if eye['prefix']=='eye_R' else -1
  def fade(i,d):
   t=max(0,min(1,(sign*p[ids[i]][0]-.008)/.006));return (1-(d/8)**2)**2*t*t*(3-2*t)
  cage={i:fade(i,d) for i,d in dist.items()};cage.update({i:1. for i in inside});weights={i:sum(cage.get(v,0)*w for v,w in row.items()) for i,row in enumerate(rows)}
  rim=[]
  for a,b in zip(ring,ring[1:]+ring[:1]):rim.extend([a,edge_index[tuple(sorted((a,b)))]])
  for v in rim:weights[v]=1.
  eye['fit']=dict(part='body',rim=rim,lid=[dict(vertex=i,weight=round(w,7)) for i,w in sorted(weights.items()) if w>0])
  for i in rim:contact.update(ids[v] for v in rows[i])
  for name,lo,hi,default in [('globe_depth',-.003,.008,0),('socket_fit',0,1,1)]:s['parameters'].append(dict(name=eye['prefix']+'_'+name,minimum=lo,maximum=hi,default=default))
 protected=set();ocular_pins=set();alar_pins=set()
 for h in s['handles']:
  dest=ocular_pins if h['name'].startswith(('eye_inner_','eye_outer_','upper_lid_','lower_lid_')) else alar_pins if h['name'].startswith(('nose_wing_','nose_alar_base_')) else None
  if dest is not None:
   for a in h['anchor']['vertices']:dest.update(ids[v] for v in rows[local[a]])
  if h.get('group') in ['Lábios','Orelhas'] or h['name'] in ['nose_tip','nose_bridge']:
   for a in h['anchor']['vertices']:protected.update(ids[v] for v in rows[local[a]])
 specs=[('submental','Papada central','Papada',[0,.567,-.096],[.021,.015,.021],[.012,.020,.020],True),
 ('submental_side','Papada lateral','Papada',[.020,.568,-.085],[.016,.015,.021],[.012,.018,.018],False),
 ('malar_upper','Acima da maçã do rosto','Olhos e maçãs',[.052,.650,-.112],[.010,.009,.012],[.008,.008,.012],False),
 ('orbital_lateral','Lateral da órbita','Olhos e maçãs',[.055,.664,-.105],[.010,.008,.012],[.008,.008,.010],False),
 ('nose_upper_side','Largura superior do nariz','Nariz',[.006,.664,-.134],[.006,.006,.012],[.004,.005,.006],False),
 ('nose_middle_side','Largura média do nariz','Nariz',[.007,.648,-.143],[.006,.005,.011],[.004,.004,.006],False),
 ('nose_tip_side','Largura da ponta do nariz','Nariz',[.008,.638,-.150],[.006,.004,.010],[.004,.004,.006],False),
 ('nostril_roof','Teto da narina','Nariz',[.010,.632,-.144],[.004,.0025,.005],[.002,.003,.003],False)]
 used={h['anchor']['vertices'][0] for h in s['handles']};head=next(i for i,j in enumerate(s['joints']) if j['name']=='head');report=[]
 for stem,label,group,goal,radii,limits,mid in specs:
  banned=protected|(contact if group in ['Olhos e maçãs','Nariz'] else set())
  if group=='Olhos e maçãs':banned|=ocular_pins
  if stem=='nostril_roof':banned|=alar_pins
  for side,sign,text in [('',1,'')] if mid else [('L',-1,'E'),('R',1,'D')]:
   name=stem+('_'+side if side else '');target=[goal[0]*sign,*goal[1:]]
   allowed=[i for i in ids if i not in used|banned and (abs(p[i][0])<1e-6 if mid else sign*p[i][0]>.002)]
   anchor=min(allowed,key=lambda i:math.dist(p[i],target));used.add(anchor);center=p[anchor];weights={}
   for i in ids:
    if i in banned or (not mid and sign*p[i][0]<.002):continue
    d=sum(((p[i][a]-center[a])/radii[a])**2 for a in range(3))
    if d<1:weights[i]=(1-d)**2
   response=sum(w*weights.get(ids[v],0) for v,w in rows[local[anchor]].items());assert .2<response<=1,(name,response)
   channels=['face_'+name+'_'+a for a in 'xyz']
   for axis,(channel,limit) in enumerate(zip(channels,limits)):
    s['parameters'].append(dict(name=channel,minimum=-limit,maximum=limit,default=0))
    s['targets'].append(dict(name='handle_'+name+'_'+str(axis),gain=1,factors=[dict(parameter=channel,curve=[[-limit,-1],[0,0],[limit,1]])],deltas=[dict(vertex=i,offset=[round(limit*w/response,9) if k==axis else 0 for k in range(3)]) for i,w in sorted(weights.items())]))
   s['handles'].append(dict(name=name,label=label+(' · '+text if side else ''),group=group,part='body',anchor=dict(vertices=[anchor]),parameters=channels,limits=limits,mirror=stem+'_'+('R' if side=='L' else 'L') if side else None,midline=mid,frame_joint=head))
   report.append(dict(name=name,anchor=anchor,center=center,radii=radii,limits=limits,affected=len(weights),response=response))
 for h in s['handles']:
  if h['name'].startswith('nose_'):h['group']='Nariz'
 s['parameters'].sort(key=lambda p:p['name']);assert len(s['parameters'])==392 and len(s['handles'])==95 and len(s['targets'])==553
 data=json.dumps(s,separators=(',',':')).encode();path=OUT/'human-face.ignis-character';path.write_bytes(data)
 result=dict(path=str(path),bytes=len(data),sha256=hashlib.sha256(data).hexdigest(),parameters=len(s['parameters']),handles=len(s['handles']),targets=len(s['targets']),fields=report,contact=[dict(eye=e['prefix'],rim=len(e['fit']['rim']),lid=len(e['fit']['lid'])) for e in s['eyes']]);out=ROOT/'artifacts/character-face';out.mkdir(exist_ok=True);(out/'source.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
if __name__=='__main__':build()
