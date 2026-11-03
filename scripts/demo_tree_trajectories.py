#!/usr/bin/env python3
"""Apply a native axis to the existing linked branch; never rebuild the tree."""
import copy,json,math,os,shutil,hashlib,subprocess
from pathlib import Path
PROJECT=Path(__file__).resolve().parents[1]
OUTPUT=PROJECT/'artifacts/tree-trajectories'
(OUTPUT/'examples').mkdir(parents=True,exist_ok=True);(OUTPUT/'artifacts').mkdir(exist_ok=True)
os.environ['IGNIS_VERIFY_ROOT']=str(OUTPUT)
from verify_lab import Client

def fit_axis(stations,protected):
    # Fixed endpoints plus the actual rig socket; least squares along the free
    # scalar coefficient. This is authoring controls, not a mesh generator.
    t=stations[protected]['at'];v=3*(1-t)*t*t;u=3*(1-t)**2*t;w=t**3
    controls=[[0.,0.],[0.,0.],[0.,0.],stations[-1].get('offset',[0.,0.])]
    for axis in range(2):
        end=controls[-1][axis];anchor=stations[protected]['offset'][axis]
        a=(anchor-w*end)/v;b=-u/v
        num=den=0.
        for s in stations:
            q=s['at'];u2=3*(1-q)**2*q;v2=3*(1-q)*q*q
            x=u2+v2*b;y=s.get('offset',[0,0])[axis]-q**3*end-v2*a
            num+=x*y;den+=x*x
        c1=num/den;controls[1][axis]=c1;controls[2][axis]=a+b*c1
    return controls

def run():
    snapshot=OUTPUT/'examples/input.scene.json'
    if not snapshot.exists():
        # A fixed Git input keeps reproduction valid after the maintained tree
        # advances; this is source data, never an evaluated mesh used as source.
        snapshot.write_bytes(subprocess.run(['git','show','5ee628d:examples/native-tree-study.scene.json'],cwd=PROJECT,check=True,stdout=subprocess.PIPE).stdout)
    c=Client(PROJECT/'target/release/ignis',False);call=c.call
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'tree-trajectories','version':'1'}});c.request('notifications/initialized',notification=True)
        call('load_scene',path='examples/input.scene.json')
        original=call('inspect')['scene'];id='branch-editable-1'
        old=call('inspect_plant',id=id)['plant'];d=copy.deepcopy(old['definition']);stations=d['sections']['stations']
        poses={(i['id'],tick):call('inspect_mesh_pose',instance=i['id'],tick=tick)['pose'] for i in original['meshes']['instances'] if i['asset']!=id for tick in [0,60]}
        controls=fit_axis(stations,2)
        d['trajectory']={'kind':'cubic_bezier','id':'branch1-axis','control_points_xz':controls,'surface_tolerance':.004}
        d['segments']=[6,6,6]
        for s in stations:s['offset']=[0,0]
        cameras={'whole':original['camera'],'branch':{'position':[3.5,3.2,4.7],'target':[.55,2.95,.13],'fov_degrees':30},'fork':{'position':[1.4,2.2,3.5],'target':[-.05,1.7,.1],'fov_degrees':40}}
        def capture(name,camera):
            call('set_camera',camera=camera);call('step',ticks=60);assert call('inspect')['tick']==60
            path=f'artifacts/{name}.png';call('capture',path=path,width=1200,height=1100)
            return hashlib.sha256((OUTPUT/path).read_bytes()).hexdigest()
        for name,camera in cameras.items():capture('before-'+name,camera)
        edit={'kind':'adopt_trajectory','definition':d,'detail_policy':'physical','animation_translation_policy':'physical'}
        state=call('inspect');preview=call('preview_plant',edit=edit);assert call('inspect')==state
        call('edit_plant',edit=edit);new=call('inspect_plant',id=id)
        assert new['plant']['source']==old['source'] and new['plant']['reference']==old['reference']
        updated=call('inspect')['scene'];assert updated['meshes']['instances']==original['meshes']['instances']
        assert [a for a in updated['meshes']['assets'] if a['id']!=id]==[a for a in original['meshes']['assets'] if a['id']!=id]
        call('undo');assert call('inspect_plant',id=id)['plant']==old;call('redo');assert call('inspect_plant',id=id)==new
        max_pose_error=0.
        for (instance,tick),before in poses.items():
            after=call('inspect_mesh_pose',instance=instance,tick=tick)['pose']
            error=max(math.dist(a,b) for a,b in zip(before['positions'],after['positions']))
            max_pose_error=max(max_pose_error,error);assert error<1e-6,(instance,tick,error)
        bent=next(a for a in updated['meshes']['assets'] if a['id']==id)
        for _ in range(2):
            changed=copy.deepcopy(d);changed['trajectory']['control_points_xz'][1][0]+=.015
            call('edit_plant',edit=dict(edit,kind='put',definition=changed))
            call('edit_plant',edit=dict(edit,kind='put'))
            assert call('inspect_plant',id=id)['plant']==new['plant']
            assert call('inspect_mesh',asset=id)['asset']==bent
        for name,camera in cameras.items():capture('after-'+name,camera)
        call('set_camera',camera=original['camera']);call('save_scene',path='examples/candidate.scene.json')
        call('capture',path='artifacts/start.png',width=1200,height=1100)
        call('step',ticks=60);call('capture',path='artifacts/live.png',width=1200,height=1100)
        call('save_replay',path='artifacts/tree.replay.json');assert call('run_replay',path='artifacts/tree.replay.json')['meshes_matched']
        for name,tick,source in [('replay',60,'live'),('prefix',0,'start')]:
            call('capture_replay',replay_path='artifacts/tree.replay.json',path=f'artifacts/{name}.png',tick=tick,width=1200,height=1100)
            assert (OUTPUT/f'artifacts/{name}.png').read_bytes()==(OUTPUT/f'artifacts/{source}.png').read_bytes()
        call('load_scene',path='examples/candidate.scene.json');loaded=call('inspect')['scene'];assert loaded['meshes']==updated['meshes'] and loaded['plants']==updated['plants']
        assert call('inspect_plant',id=id)['plant']==new['plant']
        (OUTPUT/'artifacts/report.json').write_text(json.dumps({'build_id':call('inspect')['build_id'],'report':new['report'],'canonical_source_and_reference_exact':True,'other_assets_and_instances_exact':True,'other_poses_max_error_m':max_pose_error,'edit_cycles_exact':True,'save_load_replay_prefix_exact':True,'definition':d},indent=2)+'\n')
        print(json.dumps(new['report'],indent=2))
    finally:c.close()
if __name__=='__main__':run()
