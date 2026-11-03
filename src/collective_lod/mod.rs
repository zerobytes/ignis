//! Experimental collective geometry hierarchy packets. OFF unless explicitly installed
//! through the renderer's diagnostic API; no Scene, physics or replay mutation.
mod gpu;
mod packet;
pub(crate) use gpu::Gpu;
pub use packet::{Pack, compile, world};
#[cfg(test)]
mod tests;
