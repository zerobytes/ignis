#!/usr/bin/env python3
"""Same source/camera, native rest-curve variants and physical overlay."""
import copy,json
from verify_lab import Client,ROOT
from botany_exemplars import Builder

OUT=ROOT/'artifacts/botany-native'
def main():
    c=Client(ROOT/'target/release/ignis',log_path=OUT/'deformation-runtime.log');report={}
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'botany-rest-curves','version':'1'}});c.request('notifications/initialized',notification=True)
        report['build_id']=c.call('inspect')['build_id']
        b=Builder('Curva de crescimento · mesma fonte e descendentes');root=b.axis('branch',None,0,(1,.2,0),2.,.022,position=(-.9,.7,0),normal=(0,1,0),young=4e6)
        for k in range(10):
            for side in [-1,1]:b.leaf(f'leaf{k}_{side}',root,.08+.085*k,(.2,.3,side),.22,.07,normal=(0,1,0))
        base=b.recipe('curve');base['settings']['axis_samples']=32
        variants={
            'straight':{},
            'arch':{'base':[.95,0.],'tip':[-1.4,.15],'twist':.2},
            'sag':{'base':[.3,0.],'tip':[0.,.25],'sag':1.6,'twist':.5},
            'irregular':{'base':[.8,.4],'tip':[-.7,1.5],'twist':1.2,'irregularity':1.8,'frequency':.8},
        }
        for name,field in variants.items():
            c.call('load_scene',path='artifacts/botany-native/empty.scene.json')
            r=copy.deepcopy(base);r['edits']={root:{'deformation':field}}
            path=f'artifacts/botany-native/curve-{name}.ignis-botany';(ROOT/path).write_text(json.dumps(r,separators=(',',':')))
            result=c.call('botany',operation={'kind':'load','path':path,'publish':True})
            c.call('set_camera',camera={'position':[2.5,2.5,4.5],'target':[.1,1.,0.],'fov_degrees':40})
            c.call('capture',path=f'artifacts/botany-native/curve-{name}.png',width=1100,height=900)
            report[name]=result
            if name=='irregular':
                before=c.call('botany',operation={'kind':'physics','prefix':'curve','action':{'kind':'start','settings':{'gravity':[0,0,0],'ground':None,'wind':[0,0,2],'iterations':32}}})
                after=c.call('botany',operation={'kind':'physics','prefix':'curve','action':{'kind':'step','ticks':120}})
                assert before['node_identity']==after['node_identity'] and after['attachment_error_m']<1e-4
                c.call('capture',path='artifacts/botany-native/curve-wind.png',width=1100,height=900)
                report['wind']={'before':before,'after':after}
        (OUT/'deformation-report.json').write_text(json.dumps(report,indent=2))
    finally:c.close()
if __name__=='__main__':main()
