use crate::{app::Buffers, camera::Camera, AppInfo, Scene};
use vulkano::command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder};
use crate::auto_command_buffer_builder_wrap::AutoCommandBufferBuilderWrap;

pub trait CommandFactory {
    fn make_command<'m>(&mut self, ctx: CommandFactoryContext, commands: &mut Vec<AutoCommandBuffer>);
}

#[derive(Clone)]
pub struct CommandFactoryContext<'a> {
    pub app_info: &'a AppInfo,
    pub buffers: Buffers,
    pub scene: &'a Scene,
    pub camera: &'a Camera,
}

impl CommandFactoryContext<'_> {
    pub fn create_command_buffer(&self) -> AutoCommandBufferBuilderWrap {
        AutoCommandBufferBuilderWrap(AutoCommandBufferBuilder::new(
            self.app_info.device.clone(),
            self.app_info.graphics_queue.family(),
        ).unwrap())
    }
}
