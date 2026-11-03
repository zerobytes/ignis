//! Bounded convex box primitives with one planar chamfer along each edge.
use crate::{
    meshes::{Geometry, MAX_TRIANGLES, MAX_VERTICES},
    model::{Result, invalid},
};
use glam::{DQuat, DVec3};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BoxSpec {
    pub center: [f32; 3],
    pub size: [f32; 3],
    #[serde(default = "identity")]
    pub rotation: [f32; 4],
    pub bevel: f32,
}
fn identity() -> [f32; 4] {
    [0., 0., 0., 1.]
}
impl BoxSpec {
    fn validate(&self) -> Result<()> {
        let norm: f64 = self.rotation.iter().map(|x| f64::from(*x).powi(2)).sum();
        let minimum = self.size.iter().copied().fold(f32::INFINITY, f32::min);
        if self
            .center
            .iter()
            .any(|x| !x.is_finite() || x.abs() > 10000.)
            || self
                .size
                .iter()
                .any(|x| !x.is_finite() || !(0.0001..=1000.).contains(x))
            || self.rotation.iter().any(|x| !x.is_finite())
            || (norm - 1.).abs() > 0.0001
            || !self.bevel.is_finite()
            || self.bevel != 0. && !(0.00001..=minimum * 0.25).contains(&self.bevel)
        {
            return Err(invalid(
                "Box requires finite center within10000m, size0.0001..1000m, unit quaternion and bevel0 or0.00001..min(size)/4",
            ));
        }
        Ok(())
    }
}
pub fn counts(boxes: &[BoxSpec]) -> Result<(usize, usize)> {
    if !(1..=512).contains(&boxes.len()) {
        return Err(invalid("Chamfer boxes requires1..512 boxes"));
    }
    let (mut v, mut t) = (0, 0);
    for (i, b) in boxes.iter().enumerate() {
        b.validate()
            .map_err(|e| invalid(format!("Box{i}: {}", e.message)))?;
        let bevel = b.bevel > 0.;
        v += if bevel { 24 } else { 8 };
        t += if bevel { 44 } else { 12 };
        if v > MAX_VERTICES || t > MAX_TRIANGLES {
            return Err(invalid("Chamfer boxes exceeds mesh budget"));
        }
    }
    Ok((v, t))
}
pub(crate) struct Plan<'a> {
    boxes: &'a [BoxSpec],
    pub vertices: usize,
    pub triangles: usize,
}
impl<'a> Plan<'a> {
    pub(crate) fn new(boxes: &'a [BoxSpec]) -> Result<Self> {
        let (vertices, triangles) = counts(boxes)?;
        Ok(Self {
            boxes,
            vertices,
            triangles,
        })
    }
    pub(crate) fn generate(self) -> Result<Geometry> {
        self.append_to(Geometry {
            positions: vec![],
            triangles: vec![],
            material_coordinates: Some(vec![]),
        })
    }
    pub(crate) fn append_to(self, mut g: Geometry) -> Result<Geometry> {
        g.positions.reserve(self.vertices);
        g.triangles.reserve(self.triangles);
        g.material_coordinates
            .get_or_insert_with(|| g.positions.clone())
            .reserve(self.vertices);
        for (i, b) in self.boxes.iter().enumerate() {
            append(&mut g, b).map_err(|e| invalid(format!("Box{i}: {}", e.message)))?;
        }
        g.validate()?;
        Ok(g)
    }
}
pub fn generate(boxes: &[BoxSpec]) -> Result<Geometry> {
    Plan::new(boxes)?.generate()
}
fn append(g: &mut Geometry, b: &BoxSpec) -> Result<()> {
    let h = DVec3::from_array(b.size.map(f64::from)) * 0.5;
    let c = f64::from(b.bevel);
    let mut points = Vec::with_capacity(if c > 0. { 24 } else { 8 });
    let mut planes = Vec::with_capacity(if c > 0. { 26 } else { 6 });
    let signs = [-1., 1.];
    if c == 0. {
        for x in signs {
            for y in signs {
                for z in signs {
                    points.push(h * DVec3::new(x, y, z));
                }
            }
        }
    } else {
        for axis in 0..3 {
            let a = (axis + 1) % 3;
            let d = (axis + 2) % 3;
            for s in signs {
                for u in signs {
                    for v in signs {
                        let mut p = DVec3::ZERO;
                        p[axis] = s * h[axis];
                        p[a] = u * (h[a] - c);
                        p[d] = v * (h[d] - c);
                        points.push(p);
                    }
                }
            }
        }
    }
    for axis in 0..3 {
        for s in signs {
            let mut n = DVec3::ZERO;
            n[axis] = s;
            planes.push((n, h[axis], 4));
        }
    }
    if c > 0. {
        for a in 0..3 {
            for d in a + 1..3 {
                for s in signs {
                    for t in signs {
                        let mut n = DVec3::ZERO;
                        n[a] = s;
                        n[d] = t;
                        planes.push((n, h[a] + h[d] - c, 4));
                    }
                }
            }
        }
        for x in signs {
            for y in signs {
                for z in signs {
                    planes.push((DVec3::new(x, y, z), h.element_sum() - 2. * c, 3));
                }
            }
        }
    }
    let start = g.positions.len() as u32;
    let q = DQuat::from_array(b.rotation.map(f64::from)).normalize();
    let origin = DVec3::from_array(b.center.map(f64::from));
    for p in &points {
        let out = (origin + q * *p).as_vec3().to_array();
        g.positions.push(out);
        g.material_coordinates.as_mut().unwrap().push(out);
    }
    for (normal, distance, expected) in planes {
        let mut ids: Vec<usize> = points
            .iter()
            .enumerate()
            .filter_map(|(i, p)| ((normal.dot(*p) - distance).abs() < 1e-8).then_some(i))
            .collect();
        if ids.len() != expected {
            return Err(invalid(
                "Box face precision cannot certify expected connectivity",
            ));
        }
        let n = normal.normalize();
        let axis = if n.x.abs() < 0.9 { DVec3::X } else { DVec3::Y };
        let u = n.cross(axis).normalize();
        let v = n.cross(u);
        let center = ids.iter().map(|i| points[*i]).sum::<DVec3>() / ids.len() as f64;
        let angle = |i: usize| {
            let p = points[i] - center;
            p.dot(v).atan2(p.dot(u))
        };
        ids.sort_by(|a, b| angle(*a).total_cmp(&angle(*b)).then(a.cmp(b)));
        for i in 1..ids.len() - 1 {
            let face = [
                start + ids[0] as u32,
                start + ids[i] as u32,
                start + ids[i + 1] as u32,
            ];
            let [a, b, c] = face.map(|j| DVec3::from_array(g.positions[j as usize].map(f64::from)));
            if (b - a).cross(c - a).dot(q * n) <= 0. {
                return Err(invalid(
                    "Box face collapses or reverses after f32 publication",
                ));
            }
            g.triangles.push(face);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh_topology::Topology;
    fn fixture(bevel: f32) -> BoxSpec {
        BoxSpec {
            center: [0.; 3],
            size: [2., 3., 4.],
            rotation: identity(),
            bevel,
        }
    }
    fn volume(g: &Geometry) -> f64 {
        g.triangles
            .iter()
            .map(|t| {
                let [a, b, c] =
                    t.map(|i| DVec3::from_array(g.positions[i as usize].map(f64::from)));
                a.dot(b.cross(c)) / 6.
            })
            .sum()
    }
    #[test]
    fn analytic_counts_volume_manifold_and_outward() {
        for bevel in [0., 0.1, 0.5] {
            let b = fixture(bevel);
            let g = generate(&[b]).unwrap();
            assert_eq!(
                (g.positions.len(), g.triangles.len()),
                if bevel == 0. { (8, 12) } else { (24, 44) }
            );
            let c = f64::from(bevel);
            let expected = 24. - 2. * c * c * 9. + 16. / 3. * c * c * c;
            assert!((volume(&g) - expected).abs() < 2e-6);
            let t = Topology::inspect(&g).unwrap();
            assert_eq!((t.boundary_edges, t.non_manifold_edges), (0, 0));
            for edge in t.edges {
                assert_eq!(edge.faces.len(), 2);
            }
            for f in &g.triangles {
                let [a, b, c] =
                    f.map(|i| DVec3::from_array(g.positions[i as usize].map(f64::from)));
                assert!((b - a).cross(c - a).dot((a + b + c) / 3.) > 0.);
            }
            for axis in 0..3 {
                let extent = [1., 1.5, 2.][axis];
                assert_eq!(
                    g.positions
                        .iter()
                        .map(|p| p[axis])
                        .fold(f32::NEG_INFINITY, f32::max),
                    extent
                );
                assert_eq!(
                    g.positions
                        .iter()
                        .map(|p| p[axis])
                        .fold(f32::INFINITY, f32::min),
                    -extent
                );
            }
            assert_eq!(g.material_coordinates.as_ref().unwrap(), &g.positions);
        }
    }
    #[test]
    fn transformed_batch_and_append_are_deterministic() {
        let mut b = fixture(0.1);
        b.rotation = glam::Quat::from_rotation_y(0.7).to_array();
        b.center = [3., 2., -4.];
        let g = generate(std::slice::from_ref(&b)).unwrap();
        assert_eq!(g, generate(&[b]).unwrap());
        assert!(volume(&g) > 23.8);
        let old = generate(&[fixture(0.)]).unwrap();
        let mut source = old.clone();
        source.material_coordinates = None;
        let g = Plan::new(&[fixture(0.1)])
            .unwrap()
            .append_to(source)
            .unwrap();
        assert_eq!(&g.positions[..8], old.positions);
        assert_eq!(&g.triangles[..12], old.triangles);
        assert_eq!(&g.material_coordinates.unwrap()[..8], old.positions);
    }
    #[test]
    fn limits_invalid_and_publication_precision() {
        assert!(counts(&[]).is_err());
        assert!(counts(&vec![fixture(0.1); 513]).is_err());
        for case in 0..7 {
            let mut b = fixture(0.1);
            match case {
                0 => b.center[0] = f32::NAN,
                1 => b.size[1] = 0.,
                2 => b.rotation = [0.; 4],
                3 => b.bevel = -0.1,
                4 => b.bevel = 0.6,
                5 => b.bevel = 0.000001,
                _ => b.rotation[0] = f32::INFINITY,
            }
            assert!(generate(&[b]).is_err());
        }
        let b = BoxSpec {
            center: [9999.; 3],
            size: [0.0001; 3],
            rotation: identity(),
            bevel: 0.00001,
        };
        assert!(counts(std::slice::from_ref(&b)).is_ok());
        assert!(generate(&[b]).is_err());
        assert_eq!(counts(&vec![fixture(0.1); 512]).unwrap(), (12288, 22528));
    }
}
