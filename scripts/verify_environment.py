#!/usr/bin/env python3
"""Probe numerical controls, lazy cache diagnostics and native GPU/MCP histories."""
from verify_lab import SCENE_VERSION, REPLAY_VERSION
import argparse
import copy
import json
import math
from pathlib import Path
from verify_lab import Client, ROOT
from verify_materials import f32
from verify_metallic import encoded, linear
from png_pixels import rgba8


def probe(sky=(.65,.8,1),horizon=(.8,.7,.5),ground=(.08,.06,.03),axis=(0,1,0),intensity=.7):
    return f32(dict(sky=sky,horizon=horizon,ground=ground,axis=axis,intensity=intensity))


def ball():
    positions=[[0,1,0]];triangles=[];rings=15;sides=32
    for row in range(1,rings+1):
        theta=math.pi*row/(rings+1)
        positions.extend([[math.sin(theta)*math.cos(2*math.pi*j/sides),math.cos(theta),math.sin(theta)*math.sin(2*math.pi*j/sides)] for j in range(sides)])
    bottom=len(positions);positions.append([0,-1,0])
    for j in range(sides):
        triangles.extend([[0,1+j,1+(j+1)%sides],[bottom,1+(rings-1)*sides+j,1+(rings-1)*sides+(j+1)%sides]])
    for row in range(rings-1):
        for j in range(sides):
            a=1+row*sides+j;b=1+row*sides+(j+1)%sides;c=a+sides;d=b+sides
            triangles.extend([[a,b,c],[b,d,c]])
    for t in triangles:
        a,b,c=[positions[i] for i in t];u=[b[k]-a[k] for k in range(3)];v=[c[k]-a[k] for k in range(3)];n=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]]
        if sum(n[k]*(a[k]+b[k]+c[k]) for k in range(3))<0:t[1],t[2]=t[2],t[1]
    return f32(dict(id='orb',geometry=dict(positions=positions,triangles=triangles),shading='smooth',material=dict(reflectance=dict(roughness=.4,f0=.04,metallic=1))))


