//! Public material asset and authoring operations, shared by MCP and the editor.
use super::{Material, recipe::*};
use crate::model::{Result, invalid};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Execution {
    #[default]
    Runtime,
    Fixed {
        resolution: u16,
    },
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Preset {
    Stone,
    Wood,
    PaintedMetal,
    Ceramic,
}
impl Preset {
    pub const ALL: [Self; 4] = [Self::Stone, Self::Wood, Self::PaintedMetal, Self::Ceramic];
    pub fn label(self) -> &'static str {
        match self {
            Self::Stone => "Pedra",
            Self::Wood => "Madeira",
            Self::PaintedMetal => "Metal pintado",
            Self::Ceramic => "Cerâmica",
        }
    }
    pub fn recipe(self) -> Recipe {
        fn surface(color: [f32; 3], roughness: f32, metal: f32, height: f32) -> Sample {
            Sample {
                color_roughness: [color[0], color[1], color[2], roughness],
                response: [metal, height, 1., 1.],
                ..Default::default()
            }
        }
        let mut r = Recipe {
            height_m: 0.018,
            relief: Relief::Parallax,
            ..Default::default()
        };
        match self {
            Self::Stone => {
                r.base = surface([0.12, 0.105, 0.085], 0.95, 0., 0.05);
                r.layers.push(Layer {
                    field: Field::Cells,
                    frequency: [4, 4],
                    octaves: 1,
                    threshold: [0.05, 0.8],
                    target: surface([0.65, 0.55, 0.4], 0.72, 0., 0.95),
                    ..Default::default()
                });
                r.layers.push(Layer {
                    frequency: [26, 26],
                    strength: 0.18,
                    target: surface([0.3, 0.28, 0.25], 0.9, 0., 0.5),
                    ..Default::default()
                });
            }
            Self::Wood => {
                r.height_m = 0.003;
                r.relief = Relief::Bump;
                r.base = surface([0.12, 0.035, 0.012], 0.65, 0., 0.2);
                r.layers.push(Layer {
                    field: Field::Waves,
                    frequency: [18, 2],
                    octaves: 2,
                    warp: 0.08,
                    target: surface([0.62, 0.28, 0.07], 0.43, 0., 0.8),
                    ..Default::default()
                });
                r.layers.push(Layer {
                    frequency: [32, 4],
                    strength: 0.22,
                    target: surface([0.25, 0.1, 0.025], 0.8, 0., 0.45),
                    ..Default::default()
                });
            }
            Self::PaintedMetal => {
                r.height_m = 0.002;
                r.relief = Relief::Bump;
                r.base = surface([0.07, 0.24, 0.19], 0.45, 0., 0.8);
                r.layers.push(Layer {
                    field: Field::Noise,
                    frequency: [7, 7],
                    threshold: [0.43, 0.63],
                    warp: 0.35,
                    target: surface([0.4, 0.11, 0.025], 0.95, 0., 0.4),
                    ..Default::default()
                });
                r.layers.push(Layer {
                    field: Field::Ridged,
                    frequency: [3, 3],
                    threshold: [0.78, 0.95],
                    seed: 47,
                    target: surface([0.62, 0.67, 0.7], 0.28, 1., 0.65),
                    ..Default::default()
                });
            }
            Self::Ceramic => {
                r.height_m = 0.012;
                r.base = surface([0.1, 0.085, 0.065], 0.95, 0., 0.02);
                r.layers.push(Layer {
                    field: Field::Bricks,
                    frequency: [6, 6],
                    octaves: 1,
                    threshold: [0.2, 0.65],
                    target: surface([0.08, 0.48, 0.53], 0.23, 0., 0.95),
                    ..Default::default()
                });
                r.layers.push(Layer {
                    frequency: [24, 24],
                    strength: 0.06,
                    target: surface([0.5, 0.7, 0.68], 0.35, 0., 0.7),
                    ..Default::default()
                });
            }
        }
        r
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    BakeDisplacement {
        instance: String,
        id: String,
        reference: crate::mesh_topology::Reference,
    },
    Inspect {
        asset: String,
    },
    Generate {
        asset: String,
        recipe: Box<Recipe>,
        #[serde(default)]
        execution: Execution,
    },
    Preset {
        asset: String,
        preset: Preset,
        #[serde(default)]
        seed: Option<u32>,
        #[serde(default)]
        unique_instances: bool,
        #[serde(default)]
        execution: Execution,
    },
    SetExecution {
        asset: String,
        execution: Execution,
    },
    Save {
        asset: String,
        path: String,
    },
    Load {
        path: String,
        #[serde(default)]
        asset: Option<String>,
    },
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    version: u16,
    material: Material,
}
pub fn material(recipe: Recipe, execution: Execution) -> Result<Material> {
    let p = Procedural::runtime(recipe);
    p.validate()?;
    let p = match execution {
        Execution::Runtime => p,
        Execution::Fixed { resolution } => p.bake(resolution)?,
    };
    let surface = p.translucent().then(|| super::Surface {
        mode: super::BlendMode::Blend,
        ..Default::default()
    });
    let m = Material {
        procedural: Some(Arc::new(p)),
        surface,
        ..Default::default()
    };
    m.validate()?;
    Ok(m)
}
pub fn encode(material: &Material) -> Result<Vec<u8>> {
    material.validate()?;
    let bytes = serde_json::to_vec(&File {
        version: VERSION,
        material: material.clone(),
    })
    .map_err(|e| invalid(e.to_string()))?;
    if bytes.len() > MAX_FILE_BYTES {
        return Err(invalid("Material file exceeds32MiB"));
    }
    Ok(bytes)
}
pub fn decode(bytes: &[u8]) -> Result<Material> {
    if bytes.len() > MAX_FILE_BYTES {
        return Err(invalid("Material file exceeds32MiB"));
    }
    let f: File = serde_json::from_slice(bytes).map_err(|e| invalid(e.to_string()))?;
    if f.version != VERSION {
        return Err(invalid("Unsupported material asset version"));
    }
    f.material.validate()?;
    Ok(f.material)
}
pub fn inspect(m: &Material) -> serde_json::Value {
    if let Some(p) = &m.procedural {
        serde_json::json!({"format":VERSION,"execution":if p.fixed.is_some(){"fixed"}else{"runtime"},"recipe":p.recipe,"resolution":p.fixed.as_ref().map(|f|f.resolution),"sample_bytes":p.fixed.as_ref().map_or(0,|f|f.samples.len()*std::mem::size_of::<Sample>()),"surface":m.surface,"world_changed":false})
    } else {
        serde_json::json!({"execution":"legacy","material":m,"world_changed":false})
    }
}

/// Explicit geometry generation. Preserve source and rest-domain coordinates;
/// reject a local triangle inversion rather than publishing a broken candidate.
pub fn displace(
    source: &crate::meshes::Asset,
    id: &str,
    instance: &str,
    world_scale: f32,
) -> Result<crate::meshes::Asset> {
    use glam::Vec3;
    source.validate()?;
    if !world_scale.is_finite() || world_scale <= 0. {
        return Err(invalid(
            "Displacement requires positive finite instance scale",
        ));
    }
    if source.rig.is_some() || !source.clips.is_empty() {
        return Err(invalid(
            "Displacement bake requires a static unrigged source",
        ));
    }
    let p = source
        .material
        .procedural
        .as_ref()
        .ok_or_else(|| invalid("Source has no material recipe"))?;
    let unique = p.instance(instance);
    let p = unique.as_ref().unwrap_or(p);
    if p.recipe.height_m == 0. {
        return Err(invalid(
            "Displacement bake requires positive material depth",
        ));
    }
    let old = &source.geometry;
    let mut normals = vec![Vec3::ZERO; old.positions.len()];
    crate::mesh_normals::accumulate(&old.triangles, &old.positions, &mut normals);
    let coordinates = old.material_coordinates.as_ref().unwrap_or(&old.positions);
    let positions = old
        .positions
        .iter()
        .zip(&normals)
        .zip(coordinates)
        .map(|((&position, normal), &rest)| {
            let height = p.sample(p.recipe.coordinate(rest), 0.).response[1];
            (Vec3::from(position)
                + normal.normalize_or_zero() * ((height - 1.) * p.recipe.height_m / world_scale))
                .to_array()
        })
        .collect::<Vec<_>>();
    for triangle in &old.triangles {
        let before = triangle.map(|i| Vec3::from(old.positions[i as usize]));
        let after = triangle.map(|i| Vec3::from(positions[i as usize]));
        let a = (before[1] - before[0]).cross(before[2] - before[0]);
        let b = (after[1] - after[0]).cross(after[2] - after[0]);
        if a.length_squared() > 1e-18 && (b.length_squared() < 1e-18 || a.dot(b) <= 0.) {
            return Err(invalid(
                "Displacement would invert/collapse a triangle; reduce depth or refine the source",
            ));
        }
    }
    let mut out = source.clone();
    out.id = id.to_owned();
    out.geometry = Arc::new(crate::meshes::Geometry {
        positions,
        triangles: old.triangles.clone(),
        material_coordinates: Some(coordinates.clone()),
    });
    let mut p = p.clone();
    p.recipe.height_m = 0.;
    p.recipe.unique_instances = false;
    out.material.procedural = Some(Arc::new(p));
    out.validate()?;
    Ok(out)
}
