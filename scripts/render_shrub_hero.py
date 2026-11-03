"""Run with headless Blender: native Ignis geometry, procedural Cycles lookdev.

blender --background --factory-startup --python-exit-code 1 \
  --python scripts/render_shrub_hero.py -- --stage build
Stages build/preview/final replace one current scene and stable render paths.
No photographs, generated bitmaps or external asset services are used.
"""
import argparse
import hashlib
import json
import math
import random
import sys
import time
from pathlib import Path

import bpy
import bmesh
import numpy as np
from mathutils import Vector

ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'artifacts/shrub-hero'


def aim(obj,target):obj.rotation_euler=(Vector(target)-obj.location).to_track_quat('-Z','Y').to_euler()


def nodes_material(name):
    mat=bpy.data.materials.new(name);mat.use_nodes=True;tree=mat.node_tree;tree.nodes.clear()
    output=tree.nodes.new('ShaderNodeOutputMaterial')
    return mat,tree,output


def math_node(tree,operation,a,b=None):
    n=tree.nodes.new('ShaderNodeMath');n.operation=operation
    for value,socket in zip([a,b],n.inputs):
        if value is None:continue
        if isinstance(value,bpy.types.NodeSocket):tree.links.new(value,socket)
        else:socket.default_value=value
    return n.outputs[0]


def leaf_material(name,color):
    mat,t,out=nodes_material(name);n=t.nodes;l=t.links
    uv=n.new('ShaderNodeUVMap');uv.uv_map='LeafSurface'
    sep=n.new('ShaderNodeSeparateXYZ');l.new(uv.outputs['UV'],sep.inputs[0]);u,v=sep.outputs['X'],sep.outputs['Y']
    distance=math_node(t,'ABSOLUTE',math_node(t,'SUBTRACT',v,.5))
    primary=math_node(t,'EXPONENT',math_node(t,'MULTIPLY',math_node(t,'POWER',math_node(t,'DIVIDE',distance,.032),2),-1))
    phase=math_node(t,'MULTIPLY',math_node(t,'SUBTRACT',u,math_node(t,'MULTIPLY',distance,.70)),math.pi*10)
    secondary=math_node(t,'MAXIMUM',math_node(t,'SUBTRACT',1,math_node(t,'DIVIDE',math_node(t,'ABSOLUTE',math_node(t,'SINE',phase)),.10)),0)
    veins=math_node(t,'ADD',primary,math_node(t,'MULTIPLY',secondary,.16))
    noise=n.new('ShaderNodeTexNoise');l.new(uv.outputs['UV'],noise.inputs['Vector']);noise.inputs['Scale'].default_value=85;noise.inputs['Detail'].default_value=2
    attr=n.new('ShaderNodeAttribute');attr.attribute_name='leaf_random'
    variation=math_node(t,'ADD',.82,math_node(t,'MULTIPLY',attr.outputs['Fac'],.30))
    variation=math_node(t,'MULTIPLY',variation,math_node(t,'ADD',.96,math_node(t,'MULTIPLY',veins,.055)))
    tint=n.new('ShaderNodeMixRGB');tint.blend_type='MULTIPLY';tint.inputs[0].default_value=1;tint.inputs[1].default_value=(*color,1);l.new(variation,tint.inputs[2])
    geo=n.new('ShaderNodeNewGeometry');back=n.new('ShaderNodeMixRGB');back.blend_type='MIX'
    l.new(geo.outputs['Backfacing'],back.inputs[0]);l.new(tint.outputs[0],back.inputs[1]);back.inputs[2].default_value=(color[0]*1.17,color[1]*1.07,color[2]*1.35,1)
    p=n.new('ShaderNodeBsdfPrincipled');l.new(back.outputs[0],p.inputs['Base Color']);p.inputs['IOR'].default_value=1.45
    p.inputs['Specular IOR Level'].default_value=.28;p.inputs['Coat Weight'].default_value=.04;p.inputs['Coat Roughness'].default_value=.30
    l.new(math_node(t,'ADD',.46,math_node(t,'MULTIPLY',noise.outputs['Fac'],.16)),p.inputs['Roughness'])
    bump=n.new('ShaderNodeBump');bump.inputs['Distance'].default_value=.000035;bump.inputs['Strength'].default_value=.5
    l.new(math_node(t,'ADD',veins,math_node(t,'MULTIPLY',noise.outputs['Fac'],.08)),bump.inputs['Height']);l.new(bump.outputs['Normal'],p.inputs['Normal'])
    thin=n.new('ShaderNodeBsdfTranslucent');l.new(back.outputs[0],thin.inputs['Color']);l.new(bump.outputs['Normal'],thin.inputs['Normal'])
    mix=n.new('ShaderNodeMixShader');mix.inputs[0].default_value=.22;l.new(p.outputs['BSDF'],mix.inputs[1]);l.new(thin.outputs[0],mix.inputs[2]);l.new(mix.outputs[0],out.inputs['Surface'])
    return mat


