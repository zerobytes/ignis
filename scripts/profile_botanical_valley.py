#!/usr/bin/env python3
"""Link a diagnostic to an existing release library without rebuilding Ignis.

The standard portable alternative is cargo run --release --example
botanical_valley_cost -- artifacts/botanical-valley/performance/cost.json.
This helper keeps active shared-checkout compiler work out of our build path.
"""
import argparse
import ctypes
import hashlib
import json
import os
import shutil
import subprocess
import threading
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'artifacts/botanical-valley/performance'


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def competing_workers():
    text = subprocess.check_output(['ps', '-axo', 'pid=,pcpu=,comm='], text=True)
    rows = []
    for line in text.splitlines():
        row = line.strip().split(maxsplit=2)
        if len(row) == 3 and Path(row[2]).name in ('rustc', 'cargo-clippy', 'xctrace'):
            rows.append(dict(pid=int(row[0]), cpu_percent=float(row[1]), name=Path(row[2]).name))
    return rows


def build(library):
    deadline = time.monotonic() + 600
    while not library.is_file() or library.stat().st_size < 1_000_000:
        if time.monotonic() > deadline:
            raise RuntimeError('The requested compiled library is still unavailable')
        time.sleep(2)
    # A library may be atomically replaced by parallel Cargo. Freeze only this
    # one link input temporarily and record its actual bytes; never copy an app.
    snapshot = OUT / 'libignis-profile.rlib'
    raw = library.read_bytes()
    snapshot.write_bytes(raw)
    source = OUT / 'runner-source.rs'
    source.write_bytes((ROOT / 'examples/botanical_valley_cost.rs').read_bytes())
    provenance = dict(library=str(library.relative_to(ROOT)),
                      library_sha256=hashlib.sha256(raw).hexdigest(),
                      runner_source_sha256=digest(source),
                      native_binary_sha256=digest(ROOT / 'target/release/ignis'),
                      rustc=subprocess.check_output(['rustc', '--version'], text=True).strip())
    command = ['rustc', '--crate-name', 'botanical_valley_cost', '--edition=2024', '-C', 'opt-level=3', '-C', 'debuginfo=1',
               str(source), '-L',
               'dependency=' + str(ROOT / 'target/release/deps'), '--extern', 'ignis=' + str(snapshot)]
    dependencies = {}
    for crate in ('anyhow', 'serde_json', 'wgpu', 'pollster'):
        choices = list((ROOT / 'target/release/deps').glob('lib' + crate + '-*.rlib'))
        path = max(choices, key=lambda p: p.stat().st_mtime_ns)
        dependencies[crate] = dict(path=str(path.relative_to(ROOT)), sha256=digest(path))
        command += ['--extern', crate + '=' + str(path)]
    # Native dependencies already built by Cargo; no new package downloads.
    for crate in ('meshopt', 'ring'):
        choices = list((ROOT / 'target/release/build').glob(crate + '-*/out'))
        path = max(choices, key=lambda p: p.stat().st_mtime_ns)
        command += ['-L', 'native=' + str(path)]
    command += ['-o', str(OUT / 'profile-runner')]
    try:
        subprocess.run(command, cwd=ROOT, check=True)
        provenance.update(dependencies=dependencies, command=command,
                          runner_sha256=digest(OUT / 'profile-runner'))
        (OUT / 'link-provenance.json').write_text(json.dumps(provenance, indent=2) + '\n')
    finally:
        snapshot.unlink(missing_ok=True)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--library', type=Path)
    p.add_argument('--stage', choices=['build', 'cost', 'trace', 'export', 'confirm'], required=True)
    p.add_argument('--variant', choices=['full', 'no_shadows'], default='full')
    args = p.parse_args()
    OUT.mkdir(parents=True, exist_ok=True)
    if args.stage == 'build':
        if args.library is None:
            p.error('--library requires the completed release rlib selected for this measurement')
        build(args.library.resolve())
    elif args.stage == 'cost':
        before = digest(ROOT / 'artifacts/botanical-valley/world.scene.json')
        stop = threading.Event()
        activity = []
        def monitor():
            while not stop.is_set():
                text = subprocess.check_output(['ps', '-axo', 'pid=,pcpu=,comm='], text=True)
                compiler = []
                for line in text.splitlines():
                    row = line.strip().split(maxsplit=2)
                    if len(row) == 3 and Path(row[2]).name in ('rustc', 'cargo-clippy', 'xctrace'):
                        compiler.append(dict(pid=int(row[0]), cpu_percent=float(row[1]), name=Path(row[2]).name))
                activity.append(dict(unix_seconds=time.time(), compiler_or_trace=compiler))
                stop.wait(2)
        thread = threading.Thread(target=monitor, daemon=True)
        thread.start()
        try:
            subprocess.run([str(OUT / 'profile-runner'), str(OUT / 'cost.json')], cwd=ROOT, check=True)
        finally:
            stop.set()
            thread.join()
            (OUT / 'cost-concurrency.json').write_text(json.dumps(activity, indent=2))
        after = digest(ROOT / 'artifacts/botanical-valley/world.scene.json')
        if before != after:
            raise RuntimeError('Authored scene changed during profiling')
        (OUT / 'scene-integrity.json').write_text(json.dumps(dict(scene_sha256=before,
            source_preserved=True, source_bytes=(ROOT / 'artifacts/botanical-valley/world.scene.json').stat().st_size), indent=2))
    elif args.stage == 'export':
        for schema, filename in [('metal-command-buffer-completed', 'gpu-full-buffers.xml'),
                                 ('metal-gpu-intervals', 'gpu-full-intervals.xml'),
                                 ('time-profile', 'cpu-profile.xml')]:
            xpath = '/trace-toc/run[@number="1"]/data/table[@schema="' + schema + '"]'
            subprocess.run(['xcrun', 'xctrace', 'export', '--input', str(OUT / 'metal.trace'),
                '--xpath', xpath, '--output', str(OUT / filename)], cwd=ROOT, check=True)
    elif args.stage == 'confirm':
        activity = []
        deadline = time.monotonic() + 120
        while any(w['cpu_percent'] > 5 for w in competing_workers()):
            if time.monotonic() > deadline:
                raise RuntimeError('Concurrent compiler/trace activity did not subside for confirmation')
            time.sleep(1)
        (OUT / 'trace-go').touch()
        (OUT / 'trace-finish').touch()
        try:
            for variant in ('full', 'no_shadows'):
                process = subprocess.Popen([str(OUT / 'profile-runner'), str(OUT / ('confirm-' + variant + '.json')), variant], cwd=ROOT)
                while process.poll() is None:
                    activity.append(dict(unix_seconds=time.time(), variant=variant, workers=competing_workers()))
                    time.sleep(.5)
                if process.returncode:
                    raise RuntimeError('Confirmation runner failed')
        finally:
            (OUT / 'confirmation-concurrency.json').write_text(json.dumps(activity, indent=2))
            for file in ('trace-ready.json', 'trace-go', 'trace-finish'):
                (OUT / file).unlink(missing_ok=True)
    else:
        trace = OUT / 'metal.trace'
        if trace.exists():
            # One current trace; extract/export before replacing it for another case.
            shutil.rmtree(trace)
        for file in ('trace-ready.json', 'trace-go', 'trace-finish'):
            (OUT / file).unlink(missing_ok=True)
        with (OUT / 'trace-stdout.log').open('w') as log:
            runner = subprocess.Popen([str(OUT / 'profile-runner'),
                str(OUT / ('trace-' + args.variant + '.json')), args.variant], cwd=ROOT, stdout=log, stderr=log)
            recorder = None
            try:
                deadline = time.monotonic() + 120
                while not (OUT / 'trace-ready.json').exists():
                    if runner.poll() is not None:
                        raise RuntimeError('Trace runner exited before warm-up; inspect trace-stdout.log')
                    if time.monotonic() > deadline:
                        raise RuntimeError('Trace warm-up timeout')
                    time.sleep(.1)
                lib = ctypes.CDLL('/usr/lib/libSystem.B.dylib')
                token = ctypes.c_int()
                notification = ('dev.ignis.profile.' + str(os.getpid())).encode()
                if lib.notify_register_check(notification, ctypes.byref(token)):
                    raise RuntimeError('Cannot register Instruments start notification')
                value = ctypes.c_int()
                lib.notify_check(token, ctypes.byref(value))
                temporary = OUT / 'trace-tmp'
                temporary.mkdir(exist_ok=True)
                env = dict(os.environ, TMPDIR=str(temporary) + '/')
                recorder = subprocess.Popen(['xcrun', 'xctrace', 'record', '--template', 'Time Profiler',
                    '--instrument', 'GPU', '--time-limit', '3s', '--output', str(trace),
                    '--notify-tracing-started', notification.decode(), '--attach', str(runner.pid)], cwd=ROOT, env=env)
                deadline = time.monotonic() + 45
                while True:
                    lib.notify_check(token, ctypes.byref(value))
                    if value.value:
                        break
                    if recorder.poll() is not None or time.monotonic() > deadline:
                        raise RuntimeError('Instruments did not signal recording start')
                    time.sleep(.05)
                lib.notify_cancel(token)
                (OUT / 'trace-go').touch()
                if recorder.wait(timeout=240):
                    raise RuntimeError('Instruments failed to finish the GPU recording')
                (OUT / 'trace-finish').touch()
                if runner.wait(timeout=45):
                    raise RuntimeError('Trace runner did not finish cleanly')
            finally:
                if recorder is not None and recorder.poll() is None:
                    recorder.terminate()
                    recorder.wait(timeout=15)
                if runner.poll() is None:
                    runner.terminate()
                    runner.wait(timeout=15)


if __name__ == '__main__':
    main()
