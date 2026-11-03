#!/usr/bin/env python3
"""Summarize measured CPU/submission/wait costs without calling wait GPU time."""
import json
import bisect
import statistics
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'artifacts/botanical-valley/performance'


def main():
    data = json.loads((OUT / 'cost.json').read_text())
    assert len(data['rows']) == 12, 'The complete 3-camera, 2-resolution, 2-shadow matrix is required'
    assert all(row['draw_wait']['n'] == 16 for row in data['rows'])
    rows = []
    for row in data['rows']:
        gpu = row['renderer']
        instancing = gpu['instancing']
        rows.append(dict(pose=row['pose'], pixels=[row['width'], row['height']], variant=row['variant'],
            cpu_draw_submit_ms=row['cpu_draw_submit']['median_ms'],
            residual_completion_wait_ms=row['completion_wait']['median_ms'],
            draw_wait_ms=row['draw_wait']['median_ms'], p95_draw_wait_ms=row['draw_wait']['p95_ms'],
            source_triangles_selected=instancing['detailed_triangles'],
            batches_reported=instancing['draw_calls'], occlusion_active=gpu['occlusion']['active'],
            lod_active=gpu['mesh_lods']['enabled'], geometry_uploads=instancing['geometry_uploads']))
    def pick(pose, width, variant):
        return next(r for r in rows if r['pose'] == pose and r['pixels'][0] == width and r['variant'] == variant)
    contrasts = []
    for pose in ('player_spawn', 'valley', 'panorama'):
        full, off, quarter = pick(pose, 2200, 'full'), pick(pose, 2200, 'no_shadows'), pick(pose, 1100, 'full')
        contrasts.append(dict(pose=pose,
            shadow_off_reduction_ms=full['draw_wait_ms'] - off['draw_wait_ms'],
            shadow_off_reduction_percent=100 * (1 - off['draw_wait_ms'] / full['draw_wait_ms']),
            quarter_pixels_reduction_percent=100 * (1 - quarter['draw_wait_ms'] / full['draw_wait_ms'])))
    activity = json.loads((OUT / 'cost-concurrency.json').read_text())
    other = [r for r in activity if any(p['cpu_percent'] > 5 for p in r['compiler_or_trace'])]
    times = [r['unix_seconds'] for r in activity]
    low_compiler_subsets = []
    for row in data['rows']:
        selected = []
        for sample in row['samples']:
            start = int(sample['start_unix_ns']) / 1e9
            end = start + sample['draw_wait_ms'] / 1000
            first = max(0, bisect.bisect_right(times, start) - 1)
            last = min(len(activity), bisect.bisect_left(times, end) + 1)
            if all(all(p['cpu_percent'] <= 5 for p in a['compiler_or_trace']) for a in activity[first:last]):
                selected.append(sample)
        low_compiler_subsets.append(dict(pose=row['pose'], width=row['width'], variant=row['variant'],
            samples=len(selected), median_draw_wait_ms=statistics.median(s['draw_wait_ms'] for s in selected) if selected else None))
    report = dict(build_id=data['build_id'], samples=192, rows=rows, contrasts=contrasts,
        cpu=data['cpu'], cold=data['cold'], low_compiler_subsets=low_compiler_subsets,
        concurrency=dict(observations=len(activity), observations_with_other_compiler_or_trace_over_5_percent=len(other),
                         peak_other_cpu_percent=max((sum(p['cpu_percent'] for p in r['compiler_or_trace']) for r in activity), default=0)),
        method='16 warm observations/case in alternating blocks, 3 cameras, full/quarter pixels and shadows on/off. Serial CPU draw+submit then completion wait. No PNG/readback/present in timed loop. Wait is NOT isolated GPU execution. GPU and CPU may overlap inside draw; do not add independent GPU hardware intervals to these wall-clock stages.')
    (OUT / 'summary.json').write_text(json.dumps(report, indent=2) + '\n')
    lines = ['# Botanical valley: measured costs', '', f"Compiled engine: `{data['build_id']}`.", '',
        'Medians in milliseconds. Residual wait is a CPU wait for completion, not a GPU timestamp.', '',
        '| Camera | Pixels | Shadows | CPU draw/submit | Residual wait | Draw + wait | p95 |',
        '| --- | --- | --- | ---: | ---: | ---: | ---: |']
    for row in rows:
        lines.append(f"| {row['pose']} | {row['pixels'][0]}×{row['pixels'][1]} | {row['variant']} | {row['cpu_draw_submit_ms']:.2f} | {row['residual_completion_wait_ms']:.2f} | {row['draw_wait_ms']:.2f} | {row['p95_draw_wait_ms']:.2f} |")
    lines += ['', '## Controlled contrasts', '']
    for c in contrasts:
        lines.append(f"- {c['pose']}: shadow-off combined effect {c['shadow_off_reduction_ms']:.2f} ms / {c['shadow_off_reduction_percent']:.1f}%; quarter pixels {c['quarter_pixels_reduction_percent']:.1f}%.")
    lines += ['', '## Separate CPU simulation/frame preparation', '']
    for row in data['cpu']:
        lines.append(f"- {row['pose']}: one tick {row['one_tick']['median_ms']:.4f} ms; frame preparation {row['prepare_frame']['median_ms']:.2f} ms (24 observations each).")
    lines += ['', f"Background compiler/trace activity over5% CPU: {len(other)}/{len(activity)} monitoring observations. See cost-concurrency.json; do not describe the main matrix as an entirely idle machine.", '',
        'The authored source hash is checked before and after the run. No vegetation density, seed, mesh quality or placement changed. The no-shadow images are a diagnostic control, not a proposed visual replacement. This report does not measure presentation FPS.']
    (OUT / 'measurements.md').write_text('\n'.join(lines) + '\n')
    print(json.dumps(dict(contrasts=contrasts, concurrency=report['concurrency']), indent=2))


if __name__ == '__main__':
    main()
