mod models;

use nalgebra::{Point3, Point4, UnitQuaternion, Vector3};
use rencan_render::core::{
    camera::Camera,
    light::{DirectionLight, LightInfo, PointLight},
    model::{AppModel, SphereModel},
    Model, Scene,
};
use std::time::{Duration, Instant};
use winit::{
    dpi::{PhysicalSize, Size},
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use std::sync::Arc;
use vulkano::device::Device;

#[allow(unused)]
fn make_pyramid(position: Point3<f32>, scale: f32) -> AppModel {
    let mut model = Model::new(
        vec![
            [-0.4, -0.3, -0.4, 0.0].into(), // A
            [0.4, -0.3, -0.4, 0.0].into(),  // B
            [0.4, -0.3, 0.4, 0.0].into(),   // C
            [-0.4, -0.3, 0.4, 0.0].into(),  // D
            [0.0, 0.5, 0.0, 0.0].into(),    // O
        ],
        vec![
            [0, 4, 1, 0].into(),
            [1, 4, 2, 0].into(),
            [2, 4, 3, 0].into(),
            [3, 4, 0, 0].into(),
            [0, 1, 3, 0].into(),
            [1, 2, 3, 0].into(),
        ],
    );
    model.position = position;
    model.scaling = scale;

    AppModel::new(model)
}

#[allow(unused)]
fn make_plane(position: Point3<f32>, scale: f32) -> AppModel {
    let mut plane = Model::new(
        vec![
            [-0.4, 0.0, -0.4, 0.0].into(), // A
            [0.4, 0.0, -0.4, 0.0].into(),  // B
            [0.4, 0.0, 0.4, 0.0].into(),   // C
            [-0.4, 0.0, 0.4, 0.0].into(),  // D
        ],
        vec![[0, 2, 1, 0].into(), [0, 3, 2, 0].into(), [0, 1, 3, 0].into(), [1, 2, 3, 0].into()],
    );
    plane.position = position;
    plane.scaling = scale;

    AppModel::new(plane)
}

fn init_scene(device: Arc<Device>) -> Scene {
    let mut models = models::make_desk(Point3::new(0.0, -1.5, 0.0), 3.0);
    models.push(models::make_room([0.0, 2.5, 0.0].into(), 5.0));
    models.push(models::make_mirror(
        Point3::new(-4.99, 0.0, 0.0),
        UnitQuaternion::from_euler_angles(0.0, std::f32::consts::FRAC_PI_2, 0.0),
        2.0,
    ));
    models.push(models::make_mirror(
        Point3::new(4.99, 0.0, 0.0),
        UnitQuaternion::from_euler_angles(0.0, -std::f32::consts::FRAC_PI_2, 0.0),
        2.0,
    ));
    Scene::new(
        device,
        models,
        vec![SphereModel::new(Point3::new(0.0, -1.2, 0.0), 0.3)],
        DirectionLight::new(
            LightInfo::new(Point4::new(1.0, 0.98, 0.96, 0.0), 0.0),
            Vector3::new(0.2, -0.4, -0.3).normalize(),
        ),
        vec![
            PointLight::new(
                LightInfo::new(Point4::new(1.0, 0.98, 0.96, 0.0), 600.0),
                Point3::new(0.0, 2.3, 0.0),
            ),
        ],
        Camera::from_origin().move_at(4.185082,
                1.1902695,
                4.007931,).rotate(-0.24999996,
        0.8000001,
        0.0,),
    )
}

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_resizable(true)
        .with_inner_size(Size::Physical(PhysicalSize::new(512, 512)));

    let mut app = rencan_ui::GuiApp::new(window, &event_loop);

    let mut frames = 0;
    let mut next = Instant::now() + Duration::from_secs(1);
/*
    let mut models = models::make_desk(Point3::new(0.0, -1.5, 0.0), 3.0);
    models.push(models::make_room([0.0, 2.5, 0.0].into(), 5.0));
    models.push(models::make_mirror(
        Point3::new(-4.99, 0.0, 0.0),
        UnitQuaternion::from_euler_angles(0.0, std::f32::consts::FRAC_PI_2, 0.0),
        2.0,
    ));
    models.push(models::make_mirror(
        Point3::new(4.99, 0.0, 0.0),
        UnitQuaternion::from_euler_angles(0.0, -std::f32::consts::FRAC_PI_2, 0.0),
        2.0,
    ));

    let mut scene = Scene::new(
        app.device(),
        models,
        vec![
            SphereModel::new(Point3::new(0.0, -1.0, 0.0), 0.2), // TODO: vulkano bug
        ],
        DirectionLight::new(
            LightInfo::new(Point4::new(1.0, 0.98, 0.96, 0.0), 10.0),
            Vector3::new(0.2, -0.4, 0.3),
        ),
        vec![
            PointLight::new(
                LightInfo::new(Point4::new(0.8, 0.2, 0.0, 0.0), 1500.0),
                Point3::new(0.0, 2.49, 0.0),
            ),
            PointLight::new(
                LightInfo::new(Point4::new(0.1, 0.9, 0.1, 0.0), 300.0),
                Point3::new(0.0, -2.0, 0.0),
            ),
        ],
        Camera::from_origin().move_at(0.0, 0.0, 5.0),
    );
*/

    let (rot_tx, rot_rx) = std::sync::mpsc::sync_channel(1000);
    let mut scene = init_scene(app.device());
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_millis(10));
        if let Err(_) = rot_tx.send(UnitQuaternion::<f32>::from_euler_angles(-0.01, 0.0, 0.0)) {
            break;
        }
    });

    let microseconds_per_frame = (1000_000.0 / 60.0) as u64;
    let frame_duration = Duration::from_micros(microseconds_per_frame);

    event_loop.run(move |event, _, control_flow| {
        //*control_flow = ControlFlow::WaitUntil(Instant::now() + frame_duration);
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent { event: WindowEvent::Resized(_), .. } => {
                app.reacreate_swapchain();
            }
            Event::RedrawEventsCleared => {
                //rx.recv().unwrap();
                //while let Ok(rot) = rot_rx.try_recv() {
                //    scene.data.global_light.direction = rot *
                // &scene.data.global_light.direction;
                //}
                frames += 1;
                if Instant::now() >= next {
                    println!("fps: {}", frames);
                    next = Instant::now() + Duration::from_secs(1);
                    frames = 0;
                }
                app.render_frame(&mut scene);
            }
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input: input @ KeyboardInput { state: ElementState::Pressed, .. },
                        ..
                    },
                ..
            } => {
                println!("{:?}", input.virtual_keycode.as_ref());
                match input.virtual_keycode.unwrap() {
                    VirtualKeyCode::Left => {
                        scene.update_camera(|cam| cam.rotate(0.0, 0.05, 0.0));
                    }
                    VirtualKeyCode::Right => {
                        scene.update_camera(|cam| cam.rotate(0.0, -0.05, 0.0));
                    }
                    VirtualKeyCode::Up => {
                        scene.update_camera(|cam| cam.rotate(0.05, 0.0, 0.0));
                    }
                    VirtualKeyCode::Down => {
                        scene.update_camera(|cam| cam.rotate(-0.05, 0.0, 0.0));
                    }
                    VirtualKeyCode::A => {
                        scene.update_camera(|cam| cam.move_at(-0.05, 0.0, 0.0));
                    }
                    VirtualKeyCode::D => {
                        scene.update_camera(|cam| cam.move_at(0.05, 0.0, 0.0));
                    }
                    VirtualKeyCode::W => {
                        scene.update_camera(|cam| cam.move_at(0.0, 0.0, -0.05));
                    }
                    VirtualKeyCode::S => {
                        scene.update_camera(|cam| cam.move_at(0.0, 0.0, 0.05));
                    }
                    _ => {}
                }
                dbg!(&scene.data.camera);
            }
            _ => (),
        }
    });
}
