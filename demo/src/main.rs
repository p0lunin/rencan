use std::time::{Duration, Instant};
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_resizable(true);

    let app = rencan_ui::GuiApp::new(window, &event_loop);

    let mut frames = 0;
    let mut next = Instant::now() + Duration::from_secs(1);
    app.run(event_loop, move |event, app| {
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
            } => match input.virtual_keycode.unwrap() {
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
            },
            _ => {}
        }
    })
}
