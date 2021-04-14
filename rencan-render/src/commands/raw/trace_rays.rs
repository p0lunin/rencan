use crate::core::CommandFactoryContext;
use std::sync::Arc;
use vulkano::{
    command_buffer::{AutoCommandBufferBuilder},
    descriptor::{
        descriptor::ShaderStages,
        pipeline_layout::{PipelineLayout},
        DescriptorSet, PipelineLayoutAbstract,
    },
    device::Device,
    pipeline::ComputePipeline,
};

mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/trace_rays.glsl"
    }
}

pub enum TraceType {
    Nearest,
    First,
}

pub struct TraceRaysCommandFactory {
    pipeline: Arc<ComputePipeline<Box<dyn PipelineLayoutAbstract + Send + Sync>>>,
    local_size_x: u32,
}

impl TraceRaysCommandFactory {
    pub fn new(device: Arc<Device>, trace_type: TraceType) -> Self {
        let local_size_x =
            device.physical_device().extended_properties().subgroup_size().unwrap_or(32);
        let pipeline = match trace_type {
            TraceType::Nearest => {
                let shader = cs::Shader::load(device.clone()).unwrap();
                let constants = cs::SpecializationConstants { constant_0: local_size_x };
                Arc::new(
                    ComputePipeline::<Box<dyn PipelineLayoutAbstract + Send + Sync>>::with_pipeline_layout(
                        device.clone(),
                        &shader.main_entry_point(),
                        &constants,
                        Box::new(
                            PipelineLayout::new(
                                device.clone(),
                                cs::MainLayout(ShaderStages::compute())
                            ).unwrap()
                        ),
                        None
                    ).unwrap(),
                )
            }
            TraceType::First => {
                unimplemented!()
            }
        };
        TraceRaysCommandFactory { pipeline, local_size_x }
    }

    pub fn add_trace_rays_to_buffer<RaysSet, IntersSer, WorkgroupsSet>(
        &self,
        ctx: &CommandFactoryContext,
        rays_set: RaysSet,
        intersections_set: IntersSer,
        workgroups: WorkgroupsSet,
        buffer: &mut AutoCommandBufferBuilder,
    ) where
        RaysSet: DescriptorSet + Send + Sync + 'static,
        IntersSer: DescriptorSet + Send + Sync + 'static,
        WorkgroupsSet: DescriptorSet + Send + Sync + 'static,
    {
        let sets = (
            rays_set,
            intersections_set,
            ctx.buffers.models_set.clone(),
            ctx.buffers.sphere_models_set.clone(),
            workgroups,
        );

        buffer
            .dispatch(
                [ctx.app_info.size_of_image_array() as u32 / self.local_size_x, 1, 1],
                self.pipeline.clone(),
                sets,
                (),
                std::iter::empty(),
            )
            .unwrap();
    }
}
