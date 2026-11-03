#!/usr/bin/env python3
"""Editable lawn modules: emerald carpet, Japanese cushions and fine carpet."""
import argparse
import json
import math
from pathlib import Path

from groundcover_assets import Plant, ROOT, TAU, atomic_json, library as build_library

DEST = ROOT / 'assets/botany/lawns'
FAMILIES = [
    dict(id='esmeralda', name='Grama esmeralda', scientific_label='Zoysia japonica (reference label)',
         form='carpet', module_meters=.24, height=.045, width=.0032, blades=768,
         fresh_fraction=.14, green=[.13, .29, .035], accent=[.18, .36, .055]),
    dict(id='japonesa', name='Grama japonesa', scientific_label='Zoysia tenuifolia (reference label)',
         form='cushion', module_meters=.24, radius=.118, height=.090, rim_height=.035,
         width=.0009, blades=2048, fresh_fraction=.08,
         green=[.070, .19, .025], accent=[.105, .25, .035]),
]
# Preserve the same fine-leaf material and cross-section, while removing the
# radial height envelope and fan-shaped distribution of the cushion preset.
FAMILIES.append(dict(FAMILIES[1], id='japonesa_tapete', name='Grama japonesa em tapete',
                     form='fine_carpet', height=.040))


class LawnPlant(Plant):
    def build(self):
        f, r = self.f, self.rng
        count = f['blades']
        columns = max(d for d in range(1, math.isqrt(count) + 1) if count % d == 0)
        rows = count // columns
        height_scale = r.uniform(.92, 1.08)
        stretch = r.uniform(.90, 1.10)
        phase = r.uniform(0, TAU)
        for index in range(count):
            if f['form'] in ('carpet', 'fine_carpet'):
                # A stratified square can cover neighbouring modules without a
                # collection of isolated fountain-shaped tufts.
                x = ((index % columns + r.uniform(.10, .90)) / columns - .5) * f['module_meters']
                z = ((index // columns + r.uniform(.10, .90)) / rows - .5) * f['module_meters']
                angle = phase + r.uniform(-math.pi, math.pi)
                h = f['height'] * height_scale * r.uniform(.76, 1.14)
                if f['form'] == 'fine_carpet':
                    reach = r.uniform(.006, .018)
                    control_y, tip_y = h * .64, h
                    fold, thickness = .24, .00007
                else:
                    reach = r.uniform(.012, .034)
                    control_y, tip_y = h * 1.10, h * .87
                    fold, thickness = .14, .00012
            else:
                angle = r.uniform(0, TAU)
                radius = f['radius'] * math.sqrt(r.random())
                q = radius / f['radius']
                x, z = radius * math.cos(angle) * stretch, radius * math.sin(angle) / stretch
                envelope = max(0, 1 - q * q) ** .72
                h = (f['rim_height'] + (f['height'] - f['rim_height']) * envelope) * height_scale * r.uniform(.84, 1.12)
                angle += r.uniform(-.32, .32)
                reach = (.008 + .033 * q * q) * r.uniform(.75, 1.2)
                control_y, tip_y = h * .64, h
                fold, thickness = .24, .00007
            start = [x, -.008, z]
            control = [x + reach * .18 * math.cos(angle), control_y, z + reach * .18 * math.sin(angle)]
            tip = [x + reach * math.cos(angle), tip_y, z + reach * math.sin(angle)]
            roll = angle + r.uniform(-.55, .55)
            layer = 'fresh' if r.random() < f['fresh_fraction'] else 'green'
            self.leaf(layer, start, control, tip, f['width'] * r.uniform(.78, 1.14),
                      [-math.sin(roll), 0, math.cos(roll)], base=.10, tip=1.10, fold=fold)
            self.layers[layer][-1]['shape']['thickness'] = thickness
            self.layers[layer][-1]['shape']['cup'] = .012
        return self.layers


def library(seed=216731, variants=3):
    return build_library(seed, variants, families=FAMILIES, plant_class=LawnPlant, prefix_tag='ln')


def write_library(destination=DEST, seed=216731, variants=3):
    manifest, files = library(seed, variants)
    for value in [manifest, *files.values()]:
        json.dumps(value, allow_nan=False)
    for path, value in files.items():
        atomic_json(destination / path, value)
    atomic_json(destination / 'library.json', manifest)
    return manifest


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--seed', type=int, default=216731)
    p.add_argument('--variants', type=int, default=3)
    p.add_argument('--output', type=Path, default=DEST)
    args = p.parse_args()
    manifest = write_library(args.output, args.seed, args.variants)
    print(json.dumps(dict(families=len(manifest['families']), variants=len(manifest['entries']),
                         sources=sum(len(e['components']) for e in manifest['entries'])), ensure_ascii=False))


if __name__ == '__main__':
    main()
