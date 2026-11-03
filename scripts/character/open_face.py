#!/usr/bin/env python3
"""Reopen the saved human scene and upgrade its latest authored values in place."""
import json,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'scripts'))
from verify_lab import Client

def main():
 out=ROOT/'artifacts/character-face';scene='artifacts/character-eyes/editor.scene.json';source='artifacts/character-base/human-soft.ignis-character'
 for receipt in [out/'launch.json',ROOT/'artifacts/character-eyes/launch.json']:
  if receipt.exists():
   pid=json.loads(receipt.read_text())['pid'];probe=subprocess.run(['ps','-p',str(pid),'-o','command='],capture_output=True,text=True)
   if probe.returncode==0 and '--scene '+scene in probe.stdout:
    if receipt==out/'launch.json':print(json.dumps({'pid':pid,'already_running':True}));return
    raise RuntimeError('Save and close the previous human editor before replacing its executable session')
 binary=Path.home()/'Applications/Ignis Editor.app/Contents/MacOS/ignis'
 if not binary.is_symlink() or binary.resolve()!=(ROOT/'target/release/ignis').resolve():binary=ROOT/'target/release/ignis'
 c=Client(binary,window=True,detached=True,log_path=out/'native.log',startup_args=('--editor','--scene',scene))
 try:
  c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'facial-editor','version':'10'}});c.request('notifications/initialized',notification=True)
  layout=out/'user-current.layout.json'
  if layout.exists():c.call('load_editor',path='artifacts/character-face/user-current.layout.json')
  c.call('open_character_panel',open=True,prefix='female');c.call('load_character_editor_source',prefix='female');before=c.call('inspect_character_editor');assert before['ready']
  if before['binding']['recipe']['source_path']!=source:c.call('upgrade_character_source',prefix='female',path=source,reference=before['reference'])
  state=c.call('inspect_character_editor');params=state['binding']['recipe']['configuration']['parameters'];assert state['ready'] and len(params)==393 and len(state['handles'])==95
  assert all(params[k]==v for k,v in before['binding']['recipe']['configuration']['parameters'].items())
  c.call('set_character_handle_view',view={'enabled':True,'symmetry':True,'selected':None,'group':None});c.call('set_viewport_performance',settings={'enabled':False,'vsync':True})
  c.call('save_scene',path=scene);c.call('save_character_configuration',prefix='female',path='artifacts/character-face/current.ignis-character-config');c.call('save_editor',path='artifacts/character-face/current.layout.json');c.call('capture_editor',path='artifacts/character-face/editor.png',width=1440,height=1000)
  report=dict(pid=c.process.pid,build=c.call('inspect')['build_id'],scene=scene,source=source,source_sha256=state['binding']['recipe']['source_sha256'],values=len(params),handles=len(state['handles']),retained_values=len(before['binding']['recipe']['configuration']['parameters']),bake=state['binding']['bake_sha256'])
  (out/'launch.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
 except BaseException:c.close();raise
 else:c.process.stdin.close();c.process.stdout.close();c.log.close()
if __name__=='__main__':main()
