use std::sync::Arc;

use vulkano::{
    descriptor::pipeline_layout::PipelineLayout, device::Device, pipeline::ComputePipeline,
};

use crate::core::{CommandFactory, CommandFactoryContext};
use vulkano::sync::GpuFuture;

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
    fn make_command(
        &mut self,
        ctx: CommandFactoryContext,
        fut: Box<dyn GpuFuture>,
    ) -> Box<dyn GpuFuture> {
        let buffers = &ctx.buffers;

        let set_0 = buffers.global_app_set.clone();
        let set_1 = buffers.intersections_set.clone();
        let set_2 = buffers.models_set.clone();
        let set_3 = buffers.image_set.clone();

        let command = ctx
            .create_command_buffer()
            .dispatch(
                ctx.app_info.size_of_image_array() as u32 / 64,
                self.pipeline.clone(),
                (set_0, set_1, set_2, set_3),
            )
            .unwrap()
            .build();

        Box::new(fut.then_execute(ctx.graphics_queue(), command).unwrap())
    }
}
