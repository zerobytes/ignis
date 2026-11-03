#!/usr/bin/env python3
"""Live canonical graft recipe, source preservation, editing and pixel replay."""
from verify_lab import SCENE_VERSION, REPLAY_VERSION
import argparse,copy,json
from pathlib import Path
from verify_lab import Client,ROOT
ID='branch-editable-1'

def verify(binary,window=False):
 c=Client(binary,window);call=c.call;checks=[]
 def entry():return call('inspect_plant',id=ID)['plant']
 def asset():return call('inspect_mesh',asset=ID)['asset']
 def put(d):return call('edit_plant',edit={'kind':'put','definition':d,'detail_policy':'physical','animation_translation_policy':'physical'})
 try:
  c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'plant-assemblies','version':'1'}});c.request('notifications/initialized',notification=True)
  schema=json.dumps(c.request('tools/list')['result']);assert 'set_assembly' in schema and 'preview_scene_edits' in schema
  call('load_scene',path='examples/native-tree-study.scene.json');canonical=entry();original=call('inspect')['scene']
  if canonical.get('assembly') is None:
   info=call('inspect_mesh_islands',asset='wood');assert info['topology']['islands'][0]['vertices']==2127
   extract={'kind':'extract_islands','asset':'wood','reference':info['reference'],'islands':[0],'id':'verification-context','mode':'move'}
   before=call('inspect');preview=call('preview_mesh_edit',edit=extract);assert call('inspect')==before
   context=next(a for a in preview['meshes']['assets'] if a['id']=='verification-context');instance=next(i for i in original['meshes']['instances'] if i['asset']==ID)
   assert instance['rotation']==[0,0,0,1] and instance['scale']==1
   for p in context['geometry']['positions']:
    for k in range(3):p[k]-=instance['position'][k]
   tile=set(range(943,1103));faces=[i for i,t in enumerate(context['geometry']['triangles']) if set(t)<=tile];assert len(faces)==256
   options=json.loads((ROOT/'examples/mesh-junction-union.input.json').read_text())['options']
   recipe={'context':context,'context_faces':faces,'context_loops':[list(range(943,975)),list(range(1071,1103))],'station':'branch1-030','context_bone':'base','options':options}
   edits=[{'op':'mesh','edit':extract},{'op':'mesh','edit':{'kind':'remove_asset','id':'verification-context'}},{'op':'plant','edit':{'kind':'set_assembly','id':ID,'assembly':recipe}}]
   before=call('inspect');predicted=call('preview_scene_edits',edits=edits)['scene'];assert call('inspect')==before
   call('edit_scene',edits=edits);assert call('inspect')['scene']==predicted
   assembled=call('inspect')['scene'];call('undo');assert call('inspect')['scene']==original;call('redo');assert call('inspect')['scene']==assembled
  linked=entry();out=asset();scene=call('inspect')['scene'];assert linked['source']==canonical['source'] and linked['reference']==canonical['reference'] and scene['meshes']['instances']==original['meshes']['instances']
  recipe=linked['assembly'];before=call('inspect');assert call('preview_scene_edits',edits=[{'op':'plant','edit':{'kind':'set_assembly','id':ID,'assembly':recipe}}])['scene']==before['scene'];assert call('inspect')==before;pred=call('preview_plant',edit={'kind':'set_assembly','id':ID,'assembly':recipe});assert call('inspect')==before and next(a for a in pred['assets'] if a['id']==ID)==out
  assert len(call('inspect_mesh_islands',asset=ID)['topology']['islands'])==1
  topology=call('inspect_mesh_topology',asset=ID)['topology'];assert topology['boundary_edges']==topology['non_manifold_edges']==0 and not topology['isolated_vertices']
  for _ in range(2):
   d=copy.deepcopy(linked['definition']);d['trajectory']['control_points_xz'][3][0]+=.001;put(d);assert asset()['geometry']['positions']!=out['geometry']['positions'];put(linked['definition']);assert asset()==out and entry()==linked
  call('edit_plant',edit={'kind':'set_assembly','id':ID,'assembly':None});assert entry()['source']==canonical['source'] and entry()['reference']==canonical['reference'];call('undo');assert entry()==linked and asset()==out
  checks.append('Native pure recipe preview and atomic publication, one connected manifold, source/reference/instances intact, reversible removal and repeated parameter cycles without baking or drift')
  call('step',ticks=13)
  for kind in ['station','root','grid','rebase','surface_rebase','mask','loft_cut','stale']:
   before=call('inspect');bad=copy.deepcopy(recipe)
   if kind=='station':bad['station']='missing'
   elif kind=='root':bad['context_bone']='missing'
   elif kind=='grid':bad['options']['cell_size']=.001
   elif kind=='mask':bad['context_surface']={'vertices':[99999],'anchors':[],'options':{'iterations':1,'strength':.1,'max_displacement':.001}}
   elif kind=='loft_cut' and bad.get('context_lofts'):
    preview=call('preview_mesh_ring_loft',source=bad['context'],lofts=bad['context_lofts']);bad['context_faces'].append(preview['report']['replaced_faces'][0])
   if kind in ['station','root','grid','mask','loft_cut']:call('edit_plant',edit={'kind':'set_assembly','id':ID,'assembly':bad},expected_error=True)
   elif kind=='surface_rebase':
    ref=call('inspect_mesh_topology',asset=ID)['reference'];call('edit_mesh',edit={'kind':'smooth_region','asset':ID,'reference':ref,'vertices':[0],'anchors':[],'options':{'iterations':1,'strength':.1,'max_displacement':.001}},expected_error=True)
   elif kind=='rebase':call('edit_mesh',edit={'kind':'set_rig','asset':ID,'rig':out['rig']},expected_error=True)
   else:
    ref=call('inspect_mesh_topology',asset=ID)['reference'];call('edit_scene',expected_error=True,edits=[{'op':'plant','edit':{'kind':'set_assembly','id':ID,'assembly':recipe}},{'op':'mesh','edit':{'kind':'subdivide','asset':ID,'reference':ref,'components':None}}])
   assert call('inspect')==before
  if recipe.get('context_grafts'):
   graft=recipe['context_grafts'][0];assert graft['source'].get('rig') is None and not graft['source'].get('clips')
   assert len(call('inspect_mesh_islands',asset='wood')['topology']['islands'])==176
   report=call('inspect_plant',id=ID)['report']['assembly']['context_grafts'][0]
   assert report['retained_source_vertices']==91 and report['junction']['retained_vertices_exact'] and report['junction']['retained_triangles_exact']
   assert report['surface']['bends_after']['maximum_degrees'] < report['surface']['bends_before']['maximum_degrees']
   for kind in ['duplicate_mask','generated_port','removed_child_port','source_faces','frame','grid','overlap','count','source_domain','source_id']:
    bad=copy.deepcopy(recipe);g=bad['context_grafts'][0]
    if kind=='duplicate_mask':g['context_vertices'].append(g['context_vertices'][0])
    elif kind=='generated_port':g['context_loops'][0][0]=len(bad['context']['geometry']['positions'])
    elif kind=='removed_child_port':g['source_loop'][0]=0
    elif kind=='source_faces':g['source_faces'].append(999999)
    elif kind=='frame':g['source_translation'][0]=10001
    elif kind=='grid':g['options']['cell_size']=.001
    elif kind=='overlap':bad['context_grafts'].append(copy.deepcopy(g))
    elif kind=='count':bad['context_grafts']=[copy.deepcopy(g) for _ in range(9)]
    elif kind=='source_domain':g['source']['geometry']['material_coordinates']=None
    else:g['source']['id']=bad['context']['id']
    state=call('inspect');edit={'kind':'set_assembly','id':ID,'assembly':bad}
    call('preview_plant',edit=edit,expected_error=True);assert call('inspect')==state
    call('edit_plant',edit=edit,expected_error=True);assert call('inspect')==state
   changed=copy.deepcopy(recipe);changed['context_lofts'][0]['spans']=4
   before=call('inspect');prediction=call('preview_plant',edit={'kind':'set_assembly','id':ID,'assembly':changed});assert call('inspect')==before
   call('edit_plant',edit={'kind':'set_assembly','id':ID,'assembly':changed});assert asset()!=out
   assert next(a for a in prediction['assets'] if a['id']==ID)==asset()
   call('edit_plant',edit={'kind':'set_assembly','id':ID,'assembly':recipe});assert asset()==out and entry()==linked
   checks.append('Canonical child frame/source, 91 retained points, closed connected side graft; generated/removed/duplicate IDs, frame/domain/grid/overlap/count reject atomically in preview and execution')
  if recipe.get('context_grafts') and recipe['context_grafts'][0].get('collar_surface'):
   collar=call('inspect_plant',id=ID)['report']['assembly']['context_grafts'][0]['collar_surface']
   assert collar['moved_retained_context']>0 and collar['moved_generated']>0 and collar['moved_retained_source']==0
   assert collar['filter']['bends_after']['maximum_degrees'] < collar['filter']['bends_before']['maximum_degrees']
   assert collar['filter']['max_displacement'] <= .010001
   for kind in ['empty','duplicate','port','rings','pins','excessive']:
    bad=copy.deepcopy(recipe);p=bad['context_grafts'][0]['collar_surface']
    if kind=='empty':p['ports']=[]
    elif kind=='duplicate':p['ports']=[0,0]
    elif kind=='port':p['ports']=[3]
    elif kind=='rings':p['rings']=17
    elif kind=='pins':p['fixed_rings']=0
    else:p['options'].update(iterations=32,max_displacement=.03)
    state=call('inspect');ed={'kind':'set_assembly','id':ID,'assembly':bad}
    call('preview_plant',edit=ed,expected_error=True);assert call('inspect')==state
    call('edit_plant',edit=ed,expected_error=True);assert call('inspect')==state
   checks.append('Collar selection derived from canonical ports with protected outer rings, intentional retained geometry finishing; source child fixed and malformed/overworked requests reject atomically')
  before=entry();call('edit_mesh',edit={'kind':'set_material','asset':ID,'material':{}});neutral=entry();assert neutral['source']['geometry']==before['source']['geometry'] and neutral['assembly']['context']['geometry']==before['assembly']['context']['geometry'];assert neutral['source']['material']==neutral['assembly']['context']['material'];assert all(g['source']['material']==neutral['source']['material'] and g['source']['geometry']==before['assembly']['context_grafts'][i]['source']['geometry'] for i,g in enumerate(neutral['assembly'].get('context_grafts',[])));call('undo');assert entry()==before
  checks.append('Invalid section/root/grid, direct rebase and stale batch reject with tick/history unchanged; appearance updates both immutable geometric sources with exact undo')
  call('set_camera',camera={'position':[3.5,3.2,4.7],'target':[.55,2.95,.13],'fov_degrees':30});call('save_scene',path='examples/plant-assembly.scene.json');saved=call('inspect')['scene'];assert saved['version']==SCENE_VERSION
  minimum=69 if any(g.get('collar_surface') for g in recipe.get('context_grafts',[])) else 68 if recipe.get('context_grafts') else 67 if any(recipe.get(k) for k in ['context_lofts','context_surface','junction_surface']) else 66;old=copy.deepcopy(saved);old['version']=minimum-1;(ROOT/'artifacts/plant-assembly-old.scene.json').write_text(json.dumps(old));before=call('inspect');err=call('load_scene',path='artifacts/plant-assembly-old.scene.json',expected_error=True);assert f'version{minimum}' in str(err) and call('inspect')==before
  def capture(name):
   path=f'artifacts/assembly-{name}.png';call('capture',path=path,width=1000,height=1000);return (ROOT/path).read_bytes()
  start=capture('start');call('step',ticks=60);live=capture('live');assert live!=start;assert capture('repeat')==live
  call('save_replay',path='artifacts/plant-assembly.replay.json');assert json.loads((ROOT/'artifacts/plant-assembly.replay.json').read_text())['version']==REPLAY_VERSION;assert call('run_replay',path='artifacts/plant-assembly.replay.json')['meshes_matched']
  for name,tick,pixels in [('replay',60,live),('prefix',0,start)]:
   path=f'artifacts/assembly-{name}.png';call('capture_replay',replay_path='artifacts/plant-assembly.replay.json',path=path,tick=tick,width=1000,height=1000);assert (ROOT/path).read_bytes()==pixels
  call('load_scene',path='examples/plant-assembly.scene.json');assert call('inspect')['scene']==saved and capture('loaded')==start and entry()==linked
  checks.append('Scene69 collar/graft/layers/storage gate, bounded save/load, live animation and exact repeat/replay74/prefix pixels with canonical recipe re-evaluation')
  result={'passed':True,'checks':checks,'native_window':window,'build_id':call('inspect')['build_id']};(ROOT/'artifacts/plant-assemblies-verification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'passed':True,'checks':len(checks),'build_id':result['build_id']}))
 finally:c.close()
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--window',action='store_true');a=p.parse_args();verify(a.binary,a.window)
