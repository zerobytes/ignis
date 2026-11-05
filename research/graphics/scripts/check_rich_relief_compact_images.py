#!/usr/bin/env python3
"""Verify saved image inventory and trajectory returns; no timing work here."""
import hashlib
import json
from pathlib import Path
from PIL import Image

out = Path(__file__).resolve().parents[1] / 'artifacts/appearance-probe/rich-relief'
data = json.loads((out/'compact-results.json').read_text())
modes = ['implicit', 'geometric-grad', 'geometric-screen', 'geometric-packed']
verified = 0
returns = 0
for row in data['quality']:
    for mode in modes:
        name = f"compact-{row['width']}-{row['case']}-{row['frame']}-{mode}.png"
        with Image.open(out/name) as im:
            im.load()
            assert im.size == (row['width'], row['height']), name
            assert im.mode == 'RGB', name
        verified += 1
for width in [1728, 3456]:
    for case in range(5):
        for mode in modes:
            count = 32 if case == 4 else 8
            # Image equality is weaker than float equality: retain both findings.
            with Image.open(out/f'compact-{width}-{case}-0-{mode}.png') as x, Image.open(out/f'compact-{width}-{case}-{count}-{mode}.png') as y:
                equal = x.tobytes() == y.tobytes()
            expected = next(r for r in data['returns'] if r['width']==width and r['case']==case and r['mode']==mode)
            assert not expected['pass'] or equal
            returns += int(equal)
assert verified == 552
result = {'decoded_pngs': verified, 'equal_png_returns': returns, 'png_return_pairs':40,
          'input_sha256':hashlib.sha256((out/'compact-results.json').read_bytes()).hexdigest()}
(out/'compact-verification.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result))
