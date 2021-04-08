use crate::{
    app::Buffers, auto_command_buffer_builder_wrap::AutoCommandBufferBuilderWrap, camera::Camera,
    AppInfo, Scene,
};
use std::sync::Arc;
use vulkano::{
    buffer::{BufferUsage, DeviceLocalBuffer},
    command_buffer::AutoCommandBufferBuilder,
    device::Queue,
    sync::GpuFuture,
};

pub trait CommandFactory {
    fn make_command<'m>(
        &mut self,
        ctx: CommandFactoryContext,
        prev: Box<dyn GpuFuture>,
    ) -> Box<dyn GpuFuture>;
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
        AutoCommandBufferBuilderWrap(
            AutoCommandBufferBuilder::new(
                self.app_info.device.clone(),
                self.app_info.graphics_queue.family(),
            )
            .unwrap(),
        )
    }
    pub fn graphics_queue(&self) -> Arc<Queue> {
        self.app_info.graphics_queue.clone()
    }
    pub fn create_device_local_buffer_array<T>(
        &self,
        len: usize,
        usage: BufferUsage,
    ) -> Arc<DeviceLocalBuffer<[T]>> {
        DeviceLocalBuffer::array(
            self.app_info.device.clone(),
            len,
            usage,
            std::iter::once(self.graphics_queue().family()),
        )
        .unwrap()
    }
    pub fn create_device_local_buffer<T>(&self, usage: BufferUsage) -> Arc<DeviceLocalBuffer<T>> {
        DeviceLocalBuffer::new(
            self.app_info.device.clone(),
            usage,
            std::iter::once(self.graphics_queue().family()),
        )
        .unwrap()
    }
}
