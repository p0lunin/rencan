use crate::commands;
use rencan_core::app::AppBuilder;

pub trait AppBuilderRtExt: Sized {
    fn then_ray_tracing_pipeline(self) -> Self;
}

impl AppBuilderRtExt for AppBuilder {
    fn then_ray_tracing_pipeline(self) -> Self {
        let device = self.info().device.clone();
        self.then_command(|bufs| Box::new(commands::ComputeRaysCommandFactory::new(bufs, device.clone())))
            .then_command(move |bufs| Box::new(commands::RayTraceCommandFactory::new(bufs, device)))
    }
}
