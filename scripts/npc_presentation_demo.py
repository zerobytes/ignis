#!/usr/bin/env python3
"""Native, bounded articulated demonstration; no Blender/import/generated binary dependency."""
import copy
import json
import math
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

def quat(axis, angle):
    return [*(x * math.sin(angle / 2) for x in axis), math.cos(angle / 2)]

def creature(name, legs, long=1., prefix=''):
    bones = [dict(name=prefix+'root', parent=None, translation=[0,0,0], rotation=[0,0,0,1])]
    positions, triangles, weights, chains = [], [], [], []
    def bone(label, parent, t):
        index = len(bones)
        bones.append(dict(name=prefix+label, parent=parent, translation=t, rotation=[0,0,0,1]))
        return index
    def global_pos(index):
        out = list(bones[index]['translation'])
        parent = bones[index]['parent']
        while parent is not None:
            out = [a+b for a,b in zip(out,bones[parent]['translation'])]
            parent = bones[parent]['parent']
        return out
    def ellipsoid(center, radii, joint, direction=None):
        # Rings exclude poles to keep triangles nondegenerate.
        n, rings, base = 12, 7, len(positions)
        if direction:
            d = [b-a for a,b in zip(*direction)]
            length = sum(x*x for x in d)**.5
            y = [x/length for x in d]
            x = [y[1],-y[0],0] if abs(y[2])<.9 else [0,y[2],-y[1]]
            mag=sum(a*a for a in x)**.5;x=[a/mag for a in x]
            z=[x[1]*y[2]-x[2]*y[1], x[2]*y[0]-x[0]*y[2], x[0]*y[1]-x[1]*y[0]]
        else:x,y,z=[1,0,0],[0,1,0],[0,0,1]
        def add(v):
            positions.append([center[k]+x[k]*v[0]+y[k]*v[1]+z[k]*v[2] for k in range(3)])
            weights.append(dict(joints=[joint,0,0,0], weights=[1,0,0,0]))
        add([0,radii[1],0])
        for j in range(1,rings):
            p=math.pi*j/rings
            for i in range(n):
                t=math.tau*i/n
                add([radii[0]*math.sin(p)*math.cos(t),radii[1]*math.cos(p),radii[2]*math.sin(p)*math.sin(t)])
        bottom=len(positions);add([0,-radii[1],0])
        for i in range(n):
            a=base+1+i;b=base+1+(i+1)%n
            triangles.append([base,b,a])
            for j in range(rings-2):
                c=a+j*n;d=b+j*n
                triangles.extend([[c,d,c+n],[d,d+n,c+n]])
            a=base+1+(rings-2)*n+i;b=base+1+(rings-2)*n+(i+1)%n
            triangles.append([a,b,bottom])
    ellipsoid([0,0,0],[.43*long,.3*long,.65*long],0)
    head=bone('head',0,[0,.26*long,.62*long])
    ellipsoid(global_pos(head),[.3*long,.28*long,.38*long],head)
    ellipsoid([0,.18*long,.95*long],[.22*long,.15*long,.22*long],head)
    for side in [-1,1]:
        ellipsoid([side*.22*long,.57*long,.59*long],[.08*long,.22*long,.12*long],head)
        ellipsoid([side*.235*long,.36*long,.83*long],[.09*long,.09*long,.09*long],head)
    for i in range(legs):
        side=-1 if i%2==0 else 1
        z=(.43 if i<2 else -.43) if legs==4 else (.45-(i//2)*.45)
        a=bone(f'limb_{i}_upper',0,[side*.34*long,-.05*long,z*long])
        b=bone(f'limb_{i}_lower',a,[side*.08*long,-.35*long,.12*long])
        c=bone(f'limb_{i}_tip',b,[side*.08*long,-.35*long,-.12*long])
        for first,last,width in [(a,b,.095*long),(b,c,.065*long)]:
            p,q=global_pos(first),global_pos(last)
            center=[(x+y)/2 for x,y in zip(p,q)]
            length=sum((x-y)**2 for x,y in zip(p,q))**.5
            ellipsoid(center,[width,length*.59,width],first,[p,q])
            ellipsoid(p,[width*1.1]*3,first)
        ellipsoid([x+y for x,y in zip(global_pos(c),[0,0,.06*long])],[.12*long,.06*long,.18*long],c)
        chains.append(dict(bones=[a,b,c],pole=[0,0,1],normal_axis=[0,1,0],probe_up=.2,probe_down=.3,max_correction=.17,sole_offset=.05*long,support_radius=.04*long,max_slope_degrees=45))
    tail=bone('tail',0,[0,.06*long,-.64*long])
    ellipsoid([0,.16*long,-.98*long],[.11*long,.11*long,.48*long],tail)
    tracks=[]
    def track(joint, angles, duration=60, travel=None):
        rest=bones[joint]['translation']
        return dict(bone=joint,keys=[dict(tick=round(k*duration/(len(angles)-1)),translation=[rest[0],rest[1],rest[2]+(travel or 0)*k/(len(angles)-1)],rotation=quat([1,0,0],v)) for k,v in enumerate(angles)])
    for i,chain in enumerate(chains):
        sign=1 if (i//2+i%2)%2 else -1
        tracks.append(track(chain['bones'][0],[0,sign*.35,0,-sign*.35,0]))
        tracks.append(track(chain['bones'][1],[0,abs(sign)*.2,0,.2,0]))
    tracks.append(track(0,[0,0],travel=1.5*long))
    clips=[dict(id='walk',duration_ticks=60,tracks=tracks)]
    for clip,angles in [('bold',[0,-.2,0]),('careful',[.2,.35,.2]),('cast',[0,-.5,0]),('dead',[0,1.25,1.25]),('air',[0,-.25,0,.25,0])]:
        clips.append(dict(id=clip,duration_ticks=60,tracks=[track(head,angles),dict(bone=tail,keys=[dict(tick=0,translation=bones[tail]['translation'],rotation=quat([0,1,0],-.3)),dict(tick=30,translation=bones[tail]['translation'],rotation=quat([0,1,0],.3)),dict(tick=60,translation=bones[tail]['translation'],rotation=quat([0,1,0],-.3))])]))
    return dict(id=name,geometry=dict(positions=positions,triangles=triangles),shading='smooth',rig=dict(bones=bones,weights=weights),clips=clips),chains

def scene(base=None):
    s=json.loads((base or ROOT/'examples/npc-presentation.scene.json').read_text())
    s['version']=99;s['name']='Articulated morphologies — root motion, contacts and retarget'
    s['camera']=dict(position=[9,8,12],target=[0,.7,0],fov_degrees=48)
    source,chains=creature('canid',4)
    target,tchains=creature('grazer',4,1.5,'long_')
    insect,ichains=creature('crawler',6,.8)
    # A taller, narrower rest skeleton, rather than a uniform scale clone.
    for point in target['geometry']['positions']:
        point[0]*=.8;point[2]*=.8
    for bone in target['rig']['bones']:
        bone['translation'][0]*=.8;bone['translation'][2]*=.8
    for clip in target['clips']:
        for track in clip['tracks']:
            for key in track['keys']:
                key['translation'][0]*=.8;key['translation'][2]*=.8

    # Target starts with a different, restrained gait. The MCP retarget step
    # replaces its native walk keys with the source action on this rest skeleton.
    for track in next(c for c in target['clips'] if c['id']=='walk')['tracks']:
        if track['bone']:
            for key in track['keys']:
                angle=2*math.atan2(key['rotation'][0],key['rotation'][3])
                key['rotation']=quat([1,0,0],angle*.5)
    # 16 clips is a document-wide native budget. Ground-only actors do not
    # carry an unused flight clip; all actual phases retain their authored clips.
    for asset in (source,target):
        asset['clips']=[c for c in asset['clips'] if c['id']!='air']
    air=next(c for c in insect['clips'] if c['id']=='air')
    for i,chain in enumerate(ichains):
        for joint in chain['bones'][:2]:
            rest=insect['rig']['bones'][joint]['translation']
            air['tracks'].append(dict(bone=joint,keys=[dict(tick=t,translation=rest,rotation=quat([1,0,0],angle)) for t,angle in [(0,0),(15,.5 if i%2 else -.5),(30,0),(45,-.5 if i%2 else .5),(60,0)]]))
    # Explicit authored death poses, kept inside the conservative physical proxy.
    for asset,half_height in [(source,.8),(target,1.2),(insect,.8)]:
        dead=next(c for c in asset['clips'] if c['id']=='dead')
        drop=-half_height-min(p[0] for p in asset['geometry']['positions'])
        dead['tracks'].append(dict(bone=0,keys=[dict(tick=0,translation=[0,0,0],rotation=[0,0,0,1]),dict(tick=30,translation=[0,drop,0],rotation=quat([0,0,1],math.pi/2)),dict(tick=60,translation=[0,drop,0],rotation=quat([0,0,1],math.pi/2))]))
    s['meshes']=dict(assets=[source,target,insect],instances=[])
    by_id={e['id']:e for e in s['entities']}
    for actor,asset,ik,scale,color in [('a','canid',chains,1,[.14,.57,.63]),('b','grazer',tchains,1.5,[.8,.43,.12]),('flyer','crawler',ichains,.8,[.48,.3,.72])]:
        e=by_id[actor];e['avatar']['asset']=asset;e['avatar']['ik']=ik;e['avatar']['orientation']=[0,1,0,0]
        e['color']=color
        e['npc']['awareness']['initial_facing_degrees']=140
        if actor!='flyer':
            e['transform']['size']=[3.2 if actor=='a' else 3.9,1.6*scale,3.2 if actor=='a' else 3.9]
            e['transform']['position']=[-3, .8*scale+.01,-2 if actor=='a' else 4]
            e['avatar']['root_motion']=dict(bone=0,distance_scale=1,forward_axis=[0,1])
            e['npc']['speed']=1.5 if actor=='a' else 1.8
            e['npc']['objectives']=[dict(priority=0,when='always',action=dict(kind='guard_3d',point=[3,.8*scale+.01,-2 if actor=='a' else 4],arrival_distance=.2))]
            e['character_controller']={}
            e['avatar']['variants']=[v for v in e['avatar']['variants'] if v['phase']!='air']
        else:
            # Free-space flight requires a local connector when no graph is authored.
            e['npc']['local_connections']=dict(walk=False,fly=True,max_distance=8)
            e['transform']['size']=[2.4,1.6,2.4]
    # Shared authored banks; playback remains separate from sensory gameplay noise.
    sounds=[]
    for name,frames,hz in [('footstep',1200,85),('call',3600,360),('wing',1800,180)]:
        samples=[]
        for i in range(frames):
            envelope=(1-i/frames)**2
            grain=((i*1103515245+12345)&65535)/32767.5-1
            value=math.sin(math.tau*hz*i/24000)+.3*grain
            samples.append(round(4500*envelope*value))
        sounds.append(dict(id=name,samples=samples))
    s['avatar_audio']=dict(sounds=sounds)
    for e in s['entities']:
        if 'avatar' not in e:continue
        e['avatar']['step_ticks']=15
        for v in e['avatar']['variants']:
            v['sound']={'walk':'footstep','air':'wing','cast':'call','dead':'footstep'}.get(v['phase'])
    next(a for a in s['ability_catalog']['abilities'] if a['id']=='meditate')['movement']={'mode':'lock'}
    by_id['player']['transform']['position']=[8,.8,7]
    s['npc_templates']=[]
    return s

if __name__=='__main__':
    target=ROOT/'examples/npc-presentation-complete.scene.json'
    target.write_text(json.dumps(scene(),indent=2)+'\n')
    print(target)
