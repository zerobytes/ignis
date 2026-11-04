#!/usr/bin/env python3
"""Open the neutral woody-tool specimen in a separate native EDITOR window."""
import json
from verify_lab import Client,ROOT
out=ROOT/'artifacts/wood-authoring'
scene=out/'wood.scene.json'
if not scene.is_file():raise SystemExit('Run python3 scripts/demo_wood_authoring.py first')
c=Client(ROOT/'target/release/ignis',window=True,detached=True,log_path=out/'window.log',startup_args=('--editor','--scene',str(scene.relative_to(ROOT))))
try:
 c.request('initialize',dict(protocolVersion='2025-11-25',capabilities={},clientInfo=dict(name='wood-authoring-editor',version='1')));c.request('notifications/initialized',notification=True)
 c.call('botany',operation=dict(kind='open',open=True,prefix='woodlab'))
 c.call('botany',operation=dict(kind='select_organ',prefix='woodlab',organ='trunk'))
 result=dict(pid=c.process.pid,scene=str(scene),build=c.call('inspect')['build_id'],mode='editor')
 (out/'window.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
except BaseException:c.close();raise
else:c.process.stdin.close();c.process.stdout.close();c.log.close()
