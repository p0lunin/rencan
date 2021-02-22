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
        path: "shaders/lightning.glsl"
    }
}

pub struct LightningCommandFactory {
    pipeline: Arc<ComputePipeline<PipelineLayout<cs::Layout>>>,
}

impl LightningCommandFactory {
    pub fn new(device: Arc<Device>) -> Self {
        let shader = cs::Shader::load(device.clone()).unwrap();
        let pipeline = Arc::new(
            ComputePipeline::new(device.clone(), &shader.main_entry_point(), &(), None).unwrap(),
        );
        LightningCommandFactory { pipeline }
    }
}

impl CommandFactory for LightningCommandFactory {
    fn make_command(&self, ctx: CommandFactoryContext) -> AutoCommandBuffer {
        let CommandFactoryContext { app_info, buffers, count_of_workgroups, scene } = ctx;
        let device = app_info.device.clone();

        let layout_0 = self.pipeline.layout().descriptor_set_layout(0).unwrap();
        let set_0 = Arc::new(
            PersistentDescriptorSet::start(layout_0.clone())
                .add_buffer(buffers.screen.clone())
                .unwrap()
                .add_buffer(buffers.rays.clone())
                .unwrap()
                .add_image(buffers.output_image.clone())
                .unwrap()
                .add_buffer(buffers.intersections.clone())
                .unwrap()
                .add_buffer(buffers.direction_light.clone())
                .unwrap()
                .build()
                .unwrap()
        );

        let layout_1 = self.pipeline.layout().descriptor_set_layout(1).unwrap();
        let mut command =
            AutoCommandBufferBuilder::new(device.clone(), app_info.graphics_queue.family())
                .unwrap();

        for (i, model) in scene.models.iter().enumerate() {
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
                    (set_0.clone(), set_1),
                    (),
                )
                .unwrap();
        }

        let command = command.build().unwrap();

        command
    }
}
