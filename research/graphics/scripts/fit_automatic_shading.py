"""Fit only the prespecified calibration; validation data never read here."""
import hashlib,json,math,statistics as st
from pathlib import Path
ROOT=Path('artifacts/appearance-probe/automatic-shading')
raw=(ROOT/'calibration.json').read_bytes();cs=json.loads(raw)
assert len(cs)==24 and {c['distance'] for c in cs}=={12,16,24,48,96,192}
def pair(c,d,m):
    n=len(c['codes']);i=d*n+c['codes'].index(m);h=len(c['timings'])//2
    return st.mean(c['timings'][j]['throughput_ms']['median'] for j in [i,i+h])
x=[];y=[];base=[];groups=[]
for g,c in enumerate(cs):
    for d in range(4):
        x.append(c['features'][d]);y.append(pair(c,d,5)-pair(c,d,85));base.append(pair(c,d,5));groups.append(g)
def fit(xs,ys):
    scales=[math.sqrt(st.mean(r[i]**2 for r in xs)) or 1 for i in range(6)]
    a=[[v/scales[i] for i,v in enumerate(r)] for r in xs]
    # Six-variable normal equations, scaled ridge; deterministic pivoted solve.
    mat=[[sum(r[i]*r[j] for r in a)+(0.001*len(a) if i==j and i else 0) for j in range(6)]+[sum(r[i]*v for r,v in zip(a,ys))] for i in range(6)]
    for i in range(6):
        k=max(range(i,6),key=lambda k:abs(mat[k][i]));mat[i],mat[k]=mat[k],mat[i]
        v=mat[i][i];assert abs(v)>1e-12;mat[i]=[z/v for z in mat[i]]
        for j in range(6):
            if i!=j:
                v=mat[j][i];mat[j]=[a-v*b for a,b in zip(mat[j],mat[i])]
    return [mat[i][-1]/scales[i] for i in range(6)]
def dot(c,r):return sum(a*b for a,b in zip(c,r))
errors=[]
for g in range(len(cs)):
    coef=fit([r for r,h in zip(x,groups) if h!=g],[v for v,h in zip(y,groups) if h!=g])
    errors.extend(dot(coef,r)-v for r,v,h in zip(x,y,groups) if h==g)
margin=max(0,sorted(errors)[math.ceil(.9*len(errors))-1])+.005
model={'benefit':fit(x,y),'baseline':fit(x,base),'margin_ms':margin,'max_work':[max(r[i] for r in x) for i in range(6)],'screen_range':[min(r[1] for r in x),max(r[1] for r in x)],'adapter':cs[0]['adapter']}
text=json.dumps(model,indent=2)+'\n';(ROOT/'model.json').write_text(text)
audit={'calibration_sha256':hashlib.sha256(raw).hexdigest(),'model_sha256':hashlib.sha256(text.encode()).hexdigest(),'training_distances':[12,16,24,48,96,192],'points':len(x),'ridge':.001,'margin_method':'p90 positive overprediction in leave-one-cohort-out + 0.005ms; also require5% predicted baseline','margin_ms':margin,'heldout_cohort_errors_ms':errors,'features':['constant','screenMP','selectedTrianglesM','sqrt(trianglesM*projectedMP)','projectedMP','nativeNormalMP'],'model':model}
(ROOT/'fit.json').write_text(json.dumps(audit,indent=2));print(json.dumps(model,indent=2))
