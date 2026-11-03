"""Live authoring gestures, component transforms and discoverable NPC/rig contracts."""
import argparse,copy,json
from pathlib import Path
from verify_lab import Client,ROOT

def verify(binary,window):
    if window:raise ValueError('Desktop presentation is a separate integration gate')
    c=Client(binary,False);checks=[]
    try:
        c.request('initialize',dict(protocolVersion='2025-11-25',capabilities={},clientInfo=dict(name='editor-authoring',version='1')));c.request('notifications/initialized',notification=True)
        c.call('generate',recipe=dict(width=16,depth=14,seed=42,obstacles=0))
        original=c.call('inspect')['scene'];p=copy.deepcopy(next(e for e in original['entities'] if e['id']=='player'))
        before=c.call('inspect')['history']['undo']
        for i in range(5):
            p['transform']['position'][0]+=.1
            c.call('edit_scene_live',gesture='mcp-drag',edits=[dict(op='upsert',entity=p)])
            assert c.call('inspect')['history']['undo']==before+1
        latest=c.call('inspect');bad=copy.deepcopy(p);bad['transform']['size'][0]=-1
        c.call('edit_scene_live',gesture='mcp-drag',edits=[dict(op='upsert',entity=bad)],expected_error=True);assert c.call('inspect')==latest
        c.call('undo');assert c.call('inspect')['scene']==original
        c.call('redo');assert c.call('inspect')['scene']==latest['scene']
        checks.append('Live updates appear immediately, share one undo and reject invalid updates without mutation')
        c.call('create_editor_mesh',primitive='plane',resolution=8,size=2)
        selected=c.call('inspect_editor')['selected_mesh'];asset=c.call('inspect_mesh',asset=selected)['asset'];ref=c.call('inspect_mesh_topology',asset=selected)['reference']
        edit=dict(kind='transform_vertices',asset=selected,reference=ref,vertices=[40],transform=dict(pivot=[0,0,0],translation=[0,.1,0],rotation=[0,0,0,1],scale=1))
        predicted=c.call('preview_mesh_edit',edit=edit)['meshes'];c.call('edit_mesh',edit=edit);assert c.call('inspect')['scene']['meshes']==predicted
        state=c.call('inspect');c.call('edit_mesh',edit=edit,expected_error=True);assert c.call('inspect')==state
        changed=c.call('inspect_mesh',asset=selected)['asset'];assert asset['geometry']['triangles']==changed['geometry']['triangles']
        assert changed['geometry']['material_coordinates']==asset['geometry']['positions']
        checks.append('Revision-scoped vertex transforms preserve topology/material domain; preview matches commit and stale references fail atomically')
        rig=dict(bones=[dict(name='root',parent=None,translation=[0,0,0],rotation=[0,0,0,1]),dict(name='child',parent=0,translation=[0,.25,0],rotation=[0,0,0,1])],weights=[dict(joints=[0,0,0,0],weights=[1,0,0,0]) for _ in changed['geometry']['positions']])
        rig['weights'][40]=dict(joints=[1,0,0,0],weights=[1,0,0,0])
        c.call('edit_mesh',edit=dict(kind='set_rig',asset=selected,rig=rig))
        clip=dict(id='gesture',duration_ticks=120,tracks=[dict(bone=1,keys=[dict(tick=0,translation=[0,.25,0],rotation=[0,0,0,1]),dict(tick=60,translation=[0,.5,0],rotation=[0,0,0,1])])])
        c.call('edit_mesh',edit=dict(kind='put_clip',asset=selected,clip=clip))
        instance=copy.deepcopy(next(i for i in c.call('inspect')['scene']['meshes']['instances'] if i['id']==selected));instance['playback']=dict(clip='gesture',offset_ticks=0,looping=True)
        c.call('edit_mesh',edit=dict(kind='put_instance',instance=instance))
        state=c.call('inspect');c.call('set_editor_mesh_view',view=dict(edges=True,vertices=True,bones=True));assert c.call('inspect')==state
        c.call('capture_editor',path='artifacts/authoring-components.png',width=1440,height=900);assert c.call('inspect')==state
        c.call('save_scene',path='examples/editor-authoring.scene.json');authored=c.call('inspect')['scene'];c.call('step',ticks=61)
        c.call('save_replay',path='artifacts/editor-authoring.replay.json');assert c.call('run_replay',path='artifacts/editor-authoring.replay.json')['matched']
        c.call('load_scene',path='examples/editor-authoring.scene.json');assert c.call('inspect')['scene']==authored
        checks.append('Real rigs, selected weights and clips coexist with pure session overlays and survive save/load/replay')
        c.call('create_scene_object',kind='npc');state=c.call('inspect');assert c.call('inspect_editor')['selected_mesh'] is None
        npc=copy.deepcopy(next(e for e in state['scene']['entities'] if e['id']==state['selection']))
        assert npc['npc']['enabled'];npc['npc']['target']='player'
        npc['npc']['awareness']=dict(mode='perceptive',vision_range=12,fov_degrees=100,initial_facing_degrees=0,memory_ticks=180,search_ticks=240,search_turn_degrees_per_second=90)
        c.call('edit_scene_live',gesture='npc-components',edits=[dict(op='upsert',entity=npc)])
        assert next(e for e in c.call('inspect')['scene']['entities'] if e['id']==npc['id'])['npc']['target']=='player'
        c.call('capture_editor',path='artifacts/authoring-npc.png',width=1440,height=900)
        checks.append('NPC creation selects the character and exposes validated live target/perception components through the shared API')
        report=dict(passed=True,checks=checks,build_id=c.call('inspect')['build_id'],native_window=False)
        (ROOT/'artifacts/editor-authoring-verification.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({**report,'checks':len(checks)}))
    finally:c.close()
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=Path(__file__).resolve().parents[1]/'target/release/ignis');p.add_argument('--window',action='store_true');args=p.parse_args();verify(args.binary,args.window)
