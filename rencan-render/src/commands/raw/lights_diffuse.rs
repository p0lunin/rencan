use crate::core::CommandFactoryContext;
use std::sync::Arc;
use vulkano::{
    buffer::{BufferAccess, TypedBufferAccess},
    command_buffer::{AutoCommandBufferBuilder, DispatchIndirectCommand},
    descriptor::{
        pipeline_layout::PipelineLayout, DescriptorSet,
    },
    device::Device,
    pipeline::ComputePipeline,
};

mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/lightning/lightning.glsl"
    }
}

pub struct LightsDiffuseCommandFactory {
    pipeline: Arc<ComputePipeline<PipelineLayout<cs::Layout>>>,
}

impl LightsDiffuseCommandFactory {
    pub fn new(device: Arc<Device>) -> Self {
        let local_size_x =
            device.physical_device().extended_properties().subgroup_size().unwrap_or(32);
        let shader = cs::Shader::load(device.clone()).unwrap();

        let constants = cs::SpecializationConstants { constant_0: local_size_x };
        let pipeline = Arc::new(
            ComputePipeline::new(device.clone(), &shader.main_entry_point(), &constants, None)
                .unwrap(),
        );
        LightsDiffuseCommandFactory { pipeline }
    }

    pub fn add_lights_diffuse_to_buffer<WI, IMS, IS, PIS>(
        &self,
        ctx: &CommandFactoryContext,
        workgroups_input: WI,
        intersections_set: IS,
        previous_intersections_set: PIS,
        image_set: IMS,
        buffer: &mut AutoCommandBufferBuilder,
    ) where
        WI: BufferAccess
            + TypedBufferAccess<Content = [DispatchIndirectCommand]>
            + Send
            + Sync
            + 'static,
        IMS: DescriptorSet + Send + Sync + 'static,
        IS: DescriptorSet + Send + Sync + 'static,
        PIS: DescriptorSet + Send + Sync + 'static,
    {
        let sets = (
            ctx.buffers.global_app_set.clone(),
            intersections_set,
            image_set,
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
}