def earth_material(name,wood=False):
    mat,t,out=nodes_material(name);n=t.nodes;l=t.links
    coord=n.new('ShaderNodeTexCoord');mapping=n.new('ShaderNodeVectorMath');mapping.operation='MULTIPLY'
    l.new(coord.outputs['Object'],mapping.inputs[0]);mapping.inputs[1].default_value=(60,60,5) if wood else (1,1,1)
    noise=n.new('ShaderNodeTexNoise');l.new(mapping.outputs[0],noise.inputs['Vector']);noise.inputs['Scale'].default_value=4 if wood else 110;noise.inputs['Detail'].default_value=3
    ramp=n.new('ShaderNodeValToRGB');l.new(noise.outputs['Fac'],ramp.inputs[0])
    ramp.color_ramp.elements[0].position=.2;ramp.color_ramp.elements[0].color=(.035,.019,.009,1) if wood else (.0015,.001,.0005,1)
    ramp.color_ramp.elements[1].position=.8;ramp.color_ramp.elements[1].color=(.18,.105,.04,1) if wood else (.012,.008,.004,1)
    p=n.new('ShaderNodeBsdfPrincipled');l.new(ramp.outputs[0],p.inputs['Base Color']);p.inputs['Roughness'].default_value=.78 if wood else .94
    if not wood:p.inputs['Specular IOR Level'].default_value=.18
    bump=n.new('ShaderNodeBump');bump.inputs['Distance'].default_value=.0005 if wood else .0008;bump.inputs['Strength'].default_value=.35
    l.new(noise.outputs['Fac'],bump.inputs['Height']);l.new(bump.outputs[0],p.inputs['Normal']);l.new(p.outputs[0],out.inputs['Surface'])
    return mat


