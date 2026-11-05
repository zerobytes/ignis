"""Validate recorded selection, full-frame image dimensions, cache sharing and measurements."""
import json,math,struct
from pathlib import Path
ROOT=Path('artifacts/appearance-probe/regional-scene')

def png_size(p):
    with p.open('rb') as f:b=f.read(24)
    assert b[:8]==b'\x89PNG\r\n\x1a\n';return struct.unpack('>II',b[16:24])

def run():
    d=json.loads((ROOT/'report.json').read_text());assert len(d['objects'])==96 and len(d['rows'])==52
    assert [sum(o['asset']==i for o in d['objects']) for i in range(3)]==[32]*3
    assert len(d['checks'])==2 and all(c['repeat_identical'] and c['reverse_draw_error']['foreground_rmse']<.0001 for c in d['checks'])
    assert all(a['cache_bytes']==a['regions']*6*341*32 for a in d['assets'])
    assert 0<=d['cache_allocated_bytes']-sum(a['cache_bytes'] for a in d['assets'])<1024*32
    mixes=[];max_error=0.;checks=0
    for res in [0,1]:
        rows=[r for r in d['rows'] if r['resolution']==res]
        assert len(rows)==26 and rows[0]['eye']==rows[-1]['eye']
        assert rows[12]['eye']==rows[13]['eye']
        assert rows[12]['adaptive']['triangles']==rows[13]['adaptive']['triangles']
        for r in rows:
            a=r['adaptive'];f=r['full'];assert a['max_summary_bound']<1 and a['triangles']<f['triangles']
            assert a['summary_regions']==sum(o['summary_regions'] for o in a['per_object'])
            assert a['mixed_objects']==sum(o['summary_regions']>0 and o['geometry_regions']>0 for o in a['per_object'])
            visible=[o for o in r['visible_regions'] if o['geometry_pixels'] and o['summary_pixels']];assert visible
            mixes.append(len(visible));max_error=max(max_error,r['error_vs_full']['foreground_rmse'])
            assert r['error_vs_full']['foreground_pixels']>0
            # Detect regressions of the large specular spikes observed before the glossy fallback.
            assert r['error_vs_full']['max_rgb_error_length']<5
            for key in ['foreground_rmse','frame_rmse','max_rgb_error_length']:assert math.isfinite(r['error_vs_full'][key])
            for suffix in ['full','adaptive','map','error']:
                assert png_size(ROOT/f'{r["id"]}_{suffix}.png')==tuple(r['size']);checks+=1
            if r['timings']:
                assert r['reference_full_error'] is not None
                assert png_size(ROOT/f'{r["id"]}_reference.png')==tuple(r['size']);checks+=1
                for t in r['timings']:
                    assert t['gpu_ms']['samples']==9 and t['serial_ms']['samples']==5
                    assert t['gpu_ms']['median']>0 and t['serial_ms']['median']>0
            assert r['cpu_planning']['adaptive_us']['samples']==9
    for a,b in zip(d['rows'][:26],d['rows'][26:]):
        assert a['eye']==b['eye'] and a['step']==b['step']
        assert a['full']['triangles']==b['full']['triangles']
        # Resolution-specific hysteresis cannot turn the higher-resolution eligibility into more summaries.
        assert a['adaptive']['summary_regions']<=b['adaptive']['summary_regions']
    result={'poses':52,'pngs':checks,'visible_mixed_objects_range':[min(mixes),max(mixes)],'max_foreground_rmse':max_error,'repeat_and_depth_order':True,'cache_shared_for_96_objects':True,'native_browser':False}
    (ROOT/'verification.json').write_text(json.dumps(result,indent=2));print(json.dumps(result,indent=2))
if __name__=='__main__':run()
