use vulkano::descriptor::DescriptorSet;
use vulkano::command_buffer::{CommandBuffer, AutoCommandBufferBuilder};
use std::sync::Arc;
use vulkano::pipeline::ComputePipeline;
use vulkano::descriptor::pipeline_layout::PipelineLayout;
use vulkano::device::Device;
use crate::core::CommandFactoryContext;

mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/divide_workgroup_size.glsl"
    }
}

pub struct DivideWorkgroupsCommandFactory {
    pipeline: Arc<ComputePipeline<PipelineLayout<cs::Layout>>>,
}

impl DivideWorkgroupsCommandFactory {
    pub fn new(device: Arc<Device>, divider: u32) -> Self {
        let shader = cs::Shader::load(device.clone()).unwrap();

        let constants = cs::SpecializationConstants {
            DIVIDER: divider,
        };
        let pipeline = Arc::new(
            ComputePipeline::new(device.clone(), &shader.main_entry_point(), &constants, None)
                .unwrap(),
        );
        DivideWorkgroupsCommandFactory { pipeline }
    }

    pub fn add_divider_to_buffer<WorkgroupsSet>(
        &self,
        workgroups: WorkgroupsSet,
        buffer: &mut AutoCommandBufferBuilder,
    )
    where
        WorkgroupsSet: DescriptorSet + Send + Sync + 'static,
    {
        let sets = workgroups;

        buffer
            .dispatch(
                [1, 1, 1],
                self.pipeline.clone(),
                sets,
                (),
                std::iter::empty()
            )
            .unwrap();
    }
}
