#!/usr/bin/env python3
"""Compare local junction resolution on the complete neutral ancestral attempt.
Reads the preserved game experiment; writes only artifacts/botany-junction-quality.
"""
import copy,json,time
from collections import Counter
from verify_lab import ROOT,Client
OUT=ROOT/'artifacts/botany-junction-quality'
PREFIX='elven_ancestral'
VIEWS=dict(front=([3,14,54],[0,13.5,0],36),side=([68,20,4],[0,13,0],42),back=([-6,18,-74],[0,13,0],40),top=([3,84,5],[0,4,0],40),junction=([14,15,17],[0,13,0],42),roots=([18,7,22],[0,1.4,0],44))
def main():
 OUT.mkdir(parents=True,exist_ok=True)
 c=Client(ROOT/'target/release/ignis',log_path=OUT/'native.log',startup_args=('--scene','games/land-of-survivors/artifacts/ancient-tree/current.scene.json'))
 try:
  c.request('initialize',dict(protocolVersion='2025-11-25',capabilities={},clientInfo=dict(name='junction-quality',version='1')));c.request('notifications/initialized',notification=True)
  def bot(**op):return c.call('botany',operation=op)
  def recipe():return next(r for r in bot(kind='inspect')['recipes'] if r['prefix']==PREFIX)
  def snap(name):c.call('capture',path=f'artifacts/botany-junction-quality/{name}.png',width=1400,height=1200)
  def camera(v):p,t,f=VIEWS[v];c.call('set_camera',camera=dict(position=p,target=t,fov_degrees=f))
  def geometry():return next(a['geometry'] for a in c.call('inspect')['scene']['meshes']['assets'] if a['id']==f'{PREFIX}_wood')
  original=recipe();material=c.call('material',operation=dict(kind='inspect',asset=f'{PREFIX}_wood'))
  # Recompile both sides with the current kernel; do not compare an old cached bake.
  source=copy.deepcopy(original['source']);bot(kind='publish',recipe=original);original=recipe();assert original['source']==source
  c.call('set_light',light=dict(direction=[.45,-.8,-.4],ambient=.3,direct_intensity=1.1,environment=dict(sky=[.5]*3,horizon=[.5]*3,ground=[.3]*3,axis=[0,1,0],intensity=.35)))
  g0=geometry()
  for view in VIEWS:camera(view);snap(view+'-before')
  edit=copy.deepcopy(original['edits']['trunk']);edit['junction_refinement']=2
  draft=copy.deepcopy(original);draft['version']=11;draft['edits']['trunk']=edit
  camera('junction');bot(kind='preview',recipe=draft);snap('preview');assert recipe()==original
  bot(kind='cancel_preview');snap('cancel');assert (OUT/'cancel.png').read_bytes()==(OUT/'junction-before.png').read_bytes()
  start=time.monotonic();result=bot(kind='edit_organ',prefix=PREFIX,organ='trunk',edit=edit);elapsed=time.monotonic()-start
  assert recipe()['source']==original['source']
  assert c.call('material',operation=dict(kind='inspect',asset=f'{PREFIX}_wood'))==material
  snap('published');assert (OUT/'published.png').read_bytes()==(OUT/'preview.png').read_bytes()
  c.call('undo');snap('undo');assert (OUT/'undo.png').read_bytes()==(OUT/'junction-before.png').read_bytes()
  c.call('redo');snap('redo');assert (OUT/'redo.png').read_bytes()==(OUT/'published.png').read_bytes()
  valid=recipe();bad=copy.deepcopy(edit);bad['junction_refinement']=4
  c.call('botany',expected_error=True,operation=dict(kind='edit_organ',prefix=PREFIX,organ='trunk',edit=bad));assert recipe()==valid
  for view in VIEWS:camera(view);snap(view+'-after')
  g1=geometry();edges=Counter();orientation=Counter();volume=0.
  for a,b,d in g1['triangles']:
   p,q,r=(g1['positions'][i] for i in (a,b,d));cross=[q[1]*r[2]-q[2]*r[1],q[2]*r[0]-q[0]*r[2],q[0]*r[1]-q[1]*r[0]];volume+=sum(x*y for x,y in zip(p,cross))/6
   for i,j in [(a,b),(b,d),(d,a)]:key=(min(i,j),max(i,j));edges[key]+=1;orientation[key]+=1 if i<j else -1
  assert volume>0 and all(n==2 and orientation[e]==0 for e,n in edges.items())
  camera('junction');c.call('save_scene',path='artifacts/botany-junction-quality/current.scene.json');snap('saved')
  c.call('load_scene',path='artifacts/botany-junction-quality/current.scene.json');snap('reopened');assert (OUT/'saved.png').read_bytes()==(OUT/'reopened.png').read_bytes()
  bot(kind='open',open=True,prefix=PREFIX);bot(kind='select_organ',prefix=PREFIX,organ='trunk');c.call('capture_editor',path='artifacts/botany-junction-quality/editor.png',width=1700,height=1200)
  report=dict(build=c.call('inspect')['build_id'],seconds_apply=elapsed,compiled=result['compiled'],before=dict(vertices=len(g0['positions']),triangles=len(g0['triangles'])),after=dict(vertices=len(g1['positions']),triangles=len(g1['triangles'])),closed_oriented=True,positive_volume=volume,source_graph_unchanged=True,material_preserved=True,preview_equals_publish=True,cancel_undo_redo_png_exact=True,save_reopen_png_exact=True,invalid_atomic=True,installed_in_game=False,artistic_acceptance=False,collar_placement_locked=False)
  (OUT/'verification.json').write_text(json.dumps(report,indent=2)+'\n')
  cards=''.join(f'<h2>{v}</h2><div><figure><img src="{v}-before.png"><figcaption>Before</figcaption></figure><figure><img src="{v}-after.png"><figcaption>Local junction refinement2</figcaption></figure></div>' for v in VIEWS)
  (OUT/'review.html').write_text('<!doctype html><meta charset="utf-8"><title>Local junction resolution</title><style>body{background:#20252b;color:#eee;font:16px system-ui;margin:24px}div{display:flex}figure{width:50%;margin:8px}img{width:100%}</style><h1>Ancestral structure: local junction resolution</h1><p>Same complete source and neutral light, no bark or leaves. This is a resolution control, not a solution for collar size, tangent continuity or field transport. Artistic acceptance pending.</p>'+cards)
  print(json.dumps(report))
 finally:c.close()
if __name__=='__main__':main()
