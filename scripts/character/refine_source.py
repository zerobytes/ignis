#!/usr/bin/env python3
"""Source3: adult male/female bases, detailed lips and rigid head/neck pivots and humanoid head forms.

All imported files are pinned CC0 graphical targets; no upstream program code.
Source1/2 files remain available for existing recipes and undo.
"""
import argparse
import copy
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
import math
from pathlib import Path
import urllib.request
from extend_source import ROOT, DATA, OUT, REVISION, neutral, read_target
from prepare_source import specifications

LIPS = {
    'lip_upper_height': ('mouth-upperlip-height','decr','incr'),
    'lip_lower_height': ('mouth-lowerlip-height','decr','incr'),
    'lip_upper_volume': ('mouth-upperlip-volume','decr','incr'),
    'lip_lower_volume': ('mouth-lowerlip-volume','decr','incr'),
    'lip_upper_width': ('mouth-upperlip-width','decr','incr'),
    'lip_lower_width': ('mouth-lowerlip-width','decr','incr'),
    'lip_cupid_shape': ('mouth-cupidsbow','decr','incr'),
    'lip_cupid_width': ('mouth-cupidsbow-width','decr','incr'),
    'lip_philtrum': ('mouth-philtrum-volume','decr','incr'),
    'lip_upper_middle': ('mouth-upperlip-middle','down','up'),
    'lip_lower_middle': ('mouth-lowerlip-middle','down','up'),
    'lip_corners': ('mouth-angles','down','up'),
    'lip_upper_outer': ('mouth-upperlip-ext','down','up'),
    'lip_lower_outer': ('mouth-lowerlip-ext','down','up'),
}

def extra_paths():
    male=[p.replace('female','male') for p,_,_ in specifications() if 'female' in p and not p.startswith('breast/')]
    lips=[f'mouth/{p}-{suffix}.target' for p,lo,hi in LIPS.values() for suffix in [lo,hi]]
    return sorted(set(male+lips))


def acquire():
    manifest=json.loads((DATA/'provenance.json').read_text());rows={r['path']:r for r in manifest['files']}
    def one(relative):
        name='makehuman/data/targets/'+relative;path=DATA/name
        if not path.exists():
            data=urllib.request.urlopen(f'https://raw.githubusercontent.com/makehumancommunity/makehuman/{REVISION}/{name}',timeout=45).read()
            path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(data)
        row={'path':name,'sha256':hashlib.sha256(path.read_bytes()).hexdigest()}
        if name in rows and rows[name]!=row:raise ValueError('Changed pinned target: '+name)
        return row
    with ThreadPoolExecutor(max_workers=6) as pool:
        for row in pool.map(one,extra_paths()):rows[row['path']]=row
    manifest['files']=list(rows.values());(DATA/'provenance.json').write_text(json.dumps(manifest,indent=2)+'\n')


