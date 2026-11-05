#!/usr/bin/env python3
"""Verify the shared UI runtime; optionally open a temporary native MCP window."""
import argparse
import json
from pathlib import Path
import selectors
import subprocess
import time

ROOT = Path(__file__).resolve().parents[1]
BINARY = ROOT / 'target/release/examples/ui_workbench'


def native_mcp():
    process = subprocess.Popen([str(BINARY), '--mcp'], cwd=ROOT,
                               stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                               stderr=subprocess.PIPE, text=True, bufsize=1)
    counter = 0
    selector = selectors.DefaultSelector()
    selector.register(process.stdout, selectors.EVENT_READ)

    def rpc(method, params):
        nonlocal counter
        counter += 1
        process.stdin.write(json.dumps(dict(jsonrpc='2.0', id=counter, method=method, params=params)) + '\n')
        process.stdin.flush()
        deadline = time.monotonic() + 120
        while time.monotonic() < deadline:
            if selector.select(timeout=1):
                line = process.stdout.readline()
                if not line:
                    raise RuntimeError('Native UI MCP exited before reply')
                reply = json.loads(line)
                if reply.get('id') == counter:
                    if 'error' in reply:
                        raise RuntimeError(reply['error'])
                    return reply['result']
        raise TimeoutError('Native UI MCP timed out')

    def command(operation):
        reply = rpc('tools/call', dict(name='engine_command', arguments=dict(command=operation)))
        if reply.get('isError'):
            raise RuntimeError(reply)
        return reply['structuredContent']

    def ui(operation, revision=None):
        return command(dict(op='ui', operation=operation, expected_ui_revision=revision))

    try:
        rpc('initialize', dict(protocolVersion='2025-11-25', capabilities={},
                               clientInfo=dict(name='ignis-ui-verifier', version='1')))
        process.stdin.write(json.dumps(dict(jsonrpc='2.0', method='notifications/initialized')) + '\n')
        process.stdin.flush()
        initial = ui(dict(op='inspect'))
        node = initial['document']['root']['children'][0]
        node['text'] = dict(source='literal', value='Native MCP preview')
        receipt = ui(dict(op='edit', edits=[dict(op='replace_node', id=node['id'], node=node)]), initial['ui_revision'])
        current = ui(dict(op='inspect'))
        assert current['document']['root']['children'][0]['text']['value'] == 'Native MCP preview'
        ui(dict(op='undo'), receipt['ui_revision'])
        restored = ui(dict(op='inspect'))
        assert restored['document']['root']['children'][0]['text'] == dict(source='key', value='title')
        # The UI thread's state is observable after the same dispatcher commits.
        command(dict(op='exit'))
        process.wait(timeout=20)
        assert process.returncode == 0
        return dict(verified=True, native_window=True, transport='existing MCP stdio + event-loop mailbox',
                    checks=['initialize', 'inspect', 'guarded edit on native event-loop state', 'undo', 'exit'],
                    desktop_mouse_keyboard=False)
    finally:
        selector.close()
        if process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()
        for pipe in [process.stdin, process.stdout, process.stderr]:
            pipe.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--build', action='store_true', help='Replace the current example build first')
    parser.add_argument('--native-mcp', action='store_true', help='Also open a temporary neutral native window and verify its MCP session')
    args = parser.parse_args()
    if args.build:
        subprocess.run(['cargo', 'build', '--locked', '--release', '--example', 'ui_workbench'], cwd=ROOT, check=True)
    if not BINARY.is_file():
        parser.error('Build the example first or pass --build')
    subprocess.run([str(BINARY), '--verify'], cwd=ROOT, check=True)
    if args.native_mcp:
        result = native_mcp()
        path = ROOT / 'artifacts/ui-workbench/verification.json'
        report = json.loads(path.read_text())
        report['native_mcp'] = result
        path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
        print(json.dumps(result, ensure_ascii=False, indent=2))


if __name__ == '__main__':
    main()
