//! Publication checks shared by the native compiler and its standalone laboratory.
use crate::meshes::Geometry;
use anyhow::{Result, ensure};
use glam::Vec3;
fn root(p: &mut [u32], mut a: u32) -> u32 {
    while p[a as usize] != a {
        p[a as usize] = p[p[a as usize] as usize];
        a = p[a as usize];
    }
    a
}
pub fn labels(g: &Geometry) -> Vec<u32> {
    let mut p = (0..g.positions.len() as u32).collect::<Vec<_>>();
    for t in &g.triangles {
        let a = root(&mut p, t[0]);
        for &v in &t[1..] {
            let b = root(&mut p, v);
            p[b as usize] = a;
        }
    }
    for i in 0..p.len() {
        p[i] = root(&mut p, i as u32);
    }
    p
}
pub fn repair(
    g: &Geometry,
    labels: &[u32],
    vertices: &mut Vec<u32>,
    triangles: &mut Vec<[u32; 3]>,
) -> Result<(usize, usize)> {
    ensure!(
        vertices.iter().all(|i| (*i as usize) < g.positions.len()),
        "Candidate source index invalid"
    );
    let mut faces = Vec::with_capacity(triangles.len());
    let mut removed = 0;
    let mut present = vec![false; g.positions.len()];
    for t in triangles.iter() {
        ensure!(
            t.iter().all(|i| (*i as usize) < vertices.len()),
            "Candidate local index invalid"
        );
        let f = t.map(|i| vertices[i as usize]);
        let [a, b, c] = f.map(|i| Vec3::from(g.positions[i as usize]));
        if (b - a).cross(c - a).length_squared() <= 0. {
            removed += 1;
            continue;
        }
        ensure!(
            f.iter()
                .all(|i| labels[*i as usize] == labels[f[0] as usize]),
            "Candidate joins separate components"
        );
        present[labels[f[0] as usize] as usize] = true;
        faces.push(f);
    }
    let mut restored = 0;
    for f in &g.triangles {
        if !present[labels[f[0] as usize] as usize] {
            faces.push(*f);
            restored += 1;
        }
    }
    let mut remap = vec![u32::MAX; g.positions.len()];
    vertices.clear();
    triangles.clear();
    for f in faces {
        triangles.push(f.map(|i| {
            if remap[i as usize] == u32::MAX {
                remap[i as usize] = vertices.len() as u32;
                vertices.push(i);
            }
            remap[i as usize]
        }));
    }
    Ok((removed, restored))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn removes_degenerate_faces_and_restores_missing_component() {
        let mut g = Geometry::grid(1, 1.).unwrap();
        let positions = g.positions.clone();
        let faces = g.triangles.clone();
        let offset = positions.len() as u32;
        g.positions
            .extend(positions.iter().map(|p| [p[0] + 10., p[1], p[2]]));
        g.triangles
            .extend(faces.iter().map(|t| t.map(|i| i + offset)));
        let labels = labels(&g);
        let mut vertices = (0..g.positions.len() as u32).collect();
        let mut triangles = vec![faces[0], [0, 0, 0]];
        let (removed, restored) = repair(&g, &labels, &mut vertices, &mut triangles).unwrap();
        assert_eq!(removed, 1);
        assert_eq!(restored, faces.len());
        let kept: std::collections::BTreeSet<_> = triangles
            .iter()
            .map(|t| labels[vertices[t[0] as usize] as usize])
            .collect();
        assert_eq!(kept.len(), 2);
        for t in triangles {
            let [a, b, c] = t.map(|i| Vec3::from(g.positions[vertices[i as usize] as usize]));
            assert!((b - a).cross(c - a).length_squared() > 0.);
        }
    }
}
