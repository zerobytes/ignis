#!/usr/bin/env python3
"""Real MCP: publish one shared surface, walk it, persist and replay without a window."""
import json
import copy
from pathlib import Path
from verify_lab import Client, ROOT
from verify_terrain import position


def verify(binary,window=False):
    c=Client(binary,window)
    checks=[]
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'terrain-collision','version':'1'}})
        c.request('notifications/initialized',notification=True)
        c.call('generate',recipe={'width':30,'depth':24,'seed':182,'obstacles':0})
        initial=c.call('inspect')['scene']
        player=copy.deepcopy(next(e for e in initial['entities'] if e['id']=='player'))
        player['transform']={'position':[-10,1.3,-2],'size':[.6,1.6,.6]}
        player.pop('footsteps',None)
        player['character_controller']={}
        c.call('edit_scene',edits=[{'op':'remove','id':e['id']} for e in initial['entities'] if e['id']!='player']+[{'op':'upsert','entity':player}])
        doc={'version':1,'space_version':1,'id':'walking_hills','revision':1,'datum':position(0,0),'base':{'kind':'constant','height_meters':0},'operations':[{'kind':'raise','center':position(0,0),'radius_meters':10,'amount_meters':3},{'kind':'noise','seed':182,'period_meters':16,'octaves':2,'persistence':.4,'amplitude_meters':.12,'style':'smooth'}],'water_level_meters':None}
        patches=[{'origin':position(x,-8),'cells':[24,32],'spacing_meters':.5} for x in [-12,0]]
        c.call('save_terrain',path='examples/terrain-walk.ignis-terrain',document=doc)
        for index,patch in enumerate(patches):
            pub={'document':doc,'patch':patch,'stride':1,'asset':f'ground_{index}','instance':f'ground_{index}','frame_origin':position(0,0),'color':[.3,.48,.22],'collision':True}
            before=c.call('inspect')
            bad=copy.deepcopy(pub);bad['instance']=''
            c.call('publish_terrain',publication=bad,expected_error=True)
            assert c.call('inspect')==before
            result=c.call('publish_terrain',publication=pub)
            assert not result['visual_only']
        bound=c.call('inspect')['scene']
        c.call('edit_scene',edits=[{'op':'set_mesh_collision','instance':'ground_0','enabled':False}])
        assert c.call('inspect_mesh_collisions')['instances']==['ground_1']
        c.call('undo')
        assert c.call('inspect')['scene']==bound
        assert len(c.call('inspect_mesh_collisions')['instances'])==2
        # A native arch intersects the hill with an actual opening; no proxy collider.
        form=c.call('query_landform',query={'kind':'preset','id':'rock_arch','origin':position(0,0),'preset':{'kind':'arch','outer_radius_meters':3,'inner_radius_meters':2,'leg_height_meters':2,'depth_meters':1,'segments':16}})['document']
        for p in form['pieces']:
            p['anchor'][1]+=2
        c.call('save_landform',path='examples/terrain-walk.ignis-landform',document=form)
        c.call('publish_landform',publication={'document':form,'asset':'rock_arch','instance':'rock_arch','frame_origin':position(0,0),'color':[.5,.43,.32],'shading':'smooth','collision':True})
        c.call('set_camera',camera={'position':[22,18,-24],'target':[0,1.5,0],'fov_degrees':48})
        c.call('save_scene',path='examples/terrain-walk.scene.json')
        recipe={'document':doc,'patches':patches,'landform':form,'source_revision':1,'description':'Local walking demo; exact mesh collisions and current renderer. No world streaming or automatic navigation promised.'}
        (ROOT/'examples/terrain-walk.recipe.json').write_text(json.dumps(recipe,indent=2)+'\n')
        checks.append('Transactional native Terrain/Landform publication binds the same full meshes, with late rejection and no hidden floor')
        c.call('step',ticks=45)
        start=c.call('inspect')['runtime_positions']['player']
        c.call('set_input',input={'x':1,'z':0})
        c.call('step',ticks=180)
        summit=c.call('inspect')
        assert summit['runtime_positions']['player'][0]>0, summit['runtime_positions']['player']
        assert summit['runtime_positions']['player'][1]>start[1]+2, (start,summit['runtime_positions']['player'])
        assert summit['character_controllers']['player']['grounded']
        c.call('set_input',input={'x':0,'z':0})
        c.call('step',ticks=1)
        c.call('save_checkpoint',path='artifacts/terrain-walk.ignis-save')
        c.call('save_portable_save',path='artifacts/terrain-walk.ignis-portable')
        c.call('save_replay',path='artifacts/terrain-walk.replay.json')
        assert c.call('run_replay',path='artifacts/terrain-walk.replay.json')['matched']
        path='artifacts/terrain-walk.png'
        c.call('capture',path=path,width=1280,height=800)
        c.call('capture_replay',replay_path='artifacts/terrain-walk.replay.json',path='artifacts/terrain-walk-replay.png',tick=226,width=1280,height=800)
        assert (ROOT/path).read_bytes()==(ROOT/'artifacts/terrain-walk-replay.png').read_bytes()
        checks.append('Real KCC climbs the sampled hill and crosses the patch seam; grounded state, replay and native render agree')
        for op,path in [('load_checkpoint','artifacts/terrain-walk.ignis-save'),('load_portable_save','artifacts/terrain-walk.ignis-portable')]:
            c.call(op,path=path)
            assert c.call('inspect_mesh_collisions')['instances']==['ground_0','ground_1','rock_arch']
            c.call('set_input',input={'x':1,'z':0})
            c.call('step',ticks=130)
            end=c.call('inspect')['runtime_positions']['player']
            assert end[0]>6 and end[1]<summit['runtime_positions']['player'][1],end
        c.call('load_scene',path='examples/terrain-walk.scene.json')
        player['transform']['position']=[0,4,-3]
        c.call('edit_scene',edits=[{'op':'upsert','entity':player}])
        c.call('step',ticks=45)
        c.call('set_input',input={'x':0,'z':1})
        c.call('step',ticks=90)
        through=c.call('inspect')['runtime_positions']['player']
        assert through[2]>2,through
        c.call('load_scene',path='examples/terrain-walk.scene.json')
        checks.append('Native and portable reload reconstruct the surface and continue downhill; a separate real KCC crossing traverses the arch opening')
        report={'passed':True,'build_id':c.call('inspect')['build_id'],'checks':checks,'native_window':window,'scope':{'real_mesh_collision':True,'streaming':False,'combat_mesh_occlusion':False,'human_acceptance':False}}
        (ROOT/'artifacts/terrain-collision-verification.json').write_text(json.dumps(report,indent=2)+'\n')
        print(json.dumps({'passed':True,'groups':len(checks),'build_id':report['build_id']}))
    finally:
        c.close()

if __name__=='__main__':
    import argparse
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');p.add_argument('--window',action='store_true')
    args=p.parse_args();verify(args.binary.resolve(),args.window)
