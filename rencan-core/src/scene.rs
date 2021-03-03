use crate::{
    light::{DirectionLight, PointLight},
    model::AppModel,
    model_buffers::{SceneBuffers, SceneBuffersStorage},
};
use std::sync::Arc;
use vulkano::device::Device;

pub struct Scene {
    pub models: Vec<AppModel>,
    pub global_light: DirectionLight,
    pub buffers: SceneBuffersStorage,
    pub point_lights: Vec<PointLight>,
}

impl Scene {
    pub fn new(
        device: Arc<Device>,
        models: Vec<AppModel>,
        global_light: DirectionLight,
        point_lights: Vec<PointLight>,
    ) -> Self {
        Scene { models, global_light, buffers: SceneBuffersStorage::init(device), point_lights }
    }

    pub fn frame_buffers(&self) -> SceneBuffers {
        self.buffers.get_buffers(self)
    }
}
