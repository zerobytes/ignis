#!/usr/bin/env python3
"""Render and inspect the four native sources over MCP; stable output paths."""
import argparse,copy,json,time,math,itertools
from pathlib import Path
from verify_lab import Client,ROOT

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');parser.add_argument('--only',choices=['palmeirinha','platano','eucalipto','figueira']);args=parser.parse_args()
    out=ROOT/'artifacts/botany-native';out.mkdir(parents=True,exist_ok=True)
    c=Client(args.binary,log_path=out/'runtime.log');reports={}
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'botany-native-demo','version':'1'}});c.request('notifications/initialized',notification=True)
        c.call('generate',recipe={'width':40,'depth':40,'seed':191,'obstacles':0})
        initial=c.call('inspect');reports['build_id']=initial['build_id'];entities=initial['scene']['entities'];c.call('edit_scene',edits=[{'op':'remove','id':e['id']} for e in entities])
        ground={'id':'ground','geometry':{'positions':[[-80,-.025,-80],[80,-.025,-80],[-80,-.025,80],[80,-.025,80]],'triangles':[[0,2,1],[1,2,3]]}}
        instance={'id':'ground','asset':'ground','position':[0,0,0],'rotation':[0,0,0,1],'scale':1,'color':[.24,.27,.21]}
        c.call('edit_scene',edits=[{'op':'mesh','edit':{'kind':'put_asset','asset':ground}},{'op':'mesh','edit':{'kind':'put_instance','instance':instance}}])
        c.call('set_light',light={'direction':[.5,-.8,-.3],'ambient':.27,'direct_intensity':1.2,'environment':{'sky':[.5,.6,.7],'horizon':[.5,.5,.4],'ground':[.1,.14,.06],'axis':[0,1,0],'intensity':.5},'shadow':{'center':[0,2,0],'extent':16,'resolution':2048,'bias':.0005}})
        c.call('save_scene',path='artifacts/botany-native/empty.scene.json')
        recipes=[]
        for id in ([args.only] if args.only else ['palmeirinha','platano','eucalipto','figueira']):
            c.call('load_scene',path='artifacts/botany-native/empty.scene.json')
            start=time.monotonic();result=c.call('botany',operation={'kind':'load','path':f'assets/botany/{id}.ignis-botany','publish':True});reports[id]={'publish_seconds':time.monotonic()-start,**result}
            lo,hi=result['compiled']['bounds_min'],result['compiled']['bounds_max']
            center=[(a+b)/2 for a,b in zip(lo,hi)]
            import botany_math as bm
            direction=bm.unit((.53,.16,.83));right=bm.unit(bm.cross(bm.mul(direction,-1),bm.UP));up=bm.cross(right,bm.mul(direction,-1));tan=math.tan(math.radians(35/2))
            distance=max(bm.dot(bm.sub(corner,center),direction)+1.10*max(abs(bm.dot(bm.sub(corner,center),right))/(tan*1200/1100),abs(bm.dot(bm.sub(corner,center),up))/tan) for corner in itertools.product(*zip(lo,hi)))
            c.call('set_camera',camera={'position':bm.add(center,bm.mul(direction,distance)),'target':center,'fov_degrees':35})
            c.call('capture',path=f'artifacts/botany-native/{id}.png',width=1200,height=1100)
            r=json.loads((ROOT/f'assets/botany/{id}.ignis-botany').read_text())
            bare=copy.deepcopy(r);bare['settings']['leaf_density']=0.
            path=f'artifacts/botany-native/{id}-structure.ignis-botany';(ROOT/path).write_text(json.dumps(bare,separators=(',',':')))
            c.call('botany',operation={'kind':'load','path':path,'publish':True})
            c.call('capture',path=f'artifacts/botany-native/{id}-structure.png',width=1200,height=1100)
            r['settings']['position']=[{'palmeirinha':-11,'platano':-5,'eucalipto':3,'figueira':11}[id],0,0];recipes.append(r)
            print(id,result['compiled'],flush=True)
        if args.only:
            (out/f'{args.only}-report.json').write_text(json.dumps(reports,indent=2));return
        c.call('load_scene',path='artifacts/botany-native/empty.scene.json')
        # Files avoid the generic MCP input envelope for the largest source.
        for r in recipes:
            path=f"artifacts/botany-native/{r['prefix']}.ignis-botany";(ROOT/path).write_text(json.dumps(r,separators=(',',':')))
            c.call('botany',operation={'kind':'load','path':path,'publish':True})
        c.call('set_camera',camera={'position':[1,9.5,27],'target':[1,4.2,0],'fov_degrees':43})
        c.call('capture',path='artifacts/botany-native/gallery.png',width=1920,height=1000)
        c.call('save_scene',path='artifacts/botany-native/gallery.scene.json')
        c.call('load_scene',path='artifacts/botany-native/gallery.scene.json')
        c.call('capture',path='artifacts/botany-native/roundtrip.png',width=1920,height=1000)
        assert (out/'gallery.png').read_bytes()==(out/'roundtrip.png').read_bytes()
        reports['scene_roundtrip_pixels_equal']=True
        c.call('botany',operation={'kind':'open','open':True,'prefix':'figueira'})
        c.call('capture_editor',path='artifacts/botany-native/editor.png',width=1600,height=1000)
        r=json.loads((ROOT/'assets/botany/platano.ignis-botany').read_text());leaf=copy.deepcopy(next(o for o in r['source']['organs'] if o['shape']['kind']=='blade'));origin=leaf['path'][0]
        leaf['path']=[[a-b for a,b in zip(p,origin)]for p in leaf['path']];leaf['parent']=None;leaf['insertion']=0
        r['prefix']='leaf_shape';r['source']['organs']=[leaf]
        path='artifacts/botany-native/leaf-shape.ignis-botany';(ROOT/path).write_text(json.dumps(r))
        c.call('load_scene',path='artifacts/botany-native/empty.scene.json');c.call('botany',operation={'kind':'load','path':path,'publish':True})
        import botany_math as mathlab
        t=mathlab.unit(mathlab.sub(leaf['path'][-1],leaf['path'][0]));n=mathlab.normal_to(t,leaf['normal']);normal=mathlab.cross(t,n)
        if normal[1]<0:normal=mathlab.mul(normal,-1)
        center=leaf['path'][len(leaf['path'])//2];eye=mathlab.add(mathlab.add(center,mathlab.mul(normal,.8)),(.08,.04,.12))
        c.call('set_camera',camera={'position':eye,'target':center,'fov_degrees':35});c.call('capture',path='artifacts/botany-native/platano-folha.png',width=1000,height=850)
        (out/'report.json').write_text(json.dumps(reports,indent=2))
    finally:c.close()
if __name__=='__main__':main()
