use nalgebra::{Point4, UnitQuaternion, Vector3};
use rencan_render::core::{
    light::{DirectionLight, LightInfo},
    Model, Scene,
};
use std::time::{Duration, Instant};
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};
use rencan_render::core::model::AppModel;

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_resizable(true);

    let app = rencan_ui::GuiApp::new(window, &event_loop);

    let mut frames = 0;
    let mut next = Instant::now() + Duration::from_secs(1);

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
    model.scaling = 3.0;

    let mut plane = Model::new(
        vec![
            [-0.4, -0.3, -0.4, 0.0].into(), // A
            [0.4, -0.3, -0.4, 0.0].into(),  // B
            [0.4, -0.3, 0.4, 0.0].into(),   // C
            [-0.4, -0.3, 0.4, 0.0].into(),  // D
        ],
        vec![[0, 2, 1, 0].into(), [0, 3, 2, 0].into(), [0, 1, 3, 0].into(), [1, 2, 3, 0].into()],
    );
    plane.scaling = 5.0;
    plane.position.y = -3.0;

    let models = vec![
        AppModel::new(model),
        AppModel::new(plane),
    ];

    let (rot_tx, rot_rx) = std::sync::mpsc::sync_channel(1000);

    let scene = Scene {
        models,
        global_light: DirectionLight::new(
            LightInfo::new(Point4::new(1.0, 1.0, 1.0, 0.0), 15.0),
            Vector3::new(0.0, -1.0, 0.0),
        ),
    };

    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_millis(10));
        if let Err(_) = rot_tx.send(UnitQuaternion::<f32>::from_euler_angles(-0.01, 0.0, 0.0)) {
            break;
        }
    });

    app.run(event_loop, scene, move |event, app, scene| {
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
