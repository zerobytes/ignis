#!/usr/bin/env python3
"""Compare general dielectric materials on the same saved native tree via MCP."""
import argparse
import copy
import hashlib
import json
import math
from pathlib import Path
import shutil
from demo_tree_authoring import PROJECT, OUTPUT, CAMERA, LIGHT
from demo_tree_smoothing import run as finish
from verify_lab import Client


def unit(v):
    length=math.sqrt(sum(x*x for x in v)); return [x/length for x in v]


def run(frames=48, input_path=None):
    input_path=input_path or PROJECT/'examples/native-tree-study.scene.json'
    snapshot=OUTPUT/'examples/reflectance-input.scene.json'
    if snapshot.resolve()!=input_path.resolve(): shutil.copyfile(input_path,snapshot)
    c=Client(PROJECT/'target/release/ignis',False); call=c.call
    def asset(id): return call('inspect_mesh',asset=id)['asset']
    def pose(id,tick): return call('inspect_mesh_pose',instance=id,tick=tick)['pose']
    def capture(name):
        call('capture',path=f'artifacts/reflectance-{name}.png',width=1000,height=900)
        return hashlib.sha256((OUTPUT/f'artifacts/reflectance-{name}.png').read_bytes()).hexdigest()
    cameras={'whole':CAMERA,'bark':{'position':[1.3,2.3,2],'target':[0.1,2,0.1],'fov_degrees':32},
             'fork':{'position':[1.4,2.2,3.5],'target':[-0.05,1.7,0.1],'fov_degrees':40}}
    def leaf_view(name, override=None):
        before=call('inspect')['scene']
        inspected=call('inspect_mesh_islands',asset='foliage-0')
        edits=[{'op':'mesh','edit':{'kind':'extract_islands','asset':'foliage-0','id':'leaf-response-preview','reference':inspected['reference'],
            'islands':[inspected['topology']['islands'][0]['id']],'mode':'copy'}}]
        if override is not None: edits.append({'op':'mesh','edit':{'kind':'set_material','asset':'leaf-response-preview','material':override}})
        edits += [{'op':'mesh','edit':{'kind':'remove_instance','id':i['id']}} for i in before['meshes']['instances']]
        edits.append({'op':'mesh','edit':{'kind':'put_instance','instance':{'id':'leaf-response-view','asset':'leaf-response-preview',
            'position':[0,3.2,0],'rotation':[0,0,0,1],'scale':1,'color':[0.09,0.26,0.045],
            'playback':{'clip':'breeze','offset_ticks':0,'looping':True}}}})
        call('edit_scene',edits=edits)
        points=pose('leaf-response-view',0)['positions']; triangles=asset('leaf-response-preview')['geometry']['triangles']
        n=[0.,0.,0.]
        for a,b,d in triangles:
            u=[points[b][k]-points[a][k] for k in range(3)];v=[points[d][k]-points[a][k] for k in range(3)]
            cross=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]]
            n=[n[k]+cross[k] for k in range(3)]
        n=unit(n); tangent=unit([points[-1][k]-points[0][k] for k in range(3)])
        tangent=unit([tangent[k]-sum(tangent[j]*n[j] for j in range(3))*n[k] for k in range(3)])
        centre=[(min(p[k] for p in points)+max(p[k] for p in points))/2 for k in range(3)]
        extent=max(max(p[k] for p in points)-min(p[k] for p in points) for k in range(3))
        view=unit([n[k]+0.4*tangent[k] for k in range(3)])
        light={'direction':[-x for x in unit([n[k]-0.4*tangent[k] for k in range(3)])],'ambient':0.22}
        camera={'position':[centre[k]+view[k]*extent*2.6 for k in range(3)],'target':centre,'fov_degrees':32}
        call('edit_scene',edits=[{'op':'set_camera','camera':camera},{'op':'set_light','light':light}])
        digest=capture(name)
        call('undo');call('undo');assert call('inspect')['scene']==before
        return {'sha256':digest,'camera':camera,'light':light,'tick':0}
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'ignis-tree-reflectance','version':'1'}})
        c.request('notifications/initialized',notification=True)
        call('load_scene',path='examples/reflectance-input.scene.json')
        # Re-running on the current tree compares an explicit neutral ablation,
        # rather than silently treating the already-applied material as "before".
        loaded_assets=call('inspect')['scene']['meshes']['assets']
        neutral_edits=[]
        for a in loaded_assets:
            if a['material'].get('reflectance') is not None:
                m=copy.deepcopy(a['material']);m.pop('reflectance')
                neutral_edits.append({'op':'mesh','edit':{'kind':'set_material','asset':a['id'],'material':m}})
        if neutral_edits:call('edit_scene',edits=neutral_edits)
        initial=call('inspect')['scene']; original_assets=initial['meshes']['assets']; instances=initial['meshes']['instances']
        initial_plant=call('inspect_plant',id='branch-editable-1')['plant']
        poses={tick:{i['id']:pose(i['id'],tick) for i in instances} for tick in [0,60]}
        call('set_light',light=LIGHT)
        views={}
        for name,camera in cameras.items():
            call('set_camera',camera=camera);call('step',ticks=60);views[name]={'before':capture('before-'+name)}
        views['crown_leaf']={'before':leaf_view('before-crown-leaf')}
        edits=[]
        for a in original_assets:
            if a['id']=='blade-source' or a['id'].startswith('foliage-'): roughness,f0=0.58,0.035
            elif a['id'] in ['wood','branch-editable-1','secondary-06','secondary-twigs']: roughness,f0=0.92,0.035
            else:continue
            m=copy.deepcopy(a['material']);m['reflectance']={'roughness':roughness,'f0':f0}
            edits.append({'op':'mesh','edit':{'kind':'set_material','asset':a['id'],'material':m}})
        call('edit_scene',edits=edits)
        changed=call('inspect')['scene']['meshes']; assert changed['instances']==instances
        for a in original_assets:
            b=asset(a['id']); aa=copy.deepcopy(a);bb=copy.deepcopy(b)
            aa['material'].pop('reflectance',None);bb['material'].pop('reflectance',None); assert aa==bb
        for tick in [0,60]:
            for id,p in poses[tick].items(): assert pose(id,tick)==p
        plant=call('inspect_plant',id='branch-editable-1')['plant']
        for key in ['definition','reference']:assert plant[key]==initial_plant[key]
        old_source=copy.deepcopy(initial_plant['source']);new_source=copy.deepcopy(plant['source'])
        old_source['material'].pop('reflectance',None);new_source['material'].pop('reflectance',None);assert old_source==new_source
        call('undo');assert call('inspect')['scene']['meshes']==initial['meshes']
        call('redo');assert call('inspect')['scene']['meshes']==changed
        for name,camera in cameras.items():
            call('set_camera',camera=camera);call('step',ticks=60);views[name]['after']=capture('after-'+name)
            assert views[name]['before']!=views[name]['after']
        views['crown_leaf']['after']=leaf_view('after-crown-leaf')
        before,after=views['crown_leaf']['before'],views['crown_leaf']['after']
        assert before['camera']==after['camera'] and before['light']==after['light'] and before['sha256']!=after['sha256']
        material=asset('foliage-0')['material']
        for r in [0.3,0.9]:
            m=copy.deepcopy(material);m['reflectance']['roughness']=r
            views['crown_leaf'][f'roughness_{r}']=leaf_view(f'leaf-roughness-{r}',m)
            assert views['crown_leaf'][f'roughness_{r}']['sha256']!=after['sha256']
        m=copy.deepcopy(material);m['reflectance']['f0']=0
        views['crown_leaf']['zero']=leaf_view('leaf-zero',m)
        assert views['crown_leaf']['zero']['sha256']==before['sha256']
        call('set_camera',camera=cameras['bark']);call('step',ticks=60)
        bark_edit=[]
        for id in ['wood','branch-editable-1','secondary-06','secondary-twigs']:
            m=copy.deepcopy(asset(id)['material']);m['reflectance']['roughness']=0.3
            bark_edit.append({'op':'mesh','edit':{'kind':'set_material','asset':id,'material':m}})
        call('edit_scene',edits=bark_edit);call('step',ticks=60);views['bark']['glossy_control']=capture('bark-glossy-control')
        assert views['bark']['glossy_control']!=views['bark']['after']
        call('undo');call('step',ticks=60);assert capture('bark-restored')==views['bark']['after']
        call('set_camera',camera=CAMERA);call('save_scene',path='examples/reflectance-tree-candidate.scene.json')
        report={'passed':True,'build_id':call('inspect')['build_id'],'native_window':False,'input_sha256':hashlib.sha256(snapshot.read_bytes()).hexdigest(),
            'input_response_removed_for_neutral_control':bool(neutral_edits),
            'materials_only':True,'geometry_channels_rigs_clips_instances_poses_exact':True,'plant_source_and_reference_preserved':True,
            'leaf_response':material['reflectance'],'bark_response':asset('wood')['material']['reflectance'],'views':views,
            'assets':len(changed['assets']),'instances':len(instances)}
        # Offline measurements of native captures, not image alteration or a
        # realism score. Same camera/light/tick and foreground in each pair.
        import numpy as np
        from PIL import Image
        report['pixel_metrics']={}
        for label,a,b in [('leaf-before-after','before-crown-leaf','after-crown-leaf'),
                          ('leaf-roughness','leaf-roughness-0.3','leaf-roughness-0.9'),
                          ('bark-before-after','before-bark','after-bark'),
                          ('bark-roughness','bark-glossy-control','after-bark')]:
            with Image.open(OUTPUT/f'artifacts/reflectance-{a}.png') as image: x=np.array(image.convert('RGB'),dtype=np.int16)
            with Image.open(OUTPUT/f'artifacts/reflectance-{b}.png') as image: y=np.array(image.convert('RGB'),dtype=np.int16)
            mask=np.any(x!=x[0,0],axis=2)|np.any(y!=y[0,0],axis=2);delta=np.abs(x-y)
            report['pixel_metrics'][label]={'foreground_pixels':int(mask.sum()),'changed_pixels':int(np.any(delta,axis=2).sum()),
                'mean_absolute_channel_delta_foreground_255':float(delta[mask].mean()),'maximum_channel_delta':int(delta.max())}
        (OUTPUT/'artifacts/reflectance-report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
    finally:c.close()
    finish(frames,4,OUTPUT/'examples/reflectance-tree-candidate.scene.json')


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--frames',type=int,default=48);p.add_argument('--input',type=Path);a=p.parse_args()
    if a.frames<0 or a.frames>120 or (a.frames and 240%a.frames):p.error('frames must be0 or a positive divisor of240 up to120')
    run(a.frames,a.input)
