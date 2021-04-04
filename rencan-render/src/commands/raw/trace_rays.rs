use vulkano::descriptor::{DescriptorSet, PipelineLayoutAbstract};
use vulkano::command_buffer::{CommandBuffer, AutoCommandBufferBuilder};
use std::sync::Arc;
use vulkano::pipeline::ComputePipeline;
use vulkano::descriptor::pipeline_layout::{PipelineLayout, PipelineLayoutDesc};
use vulkano::device::Device;
use crate::core::CommandFactoryContext;
use vulkano::descriptor::descriptor::ShaderStages;

mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/trace_rays.glsl"
    }
}

mod cs_first {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/trace_rays_first.glsl"
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
                let constants = cs::SpecializationConstants {
                    constant_0: local_size_x,
                };
                Arc::new(
                    ComputePipeline::<Box<dyn PipelineLayoutAbstract + Send + Sync>>::with_pipeline_layout(
                        device.clone(),
                        &shader.main_entry_point(),
                        &constants,
                        Box::new(
                            PipelineLayout::new(
                                device.clone(),
                                cs::Layout(ShaderStages::compute())
                            ).unwrap()
                        ),
                        None
                    ).unwrap(),
                )
            }
            TraceType::First => {
                let shader = cs_first::Shader::load(device.clone()).unwrap();
                let constants = cs_first::SpecializationConstants {
                    constant_0: local_size_x,
                };
                Arc::new(
                    ComputePipeline::<Box<dyn PipelineLayoutAbstract + Send + Sync>>::with_pipeline_layout(
                        device.clone(),
                        &shader.main_entry_point(),
                        &constants,
                        Box::new(
                            PipelineLayout::new(
                                device.clone(),
                                cs_first::Layout(ShaderStages::compute())
                            ).unwrap()
                        ),
                        None
                    ).unwrap(),
                )
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
    )
    where
        RaysSet: DescriptorSet + Send + Sync + 'static,
        IntersSer: DescriptorSet + Send + Sync + 'static,
        WorkgroupsSet: DescriptorSet + Send + Sync + 'static,
    {
        let sets = (
            rays_set,
            intersections_set,
            ctx.buffers.models_set.clone(),
            ctx.buffers.sphere_models_set.clone(),
            workgroups
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
}
