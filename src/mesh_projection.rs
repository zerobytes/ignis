//! Pose-independent, unsigned material projection weights with bounded ownership.
use crate::{
    meshes::{Asset, Geometry},
    model::Result,
};
use glam::DVec3;
use std::sync::Arc;

pub fn weights(geometry: &Geometry) -> Result<Vec<[f32; 3]>> {
    geometry.validate()?;
    let rest = geometry
        .material_coordinates
        .as_deref()
        .unwrap_or(&geometry.positions);
    let mut sums = vec![DVec3::ZERO; rest.len()];
    for t in &geometry.triangles {
        let [a, b, c] = t.map(|i| DVec3::from_array(rest[i as usize].map(f64::from)));
        let n = (b - a).cross(c - a);
        let square = n.length_squared();
        if square > 0. {
            let contribution = (n * n) / square * (square.sqrt() * 0.5);
            for &i in t {
                sums[i as usize] += contribution;
            }
        }
    }
    Ok(sums
        .into_iter()
        .map(|s| {
            let total = s.element_sum();
            if total > 0. {
                (s / total).to_array().map(|x| x as f32)
            } else {
                [1. / 3.; 3]
            }
        })
        .collect())
}
struct Entry {
    geometry: Arc<Geometry>,
    weights: Vec<[f32; 3]>,
}
#[derive(Default)]
pub(crate) struct Cache {
    entries: Vec<Entry>,
}
impl Cache {
    pub fn prepare(&mut self, assets: &[Asset]) {
        self.entries
            .retain(|e| assets.iter().any(|a| Arc::ptr_eq(&a.geometry, &e.geometry)));
        for asset in assets {
            if !self
                .entries
                .iter()
                .any(|e| Arc::ptr_eq(&e.geometry, &asset.geometry))
            {
                self.entries.push(Entry {
                    geometry: asset.geometry.clone(),
                    weights: weights(&asset.geometry).expect("validated geometry"),
                });
            }
        }
    }
    pub fn get(&self, geometry: &Arc<Geometry>) -> &[[f32; 3]] {
        &self
            .entries
            .iter()
            .find(|e| Arc::ptr_eq(&e.geometry, geometry))
            .expect("prepared projection cache")
            .weights
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn geometry() -> Geometry {
        Geometry {
            positions: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
            triangles: vec![[0, 1, 2], [0, 3, 1]],
            material_coordinates: None,
        }
    }
    fn asset(g: Arc<Geometry>, id: &str) -> Asset {
        Asset {
            id: id.into(),
            geometry: g,
            rig: None,
            clips: vec![],
            shading: Default::default(),
            material: Default::default(),
        }
    }
    #[test]
    fn unsigned_area_weighted_projection_is_continuous_and_does_not_cancel() {
        let mut g = geometry();
        let w = weights(&g).unwrap();
        assert_eq!(w[0], [0., 0.5, 0.5]);
        assert_eq!(w[2], [0., 0., 1.]);
        assert_eq!(w[3], [0., 1., 0.]);
        g.triangles[1].reverse();
        assert_eq!(weights(&g).unwrap(), w);
        g.material_coordinates = Some(vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.], [0., 0., 2.]]);
        let w = weights(&g).unwrap();
        assert!((w[0][1] - 2. / 3.).abs() < 1e-6);
        for p in &w {
            assert!(p.iter().all(|v| v.is_finite() && *v >= 0.));
            assert!((p.iter().sum::<f32>() - 1.).abs() < 1e-6);
        }
        // A shared edge uses exactly the same endpoint weights on both faces;
        // any positive power and renormalization is continuous and nonzero.
        for t in [0., 0.2, 0.5, 0.8, 1.] {
            for power in [0.5, 1., 4.] {
                let mixed = std::array::from_fn::<_, 3, _>(|i| {
                    ((1. - t) * w[0][i] + t * w[1][i]).powf(power)
                });
                assert!(mixed.iter().sum::<f32>() > 0.);
            }
        }
    }
    #[test]
    fn rank_zero_one_and_unused_vertices_have_finite_fallback() {
        let mut g = geometry();
        g.positions.push([2., 2., 2.]);
        g.material_coordinates = Some(g.positions.iter().map(|p| [p[0], 0., 0.]).collect());
        assert!(weights(&g).unwrap().iter().all(|w| *w == [1. / 3.; 3]));
    }
    #[test]
    fn arc_cache_shares_prunes_and_cannot_survive_cow_with_stale_data() {
        let mut a = asset(Arc::new(geometry()), "a");
        let b = asset(a.geometry.clone(), "b");
        let mut cache = Cache::default();
        cache.prepare(&[a.clone(), b.clone()]);
        assert_eq!(cache.entries.len(), 1);
        let before = cache.get(&a.geometry).as_ptr();
        cache.prepare(&[a.clone()]);
        assert_eq!(before, cache.get(&a.geometry).as_ptr());
        let old = Arc::downgrade(&a.geometry);
        drop(b);
        Arc::make_mut(&mut a.geometry).material_coordinates = Some(
            a.geometry
                .positions
                .iter()
                .map(|p| [p[2], p[1], p[0]])
                .collect(),
        );
        cache.prepare(&[a.clone()]);
        assert_eq!(cache.entries.len(), 1);
        assert!(old.upgrade().is_none());
        assert_eq!(cache.get(&a.geometry)[2], [1., 0., 0.]);
        // Recreated geometry with equal values is still a new current identity.
        a.geometry = Arc::new((*a.geometry).clone());
        cache.prepare(&[a]);
        assert_eq!(cache.entries.len(), 1);
        cache.prepare(&[]);
        assert!(cache.entries.is_empty());
    }
}
