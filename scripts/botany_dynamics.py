"""Coupled reduced bending/twist reference, implicit integration and stable IDs.

Rest Jacobians and mass matrix are linearized. Pose reconstruction preserves
segment lengths; this is not a nonlinear continuum/contact/fracture solver.
"""
from dataclasses import asdict, dataclass, replace
import hashlib
import json
import math

import botany_math as b
from botany_ecology import descendants
from botany_surfaces import SolidAxis

DYNAMICS_VERSION = "botany-reduced-1"


def matvec(matrix, vector):
    return [sum(a*x for a, x in zip(row, vector)) for row in matrix]


def cholesky(matrix):
    n = len(matrix)
    result = [[0.]*n for _ in range(n)]
    for i in range(n):
        for j in range(i+1):
            value = matrix[i][j]-sum(result[i][k]*result[j][k] for k in range(j))
            if i == j:
                b.require(math.isfinite(value) and value > 0, "nonpositive physical system")
                result[i][j] = math.sqrt(value)
            else:
                result[i][j] = value/result[j][j]
    return result


def solve(factor, rhs):
    n = len(rhs)
    y, x = [0.]*n, [0.]*n
    for i in range(n):
        y[i] = (rhs[i]-sum(factor[i][j]*y[j] for j in range(i)))/factor[i][i]
    for i in range(n-1, -1, -1):
        x[i] = (y[i]-sum(factor[j][i]*x[j] for j in range(i+1, n)))/factor[i][i]
    return x


def map_frame(vector, source, target):
    return b.add(b.add(b.mul(target.t, b.dot(vector, source.t)),
                       b.mul(target.n, b.dot(vector, source.n))),
                 b.mul(target.b, b.dot(vector, source.b)))


@dataclass(frozen=True)
class Particle:
    organ: str
    u: float
    v: float
    offset: b.Vec   # local transverse offsets encoded as (N,B,0)
    mass: float
    area: float
    normal: b.Vec
    leaf: bool


@dataclass(frozen=True)
class DynamicState:
    model: str
    q: tuple[float, ...]
    velocity: tuple[float, ...]
    plastic: tuple[float, ...]
    broken: tuple[str, ...] = ()
    tick: int = 0


@dataclass(frozen=True)
class Response:
    damping_mass: float = .5       # Rayleigh alpha, 1/s
    damping_stiffness: float = .005  # Rayleigh beta, s
    yield_strain: float = .02
    fracture_strain: float = .08
    max_angle: float = .65


