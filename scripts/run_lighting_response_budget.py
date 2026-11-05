# /// script
# dependencies = ["numpy", "pillow"]
# ///
"""Reproduce regional coefficients, asynchronous frame admission and quality audit."""
from pathlib import Path
import argparse
import hashlib
import html
import json
import subprocess
import numpy as np
from PIL import Image
from lighting_bake_report import direct_cpu, hash32

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'artifacts/lighting-response-budget'
EXE = ROOT / 'target/release/examples/lighting_bounce'
SOURCE = ['examples/lighting_bounce.rs', 'scripts/run_lighting_response_budget.py', 'scripts/lighting_bake_report.py', *[str(p.relative_to(ROOT)) for p in sorted((ROOT / 'examples/lighting_bounce').glob('*')) if p.is_file()]]

def hashes():
    return {p: hashlib.sha256((ROOT / p).read_bytes()).hexdigest() for p in SOURCE}

def png_error(a, b):
    a = np.asarray(Image.open(a)).astype(int)
    b = np.asarray(Image.open(b)).astype(int)
    return int(abs(a - b).max()), float(np.mean((a - b) ** 2))

def basis_check(weights, scene, side):
    source = int(np.argmax(weights.max(axis=0)))
    index = int(np.argmax(weights[:, source]))
    assert weights[index, source] > 0
    obj, rem = divmod(index, 6 * side * side)
    face, pixel = divmod(rem, side * side)
    y, x = divmod(pixel, side)
    entity = scene['entities'][obj]
    lo = np.array(entity['transform']['position']) - np.array(entity['transform']['size']) / 2
    hi = np.array(entity['transform']['position']) + np.array(entity['transform']['size']) / 2
    uvw = np.zeros(3)
    axis = face // 2
    uvw[axis] = float(face % 2 == 0)
    uvw[[a for a in range(3) if a != axis]] = [(x + .5) / side, (y + .5) / side]
    receiver = np.zeros((1, 12))
    receiver[0, :3] = lo + (hi - lo) * uvw
    receiver[0, 4 + axis] = 1 if face % 2 == 0 else -1
    receiver[0, 8:11] = 1
    # Build a fresh basis scene; direct_cpu does independent ray/slab integration.
    scene = json.loads(json.dumps(scene))
    for j, entity in enumerate(scene['entities']):
        entity['color'] = [float(j == source)] * 3
    scene['light']['direct_color'] = [1.] * 3
    scene['light']['direct_intensity'] = 1.
    expected = direct_cpu(receiver, scene, 0, 64, 64, 317, 519983, index, False)
    error = float(abs(expected - weights[index, source]).max())
    assert error < 5e-6, error
    return error

