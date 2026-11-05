"""Summarize complete frames and representation errors without quality-based winner selection."""
import json
import statistics
from pathlib import Path
root=Path('artifacts/appearance-probe/rock-symbol')
read=lambda name:json.loads((root/f'{name}.json').read_text())
raw=read('timings')+read('repeat')
summary=[]
for res in range(2):
 for distance in range(3):
  for frame in [0,3,6]:
   cs=[c for c in raw if c['resolution']==res and c['distance_id']==distance and c['frame']==frame]
   methods=[]
   for m in range(7):
    ss=[s for c in cs for s in c['samples'] if s['method']==m]
    methods.append({k:{'mean':statistics.mean(s[k]['median'] for s in ss),'min':min(s[k]['median'] for s in ss),'max':max(s[k]['median'] for s in ss)} for k in ['total_ms','gpu_ms','cpu_ms']})
   summary.append({'resolution':res,'distance_id':distance,'frame':frame,'pose':cs[0]['pose'],'methods':methods,'median_count':len(cs)*2})
rows=read('quality');b=read('build')
agg=[]
for res in range(2):
 for d in range(3):
  qs=[r for r in rows if r['resolution']==res and r['distance_id']==d]
  a={'resolution':res,'distance_id':d,'methods':[]}
  for m in range(6):
   errors=[r['checks'][m] for r in qs]
   a['methods'].append({'passed':sum(q['pass'] for q in errors),'poses':len(errors),**{k:statistics.median(q[k] for q in errors) for k in ['rmse','p99','coverage_fraction']}})
  agg.append(a)
data={'rows':rows,'build':b,'timings':summary,'quality_summary':agg,'integrated':read('integrated'),'verification':read('verification')}
(root/'summary.json').write_text(json.dumps(data,indent=2)+'\n')
(root/'index.html').write_text(Path('scripts/rock_symbol_report.html').read_text().replace('__DATA__',json.dumps(data,separators=(',',':'))))
for s in summary:
 if s['frame']==0:print('res',s['resolution'],'distance',[4,24,96][s['distance_id']],'total',[round(m['total_ms']['mean'],4) for m in s['methods']],'GPU',[round(m['gpu_ms']['mean'],4) for m in s['methods']])
for a in agg:print('quality',a)