class ReducedModel:
    def __init__(self, plant, response=Response(), include_blades=False,
                 max_dofs=96, max_particles=2048):
        b.require(all(math.isfinite(x) for x in asdict(response).values())
                  and response.damping_mass >= 0 and response.damping_stiffness >= 0
                  and 0 < response.yield_strain < response.fracture_strain
                  and 0 < response.max_angle <= 1., "invalid constitutive response")
        self.plant, self.response = plant, response
        self.ordered = tuple(sorted(plant.organs, key=lambda o: (o.level, o.id)))
        self.lookup = {o.id: o for o in plant.organs}
        self.dof_organs = tuple(o.id for o in self.ordered
            if not isinstance(plant.recipe.definitions[o.definition], SolidAxis)
            and (include_blades or o.width == 0))
        self.ndof = 3*len(self.dof_organs)
        b.require(0 < self.ndof <= max_dofs and len(plant.organs) <= 256,
                  "reduced model DOF/organ budget exceeded")
        b.require(2*self.ndof*sum(len(o.samples) for o in plant.organs) <= 5_000_000,
                  "physical Jacobian preparation budget exceeded")
        self.offsets = {identity: 3*i for i, identity in enumerate(self.dof_organs)}
        self.particles = []
        def append_particle(particle):
            b.require(len(self.particles) < max_particles, "reduced model particle budget exceeded")
            self.particles.append(particle)
        self.stiffness = []
        for organ in self.ordered:
            spec = plant.recipe.definitions[organ.definition]
            if isinstance(spec, b.Axis):
                for i in range(8):
                    u = (i+.5)/8
                    section = b.axis_section(organ, spec, u)
                    radius = b.radius_at(organ, spec, u)
                    mass = section["mass_per_length_kg_m"]*organ.length/32
                    for x, y in ((radius/math.sqrt(2), 0.), (-radius/math.sqrt(2), 0.),
                                 (0., radius*spec.ellipticity/math.sqrt(2)),
                                 (0., -radius*spec.ellipticity/math.sqrt(2))):
                        append_particle(Particle(organ.id, u, 0., (x, y, 0.), mass,
                            2*radius*organ.length/32, organ.sample(u).t, False))
            else:
                for i in range(4):
                    for j in range(4):
                        u, v, epsilon = (i+.5)/4, -1+2*(j+.5)/4, 1e-5
                        if hasattr(spec, "contains_uv") and not spec.contains_uv(u, v):
                            continue
                        du = b.mul(b.sub(b.blade_surface(organ, spec, u+epsilon, v),
                                         b.blade_surface(organ, spec, u-epsilon, v)), .5/epsilon)
                        dv = b.mul(b.sub(b.blade_surface(organ, spec, u, v+epsilon),
                                         b.blade_surface(organ, spec, u, v-epsilon)), .5/epsilon)
                        cross = b.cross(du, dv)
                        area = b.norm(cross)*2/16
                        append_particle(Particle(organ.id, u, v, (0., 0., 0.),
                            area*spec.thickness*spec.material.density, area, b.unit(cross), True))
                if hasattr(spec, "graph"):
                    for a, c, edge in spec.graph.segments():
                        u, v = spec.uv_from_xy((a[0]+c[0])/2, (a[1]+c[1])/2)
                        length = math.hypot((a[0]-c[0])*organ.width, (a[1]-c[1])*organ.length)
                        mass = 950.*math.pi*edge.radius**2*length
                        append_particle(Particle(organ.id, u, v, (0., 0., 0.), mass,
                                                       0., organ.sample(.5).b, True))
            if organ.id in self.offsets:
                stiffness = [0., 0., 0.]
                for i in range(32):
                    u = (i+.5)/32
                    if isinstance(spec, b.Axis):
                        section = b.axis_section(organ, spec, u)
                        ei = (section["EI_B_Nm2"], section["EI_N_Nm2"], section["GJ_approx_Nm2"])
                    else:
                        width = organ.width
                        ei = (spec.material.young*spec.thickness*width**3/12,
                              spec.material.young*width*spec.thickness**3/12,
                              spec.material.shear*width*spec.thickness**3/3)
                    for k in range(3):
                        derivative = 2*(1-u) if k < 2 else 1.
                        stiffness[k] += ei[k]*derivative**2/(32*organ.length)
                self.stiffness.extend(stiffness)
                if isinstance(spec, b.Blade) and hasattr(spec, "graph"):
                    reinforcement = spec.graph.mechanics(organ.length, organ.width)
                    extra = reinforcement["integrated_directional_EI_Nm3"][1][1]/organ.length**2
                    self.stiffness[-3] += 4*extra/3
                    self.stiffness[-2] += 4*extra/3
        b.require(len(self.particles) <= max_particles, "reduced model particle budget exceeded")
        data = dict(implementation=DYNAMICS_VERSION, recipe=asdict(plant.recipe),
                    definition_types={key:type(spec).__qualname__ for key, spec in plant.recipe.definitions.items()},
                    organs=[asdict(o) for o in plant.organs], response=asdict(response), dofs=self.dof_organs)
        self.fingerprint = hashlib.sha256(json.dumps(data, sort_keys=True, allow_nan=False,
                                                   separators=(",", ":")).encode()).hexdigest()
        zero = [0.]*self.ndof
        self.rest_positions = self.positions(self.pose(zero))
        self.jacobian = []
        epsilon = 1e-5
        for i in range(self.ndof):
            positive, negative = zero.copy(), zero.copy()
            positive[i], negative[i] = epsilon, -epsilon
            a, c = self.positions(self.pose(positive)), self.positions(self.pose(negative))
            self.jacobian.append(tuple(b.mul(b.sub(x, y), .5/epsilon) for x, y in zip(a, c)))
        self._cached = None  # exactly one current factorization, replaced on dt/break changes

    def initial(self):
        zeros = (0.,)*self.ndof
        return DynamicState(self.fingerprint, zeros, zeros, zeros)

    def validate(self, state):
        b.require(state.model == self.fingerprint and type(state.tick) is int and state.tick >= 0,
                  "physical state belongs to another model/version")
        b.require(all(len(row) == self.ndof and all(math.isfinite(x) for x in row)
                      for row in (state.q, state.velocity, state.plastic)), "corrupt physical state")
        b.require(len(set(state.broken)) == len(state.broken) and set(state.broken) <= set(self.lookup),
                  "invalid broken identities")
        for root in state.broken:
            b.require(descendants(self.plant, root) <= set(state.broken), "incomplete broken subtree")
        for identity, offset in self.offsets.items():
            if identity in state.broken:
                b.require(all(x == 0 for x in state.q[offset:offset+3]+state.velocity[offset:offset+3]),
                          "retired DOF is active")
            else:
                b.require(b.norm(state.q[offset:offset+3]) <= self.response.max_angle,
                          "reduced model outside angular validity domain")

    def pose(self, q):
        b.require(len(q) == self.ndof and all(math.isfinite(x) for x in q), "invalid pose coordinates")
        result = {}
        for organ in self.ordered:
            old = organ.samples[0]
            if organ.parent is None:
                parent_old, parent_new = old, old
                origin = old.p
            else:
                parent_old = self.lookup[organ.parent].sample(organ.insertion)
                parent_new = result[organ.parent].sample(organ.insertion)
                origin = parent_new.p
            root_t = map_frame(old.t, parent_old, parent_new)
            root_n = map_frame(old.n, parent_old, parent_new)
            root_b = b.cross(root_t, root_n)
            offset = self.offsets.get(organ.id)
            theta = q[offset:offset+3] if offset is not None else (0., 0., 0.)

            def rotation(u):
                f = 2*u-u*u
                return b.add(b.add(b.mul(root_b, theta[0]*f), b.mul(root_n, -theta[1]*f)),
                             b.mul(root_t, theta[2]*u))

            samples = []
            for i, sample in enumerate(organ.samples):
                u = i/(len(organ.samples)-1)
                if i:
                    difference = b.sub(sample.p, organ.samples[i-1].p)
                    difference = map_frame(difference, parent_old, parent_new)
                    origin = b.add(origin, b.rotate(difference, rotation((i-.5)/(len(organ.samples)-1))))
                t = b.unit(b.rotate(map_frame(sample.t, parent_old, parent_new), rotation(u)))
                n = b.normal_to(t, b.rotate(map_frame(sample.n, parent_old, parent_new), rotation(u)))
                samples.append(b.Sample(origin, t, n))
            result[organ.id] = replace(organ, samples=tuple(samples))
        return replace(self.plant, organs=tuple(result[o.id] for o in self.plant.organs))

    def positions(self, posed):
        lookup = {o.id: o for o in posed.organs}
        result = []
        for particle in self.particles:
            organ = lookup[particle.organ]
            spec = self.plant.recipe.definitions[organ.definition]
            if particle.leaf:
                point = b.blade_surface(organ, spec, particle.u, particle.v)
            else:
                sample = organ.sample(particle.u)
                point = b.add(sample.p, b.add(b.mul(sample.n, particle.offset[0]),
                                             b.mul(sample.b, particle.offset[1])))
            result.append(point)
        return tuple(result)

    def mass_matrix(self, broken=()):
        active = [i for i in range(self.ndof) if self.dof_organs[i//3] not in broken]
        matrix = [[sum(p.mass*b.dot(self.jacobian[i][k], self.jacobian[j][k])
                       for k, p in enumerate(self.particles) if p.organ not in broken)
                   for j in active] for i in active]
        return active, matrix

    def generalized_forces(self, state, gravity, wind, air_density=1.225, drag=1.1):
        force = [0.]*self.ndof
        for k, particle in enumerate(self.particles):
            if particle.organ in state.broken:
                continue
            velocity = (0., 0., 0.)
            for i in range(self.ndof):
                velocity = b.add(velocity, b.mul(self.jacobian[i][k], state.velocity[i]))
            relative = b.sub(wind, velocity)
            speed = b.norm(relative)
            projected = 0.
            if speed > 1e-12:
                cosine = b.dot(particle.normal, b.mul(relative, 1/speed))
                projected = abs(cosine) if particle.leaf else math.sqrt(max(0., 1-cosine*cosine))
            load = b.add(b.mul(gravity, particle.mass),
                         b.mul(relative, .5*air_density*drag*particle.area*projected*speed))
            for i in range(self.ndof):
                force[i] += b.dot(self.jacobian[i][k], load)
        return force

    def step(self, state, dt=1/120, gravity=(0., -9.81, 0.), wind=(0., 0., 0.),
             external=None):
        self.validate(state)
        b.require(math.isfinite(dt) and 0 < dt <= .05
                  and all(math.isfinite(x) for x in gravity+wind), "invalid physical input")
        key = (dt, state.broken)
        if self._cached is None or self._cached[0] != key:
            active, mass = self.mass_matrix(state.broken)
            alpha, beta = self.response.damping_mass, self.response.damping_stiffness
            effective = [[(1+dt*alpha)*mass[i][j]+(dt*beta+dt*dt)*self.stiffness[a]*(i == j)
                          for j, _ in enumerate(active)] for i, a in enumerate(active)]
            self._cached = (key, active, mass, cholesky(effective))
        _, active, mass, factor = self._cached
        loads = self.generalized_forces(state, gravity, wind)
        if external is not None:
            b.require(len(external) == self.ndof and all(math.isfinite(x) for x in external),
                      "invalid generalized load")
            loads = [a+c for a, c in zip(loads, external)]
        previous_velocity = [state.velocity[i] for i in active]
        rhs = matvec(mass, previous_velocity)
        for i, dof in enumerate(active):
            rhs[i] += dt*(loads[dof]-self.stiffness[dof]*(state.q[dof]-state.plastic[dof]))
        velocity = [0.]*self.ndof
        q, plastic = list(state.q), list(state.plastic)
        for dof, value in zip(active, solve(factor, rhs)):
            velocity[dof] = value
            q[dof] += dt*value
        broken = set(state.broken)
        for identity, offset in self.offsets.items():
            if identity in broken:
                continue
            organ = self.lookup[identity]
            spec = self.plant.recipe.definitions[organ.definition]
            elastic = b.sub(tuple(q[offset:offset+3]), tuple(plastic[offset:offset+3]))
            radius = max(b.radius_at(organ, spec, i/32) for i in range(33)) if isinstance(spec, b.Axis) else spec.thickness/2
            strain = radius/organ.length*math.sqrt(4*(elastic[0]**2+elastic[1]**2)+elastic[2]**2)
            if strain >= self.response.fracture_strain:
                broken.update(descendants(self.plant, identity))
            elif strain > self.response.yield_strain:
                ratio = self.response.yield_strain/strain
                for j in range(3):
                    plastic[offset+j] = q[offset+j]-elastic[j]*ratio
        for identity, offset in self.offsets.items():
            if identity in broken:
                q[offset:offset+3], velocity[offset:offset+3] = [0.]*3, [0.]*3
        candidate = DynamicState(self.fingerprint, tuple(q), tuple(velocity), tuple(plastic),
                                 tuple(sorted(broken)), state.tick+1)
        self.validate(candidate)
        return candidate

    def energy(self, state):
        self.validate(state)
        active, mass = self.mass_matrix(state.broken)
        v = [state.velocity[i] for i in active]
        kinetic = .5*sum(a*c for a, c in zip(v, matvec(mass, v)))
        elastic = .5*sum(self.stiffness[i]*(state.q[i]-state.plastic[i])**2 for i in active)
        return kinetic+elastic

    def restore(self, value):
        b.require(set(value) == set(asdict(self.initial())), "unknown physical state fields")
        state = DynamicState(value["model"], tuple(value["q"]), tuple(value["velocity"]),
                             tuple(value["plastic"]), tuple(value["broken"]), value["tick"])
        self.validate(state)
        return state
