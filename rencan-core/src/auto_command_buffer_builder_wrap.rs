use vulkano::command_buffer::{AutoCommandBufferBuilder, DispatchError, AutoCommandBuffer};
use vulkano::pipeline::ComputePipelineAbstract;
use vulkano::descriptor::descriptor_set::DescriptorSetsCollection;

pub struct AutoCommandBufferBuilderWrap(pub AutoCommandBufferBuilder);

impl AutoCommandBufferBuilderWrap {
    pub fn dispatch<Cp, S>(mut self, workgroups: u32, pipeline: Cp, sets: S) -> Result<Self, DispatchError>
    where
        Cp: ComputePipelineAbstract + Send + Sync + 'static + Clone,
        S: DescriptorSetsCollection,
    {
        self.0.dispatch([workgroups, 1, 1], pipeline, sets, ())?;
        Ok(self)
    }

    pub fn build(self) -> AutoCommandBuffer {
        self.0.build().unwrap()
    }
}
