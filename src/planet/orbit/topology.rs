use super::*;
type Line = (usize, [i64; 2]);
pub(super) struct Registry {
    lines: BTreeMap<Line, BTreeSet<i64>>,
    levels: BTreeMap<Key, u8>,
    max: u8,
}
fn line(a: Key, b: Key) -> Line {
    let axis = (0..3).find(|i| a[*i] != b[*i]).unwrap();
    let other = [(axis + 1) % 3, (axis + 2) % 3];
    let pair = if other[0] < other[1] {
        other
    } else {
        [other[1], other[0]]
    };
    (axis, [a[pair[0]], a[pair[1]]])
}
fn on_line(l: Line, v: i64) -> Key {
    let mut key = [0; 3];
    let mut i = 0;
    for (a, k) in key.iter_mut().enumerate() {
        if a == l.0 {
            *k = v;
        } else {
            *k = l.1[i];
            i += 1;
        }
    }
    key
}
fn key(t: Tile, x: u32, y: u32, max: u8) -> Key {
    let shift = max - t.level;
    let denom = i64::from(N) << max;
    t.face.key(
        ((2 * (i64::from(t.xy[0]) * i64::from(N) + i64::from(x))) << shift) - denom,
        ((2 * (i64::from(t.xy[1]) * i64::from(N) + i64::from(y))) << shift) - denom,
        denom,
    )
}
fn corners(t: Tile, max: u8) -> [Key; 4] {
    [
        key(t, 0, 0, max),
        key(t, N, 0, max),
        key(t, N, N, max),
        key(t, 0, N, max),
    ]
}
impl Registry {
    pub fn new(leaves: &BTreeSet<Tile>, max: u8) -> Self {
        let mut r = Self {
            lines: Default::default(),
            levels: Default::default(),
            max,
        };
        for &t in leaves {
            let c = corners(t, max);
            for e in 0..4 {
                let a = c[e];
                let b = c[(e + 1) % 4];
                let l = line(a, b);
                for n in 0..=N {
                    let v = a[l.0] + (b[l.0] - a[l.0]) * i64::from(n) / i64::from(N);
                    let k = on_line(l, v);
                    r.lines.entry(l).or_default().insert(v);
                    r.levels
                        .entry(k)
                        .and_modify(|x| *x = (*x).max(t.level))
                        .or_insert(t.level);
                }
            }
        }
        r
    }
    fn between(&self, a: Key, b: Key) -> Vec<Key> {
        let l = line(a, b);
        let lo = a[l.0].min(b[l.0]);
        let hi = a[l.0].max(b[l.0]);
        let mut points: Vec<_> = self.lines[&l]
            .range(lo..=hi)
            .map(|v| on_line(l, *v))
            .collect();
        if a[l.0] > b[l.0] {
            points.reverse();
        }
        points
    }
    pub fn signature(&self, t: Tile) -> Vec<(Key, u8)> {
        let c = corners(t, self.max);
        let mut out = vec![];
        for e in 0..4 {
            for p in self.between(c[e], c[(e + 1) % 4]) {
                out.push((p, self.levels[&p]));
            }
        }
        out
    }
}
struct Builder<'a> {
    doc: &'a Document,
    sampler: Sampler<'a>,
    anchor: Anchor,
    unit: f64,
    max: u8,
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    coordinates: Vec<[f32; 3]>,
    high_low: Vec<crate::meshes::SplitPosition>,
    indices: BTreeMap<Key, u32>,
    cast_error: f64,
}
impl Builder<'_> {
    fn vertex(&mut self, k: Key, level: u8) -> u32 {
        if let Some(&i) = self.indices.get(&k) {
            return i;
        }
        let d = DVec3::new(k[0] as f64, k[1] as f64, k[2] as f64).normalize();
        let h = self.sampler.height(d);
        let local = self.anchor.project(d, h) / self.unit;
        let p = local.as_vec3();
        self.cast_error = self
            .cast_error
            .max((local - p.as_dvec3()).length() * self.unit);
        let finest = 0.25f64.min(self.doc.radius() * 0.0001);
        let step = if level == self.max {
            finest
        } else {
            finest.max(self.doc.radius() / (f64::from(N) * (1u64 << level) as f64))
        };
        let normal = normal(&self.sampler, self.doc.radius(), d, step);
        let index = self.positions.len() as u32;
        self.indices.insert(k, index);
        self.positions.push(p.to_array());
        self.normals.push(normal.as_vec3().to_array());
        self.high_low.push(crate::meshes::SplitPosition::new(
            d * (1. + h / self.doc.radius()),
        ));
        self.coordinates.push((d * 4096.).as_vec3().to_array());
        index
    }
}
fn normal(s: &Sampler<'_>, r: f64, d: DVec3, step: f64) -> DVec3 {
    let reference = if d.z.abs() < 0.9 { DVec3::Z } else { DVec3::X };
    let a = reference.cross(d).normalize();
    let b = d.cross(a);
    let gradient = |v: DVec3| {
        (s.height((d + v * (step / r)).normalize()) - s.height((d - v * (step / r)).normalize()))
            / (2. * step)
    };
    (d - r / (r + s.height(d)) * (a * gradient(a) + b * gradient(b))).normalize()
}
pub(super) fn resource(
    doc: &Document,
    config: &super::super::runtime::Config,
    k: &ResourceKey,
    registry: &Registry,
    max: u8,
) -> Result<Resource> {
    // Bound the seven height evaluations per vertex before allocating/cooking.
    // A fine boundary may add many vertices to an otherwise coarse cell.
    doc.budget((((N + 1) * (N + 1)) as usize + k.signature.len()) * 7)?;
    let t = k.tile;
    let direction = t.direction([0.5, 0.5]);
    let sampler = Sampler::new(doc)?;
    let anchor = Anchor::new(
        doc,
        Pose {
            direction: direction.to_array(),
            forward: if direction.z.abs() < 0.9 {
                DVec3::Z.to_array()
            } else {
                DVec3::X.to_array()
            },
            altitude_meters: sampler.height(direction),
        },
    )?;
    let (_, cone) = t.cone();
    let span = doc.radius() * 2. * (cone * 0.5).sin() + doc.radius() * 0.5;
    // Power of two local units preserve the immutable bake when the view changes.
    let unit = 2f64.powf((span / 4096.).log2().ceil().max(0.));
    let mut b = Builder {
        doc,
        sampler,
        anchor,
        unit,
        max,
        positions: vec![],
        normals: vec![],
        coordinates: vec![],
        high_low: vec![],
        indices: Default::default(),
        cast_error: 0.,
    };
    let mut triangles = Vec::with_capacity((N * N * 2) as usize);
    let mut polygon = Vec::with_capacity(8);
    let mut vertices = Vec::with_capacity(8);
    for y in 0..N {
        for x in 0..N {
            let c = [
                key(t, x, y, max),
                key(t, x + 1, y, max),
                key(t, x + 1, y + 1, max),
                key(t, x, y + 1, max),
            ];
            polygon.clear();
            for e in 0..4 {
                let boundary = match e {
                    0 => y == 0,
                    1 => x + 1 == N,
                    2 => y + 1 == N,
                    _ => x == 0,
                };
                if boundary {
                    let points = registry.between(c[e], c[(e + 1) % 4]);
                    polygon.extend(points[..points.len() - 1].iter().copied());
                } else {
                    polygon.push(c[e]);
                }
            }
            vertices.clear();
            vertices.extend(
                polygon
                    .iter()
                    .map(|p| b.vertex(*p, registry.levels.get(p).copied().unwrap_or(t.level))),
            );
            if vertices.len() == 4 {
                triangles.extend([
                    [vertices[0], vertices[1], vertices[3]],
                    [vertices[1], vertices[2], vertices[3]],
                ]);
            } else {
                let middle = std::array::from_fn(|i| (c[0][i] + c[2][i]) / 2);
                let center = b.vertex(middle, t.level);
                for i in 0..vertices.len() {
                    triangles.push([center, vertices[i], vertices[(i + 1) % vertices.len()]]);
                }
            }
        }
    }
    let geometry = Geometry {
        positions: b.positions,
        triangles,
        material_coordinates: Some(b.coordinates),
    };
    geometry.validate()?;
    let mut material = config.material.clone();
    for p in [&mut material.pattern, &mut material.secondary_pattern]
        .into_iter()
        .flatten()
    {
        for f in &mut p.frequency {
            *f *= doc.radius() as f32 / 4096.;
        }
    }
    material.validate()?;
    Ok(Resource {
        coordinates: Some(Arc::new(b.high_low)),
        #[cfg(test)]
        keys: {
            let mut v = vec![[0; 3]; b.indices.len()];
            for (k, i) in b.indices {
                v[i as usize] = k;
            }
            v
        },
        asset: Asset {
            id: format!("orbit_{}", t.label()),
            geometry: Arc::new(geometry),
            shading: Shading::Smooth,
            material,
            rig: None,
            clips: vec![],
        },
        normals: Arc::new(b.normals),
        anchor,
        unit,
        cast_error: b.cast_error,
        bounds: global_bounds(doc, t),
        error: error(doc, t, max) + b.cast_error * 2.,
    })
}
pub(super) fn body(
    doc: &Document,
    anchor: Anchor,
    size: [f32; 3],
    _color: [f32; 3],
) -> Result<Resource> {
    let mut p = vec![];
    let mut n = vec![];
    let mut tri = vec![];
    let cells = 12;
    for axis in 0..3 {
        for sign in [-1., 1.] {
            let a = (axis + 1) % 3;
            let b = (axis + 2) % 3;
            let start = p.len() as u32;
            for y in 0..=cells {
                for x in 0..=cells {
                    let mut v = [0.; 3];
                    v[axis] = sign * size[axis] * 0.5;
                    v[a] = (x as f32 / cells as f32 - 0.5) * size[a];
                    v[b] = (y as f32 / cells as f32 - 0.5) * size[b];
                    p.push(v);
                    let mut norm = [0.; 3];
                    norm[axis] = sign;
                    n.push(norm);
                }
            }
            for y in 0..cells {
                for x in 0..cells {
                    let i = start + (y * (cells + 1) + x) as u32;
                    let j = i + (cells + 1) as u32;
                    if sign > 0. {
                        tri.extend([[i, i + 1, j], [i + 1, j + 1, j]]);
                    } else {
                        tri.extend([[i, j, i + 1], [i + 1, j, j + 1]]);
                    }
                }
            }
        }
    }
    let g = Geometry {
        positions: p,
        triangles: tri,
        material_coordinates: None,
    };
    g.validate()?;
    let _ = doc;
    Ok(Resource {
        coordinates: None,
        #[cfg(test)]
        keys: vec![],
        asset: Asset {
            id: "orbit_avatar".into(),
            geometry: Arc::new(g),
            shading: Shading::Smooth,
            material: Default::default(),
            rig: None,
            clips: vec![],
        },
        normals: Arc::new(n),
        anchor,
        unit: 1.,
        cast_error: 0.,
        bounds: [DVec3::ZERO; 8],
        error: 0.,
    })
}
