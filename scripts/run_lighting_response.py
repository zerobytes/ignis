# /// script
# dependencies = ["numpy", "pillow"]
# ///
"""Run and independently audit the bounded scalar-response experiment."""
from pathlib import Path
import argparse
import copy
import hashlib
import html
import json
import subprocess
import numpy as np
from PIL import Image
from lighting_bake_report import direct_cpu

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'artifacts/lighting-response'
EXE = ROOT / 'target/release/examples/lighting_bounce'
SOURCES = ['examples/lighting_bounce.rs', *[str(p.relative_to(ROOT)) for p in sorted((ROOT / 'examples/lighting_bounce').glob('*')) if p.is_file()], 'scripts/run_lighting_response.py', 'scripts/lighting_bake_report.py']

def hashes():
    return {f: hashlib.sha256((ROOT / f).read_bytes()).hexdigest() for f in SOURCES}

def audit(label):
    path = OUT / label
    data = json.loads((path / 'results.json').read_text())
    scene = json.loads((path / 'source-scene.json').read_text())
    n, side, objects = data['texels'], data['side'], data['objects']
    weights = np.fromfile(path / 'weights.bin', dtype='<f4').reshape(n, -1).astype(float)
    assert np.isfinite(weights).all() and (weights >= 0).all()
    assert weights.sum(axis=1).max() <= 1.0001
    assert (weights[:, objects:] == 0).all()
    basis_errors = []
    # Brightest row for two distinct source columns: independent scalar ray oracle.
    for source in np.argsort(weights[:, :objects].max(axis=0))[-2:]:
        idx = int(np.argmax(weights[:, source]))
        obj, remainder = divmod(idx, 6 * side * side)
        face, pixel = divmod(remainder, side * side)
        y, x = divmod(pixel, side)
        entity = scene['entities'][obj]
        lo = np.array(entity['transform']['position']) - np.array(entity['transform']['size']) / 2
        hi = np.array(entity['transform']['position']) + np.array(entity['transform']['size']) / 2
        v = np.zeros(3)
        axis = face // 2
        v[axis] = 1 if face % 2 == 0 else 0
        v[[a for a in range(3) if a != axis]] = [(x + .5) / side, (y + .5) / side]
        recv = np.zeros((1, 12))
        recv[0, :3] = lo + (hi - lo) * v
        recv[0, 4 + axis] = 1 if face % 2 == 0 else -1
        recv[0, 8:11] = 1
        basis = copy.deepcopy(scene)
        for j, e in enumerate(basis['entities']):
            e['color'] = [float(j == source)] * 3
        basis['light']['direct_color'] = [1.] * 3
        basis['light']['direct_intensity'] = 1.
        expected = direct_cpu(recv, basis, 0, 64, 64, 317, 519983, idx, True)
        assert expected.max() > 0 and weights[idx, source] > 0
        error = float(np.max(abs(expected - weights[idx, source])))
        assert error < 5e-6, (label, idx, int(source), error)
        basis_errors.append(error)
    cases = []
    for case in data['cases']:
        loc = path / f"{case['step']:02}"
        current = json.loads((loc / 'scene.json').read_text())
        a = np.fromfile(loc / 'combined.bin', dtype='<f4').reshape(n, 4).astype(float)
        b = np.fromfile(loc / 'reference.bin', dtype='<f4').reshape(n, 4).astype(float)
        assert np.isfinite(a).all() and np.isfinite(b).all()
        assert np.array_equal(a[:, 3], b[:, 3])
        colors = np.array([e['color'] for e in current['entities']])
        energy = np.array(current['light']['direct_color']) * current['light']['direct_intensity']
        expected = (weights[:, :objects] @ colors) * energy
        cpu_error = float(np.max(abs(expected - a[:, :3])))
        error = float(np.max(abs(a[:, :3] - b[:, :3])))
        assert cpu_error < 3e-6 and error <= 2e-4, (label, case['step'], cpu_error, error)
        if case['step'] in (1, 7):
            assert (a[:, :3] == 0).all()
        if case['step'] == 8:
            assert (a[:, :3].max(axis=0) > 0).all()
        png_a = np.asarray(Image.open(loc / 'combined.png')).astype(int)
        png_b = np.asarray(Image.open(loc / 'reference.png')).astype(int)
        png_error = int(np.max(abs(png_a - png_b)))
        assert png_error <= 1, (label, case['step'], png_error)
        timing = {}
        for method in ('coefficients', 'full-bake'):
            rows = [r for r in case['rows'] if r['method'] == method]
            assert len(rows) == 16
            timing[method] = {key: float(np.median([r[key] for r in rows])) for key in ('gpu_ms', 'cpu_submit_ms', 'compute_gpu_ms', 'publication_gpu_ms')}
        cases.append(dict(step=case['step'], field_max_error=error, field_mse=float(np.mean((a[:, :3] - b[:, :3]) ** 2)), cpu_recombine_max_error=cpu_error, png_max_error=png_error, timing=timing))
    return dict(scene=label, weight_bytes=data['weight_bytes'], cold_build_gpu_ms=data['cold_build_gpu_ms'], cold_build_cpu_ms=data['cold_build_cpu_ms'], cpu_basis_max_error=max(basis_errors), cases=cases)

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--report-only', action='store_true')
    args = parser.parse_args()
    OUT.mkdir(parents=True, exist_ok=True)
    if not args.report_only:
        source = hashes()
        subprocess.run(['cargo', 'build', '--locked', '--release', '--example', 'lighting_bounce'], cwd=ROOT, check=True)
        assert source == hashes(), 'Sources changed during build'
        manifest = dict(sources=source, executable=hashlib.sha256(EXE.read_bytes()).hexdigest())
        for thin in (False, True):
            subprocess.run([str(EXE), '--response-audit', *(['--thin-wall'] if thin else [])], cwd=ROOT, check=True)
        assert source == hashes(), 'Sources changed during experiment'
        (OUT / 'manifest.json').write_text(json.dumps(manifest, indent=2))
    manifest = json.loads((OUT / 'manifest.json').read_text())
    assert manifest['sources'] == hashes(), 'Stale source manifest'
    assert manifest['executable'] == hashlib.sha256(EXE.read_bytes()).hexdigest(), 'Executable changed'
    reports = [audit(label) for label in ('open', 'thin-wall')]
    (OUT / 'summary.json').write_text(json.dumps(dict(manifest=manifest, scenes=reports), indent=2))
    body = ['<h1>Fixed geometry scalar color response</h1><p>One diffuse bounce; geometry/sun direction fixed. Timings include computation and GPU publication, exclude uploads, initialization, fences and readback. Added GI remains extra versus GI off.</p><table><tr><th>Scene</th><th>Weights MiB</th><th>Cold GPU ms</th><th>Update GPU ms</th><th>Full bake GPU ms</th><th>Max RGB error</th></tr>']
    for report in reports:
        cases = report['cases']
        fast = np.median([c['timing']['coefficients']['gpu_ms'] for c in cases])
        full = np.median([c['timing']['full-bake']['gpu_ms'] for c in cases])
        body.append(f"<tr><td>{html.escape(report['scene'])}</td><td>{report['weight_bytes']/2**20:.3f}</td><td>{report['cold_build_gpu_ms']:.3f}</td><td>{fast:.4f}</td><td>{full:.3f}</td><td>{max(c['field_max_error'] for c in cases):.3g}</td></tr>")
    body.append('</table>')
    for label in ('open', 'thin-wall'):
        for step in (0, 2, 7, 8, 19):
            body.append(f'<h2>{label} case {step:02}: full bake / coefficients</h2><img width="48%" src="{label}/{step:02}/reference.png"><img width="48%" src="{label}/{step:02}/combined.png">')
    (OUT / 'review.html').write_text('<!doctype html><meta charset="utf-8"><style>body{background:#202127;color:#eee;font:16px system-ui;margin:30px}td,th{padding:12px;border:1px solid #666}img{vertical-align:top}</style>' + ''.join(body))
    print(json.dumps([{k: v for k, v in r.items() if k != 'cases'} for r in reports], indent=2))

if __name__ == '__main__':
    main()
