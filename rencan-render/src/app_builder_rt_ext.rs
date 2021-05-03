use crate::commands;
use rencan_core::app::AppBuilder;

pub trait AppBuilderRtExt: Sized {
    fn then_ray_tracing_pipeline(self) -> Self;
}

impl AppBuilderRtExt for AppBuilder {
    fn then_ray_tracing_pipeline(self) -> Self {
        let cmd = Box::new(commands::RayTraceCommandFactory::new(self.info()));
        self.then_command(cmd)
    }
}
