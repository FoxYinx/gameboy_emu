use crate::components::ppu::PpuMode::OamSearch;
use crate::window::emulator_app::{HEIGHT, WIDTH};

enum PpuMode {
    OamSearch,
    PixelTransfer,
    HBlank,
    VBlank,
}

pub struct PPU {
    mode: PpuMode,
    pub framebuffer: [u8; WIDTH * HEIGHT * 4],
    line: u8,
    mode_clock: u32
}

impl PPU {
    pub fn new() -> Self {
        PPU {
            mode: OamSearch,
            framebuffer: [0; WIDTH * HEIGHT * 4],
            line: 0,
            mode_clock: 0
        }
    }
    
    pub fn copy_to_framebuffer(&self, output: &mut [u8]) {
        output.copy_from_slice(&self.framebuffer);
    }

    pub fn render_test_pattern(&mut self) {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let idx = (y * 160 + x) * 4;
                self.framebuffer[idx] = x as u8;     // R
                self.framebuffer[idx + 1] = y as u8; // G
                self.framebuffer[idx + 2] = 0x7F;    // B
                self.framebuffer[idx + 3] = 0xFF;    // A
            }
        }
    }
}