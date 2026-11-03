//! Continuous support under the swept footprint, independent of collision/path finding.
use crate::model::{BodyKind, Entity, Scene};
use glam::Vec3;

const HEIGHT_TOLERANCE: f32 = 0.06;
const EDGE_TOLERANCE: f32 = 0.01;
const MAX_SURFACES: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Support {
    Continuous,
    HeightMismatch,
    Gap,
    ComplexityLimit,
}
impl Support {
    pub fn continuous(self) -> bool {
        self == Self::Continuous
    }
    pub fn rejection(self) -> Option<&'static str> {
        match self {
            Self::Continuous => None,
            Self::HeightMismatch => Some("walk_height_mismatch"),
            Self::Gap => Some("walk_support_gap"),
            Self::ComplexityLimit => Some("walk_support_limit"),
        }
    }
}

fn supported_by(actor: &Entity, p: Vec3, surface: &Entity) -> bool {
    if surface.body != Some(BodyKind::Fixed) || !surface.geometry.is_box() {
        return false;
    }
    let center = Vec3::from(surface.transform.position);
    let half = Vec3::from(surface.transform.size) * 0.5;
    (p.y - center.y - half.y).abs() <= HEIGHT_TOLERANCE
        && [0, 2].into_iter().all(|axis| {
            (p[axis] - center[axis]).abs() + actor.transform.size[axis] * 0.5
                <= half[axis] + EDGE_TOLERANCE
        })
}

#[derive(Clone, Copy, Default)]
struct Rect {
    min_x: f32,
    max_x: f32,
    min_z: f32,
    max_z: f32,
}

/// Z extent of the swept axis-aligned footprint at a vertical X slice.
fn z_extent(x: f32, a: Vec3, b: Vec3, half: Vec3) -> (f32, f32) {
    let dx = b.x - a.x;
    let (lo, hi) = if dx == 0. {
        (0., 1.)
    } else {
        let t0 = (x - half.x - a.x) / dx;
        let t1 = (x + half.x - a.x) / dx;
        (t0.min(t1).clamp(0., 1.), t0.max(t1).clamp(0., 1.))
    };
    let z0 = a.z + (b.z - a.z) * lo;
    let z1 = a.z + (b.z - a.z) * hi;
    (z0.min(z1) - half.z, z0.max(z1) + half.z)
}

/// Feet coordinates. Callers validate authored geometry and separately check body clearance.
/// Box support is convex, so checking both endpoints suffices for the common single-box case.
pub fn query(scene: &Scene, actor: &Entity, a: Vec3, b: Vec3) -> Support {
    if (a.y - b.y).abs() > HEIGHT_TOLERANCE {
        return Support::HeightMismatch;
    }
    if scene
        .entities
        .iter()
        .any(|s| supported_by(actor, a, s) && supported_by(actor, b, s))
    {
        return Support::Continuous;
    }

    // Shrink the footprint by the existing edge tolerance instead of growing each surface:
    // growing adjacent rectangles would incorrectly close real gaps between them.
    let half = (Vec3::from(actor.transform.size) * 0.5 - Vec3::splat(EDGE_TOLERANCE))
        .max(Vec3::splat(f32::EPSILON));
    let min = a.min(b) - half;
    let max = a.max(b) + half;
    let mut surfaces = [Rect::default(); MAX_SURFACES];
    let mut count = 0;
    let mut cuts = [0.; MAX_SURFACES * 2 + 4];
    cuts[..4].copy_from_slice(&[a.x - half.x, a.x + half.x, b.x - half.x, b.x + half.x]);
    let mut cut_count = 4;
    for s in &scene.entities {
        if s.body != Some(BodyKind::Fixed) || !s.geometry.is_box() {
            continue;
        }
        let center = Vec3::from(s.transform.position);
        let size = Vec3::from(s.transform.size) * 0.5;
        let top = center.y + size.y;
        if (top - a.y).abs() > HEIGHT_TOLERANCE || (top - b.y).abs() > HEIGHT_TOLERANCE {
            continue;
        }
        let rect = Rect {
            min_x: (center.x - size.x).max(min.x),
            max_x: (center.x + size.x).min(max.x),
            min_z: (center.z - size.z).max(min.z),
            max_z: (center.z + size.z).min(max.z),
        };
        if rect.min_x >= rect.max_x || rect.min_z >= rect.max_z {
            continue;
        }
        if count == MAX_SURFACES {
            return Support::ComplexityLimit;
        }
        surfaces[count] = rect;
        count += 1;
        cuts[cut_count] = rect.min_x;
        cuts[cut_count + 1] = rect.max_x;
        cut_count += 2;
    }
    surfaces[..count].sort_unstable_by(|a, b| {
        a.min_z
            .total_cmp(&b.min_z)
            .then(a.max_z.total_cmp(&b.max_z))
    });
    cuts[..cut_count].sort_unstable_by(f32::total_cmp);
    for slab in cuts[..cut_count].windows(2) {
        let (left, right) = (slab[0], slab[1]);
        if left == right {
            continue;
        }
        // Surface membership is constant within this slab. The footprint boundaries are
        // linear here, because cuts also contain its four changes of slope in X.
        let l = z_extent(left, a, b, half);
        let r = z_extent(right, a, b, half);
        let mut covered = l.0.min(r.0);
        let required = l.1.max(r.1);
        for s in &surfaces[..count] {
            if s.min_x > left || s.max_x < right || s.max_z < covered {
                continue;
            }
            if s.min_z > covered {
                break;
            }
            covered = covered.max(s.max_z);
            if covered >= required {
                break;
            }
        }
        if covered < required {
            return Support::Gap;
        }
    }
    Support::Continuous
}
