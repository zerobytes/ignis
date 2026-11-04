#!/usr/bin/env python3
"""Native isolated-leaf/cluster comparison and progressive-curvature evidence."""
import copy,json,math,subprocess
from pathlib import Path
from verify_lab import Client,ROOT
import botany_math as b
from botany_exemplars import palm

OUT=ROOT/'artifacts/botany-native'

def main():
    current=json.loads((ROOT/'assets/botany/palmeirinha.ignis-botany').read_text())
    assert current==json.loads(json.dumps(palm())), 'Canonical source differs from its seeded generator'
    curves=[]
    for organ in current['source']['organs']:
        if '/raquis' not in organ['id'] or '/foliolo' in organ['id']:continue
        points=organ['path'];t=[b.unit(b.sub(c,a)) for a,c in zip(points,points[1:])]
        angles=[math.acos(max(-1,min(1,b.dot(a,c)))) for a,c in zip(t,t[1:])]
        assert all(c>a for a,c in zip(angles,angles[1:])),organ['id']
        assert angles[-1]>10*angles[0],organ['id']
        curves.append({'id':organ['id'],'first_turn_degrees':math.degrees(angles[0]),'last_turn_degrees':math.degrees(angles[-1])})
    report={'curves':curves,'seeded_regeneration_equal':True}
    baseline=json.loads(subprocess.check_output(['git','show','5c774be:assets/botany/palmeirinha.ignis-botany'],cwd=ROOT))
    path=OUT/'palmeirinha-review.ignis-botany'
    c=Client(ROOT/'target/release/ignis',log_path=OUT/'palm-review.log')
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'palm-frond-review','version':'1'}})
        c.request('notifications/initialized',notification=True)
        report['build_id']=c.call('inspect')['build_id']
        camera={'position':[6,4.1,10],'target':[0,2.4,0],'fov_degrees':35}
        for label,recipe in [('before',baseline),('after',current)]:
            path.write_text(json.dumps(recipe))
            c.call('load_scene',path='artifacts/botany-native/empty.scene.json')
            report[label]=c.call('botany',operation={'kind':'load','path':str(path.relative_to(ROOT)),'publish':True})
            c.call('set_camera',camera=camera)
            c.call('capture',path=f'artifacts/botany-native/palmeirinha-{label}.png',width=1200,height=1100)
        c.call('save_scene',path='artifacts/botany-native/palmeirinha-review.scene.json')
        c.call('load_scene',path='artifacts/botany-native/palmeirinha-review.scene.json')
        c.call('capture',path='artifacts/botany-native/palmeirinha-roundtrip.png',width=1200,height=1100)
        assert (OUT/'palmeirinha-roundtrip.png').read_bytes()==(OUT/'palmeirinha-after.png').read_bytes()
        report['scene_roundtrip_pixels_equal']=True
        # One complete leaf, including its own bare basal region; no cane.
        leaf=copy.deepcopy(current);leaf['settings'].pop('age_years',None);leaf['source'].pop('development',None);leaf['version']=leaf['source']['version']=2
        for o in leaf['source']['organs']:o.pop('life',None)
        key='caule0/raquis3'
        leaf['source']['organs']=[o for o in leaf['source']['organs'] if o['id']==key or o['id'].startswith(key+'/')]
        leaf['source']['organs'][0].update(parent=None,insertion=0)
        path.write_text(json.dumps(leaf))
        c.call('load_scene',path='artifacts/botany-native/empty.scene.json')
        result=c.call('botany',operation={'kind':'load','path':str(path.relative_to(ROOT)),'publish':True})
        axis=leaf['source']['organs'][0];a,z=axis['path'][0],axis['path'][-1]
        horizontal=b.unit((z[0]-a[0],0,z[2]-a[2]));side=b.cross(horizontal,b.UP)
        lo,hi=result['compiled']['bounds_min'],result['compiled']['bounds_max'];center=b.mul(b.add(lo,hi),.5)
        distance=max(b.norm(b.sub(hi,lo))*1.6,4.)
        c.call('set_camera',camera={'position':b.add(center,b.mul(side,distance)),'target':center,'fov_degrees':35})
        c.call('capture',path='artifacts/botany-native/palmeirinha-whole-leaf.png',width=1200,height=1100)
        leaf['settings']['leaf_density']=0;path.write_text(json.dumps(leaf))
        c.call('botany',operation={'kind':'load','path':str(path.relative_to(ROOT)),'publish':True})
        c.call('capture',path='artifacts/botany-native/palmeirinha-whole-leaf-axis.png',width=1200,height=1100)
        report['camera']=camera
        (OUT/'palm-frond-verification.json').write_text(json.dumps(report,indent=2))
        print(json.dumps({'curves':len(curves),'first':curves[0],'roundtrip':True}))
    finally:c.close()
if __name__=='__main__':main()
