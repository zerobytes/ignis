#!/usr/bin/env python3
"""A larger continuous density-field experiment. Python schedules MCP, never meshes."""
import argparse,copy,json,statistics,time
from verify_lab import Client,ROOT
from open_earth_walk import init
from volume_terrain_demo import recipe
PREFIX='artifacts/volumetric-terrain-region'
OUT=ROOT/PREFIX

def timed(call,*args,**kwargs):
    t=time.perf_counter();result=call(*args,**kwargs)
    return result,(time.perf_counter()-t)*1000

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--tiles',type=int,default=4);parser.add_argument('--seed',type=int,default=2718);args=parser.parse_args()
    if not 1<=args.tiles<=8:raise ValueError('Use1..8 tiles per horizontal axis for this resident experiment')
    OUT.mkdir(parents=True,exist_ok=True);doc=recipe(args.seed)
    spacing=.5;size=52.;extent=args.tiles*size;half=extent/2
    c=Client(ROOT/'target/release/ignis',log_path=OUT/'native.log')
    try:
        init(c);bench=[]
        for i in range(4):
            result,ms=timed(c.call,'query_volume_terrain_gpu',document=doc,grid=dict(origin=[-32,-24,-32],cells=[128,96,128],spacing=.5))
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
                origin=[-half+x*size,-48.,-half+z*size];name=f'volume_{x}_{z}'
                grid=dict(origin=origin,cells=[104,192,104],spacing=spacing)
                result,ms=timed(c.call,'publish_volume_terrain_gpu',publication=dict(document=doc,grid=grid,asset=name,instance=name,position=origin,color=[.51,.48,.40],material=material,collision=True))
                assert not result['visual_only'];publication_mcp_ms+=ms
                blocks.append(dict(id=name,grid=grid,mcp_ms=ms,terrain=result['terrain']))
                # Read-only comparison of native output on shared block planes.
                geometry=c.call('inspect_mesh',asset=name)['asset']['geometry']
                faces={}
                for axis,label in [(0,'x'),(2,'z')]:
                    for value,suffix in [(0.,'lo'),(size,'hi')]:
                        faces[label+suffix]={tuple(float(v)+origin[i] for i,v in enumerate(p)) for p in geometry['positions'] if p[axis]==value}
                # A bounded cut must not clip the top/bottom of the intended terrain.
                assert not any(p[1] in (0.,96.) for p in geometry['positions']),f'Vertical extent clipped in {name}'
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
        points=[[0,-48+k*.25,0] for k in range(385)]
        samples=c.call('query_volume_terrain',query=dict(kind='samples',document=doc,points=points))['samples']
        top=max(p['position'][1] for p in samples if p['density']>0)
        player['transform']['position']=[0,top+2,0]
        player['character_controller']=dict(max_slope_degrees=45,slide_slope_degrees=50,offset=.01,snap_distance=.2)
        c.call('edit_scene',edits=[dict(op='upsert',entity=player)])
        c.call('set_execution_mode',mode='continuous');c.call('step',ticks=120)
        assert c.call('inspect_character_controllers')['controllers']['player']['grounded']
        c.call('set_light',light=dict(direction=[.5,-.78,.36],ambient=.22,direct_intensity=1.25,direct_color=[1,.96,.88]))
        views=dict(overview=dict(position=[extent*.82,extent*.54,-extent*.89],target=[0,0,0],fov_degrees=50),
            ledges=dict(position=[30,20,-42],target=[0,0,0],fov_degrees=55),
            reverse=dict(position=[-extent*.8,extent*.5,extent*.8],target=[0,0,0],fov_degrees=52))
        c.call('set_camera',camera=views['overview'])
        _,save_ms=timed(c.call,'save_scene',path=f'{PREFIX}/world.scene.json')
        c.call('load_scene',path=f'{PREFIX}/world.scene.json');captures=[]
        for name,camera in views.items():
            c.call('set_camera',camera=camera)
            _,ms=timed(c.call,'capture',path=f'{PREFIX}/{name}.png',width=1600,height=1000);captures.append(dict(view=name,mcp_ms=ms))
        c.call('load_scene',path=f'{PREFIX}/world.scene.json');c.call('capture',path=f'{PREFIX}/reload.png',width=1600,height=1000)
        assert (OUT/'reload.png').read_bytes()==(OUT/'overview.png').read_bytes()
        report=dict(area_meters=[extent,extent],area_vs_previous=(extent/64)**2,spacing_meters=spacing,blocks=blocks,
            vertices=sum(b['terrain']['vertices'] for b in blocks),triangles=sum(b['terrain']['triangles'] for b in blocks),
            shared_boundary_vertex_sets_equal=True,seams=seams,vertical_extent_unclipped=True,grounded=True,scene_reload_pixels_equal=True,
            original_patch_benchmark=bench,original_patch_warm_median_mcp_ms=statistics.median(b["mcp_ms"] for b in bench[1:]),publication_mcp_total_ms=publication_mcp_ms,
            generation_including_readonly_mesh_checks_ms=generation_with_readonly_checks_ms,
            save_scene_mcp_ms=save_ms,captures=captures,fully_resident=True,automatic_streaming=False,native_window=False,human_acceptance=False)
        (OUT/'verification.json').write_text(json.dumps(report,indent=2)+'\n')
        (ROOT/'examples/volume-terrain-region.recipe.json').write_text(json.dumps(dict(document=doc,blocks=[dict(id=b['id'],grid=b['grid']) for b in blocks],views=views),indent=2)+'\n')
        print(json.dumps({k:v for k,v in report.items() if k not in ('blocks','seams')}))
    finally:c.close()
if __name__=='__main__':main()
