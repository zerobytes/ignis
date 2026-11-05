#!/usr/bin/env python3
"""Author game walk/run clips through native Ignis pose/IK and save the editable library."""
import argparse
import copy
import json
import math
from pathlib import Path
import subprocess

GAME = Path(__file__).resolve().parent
ROOT = GAME.parents[1]
LIBRARY = "games/land-of-survivors/assets/animations/locomotion.ignis-motion"
SCENE = "games/land-of-survivors/artifacts/app/animation.scene.json"


def qmul(a, b):
    x, y, z, w = a
    X, Y, Z, W = b
    return [w*X+x*W+y*Z-z*Y, w*Y-x*Z+y*W+z*X,
            w*Z+x*Y-y*X+z*W, w*W-x*X-y*Y-z*Z]


def inverse(q):
    return [-q[0], -q[1], -q[2], q[3]]


def axis(angle, index):
    q = [0., 0., 0., math.cos(math.radians(angle)*.5)]
    q[index] = math.sin(math.radians(angle)*.5)
    return q


def rotation(matrix):
    # Native matrices are column-major. Extract a unit rotation, no scale/shear.
    m = [[matrix[c][r] for c in range(3)] for r in range(3)]
    trace = sum(m[i][i] for i in range(3))
    if trace > 0:
        s = math.sqrt(trace+1)*2
        q = [(m[2][1]-m[1][2])/s, (m[0][2]-m[2][0])/s,
             (m[1][0]-m[0][1])/s, .25*s]
    else:
        i = max(range(3), key=lambda j:m[j][j]); j=(i+1)%3; k=(i+2)%3
        s=math.sqrt(max(0.,1+m[i][i]-m[j][j]-m[k][k]))*2
        q=[0.,0.,0.,0.];q[i]=.25*s;q[j]=(m[j][i]+m[i][j])/s
        q[k]=(m[k][i]+m[i][k])/s;q[3]=(m[k][j]-m[j][k])/s
    length=math.sqrt(sum(v*v for v in q))
    return [v/length for v in q]


def rotate(q,v):
    return qmul(qmul(q,[*v,0.]),inverse(q))[:3]


def cross(a,b):
    return [a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]]


def normalized(v):
    length=math.sqrt(sum(x*x for x in v))
    return [x/length for x in v]


def smooth(x):
    x=max(0.,min(1.,x)); return x*x*(3-2*x)


def hermite_curve(u,points):
    for (a,x,dx),(b,y,dy) in zip(points,points[1:]):
        if u<=b:
            t=(u-a)/(b-a);t2=t*t;t3=t2*t
            return (2*t3-3*t2+1)*x+(t3-2*t2+t)*dx*(b-a)+(-2*t3+3*t2)*y+(t3-t2)*dy*(b-a)
    return points[-1][1]


def curve(u, points):
    for (a,x),(b,y) in zip(points,points[1:]):
        if u<=b: return x+(y-x)*smooth((u-a)/(b-a))
    return points[-1][1]


class Native:
    def __init__(self):
        self.process = subprocess.Popen([str(ROOT/'target/release/ignis'), '--root',str(ROOT),'--mcp'],
                                        stdin=subprocess.PIPE,stdout=subprocess.PIPE,text=True)
        self.sequence=0
        self.rpc('initialize',{'protocolVersion':'2025-11-25','capabilities':{},
                              'clientInfo':{'name':'los-locomotion-author','version':'1'}})
        self.process.stdin.write(json.dumps({'jsonrpc':'2.0','method':'notifications/initialized'})+'\n')
        self.process.stdin.flush()

    def rpc(self, method, params):
        self.sequence+=1
        self.process.stdin.write(json.dumps({'jsonrpc':'2.0','id':self.sequence,'method':method,'params':params})+'\n')
        self.process.stdin.flush()
        reply=json.loads(self.process.stdout.readline())
        if 'error' in reply: raise RuntimeError(reply['error'])
        return reply['result']

    def animation(self,kind,**fields):
        result=self.rpc('tools/call',{'name':'engine_command','arguments':{
            'command':{'op':'animation','operation':{'kind':kind,**fields}}}})
        data=json.loads(result['content'][0]['text'])
        if result.get('isError'): raise RuntimeError(data)
        return data

    def close(self):
        self.process.stdin.close();self.process.wait()


