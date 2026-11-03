//! Live editor presentation cadence, separate from simulation and GPU timestamps.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, time::Instant};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub enabled: bool,
    pub vsync: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: false,
            vsync: true,
        }
    }
}
#[derive(Default, Serialize)]
pub struct Sample {
    pub fps: Option<f64>,
    pub frame_ms: Option<f64>,
    pub redraw_wall_ms: Option<f64>,
    pub intervals: usize,
    pub seconds: f64,
    pub viewport: [u32; 2],
    pub present_mode: String,
    pub optimized: bool,
    pub renderer: serde_json::Value,
}
pub struct State {
    pub settings: Settings,
    pub sample: Sample,
    last: Option<Instant>,
    published: Option<Instant>,
    intervals: VecDeque<(Instant, f64, f64)>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            sample: Sample::default(),
            last: None,
            published: None,
            intervals: VecDeque::with_capacity(512),
        }
    }
}
impl State {
    pub fn configure(&mut self, settings: Settings) {
        self.settings = settings;
        self.pause();
    }
    pub fn pause(&mut self) {
        self.last = None;
        self.published = None;
        self.intervals.clear();
        self.sample = Sample::default();
    }
    pub fn presented(
        &mut self,
        now: Instant,
        cpu_ms: f64,
        viewport: [u32; 2],
        mode: &str,
        optimized: bool,
    ) -> bool {
        if !self.settings.enabled {
            return false;
        }
        if self.sample.viewport != viewport
            || self.sample.present_mode != mode
            || self.sample.optimized != optimized
        {
            self.pause();
            self.sample.viewport = viewport;
            self.sample.present_mode = mode.into();
            self.sample.optimized = optimized;
        }
        if let Some(last) = self.last.replace(now) {
            if self.intervals.len() == 512 {
                self.intervals.pop_front();
            }
            self.intervals
                .push_back((now, now.duration_since(last).as_secs_f64(), cpu_ms));
        }
        while self
            .intervals
            .front()
            .is_some_and(|s| now.duration_since(s.0).as_secs_f64() > 1.)
        {
            self.intervals.pop_front();
        }
        if self
            .published
            .is_some_and(|p| now.duration_since(p).as_secs_f64() < 0.5)
        {
            return false;
        }
        self.published = Some(now);
        let seconds: f64 = self.intervals.iter().map(|s| s.1).sum();
        let n = self.intervals.len();
        self.sample.intervals = n;
        self.sample.seconds = seconds;
        self.sample.fps = (n > 0 && seconds > 0.).then_some(n as f64 / seconds);
        self.sample.frame_ms = (n > 0).then_some(seconds * 1000. / n as f64);
        self.sample.redraw_wall_ms =
            (n > 0).then_some(self.intervals.iter().map(|s| s.2).sum::<f64>() / n as f64);
        true
    }
    pub fn inspect(&self) -> serde_json::Value {
        serde_json::json!({"settings":self.settings,"sample":self.sample,
            "age_ms":self.published.map(|p|p.elapsed().as_secs_f64()*1000.),
            "method":"Native window present-call cadence over up to1s/512intervals; includes simulation/UI/acquisition/submission/scheduling when applicable. Not GPU timestamps or scanout."})
    }
}
