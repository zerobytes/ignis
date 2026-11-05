# /// script
# requires-python = ">=3.11"
# dependencies = ["numpy>=2,<3"]
# ///
"""Read-only evidence for GPU Gems applicability; no rendering or speedup test."""
from pathlib import Path
import hashlib
import json
import numpy as np
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'artifacts/lighting-gems-review'
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def main():
    inputs={};views=[];storage=[]
    for label,relative in [('front','artifacts/lighting-bake/open/base'),('E','artifacts/lighting-bake/open/reveal-e'),('F','artifacts/lighting-bake/open/reveal-f')]:
        path=ROOT/relative;scene_path=path/'scene.json';receiver_path=path/'receivers.bin'
        scene=json.loads(scene_path.read_text());r=np.fromfile(receiver_path,dtype='<f4').reshape(-1,12);live=r[:,3]>0
        assert np.isfinite(r[live]).all() and live.any()
        sun=-np.array(scene['light']['direction']);sun/=np.linalg.norm(sun);zero=(r[live,4:7]@sun)<=0
        views.append(dict(view=label,visible_receivers=int(live.sum()),zero_direct_cosine_receivers=int(zero.sum()),fraction=float(zero.mean()),meaning='Eligibility only for the flat opaque zero-transmission laboratory; not measured shader invocations or GPU savings'))
        inputs.update({str(p.relative_to(ROOT)):sha(p) for p in [scene_path,receiver_path]})
    for label in ['open','thin-wall']:
        path=ROOT/f'artifacts/lighting-regional/{label}/initial-deps.bin';u=np.fromfile(path,dtype='<u4').reshape(-1,12);n=len(u);objects=n//(6*32*32);assert objects in [8,9]
        hits=np.array([int(v).bit_count() for v in u[:,3]]);entries=int(hits.sum())
        storage.append(dict(scene=label,texels=n,objects=objects,mean_first_hit_objects=float(hits.mean()),max_first_hit_objects=int(hits.max()),dense_scalar_weights_bytes=n*objects*4,sparse_scalar_weights_upper_bytes=(n+1)*4+entries*8,meaning='Planning estimate for fixed-geometry/fixed-sun one-bounce constant-albedo coefficients; first-hit masks include zero-contribution hits, so sparse entry count is an upper bound. Not an implemented matrix or GPU allocation.'))
        inputs[str(path.relative_to(ROOT))]=sha(path)
    source={f:sha(ROOT/f) for f in ['src/render.rs','src/scene.wgsl','src/environment.rs','examples/lighting_bounce/transport.wgsl','examples/lighting_bounce/regional.wgsl']}
    result=dict(date='2026-11-05',thread_id='01a1a9ae-16ea-76e3-86db-d55c4a43981b',thread_title='Ler e documentar GPU Gems 3',source_sha256=source,input_sha256=inputs,shadow_eligibility=views,coefficient_storage=storage,no_new_render_or_performance_measurement=True)
    OUT.mkdir(parents=True,exist_ok=True);(OUT/'evidence.json').write_text(json.dumps(result,indent=2))
    print(json.dumps(dict(shadow_eligibility=views,coefficient_storage=storage),indent=2))
if __name__=='__main__':main()
