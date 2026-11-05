"""Verify physical-pixel captures and state transitions; offline, not an MCP scenario."""
import json, math
from pathlib import Path
from PIL import Image, ImageChops
ROOT=Path('artifacts/appearance-probe/adaptive-assets')

def run():
    result={}
    for name in ['rock','chest','tree']:
        root=ROOT/name;d=json.loads((root/'report.json').read_text());rows=d['cases']
        assert len(rows)==84 and d['window']==512
        switches=[];counts={'geometry':0,'summary':0}
        for res in [0,1]:
            seq=[r for r in rows if r['resolution']==res];state=False
            for r in seq:
                bounds=[r['triangle_bound_px'],r['filter_bound_px']]
                assert all(math.isfinite(x) and x>0 for x in bounds)
                if max(bounds)>=1:state=False
                elif max(bounds)<.75:state=True
                assert r['summary']==state
                assert not state or max(bounds)<1
                counts['summary' if state else 'geometry']+=1
                if r['switched']:switches.append({k:r[k] for k in ['id','summary','pose','density']})
                assert r['density']['max_visible_edge_px']<=bounds[0]*1.001+.001
                assert 0<=r['density']['mean_ids_per_covered_pixel']<=16
                assert r['drawn_triangles']==(2 if state else d['triangles'])
                for metric in ['full_error','adaptive_error','forced_error','replacement_delta']:
                    assert all(math.isfinite(x) for x in r[metric].values())
                for suffix in ['reference','full','forced','adaptive','error']:
                    with Image.open(root/f'{r["id"]}_{suffix}.png') as im:assert im.size==(512,512)
                with Image.open(root/f'{r["id"]}_adaptive.png') as a,Image.open(root/f'{r["id"]}_{"forced" if state else "full"}.png') as b:
                    assert ImageChops.difference(a,b).getbbox() is None
                # Entire object must fit inside the central window; all boundary pixels are background.
                with Image.open(root/f'{r["id"]}_reference.png') as ref:
                    for box in [(0,0,512,1),(0,511,512,512),(0,0,1,512),(511,0,512,512)]:
                        assert len(ref.crop(box).getcolors(2) or [])==1
                if r['timings']:
                    assert len(r['timings'])==4
                    assert all(t['gpu']['samples'] in [0,9] for t in r['timings'])
            assert seq[0]['summary'] is False and seq[-1]['summary'] is False
            assert any(r['summary'] for r in seq)
            assert len([r for r in seq if r['switched']])==2
        hi=[r for r in rows if r['resolution']==0];lo=[r for r in rows if r['resolution']==1]
        for a,b in zip(hi,lo):
            assert a['pose']==b['pose'] and a['way']==b['way'] and a['frame']==b['frame']
            assert abs(a['triangle_bound_px']-2*b['triangle_bound_px'])<.0001
            assert abs(a['filter_bound_px']-2*b['filter_bound_px'])<.0001
            assert not a['summary'] or b['summary']
        result[name]={'poses':len(rows),'counts':counts,'switches':switches,'exact_adaptive_pngs':len(rows),'same_distances_resolution_scaling':True,'reference_max_16_to_64_rmse':max(c['16_to_64_spp']['rmse_surface'] for c in d['convergence'])}
    (ROOT/'verification.json').write_text(json.dumps(result,indent=2))
    print(json.dumps({n:{k:v for k,v in d.items() if k!='switches'} for n,d in result.items()},indent=2))
if __name__=='__main__':run()
