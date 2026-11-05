"""Native same-camera before/after for correlated sequoia bark depth."""
import json,copy
from verify_lab import ROOT,Client
OUT=ROOT/'artifacts/sequoia-relief'
def main():
 OUT.mkdir(parents=True,exist_ok=True)
 c=Client(ROOT/'target/release/ignis',log_path=OUT/'native.log',startup_args=('--scene','artifacts/sequoia-realistic/sequoia.scene.json'))
 try:
  c.request('initialize',dict(protocolVersion='2025-11-25',capabilities={},clientInfo=dict(name='sequoia-relief',version='1')));c.request('notifications/initialized',notification=True)
  original=c.call('botany',operation=dict(kind='inspect'))['recipes']
  before=json.loads((ROOT/'examples/sequoia-bark-shallow.recipe.json').read_text())
  after=json.loads((ROOT/'assets/materials/sequoia/bark.recipe.json').read_text())
  views=dict(front=dict(position=[11,5,15],target=[0,6,0],fov_degrees=44),grazing=dict(position=[15,5,4],target=[0,6,0],fov_degrees=38))
  for name,recipe in [('before',before),('after',after)]:
   c.call('material',operation=dict(kind='generate',asset='sequoia_real_wood',recipe=recipe,execution=dict(fixed=dict(resolution=512))))
   for view,camera in views.items():
    c.call('set_camera',camera=camera);c.call('capture',path=f'artifacts/sequoia-relief/{view}-{name}.png',width=1500,height=1400)
   neutral=copy.deepcopy(recipe)
   for sample in [neutral['base']]+[l['target'] for l in neutral['layers']]:
    sample['color_roughness']=[.28,.28,.28,.9];sample['response'][3]=1;sample['emission']=[0,0,0,0]
   c.call('material',operation=dict(kind='generate',asset='sequoia_real_wood',recipe=neutral,execution=dict(fixed=dict(resolution=512))))
   c.call('set_camera',camera=views['grazing'])
   c.call('capture',path=f'artifacts/sequoia-relief/neutral-{name}.png',width=1500,height=1400)
  assert (OUT/'neutral-before.png').read_bytes()!=(OUT/'neutral-after.png').read_bytes()
  c.call('material',operation=dict(kind='generate',asset='sequoia_real_wood',recipe=after,execution=dict(fixed=dict(resolution=512))))
  assert c.call('botany',operation=dict(kind='inspect'))['recipes']==original
  c.call('material',operation=dict(kind='save',asset='sequoia_real_wood',path='assets/materials/sequoia/bark.ignis-material'))
  appearance=c.call('material',operation=dict(kind='inspect',asset='sequoia_real_wood'))
  c.call('set_camera',camera=dict(position=[72,6,110],target=[0,42,0],fov_degrees=42))
  c.call('capture',path='artifacts/sequoia-relief/whole.png',width=2000,height=2000)
  c.call('save_scene',path='artifacts/sequoia-realistic/sequoia.scene.json')
  c.call('load_scene',path='artifacts/sequoia-realistic/sequoia.scene.json')
  c.call('capture',path='artifacts/sequoia-relief/reopened.png',width=2000,height=2000)
  assert (OUT/'whole.png').read_bytes()==(OUT/'reopened.png').read_bytes()
  assert c.call('material',operation=dict(kind='inspect',asset='sequoia_real_wood'))==appearance
  for years in [5,1000]:
   c.call('botany',operation=dict(kind='preview_living_age',prefix='sequoia_real',time=years*365))
   assert c.call('material',operation=dict(kind='inspect',asset='sequoia_real_wood'))==appearance
  c.call('botany',operation=dict(kind='cancel_preview'))
  c.call('set_camera',camera=views['front'])
  c.call('capture',path='artifacts/sequoia-relief/after-preview.png',width=1500,height=1400)
  assert (OUT/'after-preview.png').read_bytes()==(OUT/'front-after.png').read_bytes()
  result=dict(build=c.call('inspect')['build_id'],before_depth_m=before['height_m'],after_depth_m=after['height_m'],view_steps=after['view_steps'],shadow_steps=after['shadow_steps'],fixed_resolution=512,map_sample_bytes=appearance['sample_bytes'],neutral_albedo_comparison=True,plant_recipe_unchanged=True,save_reopen_png_exact=True,material_preserved_through_age_preview=True,preview_cancel_png_exact=True,geometric_displacement_added=False)
  (OUT/'verification.json').write_text(json.dumps(result,indent=2)+'\n')
  cards=''.join(f'<h2>{v}</h2><div><figure><img src="{v}-before.png"><figcaption>Antes</figcaption></figure><figure><img src="{v}-after.png"><figcaption>Depois</figcaption></figure></div>' for v in [*views,'neutral'])
  (OUT/'review.html').write_text('<!doctype html><meta charset="utf-8"><title>Profundidade da casca</title><style>body{background:#1b2025;color:#eee;font:16px system-ui;max-width:1500px;margin:32px auto}div{display:flex}figure{width:50%;margin:8px}img{width:100%}</style><h1>Sequoia · profundidade dos sulcos</h1><p>Mesma árvore, câmera e luz. Mapas512, altura material correlacionada, parallax e auto-sombra. A geometria permanece a anterior.</p>'+cards)
  print(json.dumps(result))
 finally:c.close()
if __name__=='__main__':main()
