"""Preview the sequoia bark recipe through the actual native material generator."""
import json
from verify_lab import ROOT,Client
OUT=ROOT/'artifacts/sequoia-realistic'
OUT.mkdir(parents=True,exist_ok=True)
c=Client(ROOT/'target/release/ignis',log_path=OUT/'material-preview.log',startup_args=('--scene','artifacts/living-plants/empty.scene.json'))
try:
 c.request('initialize',dict(protocolVersion='2025-11-25',capabilities={},clientInfo=dict(name='bark-preview',version='1')))
 c.request('notifications/initialized',notification=True)
 c.call('edit_mesh',edit=dict(kind='put_asset',asset=dict(id='bark_preview',geometry=dict(positions=[[-1.2,0,0],[1.2,0,0],[1.2,5,0],[-1.2,5,0]],triangles=[[0,1,2],[0,2,3]]),material={})))
 c.call('edit_mesh',edit=dict(kind='put_instance',instance=dict(id='bark_preview',asset='bark_preview',position=[0,0,0],rotation=[0,0,0,1],scale=1,color=[1,1,1])))
 recipe=json.loads((ROOT/'assets/materials/sequoia/bark.recipe.json').read_text())
 c.call('material',operation=dict(kind='generate',asset='bark_preview',recipe=recipe,execution='runtime'))
 c.call('set_light',light=dict(direction=[.65,-.18,-1],ambient=.12,direct_intensity=1.2,environment=dict(sky=[.55,.68,.85],horizon=[.6,.65,.66],ground=[.15,.1,.05],axis=[0,1,0],intensity=.35)))
 c.call('set_camera',camera=dict(position=[.3,2.5,7.5],target=[0,2.5,0],fov_degrees=40))
 c.call('capture',path='artifacts/sequoia-realistic/bark-generator-preview.png',width=1100,height=1400)
finally:c.close()
