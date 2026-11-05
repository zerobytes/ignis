"""Native geometry-only sequoia volume comparison; no texture or POM in gray views."""
import argparse,copy,json
from verify_lab import ROOT,Client
OUT=ROOT/'artifacts/sequoia-volume'
PREFIX='sequoia_volume'
def main():
 ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('--skip-age-checks',action='store_true');args=ap.parse_args()
 OUT.mkdir(parents=True,exist_ok=True)
 c=Client(ROOT/'target/release/ignis',log_path=OUT/'native.log',startup_args=('--scene','artifacts/sequoia-realistic/empty.scene.json'))
 try:
  c.request('initialize',dict(protocolVersion='2025-11-25',capabilities={},clientInfo=dict(name='sequoia-volume',version='1')));c.request('notifications/initialized',notification=True)
  definition=json.loads((ROOT/'examples/sequoia-volume.recipe.json').read_text());base=copy.deepcopy(definition);base['version']=6;base.pop('wood_volume')
  c.call('botany',operation=dict(kind='create_living',definition=base,born=0))
  c.call('botany',operation=dict(kind='living',prefix=PREFIX,action=dict(kind='advance',time=365000)))
  def recipe():return c.call('botany',operation=dict(kind='inspect'))['recipes'][0]
  def publish(r):
   path=OUT/'candidate.ignis-botany';path.write_text(json.dumps(r,separators=(',',':')))
   result=c.call('botany',operation=dict(kind='load',path='artifacts/sequoia-volume/candidate.ignis-botany',publish=True));path.unlink();return result
  r=recipe();r['settings'].update(leaf_density=0,bark_color=[.42,.42,.42]);before=publish(r)
  c.call('set_light',light=dict(direction=[.45,-.7,-.55],ambient=.22,direct_intensity=1.2,
   environment=dict(sky=[.55,.62,.72],horizon=[.5,.52,.54],ground=[.12,.13,.14],axis=[0,1,0],intensity=.3),
   atmosphere=dict(density=0,reduce_details=False,color=[.2,.24,.28],sun_color=[1,1,1],sun_strength=0,layers=dict(sky_color=[.12,.16,.2],mountain_color=[.12,.16,.2],detail_start=10000,detail_end=20000,influence=0)),
   shadow=dict(center=[0,35,0],extent=65,resolution=2048,bias=.0003,receiver_plane_bias=True)))
  views=dict(front=dict(position=[15,8,24],target=[0,9,0],fov_degrees=42),side=dict(position=[24,8,-15],target=[0,9,0],fov_degrees=42),back=dict(position=[-15,8,-24],target=[0,9,0],fov_degrees=42),whole=dict(position=[72,6,110],target=[0,42,0],fov_degrees=42))
  def capture(name):c.call('capture',path=f'artifacts/sequoia-volume/{name}.png',width=1500,height=1400)
  c.call('set_camera',camera=views['front']);c.call('save_scene',path='artifacts/sequoia-volume/before.scene.json');capture('front-before')
  source=recipe()['source']
  after=c.call('botany',operation=dict(kind='living',prefix=PREFIX,action=dict(kind='set_wood_volume',profile=definition['wood_volume'])))
  capture('front-after')
  gray_material=c.call('material',operation=dict(kind='inspect',asset=f'{PREFIX}_wood'));assert gray_material['execution']=='legacy'
  c.call('undo');capture('undo');assert (OUT/'undo.png').read_bytes()==(OUT/'front-before.png').read_bytes()
  c.call('redo');capture('redo');assert (OUT/'redo.png').read_bytes()==(OUT/'front-after.png').read_bytes()
  current=recipe();assert current['source']==source and current['living']['definition']['version']==7
  bad=copy.deepcopy(definition['wood_volume']);bad['valley_depth_ratio']=.9
  c.call('botany',expected_error=True,operation=dict(kind='living',prefix=PREFIX,action=dict(kind='set_wood_volume',profile=bad)));assert recipe()==current
  c.call('save_scene',path='artifacts/sequoia-volume/neutral.scene.json')
  for label,scene in [('before','before.scene.json'),('after','neutral.scene.json')]:
   c.call('load_scene',path=f'artifacts/sequoia-volume/{scene}')
   for view,camera in views.items():c.call('set_camera',camera=camera);capture(f'{view}-{label}')
  c.call('load_scene',path='artifacts/sequoia-volume/neutral.scene.json');c.call('set_camera',camera=views['front']);capture('roundtrip')
  assert (OUT/'roundtrip.png').read_bytes()==(OUT/'front-after.png').read_bytes()
  light=json.loads((OUT/'neutral.scene.json').read_text())['light'];unshadowed=copy.deepcopy(light);unshadowed.pop('shadow')
  c.call('set_light',light=unshadowed);capture('shadow-off');c.call('set_light',light=light)
  if not args.skip_age_checks:
   saved=recipe()
   for name,year in [('young-a',5),('adult',1000),('young-b',5),('birth',0)]:
    c.call('botany',operation=dict(kind='preview_living_age',prefix=PREFIX,time=year*365));capture(f'age-{name}');assert recipe()==saved
   assert (OUT/'age-young-a.png').read_bytes()==(OUT/'age-young-b.png').read_bytes()
   c.call('botany',operation=dict(kind='cancel_preview'));capture('cancel');assert (OUT/'cancel.png').read_bytes()==(OUT/'front-after.png').read_bytes()
  r=recipe();r['settings'].update(leaf_density=1,bark_color=[1,1,1]);publish(r)
  for part,name in [('wood','bark'),('leaves','foliage')]:c.call('material',operation=dict(kind='load',asset=f'{PREFIX}_{part}',path=f'assets/materials/sequoia/{name}.ignis-material'))
  c.call('set_camera',camera=views['whole']);capture('textured-whole')
  c.call('set_camera',camera=views['front']);capture('textured-detail')
  c.call('botany',operation=dict(kind='open',open=True,prefix=PREFIX));c.call('save_scene',path='artifacts/sequoia-volume/tree.scene.json')
  c.call('capture_editor',path='artifacts/sequoia-volume/editor.png',width=1700,height=1200)
  c.call('load_scene',path='artifacts/sequoia-volume/tree.scene.json');capture('textured-reopened');assert (OUT/'textured-reopened.png').read_bytes()==(OUT/'textured-detail.png').read_bytes()
  if not args.skip_age_checks:
   c.call('load_scene',path='artifacts/sequoia-realistic/empty.scene.json');c.call('botany',operation=dict(kind='create_living',definition=definition,born=0));c.call('botany',operation=dict(kind='save',prefix=PREFIX,path='assets/botany/sequoia-volume.ignis-botany'))
  result=dict(build=c.call('inspect')['build_id'],before=before['compiled'],after=after['compiled'],source_graph_unchanged=True,undo_redo_png_exact=True,save_reopen_png_exact=True,invalid_profile_atomic=True,age_checks=not args.skip_age_checks,gray_material=gray_material)
  (OUT/'verification.json').write_text(json.dumps(result,indent=2)+'\n')
  cards=''.join(f'<h2>{v}</h2><div><figure><img src="{v}-before.png"><figcaption>Antes</figcaption></figure><figure><img src="{v}-after.png"><figcaption>Depois</figcaption></figure></div>' for v in views)
  (OUT/'review.html').write_text('<!doctype html><meta charset="utf-8"><title>Volumes do tronco</title><style>body{background:#1b2025;color:#eee;font:16px system-ui;max-width:1500px;margin:32px auto}div{display:flex}figure{width:50%;margin:8px}img{width:100%}</style><h1>Sequoia · volumes reais no mesh</h1><p>Albedo cinza uniforme, sem textura/paralaxe. Mesma câmera/luz antes e depois. Cordões, vales e saliências são geometria.</p>'+cards+'<h2>Controle sem sombra projetada</h2><p>Artefatos estreitos nos vales desaparecem sem shadow map; refinamento do renderer ainda pendente.</p><img src="shadow-off.png"><h2>Materiais aplicados</h2><img src="textured-detail.png">')
  print(json.dumps(result))
 finally:c.close()
if __name__=='__main__':main()
