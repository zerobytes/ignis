"""MCP-first convex union transaction, preview, undo and replay regression."""
import json,copy
from verify_lab import Client,ROOT
from open_earth_walk import init

def main():
 out=ROOT/'artifacts/weathered-plateau';out.mkdir(parents=True,exist_ok=True)
 c=Client(ROOT/'target/release/ignis',log_path=out/'union-test.log')
 try:
  init(c);c.call('generate',recipe=dict(width=12,depth=12,seed=221,obstacles=0));c.call('create_editor_mesh',primitive='cube',resolution=4,size=2);name=c.call('inspect_editor')['selected_mesh']
  ref=c.call('inspect_mesh_topology',asset=name)['reference'];base=c.call('inspect')
  lo=[.4,-.7,.4];hi=[1.6,.6,1.3];points=[[hi[a] if i&(1<<a) else lo[a] for a in range(3)] for i in range(8)]
  op=dict(kind='union_convex',points=points)
  preview=c.call('model_editor_mesh',instance=name,reference=ref,operation=op,preview=True);assert c.call('inspect')==base
  c.call('model_editor_mesh',instance=name,reference=ref,operation=op);final=c.call('inspect');assert final['scene']==preview['scene']
  c.call('undo');assert c.call('inspect')['scene']==base['scene'];c.call('redo');assert c.call('inspect')['scene']==final['scene']
  frozen=c.call('inspect');c.call('model_editor_mesh',instance=name,reference=ref,operation=op,expected_error=True);assert c.call('inspect')==frozen
  ref=c.call('inspect_mesh_topology',asset=name)['reference'];invalid=dict(kind='union_convex',points=[[3+(i&1),3+((i>>1)&1),3+((i>>2)&1)] for i in range(8)])
  c.call('model_editor_mesh',instance=name,reference=ref,operation=invalid,expected_error=True);assert c.call('inspect')==frozen
  c.call('play');frozen=c.call('inspect');c.call('model_editor_mesh',instance=name,reference=ref,operation=op,expected_error=True);assert c.call('inspect')==frozen;c.call('pause')
  c.call('step',ticks=2);c.call('save_replay',path='artifacts/weathered-plateau/union-test.replay.json');r=c.call('run_replay',path='artifacts/weathered-plateau/union-test.replay.json');assert r['matched'] and r['meshes_matched']
  c.call('generate',recipe=dict(width=12,depth=12,seed=221,obstacles=0));c.call('create_editor_mesh',primitive='cube',resolution=4,size=2);name=c.call('inspect_editor')['selected_mesh']
  inst=next(i for i in c.call('inspect_meshes')['instances'] if i['id']==name);inst['position']=[0,1,0];c.call('edit_mesh',edit=dict(kind='put_instance',instance=inst))
  lo=[-.6,.5,-.6];hi=[.6,2,.6];points=[[hi[a] if i&(1<<a) else lo[a] for a in range(3)] for i in range(8)]
  c.call('model_editor_mesh',instance=name,reference=c.call('inspect_mesh_topology',asset=name)['reference'],operation=dict(kind='union_convex',points=points))
  player=copy.deepcopy(next(e for e in c.call('inspect')['scene']['entities'] if e['id']=='player'));player['transform']=dict(position=[0,5.5,0],size=[.2,.4,.2]);player['character_controller']={}
  c.call('edit_scene',edits=[dict(op='upsert',entity=player),dict(op='set_mesh_collision',instance=name,enabled=True)]);c.call('step',ticks=90)
  state=c.call('inspect');assert abs(state['runtime_positions']['player'][1]-3.2)<.03,state['runtime_positions']['player'];assert state['character_controllers']['player']['grounded']
  report=dict(kcc_on_added_surface=True,build=c.call('inspect')['build_id'],passed=True,preview_undo_redo=True,stale_disconnected_play_atomic=True,replay=True)
  (out/'union-verification.json').write_text(json.dumps(report,indent=2));print(json.dumps(report))
 finally:c.close()
if __name__=='__main__':main()
