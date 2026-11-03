#!/usr/bin/env python3
"""Two bounded procedural fields through native MCP, GPU, history and replay."""
from verify_lab import SCENE_VERSION, REPLAY_VERSION
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT
from verify_materials import f32
from png_pixels import max_error


def pattern(seed=23, frequency=(4, 2, 6), bump=.006, low=(.7,.8,.6), high=(1.2,1.1,1.3)):
    return f32(dict(frequency=frequency, low=low, high=high, bump=bump,
                    warp=.3, octaves=2, seed=seed, contrast=1.2))


def verify(binary, window):
    c=Client(binary, window); checks=[]
    def edit(kind, **kw): return c.call('edit_mesh', edit=dict(kind=kind, **kw))
    def mat(m): edit('set_material', asset='surface', material=f32(m))
    def capture(name, width=800, height=600):
        p=f'artifacts/layered-{name}.png'; c.call('capture', path=p, width=width, height=height)
        return (ROOT/p).read_bytes()
    try:
        c.request('initialize', {'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'layered-patterns','version':'1'}})
        c.request('notifications/initialized', notification=True)
        assert 'secondary_pattern' in json.dumps(c.request('tools/list'))
        c.call('generate', recipe=dict(width=16,depth=14,seed=7,obstacles=0))
        c.call('edit_scene', edits=[dict(op='upsert',entity=dict(e,visible=False)) for e in c.call('inspect')['scene']['entities']])
        edit('grid', id='unused', segments=1, size=1)
        edit('grid', id='surface', segments=3, size=3)
        edit('set_shading', asset='surface', shading='smooth')
        instance=dict(id='sheet',asset='surface',position=[0,1.5,0],rotation=[0,0,0,1],scale=1,color=[.35,.29,.17],playback=None)
        edit('put_instance', instance=instance)
        camera=dict(position=[-2.5,6.5,-1.25],target=[0,1.5,0],fov_degrees=36)
        c.call('set_camera',camera=camera); c.call('set_light',light=dict(direction=[-.5,-1,-.25],ambient=.2))
        p=pattern(); q=pattern(71,(1.4,1.2,1.7),.004,(.65,.8,.95),(1.1,1.05,.9))
        material=dict(pattern=p,secondary_pattern=q,reflectance=dict(roughness=.65,f0=.04))
        mat(dict(pattern=p)); primary=capture('primary')
        mat(dict(secondary_pattern=p)); assert capture('secondary-alone')==primary
        mat(dict(pattern=p,secondary_pattern=pattern(1,(2,3,4),0,(1,1,1),(1,1,1))))
        assert capture('neutral-secondary')==primary
        mat(material); mixed=capture('mixed'); assert mixed!=primary and capture('repeat')==mixed
        for name,change in [('color',dict(low=[1]*3,high=[1]*3)),('bump',dict(bump=0)),('frequency',dict(frequency=[8,6,7])),('seed',dict(seed=13)),('kind',dict(kind='cellular'))]:
            mat(dict(material,secondary_pattern=dict(q,**change))); assert capture(name)!=mixed
        mat(material)
        mat(dict(material,pattern=q,secondary_pattern=p)); swap_error=max_error(capture('swapped'),mixed); assert swap_error<=1,swap_error
        mat(material)
        checks.append('Schema, neutral layer exact, primary/secondary-alone exact, independent color/bump/frequency/seed/kind, exact subpixel fade and commutative fields within1LSB')
        before=c.call('inspect'); old=c.call('inspect_mesh',asset='surface')['asset']
        for field,value in [('frequency',[0,1,1]),('low',[-.1,1,1]),('high',[3,1,1]),('bump',.051),('warp',3),('octaves',0),('contrast',0)]:
            bad=dict(material,secondary_pattern=dict(q,**{field:value}))
            for op in ['preview_mesh_edit','edit_mesh']:
                c.call(op,edit=dict(kind='set_material',asset='surface',material=bad),expected_error=True)
                assert c.call('inspect')==before
        bad=dict(pattern=dict(p,bump=.03),secondary_pattern=dict(q,bump=.03))
        c.call('edit_mesh',edit=dict(kind='set_material',asset='surface',material=bad),expected_error=True)
        assert c.call('inspect')==before
        mat(dict(pattern=p)); plain=capture('history-plain'); before=c.call('inspect')
        preview=c.call('preview_mesh_edit',edit=dict(kind='set_material',asset='surface',material=material))
        assert c.call('inspect')==before
        mat(material); assert c.call('inspect_mesh',asset='surface')['asset']==preview['meshes']['assets'][1]
        assert c.call('inspect_mesh',asset='surface')['asset']['geometry']==old['geometry']
        assert capture('published')==mixed
        c.call('undo'); assert capture('undo')==plain
        c.call('redo'); assert capture('redo')==mixed
        edit('remove_asset',id='unused'); assert capture('slot-reindexed')==mixed
        checks.append('Same validation, aggregate bump rejection, atomic errors/pure previews, publication/geometry/undo/redo and material slot cache')
        c.call('save_scene',path='examples/layered-patterns.scene.json'); saved=c.call('inspect')['scene']; assert saved['version']==SCENE_VERSION
        forged=copy.deepcopy(saved); forged['version']=72
        (ROOT/'artifacts/layered-old.scene.json').write_text(json.dumps(forged))
        before=c.call('inspect'); c.call('load_scene',path='artifacts/layered-old.scene.json',expected_error=True); assert c.call('inspect')==before
        # Hidden canonical sources must be gated before regenerated visible meshes.
        source_path=ROOT/'examples/native-tree-study.scene.json'
        tree=json.loads(source_path.read_text()); tree['version']=72
        def hidden_sources(scene):
            out=[]
            for entry in scene['plants']['entries']:
                out.append(entry['source'])
                if entry.get('assembly'):
                    assembly=entry['assembly'];out.append(assembly['context'])
                    out.extend(g['source'] for g in assembly.get('context_grafts',[]))
            return out
        candidates=hidden_sources(tree);assert len(candidates)==3
        for i in range(len(candidates)):
            forged=copy.deepcopy(tree)
            hidden_sources(forged)[i]['material']['secondary_pattern']=q
            path='artifacts/layered-hidden.scene.json'; (ROOT/path).write_text(json.dumps(forged))
            error=c.call('load_scene',path=path,expected_error=True)
            assert 'Secondary material patterns require scene version73' in json.dumps(error),error
            assert c.call('inspect')==before
        checks.append(f'Scene73 root and{len(candidates)} canonical source gates reject scene72 before recipe evaluation')
        count=len(old['geometry']['positions'])
        edit('set_rig',asset='surface',rig=dict(bones=[dict(name='root',parent=None,translation=[0,0,0],rotation=[0,0,0,1])],weights=[dict(joints=[0]*4,weights=[1,0,0,0]) for _ in range(count)]))
        edit('put_clip',asset='surface',clip=dict(id='move',duration_ticks=120,tracks=[dict(bone=0,keys=[dict(tick=t,translation=[0,y,0],rotation=[0,0,0,1]) for t,y in [(0,0),(60,.5),(120,0)]])]))
        edit('put_instance',instance=dict(instance,position=[0,1,0],playback=dict(clip='move',offset_ticks=60,looping=True)))
        assert capture('compensated-rig')==mixed
        edit('put_instance',instance=dict(instance,playback=dict(clip='move',offset_ticks=0,looping=True)))
        assert capture('restored-rig')==mixed
        c.call('save_scene',path='examples/layered-patterns.scene.json'); saved=c.call('inspect')['scene']
        c.call('step',ticks=60); live=capture('live'); assert live!=mixed
        c.call('save_replay',path='artifacts/layered.replay.json')
        assert json.loads((ROOT/'artifacts/layered.replay.json').read_text())['version']==REPLAY_VERSION
        assert c.call('run_replay',path='artifacts/layered.replay.json')['meshes_matched']
        for name,tick,expected in [('replay',None,live),('prefix',0,mixed)]:
            path=f'artifacts/layered-{name}.png';c.call('capture_replay',replay_path='artifacts/layered.replay.json',path=path,tick=tick,width=800,height=600);assert (ROOT/path).read_bytes()==expected
        c.call('load_scene',path='examples/layered-patterns.scene.json'); assert c.call('inspect')['scene']==saved and capture('loaded')==mixed
        c.call('reset'); assert capture('reset')==mixed
        capture('viewport',640,480);assert capture('viewport-restored')==mixed
        checks.append('Animated rest-domain attachment and compensated rig exact; Replay77/prefix, load/reset, repeat and viewport restoration exact')
        report=dict(passed=True,checks=checks,build_id=c.call('inspect')['build_id'],swap_max_error=swap_error,native_window=window)
        (ROOT/'artifacts/layered-patterns-verification.json').write_text(json.dumps(report,indent=2)+'\n')
        print(json.dumps(dict(passed=True,checks=len(checks),build_id=report['build_id'])))
    finally: c.close()


if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');p.add_argument('--window',action='store_true');a=p.parse_args();verify(a.binary.resolve(),a.window)
