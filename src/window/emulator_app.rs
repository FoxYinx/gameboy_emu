use crate::components::gameboy::Gameboy;
use eframe::egui::Context;
use eframe::{egui, Frame};
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;

pub struct EmulatorApp {
    pixels: Vec<u8>,
    texture: Option<egui::TextureHandle>,
    rx: Receiver<Vec<u8>>
}

impl EmulatorApp {
    pub(crate) fn new(cc: &eframe::CreationContext<'_>, rom_path: String) -> Self {
        let mut gameboy = Gameboy::new();
        gameboy.cartridge_to_rom(rom_path);

        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            loop {
                for _ in 0..70224 {
                    gameboy.execute_cycle();
                }
                let mut pixels = vec![0; WIDTH * HEIGHT * 4];
                gameboy.ppu.copy_to_framebuffer(&mut pixels);
                tx.send(pixels).expect("Failed to send pixels");
                thread::sleep(Duration::from_millis(16));
            }
        });

        Self {
            pixels: vec![0; WIDTH * HEIGHT * 4],
            texture: None,
            rx
        }
    }
}

impl eframe::App for EmulatorApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        if let Ok(new_pixels) = self.rx.try_recv() {
            self.pixels = new_pixels;

            if let Some(texture) = &mut self.texture {
                texture.set(
                    egui::ColorImage::from_rgba_unmultiplied([WIDTH, HEIGHT], &self.pixels),
                    Default::default(),
                );
            } else {
                self.texture = Some(ctx.load_texture(
                    "gameboy-screen",
                    egui::ColorImage::from_rgba_unmultiplied([WIDTH, HEIGHT], &self.pixels),
                    Default::default(),
                ));
            }
        }
        
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(texture) = &self.texture {
                let size = texture.size_vec2();
                let sized_texture = egui::load::SizedTexture::new(texture, size);
                ui.add(egui::Image::new(sized_texture).fit_to_exact_size(size));
            }
        });
    }
}