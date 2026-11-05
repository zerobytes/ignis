"""Constrained ridge fit on calibration poses only; no validation/report reads."""
import hashlib,json,math,statistics as st
from pathlib import Path
ROOT=Path('artifacts/appearance-probe/cost-scaling')
raw=(ROOT/'calibration.json').read_bytes();cohorts=json.loads(raw);assert len(cohorts)==16
assert {c['distance']for c in cohorts}=={18.,40.,80.,160.}
x=[];y=[];weights=[]
for c in cohorts:
    assert len(c['timings'])==16
    for d in range(4):
        x.append(c['features'][d]);pairs=[];costs=[]
        for dup in [0,8]:
            r=c['timings'][d*2+dup]['samples_ms'];g=c['timings'][d*2+1+dup]['samples_ms'];assert len(r)==len(g)==8
            pairs.extend(b-a for a,b in zip(r,g));costs.extend(r+g)
        y.append(st.median(pairs));weights.append(1/max(.05,st.median(costs)))
original_y=y[:]
x=[[v*w for v in row]for row,w in zip(x,weights)];y=[v*w for v,w in zip(y,weights)]
scale=[math.sqrt(st.mean(row[i]**2 for row in x))or 1 for i in range(6)]
a=[[v/scale[i]for i,v in enumerate(row)]for row in x];coef=[0.]*6;pred=[0.]*len(y);ridge=1e-4
for iteration in range(20000):
    change=0.
    for j in range(6):
        numerator=sum(row[j]*(target-estimate+row[j]*coef[j])for row,target,estimate in zip(a,y,pred))
        denominator=sum(row[j]**2 for row in a)+ridge*len(y)
        value=numerator/denominator
        if j in[1,2,3,5]:value=max(0.,value)
        delta=value-coef[j];coef[j]=value;change=max(change,abs(delta))
        pred=[v+row[j]*delta for row,v in zip(a,pred)]
    if change<1e-12:break
coefficients=[v/s for v,s in zip(coef,scale)];assert all(math.isfinite(v)for v in coefficients)
model={'coefficients':coefficients};(ROOT/'model.json').write_text(json.dumps(model,indent=2))
audit={'calibration_sha256':hashlib.sha256(raw).hexdigest(),'training_distances':[18,40,80,160],'samples':len(y),'features':['intercept','triangle_delta_million','draw_delta','perimeter_proxy_delta','projected_megapixels','native_normal_saving_megapixels'],'ridge':ridge,'weighting':'inverse median candidate time, floor0.05ms','iterations':iteration+1,'model':model,'training_rmse_ms':math.sqrt(st.mean((a-b/w)**2 for a,b,w in zip(original_y,pred,weights))),'training_sign_errors':sum((a>0)!=(b>0)for a,b in zip(y,pred))}
(ROOT/'fit.json').write_text(json.dumps(audit,indent=2));print(json.dumps(audit,indent=2))
