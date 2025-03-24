mod components;
mod io;
mod utils;
mod window;

use std::sync::Arc;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use crate::window::emulator_app::{EmulatorApp, HEIGHT, WIDTH};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    let window = Arc::new(WindowBuilder::new()
        .with_title("Gameboy Emulator")
        .with_inner_size(winit::dpi::LogicalSize::new(WIDTH * 4, HEIGHT * 4))
        .build(&event_loop)?);

    let mut emulator_app = EmulatorApp::new(&window, "resources/roms/games/drmario.gb");

    let window_clone = Arc::clone(&window);
    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Wait);

        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                elwt.exit();
            },
            Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
                emulator_app.update();
                emulator_app.render().expect("Failed to render");
            },
            Event::AboutToWait => {
                window_clone.request_redraw();
            }
            _ => (),
        }
    })?;

    Ok(())
}
