use crate::{app::Buffers, camera::Camera, AppInfo, Scene};
use vulkano::command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder, CommandBuffer};
use crate::auto_command_buffer_builder_wrap::AutoCommandBufferBuilderWrap;
use vulkano::sync::GpuFuture;
use vulkano::device::Queue;
use std::sync::Arc;

pub trait CommandFactory {
    fn make_command<'m>(&mut self, ctx: CommandFactoryContext, prev: Box<dyn GpuFuture>) -> Box<dyn GpuFuture>;
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
    pub fn graphics_queue(&self) -> Arc<Queue> {
        self.app_info.graphics_queue.clone()
    }
}
