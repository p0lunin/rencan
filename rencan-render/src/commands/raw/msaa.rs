use crate::core::{AppInfo, CommandFactoryContext};
use std::sync::Arc;
use vulkano::{
    buffer::{BufferAccess, TypedBufferAccess},
    command_buffer::{AutoCommandBufferBuilder, DispatchIndirectCommand},
    descriptor::{
        descriptor_set::UnsafeDescriptorSetLayout, pipeline_layout::PipelineLayout, DescriptorSet,
        PipelineLayoutAbstract,
    },
    device::Device,
    pipeline::ComputePipeline,
};

mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/msaa.glsl"
    }
}

pub struct MsaaCommandFactory {
    pipeline: Arc<ComputePipeline<PipelineLayout<cs::MainLayout>>>,
}

impl MsaaCommandFactory {
    pub fn new(info: &AppInfo, msaa_multiplier: u32) -> Self {
        let device = &info.device;
        let shader = cs::Shader::load(device.clone()).unwrap();
        let local_size_x = info.recommend_workgroups_length;

        let shader = cs::Shader::load(device.clone()).unwrap();
        let constants =
            cs::SpecializationConstants { constant_0: local_size_x, constant_1: msaa_multiplier };
        let pipeline = Arc::new(
            ComputePipeline::new(device.clone(), &shader.main_entry_point(), &constants, None)
                .unwrap(),
        );
        MsaaCommandFactory { pipeline }
    }

    pub fn add_msaa<OIS>(
        &self,
        ctx: &CommandFactoryContext,
        output_image_set: OIS,
        buffer: &mut AutoCommandBufferBuilder,
    ) where
        OIS: DescriptorSet + Send + Sync + 'static,
    {
        let sets =
            (ctx.buffers.global_app_set.clone(), ctx.buffers.image_set.clone(), output_image_set);

        buffer
            .dispatch(
                [ctx.app_info.screen.size(), 1, 1],
                self.pipeline.clone(),
                sets,
                (),
                std::iter::empty(),
            )
            .unwrap();
    }

    pub fn output_image_layout(&self) -> Arc<UnsafeDescriptorSetLayout> {
        self.pipeline.layout().descriptor_set_layout(2).unwrap().clone()
    }
}
