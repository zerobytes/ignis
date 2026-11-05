//! Cold, disjoint slope partition of a native static mesh. No overlapping grass shell.
use crate::{
    engine::SceneEdit,
    meshes::{self, Geometry},
    model::{Result, invalid},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
/// Lift strata without multiplying variation inside each original height band.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RandomHeights {
    pub min: f64,
    pub max: f64,
    pub seed: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TerraceLift {
    pub spacing: f64,
    pub factor: f64,
    pub phase: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub random_heights: Option<RandomHeights>,
}
pub fn lift(g: &Geometry, c: &TerraceLift, origin_y: f64) -> Result<Geometry> {
    if !c.spacing.is_finite()
        || c.spacing <= 0.
        || !c.factor.is_finite()
        || !(1. ..=1000.).contains(&c.factor)
        || !c.phase.is_finite()
    {
        return Err(invalid(
            "Terrace lift requires positive spacing, factor1..1000 and finite phase",
        ));
    }
    let profile = if let Some(r) = &c.random_heights {
        if !r.min.is_finite() || !r.max.is_finite() || r.min <= 0. || r.max < r.min {
            return Err(invalid("Random terrace heights require0<min<=max"));
        }
        let mut lo = 0i64;
        let mut hi = 0i64;
        for p in &g.positions {
            let b = ((p[1] as f64 + origin_y - c.phase) / c.spacing).floor();
            if !b.is_finite() || b.abs() > 4096. {
                return Err(invalid(
                    "Random height authoring window exceeds4096bands from datum",
                ));
            }
            lo = lo.min(b as i64);
            hi = hi.max(b as i64 + 1);
        }
        let mut levels = vec![0.; (hi - lo + 1) as usize];
        for b in lo..hi {
            let h =
                r.min + (r.max - r.min) * crate::mesh_chamfer::random(r.seed ^ (b as u64)) as f64;
            let i = (b - lo) as usize;
            levels[i + 1] = levels[i] + h;
        }
        let datum = levels[(-lo) as usize];
        for v in &mut levels {
            *v -= datum;
        }
        Some((lo, levels))
    } else {
        None
    };
    let mut out = g.clone();
    for p in &mut out.positions {
        let y = p[1] as f64 + origin_y;
        let band = ((y - c.phase) / c.spacing).floor();
        p[1] = if let Some((lo, levels)) = &profile {
            let i = (band as i64 - lo) as usize;
            let fraction = (y - c.phase) / c.spacing - band;
            (c.phase + levels[i] + fraction * (levels[i + 1] - levels[i]) - origin_y) as f32
        } else {
            (p[1] as f64 + (c.factor - 1.) * c.spacing * band) as f32
        };
    }
    // Material coordinates describe the new rest surface, not the pre-lift surface.
    out.material_coordinates = None;
    out.validate()?;
    Ok(out)
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Style {
    pub instance: String,
    pub wall_asset: String,
    pub top_asset: String,
    pub top_instance: String,
    pub min_up: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chamfer: Option<crate::mesh_chamfer::Config>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terrace_lift: Option<TerraceLift>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wall_rock: Option<crate::mesh_wall_rock::Config>,
    pub walls: crate::materials::Material,
    pub top: crate::materials::Material,
    pub wall_color: [f32; 3],
    pub top_color: [f32; 3],
}
pub fn plan(scene: &crate::model::Scene, s: Style) -> Result<(Vec<SceneEdit>, serde_json::Value)> {
    if !s.min_up.is_finite() || !(0. ..=1.).contains(&s.min_up) {
        return Err(invalid("Slope threshold requires0..1"));
    }
    let instance = scene
        .meshes
        .instances
        .iter()
        .find(|i| i.id == s.instance)
        .ok_or_else(|| invalid("Unknown slope instance"))?;
    let asset = scene
        .meshes
        .assets
        .iter()
        .find(|a| a.id == instance.asset)
        .unwrap();
    if asset.rig.is_some() || !asset.clips.is_empty() || instance.attachment.is_some() {
        return Err(invalid(
            "Slope partition requires an unattached static mesh",
        ));
    }
    if s.wall_asset == s.top_asset
        || scene
            .meshes
            .assets
            .iter()
            .any(|a| a.id == s.wall_asset || a.id == s.top_asset)
        || scene
            .meshes
            .instances
            .iter()
            .any(|i| i.id == s.top_instance)
    {
        return Err(invalid("Slope output IDs must be distinct and unused"));
    }
    if (s.terrace_lift.is_some() || s.wall_rock.is_some())
        && (instance.rotation != [0., 0., 0., 1.] || instance.scale != 1.)
    {
        return Err(invalid(
            "Terrace/rock authoring requires world-aligned unit-scale instance",
        ));
    }
    let lifted = s
        .terrace_lift
        .as_ref()
        .map(|c| lift(&asset.geometry, c, instance.position[1] as f64))
        .transpose()?;
    let source = lifted.as_ref().unwrap_or(&asset.geometry);
    let rock = s
        .wall_rock
        .as_ref()
        .map(|c| crate::mesh_wall_rock::apply(source, &asset.geometry, c, instance.position))
        .transpose()?;
    let source = rock.as_ref().unwrap_or(source);
    let refined = s
        .chamfer
        .as_ref()
        .map(|c| crate::mesh_chamfer::apply(source, c))
        .transpose()?;
    let g = refined.as_ref().unwrap_or(source);
    if g.positions.len() > 1_048_576 || g.triangles.len() > 2_000_000 {
        return Err(invalid(
            "Slope partition exceeds cold-operation budget; partition the source into smaller authored assets",
        ));
    }
    let mut outputs = [
        Geometry {
            positions: vec![],
            triangles: vec![],
            material_coordinates: Some(vec![]),
        },
        Geometry {
            positions: vec![],
            triangles: vec![],
            material_coordinates: Some(vec![]),
        },
    ];
    let mut maps = [
        vec![u32::MAX; g.positions.len()],
        vec![u32::MAX; g.positions.len()],
    ];
    let rotation = glam::Quat::from_array(instance.rotation);
    for &face in &g.triangles {
        let p = face.map(|i| glam::Vec3::from_array(g.positions[i as usize]));
        let normal = (p[1] - p[0]).cross(p[2] - p[0]).normalize_or_zero();
        let group = usize::from((rotation * normal).y >= s.min_up);
        let out = &mut outputs[group];
        let map = &mut maps[group];
        let triangle = face.map(|v| {
            let slot = &mut map[v as usize];
            if *slot == u32::MAX {
                *slot = out.positions.len() as u32;
                out.positions.push(g.positions[v as usize]);
                if let Some(uv) = out.material_coordinates.as_mut() {
                    uv.push(g.material_coordinates.as_ref().map_or_else(
                        || {
                            (rotation
                                * (glam::Vec3::from_array(g.positions[v as usize])
                                    * instance.scale)
                                + glam::Vec3::from_array(instance.position))
                            .to_array()
                        },
                        |uv| uv[v as usize],
                    ));
                }
            }
            *slot
        });
        out.triangles.push(triangle);
    }
    if outputs.iter().any(|o| o.triangles.is_empty()) {
        return Err(invalid(
            "Slope split requires both top and wall faces at the selected threshold",
        ));
    }
    for o in &outputs {
        o.validate()?;
    }
    let counts = outputs.each_ref().map(|o| o.triangles.len());
    let [walls, top] = outputs;
    let mut wall_asset = asset.clone();
    wall_asset.id = s.wall_asset.clone();
    wall_asset.geometry = Arc::new(walls);
    wall_asset.material = s.walls;
    let mut top_asset = asset.clone();
    top_asset.id = s.top_asset.clone();
    top_asset.geometry = Arc::new(top);
    top_asset.material = s.top;
    let mut wall_instance = instance.clone();
    wall_instance.asset = s.wall_asset;
    wall_instance.color = s.wall_color;
    let mut top_instance = instance.clone();
    top_instance.id = s.top_instance.clone();
    top_instance.asset = s.top_asset;
    top_instance.color = s.top_color;
    let mut edits = vec![
        SceneEdit::Mesh {
            edit: meshes::Edit::PutAsset { asset: wall_asset },
        },
        SceneEdit::Mesh {
            edit: meshes::Edit::PutAsset { asset: top_asset },
        },
        SceneEdit::Mesh {
            edit: meshes::Edit::PutInstance {
                instance: wall_instance,
            },
        },
        SceneEdit::Mesh {
            edit: meshes::Edit::PutInstance {
                instance: top_instance,
            },
        },
    ];
    if scene.collision_meshes.contains(&s.instance) {
        edits.push(SceneEdit::SetMeshCollision {
            instance: s.top_instance,
            enabled: true,
        });
    }
    if scene
        .meshes
        .instances
        .iter()
        .filter(|i| i.asset == asset.id)
        .count()
        == 1
    {
        edits.push(SceneEdit::Mesh {
            edit: meshes::Edit::RemoveAsset {
                id: asset.id.clone(),
            },
        });
    }
    Ok((
        edits,
        serde_json::json!({"wall_triangles":counts[0],"top_triangles":counts[1],"source_triangles":g.triangles.len(),"disjoint":true,"geometry_displaced":s.chamfer.is_some() || s.terrace_lift.is_some() || s.wall_rock.is_some(),"terrace_lift":s.terrace_lift,"wall_rock":s.wall_rock,"input_triangles":asset.geometry.triangles.len(),"chamfer":s.chamfer}),
    ))
}
