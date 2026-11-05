use super::{
    compile::Asset,
    render::{Bound, Pose},
};
use glam::Vec3;
#[derive(Debug, serde::Serialize)]
pub struct Selection {
    pub ranges: Vec<(u32, u32)>,
    pub triangles: u32,
    pub reduced: usize,
    pub silhouette_native: usize,
    pub max_error_px: f32,
}
pub fn choose(asset: &Asset, pose: Pose, height: u32, budget: f32, force: bool) -> Selection {
    choose_with_normal(asset, pose, height, budget, force, 0.01)
}
pub fn choose_with_normal(
    asset: &Asset,
    pose: Pose,
    height: u32,
    budget: f32,
    force: bool,
    normal_limit: f32,
) -> Selection {
    let mut choices = Vec::with_capacity(asset.patches.len());
    let mut reduced = 0;
    let mut silhouette_native = 0;
    let mut max_error_px = 0f32;
    for p in &asset.patches {
        if !force && !p.levels.iter().skip(1).any(|l| l.normal <= normal_limit) {
            choices.push(0);
            continue;
        }
        let low = Vec3::from_array(p.low);
        let high = Vec3::from_array(p.high);
        let center = (low + high) * 0.5;
        let to_eye = pose.eye() - center;
        let radius = (high - low).length() * 0.5;
        let sine = p.cone[3] + radius / to_eye.length();
        let silhouette = Vec3::from_array([p.cone[0], p.cone[1], p.cone[2]])
            .dot(to_eye.normalize())
            .abs()
            <= sine;
        let mut best = 0;
        let mut error = 0.;
        if force || !silhouette {
            for (i, l) in p.levels.iter().enumerate().skip(1) {
                let bound = Bound {
                    low,
                    high,
                    error: l.error,
                    normal_error: 0.,
                    reduces: true,
                };
                let px = bound.pixels(pose, height);
                if (force || (px <= budget && l.normal <= normal_limit))
                    && l.count < p.levels[best].count
                {
                    best = i;
                    error = px;
                }
            }
        }
        if best > 0 {
            reduced += 1;
            max_error_px = max_error_px.max(error);
        } else if silhouette && p.levels.len() > 1 {
            silhouette_native += 1;
        }
        choices.push(best);
    }
    // Storage is level-major. Coalesce consecutive ranges, including all-native to one draw.
    let mut ranges: Vec<(u32, u32)> = vec![];
    let mut triangles = 0;
    for level in 0..5 {
        for (p, &choice) in asset.patches.iter().zip(&choices) {
            if choice == level {
                let l = &p.levels[choice];
                triangles += l.count / 3;
                if let Some(last) = ranges.last_mut()
                    && last.0 + last.1 == l.start
                {
                    last.1 += l.count;
                    continue;
                }
                ranges.push((l.start, l.count));
            }
        }
    }
    Selection {
        ranges,
        triangles,
        reduced,
        silhouette_native,
        max_error_px,
    }
}
