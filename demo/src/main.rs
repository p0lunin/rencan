mod models;

use nalgebra::{Point3, Point4, UnitQuaternion, Vector3};
use rencan_render::core::{
    light::{DirectionLight, LightInfo, PointLight},
    model::AppModel,
    Model, Scene,
};
use std::time::{Duration, Instant};
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit::dpi::{Size, PhysicalSize};

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

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_resizable(true).with_inner_size(Size::Physical(PhysicalSize::new(512, 512)));

    let mut app = rencan_ui::GuiApp::new(window, &event_loop);

    let mut frames = 0;
    let mut next = Instant::now() + Duration::from_secs(1);

    let mut models = models::make_desk(Point3::new(0.0, -1.5, 0.0), 3.0);
    models.push(models::make_room([0.0, 2.5, 0.0].into(), 5.0));
    models.push(models::make_mirror(
        Point3::new(-2.49, 0.0, 0.0),
        UnitQuaternion::from_euler_angles(0.0, std::f32::consts::FRAC_PI_2, 0.0),
        2.0)
    );
    models.push(models::make_mirror(
        Point3::new(2.49, 0.0, 0.0),
        UnitQuaternion::from_euler_angles(0.0, -std::f32::consts::FRAC_PI_2, 0.0),
        2.0)
    );
    /*
        for i in 0..20 {
            let model = make_pyramid(Point3::new((i * 5) as f32, 0.0, 0.0), 3.0);
            let plane = make_plane(Point3::new((i * 5) as f32, -1.8, 0.0), 5.0);
            models.push(model);
            models.push(plane);
        }
    */
    let (rot_tx, rot_rx) = std::sync::mpsc::sync_channel(1000);

    let mut scene = Scene::new(
        app.device(),
        models,
        DirectionLight::new(
            LightInfo::new(Point4::new(1.0, 0.98, 0.96, 0.0), 7.0),
            Vector3::new(0.2, -0.4, 0.3),
        ),
        vec![
            PointLight::new(
                LightInfo::new(Point4::new(0.8, 0.2, 0.0, 0.0), 100.0),
                Point3::new(0.0, 2.49, 0.0),
            ),
            PointLight::new(
                LightInfo::new(Point4::new(0.1, 0.9, 0.1, 0.0), 10.0),
                Point3::new(0.0, -2.0, 0.0),
            ),
        ],
    );

    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_millis(10));
        if let Err(_) = rot_tx.send(UnitQuaternion::<f32>::from_euler_angles(-0.01, 0.0, 0.0)) {
            break;
        }
    });

    let microseconds_per_frame = (1000_000.0 / 60.0) as u64;
    let frame_duration = Duration::from_micros(microseconds_per_frame);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::WaitUntil(Instant::now() + frame_duration);

        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent { event: WindowEvent::Resized(_), .. } => {
                app.reacreate_swapchain();
            }
            Event::RedrawEventsCleared => {
                //rx.recv().unwrap();
                while let Ok(rot) = rot_rx.try_recv() {
                    scene.global_light.direction = rot * &scene.global_light.direction;
                }
                frames += 1;
                if Instant::now() >= next {
                    println!("fps: {}", frames);
                    next = Instant::now() + Duration::from_secs(1);
                    frames = 0;
                }
                app.render_frame(&scene);
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
                let app = app.app_mut();
                match input.virtual_keycode.unwrap() {
                    VirtualKeyCode::Left => {
                        app.update_camera(|cam| cam.rotate(0.0, 0.05, 0.0));
                    }
                    VirtualKeyCode::Right => {
                        app.update_camera(|cam| cam.rotate(0.0, -0.05, 0.0));
                    }
                    VirtualKeyCode::Up => {
                        app.update_camera(|cam| cam.rotate(0.05, 0.0, 0.0));
                    }
                    VirtualKeyCode::Down => {
                        app.update_camera(|cam| cam.rotate(-0.05, 0.0, 0.0));
                    }
                    VirtualKeyCode::A => {
                        app.update_camera(|cam| cam.move_at(-0.05, 0.0, 0.0));
                    }
                    VirtualKeyCode::D => {
                        app.update_camera(|cam| cam.move_at(0.05, 0.0, 0.0));
                    }
                    VirtualKeyCode::W => {
                        app.update_camera(|cam| cam.move_at(0.0, 0.0, -0.05));
                    }
                    VirtualKeyCode::S => {
                        app.update_camera(|cam| cam.move_at(0.0, 0.0, 0.05));
                    }
                    _ => {}
                }
            }
            _ => (),
        }
    });
}
