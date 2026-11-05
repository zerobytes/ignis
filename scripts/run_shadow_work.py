# /// script
# requires-python = ">=3.11"
# dependencies = ["numpy>=2,<3", "pillow>=11,<13"]
# ///
"""Build and evaluate native-only shadow-query rejection against its legacy oracle."""
from pathlib import Path
import hashlib
import json
import subprocess
import sys
import numpy as np
from PIL import Image,ImageDraw,ImageFont
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'artifacts/shadow-work';OUT.mkdir(parents=True,exist_ok=True)
assert sys.argv[1:] in [[],['--report-only']], 'Expected no arguments or --report-only'
if '--report-only' not in sys.argv:
    with (OUT/'build.log').open('w') as log:subprocess.run(['cargo','build','--locked','--release','--example','shadow_work'],cwd=ROOT,stdout=log,stderr=subprocess.STDOUT,check=True)
    with (OUT/'run.log').open('w') as log:subprocess.run([str(ROOT/'target/release/examples/shadow_work')],cwd=ROOT,stdout=log,stderr=subprocess.STDOUT,check=True)
else:
    previous=json.loads((OUT/'summary.json').read_text())
    assert all(hashlib.sha256((ROOT/f).read_bytes()).hexdigest()==v for f,v in previous['source_sha256'].items())
    assert previous['executable_sha256']==hashlib.sha256((ROOT/'target/release/examples/shadow_work').read_bytes()).hexdigest()
d=json.loads((OUT/'results.json').read_text());assert not d['added_gi'] and len(d['cases'])==51
rng=np.random.default_rng(7443);cases=[]
for c in d['cases']:
    assert c['png_exact'] and c['moving_pairs']==4
    paths={}
    for enabled in [False,True]:
        rows=sorted([r for r in c['rows'] if r['enabled']==enabled],key=lambda r:r['trial']);assert len(rows)==64
        total=np.array([r['timing']['sum_ms'] for r in rows]);assert np.all(total>0)
        labels=sorted({p['pass'] for r in rows for p in r['timing']['passes']})
        passes={label:float(np.median([p['ms'] for r in rows for p in r['timing']['passes'] if p['pass']==label])) for label in labels}
        paths[enabled]=dict(total=total,passes=passes,cpu_ms=float(np.median([r['cpu_ms'] for r in rows])))
    baseline=paths[False]['total'];candidate=paths[True]['total'];ratio=float(np.median(candidate)/np.median(baseline));indices=rng.integers(0,64,size=(2000,64));ratios=np.median(candidate[indices],axis=1)/np.median(baseline[indices],axis=1)
    cases.append(dict(label=c['label'],mode=c['mode'],pose=c['pose'],baseline_gpu_ms=float(np.median(baseline)),candidate_gpu_ms=float(np.median(candidate)),ratio=ratio,ratio_ci95=np.percentile(ratios,[2.5,97.5]).tolist(),baseline_cpu_ms=paths[False]['cpu_ms'],candidate_cpu_ms=paths[True]['cpu_ms'],baseline_passes=paths[False]['passes'],candidate_passes=paths[True]['passes']))
representative=[c for c in cases if c['mode'] in ['matte','bias','dielectric','metal','bump','emissive-ibl'] and c['pose'] in [1,2]];assert len(representative)==12
ratio=float(np.exp(np.mean(np.log([c['ratio'] for c in representative]))))
controls=[c for c in cases if (c['mode'] in ['matte','bias'] and c['pose']==0) or c['mode']=='shadow-off' or c['mode'].endswith(('glass','transmission','mask'))]
passed=ratio<=.98 and all(c['ratio']<=1.05 for c in controls)
source={f:hashlib.sha256((ROOT/f).read_bytes()).hexdigest() for f in ['src/scene.wgsl','src/render.rs','src/render_instances.rs','examples/shadow_work.rs']}
summary=dict(adapter=d['adapter'],width=d['width'],height=d['height'],static_exact_pairs=51,moving_exact_pairs=204,perturbed_exact_pairs=153,sequence_zero_step_pairs=51,representative_geomean_ratio=ratio,controls_pass=all(c['ratio']<=1.05 for c in controls),acceptance_pass=passed,cases=cases,source_sha256=source,executable_sha256=hashlib.sha256((ROOT/'target/release/examples/shadow_work').read_bytes()).hexdigest(),analysis_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest())
(OUT/'summary.json').write_text(json.dumps(summary,indent=2));rows=''
for c in cases:rows+=f'<tr><td>{c["label"]}</td><td>{c["baseline_gpu_ms"]:.3f}</td><td>{c["candidate_gpu_ms"]:.3f}</td><td>{1-c["ratio"]:.1%}</td><td>{c["ratio_ci95"][0]:.3f}..{c["ratio_ci95"][1]:.3f}</td><td>{c["baseline_cpu_ms"]:.3f} / {c["candidate_cpu_ms"]:.3f}</td></tr>'
font=ImageFont.truetype('/System/Library/Fonts/Supplemental/Arial.ttf',22);canvas=Image.new('RGB',(1920,669),'#13191f');draw=ImageDraw.Draw(canvas)
for i,label in enumerate(['matte-2','transmission-2','glass-2']):
    im=Image.open(OUT/f'{label}.png').convert('RGB');im.thumbnail((640,624));canvas.paste(im,(i*640,45));draw.text((i*640+10,8),label+' — pixels idênticos',font=font,fill='white')
canvas.save(OUT/'comparison.png')
(OUT/'review.html').write_text(f'''<!doctype html><meta charset="utf-8"><title>Ignis — custo nativo das sombras</title><style>body{{background:#13191f;color:#e5edf4;font:16px system-ui;max-width:1500px;margin:24px auto}}td,th{{padding:8px;border-bottom:1px solid #40505d}}img{{width:100%}}</style><h1>Descarte de consultas de sombra sem contribuição</h1><p>SEM GI nova.51pares estáticos +204pares de sequência (153perturbações efetivas de câmera/sol): PNGs exatos. Aceite numérico/performance: {passed}; razão GPU geomédia representativa: {ratio:.3f}. Intervalos por bootstrap pareado de64trials, condicionais a estes casos/máquina. CPU preparação/submissão; GPU passes nativos, sem readback/PNG/compilação. Não é FPS nem ganho universal. Todos os casos no <a href="summary.json">JSON</a>.</p><img src="comparison.png"><table><tr><th>Caso</th><th>GPU referência ms</th><th>GPU candidato ms</th><th>Redução</th><th>Razão CI95</th><th>CPU referência/candidato ms</th></tr>{rows}</table>''')
print(json.dumps(dict(acceptance_pass=passed,representative_geomean_ratio=ratio,control_failures=[(c['label'],c['ratio']) for c in controls if c['ratio']>1.05]),indent=2))
