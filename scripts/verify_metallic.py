#!/usr/bin/env python3
"""Direct metal BRDF, legacy neutrality, histories and GPU/replay through MCP."""
from verify_lab import SCENE_VERSION, REPLAY_VERSION
import argparse
import copy
import json
import math
from pathlib import Path
from verify_lab import Client, ROOT
from verify_materials import f32
from verify_layered_patterns import pattern
from png_pixels import rgba8


def linear(v):
    v=v/255
    return v/12.92 if v<=.04045 else ((v+.055)/1.055)**2.4


def encoded(v):
    return round(255*(12.92*v if v<=.0031308 else 1.055*v**(1/2.4)-.055))


def verify(binary, window):
    c=Client(binary,window); checks=[]
    def edit(kind,**kw):return c.call('edit_mesh',edit=dict(kind=kind,**kw))
    def response(m=1,r=.7,f0=.04):return f32(dict(reflectance=dict(roughness=r,f0=f0,metallic=m)))
    def material(m):edit('set_material',asset='surface',material=m)
    def capture(name,width=800,height=600):
        p=f'artifacts/metallic-{name}.png';c.call('capture',path=p,width=width,height=height);return (ROOT/p).read_bytes()
    def center(image):
        w,h,p=rgba8(image);return list(p[(h//2*w+w//2)*4:][:3])
    try:
        c.request('initialize',dict(protocolVersion='2025-11-25',capabilities={},clientInfo=dict(name='metallic-verifier',version='1')));c.request('notifications/initialized',notification=True)
        assert 'metallic' in json.dumps(c.request('tools/list'))
        c.call('generate',recipe=dict(width=16,depth=14,seed=17,obstacles=0))
        entities=c.call('inspect')['scene']['entities']
        c.call('edit_scene',edits=[dict(op='upsert',entity=dict(e,visible=False)) for e in entities])
        edit('grid',id='unused',segments=1,size=1);edit('grid',id='surface',segments=4,size=3);edit('set_shading',asset='surface',shading='smooth')
        instance=dict(id='sheet',asset='surface',position=[0,1.5,0],rotation=[0,0,0,1],scale=1,color=[.2,.08,.025],playback=None)
        edit('put_instance',instance=instance)
        normal_camera=dict(position=[0,7,.06],target=[0,1.5,0],fov_degrees=36)
        light=dict(direction=[0,-1,0],ambient=.1)
        c.call('set_camera',camera=normal_camera);c.call('set_light',light=light)
        legacy=dict(reflectance=dict(roughness=.7,f0=.04));material(legacy);dielectric=capture('dielectric')
        material(response(0));assert capture('explicit-zero')==dielectric
        material(response(1));metal=capture('metal');assert metal!=dielectric and capture('repeat')==metal
        # At normal incidence with GGX/Smith, pi*D*V*NoL =1/(4*roughness^4).
        # Full metal has zero diffuse, Schlick F=base and the explicit legacy ambient.
        expected=[encoded(v*(.1+.9/(4*.7**4))) for v in instance['color']]
        actual=center(metal);assert max(abs(a-b) for a,b in zip(actual,expected))<=1,(actual,expected)
        material(response(.5));half=capture('half');_,_,pa=rgba8(dielectric);_,_,pb=rgba8(metal);_,_,pm=rgba8(half)
        mix_error=max(abs(encoded((linear(a)+linear(b))*.5)-m) for i,(a,b,m) in enumerate(zip(pa,pb,pm)) if i%4!=3)
        assert mix_error<=2,mix_error
        material(response(1,f0=0));assert capture('metal-zero-f0')==metal
        material(response(1,f0=.16));assert capture('metal-high-f0')==metal
        material(response(1,r=.9));assert capture('rough')!=metal
        edit('put_instance',instance=dict(instance,color=[.025,.08,.2]));material(response());blue=capture('blue');assert center(blue)[2]>center(blue)[0]
        edit('put_instance',instance=instance);assert capture('color-restored')==metal
        checks.append('Schema/zero legacy exact; near-normal metal matches analytic GGX within1LSB, BRDF blend within2LSB; full metal ignores dielectric F0, roughness and reflected color distinct')
        before=c.call('inspect');old=c.call('inspect_mesh',asset='surface')['asset'];pose=c.call('inspect_mesh_pose',instance='sheet')['pose']
        for m in [-.01,1.01]:
            for op in ['preview_mesh_edit','edit_mesh']:
                c.call(op,edit=dict(kind='set_material',asset='surface',material=response(m)),expected_error=True);assert c.call('inspect')==before
        bad=dict(response(),transmission=.1);c.call('edit_mesh',edit=dict(kind='set_material',asset='surface',material=bad),expected_error=True);assert c.call('inspect')==before
        material(legacy);before=c.call('inspect');preview=c.call('preview_mesh_edit',edit=dict(kind='set_material',asset='surface',material=response()));assert c.call('inspect')==before
        material(response());assert c.call('inspect_mesh',asset='surface')['asset']==preview['meshes']['assets'][1]
        assert c.call('inspect_mesh',asset='surface')['asset']['geometry']==old['geometry'] and c.call('inspect_mesh_pose',instance='sheet')['pose']==pose
        c.call('undo');assert capture('undo')==dielectric
        c.call('redo');assert capture('redo')==metal
        edit('remove_asset',id='unused');assert capture('slot-reindexed')==metal
        checks.append('Bounds/transmission reject atomically, preview pure/equal, geometry/pose stable, material toggle/undo/redo and slot reindex cache exact')
        c.call('save_scene',path='examples/metallic.scene.json');saved=c.call('inspect')['scene'];assert saved['version']==SCENE_VERSION
        bad=copy.deepcopy(saved);bad['version']=73
        (ROOT/'artifacts/metallic-old.scene.json').write_text(json.dumps(bad));before=c.call('inspect');error=c.call('load_scene',path='artifacts/metallic-old.scene.json',expected_error=True);assert 'version74' in json.dumps(error) and c.call('inspect')==before
        tree=json.loads((ROOT/'examples/native-tree-study.scene.json').read_text());tree['version']=73
        def sources(s):
            out=[]
            for e in s['plants']['entries']:
                out.append(e['source'])
                if e.get('assembly'):
                    out.append(e['assembly']['context']);out.extend(g['source'] for g in e['assembly'].get('context_grafts',[]))
            return out
        assert len(sources(tree))==3
        for i in range(3):
            bad=copy.deepcopy(tree);sources(bad)[i]['material']['reflectance']=response()['reflectance'];(ROOT/'artifacts/metallic-hidden.scene.json').write_text(json.dumps(bad))
            error=c.call('load_scene',path='artifacts/metallic-hidden.scene.json',expected_error=True)
            assert 'Metallic response requires scene version74' in json.dumps(error) and c.call('inspect')==before
        checks.append('Scene74 gate in visible root and3 hidden source/context/graft materials before recipe evaluation')
        camera=dict(position=[-2.5,6.5,-1.25],target=[0,1.5,0],fov_degrees=36)
        c.call('set_camera',camera=camera);c.call('set_light',light=dict(direction=[-.5,-1,-.25],ambient=.2));angled=capture('angled');assert angled!=metal
        c.call('set_light',light=dict(direction=[.5,-1,.25],ambient=.2));assert capture('light-moved')!=angled
        c.call('set_light',light=dict(direction=[0,1,0],ambient=.2));back=capture('back');material({});assert capture('back-neutral')==back
        material(response());c.call('set_light',light=dict(direction=[0,-1,0],ambient=1));ambient=capture('ambient');material({});assert capture('ambient-neutral')==ambient
        material(response());c.call('set_light',light=dict(direction=[-.5,-1,-.25],ambient=.2))
        patterned=dict(response(),pattern=pattern(bump=.004),secondary_pattern=pattern(71,(1,2,1),.002));material(patterned);bump=capture('bump');assert bump!=angled
        material(response());assert capture('bump-removed')==angled
        for name,cam in [('grazing',dict(position=[4,1.501,4],target=[0,1.5,0],fov_degrees=36)),('coincident-plane',dict(position=[0,1.5,0],target=[0,1.5,1],fov_degrees=36))]:
            c.call('set_camera',camera=cam);capture(name)
        c.call('set_camera',camera=camera)
        # Entire sheet in a directional shadow: both direct BRDFs disappear.
        box=copy.deepcopy(next(e for e in entities if e['id']=='wall_north'));box.update(id='cover',visible=True,color=[.3]*3);box['transform']=dict(position=[3,7.5,1.5],size=[4.5,.3,4.5])
        c.call('edit_scene',edits=[dict(op='upsert',entity=box)]);c.call('set_light',light=dict(direction=[-.5,-1,-.25],ambient=.2,shadow=dict(center=[1,4,.5],extent=8,resolution=2048,bias=.0001)))
        shadow=capture('shadow');material({});assert capture('shadow-neutral')==shadow
        material(response());c.call('remove',id='cover');c.call('set_light',light=dict(direction=[-.5,-1,-.25],ambient=.2));assert capture('uncovered')==angled
        checks.append('Light/view direction and both bump layers affect metal; backlight/all-ambient/full-shadow controls exact, grazing/degenerate views safe')
        count=len(old['geometry']['positions']);edit('set_rig',asset='surface',rig=dict(bones=[dict(name='root',parent=None,translation=[0,0,0],rotation=[0,0,0,1])],weights=[dict(joints=[0]*4,weights=[1,0,0,0]) for _ in range(count)]))
        edit('put_clip',asset='surface',clip=dict(id='move',duration_ticks=120,tracks=[dict(bone=0,keys=[dict(tick=t,translation=[0,y,0],rotation=[0,0,0,1]) for t,y in [(0,0),(60,.5),(120,0)]])]))
        edit('put_instance',instance=dict(instance,position=[0,1,0],playback=dict(clip='move',offset_ticks=60,looping=True)));assert capture('compensated-rig')==angled
        edit('put_instance',instance=dict(instance,playback=dict(clip='move',offset_ticks=0,looping=True)));assert capture('restored-rig')==angled
        c.call('save_scene',path='examples/metallic.scene.json');saved=c.call('inspect')['scene'];c.call('step',ticks=60);live=capture('live');assert live!=angled
        c.call('save_replay',path='artifacts/metallic.replay.json');assert json.loads((ROOT/'artifacts/metallic.replay.json').read_text())['version']==REPLAY_VERSION
        assert c.call('run_replay',path='artifacts/metallic.replay.json')['meshes_matched']
        for name,tick,expected_pixels in [('replay',None,live),('prefix',0,angled)]:
            p=f'artifacts/metallic-{name}.png';c.call('capture_replay',replay_path='artifacts/metallic.replay.json',path=p,tick=tick,width=800,height=600);assert (ROOT/p).read_bytes()==expected_pixels
        c.call('load_scene',path='examples/metallic.scene.json');assert c.call('inspect')['scene']==saved and capture('loaded')==angled
        c.call('reset');assert capture('reset')==angled
        capture('viewport',640,480);assert capture('viewport-restored')==angled
        checks.append('Compensated rig exact, animated response changes; Replay78/prefix, scene74 load/reset and viewport restoration exact')
        report=dict(passed=True,checks=checks,build_id=c.call('inspect')['build_id'],normal_incidence_actual=actual,normal_incidence_expected=expected,blend_max_error=mix_error,native_window=window)
        (ROOT/'artifacts/metallic-verification.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(dict(passed=True,checks=len(checks),build_id=report['build_id'])))
    finally:c.close()


if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');p.add_argument('--window',action='store_true');a=p.parse_args();verify(a.binary.resolve(),a.window)
