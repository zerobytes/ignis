#!/usr/bin/env python3
"""Compare the native whole-frond growth control with a fixed camera."""
import copy,json
from verify_lab import Client,ROOT

OUT=ROOT/'artifacts/botany-native'
def main():
    source=json.loads((ROOT/'assets/botany/palmeirinha.ignis-botany').read_text())
    out=OUT/'palmeirinha-growth.ignis-botany'
    c=Client(ROOT/'target/release/ignis',log_path=OUT/'palm-growth.log');report={}
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'palm-growth-review','version':'1'}})
        c.request('notifications/initialized',notification=True)
        report['build_id']=c.call('inspect')['build_id']
        camera={'position':[6,4.1,10],'target':[0,2.4,0],'fov_degrees':35}
        for value in [0.,.45,.8,1.]:
            recipe=copy.deepcopy(source);recipe['settings']['frond_growth_variation']=value
            out.write_text(json.dumps(recipe))
            c.call('load_scene',path='artifacts/botany-native/empty.scene.json')
            report[str(value)]=c.call('botany',operation={'kind':'load','path':str(out.relative_to(ROOT)),'publish':True})
            c.call('set_camera',camera=camera)
            c.call('capture',path=f'artifacts/botany-native/palmeirinha-growth-{value:g}.png',width=1200,height=1100)
            if value==.8:
                c.call('save_scene',path='artifacts/botany-native/palmeirinha-growth.scene.json')
                c.call('load_scene',path='artifacts/botany-native/palmeirinha-growth.scene.json')
                c.call('capture',path='artifacts/botany-native/palmeirinha-growth-roundtrip.png',width=1200,height=1100)
                assert (OUT/'palmeirinha-growth-roundtrip.png').read_bytes()==(OUT/'palmeirinha-growth-0.8.png').read_bytes()
                before=c.call('botany',operation={'kind':'physics','prefix':'palmeirinha','action':{'kind':'start','settings':{'gravity':[0,0,0],'ground':None,'iterations':8}}})
                after=c.call('botany',operation={'kind':'physics','prefix':'palmeirinha','action':{'kind':'step','ticks':2}})
                assert before['node_identity']==after['node_identity']
                assert after['attachment_error_m']<1e-4
                report['physics']={'before':before,'after':after}
        report['roundtrip_pixels_equal']=True
        report['camera']=camera
        (OUT/'palm-growth-verification.json').write_text(json.dumps(report,indent=2))
        print(json.dumps({'build_id':report['build_id'],'roundtrip':True,'attachment_error':report['physics']['after']['attachment_error_m']}))
    finally:c.close()
if __name__=='__main__':main()
