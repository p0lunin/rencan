use crate::core::{CommandFactoryContext, AppInfo};
use std::sync::Arc;
use vulkano::{
    buffer::{BufferAccess, TypedBufferAccess},
    command_buffer::{AutoCommandBufferBuilder, DispatchIndirectCommand},
    descriptor::{
        descriptor_set::UnsafeDescriptorSetLayout,
        pipeline_layout::{PipelineLayout},
        DescriptorSet, PipelineLayoutAbstract,
    },
    device::Device,
    pipeline::ComputePipeline,
};

mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/lightning/reflect_from_mirrors.glsl"
    }
}

pub struct ReflectFromMirrorsCommandFactory {
    pipeline: Arc<ComputePipeline<PipelineLayout<cs::MainLayout>>>,
}

impl ReflectFromMirrorsCommandFactory {
    pub fn new(info: &AppInfo) -> Self {
        let device = &info.device;
        let shader = cs::Shader::load(device.clone()).unwrap();
        let local_size_x = info.recommend_workgroups_length;

        let constants = cs::SpecializationConstants { constant_0: local_size_x };
        let pipeline = Arc::new(
            ComputePipeline::new(device.clone(), &shader.main_entry_point(), &constants, None)
                .unwrap(),
        );
        ReflectFromMirrorsCommandFactory { pipeline }
    }

    pub fn add_reflects_rays_to_buffer<WI, IntersSer>(
        &self,
        ctx: &CommandFactoryContext,
        workgroups_input: WI,
        intersections_set: IntersSer,
        buffer: &mut AutoCommandBufferBuilder,
    ) where
        WI: BufferAccess
            + TypedBufferAccess<Content = [DispatchIndirectCommand]>
            + Send
            + Sync
            + 'static,
        IntersSer: DescriptorSet + Send + Sync + 'static,
    {
        let sets = (
            ctx.buffers.models_set.clone(),
            ctx.buffers.sphere_models_set.clone(),
            intersections_set,
        );

        buffer
            .dispatch_indirect(
                workgroups_input,
                self.pipeline.clone(),
                sets,
                (),
                std::iter::empty(),
            )
            .unwrap();
    }
}
