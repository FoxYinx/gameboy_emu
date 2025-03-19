use crate::components::gameboy::Gameboy;
use pixels::{Pixels, SurfaceTexture};
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;
use winit::window::Window;

pub const WIDTH: u32 = 160;
pub const HEIGHT: u32 = 144;

pub struct EmulatorApp<'a> {
    pixels: Pixels<'a>,
    rx: Receiver<Vec<u8>>,
    _window: &'a Window
}

impl<'a> EmulatorApp<'a> {
    pub(crate) fn new(window: &'a Window, rom_path: &str) -> Self {
        let mut gameboy = Gameboy::new();
        gameboy.cartridge_to_rom(rom_path.to_string());

        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            loop {
                for _ in 0..70224 {
                    gameboy.execute_cycle();
                }

                let mut pixels = vec![0; (WIDTH * HEIGHT * 4) as usize];
                gameboy.ppu.copy_to_framebuffer(&mut pixels);

                if tx.send(pixels).is_err() {
                    break;
                }
                thread::sleep(Duration::from_millis(16));
            }
        });

        let surface_texture = SurfaceTexture::new(WIDTH, HEIGHT, window);
        let pixels = Pixels::new(WIDTH, HEIGHT, surface_texture)
            .expect("Failed to create pixels context");

        Self {
            pixels,
            rx,
            _window: window
        }
    }

    pub fn update(&mut self) {
        if let Ok(new_pixels) = self.rx.try_recv() {
            self.pixels.frame_mut().copy_from_slice(&new_pixels);
        }
    }

    pub fn render(&mut self) -> Result<(), pixels::Error> {
        self.pixels.render()
    }
}
