use crate::{
    light::{DirectionLight, PointLight},
    model::{AppModel, SphereModel},
    model_buffers::{SceneBuffers, SceneBuffersStorage},
};
use std::sync::Arc;
use vulkano::device::Device;
use crate::setable::Mutable;
use vulkano::buffer::TypedBufferAccess;
use vulkano::descriptor::DescriptorSet;

pub struct Scene {
    pub data: SceneData,
    pub buffers: SceneBuffersStorage,
}

impl Scene {
    pub fn new(
        device: Arc<Device>,
        models: Vec<AppModel>,
        sphere_models: Vec<SphereModel>,
        global_light: DirectionLight,
        point_lights: Vec<PointLight>,
    ) -> Self {
        Scene {
            data: SceneData {
                models: Mutable::new(models),
                sphere_models: Mutable::new(sphere_models),
                global_light,
                point_lights,
            },
            buffers: SceneBuffersStorage::init(device),
        }
    }

    pub fn frame_buffers(&mut self) -> SceneBuffers {
        self.buffers.get_buffers(&mut self.data)
    }
}

pub struct SceneData {
    pub models: Mutable<Vec<AppModel>, Arc<dyn DescriptorSet + Send + Sync>>,
    pub sphere_models: Mutable<Vec<SphereModel>, Arc<dyn DescriptorSet + Send + Sync>>,
    pub global_light: DirectionLight,
    pub point_lights: Vec<PointLight>,
}
