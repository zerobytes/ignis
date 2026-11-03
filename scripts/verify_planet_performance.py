#!/usr/bin/env python3
"""MCP-only warm pan, submission grouping and source/physics identity acceptance."""
import argparse
import json
from pathlib import Path
from verify_lab import ROOT
from verify_planet_zoom import setup, SCENE, CONFIG
from png_pixels import rgba8

OUT=ROOT/'artifacts/planet-performance'

def verify(binary):
    OUT.mkdir(parents=True,exist_ok=True)
    if not (ROOT/SCENE).is_file() or not (ROOT/CONFIG).is_file():
        from open_planet_walk import prepare
        prepare(binary)
    c,_=setup(binary)
    try:
        scene=c.call('inspect')['scene']
        physics=c.call('inspect_planet_runtime')
        c.call('set_planet_view',view={'render_group_size':8})
        def capture(name):
            path=f'artifacts/planet-performance/{name}.png'
            result=c.call('capture',path=path,width=1280,height=800)
            return result,rgba8((ROOT/path).read_bytes())[2]
        initial,pixels=capture('warm-start')
        state=c.call('inspect_planet_view')
        warm=[]
        for index,delta in enumerate([0.1,0.1,-0.1,-0.1]):
            c.call('look_planet_view',yaw_delta_degrees=delta,tilt_delta_degrees=0)
            result,image=capture(f'warm-pan-{index}')
            current=c.call('inspect_planet_view')
            for key in ['geometry_builds','group_builds','residence_plans']:
                assert current[key]==state[key],f'warm look rebuilt {key}'
            assert result['adapter']['instancing']['geometry_uploads']==initial['adapter']['instancing']['geometry_uploads'],'warm look uploaded geometry'
            warm.append({'yaw':current['settings']['yaw_degrees'],'view':current,'adapter':result['adapter']})
        assert image==pixels,'roundtrip pan did not restore pixels'
        grouped=[]
        for index,yaw in enumerate([0,20,60,90,180,270]):
            c.call('set_planet_view',view={'yaw_degrees':yaw,'render_group_size':1,'atmosphere_optimization':False})
            one,reference=capture(f'pan-{index}-individual')
            before=c.call('inspect_planet_view')
            c.call('set_planet_view',view={'yaw_degrees':yaw,'render_group_size':8,'atmosphere_optimization':False})
            eight,candidate=capture(f'pan-{index}-grouped')
            after=c.call('inspect_planet_view')
            assert candidate==reference,f'grouping changed pixels at yaw{yaw}'
            assert after['geometry_builds']==before['geometry_builds'],'grouping resampled source'
            grouped.append({'yaw':yaw,'individual':one['adapter'],'grouped':eight['adapter'],'view':after})
        assert c.call('inspect')['scene']==scene
        assert c.call('inspect_planet_runtime')==physics
        report={'method':'native MCP captures, no window; warm +/-0.2deg pan; six individual/grouped yaw pairs; source and physics unchanged','warm':warm,'grouping':grouped}
        (OUT/'mcp.json').write_text(json.dumps(report,indent=2))
        print(json.dumps({'passed':True,'warm_pan_frames':len(warm),'grouping_poses':len(grouped)}))
    finally:
        c.close()

if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis')
    verify(parser.parse_args().binary)
