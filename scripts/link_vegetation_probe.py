#!/usr/bin/env python3
"""Link one current renderer diagnostic to an existing, identified release rlib."""
import argparse
import hashlib
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--library', type=Path, required=True)
    parser.add_argument('--probe', choices=['batch', 'field'], default='batch')
    args = parser.parse_args()
    out = ROOT / 'artifacts/horizon-meadow'
    out.mkdir(parents=True, exist_ok=True)
    library = args.library.resolve()
    raw = library.read_bytes()
    frozen = out / 'libignis-probe.rlib'
    name = 'vegetation_batch_probe' if args.probe == 'batch' else 'horizon_meadow'
    executable = out / ('batch-probe' if args.probe == 'batch' else 'field-probe')
    source = ROOT / 'examples' / (name + '.rs')
    frozen.write_bytes(raw)
    command = ['rustc', '--crate-name', name, '--edition=2024',
               '-C', 'opt-level=3', str(source), '-L', f'dependency={ROOT}/target/release/deps',
               '--extern', f'ignis={frozen}']
    dependencies = {}
    for name in ('anyhow', 'serde_json', 'wgpu', 'pollster', 'glam'):
        path = max((ROOT / 'target/release/deps').glob(f'lib{name}-*.rlib'), key=lambda p: p.stat().st_mtime_ns)
        command.extend(['--extern', f'{name}={path}'])
        dependencies[name] = dict(path=str(path.relative_to(ROOT)), sha256=hashlib.sha256(path.read_bytes()).hexdigest())
    for name in ('meshopt', 'ring'):
        path = max((ROOT / 'target/release/build').glob(f'{name}-*/out'), key=lambda p: p.stat().st_mtime_ns)
        command.extend(['-L', f'native={path}'])
    command.extend(['-o', str(executable)])
    try:
        subprocess.run(command, cwd=ROOT, check=True)
        (out / (args.probe + '-link-provenance.json')).write_text(json.dumps(dict(
            library=str(library), library_sha256=hashlib.sha256(raw).hexdigest(),
            source_sha256=hashlib.sha256(source.read_bytes()).hexdigest(), dependencies=dependencies,
            compiler=subprocess.check_output(['rustc', '--version'], text=True).strip(),
            executable_sha256=hashlib.sha256(executable.read_bytes()).hexdigest()), indent=2))
    finally:
        frozen.unlink(missing_ok=True)


if __name__ == '__main__':
    main()
