"""Summarize paired controls without selecting winners or refitting a policy."""
import json,statistics as st
from pathlib import Path
ROOT=Path('artifacts/appearance-probe/scaling-ablation')
def pair(c,d,code,key='throughput_ms'):
    n=len(c['codes']);i=d*n+c['codes'].index(code);h=len(c['timings'])//2
    return st.mean(c['timings'][j][key]['median'] for j in (i,i+h))
def summarize(name):
    rows=[]
    for c in json.loads((ROOT/name).read_text()):
        for d in range(4):
            rows.append({k:c[k] for k in ['family','resolution','distance']}|{'density':d,'factor':[1,2,10,100][d],'costs':{key:{str(code):pair(c,d,code,key) for code in c['codes']}for key in ['throughput_ms','gpu_ms','cpu_ms']}})
    return rows
if __name__=='__main__':
    out={}
    for name in ['diagnostic.json','order.json','confirmation.json','recheck.json','depth.json','depth-recheck.json']:
        if (ROOT/name).exists():out[name]=summarize(name)
    for name,rows in out.items():
        print(name,len(rows),'points')
        for r in rows:
            if r['family']==1 and r['resolution']==0 and r['distance'] in [14,224]:print(r['distance'],r['factor'],{k:round(v,5)for k,v in r['costs']['throughput_ms'].items()})
    (ROOT/'analysis.json').write_text(json.dumps(out,indent=2))
