#!/usr/bin/env python3
"""Source5: a local XYZ control for the central Cupid's bow valley.

Existing targets and all zero-offset geometry remain exactly as in source4.
The control cage's outer lip border separates this landmark from the upper
vermilion/tubercle and both paired Cupid peaks. No upstream data is acquired.
"""
import hashlib,json
from collections import defaultdict
from extend_source import ROOT,OUT,neutral

def main():
    s=json.loads((OUT/'human-smooth.ignis-character').read_text());assert s['version']==4
    s.update(version=5,id='human_adult_v5')
    s['provenance']+=' Source5: Ignis-authored local central Cupid valley XYZ field; source4 geometry/targets unchanged at zero.'
    points=neutral(s);skin=next(p for p in s['parts'] if p['name']=='body');local={v:i for i,v in enumerate(skin['vertices'])};faces=s['refinements'][0]['faces']
    # On the original cage this is the central OUTER vermilion border, above
    # upper_lip467 and upper_tubercle509, between paired peaks456/7215.
    anchor=455;center=points[anchor];assert abs(center[0])<1e-8 and .614<center[1]<.617
    radii=[.0065,.010,.020];limits=[.004,.004,.010];channels=['face_cupid_valley_'+axis for axis in ['x','y','z']]
    weights={}
    for i in skin['vertices']:
        p=points[i]
        if p[2]>-.12 or p[1]<.608:continue # Exclude inside the mouth/lower lip.
        local_radii=[radii[0],.0038 if p[1]<center[1] else radii[1],radii[2]]
        d=sum(((p[a]-center[a])/local_radii[a])**2 for a in range(3))
        if d<1:weights[i]=(1-d)**2
    incident=defaultdict(list);neighbors=defaultdict(set)
    for f in faces:
        if local[anchor] in f:
            incident[anchor].append(f)
            at=f.index(local[anchor]);neighbors[anchor].update([f[at-1],f[(at+1)%len(f)]])
    n=len(neighbors[anchor]);row=defaultdict(float);row[local[anchor]]+=(n-3)/n
    for f in incident[anchor]:
        for v in f:row[v]+=1/(n*n*len(f))
    for v in neighbors[anchor]:row[local[anchor]]+=1/(n*n);row[v]+=1/(n*n)
    response=sum(w*weights.get(skin['vertices'][v],0) for v,w in row.items());assert .25<response<=1
    for axis,(name,limit) in enumerate(zip(channels,limits)):
        s['parameters'].append(dict(name=name,minimum=-limit,maximum=limit,default=0))
        s['targets'].append(dict(name='handle_cupid_valley_'+str(axis),gain=1,factors=[dict(parameter=name,curve=[[-limit,-1],[0,0],[limit,1]])],deltas=[dict(vertex=i,offset=[round(limit*w/response,9) if k==axis else 0 for k in range(3)]) for i,w in sorted(weights.items())]))
    s['handles'].append(dict(name='cupid_valley',label='Vale central do arco do cupido (V)',group='Lábios',part='body',anchor=dict(vertices=[anchor]),parameters=channels,limits=limits,mirror=None,midline=True,frame_joint=next(i for i,j in enumerate(s['joints']) if j['name']=='head')))
    s['parameters'].sort(key=lambda p:p['name']);assert len(s['parameters'])==256 and len(s['handles'])==65
    data=json.dumps(s,separators=(',',':')).encode();path=OUT/'human-cupid.ignis-character';path.write_bytes(data)
    out=ROOT/'artifacts/character-cupid';out.mkdir(exist_ok=True);report=dict(path=str(path),bytes=len(data),sha256=hashlib.sha256(data).hexdigest(),parameters=len(s['parameters']),targets=len(s['targets']),handles=len(s['handles']),anchor=anchor,affected_vertices=len(weights),subdivision_response=response)
    (out/'source.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
if __name__=='__main__':main()
