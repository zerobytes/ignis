//! Seeded planar cuts. Cold authoring operation; open boundary vertices stay fixed.
use crate::{
    meshes::Geometry,
    model::{Result, invalid},
};
use glam::Vec3;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Fraction of each corner-to-centroid vector, not a world-space radius.
    pub fraction: f32,
    pub variation: f32,
    pub seed: u64,
}
pub(crate) fn random(mut x: u64) -> f32 {
    x = x.wrapping_add(0x9e3779b97f4a7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    ((x ^ (x >> 31)) >> 40) as f32 / 16777215.
}
pub fn apply(g: &Geometry, c: &Config) -> Result<Geometry> {
    if !c.fraction.is_finite()
        || !(0.001..=0.25).contains(&c.fraction)
        || !c.variation.is_finite()
        || !(0. ..=0.9).contains(&c.variation)
    {
        return Err(invalid(
            "Chamfer fraction requires0.001..0.25; variation0..0.9",
        ));
    }
    g.validate()?;
    if g.triangles.len() > 100_000 || g.positions.len() > 100_000 {
        return Err(invalid(
            "Chamfer cold-operation budget100000source faces/vertices; split the source",
        ));
    }
    let mut edges = BTreeMap::<(u32, u32), Vec<(usize, usize)>>::new();
    for (f, t) in g.triangles.iter().enumerate() {
        for k in 0..3 {
            let (a, b) = (t[k], t[(k + 1) % 3]);
            edges.entry((a.min(b), a.max(b))).or_default().push((f, k));
        }
    }
    let mut fixed = vec![false; g.positions.len()];
    for (&(a, b), list) in &edges {
        if list.len() > 2 {
            return Err(invalid("Chamfer requires manifold edges"));
        }
        if list.len() == 1 {
            fixed[a as usize] = true;
            fixed[b as usize] = true;
        } else {
            let (f, k) = list[0];
            let (h, j) = list[1];
            if g.triangles[f][k] != g.triangles[h][(j + 1) % 3] {
                return Err(invalid("Chamfer requires consistent winding"));
            }
        }
    }
    let mut out = Geometry {
        positions: g.positions.clone(),
        triangles: vec![],
        material_coordinates: g.material_coordinates.clone(),
    };
    let mut corners = Vec::<[u32; 3]>::with_capacity(g.triangles.len());
    let mut links = vec![BTreeMap::<u32, u32>::new(); g.positions.len()];
    for (f, t) in g.triangles.iter().enumerate() {
        let p = t.map(|v| Vec3::from_array(g.positions[v as usize]));
        let center = (p[0] + p[1] + p[2]) / 3.;
        let ids = std::array::from_fn(|k| {
            let v = t[k];
            if fixed[v as usize] {
                return v;
            }
            let alpha = c.fraction
                * (1.
                    + c.variation
                        * (2. * random(c.seed ^ ((f as u64).wrapping_mul(3) + k as u64)) - 1.));
            let id = out.positions.len() as u32;
            out.positions
                .push((p[k] + (center - p[k]) * alpha).to_array());
            if let (Some(src), Some(dst)) = (&g.material_coordinates, &mut out.material_coordinates)
            {
                let uv = t.map(|v| Vec3::from_array(src[v as usize]));
                let avg = (uv[0] + uv[1] + uv[2]) / 3.;
                dst.push((uv[k] + (avg - uv[k]) * alpha).to_array());
            }
            id
        });
        corners.push(ids);
        out.triangles.push(ids);
    }
    for list in edges.values().filter(|l| l.len() == 2) {
        let (f, k) = list[0];
        let (h, j) = list[1];
        let (a, b) = (corners[f][k], corners[f][(k + 1) % 3]);
        let (d, e) = (corners[h][j], corners[h][(j + 1) % 3]);
        out.triangles.extend([[b, a, e], [b, e, d]]);
        let va = g.triangles[f][k] as usize;
        let vb = g.triangles[f][(k + 1) % 3] as usize;
        links[va].insert(e, a);
        links[vb].insert(b, d);
    }
    for (v, ring) in links
        .iter()
        .enumerate()
        .filter(|(v, r)| !fixed[*v] && !r.is_empty())
    {
        let start = *ring.keys().next().unwrap();
        let mut ids = vec![start];
        let mut next = ring[&start];
        while next != start {
            if ids.len() >= ring.len() {
                return Err(invalid("Chamfer requires one manifold fan per vertex"));
            }
            ids.push(next);
            next = *ring
                .get(&next)
                .ok_or_else(|| invalid("Broken chamfer fan"))?;
        }
        if ids.len() != ring.len() {
            return Err(invalid("Disconnected chamfer vertex fan"));
        }
        let center = ids
            .iter()
            .map(|&i| Vec3::from_array(out.positions[i as usize]))
            .sum::<Vec3>()
            / ids.len() as f32;
        let id = out.positions.len() as u32;
        out.positions.push(center.to_array());
        if let (Some(src), Some(dst)) = (&g.material_coordinates, &mut out.material_coordinates) {
            dst.push(src[v]);
        }
        for k in 0..ids.len() {
            out.triangles.push([id, ids[k], ids[(k + 1) % ids.len()]]);
        }
    }
    out.triangles.retain(|t| {
        let p = t.map(|v| Vec3::from_array(out.positions[v as usize]));
        (p[1] - p[0]).cross(p[2] - p[0]).length_squared() > 0.
    });
    // Compact unused original vertices, preserving shared indexed connectivity.
    let mut map = vec![u32::MAX; out.positions.len()];
    let mut compact = Geometry {
        positions: vec![],
        triangles: vec![],
        material_coordinates: out.material_coordinates.as_ref().map(|_| vec![]),
    };
    for t in &out.triangles {
        compact.triangles.push(t.map(|v| {
            let slot = &mut map[v as usize];
            if *slot == u32::MAX {
                *slot = compact.positions.len() as u32;
                compact.positions.push(out.positions[v as usize]);
                if let (Some(src), Some(dst)) =
                    (&out.material_coordinates, &mut compact.material_coordinates)
                {
                    dst.push(src[v as usize]);
                }
            }
            *slot
        }));
    }
    compact.validate()?;
    Ok(compact)
}
