use vulkano::descriptor::DescriptorSet;
use vulkano::command_buffer::{CommandBuffer, AutoCommandBufferBuilder, DispatchIndirectCommand};
use std::sync::Arc;
use vulkano::pipeline::ComputePipeline;
use vulkano::descriptor::pipeline_layout::PipelineLayout;
use vulkano::device::Device;
use crate::core::CommandFactoryContext;
use vulkano::descriptor::descriptor_set::DescriptorSetsCollection;
use vulkano::buffer::{BufferAccess, TypedBufferAccess};

mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/lightning/make_lightning_rays_lambertarian.glsl"
    }
}

pub struct MakeLightningRaysDiffuseCommandFactory {
    pipeline: Arc<ComputePipeline<PipelineLayout<cs::Layout>>>,
}

impl MakeLightningRaysDiffuseCommandFactory {
    pub fn new(device: Arc<Device>) -> Self {
        let local_size_x =
            device.physical_device().extended_properties().subgroup_size().unwrap_or(32);
        let shader = cs::Shader::load(device.clone()).unwrap();

        let constants = cs::SpecializationConstants {
            constant_0: local_size_x,
        };
        let pipeline = Arc::new(
            ComputePipeline::new(device.clone(), &shader.main_entry_point(), &constants, None)
                .unwrap(),
        );
        MakeLightningRaysDiffuseCommandFactory { pipeline }
    }

    pub fn add_making_rays_to_buffer<WI, RS, IS, WOS>(
        &self,
        ctx: &CommandFactoryContext,
        workgroups_input: WI,
        rays_set: RS,
        intersections_set: IS,
        workgroups_output_set: WOS,
        buffer: &mut AutoCommandBufferBuilder,
    )
    where
        WI: BufferAccess + TypedBufferAccess<Content = [DispatchIndirectCommand]> + Send + Sync + 'static,
        RS: DescriptorSet + Send + Sync + 'static,
        IS: DescriptorSet + Send + Sync + 'static,
        WOS: DescriptorSet + Send + Sync + 'static,
    {
        let sets = (
            intersections_set,
            rays_set,
            ctx.buffers.lights_set.clone(),
            workgroups_output_set,
        );

        buffer
            .dispatch_indirect(
                workgroups_input,
                self.pipeline.clone(),
                sets,
                (),
                std::iter::empty()
            )
            .unwrap();
    }
}
