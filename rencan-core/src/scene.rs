use crate::{light::{DirectionLight, PointLight}, model::{AppModel, SphereModel}, model_buffers::{SceneBuffers, SceneBuffersStorage}, AppInfo};
use std::sync::Arc;
use vulkano::device::Device;
use crate::setable::Mutable;
use vulkano::buffer::TypedBufferAccess;
use vulkano::descriptor::DescriptorSet;
use vulkano::sync::GpuFuture;
use crate::camera::Camera;

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
        camera: Camera,
    ) -> Self {
        Scene {
            data: SceneData {
                models: Mutable::new(models),
                sphere_models: Mutable::new(sphere_models),
                global_light,
                point_lights,
                camera,
            },
            buffers: SceneBuffersStorage::init(device),
        }
    }

    pub fn frame_buffers(&mut self, app: &AppInfo) -> (SceneBuffers, Box<dyn GpuFuture>) {
        self.buffers.get_buffers(app, &mut self.data)
    }

    pub fn update_camera(&mut self, f: impl FnOnce(Camera) -> Camera) {
        self.data.camera = f(self.data.camera.clone());
    }
}

pub struct SceneData {
    pub models: Mutable<Vec<AppModel>, Arc<dyn DescriptorSet + Send + Sync>>,
    pub sphere_models: Mutable<Vec<SphereModel>, Arc<dyn DescriptorSet + Send + Sync>>,
    pub global_light: DirectionLight,
    pub point_lights: Vec<PointLight>,
    pub camera: Camera,
}
