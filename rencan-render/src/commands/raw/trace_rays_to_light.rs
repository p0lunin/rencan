use crate::core::CommandFactoryContext;
use std::sync::Arc;
use vulkano::{
    buffer::{BufferAccess, TypedBufferAccess},
    command_buffer::{AutoCommandBufferBuilder, CommandBuffer, DispatchIndirectCommand},
    descriptor::{
        descriptor::ShaderStages,
        descriptor_set::UnsafeDescriptorSetLayout,
        pipeline_layout::{PipelineLayout, PipelineLayoutDesc},
        DescriptorSet, PipelineLayoutAbstract,
    },
    device::Device,
    pipeline::ComputePipeline,
};

mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/lightning/trace_rays_to_lights.glsl"
    }
}

pub struct TraceRaysToLightCommandFactory {
    pipeline: Arc<ComputePipeline<PipelineLayout<cs::Layout>>>,
    local_size_x: u32,
}

impl TraceRaysToLightCommandFactory {
    pub fn new(device: Arc<Device>) -> Self {
        let local_size_x =
            device.physical_device().extended_properties().subgroup_size().unwrap_or(32);

        let shader = cs::Shader::load(device.clone()).unwrap();
        let constants = cs::SpecializationConstants { constant_0: local_size_x };
        let pipeline = Arc::new(
            ComputePipeline::new(device.clone(), &shader.main_entry_point(), &constants, None)
                .unwrap(),
        );
        TraceRaysToLightCommandFactory { pipeline, local_size_x }
    }

    pub fn add_trace_rays_to_buffer<PIS, WI, WOS, IntersSer>(
        &self,
        ctx: &CommandFactoryContext,
        workgroups_input: WI,
        previous_intersections_set: PIS,
        intersections_set: IntersSer,
        workgroups_out_set: WOS,
        buffer: &mut AutoCommandBufferBuilder,
    ) where
        PIS: DescriptorSet + Send + Sync + 'static,
        WI: BufferAccess
            + TypedBufferAccess<Content = [DispatchIndirectCommand]>
            + Send
            + Sync
            + 'static,
        WOS: DescriptorSet + Send + Sync + 'static,
        IntersSer: DescriptorSet + Send + Sync + 'static,
    {
        let sets = (
            intersections_set,
            ctx.buffers.models_set.clone(),
            ctx.buffers.sphere_models_set.clone(),
            ctx.buffers.lights_set.clone(),
            workgroups_out_set,
            previous_intersections_set,
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

    pub fn rays_layout(&self) -> Arc<UnsafeDescriptorSetLayout> {
        self.pipeline.layout().descriptor_set_layout(0).unwrap().clone()
    }
    pub fn intersections_layout(&self) -> Arc<UnsafeDescriptorSetLayout> {
        self.pipeline.layout().descriptor_set_layout(0).unwrap().clone()
    }
}
