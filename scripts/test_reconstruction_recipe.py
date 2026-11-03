"""Independent numerical checks for the generic recipe client."""
import math
import unittest
from reconstruction_recipe import compose, quat, rotate, endpoint_transform, front_projector, shell, expand
from reconstruction_structure import inspect_surface


def cube(w,h,d):
    p=[[-w/2,-h/2,-d/2],[w/2,-h/2,-d/2],[w/2,h/2,-d/2],[-w/2,h/2,-d/2],
       [-w/2,-h/2,d/2],[w/2,-h/2,d/2],[w/2,h/2,d/2],[-w/2,h/2,d/2]]
    return {'positions':p,'triangles':[[0,3,2],[0,2,1],[4,5,6],[4,6,7],[0,4,7],[0,7,3],
           [1,2,6],[1,6,5],[0,1,5],[0,5,4],[3,7,6],[3,6,2]]}


class RecipeTests(unittest.TestCase):
    def test_right_handed_transforms_and_frame_composition(self):
        q=quat([0,90,0]);p=rotate([0,0,1],q)
        self.assertAlmostEqual(p[0],1);self.assertAlmostEqual(p[2],0)
        p,q,s=compose(([1,2,3],quat([0,0,90]),2),([1,0,0],quat([0,0,0]),.5))
        self.assertAlmostEqual(p[0],1);self.assertAlmostEqual(p[1],4);self.assertAlmostEqual(s,1)
    def test_endpoint_orientation_including_antiparallel(self):
        for a,b in [([0,0,0],[2,0,0]),([1,3,4],[1,1,4]),([0,0,0],[1,2,3])]:
            p,q,h=endpoint_transform(a,b)
            for local,expected in [([0,-h/2,0],a),([0,h/2,0],b)]:
                v=rotate(local,q)
                self.assertLess(math.dist([p[k]+v[k] for k in range(3)],expected),1e-10)
        with self.assertRaises(ValueError):endpoint_transform([0,0,0],[0,0,0])
    def test_shell_is_closed_material_with_an_open_center(self):
        g=shell(cube(2,2,2),cube(1,2,1),2)
        report=inspect_surface(g)
        self.assertTrue(report['solid_preflight_passed'])
        self.assertAlmostEqual(report['components'][0]['signed_volume_diagnostic'],6)
        # Vertical ray through center sees no horizontal cap over the hole.
        project=front_projector({'positions':[[p[0],p[2],p[1]] for p in g['positions']], 'triangles':g['triangles']})
        self.assertIsNone(project(0,0))
        self.assertAlmostEqual(project(.75,0),1)
    def test_front_projection_uses_nearest_visible_world_surface(self):
        g={'positions':[[0,0,1],[1,0,3],[0,1,1],[0,0,4],[1,0,6],[0,1,4]],
           'triangles':[[0,1,2],[3,4,5]]}
        project=front_projector(g)
        self.assertAlmostEqual(project(.25,.25),4.5)
        self.assertIsNone(project(.9,.9))
        lo,hi=project.band(.25)
        self.assertAlmostEqual(lo,0);self.assertAlmostEqual(hi,.75)
    def test_leaf_base_anchor_and_source_are_data_driven(self):
        d={'sources':{},'parts':[],'chains':[{'id':'ornament','material':'m','radius':.01,
            'points':[[0,0,0],[0,1,0]],'leaves':[{'position':[0,.5,0],'dimensions':[.05,.2,.01],
                                                'rotation':[0,0,0],'anchor':'base'}]}]}
        sources,parts=expand(d)
        leaf=next(p for p in parts if 'leaf' in p['id'])
        self.assertAlmostEqual(leaf['position'][1],.6)
        self.assertEqual(len(sources),2)
        self.assertEqual(d['sources'],{})

if __name__=='__main__':unittest.main()
