use nalgebra::{Point3, Point4, Vector3, UnitQuaternion};
use rencan_animation::{AnimationApp, Renderer};
use rencan_render::core::{
    camera::Camera,
    light::{DirectionLight, LightInfo, PointLight},
    model::SphereModel,
    Scene, Screen,
};
use std::sync::Arc;
use vulkano::device::Device;

fn init_scene(device: Arc<Device>) -> Scene {
    Scene::new(
        device,
        vec![],
        vec![SphereModel::new(Point3::new(0.0, 0.0, 0.0), 0.5)],
        DirectionLight::new(
            LightInfo::new(Point4::new(1.0, 0.98, 0.96, 0.0), 20.0),
            Vector3::new(0.2, -0.4, -0.3).normalize(),
        ),
        vec![
            PointLight::new(
                LightInfo::new(Point4::new(0.8, 0.2, 0.0, 0.0), 0.0),
                Point3::new(0.0, 2.49, 0.0),
            ),
        ],
        Camera::from_origin().move_at(0.0, 0.0, 3.0),
    )
}

fn main() {
    let app = AnimationApp::new(Screen::new(1000, 1000));
    let device = app.vulkan_device();
    let mut renderer = Renderer::new(app, 60, &"video.mp4");
    let mut scene = init_scene(device);
    for i in 0..180 {
        println!("Render frame {}", i);
        renderer.render_frame_to_video(&mut scene);
        scene.data.global_light.direction =
            UnitQuaternion::from_euler_angles(1.0/60.0, 1.0/60.0, 1.0/60.0)
                * &scene.data.global_light.direction;
    }
    renderer.end_video();
}