def import_native(report):
    document=json.loads((OUT/'native.scene.json').read_text());owned=[];audit=[]
    settings=json.loads((OUT/'native-report.json').read_text());nu=settings['blade_samples'];per_leaf=10*nu-6
    for asset in document['meshes']['assets']:
        geometry=asset['geometry']
        if len(geometry['triangles'])<=1:continue
        positions=np.array(geometry['positions'],dtype=np.float32);tri=np.array(geometry['triangles'],dtype=np.int32)
        assert tri.min()>=0 and tri.max()<len(positions) and np.isfinite(positions).all()
        # Rotation, determinant+1: Ignis Y-up -> Blender Z-up. No remodeling.
        positions=positions[:,[0,2,1]].copy();positions[:,1]*=-1
        mesh=bpy.data.meshes.new(asset['id']);mesh.vertices.add(len(positions));mesh.vertices.foreach_set('co',positions.ravel())
        mesh.loops.add(tri.size);mesh.loops.foreach_set('vertex_index',tri.ravel());mesh.polygons.add(len(tri))
        mesh.polygons.foreach_set('loop_start',np.arange(0,tri.size,3,dtype=np.int32));mesh.polygons.foreach_set('loop_total',np.full(len(tri),3,dtype=np.int32))
        mesh.update(calc_edges=True);assert not mesh.validate(verbose=False)
        obj=bpy.data.objects.new(asset['id'],mesh);bpy.context.collection.objects.link(obj);obj['ignis_hero']=True;obj.pass_index=1;owned.append(obj)
        mesh.polygons.foreach_set('use_smooth',np.ones(len(tri),dtype=np.bool_))
        check=np.empty_like(positions);mesh.vertices.foreach_get('co',check.ravel());assert np.array_equal(check,positions)
        idx=np.empty(tri.size,dtype=np.int32);mesh.loops.foreach_get('vertex_index',idx);assert np.array_equal(idx,tri.ravel())
        record=dict(id=asset['id'],vertices=len(positions),triangles=len(tri),import_exact=True,sha256=hashlib.sha256(positions.tobytes()+tri.tobytes()).hexdigest())
        if asset['id'].endswith('_leaves'):
            prefix=asset['id'][:-7];component=settings['components'][prefix]
            source=json.loads((ROOT/'assets/botany/reference-shrubs'/component['path']).read_text())
            organs=[o for o in source['source']['organs'] if o['shape']['kind']=='blade'];assert len(positions)==len(organs)*per_leaf
            blocks=positions.reshape(len(organs),per_leaf,3)
            roots=np.array([o['path'][0] for o in organs],np.float32)[:,[0,2,1]];roots[:,1]*=-1
            root_error=float(np.max(np.abs((blocks[:,0]+blocks[:,per_leaf//2])*.5-roots)));assert root_error<2e-6
            # Exact native strip grid order; every leaf gets its own local UV.
            side=[(0.,.5)]+[(i/nu,j/4) for i in range(1,nu) for j in range(5)]+[(1.,.5)]
            base=np.array(side+side,np.float32);assert len(base)==per_leaf
            vertex_uv=np.tile(base,(len(organs),1));uv=mesh.uv_layers.new(name='LeafSurface');uv.data.foreach_set('uv',vertex_uv[tri.ravel()].ravel())
            rng=np.random.default_rng(source['source']['seed']);attribute=mesh.attributes.new('leaf_random','FLOAT','POINT')
            attribute.data.foreach_set('value',np.repeat(rng.uniform(0,1,len(organs)).astype(np.float32),per_leaf))
            flush=component['layer']=='flush';mat=leaf_material('Fresh terminal leaves' if flush else 'Mature leaves',(.17,.30,.032) if flush else (.032,.105,.012))
            record.update(blades=len(organs),source_root_error_m=root_error,grid_uv_verified=True)
        else:mat=earth_material('Fine branch bark',wood=True)
        mesh.materials.append(mat);audit.append(record)
    report['geometry']=audit;report['leaf_count']=sum(a.get('blades',0) for a in audit);assert report['leaf_count']==13171
    return owned


def light(name,location,energy,size,color,target):
    data=bpy.data.lights.new(name,'AREA');obj=bpy.data.objects.new(name,data);bpy.context.collection.objects.link(obj)
    obj.location=location;data.energy=energy;data.shape='DISK';data.size=size;data.color=color;aim(obj,target);obj['ignis_hero']=True
    return obj


def configure(scene):
    scene.render.engine='CYCLES';prefs=bpy.context.preferences.addons['cycles'].preferences;prefs.compute_device_type='METAL';prefs.get_devices()
    devices=[]
    for device in prefs.devices:device.use=device.type=='METAL';devices.append(dict(name=device.name,type=device.type,enabled=device.use))
    assert any(d['enabled'] for d in devices),'Metal GPU unavailable'
    scene.cycles.device='GPU';scene.cycles.use_denoising=True;scene.cycles.denoising_input_passes='RGB_ALBEDO_NORMAL'
    scene.cycles.denoising_prefilter='ACCURATE';scene.cycles.max_bounces=12;scene.cycles.diffuse_bounces=4
    scene.cycles.glossy_bounces=4;scene.cycles.transmission_bounces=8;scene.cycles.transparent_max_bounces=8
    scene.cycles.caustics_reflective=False;scene.cycles.caustics_refractive=False;scene.cycles.seed=11701
    scene.render.resolution_x=4096;scene.render.resolution_y=4096;scene.render.resolution_percentage=100
    scene.render.image_settings.color_mode='RGBA';scene.render.image_settings.color_depth='16';scene.render.image_settings.file_format='PNG'
    scene.render.film_transparent=False;scene.view_settings.view_transform='AgX'
    scene.view_settings.look='AgX - Medium High Contrast';scene.view_settings.exposure=.2
    scene.world.color=(.04,.04,.04);scene.world.use_nodes=True
    background=next(n for n in scene.world.node_tree.nodes if n.type=='BACKGROUND');background.inputs['Color'].default_value=(.27,.34,.45,1);background.inputs['Strength'].default_value=.25
    bpy.context.view_layer.use_pass_object_index=True
    scene.render.use_persistent_data=True;bpy.context.preferences.filepaths.save_version=0
    return devices


def build():
    for obj in list(bpy.data.objects):bpy.data.objects.remove(obj,do_unlink=True)
    scene=bpy.context.scene;report=dict(blender=bpy.app.version_string,asset='esfera_miuda_0',canonical_curves_preserved=True)
    report['devices']=configure(scene);plant=import_native(report)
    soil=earth_material('Dark fine soil')
    mesh=bpy.data.meshes.new('Ground');mesh.from_pydata([(-50,-50,0),(50,-50,0),(50,50,0),(-50,50,0)],[],[(0,1,2,3)]);mesh.update()
    ground=bpy.data.objects.new('Ground',mesh);bpy.context.collection.objects.link(ground);mesh.materials.append(soil);ground['ignis_hero']=True
    bm=bmesh.new();bmesh.ops.create_icosphere(bm,subdivisions=2,radius=1);stone_mesh=bpy.data.meshes.new('Shared pebble');bm.to_mesh(stone_mesh);bm.free();stone_mesh.materials.append(soil)
    randomizer=random.Random(11701)
    for j in range(90):
        obj=bpy.data.objects.new(f'Pebble_{j:03}',stone_mesh);bpy.context.collection.objects.link(obj);a=randomizer.random()*math.tau;r=randomizer.uniform(.22,1.5);size=randomizer.uniform(.003,.012)
        obj.location=(r*math.cos(a),r*math.sin(a),size*.24);obj.scale=(size*1.5,size,size*.5);obj.rotation_euler=(randomizer.random(),randomizer.random(),randomizer.random()*math.tau);obj['ignis_hero']=True
    camera_data=bpy.data.cameras.new('Hero camera');camera=bpy.data.objects.new('Hero camera',camera_data);bpy.context.collection.objects.link(camera)
    camera.location=(1.35,-3.10,1.16);target=(0,0,.52);aim(camera,target);camera_data.lens=85;camera_data.clip_start=.01;camera_data.clip_end=150
    camera_data.dof.use_dof=True;camera_data.dof.focus_distance=(camera.location-Vector((0,-.08,.52))).length;camera_data.dof.aperture_fstop=11;scene.camera=camera
    lights=[light('Warm broad key',(-2.8,-3.0,4),650,2.8,(1,.92,.80),target),light('Cool sky fill',(2.5,-1.5,2.1),90,3.,(.72,.83,1),target),light('Soft back edge',(-1.8,1.8,2.7),380,1.8,(1,.96,.85),target)]
    bpy.context.view_layer.update()
    report['camera']=dict(position=list(camera.location),target=target,lens_mm=85,fstop=11)
    report['lights']=[dict(name=o.name,location=list(o.location),watts=o.data.energy,size_m=o.data.size,color=list(o.data.color)) for o in lights]
    report['materials_are_render_only']=True
    report['image_texture_nodes']=sum(n.type=='TEX_IMAGE' for m in bpy.data.materials if m.node_tree for n in m.node_tree.nodes)
    assert report['image_texture_nodes']==0
    assert all(link.is_valid for material in bpy.data.materials if material.node_tree for link in material.node_tree.links)
    report['node_links_valid']=True;report['objects']=len(bpy.data.objects)
    (OUT/'bridge-report.json').write_text(json.dumps(report,indent=2));bpy.ops.wm.save_as_mainfile(filepath=str(OUT/'hero.blend'),compress=True)
    return scene


def save_images(scene,stem,master=False):
    result=bpy.data.images['Render Result'];scene.render.image_settings.media_type='IMAGE';scene.render.image_settings.file_format='PNG';scene.render.image_settings.color_depth='16';result.save_render(str(OUT/f'{stem}.png'),scene=scene)
    if master:
        scene.render.image_settings.media_type='MULTI_LAYER_IMAGE';scene.render.image_settings.file_format='OPEN_EXR_MULTILAYER';scene.render.image_settings.color_depth='32';scene.render.image_settings.exr_codec='ZIP';result.save_render(str(OUT/f'{stem}.exr'),scene=scene)
        scene.render.image_settings.media_type='IMAGE';scene.render.image_settings.file_format='PNG';scene.render.image_settings.color_depth='16'


def tune(scene):
    scene.camera.location=(1.35,-3.10,1.55);aim(scene.camera,(0,0,.52))
    scene.camera.data.dof.focus_distance=(scene.camera.location-Vector((0,-.08,.52))).length
    fill=bpy.data.objects['Cool sky fill'];fill.data.energy=300
    receivers=bpy.data.collections.get('Hero plant receivers') or bpy.data.collections.new('Hero plant receivers')
    for obj in scene.objects:
        if obj.type=='MESH' and obj.pass_index==1 and obj.name not in receivers.objects:receivers.objects.link(obj)
    fill.light_linking.receiver_collection=receivers
    for name in ['Mature leaves','Fresh terminal leaves']:
        material=bpy.data.materials[name];p=next(n for n in material.node_tree.nodes if n.type=='BSDF_PRINCIPLED')
        p.inputs['Specular IOR Level'].default_value=.28;p.inputs['Coat Weight'].default_value=.04
        rough=p.inputs['Roughness'].links[0].from_node;assert rough.type=='MATH' and rough.operation=='ADD';rough.inputs[0].default_value=.46
    soil=bpy.data.materials['Dark fine soil'];next(n for n in soil.node_tree.nodes if n.type=='BUMP').inputs['Distance'].default_value=.0008
    ramp=next(n for n in soil.node_tree.nodes if n.type=='VALTORGB').color_ramp
    ramp.elements[0].color=(.0015,.001,.0005,1);ramp.elements[1].color=(.012,.008,.004,1)
    next(n for n in soil.node_tree.nodes if n.type=='BSDF_PRINCIPLED').inputs['Specular IOR Level'].default_value=.18
    report=json.loads((OUT/'bridge-report.json').read_text());report['camera']['position']=list(scene.camera.location)
    report['lights']=[dict(name=o.name,location=list(o.location),watts=o.data.energy,size_m=o.data.size,color=list(o.data.color)) for o in scene.objects if o.type=='LIGHT']
    report['fill_receivers']=sorted(obj.name for obj in receivers.objects)
    report.pop('image_textures',None);report['image_texture_nodes']=sum(n.type=='TEX_IMAGE' for m in bpy.data.materials if m.node_tree for n in m.node_tree.nodes);assert report['image_texture_nodes']==0
    for record in report['geometry']:
        mesh=bpy.data.objects[record['id']].data;positions=np.empty((len(mesh.vertices),3),np.float32);tri=np.empty((len(mesh.polygons),3),np.int32)
        mesh.vertices.foreach_get('co',positions.ravel());mesh.loops.foreach_get('vertex_index',tri.ravel())
        assert hashlib.sha256(positions.tobytes()+tri.tobytes()).hexdigest()==record['sha256']
    report['geometry_reopen_hash_exact']=True;(OUT/'bridge-report.json').write_text(json.dumps(report,indent=2))


def review(scene):
    scene.cycles.samples=24;scene.cycles.use_adaptive_sampling=True;scene.cycles.adaptive_threshold=.05;scene.cycles.adaptive_min_samples=8;scene.render.resolution_percentage=12
    camera=scene.camera;target=Vector((0,0,.52));distance=(camera.location-target).length
    paths=[]
    for name,direction in [('front',(.42,-1,.32)),('side',(1,.15,.32)),('back',(-.25,1,.32))]:
        camera.location=target+Vector(direction).normalized()*distance;aim(camera,target);camera.data.dof.focus_distance=distance
        bpy.ops.render.render();save_images(scene,f'review-{name}');paths.append(str(OUT/f'review-{name}.png'))
    return paths


def quality_check(scene,stem,toolkit):
    sys.path.insert(0,str(toolkit));import bx_light as BL
    import OpenImageIO as oiio
    # Explicit float64 luminance avoids spurious floating-point warnings from
    # the local NumPy/Accelerate matmul path on large4K arrays. Check inputs too.
    def luminance(rgb):
        a=np.asarray(rgb[...,:3],np.float64);assert np.isfinite(a).all()
        return a[...,0]*.2126+a[...,1]*.7152+a[...,2]*.0722
    BL.luma=luminance
    path=str(OUT/f'{stem}.exr');inp=oiio.ImageInput.open(path);part=0;mask=None
    while inp.seek_subimage(part,0):
        if 'Object Index' in (inp.spec().getattribute('name') or ''):
            values=np.asarray(inp.read_image(part,0,0,1,oiio.FLOAT));mask=(values[...,0] if values.ndim==3 else values)>.5;break
        part+=1
    inp.close();assert mask is not None
    report=BL.analyse(str(OUT/f'{stem}.png'),mask,exr=path);report['verdict']=BL.verdict(report)
    report['color_audit']=BL.color_audit(scene);report['lighting']=BL.light_report(scene=scene,target=(0,0,.52),cam=scene.camera,key=bpy.data.objects['Warm broad key'])
    BL.value_study(str(OUT/f'{stem}.png'),str(OUT/f'{stem}-values.png'),mask=mask)
    reviews=[OUT/f'review-{name}.png' for name in ['front','side','back']]
    if all(p.exists() for p in reviews):BL.contact_sheet([str(p) for p in reviews],str(OUT/'review-sheet.png'),cols=3,width=480)
    if stem=='hero':
        pixels=BL.load_rgba(str(OUT/'hero.png'));BL.save_png(pixels[1000:1900,1150:2050],str(OUT/'detail-crop.png'))
        report['detail_crop_box']=[1150,1000,900,900]
    report['finite_input_pixels']=True
    (OUT/f'{stem}-quality.json').write_text(json.dumps(report,indent=2,default=str));return report['verdict']


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--stage',choices=['build','preview','final','review','qa'],default='build')
    parser.add_argument('--toolkit',type=Path);parser.add_argument('--image-stem',choices=['preview','hero'],default='preview')
    args=parser.parse_args(sys.argv[sys.argv.index('--')+1:] if '--' in sys.argv else [])
    OUT.mkdir(parents=True,exist_ok=True);started=time.monotonic()
    if args.stage=='build':build()
    else:bpy.ops.wm.open_mainfile(filepath=str(OUT/'hero.blend'))
    scene=bpy.context.scene;configure(scene);tune(scene)
    if args.stage=='review':
        paths=review(scene);print('RESULT '+json.dumps(dict(stage='review',images=paths)),flush=True);return
    if args.stage=='qa':
        assert args.toolkit is not None,'Supply the optional local bx_light toolkit for quality analysis'
        verdict=quality_check(scene,args.image_stem,args.toolkit);print('RESULT '+json.dumps(dict(stage='qa',verdict=verdict)),flush=True);return
    final=args.stage=='final';scene.cycles.samples=1024 if final else 64;scene.cycles.use_adaptive_sampling=True;scene.cycles.adaptive_threshold=.005 if final else .025;scene.cycles.adaptive_min_samples=64 if final else 16
    scene.render.resolution_percentage=100 if final else 25
    bpy.ops.render.render();save_images(scene,'hero' if final else 'preview',master=True)
    bpy.ops.wm.save_as_mainfile(filepath=str(OUT/'hero.blend'),compress=True)
    result=dict(stage=args.stage,seconds=time.monotonic()-started,blender=bpy.app.version_string,width=scene.render.resolution_x*scene.render.resolution_percentage//100,height=scene.render.resolution_y*scene.render.resolution_percentage//100,samples_max=scene.cycles.samples,adaptive_threshold=scene.cycles.adaptive_threshold)
    (OUT/f'{"render" if final else "preview"}-report.json').write_text(json.dumps(result,indent=2));print('RESULT '+json.dumps(result),flush=True)


if __name__=='__main__':main()
