use std::sync::Arc;

use vulkano::{
    command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder},
    descriptor::pipeline_layout::PipelineLayout,
    device::Device,
    pipeline::ComputePipeline,
};

use crate::core::{CommandFactory, CommandFactoryContext};

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
    pub fn new(device: Arc<Device>) -> Self {
        let local_size_x = device.physical_device().extended_properties().subgroup_size().unwrap_or(32);

        let constants = lightning_cs::SpecializationConstants {
            constant_0: local_size_x,
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
    fn make_command(&self, ctx: CommandFactoryContext, commands: &mut Vec<AutoCommandBuffer>) {
        let mut command = AutoCommandBufferBuilder::new(
            ctx.app_info.device.clone(),
            ctx.app_info.graphics_queue.family(),
        )
        .unwrap();

        add_lightning(self, &ctx, &mut command);

        let command = command.build().unwrap();

        commands.push(command)
    }
}

fn add_lightning(
    factory: &LightningCommandFactory,
    ctx: &CommandFactoryContext,
    command: &mut AutoCommandBufferBuilder,
) {
    let CommandFactoryContext { buffers, .. } = ctx;

    let set_0 = buffers.global_app_set.clone();
    let set_1 = buffers.rays_set.clone();
    let set_2 = buffers.models_set.clone();
    let set_3 = buffers.lights_set.clone();
    let set_4 = buffers.image_set.clone();

    command
        .dispatch(
            [ctx.app_info.size_of_image_array() as u32 / factory.local_size_x, 1, 1],
            factory.lightning_pipeline.clone(),
            (set_0, set_1, set_2, set_3, set_4),
            (),
        )
        .unwrap();
}
