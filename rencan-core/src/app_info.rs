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
}

impl AppInfo {
    pub fn new(
        instance: Arc<Instance>,
        graphics_queue: Arc<Queue>,
        device: Arc<Device>,
        screen: Screen,
    ) -> Self {
        AppInfo { instance, graphics_queue, device, screen }
    }

    pub fn size_of_image_array(&self) -> usize {
        (self.screen.width() * self.screen.height()) as usize
    }
}
