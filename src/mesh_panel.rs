//! Bounded Cartesian solid panels. Openings remove material through both skins.
use crate::{
    meshes::Geometry,
    model::{Result, invalid},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const MAX_OPENINGS: usize = 16;
const MIN_FEATURE: f32 = 0.001;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Opening {
    /// Local XY bounds. Only the bottom may coincide with the panel perimeter.
    pub min: [f32; 2],
    pub max: [f32; 2],
}

pub fn build(dimensions: [f32; 3], openings: &[Opening]) -> Result<Geometry> {
    if dimensions
        .iter()
        .any(|v| !v.is_finite() || !(0.01..=100.).contains(v))
        || openings.len() > MAX_OPENINGS
    {
        return Err(invalid(
            "Panel dimensions require 0.01..100 and at most16 openings",
        ));
    }
    let [w, h, d] = dimensions;
    let lo = [-w / 2., -h / 2.];
    let hi = [w / 2., h / 2.];
    for (i, a) in openings.iter().enumerate() {
        if a.min.iter().chain(&a.max).any(|v| !v.is_finite())
            || (0..2).any(|k| a.max[k] - a.min[k] < MIN_FEATURE)
            || a.min[0] - lo[0] < MIN_FEATURE
            || hi[0] - a.max[0] < MIN_FEATURE
            || hi[1] - a.max[1] < MIN_FEATURE
            || (a.min[1] != lo[1] && a.min[1] - lo[1] < MIN_FEATURE)
        {
            return Err(invalid(format!(
                "Panel opening {i} requires positive spans and0.001 margins; only bottom may meet perimeter"
            )));
        }
        for b in &openings[..i] {
            if !(0..2)
                .any(|k| a.min[k] - b.max[k] >= MIN_FEATURE || b.min[k] - a.max[k] >= MIN_FEATURE)
            {
                return Err(invalid(
                    "Panel openings must be disjoint with at least0.001 separation on one axis",
                ));
            }
        }
    }
    let mut xs = vec![lo[0], hi[0]];
    let mut ys = vec![lo[1], hi[1]];
    for a in openings {
        xs.extend([a.min[0], a.max[0]]);
        ys.extend([a.min[1], a.max[1]]);
    }
    for axis in [&mut xs, &mut ys] {
        axis.sort_by(f32::total_cmp);
        axis.dedup();
    }
    let nx = xs.len() - 1;
    let ny = ys.len() - 1;
    let mut filled = vec![true; nx * ny];
    for y in 0..ny {
        for x in 0..nx {
            filled[y * nx + x] = !openings.iter().any(|a| {
                xs[x] >= a.min[0]
                    && xs[x + 1] <= a.max[0]
                    && ys[y] >= a.min[1]
                    && ys[y + 1] <= a.max[1]
            });
        }
    }
    let mut geometry = Geometry {
        positions: vec![],
        triangles: vec![],
        material_coordinates: None,
    };
    // Integer lattice keys share corners without proximity tolerances or duplicated skins.
    let mut nodes = vec![None; 2 * xs.len() * ys.len()];
    let mut quad = |corners: [(usize, usize, usize); 4]| {
        let ids = corners.map(|(x, y, z)| {
            let slot = (z * ys.len() + y) * xs.len() + x;
            *nodes[slot].get_or_insert_with(|| {
                let id = geometry.positions.len() as u32;
                geometry
                    .positions
                    .push([xs[x], ys[y], if z == 0 { -d / 2. } else { d / 2. }]);
                id
            })
        });
        geometry
            .triangles
            .extend([[ids[0], ids[1], ids[2]], [ids[0], ids[2], ids[3]]]);
    };
    for y in 0..ny {
        for x in 0..nx {
            if !filled[y * nx + x] {
                continue;
            }
            quad([(x, y, 1), (x + 1, y, 1), (x + 1, y + 1, 1), (x, y + 1, 1)]);
            quad([(x, y, 0), (x, y + 1, 0), (x + 1, y + 1, 0), (x + 1, y, 0)]);
            if x == 0 || !filled[y * nx + x - 1] {
                quad([(x, y, 1), (x, y + 1, 1), (x, y + 1, 0), (x, y, 0)]);
            }
            if x + 1 == nx || !filled[y * nx + x + 1] {
                quad([
                    (x + 1, y, 0),
                    (x + 1, y + 1, 0),
                    (x + 1, y + 1, 1),
                    (x + 1, y, 1),
                ]);
            }
            if y == 0 || !filled[(y - 1) * nx + x] {
                quad([(x, y, 1), (x, y, 0), (x + 1, y, 0), (x + 1, y, 1)]);
            }
            if y + 1 == ny || !filled[(y + 1) * nx + x] {
                quad([
                    (x, y + 1, 0),
                    (x, y + 1, 1),
                    (x + 1, y + 1, 1),
                    (x + 1, y + 1, 0),
                ]);
            }
        }
    }
    geometry.validate()?;
    Ok(geometry)
}
