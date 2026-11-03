//! Opt-in device consumer of the same 24kHz PCM as WAV capture. No world access
//! on the driver thread. A bounded mailbox drops excess simulated time explicitly.
use crate::{
    model::{Result, invalid},
    npc_presentation::RATE,
};
use serde::Serialize;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
    mpsc::SyncSender,
};

#[derive(Debug, Clone, Serialize)]
pub struct Info {
    pub device: String,
    pub rate: u32,
    pub channels: u16,
    pub sample_format: String,
}
#[derive(Default)]
pub struct Counters {
    pub dropped: AtomicU64,
    pub underruns: AtomicU64,
    pub errors: AtomicU64,
}
// The reader is intentionally absent in builds without the device backend.
#[cfg_attr(not(feature = "device-audio"), expect(dead_code))]
pub(crate) struct Packet {
    pub epoch: u64,
    pub samples: Vec<i16>,
}
pub struct Output {
    pub info: Info,
    pub listener: [f32; 3],
    pub radius: f32,
    queue: SyncSender<Packet>,
    epoch: Arc<AtomicU64>,
    pub counters: Arc<Counters>,
    stop: SyncSender<()>,
    worker: Option<std::thread::JoinHandle<()>>,
}
impl Output {
    pub fn start(listener: [f32; 3], radius: f32) -> Result<Self> {
        crate::npc_presentation::pcm(&Default::default(), &[], 0, 1, listener, radius)?;
        backend::start(listener, radius)
    }
    pub fn invalidate(&self) {
        self.epoch.fetch_add(1, Ordering::Release);
    }
    pub fn submit(&self, samples: Vec<i16>) {
        if samples.len() != 400 {
            self.counters.dropped.fetch_add(1, Ordering::Relaxed);
            return;
        }
        let packet = Packet {
            epoch: self.epoch.load(Ordering::Acquire),
            samples,
        };
        if self.queue.try_send(packet).is_err() {
            self.counters.dropped.fetch_add(1, Ordering::Relaxed);
        }
    }
    pub fn inspect(&self) -> serde_json::Value {
        serde_json::json!({"enabled":true,"device":self.info,"source_rate":RATE,"queue_ticks":8,"listener":self.listener,"radius":self.radius,"dropped_ticks":self.counters.dropped.load(Ordering::Relaxed),"underruns":self.counters.underruns.load(Ordering::Relaxed),"errors":self.counters.errors.load(Ordering::Relaxed)})
    }
}
impl Drop for Output {
    fn drop(&mut self) {
        self.invalidate();
        let _ = self.stop.try_send(());
        if let Some(t) = self.worker.take() {
            let _ = t.join();
        }
    }
}
#[cfg(not(feature = "device-audio"))]
mod backend {
    use super::*;
    pub fn start(_: [f32; 3], _: f32) -> Result<Output> {
        Err(invalid(
            "Build with device-audio to enable output; headless PCM capture remains available",
        ))
    }
}
#[cfg(feature = "device-audio")]
mod backend {
    use super::*;
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use std::sync::mpsc;
    pub fn start(listener: [f32; 3], radius: f32) -> Result<Output> {
        let (queue, rx) = mpsc::sync_channel(8);
        let (stop, quit) = mpsc::sync_channel(1);
        let (ready, started) = mpsc::sync_channel(1);
        let epoch = Arc::new(AtomicU64::new(1));
        let generation = epoch.clone();
        let counters = Arc::new(Counters::default());
        let stats = counters.clone();
        // CPAL stream stays on its owner thread, including platforms where it is !Send.
        let worker = std::thread::Builder::new()
            .name("ignis-audio".into())
            .spawn(move || {
                let result = (|| -> std::result::Result<_, String> {
                    let device = cpal::default_host()
                        .default_output_device()
                        .ok_or("No audio output device")?;
                    let config = device.default_output_config().map_err(|e| e.to_string())?;
                    let info = Info {
                        device: device.name().map_err(|e| e.to_string())?,
                        rate: config.sample_rate().0,
                        channels: config.channels(),
                        sample_format: config.sample_format().to_string(),
                    };
                    if !(8000..=192000).contains(&info.rate) || !(1..=16).contains(&info.channels) {
                        return Err("Unsupported audio rate/channels".into());
                    }
                    let format = config.sample_format();
                    let config: cpal::StreamConfig = config.into();
                    let reader = Reader {
                        rx,
                        epoch: generation,
                        counters: stats.clone(),
                        packet: None,
                        index: 0,
                        phase: 0,
                        rate: info.rate,
                        channels: info.channels as usize,
                    };
                    let stream = match format {
                        cpal::SampleFormat::F32 => build::<f32>(&device, &config, reader),
                        cpal::SampleFormat::I16 => build::<i16>(&device, &config, reader),
                        cpal::SampleFormat::U16 => build::<u16>(&device, &config, reader),
                        _ => return Err("Audio device needs f32/i16/u16 output".into()),
                    }?;
                    stream.play().map_err(|e| e.to_string())?;
                    Ok((stream, info))
                })();
                match result {
                    Ok((stream, info)) => {
                        if ready.send(Ok(info)).is_ok() {
                            let _ = quit.recv();
                        }
                        drop(stream);
                    }
                    Err(e) => {
                        let _ = ready.send(Err(e));
                    }
                }
            })
            .map_err(|e| invalid(e.to_string()))?;
        match started.recv_timeout(std::time::Duration::from_secs(3)) {
            Ok(Ok(info)) => Ok(Output {
                info,
                listener,
                radius,
                queue,
                epoch,
                counters,
                stop,
                worker: Some(worker),
            }),
            Ok(Err(e)) => {
                let _ = worker.join();
                Err(invalid(e))
            }
            Err(_) => {
                let _ = stop.try_send(());
                Err(invalid("Audio output initialization timed out"))
            }
        }
    }
    struct Reader {
        rx: mpsc::Receiver<Packet>,
        epoch: Arc<AtomicU64>,
        counters: Arc<Counters>,
        packet: Option<Packet>,
        index: usize,
        phase: u32,
        rate: u32,
        channels: usize,
    }
    impl Reader {
        fn sample(&mut self) -> f32 {
            let epoch = self.epoch.load(Ordering::Acquire);
            if self
                .packet
                .as_ref()
                .is_some_and(|p| p.epoch != epoch || self.index >= p.samples.len())
            {
                self.packet = None;
            }
            if self.packet.is_none() {
                // At most8 queued packets. Never block or allocate in callback.
                for _ in 0..8 {
                    match self.rx.try_recv() {
                        Ok(p) if p.epoch == epoch => {
                            self.packet = Some(p);
                            self.index = 0;
                            break;
                        }
                        Ok(_) => {}
                        Err(_) => break,
                    }
                }
            }
            self.packet
                .as_ref()
                .map_or(0., |p| f32::from(p.samples[self.index]) / 32768.)
        }
        fn fill<T: cpal::Sample + cpal::FromSample<f32>>(&mut self, data: &mut [T]) {
            let mut empty = false;
            for frame in data.chunks_mut(self.channels) {
                let value = self.sample();
                empty |= self.packet.is_none();
                for output in frame {
                    *output = T::from_sample(value);
                }
                self.phase += RATE;
                while self.phase >= self.rate {
                    self.phase -= self.rate;
                    self.index += 1;
                    if self
                        .packet
                        .as_ref()
                        .is_some_and(|p| self.index >= p.samples.len())
                    {
                        self.packet = None;
                        self.index = 0;
                        let _ = self.sample();
                    }
                }
            }
            if empty {
                self.counters.underruns.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
    fn build<T: cpal::SizedSample + cpal::FromSample<f32>>(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        mut reader: Reader,
    ) -> std::result::Result<cpal::Stream, String> {
        let errors = reader.counters.clone();
        device
            .build_output_stream(
                config,
                move |data: &mut [T], _| reader.fill(data),
                move |_| {
                    errors.errors.fetch_add(1, Ordering::Relaxed);
                },
                None,
            )
            .map_err(|e| e.to_string())
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        // Critical lifecycle/integration: stale PCM cannot cross a world epoch,
        // and the callback's real rate/channel adapter consumes each source frame once.
        #[test]
        fn callback_consumes_pcm_and_invalidates_world_without_device() {
            let (tx, rx) = mpsc::sync_channel(8);
            let epoch = Arc::new(AtomicU64::new(1));
            let stats = Arc::new(Counters::default());
            let mut reader = Reader {
                rx,
                epoch: epoch.clone(),
                counters: stats.clone(),
                packet: None,
                index: 0,
                phase: 0,
                rate: 48000,
                channels: 2,
            };
            tx.send(Packet {
                epoch: 1,
                samples: (0..400).map(|x| x * 50).collect(),
            })
            .unwrap();
            let mut out = vec![0f32; 1600];
            reader.fill(&mut out);
            for i in 0..400 {
                for j in 0..4 {
                    assert_eq!(out[i * 4 + j], f32::from(i as i16 * 50) / 32768.);
                }
            }
            tx.send(Packet {
                epoch: 1,
                samples: vec![10000; 400],
            })
            .unwrap();
            epoch.store(2, Ordering::Release);
            let mut out = [1f32; 16];
            reader.fill(&mut out);
            assert_eq!(out, [0.; 16]);
            tx.send(Packet {
                epoch: 2,
                samples: vec![-10000; 400],
            })
            .unwrap();
            reader.fill(&mut out);
            assert!(out.iter().all(|x| *x == -10000. / 32768.));
            epoch.store(3, Ordering::Release);
            reader.fill(&mut out);
            assert_eq!(out, [0.; 16]);
            assert!(stats.underruns.load(Ordering::Relaxed) > 0);
        }
    }
}
