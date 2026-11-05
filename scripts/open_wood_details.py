#!/usr/bin/env python3
"""Open the wood morphology/history specimen in a separate native EDITOR window."""
import json
from verify_lab import Client,ROOT
out=ROOT/'artifacts/wood-details'
scene=out/'history.scene.json'
if not scene.is_file():raise SystemExit('Run python3 scripts/demo_wood_details.py first')
c=Client(ROOT/'target/release/ignis',window=True,detached=True,log_path=out/'window.log',startup_args=('--editor','--scene',str(scene.relative_to(ROOT))))
try:
 c.request('initialize',dict(protocolVersion='2025-11-25',capabilities={},clientInfo=dict(name='wood-history-editor',version='1')));c.request('notifications/initialized',notification=True)
 c.call('botany',operation=dict(kind='open',open=True,prefix='wood_history'))
 c.call('botany',operation=dict(kind='select_organ',prefix='wood_history',organ='stem'))
 result=dict(pid=c.process.pid,scene=str(scene),build=c.call('inspect')['build_id'],mode='editor')
 (out/'window.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
except BaseException:c.close();raise
else:c.process.stdin.close();c.process.stdout.close();c.log.close()
