use crate::components::gameboy::Gameboy;
use eframe::egui::Context;
use eframe::{egui, Frame};

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;

pub struct EmulatorApp {
    gameboy: Gameboy,
    pixels: Vec<u8>,
    texture: Option<egui::TextureHandle>
}

impl EmulatorApp {
    pub(crate) fn new(cc: &eframe::CreationContext<'_>, rom_path: String) -> Self {
        let mut gameboy = Gameboy::new();
        gameboy.cartridge_to_rom(rom_path);

        Self {
            gameboy,
            pixels: vec![0; WIDTH * HEIGHT * 4],
            texture: None,
        }
    }
}

impl eframe::App for EmulatorApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        for _ in 0..70224 {
            self.gameboy.execute_cycle();
        }

        self.gameboy.ppu.copy_to_framebuffer(&mut self.pixels);

        let texture = self.texture.get_or_insert_with(|| {
            ctx.load_texture(
                "gameboy-screen",
                egui::ColorImage::from_rgba_unmultiplied([WIDTH, HEIGHT], &self.pixels),
                Default::default(),
            )
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let size = texture.size_vec2();
            let sized_texture = egui::load::SizedTexture::new(texture, size);
            ui.add(egui::Image::new(sized_texture).fit_to_exact_size(size));
        });
    }
}