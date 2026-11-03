use super::*;
type Line = (usize, [i64; 2]);
pub(super) struct Registry {
    lines: BTreeMap<Line, BTreeSet<i64>>,
    levels: BTreeMap<Key, u8>,
    max: u8,
    n: u32,
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
fn key(t: Tile, x: u32, y: u32, max: u8, n: u32) -> Key {
    let shift = max - t.level;
    let denom = i64::from(n) << max;
    t.face.key(
        ((2 * (i64::from(t.xy[0]) * i64::from(n) + i64::from(x))) << shift) - denom,
        ((2 * (i64::from(t.xy[1]) * i64::from(n) + i64::from(y))) << shift) - denom,
        denom,
    )
}
fn corners(t: Tile, max: u8, n: u32) -> [Key; 4] {
    [
        key(t, 0, 0, max, n),
        key(t, n, 0, max, n),
        key(t, n, n, max, n),
        key(t, 0, n, max, n),
    ]
}
impl Registry {
    pub fn new(leaves: &BTreeSet<Tile>, max: u8, n: u32) -> Self {
        let mut r = Self {
            lines: Default::default(),
            levels: Default::default(),
            max,
            n,
        };
        for &t in leaves {
            let c = corners(t, max, n);
            for e in 0..4 {
                let a = c[e];
                let b = c[(e + 1) % 4];
                let l = line(a, b);
                for i in 0..=n {
                    let v = a[l.0] + (b[l.0] - a[l.0]) * i64::from(i) / i64::from(n);
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
        let c = corners(t, self.max, self.n);
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
    n: u32,
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    coordinates: Vec<[f32; 3]>,
    high_low: Vec<crate::meshes::SplitPosition>,
    // Indices are assigned only by deterministic lattice traversal, never by
    // map iteration. Randomized lookup does not change the emitted stream.
    indices: std::collections::HashMap<Key, u32>,
    cast_error: f64,
    previous: Option<&'a Resource>,
    old_indices: std::collections::HashMap<Key, usize>,
    normal_levels: Vec<u8>,
    sample_hits: usize,
}
impl Builder<'_> {
    fn vertex(&mut self, k: Key, level: u8) -> u32 {
        if let Some(&i) = self.indices.get(&k) {
            return i;
        }
        if let Some(old) = self.previous
            && let Some(&at) = self.old_indices.get(&k)
            && old.normal_levels[at] == level
        {
            let index = self.positions.len() as u32;
            self.indices.insert(k, index);
            self.positions.push(old.asset.geometry.positions[at]);
            self.normals.push(old.normals[at]);
            self.coordinates
                .push(old.asset.geometry.material_coordinates.as_ref().unwrap()[at]);
            self.high_low.push(old.coordinates.as_ref().unwrap()[at]);
            self.normal_levels.push(level);
            self.cast_error = self.cast_error.max(old.cast_error);
            self.sample_hits += 1;
            return index;
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
            finest.max(self.doc.radius() / (f64::from(self.n) * (1u64 << level) as f64))
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
        self.normal_levels.push(level);
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
#[cfg(test)]
pub(super) fn resource(
    doc: &Document,
    config: &super::super::runtime::Config,
    k: &ResourceKey,
    registry: &Registry,
    max: u8,
) -> Result<Resource> {
    resource_reusing(doc, config, k, registry, max, None)
}
pub(super) fn resource_reusing<'a>(
    doc: &'a Document,
    config: &super::super::runtime::Config,
    k: &ResourceKey,
    registry: &Registry,
    max: u8,
    previous: Option<&'a Resource>,
) -> Result<Resource> {
    // Bound the seven height evaluations per vertex before allocating/cooking.
    // A fine boundary may add many vertices to an otherwise coarse cell.
    let n = k.grid;
    doc.budget((((n + 1) * (n + 1)) as usize + k.signature.len()) * 7)?;
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
        n,
        positions: vec![],
        normals: vec![],
        coordinates: vec![],
        high_low: vec![],
        indices: Default::default(),
        cast_error: 0.,
        previous,
        old_indices: previous
            .map(|r| r.keys.iter().enumerate().map(|(i, k)| (*k, i)).collect())
            .unwrap_or_default(),
        normal_levels: vec![],
        sample_hits: 0,
    };
    let mut triangles = Vec::with_capacity((n * n * 2) as usize);
    let mut polygon = Vec::with_capacity(8);
    let mut vertices = Vec::with_capacity(8);
    for y in 0..n {
        for x in 0..n {
            let c = [
                key(t, x, y, max, n),
                key(t, x + 1, y, max, n),
                key(t, x + 1, y + 1, max, n),
                key(t, x, y + 1, max, n),
            ];
            polygon.clear();
            for e in 0..4 {
                let boundary = match e {
                    0 => y == 0,
                    1 => x + 1 == n,
                    2 => y + 1 == n,
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
        keys: {
            let mut v = vec![[0; 3]; b.indices.len()];
            for (k, i) in b.indices {
                v[i as usize] = k;
            }
            v
        },
        normal_levels: b.normal_levels,
        sample_hits: b.sample_hits,
        asset: Asset {
            id: format!("orbit_{}", t.label()),
            geometry: Arc::new(geometry),
            shading: Shading::Smooth,
            material,
            rig: None,
            clips: vec![],
        },
        normals: Arc::new(b.normals),
        cast_error: b.cast_error,
        bounds: global_bounds(doc, t),
        radial_upper: doc.radius() + height_bounds(doc, t.cone().0, t.cone().1).1,
        error: error(doc, t, max, n) + b.cast_error * 2.,
    })
}
pub(super) fn body(
    doc: &Document,
    _anchor: Anchor,
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
        keys: vec![],
        normal_levels: vec![],
        sample_hits: 0,
        asset: Asset {
            id: "orbit_avatar".into(),
            geometry: Arc::new(g),
            shading: Shading::Smooth,
            material: Default::default(),
            rig: None,
            clips: vec![],
        },
        normals: Arc::new(n),
        cast_error: 0.,
        bounds: [DVec3::ZERO; 8],
        radial_upper: 0.,
        error: 0.,
    })
}

pub(super) struct Grouped {
    pub built: u64,
    pub assets: Vec<Asset>,
    pub coordinates: Vec<Option<Arc<Vec<crate::meshes::SplitPosition>>>>,
    pub normals: Vec<Arc<Vec<[f32; 3]>>>,
    pub groups: Vec<Group>,
}
/// Submission-only aggregation, preserving triangle order and every consumed
/// attribute. Local positions remain per-piece validation proxies; the explicit
/// common high/low stream is the sole planetary raster/bounds coordinate system.
/// Detail projection still depends on local proxy geometry: keep it ungrouped.
pub(super) fn groups(
    resources: &BTreeMap<ResourceKey, Arc<Resource>>,
    requested: usize,
    body: &Resource,
    previous: &Atlas,
    anchor: Anchor,
    view: &View,
) -> Result<Grouped> {
    let old: BTreeMap<_, _> = previous
        .groups
        .iter()
        .enumerate()
        .map(|(i, g)| (Arc::as_ptr(&g.members[0]) as usize, i))
        .collect();
    let mut out = Grouped {
        built: 0,
        assets: vec![],
        coordinates: vec![],
        normals: vec![],
        groups: vec![],
    };
    let entries: Vec<_> = resources.iter().collect();
    let mut offset = 0;
    while offset < entries.len() {
        let first = entries[offset];
        let mut end = offset + 1;
        if first.1.asset.material.detail.is_none()
            && interior(first.1, anchor, view, &previous.settings)
        {
            while end < entries.len()
                && end - offset < requested
                && entries[end].0.tile.face == first.0.tile.face
                && entries[end].0.tile.level == first.0.tile.level
                && entries[end].0.tile.xy[0].abs_diff(first.0.tile.xy[0]) <= 1
                && entries[end].0.tile.xy[1].abs_diff(first.0.tile.xy[1]) <= 7
                && entries[end].1.asset.material == first.1.asset.material
                && interior(entries[end].1, anchor, view, &previous.settings)
            {
                end += 1;
            }
        }
        let chunk = &entries[offset..end];
        let mut asset = first.1.asset.clone();
        let mut lo = DVec3::splat(f64::INFINITY);
        let mut hi = DVec3::splat(f64::NEG_INFINITY);
        for (_, r) in chunk {
            for p in r.bounds {
                lo = lo.min(p);
                hi = hi.max(p);
            }
        }
        let bounds = std::array::from_fn(|i| {
            DVec3::new(
                if i & 1 == 0 { lo.x } else { hi.x },
                if i & 2 == 0 { lo.y } else { hi.y },
                if i & 4 == 0 { lo.z } else { hi.z },
            )
        });
        let retained = old
            .get(&(Arc::as_ptr(first.1) as usize))
            .copied()
            .filter(|&i| {
                let g = &previous.groups[i];
                g.members.len() == chunk.len()
                    && g.members
                        .iter()
                        .zip(chunk)
                        .all(|(a, (_, b))| Arc::ptr_eq(a, b))
            });
        out.groups.push(Group {
            members: chunk.iter().map(|(_, r)| Arc::clone(r)).collect(),
            envelope: crate::atmosphere::transport::Envelope {
                corners: bounds,
                radius_upper: chunk.iter().map(|(_, r)| r.radial_upper).fold(0., f64::max),
            },
            material: asset.material.clone(),
        });
        if let Some(i) = retained {
            asset.geometry = previous.meshes.assets[i].geometry.clone();
            out.coordinates.push(previous.coordinates[i].clone());
            out.normals.push(previous.normals[i].clone());
        } else if chunk.len() == 1 {
            out.coordinates.push(first.1.coordinates.clone());
            out.normals.push(first.1.normals.clone());
        } else {
            out.built += 1;
            let nv = chunk
                .iter()
                .map(|(_, r)| r.asset.geometry.positions.len())
                .sum();
            let nt = chunk
                .iter()
                .map(|(_, r)| r.asset.geometry.triangles.len())
                .sum();
            let mut positions = Vec::with_capacity(nv);
            let mut triangles = Vec::with_capacity(nt);
            let mut rest = Vec::with_capacity(nv);
            let mut normals = Vec::with_capacity(nv);
            let mut split = Vec::with_capacity(nv);
            for (_, r) in chunk {
                let g = &r.asset.geometry;
                let start = positions.len() as u32;
                positions.extend_from_slice(&g.positions);
                triangles.extend(g.triangles.iter().map(|t| t.map(|i| i + start)));
                rest.extend_from_slice(
                    g.material_coordinates
                        .as_ref()
                        .expect("canonical terrain domain"),
                );
                normals.extend_from_slice(&r.normals);
                split.extend_from_slice(
                    r.coordinates
                        .as_ref()
                        .expect("canonical terrain coordinates"),
                );
            }
            let geometry = Geometry {
                positions,
                triangles,
                material_coordinates: Some(rest),
            };
            geometry.validate()?;
            asset.geometry = Arc::new(geometry);
            out.coordinates.push(Some(Arc::new(split)));
            out.normals.push(Arc::new(normals));
        }
        out.assets.push(asset);
        offset = end;
    }
    out.assets.push(body.asset.clone());
    out.coordinates.push(None);
    out.normals.push(body.normals.clone());
    Ok(out)
}

// A group is admitted only well inside the current frustum. Boundary/outside
// tiles retain individual culling, avoiding the extra triangles of a loose union.
// The margin also covers the small look/motion window before the next replan.
fn interior(resource: &Resource, anchor: Anchor, view: &View, settings: &Settings) -> bool {
    let transform = view.basis.transpose() * anchor.basis.transpose();
    let offset = view.basis.transpose()
        * (-DVec3::Y * (anchor.radius + anchor.pose.altitude_meters) - view.eye);
    let tan = (settings.fov_degrees.to_radians() * 0.5).tan();
    let aspect = f64::from(settings.viewport[0]) / f64::from(settings.viewport[1]);
    resource.bounds.iter().all(|p| {
        let p = transform * *p + offset;
        let margin = 64. + settings.distance_meters * 0.02 + p.length() * 0.02;
        p.z < -f64::from(view.clip.near) - margin
            && p.z > -f64::from(view.clip.far) + margin
            && p.x.abs() < -p.z * tan * aspect - margin
            && p.y.abs() < -p.z * tan - margin
    })
}
