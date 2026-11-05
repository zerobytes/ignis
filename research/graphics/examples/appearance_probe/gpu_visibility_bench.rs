use super::*;
impl Renderer {
    pub fn bench(
        &mut self,
        world: &World,
        scenario: usize,
        frame: usize,
        size: [u32; 2],
    ) -> Result<serde_json::Value> {
        let mut pose = Pose::new();
        world.pose(scenario, step(frame) as f32 / 12., size, &mut pose);
        self.target(size);
        for _ in 0..20 {
            for mode in 0..6 {
                self.update(&pose, mode);
                let mut e = self.gpu.device.create_command_encoder(&Default::default());
                self.encode(&mut e, &pose, mode, None);
                self.submit(e)?;
            }
        }
        let n = 12;
        let rounds = 12;
        let mut serial = vec![Vec::new(); n];
        let mut gpu = serial.clone();
        let mut pyramid = serial.clone();
        let mut cull = serial.clone();
        let mut cpu = serial.clone();
        for r in 0..rounds {
            for offset in 0..n {
                let slot = (r + offset) % n;
                let mode = (slot % 6) as u32;
                let start = Instant::now();
                world.pose(scenario, step(frame) as f32 / 12., size, &mut pose);
                self.update(&pose, mode);
                let mut e = self.gpu.device.create_command_encoder(&Default::default());
                self.encode(&mut e, &pose, mode, Some(((r * n + slot) * 6) as u32));
                cpu[slot].push(start.elapsed().as_secs_f64() * 1000.);
                self.submit(e)?;
                serial[slot].push(start.elapsed().as_secs_f64() * 1000.);
            }
        }
        if let Some(q) = &self.gpu.query {
            let mut e = self.gpu.device.create_command_encoder(&Default::default());
            e.resolve_query_set(q, 0..(rounds * n * 6) as u32, &self.gpu.query_resolve, 0);
            e.copy_buffer_to_buffer(
                &self.gpu.query_resolve,
                0,
                &self.gpu.query_readback,
                0,
                8192,
            );
            self.submit(e)?;
            self.gpu.map(&self.gpu.query_readback)?;
            let bytes = self.gpu.query_readback.slice(..).get_mapped_range();
            let stamps = bytemuck::cast_slice::<u8, u64>(&bytes);
            let scale = f64::from(self.gpu.timestamp_period_ns.unwrap()) / 1e6;
            for r in 0..rounds {
                for s in 0..n {
                    let base = (r * n + s) * 6;
                    let interval = |i: usize| -> Result<f64> {
                        ensure!(stamps[base + i + 1] >= stamps[base + i], "GPU clock order");
                        Ok((stamps[base + i + 1] - stamps[base + i]) as f64 * scale)
                    };
                    gpu[s].push(interval(0)?);
                    if matches!(s % 6, 2 | 3) {
                        pyramid[s].push(interval(2)?);
                    }
                    if s % 6 > 0 {
                        if s % 6 >= 4 {
                            ensure!(stamps[base + 5] >= stamps[base], "cull clock order");
                            cull[s].push((stamps[base + 5] - stamps[base]) as f64 * scale);
                        } else {
                            cull[s].push(interval(4)?);
                        }
                    }
                }
            }
            drop(bytes);
            self.gpu.query_readback.unmap();
        }
        let mut throughput = vec![Vec::new(); n];
        for r in 0..12 {
            for offset in 0..n {
                let slot = (r + offset) % n;
                let mode = (slot % 6) as u32;
                let start = Instant::now();
                let mut last = None;
                for k in 0..8 {
                    world.pose(
                        scenario,
                        step((frame + k) % 26) as f32 / 12.,
                        size,
                        &mut pose,
                    );
                    self.update(&pose, mode);
                    let mut e = self.gpu.device.create_command_encoder(&Default::default());
                    self.encode(&mut e, &pose, mode, None);
                    last = Some(self.gpu.queue.submit([e.finish()]));
                }
                self.gpu.device.poll(wgpu::PollType::Wait {
                    submission_index: last,
                    timeout: Some(Duration::from_secs(60)),
                })?;
                throughput[slot].push(start.elapsed().as_secs_f64() * 1000. / 8.);
            }
        }
        Ok(json!((0..n).map(|s|json!({"mode":MODES[s%6],"gpu_ms":stats(&mut gpu[s]),"pyramid_ms":if matches!(s%6,2|3){Some(stats(&mut pyramid[s]))}else{None},"cull_ms":if s%6>0{Some(stats(&mut cull[s]))}else{None},"cpu_prepare_encode_ms":stats(&mut cpu[s]),"serial_ms":stats(&mut serial[s]),"moving_throughput_ms":stats(&mut throughput[s])})).collect::<Vec<_>>()))
    }
}
