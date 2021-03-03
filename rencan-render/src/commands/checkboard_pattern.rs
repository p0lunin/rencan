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
    intersections: Arc<dyn BufferAccessData<Data = [IntersectionUniform]> + Send + Sync>,
}

impl CheckBoardCommandFactory {
    pub fn new(buffers: GlobalAppBuffers, device: Arc<Device>, scale: f32) -> Self {
        let shader = cs::Shader::load(device.clone()).unwrap();
        let constants = cs::SpecializationConstants { CHESSBOARD_SCALE: scale };
        let pipeline = Arc::new(
            ComputePipeline::new(device.clone(), &shader.main_entry_point(), &constants, None)
                .unwrap(),
        );
        let intersections = buffers.intersections;
        CheckBoardCommandFactory { pipeline, intersections }
    }
}

impl CommandFactory for CheckBoardCommandFactory {
    fn make_command(&self, ctx: CommandFactoryContext) -> AutoCommandBuffer {
        let CommandFactoryContext { app_info, buffers, count_of_workgroups, scene } = ctx;
        let device = app_info.device.clone();

        let layout_0 = self.pipeline.layout().descriptor_set_layout(0).unwrap();

        let set_0 = Arc::new(
            PersistentDescriptorSet::start(layout_0.clone())
                .add_buffer(buffers.screen.clone())
                .unwrap()
                .add_image(buffers.output_image.clone())
                .unwrap()
                .add_buffer(self.intersections.clone())
                .unwrap()
                .add_buffer(buffers.direction_light.clone())
                .unwrap()
                .build()
                .unwrap(),
        );

        let layout_1 = self.pipeline.layout().descriptor_set_layout(1).unwrap();

        let set_1 = Arc::new(
            PersistentDescriptorSet::start(layout_1.clone())
                .add_buffer(buffers.models_buffers.infos.clone())
                .unwrap()
                .add_buffer(buffers.models_buffers.vertices.clone())
                .unwrap()
                .add_buffer(buffers.models_buffers.indices.clone())
                .unwrap()
                .build()
                .unwrap(),
        );

        let mut command =
            AutoCommandBufferBuilder::new(device.clone(), app_info.graphics_queue.family())
                .unwrap();

        command
            .dispatch([count_of_workgroups, 1, 1], self.pipeline.clone(), (set_0, set_1), ())
            .unwrap();

        let command = command.build().unwrap();

        command
    }

    fn update_buffers(&mut self, buffers: GlobalAppBuffers) {
        self.intersections = buffers.intersections;
    }
}
