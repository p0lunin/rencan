use crate::{
    app::{Buffers, GlobalAppBuffers},
    AppInfo, Scene,
};
use vulkano::command_buffer::AutoCommandBuffer;

pub trait CommandFactory {
    fn make_command<'m>(&self, ctx: CommandFactoryContext) -> AutoCommandBuffer;
    fn update_buffers(&mut self, buffers: GlobalAppBuffers);
}

#[derive(Clone)]
pub struct CommandFactoryContext<'a> {
    pub app_info: &'a AppInfo,
    pub buffers: Buffers,
    pub count_of_workgroups: u32,
    pub scene: &'a Scene,
}
