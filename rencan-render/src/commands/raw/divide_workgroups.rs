use std::sync::Arc;
use vulkano::{
    command_buffer::{AutoCommandBufferBuilder},
    descriptor::{pipeline_layout::PipelineLayout, DescriptorSet},
    device::Device,
    pipeline::ComputePipeline,
};

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

        let constants = cs::SpecializationConstants { DIVIDER: divider };
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
    ) where
        WorkgroupsSet: DescriptorSet + Send + Sync + 'static,
    {
        let sets = workgroups;

        buffer.dispatch([1, 1, 1], self.pipeline.clone(), sets, (), std::iter::empty()).unwrap();
    }
}
