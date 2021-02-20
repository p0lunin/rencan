use std::sync::Arc;

use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder},
    descriptor::{
        descriptor_set::PersistentDescriptorSet, pipeline_layout::PipelineLayout,
        PipelineLayoutAbstract,
    },
    device::Device,
    pipeline::ComputePipeline,
};

use crate::core::{CommandFactory, CommandFactoryContext};

mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/facing_ratio.glsl"
    }
}

pub struct FacingRatioCommandFactory {
    pipeline: Arc<ComputePipeline<PipelineLayout<cs::Layout>>>,
}

impl FacingRatioCommandFactory {
    pub fn new(device: Arc<Device>) -> Self {
        let shader = cs::Shader::load(device.clone()).unwrap();
        let pipeline = Arc::new(
            ComputePipeline::new(device.clone(), &shader.main_entry_point(), &(), None)
                .unwrap(),
        );
        FacingRatioCommandFactory { pipeline }
    }
}

impl CommandFactory for FacingRatioCommandFactory {
    fn make_command(&self, ctx: CommandFactoryContext) -> AutoCommandBuffer {
        let CommandFactoryContext { app_info, global_set, count_of_workgroups, models } = ctx;
        let device = app_info.device.clone();

        let layout_1 = self.pipeline.layout().descriptor_set_layout(1).unwrap();
        let mut command =
            AutoCommandBufferBuilder::new(device.clone(), app_info.graphics_queue.family())
                .unwrap();

        for (i, model) in models.iter().enumerate() {
            let vertices_buffer = CpuAccessibleBuffer::from_iter(
                device.clone(),
                BufferUsage::all(),
                false,
                model.vertices.iter().cloned(),
            )
            .unwrap();
            let model_info_buffer = CpuAccessibleBuffer::from_data(
                device.clone(),
                BufferUsage::all(),
                false,
                model.get_uniform_info(i as u32).as_std140(),
            )
            .unwrap();
            let indices_buffer = CpuAccessibleBuffer::from_iter(
                device.clone(),
                BufferUsage::all(),
                false,
                model.indexes.iter().cloned(),
            )
            .unwrap();

            let set_1 = Arc::new(
                PersistentDescriptorSet::start(layout_1.clone())
                    .add_buffer(model_info_buffer)
                    .unwrap()
                    .add_buffer(vertices_buffer)
                    .unwrap()
                    .add_buffer(indices_buffer)
                    .unwrap()
                    .build()
                    .unwrap(),
            );

            command
                .dispatch(
                    [count_of_workgroups, 1, 1],
                    self.pipeline.clone(),
                    (global_set.clone(), set_1),
                    (),
                )
                .unwrap();
        }

        let command = command.build().unwrap();

        command
    }
}
