#!/usr/bin/env python3
"""Source4: smooth polygon cage, anatomical ear/jaw handles and ear shape data.

CC0 pinned graphical data only; all subdivision/handle assembly is Ignis code.
Sources1/2/3 remain immutable for old recipes and undo.
"""
import argparse
import hashlib
import json
import math
from collections import defaultdict
from extend_source import ROOT, DATA, OUT, neutral, read_target
from prepare_source import obj, retrieve

EAR = {'size':'scale','lobe_size':'lobe','height':'scale-vert','depth':'scale-depth','flare':'wing'}

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--fetch',action='store_true');args=parser.parse_args()
    if args.fetch:
        retrieve(['makehuman/data/targets/ears/'+side+'-ear-'+path+'-'+suffix+'.target' for side in ['l','r'] for path in EAR.values() for suffix in ['decr','incr']]+['makehuman/data/targets/ears/'+side+'-ear-shape-pointed.target' for side in ['l','r']])
    manifest=json.loads((DATA/'provenance.json').read_text())
    for row in manifest['files']:
        if hashlib.sha256((DATA/row['path']).read_bytes()).hexdigest()!=row['sha256']:raise ValueError('Changed pinned data: '+row['path'])
    s=json.loads((OUT/'human.ignis-character').read_text());s.update(version=4,id='human_adult_v4')
    s['provenance']+=' Source4: original polygon cage refined by native Catmull-Clark; corrected jaw/ear landmarks and pinned CC0 ear forms. Artificial fibre relief retired.'
    s.pop('surfaces',None)
    next(p for p in s['parameters'] if p['name']=='muscle_striation')['default']=0
    points=neutral(s);skin=next(p for p in s['parts'] if p['name']=='body');ids=set(skin['vertices']);local={v:i for i,v in enumerate(skin['vertices'])}
    _,groups=obj(DATA/'makehuman/data/3dobjs/base.obj');faces=[[local[i] for i in face] for face in groups['body']]
    s['refinements']=[dict(part='body',faces=faces)]
    head=next(i for i,j in enumerate(s['joints']) if j['name']=='head')
    def param(name,lo=-1,hi=1):s['parameters'].append(dict(name=name,minimum=lo,maximum=hi,default=0))
    def target(name,vectors,factors,gain=1):
        s['targets'].append(dict(name=name,gain=gain,factors=factors,deltas=[dict(vertex=i,offset=[round(v,9) for v in value]) for i,value in sorted(vectors.items()) if max(map(abs,value))>1e-9]))
    def factor(name,curve):return dict(parameter=name,curve=curve)
    # Source target support identifies the ear, independently of facial proportions.
    for side,letter,sign,text in [('l','L',-1,'E'),('r','R',1,'D')]:
        for name,path in EAR.items():
            channel='ear_'+name+'_'+letter;param(channel)
            for suffix,curve in [('decr',[[-1,1],[0,0],[1,0]]),('incr',[[-1,0],[0,0],[1,1]])]:
                target(channel+'_'+suffix,read_target('ears/'+side+'-ear-'+path+'-'+suffix+'.target'),[factor(channel,curve)])
        channel='ear_pointed_'+letter;param(channel,0,2)
        pointed=read_target('ears/'+side+'-ear-shape-pointed.target')
        target(channel,pointed,[factor(channel,[[0,0],[2,1]])],2)
        lobe=read_target('ears/'+side+'-ear-lobe-incr.target')
        ear_ids={i for i in (set(pointed)|set(lobe)|set(read_target('ears/'+side+'-ear-scale-depth-incr.target'))) & ids if sign*points[i][0]>.061 and .619<points[i][1]<.683 and -.062<points[i][2]<-.018}
        def nearest(center,candidates):return min(candidates,key=lambda i:math.dist(points[i],center))
        tip=max(pointed.keys()&ids,key=lambda i:math.dist([0,0,0],pointed[i]))
        lobe_anchor=min(lobe.keys()&ids,key=lambda i:points[i][1]+abs(points[i][0]-sign*.072)*.15)
        specs=[('ear_upper','Borda superior da orelha',nearest([sign*.074,.670,-.046],ear_ids),[.016,.020,.023]),
               ('ear_lobe','Lóbulo da orelha',lobe_anchor,[.014,.013,.017]),
               ('ear_tip','Ponta da orelha',tip,[.014,.019,.020]),
               ('ear_rim','Borda externa da orelha',nearest([sign*.081,.650,-.038],ear_ids),[.012,.019,.018]),
               ('ear_concha','Concha da orelha',nearest([sign*.073,.645,-.049],ear_ids),[.010,.013,.012]),
               ('jaw_angle','Ângulo da mandíbula',nearest([sign*.057,.611,-.062],ids),[.030,.032,.038])]
        for name,label,anchor,radii in specs:
            full=name+'_'+letter;h=next((h for h in s['handles'] if h['name']==full),None)
            if h is None:
                limits=[.018,.024,.024];channels=['face_'+full+'_'+axis for axis in ['x','y','z']]
                for ch,limit in zip(channels,limits):param(ch,-limit,limit)
                h=dict(name=full,label=label+' · '+text,part='body',anchor=dict(vertices=[anchor]),parameters=channels,limits=limits,mirror=name+('_R' if letter=='L' else '_L'),midline=False,frame_joint=head);s['handles'].append(h)
            h['anchor']={'vertices':[anchor]};h['group']='Orelhas' if name.startswith('ear_') else 'Rosto'
            s['targets']=[t for t in s['targets'] if not t['name'].startswith('handle_'+full+'_')]
            center=points[anchor];vectors={}
            candidates=ids if name=='jaw_angle' else ear_ids
            for i in candidates:
                d=sum(((points[i][a]-center[a])/radii[a])**2 for a in range(3))
                if d<1:vectors[i]=(1-d)**2
            for axis,(ch,limit) in enumerate(zip(h['parameters'],h['limits'])):
                target('handle_'+full+'_'+str(axis),{i:[limit*w if k==axis else 0 for k in range(3)] for i,w in vectors.items()},[factor(ch,[[-limit,-1],[0,0],[limit,1]])])
    # Normalize each XYZ field at its smoothed anchor: dragging 1mm moves the
    # visible anchor 1mm, instead of losing response to the subdivision average.
    incident=defaultdict(list);neighbors=defaultdict(set);edge_faces=defaultdict(list)
    for fi,face in enumerate(faces):
        for a,b in zip(face,face[1:]+face[:1]):
            incident[a].append(fi);neighbors[a].add(b);neighbors[b].add(a);edge_faces[tuple(sorted((a,b)))].append(fi)
    def stencil(v):
        row=defaultdict(float);boundary=[b for b in neighbors[v] if len(edge_faces[tuple(sorted((v,b)))])==1]
        if boundary:
            assert len(boundary)==2;row[v]=.75
            for b in boundary:row[b]+=.125
        else:
            n=len(neighbors[v]);row[v]+=(n-3)/n
            for fi in incident[v]:
                for b in faces[fi]:row[b]+=1/(n*n*len(faces[fi]))
            for b in neighbors[v]:row[v]+=1/(n*n);row[b]+=1/(n*n)
        return row
    for h in s['handles']:
        if h['part']!='body':continue
        for axis in range(3):
            t=next(t for t in s['targets'] if t['name']=='handle_'+h['name']+'_'+str(axis));values={d['vertex']:d['offset'][axis] for d in t['deltas']}
            response=sum(sum(w*values.get(skin['vertices'][v],0) for v,w in stencil(local[a]).items()) for a in h['anchor']['vertices'])/len(h['anchor']['vertices'])
            gain=h['limits'][axis]/response
            assert .9<gain<8,(h['name'],gain)
            for d in t['deltas']:d['offset'][axis]=round(d['offset'][axis]*gain,9)
    s['parameters'].sort(key=lambda p:p['name'])
    assert len(s['parameters'])<=256 and len(s['handles'])<=64 and len(s['targets'])<=512
    data=json.dumps(s,separators=(',',':')).encode();path=OUT/'human-smooth.ignis-character';path.write_bytes(data)
    report=dict(path=str(path),bytes=len(data),sha256=hashlib.sha256(data).hexdigest(),parameters=len(s['parameters']),handles=len(s['handles']),targets=len(s['targets']),cage_vertices=len(skin['vertices']),refined_vertices=len(skin['vertices'])+len(edge_faces)+len(faces),refined_triangles=sum(len(f)*2 for f in faces))
    (ROOT/'artifacts/character-smoothing/source.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))

if __name__=='__main__':main()
