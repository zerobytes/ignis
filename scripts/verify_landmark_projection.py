#!/usr/bin/env python3
"""Exercise read-only projection diagnostics through the real MCP dispatcher."""
import argparse
import json
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary, window=False):
    c = Client(binary, window)
    call = c.call
    checks = []
    camera = dict(position=[0, 0, 5], target=[0, 0, 0], fov_degrees=90)
    points = [dict(id='origin', world=[0, 0, 0], reference_pixel=[397, 296]),
              dict(id='offscreen', world=[100, 0, 0], reference_pixel=[400, 300]),
              dict(id='behind', world=[0, 0, 6], reference_pixel=[1, 1])]
    def query(**args):
        return call('project_landmarks', **dict(width=800, height=600,
                    landmarks=points, camera=camera, **args))
    def capture(name):
        path = f'artifacts/projection-{name}.png'
        call('capture', path=path, width=800, height=600)
        return (ROOT / path).read_bytes()
    try:
        c.request('initialize', dict(protocolVersion='2025-11-25', capabilities={},
                  clientInfo=dict(name='landmark-projection-verifier', version='1')))
        c.request('notifications/initialized', notification=True)
        assert 'project_landmarks' in json.dumps(c.request('tools/list'))
        call('generate', recipe=dict(seed=31, width=20, depth=20, obstacles=0))
        call('step', ticks=7)
        state = call('inspect'); image = capture('before')
        report = query()
        assert (report['projectable'], report['in_viewport'], report['compared'], report['excluded']) == (2, 1, 2, 1)
        assert abs(report['samples'][0]['distance_pixels'] - 5) < 0.001
        assert report['samples'][1]['status'] == 'outside_viewport'
        assert report['samples'][2]['pixel'] is None
        assert report == query()
        assert call('inspect') == state and capture('after') == image
        checks.append('Public schema and hypothetical camera query preserve live state, revision, tick and pixels; offscreen residuals counted')
        for args in [dict(width=0), dict(height=16385), dict(landmarks=[]),
                     dict(landmarks=points + points), dict(landmarks=points * 44),
                     dict(landmarks=[dict(id='bad', world=[10001, 0, 0])]),
                     dict(landmarks=[dict(id='bad', world=[0, 0, 0], reference_pixel=[800, 0])]),
                     dict(camera=dict(position=[0, 0, 0], target=[0, 0, 0], fov_degrees=34))]:
            payload = dict(width=800, height=600, camera=camera, landmarks=points)
            payload.update(args)
            call('project_landmarks', **payload, expected_error=True)
        query(expected_revision=state['revision'] + 1, expected_error=True)
        assert call('inspect') == state
        checks.append('Invalid camera/viewport/point/correspondence/duplicate/budget and stale revision reject atomically')
        call('set_camera', camera=camera)
        default = call('project_landmarks', width=800, height=600, landmarks=points)
        assert default == report
        call('undo'); assert call('inspect')['scene']['camera'] == state['scene']['camera']
        call('redo'); assert call('inspect')['scene']['camera'] == camera
        # Query must not add history or consume the redo entry.
        call('undo'); query(); call('redo')
        assert call('inspect')['scene']['camera'] == camera
        checks.append('Scene camera default equals explicit camera; diagnostics preserve undo/redo history')
        call('save_scene', path='examples/projection.scene.json')
        call('step', ticks=12); live = capture('live')
        call('save_replay', path='artifacts/projection.replay.json')
        call('run_replay', path='artifacts/projection.replay.json')
        assert query() == report
        call('capture_replay', replay_path='artifacts/projection.replay.json', path='artifacts/projection-replay.png', width=800, height=600)
        assert (ROOT / 'artifacts/projection-replay.png').read_bytes() == live
        call('load_scene', path='examples/projection.scene.json'); call('reset')
        assert query() == report
        checks.append('Save/load/reset/replay retain projection results and exact live/replay captures without new persistent fields')
        result = dict(passed=True, checks=len(checks), build_id=call('inspect')['build_id'])
        (ROOT / 'artifacts/landmark-projection-verification.json').write_text(json.dumps(result, indent=2) + '\n')
        print(json.dumps(result))
    finally:
        c.close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--window', action='store_true')
    args = parser.parse_args()
    verify(args.binary, args.window)
