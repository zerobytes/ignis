"""Validate result completeness and compression artifacts in displayed PNGs (requires Pillow)."""
import json, math
from pathlib import Path
from PIL import Image, ImageChops
ROOT=Path('artifacts/appearance-probe/assets')

def run():
    result={}
    for name in ['rock','chest','tree']:
        root=ROOT/name; d=json.loads((root/'report.json').read_text()); assert len(d['cases'])==114
        assert d['checks']['bounded_pairs']==114 and d['checks']['repeat_wrap_light_return']
        assert d['timing_batch']==32 and d['timing_rounds']==9
        pixel_checks={mode:{'max_png_delta':0,'pixels_over_one_level':0,'longest_vertical_run_over_one_level':0} for mode in ['integrated','native']}
        for row in d['cases']:
            for key in ['compact_error','original_error','native_full_error','native_compact_error','packing_error']:
                assert all(math.isfinite(x) for x in row[key].values())
            assert row['packing_error']['rmse_surface']<.002 and row['packing_error']['alpha_rmse']<.002
            for method in ['reference','full','original','compact','native_full','native_original','native_compact','error','packing']:
                with Image.open(root/f'{row["id"]}_{method}.png') as im: assert im.size==(256,256)
            for mode,prefix in [('integrated',''),('native','native_')]:
                check=pixel_checks[mode]
                with Image.open(root/f'{row["id"]}_{prefix}original.png') as a, Image.open(root/f'{row["id"]}_{prefix}compact.png') as b:
                    diff=ImageChops.difference(a,b); p=diff.load()
                    check['max_png_delta']=max(check['max_png_delta'],max(v[1] for v in diff.getextrema()))
                    for x in range(256):
                        length=0
                        for y in range(256):
                            changed=max(p[x,y])>1
                            length=length+1 if changed else 0
                            check['pixels_over_one_level']+=changed
                            check['longest_vertical_run_over_one_level']=max(check['longest_vertical_run_over_one_level'],length)
            if row['timings']:
                for t in row['timings']:
                    assert t['gpu']['samples']==9
                    assert t['gpu']['median'] is None or t['gpu']['median']>0
        result[name]={'poses':len(d['cases']), 'pixel_checks':pixel_checks, 'max_linear_packing_rmse':max(r['packing_error']['rmse_surface'] for r in d['cases'])}
        assert all(c['longest_vertical_run_over_one_level']<=2 for c in pixel_checks.values()), (name,'introduced vertical run',pixel_checks)
    (ROOT/'verification.json').write_text(json.dumps(result,indent=2));print(json.dumps(result,indent=2))
if __name__=='__main__': run()
