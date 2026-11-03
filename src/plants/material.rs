//! Explicit, reversible material-domain authoring, independent of geometric frames.
use crate::{
    mesh_material_finish::Affine,
    meshes,
    model::{Result, invalid},
};
use std::sync::Arc;

pub(super) fn transform(asset: &mut meshes::Asset, frame: &Affine) -> Result<()> {
    if frame
        .columns
        .iter()
        .flatten()
        .chain(frame.offset.iter())
        .any(|v| !v.is_finite() || v.abs() > 10000.)
    {
        return Err(invalid(
            "Source material frame must be finite and within ±10000",
        ));
    }
    let coords = asset
        .geometry
        .material_coordinates
        .as_ref()
        .ok_or_else(|| invalid("Source material frame requires explicit material coordinates"))?;
    // Validate every result before publishing any coordinate or cloning Geometry.
    let mut out = Vec::with_capacity(coords.len());
    for &p in coords {
        let q = std::array::from_fn(|i| {
            (f64::from(frame.offset[i])
                + (0..3)
                    .map(|j| f64::from(frame.columns[j][i]) * f64::from(p[j]))
                    .sum::<f64>()) as f32
        });
        if q.iter().any(|v| !v.is_finite() || v.abs() > 10000.) {
            return Err(invalid(
                "Transformed source material exceeds finite coordinate bounds",
            ));
        }
        out.push(q);
    }
    Arc::make_mut(&mut asset.geometry).material_coordinates = Some(out);
    Ok(())
}
