use super::*;
use crate::wind::Operation as Op;
impl Engine {
    pub(crate) fn wind_command(&mut self, operation: &Op) -> Result<Value> {
        match operation {
            Op::Inspect => Ok(self.simulation.inspect_wind()),
            Op::Sample {
                positions,
                time_seconds,
            } => {
                if positions.len() > 4096
                    || !crate::wind::bounded(*time_seconds, 0., MAX_RUNTIME_TICK as f64 / 60.)
                    || positions
                        .iter()
                        .any(|p| !crate::wind::vec(*p, crate::wind::MAX_COORDINATE))
                {
                    return Err(invalid(
                        "Wind sampling requires finite local positions/time and at most4096 samples",
                    ));
                }
                let field = self.simulation.wind.field.at_time(*time_seconds);
                let values: Vec<_> = positions
                    .iter()
                    .map(|p| field.sample(glam::DVec3::from_array(*p)).to_array())
                    .collect();
                Ok(json!({"positions":positions,"time_seconds":time_seconds,"velocity_mps":values}))
            }
            Op::Replace { document } => {
                if self.playing || self.planet_active() {
                    return Err(invalid("Pause the ordinary scene before authoring wind"));
                }
                self.apply_edits(std::iter::once(SceneEdit::SetWind {
                    document: document.clone(),
                }))?;
                Ok(self.simulation.inspect_wind())
            }
            Op::Open { .. } | Op::Save { .. } | Op::Load { .. } => Err(invalid(
                "Wind panel/file operations require the editor service",
            )),
        }
    }
}
