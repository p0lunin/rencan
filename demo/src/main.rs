use nalgebra::{Point3, Point4, UnitQuaternion, Vector3};
use rencan_render::core::{
    light::{DirectionLight, LightInfo, PointLight},
    model::AppModel,
    Model, Scene,
};
use std::time::{Duration, Instant};
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

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
    let window = WindowBuilder::new().with_resizable(true);

    let app = rencan_ui::GuiApp::new(window, &event_loop);

    let mut frames = 0;
    let mut next = Instant::now() + Duration::from_secs(1);

    let mut models = Vec::new();

    for i in 0..3 {
        let model = make_pyramid(Point3::new((i * 5) as f32, 0.0, 0.0), 3.0);
        let plane = make_plane(Point3::new((i * 5) as f32, -1.8, 0.0), 5.0);
        models.push(model);
        models.push(plane);
    }

    let (rot_tx, rot_rx) = std::sync::mpsc::sync_channel(1000);

    let scene = Scene::new(
        app.device(),
        models,
        DirectionLight::new(
            LightInfo::new(Point4::new(1.0, 1.0, 1.0, 0.0), 15.0),
            Vector3::new(0.0, -1.0, 0.0),
        ),
        vec![PointLight::new(
            LightInfo::new(Point4::new(1.0, 1.0, 1.0, 0.0), 15.0),
            Point3::new(0.0, 0.0, 3.0),
        )],
    );

    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_millis(10));
        if let Err(_) = rot_tx.send(UnitQuaternion::<f32>::from_euler_angles(-0.01, 0.0, 0.0)) {
            break;
        }
    });

    app.run(event_loop, scene, 60, move |event, app, scene| {
        while let Ok(rot) = rot_rx.try_recv() {
            scene.global_light.direction = rot * &scene.global_light.direction;
        }
        frames += 1;
        if Instant::now() >= next {
            println!("fps: {}", frames);
            next = Instant::now() + Duration::from_secs(1);
            frames = 0;
        }
        match event {
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
            _ => {}
        };
    })
}
