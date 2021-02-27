use crate::{light::DirectionLight};
use crate::model::AppModel;
use crate::model_buffers::{ModelsBuffers, ModelsBuffersStorage};
use std::sync::Arc;
use vulkano::device::Device;

pub struct Scene {
    pub models: Vec<AppModel>,
    pub global_light: DirectionLight,
    pub buffers: ModelsBuffersStorage,
}

impl Scene {
    pub fn new(device: Arc<Device>, models: Vec<AppModel>, global_light: DirectionLight) -> Self {
        Scene { models, global_light, buffers: ModelsBuffersStorage::init(device) }
    }

    pub fn frame_buffers(&self) -> ModelsBuffers {
        self.buffers.get_buffers(&self.models)
    }
}
