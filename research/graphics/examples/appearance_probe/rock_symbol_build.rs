use super::*;
use std::collections::BTreeMap;
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub(super) struct Symbol {
    pub center_radius: [f32; 4],
    pub normal_thickness: [f32; 4],
    pub color_area: [f32; 4],
}
pub(super) fn build(v: &[Vertex], indices: &[u32], grid: u32) -> Result<Vec<Symbol>> {
    ensure!((1..=256).contains(&grid), "grid outside budget");
    ensure!(indices.len().is_multiple_of(3), "triangle count");
    let mut groups: BTreeMap<[i32; 4], Vec<usize>> = BTreeMap::new();
    let step = 2.5 / grid as f32;
    for (id, t) in indices.chunks_exact(3).enumerate() {
        ensure!(t.iter().all(|&i| (i as usize) < v.len()), "triangle index");
        let p = t.map_positions(v);
        let center = (p[0] + p[1] + p[2]) / 3.;
        let n = t
            .iter()
            .map(|&i| Vec3::from_array(v[i as usize].normal))
            .sum::<Vec3>();
        let octant =
            i32::from(n.x >= 0.) | (i32::from(n.y >= 0.) << 1) | (i32::from(n.z >= 0.) << 2);
        let c = ((center - CENTER) / step).floor().as_ivec3();
        groups.entry([c.x, c.y, c.z, octant]).or_default().push(id);
    }
    let mut result = Vec::with_capacity(groups.len());
    for tris in groups.values() {
        let (mut pos, mut normal, mut color, mut area) = (
            glam::DVec3::ZERO,
            glam::DVec3::ZERO,
            glam::DVec3::ZERO,
            0f64,
        );
        for &id in tris {
            let t = &indices[id * 3..id * 3 + 3];
            let p = t.map_positions(v).map(|p| p.as_dvec3());
            let a = (p[1] - p[0]).cross(p[2] - p[0]).length() * 0.5;
            ensure!(a.is_finite() && a > 1e-14, "degenerate triangle");
            let mut n = glam::DVec3::ZERO;
            let mut c = glam::DVec3::ZERO;
            for &i in t {
                ensure!(v[i as usize].tile == 0, "stone-only experiment");
                n += Vec3::from_array(v[i as usize].normal).as_dvec3();
                c += Vec3::from_array(v[i as usize].color).as_dvec3();
            }
            pos += (p[0] + p[1] + p[2]) * (a / 3.);
            normal += n * (a / 3.);
            color += c * (a / 3.);
            area += a;
        }
        ensure!(normal.length() > 1e-12, "opposed normals in symbol");
        let center = (pos / area).as_vec3();
        let n = normal.normalize().as_vec3();
        let mut radius = 0f32;
        let mut thickness = 0f32;
        for &id in tris {
            for &i in &indices[id * 3..id * 3 + 3] {
                let d = Vec3::from_array(v[i as usize].position) - center;
                let h = d.dot(n);
                radius = radius.max((d - h * n).length());
                thickness = thickness.max(h.abs());
            }
        }
        ensure!(radius > 0., "empty symbol");
        result.push(Symbol {
            center_radius: center.extend(radius * 1.001).to_array(),
            normal_thickness: n.extend(thickness + 1e-6).to_array(),
            color_area: (color / area).as_vec3().extend(area as f32).to_array(),
        });
    }
    ensure!(
        result
            .iter()
            .flat_map(|s| bytemuck::cast_slice::<Symbol, f32>(std::slice::from_ref(s)))
            .all(|v| v.is_finite()),
        "nonfinite symbol"
    );
    Ok(result)
}
trait Positions {
    fn map_positions(&self, v: &[Vertex]) -> [Vec3; 3];
}
impl Positions for [u32] {
    fn map_positions(&self, v: &[Vertex]) -> [Vec3; 3] {
        std::array::from_fn(|k| Vec3::from_array(v[self[k] as usize].position))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn square() -> (Vec<Vertex>, Vec<u32>) {
        let v = [[0., 0., 0.], [1., 0., 0.], [1., 0., 1.], [0., 0., 1.]]
            .map(|p| Vertex {
                position: p,
                normal: [0., 1., 0.],
                color: [0.2, 0.3, 0.4],
                tile: 0,
            })
            .to_vec();
        (v, vec![0, 2, 1, 0, 3, 2])
    }
    #[test]
    fn aggregation_conserves_area_attributes_and_contains_projected_support() {
        let (v, i) = square();
        let s = build(&v, &i, 1).unwrap();
        assert_eq!(s, build(&v, &i, 1).unwrap());
        let area: f32 = s.iter().map(|x| x.color_area[3]).sum();
        assert!((area - 1.).abs() < 1e-6);
        for x in s {
            assert!((x.normal_thickness[1] - 1.).abs() < 1e-6);
            assert!((x.color_area[0] - 0.2).abs() < 1e-6);
            assert!(x.normal_thickness[3] < 1e-5);
        }
        assert!(build(&v, &[0, 0, 0], 1).is_err());
        assert!(build(&v, &[0, 1, 8], 1).is_err());
        assert!(build(&v, &i, 0).is_err());
    }
    #[test]
    fn opposite_surfaces_and_relieved_positions_are_not_one_symbol() {
        let (mut v, mut i) = square();
        let mut second = v.clone();
        for x in &mut second {
            x.normal = [0., -1., 0.];
            x.position[1] = 0.2;
        }
        v.extend(second);
        i.extend([4, 5, 6, 4, 6, 7]);
        let s = build(&v, &i, 1).unwrap();
        assert!(s.iter().any(|x| x.normal_thickness[1] < 0.));
        assert!(s.iter().any(|x| x.normal_thickness[1] > 0.));
        assert!((s.iter().map(|x| x.color_area[3]).sum::<f32>() - 2.).abs() < 1e-6);
    }
}
