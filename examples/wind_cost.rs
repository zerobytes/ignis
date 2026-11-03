//! CPU field and authoritative physics costs; no viewport/FPS inference.
use ignis::{
    engine::{Engine, ExecutionMode},
    model::Scene,
    wind::CompiledField,
};
use std::{hint::black_box, sync::Arc, time::Instant};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "examples/wind-leaves.scene.json".into());
    let source: Scene = serde_json::from_reader(std::fs::File::open(path)?)?;
    let field = CompiledField::new(&source.wind.field)?;
    let field = field.at_time(3.);
    let started = Instant::now();
    let mut checksum = glam::DVec3::ZERO;
    for i in 0..100_000 {
        checksum += field.sample(glam::DVec3::new(
            (i % 137) as f64 * 0.07,
            (i % 41) as f64 * 0.13,
            (i % 97) as f64 * 0.09,
        ));
    }
    black_box(checksum);
    let field_ns = started.elapsed().as_nanos() as f64 / 100_000.;
    let mut cases = Vec::new();
    for count in [1, 64, 256] {
        let mut scene = source.clone();
        let template = scene.meshes.instances[0].clone();
        let body = scene.wind.bodies[0].clone();
        let mut document = (*scene.wind).clone();
        scene.meshes.instances.clear();
        document.bodies.clear();
        for i in 0..count {
            let id = format!("cost-{i}");
            let mut instance = template.clone();
            instance.id = id.clone();
            instance.position = [
                (i % 8) as f32 * 0.6 - 2.1,
                0.6 + ((i / 64) % 4) as f32 * 0.7,
                ((i / 8) % 8) as f32 * 0.6 - 2.1,
            ];
            let mut b = body.clone();
            b.instance = id;
            scene.meshes.instances.push(instance);
            document.bodies.push(b);
        }
        let work = document.work();
        scene.wind = Arc::new(document);
        let mut engine = Engine::new_with_mode(scene, ExecutionMode::Continuous)?;
        engine.advance(120)?;
        let mut samples = Vec::new();
        for _ in 0..12 {
            let started = Instant::now();
            engine.advance(60)?;
            samples.push(started.elapsed().as_secs_f64() * 1000. / 60.);
        }
        samples.sort_by(f64::total_cmp);
        black_box(engine);
        cases.push(serde_json::json!({"bodies":count,"samples":samples.len(),"ticks_per_sample":60,
            "median_cpu_ms_per_tick":samples[samples.len()/2],"p95_cpu_ms_per_tick":samples[samples.len()-1],
            "field_work_per_tick":work}));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(
            &serde_json::json!({"build_id":env!("IGNIS_BUILD_ID"),"field_ns_per_query":field_ns,"cases":cases,"scope":"CPU authoritative tick, no rendering, no FPS claim"})
        )?
    );
    Ok(())
}
