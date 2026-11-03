"""Build-time mountain reference. Produces ordinary editable native mesh assets.

The defaults reproduce the approved landscape exactly. This finite parametric
ridge is an art recipe, not an erosion simulation or a continuous-world sampler.
"""
import math
from dataclasses import dataclass

from verify_materials import f32


@dataclass(frozen=True)
class RidgeShape:
    width_meters: float = 4000
    depth_meters: float = 1600
    height_meters: float = 850
    crest_wander: float = .13
    slope_exponent: float = 1.15
    folds_meters: float = 48

    def validate(self):
        values = vars(self)
        if not all(math.isfinite(v) for v in values.values()):
            raise ValueError('Ridge parameters must be finite')
        if not (1 <= self.width_meters <= 20000 and 1 <= self.depth_meters <= 20000
                and 0 < self.height_meters <= 10000 and 0 <= self.crest_wander <= .5
                and .25 <= self.slope_exponent <= 4 and 0 <= self.folds_meters <= 1000):
            raise ValueError('Ridge parameters exceed the recipe domain')


def ridge(name, phase, n=110, *, shape=None):
    shape = shape or RidgeShape()
    shape.validate()
    if (not math.isfinite(phase) or not 0 <= phase <= 10000
            or type(n) is not int or not 1 <= n <= 127):
        raise ValueError('Ridge needs phase0..10000 and1..127integer cells/axis')
    positions, triangles = [], []
    for row in range(n+1):
        v = 2*row/n-1
        for col in range(n+1):
            u = 2*col/n-1
            crest = shape.crest_wander*math.sin(u*6+phase)
            slope = max(0, 1-abs(v-crest))**shape.slope_exponent
            profile = .76 + .18*math.sin(u*5+phase) + .10*math.sin(u*13-phase) + .05*math.sin(u*31+phase)
            folds = shape.folds_meters*math.sin(u*47+v*19+phase)*math.sin(v*17-u*11)
            y = max(0, (shape.height_meters*profile+folds)*slope)
            positions.append([u*(shape.width_meters/2), y, v*(shape.depth_meters/2)])
    for row in range(n):
        for col in range(n):
            a = row*(n+1)+col
            b = a+n+1
            triangles.extend([[a,b,a+1],[a+1,b,b+1]])
    return f32(dict(id=name, geometry=dict(positions=positions,triangles=triangles),shading='smooth',
        material=dict(pattern=dict(kind='ridged',frequency=[.01,.015,.01],low=[.38,.43,.31],
            high=[1,.94,.80],bump=.04,warp=.45,octaves=4,seed=int(phase*100+17),contrast=1.4))))
