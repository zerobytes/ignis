#!/usr/bin/env python3
"""Depth-guided edge finish, on/off equivalence and native moving-view comparisons."""
import copy
import json
from png_pixels import rgba8
from verify_lab import Client, ROOT

OUT=ROOT/'artifacts/atmosphere-edges'
PREFIX='artifacts/atmosphere-edges/'


def difference(a,b):
    x=rgba8(a)[2];y=rgba8(b)[2]
    values=[abs(v-w) for i,(v,w) in enumerate(zip(x,y)) if i%4!=3]
    return dict(max_rgb8=max(values),mae_rgb8=sum(values)/len(values),changed_channels=sum(v>0 for v in values))


def verify():
    OUT.mkdir(parents=True,exist_ok=True)
    authored=json.loads((ROOT/'artifacts/atmosphere-landscape/verification.json').read_text())
    c=Client(ROOT/'target/release/ignis');checks=[];cases=[]
    def capture(name,width=1728,height=1117):
        meta=c.call('capture',path=PREFIX+name+'.png',width=width,height=height)
        return (OUT/(name+'.png')).read_bytes(),meta['adapter']
    try:
        c.request('initialize',dict(protocolVersion='2025-11-25',capabilities={},clientInfo=dict(name='atmospheric-edges',version='1')))
        c.request('notifications/initialized',notification=True)
        assert 'edge_softness' in json.dumps(c.request('tools/list'))
        c.call('load_scene',path='artifacts/atmosphere-landscape/landscape.scene.json')
        for view in ['haze','sunset','move']:
            source=next(v for v in authored['cases'] if v['id']==view)
            c.call('set_camera',camera=source['camera'])
            light=copy.deepcopy(source['light']);light['atmosphere']['reduce_details']=True
            c.call('set_light',light=light)
            original,meta=capture(view+'-off')
            assert original==(ROOT/f'artifacts/atmosphere-landscape/{view}-selected.png').read_bytes(),'disabled changed previous pixels'
            assert meta['atmospheric_edges']['image_bytes']==0
            for mode,strength,radius in [('zero',0,12),('radius-zero',1,0),('soft',.5,6),('strong',1,12)]:
                light['atmosphere']['edge_softness']=dict(strength=strength,radius_px=radius,start=2000,end=10000)
                c.call('set_light',light=light)
                selected,meta=capture(view+'-'+mode+'-selected')
                if strength==0 or radius==0:
                    assert selected==original
                    assert meta['atmospheric_edges']['passes']==meta['atmospheric_edges']['image_bytes']==0
                    continue
                assert selected!=original
                light['atmosphere']['reduce_details']=False;c.call('set_light',light=light)
                full,full_meta=capture(view+'-'+mode+'-full')
                diff=difference(full,selected)
                assert diff['max_rgb8']<=3,(view,mode,diff)
                light['atmosphere']['reduce_details']=True
                cases.append(dict(id=view+'-'+mode,label=view+' '+mode,camera=source['camera'],light=copy.deepcopy(light),
                    full=full_meta['distant_selection'],selected=meta['distant_selection'],edges=meta['atmospheric_edges'],
                    against_off=difference(original,selected),**diff))
                # Preserve UI/scene/history contract, not a separate screenshot filter.
                if view=='sunset' and mode=='strong':
                    c.call('set_light',light=light)
                    c.call('save_scene',path=PREFIX+'sunset.scene.json')
                    c.call('load_scene',path=PREFIX+'sunset.scene.json')
                    assert capture('sunset-loaded')[0]==selected
                    c.call('step',ticks=2);c.call('save_replay',path=PREFIX+'sunset.replay.json')
                    c.call('capture_replay',path=PREFIX+'sunset-replay.png',replay_path=PREFIX+'sunset.replay.json',width=1728,height=1117)
                    assert (OUT/'sunset-replay.png').read_bytes()==selected
            light['atmosphere'].pop('edge_softness');light['atmosphere']['reduce_details']=True
            c.call('set_light',light=light)
            restored,meta=capture(view+'-restored')
            assert restored==original and meta['atmospheric_edges']['image_bytes']==0
            if view=='haze':
                cases.insert(0,dict(id='off',label='Nítido',camera=source['camera'],light=copy.deepcopy(light)))
        checks.append('3 camera/light views: absence, strength0, radius0 and return are byte-exact to the prior renderer')
        checks.append('6 soft/strong pairs match full geometry with the same finish; scene/replay pixels exact')
        # A close opaque square against a distant surface/sky: its silhouette and
        # interior must stay exact, and no foreground colour can bleed behind it.
        c.call('generate',recipe=dict(width=16,depth=14,seed=17,obstacles=0))
        entities=c.call('inspect')['scene']['entities']
        c.call('edit_scene',edits=[dict(op='upsert',entity=dict(e,visible=False)) for e in entities])
        for name,positions in [('near',[[-2,1,5],[2,1,5],[2,5,5],[-2,5,5]]),('far',[[-25,-15,-30],[25,-15,-30],[25,20,-30],[-25,20,-30]])]:
            c.call('edit_mesh',edit=dict(kind='put_asset',asset=dict(id=name,geometry=dict(positions=positions,triangles=[[0,1,2],[0,2,3]]))))
            c.call('edit_mesh',edit=dict(kind='put_instance',instance=dict(id=name,asset=name,position=[0,0,0],rotation=[0,0,0,1],scale=1,color=[1,0,0] if name=='near' else [.1,.35,.2],playback=None)))
        c.call('set_camera',camera=dict(position=[0,3,10],target=[0,3,0],fov_degrees=55))
        light=dict(direction=[0,-1,0],ambient=1,atmosphere=dict(density=0,color=[.4,.6,.8],sun_color=[1,.8,.4],sun_strength=0))
        c.call('set_light',light=light);plain,_=capture('foreground-off',800,600)
        light['atmosphere']['edge_softness']=dict(strength=1,radius_px=12,start=10,end=20)
        c.call('set_light',light=light);soft,_=capture('foreground-on',800,600)
        a=rgba8(plain)[2];b=rgba8(soft)[2];near=changed=0
        for x,y in zip([a[i:i+4] for i in range(0,len(a),4)],[b[i:i+4] for i in range(0,len(b),4)]):
            if x[0]>240 and x[1]<5:
                assert x==y,'foreground blurred';near+=1
            if x!=y:
                assert y[0]<y[1],'red foreground leaked into background';changed+=1
        assert near>10000 and changed>100
        checks.append('near foreground pixels exact; distant edge changes without foreground-colour leakage')
        # Reuse the current authored landscape for the existing cost harness.
        c.call('load_scene',path='artifacts/atmosphere-landscape/landscape.scene.json')
        c.call('save_scene',path=PREFIX+'landscape.scene.json')
        report=dict(build_id=c.call('inspect')['build_id'],passed=True,checks=checks,cases=cases,
            foreground_pixels=near,foreground_background_changed=changed,width=1728,height=1117)
        (OUT/'verification.json').write_text(json.dumps(report,indent=2)+'\n')
        text='# Suavização opcional das bordas\n\nRenders nativos, 1728×1117 pixels.\n\n'
        for name,label in [('off','Desligado'),('soft-selected','Suave'),('strong-selected','Forte')]:
            text+=f'## {label}\n\n![{label}]({OUT / ("sunset-"+name+".png")})\n\n'
        (OUT/'comparison.md').write_text(text)
        print(json.dumps(dict(passed=True,checks=checks,build_id=report['build_id'])),flush=True)
    finally:c.close()


if __name__=='__main__':verify()
