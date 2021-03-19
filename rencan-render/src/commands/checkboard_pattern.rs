use std::sync::Arc;

use vulkano::{
    command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder},
    descriptor::{
        descriptor_set::PersistentDescriptorSet, pipeline_layout::PipelineLayout,
        PipelineLayoutAbstract,
    },
    device::Device,
    pipeline::ComputePipeline,
};

use crate::core::{
    app::GlobalAppBuffers, intersection::IntersectionUniform, BufferAccessData, CommandFactory,
    CommandFactoryContext,
};

mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/checkboard_pattern.glsl"
    }
}

pub struct CheckBoardCommandFactory {
    pipeline: Arc<ComputePipeline<PipelineLayout<cs::Layout>>>,
}

impl CheckBoardCommandFactory {
    pub fn new(device: Arc<Device>, scale: f32) -> Self {
        let shader = cs::Shader::load(device.clone()).unwrap();
        let constants = cs::SpecializationConstants { CHESSBOARD_SCALE: scale };
        let pipeline = Arc::new(
            ComputePipeline::new(device.clone(), &shader.main_entry_point(), &constants, None)
                .unwrap(),
        );
        CheckBoardCommandFactory { pipeline }
    }
}

impl CommandFactory for CheckBoardCommandFactory {
    fn make_command(&self, ctx: CommandFactoryContext, commands: &mut Vec<AutoCommandBuffer>) {
        let CommandFactoryContext { app_info, buffers, .. } = ctx;
        let device = app_info.device.clone();

        let set_0 = buffers.global_app_set.clone();
        let set_1 = buffers.rays_set.clone();
        let set_2 = buffers.models_set.clone();
        let set_3 = buffers.image_set.clone();

        let mut command =
            AutoCommandBufferBuilder::new(device.clone(), app_info.graphics_queue.family())
                .unwrap();

        command
            .dispatch([ctx.app_info.size_of_image_array() as u32 / 64, 1, 1], self.pipeline.clone(), (set_0, set_1, set_2, set_3), ())
            .unwrap();

        let command = command.build().unwrap();

        commands.push(command);
    }
}
