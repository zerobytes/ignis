#!/usr/bin/env python3
"""Assemble hash-pinned large previews from Releases without storing them in Git."""
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import subprocess
from urllib.parse import urlsplit

MANIFEST = 'catalog/data/runtime-files.json'


def declarations(root):
    manifest = root / MANIFEST
    if not manifest.exists():
        return {}
    data = json.loads(manifest.read_text())
    if data.get('version') != 1 or not isinstance(data.get('files'), dict):
        raise RuntimeError('Unsupported runtime asset manifest')
    total = 0
    legacy_total = 0
    for name, item in data['files'].items():
        path = PurePosixPath(name)
        url = urlsplit(item['url'])
        native = re.fullmatch(r'catalog/data/meshes/[a-z0-9.-]+\.native-part-\d+\.igmpz',name) is not None
        if (str(path) != name or '..' in path.parts or '\\' in name
                or not name.startswith('catalog/data/meshes/') or not name.endswith('.igmpz')
                or url.scheme != 'https' or url.netloc != 'github.com' or url.query or url.fragment
                or not url.path.startswith('/zerobytes/ignis-catalog/releases/download/')
                or '/latest/' in url.path or not re.fullmatch('[a-f0-9]{64}', item['sha256'])
                or type(item['bytes']) is not int or not 0 < item['bytes'] <= (256 if native else 96) * 1024 * 1024):
            raise RuntimeError(f'Invalid runtime preview declaration: {name}')
        total += item['bytes']
        if not native: legacy_total += item['bytes']
    if legacy_total > 768 * 1024 * 1024 or total > 4 * 1024 * 1024 * 1024:
        raise RuntimeError('Runtime preview snapshot exceeds its legacy/native byte budget')
    return data['files']


def valid(path, item):
    if not path.is_file() or path.is_symlink() or path.stat().st_size != item['bytes']:
        return False
    digest = hashlib.sha256()
    with path.open('rb') as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b''):
            digest.update(chunk)
    return digest.hexdigest() == item['sha256']


def materialize(root):
    entries = declarations(root)
    for name, item in entries.items():
        path = root / name
        current = root
        for part in PurePosixPath(name).parts:
            current /= part
            if current.is_symlink():
                raise RuntimeError('Runtime previews cannot traverse symlinks')
        if valid(path, item):
            continue
        path.parent.mkdir(parents=True, exist_ok=True)
        temporary = path.with_name('.' + path.name + '.part')
        with temporary.open('xb'):
            pass  # Exclusive, stable staging; never overwrite another active fetch.
        try:
            subprocess.run(['curl', '--fail', '--location', '--silent', '--show-error', '--retry', '2',
                            '--max-time', '180', '--max-filesize', str(item['bytes']), '--proto', '=https', '--proto-redir', '=https',
                            item['url'], '--output', str(temporary)], check=True)
            if not valid(temporary, item):
                raise RuntimeError(f'Runtime preview hash/size mismatch: {name}')
            temporary.replace(path)
        finally:
            temporary.unlink(missing_ok=True)
    print(f'Runtime previews verified: {len(entries)}', flush=True)
    return entries


if __name__ == '__main__':
    import sys
    if '--manifest-only' in sys.argv:
        print(f'Validated shared runtime declarations: {len(declarations(Path(__file__).resolve().parent / "dist"))}')
    else:
        materialize(Path(__file__).resolve().parent / 'dist')
