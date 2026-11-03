"""Section data derived from the shared tree seed20261026; no mesh generation.

Parallel XZ planes intentionally differ from tangent-oriented sweeps. This is
an editable native trunk study, not a replacement for the connected fork asset.
"""
import math


def center(t):
    return [0.18*math.sin(t*2.2)+0.10*t, 4.8*(t-0.5), 0.15*math.sin(t*3)]


def radius(t):
    return 0.46*(1-t*21/24)**1.25+0.015


def definition():
    diameter = 2*radius(0)
    at = [0, 0.08, 0.16, 0.24, 0.34, 0.42, 0.44, 0.59, 0.72, 0.86, 1]
    return {'id': 'tree-trunk',
            'profile': {'id': 'tree-trunk-profile', 'width': diameter, 'height': 4.8},
            'depth': diameter,
            'sections': {'id': 'tree-trunk-sections', 'cross_section': 'ellipse',
                         'stations': [{'id': f'trunk-{i:02}', 'at': t,
                                       'offset': [center(t)[0], center(t)[2]],
                                       'width': 2*radius(t)/diameter,
                                       'depth': 2*radius(t)/diameter}
                                      for i, t in enumerate(at)]},
            'segments': [8, 3, 8], 'subdivision_levels': 0,
            'sampling_tolerance': 0.001}
