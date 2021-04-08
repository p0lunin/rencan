use rencan_animation::{AnimationApp, Renderer};
use rencan_render::core::{Screen, Scene};
use vulkano::device::Device;
use std::sync::Arc;
use rencan_render::core::model::SphereModel;
use rencan_render::core::light::{DirectionLight, LightInfo, PointLight};
use nalgebra::{Point3, Point4, Vector3};
use rencan_render::core::camera::Camera;

fn init_scene(device: Arc<Device>) -> Scene {
    Scene::new(
        device,
        vec![],
        vec![
            SphereModel::new(Point3::new(0.0, 0.0, 0.0), 0.5),
        ],
        DirectionLight::new(
            LightInfo::new(Point4::new(1.0, 0.98, 0.96, 0.0), 20.0),
            Vector3::new(0.2, -0.4, -0.3).normalize(),
        ),
        vec![
            PointLight::new(
                LightInfo::new(Point4::new(0.8, 0.2, 0.0, 0.0), 3000.0),
                Point3::new(0.0, 2.49, 0.0),
            ),
            PointLight::new(
                LightInfo::new(Point4::new(0.1, 0.9, 0.1, 0.0), 1500.0),
                Point3::new(0.0, -2.0, 0.0),
            ),
        ],
        Camera::from_origin().move_at(0.0, 0.0, 3.0)
    )
}

fn main() {
    let app = AnimationApp::new(
        Screen::new(1000, 1000),
        60
    );
    let device = app.vulkan_device();
    let mut renderer = Renderer::new(
        app,
        &"some.png"
    );
    let mut scene = init_scene(device);
    renderer.render_frame(
        &mut scene
    );
}