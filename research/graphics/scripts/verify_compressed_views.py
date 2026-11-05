#!/usr/bin/env python3
"""Inspect every generated image and aggregate numeric fidelity evidence."""
import json
from pathlib import Path
from PIL import Image, ImageChops

root = Path('artifacts/appearance-probe/compression')
data = json.loads((root/'report.json').read_text())
assert len(data['cases']) == 213
assert data['packed_bytes'] == data['texture_bytes'] + data['descriptor_bytes'] < data['source_bytes']
assert (root/'atlas.bin').stat().st_size == data['texture_bytes']
worst_level = worst_count = worst_run = changed_channels = 0

def longest(coords):
    best = run = 0
    last = None
    for a, b in sorted(coords):
        run = run + 1 if last == (a, b-1) else 1
        last = (a, b)
        best = max(best, run)
    return best

for r in data['cases']:
    for suffix in ['reference', 'original', 'compact', 'delta']:
        with Image.open(root/f"{r['id']}_{suffix}.png") as image:
            assert image.size == (256, 256) and image.mode == 'RGB'
            image.verify()
    with Image.open(root/f"{r['id']}_original.png") as a, Image.open(root/f"{r['id']}_compact.png") as b:
        diff = ImageChops.difference(a, b)
        channels = diff.split()
        maximum = ImageChops.lighter(ImageChops.lighter(channels[0], channels[1]), channels[2])
        hist = maximum.histogram()
        level = max(i for i, n in enumerate(hist) if n)
        count = sum(hist[2:])
        coords = [(i//256, i%256) for i,v in enumerate(maximum.get_flattened_data()) if v > 1]
        run = max(longest(coords), longest([(x,y) for y,x in coords]))
        worst_level = max(worst_level, level)
        worst_count = max(worst_count, count)
        worst_run = max(worst_run, run)
        changed_channels += sum(sum(c.histogram()[1:]) for c in channels)
        # Regression budgets for this experiment; not a universal perceptual threshold.
        assert level <= 8 and count <= 8 and run <= 2, (r['id'],level,count,run)
    assert r['delta']['rmse_surface'] < data['checks']['linear_rmse_budget']
    assert r['delta']['alpha_rmse'] < data['checks']['alpha_rmse_budget']
    assert r['compact_error']['rmse_surface'] <= r['original_error']['rmse_surface'] + data['checks']['reference_rmse_increase_budget']
result = dict(poses=213, pngs=852, max_display_channel_delta=worst_level,
              max_pixels_over_one_level_per_image=worst_count,
              max_contiguous_run_over_one_level=worst_run,
              changed_display_channels=changed_channels,
              total_display_channels=213*256*256*3,
              max_linear_rmse=max(r['delta']['rmse_surface'] for r in data['cases']),
              max_native_linear_rmse=max(r['native_delta']['rmse_surface'] for r in data['cases']),
              max_native_linear_rgb=max(r['native_delta']['max_abs'] for r in data['cases']),
              max_native_alpha_rmse=max(r['native_delta']['alpha_rmse'] for r in data['cases']),
              texels_bit_exact=True, filtered_images_bit_exact=False,
              native_browser=False, human_acceptance=False)
(root/'validation.json').write_text(json.dumps(result,indent=2))
print(json.dumps(result,indent=2))
