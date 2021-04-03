mod checkboard_pattern;
mod lightning;
mod ray_trace;
mod sky;
//mod squeeze;

pub use checkboard_pattern::CheckBoardCommandFactory;
pub use lightning::LightningCommandFactory;
pub use ray_trace::RayTraceCommandFactory;
//pub use squeeze::SqueezeCommandFactory;
pub use sky::SkyCommandFactory;

pub mod shaders {
    pub use super::{lightning::lightning_cs as lightning_shader, ray_trace::ray_trace_shader};
}
