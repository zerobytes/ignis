#!/usr/bin/env python3
"""Critical MCP boundary: source versions and regional atomic publication."""
import argparse,copy,json
from pathlib import Path
from verify_lab import Client,ROOT
from verify_terrain import position

def verify(binary,window=False):
    c=Client(binary,window);checks=[]
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'terrain-region','version':'1'}});c.request('notifications/initialized',notification=True)
        before=c.call('inspect')
        old=c.call('load_terrain',path='examples/terrain-tools.ignis-terrain')['document']
        saved=c.call('save_terrain',path='artifacts/region-v1.ignis-terrain',document=old)
        assert saved['format']==1
        assert (ROOT/'artifacts/region-v1.ignis-terrain').read_bytes()==(ROOT/'examples/terrain-tools.ignis-terrain').read_bytes()
        doc=json.loads((ROOT/'examples/representative-world.recipe.json').read_text())['document']
        saved=c.call('save_terrain',path='artifacts/region-v2.ignis-terrain',document=doc)
        assert saved['format']==2 and c.call('load_terrain',path='artifacts/region-v2.ignis-terrain')['document']==doc
        region={'origin':position(-16,-16),'cells':[8,8],'spacing_meters':4}
        report=c.call('query_terrain',query={'kind':'inspect_region','document':doc,'region':region})
        assert report['vertices']==81 and report['triangles']==128 and report['shared_grid_vertices']
        bad=copy.deepcopy(doc);bad['version']=1
        c.call('query_terrain',query={'kind':'inspect','document':bad},expected_error=True)
        assert c.call('inspect')==before
        checks.append('Version1 is re-encoded bit-exact with truthful metadata; version2 roundtrip and regional query are pure, downgrade of ridge source rejected')
        flat=copy.deepcopy(doc);flat['operations']=[];flat['base']={'kind':'constant','height_meters':0}
        pub={'document':flat,'region':{'origin':position(-64,-64),'cells':[128,128],'spacing_meters':1},'frame_origin':position(0,0),'asset':'regional_ground','instance':'','color':[.3,.5,.2],'collision':True}
        c.call('publish_terrain_region',publication=pub,expected_error=True)
        assert c.call('inspect')==before
        pub['instance']='regional_ground'
        out=c.call('publish_terrain_region',publication=pub)
        assert not out['visual_only'] and out['terrain']['vertices']==16641
        assert c.call('inspect_mesh_collisions')['instances']==['regional_ground']
        c.call('undo');assert c.call('inspect')['scene']==before['scene']
        c.call('redo');assert c.call('inspect_mesh_collisions')['instances']==['regional_ground']
        c.call('save_scene',path='artifacts/region.scene.json')
        c.call('step',ticks=2)
        c.call('save_replay',path='artifacts/region.replay.json')
        assert c.call('run_replay',path='artifacts/region.replay.json')['matched']
        c.call('load_scene',path='artifacts/region.scene.json')
        assert c.call('inspect_mesh_collisions')['instances']==['regional_ground']
        checks.append('Publication beyond modeling budget rejects late failure atomically and preserves collision/source through one undo, redo, scene and replay')
        report={'passed':True,'checks':checks,'build_id':c.call('inspect')['build_id'],'native_window':window}
        (ROOT/'artifacts/terrain-region-verification.json').write_text(json.dumps(report,indent=2)+'\n')
        print(json.dumps({'passed':True,'checks':len(checks),'build_id':report['build_id']}))
    finally:c.close()

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');p.add_argument('--window',action='store_true');a=p.parse_args();verify(a.binary.resolve(),a.window)