def verify(binary,window):
    c=Client(binary,window);checks=[]
    def edit(kind,**kw):return c.call('edit_mesh',edit=dict(kind=kind,**kw))
    def mat(m=1,r=1):edit('set_material',asset='surface',material=f32(dict(reflectance=dict(roughness=r,f0=.04,metallic=m))))
    def capture(name,width=800,height=600):
        p=f'artifacts/environment-{name}.png';meta=c.call('capture',path=p,width=width,height=height);return (ROOT/p).read_bytes(),meta['adapter']['environment_preparation']
    def center(image):
        w,h,p=rgba8(image);return list(p[(h//2*w+w//2)*4:][:3])
    def light(env=None,direct=0,ambient=0,**kw):
        result=dict(direction=[0,-1,0],ambient=ambient,direct_intensity=direct,**kw)
        if env is not None:result['environment']=env
        c.call('set_light',light=result);return result
    try:
        c.request('initialize',dict(protocolVersion='2025-11-25',capabilities={},clientInfo=dict(name='environment-verifier',version='1')));c.request('notifications/initialized',notification=True)
        schema=json.dumps(c.request('tools/list'));assert 'environment' in schema and 'direct_intensity' in schema
        c.call('generate',recipe=dict(width=16,depth=14,seed=17,obstacles=0));entities=c.call('inspect')['scene']['entities'];c.call('edit_scene',edits=[dict(op='upsert',entity=dict(e,visible=False)) for e in entities])
        edit('grid',id='unused',segments=1,size=1);edit('grid',id='surface',segments=3,size=3);edit('set_shading',asset='surface',shading='smooth')
        instance=dict(id='sheet',asset='surface',position=[0,1.5,0],rotation=[0,0,0,1],scale=1,color=[1,1,1],playback=None);edit('put_instance',instance=instance)
        camera=dict(position=[0,7,.06],target=[0,1.5,0],fov_degrees=36);c.call('set_camera',camera=camera);mat()
        light(None,direct=1,ambient=.1);legacy,stats=capture('legacy');assert stats==dict(probe_bakes=0,lut_bakes=0,logical_texture_bytes=148984)
        light(probe(intensity=0),direct=1,ambient=.1);assert capture('zero')[0]==legacy and capture('zero-repeat')[1]==stats
        constant=probe((1,1,1),(1,1,1),(1,1,1),intensity=1);light(constant);white,stats=capture('white-rough')
        target=encoded(1-math.log(2));actual=center(white);assert max(abs(v-target) for v in actual)<=2,(actual,target)
        assert stats['probe_bakes']==stats['lut_bakes']==1
        assert capture('repeat')==(white,stats)
        light(dict(constant,intensity=.5));half,half_stats=capture('intensity');assert half_stats==stats
        assert max(abs(encoded(linear(a)*.5)-b) for a,b in zip(actual,center(half)))<=1
        light(constant);mat(r=.2);smooth,s=capture('white-smooth');assert min(center(smooth))>=250 and center(smooth)[0]>actual[0] and s==stats
        mat();edit('set_material',asset='surface',material={});diffuse,s=capture('white-diffuse');assert center(diffuse)==[255]*3 and s==stats
        edit('put_instance',instance=dict(instance,color=[.2,.08,.025]));colored=capture('diffuse-color')[0];assert max(abs(a-encoded(b)) for a,b in zip(center(colored),[.2,.08,.025]))<=1
        edit('put_instance',instance=instance);mat()
        chromatic=probe((.2,.4,.6),(.2,.4,.6),(.2,.4,.6),intensity=1);light(chromatic);tinted,s=capture('chromatic');assert s['probe_bakes']==2 and s['lut_bakes']==1
        assert max(abs(a-encoded(b*(1-math.log(2)))) for a,b in zip(center(tinted),[.2,.4,.6]))<=2
        light(dict(chromatic,axis=[0,.5,0]));assert capture('normalized-axis')==(tinted,s)
        checks.append('Schema/zero legacy exact/lazy resources; constant metal matches1-ln2 within2LSB, diffuse radiance/color analytic, intensity scales without bake and normalized cache key stable')
        env=probe();baseline_light=light(env);a,stats=capture('gradient');assert a!=tinted and stats['probe_bakes']==3
        edit('set_material',asset='surface',material={});opaque,source_stats=capture('opaque-environment')
        edit('set_material',asset='surface',material={'transmission':.5});transmitted,transmission_stats=capture('transmitted-environment');assert transmitted!=opaque and transmission_stats==source_stats
        mat();assert capture('metal-after-transmission')[0]==a
        before=c.call('inspect');new_light=dict(baseline_light,environment=dict(env,axis=[1,0,0]));edits=[dict(op='set_light',light=new_light)]
        predicted=c.call('preview_scene_edits',edits=edits)['scene'];assert c.call('inspect')==before
        c.call('edit_scene',edits=edits);assert c.call('inspect')['scene']==predicted;b,s=capture('axis-rotated');assert b!=a and s['probe_bakes']==4
        c.call('undo');assert c.call('inspect')['scene']==before['scene'];restored,s=capture('undo');assert restored==a and s['probe_bakes']==5
        c.call('redo');assert capture('redo')[0]==b;c.call('undo');assert capture('axis-restored')[0]==a
        before=c.call('inspect')
        for bad in [dict(baseline_light,direct_intensity=-.1),dict(baseline_light,direct_intensity=4.1),dict(baseline_light,environment=dict(env,axis=[0,0,0])),dict(baseline_light,environment=dict(env,sky=[-1,0,0])),dict(baseline_light,environment=dict(env,intensity=4.1))]:
            for op in ['preview_scene_edits','edit_scene']:
                c.call(op,edits=[dict(op='set_light',light=bad)],expected_error=True);assert c.call('inspect')==before
        edit('remove_asset',id='unused');assert capture('slot-reindexed')[0]==a
        light(None);black,s=capture('disabled');assert center(black)==[0,0,0]
        light(env);assert capture('reenabled')[0]==a and capture('stable-cache')[1]==s
        c.call('save_scene',path='examples/environment.scene.json');saved=c.call('inspect')['scene'];assert saved['version']==SCENE_VERSION
        for case in ['environment','direct']:
            bad=copy.deepcopy(saved);bad['version']=75
            if case=='direct':bad['light'].pop('environment',None)
            p='artifacts/environment-old.scene.json';(ROOT/p).write_text(json.dumps(bad));state=c.call('inspect');error=c.call('load_scene',path=p,expected_error=True);assert 'version76' in json.dumps(error) and c.call('inspect')==state
        checks.append('Gradient/axis distinct, pure preview/publication/history/atomic bounds/gate75, slot reindex; None/A/B/A and reenable exact, one-entry content cache/LUT reused')
        # Curved native asset makes directional environment/reflection legible.
        edit('put_asset',asset=ball());edit('remove_instance',id='sheet');orb=dict(instance,id='orb-instance',asset='orb',color=[.7,.45,.15]);edit('put_instance',instance=orb)
        curved_camera=dict(position=[3,2.7,4],target=[0,1.5,0],fov_degrees=36);c.call('set_camera',camera=curved_camera);orb_a,stats=capture('orb');assert orb_a!=a
        edit('set_material',asset='orb',material=f32(dict(reflectance=dict(roughness=.9,f0=.04,metallic=1))));assert capture('orb-rough')[0]!=orb_a and capture('orb-rough-repeat')[1]==stats
        edit('set_material',asset='orb',material=f32(dict(reflectance=dict(roughness=.4,f0=.04,metallic=1))));assert capture('orb-restored')[0]==orb_a
        c.call('set_camera',camera=dict(curved_camera,position=[-3,2.7,4]));assert capture('orb-view')[0]!=orb_a and capture('orb-view-repeat')[1]==stats
        c.call('set_camera',camera=curved_camera);assert capture('orb-camera-restored')[0]==orb_a
        checks.append('Curved closed sphere, colored metal/roughness/view distinguish response; view/material changes never rebake the probe')
        edit('remove_instance',id='orb-instance');edit('put_instance',instance=instance);c.call('set_camera',camera=camera);assert capture('sheet-restored')[0]==a
        # Directional depth does not substitute visibility of all environment directions.
        angled=dict(position=[-2.5,6.5,-1.25],target=[0,1.5,0],fov_degrees=36);c.call('set_camera',camera=angled);base=capture('shadow-control')[0]
        box=copy.deepcopy(next(e for e in entities if e['id']=='wall_north'));box.update(id='cover',visible=True,color=[.3]*3);box['transform']=dict(position=[3,7.5,1.5],size=[4.5,.3,4.5]);c.call('edit_scene',edits=[dict(op='upsert',entity=box)])
        shadow_light=dict(baseline_light,direction=[-.5,-1,-.25],shadow=dict(center=[1,4,.5],extent=8,resolution=2048,bias=.0001));c.call('set_light',light=shadow_light);shadow,s=capture('shadow');assert center(shadow)==center(base)
        c.call('remove',id='cover');c.call('set_light',light=baseline_light);assert capture('uncovered')[0]==base
        geometry=c.call('inspect_mesh',asset='surface')['asset']['geometry'];n=len(geometry['positions'])
        edit('set_rig',asset='surface',rig=dict(bones=[dict(name='root',parent=None,translation=[0,0,0],rotation=[0,0,0,1])],weights=[dict(joints=[0]*4,weights=[1,0,0,0]) for _ in range(n)]))
        edit('put_clip',asset='surface',clip=dict(id='move',duration_ticks=120,tracks=[dict(bone=0,keys=[dict(tick=t,translation=[0,y,0],rotation=[0,0,0,1]) for t,y in [(0,0),(60,.5),(120,0)]])]))
        edit('put_instance',instance=dict(instance,position=[0,1,0],playback=dict(clip='move',offset_ticks=60,looping=True)));assert capture('compensated-rig')[0]==base
        edit('put_instance',instance=dict(instance,playback=dict(clip='move',offset_ticks=0,looping=True)));assert capture('restored-rig')[0]==base
        c.call('save_scene',path='examples/environment.scene.json');saved=c.call('inspect')['scene'];c.call('step',ticks=60);live,s=capture('live');assert live!=base and s==stats
        c.call('save_replay',path='artifacts/environment.replay.json');assert json.loads((ROOT/'artifacts/environment.replay.json').read_text())['version']==REPLAY_VERSION
        assert c.call('run_replay',path='artifacts/environment.replay.json')['meshes_matched']
        for name,tick,expected in [('replay',None,live),('prefix',0,base)]:
            p=f'artifacts/environment-{name}.png';c.call('capture_replay',replay_path='artifacts/environment.replay.json',path=p,tick=tick,width=800,height=600);assert (ROOT/p).read_bytes()==expected
        c.call('load_scene',path='examples/environment.scene.json');assert c.call('inspect')['scene']==saved and capture('loaded')[0]==base
        c.call('reset');assert capture('reset')[0]==base
        capture('viewport',640,480);assert capture('viewport-restored')[0]==base
        checks.append('Directional shadow leaves global environment intact; compensated/animated rig, scene76 load/reset, Replay80/prefix and viewport/cache restoration exact')
        report=dict(passed=True,checks=checks,build_id=c.call('inspect')['build_id'],white_rough_actual=actual,white_rough_target=target,preparation=s,native_window=window)
        (ROOT/'artifacts/environment-verification.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(dict(passed=True,checks=len(checks),build_id=report['build_id'])))
    finally:c.close()


if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');p.add_argument('--window',action='store_true');a=p.parse_args();verify(a.binary.resolve(),a.window)
