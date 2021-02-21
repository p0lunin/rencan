use crate::{AppInfo, Scene};
use std::sync::Arc;
use vulkano::{command_buffer::AutoCommandBuffer, descriptor::DescriptorSet};

pub trait CommandFactory {
    fn make_command<'m>(&self, ctx: CommandFactoryContext) -> AutoCommandBuffer;
}

#[derive(Clone)]
pub struct CommandFactoryContext<'a> {
    pub app_info: &'a AppInfo,
    pub global_set: Arc<dyn DescriptorSet + Send + Sync>,
    pub count_of_workgroups: u32,
    pub scene: &'a Scene,
}
