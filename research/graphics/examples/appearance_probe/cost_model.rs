//! Monotone, hardware-calibrated difference between two quality-admissible paths.
use super::*;
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(super) struct Model {
    pub coefficients: [f64; 6],
}
impl Model {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.coefficients.iter().all(|x| x.is_finite()),
            "nonfinite model"
        );
        ensure!(
            [1, 2, 3, 5].into_iter().all(|i| self.coefficients[i] >= 0.),
            "nonmonotone model"
        );
        Ok(())
    }
    pub fn predict(&self, features: [f64; 6]) -> f64 {
        self.coefficients
            .iter()
            .zip(features)
            .map(|(a, b)| a * b)
            .sum()
    }
}
pub(super) fn area(a: &Asset, cam: &Camera) -> f64 {
    let mut lo = glam::Vec2::ONE;
    let mut hi = -glam::Vec2::ONE;
    for x in 0..2 {
        for y in 0..2 {
            for z in 0..2 {
                let p = cam.matrix
                    * Vec3::new(a.bounds[x][0], a.bounds[y][1], a.bounds[z][2]).extend(1.);
                if p.w <= 0.1 {
                    return cam.size[0] as f64 * cam.size[1] as f64 / 1e6;
                }
                let q = (p.truncate().truncate() / p.w).clamp(-glam::Vec2::ONE, glam::Vec2::ONE);
                lo = lo.min(q);
                hi = hi.max(q);
            }
        }
    }
    let size = (hi - lo).max(glam::Vec2::ZERO);
    size.x as f64 * size.y as f64 * cam.size[0] as f64 * cam.size[1] as f64 / 4e6
}
fn terms(g: u32, r: u32, draws: usize, pixels: f64, native: bool) -> [f64; 6] {
    [
        1.,
        (g as f64 - r as f64) / 1e6,
        1. - draws as f64,
        ((g as f64).sqrt() - (r as f64).sqrt()) * pixels.sqrt() / 1000.,
        pixels,
        if native { -pixels } else { 0. },
    ]
}
pub(super) fn features(a: &Asset, cam: &Camera) -> [f64; 6] {
    let g = render::selection(a, cam, 4);
    let r = render::selection(a, cam, 3);
    terms(
        g.2,
        r.2,
        r.1,
        area(a, cam),
        g.0[0].1 == a.sealed.last().unwrap().count,
    )
}
pub(super) fn choose(
    a: &Asset,
    cam: &Camera,
    model: &Model,
    prune: bool,
) -> (render::DrawSelection, u32, usize) {
    let g = render::selection(a, cam, 4);
    let pixels = area(a, cam);
    let native = g.0[0].1 == a.sealed.last().unwrap().count;
    let delta = |triangles, draws| model.predict(terms(g.2, triangles, draws, pixels, native));
    if prune && delta(a.regional_floor, 1) <= 0. {
        return (g, 4, 0);
    }
    let mut draws = [(0, 0); 64];
    let (mut count, mut triangles, mut remaining) = (0, 0, a.regional_floor);
    for (i, r) in a.regions.iter().enumerate() {
        let l = &r.levels[render::selected(&r.levels, cam, r.bounds, false)];
        triangles += l.count / 3;
        remaining -= r.floor;
        if count > 0 && draws[count - 1].0 + draws[count - 1].1 == l.first {
            draws[count - 1].1 += l.count;
        } else {
            draws[count] = (l.first, l.count);
            count += 1;
        }
        if prune && delta(triangles + remaining, count) <= 0. {
            return (g, 4, i + 1);
        }
    }
    if delta(triangles, count) <= 0. {
        (g, 4, 64)
    } else {
        ((draws, count, triangles), 3, 64)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cost_difference_is_monotone_and_rejects_invalid_models() {
        let m = Model {
            coefficients: [-0.01, 0.7, 0.001, 0.02, 0.003, 0.04],
        };
        m.validate().unwrap();
        for area in [0., 0.01, 1., 8.] {
            for native in [false, true] {
                let mut last = f64::INFINITY;
                for r in [1, 32, 1024, 32768, 3276800] {
                    let d = m.predict(terms(32768, r, 1, area, native));
                    assert!(d <= last);
                    last = d;
                    assert!(m.predict(terms(32768, r, 8, area, native)) <= d);
                }
            }
        }
        for i in [1, 2, 3, 5] {
            let mut bad = m.clone();
            bad.coefficients[i] = -1.;
            assert!(bad.validate().is_err());
        }
        let mut bad = m;
        bad.coefficients[0] = f64::NAN;
        assert!(bad.validate().is_err());
    }
}
