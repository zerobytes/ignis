"""Probe evaluator failure modes without modifying the E02 scene or its evidence.

These deliberately damaged photographs are metric fixtures, never 3D candidates.
Run with the bundled Python containing Pillow. Uses an isolated public MCP process.
"""

import argparse
import hashlib
import importlib.util
import json
from collections import Counter
from pathlib import Path
import shutil

from PIL import Image, ImageChops, ImageStat

PROJECT = Path(__file__).resolve().parents[1]
ROOT = PROJECT / 'artifacts/e02-fidelity-audit'
REFERENCE = PROJECT / 'docs/references/image-to-3d/e02.png'
EXPECTED = '4153a456f9fdf2e9e6f43328171692156db7563c2062654199e66c0c540882ad'
REGIONS = [
    {'id': 'entry-tree', 'x': 695, 'y': 505, 'width': 85, 'height': 155},
    {'id': 'floor-medallion', 'x': 565, 'y': 735, 'width': 225, 'height': 100},
]


def reconstruction_module():
    spec = importlib.util.spec_from_file_location('e02_recipe', PROJECT / 'scripts/reconstruct_e02.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main(binary=None):
    if sha(REFERENCE) != EXPECTED:
        raise ValueError('Unexpected reference; the selected regions belong to E02')
    ROOT.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(REFERENCE, ROOT / 'reference.png')
    with Image.open(REFERENCE) as source:
        original = source.convert('RGB')
    cases = [('identity', original.copy(), None)]
    for region in REGIONS:
        candidate = original.copy()
        x, y, w, h = [region[k] for k in ('x', 'y', 'width', 'height')]
        # Replace the entire existing detail ROI with its own mean color.
        # The rest of the reference is byte-identical: a deliberate adversarial control.
        mean = tuple(round(v) for v in ImageStat.Stat(original.crop((x, y, x+w, y+h))).mean)
        candidate.paste(mean, (x, y, x+w, y+h))
        cases.append(('erase-' + region['id'], candidate, region['id']))
    recipe = reconstruction_module()
    engine = recipe.Engine(root=ROOT, binary=binary or recipe.BINARY)
    observations = []
    try:
        for name, candidate, defect in cases:
            path = ROOT / (name + '.png')
            candidate.save(path)
            response = engine.command('compare_images', reference_path='reference.png',
                                      candidate_path=path.name, regions=REGIONS)
            (ROOT / (name + '.json')).write_text(json.dumps(response, indent=2) + '\n')
            metrics = response['report']['metrics']
            delta = ImageChops.difference(original, candidate)
            mae = sum(ImageStat.Stat(delta).mean) / (3 * 255)
            unchanged = sum(a == b for a, b in zip(original.get_flattened_data(), candidate.get_flattened_data()))
            unchanged /= original.width * original.height
            if abs(mae - metrics['rgb_mae']) > 1e-12:
                raise AssertionError('Independent byte MAE disagrees with native comparison')
            if defect:
                region = next(r for r in REGIONS if r['id'] == defect)
                x, y, w, h = [region[k] for k in ('x', 'y', 'width', 'height')]
                local = candidate.crop((x, y, x+w, y+h))
                if len(set(local.get_flattened_data())) != 1:
                    raise AssertionError('Defect fixture still contains detail')
            observations.append({
                'case': name, 'input_kind': 'photograph metric fixture; no 3D candidate',
                'candidate_sha256': sha(path), 'defect_region': defect,
                'global': metrics, 'regions': response['report']['regions'],
                'unchanged_pixel_fraction_independent': unchanged,
                'independent_rgb_mae': mae,
                'counterexample_only_not_fidelity': {
                    'one_minus_mae_at_least_098': 1-mae >= .98,
                    'ssim_at_least_098': metrics['ssim_luma'] >= .98,
                    'unchanged_pixels_at_least_098': unchanged >= .98,
                },
                'build_id': response.get('build_id'),
            })
    finally:
        engine.close()
    reviews = json.loads((PROJECT / 'docs/references/image-to-3d/e02-review.json').read_text())
    stages = {}
    for stage in ['mass', 'detail']:
        groups, columns, _ = recipe.build_house(stage, 2)
        stages[stage] = {
            'direct_vertices': sum(len(m.positions) for m in groups.values()),
            'direct_triangles': sum(len(m.triangles) for m in groups.values()),
            'direct_groups': len(groups), 'column_instances': len(columns),
            'scope': 'client geometry only; native column source adds 80 vertices',
        }
    comparison = json.loads((PROJECT / 'artifacts/house-e02/comparison.json').read_text())
    baseline = json.loads((PROJECT / 'artifacts/house-e02/baseline-comparison.json').read_text())
    report = {
        'version': 1, 'reference_sha256': EXPECTED, 'dimensions': list(original.size),
        'scope': 'Evaluator audit; no scene mutation, new render, or reconstruction acceptance',
        'cases': observations,
        'recipe_stages': stages,
        'review_status_counts': dict(Counter(r['status'] for r in reviews.values())),
        'reviews_with_pending_next_action': sum(bool(r.get('next_action')) for r in reviews.values()),
        'existing_e02_metrics': {
            'baseline': baseline['report']['metrics'], 'final': comparison['report']['metrics'],
            'scope': 'Previously captured evidence, not a new render in this audit',
        },
        'checks': {
            'source_hash_matches': sha(REFERENCE) == EXPECTED,
            'independent_mae_matches_native': True,
            'identity_has_exact_pixels': observations[0]['global']['exact_rgb_match'],
            'two_erased_rois_are_constant': True,
            'erased_roi_cases_pass_all_three_naive_global_098_rules': all(
                all(c['counterexample_only_not_fidelity'].values()) for c in observations[1:]),
        },
    }
    if not all(report['checks'].values()):
        raise AssertionError(report['checks'])
    (ROOT / 'report.json').write_text(json.dumps(report, indent=2, ensure_ascii=False) + '\n')
    target = PROJECT / 'docs/references/image-to-3d/e02-fidelity-audit.json'
    target.write_text(json.dumps(report, indent=2, ensure_ascii=False) + '\n')
    print(json.dumps({'report': str(target), 'checks': report['checks'], 'cases': [
        {'case': c['case'], 'ssim': c['global']['ssim_luma'], 'mae': c['global']['rgb_mae'],
         'unchanged': c['unchanged_pixel_fraction_independent'],
         'local_ssim': {r['region']['id']: r['metrics']['ssim_luma'] for r in c['regions']}}
        for c in observations]}, ensure_ascii=False))


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary')
    main(parser.parse_args().binary)
