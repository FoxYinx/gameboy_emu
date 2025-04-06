use crate::components::gameboy::Gameboy;
use pixels::{Pixels, SurfaceTexture};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};
use winit::event::ElementState;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Window;

pub const WIDTH: u32 = 160;
pub const HEIGHT: u32 = 144;

pub struct EmulatorApp<'a> {
    pixels: Pixels<'a>,
    rx_pixels: Receiver<Vec<u8>>,
    tx_inputs: Sender<u8>,
    _window: &'a Window,
    input_buffer: u8,
}

impl<'a> EmulatorApp<'a> {
    pub(crate) fn new(window: &'a Window, rom_path: &str) -> Self {
        let mut gameboy = Gameboy::new();
        gameboy.cartridge_to_rom(rom_path.to_string());

        let (tx_pixels, rx_pixels) = mpsc::channel();
        let (tx_inputs, rx_inputs) = mpsc::channel();

        thread::spawn(move || {
            let frame_duration = Duration::from_secs_f64(1.0 / 60.0);
            let cycles_per_frame = 69904;

            loop {
                let start_time = Instant::now();

                if let Ok(inputs) = rx_inputs.try_recv() {
                    gameboy.write_inputs(inputs);
                }

                while gameboy.cycles < cycles_per_frame {
                    gameboy.execute_cycle();
                }
                gameboy.cycles = 0;

                let mut pixels = vec![0; (WIDTH * HEIGHT * 4) as usize];
                gameboy.ppu.copy_to_framebuffer(&mut pixels);

                if tx_pixels.send(pixels).is_err() {
                    break;
                }

                let elapsed = start_time.elapsed();
                if elapsed < frame_duration {
                    thread::sleep(frame_duration - elapsed);
                }
            }
        });

        let surface_texture = SurfaceTexture::new(WIDTH, HEIGHT, window);
        let pixels =
            Pixels::new(WIDTH, HEIGHT, surface_texture).expect("Failed to create pixels context");

        Self {
            pixels,
            rx_pixels,
            tx_inputs,
            _window: window,
            input_buffer: 0xFF,
        }
    }

    pub(crate) fn update(&mut self) {
        if let Ok(new_pixels) = self.rx_pixels.try_recv() {
            self.pixels.frame_mut().copy_from_slice(&new_pixels);
        }
    }

    pub(crate) fn render(&mut self) -> Result<(), pixels::Error> {
        self.pixels.render()
    }

    pub(crate) fn update_inputs(&mut self, keycode: PhysicalKey, state: ElementState) {
        match keycode {
            PhysicalKey::Code(KeyCode::ArrowRight) | PhysicalKey::Code(KeyCode::KeyD) => {
                self.set_input_state(0b0000_0001, state)
            } // Right
            PhysicalKey::Code(KeyCode::ArrowLeft) | PhysicalKey::Code(KeyCode::KeyA) => {
                self.set_input_state(0b0000_0010, state)
            } // Left
            PhysicalKey::Code(KeyCode::ArrowUp) | PhysicalKey::Code(KeyCode::KeyW) => {
                self.set_input_state(0b0000_0100, state)
            } // Up
            PhysicalKey::Code(KeyCode::ArrowDown) | PhysicalKey::Code(KeyCode::KeyS) => {
                self.set_input_state(0b0000_1000, state)
            } // Down
            PhysicalKey::Code(KeyCode::KeyZ) => self.set_input_state(0b0001_0000, state), // A
            PhysicalKey::Code(KeyCode::KeyX) => self.set_input_state(0b0010_0000, state), // B
            PhysicalKey::Code(KeyCode::ShiftRight) => self.set_input_state(0b0100_0000, state), // Select
            PhysicalKey::Code(KeyCode::Enter) => self.set_input_state(0b1000_0000, state), // Start
            _ => (),
        }
    }

    fn set_input_state(&mut self, mask: u8, state: ElementState) {
        if state.is_pressed() {
            self.input_buffer &= !mask;
        } else {
            self.input_buffer |= mask;
        }
        self.tx_inputs.send(self.input_buffer).unwrap();
    }
}
