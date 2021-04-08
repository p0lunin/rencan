use vulkano::{
    buffer::{BufferAccess, TypedBufferAccess},
    command_buffer::{
        AutoCommandBuffer, AutoCommandBufferBuilder, DispatchError, DispatchIndirectCommand,
    },
    descriptor::descriptor_set::DescriptorSetsCollection,
    pipeline::ComputePipelineAbstract,
};

pub struct AutoCommandBufferBuilderWrap(pub AutoCommandBufferBuilder);

impl AutoCommandBufferBuilderWrap {
    pub fn dispatch<Cp, S>(
        mut self,
        workgroups: u32,
        pipeline: Cp,
        sets: S,
    ) -> Result<Self, DispatchError>
    where
        Cp: ComputePipelineAbstract + Send + Sync + 'static + Clone,
        S: DescriptorSetsCollection,
    {
        self.0.dispatch([workgroups, 1, 1], pipeline, sets, (), std::iter::empty())?;
        Ok(self)
    }
    pub fn dispatch_indirect<Buff, Cp, S>(mut self, workgroups: Buff, pipeline: Cp, sets: S) -> Self
    where
        Buff: BufferAccess
            + TypedBufferAccess<Content = [DispatchIndirectCommand]>
            + Send
            + Sync
            + 'static,
        Cp: ComputePipelineAbstract + Send + Sync + 'static + Clone,
        S: DescriptorSetsCollection,
    {
        self.0.dispatch_indirect(workgroups, pipeline, sets, (), std::iter::empty()).unwrap();
        self
    }

    pub fn update_with(mut self, f: impl FnOnce(&mut Self)) -> Self {
        f(&mut self);
        self
    }

    pub fn build(self) -> Box<AutoCommandBuffer> {
        Box::new(self.0.build().unwrap())
    }
}
