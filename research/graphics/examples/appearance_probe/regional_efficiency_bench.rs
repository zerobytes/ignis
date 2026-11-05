//! All variants update their own instance buffer before submission; GPU and serial
//! intervals are from the same operation, with duplicated/rotated controls.
use super::super::efficiency::{MODES, prepare};
use super::*;
impl Renderer {
    pub fn bench_efficiency(
        &mut self,
        cam: &Camera,
        scene: &Scene,
        states: &[Vec<Vec<bool>>],
        mask: &[bool],
    ) -> Result<serde_json::Value> {
        self.target(cam.size);
        let mut state = states.to_vec();
        let mut plan = Plan::new(scene.objects.len() * scene.regions.len() + 1);
        // Warm up each representation, then alternate variants and duplicate controls.
        for _ in 0..20 {
            for (m, s) in state.iter_mut().enumerate() {
                prepare(scene, cam, s, &mut plan, m, mask);
                self.update(cam, &plan, 0, [0.; 2]);
                let mut e = self.gpu.device.create_command_encoder(&Default::default());
                self.draw(&mut e, &plan, None);
                self.submit(e)?;
            }
        }
        let n = MODES.len() * 2;
        let rounds = 24;
        let mut serial = vec![Vec::new(); n];
        let mut cpu = serial.clone();
        let mut upload = serial.clone();
        let mut encode = serial.clone();
        let mut gpu = serial.clone();
        for r in 0..rounds {
            for offset in 0..n {
                let slot = (offset + r) % n;
                let mode = slot % MODES.len();
                let start = Instant::now();
                prepare(scene, cam, &mut state[mode], &mut plan, mode, mask);
                cpu[slot].push(start.elapsed().as_secs_f64() * 1e6);
                let u = Instant::now();
                self.update(cam, &plan, 0, [0.; 2]);
                upload[slot].push(u.elapsed().as_secs_f64() * 1000.);
                let c = Instant::now();
                let mut e = self.gpu.device.create_command_encoder(&Default::default());
                self.draw(&mut e, &plan, Some(((r * n + slot) * 2) as u32));
                encode[slot].push(c.elapsed().as_secs_f64() * 1000.);
                self.submit(e)?;
                serial[slot].push(start.elapsed().as_secs_f64() * 1000.);
            }
        }
        if let Some(q) = &self.gpu.query {
            let mut e = self.gpu.device.create_command_encoder(&Default::default());
            e.resolve_query_set(q, 0..(rounds * n * 2) as u32, &self.gpu.query_resolve, 0);
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
                for (slot, g) in gpu.iter_mut().enumerate() {
                    let i = (r * n + slot) * 2;
                    ensure!(stamps[i + 1] >= stamps[i], "timestamp order");
                    g.push((stamps[i + 1] - stamps[i]) as f64 * scale);
                }
            }
            drop(bytes);
            self.gpu.query_readback.unmap();
        }
        // Eight queued frames allow CPU/GPU overlap. Same pose: steady throughput,
        // not input latency, moving-camera FPS, or the serial interval above.
        let mut throughput = vec![Vec::new(); n];
        for round in 0..12 {
            for offset in 0..n {
                let slot = (round + offset) % n;
                let mode = slot % MODES.len();
                let start = Instant::now();
                let mut last = None;
                for _ in 0..8 {
                    prepare(scene, cam, &mut state[mode], &mut plan, mode, mask);
                    self.update(cam, &plan, 0, [0.; 2]);
                    let mut e = self.gpu.device.create_command_encoder(&Default::default());
                    self.draw(&mut e, &plan, None);
                    last = Some(self.gpu.queue.submit([e.finish()]));
                }
                self.gpu.device.poll(wgpu::PollType::Wait {
                    submission_index: last,
                    timeout: Some(Duration::from_secs(60)),
                })?;
                throughput[slot].push(start.elapsed().as_secs_f64() * 1000. / 8.);
            }
        }
        Ok(json!((0..n).map(|i|json!({"mode":MODES[i%MODES.len()],"throughput_ms":stats(&mut throughput[i]),"serial_ms":stats(&mut serial[i]),"gpu_ms":stats(&mut gpu[i]),"planning_us":stats(&mut cpu[i]),"upload_ms":stats(&mut upload[i]),"encode_ms":stats(&mut encode[i])})).collect::<Vec<_>>()))
    }
}
