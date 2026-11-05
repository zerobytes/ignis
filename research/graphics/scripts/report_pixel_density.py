"""Build the offline comparison without hiding repeats or preparation cost."""
import json
from pathlib import Path

root = Path('artifacts/appearance-probe/pixel-density')
read = lambda name: json.loads((root / f'{name}.json').read_text())
cohorts = read('validation') + read('repeat')
curves = []
for resolution in range(2):
    methods = []
    for method in range(3):
        points = []
        for density in range(4):
            ss = [s for c in cohorts if c['resolution'] == resolution for s in c['samples'] if s['density'] == density and s['method'] == method]
            point = {'frames': ss[0]['frames'], 'gpu_frames': ss[0]['gpu_frames']}
            for key in ['total_ms', 'gpu_ms', 'cpu_ms']:
                medians = [s[key]['median'] for s in ss]
                point[key] = {'mean': sum(medians)/len(medians), 'min': min(medians), 'max': max(medians), 'medians': medians}
            points.append(point)
        methods.append(points)
    curves.append(methods)
data = {'rows': read('quality'), 'preparation': read('preparation'), 'verification': read('verification'), 'curves': curves}
(root / 'summary.json').write_text(json.dumps(data, indent=2) + '\n')
template = Path('scripts/pixel_density_report.html').read_text()
(root / 'index.html').write_text(template.replace('__DATA__', json.dumps(data, separators=(',', ':'))))
for res, methods in enumerate(curves):
    print('resolution', res)
    for d in range(4):
        p = next(p['metadata'] for p in data['preparation'] if p['resolution'] == res and p['density'] == d)
        print(p['target_triangles_per_pixel'], p['triangles'], 'total', [round(m[d]['total_ms']['mean'], 6) for m in methods], 'GPU', [round(m[d]['gpu_ms']['mean'], 6) for m in methods], 'MiB', round(p['geometry_bytes']/1048576, 2), 'prep_ms', round(p['prepare_upload_ms'], 2))
