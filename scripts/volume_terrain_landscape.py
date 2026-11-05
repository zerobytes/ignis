#!/usr/bin/env python3
"""A ten-kilometre landscape with independent feature width and altitude scales."""
import argparse,copy,json,statistics,time
from verify_lab import Client,ROOT
from open_earth_walk import init
from volume_terrain_demo import recipe
PREFIX='artifacts/volumetric-terrain-landscape'
OUT=ROOT/PREFIX

def timed(call,*args,**kwargs):
    t=time.perf_counter();result=call(*args,**kwargs)
    return result,(time.perf_counter()-t)*1000

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--tiles',type=int,default=4);parser.add_argument('--seed',type=int,default=2718);parser.add_argument('--extent',type=float,default=10000.);parser.add_argument('--height-scale',type=float,default=8.);args=parser.parse_args()
    if not 1<=args.tiles<=8:raise ValueError('Use1..8 tiles per horizontal axis for this resident experiment')
    if not 1000<=args.extent<=10000 or not 1<=args.height_scale<=10:raise ValueError('This fixture covers1000..10000m and height scale1..10')
    OUT.mkdir(parents=True,exist_ok=True);doc=recipe(args.seed)
    extent=args.extent;size=extent/args.tiles;half=extent/2;spacing=size/50;vstep=args.height_scale
    doc.update(version=2,noise_domain_scale=[208/extent,1/args.height_scale,208/extent])
    doc['vertical_strength']/=args.height_scale
    doc['terraces']['spacing']*=args.height_scale
    doc['floor']['altitude']*=args.height_scale;doc['floor']['transition']*=args.height_scale
    c=Client(ROOT/'target/release/ignis',log_path=OUT/'native.log')
    try:
        init(c);bench=[]
        for i in range(4):
            result,ms=timed(c.call,'query_volume_terrain_gpu',document=recipe(args.seed),grid=dict(origin=[-32,-24,-32],cells=[128,96,128],spacing=.5))
            bench.append(dict(mcp_ms=ms,**result['timing_wall_ms']))
        c.call('generate',recipe=dict(width=20,depth=20,seed=221,obstacles=0))
        scene=c.call('inspect')['scene'];player=copy.deepcopy(next(e for e in scene['entities'] if e['id']=='player'))
        player['transform']['position']=[0,-90,0]
        c.call('edit_scene',edits=[dict(op='remove',id=e['id']) for e in scene['entities'] if e['id']!='player']+[dict(op='upsert',entity=player)])
        c.call('save_volume_terrain',path=f'{PREFIX}/source.ignis-volume',document=doc)
        material=dict(pattern=dict(kind='ridged',frequency=[.13,.17,.13],low=[.68,.69,.66],high=[1,.98,.9],bump=.025,warp=.35,octaves=3,seed=47,contrast=1.1),reflectance=dict(roughness=.95,f0=.02))
        blocks=[];bounds={};publication_mcp_ms=0.;generation_start=time.perf_counter()
        for z in range(args.tiles):
            for x in range(args.tiles):
                origin=[-half+x*size,-48.*args.height_scale,-half+z*size];name=f'volume_{x}_{z}'
                grid=dict(origin=origin,cells=[50,96,50],spacing=spacing,vertical_spacing=vstep)
                result,ms=timed(c.call,'publish_volume_terrain_gpu',publication=dict(document=doc,grid=grid,asset=name,instance=name,position=origin,color=[.51,.48,.40],material=material,collision=True))
                assert not result['visual_only'];publication_mcp_ms+=ms
                blocks.append(dict(id=name,grid=grid,mcp_ms=ms,terrain=result['terrain']))
                # Read-only comparison of native output on shared block planes.
                geometry=c.call('inspect_mesh',asset=name)['asset']['geometry']
                blocks[-1]['surface_height_range']=[min(p[1] for p in geometry['positions'])+origin[1],max(p[1] for p in geometry['positions'])+origin[1]]
                faces={}
                for axis,label in [(0,'x'),(2,'z')]:
                    for value,suffix in [(0.,'lo'),(size,'hi')]:
                        faces[label+suffix]={tuple(float(v)+origin[i] for i,v in enumerate(p)) for p in geometry['positions'] if p[axis]==value}
                # A bounded cut must not clip the top/bottom of the intended terrain.
                assert not any(p[1] in (0.,96.*args.height_scale) for p in geometry['positions']),f'Vertical extent clipped in {name}'
                bounds[x,z]=faces
            print(f'Published row {z+1}/{args.tiles}',flush=True)
        generation_with_readonly_checks_ms=(time.perf_counter()-generation_start)*1000
        seams=[]
        for z in range(args.tiles):
            for x in range(args.tiles):
                if x+1<args.tiles:
                    a,b=bounds[x,z]['xhi'],bounds[x+1,z]['xlo'];assert a==b,('x',x,z,len(a^b));seams.append(dict(axis='x',block=[x,z],vertices=len(a)))
                if z+1<args.tiles:
                    a,b=bounds[x,z]['zhi'],bounds[x,z+1]['zlo'];assert a==b,('z',x,z,len(a^b));seams.append(dict(axis='z',block=[x,z],vertices=len(a)))
        points=[[0,(-48+k*.25)*args.height_scale,0] for k in range(385)]
        samples=c.call('query_volume_terrain',query=dict(kind='samples',document=doc,points=points))['samples']
        top=max(p['position'][1] for p in samples if p['density']>0)
        player['transform']['position']=[0,top+3*vstep,0]
        player['character_controller']=dict(max_slope_degrees=45,slide_slope_degrees=50,offset=.01,snap_distance=.2)
        c.call('edit_scene',edits=[dict(op='upsert',entity=player)])
        c.call('set_execution_mode',mode='continuous');c.call('step',ticks=600)
        assert c.call('inspect_character_controllers')['controllers']['player']['grounded']
        c.call('set_light',light=dict(direction=[.5,-.78,.36],ambient=.22,direct_intensity=1.25,direct_color=[1,.96,.88]))
        views=dict(overview=dict(position=[extent*.82,extent*.54,-extent*.89],target=[0,0,0],fov_degrees=50),
            ledges=dict(position=[1400,top+600,-1900],target=[0,top-40,0],fov_degrees=55),
            reverse=dict(position=[-extent*.8,extent*.5,extent*.8],target=[0,0,0],fov_degrees=52))
        c.call('set_camera',camera=views['overview'])
        _,save_ms=timed(c.call,'save_scene',path=f'{PREFIX}/world.scene.json')
        c.call('load_scene',path=f'{PREFIX}/world.scene.json');captures=[]
        for name,camera in views.items():
            c.call('set_camera',camera=camera)
            _,ms=timed(c.call,'capture',path=f'{PREFIX}/{name}.png',width=1600,height=1000,clip=dict(near=20 if name=='ledges' else 100,far=30000,reverse_z=False));captures.append(dict(view=name,mcp_ms=ms))
        c.call('load_scene',path=f'{PREFIX}/world.scene.json');c.call('capture',path=f'{PREFIX}/reload.png',width=1600,height=1000,clip=dict(near=100,far=30000,reverse_z=False))
        assert (OUT/'reload.png').read_bytes()==(OUT/'overview.png').read_bytes()
        before=c.call('inspect')
        c.call('capture',path=f'{PREFIX}/invalid-clip.png',width=640,height=400,clip=dict(near=100,far=50,reverse_z=False),expected_error=True)
        assert c.call('inspect')['revision']==before['revision']
        c.call('capture',path=f'{PREFIX}/clip-default.png',width=640,height=400)
        c.call('capture',path=f'{PREFIX}/clip-equivalent.png',width=640,height=400,clip=dict(near=.05,far=30000,reverse_z=False))
        assert (OUT/'clip-default.png').read_bytes()==(OUT/'clip-equivalent.png').read_bytes()
        assert c.call('inspect')['scene']['camera']==before['scene']['camera']
        legacy=c.call('load_volume_terrain',path='artifacts/volumetric-terrain/source.ignis-volume')['document']
        assert legacy['version']==1 and 'noise_domain_scale' not in legacy
        c.call('save_volume_terrain',path=f'{PREFIX}/legacy-roundtrip.ignis-volume',document=legacy)
        assert (OUT/'legacy-roundtrip.ignis-volume').read_bytes()==(ROOT/'artifacts/volumetric-terrain/source.ignis-volume').read_bytes()
        report=dict(area_meters=[extent,extent],area_vs_previous=(extent/64)**2,horizontal_spacing_meters=spacing,vertical_spacing_meters=vstep,horizontal_feature_stretch=extent/208,vertical_feature_stretch=args.height_scale,blocks=blocks,
            vertices=sum(b['terrain']['vertices'] for b in blocks),triangles=sum(b['terrain']['triangles'] for b in blocks),
            surface_height_range=[min(b['surface_height_range'][0] for b in blocks),max(b['surface_height_range'][1] for b in blocks)],
            clip_default_equivalent_pixels=True,invalid_clip_rejected=True,camera_unchanged_by_capture=True,legacy_source_bytes_preserved=True,
            shared_boundary_vertex_sets_equal=True,seams=seams,vertical_extent_unclipped=True,grounded=True,scene_reload_pixels_equal=True,
            original_patch_benchmark=bench,original_patch_warm_median_mcp_ms=statistics.median(b["mcp_ms"] for b in bench[1:]),publication_mcp_total_ms=publication_mcp_ms,
            generation_including_readonly_mesh_checks_ms=generation_with_readonly_checks_ms,
            save_scene_mcp_ms=save_ms,captures=captures,fully_resident=True,automatic_streaming=False,native_window=False,human_acceptance=False)
        (OUT/'verification.json').write_text(json.dumps(report,indent=2)+'\n')
        (ROOT/'examples/volume-terrain-landscape.recipe.json').write_text(json.dumps(dict(document=doc,blocks=[dict(id=b['id'],grid=b['grid']) for b in blocks],views=views,capture_clip_overview=dict(near=100,far=30000,reverse_z=False),capture_clip_ledges=dict(near=20,far=30000,reverse_z=False)),indent=2)+'\n')
        print(json.dumps({k:v for k,v in report.items() if k not in ('blocks','seams')}))
    finally:c.close()
if __name__=='__main__':main()
