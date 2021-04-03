use std::sync::Arc;

use vulkano::{
    command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder},
    descriptor::pipeline_layout::PipelineLayout,
    device::Device,
    pipeline::ComputePipeline,
};

use crate::core::{CommandFactory, CommandFactoryContext, AutoCommandBufferBuilderWrap};
use vulkano::command_buffer::CommandBuffer;
use vulkano::sync::GpuFuture;

pub mod lightning_cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/lightning.glsl"
    }
}

pub struct LightningCommandFactory {
    lightning_pipeline: Arc<ComputePipeline<PipelineLayout<lightning_cs::Layout>>>,
    local_size_x: u32,
}

impl LightningCommandFactory {
    pub fn new(device: Arc<Device>, sampling: bool) -> Self {
        let local_size_x = device.physical_device().extended_properties().subgroup_size().unwrap_or(32);

        let constants = lightning_cs::SpecializationConstants {
            constant_0: local_size_x,
            SAMPLING: if sampling { 1 } else { 0 },
        };

        let lightning_pipeline = Arc::new(
            ComputePipeline::new(
                device.clone(),
                &lightning_cs::Shader::load(device).unwrap().main_entry_point(),
                &constants,
                None,
            )
            .unwrap(),
        );
        LightningCommandFactory { lightning_pipeline, local_size_x }
    }
}

impl CommandFactory for LightningCommandFactory {
    fn make_command(&mut self, ctx: CommandFactoryContext, fut: Box<dyn GpuFuture>) -> Box<dyn GpuFuture> {
        let command = add_lightning(self, &ctx).build();

        Box::new(fut.then_execute(ctx.graphics_queue(), command).unwrap())
    }
}

fn add_lightning(
    factory: &LightningCommandFactory,
    ctx: &CommandFactoryContext,
) -> AutoCommandBufferBuilderWrap {
    let CommandFactoryContext { buffers, .. } = ctx;

    let set_0 = buffers.global_app_set.clone();
    let set_1 = buffers.rays_set.clone();
    let set_2 = buffers.models_set.clone();
    let set_3 = buffers.sphere_models_set.clone();
    let set_4 = buffers.lights_set.clone();
    let set_5 = buffers.image_set.clone();

    ctx
        .create_command_buffer()
        .dispatch_indirect(
            buffers.workgroups.clone(),
            factory.lightning_pipeline.clone(),
            (set_0, set_1, set_2, set_3, set_4, set_5),
        )
}
