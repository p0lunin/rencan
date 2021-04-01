use crate::{
    light::{DirectionLight, PointLight},
    model::AppModel,
    model_buffers::{SceneBuffers, SceneBuffersStorage},
};
use std::sync::Arc;
use vulkano::device::Device;
use crate::model::SphereModel;

pub struct Scene {
    pub models: Vec<AppModel>,
    pub sphere_models: Vec<SphereModel>,
    pub global_light: DirectionLight,
    pub buffers: SceneBuffersStorage,
    pub point_lights: Vec<PointLight>,
}

impl Scene {
    pub fn new(
        device: Arc<Device>,
        models: Vec<AppModel>,
        sphere_models: Vec<SphereModel>,
        global_light: DirectionLight,
        point_lights: Vec<PointLight>,
    ) -> Self {
        Scene { models, sphere_models, global_light, buffers: SceneBuffersStorage::init(device), point_lights }
    }

    pub fn frame_buffers(&self) -> SceneBuffers {
        self.buffers.get_buffers(self)
    }
}
