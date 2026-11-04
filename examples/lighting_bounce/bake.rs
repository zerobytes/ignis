//! Bounded build-time atlas experiment, deliberately outside production formats.
use super::*;
use serde::{Deserialize, Serialize};
const MAGIC: &[u8; 8] = b"IGLBAKE1";
const RESOLUTIONS: [u32; 3] = [16, 32, 64];
#[derive(Clone, Serialize, Deserialize)]
struct Header {
    version: u32,
    geometry: String,
    kernel: String,
    sun: [f32; 3],
    energy: [f32; 3],
    side: u32,
    samples: u32,
    objects: u32,
    payload_sha256: String,
}
fn digest(bytes: &[u8]) -> String {
    ring::digest::digest(&ring::digest::SHA256, bytes)
        .as_ref()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn geometry_key(boxes: &[BoxData]) -> String {
    digest(bytemuck::cast_slice(boxes))
}
fn kernel_key() -> String {
    digest(concat!(include_str!("transport.wgsl"), include_str!("bake.wgsl")).as_bytes())
}
fn decode(bytes: &[u8]) -> Result<(Header, Vec<u8>)> {
    ensure!(
        bytes.len() >= 12 && bytes.len() <= 64 * 1024 * 1024,
        "Invalid bake envelope"
    );
    ensure!(&bytes[..8] == MAGIC, "Invalid bake magic");
    let n = u32::from_le_bytes(bytes[8..12].try_into()?) as usize;
    ensure!(
        n <= 16384 && 12 + n <= bytes.len(),
        "Invalid bake header length"
    );
    let h: Header = serde_json::from_slice(&bytes[12..12 + n])?;
    ensure!(
        h.version == 1 && h.kernel == kernel_key(),
        "Incompatible bake kernel"
    );
    ensure!(
        RESOLUTIONS.contains(&h.side) && h.samples == 4096 && h.objects > 0 && h.objects <= 64,
        "Invalid bake shape"
    );
    ensure!(
        h.sun.iter().chain(h.energy.iter()).all(|x| x.is_finite())
            && h.energy.iter().all(|x| *x >= 0.),
        "Invalid bake light"
    );
    let payload = &bytes[12 + n..];
    ensure!(
        payload.len() == h.objects as usize * 6 * h.side as usize * h.side as usize * 16,
        "Invalid bake payload length"
    );
    ensure!(digest(payload) == h.payload_sha256, "Corrupt bake payload");
    ensure!(
        payload.chunks_exact(16).all(|v| {
            let x: Vec<f32> = v
                .chunks_exact(4)
                .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
                .collect();
            x.iter().all(|x| x.is_finite() && *x >= 0.) && (x[3] == 0. || x[3] == 1.)
        }),
        "Invalid bake samples"
    );
    Ok((h, payload.to_vec()))
}
fn encode(h: &Header, payload: &[u8]) -> Result<Vec<u8>> {
    let header = serde_json::to_vec(h)?;
    let mut bytes = MAGIC.to_vec();
    bytes.extend((header.len() as u32).to_le_bytes());
    bytes.extend(header);
    bytes.extend(payload);
    Ok(bytes)
}
fn reuse(h: &Header, boxes: &[BoxData], p: &Params) -> Option<[f32; 3]> {
    if h.version != 1
        || h.objects as usize != boxes.len()
        || h.geometry != geometry_key(boxes)
        || h.kernel != kernel_key()
        || h.sun != p.sun[..3]
    {
        return None;
    }
    let mut ratio = [0.; 3];
    for (i, r) in ratio.iter_mut().enumerate() {
        if h.energy[i] == 0. {
            if p.energy[i] != 0. {
                return None;
            }
        } else {
            *r = p.energy[i] / h.energy[i];
        }
        if !r.is_finite() || *r < 0. {
            return None;
        }
    }
    Some(ratio)
}
pub(super) fn pull(r: &Renderer, b: &wgpu::Buffer, size: u64) -> Result<Vec<u8>> {
    let stage = buffer(
        r,
        "bake readback",
        size,
        wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
    );
    let mut e = r.device.create_command_encoder(&Default::default());
    e.copy_buffer_to_buffer(b, 0, &stage, 0, size);
    r.queue.submit([e.finish()]);
    read(r, &stage, size)
}
pub(super) fn boxes_for(engine: &Engine) -> Vec<BoxData> {
    engine
        .scene
        .entities
        .iter()
        .map(|e| {
            let p = e.transform.position;
            let s = e.transform.size;
            BoxData {
                lo: [p[0] - s[0] / 2., p[1] - s[1] / 2., p[2] - s[2] / 2., 0.],
                hi: [p[0] + s[0] / 2., p[1] + s[1] / 2., p[2] + s[2] / 2., 0.],
                color: [e.color[0], e.color[1], e.color[2], 0.],
            }
        })
        .collect()
}
pub(super) fn params(engine: &Engine, mut p: Params) -> Params {
    let f = engine.render_frame();
    p.inv = ignis::picking::view_projection(&f.camera, W, H)
        .inverse()
        .to_cols_array_2d();
    p.eye = Vec3::from(f.camera.position).extend(1.).to_array();
    p.sun = (-Vec3::from(f.light.direction).normalize())
        .extend(0.)
        .to_array();
    p.energy = [
        f.light.direct_intensity * f.light.direct_color[0],
        f.light.direct_intensity * f.light.direct_color[1],
        f.light.direct_intensity * f.light.direct_color[2],
        0.,
    ];
    p
}
#[allow(clippy::too_many_arguments)]
pub(super) fn run(
    r: &mut Renderer,
    engine: Engine,
    boxes: Vec<BoxData>,
    native: wgpu::Texture,
    target: wgpu::Texture,
    p: Params,
    dir: &str,
    validation: bool,
    final_audit: bool,
) -> Result<()> {
    let nv = native.create_view(&Default::default());
    let tv = target.create_view(&Default::default());
    let max_bytes = boxes.len() as u64 * 6 * 64 * 64 * 16;
    let mut builds = vec![];
    if !validation {
        let lab = Lab::new_sized(r, &p, &boxes, &nv, &tv, max_bytes);
        for side in RESOLUTIONS {
            let mut bp = p;
            bp.grid[3] = 0.5;
            bp.counts = [side, 64, 0, 317];
            let t = Instant::now();
            let cost = lab.run(r, &bp, "bake-build")?;
            let wall = ms(t);
            let payload = pull(
                r,
                &lab.probes[1],
                boxes.len() as u64 * 6 * u64::from(side * side) * 16,
            )?;
            let h = Header {
                version: 1,
                geometry: geometry_key(&boxes),
                kernel: kernel_key(),
                sun: p.sun[..3].try_into()?,
                energy: p.energy[..3].try_into()?,
                side,
                samples: 4096,
                objects: boxes.len() as u32,
                payload_sha256: digest(&payload),
            };
            let mut zero_header = h.clone();
            zero_header.energy[0] = 0.;
            let mut zero_params = p;
            zero_params.energy[0] = 0.;
            ensure!(
                reuse(&zero_header, &boxes, &zero_params).is_some(),
                "Zero-to-zero channel rejected"
            );
            zero_params.energy[0] = 1.;
            ensure!(
                reuse(&zero_header, &boxes, &zero_params).is_none(),
                "Unrecoverable zero channel reused"
            );
            zero_params.energy[0] = 0.;
            ensure!(
                reuse(&h, &boxes, &zero_params).unwrap()[0] == 0.,
                "Light-off scaling failed"
            );
            let bytes = encode(&h, &payload)?;
            let t = Instant::now();
            let path = format!("{dir}/atlas-{side}.bin");
            let tmp = format!("{path}.tmp");
            fs::write(&tmp, &bytes)?;
            fs::rename(tmp, &path)?;
            let write_ms = ms(t);
            let (loaded, check) = decode(&fs::read(&path)?)?;
            ensure!(
                check == payload && reuse(&loaded, &boxes, &p) == Some([1.; 3]),
                "Bake round trip changed lighting"
            );
            ensure!(
                decode(&bytes[..bytes.len() - 1]).is_err(),
                "Truncated bake accepted"
            );
            let mut corrupt = bytes.clone();
            let last = corrupt.len() - 1;
            corrupt[last] ^= 1;
            ensure!(decode(&corrupt).is_err(), "Corrupt bake accepted");
            builds.push(json!({"side":side,"samples":4096,"cpu_submit_ms":cost.0,"gpu_ms":cost.1,"wall_ms":wall,"write_ms":write_ms,"payload_bytes":payload.len(),"file_bytes":bytes.len()}));
        }
    }
    let selected = if validation {
        let selection: serde_json::Value =
            serde_json::from_slice(&fs::read("artifacts/lighting-bake/selection.json")?)?;
        let side = selection["side"]
            .as_u64()
            .ok_or_else(|| anyhow::anyhow!("Frozen bake policy missing"))?
            as u32;
        ensure!(RESOLUTIONS.contains(&side), "Invalid frozen resolution");
        Some(side)
    } else {
        None
    };
    let sides: Vec<u32> = selected.map_or_else(|| RESOLUTIONS.to_vec(), |s| vec![s]);
    let labels = if final_audit {
        vec!["reveal-e", "reveal-f"]
    } else if validation {
        vec![
            "camera",
            "scaled-light",
            "changed-sun",
            "moved-door",
            "changed-material",
        ]
    } else {
        vec!["base"]
    };
    let original = engine.scene.clone();
    let mut cases = vec![];
    for label in labels {
        let mut scene = original.clone();
        // Fixed before evaluation: camera-only reveal, channel scaling, then invalidation controls.
        if label != "base" {
            scene.camera.position = [-5.8, 5.2, 10.6];
            scene.camera.target = [0., 0.7, -0.3];
        }
        if label == "reveal-e" {
            scene.camera.position = [-7., 3.6, 8.9];
            scene.camera.target = [-0.2, 0.5, -1.];
            for (x, f) in scene.light.direct_color.iter_mut().zip([0.8, 0.4, 0.9]) {
                *x *= f;
            }
        }
        if label == "reveal-f" {
            scene.camera.position = [8., 3.5, -9.];
            scene.camera.target = [0., 1., 0.];
        }
        if label == "scaled-light" {
            for (x, f) in scene.light.direct_color.iter_mut().zip([0.45, 0.8, 1.1]) {
                *x *= f;
            }
        }
        if label == "changed-sun" {
            scene.light.direction = [0.45, -0.8, -0.25];
        }
        if label == "moved-door" {
            let i = scene.entities.len() - 1;
            scene.entities[i].transform.position[0] += 1.7;
            scene.entities[i].transform.position[2] -= 1.2;
        }
        if label == "changed-material" {
            scene.entities[1].color = [0.8, 0.12, 0.03];
        }
        let current = Engine::new(scene)?;
        let current_boxes = boxes_for(&current);
        let mut cp = params(&current, p);
        let case_dir = format!("{dir}/{label}");
        fs::create_dir_all(&case_dir)?;
        fs::write(
            format!("{case_dir}/scene.json"),
            serde_json::to_vec_pretty(&current.scene)?,
        )?;
        let capacity = u64::from(*sides.iter().max().unwrap());
        let case_bytes = current_boxes.len() as u64 * 6 * capacity * capacity * 16;
        let case_lab = Lab::new_sized(r, &cp, &current_boxes, &nv, &tv, case_bytes);
        let mut frame = current.render_frame();
        frame.suppress_diagnostic_grid = true;
        r.draw(&frame, &nv, W, H)?;
        cp.dims[3] = 0;
        case_lab.run(r, &cp, "full")?;
        fs::write(
            format!("{case_dir}/native.png"),
            r.read_texture_png(&target, W, H)?,
        )?;
        case_lab.run(r, &cp, "primary")?;
        fs::write(
            format!("{case_dir}/receivers.bin"),
            pull(r, &case_lab.receivers, u64::from(W * H) * 48)?,
        )?;
        let mut references = vec![];
        for (mode, seed) in [(7, 91), (8, 193)] {
            cp.dims[3] = mode;
            cp.counts[3] = seed;
            let c = case_lab.run(r, &cp, "shade")?;
            case_lab.save_values(r, &format!("{case_dir}/reference-{mode}.rgba32f"))?;
            if mode == 7 {
                fs::write(
                    format!("{case_dir}/reference.png"),
                    r.read_texture_png(&target, W, H)?,
                )?;
            }
            references.push(json!({"mode":mode,"gpu_ms":c.1}));
        }
        let mut variants = vec![];
        for side in &sides {
            let t = Instant::now();
            let bytes = fs::read(format!("{dir}/atlas-{side}.bin"))?;
            let (h, payload) = decode(&bytes)?;
            let load_ms = ms(t);
            let allowed = reuse(&h, &current_boxes, &cp);
            ensure!(
                allowed.is_some()
                    == matches!(
                        label,
                        "base" | "camera" | "scaled-light" | "reveal-e" | "reveal-f"
                    ),
                "Incorrect bake invalidation"
            );
            let t = Instant::now();
            r.queue.write_buffer(&case_lab.probes[1], 0, &payload);
            wait(r)?;
            let upload_wall_ms = ms(t);
            let methods = if allowed.is_some() {
                vec!["direct64", "compact20", "baked", "hybrid"]
            } else {
                vec!["direct64", "compact20", "stale", "hybrid"]
            };
            let mut rows = vec![];
            let configure = |p: &mut Params, method: &str, seed: u32| -> &str {
                p.counts = [*side, 64, 0, seed];
                p.grid = allowed
                    .unwrap_or([1.; 3])
                    .into_iter()
                    .chain([0.5])
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap();
                p.dims[3] = match method {
                    "direct64" => 2,
                    "compact20" => 21,
                    "hybrid" => {
                        if allowed.is_some() {
                            26
                        } else {
                            25
                        }
                    }
                    _ => 24,
                };
                if method == "direct64" || method == "compact20" {
                    "full"
                } else {
                    "bake-full"
                }
            };
            for seed in 0..4 {
                for method in &methods {
                    let stage = configure(&mut cp, method, [7, 23, 39, 55][seed]);
                    case_lab.run(r, &cp, stage)?;
                    case_lab
                        .save_values(r, &format!("{case_dir}/{side}-{method}-{seed}.rgba32f"))?;
                    if seed == 0 {
                        fs::write(
                            format!("{case_dir}/{side}-{method}.png"),
                            r.read_texture_png(&target, W, H)?,
                        )?;
                    }
                }
            }
            // Rotate order; all methods regenerate receiver maps, including baked queries.
            for trial in 0..64 {
                for offset in 0..methods.len() {
                    let method = methods[(trial + offset) % methods.len()];
                    let stage = configure(&mut cp, method, trial as u32);
                    let t = Instant::now();
                    let mut f = current.render_frame();
                    f.suppress_diagnostic_grid = true;
                    let prep = ms(t);
                    let t = Instant::now();
                    r.draw(&f, &nv, W, H)?;
                    let cpu = ms(t);
                    let raster = r.read_gpu_timing()?.unwrap()["sum_ms"].as_f64().unwrap();
                    let c = case_lab.run(r, &cp, stage)?;
                    rows.push(json!({"trial":trial,"method":method,"cpu_ms":prep+cpu+c.0,"gpu_ms":raster+c.1,"compute_gpu_ms":c.1,"raster_gpu_ms":raster}));
                }
            }
            // Full invalidation must equal direct64, including fresh RNG keyed by receiver.
            if allowed.is_none() {
                for seed in 0..4 {
                    let a = fs::read(format!("{case_dir}/{side}-direct64-{seed}.rgba32f"))?;
                    let b = fs::read(format!("{case_dir}/{side}-hybrid-{seed}.rgba32f"))?;
                    ensure!(
                        a.chunks_exact(16)
                            .zip(b.chunks_exact(16))
                            .all(|(a, b)| a[..12] == b[..12]),
                        "Invalid bake did not fall back exactly"
                    );
                }
            }
            variants.push(json!({"side":side,"reused":allowed.is_some(),"scale":allowed,"load_ms":load_ms,"upload_wall_ms":upload_wall_ms,"rows":rows}));
        }
        cases.push(json!({"label":label,"references":references,"variants":variants}));
        println!("Bake case {label} complete");
    }
    fs::write(
        format!(
            "{dir}/{}.json",
            if final_audit {
                "final"
            } else if validation {
                "validation"
            } else {
                "calibration"
            }
        ),
        serde_json::to_vec_pretty(
            &json!({"width":W,"height":H,"adapter":r.adapter.get_info().name,"backend":format!("{:?}",r.adapter.get_info().backend),"kernel":kernel_key(),"builds":builds,"cases":cases,"round_trip":true,"corrupt_rejected":true,"truncated_rejected":true,"native_profile_identity":true,"full_invalidation_exact":true}),
        )?,
    )?;
    Ok(())
}
