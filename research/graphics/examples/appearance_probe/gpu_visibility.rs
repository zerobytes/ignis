//! Current-frame conservative visibility with geometry-only exact-pixel gates.
use super::*;
#[path = "gpu_visibility_render.rs"]
mod render;
#[path = "gpu_visibility_scene.rs"]
mod scene;
use render::Renderer;
use scene::{Pose, World, hierarchy};
#[path = "gpu_visibility_edges.rs"]
mod edges;
const ROOT: &str = "artifacts/appearance-probe/gpu-visibility";
const MODES: [&str; 6] = [
    "full_grouped",
    "gpu_frustum_regions",
    "gpu_occlusion_objects",
    "gpu_occlusion_regions",
    "gpu_analytic_objects",
    "gpu_analytic_regions",
];
const SCENARIOS: [&str; 3] = ["open", "static_obstacles", "moving_occluder"];
fn step(frame: usize) -> usize {
    if frame < 13 { frame } else { 25 - frame }
}
fn compare(a: &[[f32; 4]], b: &[[f32; 4]]) -> serde_json::Value {
    let mut changed = 0;
    let mut ids = 0;
    let mut max = 0f32;
    let mut first = None;
    for (i, (a, b)) in a.iter().zip(b).enumerate() {
        if a != b {
            changed += 1;
            first.get_or_insert(i);
        }
        ids += usize::from(a[3] != b[3]);
        for c in 0..3 {
            max = max.max((a[c] - b[c]).abs());
        }
    }
    json!({"changed_pixels":changed,"id_mismatches":ids,"max_rgb_difference":max,"first_changed_index":first})
}
pub(super) fn run(mode: &str) -> Result<()> {
    fs::create_dir_all(ROOT)?;
    let scene = Scene::new(assets(false)?);
    let h = hierarchy(&scene, mode == "--bake-gpu-visibility")?;
    if mode == "--bake-gpu-visibility" {
        println!("{} regional nodes + 3 roots baked", h.regions);
        return Ok(());
    }
    let world = World::new(scene, h);
    let mut renderer = pollster::block_on(Renderer::new(&world))?;
    if mode == "--benchmark-gpu-visibility" {
        return edges::benchmark(&world, &mut renderer);
    }
    if mode == "--verify-gpu-visibility" {
        return edges::run(&world, &mut renderer);
    }
    let mut p = Pose::new();
    let root = Path::new(ROOT);
    let mut rows = Vec::new();
    for (resolution, size) in RESOLUTIONS.into_iter().enumerate() {
        for (scenario, name) in SCENARIOS.iter().enumerate() {
            for frame in 0..26 {
                world.pose(scenario, step(frame) as f32 / 12., size, &mut p);
                let id = format!("r{resolution}_s{scenario}_{frame:02}");
                let (reference, base) = renderer.capture(&p, 0)?;
                save(&root.join(format!("{id}_full.png")), &reference, size[0])?;
                let mut counts = vec![base];
                let mut checks = Vec::new();
                for m in 1..6 {
                    let (image, count) = renderer.capture(&p, m)?;
                    let error = compare(&reference, &image);
                    if m >= 2 || error["changed_pixels"] != 0 {
                        save(&root.join(format!("{id}_{m}.png")), &image, size[0])?;
                    }
                    ensure!(
                        error["changed_pixels"] == 0,
                        "pixel change {id} mode{m}: {error}"
                    );
                    checks.push(error);
                    counts.push(count);
                }
                if frame == 0 {
                    let (repeat, _) = renderer.capture(&p, 5)?;
                    ensure!(reference == repeat, "repeat changed");
                }
                let timings = if [0, 3, 6, 9, 12, 13, 16, 19, 22, 25].contains(&frame) {
                    Some(renderer.bench(&world, scenario, frame, size)?)
                } else {
                    None
                };
                println!(
                    "{id}: {} -> {} / {} triangles; exact pixels",
                    counts[0]["triangles"], counts[2]["triangles"], counts[3]["triangles"]
                );
                rows.push(json!({"id":id,"resolution":resolution,"size":size,"scenario":name,"scenario_index":scenario,"frame":frame,"step":step(frame),"eye":p.camera.eye,"counts":counts,"checks":checks,"timings":timings}));
            }
        }
    }
    fs::write(
        root.join("report.json"),
        serde_json::to_vec_pretty(
            &json!({"schema":1,"adapter":renderer.gpu.adapter,"modes":MODES,"scenarios":SCENARIOS,"region_count":world.hierarchy.regions,"static_gpu_bytes":renderer.static_bytes,"rows":rows,"visibility_readback_in_render_loop":false,"appearance_summaries":false,"timestamp_scope":"occluders + depth hierarchy + culling + indirect geometry","throughput_scope":"8 consecutive moving poses, current-frame depth each time"}),
        )?,
    )?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reverse_depth_hierarchy_retains_holes_and_far_edges() {
        let depth = [0.8f32, 0.7, 0., 0.9];
        let coarse = depth.into_iter().fold(1., f32::min);
        assert_eq!(coarse, 0.);
        assert!(0.1 + 0.000002 >= coarse);
        let full = [0.8f32, 0.7, 0.6, 0.9].into_iter().fold(1., f32::min);
        assert!(0.5 + 0.000002 < full);
        assert!(0.65 + 0.000002 >= full);
    }
    #[test]
    fn zero_pixel_gate_includes_color_and_identity() {
        let a = [[0.1f32, 0.2, 0.3, 1.]];
        let mut b = a;
        b[0][0] = f32::from_bits(a[0][0].to_bits() + 1);
        assert_eq!(compare(&a, &b)["changed_pixels"], 1);
        b = a;
        b[0][3] = 2.;
        assert_eq!(compare(&a, &b)["id_mismatches"], 1);
    }
    #[test]
    fn prefix_compaction_is_stable_and_bounded() {
        for mask in [0u32, u32::MAX, 0x55555555, 1 << 31] {
            let mut packed = [u32::MAX; 32];
            for lane in 0..32 {
                if (mask >> lane) & 1 == 1 {
                    let prefix = (mask & ((1u64 << lane) - 1) as u32).count_ones();
                    packed[prefix as usize] = lane;
                }
            }
            let expected = (0..32).filter(|i| (mask >> i) & 1 == 1).collect::<Vec<_>>();
            assert_eq!(&packed[..expected.len()], expected);
        }
    }
    #[test]
    fn physical_rect_mip_covers_all_samples_including_padding() {
        for width in [1728u32, 3456] {
            for (lo, hi) in [(0, width - 1), (width - 3, width - 1), (41, 42), (100, 900)] {
                let level = (((hi - lo + 1) as f32 / 4.).max(1.).log2().ceil()) as u32;
                let cell = 4 << level;
                assert!(hi / cell - lo / cell <= 1);
                for x in lo..=hi {
                    assert!(x / cell >= lo / cell && x / cell <= hi / cell);
                }
            }
        }
    }
}
