#!/usr/bin/env python3
"""Author the GPU Gems density experiment through MCP; never calculate mesh vertices."""
import argparse,copy,json
from verify_lab import Client,ROOT
from open_earth_walk import init
PREFIX='artifacts/volumetric-terrain'
OUT=ROOT/PREFIX

def recipe(seed=2718):
    # Near-doubling frequencies and unequal rotation per octave; all exposed source data.
    octaves=[dict(frequency=f*16,amplitude=a,rotation_degrees=r) for f,a,r in [
        (.16336,.3712,[0,0,0]),(.0788,.7168,[0,0,0]),(.04204,1.3824,[0,0,0]),
        (.0204,2.6624,[0,0,0]),(.00968,5,[0,0,0]),(.00497,10,[0,0,0]),
        (.0026125,18,[19,37,11]),(.0011664,32,[41,17,53])]]
    return dict(version=1,seed=seed,base=dict(kind='plane',height=0),vertical_strength=1,
        octaves=octaves[3:],warp=dict(frequency=.0041*16,amplitude=2),
        terraces=dict(spacing=7.6923076923,strength=4,overhang=1.6,sharpness=16,warp=.15,phase=0),
        shelf=None,
        floor=dict(altitude=-30,transition=3,strength=40))

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--seed',type=int,default=2718);parser.add_argument('--backend',choices=['gpu','cpu'],default='gpu');args=parser.parse_args()
    OUT.mkdir(parents=True,exist_ok=True);doc=recipe(args.seed)
    grid=dict(origin=[-32,-24,-32],cells=[128,96,128],spacing=.5)
    views=dict(overview=dict(position=[46,34,-55],target=[0,-1,0],fov_degrees=50),
        ledges=dict(position=[17,12,-23],target=[0,0,0],fov_degrees=55),
        reverse=dict(position=[-47,32.5,45],target=[0,-2,0],fov_degrees=52))
    c=Client(ROOT/'target/release/ignis',log_path=OUT/'native.log')
    try:
        init(c);schema=json.dumps(c.request('tools/list')['result']);assert 'publish_volume_terrain_gpu' in schema
        c.call('generate',recipe=dict(width=20,depth=20,seed=221,obstacles=0))
        scene=c.call('inspect')['scene'];player=copy.deepcopy(next(e for e in scene['entities'] if e['id']=='player'))
        player['transform']['position']=[0,-90,0]
        edits=[dict(op='remove',id=e['id']) for e in scene['entities'] if e['id']!='player']+[dict(op='upsert',entity=player)]
        c.call('edit_scene',edits=edits)
        doc_path=f'{PREFIX}/source.ignis-volume';c.call('save_volume_terrain',path=doc_path,document=doc)
        loaded=c.call('load_volume_terrain',path=doc_path)['document'];assert loaded==doc
        material=dict(pattern=dict(kind='ridged',frequency=[.13,.17,.13],low=[.68,.69,.66],high=[1,.98,.9],bump=.025,warp=.35,octaves=3,seed=47,contrast=1.1),reflectance=dict(roughness=.95,f0=.02))
        publication=dict(document=doc,grid=grid,asset='volume',instance='volume',position=grid['origin'],color=[.51,.48,.40],material=material,collision=True)
        before=c.call('inspect')['revision']
        result=c.call('publish_volume_terrain_gpu' if args.backend=='gpu' else 'publish_volume_terrain',publication=publication)
        assert not result['visual_only']
        c.call('undo');assert not any(a['id']=='volume' for a in c.call('inspect_meshes')['assets'])
        c.call('redo');assert any(a['id']=='volume' for a in c.call('inspect_meshes')['assets'])
        # Read-only vertical density probes: detect genuine solid/air/solid transitions.
        columns=[(x,z) for x in [-24,-16,-8,0,8,16,24] for z in [-24,-16,-8,0,8,16,24]]
        probes=[[x,-24+k*.25,z] for x,z in columns for k in range(193)]
        samples=[]
        for k in range(0,len(probes),4096):
            samples.extend(c.call('query_volume_terrain',query=dict(kind='samples',document=doc,points=probes[k:k+4096]))['samples'])
        crossings=[]
        for i in range(len(columns)):
            signs=[v['density']>0 for v in samples[i*193:(i+1)*193]]
            crossings.append(sum(a!=b for a,b in zip(signs,signs[1:])))
        assert max(crossings)>=3,crossings
        # A failed recipe must leave the published scene unchanged.
        revision=c.call('inspect')['revision'];bad=copy.deepcopy(publication);bad['document']['terraces']['spacing']=0
        c.call('publish_volume_terrain_gpu',publication=bad,expected_error=True)
        assert c.call('inspect')['revision']==revision
        c.call('set_light',light=dict(direction=[.5,-.78,.36],ambient=.22,direct_intensity=1.25,direct_color=[1,.96,.88]))
        # The same extracted mesh is a real Rapier collider, not density-only proof.
        center=samples[columns.index((0,0))*193:(columns.index((0,0))+1)*193]
        solid=[v['position'][1] for v in center if v['density']>0]
        player['transform']['position']=[0,max(solid)+2,0]
        player['character_controller']=dict(max_slope_degrees=45,slide_slope_degrees=50,offset=.01,snap_distance=.2)
        c.call('edit_scene',edits=[dict(op='upsert',entity=player)])
        c.call('set_execution_mode',mode='continuous');c.call('step',ticks=120)
        grounded=c.call('inspect_character_controllers')['controllers']['player']['grounded'];assert grounded
        c.call('set_camera',camera=views['overview']);c.call('save_scene',path=f'{PREFIX}/world.scene.json')
        c.call('load_scene',path=f'{PREFIX}/world.scene.json')
        for name,camera in views.items():
            c.call('set_camera',camera=camera);c.call('capture',path=f'{PREFIX}/{name}.png',width=1280,height=900)
        c.call('set_camera',camera=views['overview']);c.call('load_scene',path=f'{PREFIX}/world.scene.json')
        c.call('capture',path=f'{PREFIX}/reload.png',width=1280,height=900)
        assert (OUT/'reload.png').read_bytes()==(OUT/'overview.png').read_bytes()
        empty=copy.deepcopy(doc);empty.update(base=dict(kind='plane',height=-100),octaves=[],warp=None,terraces=None,shelf=None,floor=None)
        assert c.call('query_volume_terrain_gpu',document=empty,grid=dict(origin=[0,0,0],cells=[4,4,4],spacing=1))['empty']
        c.call('set_execution_mode',mode='recorded');c.call('step',ticks=2)
        saved=c.request('tools/call',dict(name='engine_command',arguments=dict(command=dict(op='save_replay',path=f'{PREFIX}/world.replay.json'))))['result']
        replay=dict(saved=not saved['isError'])
        if replay['saved']:
            checked=c.call('run_replay',path=f'{PREFIX}/world.replay.json');assert checked['matched'] and checked['meshes_matched'];replay['matched']=True
        else:replay['error']=saved.get('structuredContent',saved)
        report=dict(terrain=result['terrain'],source_roundtrip=True,scene_reload_pixels_equal=True,failed_publication_atomic=True,undo_redo=True,
            vertical_probes=dict(columns=len(columns),max_crossings=max(crossings),columns_with_overhang=sum(n>=3 for n in crossings)),
            collider_grounded=True,empty_grid_inspected=True,replay=replay,native_window=False,human_acceptance=False)
        (OUT/'verification.json').write_text(json.dumps(report,indent=2)+'\n')
        (ROOT/'examples/volume-terrain.recipe.json').write_text(json.dumps(dict(document=doc,grid=grid,publication=publication,views=views),indent=2)+'\n')
        print(json.dumps(report))
    finally:c.close()
if __name__=='__main__':main()
