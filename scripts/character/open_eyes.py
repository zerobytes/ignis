#!/usr/bin/env python3
"""Open the verified ocular scene in its own editor process, one current binary."""
import json,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'scripts'))
from verify_lab import Client

def main():
 out=ROOT/'artifacts/character-eyes';scene='artifacts/character-eyes/editor.scene.json'
 if any(b['recipe']['source_path'].endswith(('/human-face.ignis-character','/human-spherical.ignis-character')) for b in json.loads((ROOT/scene).read_text()).get('characters',[])):
  from open_face import main as open_current
  return open_current()
 binary=Path.home()/'Applications/Ignis Editor.app/Contents/MacOS/ignis'
 if not binary.is_symlink() or binary.resolve()!=(ROOT/'target/release/ignis').resolve():
  binary=ROOT/'target/release/ignis'
 launch=out/'launch.json'
 if launch.exists():
  previous=json.loads(launch.read_text());probe=subprocess.run(['ps','-p',str(previous['pid']),'-o','command='],capture_output=True,text=True)
  if probe.returncode==0 and '--scene '+scene in probe.stdout:
   print(json.dumps({'pid':previous['pid'],'already_running':True}));return
 c=Client(binary,window=True,detached=True,log_path=out/'native.log',startup_args=('--editor','--scene',scene))
 try:
  c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'ocular-editor','version':'7'}});c.request('notifications/initialized',notification=True)
  if (ROOT/'artifacts/character-nose/user-current.layout.json').exists():
   layout=json.loads((ROOT/'artifacts/character-nose/user-current.layout.json').read_text());layout['work_light']=None
   for ws in layout['workspaces']:
    if ws['id']==layout['active']:
     center=json.loads((out/'verification.json').read_text())['eye_center'];center=[0,center[1],center[2]]
     ws['camera']={'position':[0,center[1],center[2]-.18],'target':center,'fov_degrees':30};ws['layout']['ratio']=.68
     ws['layout']['second']['ratio']=.1
   (out/'current.layout.json').write_text(json.dumps(layout,indent=2));c.call('load_editor',path='artifacts/character-eyes/current.layout.json')
  c.call('open_character_panel',open=True,prefix='female');c.call('load_character_editor_source',prefix='female')
  c.call('set_character_handle_view',view={'enabled':False,'symmetry':True,'selected':None,'group':None})
  c.call('set_viewport_performance',settings={'enabled':False,'vsync':True})
  state=c.call('inspect_character_editor');assert state['ready'] and all(name in state['binding']['recipe']['configuration']['parameters'] for name in ['eye_L_red','eye_R_red'])
  c.call('save_editor',path='artifacts/character-eyes/current.layout.json');c.call('capture_editor',path='artifacts/character-eyes/editor.png',width=1440,height=1000)
  report={'pid':c.process.pid,'scene':scene,'build':c.call('inspect')['build_id'],'ready':state['ready'],'source':state['binding']['recipe']['source_path']}
  launch.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
 except BaseException:c.close();raise
 else:c.process.stdin.close();c.process.stdout.close();c.log.close()
if __name__=='__main__':main()
