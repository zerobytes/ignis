"""Validate physical captures, paired controls, unchanged appearance, and oracle limits."""
import json, math, struct, zlib
from pathlib import Path
ROOT=Path('artifacts/appearance-probe/regional-efficiency')
d=json.loads((ROOT/'report.json').read_text())
assert len(d['rows'])==52 and len(d['modes'])==6
pngs=0;timed=0;stale=[]
for r in d['rows']:
    assert r['size']==[[3456,2234],[1728,1117]][r['resolution']]
    for key in ['grouped_full_vs_full','optimized_vs_old']:
        assert r['quality'][key]['foreground_rmse']==0
        assert r['quality'][key]['id_mismatch_pixels']==0
    assert r['counts'][1]['triangles']==r['counts'][2]['triangles']==r['counts'][4]['triangles']
    assert r['counts'][0]['triangles']==r['counts'][3]['triangles']
    assert r['counts'][3]['draw_calls']==4
    assert r['counts'][4]['draw_calls']<r['counts'][1]['draw_calls']
    assert r['counts'][4]['max_summary_bound']<1
    keys=['full','optimized','map']
    if r['oracle']:
        timed+=1;keys+=['oracle'];o=r['oracle']
        assert o['error_vs_full']['foreground_rmse']==0 and o['error_vs_full']['id_mismatch_pixels']==0
        assert o['diagnostic_error']['foreground_rmse']==0
        assert 0<o['visible_source_regions']<r['counts'][0]['geometry_regions']
        assert 0<o['counts']['triangles']<r['counts'][0]['triangles']
        assert o['discovery_cpu_gpu_readback_scan_ms']>0
        assert len(r['timings'])==12
        for i,t in enumerate(r['timings']):
            assert t['mode']==d['modes'][i%6]
            for k in ['gpu_ms','serial_ms','throughput_ms','planning_us']:
                assert t[k]['median']>0 and math.isfinite(t[k]['median'])
        if o['stale_previous_sample_error']:stale.append(o['stale_previous_sample_error']['foreground_rmse'])
    for key in keys:
        b=(ROOT/f"{r['id']}_{key}.png").read_bytes()
        assert b[:8]==b'\x89PNG\r\n\x1a\n' and list(struct.unpack('>II',b[16:24]))==r['size']
        offset=8;compressed=bytearray();ended=False
        while offset<len(b):
            n=struct.unpack('>I',b[offset:offset+4])[0];tag=b[offset+4:offset+8];payload=b[offset+8:offset+8+n]
            assert zlib.crc32(tag+payload)==struct.unpack('>I',b[offset+8+n:offset+12+n])[0]
            if tag==b'IDAT':compressed.extend(payload)
            offset+=n+12
            if tag==b'IEND':ended=True;break
        assert ended and offset==len(b) and b[24]==8
        channels={2:3,6:4}[b[25]]
        assert len(zlib.decompress(compressed))==(r['size'][0]*channels+1)*r['size'][1]
        pngs+=1
assert timed==20 and max(stale)>.1
for res in [0,1]:
    rows=[r for r in d['rows'] if r['resolution']==res]
    assert [r['step'] for r in rows]==list(range(13))+list(reversed(range(13)))
    for a,b in zip(rows, reversed(rows)):
        assert a['eye']==b['eye']
result={'poses':52,'timed_poses':timed,'pngs':pngs,'unchanged_rgb_and_ids':True,'oracle_excludes_discovery_cost':True,'stale_mask_fails':True,'native_browser':False}
(ROOT/'verification.json').write_text(json.dumps(result,indent=2));print(json.dumps(result))
