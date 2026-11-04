"""Seeded authorship of native contour/section data; Rust makes the mesh."""
import math
import random
from relief_compositions import ZERO, authored_precision


def tent(distance, extent, core=0):
    return max(0, min(1, (1-abs(distance)/extent)/(1-core)))


def document(recipe, detailed=True):
    rng = random.Random(recipe['seed'])
    n = recipe['profile_samples']
    height = recipe['height_meters']
    spacing = recipe['vertical_spacing_meters']
    banks = recipe['bank_boundaries_meters']
    if not 12 <= n <= 96 or not .5 <= spacing <= 4 or not 0 < height <= 1000:
        raise ValueError('Unsupported contour/section authoring resolution')
    if any(not math.isfinite(recipe[k]) or not .1 <= recipe[k] <= 1000
           for k in ['half_width_meters','half_depth_meters']):
        raise ValueError('Regional half dimensions need0.1..1000 metres')
    if not 2 <= len(banks) <= 9 or not 4 <= recipe['rock_spacing_meters'] <= 100:
        raise ValueError('Use1..8 banks and4..100m rock spacing')
    if banks[0] != 0 or banks[-1] != height or any(a >= b for a,b in zip(banks,banks[1:])):
        raise ValueError('Bank boundaries must strictly cover the chosen height')
    profile = []
    for i in range(n):
        a = math.tau*i/n
        radial = 1+.065*math.sin(3*a+.6)+.045*math.cos(5*a-.2)
        profile.append([recipe['half_width_meters']*math.copysign(abs(math.cos(a))**.64, math.cos(a))*radial,
                        recipe['half_depth_meters']*math.copysign(abs(math.sin(a))**.76, math.sin(a))*radial])
    lengths = [math.dist(profile[i], profile[(i+1)%n]) for i in range(n)]
    perimeter = sum(lengths)
    arclength = [sum(lengths[:i]) for i in range(n)]
    def distance(a,b):
        return (a-b+perimeter/2)%perimeter-perimeter/2
    cells = max(6, round(perimeter/recipe['rock_spacing_meters']))
    if cells > 64:
        raise ValueError('More than64 rock lenses per bank; increase spacing or split the source')
    lenses = []
    phases = []
    for j,(low,high) in enumerate(zip(banks,banks[1:])):
        phase = rng.uniform(0,math.tau)
        phases.append(phase)
        for k in range(cells):
            lenses.append(((k+rng.uniform(-.25,.25)+j*.42)*perimeter/cells,
                (low+high)/2+rng.uniform(-1.6,1.6),
                perimeter/cells*rng.uniform(.5,.82), (high-low)*rng.uniform(.5,.8),
                rng.uniform(*recipe['projection_meters']), rng.uniform(-.16,.16)))
    faults = [(k*perimeter/11+rng.uniform(-4,4),rng.uniform(0,math.tau)) for k in range(11)]
    front = arclength[max(range(n),key=lambda i:profile[i][1])]
    levels = {0, height}
    for i in range(1,math.ceil(height/spacing)):
        levels.add(min(height,i*spacing))
    for y in banks[1:-1]:
        for delta in [-.45,.25]:
            if 0 < y+delta < height: levels.add(y+delta)
    if len(levels)>128: raise ValueError('Too many native section levels')
    if len(levels)*n*(len(lenses)+len(faults)+len(banks)) > 2_000_000:
        raise ValueError('Regional source authorship exceeds the cold work budget')
    sections = []
    for y in sorted(levels):
        factors = []
        for i,(p,s) in enumerate(zip(profile,arclength)):
            radial = math.hypot(*p)
            relief = 1.5*tent(y,4)
            if detailed:
                for center,cy,wx,wy,amplitude,tilt in lenses:
                    along = distance(s,center)
                    q = max(abs(along)/wx,abs(y-cy-along*tilt)/wy)
                    relief += amplitude*max(0,min(1,(1-q)/.65))
                for j,boundary in enumerate(banks[1:-1]):
                    shifted = boundary+2.6*math.sin(math.tau*s/perimeter*3+phases[j])
                    interrupted = .15+.85*max(0,math.cos(s*.055+phases[j]))
                    relief -= recipe['joint_depth_meters']*tent(y-shifted,1.1)*interrupted
                for center,phase in faults:
                    relief -= 2.1*tent(distance(s,center+2.4*math.sin(y*.18+phase)),5.5)*tent(y-height*.5,height*.6)
                relief += recipe['front_projection_meters']*tent(distance(s,front+5),52,.15)*tent(y-height*.77,8,.2)
                relief -= 2.2*tent(distance(s,front),45)*tent(y-height*.5,2.8)
                relief += .8*tent(y-height+1.4,2)*(.5+.5*math.sin(s*.14))
            factors.append(1+relief/radial)
        sections.append({'at':y,'offset':[0,0],'scale':1,'radial_scales':factors})
    return authored_precision({'version':2,'space_version':1,'id':'plateau_strata','revision':1,
        'origin':ZERO,'pieces':[{'id':'continuous_stratified_body','anchor':[0,0,0],
            'axis':[0,1,0],'profile_x':[1,0,0],'inner_scale':None,'profile':profile,'sections':sections}]})
