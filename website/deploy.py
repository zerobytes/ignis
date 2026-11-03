#!/usr/bin/env python3
"""Deploy a committed public static release to Firebase and GitHub Pages."""
import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parent
REPO = 'zerobytes/ignis'
BASES = ['https://ignis.web.app/', 'https://zerobytes.github.io/ignis/']


def run(args, **kw):
    return subprocess.check_output(args, **kw)


def api(path, body=None, method=None):
    args = ['gh', 'api', f'repos/{REPO}/{path}']
    if method:
        args += ['--method', method]
    if body is not None:
        args += ['--input', '-']
    return json.loads(run(args, input=json.dumps(body).encode() if body is not None else None))


def read_dist():
    files = {}
    for p in sorted((ROOT / 'dist').rglob('*')):
        if p.is_symlink():
            raise RuntimeError(f'Symlink not allowed in public root: {p}')
        if p.is_file() and not any(x.startswith('.') or x == 'node_modules' for x in p.relative_to(ROOT / 'dist').parts):
            files[p.relative_to(ROOT / 'dist').as_posix()] = p.read_bytes()
    return files


def prepare():
    files = read_dist()
    files.pop('version.json', None)
    hashes = {p: hashlib.sha256(b).hexdigest() for p, b in files.items()}
    version = hashlib.sha256(json.dumps(hashes, sort_keys=True).encode()).hexdigest()
    (ROOT / 'dist/version.json').write_text(json.dumps({'schema': 1, 'version': version}, indent=2) + '\n')
    print(f'Static release: {version}', flush=True)
    return read_dist(), version


def github_publish(files, version):
    # Public Git is the source of truth. Deploy only bytes already committed there;
    # never create remote-only source commits or push unrelated local work.
    head = api('git/ref/heads/main')['object']['sha']
    commit = api(f'git/commits/{head}')
    tree = api(f'git/trees/{commit["tree"]["sha"]}?recursive=1')
    if tree.get('truncated'):
        raise RuntimeError('Remote tree truncated; refusing partial synchronization')
    remote = {x['path']: x['sha'] for x in tree['tree'] if x['type'] == 'blob' and x['path'].startswith('website/dist/')}
    expected = {}
    for path, data in files.items():
        full = 'website/dist/' + path
        sha = hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()
        expected[full] = sha
    if remote != expected:
        differing = sorted(p for p in remote.keys() | expected.keys() if remote.get(p) != expected.get(p))
        raise RuntimeError('Website source differs from public main. Run --prepare, commit and push the intended website files first: ' + ', '.join(differing))
    print(f'Public static source matches {head} (release {version[:12]})', flush=True)
    subprocess.run(['gh', 'workflow', 'run', 'pages.yml', '--repo', REPO, '--ref', 'main'], check=True)
    return head


def wait_pages(head):
    deadline = time.monotonic() + 600
    while time.monotonic() < deadline:
        runs = json.loads(run(['gh', 'run', 'list', '--repo', REPO, '--workflow', 'pages.yml', '--commit', head, '--json', 'status,conclusion,url']))
        if runs and runs[0]['status'] == 'completed':
            if runs[0]['conclusion'] != 'success':
                raise RuntimeError(f'Pages failed: {runs[0]["url"]}')
            print(f'GitHub Pages ready: {runs[0]["url"]}', flush=True)
            return
        time.sleep(5)
    raise RuntimeError('Pages deployment not confirmed within 10 minutes')


def firebase_publish(files):
    # Publish the captured bytes, even if another task edits the working tree.
    with tempfile.TemporaryDirectory(prefix='ignis-website-') as tmp:
        stage = Path(tmp)
        for path, data in files.items():
            out = stage / 'dist' / path
            out.parent.mkdir(parents=True, exist_ok=True)
            out.write_bytes(data)
        for name in ['firebase.json', '.firebaserc']:
            shutil.copyfile(ROOT / name, stage / name)
        subprocess.run(['npx', '-y', 'firebase-tools@latest', 'deploy', '--only', 'hosting:website', '--project', 'ignisengine', '--account', 'zerobyte30k@gmail.com', '--non-interactive'], cwd=stage, check=True)


def verify(files, version):
    def check(item):
        base, path, expected = item
        data = run(['curl', '--fail', '--silent', '--show-error', '--max-time', '30', base + path + '?release=' + version])
        if hashlib.sha256(data).digest() != hashlib.sha256(expected).digest():
            raise RuntimeError(f'Served content differs: {base}{path}')
    for base in BASES:
        deadline = time.monotonic() + 300
        while True:
            try:
                check((base, 'version.json', files['version.json']))
                with ThreadPoolExecutor(max_workers=6) as pool:
                    list(pool.map(check, [(base, p, b) for p, b in files.items()]))
                break
            except (RuntimeError, subprocess.CalledProcessError):
                if time.monotonic() >= deadline:
                    raise
                time.sleep(5)
        print(f'Verified {len(files)} served hashes: {base}', flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--prepare', action='store_true', help='Only generate deterministic version.json, without publishing')
    args = parser.parse_args()
    files, version = prepare()
    if args.prepare:
        return
    for path in files:
        if path.endswith('.js'):
            subprocess.run(['node', '--check', str(ROOT / 'dist' / path)], check=True)
    head = github_publish(files, version)
    wait_pages(head)
    firebase_publish(files)
    verify(files, version)
    print('Both public destinations serve the same release.', flush=True)


if __name__ == '__main__':
    main()