def author(recipe):
    # Only clips and support metadata are published; no character mesh/save writes.
    if not (ROOT/SCENE).is_file():
        subprocess.run([str(ROOT/'target/release/examples/land_of_survivors_app'),
                        '--export-animation-scene',SCENE],check=True,cwd=ROOT)
    scene=json.loads((ROOT/SCENE).read_text())
    body=next(a for a in scene['meshes']['assets'] if a['rig'] and a['clips'])
    source_height=max(p[1] for p in body['geometry']['positions'])-min(p[1] for p in body['geometry']['positions'])
    unit=source_height/recipe['character_height']
    native=Native()
    try:
        native.animation('load',path=LIBRARY)
        document=native.animation('inspect')['document']
        bones=document['bones'];indices={b['name']:i for i,b in enumerate(bones)}
        idle=next(c for c in document['clips'] if c['id']=='los_idle')
        reference=copy.deepcopy(document)
        reference['clips'][reference['clips'].index(idle)]={'id':'los_idle','duration_ticks':1,
            'tracks':[{'bone':0,'keys':[{'tick':0,'translation':bones[0]['translation'],'rotation':bones[0]['rotation']}]}]}
        native.animation('replace',document=reference)
        rest=native.animation('sample',motion='idle',phase=0)['bones']
        matrices={b['name']:b['matrix'] for b in rest}
        rotations={n:rotation(m) for n,m in matrices.items()}
        original_feet=copy.deepcopy(next(m for m in document['motions'] if m['name']=='walk')['feet'])
        soles={}
        for foot in original_feet:
            ankle=matrices[foot['name']][3][:3]
            side=-1 if foot['name'].endswith('_L') else 1
            soles[foot['name']]=[(v[1]-ankle[1],v[2]-ankle[2]) for v in body['geometry']['positions']
                                if v[1]<ankle[1]+.01 and v[0]*side>0]
        def roll_support(foot,pitch):
            a=math.radians(pitch);c=math.cos(a);s=math.sin(a)
            y,z=min(soles[foot['name']],key=lambda v:c*v[0]-s*v[1])
            return -foot['sole_offset']-(c*y-s*z), z-(s*y+c*z)
        elbow_rest={};palm_normals={}
        for side,sign in [('L',1),('R',-1)]:
            def point(n):return matrices[n+'_'+side][3][:3]
            def span(a,b):return [y-x for x,y in zip(point(a),point(b))]
            upper=rotate(axis(sign*33.,2),span('upperarm01','lowerarm01'))
            lower=rotate(axis(sign*33.,2),span('lowerarm01','wrist'))
            angle=lambda v:math.degrees(math.atan2(-v[2],-v[1]))
            elbow_rest[side]=angle(lower)-angle(upper)
            normal=normalized(cross(span('finger2-1','finger5-1'),span('wrist','finger3-1')))
            if normal[2]>0:normal=[-x for x in normal]
            palm_normals[side]=normal
        plans={}
        for name in ['walk','run']:
            p=recipe[name];duration=p['duration_ticks'];stride=p['stride_metres']*unit
            d=p['stance_fraction'];tracks={};targets=[]
            def key(bone,tick,q,translation=None):
                index=indices[bone]
                tracks.setdefault(index,[]).append({'tick':tick,'translation':translation or bones[index]['translation'], 'rotation':q})
            def delta(bone,tick,x=0.,y=0.,z=0.,translation=None):
                world=qmul(qmul(axis(x,0),axis(y,1)),axis(z,2));g=rotations[bone]
                local=qmul(qmul(inverse(g),world),g)
                key(bone,tick,qmul(bones[indices[bone]]['rotation'],local),translation)
            for tick in range(duration+1):
                phase=(tick/duration)%1; wave=2*math.pi*phase
                if name=='walk':
                    drop=(-p['root_drop_metres']+p['root_bob_metres']*math.sin(wave)**2)*unit
                else:
                    drop=(-p['root_drop_metres']-p['root_bob_metres']*math.cos(2*wave-4*math.pi*.12))*unit
                yaw=math.cos(wave)*(3.5 if name=='walk' else 5.)
                roll=math.sin(wave)*(2. if name=='walk' else 3.)
                root=bones[0]['translation'].copy();root[1]+=drop
                delta('root',tick,x=p['lean_degrees'],y=yaw,z=roll,translation=root)
                delta('spine03',tick,x=-p['lean_degrees']*.18,y=-yaw*1.6,z=-roll*.6)
                delta('spine01',tick,x=-p['lean_degrees']*.1,y=0.,z=0.)
                delta('head',tick,x=-p['lean_degrees']*.72,y=yaw*.4,z=-roll*.4)
                for side,sign in [('L',1),('R',-1)]:
                    swing=-sign*math.cos(wave-.12)*p['arm_swing_degrees']
                    delta('upperarm01_'+side,tick,x=swing,z=sign*33.)
                    delta('lowerarm01_'+side,tick,x=p['elbow_degrees']-elbow_rest[side]+sign*math.sin(wave)*4.)
                frame_targets=[];desired={}
                for foot in original_feet:
                    side=foot['name'][-1];sign=-1 if side=='L' else 1
                    phase_foot=(phase+(0 if side=='L' else .5))%1
                    ankle=matrices[foot['name']][3][:3]
                    front=-stride*d*.5;back=stride*d*.5
                    # Running lands near the hips and leaves farther behind.
                    bias=(.075*unit if name=='run' else 0.)
                    if phase_foot<d:
                        s=phase_foot/d
                        z=front+stride*phase_foot+bias
                        lift=0.
                        pitch=curve(s,[(0,p['heel_strike_degrees']),(.15,0.),(.68,0.),(1,p['toe_off_degrees'])])
                        # Roll over the actual source sole instead of through the floor.
                        dy,dz=roll_support(foot,pitch)
                    else:
                        u=(phase_foot-d)/(1-d);u2=u*u;u3=u2*u
                        tangent=stride*(1-d)
                        z=(2*u3-3*u2+1)*back+(u3-2*u2+u)*tangent+(-2*u3+3*u2)*front+(u3-u2)*tangent+bias
                        if name=='run':
                            z=hermite_curve(u,[(0,back,tangent),(.15,back+.04*unit,0.),
                                (.68,front-.01*unit,-stride*.35),(.9,front-.055*unit,0.),(1,front,tangent)])+bias
                        peak=u if name=='walk' else u**.6
                        lift=p['clearance_metres']*unit*math.sin(math.pi*peak)**2
                        pitch=curve(u,[(0,p['toe_off_degrees']),(.28,-12. if name=='walk' else -62.),(.72,7.),(1,p['heel_strike_degrees'])])
                        # Same pivot correction fades into the swing arc at release.
                        fade=1-smooth(u/.2)+smooth((u-.8)/.2)
                        dy,dz=roll_support(foot,pitch)
                        dy*=fade;dz*=fade
                    point=[sign*(.115 if name=='walk' else .105)*unit,
                           document['floor_y']+foot['sole_offset']+lift+dy,ankle[2]+z+dz]
                    a=math.radians(pitch)
                    frame_targets.append({'chain':{'bones':[indices[n] for n in foot['bones']],
                        'pole':[0.,0.,-1.],'normal_axis':foot['normal_axis'],'probe_up':.25,'probe_down':.5,
                        'max_correction':2.,'support_radius':0.,'support_tolerance':.03,
                        'sole_offset':foot['sole_offset'],'max_slope_degrees':45.},
                        'point':point,'normal':[0.,math.cos(a),math.sin(a)],'weight':1.})
                    desired[foot['name']]=qmul(axis(pitch,0),rotations[foot['name']])
                targets.append((frame_targets,desired))
            # Relax the open source hands; running uses a loose grip, not splayed fingers.
            for b in bones:
                if not b['name'].startswith('finger'):continue
                side=b['name'][-1];finger=int(b['name'][6]);segment=int(b['name'][8])
                index=indices[b['name']]
                direction=rotate(rotations[b['name']],[0.,1.,0.])
                bend=normalized(cross(direction,palm_normals[side]))
                degrees=([14.,22.,10.] if name=='walk' else [45.,58.,32.])[segment-1]
                if finger==1:degrees*=.45
                a=math.radians(degrees)*.5
                world=[*(v*math.sin(a) for v in bend),math.cos(a)]
                q=rotations[b['name']]
                local=qmul(qmul(inverse(q),world),q)
                for tick in [0,duration]:key(b['name'],tick,qmul(b['rotation'],local))
            clip={'id':'los_'+name,'duration_ticks':duration,
                  'tracks':[{'bone':b,'keys':k} for b,k in sorted(tracks.items())]}
            document['clips'][next(i for i,c in enumerate(document['clips']) if c['id']==clip['id'])]=clip
            motion=next(m for m in document['motions'] if m['name']==name)
            motion['stride']=stride
            for f in motion['feet']:
                f['stance']=[0.,d] if f['name'].endswith('_L') else [.5,(.5+d)%1]
            plans[name]=(tracks,targets)
        native.animation('replace',document=document)
        for name,(tracks,targets) in plans.items():
            duration=recipe[name]['duration_ticks']
            # Set pelvis height from the real posed limb reach before solving the
            # feet. A smooth upper envelope keeps contact poses below extension.
            corrections=[]
            for tick,(frame_targets,_) in enumerate(targets[:-1]):
                pose=native.animation('sample',motion=name,phase=tick/duration)
                needed=0.
                for target in frame_targets:
                    a,b,c=target['chain']['bones']
                    hip,knee,ankle=[pose['bones'][i]['matrix'][3][:3] for i in [a,b,c]]
                    reach=(math.dist(hip,knee)+math.dist(knee,ankle))*.998
                    point=target['point'];horizontal=(point[0]-hip[0])**2+(point[2]-hip[2])**2
                    if horizontal>=reach*reach:
                        raise ValueError('Foot arc exceeds horizontal limb reach; shorten the recipe stride')
                    needed=max(needed,hip[1]-point[1]-math.sqrt(reach*reach-horizontal))
                corrections.append(needed)
            epsilon=.004*unit
            envelope=[]
            for i in range(duration):
                values=[corrections[(i+j)%duration]-.002*unit*j*j for j in range(-3,4)]
                top=max(values)
                envelope.append(top+epsilon*math.log(sum(math.exp((x-top)/epsilon) for x in values)))
            for tick,keyframe in enumerate(tracks[0]):
                keyframe['translation'][1]-=envelope[tick%duration]
            native.animation('put_clip',clip={'id':'los_'+name,'duration_ticks':duration,
                'tracks':[{'bone':b,'keys':k} for b,k in sorted(tracks.items())]})
            limited=0
            for tick,(frame_targets,desired) in enumerate(targets):
                sample=native.animation('sample',motion=name,phase=(tick/duration)%1,targets=frame_targets)
                limited+=int(sample['pose']['ik_limited']>0)
                for joint in sample['pose']['joints'][:sample['pose']['joint_count']]:
                    bone=joint['bone'];q=joint['rotation'];b=bones[bone]
                    if b['name'] in desired:
                        parent=rotation(sample['bones'][b['parent']]['matrix'])
                        q=qmul(inverse(parent),desired[b['name']])
                    tracks.setdefault(bone,[]).append({'tick':tick,'translation':joint['translation'],'rotation':q})
            clip={'id':'los_'+name,'duration_ticks':duration,
                  'tracks':[{'bone':b,'keys':k} for b,k in sorted(tracks.items())]}
            native.animation('put_clip',clip=clip)
            print(f'Authored {name}: {duration} ticks, {recipe[name]["stride_metres"]} m/cycle; reach limited in {limited} poses',flush=True)
        native.animation('save',path=LIBRARY)
        print('Saved '+LIBRARY,flush=True)
    finally:
        native.close()


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--recipe',type=Path,default=GAME/'content/locomotion.recipe.json')
    args=parser.parse_args();recipe=json.loads(args.recipe.read_text())
    if recipe['version']!=1: raise ValueError('Unsupported locomotion recipe')
    author(recipe)
