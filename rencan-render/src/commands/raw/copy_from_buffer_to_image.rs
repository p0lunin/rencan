use vulkano::descriptor::{DescriptorSet, PipelineLayoutAbstract};
use vulkano::command_buffer::{CommandBuffer, AutoCommandBufferBuilder, DispatchIndirectCommand};
use std::sync::Arc;
use vulkano::pipeline::ComputePipeline;
use vulkano::descriptor::pipeline_layout::{PipelineLayout, PipelineLayoutDesc};
use vulkano::device::Device;
use crate::core::CommandFactoryContext;
use vulkano::descriptor::descriptor::ShaderStages;
use vulkano::buffer::{BufferAccess, TypedBufferAccess};
use vulkano::descriptor::descriptor_set::UnsafeDescriptorSetLayout;

mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/lightning/copy_from_buffer_to_image.glsl"
    }
}

pub struct CopyFromBufferToImageCommandFactory {
    pipeline: Arc<ComputePipeline<PipelineLayout<cs::Layout>>>,
    local_size_x: u32,
}

impl CopyFromBufferToImageCommandFactory {
    pub fn new(device: Arc<Device>) -> Self {
        let local_size_x =
            device.physical_device().extended_properties().subgroup_size().unwrap_or(32);

        let shader = cs::Shader::load(device.clone()).unwrap();
        let constants = cs::SpecializationConstants {
            constant_0: local_size_x,
        };
        let pipeline = Arc::new(
            ComputePipeline::new(
                device.clone(),
                &shader.main_entry_point(),
                &constants,
                None
            ).unwrap(),
        );
        CopyFromBufferToImageCommandFactory { pipeline, local_size_x }
    }

    pub fn add_copy<CI>(
        &self,
        ctx: &CommandFactoryContext,
        colors_input: CI,
        buffer: &mut AutoCommandBufferBuilder,
    )
    where
        CI: DescriptorSet + Send + Sync + 'static,
    {
        let sets = (
            ctx.buffers.global_app_set.clone(),
            colors_input,
            ctx.buffers.image_set.clone(),
        );

        buffer
            .dispatch(
                [ctx.app_info.size_of_image_array() as u32 / self.local_size_x, 1, 1],
                self.pipeline.clone(),
                sets,
                (),
                std::iter::empty()
            )
            .unwrap();
    }

    fn image_buffer_set_layout(&self) -> Arc<UnsafeDescriptorSetLayout> {
        self.pipeline.descriptor_set_layout(1).unwrap().clone()
    }
}
