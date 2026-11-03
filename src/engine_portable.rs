//! Portable save boundary, kept out of the hot simulation loop.
use super::*;
impl Engine {
    pub(crate) fn capture_portable_save(&self) -> Result<crate::portable_save::PortableSave> {
        if self.planet_active() {
            return Err(invalid(
                "Planet runtime primaries are not part of PSV1; refusing an incomplete save",
            ));
        }
        if self.playing
            || self.consumed_ability_inputs != self.ability_inputs.len()
            || self.consumed_population_inputs != self.population_inputs.len()
            || self.consumed_noises != self.noises.len()
        {
            return Err(invalid(
                "Portable save: pause and advance pending gameplay commands",
            ));
        }
        let mut manual = VecDeque::with_capacity(RECENT_MANUAL_NOISES);
        let mut dropped = self.dropped_manual_noises;
        for n in self.recent_manual_noises.iter().chain(&self.noises) {
            if self.simulation.tick.saturating_sub(n.tick) < 90 {
                if manual.len() == RECENT_MANUAL_NOISES {
                    manual.pop_front();
                    dropped = dropped.saturating_add(1);
                }
                manual.push_back(*n);
            }
        }
        Ok(crate::portable_save::PortableSave {
            version: crate::portable_save::VERSION,
            scene_version: SCENE_VERSION,
            spatial_version: crate::world_space::VERSION,
            origin: crate::portable_save::origin(),
            scene: self.scene.clone(),
            mode: self.execution_mode,
            sequences: [
                self.admission.noise,
                self.admission.ability,
                self.admission.population,
            ],
            manual,
            dropped_manual: dropped,
            state: self.simulation.portable_checkpoint()?,
        })
    }
    pub fn from_portable_save(bytes: &[u8], mode: Option<ExecutionMode>) -> Result<Self> {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<Self> {
            let saved = crate::portable_save::PortableSave::decode(bytes)?;
            let mut candidate = Self::new(saved.scene.clone())?;
            candidate.simulation = saved.state.restore(&candidate.scene)?;
            candidate.controls =
                crate::controls::Runtime::new(candidate.simulation.actors.actors.len());
            candidate.admission = Admission::new(candidate.simulation.actors.actors.len());
            candidate.baseline_tick = candidate.simulation.tick;
            candidate.baseline_sequences = saved.sequences;
            candidate.admission.noise = saved.sequences[0];
            candidate.admission.ability = saved.sequences[1];
            candidate.admission.population = saved.sequences[2];
            candidate.recent_manual_noises = saved.manual;
            candidate.dropped_manual_noises = saved.dropped_manual;
            candidate.execution_mode = mode.unwrap_or(saved.mode);
            candidate.input = Input::default();
            candidate.playing = false;
            // A local native baseline serves recorded replay only. Continuous
            // restoration never serializes native caches and retains no history.
            if candidate.execution_mode == ExecutionMode::Recorded {
                candidate.baseline = Some(std::sync::Arc::from(
                    candidate.capture_checkpoint()?.encode()?,
                ));
            }
            Ok(candidate)
        }))
        .map_err(|_| invalid("Portable save restore: invalid runtime structure"))?
    }
    pub fn load_portable_save(&mut self, bytes: &[u8], mode: Option<ExecutionMode>) -> Result<()> {
        if self.planet_active() {
            return Err(invalid(
                "Disable planet runtime before loading a portable save",
            ));
        }
        if self.playing {
            return Err(invalid("Pause before loading portable save"));
        }
        let mut candidate = Self::from_portable_save(bytes, mode)?;
        candidate.debug_view = self.debug_view;
        candidate.revision = self
            .revision
            .checked_add(1)
            .ok_or_else(|| invalid("Revision exhausted"))?;
        candidate.input_revision = self
            .input_revision
            .checked_add(1)
            .ok_or_else(|| invalid("Input revision exhausted"))?;
        candidate.view_revision = self
            .view_revision
            .checked_add(1)
            .ok_or_else(|| invalid("View revision exhausted"))?;
        *self = candidate;
        Ok(())
    }
}
