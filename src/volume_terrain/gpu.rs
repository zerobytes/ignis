//! Cold GPU density generation; one explicit readback feeds indexed CPU extraction.
//! This is not the chapter's fully GPU-resident polygon pool or world streamer.
use super::*;
use wgpu::util::DeviceExt;
pub struct Gpu {
    pipeline: wgpu::ComputePipeline,
}
impl Gpu {
    pub fn new(device: &wgpu::Device) -> Self {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("volumetric density"),
            source: wgpu::ShaderSource::Wgsl(include_str!("density.wgsl").into()),
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("volumetric density"),
            layout: None,
            module: &module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        Self { pipeline }
    }
    pub fn compile(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        doc: &Document,
        grid: &Grid,
    ) -> Result<Compiled> {
        let start = std::time::Instant::now();
        let (n, values) = self.sample(device, queue, doc, grid)?;
        let density_readback_ms = start.elapsed().as_secs_f64() * 1000.;
        let extraction_start = std::time::Instant::now();
        let mut c =
            super::extract::extract_values(doc, grid, n, values, "gpu_density_cpu_extraction")?;
        c.report["timing_wall_ms"] = serde_json::json!({"density_and_readback":density_readback_ms,"cpu_extraction":extraction_start.elapsed().as_secs_f64()*1000.});
        c.report["gpu_readback_bytes"] = serde_json::json!(n.iter().product::<usize>() * 4);
        c.report["gpu_resident_extraction"] = serde_json::json!(false);
        Ok(c)
    }
    pub fn sample(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        doc: &Document,
        grid: &Grid,
    ) -> Result<([usize; 3], Vec<f64>)> {
        let s = Sampler::new(doc)?;
        let (n, count) = grid.dimensions()?;
        // A local f32 backend must not silently round away its sampling lattice.
        // Reference f64 remains available for larger authoring coordinates.
        let largest = (0..3)
            .map(|i| grid.origin[i].abs() + f64::from(grid.cells[i]) * grid.spacing)
            .fold(0., f64::max);
        let base_scale = match doc.base {
            Base::Plane { height } => height.abs(),
            Base::Sphere { center, radius } => {
                center.iter().map(|v| v.abs()).fold(radius, f64::max)
            }
        };
        if largest.max(base_scale) > grid.spacing * 8192. {
            return Err(invalid(
                "GPU density requires local coordinates/base within8192grid steps; use CPU reference for larger coordinates",
            ));
        }
        let mut p = vec![[0f32; 4]; 9];
        p[0] = [
            grid.origin[0] as f32,
            grid.origin[1] as f32,
            grid.origin[2] as f32,
            grid.spacing as f32,
        ];
        p[1] = [
            n[0] as f32,
            n[1] as f32,
            n[2] as f32,
            f32::from_bits(doc.seed),
        ];
        match doc.base {
            Base::Plane { height } => {
                p[3][1] = height as f32;
            }
            Base::Sphere { center, radius } => {
                p[2] = [
                    center[0] as f32,
                    center[1] as f32,
                    center[2] as f32,
                    radius as f32,
                ];
                p[3][0] = 1.;
            }
        }
        p[3][2] = doc.vertical_strength as f32;
        p[3][3] = s.octaves.len() as f32;
        if let Some(w) = &doc.warp {
            p[4] = [w.frequency as f32, w.amplitude as f32, 1., 0.];
        }
        if let Some(t) = &doc.terraces {
            p[5] = [t.spacing as f32, t.strength as f32, t.overhang as f32, 0.];
            p[6] = [t.sharpness as f32, t.warp as f32, t.phase as f32, 1.];
        }
        if let Some(v) = &doc.shelf {
            p[7] = [
                v.altitude as f32,
                v.half_thickness as f32,
                v.strength as f32,
                v.blend as f32,
            ];
        }
        if let Some(v) = &doc.floor {
            p[8] = [
                v.altitude as f32,
                v.transition as f32,
                v.strength as f32,
                1.,
            ];
        }
        for o in &s.octaves {
            p.push([
                o.frequency as f32,
                o.amplitude as f32,
                f32::from_bits(o.seed),
                0.,
            ]);
            for column in o.rotation.to_cols_array_2d() {
                p.push([column[0] as f32, column[1] as f32, column[2] as f32, 0.]);
            }
        }
        let params = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("density recipe"),
            contents: bytemuck::cast_slice(&p),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let bytes = count as u64 * 4;
        if bytes > u64::from(device.limits().max_storage_buffer_binding_size)
            || ((count as u32).div_ceil(64)) > device.limits().max_compute_workgroups_per_dimension
        {
            return Err(invalid(
                "Density dispatch exceeds GPU storage or workgroup capabilities",
            ));
        }
        let output = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("density lattice"),
            size: bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let read = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("density readback"),
            size: bytes,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("density generation"),
            layout: &self.pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: output.as_entire_binding(),
                },
            ],
        });
        let mut encoder = device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("density sampling"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &group, &[]);
            pass.dispatch_workgroups((count as u32).div_ceil(64), 1, 1);
        }
        encoder.copy_buffer_to_buffer(&output, 0, &read, 0, bytes);
        queue.submit([encoder.finish()]);
        let slice = read.slice(..);
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(std::time::Duration::from_secs(30)),
            })
            .map_err(|e| invalid(e.to_string()))?;
        rx.recv_timeout(std::time::Duration::from_secs(30))
            .map_err(|e| invalid(e.to_string()))?
            .map_err(|e| invalid(e.to_string()))?;
        let mapped = slice.get_mapped_range();
        let values = mapped
            .chunks_exact(4)
            .map(|b| f64::from(f32::from_le_bytes(b.try_into().unwrap())))
            .collect();
        drop(mapped);
        read.unmap();
        Ok((n, values))
    }
}
