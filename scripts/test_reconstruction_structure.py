"""Counterexamples for the structural preflight, including the rejected scene."""
import copy
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

from reconstruction_structure import inspect_surface, building_core_preflight

PROJECT = Path(__file__).resolve().parents[1]


def cube():
    return {'positions': [[0,0,0], [1,0,0], [1,1,0], [0,1,0],
                          [0,0,1], [1,0,1], [1,1,1], [0,1,1]],
            'triangles': [[0,3,2], [0,2,1], [4,5,6], [4,6,7],
                          [0,1,5], [0,5,4], [3,7,6], [3,6,2],
                          [0,4,7], [0,7,3], [1,2,6], [1,6,5]]}


class SurfaceTests(unittest.TestCase):
    def test_outward_cube_volume_and_purity(self):
        geometry = cube()
        before = copy.deepcopy(geometry)
        report = inspect_surface(geometry)
        self.assertTrue(report['solid_preflight_passed'])
        self.assertAlmostEqual(report['components'][0]['signed_volume_diagnostic'], 1)
        self.assertEqual(geometry, before)
        self.assertEqual(report, inspect_surface(geometry))

    def test_exact_seams_are_not_open_edges(self):
        source = cube()
        geometry = {'positions': [source['positions'][v] for t in source['triangles'] for v in t],
                    'triangles': [[3*i,3*i+1,3*i+2] for i in range(12)]}
        report = inspect_surface(geometry)
        self.assertTrue(report['solid_preflight_passed'])
        self.assertEqual(report['coincident_positions'], 28)

    def test_near_seams_are_not_silently_repaired(self):
        geometry = cube()
        geometry['positions'].append([0,0,1e-10])
        geometry['triangles'][0][0] = 8
        self.assertIn('open_boundary', inspect_surface(geometry)['defects'])

    def test_missing_face_fails(self):
        geometry = cube()
        geometry['triangles'].pop()
        self.assertEqual(inspect_surface(geometry)['boundary_edges'], 3)
        self.assertFalse(inspect_surface(geometry)['solid_preflight_passed'])

    def test_flipped_face_fails(self):
        geometry = cube()
        geometry['triangles'][0].reverse()
        self.assertIn('inconsistent_orientation', inspect_surface(geometry)['defects'])

    def test_inward_closed_mesh_fails(self):
        geometry = cube()
        geometry['triangles'] = [list(reversed(t)) for t in geometry['triangles']]
        self.assertIn('zero_or_inward_volume', inspect_surface(geometry)['defects'])

    def test_non_manifold_edge_fails(self):
        geometry = cube()
        geometry['triangles'].append(geometry['triangles'][0][:])
        self.assertEqual(inspect_surface(geometry)['non_manifold_edges'], 3)

    def test_disconnected_solids_do_not_approve_one_body(self):
        geometry = cube()
        other = cube()
        geometry['positions'] += [[x+3,y,z] for x,y,z in other['positions']]
        geometry['triangles'] += [[v+8 for v in t] for t in other['triangles']]
        report = inspect_surface(geometry)
        self.assertEqual(len(report['components']), 2)
        self.assertIn('multiple_components', report['defects'])

    def test_flat_double_surface_has_zero_volume(self):
        geometry = {'positions': [[0,0,0], [1,0,0], [0,1,0]], 'triangles': [[0,1,2], [2,1,0]]}
        report = inspect_surface(geometry)
        self.assertEqual(report['boundary_edges'], 0)
        self.assertIn('zero_or_inward_volume', report['defects'])

    def test_degenerate_and_isolated_vertices_fail(self):
        geometry = cube()
        geometry['positions'].append([8,8,8])
        geometry['triangles'].append([0,0,1])
        report = inspect_surface(geometry)
        self.assertIn('isolated_vertex', report['defects'])
        self.assertIn('degenerate_face', report['defects'])

    def test_volume_translation_and_scale(self):
        for scale in (1e-4, 1, 100):
            geometry = cube()
            geometry['positions'] = [[500000+x*scale, 400000+y*scale, -500000+z*scale]
                                     for x,y,z in geometry['positions']]
            report = inspect_surface(geometry)
            self.assertTrue(report['solid_preflight_passed'])
            self.assertAlmostEqual(report['components'][0]['signed_volume_diagnostic']/scale**3, 1, places=5)

    def test_malformed_data_is_not_accepted(self):
        cases = [([], []), ([[float('nan'),0,0]], [[0,0,0]]),
                 ([[True,0,0]], [[0,0,0]]), ([[0,0,0]], [[0,0,8]]),
                 ([[0,0,0]], [[False,0,0]]), ([[0,0]], [[0,0,0]]),
                 ([[0,0,0]]*8193, [[0,0,0]])]
        for positions, triangles in cases:
            with self.subTest(positions=len(positions)), self.assertRaises(ValueError):
                inspect_surface({'positions': positions, 'triangles': triangles})

    def test_e02_actual_assets_cannot_be_solid_body(self):
        scene = json.loads((PROJECT/'examples/house-e02-rebuild.scene.json').read_text())
        assets = {a['id']:a for a in scene['meshes']['assets']}
        for key in ('rebuild-stone', 'rebuild-roof'):
            report = inspect_surface(assets[key]['geometry'])
            self.assertFalse(report['solid_preflight_passed'])
            self.assertGreater(report['boundary_edges'], 0)
        self.assertNotIn('building-core', assets)

    def test_optional_open_ornament_does_not_fail_solid_core(self):
        from types import SimpleNamespace
        geometry = cube()
        meshes = {'building-core': SimpleNamespace(p=geometry['positions'], t=geometry['triangles']),
                  'leaf': SimpleNamespace(p=[[0,0,0]], t=[])}
        self.assertTrue(building_core_preflight(meshes)['solid_preflight_passed'])

    def test_legacy_publisher_refuses_before_mcp_and_detail(self):
        import e02_rebuild
        with patch.object(e02_rebuild, 'build', return_value=({}, [], {}, [], {})) as build, \
                patch.object(e02_rebuild, 'Engine') as engine, patch.object(e02_rebuild, 'save'):
            with self.assertRaisesRegex(RuntimeError, 'missing_building_core'):
                e02_rebuild.publish('detail')
            engine.assert_not_called()
            build.assert_called_once_with('structure')

    def test_publisher_rechecks_detail_and_vertex_remapping(self):
        import e02_rebuild
        from types import SimpleNamespace
        def mesh(closed=True, corrupt_weld=False):
            geometry = cube()
            if not closed:
                geometry['triangles'].pop()
            value = SimpleNamespace(p=geometry['positions'], t=geometry['triangles'])
            value.weld = (lambda: value.t.pop()) if corrupt_weld else lambda: None
            return {'building-core': value}
        for detailed, remapped in [(False, False), (True, False), (False, True)]:
            stage = 'detail' if detailed else 'structure'
            values = [(mesh(closed=detailed or remapped, corrupt_weld=remapped), [], {}, [], {})]
            if detailed:
                values.append((mesh(closed=False), [], {}, [], {}))
            with self.subTest(stage=stage, remapped=remapped), \
                    patch.object(e02_rebuild, 'build', side_effect=values), \
                    patch.object(e02_rebuild, 'Engine') as engine, patch.object(e02_rebuild, 'save'):
                with self.assertRaisesRegex(RuntimeError, 'structure rejected'):
                    e02_rebuild.publish(stage)
                engine.assert_not_called()

    def test_actual_structure_recipe_is_blocked_without_touching_outputs(self):
        import e02_rebuild
        with tempfile.TemporaryDirectory() as directory, \
                patch.object(e02_rebuild, 'ROOT', Path(directory)), \
                patch.object(e02_rebuild, 'Engine') as engine:
            with self.assertRaisesRegex(RuntimeError, 'missing_building_core'):
                e02_rebuild.publish('detail')
            engine.assert_not_called()
            report = json.loads((Path(directory)/'structural-preflight.json').read_text())
            self.assertFalse(report['solid_preflight_passed'])
            self.assertFalse((Path(directory)/'detail.png').exists())
            self.assertFalse((Path(directory)/'house.scene.json').exists())

    def test_cli_required_missing_and_malformed(self):
        script = PROJECT/'scripts/reconstruction_structure.py'
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)/'geometry.json'
            path.write_text(json.dumps(cube()))
            ok = subprocess.run([sys.executable, str(script), str(path), '--require-solid', 'geometry'],
                                capture_output=True, text=True, check=False)
            self.assertEqual(ok.returncode, 0, ok.stdout+ok.stderr)
            self.assertEqual(json.loads(ok.stdout)['semantic_acceptance'], 'not_evaluated')
            missing = subprocess.run([sys.executable, str(script), str(path), '--require-solid', 'missing'],
                                     capture_output=True, text=True, check=False)
            self.assertEqual(missing.returncode, 1)
            path.write_text('{bad')
            bad = subprocess.run([sys.executable, str(script), str(path)], capture_output=True, text=True, check=False)
            self.assertEqual(bad.returncode, 2)


if __name__ == '__main__':
    unittest.main()