def smooth(a,b,v):
    t=max(0,min(1,(v-a)/(b-a)));return t*t*(3-2*t)


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--fetch',action='store_true');args=parser.parse_args()
    if args.fetch:acquire()
    manifest=json.loads((DATA/'provenance.json').read_text())
    for row in manifest['files']:
        if hashlib.sha256((DATA/row['path']).read_bytes()).hexdigest()!=row['sha256']:raise ValueError('Checksum changed: '+row['path'])
    s=json.loads((OUT/'female-extended.ignis-character').read_text());assert s['version']==2
    s.update(version=3,id='human_adult_v3',surfaces=[])
    s['provenance']+=' Source3: adult male graphical targets, lip controls, rigid craniofacial head/neck pivots and rest-anchored muscle relief.'
    base=json.loads((OUT/'female.ignis-character').read_text());points=neutral(base)
    skin=next(p for p in s['parts'] if p['name']=='body');ids=set(skin['vertices']);joints={j['name']:i for i,j in enumerate(s['joints'])}
    original_count=min(p['vertices'][0] for p in s['parts'] if p.get('socket') is not None)
    def param(name,lo=-1,hi=1,default=0):s['parameters'].append(dict(name=name,minimum=lo,maximum=hi,default=default))
    def f(name,curve=None):return dict(parameter=name,curve=curve or [[-1,-1],[0,0],[1,1]])
    def target(name,values,factors,gain=1):
        s['targets'].append(dict(name=name,gain=gain,factors=factors,deltas=[dict(vertex=i,offset=[round(v,9) for v in value]) for i,value in sorted(values.items()) if max(map(abs,value))>1e-9]))
    param('body_masculinity',0,1)
    female=f('body_masculinity',[[0,1],[1,0]]);male=f('body_masculinity',[[0,0],[1,1]])
    for old,(path,_,_) in zip(s['targets'],specifications()):
        if 'female' not in path and not path.startswith('breast/'):continue
        old_factors=copy.deepcopy(old['factors']);old['factors'].append(copy.deepcopy(female))
        if path.startswith('breast/'):continue
        target('male_'+old['name'],read_target(path.replace('female','male')),old_factors+[copy.deepcopy(male)],old['gain'])
    # Condition the extended endpoints on the selected adult anatomical base, too.
    for mi,muscle in enumerate(['minmuscle','averagemuscle','maxmuscle']):
        old=next(t for t in s['targets'] if t['name']==f'extreme_adiposity_{mi}');old['factors'].append(copy.deepcopy(female))
        a=read_target(f'macrodetails/universal-male-young-{muscle}-maxweight.target');b=read_target(f'macrodetails/universal-male-young-{muscle}-averageweight.target')
        target(f'male_extreme_adiposity_{mi}',{i:[2.75*(a.get(i,[0]*3)[k]-b.get(i,[0]*3)[k]) for k in range(3)] for i in a.keys()|b.keys()},copy.deepcopy(old['factors'][:-1])+[copy.deepcopy(male)])
    for wi,weight in enumerate(['minweight','averageweight','maxweight']):
        old=next(t for t in s['targets'] if t['name']==f'extreme_musculature_{wi}');old['factors'].append(copy.deepcopy(female))
        a=read_target(f'macrodetails/universal-male-young-maxmuscle-{weight}.target');b=read_target(f'macrodetails/universal-male-young-averagemuscle-{weight}.target')
        target(f'male_extreme_musculature_{wi}',{i:[.8*(a.get(i,[0]*3)[k]-b.get(i,[0]*3)[k]) for k in range(3)] for i in a.keys()|b.keys()},copy.deepcopy(old['factors'][:-1])+[copy.deepcopy(male)])
    for name,(path,lo,hi) in LIPS.items():
        param(name)
        for suffix,curve in [(lo,[[-1,1],[0,0],[1,0]]),(hi,[[-1,0],[0,0],[1,1]])]:target(name+'_'+suffix,read_target('mouth/'+path+'-'+suffix+'.target'),[f(name,curve)])
    # Preserve the existing spine and stylized upper-head fields exactly.
    # Full craniofacial region and helper anchors share exactly the same transform.
    head=joints['head'];desc={head}
    for i,j in enumerate(s['joints']):
        if j['parent'] in desc:desc.add(i)
    # Follow the jaw/nape boundary rather than making every tiny head influence
    # rigid (the rig intentionally blends some of those into clavicles/trapezius).
    def head_weight(p):
        boundary=.598+.38*p[2]
        return smooth(boundary-.045,boundary+.004,p[1])*(1-smooth(.10,.15,abs(p[0])))
    protected={v for v in ids if head_weight(points[v])>=1-1e-9}
    for i in desc:
        for key in ['head','tail']:protected.update(s['joints'][i][key]['vertices'])
    # New pivots use a rigid craniofacial mask; only the neck/body junction blends.
    def add_pivot(name,joint,angle,weights):
        param(name)
        s['warps'].append(dict(name=name,parameter=name,origin=copy.deepcopy(s['joints'][joints[joint]]['head']),axis=[1,0,0],angle_degrees=angle,weights=weights))
    head_weights=[];neck_weights=[]
    neck_desc={joints['neck01']}
    for i,j in enumerate(s['joints']):
        if j['parent'] in neck_desc:neck_desc.add(i)
    helper_neck=set()
    for j in neck_desc:
        for key in ['head','tail']:helper_neck.update(s['joints'][j][key]['vertices'])
    for i,p in enumerate(points[:original_count]):
        if i in protected:hw=nw=1
        elif i in ids or i in helper_neck:
            hw=head_weight(p)
            nw=max(hw,smooth(.49,.535,p[1])*(1-smooth(.07,.125,abs(p[0]))))
        else:hw=nw=0
        if hw:head_weights.append(dict(vertex=i,weight=round(hw,7)))
        if nw:neck_weights.append(dict(vertex=i,weight=round(nw,7)))
    add_pivot('neck_pitch','neck01',40,neck_weights)
    add_pivot('head_pitch','head',40,head_weights)
    param('head_lower_pitch')
    bridge=next(h for h in s['handles'] if h['name']=='nose_bridge')['anchor']
    lower_weights=[dict(vertex=v['vertex'],weight=round(v['weight']*(1-smooth(.63,.68,points[v['vertex']][1])),7)) for v in head_weights if points[v['vertex']][1]<.68]
    s['warps'].append(dict(name='head_lower_pitch',parameter='head_lower_pitch',origin=copy.deepcopy(bridge),axis=[1,0,0],angle_degrees=35,weights=lower_weights))
    param('head_height')
    height_origin=min(points[i][1] for i in protected&ids)
    target('head_height',{v['vertex']:[0,(points[v['vertex']][1]-height_origin)*v['weight'],0] for v in head_weights},[f('head_height',[[-1,-.55/1.2],[0,0],[1,1]])],1.2)
    rigid_sockets=sorted({p['socket'] for p in s['parts'] if p.get('socket') is not None})
    for w in s['warps']:
        if w['parameter'] in ['neck_pitch','head_pitch']:w['rigid_sockets']=rigid_sockets
    s['warps'].sort(key=lambda w:bool(w.get('rigid_sockets')))
    # Lip handles use topology-local fields; upper/lower seam masks are independent.
    def landmark(name,label,center,radii,limits,mirror=None,zone=None):
        candidates=[v for v in skin['vertices'] if points[v][2]<-.105 and (zone is None or zone(points[v]))]
        anchor=min(candidates,key=lambda i:sum(((points[i][a]-center[a])/radii[a])**2 for a in range(3)))
        center=points[anchor];channels=['face_'+name+'_'+a for a in ['x','y','z']]
        weights={}
        for i in candidates:
            d=sum(((points[i][a]-center[a])/radii[a])**2 for a in range(3))
            if d<1:weights[i]=(1-d)**2
        for axis,(channel,limit) in enumerate(zip(channels,limits)):
            param(channel,-limit,limit)
            target('handle_'+name+'_'+str(axis),{i:[limit*value if k==axis else 0 for k in range(3)] for i,value in weights.items()},[f(channel,[[-limit,-1],[0,0],[limit,1]])])
        s['handles'].append(dict(name=name,label=label,part='body',anchor=dict(vertices=[anchor]),parameters=channels,limits=limits,mirror=mirror,midline=mirror is None,frame_joint=head))
    upper=lambda p:p[1]>.6075
    lower=lambda p:p[1]<.6075
    for name,label,x,y,z,radii,zone in [
        ('cupid_peak','Pico do arco do cupido',.008,.616,-.143,[.008,.010,.025],upper),
        ('upper_lip_lobe','Volume lateral do lábio superior',.017,.613,-.142,[.011,.008,.023],upper),
        ('upper_lip_rim','Borda externa do lábio superior',.023,.612,-.135,[.009,.008,.022],upper),
        ('upper_lip_seam','Borda interna do lábio superior',.012,.6085,-.145,[.009,.005,.020],upper),
        ('lower_lip_seam','Borda interna do lábio inferior',.011,.605,-.143,[.009,.005,.022],lower),
        ('lower_lip_lobe','Volume lateral do lábio inferior',.015,.601,-.141,[.011,.008,.023],lower),
        ('lower_lip_rim','Borda externa do lábio inferior',.018,.595,-.131,[.010,.008,.023],lower),
    ]:
        for side,sign,text in [('L',-1,'E'),('R',1,'D')]:landmark(name+'_'+side,label+' · '+text,[sign*x,y,z],radii,[.006,.007,.010],name+('_R' if side=='L' else '_L'),zone)
    landmark('upper_tubercle','Tubérculo central do lábio superior',[0,.610,-.147],[.009,.006,.022],[.005,.006,.010],zone=upper)
    landmark('lower_lip_border','Contorno inferior central do lábio',[0,.593,-.131],[.011,.008,.025],[.005,.008,.010],zone=lower)
    for h in s['handles']:
        h['group']='Lábios' if any(k in h['name'] for k in ['lip','cupid','tubercle','mouth_corner']) else 'Rosto'
    # Reference-coordinate relief stays attached through posture and animation.
    param('muscle_striation',0,1,.75)
    # A shared local muscle patch reuses31 fibres across anatomical regions,
    # instead of carving32 long world-space lines into the whole body.
    segments=[dict(start=[-.96+i*.064,-.92],end=[-.96+i*.064,.92],widths=[.012,.008]) for i in range(31)]
    def dot(a,b):return sum(x*y for x,y in zip(a,b))
    def cross(a,b):return [a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]]
    def unit(v):
        length=math.sqrt(dot(v,v));return [x/length for x in v]
    fibres=[]
    for side in ['L','R']:
        for region in ['upperarm','lowerarm','upperleg','lowerleg']:
            first=s['joints'][joints[region+'01_'+side]]['head']['vertices'];last=s['joints'][joints[region+'02_'+side]]['tail']['vertices']
            a=[sum(points[v][k] for v in first)/len(first) for k in range(3)];b=[sum(points[v][k] for v in last)/len(last) for k in range(3)]
            axis=unit([b[k]-a[k] for k in range(3)]);length=math.dist(a,b);right=unit(cross(axis,[0,0,-1]));front=unit(cross(right,axis))
            fibres.append((region,side,a,axis,length,right,front))
    coordinates=[]
    for vertex,row in zip(skin['vertices'],skin['weights']):
        x,y,z=points[vertex];uv=[5.,5.,0.]
        influences=list(zip(row['joints'],row['weights']))+[(v['joint'],v['weight']) for v in row.get('extra',[])]
        for region,side,a,axis,length,right,front in fibres:
            weight=sum(value for bone,value in influences if s['joints'][bone]['name'].startswith(region) and s['joints'][bone]['name'].endswith('_'+side))
            if weight<.35:continue
            relative=[points[vertex][k]-a[k] for k in range(3)];distance=dot(relative,axis);radial=[relative[k]-distance*axis[k] for k in range(3)]
            uv=[math.atan2(dot(radial,right),dot(radial,front))/math.pi*1.25,(distance/length-.45)/.5,0.];break
        if abs(x)<.16 and .35<y<.50 and z<-.03:
            radius=math.hypot(abs(x)+.005,y-.35)
            uv=[(math.atan2(y-.35,abs(x)+.005)-.65)/.65,(radius-.09)/.08,0.]
        elif abs(x)<.10 and .0<y<.29 and z<-.045:uv=[x*10,(y-.145)/.145,0.]
        elif abs(x)<.18 and .14<y<.50 and z>.015:
            radius=math.hypot(abs(x)+.02,y-.14)
            uv=[(math.atan2(y-.14,abs(x)+.02)-.9)/.65,(radius-.18)/.18,0.]
        coordinates.append([round(v,7) for v in uv])
    s['surfaces']=[dict(part='body',coordinates=coordinates,factors=[f('muscle_definition',[[0,0],[1,1]]),f('muscle_striation',[[0,0],[1,1]]),f('adiposity',[[0,1],[.5,.8],[1,.1],[2,0]])],detail=dict(profile='rounded',plane='xy',tint=[1,1,1],opacity=0,bump=-.00018,segments=segments))]
    # Broader inter-muscle cuts affect geometry as well as the fine normal relief.
    normals=[[0.,0.,0.] for _ in skin['vertices']]
    for a,b,c in skin['triangles']:
        pa,pb,pc=(points[skin['vertices'][i]] for i in [a,b,c]);u=[pb[i]-pa[i] for i in range(3)];v=[pc[i]-pa[i] for i in range(3)]
        n=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]]
        for j in [a,b,c]:
            for k in range(3):normals[j][k]+=n[k]
    cuts=[]
    for sign in [-1,1]:
        cuts.extend([([sign*.015,.40],[sign*.14,.39]),([sign*.155,.47],[sign*.205,.39]),([sign*.075,-.13],[sign*.105,-.42])])
        for i in range(3):cuts.append(([sign*.015,.075+i*.072],[sign*.08,.09+i*.064]))
    separation={}
    for vertex,n in zip(skin['vertices'],normals):
        x,y,z=points[vertex]
        if z>-.03 or y>.54:continue
        distance=1
        for a,b in cuts:
            v=[b[k]-a[k] for k in range(2)];t=max(0,min(1,sum(([x,y][k]-a[k])*v[k] for k in range(2))/sum(q*q for q in v)))
            distance=min(distance,math.hypot(x-a[0]-t*v[0],y-a[1]-t*v[1]))
        if distance<.026:
            length=math.sqrt(sum(q*q for q in n))
            if length:separation[vertex]=[-q/length*.0045*math.exp(-(distance/.012)**2) for q in n]
    target('muscular_separation',separation,[f('muscle_definition',[[0,0],[1,1]]),f('adiposity',[[0,1],[.5,.8],[1,.1],[2,0]])])
    s['parameters'].sort(key=lambda p:p['name'])
    encoded=json.dumps(s,separators=(',',':')).encode();assert len(encoded)<=96*1024*1024,len(encoded)
    path=OUT/'human.ignis-character';path.write_bytes(encoded)
    summary=dict(path=str(path),bytes=len(encoded),parameters=len(s['parameters']),targets=len(s['targets']),handles=len(s['handles']),warps=len(s['warps']),sha256=hashlib.sha256(encoded).hexdigest(),protected_head_vertices=sorted(protected&ids))
    report=ROOT/'artifacts/character-refinement';report.mkdir(exist_ok=True);(report/'source.json').write_text(json.dumps(summary,indent=2)+'\n')
    (OUT/'human-default.json').write_text(json.dumps({'parameters':{p['name']:p['default'] for p in s['parameters']}},indent=2)+'\n')
    print(json.dumps({k:v for k,v in summary.items() if k!='protected_head_vertices'}))

if __name__=='__main__':main()
