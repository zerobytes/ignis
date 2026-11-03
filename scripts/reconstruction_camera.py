"""Offline landmark camera fitting; no scene mutation or likeness claim.

The numerical core is the Apache-2.0 img2threejs subset in vendor/. The exact
camera has an explicit up vector. Ignis currently fixes world-up, so a rolled
solution is reported as incompatible rather than silently losing its roll.
"""
from __future__ import annotations

import argparse
from dataclasses import asdict
import json
import math
from pathlib import Path

from vendor.img2threejs import camera_fitting_types as types
from vendor.img2threejs import camera_fitting_math as projection
from vendor.img2threejs import camera_fitting_solver as solver

MAX_POINTS = 256


def view_camera(camera):
    """Invert upstream pitch→yaw→roll into right-handed world forward/up."""
    pitch, yaw, roll = map(math.radians, (camera.pitch_degrees, camera.yaw_degrees, camera.roll_degrees))
    def inverse(v):
        x, y, z = v
        x, y = math.cos(roll)*x+math.sin(roll)*y, -math.sin(roll)*x+math.cos(roll)*y
        x, z = math.cos(yaw)*x-math.sin(yaw)*z, math.sin(yaw)*x+math.cos(yaw)*z
        y, z = math.cos(pitch)*y+math.sin(pitch)*z, -math.sin(pitch)*y+math.cos(pitch)*z
        return [x, y, -z]
    forward, up = inverse((0, 0, 1)), inverse((0, 1, 0))
    position = list(camera.position)
    target = [a+b for a, b in zip(position, forward)]
    # A native look-at uses projected world Y, whereas Three.js can use exact up.
    y_dot = forward[1]
    native_up = [-forward[0]*y_dot, 1-forward[1]*y_dot, -forward[2]*y_dot]
    length = math.sqrt(sum(v*v for v in native_up))
    compatible = length > .01 and sum((a-b/length)**2 for a, b in zip(up, native_up)) < 1e-12
    descriptor = {'position': position, 'target': target, 'fov_degrees': camera.fov_degrees}
    return {'exact_camera': {**descriptor, 'up': up}, 'ignis_camera': descriptor if compatible else None,
            'ignis_compatible': compatible,
            'compatibility_note': 'world-up look-at is exact' if compatible else
                'Exact projection requires camera roll/up unsupported by current Ignis Camera; do not drop up.'}


def fit(payload):
    rows = payload['correspondences']
    if not isinstance(rows, list) or not 6 <= len(rows) <= MAX_POINTS:
        raise ValueError('expected 6..256 named correspondences')
    names = [row['name'] for row in rows]
    if any(not isinstance(n, str) or not n.strip() for n in names) or len(set(names)) != len(names):
        raise ValueError('landmark names must be nonempty and unique')
    camera = types.CameraParameters(**payload['initial_camera'])
    points = [types.NormalizedCorrespondence(row['name'], row['world'], row['observed']) for row in rows]
    normalized = types.normalize_fit_input(points, camera)
    points, camera = normalized.correspondences, normalized.initial_camera
    if projection.has_degenerate_world_geometry(points):
        raise ValueError('collinear/coincident 3D landmarks do not constrain this fit')
    initial = projection.residual_components(points, camera)
    if initial is None:
        raise ValueError('initial camera cannot project all landmarks')
    limits = types.SolverLimits(types.DEFAULT_MAXIMUM_ITERATIONS, types.DEFAULT_MAXIMUM_DAMPING_RETRIES,
        types.INITIAL_DAMPING, types.MAXIMUM_DAMPING, types.CONVERGED_RMS_PIXELS, types.FINITE_DIFFERENCE_STEPS)
    result = solver.fit_parameters(points, camera, limits)
    final = projection.residual_components(points, result.parameters)
    if final is None:
        raise ValueError('solver emitted an invalid projection')
    return {'version': 1, 'source': 'img2threejs@3e7099e1b7fe93e187e06db4bdbe1e1906c21a28',
            'parameters': asdict(result.parameters), **view_camera(result.parameters),
            'initial_rms_pixels': projection.rms_reprojection_error(initial),
            'final_rms_pixels': projection.rms_reprojection_error(final),
            'status': result.status, 'iterations': result.iterations,
            'accepted_steps': result.accepted_steps, 'rejected_steps': result.rejected_steps,
            'residuals': [{'name': p.name, 'projected': projection.project_landmark(p.world, result.parameters),
                           'error_pixels': math.hypot(final[2*i], final[2*i+1])} for i, p in enumerate(points)],
            'limitations': ['Landmarks and 3D shape supplied by caller; neither is inferred here.',
                            'RMS is reprojection error, not reconstruction accuracy or statistical confidence.',
                            'A stalled or max-iterations result remains diagnostic, not approved.']}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('input', type=Path)
    parser.add_argument('--out', type=Path)
    args = parser.parse_args()
    result = fit(json.loads(args.input.read_text()))
    text = json.dumps(result, indent=2, allow_nan=False)+'\n'
    if args.out:
        args.out.parent.mkdir(parents=True, exist_ok=True)
        args.out.write_text(text)
    else:
        print(text, end='')


if __name__ == '__main__':
    main()
