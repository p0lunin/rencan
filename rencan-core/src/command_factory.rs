use crate::{
    app::{Buffers, GlobalAppBuffers},
    AppInfo, Scene,
};
use vulkano::command_buffer::AutoCommandBuffer;
use crate::camera::Camera;

pub trait CommandFactory {
    fn make_command<'m>(&self, ctx: CommandFactoryContext) -> AutoCommandBuffer;
}

#[derive(Clone)]
pub struct CommandFactoryContext<'a> {
    pub app_info: &'a AppInfo,
    pub buffers: Buffers,
    pub count_of_workgroups: u32,
    pub scene: &'a Scene,
    pub camera: &'a Camera,
}
