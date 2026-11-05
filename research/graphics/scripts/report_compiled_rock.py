"""Offline evidence/report. JSON belongs to diagnostics, never to the asset decoder."""
import json
import statistics
from pathlib import Path
R=Path('artifacts/appearance-probe/compiled-rock')
def read(n): return json.loads((R/f'{n}.json').read_text())
build,quality,timings,integrated,load,checks=map(read,['build','quality','timings','integrated','load','checks'])
summary={'build':build,'load':load,'poses':len(quality),'gates':[sum(r['checks'][i]['pass'] for r in quality) for i in range(4)],'near_native':[],'paired_ratios':{},'timings':[]}
for di in range(3):
    rows=[r for r in timings if r['resolution']==0 and r['distance_id']==di and r['frame']==0]
    summary['near_native'].append({'distance':[4,24,96][di],'total_ms':{str(m):statistics.mean(s['total_ms']['median'] for r in rows for s in r['samples'] if s['method']==m) for m in [0,7,8,9,10]}})
for m in [7,8,9,10]:
    ratios=[]
    for r in timings:
        for control in [0,1]:
            a=next(s for s in r['samples'] if s['method']==0 and s['control']==control)
            b=next(s for s in r['samples'] if s['method']==m and s['control']==control)
            ratios.append(b['total_ms']['median']/a['total_ms']['median'])
    summary['paired_ratios'][str(m)]={'min':min(ratios),'median':statistics.median(ratios),'max':max(ratios),'ratios':ratios}
for r in timings:
    summary['timings'].append({k:v for k,v in r.items() if k!='samples'}|{'methods':{str(m):{metric:statistics.mean(s[metric]['median'] for s in r['samples'] if s['method']==m) for metric in ['total_ms','gpu_ms','cpu_ms']} for m in [0,7,8,9,10]}})
(R/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
data={'build':build,'rows':quality,'timings':summary['timings'],'integrated':integrated,'load':load,'summary':summary}
html=Path('scripts/compiled_rock_report.html').read_text().replace('__DATA__',json.dumps(data,separators=(',',':')))
(R/'index.html').write_text(html)
print(json.dumps({k:v for k,v in summary.items() if k not in ['timings','paired_ratios']},indent=2))
