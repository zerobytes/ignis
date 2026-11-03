"""Package the actual saved scene in a standalone geometry-inspection viewer."""
import hashlib
import json
from pathlib import Path

PROJECT=Path(__file__).resolve().parents[1]
ROOT=PROJECT/'artifacts/house-e02-rebuild'

def main():
    raw=(PROJECT/'examples/house-e02-rebuild.scene.json').read_bytes()
    s=json.loads(raw)
    payload={**s['meshes'],'camera':s['camera']}
    template=(PROJECT/'scripts/viewers/scene_orbit.html').read_text()
    compact=json.dumps(payload,separators=(',',':'),allow_nan=False).replace('<','\\u003c')
    out=template.replace('__SCENE_JSON__',compact)
    (ROOT/'orbit.html').write_text(out)
    (ROOT/'orbit-source.json').write_text(json.dumps({'scene_sha256':hashlib.sha256(raw).hexdigest(),'assets':len(payload['assets']),'instances':len(payload['instances']),'renderer':'WebGL inspection; not the native Ignis renderer','scene_modified':False},indent=2)+'\n')
    print(ROOT/'orbit.html')

if __name__=='__main__':main()
