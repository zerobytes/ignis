//! Bounded JSON bridge for the pure kernel's authoring experiment; not a scene format.
use ignis::mesh_organic::{Collar, Fusion, Options, fused_junction};
use serde::Deserialize;
use std::io::Read;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InputCollar {
    id: String,
    center: [f32; 3],
    normal: [f32; 3],
    radial: [f32; 3],
    radius: f32,
    samples: u8,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    collars: [InputCollar; 3],
    center: [f32; 3],
    center_radius: f32,
    blend_radius: f32,
    guard_length: f32,
    max_displacement: f32,
    #[serde(default)]
    transition_lengths: [f32; 3],
    #[serde(default = "default_refinement")]
    refinement_levels: u8,
}
fn default_refinement() -> u8 {
    2
}
fn main() -> anyhow::Result<()> {
    let mut text = String::new();
    std::io::stdin()
        .take(2 * 1024 * 1024 + 1)
        .read_to_string(&mut text)?;
    anyhow::ensure!(text.len() <= 2 * 1024 * 1024, "Organic request exceeds2MiB");
    let request: Request = serde_json::from_str(&text)?;
    let collars = request.collars.map(|c| Collar {
        id: c.id,
        center: c.center,
        normal: c.normal,
        radial: c.radial,
        radius: c.radius,
        samples: c.samples,
    });
    let fusion = Fusion {
        center: request.center,
        center_radius: request.center_radius,
        blend_radius: request.blend_radius,
        guard_length: request.guard_length,
        max_displacement: request.max_displacement,
        transition_lengths: request.transition_lengths,
        ..Default::default()
    };
    let output = match fused_junction(
        &collars,
        Options {
            refinement_levels: request.refinement_levels,
            ..Default::default()
        },
        fusion,
    ) {
        Ok(junction) => serde_json::json!({"passed":true,"junction":junction}),
        Err(error) => serde_json::json!({"passed":false,"error":error.message}),
    };
    println!("{output}");
    Ok(())
}
