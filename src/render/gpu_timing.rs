//! Opt-in pass-boundary measurements for headless diagnostics.
use anyhow::Result;

pub(super) struct Timing {
    queries: wgpu::QuerySet,
    resolve: wgpu::Buffer,
    readback: wgpu::Buffer,
    labels: Vec<&'static str>,
}
impl Timing {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            queries: device.create_query_set(&wgpu::QuerySetDescriptor {
                label: Some("diagnostic render timestamps"),
                ty: wgpu::QueryType::Timestamp,
                count: 8,
            }),
            resolve: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("render timestamp resolve"),
                size: 256,
                usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            }),
            readback: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("render timestamp readback"),
                size: 256,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            labels: Vec::with_capacity(4),
        }
    }
    pub fn reset(&mut self) {
        self.labels.clear();
    }
    pub fn pass(&mut self, label: &'static str) -> wgpu::RenderPassTimestampWrites<'_> {
        let index = (self.labels.len() * 2) as u32;
        assert!(index < 8);
        self.labels.push(label);
        wgpu::RenderPassTimestampWrites {
            query_set: &self.queries,
            beginning_of_pass_write_index: Some(index),
            end_of_pass_write_index: Some(index + 1),
        }
    }
    pub fn resolve(&self, encoder: &mut wgpu::CommandEncoder) {
        let count = (self.labels.len() * 2) as u32;
        if count > 0 {
            encoder.resolve_query_set(&self.queries, 0..count, &self.resolve, 0);
            encoder.copy_buffer_to_buffer(&self.resolve, 0, &self.readback, 0, count as u64 * 8);
        }
    }
    pub fn read(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Result<serde_json::Value> {
        if self.labels.is_empty() {
            return Ok(serde_json::json!({"passes":[],"sum_ms":0.}));
        }
        // Fence pass completion before resolving Metal stage-boundary counters.
        // Resolving in the same submission can observe a preceding sample.
        device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(std::time::Duration::from_secs(30)),
        })?;
        let mut encoder = device.create_command_encoder(&Default::default());
        self.resolve(&mut encoder);
        queue.submit([encoder.finish()]);
        let bytes = (self.labels.len() * 16) as u64;
        let slice = self.readback.slice(0..bytes);
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(std::time::Duration::from_secs(30)),
        })?;
        rx.recv_timeout(std::time::Duration::from_secs(30))??;
        let mapped = slice.get_mapped_range();
        let values: Vec<u64> = mapped
            .chunks_exact(8)
            .map(|b| u64::from_le_bytes(b.try_into().unwrap()))
            .collect();
        let period = queue.get_timestamp_period() as f64;
        let mut sum = 0.;
        let passes: Vec<_> = self
            .labels
            .iter()
            .enumerate()
            .map(|(i, label)| {
                let ms = values[2 * i + 1].saturating_sub(values[2 * i]) as f64 * period / 1e6;
                sum += ms;
                serde_json::json!({"pass":label,"ms":ms})
            })
            .collect();
        drop(mapped);
        self.readback.unmap();
        Ok(serde_json::json!({"passes":passes,"sum_ms":sum,"timestamp_period_ns":period}))
    }
}
