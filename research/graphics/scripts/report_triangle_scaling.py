"""Density curves: paired mode controls, quality gates, source growth and resident memory."""
import json
import statistics as st
from pathlib import Path
ROOT=Path('artifacts/appearance-probe/triangle-scaling')
KEYS=['gpu_ms','serial_ms','throughput_ms','cpu_ms']

def pair(row,mode,key):
    return st.mean(row['timings'][i][key]['median'] for i in [mode,mode+len(row['timings'])//2])

def apply_confirmation(data):
    count=len(data['modes'])
    confirmation=json.loads((ROOT/'confirmation.json').read_text())
    for c in confirmation:
        for r in data['rows']:
            if (r['family'],r['resolution'],r['frame'])==(c['family'],c['resolution'],c['frame']):
                d=r['density'];r['timings']=[c['timings'][d*count+i] for i in range(count)]+[c['timings'][4*count+d*count+i] for i in range(count)]
                r['batch_frames']=128
    data['confirmation']=confirmation

def summarize(data):
    out=[]
    for family in range(2):
        for resolution in range(2):
            for frame in range(5):
                rows=sorted([r for r in data['rows'] if r['family']==family and r['resolution']==resolution and r['frame']==frame],key=lambda r:r['factor'])
                assert len(rows)==4
                modes=[]
                for m,name in enumerate(data['modes']):
                    curve={k:[pair(r,m,k) for r in rows] for k in KEYS}
                    modes.append({'mode':name,'curve_ms':curve,'growth_100_over_1':{k:v[-1]/v[0] for k,v in curve.items()},'curve_max_over_min':{k:max(v)/min(v) for k,v in curve.items()},'ratio_to_full_100':{k:pair(rows[-1],m,k)/pair(rows[-1],0,k) for k in KEYS},'quality_all_densities':m==0 or all(r['checks'][m-1]['quality_gate'] for r in rows),'duplicate_ratio_range':{k:[min(r['timings'][m+len(data['modes'])][k]['median']/r['timings'][m][k]['median'] for r in rows),max(r['timings'][m+len(data['modes'])][k]['median']/r['timings'][m][k]['median'] for r in rows)] for k in KEYS}})
                out.append({'family':family,'resolution':resolution,'frame':frame,'distance':rows[0]['distance'],'batch_frames':rows[0].get('batch_frames',8),'modes':modes})
    return out

def main():
    data=json.loads((ROOT/'report.json').read_text());assert len(data['rows'])==144
    apply_confirmation(data)
    data['summary']=summarize(data);(ROOT/'summary.json').write_text(json.dumps(data['summary'],indent=2))
    for r in data['rows']:
        if r['timings']:
            for t in r['timings']:
                for k in KEYS:t[k]={'median':t[k]['median']}
    data.pop('confirmation',None)
    html=Path(__file__).with_name('triangle_scaling_report.html').read_text()
    (ROOT/'index.html').write_text(html.replace('__DATA__',json.dumps(data,separators=(',',':'))))
    for s in data['summary']:
        print(s['family'],s['resolution'],round(s['distance']),[(m['mode'],round(m['growth_100_over_1']['throughput_ms'],2),m['quality_all_densities']) for m in s['modes']])

if __name__=='__main__':main()
