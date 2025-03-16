use crate::components::ppu::PpuMode::OamSearch;

enum PpuMode {
    OamSearch,
    PixelTransfer,
    HBlank,
    VBlank,
}

pub struct PPU {
    mode: PpuMode
}

impl PPU {
    pub fn new() -> Self {
        PPU {
            mode: OamSearch
        }
    }
}