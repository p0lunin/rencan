use crate::Screen;
use std::sync::Arc;
use vulkano::{
    device::{Device, Queue},
    instance::Instance,
};

pub struct AppInfo {
    pub instance: Arc<Instance>,
    pub graphics_queue: Arc<Queue>,
    pub device: Arc<Device>,
    pub screen: Screen,
    pub render_steps: u32,
    pub msaa: u8,
}

impl AppInfo {
    pub fn new(
        instance: Arc<Instance>,
        graphics_queue: Arc<Queue>,
        device: Arc<Device>,
        screen: Screen,
        render_steps: u32,
        msaa: u8,
    ) -> Self {
        AppInfo { instance, graphics_queue, device, screen, render_steps, msaa }
    }

    pub fn size_of_image_array(&self) -> usize {
        (self.screen.width() * self.screen.height() / self.render_steps * (self.msaa as u32 * self.msaa as u32)) as usize
    }
}
