#!/usr/bin/env python3
"""Summarize paired native geometry experiments. Requires numpy and Pillow.

No alignment, image enhancement or rescaling is used in error measurements.
"""
import json
import statistics
from pathlib import Path

import numpy as np
from PIL import Image


def summarize(directory):
    directory = Path(directory)
    report = json.loads((directory / 'cost.json').read_text())
    result = []
    for pose in sorted({r['pose'] for r in report['rows']}):
        rows = [r for r in report['rows'] if r['pose'] == pose]
        modes = {}
        for enabled in (False, True):
            samples = [r for r in rows if r['enabled'] == enabled]
            latest = samples[-1]['adapter']['instancing']
            modes[str(enabled).lower()] = dict(
                draw_wait_ms=statistics.median(r['draw_wait_median_ms'] for r in samples),
                cpu_submission_ms=statistics.median(r['cpu_submission_median_ms'] for r in samples),
                triangles=latest['detailed_triangles'] + latest['cheap_triangles'],
                selected_geometry_bytes=latest['selected_geometry_bytes'],
                allocated_geometry_bytes=latest['allocated_geometry_bytes'],
                selected_vertices=latest['selected_vertices'],
                selected_indices=latest['selected_indices'],
                camera_footprint=latest['camera_footprint'],
            )
        a = np.asarray(Image.open(directory / f'{pose}-reference.png').convert('RGB'), dtype=np.int16)
        b = np.asarray(Image.open(directory / f'{pose}-optimized.png').convert('RGB'), dtype=np.int16)
        delta = np.abs(a - b)
        peak = delta.max(axis=2)
        y, x = np.unravel_index(peak.argmax(), peak.shape)
        # Preserve raw crops at native scale for visual inspection of the worst pixel.
        h, w = peak.shape
        box = (max(0, int(x)-128), max(0, int(y)-128), min(w, int(x)+129), min(h, int(y)+129))
        for suffix in ('reference', 'optimized'):
            Image.open(directory / f'{pose}-{suffix}.png').crop(box).save(directory / f'{pose}-{suffix}-crop.png')
        # Only the difference visualization is amplified; never the measured pair.
        Image.fromarray(np.minimum(delta * 16, 255).astype('uint8')).save(directory / f'{pose}-difference-x16.png')
        result.append(dict(
            pose=pose, modes=modes,
            reduction_percent=100 * (1-modes['true']['draw_wait_ms']/modes['false']['draw_wait_ms']),
            max_rgb8=int(delta.max()), mae_rgb8=float(delta.mean()),
            changed_pixels=int(np.count_nonzero(peak)),
            pixels_over_2=int(np.count_nonzero(peak > 2)),
            changed_fraction=float(np.mean(peak != 0)),
            worst_pixel=[int(x), int(y)], worst_crop=list(box),
        ))
    (directory / 'summary.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    import sys
    for path in sys.argv[1:]:
        summarize(path)