def audit(label):
    path = OUT / label
    data = json.loads((path / 'results.json').read_text())
    seeds = np.fromfile(path/'seeds.bin', dtype='<u4').reshape(data['texels'],64)
    for row in (0,1,123,data['texels']//2,data['texels']-1):
        seed = hash32(row+317*91237+519983)
        for block in range(64):
            assert seeds[row,block] == seed, ('RNG checkpoint',label,row,block)
            for _ in range(64):
                seed=hash32(seed+0x9e3779b9)
                seed=hash32(seed+0x9e3779b9)
    reports = []
    for budget in (1., 2.):
        events = [e for e in data['events'] if e['budget_ms'] == budget]
        assert len(events) == 17
        previous_png = path / f'{budget:.1f}/initial.png'
        for e in events:
            loc = path / f"{budget:.1f}/{e['step']:02}-{e['label']}"
            scene = json.loads((loc / 'scene.json').read_text())
            n, objects = data['texels'], len(scene['entities'])
            weights = np.fromfile(loc / 'weights.bin', dtype='<f4').reshape(n, -1).astype(float)
            assert np.isfinite(weights).all() and (weights >= 0).all()
            assert weights.sum(axis=1).max() <= 1.0001
            assert (weights[:, objects:] == 0).all()
            deps = np.fromfile(loc / 'deps.bin', dtype='<u4').reshape(n, 12)
            valid = deps[:, 8] == 0
            assert (weights[~valid] == 0).all()
            colors = np.array([entity['color'] for entity in scene['entities']])
            energy = np.array(scene['light']['direct_color']) * scene['light']['direct_intensity']
            combined = (weights[:, :objects] @ colors) * energy
            plain = np.fromfile(loc / 'reference-atlas.bin', dtype='<f4').reshape(n, 4).astype(float)
            assert np.array_equal(plain[:, 3], valid.astype(float))
            field_error = float(abs(combined - plain[:, :3]).max())
            assert field_error <= 2e-4
            published = np.fromfile(loc / 'published.rgba32f', dtype='<f4').reshape(-1, 4).astype(float)
            reference = np.fromfile(loc / 'reference.rgba32f', dtype='<f4').reshape(-1, 4).astype(float)
            full = np.fromfile(loc / 'plain.rgba32f', dtype='<f4').reshape(-1, 4).astype(float)
            pending = np.fromfile(loc / 'pending64.rgba32f', dtype='<f4').reshape(-1, 4).astype(float)
            assert np.array_equal(published, reference)
            assert abs(published[:, :3] - full[:, :3]).max() <= 2e-4
            assert png_error(loc / 'published.png', loc / 'reference.png')[0] == 0
            png, _ = png_error(loc / 'published.png', loc / 'plain.png')
            assert png <= 1
            assert (loc / 'held.png').read_bytes() == previous_png.read_bytes()
            assert (loc/'background-final.png').read_bytes() == previous_png.read_bytes()
            previous_png = loc / 'published.png'
            fallback_png, fallback_png_mse = png_error(loc / 'pending64.png', loc / 'reference.png')
            held_png, held_png_mse = png_error(loc / 'held.png', loc / 'reference.png')
            rows, ticks = e['rows'], e['ticks']
            assert len(ticks) == e['frames'] + 1
            assert all(row['completion_frame'] > row['issued_frame'] for row in rows)
            assert len({row['issued_frame'] for row in rows}) == len(rows)
            assert sum(t.get('published_epoch', 0) != 0 for t in ticks) == int(e['budget_completed'])
            assert ticks[-1]['published_epoch'] == (e['epoch'] if e['budget_completed'] else 0)
            if not e['budget_completed']:
                assert ticks[-1]['deadline_rejected'] and e['offline_completion_gpu_ms']>0
            if e['step'] >= 15:
                assert e['cancelled'] == 1 and any(r['stale_discarded'] for r in rows)
                if e['step'] == 16:
                    assert any(r['stale_discarded'] and r['phase'] == 'Publish' for r in rows)
            else:
                assert e['cancelled'] == 0 and not any(r['stale_discarded'] for r in rows)
            admitted = sum(r['rows'] for r in rows if r['phase'] == 'Repair' and not r['stale_discarded'])
            assert admitted == e['dirty'] if e['budget_completed'] else admitted <= e['dirty']
            assert all(r['rows'] <= 2048 for r in rows if r['phase'] == 'Repair')
            assert all(e[k] for k in ('weights_exact', 'dependencies_exact', 'published_exact', 'background_exact'))
            assert all(c['gpu_ms'] >= e['full_kernel_gpu_ms']*.5 for c in e['controls'] if c['method']=='FullControl')
            if e['step']==7:
                assert e['visible_last_object_pixels']==0 and e['changed_weight_rows']>0
            cpu_basis = basis_check(weights[:, :objects], scene, 32) if e['step'] in (2, 9, 12, 15) and budget == 1. else None
            controls = {name: float(np.median([r['gpu_ms'] for r in e['controls'] if r['method'] == name])) for name in ('RegionalControl', 'FullControl', 'PlainControl')}
            reports.append(dict(budget_completed=e['budget_completed'], offline_completion_gpu_ms=e['offline_completion_gpu_ms'], budget_ms=budget, step=e['step'], label=e['label'], dirty_fraction=e['dirty']/n, changed_weight_rows=e['changed_weight_rows'], frames=e['frames'], latency_ms=e['latency_ms'], gpu_total_ms=sum(r['gpu_ms'] for r in rows), gpu_slice_max_ms=max(r['gpu_ms'] for r in rows), gpu_slice_p95_ms=float(np.percentile([r['gpu_ms'] for r in rows], 95)), over_budget=sum(r['over_budget'] for r in rows), batches=len(rows), cpu_tick_p95_ms=float(np.percentile([t['cpu_tick_ms'] for t in ticks],95)), cpu_tick_max_ms=max(t['cpu_tick_ms'] for t in ticks), background_cpu_p95_ms=float(np.percentile([t.get('background_cpu_ms',0) for t in ticks],95)), total_cpu_tick_p95_ms=float(np.percentile([t.get('total_cpu_tick_ms',t['cpu_tick_ms']) for t in ticks],95)), background_native_gpu_ms=e['background_native_gpu_ms'], background_query_gpu_ms=e['background_query_gpu_ms'], full_controls_ms=controls, native_gpu_ms=e['native_gpu_ms'], query_gpu_ms=e['baked_query_gpu_ms'], fallback_query_gpu_ms=e['pending_query_gpu_ms'], field_max_error=field_error, png_max_error=png, fallback_png_max_error=fallback_png, fallback_png_mse=fallback_png_mse, fallback_rgb_mse=float(np.mean((pending[:, :3]-reference[:, :3])**2)), held_png_max_error=held_png, held_png_mse=held_png_mse, cpu_basis_max_error=cpu_basis, cancelled=e['cancelled']))
    return dict(scene=label, adapter=data['adapter'], weight_bytes_per_bank=data['weight_bytes_per_bank'], metadata_bytes_per_bank=data['metadata_bytes_per_bank'], diagnostic_lab_gpu_buffer_bytes=data['diagnostic_lab_gpu_buffer_bytes'], events=reports)

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--report-only', action='store_true')
    parser.add_argument('--cached-backend', type=Path, help='JSON files map for an explicitly pinned prebuilt Ignis backend')
    args = parser.parse_args()
    OUT.mkdir(parents=True, exist_ok=True)
    if not args.report_only:
        source = hashes()
        backend = None
        if args.cached_backend:
            files = json.loads(args.cached_backend.read_text())['files']
            allowed = {'ignis','anyhow','bytemuck','glam','serde_json','wgpu','pollster','serde','ring'}
            assert set(files) == allowed
            assert all(Path(f).name == f and f.endswith('.rlib') for f in files.values())
            backend = {f'target/release/deps/{f}': hashlib.sha256((ROOT/'target/release/deps'/f).read_bytes()).hexdigest() for f in files.values()}
            command = ['rustc','--crate-name','lighting_bounce','--edition=2024','examples/lighting_bounce.rs','-C','opt-level=3','-C','strip=symbols','-o',str(EXE),'-L','dependency=target/release/deps']
            for name, file in files.items():
                command += ['--extern', f'{name}=target/release/deps/{file}']
            for out in (ROOT/'target/release/build').glob('*/out'):
                if out.is_dir():
                    command += ['-L', f'native={out}']
            subprocess.run(command,cwd=ROOT,check=True)
            assert all(hashlib.sha256((ROOT/file).read_bytes()).hexdigest()==expected for file,expected in backend.items()), 'Backend changed while linking'
            (OUT/'cached-backend.json').write_text(json.dumps(dict(files=files,hashes=backend,command=command),indent=2))
        else:
            subprocess.run(['cargo','build','--locked','--release','--example','lighting_bounce'],cwd=ROOT,check=True)
        assert source == hashes(), 'Sources changed during build'
        manifest = dict(sources=source, executable=hashlib.sha256(EXE.read_bytes()).hexdigest(), backend=backend)
        for thin in (False, True):
            subprocess.run([str(EXE),'--response-budget-audit',*(['--thin-wall'] if thin else [])],cwd=ROOT,check=True)
        assert source == hashes(), 'Sources changed during experiment'
        (OUT / 'manifest.json').write_text(json.dumps(manifest,indent=2))
    manifest = json.loads((OUT / 'manifest.json').read_text())
    assert manifest['sources'] == hashes(), 'Stale sources'
    assert manifest['executable'] == hashlib.sha256(EXE.read_bytes()).hexdigest(), 'Stale executable'
    # Backend hashes describe what was linked. Parallel Cargo work may replace
    # those libraries later; the immutable experimental identity is the EXE hash.
    reports = [audit(label) for label in ('open','thin-wall')]
    (OUT/'summary.json').write_text(json.dumps(dict(manifest=manifest,scenes=reports),indent=2))
    body = ['<h1>Regional coefficient repair and frame budget</h1><p>Strict publication holds the last COMPLETE SCENE IMAGE until ready. Jobs exceeding150ticks are rejected; their later settled gallery uses a separately charged OFFLINE test-driver repair, not a successful frame-budget update. Scene changes are delayed. This is not immediate gameplay lighting. Direct64 preview is diagnostic and may fail the quality gate. GPU budgets below cover repair batches, not the complete frame.</p><table><tr><th>Scene/budget</th><th>Edit</th><th>Dirty %</th><th>GPU total / max slice ms</th><th>Overruns</th><th>Budget status; frames / wall ms</th><th>Unbudgeted regional / full ms</th><th>Final PNG error</th><th>Preview PNG error</th></tr>']
    for report in reports:
        for e in report['events']:
            body.append(f"<tr><td>{report['scene']} / {e['budget_ms']}</td><td>{html.escape(e['label'])}</td><td>{100*e['dirty_fraction']:.1f}</td><td>{e['gpu_total_ms']:.3f} / {e['gpu_slice_max_ms']:.3f}</td><td>{e['over_budget']}/{e['batches']}</td><td>{'complete' if e['budget_completed'] else 'REJECTED'}; {e['frames']} / {e['latency_ms']:.1f}; offline {e['offline_completion_gpu_ms']:.2f} ms</td><td>{e['full_controls_ms']['RegionalControl']:.3f} / {e['full_controls_ms']['FullControl']:.3f}</td><td>{e['png_max_error']}</td><td>{e['fallback_png_max_error']}</td></tr>")
    body.append('</table><p>Gallery: held prior image / approximate current preview / completed current image.</p>')
    for label in ('open','thin-wall'):
        for step, name in ((2,'move-a'),(7,'offscreen-blocker'),(9,'sun-direction'),(15,'supersede-repair')):
            prefix=f'{label}/1.0/{step:02}-{name}'
            body.append(f'<h2>{label} {name}</h2>'+''.join(f'<img width="32%" src="{prefix}/{file}.png">' for file in ('held','pending64','published')))
    (OUT/'review.html').write_text('<!doctype html><meta charset="utf-8"><style>body{background:#202127;color:#eee;font:14px system-ui;margin:25px}table{border-collapse:collapse}td,th{padding:8px;border:1px solid #555}img{vertical-align:top}</style>'+''.join(body))
    for report in reports:
        for budget in (1.,2.):
            events=[e for e in report['events'] if e['budget_ms']==budget]
            print(json.dumps(dict(scene=report['scene'],budget=budget,max_slice_ms=max(e['gpu_slice_max_ms'] for e in events),overruns=sum(e['over_budget'] for e in events),batches=sum(e['batches'] for e in events),max_frames=max(e['frames'] for e in events),max_latency_ms=max(e['latency_ms'] for e in events),rejected=sum(not e['budget_completed'] for e in events),max_final_png_error=max(e['png_max_error'] for e in events),max_preview_png_error=max(e['fallback_png_max_error'] for e in events))))

if __name__ == '__main__':
    main()
