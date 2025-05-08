mod components;
mod io;
mod utils;
mod window;

use crate::window::emulator_app::{EmulatorApp, HEIGHT, WIDTH};
use std::sync::Arc;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Gameboy Emulator")
            .with_inner_size(winit::dpi::LogicalSize::new(WIDTH * 4, HEIGHT * 4))
            .build(&event_loop)?,
    );

    let mut emulator_app = EmulatorApp::new(&window, "resources/roms/blargg/cpu_instrs/cpu_instrs.gb");

    let window_clone = Arc::clone(&window);
    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Wait);

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::KeyboardInput { event, .. } => {
                    let keycode = event.physical_key;
                    let pressed = event.state;
                    emulator_app.update_inputs(keycode, pressed);
                }
                WindowEvent::RedrawRequested => {
                    emulator_app.update();
                    emulator_app.render().expect("Failed to render");
                }
                _ => (),
            },
            Event::AboutToWait => {
                window_clone.request_redraw();
            }
            _ => (),
        }
    })?;

    Ok(())
}
