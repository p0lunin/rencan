use std::sync::Arc;

use vulkano::{
    descriptor::pipeline_layout::PipelineLayout, device::Device, pipeline::ComputePipeline,
};

use crate::core::{CommandFactory, CommandFactoryContext};
use vulkano::sync::GpuFuture;

pub mod blue_sky_cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/sky/blue.glsl",
        include: ["shaders"]
    }
}

pub struct SkyCommandFactory {
    pipeline: Arc<ComputePipeline<PipelineLayout<blue_sky_cs::MainLayout>>>,
    local_size_x: u32,
}

impl SkyCommandFactory {
    pub fn new(device: Arc<Device>) -> Self {
        let local_size_x =
            device.physical_device().extended_properties().subgroup_size().unwrap_or(32);

        let constants = blue_sky_cs::SpecializationConstants { constant_0: local_size_x };

        let pipeline = Arc::new(
            ComputePipeline::new(
                device.clone(),
                &blue_sky_cs::Shader::load(device).unwrap().main_entry_point(),
                &constants,
                None,
            )
            .unwrap(),
        );
        SkyCommandFactory { pipeline, local_size_x }
    }
}

impl CommandFactory for SkyCommandFactory {
    fn make_command(
        &mut self,
        ctx: CommandFactoryContext,
        fut: Box<dyn GpuFuture>,
    ) -> Box<dyn GpuFuture> {
        let set_0 = ctx.buffers.global_app_set.clone();
        let set_1 = ctx.buffers.image_set.clone();

        let command = ctx
            .create_command_buffer()
            .dispatch(
                ctx.app_info.size_of_image_array() as u32 * ctx.app_info.render_steps / self.local_size_x,
                self.pipeline.clone(),
                (set_0, set_1),
            )
            .unwrap()
            .build();

        Box::new(fut.then_execute(ctx.graphics_queue(), command).unwrap())
    }
}
